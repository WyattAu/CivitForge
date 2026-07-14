//! Pipeline Caches API endpoints.

#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use chrono::Utc;
use civit_ci::caches::{self, CacheEntryResponse, CacheListParams, CreateCacheRequest};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateCacheStrategyRequest {
    pub name: String,
    pub strategy_type: String,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCacheStrategyRequest {
    pub name: Option<String>,
    pub strategy_type: Option<String>,
    pub config: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

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
        .route(
            "/api/v1/repos/{owner}/{repo}/caches/strategies",
            get(list_strategies).post(create_strategy),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/caches/strategies/{strategy_id}",
            get(get_strategy)
                .patch(update_strategy)
                .delete(delete_strategy),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/caches/analytics/{cache_id}",
            get(get_cache_analytics_handler),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/caches/optimization",
            get(get_optimization_report),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/caches/cost",
            get(get_cost_analysis),
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

pub async fn list_strategies(
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

    match caches::list_cache_strategies(pool, repo_id).await {
        Ok(strategies) => (axum::http::StatusCode::OK, Json(strategies)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn create_strategy(
    State(state): State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    _auth: AuthUser,
    Json(req): Json<CreateCacheStrategyRequest>,
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

    let config = req.config.unwrap_or(serde_json::json!({}));
    match caches::create_cache_strategy(pool, repo_id, &req.name, &req.strategy_type, &config).await {
        Ok(strategy) => (axum::http::StatusCode::CREATED, Json(strategy)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_strategy(
    State(state): State<AppState>,
    Path((owner, repo_name, strategy_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let _repo_id = match get_repo_id(pool, &owner, &repo_name).await {
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

    let sid = match Uuid::parse_str(&strategy_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid strategy ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match caches::get_cache_strategy(pool, sid).await {
        Ok(Some(strategy)) => (axum::http::StatusCode::OK, Json(strategy)).into_response(),
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("strategy not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_strategy(
    State(state): State<AppState>,
    Path((owner, repo_name, strategy_id)): Path<(String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<UpdateCacheStrategyRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let _repo_id = match get_repo_id(pool, &owner, &repo_name).await {
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

    let sid = match Uuid::parse_str(&strategy_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid strategy ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match caches::update_cache_strategy(
        pool,
        sid,
        req.name.as_deref(),
        req.strategy_type.as_deref(),
        req.config.as_ref(),
        req.enabled,
    )
    .await
    {
        Ok(strategy) => (axum::http::StatusCode::OK, Json(strategy)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_strategy(
    State(state): State<AppState>,
    Path((owner, repo_name, strategy_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let _repo_id = match get_repo_id(pool, &owner, &repo_name).await {
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

    let sid = match Uuid::parse_str(&strategy_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid strategy ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match caches::delete_cache_strategy(pool, sid).await {
        Ok(true) => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({"status": "deleted"})),
        )
            .into_response(),
        Ok(false) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("strategy not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_cache_analytics_handler(
    State(state): State<AppState>,
    Path((owner, repo_name, cache_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let _repo_id = match get_repo_id(pool, &owner, &repo_name).await {
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

    let cid = match Uuid::parse_str(&cache_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid cache ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match caches::get_cache_analytics(pool, cid).await {
        Ok(Some(analytics)) => (axum::http::StatusCode::OK, Json(analytics)).into_response(),
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("analytics not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_optimization_report(
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

    match caches::get_cache_optimization_report(pool, repo_id).await {
        Ok(report) => (axum::http::StatusCode::OK, Json(report)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_cost_analysis(
    State(state): State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    _auth: AuthUser,
    axum::extract::Query(params): axum::extract::Query<CostParams>,
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

    match caches::get_cache_cost_analysis(pool, repo_id, params.cost_per_gb).await {
        Ok(analysis) => (axum::http::StatusCode::OK, Json(analysis)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct CostParams {
    #[serde(default = "default_cost_per_gb")]
    pub cost_per_gb: f64,
}

fn default_cost_per_gb() -> f64 {
    0.023
}
