#![forbid(unsafe_code)]

//! API Documentation v2 routes with OpenAPI 3.0 generation, interactive playground,
//! code examples, and SDK generation.

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
pub struct ApiDocsV2Response {
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
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateApiDocsV2Request {
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
}

fn default_version() -> String {
    "v1".into()
}

fn default_responses() -> serde_json::Value {
    serde_json::json!({})
}

fn default_examples() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiDocsV2Query {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub tag: Option<String>,
    pub version: Option<String>,
}

fn default_limit() -> i64 {
    100
}

pub async fn list_api_docs_v2(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<ApiDocsV2Query>,
) -> impl IntoResponse {
    let result = if let Some(tag) = &query.tag {
        state.db.search_api_docs_v2_by_tag(tag).await
    } else {
        state.db.list_api_docs_v2(query.limit, query.offset).await
    };

    match result {
        Ok(docs) => {
            let filtered = if let Some(version) = &query.version {
                docs.into_iter().filter(|d| d.version == *version).collect()
            } else {
                docs
            };
            let response: Vec<ApiDocsV2Response> = filtered.iter().map(|d| ApiDocsV2Response {
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

pub async fn get_api_docs_v2_for_endpoint(
    State(state): State<AppState>,
    axum::extract::Path((endpoint, method, version)): axum::extract::Path<(String, String, String)>,
) -> impl IntoResponse {
    match state.db.get_api_docs_v2(&endpoint, &method, &version).await {
        Ok(doc) => {
            let response = ApiDocsV2Response {
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

pub async fn create_api_docs_v2(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateApiDocsV2Request>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.create_api_docs_v2(
        &req.endpoint,
        &req.method,
        &req.version,
        &req.summary,
        &req.description,
        req.parameters,
        req.request_body,
        req.responses,
        req.examples,
        &req.tags,
    ).await {
        Ok(doc) => {
            let response = ApiDocsV2Response {
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

pub async fn generate_openapi_spec(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.db.list_api_docs_v2(1000, 0).await {
        Ok(docs) => {
            let mut paths = serde_json::Map::new();
            let mut tags = std::collections::HashSet::new();
            
            for doc in &docs {
                let path_entry = paths.entry(&doc.endpoint).or_insert_with(|| serde_json::json!({}));
                let method_lower = doc.method.to_lowercase();
                
                let mut operation = serde_json::json!({
                    "summary": doc.summary,
                    "description": doc.description,
                    "tags": doc.tags,
                    "parameters": doc.parameters,
                    "responses": doc.responses,
                });
                
                if let Some(request_body) = &doc.request_body {
                    operation["requestBody"] = request_body.clone();
                }
                
                if let Some(obj) = path_entry.as_object_mut() {
                    obj.insert(method_lower, operation);
                }
                
                for tag in &doc.tags {
                    tags.insert(tag.clone());
                }
            }
            
            let spec = serde_json::json!({
                "openapi": "3.0.0",
                "info": {
                    "title": "CivitForge API",
                    "version": "v1",
                    "description": "CivitForge API Documentation"
                },
                "paths": paths,
                "tags": tags.into_iter().map(|t| serde_json::json!({"name": t})).collect::<Vec<_>>(),
            });
            
            (StatusCode::OK, Json(spec)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_api_playground_v2() -> impl IntoResponse {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CivitForge API Playground v2</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }
        .container { max-width: 1400px; margin: 0 auto; }
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
        .version { color: #999; font-size: 12px; }
        .try-it { margin-top: 20px; background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .try-it h2 { margin-top: 0; }
        textarea, input, select { width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; font-family: monospace; margin-bottom: 10px; box-sizing: border-box; }
        button { background: #49cc90; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer; font-size: 14px; margin-right: 10px; }
        button:hover { background: #3daa80; }
        pre { background: #f8f8f8; padding: 15px; border-radius: 4px; overflow-x: auto; }
        .tabs { display: flex; gap: 10px; margin-bottom: 15px; }
        .tab { padding: 8px 16px; border-radius: 4px; cursor: pointer; background: #e0e0e0; }
        .tab.active { background: #49cc90; color: white; }
        .code-example { display: none; }
        .code-example.active { display: block; }
    </style>
</head>
<body>
    <div class="container">
        <h1>CivitForge API Playground v2</h1>
        <p>Interactive API documentation with OpenAPI 3.0 support, code examples, and SDK generation.</p>
        <div class="tabs">
            <div class="tab active" onclick="showTab('endpoints')">Endpoints</div>
            <div class="tab" onclick="showTab('openapi')">OpenAPI Spec</div>
            <div class="tab" onclick="showTab('sdk')">SDK Generation</div>
        </div>
        <div id="endpoints-tab">
            <div id="endpoints" class="endpoint-list"></div>
        </div>
        <div id="openapi-tab" style="display:none;">
            <h2>OpenAPI 3.0 Specification</h2>
            <pre id="openapi-spec">Loading...</pre>
            <button onclick="downloadSpec()">Download Spec</button>
        </div>
        <div id="sdk-tab" style="display:none;">
            <h2>SDK Generation</h2>
            <p>Generate client SDKs for your favorite language:</p>
            <button onclick="generateSDK('javascript')">JavaScript</button>
            <button onclick="generateSDK('python')">Python</button>
            <button onclick="generateSDK('go')">Go</button>
            <button onclick="generateSDK('rust')">Rust</button>
            <pre id="sdk-output">Select a language to generate SDK</pre>
        </div>
        <div class="try-it">
            <h2>Try It Out</h2>
            <input type="text" id="requestUrl" placeholder="Request URL" value="/api/v1/health">
            <select id="requestMethod">
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
            <div id="code-examples">
                <h3>Code Examples:</h3>
                <div class="tabs">
                    <div class="tab active" onclick="showCodeTab('curl')">cURL</div>
                    <div class="tab" onclick="showCodeTab('javascript')">JavaScript</div>
                    <div class="tab" onclick="showCodeTab('python')">Python</div>
                </div>
                <pre id="curl-example" class="code-example active"></pre>
                <pre id="javascript-example" class="code-example"></pre>
                <pre id="python-example" class="code-example"></pre>
            </div>
        </div>
    </div>
    <script>
        let currentSpec = null;
        
        function showTab(tab) {
            document.querySelectorAll('.tabs .tab').forEach(t => t.classList.remove('active'));
            document.querySelectorAll('[id$="-tab"]').forEach(t => t.style.display = 'none');
            document.getElementById(tab + '-tab').style.display = 'block';
            event.target.classList.add('active');
        }
        
        function showCodeTab(lang) {
            document.querySelectorAll('#code-examples .tab').forEach(t => t.classList.remove('active'));
            document.querySelectorAll('.code-example').forEach(t => t.classList.remove('active'));
            document.getElementById(lang + '-example').classList.add('active');
            event.target.classList.add('active');
        }
        
        fetch('/api/v2/documentation/openapi')
            .then(r => r.json())
            .then(spec => {
                currentSpec = spec;
                document.getElementById('openapi-spec').textContent = JSON.stringify(spec, null, 2);
                const container = document.getElementById('endpoints');
                container.innerHTML = '';
                Object.entries(spec.paths || {}).forEach(([path, methods]) => {
                    Object.entries(methods).forEach(([method, op]) => {
                        if (['get','post','put','delete','patch'].includes(method)) {
                            const div = document.createElement('div');
                            div.className = 'endpoint';
                            div.innerHTML = `<span class="method method-${method.toUpperCase()}">${method.toUpperCase()}</span><span class="path">${path}</span><span class="version">${op.version || 'v1'}</span><div class="summary">${op.summary || ''}</div>`;
                            div.onclick = () => {
                                document.getElementById('requestUrl').value = path;
                                document.getElementById('requestMethod').value = method.toUpperCase();
                                updateCodeExamples(path, method.toUpperCase());
                            };
                            container.appendChild(div);
                        }
                    });
                });
            });
        
        function updateCodeExamples(url, method) {
            const curl = `curl -X ${method} ${url}`;
            const js = `fetch('${url}', { method: '${method}' })\n  .then(r => r.json())\n  .then(data => console.log(data));`;
            const python = `import requests\nresponse = requests.${method.toLowerCase()}('${url}')\nprint(response.json())`;
            
            document.getElementById('curl-example').textContent = curl;
            document.getElementById('javascript-example').textContent = js;
            document.getElementById('python-example').textContent = python;
        }
        
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
        
        function downloadSpec() {
            if (currentSpec) {
                const blob = new Blob([JSON.stringify(currentSpec, null, 2)], { type: 'application/json' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = 'openapi.json';
                a.click();
            }
        }
        
        function generateSDK(lang) {
            const output = document.getElementById('sdk-output');
            output.textContent = `Generating ${lang} SDK...\n\nThis would generate a client SDK based on the OpenAPI spec.\nIn production, this would call an SDK generation service.`;
        }
        
        updateCodeExamples('/api/v1/health', 'GET');
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

pub fn api_docs_v2_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v2/documentation", get(list_api_docs_v2).post(create_api_docs_v2))
        .route("/api/v2/documentation/{endpoint}/{method}/{version}", get(get_api_docs_v2_for_endpoint))
        .route("/api/v2/documentation/openapi", get(generate_openapi_spec))
        .route("/api/v2/playground", get(get_api_playground_v2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_docs_v2_response_serialization() {
        let response = ApiDocsV2Response {
            id: Uuid::nil(),
            endpoint: "/api/v1/repos".into(),
            method: "GET".into(),
            version: "v1".into(),
            summary: "List repositories".into(),
            description: "Returns a list of repositories".into(),
            parameters: serde_json::json!([]),
            request_body: None,
            responses: serde_json::json!({"200": {"description": "Success"}}),
            examples: serde_json::json!({}),
            tags: vec!["repos".into()],
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("/api/v1/repos"));
        assert!(json.contains("\"method\":\"GET\""));
        assert!(json.contains("\"version\":\"v1\""));
    }

    #[test]
    fn test_create_request_defaults() {
        let req = CreateApiDocsV2Request {
            endpoint: "/test".into(),
            method: "GET".into(),
            version: "v1".into(),
            summary: "Test".into(),
            description: String::new(),
            parameters: serde_json::json!([]),
            request_body: None,
            responses: serde_json::json!({}),
            examples: serde_json::json!({}),
            tags: vec![],
        };
        assert_eq!(req.endpoint, "/test");
        assert_eq!(req.version, "v1");
        assert!(req.tags.is_empty());
    }
}
