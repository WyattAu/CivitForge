#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::api::users::UserResponse;
use crate::error::CoreError;
use crate::ldap::LdapAuth;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub email: String,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyEmailResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct LdapSyncRequest {
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct LdapSyncResponse {
    pub status: String,
    pub groups: Vec<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub user_id: String,
    pub username: String,
    pub role: String,
    pub org_id: Option<String>,
}

impl From<AuthUser> for MeResponse {
    fn from(u: AuthUser) -> Self {
        Self {
            user_id: u.user_id,
            username: u.username,
            role: u.role.as_str().to_string(),
            org_id: u.org_id,
        }
    }
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    match do_register(&state, req).await {
        Ok(resp) => (StatusCode::CREATED, Json(resp)).into_response(),
        Err(e) => (e.status_code(), Json(e.error_response())).into_response(),
    }
}

async fn do_register(
    state: &AppState,
    req: RegisterRequest,
) -> crate::error::Result<LoginResponse> {
    if req.username.is_empty() {
        return Err(CoreError::Auth("Username is required".into()));
    }
    if req.email.is_empty() {
        return Err(CoreError::Auth("Email is required".into()));
    }
    let violations =
        crate::api::password::validate_password_policy(&req.password, &state.config.security);
    if !violations.is_empty() {
        return Err(CoreError::Auth(violations.join("; ")));
    }

    // Check if username or email already exists
    if state.db.get_user_by_username(&req.username).await.is_ok() {
        return Err(CoreError::Auth("Username already taken".into()));
    }
    if state.db.get_user_by_email(&req.email).await.is_ok() {
        return Err(CoreError::Auth("Email already registered".into()));
    }

    let password_hash = civit_auth::password::hash_password(&req.password)?;

    let user = state
        .db
        .create_user(
            &req.username,
            &req.email,
            if req.display_name.is_empty() {
                &req.username
            } else {
                &req.display_name
            },
            "member",
            &password_hash,
        )
        .await?;

    // Generate and store a 6-digit verification code
    let verification_code = generate_verification_code();
    let expires_at = Utc::now() + Duration::minutes(15);
    if let Err(e) = state
        .db
        .store_verification_code(user.id, &req.email, &verification_code, expires_at)
        .await
    {
        tracing::warn!("Failed to store verification code: {e}");
    }

    if state.config.debug_mode {
        tracing::info!(
            email = %req.email,
            code = %verification_code,
            "Email verification code (dev mode)"
        );
    } else {
        tracing::info!(
            email = %req.email,
            "Email verification code generated (email sending not yet implemented)"
        );
    }

    let token =
        state
            .jwt_service
            .generate_token(&user.id.to_string(), &user.username, &user.role, None)?;

    Ok(LoginResponse {
        token,
        user: UserResponse::from(user),
    })
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    match do_login(&state, req).await {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => (e.status_code(), Json(e.error_response())).into_response(),
    }
}

async fn do_login(state: &AppState, req: LoginRequest) -> crate::error::Result<LoginResponse> {
    if req.username.is_empty() {
        return Err(CoreError::Auth("Username is required".into()));
    }
    if req.password.is_empty() {
        return Err(CoreError::Auth("Password is required".into()));
    }

    // Check lockout: count recent failures
    let sec = &state.config.security;
    let recent_failures = state
        .db
        .count_recent_failed_logins(&req.username, sec.login_lockout_secs)
        .await
        .unwrap_or(0);

    if recent_failures >= sec.login_max_attempts as i64 {
        return Err(CoreError::TooManyRequests(format!(
            "Account temporarily locked due to too many failed login attempts. Try again in {} seconds.",
            sec.login_lockout_secs
        )));
    }

    // Try LDAP authentication first if enabled
    if sec.ldap_enabled {
        let ldap_config: crate::ldap::LdapConfig = sec.into();
        match LdapAuth::authenticate(&ldap_config, &req.username, &req.password).await {
            Ok(ldap_info) => {
                // Auto-provision or fetch the user
                let user = match state.db.get_user_by_username(&ldap_info.username).await {
                    Ok(u) => u,
                    Err(_) => {
                        // Create new user from LDAP
                        state
                            .db
                            .create_user(
                                &ldap_info.username,
                                &ldap_info.email,
                                &ldap_info.display_name,
                                "member",
                                "", // No local password for LDAP users
                            )
                            .await?
                    }
                };

                let _ = state
                    .db
                    .record_login_attempt(&req.username, "ldap", true)
                    .await;
                let _ = state.db.clear_login_attempts(&req.username).await;

                let token = state.jwt_service.generate_token(
                    &user.id.to_string(),
                    &user.username,
                    &user.role,
                    None,
                )?;

                return Ok(LoginResponse {
                    token,
                    user: UserResponse::from(user),
                });
            }
            Err(_) => {
                // Fall through to local auth on LDAP failure
            }
        }
    }

    let user = match state.db.get_user_by_username(&req.username).await {
        Ok(u) => u,
        Err(_) => {
            let _ = state
                .db
                .record_login_attempt(&req.username, "unknown", false)
                .await;
            return Err(CoreError::Auth("Invalid username or password".into()));
        }
    };

    let stored_hash = match state.db.get_password_hash(user.id).await {
        Ok(Some(h)) if !h.is_empty() => h,
        _ => {
            return Err(CoreError::Auth("Invalid username or password".into()));
        }
    };

    if !civit_auth::password::verify_password(&req.password, &stored_hash) {
        let _ = state
            .db
            .record_login_attempt(&req.username, "unknown", false)
            .await;
        return Err(CoreError::Auth("Invalid username or password".into()));
    }

    // Record success and clear failures
    let _ = state
        .db
        .record_login_attempt(&req.username, "unknown", true)
        .await;
    let _ = state.db.clear_login_attempts(&req.username).await;

    let token =
        state
            .jwt_service
            .generate_token(&user.id.to_string(), &user.username, &user.role, None)?;

    Ok(LoginResponse {
        token,
        user: UserResponse::from(user),
    })
}

pub async fn me(auth: AuthUser) -> impl IntoResponse {
    (StatusCode::OK, Json(MeResponse::from(auth))).into_response()
}

pub async fn refresh(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let auth_header = match headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| crate::auth::jwt::JwtService::extract_bearer(v))
    {
        Some(h) => h.to_string(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(CoreError::Forbidden("missing authorization header".into()).error_response()),
            )
                .into_response();
        }
    };

    let claims = match state.jwt_service.validate_token(&auth_header) {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(CoreError::Forbidden("invalid or expired token".into()).error_response()),
            )
                .into_response();
        }
    };

    let user_id = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(
                    CoreError::Forbidden("invalid user id in token claims".into()).error_response(),
                ),
            )
                .into_response();
        }
    };

    let user = match state.db.get_user_by_id(user_id).await {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("user not found".into()).error_response()),
            )
                .into_response();
        }
    };

    let token = match state.jwt_service.generate_token(
        &user.id.to_string(),
        &user.username,
        &user.role,
        None,
    ) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Internal(format!("token generation failed: {e}")).error_response()),
            )
                .into_response();
        }
    };

    (StatusCode::OK, Json(RefreshResponse { token })).into_response()
}

