//! Environment Health Checks types and logic.

#![forbid(unsafe_code)]

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub id: String,
    pub environment_id: String,
    pub check_type: String,
    pub endpoint: Option<String>,
    pub interval_seconds: i32,
    pub timeout_seconds: i32,
    pub enabled: bool,
    pub last_checked_at: Option<String>,
    pub last_status: String,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct HealthCheckRow {
    pub id: Uuid,
    pub environment_id: Uuid,
    pub check_type: String,
    pub endpoint: Option<String>,
    pub interval_seconds: i32,
    pub timeout_seconds: i32,
    pub enabled: bool,
    pub last_checked_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<HealthCheckRow> for HealthCheckResponse {
    fn from(r: HealthCheckRow) -> Self {
        Self {
            id: r.id.to_string(),
            environment_id: r.environment_id.to_string(),
            check_type: r.check_type,
            endpoint: r.endpoint,
            interval_seconds: r.interval_seconds,
            timeout_seconds: r.timeout_seconds,
            enabled: r.enabled,
            last_checked_at: r.last_checked_at.map(|t| t.to_rfc3339()),
            last_status: r.last_status,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

/// Create a health check for an environment.
pub async fn create_health_check(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
    check_type: &str,
    endpoint: Option<&str>,
    interval_seconds: i32,
    timeout_seconds: i32,
) -> std::result::Result<HealthCheckResponse, sqlx::Error> {
    sqlx::query_as::<_, HealthCheckRow>(
        "INSERT INTO environment_health_checks (environment_id, check_type, endpoint, interval_seconds, timeout_seconds) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING *",
    )
    .bind(environment_id)
    .bind(check_type)
    .bind(endpoint)
    .bind(interval_seconds)
    .bind(timeout_seconds)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// List health checks for an environment.
pub async fn list_health_checks(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
) -> std::result::Result<Vec<HealthCheckResponse>, sqlx::Error> {
    sqlx::query_as::<_, HealthCheckRow>(
        "SELECT * FROM environment_health_checks WHERE environment_id = $1 ORDER BY created_at DESC",
    )
    .bind(environment_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Update health check status.
pub async fn update_health_check_status(
    pool: &sqlx::PgPool,
    check_id: Uuid,
    status: &str,
) -> std::result::Result<HealthCheckResponse, sqlx::Error> {
    sqlx::query_as::<_, HealthCheckRow>(
        "UPDATE environment_health_checks \
         SET last_status = $2, last_checked_at = NOW() \
         WHERE id = $1 \
         RETURNING *",
    )
    .bind(check_id)
    .bind(status)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Delete a health check.
pub async fn delete_health_check(
    pool: &sqlx::PgPool,
    check_id: Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM environment_health_checks WHERE id = $1")
        .bind(check_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_response_serialize() {
        let resp = HealthCheckResponse {
            id: "00000000-0000-0000-0000-000000000001".to_string(),
            environment_id: "00000000-0000-0000-0000-000000000002".to_string(),
            check_type: "http".to_string(),
            endpoint: Some("https://staging.example.com/health".to_string()),
            interval_seconds: 60,
            timeout_seconds: 10,
            enabled: true,
            last_checked_at: Some("2025-01-01T00:00:00+00:00".to_string()),
            last_status: "healthy".to_string(),
            created_at: "2025-01-01T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("healthy"));
    }
}
