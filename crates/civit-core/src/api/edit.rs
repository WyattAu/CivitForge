#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::routing::put;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Helper: get repo id
// ---------------------------------------------------------------------------

async fn get_repo_id(pool: &sqlx::PgPool, owner: &str, name: &str) -> Option<uuid::Uuid> {
    sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT r.id FROM repositories r JOIN users u ON r.owner_id = u.id WHERE u.username = $1 AND r.name = $2",
    )
    .bind(owner)
    .bind(name)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

// ---------------------------------------------------------------------------
// Helper: error responses
// ---------------------------------------------------------------------------

fn err_response(status: StatusCode, msg: &str) -> axum::response::Response {
    (
        status,
        Json(CoreError::NotFound(msg.to_string()).error_response()),
    )
        .into_response()
}

fn internal_err(msg: &str) -> axum::response::Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(CoreError::Internal(msg.to_string()).error_response()),
    )
        .into_response()
}

fn git_err(msg: &str) -> axum::response::Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(CoreError::Git(msg.to_string()).error_response()),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Request structs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateOrUpdateFileRequest {
    pub content: String,
    pub message: String,
    #[serde(default = "default_branch")]
    pub branch: String,
    #[serde(default)]
    pub commit_choice: CommitChoice,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum CommitChoice {
    #[default]
    Direct,
    NewBranch,
}

fn default_branch() -> String {
    "main".into()
}

#[derive(Debug, Deserialize)]
pub struct DeleteFileRequest {
    pub message: String,
    #[serde(default = "default_branch")]
    pub branch: String,
    #[serde(default)]
    pub commit_choice: CommitChoice,
}

#[derive(serde::Serialize)]
pub struct FileEditResponse {
    pub commit_sha: String,
    pub branch: String,
    pub path: String,
    pub pr_url: Option<String>,
}

// ---------------------------------------------------------------------------
// Git helper: run a git command in the repo directory
// ---------------------------------------------------------------------------

async fn run_git(repo_path: &std::path::Path, args: &[&str]) -> Result<String, String> {
    let output = tokio::process::Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .output()
        .await
        .map_err(|e| format!("failed to run git: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git {} failed: {}", args.join(" "), stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// ---------------------------------------------------------------------------
// Helper: ensure we have a worktree for the given branch
// ---------------------------------------------------------------------------

fn ensure_worktree(
    storage_root: &std::path::Path,
    owner: &str,
    name: &str,
    branch: &str,
) -> Result<std::path::PathBuf, String> {
    let bare_path = storage_root.join(owner).join(format!("{name}.git"));
    let worktree_path = storage_root
        .join(owner)
        .join(format!("{name}.git"))
        .join(format!("..worktree-{branch}"));

    // Check if worktree already exists
    if worktree_path.join(".git").exists() || worktree_path.join(".git").is_file() {
        return Ok(worktree_path);
    }

    // Create worktree from bare repo
    let output = std::process::Command::new("git")
        .args(["worktree", "add", worktree_path.to_str().expect("path not valid utf-8"), branch])
        .current_dir(&bare_path)
        .output()
        .map_err(|e| format!("failed to create worktree: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git worktree add failed: {stderr}"));
    }

    Ok(worktree_path)
}

// ---------------------------------------------------------------------------
// Helper: clean up worktree after commit
// ---------------------------------------------------------------------------

fn remove_worktree(storage_root: &std::path::Path, owner: &str, name: &str, branch: &str) {
    let bare_path = storage_root.join(owner).join(format!("{name}.git"));
    let worktree_path = storage_root
        .join(owner)
        .join(format!("{name}.git"))
        .join(format!("..worktree-{branch}"));

    let _ = std::process::Command::new("git")
        .args([
            "worktree",
            "remove",
            "--force",
            worktree_path.to_str().expect("path not valid utf-8"),
        ])
        .current_dir(&bare_path)
        .output();
}

// ---------------------------------------------------------------------------
// 1. PUT /repos/{owner}/{name}/edit/{path:.+} — create/update file
// ---------------------------------------------------------------------------

pub async fn create_or_update_file(
    State(state): State<AppState>,
    Path((owner, name, file_path)): Path<(String, String, String)>,
    auth: AuthUser,
    Json(req): Json<CreateOrUpdateFileRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };
    let _ = repo_id;

    let storage_root = std::path::PathBuf::from(&state.config.storage_path);
    let bare_path = storage_root.join(&owner).join(format!("{name}.git"));

    if !bare_path.join("HEAD").exists() {
        return err_response(StatusCode::NOT_FOUND, "git repository not found");
    }

    if req.message.trim().is_empty() {
        return err_response(StatusCode::BAD_REQUEST, "commit message is required");
    }

    let target_branch = match req.commit_choice {
        CommitChoice::Direct => req.branch.clone(),
        CommitChoice::NewBranch => {
            let edit_branch = format!("edit/{}", file_path.replace('/', "-"));
            // Create the new branch from the source branch
            match run_git(&bare_path, &["branch", &edit_branch, &req.branch]).await {
                Ok(_) => edit_branch,
                Err(e) => return git_err(&e),
            }
        }
    };

    // Create worktree
    let worktree = match ensure_worktree(&storage_root, &owner, &name, &target_branch) {
        Ok(p) => p,
        Err(e) => return git_err(&e),
    };

    // Write file content
    let full_path = worktree.join(&file_path);
    if let Some(parent) = full_path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        remove_worktree(&storage_root, &owner, &name, &target_branch);
        return internal_err(&format!("failed to create directories: {e}"));
    }

    if let Err(e) = std::fs::write(&full_path, &req.content) {
        remove_worktree(&storage_root, &owner, &name, &target_branch);
        return internal_err(&format!("failed to write file: {e}"));
    }

    // Stage and commit
    if let Err(e) = run_git(&worktree, &["add", &file_path]).await {
        remove_worktree(&storage_root, &owner, &name, &target_branch);
        return git_err(&e);
    }

    let author_name = &auth.username;
    let commit_result = run_git(
        &worktree,
        &[
            "commit",
            "-m",
            &req.message,
            "--author",
            &format!("{author_name} <{author_name}@civitforge>"),
        ],
    )
    .await;

    if let Err(e) = commit_result {
        remove_worktree(&storage_root, &owner, &name, &target_branch);
        return git_err(&e);
    }

    // Get the commit SHA
    let sha = match run_git(&worktree, &["rev-parse", "HEAD"]).await {
        Ok(s) => s,
        Err(e) => {
            remove_worktree(&storage_root, &owner, &name, &target_branch);
            return git_err(&e);
        }
    };

    // Push to bare repo
    let refspec = format!("{target_branch}:refs/heads/{target_branch}");
    if let Err(e) = run_git(&worktree, &["push", "origin", &refspec]).await {
        remove_worktree(&storage_root, &owner, &name, &target_branch);
        return git_err(&e);
    }

    remove_worktree(&storage_root, &owner, &name, &target_branch);

    // If new_branch choice, create a PR automatically
    let pr_url = if req.commit_choice == CommitChoice::NewBranch {
        let title = format!("Edit {file_path}");
        let body = req.message.clone();
        let source = match run_git(&bare_path, &["rev-parse", "--abbrev-ref", "HEAD"]).await {
            Ok(b) => b,
            Err(_) => target_branch.clone(),
        };

        // Create PR via the pull_requests module
        let pr_result = sqlx::query_as::<_, (uuid::Uuid, i32)>(
            "INSERT INTO pull_requests (repo_id, title, body, source_branch, target_branch, author_id, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, 'open', NOW(), NOW()) RETURNING id, number",
        )
        .bind(repo_id)
        .bind(&title)
        .bind(&body)
        .bind(&source)
        .bind(&req.branch)
        .bind(uuid::Uuid::parse_str(&auth.user_id).unwrap_or(uuid::Uuid::nil()))
        .fetch_optional(pool)
        .await;

        match pr_result {
            Ok(Some((_pr_id, pr_number))) => Some(format!(
                "/api/v1/repos/{owner}/{name}/pull-requests/{pr_number}"
            )),
            _ => None,
        }
    } else {
        None
    };

    let resp = FileEditResponse {
        commit_sha: sha,
        branch: target_branch,
        path: file_path,
        pr_url,
    };

    (StatusCode::OK, Json(resp)).into_response()
}

// ---------------------------------------------------------------------------
// 2. DELETE /repos/{owner}/{name}/edit/{path:.+} — delete file
// ---------------------------------------------------------------------------

pub async fn delete_file(
    State(state): State<AppState>,
    Path((owner, name, file_path)): Path<(String, String, String)>,
    auth: AuthUser,
    Json(req): Json<DeleteFileRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };
    let _ = repo_id;

    let storage_root = std::path::PathBuf::from(&state.config.storage_path);
    let bare_path = storage_root.join(&owner).join(format!("{name}.git"));

    if !bare_path.join("HEAD").exists() {
        return err_response(StatusCode::NOT_FOUND, "git repository not found");
    }

    if req.message.trim().is_empty() {
        return err_response(StatusCode::BAD_REQUEST, "commit message is required");
    }

    let target_branch = match req.commit_choice {
        CommitChoice::Direct => req.branch.clone(),
        CommitChoice::NewBranch => {
            let edit_branch = format!("delete/{}", file_path.replace('/', "-"));
            match run_git(&bare_path, &["branch", &edit_branch, &req.branch]).await {
                Ok(_) => edit_branch,
                Err(e) => return git_err(&e),
            }
        }
    };

    let worktree = match ensure_worktree(&storage_root, &owner, &name, &target_branch) {
        Ok(p) => p,
        Err(e) => return git_err(&e),
    };

    let full_path = worktree.join(&file_path);
    if !full_path.exists() {
        remove_worktree(&storage_root, &owner, &name, &target_branch);
        return err_response(StatusCode::NOT_FOUND, "file not found in repository");
    }

    // Remove and commit
    if let Err(e) = run_git(&worktree, &["rm", &file_path]).await {
        remove_worktree(&storage_root, &owner, &name, &target_branch);
        return git_err(&e);
    }

    let author_name = &auth.username;
    let commit_result = run_git(
        &worktree,
        &[
            "commit",
            "-m",
            &req.message,
            "--author",
            &format!("{author_name} <{author_name}@civitforge>"),
        ],
    )
    .await;

    if let Err(e) = commit_result {
        remove_worktree(&storage_root, &owner, &name, &target_branch);
        return git_err(&e);
    }

    let sha = match run_git(&worktree, &["rev-parse", "HEAD"]).await {
        Ok(s) => s,
        Err(e) => {
            remove_worktree(&storage_root, &owner, &name, &target_branch);
            return git_err(&e);
        }
    };

    let refspec = format!("{target_branch}:refs/heads/{target_branch}");
    if let Err(e) = run_git(&worktree, &["push", "origin", &refspec]).await {
        remove_worktree(&storage_root, &owner, &name, &target_branch);
        return git_err(&e);
    }

    remove_worktree(&storage_root, &owner, &name, &target_branch);

    let resp = FileEditResponse {
        commit_sha: sha,
        branch: target_branch,
        path: file_path,
        pr_url: None,
    };

    (StatusCode::OK, Json(resp)).into_response()
}

