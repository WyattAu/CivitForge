#![forbid(unsafe_code)]

//! Usage quotas routes for managing per-user/plan usage limits.

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_admin};
use axum::{
    Json,
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, delete},
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageQuotaResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub quota_type: String,
    pub quota_limit: i32,
    pub quota_used: i32,
    pub period_start: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateQuotaRequest {
    pub user_id: Uuid,
    pub quota_type: String,
    pub quota_limit: i32,
    pub period_start: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncrementQuotaRequest {
    pub user_id: Uuid,
    pub quota_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageQuotasResponse {
    pub quotas: Vec<UsageQuotaResponse>,
    pub total: usize,
}

pub async fn list_usage_quotas(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.get_user_usage_quotas(user_id).await {
        Ok(quotas) => {
            let response = UsageQuotasResponse {
                quotas: quotas.iter().map(|q| UsageQuotaResponse {
                    id: q.id,
                    user_id: q.user_id,
                    quota_type: q.quota_type.clone(),
                    quota_limit: q.quota_limit,
                    quota_used: q.quota_used,
                    period_start: q.period_start,
                    created_at: q.created_at,
                }).collect(),
                total: quotas.len(),
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

pub async fn get_usage_quota(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((user_id, quota_type)): Path<(Uuid, String)>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.get_usage_quota(user_id, &quota_type).await {
        Ok(Some(quota)) => {
            let response = UsageQuotaResponse {
                id: quota.id,
                user_id: quota.user_id,
                quota_type: quota.quota_type.clone(),
                quota_limit: quota.quota_limit,
                quota_used: quota.quota_used,
                period_start: quota.period_start,
                created_at: quota.created_at,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "quota not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create_usage_quota(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateQuotaRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    if req.quota_type.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "quota_type is required"})),
        )
            .into_response();
    }

    if req.quota_limit <= 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "quota_limit must be positive"})),
        )
            .into_response();
    }

    let period_start = req.period_start.unwrap_or_else(Utc::now);

    match state.db.create_usage_quota(req.user_id, &req.quota_type, req.quota_limit, period_start).await {
        Ok(quota) => {
            let response = UsageQuotaResponse {
                id: quota.id,
                user_id: quota.user_id,
                quota_type: quota.quota_type.clone(),
                quota_limit: quota.quota_limit,
                quota_used: quota.quota_used,
                period_start: quota.period_start,
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

pub async fn increment_usage_quota(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<IncrementQuotaRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.get_usage_quota(req.user_id, &req.quota_type).await {
        Ok(Some(quota)) => {
            if quota.quota_used >= quota.quota_limit {
                return (
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({"error": "quota exceeded", "quota_type": req.quota_type, "quota_used": quota.quota_used, "quota_limit": quota.quota_limit})),
                )
                    .into_response();
            }

            match state.db.increment_usage_quota(req.user_id, &req.quota_type).await {
                Ok(updated) => {
                    let response = UsageQuotaResponse {
                        id: updated.id,
                        user_id: updated.user_id,
                        quota_type: updated.quota_type.clone(),
                        quota_limit: updated.quota_limit,
                        quota_used: updated.quota_used,
                        period_start: updated.period_start,
                        created_at: updated.created_at,
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
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "quota not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn delete_usage_quota(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.delete_usage_quota(id).await {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub fn usage_quota_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/admin/quotas/{user_id}",
            get(list_usage_quotas),
        )
        .route(
            "/api/v1/admin/quotas/{user_id}/{quota_type}",
            get(get_usage_quota),
        )
        .route(
            "/api/v1/admin/quotas",
            post(create_usage_quota),
        )
        .route(
            "/api/v1/admin/quotas/increment",
            post(increment_usage_quota),
        )
        .route(
            "/api/v1/admin/quotas/{user_id}",
            delete(delete_usage_quota),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_quota_response_serialization() {
        let response = UsageQuotaResponse {
            id: Uuid::nil(),
            user_id: Uuid::nil(),
            quota_type: "api_requests".into(),
            quota_limit: 1000,
            quota_used: 50,
            period_start: Utc::now(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"api_requests\""));
        assert!(json.contains("\"quota_limit\":1000"));
    }

    #[test]
    fn test_create_quota_request_deserialization() {
        let json = r#"{"user_id": "00000000-0000-0000-0000-000000000000", "quota_type": "api_requests", "quota_limit": 1000}"#;
        let req: CreateQuotaRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.quota_type, "api_requests");
        assert_eq!(req.quota_limit, 1000);
    }

    #[test]
    fn test_increment_quota_request_deserialization() {
        let json = r#"{"user_id": "00000000-0000-0000-0000-000000000000", "quota_type": "api_requests"}"#;
        let req: IncrementQuotaRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.quota_type, "api_requests");
    }
}