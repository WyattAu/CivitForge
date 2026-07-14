use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::types::*;

#[derive(Debug, sqlx::FromRow)]
struct SamplingRuleRow {
    id: Uuid,
    service_name: String,
    endpoint: String,
    sample_rate: f64,
    enabled: bool,
    created_at: DateTime<Utc>,
}

impl From<SamplingRuleRow> for SamplingRule {
    fn from(row: SamplingRuleRow) -> Self {
        SamplingRule {
            id: row.id,
            service_name: row.service_name,
            endpoint: row.endpoint,
            sample_rate: row.sample_rate,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct SamplingRuleV2Row {
    id: Uuid,
    service_name: String,
    endpoint: String,
    sample_rate: f64,
    max_traces_per_second: i32,
    enabled: bool,
    created_at: DateTime<Utc>,
}

impl From<SamplingRuleV2Row> for SamplingRuleV2 {
    fn from(row: SamplingRuleV2Row) -> Self {
        SamplingRuleV2 {
            id: row.id,
            service_name: row.service_name,
            endpoint: row.endpoint,
            sample_rate: row.sample_rate,
            max_traces_per_second: row.max_traces_per_second,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct TraceDependencyRow {
    id: Uuid,
    parent_trace_id: String,
    child_trace_id: String,
    dependency_type: String,
    created_at: DateTime<Utc>,
}

impl From<TraceDependencyRow> for TraceDependency {
    fn from(row: TraceDependencyRow) -> Self {
        TraceDependency {
            id: row.id,
            parent_trace_id: row.parent_trace_id,
            child_trace_id: row.child_trace_id,
            dependency_type: row.dependency_type,
            created_at: row.created_at,
        }
    }
}

pub struct DistributedTracingV2Service {
    pool: PgPool,
}

impl DistributedTracingV2Service {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_rule(
        &self,
        input: CreateSamplingRule,
    ) -> Result<SamplingRule, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleRow>(
            r#"INSERT INTO trace_sampling_rules (service_name, endpoint, sample_rate, enabled)
             VALUES ($1, $2, $3, $4)
             RETURNING id, service_name, endpoint, sample_rate, enabled, created_at"#,
        )
        .bind(&input.service_name)
        .bind(&input.endpoint)
        .bind(input.sample_rate.unwrap_or(1.0))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_rule(
        &self,
        id: Uuid,
    ) -> Result<Option<SamplingRule>, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleRow>(
            r#"SELECT id, service_name, endpoint, sample_rate, enabled, created_at
             FROM trace_sampling_rules WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_rules(&self) -> Result<Vec<SamplingRule>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SamplingRuleRow>(
            r#"SELECT id, service_name, endpoint, sample_rate, enabled, created_at
             FROM trace_sampling_rules ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_rule(
        &self,
        id: Uuid,
        input: UpdateSamplingRule,
    ) -> Result<SamplingRule, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleRow>(
            r#"UPDATE trace_sampling_rules SET
             service_name = COALESCE($2, service_name),
             endpoint = COALESCE($3, endpoint),
             sample_rate = COALESCE($4, sample_rate),
             enabled = COALESCE($5, enabled)
             WHERE id = $1
             RETURNING id, service_name, endpoint, sample_rate, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.service_name)
        .bind(&input.endpoint)
        .bind(input.sample_rate)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_rule(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM trace_sampling_rules WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn should_sample(
        &self,
        service_name: &str,
        endpoint: &str,
    ) -> Result<bool, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleRow>(
            r#"SELECT id, service_name, endpoint, sample_rate, enabled, created_at
             FROM trace_sampling_rules
             WHERE service_name = $1 AND endpoint = $2 AND enabled = true
             ORDER BY created_at DESC
             LIMIT 1"#,
        )
        .bind(service_name)
        .bind(endpoint)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(rule) => {
                let sample = rand::random::<f64>() <= rule.sample_rate;
                Ok(sample)
            }
            None => Ok(true),
        }
    }

    pub async fn record_latency(
        &self,
        trace_id: &str,
        service_name: &str,
        endpoint: &str,
        latency_ms: f64,
    ) -> Result<LatencyRecord, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO trace_sampling_rules (service_name, endpoint, sample_rate, enabled)
             VALUES ($1, $2, $3, $4)"#,
        )
        .bind(service_name)
        .bind(endpoint)
        .bind(latency_ms)
        .bind(true)
        .execute(&self.pool)
        .await?;

        Ok(LatencyRecord {
            id,
            trace_id: trace_id.to_string(),
            service_name: service_name.to_string(),
            endpoint: endpoint.to_string(),
            latency_ms,
            recorded_at: now,
        })
    }

    pub async fn get_sampling_stats(
        &self,
    ) -> Result<SamplingRuleStats, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct StatsRow {
            total: i64,
            enabled: i64,
            avg_rate: f64,
        }

        let row = sqlx::query_as::<_, StatsRow>(
            r#"SELECT
             COUNT(*) as total,
             COUNT(*) FILTER (WHERE enabled) as enabled,
             COALESCE(AVG(sample_rate), 0.0) as avg_rate
             FROM trace_sampling_rules"#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(SamplingRuleStats {
            total_rules: row.total,
            enabled_rules: row.enabled,
            avg_sample_rate: row.avg_rate,
        })
    }

    pub async fn analyze_trace(
        &self,
        trace_id: &str,
        service_name: &str,
        endpoint: &str,
        duration_ms: f64,
        span_count: i64,
        error_count: i64,
    ) -> Result<TraceAnalysis, sqlx::Error> {
        let sampled = self.should_sample(service_name, endpoint).await?;

        Ok(TraceAnalysis {
            trace_id: trace_id.to_string(),
            service_name: service_name.to_string(),
            endpoint: endpoint.to_string(),
            duration_ms,
            span_count,
            error_count,
            sampled,
            analyzed_at: Utc::now(),
        })
    }

    pub async fn correlate_error(
        &self,
        trace_id: &str,
        error_type: &str,
        error_message: &str,
        service_name: &str,
        endpoint: &str,
        span_id: Option<&str>,
    ) -> Result<ErrorCorrelation, sqlx::Error> {
        Ok(ErrorCorrelation {
            trace_id: trace_id.to_string(),
            error_type: error_type.to_string(),
            error_message: error_message.to_string(),
            service_name: service_name.to_string(),
            endpoint: endpoint.to_string(),
            span_id: span_id.map(|s| s.to_string()),
            correlated_at: Utc::now(),
        })
    }

    // V3: Rate-based sampling rules

    pub async fn create_rule_v2(
        &self,
        input: CreateSamplingRuleV2,
    ) -> Result<SamplingRuleV2, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV2Row>(
            r#"INSERT INTO trace_sampling_rules_v2 (service_name, endpoint, sample_rate, max_traces_per_second, enabled)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, service_name, endpoint, sample_rate, max_traces_per_second, enabled, created_at"#,
        )
        .bind(&input.service_name)
        .bind(&input.endpoint)
        .bind(input.sample_rate.unwrap_or(1.0))
        .bind(input.max_traces_per_second.unwrap_or(100))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_rule_v2(
        &self,
        id: Uuid,
    ) -> Result<Option<SamplingRuleV2>, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV2Row>(
            r#"SELECT id, service_name, endpoint, sample_rate, max_traces_per_second, enabled, created_at
             FROM trace_sampling_rules_v2 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_rules_v2(&self) -> Result<Vec<SamplingRuleV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SamplingRuleV2Row>(
            r#"SELECT id, service_name, endpoint, sample_rate, max_traces_per_second, enabled, created_at
             FROM trace_sampling_rules_v2 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_rule_v2(
        &self,
        id: Uuid,
        input: UpdateSamplingRuleV2,
    ) -> Result<SamplingRuleV2, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV2Row>(
            r#"UPDATE trace_sampling_rules_v2 SET
             service_name = COALESCE($2, service_name),
             endpoint = COALESCE($3, endpoint),
             sample_rate = COALESCE($4, sample_rate),
             max_traces_per_second = COALESCE($5, max_traces_per_second),
             enabled = COALESCE($6, enabled)
             WHERE id = $1
             RETURNING id, service_name, endpoint, sample_rate, max_traces_per_second, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.service_name)
        .bind(&input.endpoint)
        .bind(input.sample_rate)
        .bind(input.max_traces_per_second)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_rule_v2(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM trace_sampling_rules_v2 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn should_sample_v2(
        &self,
        service_name: &str,
        endpoint: &str,
    ) -> Result<bool, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV2Row>(
            r#"SELECT id, service_name, endpoint, sample_rate, max_traces_per_second, enabled, created_at
             FROM trace_sampling_rules_v2
             WHERE service_name = $1 AND endpoint = $2 AND enabled = true
             ORDER BY created_at DESC
             LIMIT 1"#,
        )
        .bind(service_name)
        .bind(endpoint)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(rule) => {
                let sample = rand::random::<f64>() <= rule.sample_rate;
                Ok(sample)
            }
            None => Ok(true),
        }
    }

    pub async fn get_sampling_stats_v2(
        &self,
    ) -> Result<SamplingRuleStats, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct StatsRow {
            total: i64,
            enabled: i64,
            avg_rate: f64,
        }

        let row = sqlx::query_as::<_, StatsRow>(
            r#"SELECT
             COUNT(*) as total,
             COUNT(*) FILTER (WHERE enabled) as enabled,
             COALESCE(AVG(sample_rate), 0.0) as avg_rate
             FROM trace_sampling_rules_v2"#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(SamplingRuleStats {
            total_rules: row.total,
            enabled_rules: row.enabled,
            avg_sample_rate: row.avg_rate,
        })
    }

    // V3: Trace dependency tracking

    pub async fn create_dependency(
        &self,
        input: CreateTraceDependency,
    ) -> Result<TraceDependency, sqlx::Error> {
        let row = sqlx::query_as::<_, TraceDependencyRow>(
            r#"INSERT INTO trace_dependencies (parent_trace_id, child_trace_id, dependency_type)
             VALUES ($1, $2, $3)
             RETURNING id, parent_trace_id, child_trace_id, dependency_type, created_at"#,
        )
        .bind(&input.parent_trace_id)
        .bind(&input.child_trace_id)
        .bind(&input.dependency_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_dependencies_for_trace(
        &self,
        trace_id: &str,
    ) -> Result<Vec<TraceDependency>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TraceDependencyRow>(
            r#"SELECT id, parent_trace_id, child_trace_id, dependency_type, created_at
             FROM trace_dependencies
             WHERE parent_trace_id = $1 OR child_trace_id = $1
             ORDER BY created_at ASC"#,
        )
        .bind(trace_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_child_traces(
        &self,
        parent_trace_id: &str,
    ) -> Result<Vec<TraceDependency>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TraceDependencyRow>(
            r#"SELECT id, parent_trace_id, child_trace_id, dependency_type, created_at
             FROM trace_dependencies
             WHERE parent_trace_id = $1
             ORDER BY created_at ASC"#,
        )
        .bind(parent_trace_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_parent_traces(
        &self,
        child_trace_id: &str,
    ) -> Result<Vec<TraceDependency>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TraceDependencyRow>(
            r#"SELECT id, parent_trace_id, child_trace_id, dependency_type, created_at
             FROM trace_dependencies
             WHERE child_trace_id = $1
             ORDER BY created_at ASC"#,
        )
        .bind(child_trace_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn delete_dependency(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM trace_dependencies WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_dependency_stats(
        &self,
    ) -> Result<TraceDependencyStats, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct StatsRow {
            total: i64,
            unique_parents: i64,
            unique_children: i64,
        }

        let row = sqlx::query_as::<_, StatsRow>(
            r#"SELECT
             COUNT(*) as total,
             COUNT(DISTINCT parent_trace_id) as unique_parents,
             COUNT(DISTINCT child_trace_id) as unique_children
             FROM trace_dependencies"#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(TraceDependencyStats {
            total_dependencies: row.total,
            unique_parent_traces: row.unique_parents,
            unique_child_traces: row.unique_children,
        })
    }

    // V3: Latency analysis

    pub async fn record_latency_analysis(
        &self,
        trace_id: &str,
        service_name: &str,
        endpoint: &str,
        latency_ms: f64,
    ) -> Result<LatencyAnalysis, sqlx::Error> {
        let id = Uuid::new_v4();

        #[derive(Debug, sqlx::FromRow)]
        struct LatencyRow {
            id: Uuid,
            trace_id: String,
            service_name: String,
            endpoint: String,
            latency_ms: f64,
            recorded_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, LatencyRow>(
            r#"INSERT INTO trace_dependencies (parent_trace_id, child_trace_id, dependency_type)
             VALUES ($1, $2, $3)
             RETURNING $1::text as parent_trace_id, $2::text as child_trace_id, $3::text as dependency_type"#,
        )
        .bind(trace_id)
        .bind(service_name)
        .bind(endpoint)
        .fetch_one(&self.pool)
        .await?;

        Ok(LatencyAnalysis {
            id,
            trace_id: row.trace_id,
            service_name: row.service_name,
            endpoint: row.endpoint,
            latency_ms,
            recorded_at: Utc::now(),
        })
    }

    pub async fn get_latency_stats(
        &self,
        service_name: Option<&str>,
        endpoint: Option<&str>,
        since: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
    ) -> Result<LatencyStats, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct LatencyRow {
            avg_ms: f64,
            p50_ms: f64,
            p95_ms: f64,
            p99_ms: f64,
            count: i64,
        }

        let row = sqlx::query_as::<_, LatencyRow>(
            r#"SELECT
             COALESCE(AVG(latency_ms), 0.0) as avg_ms,
             COALESCE(PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY latency_ms), 0.0) as p50_ms,
             COALESCE(PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY latency_ms), 0.0) as p95_ms,
             COALESCE(PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY latency_ms), 0.0) as p99_ms,
             COUNT(*) as count
             FROM latency_analysis
             WHERE ($1::text IS NULL OR service_name = $1)
             AND ($2::text IS NULL OR endpoint = $2)
             AND ($3::timestamptz IS NULL OR recorded_at >= $3)
             AND ($4::timestamptz IS NULL OR recorded_at <= $4)"#,
        )
        .bind(service_name)
        .bind(endpoint)
        .bind(since)
        .bind(until)
        .fetch_one(&self.pool)
        .await?;

        Ok(LatencyStats {
            avg_latency_ms: row.avg_ms,
            p50_latency_ms: row.p50_ms,
            p95_latency_ms: row.p95_ms,
            p99_latency_ms: row.p99_ms,
            sample_count: row.count,
        })
    }

    // V3: Error correlation

    pub async fn correlate_error_v3(
        &self,
        trace_id: &str,
        error_type: &str,
        error_message: &str,
        service_name: &str,
        endpoint: &str,
        span_id: Option<&str>,
    ) -> Result<ErrorCorrelationV3, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        Ok(ErrorCorrelationV3 {
            id,
            trace_id: trace_id.to_string(),
            error_type: error_type.to_string(),
            error_message: error_message.to_string(),
            service_name: service_name.to_string(),
            endpoint: endpoint.to_string(),
            span_id: span_id.map(|s| s.to_string()),
            correlated_at: now,
        })
    }

    // V4: Priority-based sampling rules

    pub async fn create_rule_v3(
        &self,
        input: CreateSamplingRuleV3,
    ) -> Result<SamplingRuleV3, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV3Row>(
            r#"INSERT INTO trace_sampling_rules_v3 (service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at"#,
        )
        .bind(&input.service_name)
        .bind(&input.endpoint)
        .bind(input.sample_rate.unwrap_or(1.0))
        .bind(input.max_traces_per_second.unwrap_or(100))
        .bind(input.priority.unwrap_or(0))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_rule_v3(
        &self,
        id: Uuid,
    ) -> Result<Option<SamplingRuleV3>, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV3Row>(
            r#"SELECT id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at
             FROM trace_sampling_rules_v3 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_rules_v3(&self) -> Result<Vec<SamplingRuleV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SamplingRuleV3Row>(
            r#"SELECT id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at
             FROM trace_sampling_rules_v3 ORDER BY priority DESC, created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_rule_v3(
        &self,
        id: Uuid,
        input: UpdateSamplingRuleV3,
    ) -> Result<SamplingRuleV3, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV3Row>(
            r#"UPDATE trace_sampling_rules_v3 SET
             service_name = COALESCE($2, service_name),
             endpoint = COALESCE($3, endpoint),
             sample_rate = COALESCE($4, sample_rate),
             max_traces_per_second = COALESCE($5, max_traces_per_second),
             priority = COALESCE($6, priority),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.service_name)
        .bind(&input.endpoint)
        .bind(input.sample_rate)
        .bind(input.max_traces_per_second)
        .bind(input.priority)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_rule_v3(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM trace_sampling_rules_v3 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn should_sample_v3(
        &self,
        service_name: &str,
        endpoint: &str,
    ) -> Result<bool, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV3Row>(
            r#"SELECT id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at
             FROM trace_sampling_rules_v3
             WHERE service_name = $1 AND endpoint = $2 AND enabled = true
             ORDER BY priority DESC, created_at DESC
             LIMIT 1"#,
        )
        .bind(service_name)
        .bind(endpoint)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(rule) => {
                let sample = rand::random::<f64>() <= rule.sample_rate;
                Ok(sample)
            }
            None => Ok(true),
        }
    }

    pub async fn get_sampling_stats_v3(
        &self,
    ) -> Result<SamplingRuleStats, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct StatsRow {
            total: i64,
            enabled: i64,
            avg_rate: f64,
        }

        let row = sqlx::query_as::<_, StatsRow>(
            r#"SELECT
             COUNT(*) as total,
             COUNT(*) FILTER (WHERE enabled) as enabled,
             COALESCE(AVG(sample_rate), 0.0) as avg_rate
             FROM trace_sampling_rules_v3"#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(SamplingRuleStats {
            total_rules: row.total,
            enabled_rules: row.enabled,
            avg_sample_rate: row.avg_rate,
        })
    }

    // V4: Service map generation

    pub async fn update_service_map(
        &self,
        service_name: &str,
        endpoint: &str,
        duration_ms: f64,
        is_error: bool,
    ) -> Result<ServiceMapEntry, sqlx::Error> {
        let row = sqlx::query_as::<_, ServiceMapEntryRow>(
            r#"INSERT INTO trace_service_map (service_name, endpoint, call_count, avg_duration_ms, error_rate)
             VALUES ($1, $2, 1, $3, $4)
             ON CONFLICT (service_name, endpoint) DO UPDATE SET
             call_count = trace_service_map.call_count + 1,
             avg_duration_ms = (trace_service_map.avg_duration_ms * trace_service_map.call_count + $3) / (trace_service_map.call_count + 1),
             error_rate = (trace_service_map.error_rate * trace_service_map.call_count + $4) / (trace_service_map.call_count + 1),
             last_updated_at = NOW()
             RETURNING id, service_name, endpoint, call_count, avg_duration_ms, error_rate, last_updated_at"#,
        )
        .bind(service_name)
        .bind(endpoint)
        .bind(duration_ms)
        .bind(if is_error { 1.0 } else { 0.0 })
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_service_map(
        &self,
    ) -> Result<ServiceMap, sqlx::Error> {
        let rows = sqlx::query_as::<_, ServiceMapEntryRow>(
            r#"SELECT id, service_name, endpoint, call_count, avg_duration_ms, error_rate, last_updated_at
             FROM trace_service_map
             ORDER BY call_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let total_endpoints = rows.len() as i64;
        let total_services = rows
            .iter()
            .map(|r| &r.service_name)
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;

        Ok(ServiceMap {
            services: rows.into_iter().map(|r| r.into()).collect(),
            total_services,
            total_endpoints,
        })
    }

    pub async fn get_dependencies(
        &self,
    ) -> Result<DependencyAnalysis, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct DependencyRow {
            from_service: String,
            to_service: String,
            call_count: i64,
            avg_duration_ms: f64,
            error_rate: f64,
        }

        let rows = sqlx::query_as::<_, DependencyRow>(
            r#"SELECT
             a.service_name as from_service,
             b.service_name as to_service,
             SUM(a.call_count) as call_count,
             AVG(b.avg_duration_ms) as avg_duration_ms,
             AVG(b.error_rate) as error_rate
             FROM trace_service_map a
             CROSS JOIN trace_service_map b
             WHERE a.service_name != b.service_name
             GROUP BY a.service_name, b.service_name
             HAVING SUM(a.call_count) > 0
             ORDER BY call_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let critical_paths: Vec<String> = rows
            .iter()
            .filter(|r| r.error_rate > 0.1 || r.avg_duration_ms > 1000.0)
            .map(|r| format!("{} -> {}", r.from_service, r.to_service))
            .collect();

        let total_dependencies = rows.len() as i64;

        Ok(DependencyAnalysis {
            dependencies: rows
                .into_iter()
                .map(|r| ServiceDependency {
                    from_service: r.from_service,
                    to_service: r.to_service,
                    call_count: r.call_count,
                    avg_duration_ms: r.avg_duration_ms,
                    error_rate: r.error_rate,
                })
                .collect(),
            total_dependencies,
            critical_paths,
        })
    }

    pub async fn get_capacity_planning(
        &self,
    ) -> Result<Vec<CapacityPlanningData>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct CapacityRow {
            service_name: String,
            current_load: f64,
            bottleneck_endpoints: Vec<String>,
        }

        let rows = sqlx::query_as::<_, CapacityRow>(
            r#"SELECT
             service_name,
             SUM(call_count)::double precision as current_load,
             ARRAY_AGG(endpoint) FILTER (WHERE avg_duration_ms > 500) as bottleneck_endpoints
             FROM trace_service_map
             GROUP BY service_name
             ORDER BY current_load DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| CapacityPlanningData {
                service_name: r.service_name,
                current_load: r.current_load,
                projected_load: r.current_load * 1.5,
                recommended_capacity: r.current_load * 2.0,
                bottleneck_endpoints: r.bottleneck_endpoints,
            })
            .collect())
    }

    // V4: Priority-based sampling rules

    pub async fn create_rule_v4(
        &self,
        input: CreateSamplingRuleV4,
    ) -> Result<SamplingRuleV4, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV4Row>(
            r#"INSERT INTO trace_sampling_rules_v4 (service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at"#,
        )
        .bind(&input.service_name)
        .bind(&input.endpoint)
        .bind(input.sample_rate.unwrap_or(1.0))
        .bind(input.max_traces_per_second.unwrap_or(100))
        .bind(input.priority.unwrap_or(0))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_rule_v4(
        &self,
        id: Uuid,
    ) -> Result<Option<SamplingRuleV4>, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV4Row>(
            r#"SELECT id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at
             FROM trace_sampling_rules_v4 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_rules_v4(&self) -> Result<Vec<SamplingRuleV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SamplingRuleV4Row>(
            r#"SELECT id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at
             FROM trace_sampling_rules_v4 ORDER BY priority DESC, created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_rule_v4(
        &self,
        id: Uuid,
        input: UpdateSamplingRuleV4,
    ) -> Result<SamplingRuleV4, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV4Row>(
            r#"UPDATE trace_sampling_rules_v4 SET
             service_name = COALESCE($2, service_name),
             endpoint = COALESCE($3, endpoint),
             sample_rate = COALESCE($4, sample_rate),
             max_traces_per_second = COALESCE($5, max_traces_per_second),
             priority = COALESCE($6, priority),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.service_name)
        .bind(&input.endpoint)
        .bind(input.sample_rate)
        .bind(input.max_traces_per_second)
        .bind(input.priority)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_rule_v4(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM trace_sampling_rules_v4 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn should_sample_v4(
        &self,
        service_name: &str,
        endpoint: &str,
    ) -> Result<bool, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV4Row>(
            r#"SELECT id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at
             FROM trace_sampling_rules_v4
             WHERE service_name = $1 AND endpoint = $2 AND enabled = true
             ORDER BY priority DESC, created_at DESC
             LIMIT 1"#,
        )
        .bind(service_name)
        .bind(endpoint)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(rule) => {
                let sample = rand::random::<f64>() <= rule.sample_rate;
                Ok(sample)
            }
            None => Ok(true),
        }
    }

    pub async fn get_sampling_stats_v4(
        &self,
    ) -> Result<SamplingRuleStats, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct StatsRow {
            total: i64,
            enabled: i64,
            avg_rate: f64,
        }

        let row = sqlx::query_as::<_, StatsRow>(
            r#"SELECT
             COUNT(*) as total,
             COUNT(*) FILTER (WHERE enabled) as enabled,
             COALESCE(AVG(sample_rate), 0.0) as avg_rate
             FROM trace_sampling_rules_v4"#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(SamplingRuleStats {
            total_rules: row.total,
            enabled_rules: row.enabled,
            avg_sample_rate: row.avg_rate,
        })
    }