pub async fn ldap_sync(
    State(state): State<AppState>,
    Json(req): Json<LdapSyncRequest>,
) -> impl IntoResponse {
    match do_ldap_sync(&state, req).await {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => (e.status_code(), Json(e.error_response())).into_response(),
    }
}

async fn do_ldap_sync(
    state: &AppState,
    req: LdapSyncRequest,
) -> crate::error::Result<LdapSyncResponse> {
    if req.username.is_empty() {
        return Err(CoreError::Auth("Username is required".into()));
    }

    let sec = &state.config.security;
    if !sec.ldap_enabled {
        return Err(CoreError::Auth("LDAP is not enabled".into()));
    }

    let ldap_config: crate::ldap::LdapConfig = sec.into();
    let groups = LdapAuth::sync_groups(&ldap_config, &req.username).await?;

    Ok(LdapSyncResponse {
        status: "ok".into(),
        groups: groups.clone(),
        message: format!("Synced {} LDAP groups", groups.len()),
    })
}

/// Generate a random 6-digit verification code.
fn generate_verification_code() -> String {
    let code: u32 = rand::random::<u32>() % 1_000_000;
    format!("{code:06}")
}

/// Verify email with a code sent during registration.
pub async fn verify_email(
    State(state): State<AppState>,
    Json(req): Json<VerifyEmailRequest>,
) -> impl IntoResponse {
    match do_verify_email(&state, req).await {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => (e.status_code(), Json(e.error_response())).into_response(),
    }
}

