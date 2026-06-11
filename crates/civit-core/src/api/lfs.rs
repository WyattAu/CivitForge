//! Git LFS API endpoints.

#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, OptionalAuthUser, require_permission};
use crate::error::CoreError;
use axum::{
    Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
};
use civit_shared::permissions::{Action, Resource};
use civit_storage::lfs::{self, LfsBatchRequest, LfsBatchResponse, LfsObjectRef, LfsObjectResponse, LfsActions, LfsAction, LfsVerifyResponse};
use sha2::{Digest, Sha256};
use uuid::Uuid;

fn lfs_object_path(state: &AppState, oid: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(&state.config.storage_path)
        .join("lfs")
        .join(oid)
}

async fn resolve_repo(
    state: &AppState,
    owner: &str,
    name: &str,
) -> Result<(Uuid, String), Response> {
    let owner_uuid = match Uuid::parse_str(owner) {
        Ok(id) => {
            let uname = state.db.get_user_by_id(id).await.map(|u| u.username).unwrap_or_else(|_| id.to_string());
            (id, uname)
        }
        Err(_) => match state.db.get_user_by_username(owner).await {
            Ok(user) => (user.id, user.username),
            Err(_) => {
                return Err((StatusCode::NOT_FOUND, Json(CoreError::NotFound("user not found".into()).error_response())).into_response());
            }
        },
    };

    match state.db.get_repo_by_owner_name(owner_uuid.0, name).await {
        Ok(repo) => Ok((repo.id, owner_uuid.1)),
        Err(_) => Err((StatusCode::NOT_FOUND, Json(CoreError::NotFound("repository not found".into()).error_response())).into_response()),
    }
}

pub async fn batch_api(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: OptionalAuthUser,
    Json(req): Json<LfsBatchRequest>,
) -> impl IntoResponse {
    let repo_id = match resolve_repo(&state, &owner, &name).await {
        Ok((id, _)) => id,
        Err(resp) => return resp,
    };

    let pool = state.db.pool();
    lfs::ensure_lfs_table(pool).await;

    let base_url = format!("http://{}:{}", state.config.host, state.config.port);

    let mut object_exists = std::collections::HashMap::new();
    for obj in &req.objects {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM lfs_objects WHERE repo_id = $1 AND oid = $2)",
        )
        .bind(repo_id).bind(&obj.oid).fetch_one(pool).await.unwrap_or(false);
        object_exists.insert(obj.oid.clone(), exists);
    }

    let objects: Vec<LfsObjectResponse> = req.objects.iter().map(|obj| {
        let exists = *object_exists.get(&obj.oid).unwrap_or(&false);
        let actions = match req.operation.as_str() {
            "download" => {
                if exists {
                    let href = format!("{base_url}/api/v1/repos/{owner}/{name}/lfs/objects/{oid}", oid = obj.oid);
                    Some(LfsActions { download: Some(LfsAction { href, header: None, expires_in: Some(86400) }), upload: None, verify: None })
                } else {
                    Some(LfsActions { download: None, upload: None, verify: None })
                }
            }
            "upload" => {
                if exists {
                    Some(LfsActions { download: None, upload: None, verify: None })
                } else {
                    let href = format!("{base_url}/api/v1/repos/{owner}/{name}/lfs/objects/{oid}", oid = obj.oid);
                    let verify_href = format!("{base_url}/api/v1/repos/{owner}/{name}/lfs/verify");
                    Some(LfsActions { download: None, upload: Some(LfsAction { href, header: None, expires_in: Some(86400) }), verify: Some(LfsAction { href: verify_href, header: None, expires_in: Some(86400) }) })
                }
            }
            _ => None,
        };
        LfsObjectResponse { oid: obj.oid.clone(), size: obj.size, actions, error: None }
    }).collect();

    let resp = LfsBatchResponse { transfer: "basic".into(), objects, hash_algo: "sha256".into() };

    let mut headers = HeaderMap::new();
    headers.insert("content-type", HeaderValue::from_static("application/vnd.git-lfs+json"));
    headers.insert("x-content-sha256", HeaderValue::from_static("REQUIRE"));

    (StatusCode::OK, headers, Json(resp)).into_response()
}

pub async fn get_object(
    State(state): State<AppState>,
    Path((owner, name, oid)): Path<(String, String, String)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let _repo = match resolve_repo(&state, &owner, &name).await {
        Ok((id, _)) => id,
        Err(resp) => return resp,
    };

    let path = lfs_object_path(&state, &oid);
    if !path.exists() {
        return (StatusCode::NOT_FOUND, Json(CoreError::NotFound("LFS object not found".into()).error_response())).into_response();
    }

    let data = match tokio::fs::read(&path).await {
        Ok(d) => d,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(CoreError::Internal(format!("failed to read LFS object: {e}")).error_response())).into_response();
        }
    };

    let size = data.len() as u64;
    let mut headers = HeaderMap::new();
    headers.insert("content-type", HeaderValue::from_static("application/octet-stream"));
    headers.insert("content-length", HeaderValue::from_str(&size.to_string()).unwrap());

    (StatusCode::OK, headers, axum::body::Body::from(data)).into_response()
}

