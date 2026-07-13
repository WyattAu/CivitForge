#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::{CoreError, ErrorResponse};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct PerformanceMetricResponse {
    pub id: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub labels: serde_json::Value,
    pub recorded_at: String,
}

#[derive(Debug, Deserialize)]
pub struct RecordMetricRequest {
    pub metric_name: String,
    pub metric_value: f64,
    #[serde(default)]
    pub labels: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct QueryMetricsParams {
    pub metric_name: String,
    pub since: Option<String>,
    pub until: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

#[derive(Debug, Deserialize)]
pub struct MetricSummaryParams {
    pub metric_name: String,
    pub since: Option<String>,
}

fn default_limit() -> i64 {
    1000
}

async fn resolve_repo_id(
    state: &AppState,
    owner: &str,
    name: &str,
) -> Result<Uuid, (StatusCode, Json<ErrorResponse>)> {
    let owner_uuid = if let Ok(id) = Uuid::parse_str(owner) {
        id
    } else if let Ok(user) = state.db.get_user_by_username(owner).await {
        user.id
    } else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("user not found".into()).error_response()),
        ));
    };

    match state.db.get_repo_by_owner_name(owner_uuid, name).await {
        Ok(repo) => Ok(repo.id),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )),
    }
}

pub async fn record_metric(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Json(req): Json<RecordMetricRequest>,
) -> impl IntoResponse {
    let _repo_id = match resolve_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp.into_response(),
    };

    match state
        .db
        .record_performance_metric(&req.metric_name, req.metric_value, &req.labels)
        .await
    {
        Ok(metric) => (
            StatusCode::CREATED,
            Json(PerformanceMetricResponse {
                id: metric.id.to_string(),
                metric_name: metric.metric_name,
                metric_value: metric.metric_value,
                labels: metric.labels,
                recorded_at: metric.recorded_at.to_rfc3339(),
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

pub async fn query_metrics(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Query(params): Query<QueryMetricsParams>,
) -> impl IntoResponse {
    let _repo_id = match resolve_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp.into_response(),
    };

    let since = params
        .since
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| Utc::now() - chrono::Duration::hours(24));

    let until = params
        .until
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    match state
        .db
        .query_performance_metrics(&params.metric_name, since, until, params.limit)
        .await
    {
        Ok(metrics) => {
            let resp: Vec<PerformanceMetricResponse> = metrics
                .into_iter()
                .map(|m| PerformanceMetricResponse {
                    id: m.id.to_string(),
                    metric_name: m.metric_name,
                    metric_value: m.metric_value,
                    labels: m.labels,
                    recorded_at: m.recorded_at.to_rfc3339(),
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

pub async fn get_metric_summary(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Query(params): Query<MetricSummaryParams>,
) -> impl IntoResponse {
    let _repo_id = match resolve_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp.into_response(),
    };

    let since = params
        .since
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| Utc::now() - chrono::Duration::hours(24));

    match state
        .db
        .get_performance_metric_summary(&params.metric_name, since)
        .await
    {
        Ok(summary) => (StatusCode::OK, Json(summary)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub fn performance_metric_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/metrics",
            post(record_metric).get(query_metrics),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/metrics/summary",
            get(get_metric_summary),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_response_serializes() {
        let resp = PerformanceMetricResponse {
            id: "00000000-0000-0000-0000-000000000001".into(),
            metric_name: "http_request_duration_ms".into(),
            metric_value: 42.5,
            labels: serde_json::json!({"method": "GET", "status": 200}),
            recorded_at: "2024-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("http_request_duration_ms"));
        assert!(json.contains("42.5"));
    }

    #[test]
    fn test_record_metric_request() {
        let req: RecordMetricRequest = serde_json::from_str(
            r#"{"metric_name": "cpu_usage", "metric_value": 75.3, "labels": {"host": "web-1"}}"#,
        )
        .unwrap();
        assert_eq!(req.metric_name, "cpu_usage");
        assert_eq!(req.metric_value, 75.3);
    }

    #[test]
    fn test_query_metrics_params() {
        let params: QueryMetricsParams = serde_json::from_str(
            r#"{"metric_name": "http_requests", "since": "2024-01-01T00:00:00Z", "limit": 100}"#,
        )
        .unwrap();
        assert_eq!(params.metric_name, "http_requests");
        assert_eq!(params.limit, 100);
    }
}
