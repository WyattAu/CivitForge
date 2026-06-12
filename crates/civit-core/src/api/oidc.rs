#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ExchangeOidcTokenRequest {
    pub provider: String,
    pub id_token: String,
    pub access_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OidcExchangeResponse {
    pub user_id: String,
    pub username: String,
    pub provider: String,
    pub provider_user_id: String,
    pub linked: bool,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct OidcProvider {
    pub id: Uuid,
    pub name: String,
    pub issuer: String,
    pub client_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct OidcIdentity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub email: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn exchange_oidc_token(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<ExchangeOidcTokenRequest>,
) -> impl IntoResponse {
    let user_uuid = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid user id".into()).error_response()),
            )
                .into_response();
        }
    };

    if req.provider.is_empty() || req.id_token.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::Config("provider and id_token required".into()).error_response()),
        )
            .into_response();
    }

    let pool = state.db.pool();

    // Upsert OIDC provider record
    let _ = sqlx::query(
        r#"INSERT INTO oidc_providers (name, issuer, client_id)
           VALUES ($1, $1, '')
           ON CONFLICT (name) DO NOTHING"#,
    )
    .bind(&req.provider)
    .execute(pool)
    .await;

    // Extract a provider_user_id from the id_token payload (simplified: use base64-decoded payload sub claim).
    // In production this would verify the JWT signature against the provider's JWKS.
    let provider_user_id = decode_id_token_sub(&req.id_token).unwrap_or_default();

    // Upsert OIDC identity linking
    let result = sqlx::query_as::<_, OidcIdentity>(
        r#"INSERT INTO oidc_identities (user_id, provider, provider_user_id)
           VALUES ($1, $2, $3)
           ON CONFLICT (provider, provider_user_id) DO UPDATE SET user_id = $1
           RETURNING *"#,
    )
    .bind(user_uuid)
    .bind(&req.provider)
    .bind(&provider_user_id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(identity)) => (
            StatusCode::OK,
            Json(OidcExchangeResponse {
                user_id: user_uuid.to_string(),
                username: auth.username,
                provider: identity.provider,
                provider_user_id: identity.provider_user_id,
                linked: true,
            }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::CREATED,
            Json(OidcExchangeResponse {
                user_id: user_uuid.to_string(),
                username: auth.username,
                provider: req.provider,
                provider_user_id,
                linked: true,
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Simple base64 decode of JWT payload to extract "sub" claim.
/// Returns None if decoding/parsing fails.
fn decode_id_token_sub(id_token: &str) -> Option<String> {
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() < 2 {
        return None;
    }
    use base64::Engine;
    let payload = parts.get(1)?;
    // URL-safe base64 with padding
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload.as_bytes())
        .ok()?;
    let json: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    json.get("sub")?.as_str().map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_id_token_sub_valid() {
        use base64::Engine;
        let payload = serde_json::json!({"sub": "user123", "iss": "https://idp.example.com"});
        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap());
        let token = format!("header.{}", encoded);
        assert_eq!(decode_id_token_sub(&token).as_deref(), Some("user123"));
    }

    #[test]
    fn test_decode_id_token_sub_invalid_format() {
        assert!(decode_id_token_sub("not-a-jwt").is_none());
    }

    #[test]
    fn test_decode_id_token_sub_empty() {
        assert!(decode_id_token_sub("").is_none());
    }

    #[test]
    fn test_exchange_request_deserialize() {
        let json = r#"{"provider":"github","id_token":"abc.def"}"#;
        let req: ExchangeOidcTokenRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.provider, "github");
        assert!(req.access_token.is_none());
    }

    #[test]
    fn test_exchange_response_serialize() {
        let resp = OidcExchangeResponse {
            user_id: "u1".into(),
            username: "alice".into(),
            provider: "github".into(),
            provider_user_id: "12345".into(),
            linked: true,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"provider\":\"github\""));
    }
}
