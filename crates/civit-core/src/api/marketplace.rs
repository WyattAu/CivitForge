#![forbid(unsafe_code)]

use axum::{
    Router,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use parking_lot::Mutex;

// ---------------------------------------------------------------------------
// Extension types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ExtensionManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub permissions: Vec<ExtensionPermission>,
    pub hooks: Vec<ExtensionHook>,
    pub entrypoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
pub enum ExtensionPermission {
    ReadRepos,
    WriteRepos,
    ReadIssues,
    WriteIssues,
    ReadPipelines,
    NetworkAccess,
    StorageAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ExtensionHook {
    pub event: String,
    pub handler: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct ExtensionSummary {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub installed: bool,
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

// ---------------------------------------------------------------------------
// In-memory extension store (placeholder for future DB integration)
// ---------------------------------------------------------------------------

static EXTENSIONS: std::sync::OnceLock<Mutex<Vec<ExtensionManifest>>> =
    std::sync::OnceLock::new();

static INSTALLED: std::sync::OnceLock<Mutex<Vec<String>>> = std::sync::OnceLock::new();

fn extensions_store() -> &'static Mutex<Vec<ExtensionManifest>> {
    EXTENSIONS.get_or_init(|| Mutex::new(Vec::new()))
}

fn installed_store() -> &'static Mutex<Vec<String>> {
    INSTALLED.get_or_init(|| Mutex::new(Vec::new()))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn marketplace_routes() -> Router<crate::api::AppState> {
    Router::new()
        .route(
            "/api/v1/marketplace/extensions",
            get(list_extensions).post(publish_extension),
        )
        .route(
            "/api/v1/marketplace/extensions/{name}",
            get(get_extension).delete(delete_extension),
        )
        .route(
            "/api/v1/marketplace/extensions/{name}/verify",
            post(verify_extension),
        )
        .route("/api/v1/marketplace/installed", get(list_installed))
        .route(
            "/api/v1/marketplace/installed/{name}",
            post(install_extension).delete(uninstall_extension),
        )
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn list_extensions() -> impl IntoResponse {
    let store = extensions_store().lock();
    let summaries: Vec<ExtensionSummary> = store
        .iter()
        .map(|ext| {
            let installed = installed_store().lock().contains(&ext.name);
            ExtensionSummary {
                name: ext.name.clone(),
                version: ext.version.clone(),
                description: ext.description.clone(),
                author: ext.author.clone(),
                installed,
            }
        })
        .collect();
    (StatusCode::OK, Json(summaries)).into_response()
}

pub async fn get_extension(Path(name): Path<String>) -> impl IntoResponse {
    let store = extensions_store().lock();
    match store.iter().find(|e| e.name == name) {
        Some(ext) => (StatusCode::OK, Json(ext.clone())).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(MessageResponse {
                message: format!("extension '{name}' not found"),
            }),
        )
            .into_response(),
    }
}

pub async fn publish_extension(Json(manifest): Json<ExtensionManifest>) -> impl IntoResponse {
    let validation = ExtensionSandbox::validate_manifest(&manifest);
    if !validation.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(MessageResponse {
                message: format!("manifest validation failed: {}", validation.join(", ")),
            }),
        )
            .into_response();
    }

    if let Err(e) = ExtensionSandbox::check_permissions(&manifest) {
        return (
            StatusCode::FORBIDDEN,
            Json(MessageResponse {
                message: format!("permission check failed: {e}"),
            }),
        )
            .into_response();
    }

    let mut store = extensions_store().lock();
    if let Some(existing) = store.iter_mut().find(|e| e.name == manifest.name) {
        *existing = manifest;
        (
            StatusCode::OK,
            Json(MessageResponse {
                message: "extension updated".into(),
            }),
        )
            .into_response()
    } else {
        store.push(manifest);
        (
            StatusCode::CREATED,
            Json(MessageResponse {
                message: "extension published".into(),
            }),
        )
            .into_response()
    }
}

pub async fn delete_extension(Path(name): Path<String>) -> impl IntoResponse {
    let mut store = extensions_store().lock();
    let before = store.len();
    store.retain(|e| e.name != name);
    if store.len() < before {
        (
            StatusCode::NO_CONTENT,
            Json(MessageResponse {
                message: "extension deleted".into(),
            }),
        )
            .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(MessageResponse {
                message: format!("extension '{name}' not found"),
            }),
        )
            .into_response()
    }
}

