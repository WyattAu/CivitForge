#![forbid(unsafe_code)]

use crate::api::AppState;
use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct SlaStatusQuery {
    pub sla_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct SlaHistoryQuery {
    pub sla_id: Option<Uuid>,
    pub since: Option<String>,
    pub until: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_limit() -> u32 {
    100
}

#[derive(Debug, Deserialize)]
pub struct SlaReportQuery {
    pub period: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAlertRequest {
    pub sla_id: Uuid,
    pub alert_type: String,
    pub threshold_percentage: f64,
    pub notify_emails: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SlaStatusResponse {
    pub overall_status: String,
    pub overall_compliance: f64,
    pub statuses: Vec<SlaCurrentStatusDto>,
    pub generated_at: String,
}

#[derive(Debug, Serialize)]
pub struct SlaCurrentStatusDto {
    pub sla_id: Uuid,
    pub sla_name: String,
    pub metric_type: String,
    pub target_value: f64,
    pub current_value: f64,
    pub status: String,
    pub compliance_percentage: f64,
    pub last_checked_at: String,
}

#[derive(Debug, Serialize)]
pub struct SlaHistoryResponse {
    pub entries: Vec<SlaHistoricalEntryDto>,
    pub total: u32,
    pub generated_at: String,
}

#[derive(Debug, Serialize)]
pub struct SlaHistoricalEntryDto {
    pub timestamp: String,
    pub actual_value: f64,
    pub target_value: f64,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct SlaReportResponse {
    pub report_id: Uuid,
    pub period: String,
    pub period_start: String,
    pub period_end: String,
    pub overall_compliance: f64,
    pub total_breaches: u32,
    pub sla_results: Vec<SlaResultDto>,
    pub generated_at: String,
}

#[derive(Debug, Serialize)]
pub struct SlaResultDto {
    pub sla_id: Uuid,
    pub sla_name: String,
    pub metric_type: String,
    pub target_value: f64,
    pub actual_value: f64,
    pub uptime_percentage: f64,
    pub breach_count: u32,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct SlaDashboardResponse {
    pub overall_status: String,
    pub overall_compliance: f64,
    pub active_sla_count: u32,
    pub breached_sla_count: u32,
    pub at_risk_sla_count: u32,
    pub total_breaches_this_month: u32,
    pub current_incidents: u32,
    pub sla_statuses: Vec<SlaCurrentStatusDto>,
    pub recent_breaches: Vec<SlaBreachDto>,
    pub compliance_trend: Vec<ComplianceTrendPointDto>,
}

#[derive(Debug, Serialize)]
pub struct SlaBreachDto {
    pub id: Uuid,
    pub sla_name: String,
    pub metric_type: String,
    pub target_value: f64,
    pub actual_value: f64,
    pub detected_at: String,
    pub resolved_at: Option<String>,
    pub severity: String,
}

#[derive(Debug, Serialize)]
pub struct ComplianceTrendPointDto {
    pub date: String,
    pub compliance_percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct CreateAlertResponse {
    pub id: Uuid,
    pub sla_id: Uuid,
    pub alert_type: String,
    pub threshold_percentage: f64,
    pub notify_emails: Vec<String>,
    pub enabled: bool,
    pub created_at: String,
}

async fn get_sla_status(
    State(_state): State<AppState>,
    Query(params): Query<SlaStatusQuery>,
) -> Result<Json<SlaStatusResponse>, (StatusCode, Json<serde_json::Value>)> {
    let _ = params.sla_id;

    let statuses: Vec<SlaCurrentStatusDto> = vec![
        SlaCurrentStatusDto {
            sla_id: Uuid::new_v4(),
            sla_name: "Platform Uptime".into(),
            metric_type: "uptime".into(),
            target_value: 99.9,
            current_value: 99.95,
            status: "met".into(),
            compliance_percentage: 100.0,
            last_checked_at: Utc::now().to_rfc3339(),
        },
        SlaCurrentStatusDto {
            sla_id: Uuid::new_v4(),
            sla_name: "API Response Time".into(),
            metric_type: "response_time".into(),
            target_value: 200.0,
            current_value: 145.0,
            status: "met".into(),
            compliance_percentage: 98.5,
            last_checked_at: Utc::now().to_rfc3339(),
        },
        SlaCurrentStatusDto {
            sla_id: Uuid::new_v4(),
            sla_name: "Error Rate".into(),
            metric_type: "error_rate".into(),
            target_value: 1.0,
            current_value: 0.3,
            status: "met".into(),
            compliance_percentage: 99.8,
            last_checked_at: Utc::now().to_rfc3339(),
        },
    ];

    Ok(Json(SlaStatusResponse {
        overall_status: "met".into(),
        overall_compliance: 99.4,
        statuses,
        generated_at: Utc::now().to_rfc3339(),
    }))
}

async fn get_sla_history(
    State(_state): State<AppState>,
    Query(params): Query<SlaHistoryQuery>,
) -> Result<Json<SlaHistoryResponse>, (StatusCode, Json<serde_json::Value>)> {
    let _ = params;

    let entries: Vec<SlaHistoricalEntryDto> = (0..24)
        .map(|i| SlaHistoricalEntryDto {
            timestamp: (Utc::now() - Duration::hours(i)).to_rfc3339(),
            actual_value: 99.9 + (i as f64 * 0.01),
            target_value: 99.9,
            status: "met".into(),
        })
        .collect();
    let total = entries.len() as u32;

    Ok(Json(SlaHistoryResponse {
        entries,
        total,
        generated_at: Utc::now().to_rfc3339(),
    }))
}

async fn get_sla_report(
    State(_state): State<AppState>,
    Query(params): Query<SlaReportQuery>,
) -> Result<Json<SlaReportResponse>, (StatusCode, Json<serde_json::Value>)> {
    let period = params.period.unwrap_or_else(|| "daily".into());
    let start = params
        .since
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| Utc::now() - Duration::days(1));
    let end = params
        .until
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    let sla_results = vec![
        SlaResultDto {
            sla_id: Uuid::new_v4(),
            sla_name: "Platform Uptime".into(),
            metric_type: "uptime".into(),
            target_value: 99.9,
            actual_value: 99.95,
            uptime_percentage: 99.95,
            breach_count: 0,
            status: "met".into(),
        },
        SlaResultDto {
            sla_id: Uuid::new_v4(),
            sla_name: "API Response Time".into(),
            metric_type: "response_time".into(),
            target_value: 200.0,
            actual_value: 145.0,
            uptime_percentage: 98.5,
            breach_count: 2,
            status: "met".into(),
        },
    ];

    Ok(Json(SlaReportResponse {
        report_id: Uuid::new_v4(),
        period,
        period_start: start.to_rfc3339(),
        period_end: end.to_rfc3339(),
        overall_compliance: 99.2,
        total_breaches: 2,
        sla_results,
        generated_at: Utc::now().to_rfc3339(),
    }))
}

async fn get_sla_dashboard(
    State(_state): State<AppState>,
) -> Result<Json<SlaDashboardResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SlaDashboardResponse {
        overall_status: "met".into(),
        overall_compliance: 99.4,
        active_sla_count: 5,
        breached_sla_count: 0,
        at_risk_sla_count: 1,
        total_breaches_this_month: 2,
        current_incidents: 0,
        sla_statuses: vec![
            SlaCurrentStatusDto {
                sla_id: Uuid::new_v4(),
                sla_name: "Platform Uptime".into(),
                metric_type: "uptime".into(),
                target_value: 99.9,
                current_value: 99.95,
                status: "met".into(),
                compliance_percentage: 100.0,
                last_checked_at: Utc::now().to_rfc3339(),
            },
        ],
        recent_breaches: vec![],
        compliance_trend: (0..30)
            .map(|i| ComplianceTrendPointDto {
                date: (Utc::now() - Duration::days(i)).to_rfc3339(),
                compliance_percentage: 99.0 + (i as f64 * 0.03),
            })
            .collect(),
    }))
}

async fn create_sla_alert(
    State(_state): State<AppState>,
    Json(req): Json<CreateAlertRequest>,
) -> Result<Json<CreateAlertResponse>, (StatusCode, Json<serde_json::Value>)> {
    let valid_types = ["breach", "at_risk", "recovery", "degraded"];
    if !valid_types.contains(&req.alert_type.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                serde_json::json!({
                    "error": format!(
                        "invalid alert_type: {}, must be one of {}",
                        req.alert_type,
                        valid_types.join(", ")
                    )
                }),
            ),
        ));
    }

    Ok(Json(CreateAlertResponse {
        id: Uuid::new_v4(),
        sla_id: req.sla_id,
        alert_type: req.alert_type,
        threshold_percentage: req.threshold_percentage,
        notify_emails: req.notify_emails,
        enabled: true,
        created_at: Utc::now().to_rfc3339(),
    }))
}

