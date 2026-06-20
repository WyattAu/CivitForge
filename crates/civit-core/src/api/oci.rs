//! OCI Container Registry API endpoints.

#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_admin};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use civit_storage::oci::{
    self, CatalogParams, CompleteUploadParams, CreatePolicy, OCI_API_VERSION, ReferrersParams,
    RegistryListParams, TagsParams, UploadChunkParams,
};
use sha2::{Digest as _, Sha256};

pub async fn version_check() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert("oci-api-version", HeaderValue::from_static(OCI_API_VERSION));
    (StatusCode::OK, headers)
}

pub async fn catalog(
    State(state): State<AppState>,
    Query(params): Query<CatalogParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let n = params.n.unwrap_or(100).min(1000);
    let last = params.last.as_deref().unwrap_or("");

    let rows = sqlx::query_as::<_, (i64, String, String, String)>(
        "SELECT id, name, namespace_type, visibility FROM oci_repositories WHERE name > $1 ORDER BY name LIMIT $2",
    )
    .bind(last)
    .bind(n as i64)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let repos: Vec<String> = rows.iter().map(|r| r.1.clone()).collect();

    (
        StatusCode::OK,
        Json(serde_json::json!({ "repositories": repos })),
    )
        .into_response()
}

pub async fn list_tags(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(params): Query<TagsParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match oci::resolve_repo(pool, &name).await {
        Some(r) => r,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"errors": [{"code": "NAME_UNKNOWN", "message": "repository not found"}]})),
            ).into_response();
        }
    };

    let n = params.n.unwrap_or(100).min(1000);
    let last = params.last.as_deref().unwrap_or("");

    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT name FROM oci_tags WHERE repo_id = $1 AND name > $2 ORDER BY name LIMIT $3",
    )
    .bind(repo_id)
    .bind(last)
    .bind(n as i64)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let tags: Vec<String> = rows.into_iter().map(|r| r.0).collect();
    (
        StatusCode::OK,
        Json(serde_json::json!({ "name": name, "tags": tags })),
    )
        .into_response()
}

