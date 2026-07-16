use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use uuid::Uuid;

use crate::api::AppState;
use crate::service_mesh::types::*;

pub fn service_mesh_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/service-mesh/services",
            get(list_services).post(create_service),
        )
        .route(
            "/api/v1/service-mesh/services/{id}",
            get(get_service).patch(update_service).delete(delete_service),
        )
        .route(
            "/api/v1/service-mesh/routes",
            get(list_routes).post(create_route),
        )
        .route(
            "/api/v1/service-mesh/routes/{id}",
            get(get_route).patch(update_route).delete(delete_route),
        )
}

async fn create_service(
    State(state): State<AppState>,
    Json(req): Json<CreateServiceRequest>,
) -> impl IntoResponse {
    let store = crate::service_mesh::ServiceMeshStore::new(state.db.pool().clone());
    match store.create_service(req).await {
        Ok(service) => (StatusCode::CREATED, Json(service)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn get_service(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let store = crate::service_mesh::ServiceMeshStore::new(state.db.pool().clone());
    match store.get_service(id).await {
        Ok(Some(service)) => Json(service).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn list_services(State(state): State<AppState>) -> impl IntoResponse {
    let store = crate::service_mesh::ServiceMeshStore::new(state.db.pool().clone());
    match store.list_services(100, 0).await {
        Ok(services) => Json(services).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn update_service(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateServiceRequest>,
) -> impl IntoResponse {
    let store = crate::service_mesh::ServiceMeshStore::new(state.db.pool().clone());
    match store.update_service(id, req).await {
        Ok(service) => Json(service).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn delete_service(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let store = crate::service_mesh::ServiceMeshStore::new(state.db.pool().clone());
    match store.delete_service(id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn create_route(
    State(state): State<AppState>,
    Json(req): Json<CreateRouteRequest>,
) -> impl IntoResponse {
    let store = crate::service_mesh::ServiceMeshStore::new(state.db.pool().clone());
    match store.create_route(req).await {
        Ok(route) => (StatusCode::CREATED, Json(route)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn get_route(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let store = crate::service_mesh::ServiceMeshStore::new(state.db.pool().clone());
    match store.get_route(id).await {
        Ok(Some(route)) => Json(route).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn list_routes(State(state): State<AppState>) -> impl IntoResponse {
    let store = crate::service_mesh::ServiceMeshStore::new(state.db.pool().clone());
    match store.list_routes(100, 0).await {
        Ok(routes) => Json(routes).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn update_route(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRouteRequest>,
) -> impl IntoResponse {
    let store = crate::service_mesh::ServiceMeshStore::new(state.db.pool().clone());
    match store.update_route(id, req).await {
        Ok(route) => Json(route).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn delete_route(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let store = crate::service_mesh::ServiceMeshStore::new(state.db.pool().clone());
    match store.delete_route(id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
