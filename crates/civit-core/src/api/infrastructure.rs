use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::api::AppState;
use crate::infrastructure::types::*;

pub fn infrastructure_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/infrastructure/templates",
            get(list_templates).post(create_template),
        )
        .route(
            "/api/v1/infrastructure/templates/{id}",
            get(get_template).patch(update_template).delete(delete_template),
        )
        .route(
            "/api/v1/infrastructure/templates/{id}/deploy",
            post(deploy_template),
        )
        .route(
            "/api/v1/infrastructure/deployments/{id}",
            get(get_deployment),
        )
        .route(
            "/api/v1/infrastructure/deployments/{id}/complete",
            post(complete_deployment),
        )
}

async fn create_template(
    State(state): State<AppState>,
    Json(req): Json<CreateTemplateRequest>,
) -> impl IntoResponse {
    let store = crate::infrastructure::InfrastructureStore::new(state.db.pool().clone());
    match store.create_template(req).await {
        Ok(template) => (StatusCode::CREATED, Json(template)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn get_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let store = crate::infrastructure::InfrastructureStore::new(state.db.pool().clone());
    match store.get_template(id).await {
        Ok(Some(template)) => Json(template).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn list_templates(State(state): State<AppState>) -> impl IntoResponse {
    let store = crate::infrastructure::InfrastructureStore::new(state.db.pool().clone());
    match store.list_templates(100, 0).await {
        Ok(templates) => Json(templates).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn update_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTemplateRequest>,
) -> impl IntoResponse {
    let store = crate::infrastructure::InfrastructureStore::new(state.db.pool().clone());
    match store.update_template(id, req).await {
        Ok(template) => Json(template).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn delete_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let store = crate::infrastructure::InfrastructureStore::new(state.db.pool().clone());
    match store.delete_template(id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn deploy_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<DeployRequest>,
) -> impl IntoResponse {
    let store = crate::infrastructure::InfrastructureStore::new(state.db.pool().clone());
    match store.deploy(id, req).await {
        Ok(deployment) => (StatusCode::CREATED, Json(deployment)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn get_deployment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let store = crate::infrastructure::InfrastructureStore::new(state.db.pool().clone());
    match store.get_deployment(id).await {
        Ok(Some(deployment)) => Json(deployment).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn complete_deployment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let store = crate::infrastructure::InfrastructureStore::new(state.db.pool().clone());
    match store.complete_deployment(id, true).await {
        Ok(deployment) => Json(deployment).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
