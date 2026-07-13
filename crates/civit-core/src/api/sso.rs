#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_admin};
use crate::error::CoreError;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ActiveSessionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub session_token_hash: String,
    pub ip_address: String,
    pub user_agent: String,
    pub provider: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_active_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: String,
    pub user_id: String,
    pub ip_address: String,
    pub user_agent: String,
    pub provider: String,
    pub created_at: String,
    pub last_active_at: String,
    pub expires_at: String,
}

impl From<ActiveSessionRow> for SessionResponse {
    fn from(row: ActiveSessionRow) -> Self {
        Self {
            id: row.id.to_string(),
            user_id: row.user_id.to_string(),
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            provider: row.provider,
            created_at: row.created_at.to_rfc3339(),
            last_active_at: row.last_active_at.to_rfc3339(),
            expires_at: row.expires_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct LoginHistoryRow {
    pub id: Uuid,
    pub username: String,
    pub provider: String,
    pub success: bool,
    pub ip_address: String,
    pub user_agent: String,
    pub failure_reason: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct LoginHistoryResponse {
    pub id: String,
    pub username: String,
    pub provider: String,
    pub success: bool,
    pub ip_address: String,
    pub user_agent: String,
    pub failure_reason: Option<String>,
    pub created_at: String,
}

impl From<LoginHistoryRow> for LoginHistoryResponse {
    fn from(row: LoginHistoryRow) -> Self {
        Self {
            id: row.id.to_string(),
            username: row.username,
            provider: row.provider,
            success: row.success,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            failure_reason: row.failure_reason,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct SsoGroupMappingRow {
    pub id: Uuid,
    pub sso_provider: String,
    pub sso_group_name: String,
    pub civitforge_role: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct SsoGroupMappingResponse {
    pub id: String,
    pub sso_provider: String,
    pub sso_group_name: String,
    pub civitforge_role: String,
    pub created_at: String,
}

impl From<SsoGroupMappingRow> for SsoGroupMappingResponse {
    fn from(row: SsoGroupMappingRow) -> Self {
        Self {
            id: row.id.to_string(),
            sso_provider: row.sso_provider,
            sso_group_name: row.sso_group_name,
            civitforge_role: row.civitforge_role,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateSsoGroupMappingRequest {
    pub sso_provider: String,
    pub sso_group_name: String,
    pub civitforge_role: String,
}

// ---------------------------------------------------------------------------
// Session Management
// ---------------------------------------------------------------------------

/// GET /api/v1/admin/sessions – List active sessions
pub async fn list_active_sessions(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let pool = state.db.pool();
    let rows: Vec<ActiveSessionRow> = sqlx::query_as(
        "SELECT id, user_id, session_token_hash, ip_address, user_agent, provider,
                created_at, last_active_at, expires_at
         FROM active_sessions
         WHERE expires_at > NOW()
         ORDER BY last_active_at DESC
         LIMIT 500",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let out: Vec<SessionResponse> = rows.into_iter().map(SessionResponse::from).collect();
    (StatusCode::OK, Json(out)).into_response()
}

/// DELETE /api/v1/admin/sessions/:id – Revoke a session
pub async fn revoke_session(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let session_uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid session id".into()).error_response()),
            )
                .into_response();
        }
    };

    let pool = state.db.pool();
    let result = sqlx::query("DELETE FROM active_sessions WHERE id = $1")
        .bind(session_uuid)
        .execute(pool)
        .await;

    match result {
        Ok(r) => {
            if r.rows_affected() == 0 {
                StatusCode::NOT_FOUND.into_response()
            } else {
                StatusCode::NO_CONTENT.into_response()
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// DELETE /api/v1/admin/sessions – Revoke all sessions for a user
pub async fn revoke_all_sessions(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let pool = state.db.pool();
    let result = sqlx::query("DELETE FROM active_sessions WHERE user_id = $1")
        .bind(auth.user_id.parse::<Uuid>().unwrap_or(Uuid::nil()))
        .execute(pool)
        .await;

    match result {
        Ok(r) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "revoked": r.rows_affected()
            })),
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
// Login History
// ---------------------------------------------------------------------------

/// GET /api/v1/admin/login-history – List recent login attempts
pub async fn list_login_history(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let pool = state.db.pool();
    let rows: Vec<LoginHistoryRow> = sqlx::query_as(
        "SELECT id, username, provider, success, ip_address, user_agent, failure_reason, created_at
         FROM login_history
         ORDER BY created_at DESC
         LIMIT 500",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let out: Vec<LoginHistoryResponse> = rows.into_iter().map(LoginHistoryResponse::from).collect();
    (StatusCode::OK, Json(out)).into_response()
}

// ---------------------------------------------------------------------------
// SSO Group Mapping
// ---------------------------------------------------------------------------

/// GET /api/v1/admin/sso-group-mappings – List SSO group mappings
pub async fn list_sso_group_mappings(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let pool = state.db.pool();
    let rows: Vec<SsoGroupMappingRow> = sqlx::query_as(
        "SELECT id, sso_provider, sso_group_name, civitforge_role, created_at
         FROM sso_group_mappings
         ORDER BY sso_provider, sso_group_name",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let out: Vec<SsoGroupMappingResponse> =
        rows.into_iter().map(SsoGroupMappingResponse::from).collect();
    (StatusCode::OK, Json(out)).into_response()
}

/// POST /api/v1/admin/sso-group-mappings – Create SSO group mapping
pub async fn create_sso_group_mapping(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateSsoGroupMappingRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let valid_roles = ["admin", "member", "guest"];
    if !valid_roles.contains(&req.civitforge_role.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::Config(format!(
                "invalid role: must be one of {:?}",
                valid_roles
            ))
            .error_response()),
        )
            .into_response();
    }

    let pool = state.db.pool();
    let result = sqlx::query_as::<_, SsoGroupMappingRow>(
        r#"INSERT INTO sso_group_mappings (sso_provider, sso_group_name, civitforge_role)
           VALUES ($1, $2, $3)
           ON CONFLICT (sso_provider, sso_group_name) DO UPDATE SET civitforge_role = $3
           RETURNING id, sso_provider, sso_group_name, civitforge_role, created_at"#,
    )
    .bind(&req.sso_provider)
    .bind(&req.sso_group_name)
    .bind(&req.civitforge_role)
    .fetch_one(pool)
    .await;

    match result {
        Ok(row) => (
            StatusCode::CREATED,
            Json(SsoGroupMappingResponse::from(row)),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// DELETE /api/v1/admin/sso-group-mappings/:id
pub async fn delete_sso_group_mapping(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let mapping_uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid mapping id".into()).error_response()),
            )
                .into_response();
        }
    };

    let pool = state.db.pool();
    let result = sqlx::query("DELETE FROM sso_group_mappings WHERE id = $1")
        .bind(mapping_uuid)
        .execute(pool)
        .await;

    match result {
        Ok(r) => {
            if r.rows_affected() == 0 {
                StatusCode::NOT_FOUND.into_response()
            } else {
                StatusCode::NO_CONTENT.into_response()
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub fn sso_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/admin/sessions",
            get(list_active_sessions).delete(revoke_all_sessions),
        )
        .route(
            "/api/v1/admin/sessions/{id}",
            delete(revoke_session),
        )
        .route(
            "/api/v1/admin/login-history",
            get(list_login_history),
        )
        .route(
            "/api/v1/admin/sso-group-mappings",
            get(list_sso_group_mappings).post(create_sso_group_mapping),
        )
        .route(
            "/api/v1/admin/sso-group-mappings/{id}",
            delete(delete_sso_group_mapping),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_response_serialize() {
        let resp = SessionResponse {
            id: "00000000-0000-0000-0000-000000000001".into(),
            user_id: "00000000-0000-0000-0000-000000000002".into(),
            ip_address: "127.0.0.1".into(),
            user_agent: "Mozilla/5.0".into(),
            provider: "saml".into(),
            created_at: "2025-01-01T00:00:00+00:00".into(),
            last_active_at: "2025-01-01T01:00:00+00:00".into(),
            expires_at: "2025-01-08T00:00:00+00:00".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"provider\":\"saml\""));
    }

    #[test]
    fn test_login_history_response_serialize() {
        let resp = LoginHistoryResponse {
            id: "00000000-0000-0000-0000-000000000001".into(),
            username: "alice".into(),
            provider: "saml".into(),
            success: true,
            ip_address: "127.0.0.1".into(),
            user_agent: "Mozilla/5.0".into(),
            failure_reason: None,
            created_at: "2025-01-01T00:00:00+00:00".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"success\":true"));
    }

    #[test]
    fn test_sso_routes_compile() {
        let _ = sso_routes();
    }
}
