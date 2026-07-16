#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyAccessControlEntryV21 {
    pub id: Uuid,
    pub key_id: Uuid,
    pub principal_type: String,
    pub principal_id: Uuid,
    pub permission: String,
    pub granted_by: Uuid,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionAuditLogEntryV21 {
    pub id: Uuid,
    pub key_id: Uuid,
    pub operation: String,
    pub principal_id: Uuid,
    pub success: bool,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyUsageAnalyticsV23 {
    pub key_id: Uuid,
    pub key_name: String,
    pub total_operations: i64,
    pub successful_operations: i64,
    pub failed_operations: i64,
    pub unique_principals: i64,
    pub operations_by_type: serde_json::Value,
    pub last_operation_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReportV23 {
    pub report_id: Uuid,
    pub report_type: String,
    pub total_keys: i64,
    pub keys_with_access_control: i64,
    pub keys_without_access_control: i64,
    pub expired_permissions: i64,
    pub total_audit_entries: i64,
    pub compliance_score: f64,
    pub generated_at: DateTime<Utc>,
    pub findings: serde_json::Value,
}

pub struct EncryptionV23Service {
    pool: PgPool,
}

impl EncryptionV23Service {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn grant_key_access(
        &self,
        key_id: Uuid,
        principal_type: &str,
        principal_id: Uuid,
        granted_by: Uuid,
        permission: &str,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<KeyAccessControlEntryV21, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AclRow {
            id: Uuid,
            key_id: Uuid,
            principal_type: String,
            principal_id: Uuid,
            permission: String,
            granted_by: Uuid,
            granted_at: DateTime<Utc>,
            expires_at: Option<DateTime<Utc>>,
        }

        let row = sqlx::query_as::<_, AclRow>(
            r#"INSERT INTO encryption_key_access_control_v21 (key_id, principal_type, principal_id, granted_by, permission, expires_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (key_id, principal_type, principal_id) DO UPDATE SET permission = $5, expires_at = $6
             RETURNING id, key_id, principal_type, principal_id, permission, granted_by, granted_at, expires_at"#,
        )
        .bind(key_id)
        .bind(principal_type)
        .bind(principal_id)
        .bind(granted_by)
        .bind(permission)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(KeyAccessControlEntryV21 {
            id: row.id,
            key_id: row.key_id,
            principal_type: row.principal_type,
            principal_id: row.principal_id,
            permission: row.permission,
            granted_by: row.granted_by,
            granted_at: row.granted_at,
            expires_at: row.expires_at,
        })
    }

    pub async fn revoke_key_access(
        &self,
        key_id: Uuid,
        principal_type: &str,
        principal_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"DELETE FROM encryption_key_access_control_v21
             WHERE key_id = $1 AND principal_type = $2 AND principal_id = $3"#,
        )
        .bind(key_id)
        .bind(principal_type)
        .bind(principal_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_key_access_control(
        &self,
        key_id: Uuid,
    ) -> Result<Vec<KeyAccessControlEntryV21>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AclRow {
            id: Uuid,
            key_id: Uuid,
            principal_type: String,
            principal_id: Uuid,
            permission: String,
            granted_by: Uuid,
            granted_at: DateTime<Utc>,
            expires_at: Option<DateTime<Utc>>,
        }

        let rows = sqlx::query_as::<_, AclRow>(
            r#"SELECT id, key_id, principal_type, principal_id, permission, granted_by, granted_at, expires_at
             FROM encryption_key_access_control_v21
             WHERE key_id = $1
             ORDER BY granted_at DESC"#,
        )
        .bind(key_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| KeyAccessControlEntryV21 {
                id: r.id,
                key_id: r.key_id,
                principal_type: r.principal_type,
                principal_id: r.principal_id,
                permission: r.permission,
                granted_by: r.granted_by,
                granted_at: r.granted_at,
                expires_at: r.expires_at,
            })
            .collect())
    }

    pub async fn log_encryption_audit(
        &self,
        key_id: Uuid,
        operation: &str,
        principal_id: Uuid,
        success: bool,
        ip_address: Option<&str>,
    ) -> Result<EncryptionAuditLogEntryV21, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AuditRow {
            id: Uuid,
            key_id: Uuid,
            operation: String,
            principal_id: Uuid,
            success: bool,
            ip_address: Option<String>,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, AuditRow>(
            r#"INSERT INTO encryption_audit_log_v21 (key_id, operation, principal_id, success, ip_address)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, key_id, operation, principal_id, success, ip_address, created_at"#,
        )
        .bind(key_id)
        .bind(operation)
        .bind(principal_id)
        .bind(success)
        .bind(ip_address)
        .fetch_one(&self.pool)
        .await?;

        Ok(EncryptionAuditLogEntryV21 {
            id: row.id,
            key_id: row.key_id,
            operation: row.operation,
            principal_id: row.principal_id,
            success: row.success,
            ip_address: row.ip_address,
            created_at: row.created_at,
        })
    }

    pub async fn get_encryption_audit_logs(
        &self,
        key_id: Option<Uuid>,
        principal_id: Option<Uuid>,
        limit: i64,
    ) -> Result<Vec<EncryptionAuditLogEntryV21>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AuditRow {
            id: Uuid,
            key_id: Uuid,
            operation: String,
            principal_id: Uuid,
            success: bool,
            ip_address: Option<String>,
            created_at: DateTime<Utc>,
        }

        let rows = match (key_id, principal_id) {
            (Some(kid), Some(pid)) => {
                sqlx::query_as::<_, AuditRow>(
                    r#"SELECT id, key_id, operation, principal_id, success, ip_address, created_at
                     FROM encryption_audit_log_v21
                     WHERE key_id = $1 AND principal_id = $2
                     ORDER BY created_at DESC
                     LIMIT $3"#,
                )
                .bind(kid)
                .bind(pid)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(kid), None) => {
                sqlx::query_as::<_, AuditRow>(
                    r#"SELECT id, key_id, operation, principal_id, success, ip_address, created_at
                     FROM encryption_audit_log_v21
                     WHERE key_id = $1
                     ORDER BY created_at DESC
                     LIMIT $2"#,
                )
                .bind(kid)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(pid)) => {
                sqlx::query_as::<_, AuditRow>(
                    r#"SELECT id, key_id, operation, principal_id, success, ip_address, created_at
                     FROM encryption_audit_log_v21
                     WHERE principal_id = $1
                     ORDER BY created_at DESC
                     LIMIT $2"#,
                )
                .bind(pid)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            (None, None) => {
                sqlx::query_as::<_, AuditRow>(
                    r#"SELECT id, key_id, operation, principal_id, success, ip_address, created_at
                     FROM encryption_audit_log_v21
                     ORDER BY created_at DESC
                     LIMIT $1"#,
                )
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(rows
            .into_iter()
            .map(|r| EncryptionAuditLogEntryV21 {
                id: r.id,
                key_id: r.key_id,
                operation: r.operation,
                principal_id: r.principal_id,
                success: r.success,
                ip_address: r.ip_address,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn get_key_usage_analytics(
        &self,
        key_id: Uuid,
    ) -> Result<Option<KeyUsageAnalyticsV23>, sqlx::Error> {
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
                r#"SELECT COUNT(*) FROM encryption_audit_log_v21 WHERE key_id = $1"#,
            )
            .bind(key_id)
            .fetch_one(&self.pool)
            .await?;

            let successful_ops: (i64,) = sqlx::query_as(
                r#"SELECT COUNT(*) FROM encryption_audit_log_v21 WHERE key_id = $1 AND success = true"#,
            )
            .bind(key_id)
            .fetch_one(&self.pool)
            .await?;

            let failed_ops: (i64,) = sqlx::query_as(
                r#"SELECT COUNT(*) FROM encryption_audit_log_v21 WHERE key_id = $1 AND success = false"#,
            )
            .bind(key_id)
            .fetch_one(&self.pool)
            .await?;

            let unique_principals: (i64,) = sqlx::query_as(
                r#"SELECT COUNT(DISTINCT principal_id) FROM encryption_audit_log_v21 WHERE key_id = $1"#,
            )
            .bind(key_id)
            .fetch_one(&self.pool)
            .await?;

            let last_op: Option<(Option<DateTime<Utc>>,)> = sqlx::query_as(
                r#"SELECT MAX(created_at) FROM encryption_audit_log_v21 WHERE key_id = $1"#,
            )
            .bind(key_id)
            .fetch_optional(&self.pool)
            .await?;

            Ok(Some(KeyUsageAnalyticsV23 {
                key_id: k.id,
                key_name: k.name,
                total_operations: total_ops.0,
                successful_operations: successful_ops.0,
                failed_operations: failed_ops.0,
                unique_principals: unique_principals.0,
                operations_by_type: serde_json::json!({}),
                last_operation_at: last_op.and_then(|l| l.0),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn generate_compliance_report(
        &self,
        report_type: &str,
    ) -> Result<ComplianceReportV23, sqlx::Error> {
        let total_keys: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM encryption_keys"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let keys_with_acl: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(DISTINCT key_id) FROM encryption_key_access_control_v21"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let expired_permissions: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM encryption_key_access_control_v21
             WHERE expires_at IS NOT NULL AND expires_at < NOW()"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let total_audit_entries: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM encryption_audit_log_v21"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let keys_without_acl = total_keys.0 - keys_with_acl.0;
        let compliance_score = if total_keys.0 > 0 {
            let acl_coverage = (keys_with_acl.0 as f64 / total_keys.0 as f64) * 100.0;
            let expired_penalty = (expired_permissions.0 as f64 * 5.0).min(30.0);
            (acl_coverage - expired_penalty).max(0.0)
        } else {
            100.0
        };

        Ok(ComplianceReportV23 {
            report_id: Uuid::new_v4(),
            report_type: report_type.to_string(),
            total_keys: total_keys.0,
            keys_with_access_control: keys_with_acl.0,
            keys_without_access_control: keys_without_acl,
            expired_permissions: expired_permissions.0,
            total_audit_entries: total_audit_entries.0,
            compliance_score,
            generated_at: Utc::now(),
            findings: serde_json::json!({
                "total_keys": total_keys.0,
                "keys_with_access_control": keys_with_acl.0,
                "keys_without_access_control": keys_without_acl,
                "expired_permissions": expired_permissions.0,
                "total_audit_entries": total_audit_entries.0,
                "compliance_score": compliance_score,
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_access_control_entry_serialization() {
        let entry = KeyAccessControlEntryV21 {
            id: Uuid::nil(),
            key_id: Uuid::nil(),
            principal_type: "user".to_string(),
            principal_id: Uuid::nil(),
            permission: "use".to_string(),
            granted_by: Uuid::nil(),
            granted_at: Utc::now(),
            expires_at: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deser: KeyAccessControlEntryV21 = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.principal_type, "user");
        assert_eq!(deser.permission, "use");
    }

    #[test]
    fn test_encryption_audit_log_entry_serialization() {
        let entry = EncryptionAuditLogEntryV21 {
            id: Uuid::nil(),
            key_id: Uuid::nil(),
            operation: "encrypt".to_string(),
            principal_id: Uuid::nil(),
            success: true,
            ip_address: Some("192.168.1.1".to_string()),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deser: EncryptionAuditLogEntryV21 = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.operation, "encrypt");
        assert!(deser.success);
    }

    #[test]
    fn test_key_usage_analytics_serialization() {
        let analytics = KeyUsageAnalyticsV23 {
            key_id: Uuid::nil(),
            key_name: "test-key".to_string(),
            total_operations: 200,
            successful_operations: 190,
            failed_operations: 10,
            unique_principals: 5,
            operations_by_type: serde_json::json!({"encrypt": 100, "decrypt": 100}),
            last_operation_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&analytics).unwrap();
        let deser: KeyUsageAnalyticsV23 = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.total_operations, 200);
        assert_eq!(deser.unique_principals, 5);
    }

    #[test]
    fn test_compliance_report_serialization() {
        let report = ComplianceReportV23 {
            report_id: Uuid::nil(),
            report_type: "full".to_string(),
            total_keys: 10,
            keys_with_access_control: 8,
            keys_without_access_control: 2,
            expired_permissions: 1,
            total_audit_entries: 500,
            compliance_score: 75.0,
            generated_at: Utc::now(),
            findings: serde_json::json!({}),
        };
        let json = serde_json::to_string(&report).unwrap();
        let deser: ComplianceReportV23 = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.total_keys, 10);
        assert_eq!(deser.compliance_score, 75.0);
    }
}
