//! Environment Deployment v16: Advanced deployment tracking with metadata v16,
//! rollback tracking v16, deployment comparison v16, and deployment analytics v16.

#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentDeploymentHistoryV16 {
    pub id: Uuid,
    pub environment_id: Uuid,
    pub version: String,
    pub sha: String,
    pub status: String,
    pub deployed_by: Uuid,
    pub rollback_of: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentComparisonV16 {
    pub id: Uuid,
    pub from_deployment_id: Uuid,
    pub to_deployment_id: Uuid,
    pub diff_summary: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentAnalyticsV16 {
    pub id: Uuid,
    pub environment_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub total_deployments: i32,
    pub successful_deployments: i32,
    pub failed_deployments: i32,
    pub avg_deploy_time_ms: i64,
    pub rollback_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDeploymentRequestV16 {
    pub environment_id: Uuid,
    pub version: String,
    pub sha: String,
    pub status: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackDeploymentRequestV16 {
    pub deployment_id: Uuid,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateComparisonRequestV16 {
    pub from_deployment_id: Uuid,
    pub to_deployment_id: Uuid,
    pub diff_summary: serde_json::Value,
}

#[derive(Debug, sqlx::FromRow)]
struct DeploymentRowV16 {
    id: Uuid,
    environment_id: Uuid,
    version: String,
    sha: String,
    status: String,
    deployed_by: Uuid,
    rollback_of: Option<Uuid>,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl From<DeploymentRowV16> for EnvironmentDeploymentHistoryV16 {
    fn from(row: DeploymentRowV16) -> Self {
        EnvironmentDeploymentHistoryV16 {
            id: row.id,
            environment_id: row.environment_id,
            version: row.version,
            sha: row.sha,
            status: row.status,
            deployed_by: row.deployed_by,
            rollback_of: row.rollback_of,
            metadata: row.metadata,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ComparisonRowV16 {
    id: Uuid,
    from_deployment_id: Uuid,
    to_deployment_id: Uuid,
    diff_summary: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl From<ComparisonRowV16> for DeploymentComparisonV16 {
    fn from(row: ComparisonRowV16) -> Self {
        DeploymentComparisonV16 {
            id: row.id,
            from_deployment_id: row.from_deployment_id,
            to_deployment_id: row.to_deployment_id,
            diff_summary: row.diff_summary,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct AnalyticsRowV16 {
    id: Uuid,
    environment_id: Uuid,
    period_start: DateTime<Utc>,
    total_deployments: i32,
    successful_deployments: i32,
    failed_deployments: i32,
    avg_deploy_time_ms: i64,
    rollback_count: i32,
    created_at: DateTime<Utc>,
}

impl From<AnalyticsRowV16> for DeploymentAnalyticsV16 {
    fn from(row: AnalyticsRowV16) -> Self {
        DeploymentAnalyticsV16 {
            id: row.id,
            environment_id: row.environment_id,
            period_start: row.period_start,
            total_deployments: row.total_deployments,
            successful_deployments: row.successful_deployments,
            failed_deployments: row.failed_deployments,
            avg_deploy_time_ms: row.avg_deploy_time_ms,
            rollback_count: row.rollback_count,
            created_at: row.created_at,
        }
    }
}

pub struct EnvironmentDeploymentServiceV16 {
    pool: PgPool,
}

impl EnvironmentDeploymentServiceV16 {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_deployment(
        &self,
        deployed_by: Uuid,
        request: CreateDeploymentRequestV16,
    ) -> Result<EnvironmentDeploymentHistoryV16, sqlx::Error> {
        let status = request.status.unwrap_or_else(|| "deployed".to_string());
        let metadata = request.metadata.unwrap_or(serde_json::json!({}));

        let row = sqlx::query_as::<_, DeploymentRowV16>(
            "INSERT INTO environment_deployment_history_v16
             (environment_id, version, sha, status, deployed_by, metadata, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, NOW())
             RETURNING id, environment_id, version, sha, status, deployed_by, rollback_of, metadata, created_at",
        )
        .bind(request.environment_id)
        .bind(&request.version)
        .bind(&request.sha)
        .bind(&status)
        .bind(deployed_by)
        .bind(&metadata)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_deployment(
        &self,
        deployment_id: Uuid,
    ) -> Result<Option<EnvironmentDeploymentHistoryV16>, sqlx::Error> {
        let row = sqlx::query_as::<_, DeploymentRowV16>(
            "SELECT id, environment_id, version, sha, status, deployed_by, rollback_of, metadata, created_at
             FROM environment_deployment_history_v16
             WHERE id = $1",
        )
        .bind(deployment_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_deployments_for_environment(
        &self,
        environment_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<EnvironmentDeploymentHistoryV16>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DeploymentRowV16>(
            "SELECT id, environment_id, version, sha, status, deployed_by, rollback_of, metadata, created_at
             FROM environment_deployment_history_v16
             WHERE environment_id = $1
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3",
        )
        .bind(environment_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn rollback_deployment(
        &self,
        deployed_by: Uuid,
        request: RollbackDeploymentRequestV16,
    ) -> Result<EnvironmentDeploymentHistoryV16, sqlx::Error> {
        let original = self.get_deployment(request.deployment_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let metadata = if let Some(reason) = &request.reason {
            serde_json::json!({
                "rollback_reason": reason,
                "original_deployment_id": request.deployment_id
            })
        } else {
            serde_json::json!({
                "original_deployment_id": request.deployment_id
            })
        };

        let row = sqlx::query_as::<_, DeploymentRowV16>(
            "INSERT INTO environment_deployment_history_v16
             (environment_id, version, sha, status, deployed_by, rollback_of, metadata, created_at)
             VALUES ($1, $2, $3, 'rolled_back', $4, $5, $6, NOW())
             RETURNING id, environment_id, version, sha, status, deployed_by, rollback_of, metadata, created_at",
        )
        .bind(original.environment_id)
        .bind(&original.version)
        .bind(&original.sha)
        .bind(deployed_by)
        .bind(request.deployment_id)
        .bind(&metadata)
        .fetch_one(&self.pool)
        .await?;

        sqlx::query(
            "UPDATE environment_deployment_history_v16 SET status = 'rolled_back' WHERE id = $1",
        )
        .bind(request.deployment_id)
        .execute(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn create_comparison(
        &self,
        request: CreateComparisonRequestV16,
    ) -> Result<DeploymentComparisonV16, sqlx::Error> {
        let row = sqlx::query_as::<_, ComparisonRowV16>(
            "INSERT INTO deployment_comparison_v16
             (from_deployment_id, to_deployment_id, diff_summary, created_at)
             VALUES ($1, $2, $3, NOW())
             ON CONFLICT (from_deployment_id, to_deployment_id) DO UPDATE
             SET diff_summary = EXCLUDED.diff_summary
             RETURNING id, from_deployment_id, to_deployment_id, diff_summary, created_at",
        )
        .bind(request.from_deployment_id)
        .bind(request.to_deployment_id)
        .bind(&request.diff_summary)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_comparison(
        &self,
        from_deployment_id: Uuid,
        to_deployment_id: Uuid,
    ) -> Result<Option<DeploymentComparisonV16>, sqlx::Error> {
        let row = sqlx::query_as::<_, ComparisonRowV16>(
            "SELECT id, from_deployment_id, to_deployment_id, diff_summary, created_at
             FROM deployment_comparison_v16
             WHERE from_deployment_id = $1 AND to_deployment_id = $2",
        )
        .bind(from_deployment_id)
        .bind(to_deployment_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn get_deployment_analytics(
        &self,
        environment_id: Uuid,
    ) -> Result<Vec<DeploymentAnalyticsV16>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AnalyticsRowV16>(
            "SELECT id, environment_id, period_start, total_deployments, successful_deployments,
                    failed_deployments, avg_deploy_time_ms, rollback_count, created_at
             FROM deployment_analytics_v16
             WHERE environment_id = $1
             ORDER BY period_start DESC",
        )
        .bind(environment_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn generate_deployment_analytics(
        &self,
        environment_id: Uuid,
        period_start: DateTime<Utc>,
    ) -> Result<DeploymentAnalyticsV16, sqlx::Error> {
        let row = sqlx::query_as::<_, AnalyticsRowV16>(
            "INSERT INTO deployment_analytics_v16
             (environment_id, period_start, total_deployments, successful_deployments,
              failed_deployments, avg_deploy_time_ms, rollback_count, created_at)
             SELECT
                 $1 as environment_id,
                 $2 as period_start,
                 COUNT(*)::INTEGER as total_deployments,
                 COUNT(*) FILTER (WHERE status = 'deployed')::INTEGER as successful_deployments,
                 COUNT(*) FILTER (WHERE status = 'failed')::INTEGER as failed_deployments,
                 0::BIGINT as avg_deploy_time_ms,
                 COUNT(*) FILTER (WHERE status = 'rolled_back')::INTEGER as rollback_count,
                 NOW() as created_at
             FROM environment_deployment_history_v16
             WHERE environment_id = $1 AND created_at >= $2
             RETURNING id, environment_id, period_start, total_deployments, successful_deployments,
                       failed_deployments, avg_deploy_time_ms, rollback_count, created_at",
        )
        .bind(environment_id)
        .bind(period_start)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployment_v16_serialize() {
        let deployment = EnvironmentDeploymentHistoryV16 {
            id: Uuid::new_v4(),
            environment_id: Uuid::new_v4(),
            version: "v1.2.3".to_string(),
            sha: "abc123".to_string(),
            status: "deployed".to_string(),
            deployed_by: Uuid::new_v4(),
            rollback_of: None,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&deployment).unwrap();
        assert!(json.contains("v1.2.3"));
        assert!(json.contains("abc123"));
    }

    #[test]
    fn test_create_deployment_request_v16_deserialize() {
        let json = r#"{"environment_id": "550e8400-e29b-41d4-a716-446655440000", "version": "v1.0.0", "sha": "def456"}"#;
        let req: CreateDeploymentRequestV16 = serde_json::from_str(json).unwrap();
        assert_eq!(req.version, "v1.0.0");
        assert_eq!(req.sha, "def456");
    }

    #[test]
    fn test_rollback_request_v16_deserialize() {
        let json = r#"{"deployment_id": "550e8400-e29b-41d4-a716-446655440000", "reason": "Bug fix"}"#;
        let req: RollbackDeploymentRequestV16 = serde_json::from_str(json).unwrap();
        assert_eq!(req.reason, Some("Bug fix".to_string()));
    }

    #[test]
    fn test_comparison_v16_serialize() {
        let c = DeploymentComparisonV16 {
            id: Uuid::new_v4(),
            from_deployment_id: Uuid::new_v4(),
            to_deployment_id: Uuid::new_v4(),
            diff_summary: serde_json::json!({"files_changed": 3}),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&c).unwrap();
        assert!(json.contains("files_changed"));
    }
}
