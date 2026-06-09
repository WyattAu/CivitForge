//! Pipeline Caches API endpoints.
//!
//! Manages cached artifacts for CI/CD pipelines (dependency caches,
//! build outputs, etc.).

#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{delete, get};
use axum::{Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Response / Request types
// ---------------------------------------------------------------------------

/// Cache entry metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntryResponse {
    pub key: String,
    pub path: String,
    pub size_bytes: i64,
    pub created_at: String,
    pub expires_at: Option<String>,
}

/// Request body for creating a cache entry.
#[derive(Debug, Deserialize)]
pub struct CreateCacheRequest {
    pub key: String,
    pub path: String,
    #[serde(default)]
    pub size_bytes: i64,
    /// TTL in seconds from now. If None, cache never expires.
    pub ttl_secs: Option<i64>,
}

/// List query parameters.
#[derive(Debug, Deserialize)]
pub struct CacheListParams {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    /// Optional prefix filter on cache key.
    pub prefix: Option<String>,
}

fn default_limit() -> i64 {
    50
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn pipeline_cache_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{repo}/caches",
            get(list_caches).post(create_cache),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/caches/{cache_key}",
            delete(delete_cache),
        )
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// List cache entries for a repository.
pub async fn list_caches(
    State(state): State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    _auth: AuthUser,
    axum::extract::Query(params): axum::extract::Query<CacheListParams>,
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

    match list_caches_db(
        pool,
        repo_id,
        params.limit,
        params.offset,
        params.prefix.as_deref(),
    )
    .await
    {
        Ok(caches) => (axum::http::StatusCode::OK, Json(caches)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Create a cache entry.
pub async fn create_cache(
    State(state): State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    _auth: AuthUser,
    Json(req): Json<CreateCacheRequest>,
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

    let now = Utc::now();
    let expires_at = req.ttl_secs.map(|ttl| now + chrono::Duration::seconds(ttl));

    match sqlx::query(
        "INSERT INTO pipeline_caches (repo_id, key, path, size_bytes, created_at, expires_at)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (repo_id, key)
         DO UPDATE SET path = $3, size_bytes = $4, created_at = $5, expires_at = $6",
    )
    .bind(repo_id)
    .bind(&req.key)
    .bind(&req.path)
    .bind(req.size_bytes)
    .bind(now)
    .bind(expires_at)
    .execute(pool)
    .await
    {
        Ok(_) => (
            axum::http::StatusCode::CREATED,
            Json(CacheEntryResponse {
                key: req.key,
                path: req.path,
                size_bytes: req.size_bytes,
                created_at: now.to_rfc3339(),
                expires_at: expires_at.map(|e| e.to_rfc3339()),
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

/// Delete a cache entry.
pub async fn delete_cache(
    State(state): State<AppState>,
    Path((owner, repo_name, cache_key)): Path<(String, String, String)>,
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

    match sqlx::query("DELETE FROM pipeline_caches WHERE repo_id = $1 AND key = $2")
        .bind(repo_id)
        .bind(&cache_key)
        .execute(pool)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => {
            (axum::http::StatusCode::NO_CONTENT, "").into_response()
        }
        Ok(_) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("cache entry not found".into()).error_response()),
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

type CacheRow = (
    String,
    String,
    i64,
    chrono::DateTime<Utc>,
    Option<chrono::DateTime<Utc>>,
);

async fn list_caches_db(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    limit: i64,
    offset: i64,
    prefix: Option<&str>,
) -> std::result::Result<Vec<CacheEntryResponse>, sqlx::Error> {
    let rows: Vec<CacheRow> = if let Some(pfx) = prefix {
        sqlx::query_as(
            "SELECT key, path, size_bytes, created_at, expires_at FROM pipeline_caches
             WHERE repo_id = $1 AND key LIKE $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
        )
        .bind(repo_id)
        .bind(format!("{pfx}%"))
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as(
            "SELECT key, path, size_bytes, created_at, expires_at FROM pipeline_caches
             WHERE repo_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(repo_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };

    Ok(rows
        .into_iter()
        .map(
            |(key, path, size_bytes, created_at, expires_at)| CacheEntryResponse {
                key,
                path,
                size_bytes,
                created_at: created_at.to_rfc3339(),
                expires_at: expires_at.map(|e| e.to_rfc3339()),
            },
        )
        .collect())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_response_serialize() {
        let resp = CacheEntryResponse {
            key: "cargo-deps-v1".to_string(),
            path: "target/".to_string(),
            size_bytes: 1024 * 1024,
            created_at: "2025-01-01T00:00:00+00:00".to_string(),
            expires_at: Some("2025-01-08T00:00:00+00:00".to_string()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("cargo-deps-v1"));
        assert!(json.contains("1048576"));
    }

    #[test]
    fn test_create_cache_request_deserialize() {
        let json = r#"{"key": "deps-v1", "path": "node_modules/", "size_bytes": 2048}"#;
        let req: CreateCacheRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.key, "deps-v1");
        assert_eq!(req.path, "node_modules/");
        assert_eq!(req.size_bytes, 2048);
        assert!(req.ttl_secs.is_none());
    }

    #[test]
    fn test_create_cache_request_with_ttl() {
        let json =
            r#"{"key": "deps-v2", "path": "target/", "size_bytes": 4096, "ttl_secs": 604800}"#;
        let req: CreateCacheRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.ttl_secs, Some(604800));
    }

    #[test]
    fn test_cache_list_params_defaults() {
        let json = r#"{}"#;
        let params: CacheListParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.limit, 50);
        assert_eq!(params.offset, 0);
        assert!(params.prefix.is_none());
    }

    #[test]
    fn test_cache_list_params_custom() {
        let json = r#"{"limit": 10, "prefix": "cargo-"}"#;
        let params: CacheListParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.limit, 10);
        assert_eq!(params.prefix.as_deref(), Some("cargo-"));
    }

    #[test]
    fn test_cache_entry_response_no_expiry() {
        let resp = CacheEntryResponse {
            key: "build-output".to_string(),
            path: "dist/".to_string(),
            size_bytes: 0,
            created_at: "2025-01-01T00:00:00+00:00".to_string(),
            expires_at: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("null") || !json.contains("expires_at"));
    }
}
