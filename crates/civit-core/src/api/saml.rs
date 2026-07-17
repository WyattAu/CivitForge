#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_admin};
use crate::auth::saml::{SamlConfig, SamlService};
use crate::error::CoreError;
use axum::{
    Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use axum::extract::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct SamlProviderRow {
    pub id: Uuid,
    pub name: String,
    pub entity_id: String,
    pub sso_url: String,
    pub certificate: String,
    pub metadata_url: Option<String>,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct SamlProviderResponse {
    pub id: String,
    pub name: String,
    pub entity_id: String,
    pub sso_url: String,
    pub enabled: bool,
    pub created_at: String,
}

impl From<SamlProviderRow> for SamlProviderResponse {
    fn from(row: SamlProviderRow) -> Self {
        Self {
            id: row.id.to_string(),
            name: row.name,
            entity_id: row.entity_id,
            sso_url: row.sso_url,
            enabled: row.enabled,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateSamlProviderRequest {
    pub name: String,
    pub entity_id: String,
    pub sso_url: String,
    pub certificate: String,
    #[serde(default)]
    pub metadata_url: Option<String>,
}

// ---------------------------------------------------------------------------
// GET /api/v1/saml/:name/metadata
// Returns SAML metadata XML for a given provider.
// ---------------------------------------------------------------------------
pub async fn saml_metadata(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let row: Option<SamlProviderRow> = sqlx::query_as(
        "SELECT id, name, entity_id, sso_url, certificate, metadata_url, enabled, created_at
         FROM saml_providers WHERE name = $1 AND enabled = true",
    )
    .bind(&name)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let provider = match row {
        Some(r) => r,
        None => {
            return (
                StatusCode::NOT_FOUND,
                HeaderMap::new(),
                "SAML provider not found".to_string(),
            )
                .into_response();
        }
    };

    let acs_url = format!("/api/v1/saml/{}/acs", provider.name);
    let metadata_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<EntityDescriptor xmlns="urn:oasis:names:tc:SAML:2.0:metadata"
                  entityID="{entity_id}">
  <SPSSODescriptor
      AuthnRequestsSigned="false"
      WantAssertionsSigned="true"
      protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
    <NameIDFormat>urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress</NameIDFormat>
    <AssertionConsumerService
        Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
        Location="{acs_url}"
        index="0" />
  </SPSSODescriptor>
</EntityDescriptor>"#,
        entity_id = provider.entity_id,
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/xml; charset=utf-8"),
    );

    (StatusCode::OK, headers, metadata_xml).into_response()
}

// ---------------------------------------------------------------------------
// POST /api/v1/saml/:name/acs
// Assertion Consumer Service – receives SAML responses from the IdP.
// ---------------------------------------------------------------------------
pub async fn saml_acs(
    State(state): State<AppState>,
    Path(name): Path<String>,
    axum::extract::Form(form): axum::extract::Form<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let row: Option<SamlProviderRow> = sqlx::query_as(
        "SELECT id, name, entity_id, sso_url, certificate, metadata_url, enabled, created_at
         FROM saml_providers WHERE name = $1 AND enabled = true",
    )
    .bind(&name)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let provider = match row {
        Some(r) => r,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                "SAML provider not found".to_string(),
            )
                .into_response();
        }
    };

    let saml_response_b64 = match form.get("SAMLResponse") {
        Some(v) => v,
        None => {
            return (StatusCode::BAD_REQUEST, "missing SAMLResponse".to_string()).into_response();
        }
    };

    use base64::Engine;
    let decoded_bytes = match base64::engine::general_purpose::STANDARD.decode(saml_response_b64) {
        Ok(b) => b,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                "invalid base64 in SAMLResponse".to_string(),
            )
                .into_response();
        }
    };
    let decoded_xml = match String::from_utf8(decoded_bytes) {
        Ok(s) => s,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                "SAMLResponse is not valid UTF-8".to_string(),
            )
                .into_response();
        }
    };

    let saml_svc = SamlService::new(SamlConfig {
        entity_id: provider.entity_id.clone(),
        sso_url: provider.sso_url.clone(),
        slo_url: String::new(),
        certificate: provider.certificate.clone(),
        name_id_format: "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress".into(),
    });

    let response = match SamlService::parse_response(&decoded_xml) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                format!("failed to parse SAML response: {e}"),
            )
                .into_response();
        }
    };

    if response.status != crate::auth::saml::SamlStatus::Success {
        return (
            StatusCode::UNAUTHORIZED,
            "SAML authentication failed".to_string(),
        )
            .into_response();
    }

    // Signature validation is fail-closed until XML-DSIG is implemented
    if !saml_svc.is_valid_signature() {
        return (
            StatusCode::UNAUTHORIZED,
            "SAML signature validation not yet implemented".to_string(),
        )
            .into_response();
    }

    // Look up or JIT-provision user by NameID (email)
    let email = &response.name_id;
    let username = response
        .attributes
        .get("given_name")
        .or_else(|| response.attributes.get("username"))
        .cloned()
        .unwrap_or_else(|| email.split('@').next().unwrap_or("user").to_string());

    let user = match state.db.get_user_by_email(email).await {
        Ok(u) => u,
        Err(_) => {
            // JIT provisioning: create user on first SSO login
            let display_name = response
                .attributes
                .get("displayName")
                .or_else(|| response.attributes.get("given_name"))
                .cloned()
                .unwrap_or_else(|| username.clone());
            match state
                .db
                .create_user(email, email, &display_name, "member", "")
                .await
            {
                Ok(u) => u,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("JIT provisioning failed: {e}"),
                    )
                        .into_response();
                }
            }
        }
    };

    // Generate JWT token
    let token = match state
        .jwt_service
        .generate_token(&user.id.to_string(), &user.username, &user.role, None)
    {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("token generation failed: {e}"),
            )
                .into_response();
        }
    };

    // Record login history
    let _ = sqlx::query(
        "INSERT INTO login_history (username, provider, success) VALUES ($1, 'saml', true)",
    )
    .bind(&user.username)
    .execute(state.db.pool())
    .await;

    // Return a redirect with token in query string (SP-initiated flow)
    let redirect_url = format!("/auth/sso-callback?token={token}");
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::LOCATION,
        HeaderValue::try_from(&redirect_url).unwrap_or_else(|_| HeaderValue::from_static("/")),
    );
    (StatusCode::FOUND, headers, "").into_response()
}