    // V4: Service dependency tracking

    pub async fn create_service_dependency(
        &self,
        input: CreateTraceServiceDependency,
    ) -> Result<TraceServiceDependency, sqlx::Error> {
        let row = sqlx::query_as::<_, TraceServiceDependencyRow>(
            r#"INSERT INTO trace_service_dependencies (service_name, depends_on_service, call_count, avg_duration_ms, error_rate)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, service_name, depends_on_service, call_count, avg_duration_ms, error_rate, last_updated_at"#,
        )
        .bind(&input.service_name)
        .bind(&input.depends_on_service)
        .bind(input.call_count.unwrap_or(0))
        .bind(input.avg_duration_ms.unwrap_or(0.0))
        .bind(input.error_rate.unwrap_or(0.0))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_service_dependencies(
        &self,
    ) -> Result<ServiceDependencyGraph, sqlx::Error> {
        let rows = sqlx::query_as::<_, TraceServiceDependencyRow>(
            r#"SELECT id, service_name, depends_on_service, call_count, avg_duration_ms, error_rate, last_updated_at
             FROM trace_service_dependencies
             ORDER BY call_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let total_services = rows
            .iter()
            .map(|r| &r.service_name)
            .chain(rows.iter().map(|r| &r.depends_on_service))
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;

        let total_dependencies = rows.len() as i64;

        Ok(ServiceDependencyGraph {
            dependencies: rows.into_iter().map(|r| r.into()).collect(),
            total_services,
            total_dependencies,
        })
    }

    pub async fn get_dependencies_for_service(
        &self,
        service_name: &str,
    ) -> Result<Vec<TraceServiceDependency>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TraceServiceDependencyRow>(
            r#"SELECT id, service_name, depends_on_service, call_count, avg_duration_ms, error_rate, last_updated_at
             FROM trace_service_dependencies
             WHERE service_name = $1
             ORDER BY call_count DESC"#,
        )
        .bind(service_name)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_service_dependency(
        &self,
        id: Uuid,
        call_count: i64,
        avg_duration_ms: f64,
        error_rate: f64,
    ) -> Result<TraceServiceDependency, sqlx::Error> {
        let row = sqlx::query_as::<_, TraceServiceDependencyRow>(
            r#"UPDATE trace_service_dependencies SET
             call_count = $2,
             avg_duration_ms = $3,
             error_rate = $4,
             last_updated_at = NOW()
             WHERE id = $1
             RETURNING id, service_name, depends_on_service, call_count, avg_duration_ms, error_rate, last_updated_at"#,
        )
        .bind(id)
        .bind(call_count)
        .bind(avg_duration_ms)
        .bind(error_rate)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_service_dependency(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM trace_service_dependencies WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // V5: Priority-based sampling rules v5

    pub async fn create_rule_v5(
        &self,
        input: CreateSamplingRuleV5,
    ) -> Result<SamplingRuleV5, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV5Row>(
            r#"INSERT INTO trace_sampling_rules_v5 (service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at"#,
        )
        .bind(&input.service_name)
        .bind(&input.endpoint)
        .bind(input.sample_rate.unwrap_or(1.0))
        .bind(input.max_traces_per_second.unwrap_or(100))
        .bind(input.priority.unwrap_or(0))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_rule_v5(
        &self,
        id: Uuid,
    ) -> Result<Option<SamplingRuleV5>, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV5Row>(
            r#"SELECT id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at
             FROM trace_sampling_rules_v5 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn list_rules_v5(&self) -> Result<Vec<SamplingRuleV5>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SamplingRuleV5Row>(
            r#"SELECT id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at
             FROM trace_sampling_rules_v5 ORDER BY priority DESC, created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_rule_v5(
        &self,
        id: Uuid,
        input: UpdateSamplingRuleV5,
    ) -> Result<SamplingRuleV5, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV5Row>(
            r#"UPDATE trace_sampling_rules_v5 SET
             service_name = COALESCE($2, service_name),
             endpoint = COALESCE($3, endpoint),
             sample_rate = COALESCE($4, sample_rate),
             max_traces_per_second = COALESCE($5, max_traces_per_second),
             priority = COALESCE($6, priority),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.service_name)
        .bind(&input.endpoint)
        .bind(input.sample_rate)
        .bind(input.max_traces_per_second)
        .bind(input.priority)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn delete_rule_v5(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM trace_sampling_rules_v5 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn should_sample_v5(
        &self,
        service_name: &str,
        endpoint: &str,
    ) -> Result<bool, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV5Row>(
            r#"SELECT id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at
             FROM trace_sampling_rules_v5
             WHERE service_name = $1 AND endpoint = $2 AND enabled = true
             ORDER BY priority DESC, created_at DESC
             LIMIT 1"#,
        )
        .bind(service_name)
        .bind(endpoint)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(rule) => {
                let sample = rand::random::<f64>() <= rule.sample_rate;
                Ok(sample)
            }
            None => Ok(true),
        }
    }

    pub async fn get_sampling_stats_v5(
        &self,
    ) -> Result<SamplingRuleStats, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct StatsRow {
            total: i64,
            enabled: i64,
            avg_rate: f64,
        }
        let row = sqlx::query_as::<_, StatsRow>(
            r#"SELECT
             COUNT(*) as total,
             COUNT(*) FILTER (WHERE enabled) as enabled,
             COALESCE(AVG(sample_rate), 0.0) as avg_rate
             FROM trace_sampling_rules_v5"#,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(SamplingRuleStats {
            total_rules: row.total,
            enabled_rules: row.enabled,
            avg_sample_rate: row.avg_rate,
        })
    }

    // V5: Service dependency tracking v2

    pub async fn create_service_dependency_v2(
        &self,
        input: CreateTraceServiceDependencyV2,
    ) -> Result<TraceServiceDependencyV2, sqlx::Error> {
        let row = sqlx::query_as::<_, TraceServiceDependencyV2Row>(
            r#"INSERT INTO trace_service_dependencies_v2 (service_name, depends_on_service, call_count, avg_duration_ms, error_rate)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, service_name, depends_on_service, call_count, avg_duration_ms, error_rate, last_updated_at"#,
        )
        .bind(&input.service_name)
        .bind(&input.depends_on_service)
        .bind(input.call_count.unwrap_or(0))
        .bind(input.avg_duration_ms.unwrap_or(0.0))
        .bind(input.error_rate.unwrap_or(0.0))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_service_dependencies_v2(
        &self,
    ) -> Result<ServiceDependencyGraphV2, sqlx::Error> {
        let rows = sqlx::query_as::<_, TraceServiceDependencyV2Row>(
            r#"SELECT id, service_name, depends_on_service, call_count, avg_duration_ms, error_rate, last_updated_at
             FROM trace_service_dependencies_v2
             ORDER BY call_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        let total_services = rows
            .iter()
            .map(|r| &r.service_name)
            .chain(rows.iter().map(|r| &r.depends_on_service))
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;
        let total_dependencies = rows.len() as i64;
        Ok(ServiceDependencyGraphV2 {
            dependencies: rows.into_iter().map(|r| r.into()).collect(),
            total_services,
            total_dependencies,
        })
    }

    pub async fn get_dependencies_for_service_v2(
        &self,
        service_name: &str,
    ) -> Result<Vec<TraceServiceDependencyV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TraceServiceDependencyV2Row>(
            r#"SELECT id, service_name, depends_on_service, call_count, avg_duration_ms, error_rate, last_updated_at
             FROM trace_service_dependencies_v2
             WHERE service_name = $1
             ORDER BY call_count DESC"#,
        )
        .bind(service_name)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_service_dependency_v2(
        &self,
        id: Uuid,
        call_count: i64,
        avg_duration_ms: f64,
        error_rate: f64,
    ) -> Result<TraceServiceDependencyV2, sqlx::Error> {
        let row = sqlx::query_as::<_, TraceServiceDependencyV2Row>(
            r#"UPDATE trace_service_dependencies_v2 SET
             call_count = $2,
             avg_duration_ms = $3,
             error_rate = $4,
             last_updated_at = NOW()
             WHERE id = $1
             RETURNING id, service_name, depends_on_service, call_count, avg_duration_ms, error_rate, last_updated_at"#,
        )
        .bind(id)
        .bind(call_count)
        .bind(avg_duration_ms)
        .bind(error_rate)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn delete_service_dependency_v2(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM trace_service_dependencies_v2 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    // V5: Latency analysis v2

    pub async fn get_latency_stats_v2(
        &self,
        service_name: Option<&str>,
        endpoint: Option<&str>,
        since: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
    ) -> Result<LatencyAnalysisV2, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct LatencyRow {
            svc: String,
            ep: String,
            avg_ms: f64,
            p50_ms: f64,
            p95_ms: f64,
            p99_ms: f64,
            count: i64,
        }
        let row = sqlx::query_as::<_, LatencyRow>(
            r#"SELECT
             service_name as svc,
             endpoint as ep,
             COALESCE(AVG(latency_ms), 0.0) as avg_ms,
             COALESCE(PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY latency_ms), 0.0) as p50_ms,
             COALESCE(PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY latency_ms), 0.0) as p95_ms,
             COALESCE(PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY latency_ms), 0.0) as p99_ms,
             COUNT(*) as count
             FROM latency_analysis
             WHERE ($1::text IS NULL OR service_name = $1)
             AND ($2::text IS NULL OR endpoint = $2)
             AND ($3::timestamptz IS NULL OR recorded_at >= $3)
             AND ($4::timestamptz IS NULL OR recorded_at <= $4)
             GROUP BY service_name, endpoint
             LIMIT 1"#,
        )
        .bind(service_name)
        .bind(endpoint)
        .bind(since)
        .bind(until)
        .fetch_one(&self.pool)
        .await?;
        Ok(LatencyAnalysisV2 {
            service_name: row.svc,
            endpoint: row.ep,
            avg_latency_ms: row.avg_ms,
            p50_latency_ms: row.p50_ms,
            p95_latency_ms: row.p95_ms,
            p99_latency_ms: row.p99_ms,
            sample_count: row.count,
        })
    }

    // V5: Error correlation v4

    pub async fn correlate_error_v4(
        &self,
        trace_id: &str,
        error_type: &str,
        error_message: &str,
        service_name: &str,
        endpoint: &str,
        span_id: Option<&str>,
    ) -> Result<ErrorCorrelationV4, sqlx::Error> {
        let id = Uuid::new_v4();
        Ok(ErrorCorrelationV4 {
            id,
            trace_id: trace_id.to_string(),
            error_type: error_type.to_string(),
            error_message: error_message.to_string(),
            service_name: service_name.to_string(),
            endpoint: endpoint.to_string(),
            span_id: span_id.map(|s| s.to_string()),
            correlated_at: Utc::now(),
        })
    }

    // V5: Capacity planning v2

    pub async fn get_capacity_planning_v2(
        &self,
    ) -> Result<Vec<CapacityPlanningDataV2>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct CapacityRow {
            service_name: String,
            current_load: f64,
            bottleneck_endpoints: Vec<String>,
        }
        let rows = sqlx::query_as::<_, CapacityRow>(
            r#"SELECT
             service_name,
             SUM(call_count)::double precision as current_load,
             ARRAY_AGG(endpoint) FILTER (WHERE avg_duration_ms > 500) as bottleneck_endpoints
             FROM trace_service_map
             GROUP BY service_name
             ORDER BY current_load DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| CapacityPlanningDataV2 {
                service_name: r.service_name,
                current_load: r.current_load,
                projected_load: r.current_load * 1.5,
                recommended_capacity: r.current_load * 2.0,
                bottleneck_endpoints: r.bottleneck_endpoints,
                growth_rate: 0.5,
                time_to_capacity_hours: if r.current_load > 0.0 { (r.current_load / (r.current_load * 0.5)) * 24.0 } else { f64::INFINITY },
            })
            .collect())
    }

    // V7: Priority-based sampling rules v6

    pub async fn create_rule_v6(
        &self,
        input: CreateSamplingRuleV6,
    ) -> Result<SamplingRuleV6, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV6Row>(
            r#"INSERT INTO trace_sampling_rules_v6 (service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at"#,
        )
        .bind(&input.service_name)
        .bind(&input.endpoint)
        .bind(input.sample_rate.unwrap_or(1.0))
        .bind(input.max_traces_per_second.unwrap_or(100))
        .bind(input.priority.unwrap_or(0))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_rule_v6(
        &self,
        id: Uuid,
    ) -> Result<Option<SamplingRuleV6>, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV6Row>(
            r#"SELECT id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at
             FROM trace_sampling_rules_v6 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn list_rules_v6(&self) -> Result<Vec<SamplingRuleV6>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SamplingRuleV6Row>(
            r#"SELECT id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at
             FROM trace_sampling_rules_v6 ORDER BY priority DESC, created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_rule_v6(
        &self,
        id: Uuid,
        input: UpdateSamplingRuleV6,
    ) -> Result<SamplingRuleV6, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV6Row>(
            r#"UPDATE trace_sampling_rules_v6 SET
             service_name = COALESCE($2, service_name),
             endpoint = COALESCE($3, endpoint),
             sample_rate = COALESCE($4, sample_rate),
             max_traces_per_second = COALESCE($5, max_traces_per_second),
             priority = COALESCE($6, priority),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.service_name)
        .bind(&input.endpoint)
        .bind(input.sample_rate)
        .bind(input.max_traces_per_second)
        .bind(input.priority)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn delete_rule_v6(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM trace_sampling_rules_v6 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn should_sample_v6(
        &self,
        service_name: &str,
        endpoint: &str,
    ) -> Result<bool, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV6Row>(
            r#"SELECT id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at
             FROM trace_sampling_rules_v6
             WHERE service_name = $1 AND endpoint = $2 AND enabled = true
             ORDER BY priority DESC, created_at DESC
             LIMIT 1"#,
        )
        .bind(service_name)
        .bind(endpoint)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(rule) => {
                let sample = rand::random::<f64>() <= rule.sample_rate;
                Ok(sample)
            }
            None => Ok(true),
        }
    }

    pub async fn get_sampling_stats_v6(
        &self,
    ) -> Result<SamplingRuleStats, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct StatsRow {
            total: i64,
            enabled: i64,
            avg_rate: f64,
        }
        let row = sqlx::query_as::<_, StatsRow>(
            r#"SELECT
             COUNT(*) as total,
             COUNT(*) FILTER (WHERE enabled) as enabled,
             COALESCE(AVG(sample_rate), 0.0) as avg_rate
             FROM trace_sampling_rules_v6"#,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(SamplingRuleStats {
            total_rules: row.total,
            enabled_rules: row.enabled,
            avg_sample_rate: row.avg_rate,
        })
    }

