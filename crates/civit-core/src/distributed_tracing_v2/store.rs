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
}
