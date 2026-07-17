#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::error::CoreError;
use axum::{
    Json,
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub id: String,
    pub repo_id: String,
    pub name: String,
    pub version: String,
    pub ecosystem: String,
    pub latest_version: Option<String>,
    pub is_outdated: bool,
    pub has_vulnerabilities: bool,
    pub vulnerability_count: i32,
    pub last_scanned_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityAdvisory {
    pub id: String,
    pub dependency_id: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub url: Option<String>,
    pub patched_version: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DependencyListResponse {
    pub dependencies: Vec<Dependency>,
    pub total: usize,
    pub outdated_count: usize,
    pub vulnerable_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct VulnerabilityListResponse {
    pub vulnerabilities: Vec<VulnerabilityAdvisory>,
    pub total: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScanDependenciesRequest {
    pub ecosystem: Option<String>,
}

async fn get_repo_id(
    state: &AppState,
    owner: &str,
    name: &str,
) -> Result<Uuid, CoreError> {
    let result = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM repositories WHERE owner_id = (SELECT id FROM users WHERE username = $1) AND name = $2",
    )
    .bind(owner)
    .bind(name)
    .fetch_optional(state.db.pool())
    .await;

    match result {
        Ok(Some(id)) => Ok(id),
        Ok(None) => Err(CoreError::NotFound("repository not found".into())),
        Err(e) => Err(CoreError::Internal(format!("database error: {e}"))),
    }
}

fn scan_repo_dependencies(repo_path: &std::path::Path) -> Result<Vec<(String, String, String)>, String> {
    let repo = gix::open(repo_path).map_err(|e| format!("git error: {e}"))?;

    let head_id = repo.head_id().map_err(|e| format!("git error: {e}"))?;
    let commit = head_id
        .object()
        .map_err(|e| format!("git error: {e}"))?
        .try_into_commit()
        .map_err(|e| format!("git error: {e}"))?;

    let tree = commit
        .tree_id()
        .map_err(|e| format!("git error: {e}"))?
        .object()
        .map_err(|e| format!("git error: {e}"))?
        .try_into_tree()
        .map_err(|e| format!("git error: {e}"))?;

    let mut dependencies: Vec<(String, String, String)> = Vec::new();

    fn walk_tree_for_deps(
        tree: &gix::Tree<'_>,
        prefix: &str,
        deps: &mut Vec<(String, String, String)>,
    ) {
        for entry_result in tree.iter() {
            let entry = match entry_result {
                Ok(e) => e,
                Err(_) => continue,
            };
            let mode = entry.mode();
            if mode.is_tree() {
                if let Some(subtree) = entry.object().ok().and_then(|o| o.try_into_tree().ok()) {
                    let sub_prefix = if prefix.is_empty() {
                        entry.filename().to_string()
                    } else {
                        format!("{}/{}", prefix, entry.filename())
                    };
                    walk_tree_for_deps(&subtree, &sub_prefix, deps);
                }
            } else if mode.is_blob() {
                let full_path = if prefix.is_empty() {
                    entry.filename().to_string()
                } else {
                    format!("{}/{}", prefix, entry.filename())
                };

                let ecosystem = match full_path.as_str() {
                    p if p.ends_with("Cargo.toml") => "cargo",
                    p if p.ends_with("package.json") => "npm",
                    p if p.ends_with("requirements.txt") => "pip",
                    p if p.ends_with("go.mod") => "go",
                    p if p.ends_with("pom.xml") => "maven",
                    p if p.ends_with("build.gradle") => "gradle",
                    _ => continue,
                };

                if let Some(blob) = entry.object().ok().and_then(|o| o.try_into_blob().ok())
                    && let Ok(content) = std::str::from_utf8(blob.data.as_ref())
                {
                    for line in content.lines() {
                        let line = line.trim();
                        if ecosystem == "cargo" && line.starts_with("name") {
                            if let Some(name) = line.split('=').nth(1).map(|s| s.trim().trim_matches('"')) {
                                deps.push((name.to_string(), "unknown".to_string(), ecosystem.to_string()));
                            }
                        } else if ecosystem == "npm" && line.contains("\"name\"")
                            && let Some(name) = line.split(':').nth(1).map(|s| s.trim().trim_matches('"').trim_matches(','))
                                && !name.starts_with('@') && name != "name" {
                                    deps.push((name.to_string(), "unknown".to_string(), ecosystem.to_string()));
                                }
                    }
                }
            }
        }
    }

    walk_tree_for_deps(&tree, "", &mut dependencies);
    Ok(dependencies)
}

pub fn dependency_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/dependencies/scan",
            post(scan_dependencies),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/dependencies",
            get(list_dependencies),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/dependencies/vulnerabilities",
            get(list_vulnerabilities),
        )
}

pub async fn scan_dependencies(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Json(req): Json<ScanDependenciesRequest>,
) -> Response {
    let repo_id = match get_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(CoreError::NotFound(msg)) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound(msg).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(e.error_response())).into_response();
        }
    };

    let repo_path = state.git_service.repo_path(&owner, &name);
    if !repo_path.join("HEAD").exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response();
    }

    let dependencies = match scan_repo_dependencies(&repo_path) {
        Ok(deps) => deps,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Internal(e).error_response()),
            )
                .into_response();
        }
    };

    let mut inserted = 0;
    for (dep_name, version, ecosystem) in dependencies {
        let filter_ecosystem = req.ecosystem.as_deref().unwrap_or(&ecosystem);
        if req.ecosystem.is_some() && filter_ecosystem != ecosystem {
            continue;
        }

        let result = sqlx::query(
            r#"INSERT INTO dependencies (repo_id, name, version, ecosystem, last_scanned_at)
               VALUES ($1, $2, $3, $4, NOW())
               ON CONFLICT (repo_id, name, ecosystem)
               DO UPDATE SET version = $3, last_scanned_at = NOW()
               RETURNING id"#,
        )
        .bind(repo_id)
        .bind(&dep_name)
        .bind(&version)
        .bind(&ecosystem)
        .fetch_optional(state.db.pool())
        .await;

        if result.is_ok() {
            inserted += 1;
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "dependencies_scanned": inserted,
            "ecosystem": req.ecosystem.unwrap_or_else(|| "all".to_string()),
        })),
    )
        .into_response()
}

