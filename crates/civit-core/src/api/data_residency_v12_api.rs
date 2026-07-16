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
pub struct TransferRequestV21 {
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ComplianceCheckV21 {
    pub id: Uuid,
    pub data_category: String,
    pub region: String,
    pub check_type: String,
    pub result: String,
    pub details: serde_json::Value,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTransferRequest {
    pub data_category: String,
    pub source_region: String,
    pub target_region: String,
    pub data_identifiers: serde_json::Value,
    pub requested_by: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RunComplianceCheckRequest {
    pub data_category: String,
    pub region: String,
    pub check_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransferRequestResponseV21 {
    pub id: Uuid,
    pub data_category: String,
    pub source_region: String,
    pub target_region: String,
    pub data_identifiers: serde_json::Value,
    pub status: String,
    pub requested_by: Uuid,
    pub approved_by: Option<Uuid>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComplianceCheckResponseV21 {
    pub id: Uuid,
    pub data_category: String,
    pub region: String,
    pub check_type: String,
    pub result: String,
    pub details: serde_json::Value,
    pub checked_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegionAnalyticsResponseV22 {
    pub region: String,
    pub total_transfers: i64,
    pub pending_requests: i64,
    pub completed_transfers: i64,
    pub data_categories: Vec<String>,
    pub compliance_score: f64,
    pub last_transfer_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComplianceReportResponseV22 {
    pub report_id: Uuid,
    pub report_type: String,
    pub total_regions: i64,
    pub total_transfers: i64,
    pub pending_transfers: i64,
    pub failed_transfers: i64,
    pub compliance_checks_passed: i64,
    pub compliance_checks_failed: i64,
    pub overall_compliance_score: f64,
    pub generated_at: String,
    pub findings: serde_json::Value,
}

impl From<TransferRequestV21> for TransferRequestResponseV21 {
    fn from(t: TransferRequestV21) -> Self {
        Self {
            id: t.id,
            data_category: t.data_category,
            source_region: t.source_region,
            target_region: t.target_region,
            data_identifiers: t.data_identifiers,
            status: t.status,
            requested_by: t.requested_by,
            approved_by: t.approved_by,
            created_at: t.created_at.to_rfc3339(),
            completed_at: t.completed_at.map(|t| t.to_rfc3339()),
        }
    }
}

impl From<ComplianceCheckV21> for ComplianceCheckResponseV21 {
    fn from(c: ComplianceCheckV21) -> Self {
        Self {
            id: c.id,
            data_category: c.data_category,
            region: c.region,
            check_type: c.check_type,
            result: c.result,
            details: c.details,
            checked_at: c.checked_at.to_rfc3339(),
        }
    }
}

async fn create_transfer_request(
    State(state): State<AppState>,
    Json(input): Json<CreateTransferRequest>,
) -> Result<(StatusCode, Json<TransferRequestResponseV21>), Response> {
    let row = sqlx::query_as::<_, TransferRequestV21>(
        r#"INSERT INTO data_residency_transfer_requests_v21 (data_category, source_region, target_region, data_identifiers, requested_by)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, data_category, source_region, target_region, data_identifiers, status, requested_by, approved_by, created_at, completed_at"#,
    )
    .bind(&input.data_category)
    .bind(&input.source_region)
    .bind(&input.target_region)
    .bind(&input.data_identifiers)
    .bind(input.requested_by)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(row.into())))
}

async fn approve_transfer_request(
    State(state): State<AppState>,
    Path(request_id): Path<Uuid>,
    Path(approved_by): Path<Uuid>,
) -> Result<Json<TransferRequestResponseV21>, Response> {
    let row = sqlx::query_as::<_, TransferRequestV21>(
        r#"UPDATE data_residency_transfer_requests_v21
         SET status = 'approved', approved_by = $2
         WHERE id = $1 AND status = 'pending'
         RETURNING id, data_category, source_region, target_region, data_identifiers, status, requested_by, approved_by, created_at, completed_at"#,
    )
    .bind(request_id)
    .bind(approved_by)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(row.into()))
}

async fn get_transfer_requests(
    State(state): State<AppState>,
) -> Result<Json<Vec<TransferRequestResponseV21>>, Response> {
    let rows = sqlx::query_as::<_, TransferRequestV21>(
        r#"SELECT id, data_category, source_region, target_region, data_identifiers, status, requested_by, approved_by, created_at, completed_at
         FROM data_residency_transfer_requests_v21
         ORDER BY created_at DESC
         LIMIT 100"#,
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(rows.into_iter().map(|r| r.into()).collect()))
}

async fn run_compliance_check(
    State(state): State<AppState>,
    Json(input): Json<RunComplianceCheckRequest>,
) -> Result<(StatusCode, Json<ComplianceCheckResponseV21>), Response> {
    let row = sqlx::query_as::<_, ComplianceCheckV21>(
        r#"INSERT INTO data_residency_compliance_checks_v21 (data_category, region, check_type)
         VALUES ($1, $2, $3)
         RETURNING id, data_category, region, check_type, result, details, checked_at"#,
    )
    .bind(&input.data_category)
    .bind(&input.region)
    .bind(&input.check_type)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(row.into())))
}

async fn get_compliance_checks(
    State(state): State<AppState>,
    Path(region): Path<String>,
) -> Result<Json<Vec<ComplianceCheckResponseV21>>, Response> {
    let rows = sqlx::query_as::<_, ComplianceCheckV21>(
        r#"SELECT id, data_category, region, check_type, result, details, checked_at
         FROM data_residency_compliance_checks_v21
         WHERE region = $1
         ORDER BY checked_at DESC
         LIMIT 100"#,
    )
    .bind(&region)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(rows.into_iter().map(|r| r.into()).collect()))
}

async fn get_region_analytics(
    State(state): State<AppState>,
) -> Result<Json<Vec<RegionAnalyticsResponseV22>>, Response> {
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
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let mut analytics = Vec::new();
    for row in rows {
        let categories: Vec<String> = sqlx::query_scalar(
            r#"SELECT DISTINCT data_category FROM data_residency_transfer_requests_v21 WHERE source_region = $1"#,
        )
        .bind(&row.source_region)
        .fetch_all(state.db.pool())
        .await
        .unwrap_or_default();

        let passed_checks: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM data_residency_compliance_checks_v21 WHERE region = $1 AND result = 'passed'"#,
        )
        .bind(&row.source_region)
        .fetch_one(state.db.pool())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

        let total_checks: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM data_residency_compliance_checks_v21 WHERE region = $1"#,
        )
        .bind(&row.source_region)
        .fetch_one(state.db.pool())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

