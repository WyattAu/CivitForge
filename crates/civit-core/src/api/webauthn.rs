#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::api::users::UserResponse;
use crate::error::CoreError;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::post,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RegisterStartRequest {
    pub user_id: String,
    pub username: String,
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterFinishResponse {
    pub status: String,
    pub credential_id: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthenticateStartRequest {
    pub user_id: String,
}

#[derive(Debug, Serialize)]
pub struct AuthenticateFinishResponse {
    pub token: String,
    pub user: UserResponse,
}

pub fn webauthn_routes() -> Router<crate::api::AppState> {
    Router::new()
        .route("/api/v1/auth/webauthn/register/start", post(register_start))
        .route(
            "/api/v1/auth/webauthn/register/finish",
            post(register_finish),
        )
        .route(
            "/api/v1/auth/webauthn/authenticate/start",
            post(authenticate_start),
        )
        .route(
            "/api/v1/auth/webauthn/authenticate/finish",
            post(authenticate_finish),
        )
}

pub async fn register_start(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<RegisterStartRequest>,
) -> impl IntoResponse {
    let webauthn = match &state.webauthn_service {
        Some(w) => w,
        None => {
            return (
                StatusCode::NOT_IMPLEMENTED,
                Json(serde_json::json!({"error": "WebAuthn is not enabled"})),
            )
                .into_response();
        }
    };

    if auth.user_id != req.user_id {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "user_id does not match authenticated user"})),
        )
            .into_response();
    }

    match webauthn.start_registration(&req.user_id, &req.username, &req.display_name) {
        Ok(ccr) => (StatusCode::OK, Json(ccr)).into_response(),
        Err(e) => {
            let core_err: CoreError = e.into();
            (core_err.status_code(), Json(core_err.error_response())).into_response()
        }
    }
}

pub async fn register_finish(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(credential): Json<civit_auth::webauthn::RegisterPublicKeyCredential>,
) -> impl IntoResponse {
    let webauthn = match &state.webauthn_service {
        Some(w) => w,
        None => {
            return (
                StatusCode::NOT_IMPLEMENTED,
                Json(serde_json::json!({"error": "WebAuthn is not enabled"})),
            )
                .into_response();
        }
    };

    match webauthn.finish_registration(&auth.user_id, credential) {
        Ok(passkey) => {
            let passkey_json = serde_json::to_vec(&passkey).unwrap_or_default();

            let user_id = match uuid::Uuid::parse_str(&auth.user_id) {
                Ok(id) => id,
                Err(_) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "invalid user ID"})),
                    )
                        .into_response();
                }
            };

            let cred_id_bytes: Vec<u8> = passkey.cred_id().as_ref().to_vec();

            let result = sqlx::query(
                "INSERT INTO webauthn_credentials (user_id, credential_id, public_key, counter) VALUES ($1, $2, $3, $4)",
            )
            .bind(user_id)
            .bind(&cred_id_bytes)
            .bind(&passkey_json)
            .bind(0i64)
            .execute(state.db.pool())
            .await;

            match result {
                Ok(_) => (
                    StatusCode::CREATED,
                    Json(RegisterFinishResponse {
                        status: "ok".into(),
                        credential_id: base64::Engine::encode(
                            &base64::engine::general_purpose::STANDARD,
                            &cred_id_bytes,
                        ),
                    }),
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("failed to store credential: {e}")})),
                )
                    .into_response(),
            }
        }
        Err(e) => {
            let core_err: CoreError = e.into();
            (core_err.status_code(), Json(core_err.error_response())).into_response()
        }
    }
}

