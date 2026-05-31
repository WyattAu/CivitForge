#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::api::users::UserResponse;
use crate::error::CoreError;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub email: String,
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
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
    let user = match state.db.get_user_by_username(&req.username).await {
        Ok(u) => u,
        Err(_) => {
            state
                .db
                .create_user(&req.username, &req.email, &req.display_name, "member")
                .await?
        }
    };

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

pub async fn refresh() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(CoreError::Internal("refresh not implemented".into()).error_response()),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::rbac::Role;

    #[test]
    fn test_login_request_parse() {
        let json = r#"{"username":"alice","email":"alice@example.com","display_name":"Alice"}"#;
        let req: LoginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.username, "alice");
        assert_eq!(req.email, "alice@example.com");
        assert_eq!(req.display_name, "Alice");
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
}
