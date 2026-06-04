#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiSpec {
    pub openapi: String,
    pub info: OpenApiInfo,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<OpenApiServer>,
    pub paths: HashMap<String, PathItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub security: Vec<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiServer {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiInfo {
    pub title: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<License>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub name: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Parameter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Parameter>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub responses: HashMap<String, Response>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBody>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<HashMap<String, Vec<String>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub content: HashMap<String, MediaType>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<SchemaRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub location: ParameterLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<SchemaRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterLocation {
    Query,
    Header,
    Path,
    Cookie,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_ref: Option<SchemaRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaRef {
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub ref_: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, SchemaRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<SchemaRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Components {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub schemas: HashMap<String, SchemaRef>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub security_schemes: HashMap<String, SecurityScheme>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScheme {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_format: Option<String>,
}

pub struct OpenApiGenerator {
    spec: OpenApiSpec,
}

impl Default for OpenApiGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenApiGenerator {
    pub fn new() -> Self {
        Self {
            spec: OpenApiSpec {
                openapi: "3.1.0".to_string(),
                info: OpenApiInfo {
                    title: String::new(),
                    version: "0.1.0".to_string(),
                    description: None,
                    contact: None,
                    license: None,
                },
                servers: Vec::new(),
                paths: HashMap::new(),
                components: None,
                security: Vec::new(),
            },
        }
    }

    pub fn with_info(mut self, title: impl Into<String>, version: impl Into<String>) -> Self {
        self.spec.info.title = title.into();
        self.spec.info.version = version.into();
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.spec.info.description = Some(desc.into());
        self
    }

    pub fn with_contact(mut self, name: impl Into<String>, email: impl Into<String>) -> Self {
        self.spec.info.contact = Some(Contact {
            name: Some(name.into()),
            email: Some(email.into()),
            url: None,
        });
        self
    }

    pub fn with_license(mut self, name: impl Into<String>, url: impl Into<String>) -> Self {
        self.spec.info.license = Some(License {
            name: name.into(),
            url: Some(url.into()),
        });
        self
    }

    pub fn with_server(mut self, url: impl Into<String>, description: impl Into<String>) -> Self {
        self.spec.servers.push(OpenApiServer {
            url: url.into(),
            description: Some(description.into()),
        });
        self
    }

    pub fn register_path(mut self, path: impl Into<String>, item: PathItem) -> Self {
        self.spec.paths.insert(path.into(), item);
        self
    }

    pub fn add_schema(mut self, name: impl Into<String>, schema: SchemaRef) -> Self {
        let components = self.spec.components.get_or_insert_with(Components::default);
        components.schemas.insert(name.into(), schema);
        self
    }

    pub fn add_security_scheme(mut self, name: impl Into<String>, scheme: SecurityScheme) -> Self {
        let components = self.spec.components.get_or_insert_with(Components::default);
        components.security_schemes.insert(name.into(), scheme);
        self
    }

    pub fn add_global_security(mut self, requirement: HashMap<String, Vec<String>>) -> Self {
        self.spec.security.push(requirement);
        self
    }

    pub fn generate(&self) -> OpenApiSpec {
        self.spec.clone()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.spec).unwrap_or_default()
    }

    pub fn to_yaml(&self) -> String {
        let json = self.to_json();
        simple_json_to_yaml(&json)
    }
}

fn simple_json_to_yaml(json: &str) -> String {
    let val: serde_json::Value = serde_json::from_str(json).unwrap_or_default();
    let mut out = String::new();
    convert_value(&val, 0, &mut out);
    out
}

fn convert_value(val: &serde_json::Value, indent: usize, out: &mut String) {
    let prefix = "  ".repeat(indent);
    match val {
        serde_json::Value::Object(map) => {
            if indent > 0 {
                out.push('\n');
            }
            let entries: Vec<_> = map.iter().collect();
            for (i, (key, v)) in entries.iter().enumerate() {
                out.push_str(&prefix);
                out.push_str(key);
                out.push(':');
                match v {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        convert_value(v, indent + 1, out);
                    }
                    serde_json::Value::String(s) => {
                        out.push(' ');
                        out.push_str(s);
                        out.push('\n');
                    }
                    other => {
                        out.push(' ');
                        out.push_str(&other.to_string());
                        out.push('\n');
                    }
                }
                if i < entries.len() - 1
                    && matches!(
                        v,
                        serde_json::Value::String(_)
                            | serde_json::Value::Null
                            | serde_json::Value::Bool(_)
                            | serde_json::Value::Number(_)
                    )
                {}
            }
        }
        serde_json::Value::Array(arr) => {
            out.push('\n');
            for item in arr {
                out.push_str(&prefix);
                out.push_str("- ");
                match item {
                    serde_json::Value::String(s) => {
                        out.push_str(s);
                        out.push('\n');
                    }
                    serde_json::Value::Object(_) => {
                        out.truncate(out.len().saturating_sub(2));
                        out.push_str(":\n");
                        convert_value(item, indent + 2, out);
                    }
                    other => {
                        out.push_str(&other.to_string());
                        out.push('\n');
                    }
                }
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Master spec generator — all known API endpoints
// ---------------------------------------------------------------------------

use std::collections::HashMap as Map;

fn p(name: &str, loc: ParameterLocation, required: bool, type_: &str) -> Parameter {
    Parameter {
        name: name.into(),
        location: loc,
        description: None,
        required,
        schema: Some(SchemaRef {
            ref_: None,
            type_: Some(type_.into()),
            properties: Map::new(),
            required: Vec::new(),
            items: None,
            description: None,
            example: None,
        }),
        example: None,
    }
}

fn op(id: &str, summary: &str, tag: &str) -> Operation {
    Operation {
        operation_id: Some(id.into()),
        summary: Some(summary.into()),
        description: None,
        parameters: Vec::new(),
        responses: Map::new(),
        tags: vec![tag.into()],
        request_body: None,
        security: None,
    }
}

fn resp(desc: &str, schema: &str) -> Response {
    Response {
        description: desc.into(),
        status_code: None,
        content_type: None,
        schema_ref: if schema.is_empty() {
            None
        } else {
            Some(SchemaRef {
                ref_: Some(schema.into()),
                type_: None,
                properties: Map::new(),
                required: Vec::new(),
                items: None,
                description: None,
                example: None,
            })
        },
    }
}

fn resp_err(code: &str, desc: &str) -> (String, Response) {
    (code.into(), resp(desc, ""))
}

fn schema_obj(props: Vec<(&str, &str)>) -> SchemaRef {
    SchemaRef {
        ref_: None,
        type_: Some("object".into()),
        properties: props
            .into_iter()
            .map(|(k, v)| {
                (
                    k.into(),
                    SchemaRef {
                        ref_: None,
                        type_: Some(v.into()),
                        properties: Map::new(),
                        required: Vec::new(),
                        items: None,
                        description: None,
                        example: None,
                    },
                )
            })
            .collect(),
        required: Vec::new(),
        items: None,
        description: None,
        example: None,
    }
}

/// Generate the full OpenAPI 3.1 specification for all CivitForge API endpoints.
pub fn generate_openapi_spec() -> OpenApiGenerator {
    let mut security = Map::new();
    security.insert("bearer".into(), Vec::new());

    let mut g = OpenApiGenerator::new()
        .with_info("CivitForge API", env!("CARGO_PKG_VERSION"))
        .with_description("Self-hosted federated forge platform API")
        .with_license(
            "AGPL-3.0-or-later",
            "https://www.gnu.org/licenses/agpl-3.0.en.html",
        )
        .with_server("/api/v1", "API v1")
        .add_security_scheme(
            "bearer",
            SecurityScheme {
                type_: "http".into(),
                name: None,
                in_: None,
                description: Some("JWT Bearer token authentication".into()),
                bearer_format: Some("JWT".into()),
            },
        )
        .add_global_security(security)
        // ── Health ──────────────────────────────────────────────
        .register_path(
            "/healthz",
            PathItem {
                get: Some({
                    let mut o = op("healthz", "Health check", "system");
                    o.responses = [("200".into(), resp("OK", ""))].into();
                    o.security = Some(vec![Map::new()]);
                    o
                }),
                post: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/health",
            PathItem {
                get: Some({
                    let mut o = op("apiHealth", "API health check", "system");
                    o.responses = [("200".into(), resp("OK", ""))].into();
                    o.security = Some(vec![Map::new()]);
                    o
                }),
                post: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        // ── Auth ────────────────────────────────────────────────
        .register_path(
            "/api/v1/auth/login",
            PathItem {
                post: Some({
                    let mut o = op("login", "Authenticate user", "auth");
                    o.parameters = vec![];
                    o.request_body = Some(RequestBody {
                        description: Some("Login credentials".into()),
                        content: [(
                            "application/json".into(),
                            MediaType {
                                schema: Some(schema_obj(vec![
                                    ("username", "string"),
                                    ("email", "string"),
                                    ("display_name", "string"),
                                ])),
                            },
                        )]
                        .into(),
                        required: true,
                    });
                    o.responses = [
                        (
                            "200".into(),
                            resp(
                                "JWT token and user info",
                                "#/components/schemas/LoginResponse",
                            ),
                        ),
                        resp_err("400", "Invalid request"),
                        resp_err("401", "Invalid credentials"),
                    ]
                    .into();
                    o.security = Some(vec![Map::new()]);
                    o
                }),
                get: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/auth/me",
            PathItem {
                get: Some({
                    let mut o = op("me", "Get current user", "auth");
                    o.responses = [
                        (
                            "200".into(),
                            resp("Current user info", "#/components/schemas/User"),
                        ),
                        resp_err("401", "Not authenticated"),
                    ]
                    .into();
                    o
                }),
                post: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/auth/refresh",
            PathItem {
                post: Some({
                    let mut o = op("refresh", "Refresh JWT token", "auth");
                    o.responses = [
                        (
                            "200".into(),
                            resp("New JWT token", "#/components/schemas/RefreshResponse"),
                        ),
                        resp_err("401", "Invalid token"),
                    ]
                    .into();
                    o
                }),
                get: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        // ── Repos ───────────────────────────────────────────────
        .register_path(
            "/api/v1/repos",
            PathItem {
                get: Some({
                    let mut o = op("listRepos", "List repositories", "repos");
                    o.parameters = vec![
                        p("limit", ParameterLocation::Query, false, "integer"),
                        p("offset", ParameterLocation::Query, false, "integer"),
                    ];
                    o.responses = [
                        (
                            "200".into(),
                            resp("List of repos", "#/components/schemas/RepoList"),
                        ),
                        resp_err("500", "Internal error"),
                    ]
                    .into();
                    o.security = Some(vec![Map::new()]);
                    o
                }),
                post: Some({
                    let mut o = op("createRepo", "Create a repository", "repos");
                    o.request_body = Some(RequestBody {
                        description: Some("Repository to create".into()),
                        content: [(
                            "application/json".into(),
                            MediaType {
                                schema: Some(schema_obj(vec![
                                    ("name", "string"),
                                    ("owner", "string"),
                                    ("description", "string"),
                                    ("visibility", "string"),
                                ])),
                            },
                        )]
                        .into(),
                        required: true,
                    });
                    o.responses = [
                        (
                            "201".into(),
                            resp("Created repo", "#/components/schemas/Repo"),
                        ),
                        resp_err("400", "Invalid request"),
                        resp_err("401", "Not authenticated"),
                        resp_err("403", "Permission denied"),
                        resp_err("500", "Internal error"),
                    ]
                    .into();
                    o
                }),
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/repos/{owner}/{name}",
            PathItem {
                get: Some({
                    let mut o = op("getRepo", "Get repository details", "repos");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("name", ParameterLocation::Path, true, "string"),
                    ];
                    o.responses = [
                        (
                            "200".into(),
                            resp("Repository", "#/components/schemas/Repo"),
                        ),
                        resp_err("404", "Not found"),
                        resp_err("500", "Internal error"),
                    ]
                    .into();
                    o.security = Some(vec![Map::new()]);
                    o
                }),
                delete: Some({
                    let mut o = op("deleteRepo", "Delete a repository", "repos");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("name", ParameterLocation::Path, true, "string"),
                    ];
                    o.responses = [
                        ("204".into(), resp("Deleted", "")),
                        resp_err("401", "Not authenticated"),
                        resp_err("403", "Permission denied"),
                        resp_err("404", "Not found"),
                        resp_err("500", "Internal error"),
                    ]
                    .into();
                    o
                }),
                put: None,
                post: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/repos/{owner}/{name}/commits",
            PathItem {
                get: Some({
                    let mut o = op("listCommits", "List commits", "repos");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("name", ParameterLocation::Path, true, "string"),
                    ];
                    o.responses = [
                        (
                            "200".into(),
                            resp("Commit list", "#/components/schemas/CommitList"),
                        ),
                        resp_err("404", "Not found"),
                        resp_err("500", "Internal error"),
                    ]
                    .into();
                    o.security = Some(vec![Map::new()]);
                    o
                }),
                post: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        // ── Code Browser ─────────────────────────────────────────
        .register_path(
            "/api/v1/repos/{owner}/{name}/tree",
            PathItem {
                get: Some({
                    let mut o = op("listTree", "List directory tree", "repos");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("name", ParameterLocation::Path, true, "string"),
                        p("path", ParameterLocation::Query, false, "string"),
                        p("ref", ParameterLocation::Query, false, "string"),
                    ];
                    o.responses = [
                        (
                            "200".into(),
                            resp("Tree entries", "#/components/schemas/TreeEntryList"),
                        ),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o.security = Some(vec![Map::new()]);
                    o
                }),
                post: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/repos/{owner}/{name}/blob",
            PathItem {
                get: Some({
                    let mut o = op("readBlob", "Read file contents", "repos");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("name", ParameterLocation::Path, true, "string"),
                        p("path", ParameterLocation::Query, false, "string"),
                        p("ref", ParameterLocation::Query, false, "string"),
                    ];
                    o.responses = [
                        (
                            "200".into(),
                            resp("Blob contents", "#/components/schemas/Blob"),
                        ),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o.security = Some(vec![Map::new()]);
                    o
                }),
                post: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        // ── Issues ──────────────────────────────────────────────
        .register_path(
            "/api/v1/repos/{owner}/{name}/issues",
            PathItem {
                get: Some({
                    let mut o = op("listIssues", "List issues", "issues");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("name", ParameterLocation::Path, true, "string"),
                        p("state", ParameterLocation::Query, false, "string"),
                        p("label", ParameterLocation::Query, false, "string"),
                        p("assignee", ParameterLocation::Query, false, "string"),
                        p("sort", ParameterLocation::Query, false, "string"),
                        p("page", ParameterLocation::Query, false, "integer"),
                        p("per_page", ParameterLocation::Query, false, "integer"),
                    ];
                    o.responses = [
                        (
                            "200".into(),
                            resp("Paginated issues", "#/components/schemas/IssueList"),
                        ),
                        resp_err("404", "Repo not found"),
                    ]
                    .into();
                    o
                }),
                post: Some({
                    let mut o = op("createIssue", "Create an issue", "issues");
                    o.request_body = Some(RequestBody {
                        description: Some("Issue to create".into()),
                        content: [(
                            "application/json".into(),
                            MediaType {
                                schema: Some(schema_obj(vec![
                                    ("title", "string"),
                                    ("description", "string"),
                                    ("assignee", "string"),
                                    ("milestone", "string"),
                                ])),
                            },
                        )]
                        .into(),
                        required: true,
                    });
                    o.responses = [
                        (
                            "201".into(),
                            resp("Created issue", "#/components/schemas/Issue"),
                        ),
                        resp_err("400", "Invalid request"),
                        resp_err("401", "Not authenticated"),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/repos/{owner}/{name}/issues/{number}",
            PathItem {
                get: Some({
                    let mut o = op("getIssue", "Get issue details", "issues");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("name", ParameterLocation::Path, true, "string"),
                        p("number", ParameterLocation::Path, true, "integer"),
                    ];
                    o.responses = [
                        (
                            "200".into(),
                            resp("Issue details", "#/components/schemas/Issue"),
                        ),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o.security = Some(vec![Map::new()]);
                    o
                }),
                patch: Some({
                    let mut o = op("updateIssue", "Update an issue", "issues");
                    o.request_body = Some(RequestBody {
                        description: Some("Fields to update".into()),
                        content: [(
                            "application/json".into(),
                            MediaType {
                                schema: Some(schema_obj(vec![
                                    ("title", "string"),
                                    ("description", "string"),
                                    ("state", "string"),
                                    ("assignee", "string"),
                                ])),
                            },
                        )]
                        .into(),
                        required: true,
                    });
                    o.responses = [
                        (
                            "200".into(),
                            resp("Updated issue", "#/components/schemas/Issue"),
                        ),
                        resp_err("400", "Invalid request"),
                        resp_err("404", "Not found"),
                        resp_err("409", "Invalid state transition"),
                    ]
                    .into();
                    o
                }),
                delete: Some({
                    let mut o = op("deleteIssue", "Delete an issue", "issues");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("name", ParameterLocation::Path, true, "string"),
                        p("number", ParameterLocation::Path, true, "integer"),
                    ];
                    o.responses = [
                        ("204".into(), resp("Deleted", "")),
                        resp_err("401", "Not authenticated"),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                post: None,
                put: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/repos/{owner}/{name}/issues/{number}/comments",
            PathItem {
                post: Some({
                    let mut o = op("addComment", "Add a comment", "issues");
                    o.request_body = Some(RequestBody {
                        description: Some("Comment body".into()),
                        content: [(
                            "application/json".into(),
                            MediaType {
                                schema: Some(schema_obj(vec![("body", "string")])),
                            },
                        )]
                        .into(),
                        required: true,
                    });
                    o.responses = [
                        (
                            "201".into(),
                            resp("Created comment", "#/components/schemas/Comment"),
                        ),
                        resp_err("400", "Invalid request"),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                get: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/repos/{owner}/{name}/issues/{number}/reactions",
            PathItem {
                post: Some({
                    let mut o = op("addReaction", "Add a reaction", "issues");
                    o.request_body = Some(RequestBody {
                        description: Some("Reaction to add".into()),
                        content: [(
                            "application/json".into(),
                            MediaType {
                                schema: Some(schema_obj(vec![("emoji", "string")])),
                            },
                        )]
                        .into(),
                        required: true,
                    });
                    o.responses = [
                        (
                            "201".into(),
                            resp("Created reaction", "#/components/schemas/Reaction"),
                        ),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                get: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/repos/{owner}/{name}/labels",
            PathItem {
                get: Some({
                    let mut o = op("listLabels", "List labels", "issues");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("name", ParameterLocation::Path, true, "string"),
                    ];
                    o.responses = [
                        (
                            "200".into(),
                            resp("Labels", "#/components/schemas/LabelList"),
                        ),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                post: Some({
                    let mut o = op("createLabel", "Create a label", "issues");
                    o.request_body = Some(RequestBody {
                        description: Some("Label to create".into()),
                        content: [(
                            "application/json".into(),
                            MediaType {
                                schema: Some(schema_obj(vec![
                                    ("name", "string"),
                                    ("color", "string"),
                                    ("description", "string"),
                                ])),
                            },
                        )]
                        .into(),
                        required: true,
                    });
                    o.responses = [
                        (
                            "201".into(),
                            resp("Created label", "#/components/schemas/Label"),
                        ),
                        resp_err("400", "Invalid request"),
                    ]
                    .into();
                    o
                }),
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/repos/{owner}/{name}/milestones",
            PathItem {
                get: Some({
                    let mut o = op("listMilestones", "List milestones", "issues");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("name", ParameterLocation::Path, true, "string"),
                        p("state", ParameterLocation::Query, false, "string"),
                    ];
                    o.responses = [
                        (
                            "200".into(),
                            resp("Milestones", "#/components/schemas/MilestoneList"),
                        ),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                post: Some({
                    let mut o = op("createMilestone", "Create a milestone", "issues");
                    o.request_body = Some(RequestBody {
                        description: Some("Milestone to create".into()),
                        content: [(
                            "application/json".into(),
                            MediaType {
                                schema: Some(schema_obj(vec![
                                    ("title", "string"),
                                    ("description", "string"),
                                    ("due_on", "string"),
                                ])),
                            },
                        )]
                        .into(),
                        required: true,
                    });
                    o.responses = [
                        (
                            "201".into(),
                            resp("Created milestone", "#/components/schemas/Milestone"),
                        ),
                        resp_err("400", "Invalid request"),
                    ]
                    .into();
                    o
                }),
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        // ── Wiki ─────────────────────────────────────────────────
        .register_path(
            "/api/v1/repos/{owner}/{name}/wiki",
            PathItem {
                get: Some({
                    let mut o = op("listWikiPages", "List wiki pages", "wiki");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("name", ParameterLocation::Path, true, "string"),
                    ];
                    o.responses = [
                        (
                            "200".into(),
                            resp("Wiki page list", "#/components/schemas/WikiPageSummaryList"),
                        ),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                post: Some({
                    let mut o = op("createWikiPage", "Create a wiki page", "wiki");
                    o.request_body = Some(RequestBody {
                        description: Some("Wiki page to create".into()),
                        content: [(
                            "application/json".into(),
                            MediaType {
                                schema: Some(schema_obj(vec![
                                    ("slug", "string"),
                                    ("title", "string"),
                                    ("content", "string"),
                                ])),
                            },
                        )]
                        .into(),
                        required: true,
                    });
                    o.responses = [
                        (
                            "201".into(),
                            resp("Created page", "#/components/schemas/WikiPage"),
                        ),
                        resp_err("400", "Invalid request"),
                        resp_err("401", "Not authenticated"),
                    ]
                    .into();
                    o
                }),
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/repos/{owner}/{name}/wiki/search",
            PathItem {
                get: Some({
                    let mut o = op("searchWiki", "Search wiki pages", "wiki");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("name", ParameterLocation::Path, true, "string"),
                        p("q", ParameterLocation::Query, true, "string"),
                    ];
                    o.responses = [
                        (
                            "200".into(),
                            resp("Search results", "#/components/schemas/WikiPageSummaryList"),
                        ),
                        resp_err("400", "Missing query"),
                    ]
                    .into();
                    o
                }),
                post: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/repos/{owner}/{name}/wiki/{slug}",
            PathItem {
                get: Some({
                    let mut o = op("getWikiPage", "Get wiki page", "wiki");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("name", ParameterLocation::Path, true, "string"),
                        p("slug", ParameterLocation::Path, true, "string"),
                    ];
                    o.responses = [
                        (
                            "200".into(),
                            resp("Wiki page", "#/components/schemas/WikiPage"),
                        ),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                put: Some({
                    let mut o = op("updateWikiPage", "Update a wiki page", "wiki");
                    o.request_body = Some(RequestBody {
                        description: Some("Fields to update".into()),
                        content: [(
                            "application/json".into(),
                            MediaType {
                                schema: Some(schema_obj(vec![
                                    ("title", "string"),
                                    ("content", "string"),
                                    ("edit_message", "string"),
                                ])),
                            },
                        )]
                        .into(),
                        required: true,
                    });
                    o.responses = [
                        (
                            "200".into(),
                            resp("Updated page", "#/components/schemas/WikiPage"),
                        ),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                delete: Some({
                    let mut o = op("deleteWikiPage", "Delete a wiki page", "wiki");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("name", ParameterLocation::Path, true, "string"),
                        p("slug", ParameterLocation::Path, true, "string"),
                    ];
                    o.responses = [
                        ("204".into(), resp("Deleted", "")),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                post: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/repos/{owner}/{name}/wiki/{slug}/history",
            PathItem {
                get: Some({
                    let mut o = op("wikiPageHistory", "Get wiki page history", "wiki");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("name", ParameterLocation::Path, true, "string"),
                        p("slug", ParameterLocation::Path, true, "string"),
                    ];
                    o.responses = [
                        (
                            "200".into(),
                            resp("Revision list", "#/components/schemas/WikiRevisionList"),
                        ),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                post: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/repos/{owner}/{name}/wiki/{slug}/diff",
            PathItem {
                get: Some({
                    let mut o = op("wikiPageDiff", "Diff between two wiki revisions", "wiki");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("name", ParameterLocation::Path, true, "string"),
                        p("slug", ParameterLocation::Path, true, "string"),
                        p("sha1", ParameterLocation::Query, true, "string"),
                        p("sha2", ParameterLocation::Query, true, "string"),
                    ];
                    o.responses = [
                        ("200".into(), resp("Diff", "#/components/schemas/Diff")),
                        resp_err("400", "Missing SHA"),
                    ]
                    .into();
                    o
                }),
                post: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/repos/{owner}/{name}/wiki/{slug}/raw",
            PathItem {
                get: Some({
                    let mut o = op("wikiPageRaw", "Get raw wiki page content", "wiki");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("name", ParameterLocation::Path, true, "string"),
                        p("slug", ParameterLocation::Path, true, "string"),
                    ];
                    o.responses = [
                        ("200".into(), resp("Raw markdown", "")),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                post: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        // ── Pipelines ──────────────────────────────────────────
        .register_path(
            "/api/v1/repos/{owner}/{repo}/pipelines",
            PathItem {
                get: Some({
                    let mut o = op("listPipelines", "List pipelines for a repo", "pipelines");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("repo", ParameterLocation::Path, true, "string"),
                        p("limit", ParameterLocation::Query, false, "integer"),
                        p("offset", ParameterLocation::Query, false, "integer"),
                        p("status", ParameterLocation::Query, false, "string"),
                    ];
                    o.responses = [
                        (
                            "200".into(),
                            resp("Pipeline runs", "#/components/schemas/PipelineRunList"),
                        ),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o.security = Some(vec![Map::new()]);
                    o
                }),
                post: Some({
                    let mut o = op("triggerPipeline", "Trigger a pipeline run", "pipelines");
                    o.request_body = Some(RequestBody {
                        description: Some("Pipeline trigger parameters".into()),
                        content: [(
                            "application/json".into(),
                            MediaType {
                                schema: Some(schema_obj(vec![
                                    ("ref_name", "string"),
                                    ("commit_sha", "string"),
                                    ("yaml_path", "string"),
                                    ("event_type", "string"),
                                ])),
                            },
                        )]
                        .into(),
                        required: true,
                    });
                    o.responses = [
                        (
                            "201".into(),
                            resp("Created pipeline run", "#/components/schemas/PipelineRun"),
                        ),
                        resp_err("401", "Not authenticated"),
                        resp_err("404", "Not found"),
                        resp_err("422", "Invalid pipeline YAML"),
                    ]
                    .into();
                    o
                }),
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/repos/{owner}/{repo}/pipelines/{pipeline_id}",
            PathItem {
                get: Some({
                    let mut o = op("getPipeline", "Get pipeline run details", "pipelines");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("repo", ParameterLocation::Path, true, "string"),
                        p("pipeline_id", ParameterLocation::Path, true, "string"),
                    ];
                    o.responses = [
                        (
                            "200".into(),
                            resp(
                                "Pipeline run detail",
                                "#/components/schemas/PipelineRunDetail",
                            ),
                        ),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o.security = Some(vec![Map::new()]);
                    o
                }),
                delete: Some({
                    let mut o = op("cancelPipeline", "Cancel a pipeline run", "pipelines");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("repo", ParameterLocation::Path, true, "string"),
                        p("pipeline_id", ParameterLocation::Path, true, "string"),
                    ];
                    o.responses = [
                        (
                            "200".into(),
                            resp("Canceled", "#/components/schemas/PipelineRun"),
                        ),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                post: None,
                put: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/repos/{owner}/{repo}/pipelines/{pipeline_id}/jobs",
            PathItem {
                get: Some({
                    let mut o = op("getPipelineJobs", "Get pipeline jobs", "pipelines");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("repo", ParameterLocation::Path, true, "string"),
                        p("pipeline_id", ParameterLocation::Path, true, "string"),
                    ];
                    o.responses = [
                        (
                            "200".into(),
                            resp("Jobs", "#/components/schemas/RunJobList"),
                        ),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o.security = Some(vec![Map::new()]);
                    o
                }),
                post: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/pipelines",
            PathItem {
                get: Some({
                    let mut o = op(
                        "listAllPipelines",
                        "List all pipelines (admin)",
                        "pipelines",
                    );
                    o.responses = [
                        (
                            "200".into(),
                            resp("Pipeline runs", "#/components/schemas/PipelineRunList"),
                        ),
                        resp_err("401", "Not authenticated"),
                    ]
                    .into();
                    o
                }),
                post: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        // ── Runners ─────────────────────────────────────────────
        .register_path(
            "/api/v1/runners",
            PathItem {
                get: Some({
                    let mut o = op("listRunners", "List registered runners", "runners");
                    o.responses = [
                        (
                            "200".into(),
                            resp("Runners", "#/components/schemas/RunnerList"),
                        ),
                        resp_err("401", "Not authenticated"),
                    ]
                    .into();
                    o
                }),
                post: Some({
                    let mut o = op("registerRunner", "Register a new runner", "runners");
                    o.request_body = Some(RequestBody {
                        description: Some("Runner registration".into()),
                        content: [(
                            "application/json".into(),
                            MediaType {
                                schema: Some(schema_obj(vec![
                                    ("name", "string"),
                                    ("scope", "string"),
                                    ("labels", "string"),
                                ])),
                            },
                        )]
                        .into(),
                        required: true,
                    });
                    o.responses = [
                        (
                            "201".into(),
                            resp("Registered", "#/components/schemas/RegisterRunnerResponse"),
                        ),
                        resp_err("401", "Not authenticated"),
                        resp_err("403", "Admin required"),
                    ]
                    .into();
                    o
                }),
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/runners/{runner_id}",
            PathItem {
                get: Some({
                    let mut o = op("getRunner", "Get runner details", "runners");
                    o.parameters = vec![p("runner_id", ParameterLocation::Path, true, "string")];
                    o.responses = [
                        ("200".into(), resp("Runner", "#/components/schemas/Runner")),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                delete: Some({
                    let mut o = op("deleteRunner", "Delete a runner", "runners");
                    o.parameters = vec![p("runner_id", ParameterLocation::Path, true, "string")];
                    o.responses = [
                        ("204".into(), resp("Deleted", "")),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                post: None,
                put: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/runners/poll",
            PathItem {
                post: Some({
                    let mut o = op("pollJob", "Poll for available jobs", "runners");
                    o.responses = [
                        (
                            "200".into(),
                            resp("Available job", "#/components/schemas/PollJobResponse"),
                        ),
                        ("204".into(), resp("No jobs available", "")),
                        resp_err("401", "Invalid runner token"),
                    ]
                    .into();
                    o
                }),
                get: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        // ── Search ───────────────────────────────────────────────
        .register_path(
            "/api/v1/search",
            PathItem {
                get: Some({
                    let mut o = op("globalSearch", "Global code search", "search");
                    o.parameters = vec![
                        p("q", ParameterLocation::Query, true, "string"),
                        p("repo", ParameterLocation::Query, false, "string"),
                        p("language", ParameterLocation::Query, false, "string"),
                        p("page", ParameterLocation::Query, false, "integer"),
                        p("per_page", ParameterLocation::Query, false, "integer"),
                    ];
                    o.responses = [
                        (
                            "200".into(),
                            resp("Search results", "#/components/schemas/SearchResults"),
                        ),
                        resp_err("400", "Missing query"),
                    ]
                    .into();
                    o.security = Some(vec![Map::new()]);
                    o
                }),
                post: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/repos/{owner}/{name}/search",
            PathItem {
                get: Some({
                    let mut o = op("repoSearch", "Search within a repo", "search");
                    o.parameters = vec![
                        p("owner", ParameterLocation::Path, true, "string"),
                        p("name", ParameterLocation::Path, true, "string"),
                        p("q", ParameterLocation::Query, true, "string"),
                        p("language", ParameterLocation::Query, false, "string"),
                        p("path", ParameterLocation::Query, false, "string"),
                    ];
                    o.responses = [
                        (
                            "200".into(),
                            resp("Search results", "#/components/schemas/SearchResults"),
                        ),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                post: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        // ── Activity ────────────────────────────────────────────
        .register_path(
            "/api/v1/activity",
            PathItem {
                get: Some({
                    let mut o = op("listActivity", "List activity feed", "activity");
                    o.parameters = vec![
                        p("limit", ParameterLocation::Query, false, "integer"),
                        p("offset", ParameterLocation::Query, false, "integer"),
                        p("repo_id", ParameterLocation::Query, false, "string"),
                        p("org_id", ParameterLocation::Query, false, "string"),
                    ];
                    o.responses = [(
                        "200".into(),
                        resp("Activity events", "#/components/schemas/ActivityList"),
                    )]
                    .into();
                    o.security = Some(vec![Map::new()]);
                    o
                }),
                post: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        // ── Users ────────────────────────────────────────────────
        .register_path(
            "/api/v1/users",
            PathItem {
                get: Some({
                    let mut o = op("listUsers", "List users", "users");
                    o.responses = [
                        (
                            "200".into(),
                            resp("User list", "#/components/schemas/UserList"),
                        ),
                        resp_err("401", "Not authenticated"),
                    ]
                    .into();
                    o
                }),
                post: Some({
                    let mut o = op("createUser", "Create a user", "users");
                    o.request_body = Some(RequestBody {
                        description: Some("User to create".into()),
                        content: [(
                            "application/json".into(),
                            MediaType {
                                schema: Some(schema_obj(vec![
                                    ("username", "string"),
                                    ("email", "string"),
                                    ("display_name", "string"),
                                    ("role", "string"),
                                ])),
                            },
                        )]
                        .into(),
                        required: true,
                    });
                    o.responses = [
                        (
                            "201".into(),
                            resp("Created user", "#/components/schemas/User"),
                        ),
                        resp_err("400", "Invalid request"),
                        resp_err("401", "Not authenticated"),
                    ]
                    .into();
                    o
                }),
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/users/{id}",
            PathItem {
                get: Some({
                    let mut o = op("getUser", "Get user by ID", "users");
                    o.parameters = vec![p("id", ParameterLocation::Path, true, "string")];
                    o.responses = [
                        ("200".into(), resp("User", "#/components/schemas/User")),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                patch: Some({
                    let mut o = op("updateUser", "Update a user", "users");
                    o.parameters = vec![p("id", ParameterLocation::Path, true, "string")];
                    o.request_body = Some(RequestBody {
                        description: Some("Fields to update".into()),
                        content: [(
                            "application/json".into(),
                            MediaType {
                                schema: Some(schema_obj(vec![
                                    ("username", "string"),
                                    ("email", "string"),
                                    ("display_name", "string"),
                                    ("bio", "string"),
                                ])),
                            },
                        )]
                        .into(),
                        required: true,
                    });
                    o.responses = [
                        (
                            "200".into(),
                            resp("Updated user", "#/components/schemas/User"),
                        ),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                delete: Some({
                    let mut o = op("deleteUser", "Delete a user", "users");
                    o.parameters = vec![p("id", ParameterLocation::Path, true, "string")];
                    o.responses = [
                        ("204".into(), resp("Deleted", "")),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                post: None,
                put: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        // ── Organizations ───────────────────────────────────────
        .register_path(
            "/api/v1/orgs",
            PathItem {
                get: Some({
                    let mut o = op("listOrgs", "List organizations", "orgs");
                    o.responses = [
                        (
                            "200".into(),
                            resp("Org list", "#/components/schemas/OrgList"),
                        ),
                        resp_err("401", "Not authenticated"),
                    ]
                    .into();
                    o
                }),
                post: Some({
                    let mut o = op("createOrg", "Create an organization", "orgs");
                    o.request_body = Some(RequestBody {
                        description: Some("Organization to create".into()),
                        content: [(
                            "application/json".into(),
                            MediaType {
                                schema: Some(schema_obj(vec![
                                    ("name", "string"),
                                    ("display_name", "string"),
                                    ("description", "string"),
                                    ("visibility", "string"),
                                ])),
                            },
                        )]
                        .into(),
                        required: true,
                    });
                    o.responses = [
                        (
                            "201".into(),
                            resp("Created org", "#/components/schemas/Org"),
                        ),
                        resp_err("400", "Invalid request"),
                        resp_err("403", "Permission denied"),
                    ]
                    .into();
                    o
                }),
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/orgs/{id}",
            PathItem {
                get: Some({
                    let mut o = op("getOrg", "Get organization", "orgs");
                    o.parameters = vec![p("id", ParameterLocation::Path, true, "string")];
                    o.responses = [
                        (
                            "200".into(),
                            resp("Organization", "#/components/schemas/Org"),
                        ),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                patch: Some({
                    let mut o = op("updateOrg", "Update an organization", "orgs");
                    o.parameters = vec![p("id", ParameterLocation::Path, true, "string")];
                    o.responses = [
                        (
                            "200".into(),
                            resp("Updated org", "#/components/schemas/Org"),
                        ),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                post: None,
                put: None,
                delete: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        // ── SSH Keys ────────────────────────────────────────────
        .register_path(
            "/api/v1/users/{user_id}/ssh-keys",
            PathItem {
                get: Some({
                    let mut o = op("listSshKeys", "List SSH keys for a user", "users");
                    o.parameters = vec![p("user_id", ParameterLocation::Path, true, "string")];
                    o.responses = [
                        (
                            "200".into(),
                            resp("SSH keys", "#/components/schemas/SshKeyList"),
                        ),
                        resp_err("401", "Not authenticated"),
                    ]
                    .into();
                    o
                }),
                post: Some({
                    let mut o = op("addSshKey", "Add SSH key", "users");
                    o.parameters = vec![p("user_id", ParameterLocation::Path, true, "string")];
                    o.request_body = Some(RequestBody {
                        description: Some("SSH key to add".into()),
                        content: [(
                            "application/json".into(),
                            MediaType {
                                schema: Some(schema_obj(vec![
                                    ("label", "string"),
                                    ("public_key", "string"),
                                ])),
                            },
                        )]
                        .into(),
                        required: true,
                    });
                    o.responses = [
                        (
                            "201".into(),
                            resp("Added key", "#/components/schemas/SshKey"),
                        ),
                        resp_err("400", "Invalid key"),
                    ]
                    .into();
                    o
                }),
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/ssh-keys/{key_id}",
            PathItem {
                delete: Some({
                    let mut o = op("deleteSshKey", "Delete SSH key", "users");
                    o.parameters = vec![p("key_id", ParameterLocation::Path, true, "string")];
                    o.responses = [
                        ("204".into(), resp("Deleted", "")),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                get: None,
                post: None,
                put: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        // ── Password ─────────────────────────────────────────────
        .register_path(
            "/api/v1/users/{user_id}/password",
            PathItem {
                post: Some({
                    let mut o = op("changePassword", "Change user password", "auth");
                    o.parameters = vec![p("user_id", ParameterLocation::Path, true, "string")];
                    o.request_body = Some(RequestBody {
                        description: Some("Password change".into()),
                        content: [(
                            "application/json".into(),
                            MediaType {
                                schema: Some(schema_obj(vec![
                                    ("current_password", "string"),
                                    ("new_password", "string"),
                                ])),
                            },
                        )]
                        .into(),
                        required: true,
                    });
                    o.responses = [
                        (
                            "200".into(),
                            resp("Password changed", "#/components/schemas/Message"),
                        ),
                        resp_err("400", "Invalid request"),
                        resp_err("401", "Not authenticated"),
                    ]
                    .into();
                    o
                }),
                get: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        // ── Federation ──────────────────────────────────────────
        .register_path(
            "/.well-known/webfinger",
            PathItem {
                get: Some({
                    let mut o = op("webfinger", "WebFinger discovery", "federation");
                    o.parameters = vec![p("resource", ParameterLocation::Query, true, "string")];
                    o.responses = [
                        (
                            "200".into(),
                            resp(
                                "WebFinger response",
                                "#/components/schemas/WebFingerResponse",
                            ),
                        ),
                        resp_err("404", "Federation disabled"),
                    ]
                    .into();
                    o.security = Some(vec![Map::new()]);
                    o
                }),
                post: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/federation/actor",
            PathItem {
                get: Some({
                    let mut o = op("federationActor", "ActivityPub actor", "federation");
                    o.responses = [
                        (
                            "200".into(),
                            resp("Actor", "#/components/schemas/ActorResponse"),
                        ),
                        resp_err("404", "Federation disabled"),
                    ]
                    .into();
                    o
                }),
                post: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/federation/inbox",
            PathItem {
                post: Some({
                    let mut o = op(
                        "federationInbox",
                        "Receive ActivityPub activities",
                        "federation",
                    );
                    o.responses = [
                        (
                            "202".into(),
                            resp("Accepted", "#/components/schemas/InboxResponse"),
                        ),
                        resp_err("401", "Invalid signature"),
                        resp_err("404", "Federation disabled"),
                    ]
                    .into();
                    o
                }),
                get: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/federation/outbox",
            PathItem {
                get: Some({
                    let mut o = op("federationOutbox", "ActivityPub outbox", "federation");
                    o.responses = [
                        (
                            "200".into(),
                            resp("Outbox", "#/components/schemas/OutboxResponse"),
                        ),
                        resp_err("404", "Federation disabled"),
                    ]
                    .into();
                    o
                }),
                post: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        // ── Marketplace ────────────────────────────────────────
        .register_path(
            "/api/v1/marketplace/extensions",
            PathItem {
                get: Some({
                    let mut o = op(
                        "listExtensions",
                        "List marketplace extensions",
                        "marketplace",
                    );
                    o.responses = [(
                        "200".into(),
                        resp("Extensions", "#/components/schemas/ExtensionSummaryList"),
                    )]
                    .into();
                    o.security = Some(vec![Map::new()]);
                    o
                }),
                post: Some({
                    let mut o = op("publishExtension", "Publish an extension", "marketplace");
                    o.request_body = Some(RequestBody {
                        description: Some("Extension manifest".into()),
                        content: [(
                            "application/json".into(),
                            MediaType {
                                schema: Some(SchemaRef {
                                    ref_: Some("#/components/schemas/ExtensionManifest".into()),
                                    type_: None,
                                    properties: Map::new(),
                                    required: Vec::new(),
                                    items: None,
                                    description: None,
                                    example: None,
                                }),
                            },
                        )]
                        .into(),
                        required: true,
                    });
                    o.responses = [
                        (
                            "201".into(),
                            resp("Published", "#/components/schemas/Message"),
                        ),
                        resp_err("400", "Invalid manifest"),
                        resp_err("403", "Permission denied"),
                    ]
                    .into();
                    o
                }),
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/marketplace/extensions/{name}",
            PathItem {
                get: Some({
                    let mut o = op("getExtension", "Get extension details", "marketplace");
                    o.parameters = vec![p("name", ParameterLocation::Path, true, "string")];
                    o.responses = [
                        (
                            "200".into(),
                            resp("Extension", "#/components/schemas/ExtensionManifest"),
                        ),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o.security = Some(vec![Map::new()]);
                    o
                }),
                delete: Some({
                    let mut o = op(
                        "deleteExtension",
                        "Remove extension from marketplace",
                        "marketplace",
                    );
                    o.parameters = vec![p("name", ParameterLocation::Path, true, "string")];
                    o.responses = [
                        ("204".into(), resp("Deleted", "")),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                post: None,
                put: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/marketplace/extensions/{name}/verify",
            PathItem {
                post: Some({
                    let mut o = op(
                        "verifyExtension",
                        "Verify extension signature",
                        "marketplace",
                    );
                    o.parameters = vec![p("name", ParameterLocation::Path, true, "string")];
                    o.responses = [
                        (
                            "200".into(),
                            resp("Verification result", "#/components/schemas/VerifyResponse"),
                        ),
                        resp_err("404", "Not found"),
                    ]
                    .into();
                    o
                }),
                get: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/marketplace/installed",
            PathItem {
                get: Some({
                    let mut o = op("listInstalled", "List installed extensions", "marketplace");
                    o.responses = [(
                        "200".into(),
                        resp(
                            "Installed extensions",
                            "#/components/schemas/ExtensionManifestList",
                        ),
                    )]
                    .into();
                    o
                }),
                post: None,
                put: None,
                delete: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        )
        .register_path(
            "/api/v1/marketplace/installed/{name}",
            PathItem {
                post: Some({
                    let mut o = op("installExtension", "Install an extension", "marketplace");
                    o.parameters = vec![p("name", ParameterLocation::Path, true, "string")];
                    o.responses = [
                        (
                            "201".into(),
                            resp("Installed", "#/components/schemas/Message"),
                        ),
                        resp_err("404", "Not found"),
                        resp_err("409", "Already installed"),
                    ]
                    .into();
                    o
                }),
                delete: Some({
                    let mut o = op(
                        "uninstallExtension",
                        "Uninstall an extension",
                        "marketplace",
                    );
                    o.parameters = vec![p("name", ParameterLocation::Path, true, "string")];
                    o.responses = [
                        ("204".into(), resp("Uninstalled", "")),
                        resp_err("404", "Not installed"),
                    ]
                    .into();
                    o
                }),
                get: None,
                put: None,
                patch: None,
                parameters: Vec::new(),
                summary: None,
                description: None,
            },
        );

    // ── Component Schemas ─────────────────────────────────────
    g = g
        .add_schema("Message", schema_obj(vec![("message", "string")]))
        .add_schema(
            "Repo",
            schema_obj(vec![
                ("id", "string"),
                ("name", "string"),
                ("owner", "string"),
                ("description", "string"),
                ("visibility", "string"),
                ("created_at", "string"),
                ("updated_at", "string"),
            ]),
        )
        .add_schema(
            "RepoList",
            SchemaRef {
                ref_: None,
                type_: Some("array".into()),
                properties: Map::new(),
                required: Vec::new(),
                items: Some(Box::new(SchemaRef {
                    ref_: Some("#/components/schemas/Repo".into()),
                    type_: None,
                    properties: Map::new(),
                    required: Vec::new(),
                    items: None,
                    description: None,
                    example: None,
                })),
                description: None,
                example: None,
            },
        )
        .add_schema("LoginResponse", schema_obj(vec![("token", "string")]))
        .add_schema("RefreshResponse", schema_obj(vec![("token", "string")]))
        .add_schema(
            "User",
            schema_obj(vec![
                ("id", "string"),
                ("username", "string"),
                ("email", "string"),
                ("display_name", "string"),
                ("role", "string"),
                ("created_at", "string"),
                ("updated_at", "string"),
            ]),
        )
        .add_schema(
            "Org",
            schema_obj(vec![
                ("id", "string"),
                ("name", "string"),
                ("display_name", "string"),
                ("description", "string"),
                ("visibility", "string"),
                ("owner_id", "string"),
                ("created_at", "string"),
                ("updated_at", "string"),
            ]),
        )
        .add_schema(
            "Issue",
            schema_obj(vec![
                ("id", "string"),
                ("repo_id", "string"),
                ("number", "integer"),
                ("title", "string"),
                ("body", "string"),
                ("status", "string"),
                ("author_id", "string"),
                ("created_at", "string"),
                ("updated_at", "string"),
            ]),
        )
        .add_schema(
            "PipelineRun",
            schema_obj(vec![
                ("id", "string"),
                ("repo_id", "string"),
                ("trigger", "string"),
                ("commit_sha", "string"),
                ("status", "string"),
                ("created_at", "string"),
                ("started_at", "string"),
                ("finished_at", "string"),
            ]),
        )
        .add_schema(
            "Runner",
            schema_obj(vec![
                ("id", "string"),
                ("name", "string"),
                ("scope", "string"),
                ("status", "string"),
                ("labels", "string"),
                ("created_at", "string"),
            ]),
        )
        .add_schema(
            "WebFingerResponse",
            schema_obj(vec![
                ("subject", "string"),
                ("aliases", "string"),
                ("links", "string"),
            ]),
        )
        .add_schema(
            "ActorResponse",
            schema_obj(vec![
                ("id", "string"),
                ("type", "string"),
                ("preferred_username", "string"),
                ("name", "string"),
                ("inbox", "string"),
                ("outbox", "string"),
            ]),
        )
        .add_schema(
            "ExtensionManifest",
            schema_obj(vec![
                ("name", "string"),
                ("version", "string"),
                ("description", "string"),
                ("author", "string"),
                ("license", "string"),
                ("entrypoint", "string"),
            ]),
        )
        .add_schema(
            "VerifyResponse",
            schema_obj(vec![
                ("valid", "boolean"),
                ("errors", "string"),
                ("warnings", "string"),
            ]),
        );
    g
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_new() {
        let generator = OpenApiGenerator::new();
        let spec = generator.generate();
        assert_eq!(spec.openapi, "3.1.0");
        assert_eq!(spec.info.version, "0.1.0");
        assert!(spec.paths.is_empty());
    }

    #[test]
    fn test_generator_with_info() {
        let generator = OpenApiGenerator::new().with_info("My API", "1.0.0");
        let spec = generator.generate();
        assert_eq!(spec.info.title, "My API");
        assert_eq!(spec.info.version, "1.0.0");
    }

    #[test]
    fn test_generator_with_description() {
        let generator = OpenApiGenerator::new()
            .with_info("Test", "1.0.0")
            .with_description("A test API");
        let spec = generator.generate();
        assert_eq!(spec.info.description.as_deref(), Some("A test API"));
    }

    #[test]
    fn test_generator_with_contact() {
        let generator = OpenApiGenerator::new()
            .with_info("Test", "1.0.0")
            .with_contact("Team", "team@example.com");
        let spec = generator.generate();
        let contact = spec.info.contact.unwrap();
        assert_eq!(contact.name.as_deref(), Some("Team"));
        assert_eq!(contact.email.as_deref(), Some("team@example.com"));
    }

    #[test]
    fn test_generator_with_license() {
        let generator = OpenApiGenerator::new()
            .with_info("Test", "1.0.0")
            .with_license("MIT", "https://opensource.org/licenses/MIT");
        let spec = generator.generate();
        let license = spec.info.license.unwrap();
        assert_eq!(license.name, "MIT");
    }

    #[test]
    fn test_generator_with_server() {
        let generator = OpenApiGenerator::new()
            .with_info("Test", "1.0.0")
            .with_server("http://localhost:8080", "Development");
        let spec = generator.generate();
        assert_eq!(spec.servers.len(), 1);
        assert_eq!(spec.servers[0].url, "http://localhost:8080");
    }

    #[test]
    fn test_register_path() {
        let path_item = PathItem {
            get: Some(Operation {
                operation_id: Some("getUsers".into()),
                summary: Some("List users".into()),
                description: None,
                parameters: vec![],
                responses: HashMap::new(),
                tags: vec!["users".into()],
                request_body: None,
                security: None,
            }),
            post: None,
            put: None,
            delete: None,
            patch: None,
            parameters: vec![],
            summary: None,
            description: None,
        };
        let generator = OpenApiGenerator::new()
            .with_info("Test", "1.0.0")
            .register_path("/users", path_item);
        let spec = generator.generate();
        assert!(spec.paths.contains_key("/users"));
        let item = spec.paths.get("/users").unwrap();
        assert!(item.get.is_some());
        assert_eq!(
            item.get.as_ref().unwrap().operation_id.as_deref(),
            Some("getUsers")
        );
    }

    #[test]
    fn test_register_multiple_paths() {
        let generator = OpenApiGenerator::new()
            .with_info("Test", "1.0.0")
            .register_path(
                "/users",
                PathItem {
                    get: Some(Operation {
                        operation_id: Some("listUsers".into()),
                        summary: None,
                        description: None,
                        parameters: vec![],
                        responses: HashMap::new(),
                        tags: vec![],
                        request_body: None,
                        security: None,
                    }),
                    post: None,
                    put: None,
                    delete: None,
                    patch: None,
                    parameters: vec![],
                    summary: None,
                    description: None,
                },
            )
            .register_path(
                "/repos",
                PathItem {
                    get: Some(Operation {
                        operation_id: Some("listRepos".into()),
                        summary: None,
                        description: None,
                        parameters: vec![],
                        responses: HashMap::new(),
                        tags: vec![],
                        request_body: None,
                        security: None,
                    }),
                    post: None,
                    put: None,
                    delete: None,
                    patch: None,
                    parameters: vec![],
                    summary: None,
                    description: None,
                },
            );
        let spec = generator.generate();
        assert_eq!(spec.paths.len(), 2);
    }

    #[test]
    fn test_parameter_struct() {
        let param = Parameter {
            name: "page".into(),
            location: ParameterLocation::Query,
            description: Some("Page number".into()),
            required: false,
            schema: Some(SchemaRef {
                ref_: None,
                type_: Some("integer".into()),
                properties: HashMap::new(),
                required: vec![],
                items: None,
                description: None,
                example: Some(serde_json::Value::Number(1.into())),
            }),
            example: None,
        };
        assert_eq!(param.name, "page");
        assert_eq!(param.location, ParameterLocation::Query);
        assert!(!param.required);
    }

    #[test]
    fn test_response_struct() {
        let resp = Response {
            description: "Not found".into(),
            status_code: Some("404".into()),
            content_type: Some("application/json".into()),
            schema_ref: None,
        };
        assert_eq!(resp.description, "Not found");
        assert_eq!(resp.status_code.as_deref(), Some("404"));
    }

    #[test]
    fn test_schema_ref() {
        let schema = SchemaRef {
            ref_: Some("#/components/schemas/User".into()),
            type_: None,
            properties: HashMap::new(),
            required: vec![],
            items: None,
            description: None,
            example: None,
        };
        assert_eq!(schema.ref_.as_deref(), Some("#/components/schemas/User"));
    }

    #[test]
    fn test_schema_ref_with_properties() {
        let mut props = HashMap::new();
        props.insert(
            "name".into(),
            SchemaRef {
                ref_: None,
                type_: Some("string".into()),
                properties: HashMap::new(),
                required: vec![],
                items: None,
                description: None,
                example: None,
            },
        );
        let schema = SchemaRef {
            ref_: None,
            type_: Some("object".into()),
            properties: props,
            required: vec!["name".into()],
            items: None,
            description: None,
            example: None,
        };
        assert_eq!(schema.type_.as_deref(), Some("object"));
        assert_eq!(schema.required.len(), 1);
        assert!(schema.properties.contains_key("name"));
    }

    #[test]
    fn test_security_scheme() {
        let scheme = SecurityScheme {
            type_: "http".into(),
            name: None,
            in_: None,
            description: Some("Bearer token".into()),
            bearer_format: Some("JWT".into()),
        };
        assert_eq!(scheme.type_, "http");
        assert_eq!(scheme.bearer_format.as_deref(), Some("JWT"));
    }

    #[test]
    fn test_add_schema() {
        let generator = OpenApiGenerator::new()
            .with_info("Test", "1.0.0")
            .add_schema(
                "User",
                SchemaRef {
                    ref_: None,
                    type_: Some("object".into()),
                    properties: HashMap::new(),
                    required: vec!["id".into()],
                    items: None,
                    description: Some("A user".into()),
                    example: None,
                },
            );
        let spec = generator.generate();
        let components = spec.components.unwrap();
        assert!(components.schemas.contains_key("User"));
    }

    #[test]
    fn test_add_security_scheme() {
        let generator = OpenApiGenerator::new()
            .with_info("Test", "1.0.0")
            .add_security_scheme(
                "bearerAuth",
                SecurityScheme {
                    type_: "http".into(),
                    name: None,
                    in_: None,
                    description: None,
                    bearer_format: Some("JWT".into()),
                },
            );
        let spec = generator.generate();
        let components = spec.components.unwrap();
        assert!(components.security_schemes.contains_key("bearerAuth"));
    }

    #[test]
    fn test_add_global_security() {
        let mut req = HashMap::new();
        req.insert("bearerAuth".into(), vec![]);
        let generator = OpenApiGenerator::new()
            .with_info("Test", "1.0.0")
            .add_global_security(req);
        let spec = generator.generate();
        assert_eq!(spec.security.len(), 1);
    }

    #[test]
    fn test_to_json() {
        let generator = OpenApiGenerator::new().with_info("Test API", "1.0.0");
        let json = generator.to_json();
        assert!(json.contains("Test API"));
        assert!(json.contains("3.1.0"));
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["info"]["title"], "Test API");
    }

    #[test]
    fn test_to_yaml() {
        let generator = OpenApiGenerator::new().with_info("Test API", "1.0.0");
        let yaml = generator.to_yaml();
        assert!(yaml.contains("openapi:"));
        assert!(yaml.contains("title:"));
    }

    #[test]
    fn test_full_api_spec() {
        let path_item = PathItem {
            get: Some(Operation {
                operation_id: Some("getUser".into()),
                summary: Some("Get a user by ID".into()),
                description: Some("Returns a single user".into()),
                parameters: vec![Parameter {
                    name: "id".into(),
                    location: ParameterLocation::Path,
                    description: Some("User ID".into()),
                    required: true,
                    schema: Some(SchemaRef {
                        ref_: None,
                        type_: Some("string".into()),
                        properties: HashMap::new(),
                        required: vec![],
                        items: None,
                        description: None,
                        example: None,
                    }),
                    example: None,
                }],
                responses: {
                    let mut r = HashMap::new();
                    r.insert(
                        "200".into(),
                        Response {
                            description: "OK".into(),
                            status_code: Some("200".into()),
                            content_type: Some("application/json".into()),
                            schema_ref: Some(SchemaRef {
                                ref_: Some("#/components/schemas/User".into()),
                                type_: None,
                                properties: HashMap::new(),
                                required: vec![],
                                items: None,
                                description: None,
                                example: None,
                            }),
                        },
                    );
                    r
                },
                tags: vec!["users".into()],
                request_body: None,
                security: None,
            }),
            post: Some(Operation {
                operation_id: Some("createUser".into()),
                summary: Some("Create a user".into()),
                description: None,
                parameters: vec![],
                responses: HashMap::new(),
                tags: vec!["users".into()],
                request_body: Some(RequestBody {
                    description: Some("User to create".into()),
                    content: {
                        let mut c = HashMap::new();
                        c.insert(
                            "application/json".into(),
                            MediaType {
                                schema: Some(SchemaRef {
                                    ref_: Some("#/components/schemas/CreateUser".into()),
                                    type_: None,
                                    properties: HashMap::new(),
                                    required: vec![],
                                    items: None,
                                    description: None,
                                    example: None,
                                }),
                            },
                        );
                        c
                    },
                    required: true,
                }),
                security: None,
            }),
            put: None,
            delete: None,
            patch: None,
            parameters: vec![],
            summary: None,
            description: None,
        };

        let generator = OpenApiGenerator::new()
            .with_info("User Service", "2.0.0")
            .with_description("User management API")
            .with_server("http://localhost:3000", "Local")
            .register_path("/users/{id}", path_item)
            .add_security_scheme(
                "bearerAuth",
                SecurityScheme {
                    type_: "http".into(),
                    name: None,
                    in_: None,
                    description: Some("JWT Bearer token".into()),
                    bearer_format: Some("JWT".into()),
                },
            );

        let spec = generator.generate();
        assert_eq!(spec.info.title, "User Service");
        assert_eq!(spec.openapi, "3.1.0");
        assert_eq!(spec.paths.len(), 1);
        let item = spec.paths.get("/users/{id}").unwrap();
        assert!(item.get.is_some());
        assert!(item.post.is_some());
        assert!(item.delete.is_none());
        assert_eq!(item.get.as_ref().unwrap().parameters.len(), 1);
        let param = &item.get.as_ref().unwrap().parameters[0];
        assert_eq!(param.location, ParameterLocation::Path);
        assert!(param.required);

        let json = generator.to_json();
        assert!(json.contains("User Service"));
        assert!(json.contains("/users/{id}"));
        let _yaml = generator.to_yaml();
    }

    #[test]
    fn test_parameter_location_equality() {
        assert_eq!(ParameterLocation::Query, ParameterLocation::Query);
        assert_ne!(ParameterLocation::Query, ParameterLocation::Header);
    }

    #[test]
    fn test_path_item_with_path_parameters() {
        let path_item = PathItem {
            get: None,
            post: None,
            put: None,
            delete: None,
            patch: None,
            parameters: vec![Parameter {
                name: "org_id".into(),
                location: ParameterLocation::Path,
                description: None,
                required: true,
                schema: Some(SchemaRef {
                    ref_: None,
                    type_: Some("string".into()),
                    properties: HashMap::new(),
                    required: vec![],
                    items: None,
                    description: None,
                    example: None,
                }),
                example: None,
            }],
            summary: None,
            description: None,
        };
        assert_eq!(path_item.parameters.len(), 1);
        assert_eq!(path_item.parameters[0].name, "org_id");
    }

    #[test]
    fn test_operation_security() {
        let mut sec_req = HashMap::new();
        sec_req.insert("bearerAuth".into(), vec!["read:users".into()]);
        let op = Operation {
            operation_id: None,
            summary: None,
            description: None,
            parameters: vec![],
            responses: HashMap::new(),
            tags: vec![],
            request_body: None,
            security: Some(vec![sec_req]),
        };
        assert!(op.security.is_some());
        assert_eq!(op.security.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_schema_ref_array() {
        let schema = SchemaRef {
            ref_: None,
            type_: Some("array".into()),
            properties: HashMap::new(),
            required: vec![],
            items: Some(Box::new(SchemaRef {
                ref_: Some("#/components/schemas/User".into()),
                type_: None,
                properties: HashMap::new(),
                required: vec![],
                items: None,
                description: None,
                example: None,
            })),
            description: Some("List of users".into()),
            example: None,
        };
        assert_eq!(schema.type_.as_deref(), Some("array"));
        assert!(schema.items.is_some());
    }
}
