#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, OptionalAuthUser, require_admin, require_permission};
use crate::error::CoreError;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{delete, get, patch, post},
};
use civit_shared::permissions::{Action, Resource};
use civit_shared::repo::{RepoResponse, StarToggleResponse, WatchToggleResponse};
use civit_shared::visibility::Visibility;
use civit_shared::{ListResponse, PaginationParams};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RequestVisibility {
    Public,
    Private,
    Internal,
}

#[derive(Debug, Deserialize)]
pub struct CreateRepoRequest {
    pub name: String,
    pub owner: String,
    pub description: String,
    pub visibility: RequestVisibility,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRepoRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub visibility: Option<RequestVisibility>,
    pub default_branch: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BranchResponse {
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TagResponse {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StarredResponse {
    pub starred: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WatchedResponse {
    pub watched: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CollaboratorResponse {
    pub user_id: String,
    pub username: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct RawFileParams {
    pub path: String,
    pub ref_: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ArchiveParams {
    pub format: Option<String>,
    #[serde(default)]
    pub ref_: Option<String>,
}

impl RequestVisibility {
    fn as_str(&self) -> &str {
        match self {
            RequestVisibility::Public => "public",
            RequestVisibility::Private => "private",
            RequestVisibility::Internal => "internal",
        }
    }
}

fn git_err(e: impl std::fmt::Display) -> CoreError {
    CoreError::Git(e.to_string())
}

pub(crate) fn repo_to_response(
    r: crate::db::Repository,
    owner_name: Option<String>,
    state: &AppState,
) -> RepoResponse {
    let vis = match r.visibility.as_str() {
        "public" => Visibility::Public,
        "internal" => Visibility::Internal,
        _ => Visibility::Private,
    };
    let display_name = owner_name.unwrap_or_else(|| r.owner_id.to_string());
    let full_name = format!("{display_name}/{}", r.name);
    let http_clone_url = Some(format!(
        "http://{host}:{port}/{display_name}/{name}",
        host = state.config.host,
        port = state.config.port,
        name = r.name,
    ));
    let ssh_clone_url = Some(format!(
        "ssh://git@{host}/{display_name}/{name}",
        host = state.config.host,
        name = r.name,
    ));
    RepoResponse {
        id: r.id.into(),
        name: r.name,
        full_name,
        description: if r.description.is_empty() {
            None
        } else {
            Some(r.description)
        },
        visibility: vis,
        owner_id: r.owner_id.into(),
        org_id: None,
        default_branch: r.default_branch,
        is_fork: r.is_fork,
        parent_repo_id: r.parent_repo_id.map(|id| id.into()),
        ssh_clone_url,
        http_clone_url,
        starred: None,
        watched: None,
        stars_count: Some(r.stars_count),
        watchers_count: Some(r.watchers_count),
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

pub(crate) async fn repos_to_responses(
    state: &AppState,
    repos: Vec<crate::db::Repository>,
) -> Vec<RepoResponse> {
    let unique_owners: std::collections::HashSet<Uuid> = repos.iter().map(|r| r.owner_id).collect();
    let mut owner_names: std::collections::HashMap<Uuid, String> =
        std::collections::HashMap::with_capacity(unique_owners.len());
    for id in unique_owners {
        let name = state
            .db
            .get_user_by_id(id)
            .await
            .map(|u| u.username)
            .unwrap_or_else(|_| id.to_string());
        owner_names.insert(id, name);
    }
    repos
        .into_iter()
        .map(|r| {
            let name = owner_names
                .get(&r.owner_id)
                .cloned()
                .unwrap_or_else(|| r.owner_id.to_string());
            repo_to_response(r, Some(name), state)
        })
        .collect()
}

pub async fn list_repos(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let limit = params.effective_per_page() as i64;
    let offset = params.effective_offset() as i64;
    match state.db.list_repos(limit, offset).await {
        Ok(repos) => {
            let total = state.db.count_repos().await.unwrap_or(repos.len() as i64) as u64;
            let out = repos_to_responses(&state, repos).await;
            let resp = ListResponse::from_total(out, total, &params);
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_repo(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: OptionalAuthUser,
) -> impl IntoResponse {
    let (owner_uuid, owner_name) = match Uuid::parse_str(&owner) {
        Ok(id) => {
            let uname = state
                .db
                .get_user_by_id(id)
                .await
                .map(|u| u.username)
                .unwrap_or_else(|_| id.to_string());
            (id, uname)
        }
        Err(_) => match state.db.get_user_by_username(&owner).await {
            Ok(user) => (user.id, user.username),
            Err(_) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(CoreError::NotFound("user not found".into()).error_response()),
                )
                    .into_response();
            }
        },
    };

    match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(repo) => {
            let mut resp = repo_to_response(repo.clone(), Some(owner_name), &state);
            if let Some(user_id) = auth.0.and_then(|a| Uuid::parse_str(&a.user_id).ok()) {
                resp.starred = Some(state.db.has_user_starred(user_id, repo.id).await.unwrap_or(false));
                resp.watched = Some(state.db.has_user_watched(user_id, repo.id).await.unwrap_or(false));
            }
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response(),
    }
}

pub async fn create_repo(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateRepoRequest>,
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

    if req.name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::Config("name required".into()).error_response()),
        )
            .into_response();
    }
    if !req
        .name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::Config("invalid repo name".into()).error_response()),
        )
            .into_response();
    }

    let owner_uuid = match Uuid::parse_str(&req.owner) {
        Ok(id) => id,
        Err(_) => match state.db.get_user_by_username(&req.owner).await {
            Ok(user) => user.id,
            Err(_) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(CoreError::NotFound("owner user not found".into()).error_response()),
                )
                    .into_response();
            }
        },
    };

    match state
        .db
        .create_repo(
            &req.name,
            &req.description,
            owner_uuid,
            None,
            req.visibility.as_str(),
            "main",
        )
        .await
    {
        Ok(repo) => {
            // Initialize bare git repository on disk
            let repo_disk_path = state.git_service.repo_path(&req.owner, &req.name);
            if let Err(e) = state.git_service.init_bare(&req.owner, &req.name) {
                eprintln!(
                    "[create_repo] Failed to init bare repo at {}: {}",
                    repo_disk_path.display(),
                    e
                );
                // Continue even if git init fails — repo exists in DB
            }
            let resp = repo_to_response(repo, Some(req.owner.clone()), &state);
            (StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_repo(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<UpdateRepoRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_permission(
        &state,
        &auth,
        Resource::Repository,
        Action::Update,
        None,
        None,
        None,
    )
    .await
    {
        return rejection.into_response();
    }

    let (owner_uuid, owner_name) = match Uuid::parse_str(&owner) {
        Ok(id) => {
            let uname = state
                .db
                .get_user_by_id(id)
                .await
                .map(|u| u.username)
                .unwrap_or_else(|_| id.to_string());
            (id, uname)
        }
        Err(_) => match state.db.get_user_by_username(&owner).await {
            Ok(user) => (user.id, user.username),
            Err(_) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(CoreError::NotFound("user not found".into()).error_response()),
                )
                    .into_response();
            }
        },
    };

    match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(repo) => {
            match state
                .db
                .update_repo(
                    repo.id,
                    req.description.as_deref(),
                    req.visibility.as_ref().map(|v| v.as_str()),
                    req.default_branch.as_deref(),
                )
                .await
            {
                Ok(updated) => {
                    let resp = repo_to_response(updated, Some(owner_name), &state);
                    (StatusCode::OK, Json(resp)).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CoreError::Database(e.to_string()).error_response()),
                )
                    .into_response(),
            }
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_repo(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_permission(
        &state,
        &auth,
        Resource::Repository,
        Action::Delete,
        None,
        None,
        None,
    )
    .await
    {
        return rejection.into_response();
    }

    let owner_uuid = match Uuid::parse_str(&owner) {
        Ok(id) => id,
        Err(_) => match state.db.get_user_by_username(&owner).await {
            Ok(user) => user.id,
            Err(_) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(CoreError::NotFound("owner user not found".into()).error_response()),
                )
                    .into_response();
            }
        },
    };

    match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(repo) => {
            if let Err(e) = state.db.delete_repo(repo.id).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CoreError::Database(e.to_string()).error_response()),
                )
                    .into_response();
            }
            (StatusCode::NO_CONTENT, ()).into_response()
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_commits(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let owner_uuid = match Uuid::parse_str(&owner) {
        Ok(id) => id,
        Err(_) => match state.db.get_user_by_username(&owner).await {
            Ok(user) => user.id,
            Err(_) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(CoreError::NotFound("owner user not found".into()).error_response()),
                )
                    .into_response();
            }
        },
    };
    match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(_) => {
            let git_svc =
                crate::git::GitService::new(std::path::PathBuf::from(&state.config.storage_path));
            match git_svc.list_commits(&owner, &name, 50) {
                Ok(commits) => (StatusCode::OK, Json(commits)).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CoreError::Git(e.to_string()).error_response()),
                )
                    .into_response(),
            }
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_branches(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let repo_path = state.git_service.repo_path(&owner, &name);

    if !repo_path.join("HEAD").exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response();
    }

    let repo = match gix::open(&repo_path) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let refs = match repo.references() {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    let head_ref = repo.head_ref().ok().flatten().and_then(|r| {
        let name = r.name().shorten().to_string();
        name.strip_prefix("refs/heads/").map(|s| s.to_string())
    });

    let mut branches = Vec::new();
    let all_refs = match refs.all() {
        Ok(iter) => iter,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    for reference in all_refs.flatten() {
        let full_name = reference.name().shorten().to_string();
        if full_name.starts_with("refs/heads/") {
            let branch_name = full_name.strip_prefix("refs/heads/").unwrap().to_string();
            let is_default = head_ref.as_ref() == Some(&branch_name);
            branches.push(BranchResponse {
                name: branch_name,
                is_default,
            });
        }
    }

    (StatusCode::OK, Json(branches)).into_response()
}

pub async fn list_tags(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let repo_path = state.git_service.repo_path(&owner, &name);

    if !repo_path.join("HEAD").exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response();
    }

    let repo = match gix::open(&repo_path) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let refs = match repo.references() {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    let mut tags = Vec::new();
    let all_refs = match refs.all() {
        Ok(iter) => iter,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    for reference in all_refs.flatten() {
        let full_name = reference.name().shorten().to_string();
        if full_name.starts_with("refs/tags/") {
            let tag_name = full_name.strip_prefix("refs/tags/").unwrap().to_string();
            tags.push(TagResponse { name: tag_name });
        }
    }

    (StatusCode::OK, Json(tags)).into_response()
}

pub async fn raw_file(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(params): Query<RawFileParams>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let repo_path = state.git_service.repo_path(&owner, &name);

    if !repo_path.join("HEAD").exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response();
    }

    let repo = match gix::open(&repo_path) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let ref_name = params.ref_.as_deref().unwrap_or("HEAD");

    let commit_id = match repo.rev_parse_single(ref_name) {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(
                    CoreError::Git(format!("cannot resolve ref {ref_name}: {e}")).error_response(),
                ),
            )
                .into_response();
        }
    };

    let commit_obj = match commit_id.object() {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let commit = match commit_obj.try_into_commit() {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git("cannot parse commit object".into()).error_response()),
            )
                .into_response();
        }
    };

    let tree = match commit.tree() {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    let entry = match tree.lookup_entry_by_path(&params.path) {
        Ok(Some(e)) => e,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(
                    CoreError::NotFound(format!("path not found: {}", params.path))
                        .error_response(),
                ),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    if !entry.mode().is_blob() {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::BadRequest("path is not a file".into()).error_response()),
        )
            .into_response();
    }

    let blob_obj = match entry.object() {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    let blob = match blob_obj.try_into_blob() {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(git_err(e).error_response()),
            )
                .into_response();
        }
    };

    let filename = std::path::Path::new(&params.path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| params.path.clone());

    let body = axum::body::Body::from(blob.data.to_vec());
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/octet-stream")
        .header(
            "content-disposition",
            format!("attachment; filename=\"{filename}\""),
        )
        .body(body)
        .unwrap();
    response.into_response()
}

