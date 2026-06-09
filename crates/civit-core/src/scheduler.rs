//! Pipeline scheduler for cron-triggered runs.
//!
//! Runs as a background task, checking every minute for due scheduled runs
//! and triggering pipeline executions.

#![forbid(unsafe_code)]

use chrono::Utc;
use civit_pipeline::trigger::{TriggerContext, compute_next_cron_run};
use civit_pipeline::{expand_matrix, matches_trigger, parse_pipeline, validate_pipeline};
use sqlx::Row;
use std::sync::Arc;
use tokio::sync::Notify;

/// Start the pipeline scheduler as a background task.
///
/// Returns a `Notify` that can be used to wake the scheduler immediately
/// (e.g., when a schedule is created or deleted).
pub fn start_scheduler(pool: sqlx::PgPool, storage_path: String) -> Arc<Notify> {
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;

            // Check for due schedules
            if let Err(e) = tick_schedules(&pool, &storage_path).await {
                tracing::warn!("scheduler tick error: {e}");
            }

            // Wait for next tick or manual wake
            tokio::select! {
                _ = interval.tick() => {},
                _ = notify_clone.notified() => {
                    // Immediately re-check
                    if let Err(e) = tick_schedules(&pool, &storage_path).await {
                        tracing::warn!("scheduler wake tick error: {e}");
                    }
                }
            }
        }
    });

    notify
}

/// Process all due scheduled runs.
async fn tick_schedules(
    pool: &sqlx::PgPool,
    storage_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let now = Utc::now();

    // Find all enabled schedules that are due
    let rows: Vec<ScheduleRow> = sqlx::query_as(
        "SELECT id, repo_id, cron, name, ref_name, yaml_path, enabled, last_run_at, next_run_at, created_at, updated_at FROM pipeline_schedules WHERE enabled = true AND next_run_at <= $1",
    )
    .bind(now)
    .fetch_all(pool)
    .await?;

    for schedule in rows {
        if let Err(e) = trigger_scheduled_run(pool, storage_path, &schedule).await {
            tracing::warn!(
                "failed to trigger scheduled run for schedule {}: {e}",
                schedule.id
            );
        }

        // Compute next run time
        let next_run = compute_next_cron_run(&schedule.cron, &now);

        // Update the schedule with new last_run_at and next_run_at
        sqlx::query(
            "UPDATE pipeline_schedules SET last_run_at = $1, next_run_at = $2, updated_at = $3 WHERE id = $4",
        )
        .bind(now)
        .bind(next_run)
        .bind(now)
        .bind(schedule.id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Trigger a single scheduled pipeline run.
async fn trigger_scheduled_run(
    pool: &sqlx::PgPool,
    storage_path: &str,
    schedule: &ScheduleRow,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repo_id = schedule.repo_id;
    let ref_name = schedule
        .ref_name
        .clone()
        .unwrap_or_else(|| "main".to_string());
    let yaml_path = &schedule.yaml_path;

    // Resolve repo owner/name from repo_id
    let (owner, repo_name) = get_repo_owner_name(pool, repo_id).await?;

    // Read pipeline YAML
    let yaml_content =
        match read_pipeline_yaml_from_fs(storage_path, &owner, &repo_name, &ref_name, yaml_path) {
            Some(content) => content,
            None => {
                tracing::debug!(
                    "no pipeline YAML found for schedule {} at {}:{}",
                    schedule.id,
                    ref_name,
                    yaml_path
                );
                return Ok(());
            }
        };

    // Parse and validate
    let pipeline = match parse_pipeline(&yaml_content) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("invalid pipeline YAML for schedule {}: {e}", schedule.id);
            return Ok(());
        }
    };

    if let Err(e) = validate_pipeline(&pipeline) {
        tracing::warn!("invalid pipeline for schedule {}: {e}", schedule.id);
        return Ok(());
    }

    let pipeline = match expand_matrix(&pipeline) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("matrix expansion failed for schedule {}: {e}", schedule.id);
            return Ok(());
        }
    };

    // Check triggers — schedule event type
    let ctx = TriggerContext::schedule();
    if !matches_trigger(&pipeline, &ctx) {
        tracing::debug!("schedule trigger not matched for schedule {}", schedule.id);
        return Ok(());
    }

    // Get HEAD commit SHA for the ref
    let commit_sha = get_head_commit_sha(storage_path, &owner, &repo_name, &ref_name)
        .unwrap_or_else(|| "0000000000000000000000000000000000000000".to_string());

    // Create pipeline run
    create_scheduled_pipeline_run(
        pool,
        repo_id,
        yaml_path,
        &ref_name,
        &commit_sha,
        &yaml_content,
        &pipeline,
    )
    .await?;

    Ok(())
}

/// Resolve repo_id to (owner, repo_name).
async fn get_repo_owner_name(
    pool: &sqlx::PgPool,
    repo_id: uuid::Uuid,
) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
    let row = sqlx::query(
        "SELECT u.username, r.name FROM repositories r JOIN users u ON r.owner_id = u.id WHERE r.id = $1",
    )
    .bind(repo_id)
    .fetch_one(pool)
    .await?;

    Ok((row.get("username"), row.get("name")))
}

/// Read pipeline YAML from filesystem (bare repo).
fn read_pipeline_yaml_from_fs(
    storage_path: &str,
    owner: &str,
    repo_name: &str,
    ref_name: &str,
    yaml_path: &str,
) -> Option<String> {
    let base = std::path::Path::new(storage_path);
    let repo_path = base.join(owner).join(format!("{repo_name}.git"));

    // Try git archive first
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

    // Fallback: read from filesystem
    let file_path = repo_path.join(yaml_path);
    std::fs::read_to_string(file_path).ok()
}

/// Get HEAD commit SHA for a ref.
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

/// Create a pipeline run for a scheduled trigger.
#[allow(clippy::too_many_arguments)]
async fn create_scheduled_pipeline_run(
    pool: &sqlx::PgPool,
    repo_id: uuid::Uuid,
    yaml_path: &str,
    ref_name: &str,
    commit_sha: &str,
    yaml_content: &str,
    pipeline: &civit_pipeline::Pipeline,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let def_id = uuid::Uuid::new_v4();
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

    let mut job_ids: Vec<(String, uuid::Uuid)> = Vec::new();
    for (idx, job) in pipeline.jobs.iter().enumerate() {
        let job_id = uuid::Uuid::new_v4();
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
            let step_id = uuid::Uuid::new_v4();
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

    let run_id = uuid::Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO pipeline_runs (id, definition_id, repo_id, trigger, ref_name, commit_sha, status, concurrency_group, created_at) VALUES ($1, $2, $3, 'schedule', $4, $5, 'pending', $6, $7)",
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
        .bind(uuid::Uuid::new_v4())
        .bind(run_id)
        .bind(job_id)
        .bind(name)
        .bind(now)
        .execute(pool)
        .await?;
    }

    tracing::info!("triggered scheduled pipeline run {run_id} for repo {repo_id}");

    Ok(())
}

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
struct ScheduleRow {
    id: uuid::Uuid,
    repo_id: uuid::Uuid,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_pipeline_yaml_from_fs_missing() {
        let result =
            read_pipeline_yaml_from_fs("/nonexistent", "owner", "repo", "main", "pipeline.yaml");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_head_commit_sha_missing() {
        let result = get_head_commit_sha("/nonexistent", "owner", "repo", "main");
        assert!(result.is_none());
    }
}
