#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_admin};
use crate::error::CoreError;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, patch},
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

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct OidcProviderRow {
    pub id: Uuid,
    pub name: String,
    pub issuer: String,
    pub client_id: String,
    pub jwks_uri: String,
    pub client_secret: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OidcProviderResponse {
    pub id: String,
    pub name: String,
    pub issuer: String,
    pub client_id: String,
    pub jwks_uri: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<OidcProviderRow> for OidcProviderResponse {
    fn from(row: OidcProviderRow) -> Self {
        Self {
            id: row.id.to_string(),
            name: row.name,
            issuer: row.issuer,
            client_id: row.client_id,
            jwks_uri: row.jwks_uri,
            enabled: row.enabled,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        }
    }
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

#[derive(Debug, Deserialize)]
pub struct CreateOidcProviderRequest {
    pub name: String,
    pub issuer: String,
    pub client_id: String,
    #[serde(default)]
    pub jwks_uri: Option<String>,
    #[serde(default)]
    pub client_secret: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOidcProviderRequest {
    pub name: Option<String>,
    pub issuer: Option<String>,
    pub client_id: Option<String>,
    pub jwks_uri: Option<String>,
    pub client_secret: Option<String>,
    pub enabled: Option<bool>,
}

// ---------------------------------------------------------------------------
// Admin OIDC provider CRUD
// ---------------------------------------------------------------------------

pub async fn list_oidc_providers(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let pool = state.db.pool();
    let rows: Vec<OidcProviderRow> = sqlx::query_as(
        "SELECT id, name, issuer, client_id, jwks_uri, client_secret, enabled, created_at, updated_at FROM oidc_providers ORDER BY name",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let out: Vec<OidcProviderResponse> = rows.into_iter().map(OidcProviderResponse::from).collect();
    (StatusCode::OK, Json(out)).into_response()
}

pub async fn create_oidc_provider(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateOidcProviderRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    if req.name.is_empty() || req.issuer.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::Config("name and issuer are required".into()).error_response()),
        )
            .into_response();
    }

    let pool = state.db.pool();
    let result = sqlx::query_as::<_, OidcProviderRow>(
        r#"INSERT INTO oidc_providers (name, issuer, client_id, jwks_uri, client_secret, enabled)
           VALUES ($1, $2, $3, $4, $5, true)
           RETURNING id, name, issuer, client_id, jwks_uri, client_secret, enabled, created_at, updated_at"#,
    )
    .bind(&req.name)
    .bind(&req.issuer)
    .bind(&req.client_id)
    .bind(req.jwks_uri.as_deref().unwrap_or(""))
    .bind(req.client_secret.as_deref().unwrap_or(""))
    .fetch_one(pool)
    .await;

    match result {
        Ok(row) => (
            StatusCode::CREATED,
            Json(OidcProviderResponse::from(row)),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_oidc_provider(
    State(state): State<AppState>,
    Path(id): Path<String>,
    auth: AuthUser,
    Json(req): Json<UpdateOidcProviderRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let provider_id = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid provider id".into()).error_response()),
            )
                .into_response();
        }
    };

    let pool = state.db.pool();

    let name = req.name.unwrap_or_default();
    let issuer = req.issuer.unwrap_or_default();
    let client_id = req.client_id.unwrap_or_default();
    let jwks_uri = req.jwks_uri.unwrap_or_default();
    let client_secret = req.client_secret.unwrap_or_default();
    let enabled = req.enabled.unwrap_or(true);

    let result = sqlx::query_as::<_, OidcProviderRow>(
        r#"UPDATE oidc_providers SET name = $1, issuer = $2, client_id = $3, jwks_uri = $4, client_secret = $5, enabled = $6, updated_at = NOW()
           WHERE id = $7
           RETURNING id, name, issuer, client_id, jwks_uri, client_secret, enabled, created_at, updated_at"#,
    )
    .bind(&name)
    .bind(&issuer)
    .bind(&client_id)
    .bind(&jwks_uri)
    .bind(&client_secret)
    .bind(enabled)
    .bind(provider_id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(row)) => (StatusCode::OK, Json(OidcProviderResponse::from(row))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("provider not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_oidc_provider(
    State(state): State<AppState>,
    Path(id): Path<String>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let provider_id = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::Config("invalid provider id".into()).error_response()),
            )
                .into_response();
        }
    };

    let pool = state.db.pool();
    let result = sqlx::query("DELETE FROM oidc_providers WHERE id = $1")
        .bind(provider_id)
        .execute(pool)
        .await;

    match result {
        Ok(r) => {
            if r.rows_affected() == 0 {
                (
                    StatusCode::NOT_FOUND,
                    Json(CoreError::NotFound("provider not found".into()).error_response()),
                )
                    .into_response()
            } else {
                (StatusCode::NO_CONTENT, ()).into_response()
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
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

    let _ = sqlx::query(
        r#"INSERT INTO oidc_providers (name, issuer, client_id)
           VALUES ($1, $1, '')
           ON CONFLICT (name) DO NOTHING"#,
    )
    .bind(&req.provider)
    .execute(pool)
    .await;

    let provider_user_id = decode_id_token_sub(&req.id_token).unwrap_or_default();

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

fn decode_id_token_sub(id_token: &str) -> Option<String> {
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() < 2 {
        return None;
    }
    use base64::Engine;
    let payload = parts.get(1)?;
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload.as_bytes())
        .ok()?;
    let json: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    json.get("sub")?.as_str().map(String::from)
}

pub fn oidc_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/admin/oidc-providers",
            get(list_oidc_providers).post(create_oidc_provider),
        )
        .route(
            "/api/v1/admin/oidc-providers/{id}",
            patch(update_oidc_provider).delete(delete_oidc_provider),
        )
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

    #[test]
    fn test_oidc_provider_response_serialize() {
        let resp = OidcProviderResponse {
            id: "00000000-0000-0000-0000-000000000001".into(),
            name: "google".into(),
            issuer: "https://accounts.google.com".into(),
            client_id: "my-client".into(),
            jwks_uri: "https://www.googleapis.com/oauth2/v3/certs".into(),
            enabled: true,
            created_at: "2025-01-01T00:00:00+00:00".into(),
            updated_at: "2025-01-01T00:00:00+00:00".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"name\":\"google\""));
        assert!(json.contains("\"enabled\":true"));
    }

    #[test]
    fn test_oidc_routes_compile() {
        let _ = oidc_routes();
    }
}
