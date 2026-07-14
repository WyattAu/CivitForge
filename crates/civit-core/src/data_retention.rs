#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRetentionPolicy {
    pub id: Uuid,
    pub name: String,
    pub data_types: Vec<String>,
    pub retention_days: i32,
    pub action: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDataRetentionPolicy {
    pub name: String,
    pub data_types: Option<Vec<String>>,
    pub retention_days: Option<i32>,
    pub action: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDataRetentionPolicy {
    pub name: Option<String>,
    pub data_types: Option<Vec<String>>,
    pub retention_days: Option<i32>,
    pub action: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRetentionAction {
    pub id: Uuid,
    pub policy_id: Uuid,
    pub data_type: String,
    pub data_id: Uuid,
    pub action_taken: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDataRetentionAction {
    pub policy_id: Uuid,
    pub data_type: String,
    pub data_id: Uuid,
    pub action_taken: String,
}

#[derive(Debug, sqlx::FromRow)]
struct DataRetentionPolicyRow {
    id: Uuid,
    name: String,
    data_types: Vec<String>,
    retention_days: i32,
    action: String,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<DataRetentionPolicyRow> for DataRetentionPolicy {
    fn from(row: DataRetentionPolicyRow) -> Self {
        DataRetentionPolicy {
            id: row.id,
            name: row.name,
            data_types: row.data_types,
            retention_days: row.retention_days,
            action: row.action,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct DataRetentionActionRow {
    id: Uuid,
    policy_id: Uuid,
    data_type: String,
    data_id: Uuid,
    action_taken: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<DataRetentionActionRow> for DataRetentionAction {
    fn from(row: DataRetentionActionRow) -> Self {
        DataRetentionAction {
            id: row.id,
            policy_id: row.policy_id,
            data_type: row.data_type,
            data_id: row.data_id,
            action_taken: row.action_taken,
            created_at: row.created_at,
        }
    }
}

pub struct DataRetentionService {
    pool: PgPool,
}

impl DataRetentionService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_policy(
        &self,
        input: CreateDataRetentionPolicy,
    ) -> Result<DataRetentionPolicy, sqlx::Error> {
        let row = sqlx::query_as::<_, DataRetentionPolicyRow>(
            r#"INSERT INTO data_retention_policies (name, data_types, retention_days, action, enabled)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, name, data_types, retention_days, action, enabled, created_at"#,
        )
        .bind(&input.name)
        .bind(input.data_types.as_deref().unwrap_or(&[]))
        .bind(input.retention_days.unwrap_or(365))
        .bind(input.action.as_deref().unwrap_or("archive"))
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_policy(
        &self,
        id: Uuid,
    ) -> Result<Option<DataRetentionPolicy>, sqlx::Error> {
        let row = sqlx::query_as::<_, DataRetentionPolicyRow>(
            r#"SELECT id, name, data_types, retention_days, action, enabled, created_at
             FROM data_retention_policies WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_policies(&self) -> Result<Vec<DataRetentionPolicy>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DataRetentionPolicyRow>(
            r#"SELECT id, name, data_types, retention_days, action, enabled, created_at
             FROM data_retention_policies ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_policy(
        &self,
        id: Uuid,
        input: UpdateDataRetentionPolicy,
    ) -> Result<DataRetentionPolicy, sqlx::Error> {
        let row = sqlx::query_as::<_, DataRetentionPolicyRow>(
            r#"UPDATE data_retention_policies SET
             name = COALESCE($2, name),
             data_types = COALESCE($3, data_types),
             retention_days = COALESCE($4, retention_days),
             action = COALESCE($5, action),
             enabled = COALESCE($6, enabled)
             WHERE id = $1
             RETURNING id, name, data_types, retention_days, action, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(input.data_types.as_deref())
        .bind(input.retention_days)
        .bind(&input.action)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_policy(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM data_retention_policies WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn record_action(
        &self,
        input: CreateDataRetentionAction,
    ) -> Result<DataRetentionAction, sqlx::Error> {
        let row = sqlx::query_as::<_, DataRetentionActionRow>(
            r#"INSERT INTO data_retention_actions (policy_id, data_type, data_id, action_taken)
             VALUES ($1, $2, $3, $4)
             RETURNING id, policy_id, data_type, data_id, action_taken, created_at"#,
        )
        .bind(input.policy_id)
        .bind(&input.data_type)
        .bind(input.data_id)
        .bind(&input.action_taken)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn list_actions_for_policy(
        &self,
        policy_id: Uuid,
    ) -> Result<Vec<DataRetentionAction>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DataRetentionActionRow>(
            r#"SELECT id, policy_id, data_type, data_id, action_taken, created_at
             FROM data_retention_actions WHERE policy_id = $1 ORDER BY created_at DESC"#,
        )
        .bind(policy_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_serialization() {
        let policy = DataRetentionPolicy {
            id: Uuid::new_v4(),
            name: "retain-logs".into(),
            data_types: vec!["logs".into(), "events".into()],
            retention_days: 90,
            action: "archive".into(),
            enabled: true,
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&policy).unwrap();
        assert!(json.contains("retain-logs"));
        assert!(json.contains("archive"));
    }

    #[test]
    fn test_action_serialization() {
        let action = DataRetentionAction {
            id: Uuid::new_v4(),
            policy_id: Uuid::new_v4(),
            data_type: "logs".into(),
            data_id: Uuid::new_v4(),
            action_taken: "archived".into(),
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("archived"));
    }
}
