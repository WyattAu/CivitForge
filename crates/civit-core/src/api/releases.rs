#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Request/Response Types ---

#[derive(Debug, Deserialize)]
pub struct CreateReleaseRequest {
    pub tag_name: String,
    pub name: String,
    pub body: Option<String>,
    pub draft: Option<bool>,
    pub prerelease: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ReleaseResponse {
    pub id: String,
    pub repo_id: String,
    pub tag_name: String,
    pub name: String,
    pub body: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub author_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub published_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReleaseAssetResponse {
    pub id: String,
    pub release_id: String,
    pub name: String,
    pub content_type: String,
    pub size: i64,
    pub download_count: i64,
    pub author_id: String,
    pub created_at: String,
}

// --- Helpers ---

fn err_response(e: CoreError) -> axum::response::Response {
    let status = e.status_code();
    let body = e.error_response();
    (status, Json(body)).into_response()
}

async fn get_repo_id(pool: &sqlx::PgPool, owner: &str, name: &str) -> Result<Uuid, CoreError> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT r.id FROM repositories r JOIN users u ON r.owner_id = u.id WHERE u.username = $1 AND r.name = $2",
    )
    .bind(owner)
    .bind(name)
    .fetch_one(pool)
    .await
    .map_err(|_| CoreError::NotFound(format!("repo {owner}/{name}")))
}

// --- Handlers ---

pub async fn list_releases(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };

    match state.db.list_releases(repo_id).await {
        Ok(releases) => {
            let items: Vec<ReleaseResponse> = releases
                .into_iter()
                .map(|r| ReleaseResponse {
                    id: r.id.to_string(),
                    repo_id: r.repo_id.to_string(),
                    tag_name: r.tag_name,
                    name: r.name,
                    body: r.body,
                    draft: r.draft,
                    prerelease: r.prerelease,
                    author_id: r.author_id.to_string(),
                    created_at: r.created_at.to_rfc3339(),
                    updated_at: r.updated_at.to_rfc3339(),
                    published_at: r.published_at.map(|t| t.to_rfc3339()),
                })
                .collect();
            (StatusCode::OK, Json(items)).into_response()
        }
        Err(e) => err_response(e),
    }
}

