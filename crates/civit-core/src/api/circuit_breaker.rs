use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use crate::api::AppState;
use crate::circuit_breaker::types::*;

pub fn circuit_breaker_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/circuit-breakers", get(list_breakers).post(create_breaker))
        .route(
            "/api/v1/circuit-breakers/{name}",
            get(get_breaker).delete(delete_breaker),
        )
        .route(
            "/api/v1/circuit-breakers/{name}/success",
            post(record_success),
        )
        .route(
            "/api/v1/circuit-breakers/{name}/failure",
            post(record_failure),
        )
        .route(
            "/api/v1/circuit-breakers/{name}/can-execute",
            get(can_execute),
        )
        .route(
            "/api/v1/circuit-breakers/{name}/reset",
            post(reset_breaker),
        )
}

async fn create_breaker(
    State(state): State<AppState>,
    Json(config): Json<CircuitBreakerConfig>,
) -> impl IntoResponse {
    let breaker = crate::circuit_breaker::CircuitBreaker::new(state.db.pool().clone());
    match breaker.create(config).await {
        Ok(metrics) => (StatusCode::CREATED, Json(metrics)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn get_breaker(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let breaker = crate::circuit_breaker::CircuitBreaker::new(state.db.pool().clone());
    match breaker.get(&name).await {
        Ok(Some(metrics)) => Json(metrics).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn list_breakers(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let breaker = crate::circuit_breaker::CircuitBreaker::new(state.db.pool().clone());
    match breaker.list().await {
        Ok(breakers) => Json(breakers).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn delete_breaker(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let breaker = crate::circuit_breaker::CircuitBreaker::new(state.db.pool().clone());
    match breaker.delete(&name).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn record_success(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let breaker = crate::circuit_breaker::CircuitBreaker::new(state.db.pool().clone());
    match breaker.record_success(&name).await {
        Ok(metrics) => Json(metrics).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn record_failure(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let breaker = crate::circuit_breaker::CircuitBreaker::new(state.db.pool().clone());
    match breaker.record_failure(&name).await {
        Ok(metrics) => Json(metrics).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn can_execute(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let breaker = crate::circuit_breaker::CircuitBreaker::new(state.db.pool().clone());
    match breaker.can_execute(&name).await {
        Ok(can) => Json(serde_json::json!({"can_execute": can})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn reset_breaker(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let breaker = crate::circuit_breaker::CircuitBreaker::new(state.db.pool().clone());
    match breaker.reset(&name).await {
        Ok(metrics) => Json(metrics).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}
