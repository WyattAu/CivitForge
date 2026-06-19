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

        if let Some(ref needs) = rj.needs {
            if let Ok(deps) = serde_json::from_value::<Vec<String>>(needs.clone()) {
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
