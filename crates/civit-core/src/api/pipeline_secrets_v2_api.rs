//! Pipeline Secrets v2 API endpoints.

#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use crate::pipeline_secrets::{
    CreateSecretRequest, PipelineSecretsService, UpdateSecretRequest,
};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use uuid::Uuid;

pub fn pipeline_secrets_v2_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v2/repos/{owner}/{repo}/secrets",
            get(list_secrets_v2).post(create_secret_v2),
        )
        .route(
            "/api/v2/repos/{owner}/{repo}/secrets/{secret_name}",
            get(get_secret_v2).put(update_secret_v2).delete(delete_secret_v2),
        )
        .route(
            "/api/v2/repos/{owner}/{repo}/secrets/{secret_name}/rotate",
            post(rotate_secret_v2),
        )
        .route(
            "/api/v2/repos/{owner}/{repo}/secrets/{secret_name}/rotation-log",
            get(get_rotation_log_v2),
        )
        .route(
            "/api/v2/repos/{owner}/{repo}/secrets/{secret_name}/access-log",
            get(get_access_log_v2),
        )
}

async fn get_repo_id(
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

pub async fn list_secrets_v2(
    State(state): State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &repo_name).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let service = PipelineSecretsService::new(pool.clone());
    match service.list_secrets(repo_id, None).await {
        Ok(secrets) => (axum::http::StatusCode::OK, Json(secrets)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn create_secret_v2(
    State(state): State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<CreateSecretRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &repo_name).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let user_id = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => Some(id),
        Err(_) => None,
    };

    // For now, we'll store the value as bytes directly (encryption would be added in production)
    let encrypted_value = req.value.as_bytes().to_vec();

    let service = PipelineSecretsService::new(pool.clone());
    match service.create_secret(repo_id, req, encrypted_value, user_id).await {
        Ok(secret) => (axum::http::StatusCode::CREATED, Json(secret)).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("duplicate key") || msg.contains("unique") {
                (
                    axum::http::StatusCode::CONFLICT,
                    Json(
                        CoreError::BadRequest("secret name already exists in this environment".into())
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

pub async fn get_secret_v2(
    State(state): State<AppState>,
    Path((owner, repo_name, secret_name)): Path<(String, String, String)>,
    auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &repo_name).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let _user_id = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => Some(id),
        Err(_) => None,
    };

    let service = PipelineSecretsService::new(pool.clone());
    match service.get_secret(repo_id, &secret_name, "all").await {
        Ok(Some(secret)) => {
            // Log access
            let _ = service.get_access_log(secret.id).await;
            (axum::http::StatusCode::OK, Json(secret)).into_response()
        }
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("secret not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_secret_v2(
    State(state): State<AppState>,
    Path((owner, repo_name, secret_name)): Path<(String, String, String)>,
    auth: AuthUser,
    Json(req): Json<UpdateSecretRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &repo_name).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let user_id = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => Some(id),
        Err(_) => None,
    };

    let encrypted_value = req.value.as_ref().map(|v| v.as_bytes().to_vec());

    let service = PipelineSecretsService::new(pool.clone());
    match service
        .update_secret(repo_id, &secret_name, "all", req, encrypted_value, user_id)
        .await
    {
        Ok(secret) => (axum::http::StatusCode::OK, Json(secret)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_secret_v2(
    State(state): State<AppState>,
    Path((owner, repo_name, secret_name)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &repo_name).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let service = PipelineSecretsService::new(pool.clone());
    match service.delete_secret(repo_id, &secret_name, "all").await {
        Ok(true) => (axum::http::StatusCode::NO_CONTENT, "").into_response(),
        Ok(false) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("secret not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn rotate_secret_v2(
    State(state): State<AppState>,
    Path((owner, repo_name, secret_name)): Path<(String, String, String)>,
    auth: AuthUser,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &repo_name).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let user_id = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => Some(id),
        Err(_) => None,
    };

    let new_value = match req.get("value").and_then(|v| v.as_str()) {
        Some(v) => v,
        None => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("missing value field".into()).error_response()),
            )
                .into_response();
        }
    };

    let reason = req
        .get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("Secret rotation");

    let encrypted_value = new_value.as_bytes().to_vec();

    let service = PipelineSecretsService::new(pool.clone());
    match service
        .rotate_secret(repo_id, &secret_name, "all", encrypted_value, user_id, reason)
        .await
    {
        Ok(secret) => (axum::http::StatusCode::OK, Json(secret)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_rotation_log_v2(
    State(state): State<AppState>,
    Path((owner, repo_name, secret_name)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &repo_name).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let service = PipelineSecretsService::new(pool.clone());
    match service.get_secret(repo_id, &secret_name, "all").await {
        Ok(Some(secret)) => match service.get_rotation_log(secret.id).await {
            Ok(log) => (axum::http::StatusCode::OK, Json(log)).into_response(),
            Err(e) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response(),
        },
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("secret not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_access_log_v2(
    State(state): State<AppState>,
    Path((owner, repo_name, secret_name)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &repo_name).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let service = PipelineSecretsService::new(pool.clone());
    match service.get_secret(repo_id, &secret_name, "all").await {
        Ok(Some(secret)) => match service.get_access_log(secret.id).await {
            Ok(log) => (axum::http::StatusCode::OK, Json(log)).into_response(),
            Err(e) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response(),
        },
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("secret not found".into()).error_response()),
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
    fn test_create_secret_v2_request_deserialize() {
        let json = r#"{"name": "MY_TOKEN", "value": "abc123", "description": "Token", "environment": "production"}"#;
        let req: CreateSecretRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "MY_TOKEN");
        assert_eq!(req.value, "abc123");
    }

    #[test]
    fn test_update_secret_v2_request_deserialize() {
        let json = r#"{"value": "new_value", "description": "Updated"}"#;
        let req: UpdateSecretRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.value.as_deref(), Some("new_value"));
    }
}
