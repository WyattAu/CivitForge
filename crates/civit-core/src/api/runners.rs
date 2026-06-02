//! CI/CD Runner API endpoints.
//!
//! Handles runner registration, authentication, job polling, claiming,
//! log streaming, and step completion.

#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::error::CoreError;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Response / Request types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRunnerRequest {
    pub name: String,
    pub description: Option<String>,
    /// Scope: "global", "org", "repo"
    #[serde(default = "default_scope")]
    pub scope: String,
    /// Labels for job matching (e.g. ["linux", "amd64"])
    pub labels: Vec<String>,
    /// Runner group (optional, for load balancing)
    pub runner_group: Option<String>,
    /// Token (auto-generated if not provided)
    pub token: Option<String>,
}

fn default_scope() -> String {
    "global".to_string()
}

#[derive(Debug, Clone, Serialize)]
pub struct RegisterRunnerResponse {
    pub id: String,
    pub name: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub scope: String,
    pub labels: Vec<String>,
    pub status: String,
    pub runner_group: Option<String>,
    pub last_seen_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PollJobResponse {
    /// ID of the available job
    pub job_id: String,
    /// Pipeline run ID
    pub run_id: String,
    /// Job name
    pub name: String,
    /// Step definitions for this job
    pub steps: Vec<JobStepSpec>,
    /// Repository clone URL
    pub repo_url: String,
    /// Commit SHA to checkout
    pub commit_sha: String,
    /// Branch/tag ref
    pub ref_name: String,
    /// Environment variables (non-secret, from job + pipeline level)
    pub env: serde_json::Value,
    /// Secrets to inject (names only, values resolved via API)
    pub secret_names: Vec<String>,
    /// Service containers
    pub services: serde_json::Value,
    /// Job timeout (ISO 8601 duration)
    pub timeout: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStepSpec {
    pub name: String,
    pub step_index: i32,
    pub step_type: String,
    pub commands: Option<Vec<String>>,
    pub action: Option<String>,
    pub action_params: Option<serde_json::Value>,
    pub image: Option<String>,
    pub workdir: String,
    pub env: serde_json::Value,
    pub continue_on_error: bool,
    pub timeout: Option<String>,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStepRequest {
    pub status: String,
    pub exit_code: Option<i32>,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateJobRequest {
    pub status: String,
    pub outputs: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeartbeatResponse {
    pub status: String,
}

// ---------------------------------------------------------------------------
// Runner auth middleware
// ---------------------------------------------------------------------------

/// Generate a random runner token.
fn generate_runner_token() -> String {
    use rand::RngExt;
    let mut bytes = [0u8; 32];
    rand::rng().fill(&mut bytes[..]);
    hex::encode(bytes)
}

/// Hash a runner token for storage.
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn runner_routes() -> Router<AppState> {
    Router::new()
        // Admin-facing runner management
        .route("/api/v1/runners", get(list_runners).post(register_runner))
        .route(
            "/api/v1/runners/{runner_id}",
            get(get_runner).delete(delete_runner),
        )
        // Runner-facing protocol
        .route("/api/v1/runners/poll", post(poll_job))
        .route(
            "/api/v1/runners/{runner_id}/claim/{job_id}",
            post(claim_job),
        )
        .route("/api/v1/runners/{runner_id}/heartbeat", post(heartbeat))
        // Step updates
        .route(
            "/api/v1/runners/{runner_id}/steps/{step_id}",
            post(update_step),
        )
        // Secret resolution
        .route("/api/v1/runners/{runner_id}/secrets", post(resolve_secrets))
        // Job completion
        .route(
            "/api/v1/runners/{runner_id}/jobs/{job_id}/complete",
            post(complete_job),
        )
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Register a new runner. Returns the runner ID and auth token.
pub async fn register_runner(
    State(state): State<AppState>,
    Json(req): Json<RegisterRunnerRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let runner_id = Uuid::new_v4();
    let token = req.token.unwrap_or_else(generate_runner_token);
    let token_hash = hash_token(&token);
    let labels_json = serde_json::to_value(&req.labels).unwrap_or(serde_json::Value::Null);
    let now = Utc::now();

    let result = sqlx::query(
        "INSERT INTO runners (id, name, description, scope, labels, runner_group, token_hash, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, 'offline', $8, $8)",
    )
    .bind(runner_id)
    .bind(&req.name)
    .bind(req.description.as_deref().unwrap_or(""))
    .bind(&req.scope)
    .bind(labels_json)
    .bind(&req.runner_group)
    .bind(&token_hash)
    .bind(now)
    .execute(pool)
    .await;

    match result {
        Ok(_) => (
            axum::http::StatusCode::CREATED,
            Json(RegisterRunnerResponse {
                id: runner_id.to_string(),
                name: req.name,
                token,
            }),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// List registered runners.
pub async fn list_runners(State(state): State<AppState>) -> impl IntoResponse {
    let pool = state.db.pool();

    match sqlx::query_as::<_, RunnerRow>(
        "SELECT id, name, description, scope, labels, status, runner_group, last_seen_at, created_at FROM runners ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
    {
        Ok(runners) => {
            let resp: Vec<RunnerResponse> = runners.into_iter().map(|r| r.into()).collect();
            (axum::http::StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Get runner details.
pub async fn get_runner(
    State(state): State<AppState>,
    Path(runner_id): Path<String>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let id = match Uuid::parse_str(&runner_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid runner ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match sqlx::query_as::<_, RunnerRow>(
        "SELECT id, name, description, scope, labels, status, runner_group, last_seen_at, created_at FROM runners WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(runner)) => (axum::http::StatusCode::OK, Json(RunnerResponse::from(runner))).into_response(),
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("runner not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Delete a runner.
pub async fn delete_runner(
    State(state): State<AppState>,
    Path(runner_id): Path<String>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let id = match Uuid::parse_str(&runner_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid runner ID".into()).error_response()),
            )
                .into_response();
        }
    };

    match sqlx::query("DELETE FROM runners WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => {
            (axum::http::StatusCode::NO_CONTENT, "").into_response()
        }
        Ok(_) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("runner not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Poll for available jobs. Runner sends auth token + labels.
pub async fn poll_job(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let pool = state.db.pool();

    // Extract runner token from request body
    let token = match body.get("token").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                Json(CoreError::Auth("missing runner token".into()).error_response()),
            )
                .into_response();
        }
    };

    let token_hash = hash_token(token);

    // Validate runner
    let runner_id = match sqlx::query_as::<_, (Uuid,)>(
        "SELECT id FROM runners WHERE token_hash = $1 AND status = 'online'",
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await
    {
        Ok(Some((id,))) => id,
        Ok(None) => {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                Json(CoreError::Auth("invalid or offline runner".into()).error_response()),
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

    // Update heartbeat
    let now = Utc::now();
    let _ = sqlx::query("UPDATE runners SET last_seen_at = $1 WHERE id = $2")
        .bind(now)
        .bind(runner_id)
        .execute(pool)
        .await;

    // Find next available job (pending, not yet claimed)
    match find_available_job(pool, runner_id).await {
        Ok(Some(job)) => (axum::http::StatusCode::OK, Json(job)).into_response(),
        Ok(None) => (
            axum::http::StatusCode::NO_CONTENT,
            Json(serde_json::Value::Null),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Claim a specific job.
pub async fn claim_job(
    State(state): State<AppState>,
    Path((runner_id_str, job_id_str)): Path<(String, String)>,
) -> impl IntoResponse {
    let pool = state.db.pool();

    let runner_id = match Uuid::parse_str(&runner_id_str) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid runner ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let job_id = match Uuid::parse_str(&job_id_str) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid job ID".into()).error_response()),
            )
                .into_response();
        }
    };

    // Mark job as running and assign runner
    let now = Utc::now();
    match sqlx::query(
        "UPDATE pipeline_run_jobs SET status = 'running', runner_id = $1, started_at = $2 WHERE id = $3 AND status = 'pending' RETURNING id",
    )
    .bind(runner_id)
    .bind(now)
    .bind(job_id)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(_)) => {
            // Also update the run status if not already running
            let _ = sqlx::query(
                "UPDATE pipeline_runs SET status = 'running', started_at = $1 WHERE id = (SELECT run_id FROM pipeline_run_jobs WHERE id = $2) AND status = 'pending'",
            )
            .bind(now)
            .bind(job_id)
            .execute(pool)
            .await;

            (axum::http::StatusCode::OK, Json(serde_json::json!({"claimed": true}))).into_response()
        }
        Ok(None) => (
            axum::http::StatusCode::CONFLICT,
            Json(CoreError::NotFound("job already claimed or not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Runner heartbeat.
pub async fn heartbeat(
    State(state): State<AppState>,
    Path(runner_id_str): Path<String>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let id = match Uuid::parse_str(&runner_id_str) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid runner ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let now = Utc::now();
    match sqlx::query("UPDATE runners SET status = 'online', last_seen_at = $1 WHERE id = $2")
        .bind(now)
        .bind(id)
        .execute(pool)
        .await
    {
        Ok(_) => (
            axum::http::StatusCode::OK,
            Json(HeartbeatResponse {
                status: "ok".into(),
            }),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Update step status (called by runner during execution).
pub async fn update_step(
    State(state): State<AppState>,
    Path((_runner_id_str, step_id_str)): Path<(String, String)>,
    Json(req): Json<UpdateStepRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();

    let step_id = match Uuid::parse_str(&step_id_str) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid step ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let now = Utc::now();

    // Update step status
    let result = if req.status == "running" {
        sqlx::query(
            "UPDATE pipeline_run_steps SET status = 'running', started_at = $1 WHERE id = $2",
        )
        .bind(now)
        .bind(step_id)
        .execute(pool)
        .await
    } else {
        // Completed (success/failure/canceled)
        sqlx::query(
            "UPDATE pipeline_run_steps SET status = $1, exit_code = $2, output = $3, finished_at = $4 WHERE id = $5",
        )
        .bind(&req.status)
        .bind(req.exit_code)
        .bind(&req.output)
        .bind(now)
        .bind(step_id)
        .execute(pool)
        .await
    };

    match result {
        Ok(_) => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({"updated": true})),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Complete a job (called by runner after all steps finish).
pub async fn complete_job(
    State(state): State<AppState>,
    Path((_runner_id_str, job_id_str)): Path<(String, String)>,
    Json(req): Json<UpdateJobRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();

    let job_id = match Uuid::parse_str(&job_id_str) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("invalid job ID".into()).error_response()),
            )
                .into_response();
        }
    };

    let now = Utc::now();
    let outputs = req.outputs.unwrap_or(serde_json::Value::Null);

    // Update job status
    match sqlx::query(
        "UPDATE pipeline_run_jobs SET status = $1, outputs = $2, finished_at = $3 WHERE id = $4",
    )
    .bind(&req.status)
    .bind(outputs)
    .bind(now)
    .bind(job_id)
    .execute(pool)
    .await
    {
        Ok(_) => {
            // Check if all jobs in the run are done
            check_run_completion(pool, job_id, now).await;
            (
                axum::http::StatusCode::OK,
                Json(serde_json::json!({"completed": true})),
            )
                .into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

/// Resolve secrets for a runner job. Returns secret name→value pairs.
pub async fn resolve_secrets(
    State(state): State<AppState>,
    Path(_runner_id_str): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let pool = state.db.pool();

    let repo_id = match body.get("repo_id").and_then(|v| v.as_str()) {
        Some(id) => match Uuid::parse_str(id) {
            Ok(id) => id,
            Err(_) => {
                return (
                    axum::http::StatusCode::BAD_REQUEST,
                    Json(CoreError::NotFound("invalid repo ID".into()).error_response()),
                )
                    .into_response();
            }
        },
        None => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(CoreError::NotFound("missing repo_id".into()).error_response()),
            )
                .into_response();
        }
    };

    let secret_names: Vec<String> = body
        .get("secret_names")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // Validate runner token
    let token = match body.get("token").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                Json(CoreError::Auth("missing runner token".into()).error_response()),
            )
                .into_response();
        }
    };
    let token_hash = hash_token(token);

    let _runner: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM runners WHERE token_hash = $1 AND status = 'online'")
            .bind(&token_hash)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    // Fetch and decrypt secrets (AES-256-GCM)
    let mut secrets = serde_json::Map::new();
    for name in &secret_names {
        let row: Option<(Vec<u8>, Vec<u8>)> = sqlx::query_as(
            "SELECT value_enc, nonce FROM pipeline_variables WHERE repo_id = $1 AND name = $2",
        )
        .bind(repo_id)
        .bind(name)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);
        if let Some((ciphertext, nonce)) = row {
            match crate::auth::permission_engine::decrypt_value(&ciphertext, &nonce) {
                Ok(plaintext) => {
                    secrets.insert(name.clone(), serde_json::Value::String(plaintext));
                }
                Err(e) => {
                    tracing::warn!(secret = %name, error = %e, "failed to decrypt secret");
                }
            }
        }
    }

    (
        axum::http::StatusCode::OK,
        Json(serde_json::Value::Object(secrets)),
    )
        .into_response()
}

/// Cancel in-progress pipeline runs in the same concurrency group.
/// Called when a new run is created with a concurrency group.
pub(crate) async fn enforce_concurrency(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    concurrency_group: &str,
    cancel_in_progress: bool,
) -> std::result::Result<(), sqlx::Error> {
    if !cancel_in_progress || concurrency_group.is_empty() {
        return Ok(());
    }

    sqlx::query(
        "UPDATE pipeline_runs SET status = 'canceled', finished_at = NOW() WHERE repo_id = $1 AND concurrency_group = $2 AND status IN ('pending', 'queued', 'running')",
    )
    .bind(repo_id)
    .bind(concurrency_group)
    .execute(pool)
    .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Find the next available pending job for a runner.
async fn find_available_job(
    pool: &sqlx::PgPool,
    runner_id: Uuid,
) -> std::result::Result<Option<PollJobResponse>, sqlx::Error> {
    // Get runner labels for matching
    let runner: Option<RunnerRow> = sqlx::query_as(
        "SELECT id, name, description, scope, labels, status, runner_group, last_seen_at, created_at FROM runners WHERE id = $1",
    )
    .bind(runner_id)
    .fetch_optional(pool)
    .await?;

    let _runner = match runner {
        Some(r) => r,
        None => return Ok(None),
    };

    // Find a pending job
    let job: Option<(Uuid, String, Uuid, Option<serde_json::Value>)> = sqlx::query_as(
        "SELECT rj.id, rj.name, rj.run_id, pj.services FROM pipeline_run_jobs rj JOIN pipeline_run_steps prs ON prs.run_job_id = rj.id JOIN pipeline_jobs pj ON pj.id = rj.job_id WHERE rj.status = 'pending' LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    let (run_job_id, job_name, run_id, services) = match job {
        Some(j) => j,
        None => return Ok(None),
    };

    // Get run info including repo owner
    let run: Option<(String, String, String, String)> = sqlx::query_as(
        "SELECT pr.ref_name, pr.commit_sha, r.name, u.username FROM pipeline_runs pr JOIN repositories r ON r.id = pr.repo_id JOIN users u ON u.id = r.owner_id WHERE pr.id = $1",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await?;

    let (ref_name, commit_sha, repo_name, repo_owner) = match run {
        Some(r) => r,
        None => return Ok(None),
    };

    // Get steps for this job
    let steps: Vec<StepSpecRow> = sqlx::query_as(
        "SELECT name, step_index, step_type, commands, action, action_params, image, workdir, env, continue_on_error, timeout, condition FROM pipeline_job_steps WHERE job_id = (SELECT job_id FROM pipeline_run_jobs WHERE id = $1) ORDER BY step_index",
    )
    .bind(run_job_id)
    .fetch_all(pool)
    .await?;

    Ok(Some(PollJobResponse {
        job_id: run_job_id.to_string(),
        run_id: run_id.to_string(),
        name: job_name,
        steps: steps.into_iter().map(|s| s.into()).collect(),
        repo_url: format!("/{repo_owner}/{repo_name}.git"),
        commit_sha,
        ref_name,
        env: serde_json::json!({}),
        secret_names: vec![],
        services: services.unwrap_or(serde_json::Value::Null),
        timeout: None,
    }))
}

/// Check if all jobs in a run are complete, and update run status accordingly.
async fn check_run_completion(
    pool: &sqlx::PgPool,
    job_id: Uuid,
    now: chrono::DateTime<chrono::Utc>,
) {
    let run_id: Option<(Uuid,)> =
        sqlx::query_as("SELECT run_id FROM pipeline_run_jobs WHERE id = $1")
            .bind(job_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    if let Some((run_id,)) = run_id {
        // Check if all jobs are done
        let pending: Option<(i64,)> = sqlx::query_as(
            "SELECT COUNT(*) FROM pipeline_run_jobs WHERE run_id = $1 AND status NOT IN ('success', 'failure', 'canceled', 'skipped')",
        )
        .bind(run_id)
        .fetch_one(pool)
        .await
        .ok();

        if let Some((count,)) = pending {
            if count == 0 {
                // All jobs done — check for any failures
                let failed: Option<(i64,)> = sqlx::query_as(
                    "SELECT COUNT(*) FROM pipeline_run_jobs WHERE run_id = $1 AND status = 'failure'",
                )
                .bind(run_id)
                .fetch_one(pool)
                .await
                .ok();

                let final_status = match failed {
                    Some((f,)) if f > 0 => "failure",
                    _ => "success",
                };

                let _ = sqlx::query(
                    "UPDATE pipeline_runs SET status = $1, finished_at = $2 WHERE id = $3",
                )
                .bind(final_status)
                .bind(now)
                .bind(run_id)
                .execute(pool)
                .await;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// DB row types
// ---------------------------------------------------------------------------

#[derive(Debug, sqlx::FromRow)]
struct RunnerRow {
    id: Uuid,
    name: String,
    description: String,
    scope: String,
    labels: serde_json::Value,
    status: String,
    runner_group: Option<String>,
    last_seen_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<RunnerRow> for RunnerResponse {
    fn from(r: RunnerRow) -> Self {
        let labels: Vec<String> = if let Some(arr) = r.labels.as_array() {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        } else {
            vec![]
        };
        Self {
            id: r.id.to_string(),
            name: r.name,
            description: r.description,
            scope: r.scope,
            labels,
            status: r.status,
            runner_group: r.runner_group,
            last_seen_at: r.last_seen_at.map(|t| t.to_rfc3339()),
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct StepSpecRow {
    name: String,
    step_index: i32,
    step_type: String,
    commands: Option<serde_json::Value>,
    action: Option<String>,
    action_params: Option<serde_json::Value>,
    image: Option<String>,
    workdir: String,
    env: Option<serde_json::Value>,
    continue_on_error: bool,
    timeout: Option<String>,
    condition: Option<String>,
}

impl From<StepSpecRow> for JobStepSpec {
    fn from(r: StepSpecRow) -> Self {
        let commands: Option<Vec<String>> = r.commands.and_then(|v| serde_json::from_value(v).ok());
        Self {
            name: r.name,
            step_index: r.step_index,
            step_type: r.step_type,
            commands,
            action: r.action,
            action_params: r.action_params,
            image: r.image,
            workdir: r.workdir,
            env: r
                .env
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
            continue_on_error: r.continue_on_error,
            timeout: r.timeout,
            condition: r.condition,
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
    fn test_default_scope() {
        assert_eq!(default_scope(), "global");
    }

    #[test]
    fn test_generate_runner_token_length() {
        let token = generate_runner_token();
        assert_eq!(token.len(), 64); // 32 bytes hex
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_token_deterministic() {
        let h1 = hash_token("test-token");
        let h2 = hash_token("test-token");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn test_hash_token_different_inputs() {
        let h1 = hash_token("token-a");
        let h2 = hash_token("token-b");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_register_runner_request_deserialize() {
        let json = r#"{"name": "linux-runner", "labels": ["linux", "amd64"]}"#;
        let req: RegisterRunnerRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "linux-runner");
        assert_eq!(req.labels, vec!["linux", "amd64"]);
        assert_eq!(req.scope, "global");
    }

    #[test]
    fn test_register_runner_request_with_scope() {
        let json = r#"{"name": "repo-runner", "scope": "repo", "labels": ["linux"]}"#;
        let req: RegisterRunnerRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.scope, "repo");
    }

    #[test]
    fn test_update_step_request_deserialize() {
        let json = r#"{"status": "success", "exit_code": 0, "output": "Done"}"#;
        let req: UpdateStepRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.status, "success");
        assert_eq!(req.exit_code, Some(0));
        assert_eq!(req.output, Some("Done".to_string()));
    }

    #[test]
    fn test_update_job_request_deserialize() {
        let json = r#"{"status": "success", "outputs": {"artifact": "build.tar.gz"}}"#;
        let req: UpdateJobRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.status, "success");
        assert!(req.outputs.is_some());
    }

    #[test]
    fn test_runner_response_serialize() {
        let resp = RunnerResponse {
            id: "00000000-0000-0000-0000-000000000001".to_string(),
            name: "linux-runner".to_string(),
            description: String::new(),
            scope: "global".to_string(),
            labels: vec!["linux".to_string()],
            status: "online".to_string(),
            runner_group: None,
            last_seen_at: None,
            created_at: "2025-01-01T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("linux-runner"));
        assert!(json.contains("online"));
    }

    #[test]
    fn test_job_step_spec_roundtrip() {
        let spec = JobStepSpec {
            name: "build".to_string(),
            step_index: 0,
            step_type: "run".to_string(),
            commands: Some(vec!["cargo build".to_string()]),
            action: None,
            action_params: None,
            image: Some("rust:1.75".to_string()),
            workdir: String::new(),
            env: serde_json::json!({"RUST_BACKTRACE": "1"}),
            continue_on_error: false,
            timeout: Some("30m".to_string()),
            condition: None,
        };
        let json = serde_json::to_string(&spec).unwrap();
        let decoded: JobStepSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, "build");
        assert_eq!(decoded.step_type, "run");
        assert_eq!(decoded.commands.as_deref().unwrap().len(), 1);
    }
}