pub async fn archive(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(params): Query<ArchiveParams>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let owner_uuid = match Uuid::parse_str(&owner) {
        Ok(id) => id,
        Err(_) => match state.db.get_user_by_username(&owner).await {
            Ok(user) => user.id,
            Err(_) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(CoreError::NotFound("user not found".into()).error_response()),
                )
                    .into_response();
            }
        },
    };

    if state
        .db
        .get_repo_by_owner_name(owner_uuid, &name)
        .await
        .is_err()
    {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response();
    }

    let format = params.format.as_deref().unwrap_or("zip");
    let git_format = match format {
        "zip" => "zip",
        "tar.gz" => "tar.gz",
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(
                    CoreError::BadRequest("format must be 'zip' or 'tar.gz'".into())
                        .error_response(),
                ),
            )
                .into_response();
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

    // Determine the reference to archive (default branch or specified ref)
    let git_ref = params.ref_.as_deref().unwrap_or("HEAD");

    // Use git archive subprocess to generate the archive
    let output = tokio::process::Command::new("git")
        .arg("archive")
        .arg("--format=zip") // git always outputs zip format for pipe; tar.gz via tar
        .arg(git_ref)
        .arg("--prefix") // empty prefix — no directory prefix
        .arg("--")
        .current_dir(&repo_path)
        .output()
        .await;

    let archive_data = match output {
        Ok(out) if out.status.success() => out.stdout,
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(format!("git archive failed: {stderr}")).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    CoreError::Internal(format!("failed to run git archive: {e}")).error_response(),
                ),
            )
                .into_response();
        }
    };

    // git archive --format=zip always produces zip regardless of the requested format
    // For tar.gz we need to pipe through gzip
    let (final_data, content_type, extension) = if format == "tar.gz" {
        // Pipe zip through: git archive zip → python3 unzip → tar → gzip
        // Simpler: just re-run git archive with tar format + gzip
        let tar_output = tokio::process::Command::new("git")
            .arg("archive")
            .arg(format!("--format=tar.{git_format}"))
            .arg(git_ref)
            .arg("--prefix")
            .arg("--")
            .current_dir(&repo_path)
            .output()
            .await;

        match tar_output {
            Ok(out) if out.status.success() => (out.stdout, "application/gzip", "tar.gz"),
            _ => {
                // Fallback: compress the zip data with gzip as-is
                let gz_result = gzip_data(&archive_data);
                match gz_result {
                    Ok(gz) => (gz, "application/gzip", "tar.gz"),
                    Err(e) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(
                                CoreError::Internal(format!("gzip compression failed: {e}"))
                                    .error_response(),
                            ),
                        )
                            .into_response();
                    }
                }
            }
        }
    } else {
        (archive_data, "application/zip", "zip")
    };

    let filename = format!("{name}-{git_ref}.{extension}");
    #[allow(clippy::useless_conversion)]
    let body = axum::body::Body::from(final_data);
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(
            "content-disposition",
            format!("attachment; filename=\"{filename}\""),
        )
        .header("content-type", content_type)
        .body(body)
        .unwrap();
    response.into_response()
}

