//! Pipeline Actions Marketplace API endpoints.

#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use civit_ci::actions;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateActionRequest {
    pub name: String,
    pub description: Option<String>,
    pub action_type: String,
    pub config: Option<serde_json::Value>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateActionRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub action_type: Option<String>,
    pub config: Option<serde_json::Value>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ActionListParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    pub action_type: Option<String>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
}

fn default_page() -> u32 {
    1
}
fn default_per_page() -> u32 {
    20
}

#[derive(Debug, Deserialize)]
pub struct RateActionRequest {
    pub rating: f64,
}

pub fn pipeline_action_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/pipeline-actions",
            get(list_actions).post(create_action),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}",
            get(get_action)
                .patch(update_action)
                .delete(delete_action),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/download",
            post(record_download),
        )
        .route(
            "/api/v1/pipeline-actions/{action_id}/rate",
            post(rate_action),
        )
}

pub async fn list_actions(
    State(state): State<AppState>,
    Query(params): Query<ActionListParams>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let offset = ((params.page.saturating_sub(1)) * params.per_page) as i64;

    match actions::list_pipeline_actions(
        pool,
        params.per_page as i64,
        offset,
        params.action_type.as_deref(),
        params.search.as_deref(),
        params.sort_by.as_deref(),
    )
    .await
    {
        Ok(actions_list) => (StatusCode::OK, Json(actions_list)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn create_action(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateActionRequest>,
) -> Response {
    let pool = state.db.pool();
    let author_id = Uuid::parse_str(&auth.user_id).ok();
    let config = req.config.unwrap_or(serde_json::json!({}));
    let version = req.version.as_deref().unwrap_or("1.0.0");
    let description = req.description.unwrap_or_default();

    match actions::create_pipeline_action(
        pool,
        &req.name,
        &description,
        &req.action_type,
        &config,
        version,
        author_id,
    )
    .await
    {
        Ok(action) => (StatusCode::CREATED, Json(action)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_action(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::get_pipeline_action(pool, aid).await {
        Ok(Some(action)) => (StatusCode::OK, Json(action)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("action not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_action(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    _auth: AuthUser,
    Json(req): Json<UpdateActionRequest>,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let config_ref = req.config.as_ref();

    match actions::update_pipeline_action(
        pool,
        aid,
        req.name.as_deref(),
        req.description.as_deref(),
        req.action_type.as_deref(),
        config_ref,
        req.version.as_deref(),
    )
    .await
    {
        Ok(action) => (StatusCode::OK, Json(action)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_action(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::delete_pipeline_action(pool, aid).await {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "deleted"})),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("action not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn record_download(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match actions::track_download(pool, aid).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "download recorded"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn rate_action(
    State(state): State<AppState>,
    Path(action_id): Path<String>,
    _auth: AuthUser,
    Json(req): Json<RateActionRequest>,
) -> Response {
    let pool = state.db.pool();
    let aid = match Uuid::parse_str(&action_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid action ID".into()).error_response()),
            )
                .into_response();
        }
    };

    if req.rating < 0.0 || req.rating > 5.0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::BadRequest("rating must be between 0 and 5".into()).error_response()),
        )
            .into_response();
    }

    match actions::update_rating(pool, aid, req.rating).await {
        Ok(action) => (StatusCode::OK, Json(action)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_action_request() {
        let json = r#"{"name": "docker-build", "action_type": "docker", "description": "Build Docker images"}"#;
        let req: CreateActionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "docker-build");
        assert_eq!(req.action_type, "docker");
    }

    #[test]
    fn test_update_action_request() {
        let json = r#"{"name": "updated-name"}"#;
        let req: UpdateActionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name.as_deref(), Some("updated-name"));
    }

    #[test]
    fn test_list_params_defaults() {
        let json = r#"{}"#;
        let params: ActionListParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.page, 1);
        assert_eq!(params.per_page, 20);
    }

    #[test]
    fn test_rate_action_request() {
        let json = r#"{"rating": 4.5}"#;
        let req: RateActionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.rating, 4.5);
    }

    #[test]
    fn test_pipeline_action_routes_compile() {
        let _ = pipeline_action_routes();
    }
}
