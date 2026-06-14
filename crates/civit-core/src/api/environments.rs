#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_admin};
use crate::error::{CoreError, ErrorResponse};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct EnvironmentResponse {
    pub id: String,
    pub repo_id: String,
    pub name: String,
    pub protection_rules: serde_json::Value,
    pub variables: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateEnvironmentRequest {
    pub name: String,
    pub protection_rules: Option<serde_json::Value>,
    pub variables: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEnvironmentRequest {
    pub name: Option<String>,
    pub protection_rules: Option<serde_json::Value>,
    pub variables: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListEnvironmentsParams {
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

pub async fn list_environments(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Query(params): Query<ListEnvironmentsParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match resolve_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp.into_response(),
    };

    let offset = (params.page.saturating_sub(1) * params.per_page) as i64;
    let rows = sqlx::query_as::<
        _,
        (
            Uuid,
            Uuid,
            String,
            serde_json::Value,
            serde_json::Value,
            chrono::DateTime<chrono::Utc>,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        "SELECT id, repo_id, name, protection_rules, variables, created_at, updated_at \
         FROM environments WHERE repo_id = $1 ORDER BY name LIMIT $2 OFFSET $3",
    )
    .bind(repo_id)
    .bind(params.per_page as i64)
    .bind(offset)
    .fetch_all(pool)
    .await;

    match rows {
        Ok(rows) => {
            let envs: Vec<EnvironmentResponse> = rows
                .into_iter()
                .map(
                    |(id, repo_id, name, protection_rules, variables, created_at, updated_at)| {
                        EnvironmentResponse {
                            id: id.to_string(),
                            repo_id: repo_id.to_string(),
                            name,
                            protection_rules,
                            variables,
                            created_at: created_at.to_rfc3339(),
                            updated_at: updated_at.to_rfc3339(),
                        }
                    },
                )
                .collect();
            (StatusCode::OK, Json(envs)).into_response()
        }
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
    auth: AuthUser,
    Json(req): Json<CreateEnvironmentRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let pool = state.db.pool();
    let repo_id = match resolve_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp.into_response(),
    };

    let protection_rules = req.protection_rules.unwrap_or(serde_json::json!({}));
    let variables = req.variables.unwrap_or(serde_json::json!({}));

    let result = sqlx::query_as::<
        _,
        (
            Uuid,
            Uuid,
            String,
            serde_json::Value,
            serde_json::Value,
            chrono::DateTime<chrono::Utc>,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        "INSERT INTO environments (repo_id, name, protection_rules, variables) \
         VALUES ($1, $2, $3, $4) \
         RETURNING id, repo_id, name, protection_rules, variables, created_at, updated_at",
    )
    .bind(repo_id)
    .bind(&req.name)
    .bind(&protection_rules)
    .bind(&variables)
    .fetch_one(pool)
    .await;

    match result {
        Ok((id, repo_id, name, protection_rules, variables, created_at, updated_at)) => (
            StatusCode::CREATED,
            Json(EnvironmentResponse {
                id: id.to_string(),
                repo_id: repo_id.to_string(),
                name,
                protection_rules,
                variables,
                created_at: created_at.to_rfc3339(),
                updated_at: updated_at.to_rfc3339(),
            }),
        )
            .into_response(),
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

pub async fn update_environment(
    State(state): State<AppState>,
    Path((owner, name, env_id)): Path<(String, String, String)>,
    auth: AuthUser,
    Json(req): Json<UpdateEnvironmentRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let pool = state.db.pool();
    let _repo_id = match resolve_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp.into_response(),
    };

    let eid = match Uuid::parse_str(&env_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid environment id".into()).error_response()),
            )
                .into_response();
        }
    };

    let new_name = req.name.unwrap_or_default();

    let result = sqlx::query_as::<
        _,
        (
            Uuid,
            Uuid,
            String,
            serde_json::Value,
            serde_json::Value,
            chrono::DateTime<chrono::Utc>,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        "UPDATE environments \
         SET name = CASE WHEN $2 != '' THEN $2 ELSE name END, \
             protection_rules = COALESCE($3, protection_rules), \
             variables = COALESCE($4, variables), \
             updated_at = NOW() \
         WHERE id = $1 \
         RETURNING id, repo_id, name, protection_rules, variables, created_at, updated_at",
    )
    .bind(eid)
    .bind(&new_name)
    .bind(&req.protection_rules)
    .bind(&req.variables)
    .fetch_one(pool)
    .await;

    match result {
        Ok((id, repo_id, name, protection_rules, variables, created_at, updated_at)) => (
            StatusCode::OK,
            Json(EnvironmentResponse {
                id: id.to_string(),
                repo_id: repo_id.to_string(),
                name,
                protection_rules,
                variables,
                created_at: created_at.to_rfc3339(),
                updated_at: updated_at.to_rfc3339(),
            }),
        )
            .into_response(),
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

pub async fn delete_environment(
    State(state): State<AppState>,
    Path((owner, name, env_id)): Path<(String, String, String)>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let pool = state.db.pool();
    let _repo_id = match resolve_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp.into_response(),
    };

    let eid = match Uuid::parse_str(&env_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid environment id".into()).error_response()),
            )
                .into_response();
        }
    };

    match sqlx::query("DELETE FROM environments WHERE id = $1")
        .bind(eid)
        .execute(pool)
        .await
    {
        Ok(row) if row.rows_affected() > 0 => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "deleted"})),
        )
            .into_response(),
        Ok(_) => (
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

pub fn environment_routes() -> axum::Router<AppState> {
    use axum::routing::delete;
    axum::Router::new().route(
        "/api/v1/repos/{owner}/{name}/environments/{env_id}",
        delete(delete_environment),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_response_serializes() {
        let resp = EnvironmentResponse {
            id: "00000000-0000-0000-0000-000000000001".into(),
            repo_id: "00000000-0000-0000-0000-000000000002".into(),
            name: "production".into(),
            protection_rules: serde_json::json!({"required_reviewers": 2}),
            variables: serde_json::json!({"REGION": "us-east-1"}),
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("production"));
        assert!(json.contains("us-east-1"));
    }

    #[test]
    fn test_create_request_deserialize() {
        let req: CreateEnvironmentRequest =
            serde_json::from_str(r#"{"name": "staging", "protection_rules": {}, "variables": {}}"#)
                .unwrap();
        assert_eq!(req.name, "staging");
    }

    #[test]
    fn test_update_request_partial() {
        let req: UpdateEnvironmentRequest = serde_json::from_str(r#"{"name": "prod"}"#).unwrap();
        assert_eq!(req.name.as_deref(), Some("prod"));
        assert!(req.protection_rules.is_none());
        assert!(req.variables.is_none());
    }
}
