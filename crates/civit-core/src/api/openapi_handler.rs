#![forbid(unsafe_code)]

use axum::http::{StatusCode, header};
use axum::response::IntoResponse;

use crate::docs::openapi::*;

/// GET /api/v1/openapi.json — Serve the OpenAPI specification as JSON
pub async fn get_openapi_spec() -> impl IntoResponse {
    let spec = generate_openapi_spec();
    let json = spec.to_json();
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        json,
    )
        .into_response()
}

/// GET /api/v1/openapi.yaml — Serve the OpenAPI specification as YAML
pub async fn get_openapi_yaml() -> impl IntoResponse {
    let spec = generate_openapi_spec();
    let yaml = spec.to_yaml();
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/yaml")],
        yaml,
    )
        .into_response()
}

pub fn openapi_routes() -> axum::Router<crate::api::AppState> {
    axum::Router::new()
        .route("/api/v1/openapi.json", axum::routing::get(get_openapi_spec))
        .route("/api/v1/openapi.yaml", axum::routing::get(get_openapi_yaml))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_spec_generates_valid_json() {
        let spec = generate_openapi_spec();
        let json = spec.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["openapi"], "3.1.0");
        assert!(!parsed["paths"].as_object().unwrap().is_empty());
        assert!(parsed["components"]["security_schemes"].is_object());
    }

    #[test]
    fn test_openapi_spec_has_info() {
        let spec = generate_openapi_spec();
        let json = spec.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["info"]["title"], "CivitForge API");
        assert!(parsed["info"]["license"]["name"].is_string());
    }

    #[test]
    fn test_openapi_yaml_contains_openapi_key() {
        let spec = generate_openapi_spec();
        let yaml = spec.to_yaml();
        assert!(yaml.contains("openapi:"));
        assert!(yaml.contains("title:"));
    }

    #[test]
    fn test_openapi_routes_compile() {
        let router = openapi_routes();
        let _ = router;
    }
}