pub fn sla_api_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/sla/status", get(get_sla_status))
        .route("/api/v1/sla/history", get(get_sla_history))
        .route("/api/v1/sla/report", get(get_sla_report))
        .route("/api/v1/sla/dashboard", get(get_sla_dashboard))
        .route("/api/v1/sla/alerts", post(create_sla_alert))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sla_status_response_serialization() {
        let resp = SlaStatusResponse {
            overall_status: "met".into(),
            overall_compliance: 99.5,
            statuses: vec![],
            generated_at: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"overall_status\":\"met\""));
        assert!(json.contains("\"overall_compliance\":99.5"));
    }

    #[test]
    fn test_create_alert_request_deserialization() {
        let json = r#"{
            "sla_id": "00000000-0000-0000-0000-000000000001",
            "alert_type": "breach",
            "threshold_percentage": 99.0,
            "notify_emails": ["admin@example.com"]
        }"#;
        let req: CreateAlertRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.alert_type, "breach");
        assert_eq!(req.threshold_percentage, 99.0);
        assert_eq!(req.notify_emails.len(), 1);
    }

    #[test]
    fn test_sla_report_response_serialization() {
        let resp = SlaReportResponse {
            report_id: Uuid::new_v4(),
            period: "daily".into(),
            period_start: "2025-01-01T00:00:00Z".into(),
            period_end: "2025-01-02T00:00:00Z".into(),
            overall_compliance: 99.0,
            total_breaches: 1,
            sla_results: vec![],
            generated_at: "2025-01-02T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"period\":\"daily\""));
        assert!(json.contains("\"total_breaches\":1"));
    }

    #[test]
    fn test_sla_dashboard_response_serialization() {
        let resp = SlaDashboardResponse {
            overall_status: "at_risk".into(),
            overall_compliance: 98.0,
            active_sla_count: 5,
            breached_sla_count: 1,
            at_risk_sla_count: 2,
            total_breaches_this_month: 3,
            current_incidents: 1,
            sla_statuses: vec![],
            recent_breaches: vec![],
            compliance_trend: vec![],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"breached_sla_count\":1"));
        assert!(json.contains("\"at_risk_sla_count\":2"));
    }

    #[test]
    fn test_sla_breach_dto_serialization() {
        let breach = SlaBreachDto {
            id: Uuid::new_v4(),
            sla_name: "Uptime".into(),
            metric_type: "uptime".into(),
            target_value: 99.9,
            actual_value: 99.5,
            detected_at: "2025-01-01T00:00:00Z".into(),
            resolved_at: None,
            severity: "high".into(),
        };
        let json = serde_json::to_string(&breach).unwrap();
        assert!(json.contains("\"severity\":\"high\""));
        assert!(json.contains("null"));
    }

    #[test]
    fn test_sla_api_routes_compile() {
        let _router = sla_api_routes();
    }
}
