use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::types::*;

#[derive(Debug, sqlx::FromRow)]
struct LogEntryRow {
    id: Uuid,
    level: String,
    message: String,
    source: String,
    service: String,
    trace_id: Option<String>,
    span_id: Option<String>,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl From<LogEntryRow> for LogEntry {
    fn from(row: LogEntryRow) -> Self {
        LogEntry {
            id: row.id,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            message: row.message,
            source: row.source,
            service: row.service,
            trace_id: row.trace_id,
            span_id: row.span_id,
            metadata: row.metadata,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct RetentionPolicyRow {
    id: Uuid,
    service: String,
    level: String,
    retention_days: i32,
    enabled: bool,
    created_at: DateTime<Utc>,
}

impl From<RetentionPolicyRow> for LogRetentionPolicy {
    fn from(row: RetentionPolicyRow) -> Self {
        LogRetentionPolicy {
            id: row.id,
            service: row.service,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            retention_days: row.retention_days,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

pub struct LogAggregationService {
    pool: PgPool,
}

impl LogAggregationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn ingest(
        &self,
        input: CreateLogEntry,
    ) -> Result<LogEntry, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryRow>(
            r#"INSERT INTO log_entries_v2 (level, message, source, service, trace_id, span_id, metadata)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, level, message, source, service, trace_id, span_id, metadata, created_at"#,
        )
        .bind(input.level.to_string())
        .bind(&input.message)
        .bind(&input.source)
        .bind(input.service.as_deref().unwrap_or("civitforge"))
        .bind(&input.trace_id)
        .bind(&input.span_id)
        .bind(input.metadata.unwrap_or(serde_json::json!({})))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn search(
        &self,
        filter: LogSearchFilter,
    ) -> Result<LogSearchResult, sqlx::Error> {
        let limit = filter.limit.unwrap_or(100).min(1000);
        let offset = filter.offset.unwrap_or(0);

        let level_str = filter.level.map(|l| l.to_string());
        let search_pattern = filter.search.map(|s| format!("%{}%", s));

        let total_count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM log_entries_v2
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::timestamptz IS NULL OR created_at >= $6)
             AND ($7::timestamptz IS NULL OR created_at <= $7)"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.since)
        .bind(filter.until)
        .fetch_one(&self.pool)
        .await?;

        let entries = sqlx::query_as::<_, LogEntryRow>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, created_at FROM log_entries_v2
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::timestamptz IS NULL OR created_at >= $6)
             AND ($7::timestamptz IS NULL OR created_at <= $7)
             ORDER BY created_at DESC LIMIT $8 OFFSET $9"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.since)
        .bind(filter.until)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(LogSearchResult {
            entries: entries.into_iter().map(|r| r.into()).collect(),
            total_count,
        })
    }

    pub async fn get_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<LogEntry>, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryRow>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, created_at
             FROM log_entries_v2 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn delete_old_entries(
        &self,
        max_age_days: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM log_entries_v2 WHERE created_at < NOW() - make_interval(days => $1::int)",
        )
        .bind(max_age_days)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    pub async fn export_logs(
        &self,
        request: LogExportRequest,
    ) -> Result<LogExportResult, sqlx::Error> {
        let result = self.search(request.filter).await?;

        Ok(LogExportResult {
            entries: result.entries,
            format: request.format,
            exported_at: Utc::now(),
        })
    }

    pub async fn get_log_stats(
        &self,
    ) -> Result<LogServiceStats, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct LevelCount {
            level: String,
            count: i64,
        }

        let rows = sqlx::query_as::<_, LevelCount>(
            r#"SELECT level, COUNT(*) as count FROM log_entries_v2 GROUP BY level ORDER BY count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let total = rows.iter().map(|r| r.count).sum();

        let level_counts: std::collections::HashMap<String, i64> = rows
            .into_iter()
            .map(|r| (r.level, r.count))
            .collect();

        #[derive(Debug, sqlx::FromRow)]
        struct ServiceCount {
            service: String,
            count: i64,
        }

        let svc_rows = sqlx::query_as::<_, ServiceCount>(
            r#"SELECT service, COUNT(*) as count FROM log_entries_v2 GROUP BY service ORDER BY count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let service_counts: std::collections::HashMap<String, i64> = svc_rows
            .into_iter()
            .map(|r| (r.service, r.count))
            .collect();

        Ok(LogServiceStats {
            total_entries: total,
            level_counts,
            service_counts,
        })
    }

    pub async fn search_by_trace(
        &self,
        trace_id: &str,
    ) -> Result<Vec<LogEntry>, sqlx::Error> {
        let rows = sqlx::query_as::<_, LogEntryRow>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, created_at
             FROM log_entries_v2 WHERE trace_id = $1
             ORDER BY created_at ASC"#,
        )
        .bind(trace_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_retention_policy(
        &self,
        input: CreateLogRetentionPolicy,
    ) -> Result<LogRetentionPolicy, sqlx::Error> {
        let row = sqlx::query_as::<_, RetentionPolicyRow>(
            r#"INSERT INTO log_retention_policies (service, level, retention_days, enabled)
             VALUES ($1, $2, $3, $4)
             RETURNING id, service, level, retention_days, enabled, created_at"#,
        )
        .bind(&input.service)
        .bind(input.level.to_string())
        .bind(input.retention_days.unwrap_or(30))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn list_retention_policies(
        &self,
    ) -> Result<Vec<LogRetentionPolicy>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RetentionPolicyRow>(
            r#"SELECT id, service, level, retention_days, enabled, created_at
             FROM log_retention_policies ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_retention_policy(
        &self,
        id: Uuid,
        input: UpdateLogRetentionPolicy,
    ) -> Result<LogRetentionPolicy, sqlx::Error> {
        let row = sqlx::query_as::<_, RetentionPolicyRow>(
            r#"UPDATE log_retention_policies SET
             service = COALESCE($2, service),
             level = COALESCE($3, level),
             retention_days = COALESCE($4, retention_days),
             enabled = COALESCE($5, enabled)
             WHERE id = $1
             RETURNING id, service, level, retention_days, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.service)
        .bind(input.level.map(|l| l.to_string()))
        .bind(input.retention_days)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_retention_policy(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM log_retention_policies WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn apply_retention_policies(
        &self,
    ) -> Result<i64, sqlx::Error> {
        let policies = sqlx::query_as::<_, RetentionPolicyRow>(
            r#"SELECT id, service, level, retention_days, enabled, created_at
             FROM log_retention_policies WHERE enabled = true"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut total_deleted: i64 = 0;

        for policy in policies {
            let result = sqlx::query(
                r#"DELETE FROM log_entries_v2
                 WHERE service = $1 AND level = $2
                 AND created_at < NOW() - make_interval(days => $3::int)"#,
            )
            .bind(&policy.service)
            .bind(&policy.level)
            .bind(policy.retention_days)
            .execute(&self.pool)
            .await?;

            total_deleted += result.rows_affected() as i64;
        }

        Ok(total_deleted)
    }

    // V3: Full-text search and correlation methods

    pub async fn ingest_v3(
        &self,
        input: CreateLogEntryV3,
    ) -> Result<LogEntryV3, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryV3Row>(
            r#"INSERT INTO log_entries_v3 (level, message, source, service, trace_id, span_id, metadata, retention_days)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, level, message, source, service, trace_id, span_id, metadata, retention_days, created_at"#,
        )
        .bind(input.level.to_string())
        .bind(&input.message)
        .bind(&input.source)
        .bind(input.service.as_deref().unwrap_or("civitforge"))
        .bind(&input.trace_id)
        .bind(&input.span_id)
        .bind(input.metadata.unwrap_or(serde_json::json!({})))
        .bind(input.retention_days.unwrap_or(30))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn search_v3(
        &self,
        filter: LogSearchFilterV3,
    ) -> Result<LogSearchResultV3, sqlx::Error> {
        let limit = filter.limit.unwrap_or(100).min(1000);
        let offset = filter.offset.unwrap_or(0);

        let level_str = filter.level.map(|l| l.to_string());
        let search_pattern = filter.search.map(|s| format!("%{}%", s));

        let total_count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM log_entries_v3
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::timestamptz IS NULL OR created_at >= $6)
             AND ($7::timestamptz IS NULL OR created_at <= $7)"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.since)
        .bind(filter.until)
        .fetch_one(&self.pool)
        .await?;

        let entries = sqlx::query_as::<_, LogEntryV3Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, created_at FROM log_entries_v3
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::timestamptz IS NULL OR created_at >= $6)
             AND ($7::timestamptz IS NULL OR created_at <= $7)
             ORDER BY created_at DESC LIMIT $8 OFFSET $9"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.since)
        .bind(filter.until)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(LogSearchResultV3 {
            entries: entries.into_iter().map(|r| r.into()).collect(),
            total_count,
        })
    }

    pub async fn full_text_search_v3(
        &self,
        query: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<LogSearchResultV3, sqlx::Error> {
        let limit = limit.unwrap_or(100).min(1000);
        let offset = offset.unwrap_or(0);

        let total_count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM log_search_index
             WHERE search_vector @@ plainto_tsquery('english', $1)"#,
        )
        .bind(query)
        .fetch_one(&self.pool)
        .await?;

        let entries = sqlx::query_as::<_, LogEntryV3Row>(
            r#"SELECT le.id, le.level, le.message, le.source, le.service, le.trace_id, le.span_id, le.metadata, le.retention_days, le.created_at
             FROM log_entries_v3 le
             INNER JOIN log_search_index lsi ON le.id = lsi.log_id
             WHERE lsi.search_vector @@ plainto_tsquery('english', $1)
             ORDER BY le.created_at DESC LIMIT $2 OFFSET $3"#,
        )
        .bind(query)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(LogSearchResultV3 {
            entries: entries.into_iter().map(|r| r.into()).collect(),
            total_count,
        })
    }

    pub async fn correlate_by_trace(
        &self,
        trace_id: &str,
    ) -> Result<LogCorrelation, sqlx::Error> {
        let entries = sqlx::query_as::<_, LogEntryV3Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, created_at
             FROM log_entries_v3 WHERE trace_id = $1
             ORDER BY created_at ASC"#,
        )
        .bind(trace_id)
        .fetch_all(&self.pool)
        .await?;

        let entry_count = entries.len() as i64;
        let service_count = entries
            .iter()
            .map(|e| &e.service)
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;

        Ok(LogCorrelation {
            trace_id: trace_id.to_string(),
            entries: entries.into_iter().map(|r| r.into()).collect(),
            service_count,
            entry_count,
        })
    }

    pub async fn enforce_retention_v3(
        &self,
        max_age_days: Option<i32>,
    ) -> Result<i64, sqlx::Error> {
        let days = max_age_days.unwrap_or(30);
        let result = sqlx::query(
            r#"DELETE FROM log_entries_v3 WHERE created_at < NOW() - make_interval(days => $1::int)"#,
        )
        .bind(days)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    pub async fn get_by_id_v3(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<LogEntryV3>, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryV3Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, created_at
             FROM log_entries_v3 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn delete_old_entries_v3(
        &self,
        max_age_days: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM log_entries_v3 WHERE created_at < NOW() - make_interval(days => $1::int)",
        )
        .bind(max_age_days)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    // V4: Log aggregation with indexing and alert rules

    pub async fn ingest_v4(
        &self,
        input: CreateLogEntryV4,
    ) -> Result<LogEntryV4, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryV4Row>(
            r#"INSERT INTO log_entries_v4 (level, message, source, service, trace_id, span_id, metadata, retention_days)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at"#,
        )
        .bind(input.level.to_string())
        .bind(&input.message)
        .bind(&input.source)
        .bind(input.service.as_deref().unwrap_or("civitforge"))
        .bind(&input.trace_id)
        .bind(&input.span_id)
        .bind(input.metadata.unwrap_or(serde_json::json!({})))
        .bind(input.retention_days.unwrap_or(30))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn search_v4(
        &self,
        filter: LogSearchFilterV4,
    ) -> Result<LogSearchResultV4, sqlx::Error> {
        let limit = filter.limit.unwrap_or(100).min(1000);
        let offset = filter.offset.unwrap_or(0);

        let level_str = filter.level.map(|l| l.to_string());
        let search_pattern = filter.search.map(|s| format!("%{}%", s));

        let total_count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM log_entries_v4
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::bool IS NULL OR indexed = $6)
             AND ($7::timestamptz IS NULL OR created_at >= $7)
             AND ($8::timestamptz IS NULL OR created_at <= $8)"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.indexed)
        .bind(filter.since)
        .bind(filter.until)
        .fetch_one(&self.pool)
        .await?;

        let entries = sqlx::query_as::<_, LogEntryV4Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at FROM log_entries_v4
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::bool IS NULL OR indexed = $6)
             AND ($7::timestamptz IS NULL OR created_at >= $7)
             AND ($8::timestamptz IS NULL OR created_at <= $8)
             ORDER BY created_at DESC LIMIT $9 OFFSET $10"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.indexed)
        .bind(filter.since)
        .bind(filter.until)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(LogSearchResultV4 {
            entries: entries.into_iter().map(|r| r.into()).collect(),
            total_count,
        })
    }

    pub async fn get_by_id_v4(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<LogEntryV4>, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryV4Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at
             FROM log_entries_v4 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn mark_indexed_v4(
        &self,
        id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE log_entries_v4 SET indexed = true WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_old_entries_v4(
        &self,
        max_age_days: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM log_entries_v4 WHERE created_at < NOW() - make_interval(days => $1::int)",
        )
        .bind(max_age_days)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    // V4: Alert rules

    pub async fn create_alert_rule(
        &self,
        input: CreateLogAlertRule,
    ) -> Result<LogAlertRule, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleRow>(
            r#"INSERT INTO log_alert_rules (name, level, pattern, threshold, window_seconds, enabled)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, level, pattern, threshold, window_seconds, enabled, created_at"#,
        )
        .bind(&input.name)
        .bind(input.level.to_string())
        .bind(&input.pattern)
        .bind(input.threshold.unwrap_or(10))
        .bind(input.window_seconds.unwrap_or(300))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_alert_rule(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<LogAlertRule>, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleRow>(
            r#"SELECT id, name, level, pattern, threshold, window_seconds, enabled, created_at
             FROM log_alert_rules WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_alert_rules(&self) -> Result<Vec<LogAlertRule>, sqlx::Error> {
        let rows = sqlx::query_as::<_, LogAlertRuleRow>(
            r#"SELECT id, name, level, pattern, threshold, window_seconds, enabled, created_at
             FROM log_alert_rules ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_alert_rule(
        &self,
        id: uuid::Uuid,
        input: UpdateLogAlertRule,
    ) -> Result<LogAlertRule, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleRow>(
            r#"UPDATE log_alert_rules SET
             name = COALESCE($2, name),
             level = COALESCE($3, level),
             pattern = COALESCE($4, pattern),
             threshold = COALESCE($5, threshold),
             window_seconds = COALESCE($6, window_seconds),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, name, level, pattern, threshold, window_seconds, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(input.level.map(|l| l.to_string()))
        .bind(&input.pattern)
        .bind(input.threshold)
        .bind(input.window_seconds)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_alert_rule(
        &self,
        id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM log_alert_rules WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn check_alert_rule(
        &self,
        id: uuid::Uuid,
    ) -> Result<(LogAlertRule, i64), sqlx::Error> {
        let rule = self.get_alert_rule(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;

        let count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM log_entries_v4
             WHERE level = $1 AND message ILIKE $2
             AND created_at >= NOW() - make_interval(seconds => $3::int)"#,
        )
        .bind(rule.level.to_string())
        .bind(&rule.pattern)
        .bind(rule.window_seconds)
        .fetch_one(&self.pool)
        .await?;

        Ok((rule, count))
    }

    // V5: Log entries with enhanced indexing and alert rules v2

    pub async fn ingest_v5(
        &self,
        input: CreateLogEntryV5,
    ) -> Result<LogEntryV5, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryV5Row>(
            r#"INSERT INTO log_entries_v5 (level, message, source, service, trace_id, span_id, metadata, retention_days)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at"#,
        )
        .bind(input.level.to_string())
        .bind(&input.message)
        .bind(&input.source)
        .bind(input.service.as_deref().unwrap_or("civitforge"))
        .bind(&input.trace_id)
        .bind(&input.span_id)
        .bind(input.metadata.unwrap_or(serde_json::json!({})))
        .bind(input.retention_days.unwrap_or(30))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn search_v5(
        &self,
        filter: LogSearchFilterV5,
    ) -> Result<LogSearchResultV5, sqlx::Error> {
        let limit = filter.limit.unwrap_or(100).min(1000);
        let offset = filter.offset.unwrap_or(0);
        let level_str = filter.level.map(|l| l.to_string());
        let search_pattern = filter.search.map(|s| format!("%{}%", s));

        let total_count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM log_entries_v5
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::bool IS NULL OR indexed = $6)
             AND ($7::timestamptz IS NULL OR created_at >= $7)
             AND ($8::timestamptz IS NULL OR created_at <= $8)"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.indexed)
        .bind(filter.since)
        .bind(filter.until)
        .fetch_one(&self.pool)
        .await?;

        let entries = sqlx::query_as::<_, LogEntryV5Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at FROM log_entries_v5
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::bool IS NULL OR indexed = $6)
             AND ($7::timestamptz IS NULL OR created_at >= $7)
             AND ($8::timestamptz IS NULL OR created_at <= $8)
             ORDER BY created_at DESC LIMIT $9 OFFSET $10"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.indexed)
        .bind(filter.since)
        .bind(filter.until)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(LogSearchResultV5 {
            entries: entries.into_iter().map(|r| r.into()).collect(),
            total_count,
        })
    }

    pub async fn get_by_id_v5(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<LogEntryV5>, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryV5Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at
             FROM log_entries_v5 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn mark_indexed_v5(
        &self,
        id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE log_entries_v5 SET indexed = true WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_old_entries_v5(
        &self,
        max_age_days: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM log_entries_v5 WHERE created_at < NOW() - make_interval(days => $1::int)",
        )
        .bind(max_age_days)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() as i64)
    }

    // V5: Alert rules v2

    pub async fn create_alert_rule_v2(
        &self,
        input: CreateLogAlertRuleV2,
    ) -> Result<LogAlertRuleV2, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV2Row>(
            r#"INSERT INTO log_alert_rules_v2 (name, level, pattern, threshold, window_seconds, enabled)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, level, pattern, threshold, window_seconds, enabled, created_at"#,
        )
        .bind(&input.name)
        .bind(input.level.to_string())
        .bind(&input.pattern)
        .bind(input.threshold.unwrap_or(10))
        .bind(input.window_seconds.unwrap_or(300))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_alert_rule_v2(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<LogAlertRuleV2>, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV2Row>(
            r#"SELECT id, name, level, pattern, threshold, window_seconds, enabled, created_at
             FROM log_alert_rules_v2 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn list_alert_rules_v2(
        &self,
    ) -> Result<Vec<LogAlertRuleV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, LogAlertRuleV2Row>(
            r#"SELECT id, name, level, pattern, threshold, window_seconds, enabled, created_at
             FROM log_alert_rules_v2 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_alert_rule_v2(
        &self,
        id: uuid::Uuid,
        input: UpdateLogAlertRuleV2,
    ) -> Result<LogAlertRuleV2, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV2Row>(
            r#"UPDATE log_alert_rules_v2 SET
             name = COALESCE($2, name),
             level = COALESCE($3, level),
             pattern = COALESCE($4, pattern),
             threshold = COALESCE($5, threshold),
             window_seconds = COALESCE($6, window_seconds),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, name, level, pattern, threshold, window_seconds, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(input.level.map(|l| l.to_string()))
        .bind(&input.pattern)
        .bind(input.threshold)
        .bind(input.window_seconds)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn delete_alert_rule_v2(
        &self,
        id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM log_alert_rules_v2 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn check_alert_rule_v2(
        &self,
        id: uuid::Uuid,
    ) -> Result<(LogAlertRuleV2, i64), sqlx::Error> {
        let rule = self.get_alert_rule_v2(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;
        let count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM log_entries_v5
             WHERE level = $1 AND message ILIKE $2
             AND created_at >= NOW() - make_interval(seconds => $3::int)"#,
        )
        .bind(rule.level.to_string())
        .bind(&rule.pattern)
        .bind(rule.window_seconds)
        .fetch_one(&self.pool)
        .await?;
        Ok((rule, count))
    }

    pub async fn check_all_alert_rules_v2(
        &self,
    ) -> Result<Vec<(LogAlertRuleV2, i64, bool)>, sqlx::Error> {
        let rules = self.list_alert_rules_v2().await?;
        let mut results = Vec::new();
        for rule in rules {
            if !rule.enabled {
                continue;
            }
            let count = sqlx::query_scalar::<_, i64>(
                r#"SELECT COUNT(*) FROM log_entries_v5
                 WHERE level = $1 AND message ILIKE $2
                 AND created_at >= NOW() - make_interval(seconds => $3::int)"#,
            )
            .bind(rule.level.to_string())
            .bind(&rule.pattern)
            .bind(rule.window_seconds)
            .fetch_one(&self.pool)
            .await?;
            let triggered = count >= rule.threshold as i64;
            results.push((rule, count, triggered));
        }
        Ok(results)
    }

    // V6: Log entries with enhanced indexing and alert rules v3

    pub async fn ingest_v6(
        &self,
        input: CreateLogEntryV6,
    ) -> Result<LogEntryV6, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryV6Row>(
            r#"INSERT INTO log_entries_v6 (level, message, source, service, trace_id, span_id, metadata, retention_days)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at"#,
        )
        .bind(input.level.to_string())
        .bind(&input.message)
        .bind(&input.source)
        .bind(input.service.as_deref().unwrap_or("civitforge"))
        .bind(&input.trace_id)
        .bind(&input.span_id)
        .bind(input.metadata.unwrap_or(serde_json::json!({})))
        .bind(input.retention_days.unwrap_or(30))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn search_v6(
        &self,
        filter: LogSearchFilterV6,
    ) -> Result<LogSearchResultV6, sqlx::Error> {
        let limit = filter.limit.unwrap_or(100).min(1000);
        let offset = filter.offset.unwrap_or(0);
        let level_str = filter.level.map(|l| l.to_string());
        let search_pattern = filter.search.map(|s| format!("%{}%", s));

        let total_count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM log_entries_v6
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::bool IS NULL OR indexed = $6)
             AND ($7::timestamptz IS NULL OR created_at >= $7)
             AND ($8::timestamptz IS NULL OR created_at <= $8)"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.indexed)
        .bind(filter.since)
        .bind(filter.until)
        .fetch_one(&self.pool)
        .await?;

        let entries = sqlx::query_as::<_, LogEntryV6Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at FROM log_entries_v6
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::bool IS NULL OR indexed = $6)
             AND ($7::timestamptz IS NULL OR created_at >= $7)
             AND ($8::timestamptz IS NULL OR created_at <= $8)
             ORDER BY created_at DESC LIMIT $9 OFFSET $10"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.indexed)
        .bind(filter.since)
        .bind(filter.until)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(LogSearchResultV6 {
            entries: entries.into_iter().map(|r| r.into()).collect(),
            total_count,
        })
    }

    pub async fn get_by_id_v6(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<LogEntryV6>, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryV6Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at
             FROM log_entries_v6 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn mark_indexed_v6(
        &self,
        id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE log_entries_v6 SET indexed = true WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_old_entries_v6(
        &self,
        max_age_days: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM log_entries_v6 WHERE created_at < NOW() - make_interval(days => $1::int)",
        )
        .bind(max_age_days)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() as i64)
    }

    // V6: Alert rules v3

    pub async fn create_alert_rule_v3(
        &self,
        input: CreateLogAlertRuleV3,
    ) -> Result<LogAlertRuleV3, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV3Row>(
            r#"INSERT INTO log_alert_rules_v3 (name, level, pattern, threshold, window_seconds, enabled)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, level, pattern, threshold, window_seconds, enabled, created_at"#,
        )
        .bind(&input.name)
        .bind(input.level.to_string())
        .bind(&input.pattern)
        .bind(input.threshold.unwrap_or(10))
        .bind(input.window_seconds.unwrap_or(300))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_alert_rule_v3(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<LogAlertRuleV3>, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV3Row>(
            r#"SELECT id, name, level, pattern, threshold, window_seconds, enabled, created_at
             FROM log_alert_rules_v3 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn list_alert_rules_v3(
        &self,
    ) -> Result<Vec<LogAlertRuleV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, LogAlertRuleV3Row>(
            r#"SELECT id, name, level, pattern, threshold, window_seconds, enabled, created_at
             FROM log_alert_rules_v3 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_alert_rule_v3(
        &self,
        id: uuid::Uuid,
        input: UpdateLogAlertRuleV3,
    ) -> Result<LogAlertRuleV3, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV3Row>(
            r#"UPDATE log_alert_rules_v3 SET
             name = COALESCE($2, name),
             level = COALESCE($3, level),
             pattern = COALESCE($4, pattern),
             threshold = COALESCE($5, threshold),
             window_seconds = COALESCE($6, window_seconds),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, name, level, pattern, threshold, window_seconds, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(input.level.map(|l| l.to_string()))
        .bind(&input.pattern)
        .bind(input.threshold)
        .bind(input.window_seconds)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn delete_alert_rule_v3(
        &self,
        id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM log_alert_rules_v3 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn check_alert_rule_v3(
        &self,
        id: uuid::Uuid,
    ) -> Result<(LogAlertRuleV3, i64), sqlx::Error> {
        let rule = self.get_alert_rule_v3(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;
        let count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM log_entries_v6
             WHERE level = $1 AND message ILIKE $2
             AND created_at >= NOW() - make_interval(seconds => $3::int)"#,
        )
        .bind(rule.level.to_string())
        .bind(&rule.pattern)
        .bind(rule.window_seconds)
        .fetch_one(&self.pool)
        .await?;
        Ok((rule, count))
    }

    pub async fn check_all_alert_rules_v3(
        &self,
    ) -> Result<Vec<(LogAlertRuleV3, i64, bool)>, sqlx::Error> {
        let rules = self.list_alert_rules_v3().await?;
        let mut results = Vec::new();
        for rule in rules {
            if !rule.enabled {
                continue;
            }
            let count = sqlx::query_scalar::<_, i64>(
                r#"SELECT COUNT(*) FROM log_entries_v6
                 WHERE level = $1 AND message ILIKE $2
                 AND created_at >= NOW() - make_interval(seconds => $3::int)"#,
            )
            .bind(rule.level.to_string())
            .bind(&rule.pattern)
            .bind(rule.window_seconds)
            .fetch_one(&self.pool)
            .await?;
            let triggered = count >= rule.threshold as i64;
            results.push((rule, count, triggered));
        }
        Ok(results)
    }

    // V7: Log entries with enhanced indexing and alert rules v4

    pub async fn ingest_v7(
        &self,
        input: CreateLogEntryV7,
    ) -> Result<LogEntryV7, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryV7Row>(
            r#"INSERT INTO log_entries_v7 (level, message, source, service, trace_id, span_id, metadata, retention_days)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at"#,
        )
        .bind(input.level.to_string())
        .bind(&input.message)
        .bind(&input.source)
        .bind(input.service.as_deref().unwrap_or("civitforge"))
        .bind(&input.trace_id)
        .bind(&input.span_id)
        .bind(input.metadata.unwrap_or(serde_json::json!({})))
        .bind(input.retention_days.unwrap_or(30))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn search_v7(
        &self,
        filter: LogSearchFilterV7,
    ) -> Result<LogSearchResultV7, sqlx::Error> {
        let limit = filter.limit.unwrap_or(100).min(1000);
        let offset = filter.offset.unwrap_or(0);
        let level_str = filter.level.map(|l| l.to_string());
        let search_pattern = filter.search.map(|s| format!("%{}%", s));

        let total_count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM log_entries_v7
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::bool IS NULL OR indexed = $6)
             AND ($7::timestamptz IS NULL OR created_at >= $7)
             AND ($8::timestamptz IS NULL OR created_at <= $8)"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.indexed)
        .bind(filter.since)
        .bind(filter.until)
        .fetch_one(&self.pool)
        .await?;

        let entries = sqlx::query_as::<_, LogEntryV7Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at FROM log_entries_v7
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::bool IS NULL OR indexed = $6)
             AND ($7::timestamptz IS NULL OR created_at >= $7)
             AND ($8::timestamptz IS NULL OR created_at <= $8)
             ORDER BY created_at DESC LIMIT $9 OFFSET $10"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.indexed)
        .bind(filter.since)
        .bind(filter.until)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(LogSearchResultV7 {
            entries: entries.into_iter().map(|r| r.into()).collect(),
            total_count,
        })
    }

    pub async fn get_by_id_v7(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<LogEntryV7>, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryV7Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at
             FROM log_entries_v7 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn mark_indexed_v7(
        &self,
        id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE log_entries_v7 SET indexed = true WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_old_entries_v7(
        &self,
        max_age_days: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM log_entries_v7 WHERE created_at < NOW() - make_interval(days => $1::int)",
        )
        .bind(max_age_days)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() as i64)
    }

    // V7: Alert rules v4

    pub async fn create_alert_rule_v4(
        &self,
        input: CreateLogAlertRuleV4,
    ) -> Result<LogAlertRuleV4, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV4Row>(
            r#"INSERT INTO log_alert_rules_v4 (name, level, pattern, threshold, window_seconds, enabled)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, level, pattern, threshold, window_seconds, enabled, created_at"#,
        )
        .bind(&input.name)
        .bind(input.level.to_string())
        .bind(&input.pattern)
        .bind(input.threshold.unwrap_or(10))
        .bind(input.window_seconds.unwrap_or(300))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_alert_rule_v4(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<LogAlertRuleV4>, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV4Row>(
            r#"SELECT id, name, level, pattern, threshold, window_seconds, enabled, created_at
             FROM log_alert_rules_v4 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn list_alert_rules_v4(
        &self,
    ) -> Result<Vec<LogAlertRuleV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, LogAlertRuleV4Row>(
            r#"SELECT id, name, level, pattern, threshold, window_seconds, enabled, created_at
             FROM log_alert_rules_v4 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_alert_rule_v4(
        &self,
        id: uuid::Uuid,
        input: UpdateLogAlertRuleV4,
    ) -> Result<LogAlertRuleV4, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV4Row>(
            r#"UPDATE log_alert_rules_v4 SET
             name = COALESCE($2, name),
             level = COALESCE($3, level),
             pattern = COALESCE($4, pattern),
             threshold = COALESCE($5, threshold),
             window_seconds = COALESCE($6, window_seconds),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, name, level, pattern, threshold, window_seconds, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(input.level.map(|l| l.to_string()))
        .bind(&input.pattern)
        .bind(input.threshold)
        .bind(input.window_seconds)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn delete_alert_rule_v4(
        &self,
        id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM log_alert_rules_v4 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn check_alert_rule_v4(
        &self,
        id: uuid::Uuid,
    ) -> Result<(LogAlertRuleV4, i64), sqlx::Error> {
        let rule = self.get_alert_rule_v4(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;
        let count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM log_entries_v7
             WHERE level = $1 AND message ILIKE $2
             AND created_at >= NOW() - make_interval(seconds => $3::int)"#,
        )
        .bind(rule.level.to_string())
        .bind(&rule.pattern)
        .bind(rule.window_seconds)
        .fetch_one(&self.pool)
        .await?;
        Ok((rule, count))
    }

    pub async fn check_all_alert_rules_v4(
        &self,
    ) -> Result<Vec<(LogAlertRuleV4, i64, bool)>, sqlx::Error> {
        let rules = self.list_alert_rules_v4().await?;
        let mut results = Vec::new();
        for rule in rules {
            if !rule.enabled {
                continue;
            }
            let count = sqlx::query_scalar::<_, i64>(
                r#"SELECT COUNT(*) FROM log_entries_v7
                 WHERE level = $1 AND message ILIKE $2
                 AND created_at >= NOW() - make_interval(seconds => $3::int)"#,
            )
            .bind(rule.level.to_string())
            .bind(&rule.pattern)
            .bind(rule.window_seconds)
            .fetch_one(&self.pool)
            .await?;
            let triggered = count >= rule.threshold as i64;
            results.push((rule, count, triggered));
        }
        Ok(results)
    }

    pub async fn match_pattern_v7(
        &self,
        pattern: &str,
        since: Option<chrono::DateTime<chrono::Utc>>,
        until: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<LogEntryV7>, sqlx::Error> {
        let search_pattern = format!("%{}%", pattern);
        let rows = sqlx::query_as::<_, LogEntryV7Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at
             FROM log_entries_v7
             WHERE message ILIKE $1
             AND ($2::timestamptz IS NULL OR created_at >= $2)
             AND ($3::timestamptz IS NULL OR created_at <= $3)
             ORDER BY created_at DESC LIMIT 1000"#,
        )
        .bind(&search_pattern)
        .bind(since)
        .bind(until)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    // V8: Log entries with enhanced indexing and alert rules v5

    pub async fn ingest_v8(
        &self,
        input: CreateLogEntryV8,
    ) -> Result<LogEntryV8, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryV8Row>(
            r#"INSERT INTO log_entries_v8 (level, message, source, service, trace_id, span_id, metadata, retention_days)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at"#,
        )
        .bind(input.level.to_string())
        .bind(&input.message)
        .bind(&input.source)
        .bind(input.service.as_deref().unwrap_or("civitforge"))
        .bind(&input.trace_id)
        .bind(&input.span_id)
        .bind(input.metadata.unwrap_or(serde_json::json!({})))
        .bind(input.retention_days.unwrap_or(30))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn search_v8(
        &self,
        filter: LogSearchFilterV8,
    ) -> Result<LogSearchResultV8, sqlx::Error> {
        let limit = filter.limit.unwrap_or(100).min(1000);
        let offset = filter.offset.unwrap_or(0);
        let level_str = filter.level.map(|l| l.to_string());
        let search_pattern = filter.search.map(|s| format!("%{}%", s));

        let total_count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM log_entries_v8
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::bool IS NULL OR indexed = $6)
             AND ($7::timestamptz IS NULL OR created_at >= $7)
             AND ($8::timestamptz IS NULL OR created_at <= $8)"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.indexed)
        .bind(filter.since)
        .bind(filter.until)
        .fetch_one(&self.pool)
        .await?;

        let entries = sqlx::query_as::<_, LogEntryV8Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at FROM log_entries_v8
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::bool IS NULL OR indexed = $6)
             AND ($7::timestamptz IS NULL OR created_at >= $7)
             AND ($8::timestamptz IS NULL OR created_at <= $8)
             ORDER BY created_at DESC LIMIT $9 OFFSET $10"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.indexed)
        .bind(filter.since)
        .bind(filter.until)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(LogSearchResultV8 {
            entries: entries.into_iter().map(|r| r.into()).collect(),
            total_count,
        })
    }

    pub async fn get_by_id_v8(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<LogEntryV8>, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryV8Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at
             FROM log_entries_v8 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn mark_indexed_v8(
        &self,
        id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE log_entries_v8 SET indexed = true WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_old_entries_v8(
        &self,
        max_age_days: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM log_entries_v8 WHERE created_at < NOW() - make_interval(days => $1::int)",
        )
        .bind(max_age_days)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() as i64)
    }

    // V8: Alert rules v5

    pub async fn create_alert_rule_v5(
        &self,
        input: CreateLogAlertRuleV5,
    ) -> Result<LogAlertRuleV5, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV5Row>(
            r#"INSERT INTO log_alert_rules_v5 (name, level, pattern, threshold, window_seconds, enabled)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, level, pattern, threshold, window_seconds, enabled, created_at"#,
        )
        .bind(&input.name)
        .bind(input.level.to_string())
        .bind(&input.pattern)
        .bind(input.threshold.unwrap_or(10))
        .bind(input.window_seconds.unwrap_or(300))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_alert_rule_v5(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<LogAlertRuleV5>, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV5Row>(
            r#"SELECT id, name, level, pattern, threshold, window_seconds, enabled, created_at
             FROM log_alert_rules_v5 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn list_alert_rules_v5(
        &self,
    ) -> Result<Vec<LogAlertRuleV5>, sqlx::Error> {
        let rows = sqlx::query_as::<_, LogAlertRuleV5Row>(
            r#"SELECT id, name, level, pattern, threshold, window_seconds, enabled, created_at
             FROM log_alert_rules_v5 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_alert_rule_v5(
        &self,
        id: uuid::Uuid,
        input: UpdateLogAlertRuleV5,
    ) -> Result<LogAlertRuleV5, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV5Row>(
            r#"UPDATE log_alert_rules_v5 SET
             name = COALESCE($2, name),
             level = COALESCE($3, level),
             pattern = COALESCE($4, pattern),
             threshold = COALESCE($5, threshold),
             window_seconds = COALESCE($6, window_seconds),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, name, level, pattern, threshold, window_seconds, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(input.level.map(|l| l.to_string()))
        .bind(&input.pattern)
        .bind(input.threshold)
        .bind(input.window_seconds)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn delete_alert_rule_v5(
        &self,
        id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM log_alert_rules_v5 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn check_alert_rule_v5(
        &self,
        id: uuid::Uuid,
    ) -> Result<(LogAlertRuleV5, i64), sqlx::Error> {
        let rule = self.get_alert_rule_v5(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;
        let count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM log_entries_v8
             WHERE level = $1 AND message ILIKE $2
             AND created_at >= NOW() - make_interval(seconds => $3::int)"#,
        )
        .bind(rule.level.to_string())
        .bind(&rule.pattern)
        .bind(rule.window_seconds)
        .fetch_one(&self.pool)
        .await?;
        Ok((rule, count))
    }

    pub async fn check_all_alert_rules_v5(
        &self,
    ) -> Result<Vec<(LogAlertRuleV5, i64, bool)>, sqlx::Error> {
        let rules = self.list_alert_rules_v5().await?;
        let mut results = Vec::new();
        for rule in rules {
            if !rule.enabled {
                continue;
            }
            let count = sqlx::query_scalar::<_, i64>(
                r#"SELECT COUNT(*) FROM log_entries_v8
                 WHERE level = $1 AND message ILIKE $2
                 AND created_at >= NOW() - make_interval(seconds => $3::int)"#,
            )
            .bind(rule.level.to_string())
            .bind(&rule.pattern)
            .bind(rule.window_seconds)
            .fetch_one(&self.pool)
            .await?;
            let triggered = count >= rule.threshold as i64;
            results.push((rule, count, triggered));
        }
        Ok(results)
    }

    pub async fn match_pattern_v8(
        &self,
        pattern: &str,
        since: Option<chrono::DateTime<chrono::Utc>>,
        until: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<LogEntryV8>, sqlx::Error> {
        let search_pattern = format!("%{}%", pattern);
        let rows = sqlx::query_as::<_, LogEntryV8Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at
             FROM log_entries_v8
             WHERE message ILIKE $1
             AND ($2::timestamptz IS NULL OR created_at >= $2)
             AND ($3::timestamptz IS NULL OR created_at <= $3)
             ORDER BY created_at DESC LIMIT 1000"#,
        )
        .bind(&search_pattern)
        .bind(since)
        .bind(until)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_pattern_matches_v8(
        &self,
        pattern: &str,
        since: Option<chrono::DateTime<chrono::Utc>>,
        until: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<LogPatternMatchV8, sqlx::Error> {
        let search_pattern = format!("%{}%", pattern);
        let rows = sqlx::query_as::<_, LogEntryV8Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at
             FROM log_entries_v8
             WHERE message ILIKE $1
             AND ($2::timestamptz IS NULL OR created_at >= $2)
             AND ($3::timestamptz IS NULL OR created_at <= $3)
             ORDER BY created_at ASC"#,
        )
        .bind(&search_pattern)
        .bind(since)
        .bind(until)
        .fetch_all(&self.pool)
        .await?;

        let match_count = rows.len() as i64;
        let affected_services: Vec<String> = rows
            .iter()
            .map(|r| &r.service)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .cloned()
            .collect();

        let first_match_at = rows.first().map(|r| r.created_at).unwrap_or_else(chrono::Utc::now);
        let last_match_at = rows.last().map(|r| r.created_at).unwrap_or_else(chrono::Utc::now);

        Ok(LogPatternMatchV8 {
            pattern: pattern.to_string(),
            match_count,
            first_match_at,
            last_match_at,
            affected_services,
        })
    }

    // V9: Log entries v9 and alert rules v6

    pub async fn ingest_v9(
        &self,
        input: CreateLogEntryV9,
    ) -> Result<LogEntryV9, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryV9Row>(
            r#"INSERT INTO log_entries_v9 (level, message, source, service, trace_id, span_id, metadata, retention_days)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at"#,
        )
        .bind(input.level.to_string())
        .bind(&input.message)
        .bind(&input.source)
        .bind(input.service.as_deref().unwrap_or("civitforge"))
        .bind(&input.trace_id)
        .bind(&input.span_id)
        .bind(input.metadata.unwrap_or(serde_json::json!({})))
        .bind(input.retention_days.unwrap_or(30))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn search_v9(
        &self,
        filter: LogSearchFilterV9,
    ) -> Result<LogSearchResultV9, sqlx::Error> {
        let limit = filter.limit.unwrap_or(100).min(1000);
        let offset = filter.offset.unwrap_or(0);
        let level_str = filter.level.map(|l| l.to_string());
        let search_pattern = filter.search.map(|s| format!("%{}%", s));

        let total_count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM log_entries_v9
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::bool IS NULL OR indexed = $6)
             AND ($7::timestamptz IS NULL OR created_at >= $7)
             AND ($8::timestamptz IS NULL OR created_at <= $8)"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.indexed)
        .bind(filter.since)
        .bind(filter.until)
        .fetch_one(&self.pool)
        .await?;

        let entries = sqlx::query_as::<_, LogEntryV9Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at FROM log_entries_v9
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::bool IS NULL OR indexed = $6)
             AND ($7::timestamptz IS NULL OR created_at >= $7)
             AND ($8::timestamptz IS NULL OR created_at <= $8)
             ORDER BY created_at DESC LIMIT $9 OFFSET $10"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.indexed)
        .bind(filter.since)
        .bind(filter.until)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(LogSearchResultV9 {
            entries: entries.into_iter().map(|r| r.into()).collect(),
            total_count,
        })
    }

    pub async fn get_by_id_v9(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<LogEntryV9>, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryV9Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at
             FROM log_entries_v9 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn mark_indexed_v9(
        &self,
        id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE log_entries_v9 SET indexed = true WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_old_entries_v9(
        &self,
        max_age_days: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM log_entries_v9 WHERE created_at < NOW() - make_interval(days => $1::int)",
        )
        .bind(max_age_days)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() as i64)
    }

    pub async fn create_alert_rule_v6(
        &self,
        input: CreateLogAlertRuleV6,
    ) -> Result<LogAlertRuleV6, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV6Row>(
            r#"INSERT INTO log_alert_rules_v6 (name, level, pattern, threshold, window_seconds, enabled)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, level, pattern, threshold, window_seconds, enabled, created_at"#,
        )
        .bind(&input.name)
        .bind(input.level.to_string())
        .bind(&input.pattern)
        .bind(input.threshold.unwrap_or(10))
        .bind(input.window_seconds.unwrap_or(300))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_alert_rule_v6(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<LogAlertRuleV6>, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV6Row>(
            r#"SELECT id, name, level, pattern, threshold, window_seconds, enabled, created_at
             FROM log_alert_rules_v6 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn list_alert_rules_v6(
        &self,
    ) -> Result<Vec<LogAlertRuleV6>, sqlx::Error> {
        let rows = sqlx::query_as::<_, LogAlertRuleV6Row>(
            r#"SELECT id, name, level, pattern, threshold, window_seconds, enabled, created_at
             FROM log_alert_rules_v6 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_alert_rule_v6(
        &self,
        id: uuid::Uuid,
        input: UpdateLogAlertRuleV6,
    ) -> Result<LogAlertRuleV6, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV6Row>(
            r#"UPDATE log_alert_rules_v6 SET
             name = COALESCE($2, name),
             level = COALESCE($3, level),
             pattern = COALESCE($4, pattern),
             threshold = COALESCE($5, threshold),
             window_seconds = COALESCE($6, window_seconds),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, name, level, pattern, threshold, window_seconds, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(input.level.map(|l| l.to_string()))
        .bind(&input.pattern)
        .bind(input.threshold)
        .bind(input.window_seconds)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn delete_alert_rule_v6(
        &self,
        id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM log_alert_rules_v6 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn check_alert_rule_v6(
        &self,
        id: uuid::Uuid,
    ) -> Result<(LogAlertRuleV6, i64), sqlx::Error> {
        let rule = self.get_alert_rule_v6(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;
        let count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM log_entries_v9
             WHERE level = $1 AND message ILIKE $2
             AND created_at >= NOW() - make_interval(seconds => $3::int)"#,
        )
        .bind(rule.level.to_string())
        .bind(&rule.pattern)
        .bind(rule.window_seconds)
        .fetch_one(&self.pool)
        .await?;
        Ok((rule, count))
    }

    pub async fn check_all_alert_rules_v6(
        &self,
    ) -> Result<Vec<(LogAlertRuleV6, i64, bool)>, sqlx::Error> {
        let rules = self.list_alert_rules_v6().await?;
        let mut results = Vec::new();
        for rule in rules {
            if !rule.enabled {
                continue;
            }
            let count = sqlx::query_scalar::<_, i64>(
                r#"SELECT COUNT(*) FROM log_entries_v9
                 WHERE level = $1 AND message ILIKE $2
                 AND created_at >= NOW() - make_interval(seconds => $3::int)"#,
            )
            .bind(rule.level.to_string())
            .bind(&rule.pattern)
            .bind(rule.window_seconds)
            .fetch_one(&self.pool)
            .await?;
            let triggered = count >= rule.threshold as i64;
            results.push((rule, count, triggered));
        }
        Ok(results)
    }

    pub async fn match_pattern_v9(
        &self,
        pattern: &str,
        since: Option<chrono::DateTime<chrono::Utc>>,
        until: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<LogEntryV9>, sqlx::Error> {
        let search_pattern = format!("%{}%", pattern);
        let rows = sqlx::query_as::<_, LogEntryV9Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at
             FROM log_entries_v9
             WHERE message ILIKE $1
             AND ($2::timestamptz IS NULL OR created_at >= $2)
             AND ($3::timestamptz IS NULL OR created_at <= $3)
             ORDER BY created_at DESC LIMIT 1000"#,
        )
        .bind(&search_pattern)
        .bind(since)
        .bind(until)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_pattern_matches_v9(
        &self,
        pattern: &str,
        since: Option<chrono::DateTime<chrono::Utc>>,
        until: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<LogPatternMatchV9, sqlx::Error> {
        let search_pattern = format!("%{}%", pattern);
        let rows = sqlx::query_as::<_, LogEntryV9Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at
             FROM log_entries_v9
             WHERE message ILIKE $1
             AND ($2::timestamptz IS NULL OR created_at >= $2)
             AND ($3::timestamptz IS NULL OR created_at <= $3)
             ORDER BY created_at ASC"#,
        )
        .bind(&search_pattern)
        .bind(since)
        .bind(until)
        .fetch_all(&self.pool)
        .await?;

        let match_count = rows.len() as i64;
        let affected_services: Vec<String> = rows
            .iter()
            .map(|r| &r.service)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .cloned()
            .collect();

        let first_match_at = rows.first().map(|r| r.created_at).unwrap_or_else(chrono::Utc::now);
        let last_match_at = rows.last().map(|r| r.created_at).unwrap_or_else(chrono::Utc::now);

        Ok(LogPatternMatchV9 {
            pattern: pattern.to_string(),
            match_count,
            first_match_at,
            last_match_at,
            affected_services,
        })
    }

    // V10: Log entries v10 and alert rules v7

    pub async fn ingest_v10(
        &self,
        input: CreateLogEntryV10,
    ) -> Result<LogEntryV10, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryV10Row>(
            r#"INSERT INTO log_entries_v10 (level, message, source, service, trace_id, span_id, metadata, retention_days)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at"#,
        )
        .bind(input.level.to_string())
        .bind(&input.message)
        .bind(&input.source)
        .bind(input.service.as_deref().unwrap_or("civitforge"))
        .bind(&input.trace_id)
        .bind(&input.span_id)
        .bind(input.metadata.unwrap_or(serde_json::json!({})))
        .bind(input.retention_days.unwrap_or(30))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn search_v10(
        &self,
        filter: LogSearchFilterV10,
    ) -> Result<LogSearchResultV10, sqlx::Error> {
        let limit = filter.limit.unwrap_or(100).min(1000);
        let offset = filter.offset.unwrap_or(0);
        let level_str = filter.level.map(|l| l.to_string());
        let search_pattern = filter.search.map(|s| format!("%{}%", s));

        let total_count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM log_entries_v10
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::bool IS NULL OR indexed = $6)
             AND ($7::timestamptz IS NULL OR created_at >= $7)
             AND ($8::timestamptz IS NULL OR created_at <= $8)"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.indexed)
        .bind(filter.since)
        .bind(filter.until)
        .fetch_one(&self.pool)
        .await?;

        let entries = sqlx::query_as::<_, LogEntryV10Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at FROM log_entries_v10
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::bool IS NULL OR indexed = $6)
             AND ($7::timestamptz IS NULL OR created_at >= $7)
             AND ($8::timestamptz IS NULL OR created_at <= $8)
             ORDER BY created_at DESC LIMIT $9 OFFSET $10"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.indexed)
        .bind(filter.since)
        .bind(filter.until)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(LogSearchResultV10 {
            entries: entries.into_iter().map(|r| r.into()).collect(),
            total_count,
        })
    }

    pub async fn get_by_id_v10(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<LogEntryV10>, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryV10Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at
             FROM log_entries_v10 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn mark_indexed_v10(
        &self,
        id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE log_entries_v10 SET indexed = true WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_old_entries_v10(
        &self,
        max_age_days: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM log_entries_v10 WHERE created_at < NOW() - make_interval(days => $1::int)",
        )
        .bind(max_age_days)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() as i64)
    }

    pub async fn create_alert_rule_v7(
        &self,
        input: CreateLogAlertRuleV7,
    ) -> Result<LogAlertRuleV7, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV7Row>(
            r#"INSERT INTO log_alert_rules_v7 (name, level, pattern, threshold, window_seconds, enabled)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, level, pattern, threshold, window_seconds, enabled, created_at"#,
        )
        .bind(&input.name)
        .bind(input.level.to_string())
        .bind(&input.pattern)
        .bind(input.threshold.unwrap_or(10))
        .bind(input.window_seconds.unwrap_or(300))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_alert_rule_v7(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<LogAlertRuleV7>, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV7Row>(
            r#"SELECT id, name, level, pattern, threshold, window_seconds, enabled, created_at
             FROM log_alert_rules_v7 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn list_alert_rules_v7(
        &self,
    ) -> Result<Vec<LogAlertRuleV7>, sqlx::Error> {
        let rows = sqlx::query_as::<_, LogAlertRuleV7Row>(
            r#"SELECT id, name, level, pattern, threshold, window_seconds, enabled, created_at
             FROM log_alert_rules_v7 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_alert_rule_v7(
        &self,
        id: uuid::Uuid,
        input: UpdateLogAlertRuleV7,
    ) -> Result<LogAlertRuleV7, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV7Row>(
            r#"UPDATE log_alert_rules_v7 SET
             name = COALESCE($2, name),
             level = COALESCE($3, level),
             pattern = COALESCE($4, pattern),
             threshold = COALESCE($5, threshold),
             window_seconds = COALESCE($6, window_seconds),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, name, level, pattern, threshold, window_seconds, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(input.level.map(|l| l.to_string()))
        .bind(&input.pattern)
        .bind(input.threshold)
        .bind(input.window_seconds)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn delete_alert_rule_v7(
        &self,
        id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM log_alert_rules_v7 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn check_alert_rule_v7(
        &self,
        id: uuid::Uuid,
    ) -> Result<(LogAlertRuleV7, i64), sqlx::Error> {
        let rule = self.get_alert_rule_v7(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;
        let count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM log_entries_v10
             WHERE level = $1 AND message ILIKE $2
             AND created_at >= NOW() - make_interval(seconds => $3::int)"#,
        )
        .bind(rule.level.to_string())
        .bind(&rule.pattern)
        .bind(rule.window_seconds)
        .fetch_one(&self.pool)
        .await?;
        Ok((rule, count))
    }

    pub async fn check_all_alert_rules_v7(
        &self,
    ) -> Result<Vec<(LogAlertRuleV7, i64, bool)>, sqlx::Error> {
        let rules = self.list_alert_rules_v7().await?;
        let mut results = Vec::new();
        for rule in rules {
            if !rule.enabled {
                continue;
            }
            let count = sqlx::query_scalar::<_, i64>(
                r#"SELECT COUNT(*) FROM log_entries_v10
                 WHERE level = $1 AND message ILIKE $2
                 AND created_at >= NOW() - make_interval(seconds => $3::int)"#,
            )
            .bind(rule.level.to_string())
            .bind(&rule.pattern)
            .bind(rule.window_seconds)
            .fetch_one(&self.pool)
            .await?;
            let triggered = count >= rule.threshold as i64;
            results.push((rule, count, triggered));
        }
        Ok(results)
    }

    pub async fn match_pattern_v10(
        &self,
        pattern: &str,
        since: Option<chrono::DateTime<chrono::Utc>>,
        until: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<LogEntryV10>, sqlx::Error> {
        let search_pattern = format!("%{}%", pattern);
        let rows = sqlx::query_as::<_, LogEntryV10Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at
             FROM log_entries_v10
             WHERE message ILIKE $1
             AND ($2::timestamptz IS NULL OR created_at >= $2)
             AND ($3::timestamptz IS NULL OR created_at <= $3)
             ORDER BY created_at DESC LIMIT 1000"#,
        )
        .bind(&search_pattern)
        .bind(since)
        .bind(until)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_pattern_matches_v10(
        &self,
        pattern: &str,
        since: Option<chrono::DateTime<chrono::Utc>>,
        until: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<LogPatternMatchV10, sqlx::Error> {
        let search_pattern = format!("%{}%", pattern);
        let rows = sqlx::query_as::<_, LogEntryV10Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at
             FROM log_entries_v10
             WHERE message ILIKE $1
             AND ($2::timestamptz IS NULL OR created_at >= $2)
             AND ($3::timestamptz IS NULL OR created_at <= $3)
             ORDER BY created_at ASC"#,
        )
        .bind(&search_pattern)
        .bind(since)
        .bind(until)
        .fetch_all(&self.pool)
        .await?;

        let match_count = rows.len() as i64;
        let affected_services: Vec<String> = rows
            .iter()
            .map(|r| &r.service)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .cloned()
            .collect();

        let first_match_at = rows.first().map(|r| r.created_at).unwrap_or_else(chrono::Utc::now);
        let last_match_at = rows.last().map(|r| r.created_at).unwrap_or_else(chrono::Utc::now);

        Ok(LogPatternMatchV10 {
            pattern: pattern.to_string(),
            match_count,
            first_match_at,
            last_match_at,
            affected_services,
        })
    }

    // V11: Log entries v11 and alert rules v8

    pub async fn ingest_v11(
        &self,
        input: CreateLogEntryV11,
    ) -> Result<LogEntryV11, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryV11Row>(
            r#"INSERT INTO log_entries_v11 (level, message, source, service, trace_id, span_id, metadata, retention_days)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at"#,
        )
        .bind(input.level.to_string())
        .bind(&input.message)
        .bind(&input.source)
        .bind(input.service.as_deref().unwrap_or("civitforge"))
        .bind(&input.trace_id)
        .bind(&input.span_id)
        .bind(input.metadata.unwrap_or(serde_json::json!({})))
        .bind(input.retention_days.unwrap_or(30))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn search_v11(
        &self,
        filter: LogSearchFilterV11,
    ) -> Result<LogSearchResultV11, sqlx::Error> {
        let limit = filter.limit.unwrap_or(100).min(1000);
        let offset = filter.offset.unwrap_or(0);
        let level_str = filter.level.map(|l| l.to_string());
        let search_pattern = filter.search.map(|s| format!("%{}%", s));

        let total_count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM log_entries_v11
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::bool IS NULL OR indexed = $6)
             AND ($7::timestamptz IS NULL OR created_at >= $7)
             AND ($8::timestamptz IS NULL OR created_at <= $8)"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.indexed)
        .bind(filter.since)
        .bind(filter.until)
        .fetch_one(&self.pool)
        .await?;

        let entries = sqlx::query_as::<_, LogEntryV11Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at FROM log_entries_v11
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::bool IS NULL OR indexed = $6)
             AND ($7::timestamptz IS NULL OR created_at >= $7)
             AND ($8::timestamptz IS NULL OR created_at <= $8)
             ORDER BY created_at DESC LIMIT $9 OFFSET $10"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.indexed)
        .bind(filter.since)
        .bind(filter.until)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(LogSearchResultV11 {
            entries: entries.into_iter().map(|r| r.into()).collect(),
            total_count,
        })
    }

    pub async fn get_by_id_v11(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<LogEntryV11>, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryV11Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at
             FROM log_entries_v11 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn mark_indexed_v11(
        &self,
        id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE log_entries_v11 SET indexed = true WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_old_entries_v11(
        &self,
        max_age_days: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM log_entries_v11 WHERE created_at < NOW() - make_interval(days => $1::int)",
        )
        .bind(max_age_days)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() as i64)
    }

    pub async fn create_alert_rule_v8(
        &self,
        input: CreateLogAlertRuleV8,
    ) -> Result<LogAlertRuleV8, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV8Row>(
            r#"INSERT INTO log_alert_rules_v8 (name, level, pattern, threshold, window_seconds, enabled)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, level, pattern, threshold, window_seconds, enabled, created_at"#,
        )
        .bind(&input.name)
        .bind(input.level.to_string())
        .bind(&input.pattern)
        .bind(input.threshold.unwrap_or(10))
        .bind(input.window_seconds.unwrap_or(300))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_alert_rule_v8(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<LogAlertRuleV8>, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV8Row>(
            r#"SELECT id, name, level, pattern, threshold, window_seconds, enabled, created_at
             FROM log_alert_rules_v8 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn list_alert_rules_v8(
        &self,
    ) -> Result<Vec<LogAlertRuleV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, LogAlertRuleV8Row>(
            r#"SELECT id, name, level, pattern, threshold, window_seconds, enabled, created_at
             FROM log_alert_rules_v8 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_alert_rule_v8(
        &self,
        id: uuid::Uuid,
        input: UpdateLogAlertRuleV8,
    ) -> Result<LogAlertRuleV8, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV8Row>(
            r#"UPDATE log_alert_rules_v8 SET
             name = COALESCE($2, name),
             level = COALESCE($3, level),
             pattern = COALESCE($4, pattern),
             threshold = COALESCE($5, threshold),
             window_seconds = COALESCE($6, window_seconds),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, name, level, pattern, threshold, window_seconds, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(input.level.map(|l| l.to_string()))
        .bind(&input.pattern)
        .bind(input.threshold)
        .bind(input.window_seconds)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn delete_alert_rule_v8(
        &self,
        id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM log_alert_rules_v8 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn check_alert_rule_v8(
        &self,
        id: uuid::Uuid,
    ) -> Result<(LogAlertRuleV8, i64), sqlx::Error> {
        let rule = self.get_alert_rule_v8(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;
        let count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM log_entries_v11
             WHERE level = $1 AND message ILIKE $2
             AND created_at >= NOW() - make_interval(seconds => $3::int)"#,
        )
        .bind(rule.level.to_string())
        .bind(&rule.pattern)
        .bind(rule.window_seconds)
        .fetch_one(&self.pool)
        .await?;
        Ok((rule, count))
    }

    pub async fn check_all_alert_rules_v8(
        &self,
    ) -> Result<Vec<(LogAlertRuleV8, i64, bool)>, sqlx::Error> {
        let rules = self.list_alert_rules_v8().await?;
        let mut results = Vec::new();
        for rule in rules {
            if !rule.enabled {
                continue;
            }
            let count = sqlx::query_scalar::<_, i64>(
                r#"SELECT COUNT(*) FROM log_entries_v11
                 WHERE level = $1 AND message ILIKE $2
                 AND created_at >= NOW() - make_interval(seconds => $3::int)"#,
            )
            .bind(rule.level.to_string())
            .bind(&rule.pattern)
            .bind(rule.window_seconds)
            .fetch_one(&self.pool)
            .await?;
            let triggered = count >= rule.threshold as i64;
            results.push((rule, count, triggered));
        }
        Ok(results)
    }

    pub async fn match_pattern_v11(
        &self,
        pattern: &str,
        since: Option<chrono::DateTime<chrono::Utc>>,
        until: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<LogEntryV11>, sqlx::Error> {
        let search_pattern = format!("%{}%", pattern);
        let rows = sqlx::query_as::<_, LogEntryV11Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at
             FROM log_entries_v11
             WHERE message ILIKE $1
             AND ($2::timestamptz IS NULL OR created_at >= $2)
             AND ($3::timestamptz IS NULL OR created_at <= $3)
             ORDER BY created_at DESC LIMIT 1000"#,
        )
        .bind(&search_pattern)
        .bind(since)
        .bind(until)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_pattern_matches_v11(
        &self,
        pattern: &str,
        since: Option<chrono::DateTime<chrono::Utc>>,
        until: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<LogPatternMatchV11, sqlx::Error> {
        let search_pattern = format!("%{}%", pattern);
        let rows = sqlx::query_as::<_, LogEntryV11Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at
             FROM log_entries_v11
             WHERE message ILIKE $1
             AND ($2::timestamptz IS NULL OR created_at >= $2)
             AND ($3::timestamptz IS NULL OR created_at <= $3)
             ORDER BY created_at ASC"#,
        )
        .bind(&search_pattern)
        .bind(since)
        .bind(until)
        .fetch_all(&self.pool)
        .await?;

        let match_count = rows.len() as i64;
        let affected_services: Vec<String> = rows
            .iter()
            .map(|r| &r.service)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .cloned()
            .collect();

        let first_match_at = rows.first().map(|r| r.created_at).unwrap_or_else(chrono::Utc::now);
        let last_match_at = rows.last().map(|r| r.created_at).unwrap_or_else(chrono::Utc::now);

        Ok(LogPatternMatchV11 {
            pattern: pattern.to_string(),
            match_count,
            first_match_at,
            last_match_at,
            affected_services,
        })
    }

    // V12: Log entries v12 and alert rules v9

    pub async fn ingest_v12(
        &self,
        input: CreateLogEntryV12,
    ) -> Result<LogEntryV12, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryV12Row>(
            r#"INSERT INTO log_entries_v12 (level, message, source, service, trace_id, span_id, metadata, retention_days)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at"#,
        )
        .bind(input.level.to_string())
        .bind(&input.message)
        .bind(&input.source)
        .bind(input.service.as_deref().unwrap_or("civitforge"))
        .bind(&input.trace_id)
        .bind(&input.span_id)
        .bind(input.metadata.unwrap_or(serde_json::json!({})))
        .bind(input.retention_days.unwrap_or(30))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn search_v12(
        &self,
        filter: LogSearchFilterV12,
    ) -> Result<LogSearchResultV12, sqlx::Error> {
        let limit = filter.limit.unwrap_or(100).min(1000);
        let offset = filter.offset.unwrap_or(0);
        let level_str = filter.level.map(|l| l.to_string());
        let search_pattern = filter.search.map(|s| format!("%{}%", s));

        let total_count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM log_entries_v12
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::bool IS NULL OR indexed = $6)
             AND ($7::timestamptz IS NULL OR created_at >= $7)
             AND ($8::timestamptz IS NULL OR created_at <= $8)"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.indexed)
        .bind(filter.since)
        .bind(filter.until)
        .fetch_one(&self.pool)
        .await?;

        let entries = sqlx::query_as::<_, LogEntryV12Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at FROM log_entries_v12
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR service = $3)
             AND ($4::text IS NULL OR trace_id = $4)
             AND ($5::text IS NULL OR message ILIKE $5)
             AND ($6::bool IS NULL OR indexed = $6)
             AND ($7::timestamptz IS NULL OR created_at >= $7)
             AND ($8::timestamptz IS NULL OR created_at <= $8)
             ORDER BY created_at DESC LIMIT $9 OFFSET $10"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&filter.service)
        .bind(&filter.trace_id)
        .bind(&search_pattern)
        .bind(filter.indexed)
        .bind(filter.since)
        .bind(filter.until)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(LogSearchResultV12 {
            entries: entries.into_iter().map(|r| r.into()).collect(),
            total_count,
        })
    }

    pub async fn get_by_id_v12(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<LogEntryV12>, sqlx::Error> {
        let row = sqlx::query_as::<_, LogEntryV12Row>(
            r#"SELECT id, level, message, source, service, trace_id, span_id, metadata, retention_days, indexed, created_at
             FROM log_entries_v12 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn mark_indexed_v12(
        &self,
        id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE log_entries_v12 SET indexed = true WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_old_entries_v12(
        &self,
        max_age_days: i32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM log_entries_v12 WHERE created_at < NOW() - make_interval(days => $1::int)",
        )
        .bind(max_age_days)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() as i64)
    }

    pub async fn create_alert_rule_v9(
        &self,
        input: CreateLogAlertRuleV9,
    ) -> Result<LogAlertRuleV9, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV9Row>(
            r#"INSERT INTO log_alert_rules_v9 (name, level, pattern, threshold, window_seconds, enabled)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, level, pattern, threshold, window_seconds, enabled, created_at"#,
        )
        .bind(&input.name)
        .bind(input.level.to_string())
        .bind(&input.pattern)
        .bind(input.threshold.unwrap_or(10))
        .bind(input.window_seconds.unwrap_or(300))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_alert_rule_v9(
        &self,
        id: uuid::Uuid,
    ) -> Result<Option<LogAlertRuleV9>, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV9Row>(
            r#"SELECT id, name, level, pattern, threshold, window_seconds, enabled, created_at
             FROM log_alert_rules_v9 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn list_alert_rules_v9(
        &self,
    ) -> Result<Vec<LogAlertRuleV9>, sqlx::Error> {
        let rows = sqlx::query_as::<_, LogAlertRuleV9Row>(
            r#"SELECT id, name, level, pattern, threshold, window_seconds, enabled, created_at
             FROM log_alert_rules_v9 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_alert_rule_v9(
        &self,
        id: uuid::Uuid,
        input: UpdateLogAlertRuleV9,
    ) -> Result<LogAlertRuleV9, sqlx::Error> {
        let row = sqlx::query_as::<_, LogAlertRuleV9Row>(
            r#"UPDATE log_alert_rules_v9 SET
             name = COALESCE($2, name),
             level = COALESCE($3, level),
             pattern = COALESCE($4, pattern),
             threshold = COALESCE($5, threshold),
             window_seconds = COALESCE($6, window_seconds),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, name, level, pattern, threshold, window_seconds, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(input.level.map(|l| l.to_string()))
        .bind(&input.pattern)
        .bind(input.threshold)
        .bind(input.window_seconds)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn delete_alert_rule_v9(
        &self,
        id: uuid::Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM log_alert_rules_v9 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn check_alert_rule_v9(
        &self,
        id: uuid::Uuid,
    ) -> Result<(LogAlertRuleV9, i64), sqlx::Error> {
        let rule = self.get_alert_rule_v9(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;
        let count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) FROM log_entries_v12
             WHERE level = $1 AND message ILIKE $2
             AND created_at >= NOW() - make_interval(seconds => $3::int)"#,
        )
        .bind(rule.level.to_string())
        .bind(&rule.pattern)
        .bind(rule.window_seconds)
        .fetch_one(&self.pool)
        .await?;
        Ok((rule, count))
    }

    pub async fn check_all_alert_rules_v9(
        &self,
    ) -> Result<Vec<(LogAlertRuleV9, i64, bool)>, sqlx::Error> {
        let rules = self.list_alert_rules_v9().await?;
        let mut results = Vec::new();
        for rule in rules {
            if !rule.enabled {
                continue;
            }
            let count = sqlx::query_scalar::<_, i64>(
                r#"SELECT COUNT(*) FROM log_entries_v12
                 WHERE level = $1 AND message ILIKE $2
                 AND created_at >= NOW() - make_interval(seconds => $3::int)"#,
            )
            .bind(rule.level.to_string())
            .bind(&rule.pattern)
            .bind(rule.window_seconds)
            .fetch_one(&self.pool)
            .await?;
            let triggered = count >= rule.threshold as i64;
            results.push((rule, count, triggered));
        }
        Ok(results)
    }
}

#[derive(Debug, sqlx::FromRow)]
struct LogEntryV3Row {
    id: uuid::Uuid,
    level: String,
    message: String,
    source: String,
    service: String,
    trace_id: Option<String>,
    span_id: Option<String>,
    metadata: serde_json::Value,
    retention_days: i32,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<LogEntryV3Row> for LogEntryV3 {
    fn from(row: LogEntryV3Row) -> Self {
        LogEntryV3 {
            id: row.id,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            message: row.message,
            source: row.source,
            service: row.service,
            trace_id: row.trace_id,
            span_id: row.span_id,
            metadata: row.metadata,
            retention_days: row.retention_days,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct LogEntryV4Row {
    id: uuid::Uuid,
    level: String,
    message: String,
    source: String,
    service: String,
    trace_id: Option<String>,
    span_id: Option<String>,
    metadata: serde_json::Value,
    retention_days: i32,
    indexed: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<LogEntryV4Row> for LogEntryV4 {
    fn from(row: LogEntryV4Row) -> Self {
        LogEntryV4 {
            id: row.id,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            message: row.message,
            source: row.source,
            service: row.service,
            trace_id: row.trace_id,
            span_id: row.span_id,
            metadata: row.metadata,
            retention_days: row.retention_days,
            indexed: row.indexed,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct LogAlertRuleRow {
    id: uuid::Uuid,
    name: String,
    level: String,
    pattern: String,
    threshold: i32,
    window_seconds: i32,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<LogAlertRuleRow> for LogAlertRule {
    fn from(row: LogAlertRuleRow) -> Self {
        LogAlertRule {
            id: row.id,
            name: row.name,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            pattern: row.pattern,
            threshold: row.threshold,
            window_seconds: row.window_seconds,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct LogEntryV5Row {
    id: uuid::Uuid,
    level: String,
    message: String,
    source: String,
    service: String,
    trace_id: Option<String>,
    span_id: Option<String>,
    metadata: serde_json::Value,
    retention_days: i32,
    indexed: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<LogEntryV5Row> for LogEntryV5 {
    fn from(row: LogEntryV5Row) -> Self {
        LogEntryV5 {
            id: row.id,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            message: row.message,
            source: row.source,
            service: row.service,
            trace_id: row.trace_id,
            span_id: row.span_id,
            metadata: row.metadata,
            retention_days: row.retention_days,
            indexed: row.indexed,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct LogAlertRuleV2Row {
    id: uuid::Uuid,
    name: String,
    level: String,
    pattern: String,
    threshold: i32,
    window_seconds: i32,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<LogAlertRuleV2Row> for LogAlertRuleV2 {
    fn from(row: LogAlertRuleV2Row) -> Self {
        LogAlertRuleV2 {
            id: row.id,
            name: row.name,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            pattern: row.pattern,
            threshold: row.threshold,
            window_seconds: row.window_seconds,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct LogEntryV6Row {
    id: uuid::Uuid,
    level: String,
    message: String,
    source: String,
    service: String,
    trace_id: Option<String>,
    span_id: Option<String>,
    metadata: serde_json::Value,
    retention_days: i32,
    indexed: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<LogEntryV6Row> for LogEntryV6 {
    fn from(row: LogEntryV6Row) -> Self {
        LogEntryV6 {
            id: row.id,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            message: row.message,
            source: row.source,
            service: row.service,
            trace_id: row.trace_id,
            span_id: row.span_id,
            metadata: row.metadata,
            retention_days: row.retention_days,
            indexed: row.indexed,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct LogAlertRuleV3Row {
    id: uuid::Uuid,
    name: String,
    level: String,
    pattern: String,
    threshold: i32,
    window_seconds: i32,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<LogAlertRuleV3Row> for LogAlertRuleV3 {
    fn from(row: LogAlertRuleV3Row) -> Self {
        LogAlertRuleV3 {
            id: row.id,
            name: row.name,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            pattern: row.pattern,
            threshold: row.threshold,
            window_seconds: row.window_seconds,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct LogEntryV7Row {
    id: uuid::Uuid,
    level: String,
    message: String,
    source: String,
    service: String,
    trace_id: Option<String>,
    span_id: Option<String>,
    metadata: serde_json::Value,
    retention_days: i32,
    indexed: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<LogEntryV7Row> for LogEntryV7 {
    fn from(row: LogEntryV7Row) -> Self {
        LogEntryV7 {
            id: row.id,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            message: row.message,
            source: row.source,
            service: row.service,
            trace_id: row.trace_id,
            span_id: row.span_id,
            metadata: row.metadata,
            retention_days: row.retention_days,
            indexed: row.indexed,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct LogAlertRuleV4Row {
    id: uuid::Uuid,
    name: String,
    level: String,
    pattern: String,
    threshold: i32,
    window_seconds: i32,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<LogAlertRuleV4Row> for LogAlertRuleV4 {
    fn from(row: LogAlertRuleV4Row) -> Self {
        LogAlertRuleV4 {
            id: row.id,
            name: row.name,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            pattern: row.pattern,
            threshold: row.threshold,
            window_seconds: row.window_seconds,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct LogEntryV8Row {
    id: uuid::Uuid,
    level: String,
    message: String,
    source: String,
    service: String,
    trace_id: Option<String>,
    span_id: Option<String>,
    metadata: serde_json::Value,
    retention_days: i32,
    indexed: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<LogEntryV8Row> for LogEntryV8 {
    fn from(row: LogEntryV8Row) -> Self {
        LogEntryV8 {
            id: row.id,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            message: row.message,
            source: row.source,
            service: row.service,
            trace_id: row.trace_id,
            span_id: row.span_id,
            metadata: row.metadata,
            retention_days: row.retention_days,
            indexed: row.indexed,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct LogAlertRuleV5Row {
    id: uuid::Uuid,
    name: String,
    level: String,
    pattern: String,
    threshold: i32,
    window_seconds: i32,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<LogAlertRuleV5Row> for LogAlertRuleV5 {
    fn from(row: LogAlertRuleV5Row) -> Self {
        LogAlertRuleV5 {
            id: row.id,
            name: row.name,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            pattern: row.pattern,
            threshold: row.threshold,
            window_seconds: row.window_seconds,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct LogEntryV9Row {
    id: uuid::Uuid,
    level: String,
    message: String,
    source: String,
    service: String,
    trace_id: Option<String>,
    span_id: Option<String>,
    metadata: serde_json::Value,
    retention_days: i32,
    indexed: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<LogEntryV9Row> for LogEntryV9 {
    fn from(row: LogEntryV9Row) -> Self {
        LogEntryV9 {
            id: row.id,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            message: row.message,
            source: row.source,
            service: row.service,
            trace_id: row.trace_id,
            span_id: row.span_id,
            metadata: row.metadata,
            retention_days: row.retention_days,
            indexed: row.indexed,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct LogAlertRuleV6Row {
    id: uuid::Uuid,
    name: String,
    level: String,
    pattern: String,
    threshold: i32,
    window_seconds: i32,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<LogAlertRuleV6Row> for LogAlertRuleV6 {
    fn from(row: LogAlertRuleV6Row) -> Self {
        LogAlertRuleV6 {
            id: row.id,
            name: row.name,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            pattern: row.pattern,
            threshold: row.threshold,
            window_seconds: row.window_seconds,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct LogEntryV10Row {
    id: uuid::Uuid,
    level: String,
    message: String,
    source: String,
    service: String,
    trace_id: Option<String>,
    span_id: Option<String>,
    metadata: serde_json::Value,
    retention_days: i32,
    indexed: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<LogEntryV10Row> for LogEntryV10 {
    fn from(row: LogEntryV10Row) -> Self {
        LogEntryV10 {
            id: row.id,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            message: row.message,
            source: row.source,
            service: row.service,
            trace_id: row.trace_id,
            span_id: row.span_id,
            metadata: row.metadata,
            retention_days: row.retention_days,
            indexed: row.indexed,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct LogAlertRuleV7Row {
    id: uuid::Uuid,
    name: String,
    level: String,
    pattern: String,
    threshold: i32,
    window_seconds: i32,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<LogAlertRuleV7Row> for LogAlertRuleV7 {
    fn from(row: LogAlertRuleV7Row) -> Self {
        LogAlertRuleV7 {
            id: row.id,
            name: row.name,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            pattern: row.pattern,
            threshold: row.threshold,
            window_seconds: row.window_seconds,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct LogEntryV11Row {
    id: uuid::Uuid,
    level: String,
    message: String,
    source: String,
    service: String,
    trace_id: Option<String>,
    span_id: Option<String>,
    metadata: serde_json::Value,
    retention_days: i32,
    indexed: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<LogEntryV11Row> for LogEntryV11 {
    fn from(row: LogEntryV11Row) -> Self {
        LogEntryV11 {
            id: row.id,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            message: row.message,
            source: row.source,
            service: row.service,
            trace_id: row.trace_id,
            span_id: row.span_id,
            metadata: row.metadata,
            retention_days: row.retention_days,
            indexed: row.indexed,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct LogAlertRuleV8Row {
    id: uuid::Uuid,
    name: String,
    level: String,
    pattern: String,
    threshold: i32,
    window_seconds: i32,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<LogAlertRuleV8Row> for LogAlertRuleV8 {
    fn from(row: LogAlertRuleV8Row) -> Self {
        LogAlertRuleV8 {
            id: row.id,
            name: row.name,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            pattern: row.pattern,
            threshold: row.threshold,
            window_seconds: row.window_seconds,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct LogEntryV12Row {
    id: uuid::Uuid,
    level: String,
    message: String,
    source: String,
    service: String,
    trace_id: Option<String>,
    span_id: Option<String>,
    metadata: serde_json::Value,
    retention_days: i32,
    indexed: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<LogEntryV12Row> for LogEntryV12 {
    fn from(row: LogEntryV12Row) -> Self {
        LogEntryV12 {
            id: row.id,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            message: row.message,
            source: row.source,
            service: row.service,
            trace_id: row.trace_id,
            span_id: row.span_id,
            metadata: row.metadata,
            retention_days: row.retention_days,
            indexed: row.indexed,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct LogAlertRuleV9Row {
    id: uuid::Uuid,
    name: String,
    level: String,
    pattern: String,
    threshold: i32,
    window_seconds: i32,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<LogAlertRuleV9Row> for LogAlertRuleV9 {
    fn from(row: LogAlertRuleV9Row) -> Self {
        LogAlertRuleV9 {
            id: row.id,
            name: row.name,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            pattern: row.pattern,
            threshold: row.threshold,
            window_seconds: row.window_seconds,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}
