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

// --- v2: Audits, migrations, compliance ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ResidencyAudit {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub audit_type: String,
    pub findings: serde_json::Value,
    pub score: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ResidencyMigration {
    pub id: Uuid,
    pub violation_id: Uuid,
    pub target_region: String,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAuditRequest {
    pub rule_id: Uuid,
    pub audit_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateMigrationRequest {
    pub violation_id: Uuid,
    pub target_region: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResidencyAuditResponse {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub audit_type: String,
    pub findings: serde_json::Value,
    pub score: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResidencyMigrationResponse {
    pub id: Uuid,
    pub violation_id: Uuid,
    pub target_region: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResidencyComplianceResponse {
    pub total_rules: i64,
    pub enabled_rules: i64,
    pub total_violations: i64,
    pub resolved_violations: i64,
    pub average_score: f64,
    pub compliance_percentage: f64,
}

impl From<ResidencyAudit> for ResidencyAuditResponse {
    fn from(audit: ResidencyAudit) -> Self {
        Self {
            id: audit.id,
            rule_id: audit.rule_id,
            audit_type: audit.audit_type,
            findings: audit.findings,
            score: audit.score,
            created_at: audit.created_at.to_rfc3339(),
        }
    }
}

impl From<ResidencyMigration> for ResidencyMigrationResponse {
    fn from(migration: ResidencyMigration) -> Self {
        Self {
            id: migration.id,
            violation_id: migration.violation_id,
            target_region: migration.target_region,
            status: migration.status,
            started_at: migration.started_at.to_rfc3339(),
            completed_at: migration.completed_at.map(|c| c.to_rfc3339()),
        }
    }
}

async fn create_residency_audit(
    State(state): State<AppState>,
    Json(input): Json<CreateAuditRequest>,
) -> Result<(StatusCode, Json<ResidencyAuditResponse>), Response> {
    let audit = sqlx::query_as::<_, ResidencyAudit>(
        r#"INSERT INTO data_residency_audits (rule_id, audit_type)
         VALUES ($1, $2)
         RETURNING id, rule_id, audit_type, findings, score, created_at"#,
    )
    .bind(input.rule_id)
    .bind(&input.audit_type)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(audit.into())))
}

async fn list_residency_audits(
    State(state): State<AppState>,
) -> Result<Json<Vec<ResidencyAuditResponse>>, Response> {
    let audits = sqlx::query_as::<_, ResidencyAudit>(
        r#"SELECT id, rule_id, audit_type, findings, score, created_at
         FROM data_residency_audits ORDER BY created_at DESC LIMIT 100"#,
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(audits.into_iter().map(|a| a.into()).collect()))
}

async fn update_residency_audit_findings(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<ResidencyAuditResponse>, Response> {
    let findings = input.get("findings").cloned().unwrap_or(serde_json::json!([]));
    let score = input.get("score").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    let audit = sqlx::query_as::<_, ResidencyAudit>(
        r#"UPDATE data_residency_audits SET findings = $2, score = $3 WHERE id = $1
         RETURNING id, rule_id, audit_type, findings, score, created_at"#,
    )
    .bind(id)
    .bind(findings)
    .bind(score)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?
    .ok_or_else(|| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "audit not found"}))).into_response())?;

    Ok(Json(audit.into()))
}

async fn create_residency_migration(
    State(state): State<AppState>,
    Json(input): Json<CreateMigrationRequest>,
) -> Result<(StatusCode, Json<ResidencyMigrationResponse>), Response> {
    let migration = sqlx::query_as::<_, ResidencyMigration>(
        r#"INSERT INTO data_residency_migrations (violation_id, target_region)
         VALUES ($1, $2)
         RETURNING id, violation_id, target_region, status, started_at, completed_at"#,
    )
    .bind(input.violation_id)
    .bind(&input.target_region)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(migration.into())))
}

async fn list_residency_migrations(
    State(state): State<AppState>,
) -> Result<Json<Vec<ResidencyMigrationResponse>>, Response> {
    let migrations = sqlx::query_as::<_, ResidencyMigration>(
        r#"SELECT id, violation_id, target_region, status, started_at, completed_at
         FROM data_residency_migrations ORDER BY started_at DESC LIMIT 100"#,
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(migrations.into_iter().map(|m| m.into()).collect()))
}

