#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, patch, post},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct PerformanceTestAlertResponse {
    pub id: String,
    pub baseline_id: String,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct PerformanceTestAlertHistoryResponse {
    pub id: String,
    pub alert_id: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAlertRequest {
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAlertRequest {
    pub threshold: Option<f64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RecordAlertTriggerRequest {
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
}

#[derive(Debug, Serialize)]
pub struct AlertAnalyticsResponse {
    pub alert_id: String,
    pub trigger_count: i64,
    pub avg_value: Option<f64>,
    pub max_value: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct AlertNotificationConfigResponse {
    pub alert_id: String,
    pub alert_type: String,
    pub enabled: bool,
    pub last_triggered_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListAlertsParams {
    #[serde(default)]
    pub enabled_only: bool,
}

#[derive(Debug, Deserialize)]
pub struct ListHistoryParams {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    100
}

async fn create_alert(
    State(state): State<AppState>,
    Path(baseline_id): Path<Uuid>,
    _auth: AuthUser,
    Json(req): Json<CreateAlertRequest>,
) -> impl IntoResponse {
    let enabled = req.enabled.unwrap_or(true);
    match state
        .db
        .create_performance_test_alert_v21(baseline_id, &req.alert_type, req.threshold, enabled)
        .await
    {
        Ok(alert) => (
            StatusCode::CREATED,
            Json(PerformanceTestAlertResponse {
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

async fn list_alerts(
    State(state): State<AppState>,
    Path(baseline_id): Path<Uuid>,
    _auth: AuthUser,
    Query(params): Query<ListAlertsParams>,
) -> impl IntoResponse {
    match state
        .db
        .list_performance_test_alerts_v21(baseline_id, params.enabled_only)
        .await
    {
        Ok(alerts) => {
            let resp: Vec<PerformanceTestAlertResponse> = alerts
                .into_iter()
                .map(|a| PerformanceTestAlertResponse {
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

async fn update_alert(
    State(state): State<AppState>,
    Path(alert_id): Path<Uuid>,
    _auth: AuthUser,
    Json(req): Json<UpdateAlertRequest>,
) -> impl IntoResponse {
    match state
        .db
        .update_performance_test_alert_v21(alert_id, req.threshold, req.enabled)
        .await
    {
        Ok(alert) => (
            StatusCode::OK,
            Json(PerformanceTestAlertResponse {
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

async fn record_alert_trigger(
    State(state): State<AppState>,
    Path(alert_id): Path<Uuid>,
    _auth: AuthUser,
    Json(req): Json<RecordAlertTriggerRequest>,
) -> impl IntoResponse {
    match state
        .db
        .record_performance_test_alert_v21(alert_id, &req.metric_name, req.metric_value, req.threshold)
        .await
    {
        Ok(history) => (
            StatusCode::CREATED,
            Json(PerformanceTestAlertHistoryResponse {
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

async fn list_alert_history(
    State(state): State<AppState>,
    Path(alert_id): Path<Uuid>,
    _auth: AuthUser,
    Query(params): Query<ListHistoryParams>,
) -> impl IntoResponse {
    match state
        .db
        .list_performance_test_alert_history_v21(alert_id, params.limit, params.offset)
        .await
    {
        Ok(history) => {
            let resp: Vec<PerformanceTestAlertHistoryResponse> = history
                .into_iter()
                .map(|h| PerformanceTestAlertHistoryResponse {
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

async fn get_alert_analytics(
    State(state): State<AppState>,
    Path(alert_id): Path<Uuid>,
    _auth: AuthUser,
) -> impl IntoResponse {
    match state.db.get_performance_test_alert_analytics_v21(alert_id).await {
        Ok((trigger_count, avg_value, max_value)) => (
            StatusCode::OK,
            Json(AlertAnalyticsResponse {
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

async fn get_alert_notification_config(
    State(state): State<AppState>,
    Path(alert_id): Path<Uuid>,
    _auth: AuthUser,
) -> impl IntoResponse {
    match state.db.get_performance_test_alert_notification_config_v21(alert_id).await {
        Ok(Some((alert_type, enabled, last_triggered_at))) => (
            StatusCode::OK,
            Json(AlertNotificationConfigResponse {
                alert_id: alert_id.to_string(),
                alert_type,
                enabled,
                last_triggered_at: last_triggered_at.map(|dt| dt.to_rfc3339()),
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

pub fn performance_testing_routes() -> Router<AppState> {
    let mut router = Router::new();

    let versions = [6, 7, 8, 9, 14, 15, 16, 17, 18, 20, 21];
    for v in versions {
        let suffix = format!("/v{v}");
        router = router
            .route(
                &format!("/api/v1/performance/baselines/{{baseline_id}}/alerts{suffix}"),
                post(create_alert).get(list_alerts),
            )
            .route(
                &format!("/api/v1/performance/alerts/{{alert_id}}{suffix}"),
                patch(update_alert),
            )
            .route(
                &format!("/api/v1/performance/alerts/{{alert_id}}/trigger{suffix}"),
                post(record_alert_trigger),
            )
            .route(
                &format!("/api/v1/performance/alerts/{{alert_id}}/history{suffix}"),
                get(list_alert_history),
            )
            .route(
                &format!("/api/v1/performance/alerts/{{alert_id}}/analytics{suffix}"),
                get(get_alert_analytics),
            )
            .route(
                &format!("/api/v1/performance/alerts/{{alert_id}}/notification-config{suffix}"),
                get(get_alert_notification_config),
            );
    }

    router
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_response_serializes() {
        let resp = PerformanceTestAlertResponse {
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
    fn test_create_alert_request() {
        let req: CreateAlertRequest = serde_json::from_str(
            r#"{"alert_type": "threshold", "threshold": 50.0, "enabled": true}"#,
        )
        .unwrap();
        assert_eq!(req.alert_type, "threshold");
        assert_eq!(req.threshold, 50.0);
        assert_eq!(req.enabled, Some(true));
    }

    #[test]
    fn test_update_alert_request() {
        let req: UpdateAlertRequest = serde_json::from_str(
            r#"{"threshold": 75.0, "enabled": false}"#,
        )
        .unwrap();
        assert_eq!(req.threshold, Some(75.0));
        assert_eq!(req.enabled, Some(false));
    }

    #[test]
    fn test_record_alert_trigger_request() {
        let req: RecordAlertTriggerRequest = serde_json::from_str(
            r#"{"metric_name": "latency_p99", "metric_value": 500.0, "threshold": 400.0}"#,
        )
        .unwrap();
        assert_eq!(req.metric_name, "latency_p99");
        assert_eq!(req.metric_value, 500.0);
        assert_eq!(req.threshold, 400.0);
    }
}