pub async fn verify_extension(Path(name): Path<String>) -> impl IntoResponse {
    let store = extensions_store().lock();
    match store.iter().find(|e| e.name == name) {
        Some(manifest) => {
            let errors = ExtensionSandbox::validate_manifest(manifest);
            let warnings = ExtensionSandbox::lint_manifest(manifest);
            (
                StatusCode::OK,
                Json(VerifyResponse {
                    valid: errors.is_empty(),
                    errors,
                    warnings,
                }),
            )
                .into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(MessageResponse {
                message: format!("extension '{name}' not found"),
            }),
        )
            .into_response(),
    }
}

pub async fn list_installed() -> impl IntoResponse {
    let installed = installed_store().lock();
    let store = extensions_store().lock();
    let results: Vec<ExtensionManifest> = installed
        .iter()
        .filter_map(|name| store.iter().find(|e| &e.name == name).cloned())
        .collect();
    (StatusCode::OK, Json(results)).into_response()
}

pub async fn install_extension(Path(name): Path<String>) -> impl IntoResponse {
    let store = extensions_store().lock();
    if !store.iter().any(|e| e.name == name) {
        return (
            StatusCode::NOT_FOUND,
            Json(MessageResponse {
                message: format!("extension '{name}' not found in marketplace"),
            }),
        )
            .into_response();
    }
    drop(store);

    let mut installed = installed_store().lock();
    if installed.contains(&name) {
        return (
            StatusCode::CONFLICT,
            Json(MessageResponse {
                message: format!("extension '{name}' is already installed"),
            }),
        )
            .into_response();
    }
    installed.push(name);
    (
        StatusCode::CREATED,
        Json(MessageResponse {
            message: "extension installed".into(),
        }),
    )
        .into_response()
}

pub async fn uninstall_extension(Path(name): Path<String>) -> impl IntoResponse {
    let mut installed = installed_store().lock();
    let before = installed.len();
    installed.retain(|n| n != &name);
    if installed.len() < before {
        (
            StatusCode::NO_CONTENT,
            Json(MessageResponse {
                message: "extension uninstalled".into(),
            }),
        )
            .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(MessageResponse {
                message: format!("extension '{name}' is not installed"),
            }),
        )
            .into_response()
    }
}

// ---------------------------------------------------------------------------
// Extension Sandbox (validates manifest, no actual execution)
// ---------------------------------------------------------------------------

pub struct ExtensionSandbox;

impl ExtensionSandbox {
    pub fn validate_manifest(manifest: &ExtensionManifest) -> Vec<String> {
        let mut errors = Vec::new();

        if manifest.name.trim().is_empty() {
            errors.push("name is required".into());
        } else if !manifest
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            errors.push(
                "name must only contain alphanumeric characters, hyphens, or underscores".into(),
            );
        }
        if manifest.name.len() > 128 {
            errors.push("name must be at most 128 characters".into());
        }

        if manifest.version.trim().is_empty() {
            errors.push("version is required".into());
        } else if !Self::is_valid_semver(&manifest.version) {
            errors.push(format!(
                "version '{}' is not valid semver",
                manifest.version
            ));
        }

        if manifest.description.trim().is_empty() {
            errors.push("description is required".into());
        } else if manifest.description.len() > 2048 {
            errors.push("description must be at most 2048 characters".into());
        }

        if manifest.author.trim().is_empty() {
            errors.push("author is required".into());
        }

        if manifest.license.trim().is_empty() {
            errors.push("license is required".into());
        } else if !Self::is_valid_spdx(&manifest.license) {
            errors.push(format!(
                "license '{}' is not a valid SPDX identifier",
                manifest.license
            ));
        }

        if manifest.permissions.is_empty() {
            errors.push("at least one permission is required".into());
        }

