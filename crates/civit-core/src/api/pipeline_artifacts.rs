//! Pipeline Artifacts API endpoints.

#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, OptionalAuthUser};
use crate::error::CoreError;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use civit_ci::pipeline;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct CreateArtifactRequest {
    pub job_id: String,
    pub name: String,
    pub path: String,
    pub size_bytes: i64,
    #[serde(default = "default_content_type")]
    pub content_type: String,
}

fn default_content_type() -> String {
    "application/octet-stream".to_string()
}

async fn resolve_repo_id(
    pool: &sqlx::PgPool,
    owner: &str,
    repo_name: &str,
) -> std::result::Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_as::<_, (Uuid,)>(
        "SELECT r.id FROM repositories r JOIN users u ON r.owner_id = u.id WHERE u.username = $1 AND r.name = $2",
    )
    .bind(owner)
    .bind(repo_name)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}

pub async fn upload_artifact(
    State(state): State<AppState>,
    Path((owner, repo_name, run_id)): Path<(String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<CreateArtifactRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();

    let _repo_id = match resolve_repo_id(pool, &owner, &repo_name).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let run_uuid = match Uuid::parse_str(&run_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid run ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let job_uuid = match Uuid::parse_str(&req.job_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid job ID".into()).error_response()),
            )
                .into_response();
        }
    };

    // Generate storage key
    let storage_key = format!(
        "{}/{}/artifacts/{}",
        run_id, req.job_id, req.name
    );

    match pipeline::create_artifact(
        pool,
        run_uuid,
        job_uuid,
        &req.name,
        &req.path,
        req.size_bytes,
        &req.content_type,
        &storage_key,
    )
    .await
    {
        Ok(artifact) => (StatusCode::CREATED, Json(artifact)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_artifacts(
    State(state): State<AppState>,
    Path((owner, repo_name, run_id)): Path<(String, String, String)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();

    let _repo_id = match resolve_repo_id(pool, &owner, &repo_name).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let run_uuid = match Uuid::parse_str(&run_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid run ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match pipeline::list_artifacts(pool, run_uuid).await {
        Ok(artifacts) => (StatusCode::OK, Json(artifacts)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn download_artifact(
    State(state): State<AppState>,
    Path((owner, repo_name, run_id, artifact_id)): Path<(String, String, String, String)>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();

    let _repo_id = match resolve_repo_id(pool, &owner, &repo_name).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let artifact_uuid = match Uuid::parse_str(&artifact_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid artifact ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match pipeline::get_artifact(pool, artifact_uuid).await {
        Ok(Some(artifact)) => {
            // In a real implementation, this would fetch from object storage
            // For now, return the artifact metadata with a placeholder download URL
            let response = serde_json::json!({
                "id": artifact.id,
                "name": artifact.name,
                "path": artifact.path,
                "size_bytes": artifact.size_bytes,
                "content_type": artifact.content_type,
                "storage_key": artifact.storage_key,
                "download_url": format!("/api/v1/repos/{}/{}/pipelines/{}/artifacts/{}/raw", owner, repo_name, run_id, artifact_id)
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("artifact not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_artifact(
    State(state): State<AppState>,
    Path((owner, repo_name, run_id, artifact_id)): Path<(String, String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();

    let _repo_id = match resolve_repo_id(pool, &owner, &repo_name).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let artifact_uuid = match Uuid::parse_str(&artifact_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid artifact ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match pipeline::delete_artifact(pool, artifact_uuid).await {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "deleted"})),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("artifact not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub fn artifact_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{repo}/pipelines/{run_id}/artifacts",
            post(upload_artifact).get(list_artifacts),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/pipelines/{run_id}/artifacts/{artifact_id}/download",
            get(download_artifact),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/pipelines/{run_id}/artifacts/{artifact_id}",
            delete(delete_artifact),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_artifact_request_deserialize() {
        let json = r#"{"job_id": "abc123", "name": "build-output", "path": "target/debug", "size_bytes": 1024}"#;
        let req: CreateArtifactRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.job_id, "abc123");
        assert_eq!(req.name, "build-output");
        assert_eq!(req.size_bytes, 1024);
        assert_eq!(req.content_type, "application/octet-stream");
    }

    #[test]
    fn test_create_artifact_request_custom_content_type() {
        let json = r#"{"job_id": "abc123", "name": "report.json", "path": "report.json", "size_bytes": 512, "content_type": "application/json"}"#;
        let req: CreateArtifactRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.content_type, "application/json");
    }
}
