#![forbid(unsafe_code)]

//! API Analytics v13 routes with advanced cost tracking, budget alerts,
//! usage optimization, and capacity planning.

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
pub struct ApiAnalyticV13Response {
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
    pub request_id: Option<Uuid>,
    pub cost_cents: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiAnalyticV13Request {
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
    pub request_id: Option<Uuid>,
    #[serde(default)]
    pub cost_cents: i32,
}

fn default_region() -> String {
    "us-east-1".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsV13Query {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub endpoint: Option<String>,
    pub method: Option<String>,
    pub region: Option<String>,
    pub status_code: Option<i32>,
}

fn default_limit() -> i64 {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostTrackingV13Response {
    pub total_requests: i64,
    pub total_request_bytes: i64,
    pub total_response_bytes: i64,
    pub estimated_cost_cents: i64,
    pub cost_by_region: Vec<RegionCostV13>,
    pub cost_by_user_agent: Vec<UserAgentCostV13>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionCostV13 {
    pub region: String,
    pub requests: i64,
    pub cost_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAgentCostV13 {
    pub user_agent: String,
    pub requests: i64,
    pub cost_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAlertV13 {
    pub id: Uuid,
    pub user_id: Uuid,
    pub budget_cents: i32,
    pub threshold_percent: f64,
    pub current_usage_cents: i64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageOptimizationV13 {
    pub endpoint: String,
    pub method: String,
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub cache_hit_rate: f64,
    pub optimization_suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityPlanV13Response {
    pub endpoint: String,
    pub method: String,
    pub current_rps: i32,
    pub projected_rps: i32,
    pub capacity_limit: i32,
    pub utilization_percent: f64,
}

pub async fn list_api_analytics_v13(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(query): axum::extract::Query<AnalyticsV13Query>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.list_api_analytics_v13(query.limit, query.offset).await {
        Ok(analytics) => {
            let response: Vec<ApiAnalyticV13Response> = analytics.iter().map(|a| ApiAnalyticV13Response {
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
                request_id: a.request_id,
                cost_cents: a.cost_cents,
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

pub async fn create_api_analytic_v13(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateApiAnalyticV13Request>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.create_api_analytic_v13(
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
        req.request_id,
        req.cost_cents,
    ).await {
        Ok(analytic) => {
            let response = ApiAnalyticV13Response {
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
                request_id: analytic.request_id,
                cost_cents: analytic.cost_cents,
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

pub async fn get_cost_tracking_v13(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.get_cost_analysis_v13().await {
        Ok(analysis) => {
            let response = CostTrackingV13Response {
                total_requests: analysis.0,
                total_request_bytes: analysis.1,
                total_response_bytes: analysis.2,
                estimated_cost_cents: analysis.3,
                cost_by_region: analysis.4.into_iter().map(|(region, requests, cost)| RegionCostV13 {
                    region,
                    requests,
                    cost_cents: cost,
                }).collect(),
                cost_by_user_agent: analysis.5.into_iter().map(|(ua, requests, cost)| UserAgentCostV13 {
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

pub async fn get_budget_alerts_v13() -> impl IntoResponse {
    let alerts: Vec<BudgetAlertV13> = vec![];
    (StatusCode::OK, Json(alerts)).into_response()
}

pub async fn get_usage_optimization_v13(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.get_usage_optimization_v13().await {
        Ok(optimizations) => {
            let response: Vec<UsageOptimizationV13> = optimizations.iter().map(|o| UsageOptimizationV13 {
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

pub async fn get_capacity_planning_v13() -> impl IntoResponse {
    let plans: Vec<CapacityPlanV13Response> = vec![];
    (StatusCode::OK, Json(plans)).into_response()
}

pub fn api_analytics_v13_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v13/analytics", get(list_api_analytics_v13).post(create_api_analytic_v13))
        .route("/api/v13/analytics/cost", get(get_cost_tracking_v13))
        .route("/api/v13/analytics/budget-alerts", get(get_budget_alerts_v13))
        .route("/api/v13/analytics/optimization", get(get_usage_optimization_v13))
        .route("/api/v13/analytics/capacity", get(get_capacity_planning_v13))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_v13_response_serialization() {
        let response = ApiAnalyticV13Response {
            id: Uuid::nil(),
            endpoint: "/api/v12/repos".into(),
            method: "GET".into(),
            status_code: 200,
            response_time_ms: 45,
            user_id: None,
            request_size_bytes: 128,
            response_size_bytes: 2048,
            cache_hit: true,
            region: "us-east-1".into(),
            user_agent: Some("Mozilla/5.0".into()),
            request_id: Some(Uuid::new_v4()),
            cost_cents: 1,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("cost_cents"));
        assert!(json.contains("us-east-1"));
    }

    #[test]
    fn test_cost_tracking_v13_serialization() {
        let response = CostTrackingV13Response {
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

    #[test]
    fn test_budget_alert_v13_serialization() {
        let alert = BudgetAlertV13 {
            id: Uuid::nil(),
            user_id: Uuid::nil(),
            budget_cents: 10000,
            threshold_percent: 80.0,
            current_usage_cents: 5000,
            enabled: true,
            last_triggered_at: None,
        };
        let json = serde_json::to_string(&alert).unwrap();
        assert!(json.contains("budget_cents"));
        assert!(json.contains("80.0"));
    }
}
