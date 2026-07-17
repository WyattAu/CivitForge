#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct TestSuiteMetricResponse {
    pub id: String,
    pub suite_id: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub measured_at: String,
}

#[derive(Debug, Serialize)]
pub struct TestSuiteBaselineResponse {
    pub id: String,
    pub suite_id: String,
    pub metric_name: String,
    pub baseline_value: f64,
    pub threshold_percent: f64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct RecordMetricRequest {
    pub metric_name: String,
    pub metric_value: f64,
}

#[derive(Debug, Deserialize)]
pub struct CreateBaselineRequest {
    pub metric_name: String,
    pub baseline_value: f64,
    pub threshold_percent: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct RegressionCheckRequest {
    pub metric_name: String,
    pub current_value: f64,
}

#[derive(Debug, Serialize)]
pub struct RegressionCheckResponse {
    pub is_regression: bool,
    pub metric_name: String,
    pub current_value: f64,
}

#[derive(Debug, Serialize)]
pub struct PerformanceAlertResponse {
    pub alerts: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListMetricsParams {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    100
}

async fn record_metric(
    State(state): State<AppState>,
    Path(suite_id): Path<Uuid>,
    _auth: AuthUser,
    Json(req): Json<RecordMetricRequest>,
) -> impl IntoResponse {
    match state
        .db
        .create_test_suite_metric_v21(suite_id, &req.metric_name, req.metric_value)
        .await
    {
        Ok(metric) => (
            StatusCode::CREATED,
            Json(TestSuiteMetricResponse {
                id: metric.id.to_string(),
                suite_id: metric.suite_id.to_string(),
                metric_name: metric.metric_name,
                metric_value: metric.metric_value,
                measured_at: metric.measured_at.to_rfc3339(),
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

async fn list_metrics(
    State(state): State<AppState>,
    Path(suite_id): Path<Uuid>,
    _auth: AuthUser,
    Query(params): Query<ListMetricsParams>,
) -> impl IntoResponse {
    match state
        .db
        .list_test_suite_metrics_v21(suite_id, params.limit, params.offset)
        .await
    {
        Ok(metrics) => {
            let resp: Vec<TestSuiteMetricResponse> = metrics
                .into_iter()
                .map(|m| TestSuiteMetricResponse {
                    id: m.id.to_string(),
                    suite_id: m.suite_id.to_string(),
                    metric_name: m.metric_name,
                    metric_value: m.metric_value,
                    measured_at: m.measured_at.to_rfc3339(),
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

async fn get_latest_metric(
    State(state): State<AppState>,
    Path((suite_id, metric_name)): Path<(Uuid, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    match state
        .db
        .get_test_suite_latest_metric_v21(suite_id, &metric_name)
        .await
    {
        Ok(Some(metric)) => (
            StatusCode::OK,
            Json(TestSuiteMetricResponse {
                id: metric.id.to_string(),
                suite_id: metric.suite_id.to_string(),
                metric_name: metric.metric_name,
                metric_value: metric.metric_value,
                measured_at: metric.measured_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("metric not found".into()).error_response()),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

async fn create_baseline(
    State(state): State<AppState>,
    Path(suite_id): Path<Uuid>,
    _auth: AuthUser,
    Json(req): Json<CreateBaselineRequest>,
) -> impl IntoResponse {
    let threshold = req.threshold_percent.unwrap_or(10.0);
    match state
        .db
        .create_test_suite_baseline_v21(suite_id, &req.metric_name, req.baseline_value, threshold)
        .await
    {
        Ok(baseline) => (
            StatusCode::CREATED,
            Json(TestSuiteBaselineResponse {
                id: baseline.id.to_string(),
                suite_id: baseline.suite_id.to_string(),
                metric_name: baseline.metric_name,
                baseline_value: baseline.baseline_value,
                threshold_percent: baseline.threshold_percent,
                created_at: baseline.created_at.to_rfc3339(),
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

async fn list_baselines(
    State(state): State<AppState>,
    Path(suite_id): Path<Uuid>,
    _auth: AuthUser,
) -> impl IntoResponse {
    match state.db.get_test_suite_baselines_v21(suite_id).await {
        Ok(baselines) => {
            let resp: Vec<TestSuiteBaselineResponse> = baselines
                .into_iter()
                .map(|b| TestSuiteBaselineResponse {
                    id: b.id.to_string(),
                    suite_id: b.suite_id.to_string(),
                    metric_name: b.metric_name,
                    baseline_value: b.baseline_value,
                    threshold_percent: b.threshold_percent,
                    created_at: b.created_at.to_rfc3339(),
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

async fn check_regression(
    State(state): State<AppState>,
    Path(suite_id): Path<Uuid>,
    _auth: AuthUser,
    Json(req): Json<RegressionCheckRequest>,
) -> impl IntoResponse {
    match state
        .db
        .detect_test_suite_regression_v21(suite_id, &req.metric_name, req.current_value)
        .await
    {
        Ok(is_regression) => (
            StatusCode::OK,
            Json(RegressionCheckResponse {
                is_regression,
                metric_name: req.metric_name,
                current_value: req.current_value,
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

async fn get_performance_alerts(
    State(state): State<AppState>,
    Path(suite_id): Path<Uuid>,
    _auth: AuthUser,
) -> impl IntoResponse {
    match state.db.get_test_suite_performance_alerts_v21(suite_id).await {
        Ok(alerts) => (
            StatusCode::OK,
            Json(PerformanceAlertResponse { alerts }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub fn test_suite_management_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/test-suites/{suite_id}/metrics",
            post(record_metric).get(list_metrics),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/metrics/{metric_name}/latest",
            get(get_latest_metric),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/baselines",
            post(create_baseline).get(list_baselines),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/regression-check",
            post(check_regression),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/performance-alerts",
            get(get_performance_alerts),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/metrics/v13",
            post(record_metric).get(list_metrics),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/metrics/{metric_name}/latest/v13",
            get(get_latest_metric),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/baselines/v13",
            post(create_baseline).get(list_baselines),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/regression-check/v13",
            post(check_regression),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/performance-alerts/v13",
            get(get_performance_alerts),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/metrics/v14",
            post(record_metric).get(list_metrics),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/metrics/{metric_name}/latest/v14",
            get(get_latest_metric),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/baselines/v14",
            post(create_baseline).get(list_baselines),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/regression-check/v14",
            post(check_regression),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/performance-alerts/v14",
            get(get_performance_alerts),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/metrics/v15",
            post(record_metric).get(list_metrics),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/metrics/{metric_name}/latest/v15",
            get(get_latest_metric),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/baselines/v15",
            post(create_baseline).get(list_baselines),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/regression-check/v15",
            post(check_regression),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/performance-alerts/v15",
            get(get_performance_alerts),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/metrics/v16",
            post(record_metric).get(list_metrics),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/metrics/{metric_name}/latest/v16",
            get(get_latest_metric),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/baselines/v16",
            post(create_baseline).get(list_baselines),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/regression-check/v16",
            post(check_regression),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/performance-alerts/v16",
            get(get_performance_alerts),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/metrics/v17",
            post(record_metric).get(list_metrics),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/metrics/{metric_name}/latest/v17",
            get(get_latest_metric),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/baselines/v17",
            post(create_baseline).get(list_baselines),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/regression-check/v17",
            post(check_regression),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/performance-alerts/v17",
            get(get_performance_alerts),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/metrics/v19",
            post(record_metric).get(list_metrics),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/metrics/{metric_name}/latest/v19",
            get(get_latest_metric),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/baselines/v19",
            post(create_baseline).get(list_baselines),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/regression-check/v19",
            post(check_regression),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/performance-alerts/v19",
            get(get_performance_alerts),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/metrics/v20",
            post(record_metric).get(list_metrics),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/metrics/{metric_name}/latest/v20",
            get(get_latest_metric),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/baselines/v20",
            post(create_baseline).get(list_baselines),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/regression-check/v20",
            post(check_regression),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/performance-alerts/v20",
            get(get_performance_alerts),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/metrics/v21",
            post(record_metric).get(list_metrics),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/metrics/{metric_name}/latest/v21",
            get(get_latest_metric),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/baselines/v21",
            post(create_baseline).get(list_baselines),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/regression-check/v21",
            post(check_regression),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/performance-alerts/v21",
            get(get_performance_alerts),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_response_serializes() {
        let resp = TestSuiteMetricResponse {
            id: "00000000-0000-0000-0000-000000000001".into(),
            suite_id: "00000000-0000-0000-0000-000000000002".into(),
            metric_name: "execution_time_ms".into(),
            metric_value: 150.5,
            measured_at: "2024-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("execution_time_ms"));
        assert!(json.contains("150.5"));
    }

    #[test]
    fn test_baseline_response_serializes() {
        let resp = TestSuiteBaselineResponse {
            id: "00000000-0000-0000-0000-000000000001".into(),
            suite_id: "00000000-0000-0000-0000-000000000002".into(),
            metric_name: "memory_usage_mb".into(),
            baseline_value: 256.0,
            threshold_percent: 15.0,
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("memory_usage_mb"));
        assert!(json.contains("256"));
    }

    #[test]
    fn test_regression_check_response() {
        let resp = RegressionCheckResponse {
            is_regression: true,
            metric_name: "cpu_usage".into(),
            current_value: 95.0,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("is_regression"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_record_metric_request() {
        let req: RecordMetricRequest = serde_json::from_str(
            r#"{"metric_name": "test_duration", "metric_value": 42.5}"#,
        )
        .unwrap();
        assert_eq!(req.metric_name, "test_duration");
        assert_eq!(req.metric_value, 42.5);
    }

    #[test]
    fn test_create_baseline_request() {
        let req: CreateBaselineRequest = serde_json::from_str(
            r#"{"metric_name": "response_time", "baseline_value": 100.0, "threshold_percent": 20.0}"#,
        )
        .unwrap();
        assert_eq!(req.metric_name, "response_time");
        assert_eq!(req.baseline_value, 100.0);
        assert_eq!(req.threshold_percent, Some(20.0));
    }
}
