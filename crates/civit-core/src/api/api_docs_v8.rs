#![forbid(unsafe_code)]

//! API Documentation v8 routes with rate limit documentation,
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
pub struct ApiDocsV8Response {
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
pub struct CreateApiDocsV8Request {
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
    "v7".into()
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
pub struct ApiDocsV8Query {
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
pub struct RateLimitDocV8 {
    pub tier: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    pub price_cents: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCodeDocV8 {
    pub code: i32,
    pub name: String,
    pub description: String,
    pub resolution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkGenerationV8 {
    pub language: String,
    pub version: String,
    pub package_url: String,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiChangelogV8 {
    pub version: String,
    pub changes: Vec<ChangelogEntryV8>,
    pub released_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntryV8 {
    pub change_type: String,
    pub endpoint: String,
    pub description: String,
}

pub async fn list_api_docs_v8(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(query): axum::extract::Query<ApiDocsV8Query>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.list_api_docs_v8(query.limit, query.offset).await {
        Ok(docs) => {
            let response: Vec<ApiDocsV8Response> = docs.iter().map(|d| ApiDocsV8Response {
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

pub async fn get_api_docs_v8_for_endpoint(
    State(state): State<AppState>,
    axum::extract::Path((endpoint, method, version)): axum::extract::Path<(String, String, String)>,
) -> impl IntoResponse {
    match state.db.get_api_docs_v8_for_endpoint(&endpoint, &method, &version).await {
        Ok(Some(doc)) => {
            let response = ApiDocsV8Response {
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

pub async fn create_api_docs_v8(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateApiDocsV8Request>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.create_api_docs_v8(
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
            let response = ApiDocsV8Response {
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

pub async fn get_rate_limit_documentation(
    State(state): State<AppState>,
    axum::extract::Path((endpoint, method)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    match state.db.get_api_docs_v8_for_endpoint(&endpoint, &method, "v7").await {
        Ok(Some(doc)) => {
            let rate_limits: Vec<RateLimitDocV8> = if let Some(obj) = doc.rate_limits.as_object() {
                obj.iter().map(|(tier, data)| RateLimitDocV8 {
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

pub async fn get_error_code_documentation() -> impl IntoResponse {
    let error_codes: Vec<ErrorCodeDocV8> = vec![
        ErrorCodeDocV8 { code: 400, name: "Bad Request".into(), description: "Invalid request parameters".into(), resolution: "Check request body and parameters".into() },
        ErrorCodeDocV8 { code: 401, name: "Unauthorized".into(), description: "Authentication required".into(), resolution: "Provide valid authentication credentials".into() },
        ErrorCodeDocV8 { code: 403, name: "Forbidden".into(), description: "Insufficient permissions".into(), resolution: "Request appropriate access level".into() },
        ErrorCodeDocV8 { code: 404, name: "Not Found".into(), description: "Resource not found".into(), resolution: "Verify resource exists and you have access".into() },
        ErrorCodeDocV8 { code: 429, name: "Rate Limited".into(), description: "Too many requests".into(), resolution: "Implement exponential backoff".into() },
        ErrorCodeDocV8 { code: 500, name: "Internal Error".into(), description: "Server error".into(), resolution: "Contact support if persistent".into() },
    ];
    (StatusCode::OK, Json(error_codes)).into_response()
}

pub async fn get_sdk_generation_info() -> impl IntoResponse {
    let sdks: Vec<SdkGenerationV8> = vec![
        SdkGenerationV8 { language: "rust".into(), version: "0.8.0".into(), package_url: "https://crates.io/crates/civitforge-sdk".into(), generated_at: Utc::now() },
        SdkGenerationV8 { language: "python".into(), version: "0.8.0".into(), package_url: "https://pypi.org/project/civitforge-sdk/".into(), generated_at: Utc::now() },
        SdkGenerationV8 { language: "javascript".into(), version: "0.8.0".into(), package_url: "https://www.npmjs.com/package/civitforge-sdk".into(), generated_at: Utc::now() },
    ];
    (StatusCode::OK, Json(sdks)).into_response()
}

pub async fn get_api_changelog() -> impl IntoResponse {
    let changelog = ApiChangelogV8 {
        version: "v8".into(),
        changes: vec![
            ChangelogEntryV8 { change_type: "added".into(), endpoint: "/api/v8/documentation".into(), description: "New v8 documentation endpoint with rate limit docs".into() },
            ChangelogEntryV8 { change_type: "added".into(), endpoint: "/api/v8/rate-limits".into(), description: "New v6 rate limiting with enhanced alerts".into() },
            ChangelogEntryV8 { change_type: "added".into(), endpoint: "/api/v7/analytics".into(), description: "New v9 analytics with cost tracking".into() },
        ],
        released_at: Utc::now(),
    };
    (StatusCode::OK, Json(changelog)).into_response()
}

pub fn api_docs_v8_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v8/documentation", get(list_api_docs_v8).post(create_api_docs_v8))
        .route("/api/v8/documentation/{endpoint}/{method}/{version}", get(get_api_docs_v8_for_endpoint))
        .route("/api/v8/documentation/{endpoint}/{method}/rate-limits", get(get_rate_limit_documentation))
        .route("/api/v8/error-codes", get(get_error_code_documentation))
        .route("/api/v8/sdks", get(get_sdk_generation_info))
        .route("/api/v8/changelog", get(get_api_changelog))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_docs_v8_response_serialization() {
        let response = ApiDocsV8Response {
            id: Uuid::nil(),
            endpoint: "/api/v1/repos".into(),
            method: "GET".into(),
            version: "v7".into(),
            summary: "List repositories".into(),
            description: "Returns a list of repositories".into(),
            parameters: serde_json::json!([]),
            request_body: None,
            responses: serde_json::json!({"200": {"description": "Success"}}),
            examples: serde_json::json!({}),
            tags: vec!["repos".into()],
            deprecated: false,
            changelog: "Initial v8 documentation".into(),
            security_schemes: serde_json::json!([]),
            rate_limits: serde_json::json!({"free": {"rate_limit": 100}}),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("rate_limits"));
        assert!(json.contains("security_schemes"));
    }

    #[test]
    fn test_error_code_doc_serialization() {
        let doc = ErrorCodeDocV8 {
            code: 429,
            name: "Rate Limited".into(),
            description: "Too many requests".into(),
            resolution: "Implement exponential backoff".into(),
        };
        let json = serde_json::to_string(&doc).unwrap();
        assert!(json.contains("429"));
        assert!(json.contains("Rate Limited"));
    }
}
