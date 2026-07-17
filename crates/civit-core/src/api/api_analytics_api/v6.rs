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
pub struct ApiAnalyticV6Response {
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
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiAnalyticV6Request {
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
}

fn default_region() -> String {
    "us-east-1".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsV6Query {
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
pub struct RequestCorrelationResponse {
    pub request_id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: i32,
    pub correlations: Vec<CorrelationEntry>,
    pub trace_chain: Vec<TraceEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationEntry {
    pub id: Uuid,
    pub request_id: Uuid,
    pub parent_request_id: Option<Uuid>,
    pub correlation_type: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEntry {
    pub trace_id: String,
    pub span_id: String,
    pub endpoint: String,
    pub duration_ms: i32,
    pub status_code: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceOptimizationV6 {
    pub endpoint: String,
    pub method: String,
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub cache_hit_rate: f64,
    pub optimization_suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAnalysisV6 {
    pub total_requests: i64,
    pub total_request_bytes: i64,
    pub total_response_bytes: i64,
    pub estimated_cost_cents: i64,
    pub cost_by_region: Vec<RegionCostV6>,
    pub cost_by_user_agent: Vec<UserAgentCostV6>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionCostV6 {
    pub region: String,
    pub requests: i64,
    pub cost_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAgentCostV6 {
    pub user_agent: String,
    pub requests: i64,
    pub cost_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityPlanResponse {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub current_rps: i32,
    pub projected_rps: i32,
    pub capacity_limit: i32,
    pub utilization_percent: f64,
    pub last_calculated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCapacityPlanRequest {
    pub endpoint: String,
    pub method: String,
    pub current_rps: i32,
    pub projected_rps: i32,
    pub capacity_limit: i32,
    pub utilization_percent: f64,
}

pub async fn list_api_analytics_v6(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Query(query): axum::extract::Query<AnalyticsV6Query>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.list_api_analytics_v6(query.limit, query.offset).await {
        Ok(analytics) => {
            let response: Vec<ApiAnalyticV6Response> = analytics.iter().map(|a| ApiAnalyticV6Response {
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

pub async fn create_api_analytic_v6(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateApiAnalyticV6Request>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.create_api_analytic_v6(
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
    ).await {
        Ok(analytic) => {
            let response = ApiAnalyticV6Response {
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

pub async fn get_request_correlation(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(request_id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.get_correlations_for_request(request_id).await {
        Ok(correlations) => {
            let correlation_entries: Vec<CorrelationEntry> = correlations.iter().map(|c| CorrelationEntry {
                id: c.id,
                request_id: c.request_id,
                parent_request_id: c.parent_request_id,
                correlation_type: c.correlation_type.clone(),
                trace_id: c.trace_id.clone(),
                span_id: c.span_id.clone(),
                created_at: c.created_at,
            }).collect();

            let trace_chain: Vec<TraceEntry> = correlations.iter()
                .filter_map(|c| {
                    let trace_id = c.trace_id.clone()?;
                    let span_id = c.span_id.clone()?;
                    Some(TraceEntry {
                        trace_id,
                        span_id,
                        endpoint: String::new(),
                        duration_ms: 0,
                        status_code: 200,
                    })
                })
                .collect();

            let response = RequestCorrelationResponse {
                request_id,
                endpoint: String::new(),
                method: String::new(),
                status_code: 200,
                response_time_ms: 0,
                correlations: correlation_entries,
                trace_chain,
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

pub async fn get_performance_optimization_v6(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.get_performance_optimization_v6().await {
        Ok(optimizations) => {
            let response: Vec<PerformanceOptimizationV6> = optimizations.iter().map(|o| PerformanceOptimizationV6 {
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

pub async fn get_cost_analysis_v6(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.get_cost_analysis_v6().await {
        Ok(analysis) => {
            let response = CostAnalysisV6 {
                total_requests: analysis.0,
                total_request_bytes: analysis.1,
                total_response_bytes: analysis.2,
                estimated_cost_cents: analysis.3,
                cost_by_region: analysis.4.into_iter().map(|(region, requests, cost)| RegionCostV6 {
                    region,
                    requests,
                    cost_cents: cost,
                }).collect(),
                cost_by_user_agent: analysis.5.into_iter().map(|(ua, requests, cost)| UserAgentCostV6 {
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

pub async fn list_capacity_plans(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.get_capacity_plans().await {
        Ok(plans) => {
            let response: Vec<CapacityPlanResponse> = plans.iter().map(|p| CapacityPlanResponse {
                id: p.id,
                endpoint: p.endpoint.clone(),
                method: p.method.clone(),
                current_rps: p.current_rps,
                projected_rps: p.projected_rps,
                capacity_limit: p.capacity_limit,
                utilization_percent: p.utilization_percent,
                last_calculated_at: p.last_calculated_at,
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

pub async fn create_capacity_plan(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateCapacityPlanRequest>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.upsert_capacity_plan(
        &req.endpoint,
        &req.method,
        req.current_rps,
        req.projected_rps,
        req.capacity_limit,
        req.utilization_percent,
    ).await {
        Ok(plan) => {
            let response = CapacityPlanResponse {
                id: plan.id,
                endpoint: plan.endpoint,
                method: plan.method,
                current_rps: plan.current_rps,
                projected_rps: plan.projected_rps,
                capacity_limit: plan.capacity_limit,
                utilization_percent: plan.utilization_percent,
                last_calculated_at: plan.last_calculated_at,
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

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v6/analytics", get(list_api_analytics_v6).post(create_api_analytic_v6))
        .route("/api/v6/analytics/correlation/{request_id}", get(get_request_correlation))
        .route("/api/v6/analytics/performance", get(get_performance_optimization_v6))
        .route("/api/v6/analytics/cost", get(get_cost_analysis_v6))
        .route("/api/v6/analytics/capacity", get(list_capacity_plans).post(create_capacity_plan))
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_analytics_v6_response_serialization() {
        let response = ApiAnalyticV6Response {
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
            request_id: Some(Uuid::new_v4()),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("request_id"));
        assert!(json.contains("us-east-1"));
        assert!(json.contains("Mozilla/5.0"));
    }

    #[test]
    fn test_capacity_plan_response_serialization() {
        let response = CapacityPlanResponse {
            id: Uuid::nil(),
            endpoint: "/api/v1/repos".into(),
            method: "GET".into(),
            current_rps: 1000,
            projected_rps: 1500,
            capacity_limit: 2000,
            utilization_percent: 50.0,
            last_calculated_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("capacity_limit"));
        assert!(json.contains("utilization_percent"));
    }

    #[test]
    fn test_cost_analysis_v6_serialization() {
        let response = CostAnalysisV6 {
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
