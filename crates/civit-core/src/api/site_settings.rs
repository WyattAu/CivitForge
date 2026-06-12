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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SiteSettings {
    pub id: i32,
    pub site_name: String,
    pub site_description: String,
    pub footer_text: String,
    pub logo_url: String,
    pub contact_email: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSiteSettingsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_email: Option<String>,
}

pub async fn get_site_settings(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let result = sqlx::query_as::<_, SiteSettingsRow>(
        "SELECT id, site_name, site_description, footer_text, logo_url, contact_email, updated_at FROM site_settings WHERE id = 1",
    )
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(row)) => (
            StatusCode::OK,
            Json(SiteSettings::from(row)),
        )
            .into_response(),
        Ok(None) => {
            let defaults = SiteSettings {
                id: 1,
                site_name: "CivitForge".into(),
                site_description: String::new(),
                footer_text: String::new(),
                logo_url: String::new(),
                contact_email: String::new(),
                updated_at: String::new(),
            };
            (StatusCode::OK, Json(defaults)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_site_settings(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdateSiteSettingsRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let pool = state.db.pool();

    let site_name = req.site_name.unwrap_or_default();
    let site_description = req.site_description.unwrap_or_default();
    let footer_text = req.footer_text.unwrap_or_default();
    let logo_url = req.logo_url.unwrap_or_default();
    let contact_email = req.contact_email.unwrap_or_default();

    let result = sqlx::query_as::<_, SiteSettingsRow>(
        r#"INSERT INTO site_settings (id, site_name, site_description, footer_text, logo_url, contact_email, updated_at)
           VALUES (1, $1, $2, $3, $4, $5, NOW())
           ON CONFLICT (id) DO UPDATE SET
               site_name = $1,
               site_description = $2,
               footer_text = $3,
               logo_url = $4,
               contact_email = $5,
               updated_at = NOW()
           RETURNING id, site_name, site_description, footer_text, logo_url, contact_email, updated_at"#,
    )
    .bind(&site_name)
    .bind(&site_description)
    .bind(&footer_text)
    .bind(&logo_url)
    .bind(&contact_email)
    .fetch_one(pool)
    .await;

    match result {
        Ok(row) => (StatusCode::OK, Json(SiteSettings::from(row))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

use crate::api::auth::require_admin;

#[derive(Debug, sqlx::FromRow)]
struct SiteSettingsRow {
    id: i32,
    site_name: String,
    site_description: String,
    footer_text: String,
    logo_url: String,
    contact_email: String,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<SiteSettingsRow> for SiteSettings {
    fn from(row: SiteSettingsRow) -> Self {
        Self {
            id: row.id,
            site_name: row.site_name,
            site_description: row.site_description,
            footer_text: row.footer_text,
            logo_url: row.logo_url,
            contact_email: row.contact_email,
            updated_at: row.updated_at.to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_site_settings_defaults() {
        let s = SiteSettings {
            id: 1,
            site_name: "CivitForge".into(),
            site_description: String::new(),
            footer_text: String::new(),
            logo_url: String::new(),
            contact_email: String::new(),
            updated_at: String::new(),
        };
        assert_eq!(s.site_name, "CivitForge");
        assert!(s.footer_text.is_empty());
    }

    #[test]
    fn test_update_request_deserialize() {
        let json = r#"{"site_name":"My Forge","footer_text":"(c) 2025"}"#;
        let req: UpdateSiteSettingsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.site_name.as_deref(), Some("My Forge"));
        assert_eq!(req.footer_text.as_deref(), Some("(c) 2025"));
        assert!(req.logo_url.is_none());
    }

    #[test]
    fn test_site_settings_serialize() {
        let s = SiteSettings {
            id: 1,
            site_name: "Test".into(),
            site_description: "Desc".into(),
            footer_text: "Footer".into(),
            logo_url: "https://logo.png".into(),
            contact_email: "a@b.com".into(),
            updated_at: "2025-01-01T00:00:00+00:00".into(),
        };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("\"site_name\":\"Test\""));
        assert!(json.contains("\"contact_email\":\"a@b.com\""));
    }
}
