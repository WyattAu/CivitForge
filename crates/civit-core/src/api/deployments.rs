#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::{CoreError, ErrorResponse};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct DeploymentResponse {
    pub id: String,
    pub repo_id: String,
    pub environment_id: Option<String>,
    pub sha: String,
    pub status: String,
    pub creator_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateDeploymentRequest {
    pub environment_id: Option<String>,
    pub sha: String,
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
    pub status: Option<String>,
}

fn default_page() -> u32 {
    1
}
fn default_per_page() -> u32 {
    20
}

async fn resolve_repo_id(
    state: &AppState,
    owner: &str,
    name: &str,
) -> Result<Uuid, (StatusCode, Json<ErrorResponse>)> {
    let owner_uuid = if let Ok(id) = Uuid::parse_str(owner) {
        id
    } else if let Ok(user) = state.db.get_user_by_username(owner).await {
        user.id
    } else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("user not found".into()).error_response()),
        ));
    };

    match state.db.get_repo_by_owner_name(owner_uuid, name).await {
        Ok(repo) => Ok(repo.id),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )),
    }
}

pub async fn create_deployment(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<CreateDeploymentRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match resolve_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp.into_response(),
    };

    let creator_id = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(CoreError::Auth("invalid user id".into()).error_response()),
            )
                .into_response();
        }
    };

    let env_id = match &req.environment_id {
        Some(eid_str) => match Uuid::parse_str(eid_str) {
            Ok(id) => Some(id),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(CoreError::BadRequest("invalid environment id".into()).error_response()),
                )
                    .into_response();
            }
        },
        None => None,
    };

    let result = sqlx::query_as::<
        _,
        (
            Uuid,
            Uuid,
            Option<Uuid>,
            String,
            String,
            Option<Uuid>,
            chrono::DateTime<chrono::Utc>,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        "INSERT INTO deployments (repo_id, environment_id, sha, creator_id) \
         VALUES ($1, $2, $3, $4) \
         RETURNING id, repo_id, environment_id, sha, status, creator_id, created_at, updated_at",
    )
    .bind(repo_id)
    .bind(env_id)
    .bind(&req.sha)
    .bind(creator_id)
    .fetch_one(pool)
    .await;

    match result {
        Ok((id, repo_id, environment_id, sha, status, creator_id, created_at, updated_at)) => (
            StatusCode::CREATED,
            Json(DeploymentResponse {
                id: id.to_string(),
                repo_id: repo_id.to_string(),
                environment_id: environment_id.map(|e| e.to_string()),
                sha,
                status,
                creator_id: creator_id.map(|c| c.to_string()),
                created_at: created_at.to_rfc3339(),
                updated_at: updated_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_deployments(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Query(params): Query<ListDeploymentsParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match resolve_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp.into_response(),
    };

    let offset = (params.page.saturating_sub(1) * params.per_page) as i64;

    let rows = if let Some(ref status_filter) = params.status {
        sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                Option<Uuid>,
                String,
                String,
                Option<Uuid>,
                chrono::DateTime<chrono::Utc>,
                chrono::DateTime<chrono::Utc>,
            ),
        >(
            "SELECT id, repo_id, environment_id, sha, status, creator_id, created_at, updated_at \
             FROM deployments WHERE repo_id = $1 AND status = $2 \
             ORDER BY created_at DESC LIMIT $3 OFFSET $4",
        )
        .bind(repo_id)
        .bind(status_filter)
        .bind(params.per_page as i64)
        .bind(offset)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                Option<Uuid>,
                String,
                String,
                Option<Uuid>,
                chrono::DateTime<chrono::Utc>,
                chrono::DateTime<chrono::Utc>,
            ),
        >(
            "SELECT id, repo_id, environment_id, sha, status, creator_id, created_at, updated_at \
             FROM deployments WHERE repo_id = $1 \
             ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(repo_id)
        .bind(params.per_page as i64)
        .bind(offset)
        .fetch_all(pool)
        .await
    };

    match rows {
        Ok(rows) => {
            let deps: Vec<DeploymentResponse> = rows
                .into_iter()
                .map(
                    |(
                        id,
                        repo_id,
                        environment_id,
                        sha,
                        status,
                        creator_id,
                        created_at,
                        updated_at,
                    )| {
                        DeploymentResponse {
                            id: id.to_string(),
                            repo_id: repo_id.to_string(),
                            environment_id: environment_id.map(|e| e.to_string()),
                            sha,
                            status,
                            creator_id: creator_id.map(|c| c.to_string()),
                            created_at: created_at.to_rfc3339(),
                            updated_at: updated_at.to_rfc3339(),
                        }
                    },
                )
                .collect();
            (StatusCode::OK, Json(deps)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_deployment_status(
    State(state): State<AppState>,
    Path((owner, name, deployment_id)): Path<(String, String, String)>,
    auth: AuthUser,
    Json(req): Json<UpdateDeploymentStatusRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let _repo_id = match resolve_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp.into_response(),
    };

    let _auth_id = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(CoreError::Auth("invalid user id".into()).error_response()),
            )
                .into_response();
        }
    };

    let did = match Uuid::parse_str(&deployment_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid deployment id".into()).error_response()),
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

    let result = sqlx::query_as::<
        _,
        (
            Uuid,
            Uuid,
            Option<Uuid>,
            String,
            String,
            Option<Uuid>,
            chrono::DateTime<chrono::Utc>,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        "UPDATE deployments SET status = $2, updated_at = NOW() \
         WHERE id = $1 \
         RETURNING id, repo_id, environment_id, sha, status, creator_id, created_at, updated_at",
    )
    .bind(did)
    .bind(&req.status)
    .fetch_one(pool)
    .await;

    match result {
        Ok((id, repo_id, environment_id, sha, status, creator_id, created_at, updated_at)) => (
            StatusCode::OK,
            Json(DeploymentResponse {
                id: id.to_string(),
                repo_id: repo_id.to_string(),
                environment_id: environment_id.map(|e| e.to_string()),
                sha,
                status,
                creator_id: creator_id.map(|c| c.to_string()),
                created_at: created_at.to_rfc3339(),
                updated_at: updated_at.to_rfc3339(),
            }),
        )
            .into_response(),
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

pub fn deployment_routes() -> axum::Router<AppState> {
    use axum::routing::patch;
    axum::Router::new().route(
        "/api/v1/repos/{owner}/{name}/deployments/{deployment_id}/status",
        patch(update_deployment_status),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployment_response_serializes() {
        let resp = DeploymentResponse {
            id: "00000000-0000-0000-0000-000000000001".into(),
            repo_id: "00000000-0000-0000-0000-000000000002".into(),
            environment_id: Some("00000000-0000-0000-0000-000000000003".into()),
            sha: "abc123def456".into(),
            status: "pending".into(),
            creator_id: Some("00000000-0000-0000-0000-000000000004".into()),
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("abc123def456"));
        assert!(json.contains("pending"));
    }

    #[test]
    fn test_create_deployment_request() {
        let req: CreateDeploymentRequest =
            serde_json::from_str(r#"{"sha": "abc123", "environment_id": null}"#).unwrap();
        assert_eq!(req.sha, "abc123");
        assert!(req.environment_id.is_none());
    }

    #[test]
    fn test_update_status_request() {
        let req: UpdateDeploymentStatusRequest =
            serde_json::from_str(r#"{"status": "success"}"#).unwrap();
        assert_eq!(req.status, "success");
    }

    #[test]
    fn test_valid_statuses() {
        let valid = ["pending", "in_progress", "success", "failure", "cancelled"];
        assert!(valid.contains(&"pending"));
        assert!(valid.contains(&"success"));
        assert!(!valid.contains(&"unknown"));
    }
}
