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
