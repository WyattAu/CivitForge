#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, OptionalAuthUser};
use crate::error::CoreError;
use axum::{
    Router,
    extract::{Path, Query, State},
    response::{IntoResponse, Json},
    routing::{delete, get},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateDiscussionRequest {
    pub title: String,
    pub body: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDiscussionRequest {
    pub title: Option<String>,
    pub body: Option<String>,
    pub category: Option<String>,
    pub is_pinned: Option<bool>,
    pub is_locked: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDiscussionCommentRequest {
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct DiscussionListParams {
    pub per_page: Option<u32>,
    pub page: Option<u32>,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DiscussionResponse {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub title: String,
    pub body: String,
    pub category: String,
    pub author_id: Uuid,
    pub is_pinned: bool,
    pub is_locked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_count: Option<i64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DiscussionCommentResponse {
    pub id: Uuid,
    pub discussion_id: Uuid,
    pub author_id: Uuid,
    pub body: String,
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

pub async fn list_discussions(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(params): Query<DiscussionListParams>,
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

    let limit = params.per_page.map(|p| p.clamp(1, 100) as i64).unwrap_or(30);
    let page = params.page.unwrap_or(1).max(1);
    let offset = ((page - 1) * limit as u32) as i64;

    match state.db.list_discussions(repo_id, limit, offset).await {
        Ok(discussions) => {
            let mut results = Vec::new();
            for d in discussions {
                let comment_count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM discussion_comments WHERE discussion_id = $1",
                )
                .bind(d.id)
                .fetch_one(pool)
                .await
                .unwrap_or(0);

                results.push(DiscussionResponse {
                    id: d.id,
                    repo_id: d.repo_id,
                    title: d.title,
                    body: d.body,
                    category: d.category,
                    author_id: d.author_id,
                    is_pinned: d.is_pinned,
                    is_locked: d.is_locked,
                    created_at: d.created_at,
                    updated_at: d.updated_at,
                    comment_count: Some(comment_count),
                });
            }
            (axum::http::StatusCode::OK, Json(results)).into_response()
        }
        Err(e) => internal_err(&e.to_string()),
    }
}

pub async fn get_discussion(
    State(state): State<AppState>,
    Path((owner, name, discussion_id)): Path<(String, String, Uuid)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let _repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                axum::http::StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    match state.db.get_discussion(discussion_id).await {
        Ok(d) => {
            let comment_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM discussion_comments WHERE discussion_id = $1",
            )
            .bind(d.id)
            .fetch_one(pool)
            .await
            .unwrap_or(0);

            let resp = DiscussionResponse {
                id: d.id,
                repo_id: d.repo_id,
                title: d.title,
                body: d.body,
                category: d.category,
                author_id: d.author_id,
                is_pinned: d.is_pinned,
                is_locked: d.is_locked,
                created_at: d.created_at,
                updated_at: d.updated_at,
                comment_count: Some(comment_count),
            };
            (axum::http::StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => internal_err(&e.to_string()),
    }
}

pub async fn create_discussion(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<CreateDiscussionRequest>,
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

    if req.title.trim().is_empty() {
        return err_response(axum::http::StatusCode::BAD_REQUEST, "title is required");
    }

    let body = req.body.unwrap_or_default();
    let category = req.category.unwrap_or_else(|| "general".into());
    let author_id = uuid::Uuid::parse_str(&auth.user_id).unwrap_or(uuid::Uuid::nil());

    match state
        .db
        .create_discussion(repo_id, &req.title, &body, &category, author_id)
        .await
    {
        Ok(d) => {
            let resp = DiscussionResponse {
                id: d.id,
                repo_id: d.repo_id,
                title: d.title,
                body: d.body,
                category: d.category,
                author_id: d.author_id,
                is_pinned: d.is_pinned,
                is_locked: d.is_locked,
                created_at: d.created_at,
                updated_at: d.updated_at,
                comment_count: Some(0),
            };
            (axum::http::StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(e) => internal_err(&e.to_string()),
    }
}

pub async fn update_discussion(
    State(state): State<AppState>,
    Path((owner, name, discussion_id)): Path<(String, String, Uuid)>,
    _auth: AuthUser,
    Json(req): Json<UpdateDiscussionRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let _repo_id = match get_repo_id(pool, &owner, &name).await {
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
        .update_discussion(
            discussion_id,
            req.title.as_deref(),
            req.body.as_deref(),
            req.category.as_deref(),
            req.is_pinned,
            req.is_locked,
        )
        .await
    {
        Ok(d) => {
            let resp = DiscussionResponse {
                id: d.id,
                repo_id: d.repo_id,
                title: d.title,
                body: d.body,
                category: d.category,
                author_id: d.author_id,
                is_pinned: d.is_pinned,
                is_locked: d.is_locked,
                created_at: d.created_at,
                updated_at: d.updated_at,
                comment_count: None,
            };
            (axum::http::StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => internal_err(&e.to_string()),
    }
}

pub async fn delete_discussion(
    State(state): State<AppState>,
    Path((owner, name, discussion_id)): Path<(String, String, Uuid)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let _repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                axum::http::StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    match state.db.delete_discussion(discussion_id).await {
        Ok(()) => (axum::http::StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

pub async fn list_discussion_comments(
    State(state): State<AppState>,
    Path((_owner, _name, discussion_id)): Path<(String, String, Uuid)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    match state.db.list_discussion_comments(discussion_id).await {
        Ok(comments) => {
            let resp: Vec<DiscussionCommentResponse> = comments
                .into_iter()
                .map(|c| DiscussionCommentResponse {
                    id: c.id,
                    discussion_id: c.discussion_id,
                    author_id: c.author_id,
                    body: c.body,
                    created_at: c.created_at,
                    updated_at: c.updated_at,
                })
                .collect();
            (axum::http::StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => internal_err(&e.to_string()),
    }
}

pub async fn create_discussion_comment(
    State(state): State<AppState>,
    Path((_owner, _name, discussion_id)): Path<(String, String, Uuid)>,
    auth: AuthUser,
    Json(req): Json<CreateDiscussionCommentRequest>,
) -> impl IntoResponse {
    let author_id = uuid::Uuid::parse_str(&auth.user_id).unwrap_or(uuid::Uuid::nil());

    if req.body.trim().is_empty() {
        return err_response(axum::http::StatusCode::BAD_REQUEST, "comment body is required");
    }

    match state
        .db
        .create_discussion_comment(discussion_id, author_id, &req.body)
        .await
    {
        Ok(c) => {
            let resp = DiscussionCommentResponse {
                id: c.id,
                discussion_id: c.discussion_id,
                author_id: c.author_id,
                body: c.body,
                created_at: c.created_at,
                updated_at: c.updated_at,
            };
            (axum::http::StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(e) => internal_err(&e.to_string()),
    }
}

pub async fn delete_discussion_comment(
    State(state): State<AppState>,
    Path((_owner, _name, _discussion_id, comment_id)): Path<(String, String, Uuid, Uuid)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    match state.db.delete_discussion_comment(comment_id).await {
        Ok(()) => (axum::http::StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

pub fn discussion_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/discussions",
            get(list_discussions).post(create_discussion),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/discussions/{discussion_id}",
            get(get_discussion)
                .patch(update_discussion)
                .delete(delete_discussion),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/discussions/{discussion_id}/comments",
            get(list_discussion_comments).post(create_discussion_comment),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/discussions/{discussion_id}/comments/{comment_id}",
            delete(delete_discussion_comment),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_discussion_request_deserialize() {
        let json = r#"{"title":"RFC: New API","body":"Proposal","category":"rfc"}"#;
        let req: CreateDiscussionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.title, "RFC: New API");
        assert_eq!(req.category, Some("rfc".into()));
    }

    #[test]
    fn test_create_discussion_comment_request_deserialize() {
        let json = r#"{"body":"Great idea!"}"#;
        let req: CreateDiscussionCommentRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.body, "Great idea!");
    }
}
