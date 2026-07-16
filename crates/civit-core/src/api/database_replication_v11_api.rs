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
pub struct ConflictResolverV20 {
    pub id: Uuid,
    pub table_name: String,
    pub resolver_type: String,
    pub custom_logic: Option<serde_json::Value>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LagAlertV20 {
    pub id: Uuid,
    pub source_node: String,
    pub target_node: String,
    pub threshold_ms: i32,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateResolverRequest {
    pub table_name: String,
    pub resolver_type: String,
    pub custom_logic: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateLagAlertRequest {
    pub source_node: String,
    pub target_node: String,
    pub threshold_ms: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConflictResolverResponseV20 {
    pub id: Uuid,
    pub table_name: String,
    pub resolver_type: String,
    pub custom_logic: Option<serde_json::Value>,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LagAlertResponseV20 {
    pub id: Uuid,
    pub source_node: String,
    pub target_node: String,
    pub threshold_ms: i32,
    pub enabled: bool,
    pub last_triggered_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConsistencyCheckResponseV21 {
    pub check_id: Uuid,
    pub source_node: String,
    pub target_node: String,
    pub table_name: String,
    pub consistent: bool,
    pub discrepancy_count: i64,
    pub checked_at: String,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct FailoverStatusResponseV21 {
    pub source_node: String,
    pub target_node: String,
    pub status: String,
    pub failover_time: Option<String>,
    pub last_heartbeat: Option<String>,
    pub lag_ms: Option<i32>,
}

impl From<ConflictResolverV20> for ConflictResolverResponseV20 {
    fn from(r: ConflictResolverV20) -> Self {
        Self {
            id: r.id,
            table_name: r.table_name,
            resolver_type: r.resolver_type,
            custom_logic: r.custom_logic,
            enabled: r.enabled,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

impl From<LagAlertV20> for LagAlertResponseV20 {
    fn from(a: LagAlertV20) -> Self {
        Self {
            id: a.id,
            source_node: a.source_node,
            target_node: a.target_node,
            threshold_ms: a.threshold_ms,
            enabled: a.enabled,
            last_triggered_at: a.last_triggered_at.map(|t| t.to_rfc3339()),
            created_at: a.created_at.to_rfc3339(),
        }
    }
}

async fn create_resolver(
    State(state): State<AppState>,
    Json(input): Json<CreateResolverRequest>,
) -> Result<(StatusCode, Json<ConflictResolverResponseV20>), Response> {
    let resolver = sqlx::query_as::<_, ConflictResolverV20>(
        r#"INSERT INTO replication_conflict_resolvers_v20 (table_name, resolver_type, custom_logic)
         VALUES ($1, $2, $3)
         ON CONFLICT (table_name) DO UPDATE SET resolver_type = $2, custom_logic = $3
         RETURNING id, table_name, resolver_type, custom_logic, enabled, created_at"#,
    )
    .bind(&input.table_name)
    .bind(&input.resolver_type)
    .bind(&input.custom_logic)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(resolver.into())))
}

async fn get_resolvers(
    State(state): State<AppState>,
) -> Result<Json<Vec<ConflictResolverResponseV20>>, Response> {
    let resolvers = sqlx::query_as::<_, ConflictResolverV20>(
        r#"SELECT id, table_name, resolver_type, custom_logic, enabled, created_at
         FROM replication_conflict_resolvers_v20
         ORDER BY table_name"#,
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(resolvers.into_iter().map(|r| r.into()).collect()))
}

async fn create_lag_alert(
    State(state): State<AppState>,
    Json(input): Json<CreateLagAlertRequest>,
) -> Result<(StatusCode, Json<LagAlertResponseV20>), Response> {
    let alert = sqlx::query_as::<_, LagAlertV20>(
        r#"INSERT INTO replication_lag_alerts_v20 (source_node, target_node, threshold_ms)
         VALUES ($1, $2, $3)
         RETURNING id, source_node, target_node, threshold_ms, enabled, last_triggered_at, created_at"#,
    )
    .bind(&input.source_node)
    .bind(&input.target_node)
    .bind(input.threshold_ms)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok((StatusCode::CREATED, Json(alert.into())))
}

async fn get_lag_alerts(
    State(state): State<AppState>,
) -> Result<Json<Vec<LagAlertResponseV20>>, Response> {
    let alerts = sqlx::query_as::<_, LagAlertV20>(
        r#"SELECT id, source_node, target_node, threshold_ms, enabled, last_triggered_at, created_at
         FROM replication_lag_alerts_v20
         ORDER BY created_at DESC"#,
    )
    .fetch_all(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    Ok(Json(alerts.into_iter().map(|a| a.into()).collect()))
}

async fn run_consistency_check(
    State(state): State<AppState>,
    Path((source_node, target_node)): Path<(String, String)>,
) -> Result<Json<ConsistencyCheckResponseV21>, Response> {
    let check_id = Uuid::new_v4();
    let checked_at = Utc::now();

    let source_count: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM database_replicas WHERE name = $1"#,
    )
    .bind(&source_node)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let target_count: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM database_replicas WHERE name = $1"#,
    )
    .bind(&target_node)
    .fetch_one(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    let discrepancy_count = (source_count.0 - target_count.0).abs();
    let consistent = discrepancy_count == 0;

    Ok(Json(ConsistencyCheckResponseV21 {
        check_id,
        source_node,
        target_node,
        table_name: "database_replicas".to_string(),
        consistent,
        discrepancy_count,
        checked_at: checked_at.to_rfc3339(),
        details: serde_json::json!({
            "source_replica_count": source_count.0,
            "target_replica_count": target_count.0,
        }),
    }))
}

async fn get_failover_status(
    State(state): State<AppState>,
    Path((source_node, target_node)): Path<(String, String)>,
) -> Result<Json<FailoverStatusResponseV21>, Response> {
    #[derive(sqlx::FromRow)]
    struct ReplicaRow {
        #[allow(dead_code)]
        name: String,
        status: String,
        lag_ms: i32,
    }

    let replica = sqlx::query_as::<_, ReplicaRow>(
        r#"SELECT name, status, lag_ms FROM database_replicas WHERE name = $1 OR name = $2"#,
    )
    .bind(&source_node)
    .bind(&target_node)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response())?;

    match replica {
        Some(r) => Ok(Json(FailoverStatusResponseV21 {
            source_node,
            target_node,
            status: r.status,
            failover_time: None,
            last_heartbeat: Some(Utc::now().to_rfc3339()),
            lag_ms: Some(r.lag_ms),
        })),
        None => Ok(Json(FailoverStatusResponseV21 {
            source_node,
            target_node,
            status: "unknown".to_string(),
            failover_time: None,
            last_heartbeat: None,
            lag_ms: None,
        })),
    }
}

pub fn replication_v11_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/replication-v11/conflict-resolvers",
            post(create_resolver).get(get_resolvers),
        )
        .route(
            "/api/v1/replication-v11/lag-alerts",
            post(create_lag_alert).get(get_lag_alerts),
        )
        .route(
            "/api/v1/replication-v11/consistency-check/{source}/{target}",
            post(run_consistency_check),
        )
        .route(
            "/api/v1/replication-v11/failover-status/{source}/{target}",
            get(get_failover_status),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_resolver_response_conversion() {
        let resolver = ConflictResolverV20 {
            id: Uuid::nil(),
            table_name: "repos".to_string(),
            resolver_type: "last_write_wins".to_string(),
            custom_logic: None,
            enabled: true,
            created_at: Utc::now(),
        };
        let response: ConflictResolverResponseV20 = resolver.into();
        assert_eq!(response.table_name, "repos");
        assert_eq!(response.resolver_type, "last_write_wins");
    }

    #[test]
    fn test_lag_alert_response_conversion() {
        let alert = LagAlertV20 {
            id: Uuid::nil(),
            source_node: "node-a".to_string(),
            target_node: "node-b".to_string(),
            threshold_ms: 500,
            enabled: true,
            last_triggered_at: None,
            created_at: Utc::now(),
        };
        let response: LagAlertResponseV20 = alert.into();
        assert_eq!(response.threshold_ms, 500);
        assert!(response.last_triggered_at.is_none());
    }
}
