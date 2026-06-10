#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, OptionalAuthUser, require_permission};
use crate::error::CoreError;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, patch, post},
};
use civit_shared::permissions::{Action, Resource};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateMirrorRequest {
    pub url: String,
    #[serde(rename = "direction")]
    pub direction: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_sync_interval")]
    pub sync_interval_minutes: u32,
}

fn default_true() -> bool {
    true
}

fn default_sync_interval() -> u32 {
    60
}

#[derive(Debug, Deserialize)]
pub struct UpdateMirrorRequest {
    pub url: Option<String>,
    pub direction: Option<String>,
    pub enabled: Option<bool>,
    pub sync_interval_minutes: Option<u32>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct MirrorRecord {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub url: String,
    pub direction: String,
    pub enabled: bool,
    pub sync_interval_minutes: i32,
    pub last_sync_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_sync_status: Option<String>,
    pub last_sync_error: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl MirrorRecord {
    /// Returns the interval in minutes between sync attempts.
    pub fn sync_interval(&self) -> i32 {
        self.sync_interval_minutes
    }

    /// Returns the last successful sync timestamp, if any.
    pub fn last_sync(&self) -> Option<&chrono::DateTime<chrono::Utc>> {
        self.last_sync_at.as_ref()
    }

    /// Returns the repository that owns this mirror.
    pub fn repository_id(&self) -> Uuid {
        self.repo_id
    }
}

async fn ensure_mirrors_table(pool: &sqlx::postgres::PgPool) {
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS repo_mirrors (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
            url TEXT NOT NULL,
            direction TEXT NOT NULL CHECK (direction IN ('push', 'pull', 'both')),
            enabled BOOLEAN NOT NULL DEFAULT true,
            sync_interval_minutes INT NOT NULL DEFAULT 60,
            last_sync_at TIMESTAMPTZ,
            last_sync_status TEXT,
            last_sync_error TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await;
}

async fn resolve_repo(
    state: &AppState,
    owner: &str,
    name: &str,
) -> Result<(Uuid, String), Response> {
    let owner_uuid = match Uuid::parse_str(owner) {
        Ok(id) => {
            let uname = state
                .db
                .get_user_by_id(id)
                .await
                .map(|u| u.username)
                .unwrap_or_else(|_| id.to_string());
            (id, uname)
        }
        Err(_) => match state.db.get_user_by_username(owner).await {
            Ok(user) => (user.id, user.username),
            Err(_) => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(CoreError::NotFound("user not found".into()).error_response()),
                )
                    .into_response());
            }
        },
    };

    match state.db.get_repo_by_owner_name(owner_uuid.0, name).await {
        Ok(repo) => Ok((repo.id, owner_uuid.1)),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response()),
    }
}

