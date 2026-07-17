#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// ─── Base types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionKey {
    pub id: Uuid,
    pub name: String,
    pub algorithm: String,
    pub rotation_date: Option<DateTime<Utc>>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEncryptionKey {
    pub name: String,
    pub algorithm: Option<String>,
    pub key_material: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPayload {
    pub id: Uuid,
    pub key_id: Uuid,
    pub data_type: String,
    pub data_id: Uuid,
    pub encrypted_data: Vec<u8>,
    pub iv: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

// ─── Key versioning (from v18) ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyVersionEntry {
    pub id: Uuid,
    pub key_id: Uuid,
    pub version: i32,
    pub key_material: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

// ─── Compliance checks (from v18) ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheckEntry {
    pub id: Uuid,
    pub check_type: String,
    pub status: String,
    pub findings: serde_json::Value,
    pub score: i32,
    pub created_at: DateTime<Utc>,
}

// ─── Key lifecycle (from v18) ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyLifecycleManagement {
    pub key_id: Uuid,
    pub key_name: String,
    pub algorithm: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub rotation_date: Option<DateTime<Utc>>,
    pub versions_count: i64,
    pub days_since_creation: i64,
    pub days_since_rotation: Option<i64>,
    pub needs_rotation: bool,
    pub compliance_score: i32,
    pub latest_version: Option<i32>,
}

// ─── Audit log (from v18) ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionAuditEntry {
    pub id: Uuid,
    pub key_id: Uuid,
    pub action: String,
    pub details: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// ─── Key usage tracking (from v20) ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyUsageEntry {
    pub id: Uuid,
    pub key_id: Uuid,
    pub operation: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ─── Rotation schedules (from v20) ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationScheduleEntry {
    pub id: Uuid,
    pub key_id: Uuid,
    pub rotation_days: i32,
    pub last_rotated_at: Option<DateTime<Utc>>,
    pub next_rotation_at: Option<DateTime<Utc>>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

// ─── Performance metrics (from v22) ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPerformanceMetrics {
    pub key_id: Uuid,
    pub key_name: String,
    pub total_operations: i64,
    pub successful_operations: i64,
    pub failed_operations: i64,
    pub success_rate: f64,
    pub avg_operation_time_ms: f64,
    pub last_operation_at: Option<DateTime<Utc>>,
}

// ─── Compliance report (from v22) ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReportEntry {
    pub report_id: Uuid,
    pub report_type: String,
    pub key_count: i64,
    pub keys_needing_rotation: i64,
    pub average_compliance_score: f64,
    pub generated_at: DateTime<Utc>,
    pub findings: serde_json::Value,
}

// ─── Access control (from v21) ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyAccessControlEntry {
    pub id: Uuid,
    pub key_id: Uuid,
    pub principal_type: String,
    pub principal_id: Uuid,
    pub permission: String,
    pub granted_by: Uuid,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

// ─── V21 audit log ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionAuditLogEntry {
    pub id: Uuid,
    pub key_id: Uuid,
    pub operation: String,
    pub principal_id: Uuid,
    pub success: bool,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ─── V23 analytics ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyUsageAnalytics {
    pub key_id: Uuid,
    pub key_name: String,
    pub total_operations: i64,
    pub successful_operations: i64,
    pub failed_operations: i64,
    pub unique_principals: i64,
    pub operations_by_type: serde_json::Value,
    pub last_operation_at: Option<DateTime<Utc>>,
}

// ─── V23 compliance report ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
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

// ─── Service ──────────────────────────────────────────────────────────────────

pub struct EncryptionService {
    pool: PgPool,
}