async fn resolve_owner(
    state: &AppState,
    owner: &str,
) -> std::result::Result<(Uuid, String), Response> {
    match Uuid::parse_str(owner) {
        Ok(id) => {
            let uname = state
                .db
                .get_user_by_id(id)
                .await
                .map(|u| u.username)
                .unwrap_or_else(|_| id.to_string());
            Ok((id, uname))
        }
        Err(_) => match state.db.get_user_by_username(owner).await {
            Ok(user) => Ok((user.id, user.username)),
            Err(_) => Err((
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("user not found".into()).error_response()),
            )
                .into_response()),
        },
    }
}

pub async fn star_repo(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
) -> impl IntoResponse {
    let (owner_uuid, owner_name) = match resolve_owner(&state, &owner).await {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    let repo = match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
    };

    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());

    let (new_count, starred) = match state.db.toggle_star(user_id, repo.id).await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let dispatcher = crate::webhooks::WebhookDispatcher::new();
    let pool = state.db.pool().clone();
    let rid = repo.id;
    let evt = crate::webhooks::WebhookEvent::Star;
    let pl = serde_json::json!({
        "action": if starred { "starred" } else { "unstarred" },
        "repo_id": rid.to_string(),
        "stars_count": new_count,
    });
    tokio::spawn(async move { dispatcher.dispatch(&pool, rid, &evt, pl).await });

    // Deliver federation activity to followers
    if state.config.federation_enabled && starred {
        let domain = &state.config.federation_instance_domain;
        let activity = crate::federation::activitypub::Activity {
            r#type: crate::federation::activitypub::ActivityType::Like,
            id: format!("https://{domain}/activities/{}", uuid::Uuid::new_v4()),
            actor: format!("https://{domain}/api/v1/users/{}", auth.user_id),
            object: crate::federation::activitypub::ActivityObject::Repository {
                id: repo.id.to_string(),
                name: repo.name.clone(),
                attributed_to: owner_name.clone(),
            },
            target: None,
            published: chrono::Utc::now().to_rfc3339(),
            to: vec![format!("https://{domain}/api/v1/federation/actor")],
            cc: vec![],
        };
        crate::api::federation_routes::deliver_to_followers(activity, state.db.pool().clone())
            .await;
    }

    let resp = StarToggleResponse {
        starred,
        stars_count: new_count,
    };
    (StatusCode::OK, Json(resp)).into_response()
}

