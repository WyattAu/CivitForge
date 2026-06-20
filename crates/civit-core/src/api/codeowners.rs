#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, OptionalAuthUser};
use crate::error::CoreError;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct CodeownersResponse {
    pub owner: String,
    pub name: String,
    pub content: String,
    pub encoding: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCodeownersRequest {
    pub content: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CodeownersOwner {
    pub pattern: String,
    pub owners: Vec<String>,
}

/// Parse CODEOWNERS file from a repository's filesystem paths.
pub async fn parse_codeowners_from_repo(
    owner: &str,
    name: &str,
    storage_path: &str,
) -> Option<Vec<CodeownersOwner>> {
    let repo_path = std::path::Path::new(storage_path)
        .join(owner)
        .join(format!("{name}.git"));

    let content = std::fs::read_to_string(repo_path.join("CODEOWNERS"))
        .or_else(|_| std::fs::read_to_string(repo_path.join(".github").join("CODEOWNERS")))
        .or_else(|_| std::fs::read_to_string(repo_path.join("docs").join("CODEOWNERS")))
        .ok()?;

    Some(parse_codeowners(&content))
}

/// Get required CODEOWNERS reviewers for a set of file paths.
/// Returns unique owner usernames (with leading @ stripped).
pub async fn get_required_reviewers(
    owner: &str,
    name: &str,
    storage_path: &str,
    paths: &[String],
) -> Vec<String> {
    let entries = match parse_codeowners_from_repo(owner, name, storage_path).await {
        Some(e) => e,
        None => return Vec::new(),
    };
    let owners = find_codeowners_for_files(&entries, paths);
    owners
        .into_iter()
        .map(|o| o.trim_start_matches('@').to_string())
        .collect()
}

/// Check if all CODEOWNERS-required reviewers have approved a PR.
/// Returns true if all required reviewers have approved.
pub async fn check_codeowners_approval(
    pool: &sqlx::PgPool,
    pr_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let rows: Vec<(bool,)> =
        sqlx::query_as("SELECT approved FROM codeowners_reviews WHERE pr_id = $1")
            .bind(pr_id)
            .fetch_all(pool)
            .await?;

    if rows.is_empty() {
        return Ok(true);
    }
    Ok(rows.iter().all(|(approved,)| *approved))
}

/// Record a CODEOWNERS review entry for a PR.
pub async fn record_codeowners_review(
    pool: &sqlx::PgPool,
    pr_id: Uuid,
    reviewer: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO codeowners_reviews (pr_id, reviewer, approved, approved_at)
           VALUES ($1, $2, false, NULL)
           ON CONFLICT (pr_id, reviewer) DO NOTHING"#,
    )
    .bind(pr_id)
    .bind(reviewer)
    .execute(pool)
    .await?;
    Ok(())
}

/// Update a CODEOWNERS review approval status.
pub async fn update_codeowners_review_approval(
    pool: &sqlx::PgPool,
    pr_id: Uuid,
    reviewer: &str,
    approved: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE codeowners_reviews
           SET approved = $3, approved_at = CASE WHEN $3 THEN NOW() ELSE NULL END
           WHERE pr_id = $1 AND reviewer = $2"#,
    )
    .bind(pr_id)
    .bind(reviewer)
    .bind(approved)
    .execute(pool)
    .await?;
    Ok(())
}

/// Insert CODEOWNERS review records for all required owners of a PR.
/// Call this when a PR is created or when file changes are updated.
pub async fn insert_codeowners_reviews_for_pr(
    pool: &sqlx::PgPool,
    pr_id: Uuid,
    required_reviewers: &[String],
) -> Result<(), sqlx::Error> {
    for reviewer in required_reviewers {
        record_codeowners_review(pool, pr_id, reviewer).await?;
    }
    Ok(())
}

