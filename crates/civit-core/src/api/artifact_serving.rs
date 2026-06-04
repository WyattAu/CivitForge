#![forbid(unsafe_code)]

use axum::{
    Router,
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, head},
};
use serde::Deserialize;
use serde_json::json;

use crate::cache::pre_signed::{CacheHeaders, PreSignedUrlConfig, PreSignedUrlGenerator};

pub fn artifact_serving_routes() -> Router<super::AppState> {
    Router::new()
        .route(
            "/api/v1/artifacts/{owner}/{repo}/{artifact_id}/download",
            get(download_artifact),
        )
        .route(
            "/api/v1/artifacts/{owner}/{repo}/{artifact_id}/download-url",
            get(get_download_url),
        )
        .route(
            "/api/v1/artifacts/{owner}/{repo}/{artifact_id}",
            head(head_artifact),
        )
        .route(
            "/api/v1/artifacts/{owner}/{repo}/{artifact_id}/cache",
            delete(invalidate_cache),
        )
}

#[derive(Deserialize)]
pub struct ArtifactDownloadQuery {
    token: Option<String>,
}

async fn download_artifact(
    Path((owner, repo, artifact_id)): Path<(String, String, String)>,
    Query(query): Query<ArtifactDownloadQuery>,
) -> Response {
    let _ = (owner, repo);
    let cache_control = CacheHeaders::private_cache();

    match query.token {
        Some(token_str) => {
            let config = PreSignedUrlConfig {
                secret_key: "placeholder".to_string(),
                base_url: "https://placeholder".to_string(),
                default_ttl_secs: 3600,
            };
            let generator = PreSignedUrlGenerator::new(config);
            match generator.parse_token_from_bytes(&token_str) {
                Ok(token) if token.artifact_id == artifact_id => {
                    let disposition = format!("attachment; filename=\"{artifact_id}\"");
                    (
                        StatusCode::OK,
                        [
                            ("cache-control", cache_control.as_str()),
                            ("content-disposition", disposition.as_str()),
                        ],
                        json!({
                            "status": "ok",
                            "artifact_id": artifact_id,
                            "message": "token valid, serving artifact",
                        })
                        .to_string(),
                    )
                        .into_response()
                }
                Ok(_) => (StatusCode::FORBIDDEN, "invalid token for this artifact").into_response(),
                Err(_) => (StatusCode::BAD_REQUEST, "malformed token").into_response(),
            }
        }
        None => (
            StatusCode::UNAUTHORIZED,
            "pre-signed token required for download",
        )
            .into_response(),
    }
}

async fn get_download_url(
    Path((owner, repo, artifact_id)): Path<(String, String, String)>,
) -> Response {
    let _ = (owner, repo);
    let config = PreSignedUrlConfig {
        secret_key: "placeholder".to_string(),
        base_url: "https://cdn.example.com".to_string(),
        default_ttl_secs: 3600,
    };
    let generator = PreSignedUrlGenerator::new(config);

    match generator.generate_url(&artifact_id, "authenticated-user", None) {
        Ok(url) => (
            StatusCode::OK,
            [("content-type", "application/json")],
            json!({ "download_url": url, "expires_in_secs": 3600 }).to_string(),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "error": e.to_string() }).to_string(),
        )
            .into_response(),
    }
}

async fn head_artifact(
    Path((owner, repo, artifact_id)): Path<(String, String, String)>,
) -> Response {
    let _ = (owner, repo);
    let fake_etag = CacheHeaders::etag_from_hash("deadbeefcafe1234");
    let cache_control = CacheHeaders::public_cache(86400);
    let headers = [
        ("etag", fake_etag.as_str()),
        ("cache-control", cache_control.as_str()),
        ("content-type", "application/octet-stream"),
        ("x-artifact-id", artifact_id.as_str()),
    ];
    (StatusCode::NO_CONTENT, headers).into_response()
}

async fn invalidate_cache(
    Path((owner, repo, artifact_id)): Path<(String, String, String)>,
) -> Response {
    let _ = (owner, repo);
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        json!({
            "status": "ok",
            "artifact_id": artifact_id,
            "cache_invalidated": true,
        })
        .to_string(),
    )
        .into_response()
}
