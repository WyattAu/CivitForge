#![forbid(unsafe_code)]

//! Data export/import routes for exporting and importing repository data.

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_admin};
use crate::error::CoreError;
use axum::{
    Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ExportJob {
    pub id: Uuid,
    pub user_id: Uuid,
    pub export_type: String,
    pub status: String,
    pub file_path: Option<String>,
    pub file_size_bytes: i64,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateExportRequest {
    pub export_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportJobResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub export_type: String,
    pub status: String,
    pub file_size_bytes: i64,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportDataRequest {
    pub data: serde_json::Value,
    pub import_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportDataResponse {
    pub status: String,
    pub message: String,
    pub imported_count: usize,
}

pub fn export_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/export", post(start_export))
        .route("/api/v1/export/{id}", get(get_export_status))
        .route("/api/v1/export/{id}/download", get(download_export))
        .route("/api/v1/import", post(import_data))
}

pub async fn start_export(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateExportRequest>,
) -> Response {
    let valid_types = ["repos", "issues", "pull_requests", "wiki", "users", "full"];
    if !valid_types.contains(&req.export_type.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                CoreError::BadRequest(format!(
                    "invalid export_type: {}, must be one of: {}",
                    req.export_type,
                    valid_types.join(", ")
                ))
                .error_response(),
            ),
        )
            .into_response();
    }

    if req.export_type == "users"
        && let Err(rejection) = require_admin(&auth) {
            return rejection.into_response();
        }

    let user_id = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => Uuid::nil(),
    };

    let job_id = Uuid::new_v4();
    let result = sqlx::query(
        r#"INSERT INTO export_jobs (id, user_id, export_type, status)
           VALUES ($1, $2, $3, 'pending')"#,
    )
    .bind(job_id)
    .bind(user_id)
    .bind(&req.export_type)
    .execute(state.db.pool())
    .await;

    match result {
        Ok(_) => {
            let db_clone = state.db.clone();
            let export_type = req.export_type.clone();

            tokio::spawn(async move {
                let _ = execute_export_job(db_clone, job_id, &export_type).await;
            });

            (
                StatusCode::ACCEPTED,
                Json(ExportJobResponse {
                    id: job_id,
                    user_id,
                    export_type: req.export_type,
                    status: "pending".into(),
                    file_size_bytes: 0,
                    started_at: Utc::now(),
                    completed_at: None,
                }),
            )
                .into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                CoreError::Internal("failed to create export job".into())
                    .error_response(),
            ),
        )
            .into_response(),
    }
}

