#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::config::SecurityConfig;
use crate::error::CoreError;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, serde::Serialize)]
pub struct MessageResponse {
    pub message: String,
}

pub fn validate_password_policy(password: &str, policy: &SecurityConfig) -> Vec<String> {
    let mut violations = Vec::new();

    if password.len() < policy.password_min_length {
        violations.push(format!(
            "Password must be at least {} characters",
            policy.password_min_length
        ));
    }

    if password.len() > policy.password_max_length {
        violations.push(format!(
            "Password must be at most {} characters",
            policy.password_max_length
        ));
    }

    if policy.password_require_uppercase && !password.chars().any(|c| c.is_ascii_uppercase()) {
        violations.push("Password must contain at least one uppercase letter".into());
    }

    if policy.password_require_lowercase && !password.chars().any(|c| c.is_ascii_lowercase()) {
        violations.push("Password must contain at least one lowercase letter".into());
    }

    if policy.password_require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
        violations.push("Password must contain at least one digit".into());
    }

    if policy.password_require_special && !password.chars().any(|c| !c.is_alphanumeric()) {
        violations.push("Password must contain at least one special character".into());
    }

    for ch in password.chars() {
        if ch.is_control() {
            violations.push("Password contains invalid characters".into());
            break;
        }
    }

    violations
}

pub async fn change_password(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    auth: AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    if auth.user_id != user_id {
        return (
            StatusCode::FORBIDDEN,
            Json(
                CoreError::Forbidden("cannot change another user's password".into())
                    .error_response(),
            ),
        )
            .into_response();
    }

    let violations = validate_password_policy(&req.new_password, &state.config.security);
    if !violations.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::Config(violations.join("; ")).error_response()),
        )
            .into_response();
    }

    if req.current_password == req.new_password {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                CoreError::Config("new password must differ from current password".into())
                    .error_response(),
            ),
        )
            .into_response();
    }

    let uid = match uuid::Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid user ID format".into()).error_response()),
            )
                .into_response();
        }
    };

    // Hash the new password with SHA-256
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(req.new_password.as_bytes());
    let new_hash = hex::encode(hasher.finalize());

    // Verify current_password against stored hash
    let stored_hash = match state.db.get_password_hash(uid).await {
        Ok(Some(h)) => h,
        Ok(None) => {
            return (
                StatusCode::FORBIDDEN,
                Json(CoreError::Forbidden("user has no password set".into()).error_response()),
            )
                .into_response();
        }
        Err(e) => return (e.status_code(), Json(e.error_response())).into_response(),
    };

    let mut current_hasher = Sha256::new();
    current_hasher.update(req.current_password.as_bytes());
    let current_hash = hex::encode(current_hasher.finalize());

    if current_hash != stored_hash {
        return (
            StatusCode::FORBIDDEN,
            Json(CoreError::Forbidden("Current password is incorrect".into()).error_response()),
        )
            .into_response();
    }

    match state.db.change_password(uid, &new_hash).await {
        Ok(()) => (
            StatusCode::OK,
            Json(MessageResponse {
                message: "password changed successfully".into(),
            }),
        )
            .into_response(),
        Err(e) => (e.status_code(), Json(e.error_response())).into_response(),
    }
}

