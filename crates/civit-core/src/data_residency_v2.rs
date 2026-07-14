#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidencyAuditEntry {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub audit_type: String,
    pub findings: serde_json::Value,
    pub score: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidencyMigrationEntry {
    pub id: Uuid,
    pub violation_id: Uuid,
    pub target_region: String,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidencyComplianceInfo {
    pub total_rules: i64,
    pub enabled_rules: i64,
    pub total_violations: i64,
    pub resolved_violations: i64,
    pub average_score: f64,
    pub compliance_percentage: f64,
}

pub struct DataResidencyV2Service {
    pool: PgPool,
}

impl DataResidencyV2Service {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_audit(
        &self,
        rule_id: Uuid,
        audit_type: &str,
    ) -> Result<ResidencyAuditEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AuditRow {
            id: Uuid,
            rule_id: Uuid,
            audit_type: String,
            findings: serde_json::Value,
            score: i32,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, AuditRow>(
            r#"INSERT INTO data_residency_audits (rule_id, audit_type)
             VALUES ($1, $2)
             RETURNING id, rule_id, audit_type, findings, score, created_at"#,
        )
        .bind(rule_id)
        .bind(audit_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(ResidencyAuditEntry {
            id: row.id,
            rule_id: row.rule_id,
            audit_type: row.audit_type,
            findings: row.findings,
            score: row.score,
            created_at: row.created_at,
        })
    }

    pub async fn create_migration(
        &self,
        violation_id: Uuid,
        target_region: &str,
    ) -> Result<ResidencyMigrationEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct MigrationRow {
            id: Uuid,
            violation_id: Uuid,
            target_region: String,
            status: String,
            started_at: DateTime<Utc>,
            completed_at: Option<DateTime<Utc>>,
        }

        let row = sqlx::query_as::<_, MigrationRow>(
            r#"INSERT INTO data_residency_migrations (violation_id, target_region)
             VALUES ($1, $2)
             RETURNING id, violation_id, target_region, status, started_at, completed_at"#,
        )
        .bind(violation_id)
        .bind(target_region)
        .fetch_one(&self.pool)
        .await?;

        Ok(ResidencyMigrationEntry {
            id: row.id,
            violation_id: row.violation_id,
            target_region: row.target_region,
            status: row.status,
            started_at: row.started_at,
            completed_at: row.completed_at,
        })
    }

    pub async fn get_compliance(&self) -> Result<ResidencyComplianceInfo, sqlx::Error> {
        let total_rules: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM data_residency_rules"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let enabled_rules: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM data_residency_rules WHERE enabled = true"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let total_violations: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM data_residency_violations"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let resolved_violations: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM data_residency_migrations WHERE status = 'completed'"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let average_score: (f64,) = sqlx::query_as(
            r#"SELECT COALESCE(AVG(score), 0.0) FROM data_residency_audits"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let compliance_percentage = if total_rules.0 > 0 {
            ((enabled_rules.0 as f64 / total_rules.0 as f64) * 100.0).min(100.0)
        } else {
            100.0
        };

        Ok(ResidencyComplianceInfo {
            total_rules: total_rules.0,
            enabled_rules: enabled_rules.0,
            total_violations: total_violations.0,
            resolved_violations: resolved_violations.0,
            average_score: average_score.0,
            compliance_percentage,
        })
    }
}
