//! Maven repository compatible API endpoints.

#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, put},
    Router,
};
use uuid::Uuid;

/// GET /api/v1/packages/maven/:group/:artifact/:version/:artifact-:version.:packaging
pub async fn get_maven_artifact(
    State(state): State<AppState>,
    Path((group, artifact, version, file)): Path<(String, String, String, String)>,
) -> impl IntoResponse {
    let pool = state.db.pool();

    // Parse packaging from filename (e.g., "my-artifact-1.0.0.jar" -> "jar")
    let packaging = file.rsplit('.').next().unwrap_or("jar");

    // Find any repo that has this maven package
    let row = sqlx::query_as::<_, (Uuid, String, String, String, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT repo_id, group_id, artifact_id, version, packaging, created_at FROM maven_packages WHERE group_id = $1 AND artifact_id = $2 AND version = $3 LIMIT 1",
    )
    .bind(&group)
    .bind(&artifact)
    .bind(&version)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    match row {
        Some((_repo_id, _gid, _aid, _ver, pkg, _created)) => {
            let mut headers = HeaderMap::new();
            let content_type = match pkg.as_str() {
                "jar" => "application/java-archive",
                "pom" => "application/xml",
                "war" => "application/java-archive",
                _ => "application/octet-stream",
            };
            headers.insert(
                "content-type",
                content_type.parse().unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
            );
            (StatusCode::OK, headers, "Binary content placeholder".as_bytes().to_vec()).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "artifact not found"})),
        )
            .into_response(),
    }
}

/// PUT /api/v1/packages/maven/:group/:artifact/:version/:artifact-:version.:packaging
pub async fn publish_maven_artifact(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((group, artifact, version, file)): Path<(String, String, String, String)>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());
    let packaging = file.rsplit('.').next().unwrap_or("jar");

    // Find or create repo
    let repo_row = sqlx::query_as::<_, (Uuid,)>(
        "SELECT id FROM repositories WHERE owner_id = $1 AND name = $2 LIMIT 1",
    )
    .bind(user_id)
    .bind(&format!("maven-{group}-{artifact}"))
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
            .bind(&format!("maven-{group}-{artifact}"))
            .bind(format!("maven artifact: {group}:{artifact}"))
            .bind(user_id)
            .execute(pool)
            .await;
            new_id
        }
    };

    let pkg_id = uuid::Uuid::new_v4();
    let _ = sqlx::query(
        r#"INSERT INTO maven_packages (id, repo_id, group_id, artifact_id, version, packaging)
           VALUES ($1, $2, $3, $4, $5, $6)
           ON CONFLICT (repo_id, group_id, artifact_id, version)
           DO UPDATE SET packaging = $6"#,
    )
    .bind(pkg_id)
    .bind(repo_id)
    .bind(&group)
    .bind(&artifact)
    .bind(&version)
    .bind(packaging)
    .execute(pool)
    .await;

    let mut headers = HeaderMap::new();
    headers.insert("location", format!("/api/v1/packages/maven/{group}/{artifact}/{version}/{file}").parse().unwrap_or_else(|_| HeaderValue::from_static("/api/v1/packages/maven")));

    (StatusCode::CREATED, headers, Json(serde_json::json!({"ok": true}))).into_response()
}

pub fn maven_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/packages/maven/{group}/{artifact}/{version}/{file}",
            get(get_maven_artifact).put(publish_maven_artifact),
        )
}
