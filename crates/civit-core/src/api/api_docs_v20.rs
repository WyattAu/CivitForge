#![forbid(unsafe_code)]

//! API Documentation v20 routes with enhanced rate limit documentation,
//! error code documentation, SDK generation info, and API changelog.

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
pub struct ApiDocsV20Response {
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
pub struct CreateApiDocsV20Request {
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
    "v19".into()
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
pub struct ApiDocsV20Query {
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
pub struct RateLimitDocV20 {
    pub tier: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    pub price_cents: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCodeDocV20 {
    pub code: i32,
    pub name: String,
    pub description: String,
    pub resolution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkGenerationV20 {
    pub language: String,
    pub version: String,
    pub package_url: String,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiChangelogV20 {
    pub version: String,
    pub changes: Vec<ChangelogEntryV20>,
    pub released_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntryV20 {
    pub change_type: String,
    pub endpoint: String,
    pub description: String,
}

pub async fn list_api_docs_v20(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(query): axum::extract::Query<ApiDocsV20Query>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.list_api_docs_v20(query.limit, query.offset).await {
        Ok(docs) => {
            let response: Vec<ApiDocsV20Response> = docs.iter().map(|d| ApiDocsV20Response {
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

pub async fn get_api_docs_v20_for_endpoint(
    State(state): State<AppState>,
    axum::extract::Path((endpoint, method, version)): axum::extract::Path<(String, String, String)>,
) -> impl IntoResponse {
    match state.db.get_api_docs_v20_for_endpoint(&endpoint, &method, &version).await {
        Ok(Some(doc)) => {
            let response = ApiDocsV20Response {
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

pub async fn create_api_docs_v20(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateApiDocsV20Request>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.create_api_docs_v20(
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
            let response = ApiDocsV20Response {
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

pub async fn get_rate_limit_documentation_v20(
    State(state): State<AppState>,
    axum::extract::Path((endpoint, method)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    match state.db.get_api_docs_v20_for_endpoint(&endpoint, &method, "v19").await {
        Ok(Some(doc)) => {
            let rate_limits: Vec<RateLimitDocV20> = if let Some(obj) = doc.rate_limits.as_object() {
                obj.iter().map(|(tier, data)| RateLimitDocV20 {
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

pub async fn get_error_code_documentation_v20() -> impl IntoResponse {
    let error_codes: Vec<ErrorCodeDocV20> = vec![
        ErrorCodeDocV20 { code: 400, name: "Bad Request".into(), description: "Invalid request parameters".into(), resolution: "Check request body and parameters".into() },
        ErrorCodeDocV20 { code: 401, name: "Unauthorized".into(), description: "Authentication required".into(), resolution: "Provide valid authentication credentials".into() },
        ErrorCodeDocV20 { code: 403, name: "Forbidden".into(), description: "Insufficient permissions".into(), resolution: "Request appropriate access level".into() },
        ErrorCodeDocV20 { code: 404, name: "Not Found".into(), description: "Resource not found".into(), resolution: "Verify resource exists and you have access".into() },
        ErrorCodeDocV20 { code: 429, name: "Rate Limited".into(), description: "Too many requests".into(), resolution: "Implement exponential backoff".into() },
        ErrorCodeDocV20 { code: 500, name: "Internal Error".into(), description: "Server error".into(), resolution: "Contact support if persistent".into() },
        ErrorCodeDocV20 { code: 503, name: "Service Unavailable".into(), description: "Service temporarily unavailable".into(), resolution: "Retry after backoff period".into() },
    ];
    (StatusCode::OK, Json(error_codes)).into_response()
}

pub async fn get_sdk_generation_info_v20() -> impl IntoResponse {
    let sdks: Vec<SdkGenerationV20> = vec![
        SdkGenerationV20 { language: "rust".into(), version: "0.20.0".into(), package_url: "https://crates.io/crates/civitforge-sdk".into(), generated_at: Utc::now() },
        SdkGenerationV20 { language: "python".into(), version: "0.20.0".into(), package_url: "https://pypi.org/project/civitforge-sdk/".into(), generated_at: Utc::now() },
        SdkGenerationV20 { language: "javascript".into(), version: "0.20.0".into(), package_url: "https://www.npmjs.com/package/civitforge-sdk".into(), generated_at: Utc::now() },
        SdkGenerationV20 { language: "go".into(), version: "0.20.0".into(), package_url: "https://pkg.go.dev/github.com/civitforge/sdk-go".into(), generated_at: Utc::now() },
        SdkGenerationV20 { language: "java".into(), version: "0.20.0".into(), package_url: "https://central.sonatype.com/artifact/io.github.civitforge/sdk".into(), generated_at: Utc::now() },
        SdkGenerationV20 { language: "typescript".into(), version: "0.20.0".into(), package_url: "https://www.npmjs.com/package/@civitforge/sdk-ts".into(), generated_at: Utc::now() },
    ];
    (StatusCode::OK, Json(sdks)).into_response()
}

pub async fn get_api_changelog_v20() -> impl IntoResponse {
    let changelog = ApiChangelogV20 {
        version: "v20".into(),
        changes: vec![
            ChangelogEntryV20 { change_type: "added".into(), endpoint: "/api/v20/documentation".into(), description: "New v20 documentation endpoint with enhanced rate limit docs".into() },
            ChangelogEntryV20 { change_type: "added".into(), endpoint: "/api/v19/rate-limits".into(), description: "New v19 rate limiting with enhanced alerts v15".into() },
            ChangelogEntryV20 { change_type: "added".into(), endpoint: "/api/v21/analytics".into(), description: "New v21 analytics with advanced cost tracking".into() },
            ChangelogEntryV20 { change_type: "enhanced".into(), endpoint: "/api/v20/analytics/cost".into(), description: "Enhanced cost tracking with regional breakdown".into() },
            ChangelogEntryV20 { change_type: "enhanced".into(), endpoint: "/api/v20/documentation/sdks".into(), description: "Added TypeScript SDK generation info".into() },
            ChangelogEntryV20 { change_type: "enhanced".into(), endpoint: "/api/v19/rate-limits/alerts".into(), description: "Enhanced alert analytics with notification history".into() },
        ],
        released_at: Utc::now(),
    };
    (StatusCode::OK, Json(changelog)).into_response()
}

pub fn api_docs_v20_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v20/documentation", get(list_api_docs_v20).post(create_api_docs_v20))
        .route("/api/v20/documentation/{endpoint}/{method}/{version}", get(get_api_docs_v20_for_endpoint))
        .route("/api/v20/documentation/{endpoint}/{method}/rate-limits", get(get_rate_limit_documentation_v20))
        .route("/api/v20/documentation/error-codes", get(get_error_code_documentation_v20))
        .route("/api/v20/documentation/sdks", get(get_sdk_generation_info_v20))
        .route("/api/v20/documentation/changelog", get(get_api_changelog_v20))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_docs_v20_response_serialization() {
        let response = ApiDocsV20Response {
            id: Uuid::nil(),
            endpoint: "/api/v20/repos".into(),
            method: "GET".into(),
            version: "v19".into(),
            summary: "List repositories".into(),
            description: "Returns a list of repositories".into(),
            parameters: serde_json::json!([]),
            request_body: None,
            responses: serde_json::json!({"200": {"description": "Success"}}),
            examples: serde_json::json!({}),
            tags: vec!["repos".into()],
            deprecated: false,
            changelog: "Initial v20 documentation".into(),
            security_schemes: serde_json::json!([]),
            rate_limits: serde_json::json!({"free": {"rate_limit": 100}}),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("rate_limits"));
        assert!(json.contains("security_schemes"));
    }

    #[test]
    fn test_error_code_doc_v20_serialization() {
        let doc = ErrorCodeDocV20 {
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
    fn test_sdk_generation_v20_serialization() {
        let sdk = SdkGenerationV20 {
            language: "typescript".into(),
            version: "0.20.0".into(),
            package_url: "https://www.npmjs.com/package/@civitforge/sdk-ts".into(),
            generated_at: Utc::now(),
        };
        let json = serde_json::to_string(&sdk).unwrap();
        assert!(json.contains("typescript"));
        assert!(json.contains("0.20.0"));
    }
}