pub async fn starred_repo(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
) -> impl IntoResponse {
    let (owner_uuid, _owner_name) = match resolve_owner(&state, &owner).await {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    let repo = match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
    };

    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());
    let starred = state.db.has_user_starred(user_id, repo.id).await.unwrap_or(false);

    let resp = StarredResponse { starred };
    (StatusCode::OK, Json(resp)).into_response()
}

pub async fn watch_repo(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
) -> impl IntoResponse {
    let (owner_uuid, _owner_name) = match resolve_owner(&state, &owner).await {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    let repo = match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
    };

    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());

    let (new_count, watched) = match state.db.toggle_watch(user_id, repo.id).await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let resp = WatchToggleResponse {
        watched,
        watchers_count: new_count,
    };
    (StatusCode::OK, Json(resp)).into_response()
}

pub async fn watched_repo(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
) -> impl IntoResponse {
    let (owner_uuid, _owner_name) = match resolve_owner(&state, &owner).await {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    let repo = match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
    };

    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());
    let watched = state.db.has_user_watched(user_id, repo.id).await.unwrap_or(false);

    let resp = WatchedResponse { watched };
    (StatusCode::OK, Json(resp)).into_response()
}

pub async fn fork_repo(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
) -> impl IntoResponse {
    let (owner_uuid, _owner_name) = match resolve_owner(&state, &owner).await {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    let repo = match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
    };

    let forker_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());
    let forker_name = state
        .db
        .get_user_by_id(forker_id)
        .await
        .map(|u| u.username)
        .unwrap_or_else(|_| forker_id.to_string());

    let fork_name = format!("{name}-fork-{forker_name}");

    match state
        .db
        .create_fork(
            &fork_name,
            &repo.description,
            forker_id,
            repo.id,
            &repo.visibility,
            &repo.default_branch,
        )
        .await
    {
        Ok(forked) => {
            let dispatcher = crate::webhooks::WebhookDispatcher::new();
            let pool = state.db.pool().clone();
            let rid = repo.id;
            let evt = crate::webhooks::WebhookEvent::Fork;
            let pl = serde_json::json!({
                "action": "forked",
                "repo_id": rid.to_string(),
                "fork_id": forked.id.to_string(),
                "forker": forker_name,
            });
            tokio::spawn(async move { dispatcher.dispatch(&pool, rid, &evt, pl).await });

            // Deliver federation activity to followers
            if state.config.federation_enabled {
                let domain = &state.config.federation_instance_domain;
                let activity = crate::federation::activitypub::Activity {
                    r#type: crate::federation::activitypub::ActivityType::Create,
                    id: format!("https://{domain}/activities/{}", uuid::Uuid::new_v4()),
                    actor: format!("https://{domain}/api/v1/users/{}", auth.user_id),
                    object: crate::federation::activitypub::ActivityObject::Repository {
                        id: forked.id.to_string(),
                        name: forked.name.clone(),
                        attributed_to: forker_name.clone(),
                    },
                    target: None,
                    published: chrono::Utc::now().to_rfc3339(),
                    to: vec![format!("https://{domain}/api/v1/federation/actor")],
                    cc: vec![],
                };
                crate::api::federation_routes::deliver_to_followers(
                    activity,
                    state.db.pool().clone(),
                )
                .await;
            }

            let resp = repo_to_response(forked, Some(forker_name), &state);
            (StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_forks(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let (owner_uuid, _owner_name) = match resolve_owner(&state, &owner).await {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    let repo = match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.list_forks(repo.id).await {
        Ok(forks) => {
            let out = repos_to_responses(&state, forks).await;
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_collaborators(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let (owner_uuid, owner_name) = match resolve_owner(&state, &owner).await {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    let repo = match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
    };

    // Owner is always first
    let mut collabs = vec![CollaboratorResponse {
        user_id: owner_uuid.to_string(),
        username: owner_name,
        role: "owner".into(),
    }];

    // Fetch additional collaborators from table
    let rows = sqlx::query_as::<_, (uuid::Uuid, String, String)>(
        "SELECT rc.user_id, u.username, rc.permission
         FROM repo_collaborators rc
         JOIN users u ON u.id = rc.user_id
         WHERE rc.repo_id = $1
         ORDER BY rc.added_at",
    )
    .bind(repo.id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for (user_id, username, permission) in rows {
        collabs.push(CollaboratorResponse {
            user_id: user_id.to_string(),
            username,
            role: permission,
        });
    }

    (StatusCode::OK, Json(collabs)).into_response()
}

#[derive(serde::Deserialize)]
pub struct AddCollaboratorRequest {
    pub username: String,
    pub permission: Option<String>,
}

pub async fn add_collaborator(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Json(req): Json<AddCollaboratorRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let owner_uuid = match resolve_owner(&state, &owner).await {
        Ok((id, _name)) => id,
        Err(resp) => return resp.into_response(),
    };

    let repo = match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
    };

    // Resolve target user
    let target_user = match state.db.get_user_by_username(&req.username).await {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("target user not found".into()).error_response()),
            )
                .into_response();
        }
    };

    let permission = req.permission.as_deref().unwrap_or("read");

    // Check if already a collaborator
    let existing: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM repo_collaborators WHERE repo_id = $1 AND user_id = $2",
    )
    .bind(repo.id)
    .bind(target_user.id)
    .fetch_one(pool)
    .await
    .ok();

    if existing.map(|(c,)| c > 0).unwrap_or(false) {
        return (
            StatusCode::CONFLICT,
            Json(CoreError::BadRequest("user is already a collaborator".into()).error_response()),
        )
            .into_response();
    }

    // Insert collaborator
    match sqlx::query(
        "INSERT INTO repo_collaborators (repo_id, user_id, permission) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
    )
    .bind(repo.id)
    .bind(target_user.id)
    .bind(permission)
    .execute(pool)
    .await
    {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"status": "added"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn remove_collaborator(
    State(state): State<AppState>,
    Path((owner, name, user_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let owner_uuid = match resolve_owner(&state, &owner).await {
        Ok((id, _name)) => id,
        Err(resp) => return resp.into_response(),
    };

    let repo = match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
    };

    let collaborator_uuid = match Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid user_id format".into()).error_response()),
            )
                .into_response();
        }
    };

    match sqlx::query("DELETE FROM repo_collaborators WHERE repo_id = $1 AND user_id = $2")
        .bind(repo.id)
        .bind(collaborator_uuid)
        .execute(pool)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "removed"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct SetTopicsRequest {
    pub topics: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct TopicsResponse {
    pub topics: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TransferRepoRequest {
    pub new_owner: String,
}

#[derive(Debug, Deserialize)]
pub struct ArchiveRepoRequest {
    pub archived: bool,
}

#[derive(Debug, Deserialize)]
pub struct SetDefaultBranchRequest {
    pub branch: String,
}

#[derive(Debug, Deserialize)]
pub struct AdminRepoListParams {
    pub search: Option<String>,
    #[serde(default = "default_admin_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_admin_limit() -> i64 {
    50
}

pub async fn get_topics(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let (owner_uuid, _) = match resolve_owner(&state, &owner).await {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    let repo = match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.get_repo_topics(repo.id).await {
        Ok(topics) => (StatusCode::OK, Json(TopicsResponse { topics })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn set_topics(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<SetTopicsRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_permission(
        &state,
        &auth,
        Resource::Repository,
        Action::Update,
        None,
        None,
        None,
    )
    .await
    {
        return rejection.into_response();
    }

    let (owner_uuid, _) = match resolve_owner(&state, &owner).await {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    let repo = match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.set_repo_topics(repo.id, &req.topics).await {
        Ok(()) => (StatusCode::OK, Json(TopicsResponse { topics: req.topics })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn transfer_repo(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<TransferRepoRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_permission(
        &state,
        &auth,
        Resource::Repository,
        Action::Delete,
        None,
        None,
        None,
    )
    .await
    {
        return rejection.into_response();
    }

    let (owner_uuid, _) = match resolve_owner(&state, &owner).await {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    let repo = match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
    };

    let new_owner_id = match Uuid::parse_str(&req.new_owner) {
        Ok(id) => id,
        Err(_) => match state.db.get_user_by_username(&req.new_owner).await {
            Ok(user) => user.id,
            Err(_) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(CoreError::NotFound("new owner not found".into()).error_response()),
                )
                    .into_response();
            }
        },
    };

    match state.db.transfer_repo(repo.id, new_owner_id).await {
        Ok(updated) => {
            let new_owner_name = state
                .db
                .get_user_by_id(new_owner_id)
                .await
                .map(|u| u.username)
                .unwrap_or_else(|_| new_owner_id.to_string());
            let resp = repo_to_response(updated, Some(new_owner_name), &state);
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn archive_repo(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<ArchiveRepoRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_permission(
        &state,
        &auth,
        Resource::Repository,
        Action::Update,
        None,
        None,
        None,
    )
    .await
    {
        return rejection.into_response();
    }

    let (owner_uuid, owner_name) = match resolve_owner(&state, &owner).await {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    let repo = match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.set_repo_archived(repo.id, req.archived).await {
        Ok(updated) => {
            let resp = repo_to_response(updated, Some(owner_name), &state);
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn set_default_branch(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<SetDefaultBranchRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_permission(
        &state,
        &auth,
        Resource::Repository,
        Action::Update,
        None,
        None,
        None,
    )
    .await
    {
        return rejection.into_response();
    }

    let (owner_uuid, owner_name) = match resolve_owner(&state, &owner).await {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    let repo = match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
    };

    let repo_path = state.git_service.repo_path(&owner, &name);
    if !repo_path.join("HEAD").exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("git repository not found on disk".into()).error_response()),
        )
            .into_response();
    }

    let git_repo = match gix::open(&repo_path) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Git(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let ref_name = format!("refs/heads/{}", req.branch);
    let branch_exists = git_repo.find_reference(&ref_name).is_ok();

    if !branch_exists {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                CoreError::BadRequest(format!("branch '{}' does not exist", req.branch))
                    .error_response(),
            ),
        )
            .into_response();
    }

    match state.db.set_default_branch(repo.id, &req.branch).await {
        Ok(updated) => {
            let resp = repo_to_response(updated, Some(owner_name), &state);
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn admin_list_repos(
    State(state): State<AppState>,
    Query(params): Query<AdminRepoListParams>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let limit = params.limit.clamp(1, 100);
    match state
        .db
        .admin_list_repos(params.search.as_deref(), limit, params.offset)
        .await
    {
        Ok(repos) => {
            let out = repos_to_responses(&state, repos).await;
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn admin_delete_repo(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let owner_uuid = match Uuid::parse_str(&owner) {
        Ok(id) => id,
        Err(_) => match state.db.get_user_by_username(&owner).await {
            Ok(user) => user.id,
            Err(_) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(CoreError::NotFound("owner user not found".into()).error_response()),
                )
                    .into_response();
            }
        },
    };

    match state.db.get_repo_by_owner_name(owner_uuid, &name).await {
        Ok(repo) => {
            if let Err(e) = state.db.delete_repo(repo.id).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CoreError::Database(e.to_string()).error_response()),
                )
                    .into_response();
            }
            (StatusCode::NO_CONTENT, ()).into_response()
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response(),
    }
}

pub async fn admin_ban_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let uid = match Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid user id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.ban_user(uid).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({"banned": true}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn admin_unban_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let uid = match Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid user id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.unban_user(uid).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({"banned": false}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub fn repo_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/repos/{owner}/{name}/branches", get(list_branches))
        .route("/api/v1/repos/{owner}/{name}/tags", get(list_tags))
        .route("/api/v1/repos/{owner}/{name}/raw", get(raw_file))
        .route("/api/v1/repos/{owner}/{name}/archive", get(archive))
        .route("/api/v1/repos/{owner}/{name}/star", post(star_repo))
        .route("/api/v1/repos/{owner}/{name}/starred", get(starred_repo))
        .route("/api/v1/repos/{owner}/{name}/watch", post(watch_repo))
        .route("/api/v1/repos/{owner}/{name}/watched", get(watched_repo))
        .route("/api/v1/repos/{owner}/{name}/fork", post(fork_repo))
        .route("/api/v1/repos/{owner}/{name}/forks", get(list_forks))
        .route(
            "/api/v1/repos/{owner}/{name}/collaborators",
            get(list_collaborators).post(add_collaborator),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/collaborators/{user_id}",
            delete(remove_collaborator),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/topics",
            get(get_topics).put(set_topics),
        )
        .route("/api/v1/repos/{owner}/{name}/transfer", post(transfer_repo))
        .route(
            "/api/v1/repos/{owner}/{name}/archive-toggle",
            post(archive_repo),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/default-branch",
            patch(set_default_branch),
        )
        .route("/api/v1/admin/repos", get(admin_list_repos))
        .route(
            "/api/v1/admin/repos/{owner}/{name}",
            delete(admin_delete_repo),
        )
        .route("/api/v1/admin/users/{id}/ban", patch(admin_ban_user))
        .route("/api/v1/admin/users/{id}/unban", patch(admin_unban_user))
}

/// Compress data using gzip (flate2).
fn gzip_data(data: &[u8]) -> std::io::Result<Vec<u8>> {
    use flate2::write::GzEncoder;
    use std::io::Write;
    let mut encoder = GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(data)?;
    let compressed = encoder.finish()?;
    Ok(compressed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_request_visibility_deserialization() {
        let v: RequestVisibility = serde_json::from_str("\"public\"").unwrap();
        assert_eq!(v, RequestVisibility::Public);
        let v: RequestVisibility = serde_json::from_str("\"private\"").unwrap();
        assert_eq!(v, RequestVisibility::Private);
    }

    #[test]
    fn test_create_repo_request_parse() {
        let json =
            r#"{"name":"my-repo","owner":"myorg","description":"A repo","visibility":"public"}"#;
        let req: CreateRepoRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "my-repo");
        assert_eq!(req.owner, "myorg");
        assert_eq!(req.visibility, RequestVisibility::Public);
    }

    #[test]
    fn test_update_repo_request_parse() {
        let json = r#"{"description":"updated","visibility":"private"}"#;
        let req: UpdateRepoRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.description, Some("updated".into()));
        assert_eq!(req.visibility, Some(RequestVisibility::Private));
        assert!(req.name.is_none());
        assert!(req.default_branch.is_none());
    }

    #[test]
    fn test_update_repo_request_all_fields() {
        let json = r#"{"name":"new-name","description":"d","visibility":"public","default_branch":"develop"}"#;
        let req: UpdateRepoRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, Some("new-name".into()));
        assert_eq!(req.default_branch, Some("develop".into()));
    }

    #[test]
    fn test_update_repo_request_empty() {
        let json = r#"{}"#;
        let req: UpdateRepoRequest = serde_json::from_str(json).unwrap();
        assert!(req.name.is_none());
        assert!(req.description.is_none());
        assert!(req.visibility.is_none());
        assert!(req.default_branch.is_none());
    }

    #[test]
    fn test_branch_response_serialization() {
        let b = BranchResponse {
            name: "main".into(),
            is_default: true,
        };
        let json = serde_json::to_string(&b).unwrap();
        assert!(json.contains("main"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_tag_response_serialization() {
        let t = TagResponse {
            name: "v1.0".into(),
        };
        let json = serde_json::to_string(&t).unwrap();
        assert!(json.contains("v1.0"));
    }

    #[test]
    fn test_repo_response_serialization() {
        use chrono::{TimeZone, Utc};
        let repo = RepoResponse {
            id: uuid::Uuid::nil().into(),
            name: "test".into(),
            full_name: "owner/test".into(),
            description: Some("desc".into()),
            visibility: Visibility::Private,
            owner_id: uuid::Uuid::nil().into(),
            org_id: None,
            default_branch: "main".into(),
            is_fork: false,
            parent_repo_id: None,
            ssh_clone_url: Some("ssh://git@host/owner/test.git".into()),
            http_clone_url: Some("http://host:8080/owner/test.git".into()),
            created_at: Utc.timestamp_opt(0, 0).unwrap(),
            updated_at: Utc.timestamp_opt(0, 0).unwrap(),
        };
        let json = serde_json::to_string(&repo).unwrap();
        assert!(json.contains("owner/test"));
        assert!(json.contains("private"));
    }

    #[test]
    fn test_pagination_defaults() {
        let p = PaginationParams::default();
        assert_eq!(p.effective_per_page(), 20);
        assert_eq!(p.effective_offset(), 0);
    }

    #[test]
    fn test_pagination_params_deserialize() {
        let p: PaginationParams = serde_json::from_str("{}").unwrap();
        assert_eq!(p.per_page, None);
        assert_eq!(p.effective_per_page(), 20);
        let p: PaginationParams = serde_json::from_str(r#"{"per_page":10,"page":2}"#).unwrap();
        assert_eq!(p.per_page, Some(10));
        assert_eq!(p.effective_offset(), 10);
    }
}
