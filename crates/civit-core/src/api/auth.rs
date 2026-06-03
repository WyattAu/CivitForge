#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::auth::jwt::JwtService;
use crate::auth::permission_engine::PermissionEngine;
use crate::auth::rbac::Role;
use crate::error::{CoreError, ErrorResponse};
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::response::Json;
use civit_shared::id::{RepoId, UserId};
use civit_shared::permissions::{Action, PermissionCheck, Resource};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AuthUser {
    pub user_id: String,
    pub username: String,
    pub role: Role,
    pub org_id: Option<String>,
}

pub struct OptionalAuthUser(pub Option<AuthUser>);

fn extract_auth_user(parts: &Parts, state: &AppState) -> Result<AuthUser, CoreError> {
    let auth_header = parts
        .headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| CoreError::Auth("missing authorization header".into()))?;

    let token = JwtService::extract_bearer(auth_header)
        .ok_or_else(|| CoreError::Auth("invalid authorization scheme".into()))?;

    let claims = state.jwt_service.validate_token(token)?;

    let role = Role::from_str(&claims.role)
        .ok_or_else(|| CoreError::Auth(format!("unknown role: {}", claims.role)))?;

    Ok(AuthUser {
        user_id: claims.sub,
        username: claims.username,
        role,
        org_id: claims.org_id,
    })
}

fn to_rejection(e: CoreError) -> (StatusCode, Json<ErrorResponse>) {
    (e.status_code(), Json(e.error_response()))
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, Json<ErrorResponse>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        extract_auth_user(parts, state).map_err(to_rejection)
    }
}

impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(OptionalAuthUser(extract_auth_user(parts, state).ok()))
    }
}

// ---------------------------------------------------------------------------
// Permission check helpers for route handlers
// ---------------------------------------------------------------------------

/// Check that the authenticated user has permission on a resource.
/// Returns `Ok(())` on success, or a (StatusCode, Json) rejection tuple on failure.
pub async fn require_permission(
    state: &AppState,
    user: &AuthUser,
    resource: Resource,
    action: Action,
    repo_id: Option<RepoId>,
    org_id: Option<uuid::Uuid>,
    branch_name: Option<&str>,
) -> Result<PermissionCheck, (StatusCode, Json<ErrorResponse>)> {
    let repo_id_val = repo_id.map(|r| r.get());
    let user_id = uuid::Uuid::parse_str(&user.user_id).unwrap_or(uuid::Uuid::nil());
    PermissionEngine::check(
        state.db.pool(),
        UserId::new(user_id),
        resource,
        action,
        repo_id_val.map(RepoId::new),
        org_id,
        branch_name,
    )
    .await
    .map_err(to_rejection)
}

