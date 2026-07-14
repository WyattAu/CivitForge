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

pub fn replication_v2_routes() -> Router<AppState> {
    Router::new()
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
}
