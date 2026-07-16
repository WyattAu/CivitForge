#![forbid(unsafe_code)]

use crate::api::AppState;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLogV20 {
    pub id: Uuid,
    pub data_category: String,
    pub source_region: String,
    pub target_region: String,
    pub action: String,
    pub user_id: Uuid,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PolicyV20 {
    pub id: Uuid,
    pub data_category: String,
    pub allowed_regions: Vec<String>,
    pub encryption_required: bool,
    pub retention_days: Option<i32>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogAuditRequest {
    pub data_category: String,
    pub source_region: String,
    pub target_region: String,
    pub action: String,
    pub user_id: Uuid,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePolicyRequest {
    pub data_category: String,
    pub allowed_regions: Vec<String>,
    pub encryption_required: bool,
    pub retention_days: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditLogResponseV20 {
    pub id: Uuid,
    pub data_category: String,
    pub source_region: String,
    pub target_region: String,
    pub action: String,
    pub user_id: Uuid,
    pub metadata: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PolicyResponseV20 {
    pub id: Uuid,
    pub data_category: String,
    pub allowed_regions: Vec<String>,
    pub encryption_required: bool,
    pub retention_days: Option<i32>,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegionAnalyticsResponseV21 {
    pub region: String,
    pub total_transfers: i64,
    pub data_categories: Vec<String>,
    pub compliance_score: f64,
    pub last_transfer_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComplianceReportResponseV21 {
    pub report_id: Uuid,
    pub report_type: String,
    pub total_policies: i64,
    pub enabled_policies: i64,
    pub total_audit_entries: i64,
    pub violations: i64,
    pub compliance_percentage: f64,
    pub generated_at: String,
    pub findings: serde_json::Value,
}

impl From<AuditLogV20> for AuditLogResponseV20 {
    fn from(a: AuditLogV20) -> Self {
        Self {
            id: a.id,
            data_category: a.data_category,
            source_region: a.source_region,
            target_region: a.target_region,
            action: a.action,
            user_id: a.user_id,
            metadata: a.metadata,
            created_at: a.created_at.to_rfc3339(),
        }
    }
}

impl From<PolicyV20> for PolicyResponseV20 {
    fn from(p: PolicyV20) -> Self {
        Self {
            id: p.id,
            data_category: p.data_category,
            allowed_regions: p.allowed_regions,
            encryption_required: p.encryption_required,
            retention_days: p.retention_days,
            enabled: p.enabled,
            created_at: p.created_at.to_rfc3339(),
        }
    }
}

async fn log_audit(
    State(state): State<AppState>,
    Json(input): Json<LogAuditRequest>,
) -> Result<(StatusCode, Json<AuditLogResponseV20>), Response> {
    let audit = sqlx::query_as::<_, AuditLogV20>(
        r#"INSERT INTO data_residency_audit_logs_v20 (data_category, source_region, target_region, action, user_id, metadata)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, data_category, source_region, target_region, action, user_id, metadata, created_at"#,
    )
    .bind(&input.data_category)
    .bind(&input.source_region)
    .bind(&input.target_region)
    .bind(&input.action)
    .bind(input.user_id)
    .bind(input.metadata.unwrap_or(serde_json::json!({})))
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(audit.into())))
}

async fn get_audit_logs(
    State(state): State<AppState>,
) -> Result<Json<Vec<AuditLogResponseV20>>, Response> {
    let logs = sqlx::query_as::<_, AuditLogV20>(
        r#"SELECT id, data_category, source_region, target_region, action, user_id, metadata, created_at
         FROM data_residency_audit_logs_v20
         ORDER BY created_at DESC"#,
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(logs.into_iter().map(|l| l.into()).collect()))
}

async fn create_policy(
    State(state): State<AppState>,
    Json(input): Json<CreatePolicyRequest>,
) -> Result<(StatusCode, Json<PolicyResponseV20>), Response> {
    let policy = sqlx::query_as::<_, PolicyV20>(
        r#"INSERT INTO data_residency_policies_v20 (data_category, allowed_regions, encryption_required, retention_days)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (data_category) DO UPDATE SET allowed_regions = $2, encryption_required = $3, retention_days = $4
         RETURNING id, data_category, allowed_regions, encryption_required, retention_days, enabled, created_at"#,
    )
    .bind(&input.data_category)
    .bind(&input.allowed_regions)
    .bind(input.encryption_required)
    .bind(input.retention_days)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(policy.into())))
}

async fn get_policies(
    State(state): State<AppState>,
) -> Result<Json<Vec<PolicyResponseV20>>, Response> {
    let policies = sqlx::query_as::<_, PolicyV20>(
        r#"SELECT id, data_category, allowed_regions, encryption_required, retention_days, enabled, created_at
         FROM data_residency_policies_v20
         ORDER BY data_category"#,
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(policies.into_iter().map(|p| p.into()).collect()))
}

async fn enforce_policy(
    State(state): State<AppState>,
    Path((data_category, source_region, target_region)): Path<(String, String, String)>,
) -> Result<Json<serde_json::Value>, Response> {
    #[derive(sqlx::FromRow)]
    struct PolicyRow {
        allowed_regions: Vec<String>,
        encryption_required: bool,
    }

    let policy = sqlx::query_as::<_, PolicyRow>(
        r#"SELECT allowed_regions, encryption_required
         FROM data_residency_policies_v20
         WHERE data_category = $1 AND enabled = true"#,
    )
    .bind(&data_category)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    match policy {
        Some(p) => {
            let region_allowed = p.allowed_regions.is_empty()
                || p.allowed_regions.contains(&target_region);
            Ok(Json(serde_json::json!({
                "allowed": region_allowed,
                "encryption_required": p.encryption_required,
                "data_category": data_category,
                "source_region": source_region,
                "target_region": target_region,
            })))
        }
        None => Ok(Json(serde_json::json!({
            "allowed": true,
            "encryption_required": false,
            "data_category": data_category,
            "source_region": source_region,
            "target_region": target_region,
        }))),
    }
}

async fn get_region_analytics(
    State(state): State<AppState>,
) -> Result<Json<Vec<RegionAnalyticsResponseV21>>, Response> {
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
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let mut analytics = Vec::new();
    for row in rows {
        let categories: Vec<String> = sqlx::query_scalar(
            r#"SELECT DISTINCT data_category FROM data_residency_audit_logs_v20 WHERE source_region = $1"#,
        )
        .bind(&row.source_region)
        .fetch_all(state.db.pool())
        .await
            .unwrap_or_default();

        analytics.push(RegionAnalyticsResponseV21 {
            region: row.source_region,
            total_transfers: row.total_transfers,
            data_categories: categories,
            compliance_score: 100.0,
            last_transfer_at: row.last_transfer_at.map(|t| t.to_rfc3339()),
        });
    }

    Ok(Json(analytics))
}

async fn generate_compliance_report(
    State(state): State<AppState>,
) -> Result<Json<ComplianceReportResponseV21>, Response> {
    let total_policies: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM data_residency_policies_v20"#,
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let enabled_policies: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM data_residency_policies_v20 WHERE enabled = true"#,
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let total_audit_entries: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM data_residency_audit_logs_v20"#,
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let compliance_percentage = if total_policies.0 > 0 {
        ((enabled_policies.0 as f64 / total_policies.0 as f64) * 100.0).min(100.0)
    } else {
        100.0
    };

    Ok(Json(ComplianceReportResponseV21 {
        report_id: Uuid::new_v4(),
        report_type: "compliance".to_string(),
        total_policies: total_policies.0,
        enabled_policies: enabled_policies.0,
        total_audit_entries: total_audit_entries.0,
        violations: 0,
        compliance_percentage,
        generated_at: Utc::now().to_rfc3339(),
        findings: serde_json::json!({
            "total_policies": total_policies.0,
            "enabled_policies": enabled_policies.0,
            "total_audit_entries": total_audit_entries.0,
            "compliance_percentage": compliance_percentage,
        }),
    }))
}

pub fn data_residency_v11_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/data-residency-v11/audit-logs",
            post(log_audit).get(get_audit_logs),
        )
        .route(
            "/api/v1/data-residency-v11/policies",
            post(create_policy).get(get_policies),
        )
        .route(
            "/api/v1/data-residency-v11/enforce/{category}/{source}/{target}",
            get(enforce_policy),
        )
        .route(
            "/api/v1/data-residency-v11/region-analytics",
            get(get_region_analytics),
        )
        .route(
            "/api/v1/data-residency-v11/compliance-report",
            get(generate_compliance_report),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_response_conversion() {
        let audit = AuditLogV20 {
            id: Uuid::nil(),
            data_category: "pii".to_string(),
            source_region: "us-east".to_string(),
            target_region: "eu-west".to_string(),
            action: "transfer".to_string(),
            user_id: Uuid::nil(),
            metadata: serde_json::json!({"size": 1024}),
            created_at: Utc::now(),
        };
        let response: AuditLogResponseV20 = audit.into();
        assert_eq!(response.data_category, "pii");
        assert_eq!(response.action, "transfer");
    }

    #[test]
    fn test_policy_response_conversion() {
        let policy = PolicyV20 {
            id: Uuid::nil(),
            data_category: "financial".to_string(),
            allowed_regions: vec!["us-east".to_string(), "eu-west".to_string()],
            encryption_required: true,
            retention_days: Some(365),
            enabled: true,
            created_at: Utc::now(),
        };
        let response: PolicyResponseV20 = policy.into();
        assert_eq!(response.allowed_regions.len(), 2);
        assert!(response.encryption_required);
    }
}
