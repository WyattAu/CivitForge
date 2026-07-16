#![forbid(unsafe_code)]

//! API Documentation v17 routes with enhanced rate limit documentation,
//! error code documentation, SDK generation info, and API changelog.

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
pub struct ApiDocsV17Response {
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
    pub security_schemes: serde_json::Value,
    pub rate_limits: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateApiDocsV17Request {
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
    #[serde(default = "default_security_schemes")]
    pub security_schemes: serde_json::Value,
    #[serde(default = "default_rate_limits")]
    pub rate_limits: serde_json::Value,
}

fn default_version() -> String {
    "v16".into()
}

fn default_responses() -> serde_json::Value {
    serde_json::json!({})
}

fn default_examples() -> serde_json::Value {
    serde_json::json!({})
}

fn default_security_schemes() -> serde_json::Value {
    serde_json::json!([])
}

fn default_rate_limits() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiDocsV17Query {
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
pub struct RateLimitDocV17 {
    pub tier: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    pub price_cents: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCodeDocV17 {
    pub code: i32,
    pub name: String,
    pub description: String,
    pub resolution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkGenerationV17 {
    pub language: String,
    pub version: String,
    pub package_url: String,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiChangelogV17 {
    pub version: String,
    pub changes: Vec<ChangelogEntryV17>,
    pub released_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntryV17 {
    pub change_type: String,
    pub endpoint: String,
    pub description: String,
}

pub async fn list_api_docs_v17(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(query): axum::extract::Query<ApiDocsV17Query>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.list_api_docs_v17(query.limit, query.offset).await {
        Ok(docs) => {
            let response: Vec<ApiDocsV17Response> = docs.iter().map(|d| ApiDocsV17Response {
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
                security_schemes: d.security_schemes.clone(),
                rate_limits: d.rate_limits.clone(),
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

pub async fn get_api_docs_v17_for_endpoint(
    State(state): State<AppState>,
    axum::extract::Path((endpoint, method, version)): axum::extract::Path<(String, String, String)>,
) -> impl IntoResponse {
    match state.db.get_api_docs_v17_for_endpoint(&endpoint, &method, &version).await {
        Ok(Some(doc)) => {
            let response = ApiDocsV17Response {
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
                security_schemes: doc.security_schemes,
                rate_limits: doc.rate_limits,
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

pub async fn create_api_docs_v17(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateApiDocsV17Request>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.create_api_docs_v17(
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
        &req.security_schemes,
        &req.rate_limits,
    ).await {
        Ok(doc) => {
            let response = ApiDocsV17Response {
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
                security_schemes: doc.security_schemes,
                rate_limits: doc.rate_limits,
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

pub async fn get_rate_limit_documentation_v17(
    State(state): State<AppState>,
    axum::extract::Path((endpoint, method)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    match state.db.get_api_docs_v17_for_endpoint(&endpoint, &method, "v16").await {
        Ok(Some(doc)) => {
            let rate_limits: Vec<RateLimitDocV17> = if let Some(obj) = doc.rate_limits.as_object() {
                obj.iter().map(|(tier, data)| RateLimitDocV17 {
                    tier: tier.clone(),
                    rate_limit: data.get("rate_limit").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                    burst_limit: data.get("burst_limit").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                    monthly_quota: data.get("monthly_quota").and_then(|v| v.as_i64()).map(|v| v as i32),
                    price_cents: data.get("price_cents").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                }).collect()
            } else {
                vec![]
            };
            (StatusCode::OK, Json(rate_limits)).into_response()
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

pub async fn get_error_code_documentation_v17() -> impl IntoResponse {
    let error_codes: Vec<ErrorCodeDocV17> = vec![
        ErrorCodeDocV17 { code: 400, name: "Bad Request".into(), description: "Invalid request parameters".into(), resolution: "Check request body and parameters".into() },
        ErrorCodeDocV17 { code: 401, name: "Unauthorized".into(), description: "Authentication required".into(), resolution: "Provide valid authentication credentials".into() },
        ErrorCodeDocV17 { code: 403, name: "Forbidden".into(), description: "Insufficient permissions".into(), resolution: "Request appropriate access level".into() },
        ErrorCodeDocV17 { code: 404, name: "Not Found".into(), description: "Resource not found".into(), resolution: "Verify resource exists and you have access".into() },
        ErrorCodeDocV17 { code: 429, name: "Rate Limited".into(), description: "Too many requests".into(), resolution: "Implement exponential backoff".into() },
        ErrorCodeDocV17 { code: 500, name: "Internal Error".into(), description: "Server error".into(), resolution: "Contact support if persistent".into() },
        ErrorCodeDocV17 { code: 503, name: "Service Unavailable".into(), description: "Service temporarily unavailable".into(), resolution: "Retry after backoff period".into() },
    ];
    (StatusCode::OK, Json(error_codes)).into_response()
}

pub async fn get_sdk_generation_info_v17() -> impl IntoResponse {
    let sdks: Vec<SdkGenerationV17> = vec![
        SdkGenerationV17 { language: "rust".into(), version: "0.17.0".into(), package_url: "https://crates.io/crates/civitforge-sdk".into(), generated_at: Utc::now() },
        SdkGenerationV17 { language: "python".into(), version: "0.17.0".into(), package_url: "https://pypi.org/project/civitforge-sdk/".into(), generated_at: Utc::now() },
        SdkGenerationV17 { language: "javascript".into(), version: "0.17.0".into(), package_url: "https://www.npmjs.com/package/civitforge-sdk".into(), generated_at: Utc::now() },
        SdkGenerationV17 { language: "go".into(), version: "0.17.0".into(), package_url: "https://pkg.go.dev/github.com/civitforge/sdk-go".into(), generated_at: Utc::now() },
        SdkGenerationV17 { language: "java".into(), version: "0.17.0".into(), package_url: "https://central.sonatype.com/artifact/io.github.civitforge/sdk".into(), generated_at: Utc::now() },
    ];
    (StatusCode::OK, Json(sdks)).into_response()
}

pub async fn get_api_changelog_v17() -> impl IntoResponse {
    let changelog = ApiChangelogV17 {
        version: "v17".into(),
        changes: vec![
            ChangelogEntryV17 { change_type: "added".into(), endpoint: "/api/v17/documentation".into(), description: "New v17 documentation endpoint with enhanced rate limit docs".into() },
            ChangelogEntryV17 { change_type: "added".into(), endpoint: "/api/v17/rate-limits".into(), description: "New v16 rate limiting with enhanced alerts v12".into() },
            ChangelogEntryV17 { change_type: "added".into(), endpoint: "/api/v17/analytics".into(), description: "New v18 analytics with advanced cost tracking".into() },
            ChangelogEntryV17 { change_type: "enhanced".into(), endpoint: "/api/v17/analytics/cost".into(), description: "Enhanced cost tracking with regional breakdown".into() },
            ChangelogEntryV17 { change_type: "enhanced".into(), endpoint: "/api/v17/documentation/sdks".into(), description: "Added Java SDK generation info".into() },
            ChangelogEntryV17 { change_type: "enhanced".into(), endpoint: "/api/v17/rate-limits/alerts".into(), description: "Enhanced alert analytics with notification history".into() },
        ],
        released_at: Utc::now(),
    };
    (StatusCode::OK, Json(changelog)).into_response()
}

pub fn api_docs_v17_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v17/documentation", get(list_api_docs_v17).post(create_api_docs_v17))
        .route("/api/v17/documentation/{endpoint}/{method}/{version}", get(get_api_docs_v17_for_endpoint))
        .route("/api/v17/documentation/{endpoint}/{method}/rate-limits", get(get_rate_limit_documentation_v17))
        .route("/api/v17/documentation/error-codes", get(get_error_code_documentation_v17))
        .route("/api/v17/documentation/sdks", get(get_sdk_generation_info_v17))
        .route("/api/v17/documentation/changelog", get(get_api_changelog_v17))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_docs_v17_response_serialization() {
        let response = ApiDocsV17Response {
            id: Uuid::nil(),
            endpoint: "/api/v17/repos".into(),
            method: "GET".into(),
            version: "v16".into(),
            summary: "List repositories".into(),
            description: "Returns a list of repositories".into(),
            parameters: serde_json::json!([]),
            request_body: None,
            responses: serde_json::json!({"200": {"description": "Success"}}),
            examples: serde_json::json!({}),
            tags: vec!["repos".into()],
            deprecated: false,
            changelog: "Initial v17 documentation".into(),
            security_schemes: serde_json::json!([]),
            rate_limits: serde_json::json!({"free": {"rate_limit": 100}}),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("rate_limits"));
        assert!(json.contains("security_schemes"));
    }

    #[test]
    fn test_error_code_doc_v17_serialization() {
        let doc = ErrorCodeDocV17 {
            code: 503,
            name: "Service Unavailable".into(),
            description: "Service temporarily unavailable".into(),
            resolution: "Retry after backoff period".into(),
        };
        let json = serde_json::to_string(&doc).unwrap();
        assert!(json.contains("503"));
        assert!(json.contains("Service Unavailable"));
    }

    #[test]
    fn test_sdk_generation_v17_serialization() {
        let sdk = SdkGenerationV17 {
            language: "java".into(),
            version: "0.17.0".into(),
            package_url: "https://central.sonatype.com/artifact/io.github.civitforge/sdk".into(),
            generated_at: Utc::now(),
        };
        let json = serde_json::to_string(&sdk).unwrap();
        assert!(json.contains("java"));
        assert!(json.contains("0.17.0"));
    }
}
