#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::{CoreError, ErrorResponse};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post, delete},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct CodeQualityMetricResponseV7 {
    pub id: String,
    pub repo_id: String,
    pub file_path: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub measured_at: String,
}

#[derive(Debug, Serialize)]
pub struct CodeQualityThresholdResponseV6 {
    pub id: String,
    pub repo_id: String,
    pub metric_name: String,
    pub threshold_value: f64,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct RecordMetricRequestV7 {
    pub file_path: String,
    pub metric_name: String,
    pub metric_value: f64,
}

#[derive(Debug, Deserialize)]
pub struct CreateThresholdRequestV6 {
    pub metric_name: String,
    pub threshold_value: f64,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CheckViolationRequestV6 {
    pub metric_name: String,
    pub metric_value: f64,
}

#[derive(Debug, Serialize)]
pub struct CheckViolationResponseV6 {
    pub is_violation: bool,
    pub metric_name: String,
    pub metric_value: f64,
}

#[derive(Debug, Serialize)]
pub struct QualityScoreResponseV6 {
    pub repo_id: String,
    pub score: f64,
}

#[derive(Debug, Serialize)]
pub struct EnforcementReportResponseV6 {
    pub total_thresholds: i64,
    pub violating_thresholds: i64,
    pub compliance_rate: f64,
}

#[derive(Debug, Deserialize)]
pub struct ListMetricsParamsV7 {
    pub metric_name: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListThresholdsParamsV6 {
    #[serde(default)]
    pub enabled_only: bool,
}

fn default_limit() -> i64 {
    100
}

pub async fn record_metric_v7(
    State(state): State<AppState>,
    Path(repo_id): Path<Uuid>,
    _auth: AuthUser,
    Json(req): Json<RecordMetricRequestV7>,
) -> impl IntoResponse {
    match state
        .db
        .create_code_quality_metric_v7(repo_id, &req.file_path, &req.metric_name, req.metric_value)
        .await
    {
        Ok(metric) => (
            StatusCode::CREATED,
            Json(CodeQualityMetricResponseV7 {
                id: metric.id.to_string(),
                repo_id: metric.repo_id.to_string(),
                file_path: metric.file_path,
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

pub async fn list_metrics_v7(
    State(state): State<AppState>,
    Path(repo_id): Path<Uuid>,
    _auth: AuthUser,
    Query(params): Query<ListMetricsParamsV7>,
) -> impl IntoResponse {
    match state
        .db
        .list_code_quality_metrics_v7(repo_id, params.metric_name.as_deref(), params.limit, params.offset)
        .await
    {
        Ok(metrics) => {
            let resp: Vec<CodeQualityMetricResponseV7> = metrics
                .into_iter()
                .map(|m| CodeQualityMetricResponseV7 {
                    id: m.id.to_string(),
                    repo_id: m.repo_id.to_string(),
                    file_path: m.file_path,
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

pub async fn get_quality_score_v7(
    State(state): State<AppState>,
    Path(repo_id): Path<Uuid>,
    _auth: AuthUser,
) -> impl IntoResponse {
    match state.db.get_code_quality_score_v7(repo_id).await {
        Ok(score) => (
            StatusCode::OK,
            Json(QualityScoreResponseV6 {
                repo_id: repo_id.to_string(),
                score,
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

pub async fn create_threshold_v6(
    State(state): State<AppState>,
    Path(repo_id): Path<Uuid>,
    _auth: AuthUser,
    Json(req): Json<CreateThresholdRequestV6>,
) -> impl IntoResponse {
    let enabled = req.enabled.unwrap_or(true);
    match state
        .db
        .create_code_quality_threshold_v6(repo_id, &req.metric_name, req.threshold_value, enabled)
        .await
    {
        Ok(threshold) => (
            StatusCode::CREATED,
            Json(CodeQualityThresholdResponseV6 {
                id: threshold.id.to_string(),
                repo_id: threshold.repo_id.to_string(),
                metric_name: threshold.metric_name,
                threshold_value: threshold.threshold_value,
                enabled: threshold.enabled,
                created_at: threshold.created_at.to_rfc3339(),
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

pub async fn list_thresholds_v6(
    State(state): State<AppState>,
    Path(repo_id): Path<Uuid>,
    _auth: AuthUser,
    Query(params): Query<ListThresholdsParamsV6>,
) -> impl IntoResponse {
    match state
        .db
        .list_code_quality_thresholds_v6(repo_id, params.enabled_only)
        .await
    {
        Ok(thresholds) => {
            let resp: Vec<CodeQualityThresholdResponseV6> = thresholds
                .into_iter()
                .map(|t| CodeQualityThresholdResponseV6 {
                    id: t.id.to_string(),
                    repo_id: t.repo_id.to_string(),
                    metric_name: t.metric_name,
                    threshold_value: t.threshold_value,
                    enabled: t.enabled,
                    created_at: t.created_at.to_rfc3339(),
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

pub async fn check_violation_v6(
    State(state): State<AppState>,
    Path(repo_id): Path<Uuid>,
    _auth: AuthUser,
    Json(req): Json<CheckViolationRequestV6>,
) -> impl IntoResponse {
    match state
        .db
        .check_code_quality_violation_v6(repo_id, &req.metric_name, req.metric_value)
        .await
    {
        Ok(is_violation) => (
            StatusCode::OK,
            Json(CheckViolationResponseV6 {
                is_violation,
                metric_name: req.metric_name,
                metric_value: req.metric_value,
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

pub async fn delete_threshold_v6(
    State(state): State<AppState>,
    Path((repo_id, metric_name)): Path<(Uuid, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    match state
        .db
        .delete_code_quality_threshold_v6(repo_id, &metric_name)
        .await
    {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_enforcement_report_v6(
    State(state): State<AppState>,
    Path(repo_id): Path<Uuid>,
    _auth: AuthUser,
) -> impl IntoResponse {
    match state.db.get_code_quality_enforcement_report_v6(repo_id).await {
        Ok((total, violating, compliance_rate)) => (
            StatusCode::OK,
            Json(EnforcementReportResponseV6 {
                total_thresholds: total,
                violating_thresholds: violating,
                compliance_rate,
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

pub fn code_quality_v6_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{repo_id}/quality/metrics",
            post(record_metric_v7).get(list_metrics_v7),
        )
        .route(
            "/api/v1/repos/{repo_id}/quality/score",
            get(get_quality_score_v7),
        )
        .route(
            "/api/v1/repos/{repo_id}/quality/thresholds",
            post(create_threshold_v6).get(list_thresholds_v6),
        )
        .route(
            "/api/v1/repos/{repo_id}/quality/check-violation",
            post(check_violation_v6),
        )
        .route(
            "/api/v1/repos/{repo_id}/quality/thresholds/{metric_name}",
            delete(delete_threshold_v6),
        )
        .route(
            "/api/v1/repos/{repo_id}/quality/enforcement-report",
            get(get_enforcement_report_v6),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_response_v7_serializes() {
        let resp = CodeQualityMetricResponseV7 {
            id: "00000000-0000-0000-0000-000000000001".into(),
            repo_id: "00000000-0000-0000-0000-000000000002".into(),
            file_path: "src/main.rs".into(),
            metric_name: "complexity".into(),
            metric_value: 12.5,
            measured_at: "2024-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("complexity"));
        assert!(json.contains("12.5"));
    }

    #[test]
    fn test_threshold_response_v6_serializes() {
        let resp = CodeQualityThresholdResponseV6 {
            id: "00000000-0000-0000-0000-000000000001".into(),
            repo_id: "00000000-0000-0000-0000-000000000002".into(),
            metric_name: "cyclomatic_complexity".into(),
            threshold_value: 20.0,
            enabled: true,
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("cyclomatic_complexity"));
        assert!(json.contains("20"));
    }

    #[test]
    fn test_violation_check_v6_response() {
        let resp = CheckViolationResponseV6 {
            is_violation: false,
            metric_name: "code_coverage".into(),
            metric_value: 85.0,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("is_violation"));
        assert!(json.contains("false"));
    }

    #[test]
    fn test_quality_score_v6_response() {
        let resp = QualityScoreResponseV6 {
            repo_id: "00000000-0000-0000-0000-000000000001".into(),
            score: 92.5,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("score"));
        assert!(json.contains("92.5"));
    }

    #[test]
    fn test_enforcement_report_v6_response() {
        let resp = EnforcementReportResponseV6 {
            total_thresholds: 10,
            violating_thresholds: 2,
            compliance_rate: 80.0,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("compliance_rate"));
        assert!(json.contains("80"));
    }

    #[test]
    fn test_record_metric_v7_request() {
        let req: RecordMetricRequestV7 = serde_json::from_str(
            r#"{"file_path": "src/lib.rs", "metric_name": "lines_of_code", "metric_value": 500.0}"#,
        )
        .unwrap();
        assert_eq!(req.file_path, "src/lib.rs");
        assert_eq!(req.metric_name, "lines_of_code");
        assert_eq!(req.metric_value, 500.0);
    }

    #[test]
    fn test_create_threshold_v6_request() {
        let req: CreateThresholdRequestV6 = serde_json::from_str(
            r#"{"metric_name": "test_coverage", "threshold_value": 80.0, "enabled": true}"#,
        )
        .unwrap();
        assert_eq!(req.metric_name, "test_coverage");
        assert_eq!(req.threshold_value, 80.0);
        assert_eq!(req.enabled, Some(true));
    }
}
