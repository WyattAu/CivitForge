//! Artifact serving API endpoints.

#![forbid(unsafe_code)]

use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, head},
};
use civit_storage::artifacts::{self, ArtifactDownloadQuery};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

use super::AppState;
use crate::cache::pre_signed::{CacheHeaders, PreSignedUrlConfig, PreSignedUrlGenerator};

fn artifact_config_from_env() -> PreSignedUrlConfig {
    PreSignedUrlConfig {
        secret_key: std::env::var("ARTIFACT_SECRET_KEY")
            .unwrap_or_else(|_| "placeholder-dev-only-secret-key-do-not-use-in-prod".into()),
        base_url: std::env::var("ARTIFACT_BASE_URL").unwrap_or_else(|_| "https://localhost".into()),
        default_ttl_secs: 3600,
    }
}

pub fn artifact_serving_routes() -> Router<super::AppState> {
    Router::new()
        .route("/api/v1/artifacts/{owner}/{repo}/{artifact_id}/download", get(download_artifact))
        .route("/api/v1/artifacts/{owner}/{repo}/{artifact_id}/download-url", get(get_download_url))
        .route("/api/v1/artifacts/{owner}/{repo}/{artifact_id}", head(head_artifact))
        .route("/api/v1/artifacts/{owner}/{repo}/{artifact_id}/cache", delete(invalidate_cache))
}

async fn download_artifact(
    Path((owner, repo, artifact_id)): Path<(String, String, String)>,
    Query(query): Query<ArtifactDownloadQuery>,
    State(state): State<AppState>,
) -> Response {
    let config = artifact_config_from_env();
    let generator = PreSignedUrlGenerator::new(config);

    let token_str = match query.token {
        Some(t) => t,
        None => {
            return (StatusCode::UNAUTHORIZED, "pre-signed token required for download").into_response();
        }
    };

    let token = match generator.parse_token_from_bytes(&token_str) {
        Ok(t) => t,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "malformed token").into_response();
        }
    };

    if token.artifact_id != artifact_id {
        return (StatusCode::FORBIDDEN, "invalid token for this artifact").into_response();
    }

    match generator.validate_token(&token) {
        Ok(true) => {}
        Ok(false) => {
            return (StatusCode::FORBIDDEN, "token expired or signature invalid").into_response();
        }
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "token validation failed").into_response();
        }
    }

    let dir = artifacts::artifact_storage_path(&state.config.storage_path, &owner, &repo, &artifact_id);
    if !dir.exists() || !dir.is_dir() {
        return (StatusCode::NOT_FOUND, json!({ "error": "artifact not found" }).to_string()).into_response();
    }

    let files: Vec<PathBuf> = match std::fs::read_dir(&dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.is_file()).collect(),
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "failed to read artifact directory").into_response();
        }
    };

    if files.len() == 1 {
        let file_path = &files[0];
        let file_name = file_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| artifact_id.clone());

        match std::fs::read(file_path) {
            Ok(bytes) => {
                let cache_control = CacheHeaders::private_cache();
                let disposition = format!("attachment; filename=\"{file_name}\"");
                let content_type = artifacts::mime_from_extension(&file_name);

                (StatusCode::OK, [("cache-control", cache_control.as_str()), ("content-disposition", disposition.as_str()), ("content-type", content_type.as_str())], bytes).into_response()
            }
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to read artifact file").into_response(),
        }
    } else {
        let cache_control = CacheHeaders::private_cache();
        let disposition = format!("attachment; filename=\"{artifact_id}.tar.gz\"");

        let mut builder = tar::Builder::new(Vec::new());
        let mut has_files = false;
        for file_path in &files {
            if let Ok(_file) = std::fs::File::open(file_path) {
                let relative = file_path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                if builder.append_path_with_name(file_path, &relative).is_ok() {
                    has_files = true;
                }
            }
        }
        if !has_files {
            return (StatusCode::NOT_FOUND, json!({ "error": "artifact directory is empty" }).to_string()).into_response();
        }
        let archive_data = match builder.into_inner() {
            Ok(data) => data,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "failed to build archive").into_response(),
        };

        let content_type = "application/gzip".to_string();
        (StatusCode::OK, [("cache-control", cache_control.as_str()), ("content-disposition", disposition.as_str()), ("content-type", content_type.as_str())], archive_data).into_response()
    }
}

