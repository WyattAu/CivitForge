#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct EventResponse {
    pub id: String,
    pub event_type: String,
    pub resource_type: String,
    pub resource_id: String,
    pub actor_id: Option<String>,
    pub payload: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateEventRequest {
    pub event_type: String,
    pub resource_type: String,
    pub resource_id: String,
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListEventsParams {
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, Serialize)]
pub struct EventSubscriptionResponse {
    pub id: String,
    pub user_id: String,
    pub event_type: String,
    pub callback_url: Option<String>,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateEventSubscriptionRequest {
    pub event_type: String,
    pub callback_url: Option<String>,
}

pub async fn publish_event(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(req): Json<CreateEventRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let event_system = crate::events::EventPublisher::new();

    match event_system
        .publish_event(
            pool,
            &req.event_type,
            &req.resource_type,
            &req.resource_id,
            None,
            req.payload.unwrap_or_else(|| json!({})),
        )
        .await
    {
        Ok(event) => (
            StatusCode::CREATED,
            Json(EventResponse {
                id: event.id.to_string(),
                event_type: event.event_type,
                resource_type: event.resource_type,
                resource_id: event.resource_id.to_string(),
                actor_id: event.actor_id.map(|id| id.to_string()),
                payload: event.payload,
                created_at: event.created_at.to_rfc3339(),
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

pub async fn list_events(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<ListEventsParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let event_system = crate::events::EventPublisher::new();

    let resource_type = params.resource_type.unwrap_or_default();
    let resource_id = params.resource_id.unwrap_or_default();

    match event_system
        .get_event_history(pool, &resource_type, &resource_id, params.limit)
        .await
    {
        Ok(events) => {
            let responses: Vec<EventResponse> = events
                .into_iter()
                .map(|e| EventResponse {
                    id: e.id.to_string(),
                    event_type: e.event_type,
                    resource_type: e.resource_type,
                    resource_id: e.resource_id.to_string(),
                    actor_id: e.actor_id.map(|id| id.to_string()),
                    payload: e.payload,
                    created_at: e.created_at.to_rfc3339(),
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

pub async fn get_event(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let _event_system = crate::events::EventPublisher::new();

    let event_uuid = match Uuid::parse_str(&event_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid event id".into()).error_response()),
            )
                .into_response();
        }
    };

    match sqlx::query_as::<_, crate::events::PublishedEvent>(
        "SELECT id, event_type, resource_type, resource_id, actor_id, payload, created_at \
         FROM events WHERE id = $1",
    )
    .bind(event_uuid)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(event)) => (
            StatusCode::OK,
            Json(EventResponse {
                id: event.id.to_string(),
                event_type: event.event_type,
                resource_type: event.resource_type,
                resource_id: event.resource_id.to_string(),
                actor_id: event.actor_id.map(|id| id.to_string()),
                payload: event.payload,
                created_at: event.created_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("event not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn replay_event(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let event_system = crate::events::EventPublisher::new();

    match event_system.replay_event(pool, &event_id).await {
        Ok(event) => (
            StatusCode::OK,
            Json(EventResponse {
                id: event.id.to_string(),
                event_type: event.event_type,
                resource_type: event.resource_type,
                resource_id: event.resource_id.to_string(),
                actor_id: event.actor_id.map(|id| id.to_string()),
                payload: event.payload,
                created_at: event.created_at.to_rfc3339(),
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

pub async fn create_event_subscription(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateEventSubscriptionRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let event_system = crate::events::EventPublisher::new();

    match event_system
        .create_subscription(pool, &auth.user_id, &req.event_type, req.callback_url.as_deref())
        .await
    {
        Ok(subscription) => (
            StatusCode::CREATED,
            Json(EventSubscriptionResponse {
                id: subscription.id.to_string(),
                user_id: subscription.user_id.to_string(),
                event_type: subscription.event_type,
                callback_url: subscription.callback_url,
                enabled: subscription.enabled,
                created_at: subscription.created_at.to_rfc3339(),
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

pub async fn list_event_subscriptions(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let event_system = crate::events::EventPublisher::new();

    match event_system.list_subscriptions(pool, &auth.user_id).await {
        Ok(subscriptions) => {
            let responses: Vec<EventSubscriptionResponse> = subscriptions
                .into_iter()
                .map(|s| EventSubscriptionResponse {
                    id: s.id.to_string(),
                    user_id: s.user_id.to_string(),
                    event_type: s.event_type,
                    callback_url: s.callback_url,
                    enabled: s.enabled,
                    created_at: s.created_at.to_rfc3339(),
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

pub async fn delete_event_subscription(
    State(state): State<AppState>,
    Path(subscription_id): Path<String>,
    auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let event_system = crate::events::EventPublisher::new();

    match event_system
        .delete_subscription(pool, &subscription_id, &auth.user_id)
        .await
    {
        Ok(true) => (
            StatusCode::OK,
            Json(json!({"status": "deleted"})),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("subscription not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub fn event_routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/api/v1/events", post(publish_event).get(list_events))
        .route("/api/v1/events/{event_id}", get(get_event))
        .route("/api/v1/events/{event_id}/replay", post(replay_event))
        .route(
            "/api/v1/event-subscriptions",
            post(create_event_subscription).get(list_event_subscriptions),
        )
        .route(
            "/api/v1/event-subscriptions/{subscription_id}",
            delete(delete_event_subscription),
        )
}