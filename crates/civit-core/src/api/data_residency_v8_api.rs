#![forbid(unsafe_code)]

use crate::api::AppState;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ResidencyReportV4 {
    pub id: Uuid,
    pub report_type: String,
    pub findings: serde_json::Value,
    pub score: i32,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ResidencyComplianceV4 {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub compliance_status: String,
    pub last_checked_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateReportRequest {
    pub report_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrackComplianceRequest {
    pub rule_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateComplianceStatusRequest {
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResolveViolationRequest {
    pub resolution_type: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResidencyReportResponseV4 {
    pub id: Uuid,
    pub report_type: String,
    pub findings: serde_json::Value,
    pub score: i32,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResidencyComplianceResponseV4 {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub compliance_status: String,
    pub last_checked_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ViolationResolutionResponse {
    pub violation_id: Uuid,
    pub resolution_type: String,
    pub details: serde_json::Value,
    pub resolved_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResidencyComplianceSummaryResponse {
    pub total_rules: i64,
    pub enabled_rules: i64,
    pub total_violations: i64,
    pub resolved_violations: i64,
    pub average_score: f64,
    pub compliance_percentage: f64,
}

impl From<ResidencyReportV4> for ResidencyReportResponseV4 {
    fn from(r: ResidencyReportV4) -> Self {
        Self {
            id: r.id,
            report_type: r.report_type,
            findings: r.findings,
            score: r.score,
            generated_at: r.generated_at.to_rfc3339(),
        }
    }
}

impl From<ResidencyComplianceV4> for ResidencyComplianceResponseV4 {
    fn from(c: ResidencyComplianceV4) -> Self {
        Self {
            id: c.id,
            rule_id: c.rule_id,
            compliance_status: c.compliance_status,
            last_checked_at: c.last_checked_at.to_rfc3339(),
            created_at: c.created_at.to_rfc3339(),
        }
    }
}

async fn generate_report(
    State(state): State<AppState>,
    Json(input): Json<GenerateReportRequest>,
) -> Result<(StatusCode, Json<ResidencyReportResponseV4>), Response> {
    let report = sqlx::query_as::<_, ResidencyReportV4>(
        r#"INSERT INTO data_residency_reports_v6 (report_type)
         VALUES ($1)
         RETURNING id, report_type, findings, score, generated_at"#,
    )
    .bind(&input.report_type)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(report.into())))
}

async fn get_reports(
    State(state): State<AppState>,
) -> Result<Json<Vec<ResidencyReportResponseV4>>, Response> {
    let reports = sqlx::query_as::<_, ResidencyReportV4>(
        r#"SELECT id, report_type, findings, score, generated_at
         FROM data_residency_reports_v6
         ORDER BY generated_at DESC"#,
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(reports.into_iter().map(|r| r.into()).collect()))
}

async fn track_compliance(
    State(state): State<AppState>,
    Json(input): Json<TrackComplianceRequest>,
) -> Result<(StatusCode, Json<ResidencyComplianceResponseV4>), Response> {
    let compliance = sqlx::query_as::<_, ResidencyComplianceV4>(
        r#"INSERT INTO data_residency_compliance_v6 (rule_id)
         VALUES ($1)
         RETURNING id, rule_id, compliance_status, last_checked_at, created_at"#,
    )
    .bind(input.rule_id)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(compliance.into())))
}

async fn update_compliance_status(
    State(state): State<AppState>,
    Path(compliance_id): Path<Uuid>,
    Json(input): Json<UpdateComplianceStatusRequest>,
) -> Result<Json<ResidencyComplianceResponseV4>, Response> {
    let compliance = sqlx::query_as::<_, ResidencyComplianceV4>(
        r#"UPDATE data_residency_compliance_v6
         SET compliance_status = $2, last_checked_at = NOW()
         WHERE id = $1
         RETURNING id, rule_id, compliance_status, last_checked_at, created_at"#,
    )
    .bind(compliance_id)
    .bind(&input.status)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?
    .ok_or_else(|| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "compliance entry not found"}))).into_response())?;

    Ok(Json(compliance.into()))
}

async fn get_compliance_by_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<Uuid>,
) -> Result<Json<Vec<ResidencyComplianceResponseV4>>, Response> {
    let entries = sqlx::query_as::<_, ResidencyComplianceV4>(
        r#"SELECT id, rule_id, compliance_status, last_checked_at, created_at
         FROM data_residency_compliance_v6
         WHERE rule_id = $1
         ORDER BY last_checked_at DESC"#,
    )
    .bind(rule_id)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(entries.into_iter().map(|c| c.into()).collect()))
}

