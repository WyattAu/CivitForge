#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
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

pub async fn change_password(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    auth: AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    // Only the user themselves can change their password
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

    // Validate new password
    if req.new_password.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                CoreError::Config("new password must be at least 8 characters".into())
                    .error_response(),
            ),
        )
            .into_response();
    }

    if req.new_password.len() > 128 {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                CoreError::Config("new password must be at most 128 characters".into())
                    .error_response(),
            ),
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

    // Check for control characters
    for ch in req.new_password.chars() {
        if ch.is_control() {
            return (
                StatusCode::BAD_REQUEST,
                Json(
                    CoreError::Config("password contains invalid characters".into())
                        .error_response(),
                ),
            )
                .into_response();
        }
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
        // SHA-256 hex is always 64 chars
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_password_routes_type() {
        fn _assert_routes() -> axum::Router<AppState> {
            password_routes()
        }
    }
}
