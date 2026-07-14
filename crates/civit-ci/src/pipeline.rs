//! CI/CD Pipeline types and logic.
//!
//! Contains pipeline run types, graph visualization types, and pipeline creation logic.
//! Route handlers that depend on AppState live in civit-core and use these types.

#![forbid(unsafe_code)]

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

pub fn default_yaml_path() -> String {
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

pub fn default_limit() -> i64 {
    50
}

#[derive(Debug, Deserialize)]
pub struct CancelPipelineRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphResponse {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub layout: GraphLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub name: String,
    pub status: String,
    pub job_index: i32,
    pub runner_id: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLayout {
    pub rank_direction: String,
    pub node_spacing: u32,
    pub rank_spacing: u32,
}

// ---------------------------------------------------------------------------
// DB row types
// ---------------------------------------------------------------------------

#[derive(Debug, sqlx::FromRow)]
pub struct PipelineRunRow {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub trigger: String,
    pub ref_name: Option<String>,
    pub commit_sha: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
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
pub struct RunJobRow {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub runner_id: Option<Uuid>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct RunStepRow {
    pub id: Uuid,
    pub name: String,
    pub step_index: i32,
    pub status: String,
    pub image: Option<String>,
    pub exit_code: Option<i32>,
    pub output: Option<String>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
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

#[derive(Debug, sqlx::FromRow)]
pub struct RunJobGraphRow {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub job_index: i32,
    pub runner_id: Option<Uuid>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub needs: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Pipeline Artifacts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactResponse {
    pub id: String,
    pub pipeline_run_id: String,
    pub job_id: String,
    pub name: String,
    pub path: String,
    pub size_bytes: i64,
    pub content_type: String,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ArtifactRow {
    pub id: Uuid,
    pub pipeline_run_id: Uuid,
    pub job_id: Uuid,
    pub name: String,
    pub path: String,
    pub size_bytes: i64,
    pub content_type: String,
    pub storage_key: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<ArtifactRow> for ArtifactResponse {
    fn from(r: ArtifactRow) -> Self {
        Self {
            id: r.id.to_string(),
            pipeline_run_id: r.pipeline_run_id.to_string(),
            job_id: r.job_id.to_string(),
            name: r.name,
            path: r.path,
            size_bytes: r.size_bytes,
            content_type: r.content_type,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

// ---------------------------------------------------------------------------
// Pipeline Environments
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentResponse {
    pub id: String,
    pub repo_id: String,
    pub name: String,
    pub url: Option<String>,
    pub protected: bool,
    pub auto_deploy: bool,
    pub deployment_branch_policy: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct EnvironmentRow {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub url: Option<String>,
    pub protected: bool,
    pub auto_deploy: bool,
    pub deployment_branch_policy: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<EnvironmentRow> for EnvironmentResponse {
    fn from(r: EnvironmentRow) -> Self {
        Self {
            id: r.id.to_string(),
            repo_id: r.repo_id.to_string(),
            name: r.name,
            url: r.url,
            protected: r.protected,
            auto_deploy: r.auto_deploy,
            deployment_branch_policy: r.deployment_branch_policy,
            created_at: r.created_at.to_rfc3339(),
            updated_at: r.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionResponse {
    pub id: String,
    pub environment_id: String,
    pub required_approvals: i32,
    pub wait_timer: i32,
    pub allow_admin_override: bool,
    pub allowed_branches: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ProtectionRow {
    pub id: Uuid,
    pub environment_id: Uuid,
    pub required_approvals: i32,
    pub wait_timer: i32,
    pub allow_admin_override: bool,
    pub allowed_branches: Option<Vec<String>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ProtectionRow> for ProtectionResponse {
    fn from(r: ProtectionRow) -> Self {
        Self {
            id: r.id.to_string(),
            environment_id: r.environment_id.to_string(),
            required_approvals: r.required_approvals,
            wait_timer: r.wait_timer,
            allow_admin_override: r.allow_admin_override,
            allowed_branches: r.allowed_branches.unwrap_or_default(),
            created_at: r.created_at.to_rfc3339(),
            updated_at: r.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentLockResponse {
    pub id: String,
    pub environment_id: String,
    pub user_id: String,
    pub reason: String,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct DeploymentLockRow {
    pub id: Uuid,
    pub environment_id: Uuid,
    pub user_id: Uuid,
    pub reason: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<DeploymentLockRow> for DeploymentLockResponse {
    fn from(r: DeploymentLockRow) -> Self {
        Self {
            id: r.id.to_string(),
            environment_id: r.environment_id.to_string(),
            user_id: r.user_id.to_string(),
            reason: r.reason,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentDeploymentResponse {
    pub id: String,
    pub environment_id: String,
    pub pipeline_run_id: Option<String>,
    pub sha: String,
    pub status: String,
    pub creator_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct EnvironmentDeploymentRow {
    pub id: Uuid,
    pub environment_id: Uuid,
    pub pipeline_run_id: Option<Uuid>,
    pub sha: String,
    pub status: String,
    pub creator_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<EnvironmentDeploymentRow> for EnvironmentDeploymentResponse {
    fn from(r: EnvironmentDeploymentRow) -> Self {
        Self {
            id: r.id.to_string(),
            environment_id: r.environment_id.to_string(),
            pipeline_run_id: r.pipeline_run_id.map(|id| id.to_string()),
            sha: r.sha,
            status: r.status,
            creator_id: r.creator_id.map(|id| id.to_string()),
            created_at: r.created_at.to_rfc3339(),
            updated_at: r.updated_at.to_rfc3339(),
        }
    }
}

// ---------------------------------------------------------------------------
// Artifact DB operations
// ---------------------------------------------------------------------------

/// Create a new artifact record.
pub async fn create_artifact(
    pool: &sqlx::PgPool,
    pipeline_run_id: Uuid,
    job_id: Uuid,
    name: &str,
    path: &str,
    size_bytes: i64,
    content_type: &str,
    storage_key: &str,
) -> std::result::Result<ArtifactResponse, sqlx::Error> {
    sqlx::query_as::<_, ArtifactRow>(
        "INSERT INTO pipeline_artifacts (pipeline_run_id, job_id, name, path, size_bytes, content_type, storage_key) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         RETURNING *",
    )
    .bind(pipeline_run_id)
    .bind(job_id)
    .bind(name)
    .bind(path)
    .bind(size_bytes)
    .bind(content_type)
    .bind(storage_key)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// List artifacts for a pipeline run.
pub async fn list_artifacts(
    pool: &sqlx::PgPool,
    pipeline_run_id: Uuid,
) -> std::result::Result<Vec<ArtifactResponse>, sqlx::Error> {
    sqlx::query_as::<_, ArtifactRow>(
        "SELECT * FROM pipeline_artifacts WHERE pipeline_run_id = $1 ORDER BY created_at",
    )
    .bind(pipeline_run_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Get artifact by ID.
pub async fn get_artifact(
    pool: &sqlx::PgPool,
    artifact_id: Uuid,
) -> std::result::Result<Option<ArtifactRow>, sqlx::Error> {
    sqlx::query_as::<_, ArtifactRow>("SELECT * FROM pipeline_artifacts WHERE id = $1")
        .bind(artifact_id)
        .fetch_optional(pool)
        .await
}

/// Delete artifact by ID.
pub async fn delete_artifact(
    pool: &sqlx::PgPool,
    artifact_id: Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM pipeline_artifacts WHERE id = $1")
        .bind(artifact_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// Environment DB operations
// ---------------------------------------------------------------------------

/// Create a new environment.
pub async fn create_environment(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    name: &str,
    url: Option<&str>,
    protected: bool,
    auto_deploy: bool,
) -> std::result::Result<EnvironmentResponse, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentRow>(
        "INSERT INTO pipeline_environments (repo_id, name, url, protected, auto_deploy) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING *",
    )
    .bind(repo_id)
    .bind(name)
    .bind(url)
    .bind(protected)
    .bind(auto_deploy)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Create a new environment v2 with deployment branch policy.
pub async fn create_environment_v2(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    name: &str,
    url: Option<&str>,
    protected: bool,
    auto_deploy: bool,
    deployment_branch_policy: Option<&serde_json::Value>,
) -> std::result::Result<EnvironmentResponse, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentRow>(
        "INSERT INTO pipeline_environments_v2 (repo_id, name, url, protected, auto_deploy, deployment_branch_policy) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING *",
    )
    .bind(repo_id)
    .bind(name)
    .bind(url)
    .bind(protected)
    .bind(auto_deploy)
    .bind(deployment_branch_policy)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// List environments for a repo.
pub async fn list_environments(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
) -> std::result::Result<Vec<EnvironmentResponse>, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentRow>(
        "SELECT * FROM pipeline_environments WHERE repo_id = $1 ORDER BY name",
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// List environments v2 for a repo.
pub async fn list_environments_v2(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
) -> std::result::Result<Vec<EnvironmentResponse>, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentRow>(
        "SELECT * FROM pipeline_environments_v2 WHERE repo_id = $1 ORDER BY name",
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Get environment by ID.
pub async fn get_environment(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
) -> std::result::Result<Option<EnvironmentRow>, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentRow>("SELECT * FROM pipeline_environments WHERE id = $1")
        .bind(environment_id)
        .fetch_optional(pool)
        .await
}

/// Update environment.
pub async fn update_environment(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
    name: Option<&str>,
    url: Option<&str>,
    protected: Option<bool>,
    auto_deploy: Option<bool>,
) -> std::result::Result<EnvironmentResponse, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentRow>(
        "UPDATE pipeline_environments \
         SET name = COALESCE($2, name), \
             url = COALESCE($3, url), \
             protected = COALESCE($4, protected), \
             auto_deploy = COALESCE($5, auto_deploy), \
             updated_at = NOW() \
         WHERE id = $1 \
         RETURNING *",
    )
    .bind(environment_id)
    .bind(name)
    .bind(url)
    .bind(protected)
    .bind(auto_deploy)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Update environment v2 with deployment branch policy.
pub async fn update_environment_v2(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
    name: Option<&str>,
    url: Option<&str>,
    protected: Option<bool>,
    auto_deploy: Option<bool>,
    deployment_branch_policy: Option<&serde_json::Value>,
) -> std::result::Result<EnvironmentResponse, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentRow>(
        "UPDATE pipeline_environments_v2 \
         SET name = COALESCE($2, name), \
             url = COALESCE($3, url), \
             protected = COALESCE($4, protected), \
             auto_deploy = COALESCE($5, auto_deploy), \
             deployment_branch_policy = COALESCE($6, deployment_branch_policy), \
             updated_at = NOW() \
         WHERE id = $1 \
         RETURNING *",
    )
    .bind(environment_id)
    .bind(name)
    .bind(url)
    .bind(protected)
    .bind(auto_deploy)
    .bind(deployment_branch_policy)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Delete environment by ID.
pub async fn delete_environment(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM pipeline_environments WHERE id = $1")
        .bind(environment_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Get protection rules for an environment.
pub async fn get_protections(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
) -> std::result::Result<Option<ProtectionResponse>, sqlx::Error> {
    sqlx::query_as::<_, ProtectionRow>(
        "SELECT * FROM deployment_protections WHERE environment_id = $1",
    )
    .bind(environment_id)
    .fetch_optional(pool)
    .await
    .map(|r| r.map(|r| r.into()))
}

/// Upsert protection rules for an environment.
pub async fn upsert_protections(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
    required_approvals: i32,
    wait_timer: i32,
    allow_admin_override: bool,
    allowed_branches: &[String],
) -> std::result::Result<ProtectionResponse, sqlx::Error> {
    sqlx::query_as::<_, ProtectionRow>(
        "INSERT INTO deployment_protections (environment_id, required_approvals, wait_timer, allow_admin_override, allowed_branches) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (environment_id) DO UPDATE \
         SET required_approvals = EXCLUDED.required_approvals, \
             wait_timer = EXCLUDED.wait_timer, \
             allow_admin_override = EXCLUDED.allow_admin_override, \
             allowed_branches = EXCLUDED.allowed_branches, \
             updated_at = NOW() \
         RETURNING *",
    )
    .bind(environment_id)
    .bind(required_approvals)
    .bind(wait_timer)
    .bind(allow_admin_override)
    .bind(allowed_branches)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Create a deployment in an environment.
pub async fn create_environment_deployment(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
    pipeline_run_id: Option<Uuid>,
    sha: &str,
    creator_id: Option<Uuid>,
) -> std::result::Result<EnvironmentDeploymentResponse, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentDeploymentRow>(
        "INSERT INTO environment_deployments (environment_id, pipeline_run_id, sha, creator_id) \
         VALUES ($1, $2, $3, $4) \
         RETURNING *",
    )
    .bind(environment_id)
    .bind(pipeline_run_id)
    .bind(sha)
    .bind(creator_id)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// List deployments for an environment.
pub async fn list_environment_deployments(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
    limit: i64,
    offset: i64,
) -> std::result::Result<Vec<EnvironmentDeploymentResponse>, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentDeploymentRow>(
        "SELECT * FROM environment_deployments WHERE environment_id = $1 \
         ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(environment_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Update deployment status.
pub async fn update_deployment_status(
    pool: &sqlx::PgPool,
    deployment_id: Uuid,
    status: &str,
) -> std::result::Result<EnvironmentDeploymentResponse, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentDeploymentRow>(
        "UPDATE environment_deployments SET status = $2, updated_at = NOW() \
         WHERE id = $1 RETURNING *",
    )
    .bind(deployment_id)
    .bind(status)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

// ---------------------------------------------------------------------------
// Deployment Locks
// ---------------------------------------------------------------------------

/// Create a deployment lock for an environment.
pub async fn create_deployment_lock(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
    user_id: Uuid,
    reason: &str,
) -> std::result::Result<DeploymentLockResponse, sqlx::Error> {
    sqlx::query_as::<_, DeploymentLockRow>(
        "INSERT INTO deployment_locks (environment_id, user_id, reason) \
         VALUES ($1, $2, $3) \
         RETURNING *",
    )
    .bind(environment_id)
    .bind(user_id)
    .bind(reason)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// List deployment locks for an environment.
pub async fn list_deployment_locks(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
) -> std::result::Result<Vec<DeploymentLockResponse>, sqlx::Error> {
    sqlx::query_as::<_, DeploymentLockRow>(
        "SELECT * FROM deployment_locks WHERE environment_id = $1 ORDER BY created_at",
    )
    .bind(environment_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Remove a deployment lock.
pub async fn remove_deployment_lock(
    pool: &sqlx::PgPool,
    lock_id: Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM deployment_locks WHERE id = $1")
        .bind(lock_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Check if an environment is locked.
pub async fn is_environment_locked(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM deployment_locks WHERE environment_id = $1",
    )
    .bind(environment_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(c,)| c > 0).unwrap_or(false))
}

// ---------------------------------------------------------------------------
// Environment Approval Rules
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRuleResponse {
    pub id: String,
    pub environment_id: String,
    pub required_approvers: i32,
    pub approver_groups: Vec<String>,
    pub auto_approve_after_hours: Option<i32>,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ApprovalRuleRow {
    pub id: Uuid,
    pub environment_id: Uuid,
    pub required_approvers: i32,
    pub approver_groups: Option<Vec<String>>,
    pub auto_approve_after_hours: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<ApprovalRuleRow> for ApprovalRuleResponse {
    fn from(r: ApprovalRuleRow) -> Self {
        Self {
            id: r.id.to_string(),
            environment_id: r.environment_id.to_string(),
            required_approvers: r.required_approvers,
            approver_groups: r.approver_groups.unwrap_or_default(),
            auto_approve_after_hours: r.auto_approve_after_hours,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentApprovalResponse {
    pub id: String,
    pub environment_id: String,
    pub deployment_id: String,
    pub approver_id: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct EnvironmentApprovalRow {
    pub id: Uuid,
    pub environment_id: Uuid,
    pub deployment_id: Uuid,
    pub approver_id: Uuid,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<EnvironmentApprovalRow> for EnvironmentApprovalResponse {
    fn from(r: EnvironmentApprovalRow) -> Self {
        Self {
            id: r.id.to_string(),
            environment_id: r.environment_id.to_string(),
            deployment_id: r.deployment_id.to_string(),
            approver_id: r.approver_id.to_string(),
            status: r.status,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

/// Create or update approval rules for an environment.
pub async fn upsert_approval_rules(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
    required_approvers: i32,
    approver_groups: &[String],
    auto_approve_after_hours: Option<i32>,
) -> std::result::Result<ApprovalRuleResponse, sqlx::Error> {
    sqlx::query_as::<_, ApprovalRuleRow>(
        "INSERT INTO environment_approval_rules (environment_id, required_approvers, approver_groups, auto_approve_after_hours) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (environment_id) DO UPDATE \
         SET required_approvers = $2, approver_groups = $3, auto_approve_after_hours = $4 \
         RETURNING *",
    )
    .bind(environment_id)
    .bind(required_approvers)
    .bind(approver_groups)
    .bind(auto_approve_after_hours)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Get approval rules for an environment.
pub async fn get_approval_rules(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
) -> std::result::Result<Option<ApprovalRuleResponse>, sqlx::Error> {
    sqlx::query_as::<_, ApprovalRuleRow>(
        "SELECT * FROM environment_approval_rules WHERE environment_id = $1",
    )
    .bind(environment_id)
    .fetch_optional(pool)
    .await
    .map(|r| r.map(|r| r.into()))
}

/// Create an approval request for a deployment.
pub async fn create_deployment_approval(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
    deployment_id: Uuid,
    approver_id: Uuid,
) -> std::result::Result<EnvironmentApprovalResponse, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentApprovalRow>(
        "INSERT INTO environment_approvals (environment_id, deployment_id, approver_id, status) \
         VALUES ($1, $2, $3, 'pending') \
         RETURNING *",
    )
    .bind(environment_id)
    .bind(deployment_id)
    .bind(approver_id)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Approve a deployment.
pub async fn approve_deployment(
    pool: &sqlx::PgPool,
    approval_id: Uuid,
) -> std::result::Result<EnvironmentApprovalResponse, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentApprovalRow>(
        "UPDATE environment_approvals SET status = 'approved' WHERE id = $1 RETURNING *",
    )
    .bind(approval_id)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Reject a deployment.
pub async fn reject_deployment(
    pool: &sqlx::PgPool,
    approval_id: Uuid,
) -> std::result::Result<EnvironmentApprovalResponse, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentApprovalRow>(
        "UPDATE environment_approvals SET status = 'rejected' WHERE id = $1 RETURNING *",
    )
    .bind(approval_id)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// List approvals for a deployment.
pub async fn list_deployment_approvals(
    pool: &sqlx::PgPool,
    deployment_id: Uuid,
) -> std::result::Result<Vec<EnvironmentApprovalResponse>, sqlx::Error> {
    sqlx::query_as::<_, EnvironmentApprovalRow>(
        "SELECT * FROM environment_approvals WHERE deployment_id = $1 ORDER BY created_at DESC",
    )
    .bind(deployment_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Check if a deployment is fully approved.
pub async fn is_deployment_approved(
    pool: &sqlx::PgPool,
    deployment_id: Uuid,
    environment_id: Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let rules: Option<ApprovalRuleRow> = sqlx::query_as(
        "SELECT * FROM environment_approval_rules WHERE environment_id = $1",
    )
    .bind(environment_id)
    .fetch_optional(pool)
    .await?;

    let required = rules.map(|r| r.required_approvers).unwrap_or(0);
    if required == 0 {
        return Ok(true);
    }

    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM environment_approvals \
         WHERE deployment_id = $1 AND status = 'approved'",
    )
    .bind(deployment_id)
    .fetch_one(pool)
    .await?;

    Ok(row.0 >= required as i64)
}

/// Auto-approve deployments that exceed the auto_approve_after_hours threshold.
pub async fn auto_approve_pending(
    pool: &sqlx::PgPool,
    environment_id: Uuid,
) -> std::result::Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE environment_approvals ea \
         SET status = 'approved' \
         FROM environment_approval_rules ear \
         WHERE ea.environment_id = ear.environment_id \
         AND ea.status = 'pending' \
         AND ear.auto_approve_after_hours IS NOT NULL \
         AND ea.created_at < NOW() - (ear.auto_approve_after_hours || ' hours')::INTERVAL \
         AND ea.environment_id = $1",
    )
    .bind(environment_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() as i64)
}

// ---------------------------------------------------------------------------
// Pipeline creation logic
// ---------------------------------------------------------------------------

/// Create a pipeline definition, its jobs/steps, and the initial run.
#[allow(clippy::too_many_arguments)]
pub async fn create_pipeline_run(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    yaml_path: &str,
    ref_name: &str,
    commit_sha: &str,
    trigger: &str,
    yaml_content: &str,
    pipeline: &civit_pipeline::Pipeline,
) -> std::result::Result<PipelineRunResponse, sqlx::Error> {
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

    let concurrency_group = pipeline.concurrency.as_ref().and_then(|c| c.group.clone());

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

// ---------------------------------------------------------------------------
// DB query helpers
// ---------------------------------------------------------------------------

/// List pipeline runs for a repo.
pub async fn list_pipelines_by_repo(
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
pub async fn list_all_pipelines_db(
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
pub async fn get_pipeline_detail(
    pool: &sqlx::PgPool,
    run_id: Uuid,
) -> std::result::Result<Option<PipelineRunDetailResponse>, sqlx::Error> {
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

    let jobs = get_pipeline_jobs_db(pool, run_id).await?;

    Ok(Some(PipelineRunDetailResponse {
        run: run.into(),
        jobs,
    }))
}

/// Get pipeline graph (DAG) visualization data.
pub async fn get_pipeline_graph_db(
    pool: &sqlx::PgPool,
    run_id: Uuid,
) -> std::result::Result<GraphResponse, sqlx::Error> {
    let run_jobs: Vec<RunJobGraphRow> = sqlx::query_as(
        "SELECT prj.id, prj.name, prj.status, pj.job_index, prj.runner_id, prj.started_at, prj.finished_at, pj.needs
         FROM pipeline_run_jobs prj
         JOIN pipeline_jobs pj ON prj.job_id = pj.id
         WHERE prj.run_id = $1
         ORDER BY pj.job_index",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await?;

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for rj in &run_jobs {
        nodes.push(GraphNode {
            id: rj.id.to_string(),
            name: rj.name.clone(),
            status: rj.status.clone(),
            job_index: rj.job_index,
            runner_id: rj.runner_id.map(|id| id.to_string()),
            started_at: rj.started_at.map(|t| t.to_rfc3339()),
            finished_at: rj.finished_at.map(|t| t.to_rfc3339()),
        });

        if let Some(ref needs) = rj.needs
            && let Ok(deps) = serde_json::from_value::<Vec<String>>(needs.clone())
        {
            for dep_name in deps {
                if let Some(source) = run_jobs.iter().find(|j| j.name == dep_name) {
                    edges.push(GraphEdge {
                        source: source.id.to_string(),
                        target: rj.id.to_string(),
                    });
                }
            }
        }
    }

    Ok(GraphResponse {
        nodes,
        edges,
        layout: GraphLayout {
            rank_direction: "LR".to_string(),
            node_spacing: 50,
            rank_spacing: 80,
        },
    })
}

/// Get jobs for a pipeline run with their steps.
pub async fn get_pipeline_jobs_db(
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
pub async fn cancel_pipeline_run(
    pool: &sqlx::PgPool,
    run_id: Uuid,
) -> std::result::Result<Option<PipelineRunResponse>, sqlx::Error> {
    use sqlx::Row as _;
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
    fn test_pipeline_status_transitions() {
        let statuses = [
            "pending", "queued", "running", "success", "failed", "canceled",
        ];
        for s in statuses {
            let resp = PipelineRunResponse {
                id: "id".into(),
                repo_id: "repo".into(),
                trigger: "push".into(),
                ref_name: None,
                commit_sha: "sha".into(),
                status: s.to_string(),
                created_at: "2025-01-01T00:00:00Z".into(),
                started_at: None,
                finished_at: None,
            };
            let json = serde_json::to_string(&resp).unwrap();
            assert!(json.contains(&format!("\"status\":\"{s}\"")));
        }
    }

    #[test]
    fn test_run_response_with_started_and_finished() {
        let resp = PipelineRunResponse {
            id: "run-1".into(),
            repo_id: "repo-1".into(),
            trigger: "webhook".into(),
            ref_name: Some("develop".into()),
            commit_sha: "deadbeef".into(),
            status: "success".into(),
            created_at: "2025-06-01T00:00:00Z".into(),
            started_at: Some("2025-06-01T00:00:05Z".into()),
            finished_at: Some("2025-06-01T00:01:30Z".into()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("2025-06-01T00:00:05Z"));
        assert!(json.contains("2025-06-01T00:01:30Z"));
    }

    #[test]
    fn test_run_response_null_ref_name() {
        let resp = PipelineRunResponse {
            id: "run-1".into(),
            repo_id: "repo-1".into(),
            trigger: "manual".into(),
            ref_name: None,
            commit_sha: "abc".into(),
            status: "pending".into(),
            created_at: "2025-01-01T00:00:00Z".into(),
            started_at: None,
            finished_at: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("null") || !json.contains("ref_name"));
    }

    #[test]
    fn test_run_job_response_serialize() {
        let job = RunJobResponse {
            id: "job-1".into(),
            name: "build".into(),
            status: "running".into(),
            runner_id: Some("runner-1".into()),
            started_at: Some("2025-06-01T00:00:10Z".into()),
            finished_at: None,
            steps: vec![RunStepResponse {
                id: "step-1".into(),
                name: "compile".into(),
                step_index: 0,
                status: "success".into(),
                image: Some("rust:latest".into()),
                exit_code: Some(0),
                output: Some("done".into()),
                started_at: Some("2025-06-01T00:00:11Z".into()),
                finished_at: Some("2025-06-01T00:00:20Z".into()),
            }],
        };
        let json = serde_json::to_string(&job).unwrap();
        assert!(json.contains("build"));
        assert!(json.contains("compile"));
    }

    #[test]
    fn test_graph_response_serialize() {
        let graph = GraphResponse {
            nodes: vec![
                GraphNode {
                    id: "n1".into(),
                    name: "build".into(),
                    status: "success".into(),
                    job_index: 0,
                    runner_id: None,
                    started_at: None,
                    finished_at: None,
                },
                GraphNode {
                    id: "n2".into(),
                    name: "test".into(),
                    status: "pending".into(),
                    job_index: 1,
                    runner_id: None,
                    started_at: None,
                    finished_at: None,
                },
            ],
            edges: vec![GraphEdge {
                source: "n1".into(),
                target: "n2".into(),
            }],
            layout: GraphLayout {
                rank_direction: "LR".into(),
                node_spacing: 50,
                rank_spacing: 80,
            },
        };
        let json = serde_json::to_string(&graph).unwrap();
        assert!(json.contains("build"));
        assert!(json.contains("LR"));
    }

    #[test]
    fn test_trigger_request_missing_fields_fails() {
        let json = r#"{"ref_name": "main"}"#;
        let result = serde_json::from_str::<TriggerPipelineRequest>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_trigger_request_with_all_fields() {
        let json = r#"{"ref_name": "main", "commit_sha": "abc123", "yaml_path": ".civit/custom.yaml", "event_type": "pull_request", "changed_files": ["a.rs", "b.rs"]}"#;
        let req: TriggerPipelineRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.yaml_path, ".civit/custom.yaml");
        assert_eq!(req.event_type, Some("pull_request".into()));
        assert_eq!(req.changed_files, vec!["a.rs", "b.rs"]);
    }

    #[test]
    fn test_pipeline_list_params_with_status() {
        let json = r#"{"limit": 10, "offset": 5, "status": "failed"}"#;
        let params: PipelineListParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.limit, 10);
        assert_eq!(params.offset, 5);
        assert_eq!(params.status.as_deref(), Some("failed"));
    }

    #[test]
    fn test_cancel_request_defaults() {
        let json = r#"{}"#;
        let req: CancelPipelineRequest = serde_json::from_str(json).unwrap();
        assert!(req.reason.is_none());
    }

    #[test]
    fn test_cancel_request_with_reason() {
        let json = r#"{"reason": "rebuild needed"}"#;
        let req: CancelPipelineRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.reason.as_deref(), Some("rebuild needed"));
    }

    #[test]
    fn test_run_step_response_no_optional() {
        let step = RunStepResponse {
            id: "s1".into(),
            name: "lint".into(),
            step_index: 0,
            status: "success".into(),
            image: None,
            exit_code: None,
            output: None,
            started_at: None,
            finished_at: None,
        };
        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("lint"));
    }

    #[test]
    fn test_run_job_empty_steps() {
        let job = RunJobResponse {
            id: "j1".into(),
            name: "deploy".into(),
            status: "pending".into(),
            runner_id: None,
            started_at: None,
            finished_at: None,
            steps: vec![],
        };
        let json = serde_json::to_string(&job).unwrap();
        assert!(json.contains("[]") || json.contains("\"steps\":[]"));
    }

    #[test]
    fn test_detail_response_flatten() {
        let detail = PipelineRunDetailResponse {
            run: PipelineRunResponse {
                id: "r1".into(),
                repo_id: "repo1".into(),
                trigger: "push".into(),
                ref_name: Some("main".into()),
                commit_sha: "sha".into(),
                status: "running".into(),
                created_at: "2025-01-01T00:00:00Z".into(),
                started_at: None,
                finished_at: None,
            },
            jobs: vec![],
        };
        let json = serde_json::to_string(&detail).unwrap();
        assert!(json.contains("r1"));
        assert!(json.contains("jobs"));
    }

    #[test]
    fn test_graph_response_empty_nodes() {
        let graph = GraphResponse {
            nodes: vec![],
            edges: vec![],
            layout: GraphLayout {
                rank_direction: "TB".into(),
                node_spacing: 50,
                rank_spacing: 80,
            },
        };
        let json = serde_json::to_string(&graph).unwrap();
        assert!(json.contains("\"nodes\":[]"));
        assert!(json.contains("\"edges\":[]"));
    }

    #[test]
    fn test_graph_node_all_optionals_none() {
        let node = GraphNode {
            id: "n1".into(),
            name: "build".into(),
            status: "pending".into(),
            job_index: 0,
            runner_id: None,
            started_at: None,
            finished_at: None,
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("build"));
        assert!(json.contains("null"));
    }

    #[test]
    fn test_graph_node_all_optionals_some() {
        let node = GraphNode {
            id: "n1".into(),
            name: "test".into(),
            status: "running".into(),
            job_index: 1,
            runner_id: Some("runner-1".into()),
            started_at: Some("2025-01-01T00:00:00Z".into()),
            finished_at: Some("2025-01-01T00:01:00Z".into()),
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("runner-1"));
        assert!(json.contains("2025-01-01T00:00:00Z"));
    }

    #[test]
    fn test_graph_layout_custom() {
        let layout = GraphLayout {
            rank_direction: "BT".into(),
            node_spacing: 100,
            rank_spacing: 200,
        };
        let json = serde_json::to_string(&layout).unwrap();
        assert!(json.contains("BT"));
        assert!(json.contains("100"));
        assert!(json.contains("200"));
    }

    #[test]
    fn test_step_response_negative_exit_code() {
        let step = RunStepResponse {
            id: "s1".into(),
            name: "fail".into(),
            step_index: 0,
            status: "failed".into(),
            image: None,
            exit_code: Some(-1),
            output: Some("signal killed".into()),
            started_at: None,
            finished_at: None,
        };
        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("-1"));
    }

    #[test]
    fn test_run_job_response_multiple_steps() {
        let job = RunJobResponse {
            id: "j1".into(),
            name: "build".into(),
            status: "success".into(),
            runner_id: None,
            started_at: None,
            finished_at: None,
            steps: vec![
                RunStepResponse {
                    id: "s1".into(),
                    name: "compile".into(),
                    step_index: 0,
                    status: "success".into(),
                    image: None,
                    exit_code: Some(0),
                    output: None,
                    started_at: None,
                    finished_at: None,
                },
                RunStepResponse {
                    id: "s2".into(),
                    name: "test".into(),
                    step_index: 1,
                    status: "success".into(),
                    image: None,
                    exit_code: Some(0),
                    output: None,
                    started_at: None,
                    finished_at: None,
                },
            ],
        };
        let json = serde_json::to_string(&job).unwrap();
        assert!(json.contains("compile"));
        assert!(json.contains("test"));
    }

    #[test]
    fn test_trigger_request_with_empty_changed_files() {
        let json = r#"{"ref_name": "main", "commit_sha": "abc", "changed_files": []}"#;
        let req: TriggerPipelineRequest = serde_json::from_str(json).unwrap();
        assert!(req.changed_files.is_empty());
    }

    #[test]
    fn test_pipeline_run_response_long_strings() {
        let resp = PipelineRunResponse {
            id: "a".repeat(1000),
            repo_id: "b".repeat(1000),
            trigger: "c".repeat(1000),
            ref_name: Some("d".repeat(1000)),
            commit_sha: "e".repeat(1000),
            status: "f".repeat(1000),
            created_at: "2025-01-01T00:00:00Z".into(),
            started_at: None,
            finished_at: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains(&"a".repeat(1000)));
    }
}