async fn complete_residency_migration(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ResidencyMigrationResponse>, Response> {
    let migration = sqlx::query_as::<_, ResidencyMigration>(
        r#"UPDATE data_residency_migrations SET status = 'completed', completed_at = NOW()
         WHERE id = $1
         RETURNING id, violation_id, target_region, status, started_at, completed_at"#,
    )
    .bind(id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?
    .ok_or_else(|| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "migration not found"}))).into_response())?;

    Ok(Json(migration.into()))
}

async fn get_residency_compliance(
    State(state): State<AppState>,
) -> Result<Json<ResidencyComplianceResponse>, Response> {
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
        r#"SELECT COUNT(*) FROM data_residency_migrations WHERE status = 'completed'"#,
    )
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0);

    let average_score = sqlx::query_scalar::<_, f64>(
        r#"SELECT COALESCE(AVG(score), 0.0) FROM data_residency_audits"#,
    )
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0.0);

    let compliance_percentage = if total_rules > 0 {
        ((enabled_rules as f64 / total_rules as f64) * 100.0).min(100.0)
    } else {
        100.0
    };

    Ok(Json(ResidencyComplianceResponse {
        total_rules,
        enabled_rules,
        total_violations,
        resolved_violations,
        average_score,
        compliance_percentage,
    }))
}

