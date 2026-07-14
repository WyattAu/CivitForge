#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::{CoreError, ErrorResponse};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post, patch},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct PerformanceTestAlertResponseV8 {
    pub id: String,
    pub baseline_id: String,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct PerformanceTestAlertHistoryResponseV8 {
    pub id: String,
    pub alert_id: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAlertRequestV8 {
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAlertRequestV8 {
    pub threshold: Option<f64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RecordAlertTriggerRequestV8 {
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
}

#[derive(Debug, Serialize)]
pub struct AlertAnalyticsResponseV8 {
    pub alert_id: String,
    pub trigger_count: i64,
    pub avg_value: Option<f64>,
    pub max_value: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct AlertNotificationConfigResponseV8 {
    pub alert_id: String,
    pub alert_type: String,
    pub enabled: bool,
    pub last_triggered_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListAlertsParamsV8 {
    #[serde(default)]
    pub enabled_only: bool,
}

#[derive(Debug, Deserialize)]
pub struct ListHistoryParamsV8 {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    100
}

pub async fn create_alert_v8(
    State(state): State<AppState>,
    Path(baseline_id): Path<Uuid>,
    _auth: AuthUser,
    Json(req): Json<CreateAlertRequestV8>,
) -> impl IntoResponse {
    let enabled = req.enabled.unwrap_or(true);
    match state
        .db
        .create_performance_test_alert_v8(baseline_id, &req.alert_type, req.threshold, enabled)
        .await
    {
        Ok(alert) => (
            StatusCode::CREATED,
            Json(PerformanceTestAlertResponseV8 {
                id: alert.id.to_string(),
                baseline_id: alert.baseline_id.to_string(),
                alert_type: alert.alert_type,
                threshold: alert.threshold,
                enabled: alert.enabled,
                last_triggered_at: alert.last_triggered_at.map(|dt| dt.to_rfc3339()),
                created_at: alert.created_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_alerts_v8(
    State(state): State<AppState>,
    Path(baseline_id): Path<Uuid>,
    _auth: AuthUser,
    Query(params): Query<ListAlertsParamsV8>,
) -> impl IntoResponse {
    match state
        .db
        .list_performance_test_alerts_v8(baseline_id, params.enabled_only)
        .await
    {
        Ok(alerts) => {
            let resp: Vec<PerformanceTestAlertResponseV8> = alerts
                .into_iter()
                .map(|a| PerformanceTestAlertResponseV8 {
                    id: a.id.to_string(),
                    baseline_id: a.baseline_id.to_string(),
                    alert_type: a.alert_type,
                    threshold: a.threshold,
                    enabled: a.enabled,
                    last_triggered_at: a.last_triggered_at.map(|dt| dt.to_rfc3339()),
                    created_at: a.created_at.to_rfc3339(),
                })
                .collect();
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_alert_v8(
    State(state): State<AppState>,
    Path(alert_id): Path<Uuid>,
    _auth: AuthUser,
    Json(req): Json<UpdateAlertRequestV8>,
) -> impl IntoResponse {
    match state
        .db
        .update_performance_test_alert_v8(alert_id, req.threshold, req.enabled)
        .await
    {
        Ok(alert) => (
            StatusCode::OK,
            Json(PerformanceTestAlertResponseV8 {
                id: alert.id.to_string(),
                baseline_id: alert.baseline_id.to_string(),
                alert_type: alert.alert_type,
                threshold: alert.threshold,
                enabled: alert.enabled,
                last_triggered_at: alert.last_triggered_at.map(|dt| dt.to_rfc3339()),
                created_at: alert.created_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn record_alert_trigger_v8(
    State(state): State<AppState>,
    Path(alert_id): Path<Uuid>,
    _auth: AuthUser,
    Json(req): Json<RecordAlertTriggerRequestV8>,
) -> impl IntoResponse {
    match state
        .db
        .record_performance_test_alert_v8(alert_id, &req.metric_name, req.metric_value, req.threshold)
        .await
    {
        Ok(history) => (
            StatusCode::CREATED,
            Json(PerformanceTestAlertHistoryResponseV8 {
                id: history.id.to_string(),
                alert_id: history.alert_id.to_string(),
                metric_name: history.metric_name,
                metric_value: history.metric_value,
                threshold: history.threshold,
                created_at: history.created_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_alert_history_v8(
    State(state): State<AppState>,
    Path(alert_id): Path<Uuid>,
    _auth: AuthUser,
    Query(params): Query<ListHistoryParamsV8>,
) -> impl IntoResponse {
    match state
        .db
        .list_performance_test_alert_history_v8(alert_id, params.limit, params.offset)
        .await
    {
        Ok(history) => {
            let resp: Vec<PerformanceTestAlertHistoryResponseV8> = history
                .into_iter()
                .map(|h| PerformanceTestAlertHistoryResponseV8 {
                    id: h.id.to_string(),
                    alert_id: h.alert_id.to_string(),
                    metric_name: h.metric_name,
                    metric_value: h.metric_value,
                    threshold: h.threshold,
                    created_at: h.created_at.to_rfc3339(),
                })
                .collect();
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_alert_analytics_v8(
    State(state): State<AppState>,
    Path(alert_id): Path<Uuid>,
    _auth: AuthUser,
) -> impl IntoResponse {
    match state.db.get_performance_test_alert_analytics_v8(alert_id).await {
        Ok((trigger_count, avg_value, max_value)) => (
            StatusCode::OK,
            Json(AlertAnalyticsResponseV8 {
                alert_id: alert_id.to_string(),
                trigger_count,
                avg_value,
                max_value,
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_alert_notification_config_v8(
    State(state): State<AppState>,
    Path(alert_id): Path<Uuid>,
    _auth: AuthUser,
) -> impl IntoResponse {
    match state.db.get_performance_test_alert_notification_config_v8(alert_id).await {
        Ok(Some((alert_type, enabled, last_triggered_at))) => (
            StatusCode::OK,
            Json(AlertNotificationConfigResponseV8 {
                alert_id: alert_id.to_string(),
                alert_type,
                enabled,
                last_triggered_at,
            }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("alert not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub fn performance_testing_v8_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/performance/baselines/{baseline_id}/alerts/v8",
            post(create_alert_v8).get(list_alerts_v8),
        )
        .route(
            "/api/v1/performance/alerts/{alert_id}/v8",
            patch(update_alert_v8),
        )
        .route(
            "/api/v1/performance/alerts/{alert_id}/trigger/v8",
            post(record_alert_trigger_v8),
        )
        .route(
            "/api/v1/performance/alerts/{alert_id}/history/v8",
            get(list_alert_history_v8),
        )
        .route(
            "/api/v1/performance/alerts/{alert_id}/analytics/v8",
            get(get_alert_analytics_v8),
        )
        .route(
            "/api/v1/performance/alerts/{alert_id}/notification-config/v8",
            get(get_alert_notification_config_v8),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_response_v8_serializes() {
        let resp = PerformanceTestAlertResponseV8 {
            id: "00000000-0000-0000-0000-000000000001".into(),
            baseline_id: "00000000-0000-0000-0000-000000000002".into(),
            alert_type: "regression".into(),
            threshold: 10.0,
            enabled: true,
            last_triggered_at: Some("2024-01-01T00:00:00Z".into()),
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("regression"));
        assert!(json.contains("10"));
    }

    #[test]
    fn test_alert_history_response_v8_serializes() {
        let resp = PerformanceTestAlertHistoryResponseV8 {
            id: "00000000-0000-0000-0000-000000000001".into(),
            alert_id: "00000000-0000-0000-0000-000000000002".into(),
            metric_name: "response_time".into(),
            metric_value: 250.0,
            threshold: 200.0,
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("response_time"));
        assert!(json.contains("250"));
    }

    #[test]
    fn test_analytics_response_v8_serializes() {
        let resp = AlertAnalyticsResponseV8 {
            alert_id: "00000000-0000-0000-0000-000000000001".into(),
            trigger_count: 42,
            avg_value: Some(150.5),
            max_value: Some(300.0),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("trigger_count"));
        assert!(json.contains("42"));
    }

    #[test]
    fn test_notification_config_response_v8_serializes() {
        let resp = AlertNotificationConfigResponseV8 {
            alert_id: "00000000-0000-0000-0000-000000000001".into(),
            alert_type: "threshold".into(),
            enabled: true,
            last_triggered_at: Some("2024-01-01T00:00:00Z".into()),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("threshold"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_create_alert_v8_request() {
        let req: CreateAlertRequestV8 = serde_json::from_str(
            r#"{"alert_type": "threshold", "threshold": 50.0, "enabled": true}"#,
        )
        .unwrap();
        assert_eq!(req.alert_type, "threshold");
        assert_eq!(req.threshold, 50.0);
        assert_eq!(req.enabled, Some(true));
    }

    #[test]
    fn test_update_alert_v8_request() {
        let req: UpdateAlertRequestV8 = serde_json::from_str(
            r#"{"threshold": 75.0, "enabled": false}"#,
        )
        .unwrap();
        assert_eq!(req.threshold, Some(75.0));
        assert_eq!(req.enabled, Some(false));
    }

    #[test]
    fn test_record_alert_trigger_v8_request() {
        let req: RecordAlertTriggerRequestV8 = serde_json::from_str(
            r#"{"metric_name": "latency_p99", "metric_value": 500.0, "threshold": 400.0}"#,
        )
        .unwrap();
        assert_eq!(req.metric_name, "latency_p99");
        assert_eq!(req.metric_value, 500.0);
        assert_eq!(req.threshold, 400.0);
    }
}
