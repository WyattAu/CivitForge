#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize)]
pub struct EventQueueResponse {
    pub id: String,
    pub queue_name: String,
    pub message_count: i32,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateEventQueueRequest {
    pub queue_name: String,
}

#[derive(Debug, Serialize)]
pub struct EventQueueMessageResponse {
    pub id: String,
    pub queue_id: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub created_at: String,
    pub processed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EnqueueMessageRequest {
    pub payload: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct DequeueMessagesParams {
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    10
}

#[derive(Debug, Serialize)]
pub struct QueueStatsResponse {
    pub queue_name: String,
    pub total_messages: i64,
    pub pending_messages: i64,
    pub completed_messages: i64,
    pub failed_messages: i64,
    pub dead_letter_messages: i64,
}

pub async fn create_event_queue(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(req): Json<CreateEventQueueRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let queue_service = crate::event_queues::EventQueueService::new();

    match queue_service.create_queue(pool, &req.queue_name).await {
        Ok(queue) => (
            StatusCode::CREATED,
            Json(EventQueueResponse {
                id: queue.id.to_string(),
                queue_name: queue.queue_name,
                message_count: queue.message_count,
                created_at: queue.created_at.to_rfc3339(),
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

pub async fn get_event_queue(
    State(state): State<AppState>,
    Path(queue_name): Path<String>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let queue_service = crate::event_queues::EventQueueService::new();

    match queue_service.get_queue(pool, &queue_name).await {
        Ok(Some(queue)) => (
            StatusCode::OK,
            Json(EventQueueResponse {
                id: queue.id.to_string(),
                queue_name: queue.queue_name,
                message_count: queue.message_count,
                created_at: queue.created_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("queue not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn enqueue_message(
    State(state): State<AppState>,
    Path(queue_name): Path<String>,
    _auth: AuthUser,
    Json(req): Json<EnqueueMessageRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let queue_service = crate::event_queues::EventQueueService::new();

    match queue_service.enqueue_message(pool, &queue_name, req.payload).await {
        Ok(message) => (
            StatusCode::CREATED,
            Json(EventQueueMessageResponse {
                id: message.id.to_string(),
                queue_id: message.queue_id.to_string(),
                payload: message.payload,
                status: message.status,
                attempts: message.attempts,
                max_attempts: message.max_attempts,
                created_at: message.created_at.to_rfc3339(),
                processed_at: message.processed_at.map(|t| t.to_rfc3339()),
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

pub async fn dequeue_messages(
    State(state): State<AppState>,
    Path(queue_name): Path<String>,
    _auth: AuthUser,
    Query(params): Query<DequeueMessagesParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let queue_service = crate::event_queues::EventQueueService::new();

    match queue_service
        .dequeue_messages(pool, &queue_name, params.limit)
        .await
    {
        Ok(messages) => {
            let responses: Vec<EventQueueMessageResponse> = messages
                .into_iter()
                .map(|m| EventQueueMessageResponse {
                    id: m.id.to_string(),
                    queue_id: m.queue_id.to_string(),
                    payload: m.payload,
                    status: m.status,
                    attempts: m.attempts,
                    max_attempts: m.max_attempts,
                    created_at: m.created_at.to_rfc3339(),
                    processed_at: m.processed_at.map(|t| t.to_rfc3339()),
                })
                .collect();
            (StatusCode::OK, Json(responses)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn complete_message(
    State(state): State<AppState>,
    Path((_queue_name, message_id)): Path<(String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let queue_service = crate::event_queues::EventQueueService::new();

    match queue_service.complete_message(pool, &message_id).await {
        Ok(true) => (
            StatusCode::OK,
            Json(json!({"status": "completed"})),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("message not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn fail_message(
    State(state): State<AppState>,
    Path((_queue_name, message_id)): Path<(String, String)>,
    _auth: AuthUser,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let queue_service = crate::event_queues::EventQueueService::new();
    let error = req
        .get("error")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown error");

    match queue_service.fail_message(pool, &message_id, error).await {
        Ok(true) => (
            StatusCode::OK,
            Json(json!({"status": "failed"})),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("message not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn retry_messages(
    State(state): State<AppState>,
    Path(queue_name): Path<String>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let queue_service = crate::event_queues::EventQueueService::new();

    match queue_service.retry_messages(pool, &queue_name).await {
        Ok(count) => (
            StatusCode::OK,
            Json(json!({"retried": count})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_queue_stats(
    State(state): State<AppState>,
    Path(queue_name): Path<String>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let queue_service = crate::event_queues::EventQueueService::new();

    match queue_service.get_queue_stats(pool, &queue_name).await {
        Ok(stats) => (
            StatusCode::OK,
            Json(QueueStatsResponse {
                queue_name: stats.queue_name,
                total_messages: stats.total_messages,
                pending_messages: stats.pending_messages,
                completed_messages: stats.completed_messages,
                failed_messages: stats.failed_messages,
                dead_letter_messages: stats.dead_letter_messages,
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

pub fn event_queue_routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route(
            "/api/v1/event-queues",
            post(create_event_queue),
        )
        .route(
            "/api/v1/event-queues/{queue_name}",
            get(get_event_queue),
        )
        .route(
            "/api/v1/event-queues/{queue_name}/messages",
            post(enqueue_message).get(dequeue_messages),
        )
        .route(
            "/api/v1/event-queues/{queue_name}/messages/{message_id}/complete",
            post(complete_message),
        )
        .route(
            "/api/v1/event-queues/{queue_name}/messages/{message_id}/fail",
            post(fail_message),
        )
        .route(
            "/api/v1/event-queues/{queue_name}/retry",
            post(retry_messages),
        )
        .route(
            "/api/v1/event-queues/{queue_name}/stats",
            get(get_queue_stats),
        )
}