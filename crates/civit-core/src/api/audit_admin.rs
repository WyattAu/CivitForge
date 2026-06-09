#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::{AuthUser, require_admin};
use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
};
use chrono::{DateTime, Utc};
use civit_shared::{ListResponse, Pagination};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct AuditEventResponse {
    pub id: i64,
    pub actor_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub outcome: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct AuditLogParams {
    pub actor_id: Option<String>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct AuditStatsResponse {
    pub total_events: i64,
    pub events_per_day: Vec<EventsPerDay>,
    pub top_actors: Vec<ActorStat>,
    pub top_actions: Vec<ActionStat>,
}

#[derive(Debug, Serialize)]
pub struct EventsPerDay {
    pub date: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct ActorStat {
    pub actor_id: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct ActionStat {
    pub action: String,
    pub count: i64,
}

pub fn audit_admin_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/audit-log", get(list_audit_log))
        .route("/api/v1/audit-log/stats", get(audit_stats))
        .route("/api/v1/audit-log/export", get(export_audit_log))
}

pub async fn list_audit_log(
    State(state): State<AppState>,
    Query(params): Query<AuditLogParams>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let actor_id = params
        .actor_id
        .as_deref()
        .and_then(|s| uuid::Uuid::parse_str(s).ok());
    let limit = params
        .per_page
        .map(|p| p.clamp(1, 100) as i64)
        .unwrap_or(50);
    let page = params.page.unwrap_or(1);
    let offset = ((page.saturating_sub(1)) * limit as u32) as i64;

    let since_dt = params
        .since
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));
    let until_dt = params
        .until
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    match state
        .db
        .query_audit_events_admin(
            actor_id,
            params.action.as_deref(),
            params.resource_type.as_deref(),
            since_dt,
            until_dt,
            limit,
            offset,
        )
        .await
    {
        Ok(events) => {
            let total = events.len() as u64;
            let out: Vec<AuditEventResponse> = events
                .into_iter()
                .map(
                    |(
                        id,
                        actor_id,
                        action,
                        resource_type,
                        resource_id,
                        ip_address,
                        user_agent,
                        outcome,
                        created_at,
                    )| {
                        AuditEventResponse {
                            id,
                            actor_id: actor_id.to_string(),
                            action,
                            resource_type,
                            resource_id: resource_id.map(|id| id.to_string()),
                            ip_address,
                            user_agent,
                            outcome,
                            created_at: created_at.to_rfc3339(),
                        }
                    },
                )
                .collect();
            let pag = Pagination {
                page,
                per_page: limit as u32,
                total,
                total_pages: if total == 0 {
                    1
                } else {
                    (total as u32).div_ceil(limit as u32)
                },
            };
            (
                StatusCode::OK,
                Json(ListResponse {
                    data: out,
                    pagination: pag,
                }),
            )
                .into_response()
        }
        Err(e) => (e.status_code(), Json(e.error_response())).into_response(),
    }
}

pub async fn audit_stats(State(state): State<AppState>, auth: AuthUser) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    match state.db.audit_event_stats().await {
        Ok((total, per_day, top_actors, top_actions)) => {
            let stats = AuditStatsResponse {
                total_events: total,
                events_per_day: per_day
                    .into_iter()
                    .map(|(date, count)| EventsPerDay {
                        date: date.to_string(),
                        count,
                    })
                    .collect(),
                top_actors: top_actors
                    .into_iter()
                    .map(|(actor_id, count)| ActorStat {
                        actor_id: actor_id.to_string(),
                        count,
                    })
                    .collect(),
                top_actions: top_actions
                    .into_iter()
                    .map(|(action, count)| ActionStat { action, count })
                    .collect(),
            };
            (StatusCode::OK, Json(stats)).into_response()
        }
        Err(e) => (e.status_code(), Json(e.error_response())).into_response(),
    }
}

