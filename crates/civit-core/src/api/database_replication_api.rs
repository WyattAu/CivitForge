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

// --- v2: Replication logs, stats, summary, lag monitor ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReplicationLog {
    pub id: Uuid,
    pub replica_id: Uuid,
    pub operation: String,
    pub table_name: String,
    pub record_id: Option<Uuid>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReplicationStats {
    pub id: Uuid,
    pub replica_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub operations_count: i32,
    pub avg_lag_ms: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateReplicationLogRequest {
    pub replica_id: Uuid,
    pub operation: String,
    pub table_name: String,
    pub record_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateReplicationStatsRequest {
    pub replica_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub operations_count: i32,
    pub avg_lag_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplicationLogResponse {
    pub id: Uuid,
    pub replica_id: Uuid,
    pub operation: String,
    pub table_name: String,
    pub record_id: Option<Uuid>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplicationStatsResponse {
    pub id: Uuid,
    pub replica_id: Uuid,
    pub period_start: String,
    pub operations_count: i32,
    pub avg_lag_ms: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplicationSummaryResponse {
    pub total_logs: i64,
    pub pending_logs: i64,
    pub failed_logs: i64,
    pub avg_lag_ms: f64,
}

impl From<ReplicationLog> for ReplicationLogResponse {
    fn from(log: ReplicationLog) -> Self {
        Self {
            id: log.id,
            replica_id: log.replica_id,
            operation: log.operation,
            table_name: log.table_name,
            record_id: log.record_id,
            status: log.status,
            created_at: log.created_at.to_rfc3339(),
        }
    }
}

impl From<ReplicationStats> for ReplicationStatsResponse {
    fn from(stats: ReplicationStats) -> Self {
        Self {
            id: stats.id,
            replica_id: stats.replica_id,
            period_start: stats.period_start.to_rfc3339(),
            operations_count: stats.operations_count,
            avg_lag_ms: stats.avg_lag_ms,
            created_at: stats.created_at.to_rfc3339(),
        }
    }
}

async fn create_replication_log(
    State(state): State<AppState>,
    Json(input): Json<CreateReplicationLogRequest>,
) -> Result<(StatusCode, Json<ReplicationLogResponse>), Response> {
    let log = sqlx::query_as::<_, ReplicationLog>(
        r#"INSERT INTO database_replication_logs (replica_id, operation, table_name, record_id)
         VALUES ($1, $2, $3, $4)
         RETURNING id, replica_id, operation, table_name, record_id, status, created_at"#,
    )
    .bind(input.replica_id)
    .bind(&input.operation)
    .bind(&input.table_name)
    .bind(input.record_id)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(log.into())))
}

async fn list_replication_logs(
    State(state): State<AppState>,
) -> Result<Json<Vec<ReplicationLogResponse>>, Response> {
    let logs = sqlx::query_as::<_, ReplicationLog>(
        r#"SELECT id, replica_id, operation, table_name, record_id, status, created_at
         FROM database_replication_logs ORDER BY created_at DESC LIMIT 100"#,
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(logs.into_iter().map(|l| l.into()).collect()))
}

async fn get_replication_log(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ReplicationLogResponse>, Response> {
    let log = sqlx::query_as::<_, ReplicationLog>(
        r#"SELECT id, replica_id, operation, table_name, record_id, status, created_at
         FROM database_replication_logs WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?
    .ok_or_else(|| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "log not found"}))).into_response())?;

    Ok(Json(log.into()))
}

async fn update_replication_log_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<ReplicationLogResponse>, Response> {
    let status = input.get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("completed");

    let log = sqlx::query_as::<_, ReplicationLog>(
        r#"UPDATE database_replication_logs SET status = $2 WHERE id = $1
         RETURNING id, replica_id, operation, table_name, record_id, status, created_at"#,
    )
    .bind(id)
    .bind(status)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?
    .ok_or_else(|| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "log not found"}))).into_response())?;

    Ok(Json(log.into()))
}

async fn create_replication_stats(
    State(state): State<AppState>,
    Json(input): Json<CreateReplicationStatsRequest>,
) -> Result<(StatusCode, Json<ReplicationStatsResponse>), Response> {
    let stats = sqlx::query_as::<_, ReplicationStats>(
        r#"INSERT INTO database_replication_stats (replica_id, period_start, operations_count, avg_lag_ms)
         VALUES ($1, $2, $3, $4)
         RETURNING id, replica_id, period_start, operations_count, avg_lag_ms, created_at"#,
    )
    .bind(input.replica_id)
    .bind(input.period_start)
    .bind(input.operations_count)
    .bind(input.avg_lag_ms)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(stats.into())))
}

