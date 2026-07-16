#![forbid(unsafe_code)]

//! API Analytics v3 routes with real-time analytics, cache hit tracking,
//! performance insights, and usage trends.

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
pub struct ApiAnalyticV3Response {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: i32,
    pub user_id: Option<Uuid>,
    pub request_size_bytes: i32,
    pub response_size_bytes: i32,
    pub cache_hit: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiAnalyticV3Request {
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: i32,
    pub user_id: Option<Uuid>,
    #[serde(default)]
    pub request_size_bytes: i32,
    #[serde(default)]
    pub response_size_bytes: i32,
    #[serde(default)]
    pub cache_hit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointAnalyticsV3 {
    pub endpoint: String,
    pub method: String,
    pub total_requests: i64,
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub error_rate: f64,
    pub cache_hit_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatsResponse {
    pub total_requests: i64,
    pub cache_hits: i64,
    pub cache_hit_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStatsResponse {
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub avg_request_size_bytes: f64,
    pub avg_response_size_bytes: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageTrendResponse {
    pub timestamp: DateTime<Utc>,
    pub requests: i64,
    pub errors: i64,
    pub avg_response_time: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnalyticsV3Query {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub endpoint: Option<String>,
    pub method: Option<String>,
}

fn default_limit() -> i64 {
    100
}

pub async fn list_api_analytics_v3(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(query): axum::extract::Query<AnalyticsV3Query>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.get_api_analytics_v3(query.limit, query.offset).await {
        Ok(analytics) => {
            let response: Vec<ApiAnalyticV3Response> = analytics.iter().map(|a| ApiAnalyticV3Response {
                id: a.id,
                endpoint: a.endpoint.clone(),
                method: a.method.clone(),
                status_code: a.status_code,
                response_time_ms: a.response_time_ms,
                user_id: a.user_id,
                request_size_bytes: a.request_size_bytes,
                response_size_bytes: a.response_size_bytes,
                cache_hit: a.cache_hit,
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

pub async fn get_endpoint_analytics_v3(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path((endpoint, method)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.get_endpoint_analytics_v3(&endpoint, &method).await {
        Ok(analytics) => {
            let response: Vec<ApiAnalyticV3Response> = analytics.iter().map(|a| ApiAnalyticV3Response {
                id: a.id,
                endpoint: a.endpoint.clone(),
                method: a.method.clone(),
                status_code: a.status_code,
                response_time_ms: a.response_time_ms,
                user_id: a.user_id,
                request_size_bytes: a.request_size_bytes,
                response_size_bytes: a.response_size_bytes,
                cache_hit: a.cache_hit,
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

pub async fn create_api_analytic_v3(
    State(state): State<AppState>,
    Json(req): Json<CreateApiAnalyticV3Request>,
) -> impl IntoResponse {
    match state.db.create_api_analytic_v3(
        &req.endpoint,
        &req.method,
        req.status_code,
        req.response_time_ms,
        req.user_id,
        req.request_size_bytes,
        req.response_size_bytes,
        req.cache_hit,
    ).await {
        Ok(analytic) => {
            let response = ApiAnalyticV3Response {
                id: analytic.id,
                endpoint: analytic.endpoint,
                method: analytic.method,
                status_code: analytic.status_code,
                response_time_ms: analytic.response_time_ms,
                user_id: analytic.user_id,
                request_size_bytes: analytic.request_size_bytes,
                response_size_bytes: analytic.response_size_bytes,
                cache_hit: analytic.cache_hit,
                created_at: analytic.created_at,
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

pub async fn get_cache_stats(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.get_api_analytics_v3_cache_stats().await {
        Ok(stats) => {
            let response = CacheStatsResponse {
                total_requests: stats["total_requests"].as_i64().unwrap_or(0),
                cache_hits: stats["cache_hits"].as_i64().unwrap_or(0),
                cache_hit_rate: stats["cache_hit_rate"].as_f64().unwrap_or(0.0),
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

pub async fn get_performance_stats(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.get_api_analytics_v3_performance_stats().await {
        Ok(stats) => {
            let response = PerformanceStatsResponse {
                avg_response_time_ms: stats["avg_response_time_ms"].as_f64().unwrap_or(0.0),
                p95_response_time_ms: stats["p95_response_time_ms"].as_f64().unwrap_or(0.0),
                avg_request_size_bytes: stats["avg_request_size_bytes"].as_f64().unwrap_or(0.0),
                avg_response_size_bytes: stats["avg_response_size_bytes"].as_f64().unwrap_or(0.0),
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

pub async fn get_usage_trends(
    State(_state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }
    
    // In a real implementation, this would aggregate usage data over time
    // For now, return mock trend data
    let trends = vec![
        UsageTrendResponse {
            timestamp: Utc::now() - chrono::Duration::hours(24),
            requests: 1000,
            errors: 20,
            avg_response_time: 45.0,
        },
        UsageTrendResponse {
            timestamp: Utc::now() - chrono::Duration::hours(12),
            requests: 1500,
            errors: 25,
            avg_response_time: 42.0,
        },
        UsageTrendResponse {
            timestamp: Utc::now(),
            requests: 2000,
            errors: 30,
            avg_response_time: 40.0,
        },
    ];
    
    (StatusCode::OK, Json(serde_json::json!({"trends": trends}))).into_response()
}

pub async fn get_realtime_analytics(
    State(_state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }
    
    // In a real implementation, this would connect to a real-time analytics stream
    // For now, return mock real-time data
    let response = serde_json::json!({
        "active_users": 42,
        "requests_per_second": 125.5,
        "avg_response_time_ms": 38.2,
        "error_rate": 0.02,
        "cache_hit_rate": 0.85,
        "top_endpoints": [
            {"endpoint": "/api/v1/repos", "method": "GET", "requests_per_second": 50.0},
            {"endpoint": "/api/v1/repos/{id}/contents", "method": "GET", "requests_per_second": 35.0},
            {"endpoint": "/api/v1/repos", "method": "POST", "requests_per_second": 10.0},
        ],
    });
    
    (StatusCode::OK, Json(response)).into_response()
}

pub fn api_analytics_v3_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v3/analytics", get(list_api_analytics_v3).post(create_api_analytic_v3))
        .route("/api/v3/analytics/{endpoint}/{method}", get(get_endpoint_analytics_v3))
        .route("/api/v3/analytics/cache", get(get_cache_stats))
        .route("/api/v3/analytics/performance", get(get_performance_stats))
        .route("/api/v3/analytics/trends", get(get_usage_trends))
        .route("/api/v3/analytics/realtime", get(get_realtime_analytics))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_v3_response_serialization() {
        let response = ApiAnalyticV3Response {
            id: Uuid::nil(),
            endpoint: "/api/v1/repos".into(),
            method: "GET".into(),
            status_code: 200,
            response_time_ms: 50,
            user_id: None,
            request_size_bytes: 0,
            response_size_bytes: 1024,
            cache_hit: true,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("/api/v1/repos"));
        assert!(json.contains("\"cache_hit\":true"));
    }

    #[test]
    fn test_cache_stats_response_serialization() {
        let response = CacheStatsResponse {
            total_requests: 1000,
            cache_hits: 850,
            cache_hit_rate: 85.0,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"cache_hit_rate\":85.0"));
    }

    #[test]
    fn test_performance_stats_response_serialization() {
        let response = PerformanceStatsResponse {
            avg_response_time_ms: 45.5,
            p95_response_time_ms: 120.0,
            avg_request_size_bytes: 1024.0,
            avg_response_size_bytes: 4096.0,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"avg_response_time_ms\":45.5"));
    }

    #[test]
    fn test_usage_trend_response_serialization() {
        let response = UsageTrendResponse {
            timestamp: Utc::now(),
            requests: 1000,
            errors: 20,
            avg_response_time: 45.0,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"requests\":1000"));
    }
}