pub async fn head_blob(
    State(state): State<AppState>,
    Path((name, digest)): Path<(String, String)>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match oci::resolve_repo(pool, &name).await {
        Some(r) => r,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let row = sqlx::query_as::<_, (i64, String)>(
        "SELECT size_bytes, media_type FROM oci_blobs WHERE repo_id = $1 AND digest = $2",
    )
    .bind(repo_id)
    .bind(&digest)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    match row {
        Some((size, media_type)) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                "content-length",
                size.to_string()
                    .parse()
                    .unwrap_or(HeaderValue::from_static("0")),
            );
            headers.insert(
                "content-type",
                HeaderValue::from_str(&media_type)
                    .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
            );
            headers.insert(
                "docker-content-digest",
                HeaderValue::from_str(&digest).unwrap_or_else(|_| HeaderValue::from_static("")),
            );
            (StatusCode::OK, headers).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn get_blob(
    State(state): State<AppState>,
    Path((name, digest)): Path<(String, String)>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match oci::resolve_repo(pool, &name).await {
        Some(r) => r,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let row = sqlx::query_as::<_, (String, i64, String)>(
        "SELECT storage_path, size_bytes, media_type FROM oci_blobs WHERE repo_id = $1 AND digest = $2",
    )
    .bind(repo_id)
    .bind(&digest)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    match row {
        Some((storage_path, size, media_type)) => {
            let data = match tokio::fs::read(&storage_path).await {
                Ok(d) => d,
                Err(_) => return StatusCode::NOT_FOUND.into_response(),
            };
            let mut headers = HeaderMap::new();
            headers.insert(
                "content-length",
                size.to_string()
                    .parse()
                    .unwrap_or(HeaderValue::from_static("0")),
            );
            headers.insert(
                "content-type",
                HeaderValue::from_str(&media_type)
                    .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
            );
            headers.insert(
                "docker-content-digest",
                HeaderValue::from_str(&digest).unwrap_or_else(|_| HeaderValue::from_static("")),
            );
            (StatusCode::OK, headers, axum::body::Body::from(data)).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn initiate_blob_upload(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let _repo_id = oci::resolve_or_create_repo(pool, &name).await;
    let session_id = uuid::Uuid::new_v4().to_string();

    let mut headers = HeaderMap::new();
    headers.insert(
        "location",
        HeaderValue::from_str(&format!("/v2/{name}/blobs/uploads/{session_id}"))
            .unwrap_or_else(|_| HeaderValue::from_static("/v2/blobs/uploads/")),
    );
    headers.insert(
        "oci-upload-uuid",
        HeaderValue::from_str(&session_id).unwrap_or_else(|_| HeaderValue::from_static("")),
    );
    headers.insert("range", HeaderValue::from_static("0-0"));

    (
        StatusCode::ACCEPTED,
        headers,
        Json(serde_json::json!({ "session_id": session_id })),
    )
        .into_response()
}

pub async fn upload_blob_chunk(
    State(_state): State<AppState>,
    Path((name, session_id)): Path<(String, String)>,
    Query(params): Query<UploadChunkParams>,
    _body: axum::body::Bytes,
) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        "location",
        HeaderValue::from_str(&format!("/v2/{name}/blobs/uploads/{session_id}"))
            .unwrap_or_else(|_| HeaderValue::from_static("/v2/blobs/uploads/")),
    );
    let range_start = params.range_start.unwrap_or(0);
    headers.insert(
        "range",
        HeaderValue::from_str(&format!("{range_start}-{range_start}"))
            .unwrap_or_else(|_| HeaderValue::from_static("0-0")),
    );
    (StatusCode::ACCEPTED, headers).into_response()
}

pub async fn complete_blob_upload(
    State(state): State<AppState>,
    Path((name, session_id)): Path<(String, String)>,
    Query(params): Query<CompleteUploadParams>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let _ = session_id;
    let pool = state.db.pool();
    let repo_id = oci::resolve_or_create_repo(pool, &name).await;

    let digest = match &params.digest {
        Some(d) => d.clone(),
        None => {
            let hash = Sha256::digest(&body);
            format!("sha256:{hash:x}")
        }
    };

    let media_type = params
        .content_type
        .as_deref()
        .unwrap_or("application/octet-stream");

    let blob_dir = format!("/var/lib/civitforge/oci/blobs/{}", &digest[7..12]);
    let blob_path = format!("{}/{}", blob_dir, digest.replace(':', "_"));

    let _ = tokio::fs::create_dir_all(&blob_dir).await;
    if let Err(e) = tokio::fs::write(&blob_path, &body).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"errors": [{"code": "BLOB_UPLOAD_INVALID", "message": e.to_string()}]}))).into_response();
    }

    let result = sqlx::query(
        "INSERT INTO oci_blobs (repo_id, digest, media_type, size_bytes, storage_path) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (repo_id, digest) DO NOTHING",
    )
    .bind(repo_id)
    .bind(&digest)
    .bind(media_type)
    .bind(body.len() as i64)
    .bind(&blob_path)
    .execute(pool)
    .await;

    match result {
        Ok(_) => {
            let mut headers = HeaderMap::new();
            headers.insert("location", HeaderValue::from_str(&format!("/v2/{name}/blobs/{digest}")).unwrap_or_else(|_| HeaderValue::from_static("/v2/blobs/sha256:")));
            headers.insert("docker-content-digest", HeaderValue::from_str(&digest).unwrap_or_else(|_| HeaderValue::from_static("sha256:")));
            (StatusCode::CREATED, headers).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"errors": [{"code": "BLOB_UPLOAD_INVALID", "message": e.to_string()}]}))).into_response(),
    }
}

