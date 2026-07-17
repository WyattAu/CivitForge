#![forbid(unsafe_code)]

//! Consolidated Rate Limiting API routes.
//!
//! Merges all rate_limiting_v2 through v21 routes into a single module.
//! Uses the latest v21 database methods and includes unique endpoints
//! from older versions (dashboard, quota, features, overages, costs,
//! enforcement, alerts).

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
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Response / Request types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitTierQuotaResponse {
    pub id: Uuid,
    pub tier: String,
    pub requests_per_second: i32,
    pub requests_per_day: i32,
    pub burst_size: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRateLimitTierQuotaRequest {
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
pub struct UpdateRateLimitTierQuotaRequest {
    pub requests_per_second: Option<i32>,
    pub requests_per_day: Option<i32>,
    pub burst_size: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitUsageAnalyticsResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier: String,
    pub requests_used: i32,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicAdjustment {
    pub tier: String,
    pub current_rps: i32,
    pub recommended_rps: i32,
    pub reason: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityPlan {
    pub tier: String,
    pub current_utilization: f64,
    pub projected_utilization_7d: f64,
    pub projected_utilization_30d: f64,
    pub recommended_capacity: i32,
    pub headroom_percent: f64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitCheckResponse {
    pub allowed: bool,
    pub remaining: i64,
    pub limit: i64,
    pub reset_at: DateTime<Utc>,
    pub tier: String,
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
pub struct AlertHistoryEntry {
    pub id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub triggered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNotificationEntry {
    pub id: Uuid,
    pub alert_type: String,
    pub channel: String,
    pub sent_at: DateTime<Utc>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertAnalyticsEntry {
    pub alert_type: String,
    pub total_triggers: i64,
    pub avg_response_time_ms: f64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitAlertListEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRateLimitAlertRequest {
    pub tier_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
}

// ---------------------------------------------------------------------------
// Quota handlers (v21)
// ---------------------------------------------------------------------------

pub async fn list_rate_limit_tier_quotas(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.list_rate_limit_tier_quotas_v21().await {
        Ok(quotas) => {
            let response: Vec<RateLimitTierQuotaResponse> = quotas.iter().map(|q| RateLimitTierQuotaResponse {
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

pub async fn get_rate_limit_tier_quota(
    State(state): State<AppState>,
    axum::extract::Path(tier): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state.db.get_rate_limit_tier_quota_v21_by_tier(&tier).await {
        Ok(Some(quota)) => {
            let response = RateLimitTierQuotaResponse {
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

pub async fn create_rate_limit_tier_quota(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateRateLimitTierQuotaRequest>,
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
            let response = RateLimitTierQuotaResponse {
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

pub async fn update_rate_limit_tier_quota(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(tier): axum::extract::Path<String>,
    Json(req): Json<UpdateRateLimitTierQuotaRequest>,
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
            let response = RateLimitTierQuotaResponse {
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

// ---------------------------------------------------------------------------
// Usage analytics (v21)
// ---------------------------------------------------------------------------

pub async fn get_usage_analytics(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());
    match state.db.get_rate_limit_usage_analytics_v21_by_user(user_id).await {
        Ok(analytics) => {
            let response: Vec<RateLimitUsageAnalyticsResponse> = analytics.iter().map(|a| RateLimitUsageAnalyticsResponse {
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

// ---------------------------------------------------------------------------
// Dynamic adjustments & capacity planning (v21)
// ---------------------------------------------------------------------------

pub async fn get_dynamic_adjustments() -> impl IntoResponse {
    let adjustments: Vec<DynamicAdjustment> = vec![];
    (StatusCode::OK, Json(adjustments)).into_response()
}

pub async fn get_capacity_planning() -> impl IntoResponse {
    let plans: Vec<CapacityPlan> = vec![];
    (StatusCode::OK, Json(plans)).into_response()
}

// ---------------------------------------------------------------------------
// Overages (v4)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Cost tracking (v4)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Limit enforcement (v4)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Rate limit check (v2)
// ---------------------------------------------------------------------------

pub async fn check_rate_limit(
    State(_state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let _user_id = auth.user_id;
    let response = RateLimitCheckResponse {
        allowed: true,
        remaining: 999,
        limit: 1000,
        reset_at: Utc::now() + chrono::Duration::hours(1),
        tier: "free".into(),
    };
    (StatusCode::OK, Json(response)).into_response()
}

// ---------------------------------------------------------------------------
// User usage (v2)
// ---------------------------------------------------------------------------

pub async fn get_user_usage(
    State(_state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());
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

// ---------------------------------------------------------------------------
// Dashboard (v2)
// ---------------------------------------------------------------------------

pub async fn get_rate_limit_dashboard(
    State(_state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let _user_id = auth.user_id;
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

// ---------------------------------------------------------------------------
// Quota management (v3)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Feature access check (v3)
// ---------------------------------------------------------------------------

pub async fn check_feature_access(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(feature): axum::extract::Path<String>,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());
    match state.db.check_feature_access(user_id, &feature).await {
        Ok((enabled, tier)) => {
            let response = FeatureCheckResponse { feature, enabled, tier };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// Alerts (v6+)
// ---------------------------------------------------------------------------

pub async fn get_user_rate_limit_alerts(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());
    match state.db.get_user_rate_limit_alerts(user_id).await {
        Ok(alerts) => {
            let response: Vec<serde_json::Value> = alerts.iter().map(|a| serde_json::json!({
                "id": a.id,
                "user_id": a.user_id,
                "tier_id": a.tier_id,
                "alert_type": a.alert_type,
                "threshold_percent": a.threshold_percent,
                "current_usage": a.current_usage,
                "triggered_at": a.triggered_at,
                "acknowledged": a.acknowledged,
            })).collect();
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create_rate_limit_alert(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateRateLimitAlertRequest>,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());
    match state.db.create_rate_limit_alert_v11_for_v14(user_id, req.tier_id, &req.alert_type, req.threshold).await {
        Ok(alert) => {
            let response = RateLimitAlertListEntry {
                id: alert.id,
                user_id: alert.user_id,
                tier_id: alert.tier_id,
                alert_type: alert.alert_type,
                threshold: alert.threshold,
                enabled: alert.enabled,
                last_triggered_at: alert.last_triggered_at,
                created_at: alert.created_at,
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

pub async fn get_alert_history() -> impl IntoResponse {
    let history: Vec<AlertHistoryEntry> = vec![];
    (StatusCode::OK, Json(history)).into_response()
}

pub async fn get_alert_notifications() -> impl IntoResponse {
    let notifications: Vec<AlertNotificationEntry> = vec![];
    (StatusCode::OK, Json(notifications)).into_response()
}

pub async fn get_alert_analytics() -> impl IntoResponse {
    let analytics: Vec<AlertAnalyticsEntry> = vec![];
    (StatusCode::OK, Json(analytics)).into_response()
}

// ---------------------------------------------------------------------------
// Consolidated router
// ---------------------------------------------------------------------------

pub fn routes() -> Router<AppState> {
    Router::new()
        // Quota management (v21)
        .route("/api/v21/rate-limits/quotas", get(list_rate_limit_tier_quotas).post(create_rate_limit_tier_quota))
        .route("/api/v21/rate-limits/quotas/{tier}", get(get_rate_limit_tier_quota).put(update_rate_limit_tier_quota))
        .route("/api/v21/rate-limits/usage", get(get_usage_analytics))
        .route("/api/v21/rate-limits/adjustments", get(get_dynamic_adjustments))
        .route("/api/v21/rate-limits/capacity", get(get_capacity_planning))
        // Legacy v2 endpoints
        .route("/api/v2/rate-limits/check", get(check_rate_limit))
        .route("/api/v2/rate-limits/usage", get(get_user_usage))
        .route("/api/v2/rate-limits/dashboard", get(get_rate_limit_dashboard))
        // Legacy v3 endpoints
        .route("/api/v3/rate-limits/quota", get(get_quota_management))
        .route("/api/v3/rate-limits/features/{feature}", get(check_feature_access))
        // Legacy v4 endpoints
        .route("/api/v4/rate-limits/overages", get(get_user_overages))
        .route("/api/v4/rate-limits/costs", get(get_user_cost_tracking))
        .route("/api/v4/rate-limits/enforcement", get(get_limit_enforcement))
        // Alert endpoints (v6+)
        .route("/api/v6/rate-limits/alerts", get(get_user_rate_limit_alerts).post(create_rate_limit_alert))
        .route("/api/v6/rate-limits/alerts/history", get(get_alert_history))
        .route("/api/v6/rate-limits/alerts/notifications", get(get_alert_notifications))
        .route("/api/v6/rate-limits/alerts/analytics", get(get_alert_analytics))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_tier_quota_response_serialization() {
        let response = RateLimitTierQuotaResponse {
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
    fn test_dynamic_adjustment_serialization() {
        let adjustment = DynamicAdjustment {
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
    fn test_capacity_plan_serialization() {
        let plan = CapacityPlan {
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