pub async fn export_audit_log(
    State(state): State<AppState>,
    Query(params): Query<AuditLogParams>,
    auth: AuthUser,
) -> impl IntoResponse {
    if let Err(rejection) = require_admin(&auth) {
        return rejection.into_response();
    }

    let actor_id = params
        .actor_id
        .as_deref()
        .and_then(|s| uuid::Uuid::parse_str(s).ok());
    let limit = 10_000i64;
    let offset = 0i64;

    let since_dt = params
        .since
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));
    let until_dt = params
        .until
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    match state
        .db
        .query_audit_events_admin(
            actor_id,
            params.action.as_deref(),
            params.resource_type.as_deref(),
            since_dt,
            until_dt,
            limit,
            offset,
        )
        .await
    {
        Ok(events) => {
            let mut csv = String::from(
                "id,actor_id,action,resource_type,resource_id,ip_address,user_agent,outcome,created_at\n",
            );
            for (
                id,
                actor_id,
                action,
                resource_type,
                resource_id,
                ip_address,
                user_agent,
                outcome,
                created_at,
            ) in &events
            {
                csv.push_str(&format!(
                    "{},\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                    id,
                    actor_id,
                    action,
                    resource_type,
                    resource_id.map(|u| u.to_string()).unwrap_or_default(),
                    ip_address.as_deref().unwrap_or(""),
                    user_agent.as_deref().unwrap_or(""),
                    outcome,
                    created_at.to_rfc3339(),
                ));
            }
            (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, "text/csv; charset=utf-8")],
                csv,
            )
                .into_response()
        }
        Err(e) => (e.status_code(), Json(e.error_response())).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_response_serialization() {
        let resp = AuditEventResponse {
            id: 1,
            actor_id: "00000000-0000-0000-0000-000000000000".into(),
            action: "login".into(),
            resource_type: "user".into(),
            resource_id: None,
            ip_address: Some("127.0.0.1".into()),
            user_agent: Some("curl/7.68".into()),
            outcome: "success".into(),
            created_at: "2025-01-01T00:00:00+00:00".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"action\":\"login\""));
        assert!(json.contains("\"outcome\":\"success\""));
    }

    #[test]
    fn test_audit_log_params_defaults() {
        let params = AuditLogParams {
            actor_id: None,
            action: None,
            resource_type: None,
            since: None,
            until: None,
            page: None,
            per_page: None,
        };
        assert!(params.actor_id.is_none());
        assert!(params.action.is_none());
        assert!(params.since.is_none());
    }

    #[test]
    fn test_audit_log_params_parse() {
        let params = AuditLogParams {
            actor_id: Some("00000000-0000-0000-0000-000000000000".into()),
            action: Some("push".into()),
            resource_type: Some("repo".into()),
            since: Some("2025-01-01T00:00:00Z".into()),
            until: Some("2025-12-31T23:59:59Z".into()),
            page: Some(2),
            per_page: Some(25),
        };
        assert_eq!(params.page, Some(2));
        assert_eq!(params.per_page, Some(25));
    }

    #[test]
    fn test_audit_stats_response_serialization() {
        let stats = AuditStatsResponse {
            total_events: 100,
            events_per_day: vec![EventsPerDay {
                date: "2025-06-01".into(),
                count: 42,
            }],
            top_actors: vec![ActorStat {
                actor_id: "00000000-0000-0000-0000-000000000001".into(),
                count: 10,
            }],
            top_actions: vec![ActionStat {
                action: "login".into(),
                count: 50,
            }],
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"total_events\":100"));
        assert!(json.contains("\"action\":\"login\""));
    }

    #[test]
    fn test_events_per_day_serialization() {
        let epd = EventsPerDay {
            date: "2025-06-01".into(),
            count: 42,
        };
        let json = serde_json::to_string(&epd).unwrap();
        assert!(json.contains("\"date\":\"2025-06-01\""));
        assert!(json.contains("\"count\":42"));
    }

    #[test]
    fn test_actor_stat_serialization() {
        let stat = ActorStat {
            actor_id: "abc".into(),
            count: 5,
        };
        let json = serde_json::to_string(&stat).unwrap();
        assert!(json.contains("\"actor_id\":\"abc\""));
        assert!(json.contains("\"count\":5"));
    }

    #[test]
    fn test_action_stat_serialization() {
        let stat = ActionStat {
            action: "create".into(),
            count: 3,
        };
        let json = serde_json::to_string(&stat).unwrap();
        assert!(json.contains("\"action\":\"create\""));
        assert!(json.contains("\"count\":3"));
    }

    #[test]
    fn test_audit_admin_routes_compile() {
        let router = audit_admin_routes();
        let _ = router;
    }
}
