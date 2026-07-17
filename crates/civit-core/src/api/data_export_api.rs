#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::data_portability::{
    DataPortabilityService, ExportFormat, ExportRequest, ImportRequest,
};
use crate::error::CoreError;
use axum::{
    Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct StartExportRequest {
    pub organization_id: Uuid,
    pub format: String,
    pub data_types: Vec<String>,
    pub repo_ids: Option<Vec<Uuid>>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportJobResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub format: String,
    pub status: String,
    pub data_types: Vec<String>,
    pub file_size_bytes: i64,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StartImportRequest {
    pub source: String,
    pub source_url: String,
    pub api_token: Option<String>,
    pub organization_id: Uuid,
    pub repo_mapping: Option<serde_json::Value>,
    pub conflict_resolution: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportJobResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub source: String,
    pub status: String,
    pub items_imported: i64,
    pub items_skipped: i64,
    pub items_failed: i64,
    pub created_at: String,
    pub completed_at: Option<String>,
}

pub fn data_export_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/data/export", post(start_export))
        .route("/api/v1/data/export/{id}", get(get_export_status))
        .route("/api/v1/data/export/{id}/download", get(download_export))
        .route("/api/v1/data/import", post(start_import))
        .route("/api/v1/data/import/{id}", get(get_import_status))
}

pub async fn start_export(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<StartExportRequest>,
) -> Response {
    let valid_formats = ["json", "csv", "git_archive"];
    if !valid_formats.contains(&req.format.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                CoreError::BadRequest(format!(
                    "invalid format: {}, must be one of: {}",
                    req.format,
                    valid_formats.join(", ")
                ))
                .error_response(),
            ),
        )
            .into_response();
    }

    let format: ExportFormat = match req.format.parse() {
        Ok(f) => f,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest(e).error_response()),
            )
                .into_response();
        }
    };

    let pool = state.db.pool().clone();
    let svc = DataPortabilityService::new(pool);

    let user_id = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => Uuid::nil(),
    };

    let date_from = req
        .date_from
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let date_to = req
        .date_to
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let export_req = ExportRequest {
        organization_id: req.organization_id,
        format,
        data_types: req.data_types.clone(),
        repo_ids: req.repo_ids,
        date_from,
        date_to,
    };

    match svc.create_export_job(&export_req, user_id).await {
        Ok(job) => {
            let response = ExportJobResponse {
                id: job.id,
                organization_id: job.organization_id,
                format: job.format,
                status: job.status,
                data_types: req.data_types,
                file_size_bytes: job.file_size_bytes,
                started_at: job.created_at.to_rfc3339(),
                completed_at: job.completed_at.map(|dt| dt.to_rfc3339()),
            };
            (StatusCode::ACCEPTED, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Internal(format!("failed to create export job: {e}")).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_export_status(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Response {
    let pool = state.db.pool().clone();
    let svc = DataPortabilityService::new(pool);

    match svc.get_export_job(id).await {
        Ok(Some(job)) => {
            let response = ExportJobResponse {
                id: job.id,
                organization_id: job.organization_id,
                format: job.format,
                status: job.status,
                data_types: serde_json::from_value(
                    serde_json::to_value(&job.data_types).unwrap_or_default(),
                )
                .unwrap_or_default(),
                file_size_bytes: job.file_size_bytes,
                started_at: job.created_at.to_rfc3339(),
                completed_at: job.completed_at.map(|dt| dt.to_rfc3339()),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("export job not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Internal(format!("database error: {e}")).error_response()),
        )
            .into_response(),
    }
}

pub async fn download_export(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Response {
    let pool = state.db.pool().clone();
    let svc = DataPortabilityService::new(pool);

    match svc.get_export_job(id).await {
        Ok(Some(job)) if job.status == "completed" => {
            if let Some(ref file_path) = job.file_path {
                match std::fs::read(file_path) {
                    Ok(data) => {
                        let mut headers = HeaderMap::new();
                        let content_type = match job.format.as_str() {
                            "csv" => "text/csv",
                            _ => "application/json",
                        };
                        headers.insert(
                            axum::http::header::CONTENT_TYPE,
                            HeaderValue::from_static(content_type),
                        );
                        headers.insert(
                            axum::http::header::CONTENT_DISPOSITION,
                            HeaderValue::from_str(&format!(
                                "attachment; filename=\"export-{}.{}`",
                                job.id, job.format
                            ))
                            .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
                        );
                        (headers, data).into_response()
                    }
                    Err(_) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(CoreError::Internal("failed to read export file".into()).error_response()),
                    )
                        .into_response(),
                }
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(CoreError::NotFound("export file not available".into()).error_response()),
                )
                    .into_response()
            }
        }
        Ok(Some(_)) => (
            StatusCode::CONFLICT,
            Json(CoreError::BadRequest("export job is not completed".into()).error_response()),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("export job not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Internal(format!("database error: {e}")).error_response()),
        )
            .into_response(),
    }
}

pub async fn start_import(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<StartImportRequest>,
) -> Response {
    let valid_sources = ["github", "gitlab", "gitea"];
    if !valid_sources.contains(&req.source.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                CoreError::BadRequest(format!(
                    "invalid source: {}, must be one of: {}",
                    req.source,
                    valid_sources.join(", ")
                ))
                .error_response(),
            ),
        )
            .into_response();
    }

    let pool = state.db.pool().clone();
    let svc = DataPortabilityService::new(pool);

    let user_id = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => Uuid::nil(),
    };

    let import_req = ImportRequest {
        source: req.source.clone(),
        source_url: req.source_url.clone(),
        api_token: req.api_token,
        organization_id: req.organization_id,
        repo_mapping: req.repo_mapping,
        conflict_resolution: req.conflict_resolution,
    };

    match svc.create_import_job(&import_req, user_id).await {
        Ok(job) => {
            let response = ImportJobResponse {
                id: job.id,
                organization_id: job.organization_id,
                source: job.source,
                status: job.status,
                items_imported: job.items_imported,
                items_skipped: job.items_skipped,
                items_failed: job.items_failed,
                created_at: job.created_at.to_rfc3339(),
                completed_at: job.completed_at.map(|dt| dt.to_rfc3339()),
            };
            (StatusCode::ACCEPTED, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Internal(format!("failed to create import job: {e}")).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_import_status(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Response {
    let pool = state.db.pool().clone();
    let svc = DataPortabilityService::new(pool);

    match svc.get_import_job(id).await {
        Ok(Some(job)) => {
            let response = ImportJobResponse {
                id: job.id,
                organization_id: job.organization_id,
                source: job.source,
                status: job.status,
                items_imported: job.items_imported,
                items_skipped: job.items_skipped,
                items_failed: job.items_failed,
                created_at: job.created_at.to_rfc3339(),
                completed_at: job.completed_at.map(|dt| dt.to_rfc3339()),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("import job not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Internal(format!("database error: {e}")).error_response()),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_export_request_deserialization() {
        let json = r#"{
            "organization_id": "00000000-0000-0000-0000-000000000001",
            "format": "json",
            "data_types": ["repos", "issues"]
        }"#;
        let req: StartExportRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.format, "json");
        assert_eq!(req.data_types.len(), 2);
    }

    #[test]
    fn test_export_job_response_serialization() {
        let response = ExportJobResponse {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            format: "json".into(),
            status: "completed".into(),
            data_types: vec!["repos".into()],
            file_size_bytes: 2048,
            started_at: Utc::now().to_rfc3339(),
            completed_at: Some(Utc::now().to_rfc3339()),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"format\":\"json\""));
        assert!(json.contains("\"file_size_bytes\":2048"));
    }

    #[test]
    fn test_start_import_request_deserialization() {
        let json = r#"{
            "source": "github",
            "source_url": "https://github.com/org/repo",
            "organization_id": "00000000-0000-0000-0000-000000000001",
            "conflict_resolution": "skip"
        }"#;
        let req: StartImportRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.source, "github");
        assert_eq!(req.conflict_resolution, "skip");
    }

    #[test]
    fn test_import_job_response_serialization() {
        let response = ImportJobResponse {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            source: "gitlab".into(),
            status: "completed".into(),
            items_imported: 42,
            items_skipped: 3,
            items_failed: 1,
            created_at: Utc::now().to_rfc3339(),
            completed_at: Some(Utc::now().to_rfc3339()),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"items_imported\":42"));
        assert!(json.contains("\"items_failed\":1"));
    }

    #[test]
    fn test_data_export_routes_compile() {
        let router = data_export_routes();
        let _ = router;
    }
}
