#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{OptionalAuthUser, require_permission};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use civit_shared::permissions::{Action, Resource};

pub async fn info_refs(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    headers: HeaderMap,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    if !state.git_service.repo_exists(&owner, &name) {
        return (StatusCode::NOT_FOUND, "repository not found").into_response();
    }

    // Extract Git-Protocol header (e.g., "version=2") for protocol negotiation
    let git_protocol = headers
        .get("git-protocol")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Determine which service the client wants (upload-pack for fetch, receive-pack for push)
    let service = params
        .get("service")
        .and_then(|s| s.strip_prefix("git-"))
        .unwrap_or("upload-pack");

    let repo_path = state.git_service.repo_path(&owner, &name);
    let proto_ref: Option<&str> = git_protocol.as_deref();
    let (data, content_type) = match service {
        "receive-pack" => {
            match crate::git::http::info_refs(&repo_path, "receive-pack", proto_ref) {
                Ok(d) => (d, "application/x-git-receive-pack-advertisement"),
                Err(e) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, format!("git error: {e}"))
                        .into_response();
                }
            }
        }
        _ => match crate::git::http::info_refs(&repo_path, "upload-pack", proto_ref) {
            Ok(d) => (d, "application/x-git-upload-pack-advertisement"),
            Err(e) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("git error: {e}"))
                    .into_response();
            }
        },
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "no-cache")
        .body(axum::body::Body::from(data))
        .unwrap_or_else(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, "response build error").into_response()
        })
}

pub async fn upload_pack(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    headers: HeaderMap,
    _auth: OptionalAuthUser,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    if !state.git_service.repo_exists(&owner, &name) {
        return (StatusCode::NOT_FOUND, "repository not found").into_response();
    }

    let git_protocol = headers.get("git-protocol").and_then(|v| v.to_str().ok());
    let repo_path = state.git_service.repo_path(&owner, &name);
    match crate::git::http::upload_pack(&repo_path, &body, git_protocol) {
        Ok(data) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/x-git-upload-pack-result")
            .header(header::CACHE_CONTROL, "no-cache")
            .body(axum::body::Body::from(data))
            .unwrap_or_else(|_| {
                (StatusCode::INTERNAL_SERVER_ERROR, "response build error").into_response()
            }),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("git error: {e}")).into_response(),
    }
}

pub async fn receive_pack(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    headers: HeaderMap,
    auth: OptionalAuthUser,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    // Require push permission on the repository (skip for unauthenticated / read-only repos)
    if let Some(auth_user) = &auth.0 {
        // Resolve repo_id from owner/name for permission checking
        let repo_id = {
            let user = state.db.get_user_by_username(&owner).await.ok();
            if let Some(user) = user {
                state
                    .db
                    .get_repo_by_owner_name(user.id, &name)
                    .await
                    .ok()
                    .map(|r| civit_shared::RepoId::new(r.id))
            } else {
                None
            }
        };

        if let Err(rejection) = require_permission(
            &state,
            auth_user,
            Resource::Repository,
            Action::Push,
            repo_id,
            None,
            None,
        )
        .await
        {
            return rejection.into_response();
        }
    }

    if !state.git_service.repo_exists(&owner, &name) {
        return (StatusCode::NOT_FOUND, "repository not found").into_response();
    }

    let git_protocol = headers.get("git-protocol").and_then(|v| v.to_str().ok());
    let repo_path = state.git_service.repo_path(&owner, &name);
    tracing::info!(body_len = body.len(), "receive-pack request received");
    match crate::git::http::receive_pack(&repo_path, &body, git_protocol) {
        Ok(data) => {
            // Fire-and-forget: trigger CI/CD pipelines on push
            let state_clone = state.clone();
            let owner_clone = owner.clone();
            let name_clone = name.clone();
            tokio::spawn(async move {
                crate::api::pipelines::trigger_pipelines_on_push(
                    &state_clone,
                    &owner_clone,
                    &name_clone,
                )
                .await;

                // Auto-update open PRs whose source branch was pushed to
                update_prs_on_push(&state_clone, &owner_clone, &name_clone).await;

                // Incremental search re-index after push
                if let Some(repo_id) = get_repo_id_from_owner_name(&state_clone, &owner_clone, &name_clone).await {
                    let pool = state_clone.db.pool();
                    let rp = state_clone.git_service.repo_path(&owner_clone, &name_clone);
                    if let Err(e) = crate::api::search::reindex_repo_after_push(pool, &repo_id, &rp).await {
                        tracing::warn!(error = %e, "failed to re-index search after push for {owner_clone}/{name_clone}");
                    }
                }
            });

            Response::builder()
                .status(StatusCode::OK)
                .header(
                    header::CONTENT_TYPE,
                    "application/x-git-receive-pack-result",
                )
                .header(header::CACHE_CONTROL, "no-cache")
                .body(axum::body::Body::from(data))
                .unwrap_or_else(|_| {
                    (StatusCode::INTERNAL_SERVER_ERROR, "response build error").into_response()
                })
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("git error: {e}")).into_response(),
    }
}

