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
            metadata: row.metadata,
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
            r#"INSERT INTO log_entries (level, message, source, metadata)
             VALUES ($1, $2, $3, $4)
             RETURNING id, level, message, source, metadata, created_at"#,
        )
        .bind(input.level.to_string())
        .bind(&input.message)
        .bind(&input.source)
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
            r#"SELECT COUNT(*) FROM log_entries
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR message ILIKE $3)
             AND ($4::timestamptz IS NULL OR created_at >= $4)
             AND ($5::timestamptz IS NULL OR created_at <= $5)"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
        .bind(&search_pattern)
        .bind(filter.since)
        .bind(filter.until)
        .fetch_one(&self.pool)
        .await?;

        let entries = sqlx::query_as::<_, LogEntryRow>(
            r#"SELECT id, level, message, source, metadata, created_at FROM log_entries
             WHERE ($1::text IS NULL OR level = $1)
             AND ($2::text IS NULL OR source = $2)
             AND ($3::text IS NULL OR message ILIKE $3)
             AND ($4::timestamptz IS NULL OR created_at >= $4)
             AND ($5::timestamptz IS NULL OR created_at <= $5)
             ORDER BY created_at DESC LIMIT $6 OFFSET $7"#,
        )
        .bind(&level_str)
        .bind(&filter.source)
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
            r#"SELECT id, level, message, source, metadata, created_at
             FROM log_entries WHERE id = $1"#,
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
            "DELETE FROM log_entries WHERE created_at < NOW() - make_interval(days => $1::int)",
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
    ) -> Result<LogStats, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct LevelCount {
            level: String,
            count: i64,
        }

        let rows = sqlx::query_as::<_, LevelCount>(
            r#"SELECT level, COUNT(*) as count FROM log_entries GROUP BY level ORDER BY count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let total = rows.iter().map(|r| r.count).sum();

        let level_counts: std::collections::HashMap<String, i64> = rows
            .into_iter()
            .map(|r| (r.level, r.count))
            .collect();

        Ok(LogStats {
            total_entries: total,
            level_counts,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogStats {
    pub total_entries: i64,
    pub level_counts: std::collections::HashMap<String, i64>,
}
