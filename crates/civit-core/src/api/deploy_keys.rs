#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct DeployKeyResponse {
    pub id: String,
    pub title: String,
    pub public_key: String,
    pub fingerprint: String,
    pub read_only: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateDeployKeyRequest {
    pub title: String,
    pub public_key: String,
    pub read_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListDeployKeysParams {
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

fn fingerprint(key: &str) -> String {
    use sha2::{Digest, Sha256};
    // Extract the key data part (skip "ssh-rsa ..." etc)
    let parts: Vec<&str> = key.split_whitespace().collect();
    let data = if parts.len() >= 2 { parts[1] } else { key };
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let hash = hasher.finalize();
    let hex = hex::encode(hash);
    // Format as XX:XX:XX:...
    hex.as_bytes()
        .chunks(2)
        .map(|b| {
            let s = std::str::from_utf8(b).unwrap_or("");
            s.to_uppercase()
        })
        .collect::<Vec<_>>()
        .join(":")
}

pub async fn list_deploy_keys(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Query(params): Query<ListDeployKeysParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();

    let owner_uuid = if let Ok(id) = Uuid::parse_str(&owner) {
        id
    } else if let Ok(user) = state.db.get_user_by_username(&owner).await {
        user.id
    } else {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("user not found".into()).error_response()),
        )
            .into_response();
    };

    let repo = match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
    };

    let offset = (params.page.saturating_sub(1) * params.per_page) as i64;
    let rows = sqlx::query_as::<_, (Uuid, String, String, String, bool, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, title, public_key, fingerprint, read_only, created_at FROM deploy_keys WHERE repo_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(repo.id)
    .bind(params.per_page as i64)
    .bind(offset)
    .fetch_all(pool)
    .await;

    match rows {
        Ok(rows) => {
            let keys: Vec<DeployKeyResponse> = rows
                .into_iter()
                .map(
                    |(id, title, public_key, fp, read_only, created_at)| DeployKeyResponse {
                        id: id.to_string(),
                        title,
                        public_key,
                        fingerprint: fp,
                        read_only,
                        created_at: created_at.to_rfc3339(),
                    },
                )
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

pub async fn create_deploy_key(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Json(req): Json<CreateDeployKeyRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();

    let owner_uuid = if let Ok(id) = Uuid::parse_str(&owner) {
        id
    } else if let Ok(user) = state.db.get_user_by_username(&owner).await {
        user.id
    } else {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("user not found".into()).error_response()),
        )
            .into_response();
    };

    let repo = match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
    };

    let read_only = req.read_only.unwrap_or(true);
    let fp = fingerprint(&req.public_key);

    let result = sqlx::query_as::<_, (Uuid, String, String, String, bool, chrono::DateTime<chrono::Utc>) >(
        "INSERT INTO deploy_keys (repo_id, title, public_key, fingerprint, read_only) VALUES ($1, $2, $3, $4, $5) RETURNING id, title, public_key, fingerprint, read_only, created_at",
    )
    .bind(repo.id)
    .bind(&req.title)
    .bind(&req.public_key)
    .bind(&fp)
    .bind(read_only)
    .fetch_one(pool)
    .await;

    match result {
        Ok((id, title, public_key, fp, read_only, created_at)) => (
            StatusCode::CREATED,
            Json(DeployKeyResponse {
                id: id.to_string(),
                title,
                public_key,
                fingerprint: fp,
                read_only,
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

pub async fn delete_deploy_key(
    State(state): State<AppState>,
    Path((owner, name, key_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();

    let owner_uuid = if let Ok(id) = Uuid::parse_str(&owner) {
        id
    } else if let Ok(user) = state.db.get_user_by_username(&owner).await {
        user.id
    } else {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("user not found".into()).error_response()),
        )
            .into_response();
    };

    let repo = match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
    };

    let kid = match Uuid::parse_str(&key_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid key id".into()).error_response()),
            )
                .into_response();
        }
    };

    match sqlx::query("DELETE FROM deploy_keys WHERE id = $1 AND repo_id = $2")
        .bind(kid)
        .bind(repo.id)
        .execute(pool)
        .await
    {
        Ok(row) if row.rows_affected() > 0 => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "deleted"})),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("deploy key not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub fn deploy_key_routes() -> axum::Router<AppState> {
    use axum::routing::delete;
    axum::Router::new().route(
        "/api/v1/repos/{owner}/{name}/deploy-keys/{key_id}",
        delete(delete_deploy_key),
    )
}
