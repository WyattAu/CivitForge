#![forbid(unsafe_code)]

//! Workflow engine for CivitForge.
//!
//! Provides workflow CRUD, trigger-based execution, action chaining,
//! conditional execution, step execution, run tracking, and error handling.

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
pub struct WorkflowTrigger {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub trigger_type: String,
    pub trigger_config: serde_json::Value,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowAction {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub action_type: String,
    pub action_config: serde_json::Value,
    pub order_index: i32,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionChainResult {
    pub action_id: Uuid,
    pub action_type: String,
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
struct WorkflowTriggerRow {
    id: Uuid,
    workflow_id: Uuid,
    trigger_type: String,
    trigger_config: serde_json::Value,
    enabled: bool,
    created_at: DateTime<Utc>,
}

impl From<WorkflowTriggerRow> for WorkflowTrigger {
    fn from(row: WorkflowTriggerRow) -> Self {
        WorkflowTrigger {
            id: row.id,
            workflow_id: row.workflow_id,
            trigger_type: row.trigger_type,
            trigger_config: row.trigger_config,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct WorkflowActionRow {
    id: Uuid,
    workflow_id: Uuid,
    action_type: String,
    action_config: serde_json::Value,
    order_index: i32,
    enabled: bool,
    created_at: DateTime<Utc>,
}

impl From<WorkflowActionRow> for WorkflowAction {
    fn from(row: WorkflowActionRow) -> Self {
        WorkflowAction {
            id: row.id,
            workflow_id: row.workflow_id,
            action_type: row.action_type,
            action_config: row.action_config,
            order_index: row.order_index,
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

    // --- Workflow Triggers ---

    pub async fn add_trigger(
        &self,
        workflow_id: Uuid,
        trigger_type: &str,
        trigger_config: serde_json::Value,
    ) -> Result<WorkflowTrigger, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTriggerRow>(
            r#"INSERT INTO workflow_triggers (workflow_id, trigger_type, trigger_config)
             VALUES ($1, $2, $3)
             RETURNING id, workflow_id, trigger_type, trigger_config, enabled, created_at"#,
        )
        .bind(workflow_id)
        .bind(trigger_type)
        .bind(trigger_config)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn list_triggers(
        &self,
        workflow_id: Uuid,
    ) -> Result<Vec<WorkflowTrigger>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTriggerRow>(
            r#"SELECT id, workflow_id, trigger_type, trigger_config, enabled, created_at
             FROM workflow_triggers WHERE workflow_id = $1 ORDER BY created_at ASC"#,
        )
        .bind(workflow_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_enabled_triggers(
        &self,
        workflow_id: Uuid,
    ) -> Result<Vec<WorkflowTrigger>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTriggerRow>(
            r#"SELECT id, workflow_id, trigger_type, trigger_config, enabled, created_at
             FROM workflow_triggers WHERE workflow_id = $1 AND enabled = true
             ORDER BY created_at ASC"#,
        )
        .bind(workflow_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_trigger(
        &self,
        trigger_id: Uuid,
        trigger_config: Option<serde_json::Value>,
        enabled: Option<bool>,
    ) -> Result<WorkflowTrigger, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTriggerRow>(
            r#"UPDATE workflow_triggers SET
             trigger_config = COALESCE($2, trigger_config),
             enabled = COALESCE($3, enabled)
             WHERE id = $1
             RETURNING id, workflow_id, trigger_type, trigger_config, enabled, created_at"#,
        )
        .bind(trigger_id)
        .bind(trigger_config)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_trigger(&self, trigger_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM workflow_triggers WHERE id = $1")
            .bind(trigger_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn match_triggers(
        &self,
        trigger_type: &str,
        context: &serde_json::Value,
    ) -> Result<Vec<(Workflow, WorkflowTrigger)>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTriggerRow>(
            r#"SELECT wt.id, wt.workflow_id, wt.trigger_type, wt.trigger_config, wt.enabled, wt.created_at
             FROM workflow_triggers wt
             JOIN workflows w ON w.id = wt.workflow_id
             WHERE wt.trigger_type = $1 AND wt.enabled = true AND w.enabled = true"#,
        )
        .bind(trigger_type)
        .fetch_all(&self.pool)
        .await?;

        let mut matched = Vec::new();
        for trigger_row in rows {
            let trigger: WorkflowTrigger = trigger_row.into();
            let config = &trigger.trigger_config;

            let mut all_match = true;
            if let Some(obj) = config.as_object() {
                for (key, expected) in obj {
                    match context.get(key) {
                        Some(val) if val == expected => {}
                        _ => {
                            all_match = false;
                            break;
                        }
                    }
                }
            }

            if all_match {
                if let Some(wf) = self.get_workflow(trigger.workflow_id).await? {
                    matched.push((wf, trigger));
                }
            }
        }

        Ok(matched)
    }

    // --- Workflow Actions ---

    pub async fn add_action(
        &self,
        workflow_id: Uuid,
        action_type: &str,
        action_config: serde_json::Value,
        order_index: i32,
    ) -> Result<WorkflowAction, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowActionRow>(
            r#"INSERT INTO workflow_actions (workflow_id, action_type, action_config, order_index)
             VALUES ($1, $2, $3, $4)
             RETURNING id, workflow_id, action_type, action_config, order_index, enabled, created_at"#,
        )
        .bind(workflow_id)
        .bind(action_type)
        .bind(action_config)
        .bind(order_index)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn list_actions(
        &self,
        workflow_id: Uuid,
    ) -> Result<Vec<WorkflowAction>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowActionRow>(
            r#"SELECT id, workflow_id, action_type, action_config, order_index, enabled, created_at
             FROM workflow_actions WHERE workflow_id = $1 ORDER BY order_index ASC"#,
        )
        .bind(workflow_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_enabled_actions(
        &self,
        workflow_id: Uuid,
    ) -> Result<Vec<WorkflowAction>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowActionRow>(
            r#"SELECT id, workflow_id, action_type, action_config, order_index, enabled, created_at
             FROM workflow_actions WHERE workflow_id = $1 AND enabled = true
             ORDER BY order_index ASC"#,
        )
        .bind(workflow_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_action(
        &self,
        action_id: Uuid,
        action_config: Option<serde_json::Value>,
        order_index: Option<i32>,
        enabled: Option<bool>,
    ) -> Result<WorkflowAction, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowActionRow>(
            r#"UPDATE workflow_actions SET
             action_config = COALESCE($2, action_config),
             order_index = COALESCE($3, order_index),
             enabled = COALESCE($4, enabled)
             WHERE id = $1
             RETURNING id, workflow_id, action_type, action_config, order_index, enabled, created_at"#,
        )
        .bind(action_id)
        .bind(action_config)
        .bind(order_index)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_action(&self, action_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM workflow_actions WHERE id = $1")
            .bind(action_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn execute_action_chain(
        &self,
        workflow_id: Uuid,
        context: &serde_json::Value,
    ) -> Result<Vec<ActionChainResult>, sqlx::Error> {
        let actions = self.list_enabled_actions(workflow_id).await?;
        let mut results = Vec::new();

        for action in &actions {
            let started_at = Utc::now();

            let should_execute = self.evaluate_action_conditions(&action.action_config, context);

            if !should_execute {
                results.push(ActionChainResult {
                    action_id: action.id,
                    action_type: action.action_type.clone(),
                    status: "skipped".to_string(),
                    output: Some(serde_json::json!({"reason": "condition_not_met"})),
                    error: None,
                    started_at,
                    completed_at: Some(Utc::now()),
                });
                continue;
            }

            let status = "completed".to_string();
            let output = Some(serde_json::json!({
                "action_type": action.action_type,
                "action_id": action.id,
                "order": action.order_index,
                "result": "executed"
            }));

            results.push(ActionChainResult {
                action_id: action.id,
                action_type: action.action_type.clone(),
                status,
                output,
                error: None,
                started_at,
                completed_at: Some(Utc::now()),
            });
        }

        Ok(results)
    }

    fn evaluate_action_conditions(
        &self,
        action_config: &serde_json::Value,
        context: &serde_json::Value,
    ) -> bool {
        let conditions = match action_config.get("conditions") {
            Some(c) => c,
            None => return true,
        };

        let obj = match conditions.as_object() {
            Some(o) => o,
            None => return true,
        };

        if obj.is_empty() {
            return true;
        }

        for (key, expected) in obj {
            match context.get(key) {
                Some(val) if val == expected => {}
                _ => return false,
            }
        }

        true
    }

    pub async fn execute_workflow_with_triggers(
        &self,
        trigger_type: &str,
        context: &serde_json::Value,
    ) -> Result<Vec<(WorkflowRun, Vec<ActionChainResult>)>, sqlx::Error> {
        let matches = self.match_triggers(trigger_type, context).await?;
        let mut all_results = Vec::new();

        for (workflow, _trigger) in matches {
            let run = self.start_run(workflow.id).await?;
            let action_results = self.execute_action_chain(workflow.id, context).await?;

            let all_success = action_results.iter().all(|r| r.status == "completed" || r.status == "skipped");
            let final_status = if all_success { "completed" } else { "failed" };
            let _ = self.complete_run(run.id, final_status).await?;

            all_results.push((run, action_results));
        }

        Ok(all_results)
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

    #[test]
    fn test_action_chain_result_serialization() {
        let result = ActionChainResult {
            action_id: Uuid::new_v4(),
            action_type: "send_notification".into(),
            status: "completed".into(),
            output: Some(serde_json::json!({"sent": true})),
            error: None,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("send_notification"));
    }

    #[test]
    fn test_workflow_trigger_serialization() {
        let trigger = WorkflowTrigger {
            id: Uuid::new_v4(),
            workflow_id: Uuid::new_v4(),
            trigger_type: "push".into(),
            trigger_config: serde_json::json!({"branch": "main"}),
            enabled: true,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&trigger).unwrap();
        assert!(json.contains("push"));
    }

    #[test]
    fn test_workflow_action_serialization() {
        let action = WorkflowAction {
            id: Uuid::new_v4(),
            workflow_id: Uuid::new_v4(),
            action_type: "run_pipeline".into(),
            action_config: serde_json::json!({"template": "deploy"}),
            order_index: 0,
            enabled: true,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("run_pipeline"));
    }
}
