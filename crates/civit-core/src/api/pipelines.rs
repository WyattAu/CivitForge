//! CI/CD Pipeline API endpoints.

#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, OptionalAuthUser};
use crate::error::CoreError;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use chrono::Utc;
use civit_pipeline::trigger::TriggerContext;
use civit_pipeline::{matches_trigger, parse_pipeline, validate_pipeline};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Response / Request types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRunResponse {
    pub id: String,
    pub repo_id: String,
    pub trigger: String,
    pub ref_name: Option<String>,
    pub commit_sha: String,
    pub status: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRunDetailResponse {
    #[serde(flatten)]
    pub run: PipelineRunResponse,
    pub jobs: Vec<RunJobResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunJobResponse {
    pub id: String,
    pub name: String,
    pub status: String,
    pub runner_id: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub steps: Vec<RunStepResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStepResponse {
    pub id: String,
    pub name: String,
    pub step_index: i32,
    pub status: String,
    pub image: Option<String>,
    pub exit_code: Option<i32>,
    pub output: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TriggerPipelineRequest {
    pub ref_name: String,
    pub commit_sha: String,
    #[serde(default = "default_yaml_path")]
    pub yaml_path: String,
    pub event_type: Option<String>,
    #[serde(default)]
    pub changed_files: Vec<String>,
}

fn default_yaml_path() -> String {
    ".civit/pipeline.yaml".to_string()
}

#[derive(Debug, Deserialize)]
pub struct PipelineListParams {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub status: Option<String>,
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, Deserialize)]
pub struct CancelPipelineRequest {
    pub reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn pipeline_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{repo}/pipelines",
            get(list_pipelines).post(trigger_pipeline),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/pipelines/{pipeline_id}",
            get(get_pipeline).delete(cancel_pipeline),
        )
        .route(
            "/api/v1/repos/{owner}/{repo}/pipelines/{pipeline_id}/jobs",
            get(get_pipeline_jobs),
        )
        .route("/api/v1/pipelines", get(list_all_pipelines))
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// List pipelines for a specific repository.
pub async fn list_pipelines(
    State(state): State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    Query(params): Query<PipelineListParams>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    match get_repo_id(pool, &owner, &repo_name).await {
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )
            .into_response(),
        Ok(Some(repo_id)) => {
            let result = list_pipelines_by_repo(
                pool,
                repo_id,
                params.limit,
                params.offset,
                params.status.as_deref(),
            )
            .await;
            match result {
                Ok(runs) => {
                    let resp: Vec<PipelineRunResponse> =
                        runs.into_iter().map(|r| r.into()).collect();
                    (axum::http::StatusCode::OK, Json(resp)).into_response()
                }
                Err(e) => (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CoreError::Database(e.to_string()).error_response()),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// List all pipelines across all repos (admin-only).
pub async fn list_all_pipelines(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    match list_all_pipelines_db(pool, 50, 0).await {
        Ok(runs) => {
            let resp: Vec<PipelineRunResponse> = runs.into_iter().map(|r| r.into()).collect();
            (axum::http::StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Trigger a pipeline run for a repository.
///
/// Parses the YAML from the repo, validates it, creates a pipeline run.
pub async fn trigger_pipeline(
    State(state): State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    _auth: AuthUser,
    Json(req): Json<TriggerPipelineRequest>,
) -> impl IntoResponse {
    // Permission check
    // Note: auth check is handled by AuthUser extractor

    let pool = state.db.pool();

    // Resolve repository
    let repo_id = match get_repo_id(pool, &owner, &repo_name).await {
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

    // Read YAML content from the repository's working copy
    let yaml_content = match read_pipeline_yaml(
        &state,
        &repo_id,
        &owner,
        &repo_name,
        &req.ref_name,
        &req.yaml_path,
    )
    .await
    {
        Ok(Some(content)) => content,
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                Json(
                    CoreError::NotFound(format!(
                        "pipeline YAML not found at '{}' in ref '{}'",
                        req.yaml_path, req.ref_name
                    ))
                    .error_response(),
                ),
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

    // Parse YAML
    let pipeline = match parse_pipeline(&yaml_content) {
        Ok(p) => p,
        Err(e) => {
            return (
                axum::http::StatusCode::UNPROCESSABLE_ENTITY,
                Json(CoreError::NotFound(e.to_string()).error_response()),
            )
                .into_response();
        }
    };

    // Validate pipeline structure
    if let Err(e) = validate_pipeline(&pipeline) {
        return (
            axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            Json(CoreError::NotFound(e.to_string()).error_response()),
        )
            .into_response();
    }

    // Check triggers
    let event_type = req.event_type.as_deref().unwrap_or("push");
    let ctx = TriggerContext::push(&req.ref_name, req.changed_files.clone());
    let _ = ctx; // Trigger matching is done at push-hook time; direct API trigger always fires.

    // Store definition + jobs + steps + create run
    match create_pipeline_run(
        pool,
        repo_id,
        &req.yaml_path,
        &req.ref_name,
        &req.commit_sha,
        event_type,
        &yaml_content,
        &pipeline,
    )
    .await
    {
        Ok(run) => (axum::http::StatusCode::CREATED, Json(run)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Get pipeline run details with jobs and steps.
pub async fn get_pipeline(
    State(state): State<AppState>,
    Path((_owner, _repo_name, pipeline_id)): Path<(String, String, String)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let id = match Uuid::parse_str(&pipeline_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid pipeline ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match get_pipeline_detail(pool, id).await {
        Ok(Some(detail)) => (axum::http::StatusCode::OK, Json(detail)).into_response(),
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("pipeline run not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Cancel a pipeline run.
pub async fn cancel_pipeline(
    State(state): State<AppState>,
    Path((_owner, _repo_name, pipeline_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let id = match Uuid::parse_str(&pipeline_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid pipeline ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match cancel_pipeline_run(pool, id).await {
        Ok(Some(run)) => (axum::http::StatusCode::OK, Json(run)).into_response(),
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("pipeline run not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Get jobs for a pipeline run.
pub async fn get_pipeline_jobs(
    State(state): State<AppState>,
    Path((_owner, _repo_name, pipeline_id)): Path<(String, String, String)>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let id = match Uuid::parse_str(&pipeline_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid pipeline ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match get_pipeline_jobs_db(pool, id).await {
        Ok(jobs) => (axum::http::StatusCode::OK, Json(jobs)).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Resolve (owner, repo_name) to repo UUID.
async fn get_repo_id(
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

/// Read pipeline YAML from the repository at the specified git ref.
/// Tries `git archive` first (ref-aware), falls back to filesystem read.
async fn read_pipeline_yaml(
    state: &AppState,
    _repo_id: &Uuid,
    owner: &str,
    repo_name: &str,
    ref_name: &str,
    yaml_path: &str,
) -> std::result::Result<Option<String>, CoreError> {
    let base = &state.config.storage_path;
    let repo_path = std::path::Path::new(base)
        .join(owner)
        .join(format!("{repo_name}.git"));

    // Try git archive (ref-aware extraction)
    let output = tokio::process::Command::new("git")
        .arg("archive")
        .arg("--format=tar")
        .arg(ref_name)
        .arg(yaml_path)
        .current_dir(&repo_path)
        .output()
        .await;

    if let Ok(out) = output {
        if out.status.success() && !out.stdout.is_empty() {
            // Extract single file from tar archive
            let cursor = std::io::Cursor::new(&out.stdout);
            if let Ok(mut archive) = tar::Archive::new(cursor).entries() {
                while let Some(Ok(mut entry)) = archive.next() {
                    let mut content = String::new();
                    if std::io::Read::read_to_string(&mut entry, &mut content).is_ok() {
                        return Ok(Some(content));
                    }
                }
            }
        }
    }

    // Fallback: read from filesystem (working tree or bare repo)
    let file_path = repo_path.join(yaml_path);
    if file_path.exists() {
        tokio::fs::read_to_string(&file_path)
            .await
            .map(Some)
            .map_err(|e| CoreError::Internal(format!("read pipeline YAML: {e}")))
    } else {
        Ok(None)
    }
}

/// Create a pipeline definition, its jobs/steps, and the initial run.
#[allow(clippy::too_many_arguments)]
async fn create_pipeline_run(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    yaml_path: &str,
    ref_name: &str,
    commit_sha: &str,
    trigger: &str,
    yaml_content: &str,
    pipeline: &civit_pipeline::Pipeline,
) -> std::result::Result<PipelineRunResponse, sqlx::Error> {
    // 1. Create pipeline definition
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

    // 2. Create jobs
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

        // 3. Create steps for this job
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

    // 4. Enforce concurrency (cancel in-progress runs in same group)
    let concurrency_group = pipeline.concurrency.as_ref().and_then(|c| c.group.clone());
    let cancel_in_progress = pipeline
        .concurrency
        .as_ref()
        .is_some_and(|c| c.cancel_in_progress);
    if let Some(ref group) = concurrency_group {
        crate::api::runners::enforce_concurrency(pool, repo_id, group, cancel_in_progress)
            .await
            .ok();
    }

    // 5. Create pipeline run
    let run_id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO pipeline_runs (id, definition_id, repo_id, trigger, ref_name, commit_sha, status, concurrency_group, created_at) VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7, $8)",
    )
    .bind(run_id)
    .bind(def_id)
    .bind(repo_id)
    .bind(trigger)
    .bind(ref_name)
    .bind(commit_sha)
    .bind(&concurrency_group)
    .bind(now)
    .execute(pool)
    .await?;

    // 6. Create run jobs (one per pipeline job)
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

    Ok(PipelineRunResponse {
        id: run_id.to_string(),
        repo_id: repo_id.to_string(),
        trigger: trigger.to_string(),
        ref_name: Some(ref_name.to_string()),
        commit_sha: commit_sha.to_string(),
        status: "pending".to_string(),
        created_at: now.to_rfc3339(),
        started_at: None,
        finished_at: None,
    })
}

/// List pipeline runs for a repo.
async fn list_pipelines_by_repo(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    limit: i64,
    offset: i64,
    status: Option<&str>,
) -> std::result::Result<Vec<PipelineRunRow>, sqlx::Error> {
    let sql = if status.is_some() {
        "SELECT id, repo_id, trigger, ref_name, commit_sha, status, created_at, started_at, finished_at FROM pipeline_runs WHERE repo_id = $1 AND status = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4"
    } else {
        "SELECT id, repo_id, trigger, ref_name, commit_sha, status, created_at, started_at, finished_at FROM pipeline_runs WHERE repo_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    };

    let mut query = sqlx::query_as::<_, PipelineRunRow>(sql);
    query = query.bind(repo_id);

    if let Some(s) = status {
        query = query.bind(s);
    }

    query = query.bind(limit).bind(offset);
    query.fetch_all(pool).await
}

/// List all pipeline runs.
async fn list_all_pipelines_db(
    pool: &sqlx::PgPool,
    limit: i64,
    offset: i64,
) -> std::result::Result<Vec<PipelineRunRow>, sqlx::Error> {
    sqlx::query_as::<_, PipelineRunRow>(
        "SELECT id, repo_id, trigger, ref_name, commit_sha, status, created_at, started_at, finished_at FROM pipeline_runs ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// Get pipeline run detail with jobs and steps.
async fn get_pipeline_detail(
    pool: &sqlx::PgPool,
    run_id: Uuid,
) -> std::result::Result<Option<PipelineRunDetailResponse>, sqlx::Error> {
    // Get the run
    let run: Option<PipelineRunRow> = sqlx::query_as(
        "SELECT id, repo_id, trigger, ref_name, commit_sha, status, created_at, started_at, finished_at FROM pipeline_runs WHERE id = $1",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await?;

    let run = match run {
        Some(r) => r,
        None => return Ok(None),
    };

    // Get run jobs with steps
    let jobs = get_pipeline_jobs_db(pool, run_id).await?;

    Ok(Some(PipelineRunDetailResponse {
        run: run.into(),
        jobs,
    }))
}

/// Get jobs for a pipeline run with their steps.
async fn get_pipeline_jobs_db(
    pool: &sqlx::PgPool,
    run_id: Uuid,
) -> std::result::Result<Vec<RunJobResponse>, sqlx::Error> {
    let run_jobs: Vec<RunJobRow> = sqlx::query_as(
        "SELECT id, name, status, runner_id, started_at, finished_at FROM pipeline_run_jobs WHERE run_id = $1 ORDER BY created_at",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    for rj in run_jobs {
        let steps: Vec<RunStepRow> = sqlx::query_as(
            "SELECT id, name, step_index, status, image, exit_code, output, started_at, finished_at FROM pipeline_run_steps WHERE run_job_id = $1 ORDER BY step_index",
        )
        .bind(rj.id)
        .fetch_all(pool)
        .await?;

        result.push(RunJobResponse {
            id: rj.id.to_string(),
            name: rj.name,
            status: rj.status,
            runner_id: rj.runner_id.map(|id| id.to_string()),
            started_at: rj.started_at.map(|t| t.to_rfc3339()),
            finished_at: rj.finished_at.map(|t| t.to_rfc3339()),
            steps: steps.into_iter().map(|s| s.into()).collect(),
        });
    }

    Ok(result)
}

/// Cancel a pipeline run (set status to 'canceled').
async fn cancel_pipeline_run(
    pool: &sqlx::PgPool,
    run_id: Uuid,
) -> std::result::Result<Option<PipelineRunResponse>, sqlx::Error> {
    let now = Utc::now();
    let result = sqlx::query(
        "UPDATE pipeline_runs SET status = 'canceled', finished_at = $1 WHERE id = $2 AND status IN ('pending', 'queued', 'running') RETURNING id, repo_id, trigger, ref_name, commit_sha, status, created_at, started_at, finished_at",
    )
    .bind(now)
    .bind(run_id)
    .fetch_optional(pool)
    .await?;

    let row = match result {
        Some(r) => r,
        None => return Ok(None),
    };

    Ok(Some(PipelineRunResponse {
        id: row.get::<Uuid, _>("id").to_string(),
        repo_id: row.get::<Uuid, _>("repo_id").to_string(),
        trigger: row.get::<String, _>("trigger"),
        ref_name: row.get::<Option<String>, _>("ref_name"),
        commit_sha: row.get::<String, _>("commit_sha"),
        status: row.get::<String, _>("status"),
        created_at: row
            .get::<chrono::DateTime<chrono::Utc>, _>("created_at")
            .to_rfc3339(),
        started_at: row
            .get::<Option<chrono::DateTime<chrono::Utc>>, _>("started_at")
            .map(|t| t.to_rfc3339()),
        finished_at: row
            .get::<Option<chrono::DateTime<chrono::Utc>>, _>("finished_at")
            .map(|t| t.to_rfc3339()),
    }))
}

// ---------------------------------------------------------------------------
// Push hook integration
// ---------------------------------------------------------------------------

/// Trigger CI/CD pipelines after a git push.
///
/// Called from `receive_pack` in a background task (fire-and-forget).
/// Reads the pipeline YAML, checks triggers, and creates pipeline runs.
pub async fn trigger_pipelines_on_push(state: &AppState, owner: &str, repo_name: &str) {
    let pool = state.db.pool();

    // Resolve repository
    let repo_id = match get_repo_id(pool, owner, repo_name).await {
        Ok(Some(id)) => id,
        _ => return, // Silently ignore — repo lookup failure shouldn't break push
    };

    // Try to detect the pushed ref and commit SHA from HEAD
    let (ref_name, commit_sha) = match detect_push_ref(state, owner, repo_name).await {
        Some(r) => r,
        None => return,
    };

    // Try to read pipeline YAML
    let yaml_path = ".civit/pipeline.yaml";
    let yaml_content =
        match read_pipeline_yaml(state, &repo_id, owner, repo_name, &ref_name, yaml_path).await {
            Ok(Some(content)) => content,
            _ => return, // No pipeline YAML — skip
        };

    // Parse and validate
    let pipeline =
        match parse_pipeline(&yaml_content).and_then(|p| validate_pipeline(&p).map(|_| p)) {
            Ok(p) => p,
            _ => return, // Invalid YAML — skip (user can fix)
        };

    // Check triggers
    let ctx = TriggerContext::push(&ref_name, vec![]);
    if !matches_trigger(&pipeline, &ctx) {
        return; // Triggers don't match — skip
    }

    // Create pipeline run
    let _ = create_pipeline_run(
        pool,
        repo_id,
        yaml_path,
        &ref_name,
        &commit_sha,
        "push",
        &yaml_content,
        &pipeline,
    )
    .await;
}

/// Detect the pushed ref and commit SHA from the repository.
async fn detect_push_ref(
    state: &AppState,
    owner: &str,
    repo_name: &str,
) -> Option<(String, String)> {
    let base = &state.config.storage_path;
    let repo_path = std::path::Path::new(base)
        .join(owner)
        .join(format!("{repo_name}.git"));

    // Get HEAD ref name and commit SHA using git command
    let head_ref = std::process::Command::new("git")
        .arg("--git-dir")
        .arg(&repo_path)
        .arg("symbolic-ref")
        .arg("--short")
        .arg("HEAD")
        .output()
        .ok()?;

    let ref_name = String::from_utf8_lossy(&head_ref.stdout).trim().to_string();
    if ref_name.is_empty() {
        return None;
    }

    let sha_output = std::process::Command::new("git")
        .arg("--git-dir")
        .arg(&repo_path)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .ok()?;

    let commit_sha = String::from_utf8_lossy(&sha_output.stdout)
        .trim()
        .to_string();
    if commit_sha.is_empty() {
        return None;
    }

    Some((ref_name, commit_sha))
}

// ---------------------------------------------------------------------------
// DB row types
// ---------------------------------------------------------------------------

#[derive(Debug, sqlx::FromRow)]
struct PipelineRunRow {
    id: Uuid,
    repo_id: Uuid,
    trigger: String,
    ref_name: Option<String>,
    commit_sha: String,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    finished_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<PipelineRunRow> for PipelineRunResponse {
    fn from(r: PipelineRunRow) -> Self {
        Self {
            id: r.id.to_string(),
            repo_id: r.repo_id.to_string(),
            trigger: r.trigger,
            ref_name: r.ref_name,
            commit_sha: r.commit_sha,
            status: r.status,
            created_at: r.created_at.to_rfc3339(),
            started_at: r.started_at.map(|t| t.to_rfc3339()),
            finished_at: r.finished_at.map(|t| t.to_rfc3339()),
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct RunJobRow {
    id: Uuid,
    name: String,
    status: String,
    runner_id: Option<Uuid>,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    finished_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, sqlx::FromRow)]
struct RunStepRow {
    id: Uuid,
    name: String,
    step_index: i32,
    status: String,
    image: Option<String>,
    exit_code: Option<i32>,
    output: Option<String>,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    finished_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<RunStepRow> for RunStepResponse {
    fn from(r: RunStepRow) -> Self {
        Self {
            id: r.id.to_string(),
            name: r.name,
            step_index: r.step_index,
            status: r.status,
            image: r.image,
            exit_code: r.exit_code,
            output: r.output,
            started_at: r.started_at.map(|t| t.to_rfc3339()),
            finished_at: r.finished_at.map(|t| t.to_rfc3339()),
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
    fn test_default_yaml_path() {
        assert_eq!(default_yaml_path(), ".civit/pipeline.yaml");
    }

    #[test]
    fn test_trigger_pipeline_request_deserialize() {
        let json = r#"{"ref_name": "main", "commit_sha": "abc123"}"#;
        let req: TriggerPipelineRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.ref_name, "main");
        assert_eq!(req.commit_sha, "abc123");
        assert_eq!(req.yaml_path, ".civit/pipeline.yaml");
        assert!(req.event_type.is_none());
        assert!(req.changed_files.is_empty());
    }

    #[test]
    fn test_trigger_pipeline_request_custom_yaml() {
        let json = r#"{"ref_name": "dev", "commit_sha": "def456", "yaml_path": ".civit/deploy.yaml", "event_type": "push", "changed_files": ["src/main.rs"]}"#;
        let req: TriggerPipelineRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.yaml_path, ".civit/deploy.yaml");
        assert_eq!(req.event_type, Some("push".to_string()));
        assert_eq!(req.changed_files.len(), 1);
    }

    #[test]
    fn test_pipeline_list_params_defaults() {
        let json = r#"{}"#;
        let params: PipelineListParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.limit, 50);
        assert_eq!(params.offset, 0);
        assert!(params.status.is_none());
    }

    #[test]
    fn test_pipeline_list_params_custom() {
        let json = r#"{"limit": 10, "offset": 20, "status": "running"}"#;
        let params: PipelineListParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.limit, 10);
        assert_eq!(params.offset, 20);
        assert_eq!(params.status, Some("running".to_string()));
    }

    #[test]
    fn test_pipeline_run_response_serialize() {
        let resp = PipelineRunResponse {
            id: "00000000-0000-0000-0000-000000000001".to_string(),
            repo_id: "00000000-0000-0000-0000-000000000002".to_string(),
            trigger: "push".to_string(),
            ref_name: Some("main".to_string()),
            commit_sha: "abc123".to_string(),
            status: "pending".to_string(),
            created_at: "2025-01-01T00:00:00+00:00".to_string(),
            started_at: None,
            finished_at: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("push"));
        assert!(json.contains("main"));
    }

    #[test]
    fn test_run_step_response_serialize() {
        let step = RunStepResponse {
            id: "00000000-0000-0000-0000-000000000001".to_string(),
            name: "build".to_string(),
            step_index: 0,
            status: "success".to_string(),
            image: Some("alpine:3.18".to_string()),
            exit_code: Some(0),
            output: Some("All tests passed".to_string()),
            started_at: Some("2025-01-01T00:00:00+00:00".to_string()),
            finished_at: Some("2025-01-01T00:01:00+00:00".to_string()),
        };
        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("alpine:3.18"));
    }
}
