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

// ---------------------------------------------------------------------------
// Deployment History V2
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct DeploymentHistoryV2Response {
    pub id: String,
    pub environment_id: String,
    pub version: String,
    pub sha: String,
    pub status: String,
    pub deployed_by: String,
    pub rollback_of: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateDeploymentV2Request {
    pub version: String,
    pub sha: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CompareDeploymentsRequest {
    pub deployment_id_a: String,
    pub deployment_id_b: String,
}

#[derive(Debug, Deserialize)]
pub struct DeploymentAnalyticsParams {
    #[serde(default = "default_analytics_days")]
    pub days: i32,
}

fn default_analytics_days() -> i32 {
    30
}

pub async fn list_deployments_v2(
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

    let rows = sqlx::query_as::<_, (Uuid, Uuid, String, String, String, Uuid, Option<Uuid>, serde_json::Value, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, environment_id, version, sha, status, deployed_by, rollback_of, metadata, created_at \
         FROM environment_deployment_history_v2 \
         WHERE environment_id = $1 \
         ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(env_id)
    .bind(params.per_page as i64)
    .bind(offset)
    .fetch_all(pool)
    .await;

    match rows {
        Ok(rows) => {
            let deps: Vec<DeploymentHistoryV2Response> = rows
                .into_iter()
                .map(|(id, environment_id, version, sha, status, deployed_by, rollback_of, metadata, created_at)| {
                    DeploymentHistoryV2Response {
                        id: id.to_string(),
                        environment_id: environment_id.to_string(),
                        version,
                        sha,
                        status,
                        deployed_by: deployed_by.to_string(),
                        rollback_of: rollback_of.map(|id| id.to_string()),
                        metadata,
                        created_at: created_at.to_rfc3339(),
                    }
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

pub async fn create_deployment_v2(
    State(state): State<AppState>,
    Path((owner, name, env)): Path<(String, String, String)>,
    auth: AuthUser,
    Json(req): Json<CreateDeploymentV2Request>,
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

    let metadata = req.metadata.unwrap_or(serde_json::json!({}));

    let result = sqlx::query_as::<_, (Uuid, Uuid, String, String, String, Uuid, Option<Uuid>, serde_json::Value, chrono::DateTime<chrono::Utc>)>(
        "INSERT INTO environment_deployment_history_v2 (environment_id, version, sha, deployed_by, metadata) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, environment_id, version, sha, status, deployed_by, rollback_of, metadata, created_at",
    )
    .bind(env_id)
    .bind(&req.version)
    .bind(&req.sha)
    .bind(user_id)
    .bind(&metadata)
    .fetch_one(pool)
    .await;

    match result {
        Ok((id, environment_id, version, sha, status, deployed_by, rollback_of, metadata, created_at)) => (
            StatusCode::CREATED,
            Json(DeploymentHistoryV2Response {
                id: id.to_string(),
                environment_id: environment_id.to_string(),
                version,
                sha,
                status,
                deployed_by: deployed_by.to_string(),
                rollback_of: rollback_of.map(|id| id.to_string()),
                metadata,
                created_at: created_at.to_rfc3339(),
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

pub async fn compare_deployments(
    State(state): State<AppState>,
    Path((owner, name, env)): Path<(String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<CompareDeploymentsRequest>,
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

    let did_a = match Uuid::parse_str(&req.deployment_id_a) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid deployment id A".into()).error_response()),
            )
                .into_response();
        }
    };

    let did_b = match Uuid::parse_str(&req.deployment_id_b) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid deployment id B".into()).error_response()),
            )
                .into_response();
        }
    };

    let row_a = sqlx::query_as::<_, (String, String, String, serde_json::Value, chrono::DateTime<chrono::Utc>)>(
        "SELECT version, sha, status, metadata, created_at FROM environment_deployment_history_v2 WHERE id = $1",
    )
    .bind(did_a)
    .fetch_optional(pool)
    .await;

    let row_b = sqlx::query_as::<_, (String, String, String, serde_json::Value, chrono::DateTime<chrono::Utc>)>(
        "SELECT version, sha, status, metadata, created_at FROM environment_deployment_history_v2 WHERE id = $1",
    )
    .bind(did_b)
    .fetch_optional(pool)
    .await;

    match (row_a, row_b) {
        (Ok(Some(a)), Ok(Some(b))) => {
            let comparison = serde_json::json!({
                "deployment_a": {
                    "id": did_a.to_string(),
                    "version": a.0,
                    "sha": a.1,
                    "status": a.2,
                    "metadata": a.3,
                    "created_at": a.4.to_rfc3339()
                },
                "deployment_b": {
                    "id": did_b.to_string(),
                    "version": b.0,
                    "sha": b.1,
                    "status": b.2,
                    "metadata": b.3,
                    "created_at": b.4.to_rfc3339()
                },
                "sha_changed": a.1 != b.1,
                "version_changed": a.0 != b.0,
                "time_diff_seconds": (b.4 - a.4).num_seconds()
            });
            (StatusCode::OK, Json(comparison)).into_response()
        }
        (Ok(None), _) | (_, Ok(None)) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("one or both deployments not found".into()).error_response()),
        )
            .into_response(),
        (Err(e), _) | (_, Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_deployment_analytics(
    State(state): State<AppState>,
    Path((owner, name, env)): Path<(String, String, String)>,
    _auth: AuthUser,
    Query(params): Query<DeploymentAnalyticsParams>,
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

    let row = sqlx::query_as::<_, (Option<i64>, Option<i64>, Option<i64>, Option<i64>)>(
        "SELECT \
            COUNT(*), \
            COUNT(*) FILTER (WHERE status = 'deployed'), \
            COUNT(*) FILTER (WHERE status = 'failed'), \
            COUNT(*) FILTER (WHERE rollback_of IS NOT NULL) \
         FROM environment_deployment_history_v2 \
         WHERE environment_id = $1 AND created_at >= NOW() - INTERVAL '1 day' * $2",
    )
    .bind(env_id)
    .bind(params.days)
    .fetch_one(pool)
    .await;

    match row {
        Ok((total, successful, failed, rollbacks)) => {
            let total = total.unwrap_or(0);
            let successful = successful.unwrap_or(0);
            let failed = failed.unwrap_or(0);
            let rollbacks = rollbacks.unwrap_or(0);
            let success_rate = if total > 0 {
                successful as f64 / total as f64
            } else {
                0.0
            };

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "environment_id": env_id.to_string(),
                    "period_days": params.days,
                    "total_deployments": total,
                    "successful_deployments": successful,
                    "failed_deployments": failed,
                    "rollback_count": rollbacks,
                    "success_rate": success_rate
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub fn deployment_history_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/rollback/{id}",
            post(rollback_deployment),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/rollback-status",
            get(get_rollback_status),
        )
        // V2 endpoints
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/deployments/v2",
            get(list_deployments_v2).post(create_deployment_v2),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/deployments/compare",
            post(compare_deployments),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/environments/{env_id}/deployments/analytics",
            get(get_deployment_analytics),
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

    #[test]
    fn test_deployment_history_v2_response_serializes() {
        let resp = DeploymentHistoryV2Response {
            id: "00000000-0000-0000-0000-000000000001".into(),
            environment_id: "00000000-0000-0000-0000-000000000002".into(),
            version: "2.0.0".into(),
            sha: "def456".into(),
            status: "deployed".into(),
            deployed_by: "00000000-0000-0000-0000-000000000003".into(),
            rollback_of: None,
            metadata: serde_json::json!({"environment": "production"}),
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("2.0.0"));
        assert!(json.contains("production"));
    }

    #[test]
    fn test_create_deployment_v2_request() {
        let req: CreateDeploymentV2Request = serde_json::from_str(
            r#"{"version": "1.0.0", "sha": "abc123", "metadata": {"env": "prod"}}"#,
        )
        .unwrap();
        assert_eq!(req.version, "1.0.0");
        assert_eq!(req.sha, "abc123");
    }
}
