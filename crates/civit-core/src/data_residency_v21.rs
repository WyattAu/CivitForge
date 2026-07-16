#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntryV20 {
    pub id: Uuid,
    pub data_category: String,
    pub source_region: String,
    pub target_region: String,
    pub action: String,
    pub user_id: Uuid,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEntryV20 {
    pub id: Uuid,
    pub data_category: String,
    pub allowed_regions: Vec<String>,
    pub encryption_required: bool,
    pub retention_days: Option<i32>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionAnalyticsV21 {
    pub region: String,
    pub total_transfers: i64,
    pub data_categories: Vec<String>,
    pub compliance_score: f64,
    pub last_transfer_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReportV21 {
    pub report_id: Uuid,
    pub report_type: String,
    pub total_policies: i64,
    pub enabled_policies: i64,
    pub total_audit_entries: i64,
    pub violations: i64,
    pub compliance_percentage: f64,
    pub generated_at: DateTime<Utc>,
    pub findings: serde_json::Value,
}

pub struct DataResidencyV21Service {
    pool: PgPool,
}

impl DataResidencyV21Service {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn log_audit(
        &self,
        data_category: &str,
        source_region: &str,
        target_region: &str,
        action: &str,
        user_id: Uuid,
        metadata: serde_json::Value,
    ) -> Result<AuditLogEntryV20, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AuditRow {
            id: Uuid,
            data_category: String,
            source_region: String,
            target_region: String,
            action: String,
            user_id: Uuid,
            metadata: serde_json::Value,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, AuditRow>(
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

        Ok(AuditLogEntryV20 {
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
    ) -> Result<Vec<AuditLogEntryV20>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AuditRow {
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
                sqlx::query_as::<_, AuditRow>(
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
                sqlx::query_as::<_, AuditRow>(
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
                sqlx::query_as::<_, AuditRow>(
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
                sqlx::query_as::<_, AuditRow>(
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
            .map(|r| AuditLogEntryV20 {
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

    pub async fn create_policy(
        &self,
        data_category: &str,
        allowed_regions: Vec<String>,
        encryption_required: bool,
        retention_days: Option<i32>,
    ) -> Result<PolicyEntryV20, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct PolicyRow {
            id: Uuid,
            data_category: String,
            allowed_regions: Vec<String>,
            encryption_required: bool,
            retention_days: Option<i32>,
            enabled: bool,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, PolicyRow>(
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

        Ok(PolicyEntryV20 {
            id: row.id,
            data_category: row.data_category,
            allowed_regions: row.allowed_regions,
            encryption_required: row.encryption_required,
            retention_days: row.retention_days,
            enabled: row.enabled,
            created_at: row.created_at,
        })
    }

    pub async fn get_policies(
        &self,
    ) -> Result<Vec<PolicyEntryV20>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct PolicyRow {
            id: Uuid,
            data_category: String,
            allowed_regions: Vec<String>,
            encryption_required: bool,
            retention_days: Option<i32>,
            enabled: bool,
            created_at: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, PolicyRow>(
            r#"SELECT id, data_category, allowed_regions, encryption_required, retention_days, enabled, created_at
             FROM data_residency_policies_v20
             ORDER BY data_category"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| PolicyEntryV20 {
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
        struct PolicyRow {
            allowed_regions: Vec<String>,
            #[allow(dead_code)]
            encryption_required: bool,
        }

        let policy = sqlx::query_as::<_, PolicyRow>(
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

    pub async fn get_region_analytics(
        &self,
    ) -> Result<Vec<RegionAnalyticsV21>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct RegionRow {
            source_region: String,
            total_transfers: i64,
            last_transfer_at: Option<DateTime<Utc>>,
        }

        let rows = sqlx::query_as::<_, RegionRow>(
            r#"SELECT source_region, COUNT(*) as total_transfers, MAX(created_at) as last_transfer_at
             FROM data_residency_audit_logs_v20
             GROUP BY source_region
             ORDER BY total_transfers DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut analytics = Vec::new();
        for row in rows {
            let categories: Vec<String> = sqlx::query_scalar(
                r#"SELECT DISTINCT data_category FROM data_residency_audit_logs_v20 WHERE source_region = $1"#,
            )
            .bind(&row.source_region)
            .fetch_all(&self.pool)
            .await
                .unwrap_or_default();

            analytics.push(RegionAnalyticsV21 {
                region: row.source_region,
                total_transfers: row.total_transfers,
                data_categories: categories,
                compliance_score: 100.0,
                last_transfer_at: row.last_transfer_at,
            });
        }

        Ok(analytics)
    }

    pub async fn generate_compliance_report(
        &self,
        report_type: &str,
    ) -> Result<ComplianceReportV21, sqlx::Error> {
        let total_policies: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM data_residency_policies_v20"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let enabled_policies: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM data_residency_policies_v20 WHERE enabled = true"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let total_audit_entries: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM data_residency_audit_logs_v20"#,
        )
        .fetch_one(&self.pool)
        .await?;

        let compliance_percentage = if total_policies.0 > 0 {
            ((enabled_policies.0 as f64 / total_policies.0 as f64) * 100.0).min(100.0)
        } else {
            100.0
        };

        let findings = serde_json::json!({
            "total_policies": total_policies.0,
            "enabled_policies": enabled_policies.0,
            "total_audit_entries": total_audit_entries.0,
            "compliance_percentage": compliance_percentage,
        });

        Ok(ComplianceReportV21 {
            report_id: Uuid::new_v4(),
            report_type: report_type.to_string(),
            total_policies: total_policies.0,
            enabled_policies: enabled_policies.0,
            total_audit_entries: total_audit_entries.0,
            violations: 0,
            compliance_percentage,
            generated_at: Utc::now(),
            findings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_entry_serialization() {
        let entry = AuditLogEntryV20 {
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
        let deser: AuditLogEntryV20 = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.data_category, "pii");
        assert_eq!(deser.action, "transfer");
    }

    #[test]
    fn test_policy_entry_serialization() {
        let entry = PolicyEntryV20 {
            id: Uuid::nil(),
            data_category: "financial".to_string(),
            allowed_regions: vec!["us-east".to_string(), "eu-west".to_string()],
            encryption_required: true,
            retention_days: Some(365),
            enabled: true,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deser: PolicyEntryV20 = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.allowed_regions.len(), 2);
        assert!(deser.encryption_required);
    }

    #[test]
    fn test_region_analytics_serialization() {
        let analytics = RegionAnalyticsV21 {
            region: "us-east".to_string(),
            total_transfers: 100,
            data_categories: vec!["pii".to_string(), "financial".to_string()],
            compliance_score: 95.0,
            last_transfer_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&analytics).unwrap();
        let deser: RegionAnalyticsV21 = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.total_transfers, 100);
        assert_eq!(deser.compliance_score, 95.0);
    }
}
