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
use civit_ci::environment_webhooks;
use civit_ci::health_checks;
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
    pub deployment_branch_policy: Option<serde_json::Value>,
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
    pub deployment_branch_policy: Option<Option<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProtectionRequest {
    #[serde(default = "default_one")]
    pub required_approvals: i32,
    #[serde(default)]
    pub wait_timer: i32,
    #[serde(default = "default_true")]
    pub allow_admin_override: bool,
    #[serde(default)]
    pub allowed_branches: Vec<String>,
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

#[derive(Debug, Deserialize)]
pub struct CreateLockRequest {
    #[serde(default)]
    pub reason: String,
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
    let deployment_branch_policy = req.deployment_branch_policy.unwrap_or(None);

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
        &req.allowed_branches,
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

pub async fn create_lock(
    State(state): State<AppState>,
    Path((owner, name, env_id)): Path<(String, String, String)>,
    auth: AuthUser,
    Json(req): Json<CreateLockRequest>,
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

    let user_id = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid user ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match pipeline::create_deployment_lock(pool, eid, user_id, &req.reason).await {
        Ok(lock) => (StatusCode::CREATED, Json(lock)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_locks(
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

    match pipeline::list_deployment_locks(pool, eid).await {
        Ok(locks) => (StatusCode::OK, Json(locks)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn remove_lock(
    State(state): State<AppState>,
    Path((owner, name, env_id, lock_id)): Path<(String, String, String, String)>,
    _auth: AuthUser,
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

    let lid = match Uuid::parse_str(&lock_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid lock ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match pipeline::remove_deployment_lock(pool, lid).await {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "deleted"})),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("lock not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_approval_rules(
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

    match pipeline::get_approval_rules(pool, eid).await {
        Ok(Some(rules)) => (StatusCode::OK, Json(rules)).into_response(),
        Ok(None) => (StatusCode::OK, Json(serde_json::json!({}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn upsert_approval_rules(
    State(state): State<AppState>,
    Path((owner, name, env_id)): Path<(String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<CreateApprovalRuleRequest>,
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

    match pipeline::upsert_approval_rules(
        pool,
        eid,
        req.required_approvers,
        &req.approver_groups,
        req.auto_approve_after_hours,
    )
    .await
    {
        Ok(rules) => (StatusCode::OK, Json(rules)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_approvals(
    State(state): State<AppState>,
    Path((owner, name, env_id)): Path<(String, String, String)>,
    _auth: AuthUser,
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

    match pipeline::list_deployment_approvals(pool, Uuid::nil()).await {
        Ok(approvals) => (StatusCode::OK, Json(approvals)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn create_approval(
    State(state): State<AppState>,
    Path((owner, name, env_id)): Path<(String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<CreateApprovalRequest>,
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

    let did = match Uuid::parse_str(&req.deployment_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid deployment ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let approver_id = match Uuid::parse_str(&req.approver_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid approver ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match pipeline::create_deployment_approval(pool, eid, did, approver_id).await {
        Ok(approval) => (StatusCode::CREATED, Json(approval)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_approval_status(
    State(state): State<AppState>,
    Path((owner, name, env_id, approval_id)): Path<(String, String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<UpdateApprovalStatusRequest>,
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

    let aid = match Uuid::parse_str(&approval_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid approval ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let result = match req.status.as_str() {
        "approved" => pipeline::approve_deployment(pool, aid).await,
        "rejected" => pipeline::reject_deployment(pool, aid).await,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("status must be 'approved' or 'rejected'".into()).error_response()),
            )
                .into_response();
        }
    };

    match result {
        Ok(approval) => (StatusCode::OK, Json(approval)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn trigger_auto_approve(
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

    match pipeline::auto_approve_pending(pool, eid).await {
        Ok(count) => (
            StatusCode::OK,
            Json(serde_json::json!({"auto_approved": count})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// Webhook handlers
// ---------------------------------------------------------------------------

pub async fn list_webhooks(
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

    match environment_webhooks::list_webhooks(pool, eid).await {
        Ok(webhooks) => (StatusCode::OK, Json(webhooks)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn create_webhook(
    State(state): State<AppState>,
    Path((owner, name, env_id)): Path<(String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<CreateWebhookRequest>,
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

    match environment_webhooks::create_webhook(pool, eid, &req.url, &req.events, req.enabled).await {
        Ok(webhook) => (StatusCode::CREATED, Json(webhook)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_webhook(
    State(state): State<AppState>,
    Path((owner, name, env_id, webhook_id)): Path<(String, String, String, String)>,
    _auth: AuthUser,
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

    let wid = match Uuid::parse_str(&webhook_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid webhook ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match environment_webhooks::get_webhook(pool, wid).await {
        Ok(Some(webhook)) => (StatusCode::OK, Json(webhook)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("webhook not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_webhook(
    State(state): State<AppState>,
    Path((owner, name, env_id, webhook_id)): Path<(String, String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<UpdateWebhookRequest>,
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

    let wid = match Uuid::parse_str(&webhook_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid webhook ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match environment_webhooks::update_webhook(pool, wid, req.url.as_deref(), req.events.as_deref(), req.enabled).await {
        Ok(webhook) => (StatusCode::OK, Json(webhook)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_webhook(
    State(state): State<AppState>,
    Path((owner, name, env_id, webhook_id)): Path<(String, String, String, String)>,
    _auth: AuthUser,
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

    let wid = match Uuid::parse_str(&webhook_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid webhook ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match environment_webhooks::delete_webhook(pool, wid).await {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "deleted"})),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("webhook not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_webhook_deliveries(
    State(state): State<AppState>,
    Path((owner, name, env_id, webhook_id)): Path<(String, String, String, String)>,
    _auth: AuthUser,
    Query(params): Query<ListDeliveriesParams>,
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

    let wid = match Uuid::parse_str(&webhook_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid webhook ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let offset = (params.page.saturating_sub(1) * params.per_page) as i64;

    match environment_webhooks::list_deliveries(pool, wid, params.per_page as i64, offset).await {
        Ok(deliveries) => (StatusCode::OK, Json(deliveries)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_webhook_stats(
    State(state): State<AppState>,
    Path((owner, name, env_id, webhook_id)): Path<(String, String, String, String)>,
    _auth: AuthUser,
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

    let wid = match Uuid::parse_str(&webhook_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid webhook ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match environment_webhooks::get_delivery_stats(pool, wid).await {
        Ok(stats) => (StatusCode::OK, Json(stats)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// Notification handlers
// ---------------------------------------------------------------------------

pub async fn list_notifications(
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

    match environment_webhooks::list_notifications(pool, eid).await {
        Ok(notifications) => (StatusCode::OK, Json(notifications)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn create_notification(
    State(state): State<AppState>,
    Path((owner, name, env_id)): Path<(String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<CreateNotificationRequest>,
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

    let config = req.config.unwrap_or(serde_json::json!({}));

    match environment_webhooks::create_notification(pool, eid, &req.notification_type, &config, req.enabled).await {
        Ok(notification) => (StatusCode::CREATED, Json(notification)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_notification(
    State(state): State<AppState>,
    Path((owner, name, env_id, notification_id)): Path<(String, String, String, String)>,
    _auth: AuthUser,
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

    let nid = match Uuid::parse_str(&notification_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid notification ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match environment_webhooks::get_notification(pool, nid).await {
        Ok(Some(notification)) => (StatusCode::OK, Json(notification)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("notification not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_notification(
    State(state): State<AppState>,
    Path((owner, name, env_id, notification_id)): Path<(String, String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<UpdateNotificationRequest>,
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

    let nid = match Uuid::parse_str(&notification_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid notification ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match environment_webhooks::update_notification(pool, nid, req.notification_type.as_deref(), req.config.as_ref(), req.enabled).await {
        Ok(notification) => (StatusCode::OK, Json(notification)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_notification(
    State(state): State<AppState>,
    Path((owner, name, env_id, notification_id)): Path<(String, String, String, String)>,
    _auth: AuthUser,
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

    let nid = match Uuid::parse_str(&notification_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid notification ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match environment_webhooks::delete_notification(pool, nid).await {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "deleted"})),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("notification not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// Health Check handlers
// ---------------------------------------------------------------------------

pub async fn list_health_checks(
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

    match health_checks::list_health_checks(pool, eid).await {
        Ok(checks) => (StatusCode::OK, Json(checks)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn create_health_check(
    State(state): State<AppState>,
    Path((owner, name, env_id)): Path<(String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<CreateHealthCheckRequest>,
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

    match health_checks::create_health_check(
        pool,
        eid,
        &req.check_type,
        req.endpoint.as_deref(),
        req.interval_seconds,
        req.timeout_seconds,
    )
    .await
    {
        Ok(check) => (StatusCode::CREATED, Json(check)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_health_check(
    State(state): State<AppState>,
    Path((owner, name, env_id, check_id)): Path<(String, String, String, String)>,
    _auth: AuthUser,
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

    let cid = match Uuid::parse_str(&check_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid check ID".into()).error_response()),
            )
                .into_response();
        }
    };

    // For now, return the check if found in the list
    let eid_uuid = Uuid::parse_str(&env_id).unwrap_or_default();
    match health_checks::list_health_checks(pool, eid_uuid).await {
        Ok(checks) => {
            if let Some(check) = checks.into_iter().find(|c| c.id == check_id) {
                (StatusCode::OK, Json(check)).into_response()
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(CoreError::NotFound("health check not found".into()).error_response()),
                )
                    .into_response()
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_health_check_status(
    State(state): State<AppState>,
    Path((owner, name, env_id, check_id)): Path<(String, String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<UpdateHealthCheckStatusRequest>,
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

    let cid = match Uuid::parse_str(&check_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid check ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match health_checks::update_health_check_status(pool, cid, &req.status).await {
        Ok(check) => (StatusCode::OK, Json(check)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_health_check(
    State(state): State<AppState>,
    Path((owner, name, env_id, check_id)): Path<(String, String, String, String)>,
    _auth: AuthUser,
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

    let cid = match Uuid::parse_str(&check_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid check ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match health_checks::delete_health_check(pool, cid).await {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "deleted"})),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("health check not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateApprovalRuleRequest {
    #[serde(default = "default_one")]
    pub required_approvers: i32,
    #[serde(default)]
    pub approver_groups: Vec<String>,
    pub auto_approve_after_hours: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateApprovalRequest {
    pub deployment_id: String,
    pub approver_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateApprovalStatusRequest {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWebhookRequest {
    pub url: String,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWebhookRequest {
    pub url: Option<String>,
    pub events: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNotificationRequest {
    pub notification_type: String,
    pub config: Option<serde_json::Value>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNotificationRequest {
    pub notification_type: Option<String>,
    pub config: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListDeliveriesParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

#[derive(Debug, Deserialize)]
pub struct CreateHealthCheckRequest {
    pub check_type: String,
    pub endpoint: Option<String>,
    #[serde(default = "default_sixty")]
    pub interval_seconds: i32,
    #[serde(default = "default_ten")]
    pub timeout_seconds: i32,
}

fn default_sixty() -> i32 {
    60
}

fn default_ten() -> i32 {
    10
}

#[derive(Debug, Deserialize)]
pub struct UpdateHealthCheckStatusRequest {
    pub status: String,
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
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/locks",
            get(list_locks).post(create_lock),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/locks/{lock_id}",
            delete(remove_lock),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/approval-rules",
            get(get_approval_rules).post(upsert_approval_rules),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/approvals",
            get(list_approvals).post(create_approval),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/approvals/{approval_id}",
            patch(update_approval_status),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/auto-approve",
            post(trigger_auto_approve),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/webhooks",
            get(list_webhooks).post(create_webhook),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/webhooks/{webhook_id}",
            get(get_webhook)
                .patch(update_webhook)
                .delete(delete_webhook),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/webhooks/{webhook_id}/deliveries",
            get(list_webhook_deliveries),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/webhooks/{webhook_id}/stats",
            get(get_webhook_stats),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/notifications",
            get(list_notifications).post(create_notification),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/notifications/{notification_id}",
            get(get_notification)
                .patch(update_notification)
                .delete(delete_notification),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/health-checks",
            get(list_health_checks).post(create_health_check),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/health-checks/{check_id}",
            get(get_health_check)
                .patch(update_health_check_status)
                .delete(delete_health_check),
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
        let json = r#"{"required_approvals": 2, "wait_timer": 300, "allow_admin_override": false, "allowed_branches": ["main", "release/*"]}"#;
        let req: CreateProtectionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.required_approvals, 2);
        assert_eq!(req.wait_timer, 300);
        assert!(!req.allow_admin_override);
        assert_eq!(req.allowed_branches, vec!["main", "release/*"]);
    }

    #[test]
    fn test_create_protection_request_defaults() {
        let json = r#"{}"#;
        let req: CreateProtectionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.required_approvals, 1);
        assert_eq!(req.wait_timer, 0);
        assert!(req.allow_admin_override);
        assert!(req.allowed_branches.is_empty());
    }

    #[test]
    fn test_create_deployment_request() {
        let json = r#"{"sha": "abc123", "pipeline_run_id": null}"#;
        let req: CreateDeploymentRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.sha, "abc123");
        assert!(req.pipeline_run_id.is_none());
    }
}