pub async fn put_manifest(
    State(state): State<AppState>,
    Path((name, reference)): Path<(String, String)>,
    _headers: HeaderMap,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = oci::resolve_or_create_repo(pool, &name).await;

    let manifest_value: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"errors": [{"code": "MANIFEST_INVALID", "message": e.to_string()}]}))).into_response();
        }
    };

    let media_type = manifest_value
        .get("mediaType")
        .and_then(|v| v.as_str())
        .unwrap_or("application/vnd.oci.image.manifest.v1+json");
    let digest = oci::compute_digest(&body);

    let config_digest = manifest_value
        .get("config")
        .and_then(|v| v.get("digest"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let config_size = manifest_value
        .get("config")
        .and_then(|v| v.get("size"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    if let Err(e) = sqlx::query(
        "INSERT INTO oci_manifests (repo_id, digest, media_type, raw_json, config_digest, config_size) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (repo_id, digest) DO UPDATE SET raw_json = EXCLUDED.raw_json",
    )
    .bind(repo_id)
    .bind(&digest)
    .bind(media_type)
    .bind(body.to_vec())
    .bind(&config_digest)
    .bind(config_size)
    .execute(pool)
    .await
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"errors": [{"code": "MANIFEST_INVALID", "message": e.to_string()}]}))).into_response();
    }

    if let Some(layers) = manifest_value.get("layers").and_then(|v| v.as_array()) {
        for (i, layer) in layers.iter().enumerate() {
            let layer_digest = layer.get("digest").and_then(|v| v.as_str()).unwrap_or("");
            let layer_size = layer.get("size").and_then(|v| v.as_i64()).unwrap_or(0);
            let layer_media = layer
                .get("mediaType")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let _ = sqlx::query(
                "INSERT INTO oci_manifest_layers (manifest_id, blob_digest, blob_size, media_type, sort_order) VALUES ((SELECT id FROM oci_manifests WHERE repo_id = $1 AND digest = $2), $3, $4, $5, $6) ON CONFLICT DO NOTHING",
            )
            .bind(repo_id)
            .bind(&digest)
            .bind(layer_digest)
            .bind(layer_size)
            .bind(layer_media)
            .bind(i as i32)
            .execute(pool)
            .await;
        }
    }

    if !reference.starts_with("sha256:") {
        if let Some((immutable,)) = sqlx::query_as::<_, (bool,)>(
            "SELECT immutable FROM oci_tags WHERE repo_id = $1 AND name = $2",
        )
        .bind(repo_id)
        .bind(&reference)
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
            && immutable
        {
            return (StatusCode::FORBIDDEN, Json(serde_json::json!({"errors": [{"code": "TAG_IMMUTABLE", "message": "tag is immutable"}]}))).into_response();
        }

        let _ = sqlx::query(
            "INSERT INTO oci_tags (repo_id, name, manifest_digest) VALUES ($1, $2, $3) ON CONFLICT (repo_id, name) DO UPDATE SET manifest_digest = EXCLUDED.manifest_digest, updated_at = NOW()",
        )
        .bind(repo_id)
        .bind(&reference)
        .bind(&digest)
        .execute(pool)
        .await;
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        "location",
        HeaderValue::from_str(&format!("/v2/{name}/manifests/{digest}"))
            .unwrap_or_else(|_| HeaderValue::from_static("/v2/manifests/sha256:")),
    );
    headers.insert(
        "docker-content-digest",
        HeaderValue::from_str(&digest).unwrap_or_else(|_| HeaderValue::from_static("sha256:")),
    );
    (StatusCode::CREATED, headers).into_response()
}

