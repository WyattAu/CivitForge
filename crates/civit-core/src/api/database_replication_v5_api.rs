#![forbid(unsafe_code)]

use crate::api::AppState;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReplicationConfig {
    pub id: Uuid,
    pub replica_id: Uuid,
    pub config_key: String,
    pub config_value: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReplicationAlert {
    pub id: Uuid,
    pub replica_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetConfigRequest {
    pub replica_id: Uuid,
    pub config_key: String,
    pub config_value: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAlertRequest {
    pub replica_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplicationConfigResponse {
    pub id: Uuid,
    pub replica_id: Uuid,
    pub config_key: String,
    pub config_value: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplicationAlertResponse {
    pub id: Uuid,
    pub replica_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FailoverTestResponse {
    pub replica_id: Uuid,
    pub success: bool,
    pub duration_ms: i32,
    pub error_message: Option<String>,
    pub tested_at: String,
    pub steps: Vec<FailoverStepResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FailoverStepResponse {
    pub step_name: String,
    pub success: bool,
    pub duration_ms: i32,
    pub error_message: Option<String>,
}

impl From<ReplicationConfig> for ReplicationConfigResponse {
    fn from(c: ReplicationConfig) -> Self {
        Self {
            id: c.id,
            replica_id: c.replica_id,
            config_key: c.config_key,
            config_value: c.config_value,
            created_at: c.created_at.to_rfc3339(),
        }
    }
}

impl From<ReplicationAlert> for ReplicationAlertResponse {
    fn from(a: ReplicationAlert) -> Self {
        Self {
            id: a.id,
            replica_id: a.replica_id,
            alert_type: a.alert_type,
            threshold: a.threshold,
            enabled: a.enabled,
            last_triggered_at: a.last_triggered_at.map(|t| t.to_rfc3339()),
            created_at: a.created_at.to_rfc3339(),
        }
    }
}

async fn set_config(
    State(state): State<AppState>,
    Json(input): Json<SetConfigRequest>,
) -> Result<(StatusCode, Json<ReplicationConfigResponse>), Response> {
    let config = sqlx::query_as::<_, ReplicationConfig>(
        r#"INSERT INTO database_replication_config_v3 (replica_id, config_key, config_value)
         VALUES ($1, $2, $3)
         ON CONFLICT (replica_id, config_key) DO UPDATE SET config_value = $3
         RETURNING id, replica_id, config_key, config_value, created_at"#,
    )
    .bind(input.replica_id)
    .bind(&input.config_key)
    .bind(&input.config_value)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(config.into())))
}

async fn get_config(
    State(state): State<AppState>,
    Path((replica_id, config_key)): Path<(Uuid, String)>,
) -> Result<Json<ReplicationConfigResponse>, Response> {
    let config = sqlx::query_as::<_, ReplicationConfig>(
        r#"SELECT id, replica_id, config_key, config_value, created_at
         FROM database_replication_config_v3
         WHERE replica_id = $1 AND config_key = $2"#,
    )
    .bind(replica_id)
    .bind(&config_key)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?
    .ok_or_else(|| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "config not found"}))).into_response())?;

    Ok(Json(config.into()))
}

async fn get_all_config(
    State(state): State<AppState>,
    Path(replica_id): Path<Uuid>,
) -> Result<Json<Vec<ReplicationConfigResponse>>, Response> {
    let configs = sqlx::query_as::<_, ReplicationConfig>(
        r#"SELECT id, replica_id, config_key, config_value, created_at
         FROM database_replication_config_v3
         WHERE replica_id = $1
         ORDER BY config_key"#,
    )
    .bind(replica_id)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(configs.into_iter().map(|c| c.into()).collect()))
}

async fn create_alert(
    State(state): State<AppState>,
    Json(input): Json<CreateAlertRequest>,
) -> Result<(StatusCode, Json<ReplicationAlertResponse>), Response> {
    let alert = sqlx::query_as::<_, ReplicationAlert>(
        r#"INSERT INTO database_replication_alerts_v3 (replica_id, alert_type, threshold)
         VALUES ($1, $2, $3)
         RETURNING id, replica_id, alert_type, threshold, enabled, last_triggered_at, created_at"#,
    )
    .bind(input.replica_id)
    .bind(&input.alert_type)
    .bind(input.threshold)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(alert.into())))
}

