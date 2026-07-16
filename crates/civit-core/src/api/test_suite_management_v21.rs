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
pub struct TestSuiteMetricResponseV21 {
    pub id: String,
    pub suite_id: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub measured_at: String,
}

#[derive(Debug, Serialize)]
pub struct TestSuiteBaselineResponseV21 {
    pub id: String,
    pub suite_id: String,
    pub metric_name: String,
    pub baseline_value: f64,
    pub threshold_percent: f64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct RecordMetricRequestV21 {
    pub metric_name: String,
    pub metric_value: f64,
}

#[derive(Debug, Deserialize)]
pub struct CreateBaselineRequestV21 {
    pub metric_name: String,
    pub baseline_value: f64,
    pub threshold_percent: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct RegressionCheckRequestV21 {
    pub metric_name: String,
    pub current_value: f64,
}

#[derive(Debug, Serialize)]
pub struct RegressionCheckResponseV21 {
    pub is_regression: bool,
    pub metric_name: String,
    pub current_value: f64,
}

#[derive(Debug, Serialize)]
pub struct PerformanceAlertResponseV21 {
    pub alerts: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListMetricsParamsV21 {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    100
}

pub async fn record_metric_v21(
    State(state): State<AppState>,
    Path(suite_id): Path<Uuid>,
    _auth: AuthUser,
    Json(req): Json<RecordMetricRequestV21>,
) -> impl IntoResponse {
    match state
        .db
        .create_test_suite_metric_v21(suite_id, &req.metric_name, req.metric_value)
        .await
    {
        Ok(metric) => (
            StatusCode::CREATED,
            Json(TestSuiteMetricResponseV21 {
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

pub async fn list_metrics_v21(
    State(state): State<AppState>,
    Path(suite_id): Path<Uuid>,
    _auth: AuthUser,
    Query(params): Query<ListMetricsParamsV21>,
) -> impl IntoResponse {
    match state
        .db
        .list_test_suite_metrics_v21(suite_id, params.limit, params.offset)
        .await
    {
        Ok(metrics) => {
            let resp: Vec<TestSuiteMetricResponseV21> = metrics
                .into_iter()
                .map(|m| TestSuiteMetricResponseV21 {
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

pub async fn get_latest_metric_v21(
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
            Json(TestSuiteMetricResponseV21 {
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

pub async fn create_baseline_v21(
    State(state): State<AppState>,
    Path(suite_id): Path<Uuid>,
    _auth: AuthUser,
    Json(req): Json<CreateBaselineRequestV21>,
) -> impl IntoResponse {
    let threshold = req.threshold_percent.unwrap_or(10.0);
    match state
        .db
        .create_test_suite_baseline_v21(suite_id, &req.metric_name, req.baseline_value, threshold)
        .await
    {
        Ok(baseline) => (
            StatusCode::CREATED,
            Json(TestSuiteBaselineResponseV21 {
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

pub async fn list_baselines_v21(
    State(state): State<AppState>,
    Path(suite_id): Path<Uuid>,
    _auth: AuthUser,
) -> impl IntoResponse {
    match state.db.get_test_suite_baselines_v21(suite_id).await {
        Ok(baselines) => {
            let resp: Vec<TestSuiteBaselineResponseV21> = baselines
                .into_iter()
                .map(|b| TestSuiteBaselineResponseV21 {
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

pub async fn check_regression_v21(
    State(state): State<AppState>,
    Path(suite_id): Path<Uuid>,
    _auth: AuthUser,
    Json(req): Json<RegressionCheckRequestV21>,
) -> impl IntoResponse {
    match state
        .db
        .detect_test_suite_regression_v21(suite_id, &req.metric_name, req.current_value)
        .await
    {
        Ok(is_regression) => (
            StatusCode::OK,
            Json(RegressionCheckResponseV21 {
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

pub async fn get_performance_alerts_v21(
    State(state): State<AppState>,
    Path(suite_id): Path<Uuid>,
    _auth: AuthUser,
) -> impl IntoResponse {
    match state.db.get_test_suite_performance_alerts_v21(suite_id).await {
        Ok(alerts) => (
            StatusCode::OK,
            Json(PerformanceAlertResponseV21 { alerts }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub fn test_suite_v21_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/test-suites/{suite_id}/metrics/v21",
            post(record_metric_v21).get(list_metrics_v21),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/metrics/{metric_name}/latest/v21",
            get(get_latest_metric_v21),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/baselines/v21",
            post(create_baseline_v21).get(list_baselines_v21),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/regression-check/v21",
            post(check_regression_v21),
        )
        .route(
            "/api/v1/test-suites/{suite_id}/performance-alerts/v21",
            get(get_performance_alerts_v21),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_response_v21_serializes() {
        let resp = TestSuiteMetricResponseV21 {
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
    fn test_baseline_response_v21_serializes() {
        let resp = TestSuiteBaselineResponseV21 {
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
    fn test_regression_check_v21_response() {
        let resp = RegressionCheckResponseV21 {
            is_regression: true,
            metric_name: "cpu_usage".into(),
            current_value: 95.0,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("is_regression"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_performance_alerts_v21_response() {
        let resp = PerformanceAlertResponseV21 {
            alerts: vec!["High execution time: 1500ms".into()],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("High execution time"));
    }

    #[test]
    fn test_record_metric_v21_request() {
        let req: RecordMetricRequestV21 = serde_json::from_str(
            r#"{"metric_name": "test_duration", "metric_value": 42.5}"#,
        )
        .unwrap();
        assert_eq!(req.metric_name, "test_duration");
        assert_eq!(req.metric_value, 42.5);
    }

    #[test]
    fn test_create_baseline_v21_request() {
        let req: CreateBaselineRequestV21 = serde_json::from_str(
            r#"{"metric_name": "response_time", "baseline_value": 100.0, "threshold_percent": 20.0}"#,
        )
        .unwrap();
        assert_eq!(req.metric_name, "response_time");
        assert_eq!(req.baseline_value, 100.0);
        assert_eq!(req.threshold_percent, Some(20.0));
    }
}
