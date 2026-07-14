use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use uuid::Uuid;

use crate::api::AppState;
use crate::chaos::types::*;

pub fn chaos_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/chaos/experiments", get(list_experiments).post(create_experiment))
        .route(
            "/api/v1/chaos/experiments/{id}",
            get(get_experiment).delete(delete_experiment),
        )
        .route("/api/v1/chaos/experiments/{id}/start", post(start_experiment))
        .route(
            "/api/v1/chaos/experiments/{id}/complete",
            post(complete_experiment),
        )
        .route(
            "/api/v1/chaos/experiments/{id}/cancel",
            post(cancel_experiment),
        )
        .route(
            "/api/v1/chaos/experiments/{id}/results",
            get(get_results).post(record_result),
        )
}

async fn create_experiment(
    State(state): State<AppState>,
    Json(req): Json<CreateExperimentRequest>,
) -> impl IntoResponse {
    let engine = crate::chaos::ChaosEngine::new(state.db.pool().clone());
    match engine.create_experiment(req).await {
        Ok(experiment) => (StatusCode::CREATED, Json(experiment)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn get_experiment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = crate::chaos::ChaosEngine::new(state.db.pool().clone());
    match engine.get_experiment(id).await {
        Ok(Some(experiment)) => Json(experiment).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn list_experiments(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let engine = crate::chaos::ChaosEngine::new(state.db.pool().clone());
    match engine.list_experiments(None, 100, 0).await {
        Ok(experiments) => Json(experiments).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn delete_experiment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = crate::chaos::ChaosEngine::new(state.db.pool().clone());
    match engine.delete_experiment(id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn start_experiment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = crate::chaos::ChaosEngine::new(state.db.pool().clone());
    match engine.start_experiment(id).await {
        Ok(experiment) => Json(experiment).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn complete_experiment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = crate::chaos::ChaosEngine::new(state.db.pool().clone());
    match engine.complete_experiment(id, true).await {
        Ok(experiment) => Json(experiment).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn cancel_experiment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = crate::chaos::ChaosEngine::new(state.db.pool().clone());
    match engine.cancel_experiment(id).await {
        Ok(experiment) => Json(experiment).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

async fn get_results(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = crate::chaos::ChaosEngine::new(state.db.pool().clone());
    match engine.get_results(id).await {
        Ok(results) => Json(results).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

#[derive(serde::Deserialize)]
struct RecordResultRequest {
    metric_name: String,
    metric_value: f64,
    baseline_value: Option<f64>,
    impact: ImpactLevel,
}

async fn record_result(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<RecordResultRequest>,
) -> impl IntoResponse {
    let engine = crate::chaos::ChaosEngine::new(state.db.pool().clone());
    match engine
        .record_result(
            id,
            req.metric_name,
            req.metric_value,
            req.baseline_value,
            req.impact,
        )
        .await
    {
        Ok(result) => (StatusCode::CREATED, Json(result)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}
