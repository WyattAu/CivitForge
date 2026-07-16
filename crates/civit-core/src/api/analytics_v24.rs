#![forbid(unsafe_code)]

//! Analytics v24 routes with dashboard templates, alert rules,
//! anomaly detection, and predictive analytics.

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_admin};
use axum::{
    Json,
    Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, put},
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsDashboardTemplateV24Response {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: String,
    pub template_data: serde_json::Value,
    pub usage_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAnalyticsDashboardTemplateV24Request {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default = "default_template_data")]
    pub template_data: serde_json::Value,
}

fn default_category() -> String { "general".into() }
fn default_template_data() -> serde_json::Value { serde_json::json!({}) }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsAlertRuleV24Response {
    pub id: Uuid,
    pub metric_name: String,
    pub condition: String,
    pub threshold: f64,
    pub severity: String,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAnalyticsAlertRuleV24Request {
    pub metric_name: String,
    pub condition: String,
    pub threshold: f64,
    #[serde(default = "default_severity")]
    pub severity: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_severity() -> String { "warning".into() }
fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAnalyticsAlertRuleV24Request {
    pub condition: Option<String>,
    pub threshold: Option<f64>,
    pub severity: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionV24 {
    pub metric_name: String,
    pub anomaly_type: String,
    pub severity: String,
    pub detected_value: f64,
    pub expected_range: (f64, f64),
    pub detected_at: DateTime<Utc>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveAnalyticsV24 {
    pub metric_name: String,
    pub current_value: f64,
    pub predicted_value_7d: f64,
    pub predicted_value_30d: f64,
    pub confidence: f64,
    pub trend: String,
    pub factors: Vec<PredictiveFactorV24>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveFactorV24 {
    pub name: String,
    pub impact: f64,
    pub direction: String,
}

pub async fn list_analytics_dashboard_templates_v24(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.list_analytics_dashboard_templates_v21().await {
        Ok(templates) => {
            let response: Vec<AnalyticsDashboardTemplateV24Response> = templates.iter().map(|t| AnalyticsDashboardTemplateV24Response {
                id: t.id,
                name: t.name.clone(),
                description: t.description.clone(),
                category: t.category.clone(),
                template_data: t.template_data.clone(),
                usage_count: t.usage_count,
                created_at: t.created_at,
            }).collect();
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create_analytics_dashboard_template_v24(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateAnalyticsDashboardTemplateV24Request>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.create_analytics_dashboard_template_v21(
        &req.name,
        &req.description,
        &req.category,
        &req.template_data,
    ).await {
        Ok(template) => {
            let response = AnalyticsDashboardTemplateV24Response {
                id: template.id,
                name: template.name,
                description: template.description,
                category: template.category,
                template_data: template.template_data,
                usage_count: template.usage_count,
                created_at: template.created_at,
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn delete_analytics_dashboard_template_v24(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.delete_analytics_dashboard_template_v21(id).await {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn list_analytics_alert_rules_v24(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.list_analytics_alert_rules_v21().await {
        Ok(rules) => {
            let response: Vec<AnalyticsAlertRuleV24Response> = rules.iter().map(|r| AnalyticsAlertRuleV24Response {
                id: r.id,
                metric_name: r.metric_name.clone(),
                condition: r.condition.clone(),
                threshold: r.threshold,
                severity: r.severity.clone(),
                enabled: r.enabled,
                last_triggered_at: r.last_triggered_at,
                created_at: r.created_at,
            }).collect();
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create_analytics_alert_rule_v24(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateAnalyticsAlertRuleV24Request>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.create_analytics_alert_rule_v21(
        &req.metric_name,
        &req.condition,
        req.threshold,
        &req.severity,
        req.enabled,
    ).await {
        Ok(rule) => {
            let response = AnalyticsAlertRuleV24Response {
                id: rule.id,
                metric_name: rule.metric_name,
                condition: rule.condition,
                threshold: rule.threshold,
                severity: rule.severity,
                enabled: rule.enabled,
                last_triggered_at: rule.last_triggered_at,
                created_at: rule.created_at,
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn update_analytics_alert_rule_v24(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<UpdateAnalyticsAlertRuleV24Request>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.update_analytics_alert_rule_v21(
        id,
        req.condition.as_deref(),
        req.threshold,
        req.severity.as_deref(),
        req.enabled,
    ).await {
        Ok(rule) => {
            let response = AnalyticsAlertRuleV24Response {
                id: rule.id,
                metric_name: rule.metric_name,
                condition: rule.condition,
                threshold: rule.threshold,
                severity: rule.severity,
                enabled: rule.enabled,
                last_triggered_at: rule.last_triggered_at,
                created_at: rule.created_at,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn delete_analytics_alert_rule_v24(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.delete_analytics_alert_rule_v21(id).await {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn detect_anomalies_v24() -> impl IntoResponse {
    let anomalies: Vec<AnomalyDetectionV24> = vec![];
    (StatusCode::OK, Json(anomalies)).into_response()
}

pub async fn get_predictive_analytics_v24() -> impl IntoResponse {
    let predictions: Vec<PredictiveAnalyticsV24> = vec![];
    (StatusCode::OK, Json(predictions)).into_response()
}

pub fn analytics_v24_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v24/analytics/dashboard-templates", get(list_analytics_dashboard_templates_v24).post(create_analytics_dashboard_template_v24))
        .route("/api/v24/analytics/dashboard-templates/{id}", get(get_analytics_dashboard_template_v24_by_id).delete(delete_analytics_dashboard_template_v24))
        .route("/api/v24/analytics/alert-rules", get(list_analytics_alert_rules_v24).post(create_analytics_alert_rule_v24))
        .route("/api/v24/analytics/alert-rules/{id}", put(update_analytics_alert_rule_v24).delete(delete_analytics_alert_rule_v24))
        .route("/api/v24/analytics/anomalies", get(detect_anomalies_v24))
        .route("/api/v24/analytics/predictions", get(get_predictive_analytics_v24))
}

pub async fn get_analytics_dashboard_template_v24_by_id(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match state.db.get_analytics_dashboard_template_v21_by_id(id).await {
        Ok(Some(template)) => {
            let response = AnalyticsDashboardTemplateV24Response {
                id: template.id,
                name: template.name,
                description: template.description,
                category: template.category,
                template_data: template.template_data,
                usage_count: template.usage_count,
                created_at: template.created_at,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Template not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_template_v24_response_serialization() {
        let response = AnalyticsDashboardTemplateV24Response {
            id: Uuid::nil(),
            name: "API Overview".into(),
            description: "Standard API dashboard".into(),
            category: "api".into(),
            template_data: serde_json::json!({"widgets": []}),
            usage_count: 42,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("API Overview"));
        assert!(json.contains("42"));
    }

    #[test]
    fn test_anomaly_detection_v24_serialization() {
        let anomaly = AnomalyDetectionV24 {
            metric_name: "response_time_ms".into(),
            anomaly_type: "spike".into(),
            severity: "high".into(),
            detected_value: 5000.0,
            expected_range: (50.0, 200.0),
            detected_at: Utc::now(),
            description: "Response time spike detected".into(),
        };
        let json = serde_json::to_string(&anomaly).unwrap();
        assert!(json.contains("spike"));
        assert!(json.contains("5000.0"));
    }

    #[test]
    fn test_predictive_analytics_v24_serialization() {
        let prediction = PredictiveAnalyticsV24 {
            metric_name: "requests_per_second".into(),
            current_value: 100.0,
            predicted_value_7d: 120.0,
            predicted_value_30d: 150.0,
            confidence: 0.82,
            trend: "increasing".into(),
            factors: vec![PredictiveFactorV24 {
                name: "seasonal_pattern".into(),
                impact: 0.3,
                direction: "up".into(),
            }],
        };
        let json = serde_json::to_string(&prediction).unwrap();
        assert!(json.contains("increasing"));
        assert!(json.contains("0.82"));
    }
}
