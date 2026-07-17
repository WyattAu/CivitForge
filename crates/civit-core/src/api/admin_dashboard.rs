#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_admin};
use axum::{
    Json,
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminDashboardWidgetResponse {
    pub id: String,
    pub widget_name: String,
    pub widget_config: serde_json::Value,
    pub position: i32,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminDashboardWidgetsResponse {
    pub widgets: Vec<AdminDashboardWidgetResponse>,
    pub total: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpsertWidgetRequest {
    pub widget_name: String,
    #[serde(default)]
    pub widget_config: serde_json::Value,
    #[serde(default)]
    pub position: i32,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWidgetPositionRequest {
    pub position: i32,
}

fn widget_to_response(widget: &civit_db::models::AdminDashboardConfig) -> AdminDashboardWidgetResponse {
    AdminDashboardWidgetResponse {
        id: widget.id.to_string(),
        widget_name: widget.widget_name.clone(),
        widget_config: widget.widget_config.clone(),
        position: widget.position,
        enabled: widget.enabled,
        created_at: widget.created_at.to_rfc3339(),
    }
}

pub async fn list_admin_dashboard_widgets(
    State(state): State<AppState>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.list_admin_dashboard_widgets().await {
        Ok(widgets) => {
            let response = AdminDashboardWidgetsResponse {
                widgets: widgets.iter().map(widget_to_response).collect(),
                total: widgets.len(),
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

pub async fn get_admin_dashboard_widget(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(widget_name): Path<String>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.get_admin_dashboard_widget(&widget_name).await {
        Ok(Some(widget)) => (StatusCode::OK, Json(widget_to_response(&widget))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "widget not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn upsert_admin_dashboard_widget(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpsertWidgetRequest>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    if req.widget_name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "widget_name is required"})),
        )
            .into_response();
    }

    match state
        .db
        .upsert_admin_dashboard_widget(
            &req.widget_name,
            &req.widget_config,
            req.position,
            req.enabled,
        )
        .await
    {
        Ok(widget) => (StatusCode::OK, Json(widget_to_response(&widget))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn delete_admin_dashboard_widget(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(widget_name): Path<String>,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.delete_admin_dashboard_widget(&widget_name).await {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub fn admin_dashboard_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/admin/dashboard",
            get(list_admin_dashboard_widgets).post(upsert_admin_dashboard_widget),
        )
        .route(
            "/api/v1/admin/dashboard/{widget_name}",
            get(get_admin_dashboard_widget).delete(delete_admin_dashboard_widget),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_response_serialization() {
        let response = AdminDashboardWidgetResponse {
            id: "test-id".into(),
            widget_name: "audit-log".into(),
            widget_config: serde_json::json!({"refresh_interval": 30}),
            position: 0,
            enabled: true,
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"audit-log\""));
        assert!(json.contains("\"enabled\":true"));
    }

    #[test]
    fn test_upsert_request_deserialization() {
        let json = r#"{"widget_name": "metrics", "widget_config": {"show": true}, "position": 5, "enabled": true}"#;
        let req: UpsertWidgetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.widget_name, "metrics");
        assert!(req.enabled);
        assert_eq!(req.position, 5);
    }
}
