#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct CodeQualityMetricResponse {
    pub id: String,
    pub repo_id: String,
    pub file_path: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub measured_at: String,
}

#[derive(Debug, Serialize)]
pub struct CodeQualityThresholdResponse {
    pub id: String,
    pub repo_id: String,
    pub metric_name: String,
    pub threshold_value: f64,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct RecordMetricRequest {
    pub file_path: String,
    pub metric_name: String,
    pub metric_value: f64,
}

#[derive(Debug, Deserialize)]
pub struct CreateThresholdRequest {
    pub metric_name: String,
    pub threshold_value: f64,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CheckViolationRequest {
    pub metric_name: String,
    pub metric_value: f64,
}

#[derive(Debug, Serialize)]
pub struct CheckViolationResponse {
    pub is_violation: bool,
    pub metric_name: String,
    pub metric_value: f64,
}

#[derive(Debug, Serialize)]
pub struct QualityScoreResponse {
    pub repo_id: String,
    pub score: f64,
}

#[derive(Debug, Serialize)]
pub struct EnforcementReportResponse {
    pub total_thresholds: i64,
    pub violating_thresholds: i64,
    pub compliance_rate: f64,
}

#[derive(Debug, Deserialize)]
pub struct ListMetricsParams {
    pub metric_name: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListThresholdsParams {
    #[serde(default)]
    pub enabled_only: bool,
}

fn default_limit() -> i64 {
    100
}

async fn record_metric(
    State(state): State<AppState>,
    Path(repo_id): Path<Uuid>,
    _auth: AuthUser,
    Json(req): Json<RecordMetricRequest>,
) -> impl IntoResponse {
    match state
        .db
        .create_code_quality_metric_v20(repo_id, &req.file_path, &req.metric_name, req.metric_value)
        .await
    {
        Ok(metric) => (
            StatusCode::CREATED,
            Json(CodeQualityMetricResponse {
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

async fn list_metrics(
    State(state): State<AppState>,
    Path(repo_id): Path<Uuid>,
    _auth: AuthUser,
    Query(params): Query<ListMetricsParams>,
) -> impl IntoResponse {
    match state
        .db
        .list_code_quality_metrics_v20(repo_id, params.metric_name.as_deref(), params.limit, params.offset)
        .await
    {
        Ok(metrics) => {
            let resp: Vec<CodeQualityMetricResponse> = metrics
                .into_iter()
                .map(|m| CodeQualityMetricResponse {
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

async fn get_quality_score(
    State(state): State<AppState>,
    Path(repo_id): Path<Uuid>,
    _auth: AuthUser,
) -> impl IntoResponse {
    match state.db.get_code_quality_score_v20(repo_id).await {
        Ok(score) => (
            StatusCode::OK,
            Json(QualityScoreResponse {
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

async fn create_threshold(
    State(state): State<AppState>,
    Path(repo_id): Path<Uuid>,
    _auth: AuthUser,
    Json(req): Json<CreateThresholdRequest>,
) -> impl IntoResponse {
    let enabled = req.enabled.unwrap_or(true);
    match state
        .db
        .create_code_quality_threshold_v20(repo_id, &req.metric_name, req.threshold_value, enabled)
        .await
    {
        Ok(threshold) => (
            StatusCode::CREATED,
            Json(CodeQualityThresholdResponse {
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

async fn list_thresholds(
    State(state): State<AppState>,
    Path(repo_id): Path<Uuid>,
    _auth: AuthUser,
    Query(params): Query<ListThresholdsParams>,
) -> impl IntoResponse {
    match state
        .db
        .list_code_quality_thresholds_v20(repo_id, params.enabled_only)
        .await
    {
        Ok(thresholds) => {
            let resp: Vec<CodeQualityThresholdResponse> = thresholds
                .into_iter()
                .map(|t| CodeQualityThresholdResponse {
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

async fn check_violation(
    State(state): State<AppState>,
    Path(repo_id): Path<Uuid>,
    _auth: AuthUser,
    Json(req): Json<CheckViolationRequest>,
) -> impl IntoResponse {
    match state
        .db
        .check_code_quality_violation_v20(repo_id, &req.metric_name, req.metric_value)
        .await
    {
        Ok(is_violation) => (
            StatusCode::OK,
            Json(CheckViolationResponse {
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

async fn delete_threshold(
    State(state): State<AppState>,
    Path((repo_id, metric_name)): Path<(Uuid, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    match state
        .db
        .delete_code_quality_threshold_v20(repo_id, &metric_name)
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

async fn get_enforcement_report(
    State(state): State<AppState>,
    Path(repo_id): Path<Uuid>,
    _auth: AuthUser,
) -> impl IntoResponse {
    match state.db.get_code_quality_enforcement_report_v20(repo_id).await {
        Ok((total, violating, compliance_rate)) => (
            StatusCode::OK,
            Json(EnforcementReportResponse {
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

pub fn code_quality_rules_routes() -> Router<AppState> {
    let mut router = Router::new();

    let versions = [5, 6, 7, 9, 13, 14, 15, 16, 17, 19, 20];
    for v in versions {
        let suffix = format!("/v{v}");
        router = router
            .route(
                &format!("/api/v1/repos/{{repo_id}}/quality/metrics{suffix}"),
                post(record_metric).get(list_metrics),
            )
            .route(
                &format!("/api/v1/repos/{{repo_id}}/quality/score{suffix}"),
                get(get_quality_score),
            )
            .route(
                &format!("/api/v1/repos/{{repo_id}}/quality/thresholds{suffix}"),
                post(create_threshold).get(list_thresholds),
            )
            .route(
                &format!("/api/v1/repos/{{repo_id}}/quality/check-violation{suffix}"),
                post(check_violation),
            )
            .route(
                &format!("/api/v1/repos/{{repo_id}}/quality/thresholds/{{metric_name}}{suffix}"),
                delete(delete_threshold),
            )
            .route(
                &format!("/api/v1/repos/{{repo_id}}/quality/enforcement-report{suffix}"),
                get(get_enforcement_report),
            );
    }

    router
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_response_serializes() {
        let resp = CodeQualityMetricResponse {
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
    fn test_threshold_response_serializes() {
        let resp = CodeQualityThresholdResponse {
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
    fn test_violation_check_response() {
        let resp = CheckViolationResponse {
            is_violation: false,
            metric_name: "code_coverage".into(),
            metric_value: 85.0,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("is_violation"));
        assert!(json.contains("false"));
    }

    #[test]
    fn test_quality_score_response() {
        let resp = QualityScoreResponse {
            repo_id: "00000000-0000-0000-0000-000000000001".into(),
            score: 92.5,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("score"));
        assert!(json.contains("92.5"));
    }

    #[test]
    fn test_enforcement_report_response() {
        let resp = EnforcementReportResponse {
            total_thresholds: 10,
            violating_thresholds: 2,
            compliance_rate: 80.0,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("compliance_rate"));
        assert!(json.contains("80"));
    }

    #[test]
    fn test_record_metric_request() {
        let req: RecordMetricRequest = serde_json::from_str(
            r#"{"file_path": "src/lib.rs", "metric_name": "lines_of_code", "metric_value": 500.0}"#,
        )
        .unwrap();
        assert_eq!(req.file_path, "src/lib.rs");
        assert_eq!(req.metric_name, "lines_of_code");
        assert_eq!(req.metric_value, 500.0);
    }

    #[test]
    fn test_create_threshold_request() {
        let req: CreateThresholdRequest = serde_json::from_str(
            r#"{"metric_name": "test_coverage", "threshold_value": 80.0, "enabled": true}"#,
        )
        .unwrap();
        assert_eq!(req.metric_name, "test_coverage");
        assert_eq!(req.threshold_value, 80.0);
        assert_eq!(req.enabled, Some(true));
    }
}
