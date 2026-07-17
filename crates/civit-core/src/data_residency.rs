#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// --- Transfer Request types (from v22) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequestEntry {
    pub id: Uuid,
    pub data_category: String,
    pub source_region: String,
    pub target_region: String,
    pub data_identifiers: serde_json::Value,
    pub status: String,
    pub requested_by: Uuid,
    pub approved_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

// --- Compliance Check types (from v22) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheckEntry {
    pub id: Uuid,
    pub data_category: String,
    pub region: String,
    pub check_type: String,
    pub result: String,
    pub details: serde_json::Value,
    pub checked_at: DateTime<Utc>,
}

// --- Analytics types (from v22) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionAnalytics {
    pub region: String,
    pub total_transfers: i64,
    pub pending_requests: i64,
    pub completed_transfers: i64,
    pub data_categories: Vec<String>,
    pub compliance_score: f64,
    pub last_transfer_at: Option<DateTime<Utc>>,
}

// --- Compliance Report types (from v22) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub report_id: Uuid,
    pub report_type: String,
    pub total_regions: i64,
    pub total_transfers: i64,
    pub pending_transfers: i64,
    pub failed_transfers: i64,
    pub compliance_checks_passed: i64,
    pub compliance_checks_failed: i64,
    pub overall_compliance_score: f64,
    pub generated_at: DateTime<Utc>,
    pub findings: serde_json::Value,
}

// --- Audit Log types (from v21) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub data_category: String,
    pub source_region: String,
    pub target_region: String,
    pub action: String,
    pub user_id: Uuid,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// --- Policy types (from v21) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEntry {
    pub id: Uuid,
    pub data_category: String,
    pub allowed_regions: Vec<String>,
    pub encryption_required: bool,
    pub retention_days: Option<i32>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

// --- Legacy types (from v2, kept for backward compat) ---

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