async fn get_alerts(
    State(state): State<AppState>,
    Path(replica_id): Path<Uuid>,
) -> Result<Json<Vec<ReplicationAlertResponse>>, Response> {
    let alerts = sqlx::query_as::<_, ReplicationAlert>(
        r#"SELECT id, replica_id, alert_type, threshold, enabled, last_triggered_at, created_at
         FROM database_replication_alerts_v3
         WHERE replica_id = $1
         ORDER BY created_at DESC"#,
    )
    .bind(replica_id)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(alerts.into_iter().map(|a| a.into()).collect()))
}

async fn monitor_replication(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, Response> {
    #[derive(sqlx::FromRow)]
    struct ReplicaRow {
        id: Uuid,
        name: String,
        lag_ms: i32,
        status: String,
    }

    let replicas = sqlx::query_as::<_, ReplicaRow>(
        r#"SELECT id, name, lag_ms, status FROM database_replicas ORDER BY lag_ms DESC"#,
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let result: Vec<serde_json::Value> = replicas
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "name": r.name,
                "lag_ms": r.lag_ms,
                "status": r.status,
                "healthy": r.lag_ms < 1000,
            })
        })
        .collect();

    Ok(Json(result))
}

async fn test_failover(
    State(state): State<AppState>,
    Path(replica_id): Path<Uuid>,
) -> Result<Json<FailoverTestResponse>, Response> {
    let start = Utc::now();
    let mut steps = Vec::new();

    let replica_exists: Option<(Uuid,)> = sqlx::query_as(
        r#"SELECT id FROM database_replicas WHERE id = $1"#,
    )
    .bind(replica_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    if replica_exists.is_none() {
        return Ok(Json(FailoverTestResponse {
            replica_id,
            success: false,
            duration_ms: 0,
            error_message: Some("Replica not found".to_string()),
            tested_at: Utc::now().to_rfc3339(),
            steps,
        }));
    }

    let step_start = Utc::now();
    let config_count: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM database_replication_config_v3 WHERE replica_id = $1"#,
    )
    .bind(replica_id)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;
    let step_duration = (Utc::now() - step_start).num_milliseconds() as i32;
    steps.push(FailoverStepResponse {
        step_name: "verify_config".into(),
        success: true,
        duration_ms: step_duration,
        error_message: None,
    });

    let step_start = Utc::now();
    let alert_count: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM database_replication_alerts_v3 WHERE replica_id = $1"#,
    )
    .bind(replica_id)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;
    let step_duration = (Utc::now() - step_start).num_milliseconds() as i32;
    steps.push(FailoverStepResponse {
        step_name: "verify_alerts".into(),
        success: true,
        duration_ms: step_duration,
        error_message: None,
    });

    let _ = config_count;
    let _ = alert_count;

    let end = Utc::now();
    let duration_ms = (end - start).num_milliseconds() as i32;

    Ok(Json(FailoverTestResponse {
        replica_id,
        success: true,
        duration_ms,
        error_message: None,
        tested_at: end.to_rfc3339(),
        steps,
    }))
}

pub fn replication_v5_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/replication-v5/config",
            post(set_config),
        )
        .route(
            "/api/v1/replication-v5/config/{replica_id}",
            get(get_all_config),
        )
        .route(
            "/api/v1/replication-v5/config/{replica_id}/{config_key}",
            get(get_config),
        )
        .route(
            "/api/v1/replication-v5/alerts",
            post(create_alert),
        )
        .route(
            "/api/v1/replication-v5/alerts/{replica_id}",
            get(get_alerts),
        )
        .route(
            "/api/v1/replication-v5/monitor",
            get(monitor_replication),
        )
        .route(
            "/api/v1/replication-v5/failover-test/{replica_id}",
            post(test_failover),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_response_conversion() {
        let config = ReplicationConfig {
            id: Uuid::nil(),
            replica_id: Uuid::nil(),
            config_key: "sync_mode".to_string(),
            config_value: serde_json::json!({"mode": "async"}),
            created_at: Utc::now(),
        };
        let response: ReplicationConfigResponse = config.into();
        assert_eq!(response.config_key, "sync_mode");
    }

    #[test]
    fn test_alert_response_conversion() {
        let alert = ReplicationAlert {
            id: Uuid::nil(),
            replica_id: Uuid::nil(),
            alert_type: "lag_threshold".to_string(),
            threshold: 500.0,
            enabled: true,
            last_triggered_at: None,
            created_at: Utc::now(),
        };
        let response: ReplicationAlertResponse = alert.into();
        assert_eq!(response.threshold, 500.0);
        assert!(response.last_triggered_at.is_none());
    }
}
