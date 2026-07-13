//! Pipeline Environments API endpoints.

#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use civit_ci::pipeline;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateEnvironmentRequest {
    pub name: String,
    pub url: Option<String>,
    #[serde(default)]
    pub protected: bool,
    #[serde(default = "default_true")]
    pub auto_deploy: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct UpdateEnvironmentRequest {
    pub name: Option<String>,
    pub url: Option<Option<String>>,
    pub protected: Option<bool>,
    pub auto_deploy: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProtectionRequest {
    #[serde(default = "default_one")]
    pub required_approvals: i32,
    #[serde(default)]
    pub wait_timer: i32,
    #[serde(default = "default_true")]
    pub allow_admin_override: bool,
}

fn default_one() -> i32 {
    1
}

#[derive(Debug, Deserialize)]
pub struct CreateDeploymentRequest {
    pub sha: String,
    pub pipeline_run_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDeploymentStatusRequest {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ListDeploymentsParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}
fn default_per_page() -> u32 {
    20
}

async fn resolve_repo_id(
    pool: &sqlx::PgPool,
    owner: &str,
    name: &str,
) -> std::result::Result<Uuid, Response> {
    let owner_uuid = if let Ok(id) = Uuid::parse_str(owner) {
        id
    } else {
        // Try to find user by username
        let user_row = sqlx::query_as::<_, (Uuid,)>("SELECT id FROM users WHERE username = $1")
            .bind(owner)
            .fetch_optional(pool)
            .await;

        match user_row {
            Ok(Some((id,))) => id,
            Ok(None) => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(CoreError::NotFound("user not found".into()).error_response())
                        .into_response(),
                )
                    .into_response());
            }
            Err(e) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CoreError::Database(e.to_string()).error_response())
                        .into_response(),
                )
                    .into_response());
            }
        }
    };

    let repo_row = sqlx::query_as::<_, (Uuid,)>(
        "SELECT id FROM repositories WHERE owner_id = $1 AND name = $2",
    )
    .bind(owner_uuid)
    .bind(name)
    .fetch_optional(pool)
    .await;

    match repo_row {
        Ok(Some((id,))) => Ok(id),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response())
                .into_response(),
        )
            .into_response()),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response())
                .into_response(),
        )
            .into_response()),
    }
}

pub async fn list_environments(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let repo_id = match resolve_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    match pipeline::list_environments(pool, repo_id).await {
        Ok(envs) => (StatusCode::OK, Json(envs)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn create_environment(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Json(req): Json<CreateEnvironmentRequest>,
) -> Response {
    let pool = state.db.pool();
    let repo_id = match resolve_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    match pipeline::create_environment(
        pool,
        repo_id,
        &req.name,
        req.url.as_deref(),
        req.protected,
        req.auto_deploy,
    )
    .await
    {
        Ok(env) => (StatusCode::CREATED, Json(env)).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("duplicate key") || msg.contains("unique") {
                (
                    StatusCode::CONFLICT,
                    Json(
                        CoreError::BadRequest("environment name already exists".into())
                            .error_response(),
                    ),
                )
                    .into_response()
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CoreError::Database(msg).error_response()),
                )
                    .into_response()
            }
        }
    }
}

