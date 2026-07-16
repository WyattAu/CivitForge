#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequestEntryV21 {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheckEntryV21 {
    pub id: Uuid,
    pub data_category: String,
    pub region: String,
    pub check_type: String,
    pub result: String,
    pub details: serde_json::Value,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionAnalyticsV22 {
    pub region: String,
    pub total_transfers: i64,
    pub pending_requests: i64,
    pub completed_transfers: i64,
    pub data_categories: Vec<String>,
    pub compliance_score: f64,
    pub last_transfer_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReportV22 {
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

pub struct DataResidencyV22Service {
    pool: PgPool,
}

impl DataResidencyV22Service {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_transfer_request(
        &self,
        data_category: &str,
        source_region: &str,
        target_region: &str,
        data_identifiers: serde_json::Value,
        requested_by: Uuid,
    ) -> Result<TransferRequestEntryV21, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct RequestRow {
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

        let row = sqlx::query_as::<_, RequestRow>(
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

        Ok(TransferRequestEntryV21 {
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
    ) -> Result<TransferRequestEntryV21, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct RequestRow {
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

        let row = sqlx::query_as::<_, RequestRow>(
            r#"UPDATE data_residency_transfer_requests_v21
             SET status = 'approved', approved_by = $2
             WHERE id = $1 AND status = 'pending'
             RETURNING id, data_category, source_region, target_region, data_identifiers, status, requested_by, approved_by, created_at, completed_at"#,
        )
        .bind(request_id)
        .bind(approved_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(TransferRequestEntryV21 {
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
    ) -> Result<TransferRequestEntryV21, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct RequestRow {
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

        let row = sqlx::query_as::<_, RequestRow>(
            r#"UPDATE data_residency_transfer_requests_v21
             SET status = 'completed', completed_at = NOW()
             WHERE id = $1 AND status = 'approved'
             RETURNING id, data_category, source_region, target_region, data_identifiers, status, requested_by, approved_by, created_at, completed_at"#,
        )
        .bind(request_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TransferRequestEntryV21 {
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
    ) -> Result<Vec<TransferRequestEntryV21>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct RequestRow {
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
                sqlx::query_as::<_, RequestRow>(
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
                sqlx::query_as::<_, RequestRow>(
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
                sqlx::query_as::<_, RequestRow>(
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
                sqlx::query_as::<_, RequestRow>(
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
            .map(|r| TransferRequestEntryV21 {
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

    pub async fn run_compliance_check(
        &self,
        data_category: &str,
        region: &str,
        check_type: &str,
    ) -> Result<ComplianceCheckEntryV21, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct CheckRow {
            id: Uuid,
            data_category: String,
            region: String,
            check_type: String,
            result: String,
            details: serde_json::Value,
            checked_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, CheckRow>(
            r#"INSERT INTO data_residency_compliance_checks_v21 (data_category, region, check_type)
             VALUES ($1, $2, $3)
             RETURNING id, data_category, region, check_type, result, details, checked_at"#,
        )
        .bind(data_category)
        .bind(region)
        .bind(check_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(ComplianceCheckEntryV21 {
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
    ) -> Result<Vec<ComplianceCheckEntryV21>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct CheckRow {
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
                sqlx::query_as::<_, CheckRow>(
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
                sqlx::query_as::<_, CheckRow>(
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
                sqlx::query_as::<_, CheckRow>(
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
                sqlx::query_as::<_, CheckRow>(
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
            .map(|r| ComplianceCheckEntryV21 {
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

    pub async fn get_region_analytics(
        &self,
    ) -> Result<Vec<RegionAnalyticsV22>, sqlx::Error> {
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

            analytics.push(RegionAnalyticsV22 {
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

    pub async fn generate_compliance_report(
        &self,
        report_type: &str,
    ) -> Result<ComplianceReportV22, sqlx::Error> {
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

        Ok(ComplianceReportV22 {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_request_entry_serialization() {
        let entry = TransferRequestEntryV21 {
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
        let deser: TransferRequestEntryV21 = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.data_category, "pii");
        assert_eq!(deser.status, "pending");
    }

    #[test]
    fn test_compliance_check_entry_serialization() {
        let entry = ComplianceCheckEntryV21 {
            id: Uuid::nil(),
            data_category: "financial".to_string(),
            region: "eu-west".to_string(),
            check_type: "encryption".to_string(),
            result: "passed".to_string(),
            details: serde_json::json!({"encrypted": true}),
            checked_at: Utc::now(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deser: ComplianceCheckEntryV21 = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.result, "passed");
    }

    #[test]
    fn test_region_analytics_serialization() {
        let analytics = RegionAnalyticsV22 {
            region: "us-east".to_string(),
            total_transfers: 100,
            pending_requests: 5,
            completed_transfers: 90,
            data_categories: vec!["pii".to_string(), "financial".to_string()],
            compliance_score: 95.0,
            last_transfer_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&analytics).unwrap();
        let deser: RegionAnalyticsV22 = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.total_transfers, 100);
        assert_eq!(deser.compliance_score, 95.0);
    }

    #[test]
    fn test_compliance_report_serialization() {
        let report = ComplianceReportV22 {
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
        let deser: ComplianceReportV22 = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.total_regions, 3);
        assert_eq!(deser.overall_compliance_score, 90.0);
    }
}
