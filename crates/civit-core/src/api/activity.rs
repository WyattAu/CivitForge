#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::OptionalAuthUser;
use crate::error::CoreError;
use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct ActivityResponse {
    pub id: i64,
    pub actor_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub repo_id: Option<String>,
    pub org_id: Option<String>,
    pub description: String,
    pub metadata: serde_json::Value,
    pub created_at: String,
}

impl From<crate::db::ActivityEvent> for ActivityResponse {
    fn from(e: crate::db::ActivityEvent) -> Self {
        Self {
            id: e.id,
            actor_id: e.actor_id.to_string(),
            action: e.action,
            resource_type: e.resource_type,
            resource_id: e.resource_id.map(|id| id.to_string()),
            repo_id: e.repo_id.map(|id| id.to_string()),
            org_id: e.org_id.map(|id| id.to_string()),
            description: e.description,
            metadata: e.metadata,
            created_at: e.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ActivityQueryParams {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default = "default_offset")]
    pub offset: i64,
    pub repo_id: Option<String>,
    pub org_id: Option<String>,
}

fn default_limit() -> i64 {
    50
}
fn default_offset() -> i64 {
    0
}

pub fn activity_routes() -> Router<AppState> {
    Router::new().route("/api/v1/activity", get(list_activity))
}

pub async fn list_activity(
    State(state): State<AppState>,
    Query(params): Query<ActivityQueryParams>,
    _auth: OptionalAuthUser,
) -> impl IntoResponse {
    let repo_id = params.repo_id.and_then(|s| uuid::Uuid::parse_str(&s).ok());
    let org_id = params.org_id.and_then(|s| uuid::Uuid::parse_str(&s).ok());

    match state
        .db
        .list_activity_events(repo_id, org_id, params.limit, params.offset)
        .await
    {
        Ok(events) => {
            let resp: Vec<ActivityResponse> =
                events.into_iter().map(ActivityResponse::from).collect();
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => (e.status_code(), Json(e.error_response())).into_response(),
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn record_activity(
    state: &AppState,
    actor_id: uuid::Uuid,
    action: &str,
    resource_type: &str,
    resource_id: Option<uuid::Uuid>,
    repo_id: Option<uuid::Uuid>,
    org_id: Option<uuid::Uuid>,
    description: &str,
    metadata: serde_json::Value,
) -> Result<ActivityResponse, CoreError> {
    let event = state
        .db
        .record_activity_event(
            actor_id,
            action,
            resource_type,
            resource_id,
            repo_id,
            org_id,
            description,
            metadata,
        )
        .await?;
    Ok(ActivityResponse::from(event))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_response_from_event() {
        let event = crate::db::ActivityEvent {
            id: 1,
            actor_id: uuid::Uuid::nil(),
            action: "push".into(),
            resource_type: "repo".into(),
            resource_id: None,
            repo_id: None,
            org_id: None,
            description: "Pushed 3 commits".into(),
            metadata: serde_json::Value::Null,
            created_at: chrono::Utc::now(),
        };
        let resp = ActivityResponse::from(event);
        assert_eq!(resp.action, "push");
        assert_eq!(resp.id, 1);
    }

    #[test]
    fn test_activity_query_params_defaults() {
        let params = ActivityQueryParams {
            limit: 50,
            offset: 0,
            repo_id: None,
            org_id: None,
        };
        assert_eq!(params.limit, 50);
        assert_eq!(params.offset, 0);
    }

    #[test]
    fn test_activity_query_params_parse() {
        let json = r#"{"limit":10,"offset":20,"repo_id":"00000000-0000-0000-0000-000000000000"}"#;
        let params: ActivityQueryParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.limit, 10);
        assert!(params.repo_id.is_some());
    }

    #[test]
    fn test_activity_response_serialization() {
        let resp = ActivityResponse {
            id: 1,
            actor_id: "00000000-0000-0000-0000-000000000000".into(),
            action: "push".into(),
            resource_type: "repo".into(),
            resource_id: None,
            repo_id: None,
            org_id: None,
            description: "Pushed to main".into(),
            metadata: serde_json::json!({"commits": 3}),
            created_at: "2025-01-01T00:00:00+00:00".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"action\":\"push\""));
        assert!(json.contains("\"commits\":3"));
    }

    #[test]
    fn test_activity_response_with_ids() {
        let resp = ActivityResponse {
            id: 2,
            actor_id: uuid::Uuid::new_v4().to_string(),
            action: "open_issue".into(),
            resource_type: "issue".into(),
            resource_id: Some(uuid::Uuid::new_v4().to_string()),
            repo_id: Some(uuid::Uuid::new_v4().to_string()),
            org_id: None,
            description: "Opened new issue".into(),
            metadata: serde_json::Value::Null,
            created_at: "2025-06-04T00:00:00+00:00".into(),
        };
        assert!(resp.resource_id.is_some());
        assert!(resp.repo_id.is_some());
        assert!(resp.org_id.is_none());
    }
}