pub async fn authenticate_start(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<AuthenticateStartRequest>,
) -> impl IntoResponse {
    let webauthn = match &state.webauthn_service {
        Some(w) => w,
        None => {
            return (
                StatusCode::NOT_IMPLEMENTED,
                Json(serde_json::json!({"error": "WebAuthn is not enabled"})),
            )
                .into_response();
        }
    };

    if auth.user_id != req.user_id {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "user_id does not match authenticated user"})),
        )
            .into_response();
    }

    let user_id = match uuid::Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "invalid user ID"})),
            )
                .into_response();
        }
    };

    let passkeys_result = sqlx::query_scalar::<_, Vec<u8>>(
        "SELECT public_key FROM webauthn_credentials WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(state.db.pool())
    .await;

    let passkey_rows = match passkeys_result {
        Ok(rows) => rows,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("failed to fetch credentials: {e}")})),
            )
                .into_response();
        }
    };

    if passkey_rows.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "no WebAuthn credentials registered"})),
        )
            .into_response();
    }

    let mut passkeys = Vec::new();
    for row in &passkey_rows {
        match serde_json::from_slice::<civit_auth::webauthn::Passkey>(row) {
            Ok(passkey) => passkeys.push(passkey),
            Err(e) => {
                tracing::warn!("failed to deserialize passkey: {e}");
            }
        }
    }

    if passkeys.is_empty() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "no valid credentials found"})),
        )
            .into_response();
    }

    match webauthn.start_authentication(&auth.user_id, passkeys) {
        Ok(rcr) => (StatusCode::OK, Json(rcr)).into_response(),
        Err(e) => {
            let core_err: CoreError = e.into();
            (core_err.status_code(), Json(core_err.error_response())).into_response()
        }
    }
}

pub async fn authenticate_finish(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(credential): Json<civit_auth::webauthn::PublicKeyCredential>,
) -> impl IntoResponse {
    let webauthn = match &state.webauthn_service {
        Some(w) => w,
        None => {
            return (
                StatusCode::NOT_IMPLEMENTED,
                Json(serde_json::json!({"error": "WebAuthn is not enabled"})),
            )
                .into_response();
        }
    };

    match webauthn.finish_authentication(&auth.user_id, credential) {
        Ok(()) => {
            let user_id = match uuid::Uuid::parse_str(&auth.user_id) {
                Ok(id) => id,
                Err(_) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "invalid user ID"})),
                    )
                        .into_response();
                }
            };

            let user = match state.db.get_user_by_id(user_id).await {
                Ok(u) => u,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": format!("failed to fetch user: {e}")})),
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
                        Json(
                            serde_json::json!({"error": format!("failed to generate token: {e}")}),
                        ),
                    )
                        .into_response();
                }
            };

            (
                StatusCode::OK,
                Json(AuthenticateFinishResponse {
                    token,
                    user: UserResponse::from(user),
                }),
            )
                .into_response()
        }
        Err(e) => {
            let core_err: CoreError = e.into();
            (core_err.status_code(), Json(core_err.error_response())).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_start_request_deserialize() {
        let json = r#"{"user_id":"u-1","username":"alice","display_name":"Alice Smith"}"#;
        let req: RegisterStartRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.user_id, "u-1");
        assert_eq!(req.username, "alice");
        assert_eq!(req.display_name, "Alice Smith");
    }

    #[test]
    fn test_authenticate_start_request_deserialize() {
        let json = r#"{"user_id":"u-1"}"#;
        let req: AuthenticateStartRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.user_id, "u-1");
    }

    #[test]
    fn test_register_finish_response_serialize() {
        let resp = RegisterFinishResponse {
            status: "ok".into(),
            credential_id: "abc123".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"credential_id\":\"abc123\""));
    }

    #[test]
    fn test_authenticate_finish_response_serialize() {
        let resp = AuthenticateFinishResponse {
            token: "jwt-token".into(),
            user: UserResponse {
                id: "123".into(),
                username: "alice".into(),
                email: "alice@example.com".into(),
                display_name: "Alice".into(),
                bio: None,
                role: "admin".into(),
                avatar_url: None,
                location: None,
                website: None,
                created_at: "2025-01-01T00:00:00+00:00".into(),
                updated_at: "2025-01-01T00:00:00+00:00".into(),
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"token\":\"jwt-token\""));
        assert!(json.contains("\"username\":\"alice\""));
    }
}
