//! Environment Variables v3 API endpoints.

#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use crate::environment_variables::{
    CreateVariableRequest, EnvironmentVariablesService, UpdateVariableRequest,
};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post, put, delete};
use axum::{Json, Router};
use uuid::Uuid;

pub fn environment_variables_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v2/repos/{owner}/{repo}/environments/{env_id}/variables",
            get(list_variables_v3).post(create_variable_v3),
        )
        .route(
            "/api/v2/repos/{owner}/{repo}/environments/{env_id}/variables/{var_name}",
            get(get_variable_v3).put(update_variable_v3).delete(delete_variable_v3),
        )
        .route(
            "/api/v2/repos/{owner}/{repo}/environments/{env_id}/inheritance",
            get(list_inheritances_v3).post(add_inheritance_v3),
        )
        .route(
            "/api/v2/repos/{owner}/{repo}/environments/{env_id}/inheritance/{parent_id}",
            delete(remove_inheritance_v3),
        )
}

async fn resolve_repo_id(
    pool: &sqlx::PgPool,
    owner: &str,
    name: &str,
) -> std::result::Result<Uuid, (axum::http::StatusCode, axum::response::Response)> {
    let owner_uuid = if let Ok(id) = Uuid::parse_str(owner) {
        id
    } else {
        let user_row = sqlx::query_as::<_, (Uuid,)>("SELECT id FROM users WHERE username = $1")
            .bind(owner)
            .fetch_optional(pool)
            .await;

        match user_row {
            Ok(Some((id,))) => id,
            Ok(None) => {
                return Err((
                    axum::http::StatusCode::NOT_FOUND,
                    Json(CoreError::NotFound("user not found".into()).error_response())
                        .into_response(),
                ));
            }
            Err(e) => {
                return Err((
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CoreError::Database(e.to_string()).error_response())
                        .into_response(),
                ));
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
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response())
                .into_response(),
        )),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response())
                .into_response(),
        )),
    }
}

pub async fn list_variables_v3(
    State(state): State<AppState>,
    Path((_owner, _repo, env_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let eid = match Uuid::parse_str(&env_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid environment ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let service = EnvironmentVariablesService::new(pool.clone());
    match service.list_variables(eid, true).await {
        Ok(variables) => (axum::http::StatusCode::OK, Json(variables)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn create_variable_v3(
    State(state): State<AppState>,
    Path((_owner, _repo, env_id)): Path<(String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<CreateVariableRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let eid = match Uuid::parse_str(&env_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid environment ID".into()).error_response()),
            )
                .into_response();
        }
    };

    // Validate variable name
    if let Err(e) = EnvironmentVariablesService::validate_variable_name(&req.name).await {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(CoreError::BadRequest(e).error_response()),
        )
            .into_response();
    }

    // Validate variable value
    if let Err(e) = EnvironmentVariablesService::validate_variable_value(&req.value).await {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(CoreError::BadRequest(e).error_response()),
        )
            .into_response();
    }

    let service = EnvironmentVariablesService::new(pool.clone());
    match service.create_variable(eid, req).await {
        Ok(variable) => (axum::http::StatusCode::CREATED, Json(variable)).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("duplicate key") || msg.contains("unique") {
                (
                    axum::http::StatusCode::CONFLICT,
                    Json(
                        CoreError::BadRequest("variable name already exists in this environment".into())
                            .error_response(),
                    ),
                )
                    .into_response()
            } else {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CoreError::Database(msg).error_response()),
                )
                    .into_response()
            }
        }
    }
}

pub async fn get_variable_v3(
    State(state): State<AppState>,
    Path((_owner, _repo, env_id, var_name)): Path<(String, String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let eid = match Uuid::parse_str(&env_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid environment ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let service = EnvironmentVariablesService::new(pool.clone());
    match service.get_variable(eid, &var_name).await {
        Ok(Some(variable)) => (axum::http::StatusCode::OK, Json(variable)).into_response(),
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("variable not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_variable_v3(
    State(state): State<AppState>,
    Path((_owner, _repo, env_id, var_name)): Path<(String, String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<UpdateVariableRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let eid = match Uuid::parse_str(&env_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid environment ID".into()).error_response()),
            )
                .into_response();
        }
    };

    // Validate variable value if provided
    if let Some(ref value) = req.value {
        if let Err(e) = EnvironmentVariablesService::validate_variable_value(value).await {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest(e).error_response()),
            )
                .into_response();
        }
    }

    let service = EnvironmentVariablesService::new(pool.clone());
    match service.update_variable(eid, &var_name, req).await {
        Ok(variable) => (axum::http::StatusCode::OK, Json(variable)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_variable_v3(
    State(state): State<AppState>,
    Path((_owner, _repo, env_id, var_name)): Path<(String, String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let eid = match Uuid::parse_str(&env_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid environment ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let service = EnvironmentVariablesService::new(pool.clone());
    match service.delete_variable(eid, &var_name).await {
        Ok(true) => (axum::http::StatusCode::NO_CONTENT, "").into_response(),
        Ok(false) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("variable not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_inheritances_v3(
    State(state): State<AppState>,
    Path((_owner, _repo, env_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let eid = match Uuid::parse_str(&env_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid environment ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let service = EnvironmentVariablesService::new(pool.clone());
    match service.list_inheritances(eid).await {
        Ok(inheritances) => (axum::http::StatusCode::OK, Json(inheritances)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn add_inheritance_v3(
    State(state): State<AppState>,
    Path((_owner, _repo, env_id)): Path<(String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let child_id = match Uuid::parse_str(&env_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid environment ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let parent_id = match req.get("parent_env_id").and_then(|v| v.as_str()) {
        Some(id_str) => match Uuid::parse_str(id_str) {
            Ok(id) => id,
            Err(_) => {
                return (
                    axum::http::StatusCode::BAD_REQUEST,
                    Json(CoreError::BadRequest("invalid parent environment ID".into()).error_response()),
                )
                    .into_response();
            }
        },
        None => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("missing parent_env_id".into()).error_response()),
            )
                .into_response();
        }
    };

    let service = EnvironmentVariablesService::new(pool.clone());
    match service.add_inheritance(child_id, parent_id).await {
        Ok(inheritance) => (axum::http::StatusCode::CREATED, Json(inheritance)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn remove_inheritance_v3(
    State(state): State<AppState>,
    Path((_owner, _repo, env_id, parent_id)): Path<(String, String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let child_id = match Uuid::parse_str(&env_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid environment ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let parent_id = match Uuid::parse_str(&parent_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid parent environment ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let service = EnvironmentVariablesService::new(pool.clone());
    match service.remove_inheritance(child_id, parent_id).await {
        Ok(true) => (axum::http::StatusCode::NO_CONTENT, "").into_response(),
        Ok(false) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("inheritance not found".into()).error_response()),
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
    fn test_create_variable_request_deserialize() {
        let json = r#"{"name": "DATABASE_URL", "value": "postgres://localhost/mydb", "encrypted": false}"#;
        let req: CreateVariableRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "DATABASE_URL");
        assert_eq!(req.value, "postgres://localhost/mydb");
    }

    #[test]
    fn test_update_variable_request_deserialize() {
        let json = r#"{"value": "new_value", "encrypted": true}"#;
        let req: UpdateVariableRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.value.as_deref(), Some("new_value"));
        assert_eq!(req.encrypted, Some(true));
    }
}
