#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationEntry {
    pub id: Uuid,
    pub key_id: Uuid,
    pub old_key_id: Option<Uuid>,
    pub rotated_at: DateTime<Utc>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionAuditEntry {
    pub id: Uuid,
    pub key_id: Uuid,
    pub action: String,
    pub user_id: Option<Uuid>,
    pub details: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyLifecycleInfo {
    pub key_id: Uuid,
    pub key_name: String,
    pub algorithm: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub rotation_date: Option<DateTime<Utc>>,
    pub days_since_creation: i64,
    pub days_since_rotation: Option<i64>,
    pub needs_rotation: bool,
}

pub struct EncryptionV3Service {
    pool: PgPool,
}

impl EncryptionV3Service {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn rotate_key(
        &self,
        key_id: Uuid,
        old_key_id: Option<Uuid>,
        reason: &str,
    ) -> Result<KeyRotationEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct RotationRow {
            id: Uuid,
            key_id: Uuid,
            old_key_id: Option<Uuid>,
            rotated_at: DateTime<Utc>,
            reason: String,
        }

        let row = sqlx::query_as::<_, RotationRow>(
            r#"INSERT INTO encryption_key_rotations (key_id, old_key_id, reason)
             VALUES ($1, $2, $3)
             RETURNING id, key_id, old_key_id, rotated_at, reason"#,
        )
        .bind(key_id)
        .bind(old_key_id)
        .bind(reason)
        .fetch_one(&self.pool)
        .await?;

        Ok(KeyRotationEntry {
            id: row.id,
            key_id: row.key_id,
            old_key_id: row.old_key_id,
            rotated_at: row.rotated_at,
            reason: row.reason,
        })
    }

    pub async fn log_audit(
        &self,
        key_id: Uuid,
        action: &str,
        user_id: Option<Uuid>,
        details: serde_json::Value,
    ) -> Result<EncryptionAuditEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AuditRow {
            id: Uuid,
            key_id: Uuid,
            action: String,
            user_id: Option<Uuid>,
            details: serde_json::Value,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, AuditRow>(
            r#"INSERT INTO encryption_audit_logs (key_id, action, user_id, details)
             VALUES ($1, $2, $3, $4)
             RETURNING id, key_id, action, user_id, details, created_at"#,
        )
        .bind(key_id)
        .bind(action)
        .bind(user_id)
        .bind(details)
        .fetch_one(&self.pool)
        .await?;

        Ok(EncryptionAuditEntry {
            id: row.id,
            key_id: row.key_id,
            action: row.action,
            user_id: row.user_id,
            details: row.details,
            created_at: row.created_at,
        })
    }

    pub async fn get_key_lifecycle(&self, key_id: Uuid) -> Result<Option<KeyLifecycleInfo>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct KeyRow {
            id: Uuid,
            name: String,
            algorithm: String,
            enabled: bool,
            created_at: DateTime<Utc>,
            rotation_date: Option<DateTime<Utc>>,
        }

        let row = sqlx::query_as::<_, KeyRow>(
            r#"SELECT id, name, algorithm, enabled, created_at, rotation_date
             FROM encryption_keys WHERE id = $1"#,
        )
        .bind(key_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            let now = Utc::now();
            let days_since_creation = (now - r.created_at).num_days();
            let days_since_rotation = r.rotation_date.map(|rot| (now - rot).num_days());
            let needs_rotation = days_since_rotation.unwrap_or(days_since_creation) > 90;

            KeyLifecycleInfo {
                key_id: r.id,
                key_name: r.name,
                algorithm: r.algorithm,
                enabled: r.enabled,
                created_at: r.created_at,
                rotation_date: r.rotation_date,
                days_since_creation,
                days_since_rotation,
                needs_rotation,
            }
        }))
    }
}