    // V7: Service dependency tracking v3

    pub async fn create_service_dependency_v3(
        &self,
        input: CreateTraceServiceDependencyV3,
    ) -> Result<TraceServiceDependencyV3, sqlx::Error> {
        let row = sqlx::query_as::<_, TraceServiceDependencyV3Row>(
            r#"INSERT INTO trace_service_dependencies_v3 (service_name, depends_on_service, call_count, avg_duration_ms, error_rate)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, service_name, depends_on_service, call_count, avg_duration_ms, error_rate, last_updated_at"#,
        )
        .bind(&input.service_name)
        .bind(&input.depends_on_service)
        .bind(input.call_count.unwrap_or(0))
        .bind(input.avg_duration_ms.unwrap_or(0.0))
        .bind(input.error_rate.unwrap_or(0.0))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_service_dependencies_v3(
        &self,
    ) -> Result<ServiceDependencyGraphV3, sqlx::Error> {
        let rows = sqlx::query_as::<_, TraceServiceDependencyV3Row>(
            r#"SELECT id, service_name, depends_on_service, call_count, avg_duration_ms, error_rate, last_updated_at
             FROM trace_service_dependencies_v3
             ORDER BY call_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        let total_services = rows
            .iter()
            .map(|r| &r.service_name)
            .chain(rows.iter().map(|r| &r.depends_on_service))
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;
        let total_dependencies = rows.len() as i64;
        Ok(ServiceDependencyGraphV3 {
            dependencies: rows.into_iter().map(|r| r.into()).collect(),
            total_services,
            total_dependencies,
        })
    }

    pub async fn get_dependencies_for_service_v3(
        &self,
        service_name: &str,
    ) -> Result<Vec<TraceServiceDependencyV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TraceServiceDependencyV3Row>(
            r#"SELECT id, service_name, depends_on_service, call_count, avg_duration_ms, error_rate, last_updated_at
             FROM trace_service_dependencies_v3
             WHERE service_name = $1
             ORDER BY call_count DESC"#,
        )
        .bind(service_name)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_service_dependency_v3(
        &self,
        id: Uuid,
        call_count: i64,
        avg_duration_ms: f64,
        error_rate: f64,
    ) -> Result<TraceServiceDependencyV3, sqlx::Error> {
        let row = sqlx::query_as::<_, TraceServiceDependencyV3Row>(
            r#"UPDATE trace_service_dependencies_v3 SET
             call_count = $2,
             avg_duration_ms = $3,
             error_rate = $4,
             last_updated_at = NOW()
             WHERE id = $1
             RETURNING id, service_name, depends_on_service, call_count, avg_duration_ms, error_rate, last_updated_at"#,
        )
        .bind(id)
        .bind(call_count)
        .bind(avg_duration_ms)
        .bind(error_rate)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn delete_service_dependency_v3(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM trace_service_dependencies_v3 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    // V7: Latency analysis v3

    pub async fn get_latency_stats_v3(
        &self,
        service_name: Option<&str>,
        endpoint: Option<&str>,
        since: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
    ) -> Result<Vec<LatencyAnalysisV3>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct LatencyRow {
            svc: String,
            ep: String,
            avg_ms: f64,
            p50_ms: f64,
            p95_ms: f64,
            p99_ms: f64,
            count: i64,
        }
        let rows = sqlx::query_as::<_, LatencyRow>(
            r#"SELECT
             service_name as svc,
             endpoint as ep,
             COALESCE(AVG(latency_ms), 0.0) as avg_ms,
             COALESCE(PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY latency_ms), 0.0) as p50_ms,
             COALESCE(PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY latency_ms), 0.0) as p95_ms,
             COALESCE(PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY latency_ms), 0.0) as p99_ms,
             COUNT(*) as count
             FROM latency_analysis
             WHERE ($1::text IS NULL OR service_name = $1)
             AND ($2::text IS NULL OR endpoint = $2)
             AND ($3::timestamptz IS NULL OR recorded_at >= $3)
             AND ($4::timestamptz IS NULL OR recorded_at <= $4)
             GROUP BY service_name, endpoint
             ORDER BY count DESC"#,
        )
        .bind(service_name)
        .bind(endpoint)
        .bind(since)
        .bind(until)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| LatencyAnalysisV3 {
                service_name: r.svc,
                endpoint: r.ep,
                avg_latency_ms: r.avg_ms,
                p50_latency_ms: r.p50_ms,
                p95_latency_ms: r.p95_ms,
                p99_latency_ms: r.p99_ms,
                sample_count: r.count,
            })
            .collect())
    }

    // V7: Error correlation v5

    pub async fn correlate_error_v5(
        &self,
        trace_id: &str,
        error_type: &str,
        error_message: &str,
        service_name: &str,
        endpoint: &str,
        span_id: Option<&str>,
    ) -> Result<ErrorCorrelationV5, sqlx::Error> {
        let id = Uuid::new_v4();
        Ok(ErrorCorrelationV5 {
            id,
            trace_id: trace_id.to_string(),
            error_type: error_type.to_string(),
            error_message: error_message.to_string(),
            service_name: service_name.to_string(),
            endpoint: endpoint.to_string(),
            span_id: span_id.map(|s| s.to_string()),
            correlated_at: Utc::now(),
        })
    }

    // V7: Capacity planning v3

    pub async fn get_capacity_planning_v3(
        &self,
    ) -> Result<Vec<CapacityPlanningDataV3>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct CapacityRow {
            service_name: String,
            current_load: f64,
            bottleneck_endpoints: Vec<String>,
        }
        let rows = sqlx::query_as::<_, CapacityRow>(
            r#"SELECT
             service_name,
             SUM(call_count)::double precision as current_load,
             ARRAY_AGG(endpoint) FILTER (WHERE avg_duration_ms > 500) as bottleneck_endpoints
             FROM trace_service_map
             GROUP BY service_name
             ORDER BY current_load DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                let utilization = if r.current_load > 0.0 { (r.current_load / (r.current_load * 2.0)).min(1.0) } else { 0.0 };
                CapacityPlanningDataV3 {
                    service_name: r.service_name,
                    current_load: r.current_load,
                    projected_load: r.current_load * 1.5,
                    recommended_capacity: r.current_load * 2.0,
                    bottleneck_endpoints: r.bottleneck_endpoints,
                    growth_rate: 0.5,
                    time_to_capacity_hours: if r.current_load > 0.0 { (r.current_load / (r.current_load * 0.5)) * 24.0 } else { f64::INFINITY },
                    utilization_score: utilization,
                }
            })
            .collect())
    }

    // V8: Sampling rules v7

    pub async fn create_rule_v7(
        &self,
        input: CreateSamplingRuleV7,
    ) -> Result<SamplingRuleV7, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV7Row>(
            r#"INSERT INTO trace_sampling_rules_v7 (service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at"#,
        )
        .bind(&input.service_name)
        .bind(&input.endpoint)
        .bind(input.sample_rate.unwrap_or(1.0))
        .bind(input.max_traces_per_second.unwrap_or(100))
        .bind(input.priority.unwrap_or(0))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_rule_v7(
        &self,
        id: Uuid,
    ) -> Result<Option<SamplingRuleV7>, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV7Row>(
            r#"SELECT id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at
             FROM trace_sampling_rules_v7 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    pub async fn list_rules_v7(&self) -> Result<Vec<SamplingRuleV7>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SamplingRuleV7Row>(
            r#"SELECT id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at
             FROM trace_sampling_rules_v7 ORDER BY priority DESC, created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_rule_v7(
        &self,
        id: Uuid,
        input: UpdateSamplingRuleV7,
    ) -> Result<SamplingRuleV7, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV7Row>(
            r#"UPDATE trace_sampling_rules_v7 SET
             service_name = COALESCE($2, service_name),
             endpoint = COALESCE($3, endpoint),
             sample_rate = COALESCE($4, sample_rate),
             max_traces_per_second = COALESCE($5, max_traces_per_second),
             priority = COALESCE($6, priority),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.service_name)
        .bind(&input.endpoint)
        .bind(input.sample_rate)
        .bind(input.max_traces_per_second)
        .bind(input.priority)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn delete_rule_v7(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM trace_sampling_rules_v7 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn should_sample_v7(
        &self,
        service_name: &str,
        endpoint: &str,
    ) -> Result<bool, sqlx::Error> {
        let row = sqlx::query_as::<_, SamplingRuleV7Row>(
            r#"SELECT id, service_name, endpoint, sample_rate, max_traces_per_second, priority, enabled, created_at
             FROM trace_sampling_rules_v7
             WHERE service_name = $1 AND endpoint = $2 AND enabled = true
             ORDER BY priority DESC, created_at DESC
             LIMIT 1"#,
        )
        .bind(service_name)
        .bind(endpoint)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(rule) => {
                let sample = rand::random::<f64>() <= rule.sample_rate;
                Ok(sample)
            }
            None => Ok(true),
        }
    }

    pub async fn get_sampling_stats_v7(
        &self,
    ) -> Result<SamplingRuleStats, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct StatsRow {
            total: i64,
            enabled: i64,
            avg_rate: f64,
        }
        let row = sqlx::query_as::<_, StatsRow>(
            r#"SELECT
             COUNT(*) as total,
             COUNT(*) FILTER (WHERE enabled) as enabled,
             COALESCE(AVG(sample_rate), 0.0) as avg_rate
             FROM trace_sampling_rules_v7"#,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(SamplingRuleStats {
            total_rules: row.total,
            enabled_rules: row.enabled,
            avg_sample_rate: row.avg_rate,
        })
    }

    // V8: Service dependency tracking v4

    pub async fn create_service_dependency_v4(
        &self,
        input: CreateTraceServiceDependencyV4,
    ) -> Result<TraceServiceDependencyV4, sqlx::Error> {
        let row = sqlx::query_as::<_, TraceServiceDependencyV4Row>(
            r#"INSERT INTO trace_service_dependencies_v4 (service_name, depends_on_service, call_count, avg_duration_ms, error_rate)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, service_name, depends_on_service, call_count, avg_duration_ms, error_rate, last_updated_at"#,
        )
        .bind(&input.service_name)
        .bind(&input.depends_on_service)
        .bind(input.call_count.unwrap_or(0))
        .bind(input.avg_duration_ms.unwrap_or(0.0))
        .bind(input.error_rate.unwrap_or(0.0))
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn get_service_dependencies_v4(
        &self,
    ) -> Result<ServiceDependencyGraphV4, sqlx::Error> {
        let rows = sqlx::query_as::<_, TraceServiceDependencyV4Row>(
            r#"SELECT id, service_name, depends_on_service, call_count, avg_duration_ms, error_rate, last_updated_at
             FROM trace_service_dependencies_v4
             ORDER BY call_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        let total_services = rows
            .iter()
            .map(|r| &r.service_name)
            .chain(rows.iter().map(|r| &r.depends_on_service))
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;
        let total_dependencies = rows.len() as i64;
        Ok(ServiceDependencyGraphV4 {
            dependencies: rows.into_iter().map(|r| r.into()).collect(),
            total_services,
            total_dependencies,
        })
    }

    pub async fn get_dependencies_for_service_v4(
        &self,
        service_name: &str,
    ) -> Result<Vec<TraceServiceDependencyV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TraceServiceDependencyV4Row>(
            r#"SELECT id, service_name, depends_on_service, call_count, avg_duration_ms, error_rate, last_updated_at
             FROM trace_service_dependencies_v4
             WHERE service_name = $1
             ORDER BY call_count DESC"#,
        )
        .bind(service_name)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_service_dependency_v4(
        &self,
        id: Uuid,
        call_count: i64,
        avg_duration_ms: f64,
        error_rate: f64,
    ) -> Result<TraceServiceDependencyV4, sqlx::Error> {
        let row = sqlx::query_as::<_, TraceServiceDependencyV4Row>(
            r#"UPDATE trace_service_dependencies_v4 SET
             call_count = $2,
             avg_duration_ms = $3,
             error_rate = $4,
             last_updated_at = NOW()
             WHERE id = $1
             RETURNING id, service_name, depends_on_service, call_count, avg_duration_ms, error_rate, last_updated_at"#,
        )
        .bind(id)
        .bind(call_count)
        .bind(avg_duration_ms)
        .bind(error_rate)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    pub async fn delete_service_dependency_v4(
        &self,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM trace_service_dependencies_v4 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    // V8: Latency analysis v4

    pub async fn get_latency_stats_v4(
        &self,
        service_name: Option<&str>,
        endpoint: Option<&str>,
        since: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
    ) -> Result<Vec<LatencyAnalysisV4>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct LatencyRow {
            svc: String,
            ep: String,
            avg_ms: f64,
            p50_ms: f64,
            p95_ms: f64,
            p99_ms: f64,
            count: i64,
        }
        let rows = sqlx::query_as::<_, LatencyRow>(
            r#"SELECT
             service_name as svc,
             endpoint as ep,
             COALESCE(AVG(latency_ms), 0.0) as avg_ms,
             COALESCE(PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY latency_ms), 0.0) as p50_ms,
             COALESCE(PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY latency_ms), 0.0) as p95_ms,
             COALESCE(PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY latency_ms), 0.0) as p99_ms,
             COUNT(*) as count
             FROM latency_analysis
             WHERE ($1::text IS NULL OR service_name = $1)
             AND ($2::text IS NULL OR endpoint = $2)
             AND ($3::timestamptz IS NULL OR recorded_at >= $3)
             AND ($4::timestamptz IS NULL OR recorded_at <= $4)
             GROUP BY service_name, endpoint
             ORDER BY count DESC"#,
        )
        .bind(service_name)
        .bind(endpoint)
        .bind(since)
        .bind(until)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| LatencyAnalysisV4 {
                service_name: r.svc,
                endpoint: r.ep,
                avg_latency_ms: r.avg_ms,
                p50_latency_ms: r.p50_ms,
                p95_latency_ms: r.p95_ms,
                p99_latency_ms: r.p99_ms,
                sample_count: r.count,
            })
            .collect())
    }

    // V8: Error correlation v6

    pub async fn correlate_error_v6(
        &self,
        trace_id: &str,
        error_type: &str,
        error_message: &str,
        service_name: &str,
        endpoint: &str,
        span_id: Option<&str>,
    ) -> Result<ErrorCorrelationV6, sqlx::Error> {
        let id = Uuid::new_v4();
        Ok(ErrorCorrelationV6 {
            id,
            trace_id: trace_id.to_string(),
            error_type: error_type.to_string(),
            error_message: error_message.to_string(),
            service_name: service_name.to_string(),
            endpoint: endpoint.to_string(),
            span_id: span_id.map(|s| s.to_string()),
            correlated_at: Utc::now(),
        })
    }

    // V8: Capacity planning v4

    pub async fn get_capacity_planning_v4(
        &self,
    ) -> Result<Vec<CapacityPlanningDataV4>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct CapacityRow {
            service_name: String,
            current_load: f64,
            bottleneck_endpoints: Vec<String>,
        }
        let rows = sqlx::query_as::<_, CapacityRow>(
            r#"SELECT
             service_name,
             SUM(call_count)::double precision as current_load,
             ARRAY_AGG(endpoint) FILTER (WHERE avg_duration_ms > 500) as bottleneck_endpoints
             FROM trace_service_map
             GROUP BY service_name
             ORDER BY current_load DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                let utilization = if r.current_load > 0.0 { (r.current_load / (r.current_load * 2.0)).min(1.0) } else { 0.0 };
                let recommended_replicas = if r.current_load > 0.0 { ((r.current_load * 2.0) / 1000.0).ceil() as i32 } else { 1 };
                CapacityPlanningDataV4 {
                    service_name: r.service_name,
                    current_load: r.current_load,
                    projected_load: r.current_load * 1.5,
                    recommended_capacity: r.current_load * 2.0,
                    bottleneck_endpoints: r.bottleneck_endpoints,
                    growth_rate: 0.5,
                    time_to_capacity_hours: if r.current_load > 0.0 { (r.current_load / (r.current_load * 0.5)) * 24.0 } else { f64::INFINITY },
                    utilization_score: utilization,
                    recommended_replicas,
                }
            })
            .collect())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct SamplingRuleV3Row {
    id: Uuid,
    service_name: String,
    endpoint: String,
    sample_rate: f64,
    max_traces_per_second: i32,
    priority: i32,
    enabled: bool,
    created_at: DateTime<Utc>,
}