pub async fn create_mirror(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<CreateMirrorRequest>,
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

    let repo_id = match resolve_repo(&state, &owner, &name).await {
        Ok((id, _)) => id,
        Err(resp) => return resp,
    };

    let direction = match req.direction.as_str() {
        "push" | "pull" | "both" => req.direction.clone(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(
                    CoreError::BadRequest("direction must be 'push', 'pull', or 'both'".into())
                        .error_response(),
                ),
            )
                .into_response();
        }
    };

    let pool = state.db.pool();
    ensure_mirrors_table(pool).await;

    let interval = req.sync_interval_minutes as i32;
    let row = sqlx::query_as::<_, MirrorRecord>(
        r#"INSERT INTO repo_mirrors (repo_id, url, direction, enabled, sync_interval_minutes)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING *"#,
    )
    .bind(repo_id)
    .bind(&req.url)
    .bind(&direction)
    .bind(req.enabled)
    .bind(interval)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
    });

    match row {
        Ok(mirror) => (StatusCode::CREATED, Json(mirror)).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn list_mirrors(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let repo_id = match resolve_repo(&state, &owner, &name).await {
        Ok((id, _)) => id,
        Err(resp) => return resp,
    };

    let pool = state.db.pool();
    ensure_mirrors_table(pool).await;

    let rows = sqlx::query_as::<_, MirrorRecord>(
        "SELECT * FROM repo_mirrors WHERE repo_id = $1 ORDER BY created_at DESC",
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
    });

    match rows {
        Ok(mirrors) => (StatusCode::OK, Json(mirrors)).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn update_mirror(
    State(state): State<AppState>,
    Path((owner, name, mirror_id)): Path<(String, String, String)>,
    auth: AuthUser,
    Json(req): Json<UpdateMirrorRequest>,
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

    let (_repo_id, _owner_name) = match resolve_repo(&state, &owner, &name).await {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    let mirror_uuid = match Uuid::parse_str(&mirror_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid mirror id".into()).error_response()),
            )
                .into_response();
        }
    };

    if let Some(ref dir) = req.direction {
        if !matches!(dir.as_str(), "push" | "pull" | "both") {
            return (
                StatusCode::BAD_REQUEST,
                Json(
                    CoreError::BadRequest("direction must be 'push', 'pull', or 'both'".into())
                        .error_response(),
                ),
            )
                .into_response();
        }
    }

    let pool = state.db.pool();

    let result = sqlx::query_as::<_, MirrorRecord>(
        r#"UPDATE repo_mirrors
           SET url = COALESCE($2, url),
               direction = COALESCE($3, direction),
               enabled = COALESCE($4, enabled),
               sync_interval_minutes = COALESCE($5, sync_interval_minutes),
               updated_at = NOW()
           WHERE id = $1
           RETURNING *"#,
    )
    .bind(mirror_uuid)
    .bind(req.url.as_deref())
    .bind(req.direction.as_deref())
    .bind(req.enabled)
    .bind(req.sync_interval_minutes.map(|v| v as i32))
    .fetch_one(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
    });

    match result {
        Ok(mirror) => (StatusCode::OK, Json(mirror)).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn delete_mirror(
    State(state): State<AppState>,
    Path((owner, name, mirror_id)): Path<(String, String, String)>,
    auth: AuthUser,
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

    let (_repo_id, _owner_name) = match resolve_repo(&state, &owner, &name).await {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    let mirror_uuid = match Uuid::parse_str(&mirror_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid mirror id".into()).error_response()),
            )
                .into_response();
        }
    };

    let pool = state.db.pool();
    let result = sqlx::query("DELETE FROM repo_mirrors WHERE id = $1")
        .bind(mirror_uuid)
        .execute(pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
        });

    match result {
        Ok(r) if r.rows_affected() > 0 => (StatusCode::NO_CONTENT, ()).into_response(),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("mirror not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn sync_mirror(
    State(state): State<AppState>,
    Path((owner, name, mirror_id)): Path<(String, String, String)>,
    auth: AuthUser,
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

    let (_repo_id, _owner_name) = match resolve_repo(&state, &owner, &name).await {
        Ok(r) => r,
        Err(resp) => return resp,
    };

    let mirror_uuid = match Uuid::parse_str(&mirror_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid mirror id".into()).error_response()),
            )
                .into_response();
        }
    };

    let pool = state.db.pool();
    let mirror: MirrorRecord =
        match sqlx::query_as::<_, MirrorRecord>("SELECT * FROM repo_mirrors WHERE id = $1")
            .bind(mirror_uuid)
            .fetch_optional(pool)
            .await
        {
            Ok(Some(m)) => m,
            Ok(None) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(CoreError::NotFound("mirror not found".into()).error_response()),
                )
                    .into_response();
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CoreError::Database(e.to_string()).error_response()),
                )
                    .into_response();
            }
        };

    // Validate the mirror belongs to this repo
    if mirror.repo_id != _repo_id {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("mirror not found in this repository".into()).error_response()),
        )
            .into_response();
    }

    let repo_path = state.git_service.repo_path(&owner, &name);
    if !repo_path.join("HEAD").exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("git repository not found on disk".into()).error_response()),
        )
            .into_response();
    }

    // Validate mirror fields and compute sync metadata
    if mirror.repository_id().is_nil() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Internal("mirror has no repo_id".into()).error_response()),
        )
            .into_response();
    }
    let sync_interval = mirror.sync_interval();
    let last_sync_str = match mirror.last_sync() {
        Some(dt) => dt.to_rfc3339(),
        None => "never".to_string(),
    };
    tracing::info!(interval = sync_interval, last = %last_sync_str, "mirror sync metadata");

    let sync_result = match mirror.direction.as_str() {
        "push" => {
            let output = tokio::process::Command::new("git")
                .args(["push", "--mirror", &mirror.url])
                .current_dir(&repo_path)
                .output()
                .await;
            handle_sync_output(output).await
        }
        "pull" => {
            let output = tokio::process::Command::new("git")
                .args(["fetch", "--all"])
                .current_dir(&repo_path)
                .output()
                .await;
            handle_sync_output(output).await
        }
        "both" => {
            let fetch_result = {
                let output = tokio::process::Command::new("git")
                    .args(["fetch", "--all"])
                    .current_dir(&repo_path)
                    .output()
                    .await;
                handle_sync_output(output).await
            };
            if fetch_result.is_err() {
                return fetch_result.into_response();
            }
            let output = tokio::process::Command::new("git")
                .args(["push", "--mirror", &mirror.url])
                .current_dir(&repo_path)
                .output()
                .await;
            handle_sync_output(output).await
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid mirror direction".into()).error_response()),
            )
                .into_response();
        }
    };

    let (status, error_msg) = match &sync_result {
        Ok(_) => ("success".to_string(), None),
        Err(e) => ("failed".to_string(), Some(e.clone())),
    };

    let _ = sqlx::query(
        r#"UPDATE repo_mirrors
           SET last_sync_at = NOW(),
               last_sync_status = $2,
               last_sync_error = $3,
               updated_at = NOW()
           WHERE id = $1"#,
    )
    .bind(mirror_uuid)
    .bind(&status)
    .bind(error_msg.as_deref())
    .execute(pool)
    .await;

    match sync_result {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "synced", "detail": output})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Git(format!("mirror sync failed: {e}")).error_response()),
        )
            .into_response(),
    }
}

