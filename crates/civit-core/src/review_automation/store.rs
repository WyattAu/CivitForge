use super::types::*;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct ReviewAutomationStore {
    pool: PgPool,
}

impl ReviewAutomationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_rule(
        &self,
        repo_id: Uuid,
        req: CreateReviewRuleRequest,
    ) -> Result<ReviewAutomationRule, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let conditions = serde_json::to_value(&req.conditions)
            .unwrap_or(serde_json::json!({}));
        let actions = serde_json::to_value(&req.actions)
            .unwrap_or(serde_json::json!([]));

        sqlx::query(
            r#"INSERT INTO review_automation_rules (id, repo_id, name, trigger_type, conditions, actions, enabled, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        )
        .bind(id)
        .bind(repo_id)
        .bind(&req.name)
        .bind(req.trigger_type.to_string())
        .bind(&conditions)
        .bind(&actions)
        .bind(req.enabled.unwrap_or(true))
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(ReviewAutomationRule {
            id,
            repo_id,
            name: req.name,
            trigger_type: req.trigger_type,
            conditions,
            actions: serde_json::from_value(actions).unwrap_or_default(),
            enabled: req.enabled.unwrap_or(true),
            created_at: now,
        })
    }

    pub async fn get_rule(&self, id: Uuid) -> Result<Option<ReviewAutomationRule>, sqlx::Error> {
        let row = sqlx::query_as::<_, RuleRow>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, enabled, created_at
               FROM review_automation_rules WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(ReviewAutomationRule::from))
    }

    pub async fn list_rules(
        &self,
        repo_id: Uuid,
        enabled_only: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ReviewAutomationRule>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RuleRow>(
            r#"SELECT id, repo_id, name, trigger_type, conditions, actions, enabled, created_at
               FROM review_automation_rules
               WHERE repo_id = $1
                 AND ($2 = false OR enabled = true)
               ORDER BY created_at DESC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(repo_id)
        .bind(enabled_only)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(ReviewAutomationRule::from).collect())
    }

    pub async fn update_rule(
        &self,
        id: Uuid,
        req: UpdateReviewRuleRequest,
    ) -> Result<ReviewAutomationRule, sqlx::Error> {
        let conditions_val = req
            .conditions
            .map(|c| serde_json::to_value(c).unwrap_or(serde_json::json!({})));
        let actions_val = req
            .actions
            .map(|a| serde_json::to_value(a).unwrap_or(serde_json::json!([])));
        let trigger_type_str = req.trigger_type.map(|t| t.to_string());

        let row = sqlx::query_as::<_, RuleRow>(
            r#"UPDATE review_automation_rules SET
               name = COALESCE($2, name),
               trigger_type = COALESCE($3, trigger_type),
               conditions = COALESCE($4, conditions),
               actions = COALESCE($5, actions),
               enabled = COALESCE($6, enabled)
               WHERE id = $1
               RETURNING id, repo_id, name, trigger_type, conditions, actions, enabled, created_at"#,
        )
        .bind(id)
        .bind(&req.name)
        .bind(trigger_type_str)
        .bind(conditions_val)
        .bind(actions_val)
        .bind(req.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(ReviewAutomationRule::from(row))
    }

    pub async fn delete_rule(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM review_automation_rules WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn test_rule(
        &self,
        rule_id: Uuid,
        context: &serde_json::Value,
    ) -> Result<ReviewRuleTestResult, sqlx::Error> {
        let rule = self
            .get_rule(rule_id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

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

        let actions_to_execute: Vec<String> = rule
            .actions
            .iter()
            .map(|a| match a {
                ReviewAction::AssignReviewer { .. } => "assign_reviewer".into(),
                ReviewAction::AddLabel { .. } => "add_label".into(),
                ReviewAction::RemoveLabel { .. } => "remove_label".into(),
                ReviewAction::Comment { .. } => "comment".into(),
                ReviewAction::RequestReview { .. } => "request_review".into(),
                ReviewAction::SetReviewers { .. } => "set_reviewers".into(),
                ReviewAction::Reminder { .. } => "reminder".into(),
            })
            .collect();

        Ok(ReviewRuleTestResult {
            rule_id,
            matched,
            conditions_met,
            conditions_failed,
            actions_to_execute,
        })
    }

    pub async fn execute_rule(
        &self,
        rule_id: Uuid,
        trigger_event: &str,
    ) -> Result<ReviewRuleExecutionLog, sqlx::Error> {
        let rule = self
            .get_rule(rule_id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let actions_executed: Vec<String> = rule
            .actions
            .iter()
            .map(|a| match a {
                ReviewAction::AssignReviewer { .. } => "assign_reviewer".into(),
                ReviewAction::AddLabel { .. } => "add_label".into(),
                ReviewAction::RemoveLabel { .. } => "remove_label".into(),
                ReviewAction::Comment { .. } => "comment".into(),
                ReviewAction::RequestReview { .. } => "request_review".into(),
                ReviewAction::SetReviewers { .. } => "set_reviewers".into(),
                ReviewAction::Reminder { .. } => "reminder".into(),
            })
            .collect();

        Ok(ReviewRuleExecutionLog {
            rule_id,
            trigger_event: trigger_event.to_string(),
            matched: true,
            actions_executed,
            executed_at: Utc::now(),
        })
    }
}

#[derive(sqlx::FromRow)]
struct RuleRow {
    id: Uuid,
    repo_id: Uuid,
    name: String,
    trigger_type: String,
    conditions: serde_json::Value,
    actions: serde_json::Value,
    enabled: bool,
    created_at: chrono::DateTime<Utc>,
}

impl From<RuleRow> for ReviewAutomationRule {
    fn from(row: RuleRow) -> Self {
        Self {
            id: row.id,
            repo_id: row.repo_id,
            name: row.name,
            trigger_type: row.trigger_type.parse().unwrap_or(ReviewTriggerType::PullRequestOpened),
            conditions: row.conditions,
            actions: serde_json::from_value(row.actions).unwrap_or_default(),
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}
