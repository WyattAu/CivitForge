#![forbid(unsafe_code)]

//! API Rate Limiting v3 routes with feature-based tiers, usage tracking,
//! overage billing, and quota management.

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
pub struct RateLimitTierV2Response {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    pub price_cents: i32,
    pub features: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRateLimitTierV2Request {
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateRateLimitTierV2Request {
    #[serde(default)]
    pub description: Option<String>,
    pub rate_limit: Option<i32>,
    pub burst_limit: Option<i32>,
    pub monthly_quota: Option<Option<i32>>,
    pub price_cents: Option<i32>,
    pub features: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitUsageV2Response {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub usage_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitCheckV3Response {
    pub allowed: bool,
    pub remaining: i64,
    pub limit: i64,
    pub reset_at: DateTime<Utc>,
    pub tier: String,
    pub features: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaManagementResponse {
    pub user_id: Uuid,
    pub tier_name: String,
    pub monthly_usage: i64,
    pub monthly_quota: Option<i64>,
    pub quota_remaining: Option<i64>,
    pub overage_cents: i32,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureCheckResponse {
    pub feature: String,
    pub enabled: bool,
    pub tier: String,
}

pub async fn list_rate_limit_tiers_v2(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.list_rate_limit_tiers_v2().await {
        Ok(tiers) => {
            let response: Vec<RateLimitTierV2Response> = tiers.iter().map(|t| RateLimitTierV2Response {
                id: t.id,
                name: t.name.clone(),
                description: t.description.clone(),
                rate_limit: t.rate_limit,
                burst_limit: t.burst_limit,
                monthly_quota: t.monthly_quota,
                price_cents: t.price_cents,
                features: t.features.clone(),
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

pub async fn get_rate_limit_tier_v2(
    State(state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state.db.get_rate_limit_tier_v2_by_name(&name).await {
        Ok(Some(tier)) => {
            let response = RateLimitTierV2Response {
                id: tier.id,
                name: tier.name,
                description: tier.description,
                rate_limit: tier.rate_limit,
                burst_limit: tier.burst_limit,
                monthly_quota: tier.monthly_quota,
                price_cents: tier.price_cents,
                features: tier.features,
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

pub async fn create_rate_limit_tier_v2(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateRateLimitTierV2Request>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.create_rate_limit_tier_v2(
        &req.name,
        &req.description,
        req.rate_limit,
        req.burst_limit,
        req.monthly_quota,
        req.price_cents,
        &req.features,
    ).await {
        Ok(tier) => {
            let response = RateLimitTierV2Response {
                id: tier.id,
                name: tier.name,
                description: tier.description,
                rate_limit: tier.rate_limit,
                burst_limit: tier.burst_limit,
                monthly_quota: tier.monthly_quota,
                price_cents: tier.price_cents,
                features: tier.features,
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

pub async fn update_rate_limit_tier_v2(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(req): Json<UpdateRateLimitTierV2Request>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.update_rate_limit_tier_v2(
        &name,
        req.description.as_deref(),
        req.rate_limit,
        req.burst_limit,
        req.monthly_quota,
        req.price_cents,
        req.features.as_ref(),
    ).await {
        Ok(tier) => {
            let response = RateLimitTierV2Response {
                id: tier.id,
                name: tier.name,
                description: tier.description,
                rate_limit: tier.rate_limit,
                burst_limit: tier.burst_limit,
                monthly_quota: tier.monthly_quota,
                price_cents: tier.price_cents,
                features: tier.features,
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

pub async fn delete_rate_limit_tier_v2(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.delete_rate_limit_tier_v2(&name).await {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn check_rate_limit_v3(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());
    match state.db.check_rate_limit_v3(user_id).await {
        Ok((allowed, remaining, limit, reset_at, tier, features)) => {
            let response = RateLimitCheckV3Response {
                allowed,
                remaining,
                limit,
                reset_at,
                tier,
                features,
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

pub async fn get_user_usage_v2(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());
    match state.db.get_user_usage_v2(user_id).await {
        Ok(usage) => {
            let response: Vec<RateLimitUsageV2Response> = usage.iter().map(|u| RateLimitUsageV2Response {
                id: u.id,
                user_id: u.user_id,
                tier_id: u.tier_id,
                period_start: u.period_start,
                usage_count: u.usage_count,
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

pub async fn get_quota_management(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());
    match state.db.get_quota_management(user_id).await {
        Ok(Some(mgmt)) => {
            let response = QuotaManagementResponse {
                user_id,
                tier_name: mgmt.0,
                monthly_usage: mgmt.1,
                monthly_quota: mgmt.2,
                quota_remaining: mgmt.3,
                overage_cents: mgmt.4,
                period_start: mgmt.5,
                period_end: mgmt.6,
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

pub async fn check_feature_access(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(feature): axum::extract::Path<String>,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());
    match state.db.check_feature_access(user_id, &feature).await {
        Ok((enabled, tier)) => {
            let response = FeatureCheckResponse {
                feature,
                enabled,
                tier,
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

pub fn rate_limiting_v3_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v3/rate-limits/tiers", get(list_rate_limit_tiers_v2).post(create_rate_limit_tier_v2))
        .route("/api/v3/rate-limits/tiers/{name}", get(get_rate_limit_tier_v2).put(update_rate_limit_tier_v2).delete(delete_rate_limit_tier_v2))
        .route("/api/v3/rate-limits/check", get(check_rate_limit_v3))
        .route("/api/v3/rate-limits/usage", get(get_user_usage_v2))
        .route("/api/v3/rate-limits/quota", get(get_quota_management))
        .route("/api/v3/rate-limits/features/{feature}", get(check_feature_access))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_tier_v2_response_serialization() {
        let response = RateLimitTierV2Response {
            id: Uuid::nil(),
            name: "pro".into(),
            description: "Professional tier".into(),
            rate_limit: 10000,
            burst_limit: 500,
            monthly_quota: Some(1000000),
            price_cents: 2900,
            features: serde_json::json!({"analytics": true, "webhooks": true}),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"name\":\"pro\""));
        assert!(json.contains("analytics"));
        assert!(json.contains("\"price_cents\":2900"));
    }

    #[test]
    fn test_quota_management_response_serialization() {
        let response = QuotaManagementResponse {
            user_id: Uuid::nil(),
            tier_name: "pro".into(),
            monthly_usage: 50000,
            monthly_quota: Some(100000),
            quota_remaining: Some(50000),
            overage_cents: 0,
            period_start: Utc::now(),
            period_end: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("quota_remaining"));
    }
}
