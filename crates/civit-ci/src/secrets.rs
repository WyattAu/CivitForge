//! Pipeline Secrets types.
//!
//! Manages encrypted repository secrets for CI/CD pipelines.

#![forbid(unsafe_code)]

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretNameResponse {
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretDetailResponse {
    pub name: String,
    pub value: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateSecretRequest {
    pub name: String,
    pub value: String,
}

pub async fn list_secrets_db(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
) -> std::result::Result<Vec<SecretNameResponse>, sqlx::Error> {
    let rows: Vec<(String, chrono::DateTime<Utc>, chrono::DateTime<Utc>)> = sqlx::query_as(
        "SELECT name, created_at, updated_at FROM repo_secrets WHERE repo_id = $1 ORDER BY name",
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(name, created_at, updated_at)| SecretNameResponse {
            name,
            created_at: created_at.to_rfc3339(),
            updated_at: updated_at.to_rfc3339(),
        })
        .collect())
}

type SecretValueRow = (
    Vec<u8>,
    Vec<u8>,
    chrono::DateTime<Utc>,
    chrono::DateTime<Utc>,
);

pub async fn get_secret_value_db(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    secret_name: &str,
) -> std::result::Result<Option<(Vec<u8>, Vec<u8>, chrono::DateTime<Utc>, chrono::DateTime<Utc>)>, sqlx::Error> {
    sqlx::query_as::<_, SecretValueRow>(
        "SELECT value_enc, nonce, created_at, updated_at FROM repo_secrets WHERE repo_id = $1 AND name = $2",
    )
    .bind(repo_id)
    .bind(secret_name)
    .fetch_optional(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_name_response_serialize() {
        let resp = SecretNameResponse {
            name: "MY_SECRET".to_string(),
            created_at: "2025-01-01T00:00:00+00:00".to_string(),
            updated_at: "2025-01-01T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("MY_SECRET"));
    }

    #[test]
    fn test_create_secret_request_deserialize() {
        let json = r#"{"name": "MY_TOKEN", "value": "abc123"}"#;
        let req: CreateSecretRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "MY_TOKEN");
        assert_eq!(req.value, "abc123");
    }
}
