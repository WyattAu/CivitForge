//! Pipeline Action Installations types and logic.

#![forbid(unsafe_code)]

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionInstallationResponse {
    pub id: String,
    pub action_id: String,
    pub repo_id: String,
    pub installed_by: String,
    pub version: String,
    pub config: serde_json::Value,
    pub installed_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ActionInstallationRow {
    pub id: Uuid,
    pub action_id: Uuid,
    pub repo_id: Uuid,
    pub installed_by: Uuid,
    pub version: String,
    pub config: serde_json::Value,
    pub installed_at: chrono::DateTime<chrono::Utc>,
}

impl From<ActionInstallationRow> for ActionInstallationResponse {
    fn from(r: ActionInstallationRow) -> Self {
        Self {
            id: r.id.to_string(),
            action_id: r.action_id.to_string(),
            repo_id: r.repo_id.to_string(),
            installed_by: r.installed_by.to_string(),
            version: r.version,
            config: r.config,
            installed_at: r.installed_at.to_rfc3339(),
        }
    }
}

/// Install a pipeline action for a repository.
pub async fn install_action(
    pool: &sqlx::PgPool,
    action_id: Uuid,
    repo_id: Uuid,
    installed_by: Uuid,
    version: &str,
    config: &serde_json::Value,
) -> std::result::Result<ActionInstallationResponse, sqlx::Error> {
    sqlx::query_as::<_, ActionInstallationRow>(
        "INSERT INTO pipeline_action_installations (action_id, repo_id, installed_by, version, config) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING *",
    )
    .bind(action_id)
    .bind(repo_id)
    .bind(installed_by)
    .bind(version)
    .bind(config)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Get installations for a repository.
pub async fn list_installations(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
) -> std::result::Result<Vec<ActionInstallationResponse>, sqlx::Error> {
    sqlx::query_as::<_, ActionInstallationRow>(
        "SELECT * FROM pipeline_action_installations WHERE repo_id = $1 ORDER BY installed_at DESC",
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Uninstall an action from a repository.
pub async fn uninstall_action(
    pool: &sqlx::PgPool,
    installation_id: Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM pipeline_action_installations WHERE id = $1")
        .bind(installation_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_installation_response_serialize() {
        let resp = ActionInstallationResponse {
            id: "00000000-0000-0000-0000-000000000001".to_string(),
            action_id: "00000000-0000-0000-0000-000000000002".to_string(),
            repo_id: "00000000-0000-0000-0000-000000000003".to_string(),
            installed_by: "00000000-0000-0000-0000-000000000004".to_string(),
            version: "1.0.0".to_string(),
            config: serde_json::json!({"timeout": 300}),
            installed_at: "2025-01-01T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("1.0.0"));
    }
}
