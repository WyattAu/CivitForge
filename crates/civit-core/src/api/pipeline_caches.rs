//! Pipeline Caches API endpoints.

#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use chrono::Utc;
use civit_ci::caches::{self, CacheEntryResponse, CacheListParams, CreateCacheRequest};
use uuid::Uuid;

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
        .route(
            "/api/v1/repos/{owner}/{repo}/caches/v2",
            get(list_caches_v2),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/caches/v2/stats",
            get(get_cache_statistics),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/caches/v2/{cache_key}/hit",
            post(record_hit),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/caches/v2/{cache_key}/invalidate",
            post(invalidate_specific_cache),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/caches/v2/invalidate-expired",
            post(invalidate_expired),
        )
}

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

    match caches::list_caches_db(
        pool,
        repo_id,
        params.limit,
        params.offset,
        params.prefix.as_deref(),
    )
    .await
    {
        Ok(c) => (axum::http::StatusCode::OK, Json(c)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

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

pub async fn list_caches_v2(
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

    match caches::list_caches_v2(
        pool,
        repo_id,
        params.limit,
        params.offset,
        params.prefix.as_deref(),
    )
    .await
    {
        Ok(c) => (axum::http::StatusCode::OK, Json(c)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_cache_statistics(
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

    match caches::get_cache_stats(pool, repo_id).await {
        Ok(stats) => (axum::http::StatusCode::OK, Json(stats)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn record_hit(
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

    match caches::record_cache_hit(pool, repo_id, &cache_key).await {
        Ok(()) => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({"status": "hit recorded"})),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn invalidate_specific_cache(
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

    match caches::invalidate_cache(pool, repo_id, &cache_key).await {
        Ok(true) => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({"status": "invalidated"})),
        )
            .into_response(),
        Ok(false) => (
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

pub async fn invalidate_expired(
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

    match caches::invalidate_expired_caches(pool, repo_id).await {
        Ok(count) => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({"invalidated": count})),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}