pub async fn get_codeowners(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let owner_user = match state.db.get_user_by_username(&owner).await {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("owner not found".into()).error_response()),
            )
                .into_response();
        }
    };

    let repo = match state.db.get_repo_by_owner_name(owner_user.id, &name).await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repo not found".into()).error_response()),
            )
                .into_response();
        }
    };

    // Read CODEOWNERS from the git repo's default branch
    let git_service = &state.git_service;
    let repo_path = git_service.repo_path(&owner, &name);

    let content = match std::fs::read_to_string(repo_path.join("CODEOWNERS")) {
        Ok(c) => c,
        Err(_) => {
            // Try .github/CODEOWNERS
            match std::fs::read_to_string(repo_path.join(".github").join("CODEOWNERS")) {
                Ok(c) => c,
                Err(_) => {
                    // Try docs/CODEOWNERS
                    match std::fs::read_to_string(repo_path.join("docs").join("CODEOWNERS")) {
                        Ok(c) => c,
                        Err(_) => {
                            return (
                                StatusCode::NOT_FOUND,
                                Json(
                                    CoreError::NotFound("CODEOWNERS file not found".into())
                                        .error_response(),
                                ),
                            )
                                .into_response();
                        }
                    }
                }
            }
        }
    };

    let _ = repo; // repo used for access control if needed

    (
        StatusCode::OK,
        Json(CodeownersResponse {
            owner,
            name,
            content,
            encoding: "plain".into(),
        }),
    )
        .into_response()
}

pub async fn update_codeowners(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<UpdateCodeownersRequest>,
) -> impl IntoResponse {
    let owner_user = match state.db.get_user_by_username(&owner).await {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("owner not found".into()).error_response()),
            )
                .into_response();
        }
    };

    let repo = match state.db.get_repo_by_owner_name(owner_user.id, &name).await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repo not found".into()).error_response()),
            )
                .into_response();
        }
    };

    // Check the user is the owner or an admin
    let is_admin = auth.role.as_str() == "admin";
    let is_owner = auth.user_id == owner_user.id.to_string();
    if !is_admin && !is_owner {
        return (
            StatusCode::FORBIDDEN,
            Json(CoreError::Forbidden("not repo owner".into()).error_response()),
        )
            .into_response();
    }

    let git_service = &state.git_service;
    let repo_path = git_service.repo_path(&owner, &name);
    let codeowners_path = repo_path.join("CODEOWNERS");

    let message = req.message.unwrap_or_else(|| "Update CODEOWNERS".into());

    // Write CODEOWNERS file
    if let Err(e) = std::fs::write(&codeowners_path, &req.content) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Io(e).error_response()),
        )
            .into_response();
    }

    // Commit the change
    let _ = repo; // repo_id available for future DB tracking

    // Try to commit via git
    let commit_result = std::process::Command::new("git")
        .arg("-C")
        .arg(&repo_path)
        .arg("add")
        .arg("CODEOWNERS")
        .output();

    if let Ok(output) = commit_result
        && output.status.success()
    {
        let _ = std::process::Command::new("git")
            .arg("-C")
            .arg(&repo_path)
            .arg("commit")
            .arg("-m")
            .arg(&message)
            .arg("--author")
            .arg(format!("{owner} <civitforge@localhost>"))
            .output();
    }

    (
        StatusCode::OK,
        Json(CodeownersResponse {
            owner,
            name,
            content: req.content,
            encoding: "plain".into(),
        }),
    )
        .into_response()
}

/// Parse a CODEOWNERS file into structured owner entries.
pub fn parse_codeowners(content: &str) -> Vec<CodeownersOwner> {
    let mut entries = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let pattern = parts[0].to_string();
            let owners: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
            entries.push(CodeownersOwner { pattern, owners });
        }
    }
    entries
}

/// Given a list of changed file paths, find which CODEOWNERS patterns match
/// and return the union of owner usernames.
pub fn find_codeowners_for_files(
    entries: &[CodeownersOwner],
    changed_files: &[String],
) -> Vec<String> {
    let mut owners = std::collections::HashSet::new();
    for file in changed_files {
        // Check patterns from last to first (most specific wins, but we collect all)
        for entry in entries.iter().rev() {
            if matches_pattern(&entry.pattern, file) {
                for owner in &entry.owners {
                    owners.insert(owner.clone());
                }
            }
        }
    }
    let mut result: Vec<String> = owners.into_iter().collect();
    result.sort();
    result
}

