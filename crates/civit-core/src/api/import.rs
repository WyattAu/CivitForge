#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_permission};
use crate::error::CoreError;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use civit_shared::permissions::{Action, Resource};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct GitHubImportRequest {
    pub github_url: String,
    pub token: Option<String>,
    pub owner_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitLabImportRequest {
    pub gitlab_url: String,
    pub token: Option<String>,
    pub owner_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UrlImportRequest {
    pub url: String,
    pub token: Option<String>,
    pub owner_id: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ImportResponse {
    pub status: String,
    pub message: String,
    pub repo_id: Option<String>,
}

fn parse_github_url(url: &str) -> Option<(&str, &str)> {
    let url = url.trim_end_matches('/');
    let url = url.trim_end_matches(".git");
    // https://github.com/owner/repo or git@github.com:owner/repo.git
    if let Some(rest) = url.strip_prefix("https://github.com/") {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return Some((parts[0], parts[1]));
        }
    }
    if let Some(rest) = url.strip_prefix("git@github.com:") {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return Some((parts[0], parts[1]));
        }
    }
    None
}

fn parse_gitlab_url(url: &str) -> Option<(&str, &str)> {
    let url = url.trim_end_matches('/');
    let url = url.trim_end_matches(".git");
    // https://gitlab.com/owner/repo or git@gitlab.com:owner/repo.git
    if let Some(rest) = url.strip_prefix("https://gitlab.com/") {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return Some((parts[0], parts[1]));
        }
    }
    if let Some(rest) = url.strip_prefix("git@gitlab.com:") {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return Some((parts[0], parts[1]));
        }
    }
    None
}

#[derive(Debug, Serialize)]
struct GitHubRepoMeta {
    name: String,
    description: String,
    default_branch: String,
    visibility: String,
}

async fn fetch_github_metadata(
    owner: &str,
    repo: &str,
    token: Option<&str>,
) -> Result<GitHubRepoMeta, CoreError> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{owner}/{repo}");
    let mut req = client
        .get(&url)
        .header("User-Agent", "CivitForge/1.0")
        .header("Accept", "application/vnd.github+json");
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    let resp = req
        .send()
        .await
        .map_err(|e| CoreError::Internal(format!("GitHub API request failed: {e}")))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_else(|_| "unknown error".into());
        return Err(CoreError::Internal(format!(
            "GitHub API returned {status}: {body}"
        )));
    }
    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| CoreError::Internal(format!("failed to parse GitHub response: {e}")))?;
    let name = json["name"].as_str().unwrap_or(repo).to_string();
    let description = json["description"].as_str().unwrap_or("").to_string();
    let default_branch = json["default_branch"]
        .as_str()
        .unwrap_or("main")
        .to_string();
    let visibility = if json["private"].as_bool().unwrap_or(false) {
        "private"
    } else {
        "public"
    }
    .to_string();
    Ok(GitHubRepoMeta {
        name,
        description,
        default_branch,
        visibility,
    })
}

#[derive(Debug, Serialize)]
struct GitLabRepoMeta {
    name: String,
    description: String,
    default_branch: String,
    visibility: String,
}

async fn fetch_gitlab_metadata(
    owner: &str,
    repo: &str,
    token: Option<&str>,
) -> Result<GitLabRepoMeta, CoreError> {
    let client = reqwest::Client::new();
    let project_path = format!("{owner}/{repo}");
    let encoded = urlencoding::encode(&project_path);
    let url = format!("https://gitlab.com/api/v4/projects/{encoded}");
    let mut req = client.get(&url).header("User-Agent", "CivitForge/1.0");
    if let Some(t) = token {
        req = req.header("PRIVATE-TOKEN", t);
    }
    let resp = req
        .send()
        .await
        .map_err(|e| CoreError::Internal(format!("GitLab API request failed: {e}")))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_else(|_| "unknown error".into());
        return Err(CoreError::Internal(format!(
            "GitLab API returned {status}: {body}"
        )));
    }
    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| CoreError::Internal(format!("failed to parse GitLab response: {e}")))?;
    let name = json["name"].as_str().unwrap_or(repo).to_string();
    let description = json["description"].as_str().unwrap_or("").to_string();
    let default_branch = json["default_branch"]
        .as_str()
        .unwrap_or("main")
        .to_string();
    let visibility = match json["visibility"].as_str().unwrap_or("private") {
        "public" => "public",
        "internal" => "internal",
        _ => "private",
    }
    .to_string();
    Ok(GitLabRepoMeta {
        name,
        description,
        default_branch,
        visibility,
    })
}

