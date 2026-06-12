#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct WebhookResponse {
    pub id: String,
    pub repo_id: String,
    pub url: String,
    pub events: Vec<String>,
    pub active: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWebhookRequest {
    pub url: String,
    #[serde(default = "default_events")]
    pub events: Vec<String>,
    pub active: Option<bool>,
}

fn default_events() -> Vec<String> {
    vec!["push".to_string()]
}

#[derive(Debug, Deserialize)]
pub struct ListWebhooksParams {
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

fn validate_events(events: &[String]) -> Result<(), String> {
    let allowed = [
        "push",
        "tag",
        "delete",
        "issue",
        "issue_comment",
        "pull_request",
        "pull_request_review",
        "wiki",
        "release",
        "fork",
        "member",
        "repository",
        "star",
        "watch",
        "pipeline",
        "deploy",
    ];
    for e in events {
        if !allowed.contains(&e.as_str()) {
            return Err(format!("invalid event: {e}"));
        }
    }
    Ok(())
}

async fn resolve_repo(
    state: &AppState,
    owner: &str,
    name: &str,
) -> Result<Uuid, axum::response::Response> {
    let owner_uuid = if let Ok(id) = Uuid::parse_str(owner) {
        id
    } else if let Ok(user) = state.db.get_user_by_username(owner).await {
        user.id
    } else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("user not found".into()).error_response()),
        )
            .into_response());
    };

    state
        .db
        .get_repo_by_owner_name(owner_uuid, name)
        .await
        .map(|r| r.id)
        .map_err(|_| {
            (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response()
        })
}

pub async fn list_webhooks(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Query(params): Query<ListWebhooksParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match resolve_repo(&state, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let offset = (params.page.saturating_sub(1) * params.per_page) as i64;
    let rows = sqlx::query_as::<_, (Uuid, Uuid, String, Vec<String>, bool, DateTime<Utc>)>(
        "SELECT id, repo_id, url, events, active, created_at FROM webhooks WHERE repo_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(repo_id)
    .bind(params.per_page as i64)
    .bind(offset)
    .fetch_all(pool)
    .await;

    match rows {
        Ok(rows) => {
            let hooks: Vec<WebhookResponse> = rows
                .into_iter()
                .map(
                    |(id, _rid, url, events, active, created_at)| WebhookResponse {
                        id: id.to_string(),
                        repo_id: repo_id.to_string(),
                        url,
                        events,
                        active,
                        created_at: created_at.to_rfc3339(),
                    },
                )
                .collect();
            (StatusCode::OK, Json(hooks)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn create_webhook(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Json(req): Json<CreateWebhookRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match resolve_repo(&state, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    if let Err(e) = validate_events(&req.events) {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::BadRequest(e).error_response()),
        )
            .into_response();
    }

    let active = req.active.unwrap_or(true);

    let result = sqlx::query_as::<_, (Uuid, String, Vec<String>, bool, DateTime<Utc>)>(
        "INSERT INTO webhooks (repo_id, url, events, active) VALUES ($1, $2, $3, $4) RETURNING id, url, events, active, created_at",
    )
    .bind(repo_id)
    .bind(&req.url)
    .bind(&req.events)
    .bind(active)
    .fetch_one(pool)
    .await;

    match result {
        Ok((id, url, events, active, created_at)) => (
            StatusCode::CREATED,
            Json(WebhookResponse {
                id: id.to_string(),
                repo_id: repo_id.to_string(),
                url,
                events,
                active,
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

pub async fn delete_webhook(
    State(state): State<AppState>,
    Path((owner, name, webhook_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match resolve_repo(&state, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let wid = match Uuid::parse_str(&webhook_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid webhook id".into()).error_response()),
            )
                .into_response();
        }
    };

    let result = sqlx::query("DELETE FROM webhooks WHERE id = $1 AND repo_id = $2")
        .bind(wid)
        .bind(repo_id)
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
            Json(CoreError::NotFound("webhook not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListDeliveriesParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

#[derive(Debug, Serialize)]
pub struct DeliveryResponse {
    pub id: String,
    pub event: String,
    pub status: String,
    pub attempts: i32,
    pub last_error: Option<String>,
    pub next_retry_at: Option<String>,
    pub created_at: String,
}

pub async fn list_webhook_deliveries(
    State(state): State<AppState>,
    Path((owner, name, webhook_id)): Path<(String, String, String)>,
    _auth: AuthUser,
    Query(params): Query<ListDeliveriesParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let _repo_id = match resolve_repo(&state, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let wid = match Uuid::parse_str(&webhook_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid webhook id".into()).error_response()),
            )
                .into_response();
        }
    };

    let offset = (params.page.saturating_sub(1) * params.per_page) as i64;
    let rows = sqlx::query_as::<_, (Uuid, String, String, i32, Option<String>, Option<DateTime<Utc>>, DateTime<Utc>)>(
        "SELECT id, event, status, attempts, last_error, next_retry_at, created_at \
         FROM webhook_deliveries WHERE webhook_id = $1 \
         ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(wid)
    .bind(params.per_page as i64)
    .bind(offset)
    .fetch_all(pool)
    .await;

    match rows {
        Ok(rows) => {
            let deliveries: Vec<DeliveryResponse> = rows
                .into_iter()
                .map(
                    |(id, event, status, attempts, last_error, next_retry_at, created_at)| {
                        DeliveryResponse {
                            id: id.to_string(),
                            event,
                            status,
                            attempts,
                            last_error,
                            next_retry_at: next_retry_at.map(|t| t.to_rfc3339()),
                            created_at: created_at.to_rfc3339(),
                        }
                    },
                )
                .collect();
            (StatusCode::OK, Json(deliveries)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub fn webhook_routes() -> axum::Router<AppState> {
    use axum::routing::delete;
    axum::Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/webhooks/{webhook_id}",
            delete(delete_webhook),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/webhooks/{webhook_id}/deliveries",
            get(list_webhook_deliveries),
        )
}
