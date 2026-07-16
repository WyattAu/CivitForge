use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use uuid::Uuid;

use crate::api::AppState;
use crate::deployment_strategy::types::*;

pub fn deployment_strategy_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/deployment-strategies",
            get(list_strategies).post(create_strategy),
        )
        .route(
            "/api/v1/deployment-strategies/{id}",
            get(get_strategy).patch(update_strategy).delete(delete_strategy),
        )
}

async fn create_strategy(
    State(state): State<AppState>,
    Json(req): Json<CreateStrategyRequest>,
) -> impl IntoResponse {
    let store = crate::deployment_strategy::DeploymentStrategyStore::new(state.db.pool().clone());
    match store.create(req).await {
        Ok(strategy) => (StatusCode::CREATED, Json(strategy)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn get_strategy(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let store = crate::deployment_strategy::DeploymentStrategyStore::new(state.db.pool().clone());
    match store.get(id).await {
        Ok(Some(strategy)) => Json(strategy).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn list_strategies(State(state): State<AppState>) -> impl IntoResponse {
    let store = crate::deployment_strategy::DeploymentStrategyStore::new(state.db.pool().clone());
    match store.list_by_repo(Uuid::nil(), 100, 0).await {
        Ok(strategies) => Json(strategies).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn update_strategy(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateStrategyRequest>,
) -> impl IntoResponse {
    let store = crate::deployment_strategy::DeploymentStrategyStore::new(state.db.pool().clone());
    match store.update(id, req).await {
        Ok(strategy) => Json(strategy).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn delete_strategy(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let store = crate::deployment_strategy::DeploymentStrategyStore::new(state.db.pool().clone());
    match store.delete(id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
