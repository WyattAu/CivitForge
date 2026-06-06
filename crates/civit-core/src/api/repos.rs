#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, OptionalAuthUser, require_permission};
use crate::error::CoreError;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post},
};
use civit_shared::permissions::{Action, Resource};
use civit_shared::repo::RepoResponse;
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
pub struct StarToggleResponse {
    pub starred: bool,
    pub stars_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WatchToggleResponse {
    pub watched: bool,
    pub watchers_count: i64,
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

fn repo_to_response(
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
        "http://{host}:{port}/{display_name}/{name}.git",
        host = state.config.host,
        port = state.config.port,
        name = r.name,
    ));
    let ssh_clone_url = Some(format!(
        "ssh://git@{host}/{display_name}/{name}.git",
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
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

async fn repos_to_responses(
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
    _auth: OptionalAuthUser,
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
            let resp = repo_to_response(repo, Some(owner_name), &state);
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
    if format != "zip" && format != "tar.gz" {
        return (
            StatusCode::BAD_REQUEST,
            Json(CoreError::BadRequest("format must be 'zip' or 'tar.gz'".into()).error_response()),
        )
            .into_response();
    }

    (
        StatusCode::NOT_IMPLEMENTED,
        Json(CoreError::Internal("archive generation not yet implemented".into()).error_response()),
    )
        .into_response()
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

    let _user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());

    let current_starred = repo.stars_count > 0;
    let (new_count, starred) = if current_starred {
        match state.db.decrement_stars(repo.id).await {
            Ok(c) => (c, false),
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CoreError::Database(e.to_string()).error_response()),
                )
                    .into_response();
            }
        }
    } else {
        match state.db.increment_stars(repo.id).await {
            Ok(c) => (c, true),
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CoreError::Database(e.to_string()).error_response()),
                )
                    .into_response();
            }
        }
    };

    let resp = StarToggleResponse {
        starred,
        stars_count: new_count,
    };
    (StatusCode::OK, Json(resp)).into_response()
}

pub async fn starred_repo(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
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

    let resp = StarredResponse {
        starred: repo.stars_count > 0,
    };
    (StatusCode::OK, Json(resp)).into_response()
}

pub async fn watch_repo(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
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

    let current_watched = repo.watchers_count > 0;
    let (new_count, watched) = if current_watched {
        match state.db.decrement_watchers(repo.id).await {
            Ok(c) => (c, false),
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CoreError::Database(e.to_string()).error_response()),
                )
                    .into_response();
            }
        }
    } else {
        match state.db.increment_watchers(repo.id).await {
            Ok(c) => (c, true),
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CoreError::Database(e.to_string()).error_response()),
                )
                    .into_response();
            }
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
    _auth: AuthUser,
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

    let resp = WatchedResponse {
        watched: repo.watchers_count > 0,
    };
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
    let (owner_uuid, owner_name) = match resolve_owner(&state, &owner).await {
        Ok(r) => r,
        Err(resp) => return resp,
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

    let collabs = vec![CollaboratorResponse {
        user_id: owner_uuid.to_string(),
        username: owner_name,
        role: "owner".into(),
    }];
    (StatusCode::OK, Json(collabs)).into_response()
}

pub async fn add_collaborator(
    State(_state): State<AppState>,
    Path((_owner, _name)): Path<(String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(CoreError::Internal("not yet implemented".into()).error_response()),
    )
        .into_response()
}

pub async fn remove_collaborator(
    State(_state): State<AppState>,
    Path((_owner, _name, _user_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(CoreError::Internal("not yet implemented".into()).error_response()),
    )
        .into_response()
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
