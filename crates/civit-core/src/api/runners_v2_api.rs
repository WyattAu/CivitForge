//! Pipeline Runners v2 API endpoints.

#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use crate::pipeline_runners::{
    PipelineRunnersService, RecordMetricsRequest, RegisterRunnerRequest, UpdateRunnerRequest,
};
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListRunnersParams {
    pub status: Option<String>,
    pub tags: Option<String>,
}

pub fn runners_v2_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v2/runners", get(list_runners_v2).post(register_runner_v2))
        .route(
            "/api/v2/runners/{runner_id}",
            get(get_runner_v2).put(update_runner_v2).delete(delete_runner_v2),
        )
        .route("/api/v2/runners/{runner_id}/heartbeat", post(heartbeat_v2))
        .route(
            "/api/v2/runners/{runner_id}/assign/{job_id}",
            post(assign_job_v2),
        )
        .route("/api/v2/runners/{runner_id}/clear-job", post(clear_job_v2))
        .route(
            "/api/v2/runners/{runner_id}/metrics",
            get(get_metrics_v2).post(record_metrics_v2),
        )
        .route(
            "/api/v2/runners/available",
            get(find_available_runners_v2),
        )
        .route("/api/v2/runners/cleanup", post(cleanup_stale_runners_v2))
}

pub async fn list_runners_v2(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(params): Query<ListRunnersParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let service = PipelineRunnersService::new(pool.clone());

    let tags: Option<Vec<String>> = params.tags.map(|t| {
        t.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    });

    match service
        .list_runners(params.status.as_deref(), tags.as_deref())
        .await
    {
        Ok(runners) => (axum::http::StatusCode::OK, Json(runners)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn register_runner_v2(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(req): Json<RegisterRunnerRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let service = PipelineRunnersService::new(pool.clone());

    match service.register_runner(req).await {
        Ok(runner) => (axum::http::StatusCode::CREATED, Json(runner)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_runner_v2(
    State(state): State<AppState>,
    Path(runner_id): Path<String>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let id = match Uuid::parse_str(&runner_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid runner ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let service = PipelineRunnersService::new(pool.clone());
    match service.get_runner(id).await {
        Ok(Some(runner)) => (axum::http::StatusCode::OK, Json(runner)).into_response(),
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("runner not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_runner_v2(
    State(state): State<AppState>,
    Path(runner_id): Path<String>,
    _auth: AuthUser,
    Json(req): Json<UpdateRunnerRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let id = match Uuid::parse_str(&runner_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid runner ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let service = PipelineRunnersService::new(pool.clone());
    match service.update_runner(id, req).await {
        Ok(runner) => (axum::http::StatusCode::OK, Json(runner)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_runner_v2(
    State(state): State<AppState>,
    Path(runner_id): Path<String>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let id = match Uuid::parse_str(&runner_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid runner ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let service = PipelineRunnersService::new(pool.clone());
    match service.delete_runner(id).await {
        Ok(true) => (axum::http::StatusCode::NO_CONTENT, "").into_response(),
        Ok(false) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("runner not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn heartbeat_v2(
    State(state): State<AppState>,
    Path(runner_id): Path<String>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let id = match Uuid::parse_str(&runner_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid runner ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let service = PipelineRunnersService::new(pool.clone());
    match service.heartbeat(id).await {
        Ok(_) => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({"status": "ok"})),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn assign_job_v2(
    State(state): State<AppState>,
    Path((runner_id, job_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let runner_id = match Uuid::parse_str(&runner_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid runner ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let job_id = match Uuid::parse_str(&job_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid job ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let service = PipelineRunnersService::new(pool.clone());
    match service.assign_job(runner_id, job_id).await {
        Ok(_) => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({"assigned": true})),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn clear_job_v2(
    State(state): State<AppState>,
    Path(runner_id): Path<String>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let id = match Uuid::parse_str(&runner_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid runner ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let service = PipelineRunnersService::new(pool.clone());
    match service.clear_job(id).await {
        Ok(_) => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({"cleared": true})),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_metrics_v2(
    State(state): State<AppState>,
    Path(runner_id): Path<String>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let id = match Uuid::parse_str(&runner_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid runner ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let service = PipelineRunnersService::new(pool.clone());
    match service.get_runner_metrics(id, 100).await {
        Ok(metrics) => (axum::http::StatusCode::OK, Json(metrics)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn record_metrics_v2(
    State(state): State<AppState>,
    Path(runner_id): Path<String>,
    Json(req): Json<RecordMetricsRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let id = match Uuid::parse_str(&runner_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid runner ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let service = PipelineRunnersService::new(pool.clone());
    match service.record_metrics(id, req).await {
        Ok(metrics) => (axum::http::StatusCode::CREATED, Json(metrics)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn find_available_runners_v2(
    State(state): State<AppState>,
    Query(params): Query<ListRunnersParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let service = PipelineRunnersService::new(pool.clone());

    let required_tags: Vec<String> = params
        .tags
        .map(|t| {
            t.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    match service.find_available_runners(&required_tags).await {
        Ok(runners) => (axum::http::StatusCode::OK, Json(runners)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn cleanup_stale_runners_v2(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let service = PipelineRunnersService::new(pool.clone());

    match service.cleanup_stale_runners(5).await {
        Ok(count) => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({"cleaned": count})),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_runner_v2_request_deserialize() {
        let json = r#"{"name": "linux-runner", "description": "Linux build runner", "tags": ["linux", "amd64"]}"#;
        let req: RegisterRunnerRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "linux-runner");
        assert_eq!(req.tags.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_record_metrics_request_deserialize() {
        let json = r#"{"cpu_usage": 75.5, "memory_usage": 82.3, "disk_usage": 45.0}"#;
        let req: RecordMetricsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.cpu_usage, 75.5);
    }
}