async fn handle_sync_output(
    output: std::result::Result<std::process::Output, std::io::Error>,
) -> Result<String, String> {
    match output {
        Ok(out) => {
            if out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                Ok(stderr.to_string())
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                Err(stderr.to_string())
            }
        }
        Err(e) => Err(format!("failed to execute git: {e}")),
    }
}

pub fn mirror_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/mirrors",
            get(list_mirrors).post(create_mirror),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/mirrors/{mirror_id}",
            patch(update_mirror).delete(delete_mirror),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/mirrors/{mirror_id}/sync",
            post(sync_mirror),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_mirror_request_parse() {
        let json = r#"{"url":"https://github.com/example/repo.git","direction":"push","enabled":true,"sync_interval_minutes":30}"#;
        let req: CreateMirrorRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.url, "https://github.com/example/repo.git");
        assert_eq!(req.direction, "push");
        assert!(req.enabled);
        assert_eq!(req.sync_interval_minutes, 30);
    }

    #[test]
    fn test_create_mirror_request_defaults() {
        let json = r#"{"url":"https://example.com/repo.git","direction":"pull"}"#;
        let req: CreateMirrorRequest = serde_json::from_str(json).unwrap();
        assert!(req.enabled);
        assert_eq!(req.sync_interval_minutes, 60);
    }

    #[test]
    fn test_update_mirror_request_parse() {
        let json = r#"{"direction":"both","enabled":false}"#;
        let req: UpdateMirrorRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.direction, Some("both".into()));
        assert_eq!(req.enabled, Some(false));
        assert!(req.url.is_none());
    }

    #[test]
    fn test_update_mirror_request_empty() {
        let json = r#"{}"#;
        let req: UpdateMirrorRequest = serde_json::from_str(json).unwrap();
        assert!(req.url.is_none());
        assert!(req.direction.is_none());
        assert!(req.enabled.is_none());
        assert!(req.sync_interval_minutes.is_none());
    }

    #[test]
    fn test_default_true_fn() {
        assert!(default_true());
    }

    #[test]
    fn test_default_sync_interval_fn() {
        assert_eq!(default_sync_interval(), 60);
    }
}
