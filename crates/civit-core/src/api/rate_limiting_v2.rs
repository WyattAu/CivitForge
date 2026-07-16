#![forbid(unsafe_code)]

//! API Rate Limiting v2 routes with tier-based rate limiting, usage metering,
//! overage billing, and quota management.

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
pub struct RateLimitTierResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    pub price_cents: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRateLimitTierRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    #[serde(default)]
    pub price_cents: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateRateLimitTierRequest {
    #[serde(default)]
    pub description: Option<String>,
    pub rate_limit: Option<i32>,
    pub burst_limit: Option<i32>,
    pub monthly_quota: Option<Option<i32>>,
    pub price_cents: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitUsageResponse {
    pub user_id: Uuid,
    pub tier_name: String,
    pub current_usage: i64,
    pub monthly_usage: i64,
    pub quota_remaining: Option<i64>,
    pub overage_cents: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitCheckResponse {
    pub allowed: bool,
    pub remaining: i64,
    pub limit: i64,
    pub reset_at: DateTime<Utc>,
    pub tier: String,
}

pub async fn list_rate_limit_tiers(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.db.list_rate_limit_tiers().await {
        Ok(tiers) => {
            let response: Vec<RateLimitTierResponse> = tiers.iter().map(|t| RateLimitTierResponse {
                id: t.id,
                name: t.name.clone(),
                description: t.description.clone(),
                rate_limit: t.rate_limit,
                burst_limit: t.burst_limit,
                monthly_quota: t.monthly_quota,
                price_cents: t.price_cents,
                created_at: t.created_at,
            }).collect();
            (StatusCode::OK, Json(serde_json::json!({"tiers": response, "total": response.len()}))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_rate_limit_tier(
    State(state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state.db.get_rate_limit_tier(&name).await {
        Ok(tier) => {
            let response = RateLimitTierResponse {
                id: tier.id,
                name: tier.name,
                description: tier.description,
                rate_limit: tier.rate_limit,
                burst_limit: tier.burst_limit,
                monthly_quota: tier.monthly_quota,
                price_cents: tier.price_cents,
                created_at: tier.created_at,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create_rate_limit_tier(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateRateLimitTierRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.create_rate_limit_tier(
        &req.name,
        &req.description,
        req.rate_limit,
        req.burst_limit,
        req.monthly_quota,
        req.price_cents,
    ).await {
        Ok(tier) => {
            let response = RateLimitTierResponse {
                id: tier.id,
                name: tier.name,
                description: tier.description,
                rate_limit: tier.rate_limit,
                burst_limit: tier.burst_limit,
                monthly_quota: tier.monthly_quota,
                price_cents: tier.price_cents,
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

pub async fn update_rate_limit_tier(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(req): Json<UpdateRateLimitTierRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.update_rate_limit_tier(
        &name,
        req.description.as_deref(),
        req.rate_limit,
        req.burst_limit,
        req.monthly_quota,
        req.price_cents,
    ).await {
        Ok(tier) => {
            let response = RateLimitTierResponse {
                id: tier.id,
                name: tier.name,
                description: tier.description,
                rate_limit: tier.rate_limit,
                burst_limit: tier.burst_limit,
                monthly_quota: tier.monthly_quota,
                price_cents: tier.price_cents,
                created_at: tier.created_at,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn delete_rate_limit_tier(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.delete_rate_limit_tier(&name).await {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn check_rate_limit(
    State(_state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let _user_id = auth.user_id;
    
    // In a real implementation, this would check the user's tier and current usage
    // For now, return a mock response
    let response = RateLimitCheckResponse {
        allowed: true,
        remaining: 999,
        limit: 1000,
        reset_at: Utc::now() + chrono::Duration::hours(1),
        tier: "free".into(),
    };
    
    (StatusCode::OK, Json(response)).into_response()
}

pub async fn get_user_usage(
    State(_state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());
    
    // In a real implementation, this would query the user's actual usage
    // For now, return a mock response
    let response = RateLimitUsageResponse {
        user_id,
        tier_name: "free".into(),
        current_usage: 42,
        monthly_usage: 1234,
        quota_remaining: Some(8766),
        overage_cents: 0,
    };
    
    (StatusCode::OK, Json(response)).into_response()
}

pub async fn get_rate_limit_dashboard(
    State(_state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }
    
    // In a real implementation, this would aggregate usage data
    // For now, return a mock dashboard
    let response = serde_json::json!({
        "total_users": 1000,
        "active_users_today": 250,
        "total_requests_today": 50000,
        "top_tiers": [
            {"name": "free", "users": 800, "requests": 30000},
            {"name": "pro", "users": 150, "requests": 15000},
            {"name": "enterprise", "users": 50, "requests": 5000},
        ],
        "overage_revenue_cents": 12500,
    });
    
    (StatusCode::OK, Json(response)).into_response()
}

pub fn rate_limiting_v2_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v2/rate-limits/tiers", get(list_rate_limit_tiers).post(create_rate_limit_tier))
        .route("/api/v2/rate-limits/tiers/{name}", get(get_rate_limit_tier).put(update_rate_limit_tier).delete(delete_rate_limit_tier))
        .route("/api/v2/rate-limits/check", get(check_rate_limit))
        .route("/api/v2/rate-limits/usage", get(get_user_usage))
        .route("/api/v2/rate-limits/dashboard", get(get_rate_limit_dashboard))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_tier_response_serialization() {
        let response = RateLimitTierResponse {
            id: Uuid::nil(),
            name: "free".into(),
            description: "Free tier".into(),
            rate_limit: 1000,
            burst_limit: 100,
            monthly_quota: Some(100000),
            price_cents: 0,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"name\":\"free\""));
        assert!(json.contains("\"rate_limit\":1000"));
    }

    #[test]
    fn test_rate_limit_check_response_serialization() {
        let response = RateLimitCheckResponse {
            allowed: true,
            remaining: 999,
            limit: 1000,
            reset_at: Utc::now(),
            tier: "pro".into(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"allowed\":true"));
        assert!(json.contains("\"remaining\":999"));
    }

    #[test]
    fn test_rate_limit_usage_response_serialization() {
        let response = RateLimitUsageResponse {
            user_id: Uuid::nil(),
            tier_name: "enterprise".into(),
            current_usage: 500,
            monthly_usage: 50000,
            quota_remaining: Some(50000),
            overage_cents: 0,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"tier_name\":\"enterprise\""));
        assert!(json.contains("\"current_usage\":500"));
    }
}
