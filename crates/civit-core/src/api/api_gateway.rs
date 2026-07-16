#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayRoute {
    pub id: String,
    pub path: String,
    pub method: String,
    pub backend_url: String,
    pub rate_limit: i32,
    pub timeout_ms: i32,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateRouteRequest {
    pub path: String,
    pub method: String,
    pub backend_url: String,
    #[serde(default = "default_rate_limit")]
    pub rate_limit: i32,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: i32,
}

fn default_rate_limit() -> i32 {
    100
}
fn default_timeout_ms() -> i32 {
    30000
}

#[derive(Debug, Deserialize)]
pub struct UpdateRouteRequest {
    pub path: Option<String>,
    pub method: Option<String>,
    pub backend_url: Option<String>,
    pub rate_limit: Option<i32>,
    pub timeout_ms: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayKey {
    pub id: String,
    pub name: String,
    pub scopes: Vec<String>,
    pub rate_limit: i32,
    pub enabled: bool,
    pub expires_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateKeyRequest {
    pub name: String,
    #[serde(default = "default_key_scopes")]
    pub scopes: Vec<String>,
    #[serde(default = "default_rate_limit")]
    pub rate_limit: i32,
    pub expires_at: Option<String>,
}

fn default_key_scopes() -> Vec<String> {
    vec!["read".to_string()]
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}
fn default_per_page() -> u32 {
    20
}

/// Hash an API key for storage (SHA-256)
fn hash_api_key(key: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Generate a random API key with cf_gw_ prefix
fn generate_api_key() -> String {
    use rand::RngExt;
    let mut bytes = [0u8; 48];
    rand::rng().fill(&mut bytes[..]);
    format!("cf_gw_{}", hex::encode(bytes))
}

/// List API gateway routes (admin only)
pub async fn list_routes(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let offset = (params.page.saturating_sub(1) * params.per_page) as i64;

    let rows = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            String,
            String,
            i32,
            i32,
            bool,
            DateTime<Utc>,
        ),
    >(
        "SELECT id, path, method, backend_url, rate_limit, timeout_ms, enabled, created_at
         FROM api_gateway_routes ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(params.per_page as i64)
    .bind(offset)
    .fetch_all(pool)
    .await;

    match rows {
        Ok(rows) => {
            let routes: Vec<GatewayRoute> = rows
                .into_iter()
                .map(|(id, path, method, backend_url, rate_limit, timeout_ms, enabled, created_at)| {
                    GatewayRoute {
                        id: id.to_string(),
                        path,
                        method,
                        backend_url,
                        rate_limit,
                        timeout_ms,
                        enabled,
                        created_at: created_at.to_rfc3339(),
                    }
                })
                .collect();
            (StatusCode::OK, Json(routes)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Create an API gateway route (admin only)
pub async fn create_route(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(req): Json<CreateRouteRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();

    if req.path.is_empty() || req.method.is_empty() || req.backend_url.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::BadRequest("path, method, and backend_url are required".into()).error_response()),
        )
            .into_response();
    }

    let result = sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
        "INSERT INTO api_gateway_routes (path, method, backend_url, rate_limit, timeout_ms)
         VALUES ($1, $2, $3, $4, $5) RETURNING id, created_at",
    )
    .bind(&req.path)
    .bind(&req.method)
    .bind(&req.backend_url)
    .bind(req.rate_limit)
    .bind(req.timeout_ms)
    .fetch_one(pool)
    .await;

    match result {
        Ok((id, created_at)) => {
            let path = req.path.clone();
            let method = req.method.clone();
            let backend_url = req.backend_url.clone();
            (
                StatusCode::CREATED,
                Json(GatewayRoute {
                    id: id.to_string(),
                    path,
                    method,
                    backend_url,
                    rate_limit: req.rate_limit,
                    timeout_ms: req.timeout_ms,
                    enabled: true,
                    created_at: created_at.to_rfc3339(),
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Update an API gateway route (admin only)
pub async fn update_route(
    State(state): State<AppState>,
    Path(route_id): Path<String>,
    _auth: AuthUser,
    Json(req): Json<UpdateRouteRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let route_uuid = match Uuid::parse_str(&route_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid route id".into()).error_response()),
            )
                .into_response();
        }
    };

    let result = sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
        "UPDATE api_gateway_routes SET
            path = COALESCE($2, path),
            method = COALESCE($3, method),
            backend_url = COALESCE($4, backend_url),
            rate_limit = COALESCE($5, rate_limit),
            timeout_ms = COALESCE($6, timeout_ms),
            enabled = COALESCE($7, enabled)
         WHERE id = $1 RETURNING id, created_at",
    )
    .bind(route_uuid)
    .bind(req.path.clone())
    .bind(req.method.clone())
    .bind(req.backend_url.clone())
    .bind(req.rate_limit)
    .bind(req.timeout_ms)
    .bind(req.enabled)
    .fetch_one(pool)
    .await;

    match result {
        Ok((id, created_at)) => (
            StatusCode::OK,
            Json(GatewayRoute {
                id: id.to_string(),
                path: req.path.unwrap_or_default(),
                method: req.method.unwrap_or_default(),
                backend_url: req.backend_url.unwrap_or_default(),
                rate_limit: req.rate_limit.unwrap_or(100),
                timeout_ms: req.timeout_ms.unwrap_or(30000),
                enabled: req.enabled.unwrap_or(true),
                created_at: created_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Delete an API gateway route (admin only)
pub async fn delete_route(
    State(state): State<AppState>,
    Path(route_id): Path<String>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let route_uuid = match Uuid::parse_str(&route_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid route id".into()).error_response()),
            )
                .into_response();
        }
    };

    let result = sqlx::query("DELETE FROM api_gateway_routes WHERE id = $1")
        .bind(route_uuid)
        .execute(pool)
        .await;

    match result {
        Ok(row) if row.rows_affected() > 0 => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "deleted"})),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("route not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// List API gateway keys (admin only)
pub async fn list_keys(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let offset = (params.page.saturating_sub(1) * params.per_page) as i64;

    let rows = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            serde_json::Value,
            i32,
            bool,
            Option<DateTime<Utc>>,
            DateTime<Utc>,
        ),
    >(
        "SELECT id, name, scopes, rate_limit, enabled, expires_at, created_at
         FROM api_gateway_keys ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(params.per_page as i64)
    .bind(offset)
    .fetch_all(pool)
    .await;

    match rows {
        Ok(rows) => {
            let keys: Vec<GatewayKey> = rows
                .into_iter()
                .map(|(id, name, scopes, rate_limit, enabled, expires_at, created_at)| {
                    let scopes_vec =
                        serde_json::from_value::<Vec<String>>(scopes).unwrap_or_default();
                    GatewayKey {
                        id: id.to_string(),
                        name,
                        scopes: scopes_vec,
                        rate_limit,
                        enabled,
                        expires_at: expires_at.map(|dt| dt.to_rfc3339()),
                        created_at: created_at.to_rfc3339(),
                    }
                })
                .collect();
            (StatusCode::OK, Json(keys)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Create an API gateway key (admin only)
pub async fn create_key(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(req): Json<CreateKeyRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();

    if req.name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::BadRequest("name is required".into()).error_response()),
        )
            .into_response();
    }

    let raw_key = generate_api_key();
    let key_hash = hash_api_key(&raw_key);
    let scopes_json = serde_json::json!(req.scopes);

    let expires_at = match req.expires_at.as_deref() {
        None => None,
        Some("never") => None,
        Some(days) if days.ends_with('d') => {
            let n: i64 = days.trim_end_matches('d').parse().unwrap_or(365);
            Some(chrono::Utc::now() + chrono::Duration::days(n))
        }
        Some(iso) => match chrono::DateTime::parse_from_rfc3339(iso) {
            Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(
                        CoreError::BadRequest(
                            "expires_at must be ISO8601 or 'Nd' (e.g., '90d')".into(),
                        )
                        .error_response(),
                    ),
                )
                    .into_response();
            }
        },
    };

    let result = sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
        "INSERT INTO api_gateway_keys (key_hash, name, scopes, rate_limit, expires_at)
         VALUES ($1, $2, $3, $4, $5) RETURNING id, created_at",
    )
    .bind(&key_hash)
    .bind(&req.name)
    .bind(&scopes_json)
    .bind(req.rate_limit)
    .bind(expires_at)
    .fetch_one(pool)
    .await;

    match result {
        Ok((id, created_at)) => {
            #[derive(Serialize)]
            struct CreatedKey {
                id: String,
                name: String,
                key: String,
                scopes: Vec<String>,
                rate_limit: i32,
                created_at: String,
                expires_at: Option<String>,
            }
            let expires_str = expires_at.map(|dt| dt.to_rfc3339());
            (
                StatusCode::CREATED,
                Json(CreatedKey {
                    id: id.to_string(),
                    name: req.name,
                    key: raw_key,
                    scopes: req.scopes,
                    rate_limit: req.rate_limit,
                    created_at: created_at.to_rfc3339(),
                    expires_at: expires_str,
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Delete an API gateway key (admin only)
pub async fn delete_key(
    State(state): State<AppState>,
    Path(key_id): Path<String>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let key_uuid = match Uuid::parse_str(&key_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid key id".into()).error_response()),
            )
                .into_response();
        }
    };

    let result = sqlx::query("DELETE FROM api_gateway_keys WHERE id = $1")
        .bind(key_uuid)
        .execute(pool)
        .await;

    match result {
        Ok(row) if row.rows_affected() > 0 => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "deleted"})),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("key not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Validate an API gateway key (used by gateway middleware)
pub async fn validate_api_key(
    pool: &sqlx::PgPool,
    key: &str,
) -> Result<Option<(String, Vec<String>, i32)>, sqlx::Error> {
    let key_hash = hash_api_key(key);

    let row = sqlx::query_as::<_, (Uuid, serde_json::Value, i32)>(
        "SELECT id, scopes, rate_limit FROM api_gateway_keys
         WHERE key_hash = $1 AND enabled = true AND (expires_at IS NULL OR expires_at > NOW())",
    )
    .bind(&key_hash)
    .fetch_optional(pool)
    .await?;

    match row {
        Some((id, scopes, rate_limit)) => {
            let scopes_vec =
                serde_json::from_value::<Vec<String>>(scopes).unwrap_or_default();
            Ok(Some((id.to_string(), scopes_vec, rate_limit)))
        }
        None => Ok(None),
    }
}

pub fn gateway_routes() -> axum::Router<AppState> {
    use axum::routing::{delete, get};
    axum::Router::new()
        .route("/api/v1/admin/gateway/routes", get(list_routes).post(create_route))
        .route(
            "/api/v1/admin/gateway/routes/{route_id}",
            get(get_route).put(update_route).delete(delete_route),
        )
        .route("/api/v1/admin/gateway/keys", get(list_keys).post(create_key))
        .route("/api/v1/admin/gateway/keys/{key_id}", delete(delete_key))
}

async fn get_route(
    State(state): State<AppState>,
    Path(route_id): Path<String>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let route_uuid = match Uuid::parse_str(&route_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid route id".into()).error_response()),
            )
                .into_response();
        }
    };

    let row = sqlx::query_as::<_, (Uuid, String, String, String, i32, i32, bool, DateTime<Utc>)>(
        "SELECT id, path, method, backend_url, rate_limit, timeout_ms, enabled, created_at
         FROM api_gateway_routes WHERE id = $1",
    )
    .bind(route_uuid)
    .fetch_optional(pool)
    .await;

    match row {
        Ok(Some((id, path, method, backend_url, rate_limit, timeout_ms, enabled, created_at))) => {
            (
                StatusCode::OK,
                Json(GatewayRoute {
                    id: id.to_string(),
                    path,
                    method,
                    backend_url,
                    rate_limit,
                    timeout_ms,
                    enabled,
                    created_at: created_at.to_rfc3339(),
                }),
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("route not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_api_key_deterministic() {
        let key = "test_key_123";
        let h1 = hash_api_key(key);
        let h2 = hash_api_key(key);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_generate_api_key_prefix() {
        let key = generate_api_key();
        assert!(key.starts_with("cf_gw_"));
        assert!(key.len() > 10);
    }

    #[test]
    fn test_default_rate_limit() {
        assert_eq!(default_rate_limit(), 100);
    }

    #[test]
    fn test_default_timeout_ms() {
        assert_eq!(default_timeout_ms(), 30000);
    }

    #[test]
    fn test_default_key_scopes() {
        assert_eq!(default_key_scopes(), vec!["read".to_string()]);
    }
}
