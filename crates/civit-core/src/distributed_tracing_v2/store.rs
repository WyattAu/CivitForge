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
}
