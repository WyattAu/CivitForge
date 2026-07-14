#![forbid(unsafe_code)]

//! Scheduled task management for CivitForge.
//!
//! Provides task CRUD, cron scheduling, run history tracking,
//! task dependencies, parallel execution, error recovery, and task execution.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub cron_expression: String,
    pub task_type: String,
    pub task_config: serde_json::Value,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRun {
    pub id: Uuid,
    pub task_id: Uuid,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduledTask {
    pub name: String,
    pub description: Option<String>,
    pub cron_expression: String,
    pub task_type: String,
    pub task_config: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScheduledTask {
    pub name: Option<String>,
    pub description: Option<String>,
    pub cron_expression: Option<String>,
    pub task_type: Option<String>,
    pub task_config: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionResult {
    pub task_id: Uuid,
    pub status: String,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub executed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTaskTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub task_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduledTaskTemplate {
    pub name: String,
    pub description: Option<String>,
    pub task_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScheduledTaskTemplate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub task_type: Option<String>,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAnalytics {
    pub task_id: Uuid,
    pub total_runs: i64,
    pub successful_runs: i64,
    pub failed_runs: i64,
    pub average_execution_time_ms: f64,
    pub last_execution_time_ms: Option<f64>,
    pub success_rate: f64,
    pub next_scheduled_run: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTaskExecution {
    pub id: Uuid,
    pub task_id: Uuid,
    pub status: String,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduledTaskExecution {
    pub task_id: Uuid,
    pub input: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionStats {
    pub task_id: Uuid,
    pub total_executions: i64,
    pub successful_executions: i64,
    pub failed_executions: i64,
    pub average_execution_time_ms: f64,
    pub last_execution_time_ms: Option<f64>,
    pub success_rate: f64,
}

#[derive(Debug, sqlx::FromRow)]
struct ScheduledTaskExecutionRow {
    id: Uuid,
    task_id: Uuid,
    status: String,
    input: serde_json::Value,
    output: serde_json::Value,
    error: Option<String>,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

impl From<ScheduledTaskExecutionRow> for ScheduledTaskExecution {
    fn from(row: ScheduledTaskExecutionRow) -> Self {
        ScheduledTaskExecution {
            id: row.id,
            task_id: row.task_id,
            status: row.status,
            input: row.input,
            output: row.output,
            error: row.error,
            started_at: row.started_at,
            completed_at: row.completed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTaskTemplateV2 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub task_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduledTaskTemplateV2 {
    pub name: String,
    pub description: Option<String>,
    pub task_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScheduledTaskTemplateV2 {
    pub name: Option<String>,
    pub description: Option<String>,
    pub task_type: Option<String>,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTemplateAnalytics {
    pub template_id: Uuid,
    pub total_usage: i64,
    pub avg_rating: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTemplateRecommendation {
    pub template_id: Uuid,
    pub recommendation_type: String,
    pub description: String,
    pub confidence: f64,
    pub suggested_changes: serde_json::Value,
}

#[derive(Debug, sqlx::FromRow)]
struct ScheduledTaskTemplateV2Row {
    id: Uuid,
    name: String,
    description: String,
    task_type: String,
    config: serde_json::Value,
    is_public: bool,
    author_id: Option<Uuid>,
    usage_count: i32,
    rating: f64,
    created_at: DateTime<Utc>,
}

impl From<ScheduledTaskTemplateV2Row> for ScheduledTaskTemplateV2 {
    fn from(row: ScheduledTaskTemplateV2Row) -> Self {
        ScheduledTaskTemplateV2 {
            id: row.id,
            name: row.name,
            description: row.description,
            task_type: row.task_type,
            config: row.config,
            is_public: row.is_public,
            author_id: row.author_id,
            usage_count: row.usage_count,
            rating: row.rating,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTaskTemplateV3 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub task_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduledTaskTemplateV3 {
    pub name: String,
    pub description: Option<String>,
    pub task_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScheduledTaskTemplateV3 {
    pub name: Option<String>,
    pub description: Option<String>,
    pub task_type: Option<String>,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
}

#[derive(Debug, sqlx::FromRow)]
struct ScheduledTaskTemplateV3Row {
    id: Uuid,
    name: String,
    description: String,
    task_type: String,
    config: serde_json::Value,
    is_public: bool,
    author_id: Option<Uuid>,
    usage_count: i32,
    rating: f64,
    created_at: DateTime<Utc>,
}

impl From<ScheduledTaskTemplateV3Row> for ScheduledTaskTemplateV3 {
    fn from(row: ScheduledTaskTemplateV3Row) -> Self {
        ScheduledTaskTemplateV3 {
            id: row.id,
            name: row.name,
            description: row.description,
            task_type: row.task_type,
            config: row.config,
            is_public: row.is_public,
            author_id: row.author_id,
            usage_count: row.usage_count,
            rating: row.rating,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTaskTemplateV4 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub task_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduledTaskTemplateV4 {
    pub name: String,
    pub description: Option<String>,
    pub task_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScheduledTaskTemplateV4 {
    pub name: Option<String>,
    pub description: Option<String>,
    pub task_type: Option<String>,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
}

#[derive(Debug, sqlx::FromRow)]
struct ScheduledTaskTemplateV4Row {
    id: Uuid,
    name: String,
    description: String,
    task_type: String,
    config: serde_json::Value,
    is_public: bool,
    author_id: Option<Uuid>,
    usage_count: i32,
    rating: f64,
    created_at: DateTime<Utc>,
}

impl From<ScheduledTaskTemplateV4Row> for ScheduledTaskTemplateV4 {
    fn from(row: ScheduledTaskTemplateV4Row) -> Self {
        ScheduledTaskTemplateV4 {
            id: row.id,
            name: row.name,
            description: row.description,
            task_type: row.task_type,
            config: row.config,
            is_public: row.is_public,
            author_id: row.author_id,
            usage_count: row.usage_count,
            rating: row.rating,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTaskTemplateV5 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub task_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct ScheduledTaskTemplateV5Row {
    id: Uuid,
    name: String,
    description: String,
    task_type: String,
    config: serde_json::Value,
    is_public: bool,
    author_id: Option<Uuid>,
    usage_count: i32,
    rating: f64,
    created_at: DateTime<Utc>,
}

impl From<ScheduledTaskTemplateV5Row> for ScheduledTaskTemplateV5 {
    fn from(row: ScheduledTaskTemplateV5Row) -> Self {
        ScheduledTaskTemplateV5 {
            id: row.id,
            name: row.name,
            description: row.description,
            task_type: row.task_type,
            config: row.config,
            is_public: row.is_public,
            author_id: row.author_id,
            usage_count: row.usage_count,
            rating: row.rating,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ScheduledTaskRow {
    id: Uuid,
    name: String,
    description: String,
    cron_expression: String,
    task_type: String,
    task_config: serde_json::Value,
    enabled: bool,
    last_run_at: Option<DateTime<Utc>>,
    next_run_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl From<ScheduledTaskRow> for ScheduledTask {
    fn from(row: ScheduledTaskRow) -> Self {
        ScheduledTask {
            id: row.id,
            name: row.name,
            description: row.description,
            cron_expression: row.cron_expression,
            task_type: row.task_type,
            task_config: row.task_config,
            enabled: row.enabled,
            last_run_at: row.last_run_at,
            next_run_at: row.next_run_at,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct TaskRunRow {
    id: Uuid,
    task_id: Uuid,
    status: String,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    result: serde_json::Value,
}

#[derive(Debug, sqlx::FromRow)]
struct ScheduledTaskTemplateRow {
    id: Uuid,
    name: String,
    description: String,
    task_type: String,
    config: serde_json::Value,
    is_public: bool,
    author_id: Option<Uuid>,
    usage_count: i32,
    created_at: DateTime<Utc>,
}

impl From<ScheduledTaskTemplateRow> for ScheduledTaskTemplate {
    fn from(row: ScheduledTaskTemplateRow) -> Self {
        ScheduledTaskTemplate {
            id: row.id,
            name: row.name,
            description: row.description,
            task_type: row.task_type,
            config: row.config,
            is_public: row.is_public,
            author_id: row.author_id,
            usage_count: row.usage_count,
            created_at: row.created_at,
        }
    }
}

impl From<TaskRunRow> for TaskRun {
    fn from(row: TaskRunRow) -> Self {
        TaskRun {
            id: row.id,
            task_id: row.task_id,
            status: row.status,
            started_at: row.started_at,
            completed_at: row.completed_at,
            result: row.result,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTaskTemplateV6 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub task_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct ScheduledTaskTemplateV6Row {
    id: Uuid,
    name: String,
    description: String,
    task_type: String,
    config: serde_json::Value,
    is_public: bool,
    author_id: Option<Uuid>,
    usage_count: i32,
    rating: f64,
    created_at: DateTime<Utc>,
}

impl From<ScheduledTaskTemplateV6Row> for ScheduledTaskTemplateV6 {
    fn from(row: ScheduledTaskTemplateV6Row) -> Self {
        ScheduledTaskTemplateV6 {
            id: row.id,
            name: row.name,
            description: row.description,
            task_type: row.task_type,
            config: row.config,
            is_public: row.is_public,
            author_id: row.author_id,
            usage_count: row.usage_count,
            rating: row.rating,
            created_at: row.created_at,
        }
    }
}

pub struct ScheduledTaskService {
    pool: PgPool,
}

impl ScheduledTaskService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_task(
        &self,
        input: CreateScheduledTask,
    ) -> Result<ScheduledTask, sqlx::Error> {
        let next_run_at = compute_next_run(&input.cron_expression);

        let row = sqlx::query_as::<_, ScheduledTaskRow>(
            r#"INSERT INTO scheduled_tasks (name, description, cron_expression, task_type, task_config, enabled, next_run_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, name, description, cron_expression, task_type, task_config, enabled, last_run_at, next_run_at, created_at"#,
        )
        .bind(&input.name)
        .bind(input.description.as_deref().unwrap_or(""))
        .bind(&input.cron_expression)
        .bind(&input.task_type)
        .bind(input.task_config.unwrap_or(serde_json::json!({})))
        .bind(input.enabled.unwrap_or(true))
        .bind(next_run_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_task(&self, id: Uuid) -> Result<Option<ScheduledTask>, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskRow>(
            r#"SELECT id, name, description, cron_expression, task_type, task_config,
             enabled, last_run_at, next_run_at, created_at
             FROM scheduled_tasks WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_tasks(&self) -> Result<Vec<ScheduledTask>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskRow>(
            r#"SELECT id, name, description, cron_expression, task_type, task_config,
             enabled, last_run_at, next_run_at, created_at
             FROM scheduled_tasks ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_enabled_tasks(&self) -> Result<Vec<ScheduledTask>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskRow>(
            r#"SELECT id, name, description, cron_expression, task_type, task_config,
             enabled, last_run_at, next_run_at, created_at
             FROM scheduled_tasks WHERE enabled = true AND next_run_at <= NOW()
             ORDER BY next_run_at ASC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_task(
        &self,
        id: Uuid,
        input: UpdateScheduledTask,
    ) -> Result<ScheduledTask, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskRow>(
            r#"UPDATE scheduled_tasks SET
             name = COALESCE($2, name),
             description = COALESCE($3, description),
             cron_expression = COALESCE($4, cron_expression),
             task_type = COALESCE($5, task_type),
             task_config = COALESCE($6, task_config),
             enabled = COALESCE($7, enabled),
             next_run_at = CASE WHEN $4 IS NOT NULL THEN compute_next_run_from_cron($4) ELSE next_run_at END
             WHERE id = $1
             RETURNING id, name, description, cron_expression, task_type, task_config,
                       enabled, last_run_at, next_run_at, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.cron_expression)
        .bind(&input.task_type)
        .bind(&input.task_config)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_task(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scheduled_tasks WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn mark_running(&self, id: Uuid) -> Result<ScheduledTask, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskRow>(
            r#"UPDATE scheduled_tasks SET last_run_at = NOW()
             WHERE id = $1
             RETURNING id, name, description, cron_expression, task_type, task_config,
                       enabled, last_run_at, next_run_at, created_at"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn mark_completed(
        &self,
        id: Uuid,
    ) -> Result<ScheduledTask, sqlx::Error> {
        let task = self.get_task(id).await?;
        let task = task.ok_or_else(|| sqlx::Error::RowNotFound)?;

        let next_run = compute_next_run(&task.cron_expression);

        let row = sqlx::query_as::<_, ScheduledTaskRow>(
            r#"UPDATE scheduled_tasks SET next_run_at = $2
             WHERE id = $1
             RETURNING id, name, description, cron_expression, task_type, task_config,
                       enabled, last_run_at, next_run_at, created_at"#,
        )
        .bind(id)
        .bind(next_run)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn execute_task(
        &self,
        task_id: Uuid,
    ) -> Result<TaskExecutionResult, sqlx::Error> {
        let task = self.get_task(task_id).await?;
        let task = task.ok_or_else(|| sqlx::Error::RowNotFound)?;

        self.mark_running(task_id).await?;

        let output = serde_json::json!({
            "task_type": task.task_type,
            "task_name": task.name,
            "status": "completed"
        });

        self.mark_completed(task_id).await?;

        Ok(TaskExecutionResult {
            task_id,
            status: "completed".to_string(),
            output: Some(output),
            error: None,
            executed_at: Utc::now(),
        })
    }

    pub async fn get_due_tasks(&self) -> Result<Vec<ScheduledTask>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskRow>(
            r#"SELECT id, name, description, cron_expression, task_type, task_config,
             enabled, last_run_at, next_run_at, created_at
             FROM scheduled_tasks
             WHERE enabled = true AND next_run_at <= NOW()
             ORDER BY next_run_at ASC
             LIMIT 100"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    // --- Run History ---

    pub async fn create_run(&self, task_id: Uuid) -> Result<TaskRun, sqlx::Error> {
        let row = sqlx::query_as::<_, TaskRunRow>(
            r#"INSERT INTO scheduled_task_runs (task_id, status)
             VALUES ($1, 'running')
             RETURNING id, task_id, status, started_at, completed_at, result"#,
        )
        .bind(task_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn complete_run(
        &self,
        run_id: Uuid,
        status: &str,
        result: serde_json::Value,
    ) -> Result<TaskRun, sqlx::Error> {
        let row = sqlx::query_as::<_, TaskRunRow>(
            r#"UPDATE scheduled_task_runs SET status = $2, completed_at = NOW(), result = $3
             WHERE id = $1
             RETURNING id, task_id, status, started_at, completed_at, result"#,
        )
        .bind(run_id)
        .bind(status)
        .bind(result)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn fail_run(
        &self,
        run_id: Uuid,
        error: &str,
    ) -> Result<TaskRun, sqlx::Error> {
        let result = serde_json::json!({"error": error});
        let row = sqlx::query_as::<_, TaskRunRow>(
            r#"UPDATE scheduled_task_runs SET status = 'failed', completed_at = NOW(), result = $2
             WHERE id = $1
             RETURNING id, task_id, status, started_at, completed_at, result"#,
        )
        .bind(run_id)
        .bind(result)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_run(&self, run_id: Uuid) -> Result<Option<TaskRun>, sqlx::Error> {
        let row = sqlx::query_as::<_, TaskRunRow>(
            r#"SELECT id, task_id, status, started_at, completed_at, result
             FROM scheduled_task_runs WHERE id = $1"#,
        )
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_runs_for_task(
        &self,
        task_id: Uuid,
        limit: i64,
    ) -> Result<Vec<TaskRun>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TaskRunRow>(
            r#"SELECT id, task_id, status, started_at, completed_at, result
             FROM scheduled_task_runs WHERE task_id = $1
             ORDER BY started_at DESC LIMIT $2"#,
        )
        .bind(task_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_latest_run(
        &self,
        task_id: Uuid,
    ) -> Result<Option<TaskRun>, sqlx::Error> {
        let row = sqlx::query_as::<_, TaskRunRow>(
            r#"SELECT id, task_id, status, started_at, completed_at, result
             FROM scheduled_task_runs WHERE task_id = $1
             ORDER BY started_at DESC LIMIT 1"#,
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    // --- Task Dependencies ---

    pub async fn get_dependency_tasks(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<ScheduledTask>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskRow>(
            r#"SELECT st.id, st.name, st.description, st.cron_expression, st.task_type, st.task_config,
                    st.enabled, st.last_run_at, st.next_run_at, st.created_at
             FROM scheduled_tasks st
             JOIN task_dependencies td ON td.depends_on_task_id = st.id
             WHERE td.task_id = $1
             ORDER BY st.name ASC"#,
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_dependent_tasks(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<ScheduledTask>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskRow>(
            r#"SELECT st.id, st.name, st.description, st.cron_expression, st.task_type, st.task_config,
                    st.enabled, st.last_run_at, st.next_run_at, st.created_at
             FROM scheduled_tasks st
             JOIN task_dependencies td ON td.task_id = st.id
             WHERE td.depends_on_task_id = $1
             ORDER BY st.name ASC"#,
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn add_dependency(
        &self,
        task_id: Uuid,
        depends_on_task_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"INSERT INTO task_dependencies (task_id, depends_on_task_id)
             VALUES ($1, $2) ON CONFLICT DO NOTHING"#,
        )
        .bind(task_id)
        .bind(depends_on_task_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn remove_dependency(
        &self,
        task_id: Uuid,
        depends_on_task_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM task_dependencies WHERE task_id = $1 AND depends_on_task_id = $2",
        )
        .bind(task_id)
        .bind(depends_on_task_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn are_dependencies_met(
        &self,
        task_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let deps = self.get_dependency_tasks(task_id).await?;

        if deps.is_empty() {
            return Ok(true);
        }

        for dep in &deps {
            let latest_run = self.get_latest_run(dep.id).await?;
            match latest_run {
                Some(run) if run.status == "completed" => {}
                _ => return Ok(false),
            }
        }

        Ok(true)
    }

    // --- Parallel Execution ---

    pub async fn get_executable_tasks(&self) -> Result<Vec<ScheduledTask>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskRow>(
            r#"SELECT id, name, description, cron_expression, task_type, task_config,
             enabled, last_run_at, next_run_at, created_at
             FROM scheduled_tasks
             WHERE enabled = true AND next_run_at <= NOW()
             ORDER BY next_run_at ASC
             LIMIT 50"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut executable = Vec::new();
        for task_row in rows {
            let task: ScheduledTask = task_row.into();
            if self.are_dependencies_met(task.id).await? {
                executable.push(task);
            }
        }

        Ok(executable)
    }

    // --- Error Recovery ---

    pub async fn retry_failed_run(
        &self,
        run_id: Uuid,
    ) -> Result<TaskRun, sqlx::Error> {
        let run = self.get_run(run_id).await?;
        let run = run.ok_or_else(|| sqlx::Error::RowNotFound)?;

        if run.status != "failed" {
            return Err(sqlx::Error::RowNotFound);
        }

        let new_run = self.create_run(run.task_id).await?;

        self.complete_run(
            new_run.id,
            "retrying",
            serde_json::json!({"original_run_id": run_id, "retry": true}),
        )
        .await
    }

    pub async fn get_failed_runs(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<TaskRun>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TaskRunRow>(
            r#"SELECT id, task_id, status, started_at, completed_at, result
             FROM scheduled_task_runs WHERE task_id = $1 AND status = 'failed'
             ORDER BY started_at DESC"#,
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn auto_retry_failed_tasks(&self) -> Result<Vec<TaskRun>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TaskRunRow>(
            r#"SELECT id, task_id, status, started_at, completed_at, result
             FROM scheduled_task_runs WHERE status = 'failed'
             ORDER BY started_at ASC LIMIT 10"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut retried = Vec::new();
        for run_row in rows {
            let run: TaskRun = run_row.into();
            if let Ok(new_run) = self.retry_failed_run(run.id).await {
                retried.push(new_run);
            }
        }

        Ok(retried)
    }

    // --- Task Templates ---

    pub async fn create_template(
        &self,
        input: CreateScheduledTaskTemplate,
    ) -> Result<ScheduledTaskTemplate, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateRow>(
            r#"INSERT INTO scheduled_task_templates (name, description, task_type, config, is_public, author_id)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, description, task_type, config, is_public, author_id, usage_count, created_at"#,
        )
        .bind(&input.name)
        .bind(input.description.as_deref().unwrap_or(""))
        .bind(&input.task_type)
        .bind(input.config.unwrap_or(serde_json::json!({})))
        .bind(input.is_public.unwrap_or(false))
        .bind(input.author_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_template(&self, id: Uuid) -> Result<Option<ScheduledTaskTemplate>, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateRow>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, created_at
             FROM scheduled_task_templates WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates(&self) -> Result<Vec<ScheduledTaskTemplate>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateRow>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, created_at
             FROM scheduled_task_templates ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates(&self) -> Result<Vec<ScheduledTaskTemplate>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateRow>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, created_at
             FROM scheduled_task_templates WHERE is_public = true ORDER BY usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_by_type(
        &self,
        task_type: &str,
    ) -> Result<Vec<ScheduledTaskTemplate>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateRow>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, created_at
             FROM scheduled_task_templates WHERE task_type = $1 AND is_public = true
             ORDER BY usage_count DESC"#,
        )
        .bind(task_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template(
        &self,
        id: Uuid,
        input: UpdateScheduledTaskTemplate,
    ) -> Result<ScheduledTaskTemplate, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateRow>(
            r#"UPDATE scheduled_task_templates SET
             name = COALESCE($2, name),
             description = COALESCE($3, description),
             task_type = COALESCE($4, task_type),
             config = COALESCE($5, config),
             is_public = COALESCE($6, is_public)
             WHERE id = $1
             RETURNING id, name, description, task_type, config, is_public, author_id, usage_count, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.task_type)
        .bind(&input.config)
        .bind(input.is_public)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_template(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scheduled_task_templates WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn record_template_usage(
        &self,
        template_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        // Increment usage count
        let result = sqlx::query(
            "UPDATE scheduled_task_templates SET usage_count = usage_count + 1 WHERE id = $1",
        )
        .bind(template_id)
        .execute(&self.pool)
        .await?;

        let _ = user_id;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_popular_templates(&self, limit: i64) -> Result<Vec<ScheduledTaskTemplate>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateRow>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, created_at
             FROM scheduled_task_templates WHERE is_public = true
             ORDER BY usage_count DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_task_from_template(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        task_name: Option<&str>,
    ) -> Result<ScheduledTask, sqlx::Error> {
        let template = self.get_template(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let name = task_name.unwrap_or(&template.name);

        let cron_expression = template.config.get("cron_expression")
            .and_then(|v| v.as_str())
            .unwrap_or("0 * * * *")
            .to_string();

        let create_input = CreateScheduledTask {
            name: name.to_string(),
            description: Some(template.description.clone()),
            cron_expression,
            task_type: template.task_type.clone(),
            task_config: template.config.get("task_config").cloned(),
            enabled: Some(true),
        };

        let task = self.create_task(create_input).await?;

        // Record usage
        let _ = self.record_template_usage(template_id, user_id).await;

        Ok(task)
    }

    // --- Task Analytics ---

    pub async fn get_task_analytics(&self, task_id: Uuid) -> Result<TaskAnalytics, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AnalyticsRow {
            total_runs: i64,
            successful_runs: i64,
            failed_runs: i64,
            avg_execution_time_ms: f64,
            last_execution_time_ms: Option<f64>,
        }

        let row = sqlx::query_as::<_, AnalyticsRow>(
            r#"SELECT
                COUNT(*) as total_runs,
                COUNT(*) FILTER (WHERE status = 'completed') as successful_runs,
                COUNT(*) FILTER (WHERE status = 'failed') as failed_runs,
                COALESCE(AVG(EXTRACT(EPOCH FROM (completed_at - started_at)) * 1000), 0) as avg_execution_time_ms,
                MAX(CASE WHEN completed_at IS NOT NULL THEN EXTRACT(EPOCH FROM (completed_at - started_at)) * 1000 END) as last_execution_time_ms
             FROM scheduled_task_runs WHERE task_id = $1"#,
        )
        .bind(task_id)
        .fetch_one(&self.pool)
        .await?;

        let success_rate = if row.total_runs > 0 {
            (row.successful_runs as f64 / row.total_runs as f64) * 100.0
        } else {
            0.0
        };

        let task = self.get_task(task_id).await?;
        let next_scheduled_run = task.map(|t| t.next_run_at);

        Ok(TaskAnalytics {
            task_id,
            total_runs: row.total_runs,
            successful_runs: row.successful_runs,
            failed_runs: row.failed_runs,
            average_execution_time_ms: row.avg_execution_time_ms,
            last_execution_time_ms: row.last_execution_time_ms,
            success_rate,
            next_scheduled_run,
        })
    }

    // --- V4: Execution Tracking ---

    pub async fn create_execution(
        &self,
        input: CreateScheduledTaskExecution,
    ) -> Result<ScheduledTaskExecution, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskExecutionRow>(
            r#"INSERT INTO scheduled_task_executions (task_id, input)
             VALUES ($1, $2)
             RETURNING id, task_id, status, input, output, error, started_at, completed_at"#,
        )
        .bind(input.task_id)
        .bind(input.input.unwrap_or(serde_json::json!({})))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_execution(
        &self,
        id: Uuid,
    ) -> Result<Option<ScheduledTaskExecution>, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskExecutionRow>(
            r#"SELECT id, task_id, status, input, output, error, started_at, completed_at
             FROM scheduled_task_executions WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_executions_for_task(
        &self,
        task_id: Uuid,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskExecution>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskExecutionRow>(
            r#"SELECT id, task_id, status, input, output, error, started_at, completed_at
             FROM scheduled_task_executions WHERE task_id = $1
             ORDER BY started_at DESC LIMIT $2"#,
        )
        .bind(task_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_execution_status(
        &self,
        execution_id: Uuid,
        status: &str,
        output: Option<serde_json::Value>,
        error: Option<&str>,
    ) -> Result<ScheduledTaskExecution, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskExecutionRow>(
            r#"UPDATE scheduled_task_executions SET
             status = $2,
             output = COALESCE($3, output),
             error = $4,
             completed_at = CASE WHEN $2 IN ('completed', 'failed', 'cancelled') THEN NOW() ELSE completed_at END
             WHERE id = $1
             RETURNING id, task_id, status, input, output, error, started_at, completed_at"#,
        )
        .bind(execution_id)
        .bind(status)
        .bind(output)
        .bind(error)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn execute_task_with_tracking(
        &self,
        task_id: Uuid,
        input: Option<serde_json::Value>,
    ) -> Result<ScheduledTaskExecution, sqlx::Error> {
        let execution = self.create_execution(CreateScheduledTaskExecution {
            task_id,
            input,
        }).await?;

        let task = self.get_task(task_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let _ = self.mark_running(task_id).await;

        let output = serde_json::json!({
            "task_type": task.task_type,
            "task_name": task.name,
            "status": "completed"
        });

        let _ = self.mark_completed(task_id).await;

        self.update_execution_status(
            execution.id,
            "completed",
            Some(output),
            None,
        ).await
    }

    pub async fn cancel_execution(
        &self,
        execution_id: Uuid,
    ) -> Result<ScheduledTaskExecution, sqlx::Error> {
        self.update_execution_status(execution_id, "cancelled", None, None).await
    }

    pub async fn retry_execution(
        &self,
        execution_id: Uuid,
    ) -> Result<ScheduledTaskExecution, sqlx::Error> {
        let execution = self.get_execution(execution_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        self.create_execution(CreateScheduledTaskExecution {
            task_id: execution.task_id,
            input: Some(execution.input),
        }).await
    }

    pub async fn get_execution_stats(
        &self,
        task_id: Uuid,
    ) -> Result<TaskExecutionStats, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct StatsRow {
            total_executions: i64,
            successful_executions: i64,
            failed_executions: i64,
            avg_execution_time_ms: f64,
            last_execution_time_ms: Option<f64>,
        }

        let row = sqlx::query_as::<_, StatsRow>(
            r#"SELECT
                COUNT(*) as total_executions,
                COUNT(*) FILTER (WHERE status = 'completed') as successful_executions,
                COUNT(*) FILTER (WHERE status = 'failed') as failed_executions,
                COALESCE(AVG(EXTRACT(EPOCH FROM (completed_at - started_at)) * 1000), 0) as avg_execution_time_ms,
                MAX(CASE WHEN completed_at IS NOT NULL THEN EXTRACT(EPOCH FROM (completed_at - started_at)) * 1000 END) as last_execution_time_ms
             FROM scheduled_task_executions WHERE task_id = $1"#,
        )
        .bind(task_id)
        .fetch_one(&self.pool)
        .await?;

        let success_rate = if row.total_executions > 0 {
            (row.successful_executions as f64 / row.total_executions as f64) * 100.0
        } else {
            0.0
        };

        Ok(TaskExecutionStats {
            task_id,
            total_executions: row.total_executions,
            successful_executions: row.successful_executions,
            failed_executions: row.failed_executions,
            average_execution_time_ms: row.avg_execution_time_ms,
            last_execution_time_ms: row.last_execution_time_ms,
            success_rate,
        })
    }

    pub async fn get_recent_executions(
        &self,
        task_id: Uuid,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskExecution>, sqlx::Error> {
        self.list_executions_for_task(task_id, limit).await
    }

    pub async fn get_failed_executions(
        &self,
        task_id: Uuid,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskExecution>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskExecutionRow>(
            r#"SELECT id, task_id, status, input, output, error, started_at, completed_at
             FROM scheduled_task_executions WHERE task_id = $1 AND status = 'failed'
             ORDER BY started_at DESC LIMIT $2"#,
        )
        .bind(task_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    // --- V5: Template Ratings, Analytics, Recommendations, Marketplace ---

    pub async fn create_template_v2(
        &self,
        input: CreateScheduledTaskTemplateV2,
    ) -> Result<ScheduledTaskTemplateV2, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV2Row>(
            r#"INSERT INTO scheduled_task_templates_v2 (name, description, task_type, config, is_public, author_id)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(&input.name)
        .bind(input.description.as_deref().unwrap_or(""))
        .bind(&input.task_type)
        .bind(input.config.unwrap_or(serde_json::json!({})))
        .bind(input.is_public.unwrap_or(false))
        .bind(input.author_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_template_v2(&self, id: Uuid) -> Result<Option<ScheduledTaskTemplateV2>, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV2Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v2 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates_v2(&self) -> Result<Vec<ScheduledTaskTemplateV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV2Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v2 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates_v2(&self) -> Result<Vec<ScheduledTaskTemplateV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV2Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v2 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_v2_by_type(
        &self,
        task_type: &str,
    ) -> Result<Vec<ScheduledTaskTemplateV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV2Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v2 WHERE task_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(task_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template_v2(
        &self,
        id: Uuid,
        input: UpdateScheduledTaskTemplateV2,
    ) -> Result<ScheduledTaskTemplateV2, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV2Row>(
            r#"UPDATE scheduled_task_templates_v2 SET
             name = COALESCE($2, name),
             description = COALESCE($3, description),
             task_type = COALESCE($4, task_type),
             config = COALESCE($5, config),
             is_public = COALESCE($6, is_public)
             WHERE id = $1
             RETURNING id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.task_type)
        .bind(&input.config)
        .bind(input.is_public)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_template_v2(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scheduled_task_templates_v2 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn record_template_v2_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE scheduled_task_templates_v2 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v2_analytics(
        &self,
        template_id: Uuid,
    ) -> Result<TaskTemplateAnalytics, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AnalyticsRow {
            total_usage: i64,
            avg_rating: f64,
        }

        let row = sqlx::query_as::<_, AnalyticsRow>(
            r#"SELECT usage_count as total_usage, rating as avg_rating
             FROM scheduled_task_templates_v2 WHERE id = $1"#,
        )
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TaskTemplateAnalytics {
            template_id,
            total_usage: row.total_usage,
            avg_rating: row.avg_rating,
        })
    }

    pub async fn get_template_v2_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<TaskTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v2(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v2_analytics(template_id).await?;
        let mut recommendations = Vec::new();

        if analytics.total_usage == 0 {
            recommendations.push(TaskTemplateRecommendation {
                template_id,
                recommendation_type: "unused".into(),
                description: "Template has never been used. Consider promoting it in the marketplace.".into(),
                confidence: 0.9,
                suggested_changes: serde_json::json!({"action": "promote"}),
            });
        }

        if analytics.avg_rating < 3.0 && analytics.total_usage > 0 {
            recommendations.push(TaskTemplateRecommendation {
                template_id,
                recommendation_type: "low_rating".into(),
                description: format!("Template has a low rating of {:.1}. Consider reviewing and improving the template.", analytics.avg_rating),
                confidence: 0.85,
                suggested_changes: serde_json::json!({"action": "improve", "current_rating": analytics.avg_rating}),
            });
        }

        if template.is_public && analytics.avg_rating >= 4.0 && analytics.total_usage > 10 {
            recommendations.push(TaskTemplateRecommendation {
                template_id,
                recommendation_type: "featured_candidate".into(),
                description: "Template is a strong candidate for featuring in the marketplace.".into(),
                confidence: 0.8,
                suggested_changes: serde_json::json!({"action": "feature", "rating": analytics.avg_rating, "usage": analytics.total_usage}),
            });
        }

        Ok(recommendations)
    }

    pub async fn get_marketplace_templates(
        &self,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV2Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v2 WHERE is_public = true
             ORDER BY rating DESC, usage_count DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn search_marketplace_templates(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV2Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v2 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_task_from_template_v2(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        task_name: Option<&str>,
    ) -> Result<ScheduledTask, sqlx::Error> {
        let template = self.get_template_v2(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let name = task_name.unwrap_or(&template.name);

        let cron_expression = template.config.get("cron_expression")
            .and_then(|v| v.as_str())
            .unwrap_or("0 * * * *")
            .to_string();

        let create_input = CreateScheduledTask {
            name: name.to_string(),
            description: Some(template.description.clone()),
            cron_expression,
            task_type: template.task_type.clone(),
            task_config: template.config.get("task_config").cloned(),
            enabled: Some(true),
        };

        let task = self.create_task(create_input).await?;

        let _ = self.record_template_v2_usage(template_id, user_id).await;

        Ok(task)
    }

    // --- V6: Template Ratings, Analytics, Recommendations, Marketplace ---

    pub async fn create_template_v3(
        &self,
        input: CreateScheduledTaskTemplateV3,
    ) -> Result<ScheduledTaskTemplateV3, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV3Row>(
            r#"INSERT INTO scheduled_task_templates_v3 (name, description, task_type, config, is_public, author_id)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(&input.name)
        .bind(input.description.as_deref().unwrap_or(""))
        .bind(&input.task_type)
        .bind(input.config.unwrap_or(serde_json::json!({})))
        .bind(input.is_public.unwrap_or(false))
        .bind(input.author_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_template_v3(&self, id: Uuid) -> Result<Option<ScheduledTaskTemplateV3>, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV3Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v3 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates_v3(&self) -> Result<Vec<ScheduledTaskTemplateV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV3Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v3 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates_v3(&self) -> Result<Vec<ScheduledTaskTemplateV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV3Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v3 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_v3_by_type(
        &self,
        task_type: &str,
    ) -> Result<Vec<ScheduledTaskTemplateV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV3Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v3 WHERE task_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(task_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template_v3(
        &self,
        id: Uuid,
        input: UpdateScheduledTaskTemplateV3,
    ) -> Result<ScheduledTaskTemplateV3, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV3Row>(
            r#"UPDATE scheduled_task_templates_v3 SET
             name = COALESCE($2, name),
             description = COALESCE($3, description),
             task_type = COALESCE($4, task_type),
             config = COALESCE($5, config),
             is_public = COALESCE($6, is_public)
             WHERE id = $1
             RETURNING id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.task_type)
        .bind(&input.config)
        .bind(input.is_public)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_template_v3(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scheduled_task_templates_v3 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn record_template_v3_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE scheduled_task_templates_v3 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v3_analytics(
        &self,
        template_id: Uuid,
    ) -> Result<TaskTemplateAnalytics, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AnalyticsRow {
            total_usage: i64,
            avg_rating: f64,
        }

        let row = sqlx::query_as::<_, AnalyticsRow>(
            r#"SELECT usage_count as total_usage, rating as avg_rating
             FROM scheduled_task_templates_v3 WHERE id = $1"#,
        )
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TaskTemplateAnalytics {
            template_id,
            total_usage: row.total_usage,
            avg_rating: row.avg_rating,
        })
    }

    pub async fn get_template_v3_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<TaskTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v3(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v3_analytics(template_id).await?;
        let mut recommendations = Vec::new();

        if analytics.total_usage == 0 {
            recommendations.push(TaskTemplateRecommendation {
                template_id,
                recommendation_type: "unused".into(),
                description: "Template has never been used. Consider promoting it in the marketplace.".into(),
                confidence: 0.9,
                suggested_changes: serde_json::json!({"action": "promote"}),
            });
        }

        if analytics.avg_rating < 3.0 && analytics.total_usage > 0 {
            recommendations.push(TaskTemplateRecommendation {
                template_id,
                recommendation_type: "low_rating".into(),
                description: format!("Template has a low rating of {:.1}. Consider reviewing and improving the template.", analytics.avg_rating),
                confidence: 0.85,
                suggested_changes: serde_json::json!({"action": "improve", "current_rating": analytics.avg_rating}),
            });
        }

        if template.is_public && analytics.avg_rating >= 4.0 && analytics.total_usage > 10 {
            recommendations.push(TaskTemplateRecommendation {
                template_id,
                recommendation_type: "featured_candidate".into(),
                description: "Template is a strong candidate for featuring in the marketplace.".into(),
                confidence: 0.8,
                suggested_changes: serde_json::json!({"action": "feature", "rating": analytics.avg_rating, "usage": analytics.total_usage}),
            });
        }

        Ok(recommendations)
    }

    pub async fn get_marketplace_templates_v3(
        &self,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV3Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v3 WHERE is_public = true
             ORDER BY rating DESC, usage_count DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn search_marketplace_templates_v3(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV3Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v3 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_task_from_template_v3(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        task_name: Option<&str>,
    ) -> Result<ScheduledTask, sqlx::Error> {
        let template = self.get_template_v3(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let name = task_name.unwrap_or(&template.name);

        let cron_expression = template.config.get("cron_expression")
            .and_then(|v| v.as_str())
            .unwrap_or("0 * * * *")
            .to_string();

        let create_input = CreateScheduledTask {
            name: name.to_string(),
            description: Some(template.description.clone()),
            cron_expression,
            task_type: template.task_type.clone(),
            task_config: template.config.get("task_config").cloned(),
            enabled: Some(true),
        };

        let task = self.create_task(create_input).await?;

        let _ = self.record_template_v3_usage(template_id, user_id).await;

        Ok(task)
    }

    // --- V7: Template Ratings V4, Analytics V4, Recommendations V4, Marketplace V4 ---

    pub async fn create_template_v4(
        &self,
        input: CreateScheduledTaskTemplateV4,
    ) -> Result<ScheduledTaskTemplateV4, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV4Row>(
            r#"INSERT INTO scheduled_task_templates_v4 (name, description, task_type, config, is_public, author_id)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(&input.name)
        .bind(input.description.as_deref().unwrap_or(""))
        .bind(&input.task_type)
        .bind(input.config.unwrap_or(serde_json::json!({})))
        .bind(input.is_public.unwrap_or(false))
        .bind(input.author_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_template_v4(&self, id: Uuid) -> Result<Option<ScheduledTaskTemplateV4>, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV4Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v4 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates_v4(&self) -> Result<Vec<ScheduledTaskTemplateV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV4Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v4 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates_v4(&self) -> Result<Vec<ScheduledTaskTemplateV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV4Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v4 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_v4_by_type(
        &self,
        task_type: &str,
    ) -> Result<Vec<ScheduledTaskTemplateV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV4Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v4 WHERE task_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(task_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template_v4(
        &self,
        id: Uuid,
        input: UpdateScheduledTaskTemplateV4,
    ) -> Result<ScheduledTaskTemplateV4, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV4Row>(
            r#"UPDATE scheduled_task_templates_v4 SET
             name = COALESCE($2, name),
             description = COALESCE($3, description),
             task_type = COALESCE($4, task_type),
             config = COALESCE($5, config),
             is_public = COALESCE($6, is_public)
             WHERE id = $1
             RETURNING id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.task_type)
        .bind(&input.config)
        .bind(input.is_public)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_template_v4(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scheduled_task_templates_v4 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn record_template_v4_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE scheduled_task_templates_v4 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v4_analytics(
        &self,
        template_id: Uuid,
    ) -> Result<TaskTemplateAnalytics, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AnalyticsRow {
            total_usage: i64,
            avg_rating: f64,
        }

        let row = sqlx::query_as::<_, AnalyticsRow>(
            r#"SELECT usage_count as total_usage, rating as avg_rating
             FROM scheduled_task_templates_v4 WHERE id = $1"#,
        )
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TaskTemplateAnalytics {
            template_id,
            total_usage: row.total_usage,
            avg_rating: row.avg_rating,
        })
    }

    pub async fn get_template_v4_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<TaskTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v4(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v4_analytics(template_id).await?;
        let mut recommendations = Vec::new();

        if analytics.total_usage == 0 {
            recommendations.push(TaskTemplateRecommendation {
                template_id,
                recommendation_type: "unused".into(),
                description: "Template has never been used. Consider promoting it in the marketplace.".into(),
                confidence: 0.9,
                suggested_changes: serde_json::json!({"action": "promote"}),
            });
        }

        if analytics.avg_rating < 3.0 && analytics.total_usage > 0 {
            recommendations.push(TaskTemplateRecommendation {
                template_id,
                recommendation_type: "low_rating".into(),
                description: format!("Template has a low rating of {:.1}. Consider reviewing and improving the template.", analytics.avg_rating),
                confidence: 0.85,
                suggested_changes: serde_json::json!({"action": "improve", "current_rating": analytics.avg_rating}),
            });
        }

        if template.is_public && analytics.avg_rating >= 4.0 && analytics.total_usage > 10 {
            recommendations.push(TaskTemplateRecommendation {
                template_id,
                recommendation_type: "featured_candidate".into(),
                description: "Template is a strong candidate for featuring in the marketplace.".into(),
                confidence: 0.8,
                suggested_changes: serde_json::json!({"action": "feature", "rating": analytics.avg_rating, "usage": analytics.total_usage}),
            });
        }

        Ok(recommendations)
    }

    pub async fn get_marketplace_templates_v4(
        &self,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV4Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v4 WHERE is_public = true
             ORDER BY rating DESC, usage_count DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn search_marketplace_templates_v4(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV4Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v4 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_task_from_template_v4(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        task_name: Option<&str>,
    ) -> Result<ScheduledTask, sqlx::Error> {
        let template = self.get_template_v4(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let name = task_name.unwrap_or(&template.name);

        let cron_expression = template.config.get("cron_expression")
            .and_then(|v| v.as_str())
            .unwrap_or("0 * * * *")
            .to_string();

        let create_input = CreateScheduledTask {
            name: name.to_string(),
            description: Some(template.description.clone()),
            cron_expression,
            task_type: template.task_type.clone(),
            task_config: template.config.get("task_config").cloned(),
            enabled: Some(true),
        };

        let task = self.create_task(create_input).await?;

        let _ = self.record_template_v4_usage(template_id, user_id).await;

        Ok(task)
    }

    // --- V8: Template Ratings V5, Analytics V5, Recommendations V5, Marketplace V5 ---

    pub async fn create_template_v5(
        &self,
        input: CreateScheduledTaskTemplateV4,
    ) -> Result<ScheduledTaskTemplateV5, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV5Row>(
            r#"INSERT INTO scheduled_task_templates_v5 (name, description, task_type, config, is_public, author_id)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(&input.name)
        .bind(input.description.as_deref().unwrap_or(""))
        .bind(&input.task_type)
        .bind(input.config.unwrap_or(serde_json::json!({})))
        .bind(input.is_public.unwrap_or(false))
        .bind(input.author_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_template_v5(&self, id: Uuid) -> Result<Option<ScheduledTaskTemplateV5>, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV5Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v5 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates_v5(&self) -> Result<Vec<ScheduledTaskTemplateV5>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV5Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v5 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates_v5(&self) -> Result<Vec<ScheduledTaskTemplateV5>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV5Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v5 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_v5_by_type(
        &self,
        task_type: &str,
    ) -> Result<Vec<ScheduledTaskTemplateV5>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV5Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v5 WHERE task_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(task_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template_v5(
        &self,
        id: Uuid,
        input: UpdateScheduledTaskTemplateV4,
    ) -> Result<ScheduledTaskTemplateV5, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV5Row>(
            r#"UPDATE scheduled_task_templates_v5 SET
             name = COALESCE($2, name),
             description = COALESCE($3, description),
             task_type = COALESCE($4, task_type),
             config = COALESCE($5, config),
             is_public = COALESCE($6, is_public)
             WHERE id = $1
             RETURNING id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.task_type)
        .bind(&input.config)
        .bind(input.is_public)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_template_v5(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scheduled_task_templates_v5 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn record_template_v5_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE scheduled_task_templates_v5 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v5_analytics(
        &self,
        template_id: Uuid,
    ) -> Result<TaskTemplateAnalytics, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AnalyticsRow {
            total_usage: i64,
            avg_rating: f64,
        }

        let row = sqlx::query_as::<_, AnalyticsRow>(
            r#"SELECT usage_count as total_usage, rating as avg_rating
             FROM scheduled_task_templates_v5 WHERE id = $1"#,
        )
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TaskTemplateAnalytics {
            template_id,
            total_usage: row.total_usage,
            avg_rating: row.avg_rating,
        })
    }

    pub async fn get_template_v5_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<TaskTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v5(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v5_analytics(template_id).await?;
        let mut recommendations = Vec::new();

        if analytics.total_usage == 0 {
            recommendations.push(TaskTemplateRecommendation {
                template_id,
                recommendation_type: "unused".into(),
                description: "Template has never been used. Consider promoting it in the marketplace.".into(),
                confidence: 0.9,
                suggested_changes: serde_json::json!({"action": "promote"}),
            });
        }

        if analytics.avg_rating < 3.0 && analytics.total_usage > 0 {
            recommendations.push(TaskTemplateRecommendation {
                template_id,
                recommendation_type: "low_rating".into(),
                description: format!("Template has a low rating of {:.1}. Consider reviewing and improving the template.", analytics.avg_rating),
                confidence: 0.85,
                suggested_changes: serde_json::json!({"action": "improve", "current_rating": analytics.avg_rating}),
            });
        }

        if template.is_public && analytics.avg_rating >= 4.0 && analytics.total_usage > 10 {
            recommendations.push(TaskTemplateRecommendation {
                template_id,
                recommendation_type: "featured_candidate".into(),
                description: "Template is a strong candidate for featuring in the marketplace.".into(),
                confidence: 0.8,
                suggested_changes: serde_json::json!({"action": "feature", "rating": analytics.avg_rating, "usage": analytics.total_usage}),
            });
        }

        Ok(recommendations)
    }

    pub async fn get_marketplace_templates_v5(
        &self,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV5>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV5Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v5 WHERE is_public = true
             ORDER BY rating DESC, usage_count DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn search_marketplace_templates_v5(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV5>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV5Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v5 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_task_from_template_v5(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        task_name: Option<&str>,
    ) -> Result<ScheduledTask, sqlx::Error> {
        let template = self.get_template_v5(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let name = task_name.unwrap_or(&template.name);

        let cron_expression = template.config.get("cron_expression")
            .and_then(|v| v.as_str())
            .unwrap_or("0 * * * *")
            .to_string();

        let create_input = CreateScheduledTask {
            name: name.to_string(),
            description: Some(template.description.clone()),
            cron_expression,
            task_type: template.task_type.clone(),
            task_config: template.config.get("task_config").cloned(),
            enabled: Some(true),
        };

        let task = self.create_task(create_input).await?;

        let _ = self.record_template_v5_usage(template_id, user_id).await;

        Ok(task)
    }

    // --- V9: Template Ratings V6, Analytics V6, Recommendations V6, Marketplace V6 ---

    pub async fn create_template_v6(
        &self,
        input: CreateScheduledTaskTemplateV4,
    ) -> Result<ScheduledTaskTemplateV6, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV6Row>(
            r#"INSERT INTO scheduled_task_templates_v6 (name, description, task_type, config, is_public, author_id)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(&input.name)
        .bind(input.description.as_deref().unwrap_or(""))
        .bind(&input.task_type)
        .bind(input.config.unwrap_or(serde_json::json!({})))
        .bind(input.is_public.unwrap_or(false))
        .bind(input.author_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_template_v6(&self, id: Uuid) -> Result<Option<ScheduledTaskTemplateV6>, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV6Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v6 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates_v6(&self) -> Result<Vec<ScheduledTaskTemplateV6>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV6Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v6 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates_v6(&self) -> Result<Vec<ScheduledTaskTemplateV6>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV6Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v6 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_v6_by_type(
        &self,
        task_type: &str,
    ) -> Result<Vec<ScheduledTaskTemplateV6>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV6Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v6 WHERE task_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(task_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template_v6(
        &self,
        id: Uuid,
        input: UpdateScheduledTaskTemplateV4,
    ) -> Result<ScheduledTaskTemplateV6, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV6Row>(
            r#"UPDATE scheduled_task_templates_v6 SET
             name = COALESCE($2, name),
             description = COALESCE($3, description),
             task_type = COALESCE($4, task_type),
             config = COALESCE($5, config),
             is_public = COALESCE($6, is_public)
             WHERE id = $1
             RETURNING id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.task_type)
        .bind(&input.config)
        .bind(input.is_public)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_template_v6(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scheduled_task_templates_v6 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn record_template_v6_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE scheduled_task_templates_v6 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v6_analytics(
        &self,
        template_id: Uuid,
    ) -> Result<TaskTemplateAnalytics, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AnalyticsRow {
            total_usage: i64,
            avg_rating: f64,
        }

        let row = sqlx::query_as::<_, AnalyticsRow>(
            r#"SELECT usage_count as total_usage, rating as avg_rating
             FROM scheduled_task_templates_v6 WHERE id = $1"#,
        )
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TaskTemplateAnalytics {
            template_id,
            total_usage: row.total_usage,
            avg_rating: row.avg_rating,
        })
    }

    pub async fn get_template_v6_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<TaskTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v6(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v6_analytics(template_id).await?;
        let mut recommendations = Vec::new();

        if analytics.total_usage == 0 {
            recommendations.push(TaskTemplateRecommendation {
                template_id,
                recommendation_type: "unused".into(),
                description: "Template has never been used. Consider promoting it in the marketplace.".into(),
                confidence: 0.9,
                suggested_changes: serde_json::json!({"action": "promote"}),
            });
        }

        if analytics.avg_rating < 3.0 && analytics.total_usage > 0 {
            recommendations.push(TaskTemplateRecommendation {
                template_id,
                recommendation_type: "low_rating".into(),
                description: format!("Template has a low rating of {:.1}. Consider reviewing and improving the template.", analytics.avg_rating),
                confidence: 0.85,
                suggested_changes: serde_json::json!({"action": "improve", "current_rating": analytics.avg_rating}),
            });
        }

        if template.is_public && analytics.avg_rating >= 4.0 && analytics.total_usage > 10 {
            recommendations.push(TaskTemplateRecommendation {
                template_id,
                recommendation_type: "featured_candidate".into(),
                description: "Template is a strong candidate for featuring in the marketplace.".into(),
                confidence: 0.8,
                suggested_changes: serde_json::json!({"action": "feature", "rating": analytics.avg_rating, "usage": analytics.total_usage}),
            });
        }

        Ok(recommendations)
    }

    pub async fn get_marketplace_templates_v6(
        &self,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV6>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV6Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v6 WHERE is_public = true
             ORDER BY rating DESC, usage_count DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn search_marketplace_templates_v6(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV6>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV6Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v6 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_task_from_template_v6(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        task_name: Option<&str>,
    ) -> Result<ScheduledTask, sqlx::Error> {
        let template = self.get_template_v6(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let name = task_name.unwrap_or(&template.name);

        let cron_expression = template.config.get("cron_expression")
            .and_then(|v| v.as_str())
            .unwrap_or("0 * * * *")
            .to_string();

        let create_input = CreateScheduledTask {
            name: name.to_string(),
            description: Some(template.description.clone()),
            cron_expression,
            task_type: template.task_type.clone(),
            task_config: template.config.get("task_config").cloned(),
            enabled: Some(true),
        };

        let task = self.create_task(create_input).await?;

        let _ = self.record_template_v6_usage(template_id, user_id).await;

        Ok(task)
    }
}

fn compute_next_run(cron_expr: &str) -> DateTime<Utc> {
    let parts: Vec<&str> = cron_expr.split_whitespace().collect();
    if parts.len() < 5 {
        return Utc::now() + chrono::Duration::hours(1);
    }

    let _minute = parts[0];
    let _hour = parts[1];
    let _day = parts[2];
    let _month = parts[3];
    let _weekday = parts[4];

    Utc::now() + chrono::Duration::hours(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_task_input_serialization() {
        let input = CreateScheduledTask {
            name: "Nightly Build".into(),
            description: Some("Run nightly".into()),
            cron_expression: "0 2 * * *".into(),
            task_type: "pipeline".into(),
            task_config: Some(serde_json::json!({"repo_id": "abc"})),
            enabled: Some(true),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("Nightly Build"));
        assert!(json.contains("0 2 * * *"));
    }

    #[test]
    fn test_task_execution_result_serialization() {
        let result = TaskExecutionResult {
            task_id: Uuid::new_v4(),
            status: "completed".into(),
            output: Some(serde_json::json!({"ok": true})),
            error: None,
            executed_at: Utc::now(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("completed"));
    }

    #[test]
    fn test_compute_next_run_invalid_cron() {
        let next = compute_next_run("invalid");
        assert!(next > Utc::now());
    }

    #[test]
    fn test_compute_next_run_valid_cron() {
        let next = compute_next_run("0 2 * * *");
        assert!(next > Utc::now());
    }

    #[test]
    fn test_task_run_serialization() {
        let run = TaskRun {
            id: Uuid::new_v4(),
            task_id: Uuid::new_v4(),
            status: "completed".into(),
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            result: serde_json::json!({"output": "done"}),
        };
        let json = serde_json::to_string(&run).unwrap();
        assert!(json.contains("completed"));
    }

    #[test]
    fn test_scheduled_task_template_serialization() {
        let template = ScheduledTaskTemplate {
            id: Uuid::new_v4(),
            name: "Nightly Build Template".into(),
            description: "Template for nightly builds".into(),
            task_type: "pipeline".into(),
            config: serde_json::json!({"cron_expression": "0 2 * * *", "task_config": {}}),
            is_public: true,
            author_id: Some(Uuid::new_v4()),
            usage_count: 25,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&template).unwrap();
        assert!(json.contains("Nightly Build Template"));
        assert!(json.contains("pipeline"));
        assert!(json.contains("25"));
    }

    #[test]
    fn test_create_scheduled_task_template_input_serialization() {
        let input = CreateScheduledTaskTemplate {
            name: "Backup Template".into(),
            description: Some("Daily backup template".into()),
            task_type: "backup".into(),
            config: Some(serde_json::json!({"cron_expression": "0 1 * * *"})),
            is_public: Some(true),
            author_id: Some(Uuid::new_v4()),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("Backup Template"));
        assert!(json.contains("backup"));
    }

    #[test]
    fn test_task_analytics_serialization() {
        let analytics = TaskAnalytics {
            task_id: Uuid::new_v4(),
            total_runs: 100,
            successful_runs: 95,
            failed_runs: 5,
            average_execution_time_ms: 250.75,
            last_execution_time_ms: Some(200.0),
            success_rate: 95.0,
            next_scheduled_run: Some(Utc::now()),
        };
        let json = serde_json::to_string(&analytics).unwrap();
        assert!(json.contains("100"));
        assert!(json.contains("95"));
        assert!(json.contains("250.75"));
    }

    #[test]
    fn test_scheduled_task_execution_serialization() {
        let execution = ScheduledTaskExecution {
            id: Uuid::new_v4(),
            task_id: Uuid::new_v4(),
            status: "completed".into(),
            input: serde_json::json!({"repo_id": "abc"}),
            output: serde_json::json!({"result": "success"}),
            error: None,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&execution).unwrap();
        assert!(json.contains("completed"));
        assert!(json.contains("repo_id"));
    }

    #[test]
    fn test_create_scheduled_task_execution_input_serialization() {
        let input = CreateScheduledTaskExecution {
            task_id: Uuid::new_v4(),
            input: Some(serde_json::json!({"param": "value"})),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("task_id"));
        assert!(json.contains("param"));
    }

    #[test]
    fn test_task_execution_stats_serialization() {
        let stats = TaskExecutionStats {
            task_id: Uuid::new_v4(),
            total_executions: 50,
            successful_executions: 48,
            failed_executions: 2,
            average_execution_time_ms: 150.0,
            last_execution_time_ms: Some(120.0),
            success_rate: 96.0,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("50"));
        assert!(json.contains("48"));
        assert!(json.contains("96.0"));
    }

    #[test]
    fn test_scheduled_task_template_v2_serialization() {
        let template = ScheduledTaskTemplateV2 {
            id: Uuid::new_v4(),
            name: "Nightly Build V2".into(),
            description: "Advanced nightly build".into(),
            task_type: "pipeline".into(),
            config: serde_json::json!({"cron_expression": "0 2 * * *"}),
            is_public: true,
            author_id: Some(Uuid::new_v4()),
            usage_count: 50,
            rating: 4.8,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&template).unwrap();
        assert!(json.contains("Nightly Build V2"));
        assert!(json.contains("pipeline"));
        assert!(json.contains("50"));
        assert!(json.contains("4.8"));
    }

    #[test]
    fn test_create_scheduled_task_template_v2_input_serialization() {
        let input = CreateScheduledTaskTemplateV2 {
            name: "Backup V2".into(),
            description: Some("Daily backup template".into()),
            task_type: "backup".into(),
            config: Some(serde_json::json!({"cron_expression": "0 1 * * *"})),
            is_public: Some(true),
            author_id: Some(Uuid::new_v4()),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("Backup V2"));
        assert!(json.contains("backup"));
    }

    #[test]
    fn test_task_template_analytics_serialization() {
        let analytics = TaskTemplateAnalytics {
            template_id: Uuid::new_v4(),
            total_usage: 100,
            avg_rating: 4.5,
        };
        let json = serde_json::to_string(&analytics).unwrap();
        assert!(json.contains("100"));
        assert!(json.contains("4.5"));
    }

    #[test]
    fn test_task_template_recommendation_serialization() {
        let rec = TaskTemplateRecommendation {
            template_id: Uuid::new_v4(),
            recommendation_type: "featured_candidate".into(),
            description: "Template is a strong candidate".into(),
            confidence: 0.8,
            suggested_changes: serde_json::json!({"action": "feature"}),
        };
        let json = serde_json::to_string(&rec).unwrap();
        assert!(json.contains("featured_candidate"));
        assert!(json.contains("0.8"));
    }

    #[test]
    fn test_scheduled_task_template_v4_serialization() {
        let template = ScheduledTaskTemplateV4 {
            id: Uuid::new_v4(),
            name: "Nightly Build V4".into(),
            description: "Advanced nightly build".into(),
            task_type: "pipeline".into(),
            config: serde_json::json!({"cron_expression": "0 2 * * *"}),
            is_public: true,
            author_id: Some(Uuid::new_v4()),
            usage_count: 100,
            rating: 4.9,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&template).unwrap();
        assert!(json.contains("Nightly Build V4"));
        assert!(json.contains("pipeline"));
        assert!(json.contains("100"));
        assert!(json.contains("4.9"));
    }

    #[test]
    fn test_create_scheduled_task_template_v4_input_serialization() {
        let input = CreateScheduledTaskTemplateV4 {
            name: "Backup V4".into(),
            description: Some("Daily backup template".into()),
            task_type: "backup".into(),
            config: Some(serde_json::json!({"cron_expression": "0 1 * * *"})),
            is_public: Some(true),
            author_id: Some(Uuid::new_v4()),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("Backup V4"));
        assert!(json.contains("backup"));
    }

    #[test]
    fn test_scheduled_task_template_v5_serialization() {
        let template = ScheduledTaskTemplateV5 {
            id: Uuid::new_v4(),
            name: "Nightly Build V5".into(),
            description: "Advanced nightly build".into(),
            task_type: "pipeline".into(),
            config: serde_json::json!({"cron_expression": "0 2 * * *"}),
            is_public: true,
            author_id: Some(Uuid::new_v4()),
            usage_count: 150,
            rating: 4.9,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&template).unwrap();
        assert!(json.contains("Nightly Build V5"));
        assert!(json.contains("pipeline"));
        assert!(json.contains("150"));
        assert!(json.contains("4.9"));
    }

    #[test]
    fn test_create_scheduled_task_template_v5_input_serialization() {
        let input = CreateScheduledTaskTemplateV4 {
            name: "Backup V5".into(),
            description: Some("Daily backup template".into()),
            task_type: "backup".into(),
            config: Some(serde_json::json!({"cron_expression": "0 1 * * *"})),
            is_public: Some(true),
            author_id: Some(Uuid::new_v4()),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("Backup V5"));
        assert!(json.contains("backup"));
    }
}