        let compliance_score = if total_checks.0 > 0 {
            (passed_checks.0 as f64 / total_checks.0 as f64) * 100.0
        } else {
            100.0
        };

        analytics.push(RegionAnalyticsResponseV22 {
            region: row.source_region,
            total_transfers: row.total_transfers,
            pending_requests: row.pending_requests,
            completed_transfers: row.completed_transfers,
            data_categories: categories,
            compliance_score,
            last_transfer_at: row.last_transfer_at.map(|t| t.to_rfc3339()),
        });
    }

    Ok(Json(analytics))
}

async fn generate_compliance_report(
    State(state): State<AppState>,
) -> Result<Json<ComplianceReportResponseV22>, Response> {
    let total_regions: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(DISTINCT source_region) FROM data_residency_transfer_requests_v21"#,
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let total_transfers: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM data_residency_transfer_requests_v21"#,
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let pending_transfers: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM data_residency_transfer_requests_v21 WHERE status = 'pending'"#,
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let failed_transfers: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM data_residency_transfer_requests_v21 WHERE status = 'failed'"#,
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let checks_passed: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM data_residency_compliance_checks_v21 WHERE result = 'passed'"#,
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let checks_failed: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM data_residency_compliance_checks_v21 WHERE result = 'failed'"#,
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let total_checks = checks_passed.0 + checks_failed.0;
    let overall_compliance_score = if total_checks > 0 {
        (checks_passed.0 as f64 / total_checks as f64) * 100.0
    } else {
        100.0
    };

    Ok(Json(ComplianceReportResponseV22 {
        report_id: Uuid::new_v4(),
        report_type: "compliance".to_string(),
        total_regions: total_regions.0,
        total_transfers: total_transfers.0,
        pending_transfers: pending_transfers.0,
        failed_transfers: failed_transfers.0,
        compliance_checks_passed: checks_passed.0,
        compliance_checks_failed: checks_failed.0,
        overall_compliance_score,
        generated_at: Utc::now().to_rfc3339(),
        findings: serde_json::json!({
            "total_regions": total_regions.0,
            "total_transfers": total_transfers.0,
            "pending_transfers": pending_transfers.0,
            "failed_transfers": failed_transfers.0,
            "compliance_checks_passed": checks_passed.0,
            "compliance_checks_failed": checks_failed.0,
            "overall_compliance_score": overall_compliance_score,
        }),
    }))
}

pub fn data_residency_v12_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/data-residency-v12/transfer-requests",
            post(create_transfer_request).get(get_transfer_requests),
        )
        .route(
            "/api/v1/data-residency-v12/transfer-requests/{request_id}/approve/{approved_by}",
            post(approve_transfer_request),
        )
        .route(
            "/api/v1/data-residency-v12/compliance-checks",
            post(run_compliance_check),
        )
        .route(
            "/api/v1/data-residency-v12/compliance-checks/{region}",
            get(get_compliance_checks),
        )
        .route(
            "/api/v1/data-residency-v12/region-analytics",
            get(get_region_analytics),
        )
        .route(
            "/api/v1/data-residency-v12/compliance-report",
            get(generate_compliance_report),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_request_response_conversion() {
        let req = TransferRequestV21 {
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
        let response: TransferRequestResponseV21 = req.into();
        assert_eq!(response.data_category, "pii");
        assert_eq!(response.status, "pending");
    }

    #[test]
    fn test_compliance_check_response_conversion() {
        let check = ComplianceCheckV21 {
            id: Uuid::nil(),
            data_category: "financial".to_string(),
            region: "eu-west".to_string(),
            check_type: "encryption".to_string(),
            result: "passed".to_string(),
            details: serde_json::json!({"encrypted": true}),
            checked_at: Utc::now(),
        };
        let response: ComplianceCheckResponseV21 = check.into();
        assert_eq!(response.result, "passed");
    }
}
