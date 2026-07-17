#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::compliance_reporting::ComplianceReportFrameworkType;
use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ReportQuery {
    pub framework: Option<String>,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EvidenceQuery {
    pub framework: Option<String>,
    pub control_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FrameworksQuery {
    pub enabled_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ScheduleRequest {
    pub framework: String,
    pub frequency: String,
    pub day_of_week: Option<u8>,
    pub day_of_month: Option<u8>,
    pub recipients: Vec<String>,
    pub distribution_method: String,
}

#[derive(Debug, Serialize)]
pub struct ComplianceReportResponse {
    pub id: Uuid,
    pub framework: String,
    pub report_name: String,
    pub period_start: String,
    pub period_end: String,
    pub status: String,
    pub controls_summary: ControlsSummaryDto,
    pub findings_count: u32,
    pub evidence_count: u32,
    pub overall_score: f64,
    pub generated_at: String,
    pub distributed: bool,
}

#[derive(Debug, Serialize)]
pub struct ControlsSummaryDto {
    pub total: u32,
    pub passing: u32,
    pub failing: u32,
    pub partially_passing: u32,
    pub not_assessed: u32,
    pub compliance_score: f64,
}

#[derive(Debug, Serialize)]
pub struct EvidenceResponse {
    pub evidence_items: Vec<EvidenceItemDto>,
    pub total: u32,
    pub generated_at: String,
}

#[derive(Debug, Serialize)]
pub struct EvidenceItemDto {
    pub id: Uuid,
    pub control_id: String,
    pub evidence_type: String,
    pub description: String,
    pub collected_at: String,
    pub expires_at: Option<String>,
    pub storage_path: Option<String>,
    pub integrity_hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FrameworkResponse {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub framework_type: String,
    pub description: String,
    pub controls_count: u32,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct FrameworksListResponse {
    pub frameworks: Vec<FrameworkResponse>,
    pub total: u32,
}

#[derive(Debug, Serialize)]
pub struct ScheduleResponse {
    pub id: Uuid,
    pub framework: String,
    pub frequency: String,
    pub recipients: Vec<String>,
    pub distribution_method: String,
    pub enabled: bool,
    pub next_run: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct SchedulesListResponse {
    pub schedules: Vec<ScheduleResponse>,
    pub total: u32,
}

async fn generate_compliance_report(
    State(_state): State<AppState>,
    Query(params): Query<ReportQuery>,
) -> Result<Json<ComplianceReportResponse>, (StatusCode, Json<serde_json::Value>)> {
    let framework_str = params.framework.unwrap_or_else(|| "SOC2".into());
    let framework_type = match framework_str.to_uppercase().as_str() {
        "SOC2" => ComplianceReportFrameworkType::Soc2TypeII,
        "GDPR" => ComplianceReportFrameworkType::Gdpr,
        "ISO27001" => ComplianceReportFrameworkType::Iso27001,
        "HIPAA" => ComplianceReportFrameworkType::Hipaa,
        "PCI_DSS" => ComplianceReportFrameworkType::PciDss,
        _ => ComplianceReportFrameworkType::Custom,
    };

    let controls = framework_type.default_controls();
    let controls_summary = crate::compliance_reporting::compute_controls_summary(&controls);

    let now = Utc::now();
    let start = params
        .period_start
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| now - chrono::Duration::days(90));
    let end = params
        .period_end
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or(now);

    Ok(Json(ComplianceReportResponse {
        id: Uuid::new_v4(),
        framework: framework_type.display_name().into(),
        report_name: format!("{} Report", framework_type.display_name()),
        period_start: start.to_rfc3339(),
        period_end: end.to_rfc3339(),
        status: "completed".into(),
        controls_summary: ControlsSummaryDto {
            total: controls_summary.total,
            passing: controls_summary.passing,
            failing: controls_summary.failing,
            partially_passing: controls_summary.partially_passing,
            not_assessed: controls_summary.not_assessed,
            compliance_score: controls_summary.compliance_score,
        },
        findings_count: 0,
        evidence_count: 0,
        overall_score: controls_summary.compliance_score,
        generated_at: now.to_rfc3339(),
        distributed: false,
    }))
}

async fn collect_evidence(
    State(_state): State<AppState>,
    Query(params): Query<EvidenceQuery>,
) -> Result<Json<EvidenceResponse>, (StatusCode, Json<serde_json::Value>)> {
    let framework_str = params.framework.unwrap_or_else(|| "SOC2".into());
    let framework_type = match framework_str.to_uppercase().as_str() {
        "SOC2" => ComplianceReportFrameworkType::Soc2TypeII,
        "GDPR" => ComplianceReportFrameworkType::Gdpr,
        "ISO27001" => ComplianceReportFrameworkType::Iso27001,
        _ => ComplianceReportFrameworkType::Custom,
    };

    let mut evidence = crate::compliance_reporting::collect_evidence_for_framework(&framework_type);

    if let Some(ref control_id) = params.control_id {
        evidence.retain(|e| &e.control_id == control_id);
    }

    let total = evidence.len() as u32;

    let evidence_dtos: Vec<EvidenceItemDto> = evidence
        .into_iter()
        .map(|e| EvidenceItemDto {
            id: e.id,
            control_id: e.control_id,
            evidence_type: e.evidence_type.display_name().into(),
            description: e.description,
            collected_at: e.collected_at.to_rfc3339(),
            expires_at: e.expires_at.map(|dt| dt.to_rfc3339()),
            storage_path: e.storage_path,
            integrity_hash: e.integrity_hash,
        })
        .collect();

    Ok(Json(EvidenceResponse {
        evidence_items: evidence_dtos,
        total,
        generated_at: Utc::now().to_rfc3339(),
    }))
}

async fn list_frameworks(
    State(_state): State<AppState>,
    Query(params): Query<FrameworksQuery>,
) -> Result<Json<FrameworksListResponse>, (StatusCode, Json<serde_json::Value>)> {
    let enabled_only = params.enabled_only.unwrap_or(false);
    let frameworks = crate::compliance_reporting::supported_frameworks();

    let frameworks: Vec<FrameworkResponse> = frameworks
        .into_iter()
        .filter(|f| !enabled_only || f.enabled)
        .map(|f| FrameworkResponse {
            id: f.id,
            name: f.name,
            version: f.version,
            framework_type: f.framework_type.display_name().into(),
            description: f.description,
            controls_count: f.controls.len() as u32,
            enabled: f.enabled,
        })
        .collect();

    let total = frameworks.len() as u32;

    Ok(Json(FrameworksListResponse {
        frameworks,
        total,
    }))
}

async fn schedule_report(
    State(_state): State<AppState>,
    Json(req): Json<ScheduleRequest>,
) -> Result<Json<ScheduleResponse>, (StatusCode, Json<serde_json::Value>)> {
    let valid_frameworks = ["SOC2", "GDPR", "ISO27001", "HIPAA", "PCI_DSS", "CUSTOM"];
    if !valid_frameworks.contains(&req.framework.to_uppercase().as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                serde_json::json!({
                    "error": format!(
                        "invalid framework: {}, must be one of {}",
                        req.framework,
                        valid_frameworks.join(", ")
                    )
                }),
            ),
        ));
    }

    let valid_frequencies = ["weekly", "monthly", "quarterly", "annually"];
    if !valid_frequencies.contains(&req.frequency.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                serde_json::json!({
                    "error": format!(
                        "invalid frequency: {}, must be one of {}",
                        req.frequency,
                        valid_frequencies.join(", ")
                    )
                }),
            ),
        ));
    }

    let valid_distributions = ["email", "download", "both"];
    if !valid_distributions.contains(&req.distribution_method.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                serde_json::json!({
                    "error": format!(
                        "invalid distribution_method: {}, must be one of {}",
                        req.distribution_method,
                        valid_distributions.join(", ")
                    )
                }),
            ),
        ));
    }

    let now = Utc::now();
    let next_run = match req.frequency.as_str() {
        "weekly" => Some(now + chrono::Duration::days(7)),
        "monthly" => Some(now + chrono::Duration::days(30)),
        "quarterly" => Some(now + chrono::Duration::days(90)),
        "annually" => Some(now + chrono::Duration::days(365)),
        _ => None,
    };

    Ok(Json(ScheduleResponse {
        id: Uuid::new_v4(),
        framework: req.framework,
        frequency: req.frequency,
        recipients: req.recipients,
        distribution_method: req.distribution_method,
        enabled: true,
        next_run: next_run.map(|dt| dt.to_rfc3339()),
        created_at: now.to_rfc3339(),
    }))
}