impl EncryptionService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ── Base key management ────────────────────────────────────────────────

    pub async fn create_key(&self, input: CreateEncryptionKey) -> Result<EncryptionKey, sqlx::Error> {
        let algorithm = input.algorithm.unwrap_or_else(|| "AES-256-GCM".into());

        let row = sqlx::query_as::<_, EncryptionKeyRow>(
            r#"INSERT INTO encryption_keys (name, algorithm, key_material)
             VALUES ($1, $2, $3)
             RETURNING id, name, algorithm, rotation_date, enabled, created_at"#,
        )
        .bind(&input.name)
        .bind(&algorithm)
        .bind(&input.key_material)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_key(&self, id: Uuid) -> Result<Option<EncryptionKey>, sqlx::Error> {
        let row = sqlx::query_as::<_, EncryptionKeyRow>(
            r#"SELECT id, name, algorithm, rotation_date, enabled, created_at
             FROM encryption_keys WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_keys(&self) -> Result<Vec<EncryptionKey>, sqlx::Error> {
        let rows = sqlx::query_as::<_, EncryptionKeyRow>(
            r#"SELECT id, name, algorithm, rotation_date, enabled, created_at
             FROM encryption_keys ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn rotate_key(&self, id: Uuid, new_material: Vec<u8>) -> Result<EncryptionKey, sqlx::Error> {
        let row = sqlx::query_as::<_, EncryptionKeyRow>(
            r#"UPDATE encryption_keys SET
             key_material = $2,
             rotation_date = NOW()
             WHERE id = $1
             RETURNING id, name, algorithm, rotation_date, enabled, created_at"#,
        )
        .bind(id)
        .bind(new_material)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn disable_key(&self, id: Uuid) -> Result<EncryptionKey, sqlx::Error> {
        let row = sqlx::query_as::<_, EncryptionKeyRow>(
            r#"UPDATE encryption_keys SET enabled = false
             WHERE id = $1
             RETURNING id, name, algorithm, rotation_date, enabled, created_at"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn encrypt_data(
        &self,
        key_id: Uuid,
        data_type: &str,
        data_id: Uuid,
        plaintext: &[u8],
    ) -> Result<EncryptedPayload, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct KeyRow {
            key_material: Vec<u8>,
            #[allow(dead_code)]
            algorithm: String,
        }

        let key_row = sqlx::query_as::<_, KeyRow>(
            r#"SELECT key_material, algorithm FROM encryption_keys WHERE id = $1 AND enabled = true"#,
        )
        .bind(key_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let iv = generate_iv();
        let encrypted = encrypt_aes_gcm(plaintext, &key_row.key_material, &iv);

        let row = sqlx::query_as::<_, EncryptedPayloadRow>(
            r#"INSERT INTO encrypted_data (key_id, data_type, data_id, encrypted_data, iv)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, key_id, data_type, data_id, encrypted_data, iv, created_at"#,
        )
        .bind(key_id)
        .bind(data_type)
        .bind(data_id)
        .bind(encrypted)
        .bind(iv)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn decrypt_data(
        &self,
        payload_id: Uuid,
    ) -> Result<Vec<u8>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct DecryptRow {
            encrypted_data: Vec<u8>,
            iv: Vec<u8>,
            key_material: Vec<u8>,
        }

        let row = sqlx::query_as::<_, DecryptRow>(
            r#"SELECT ed.encrypted_data, ed.iv, ek.key_material
             FROM encrypted_data ed
             JOIN encryption_keys ek ON ek.id = ed.key_id
             WHERE ed.id = $1 AND ek.enabled = true"#,
        )
        .bind(payload_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)?;

        Ok(decrypt_aes_gcm(&row.encrypted_data, &row.key_material, &row.iv))
    }

    pub async fn get_encrypted_data_by_reference(
        &self,
        data_type: &str,
        data_id: Uuid,
    ) -> Result<Option<EncryptedPayload>, sqlx::Error> {
        let row = sqlx::query_as::<_, EncryptedPayloadRow>(
            r#"SELECT id, key_id, data_type, data_id, encrypted_data, iv, created_at
             FROM encrypted_data
             WHERE data_type = $1 AND data_id = $2"#,
        )
        .bind(data_type)
        .bind(data_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn needs_rotation(&self, key_id: Uuid, max_age_days: i64) -> Result<bool, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct RotationRow {
            rotation_date: Option<DateTime<Utc>>,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, RotationRow>(
            r#"SELECT rotation_date, created_at FROM encryption_keys WHERE id = $1"#,
        )
        .bind(key_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let last_rotation = row.rotation_date.unwrap_or(row.created_at);
        let age = Utc::now() - last_rotation;
        Ok(age.num_days() > max_age_days)
    }

    // ── Key versioning (v18) ───────────────────────────────────────────────

    pub async fn create_key_version(
        &self,
        key_id: Uuid,
        version: i32,
        key_material: &[u8],
    ) -> Result<KeyVersionEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct VersionRow {
            id: Uuid,
            key_id: Uuid,
            version: i32,
            key_material: Vec<u8>,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, VersionRow>(
            r#"INSERT INTO encryption_key_versions_v18 (key_id, version, key_material)
             VALUES ($1, $2, $3)
             RETURNING id, key_id, version, key_material, created_at"#,
        )
        .bind(key_id)
        .bind(version)
        .bind(key_material)
        .fetch_one(&self.pool)
        .await?;

        Ok(KeyVersionEntry {
            id: row.id,
            key_id: row.key_id,
            version: row.version,
            key_material: row.key_material,
            created_at: row.created_at,
        })
    }

    pub async fn get_key_versions(
        &self,
        key_id: Uuid,
    ) -> Result<Vec<KeyVersionEntry>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct VersionRow {
            id: Uuid,
            key_id: Uuid,
            version: i32,
            key_material: Vec<u8>,
            created_at: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, VersionRow>(
            r#"SELECT id, key_id, version, key_material, created_at
             FROM encryption_key_versions_v18
             WHERE key_id = $1
             ORDER BY version DESC"#,
        )
        .bind(key_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| KeyVersionEntry {
                id: r.id,
                key_id: r.key_id,
                version: r.version,
                key_material: r.key_material,
                created_at: r.created_at,
            })
            .collect())
    }

    // ── Compliance checks (v18) ────────────────────────────────────────────

    pub async fn create_compliance_check(
        &self,
        check_type: &str,
    ) -> Result<ComplianceCheckEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct CheckRow {
            id: Uuid,
            check_type: String,
            status: String,
            findings: serde_json::Value,
            score: i32,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, CheckRow>(
            r#"INSERT INTO encryption_compliance_checks_v18 (check_type)
             VALUES ($1)
             RETURNING id, check_type, status, findings, score, created_at"#,
        )
        .bind(check_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(ComplianceCheckEntry {
            id: row.id,
            check_type: row.check_type,
            status: row.status,
            findings: row.findings,
            score: row.score,
            created_at: row.created_at,
        })
    }

    pub async fn update_compliance_check(
        &self,
        check_id: Uuid,
        status: &str,
        findings: serde_json::Value,
        score: i32,
    ) -> Result<ComplianceCheckEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct CheckRow {
            id: Uuid,
            check_type: String,
            status: String,
            findings: serde_json::Value,
            score: i32,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, CheckRow>(
            r#"UPDATE encryption_compliance_checks_v18
             SET status = $2, findings = $3, score = $4
             WHERE id = $1
             RETURNING id, check_type, status, findings, score, created_at"#,
        )
        .bind(check_id)
        .bind(status)
        .bind(findings)
        .bind(score)
        .fetch_one(&self.pool)
        .await?;

        Ok(ComplianceCheckEntry {
            id: row.id,
            check_type: row.check_type,
            status: row.status,
            findings: row.findings,
            score: row.score,
            created_at: row.created_at,
        })
    }

    pub async fn get_compliance_checks(
        &self,
        check_type: Option<&str>,
    ) -> Result<Vec<ComplianceCheckEntry>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct CheckRow {
            id: Uuid,
            check_type: String,
            status: String,
            findings: serde_json::Value,
            score: i32,
            created_at: DateTime<Utc>,
        }

        let rows = match check_type {
            Some(ct) => {
                sqlx::query_as::<_, CheckRow>(
                    r#"SELECT id, check_type, status, findings, score, created_at
                     FROM encryption_compliance_checks_v18
                     WHERE check_type = $1
                     ORDER BY created_at DESC"#,
                )
                .bind(ct)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, CheckRow>(
                    r#"SELECT id, check_type, status, findings, score, created_at
                     FROM encryption_compliance_checks_v18
                     ORDER BY created_at DESC"#,
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(rows
            .into_iter()
            .map(|r| ComplianceCheckEntry {
                id: r.id,
                check_type: r.check_type,
                status: r.status,
                findings: r.findings,
                score: r.score,
                created_at: r.created_at,
            })
            .collect())
    }

    // ── Key lifecycle management (v18) ─────────────────────────────────────

    pub async fn get_key_lifecycle_management(
        &self,
        key_id: Uuid,
    ) -> Result<Option<KeyLifecycleManagement>, sqlx::Error> {
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

        if let Some(r) = row {
            let versions_count: (i64,) = sqlx::query_as(
                r#"SELECT COUNT(*) FROM encryption_key_versions_v18 WHERE key_id = $1"#,
            )
            .bind(key_id)
            .fetch_one(&self.pool)
            .await?;

            let compliance_score: (i32,) = sqlx::query_as(
                r#"SELECT COALESCE(AVG(score), 0) FROM encryption_compliance_checks_v18"#,
            )
            .fetch_one(&self.pool)
            .await?;

            let latest_version: Option<(Option<i32>,)> = sqlx::query_as(
                r#"SELECT MAX(version) FROM encryption_key_versions_v18 WHERE key_id = $1"#,
            )
            .bind(key_id)
            .fetch_optional(&self.pool)
            .await?;

            let now = Utc::now();
            let days_since_creation = (now - r.created_at).num_days();
            let days_since_rotation = r.rotation_date.map(|rot| (now - rot).num_days());
            let needs_rotation = days_since_rotation.unwrap_or(days_since_creation) > 90;

            Ok(Some(KeyLifecycleManagement {
                key_id: r.id,
                key_name: r.name,
                algorithm: r.algorithm,
                enabled: r.enabled,
                created_at: r.created_at,
                rotation_date: r.rotation_date,
                versions_count: versions_count.0,
                days_since_creation,
                days_since_rotation,
                needs_rotation,
                compliance_score: compliance_score.0,
                latest_version: latest_version.and_then(|v| v.0),
            }))
        } else {
            Ok(None)
        }
    }

    // ── Audit logging (v18) ────────────────────────────────────────────────

    pub async fn log_audit(
        &self,
        key_id: Uuid,
        action: &str,
        details: serde_json::Value,
    ) -> Result<EncryptionAuditEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AuditRow {
            id: Uuid,
            key_id: Uuid,
            action: String,
            details: serde_json::Value,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, AuditRow>(
            r#"INSERT INTO encryption_audit_logs (key_id, action, details)
             VALUES ($1, $2, $3)
             RETURNING id, key_id, action, details, created_at"#,
        )
        .bind(key_id)
        .bind(action)
        .bind(details)
        .fetch_one(&self.pool)
        .await?;

        Ok(EncryptionAuditEntry {
            id: row.id,
            key_id: row.key_id,
            action: row.action,
            details: row.details,
            created_at: row.created_at,
        })
    }

    // ── Key usage tracking (v20) ───────────────────────────────────────────

    pub async fn log_key_usage(
        &self,
        key_id: Uuid,
        operation: &str,
        success: bool,
        error_message: Option<&str>,
    ) -> Result<KeyUsageEntry, sqlx::Error> {
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

        Ok(KeyUsageEntry {
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
    ) -> Result<Vec<KeyUsageEntry>, sqlx::Error> {
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
            .map(|r| KeyUsageEntry {
                id: r.id,
                key_id: r.key_id,
                operation: r.operation,
                success: r.success,
                error_message: r.error_message,
                created_at: r.created_at,
            })
            .collect())
    }

    // ── Rotation schedules (v20) ───────────────────────────────────────────

    pub async fn create_rotation_schedule(
        &self,
        key_id: Uuid,
        rotation_days: i32,
    ) -> Result<RotationScheduleEntry, sqlx::Error> {
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

        Ok(RotationScheduleEntry {
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
    ) -> Result<Vec<RotationScheduleEntry>, sqlx::Error> {
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
            .map(|r| RotationScheduleEntry {
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

    // ── Performance metrics (v22) ──────────────────────────────────────────

    pub async fn get_key_performance_metrics(
        &self,
        key_id: Uuid,
    ) -> Result<Option<KeyPerformanceMetrics>, sqlx::Error> {
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

            Ok(Some(KeyPerformanceMetrics {
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

    // ── Compliance report (v22) ────────────────────────────────────────────

    pub async fn generate_compliance_report_v22(
        &self,
        report_type: &str,
    ) -> Result<ComplianceReportEntry, sqlx::Error> {
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

        Ok(ComplianceReportEntry {
            report_id: Uuid::new_v4(),
            report_type: report_type.to_string(),
            key_count: key_count.0,
            keys_needing_rotation: keys_needing_rotation.0,
            average_compliance_score: average_score.0,
            generated_at: Utc::now(),
            findings,
        })
    }

    // ── Access control (v21) ───────────────────────────────────────────────

    pub async fn grant_key_access(
        &self,
        key_id: Uuid,
        principal_type: &str,
        principal_id: Uuid,
        granted_by: Uuid,
        permission: &str,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<KeyAccessControlEntry, sqlx::Error> {
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

        Ok(KeyAccessControlEntry {
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
    ) -> Result<Vec<KeyAccessControlEntry>, sqlx::Error> {
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
            .map(|r| KeyAccessControlEntry {
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

    // ── V21 audit log ──────────────────────────────────────────────────────

    pub async fn log_encryption_audit(
        &self,
        key_id: Uuid,
        operation: &str,
        principal_id: Uuid,
        success: bool,
        ip_address: Option<&str>,
    ) -> Result<EncryptionAuditLogEntry, sqlx::Error> {
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

        Ok(EncryptionAuditLogEntry {
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
    ) -> Result<Vec<EncryptionAuditLogEntry>, sqlx::Error> {
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
            .map(|r| EncryptionAuditLogEntry {
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

    // ── V23 analytics ──────────────────────────────────────────────────────

    pub async fn get_key_usage_analytics(
        &self,
        key_id: Uuid,
    ) -> Result<Option<KeyUsageAnalytics>, sqlx::Error> {
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

            Ok(Some(KeyUsageAnalytics {
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

    // ── V23 compliance report ──────────────────────────────────────────────

    pub async fn generate_compliance_report(
        &self,
        report_type: &str,
    ) -> Result<ComplianceReport, sqlx::Error> {
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

        Ok(ComplianceReport {
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

// ─── Internal row types ───────────────────────────────────────────────────────

#[derive(Debug, sqlx::FromRow)]
struct EncryptionKeyRow {
    id: Uuid,
    name: String,
    algorithm: String,
    rotation_date: Option<DateTime<Utc>>,
    enabled: bool,
    created_at: DateTime<Utc>,
}

impl From<EncryptionKeyRow> for EncryptionKey {
    fn from(row: EncryptionKeyRow) -> Self {
        EncryptionKey {
            id: row.id,
            name: row.name,
            algorithm: row.algorithm,
            rotation_date: row.rotation_date,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct EncryptedPayloadRow {
    id: Uuid,
    key_id: Uuid,
    data_type: String,
    data_id: Uuid,
    encrypted_data: Vec<u8>,
    iv: Vec<u8>,
    created_at: DateTime<Utc>,
}

impl From<EncryptedPayloadRow> for EncryptedPayload {
    fn from(row: EncryptedPayloadRow) -> Self {
        EncryptedPayload {
            id: row.id,
            key_id: row.key_id,
            data_type: row.data_type,
            data_id: row.data_id,
            encrypted_data: row.encrypted_data,
            iv: row.iv,
            created_at: row.created_at,
        }
    }
}

// ─── Crypto helpers ───────────────────────────────────────────────────────────

fn generate_iv() -> Vec<u8> {
    use rand::RngExt;
    let mut iv = vec![0u8; 12];
    rand::rng().fill(&mut iv[..]);
    iv
}

fn encrypt_aes_gcm(plaintext: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
    use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};

    let cipher = Aes256Gcm::new_from_slice(key).expect("invalid key length");
    let nonce = Nonce::from_slice(iv);
    cipher.encrypt(nonce, plaintext).expect("encryption failed")
}

fn decrypt_aes_gcm(ciphertext: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
    use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};

    let cipher = Aes256Gcm::new_from_slice(key).expect("invalid key length");
    let nonce = Nonce::from_slice(iv);
    cipher.decrypt(nonce, ciphertext).expect("decryption failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iv_generation() {
        let iv1 = generate_iv();
        let iv2 = generate_iv();
        assert_eq!(iv1.len(), 12);
        assert_ne!(iv1, iv2);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = vec![0u8; 32];
        let iv = generate_iv();
        let plaintext = b"hello world";
        let encrypted = encrypt_aes_gcm(plaintext, &key, &iv);
        let decrypted = decrypt_aes_gcm(&encrypted, &key, &iv);
        assert_eq!(plaintext.to_vec(), decrypted);
    }
}