pub async fn get_environment(
    State(state): State<AppState>,
    Path((owner, name, env_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let _repo_id = match resolve_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let eid = match Uuid::parse_str(&env_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid environment ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match pipeline::get_environment(pool, eid).await {
        Ok(Some(env)) => {
            let response: pipeline::EnvironmentResponse = env.into();
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("environment not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_environment(
    State(state): State<AppState>,
    Path((owner, name, env_id)): Path<(String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<UpdateEnvironmentRequest>,
) -> Response {
    let pool = state.db.pool();
    let _repo_id = match resolve_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let eid = match Uuid::parse_str(&env_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid environment ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let url = req.url.unwrap_or(None);

    match pipeline::update_environment(
        pool,
        eid,
        req.name.as_deref(),
        url.as_deref(),
        req.protected,
        req.auto_deploy,
    )
    .await
    {
        Ok(env) => (StatusCode::OK, Json(env)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_environment(
    State(state): State<AppState>,
    Path((owner, name, env_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let _repo_id = match resolve_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let eid = match Uuid::parse_str(&env_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid environment ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match pipeline::delete_environment(pool, eid).await {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "deleted"})),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("environment not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_protections(
    State(state): State<AppState>,
    Path((owner, name, env_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let _repo_id = match resolve_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let eid = match Uuid::parse_str(&env_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid environment ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match pipeline::get_protections(pool, eid).await {
        Ok(Some(protections)) => (StatusCode::OK, Json(protections)).into_response(),
        Ok(None) => (StatusCode::OK, Json(serde_json::json!({}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn upsert_protections(
    State(state): State<AppState>,
    Path((owner, name, env_id)): Path<(String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<CreateProtectionRequest>,
) -> Response {
    let pool = state.db.pool();
    let _repo_id = match resolve_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let eid = match Uuid::parse_str(&env_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid environment ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match pipeline::upsert_protections(
        pool,
        eid,
        req.required_approvals,
        req.wait_timer,
        req.allow_admin_override,
    )
    .await
    {
        Ok(protections) => (StatusCode::OK, Json(protections)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn create_deployment(
    State(state): State<AppState>,
    Path((owner, name, env_id)): Path<(String, String, String)>,
    auth: AuthUser,
    Json(req): Json<CreateDeploymentRequest>,
) -> Response {
    let pool = state.db.pool();
    let _repo_id = match resolve_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let eid = match Uuid::parse_str(&env_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid environment ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let creator_id = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => Some(id),
        Err(_) => None,
    };

    let pipeline_run_id = match &req.pipeline_run_id {
        Some(id_str) => match Uuid::parse_str(id_str) {
            Ok(id) => Some(id),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(CoreError::BadRequest("invalid pipeline run ID".into()).error_response()),
                )
                    .into_response();
            }
        },
        None => None,
    };

    match pipeline::create_environment_deployment(
        pool,
        eid,
        pipeline_run_id,
        &req.sha,
        creator_id,
    )
    .await
    {
        Ok(deployment) => (StatusCode::CREATED, Json(deployment)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_deployments(
    State(state): State<AppState>,
    Path((owner, name, env_id)): Path<(String, String, String)>,
    _auth: AuthUser,
    Query(params): Query<ListDeploymentsParams>,
) -> Response {
    let pool = state.db.pool();
    let _repo_id = match resolve_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let eid = match Uuid::parse_str(&env_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid environment ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let offset = (params.page.saturating_sub(1) * params.per_page) as i64;

    match pipeline::list_environment_deployments(pool, eid, params.per_page as i64, offset).await {
        Ok(deployments) => (StatusCode::OK, Json(deployments)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_deployment_status(
    State(state): State<AppState>,
    Path((owner, name, env_id, deployment_id)): Path<(String, String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<UpdateDeploymentStatusRequest>,
) -> Response {
    let pool = state.db.pool();
    let _repo_id = match resolve_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let _eid = match Uuid::parse_str(&env_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid environment ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let did = match Uuid::parse_str(&deployment_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid deployment ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let valid_statuses = ["pending", "in_progress", "success", "failure", "cancelled"];
    if !valid_statuses.contains(&req.status.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                CoreError::BadRequest(format!(
                    "invalid status: must be one of {}",
                    valid_statuses.join(", ")
                ))
                .error_response(),
            ),
        )
            .into_response();
    }

    match pipeline::update_deployment_status(pool, did, &req.status).await {
        Ok(deployment) => (StatusCode::OK, Json(deployment)).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("returned no rows") {
                (
                    StatusCode::NOT_FOUND,
                    Json(CoreError::NotFound("deployment not found".into()).error_response()),
                )
                    .into_response()
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CoreError::Database(msg).error_response()),
                )
                    .into_response()
            }
        }
    }
}

pub fn environment_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/environments",
            get(list_environments).post(create_environment),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}",
            get(get_environment)
                .patch(update_environment)
                .delete(delete_environment),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/protections",
            get(get_protections).post(upsert_protections),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/deployments",
            get(list_deployments).post(create_deployment),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/deployments/{deployment_id}/status",
            patch(update_deployment_status),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_environment_request() {
        let json = r#"{"name": "staging", "url": "https://staging.example.com", "protected": true}"#;
        let req: CreateEnvironmentRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "staging");
        assert_eq!(req.url.as_deref(), Some("https://staging.example.com"));
        assert!(req.protected);
        assert!(req.auto_deploy);
    }

    #[test]
    fn test_update_environment_request() {
        let json = r#"{"name": "production", "protected": true}"#;
        let req: UpdateEnvironmentRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name.as_deref(), Some("production"));
        assert_eq!(req.protected, Some(true));
    }

    #[test]
    fn test_create_protection_request() {
        let json = r#"{"required_approvals": 2, "wait_timer": 300, "allow_admin_override": false}"#;
        let req: CreateProtectionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.required_approvals, 2);
        assert_eq!(req.wait_timer, 300);
        assert!(!req.allow_admin_override);
    }

    #[test]
    fn test_create_deployment_request() {
        let json = r#"{"sha": "abc123", "pipeline_run_id": null}"#;
        let req: CreateDeploymentRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.sha, "abc123");
        assert!(req.pipeline_run_id.is_none());
    }
}
