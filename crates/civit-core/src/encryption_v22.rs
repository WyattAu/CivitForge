#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyUsageEntryV20 {
    pub id: Uuid,
    pub key_id: Uuid,
    pub operation: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationScheduleEntryV20 {
    pub id: Uuid,
    pub key_id: Uuid,
    pub rotation_days: i32,
    pub last_rotated_at: Option<DateTime<Utc>>,
    pub next_rotation_at: Option<DateTime<Utc>>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPerformanceMetricsV22 {
    pub key_id: Uuid,
    pub key_name: String,
    pub total_operations: i64,
    pub successful_operations: i64,
    pub failed_operations: i64,
    pub success_rate: f64,
    pub avg_operation_time_ms: f64,
    pub last_operation_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReportEntryV22 {
    pub report_id: Uuid,
    pub report_type: String,
    pub key_count: i64,
    pub keys_needing_rotation: i64,
    pub average_compliance_score: f64,
    pub generated_at: DateTime<Utc>,
    pub findings: serde_json::Value,
}

pub struct EncryptionV22Service {
    pool: PgPool,
}

impl EncryptionV22Service {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn log_key_usage(
        &self,
        key_id: Uuid,
        operation: &str,
        success: bool,
        error_message: Option<&str>,
    ) -> Result<KeyUsageEntryV20, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct UsageRow {
            id: Uuid,
            key_id: Uuid,
            operation: String,
            success: bool,
            error_message: Option<String>,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, UsageRow>(
            r#"INSERT INTO encryption_key_usage_v20 (key_id, operation, success, error_message)
             VALUES ($1, $2, $3, $4)
             RETURNING id, key_id, operation, success, error_message, created_at"#,
        )
        .bind(key_id)
        .bind(operation)
        .bind(success)
        .bind(error_message)
        .fetch_one(&self.pool)
        .await?;

        Ok(KeyUsageEntryV20 {
            id: row.id,
            key_id: row.key_id,
            operation: row.operation,
            success: row.success,
            error_message: row.error_message,
            created_at: row.created_at,
        })
    }

    pub async fn get_key_usage(
        &self,
        key_id: Uuid,
    ) -> Result<Vec<KeyUsageEntryV20>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct UsageRow {
            id: Uuid,
            key_id: Uuid,
            operation: String,
            success: bool,
            error_message: Option<String>,
            created_at: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, UsageRow>(
            r#"SELECT id, key_id, operation, success, error_message, created_at
             FROM encryption_key_usage_v20
             WHERE key_id = $1
             ORDER BY created_at DESC"#,
        )
        .bind(key_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| KeyUsageEntryV20 {
                id: r.id,
                key_id: r.key_id,
                operation: r.operation,
                success: r.success,
                error_message: r.error_message,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn create_rotation_schedule(
        &self,
        key_id: Uuid,
        rotation_days: i32,
    ) -> Result<RotationScheduleEntryV20, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct ScheduleRow {
            id: Uuid,
            key_id: Uuid,
            rotation_days: i32,
            last_rotated_at: Option<DateTime<Utc>>,
            next_rotation_at: Option<DateTime<Utc>>,
            enabled: bool,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, ScheduleRow>(
            r#"INSERT INTO encryption_key_rotation_schedules_v20 (key_id, rotation_days, next_rotation_at)
             VALUES ($1, $2, NOW() + ($3 || ' days')::INTERVAL)
             ON CONFLICT (key_id) DO UPDATE SET rotation_days = $2, next_rotation_at = NOW() + ($3 || ' days')::INTERVAL
             RETURNING id, key_id, rotation_days, last_rotated_at, next_rotation_at, enabled, created_at"#,
        )
        .bind(key_id)
        .bind(rotation_days)
        .bind(rotation_days.to_string())
        .fetch_one(&self.pool)
        .await?;

        Ok(RotationScheduleEntryV20 {
            id: row.id,
            key_id: row.key_id,
            rotation_days: row.rotation_days,
            last_rotated_at: row.last_rotated_at,
            next_rotation_at: row.next_rotation_at,
            enabled: row.enabled,
            created_at: row.created_at,
        })
    }

    pub async fn get_rotation_schedules(
        &self,
    ) -> Result<Vec<RotationScheduleEntryV20>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct ScheduleRow {
            id: Uuid,
            key_id: Uuid,
            rotation_days: i32,
            last_rotated_at: Option<DateTime<Utc>>,
            next_rotation_at: Option<DateTime<Utc>>,
            enabled: bool,
            created_at: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, ScheduleRow>(
            r#"SELECT id, key_id, rotation_days, last_rotated_at, next_rotation_at, enabled, created_at
             FROM encryption_key_rotation_schedules_v20
             WHERE enabled = true
             ORDER BY next_rotation_at ASC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| RotationScheduleEntryV20 {
                id: r.id,
                key_id: r.key_id,
                rotation_days: r.rotation_days,
                last_rotated_at: r.last_rotated_at,
                next_rotation_at: r.next_rotation_at,
                enabled: r.enabled,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn get_key_performance_metrics(
        &self,
        key_id: Uuid,
    ) -> Result<Option<KeyPerformanceMetricsV22>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct KeyInfo {
            id: Uuid,
            name: String,
        }

        let key = sqlx::query_as::<_, KeyInfo>(
            r#"SELECT id, name FROM encryption_keys WHERE id = $1"#,
        )
        .bind(key_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(k) = key {
            let total_ops: (i64,) = sqlx::query_as(
                r#"SELECT COUNT(*) FROM encryption_key_usage_v20 WHERE key_id = $1"#,
            )
            .bind(key_id)
            .fetch_one(&self.pool)
            .await?;

            let successful_ops: (i64,) = sqlx::query_as(
                r#"SELECT COUNT(*) FROM encryption_key_usage_v20 WHERE key_id = $1 AND success = true"#,
            )
            .bind(key_id)
            .fetch_one(&self.pool)
            .await?;

            let failed_ops: (i64,) = sqlx::query_as(
                r#"SELECT COUNT(*) FROM encryption_key_usage_v20 WHERE key_id = $1 AND success = false"#,
            )
            .bind(key_id)
            .fetch_one(&self.pool)
            .await?;

            let last_op: Option<(Option<DateTime<Utc>>,)> = sqlx::query_as(
                r#"SELECT MAX(created_at) FROM encryption_key_usage_v20 WHERE key_id = $1"#,
            )
            .bind(key_id)
            .fetch_optional(&self.pool)
            .await?;

            let total = total_ops.0;
            let successful = successful_ops.0;
            let failed = failed_ops.0;
            let success_rate = if total > 0 {
                (successful as f64 / total as f64) * 100.0
            } else {
                100.0
            };

            Ok(Some(KeyPerformanceMetricsV22 {
                key_id: k.id,
                key_name: k.name,
                total_operations: total,
                successful_operations: successful,
                failed_operations: failed,
                success_rate,
                avg_operation_time_ms: 0.0,
                last_operation_at: last_op.and_then(|l| l.0),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn generate_compliance_report(
        &self,
        report_type: &str,
    ) -> Result<ComplianceReportEntryV22, sqlx::Error> {
        let key_count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM encryption_keys"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let keys_needing_rotation: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM encryption_key_rotation_schedules_v20
             WHERE enabled = true AND next_rotation_at < NOW()"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let average_score: (f64,) = sqlx::query_as(
            r#"SELECT COALESCE(AVG(score), 0) FROM encryption_compliance_checks_v18"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let findings = serde_json::json!({
            "total_keys": key_count.0,
            "keys_needing_rotation": keys_needing_rotation.0,
            "average_compliance_score": average_score.0,
        });

        Ok(ComplianceReportEntryV22 {
            report_id: Uuid::new_v4(),
            report_type: report_type.to_string(),
            key_count: key_count.0,
            keys_needing_rotation: keys_needing_rotation.0,
            average_compliance_score: average_score.0,
            generated_at: Utc::now(),
            findings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_usage_entry_serialization() {
        let entry = KeyUsageEntryV20 {
            id: Uuid::nil(),
            key_id: Uuid::nil(),
            operation: "encrypt".to_string(),
            success: true,
            error_message: None,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deser: KeyUsageEntryV20 = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.operation, "encrypt");
        assert!(deser.success);
    }

    #[test]
    fn test_rotation_schedule_entry_serialization() {
        let entry = RotationScheduleEntryV20 {
            id: Uuid::nil(),
            key_id: Uuid::nil(),
            rotation_days: 90,
            last_rotated_at: None,
            next_rotation_at: Some(Utc::now()),
            enabled: true,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deser: RotationScheduleEntryV20 = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.rotation_days, 90);
        assert!(deser.enabled);
    }

    #[test]
    fn test_key_performance_metrics_serialization() {
        let metrics = KeyPerformanceMetricsV22 {
            key_id: Uuid::nil(),
            key_name: "test-key".to_string(),
            total_operations: 100,
            successful_operations: 95,
            failed_operations: 5,
            success_rate: 95.0,
            avg_operation_time_ms: 12.5,
            last_operation_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&metrics).unwrap();
        let deser: KeyPerformanceMetricsV22 = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.total_operations, 100);
        assert_eq!(deser.success_rate, 95.0);
    }
}
