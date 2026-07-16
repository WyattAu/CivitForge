#![forbid(unsafe_code)]

//! Rate Limiting v21 routes with tier quota management, usage analytics,
//! dynamic adjustment, and capacity planning.

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
pub struct RateLimitTierQuotaV21Response {
    pub id: Uuid,
    pub tier: String,
    pub requests_per_second: i32,
    pub requests_per_day: i32,
    pub burst_size: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRateLimitTierQuotaV21Request {
    pub tier: String,
    #[serde(default = "default_rps")]
    pub requests_per_second: i32,
    #[serde(default = "default_rpd")]
    pub requests_per_day: i32,
    #[serde(default = "default_burst")]
    pub burst_size: i32,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_rps() -> i32 { 10 }
fn default_rpd() -> i32 { 10000 }
fn default_burst() -> i32 { 50 }
fn default_true() -> bool { true }

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateRateLimitTierQuotaV21Request {
    pub requests_per_second: Option<i32>,
    pub requests_per_day: Option<i32>,
    pub burst_size: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitUsageAnalyticsV21Response {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier: String,
    pub requests_used: i32,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicAdjustmentV21 {
    pub tier: String,
    pub current_rps: i32,
    pub recommended_rps: i32,
    pub reason: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityPlanV21 {
    pub tier: String,
    pub current_utilization: f64,
    pub projected_utilization_7d: f64,
    pub projected_utilization_30d: f64,
    pub recommended_capacity: i32,
    pub headroom_percent: f64,
}

pub async fn list_rate_limit_tier_quotas_v21(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.list_rate_limit_tier_quotas_v21().await {
        Ok(quotas) => {
            let response: Vec<RateLimitTierQuotaV21Response> = quotas.iter().map(|q| RateLimitTierQuotaV21Response {
                id: q.id,
                tier: q.tier.clone(),
                requests_per_second: q.requests_per_second,
                requests_per_day: q.requests_per_day,
                burst_size: q.burst_size,
                enabled: q.enabled,
                created_at: q.created_at,
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

pub async fn get_rate_limit_tier_quota_v21(
    State(state): State<AppState>,
    axum::extract::Path(tier): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state.db.get_rate_limit_tier_quota_v21_by_tier(&tier).await {
        Ok(Some(quota)) => {
            let response = RateLimitTierQuotaV21Response {
                id: quota.id,
                tier: quota.tier,
                requests_per_second: quota.requests_per_second,
                requests_per_day: quota.requests_per_day,
                burst_size: quota.burst_size,
                enabled: quota.enabled,
                created_at: quota.created_at,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Tier quota not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create_rate_limit_tier_quota_v21(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateRateLimitTierQuotaV21Request>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.create_rate_limit_tier_quota_v21(
        &req.tier,
        req.requests_per_second,
        req.requests_per_day,
        req.burst_size,
        req.enabled,
    ).await {
        Ok(quota) => {
            let response = RateLimitTierQuotaV21Response {
                id: quota.id,
                tier: quota.tier,
                requests_per_second: quota.requests_per_second,
                requests_per_day: quota.requests_per_day,
                burst_size: quota.burst_size,
                enabled: quota.enabled,
                created_at: quota.created_at,
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

pub async fn update_rate_limit_tier_quota_v21(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(tier): axum::extract::Path<String>,
    Json(req): Json<UpdateRateLimitTierQuotaV21Request>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.update_rate_limit_tier_quota_v21(
        &tier,
        req.requests_per_second,
        req.requests_per_day,
        req.burst_size,
        req.enabled,
    ).await {
        Ok(quota) => {
            let response = RateLimitTierQuotaV21Response {
                id: quota.id,
                tier: quota.tier,
                requests_per_second: quota.requests_per_second,
                requests_per_day: quota.requests_per_day,
                burst_size: quota.burst_size,
                enabled: quota.enabled,
                created_at: quota.created_at,
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

pub async fn get_usage_analytics_v21(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());
    match state.db.get_rate_limit_usage_analytics_v21_by_user(user_id).await {
        Ok(analytics) => {
            let response: Vec<RateLimitUsageAnalyticsV21Response> = analytics.iter().map(|a| RateLimitUsageAnalyticsV21Response {
                id: a.id,
                user_id: a.user_id,
                tier: a.tier.clone(),
                requests_used: a.requests_used,
                period_start: a.period_start,
                period_end: a.period_end,
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

pub async fn get_dynamic_adjustments_v21() -> impl IntoResponse {
    let adjustments: Vec<DynamicAdjustmentV21> = vec![];
    (StatusCode::OK, Json(adjustments)).into_response()
}

pub async fn get_capacity_planning_v21() -> impl IntoResponse {
    let plans: Vec<CapacityPlanV21> = vec![];
    (StatusCode::OK, Json(plans)).into_response()
}

pub fn rate_limiting_v21_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v21/rate-limits/quotas", get(list_rate_limit_tier_quotas_v21).post(create_rate_limit_tier_quota_v21))
        .route("/api/v21/rate-limits/quotas/{tier}", get(get_rate_limit_tier_quota_v21).put(update_rate_limit_tier_quota_v21))
        .route("/api/v21/rate-limits/usage", get(get_usage_analytics_v21))
        .route("/api/v21/rate-limits/adjustments", get(get_dynamic_adjustments_v21))
        .route("/api/v21/rate-limits/capacity", get(get_capacity_planning_v21))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_tier_quota_v21_response_serialization() {
        let response = RateLimitTierQuotaV21Response {
            id: Uuid::nil(),
            tier: "enterprise".into(),
            requests_per_second: 1000,
            requests_per_day: 1000000,
            burst_size: 500,
            enabled: true,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"tier\":\"enterprise\""));
        assert!(json.contains("1000"));
    }

    #[test]
    fn test_dynamic_adjustment_v21_serialization() {
        let adjustment = DynamicAdjustmentV21 {
            tier: "standard".into(),
            current_rps: 50,
            recommended_rps: 75,
            reason: "Increased traffic pattern".into(),
            confidence: 0.85,
        };
        let json = serde_json::to_string(&adjustment).unwrap();
        assert!(json.contains("standard"));
        assert!(json.contains("0.85"));
    }

    #[test]
    fn test_capacity_plan_v21_serialization() {
        let plan = CapacityPlanV21 {
            tier: "pro".into(),
            current_utilization: 0.65,
            projected_utilization_7d: 0.72,
            projected_utilization_30d: 0.85,
            recommended_capacity: 200,
            headroom_percent: 20.0,
        };
        let json = serde_json::to_string(&plan).unwrap();
        assert!(json.contains("pro"));
        assert!(json.contains("0.65"));
    }
}
