#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
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
pub struct ApiAnalyticV5Response {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: i32,
    pub user_id: Option<Uuid>,
    pub request_size_bytes: i32,
    pub response_size_bytes: i32,
    pub cache_hit: bool,
    pub region: String,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiAnalyticV5Request {
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
    #[serde(default = "default_region")]
    pub region: String,
    pub user_agent: Option<String>,
}

fn default_region() -> String {
    "us-east-1".into()
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnalyticsV5Query {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub endpoint: Option<String>,
    pub method: Option<String>,
    pub region: Option<String>,
}

fn default_limit() -> i64 {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAgentAnalysis {
    pub user_agent: String,
    pub request_count: i64,
    pub avg_response_time_ms: f64,
    pub error_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicAnalytics {
    pub region: String,
    pub request_count: i64,
    pub avg_response_time_ms: f64,
    pub error_rate: f64,
    pub unique_users: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceOptimization {
    pub endpoint: String,
    pub method: String,
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub cache_hit_rate: f64,
    pub optimization_suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAnalysis {
    pub total_requests: i64,
    pub total_request_bytes: i64,
    pub total_response_bytes: i64,
    pub estimated_cost_cents: i64,
    pub cost_by_region: Vec<RegionCost>,
    pub cost_by_user_agent: Vec<UserAgentCost>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionCost {
    pub region: String,
    pub requests: i64,
    pub cost_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAgentCost {
    pub user_agent: String,
    pub requests: i64,
    pub cost_cents: i64,
}

pub async fn list_api_analytics_v5(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(query): axum::extract::Query<AnalyticsV5Query>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.list_api_analytics_v5(query.limit, query.offset).await {
        Ok(analytics) => {
            let response: Vec<ApiAnalyticV5Response> = analytics.iter().map(|a| ApiAnalyticV5Response {
                id: a.id,
                endpoint: a.endpoint.clone(),
                method: a.method.clone(),
                status_code: a.status_code,
                response_time_ms: a.response_time_ms,
                user_id: a.user_id,
                request_size_bytes: a.request_size_bytes,
                response_size_bytes: a.response_size_bytes,
                cache_hit: a.cache_hit,
                region: a.region.clone(),
                user_agent: a.user_agent.clone(),
                created_at: a.created_at,
            }).collect();
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create_api_analytic_v5(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateApiAnalyticV5Request>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.create_api_analytic_v5(
        &req.endpoint,
        &req.method,
        req.status_code,
        req.response_time_ms,
        req.user_id,
        req.request_size_bytes,
        req.response_size_bytes,
        req.cache_hit,
        &req.region,
        req.user_agent.as_deref(),
    ).await {
        Ok(analytic) => {
            let response = ApiAnalyticV5Response {
                id: analytic.id,
                endpoint: analytic.endpoint,
                method: analytic.method,
                status_code: analytic.status_code,
                response_time_ms: analytic.response_time_ms,
                user_id: analytic.user_id,
                request_size_bytes: analytic.request_size_bytes,
                response_size_bytes: analytic.response_size_bytes,
                cache_hit: analytic.cache_hit,
                region: analytic.region,
                user_agent: analytic.user_agent,
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

pub async fn get_user_agent_analysis(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.get_user_agent_analysis().await {
        Ok(analysis) => {
            let response: Vec<UserAgentAnalysis> = analysis.iter().map(|a| UserAgentAnalysis {
                user_agent: a.0.clone(),
                request_count: a.1,
                avg_response_time_ms: a.2,
                error_rate: a.3,
            }).collect();
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_geographic_analytics(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.get_geographic_analytics().await {
        Ok(analytics) => {
            let response: Vec<GeographicAnalytics> = analytics.iter().map(|a| GeographicAnalytics {
                region: a.0.clone(),
                request_count: a.1,
                avg_response_time_ms: a.2,
                error_rate: a.3,
                unique_users: a.4,
            }).collect();
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_performance_optimization(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.get_performance_optimization().await {
        Ok(optimizations) => {
            let response: Vec<PerformanceOptimization> = optimizations.iter().map(|o| PerformanceOptimization {
                endpoint: o.0.clone(),
                method: o.1.clone(),
                avg_response_time_ms: o.2,
                p95_response_time_ms: o.3,
                cache_hit_rate: o.4,
                optimization_suggestions: o.5.clone(),
            }).collect();
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_cost_analysis(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.get_cost_analysis().await {
        Ok(analysis) => {
            let response = CostAnalysis {
                total_requests: analysis.0,
                total_request_bytes: analysis.1,
                total_response_bytes: analysis.2,
                estimated_cost_cents: analysis.3,
                cost_by_region: analysis.4.into_iter().map(|(region, requests, cost)| RegionCost {
                    region,
                    requests,
                    cost_cents: cost,
                }).collect(),
                cost_by_user_agent: analysis.5.into_iter().map(|(ua, requests, cost)| UserAgentCost {
                    user_agent: ua,
                    requests,
                    cost_cents: cost,
                }).collect(),
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

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v5/analytics", get(list_api_analytics_v5).post(create_api_analytic_v5))
        .route("/api/v5/analytics/user-agents", get(get_user_agent_analysis))
        .route("/api/v5/analytics/geographic", get(get_geographic_analytics))
        .route("/api/v5/analytics/performance", get(get_performance_optimization))
        .route("/api/v5/analytics/cost", get(get_cost_analysis))
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_analytics_v5_response_serialization() {
        let response = ApiAnalyticV5Response {
            id: Uuid::nil(),
            endpoint: "/api/v1/repos".into(),
            method: "GET".into(),
            status_code: 200,
            response_time_ms: 45,
            user_id: None,
            request_size_bytes: 128,
            response_size_bytes: 2048,
            cache_hit: true,
            region: "us-east-1".into(),
            user_agent: Some("Mozilla/5.0".into()),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("user_agent"));
        assert!(json.contains("us-east-1"));
        assert!(json.contains("Mozilla/5.0"));
    }

    #[test]
    fn test_cost_analysis_serialization() {
        let response = CostAnalysis {
            total_requests: 100000,
            total_request_bytes: 10240000,
            total_response_bytes: 102400000,
            estimated_cost_cents: 500,
            cost_by_region: vec![],
            cost_by_user_agent: vec![],
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("estimated_cost_cents"));
        assert!(json.contains("100000"));
    }
}
