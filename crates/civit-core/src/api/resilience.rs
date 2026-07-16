use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::api::AppState;
use crate::resilience::types::*;

pub fn resilience_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/resilience/tests", get(list_tests).post(create_test))
        .route(
            "/api/v1/resilience/tests/{id}",
            get(get_test).delete(delete_test),
        )
        .route("/api/v1/resilience/tests/{id}/start", post(start_test))
        .route(
            "/api/v1/resilience/tests/{id}/complete",
            post(complete_test),
        )
        .route("/api/v1/resilience/score", get(get_resilience_score))
}

async fn create_test(
    State(state): State<AppState>,
    Json(req): Json<CreateTestRequest>,
) -> impl IntoResponse {
    let tester = crate::resilience::ResilienceTester::new(state.db.pool().clone());
    match tester.create_test(req).await {
        Ok(test) => (StatusCode::CREATED, Json(test)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn get_test(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tester = crate::resilience::ResilienceTester::new(state.db.pool().clone());
    match tester.get_test(id).await {
        Ok(Some(test)) => Json(test).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn list_tests(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let tester = crate::resilience::ResilienceTester::new(state.db.pool().clone());
    match tester.list_tests(None, 100, 0).await {
        Ok(tests) => Json(tests).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn delete_test(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tester = crate::resilience::ResilienceTester::new(state.db.pool().clone());
    match tester.delete_test(id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn start_test(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tester = crate::resilience::ResilienceTester::new(state.db.pool().clone());
    match tester.start_test(id).await {
        Ok(test) => Json(test).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn complete_test(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tester = crate::resilience::ResilienceTester::new(state.db.pool().clone());
    match tester.complete_test(id, 85, true).await {
        Ok(test) => Json(test).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn get_resilience_score(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let tester = crate::resilience::ResilienceTester::new(state.db.pool().clone());
    match tester.get_resilience_score().await {
        Ok(score) => Json(score).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}
