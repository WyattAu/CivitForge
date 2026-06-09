//! Pipeline schedule API endpoints (cron-triggered runs).

#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, OptionalAuthUser};
use crate::error::CoreError;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use chrono::Utc;
use civit_pipeline::compute_next_cron_run;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateScheduleRequest {
    pub cron: String,
    pub name: Option<String>,
    pub ref_name: Option<String>,
    #[serde(default = "default_yaml_path")]
    pub yaml_path: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_yaml_path() -> String {
    ".civit/pipeline.yaml".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct UpdateScheduleRequest {
    pub cron: Option<String>,
    pub name: Option<Option<String>>,
    pub ref_name: Option<Option<String>>,
    pub yaml_path: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleResponse {
    pub id: String,
    pub repo_id: String,
    pub cron: String,
    pub name: Option<String>,
    pub ref_name: Option<String>,
    pub yaml_path: String,
    pub enabled: bool,
    pub last_run_at: Option<String>,
    pub next_run_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualRunResponse {
    pub schedule_id: String,
    pub run_id: String,
    pub status: String,
    pub triggered_at: String,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn schedule_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/schedules",
            get(list_schedules).post(create_schedule),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/schedules/{id}",
            patch(update_schedule).delete(delete_schedule),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/schedules/{id}/run",
            post(manual_trigger),
        )
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// List scheduled triggers for a repository.
pub async fn list_schedules(
    State(state): State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    _auth: OptionalAuthUser,
) -> Response {
    let pool = state.db.pool();
    let repo_id = match resolve_repo_id(pool, &owner, &repo_name).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    match list_schedules_db(pool, repo_id).await {
        Ok(schedules) => (axum::http::StatusCode::OK, Json(schedules)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Create a new pipeline schedule.
pub async fn create_schedule(
    State(state): State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    _auth: AuthUser,
    Json(req): Json<CreateScheduleRequest>,
) -> Response {
    if !civit_pipeline::validate_cron(&req.cron) {
        return (
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            Json(CoreError::BadRequest("invalid cron expression".into()).error_response()),
        )
            .into_response();
    }

    let pool = state.db.pool();
    let repo_id = match resolve_repo_id(pool, &owner, &repo_name).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("repository not found".into()).error_response()),
            )
                .into_response();
        }
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let now = Utc::now();
    let next_run = compute_next_cron_run(&req.cron, &now);
    let schedule_id = Uuid::new_v4();

    match sqlx::query(
        "INSERT INTO pipeline_schedules (id, repo_id, cron, name, ref_name, yaml_path, enabled, next_run_at, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING id",
    )
    .bind(schedule_id)
    .bind(repo_id)
    .bind(&req.cron)
    .bind(&req.name)
    .bind(&req.ref_name)
    .bind(&req.yaml_path)
    .bind(req.enabled)
    .bind(next_run)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await
    {
        Ok(row) => {
            let id: Uuid = row.get("id");
            let resp = ScheduleResponse {
                id: id.to_string(),
                repo_id: repo_id.to_string(),
                cron: req.cron,
                name: req.name,
                ref_name: req.ref_name,
                yaml_path: req.yaml_path,
                enabled: req.enabled,
                last_run_at: None,
                next_run_at: next_run.map(|t| t.to_rfc3339()),
                created_at: now.to_rfc3339(),
                updated_at: now.to_rfc3339(),
            };
            (axum::http::StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Update an existing pipeline schedule.
pub async fn update_schedule(
    State(state): State<AppState>,
    Path((_owner, _repo_name, schedule_id)): Path<(String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<UpdateScheduleRequest>,
) -> Response {
    let pool = state.db.pool();
    let id = match Uuid::parse_str(&schedule_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid schedule ID".into()).error_response()),
            )
                .into_response();
        }
    };

    // Validate cron if provided
    if let Some(ref cron) = req.cron {
        if !civit_pipeline::validate_cron(cron) {
            return (
                axum::http::StatusCode::UNPROCESSABLE_ENTITY,
                Json(CoreError::BadRequest("invalid cron expression".into()).error_response()),
            )
                .into_response();
        }
    }

    let now = Utc::now();

    // Build dynamic update query
    let mut sets: Vec<String> = Vec::new();
    let mut idx = 1u32;

    if req.cron.is_some() {
        sets.push(format!("cron = ${idx}"));
        idx += 1;
    }
    if req.name.is_some() {
        sets.push(format!("name = ${idx}"));
        idx += 1;
    }
    if req.ref_name.is_some() {
        sets.push(format!("ref_name = ${idx}"));
        idx += 1;
    }
    if req.yaml_path.is_some() {
        sets.push(format!("yaml_path = ${idx}"));
        idx += 1;
    }
    if req.enabled.is_some() {
        sets.push(format!("enabled = ${idx}"));
        idx += 1;
    }

    if sets.is_empty() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(CoreError::BadRequest("no fields to update".into()).error_response()),
        )
            .into_response();
    }

    // Compute next_run_at if cron changed
    let next_run = req
        .cron
        .as_deref()
        .and_then(|c| compute_next_cron_run(c, &now));

    sets.push(format!("updated_at = ${idx}"));
    idx += 1;

    if next_run.is_some() {
        sets.push(format!("next_run_at = ${idx}"));
        idx += 1;
    }

    sets.push(format!(
        "id = ${idx} (SELECT id FROM pipeline_schedules WHERE id = ${idx})"
    ));

    let sql = format!(
        "UPDATE pipeline_schedules SET {} WHERE id = ${} RETURNING id, repo_id, cron, name, ref_name, yaml_path, enabled, last_run_at, next_run_at, created_at, updated_at",
        sets.join(", "),
        idx
    );

    let mut query = sqlx::query_as::<_, ScheduleRow>(&sql);

    if let Some(ref cron) = req.cron {
        query = query.bind(cron);
    }
    if let Some(ref name) = req.name {
        query = query.bind(name);
    }
    if let Some(ref ref_name) = req.ref_name {
        query = query.bind(ref_name);
    }
    if let Some(ref yaml_path) = req.yaml_path {
        query = query.bind(yaml_path);
    }
    if let Some(enabled) = req.enabled {
        query = query.bind(enabled);
    }
    query = query.bind(now);
    if let Some(nr) = next_run {
        query = query.bind(nr);
    }
    query = query.bind(id);

    match query.fetch_optional(pool).await {
        Ok(Some(row)) => (
            axum::http::StatusCode::OK,
            Json(ScheduleResponse::from(row)),
        )
            .into_response(),
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("schedule not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Delete a pipeline schedule.
pub async fn delete_schedule(
    State(state): State<AppState>,
    Path((_owner, _repo_name, schedule_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let id = match Uuid::parse_str(&schedule_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid schedule ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match sqlx::query("DELETE FROM pipeline_schedules WHERE id = $1 RETURNING id")
        .bind(id)
        .fetch_optional(pool)
        .await
    {
        Ok(Some(_)) => axum::http::StatusCode::NO_CONTENT.into_response(),
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("schedule not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Manually trigger a pipeline schedule (fire immediately).
pub async fn manual_trigger(
    State(state): State<AppState>,
    Path((_owner, _repo_name, schedule_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> Response {
    let pool = state.db.pool();
    let id = match Uuid::parse_str(&schedule_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid schedule ID".into()).error_response()),
            )
                .into_response();
        }
    };

    // Fetch the schedule
    let schedule: Option<ScheduleRow> = sqlx::query_as(
        "SELECT id, repo_id, cron, name, ref_name, yaml_path, enabled, last_run_at, next_run_at, created_at, updated_at FROM pipeline_schedules WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let schedule = match schedule {
        Some(s) => s,
        None => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(CoreError::NotFound("schedule not found".into()).error_response()),
            )
                .into_response();
        }
    };

    // Resolve repo owner/name
    let (owner, repo_name) = match get_repo_owner_name(pool, schedule.repo_id).await {
        Ok(names) => names,
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let ref_name = schedule
        .ref_name
        .clone()
        .unwrap_or_else(|| "main".to_string());
    let yaml_path = &schedule.yaml_path;

    // Read pipeline YAML
    let yaml_content = match read_pipeline_yaml_from_fs(
        &state.config.storage_path,
        &owner,
        &repo_name,
        &ref_name,
        yaml_path,
    ) {
        Some(content) => content,
        None => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(
                    CoreError::NotFound(format!(
                        "pipeline YAML not found at '{yaml_path}' for ref '{ref_name}'"
                    ))
                    .error_response(),
                ),
            )
                .into_response();
        }
    };

    // Parse and validate
    let pipeline = match civit_pipeline::parse_pipeline(&yaml_content) {
        Ok(p) => p,
        Err(e) => {
            return (
                axum::http::StatusCode::UNPROCESSABLE_ENTITY,
                Json(CoreError::BadRequest(format!("invalid pipeline YAML: {e}")).error_response()),
            )
                .into_response();
        }
    };

    if let Err(e) = civit_pipeline::validate_pipeline(&pipeline) {
        return (
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            Json(CoreError::BadRequest(format!("invalid pipeline: {e}")).error_response()),
        )
            .into_response();
    }

    let pipeline = match civit_pipeline::expand_matrix(&pipeline) {
        Ok(p) => p,
        Err(e) => {
            return (
                axum::http::StatusCode::UNPROCESSABLE_ENTITY,
                Json(
                    CoreError::BadRequest(format!("matrix expansion failed: {e}")).error_response(),
                ),
            )
                .into_response();
        }
    };

    // Get HEAD commit SHA
    let commit_sha = get_head_commit_sha(&state.config.storage_path, &owner, &repo_name, &ref_name)
        .unwrap_or_else(|| "0000000000000000000000000000000000000000".to_string());

    // Create pipeline run
    let run_id = match create_manual_pipeline_run(
        pool,
        schedule.repo_id,
        yaml_path,
        &ref_name,
        &commit_sha,
        &yaml_content,
        &pipeline,
    )
    .await
    {
        Ok(id) => id,
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CoreError::Database(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    let now = Utc::now();
    let resp = ManualRunResponse {
        schedule_id: schedule.id.to_string(),
        run_id: run_id.to_string(),
        status: "pending".to_string(),
        triggered_at: now.to_rfc3339(),
    };

    (axum::http::StatusCode::CREATED, Json(resp)).into_response()
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

async fn resolve_repo_id(
    pool: &sqlx::PgPool,
    owner: &str,
    repo_name: &str,
) -> std::result::Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_as::<_, (Uuid,)>(
        "SELECT r.id FROM repositories r JOIN users u ON r.owner_id = u.id WHERE u.username = $1 AND r.name = $2",
    )
    .bind(owner)
    .bind(repo_name)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}

async fn list_schedules_db(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
) -> std::result::Result<Vec<ScheduleResponse>, sqlx::Error> {
    let rows: Vec<ScheduleRow> = sqlx::query_as(
        "SELECT id, repo_id, cron, name, ref_name, yaml_path, enabled, last_run_at, next_run_at, created_at, updated_at FROM pipeline_schedules WHERE repo_id = $1 ORDER BY created_at",
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.into()).collect())
}

async fn get_repo_owner_name(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
) -> std::result::Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
    let row = sqlx::query(
        "SELECT u.username, r.name FROM repositories r JOIN users u ON r.owner_id = u.id WHERE r.id = $1",
    )
    .bind(repo_id)
    .fetch_one(pool)
    .await?;

    Ok((row.get("username"), row.get("name")))
}

fn read_pipeline_yaml_from_fs(
    storage_path: &str,
    owner: &str,
    repo_name: &str,
    ref_name: &str,
    yaml_path: &str,
) -> Option<String> {
    let base = std::path::Path::new(storage_path);
    let repo_path = base.join(owner).join(format!("{repo_name}.git"));

    let output = std::process::Command::new("git")
        .arg("archive")
        .arg("--format=tar")
        .arg(ref_name)
        .arg(yaml_path)
        .current_dir(&repo_path)
        .output()
        .ok()?;

    if output.status.success() && !output.stdout.is_empty() {
        let cursor = std::io::Cursor::new(&output.stdout);
        if let Ok(mut archive) = tar::Archive::new(cursor).entries() {
            while let Some(Ok(mut entry)) = archive.next() {
                let mut content = String::new();
                if std::io::Read::read_to_string(&mut entry, &mut content).is_ok() {
                    return Some(content);
                }
            }
        }
    }

    let file_path = repo_path.join(yaml_path);
    std::fs::read_to_string(file_path).ok()
}

fn get_head_commit_sha(
    storage_path: &str,
    owner: &str,
    repo_name: &str,
    ref_name: &str,
) -> Option<String> {
    let base = std::path::Path::new(storage_path);
    let repo_path = base.join(owner).join(format!("{repo_name}.git"));

    let output = std::process::Command::new("git")
        .arg("--git-dir")
        .arg(&repo_path)
        .arg("rev-parse")
        .arg(ref_name)
        .output()
        .ok()?;

    if output.status.success() {
        let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !sha.is_empty() {
            return Some(sha);
        }
    }

    None
}

#[allow(clippy::too_many_arguments)]
async fn create_manual_pipeline_run(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    yaml_path: &str,
    ref_name: &str,
    commit_sha: &str,
    yaml_content: &str,
    pipeline: &civit_pipeline::Pipeline,
) -> std::result::Result<Uuid, sqlx::Error> {
    let def_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO pipeline_definitions (id, repo_id, yaml_path, ref_name, commit_sha, yaml_content, version) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(def_id)
    .bind(repo_id)
    .bind(yaml_path)
    .bind(ref_name)
    .bind(commit_sha)
    .bind(yaml_content)
    .bind(&pipeline.version)
    .execute(pool)
    .await?;

    let mut job_ids: Vec<(String, Uuid)> = Vec::new();
    for (idx, job) in pipeline.jobs.iter().enumerate() {
        let job_id = Uuid::new_v4();
        let needs_json = serde_json::to_value(&job.needs).unwrap_or(serde_json::Value::Null);
        let runs_on_json = job
            .runs_on
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .unwrap_or(None);

        sqlx::query(
            "INSERT INTO pipeline_jobs (id, definition_id, name, job_index, needs, runs_on, timeout, condition) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(job_id)
        .bind(def_id)
        .bind(&job.name)
        .bind(idx as i32)
        .bind(needs_json)
        .bind(runs_on_json)
        .bind(job.timeout.as_ref().map(|t| t.to_string()))
        .bind(&job.condition)
        .execute(pool)
        .await?;

        for (sidx, step) in job.steps.iter().enumerate() {
            let step_id = Uuid::new_v4();
            let commands_json = step
                .run
                .as_ref()
                .map(serde_json::to_value)
                .transpose()
                .unwrap_or(None);
            let action_str = step.uses.as_ref().map(|u| match u.action {
                civit_pipeline::StepAction::Checkout => "checkout",
                civit_pipeline::StepAction::Cache => "cache",
                civit_pipeline::StepAction::Artifact => "artifact",
            });

            sqlx::query(
                "INSERT INTO pipeline_job_steps (id, job_id, step_index, name, step_type, commands, action, action_params, image, workdir, env, secrets, continue_on_error, timeout, condition) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
            )
            .bind(step_id)
            .bind(job_id)
            .bind(sidx as i32)
            .bind(&step.name)
            .bind(if action_str.is_some() { "uses" } else { "run" })
            .bind(commands_json)
            .bind(action_str)
            .bind(step.uses.as_ref().map(|u| serde_json::to_value(&u.with)).transpose().unwrap_or(None))
            .bind(&step.image)
            .bind(&step.workdir)
            .bind(step.env.as_ref().map(serde_json::to_value).transpose().unwrap_or(None))
            .bind(step.secrets.as_ref().map(serde_json::to_value).transpose().unwrap_or(None))
            .bind(step.continue_on_error)
            .bind(step.timeout.as_ref().map(|t| t.to_string()))
            .bind(&step.condition)
            .execute(pool)
            .await?;
        }

        job_ids.push((job.name.clone(), job_id));
    }

    let run_id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO pipeline_runs (id, definition_id, repo_id, trigger, ref_name, commit_sha, status, concurrency_group, created_at) VALUES ($1, $2, $3, 'manual', $4, $5, 'pending', $6, $7)",
    )
    .bind(run_id)
    .bind(def_id)
    .bind(repo_id)
    .bind(ref_name)
    .bind(commit_sha)
    .bind(pipeline.concurrency.as_ref().and_then(|c| c.group.clone()))
    .bind(now)
    .execute(pool)
    .await?;

    for (name, job_id) in &job_ids {
        sqlx::query(
            "INSERT INTO pipeline_run_jobs (id, run_id, job_id, name, status, created_at) VALUES ($1, $2, $3, $4, 'pending', $5)",
        )
        .bind(Uuid::new_v4())
        .bind(run_id)
        .bind(job_id)
        .bind(name)
        .bind(now)
        .execute(pool)
        .await?;
    }

    Ok(run_id)
}

// ---------------------------------------------------------------------------
// DB row types
// ---------------------------------------------------------------------------

#[derive(Debug, sqlx::FromRow)]
struct ScheduleRow {
    id: Uuid,
    repo_id: Uuid,
    cron: String,
    name: Option<String>,
    ref_name: Option<String>,
    yaml_path: String,
    enabled: bool,
    last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    next_run_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ScheduleRow> for ScheduleResponse {
    fn from(r: ScheduleRow) -> Self {
        Self {
            id: r.id.to_string(),
            repo_id: r.repo_id.to_string(),
            cron: r.cron,
            name: r.name,
            ref_name: r.ref_name,
            yaml_path: r.yaml_path,
            enabled: r.enabled,
            last_run_at: r.last_run_at.map(|t| t.to_rfc3339()),
            next_run_at: r.next_run_at.map(|t| t.to_rfc3339()),
            created_at: r.created_at.to_rfc3339(),
            updated_at: r.updated_at.to_rfc3339(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_schedule_request_deserialize() {
        let json = r#"{"cron": "0 6 * * 1", "name": "weekly"}"#;
        let req: CreateScheduleRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.cron, "0 6 * * 1");
        assert_eq!(req.name, Some("weekly".to_string()));
        assert_eq!(req.yaml_path, ".civit/pipeline.yaml");
        assert!(req.enabled);
    }

    #[test]
    fn test_update_schedule_request_deserialize() {
        let json = r#"{"cron": "*/15 * * * *", "enabled": false}"#;
        let req: UpdateScheduleRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.cron, Some("*/15 * * * *".to_string()));
        assert_eq!(req.enabled, Some(false));
        assert!(req.name.is_none());
    }

    #[test]
    fn test_schedule_response_serialize() {
        let resp = ScheduleResponse {
            id: "00000000-0000-0000-0000-000000000001".to_string(),
            repo_id: "00000000-0000-0000-0000-000000000002".to_string(),
            cron: "0 6 * * 1".to_string(),
            name: Some("weekly".to_string()),
            ref_name: Some("main".to_string()),
            yaml_path: ".civit/pipeline.yaml".to_string(),
            enabled: true,
            last_run_at: None,
            next_run_at: Some("2025-06-02T06:00:00+00:00".to_string()),
            created_at: "2025-06-01T00:00:00+00:00".to_string(),
            updated_at: "2025-06-01T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("0 6 * * 1"));
        assert!(json.contains("weekly"));
    }

    #[test]
    fn test_manual_run_response_serialize() {
        let resp = ManualRunResponse {
            schedule_id: "s1".to_string(),
            run_id: "r1".to_string(),
            status: "pending".to_string(),
            triggered_at: "2025-06-01T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("pending"));
    }
}
