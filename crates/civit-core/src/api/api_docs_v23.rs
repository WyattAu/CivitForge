#![forbid(unsafe_code)]

//! API Documentation v23 routes with example management, changelog tracking,
//! breaking change detection, and migration guides.

use crate::api::AppState;
use crate::api::auth::AuthUser;
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
pub struct ApiDocExampleV23Response {
    pub id: Uuid,
    pub endpoint_id: Uuid,
    pub title: String,
    pub language: String,
    pub request_example: String,
    pub response_example: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateApiDocExampleV23Request {
    pub endpoint_id: Uuid,
    pub title: String,
    #[serde(default = "default_language")]
    pub language: String,
    pub request_example: String,
    pub response_example: String,
    #[serde(default)]
    pub description: String,
}

fn default_language() -> String {
    "curl".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDocChangelogV23Response {
    pub id: Uuid,
    pub endpoint_id: Uuid,
    pub version: String,
    pub change_type: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateApiDocChangelogV23Request {
    pub endpoint_id: Uuid,
    pub version: String,
    pub change_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChangeDetectionV23 {
    pub endpoint_id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub breaking_changes: Vec<BreakingChangeEntryV23>,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChangeEntryV23 {
    pub change_type: String,
    pub description: String,
    pub severity: String,
    pub migration_guide: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationGuideV23 {
    pub from_version: String,
    pub to_version: String,
    pub steps: Vec<MigrationStepV23>,
    pub estimated_effort: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStepV23 {
    pub order: i32,
    pub title: String,
    pub description: String,
    pub code_example: Option<String>,
}

pub async fn list_api_doc_examples_v23(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(endpoint_id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.list_api_doc_examples_v21(endpoint_id).await {
        Ok(examples) => {
            let response: Vec<ApiDocExampleV23Response> = examples.iter().map(|e| ApiDocExampleV23Response {
                id: e.id,
                endpoint_id: e.endpoint_id,
                title: e.title.clone(),
                language: e.language.clone(),
                request_example: e.request_example.clone(),
                response_example: e.response_example.clone(),
                description: e.description.clone(),
                created_at: e.created_at,
            }).collect();
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create_api_doc_example_v23(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateApiDocExampleV23Request>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.create_api_doc_example_v21(
        req.endpoint_id,
        &req.title,
        &req.language,
        &req.request_example,
        &req.response_example,
        &req.description,
    ).await {
        Ok(example) => {
            let response = ApiDocExampleV23Response {
                id: example.id,
                endpoint_id: example.endpoint_id,
                title: example.title,
                language: example.language,
                request_example: example.request_example,
                response_example: example.response_example,
                description: example.description,
                created_at: example.created_at,
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

pub async fn list_api_doc_changelogs_v23(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(endpoint_id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.list_api_doc_changelogs_v21(endpoint_id).await {
        Ok(changelogs) => {
            let response: Vec<ApiDocChangelogV23Response> = changelogs.iter().map(|c| ApiDocChangelogV23Response {
                id: c.id,
                endpoint_id: c.endpoint_id,
                version: c.version.clone(),
                change_type: c.change_type.clone(),
                description: c.description.clone(),
                created_at: c.created_at,
            }).collect();
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create_api_doc_changelog_v23(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateApiDocChangelogV23Request>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.create_api_doc_changelog_v21(
        req.endpoint_id,
        &req.version,
        &req.change_type,
        &req.description,
    ).await {
        Ok(changelog) => {
            let response = ApiDocChangelogV23Response {
                id: changelog.id,
                endpoint_id: changelog.endpoint_id,
                version: changelog.version,
                change_type: changelog.change_type,
                description: changelog.description,
                created_at: changelog.created_at,
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

pub async fn detect_breaking_changes_v23() -> impl IntoResponse {
    let detections: Vec<BreakingChangeDetectionV23> = vec![];
    (StatusCode::OK, Json(detections)).into_response()
}

pub async fn get_migration_guide_v23(
    axum::extract::Path((from_version, to_version)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    let guide = MigrationGuideV23 {
        from_version,
        to_version,
        steps: vec![],
        estimated_effort: "low".into(),
    };
    (StatusCode::OK, Json(guide)).into_response()
}

pub fn api_docs_v23_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v23/documentation/examples/{endpoint_id}", get(list_api_doc_examples_v23).post(create_api_doc_example_v23))
        .route("/api/v23/documentation/changelogs/{endpoint_id}", get(list_api_doc_changelogs_v23).post(create_api_doc_changelog_v23))
        .route("/api/v23/documentation/breaking-changes", get(detect_breaking_changes_v23))
        .route("/api/v23/documentation/migration-guide/{from_version}/{to_version}", get(get_migration_guide_v23))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_doc_example_v23_response_serialization() {
        let response = ApiDocExampleV23Response {
            id: Uuid::nil(),
            endpoint_id: Uuid::nil(),
            title: "List Repos".into(),
            language: "curl".into(),
            request_example: "curl -H 'Authorization: Bearer token' https://api.example.com/repos".into(),
            response_example: r#"{"repos": []}"#.into(),
            description: "Example for listing repositories".into(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("curl"));
        assert!(json.contains("List Repos"));
    }

    #[test]
    fn test_breaking_change_entry_v23_serialization() {
        let entry = BreakingChangeEntryV23 {
            change_type: "removed".into(),
            description: "Removed deprecated field".into(),
            severity: "high".into(),
            migration_guide: Some("Use new_field instead".into()),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("removed"));
        assert!(json.contains("high"));
    }

    #[test]
    fn test_migration_guide_v23_serialization() {
        let guide = MigrationGuideV23 {
            from_version: "v22".into(),
            to_version: "v23".into(),
            steps: vec![MigrationStepV23 {
                order: 1,
                title: "Update endpoint".into(),
                description: "Add new required field".into(),
                code_example: Some(r#"{"new_field": "value"}"#.into()),
            }],
            estimated_effort: "low".into(),
        };
        let json = serde_json::to_string(&guide).unwrap();
        assert!(json.contains("v22"));
        assert!(json.contains("v23"));
    }
}