async fn list_schedules(
    State(_state): State<AppState>,
) -> Result<Json<SchedulesListResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SchedulesListResponse {
        schedules: vec![],
        total: 0,
    }))
}

pub fn compliance_report_api_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/compliance/report", get(generate_compliance_report))
        .route(
            "/api/v1/compliance/evidence",
            get(collect_evidence),
        )
        .route(
            "/api/v1/compliance/schedule",
            post(schedule_report).get(list_schedules),
        )
        .route(
            "/api/v1/compliance/frameworks",
            get(list_frameworks),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_query_deserialization() {
        let json = r#"{"framework": "SOC2", "period_start": "2024-01-01T00:00:00Z"}"#;
        let query: ReportQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.framework.as_deref(), Some("SOC2"));
        assert!(query.period_start.is_some());
    }

    #[test]
    fn test_evidence_query_deserialization() {
        let json = r#"{"framework": "GDPR", "control_id": "Art.5"}"#;
        let query: EvidenceQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.framework.as_deref(), Some("GDPR"));
        assert_eq!(query.control_id.as_deref(), Some("Art.5"));
    }

    #[test]
    fn test_schedule_request_deserialization() {
        let json = r#"{
            "framework": "SOC2",
            "frequency": "quarterly",
            "recipients": ["admin@example.com"],
            "distribution_method": "email"
        }"#;
        let req: ScheduleRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.framework, "SOC2");
        assert_eq!(req.frequency, "quarterly");
        assert_eq!(req.recipients.len(), 1);
        assert_eq!(req.distribution_method, "email");
    }

    #[test]
    fn test_compliance_report_response_serialization() {
        let resp = ComplianceReportResponse {
            id: Uuid::new_v4(),
            framework: "SOC 2 Type II".into(),
            report_name: "SOC 2 Type II Report".into(),
            period_start: "2024-01-01T00:00:00Z".into(),
            period_end: "2024-04-01T00:00:00Z".into(),
            status: "completed".into(),
            controls_summary: ControlsSummaryDto {
                total: 3,
                passing: 2,
                failing: 0,
                partially_passing: 1,
                not_assessed: 0,
                compliance_score: 83.3,
            },
            findings_count: 0,
            evidence_count: 3,
            overall_score: 83.3,
            generated_at: "2024-04-01T00:00:00Z".into(),
            distributed: false,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"framework\":\"SOC 2 Type II\""));
        assert!(json.contains("\"compliance_score\":83.3"));
    }

    #[test]
    fn test_framework_response_serialization() {
        let resp = FrameworkResponse {
            id: Uuid::new_v4(),
            name: "SOC 2".into(),
            version: "2017".into(),
            framework_type: "SOC 2 Type II".into(),
            description: "Service Organization Control".into(),
            controls_count: 3,
            enabled: true,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"controls_count\":3"));
        assert!(json.contains("\"enabled\":true"));
    }

    #[test]
    fn test_evidence_item_dto_serialization() {
        let dto = EvidenceItemDto {
            id: Uuid::new_v4(),
            control_id: "CC6.1".into(),
            evidence_type: "Audit Log".into(),
            description: "Test evidence".into(),
            collected_at: "2024-01-01T00:00:00Z".into(),
            expires_at: Some("2031-01-01T00:00:00Z".into()),
            storage_path: Some("/evidence/".into()),
            integrity_hash: None,
        };
        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("\"evidence_type\":\"Audit Log\""));
        assert!(json.contains("\"storage_path\""));
    }

    #[test]
    fn test_schedule_response_serialization() {
        let resp = ScheduleResponse {
            id: Uuid::new_v4(),
            framework: "SOC2".into(),
            frequency: "quarterly".into(),
            recipients: vec!["admin@example.com".into()],
            distribution_method: "email".into(),
            enabled: true,
            next_run: Some("2024-07-01T00:00:00Z".into()),
            created_at: "2024-04-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"frequency\":\"quarterly\""));
        assert!(json.contains("\"distribution_method\":\"email\""));
    }

    #[test]
    fn test_compliance_report_api_routes_compile() {
        let _router = compliance_report_api_routes();
    }
}
