#![forbid(unsafe_code)]

//! API Documentation v4 routes with changelog tracking, breaking change detection,
//! migration guide generation, and API compatibility matrix.

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_admin};
use axum::{
    Json,
    Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDocsV4Response {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub request_body: Option<serde_json::Value>,
    pub responses: serde_json::Value,
    pub examples: serde_json::Value,
    pub tags: Vec<String>,
    pub deprecated: bool,
    pub changelog: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateApiDocsV4Request {
    pub endpoint: String,
    pub method: String,
    #[serde(default = "default_version")]
    pub version: String,
    pub summary: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub parameters: serde_json::Value,
    pub request_body: Option<serde_json::Value>,
    #[serde(default = "default_responses")]
    pub responses: serde_json::Value,
    #[serde(default = "default_examples")]
    pub examples: serde_json::Value,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub deprecated: bool,
    #[serde(default)]
    pub changelog: String,
}

fn default_version() -> String {
    "v3".into()
}

fn default_responses() -> serde_json::Value {
    serde_json::json!({})
}

fn default_examples() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiDocsV4Query {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub tag: Option<String>,
    pub version: Option<String>,
    pub deprecated: Option<bool>,
}

fn default_limit() -> i64 {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntry {
    pub version: String,
    pub changes: Vec<ChangeItem>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeItem {
    pub change_type: String,
    pub description: String,
    pub breaking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChangeDetection {
    pub endpoint: String,
    pub method: String,
    pub old_version: String,
    pub new_version: String,
    pub breaking_changes: Vec<BreakingChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChange {
    pub field: String,
    pub change_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationGuide {
    pub from_version: String,
    pub to_version: String,
    pub affected_endpoints: Vec<String>,
    pub steps: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityMatrix {
    pub versions: Vec<String>,
    pub endpoints: Vec<CompatibilityEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityEntry {
    pub endpoint: String,
    pub method: String,
    pub supported_versions: Vec<String>,
    pub deprecated_versions: Vec<String>,
}

pub async fn list_api_docs_v4(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(query): axum::extract::Query<ApiDocsV4Query>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.list_api_docs_v4(query.limit, query.offset).await {
        Ok(docs) => {
            let response: Vec<ApiDocsV4Response> = docs.iter().map(|d| ApiDocsV4Response {
                id: d.id,
                endpoint: d.endpoint.clone(),
                method: d.method.clone(),
                version: d.version.clone(),
                summary: d.summary.clone(),
                description: d.description.clone(),
                parameters: d.parameters.clone(),
                request_body: d.request_body.clone(),
                responses: d.responses.clone(),
                examples: d.examples.clone(),
                tags: d.tags.clone(),
                deprecated: d.deprecated,
                changelog: d.changelog.clone(),
                created_at: d.created_at,
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

pub async fn get_api_docs_v4_for_endpoint(
    State(state): State<AppState>,
    axum::extract::Path((endpoint, method, version)): axum::extract::Path<(String, String, String)>,
) -> impl IntoResponse {
    match state.db.get_api_docs_v4_for_endpoint(&endpoint, &method, &version).await {
        Ok(Some(doc)) => {
            let response = ApiDocsV4Response {
                id: doc.id,
                endpoint: doc.endpoint,
                method: doc.method,
                version: doc.version,
                summary: doc.summary,
                description: doc.description,
                parameters: doc.parameters,
                request_body: doc.request_body,
                responses: doc.responses,
                examples: doc.examples,
                tags: doc.tags,
                deprecated: doc.deprecated,
                changelog: doc.changelog,
                created_at: doc.created_at,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Endpoint documentation not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create_api_docs_v4(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateApiDocsV4Request>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.create_api_docs_v4(
        &req.endpoint,
        &req.method,
        &req.version,
        &req.summary,
        &req.description,
        &req.parameters,
        req.request_body.as_ref(),
        &req.responses,
        &req.examples,
        &req.tags,
        req.deprecated,
        &req.changelog,
    ).await {
        Ok(doc) => {
            let response = ApiDocsV4Response {
                id: doc.id,
                endpoint: doc.endpoint,
                method: doc.method,
                version: doc.version,
                summary: doc.summary,
                description: doc.description,
                parameters: doc.parameters,
                request_body: doc.request_body,
                responses: doc.responses,
                examples: doc.examples,
                tags: doc.tags,
                deprecated: doc.deprecated,
                changelog: doc.changelog,
                created_at: doc.created_at,
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

pub async fn get_changelog(
    State(state): State<AppState>,
    axum::extract::Path((endpoint, method)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    match state.db.get_changelog_for_endpoint(&endpoint, &method).await {
        Ok(entries) => {
            let response: Vec<ChangelogEntry> = entries.iter().map(|d| ChangelogEntry {
                version: d.version.clone(),
                changes: vec![ChangeItem {
                    change_type: if d.deprecated { "deprecated".into() } else { "added".into() },
                    description: d.changelog.clone(),
                    breaking: d.deprecated,
                }],
                timestamp: d.created_at,
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

pub async fn detect_breaking_changes(
    State(state): State<AppState>,
    axum::extract::Path((endpoint, method)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    match state.db.detect_breaking_changes(&endpoint, &method).await {
        Ok(detection) => {
            let response = BreakingChangeDetection {
                endpoint: detection.0,
                method: detection.1,
                old_version: detection.2,
                new_version: detection.3,
                breaking_changes: detection.4.into_iter().map(|(field, change_type, desc)| BreakingChange {
                    field,
                    change_type,
                    description: desc,
                }).collect(),
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

pub async fn generate_migration_guide(
    State(state): State<AppState>,
    axum::extract::Path((from_version, to_version)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    match state.db.generate_migration_guide(&from_version, &to_version).await {
        Ok((affected, steps, notes)) => {
            let response = MigrationGuide {
                from_version,
                to_version,
                affected_endpoints: affected,
                steps,
                notes,
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

pub async fn get_compatibility_matrix(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.db.get_api_compatibility_matrix().await {
        Ok(matrix) => {
            (StatusCode::OK, Json(matrix)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub fn api_docs_v4_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v4/documentation", get(list_api_docs_v4).post(create_api_docs_v4))
        .route("/api/v4/documentation/{endpoint}/{method}/{version}", get(get_api_docs_v4_for_endpoint))
        .route("/api/v4/documentation/{endpoint}/{method}/changelog", get(get_changelog))
        .route("/api/v4/documentation/{endpoint}/{method}/breaking-changes", get(detect_breaking_changes))
        .route("/api/v4/documentation/migration/{from}/{to}", get(generate_migration_guide))
        .route("/api/v4/documentation/compatibility-matrix", get(get_compatibility_matrix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_docs_v4_response_serialization() {
        let response = ApiDocsV4Response {
            id: Uuid::nil(),
            endpoint: "/api/v1/repos".into(),
            method: "GET".into(),
            version: "v3".into(),
            summary: "List repositories".into(),
            description: "Returns a list of repositories".into(),
            parameters: serde_json::json!([]),
            request_body: None,
            responses: serde_json::json!({"200": {"description": "Success"}}),
            examples: serde_json::json!({}),
            tags: vec!["repos".into()],
            deprecated: false,
            changelog: "Initial v4 documentation".into(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("/api/v1/repos"));
        assert!(json.contains("\"method\":\"GET\""));
        assert!(json.contains("\"version\":\"v3\""));
        assert!(json.contains("changelog"));
    }

    #[test]
    fn test_create_request_defaults() {
        let req = CreateApiDocsV4Request {
            endpoint: "/test".into(),
            method: "GET".into(),
            version: "v3".into(),
            summary: "Test".into(),
            description: String::new(),
            parameters: serde_json::json!([]),
            request_body: None,
            responses: serde_json::json!({}),
            examples: serde_json::json!({}),
            tags: vec![],
            deprecated: false,
            changelog: String::new(),
        };
        assert_eq!(req.endpoint, "/test");
        assert_eq!(req.version, "v3");
        assert!(!req.deprecated);
    }
}