async fn execute_export_job(
    db: crate::db::DbRepository,
    job_id: Uuid,
    export_type: &str,
) -> Result<(), CoreError> {
    let export_dir = std::path::Path::new("/tmp/civitforge-exports");
    std::fs::create_dir_all(export_dir).map_err(|e| {
        CoreError::Internal(format!("failed to create export directory: {e}"))
    })?;

    let output_file = export_dir.join(format!("{job_id}.json"));

    let data = match export_type {
        "repos" => {
            let rows: Vec<serde_json::Value> = sqlx::query_scalar::<_, serde_json::Value>(
                "SELECT row_to_json(r) FROM repositories r LIMIT 1000",
            )
            .fetch_all(db.pool())
            .await
            .unwrap_or_default();
            serde_json::json!({"repos": rows})
        }
        "issues" => {
            let rows: Vec<serde_json::Value> = sqlx::query_scalar::<_, serde_json::Value>(
                "SELECT row_to_json(i) FROM issues i LIMIT 1000",
            )
            .fetch_all(db.pool())
            .await
            .unwrap_or_default();
            let comments: Vec<serde_json::Value> = sqlx::query_scalar::<_, serde_json::Value>(
                "SELECT row_to_json(c) FROM issue_comments c LIMIT 1000",
            )
            .fetch_all(db.pool())
            .await
            .unwrap_or_default();
            serde_json::json!({"issues": rows, "comments": comments})
        }
        "pull_requests" => {
            let rows: Vec<serde_json::Value> = sqlx::query_scalar::<_, serde_json::Value>(
                "SELECT row_to_json(pr) FROM pull_requests pr LIMIT 1000",
            )
            .fetch_all(db.pool())
            .await
            .unwrap_or_default();
            let reviews: Vec<serde_json::Value> = sqlx::query_scalar::<_, serde_json::Value>(
                "SELECT row_to_json(prv) FROM pr_reviewers prv LIMIT 1000",
            )
            .fetch_all(db.pool())
            .await
            .unwrap_or_default();
            serde_json::json!({"pull_requests": rows, "reviews": reviews})
        }
        "wiki" => {
            let pages: Vec<serde_json::Value> = sqlx::query_scalar::<_, serde_json::Value>(
                "SELECT row_to_json(wp) FROM wiki_pages wp LIMIT 1000",
            )
            .fetch_all(db.pool())
            .await
            .unwrap_or_default();
            let revisions: Vec<serde_json::Value> = sqlx::query_scalar::<_, serde_json::Value>(
                "SELECT row_to_json(wr) FROM wiki_revisions wr LIMIT 1000",
            )
            .fetch_all(db.pool())
            .await
            .unwrap_or_default();
            serde_json::json!({"pages": pages, "revisions": revisions})
        }
        "users" => {
            let rows: Vec<serde_json::Value> = sqlx::query_scalar::<_, serde_json::Value>(
                "SELECT row_to_json(u) FROM users u LIMIT 1000",
            )
            .fetch_all(db.pool())
            .await
            .unwrap_or_default();
            serde_json::json!({"users": rows})
        }
        "full" => {
            let repos: Vec<serde_json::Value> = sqlx::query_scalar::<_, serde_json::Value>(
                "SELECT row_to_json(r) FROM repositories r LIMIT 500",
            )
            .fetch_all(db.pool())
            .await
            .unwrap_or_default();
            let issues: Vec<serde_json::Value> = sqlx::query_scalar::<_, serde_json::Value>(
                "SELECT row_to_json(i) FROM issues i LIMIT 500",
            )
            .fetch_all(db.pool())
            .await
            .unwrap_or_default();
            let prs: Vec<serde_json::Value> = sqlx::query_scalar::<_, serde_json::Value>(
                "SELECT row_to_json(pr) FROM pull_requests pr LIMIT 500",
            )
            .fetch_all(db.pool())
            .await
            .unwrap_or_default();
            let users: Vec<serde_json::Value> = sqlx::query_scalar::<_, serde_json::Value>(
                "SELECT row_to_json(u) FROM users u LIMIT 500",
            )
            .fetch_all(db.pool())
            .await
            .unwrap_or_default();
            let wiki: Vec<serde_json::Value> = sqlx::query_scalar::<_, serde_json::Value>(
                "SELECT row_to_json(wp) FROM wiki_pages wp LIMIT 500",
            )
            .fetch_all(db.pool())
            .await
            .unwrap_or_default();
            serde_json::json!({
                "repos": repos,
                "issues": issues,
                "pull_requests": prs,
                "users": users,
                "wiki": wiki,
            })
        }
        _ => serde_json::json!({}),
    };

    let json_bytes = serde_json::to_vec_pretty(&data).unwrap_or_default();
    let file_size = json_bytes.len() as i64;

    std::fs::write(&output_file, &json_bytes).map_err(|e| {
        CoreError::Internal(format!("failed to write export file: {e}"))
    })?;

    let _ = sqlx::query(
        r#"UPDATE export_jobs
           SET status = 'completed', file_path = $1, file_size_bytes = $2, completed_at = NOW()
           WHERE id = $3"#,
    )
    .bind(output_file.to_string_lossy().to_string())
    .bind(file_size)
    .bind(job_id)
    .execute(db.pool())
    .await;

    Ok(())
}