// --- v5/v8/v9/v11: Reports, compliance, violations ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ResidencyReportV5 {
    pub id: Uuid,
    pub report_type: String,
    pub findings: serde_json::Value,
    pub score: i32,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ResidencyComplianceV5 {
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
pub struct ResidencyReportResponseV5 {
    pub id: Uuid,
    pub report_type: String,
    pub findings: serde_json::Value,
    pub score: i32,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResidencyComplianceResponseV5 {
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

impl From<ResidencyReportV5> for ResidencyReportResponseV5 {
    fn from(r: ResidencyReportV5) -> Self {
        Self {
            id: r.id,
            report_type: r.report_type,
            findings: r.findings,
            score: r.score,
            generated_at: r.generated_at.to_rfc3339(),
        }
    }
}

impl From<ResidencyComplianceV5> for ResidencyComplianceResponseV5 {
    fn from(c: ResidencyComplianceV5) -> Self {
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
) -> Result<(StatusCode, Json<ResidencyReportResponseV5>), Response> {
    let report = sqlx::query_as::<_, ResidencyReportV5>(
        r#"INSERT INTO data_residency_reports_v13 (report_type)
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
) -> Result<Json<Vec<ResidencyReportResponseV5>>, Response> {
    let reports = sqlx::query_as::<_, ResidencyReportV5>(
        r#"SELECT id, report_type, findings, score, generated_at
         FROM data_residency_reports_v13
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
) -> Result<(StatusCode, Json<ResidencyComplianceResponseV5>), Response> {
    let compliance = sqlx::query_as::<_, ResidencyComplianceV5>(
        r#"INSERT INTO data_residency_compliance_v13 (rule_id)
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
) -> Result<Json<ResidencyComplianceResponseV5>, Response> {
    let compliance = sqlx::query_as::<_, ResidencyComplianceV5>(
        r#"UPDATE data_residency_compliance_v13
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
) -> Result<Json<Vec<ResidencyComplianceResponseV5>>, Response> {
    let entries = sqlx::query_as::<_, ResidencyComplianceV5>(
        r#"SELECT id, rule_id, compliance_status, last_checked_at, created_at
         FROM data_residency_compliance_v13
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
        r#"SELECT COALESCE(AVG(score), 0.0) FROM data_residency_reports_v13"#,
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

// --- v12: Transfer requests, compliance checks, region analytics, compliance report ---

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
    Path((request_id, approved_by)): Path<(Uuid, Uuid)>,
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

pub fn data_residency_routes() -> Router<AppState> {
    Router::new()
        // v2 routes
        .route("/api/v1/data-residency/audits", post(create_residency_audit).get(list_residency_audits))
        .route("/api/v1/data-residency/audits/{id}", patch(update_residency_audit_findings))
        .route("/api/v1/data-residency/migrations", post(create_residency_migration).get(list_residency_migrations))
        .route("/api/v1/data-residency/migrations/{id}/complete", post(complete_residency_migration))
        .route("/api/v1/data-residency/compliance", get(get_residency_compliance))
        // v5 routes
        .route("/api/v1/data-residency-v5/reports", post(generate_report).get(get_reports))
        .route("/api/v1/data-residency-v5/compliance", post(track_compliance).get(get_residency_compliance_summary))
        .route("/api/v1/data-residency-v5/compliance/{compliance_id}", patch(update_compliance_status))
        .route("/api/v1/data-residency-v5/compliance/rule/{rule_id}", get(get_compliance_by_rule))
        .route("/api/v1/data-residency-v5/violations/{violation_id}/resolve", post(resolve_violation))
        // v8 routes
        .route("/api/v1/data-residency-v8/reports", post(generate_report).get(get_reports))
        .route("/api/v1/data-residency-v8/compliance", post(track_compliance).get(get_residency_compliance_summary))
        .route("/api/v1/data-residency-v8/compliance/{compliance_id}", patch(update_compliance_status))
        .route("/api/v1/data-residency-v8/compliance/rule/{rule_id}", get(get_compliance_by_rule))
        .route("/api/v1/data-residency-v8/violations/{violation_id}/resolve", post(resolve_violation))
        // v9 routes
        .route("/api/v1/data-residency-v9/reports", post(generate_report).get(get_reports))
        .route("/api/v1/data-residency-v9/compliance", post(track_compliance).get(get_residency_compliance_summary))
        .route("/api/v1/data-residency-v9/compliance/{compliance_id}", patch(update_compliance_status))
        .route("/api/v1/data-residency-v9/compliance/rule/{rule_id}", get(get_compliance_by_rule))
        .route("/api/v1/data-residency-v9/violations/{violation_id}/resolve", post(resolve_violation))
        // v11 routes
        .route("/api/v1/data-residency-v11/reports", post(generate_report).get(get_reports))
        .route("/api/v1/data-residency-v11/compliance", post(track_compliance).get(get_residency_compliance_summary))
        .route("/api/v1/data-residency-v11/compliance/{compliance_id}", patch(update_compliance_status))
        .route("/api/v1/data-residency-v11/compliance/rule/{rule_id}", get(get_compliance_by_rule))
        .route("/api/v1/data-residency-v11/violations/{violation_id}/resolve", post(resolve_violation))
        // v12 routes
        .route("/api/v1/data-residency-v12/transfer-requests", post(create_transfer_request).get(get_transfer_requests))
        .route("/api/v1/data-residency-v12/transfer-requests/{request_id}/approve/{approved_by}", post(approve_transfer_request))
        .route("/api/v1/data-residency-v12/compliance-checks", post(run_compliance_check))
        .route("/api/v1/data-residency-v12/compliance-checks/{region}", get(get_compliance_checks))
        .route("/api/v1/data-residency-v12/region-analytics", get(get_region_analytics))
        .route("/api/v1/data-residency-v12/compliance-report", get(generate_compliance_report))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_residency_audit_response_conversion() {
        let audit = ResidencyAudit {
            id: Uuid::nil(),
            rule_id: Uuid::nil(),
            audit_type: "compliance_check".to_string(),
            findings: serde_json::json!([]),
            score: 85,
            created_at: Utc::now(),
        };
        let response: ResidencyAuditResponse = audit.into();
        assert_eq!(response.score, 85);
        assert_eq!(response.audit_type, "compliance_check");
    }

    #[test]
    fn test_residency_migration_response_conversion() {
        let migration = ResidencyMigration {
            id: Uuid::nil(),
            violation_id: Uuid::nil(),
            target_region: "eu-west-1".to_string(),
            status: "pending".to_string(),
            started_at: Utc::now(),
            completed_at: None,
        };
        let response: ResidencyMigrationResponse = migration.into();
        assert_eq!(response.target_region, "eu-west-1");
        assert!(response.completed_at.is_none());
    }

    #[test]
    fn test_report_response_conversion() {
        let report = ResidencyReportV5 {
            id: Uuid::nil(),
            report_type: "compliance".to_string(),
            findings: serde_json::json!([{"rule": "gdpr"}]),
            score: 90,
            generated_at: Utc::now(),
        };
        let response: ResidencyReportResponseV5 = report.into();
        assert_eq!(response.score, 90);
        assert_eq!(response.report_type, "compliance");
    }

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
