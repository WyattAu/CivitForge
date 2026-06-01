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