#[allow(dead_code)]
fn git_err_to_core(e: impl std::fmt::Display) -> CoreError {
    CoreError::Git(e.to_string())
}

pub async fn import_github(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<GitHubImportRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_permission(
        &state,
        &auth,
        Resource::Repository,
        Action::Create,
        None,
        None,
        None,
    )
    .await
    {
        return rejection.into_response();
    }

    let (gh_owner, gh_repo) = match parse_github_url(&req.github_url) {
        Some(parts) => parts,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(
                    CoreError::BadRequest(
                        "invalid GitHub URL, expected https://github.com/owner/repo".into(),
                    )
                    .error_response(),
                ),
            )
                .into_response();
        }
    };
    let gh_owner = gh_owner.to_string();
    let gh_repo = gh_repo.to_string();

    let meta = match fetch_github_metadata(&gh_owner, &gh_repo, req.token.as_deref()).await {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(CoreError::Internal(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let owner_uuid = match req.owner_id {
        Some(ref id_str) => match Uuid::parse_str(id_str) {
            Ok(id) => id,
            Err(_) => Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil()),
        },
        None => Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil()),
    };

    match state
        .db
        .create_repo(
            &meta.name,
            &meta.description,
            owner_uuid,
            None,
            &meta.visibility,
            &meta.default_branch,
        )
        .await
    {
        Ok(repo) => {
            let storage_path = state.config.storage_path.clone();
            let owner_name = state
                .db
                .get_user_by_id(owner_uuid)
                .await
                .map(|u| u.username)
                .unwrap_or_else(|_| owner_uuid.to_string());
            let repo_name = meta.name.clone();
            let repo_path = std::path::Path::new(&storage_path)
                .join(&owner_name)
                .join(&repo_name);

            let clone_url = if let Some(ref token) = req.token {
                format!("https://{token}@github.com/{gh_owner}/{gh_repo}.git")
            } else {
                format!("https://github.com/{gh_owner}/{gh_repo}.git")
            };

            let token_clone = req.token.clone();
            let db_clone = state.db.clone();
            let repo_id = repo.id;
            let gh_owner_clone = gh_owner.clone();
            let gh_repo_clone = gh_repo.clone();

            tokio::spawn(async move {
                if let Err(e) = tokio::fs::create_dir_all(&repo_path).await {
                    eprintln!("[import_github] failed to create dir: {e}");
                    return;
                }
                if let Err(e) = tokio::process::Command::new("git")
                    .args(["clone", "--bare", &clone_url, &repo_path.to_string_lossy()])
                    .output()
                    .await
                {
                    eprintln!("[import_github] git clone failed: {e}");
                    return;
                }

                // Fetch issues
                let client = reqwest::Client::new();
                let mut page = 1u32;
                loop {
                    let url = format!(
                        "https://api.github.com/repos/{gh_owner}/{gh_repo}/issues?state=all&per_page=100&page={page}"
                    );
                    let mut req_builder = client
                        .get(&url)
                        .header("User-Agent", "CivitForge/1.0")
                        .header("Accept", "application/vnd.github+json");
                    if let Some(ref token) = token_clone {
                        req_builder = req_builder.bearer_auth(token);
                    }
                    let resp = match req_builder.send().await {
                        Ok(r) => r,
                        Err(_) => break,
                    };
                    if !resp.status().is_success() {
                        break;
                    }
                    let issues: Vec<serde_json::Value> = match resp.json().await {
                        Ok(v) => v,
                        Err(_) => break,
                    };
                    if issues.is_empty() {
                        break;
                    }
                    for issue in &issues {
                        if issue["pull_request"].is_object() {
                            continue;
                        }
                        let title = issue["title"].as_str().unwrap_or("Untitled");
                        let body = issue["body"].as_str().unwrap_or("");
                        let _ = db_clone
                            .create_issue(repo_id, title, body, owner_uuid)
                            .await;
                    }
                    page += 1;
                }

                // Fetch PRs
                let mut page = 1u32;
                loop {
                    let url = format!(
                        "https://api.github.com/repos/{gh_owner}/{gh_repo}/pulls?state=all&per_page=100&page={page}"
                    );
                    let mut req_builder = client
                        .get(&url)
                        .header("User-Agent", "CivitForge/1.0")
                        .header("Accept", "application/vnd.github+json");
                    if let Some(ref token) = token_clone {
                        req_builder = req_builder.bearer_auth(token);
                    }
                    let resp = match req_builder.send().await {
                        Ok(r) => r,
                        Err(_) => break,
                    };
                    if !resp.status().is_success() {
                        break;
                    }
                    let prs: Vec<serde_json::Value> = match resp.json().await {
                        Ok(v) => v,
                        Err(_) => break,
                    };
                    if prs.is_empty() {
                        break;
                    }
                    for pr in &prs {
                        let title = pr["title"].as_str().unwrap_or("Untitled");
                        let body = pr["body"].as_str().unwrap_or("");
                        let source = pr["head"]["ref"].as_str().unwrap_or("feature");
                        let target = pr["base"]["ref"].as_str().unwrap_or("main");
                        let draft = pr["draft"].as_bool().unwrap_or(false);
                        let _ = db_clone
                            .create_pr(repo_id, title, body, owner_uuid, source, target, draft)
                            .await;
                    }
                    page += 1;
                }
            });

            (
                StatusCode::ACCEPTED,
                Json(ImportResponse {
                    status: "importing".into(),
                    message: format!(
                        "Import started for {gh_owner_clone}/{gh_repo_clone}. Git clone and data import running in background."
                    ),
                    repo_id: Some(repo.id.to_string()),
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn import_gitlab(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<GitLabImportRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_permission(
        &state,
        &auth,
        Resource::Repository,
        Action::Create,
        None,
        None,
        None,
    )
    .await
    {
        return rejection.into_response();
    }

    let (gl_owner, gl_repo) = match parse_gitlab_url(&req.gitlab_url) {
        Some(parts) => parts,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(
                    CoreError::BadRequest(
                        "invalid GitLab URL, expected https://gitlab.com/owner/repo".into(),
                    )
                    .error_response(),
                ),
            )
                .into_response();
        }
    };
    let gl_owner = gl_owner.to_string();
    let gl_repo = gl_repo.to_string();

    let meta = match fetch_gitlab_metadata(&gl_owner, &gl_repo, req.token.as_deref()).await {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(CoreError::Internal(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let owner_uuid = match req.owner_id {
        Some(ref id_str) => match Uuid::parse_str(id_str) {
            Ok(id) => id,
            Err(_) => Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil()),
        },
        None => Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil()),
    };

    match state
        .db
        .create_repo(
            &meta.name,
            &meta.description,
            owner_uuid,
            None,
            &meta.visibility,
            &meta.default_branch,
        )
        .await
    {
        Ok(repo) => {
            let storage_path = state.config.storage_path.clone();
            let owner_name = state
                .db
                .get_user_by_id(owner_uuid)
                .await
                .map(|u| u.username)
                .unwrap_or_else(|_| owner_uuid.to_string());
            let repo_name = meta.name.clone();
            let repo_path = std::path::Path::new(&storage_path)
                .join(&owner_name)
                .join(&repo_name);

            let clone_url = if let Some(ref token) = req.token {
                format!("https://oauth2:{token}@gitlab.com/{gl_owner}/{gl_repo}.git")
            } else {
                format!("https://gitlab.com/{gl_owner}/{gl_repo}.git")
            };

            let token_clone = req.token.clone();
            let db_clone = state.db.clone();
            let repo_id = repo.id;
            let gl_owner_clone = gl_owner.clone();
            let gl_repo_clone = gl_repo.clone();

            tokio::spawn(async move {
                if let Err(e) = tokio::fs::create_dir_all(&repo_path).await {
                    eprintln!("[import_gitlab] failed to create dir: {e}");
                    return;
                }
                if let Err(e) = tokio::process::Command::new("git")
                    .args(["clone", "--bare", &clone_url, &repo_path.to_string_lossy()])
                    .output()
                    .await
                {
                    eprintln!("[import_gitlab] git clone failed: {e}");
                    return;
                }

                // Fetch issues from GitLab API
                let client = reqwest::Client::new();
                let project_path = format!("{gl_owner}/{gl_repo}");
                let encoded = urlencoding::encode(&project_path);
                let mut page = 1u32;
                loop {
                    let url = format!(
                        "https://gitlab.com/api/v4/projects/{encoded}/issues?state=all&per_page=100&page={page}"
                    );
                    let mut req_builder = client.get(&url).header("User-Agent", "CivitForge/1.0");
                    if let Some(ref token) = token_clone {
                        req_builder = req_builder.header("PRIVATE-TOKEN", token);
                    }
                    let resp = match req_builder.send().await {
                        Ok(r) => r,
                        Err(_) => break,
                    };
                    if !resp.status().is_success() {
                        break;
                    }
                    let issues: Vec<serde_json::Value> = match resp.json().await {
                        Ok(v) => v,
                        Err(_) => break,
                    };
                    if issues.is_empty() {
                        break;
                    }
                    for issue in &issues {
                        let title = issue["title"].as_str().unwrap_or("Untitled");
                        let body = issue["description"].as_str().unwrap_or("");
                        let _ = db_clone
                            .create_issue(repo_id, title, body, owner_uuid)
                            .await;
                    }
                    page += 1;
                }

                // Fetch merge requests from GitLab API
                let mut page = 1u32;
                loop {
                    let url = format!(
                        "https://gitlab.com/api/v4/projects/{encoded}/merge_requests?state=all&per_page=100&page={page}"
                    );
                    let mut req_builder = client.get(&url).header("User-Agent", "CivitForge/1.0");
                    if let Some(ref token) = token_clone {
                        req_builder = req_builder.header("PRIVATE-TOKEN", token);
                    }
                    let resp = match req_builder.send().await {
                        Ok(r) => r,
                        Err(_) => break,
                    };
                    if !resp.status().is_success() {
                        break;
                    }
                    let mrs: Vec<serde_json::Value> = match resp.json().await {
                        Ok(v) => v,
                        Err(_) => break,
                    };
                    if mrs.is_empty() {
                        break;
                    }
                    for mr in &mrs {
                        let title = mr["title"].as_str().unwrap_or("Untitled");
                        let body = mr["description"].as_str().unwrap_or("");
                        let source = mr["source_branch"].as_str().unwrap_or("feature");
                        let target = mr["target_branch"].as_str().unwrap_or("main");
                        let draft = mr["draft"].as_bool().unwrap_or(false);
                        let _ = db_clone
                            .create_pr(repo_id, title, body, owner_uuid, source, target, draft)
                            .await;
                    }
                    page += 1;
                }
            });

            (
                StatusCode::ACCEPTED,
                Json(ImportResponse {
                    status: "importing".into(),
                    message: format!(
                        "Import started for {gl_owner_clone}/{gl_repo_clone}. Git clone and data import running in background."
                    ),
                    repo_id: Some(repo.id.to_string()),
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

fn parse_git_url(url: &str) -> Option<String> {
    let url = url.trim_end_matches('/');
    let url = url.trim_end_matches(".git");

    // HTTPS: https://example.com/owner/repo
    if let Some(rest) = url.strip_prefix("https://") {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() >= 3 && !parts[0].is_empty() {
            return parts.last().map(|s| s.to_string());
        }
    }
    // HTTP: http://example.com/owner/repo
    if let Some(rest) = url.strip_prefix("http://") {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() >= 3 && !parts[0].is_empty() {
            return parts.last().map(|s| s.to_string());
        }
    }
    // SSH: git@host:owner/repo
    if let Some(rest) = url.strip_prefix("git@") {
        if let Some(colon_pos) = rest.find(':') {
            let path_part = &rest[colon_pos + 1..];
            let parts: Vec<&str> = path_part.split('/').collect();
            if parts.len() >= 2 {
                return parts.last().map(|s| s.to_string());
            }
        }
    }
    None
}

pub async fn import_url(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UrlImportRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_permission(
        &state,
        &auth,
        Resource::Repository,
        Action::Create,
        None,
        None,
        None,
    )
    .await
    {
        return rejection.into_response();
    }

    let repo_name = match req.name {
        Some(ref n) if !n.is_empty() => n.clone(),
        _ => match parse_git_url(&req.url) {
            Some(name) => name,
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(
                        CoreError::BadRequest(
                            "could not extract repo name from URL, provide a 'name' field".into(),
                        )
                        .error_response(),
                    ),
                )
                    .into_response();
            }
        },
    };

    if !repo_name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                CoreError::BadRequest(
                    "repo name must be alphanumeric with hyphens or underscores".into(),
                )
                .error_response(),
            ),
        )
            .into_response();
    }

    let owner_uuid = match req.owner_id {
        Some(ref id_str) => match Uuid::parse_str(id_str) {
            Ok(id) => id,
            Err(_) => Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil()),
        },
        None => Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil()),
    };

    match state
        .db
        .create_repo(
            &repo_name,
            &format!("Imported from {}", req.url),
            owner_uuid,
            None,
            "public",
            "main",
        )
        .await
    {
        Ok(repo) => {
            let storage_path = state.config.storage_path.clone();
            let owner_name = state
                .db
                .get_user_by_id(owner_uuid)
                .await
                .map(|u| u.username)
                .unwrap_or_else(|_| owner_uuid.to_string());
            let repo_path = std::path::Path::new(&storage_path)
                .join(&owner_name)
                .join(&repo_name);

            let clone_url = if let Some(ref token) = req.token {
                if req.url.starts_with("https://") {
                    let without_proto = req.url.strip_prefix("https://").unwrap();
                    format!("https://{token}@{without_proto}")
                } else {
                    req.url.clone()
                }
            } else {
                req.url.clone()
            };

            let db_clone = state.db.clone();
            let repo_id = repo.id;
            let url_for_log = req.url.clone();

            tokio::spawn(async move {
                if let Err(e) = tokio::fs::create_dir_all(&repo_path).await {
                    eprintln!("[import_url] failed to create dir: {e}");
                    return;
                }
                let output = tokio::process::Command::new("git")
                    .args([
                        "clone",
                        "--mirror",
                        &clone_url,
                        &repo_path.to_string_lossy(),
                    ])
                    .output()
                    .await;
                match output {
                    Ok(o) if !o.status.success() => {
                        let stderr = String::from_utf8_lossy(&o.stderr);
                        eprintln!("[import_url] git clone failed: {stderr}");
                        let _ = tokio::fs::remove_dir_all(&repo_path).await;
                        let _ = db_clone.delete_repo(repo_id).await;
                    }
                    Err(e) => {
                        eprintln!("[import_url] git clone failed: {e}");
                        let _ = tokio::fs::remove_dir_all(&repo_path).await;
                        let _ = db_clone.delete_repo(repo_id).await;
                    }
                    _ => {}
                }
                eprintln!("[import_url] clone of {url_for_log} completed for repo {repo_id}");
            });

            let message = format!(
                "Import started for {}. Git clone running in background.",
                req.url,
            );
            (
                StatusCode::ACCEPTED,
                Json(ImportResponse {
                    status: "importing".into(),
                    message,
                    repo_id: Some(repo.id.to_string()),
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url_https() {
        let (owner, repo) = parse_github_url("https://github.com/rust-lang/rust").unwrap();
        assert_eq!(owner, "rust-lang");
        assert_eq!(repo, "rust");
    }

    #[test]
    fn test_parse_github_url_with_git_suffix() {
        let (owner, repo) = parse_github_url("https://github.com/rust-lang/rust.git").unwrap();
        assert_eq!(owner, "rust-lang");
        assert_eq!(repo, "rust");
    }

    #[test]
    fn test_parse_github_url_ssh() {
        let (owner, repo) = parse_github_url("git@github.com:rust-lang/rust.git").unwrap();
        assert_eq!(owner, "rust-lang");
        assert_eq!(repo, "rust");
    }

    #[test]
    fn test_parse_github_url_with_trailing_slash() {
        let (owner, repo) = parse_github_url("https://github.com/rust-lang/rust/").unwrap();
        assert_eq!(owner, "rust-lang");
        assert_eq!(repo, "rust");
    }

    #[test]
    fn test_parse_github_url_invalid() {
        assert!(parse_github_url("https://github.com/").is_none());
        assert!(parse_github_url("https://github.com/only-owner").is_none());
        assert!(parse_github_url("https://gitlab.com/owner/repo").is_none());
    }

    #[test]
    fn test_parse_gitlab_url_https() {
        let (owner, repo) = parse_gitlab_url("https://gitlab.com/gitlab-org/gitlab").unwrap();
        assert_eq!(owner, "gitlab-org");
        assert_eq!(repo, "gitlab");
    }

    #[test]
    fn test_parse_gitlab_url_ssh() {
        let (owner, repo) = parse_gitlab_url("git@gitlab.com:gitlab-org/gitlab.git").unwrap();
        assert_eq!(owner, "gitlab-org");
        assert_eq!(repo, "gitlab");
    }

    #[test]
    fn test_parse_gitlab_url_invalid() {
        assert!(parse_gitlab_url("https://github.com/owner/repo").is_none());
        assert!(parse_gitlab_url("https://gitlab.com/").is_none());
    }

    #[test]
    fn test_github_import_request_parse() {
        let req: GitHubImportRequest = serde_json::from_str(
            r#"{"github_url":"https://github.com/rust-lang/rust","token":"ghp_abc123"}"#,
        )
        .unwrap();
        assert_eq!(req.github_url, "https://github.com/rust-lang/rust");
        assert_eq!(req.token.as_deref(), Some("ghp_abc123"));
        assert!(req.owner_id.is_none());
    }

    #[test]
    fn test_gitlab_import_request_parse() {
        let req: GitLabImportRequest = serde_json::from_str(
            r#"{"gitlab_url":"https://gitlab.com/gitlab-org/gitlab","token":"glpat-xyz"}"#,
        )
        .unwrap();
        assert_eq!(req.gitlab_url, "https://gitlab.com/gitlab-org/gitlab");
        assert_eq!(req.token.as_deref(), Some("glpat-xyz"));
    }

    #[test]
    fn test_import_response_serialization() {
        let resp = ImportResponse {
            status: "importing".into(),
            message: "Started".into(),
            repo_id: Some(Uuid::nil().to_string()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"status\":\"importing\""));
    }

    #[test]
    fn test_parse_git_url_https() {
        let name = parse_git_url("https://github.com/owner/repo").unwrap();
        assert_eq!(name, "repo");
    }

    #[test]
    fn test_parse_git_url_https_with_git_suffix() {
        let name = parse_git_url("https://github.com/owner/repo.git").unwrap();
        assert_eq!(name, "repo");
    }

    #[test]
    fn test_parse_git_url_ssh() {
        let name = parse_git_url("git@github.com:owner/repo.git").unwrap();
        assert_eq!(name, "repo");
    }

    #[test]
    fn test_parse_git_url_trailing_slash() {
        let name = parse_git_url("https://example.com/org/project/").unwrap();
        assert_eq!(name, "project");
    }

    #[test]
    fn test_parse_git_url_invalid() {
        assert!(parse_git_url("https://github.com/").is_none());
        assert!(parse_git_url("ftp://example.com/repo").is_none());
    }

    #[test]
    fn test_url_import_request_parse() {
        let req: UrlImportRequest = serde_json::from_str(
            r#"{"url":"https://github.com/rust-lang/rust","token":"ghp_abc","name":"my-rust"}"#,
        )
        .unwrap();
        assert_eq!(req.url, "https://github.com/rust-lang/rust");
        assert_eq!(req.token.as_deref(), Some("ghp_abc"));
        assert_eq!(req.name.as_deref(), Some("my-rust"));
        assert!(req.owner_id.is_none());
    }

    #[test]
    fn test_url_import_request_minimal() {
        let req: UrlImportRequest =
            serde_json::from_str(r#"{"url":"https://example.com/repo.git"}"#).unwrap();
        assert_eq!(req.url, "https://example.com/repo.git");
        assert!(req.token.is_none());
        assert!(req.name.is_none());
        assert!(req.owner_id.is_none());
    }
}