async fn do_verify_email(
    state: &AppState,
    req: VerifyEmailRequest,
) -> crate::error::Result<VerifyEmailResponse> {
    if req.email.is_empty() || req.code.is_empty() {
        return Err(CoreError::Auth("Email and code are required".into()));
    }

    let code_record = state
        .db
        .validate_verification_code(&req.email, &req.code)
        .await?;

    // Mark the code as used
    state.db.mark_verification_code_used(code_record.id).await?;

    // Set email_verified = true
    state.db.set_email_verified(code_record.user_id).await?;

    Ok(VerifyEmailResponse {
        status: "ok".into(),
        message: "Email verified successfully".into(),
    })
}

#[allow(dead_code)]
fn validate_input(username: &str, email: &str, display_name: &str) -> Result<(), CoreError> {
    if username.is_empty() || username.len() > 64 {
        return Err(CoreError::Config("username must be 1-64 characters".into()));
    }
    if email.len() > 254 {
        return Err(CoreError::Config(
            "email must be at most 254 characters".into(),
        ));
    }
    if display_name.len() > 256 {
        return Err(CoreError::Config(
            "display_name must be at most 256 characters".into(),
        ));
    }

    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(CoreError::Config(
            "username must contain only alphanumeric characters, hyphens, and underscores".into(),
        ));
    }

    if !email.contains('@') || !email.contains('.') {
        return Err(CoreError::Config("invalid email format".into()));
    }

    for s in [username, email, display_name] {
        for ch in s.chars() {
            if ch.is_control() || ch == '\u{202E}' || ch == '\u{200B}' {
                return Err(CoreError::Config(
                    "input contains invalid characters".into(),
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::rbac::Role;

    #[test]
    fn test_validate_input_success() {
        assert!(validate_input("alice", "alice@example.com", "Alice").is_ok());
        assert!(validate_input("user-name", "a@b.co", "Bob").is_ok());
        assert!(validate_input("user_name", "test@domain.org", "C D").is_ok());
    }

    #[test]
    fn test_validate_input_empty_username() {
        assert!(validate_input("", "a@b.co", "Alice").is_err());
    }

    #[test]
    fn test_validate_input_long_username() {
        let long = "a".repeat(65);
        assert!(validate_input(&long, "a@b.co", "Alice").is_err());
    }

    #[test]
    fn test_validate_input_long_email() {
        let long = "a".repeat(255) + "@b.co";
        assert!(validate_input("alice", &long, "Alice").is_err());
    }

    #[test]
    fn test_validate_input_long_display_name() {
        let long = "A".repeat(257);
        assert!(validate_input("alice", "a@b.co", &long).is_err());
    }

    #[test]
    fn test_validate_input_invalid_username_chars() {
        assert!(validate_input("alice!", "a@b.co", "Alice").is_err());
        assert!(validate_input("alice.", "a@b.co", "Alice").is_err());
        assert!(validate_input("alice@", "a@b.co", "Alice").is_err());
    }

    #[test]
    fn test_validate_input_invalid_email() {
        assert!(validate_input("alice", "noat", "Alice").is_err());
        assert!(validate_input("alice", "nodot@", "Alice").is_err());
    }

    #[test]
    fn test_validate_input_control_chars() {
        assert!(validate_input("alice\n", "a@b.co", "Alice").is_err());
        assert!(validate_input("alice", "a@b.co\n", "Alice").is_err());
        assert!(validate_input("alice", "a@b.co", "Alice\0").is_err());
    }

    #[test]
    fn test_validate_input_rtl_override() {
        assert!(validate_input("\u{202E}", "a@b.co", "Alice").is_err());
    }

    #[test]
    fn test_validate_input_zero_width() {
        assert!(validate_input("alice", "a@b.co", "\u{200B}").is_err());
    }

    #[test]
    fn test_login_request_parse() {
        let json = r#"{"username":"alice","password":"secret123"}"#;
        let req: LoginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.username, "alice");
        assert_eq!(req.password, "secret123");
    }

    #[test]
    fn test_register_request_parse() {
        let json = r#"{"username":"alice","email":"alice@example.com","display_name":"Alice","password":"secret123"}"#;
        let req: RegisterRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.username, "alice");
        assert_eq!(req.email, "alice@example.com");
        assert_eq!(req.display_name, "Alice");
        assert_eq!(req.password, "secret123");
    }

    #[test]
    fn test_login_request_missing_fields() {
        let json = r#"{"username":"alice"}"#;
        let result = serde_json::from_str::<LoginRequest>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_me_response_from_auth_user() {
        let auth = AuthUser {
            user_id: "u-1".into(),
            username: "alice".into(),
            role: Role::Admin,
            org_id: Some("org-1".into()),
        };
        let resp = MeResponse::from(auth);
        assert_eq!(resp.user_id, "u-1");
        assert_eq!(resp.username, "alice");
        assert_eq!(resp.role, "admin");
        assert_eq!(resp.org_id.as_deref(), Some("org-1"));
    }

    #[test]
    fn test_me_response_no_org() {
        let auth = AuthUser {
            user_id: "u-2".into(),
            username: "bob".into(),
            role: Role::Guest,
            org_id: None,
        };
        let resp = MeResponse::from(auth);
        assert!(resp.org_id.is_none());
        assert_eq!(resp.role, "guest");
    }

    #[test]
    fn test_me_response_serialization() {
        let auth = AuthUser {
            user_id: "u-1".into(),
            username: "alice".into(),
            role: Role::Member,
            org_id: None,
        };
        let resp = MeResponse::from(auth);
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"username\":\"alice\""));
        assert!(json.contains("\"role\":\"member\""));
        assert!(json.contains("\"org_id\":null"));
    }

    #[test]
    fn test_login_response_serialization() {
        let resp = LoginResponse {
            token: "jwt-token".into(),
            user: UserResponse {
                id: "123".into(),
                username: "alice".into(),
                email: "alice@example.com".into(),
                display_name: "Alice".into(),
                bio: None,
                role: "admin".into(),
                created_at: "2025-01-01T00:00:00+00:00".into(),
                updated_at: "2025-01-01T00:00:00+00:00".into(),
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"token\":\"jwt-token\""));
        assert!(json.contains("\"username\":\"alice\""));
    }

    #[test]
    fn test_generate_verification_code_is_six_digits() {
        let code = generate_verification_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
        let num: u32 = code.parse().unwrap();
        assert!(num < 1_000_000);
    }

    #[test]
    fn test_generate_verification_code_varies() {
        let codes: Vec<String> = (0..10).map(|_| generate_verification_code()).collect();
        let unique: std::collections::HashSet<&str> = codes.iter().map(|s| s.as_str()).collect();
        assert!(unique.len() > 1, "codes should vary across calls");
    }

    #[test]
    fn test_verify_email_request_parse() {
        let json = r#"{"email":"alice@example.com","code":"123456"}"#;
        let req: VerifyEmailRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.email, "alice@example.com");
        assert_eq!(req.code, "123456");
    }

    #[test]
    fn test_verify_email_response_serialization() {
        let resp = VerifyEmailResponse {
            status: "ok".into(),
            message: "Email verified successfully".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("Email verified successfully"));
    }
}
