#![forbid(unsafe_code)]

//! API documentation routes for auto-generated OpenAPI documentation,
//! interactive playground, and endpoint metadata.

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
pub struct ApiDocumentationResponse {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub summary: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub request_body: Option<serde_json::Value>,
    pub responses: serde_json::Value,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateApiDocumentationRequest {
    pub endpoint: String,
    pub method: String,
    pub summary: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub parameters: serde_json::Value,
    pub request_body: Option<serde_json::Value>,
    #[serde(default = "default_responses")]
    pub responses: serde_json::Value,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_responses() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocumentationQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub tag: Option<String>,
}

fn default_limit() -> i64 {
    100
}

pub async fn list_api_documentation(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<DocumentationQuery>,
) -> impl IntoResponse {
    let result = if let Some(tag) = &query.tag {
        state.db.search_api_documentation_by_tag(tag).await
    } else {
        state.db.list_api_documentation(query.limit, query.offset).await
    };

    match result {
        Ok(docs) => {
            let response: Vec<ApiDocumentationResponse> = docs.iter().map(|d| ApiDocumentationResponse {
                id: d.id,
                endpoint: d.endpoint.clone(),
                method: d.method.clone(),
                summary: d.summary.clone(),
                description: d.description.clone(),
                parameters: d.parameters.clone(),
                request_body: d.request_body.clone(),
                responses: d.responses.clone(),
                tags: d.tags.clone(),
                created_at: d.created_at,
            }).collect();
            (StatusCode::OK, Json(serde_json::json!({"documentation": response, "total": response.len()}))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_api_documentation_for_endpoint(
    State(state): State<AppState>,
    axum::extract::Path((endpoint, method)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    match state.db.get_api_documentation(&endpoint, &method).await {
        Ok(doc) => {
            let response = ApiDocumentationResponse {
                id: doc.id,
                endpoint: doc.endpoint,
                method: doc.method,
                summary: doc.summary,
                description: doc.description,
                parameters: doc.parameters,
                request_body: doc.request_body,
                responses: doc.responses,
                tags: doc.tags,
                created_at: doc.created_at,
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

pub async fn create_api_documentation(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateApiDocumentationRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.create_api_documentation(
        &req.endpoint,
        &req.method,
        &req.summary,
        &req.description,
        req.parameters,
        req.request_body,
        req.responses,
        &req.tags,
    ).await {
        Ok(doc) => {
            let response = ApiDocumentationResponse {
                id: doc.id,
                endpoint: doc.endpoint,
                method: doc.method,
                summary: doc.summary,
                description: doc.description,
                parameters: doc.parameters,
                request_body: doc.request_body,
                responses: doc.responses,
                tags: doc.tags,
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

pub async fn get_api_playground() -> impl IntoResponse {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CivitForge API Playground</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; }
        h1 { color: #333; }
        .endpoint-list { display: grid; gap: 10px; }
        .endpoint { background: white; padding: 15px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); cursor: pointer; }
        .endpoint:hover { box-shadow: 0 4px 8px rgba(0,0,0,0.15); }
        .method { display: inline-block; padding: 2px 8px; border-radius: 4px; color: white; font-weight: bold; font-size: 12px; margin-right: 10px; }
        .method-GET { background: #61affe; }
        .method-POST { background: #49cc90; }
        .method-PUT { background: #fca130; }
        .method-DELETE { background: #f93e3e; }
        .method-PATCH { background: #50e3c2; }
        .path { font-family: monospace; font-size: 14px; }
        .summary { color: #666; margin-top: 5px; }
        .try-it { margin-top: 20px; background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .try-it h2 { margin-top: 0; }
        textarea, input { width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; font-family: monospace; margin-bottom: 10px; box-sizing: border-box; }
        button { background: #49cc90; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer; font-size: 14px; }
        button:hover { background: #3daa80; }
        pre { background: #f8f8f8; padding: 15px; border-radius: 4px; overflow-x: auto; }
    </style>
</head>
<body>
    <div class="container">
        <h1>CivitForge API Playground</h1>
        <p>Interactive API documentation and testing interface.</p>
        <div id="endpoints" class="endpoint-list"></div>
        <div class="try-it">
            <h2>Try It Out</h2>
            <input type="text" id="requestUrl" placeholder="Request URL" value="/api/v1/health">
            <select id="requestMethod" style="padding: 10px; margin-bottom: 10px;">
                <option value="GET">GET</option>
                <option value="POST">POST</option>
                <option value="PUT">PUT</option>
                <option value="DELETE">DELETE</option>
                <option value="PATCH">PATCH</option>
            </select>
            <textarea id="requestBody" rows="5" placeholder="Request body (JSON)"></textarea>
            <input type="text" id="authToken" placeholder="Authorization token (optional)">
            <button onclick="sendRequest()">Send Request</button>
            <h3>Response:</h3>
            <pre id="response">Click "Send Request" to see the response</pre>
        </div>
    </div>
    <script>
        fetch('/api/v1/openapi.json')
            .then(r => r.json())
            .then(spec => {
                const container = document.getElementById('endpoints');
                Object.entries(spec.paths || {}).forEach(([path, methods]) => {
                    Object.entries(methods).forEach(([method, op]) => {
                        if (['get','post','put','delete','patch'].includes(method)) {
                            const div = document.createElement('div');
                            div.className = 'endpoint';
                            div.innerHTML = `<span class="method method-${method.toUpperCase()}">${method.toUpperCase()}</span><span class="path">${path}</span><div class="summary">${op.summary || ''}</div>`;
                            div.onclick = () => {
                                document.getElementById('requestUrl').value = path;
                                document.getElementById('requestMethod').value = method.toUpperCase();
                            };
                            container.appendChild(div);
                        }
                    });
                });
            });
        function sendRequest() {
            const url = document.getElementById('requestUrl').value;
            const method = document.getElementById('requestMethod').value;
            const body = document.getElementById('requestBody').value;
            const token = document.getElementById('authToken').value;
            const headers = { 'Content-Type': 'application/json' };
            if (token) headers['Authorization'] = `Bearer ${token}`;
            const opts = { method, headers };
            if (body && method !== 'GET') opts.body = body;
            fetch(url, opts)
                .then(r => r.text())
                .then(text => { document.getElementById('response').textContent = text; })
                .catch(err => { document.getElementById('response').textContent = 'Error: ' + err.message; });
        }
    </script>
</body>
</html>"#;
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/html")],
        html,
    )
        .into_response()
}

pub fn api_documentation_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/documentation", get(list_api_documentation).post(create_api_documentation))
        .route("/api/v1/documentation/{endpoint}/{method}", get(get_api_documentation_for_endpoint))
        .route("/api/v1/playground", get(get_api_playground))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_documentation_response_serialization() {
        let response = ApiDocumentationResponse {
            id: Uuid::nil(),
            endpoint: "/api/v1/repos".into(),
            method: "GET".into(),
            summary: "List repositories".into(),
            description: "Returns a list of repositories".into(),
            parameters: serde_json::json!([]),
            request_body: None,
            responses: serde_json::json!({"200": {"description": "Success"}}),
            tags: vec!["repos".into()],
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("/api/v1/repos"));
        assert!(json.contains("\"method\":\"GET\""));
    }

    #[test]
    fn test_create_request_defaults() {
        let req = CreateApiDocumentationRequest {
            endpoint: "/test".into(),
            method: "GET".into(),
            summary: "Test".into(),
            description: String::new(),
            parameters: serde_json::json!([]),
            request_body: None,
            responses: serde_json::json!({}),
            tags: vec![],
        };
        assert_eq!(req.endpoint, "/test");
        assert!(req.tags.is_empty());
    }
}
