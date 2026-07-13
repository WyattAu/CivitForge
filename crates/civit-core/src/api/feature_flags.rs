#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_admin};
use crate::error::CoreError;
use axum::{
    Json,
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put, delete},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlagResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub enabled_for_users: Vec<String>,
    pub enabled_for_percentage: i32,
    pub enabled_for_orgs: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlagsListResponse {
    pub flags: Vec<FeatureFlagResponse>,
    pub total: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateFeatureFlagRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub enabled_for_percentage: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateFeatureFlagRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_for_percentage: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToggleUserRequest {
    pub user_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToggleOrgRequest {
    pub org_id: String,
}

fn flag_to_response(flag: &civit_db::models::FeatureFlag) -> FeatureFlagResponse {
    FeatureFlagResponse {
        id: flag.id.to_string(),
        name: flag.name.clone(),
        description: flag.description.clone(),
        enabled: flag.enabled,
        enabled_for_users: flag.enabled_for_users.iter().map(|u| u.to_string()).collect(),
        enabled_for_percentage: flag.enabled_for_percentage,
        enabled_for_orgs: flag.enabled_for_orgs.iter().map(|o| o.to_string()).collect(),
        created_at: flag.created_at.to_rfc3339(),
        updated_at: flag.updated_at.to_rfc3339(),
    }
}

pub async fn list_feature_flags_for_user(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    let user_id: Uuid = match auth.user_id.parse() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "invalid user id"})),
            )
                .into_response();
        }
    };

    match state.db.list_enabled_feature_flags_for_user(user_id).await {
        Ok(flags) => {
            let response = FeatureFlagsListResponse {
                flags: flags.iter().map(|f| flag_to_response(f)).collect(),
                total: flags.len(),
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

pub async fn list_all_feature_flags(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.list_feature_flags().await {
        Ok(flags) => {
            let response = FeatureFlagsListResponse {
                flags: flags.iter().map(|f| flag_to_response(f)).collect(),
                total: flags.len(),
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

pub async fn create_feature_flag(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateFeatureFlagRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    if req.name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "name is required"})),
        )
            .into_response();
    }

    match state
        .db
        .create_feature_flag(&req.name, &req.description, req.enabled, req.enabled_for_percentage)
        .await
    {
        Ok(flag) => (StatusCode::CREATED, Json(flag_to_response(&flag))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn update_feature_flag(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateFeatureFlagRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state
        .db
        .update_feature_flag(
            id,
            req.name.as_deref(),
            req.description.as_deref(),
            req.enabled,
            req.enabled_for_percentage,
        )
        .await
    {
        Ok(flag) => (StatusCode::OK, Json(flag_to_response(&flag))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn delete_feature_flag(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.delete_feature_flag(id).await {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn toggle_feature_flag(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.toggle_feature_flag(id).await {
        Ok(flag) => {
            let _ = state
                .db
                .record_feature_flag_event(id, None, flag.enabled)
                .await;
            (StatusCode::OK, Json(flag_to_response(&flag))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn add_feature_flag_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<ToggleUserRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let user_id: Uuid = match req.user_id.parse() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "invalid user id"})),
            )
                .into_response();
        }
    };

    match state.db.add_feature_flag_user(id, user_id).await {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn remove_feature_flag_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, user_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.remove_feature_flag_user(id, user_id).await {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn add_feature_flag_org(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<ToggleOrgRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let org_id: Uuid = match req.org_id.parse() {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "invalid org id"})),
            )
                .into_response();
        }
    };

    match state.db.add_feature_flag_org(id, org_id).await {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn remove_feature_flag_org(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, org_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.remove_feature_flag_org(id, org_id).await {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub fn feature_flag_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/feature-flags", get(list_feature_flags_for_user))
        .route("/api/v1/admin/feature-flags", get(list_all_feature_flags).post(create_feature_flag))
        .route(
            "/api/v1/admin/feature-flags/{id}",
            put(update_feature_flag).delete(delete_feature_flag),
        )
        .route("/api/v1/admin/feature-flags/{id}/toggle", post(toggle_feature_flag))
        .route(
            "/api/v1/admin/feature-flags/{id}/users",
            post(add_feature_flag_user),
        )
        .route(
            "/api/v1/admin/feature-flags/{id}/users/{user_id}",
            delete(remove_feature_flag_user),
        )
        .route(
            "/api/v1/admin/feature-flags/{id}/orgs",
            post(add_feature_flag_org),
        )
        .route(
            "/api/v1/admin/feature-flags/{id}/orgs/{org_id}",
            delete(remove_feature_flag_org),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_response_serialization() {
        let response = FeatureFlagResponse {
            id: "test-id".into(),
            name: "dark-mode".into(),
            description: "Enable dark mode".into(),
            enabled: true,
            enabled_for_users: vec!["user1".into()],
            enabled_for_percentage: 50,
            enabled_for_orgs: vec!["org1".into()],
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"dark-mode\""));
        assert!(json.contains("\"enabled\":true"));
    }

    #[test]
    fn test_create_request_deserialization() {
        let json = r#"{"name": "new-feature", "description": "A new feature", "enabled": false, "enabled_for_percentage": 0}"#;
        let req: CreateFeatureFlagRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "new-feature");
        assert!(!req.enabled);
    }

    #[test]
    fn test_update_request_deserialization() {
        let json = r#"{"enabled": true, "enabled_for_percentage": 75}"#;
        let req: UpdateFeatureFlagRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.enabled, Some(true));
        assert_eq!(req.enabled_for_percentage, Some(75));
    }
}
