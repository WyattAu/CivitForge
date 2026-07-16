#![forbid(unsafe_code)]

//! API analytics dashboard routes for usage analytics, performance metrics,
//! error tracking, and user activity analysis.

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_admin};
use axum::{
    Json,
    Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiAnalyticV2Response {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: i32,
    pub user_id: Option<Uuid>,
    pub request_size_bytes: i32,
    pub response_size_bytes: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiAnalyticsV2Summary {
    pub total_requests: i64,
    pub avg_response_time_ms: f64,
    pub error_rate: f64,
    pub unique_users: i64,
    pub total_request_bytes: i64,
    pub total_response_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBreakdownItem {
    pub status_code: i32,
    pub count: i64,
    pub avg_response_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsDashboard {
    pub summary: ApiAnalyticsV2Summary,
    pub error_breakdown: Vec<ErrorBreakdownItem>,
    pub top_endpoints: Vec<EndpointPerformance>,
    pub user_activity: Vec<UserActivitySummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointPerformance {
    pub endpoint: String,
    pub method: String,
    pub total_requests: i64,
    pub avg_response_time_ms: f64,
    pub error_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivitySummary {
    pub user_id: Option<Uuid>,
    pub total_requests: i64,
    pub avg_response_time_ms: f64,
    pub error_rate: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnalyticsQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub endpoint: Option<String>,
    #[serde(default = "default_hours")]
    pub hours: i64,
}

fn default_limit() -> i64 {
    100
}

fn default_hours() -> i64 {
    24
}

pub async fn list_api_analytics_v2(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(query): axum::extract::Query<AnalyticsQuery>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let result = if let Some(endpoint) = &query.endpoint {
        state.db.get_api_analytics_v2_by_endpoint(endpoint, query.limit).await
    } else {
        state.db.get_api_analytics_v2(query.limit, query.offset).await
    };

    match result {
        Ok(analytics) => {
            let response: Vec<ApiAnalyticV2Response> = analytics.iter().map(|a| ApiAnalyticV2Response {
                id: a.id,
                endpoint: a.endpoint.clone(),
                method: a.method.clone(),
                status_code: a.status_code,
                response_time_ms: a.response_time_ms,
                user_id: a.user_id,
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

pub async fn get_api_analytics_v2_summary(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(query): axum::extract::Query<AnalyticsQuery>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let since = Utc::now() - chrono::Duration::hours(query.hours);

    match state.db.get_api_analytics_v2_summary(since).await {
        Ok(summary) => {
            (StatusCode::OK, Json(summary)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_api_analytics_v2_errors(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(query): axum::extract::Query<AnalyticsQuery>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let since = Utc::now() - chrono::Duration::hours(query.hours);

    match state.db.get_api_analytics_v2_error_breakdown(since).await {
        Ok(errors) => {
            let response: Vec<ErrorBreakdownItem> = errors.iter().filter_map(|e| {
                serde_json::from_value(e.clone()).ok()
            }).collect();
            (StatusCode::OK, Json(serde_json::json!({"errors": response}))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_analytics_dashboard(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(query): axum::extract::Query<AnalyticsQuery>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let since = Utc::now() - chrono::Duration::hours(query.hours);

    let summary_result = state.db.get_api_analytics_v2_summary(since).await;
    let errors_result = state.db.get_api_analytics_v2_error_breakdown(since).await;

    let summary = match summary_result {
        Ok(s) => serde_json::from_value(s).unwrap_or(ApiAnalyticsV2Summary {
            total_requests: 0,
            avg_response_time_ms: 0.0,
            error_rate: 0.0,
            unique_users: 0,
            total_request_bytes: 0,
            total_response_bytes: 0,
        }),
        Err(_) => ApiAnalyticsV2Summary {
            total_requests: 0,
            avg_response_time_ms: 0.0,
            error_rate: 0.0,
            unique_users: 0,
            total_request_bytes: 0,
            total_response_bytes: 0,
        },
    };

    let error_breakdown = match errors_result {
        Ok(e) => e.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect(),
        Err(_) => vec![],
    };

    let dashboard = AnalyticsDashboard {
        summary,
        error_breakdown,
        top_endpoints: vec![],
        user_activity: vec![],
    };

    (StatusCode::OK, Json(dashboard)).into_response()
}

pub fn api_analytics_v2_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/admin/analytics/v2", get(list_api_analytics_v2))
        .route("/api/v1/admin/analytics/v2/summary", get(get_api_analytics_v2_summary))
        .route("/api/v1/admin/analytics/v2/errors", get(get_api_analytics_v2_errors))
        .route("/api/v1/admin/analytics/v2/dashboard", get(get_analytics_dashboard))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_v2_response_serialization() {
        let response = ApiAnalyticV2Response {
            id: Uuid::nil(),
            endpoint: "/api/v1/repos".into(),
            method: "GET".into(),
            status_code: 200,
            response_time_ms: 50,
            user_id: None,
            request_size_bytes: 0,
            response_size_bytes: 1024,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("/api/v1/repos"));
        assert!(json.contains("\"status_code\":200"));
    }

    #[test]
    fn test_analytics_summary_serialization() {
        let summary = ApiAnalyticsV2Summary {
            total_requests: 1000,
            avg_response_time_ms: 50.5,
            error_rate: 0.02,
            unique_users: 100,
            total_request_bytes: 1024000,
            total_response_bytes: 5120000,
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"total_requests\":1000"));
        assert!(json.contains("\"avg_response_time_ms\":50.5"));
    }

    #[test]
    fn test_error_breakdown_serialization() {
        let item = ErrorBreakdownItem {
            status_code: 500,
            count: 10,
            avg_response_time_ms: 200.0,
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"status_code\":500"));
        assert!(json.contains("\"count\":10"));
    }

    #[test]
    fn test_analytics_query_defaults() {
        let query = AnalyticsQuery {
            limit: 100,
            offset: 0,
            endpoint: None,
            hours: 24,
        };
        assert_eq!(query.limit, 100);
        assert_eq!(query.hours, 24);
    }
}