// ---------------------------------------------------------------------------
// POST /api/v1/admin/saml – Create SAML provider
// ---------------------------------------------------------------------------
pub async fn create_saml_provider(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Json(req): axum::extract::Json<CreateSamlProviderRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    if req.name.is_empty() || req.entity_id.is_empty() || req.sso_url.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::Config("name, entity_id, and sso_url are required".into()).error_response()),
        )
            .into_response();
    }

    let pool = state.db.pool();
    let result = sqlx::query_as::<_, SamlProviderRow>(
        r#"INSERT INTO saml_providers (name, entity_id, sso_url, certificate, metadata_url)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, name, entity_id, sso_url, certificate, metadata_url, enabled, created_at"#,
    )
    .bind(&req.name)
    .bind(&req.entity_id)
    .bind(&req.sso_url)
    .bind(&req.certificate)
    .bind(&req.metadata_url)
    .fetch_one(pool)
    .await;

    match result {
        Ok(row) => (
            StatusCode::CREATED,
            Json(SamlProviderResponse::from(row)),
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
// GET /api/v1/admin/saml – List SAML providers
// ---------------------------------------------------------------------------
pub async fn list_saml_providers(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let pool = state.db.pool();
    let rows: Vec<SamlProviderRow> = sqlx::query_as(
        "SELECT id, name, entity_id, sso_url, certificate, metadata_url, enabled, created_at
         FROM saml_providers ORDER BY name",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let out: Vec<SamlProviderResponse> = rows.into_iter().map(SamlProviderResponse::from).collect();
    (StatusCode::OK, Json(out)).into_response()
}

pub fn saml_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/saml/{name}/metadata",
            get(saml_metadata),
        )
        .route(
            "/api/v1/saml/{name}/acs",
            post(saml_acs),
        )
        .route(
            "/api/v1/admin/saml",
            get(list_saml_providers).post(create_saml_provider),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saml_provider_response_serialize() {
        let resp = SamlProviderResponse {
            id: "00000000-0000-0000-0000-000000000001".into(),
            name: "okta".into(),
            entity_id: "https://app.example.com/saml".into(),
            sso_url: "https://okta.example.com/sso".into(),
            enabled: true,
            created_at: "2025-01-01T00:00:00+00:00".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"name\":\"okta\""));
        assert!(json.contains("\"enabled\":true"));
    }

    #[test]
    fn test_saml_routes_compile() {
        let _ = saml_routes();
    }
}
