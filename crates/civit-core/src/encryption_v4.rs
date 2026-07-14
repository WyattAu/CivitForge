#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyVersionEntry {
    pub id: Uuid,
    pub key_id: Uuid,
    pub version: i32,
    pub key_material: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheckEntry {
    pub id: Uuid,
    pub check_type: String,
    pub status: String,
    pub findings: serde_json::Value,
    pub score: i32,
    pub created_at: DateTime<Utc>,
}

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
}

pub struct EncryptionV4Service {
    pool: PgPool,
}

impl EncryptionV4Service {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

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
            r#"INSERT INTO encryption_key_versions (key_id, version, key_material)
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
             FROM encryption_key_versions
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
            r#"INSERT INTO encryption_compliance_checks (check_type)
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
            r#"UPDATE encryption_compliance_checks
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
                     FROM encryption_compliance_checks
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
                     FROM encryption_compliance_checks
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
                r#"SELECT COUNT(*) FROM encryption_key_versions WHERE key_id = $1"#,
            )
            .bind(key_id)
            .fetch_one(&self.pool)
            .await?;

            let compliance_score: (i32,) = sqlx::query_as(
                r#"SELECT COALESCE(AVG(score), 0) FROM encryption_compliance_checks"#,
            )
            .fetch_one(&self.pool)
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
            }))
        } else {
            Ok(None)
        }
    }
}