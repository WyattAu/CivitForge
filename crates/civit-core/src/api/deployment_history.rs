#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::{CoreError, ErrorResponse};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct DeploymentHistoryResponse {
    pub id: String,
    pub environment_id: String,
    pub version: String,
    pub sha: String,
    pub status: String,
    pub deployed_by: String,
    pub rollback_of: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ListDeploymentHistoryParams {
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

async fn resolve_environment_id(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    env_name: &str,
) -> Result<Uuid, (StatusCode, Json<ErrorResponse>)> {
    let eid = match Uuid::parse_str(env_name) {
        Ok(id) => id,
        Err(_) => {
            let row = sqlx::query_as::<_, (Uuid,)>(
                "SELECT id FROM pipeline_environments WHERE repo_id = $1 AND name = $2",
            )
            .bind(repo_id)
            .bind(env_name)
            .fetch_optional(pool)
            .await;
            match row {
                Ok(Some((id,))) => id,
                _ => {
                    return Err((
                        StatusCode::NOT_FOUND,
                        Json(
                            CoreError::NotFound("environment not found".into()).error_response(),
                        ),
                    ));
                }
            }
        }
    };
    Ok(eid)
}

pub async fn list_deployments(
    State(state): State<AppState>,
    Path((owner, name, env)): Path<(String, String, String)>,
    _auth: AuthUser,
    Query(params): Query<ListDeploymentHistoryParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match resolve_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp.into_response(),
    };

    let env_id = match resolve_environment_id(pool, repo_id, &env).await {
        Ok(id) => id,
        Err(resp) => return resp.into_response(),
    };

    let offset = (params.page.saturating_sub(1) * params.per_page) as i64;

    match state
        .db
        .list_deployment_history(env_id, params.per_page as i64, offset)
        .await
    {
        Ok(rows) => {
            let deps: Vec<DeploymentHistoryResponse> = rows
                .into_iter()
                .map(|r| DeploymentHistoryResponse {
                    id: r.id.to_string(),
                    environment_id: r.environment_id.to_string(),
                    version: r.version,
                    sha: r.sha,
                    status: r.status,
                    deployed_by: r.deployed_by.to_string(),
                    rollback_of: r.rollback_of.map(|id| id.to_string()),
                    created_at: r.created_at.to_rfc3339(),
                })
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

pub async fn rollback_deployment(
    State(state): State<AppState>,
    Path((owner, name, env, deployment_id)): Path<(String, String, String, String)>,
    auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let _repo_id = match resolve_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp.into_response(),
    };

    let _eid = match resolve_environment_id(pool, _repo_id, &env).await {
        Ok(id) => id,
        Err(resp) => return resp.into_response(),
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

    let user_id = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(CoreError::Auth("invalid user id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.rollback_deployment(did, user_id).await {
        Ok(rolled_back) => (
            StatusCode::CREATED,
            Json(DeploymentHistoryResponse {
                id: rolled_back.id.to_string(),
                environment_id: rolled_back.environment_id.to_string(),
                version: rolled_back.version,
                sha: rolled_back.sha,
                status: rolled_back.status,
                deployed_by: rolled_back.deployed_by.to_string(),
                rollback_of: rolled_back.rollback_of.map(|id| id.to_string()),
                created_at: rolled_back.created_at.to_rfc3339(),
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

pub async fn get_rollback_status(
    State(state): State<AppState>,
    Path((owner, name, env)): Path<(String, String, String)>,
    Query(params): Query<RollbackStatusQuery>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let _repo_id = match resolve_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp.into_response(),
    };

    let _eid = match resolve_environment_id(pool, _repo_id, &env).await {
        Ok(id) => id,
        Err(resp) => return resp.into_response(),
    };

    let did = match Uuid::parse_str(&params.deployment_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid deployment id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.get_rollback_status(did).await {
        Ok(rollback) => (
            StatusCode::OK,
            Json(DeploymentHistoryResponse {
                id: rollback.id.to_string(),
                environment_id: rollback.environment_id.to_string(),
                version: rollback.version,
                sha: rollback.sha,
                status: rollback.status,
                deployed_by: rollback.deployed_by.to_string(),
                rollback_of: rollback.rollback_of.map(|id| id.to_string()),
                created_at: rollback.created_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(
                CoreError::NotFound("no rollback found for this deployment".into())
                    .error_response(),
            ),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct RollbackStatusQuery {
    pub deployment_id: String,
}

pub fn deployment_history_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env}/deployments",
            get(list_deployments),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env}/rollback/{id}",
            post(rollback_deployment),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env}/rollback-status",
            get(get_rollback_status),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployment_history_response_serializes() {
        let resp = DeploymentHistoryResponse {
            id: "00000000-0000-0000-0000-000000000001".into(),
            environment_id: "00000000-0000-0000-0000-000000000002".into(),
            version: "1.0.0".into(),
            sha: "abc123".into(),
            status: "deployed".into(),
            deployed_by: "00000000-0000-0000-0000-000000000003".into(),
            rollback_of: None,
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("1.0.0"));
        assert!(json.contains("deployed"));
    }

    #[test]
    fn test_rollback_status_query() {
        let q: RollbackStatusQuery =
            serde_json::from_str(r#"{"deployment_id": "00000000-0000-0000-0000-000000000001"}"#)
                .unwrap();
        assert_eq!(q.deployment_id, "00000000-0000-0000-0000-000000000001");
    }
}