async fn resolve_violation(
    State(state): State<AppState>,
    Path(violation_id): Path<Uuid>,
    Json(input): Json<ResolveViolationRequest>,
) -> Result<Json<ViolationResolutionResponse>, Response> {
    let _ = sqlx::query(
        r#"UPDATE data_residency_violations
         SET status = 'resolved'
         WHERE id = $1"#,
    )
    .bind(violation_id)
    .execute(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(ViolationResolutionResponse {
        violation_id,
        resolution_type: input.resolution_type,
        details: input.details.unwrap_or(serde_json::json!({})),
        resolved_at: Utc::now().to_rfc3339(),
    }))
}

async fn get_residency_compliance_summary(
    State(state): State<AppState>,
) -> Result<Json<ResidencyComplianceSummaryResponse>, Response> {
    let total_rules = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM data_residency_rules"#,
    )
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let enabled_rules = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM data_residency_rules WHERE enabled = true"#,
    )
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let total_violations = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM data_residency_violations"#,
    )
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let resolved_violations = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM data_residency_violations WHERE status = 'resolved'"#,
    )
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let average_score = sqlx::query_scalar::<_, f64>(
        r#"SELECT COALESCE(AVG(score), 0.0) FROM data_residency_reports_v6"#,
    )
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0.0);

    let compliance_percentage = if total_rules > 0 {
        ((enabled_rules as f64 / total_rules as f64) * 100.0).min(100.0)
    } else {
        100.0
    };

    Ok(Json(ResidencyComplianceSummaryResponse {
        total_rules,
        enabled_rules,
        total_violations,
        resolved_violations,
        average_score,
        compliance_percentage,
    }))
}

pub fn data_residency_v8_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/data-residency-v8/reports",
            post(generate_report).get(get_reports),
        )
        .route(
            "/api/v1/data-residency-v8/compliance",
            post(track_compliance).get(get_residency_compliance_summary),
        )
        .route(
            "/api/v1/data-residency-v8/compliance/{compliance_id}",
            patch(update_compliance_status),
        )
        .route(
            "/api/v1/data-residency-v8/compliance/rule/{rule_id}",
            get(get_compliance_by_rule),
        )
        .route(
            "/api/v1/data-residency-v8/violations/{violation_id}/resolve",
            post(resolve_violation),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_response_conversion() {
        let report = ResidencyReportV4 {
            id: Uuid::nil(),
            report_type: "compliance".to_string(),
            findings: serde_json::json!([{"rule": "gdpr"}]),
            score: 90,
            generated_at: Utc::now(),
        };
        let response: ResidencyReportResponseV4 = report.into();
        assert_eq!(response.score, 90);
        assert_eq!(response.report_type, "compliance");
    }

    #[test]
    fn test_compliance_response_conversion() {
        let compliance = ResidencyComplianceV4 {
            id: Uuid::nil(),
            rule_id: Uuid::nil(),
            compliance_status: "compliant".to_string(),
            last_checked_at: Utc::now(),
            created_at: Utc::now(),
        };
        let response: ResidencyComplianceResponseV4 = compliance.into();
        assert_eq!(response.compliance_status, "compliant");
    }
}