/// Check that the authenticated user is an admin.
/// Returns `Ok(())` on success, or a (StatusCode, Json) rejection tuple on failure.
pub fn require_admin(user: &AuthUser) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    if user.role == Role::Admin {
        Ok(())
    } else {
        Err(to_rejection(CoreError::Forbidden(
            "admin access required".into(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use axum::http::Request;

    fn test_app_state() -> AppState {
        let config = AppConfig {
            host: "127.0.0.1".into(),
            port: 8080,
            database_url: "postgres://localhost/test".into(),
            redis_url: "redis://localhost:6379".into(),
            jwt_secret: "test-secret-key-32bytes-minimums".into(),
            jwt_expiry_hours: 24,
            federation_enabled: false,
            federation_instance_id: "test".into(),
            federation_instance_domain: "localhost".into(),
            storage_path: "/tmp/repos".into(),
            cors_allowed_origins: Vec::new(),
            rate_limit_max_requests: None,
            rate_limit_window_secs: None,
            tls_cert_path: None,
            tls_key_path: None,
        };
        let opts = sqlx::postgres::PgPoolOptions::new().max_connections(1);
        let pool = opts.connect_lazy("postgres://localhost/test").unwrap();
        AppState::new(config, pool)
    }

    #[test]
    fn test_auth_user_field_access() {
        let user = AuthUser {
            user_id: "u-1".into(),
            username: "alice".into(),
            role: Role::Admin,
            org_id: Some("org-1".into()),
        };
        assert_eq!(user.user_id, "u-1");
        assert_eq!(user.username, "alice");
        assert_eq!(user.role, Role::Admin);
        assert_eq!(user.org_id.as_deref(), Some("org-1"));
    }

    #[test]
    fn test_auth_user_no_org() {
        let user = AuthUser {
            user_id: "u-2".into(),
            username: "bob".into(),
            role: Role::Guest,
            org_id: None,
        };
        assert!(user.org_id.is_none());
        assert_eq!(user.role, Role::Guest);
    }

    #[test]
    fn test_auth_user_serialization() {
        let user = AuthUser {
            user_id: "u-1".into(),
            username: "alice".into(),
            role: Role::Member,
            org_id: None,
        };
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("\"user_id\":\"u-1\""));
        assert!(json.contains("\"username\":\"alice\""));
        assert!(json.contains("\"role\":\"Member\""));
        assert!(json.contains("\"org_id\":null"));
    }

    #[test]
    fn test_auth_user_with_org_serialization() {
        let user = AuthUser {
            user_id: "u-1".into(),
            username: "alice".into(),
            role: Role::Admin,
            org_id: Some("org-1".into()),
        };
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("\"org_id\":\"org-1\""));
    }

    #[test]
    fn test_optional_auth_user_wraps_none() {
        let opt = OptionalAuthUser(None);
        assert!(opt.0.is_none());
    }

    #[test]
    fn test_optional_auth_user_wraps_some() {
        let user = AuthUser {
            user_id: "u-1".into(),
            username: "alice".into(),
            role: Role::Admin,
            org_id: None,
        };
        let opt = OptionalAuthUser(Some(user));
        assert!(opt.0.is_some());
        assert_eq!(opt.0.as_ref().unwrap().username, "alice");
    }

    #[test]
    fn test_to_rejection_status_code() {
        let rejection = to_rejection(CoreError::Auth("missing header".into()));
        assert_eq!(rejection.0, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_to_rejection_body_serializable() {
        let rejection = to_rejection(CoreError::NotFound("user".into()));
        let resp = CoreError::NotFound("user".into()).error_response();
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("not found: user"));
        let _ = rejection;
    }

    #[tokio::test]
    async fn test_missing_header_returns_unauthorized() {
        let state = test_app_state();
        let req = Request::builder().body(()).unwrap();
        let (parts, _) = req.into_parts();
        let result = extract_auth_user(&parts, &state);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status_code(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_invalid_scheme_returns_unauthorized() {
        let state = test_app_state();
        let req = Request::builder()
            .header(AUTHORIZATION, "Basic abc123")
            .body(())
            .unwrap();
        let (parts, _) = req.into_parts();
        let result = extract_auth_user(&parts, &state);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status_code(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_invalid_token_returns_unauthorized() {
        let state = test_app_state();
        let req = Request::builder()
            .header(AUTHORIZATION, "Bearer invalid.token.here")
            .body(())
            .unwrap();
        let (parts, _) = req.into_parts();
        let result = extract_auth_user(&parts, &state);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status_code(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_valid_token_succeeds() {
        let state = test_app_state();
        let token = state
            .jwt_service
            .generate_token("user-1", "alice", "admin", Some("org-1"))
            .unwrap();
        let req = Request::builder()
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .body(())
            .unwrap();
        let (parts, _) = req.into_parts();
        let result = extract_auth_user(&parts, &state);
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.user_id, "user-1");
        assert_eq!(user.username, "alice");
        assert_eq!(user.role, Role::Admin);
        assert_eq!(user.org_id.as_deref(), Some("org-1"));
    }

    #[tokio::test]
    async fn test_optional_auth_no_header_passes() {
        let state = test_app_state();
        let req = Request::builder().body(()).unwrap();
        let (parts, _) = req.into_parts();
        assert!(extract_auth_user(&parts, &state).is_err());
    }

    #[tokio::test]
    async fn test_optional_auth_valid_token_succeeds() {
        let state = test_app_state();
        let token = state
            .jwt_service
            .generate_token("user-2", "bob", "guest", None)
            .unwrap();
        let req = Request::builder()
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .body(())
            .unwrap();
        let (parts, _) = req.into_parts();
        let result = extract_auth_user(&parts, &state);
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.username, "bob");
        assert_eq!(user.role, Role::Guest);
        assert!(user.org_id.is_none());
    }
}
