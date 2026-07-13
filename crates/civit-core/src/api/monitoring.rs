#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::{CoreError, ErrorResponse};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, patch, post},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct AlertResponse {
    pub id: String,
    pub repo_id: String,
    pub alert_type: String,
    pub condition: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAlertRequest {
    pub alert_type: String,
    pub condition: String,
    pub threshold: f64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAlertRequest {
    pub alert_type: Option<String>,
    pub condition: Option<String>,
    pub threshold: Option<f64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct IncidentResponse {
    pub id: String,
    pub alert_id: String,
    pub severity: String,
    pub message: String,
    pub status: String,
    pub resolved_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateIncidentRequest {
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ListIncidentsParams {
    pub status: Option<String>,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}
fn default_per_page() -> u32 {
    20
}

async fn resolve_repo_id(
    state: &AppState,
    owner: &str,
    name: &str,
) -> Result<Uuid, (StatusCode, Json<ErrorResponse>)> {
    let owner_uuid = if let Ok(id) = Uuid::parse_str(owner) {
        id
    } else if let Ok(user) = state.db.get_user_by_username(owner).await {
        user.id
    } else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("user not found".into()).error_response()),
        ));
    };

    match state.db.get_repo_by_owner_name(owner_uuid, name).await {
        Ok(repo) => Ok(repo.id),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("repository not found".into()).error_response()),
        )),
    }
}

// --- Alert CRUD ---