pub async fn get_manifest(
    State(state): State<AppState>,
    Path((name, reference)): Path<(String, String)>,
    _headers: HeaderMap,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match oci::resolve_repo(pool, &name).await {
        Some(r) => r,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let digest = if reference.starts_with("sha256:") {
        reference.clone()
    } else {
        match sqlx::query_as::<_, (String,)>(
            "SELECT manifest_digest FROM oci_tags WHERE repo_id = $1 AND name = $2",
        )
        .bind(repo_id)
        .bind(&reference)
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
        {
            Some((d,)) => d,
            None => return StatusCode::NOT_FOUND.into_response(),
        }
    };

    let row = sqlx::query_as::<_, (Vec<u8>, String)>(
        "SELECT raw_json, media_type FROM oci_manifests WHERE repo_id = $1 AND digest = $2",
    )
    .bind(repo_id)
    .bind(&digest)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    match row {
        Some((raw_json, media_type)) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                "content-type",
                HeaderValue::from_str(&media_type).unwrap_or_else(|_| {
                    HeaderValue::from_static("application/vnd.oci.image.manifest.v1+json")
                }),
            );
            headers.insert(
                "docker-content-digest",
                HeaderValue::from_str(&digest)
                    .unwrap_or_else(|_| HeaderValue::from_static("sha256:")),
            );
            headers.insert(
                "content-length",
                (raw_json.len() as u64)
                    .to_string()
                    .parse()
                    .unwrap_or(HeaderValue::from_static("0")),
            );
            (StatusCode::OK, headers, axum::body::Body::from(raw_json)).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn delete_manifest(
    State(state): State<AppState>,
    Path((name, reference)): Path<(String, String)>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match oci::resolve_repo(pool, &name).await {
        Some(r) => r,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    if !reference.starts_with("sha256:") {
        let result = sqlx::query("DELETE FROM oci_tags WHERE repo_id = $1 AND name = $2")
            .bind(repo_id)
            .bind(&reference)
            .execute(pool)
            .await
            .unwrap_or_default();
        return if result.rows_affected() > 0 {
            StatusCode::ACCEPTED.into_response()
        } else {
            StatusCode::NOT_FOUND.into_response()
        };
    }

    let result = sqlx::query("DELETE FROM oci_manifests WHERE repo_id = $1 AND digest = $2")
        .bind(repo_id)
        .bind(&reference)
        .execute(pool)
        .await
        .unwrap_or_default();

    if result.rows_affected() > 0 {
        StatusCode::ACCEPTED.into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

pub async fn get_referrers(
    State(state): State<AppState>,
    Path((name, digest)): Path<(String, String)>,
    Query(params): Query<ReferrersParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match oci::resolve_repo(pool, &name).await {
        Some(r) => r,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"errors": []})),
            )
                .into_response();
        }
    };

    let artifact_type = params.artifact_type.as_deref().unwrap_or("");
    let digest_pattern = format!("%\"digest\":\"{digest}\"%");

    let rows = if artifact_type.is_empty() {
        sqlx::query_as::<_, (String, String, Vec<u8>)>(
            "SELECT m.digest, m.media_type, m.raw_json FROM oci_manifests m WHERE m.repo_id = $1 AND m.raw_json::text LIKE $2",
        )
        .bind(repo_id)
        .bind(&digest_pattern)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
    } else {
        let art_pattern = format!("%\"artifactType\":\"{artifact_type}\"%");
        sqlx::query_as::<_, (String, String, Vec<u8>)>(
            "SELECT m.digest, m.media_type, m.raw_json FROM oci_manifests m WHERE m.repo_id = $1 AND m.raw_json::text LIKE $2 AND m.raw_json::text LIKE $3",
        )
        .bind(repo_id)
        .bind(&digest_pattern)
        .bind(&art_pattern)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
    };

    let mut manifests = Vec::new();
    for (rd, mt, raw) in &rows {
        if let Ok(val) = serde_json::from_slice::<serde_json::Value>(raw) {
            manifests.push(serde_json::json!({
                "mediaType": mt,
                "digest": rd,
                "size": raw.len(),
                "annotations": val.get("annotations").cloned().unwrap_or(serde_json::json!({})),
            }));
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "schemaVersion": 2,
            "mediaType": "application/vnd.oci.image.index.v1+json",
            "manifests": manifests,
        })),
    )
        .into_response()
}

