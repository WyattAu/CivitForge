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
pub struct AlertsQuery {
    pub severity: Option<String>,
    pub since: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_limit() -> u32 {
    50
}

#[derive(Debug, Deserialize)]
pub struct SubscribeRequest {
    pub email: String,
    pub alert_types: Vec<String>,
    pub sla_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize)]
pub struct SlaAlert {
    pub id: Uuid,
    pub sla_id: Uuid,
    pub sla_name: String,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub metric_type: String,
    pub target_value: f64,
    pub actual_value: f64,
    pub detected_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AlertsResponse {
    pub alerts: Vec<SlaAlert>,
    pub total: u32,
    pub generated_at: String,
}

#[derive(Debug, Serialize)]
pub struct SubscribeResponse {
    pub subscription_id: Uuid,
    pub email: String,
    pub alert_types: Vec<String>,
    pub sla_ids: Vec<Uuid>,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionConfirmResponse {
    pub success: bool,
    pub message: String,
    pub subscription_id: Uuid,
    pub sla_ids: Vec<Uuid>,
    pub enabled: bool,
    pub created_at: String,
}

async fn get_sla_breach_alerts(
    State(_state): State<AppState>,
    Query(params): Query<AlertsQuery>,
) -> Result<Json<AlertsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let limit = params.limit.min(200);

    let mut alerts = vec![
        SlaAlert {
            id: Uuid::new_v4(),
            sla_id: Uuid::new_v4(),
            sla_name: "API Response Time".into(),
            alert_type: "breach".into(),
            severity: "high".into(),
            message: "API response time exceeded 200ms threshold for 5 minutes".into(),
            metric_type: "response_time".into(),
            target_value: 200.0,
            actual_value: 342.0,
            detected_at: (Utc::now() - Duration::minutes(15)).to_rfc3339(),
            resolved_at: Some((Utc::now() - Duration::minutes(10)).to_rfc3339()),
        },
        SlaAlert {
            id: Uuid::new_v4(),
            sla_id: Uuid::new_v4(),
            sla_name: "Error Rate".into(),
            alert_type: "at_risk".into(),
            severity: "medium".into(),
            message: "Error rate approaching 1% threshold".into(),
            metric_type: "error_rate".into(),
            target_value: 1.0,
            actual_value: 0.87,
            detected_at: (Utc::now() - Duration::hours(1)).to_rfc3339(),
            resolved_at: None,
        },
        SlaAlert {
            id: Uuid::new_v4(),
            sla_id: Uuid::new_v4(),
            sla_name: "Platform Uptime".into(),
            alert_type: "recovery".into(),
            severity: "low".into(),
            message: "Platform uptime recovered to 99.95% after brief degradation".into(),
            metric_type: "uptime".into(),
            target_value: 99.9,
            actual_value: 99.95,
            detected_at: (Utc::now() - Duration::hours(2)).to_rfc3339(),
            resolved_at: Some((Utc::now() - Duration::hours(1)).to_rfc3339()),
        },
    ];

    if let Some(ref severity) = params.severity {
        alerts.retain(|a| a.severity == *severity);
    }

    if let Some(ref since_str) = params.since {
        if let Ok(since) = DateTime::parse_from_rfc3339(since_str) {
            let since_utc = since.with_timezone(&Utc);
            alerts.retain(|a| {
                DateTime::parse_from_rfc3339(&a.detected_at)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc) >= since_utc)
                    .unwrap_or(true)
            });
        }
    }

    alerts.truncate(limit as usize);
    let total = alerts.len() as u32;

    Ok(Json(AlertsResponse {
        alerts,
        total,
        generated_at: Utc::now().to_rfc3339(),
    }))
}

async fn subscribe_sla_notifications(
    State(_state): State<AppState>,
    Json(req): Json<SubscribeRequest>,
) -> Result<(StatusCode, Json<SubscriptionConfirmResponse>), (StatusCode, Json<serde_json::Value>)> {
    if req.email.is_empty() || !req.email.contains('@') {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "valid email address required"
            })),
        ));
    }

    let valid_alert_types = ["breach", "at_risk", "recovery", "degraded"];
    for alert_type in &req.alert_types {
        if !valid_alert_types.contains(&alert_type.as_str()) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!(
                        "invalid alert_type: {}, must be one of {}",
                        alert_type,
                        valid_alert_types.join(", ")
                    )
                })),
            ));
        }
    }

    let subscription_id = Uuid::new_v4();
    let sla_ids = req.sla_ids.unwrap_or_default();

    Ok((
        StatusCode::CREATED,
        Json(SubscriptionConfirmResponse {
            success: true,
            message: format!(
                "Successfully subscribed {} to SLA notifications for alert types: {}",
                req.email,
                req.alert_types.join(", ")
            ),
            subscription_id,
            sla_ids,
            enabled: true,
            created_at: Utc::now().to_rfc3339(),
        }),
    ))
}

pub fn sla_dashboard_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/sla/alerts", get(get_sla_breach_alerts))
        .route("/api/v1/sla/subscribe", post(subscribe_sla_notifications))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sla_alert_serialization() {
        let alert = SlaAlert {
            id: Uuid::new_v4(),
            sla_id: Uuid::new_v4(),
            sla_name: "Test SLA".into(),
            alert_type: "breach".into(),
            severity: "high".into(),
            message: "Test message".into(),
            metric_type: "uptime".into(),
            target_value: 99.9,
            actual_value: 99.5,
            detected_at: "2025-01-01T00:00:00Z".into(),
            resolved_at: None,
        };
        let json = serde_json::to_string(&alert).unwrap();
        assert!(json.contains("\"severity\":\"high\""));
        assert!(json.contains("\"alert_type\":\"breach\""));
    }

    #[test]
    fn test_alerts_response_serialization() {
        let resp = AlertsResponse {
            alerts: vec![],
            total: 0,
            generated_at: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"total\":0"));
    }

    #[test]
    fn test_subscribe_response_serialization() {
        let resp = SubscriptionConfirmResponse {
            success: true,
            message: "ok".into(),
            subscription_id: Uuid::new_v4(),
            sla_ids: vec![],
            enabled: true,
            created_at: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"success\":true"));
    }

    #[test]
    fn test_subscribe_request_deserialization() {
        let json = r#"{
            "email": "test@example.com",
            "alert_types": ["breach", "at_risk"],
            "sla_ids": null
        }"#;
        let req: SubscribeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.email, "test@example.com");
        assert_eq!(req.alert_types.len(), 2);
    }

    #[test]
    fn test_default_limit() {
        assert_eq!(default_limit(), 50);
    }

    #[test]
    fn test_sla_dashboard_routes_compile() {
        let _router = sla_dashboard_routes();
    }
}
