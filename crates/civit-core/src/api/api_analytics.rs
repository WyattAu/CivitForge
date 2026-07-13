#![forbid(unsafe_code)]

//! API analytics routes for dashboard, endpoint statistics, user usage, and periodic summaries.

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_admin};
use axum::{
    Json,
    Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiAnalyticResponse {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: i32,
    pub user_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_size_bytes: i32,
    pub response_size_bytes: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointStatistics {
    pub endpoint: String,
    pub method: String,
    pub total_requests: i64,
    pub avg_response_time_ms: f64,
    pub error_rate: f64,
    pub p95_response_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUsageStatistics {
    pub user_id: Uuid,
    pub total_requests: i64,
    pub avg_response_time_ms: f64,
    pub error_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiUsageSummaryResponse {
    pub id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_requests: i32,
    pub total_errors: i32,
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub unique_users: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnalyticsQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    100
}

pub async fn list_api_analytics(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(query): axum::extract::Query<AnalyticsQuery>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.get_api_analytics(query.limit, query.offset).await {
        Ok(analytics) => {
            let response: Vec<ApiAnalyticResponse> = analytics.iter().map(|a| ApiAnalyticResponse {
                id: a.id,
                endpoint: a.endpoint.clone(),
                method: a.method.clone(),
                status_code: a.status_code,
                response_time_ms: a.response_time_ms,
                user_id: a.user_id,
                ip_address: a.ip_address.clone(),
                user_agent: a.user_agent.clone(),
                request_size_bytes: a.request_size_bytes,
                response_size_bytes: a.response_size_bytes,
                created_at: a.created_at,
            }).collect();
            (StatusCode::OK, Json(serde_json::json!({"analytics": response, "total": response.len()}))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_endpoint_statistics(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.get_endpoint_statistics().await {
        Ok(stats) => {
            let response: Vec<EndpointStatistics> = stats.iter().filter_map(|s| {
                serde_json::from_value(s.clone()).ok()
            }).collect();
            (StatusCode::OK, Json(serde_json::json!({"endpoints": response}))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_user_usage_statistics(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.get_user_usage_statistics().await {
        Ok(stats) => {
            let response: Vec<UserUsageStatistics> = stats.iter().filter_map(|s| {
                serde_json::from_value(s.clone()).ok()
            }).collect();
            (StatusCode::OK, Json(serde_json::json!({"users": response}))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_api_usage_summary(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let period_start = Utc::now() - chrono::Duration::hours(24);
    let period_end = Utc::now();

    match state.db.get_api_usage_summary(period_start, period_end).await {
        Ok(summary) => {
            let response = ApiUsageSummaryResponse {
                id: summary.id,
                period_start: summary.period_start,
                period_end: summary.period_end,
                total_requests: summary.total_requests,
                total_errors: summary.total_errors,
                avg_response_time_ms: summary.avg_response_time_ms,
                p95_response_time_ms: summary.p95_response_time_ms,
                unique_users: summary.unique_users,
                created_at: summary.created_at,
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

pub fn api_analytics_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/admin/analytics", get(list_api_analytics))
        .route("/api/v1/admin/analytics/endpoints", get(get_endpoint_statistics))
        .route("/api/v1/admin/analytics/users", get(get_user_usage_statistics))
        .route("/api/v1/admin/analytics/summary", get(get_api_usage_summary))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_response_serialization() {
        let response = ApiAnalyticResponse {
            id: Uuid::nil(),
            endpoint: "/api/v1/repos".into(),
            method: "GET".into(),
            status_code: 200,
            response_time_ms: 50,
            user_id: None,
            ip_address: Some("127.0.0.1".into()),
            user_agent: Some("test-agent".into()),
            request_size_bytes: 0,
            response_size_bytes: 1024,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("/api/v1/repos"));
        assert!(json.contains("\"status_code\":200"));
    }

    #[test]
    fn test_endpoint_statistics_serialization() {
        let stats = EndpointStatistics {
            endpoint: "/api/v1/repos".into(),
            method: "GET".into(),
            total_requests: 1000,
            avg_response_time_ms: 50.5,
            error_rate: 0.02,
            p95_response_time_ms: 120.0,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("/api/v1/repos"));
        assert!(json.contains("\"total_requests\":1000"));
    }

    #[test]
    fn test_user_usage_statistics_serialization() {
        let stats = UserUsageStatistics {
            user_id: Uuid::nil(),
            total_requests: 500,
            avg_response_time_ms: 30.0,
            error_rate: 0.01,
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"total_requests\":500"));
    }

    #[test]
    fn test_analytics_query_defaults() {
        let query = AnalyticsQuery {
            limit: 100,
            offset: 0,
        };
        assert_eq!(query.limit, 100);
        assert_eq!(query.offset, 0);
    }
}