pub async fn get_export_status(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Response {
    let result = sqlx::query_as::<_, ExportJob>(
        "SELECT * FROM export_jobs WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(state.db.pool())
    .await;

    match result {
        Ok(Some(job)) => {
            let response = ExportJobResponse {
                id: job.id,
                user_id: job.user_id,
                export_type: job.export_type,
                status: job.status,
                file_size_bytes: job.file_size_bytes,
                started_at: job.started_at,
                completed_at: job.completed_at,
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
    let result = sqlx::query_as::<_, ExportJob>(
        "SELECT * FROM export_jobs WHERE id = $1 AND status = 'completed'",
    )
    .bind(id)
    .fetch_optional(state.db.pool())
    .await;

    match result {
        Ok(Some(job)) => {
            if let Some(ref file_path) = job.file_path {
                match std::fs::read(file_path) {
                    Ok(data) => {
                        let mut headers = HeaderMap::new();
                        headers.insert(
                            axum::http::header::CONTENT_TYPE,
                            HeaderValue::from_static("application/json"),
                        );
                        headers.insert(
                            axum::http::header::CONTENT_DISPOSITION,
                            HeaderValue::from_str(&format!(
                                "attachment; filename=\"export-{}.json\"",
                                job.id
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
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("export job not found or not completed".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Internal(format!("database error: {e}")).error_response()),
        )
            .into_response(),
    }
}

pub async fn import_data(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<ImportDataRequest>,
) -> Response {
    let valid_types = ["repos", "issues", "pull_requests", "wiki", "users", "full"];
    if !valid_types.contains(&req.import_type.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                CoreError::BadRequest(format!(
                    "invalid import_type: {}, must be one of: {}",
                    req.import_type,
                    valid_types.join(", ")
                ))
                .error_response(),
            ),
        )
            .into_response();
    }

    if req.import_type == "users"
        && let Err(rejection) = require_admin(&auth) {
            return rejection.into_response();
        }

    let user_id = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => Uuid::nil(),
    };

    let mut imported_count = 0usize;

    match req.import_type.as_str() {
        "repos" => {
            if let Some(repos) = req.data.get("repos").and_then(|v| v.as_array()) {
                for repo_val in repos {
                    let name = repo_val.get("name").and_then(|v| v.as_str()).unwrap_or("imported");
                    let description = repo_val.get("description").and_then(|v| v.as_str()).unwrap_or("");
                    let visibility = repo_val.get("visibility").and_then(|v| v.as_str()).unwrap_or("public");
                    let default_branch = repo_val.get("default_branch").and_then(|v| v.as_str()).unwrap_or("main");
                    let _ = state.db.create_repo(name, description, user_id, None, visibility, default_branch).await;
                    imported_count += 1;
                }
            }
        }
        "issues" => {
            if let Some(issues) = req.data.get("issues").and_then(|v| v.as_array()) {
                for issue_val in issues {
                    let title = issue_val.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled");
                    let body = issue_val.get("body").and_then(|v| v.as_str()).unwrap_or("");
                    let repo_id = issue_val.get("repo_id").and_then(|v| v.as_str())
                        .and_then(|s| Uuid::parse_str(s).ok())
                        .unwrap_or(Uuid::nil());
                    let _ = state.db.create_issue(repo_id, title, body, user_id).await;
                    imported_count += 1;
                }
            }
        }
        "pull_requests" => {
            if let Some(prs) = req.data.get("pull_requests").and_then(|v| v.as_array()) {
                for pr_val in prs {
                    let title = pr_val.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled");
                    let body = pr_val.get("body").and_then(|v| v.as_str()).unwrap_or("");
                    let source_branch = pr_val.get("source_branch").and_then(|v| v.as_str()).unwrap_or("feature");
                    let target_branch = pr_val.get("target_branch").and_then(|v| v.as_str()).unwrap_or("main");
                    let draft = pr_val.get("draft").and_then(|v| v.as_bool()).unwrap_or(false);
                    let auto_merge = pr_val.get("auto_merge").and_then(|v| v.as_bool()).unwrap_or(false);
                    let repo_id = pr_val.get("repo_id").and_then(|v| v.as_str())
                        .and_then(|s| Uuid::parse_str(s).ok())
                        .unwrap_or(Uuid::nil());
                    let _ = state.db.create_pr(repo_id, title, body, user_id, source_branch, target_branch, draft, auto_merge).await;
                    imported_count += 1;
                }
            }
        }
        _ => {
            return (
                StatusCode::NOT_IMPLEMENTED,
                Json(
                    CoreError::BadRequest(format!(
                        "import type '{}' is not yet implemented",
                        req.import_type
                    ))
                    .error_response(),
                ),
            )
                .into_response();
        }
    }

    (
        StatusCode::OK,
        Json(ImportDataResponse {
            status: "completed".into(),
            message: format!("Imported {imported_count} records of type {}", req.import_type),
            imported_count,
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_job_response_serialization() {
        let response = ExportJobResponse {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            export_type: "repos".into(),
            status: "completed".into(),
            file_size_bytes: 1024,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"export_type\":\"repos\""));
        assert!(json.contains("\"status\":\"completed\""));
        assert!(json.contains("\"file_size_bytes\":1024"));
    }

    #[test]
    fn test_create_export_request_deserialization() {
        let json = r#"{"export_type": "full"}"#;
        let req: CreateExportRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.export_type, "full");
    }

    #[test]
    fn test_import_data_response_serialization() {
        let response = ImportDataResponse {
            status: "completed".into(),
            message: "Imported 5 records".into(),
            imported_count: 5,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"imported_count\":5"));
    }

    #[test]
    fn test_import_data_request_deserialization() {
        let json = r#"{"data": {"repos": []}, "import_type": "repos"}"#;
        let req: ImportDataRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.import_type, "repos");
    }

    #[test]
    fn test_export_routes_compile() {
        let router = export_routes();
        let _ = router;
    }
}
