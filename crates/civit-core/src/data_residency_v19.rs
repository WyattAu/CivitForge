#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidencyReportEntryV17 {
    pub id: Uuid,
    pub report_type: String,
    pub findings: serde_json::Value,
    pub score: i32,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidencyComplianceEntryV17 {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub compliance_status: String,
    pub last_checked_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationResolutionV17 {
    pub violation_id: Uuid,
    pub resolution_type: String,
    pub details: serde_json::Value,
    pub resolved_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidencyAuditLogEntryV17 {
    pub id: Uuid,
    pub action: String,
    pub rule_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub details: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

pub struct DataResidencyV19Service {
    pool: PgPool,
}

impl DataResidencyV19Service {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn generate_report(
        &self,
        report_type: &str,
    ) -> Result<ResidencyReportEntryV17, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct ReportRow {
            id: Uuid,
            report_type: String,
            findings: serde_json::Value,
            score: i32,
            generated_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, ReportRow>(
            r#"INSERT INTO data_residency_reports_v17 (report_type)
             VALUES ($1)
             RETURNING id, report_type, findings, score, generated_at"#,
        )
        .bind(report_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(ResidencyReportEntryV17 {
            id: row.id,
            report_type: row.report_type,
            findings: row.findings,
            score: row.score,
            generated_at: row.generated_at,
        })
    }

    pub async fn get_reports(
        &self,
        report_type: Option<&str>,
    ) -> Result<Vec<ResidencyReportEntryV17>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct ReportRow {
            id: Uuid,
            report_type: String,
            findings: serde_json::Value,
            score: i32,
            generated_at: DateTime<Utc>,
        }

        let rows = match report_type {
            Some(rt) => {
                sqlx::query_as::<_, ReportRow>(
                    r#"SELECT id, report_type, findings, score, generated_at
                     FROM data_residency_reports_v17
                     WHERE report_type = $1
                     ORDER BY generated_at DESC"#,
                )
                .bind(rt)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, ReportRow>(
                    r#"SELECT id, report_type, findings, score, generated_at
                     FROM data_residency_reports_v17
                     ORDER BY generated_at DESC"#,
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(rows
            .into_iter()
            .map(|r| ResidencyReportEntryV17 {
                id: r.id,
                report_type: r.report_type,
                findings: r.findings,
                score: r.score,
                generated_at: r.generated_at,
            })
            .collect())
    }

    pub async fn track_compliance(
        &self,
        rule_id: Uuid,
    ) -> Result<ResidencyComplianceEntryV17, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct ComplianceRow {
            id: Uuid,
            rule_id: Uuid,
            compliance_status: String,
            last_checked_at: DateTime<Utc>,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, ComplianceRow>(
            r#"INSERT INTO data_residency_compliance_v17 (rule_id)
             VALUES ($1)
             RETURNING id, rule_id, compliance_status, last_checked_at, created_at"#,
        )
        .bind(rule_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(ResidencyComplianceEntryV17 {
            id: row.id,
            rule_id: row.rule_id,
            compliance_status: row.compliance_status,
            last_checked_at: row.last_checked_at,
            created_at: row.created_at,
        })
    }

    pub async fn update_compliance_status(
        &self,
        compliance_id: Uuid,
        status: &str,
    ) -> Result<ResidencyComplianceEntryV17, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct ComplianceRow {
            id: Uuid,
            rule_id: Uuid,
            compliance_status: String,
            last_checked_at: DateTime<Utc>,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, ComplianceRow>(
            r#"UPDATE data_residency_compliance_v17
             SET compliance_status = $2, last_checked_at = NOW()
             WHERE id = $1
             RETURNING id, rule_id, compliance_status, last_checked_at, created_at"#,
        )
        .bind(compliance_id)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;

        Ok(ResidencyComplianceEntryV17 {
            id: row.id,
            rule_id: row.rule_id,
            compliance_status: row.compliance_status,
            last_checked_at: row.last_checked_at,
            created_at: row.created_at,
        })
    }

    pub async fn get_compliance_by_rule(
        &self,
        rule_id: Uuid,
    ) -> Result<Vec<ResidencyComplianceEntryV17>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct ComplianceRow {
            id: Uuid,
            rule_id: Uuid,
            compliance_status: String,
            last_checked_at: DateTime<Utc>,
            created_at: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, ComplianceRow>(
            r#"SELECT id, rule_id, compliance_status, last_checked_at, created_at
             FROM data_residency_compliance_v17
             WHERE rule_id = $1
             ORDER BY last_checked_at DESC"#,
        )
        .bind(rule_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ResidencyComplianceEntryV17 {
                id: r.id,
                rule_id: r.rule_id,
                compliance_status: r.compliance_status,
                last_checked_at: r.last_checked_at,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn resolve_violation(
        &self,
        violation_id: Uuid,
        resolution_type: &str,
        details: serde_json::Value,
    ) -> Result<ViolationResolutionV17, sqlx::Error> {
        let _ = sqlx::query(
            r#"UPDATE data_residency_violations
             SET status = 'resolved'
             WHERE id = $1"#,
        )
        .bind(violation_id)
        .execute(&self.pool)
        .await?;

        Ok(ViolationResolutionV17 {
            violation_id,
            resolution_type: resolution_type.to_string(),
            details,
            resolved_at: Utc::now(),
        })
    }

    pub async fn log_audit(
        &self,
        action: &str,
        rule_id: Option<Uuid>,
        user_id: Option<Uuid>,
        details: serde_json::Value,
    ) -> Result<ResidencyAuditLogEntryV17, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AuditRow {
            id: Uuid,
            action: String,
            rule_id: Option<Uuid>,
            user_id: Option<Uuid>,
            details: serde_json::Value,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, AuditRow>(
            r#"INSERT INTO data_residency_audit_logs (action, rule_id, user_id, details)
             VALUES ($1, $2, $3, $4)
             RETURNING id, action, rule_id, user_id, details, created_at"#,
        )
        .bind(action)
        .bind(rule_id)
        .bind(user_id)
        .bind(details)
        .fetch_one(&self.pool)
        .await?;

        Ok(ResidencyAuditLogEntryV17 {
            id: row.id,
            action: row.action,
            rule_id: row.rule_id,
            user_id: row.user_id,
            details: row.details,
            created_at: row.created_at,
        })
    }
}
