//! Pipeline Runners v2: Advanced runner management with metrics collection,
//! tag-based matching, and lifecycle management.

#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRunnerV2 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub status: String,
    pub last_heartbeat: DateTime<Utc>,
    pub current_job: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerMetrics {
    pub id: Uuid,
    pub runner_id: Uuid,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRunnerV2Request {
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRunnerV2Request {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMetricsRequest {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
}

#[derive(Debug, sqlx::FromRow)]
struct RunnerRow {
    id: Uuid,
    name: String,
    description: String,
    tags: Vec<String>,
    status: String,
    last_heartbeat: DateTime<Utc>,
    current_job: Option<Uuid>,
    created_at: DateTime<Utc>,
}

impl From<RunnerRow> for PipelineRunnerV2 {
    fn from(row: RunnerRow) -> Self {
        PipelineRunnerV2 {
            id: row.id,
            name: row.name,
            description: row.description,
            tags: row.tags,
            status: row.status,
            last_heartbeat: row.last_heartbeat,
            current_job: row.current_job,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct MetricsRow {
    id: Uuid,
    runner_id: Uuid,
    cpu_usage: f64,
    memory_usage: f64,
    disk_usage: f64,
    recorded_at: DateTime<Utc>,
}

impl From<MetricsRow> for RunnerMetrics {
    fn from(row: MetricsRow) -> Self {
        RunnerMetrics {
            id: row.id,
            runner_id: row.runner_id,
            cpu_usage: row.cpu_usage,
            memory_usage: row.memory_usage,
            disk_usage: row.disk_usage,
            recorded_at: row.recorded_at,
        }
    }
}

pub struct PipelineRunnersV2Service {
    pool: PgPool,
}

impl PipelineRunnersV2Service {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_runners(
        &self,
        status: Option<&str>,
        tags: Option<&[String]>,
    ) -> Result<Vec<PipelineRunnerV2>, sqlx::Error> {
        let rows = if let Some(status) = status {
            sqlx::query_as::<_, RunnerRow>(
                "SELECT id, name, description, tags, status, last_heartbeat, current_job, created_at
                 FROM pipeline_runners_v2
                 WHERE status = $1
                 ORDER BY name",
            )
            .bind(status)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, RunnerRow>(
                "SELECT id, name, description, tags, status, last_heartbeat, current_job, created_at
                 FROM pipeline_runners_v2
                 ORDER BY name",
            )
            .fetch_all(&self.pool)
            .await?
        };

        let mut runners: Vec<PipelineRunnerV2> = rows.into_iter().map(|r| r.into()).collect();

        // Filter by tags if provided
        if let Some(filter_tags) = tags {
            if !filter_tags.is_empty() {
                runners.retain(|r| filter_tags.iter().all(|t| r.tags.contains(t)));
            }
        }

        Ok(runners)
    }

    pub async fn get_runner(&self, id: Uuid) -> Result<Option<PipelineRunnerV2>, sqlx::Error> {
        let row = sqlx::query_as::<_, RunnerRow>(
            "SELECT id, name, description, tags, status, last_heartbeat, current_job, created_at
             FROM pipeline_runners_v2
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn register_runner(
        &self,
        request: RegisterRunnerV2Request,
    ) -> Result<PipelineRunnerV2, sqlx::Error> {
        let tags = request.tags.unwrap_or_default();
        let description = request.description.unwrap_or_default();
        let now = Utc::now();

        let row = sqlx::query_as::<_, RunnerRow>(
            "INSERT INTO pipeline_runners_v2 (name, description, tags, status, last_heartbeat, created_at)
             VALUES ($1, $2, $3, 'online', $4, $4)
             RETURNING id, name, description, tags, status, last_heartbeat, current_job, created_at",
        )
        .bind(&request.name)
        .bind(&description)
        .bind(&tags)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn update_runner(
        &self,
        id: Uuid,
        request: UpdateRunnerV2Request,
    ) -> Result<PipelineRunnerV2, sqlx::Error> {
        let row = sqlx::query_as::<_, RunnerRow>(
            "UPDATE pipeline_runners_v2
             SET name = COALESCE($2, name),
                 description = COALESCE($3, description),
                 tags = COALESCE($4, tags),
                 status = COALESCE($5, status)
             WHERE id = $1
             RETURNING id, name, description, tags, status, last_heartbeat, current_job, created_at",
        )
        .bind(id)
        .bind(request.name)
        .bind(request.description)
        .bind(request.tags)
        .bind(request.status)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_runner(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM pipeline_runners_v2 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn heartbeat(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE pipeline_runners_v2 SET last_heartbeat = NOW(), status = 'online' WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn assign_job(
        &self,
        runner_id: Uuid,
        job_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE pipeline_runners_v2 SET current_job = $2 WHERE id = $1",
        )
        .bind(runner_id)
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn clear_job(&self, runner_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE pipeline_runners_v2 SET current_job = NULL WHERE id = $1",
        )
        .bind(runner_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn record_metrics(
        &self,
        runner_id: Uuid,
        request: RecordMetricsRequest,
    ) -> Result<RunnerMetrics, sqlx::Error> {
        let row = sqlx::query_as::<_, MetricsRow>(
            "INSERT INTO runner_metrics (runner_id, cpu_usage, memory_usage, disk_usage, recorded_at)
             VALUES ($1, $2, $3, $4, NOW())
             RETURNING id, runner_id, cpu_usage, memory_usage, disk_usage, recorded_at",
        )
        .bind(runner_id)
        .bind(request.cpu_usage)
        .bind(request.memory_usage)
        .bind(request.disk_usage)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_runner_metrics(
        &self,
        runner_id: Uuid,
        limit: i64,
    ) -> Result<Vec<RunnerMetrics>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MetricsRow>(
            "SELECT id, runner_id, cpu_usage, memory_usage, disk_usage, recorded_at
             FROM runner_metrics
             WHERE runner_id = $1
             ORDER BY recorded_at DESC
             LIMIT $2",
        )
        .bind(runner_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn find_available_runners(
        &self,
        required_tags: &[String],
    ) -> Result<Vec<PipelineRunnerV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RunnerRow>(
            "SELECT id, name, description, tags, status, last_heartbeat, current_job, created_at
             FROM pipeline_runners_v2
             WHERE status = 'online' AND current_job IS NULL
             ORDER BY last_heartbeat DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        let runners: Vec<PipelineRunnerV2> = rows.into_iter().map(|r| r.into()).collect();

        // Filter by required tags
        if required_tags.is_empty() {
            Ok(runners)
        } else {
            Ok(runners
                .into_iter()
                .filter(|r| required_tags.iter().all(|t| r.tags.contains(t)))
                .collect())
        }
    }

    pub async fn cleanup_stale_runners(
        &self,
        stale_threshold_minutes: i64,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE pipeline_runners_v2
             SET status = 'offline'
             WHERE status = 'online'
               AND last_heartbeat < NOW() - ($1 || ' minutes')::interval",
        )
        .bind(stale_threshold_minutes)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_runner_v2_serialize() {
        let runner = PipelineRunnerV2 {
            id: Uuid::new_v4(),
            name: "linux-runner".to_string(),
            description: "Linux build runner".to_string(),
            tags: vec!["linux".to_string(), "amd64".to_string()],
            status: "online".to_string(),
            last_heartbeat: Utc::now(),
            current_job: None,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&runner).unwrap();
        assert!(json.contains("linux-runner"));
        assert!(json.contains("linux"));
    }

    #[test]
    fn test_runner_metrics_serialize() {
        let metrics = RunnerMetrics {
            id: Uuid::new_v4(),
            runner_id: Uuid::new_v4(),
            cpu_usage: 45.2,
            memory_usage: 68.5,
            disk_usage: 32.1,
            recorded_at: Utc::now(),
        };
        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("45.2"));
        assert!(json.contains("68.5"));
    }

    #[test]
    fn test_register_runner_request_deserialize() {
        let json = r#"{"name": "macos-runner", "description": "macOS build runner", "tags": ["macos", "arm64"]}"#;
        let req: RegisterRunnerV2Request = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "macos-runner");
        assert_eq!(req.tags.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_record_metrics_request_deserialize() {
        let json = r#"{"cpu_usage": 75.5, "memory_usage": 82.3, "disk_usage": 45.0}"#;
        let req: RecordMetricsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.cpu_usage, 75.5);
        assert_eq!(req.memory_usage, 82.3);
        assert_eq!(req.disk_usage, 45.0);
    }
}