pub async fn list_repositories(
    State(state): State<AppState>,
    Query(params): Query<RegistryListParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    let rows = sqlx::query_as::<_, (i64, String, String, String)>(
        "SELECT id, name, namespace_type, visibility FROM oci_repositories r ORDER BY r.created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let repos: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({ "id": r.0, "name": r.1, "namespace_type": r.2, "visibility": r.3 })).collect();

    (
        StatusCode::OK,
        Json(serde_json::json!({ "repositories": repos })),
    )
        .into_response()
}

pub async fn get_repository(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match oci::resolve_repo(pool, &name).await {
        Some(r) => r,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let repo = sqlx::query_as::<_, (String, String, String, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
        "SELECT name, namespace_type, namespace_id, visibility, created_at, updated_at FROM oci_repositories WHERE id = $1",
    )
    .bind(repo_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    match repo {
        Some(r) => {
            let tag_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM oci_tags WHERE repo_id = $1")
                    .bind(repo_id)
                    .fetch_one(pool)
                    .await
                    .unwrap_or(0);
            let manifest_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM oci_manifests WHERE repo_id = $1")
                    .bind(repo_id)
                    .fetch_one(pool)
                    .await
                    .unwrap_or(0);
            let blob_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM oci_blobs WHERE repo_id = $1")
                    .bind(repo_id)
                    .fetch_one(pool)
                    .await
                    .unwrap_or(0);

            (StatusCode::OK, Json(serde_json::json!({
                "id": repo_id, "name": r.0, "namespace_type": r.1, "namespace_id": r.2, "visibility": r.3,
                "created_at": r.4, "updated_at": r.5, "tag_count": tag_count, "manifest_count": manifest_count, "blob_count": blob_count,
            }))).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn delete_repository(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(name): Path<String>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }
    let pool = state.db.pool();
    let repo_id = match oci::resolve_repo(pool, &name).await {
        Some(r) => r,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let result = sqlx::query("DELETE FROM oci_repositories WHERE id = $1")
        .bind(repo_id)
        .execute(pool)
        .await
        .unwrap_or_default();
    if result.rows_affected() > 0 {
        StatusCode::NO_CONTENT.into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

pub async fn set_policy(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(name): Path<String>,
    Json(policy): Json<CreatePolicy>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = oci::resolve_or_create_repo(pool, &name).await;

    match sqlx::query(
        "INSERT INTO oci_policies (repo_id, role, entity_type, entity_id) VALUES ($1, $2, $3, $4) ON CONFLICT (repo_id, role, entity_type, entity_id) DO NOTHING",
    )
    .bind(repo_id).bind(&policy.role).bind(&policy.entity_type).bind(&policy.entity_id)
    .execute(pool).await
    {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"status": "created"}))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"errors": [{"message": e.to_string()}]}))),
    }
}

pub async fn delete_policy(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path((name, entity_type, entity_id)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match oci::resolve_repo(pool, &name).await {
        Some(r) => r,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let result = sqlx::query(
        "DELETE FROM oci_policies WHERE repo_id = $1 AND entity_type = $2 AND entity_id = $3",
    )
    .bind(repo_id)
    .bind(&entity_type)
    .bind(&entity_id)
    .execute(pool)
    .await
    .unwrap_or_default();

    if result.rows_affected() > 0 {
        StatusCode::NO_CONTENT.into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

pub async fn trigger_gc(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(name): Path<String>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }
    let pool = state.db.pool();
    let repo_id = match oci::resolve_repo(pool, &name).await {
        Some(r) => r,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let _ = sqlx::query("DELETE FROM oci_manifests WHERE repo_id = $1 AND NOT EXISTS (SELECT 1 FROM oci_tags WHERE repo_id = $1 AND manifest_digest = digest)")
        .bind(repo_id).execute(pool).await;

    let orphaned = sqlx::query_as::<_, (String, String)>(
        "SELECT b.digest, b.storage_path FROM oci_blobs b WHERE b.repo_id = $1 AND NOT EXISTS (SELECT 1 FROM oci_manifest_layers ml WHERE ml.manifest_id IN (SELECT id FROM oci_manifests WHERE repo_id = $1) AND ml.blob_digest = b.digest)",
    )
    .bind(repo_id).fetch_all(pool).await.unwrap_or_default();

    let mut freed_bytes: i64 = 0;
    let mut freed_count: usize = 0;

    for (digest, path) in &orphaned {
        if let Ok(metadata) = tokio::fs::metadata(path).await {
            freed_bytes += metadata.len() as i64;
        }
        let _ = tokio::fs::remove_file(path).await;
        let _ = sqlx::query("DELETE FROM oci_blobs WHERE repo_id = $1 AND digest = $2")
            .bind(repo_id)
            .bind(digest)
            .execute(pool)
            .await;
        freed_count += 1;
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "freed_count": freed_count, "freed_bytes": freed_bytes })),
    )
        .into_response()
}

pub fn registry_routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/v2/", axum::routing::get(version_check))
        .route("/v2/_catalog", axum::routing::get(catalog))
        .route("/v2/{name}/tags/list", axum::routing::get(list_tags))
        .route(
            "/v2/{name}/blobs/uploads",
            axum::routing::post(initiate_blob_upload),
        )
        .route(
            "/v2/{name}/blobs/uploads/{uuid}",
            axum::routing::patch(upload_blob_chunk),
        )
        .route(
            "/v2/{name}/blobs/uploads/{uuid}",
            axum::routing::put(complete_blob_upload),
        )
        .route("/v2/{name}/blobs/{digest}", axum::routing::head(head_blob))
        .route("/v2/{name}/blobs/{digest}", axum::routing::get(get_blob))
        .route(
            "/v2/{name}/manifests/{reference}",
            axum::routing::put(put_manifest),
        )
        .route(
            "/v2/{name}/manifests/{reference}",
            axum::routing::get(get_manifest),
        )
        .route(
            "/v2/{name}/manifests/{reference}",
            axum::routing::delete(delete_manifest),
        )
        .route(
            "/v2/{name}/referrers/{digest}",
            axum::routing::get(get_referrers),
        )
        .route("/api/v1/registry", axum::routing::get(list_repositories))
        .route(
            "/api/v1/registry/{name}",
            axum::routing::get(get_repository),
        )
        .route(
            "/api/v1/registry/{name}",
            axum::routing::delete(delete_repository),
        )
        .route(
            "/api/v1/registry/{name}/policy",
            axum::routing::post(set_policy),
        )
        .route(
            "/api/v1/registry/{name}/policy/{entity_type}/{entity_id}",
            axum::routing::delete(delete_policy),
        )
        .route(
            "/api/v1/registry/{name}/gc",
            axum::routing::post(trigger_gc),
        )
}