// --- Legacy compliance tracking types (from v11-v20, kept for backward compat) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidencyReportEntry {
    pub id: Uuid,
    pub report_type: String,
    pub findings: serde_json::Value,
    pub score: i32,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidencyComplianceEntry {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub compliance_status: String,
    pub last_checked_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationResolution {
    pub violation_id: Uuid,
    pub resolution_type: String,
    pub details: serde_json::Value,
    pub resolved_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidencyAuditLogEntry {
    pub id: Uuid,
    pub action: String,
    pub rule_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub details: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// --- Consolidated Service ---

pub struct DataResidencyService {
    pool: PgPool,
}

impl DataResidencyService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // === Transfer Request methods (from v22) ===

    pub async fn create_transfer_request(
        &self,
        data_category: &str,
        source_region: &str,
        target_region: &str,
        data_identifiers: serde_json::Value,
        requested_by: Uuid,
    ) -> Result<TransferRequestEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            data_category: String,
            source_region: String,
            target_region: String,
            data_identifiers: serde_json::Value,
            status: String,
            requested_by: Uuid,
            approved_by: Option<Uuid>,
            created_at: DateTime<Utc>,
            completed_at: Option<DateTime<Utc>>,
        }

        let row = sqlx::query_as::<_, Row>(
            r#"INSERT INTO data_residency_transfer_requests_v21 (data_category, source_region, target_region, data_identifiers, requested_by)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, data_category, source_region, target_region, data_identifiers, status, requested_by, approved_by, created_at, completed_at"#,
        )
        .bind(data_category)
        .bind(source_region)
        .bind(target_region)
        .bind(data_identifiers)
        .bind(requested_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(TransferRequestEntry {
            id: row.id,
            data_category: row.data_category,
            source_region: row.source_region,
            target_region: row.target_region,
            data_identifiers: row.data_identifiers,
            status: row.status,
            requested_by: row.requested_by,
            approved_by: row.approved_by,
            created_at: row.created_at,
            completed_at: row.completed_at,
        })
    }

    pub async fn approve_transfer_request(
        &self,
        request_id: Uuid,
        approved_by: Uuid,
    ) -> Result<TransferRequestEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            data_category: String,
            source_region: String,
            target_region: String,
            data_identifiers: serde_json::Value,
            status: String,
            requested_by: Uuid,
            approved_by: Option<Uuid>,
            created_at: DateTime<Utc>,
            completed_at: Option<DateTime<Utc>>,
        }

        let row = sqlx::query_as::<_, Row>(
            r#"UPDATE data_residency_transfer_requests_v21
             SET status = 'approved', approved_by = $2
             WHERE id = $1 AND status = 'pending'
             RETURNING id, data_category, source_region, target_region, data_identifiers, status, requested_by, approved_by, created_at, completed_at"#,
        )
        .bind(request_id)
        .bind(approved_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(TransferRequestEntry {
            id: row.id,
            data_category: row.data_category,
            source_region: row.source_region,
            target_region: row.target_region,
            data_identifiers: row.data_identifiers,
            status: row.status,
            requested_by: row.requested_by,
            approved_by: row.approved_by,
            created_at: row.created_at,
            completed_at: row.completed_at,
        })
    }

    pub async fn complete_transfer_request(
        &self,
        request_id: Uuid,
    ) -> Result<TransferRequestEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            data_category: String,
            source_region: String,
            target_region: String,
            data_identifiers: serde_json::Value,
            status: String,
            requested_by: Uuid,
            approved_by: Option<Uuid>,
            created_at: DateTime<Utc>,
            completed_at: Option<DateTime<Utc>>,
        }

        let row = sqlx::query_as::<_, Row>(
            r#"UPDATE data_residency_transfer_requests_v21
             SET status = 'completed', completed_at = NOW()
             WHERE id = $1 AND status = 'approved'
             RETURNING id, data_category, source_region, target_region, data_identifiers, status, requested_by, approved_by, created_at, completed_at"#,
        )
        .bind(request_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TransferRequestEntry {
            id: row.id,
            data_category: row.data_category,
            source_region: row.source_region,
            target_region: row.target_region,
            data_identifiers: row.data_identifiers,
            status: row.status,
            requested_by: row.requested_by,
            approved_by: row.approved_by,
            created_at: row.created_at,
            completed_at: row.completed_at,
        })
    }

    pub async fn get_transfer_requests(
        &self,
        status: Option<&str>,
        data_category: Option<&str>,
        limit: i64,
    ) -> Result<Vec<TransferRequestEntry>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            data_category: String,
            source_region: String,
            target_region: String,
            data_identifiers: serde_json::Value,
            status: String,
            requested_by: Uuid,
            approved_by: Option<Uuid>,
            created_at: DateTime<Utc>,
            completed_at: Option<DateTime<Utc>>,
        }

        let rows = match (status, data_category) {
            (Some(s), Some(cat)) => {
                sqlx::query_as::<_, Row>(
                    r#"SELECT id, data_category, source_region, target_region, data_identifiers, status, requested_by, approved_by, created_at, completed_at
                     FROM data_residency_transfer_requests_v21
                     WHERE status = $1 AND data_category = $2
                     ORDER BY created_at DESC
                     LIMIT $3"#,
                )
                .bind(s)
                .bind(cat)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(s), None) => {
                sqlx::query_as::<_, Row>(
                    r#"SELECT id, data_category, source_region, target_region, data_identifiers, status, requested_by, approved_by, created_at, completed_at
                     FROM data_residency_transfer_requests_v21
                     WHERE status = $1
                     ORDER BY created_at DESC
                     LIMIT $2"#,
                )
                .bind(s)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(cat)) => {
                sqlx::query_as::<_, Row>(
                    r#"SELECT id, data_category, source_region, target_region, data_identifiers, status, requested_by, approved_by, created_at, completed_at
                     FROM data_residency_transfer_requests_v21
                     WHERE data_category = $1
                     ORDER BY created_at DESC
                     LIMIT $2"#,
                )
                .bind(cat)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            (None, None) => {
                sqlx::query_as::<_, Row>(
                    r#"SELECT id, data_category, source_region, target_region, data_identifiers, status, requested_by, approved_by, created_at, completed_at
                     FROM data_residency_transfer_requests_v21
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
            .map(|r| TransferRequestEntry {
                id: r.id,
                data_category: r.data_category,
                source_region: r.source_region,
                target_region: r.target_region,
                data_identifiers: r.data_identifiers,
                status: r.status,
                requested_by: r.requested_by,
                approved_by: r.approved_by,
                created_at: r.created_at,
                completed_at: r.completed_at,
            })
            .collect())
    }

    // === Compliance Check methods (from v22) ===

    pub async fn run_compliance_check(
        &self,
        data_category: &str,
        region: &str,
        check_type: &str,
    ) -> Result<ComplianceCheckEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            data_category: String,
            region: String,
            check_type: String,
            result: String,
            details: serde_json::Value,
            checked_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, Row>(
            r#"INSERT INTO data_residency_compliance_checks_v21 (data_category, region, check_type)
             VALUES ($1, $2, $3)
             RETURNING id, data_category, region, check_type, result, details, checked_at"#,
        )
        .bind(data_category)
        .bind(region)
        .bind(check_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(ComplianceCheckEntry {
            id: row.id,
            data_category: row.data_category,
            region: row.region,
            check_type: row.check_type,
            result: row.result,
            details: row.details,
            checked_at: row.checked_at,
        })
    }

    pub async fn get_compliance_checks(
        &self,
        data_category: Option<&str>,
        region: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ComplianceCheckEntry>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            data_category: String,
            region: String,
            check_type: String,
            result: String,
            details: serde_json::Value,
            checked_at: DateTime<Utc>,
        }

        let rows = match (data_category, region) {
            (Some(cat), Some(r)) => {
                sqlx::query_as::<_, Row>(
                    r#"SELECT id, data_category, region, check_type, result, details, checked_at
                     FROM data_residency_compliance_checks_v21
                     WHERE data_category = $1 AND region = $2
                     ORDER BY checked_at DESC
                     LIMIT $3"#,
                )
                .bind(cat)
                .bind(r)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(cat), None) => {
                sqlx::query_as::<_, Row>(
                    r#"SELECT id, data_category, region, check_type, result, details, checked_at
                     FROM data_residency_compliance_checks_v21
                     WHERE data_category = $1
                     ORDER BY checked_at DESC
                     LIMIT $2"#,
                )
                .bind(cat)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(r)) => {
                sqlx::query_as::<_, Row>(
                    r#"SELECT id, data_category, region, check_type, result, details, checked_at
                     FROM data_residency_compliance_checks_v21
                     WHERE region = $1
                     ORDER BY checked_at DESC
                     LIMIT $2"#,
                )
                .bind(r)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            (None, None) => {
                sqlx::query_as::<_, Row>(
                    r#"SELECT id, data_category, region, check_type, result, details, checked_at
                     FROM data_residency_compliance_checks_v21
                     ORDER BY checked_at DESC
                     LIMIT $1"#,
                )
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(rows
            .into_iter()
            .map(|r| ComplianceCheckEntry {
                id: r.id,
                data_category: r.data_category,
                region: r.region,
                check_type: r.check_type,
                result: r.result,
                details: r.details,
                checked_at: r.checked_at,
            })
            .collect())
    }

    // === Analytics methods (from v22) ===

    pub async fn get_region_analytics(&self) -> Result<Vec<RegionAnalytics>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct RegionRow {
            source_region: String,
            total_transfers: i64,
            pending_requests: i64,
            completed_transfers: i64,
            last_transfer_at: Option<DateTime<Utc>>,
        }

        let rows = sqlx::query_as::<_, RegionRow>(
            r#"SELECT source_region,
                    COUNT(*) as total_transfers,
                    COUNT(*) FILTER (WHERE status = 'pending') as pending_requests,
                    COUNT(*) FILTER (WHERE status = 'completed') as completed_transfers,
                    MAX(created_at) as last_transfer_at
             FROM data_residency_transfer_requests_v21
             GROUP BY source_region
             ORDER BY total_transfers DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut analytics = Vec::new();
        for row in rows {
            let categories: Vec<String> = sqlx::query_scalar(
                r#"SELECT DISTINCT data_category FROM data_residency_transfer_requests_v21 WHERE source_region = $1"#,
            )
            .bind(&row.source_region)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default();

            let passed_checks: (i64,) = sqlx::query_as(
                r#"SELECT COUNT(*) FROM data_residency_compliance_checks_v21 WHERE region = $1 AND result = 'passed'"#,
            )
            .bind(&row.source_region)
            .fetch_one(&self.pool)
            .await?;

            let total_checks: (i64,) = sqlx::query_as(
                r#"SELECT COUNT(*) FROM data_residency_compliance_checks_v21 WHERE region = $1"#,
            )
            .bind(&row.source_region)
            .fetch_one(&self.pool)
            .await?;

            let compliance_score = if total_checks.0 > 0 {
                (passed_checks.0 as f64 / total_checks.0 as f64) * 100.0
            } else {
                100.0
            };

            analytics.push(RegionAnalytics {
                region: row.source_region,
                total_transfers: row.total_transfers,
                pending_requests: row.pending_requests,
                completed_transfers: row.completed_transfers,
                data_categories: categories,
                compliance_score,
                last_transfer_at: row.last_transfer_at,
            });
        }

        Ok(analytics)
    }

    // === Compliance Report methods (from v22) ===

    pub async fn generate_compliance_report(
        &self,
        report_type: &str,
    ) -> Result<ComplianceReport, sqlx::Error> {
        let total_regions: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(DISTINCT source_region) FROM data_residency_transfer_requests_v21"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let total_transfers: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM data_residency_transfer_requests_v21"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let pending_transfers: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM data_residency_transfer_requests_v21 WHERE status = 'pending'"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let failed_transfers: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM data_residency_transfer_requests_v21 WHERE status = 'failed'"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let checks_passed: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM data_residency_compliance_checks_v21 WHERE result = 'passed'"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let checks_failed: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM data_residency_compliance_checks_v21 WHERE result = 'failed'"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let total_checks = checks_passed.0 + checks_failed.0;
        let overall_compliance_score = if total_checks > 0 {
            (checks_passed.0 as f64 / total_checks as f64) * 100.0
        } else {
            100.0
        };

        Ok(ComplianceReport {
            report_id: Uuid::new_v4(),
            report_type: report_type.to_string(),
            total_regions: total_regions.0,
            total_transfers: total_transfers.0,
            pending_transfers: pending_transfers.0,
            failed_transfers: failed_transfers.0,
            compliance_checks_passed: checks_passed.0,
            compliance_checks_failed: checks_failed.0,
            overall_compliance_score,
            generated_at: Utc::now(),
            findings: serde_json::json!({
                "total_regions": total_regions.0,
                "total_transfers": total_transfers.0,
                "pending_transfers": pending_transfers.0,
                "failed_transfers": failed_transfers.0,
                "compliance_checks_passed": checks_passed.0,
                "compliance_checks_failed": checks_failed.0,
                "overall_compliance_score": overall_compliance_score,
            }),
        })
    }

    // === Audit Log methods (from v21) ===

    pub async fn log_audit(
        &self,
        data_category: &str,
        source_region: &str,
        target_region: &str,
        action: &str,
        user_id: Uuid,
        metadata: serde_json::Value,
    ) -> Result<AuditLogEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            data_category: String,
            source_region: String,
            target_region: String,
            action: String,
            user_id: Uuid,
            metadata: serde_json::Value,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, Row>(
            r#"INSERT INTO data_residency_audit_logs_v20 (data_category, source_region, target_region, action, user_id, metadata)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, data_category, source_region, target_region, action, user_id, metadata, created_at"#,
        )
        .bind(data_category)
        .bind(source_region)
        .bind(target_region)
        .bind(action)
        .bind(user_id)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await?;

        Ok(AuditLogEntry {
            id: row.id,
            data_category: row.data_category,
            source_region: row.source_region,
            target_region: row.target_region,
            action: row.action,
            user_id: row.user_id,
            metadata: row.metadata,
            created_at: row.created_at,
        })
    }

    pub async fn get_audit_logs(
        &self,
        data_category: Option<&str>,
        source_region: Option<&str>,
    ) -> Result<Vec<AuditLogEntry>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            data_category: String,
            source_region: String,
            target_region: String,
            action: String,
            user_id: Uuid,
            metadata: serde_json::Value,
            created_at: DateTime<Utc>,
        }

        let rows = match (data_category, source_region) {
            (Some(cat), Some(region)) => {
                sqlx::query_as::<_, Row>(
                    r#"SELECT id, data_category, source_region, target_region, action, user_id, metadata, created_at
                     FROM data_residency_audit_logs_v20
                     WHERE data_category = $1 AND source_region = $2
                     ORDER BY created_at DESC"#,
                )
                .bind(cat)
                .bind(region)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(cat), None) => {
                sqlx::query_as::<_, Row>(
                    r#"SELECT id, data_category, source_region, target_region, action, user_id, metadata, created_at
                     FROM data_residency_audit_logs_v20
                     WHERE data_category = $1
                     ORDER BY created_at DESC"#,
                )
                .bind(cat)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(region)) => {
                sqlx::query_as::<_, Row>(
                    r#"SELECT id, data_category, source_region, target_region, action, user_id, metadata, created_at
                     FROM data_residency_audit_logs_v20
                     WHERE source_region = $1
                     ORDER BY created_at DESC"#,
                )
                .bind(region)
                .fetch_all(&self.pool)
                .await?
            }
            (None, None) => {
                sqlx::query_as::<_, Row>(
                    r#"SELECT id, data_category, source_region, target_region, action, user_id, metadata, created_at
                     FROM data_residency_audit_logs_v20
                     ORDER BY created_at DESC"#,
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(rows
            .into_iter()
            .map(|r| AuditLogEntry {
                id: r.id,
                data_category: r.data_category,
                source_region: r.source_region,
                target_region: r.target_region,
                action: r.action,
                user_id: r.user_id,
                metadata: r.metadata,
                created_at: r.created_at,
            })
            .collect())
    }

    // === Policy methods (from v21) ===

    pub async fn create_policy(
        &self,
        data_category: &str,
        allowed_regions: Vec<String>,
        encryption_required: bool,
        retention_days: Option<i32>,
    ) -> Result<PolicyEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            data_category: String,
            allowed_regions: Vec<String>,
            encryption_required: bool,
            retention_days: Option<i32>,
            enabled: bool,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, Row>(
            r#"INSERT INTO data_residency_policies_v20 (data_category, allowed_regions, encryption_required, retention_days)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (data_category) DO UPDATE SET allowed_regions = $2, encryption_required = $3, retention_days = $4
             RETURNING id, data_category, allowed_regions, encryption_required, retention_days, enabled, created_at"#,
        )
        .bind(data_category)
        .bind(&allowed_regions)
        .bind(encryption_required)
        .bind(retention_days)
        .fetch_one(&self.pool)
        .await?;

        Ok(PolicyEntry {
            id: row.id,
            data_category: row.data_category,
            allowed_regions: row.allowed_regions,
            encryption_required: row.encryption_required,
            retention_days: row.retention_days,
            enabled: row.enabled,
            created_at: row.created_at,
        })
    }

    pub async fn get_policies(&self) -> Result<Vec<PolicyEntry>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            data_category: String,
            allowed_regions: Vec<String>,
            encryption_required: bool,
            retention_days: Option<i32>,
            enabled: bool,
            created_at: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, Row>(
            r#"SELECT id, data_category, allowed_regions, encryption_required, retention_days, enabled, created_at
             FROM data_residency_policies_v20
             ORDER BY data_category"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| PolicyEntry {
                id: r.id,
                data_category: r.data_category,
                allowed_regions: r.allowed_regions,
                encryption_required: r.encryption_required,
                retention_days: r.retention_days,
                enabled: r.enabled,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn enforce_policy(
        &self,
        data_category: &str,
        _source_region: &str,
        target_region: &str,
    ) -> Result<bool, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct Row {
            allowed_regions: Vec<String>,
            #[allow(dead_code)]
            encryption_required: bool,
        }

        let policy = sqlx::query_as::<_, Row>(
            r#"SELECT allowed_regions, encryption_required
             FROM data_residency_policies_v20
             WHERE data_category = $1 AND enabled = true"#,
        )
        .bind(data_category)
        .fetch_optional(&self.pool)
        .await?;

        match policy {
            Some(p) => {
                let region_allowed = p.allowed_regions.is_empty()
                    || p.allowed_regions.contains(&target_region.to_string());
                Ok(region_allowed)
            }
            None => Ok(true),
        }
    }

    // === Legacy methods (from v2) ===

    pub async fn create_audit_entry(
        &self,
        rule_id: Uuid,
        audit_type: &str,
    ) -> Result<ResidencyAuditEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            rule_id: Uuid,
            audit_type: String,
            findings: serde_json::Value,
            score: i32,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, Row>(
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
        struct Row {
            id: Uuid,
            violation_id: Uuid,
            target_region: String,
            status: String,
            started_at: DateTime<Utc>,
            completed_at: Option<DateTime<Utc>>,
        }

        let row = sqlx::query_as::<_, Row>(
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

    pub async fn get_compliance_info(&self) -> Result<ResidencyComplianceInfo, sqlx::Error> {
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

    // === Legacy compliance tracking methods (from v11-v20) ===

    pub async fn generate_legacy_report(
        &self,
        report_type: &str,
    ) -> Result<ResidencyReportEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            report_type: String,
            findings: serde_json::Value,
            score: i32,
            generated_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, Row>(
            r#"INSERT INTO data_residency_reports_v17 (report_type)
             VALUES ($1)
             RETURNING id, report_type, findings, score, generated_at"#,
        )
        .bind(report_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(ResidencyReportEntry {
            id: row.id,
            report_type: row.report_type,
            findings: row.findings,
            score: row.score,
            generated_at: row.generated_at,
        })
    }

    pub async fn get_legacy_reports(
        &self,
        report_type: Option<&str>,
    ) -> Result<Vec<ResidencyReportEntry>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            report_type: String,
            findings: serde_json::Value,
            score: i32,
            generated_at: DateTime<Utc>,
        }

        let rows = match report_type {
            Some(rt) => {
                sqlx::query_as::<_, Row>(
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
                sqlx::query_as::<_, Row>(
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
            .map(|r| ResidencyReportEntry {
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
    ) -> Result<ResidencyComplianceEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            rule_id: Uuid,
            compliance_status: String,
            last_checked_at: DateTime<Utc>,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, Row>(
            r#"INSERT INTO data_residency_compliance_v17 (rule_id)
             VALUES ($1)
             RETURNING id, rule_id, compliance_status, last_checked_at, created_at"#,
        )
        .bind(rule_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(ResidencyComplianceEntry {
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
    ) -> Result<ResidencyComplianceEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            rule_id: Uuid,
            compliance_status: String,
            last_checked_at: DateTime<Utc>,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, Row>(
            r#"UPDATE data_residency_compliance_v17
             SET compliance_status = $2, last_checked_at = NOW()
             WHERE id = $1
             RETURNING id, rule_id, compliance_status, last_checked_at, created_at"#,
        )
        .bind(compliance_id)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;

        Ok(ResidencyComplianceEntry {
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
    ) -> Result<Vec<ResidencyComplianceEntry>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            rule_id: Uuid,
            compliance_status: String,
            last_checked_at: DateTime<Utc>,
            created_at: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, Row>(
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
            .map(|r| ResidencyComplianceEntry {
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
    ) -> Result<ViolationResolution, sqlx::Error> {
        let _ = sqlx::query(
            r#"UPDATE data_residency_violations
             SET status = 'resolved'
             WHERE id = $1"#,
        )
        .bind(violation_id)
        .execute(&self.pool)
        .await?;

        Ok(ViolationResolution {
            violation_id,
            resolution_type: resolution_type.to_string(),
            details,
            resolved_at: Utc::now(),
        })
    }

    pub async fn log_audit_legacy(
        &self,
        action: &str,
        rule_id: Option<Uuid>,
        user_id: Option<Uuid>,
        details: serde_json::Value,
    ) -> Result<ResidencyAuditLogEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            action: String,
            rule_id: Option<Uuid>,
            user_id: Option<Uuid>,
            details: serde_json::Value,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, Row>(
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

        Ok(ResidencyAuditLogEntry {
            id: row.id,
            action: row.action,
            rule_id: row.rule_id,
            user_id: row.user_id,
            details: row.details,
            created_at: row.created_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_request_entry_serialization() {
        let entry = TransferRequestEntry {
            id: Uuid::nil(),
            data_category: "pii".to_string(),
            source_region: "us-east".to_string(),
            target_region: "eu-west".to_string(),
            data_identifiers: serde_json::json!({"ids": [1, 2, 3]}),
            status: "pending".to_string(),
            requested_by: Uuid::nil(),
            approved_by: None,
            created_at: Utc::now(),
            completed_at: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deser: TransferRequestEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.data_category, "pii");
        assert_eq!(deser.status, "pending");
    }

    #[test]
    fn test_compliance_check_entry_serialization() {
        let entry = ComplianceCheckEntry {
            id: Uuid::nil(),
            data_category: "financial".to_string(),
            region: "eu-west".to_string(),
            check_type: "encryption".to_string(),
            result: "passed".to_string(),
            details: serde_json::json!({"encrypted": true}),
            checked_at: Utc::now(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deser: ComplianceCheckEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.result, "passed");
    }

    #[test]
    fn test_region_analytics_serialization() {
        let analytics = RegionAnalytics {
            region: "us-east".to_string(),
            total_transfers: 100,
            pending_requests: 5,
            completed_transfers: 90,
            data_categories: vec!["pii".to_string(), "financial".to_string()],
            compliance_score: 95.0,
            last_transfer_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&analytics).unwrap();
        let deser: RegionAnalytics = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.total_transfers, 100);
        assert_eq!(deser.compliance_score, 95.0);
    }

    #[test]
    fn test_compliance_report_serialization() {
        let report = ComplianceReport {
            report_id: Uuid::nil(),
            report_type: "full".to_string(),
            total_regions: 3,
            total_transfers: 500,
            pending_transfers: 10,
            failed_transfers: 2,
            compliance_checks_passed: 45,
            compliance_checks_failed: 5,
            overall_compliance_score: 90.0,
            generated_at: Utc::now(),
            findings: serde_json::json!({}),
        };
        let json = serde_json::to_string(&report).unwrap();
        let deser: ComplianceReport = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.total_regions, 3);
        assert_eq!(deser.overall_compliance_score, 90.0);
    }

    #[test]
    fn test_audit_log_entry_serialization() {
        let entry = AuditLogEntry {
            id: Uuid::nil(),
            data_category: "pii".to_string(),
            source_region: "us-east".to_string(),
            target_region: "eu-west".to_string(),
            action: "transfer".to_string(),
            user_id: Uuid::nil(),
            metadata: serde_json::json!({"size": 1024}),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deser: AuditLogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.data_category, "pii");
        assert_eq!(deser.action, "transfer");
    }

    #[test]
    fn test_policy_entry_serialization() {
        let entry = PolicyEntry {
            id: Uuid::nil(),
            data_category: "financial".to_string(),
            allowed_regions: vec!["us-east".to_string(), "eu-west".to_string()],
            encryption_required: true,
            retention_days: Some(365),
            enabled: true,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deser: PolicyEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.allowed_regions.len(), 2);
        assert!(deser.encryption_required);
    }

    #[test]
    fn test_residency_compliance_info_serialization() {
        let info = ResidencyComplianceInfo {
            total_rules: 10,
            enabled_rules: 8,
            total_violations: 2,
            resolved_violations: 1,
            average_score: 85.0,
            compliance_percentage: 80.0,
        };
        let json = serde_json::to_string(&info).unwrap();
        let deser: ResidencyComplianceInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.total_rules, 10);
        assert_eq!(deser.compliance_percentage, 80.0);
    }
}