async fn get_download_url(
    Path((owner, repo, artifact_id)): Path<(String, String, String)>,
    State(state): State<AppState>,
) -> Response {
    let config = artifact_config_from_env();
    let generator = PreSignedUrlGenerator::new(config);

    let dir = artifacts::artifact_storage_path(&state.config.storage_path, &owner, &repo, &artifact_id);
    if !dir.exists() || !dir.is_dir() {
        return (StatusCode::NOT_FOUND, json!({ "error": "artifact not found" }).to_string()).into_response();
    }

    match generator.generate_url(&artifact_id, "authenticated-user", None) {
        Ok(url) => (StatusCode::OK, [("content-type", "application/json")], json!({ "download_url": url, "expires_in_secs": 3600 }).to_string()).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, json!({ "error": e.to_string() }).to_string()).into_response(),
    }
}

async fn head_artifact(
    Path((owner, repo, artifact_id)): Path<(String, String, String)>,
    State(state): State<AppState>,
) -> Response {
    let dir = artifacts::artifact_storage_path(&state.config.storage_path, &owner, &repo, &artifact_id);
    if !dir.exists() || !dir.is_dir() {
        return (StatusCode::NOT_FOUND, "").into_response();
    }

    let files: Vec<PathBuf> = match std::fs::read_dir(&dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.is_file()).collect(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "").into_response(),
    };

    if files.is_empty() {
        return (StatusCode::NOT_FOUND, "").into_response();
    }

    let mut hasher = Sha256::new();
    let mut total_size: u64 = 0;
    let mut last_modified: chrono::DateTime<chrono::Utc> = chrono::DateTime::default();

    for file_path in &files {
        if let Ok(bytes) = std::fs::read(file_path) {
            hasher.update(&bytes);
            total_size += bytes.len() as u64;
        }
        if let Ok(meta) = std::fs::metadata(file_path) {
            if let Ok(modified) = meta.modified() {
                let dt: chrono::DateTime<chrono::Utc> = modified.into();
                if dt > last_modified {
                    last_modified = dt;
                }
            }
        }
    }

    let etag = CacheHeaders::etag_from_hash(&format!("{:x}", hasher.finalize()));
    let cache_control = CacheHeaders::public_cache(86400);
    let last_modified_str = last_modified.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

    let headers = [
        ("etag", etag.as_str()),
        ("cache-control", cache_control.as_str()),
        ("content-type", "application/octet-stream"),
        ("content-length", &total_size.to_string()),
        ("last-modified", last_modified_str.as_str()),
        ("x-artifact-id", artifact_id.as_str()),
    ];
    (StatusCode::NO_CONTENT, headers).into_response()
}

async fn invalidate_cache(
    Path((owner, repo, artifact_id)): Path<(String, String, String)>,
    State(state): State<AppState>,
) -> Response {
    let dir = artifacts::artifact_storage_path(&state.config.storage_path, &owner, &repo, &artifact_id);
    let mut cleared = 0u64;

    if dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && path.extension().is_some_and(|e| e == "cache" || e == "tmp")
                    && std::fs::remove_file(&path).is_ok()
                {
                    cleared += 1;
                }
            }
        }
    }

    (StatusCode::OK, [("content-type", "application/json")], json!({
        "status": "ok",
        "artifact_id": artifact_id,
        "cache_invalidated": true,
        "entries_cleared": cleared,
    }).to_string()).into_response()
}