pub async fn create_alert(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Json(req): Json<CreateAlertRequest>,
) -> impl IntoResponse {
    let repo_id = match resolve_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp.into_response(),
    };

    let valid_types = ["error_rate", "latency", "cpu_usage", "memory_usage", "disk_usage"];
    if !valid_types.contains(&req.alert_type.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                CoreError::BadRequest(format!(
                    "invalid alert_type: must be one of {}",
                    valid_types.join(", ")
                ))
                .error_response(),
            ),
        )
            .into_response();
    }

    match state
        .db
        .create_monitoring_alert(repo_id, &req.alert_type, &req.condition, req.threshold)
        .await
    {
        Ok(alert) => (
            StatusCode::CREATED,
            Json(AlertResponse {
                id: alert.id.to_string(),
                repo_id: alert.repo_id.to_string(),
                alert_type: alert.alert_type,
                condition: alert.condition,
                threshold: alert.threshold,
                enabled: alert.enabled,
                last_triggered_at: alert.last_triggered_at.map(|t| t.to_rfc3339()),
                created_at: alert.created_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_alerts(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let repo_id = match resolve_repo_id(&state, &owner, &name).await {
        Ok(id) => id,
        Err(resp) => return resp.into_response(),
    };

    match state.db.list_monitoring_alerts(repo_id).await {
        Ok(alerts) => {
            let resp: Vec<AlertResponse> = alerts
                .into_iter()
                .map(|a| AlertResponse {
                    id: a.id.to_string(),
                    repo_id: a.repo_id.to_string(),
                    alert_type: a.alert_type,
                    condition: a.condition,
                    threshold: a.threshold,
                    enabled: a.enabled,
                    last_triggered_at: a.last_triggered_at.map(|t| t.to_rfc3339()),
                    created_at: a.created_at.to_rfc3339(),
                })
                .collect();
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn get_alert(
    State(state): State<AppState>,
    Path((_owner, _name, alert_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let aid = match Uuid::parse_str(&alert_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid alert id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.get_monitoring_alert(aid).await {
        Ok(a) => (
            StatusCode::OK,
            Json(AlertResponse {
                id: a.id.to_string(),
                repo_id: a.repo_id.to_string(),
                alert_type: a.alert_type,
                condition: a.condition,
                threshold: a.threshold,
                enabled: a.enabled,
                last_triggered_at: a.last_triggered_at.map(|t| t.to_rfc3339()),
                created_at: a.created_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("alert not found".into()).error_response()),
        )
            .into_response(),
    }
}

pub async fn update_alert(
    State(state): State<AppState>,
    Path((_owner, _name, alert_id)): Path<(String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<UpdateAlertRequest>,
) -> impl IntoResponse {
    let aid = match Uuid::parse_str(&alert_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid alert id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state
        .db
        .update_monitoring_alert(
            aid,
            req.alert_type.as_deref(),
            req.condition.as_deref(),
            req.threshold,
            req.enabled,
        )
        .await
    {
        Ok(a) => (
            StatusCode::OK,
            Json(AlertResponse {
                id: a.id.to_string(),
                repo_id: a.repo_id.to_string(),
                alert_type: a.alert_type,
                condition: a.condition,
                threshold: a.threshold,
                enabled: a.enabled,
                last_triggered_at: a.last_triggered_at.map(|t| t.to_rfc3339()),
                created_at: a.created_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn delete_alert(
    State(state): State<AppState>,
    Path((_owner, _name, alert_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let aid = match Uuid::parse_str(&alert_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid alert id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.delete_monitoring_alert(aid).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "deleted"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

// --- Incidents ---

pub async fn create_incident(
    State(state): State<AppState>,
    Path((_owner, _name, alert_id)): Path<(String, String, String)>,
    _auth: AuthUser,
    Json(req): Json<CreateIncidentRequest>,
) -> impl IntoResponse {
    let aid = match Uuid::parse_str(&alert_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid alert id".into()).error_response()),
            )
                .into_response();
        }
    };

    let valid_severities = ["low", "medium", "high", "critical"];
    if !valid_severities.contains(&req.severity.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                CoreError::BadRequest(format!(
                    "invalid severity: must be one of {}",
                    valid_severities.join(", ")
                ))
                .error_response(),
            ),
        )
            .into_response();
    }

    match state
        .db
        .create_monitoring_incident(aid, &req.severity, &req.message)
        .await
    {
        Ok(incident) => {
            let _ = state.db.trigger_monitoring_alert(aid).await;
            (
                StatusCode::CREATED,
                Json(IncidentResponse {
                    id: incident.id.to_string(),
                    alert_id: incident.alert_id.to_string(),
                    severity: incident.severity,
                    message: incident.message,
                    status: incident.status,
                    resolved_at: incident.resolved_at.map(|t| t.to_rfc3339()),
                    created_at: incident.created_at.to_rfc3339(),
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn list_incidents(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    _auth: AuthUser,
    Query(params): Query<ListIncidentsParams>,
) -> impl IntoResponse {
    let repo_id = match resolve_repo_id(&state, &owner, &name).await {
        Ok(id) => Some(id),
        Err(_) => None,
    };

    let offset = (params.page.saturating_sub(1) * params.per_page) as i64;

    match state
        .db
        .list_monitoring_incidents(repo_id, params.status.as_deref(), params.per_page as i64, offset)
        .await
    {
        Ok(incidents) => {
            let resp: Vec<IncidentResponse> = incidents
                .into_iter()
                .map(|i| IncidentResponse {
                    id: i.id.to_string(),
                    alert_id: i.alert_id.to_string(),
                    severity: i.severity,
                    message: i.message,
                    status: i.status,
                    resolved_at: i.resolved_at.map(|t| t.to_rfc3339()),
                    created_at: i.created_at.to_rfc3339(),
                })
                .collect();
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub async fn resolve_incident(
    State(state): State<AppState>,
    Path((_owner, _name, incident_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let iid = match Uuid::parse_str(&incident_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid incident id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.resolve_monitoring_incident(iid).await {
        Ok(incident) => (
            StatusCode::OK,
            Json(IncidentResponse {
                id: incident.id.to_string(),
                alert_id: incident.alert_id.to_string(),
                severity: incident.severity,
                message: incident.message,
                status: incident.status,
                resolved_at: incident.resolved_at.map(|t| t.to_rfc3339()),
                created_at: incident.created_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("returned no rows") {
                (
                    StatusCode::NOT_FOUND,
                    Json(CoreError::NotFound("incident not found".into()).error_response()),
                )
                    .into_response()
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(CoreError::Database(msg).error_response()),
                )
                    .into_response()
            }
        }
    }
}

pub async fn get_incident_timeline(
    State(state): State<AppState>,
    Path((_owner, _name, alert_id)): Path<(String, String, String)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let aid = match Uuid::parse_str(&alert_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(CoreError::BadRequest("invalid alert id".into()).error_response()),
            )
                .into_response();
        }
    };

    match state.db.get_incident_timeline(aid).await {
        Ok(incidents) => {
            let resp: Vec<IncidentResponse> = incidents
                .into_iter()
                .map(|i| IncidentResponse {
                    id: i.id.to_string(),
                    alert_id: i.alert_id.to_string(),
                    severity: i.severity,
                    message: i.message,
                    status: i.status,
                    resolved_at: i.resolved_at.map(|t| t.to_rfc3339()),
                    created_at: i.created_at.to_rfc3339(),
                })
                .collect();
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CoreError::Database(e.to_string()).error_response()),
        )
            .into_response(),
    }
}

pub fn monitoring_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/monitoring/alerts",
            get(list_alerts).post(create_alert),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/monitoring/alerts/{alert_id}",
            get(get_alert).patch(update_alert).delete(delete_alert),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/monitoring/alerts/{alert_id}/incidents",
            get(list_incidents).post(create_incident),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/monitoring/incidents/{incident_id}/resolve",
            post(resolve_incident),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/monitoring/alerts/{alert_id}/timeline",
            get(get_incident_timeline),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_response_serializes() {
        let resp = AlertResponse {
            id: "00000000-0000-0000-0000-000000000001".into(),
            repo_id: "00000000-0000-0000-0000-000000000002".into(),
            alert_type: "error_rate".into(),
            condition: ">".into(),
            threshold: 0.05,
            enabled: true,
            last_triggered_at: None,
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("error_rate"));
        assert!(json.contains("0.05"));
    }

    #[test]
    fn test_incident_response_serializes() {
        let resp = IncidentResponse {
            id: "00000000-0000-0000-0000-000000000001".into(),
            alert_id: "00000000-0000-0000-0000-000000000002".into(),
            severity: "critical".into(),
            message: "Error rate exceeded threshold".into(),
            status: "open".into(),
            resolved_at: None,
            created_at: "2024-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("critical"));
        assert!(json.contains("open"));
    }

    #[test]
    fn test_create_alert_request() {
        let req: CreateAlertRequest =
            serde_json::from_str(r#"{"alert_type": "error_rate", "condition": ">", "threshold": 0.05}"#)
                .unwrap();
        assert_eq!(req.alert_type, "error_rate");
        assert_eq!(req.threshold, 0.05);
    }

    #[test]
    fn test_create_incident_request() {
        let req: CreateIncidentRequest =
            serde_json::from_str(r#"{"severity": "high", "message": "CPU spike"}"#).unwrap();
        assert_eq!(req.severity, "high");
        assert_eq!(req.message, "CPU spike");
    }
}