pub async fn create_release(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<CreateReleaseRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };

    let author_id = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => return err_response(CoreError::BadRequest("invalid user id".into())),
    };

    if req.tag_name.trim().is_empty() {
        return err_response(CoreError::BadRequest("tag_name is required".into()));
    }

    let draft = req.draft.unwrap_or(false);
    let prerelease = req.prerelease.unwrap_or(false);

    match state
        .db
        .create_release(
            repo_id,
            &req.tag_name,
            &req.name,
            req.body.as_deref(),
            draft,
            prerelease,
            author_id,
        )
        .await
    {
        Ok(release) => {
            let resp = ReleaseResponse {
                id: release.id.to_string(),
                repo_id: release.repo_id.to_string(),
                tag_name: release.tag_name,
                name: release.name,
                body: release.body,
                draft: release.draft,
                prerelease: release.prerelease,
                author_id: release.author_id.to_string(),
                created_at: release.created_at.to_rfc3339(),
                updated_at: release.updated_at.to_rfc3339(),
                published_at: release.published_at.map(|t| t.to_rfc3339()),
            };
            (StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(e) => err_response(e),
    }
}

pub async fn get_release(
    State(state): State<AppState>,
    Path((owner, name, release_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let _repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };

    let id = match Uuid::parse_str(&release_id) {
        Ok(id) => id,
        Err(_) => return err_response(CoreError::BadRequest("invalid release id".into())),
    };

    match state.db.get_release(id).await {
        Ok(release) => {
            let resp = ReleaseResponse {
                id: release.id.to_string(),
                repo_id: release.repo_id.to_string(),
                tag_name: release.tag_name,
                name: release.name,
                body: release.body,
                draft: release.draft,
                prerelease: release.prerelease,
                author_id: release.author_id.to_string(),
                created_at: release.created_at.to_rfc3339(),
                updated_at: release.updated_at.to_rfc3339(),
                published_at: release.published_at.map(|t| t.to_rfc3339()),
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => err_response(e),
    }
}

pub async fn delete_release(
    State(state): State<AppState>,
    Path((owner, name, release_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let _repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };

    let id = match Uuid::parse_str(&release_id) {
        Ok(id) => id,
        Err(_) => return err_response(CoreError::BadRequest("invalid release id".into())),
    };

    match state.db.delete_release(id).await {
        Ok(()) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => err_response(e),
    }
}

pub async fn list_release_assets(
    State(state): State<AppState>,
    Path((owner, name, release_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let _repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };

    let id = match Uuid::parse_str(&release_id) {
        Ok(id) => id,
        Err(_) => return err_response(CoreError::BadRequest("invalid release id".into())),
    };

    match state.db.list_release_assets(id).await {
        Ok(assets) => {
            let items: Vec<ReleaseAssetResponse> = assets
                .into_iter()
                .map(|a| ReleaseAssetResponse {
                    id: a.id.to_string(),
                    release_id: a.release_id.to_string(),
                    name: a.name,
                    content_type: a.content_type,
                    size: a.size,
                    download_count: a.download_count,
                    author_id: a.author_id.to_string(),
                    created_at: a.created_at.to_rfc3339(),
                })
                .collect();
            (StatusCode::OK, Json(items)).into_response()
        }
        Err(e) => err_response(e),
    }
}

pub async fn create_release_asset(
    State(state): State<AppState>,
    Path((owner, name, release_id)): Path<(String, String, String)>,
    auth: AuthUser,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let _repo_id = match get_repo_id(pool, &owner, &name).await {
        Ok(id) => id,
        Err(e) => return err_response(e),
    };

    let id = match Uuid::parse_str(&release_id) {
        Ok(id) => id,
        Err(_) => return err_response(CoreError::BadRequest("invalid release id".into())),
    };

    let author_id = match Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => return err_response(CoreError::BadRequest("invalid user id".into())),
    };

    let asset_name = req.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let content_type = req
        .get("content_type")
        .and_then(|v| v.as_str())
        .unwrap_or("application/octet-stream");
    let size = req.get("size").and_then(|v| v.as_i64()).unwrap_or(0);

    if asset_name.is_empty() {
        return err_response(CoreError::BadRequest("name is required".into()));
    }

    match state
        .db
        .create_release_asset(id, asset_name, content_type, size, author_id)
        .await
    {
        Ok(asset) => {
            let resp = ReleaseAssetResponse {
                id: asset.id.to_string(),
                release_id: asset.release_id.to_string(),
                name: asset.name,
                content_type: asset.content_type,
                size: asset.size,
                download_count: asset.download_count,
                author_id: asset.author_id.to_string(),
                created_at: asset.created_at.to_rfc3339(),
            };
            (StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(e) => err_response(e),
    }
}

// --- Router ---

pub fn release_routes() -> Router<AppState> {
    use axum::routing::get;

    Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/releases",
            get(list_releases).post(create_release),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/releases/{id}",
            get(get_release).delete(delete_release),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/releases/{id}/assets",
            get(list_release_assets).post(create_release_asset),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_release_response_serialization() {
        let resp = ReleaseResponse {
            id: "00000000-0000-0000-0000-000000000001".into(),
            repo_id: "00000000-0000-0000-0000-000000000002".into(),
            tag_name: "v1.0.0".into(),
            name: "Release 1.0".into(),
            body: Some("First release".into()),
            draft: false,
            prerelease: false,
            author_id: "00000000-0000-0000-0000-000000000003".into(),
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
            published_at: Some("2024-01-01T00:00:00Z".into()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("v1.0.0"));
        assert!(json.contains("Release 1.0"));
    }

    #[test]
    fn test_release_asset_response_serialization() {
        let resp = ReleaseAssetResponse {
            id: "00000000-0000-0000-0000-000000000001".into(),
            release_id: "00000000-0000-0000-0000-000000000002".into(),
            name: "binary.tar.gz".into(),
            content_type: "application/gzip".into(),
            size: 1024,
            download_count: 0,
            author_id: "00000000-0000-0000-0000-000000000003".into(),
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("binary.tar.gz"));
    }

    #[test]
    fn test_create_release_request_parse() {
        let req: CreateReleaseRequest = serde_json::from_str(
            r#"{"tag_name":"v1.0","name":"Release 1.0","body":"First","draft":false,"prerelease":false}"#,
        )
        .unwrap();
        assert_eq!(req.tag_name, "v1.0");
        assert_eq!(req.name, "Release 1.0");
        assert!(!req.draft.unwrap_or(false));
    }

    #[test]
    fn test_release_routes_compile() {
        let router = release_routes();
        let _ = router;
    }
}
