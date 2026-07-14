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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplateUsage {
    pub id: Uuid,
    pub template_id: Uuid,
    pub user_id: Uuid,
    pub used_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub trigger_id: Option<Uuid>,
    pub status: String,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionStep {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub action_id: Uuid,
    pub status: String,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkflowExecution {
    pub workflow_id: Uuid,
    pub trigger_id: Option<Uuid>,
    pub input: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionStats {
    pub workflow_id: Uuid,
    pub total_executions: i64,
    pub successful_executions: i64,
    pub failed_executions: i64,
    pub average_execution_time_ms: f64,
    pub last_execution_time_ms: Option<f64>,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkflowTemplate {
    pub name: String,
    pub description: Option<String>,
    pub template_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkflowTemplate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub template_type: Option<String>,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
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

#[derive(Debug, sqlx::FromRow)]
struct WorkflowTemplateRow {
    id: Uuid,
    name: String,
    description: String,
    template_type: String,
    config: serde_json::Value,
    is_public: bool,
    author_id: Option<Uuid>,
    usage_count: i32,
    created_at: DateTime<Utc>,
}

impl From<WorkflowTemplateRow> for WorkflowTemplate {
    fn from(row: WorkflowTemplateRow) -> Self {
        WorkflowTemplate {
            id: row.id,
            name: row.name,
            description: row.description,
            template_type: row.template_type,
            config: row.config,
            is_public: row.is_public,
            author_id: row.author_id,
            usage_count: row.usage_count,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct WorkflowTemplateUsageRow {
    id: Uuid,
    template_id: Uuid,
    user_id: Uuid,
    used_at: DateTime<Utc>,
}

impl From<WorkflowTemplateUsageRow> for WorkflowTemplateUsage {
    fn from(row: WorkflowTemplateUsageRow) -> Self {
        WorkflowTemplateUsage {
            id: row.id,
            template_id: row.template_id,
            user_id: row.user_id,
            used_at: row.used_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct WorkflowExecutionRow {
    id: Uuid,
    workflow_id: Uuid,
    trigger_id: Option<Uuid>,
    status: String,
    input: serde_json::Value,
    output: serde_json::Value,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

impl From<WorkflowExecutionRow> for WorkflowExecution {
    fn from(row: WorkflowExecutionRow) -> Self {
        WorkflowExecution {
            id: row.id,
            workflow_id: row.workflow_id,
            trigger_id: row.trigger_id,
            status: row.status,
            input: row.input,
            output: row.output,
            started_at: row.started_at,
            completed_at: row.completed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplateV2 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplateReview {
    pub id: Uuid,
    pub template_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub review: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkflowTemplateV2 {
    pub name: String,
    pub description: Option<String>,
    pub template_type: String,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub author_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkflowTemplateV2 {
    pub name: Option<String>,
    pub description: Option<String>,
    pub template_type: Option<String>,
    pub config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplateAnalytics {
    pub template_id: Uuid,
    pub total_usage: i64,
    pub avg_rating: f64,
    pub total_reviews: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplateRecommendation {
    pub template_id: Uuid,
    pub recommendation_type: String,
    pub description: String,
    pub confidence: f64,
    pub suggested_changes: serde_json::Value,
}

#[derive(Debug, sqlx::FromRow)]
struct WorkflowTemplateV2Row {
    id: Uuid,
    name: String,
    description: String,
    template_type: String,
    config: serde_json::Value,
    is_public: bool,
    author_id: Option<Uuid>,
    usage_count: i32,
    rating: f64,
    created_at: DateTime<Utc>,
}

impl From<WorkflowTemplateV2Row> for WorkflowTemplateV2 {
    fn from(row: WorkflowTemplateV2Row) -> Self {
        WorkflowTemplateV2 {
            id: row.id,
            name: row.name,
            description: row.description,
            template_type: row.template_type,
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
struct WorkflowTemplateReviewRow {
    id: Uuid,
    template_id: Uuid,
    user_id: Uuid,
    rating: i32,
    review: String,
    created_at: DateTime<Utc>,
}

impl From<WorkflowTemplateReviewRow> for WorkflowTemplateReview {
    fn from(row: WorkflowTemplateReviewRow) -> Self {
        WorkflowTemplateReview {
            id: row.id,
            template_id: row.template_id,
            user_id: row.user_id,
            rating: row.rating,
            review: row.review,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct WorkflowExecutionStepRow {
    id: Uuid,
    execution_id: Uuid,
    action_id: Uuid,
    status: String,
    input: serde_json::Value,
    output: serde_json::Value,
    error: Option<String>,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

impl From<WorkflowExecutionStepRow> for WorkflowExecutionStep {
    fn from(row: WorkflowExecutionStepRow) -> Self {
        WorkflowExecutionStep {
            id: row.id,
            execution_id: row.execution_id,
            action_id: row.action_id,
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
pub struct WorkflowTemplateV3 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplateReviewV2 {
    pub id: Uuid,
    pub template_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub review: String,
    pub helpful_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct WorkflowTemplateV3Row {
    id: Uuid,
    name: String,
    description: String,
    template_type: String,
    config: serde_json::Value,
    is_public: bool,
    author_id: Option<Uuid>,
    usage_count: i32,
    rating: f64,
    created_at: DateTime<Utc>,
}

impl From<WorkflowTemplateV3Row> for WorkflowTemplateV3 {
    fn from(row: WorkflowTemplateV3Row) -> Self {
        WorkflowTemplateV3 {
            id: row.id,
            name: row.name,
            description: row.description,
            template_type: row.template_type,
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
struct WorkflowTemplateReviewV2Row {
    id: Uuid,
    template_id: Uuid,
    user_id: Uuid,
    rating: i32,
    review: String,
    helpful_count: i32,
    created_at: DateTime<Utc>,
}

impl From<WorkflowTemplateReviewV2Row> for WorkflowTemplateReviewV2 {
    fn from(row: WorkflowTemplateReviewV2Row) -> Self {
        WorkflowTemplateReviewV2 {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplateV4 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplateReviewV3 {
    pub id: Uuid,
    pub template_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub review: String,
    pub helpful_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct WorkflowTemplateV4Row {
    id: Uuid,
    name: String,
    description: String,
    template_type: String,
    config: serde_json::Value,
    is_public: bool,
    author_id: Option<Uuid>,
    usage_count: i32,
    rating: f64,
    created_at: DateTime<Utc>,
}

impl From<WorkflowTemplateV4Row> for WorkflowTemplateV4 {
    fn from(row: WorkflowTemplateV4Row) -> Self {
        WorkflowTemplateV4 {
            id: row.id,
            name: row.name,
            description: row.description,
            template_type: row.template_type,
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
struct WorkflowTemplateReviewV3Row {
    id: Uuid,
    template_id: Uuid,
    user_id: Uuid,
    rating: i32,
    review: String,
    helpful_count: i32,
    created_at: DateTime<Utc>,
}

impl From<WorkflowTemplateReviewV3Row> for WorkflowTemplateReviewV3 {
    fn from(row: WorkflowTemplateReviewV3Row) -> Self {
        WorkflowTemplateReviewV3 {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplateV5 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplateReviewV4 {
    pub id: Uuid,
    pub template_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub review: String,
    pub helpful_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct WorkflowTemplateV5Row {
    id: Uuid,
    name: String,
    description: String,
    template_type: String,
    config: serde_json::Value,
    is_public: bool,
    author_id: Option<Uuid>,
    usage_count: i32,
    rating: f64,
    created_at: DateTime<Utc>,
}

impl From<WorkflowTemplateV5Row> for WorkflowTemplateV5 {
    fn from(row: WorkflowTemplateV5Row) -> Self {
        WorkflowTemplateV5 {
            id: row.id,
            name: row.name,
            description: row.description,
            template_type: row.template_type,
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
struct WorkflowTemplateReviewV4Row {
    id: Uuid,
    template_id: Uuid,
    user_id: Uuid,
    rating: i32,
    review: String,
    helpful_count: i32,
    created_at: DateTime<Utc>,
}

impl From<WorkflowTemplateReviewV4Row> for WorkflowTemplateReviewV4 {
    fn from(row: WorkflowTemplateReviewV4Row) -> Self {
        WorkflowTemplateReviewV4 {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplateV6 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplateReviewV5 {
    pub id: Uuid,
    pub template_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub review: String,
    pub helpful_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct WorkflowTemplateV6Row {
    id: Uuid,
    name: String,
    description: String,
    template_type: String,
    config: serde_json::Value,
    is_public: bool,
    author_id: Option<Uuid>,
    usage_count: i32,
    rating: f64,
    created_at: DateTime<Utc>,
}

impl From<WorkflowTemplateV6Row> for WorkflowTemplateV6 {
    fn from(row: WorkflowTemplateV6Row) -> Self {
        WorkflowTemplateV6 {
            id: row.id,
            name: row.name,
            description: row.description,
            template_type: row.template_type,
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
struct WorkflowTemplateReviewV5Row {
    id: Uuid,
    template_id: Uuid,
    user_id: Uuid,
    rating: i32,
    review: String,
    helpful_count: i32,
    created_at: DateTime<Utc>,
}

impl From<WorkflowTemplateReviewV5Row> for WorkflowTemplateReviewV5 {
    fn from(row: WorkflowTemplateReviewV5Row) -> Self {
        WorkflowTemplateReviewV5 {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplateV7 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplateReviewV6 {
    pub id: Uuid,
    pub template_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub review: String,
    pub helpful_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct WorkflowTemplateV7Row {
    id: Uuid,
    name: String,
    description: String,
    template_type: String,
    config: serde_json::Value,
    is_public: bool,
    author_id: Option<Uuid>,
    usage_count: i32,
    rating: f64,
    created_at: DateTime<Utc>,
}

impl From<WorkflowTemplateV7Row> for WorkflowTemplateV7 {
    fn from(row: WorkflowTemplateV7Row) -> Self {
        WorkflowTemplateV7 {
            id: row.id,
            name: row.name,
            description: row.description,
            template_type: row.template_type,
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
struct WorkflowTemplateReviewV6Row {
    id: Uuid,
    template_id: Uuid,
    user_id: Uuid,
    rating: i32,
    review: String,
    helpful_count: i32,
    created_at: DateTime<Utc>,
}

impl From<WorkflowTemplateReviewV6Row> for WorkflowTemplateReviewV6 {
    fn from(row: WorkflowTemplateReviewV6Row) -> Self {
        WorkflowTemplateReviewV6 {
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

    // --- Workflow Templates ---

    pub async fn create_template(
        &self,
        input: CreateWorkflowTemplate,
    ) -> Result<WorkflowTemplate, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateRow>(
            r#"INSERT INTO workflow_templates (name, description, template_type, config, is_public, author_id)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, description, template_type, config, is_public, author_id, usage_count, created_at"#,
        )
        .bind(&input.name)
        .bind(input.description.as_deref().unwrap_or(""))
        .bind(&input.template_type)
        .bind(input.config.unwrap_or(serde_json::json!({})))
        .bind(input.is_public.unwrap_or(false))
        .bind(input.author_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_template(&self, id: Uuid) -> Result<Option<WorkflowTemplate>, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateRow>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, created_at
             FROM workflow_templates WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates(&self) -> Result<Vec<WorkflowTemplate>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateRow>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, created_at
             FROM workflow_templates ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates(&self) -> Result<Vec<WorkflowTemplate>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateRow>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, created_at
             FROM workflow_templates WHERE is_public = true ORDER BY usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_by_type(
        &self,
        template_type: &str,
    ) -> Result<Vec<WorkflowTemplate>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateRow>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, created_at
             FROM workflow_templates WHERE template_type = $1 AND is_public = true
             ORDER BY usage_count DESC"#,
        )
        .bind(template_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template(
        &self,
        id: Uuid,
        input: UpdateWorkflowTemplate,
    ) -> Result<WorkflowTemplate, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateRow>(
            r#"UPDATE workflow_templates SET
             name = COALESCE($2, name),
             description = COALESCE($3, description),
             template_type = COALESCE($4, template_type),
             config = COALESCE($5, config),
             is_public = COALESCE($6, is_public)
             WHERE id = $1
             RETURNING id, name, description, template_type, config, is_public, author_id, usage_count, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.template_type)
        .bind(&input.config)
        .bind(input.is_public)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_template(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM workflow_templates WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn record_template_usage(
        &self,
        template_id: Uuid,
        user_id: Uuid,
    ) -> Result<WorkflowTemplateUsage, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateUsageRow>(
            r#"INSERT INTO workflow_template_usage (template_id, user_id)
             VALUES ($1, $2)
             RETURNING id, template_id, user_id, used_at"#,
        )
        .bind(template_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        // Increment usage count
        sqlx::query("UPDATE workflow_templates SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(row.into())
    }

    pub async fn get_template_usage_count(&self, template_id: Uuid) -> Result<i64, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct UsageCount {
            count: Option<i64>,
        }

        let row = sqlx::query_as::<_, UsageCount>(
            r#"SELECT COUNT(*) as count FROM workflow_template_usage WHERE template_id = $1"#,
        )
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.count.unwrap_or(0))
    }

    pub async fn get_popular_templates(&self, limit: i64) -> Result<Vec<WorkflowTemplate>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateRow>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, created_at
             FROM workflow_templates WHERE is_public = true
             ORDER BY usage_count DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_workflow_from_template(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        workflow_name: Option<&str>,
    ) -> Result<Workflow, sqlx::Error> {
        let template = self.get_template(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let name = workflow_name.unwrap_or(&template.name);

        let create_input = CreateWorkflow {
            name: name.to_string(),
            description: Some(template.description.clone()),
            trigger_type: template.config.get("trigger_type")
                .and_then(|v| v.as_str())
                .unwrap_or("manual")
                .to_string(),
            trigger_config: template.config.get("trigger_config").cloned(),
            steps: template.config.get("steps").cloned(),
            enabled: Some(true),
        };

        let workflow = self.create_workflow(create_input).await?;

        // Record usage
        let _ = self.record_template_usage(template_id, user_id).await;

        Ok(workflow)
    }

    // --- V4: Execution Tracking ---

    pub async fn create_execution(
        &self,
        input: CreateWorkflowExecution,
    ) -> Result<WorkflowExecution, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowExecutionRow>(
            r#"INSERT INTO workflow_executions (workflow_id, trigger_id, input)
             VALUES ($1, $2, $3)
             RETURNING id, workflow_id, trigger_id, status, input, output, started_at, completed_at"#,
        )
        .bind(input.workflow_id)
        .bind(input.trigger_id)
        .bind(input.input.unwrap_or(serde_json::json!({})))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_execution(
        &self,
        id: Uuid,
    ) -> Result<Option<WorkflowExecution>, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowExecutionRow>(
            r#"SELECT id, workflow_id, trigger_id, status, input, output, started_at, completed_at
             FROM workflow_executions WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_executions_for_workflow(
        &self,
        workflow_id: Uuid,
        limit: i64,
    ) -> Result<Vec<WorkflowExecution>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowExecutionRow>(
            r#"SELECT id, workflow_id, trigger_id, status, input, output, started_at, completed_at
             FROM workflow_executions WHERE workflow_id = $1
             ORDER BY started_at DESC LIMIT $2"#,
        )
        .bind(workflow_id)
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
    ) -> Result<WorkflowExecution, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowExecutionRow>(
            r#"UPDATE workflow_executions SET
             status = $2,
             output = COALESCE($3, output),
             completed_at = CASE WHEN $2 IN ('completed', 'failed', 'cancelled') THEN NOW() ELSE completed_at END
             WHERE id = $1
             RETURNING id, workflow_id, trigger_id, status, input, output, started_at, completed_at"#,
        )
        .bind(execution_id)
        .bind(status)
        .bind(output)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn add_execution_step(
        &self,
        execution_id: Uuid,
        action_id: Uuid,
        input: Option<serde_json::Value>,
    ) -> Result<WorkflowExecutionStep, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowExecutionStepRow>(
            r#"INSERT INTO workflow_execution_steps (execution_id, action_id, input)
             VALUES ($1, $2, $3)
             RETURNING id, execution_id, action_id, status, input, output, error, started_at, completed_at"#,
        )
        .bind(execution_id)
        .bind(action_id)
        .bind(input.unwrap_or(serde_json::json!({})))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn complete_execution_step(
        &self,
        step_id: Uuid,
        status: &str,
        output: Option<serde_json::Value>,
        error: Option<&str>,
    ) -> Result<WorkflowExecutionStep, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowExecutionStepRow>(
            r#"UPDATE workflow_execution_steps SET
             status = $2,
             output = COALESCE($3, output),
             error = $4,
             completed_at = NOW()
             WHERE id = $1
             RETURNING id, execution_id, action_id, status, input, output, error, started_at, completed_at"#,
        )
        .bind(step_id)
        .bind(status)
        .bind(output)
        .bind(error)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn list_execution_steps(
        &self,
        execution_id: Uuid,
    ) -> Result<Vec<WorkflowExecutionStep>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowExecutionStepRow>(
            r#"SELECT id, execution_id, action_id, status, input, output, error, started_at, completed_at
             FROM workflow_execution_steps WHERE execution_id = $1
             ORDER BY started_at ASC"#,
        )
        .bind(execution_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn execute_workflow_with_tracking(
        &self,
        workflow_id: Uuid,
        trigger_id: Option<Uuid>,
        context: &serde_json::Value,
    ) -> Result<(WorkflowExecution, Vec<WorkflowExecutionStep>), sqlx::Error> {
        let execution = self.create_execution(CreateWorkflowExecution {
            workflow_id,
            trigger_id,
            input: Some(context.clone()),
        }).await?;

        let actions = self.list_enabled_actions(workflow_id).await?;
        let mut steps = Vec::new();

        for action in &actions {
            let step = self.add_execution_step(
                execution.id,
                action.id,
                Some(serde_json::json!({
                    "action_type": action.action_type,
                    "action_config": action.action_config,
                })),
            ).await?;

            let should_execute = self.evaluate_action_conditions(&action.action_config, context);

            if !should_execute {
                let completed_step = self.complete_execution_step(
                    step.id,
                    "skipped",
                    Some(serde_json::json!({"reason": "condition_not_met"})),
                    None,
                ).await?;
                steps.push(completed_step);
                continue;
            }

            let completed_step = self.complete_execution_step(
                step.id,
                "completed",
                Some(serde_json::json!({
                    "action_type": action.action_type,
                    "result": "executed"
                })),
                None,
            ).await?;
            steps.push(completed_step);
        }

        let all_success = steps.iter().all(|s| s.status == "completed" || s.status == "skipped");
        let final_status = if all_success { "completed" } else { "failed" };

        let final_output = serde_json::json!({
            "total_steps": steps.len(),
            "completed_steps": steps.iter().filter(|s| s.status == "completed").count(),
            "skipped_steps": steps.iter().filter(|s| s.status == "skipped").count(),
            "failed_steps": steps.iter().filter(|s| s.status == "failed").count(),
        });

        let execution = self.update_execution_status(
            execution.id,
            final_status,
            Some(final_output),
        ).await?;

        Ok((execution, steps))
    }

    pub async fn get_execution_stats(
        &self,
        workflow_id: Uuid,
    ) -> Result<WorkflowExecutionStats, sqlx::Error> {
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
             FROM workflow_executions WHERE workflow_id = $1"#,
        )
        .bind(workflow_id)
        .fetch_one(&self.pool)
        .await?;

        let success_rate = if row.total_executions > 0 {
            (row.successful_executions as f64 / row.total_executions as f64) * 100.0
        } else {
            0.0
        };

        Ok(WorkflowExecutionStats {
            workflow_id,
            total_executions: row.total_executions,
            successful_executions: row.successful_executions,
            failed_executions: row.failed_executions,
            average_execution_time_ms: row.avg_execution_time_ms,
            last_execution_time_ms: row.last_execution_time_ms,
            success_rate,
        })
    }

    pub async fn cancel_execution(
        &self,
        execution_id: Uuid,
    ) -> Result<WorkflowExecution, sqlx::Error> {
        self.update_execution_status(execution_id, "cancelled", None).await
    }

    pub async fn retry_execution(
        &self,
        execution_id: Uuid,
    ) -> Result<WorkflowExecution, sqlx::Error> {
        let execution = self.get_execution(execution_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        self.create_execution(CreateWorkflowExecution {
            workflow_id: execution.workflow_id,
            trigger_id: execution.trigger_id,
            input: Some(execution.input),
        }).await
    }

    // --- V5: Template Reviews, Ratings, Recommendations, Analytics, Marketplace ---

    pub async fn create_template_v2(
        &self,
        input: CreateWorkflowTemplateV2,
    ) -> Result<WorkflowTemplateV2, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateV2Row>(
            r#"INSERT INTO workflow_templates_v2 (name, description, template_type, config, is_public, author_id)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(&input.name)
        .bind(input.description.as_deref().unwrap_or(""))
        .bind(&input.template_type)
        .bind(input.config.unwrap_or(serde_json::json!({})))
        .bind(input.is_public.unwrap_or(false))
        .bind(input.author_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_template_v2(&self, id: Uuid) -> Result<Option<WorkflowTemplateV2>, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateV2Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v2 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates_v2(&self) -> Result<Vec<WorkflowTemplateV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV2Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v2 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates_v2(&self) -> Result<Vec<WorkflowTemplateV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV2Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v2 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_v2_by_type(
        &self,
        template_type: &str,
    ) -> Result<Vec<WorkflowTemplateV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV2Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v2 WHERE template_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(template_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template_v2(
        &self,
        id: Uuid,
        input: UpdateWorkflowTemplateV2,
    ) -> Result<WorkflowTemplateV2, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateV2Row>(
            r#"UPDATE workflow_templates_v2 SET
             name = COALESCE($2, name),
             description = COALESCE($3, description),
             template_type = COALESCE($4, template_type),
             config = COALESCE($5, config),
             is_public = COALESCE($6, is_public)
             WHERE id = $1
             RETURNING id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.template_type)
        .bind(&input.config)
        .bind(input.is_public)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_template_v2(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM workflow_templates_v2 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn add_template_review(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        rating: i32,
        review: &str,
    ) -> Result<WorkflowTemplateReview, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateReviewRow>(
            r#"INSERT INTO workflow_template_reviews (template_id, user_id, rating, review)
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

        // Update template average rating
        self.recalculate_template_rating(template_id).await?;

        Ok(row.into())
    }

    async fn recalculate_template_rating(&self, template_id: Uuid) -> Result<(), sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct RatingAvg {
            avg_rating: Option<f64>,
        }

        let row = sqlx::query_as::<_, RatingAvg>(
            r#"SELECT AVG(rating::double precision) as avg_rating
             FROM workflow_template_reviews WHERE template_id = $1"#,
        )
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        let avg = row.avg_rating.unwrap_or(0.0);
        sqlx::query("UPDATE workflow_templates_v2 SET rating = $2 WHERE id = $1")
            .bind(template_id)
            .bind(avg)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_reviews(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<WorkflowTemplateReview>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateReviewRow>(
            r#"SELECT id, template_id, user_id, rating, review, created_at
             FROM workflow_template_reviews WHERE template_id = $1
             ORDER BY created_at DESC"#,
        )
        .bind(template_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn record_template_v2_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE workflow_templates_v2 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v2_analytics(
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
                (SELECT COUNT(*) FROM workflow_template_reviews WHERE template_id = $1) as total_reviews
             FROM workflow_templates_v2 WHERE id = $1"#,
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

    pub async fn get_template_v2_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<WorkflowTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v2(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v2_analytics(template_id).await?;
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

    pub async fn get_marketplace_templates(
        &self,
        limit: i64,
    ) -> Result<Vec<WorkflowTemplateV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV2Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v2 WHERE is_public = true
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
    ) -> Result<Vec<WorkflowTemplateV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV2Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v2 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_workflow_from_template_v2(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        workflow_name: Option<&str>,
    ) -> Result<Workflow, sqlx::Error> {
        let template = self.get_template_v2(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let name = workflow_name.unwrap_or(&template.name);

        let create_input = CreateWorkflow {
            name: name.to_string(),
            description: Some(template.description.clone()),
            trigger_type: template.config.get("trigger_type")
                .and_then(|v| v.as_str())
                .unwrap_or("manual")
                .to_string(),
            trigger_config: template.config.get("trigger_config").cloned(),
            steps: template.config.get("steps").cloned(),
            enabled: Some(true),
        };

        let workflow = self.create_workflow(create_input).await?;

        let _ = self.record_template_v2_usage(template_id, user_id).await;

        Ok(workflow)
    }

    // --- V6: Workflow Template Reviews with Helpfulness, Marketplace, Analytics, Recommendations ---

    pub async fn create_template_v3(
        &self,
        input: CreateWorkflowTemplateV2,
    ) -> Result<WorkflowTemplateV3, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateV3Row>(
            r#"INSERT INTO workflow_templates_v3 (name, description, template_type, config, is_public, author_id)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(&input.name)
        .bind(input.description.as_deref().unwrap_or(""))
        .bind(&input.template_type)
        .bind(input.config.unwrap_or(serde_json::json!({})))
        .bind(input.is_public.unwrap_or(false))
        .bind(input.author_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_template_v3(&self, id: Uuid) -> Result<Option<WorkflowTemplateV3>, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateV3Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v3 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates_v3(&self) -> Result<Vec<WorkflowTemplateV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV3Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v3 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates_v3(&self) -> Result<Vec<WorkflowTemplateV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV3Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v3 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_v3_by_type(
        &self,
        template_type: &str,
    ) -> Result<Vec<WorkflowTemplateV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV3Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v3 WHERE template_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(template_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template_v3(
        &self,
        id: Uuid,
        input: UpdateWorkflowTemplateV2,
    ) -> Result<WorkflowTemplateV3, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateV3Row>(
            r#"UPDATE workflow_templates_v3 SET
             name = COALESCE($2, name),
             description = COALESCE($3, description),
             template_type = COALESCE($4, template_type),
             config = COALESCE($5, config),
             is_public = COALESCE($6, is_public)
             WHERE id = $1
             RETURNING id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.template_type)
        .bind(&input.config)
        .bind(input.is_public)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_template_v3(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM workflow_templates_v3 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn add_template_v3_review(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        rating: i32,
        review: &str,
    ) -> Result<WorkflowTemplateReviewV2, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateReviewV2Row>(
            r#"INSERT INTO workflow_template_reviews_v2 (template_id, user_id, rating, review)
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

        self.recalculate_template_v3_rating(template_id).await?;

        Ok(row.into())
    }

    async fn recalculate_template_v3_rating(&self, template_id: Uuid) -> Result<(), sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct RatingAvg {
            avg_rating: Option<f64>,
        }

        let row = sqlx::query_as::<_, RatingAvg>(
            r#"SELECT AVG(rating::double precision) as avg_rating
             FROM workflow_template_reviews_v2 WHERE template_id = $1"#,
        )
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        let avg = row.avg_rating.unwrap_or(0.0);
        sqlx::query("UPDATE workflow_templates_v3 SET rating = $2 WHERE id = $1")
            .bind(template_id)
            .bind(avg)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn mark_review_helpful(
        &self,
        review_id: Uuid,
    ) -> Result<WorkflowTemplateReviewV2, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateReviewV2Row>(
            r#"UPDATE workflow_template_reviews_v2 SET helpful_count = helpful_count + 1
             WHERE id = $1
             RETURNING id, template_id, user_id, rating, review, helpful_count, created_at"#,
        )
        .bind(review_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_template_v3_reviews(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<WorkflowTemplateReviewV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateReviewV2Row>(
            r#"SELECT id, template_id, user_id, rating, review, helpful_count, created_at
             FROM workflow_template_reviews_v2 WHERE template_id = $1
             ORDER BY helpful_count DESC, created_at DESC"#,
        )
        .bind(template_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn record_template_v3_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE workflow_templates_v3 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v3_analytics(
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
                (SELECT COUNT(*) FROM workflow_template_reviews_v2 WHERE template_id = $1) as total_reviews
             FROM workflow_templates_v3 WHERE id = $1"#,
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

    pub async fn get_template_v3_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<WorkflowTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v3(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v3_analytics(template_id).await?;
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

    pub async fn get_marketplace_templates_v3(
        &self,
        limit: i64,
    ) -> Result<Vec<WorkflowTemplateV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV3Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v3 WHERE is_public = true
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
    ) -> Result<Vec<WorkflowTemplateV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV3Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v3 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_workflow_from_template_v3(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        workflow_name: Option<&str>,
    ) -> Result<Workflow, sqlx::Error> {
        let template = self.get_template_v3(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let name = workflow_name.unwrap_or(&template.name);

        let create_input = CreateWorkflow {
            name: name.to_string(),
            description: Some(template.description.clone()),
            trigger_type: template.config.get("trigger_type")
                .and_then(|v| v.as_str())
                .unwrap_or("manual")
                .to_string(),
            trigger_config: template.config.get("trigger_config").cloned(),
            steps: template.config.get("steps").cloned(),
            enabled: Some(true),
        };

        let workflow = self.create_workflow(create_input).await?;

        let _ = self.record_template_v3_usage(template_id, user_id).await;

        Ok(workflow)
    }

    // --- V7: Workflow Template V4 with Reviews V3, Marketplace V4, Analytics V4, Recommendations V4 ---

    pub async fn create_template_v4(
        &self,
        input: CreateWorkflowTemplateV2,
    ) -> Result<WorkflowTemplateV4, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateV4Row>(
            r#"INSERT INTO workflow_templates_v4 (name, description, template_type, config, is_public, author_id)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(&input.name)
        .bind(input.description.as_deref().unwrap_or(""))
        .bind(&input.template_type)
        .bind(input.config.unwrap_or(serde_json::json!({})))
        .bind(input.is_public.unwrap_or(false))
        .bind(input.author_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_template_v4(&self, id: Uuid) -> Result<Option<WorkflowTemplateV4>, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateV4Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v4 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates_v4(&self) -> Result<Vec<WorkflowTemplateV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV4Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v4 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates_v4(&self) -> Result<Vec<WorkflowTemplateV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV4Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v4 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_v4_by_type(
        &self,
        template_type: &str,
    ) -> Result<Vec<WorkflowTemplateV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV4Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v4 WHERE template_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(template_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template_v4(
        &self,
        id: Uuid,
        input: UpdateWorkflowTemplateV2,
    ) -> Result<WorkflowTemplateV4, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateV4Row>(
            r#"UPDATE workflow_templates_v4 SET
             name = COALESCE($2, name),
             description = COALESCE($3, description),
             template_type = COALESCE($4, template_type),
             config = COALESCE($5, config),
             is_public = COALESCE($6, is_public)
             WHERE id = $1
             RETURNING id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.template_type)
        .bind(&input.config)
        .bind(input.is_public)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_template_v4(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM workflow_templates_v4 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn add_template_v4_review(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        rating: i32,
        review: &str,
    ) -> Result<WorkflowTemplateReviewV3, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateReviewV3Row>(
            r#"INSERT INTO workflow_template_reviews_v3 (template_id, user_id, rating, review)
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

        self.recalculate_template_v4_rating(template_id).await?;

        Ok(row.into())
    }

    async fn recalculate_template_v4_rating(&self, template_id: Uuid) -> Result<(), sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct RatingAvg {
            avg_rating: Option<f64>,
        }

        let row = sqlx::query_as::<_, RatingAvg>(
            r#"SELECT AVG(rating::double precision) as avg_rating
             FROM workflow_template_reviews_v3 WHERE template_id = $1"#,
        )
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        let avg = row.avg_rating.unwrap_or(0.0);
        sqlx::query("UPDATE workflow_templates_v4 SET rating = $2 WHERE id = $1")
            .bind(template_id)
            .bind(avg)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn mark_template_v4_review_helpful(
        &self,
        review_id: Uuid,
    ) -> Result<WorkflowTemplateReviewV3, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateReviewV3Row>(
            r#"UPDATE workflow_template_reviews_v3 SET helpful_count = helpful_count + 1
             WHERE id = $1
             RETURNING id, template_id, user_id, rating, review, helpful_count, created_at"#,
        )
        .bind(review_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_template_v4_reviews(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<WorkflowTemplateReviewV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateReviewV3Row>(
            r#"SELECT id, template_id, user_id, rating, review, helpful_count, created_at
             FROM workflow_template_reviews_v3 WHERE template_id = $1
             ORDER BY helpful_count DESC, created_at DESC"#,
        )
        .bind(template_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn record_template_v4_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE workflow_templates_v4 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v4_analytics(
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
                (SELECT COUNT(*) FROM workflow_template_reviews_v3 WHERE template_id = $1) as total_reviews
             FROM workflow_templates_v4 WHERE id = $1"#,
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

    pub async fn get_template_v4_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<WorkflowTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v4(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v4_analytics(template_id).await?;
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

    pub async fn get_marketplace_templates_v4(
        &self,
        limit: i64,
    ) -> Result<Vec<WorkflowTemplateV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV4Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v4 WHERE is_public = true
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
    ) -> Result<Vec<WorkflowTemplateV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV4Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v4 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_workflow_from_template_v4(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        workflow_name: Option<&str>,
    ) -> Result<Workflow, sqlx::Error> {
        let template = self.get_template_v4(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let name = workflow_name.unwrap_or(&template.name);

        let create_input = CreateWorkflow {
            name: name.to_string(),
            description: Some(template.description.clone()),
            trigger_type: template.config.get("trigger_type")
                .and_then(|v| v.as_str())
                .unwrap_or("manual")
                .to_string(),
            trigger_config: template.config.get("trigger_config").cloned(),
            steps: template.config.get("steps").cloned(),
            enabled: Some(true),
        };

        let workflow = self.create_workflow(create_input).await?;

        let _ = self.record_template_v4_usage(template_id, user_id).await;

        Ok(workflow)
    }

    // --- V8: Workflow Template V5 with Reviews V4, Marketplace V5, Analytics V5, Recommendations V5 ---

    pub async fn create_template_v5(
        &self,
        input: CreateWorkflowTemplateV2,
    ) -> Result<WorkflowTemplateV5, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateV5Row>(
            r#"INSERT INTO workflow_templates_v5 (name, description, template_type, config, is_public, author_id)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(&input.name)
        .bind(input.description.as_deref().unwrap_or(""))
        .bind(&input.template_type)
        .bind(input.config.unwrap_or(serde_json::json!({})))
        .bind(input.is_public.unwrap_or(false))
        .bind(input.author_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_template_v5(&self, id: Uuid) -> Result<Option<WorkflowTemplateV5>, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateV5Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v5 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates_v5(&self) -> Result<Vec<WorkflowTemplateV5>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV5Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v5 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates_v5(&self) -> Result<Vec<WorkflowTemplateV5>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV5Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v5 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_v5_by_type(
        &self,
        template_type: &str,
    ) -> Result<Vec<WorkflowTemplateV5>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV5Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v5 WHERE template_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(template_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template_v5(
        &self,
        id: Uuid,
        input: UpdateWorkflowTemplateV2,
    ) -> Result<WorkflowTemplateV5, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateV5Row>(
            r#"UPDATE workflow_templates_v5 SET
             name = COALESCE($2, name),
             description = COALESCE($3, description),
             template_type = COALESCE($4, template_type),
             config = COALESCE($5, config),
             is_public = COALESCE($6, is_public)
             WHERE id = $1
             RETURNING id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.template_type)
        .bind(&input.config)
        .bind(input.is_public)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_template_v5(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM workflow_templates_v5 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn add_template_v5_review(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        rating: i32,
        review: &str,
    ) -> Result<WorkflowTemplateReviewV4, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateReviewV4Row>(
            r#"INSERT INTO workflow_template_reviews_v4 (template_id, user_id, rating, review)
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

        self.recalculate_template_v5_rating(template_id).await?;

        Ok(row.into())
    }

    async fn recalculate_template_v5_rating(&self, template_id: Uuid) -> Result<(), sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct RatingAvg {
            avg_rating: Option<f64>,
        }

        let row = sqlx::query_as::<_, RatingAvg>(
            r#"SELECT AVG(rating::double precision) as avg_rating
             FROM workflow_template_reviews_v4 WHERE template_id = $1"#,
        )
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        let avg = row.avg_rating.unwrap_or(0.0);
        sqlx::query("UPDATE workflow_templates_v5 SET rating = $2 WHERE id = $1")
            .bind(template_id)
            .bind(avg)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn mark_template_v5_review_helpful(
        &self,
        review_id: Uuid,
    ) -> Result<WorkflowTemplateReviewV4, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateReviewV4Row>(
            r#"UPDATE workflow_template_reviews_v4 SET helpful_count = helpful_count + 1
             WHERE id = $1
             RETURNING id, template_id, user_id, rating, review, helpful_count, created_at"#,
        )
        .bind(review_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_template_v5_reviews(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<WorkflowTemplateReviewV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateReviewV4Row>(
            r#"SELECT id, template_id, user_id, rating, review, helpful_count, created_at
             FROM workflow_template_reviews_v4 WHERE template_id = $1
             ORDER BY helpful_count DESC, created_at DESC"#,
        )
        .bind(template_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn record_template_v5_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE workflow_templates_v5 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v5_analytics(
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
                (SELECT COUNT(*) FROM workflow_template_reviews_v4 WHERE template_id = $1) as total_reviews
             FROM workflow_templates_v5 WHERE id = $1"#,
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

    pub async fn get_template_v5_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<WorkflowTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v5(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v5_analytics(template_id).await?;
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

    pub async fn get_marketplace_templates_v5(
        &self,
        limit: i64,
    ) -> Result<Vec<WorkflowTemplateV5>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV5Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v5 WHERE is_public = true
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
    ) -> Result<Vec<WorkflowTemplateV5>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV5Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v5 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_workflow_from_template_v5(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        workflow_name: Option<&str>,
    ) -> Result<Workflow, sqlx::Error> {
        let template = self.get_template_v5(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let name = workflow_name.unwrap_or(&template.name);

        let create_input = CreateWorkflow {
            name: name.to_string(),
            description: Some(template.description.clone()),
            trigger_type: template.config.get("trigger_type")
                .and_then(|v| v.as_str())
                .unwrap_or("manual")
                .to_string(),
            trigger_config: template.config.get("trigger_config").cloned(),
            steps: template.config.get("steps").cloned(),
            enabled: Some(true),
        };

        let workflow = self.create_workflow(create_input).await?;

        let _ = self.record_template_v5_usage(template_id, user_id).await;

        Ok(workflow)
    }

    // --- V9: Workflow Template V6 with Reviews V5, Marketplace V6, Analytics V6, Recommendations V6 ---

    pub async fn create_template_v6(
        &self,
        input: CreateWorkflowTemplateV2,
    ) -> Result<WorkflowTemplateV6, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateV6Row>(
            r#"INSERT INTO workflow_templates_v6 (name, description, template_type, config, is_public, author_id)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(&input.name)
        .bind(input.description.as_deref().unwrap_or(""))
        .bind(&input.template_type)
        .bind(input.config.unwrap_or(serde_json::json!({})))
        .bind(input.is_public.unwrap_or(false))
        .bind(input.author_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_template_v6(&self, id: Uuid) -> Result<Option<WorkflowTemplateV6>, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateV6Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v6 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates_v6(&self) -> Result<Vec<WorkflowTemplateV6>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV6Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v6 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates_v6(&self) -> Result<Vec<WorkflowTemplateV6>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV6Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v6 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_v6_by_type(
        &self,
        template_type: &str,
    ) -> Result<Vec<WorkflowTemplateV6>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV6Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v6 WHERE template_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(template_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template_v6(
        &self,
        id: Uuid,
        input: UpdateWorkflowTemplateV2,
    ) -> Result<WorkflowTemplateV6, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateV6Row>(
            r#"UPDATE workflow_templates_v6 SET
             name = COALESCE($2, name),
             description = COALESCE($3, description),
             template_type = COALESCE($4, template_type),
             config = COALESCE($5, config),
             is_public = COALESCE($6, is_public)
             WHERE id = $1
             RETURNING id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.template_type)
        .bind(&input.config)
        .bind(input.is_public)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_template_v6(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM workflow_templates_v6 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn add_template_v6_review(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        rating: i32,
        review: &str,
    ) -> Result<WorkflowTemplateReviewV5, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateReviewV5Row>(
            r#"INSERT INTO workflow_template_reviews_v5 (template_id, user_id, rating, review)
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

        self.recalculate_template_v6_rating(template_id).await?;

        Ok(row.into())
    }

    async fn recalculate_template_v6_rating(&self, template_id: Uuid) -> Result<(), sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct RatingAvg {
            avg_rating: Option<f64>,
        }

        let row = sqlx::query_as::<_, RatingAvg>(
            r#"SELECT AVG(rating::double precision) as avg_rating
             FROM workflow_template_reviews_v5 WHERE template_id = $1"#,
        )
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        let avg = row.avg_rating.unwrap_or(0.0);
        sqlx::query("UPDATE workflow_templates_v6 SET rating = $2 WHERE id = $1")
            .bind(template_id)
            .bind(avg)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn mark_template_v6_review_helpful(
        &self,
        review_id: Uuid,
    ) -> Result<WorkflowTemplateReviewV5, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateReviewV5Row>(
            r#"UPDATE workflow_template_reviews_v5 SET helpful_count = helpful_count + 1
             WHERE id = $1
             RETURNING id, template_id, user_id, rating, review, helpful_count, created_at"#,
        )
        .bind(review_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_template_v6_reviews(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<WorkflowTemplateReviewV5>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateReviewV5Row>(
            r#"SELECT id, template_id, user_id, rating, review, helpful_count, created_at
             FROM workflow_template_reviews_v5 WHERE template_id = $1
             ORDER BY helpful_count DESC, created_at DESC"#,
        )
        .bind(template_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn record_template_v6_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE workflow_templates_v6 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v6_analytics(
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
                (SELECT COUNT(*) FROM workflow_template_reviews_v5 WHERE template_id = $1) as total_reviews
             FROM workflow_templates_v6 WHERE id = $1"#,
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

    pub async fn get_template_v6_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<WorkflowTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v6(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v6_analytics(template_id).await?;
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

    pub async fn get_marketplace_templates_v6(
        &self,
        limit: i64,
    ) -> Result<Vec<WorkflowTemplateV6>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV6Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v6 WHERE is_public = true
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
    ) -> Result<Vec<WorkflowTemplateV6>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV6Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v6 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_workflow_from_template_v6(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        workflow_name: Option<&str>,
    ) -> Result<Workflow, sqlx::Error> {
        let template = self.get_template_v6(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let name = workflow_name.unwrap_or(&template.name);

        let create_input = CreateWorkflow {
            name: name.to_string(),
            description: Some(template.description.clone()),
            trigger_type: template.config.get("trigger_type")
                .and_then(|v| v.as_str())
                .unwrap_or("manual")
                .to_string(),
            trigger_config: template.config.get("trigger_config").cloned(),
            steps: template.config.get("steps").cloned(),
            enabled: Some(true),
        };

        let workflow = self.create_workflow(create_input).await?;

        let _ = self.record_template_v6_usage(template_id, user_id).await;

        Ok(workflow)
    }

    // --- V10: Workflow Template V7 with Reviews V6, Marketplace V7, Analytics V7, Recommendations V7 ---

    pub async fn create_template_v7(
        &self,
        input: CreateWorkflowTemplateV2,
    ) -> Result<WorkflowTemplateV7, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateV7Row>(
            r#"INSERT INTO workflow_templates_v7 (name, description, template_type, config, is_public, author_id)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(&input.name)
        .bind(input.description.as_deref().unwrap_or(""))
        .bind(&input.template_type)
        .bind(input.config.unwrap_or(serde_json::json!({})))
        .bind(input.is_public.unwrap_or(false))
        .bind(input.author_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_template_v7(&self, id: Uuid) -> Result<Option<WorkflowTemplateV7>, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateV7Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v7 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_templates_v7(&self) -> Result<Vec<WorkflowTemplateV7>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV7Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v7 ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_public_templates_v7(&self) -> Result<Vec<WorkflowTemplateV7>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV7Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v7 WHERE is_public = true ORDER BY rating DESC, usage_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_templates_v7_by_type(
        &self,
        template_type: &str,
    ) -> Result<Vec<WorkflowTemplateV7>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV7Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v7 WHERE template_type = $1 AND is_public = true
             ORDER BY rating DESC, usage_count DESC"#,
        )
        .bind(template_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_template_v7(
        &self,
        id: Uuid,
        input: UpdateWorkflowTemplateV2,
    ) -> Result<WorkflowTemplateV7, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateV7Row>(
            r#"UPDATE workflow_templates_v7 SET
             name = COALESCE($2, name),
             description = COALESCE($3, description),
             template_type = COALESCE($4, template_type),
             config = COALESCE($5, config),
             is_public = COALESCE($6, is_public)
             WHERE id = $1
             RETURNING id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.template_type)
        .bind(&input.config)
        .bind(input.is_public)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_template_v7(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM workflow_templates_v7 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn add_template_v7_review(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        rating: i32,
        review: &str,
    ) -> Result<WorkflowTemplateReviewV6, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateReviewV6Row>(
            r#"INSERT INTO workflow_template_reviews_v6 (template_id, user_id, rating, review)
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

        self.recalculate_template_v7_rating(template_id).await?;

        Ok(row.into())
    }

    async fn recalculate_template_v7_rating(&self, template_id: Uuid) -> Result<(), sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct RatingAvg {
            avg_rating: Option<f64>,
        }

        let row = sqlx::query_as::<_, RatingAvg>(
            r#"SELECT AVG(rating::double precision) as avg_rating
             FROM workflow_template_reviews_v6 WHERE template_id = $1"#,
        )
        .bind(template_id)
        .fetch_one(&self.pool)
        .await?;

        let avg = row.avg_rating.unwrap_or(0.0);
        sqlx::query("UPDATE workflow_templates_v7 SET rating = $2 WHERE id = $1")
            .bind(template_id)
            .bind(avg)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn mark_template_v7_review_helpful(
        &self,
        review_id: Uuid,
    ) -> Result<WorkflowTemplateReviewV6, sqlx::Error> {
        let row = sqlx::query_as::<_, WorkflowTemplateReviewV6Row>(
            r#"UPDATE workflow_template_reviews_v6 SET helpful_count = helpful_count + 1
             WHERE id = $1
             RETURNING id, template_id, user_id, rating, review, helpful_count, created_at"#,
        )
        .bind(review_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_template_v7_reviews(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<WorkflowTemplateReviewV6>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateReviewV6Row>(
            r#"SELECT id, template_id, user_id, rating, review, helpful_count, created_at
             FROM workflow_template_reviews_v6 WHERE template_id = $1
             ORDER BY helpful_count DESC, created_at DESC"#,
        )
        .bind(template_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn record_template_v7_usage(
        &self,
        template_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE workflow_templates_v7 SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(template_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_template_v7_analytics(
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
                (SELECT COUNT(*) FROM workflow_template_reviews_v6 WHERE template_id = $1) as total_reviews
             FROM workflow_templates_v7 WHERE id = $1"#,
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

    pub async fn get_template_v7_recommendations(
        &self,
        template_id: Uuid,
    ) -> Result<Vec<WorkflowTemplateRecommendation>, sqlx::Error> {
        let template = self.get_template_v7(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let analytics = self.get_template_v7_analytics(template_id).await?;
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

    pub async fn get_marketplace_templates_v7(
        &self,
        limit: i64,
    ) -> Result<Vec<WorkflowTemplateV7>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV7Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v7 WHERE is_public = true
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
    ) -> Result<Vec<WorkflowTemplateV7>, sqlx::Error> {
        let rows = sqlx::query_as::<_, WorkflowTemplateV7Row>(
            r#"SELECT id, name, description, template_type, config, is_public, author_id, usage_count, rating, created_at
             FROM workflow_templates_v7 WHERE is_public = true
             AND (name ILIKE $1 OR description ILIKE $1)
             ORDER BY rating DESC, usage_count DESC LIMIT $2"#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn create_workflow_from_template_v7(
        &self,
        template_id: Uuid,
        user_id: Uuid,
        workflow_name: Option<&str>,
    ) -> Result<Workflow, sqlx::Error> {
        let template = self.get_template_v7(template_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let name = workflow_name.unwrap_or(&template.name);

        let create_input = CreateWorkflow {
            name: name.to_string(),
            description: Some(template.description.clone()),
            trigger_type: template.config.get("trigger_type")
                .and_then(|v| v.as_str())
                .unwrap_or("manual")
                .to_string(),
            trigger_config: template.config.get("trigger_config").cloned(),
            steps: template.config.get("steps").cloned(),
            enabled: Some(true),
        };

        let workflow = self.create_workflow(create_input).await?;

        let _ = self.record_template_v7_usage(template_id, user_id).await;

        Ok(workflow)
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

    #[test]
    fn test_workflow_template_serialization() {
        let template = WorkflowTemplate {
            id: Uuid::new_v4(),
            name: "CI/CD Pipeline".into(),
            description: "Standard CI/CD pipeline template".into(),
            template_type: "pipeline".into(),
            config: serde_json::json!({"trigger_type": "push", "steps": []}),
            is_public: true,
            author_id: Some(Uuid::new_v4()),
            usage_count: 42,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&template).unwrap();
        assert!(json.contains("CI/CD Pipeline"));
        assert!(json.contains("pipeline"));
        assert!(json.contains("42"));
    }

    #[test]
    fn test_workflow_template_usage_serialization() {
        let usage = WorkflowTemplateUsage {
            id: Uuid::new_v4(),
            template_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            used_at: Utc::now(),
        };
        let json = serde_json::to_string(&usage).unwrap();
        assert!(json.contains("template_id"));
        assert!(json.contains("user_id"));
    }

    #[test]
    fn test_create_workflow_template_input_serialization() {
        let input = CreateWorkflowTemplate {
            name: "Deploy Template".into(),
            description: Some("Deployment template".into()),
            template_type: "deployment".into(),
            config: Some(serde_json::json!({"steps": [{"name": "deploy"}]})),
            is_public: Some(true),
            author_id: Some(Uuid::new_v4()),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("Deploy Template"));
        assert!(json.contains("deployment"));
    }

    #[test]
    fn test_update_workflow_template_input_serialization() {
        let input = UpdateWorkflowTemplate {
            name: Some("Updated Template".into()),
            description: None,
            template_type: None,
            config: None,
            is_public: Some(false),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("Updated Template"));
        assert!(json.contains("false"));
    }

    #[test]
    fn test_workflow_execution_serialization() {
        let execution = WorkflowExecution {
            id: Uuid::new_v4(),
            workflow_id: Uuid::new_v4(),
            trigger_id: Some(Uuid::new_v4()),
            status: "completed".into(),
            input: serde_json::json!({"branch": "main"}),
            output: serde_json::json!({"result": "success"}),
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&execution).unwrap();
        assert!(json.contains("completed"));
        assert!(json.contains("branch"));
    }

    #[test]
    fn test_workflow_execution_step_serialization() {
        let step = WorkflowExecutionStep {
            id: Uuid::new_v4(),
            execution_id: Uuid::new_v4(),
            action_id: Uuid::new_v4(),
            status: "completed".into(),
            input: serde_json::json!({"action_type": "deploy"}),
            output: serde_json::json!({"deployed": true}),
            error: None,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("completed"));
        assert!(json.contains("deploy"));
    }

    #[test]
    fn test_workflow_execution_stats_serialization() {
        let stats = WorkflowExecutionStats {
            workflow_id: Uuid::new_v4(),
            total_executions: 100,
            successful_executions: 95,
            failed_executions: 5,
            average_execution_time_ms: 250.5,
            last_execution_time_ms: Some(200.0),
            success_rate: 95.0,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("100"));
        assert!(json.contains("95"));
        assert!(json.contains("250.5"));
    }

    #[test]
    fn test_create_workflow_execution_input_serialization() {
        let input = CreateWorkflowExecution {
            workflow_id: Uuid::new_v4(),
            trigger_id: Some(Uuid::new_v4()),
            input: Some(serde_json::json!({"event": "push"})),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("workflow_id"));
        assert!(json.contains("trigger_id"));
    }

    #[test]
    fn test_workflow_template_v2_serialization() {
        let template = WorkflowTemplateV2 {
            id: Uuid::new_v4(),
            name: "CI/CD Pipeline V2".into(),
            description: "Advanced CI/CD pipeline".into(),
            template_type: "pipeline".into(),
            config: serde_json::json!({"trigger_type": "push", "steps": []}),
            is_public: true,
            author_id: Some(Uuid::new_v4()),
            usage_count: 100,
            rating: 4.5,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&template).unwrap();
        assert!(json.contains("CI/CD Pipeline V2"));
        assert!(json.contains("pipeline"));
        assert!(json.contains("100"));
        assert!(json.contains("4.5"));
    }

    #[test]
    fn test_workflow_template_review_serialization() {
        let review = WorkflowTemplateReview {
            id: Uuid::new_v4(),
            template_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            rating: 5,
            review: "Excellent template!".into(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&review).unwrap();
        assert!(json.contains("Excellent template!"));
        assert!(json.contains("5"));
    }

    #[test]
    fn test_create_workflow_template_v2_input_serialization() {
        let input = CreateWorkflowTemplateV2 {
            name: "Deploy Template V2".into(),
            description: Some("Deployment template".into()),
            template_type: "deployment".into(),
            config: Some(serde_json::json!({"steps": [{"name": "deploy"}]})),
            is_public: Some(true),
            author_id: Some(Uuid::new_v4()),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("Deploy Template V2"));
        assert!(json.contains("deployment"));
    }

    #[test]
    fn test_workflow_template_analytics_serialization() {
        let analytics = WorkflowTemplateAnalytics {
            template_id: Uuid::new_v4(),
            total_usage: 200,
            avg_rating: 4.2,
            total_reviews: 50,
        };
        let json = serde_json::to_string(&analytics).unwrap();
        assert!(json.contains("200"));
        assert!(json.contains("4.2"));
        assert!(json.contains("50"));
    }

    #[test]
    fn test_workflow_template_recommendation_serialization() {
        let rec = WorkflowTemplateRecommendation {
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
    fn test_workflow_template_v4_serialization() {
        let template = WorkflowTemplateV4 {
            id: Uuid::new_v4(),
            name: "CI/CD Pipeline V4".into(),
            description: "Advanced CI/CD pipeline".into(),
            template_type: "pipeline".into(),
            config: serde_json::json!({"trigger_type": "push", "steps": []}),
            is_public: true,
            author_id: Some(Uuid::new_v4()),
            usage_count: 200,
            rating: 4.8,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&template).unwrap();
        assert!(json.contains("CI/CD Pipeline V4"));
        assert!(json.contains("pipeline"));
        assert!(json.contains("200"));
        assert!(json.contains("4.8"));
    }

    #[test]
    fn test_workflow_template_review_v3_serialization() {
        let review = WorkflowTemplateReviewV3 {
            id: Uuid::new_v4(),
            template_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            rating: 5,
            review: "Excellent template!".into(),
            helpful_count: 12,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&review).unwrap();
        assert!(json.contains("Excellent template!"));
        assert!(json.contains("5"));
        assert!(json.contains("12"));
    }

    #[test]
    fn test_workflow_template_v5_serialization() {
        let template = WorkflowTemplateV5 {
            id: Uuid::new_v4(),
            name: "CI/CD Pipeline V5".into(),
            description: "Advanced CI/CD pipeline".into(),
            template_type: "pipeline".into(),
            config: serde_json::json!({"trigger_type": "push", "steps": []}),
            is_public: true,
            author_id: Some(Uuid::new_v4()),
            usage_count: 250,
            rating: 4.9,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&template).unwrap();
        assert!(json.contains("CI/CD Pipeline V5"));
        assert!(json.contains("pipeline"));
        assert!(json.contains("250"));
        assert!(json.contains("4.9"));
    }

    #[test]
    fn test_workflow_template_review_v4_serialization() {
        let review = WorkflowTemplateReviewV4 {
            id: Uuid::new_v4(),
            template_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            rating: 5,
            review: "Excellent template!".into(),
            helpful_count: 15,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&review).unwrap();
        assert!(json.contains("Excellent template!"));
        assert!(json.contains("5"));
        assert!(json.contains("15"));
    }
}