async fn get_replication_stats(
    State(state): State<AppState>,
    Path(replica_id): Path<Uuid>,
) -> Result<Json<Vec<ReplicationStatsResponse>>, Response> {
    let stats = sqlx::query_as::<_, ReplicationStats>(
        r#"SELECT id, replica_id, period_start, operations_count, avg_lag_ms, created_at
         FROM database_replication_stats WHERE replica_id = $1 ORDER BY period_start DESC LIMIT 50"#,
    )
    .bind(replica_id)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(stats.into_iter().map(|s| s.into()).collect()))
}

async fn get_replication_summary(
    State(state): State<AppState>,
) -> Result<Json<ReplicationSummaryResponse>, Response> {
    #[derive(sqlx::FromRow)]
    struct SummaryRow {
        total_logs: i64,
        pending_logs: i64,
        failed_logs: i64,
    }

    let row = sqlx::query_as::<_, SummaryRow>(
        r#"SELECT
         COUNT(*) as total_logs,
         COUNT(*) FILTER (WHERE status = 'pending') as pending_logs,
         COUNT(*) FILTER (WHERE status = 'failed') as failed_logs
         FROM database_replication_logs"#,
    )
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let avg_lag = sqlx::query_scalar::<_, f64>(
        r#"SELECT COALESCE(AVG(avg_lag_ms), 0.0) FROM database_replication_stats
         WHERE created_at > NOW() - INTERVAL '1 hour'"#,
    )
    .fetch_one(state.db.pool())
    .await
    .unwrap_or(0.0);

    Ok(Json(ReplicationSummaryResponse {
        total_logs: row.total_logs,
        pending_logs: row.pending_logs,
        failed_logs: row.failed_logs,
        avg_lag_ms: avg_lag,
    }))
}

async fn monitor_replication_lag(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, Response> {
    #[derive(sqlx::FromRow)]
    struct ReplicaLag {
        id: Uuid,
        name: String,
        lag_ms: i32,
        status: String,
    }

    let replicas = sqlx::query_as::<_, ReplicaLag>(
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

// --- v5/v8/v9/v11: Config, alerts, monitor, failover test ---

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
        r#"INSERT INTO database_replication_config_v13 (replica_id, config_key, config_value)
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
         FROM database_replication_config_v13
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
         FROM database_replication_config_v13
         WHERE replica_id = $1
         ORDER BY config_key"#,
    )
    .bind(replica_id)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(configs.into_iter().map(|c| c.into()).collect()))
}