pub async fn put_object(
    State(state): State<AppState>,
    Path((owner, name, oid)): Path<(String, String, String)>,
    auth: AuthUser,
    headers: HeaderMap,
    body: axum::body::Body,
) -> impl IntoResponse {
    if let Err(rejection) = require_permission(&state, &auth, Resource::Repository, Action::Update, None, None, None).await {
        return rejection.into_response();
    }

    let repo_id = match resolve_repo(&state, &owner, &name).await {
        Ok((id, _)) => id,
        Err(resp) => return resp,
    };

    let content_length = headers.get("content-length").and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);

    if content_length > 100 * 1024 * 1024 {
        return (StatusCode::PAYLOAD_TOO_LARGE, Json(CoreError::BadRequest("LFS object too large (max 100MB)".into()).error_response())).into_response();
    }

    let data = match axum::body::to_bytes(body, 100 * 1024 * 1024).await {
        Ok(b) => b.to_vec(),
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(CoreError::BadRequest(format!("failed to read body: {e}")).error_response())).into_response();
        }
    };

    let mut hasher = Sha256::new();
    hasher.update(&data);
    let computed_oid = hex::encode(hasher.finalize());

    if computed_oid != oid {
        return (StatusCode::UNPROCESSABLE_ENTITY, Json(CoreError::BadRequest(format!("OID mismatch: expected {oid}, got {computed_oid}")).error_response())).into_response();
    }

    let lfs_dir = std::path::PathBuf::from(&state.config.storage_path).join("lfs");
    let _ = tokio::fs::create_dir_all(&lfs_dir).await;

    let path = lfs_dir.join(&oid);
    if let Err(e) = tokio::fs::write(&path, &data).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(CoreError::Internal(format!("failed to write LFS object: {e}")).error_response())).into_response();
    }

    let pool = state.db.pool();
    let _ = lfs::ensure_lfs_table(pool).await;

    let storage_path = path.to_string_lossy().to_string();
    let size = data.len() as i64;

    let _ = sqlx::query(
        r#"INSERT INTO lfs_objects (repo_id, oid, size, storage_path, verified)
           VALUES ($1, $2, $3, $4, true)
           ON CONFLICT (repo_id, oid) DO UPDATE SET
               size = EXCLUDED.size,
               storage_path = EXCLUDED.storage_path,
               verified = true"#,
    )
    .bind(repo_id).bind(&oid).bind(size).bind(&storage_path).execute(pool).await;

    (StatusCode::OK, Json(serde_json::json!({"oid": oid, "size": size}))).into_response()
}

pub async fn verify_upload(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<LfsObjectRef>,
) -> impl IntoResponse {
    if let Err(rejection) = require_permission(&state, &auth, Resource::Repository, Action::Update, None, None, None).await {
        return rejection.into_response();
    }

    let _repo_id = match resolve_repo(&state, &owner, &name).await {
        Ok((id, _)) => id,
        Err(resp) => return resp,
    };

    let pool = state.db.pool();
    let _ = lfs::ensure_lfs_table(pool).await;

    let path = lfs_object_path(&state, &req.oid);
    if !path.exists() {
        return (StatusCode::NOT_FOUND, Json(LfsVerifyResponse { oid: req.oid, size: 0, error: Some("object not found".into()) })).into_response();
    }

    let data = match tokio::fs::read(&path).await {
        Ok(d) => d,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(LfsVerifyResponse { oid: req.oid, size: 0, error: Some(format!("read failed: {e}")) })).into_response();
        }
    };

    let oid = req.oid.clone();
    let size = req.size;

    if data.len() as u64 != size {
        return (StatusCode::UNPROCESSABLE_ENTITY, Json(LfsVerifyResponse { oid, size: data.len() as u64, error: Some(format!("size mismatch: expected {size}, got {}", data.len())) })).into_response();
    }

    let mut hasher = Sha256::new();
    hasher.update(&data);
    let computed_oid = hex::encode(hasher.finalize());

    if computed_oid != oid {
        return (StatusCode::UNPROCESSABLE_ENTITY, Json(LfsVerifyResponse { oid: oid.clone(), size: data.len() as u64, error: Some(format!("oid mismatch: expected {oid}, computed {computed_oid}")) })).into_response();
    }

    let _ = sqlx::query("UPDATE lfs_objects SET verified = true WHERE repo_id = $1 AND oid = $2")
        .bind(_repo_id).bind(&oid).execute(pool).await;

    (StatusCode::OK, Json(LfsVerifyResponse { oid, size, error: None })).into_response()
}

pub fn lfs_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/repos/{owner}/{name}/lfs/objects/batch", post(batch_api))
        .route("/api/v1/repos/{owner}/{name}/lfs/objects/{oid}", get(get_object).put(put_object))
        .route("/api/v1/repos/{owner}/{name}/lfs/verify", post(verify_upload))
}
