#![forbid(unsafe_code)]

//! Scheduled task management for CivitForge.
//!
//! Provides task CRUD, cron scheduling, run history tracking,
//! task dependencies, parallel execution, error recovery, and task execution.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use civit_db::models::{ScheduledTaskTemplateV8, WorkflowTemplateReviewV7, ScheduledTaskTemplateV10, ScheduledTaskTemplateV11, ScheduledTaskTemplateV12, ScheduledTaskTemplateV13, ScheduledTaskTemplateV15, ScheduledTaskTemplateV16, ScheduledTaskTemplateV17, ScheduledTaskTemplateV18, ScheduledTaskTemplateV19, ScheduledTaskTemplateRatingV20, ScheduledTaskTemplateCategoryV20};
use crate::workflow_engine::{WorkflowTemplateAnalytics, WorkflowTemplateRecommendation};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTaskTemplateV7 {
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
struct ScheduledTaskTemplateV7Row {
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

impl From<ScheduledTaskTemplateV7Row> for ScheduledTaskTemplateV7 {
    fn from(row: ScheduledTaskTemplateV7Row) -> Self {
        ScheduledTaskTemplateV7 {
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
struct ScheduledTaskTemplateV8Row {
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

impl From<ScheduledTaskTemplateV8Row> for ScheduledTaskTemplateV8 {
    fn from(row: ScheduledTaskTemplateV8Row) -> Self {
        ScheduledTaskTemplateV8 {
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
struct ScheduledTaskTemplateV10Row {
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

impl From<ScheduledTaskTemplateV10Row> for ScheduledTaskTemplateV10 {
    fn from(row: ScheduledTaskTemplateV10Row) -> Self {
        ScheduledTaskTemplateV10 {
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
struct WorkflowTemplateReviewV7Row {
    id: Uuid,
    template_id: Uuid,
    user_id: Uuid,
    rating: i32,
    review: String,
    helpful_count: i32,
    created_at: DateTime<Utc>,
}

impl From<WorkflowTemplateReviewV7Row> for WorkflowTemplateReviewV7 {
    fn from(row: WorkflowTemplateReviewV7Row) -> Self {
        WorkflowTemplateReviewV7 {
            id: row.id,
            template_id: row.template_id,
            user_id: row.user_id,
            rating: row.rating,
            review: row.review,
            helpful_count: row.helpful_count,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ScheduledTaskTemplateV18Row {
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

impl From<ScheduledTaskTemplateV18Row> for ScheduledTaskTemplateV18 {
    fn from(row: ScheduledTaskTemplateV18Row) -> Self {
        ScheduledTaskTemplateV18 {
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
struct ScheduledTaskTemplateV19Row {
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

impl From<ScheduledTaskTemplateV19Row> for ScheduledTaskTemplateV19 {
    fn from(row: ScheduledTaskTemplateV19Row) -> Self {
        ScheduledTaskTemplateV19 {
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

    // --- V10: Template Ratings V7, Analytics V7, Recommendations V7, Marketplace V7 ---

    pub async fn create_template_v7(
        &self,
        input: CreateScheduledTaskTemplateV4,
    ) -> Result<ScheduledTaskTemplateV7, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV7Row>(
            r#"INSERT INTO scheduled_task_templates_v7 (name, description, task_type, config, is_public, author_id)
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

    pub async fn get_template_v7(&self, id: Uuid) -> Result<Option<ScheduledTaskTemplateV7>, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV7Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v7 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates_v7(&self) -> Result<Vec<ScheduledTaskTemplateV7>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV7Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v7 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates_v7(&self) -> Result<Vec<ScheduledTaskTemplateV7>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV7Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v7 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_v7_by_type(
        &self,
        task_type: &str,
    ) -> Result<Vec<ScheduledTaskTemplateV7>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV7Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v7 WHERE task_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(task_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template_v7(
        &self,
        id: Uuid,
        input: UpdateScheduledTaskTemplateV4,
    ) -> Result<ScheduledTaskTemplateV7, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV7Row>(
            r#"UPDATE scheduled_task_templates_v7 SET
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

    pub async fn delete_template_v7(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scheduled_task_templates_v7 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn record_template_v7_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE scheduled_task_templates_v7 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v7_analytics(
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
             FROM scheduled_task_templates_v7 WHERE id = $1"#,
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

    pub async fn get_template_v7_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<TaskTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v7(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v7_analytics(template_id).await?;
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

    pub async fn get_marketplace_templates_v7(
        &self,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV7>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV7Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v7 WHERE is_public = true
             ORDER BY rating DESC, usage_count DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn search_marketplace_templates_v7(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV7>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV7Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v7 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_task_from_template_v7(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        task_name: Option<&str>,
    ) -> Result<ScheduledTask, sqlx::Error> {
        let template = self.get_template_v7(template_id).await?
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

        let _ = self.record_template_v7_usage(template_id, user_id).await;

        Ok(task)
    }

    // --- V11: Template Ratings V8, Analytics V8, Recommendations V8, Marketplace V8 ---

    pub async fn create_template_v8(
        &self,
        input: CreateScheduledTaskTemplateV4,
    ) -> Result<ScheduledTaskTemplateV8, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV8Row>(
            r#"INSERT INTO scheduled_task_templates_v8 (name, description, task_type, config, is_public, author_id)
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

    pub async fn get_template_v8(&self, id: Uuid) -> Result<Option<ScheduledTaskTemplateV8>, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV8Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v8 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates_v8(&self) -> Result<Vec<ScheduledTaskTemplateV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV8Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v8 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates_v8(&self) -> Result<Vec<ScheduledTaskTemplateV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV8Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v8 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_v8_by_type(
        &self,
        task_type: &str,
    ) -> Result<Vec<ScheduledTaskTemplateV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV8Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v8 WHERE task_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(task_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template_v8(
        &self,
        id: Uuid,
        input: UpdateScheduledTaskTemplateV4,
    ) -> Result<ScheduledTaskTemplateV8, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV8Row>(
            r#"UPDATE scheduled_task_templates_v8 SET
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

    pub async fn delete_template_v8(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scheduled_task_templates_v8 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn record_template_v8_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE scheduled_task_templates_v8 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v8_analytics(
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
             FROM scheduled_task_templates_v8 WHERE id = $1"#,
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

    pub async fn get_template_v8_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<TaskTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v8(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v8_analytics(template_id).await?;
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

    pub async fn get_marketplace_templates_v8(
        &self,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV8Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v8 WHERE is_public = true
             ORDER BY rating DESC, usage_count DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn search_marketplace_templates_v8(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV8Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v8 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_task_from_template_v8(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        task_name: Option<&str>,
    ) -> Result<ScheduledTask, sqlx::Error> {
        let template = self.get_template_v8(template_id).await?
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

        let _ = self.record_template_v8_usage(template_id, user_id).await;

        Ok(task)
    }

    // --- V9: Scheduled Task Template V9 with Ratings, Analytics, Recommendations, Marketplace ---

    pub async fn create_template_v9(
        &self,
        input: CreateScheduledTaskTemplate,
    ) -> Result<ScheduledTaskTemplateV8, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV8Row>(
            r#"INSERT INTO scheduled_task_templates_v9 (name, description, task_type, config, is_public, author_id)
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

    pub async fn get_template_v9(&self, id: Uuid) -> Result<Option<ScheduledTaskTemplateV8>, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV8Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v9 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates_v9(&self) -> Result<Vec<ScheduledTaskTemplateV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV8Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v9 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates_v9(&self) -> Result<Vec<ScheduledTaskTemplateV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV8Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v9 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_v9_by_type(
        &self,
        task_type: &str,
    ) -> Result<Vec<ScheduledTaskTemplateV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV8Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v9 WHERE task_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(task_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template_v9(
        &self,
        id: Uuid,
        input: UpdateScheduledTaskTemplate,
    ) -> Result<ScheduledTaskTemplateV8, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV8Row>(
            r#"UPDATE scheduled_task_templates_v9 SET
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

    pub async fn delete_template_v9(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scheduled_task_templates_v9 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn add_template_v9_review(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        rating: i32,
        review: &str,
    ) -> Result<WorkflowTemplateReviewV7, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateReviewV7Row>(
            r#"INSERT INTO workflow_template_reviews_v8 (template_id, user_id, rating, review)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (template_id, user_id) DO UPDATE SET rating = $3, review = $4
             RETURNING id, template_id, user_id, rating, review, helpful_count, created_at"#,
        )
        .bind(template_id)
        .bind(user_id)
        .bind(rating)
        .bind(review)
        .fetch_one(&self.pool)
        .await?;

        self.recalculate_template_v9_rating(template_id).await?;

        Ok(row.into())
    }

    async fn recalculate_template_v9_rating(&self, template_id: Uuid) -> Result<(), sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct RatingAvg {
            avg_rating: Option<f64>,
        }

        let row = sqlx::query_as::<_, RatingAvg>(
            r#"SELECT AVG(rating::double precision) as avg_rating
             FROM workflow_template_reviews_v8 WHERE template_id = $1"#,
        )
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        let avg = row.avg_rating.unwrap_or(0.0);
        sqlx::query("UPDATE scheduled_task_templates_v9 SET rating = $2 WHERE id = $1")
            .bind(template_id)
            .bind(avg)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn mark_template_v9_review_helpful(
        &self,
        review_id: Uuid,
    ) -> Result<WorkflowTemplateReviewV7, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateReviewV7Row>(
            r#"UPDATE workflow_template_reviews_v8 SET helpful_count = helpful_count + 1
             WHERE id = $1
             RETURNING id, template_id, user_id, rating, review, helpful_count, created_at"#,
        )
        .bind(review_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_template_v9_reviews(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<WorkflowTemplateReviewV7>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateReviewV7Row>(
            r#"SELECT id, template_id, user_id, rating, review, helpful_count, created_at
             FROM workflow_template_reviews_v8 WHERE template_id = $1
             ORDER BY helpful_count DESC, created_at DESC"#,
        )
        .bind(template_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn record_template_v9_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE scheduled_task_templates_v9 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v9_analytics(
        &self,
        template_id: Uuid,
    ) -> Result<WorkflowTemplateAnalytics, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AnalyticsRow {
            total_usage: i64,
            avg_rating: f64,
            total_reviews: i64,
        }

        let row = sqlx::query_as::<_, AnalyticsRow>(
            r#"SELECT
                usage_count as total_usage,
                rating as avg_rating,
                (SELECT COUNT(*) FROM workflow_template_reviews_v8 WHERE template_id = $1) as total_reviews
             FROM scheduled_task_templates_v9 WHERE id = $1"#,
        )
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(WorkflowTemplateAnalytics {
            template_id,
            total_usage: row.total_usage,
            avg_rating: row.avg_rating,
            total_reviews: row.total_reviews,
        })
    }

    pub async fn get_template_v9_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<WorkflowTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v9(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v9_analytics(template_id).await?;
        let mut recommendations = Vec::new();

        if analytics.total_usage == 0 {
            recommendations.push(WorkflowTemplateRecommendation {
                template_id,
                recommendation_type: "unused".into(),
                description: "Template has never been used. Consider promoting it in the marketplace.".into(),
                confidence: 0.9,
                suggested_changes: serde_json::json!({"action": "promote"}),
            });
        }

        if analytics.avg_rating < 3.0 && analytics.total_reviews > 0 {
            recommendations.push(WorkflowTemplateRecommendation {
                template_id,
                recommendation_type: "low_rating".into(),
                description: format!("Template has a low rating of {:.1}. Consider reviewing and improving the template.", analytics.avg_rating),
                confidence: 0.85,
                suggested_changes: serde_json::json!({"action": "improve", "current_rating": analytics.avg_rating}),
            });
        }

        if template.is_public && analytics.avg_rating >= 4.0 && analytics.total_usage > 10 {
            recommendations.push(WorkflowTemplateRecommendation {
                template_id,
                recommendation_type: "featured_candidate".into(),
                description: "Template is a strong candidate for featuring in the marketplace.".into(),
                confidence: 0.8,
                suggested_changes: serde_json::json!({"action": "feature", "rating": analytics.avg_rating, "usage": analytics.total_usage}),
            });
        }

        Ok(recommendations)
    }

    pub async fn get_marketplace_templates_v9(
        &self,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV8Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v9 WHERE is_public = true
             ORDER BY rating DESC, usage_count DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn search_marketplace_templates_v9(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV8Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v9 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_task_from_template_v9(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        task_name: Option<&str>,
    ) -> Result<ScheduledTask, sqlx::Error> {
        let template = self.get_template_v9(template_id).await?
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

        let _ = self.record_template_v9_usage(template_id, user_id).await;

        Ok(task)
    }

    // --- V13: Scheduled Task Template V10 with Ratings, Analytics, Recommendations, Marketplace ---

    pub async fn create_template_v10(
        &self,
        input: CreateScheduledTaskTemplate,
    ) -> Result<ScheduledTaskTemplateV10, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV10Row>(
            r#"INSERT INTO scheduled_task_templates_v10 (name, description, task_type, config, is_public, author_id)
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

    pub async fn get_template_v10(&self, id: Uuid) -> Result<Option<ScheduledTaskTemplateV10>, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV10Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v10 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates_v10(&self) -> Result<Vec<ScheduledTaskTemplateV10>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV10Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v10 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates_v10(&self) -> Result<Vec<ScheduledTaskTemplateV10>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV10Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v10 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_v10_by_type(
        &self,
        task_type: &str,
    ) -> Result<Vec<ScheduledTaskTemplateV10>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV10Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v10 WHERE task_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(task_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template_v10(
        &self,
        id: Uuid,
        input: UpdateScheduledTaskTemplate,
    ) -> Result<ScheduledTaskTemplateV10, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV10Row>(
            r#"UPDATE scheduled_task_templates_v10 SET
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

    pub async fn delete_template_v10(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scheduled_task_templates_v10 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn add_template_v10_review(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        rating: i32,
        review: &str,
    ) -> Result<WorkflowTemplateReviewV7, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateReviewV7Row>(
            r#"INSERT INTO workflow_template_reviews_v9 (template_id, user_id, rating, review)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (template_id, user_id) DO UPDATE SET rating = $3, review = $4
             RETURNING id, template_id, user_id, rating, review, helpful_count, created_at"#,
        )
        .bind(template_id)
        .bind(user_id)
        .bind(rating)
        .bind(review)
        .fetch_one(&self.pool)
        .await?;

        self.recalculate_template_v10_rating(template_id).await?;

        Ok(row.into())
    }

    async fn recalculate_template_v10_rating(&self, template_id: Uuid) -> Result<(), sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct RatingAvg {
            avg_rating: Option<f64>,
        }

        let row = sqlx::query_as::<_, RatingAvg>(
            r#"SELECT AVG(rating::double precision) as avg_rating
             FROM workflow_template_reviews_v9 WHERE template_id = $1"#,
        )
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        let avg = row.avg_rating.unwrap_or(0.0);
        sqlx::query("UPDATE scheduled_task_templates_v10 SET rating = $2 WHERE id = $1")
            .bind(template_id)
            .bind(avg)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn mark_template_v10_review_helpful(
        &self,
        review_id: Uuid,
    ) -> Result<WorkflowTemplateReviewV7, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateReviewV7Row>(
            r#"UPDATE workflow_template_reviews_v9 SET helpful_count = helpful_count + 1
             WHERE id = $1
             RETURNING id, template_id, user_id, rating, review, helpful_count, created_at"#,
        )
        .bind(review_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_template_v10_reviews(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<WorkflowTemplateReviewV7>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateReviewV7Row>(
            r#"SELECT id, template_id, user_id, rating, review, helpful_count, created_at
             FROM workflow_template_reviews_v9 WHERE template_id = $1
             ORDER BY helpful_count DESC, created_at DESC"#,
        )
        .bind(template_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn record_template_v10_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE scheduled_task_templates_v10 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v10_analytics(
        &self,
        template_id: Uuid,
    ) -> Result<WorkflowTemplateAnalytics, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AnalyticsRow {
            total_usage: i64,
            avg_rating: f64,
            total_reviews: i64,
        }

        let row = sqlx::query_as::<_, AnalyticsRow>(
            r#"SELECT
                usage_count as total_usage,
                rating as avg_rating,
                (SELECT COUNT(*) FROM workflow_template_reviews_v9 WHERE template_id = $1) as total_reviews
             FROM scheduled_task_templates_v10 WHERE id = $1"#,
        )
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(WorkflowTemplateAnalytics {
            template_id,
            total_usage: row.total_usage,
            avg_rating: row.avg_rating,
            total_reviews: row.total_reviews,
        })
    }

    pub async fn get_template_v10_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<WorkflowTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v10(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v10_analytics(template_id).await?;
        let mut recommendations = Vec::new();

        if analytics.total_usage == 0 {
            recommendations.push(WorkflowTemplateRecommendation {
                template_id,
                recommendation_type: "unused".into(),
                description: "Template has never been used. Consider promoting it in the marketplace.".into(),
                confidence: 0.9,
                suggested_changes: serde_json::json!({"action": "promote"}),
            });
        }

        if analytics.avg_rating < 3.0 && analytics.total_reviews > 0 {
            recommendations.push(WorkflowTemplateRecommendation {
                template_id,
                recommendation_type: "low_rating".into(),
                description: format!("Template has a low rating of {:.1}. Consider reviewing and improving the template.", analytics.avg_rating),
                confidence: 0.85,
                suggested_changes: serde_json::json!({"action": "improve", "current_rating": analytics.avg_rating}),
            });
        }

        if template.is_public && analytics.avg_rating >= 4.0 && analytics.total_usage > 10 {
            recommendations.push(WorkflowTemplateRecommendation {
                template_id,
                recommendation_type: "featured_candidate".into(),
                description: "Template is a strong candidate for featuring in the marketplace.".into(),
                confidence: 0.8,
                suggested_changes: serde_json::json!({"action": "feature", "rating": analytics.avg_rating, "usage": analytics.total_usage}),
            });
        }

        Ok(recommendations)
    }

    pub async fn get_marketplace_templates_v10(
        &self,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV10>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV10Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v10 WHERE is_public = true
             ORDER BY rating DESC, usage_count DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn search_marketplace_templates_v10(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV10>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV10Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v10 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_task_from_template_v10(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        task_name: Option<&str>,
    ) -> Result<ScheduledTask, sqlx::Error> {
        let template = self.get_template_v10(template_id).await?
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

        let _ = self.record_template_v10_usage(template_id, user_id).await;

        Ok(task)
    }

    // --- V11: Scheduled Task Templates V11 with Ratings, Analytics, Recommendations, Marketplace ---

    pub async fn create_template_v11(
        &self,
        input: CreateScheduledTaskTemplateV2,
    ) -> Result<ScheduledTaskTemplateV11, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV11>(
            r#"INSERT INTO scheduled_task_templates_v11 (name, description, task_type, config, is_public, author_id)
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

        Ok(row)
    }

    pub async fn get_template_v11(&self, id: Uuid) -> Result<Option<ScheduledTaskTemplateV11>, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV11>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v11 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn list_templates_v11(&self) -> Result<Vec<ScheduledTaskTemplateV11>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV11>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v11 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn list_public_templates_v11(&self) -> Result<Vec<ScheduledTaskTemplateV11>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV11>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v11 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn list_templates_v11_by_type(
        &self,
        task_type: &str,
    ) -> Result<Vec<ScheduledTaskTemplateV11>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV11>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v11 WHERE task_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(task_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn update_template_v11(
        &self,
        id: Uuid,
        input: UpdateScheduledTaskTemplateV2,
    ) -> Result<ScheduledTaskTemplateV11, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV11>(
            r#"UPDATE scheduled_task_templates_v11 SET
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

        Ok(row)
    }

    pub async fn delete_template_v11(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scheduled_task_templates_v11 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn record_template_v11_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE scheduled_task_templates_v11 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v11_analytics(
        &self,
        template_id: Uuid,
    ) -> Result<TaskTemplateAnalytics, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AnalyticsRow {
            total_usage: i64,
            avg_rating: f64,
        }

        let row = sqlx::query_as::<_, AnalyticsRow>(
            r#"SELECT
                usage_count as total_usage,
                rating as avg_rating
             FROM scheduled_task_templates_v11 WHERE id = $1"#,
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

    pub async fn get_template_v11_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<TaskTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v11(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v11_analytics(template_id).await?;
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

    pub async fn get_marketplace_templates_v11(
        &self,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV11>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV11>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v11 WHERE is_public = true
             ORDER BY rating DESC, usage_count DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn search_marketplace_templates_v11(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV11>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV11>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v11 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn create_task_from_template_v11(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        task_name: Option<&str>,
    ) -> Result<ScheduledTask, sqlx::Error> {
        let template = self.get_template_v11(template_id).await?
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

        let _ = self.record_template_v11_usage(template_id, user_id).await;

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

// --- V15: Scheduled Task Template V12 with Template Ratings V12, Analytics V12, Recommendations V12, Marketplace V12 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduledTaskTemplateV12 {
    pub name: String,
    pub description: Option<String>,
    pub task_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScheduledTaskTemplateV12 {
    pub name: Option<String>,
    pub description: Option<String>,
    pub task_type: Option<String>,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
}

#[derive(Debug, sqlx::FromRow)]
struct ScheduledTaskTemplateV12Row {
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

impl From<ScheduledTaskTemplateV12Row> for ScheduledTaskTemplateV12 {
    fn from(row: ScheduledTaskTemplateV12Row) -> Self {
        ScheduledTaskTemplateV12 {
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

impl ScheduledTaskService {
    pub async fn create_template_v12(
        &self,
        input: CreateScheduledTaskTemplateV12,
    ) -> Result<ScheduledTaskTemplateV12, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV12Row>(
            r#"INSERT INTO scheduled_task_templates_v12 (name, description, task_type, config, is_public, author_id)
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

    pub async fn get_template_v12(&self, id: Uuid) -> Result<Option<ScheduledTaskTemplateV12>, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV12Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v12 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates_v12(&self) -> Result<Vec<ScheduledTaskTemplateV12>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV12Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v12 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates_v12(&self) -> Result<Vec<ScheduledTaskTemplateV12>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV12Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v12 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_v12_by_type(
        &self,
        task_type: &str,
    ) -> Result<Vec<ScheduledTaskTemplateV12>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV12Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v12 WHERE task_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(task_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template_v12(
        &self,
        id: Uuid,
        input: UpdateScheduledTaskTemplateV12,
    ) -> Result<ScheduledTaskTemplateV12, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV12Row>(
            r#"UPDATE scheduled_task_templates_v12 SET
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

    pub async fn delete_template_v12(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scheduled_task_templates_v12 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn record_template_v12_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE scheduled_task_templates_v12 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v12_analytics(
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
             FROM scheduled_task_templates_v12 WHERE id = $1"#,
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

    pub async fn get_template_v12_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<TaskTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v12(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v12_analytics(template_id).await?;
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

    pub async fn get_marketplace_templates_v12(
        &self,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV12>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV12Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v12 WHERE is_public = true
             ORDER BY rating DESC, usage_count DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn search_marketplace_templates_v12(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV12>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV12Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v12 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_task_from_template_v12(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        task_name: Option<&str>,
    ) -> Result<ScheduledTask, sqlx::Error> {
        let template = self.get_template_v12(template_id).await?
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

        let _ = self.record_template_v12_usage(template_id, user_id).await;

        Ok(task)
    }
}

// --- V16: Scheduled Task Template V13 with Template Ratings V13, Analytics V13, Recommendations V13, Marketplace V13 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduledTaskTemplateV13 {
    pub name: String,
    pub description: Option<String>,
    pub task_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScheduledTaskTemplateV13 {
    pub name: Option<String>,
    pub description: Option<String>,
    pub task_type: Option<String>,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
}

#[derive(Debug, sqlx::FromRow)]
struct ScheduledTaskTemplateV13Row {
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

impl From<ScheduledTaskTemplateV13Row> for ScheduledTaskTemplateV13 {
    fn from(row: ScheduledTaskTemplateV13Row) -> Self {
        ScheduledTaskTemplateV13 {
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

impl ScheduledTaskService {
    pub async fn create_template_v13(
        &self,
        input: CreateScheduledTaskTemplateV13,
    ) -> Result<ScheduledTaskTemplateV13, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV13Row>(
            r#"INSERT INTO scheduled_task_templates_v13 (name, description, task_type, config, is_public, author_id)
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

    pub async fn get_template_v13(&self, id: Uuid) -> Result<Option<ScheduledTaskTemplateV13>, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV13Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v13 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates_v13(&self) -> Result<Vec<ScheduledTaskTemplateV13>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV13Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v13 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates_v13(&self) -> Result<Vec<ScheduledTaskTemplateV13>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV13Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v13 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_v13_by_type(
        &self,
        task_type: &str,
    ) -> Result<Vec<ScheduledTaskTemplateV13>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV13Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v13 WHERE task_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(task_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template_v13(
        &self,
        id: Uuid,
        input: UpdateScheduledTaskTemplateV13,
    ) -> Result<ScheduledTaskTemplateV13, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV13Row>(
            r#"UPDATE scheduled_task_templates_v13 SET
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

    pub async fn delete_template_v13(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scheduled_task_templates_v13 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn record_template_v13_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE scheduled_task_templates_v13 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v13_analytics(
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
             FROM scheduled_task_templates_v13 WHERE id = $1"#,
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

    pub async fn get_template_v13_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<TaskTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v13(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v13_analytics(template_id).await?;
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

    pub async fn get_marketplace_templates_v13(
        &self,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV13>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV13Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v13 WHERE is_public = true
             ORDER BY rating DESC, usage_count DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn search_marketplace_templates_v13(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV13>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV13Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v13 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_task_from_template_v13(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        task_name: Option<&str>,
    ) -> Result<ScheduledTask, sqlx::Error> {
        let template = self.get_template_v13(template_id).await?
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

        let _ = self.record_template_v13_usage(template_id, user_id).await;

        Ok(task)
    }

    // --- V15: Template Ratings V15, Analytics V15, Recommendations V15, Marketplace V15 ---

    pub async fn create_template_v15(
        &self,
        input: CreateScheduledTaskTemplateV4,
    ) -> Result<ScheduledTaskTemplateV15, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct ScheduledTaskTemplateV15Row {
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

        impl From<ScheduledTaskTemplateV15Row> for ScheduledTaskTemplateV15 {
            fn from(row: ScheduledTaskTemplateV15Row) -> Self {
                ScheduledTaskTemplateV15 {
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

        let row = sqlx::query_as::<_, ScheduledTaskTemplateV15Row>(
            r#"INSERT INTO scheduled_task_templates_v15 (name, description, task_type, config, is_public, author_id)
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

    pub async fn get_template_v15(&self, id: Uuid) -> Result<Option<ScheduledTaskTemplateV15>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct ScheduledTaskTemplateV15Row {
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

        impl From<ScheduledTaskTemplateV15Row> for ScheduledTaskTemplateV15 {
            fn from(row: ScheduledTaskTemplateV15Row) -> Self {
                ScheduledTaskTemplateV15 {
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

        let row = sqlx::query_as::<_, ScheduledTaskTemplateV15Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v15 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates_v15(&self) -> Result<Vec<ScheduledTaskTemplateV15>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct ScheduledTaskTemplateV15Row {
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

        impl From<ScheduledTaskTemplateV15Row> for ScheduledTaskTemplateV15 {
            fn from(row: ScheduledTaskTemplateV15Row) -> Self {
                ScheduledTaskTemplateV15 {
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

        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV15Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v15 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates_v15(&self) -> Result<Vec<ScheduledTaskTemplateV15>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct ScheduledTaskTemplateV15Row {
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

        impl From<ScheduledTaskTemplateV15Row> for ScheduledTaskTemplateV15 {
            fn from(row: ScheduledTaskTemplateV15Row) -> Self {
                ScheduledTaskTemplateV15 {
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

        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV15Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v15 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_v15_by_type(
        &self,
        task_type: &str,
    ) -> Result<Vec<ScheduledTaskTemplateV15>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct ScheduledTaskTemplateV15Row {
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

        impl From<ScheduledTaskTemplateV15Row> for ScheduledTaskTemplateV15 {
            fn from(row: ScheduledTaskTemplateV15Row) -> Self {
                ScheduledTaskTemplateV15 {
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

        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV15Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v15 WHERE task_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(task_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template_v15(
        &self,
        id: Uuid,
        input: UpdateScheduledTaskTemplateV4,
    ) -> Result<ScheduledTaskTemplateV15, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct ScheduledTaskTemplateV15Row {
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

        impl From<ScheduledTaskTemplateV15Row> for ScheduledTaskTemplateV15 {
            fn from(row: ScheduledTaskTemplateV15Row) -> Self {
                ScheduledTaskTemplateV15 {
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

        let row = sqlx::query_as::<_, ScheduledTaskTemplateV15Row>(
            r#"UPDATE scheduled_task_templates_v15 SET
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

    pub async fn delete_template_v15(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scheduled_task_templates_v15 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn record_template_v15_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE scheduled_task_templates_v15 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v15_analytics(
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
             FROM scheduled_task_templates_v15 WHERE id = $1"#,
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

    pub async fn get_template_v15_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<TaskTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v15(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v15_analytics(template_id).await?;
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

    pub async fn get_marketplace_templates_v15(
        &self,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV15>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct ScheduledTaskTemplateV15Row {
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

        impl From<ScheduledTaskTemplateV15Row> for ScheduledTaskTemplateV15 {
            fn from(row: ScheduledTaskTemplateV15Row) -> Self {
                ScheduledTaskTemplateV15 {
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

        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV15Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v15 WHERE is_public = true
             ORDER BY rating DESC, usage_count DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn search_marketplace_templates_v15(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV15>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct ScheduledTaskTemplateV15Row {
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

        impl From<ScheduledTaskTemplateV15Row> for ScheduledTaskTemplateV15 {
            fn from(row: ScheduledTaskTemplateV15Row) -> Self {
                ScheduledTaskTemplateV15 {
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

        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV15Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v15 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_task_from_template_v15(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        task_name: Option<&str>,
    ) -> Result<ScheduledTask, sqlx::Error> {
        let template = self.get_template_v15(template_id).await?
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

        let _ = self.record_template_v15_usage(template_id, user_id).await;

        Ok(task)
    }

    // --- V16: Scheduled Task Template V16 with Template Ratings, Analytics, Recommendations, Marketplace ---

    pub async fn create_template_v16(
        &self,
        input: CreateScheduledTaskTemplateV4,
    ) -> Result<ScheduledTaskTemplateV16, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct Row {
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

        let row = sqlx::query_as::<_, Row>(
            r#"INSERT INTO scheduled_task_templates_v16 (name, description, task_type, config, is_public, author_id)
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

        Ok(ScheduledTaskTemplateV16 {
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
        })
    }

    pub async fn get_template_v16(&self, id: Uuid) -> Result<Option<ScheduledTaskTemplateV16>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct Row {
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

        let row = sqlx::query_as::<_, Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v16 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| ScheduledTaskTemplateV16 {
            id: r.id,
            name: r.name,
            description: r.description,
            task_type: r.task_type,
            config: r.config,
            is_public: r.is_public,
            author_id: r.author_id,
            usage_count: r.usage_count,
            rating: r.rating,
            created_at: r.created_at,
        }))
    }

    pub async fn list_templates_v16(&self) -> Result<Vec<ScheduledTaskTemplateV16>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct Row {
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

        let rows = sqlx::query_as::<_, Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v16 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| ScheduledTaskTemplateV16 {
            id: r.id,
            name: r.name,
            description: r.description,
            task_type: r.task_type,
            config: r.config,
            is_public: r.is_public,
            author_id: r.author_id,
            usage_count: r.usage_count,
            rating: r.rating,
            created_at: r.created_at,
        }).collect())
    }

    pub async fn list_public_templates_v16(&self) -> Result<Vec<ScheduledTaskTemplateV16>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct Row {
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

        let rows = sqlx::query_as::<_, Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v16 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| ScheduledTaskTemplateV16 {
            id: r.id,
            name: r.name,
            description: r.description,
            task_type: r.task_type,
            config: r.config,
            is_public: r.is_public,
            author_id: r.author_id,
            usage_count: r.usage_count,
            rating: r.rating,
            created_at: r.created_at,
        }).collect())
    }

    pub async fn list_templates_v16_by_type(
        &self,
        task_type: &str,
    ) -> Result<Vec<ScheduledTaskTemplateV16>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct Row {
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

        let rows = sqlx::query_as::<_, Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v16 WHERE task_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(task_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| ScheduledTaskTemplateV16 {
            id: r.id,
            name: r.name,
            description: r.description,
            task_type: r.task_type,
            config: r.config,
            is_public: r.is_public,
            author_id: r.author_id,
            usage_count: r.usage_count,
            rating: r.rating,
            created_at: r.created_at,
        }).collect())
    }

    pub async fn update_template_v16(
        &self,
        id: Uuid,
        input: UpdateScheduledTaskTemplateV4,
    ) -> Result<ScheduledTaskTemplateV16, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct Row {
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

        let row = sqlx::query_as::<_, Row>(
            r#"UPDATE scheduled_task_templates_v16 SET
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

        Ok(ScheduledTaskTemplateV16 {
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
        })
    }

    pub async fn delete_template_v16(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scheduled_task_templates_v16 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn record_template_v16_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE scheduled_task_templates_v16 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v16_analytics(
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
             FROM scheduled_task_templates_v16 WHERE id = $1"#,
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

    pub async fn get_template_v16_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<TaskTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v16(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v16_analytics(template_id).await?;
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

    pub async fn get_marketplace_templates_v16(
        &self,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV16>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct Row {
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

        let rows = sqlx::query_as::<_, Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v16 WHERE is_public = true
             ORDER BY rating DESC, usage_count DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| ScheduledTaskTemplateV16 {
            id: r.id,
            name: r.name,
            description: r.description,
            task_type: r.task_type,
            config: r.config,
            is_public: r.is_public,
            author_id: r.author_id,
            usage_count: r.usage_count,
            rating: r.rating,
            created_at: r.created_at,
        }).collect())
    }

    pub async fn search_marketplace_templates_v16(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV16>, sqlx::Error> {
        #[derive(Debug, sqlx::FromRow)]
        struct Row {
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

        let rows = sqlx::query_as::<_, Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v16 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| ScheduledTaskTemplateV16 {
            id: r.id,
            name: r.name,
            description: r.description,
            task_type: r.task_type,
            config: r.config,
            is_public: r.is_public,
            author_id: r.author_id,
            usage_count: r.usage_count,
            rating: r.rating,
            created_at: r.created_at,
        }).collect())
    }

    pub async fn create_task_from_template_v16(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        task_name: Option<&str>,
    ) -> Result<ScheduledTask, sqlx::Error> {
        let template = self.get_template_v16(template_id).await?
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

        let _ = self.record_template_v16_usage(template_id, user_id).await;

        Ok(task)
    }

    // --- V17: Scheduled Task Template V17 with Template Ratings, Analytics, Recommendations, Marketplace ---

    pub async fn create_template_v17(
        &self,
        input: CreateScheduledTaskTemplateV4,
    ) -> Result<ScheduledTaskTemplateV17, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV17>(
            r#"INSERT INTO scheduled_task_templates_v17 (name, description, task_type, config, is_public, author_id)
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

        Ok(row)
    }

    pub async fn get_template_v17(&self, id: Uuid) -> Result<Option<ScheduledTaskTemplateV17>, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV17>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v17 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn list_templates_v17(&self) -> Result<Vec<ScheduledTaskTemplateV17>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV17>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v17 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn list_public_templates_v17(&self) -> Result<Vec<ScheduledTaskTemplateV17>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV17>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v17 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn list_templates_v17_by_type(
        &self,
        task_type: &str,
    ) -> Result<Vec<ScheduledTaskTemplateV17>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV17>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v17 WHERE task_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(task_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn update_template_v17(
        &self,
        id: Uuid,
        input: UpdateScheduledTaskTemplateV4,
    ) -> Result<ScheduledTaskTemplateV17, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV17>(
            r#"UPDATE scheduled_task_templates_v17 SET
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

        Ok(row)
    }

    pub async fn delete_template_v17(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scheduled_task_templates_v17 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn record_template_v17_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE scheduled_task_templates_v17 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v17_analytics(
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
             FROM scheduled_task_templates_v17 WHERE id = $1"#,
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

    pub async fn get_template_v17_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<TaskTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v17(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v17_analytics(template_id).await?;
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

    pub async fn get_marketplace_templates_v17(
        &self,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV17>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV17>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v17 WHERE is_public = true
             ORDER BY rating DESC, usage_count DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn search_marketplace_templates_v17(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV17>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV17>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v17 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn create_task_from_template_v17(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        task_name: Option<&str>,
    ) -> Result<ScheduledTask, sqlx::Error> {
        let template = self.get_template_v17(template_id).await?
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

        let _ = self.record_template_v17_usage(template_id, user_id).await;

        Ok(task)
    }

    // --- V18: Template Ratings V18, Analytics V18, Recommendations V18, Marketplace V18 ---

    pub async fn create_template_v18(
        &self,
        input: CreateScheduledTaskTemplateV4,
    ) -> Result<ScheduledTaskTemplateV18, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV18Row>(
            r#"INSERT INTO scheduled_task_templates_v18 (name, description, task_type, config, is_public, author_id)
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

    pub async fn get_template_v18(&self, id: Uuid) -> Result<Option<ScheduledTaskTemplateV18>, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV18Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v18 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates_v18(&self) -> Result<Vec<ScheduledTaskTemplateV18>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV18Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v18 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates_v18(&self) -> Result<Vec<ScheduledTaskTemplateV18>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV18Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v18 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_v18_by_type(
        &self,
        task_type: &str,
    ) -> Result<Vec<ScheduledTaskTemplateV18>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV18Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v18 WHERE task_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(task_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template_v18(
        &self,
        id: Uuid,
        input: UpdateScheduledTaskTemplateV4,
    ) -> Result<ScheduledTaskTemplateV18, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV18Row>(
            r#"UPDATE scheduled_task_templates_v18 SET
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

    pub async fn delete_template_v18(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scheduled_task_templates_v18 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn record_template_v18_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE scheduled_task_templates_v18 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v18_analytics(
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
             FROM scheduled_task_templates_v18 WHERE id = $1"#,
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

    pub async fn get_template_v18_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<TaskTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v18(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v18_analytics(template_id).await?;
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

    pub async fn get_marketplace_templates_v18(
        &self,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV18>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV18Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v18 WHERE is_public = true
             ORDER BY rating DESC, usage_count DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn search_marketplace_templates_v18(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV18>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV18Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v18 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_task_from_template_v18(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        task_name: Option<&str>,
    ) -> Result<ScheduledTask, sqlx::Error> {
        let template = self.get_template_v18(template_id).await?
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

        let _ = self.record_template_v18_usage(template_id, user_id).await;

        Ok(task)
    }

    // --- V19: Template Ratings, Analytics, Recommendations, Marketplace ---

    pub async fn create_template_v19(
        &self,
        input: CreateScheduledTaskTemplateV4,
    ) -> Result<ScheduledTaskTemplateV19, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV19Row>(
            r#"INSERT INTO scheduled_task_templates_v19 (name, description, task_type, config, is_public, author_id)
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

    pub async fn get_template_v19(&self, id: Uuid) -> Result<Option<ScheduledTaskTemplateV19>, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV19Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v19 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates_v19(&self) -> Result<Vec<ScheduledTaskTemplateV19>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV19Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v19 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates_v19(&self) -> Result<Vec<ScheduledTaskTemplateV19>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV19Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v19 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_v19_by_type(
        &self,
        task_type: &str,
    ) -> Result<Vec<ScheduledTaskTemplateV19>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV19Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v19 WHERE task_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(task_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template_v19(
        &self,
        id: Uuid,
        input: UpdateScheduledTaskTemplateV4,
    ) -> Result<ScheduledTaskTemplateV19, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateV19Row>(
            r#"UPDATE scheduled_task_templates_v19 SET
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

    pub async fn delete_template_v19(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scheduled_task_templates_v19 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn record_template_v19_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE scheduled_task_templates_v19 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v19_analytics(
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
             FROM scheduled_task_templates_v19 WHERE id = $1"#,
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

    pub async fn get_template_v19_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<TaskTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v19(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v19_analytics(template_id).await?;
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

    pub async fn get_marketplace_templates_v19(
        &self,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV19>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV19Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v19 WHERE is_public = true
             ORDER BY rating DESC, usage_count DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn search_marketplace_templates_v19(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV19>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV19Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v19 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_task_from_template_v19(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        task_name: Option<&str>,
    ) -> Result<ScheduledTask, sqlx::Error> {
        let template = self.get_template_v19(template_id).await?
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

        let _ = self.record_template_v19_usage(template_id, user_id).await;

        Ok(task)
    }

    // --- V20: Scheduled Task Template Ratings, Categories, Search, Recommendations ---

    pub async fn rate_task_template(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        rating: i32,
        review: &str,
    ) -> Result<ScheduledTaskTemplateRatingV20, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateRatingV20>(
            r#"INSERT INTO scheduled_task_template_ratings_v20 (template_id, user_id, rating, review)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (template_id, user_id) DO UPDATE SET rating = $3, review = $4
             RETURNING id, template_id, user_id, rating, review, created_at"#,
        )
        .bind(template_id)
        .bind(user_id)
        .bind(rating)
        .bind(review)
        .fetch_one(&self.pool)
        .await?;

        self.recalculate_task_template_rating(template_id).await?;

        Ok(row.into())
    }

    async fn recalculate_task_template_rating(
        &self,
        template_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct RatingAvg {
            avg_rating: Option<f64>,
        }

        let row = sqlx::query_as::<_, RatingAvg>(
            r#"SELECT AVG(rating::double precision) as avg_rating
             FROM scheduled_task_template_ratings_v20 WHERE template_id = $1"#,
        )
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        let avg = row.avg_rating.unwrap_or(0.0);
        sqlx::query("UPDATE scheduled_task_templates_v19 SET rating = $2 WHERE id = $1")
            .bind(template_id)
            .bind(avg)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_task_template_ratings(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<ScheduledTaskTemplateRatingV20>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateRatingV20>(
            r#"SELECT id, template_id, user_id, rating, review, created_at
             FROM scheduled_task_template_ratings_v20 WHERE template_id = $1
             ORDER BY created_at DESC"#,
        )
        .bind(template_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_task_category(
        &self,
        name: &str,
        description: &str,
        parent_id: Option<Uuid>,
    ) -> Result<ScheduledTaskTemplateCategoryV20, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateCategoryV20>(
            r#"INSERT INTO scheduled_task_template_categories_v20 (name, description, parent_id)
             VALUES ($1, $2, $3)
             RETURNING id, name, description, parent_id, created_at"#,
        )
        .bind(name)
        .bind(description)
        .bind(parent_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_task_category(
        &self,
        id: Uuid,
    ) -> Result<Option<ScheduledTaskTemplateCategoryV20>, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateCategoryV20>(
            r#"SELECT id, name, description, parent_id, created_at
             FROM scheduled_task_template_categories_v20 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_task_categories(
        &self,
    ) -> Result<Vec<ScheduledTaskTemplateCategoryV20>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateCategoryV20>(
            r#"SELECT id, name, description, parent_id, created_at
             FROM scheduled_task_template_categories_v20
             ORDER BY name ASC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_task_subcategories(
        &self,
        parent_id: Uuid,
    ) -> Result<Vec<ScheduledTaskTemplateCategoryV20>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateCategoryV20>(
            r#"SELECT id, name, description, parent_id, created_at
             FROM scheduled_task_template_categories_v20 WHERE parent_id = $1
             ORDER BY name ASC"#,
        )
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_task_category(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<ScheduledTaskTemplateCategoryV20, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskTemplateCategoryV20>(
            r#"UPDATE scheduled_task_template_categories_v20 SET
             name = COALESCE($2, name),
             description = COALESCE($3, description)
             WHERE id = $1
             RETURNING id, name, description, parent_id, created_at"#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_task_category(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM scheduled_task_template_categories_v20 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn search_task_templates_v20(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskTemplateV19>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskTemplateV19Row>(
            r#"SELECT id, name, description, task_type, config, is_public, author_id, usage_count, rating, created_at
             FROM scheduled_task_templates_v19
             WHERE name ILIKE $1 OR description ILIKE $1
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_task_template_recommendations_v20(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<TaskTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v19(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let mut recommendations = Vec::new();

        if template.usage_count == 0 {
            recommendations.push(TaskTemplateRecommendation {
                template_id,
                recommendation_type: "unused".into(),
                description: "Template has never been used. Consider promoting it.".into(),
                confidence: 0.9,
                suggested_changes: serde_json::json!({"action": "promote"}),
            });
        }

        if template.rating < 3.0 && template.usage_count > 0 {
            recommendations.push(TaskTemplateRecommendation {
                template_id,
                recommendation_type: "low_rating".into(),
                description: format!("Template has a low rating of {:.1}. Consider improving it.", template.rating),
                confidence: 0.85,
                suggested_changes: serde_json::json!({"action": "improve", "current_rating": template.rating}),
            });
        }

        if template.rating >= 4.0 && template.usage_count > 10 {
            recommendations.push(TaskTemplateRecommendation {
                template_id,
                recommendation_type: "featured_candidate".into(),
                description: "Template is a strong candidate for featuring.".into(),
                confidence: 0.8,
                suggested_changes: serde_json::json!({"action": "feature", "rating": template.rating, "usage": template.usage_count}),
            });
        }

        Ok(recommendations)
    }
}

// --- V24: Performance Metrics, Resource Usage, Cost Optimization, Capacity Planning ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTaskPerformanceMetricV24 {
    pub id: Uuid,
    pub task_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTaskResourceUsageV24 {
    pub id: Uuid,
    pub task_id: Uuid,
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: i64,
    pub disk_usage_bytes: i64,
    pub network_bytes_sent: i64,
    pub network_bytes_received: i64,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTaskCostOptimization {
    pub task_id: Uuid,
    pub total_executions: i64,
    pub avg_duration_ms: f64,
    pub avg_cpu_percent: f64,
    pub avg_memory_bytes: i64,
    pub estimated_monthly_cost_usd: f64,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTaskCapacityPlan {
    pub task_id: Uuid,
    pub avg_cpu_percent: f64,
    pub peak_cpu_percent: f64,
    pub avg_memory_bytes: i64,
    pub peak_memory_bytes: i64,
    pub execution_frequency_per_day: f64,
    pub projected_cpu_next_30d: f64,
    pub projected_memory_next_30d: i64,
    pub scaling_recommendation: String,
}

#[derive(Debug, sqlx::FromRow)]
struct ScheduledTaskPerformanceMetricV24Row {
    id: Uuid,
    task_id: Uuid,
    metric_name: String,
    metric_value: f64,
    measured_at: DateTime<Utc>,
}

impl From<ScheduledTaskPerformanceMetricV24Row> for ScheduledTaskPerformanceMetricV24 {
    fn from(row: ScheduledTaskPerformanceMetricV24Row) -> Self {
        ScheduledTaskPerformanceMetricV24 {
            id: row.id,
            task_id: row.task_id,
            metric_name: row.metric_name,
            metric_value: row.metric_value,
            measured_at: row.measured_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ScheduledTaskResourceUsageV24Row {
    id: Uuid,
    task_id: Uuid,
    cpu_usage_percent: f64,
    memory_usage_bytes: i64,
    disk_usage_bytes: i64,
    network_bytes_sent: i64,
    network_bytes_received: i64,
    measured_at: DateTime<Utc>,
}

impl From<ScheduledTaskResourceUsageV24Row> for ScheduledTaskResourceUsageV24 {
    fn from(row: ScheduledTaskResourceUsageV24Row) -> Self {
        ScheduledTaskResourceUsageV24 {
            id: row.id,
            task_id: row.task_id,
            cpu_usage_percent: row.cpu_usage_percent,
            memory_usage_bytes: row.memory_usage_bytes,
            disk_usage_bytes: row.disk_usage_bytes,
            network_bytes_sent: row.network_bytes_sent,
            network_bytes_received: row.network_bytes_received,
            measured_at: row.measured_at,
        }
    }
}

impl ScheduledTaskService {
    // --- Performance Metrics v21 ---

    pub async fn record_performance_metric_v24(
        &self,
        task_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<ScheduledTaskPerformanceMetricV24, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskPerformanceMetricV24Row>(
            r#"INSERT INTO scheduled_task_performance_v21 (task_id, metric_name, metric_value)
             VALUES ($1, $2, $3)
             RETURNING id, task_id, metric_name, metric_value, measured_at"#,
        )
        .bind(task_id)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_performance_metrics_v24(
        &self,
        task_id: Uuid,
        metric_name: Option<&str>,
    ) -> Result<Vec<ScheduledTaskPerformanceMetricV24>, sqlx::Error> {
        let rows = if let Some(name) = metric_name {
            sqlx::query_as::<_, ScheduledTaskPerformanceMetricV24Row>(
                r#"SELECT id, task_id, metric_name, metric_value, measured_at
                 FROM scheduled_task_performance_v21 WHERE task_id = $1 AND metric_name = $2
                 ORDER BY measured_at DESC"#,
            )
            .bind(task_id)
            .bind(name)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ScheduledTaskPerformanceMetricV24Row>(
                r#"SELECT id, task_id, metric_name, metric_value, measured_at
                 FROM scheduled_task_performance_v21 WHERE task_id = $1
                 ORDER BY measured_at DESC"#,
            )
            .bind(task_id)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    // --- Resource Usage Tracking v21 ---

    pub async fn record_resource_usage(
        &self,
        task_id: Uuid,
        cpu_usage_percent: f64,
        memory_usage_bytes: i64,
        disk_usage_bytes: i64,
        network_bytes_sent: i64,
        network_bytes_received: i64,
    ) -> Result<ScheduledTaskResourceUsageV24, sqlx::Error> {
        let row = sqlx::query_as::<_, ScheduledTaskResourceUsageV24Row>(
            r#"INSERT INTO scheduled_task_resource_usage_v21 (task_id, cpu_usage_percent, memory_usage_bytes, disk_usage_bytes, network_bytes_sent, network_bytes_received)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, task_id, cpu_usage_percent, memory_usage_bytes, disk_usage_bytes, network_bytes_sent, network_bytes_received, measured_at"#,
        )
        .bind(task_id)
        .bind(cpu_usage_percent)
        .bind(memory_usage_bytes)
        .bind(disk_usage_bytes)
        .bind(network_bytes_sent)
        .bind(network_bytes_received)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_resource_usage(
        &self,
        task_id: Uuid,
        limit: i64,
    ) -> Result<Vec<ScheduledTaskResourceUsageV24>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ScheduledTaskResourceUsageV24Row>(
            r#"SELECT id, task_id, cpu_usage_percent, memory_usage_bytes, disk_usage_bytes, network_bytes_sent, network_bytes_received, measured_at
             FROM scheduled_task_resource_usage_v21 WHERE task_id = $1
             ORDER BY measured_at DESC LIMIT $2"#,
        )
        .bind(task_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_avg_resource_usage(
        &self,
        task_id: Uuid,
    ) -> Result<(f64, i64, i64), sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AvgRow {
            avg_cpu: f64,
            avg_memory: i64,
            avg_disk: i64,
        }

        let row = sqlx::query_as::<_, AvgRow>(
            r#"SELECT
                COALESCE(AVG(cpu_usage_percent), 0) as avg_cpu,
                COALESCE(AVG(memory_usage_bytes), 0)::BIGINT as avg_memory,
                COALESCE(AVG(disk_usage_bytes), 0)::BIGINT as avg_disk
             FROM scheduled_task_resource_usage_v21 WHERE task_id = $1"#,
        )
        .bind(task_id)
        .fetch_one(&self.pool)
        .await?;

        Ok((row.avg_cpu, row.avg_memory, row.avg_disk))
    }

    // --- Cost Optimization v24 ---

    pub async fn get_cost_optimization(
        &self,
        task_id: Uuid,
    ) -> Result<ScheduledTaskCostOptimization, sqlx::Error> {
        let analytics = self.get_task_analytics(task_id).await?;
        let (avg_cpu, avg_memory, _avg_disk) = self.get_avg_resource_usage(task_id).await?;

        let cost_per_cpu_hour = 0.005; // $0.005 per CPU-hour
        let cost_per_gb_hour = 0.01; // $0.01 per GB-hour

        let avg_duration_hours = analytics.average_execution_time_ms / 3600000.0;
        let executions_per_month = analytics.total_runs as f64 * 30.0; // assume daily
        let cpu_cost = avg_cpu / 100.0 * avg_duration_hours * executions_per_month * cost_per_cpu_hour;
        let memory_cost = (avg_memory as f64 / 1_073_741_824.0) * avg_duration_hours * executions_per_month * cost_per_gb_hour;
        let estimated_monthly_cost_usd = cpu_cost + memory_cost;

        let recommendation = if estimated_monthly_cost_usd > 10.0 {
            "High monthly cost. Consider reducing execution frequency or optimizing resource usage.".to_string()
        } else if avg_cpu > 80.0 {
            "High CPU usage detected. Consider parallelizing or optimizing task logic.".to_string()
        } else if avg_memory > 1_073_741_824 {
            "High memory usage. Consider streaming processing or reducing batch sizes.".to_string()
        } else {
            "Resource usage is within acceptable limits.".to_string()
        };

        Ok(ScheduledTaskCostOptimization {
            task_id,
            total_executions: analytics.total_runs,
            avg_duration_ms: analytics.average_execution_time_ms,
            avg_cpu_percent: avg_cpu,
            avg_memory_bytes: avg_memory,
            estimated_monthly_cost_usd,
            recommendation,
        })
    }

    // --- Capacity Planning v24 ---

    pub async fn get_capacity_plan(
        &self,
        task_id: Uuid,
    ) -> Result<ScheduledTaskCapacityPlan, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct PeakRow {
            peak_cpu: f64,
            peak_memory: i64,
        }

        let peak = sqlx::query_as::<_, PeakRow>(
            r#"SELECT
                COALESCE(MAX(cpu_usage_percent), 0) as peak_cpu,
                COALESCE(MAX(memory_usage_bytes), 0)::BIGINT as peak_memory
             FROM scheduled_task_resource_usage_v21 WHERE task_id = $1"#,
        )
        .bind(task_id)
        .fetch_one(&self.pool)
        .await?;

        let (avg_cpu, avg_memory, _) = self.get_avg_resource_usage(task_id).await?;
        let analytics = self.get_task_analytics(task_id).await?;

        let execution_frequency_per_day = analytics.total_runs as f64 / 30.0;
        let projected_cpu_next_30d = avg_cpu * 1.1; // 10% growth assumption
        let projected_memory_next_30d = (avg_memory as f64 * 1.1) as i64;

        let scaling_recommendation = if projected_cpu_next_30d > 90.0 {
            "CPU projected to exceed 90%. Scale up or optimize task.".to_string()
        } else if projected_memory_next_30d > 2_147_483_648 {
            "Memory projected to exceed 2GB. Consider memory optimization.".to_string()
        } else if execution_frequency_per_day > 100.0 {
            "High execution frequency. Consider batching or consolidating tasks.".to_string()
        } else {
            "Current capacity is sufficient for projected growth.".to_string()
        };

        Ok(ScheduledTaskCapacityPlan {
            task_id,
            avg_cpu_percent: avg_cpu,
            peak_cpu_percent: peak.peak_cpu,
            avg_memory_bytes: avg_memory,
            peak_memory_bytes: peak.peak_memory,
            execution_frequency_per_day,
            projected_cpu_next_30d,
            projected_memory_next_30d,
            scaling_recommendation,
        })
    }
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
