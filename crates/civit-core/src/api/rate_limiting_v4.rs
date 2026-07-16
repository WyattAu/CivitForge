#![forbid(unsafe_code)]

//! API Rate Limiting v4 routes with overage billing, usage alerts,
//! limit enforcement, and cost tracking.

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
pub struct RateLimitTierV3Response {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    pub price_cents: i32,
    pub features: serde_json::Value,
    pub limits: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRateLimitTierV3Request {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    #[serde(default)]
    pub price_cents: i32,
    #[serde(default)]
    pub features: serde_json::Value,
    #[serde(default = "default_limits")]
    pub limits: serde_json::Value,
}

fn default_limits() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateRateLimitTierV3Request {
    #[serde(default)]
    pub description: Option<String>,
    pub rate_limit: Option<i32>,
    pub burst_limit: Option<i32>,
    pub monthly_quota: Option<Option<i32>>,
    pub price_cents: Option<i32>,
    pub features: Option<serde_json::Value>,
    pub limits: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitOverageResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub overage_count: i32,
    pub overage_cost_cents: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitAlertResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier_id: Uuid,
    pub alert_type: String,
    pub threshold_percent: i32,
    pub current_usage: i32,
    pub triggered_at: DateTime<Utc>,
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostTrackingResponse {
    pub user_id: Uuid,
    pub total_overage_cents: i32,
    pub current_period_overage_cents: i32,
    pub overage_count: i64,
    pub alerts: Vec<RateLimitAlertResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitEnforcementResponse {
    pub user_id: Uuid,
    pub tier: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    pub current_usage: i64,
    pub quota_remaining: Option<i64>,
    pub is_throttled: bool,
    pub throttle_until: Option<DateTime<Utc>>,
}

pub async fn list_rate_limit_tiers_v3(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.list_rate_limit_tiers_v3().await {
        Ok(tiers) => {
            let response: Vec<RateLimitTierV3Response> = tiers.iter().map(|t| RateLimitTierV3Response {
                id: t.id,
                name: t.name.clone(),
                description: t.description.clone(),
                rate_limit: t.rate_limit,
                burst_limit: t.burst_limit,
                monthly_quota: t.monthly_quota,
                price_cents: t.price_cents,
                features: t.features.clone(),
                limits: t.limits.clone(),
                created_at: t.created_at,
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

pub async fn get_rate_limit_tier_v3(
    State(state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state.db.get_rate_limit_tier_v3_by_name(&name).await {
        Ok(Some(tier)) => {
            let response = RateLimitTierV3Response {
                id: tier.id,
                name: tier.name,
                description: tier.description,
                rate_limit: tier.rate_limit,
                burst_limit: tier.burst_limit,
                monthly_quota: tier.monthly_quota,
                price_cents: tier.price_cents,
                features: tier.features,
                limits: tier.limits,
                created_at: tier.created_at,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Tier not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create_rate_limit_tier_v3(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateRateLimitTierV3Request>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.create_rate_limit_tier_v3(
        &req.name,
        &req.description,
        req.rate_limit,
        req.burst_limit,
        req.monthly_quota,
        req.price_cents,
        &req.features,
        &req.limits,
    ).await {
        Ok(tier) => {
            let response = RateLimitTierV3Response {
                id: tier.id,
                name: tier.name,
                description: tier.description,
                rate_limit: tier.rate_limit,
                burst_limit: tier.burst_limit,
                monthly_quota: tier.monthly_quota,
                price_cents: tier.price_cents,
                features: tier.features,
                limits: tier.limits,
                created_at: tier.created_at,
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

pub async fn update_rate_limit_tier_v3(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(req): Json<UpdateRateLimitTierV3Request>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.update_rate_limit_tier_v3(
        &name,
        req.description.as_deref(),
        req.rate_limit,
        req.burst_limit,
        req.monthly_quota,
        req.price_cents,
        req.features.as_ref(),
        req.limits.as_ref(),
    ).await {
        Ok(tier) => {
            let response = RateLimitTierV3Response {
                id: tier.id,
                name: tier.name,
                description: tier.description,
                rate_limit: tier.rate_limit,
                burst_limit: tier.burst_limit,
                monthly_quota: tier.monthly_quota,
                price_cents: tier.price_cents,
                features: tier.features,
                limits: tier.limits,
                created_at: tier.created_at,
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

pub async fn delete_rate_limit_tier_v3(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.delete_rate_limit_tier_v3(&name).await {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_user_overages(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());
    match state.db.get_user_overages(user_id).await {
        Ok(overages) => {
            let response: Vec<RateLimitOverageResponse> = overages.iter().map(|o| RateLimitOverageResponse {
                id: o.id,
                user_id: o.user_id,
                tier_id: o.tier_id,
                period_start: o.period_start,
                overage_count: o.overage_count,
                overage_cost_cents: o.overage_cost_cents,
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

pub async fn get_user_cost_tracking(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());
    match state.db.get_user_overages(user_id).await {
        Ok(overages) => {
            match state.db.get_user_rate_limit_alerts(user_id).await {
                Ok(alerts) => {
                    let total_cents: i32 = overages.iter().map(|o| o.overage_cost_cents).sum();
                    let alert_responses: Vec<RateLimitAlertResponse> = alerts.iter().map(|a| RateLimitAlertResponse {
                        id: a.id,
                        user_id: a.user_id,
                        tier_id: a.tier_id,
                        alert_type: a.alert_type.clone(),
                        threshold_percent: a.threshold_percent,
                        current_usage: a.current_usage,
                        triggered_at: a.triggered_at,
                        acknowledged: a.acknowledged,
                    }).collect();
                    let response = CostTrackingResponse {
                        user_id,
                        total_overage_cents: total_cents,
                        current_period_overage_cents: overages.first().map(|o| o.overage_cost_cents).unwrap_or(0),
                        overage_count: overages.len() as i64,
                        alerts: alert_responses,
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
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_limit_enforcement(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());
    match state.db.get_quota_management(user_id).await {
        Ok(Some(mgmt)) => {
            let response = LimitEnforcementResponse {
                user_id,
                tier: mgmt.0,
                rate_limit: 0,
                burst_limit: 0,
                monthly_quota: mgmt.2.map(|v| v as i32),
                current_usage: mgmt.1,
                quota_remaining: mgmt.3,
                is_throttled: mgmt.4 > 0,
                throttle_until: None,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "No active tier found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_user_rate_limit_alerts(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());
    match state.db.get_user_rate_limit_alerts(user_id).await {
        Ok(alerts) => {
            let response: Vec<RateLimitAlertResponse> = alerts.iter().map(|a| RateLimitAlertResponse {
                id: a.id,
                user_id: a.user_id,
                tier_id: a.tier_id,
                alert_type: a.alert_type.clone(),
                threshold_percent: a.threshold_percent,
                current_usage: a.current_usage,
                triggered_at: a.triggered_at,
                acknowledged: a.acknowledged,
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

pub fn rate_limiting_v4_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v4/rate-limits/tiers", get(list_rate_limit_tiers_v3).post(create_rate_limit_tier_v3))
        .route("/api/v4/rate-limits/tiers/{name}", get(get_rate_limit_tier_v3).put(update_rate_limit_tier_v3).delete(delete_rate_limit_tier_v3))
        .route("/api/v4/rate-limits/overages", get(get_user_overages))
        .route("/api/v4/rate-limits/costs", get(get_user_cost_tracking))
        .route("/api/v4/rate-limits/enforcement", get(get_limit_enforcement))
        .route("/api/v4/rate-limits/alerts", get(get_user_rate_limit_alerts))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_tier_v3_response_serialization() {
        let response = RateLimitTierV3Response {
            id: Uuid::nil(),
            name: "enterprise".into(),
            description: "Enterprise tier".into(),
            rate_limit: 50000,
            burst_limit: 2000,
            monthly_quota: Some(10000000),
            price_cents: 9900,
            features: serde_json::json!({"analytics": true, "webhooks": true, "sso": true}),
            limits: serde_json::json!({"max_repos": 1000, "max_members": 500}),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"name\":\"enterprise\""));
        assert!(json.contains("analytics"));
        assert!(json.contains("\"price_cents\":9900"));
        assert!(json.contains("limits"));
    }

    #[test]
    fn test_cost_tracking_response_serialization() {
        let response = CostTrackingResponse {
            user_id: Uuid::nil(),
            total_overage_cents: 500,
            current_period_overage_cents: 200,
            overage_count: 3,
            alerts: vec![],
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("total_overage_cents"));
        assert!(json.contains("500"));
    }

    #[test]
    fn test_limit_enforcement_response_serialization() {
        let response = LimitEnforcementResponse {
            user_id: Uuid::nil(),
            tier: "pro".into(),
            rate_limit: 10000,
            burst_limit: 500,
            monthly_quota: Some(1000000),
            current_usage: 500000,
            quota_remaining: Some(500000),
            is_throttled: false,
            throttle_until: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("is_throttled"));
        assert!(json.contains("quota_remaining"));
    }
}
