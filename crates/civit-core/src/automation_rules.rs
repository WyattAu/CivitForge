#![forbid(unsafe_code)]

//! Automation rules for CivitForge.
//!
//! Provides rule CRUD, priority-based execution, complex condition evaluation,
//! action chaining, execution history, and rule testing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRule {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: serde_json::Value,
    pub actions: serde_json::Value,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRuleV2 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: serde_json::Value,
    pub actions: serde_json::Value,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleExecutionRecord {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub status: String,
    pub matched_conditions: Vec<String>,
    pub failed_conditions: Vec<String>,
    pub actions_executed: Vec<String>,
    pub error: Option<String>,
    pub executed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAutomationRule {
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAutomationRuleV2 {
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<serde_json::Value>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAutomationRule {
    pub name: Option<String>,
    pub trigger_type: Option<String>,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAutomationRuleV2 {
    pub name: Option<String>,
    pub trigger_type: Option<String>,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<serde_json::Value>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleTestResult {
    pub rule_id: Uuid,
    pub matched: bool,
    pub conditions_met: Vec<String>,
    pub conditions_failed: Vec<String>,
    pub actions_executed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRuleV3 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: serde_json::Value,
    pub actions: serde_json::Value,
    pub priority: i32,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub run_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAutomationRuleV3 {
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<serde_json::Value>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAutomationRuleV3 {
    pub name: Option<String>,
    pub trigger_type: Option<String>,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<serde_json::Value>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulePerformanceMetrics {
    pub rule_id: Uuid,
    pub total_runs: i64,
    pub successful_runs: i64,
    pub failed_runs: i64,
    pub average_execution_time_ms: f64,
    pub last_execution_time_ms: Option<f64>,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRuleV4 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: serde_json::Value,
    pub actions: serde_json::Value,
    pub priority: i32,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub run_count: i32,
    pub success_rate: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAutomationRuleV4 {
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<serde_json::Value>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAutomationRuleV4 {
    pub name: Option<String>,
    pub trigger_type: Option<String>,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<serde_json::Value>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleRecommendation {
    pub rule_id: Uuid,
    pub recommendation_type: String,
    pub description: String,
    pub confidence: f64,
    pub suggested_changes: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRuleV5 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: serde_json::Value,
    pub actions: serde_json::Value,
    pub priority: i32,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub run_count: i32,
    pub success_rate: f64,
    pub avg_execution_time_ms: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAutomationRuleV5 {
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<serde_json::Value>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAutomationRuleV5 {
    pub name: Option<String>,
    pub trigger_type: Option<String>,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<serde_json::Value>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, sqlx::FromRow)]
struct AutomationRuleV5Row {
    id: Uuid,
    repo_id: Uuid,
    name: String,
    trigger_type: String,
    conditions: serde_json::Value,
    actions: serde_json::Value,
    priority: i32,
    enabled: bool,
    last_run_at: Option<DateTime<Utc>>,
    run_count: i32,
    success_rate: f64,
    avg_execution_time_ms: i32,
    created_at: DateTime<Utc>,
}

impl From<AutomationRuleV5Row> for AutomationRuleV5 {
    fn from(row: AutomationRuleV5Row) -> Self {
        AutomationRuleV5 {
            id: row.id,
            repo_id: row.repo_id,
            name: row.name,
            trigger_type: row.trigger_type,
            conditions: row.conditions,
            actions: row.actions,
            priority: row.priority,
            enabled: row.enabled,
            last_run_at: row.last_run_at,
            run_count: row.run_count,
            success_rate: row.success_rate,
            avg_execution_time_ms: row.avg_execution_time_ms,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRuleV6 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: serde_json::Value,
    pub actions: serde_json::Value,
    pub priority: i32,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub run_count: i32,
    pub success_rate: f64,
    pub avg_execution_time_ms: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAutomationRuleV6 {
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<serde_json::Value>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAutomationRuleV6 {
    pub name: Option<String>,
    pub trigger_type: Option<String>,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<serde_json::Value>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, sqlx::FromRow)]
struct AutomationRuleV6Row {
    id: Uuid,
    repo_id: Uuid,
    name: String,
    trigger_type: String,
    conditions: serde_json::Value,
    actions: serde_json::Value,
    priority: i32,
    enabled: bool,
    last_run_at: Option<DateTime<Utc>>,
    run_count: i32,
    success_rate: f64,
    avg_execution_time_ms: i32,
    created_at: DateTime<Utc>,
}

impl From<AutomationRuleV6Row> for AutomationRuleV6 {
    fn from(row: AutomationRuleV6Row) -> Self {
        AutomationRuleV6 {
            id: row.id,
            repo_id: row.repo_id,
            name: row.name,
            trigger_type: row.trigger_type,
            conditions: row.conditions,
            actions: row.actions,
            priority: row.priority,
            enabled: row.enabled,
            last_run_at: row.last_run_at,
            run_count: row.run_count,
            success_rate: row.success_rate,
            avg_execution_time_ms: row.avg_execution_time_ms,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRuleV7 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: serde_json::Value,
    pub actions: serde_json::Value,
    pub priority: i32,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub run_count: i32,
    pub success_rate: f64,
    pub avg_execution_time_ms: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAutomationRuleV7 {
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<serde_json::Value>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAutomationRuleV7 {
    pub name: Option<String>,
    pub trigger_type: Option<String>,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<serde_json::Value>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, sqlx::FromRow)]
struct AutomationRuleV7Row {
    id: Uuid,
    repo_id: Uuid,
    name: String,
    trigger_type: String,
    conditions: serde_json::Value,
    actions: serde_json::Value,
    priority: i32,
    enabled: bool,
    last_run_at: Option<DateTime<Utc>>,
    run_count: i32,
    success_rate: f64,
    avg_execution_time_ms: i32,
    created_at: DateTime<Utc>,
}

impl From<AutomationRuleV7Row> for AutomationRuleV7 {
    fn from(row: AutomationRuleV7Row) -> Self {
        AutomationRuleV7 {
            id: row.id,
            repo_id: row.repo_id,
            name: row.name,
            trigger_type: row.trigger_type,
            conditions: row.conditions,
            actions: row.actions,
            priority: row.priority,
            enabled: row.enabled,
            last_run_at: row.last_run_at,
            run_count: row.run_count,
            success_rate: row.success_rate,
            avg_execution_time_ms: row.avg_execution_time_ms,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRuleV8 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: serde_json::Value,
    pub actions: serde_json::Value,
    pub priority: i32,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub run_count: i32,
    pub success_rate: f64,
    pub avg_execution_time_ms: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct AutomationRuleV8Row {
    id: Uuid,
    repo_id: Uuid,
    name: String,
    trigger_type: String,
    conditions: serde_json::Value,
    actions: serde_json::Value,
    priority: i32,
    enabled: bool,
    last_run_at: Option<DateTime<Utc>>,
    run_count: i32,
    success_rate: f64,
    avg_execution_time_ms: i32,
    created_at: DateTime<Utc>,
}

impl From<AutomationRuleV8Row> for AutomationRuleV8 {
    fn from(row: AutomationRuleV8Row) -> Self {
        AutomationRuleV8 {
            id: row.id,
            repo_id: row.repo_id,
            name: row.name,
            trigger_type: row.trigger_type,
            conditions: row.conditions,
            actions: row.actions,
            priority: row.priority,
            enabled: row.enabled,
            last_run_at: row.last_run_at,
            run_count: row.run_count,
            success_rate: row.success_rate,
            avg_execution_time_ms: row.avg_execution_time_ms,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct AutomationRuleV4Row {
    id: Uuid,
    repo_id: Uuid,
    name: String,
    trigger_type: String,
    conditions: serde_json::Value,
    actions: serde_json::Value,
    priority: i32,
    enabled: bool,
    last_run_at: Option<DateTime<Utc>>,
    run_count: i32,
    success_rate: f64,
    created_at: DateTime<Utc>,
}

impl From<AutomationRuleV4Row> for AutomationRuleV4 {
    fn from(row: AutomationRuleV4Row) -> Self {
        AutomationRuleV4 {
            id: row.id,
            repo_id: row.repo_id,
            name: row.name,
            trigger_type: row.trigger_type,
            conditions: row.conditions,
            actions: row.actions,
            priority: row.priority,
            enabled: row.enabled,
            last_run_at: row.last_run_at,
            run_count: row.run_count,
            success_rate: row.success_rate,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct AutomationRuleRow {
    id: Uuid,
    repo_id: Uuid,
    name: String,
    trigger_type: String,
    conditions: serde_json::Value,
    actions: serde_json::Value,
    enabled: bool,
    created_at: DateTime<Utc>,
}

impl From<AutomationRuleRow> for AutomationRule {
    fn from(row: AutomationRuleRow) -> Self {
        AutomationRule {
            id: row.id,
            repo_id: row.repo_id,
            name: row.name,
            trigger_type: row.trigger_type,
            conditions: row.conditions,
            actions: row.actions,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct AutomationRuleV2Row {
    id: Uuid,
    repo_id: Uuid,
    name: String,
    trigger_type: String,
    conditions: serde_json::Value,
    actions: serde_json::Value,
    priority: i32,
    enabled: bool,
    created_at: DateTime<Utc>,
}

impl From<AutomationRuleV2Row> for AutomationRuleV2 {
    fn from(row: AutomationRuleV2Row) -> Self {
        AutomationRuleV2 {
            id: row.id,
            repo_id: row.repo_id,
            name: row.name,
            trigger_type: row.trigger_type,
            conditions: row.conditions,
            actions: row.actions,
            priority: row.priority,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct AutomationRuleV3Row {
    id: Uuid,
    repo_id: Uuid,
    name: String,
    trigger_type: String,
    conditions: serde_json::Value,
    actions: serde_json::Value,
    priority: i32,
    enabled: bool,
    last_run_at: Option<DateTime<Utc>>,
    run_count: i32,
    created_at: DateTime<Utc>,
}

impl From<AutomationRuleV3Row> for AutomationRuleV3 {
    fn from(row: AutomationRuleV3Row) -> Self {
        AutomationRuleV3 {
            id: row.id,
            repo_id: row.repo_id,
            name: row.name,
            trigger_type: row.trigger_type,
            conditions: row.conditions,
            actions: row.actions,
            priority: row.priority,
            enabled: row.enabled,
            last_run_at: row.last_run_at,
            run_count: row.run_count,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct RuleExecutionRecordRow {
    id: Uuid,
    rule_id: Uuid,
    status: String,
    matched_conditions: serde_json::Value,
    failed_conditions: serde_json::Value,
    actions_executed: serde_json::Value,
    error: Option<String>,
    executed_at: DateTime<Utc>,
}

impl From<RuleExecutionRecordRow> for RuleExecutionRecord {
    fn from(row: RuleExecutionRecordRow) -> Self {
        let matched = row
            .matched_conditions
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let failed = row
            .failed_conditions
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let executed = row
            .actions_executed
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        RuleExecutionRecord {
            id: row.id,
            rule_id: row.rule_id,
            status: row.status,
            matched_conditions: matched,
            failed_conditions: failed,
            actions_executed: executed,
            error: row.error,
            executed_at: row.executed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRuleV9 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: serde_json::Value,
    pub actions: serde_json::Value,
    pub priority: i32,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub run_count: i32,
    pub success_rate: f64,
    pub avg_execution_time_ms: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAutomationRuleV9 {
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<serde_json::Value>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAutomationRuleV9 {
    pub name: Option<String>,
    pub trigger_type: Option<String>,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<serde_json::Value>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, sqlx::FromRow)]
struct AutomationRuleV9Row {
    id: Uuid,
    repo_id: Uuid,
    name: String,
    trigger_type: String,
    conditions: serde_json::Value,
    actions: serde_json::Value,
    priority: i32,
    enabled: bool,
    last_run_at: Option<DateTime<Utc>>,
    run_count: i32,
    success_rate: f64,
    avg_execution_time_ms: i32,
    created_at: DateTime<Utc>,
}

impl From<AutomationRuleV9Row> for AutomationRuleV9 {
    fn from(row: AutomationRuleV9Row) -> Self {
        AutomationRuleV9 {
            id: row.id,
            repo_id: row.repo_id,
            name: row.name,
            trigger_type: row.trigger_type,
            conditions: row.conditions,
            actions: row.actions,
            priority: row.priority,
            enabled: row.enabled,
            last_run_at: row.last_run_at,
            run_count: row.run_count,
            success_rate: row.success_rate,
            avg_execution_time_ms: row.avg_execution_time_ms,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRuleV10 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: serde_json::Value,
    pub actions: serde_json::Value,
    pub priority: i32,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub run_count: i32,
    pub success_rate: f64,
    pub avg_execution_time_ms: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAutomationRuleV10 {
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<serde_json::Value>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAutomationRuleV10 {
    pub name: Option<String>,
    pub trigger_type: Option<String>,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<serde_json::Value>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, sqlx::FromRow)]
struct AutomationRuleV10Row {
    id: Uuid,
    repo_id: Uuid,
    name: String,
    trigger_type: String,
    conditions: serde_json::Value,
    actions: serde_json::Value,
    priority: i32,
    enabled: bool,
    last_run_at: Option<DateTime<Utc>>,
    run_count: i32,
    success_rate: f64,
    avg_execution_time_ms: i32,
    created_at: DateTime<Utc>,
}

impl From<AutomationRuleV10Row> for AutomationRuleV10 {
    fn from(row: AutomationRuleV10Row) -> Self {
        AutomationRuleV10 {
            id: row.id,
            repo_id: row.repo_id,
            name: row.name,
            trigger_type: row.trigger_type,
            conditions: row.conditions,
            actions: row.actions,
            priority: row.priority,
            enabled: row.enabled,
            last_run_at: row.last_run_at,
            run_count: row.run_count,
            success_rate: row.success_rate,
            avg_execution_time_ms: row.avg_execution_time_ms,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct AutomationRuleV11Row {
    id: Uuid,
    repo_id: Uuid,
    name: String,
    trigger_type: String,
    conditions: serde_json::Value,
    actions: serde_json::Value,
    priority: i32,
    enabled: bool,
    last_run_at: Option<DateTime<Utc>>,
    run_count: i32,
    success_rate: f64,
    avg_execution_time_ms: i32,
    created_at: DateTime<Utc>,
}

impl From<AutomationRuleV11Row> for AutomationRuleV11 {
    fn from(row: AutomationRuleV11Row) -> Self {
        AutomationRuleV11 {
            id: row.id,
            repo_id: row.repo_id,
            name: row.name,
            trigger_type: row.trigger_type,
            conditions: row.conditions,
            actions: row.actions,
            priority: row.priority,
            enabled: row.enabled,
            last_run_at: row.last_run_at,
            run_count: row.run_count,
            success_rate: row.success_rate,
            avg_execution_time_ms: row.avg_execution_time_ms,
            created_at: row.created_at,
        }
    }
}

pub struct AutomationRuleService {
    pool: PgPool,
}

impl AutomationRuleService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_rule(
        &self,
        input: CreateAutomationRule,
    ) -> Result<AutomationRule, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleRow>(
            r#"INSERT INTO automation_rules (repo_id, name, trigger_type, conditions, actions, enabled)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, repo_id, name, trigger_type, conditions, actions, enabled, created_at"#,
        )
        .bind(input.repo_id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(input.conditions.unwrap_or(serde_json::json!({})))
        .bind(input.actions.unwrap_or(serde_json::json!([])))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_rule(&self, id: Uuid) -> Result<Option<AutomationRule>, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleRow>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, enabled, created_at
             FROM automation_rules WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_rules_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRule>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleRow>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, enabled, created_at
             FROM automation_rules WHERE repo_id = $1 ORDER BY created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_enabled_rules_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRule>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleRow>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, enabled, created_at
             FROM automation_rules WHERE repo_id = $1 AND enabled = true
             ORDER BY created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_rule(
        &self,
        id: Uuid,
        input: UpdateAutomationRule,
    ) -> Result<AutomationRule, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleRow>(
            r#"UPDATE automation_rules SET
             name = COALESCE($2, name),
             trigger_type = COALESCE($3, trigger_type),
             conditions = COALESCE($4, conditions),
             actions = COALESCE($5, actions),
             enabled = COALESCE($6, enabled)
             WHERE id = $1
             RETURNING id, repo_id, name, trigger_type, conditions, actions, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(&input.conditions)
        .bind(&input.actions)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_rule(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM automation_rules WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn evaluate_conditions(
        &self,
        rule_id: Uuid,
        context: &serde_json::Value,
    ) -> Result<bool, sqlx::Error> {
        let rule = self.get_rule(rule_id).await?;
        let rule = match rule {
            Some(r) => r,
            None => return Ok(false),
        };

        self.evaluate_condition_set(&rule.conditions, context)
    }

    fn evaluate_condition_set(
        &self,
        conditions: &serde_json::Value,
        context: &serde_json::Value,
    ) -> Result<bool, sqlx::Error> {
        let obj = conditions.as_object();
        let obj = match obj {
            Some(c) => c,
            None => return Ok(true),
        };

        if obj.is_empty() {
            return Ok(true);
        }

        for (key, expected) in obj {
            let actual = context.get(key);
            match actual {
                Some(val) if val == expected => {}
                _ => return Ok(false),
            }
        }

        Ok(true)
    }

    pub async fn test_rule(
        &self,
        rule_id: Uuid,
        context: &serde_json::Value,
    ) -> Result<RuleTestResult, sqlx::Error> {
        let rule = self.get_rule(rule_id).await?.ok_or_else(|| {
            sqlx::Error::RowNotFound
        })?;

        let conditions = rule.conditions.as_object();
        let mut conditions_met = Vec::new();
        let mut conditions_failed = Vec::new();

        if let Some(conds) = conditions {
            for (key, expected) in conds {
                let actual = context.get(key);
                match actual {
                    Some(val) if val == expected => {
                        conditions_met.push(key.clone());
                    }
                    _ => {
                        conditions_failed.push(key.clone());
                    }
                }
            }
        }

        let matched = conditions_failed.is_empty();

        let actions = rule.actions.as_array();
        let actions_executed: Vec<String> = actions
            .map(|arr| {
                arr.iter()
                    .filter_map(|a| a.get("type").and_then(|v| v.as_str()).map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(RuleTestResult {
            rule_id,
            matched,
            conditions_met,
            conditions_failed,
            actions_executed,
        })
    }

    pub async fn execute_actions(
        &self,
        rule_id: Uuid,
        context: &serde_json::Value,
    ) -> Result<Vec<String>, sqlx::Error> {
        let rule = self.get_rule(rule_id).await?.ok_or_else(|| {
            sqlx::Error::RowNotFound
        })?;

        let actions = rule.actions.as_array();
        let mut executed = Vec::new();

        if let Some(action_list) = actions {
            for action in action_list {
                let action_type = action
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                executed.push(action_type.to_string());
            }
        }

        let _ = context;
        Ok(executed)
    }

    // --- V2: Priority-based execution ---

    pub async fn create_rule_v2(
        &self,
        input: CreateAutomationRuleV2,
    ) -> Result<AutomationRuleV2, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV2Row>(
            r#"INSERT INTO automation_rules_v2 (repo_id, name, trigger_type, conditions, actions, priority, enabled)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, repo_id, name, trigger_type, conditions, actions, priority, enabled, created_at"#,
        )
        .bind(input.repo_id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(input.conditions.unwrap_or(serde_json::json!({})))
        .bind(input.actions.unwrap_or(serde_json::json!([])))
        .bind(input.priority.unwrap_or(0))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_rule_v2(&self, id: Uuid) -> Result<Option<AutomationRuleV2>, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV2Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, created_at
             FROM automation_rules_v2 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_rules_v2_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRuleV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV2Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, created_at
             FROM automation_rules_v2 WHERE repo_id = $1 ORDER BY priority DESC, created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_enabled_rules_v2_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRuleV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV2Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, created_at
             FROM automation_rules_v2 WHERE repo_id = $1 AND enabled = true
             ORDER BY priority DESC, created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_rule_v2(
        &self,
        id: Uuid,
        input: UpdateAutomationRuleV2,
    ) -> Result<AutomationRuleV2, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV2Row>(
            r#"UPDATE automation_rules_v2 SET
             name = COALESCE($2, name),
             trigger_type = COALESCE($3, trigger_type),
             conditions = COALESCE($4, conditions),
             actions = COALESCE($5, actions),
             priority = COALESCE($6, priority),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, repo_id, name, trigger_type, conditions, actions, priority, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(&input.conditions)
        .bind(&input.actions)
        .bind(input.priority)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_rule_v2(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM automation_rules_v2 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn execute_rule_v2(
        &self,
        rule_id: Uuid,
        context: &serde_json::Value,
    ) -> Result<RuleExecutionRecord, sqlx::Error> {
        let rule = self.get_rule_v2(rule_id).await?.ok_or_else(|| {
            sqlx::Error::RowNotFound
        })?;

        let mut matched_conditions = Vec::new();
        let mut failed_conditions = Vec::new();

        if let Some(obj) = rule.conditions.as_object() {
            for (key, expected) in obj {
                match context.get(key) {
                    Some(val) if val == expected => {
                        matched_conditions.push(key.clone());
                    }
                    _ => {
                        failed_conditions.push(key.clone());
                    }
                }
            }
        }

        let all_matched = failed_conditions.is_empty();

        let mut actions_executed = Vec::new();
        if all_matched {
            if let Some(action_list) = rule.actions.as_array() {
                for action in action_list {
                    if let Some(action_type) = action.get("type").and_then(|v| v.as_str()) {
                        actions_executed.push(action_type.to_string());
                    }
                }
            }
        }

        let status = if all_matched { "matched" } else { "not_matched" };

        self.record_execution(
            rule_id,
            status,
            &matched_conditions,
            &failed_conditions,
            &actions_executed,
            None,
        )
        .await
    }

    pub async fn execute_action_chain_v2(
        &self,
        rule_id: Uuid,
        context: &serde_json::Value,
    ) -> Result<Vec<String>, sqlx::Error> {
        let rule = self.get_rule_v2(rule_id).await?.ok_or_else(|| {
            sqlx::Error::RowNotFound
        })?;

        let mut executed = Vec::new();

        if let Some(action_list) = rule.actions.as_array() {
            for action in action_list {
                let action_type = action
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                let skip = action
                    .get("condition")
                    .and_then(|c| c.as_object())
                    .map(|cond_obj| {
                        for (key, expected) in cond_obj {
                            match context.get(key) {
                                Some(val) if val == expected => {}
                                _ => return true,
                            }
                        }
                        false
                    })
                    .unwrap_or(false);

                if skip {
                    continue;
                }

                executed.push(action_type.to_string());
            }
        }

        Ok(executed)
    }

    async fn record_execution(
        &self,
        rule_id: Uuid,
        status: &str,
        matched: &[String],
        failed: &[String],
        actions: &[String],
        error: Option<&str>,
    ) -> Result<RuleExecutionRecord, sqlx::Error> {
        let matched_json = serde_json::to_value(matched).unwrap_or(serde_json::json!([]));
        let failed_json = serde_json::to_value(failed).unwrap_or(serde_json::json!([]));
        let actions_json = serde_json::to_value(actions).unwrap_or(serde_json::json!([]));

        let row = sqlx::query_as::<_, RuleExecutionRecordRow>(
            r#"INSERT INTO rule_execution_history (rule_id, status, matched_conditions, failed_conditions, actions_executed, error)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, rule_id, status, matched_conditions, failed_conditions, actions_executed, error, executed_at"#,
        )
        .bind(rule_id)
        .bind(status)
        .bind(matched_json)
        .bind(failed_json)
        .bind(actions_json)
        .bind(error)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_execution_history(
        &self,
        rule_id: Uuid,
        limit: i64,
    ) -> Result<Vec<RuleExecutionRecord>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RuleExecutionRecordRow>(
            r#"SELECT id, rule_id, status, matched_conditions, failed_conditions, actions_executed, error, executed_at
             FROM rule_execution_history WHERE rule_id = $1 ORDER BY executed_at DESC LIMIT $2"#,
        )
        .bind(rule_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_execution_history_for_repo(
        &self,
        repo_id: Uuid,
        limit: i64,
    ) -> Result<Vec<RuleExecutionRecord>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RuleExecutionRecordRow>(
            r#"SELECT reh.id, reh.rule_id, reh.status, reh.matched_conditions, reh.failed_conditions,
                    reh.actions_executed, reh.error, reh.executed_at
             FROM rule_execution_history reh
             JOIN automation_rules_v2 ar ON ar.id = reh.rule_id
             WHERE ar.repo_id = $1
             ORDER BY reh.executed_at DESC LIMIT $2"#,
        )
        .bind(repo_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    // --- V3: Run count tracking and performance analytics ---

    pub async fn create_rule_v3(
        &self,
        input: CreateAutomationRuleV3,
    ) -> Result<AutomationRuleV3, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV3Row>(
            r#"INSERT INTO automation_rules_v3 (repo_id, name, trigger_type, conditions, actions, priority, enabled)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, created_at"#,
        )
        .bind(input.repo_id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(input.conditions.unwrap_or(serde_json::json!({})))
        .bind(input.actions.unwrap_or(serde_json::json!([])))
        .bind(input.priority.unwrap_or(0))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_rule_v3(&self, id: Uuid) -> Result<Option<AutomationRuleV3>, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV3Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, created_at
             FROM automation_rules_v3 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_rules_v3_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRuleV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV3Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, created_at
             FROM automation_rules_v3 WHERE repo_id = $1 ORDER BY priority DESC, created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_enabled_rules_v3_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRuleV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV3Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, created_at
             FROM automation_rules_v3 WHERE repo_id = $1 AND enabled = true
             ORDER BY priority DESC, created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_rule_v3(
        &self,
        id: Uuid,
        input: UpdateAutomationRuleV3,
    ) -> Result<AutomationRuleV3, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV3Row>(
            r#"UPDATE automation_rules_v3 SET
             name = COALESCE($2, name),
             trigger_type = COALESCE($3, trigger_type),
             conditions = COALESCE($4, conditions),
             actions = COALESCE($5, actions),
             priority = COALESCE($6, priority),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(&input.conditions)
        .bind(&input.actions)
        .bind(input.priority)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_rule_v3(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM automation_rules_v3 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn execute_rule_v3(
        &self,
        rule_id: Uuid,
        context: &serde_json::Value,
    ) -> Result<RuleExecutionRecord, sqlx::Error> {
        let rule = self.get_rule_v3(rule_id).await?.ok_or_else(|| {
            sqlx::Error::RowNotFound
        })?;

        let mut matched_conditions = Vec::new();
        let mut failed_conditions = Vec::new();

        if let Some(obj) = rule.conditions.as_object() {
            for (key, expected) in obj {
                match context.get(key) {
                    Some(val) if val == expected => {
                        matched_conditions.push(key.clone());
                    }
                    _ => {
                        failed_conditions.push(key.clone());
                    }
                }
            }
        }

        let all_matched = failed_conditions.is_empty();

        let mut actions_executed = Vec::new();
        if all_matched {
            if let Some(action_list) = rule.actions.as_array() {
                for action in action_list {
                    if let Some(action_type) = action.get("type").and_then(|v| v.as_str()) {
                        actions_executed.push(action_type.to_string());
                    }
                }
            }
        }

        let status = if all_matched { "matched" } else { "not_matched" };

        // Update run count and last_run_at
        sqlx::query(
            r#"UPDATE automation_rules_v3 SET run_count = run_count + 1, last_run_at = NOW() WHERE id = $1"#,
        )
        .bind(rule_id)
        .execute(&self.pool)
        .await?;

        self.record_execution(
            rule_id,
            status,
            &matched_conditions,
            &failed_conditions,
            &actions_executed,
            None,
        )
        .await
    }

    pub async fn get_rule_performance_metrics(
        &self,
        rule_id: Uuid,
    ) -> Result<RulePerformanceMetrics, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct MetricsRow {
            total_runs: i64,
            successful_runs: i64,
            failed_runs: i64,
            avg_execution_time_ms: f64,
            last_execution_time_ms: Option<f64>,
        }

        let row = sqlx::query_as::<_, MetricsRow>(
            r#"SELECT
                COUNT(*) as total_runs,
                COUNT(*) FILTER (WHERE status = 'matched') as successful_runs,
                COUNT(*) FILTER (WHERE status = 'not_matched') as failed_runs,
                COALESCE(AVG(EXTRACT(EPOCH FROM (executed_at - executed_at)) * 1000), 0) as avg_execution_time_ms,
                MAX(CASE WHEN executed_at IS NOT NULL THEN EXTRACT(EPOCH FROM (executed_at - executed_at)) * 1000 END) as last_execution_time_ms
             FROM rule_execution_history WHERE rule_id = $1"#,
        )
        .bind(rule_id)
        .fetch_one(&self.pool)
        .await?;

        let success_rate = if row.total_runs > 0 {
            (row.successful_runs as f64 / row.total_runs as f64) * 100.0
        } else {
            0.0
        };

        Ok(RulePerformanceMetrics {
            rule_id,
            total_runs: row.total_runs,
            successful_runs: row.successful_runs,
            failed_runs: row.failed_runs,
            average_execution_time_ms: row.avg_execution_time_ms,
            last_execution_time_ms: row.last_execution_time_ms,
            success_rate,
        })
    }

    pub async fn get_rules_with_most_runs(
        &self,
        repo_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AutomationRuleV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV3Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, created_at
             FROM automation_rules_v3 WHERE repo_id = $1
             ORDER BY run_count DESC LIMIT $2"#,
        )
        .bind(repo_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn optimize_rule(
        &self,
        rule_id: Uuid,
    ) -> Result<serde_json::Value, sqlx::Error> {
        let metrics = self.get_rule_performance_metrics(rule_id).await?;

        let suggestions = serde_json::json!({
            "rule_id": rule_id,
            "total_runs": metrics.total_runs,
            "success_rate": metrics.success_rate,
            "suggestions": if metrics.success_rate < 50.0 {
                vec!["Consider reviewing conditions - low success rate"]
            } else if metrics.total_runs == 0 {
                vec!["Rule has never been triggered - consider enabling or adjusting trigger"]
            } else {
                vec!["Rule is performing well"]
            }
        });

        Ok(suggestions)
    }

    // --- V4: Success rate tracking and performance analytics ---

    pub async fn create_rule_v4(
        &self,
        input: CreateAutomationRuleV4,
    ) -> Result<AutomationRuleV4, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV4Row>(
            r#"INSERT INTO automation_rules_v4 (repo_id, name, trigger_type, conditions, actions, priority, enabled)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, created_at"#,
        )
        .bind(input.repo_id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(input.conditions.unwrap_or(serde_json::json!({})))
        .bind(input.actions.unwrap_or(serde_json::json!([])))
        .bind(input.priority.unwrap_or(0))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_rule_v4(&self, id: Uuid) -> Result<Option<AutomationRuleV4>, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV4Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, created_at
             FROM automation_rules_v4 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_rules_v4_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRuleV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV4Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, created_at
             FROM automation_rules_v4 WHERE repo_id = $1 ORDER BY priority DESC, created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_enabled_rules_v4_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRuleV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV4Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, created_at
             FROM automation_rules_v4 WHERE repo_id = $1 AND enabled = true
             ORDER BY priority DESC, created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_rule_v4(
        &self,
        id: Uuid,
        input: UpdateAutomationRuleV4,
    ) -> Result<AutomationRuleV4, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV4Row>(
            r#"UPDATE automation_rules_v4 SET
             name = COALESCE($2, name),
             trigger_type = COALESCE($3, trigger_type),
             conditions = COALESCE($4, conditions),
             actions = COALESCE($5, actions),
             priority = COALESCE($6, priority),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(&input.conditions)
        .bind(&input.actions)
        .bind(input.priority)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_rule_v4(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM automation_rules_v4 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn execute_rule_v4(
        &self,
        rule_id: Uuid,
        context: &serde_json::Value,
    ) -> Result<RuleExecutionRecord, sqlx::Error> {
        let rule = self.get_rule_v4(rule_id).await?.ok_or_else(|| {
            sqlx::Error::RowNotFound
        })?;

        let mut matched_conditions = Vec::new();
        let mut failed_conditions = Vec::new();

        if let Some(obj) = rule.conditions.as_object() {
            for (key, expected) in obj {
                match context.get(key) {
                    Some(val) if val == expected => {
                        matched_conditions.push(key.clone());
                    }
                    _ => {
                        failed_conditions.push(key.clone());
                    }
                }
            }
        }

        let all_matched = failed_conditions.is_empty();

        let mut actions_executed = Vec::new();
        if all_matched {
            if let Some(action_list) = rule.actions.as_array() {
                for action in action_list {
                    if let Some(action_type) = action.get("type").and_then(|v| v.as_str()) {
                        actions_executed.push(action_type.to_string());
                    }
                }
            }
        }

        let status = if all_matched { "matched" } else { "not_matched" };

        // Update run count, last_run_at, and success rate
        let new_success_rate = if all_matched {
            // Weighted average: 90% old rate + 10% new result
            (rule.success_rate * 0.9) + (100.0 * 0.1)
        } else {
            (rule.success_rate * 0.9) + (0.0 * 0.1)
        };

        sqlx::query(
            r#"UPDATE automation_rules_v4 SET
             run_count = run_count + 1,
             last_run_at = NOW(),
             success_rate = $2
             WHERE id = $1"#,
        )
        .bind(rule_id)
        .bind(new_success_rate)
        .execute(&self.pool)
        .await?;

        self.record_execution(
            rule_id,
            status,
            &matched_conditions,
            &failed_conditions,
            &actions_executed,
            None,
        )
        .await
    }

    pub async fn get_rule_v4_performance_metrics(
        &self,
        rule_id: Uuid,
    ) -> Result<RulePerformanceMetrics, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct MetricsRow {
            total_runs: i64,
            successful_runs: i64,
            failed_runs: i64,
            avg_execution_time_ms: f64,
            last_execution_time_ms: Option<f64>,
        }

        let row = sqlx::query_as::<_, MetricsRow>(
            r#"SELECT
                COUNT(*) as total_runs,
                COUNT(*) FILTER (WHERE status = 'matched') as successful_runs,
                COUNT(*) FILTER (WHERE status = 'not_matched') as failed_runs,
                COALESCE(AVG(EXTRACT(EPOCH FROM (executed_at - executed_at)) * 1000), 0) as avg_execution_time_ms,
                MAX(CASE WHEN executed_at IS NOT NULL THEN EXTRACT(EPOCH FROM (executed_at - executed_at)) * 1000 END) as last_execution_time_ms
             FROM rule_execution_history WHERE rule_id = $1"#,
        )
        .bind(rule_id)
        .fetch_one(&self.pool)
        .await?;

        let success_rate = if row.total_runs > 0 {
            (row.successful_runs as f64 / row.total_runs as f64) * 100.0
        } else {
            0.0
        };

        Ok(RulePerformanceMetrics {
            rule_id,
            total_runs: row.total_runs,
            successful_runs: row.successful_runs,
            failed_runs: row.failed_runs,
            average_execution_time_ms: row.avg_execution_time_ms,
            last_execution_time_ms: row.last_execution_time_ms,
            success_rate,
        })
    }

    pub async fn get_rules_with_lowest_success_rate(
        &self,
        repo_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AutomationRuleV4>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV4Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, created_at
             FROM automation_rules_v4 WHERE repo_id = $1 AND run_count > 0
             ORDER BY success_rate ASC LIMIT $2"#,
        )
        .bind(repo_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_rule_recommendations(
        &self,
        rule_id: Uuid,
    ) -> Result<Vec<RuleRecommendation>, sqlx::Error> {
        let rule = self.get_rule_v4(rule_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let metrics = self.get_rule_v4_performance_metrics(rule_id).await?;
        let mut recommendations = Vec::new();

        if metrics.total_runs == 0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "never_triggered".into(),
                description: "Rule has never been triggered. Consider reviewing trigger conditions or enabling the rule.".into(),
                confidence: 0.9,
                suggested_changes: serde_json::json!({
                    "action": "review_trigger",
                    "current_trigger": rule.trigger_type
                }),
            });
        } else if rule.success_rate < 50.0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "low_success_rate".into(),
                description: format!("Rule has a low success rate of {:.1}%. Consider reviewing conditions.", rule.success_rate),
                confidence: 0.85,
                suggested_changes: serde_json::json!({
                    "action": "review_conditions",
                    "current_success_rate": rule.success_rate
                }),
            });
        }

        if rule.run_count > 100 && rule.success_rate > 95.0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "high_performance".into(),
                description: "Rule is performing well with high success rate. Consider promoting to higher priority.".into(),
                confidence: 0.8,
                suggested_changes: serde_json::json!({
                    "action": "increase_priority",
                    "current_priority": rule.priority,
                    "suggested_priority": rule.priority + 5
                }),
            });
        }

        Ok(recommendations)
    }

    // --- V5: Execution Time Tracking & Performance Analytics ---

    pub async fn create_rule_v5(
        &self,
        input: CreateAutomationRuleV5,
    ) -> Result<AutomationRuleV5, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV5Row>(
            r#"INSERT INTO automation_rules_v5 (repo_id, name, trigger_type, conditions, actions, priority, enabled)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at"#,
        )
        .bind(input.repo_id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(input.conditions.unwrap_or(serde_json::json!({})))
        .bind(input.actions.unwrap_or(serde_json::json!([])))
        .bind(input.priority.unwrap_or(0))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_rule_v5(&self, id: Uuid) -> Result<Option<AutomationRuleV5>, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV5Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v5 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_rules_v5_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRuleV5>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV5Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v5 WHERE repo_id = $1 ORDER BY priority DESC, created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_enabled_rules_v5_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRuleV5>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV5Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v5 WHERE repo_id = $1 AND enabled = true
             ORDER BY priority DESC, created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_rule_v5(
        &self,
        id: Uuid,
        input: UpdateAutomationRuleV5,
    ) -> Result<AutomationRuleV5, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV5Row>(
            r#"UPDATE automation_rules_v5 SET
             name = COALESCE($2, name),
             trigger_type = COALESCE($3, trigger_type),
             conditions = COALESCE($4, conditions),
             actions = COALESCE($5, actions),
             priority = COALESCE($6, priority),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(&input.conditions)
        .bind(&input.actions)
        .bind(input.priority)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_rule_v5(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM automation_rules_v5 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn execute_rule_v5(
        &self,
        rule_id: Uuid,
        context: &serde_json::Value,
    ) -> Result<RuleExecutionRecord, sqlx::Error> {
        let rule = self.get_rule_v5(rule_id).await?.ok_or_else(|| {
            sqlx::Error::RowNotFound
        })?;

        let start_time = std::time::Instant::now();

        let mut matched_conditions = Vec::new();
        let mut failed_conditions = Vec::new();

        if let Some(obj) = rule.conditions.as_object() {
            for (key, expected) in obj {
                match context.get(key) {
                    Some(val) if val == expected => {
                        matched_conditions.push(key.clone());
                    }
                    _ => {
                        failed_conditions.push(key.clone());
                    }
                }
            }
        }

        let all_matched = failed_conditions.is_empty();

        let mut actions_executed = Vec::new();
        if all_matched {
            if let Some(action_list) = rule.actions.as_array() {
                for action in action_list {
                    if let Some(action_type) = action.get("type").and_then(|v| v.as_str()) {
                        actions_executed.push(action_type.to_string());
                    }
                }
            }
        }

        let elapsed_ms = start_time.elapsed().as_millis() as i32;
        let status = if all_matched { "matched" } else { "not_matched" };

        // Update run count, last_run_at, success rate, and avg execution time
        let new_success_rate = if all_matched {
            (rule.success_rate * 0.9) + (100.0 * 0.1)
        } else {
            (rule.success_rate * 0.9) + (0.0 * 0.1)
        };

        let new_avg_time = if rule.run_count == 0 {
            elapsed_ms
        } else {
            ((rule.avg_execution_time_ms as f64 * 0.9) + (elapsed_ms as f64 * 0.1)) as i32
        };

        sqlx::query(
            r#"UPDATE automation_rules_v5 SET
             run_count = run_count + 1,
             last_run_at = NOW(),
             success_rate = $2,
             avg_execution_time_ms = $3
             WHERE id = $1"#,
        )
        .bind(rule_id)
        .bind(new_success_rate)
        .bind(new_avg_time)
        .execute(&self.pool)
        .await?;

        self.record_execution(
            rule_id,
            status,
            &matched_conditions,
            &failed_conditions,
            &actions_executed,
            None,
        )
        .await
    }

    pub async fn get_rule_v5_performance_metrics(
        &self,
        rule_id: Uuid,
    ) -> Result<RulePerformanceMetrics, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct MetricsRow {
            total_runs: i64,
            successful_runs: i64,
            failed_runs: i64,
            avg_execution_time_ms: f64,
            last_execution_time_ms: Option<f64>,
        }

        let row = sqlx::query_as::<_, MetricsRow>(
            r#"SELECT
                COUNT(*) as total_runs,
                COUNT(*) FILTER (WHERE status = 'matched') as successful_runs,
                COUNT(*) FILTER (WHERE status = 'not_matched') as failed_runs,
                COALESCE(AVG(EXTRACT(EPOCH FROM (executed_at - executed_at)) * 1000), 0) as avg_execution_time_ms,
                MAX(CASE WHEN executed_at IS NOT NULL THEN EXTRACT(EPOCH FROM (executed_at - executed_at)) * 1000 END) as last_execution_time_ms
             FROM rule_execution_history WHERE rule_id = $1"#,
        )
        .bind(rule_id)
        .fetch_one(&self.pool)
        .await?;

        let success_rate = if row.total_runs > 0 {
            (row.successful_runs as f64 / row.total_runs as f64) * 100.0
        } else {
            0.0
        };

        Ok(RulePerformanceMetrics {
            rule_id,
            total_runs: row.total_runs,
            successful_runs: row.successful_runs,
            failed_runs: row.failed_runs,
            average_execution_time_ms: row.avg_execution_time_ms,
            last_execution_time_ms: row.last_execution_time_ms,
            success_rate,
        })
    }

    pub async fn get_rules_v5_with_slowest_execution(
        &self,
        repo_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AutomationRuleV5>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV5Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v5 WHERE repo_id = $1 AND run_count > 0
             ORDER BY avg_execution_time_ms DESC LIMIT $2"#,
        )
        .bind(repo_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_rule_v5_optimization_suggestions(
        &self,
        rule_id: Uuid,
    ) -> Result<Vec<RuleRecommendation>, sqlx::Error> {
        let rule = self.get_rule_v5(rule_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let metrics = self.get_rule_v5_performance_metrics(rule_id).await?;
        let mut recommendations = Vec::new();

        if rule.avg_execution_time_ms > 5000 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "slow_execution".into(),
                description: format!("Rule takes {:.0}ms on average. Consider optimizing conditions or actions.", rule.avg_execution_time_ms as f64),
                confidence: 0.9,
                suggested_changes: serde_json::json!({
                    "action": "optimize",
                    "current_avg_time_ms": rule.avg_execution_time_ms
                }),
            });
        }

        if metrics.total_runs == 0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "never_triggered".into(),
                description: "Rule has never been triggered. Consider reviewing trigger conditions.".into(),
                confidence: 0.9,
                suggested_changes: serde_json::json!({
                    "action": "review_trigger",
                    "current_trigger": rule.trigger_type
                }),
            });
        } else if rule.success_rate < 50.0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "low_success_rate".into(),
                description: format!("Rule has a low success rate of {:.1}%. Consider reviewing conditions.", rule.success_rate),
                confidence: 0.85,
                suggested_changes: serde_json::json!({
                    "action": "review_conditions",
                    "current_success_rate": rule.success_rate
                }),
            });
        }

        Ok(recommendations)
    }

    // --- V6: Execution Time Tracking, Performance Analytics, Rule Optimization, Rule Recommendations ---

    pub async fn create_rule_v6(
        &self,
        input: CreateAutomationRuleV6,
    ) -> Result<AutomationRuleV6, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV6Row>(
            r#"INSERT INTO automation_rules_v6 (repo_id, name, trigger_type, conditions, actions, priority, enabled)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at"#,
        )
        .bind(input.repo_id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(input.conditions.unwrap_or(serde_json::json!({})))
        .bind(input.actions.unwrap_or(serde_json::json!([])))
        .bind(input.priority.unwrap_or(0))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_rule_v6(&self, id: Uuid) -> Result<Option<AutomationRuleV6>, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV6Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v6 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_rules_v6_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRuleV6>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV6Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v6 WHERE repo_id = $1 ORDER BY priority DESC, created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_enabled_rules_v6_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRuleV6>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV6Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v6 WHERE repo_id = $1 AND enabled = true
             ORDER BY priority DESC, created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_rule_v6(
        &self,
        id: Uuid,
        input: UpdateAutomationRuleV6,
    ) -> Result<AutomationRuleV6, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV6Row>(
            r#"UPDATE automation_rules_v6 SET
             name = COALESCE($2, name),
             trigger_type = COALESCE($3, trigger_type),
             conditions = COALESCE($4, conditions),
             actions = COALESCE($5, actions),
             priority = COALESCE($6, priority),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(&input.conditions)
        .bind(&input.actions)
        .bind(input.priority)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_rule_v6(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM automation_rules_v6 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn execute_rule_v6(
        &self,
        rule_id: Uuid,
        context: &serde_json::Value,
    ) -> Result<RuleExecutionRecord, sqlx::Error> {
        let rule = self.get_rule_v6(rule_id).await?.ok_or_else(|| {
            sqlx::Error::RowNotFound
        })?;

        let start_time = std::time::Instant::now();

        let mut matched_conditions = Vec::new();
        let mut failed_conditions = Vec::new();

        if let Some(obj) = rule.conditions.as_object() {
            for (key, expected) in obj {
                match context.get(key) {
                    Some(val) if val == expected => {
                        matched_conditions.push(key.clone());
                    }
                    _ => {
                        failed_conditions.push(key.clone());
                    }
                }
            }
        }

        let all_matched = failed_conditions.is_empty();

        let mut actions_executed = Vec::new();
        if all_matched {
            if let Some(action_list) = rule.actions.as_array() {
                for action in action_list {
                    if let Some(action_type) = action.get("type").and_then(|v| v.as_str()) {
                        actions_executed.push(action_type.to_string());
                    }
                }
            }
        }

        let elapsed_ms = start_time.elapsed().as_millis() as i32;
        let status = if all_matched { "matched" } else { "not_matched" };

        let new_success_rate = if all_matched {
            (rule.success_rate * 0.9) + (100.0 * 0.1)
        } else {
            (rule.success_rate * 0.9) + (0.0 * 0.1)
        };

        let new_avg_time = if rule.run_count == 0 {
            elapsed_ms
        } else {
            ((rule.avg_execution_time_ms as f64 * 0.9) + (elapsed_ms as f64 * 0.1)) as i32
        };

        sqlx::query(
            r#"UPDATE automation_rules_v6 SET
             run_count = run_count + 1,
             last_run_at = NOW(),
             success_rate = $2,
             avg_execution_time_ms = $3
             WHERE id = $1"#,
        )
        .bind(rule_id)
        .bind(new_success_rate)
        .bind(new_avg_time)
        .execute(&self.pool)
        .await?;

        self.record_execution(
            rule_id,
            status,
            &matched_conditions,
            &failed_conditions,
            &actions_executed,
            None,
        )
        .await
    }

    pub async fn get_rule_v6_performance_metrics(
        &self,
        rule_id: Uuid,
    ) -> Result<RulePerformanceMetrics, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct MetricsRow {
            total_runs: i64,
            successful_runs: i64,
            failed_runs: i64,
            avg_execution_time_ms: f64,
            last_execution_time_ms: Option<f64>,
        }

        let row = sqlx::query_as::<_, MetricsRow>(
            r#"SELECT
                COUNT(*) as total_runs,
                COUNT(*) FILTER (WHERE status = 'matched') as successful_runs,
                COUNT(*) FILTER (WHERE status = 'not_matched') as failed_runs,
                COALESCE(AVG(EXTRACT(EPOCH FROM (executed_at - executed_at)) * 1000), 0) as avg_execution_time_ms,
                MAX(CASE WHEN executed_at IS NOT NULL THEN EXTRACT(EPOCH FROM (executed_at - executed_at)) * 1000 END) as last_execution_time_ms
             FROM rule_execution_history WHERE rule_id = $1"#,
        )
        .bind(rule_id)
        .fetch_one(&self.pool)
        .await?;

        let success_rate = if row.total_runs > 0 {
            (row.successful_runs as f64 / row.total_runs as f64) * 100.0
        } else {
            0.0
        };

        Ok(RulePerformanceMetrics {
            rule_id,
            total_runs: row.total_runs,
            successful_runs: row.successful_runs,
            failed_runs: row.failed_runs,
            average_execution_time_ms: row.avg_execution_time_ms,
            last_execution_time_ms: row.last_execution_time_ms,
            success_rate,
        })
    }

    pub async fn get_rules_v6_with_slowest_execution(
        &self,
        repo_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AutomationRuleV6>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV6Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v6 WHERE repo_id = $1 AND run_count > 0
             ORDER BY avg_execution_time_ms DESC LIMIT $2"#,
        )
        .bind(repo_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_rule_v6_optimization_suggestions(
        &self,
        rule_id: Uuid,
    ) -> Result<Vec<RuleRecommendation>, sqlx::Error> {
        let rule = self.get_rule_v6(rule_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let metrics = self.get_rule_v6_performance_metrics(rule_id).await?;
        let mut recommendations = Vec::new();

        if rule.avg_execution_time_ms > 5000 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "slow_execution".into(),
                description: format!("Rule takes {:.0}ms on average. Consider optimizing conditions or actions.", rule.avg_execution_time_ms as f64),
                confidence: 0.9,
                suggested_changes: serde_json::json!({
                    "action": "optimize",
                    "current_avg_time_ms": rule.avg_execution_time_ms
                }),
            });
        }

        if metrics.total_runs == 0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "never_triggered".into(),
                description: "Rule has never been triggered. Consider reviewing trigger conditions.".into(),
                confidence: 0.9,
                suggested_changes: serde_json::json!({
                    "action": "review_trigger",
                    "current_trigger": rule.trigger_type
                }),
            });
        } else if rule.success_rate < 50.0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "low_success_rate".into(),
                description: format!("Rule has a low success rate of {:.1}%. Consider reviewing conditions.", rule.success_rate),
                confidence: 0.85,
                suggested_changes: serde_json::json!({
                    "action": "review_conditions",
                    "current_success_rate": rule.success_rate
                }),
            });
        }

        if rule.run_count > 100 && rule.success_rate > 95.0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "high_performance".into(),
                description: "Rule is performing well with high success rate. Consider promoting to higher priority.".into(),
                confidence: 0.8,
                suggested_changes: serde_json::json!({
                    "action": "increase_priority",
                    "current_priority": rule.priority,
                    "suggested_priority": rule.priority + 5
                }),
            });
        }

        Ok(recommendations)
    }

    // --- V7: Execution Time Tracking, Performance Analytics, Rule Optimization, Rule Recommendations ---

    pub async fn create_rule_v7(
        &self,
        input: CreateAutomationRuleV7,
    ) -> Result<AutomationRuleV7, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV7Row>(
            r#"INSERT INTO automation_rules_v7 (repo_id, name, trigger_type, conditions, actions, priority, enabled)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at"#,
        )
        .bind(input.repo_id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(input.conditions.unwrap_or(serde_json::json!({})))
        .bind(input.actions.unwrap_or(serde_json::json!([])))
        .bind(input.priority.unwrap_or(0))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_rule_v7(&self, id: Uuid) -> Result<Option<AutomationRuleV7>, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV7Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v7 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_rules_v7_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRuleV7>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV7Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v7 WHERE repo_id = $1 ORDER BY priority DESC, created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_enabled_rules_v7_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRuleV7>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV7Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v7 WHERE repo_id = $1 AND enabled = true
             ORDER BY priority DESC, created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_rule_v7(
        &self,
        id: Uuid,
        input: UpdateAutomationRuleV7,
    ) -> Result<AutomationRuleV7, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV7Row>(
            r#"UPDATE automation_rules_v7 SET
             name = COALESCE($2, name),
             trigger_type = COALESCE($3, trigger_type),
             conditions = COALESCE($4, conditions),
             actions = COALESCE($5, actions),
             priority = COALESCE($6, priority),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(&input.conditions)
        .bind(&input.actions)
        .bind(input.priority)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_rule_v7(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM automation_rules_v7 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn execute_rule_v7(
        &self,
        rule_id: Uuid,
        context: &serde_json::Value,
    ) -> Result<RuleExecutionRecord, sqlx::Error> {
        let rule = self.get_rule_v7(rule_id).await?.ok_or_else(|| {
            sqlx::Error::RowNotFound
        })?;

        let start_time = std::time::Instant::now();

        let mut matched_conditions = Vec::new();
        let mut failed_conditions = Vec::new();

        if let Some(obj) = rule.conditions.as_object() {
            for (key, expected) in obj {
                match context.get(key) {
                    Some(val) if val == expected => {
                        matched_conditions.push(key.clone());
                    }
                    _ => {
                        failed_conditions.push(key.clone());
                    }
                }
            }
        }

        let all_matched = failed_conditions.is_empty();

        let mut actions_executed = Vec::new();
        if all_matched {
            if let Some(action_list) = rule.actions.as_array() {
                for action in action_list {
                    if let Some(action_type) = action.get("type").and_then(|v| v.as_str()) {
                        actions_executed.push(action_type.to_string());
                    }
                }
            }
        }

        let elapsed_ms = start_time.elapsed().as_millis() as i32;
        let status = if all_matched { "matched" } else { "not_matched" };

        let new_success_rate = if all_matched {
            (rule.success_rate * 0.9) + (100.0 * 0.1)
        } else {
            (rule.success_rate * 0.9) + (0.0 * 0.1)
        };

        let new_avg_time = if rule.run_count == 0 {
            elapsed_ms
        } else {
            ((rule.avg_execution_time_ms as f64 * 0.9) + (elapsed_ms as f64 * 0.1)) as i32
        };

        sqlx::query(
            r#"UPDATE automation_rules_v7 SET
             run_count = run_count + 1,
             last_run_at = NOW(),
             success_rate = $2,
             avg_execution_time_ms = $3
             WHERE id = $1"#,
        )
        .bind(rule_id)
        .bind(new_success_rate)
        .bind(new_avg_time)
        .execute(&self.pool)
        .await?;

        self.record_execution(
            rule_id,
            status,
            &matched_conditions,
            &failed_conditions,
            &actions_executed,
            None,
        )
        .await
    }

    pub async fn get_rule_v7_performance_metrics(
        &self,
        rule_id: Uuid,
    ) -> Result<RulePerformanceMetrics, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct MetricsRow {
            total_runs: i64,
            successful_runs: i64,
            failed_runs: i64,
            avg_execution_time_ms: f64,
            last_execution_time_ms: Option<f64>,
        }

        let row = sqlx::query_as::<_, MetricsRow>(
            r#"SELECT
                COUNT(*) as total_runs,
                COUNT(*) FILTER (WHERE status = 'matched') as successful_runs,
                COUNT(*) FILTER (WHERE status = 'not_matched') as failed_runs,
                COALESCE(AVG(EXTRACT(EPOCH FROM (executed_at - executed_at)) * 1000), 0) as avg_execution_time_ms,
                MAX(CASE WHEN executed_at IS NOT NULL THEN EXTRACT(EPOCH FROM (executed_at - executed_at)) * 1000 END) as last_execution_time_ms
             FROM rule_execution_history WHERE rule_id = $1"#,
        )
        .bind(rule_id)
        .fetch_one(&self.pool)
        .await?;

        let success_rate = if row.total_runs > 0 {
            (row.successful_runs as f64 / row.total_runs as f64) * 100.0
        } else {
            0.0
        };

        Ok(RulePerformanceMetrics {
            rule_id,
            total_runs: row.total_runs,
            successful_runs: row.successful_runs,
            failed_runs: row.failed_runs,
            average_execution_time_ms: row.avg_execution_time_ms,
            last_execution_time_ms: row.last_execution_time_ms,
            success_rate,
        })
    }

    pub async fn get_rules_v7_with_slowest_execution(
        &self,
        repo_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AutomationRuleV7>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV7Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v7 WHERE repo_id = $1 AND run_count > 0
             ORDER BY avg_execution_time_ms DESC LIMIT $2"#,
        )
        .bind(repo_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_rule_v7_optimization_suggestions(
        &self,
        rule_id: Uuid,
    ) -> Result<Vec<RuleRecommendation>, sqlx::Error> {
        let rule = self.get_rule_v7(rule_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let metrics = self.get_rule_v7_performance_metrics(rule_id).await?;
        let mut recommendations = Vec::new();

        if rule.avg_execution_time_ms > 5000 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "slow_execution".into(),
                description: format!("Rule takes {:.0}ms on average. Consider optimizing conditions or actions.", rule.avg_execution_time_ms as f64),
                confidence: 0.9,
                suggested_changes: serde_json::json!({
                    "action": "optimize",
                    "current_avg_time_ms": rule.avg_execution_time_ms
                }),
            });
        }

        if metrics.total_runs == 0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "never_triggered".into(),
                description: "Rule has never been triggered. Consider reviewing trigger conditions.".into(),
                confidence: 0.9,
                suggested_changes: serde_json::json!({
                    "action": "review_trigger",
                    "current_trigger": rule.trigger_type
                }),
            });
        } else if rule.success_rate < 50.0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "low_success_rate".into(),
                description: format!("Rule has a low success rate of {:.1}%. Consider reviewing conditions.", rule.success_rate),
                confidence: 0.85,
                suggested_changes: serde_json::json!({
                    "action": "review_conditions",
                    "current_success_rate": rule.success_rate
                }),
            });
        }

        if rule.run_count > 100 && rule.success_rate > 95.0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "high_performance".into(),
                description: "Rule is performing well with high success rate. Consider promoting to higher priority.".into(),
                confidence: 0.8,
                suggested_changes: serde_json::json!({
                    "action": "increase_priority",
                    "current_priority": rule.priority,
                    "suggested_priority": rule.priority + 5
                }),
            });
        }

        Ok(recommendations)
    }

    // --- V8: Execution Time Tracking, Performance Analytics, Rule Optimization, Rule Recommendations ---

    pub async fn create_rule_v8(
        &self,
        input: CreateAutomationRuleV7,
    ) -> Result<AutomationRuleV8, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV8Row>(
            r#"INSERT INTO automation_rules_v8 (repo_id, name, trigger_type, conditions, actions, priority, enabled)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at"#,
        )
        .bind(input.repo_id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(input.conditions.unwrap_or(serde_json::json!({})))
        .bind(input.actions.unwrap_or(serde_json::json!([])))
        .bind(input.priority.unwrap_or(0))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_rule_v8(&self, id: Uuid) -> Result<Option<AutomationRuleV8>, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV8Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v8 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_rules_v8_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRuleV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV8Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v8 WHERE repo_id = $1 ORDER BY priority DESC, created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_enabled_rules_v8_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRuleV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV8Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v8 WHERE repo_id = $1 AND enabled = true
             ORDER BY priority DESC, created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_rule_v8(
        &self,
        id: Uuid,
        input: UpdateAutomationRuleV7,
    ) -> Result<AutomationRuleV8, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV8Row>(
            r#"UPDATE automation_rules_v8 SET
             name = COALESCE($2, name),
             trigger_type = COALESCE($3, trigger_type),
             conditions = COALESCE($4, conditions),
             actions = COALESCE($5, actions),
             priority = COALESCE($6, priority),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(&input.conditions)
        .bind(&input.actions)
        .bind(input.priority)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_rule_v8(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM automation_rules_v8 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn execute_rule_v8(
        &self,
        rule_id: Uuid,
        context: &serde_json::Value,
    ) -> Result<RuleExecutionRecord, sqlx::Error> {
        let rule = self.get_rule_v8(rule_id).await?.ok_or_else(|| {
            sqlx::Error::RowNotFound
        })?;

        let start_time = std::time::Instant::now();

        let mut matched_conditions = Vec::new();
        let mut failed_conditions = Vec::new();

        if let Some(obj) = rule.conditions.as_object() {
            for (key, expected) in obj {
                match context.get(key) {
                    Some(val) if val == expected => {
                        matched_conditions.push(key.clone());
                    }
                    _ => {
                        failed_conditions.push(key.clone());
                    }
                }
            }
        }

        let all_matched = failed_conditions.is_empty();

        let mut actions_executed = Vec::new();
        if all_matched {
            if let Some(action_list) = rule.actions.as_array() {
                for action in action_list {
                    if let Some(action_type) = action.get("type").and_then(|v| v.as_str()) {
                        actions_executed.push(action_type.to_string());
                    }
                }
            }
        }

        let elapsed_ms = start_time.elapsed().as_millis() as i32;
        let status = if all_matched { "matched" } else { "not_matched" };

        let new_success_rate = if all_matched {
            (rule.success_rate * 0.9) + (100.0 * 0.1)
        } else {
            (rule.success_rate * 0.9) + (0.0 * 0.1)
        };

        let new_avg_time = if rule.run_count == 0 {
            elapsed_ms
        } else {
            ((rule.avg_execution_time_ms as f64 * 0.9) + (elapsed_ms as f64 * 0.1)) as i32
        };

        sqlx::query(
            r#"UPDATE automation_rules_v8 SET
             run_count = run_count + 1,
             last_run_at = NOW(),
             success_rate = $2,
             avg_execution_time_ms = $3
             WHERE id = $1"#,
        )
        .bind(rule_id)
        .bind(new_success_rate)
        .bind(new_avg_time)
        .execute(&self.pool)
        .await?;

        self.record_execution(
            rule_id,
            status,
            &matched_conditions,
            &failed_conditions,
            &actions_executed,
            None,
        )
        .await
    }

    pub async fn get_rule_v8_performance_metrics(
        &self,
        rule_id: Uuid,
    ) -> Result<RulePerformanceMetrics, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct MetricsRow {
            total_runs: i64,
            successful_runs: i64,
            failed_runs: i64,
            avg_execution_time_ms: f64,
            last_execution_time_ms: Option<f64>,
        }

        let row = sqlx::query_as::<_, MetricsRow>(
            r#"SELECT
                COUNT(*) as total_runs,
                COUNT(*) FILTER (WHERE status = 'matched') as successful_runs,
                COUNT(*) FILTER (WHERE status = 'not_matched') as failed_runs,
                COALESCE(AVG(EXTRACT(EPOCH FROM (executed_at - executed_at)) * 1000), 0) as avg_execution_time_ms,
                MAX(CASE WHEN executed_at IS NOT NULL THEN EXTRACT(EPOCH FROM (executed_at - executed_at)) * 1000 END) as last_execution_time_ms
             FROM rule_execution_history WHERE rule_id = $1"#,
        )
        .bind(rule_id)
        .fetch_one(&self.pool)
        .await?;

        let success_rate = if row.total_runs > 0 {
            (row.successful_runs as f64 / row.total_runs as f64) * 100.0
        } else {
            0.0
        };

        Ok(RulePerformanceMetrics {
            rule_id,
            total_runs: row.total_runs,
            successful_runs: row.successful_runs,
            failed_runs: row.failed_runs,
            average_execution_time_ms: row.avg_execution_time_ms,
            last_execution_time_ms: row.last_execution_time_ms,
            success_rate,
        })
    }

    pub async fn get_rules_v8_with_slowest_execution(
        &self,
        repo_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AutomationRuleV8>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV8Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v8 WHERE repo_id = $1 AND run_count > 0
             ORDER BY avg_execution_time_ms DESC LIMIT $2"#,
        )
        .bind(repo_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_rule_v8_optimization_suggestions(
        &self,
        rule_id: Uuid,
    ) -> Result<Vec<RuleRecommendation>, sqlx::Error> {
        let rule = self.get_rule_v8(rule_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let metrics = self.get_rule_v8_performance_metrics(rule_id).await?;
        let mut recommendations = Vec::new();

        if rule.avg_execution_time_ms > 5000 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "slow_execution".into(),
                description: format!("Rule takes {:.0}ms on average. Consider optimizing conditions or actions.", rule.avg_execution_time_ms as f64),
                confidence: 0.9,
                suggested_changes: serde_json::json!({
                    "action": "optimize",
                    "current_avg_time_ms": rule.avg_execution_time_ms
                }),
            });
        }

        if metrics.total_runs == 0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "never_triggered".into(),
                description: "Rule has never been triggered. Consider reviewing trigger conditions.".into(),
                confidence: 0.9,
                suggested_changes: serde_json::json!({
                    "action": "review_trigger",
                    "current_trigger": rule.trigger_type
                }),
            });
        } else if rule.success_rate < 50.0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "low_success_rate".into(),
                description: format!("Rule has a low success rate of {:.1}%. Consider reviewing conditions.", rule.success_rate),
                confidence: 0.85,
                suggested_changes: serde_json::json!({
                    "action": "review_conditions",
                    "current_success_rate": rule.success_rate
                }),
            });
        }

        if rule.run_count > 100 && rule.success_rate > 95.0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "high_performance".into(),
                description: "Rule is performing well with high success rate. Consider promoting to higher priority.".into(),
                confidence: 0.8,
                suggested_changes: serde_json::json!({
                    "action": "increase_priority",
                    "current_priority": rule.priority,
                    "suggested_priority": rule.priority + 5
                }),
            });
        }

        Ok(recommendations)
    }

    // --- V9: Execution Time Tracking, Performance Analytics, Rule Optimization, Rule Recommendations ---

    pub async fn create_rule_v9(
        &self,
        input: CreateAutomationRuleV9,
    ) -> Result<AutomationRuleV9, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV9Row>(
            r#"INSERT INTO automation_rules_v9 (repo_id, name, trigger_type, conditions, actions, priority, enabled)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at"#,
        )
        .bind(input.repo_id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(input.conditions.unwrap_or(serde_json::json!({})))
        .bind(input.actions.unwrap_or(serde_json::json!([])))
        .bind(input.priority.unwrap_or(0))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_rule_v9(&self, id: Uuid) -> Result<Option<AutomationRuleV9>, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV9Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v9 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_rules_v9_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRuleV9>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV9Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v9 WHERE repo_id = $1 ORDER BY priority DESC, created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_enabled_rules_v9_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRuleV9>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV9Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v9 WHERE repo_id = $1 AND enabled = true
             ORDER BY priority DESC, created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_rule_v9(
        &self,
        id: Uuid,
        input: UpdateAutomationRuleV9,
    ) -> Result<AutomationRuleV9, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV9Row>(
            r#"UPDATE automation_rules_v9 SET
             name = COALESCE($2, name),
             trigger_type = COALESCE($3, trigger_type),
             conditions = COALESCE($4, conditions),
             actions = COALESCE($5, actions),
             priority = COALESCE($6, priority),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(&input.conditions)
        .bind(&input.actions)
        .bind(input.priority)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_rule_v9(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM automation_rules_v9 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn execute_rule_v9(
        &self,
        rule_id: Uuid,
        context: &serde_json::Value,
    ) -> Result<RuleExecutionRecord, sqlx::Error> {
        let rule = self.get_rule_v9(rule_id).await?.ok_or_else(|| {
            sqlx::Error::RowNotFound
        })?;

        let start_time = std::time::Instant::now();

        let mut matched_conditions = Vec::new();
        let mut failed_conditions = Vec::new();

        if let Some(obj) = rule.conditions.as_object() {
            for (key, expected) in obj {
                match context.get(key) {
                    Some(val) if val == expected => {
                        matched_conditions.push(key.clone());
                    }
                    _ => {
                        failed_conditions.push(key.clone());
                    }
                }
            }
        }

        let all_matched = failed_conditions.is_empty();

        let mut actions_executed = Vec::new();
        if all_matched {
            if let Some(action_list) = rule.actions.as_array() {
                for action in action_list {
                    if let Some(action_type) = action.get("type").and_then(|v| v.as_str()) {
                        actions_executed.push(action_type.to_string());
                    }
                }
            }
        }

        let elapsed_ms = start_time.elapsed().as_millis() as i32;
        let status = if all_matched { "matched" } else { "not_matched" };

        let new_success_rate = if all_matched {
            (rule.success_rate * 0.9) + (100.0 * 0.1)
        } else {
            (rule.success_rate * 0.9) + (0.0 * 0.1)
        };

        let new_avg_time = if rule.run_count == 0 {
            elapsed_ms
        } else {
            ((rule.avg_execution_time_ms as f64 * 0.9) + (elapsed_ms as f64 * 0.1)) as i32
        };

        sqlx::query(
            r#"UPDATE automation_rules_v9 SET
             run_count = run_count + 1,
             last_run_at = NOW(),
             success_rate = $2,
             avg_execution_time_ms = $3
             WHERE id = $1"#,
        )
        .bind(rule_id)
        .bind(new_success_rate)
        .bind(new_avg_time)
        .execute(&self.pool)
        .await?;

        self.record_execution(
            rule_id,
            status,
            &matched_conditions,
            &failed_conditions,
            &actions_executed,
            None,
        )
        .await
    }

    pub async fn get_rule_v9_performance_metrics(
        &self,
        rule_id: Uuid,
    ) -> Result<RulePerformanceMetrics, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct MetricsRow {
            total_runs: i64,
            successful_runs: i64,
            failed_runs: i64,
            avg_execution_time_ms: f64,
            last_execution_time_ms: Option<f64>,
        }

        let row = sqlx::query_as::<_, MetricsRow>(
            r#"SELECT
                COUNT(*) as total_runs,
                COUNT(*) FILTER (WHERE status = 'matched') as successful_runs,
                COUNT(*) FILTER (WHERE status = 'not_matched') as failed_runs,
                COALESCE(AVG(EXTRACT(EPOCH FROM (executed_at - executed_at)) * 1000), 0) as avg_execution_time_ms,
                MAX(CASE WHEN executed_at IS NOT NULL THEN EXTRACT(EPOCH FROM (executed_at - executed_at)) * 1000 END) as last_execution_time_ms
             FROM rule_execution_history WHERE rule_id = $1"#,
        )
        .bind(rule_id)
        .fetch_one(&self.pool)
        .await?;

        let success_rate = if row.total_runs > 0 {
            (row.successful_runs as f64 / row.total_runs as f64) * 100.0
        } else {
            0.0
        };

        Ok(RulePerformanceMetrics {
            rule_id,
            total_runs: row.total_runs,
            successful_runs: row.successful_runs,
            failed_runs: row.failed_runs,
            average_execution_time_ms: row.avg_execution_time_ms,
            last_execution_time_ms: row.last_execution_time_ms,
            success_rate,
        })
    }

    pub async fn get_rules_v9_with_slowest_execution(
        &self,
        repo_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AutomationRuleV9>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV9Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v9 WHERE repo_id = $1 AND run_count > 0
             ORDER BY avg_execution_time_ms DESC LIMIT $2"#,
        )
        .bind(repo_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_rule_v9_optimization_suggestions(
        &self,
        rule_id: Uuid,
    ) -> Result<Vec<RuleRecommendation>, sqlx::Error> {
        let rule = self.get_rule_v9(rule_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let metrics = self.get_rule_v9_performance_metrics(rule_id).await?;
        let mut recommendations = Vec::new();

        if rule.avg_execution_time_ms > 5000 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "slow_execution".into(),
                description: format!("Rule takes {:.0}ms on average. Consider optimizing conditions or actions.", rule.avg_execution_time_ms as f64),
                confidence: 0.9,
                suggested_changes: serde_json::json!({
                    "action": "optimize",
                    "current_avg_time_ms": rule.avg_execution_time_ms
                }),
            });
        }

        if metrics.total_runs == 0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "never_triggered".into(),
                description: "Rule has never been triggered. Consider reviewing trigger conditions.".into(),
                confidence: 0.9,
                suggested_changes: serde_json::json!({
                    "action": "review_trigger",
                    "current_trigger": rule.trigger_type
                }),
            });
        } else if rule.success_rate < 50.0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "low_success_rate".into(),
                description: format!("Rule has a low success rate of {:.1}%. Consider reviewing conditions.", rule.success_rate),
                confidence: 0.85,
                suggested_changes: serde_json::json!({
                    "action": "review_conditions",
                    "current_success_rate": rule.success_rate
                }),
            });
        }

        if rule.run_count > 100 && rule.success_rate > 95.0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "high_performance".into(),
                description: "Rule is performing well with high success rate. Consider promoting to higher priority.".into(),
                confidence: 0.8,
                suggested_changes: serde_json::json!({
                    "action": "increase_priority",
                    "current_priority": rule.priority,
                    "suggested_priority": rule.priority + 5
                }),
            });
        }

        Ok(recommendations)
    }

    pub async fn get_rule_v9_recommendations(
        &self,
        rule_id: Uuid,
    ) -> Result<Vec<RuleRecommendation>, sqlx::Error> {
        self.get_rule_v9_optimization_suggestions(rule_id).await
    }

    pub async fn get_repo_rules_v9_performance_summary(
        &self,
        repo_id: Uuid,
    ) -> Result<serde_json::Value, sqlx::Error> {
        let rules = self.list_rules_v9_for_repo(repo_id).await?;

        let total_rules = rules.len();
        let enabled_rules = rules.iter().filter(|r| r.enabled).count();
        let total_runs: i64 = rules.iter().map(|r| r.run_count as i64).sum();
        let avg_success_rate = if total_rules > 0 {
            rules.iter().map(|r| r.success_rate).sum::<f64>() / total_rules as f64
        } else {
            0.0
        };
        let avg_execution_time = if total_rules > 0 {
            rules.iter().map(|r| r.avg_execution_time_ms as f64).sum::<f64>() / total_rules as f64
        } else {
            0.0
        };

        Ok(serde_json::json!({
            "repo_id": repo_id,
            "total_rules": total_rules,
            "enabled_rules": enabled_rules,
            "total_runs": total_runs,
            "avg_success_rate": avg_success_rate,
            "avg_execution_time_ms": avg_execution_time
        }))
    }

    // --- V10: Execution Time Tracking, Performance Analytics, Rule Optimization, Rule Recommendations ---

    pub async fn create_rule_v10(
        &self,
        input: CreateAutomationRuleV10,
    ) -> Result<AutomationRuleV10, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV10Row>(
            r#"INSERT INTO automation_rules_v10 (repo_id, name, trigger_type, conditions, actions, priority, enabled)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at"#,
        )
        .bind(input.repo_id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(input.conditions.unwrap_or(serde_json::json!({})))
        .bind(input.actions.unwrap_or(serde_json::json!([])))
        .bind(input.priority.unwrap_or(0))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_rule_v10(&self, id: Uuid) -> Result<Option<AutomationRuleV10>, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV10Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v10 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_rules_v10_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRuleV10>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV10Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v10 WHERE repo_id = $1 ORDER BY priority DESC, created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_enabled_rules_v10_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRuleV10>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV10Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v10 WHERE repo_id = $1 AND enabled = true
             ORDER BY priority DESC, created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_rule_v10(
        &self,
        id: Uuid,
        input: UpdateAutomationRuleV10,
    ) -> Result<AutomationRuleV10, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV10Row>(
            r#"UPDATE automation_rules_v10 SET
             name = COALESCE($2, name),
             trigger_type = COALESCE($3, trigger_type),
             conditions = COALESCE($4, conditions),
             actions = COALESCE($5, actions),
             priority = COALESCE($6, priority),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(&input.conditions)
        .bind(&input.actions)
        .bind(input.priority)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_rule_v10(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM automation_rules_v10 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn execute_rule_v10(
        &self,
        rule_id: Uuid,
        context: &serde_json::Value,
    ) -> Result<RuleExecutionRecord, sqlx::Error> {
        let rule = self.get_rule_v10(rule_id).await?.ok_or_else(|| {
            sqlx::Error::RowNotFound
        })?;

        let start_time = std::time::Instant::now();

        let mut matched_conditions = Vec::new();
        let mut failed_conditions = Vec::new();

        if let Some(obj) = rule.conditions.as_object() {
            for (key, expected) in obj {
                match context.get(key) {
                    Some(val) if val == expected => {
                        matched_conditions.push(key.clone());
                    }
                    _ => {
                        failed_conditions.push(key.clone());
                    }
                }
            }
        }

        let all_matched = failed_conditions.is_empty();

        let mut actions_executed = Vec::new();
        if all_matched {
            if let Some(action_list) = rule.actions.as_array() {
                for action in action_list {
                    if let Some(action_type) = action.get("type").and_then(|v| v.as_str()) {
                        actions_executed.push(action_type.to_string());
                    }
                }
            }
        }

        let elapsed_ms = start_time.elapsed().as_millis() as i32;
        let status = if all_matched { "matched" } else { "not_matched" };

        let new_success_rate = if all_matched {
            (rule.success_rate * 0.9) + (100.0 * 0.1)
        } else {
            (rule.success_rate * 0.9) + (0.0 * 0.1)
        };

        let new_avg_time = if rule.run_count == 0 {
            elapsed_ms
        } else {
            ((rule.avg_execution_time_ms as f64 * 0.9) + (elapsed_ms as f64 * 0.1)) as i32
        };

        sqlx::query(
            r#"UPDATE automation_rules_v10 SET
             run_count = run_count + 1,
             last_run_at = NOW(),
             success_rate = $2,
             avg_execution_time_ms = $3
             WHERE id = $1"#,
        )
        .bind(rule_id)
        .bind(new_success_rate)
        .bind(new_avg_time)
        .execute(&self.pool)
        .await?;

        self.record_execution(
            rule_id,
            status,
            &matched_conditions,
            &failed_conditions,
            &actions_executed,
            None,
        )
        .await
    }

    pub async fn get_rule_v10_performance_metrics(
        &self,
        rule_id: Uuid,
    ) -> Result<RulePerformanceMetrics, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct MetricsRow {
            total_runs: i64,
            successful_runs: i64,
            failed_runs: i64,
            avg_execution_time_ms: f64,
            last_execution_time_ms: Option<f64>,
        }

        let row = sqlx::query_as::<_, MetricsRow>(
            r#"SELECT
                COUNT(*) as total_runs,
                COUNT(*) FILTER (WHERE status = 'matched') as successful_runs,
                COUNT(*) FILTER (WHERE status = 'not_matched') as failed_runs,
                COALESCE(AVG(EXTRACT(EPOCH FROM (executed_at - executed_at)) * 1000), 0) as avg_execution_time_ms,
                MAX(CASE WHEN executed_at IS NOT NULL THEN EXTRACT(EPOCH FROM (executed_at - executed_at)) * 1000 END) as last_execution_time_ms
             FROM rule_execution_history WHERE rule_id = $1"#,
        )
        .bind(rule_id)
        .fetch_one(&self.pool)
        .await?;

        let success_rate = if row.total_runs > 0 {
            (row.successful_runs as f64 / row.total_runs as f64) * 100.0
        } else {
            0.0
        };

        Ok(RulePerformanceMetrics {
            rule_id,
            total_runs: row.total_runs,
            successful_runs: row.successful_runs,
            failed_runs: row.failed_runs,
            average_execution_time_ms: row.avg_execution_time_ms,
            last_execution_time_ms: row.last_execution_time_ms,
            success_rate,
        })
    }

    pub async fn get_rules_v10_with_slowest_execution(
        &self,
        repo_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AutomationRuleV10>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV10Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v10 WHERE repo_id = $1 AND run_count > 0
             ORDER BY avg_execution_time_ms DESC LIMIT $2"#,
        )
        .bind(repo_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_rule_v10_optimization_suggestions(
        &self,
        rule_id: Uuid,
    ) -> Result<Vec<RuleRecommendation>, sqlx::Error> {
        let rule = self.get_rule_v10(rule_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let metrics = self.get_rule_v10_performance_metrics(rule_id).await?;
        let mut recommendations = Vec::new();

        if rule.avg_execution_time_ms > 5000 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "slow_execution".into(),
                description: format!("Rule takes {:.0}ms on average. Consider optimizing conditions or actions.", rule.avg_execution_time_ms as f64),
                confidence: 0.9,
                suggested_changes: serde_json::json!({
                    "action": "optimize",
                    "current_avg_time_ms": rule.avg_execution_time_ms
                }),
            });
        }

        if metrics.total_runs == 0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "never_triggered".into(),
                description: "Rule has never been triggered. Consider reviewing trigger conditions.".into(),
                confidence: 0.9,
                suggested_changes: serde_json::json!({
                    "action": "review_trigger",
                    "current_trigger": rule.trigger_type
                }),
            });
        } else if rule.success_rate < 50.0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "low_success_rate".into(),
                description: format!("Rule has a low success rate of {:.1}%. Consider reviewing conditions.", rule.success_rate),
                confidence: 0.85,
                suggested_changes: serde_json::json!({
                    "action": "review_conditions",
                    "current_success_rate": rule.success_rate
                }),
            });
        }

        if rule.run_count > 100 && rule.success_rate > 95.0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "high_performance".into(),
                description: "Rule is performing well with high success rate. Consider promoting to higher priority.".into(),
                confidence: 0.8,
                suggested_changes: serde_json::json!({
                    "action": "increase_priority",
                    "current_priority": rule.priority,
                    "suggested_priority": rule.priority + 5
                }),
            });
        }

        Ok(recommendations)
    }

    pub async fn get_repo_v10_performance_summary(
        &self,
        repo_id: Uuid,
    ) -> Result<serde_json::Value, sqlx::Error> {
        let rules = self.list_rules_v10_for_repo(repo_id).await?;

        let total_rules = rules.len() as i64;
        let enabled_rules = rules.iter().filter(|r| r.enabled).count() as i64;
        let total_runs: i64 = rules.iter().map(|r| r.run_count as i64).sum();
        let avg_success_rate = if !rules.is_empty() {
            rules.iter().map(|r| r.success_rate).sum::<f64>() / rules.len() as f64
        } else {
            0.0
        };
        let avg_execution_time = if !rules.is_empty() {
            rules.iter().map(|r| r.avg_execution_time_ms as f64).sum::<f64>() / rules.len() as f64
        } else {
            0.0
        };

        Ok(serde_json::json!({
            "repo_id": repo_id,
            "total_rules": total_rules,
            "enabled_rules": enabled_rules,
            "total_runs": total_runs,
            "avg_success_rate": avg_success_rate,
            "avg_execution_time_ms": avg_execution_time
        }))
    }

    // --- V11: Execution Time Tracking, Performance Analytics, Rule Optimization, Rule Recommendations ---

    pub async fn create_rule_v11(
        &self,
        input: CreateAutomationRuleV10,
    ) -> Result<AutomationRuleV11, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV11Row>(
            r#"INSERT INTO automation_rules_v11 (repo_id, name, trigger_type, conditions, actions, priority, enabled)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at"#,
        )
        .bind(input.repo_id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(input.conditions.unwrap_or(serde_json::json!({})))
        .bind(input.actions.unwrap_or(serde_json::json!([])))
        .bind(input.priority.unwrap_or(0))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_rule_v11(&self, id: Uuid) -> Result<Option<AutomationRuleV11>, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV11Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v11 WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_rules_v11_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRuleV11>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV11Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v11 WHERE repo_id = $1 ORDER BY priority DESC, created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_enabled_rules_v11_for_repo(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<AutomationRuleV11>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV11Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v11 WHERE repo_id = $1 AND enabled = true
             ORDER BY priority DESC, created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_rule_v11(
        &self,
        id: Uuid,
        input: UpdateAutomationRuleV10,
    ) -> Result<AutomationRuleV11, sqlx::Error> {
        let row = sqlx::query_as::<_, AutomationRuleV11Row>(
            r#"UPDATE automation_rules_v11 SET
             name = COALESCE($2, name),
             trigger_type = COALESCE($3, trigger_type),
             conditions = COALESCE($4, conditions),
             actions = COALESCE($5, actions),
             priority = COALESCE($6, priority),
             enabled = COALESCE($7, enabled)
             WHERE id = $1
             RETURNING id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.trigger_type)
        .bind(&input.conditions)
        .bind(&input.actions)
        .bind(input.priority)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_rule_v11(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM automation_rules_v11 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn execute_rule_v11(
        &self,
        rule_id: Uuid,
        context: &serde_json::Value,
    ) -> Result<RuleExecutionRecord, sqlx::Error> {
        let rule = self.get_rule_v11(rule_id).await?.ok_or_else(|| {
            sqlx::Error::RowNotFound
        })?;

        let start_time = std::time::Instant::now();

        let mut matched_conditions = Vec::new();
        let mut failed_conditions = Vec::new();

        if let Some(obj) = rule.conditions.as_object() {
            for (key, expected) in obj {
                match context.get(key) {
                    Some(val) if val == expected => {
                        matched_conditions.push(key.clone());
                    }
                    _ => {
                        failed_conditions.push(key.clone());
                    }
                }
            }
        }

        let all_matched = failed_conditions.is_empty();

        let mut actions_executed = Vec::new();
        if all_matched {
            if let Some(action_list) = rule.actions.as_array() {
                for action in action_list {
                    if let Some(action_type) = action.get("type").and_then(|v| v.as_str()) {
                        actions_executed.push(action_type.to_string());
                    }
                }
            }
        }

        let elapsed_ms = start_time.elapsed().as_millis() as i32;
        let status = if all_matched { "matched" } else { "not_matched" };

        let new_success_rate = if all_matched {
            (rule.success_rate * 0.9) + (100.0 * 0.1)
        } else {
            (rule.success_rate * 0.9) + (0.0 * 0.1)
        };

        let new_avg_time = if rule.run_count == 0 {
            elapsed_ms
        } else {
            ((rule.avg_execution_time_ms as f64 * 0.9) + (elapsed_ms as f64 * 0.1)) as i32
        };

        sqlx::query(
            r#"UPDATE automation_rules_v11 SET
             run_count = run_count + 1,
             last_run_at = NOW(),
             success_rate = $2,
             avg_execution_time_ms = $3
             WHERE id = $1"#,
        )
        .bind(rule_id)
        .bind(new_success_rate)
        .bind(new_avg_time)
        .execute(&self.pool)
        .await?;

        self.record_execution(
            rule_id,
            status,
            &matched_conditions,
            &failed_conditions,
            &actions_executed,
            None,
        )
        .await
    }

    pub async fn get_rule_v11_performance_metrics(
        &self,
        rule_id: Uuid,
    ) -> Result<RulePerformanceMetrics, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct MetricsRow {
            total_runs: i64,
            successful_runs: i64,
            failed_runs: i64,
            avg_execution_time_ms: f64,
            last_execution_time_ms: Option<f64>,
        }

        let row = sqlx::query_as::<_, MetricsRow>(
            r#"SELECT
                COUNT(*) as total_runs,
                COUNT(*) FILTER (WHERE status = 'matched') as successful_runs,
                COUNT(*) FILTER (WHERE status = 'not_matched') as failed_runs,
                COALESCE(AVG(EXTRACT(EPOCH FROM (executed_at - executed_at)) * 1000), 0) as avg_execution_time_ms,
                MAX(CASE WHEN executed_at IS NOT NULL THEN EXTRACT(EPOCH FROM (executed_at - executed_at)) * 1000 END) as last_execution_time_ms
             FROM rule_execution_history WHERE rule_id = $1"#,
        )
        .bind(rule_id)
        .fetch_one(&self.pool)
        .await?;

        let success_rate = if row.total_runs > 0 {
            (row.successful_runs as f64 / row.total_runs as f64) * 100.0
        } else {
            0.0
        };

        Ok(RulePerformanceMetrics {
            rule_id,
            total_runs: row.total_runs,
            successful_runs: row.successful_runs,
            failed_runs: row.failed_runs,
            average_execution_time_ms: row.avg_execution_time_ms,
            last_execution_time_ms: row.last_execution_time_ms,
            success_rate,
        })
    }

    pub async fn get_rules_v11_with_slowest_execution(
        &self,
        repo_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AutomationRuleV11>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AutomationRuleV11Row>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, priority, enabled, last_run_at, run_count, success_rate, avg_execution_time_ms, created_at
             FROM automation_rules_v11 WHERE repo_id = $1 AND run_count > 0
             ORDER BY avg_execution_time_ms DESC LIMIT $2"#,
        )
        .bind(repo_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_rule_v11_optimization_suggestions(
        &self,
        rule_id: Uuid,
    ) -> Result<Vec<RuleRecommendation>, sqlx::Error> {
        let rule = self.get_rule_v11(rule_id).await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let metrics = self.get_rule_v11_performance_metrics(rule_id).await?;
        let mut recommendations = Vec::new();

        if rule.avg_execution_time_ms > 5000 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "slow_execution".into(),
                description: format!("Rule takes {:.0}ms on average. Consider optimizing conditions or actions.", rule.avg_execution_time_ms as f64),
                confidence: 0.9,
                suggested_changes: serde_json::json!({
                    "action": "optimize",
                    "current_avg_time_ms": rule.avg_execution_time_ms
                }),
            });
        }

        if metrics.total_runs == 0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "never_triggered".into(),
                description: "Rule has never been triggered. Consider reviewing trigger conditions.".into(),
                confidence: 0.9,
                suggested_changes: serde_json::json!({
                    "action": "review_trigger",
                    "current_trigger": rule.trigger_type
                }),
            });
        } else if rule.success_rate < 50.0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "low_success_rate".into(),
                description: format!("Rule has a low success rate of {:.1}%. Consider reviewing conditions.", rule.success_rate),
                confidence: 0.85,
                suggested_changes: serde_json::json!({
                    "action": "review_conditions",
                    "current_success_rate": rule.success_rate
                }),
            });
        }

        if rule.run_count > 100 && rule.success_rate > 95.0 {
            recommendations.push(RuleRecommendation {
                rule_id,
                recommendation_type: "high_performance".into(),
                description: "Rule is performing well with high success rate. Consider promoting to higher priority.".into(),
                confidence: 0.8,
                suggested_changes: serde_json::json!({
                    "action": "increase_priority",
                    "current_priority": rule.priority,
                    "suggested_priority": rule.priority + 5
                }),
            });
        }

        Ok(recommendations)
    }

    pub async fn get_repo_v11_performance_summary(
        &self,
        repo_id: Uuid,
    ) -> Result<serde_json::Value, sqlx::Error> {
        let rules = self.list_rules_v11_for_repo(repo_id).await?;

        let total_rules = rules.len() as i64;
        let enabled_rules = rules.iter().filter(|r| r.enabled).count() as i64;
        let total_runs: i64 = rules.iter().map(|r| r.run_count as i64).sum();
        let avg_success_rate = if !rules.is_empty() {
            rules.iter().map(|r| r.success_rate).sum::<f64>() / rules.len() as f64
        } else {
            0.0
        };
        let avg_execution_time = if !rules.is_empty() {
            rules.iter().map(|r| r.avg_execution_time_ms as f64).sum::<f64>() / rules.len() as f64
        } else {
            0.0
        };

        Ok(serde_json::json!({
            "repo_id": repo_id,
            "total_rules": total_rules,
            "enabled_rules": enabled_rules,
            "total_runs": total_runs,
            "avg_success_rate": avg_success_rate,
            "avg_execution_time_ms": avg_execution_time
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_rule_input_serialization() {
        let input = CreateAutomationRule {
            repo_id: Uuid::new_v4(),
            name: "Auto-label PRs".into(),
            trigger_type: "pull_request".into(),
            conditions: Some(serde_json::json!({"title": {"$contains": "feat"}})),
            actions: Some(serde_json::json!([{"type": "add_label", "label": "feature"}])),
            enabled: Some(true),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("Auto-label PRs"));
        assert!(json.contains("pull_request"));
    }

    #[test]
    fn test_rule_test_result_serialization() {
        let result = RuleTestResult {
            rule_id: Uuid::new_v4(),
            matched: true,
            conditions_met: vec!["title".into()],
            conditions_failed: vec![],
            actions_executed: vec!["add_label".into()],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("matched"));
    }

    #[test]
    fn test_evaluate_empty_conditions() {
        let conds = serde_json::json!({});
        let conds_obj = conds.as_object().unwrap();
        assert!(conds_obj.is_empty());
    }

    #[test]
    fn test_rule_v2_serialization() {
        let rule = AutomationRuleV2 {
            id: Uuid::new_v4(),
            repo_id: Uuid::new_v4(),
            name: "High Priority Auto-merge".into(),
            trigger_type: "pull_request".into(),
            conditions: serde_json::json!({"review_approved": true}),
            actions: serde_json::json!([{"type": "auto_merge"}]),
            priority: 10,
            enabled: true,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&rule).unwrap();
        assert!(json.contains("High Priority Auto-merge"));
        assert!(json.contains("10"));
    }

    #[test]
    fn test_execution_record_serialization() {
        let record = RuleExecutionRecord {
            id: Uuid::new_v4(),
            rule_id: Uuid::new_v4(),
            status: "matched".into(),
            matched_conditions: vec!["branch".into()],
            failed_conditions: vec![],
            actions_executed: vec!["deploy".into()],
            error: None,
            executed_at: Utc::now(),
        };
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("matched"));
    }

    #[test]
    fn test_automation_rule_v3_serialization() {
        let rule = AutomationRuleV3 {
            id: Uuid::new_v4(),
            repo_id: Uuid::new_v4(),
            name: "Auto-deploy v3".into(),
            trigger_type: "push".into(),
            conditions: serde_json::json!({"branch": "main"}),
            actions: serde_json::json!([{"type": "deploy"}]),
            priority: 5,
            enabled: true,
            last_run_at: Some(Utc::now()),
            run_count: 42,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&rule).unwrap();
        assert!(json.contains("Auto-deploy v3"));
        assert!(json.contains("42"));
    }

    #[test]
    fn test_create_automation_rule_v3_input_serialization() {
        let input = CreateAutomationRuleV3 {
            repo_id: Uuid::new_v4(),
            name: "Test Rule v3".into(),
            trigger_type: "pull_request".into(),
            conditions: Some(serde_json::json!({"title": {"$contains": "feat"}})),
            actions: Some(serde_json::json!([{"type": "add_label", "label": "feature"}])),
            priority: Some(10),
            enabled: Some(true),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("Test Rule v3"));
        assert!(json.contains("pull_request"));
    }

    #[test]
    fn test_rule_performance_metrics_serialization() {
        let metrics = RulePerformanceMetrics {
            rule_id: Uuid::new_v4(),
            total_runs: 100,
            successful_runs: 90,
            failed_runs: 10,
            average_execution_time_ms: 150.5,
            last_execution_time_ms: Some(120.0),
            success_rate: 90.0,
        };
        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("100"));
        assert!(json.contains("90"));
        assert!(json.contains("150.5"));
    }

    #[test]
    fn test_automation_rule_v4_serialization() {
        let rule = AutomationRuleV4 {
            id: Uuid::new_v4(),
            repo_id: Uuid::new_v4(),
            name: "Auto-deploy v4".into(),
            trigger_type: "push".into(),
            conditions: serde_json::json!({"branch": "main"}),
            actions: serde_json::json!([{"type": "deploy"}]),
            priority: 5,
            enabled: true,
            last_run_at: Some(Utc::now()),
            run_count: 42,
            success_rate: 95.5,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&rule).unwrap();
        assert!(json.contains("Auto-deploy v4"));
        assert!(json.contains("42"));
        assert!(json.contains("95.5"));
    }

    #[test]
    fn test_create_automation_rule_v4_input_serialization() {
        let input = CreateAutomationRuleV4 {
            repo_id: Uuid::new_v4(),
            name: "Test Rule v4".into(),
            trigger_type: "pull_request".into(),
            conditions: Some(serde_json::json!({"title": {"$contains": "feat"}})),
            actions: Some(serde_json::json!([{"type": "add_label", "label": "feature"}])),
            priority: Some(10),
            enabled: Some(true),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("Test Rule v4"));
        assert!(json.contains("pull_request"));
    }

    #[test]
    fn test_rule_recommendation_serialization() {
        let rec = RuleRecommendation {
            rule_id: Uuid::new_v4(),
            recommendation_type: "low_success_rate".into(),
            description: "Rule has low success rate".into(),
            confidence: 0.85,
            suggested_changes: serde_json::json!({"action": "review_conditions"}),
        };
        let json = serde_json::to_string(&rec).unwrap();
        assert!(json.contains("low_success_rate"));
        assert!(json.contains("0.85"));
    }

    #[test]
    fn test_automation_rule_v5_serialization() {
        let rule = AutomationRuleV5 {
            id: Uuid::new_v4(),
            repo_id: Uuid::new_v4(),
            name: "Auto-deploy v5".into(),
            trigger_type: "push".into(),
            conditions: serde_json::json!({"branch": "main"}),
            actions: serde_json::json!([{"type": "deploy"}]),
            priority: 5,
            enabled: true,
            last_run_at: Some(Utc::now()),
            run_count: 42,
            success_rate: 95.5,
            avg_execution_time_ms: 150,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&rule).unwrap();
        assert!(json.contains("Auto-deploy v5"));
        assert!(json.contains("42"));
        assert!(json.contains("95.5"));
        assert!(json.contains("150"));
    }

    #[test]
    fn test_create_automation_rule_v5_input_serialization() {
        let input = CreateAutomationRuleV5 {
            repo_id: Uuid::new_v4(),
            name: "Test Rule v5".into(),
            trigger_type: "pull_request".into(),
            conditions: Some(serde_json::json!({"title": {"$contains": "feat"}})),
            actions: Some(serde_json::json!([{"type": "add_label", "label": "feature"}])),
            priority: Some(10),
            enabled: Some(true),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("Test Rule v5"));
        assert!(json.contains("pull_request"));
    }

    #[test]
    fn test_automation_rule_v7_serialization() {
        let rule = AutomationRuleV7 {
            id: Uuid::new_v4(),
            repo_id: Uuid::new_v4(),
            name: "Auto-deploy v7".into(),
            trigger_type: "push".into(),
            conditions: serde_json::json!({"branch": "main"}),
            actions: serde_json::json!([{"type": "deploy"}]),
            priority: 5,
            enabled: true,
            last_run_at: Some(Utc::now()),
            run_count: 42,
            success_rate: 95.5,
            avg_execution_time_ms: 150,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&rule).unwrap();
        assert!(json.contains("Auto-deploy v7"));
        assert!(json.contains("42"));
        assert!(json.contains("95.5"));
        assert!(json.contains("150"));
    }

    #[test]
    fn test_create_automation_rule_v7_input_serialization() {
        let input = CreateAutomationRuleV7 {
            repo_id: Uuid::new_v4(),
            name: "Test Rule v7".into(),
            trigger_type: "pull_request".into(),
            conditions: Some(serde_json::json!({"title": {"$contains": "feat"}})),
            actions: Some(serde_json::json!([{"type": "add_label", "label": "feature"}])),
            priority: Some(10),
            enabled: Some(true),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("Test Rule v7"));
        assert!(json.contains("pull_request"));
    }

    #[test]
    fn test_automation_rule_v8_serialization() {
        let rule = AutomationRuleV8 {
            id: Uuid::new_v4(),
            repo_id: Uuid::new_v4(),
            name: "Auto-deploy v8".into(),
            trigger_type: "push".into(),
            conditions: serde_json::json!({"branch": "main"}),
            actions: serde_json::json!([{"type": "deploy"}]),
            priority: 5,
            enabled: true,
            last_run_at: Some(Utc::now()),
            run_count: 42,
            success_rate: 95.5,
            avg_execution_time_ms: 150,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&rule).unwrap();
        assert!(json.contains("Auto-deploy v8"));
        assert!(json.contains("42"));
        assert!(json.contains("95.5"));
        assert!(json.contains("150"));
    }

    #[test]
    fn test_create_automation_rule_v8_input_serialization() {
        let input = CreateAutomationRuleV7 {
            repo_id: Uuid::new_v4(),
            name: "Test Rule v8".into(),
            trigger_type: "pull_request".into(),
            conditions: Some(serde_json::json!({"title": {"$contains": "feat"}})),
            actions: Some(serde_json::json!([{"type": "add_label", "label": "feature"}])),
            priority: Some(10),
            enabled: Some(true),
        };
        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("Test Rule v8"));
        assert!(json.contains("pull_request"));
    }
}
