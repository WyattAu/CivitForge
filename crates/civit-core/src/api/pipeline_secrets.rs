//! Pipeline Secrets API endpoints.
//!
//! Manages encrypted repository secrets for CI/CD pipelines.
//! Secrets are stored with AES-256-GCM encryption and only decrypted
//! when resolved by an authenticated runner.

#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_admin};
use crate::auth::permission_engine::{decrypt_value, encrypt_value};
use crate::error::CoreError;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Response / Request types
// ---------------------------------------------------------------------------

/// Secret name entry (no values exposed).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretNameResponse {
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Full secret detail (admin-only, value decrypted).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretDetailResponse {
    pub name: String,
    pub value: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for creating/updating a secret.
#[derive(Debug, Deserialize)]
pub struct CreateSecretRequest {
    pub name: String,
    pub value: String,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn pipeline_secret_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{repo}/secrets",
            get(list_secrets).post(upsert_secret),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/secrets/{secret_name}",
            get(get_secret).delete(delete_secret),
        )
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// List secret names for a repository (no values exposed).
pub async fn list_secrets(
    State(state): State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &repo_name).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    match list_secrets_db(pool, repo_id).await {
        Ok(secrets) => (axum::http::StatusCode::OK, Json(secrets)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Create or update a secret (AES-256-GCM encrypted).
pub async fn upsert_secret(
    State(state): State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<CreateSecretRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &repo_name).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    // Encrypt the secret value
    let (value_enc, nonce) = match encrypt_value(&req.value) {
        Ok(v) => v,
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Internal(format!("encryption failed: {e}")).error_response()),
            )
                .into_response();
        }
    };

    let user_id = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Internal("invalid user ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let now = Utc::now();
    match sqlx::query(
        "INSERT INTO repo_secrets (repo_id, name, value_enc, nonce, created_by, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (repo_id, name)
         DO UPDATE SET value_enc = $3, nonce = $4, updated_at = $7",
    )
    .bind(repo_id)
    .bind(&req.name)
    .bind(&value_enc)
    .bind(&nonce)
    .bind(user_id)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    {
        Ok(_) => (
            axum::http::StatusCode::CREATED,
            Json(SecretNameResponse {
                name: req.name,
                created_at: now.to_rfc3339(),
                updated_at: now.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Get secret value (admin only).
pub async fn get_secret(
    State(state): State<AppState>,
    Path((owner, repo_name, secret_name)): Path<(String, String, String)>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &repo_name).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    match get_secret_value_db(pool, repo_id, &secret_name).await {
        Ok(Some(secret)) => (axum::http::StatusCode::OK, Json(secret)).into_response(),
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("secret not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Delete a secret.
pub async fn delete_secret(
    State(state): State<AppState>,
    Path((owner, repo_name, secret_name)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &repo_name).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    match sqlx::query("DELETE FROM repo_secrets WHERE repo_id = $1 AND name = $2")
        .bind(repo_id)
        .bind(&secret_name)
        .execute(pool)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => {
            (axum::http::StatusCode::NO_CONTENT, "").into_response()
        }
        Ok(_) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("secret not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

async fn get_repo_id(
    pool: &sqlx::PgPool,
    owner: &str,
    repo_name: &str,
) -> std::result::Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_as::<_, (Uuid,)>(
        "SELECT r.id FROM repositories r JOIN users u ON r.owner_id = u.id WHERE u.username = $1 AND r.name = $2",
    )
    .bind(owner)
    .bind(repo_name)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}

async fn list_secrets_db(
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

async fn get_secret_value_db(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    secret_name: &str,
) -> std::result::Result<Option<SecretDetailResponse>, sqlx::Error> {
    let row: Option<SecretValueRow> = sqlx::query_as(
        "SELECT value_enc, nonce, created_at, updated_at FROM repo_secrets WHERE repo_id = $1 AND name = $2",
    )
    .bind(repo_id)
    .bind(secret_name)
    .fetch_optional(pool)
    .await?;

    match row {
        Some((ciphertext, nonce, created_at, updated_at)) => {
            let value = decrypt_value(&ciphertext, &nonce)
                .map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
            Ok(Some(SecretDetailResponse {
                name: secret_name.to_string(),
                value,
                created_at: created_at.to_rfc3339(),
                updated_at: updated_at.to_rfc3339(),
            }))
        }
        None => Ok(None),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

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
        assert!(json.contains("created_at"));
    }

    #[test]
    fn test_secret_detail_response_serialize() {
        let resp = SecretDetailResponse {
            name: "API_KEY".to_string(),
            value: "super-secret-value".to_string(),
            created_at: "2025-01-01T00:00:00+00:00".to_string(),
            updated_at: "2025-01-01T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("API_KEY"));
        assert!(json.contains("super-secret-value"));
    }

    #[test]
    fn test_create_secret_request_deserialize() {
        let json = r#"{"name": "MY_TOKEN", "value": "abc123"}"#;
        let req: CreateSecretRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "MY_TOKEN");
        assert_eq!(req.value, "abc123");
    }

    #[test]
    fn test_secret_detail_response_roundtrip() {
        let resp = SecretDetailResponse {
            name: "DB_PASSWORD".to_string(),
            value: "p@ssw0rd!".to_string(),
            created_at: "2025-01-01T00:00:00+00:00".to_string(),
            updated_at: "2025-01-01T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let decoded: SecretDetailResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, "DB_PASSWORD");
        assert_eq!(decoded.value, "p@ssw0rd!");
    }
}
