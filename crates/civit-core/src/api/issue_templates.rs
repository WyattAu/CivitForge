#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, OptionalAuthUser};
use crate::error::CoreError;
use axum::{
    Router,
    extract::{Path, State},
    response::{IntoResponse, Json},
    routing::{delete, get, patch, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateIssueTemplateRequest {
    pub name: String,
    pub title: Option<String>,
    pub body: Option<String>,
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIssueTemplateRequest {
    pub name: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct IssueTemplateResponse {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub title: String,
    pub body: String,
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

pub async fn list_issue_templates(
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

    let templates = state.db.list_issue_templates(repo_id).await;
    match templates {
        Ok(t) => (axum::http::StatusCode::OK, Json(t)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

pub async fn create_issue_template(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Json(req): Json<CreateIssueTemplateRequest>,
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
    let labels = req.labels.unwrap_or_default();

    match state
        .db
        .create_issue_template(repo_id, &req.name, &title, &body, &labels)
        .await
    {
        Ok(t) => (axum::http::StatusCode::CREATED, Json(t)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

pub async fn update_issue_template(
    State(state): State<AppState>,
    Path((owner, name, template_id)): Path<(String, String, Uuid)>,
    _auth: AuthUser,
    Json(req): Json<UpdateIssueTemplateRequest>,
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
        .update_issue_template(
            template_id,
            repo_id,
            req.name.as_deref(),
            req.title.as_deref(),
            req.body.as_deref(),
            req.labels.as_deref(),
        )
        .await
    {
        Ok(t) => (axum::http::StatusCode::OK, Json(t)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

pub async fn delete_issue_template(
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
        .delete_issue_template(template_id, repo_id)
        .await
    {
        Ok(()) => (axum::http::StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

pub fn issue_template_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/issue-templates",
            get(list_issue_templates).post(create_issue_template),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/issue-templates/{template_id}",
            axum::routing::patch(update_issue_template)
                .delete(delete_issue_template),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_issue_template_request_deserialize() {
        let json = "{\"name\":\"Bug Report\",\"title\":\"[Bug] \",\"body\":\"## Description\",\"labels\":[\"bug\"]}";
        let req: CreateIssueTemplateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Bug Report");
        assert_eq!(req.title, Some("[Bug] ".into()));
        assert_eq!(req.labels, Some(vec!["bug".into()]));
    }

    #[test]
    fn test_update_issue_template_request_deserialize() {
        let json = r#"{"name":"Updated Name"}"#;
        let req: UpdateIssueTemplateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, Some("Updated Name".into()));
        assert!(req.title.is_none());
    }
}
