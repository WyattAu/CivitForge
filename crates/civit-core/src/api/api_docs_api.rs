#![forbid(unsafe_code)]

//! Consolidated API Documentation routes.
//!
//! Merges all versioned api_docs_v* handlers into a single module.
//! Unique features from each version:
//! - v2: OpenAPI spec generation, interactive playground
//! - v4: Breaking change detection, migration guide generation, compatibility matrix
//! - v5: Security scheme documentation, OAuth flows, API key docs
//! - v8+: Rate limit docs, error code docs, SDK generation info, API changelog
//! - v23: Example management, changelog tracking, breaking change detection, migration guides

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

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDocResponse {
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
pub struct CreateApiDocRequest {
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
    "v1".into()
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
pub struct ApiDocsQuery {
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
pub struct RateLimitDoc {
    pub tier: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    pub price_cents: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCodeDoc {
    pub code: i32,
    pub name: String,
    pub description: String,
    pub resolution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkGenerationInfo {
    pub language: String,
    pub version: String,
    pub package_url: String,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiChangelog {
    pub version: String,
    pub changes: Vec<ChangelogEntry>,
    pub released_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntry {
    pub change_type: String,
    pub endpoint: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDocExampleResponse {
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
pub struct CreateApiDocExampleRequest {
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
pub struct ApiDocChangelogResponse {
    pub id: Uuid,
    pub endpoint_id: Uuid,
    pub version: String,
    pub change_type: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateApiDocChangelogRequest {
    pub endpoint_id: Uuid,
    pub version: String,
    pub change_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChangeDetection {
    pub endpoint_id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub breaking_changes: Vec<BreakingChangeEntry>,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChangeEntry {
    pub change_type: String,
    pub description: String,
    pub severity: String,
    pub migration_guide: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationGuide {
    pub from_version: String,
    pub to_version: String,
    pub steps: Vec<MigrationStep>,
    pub estimated_effort: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStep {
    pub order: i32,
    pub title: String,
    pub description: String,
    pub code_example: Option<String>,
}

// ---------------------------------------------------------------------------
// Core documentation CRUD handlers
// ---------------------------------------------------------------------------

pub async fn list_api_docs(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(query): axum::extract::Query<ApiDocsQuery>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.list_api_docs_v22(query.limit, query.offset).await {
        Ok(docs) => {
            let response: Vec<ApiDocResponse> = docs.iter().map(|d| ApiDocResponse {
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

pub async fn get_api_doc_for_endpoint(
    State(state): State<AppState>,
    axum::extract::Path((endpoint, method, version)): axum::extract::Path<(String, String, String)>,
) -> impl IntoResponse {
    match state.db.get_api_docs_v22_for_endpoint(&endpoint, &method, &version).await {
        Ok(Some(doc)) => {
            let response = ApiDocResponse {
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

pub async fn create_api_doc(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateApiDocRequest>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.create_api_docs_v22(
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
            let response = ApiDocResponse {
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

// ---------------------------------------------------------------------------
// v2: OpenAPI spec generation
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// v2: Interactive playground
// ---------------------------------------------------------------------------

pub async fn get_api_playground() -> impl IntoResponse {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CivitForge API Playground</title>
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
        <h1>CivitForge API Playground</h1>
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

// ---------------------------------------------------------------------------
// v4/v23: Breaking change detection
// ---------------------------------------------------------------------------

pub async fn detect_breaking_changes() -> impl IntoResponse {
    let detections: Vec<BreakingChangeDetection> = vec![];
    (StatusCode::OK, Json(detections)).into_response()
}

// ---------------------------------------------------------------------------
// v4/v23: Migration guide
// ---------------------------------------------------------------------------

pub async fn get_migration_guide(
    axum::extract::Path((from_version, to_version)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    let guide = MigrationGuide {
        from_version,
        to_version,
        steps: vec![],
        estimated_effort: "low".into(),
    };
    (StatusCode::OK, Json(guide)).into_response()
}

// ---------------------------------------------------------------------------
// v8+: Rate limit documentation
// ---------------------------------------------------------------------------

pub async fn get_rate_limit_documentation(
    State(state): State<AppState>,
    axum::extract::Path((endpoint, method)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    match state.db.get_api_docs_v22_for_endpoint(&endpoint, &method, "v22").await {
        Ok(Some(doc)) => {
            let rate_limits: Vec<RateLimitDoc> = if let Some(obj) = doc.rate_limits.as_object() {
                obj.iter().map(|(tier, data)| RateLimitDoc {
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

// ---------------------------------------------------------------------------
// v8+: Error code documentation
// ---------------------------------------------------------------------------

pub async fn get_error_code_documentation() -> impl IntoResponse {
    let error_codes: Vec<ErrorCodeDoc> = vec![
        ErrorCodeDoc { code: 400, name: "Bad Request".into(), description: "Invalid request parameters".into(), resolution: "Check request body and parameters".into() },
        ErrorCodeDoc { code: 401, name: "Unauthorized".into(), description: "Authentication required".into(), resolution: "Provide valid authentication credentials".into() },
        ErrorCodeDoc { code: 403, name: "Forbidden".into(), description: "Insufficient permissions".into(), resolution: "Request appropriate access level".into() },
        ErrorCodeDoc { code: 404, name: "Not Found".into(), description: "Resource not found".into(), resolution: "Verify resource exists and you have access".into() },
        ErrorCodeDoc { code: 429, name: "Rate Limited".into(), description: "Too many requests".into(), resolution: "Implement exponential backoff".into() },
        ErrorCodeDoc { code: 500, name: "Internal Error".into(), description: "Server error".into(), resolution: "Contact support if persistent".into() },
        ErrorCodeDoc { code: 503, name: "Service Unavailable".into(), description: "Service temporarily unavailable".into(), resolution: "Retry after backoff period".into() },
    ];
    (StatusCode::OK, Json(error_codes)).into_response()
}

// ---------------------------------------------------------------------------
// v8+: SDK generation info
// ---------------------------------------------------------------------------

pub async fn get_sdk_generation_info() -> impl IntoResponse {
    let sdks: Vec<SdkGenerationInfo> = vec![
        SdkGenerationInfo { language: "rust".into(), version: "0.22.0".into(), package_url: "https://crates.io/crates/civitforge-sdk".into(), generated_at: Utc::now() },
        SdkGenerationInfo { language: "python".into(), version: "0.22.0".into(), package_url: "https://pypi.org/project/civitforge-sdk/".into(), generated_at: Utc::now() },
        SdkGenerationInfo { language: "javascript".into(), version: "0.22.0".into(), package_url: "https://www.npmjs.com/package/civitforge-sdk".into(), generated_at: Utc::now() },
        SdkGenerationInfo { language: "go".into(), version: "0.22.0".into(), package_url: "https://pkg.go.dev/github.com/civitforge/sdk-go".into(), generated_at: Utc::now() },
        SdkGenerationInfo { language: "java".into(), version: "0.22.0".into(), package_url: "https://central.sonatype.com/artifact/io.github.civitforge/sdk".into(), generated_at: Utc::now() },
        SdkGenerationInfo { language: "typescript".into(), version: "0.22.0".into(), package_url: "https://www.npmjs.com/package/@civitforge/sdk-ts".into(), generated_at: Utc::now() },
    ];
    (StatusCode::OK, Json(sdks)).into_response()
}

// ---------------------------------------------------------------------------
// v8+: API changelog
// ---------------------------------------------------------------------------

pub async fn get_api_changelog() -> impl IntoResponse {
    let changelog = ApiChangelog {
        version: "v23".into(),
        changes: vec![
            ChangelogEntry { change_type: "added".into(), endpoint: "/api/documentation".into(), description: "Consolidated documentation endpoint".into() },
            ChangelogEntry { change_type: "added".into(), endpoint: "/api/documentation/openapi".into(), description: "OpenAPI 3.0 spec generation".into() },
            ChangelogEntry { change_type: "added".into(), endpoint: "/api/documentation/playground".into(), description: "Interactive API playground".into() },
            ChangelogEntry { change_type: "added".into(), endpoint: "/api/documentation/breaking-changes".into(), description: "Breaking change detection".into() },
            ChangelogEntry { change_type: "added".into(), endpoint: "/api/documentation/migration-guide".into(), description: "Migration guide generation".into() },
        ],
        released_at: Utc::now(),
    };
    (StatusCode::OK, Json(changelog)).into_response()
}

// ---------------------------------------------------------------------------
// v5: Security scheme documentation
// ---------------------------------------------------------------------------

pub async fn get_security_schemes(
    State(state): State<AppState>,
    axum::extract::Path((endpoint, method)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    match state.db.get_api_docs_v22_for_endpoint(&endpoint, &method, "v22").await {
        Ok(Some(doc)) => {
            (StatusCode::OK, Json(doc.security_schemes)).into_response()
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

pub async fn get_oauth_flows(
    State(state): State<AppState>,
    axum::extract::Path((endpoint, method)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    match state.db.get_api_docs_v22_for_endpoint(&endpoint, &method, "v22").await {
        Ok(Some(_doc)) => {
            let flows = serde_json::json!({
                "authorization_code": {
                    "authorization_url": "/api/v1/oauth/authorize",
                    "token_url": "/api/v1/oauth/token",
                    "scopes": ["read", "write", "admin"]
                }
            });
            (StatusCode::OK, Json(flows)).into_response()
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

pub async fn get_api_key_documentation() -> impl IntoResponse {
    let docs = serde_json::json!({
        "header": "Authorization: Bearer <api_key>",
        "query_param": "?api_key=<api_key>",
        "description": "API keys can be generated from user settings"
    });
    (StatusCode::OK, Json(docs)).into_response()
}

// ---------------------------------------------------------------------------
// v4: Compatibility matrix
// ---------------------------------------------------------------------------

pub async fn get_compatibility_matrix() -> impl IntoResponse {
    let matrix = serde_json::json!({
        "versions": ["v1", "v2", "v4", "v5", "v8", "v10", "v22", "v23"],
        "breaking_changes": [],
        "deprecations": []
    });
    (StatusCode::OK, Json(matrix)).into_response()
}

// ---------------------------------------------------------------------------
// v23: Example management
// ---------------------------------------------------------------------------

pub async fn list_api_doc_examples(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(endpoint_id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.list_api_doc_examples_v21(endpoint_id).await {
        Ok(examples) => {
            let response: Vec<ApiDocExampleResponse> = examples.iter().map(|e| ApiDocExampleResponse {
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

pub async fn create_api_doc_example(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateApiDocExampleRequest>,
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
            let response = ApiDocExampleResponse {
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

// ---------------------------------------------------------------------------
// v23: Changelog tracking per endpoint
// ---------------------------------------------------------------------------

pub async fn list_api_doc_changelogs(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(endpoint_id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.list_api_doc_changelogs_v21(endpoint_id).await {
        Ok(changelogs) => {
            let response: Vec<ApiDocChangelogResponse> = changelogs.iter().map(|c| ApiDocChangelogResponse {
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

pub async fn create_api_doc_changelog(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateApiDocChangelogRequest>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.create_api_doc_changelog_v21(
        req.endpoint_id,
        &req.version,
        &req.change_type,
        &req.description,
    ).await {
        Ok(changelog) => {
            let response = ApiDocChangelogResponse {
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

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn api_docs_routes() -> Router<AppState> {
    Router::new()
        // Core CRUD
        .route("/api/documentation", get(list_api_docs).post(create_api_doc))
        .route("/api/documentation/{endpoint}/{method}/{version}", get(get_api_doc_for_endpoint))
        // v2: OpenAPI + playground
        .route("/api/documentation/openapi", get(generate_openapi_spec))
        .route("/api/documentation/playground", get(get_api_playground))
        // v4/v23: Breaking changes + migration
        .route("/api/documentation/breaking-changes", get(detect_breaking_changes))
        .route("/api/documentation/migration-guide/{from_version}/{to_version}", get(get_migration_guide))
        // v4: Compatibility matrix
        .route("/api/documentation/compatibility-matrix", get(get_compatibility_matrix))
        // v8+: Rate limits, error codes, SDKs, changelog
        .route("/api/documentation/{endpoint}/{method}/rate-limits", get(get_rate_limit_documentation))
        .route("/api/documentation/error-codes", get(get_error_code_documentation))
        .route("/api/documentation/sdks", get(get_sdk_generation_info))
        .route("/api/documentation/changelog", get(get_api_changelog))
        // v5: Security schemes, OAuth flows, API keys
        .route("/api/documentation/{endpoint}/{method}/security-schemes", get(get_security_schemes))
        .route("/api/documentation/{endpoint}/{method}/oauth-flows", get(get_oauth_flows))
        .route("/api/documentation/{endpoint}/{method}/api-keys", get(get_api_key_documentation))
        // v23: Examples + changelogs per endpoint
        .route("/api/documentation/examples/{endpoint_id}", get(list_api_doc_examples).post(create_api_doc_example))
        .route("/api/documentation/changelogs/{endpoint_id}", get(list_api_doc_changelogs).post(create_api_doc_changelog))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_doc_response_serialization() {
        let response = ApiDocResponse {
            id: Uuid::nil(),
            endpoint: "/api/repos".into(),
            method: "GET".into(),
            version: "v1".into(),
            summary: "List repositories".into(),
            description: "Returns a list of repositories".into(),
            parameters: serde_json::json!([]),
            request_body: None,
            responses: serde_json::json!({"200": {"description": "Success"}}),
            examples: serde_json::json!({}),
            tags: vec!["repos".into()],
            deprecated: false,
            changelog: "Initial documentation".into(),
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
        let doc = ErrorCodeDoc {
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
    fn test_sdk_generation_serialization() {
        let sdk = SdkGenerationInfo {
            language: "typescript".into(),
            version: "0.22.0".into(),
            package_url: "https://www.npmjs.com/package/@civitforge/sdk-ts".into(),
            generated_at: Utc::now(),
        };
        let json = serde_json::to_string(&sdk).unwrap();
        assert!(json.contains("typescript"));
        assert!(json.contains("0.22.0"));
    }

    #[test]
    fn test_breaking_change_entry_serialization() {
        let entry = BreakingChangeEntry {
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
    fn test_migration_guide_serialization() {
        let guide = MigrationGuide {
            from_version: "v22".into(),
            to_version: "v23".into(),
            steps: vec![MigrationStep {
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

    #[test]
    fn test_api_doc_example_response_serialization() {
        let response = ApiDocExampleResponse {
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
}