/// Strip `.git` suffix from repo name for smart HTTP routes.
/// Git clients append `.git` to the URL, but our repos are stored as `{name}.git` on disk.
fn strip_dotgit(name: &str) -> String {
    name.strip_suffix(".git").unwrap_or(name).to_string()
}

/// Wrapper handlers that strip `.git` from the repo name path segment.
pub async fn info_refs_dotgit(
    state: State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    headers: HeaderMap,
    auth: OptionalAuthUser,
) -> impl IntoResponse {
    info_refs(
        state,
        Path((owner, strip_dotgit(&name))),
        Query(params),
        headers,
        auth,
    )
    .await
}

pub async fn upload_pack_dotgit(
    state: State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    headers: HeaderMap,
    auth: OptionalAuthUser,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    upload_pack(
        state,
        Path((owner, strip_dotgit(&name))),
        headers,
        auth,
        body,
    )
    .await
}

pub async fn receive_pack_dotgit(
    state: State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    headers: HeaderMap,
    auth: OptionalAuthUser,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    receive_pack(
        state,
        Path((owner, strip_dotgit(&name))),
        headers,
        auth,
        body,
    )
    .await
}

// NOTE: Cannot add .git routes to axum Router because axum doesn't allow
// literal text and capture in the same segment. Git clients without .git
// in the URL will work fine with the existing routes.

/// After a push, update open PRs whose source branch was pushed to.
/// Sets the head_commit_sha to the latest commit on the pushed branch.
async fn update_prs_on_push(state: &AppState, owner: &str, name: &str) {
    // List all branches in the repo
    let branches = match list_branches(state, owner, name) {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!(error = %e, "failed to list branches for PR auto-update");
            return;
        }
    };

    // Find repo_id
    let pool = state.db.pool();
    let repo_id = match sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT r.id FROM repositories r JOIN users u ON r.owner_id = u.id WHERE u.username = $1 AND r.name = $2",
    )
    .bind(owner)
    .bind(name)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(id)) => id,
        _ => return,
    };

    // For each branch, find open PRs and update their head SHA
    for branch_name in &branches {
        let sha = match get_branch_head_sha(state, owner, name, branch_name) {
            Ok(Some(s)) => s,
            _ => continue,
        };

        if let Ok(Some(pr)) = state
            .db
            .find_open_pr_by_source_branch(repo_id, branch_name)
            .await
        {
            let _ = state.db.set_pr_commit_shas(pr.id, &sha, &sha).await;
            tracing::info!(
                pr_number = pr.number,
                branch = %branch_name,
                sha = %sha,
                "auto-updated PR head commit on push"
            );
        }
    }
}

/// List branch names in a repository.
fn list_branches(state: &AppState, owner: &str, name: &str) -> crate::error::Result<Vec<String>> {
    let git_bin = std::env::var("GIT_BIN").unwrap_or_else(|_| "git".to_string());
    let repo_path = state.git_service.repo_path(owner, name);

    use std::process::Command;
    let output = Command::new(&git_bin)
        .args(["branch", "--format=%(refname:short)"])
        .arg(&repo_path)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|e| crate::error::CoreError::Git(format!("git branch failed: {e}")))?;

    if !output.status.success() {
        return Err(crate::error::CoreError::Git(format!(
            "git branch failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let branches: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    Ok(branches)
}

/// Get the HEAD commit SHA of a branch.
fn get_branch_head_sha(
    state: &AppState,
    owner: &str,
    name: &str,
    branch: &str,
) -> crate::error::Result<Option<String>> {
    let git_bin = std::env::var("GIT_BIN").unwrap_or_else(|_| "git".to_string());
    let repo_path = state.git_service.repo_path(owner, name);

    use std::process::Command;
    let output = Command::new(&git_bin)
        .args(["rev-parse", &format!("refs/heads/{branch}")])
        .arg(&repo_path)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|e| crate::error::CoreError::Git(format!("git rev-parse failed: {e}")))?;

    if !output.status.success() {
        return Ok(None);
    }

    Ok(Some(
        String::from_utf8_lossy(&output.stdout).trim().to_string(),
    ))
}

async fn get_repo_id_from_owner_name(
    state: &AppState,
    owner: &str,
    name: &str,
) -> Option<uuid::Uuid> {
    let pool = state.db.pool();
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

#[cfg(test)]
mod tests {
    use axum::extract::Path;

    #[test]
    fn test_path_tuple_type() {
        let p: Path<(String, String)> = Path((String::new(), String::new()));
        assert_eq!(p.0.0, "");
        assert_eq!(p.0.1, "");
    }
}
