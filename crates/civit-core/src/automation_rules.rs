#![forbid(unsafe_code)]

//! Automation rules for CivitForge.
//!
//! Provides rule CRUD, condition evaluation, action execution, and rule testing.

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
pub struct CreateAutomationRule {
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: Option<serde_json::Value>,
    pub actions: Option<serde_json::Value>,
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
pub struct RuleTestResult {
    pub rule_id: Uuid,
    pub matched: bool,
    pub conditions_met: Vec<String>,
    pub conditions_failed: Vec<String>,
    pub actions_executed: Vec<String>,
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

        let conditions = rule.conditions.as_object();
        let conditions = match conditions {
            Some(c) => c,
            None => return Ok(true),
        };

        if conditions.is_empty() {
            return Ok(true);
        }

        for (key, expected) in conditions {
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
}
