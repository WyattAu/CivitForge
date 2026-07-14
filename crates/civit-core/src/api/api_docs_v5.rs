#![forbid(unsafe_code)]

//! API Documentation v5 routes with security scheme documentation,
//! OAuth flow documentation, API key documentation, and rate limit documentation.

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
pub struct ApiDocsV5Response {
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
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateApiDocsV5Request {
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
}

fn default_version() -> String {
    "v4".into()
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

#[derive(Debug, Clone, Deserialize)]
pub struct ApiDocsV5Query {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub tag: Option<String>,
    pub version: Option<String>,
    pub deprecated: Option<bool>,
    pub security_type: Option<String>,
}

fn default_limit() -> i64 {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScheme {
    pub scheme_type: String,
    pub name: String,
    pub description: String,
    pub flows: Option<serde_json::Value>,
    pub bearer_format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthFlowDocumentation {
    pub flow_type: String,
    pub authorization_url: String,
    pub token_url: String,
    pub scopes: Vec<OAuthScope>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthScope {
    pub scope: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyDocumentation {
    pub header_name: String,
    pub parameter_name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitDocumentation {
    pub tier: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
}

pub async fn list_api_docs_v5(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(query): axum::extract::Query<ApiDocsV5Query>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.list_api_docs_v5(query.limit, query.offset).await {
        Ok(docs) => {
            let response: Vec<ApiDocsV5Response> = docs.iter().map(|d| ApiDocsV5Response {
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

pub async fn get_api_docs_v5_for_endpoint(
    State(state): State<AppState>,
    axum::extract::Path((endpoint, method, version)): axum::extract::Path<(String, String, String)>,
) -> impl IntoResponse {
    match state.db.get_api_docs_v5_for_endpoint(&endpoint, &method, &version).await {
        Ok(Some(doc)) => {
            let response = ApiDocsV5Response {
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

pub async fn create_api_docs_v5(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateApiDocsV5Request>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.create_api_docs_v5(
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
    ).await {
        Ok(doc) => {
            let response = ApiDocsV5Response {
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

pub async fn get_security_schemes(
    State(state): State<AppState>,
    axum::extract::Path((endpoint, method)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    match state.db.get_security_schemes_for_endpoint(&endpoint, &method).await {
        Ok(schemes) => (StatusCode::OK, Json(schemes)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_oauth_flows(
    State(state): State<AppState>,
    axum::extract::Path((endpoint, method)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    match state.db.get_security_schemes_for_endpoint(&endpoint, &method).await {
        Ok(schemes) => {
            let oauth_flows: Vec<OAuthFlowDocumentation> = if let Some(arr) = schemes.as_array() {
                arr.iter()
                    .filter(|s| s.get("scheme_type").and_then(|v| v.as_str()) == Some("oauth2"))
                    .filter_map(|s| {
                        let flows = s.get("flows")?;
                        let flow_type = flows.as_object()?.keys().next()?.clone();
                        let flow_data = flows.get(&flow_type)?;
                        Some(OAuthFlowDocumentation {
                            flow_type,
                            authorization_url: flow_data.get("authorizationUrl")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .into(),
                            token_url: flow_data.get("tokenUrl")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .into(),
                            scopes: flows.get("scopes")
                                .and_then(|v| v.as_object())
                                .map(|obj| {
                                    obj.iter().map(|(scope, desc)| OAuthScope {
                                        scope: scope.clone(),
                                        description: desc.as_str().unwrap_or("").into(),
                                    }).collect()
                                })
                                .unwrap_or_default(),
                        })
                    })
                    .collect()
            } else {
                vec![]
            };
            (StatusCode::OK, Json(oauth_flows)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_api_key_documentation(
    State(state): State<AppState>,
    axum::extract::Path((endpoint, method)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    match state.db.get_security_schemes_for_endpoint(&endpoint, &method).await {
        Ok(schemes) => {
            let api_keys: Vec<ApiKeyDocumentation> = if let Some(arr) = schemes.as_array() {
                arr.iter()
                    .filter(|s| s.get("scheme_type").and_then(|v| v.as_str()) == Some("apiKey"))
                    .filter_map(|s| Some(ApiKeyDocumentation {
                        header_name: s.get("name").and_then(|v| v.as_str()).unwrap_or("X-API-Key").into(),
                        parameter_name: s.get("parameter_name").and_then(|v| v.as_str()).unwrap_or("api_key").into(),
                        description: s.get("description").and_then(|v| v.as_str()).unwrap_or("").into(),
                    }))
                    .collect()
            } else {
                vec![]
            };
            (StatusCode::OK, Json(api_keys)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub fn api_docs_v5_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v5/documentation", get(list_api_docs_v5).post(create_api_docs_v5))
        .route("/api/v5/documentation/{endpoint}/{method}/{version}", get(get_api_docs_v5_for_endpoint))
        .route("/api/v5/documentation/{endpoint}/{method}/security-schemes", get(get_security_schemes))
        .route("/api/v5/documentation/{endpoint}/{method}/oauth-flows", get(get_oauth_flows))
        .route("/api/v5/documentation/{endpoint}/{method}/api-keys", get(get_api_key_documentation))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_docs_v5_response_serialization() {
        let response = ApiDocsV5Response {
            id: Uuid::nil(),
            endpoint: "/api/v1/repos".into(),
            method: "GET".into(),
            version: "v4".into(),
            summary: "List repositories".into(),
            description: "Returns a list of repositories".into(),
            parameters: serde_json::json!([]),
            request_body: None,
            responses: serde_json::json!({"200": {"description": "Success"}}),
            examples: serde_json::json!({}),
            tags: vec!["repos".into()],
            deprecated: false,
            changelog: "Initial v5 documentation".into(),
            security_schemes: serde_json::json!([{"type": "oauth2", "flows": {"authorizationCode": {"authorizationUrl": "/oauth/authorize", "tokenUrl": "/oauth/token", "scopes": {"read": "Read access"}}}}]),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("/api/v1/repos"));
        assert!(json.contains("security_schemes"));
        assert!(json.contains("oauth2"));
    }

    #[test]
    fn test_create_request_defaults() {
        let req = CreateApiDocsV5Request {
            endpoint: "/test".into(),
            method: "GET".into(),
            version: "v4".into(),
            summary: "Test".into(),
            description: String::new(),
            parameters: serde_json::json!([]),
            request_body: None,
            responses: serde_json::json!({}),
            examples: serde_json::json!({}),
            tags: vec![],
            deprecated: false,
            changelog: String::new(),
            security_schemes: serde_json::json!([]),
        };
        assert_eq!(req.endpoint, "/test");
        assert_eq!(req.version, "v4");
        assert!(!req.deprecated);
    }
}