pub async fn list_dependencies(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> Response {
    let repo_id = match get_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::OK,
                Json(DependencyListResponse {
                    dependencies: Vec::new(),
                    total: 0,
                    outdated_count: 0,
                    vulnerable_count: 0,
                }),
            )
                .into_response();
        }
    };

    let result = sqlx::query_as::<_, (String, String, String, String, String, Option<String>, bool, bool, i32, Option<String>, String)>(
        r#"SELECT id::text, repo_id::text, name, version, ecosystem, latest_version,
                  is_outdated, has_vulnerabilities, vulnerability_count, last_scanned_at::text, created_at::text
           FROM dependencies WHERE repo_id = $1 ORDER BY name ASC"#,
    )
    .bind(repo_id)
    .fetch_all(state.db.pool())
    .await;

    match result {
        Ok(rows) => {
            let dependencies: Vec<Dependency> = rows
                .into_iter()
                .map(|r| Dependency {
                    id: r.0,
                    repo_id: r.1,
                    name: r.2,
                    version: r.3,
                    ecosystem: r.4,
                    latest_version: r.5,
                    is_outdated: r.6,
                    has_vulnerabilities: r.7,
                    vulnerability_count: r.8,
                    last_scanned_at: r.9,
                    created_at: r.10,
                })
                .collect();
            let total = dependencies.len();
            let outdated_count = dependencies.iter().filter(|d| d.is_outdated).count();
            let vulnerable_count = dependencies.iter().filter(|d| d.has_vulnerabilities).count();
            (
                StatusCode::OK,
                Json(DependencyListResponse {
                    dependencies,
                    total,
                    outdated_count,
                    vulnerable_count,
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Internal(format!("database error: {e}")).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_vulnerabilities(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> Response {
    let repo_id = match get_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::OK,
                Json(VulnerabilityListResponse {
                    vulnerabilities: Vec::new(),
                    total: 0,
                }),
            )
                .into_response();
        }
    };

    let result = sqlx::query_as::<_, (String, String, String, String, String, Option<String>, Option<String>, String)>(
        r#"SELECT va.id::text, va.dependency_id::text, va.severity, va.title, va.description,
                  va.url, va.patched_version, va.created_at::text
           FROM vulnerability_advisories va
           JOIN dependencies d ON va.dependency_id = d.id
           WHERE d.repo_id = $1
           ORDER BY va.created_at DESC LIMIT 100"#,
    )
    .bind(repo_id)
    .fetch_all(state.db.pool())
    .await;

    match result {
        Ok(rows) => {
            let vulnerabilities: Vec<VulnerabilityAdvisory> = rows
                .into_iter()
                .map(|r| VulnerabilityAdvisory {
                    id: r.0,
                    dependency_id: r.1,
                    severity: r.2,
                    title: r.3,
                    description: r.4,
                    url: r.5,
                    patched_version: r.6,
                    created_at: r.7,
                })
                .collect();
            let total = vulnerabilities.len();
            (
                StatusCode::OK,
                Json(VulnerabilityListResponse {
                    vulnerabilities,
                    total,
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Internal(format!("database error: {e}")).error_response()),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_serialization() {
        let dep = Dependency {
            id: "dep-1".into(),
            repo_id: "repo-1".into(),
            name: "serde".into(),
            version: "1.0.0".into(),
            ecosystem: "cargo".into(),
            latest_version: Some("1.0.1".into()),
            is_outdated: true,
            has_vulnerabilities: false,
            vulnerability_count: 0,
            last_scanned_at: Some("2025-01-01T00:00:00Z".into()),
            created_at: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&dep).unwrap();
        assert!(json.contains("serde"));
        assert!(json.contains("\"is_outdated\":true"));
    }

    #[test]
    fn test_vulnerability_advisory_serialization() {
        let vuln = VulnerabilityAdvisory {
            id: "vuln-1".into(),
            dependency_id: "dep-1".into(),
            severity: "critical".into(),
            title: "Remote Code Execution".into(),
            description: "Critical RCE vulnerability".into(),
            url: Some("https://example.com/advisory".into()),
            patched_version: Some("1.0.1".into()),
            created_at: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&vuln).unwrap();
        assert!(json.contains("Remote Code Execution"));
        assert!(json.contains("\"severity\":\"critical\""));
    }

    #[test]
    fn test_dependency_list_response_serialization() {
        let resp = DependencyListResponse {
            dependencies: Vec::new(),
            total: 0,
            outdated_count: 0,
            vulnerable_count: 0,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"total\":0"));
        assert!(json.contains("\"outdated_count\":0"));
    }

    #[test]
    fn test_vulnerability_list_response_serialization() {
        let resp = VulnerabilityListResponse {
            vulnerabilities: Vec::new(),
            total: 0,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"total\":0"));
    }

    #[test]
    fn test_scan_dependencies_request_deserialization() {
        let json = r#"{"ecosystem": "cargo"}"#;
        let req: ScanDependenciesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.ecosystem, Some("cargo".to_string()));
    }

    #[test]
    fn test_scan_dependencies_request_no_ecosystem() {
        let json = r#"{}"#;
        let req: ScanDependenciesRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.ecosystem, None);
    }
}