// ---------------------------------------------------------------------------
// Route registration
// ---------------------------------------------------------------------------

pub fn edit_routes() -> Router<AppState> {
    Router::new().route(
        "/api/v1/repos/{owner}/{name}/edit/{path}",
        put(create_or_update_file).delete(delete_file),
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_file_request_deserialize() {
        let json = r#"{"content":"hello world","message":"Add file","branch":"main","commit_choice":"direct"}"#;
        let req: CreateOrUpdateFileRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.content, "hello world");
        assert_eq!(req.message, "Add file");
        assert_eq!(req.branch, "main");
        assert_eq!(req.commit_choice, CommitChoice::Direct);
    }

    #[test]
    fn test_create_file_request_defaults() {
        let json = r#"{"content":"x","message":"msg"}"#;
        let req: CreateOrUpdateFileRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.branch, "main");
        assert_eq!(req.commit_choice, CommitChoice::Direct);
    }

    #[test]
    fn test_create_file_request_new_branch() {
        let json = r#"{"content":"x","message":"msg","commit_choice":"new_branch"}"#;
        let req: CreateOrUpdateFileRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.commit_choice, CommitChoice::NewBranch);
    }

    #[test]
    fn test_delete_file_request_deserialize() {
        let json = r#"{"message":"Remove old file","branch":"develop"}"#;
        let req: DeleteFileRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.message, "Remove old file");
        assert_eq!(req.branch, "develop");
        assert_eq!(req.commit_choice, CommitChoice::Direct);
    }

    #[test]
    fn test_commit_choice_serialization() {
        let dc = CommitChoice::Direct;
        let json = serde_json::to_string(&dc).unwrap();
        assert_eq!(json, "\"direct\"");

        let nb = CommitChoice::NewBranch;
        let json = serde_json::to_string(&nb).unwrap();
        assert_eq!(json, "\"new_branch\"");
    }

    #[test]
    fn test_file_edit_response_serialization() {
        let resp = FileEditResponse {
            commit_sha: "abc123".into(),
            branch: "main".into(),
            path: "src/main.rs".into(),
            pr_url: Some("/api/v1/repos/o/n/pull-requests/1".into()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("abc123"));
        assert!(json.contains("src/main.rs"));
    }

    #[test]
    fn test_file_edit_response_no_pr() {
        let resp = FileEditResponse {
            commit_sha: "def456".into(),
            branch: "main".into(),
            path: "README.md".into(),
            pr_url: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("def456"));
        assert!(json.contains("null"));
    }

    #[test]
    fn test_commit_choice_default_is_direct() {
        assert_eq!(CommitChoice::default(), CommitChoice::Direct);
    }

    #[test]
    fn test_default_branch() {
        assert_eq!(default_branch(), "main");
    }
}
