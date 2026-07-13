#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, OptionalAuthUser};
use crate::error::CoreError;
use axum::{
    Router,
    extract::{Path, State},
    response::{IntoResponse, Json},
    routing::{delete, get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct EnablePagesRequest {
    pub url: String,
    #[serde(default = "default_branch")]
    pub branch: String,
    #[serde(default = "default_path")]
    pub path: String,
    #[serde(default = "default_true")]
    pub public: bool,
}

fn default_branch() -> String {
    "main".into()
}

fn default_path() -> String {
    "/".into()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PagesSiteResponse {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub url: String,
    pub branch: String,
    pub path: String,
    pub public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn err_response(status: axum::http::StatusCode, msg: &str) -> axum::response::Response {
    (
        status,
        Json(CoreError::NotFound(msg.to_string()).error_response()),
    )
        .into_response()
}

fn internal_err(msg: &str) -> axum::response::Response {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        Json(CoreError::Database(msg.to_string()).error_response()),
    )
        .into_response()
}

async fn get_repo_id(pool: &sqlx::PgPool, owner: &str, name: &str) -> Option<Uuid> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT r.id FROM repositories r JOIN users u ON r.owner_id = u.id WHERE u.username = $1 AND r.name = $2",
    )
    .bind(owner)
    .bind(name)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

pub async fn enable_pages(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<EnablePagesRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                axum::http::StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    if req.url.trim().is_empty() {
        return err_response(axum::http::StatusCode::BAD_REQUEST, "url is required");
    }

    let _auth_id = uuid::Uuid::parse_str(&auth.user_id).unwrap_or(uuid::Uuid::nil());

    match state
        .db
        .enable_pages(repo_id, &req.url, &req.branch, &req.path, req.public)
        .await
    {
        Ok(site) => {
            let resp = PagesSiteResponse {
                id: site.id,
                repo_id: site.repo_id,
                url: site.url,
                branch: site.branch,
                path: site.path,
                public: site.public,
                created_at: site.created_at,
                updated_at: site.updated_at,
            };
            (axum::http::StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(e) => internal_err(&e.to_string()),
    }
}

pub async fn disable_pages(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                axum::http::StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    match state.db.disable_pages(repo_id).await {
        Ok(()) => (axum::http::StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

pub async fn get_pages(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                axum::http::StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    match state.db.get_pages_site(repo_id).await {
        Ok(Some(site)) => {
            let resp = PagesSiteResponse {
                id: site.id,
                repo_id: site.repo_id,
                url: site.url,
                branch: site.branch,
                path: site.path,
                public: site.public,
                created_at: site.created_at,
                updated_at: site.updated_at,
            };
            (axum::http::StatusCode::OK, Json(resp)).into_response()
        }
        Ok(None) => err_response(
            axum::http::StatusCode::NOT_FOUND,
            &format!("Pages not enabled for {owner}/{name}"),
        ),
        Err(e) => internal_err(&e.to_string()),
    }
}

pub fn pages_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/pages",
            get(get_pages)
                .post(enable_pages)
                .delete(disable_pages),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enable_pages_request_deserialize() {
        let json = r#"{"url":"https://example.pages.dev","branch":"main","path":"/docs","public":true}"#;
        let req: EnablePagesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.url, "https://example.pages.dev");
        assert_eq!(req.branch, "main");
        assert_eq!(req.path, "/docs");
        assert!(req.public);
    }

    #[test]
    fn test_enable_pages_request_defaults() {
        let json = r#"{"url":"https://example.pages.dev"}"#;
        let req: EnablePagesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.branch, "main");
        assert_eq!(req.path, "/");
        assert!(req.public);
    }
}