fn matches_pattern(pattern: &str, path: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    // Normalize: strip leading /
    let pat = pattern.trim_start_matches('/');
    let pat = pat.trim_end_matches('/');

    if pattern.contains("**") {
        let prefix = pattern.trim_end_matches("**").trim_end_matches('/');
        return path.starts_with(prefix);
    }

    // Extension match: *.ext
    if let Some(ext) = pat.strip_prefix("*.") {
        return path.ends_with(&format!(".{ext}"));
    }

    // Directory match: pattern ends with /
    if pattern.ends_with('/') {
        let dir = pat.trim_end_matches('/');
        return path.starts_with(dir);
    }

    // Exact or suffix match
    path == pat || path.ends_with(&format!("/{pat}")) || path.contains(&format!("/{pat}/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_codeowners_simple() {
        let content = "# comment\n*.js @frontend-team\n*.rs @backend-team\n/docs/ @docs-team\n";
        let entries = parse_codeowners(content);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].pattern, "*.js");
        assert_eq!(entries[0].owners, vec!["@frontend-team"]);
        assert_eq!(entries[1].pattern, "*.rs");
        assert_eq!(entries[1].owners, vec!["@backend-team"]);
        assert_eq!(entries[2].pattern, "/docs/");
        assert_eq!(entries[2].owners, vec!["@docs-team"]);
    }

    #[test]
    fn test_parse_codeowners_empty() {
        assert!(parse_codeowners("").is_empty());
        assert!(parse_codeowners("# just a comment\n").is_empty());
    }

    #[test]
    fn test_parse_codeowners_multiple_owners() {
        let content = "*.ts @alice @bob\n";
        let entries = parse_codeowners(content);
        assert_eq!(entries[0].owners, vec!["@alice", "@bob"]);
    }

    #[test]
    fn test_matches_pattern_star() {
        assert!(matches_pattern("*", "any/file.rs"));
    }

    #[test]
    fn test_matches_pattern_extension() {
        assert!(matches_pattern("*.rs", "src/main.rs"));
        assert!(!matches_pattern("*.rs", "src/main.py"));
    }

    #[test]
    fn test_matches_pattern_directory() {
        assert!(matches_pattern("/docs/", "docs/readme.md"));
        assert!(!matches_pattern("/docs/", "src/docs/readme.md"));
    }

    #[test]
    fn test_matches_pattern_double_star() {
        assert!(matches_pattern("src/**", "src/foo/bar.rs"));
        assert!(!matches_pattern("src/**", "lib/foo.rs"));
    }

    #[test]
    fn test_find_codeowners_for_files() {
        let entries = parse_codeowners("*.rs @rust-team\n*.js @js-team\n");
        let files = vec!["src/main.rs".into(), "src/app.js".into()];
        let owners = find_codeowners_for_files(&entries, &files);
        assert!(owners.contains(&"@rust-team".to_string()));
        assert!(owners.contains(&"@js-team".to_string()));
    }

    #[test]
    fn test_find_codeowners_no_match() {
        let entries = parse_codeowners("*.rs @rust-team\n");
        let files = vec!["README.md".into()];
        let owners = find_codeowners_for_files(&entries, &files);
        assert!(owners.is_empty());
    }

    #[test]
    fn test_codeowners_response_serialize() {
        let resp = CodeownersResponse {
            owner: "alice".into(),
            name: "myrepo".into(),
            content: "*.rs @team\n".into(),
            encoding: "plain".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"owner\":\"alice\""));
        assert!(json.contains("\"encoding\":\"plain\""));
    }

    #[test]
    fn test_update_codeowners_request_deserialize() {
        let json = r#"{"content":"*.rs @team","message":"init"}"#;
        let req: UpdateCodeownersRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.content, "*.rs @team");
        assert_eq!(req.message.as_deref(), Some("init"));
    }
}