        if manifest.entrypoint.trim().is_empty() {
            errors.push("entrypoint is required".into());
        } else if !manifest.entrypoint.ends_with(".wasm") && !manifest.entrypoint.ends_with(".js") {
            errors.push("entrypoint must end with .wasm or .js".into());
        }

        let mut hook_events = std::collections::HashSet::new();
        for hook in &manifest.hooks {
            if hook.event.trim().is_empty() {
                errors.push("hook event must not be empty".into());
            }
            if hook.handler.trim().is_empty() {
                errors.push(format!(
                    "hook handler for event '{}' must not be empty",
                    hook.event
                ));
            }
            if !hook_events.insert(&hook.event) {
                errors.push(format!("duplicate hook event '{}' detected", hook.event));
            }
            if hook.timeout_ms == 0 {
                errors.push(format!(
                    "hook timeout for event '{}' must be > 0",
                    hook.event
                ));
            }
            if hook.timeout_ms > 300_000 {
                errors.push(format!(
                    "hook timeout for event '{}' exceeds maximum (300000ms)",
                    hook.event
                ));
            }
        }

        errors
    }

    pub fn check_permissions(manifest: &ExtensionManifest) -> Result<(), String> {
        if manifest
            .permissions
            .contains(&ExtensionPermission::NetworkAccess)
            && manifest
                .permissions
                .contains(&ExtensionPermission::StorageAccess)
        {
            return Err(
                "extensions requiring both NetworkAccess and StorageAccess need admin approval"
                    .into(),
            );
        }
        Ok(())
    }

    pub fn lint_manifest(manifest: &ExtensionManifest) -> Vec<String> {
        let mut warnings = Vec::new();

        if manifest.description.len() < 10 {
            warnings
                .push("description is very short (< 10 chars), consider adding more detail".into());
        }

        if manifest.hooks.is_empty() {
            warnings.push("no hooks defined — extension will not react to any events".into());
        }

        for hook in &manifest.hooks {
            if hook.timeout_ms < 1000 {
                warnings.push(format!(
                    "hook '{}' has a very low timeout ({}ms), may timeout prematurely",
                    hook.event, hook.timeout_ms
                ));
            }
        }

        warnings
    }

    fn is_valid_semver(version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return false;
        }
        parts.iter().all(|p| p.parse::<u32>().is_ok())
    }

    fn is_valid_spdx(license: &str) -> bool {
        let known = [
            "MIT",
            "Apache-2.0",
            "GPL-3.0-or-later",
            "BSD-2-Clause",
            "BSD-3-Clause",
            "ISC",
            "0BSD",
            "AGPL-3.0-or-later",
            "LGPL-3.0-or-later",
            "MPL-2.0",
            "Unlicense",
            "CC0-1.0",
        ];
        known.contains(&license)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_manifest() -> ExtensionManifest {
        ExtensionManifest {
            name: "test-ext".into(),
            version: "1.0.0".into(),
            description: "A test extension for testing".into(),
            author: "testauthor".into(),
            license: "MIT".into(),
            permissions: vec![ExtensionPermission::ReadRepos],
            hooks: vec![ExtensionHook {
                event: "pipeline.step".into(),
                handler: "on_pipeline_step".into(),
                timeout_ms: 5000,
            }],
            entrypoint: "extension.wasm".into(),
        }
    }

    #[test]
    fn test_validate_valid_manifest() {
        let m = valid_manifest();
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
    }

    #[test]
    fn test_validate_empty_name() {
        let mut m = valid_manifest();
        m.name = String::new();
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(errors.iter().any(|e| e.contains("name is required")));
    }

    #[test]
    fn test_validate_invalid_name_chars() {
        let mut m = valid_manifest();
        m.name = "bad name!".into();
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(errors.iter().any(|e| e.contains("alphanumeric")));
    }

    #[test]
    fn test_validate_name_too_long() {
        let mut m = valid_manifest();
        m.name = "a".repeat(129);
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(errors.iter().any(|e| e.contains("128")));
    }

    #[test]
    fn test_validate_empty_version() {
        let mut m = valid_manifest();
        m.version = String::new();
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(errors.iter().any(|e| e.contains("version is required")));
    }

    #[test]
    fn test_validate_invalid_version() {
        let mut m = valid_manifest();
        m.version = "not-semver".into();
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(errors.iter().any(|e| e.contains("semver")));
    }

    #[test]
    fn test_validate_version_parts() {
        let mut m = valid_manifest();
        m.version = "1.0".into();
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(errors.iter().any(|e| e.contains("semver")));

        m.version = "1.0.0.0".into();
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(errors.iter().any(|e| e.contains("semver")));
    }

    #[test]
    fn test_validate_empty_description() {
        let mut m = valid_manifest();
        m.description = String::new();
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(errors.iter().any(|e| e.contains("description is required")));
    }

    #[test]
    fn test_validate_description_too_long() {
        let mut m = valid_manifest();
        m.description = "x".repeat(2049);
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(errors.iter().any(|e| e.contains("2048")));
    }

    #[test]
    fn test_validate_empty_author() {
        let mut m = valid_manifest();
        m.author = String::new();
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(errors.iter().any(|e| e.contains("author is required")));
    }

    #[test]
    fn test_validate_empty_license() {
        let mut m = valid_manifest();
        m.license = String::new();
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(errors.iter().any(|e| e.contains("license is required")));
    }

    #[test]
    fn test_validate_invalid_license() {
        let mut m = valid_manifest();
        m.license = "FAKE-LICENSE".into();
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(errors.iter().any(|e| e.contains("SPDX")));
    }

    #[test]
    fn test_validate_no_permissions() {
        let mut m = valid_manifest();
        m.permissions = Vec::new();
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(errors.iter().any(|e| e.contains("at least one permission")));
    }

    #[test]
    fn test_validate_empty_entrypoint() {
        let mut m = valid_manifest();
        m.entrypoint = String::new();
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(errors.iter().any(|e| e.contains("entrypoint is required")));
    }

    #[test]
    fn test_validate_entrypoint_extension() {
        let mut m = valid_manifest();
        m.entrypoint = "plugin.py".into();
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(errors.iter().any(|e| e.contains(".wasm or .js")));

        m.entrypoint = "ok.wasm".into();
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(!errors.iter().any(|e| e.contains("entrypoint")));

        m.entrypoint = "ok.js".into();
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(!errors.iter().any(|e| e.contains("entrypoint")));
    }

    #[test]
    fn test_validate_hook_empty_event() {
        let mut m = valid_manifest();
        m.hooks[0].event = String::new();
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(
            errors
                .iter()
                .any(|e| e.contains("hook event must not be empty"))
        );
    }

    #[test]
    fn test_validate_hook_empty_handler() {
        let mut m = valid_manifest();
        m.hooks[0].handler = String::new();
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(errors.iter().any(|e| e.contains("hook handler")));
    }

    #[test]
    fn test_validate_duplicate_hook_events() {
        let mut m = valid_manifest();
        m.hooks.push(ExtensionHook {
            event: "pipeline.step".into(),
            handler: "another_handler".into(),
            timeout_ms: 3000,
        });
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(errors.iter().any(|e| e.contains("duplicate hook")));
    }

    #[test]
    fn test_validate_hook_zero_timeout() {
        let mut m = valid_manifest();
        m.hooks[0].timeout_ms = 0;
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(errors.iter().any(|e| e.contains("timeout")));
    }

    #[test]
    fn test_validate_hook_timeout_exceeds_max() {
        let mut m = valid_manifest();
        m.hooks[0].timeout_ms = 300_001;
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(errors.iter().any(|e| e.contains("maximum")));
    }

    #[test]
    fn test_check_permissions_ok() {
        let m = valid_manifest();
        assert!(ExtensionSandbox::check_permissions(&m).is_ok());
    }

    #[test]
    fn test_check_permissions_network_and_storage_blocked() {
        let mut m = valid_manifest();
        m.permissions = vec![
            ExtensionPermission::NetworkAccess,
            ExtensionPermission::StorageAccess,
        ];
        let result = ExtensionSandbox::check_permissions(&m);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("admin approval"));
    }

    #[test]
    fn test_lint_short_description() {
        let mut m = valid_manifest();
        m.description = "short".into();
        let warnings = ExtensionSandbox::lint_manifest(&m);
        assert!(warnings.iter().any(|w| w.contains("very short")));
    }

    #[test]
    fn test_lint_no_hooks() {
        let mut m = valid_manifest();
        m.hooks = Vec::new();
        let warnings = ExtensionSandbox::lint_manifest(&m);
        assert!(warnings.iter().any(|w| w.contains("no hooks")));
    }

    #[test]
    fn test_lint_low_timeout() {
        let mut m = valid_manifest();
        m.hooks[0].timeout_ms = 500;
        let warnings = ExtensionSandbox::lint_manifest(&m);
        assert!(warnings.iter().any(|w| w.contains("very low timeout")));
    }

    #[test]
    fn test_manifest_serialization() {
        let m = valid_manifest();
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.contains("\"name\":\"test-ext\""));
        assert!(json.contains("\"version\":\"1.0.0\""));
        assert!(json.contains("ReadRepos"));
    }

    #[test]
    fn test_manifest_deserialization() {
        let json = r#"{
            "name": "my-ext",
            "version": "0.1.0",
            "description": "My extension",
            "author": "dev",
            "license": "Apache-2.0",
            "permissions": ["ReadIssues"],
            "hooks": [{"event": "issue.created", "handler": "on_issue", "timeout_ms": 10000}],
            "entrypoint": "ext.js"
        }"#;
        let m: ExtensionManifest = serde_json::from_str(json).unwrap();
        assert_eq!(m.name, "my-ext");
        assert_eq!(m.version, "0.1.0");
        assert_eq!(m.permissions.len(), 1);
        assert_eq!(m.hooks.len(), 1);
    }

    #[test]
    fn test_is_valid_semver() {
        assert!(ExtensionSandbox::is_valid_semver("1.0.0"));
        assert!(ExtensionSandbox::is_valid_semver("0.1.0"));
        assert!(ExtensionSandbox::is_valid_semver("10.20.30"));
        assert!(!ExtensionSandbox::is_valid_semver("1.0"));
        assert!(!ExtensionSandbox::is_valid_semver("v1.0.0"));
        assert!(!ExtensionSandbox::is_valid_semver("1.0.0-beta"));
    }

    #[test]
    fn test_is_valid_spdx() {
        assert!(ExtensionSandbox::is_valid_spdx("MIT"));
        assert!(ExtensionSandbox::is_valid_spdx("Apache-2.0"));
        assert!(ExtensionSandbox::is_valid_spdx("GPL-3.0-or-later"));
        assert!(!ExtensionSandbox::is_valid_spdx("INVALID"));
    }

    #[test]
    fn test_marketplace_routes_compile() {
        let router = marketplace_routes();
        let _ = router;
    }

    #[test]
    fn test_extension_summary_serialization() {
        let s = ExtensionSummary {
            name: "ext".into(),
            version: "1.0.0".into(),
            description: "desc".into(),
            author: "dev".into(),
            installed: true,
        };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("\"installed\":true"));
    }

    #[test]
    fn test_verify_response_serialization() {
        let r = VerifyResponse {
            valid: true,
            errors: vec![],
            warnings: vec!["minor warning".into()],
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("\"valid\":true"));
    }

    #[test]
    fn test_multiple_permissions_validation() {
        let mut m = valid_manifest();
        m.permissions = vec![
            ExtensionPermission::ReadRepos,
            ExtensionPermission::WriteRepos,
            ExtensionPermission::ReadIssues,
            ExtensionPermission::WriteIssues,
        ];
        let errors = ExtensionSandbox::validate_manifest(&m);
        assert!(!errors.iter().any(|e| e.contains("permission")));
    }

    #[test]
    fn test_extension_permission_serialization() {
        assert_eq!(
            serde_json::to_string(&ExtensionPermission::NetworkAccess).unwrap(),
            "\"NetworkAccess\""
        );
    }
}
