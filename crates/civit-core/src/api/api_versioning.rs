#![forbid(unsafe_code)]

//! API versioning routes for version management, deprecation scheduling,
//! changelog generation, and migration guides.

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_admin};
use axum::{
    Json,
    Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiVersionResponse {
    pub id: Uuid,
    pub version: String,
    pub status: String,
    pub release_date: DateTime<Utc>,
    pub deprecation_date: Option<DateTime<Utc>>,
    pub sunset_date: Option<DateTime<Utc>>,
    pub changelog: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateApiVersionRequest {
    pub version: String,
    #[serde(default = "default_active_status")]
    pub status: String,
    #[serde(default)]
    pub changelog: String,
}

fn default_active_status() -> String {
    "active".into()
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeprecateApiVersionRequest {
    pub deprecation_date: DateTime<Utc>,
    pub sunset_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationGuide {
    pub from_version: String,
    pub to_version: String,
    pub changes: Vec<MigrationChange>,
    pub breaking_changes: Vec<String>,
    pub deprecation_notices: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationChange {
    pub endpoint: String,
    pub change_type: String,
    pub description: String,
    pub migration_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionChangelog {
    pub version: String,
    pub release_date: DateTime<Utc>,
    pub sections: ChangelogSections,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogSections {
    pub added: Vec<String>,
    pub changed: Vec<String>,
    pub deprecated: Vec<String>,
    pub removed: Vec<String>,
    pub fixed: Vec<String>,
    pub security: Vec<String>,
}

pub async fn list_api_versions(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.db.list_api_versions().await {
        Ok(versions) => {
            let response: Vec<ApiVersionResponse> = versions.iter().map(|v| ApiVersionResponse {
                id: v.id,
                version: v.version.clone(),
                status: v.status.clone(),
                release_date: v.release_date,
                deprecation_date: v.deprecation_date,
                sunset_date: v.sunset_date,
                changelog: v.changelog.clone(),
                created_at: v.created_at,
            }).collect();
            (StatusCode::OK, Json(serde_json::json!({"versions": response}))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_api_version(
    State(state): State<AppState>,
    axum::extract::Path(version): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state.db.get_api_version(&version).await {
        Ok(v) => {
            let response = ApiVersionResponse {
                id: v.id,
                version: v.version,
                status: v.status,
                release_date: v.release_date,
                deprecation_date: v.deprecation_date,
                sunset_date: v.sunset_date,
                changelog: v.changelog,
                created_at: v.created_at,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create_api_version(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateApiVersionRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.create_api_version(&req.version, &req.status, &req.changelog).await {
        Ok(v) => {
            let response = ApiVersionResponse {
                id: v.id,
                version: v.version,
                status: v.status,
                release_date: v.release_date,
                deprecation_date: v.deprecation_date,
                sunset_date: v.sunset_date,
                changelog: v.changelog,
                created_at: v.created_at,
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn deprecate_api_version(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(version): axum::extract::Path<String>,
    Json(req): Json<DeprecateApiVersionRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.deprecate_api_version(&version, req.deprecation_date, req.sunset_date).await {
        Ok(v) => {
            let response = ApiVersionResponse {
                id: v.id,
                version: v.version,
                status: v.status,
                release_date: v.release_date,
                deprecation_date: v.deprecation_date,
                sunset_date: v.sunset_date,
                changelog: v.changelog,
                created_at: v.created_at,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_migration_guide(
    State(_state): State<AppState>,
    axum::extract::Path((from_version, to_version)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    let guide = MigrationGuide {
        from_version,
        to_version,
        changes: vec![],
        breaking_changes: vec![],
        deprecation_notices: vec![],
    };
    (StatusCode::OK, Json(guide)).into_response()
}

pub async fn get_version_changelog(
    State(state): State<AppState>,
    axum::extract::Path(version): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state.db.get_api_version(&version).await {
        Ok(v) => {
            let changelog = VersionChangelog {
                version: v.version,
                release_date: v.release_date,
                sections: ChangelogSections {
                    added: vec![],
                    changed: vec![],
                    deprecated: vec![],
                    removed: vec![],
                    fixed: vec![],
                    security: vec![],
                },
            };
            (StatusCode::OK, Json(changelog)).into_response()
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub fn api_version_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/versions", get(list_api_versions).post(create_api_version))
        .route("/api/v1/versions/{version}", get(get_api_version))
        .route("/api/v1/versions/{version}/deprecate", get(deprecate_api_version))
        .route("/api/v1/versions/migration-guide/{from}/{to}", get(get_migration_guide))
        .route("/api/v1/versions/{version}/changelog", get(get_version_changelog))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_response_serialization() {
        let response = ApiVersionResponse {
            id: Uuid::nil(),
            version: "v1.0.0".into(),
            status: "active".into(),
            release_date: Utc::now(),
            deprecation_date: None,
            sunset_date: None,
            changelog: "Initial release".into(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"version\":\"v1.0.0\""));
        assert!(json.contains("\"status\":\"active\""));
    }

    #[test]
    fn test_migration_guide_serialization() {
        let guide = MigrationGuide {
            from_version: "v1.0.0".into(),
            to_version: "v2.0.0".into(),
            changes: vec![],
            breaking_changes: vec!["Removed deprecated endpoint".into()],
            deprecation_notices: vec!["New endpoint available".into()],
        };
        let json = serde_json::to_string(&guide).unwrap();
        assert!(json.contains("\"from_version\":\"v1.0.0\""));
        assert!(json.contains("\"to_version\":\"v2.0.0\""));
    }

    #[test]
    fn test_changelog_sections_serialization() {
        let sections = ChangelogSections {
            added: vec!["New endpoint".into()],
            changed: vec!["Updated response format".into()],
            deprecated: vec!["Old endpoint".into()],
            removed: vec!["Legacy API".into()],
            fixed: vec!["Bug fix".into()],
            security: vec!["Security patch".into()],
        };
        let json = serde_json::to_string(&sections).unwrap();
        assert!(json.contains("\"added\""));
        assert!(json.contains("\"changed\""));
        assert!(json.contains("\"deprecated\""));
        assert!(json.contains("\"removed\""));
        assert!(json.contains("\"fixed\""));
        assert!(json.contains("\"security\""));
    }
}
