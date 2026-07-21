#![forbid(unsafe_code)]

use crate::api::AppState;
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
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize)]
pub struct AuthUser {
    pub user_id: String,
    pub username: String,
    pub role: Role,
    pub org_id: Option<String>,
}

pub struct OptionalAuthUser(pub Option<AuthUser>);

/// Hash a token for comparison (SHA-256, same as tokens.rs).
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Authenticate using a personal access token (PAT).
///
/// PATs are prefixed with `cf_pat_` to distinguish them from JWTs.
/// The raw token is hashed and looked up in the `access_tokens` table.
pub async fn authenticate_with_token(state: &AppState, token: &str) -> Result<AuthUser, CoreError> {
    let token_hash = hash_token(token);
    let (user_id, scopes, token_id) = state.db.validate_pat_token(&token_hash).await?;

    // Check that the token has at least "read" scope (base permission)
    if !scopes.iter().any(|s| s == "read" || s == "admin") {
        return Err(CoreError::Auth("token lacks required scope".into()));
    }

    // Update last_used_at (best-effort)
    let _ = state.db.touch_access_token(token_id).await;

    let user = state.db.get_user_by_id(user_id).await?;
    let role = Role::from_str(&user.role)
        .ok_or_else(|| CoreError::Auth(format!("unknown role: {}", user.role)))?;

    Ok(AuthUser {
        user_id: user.id.to_string(),
        username: user.username,
        role,
        org_id: None,
    })
}

fn extract_auth_user(parts: &Parts, state: &AppState) -> Result<AuthUser, CoreError> {
    // 1) Try Authorization header first (standard)
    if let Some(auth_header) = parts.headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok()) {
        return extract_from_auth_header(auth_header, state);
    }

    // 2) Fallback: try `token` query parameter (needed for EventSource/SSE which cannot send custom headers)
    let uri_str = parts.uri.to_string();
    if let Some(query_start) = uri_str.find('?') {
        let query = &uri_str[query_start + 1..];
        for param in query.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                if key == "token" && !value.is_empty() {
                    let decoded = urlencoding::decode(value)
                        .map_err(|_| CoreError::Auth("invalid token encoding".into()))?;
                    let token = decoded.into_owned();
                    // Try JWT first (synchronous, no DB call needed)
                    if let Ok(claims) = state.jwt_service.validate_token(&token) {
                        let role = Role::from_str(&claims.role)
                            .ok_or_else(|| CoreError::Auth(format!("unknown role: {}", claims.role)))?;
                        return Ok(AuthUser {
                            user_id: claims.sub,
                            username: claims.username,
                            role,
                            org_id: claims.org_id,
                        });
                    }
                    // Try PAT (requires async DB call)
                    if token.starts_with("cf_pat_") {
                        return tokio::task::block_in_place(|| {
                            tokio::runtime::Handle::current()
                                .block_on(async { authenticate_with_token(state, &token).await })
                        });
                    }
                    return Err(CoreError::Auth("invalid token in query parameter".into()));
                }
            }
        }
    }

    Err(CoreError::Auth(
        "missing authorization header or token query parameter".into(),
    ))
}

fn extract_from_auth_header(auth_header: &str, state: &AppState) -> Result<AuthUser, CoreError> {

    // Support Bearer token (JWT or PAT)
    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        // Try PAT first (tokens starting with cf_pat_)
        if token.starts_with("cf_pat_") {
            return tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { authenticate_with_token(state, token).await })
            });
        }

        // Otherwise try JWT
        let claims = state.jwt_service.validate_token(token)?;
        let role = Role::from_str(&claims.role)
            .ok_or_else(|| CoreError::Auth(format!("unknown role: {}", claims.role)))?;
        return Ok(AuthUser {
            user_id: claims.sub,
            username: claims.username,
            role,
            org_id: claims.org_id,
        });
    }

    // Support Basic auth (git clients send username:password or username:token)
    if let Some(basic) = auth_header.strip_prefix("Basic ") {
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(basic)
            .map_err(|_| CoreError::Auth("invalid basic auth encoding".into()))?;
        let creds =
            String::from_utf8(decoded).map_err(|_| CoreError::Auth("invalid basic auth".into()))?;
        let (_username, password) = creds
            .split_once(':')
            .ok_or_else(|| CoreError::Auth("invalid basic auth format".into()))?;

        // Try PAT first (password field starts with cf_pat_)
        if password.starts_with("cf_pat_") {
            return tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { authenticate_with_token(state, password).await })
            });
        }

        // Try as JWT token (password field is a JWT token)
        if let Ok(claims) = state.jwt_service.validate_token(password) {
            let role = Role::from_str(&claims.role)
                .ok_or_else(|| CoreError::Auth(format!("unknown role: {}", claims.role)))?;
            return Ok(AuthUser {
                user_id: claims.sub,
                username: claims.username,
                role,
                org_id: claims.org_id,
            });
        }

        return Err(CoreError::Auth(
            "invalid credentials (use token as password)".into(),
        ));
    }

    Err(CoreError::Auth("invalid authorization scheme".into()))
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
    use crate::config::{AppConfig, SecurityConfig};
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
            security: SecurityConfig::default(),
            tls_cert_path: None,
            tls_key_path: None,
            ui_assets_path: "./crates/civit-ui/dist".into(),
            debug_mode: false,
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

    #[test]
    fn test_hash_token_deterministic() {
        let token = "cf_pat_abc123";
        let h1 = hash_token(token);
        let h2 = hash_token(token);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn test_hash_token_different_inputs() {
        let h1 = hash_token("cf_pat_abc");
        let h2 = hash_token("cf_pat_def");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_pat_token_prefix_detection() {
        let token = "cf_pat_abc123def456";
        assert!(token.starts_with("cf_pat_"));
        let jwt = "eyJhbGciOiJIUzI1NiJ9.test";
        assert!(!jwt.starts_with("cf_pat_"));
    }
}
