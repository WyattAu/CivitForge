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
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct TokenResponse {
    pub id: String,
    pub name: String,
    pub scopes: Vec<String>,
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    pub name: String,
    #[serde(default = "default_scopes")]
    pub scopes: Vec<String>,
    pub expires_at: Option<String>, // ISO8601 datetime or days string like "90d"
}

fn default_scopes() -> Vec<String> {
    vec!["read".to_string()]
}

#[derive(Debug, Deserialize)]
pub struct ListTokensParams {
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

/// Validate scope names against allowed set
fn validate_scopes(scopes: &[String]) -> Result<(), String> {
    let allowed = [
        "read",
        "write",
        "admin",
        "repo:read",
        "repo:write",
        "user:read",
        "org:read",
        "org:write",
        "ci:read",
        "ci:write",
        "issues:read",
        "issues:write",
        "packages:read",
        "packages:write",
    ];
    for s in scopes {
        if !allowed.contains(&s.as_str()) {
            return Err(format!("invalid scope: {s}"));
        }
    }
    Ok(())
}

/// Hash a token for storage (SHA-256)
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Generate a random token string (40 bytes hex = 80 chars)
fn generate_token() -> String {
    let mut random_bytes = [0u8; 40];
    rand::fill(&mut random_bytes);
    hex::encode(random_bytes)
}

/// Parse user_id from AuthUser
fn parse_user_id(auth: &AuthUser) -> Uuid {
    auth.user_id
        .parse()
        .unwrap_or_else(|_| auth.user_id.parse().unwrap())
}

/// List personal access tokens for the authenticated user
pub async fn list_tokens(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<ListTokensParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let user_id = parse_user_id(&_auth);

    let offset = (params.page.saturating_sub(1) * params.per_page) as i64;

    let rows = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            serde_json::Value,
            DateTime<Utc>,
            Option<DateTime<Utc>>,
            Option<DateTime<Utc>>,
        ),
    >(
        "SELECT id, name, scopes, created_at, last_used_at, expires_at
         FROM access_tokens WHERE user_id = $1 AND expires_at IS NULL OR expires_at > NOW()
         ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(user_id)
    .bind(params.per_page as i64)
    .bind(offset)
    .fetch_all(pool)
    .await;

    match rows {
        Ok(rows) => {
            let tokens: Vec<TokenResponse> = rows
                .into_iter()
                .map(|(id, name, scopes, created_at, last_used_at, expires_at)| {
                    let scopes_vec =
                        serde_json::from_value::<Vec<String>>(scopes).unwrap_or_default();
                    TokenResponse {
                        id: id.to_string(),
                        name,
                        scopes: scopes_vec,
                        created_at: created_at.to_rfc3339(),
                        last_used_at: last_used_at.map(|dt| dt.to_rfc3339()),
                        expires_at: expires_at.map(|dt| dt.to_rfc3339()),
                    }
                })
                .collect();
            (StatusCode::OK, Json(tokens)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Create a new personal access token
pub async fn create_token(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(req): Json<CreateTokenRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let user_id = parse_user_id(&_auth);

    // Validate scopes
    if let Err(e) = validate_scopes(&req.scopes) {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::BadRequest(e).error_response()),
        )
            .into_response();
    }

    // Check name length
    if req.name.is_empty() || req.name.len() > 255 {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::BadRequest("name must be 1-255 characters".into()).error_response()),
        )
            .into_response();
    }

    // Generate token and hash
    let raw_token = generate_token();
    let token_hash = hash_token(&raw_token);
    let scopes_json = serde_json::json!(req.scopes);

    // Parse expires_at
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
        "INSERT INTO access_tokens (user_id, name, token_hash, scopes, expires_at) VALUES ($1, $2, $3, $4, $5) RETURNING id, created_at",
    )
    .bind(user_id)
    .bind(&req.name)
    .bind(&token_hash)
    .bind(&scopes_json)
    .bind(expires_at)
    .fetch_one(pool)
    .await;

    match result {
        Ok((id, created_at)) => {
            // Return the raw token ONLY on creation
            #[derive(Serialize)]
            struct CreatedToken {
                id: String,
                name: String,
                token: String,
                scopes: Vec<String>,
                created_at: String,
                expires_at: Option<String>,
            }
            let expires_str = expires_at.map(|dt| dt.to_rfc3339());
            (
                StatusCode::CREATED,
                Json(CreatedToken {
                    id: id.to_string(),
                    name: req.name,
                    token: raw_token,
                    scopes: req.scopes,
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

/// Delete a personal access token
pub async fn delete_token(
    State(state): State<AppState>,
    Path(token_id): Path<String>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let user_id = parse_user_id(&_auth);

    let token_uuid = match Uuid::parse_str(&token_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid token id".into()).error_response()),
            )
                .into_response();
        }
    };

    let result = sqlx::query("DELETE FROM access_tokens WHERE id = $1 AND user_id = $2")
        .bind(token_uuid)
        .bind(user_id)
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
            Json(CoreError::NotFound("token not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub fn token_routes() -> axum::Router<AppState> {
    use axum::routing::{delete, get};
    axum::Router::new()
        .route("/api/v1/user/tokens", get(list_tokens))
        .route("/api/v1/user/tokens/{token_id}", delete(delete_token))
}
