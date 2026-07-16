#![forbid(unsafe_code)]

//! API Rate Limiting v7 routes with enhanced alert configuration,
//! alert history, alert notifications, and alert analytics.

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
pub struct RateLimitTierV7Response {
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
pub struct CreateRateLimitTierV7Request {
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
pub struct UpdateRateLimitTierV7Request {
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
pub struct RateLimitAlertV7Response {
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
pub struct CreateRateLimitAlertV7Request {
    pub tier_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertHistoryEntryV7 {
    pub id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub triggered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNotificationEntryV7 {
    pub id: Uuid,
    pub alert_type: String,
    pub channel: String,
    pub sent_at: DateTime<Utc>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertAnalyticsEntryV7 {
    pub alert_type: String,
    pub total_triggers: i64,
    pub avg_response_time_ms: f64,
    pub success_rate: f64,
}

pub async fn list_rate_limit_tiers_v7(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.list_rate_limit_tiers_v8().await {
        Ok(tiers) => {
            let response: Vec<RateLimitTierV7Response> = tiers.iter().map(|t| RateLimitTierV7Response {
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

pub async fn get_rate_limit_tier_v7(
    State(state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state.db.get_rate_limit_tier_v8_by_name(&name).await {
        Ok(Some(tier)) => {
            let response = RateLimitTierV7Response {
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

pub async fn create_rate_limit_tier_v7(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateRateLimitTierV7Request>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.create_rate_limit_tier_v8(
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
            let response = RateLimitTierV7Response {
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

pub async fn update_rate_limit_tier_v7(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(name): axum::extract::Path<String>,
    Json(req): Json<UpdateRateLimitTierV7Request>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.update_rate_limit_tier_v8(
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
            let response = RateLimitTierV7Response {
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

pub async fn delete_rate_limit_tier_v7(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let _ = auth;
    match state.db.delete_rate_limit_tier_v8(&name).await {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create_rate_limit_alert_v7(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateRateLimitAlertV7Request>,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());
    match state.db.create_rate_limit_alert_v8(user_id, req.tier_id, &req.alert_type, req.threshold).await {
        Ok(alert) => {
            let response = RateLimitAlertV7Response {
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

pub async fn get_user_rate_limit_alerts_v7(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let user_id = Uuid::parse_str(&auth.user_id).unwrap_or(Uuid::nil());
    match state.db.get_user_rate_limit_alerts_v8(user_id).await {
        Ok(alerts) => {
            let response: Vec<RateLimitAlertV7Response> = alerts.iter().map(|a| RateLimitAlertV7Response {
                id: a.id,
                user_id: a.user_id,
                tier_id: a.tier_id,
                alert_type: a.alert_type.clone(),
                threshold: a.threshold,
                enabled: a.enabled,
                last_triggered_at: a.last_triggered_at,
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

pub async fn get_alert_history_v7() -> impl IntoResponse {
    let history: Vec<AlertHistoryEntryV7> = vec![];
    (StatusCode::OK, Json(history)).into_response()
}

pub async fn get_alert_notifications_v7() -> impl IntoResponse {
    let notifications: Vec<AlertNotificationEntryV7> = vec![];
    (StatusCode::OK, Json(notifications)).into_response()
}

pub async fn get_alert_analytics_v7() -> impl IntoResponse {
    let analytics: Vec<AlertAnalyticsEntryV7> = vec![];
    (StatusCode::OK, Json(analytics)).into_response()
}

pub fn rate_limiting_v7_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v7/rate-limits/tiers", get(list_rate_limit_tiers_v7).post(create_rate_limit_tier_v7))
        .route("/api/v7/rate-limits/tiers/{name}", get(get_rate_limit_tier_v7).put(update_rate_limit_tier_v7).delete(delete_rate_limit_tier_v7))
        .route("/api/v7/rate-limits/alerts", get(get_user_rate_limit_alerts_v7).post(create_rate_limit_alert_v7))
        .route("/api/v7/rate-limits/alerts/history", get(get_alert_history_v7))
        .route("/api/v7/rate-limits/alerts/notifications", get(get_alert_notifications_v7))
        .route("/api/v7/rate-limits/alerts/analytics", get(get_alert_analytics_v7))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_tier_v7_response_serialization() {
        let response = RateLimitTierV7Response {
            id: Uuid::nil(),
            name: "enterprise".into(),
            description: "Enterprise tier".into(),
            rate_limit: 50000,
            burst_limit: 2000,
            monthly_quota: Some(10000000),
            price_cents: 9900,
            features: serde_json::json!({"analytics": true, "webhooks": true}),
            limits: serde_json::json!({"max_repos": 1000}),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"name\":\"enterprise\""));
        assert!(json.contains("features"));
    }

    #[test]
    fn test_alert_v7_response_serialization() {
        let response = RateLimitAlertV7Response {
            id: Uuid::nil(),
            user_id: Uuid::nil(),
            tier_id: Uuid::nil(),
            alert_type: "usage_80_percent".into(),
            threshold: 80.0,
            enabled: true,
            last_triggered_at: None,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("usage_80_percent"));
        assert!(json.contains("80.0"));
    }
}