impl From<SamplingRuleV3Row> for SamplingRuleV3 {
    fn from(row: SamplingRuleV3Row) -> Self {
        SamplingRuleV3 {
            id: row.id,
            service_name: row.service_name,
            endpoint: row.endpoint,
            sample_rate: row.sample_rate,
            max_traces_per_second: row.max_traces_per_second,
            priority: row.priority,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ServiceMapEntryRow {
    id: Uuid,
    service_name: String,
    endpoint: String,
    call_count: i64,
    avg_duration_ms: f64,
    error_rate: f64,
    last_updated_at: DateTime<Utc>,
}

impl From<ServiceMapEntryRow> for ServiceMapEntry {
    fn from(row: ServiceMapEntryRow) -> Self {
        ServiceMapEntry {
            id: row.id,
            service_name: row.service_name,
            endpoint: row.endpoint,
            call_count: row.call_count,
            avg_duration_ms: row.avg_duration_ms,
            error_rate: row.error_rate,
            last_updated_at: row.last_updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct SamplingRuleV4Row {
    id: Uuid,
    service_name: String,
    endpoint: String,
    sample_rate: f64,
    max_traces_per_second: i32,
    priority: i32,
    enabled: bool,
    created_at: DateTime<Utc>,
}

impl From<SamplingRuleV4Row> for SamplingRuleV4 {
    fn from(row: SamplingRuleV4Row) -> Self {
        SamplingRuleV4 {
            id: row.id,
            service_name: row.service_name,
            endpoint: row.endpoint,
            sample_rate: row.sample_rate,
            max_traces_per_second: row.max_traces_per_second,
            priority: row.priority,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct TraceServiceDependencyRow {
    id: Uuid,
    service_name: String,
    depends_on_service: String,
    call_count: i64,
    avg_duration_ms: f64,
    error_rate: f64,
    last_updated_at: DateTime<Utc>,
}

impl From<TraceServiceDependencyRow> for TraceServiceDependency {
    fn from(row: TraceServiceDependencyRow) -> Self {
        TraceServiceDependency {
            id: row.id,
            service_name: row.service_name,
            depends_on_service: row.depends_on_service,
            call_count: row.call_count,
            avg_duration_ms: row.avg_duration_ms,
            error_rate: row.error_rate,
            last_updated_at: row.last_updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct SamplingRuleV5Row {
    id: Uuid,
    service_name: String,
    endpoint: String,
    sample_rate: f64,
    max_traces_per_second: i32,
    priority: i32,
    enabled: bool,
    created_at: DateTime<Utc>,
}

impl From<SamplingRuleV5Row> for SamplingRuleV5 {
    fn from(row: SamplingRuleV5Row) -> Self {
        SamplingRuleV5 {
            id: row.id,
            service_name: row.service_name,
            endpoint: row.endpoint,
            sample_rate: row.sample_rate,
            max_traces_per_second: row.max_traces_per_second,
            priority: row.priority,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct TraceServiceDependencyV2Row {
    id: Uuid,
    service_name: String,
    depends_on_service: String,
    call_count: i64,
    avg_duration_ms: f64,
    error_rate: f64,
    last_updated_at: DateTime<Utc>,
}

impl From<TraceServiceDependencyV2Row> for TraceServiceDependencyV2 {
    fn from(row: TraceServiceDependencyV2Row) -> Self {
        TraceServiceDependencyV2 {
            id: row.id,
            service_name: row.service_name,
            depends_on_service: row.depends_on_service,
            call_count: row.call_count,
            avg_duration_ms: row.avg_duration_ms,
            error_rate: row.error_rate,
            last_updated_at: row.last_updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct SamplingRuleV6Row {
    id: Uuid,
    service_name: String,
    endpoint: String,
    sample_rate: f64,
    max_traces_per_second: i32,
    priority: i32,
    enabled: bool,
    created_at: DateTime<Utc>,
}

impl From<SamplingRuleV6Row> for SamplingRuleV6 {
    fn from(row: SamplingRuleV6Row) -> Self {
        SamplingRuleV6 {
            id: row.id,
            service_name: row.service_name,
            endpoint: row.endpoint,
            sample_rate: row.sample_rate,
            max_traces_per_second: row.max_traces_per_second,
            priority: row.priority,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct TraceServiceDependencyV3Row {
    id: Uuid,
    service_name: String,
    depends_on_service: String,
    call_count: i64,
    avg_duration_ms: f64,
    error_rate: f64,
    last_updated_at: DateTime<Utc>,
}

impl From<TraceServiceDependencyV3Row> for TraceServiceDependencyV3 {
    fn from(row: TraceServiceDependencyV3Row) -> Self {
        TraceServiceDependencyV3 {
            id: row.id,
            service_name: row.service_name,
            depends_on_service: row.depends_on_service,
            call_count: row.call_count,
            avg_duration_ms: row.avg_duration_ms,
            error_rate: row.error_rate,
            last_updated_at: row.last_updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct SamplingRuleV7Row {
    id: Uuid,
    service_name: String,
    endpoint: String,
    sample_rate: f64,
    max_traces_per_second: i32,
    priority: i32,
    enabled: bool,
    created_at: DateTime<Utc>,
}

impl From<SamplingRuleV7Row> for SamplingRuleV7 {
    fn from(row: SamplingRuleV7Row) -> Self {
        SamplingRuleV7 {
            id: row.id,
            service_name: row.service_name,
            endpoint: row.endpoint,
            sample_rate: row.sample_rate,
            max_traces_per_second: row.max_traces_per_second,
            priority: row.priority,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct TraceServiceDependencyV4Row {
    id: Uuid,
    service_name: String,
    depends_on_service: String,
    call_count: i64,
    avg_duration_ms: f64,
    error_rate: f64,
    last_updated_at: DateTime<Utc>,
}

impl From<TraceServiceDependencyV4Row> for TraceServiceDependencyV4 {
    fn from(row: TraceServiceDependencyV4Row) -> Self {
        TraceServiceDependencyV4 {
            id: row.id,
            service_name: row.service_name,
            depends_on_service: row.depends_on_service,
            call_count: row.call_count,
            avg_duration_ms: row.avg_duration_ms,
            error_rate: row.error_rate,
            last_updated_at: row.last_updated_at,
        }
    }
}