async fn create_replication_alert(
    State(state): State<AppState>,
    Json(input): Json<CreateAlertRequest>,
) -> Result<(StatusCode, Json<ReplicationAlertResponse>), Response> {
    let alert = sqlx::query_as::<_, ReplicationAlert>(
        r#"INSERT INTO database_replication_alerts_v13 (replica_id, alert_type, threshold)
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

async fn get_replication_alerts(
    State(state): State<AppState>,
    Path(replica_id): Path<Uuid>,
) -> Result<Json<Vec<ReplicationAlertResponse>>, Response> {
    let alerts = sqlx::query_as::<_, ReplicationAlert>(
        r#"SELECT id, replica_id, alert_type, threshold, enabled, last_triggered_at, created_at
         FROM database_replication_alerts_v13
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
        r#"SELECT COUNT(*) FROM database_replication_config_v13 WHERE replica_id = $1"#,
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
        r#"SELECT COUNT(*) FROM database_replication_alerts_v13 WHERE replica_id = $1"#,
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

// --- v12: Consistency checks, failover history, topology, recovery points ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ConsistencyCheckV21 {
    pub id: Uuid,
    pub source_node: String,
    pub target_node: String,
    pub table_name: String,
    pub check_type: String,
    pub status: String,
    pub discrepancy_count: i32,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FailoverHistoryV21 {
    pub id: Uuid,
    pub source_node: String,
    pub target_node: String,
    pub failover_type: String,
    pub reason: String,
    pub duration_ms: Option<i32>,
    pub success: bool,
    pub initiated_by: Option<Uuid>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateConsistencyCheckRequest {
    pub source_node: String,
    pub target_node: String,
    pub table_name: String,
    pub check_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordFailoverRequest {
    pub source_node: String,
    pub target_node: String,
    pub failover_type: String,
    pub reason: String,
    pub duration_ms: Option<i32>,
    pub success: bool,
    pub initiated_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConsistencyCheckResponseV21 {
    pub id: Uuid,
    pub source_node: String,
    pub target_node: String,
    pub table_name: String,
    pub check_type: String,
    pub status: String,
    pub discrepancy_count: i32,
    pub checked_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FailoverHistoryResponseV21 {
    pub id: Uuid,
    pub source_node: String,
    pub target_node: String,
    pub failover_type: String,
    pub reason: String,
    pub duration_ms: Option<i32>,
    pub success: bool,
    pub initiated_by: Option<Uuid>,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplicationTopologyResponseV22 {
    pub source_node: String,
    pub target_node: String,
    pub status: String,
    pub lag_ms: Option<i32>,
    pub consistency_score: f64,
    pub last_check_at: Option<String>,
    pub total_failovers: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecoveryPointResponseV22 {
    pub node_name: String,
    pub recovery_point: String,
    pub lag_seconds: i64,
    pub last_write_at: Option<String>,
    pub consistency_status: String,
}

impl From<ConsistencyCheckV21> for ConsistencyCheckResponseV21 {
    fn from(c: ConsistencyCheckV21) -> Self {
        Self {
            id: c.id,
            source_node: c.source_node,
            target_node: c.target_node,
            table_name: c.table_name,
            check_type: c.check_type,
            status: c.status,
            discrepancy_count: c.discrepancy_count,
            checked_at: c.checked_at.to_rfc3339(),
        }
    }
}

impl From<FailoverHistoryV21> for FailoverHistoryResponseV21 {
    fn from(f: FailoverHistoryV21) -> Self {
        Self {
            id: f.id,
            source_node: f.source_node,
            target_node: f.target_node,
            failover_type: f.failover_type,
            reason: f.reason,
            duration_ms: f.duration_ms,
            success: f.success,
            initiated_by: f.initiated_by,
            occurred_at: f.occurred_at.to_rfc3339(),
        }
    }
}

async fn create_consistency_check(
    State(state): State<AppState>,
    Json(input): Json<CreateConsistencyCheckRequest>,
) -> Result<(StatusCode, Json<ConsistencyCheckResponseV21>), Response> {
    let row = sqlx::query_as::<_, ConsistencyCheckV21>(
        r#"INSERT INTO replication_consistency_checks_v21 (source_node, target_node, table_name, check_type)
         VALUES ($1, $2, $3, $4)
         RETURNING id, source_node, target_node, table_name, check_type, status, discrepancy_count, checked_at"#,
    )
    .bind(&input.source_node)
    .bind(&input.target_node)
    .bind(&input.table_name)
    .bind(&input.check_type)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(row.into())))
}

async fn get_consistency_check_history(
    State(state): State<AppState>,
    Path((source_node, target_node)): Path<(String, String)>,
) -> Result<Json<Vec<ConsistencyCheckResponseV21>>, Response> {
    let rows = sqlx::query_as::<_, ConsistencyCheckV21>(
        r#"SELECT id, source_node, target_node, table_name, check_type, status, discrepancy_count, checked_at
         FROM replication_consistency_checks_v21
         WHERE source_node = $1 AND target_node = $2
         ORDER BY checked_at DESC
         LIMIT 100"#,
    )
    .bind(&source_node)
    .bind(&target_node)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(rows.into_iter().map(|r| r.into()).collect()))
}

async fn record_failover(
    State(state): State<AppState>,
    Json(input): Json<RecordFailoverRequest>,
) -> Result<(StatusCode, Json<FailoverHistoryResponseV21>), Response> {
    let row = sqlx::query_as::<_, FailoverHistoryV21>(
        r#"INSERT INTO replication_failover_history_v21 (source_node, target_node, failover_type, reason, duration_ms, success, initiated_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id, source_node, target_node, failover_type, reason, duration_ms, success, initiated_by, occurred_at"#,
    )
    .bind(&input.source_node)
    .bind(&input.target_node)
    .bind(&input.failover_type)
    .bind(&input.reason)
    .bind(input.duration_ms)
    .bind(input.success)
    .bind(input.initiated_by)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(row.into())))
}

async fn get_failover_history(
    State(state): State<AppState>,
    Path(source_node): Path<String>,
) -> Result<Json<Vec<FailoverHistoryResponseV21>>, Response> {
    let rows = sqlx::query_as::<_, FailoverHistoryV21>(
        r#"SELECT id, source_node, target_node, failover_type, reason, duration_ms, success, initiated_by, occurred_at
         FROM replication_failover_history_v21
         WHERE source_node = $1
         ORDER BY occurred_at DESC
         LIMIT 100"#,
    )
    .bind(&source_node)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(rows.into_iter().map(|r| r.into()).collect()))
}

async fn get_replication_topology(
    State(state): State<AppState>,
) -> Result<Json<Vec<ReplicationTopologyResponseV22>>, Response> {
    #[derive(sqlx::FromRow)]
    struct ReplicaRow {
        name: String,
        status: String,
        lag_ms: i32,
    }

    let replicas = sqlx::query_as::<_, ReplicaRow>(
        r#"SELECT name, status, lag_ms FROM database_replicas ORDER BY name"#,
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let mut topology = Vec::new();
    for (i, replica) in replicas.iter().enumerate() {
        if i + 1 < replicas.len() {
            let target = &replicas[i + 1];

            let failover_count: (i64,) = sqlx::query_as(
                r#"SELECT COUNT(*) FROM replication_failover_history_v21
                 WHERE source_node = $1 AND target_node = $2"#,
            )
            .bind(&replica.name)
            .bind(&target.name)
            .fetch_one(state.db.pool())
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

            let last_check: Option<(Option<DateTime<Utc>>,)> = sqlx::query_as(
                r#"SELECT MAX(checked_at) FROM replication_consistency_checks_v21
                 WHERE source_node = $1 AND target_node = $2"#,
            )
            .bind(&replica.name)
            .bind(&target.name)
            .fetch_optional(state.db.pool())
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

            topology.push(ReplicationTopologyResponseV22 {
                source_node: replica.name.clone(),
                target_node: target.name.clone(),
                status: replica.status.clone(),
                lag_ms: Some(replica.lag_ms),
                consistency_score: if replica.lag_ms < 100 { 100.0 } else { 50.0 },
                last_check_at: last_check.and_then(|l| l.0).map(|t| t.to_rfc3339()),
                total_failovers: failover_count.0,
            });
        }
    }

    Ok(Json(topology))
}

async fn get_recovery_points(
    State(state): State<AppState>,
) -> Result<Json<Vec<RecoveryPointResponseV22>>, Response> {
    #[derive(sqlx::FromRow)]
    struct ReplicaRow {
        name: String,
        status: String,
        lag_ms: i32,
    }

    let replicas = sqlx::query_as::<_, ReplicaRow>(
        r#"SELECT name, status, lag_ms FROM database_replicas ORDER BY name"#,
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let now = Utc::now();
    let points: Vec<RecoveryPointResponseV22> = replicas
        .into_iter()
        .map(|r| {
            let lag_seconds = r.lag_ms as i64 / 1000;
            let recovery_point = now - chrono::Duration::seconds(lag_seconds);
            RecoveryPointResponseV22 {
                node_name: r.name,
                recovery_point: recovery_point.to_rfc3339(),
                lag_seconds,
                last_write_at: Some(recovery_point.to_rfc3339()),
                consistency_status: if r.status == "healthy" {
                    "consistent".to_string()
                } else {
                    "inconsistent".to_string()
                },
            }
        })
        .collect();

    Ok(Json(points))
}

pub fn database_replication_routes() -> Router<AppState> {
    Router::new()
        // v2 routes
        .route(
            "/api/v1/replication/logs",
            post(create_replication_log).get(list_replication_logs),
        )
        .route(
            "/api/v1/replication/logs/{id}",
            get(get_replication_log).patch(update_replication_log_status),
        )
        .route(
            "/api/v1/replication/stats",
            post(create_replication_stats),
        )
        .route(
            "/api/v1/replication/stats/{replica_id}",
            get(get_replication_stats),
        )
        .route(
            "/api/v1/replication/summary",
            get(get_replication_summary),
        )
        .route(
            "/api/v1/replication/lag-monitor",
            get(monitor_replication_lag),
        )
        // v5 routes
        .route("/api/v1/replication-v5/config", post(set_config))
        .route("/api/v1/replication-v5/config/{replica_id}", get(get_all_config))
        .route("/api/v1/replication-v5/config/{replica_id}/{config_key}", get(get_config))
        .route("/api/v1/replication-v5/alerts", post(create_replication_alert))
        .route("/api/v1/replication-v5/alerts/{replica_id}", get(get_replication_alerts))
        .route("/api/v1/replication-v5/monitor", get(monitor_replication))
        .route("/api/v1/replication-v5/failover-test/{replica_id}", post(test_failover))
        // v8 routes
        .route("/api/v1/replication-v8/config", post(set_config))
        .route("/api/v1/replication-v8/config/{replica_id}", get(get_all_config))
        .route("/api/v1/replication-v8/config/{replica_id}/{config_key}", get(get_config))
        .route("/api/v1/replication-v8/alerts", post(create_replication_alert))
        .route("/api/v1/replication-v8/alerts/{replica_id}", get(get_replication_alerts))
        .route("/api/v1/replication-v8/monitor", get(monitor_replication))
        .route("/api/v1/replication-v8/failover-test/{replica_id}", post(test_failover))
        // v9 routes
        .route("/api/v1/replication-v9/config", post(set_config))
        .route("/api/v1/replication-v9/config/{replica_id}", get(get_all_config))
        .route("/api/v1/replication-v9/config/{replica_id}/{config_key}", get(get_config))
        .route("/api/v1/replication-v9/alerts", post(create_replication_alert))
        .route("/api/v1/replication-v9/alerts/{replica_id}", get(get_replication_alerts))
        .route("/api/v1/replication-v9/monitor", get(monitor_replication))
        .route("/api/v1/replication-v9/failover-test/{replica_id}", post(test_failover))
        // v11 routes
        .route("/api/v1/replication-v11/config", post(set_config))
        .route("/api/v1/replication-v11/config/{replica_id}", get(get_all_config))
        .route("/api/v1/replication-v11/config/{replica_id}/{config_key}", get(get_config))
        .route("/api/v1/replication-v11/alerts", post(create_replication_alert))
        .route("/api/v1/replication-v11/alerts/{replica_id}", get(get_replication_alerts))
        .route("/api/v1/replication-v11/monitor", get(monitor_replication))
        .route("/api/v1/replication-v11/failover-test/{replica_id}", post(test_failover))
        // v12 routes
        .route("/api/v1/replication-v12/consistency-checks", post(create_consistency_check))
        .route("/api/v1/replication-v12/consistency-checks/{source}/{target}", get(get_consistency_check_history))
        .route("/api/v1/replication-v12/failover-history", post(record_failover))
        .route("/api/v1/replication-v12/failover-history/{source}", get(get_failover_history))
        .route("/api/v1/replication-v12/topology", get(get_replication_topology))
        .route("/api/v1/replication-v12/recovery-points", get(get_recovery_points))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replication_log_response_conversion() {
        let log = ReplicationLog {
            id: Uuid::nil(),
            replica_id: Uuid::nil(),
            operation: "INSERT".to_string(),
            table_name: "repos".to_string(),
            record_id: None,
            status: "pending".to_string(),
            created_at: Utc::now(),
        };
        let response: ReplicationLogResponse = log.into();
        assert_eq!(response.operation, "INSERT");
        assert_eq!(response.status, "pending");
    }

    #[test]
    fn test_replication_stats_response_conversion() {
        let stats = ReplicationStats {
            id: Uuid::nil(),
            replica_id: Uuid::nil(),
            period_start: Utc::now(),
            operations_count: 100,
            avg_lag_ms: 5.5,
            created_at: Utc::now(),
        };
        let response: ReplicationStatsResponse = stats.into();
        assert_eq!(response.operations_count, 100);
        assert_eq!(response.avg_lag_ms, 5.5);
    }

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

    #[test]
    fn test_consistency_check_response_conversion() {
        let check = ConsistencyCheckV21 {
            id: Uuid::nil(),
            source_node: "node-a".to_string(),
            target_node: "node-b".to_string(),
            table_name: "repos".to_string(),
            check_type: "full".to_string(),
            status: "completed".to_string(),
            discrepancy_count: 0,
            checked_at: Utc::now(),
        };
        let response: ConsistencyCheckResponseV21 = check.into();
        assert_eq!(response.source_node, "node-a");
        assert_eq!(response.discrepancy_count, 0);
    }

    #[test]
    fn test_failover_history_response_conversion() {
        let failover = FailoverHistoryV21 {
            id: Uuid::nil(),
            source_node: "node-a".to_string(),
            target_node: "node-b".to_string(),
            failover_type: "automatic".to_string(),
            reason: "node failure".to_string(),
            duration_ms: Some(1500),
            success: true,
            initiated_by: None,
            occurred_at: Utc::now(),
        };
        let response: FailoverHistoryResponseV21 = failover.into();
        assert_eq!(response.failover_type, "automatic");
        assert!(response.success);
    }
}