pub fn password_routes() -> axum::Router<AppState> {
    axum::Router::new().route(
        "/api/v1/users/{id}/password",
        axum::routing::post(change_password),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SecurityConfig;

    fn strict_policy() -> SecurityConfig {
        SecurityConfig {
            password_min_length: 8,
            password_max_length: 128,
            password_require_uppercase: true,
            password_require_lowercase: true,
            password_require_digit: true,
            password_require_special: true,
            ..SecurityConfig::default()
        }
    }

    fn lenient_policy() -> SecurityConfig {
        SecurityConfig {
            password_min_length: 4,
            password_max_length: 64,
            password_require_uppercase: false,
            password_require_lowercase: false,
            password_require_digit: false,
            password_require_special: false,
            ..SecurityConfig::default()
        }
    }

    #[test]
    fn test_validate_password_valid() {
        let policy = strict_policy();
        let violations = validate_password_policy("Abcdef1!", &policy);
        assert!(violations.is_empty(), "got: {violations:?}");
    }

    #[test]
    fn test_validate_password_too_short() {
        let policy = strict_policy();
        let violations = validate_password_policy("Ab1!xyz", &policy);
        assert!(violations.iter().any(|v| v.contains("at least 8")));
    }

    #[test]
    fn test_validate_password_too_long() {
        let policy = strict_policy();
        let long = "Aa1!".to_string() + &"x".repeat(130);
        let violations = validate_password_policy(&long, &policy);
        assert!(violations.iter().any(|v| v.contains("at most 128")));
    }

    #[test]
    fn test_validate_password_missing_uppercase() {
        let policy = strict_policy();
        let violations = validate_password_policy("abcdef1!", &policy);
        assert!(violations.iter().any(|v| v.contains("uppercase")));
    }

    #[test]
    fn test_validate_password_missing_lowercase() {
        let policy = strict_policy();
        let violations = validate_password_policy("ABCDEF1!", &policy);
        assert!(violations.iter().any(|v| v.contains("lowercase")));
    }

    #[test]
    fn test_validate_password_missing_digit() {
        let policy = strict_policy();
        let violations = validate_password_policy("Abcdefg!", &policy);
        assert!(violations.iter().any(|v| v.contains("digit")));
    }

    #[test]
    fn test_validate_password_missing_special() {
        let policy = strict_policy();
        let violations = validate_password_policy("Abcdefg1", &policy);
        assert!(violations.iter().any(|v| v.contains("special")));
    }

    #[test]
    fn test_validate_password_multiple_violations() {
        let policy = strict_policy();
        let violations = validate_password_policy("short", &policy);
        assert!(
            violations.len() >= 3,
            "expected >=3 violations, got: {violations:?}"
        );
    }

    #[test]
    fn test_validate_password_control_chars() {
        let policy = lenient_policy();
        let violations = validate_password_policy("abc\ndef", &policy);
        assert!(violations.iter().any(|v| v.contains("invalid characters")));
    }

    #[test]
    fn test_validate_password_lenient_policy() {
        let policy = lenient_policy();
        let violations = validate_password_policy("test", &policy);
        assert!(violations.is_empty(), "got: {violations:?}");
    }

    #[test]
    fn test_validate_password_at_min_boundary() {
        let policy = strict_policy();
        let violations = validate_password_policy("Abcde1!x", &policy);
        assert!(violations.is_empty(), "got: {violations:?}");
    }

    #[test]
    fn test_validate_password_at_max_boundary() {
        let policy = strict_policy();
        let middle = "a".repeat(124);
        let pw = format!("Ab{middle}1!");
        assert_eq!(pw.len(), 128);
        let violations = validate_password_policy(&pw, &policy);
        assert!(violations.is_empty(), "got: {violations:?}");
    }

    #[test]
    fn test_change_password_request_parse() {
        let json = r#"{"current_password":"old","new_password":"newpass123"}"#;
        let req: ChangePasswordRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.current_password, "old");
        assert_eq!(req.new_password, "newpass123");
    }

    #[test]
    fn test_change_password_request_missing_fields() {
        let json = r#"{"current_password":"old"}"#;
        let result = serde_json::from_str::<ChangePasswordRequest>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_message_response_serialization() {
        let resp = MessageResponse {
            message: "password changed successfully".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"message\":\"password changed successfully\""));
    }

    #[test]
    fn test_password_hash() {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(b"testpassword");
        let hash = hex::encode(hasher.finalize());
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_password_routes_type() {
        fn _assert_routes() -> axum::Router<AppState> {
            password_routes()
        }
    }

    #[test]
    fn test_security_config_default_password_fields() {
        let policy = SecurityConfig::default();
        assert_eq!(policy.password_min_length, 8);
        assert_eq!(policy.password_max_length, 128);
        assert!(policy.password_require_uppercase);
        assert!(policy.password_require_lowercase);
        assert!(policy.password_require_digit);
        assert!(policy.password_require_special);
    }
}
