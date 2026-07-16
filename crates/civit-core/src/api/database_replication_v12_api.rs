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

pub fn replication_v12_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/replication-v12/consistency-checks",
            post(create_consistency_check),
        )
        .route(
            "/api/v1/replication-v12/consistency-checks/{source}/{target}",
            get(get_consistency_check_history),
        )
        .route(
            "/api/v1/replication-v12/failover-history",
            post(record_failover),
        )
        .route(
            "/api/v1/replication-v12/failover-history/{source}",
            get(get_failover_history),
        )
        .route(
            "/api/v1/replication-v12/topology",
            get(get_replication_topology),
        )
        .route(
            "/api/v1/replication-v12/recovery-points",
            get(get_recovery_points),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

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
