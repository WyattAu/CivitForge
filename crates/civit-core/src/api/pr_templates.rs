#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, OptionalAuthUser};
use crate::error::CoreError;
use axum::{
    Router,
    extract::{Path, State},
    response::{IntoResponse, Json},
    routing::get,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreatePrTemplateRequest {
    pub name: String,
    pub title: Option<String>,
    pub body: Option<String>,
    pub base_branch: Option<String>,
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePrTemplateRequest {
    pub name: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub base_branch: Option<String>,
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PrTemplateResponse {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub title: String,
    pub body: String,
    pub base_branch: String,
    pub labels: Vec<String>,
    pub created_at: DateTime<Utc>,
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

pub async fn list_pr_templates(
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

    let templates = state.db.list_pr_templates(repo_id).await;
    match templates {
        Ok(t) => (axum::http::StatusCode::OK, Json(t)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

pub async fn create_pr_template(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Json(req): Json<CreatePrTemplateRequest>,
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

    if req.name.trim().is_empty() {
        return err_response(axum::http::StatusCode::BAD_REQUEST, "template name is required");
    }

    let title = req.title.unwrap_or_default();
    let body = req.body.unwrap_or_default();
    let base_branch = req.base_branch.unwrap_or_else(|| "main".into());
    let labels = req.labels.unwrap_or_default();

    match state
        .db
        .create_pr_template(repo_id, &req.name, &title, &body, &base_branch, &labels)
        .await
    {
        Ok(t) => (axum::http::StatusCode::CREATED, Json(t)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

pub async fn update_pr_template(
    State(state): State<AppState>,
    Path((owner, name, template_id)): Path<(String, String, Uuid)>,
    _auth: AuthUser,
    Json(req): Json<UpdatePrTemplateRequest>,
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

    match state
        .db
        .update_pr_template(
            template_id,
            repo_id,
            req.name.as_deref(),
            req.title.as_deref(),
            req.body.as_deref(),
            req.base_branch.as_deref(),
            req.labels.as_deref(),
        )
        .await
    {
        Ok(t) => (axum::http::StatusCode::OK, Json(t)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

pub async fn delete_pr_template(
    State(state): State<AppState>,
    Path((owner, name, template_id)): Path<(String, String, Uuid)>,
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

    match state
        .db
        .delete_pr_template(template_id, repo_id)
        .await
    {
        Ok(()) => (axum::http::StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

pub fn pr_template_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/pr-templates",
            get(list_pr_templates).post(create_pr_template),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/pr-templates/{template_id}",
            axum::routing::patch(update_pr_template)
                .delete(delete_pr_template),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_pr_template_request_deserialize() {
        let json = "{\"name\":\"Feature\",\"title\":\"feat: \",\"body\":\"## Description\",\"base_branch\":\"main\",\"labels\":[\"feature\"]}";
        let req: CreatePrTemplateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Feature");
        assert_eq!(req.title, Some("feat: ".into()));
        assert_eq!(req.base_branch, Some("main".into()));
        assert_eq!(req.labels, Some(vec!["feature".into()]));
    }

    #[test]
    fn test_update_pr_template_request_deserialize() {
        let json = r#"{"name":"Updated Name"}"#;
        let req: UpdatePrTemplateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, Some("Updated Name".into()));
        assert!(req.title.is_none());
    }
}
