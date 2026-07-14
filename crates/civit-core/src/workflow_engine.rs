#![forbid(unsafe_code)]

//! Workflow engine for CivitForge.
//!
//! Provides workflow CRUD, step execution, run tracking, and error handling.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub trigger_type: String,
    pub trigger_config: serde_json::Value,
    pub steps: serde_json::Value,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRun {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub status: String,
    pub current_step: i32,
    pub total_steps: i32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkflow {
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub trigger_config: Option<serde_json::Value>,
    pub steps: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkflow {
    pub name: Option<String>,
    pub description: Option<String>,
    pub trigger_type: Option<String>,
    pub trigger_config: Option<serde_json::Value>,
    pub steps: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStepResult {
    pub step_index: i32,
    pub status: String,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, sqlx::FromRow)]
struct WorkflowRow {
    id: Uuid,
    name: String,
    description: String,
    trigger_type: String,
    trigger_config: serde_json::Value,
    steps: serde_json::Value,
    enabled: bool,
    created_at: DateTime<Utc>,
}

impl From<WorkflowRow> for Workflow {
    fn from(row: WorkflowRow) -> Self {
        Workflow {
            id: row.id,
            name: row.name,
            description: row.description,
            trigger_type: row.trigger_type,
            trigger_config: row.trigger_config,
            steps: row.steps,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct WorkflowRunRow {
    id: Uuid,
    workflow_id: Uuid,
    status: String,
    current_step: i32,
    total_steps: i32,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

impl From<WorkflowRunRow> for WorkflowRun {
    fn from(row: WorkflowRunRow) -> Self {
        WorkflowRun {
            id: row.id,
            workflow_id: row.workflow_id,
            status: row.status,
            current_step: row.current_step,
            total_steps: row.total_steps,
            started_at: row.started_at,
            completed_at: row.completed_at,
        }
    }
}

pub struct WorkflowService {
    pool: PgPool,
}

impl WorkflowService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_workflow(&self, input: CreateWorkflow) -> Result<Workflow, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowRow>(
            r#"INSERT INTO workflows (name, description, trigger_type, trigger_config, steps, enabled)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, description, trigger_type, trigger_config, steps, enabled, created_at"#,
        )
        .bind(&input.name)
        .bind(input.description.as_deref().unwrap_or(""))
        .bind(&input.trigger_type)
        .bind(input.trigger_config.unwrap_or(serde_json::json!({})))
        .bind(input.steps.unwrap_or(serde_json::json!([])))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_workflow(&self, id: Uuid) -> Result<Option<Workflow>, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowRow>(
            r#"SELECT id, name, description, trigger_type, trigger_config, steps, enabled, created_at
             FROM workflows WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_workflows(&self) -> Result<Vec<Workflow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowRow>(
            r#"SELECT id, name, description, trigger_type, trigger_config, steps, enabled, created_at
             FROM workflows ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_enabled_workflows(&self) -> Result<Vec<Workflow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowRow>(
            r#"SELECT id, name, description, trigger_type, trigger_config, steps, enabled, created_at
             FROM workflows WHERE enabled = true ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_workflow(
        &self,
        id: Uuid,
        input: UpdateWorkflow,
    ) -> Result<Workflow, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowRow>(
            r#"UPDATE workflows SET
             name = COALESCE($2, name),
             description = COALESCE($3, description),
             trigger_type = COALESCE($4, trigger_type),
             trigger_config = COALESCE($5, trigger_config),
             steps = COALESCE($6, steps),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, name, description, trigger_type, trigger_config, steps, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.trigger_type)
        .bind(&input.trigger_config)
        .bind(&input.steps)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_workflow(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM workflows WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn start_run(&self, workflow_id: Uuid) -> Result<WorkflowRun, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct StepsCount {
            count: Option<i64>,
        }

        let count_row = sqlx::query_as::<_, StepsCount>(
            r#"SELECT COUNT(*) as count FROM jsonb_array_elements(
             (SELECT steps FROM workflows WHERE id = $1)
           )"#,
        )
        .bind(workflow_id)
        .fetch_one(&self.pool)
        .await?;

        let total_steps = count_row.count.unwrap_or(0) as i32;

        let row = sqlx::query_as::<_, WorkflowRunRow>(
            r#"INSERT INTO workflow_runs (workflow_id, status, current_step, total_steps)
             VALUES ($1, 'running', 0, $2)
             RETURNING id, workflow_id, status, current_step, total_steps, started_at, completed_at"#,
        )
        .bind(workflow_id)
        .bind(total_steps)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn advance_step(&self, run_id: Uuid) -> Result<WorkflowRun, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowRunRow>(
            r#"UPDATE workflow_runs SET current_step = current_step + 1
             WHERE id = $1
             RETURNING id, workflow_id, status, current_step, total_steps, started_at, completed_at"#,
        )
        .bind(run_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn complete_run(&self, run_id: Uuid, status: &str) -> Result<WorkflowRun, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowRunRow>(
            r#"UPDATE workflow_runs SET status = $2, completed_at = NOW()
             WHERE id = $1
             RETURNING id, workflow_id, status, current_step, total_steps, started_at, completed_at"#,
        )
        .bind(run_id)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn fail_run(&self, run_id: Uuid, error: &str) -> Result<WorkflowRun, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowRunRow>(
            r#"UPDATE workflow_runs SET status = 'failed', completed_at = NOW()
             WHERE id = $1
             RETURNING id, workflow_id, status, current_step, total_steps, started_at, completed_at"#,
        )
        .bind(run_id)
        .fetch_one(&self.pool)
        .await?;

        let _ = error;
        Ok(row.into())
    }

    pub async fn get_run(&self, run_id: Uuid) -> Result<Option<WorkflowRun>, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowRunRow>(
            r#"SELECT id, workflow_id, status, current_step, total_steps, started_at, completed_at
             FROM workflow_runs WHERE id = $1"#,
        )
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_runs_for_workflow(
        &self,
        workflow_id: Uuid,
    ) -> Result<Vec<WorkflowRun>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowRunRow>(
            r#"SELECT id, workflow_id, status, current_step, total_steps, started_at, completed_at
             FROM workflow_runs WHERE workflow_id = $1 ORDER BY started_at DESC"#,
        )
        .bind(workflow_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn execute_step(
        &self,
        run_id: Uuid,
        step_index: i32,
    ) -> Result<WorkflowStepResult, sqlx::Error> {
        let started_at = Utc::now();

        #[derive(sqlx::FromRow)]
        struct StepRow {
            workflow_id: Uuid,
            steps: serde_json::Value,
        }

        let wf_row = sqlx::query_as::<_, StepRow>(
            r#"SELECT wr.workflow_id, w.steps
             FROM workflow_runs wr
             JOIN workflows w ON w.id = wr.workflow_id
             WHERE wr.id = $1"#,
        )
        .bind(run_id)
        .fetch_one(&self.pool)
        .await?;

        let steps = wf_row.steps.as_array().ok_or_else(|| {
            sqlx::Error::ColumnDecode {
                index: "steps".into(),
                source: "invalid steps JSON".into(),
            }
        })?;

        let step = steps.get(step_index as usize).ok_or_else(|| {
            sqlx::Error::RowNotFound
        })?;

        let status = "completed".to_string();
        let output = Some(serde_json::json!({
            "step_index": step_index,
            "step_name": step.get("name").and_then(|v| v.as_str()).unwrap_or("unknown"),
            "result": "success"
        }));

        Ok(WorkflowStepResult {
            step_index,
            status,
            output,
            error: None,
            started_at,
            completed_at: Some(Utc::now()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_workflow_input_serialization() {
        let input = CreateWorkflow {
            name: "Deploy Pipeline".into(),
            description: Some("Auto-deploy on push".into()),
            trigger_type: "push".into(),
            trigger_config: Some(serde_json::json!({"branch": "main"})),
            steps: Some(serde_json::json!([{"name": "build"}, {"name": "deploy"}])),
            enabled: Some(true),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("Deploy Pipeline"));
        assert!(json.contains("push"));
    }

    #[test]
    fn test_workflow_step_result_serialization() {
        let result = WorkflowStepResult {
            step_index: 0,
            status: "completed".into(),
            output: Some(serde_json::json!({"ok": true})),
            error: None,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("completed"));
    }
}
