//! npm registry compatible API endpoints.

#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct PublishNpmPackage {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub dist_tags: serde_json::Value,
    #[serde(default)]
    pub readme: String,
    #[serde(default)]
    pub dist: Option<NpmDist>,
}

#[derive(Debug, Deserialize)]
pub struct NpmDist {
    #[serde(default)]
    pub tarball: String,
    #[serde(default)]
    pub shasum: String,
    #[serde(default)]
    pub integrity: String,
}

#[derive(Debug, Serialize)]
pub struct NpmPackageInfo {
    pub name: String,
    pub description: String,
    #[serde(rename = "dist-tags")]
    pub dist_tags: serde_json::Value,
    pub versions: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct NpmVersionInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub dist: NpmDistInfo,
}

#[derive(Debug, Serialize)]
pub struct NpmDistInfo {
    pub tarball: String,
    pub shasum: String,
    pub integrity: String,
}

/// GET /api/v1/packages/npm/:name - Get package info (npm registry format)
pub async fn get_package(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let pool = state.db.pool();

    let rows = sqlx::query_as::<_, (Uuid, String, String, serde_json::Value, String)>(
        "SELECT id, name, version, dist_tags, description FROM npm_packages WHERE name = $1 ORDER BY created_at DESC",
    )
    .bind(&name)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if rows.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "package not found"})),
        )
            .into_response();
    }

    let description = rows[0].4.clone();
    let dist_tags = rows[0].3.clone();

    let versions: serde_json::Value = serde_json::json!(
        rows.iter().map(|r| {
            serde_json::json!({
                "name": r.1,
                "version": r.2,
                "dist": {
                    "tarball": "",
                    "shasum": "",
                    "integrity": ""
                }
            })
        }).collect::<Vec<_>>()
    );

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "name": name,
            "description": description,
            "dist-tags": dist_tags,
            "versions": versions,
        })),
    )
        .into_response()
}

/// GET /api/v1/packages/npm/:name/:version - Get version info
pub async fn get_package_version(
    State(state): State<AppState>,
    Path((name, version)): Path<(String, String)>,
) -> impl IntoResponse {
    let pool = state.db.pool();

    let row = sqlx::query_as::<_, (Uuid, String, String, serde_json::Value, String)>(
        "SELECT id, name, version, dist_tags, description FROM npm_packages WHERE name = $1 AND version = $2 LIMIT 1",
    )
    .bind(&name)
    .bind(&version)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    match row {
        Some((id, pkg_name, ver, _, description)) => {
            let ver_rows = sqlx::query_as::<_, (String, String, String)>(
                "SELECT tarball_url, shasum, integrity FROM npm_versions WHERE package_id = $1 AND version = $2 LIMIT 1",
            )
            .bind(id)
            .bind(&ver)
            .fetch_optional(pool)
            .await
            .unwrap_or(None);

            let dist = match ver_rows {
                Some((tarball, shasum, integrity)) => NpmDistInfo {
                    tarball,
                    shasum,
                    integrity,
                },
                None => NpmDistInfo {
                    tarball: String::new(),
                    shasum: String::new(),
                    integrity: String::new(),
                },
            };

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "name": pkg_name,
                    "version": ver,
                    "description": description,
                    "dist": dist,
                })),
            )
                .into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "version not found"})),
        )
            .into_response(),
    }
}

/// PUT /api/v1/packages/npm/:name - Publish package
pub async fn publish_package(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(name): Path<String>,
    Json(body): Json<PublishNpmPackage>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());

    // Find or create a repo for this user
    let repo_row = sqlx::query_as::<_, (Uuid,)>(
        "SELECT id FROM repositories WHERE owner_id = $1 AND name = $2 LIMIT 1",
    )
    .bind(user_id)
    .bind(format!("npm-{name}"))
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    let repo_id = match repo_row {
        Some((id,)) => id,
        None => {
            let new_id = uuid::Uuid::new_v4();
            let _ = sqlx::query(
                "INSERT INTO repositories (id, name, description, owner_id, visibility, default_branch) VALUES ($1, $2, $3, $4, 'public', 'main') ON CONFLICT DO NOTHING",
            )
            .bind(new_id)
            .bind(format!("npm-{name}"))
            .bind(format!("npm package: {name}"))
            .bind(user_id)
            .execute(pool)
            .await;
            new_id
        }
    };

    let pkg_id = uuid::Uuid::new_v4();
    let _ = sqlx::query(
        r#"INSERT INTO npm_packages (id, repo_id, name, version, description, dist_tags, readme)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           ON CONFLICT (repo_id, name, version)
           DO UPDATE SET description = $5, dist_tags = $6, readme = $7"#,
    )
    .bind(pkg_id)
    .bind(repo_id)
    .bind(&body.name)
    .bind(&body.version)
    .bind(&body.description)
    .bind(&body.dist_tags)
    .bind(&body.readme)
    .execute(pool)
    .await;

    if let Some(dist) = &body.dist {
        let _ = sqlx::query(
            "INSERT INTO npm_versions (package_id, version, tarball_url, shasum, integrity) VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING",
        )
        .bind(pkg_id)
        .bind(&body.version)
        .bind(&dist.tarball)
        .bind(&dist.shasum)
        .bind(&dist.integrity)
        .execute(pool)
        .await;
    }

    (
        StatusCode::CREATED,
        Json(serde_json::json!({"ok": true, "id": pkg_id})),
    )
        .into_response()
}

/// GET /api/v1/packages/npm/:name/-/:filename - Download tarball
pub async fn download_tarball(
    State(state): State<AppState>,
    Path((name, filename)): Path<(String, String)>,
) -> impl IntoResponse {
    let _ = (state, name, filename);
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"error": "tarball not found"})),
    )
        .into_response()
}

pub fn npm_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/packages/npm/{name}", get(get_package).put(publish_package))
        .route("/api/v1/packages/npm/{name}/{version}", get(get_package_version))
        .route(
            "/api/v1/packages/npm/{name}/-{filename}",
            get(download_tarball),
        )
}
