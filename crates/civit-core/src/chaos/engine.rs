use super::types::*;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct ChaosEngine {
    pool: PgPool,
}

impl ChaosEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_experiment(&self, req: CreateExperimentRequest) -> Result<ChaosExperiment, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let params = req.parameters.unwrap_or(serde_json::json!({}));
        let description = req.description.unwrap_or_default();

        sqlx::query(
            r#"INSERT INTO chaos_experiments (id, name, description, experiment_type, target, parameters, status, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        )
        .bind(id)
        .bind(&req.name)
        .bind(&description)
        .bind(req.experiment_type.to_string())
        .bind(&req.target)
        .bind(&params)
        .bind(ExperimentStatus::Pending.to_string())
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(ChaosExperiment {
            id,
            name: req.name,
            description,
            experiment_type: req.experiment_type,
            target: req.target,
            parameters: params,
            status: ExperimentStatus::Pending,
            started_at: None,
            completed_at: None,
            created_at: now,
        })
    }

    pub async fn get_experiment(&self, id: Uuid) -> Result<Option<ChaosExperiment>, sqlx::Error> {
        let row = sqlx::query_as::<_, ChaosExperimentRow>(
            r#"SELECT id, name, description, experiment_type, target, parameters, status, started_at, completed_at, created_at
               FROM chaos_experiments WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(ChaosExperiment::from))
    }

    pub async fn list_experiments(
        &self,
        status: Option<ExperimentStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ChaosExperiment>, sqlx::Error> {
        let rows = if let Some(status) = status {
            sqlx::query_as::<_, ChaosExperimentRow>(
                r#"SELECT id, name, description, experiment_type, target, parameters, status, started_at, completed_at, created_at
                   FROM chaos_experiments WHERE status = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"#,
            )
            .bind(status.to_string())
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ChaosExperimentRow>(
                r#"SELECT id, name, description, experiment_type, target, parameters, status, started_at, completed_at, created_at
                   FROM chaos_experiments ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows.into_iter().map(ChaosExperiment::from).collect())
    }

    pub async fn start_experiment(&self, id: Uuid) -> Result<ChaosExperiment, sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            r#"UPDATE chaos_experiments SET status = $1, started_at = $2 WHERE id = $3"#,
        )
        .bind(ExperimentStatus::Running.to_string())
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_experiment(id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn complete_experiment(
        &self,
        id: Uuid,
        success: bool,
    ) -> Result<ChaosExperiment, sqlx::Error> {
        let now = Utc::now();
        let status = if success {
            ExperimentStatus::Completed
        } else {
            ExperimentStatus::Failed
        };

        sqlx::query(
            r#"UPDATE chaos_experiments SET status = $1, completed_at = $2 WHERE id = $3"#,
        )
        .bind(status.to_string())
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_experiment(id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn cancel_experiment(&self, id: Uuid) -> Result<ChaosExperiment, sqlx::Error> {
        sqlx::query(
            r#"UPDATE chaos_experiments SET status = $1 WHERE id = $2"#,
        )
        .bind(ExperimentStatus::Cancelled.to_string())
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_experiment(id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn record_result(
        &self,
        experiment_id: Uuid,
        metric_name: String,
        metric_value: f64,
        baseline_value: Option<f64>,
        impact: ImpactLevel,
    ) -> Result<ChaosResult, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO chaos_results (id, experiment_id, metric_name, metric_value, baseline_value, impact, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        )
        .bind(id)
        .bind(experiment_id)
        .bind(&metric_name)
        .bind(metric_value)
        .bind(baseline_value)
        .bind(impact.to_string())
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(ChaosResult {
            id,
            experiment_id,
            metric_name,
            metric_value,
            baseline_value,
            impact,
            created_at: now,
        })
    }

    pub async fn get_results(
        &self,
        experiment_id: Uuid,
    ) -> Result<Vec<ChaosResult>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ChaosResultRow>(
            r#"SELECT id, experiment_id, metric_name, metric_value, baseline_value, impact, created_at
               FROM chaos_results WHERE experiment_id = $1 ORDER BY created_at"#,
        )
        .bind(experiment_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(ChaosResult::from).collect())
    }

    pub async fn delete_experiment(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM chaos_experiments WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn execute_latency_injection(
        &self,
        experiment_id: Uuid,
        latency_ms: u64,
        duration_seconds: u64,
    ) -> Result<ExperimentExecution, sqlx::Error> {
        let start = std::time::Instant::now();
        tokio::time::sleep(std::time::Duration::from_millis(latency_ms)).await;
        let elapsed = start.elapsed();

        let result = self
            .record_result(
                experiment_id,
                "latency_ms".to_string(),
                elapsed.as_millis() as f64,
                Some(0.0),
                if latency_ms > 1000 {
                    ImpactLevel::High
                } else if latency_ms > 100 {
                    ImpactLevel::Medium
                } else {
                    ImpactLevel::Low
                },
            )
            .await?;

        Ok(ExperimentExecution {
            experiment_id,
            duration_seconds,
            metrics_collected: vec![result],
            success: true,
            error_message: None,
        })
    }

    pub async fn execute_error_injection(
        &self,
        experiment_id: Uuid,
        error_rate: f64,
    ) -> Result<ExperimentExecution, sqlx::Error> {
        let impact = if error_rate > 0.5 {
            ImpactLevel::Critical
        } else if error_rate > 0.1 {
            ImpactLevel::High
        } else if error_rate > 0.01 {
            ImpactLevel::Medium
        } else {
            ImpactLevel::Low
        };

        let result = self
            .record_result(
                experiment_id,
                "error_rate".to_string(),
                error_rate,
                Some(0.0),
                impact,
            )
            .await?;

        Ok(ExperimentExecution {
            experiment_id,
            duration_seconds: 0,
            metrics_collected: vec![result],
            success: true,
            error_message: None,
        })
    }
}

#[derive(sqlx::FromRow)]
struct ChaosExperimentRow {
    id: Uuid,
    name: String,
    description: String,
    experiment_type: String,
    target: String,
    parameters: serde_json::Value,
    status: String,
    started_at: Option<chrono::DateTime<Utc>>,
    completed_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
}

impl From<ChaosExperimentRow> for ChaosExperiment {
    fn from(row: ChaosExperimentRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            experiment_type: row.experiment_type.parse().unwrap_or(ExperimentType::LatencyInjection),
            target: row.target,
            parameters: row.parameters,
            status: row.status.parse().unwrap_or(ExperimentStatus::Pending),
            started_at: row.started_at,
            completed_at: row.completed_at,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ChaosResultRow {
    id: Uuid,
    experiment_id: Uuid,
    metric_name: String,
    metric_value: f64,
    baseline_value: Option<f64>,
    impact: String,
    created_at: chrono::DateTime<Utc>,
}

impl From<ChaosResultRow> for ChaosResult {
    fn from(row: ChaosResultRow) -> Self {
        Self {
            id: row.id,
            experiment_id: row.experiment_id,
            metric_name: row.metric_name,
            metric_value: row.metric_value,
            baseline_value: row.baseline_value,
            impact: row.impact.parse().unwrap_or(ImpactLevel::None),
            created_at: row.created_at,
        }
    }
}
