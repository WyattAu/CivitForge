#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct NotificationResponse {
    pub id: String,
    pub user_id: String,
    pub kind: String,
    pub title: String,
    pub body: Option<String>,
    pub repo_name: Option<String>,
    pub read: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ListNotificationsParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    pub read: Option<bool>,
}

fn default_page() -> u32 {
    1
}
fn default_per_page() -> u32 {
    20
}

pub async fn list_notifications(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<ListNotificationsParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let user_id = match _auth.user_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(CoreError::Auth("invalid user id in token".into()).error_response()),
            )
                .into_response();
        }
    };

    let offset = (params.page.saturating_sub(1) * params.per_page) as i64;

    let (query_str, bind_count) = match params.read {
        Some(true) => ("SELECT id, user_id, kind, title, body, repo_name, read, created_at FROM notifications WHERE user_id = $1 AND read = true ORDER BY created_at DESC LIMIT $2 OFFSET $3".to_string(), 3),
        Some(false) => ("SELECT id, user_id, kind, title, body, repo_name, read, created_at FROM notifications WHERE user_id = $1 AND read = false ORDER BY created_at DESC LIMIT $2 OFFSET $3".to_string(), 3),
        None => ("SELECT id, user_id, kind, title, body, repo_name, read, created_at FROM notifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3".to_string(), 3),
    };

    let mut q = sqlx::query_as::<
        _,
        (
            Uuid,
            Uuid,
            String,
            String,
            Option<String>,
            Option<String>,
            bool,
            String,
        ),
    >(sqlx::AssertSqlSafe(query_str))
    .bind(user_id);
    if bind_count >= 2 {
        q = q.bind(params.per_page as i64);
    }
    if bind_count >= 3 {
        q = q.bind(offset);
    }

    let rows = q.fetch_all(pool).await;

    match rows {
        Ok(rows) => {
            let notifs: Vec<NotificationResponse> = rows
                .into_iter()
                .map(
                    |(id, uid, kind, title, body, repo_name, read, created_at)| {
                        NotificationResponse {
                            id: id.to_string(),
                            user_id: uid.to_string(),
                            kind,
                            title,
                            body,
                            repo_name,
                            read,
                            created_at,
                        }
                    },
                )
                .collect();
            (StatusCode::OK, Json(notifs)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn mark_notification_read(
    State(state): State<AppState>,
    _auth: AuthUser,
    notification_id: String,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let user_id = match _auth.user_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(CoreError::Auth("invalid user id in token".into()).error_response()),
            )
                .into_response();
        }
    };

    let nid = match Uuid::parse_str(&notification_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid notification id".into()).error_response()),
            )
                .into_response();
        }
    };

    match sqlx::query("UPDATE notifications SET read = true WHERE id = $1 AND user_id = $2")
        .bind(nid)
        .bind(user_id)
        .execute(pool)
        .await
    {
        Ok(row) if row.rows_affected() > 0 => {
            (StatusCode::OK, Json(serde_json::json!({"status": "read"}))).into_response()
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("notification not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn unread_count(State(state): State<AppState>, _auth: AuthUser) -> impl IntoResponse {
    let pool = state.db.pool();
    let user_id = match _auth.user_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(CoreError::Auth("invalid user id in token".into()).error_response()),
            )
                .into_response();
        }
    };

    let count: Option<(i64,)> =
        sqlx::query_as("SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND read = false")
            .bind(user_id)
            .fetch_one(pool)
            .await
            .ok();

    (
        StatusCode::OK,
        Json(serde_json::json!({"count": count.map(|(c,)| c).unwrap_or(0)})),
    )
        .into_response()
}

pub fn notification_routes() -> axum::Router<AppState> {
    use axum::routing::{get, patch};
    axum::Router::new()
        .route("/api/v1/notifications", get(list_notifications))
        .route("/api/v1/notifications/unread-count", get(unread_count))
        .route(
            "/api/v1/notifications/{notification_id}/read",
            patch(mark_notification_read),
        )
}
