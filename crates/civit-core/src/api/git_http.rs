#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, OptionalAuthUser, require_permission};
use axum::{
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use civit_shared::permissions::{Action, Resource};

pub async fn info_refs(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    if !state.git_service.repo_exists(&owner, &name) {
        return (StatusCode::NOT_FOUND, "repository not found").into_response();
    }

    let repo_path = state.git_service.repo_path(&owner, &name);
    match crate::git::http::info_refs(&repo_path, "upload-pack") {
        Ok(data) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                "application/x-git-upload-pack-advertisement",
            )
            .header(header::CACHE_CONTROL, "no-cache")
            .body(axum::body::Body::from(data))
            .unwrap_or_else(|_| {
                (StatusCode::INTERNAL_SERVER_ERROR, "response build error").into_response()
            }),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("git error: {e}")).into_response(),
    }
}

pub async fn upload_pack(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: OptionalAuthUser,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    if !state.git_service.repo_exists(&owner, &name) {
        return (StatusCode::NOT_FOUND, "repository not found").into_response();
    }

    let repo_path = state.git_service.repo_path(&owner, &name);
    match crate::git::http::upload_pack(&repo_path, &body) {
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
    auth: AuthUser,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    // Require push permission on the repository
    if let Err(rejection) = require_permission(
        &state,
        &auth,
        Resource::Repository,
        Action::Push,
        None,
        None,
    )
    .await
    {
        return rejection.into_response();
    }

    if !state.git_service.repo_exists(&owner, &name) {
        return (StatusCode::NOT_FOUND, "repository not found").into_response();
    }

    let repo_path = state.git_service.repo_path(&owner, &name);
    match crate::git::http::receive_pack(&repo_path, &body) {
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
