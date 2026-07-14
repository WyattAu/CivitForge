#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationLogEntry {
    pub id: Uuid,
    pub replica_id: Uuid,
    pub operation: String,
    pub table_name: String,
    pub record_id: Option<Uuid>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationStatsEntry {
    pub id: Uuid,
    pub replica_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub operations_count: i32,
    pub avg_lag_ms: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationMonitorResult {
    pub replica_id: Uuid,
    pub replica_name: String,
    pub lag_ms: i32,
    pub status: String,
    pub healthy: bool,
}

pub struct DatabaseReplicationV2Service {
    pool: PgPool,
}

impl DatabaseReplicationV2Service {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn log_operation(
        &self,
        replica_id: Uuid,
        operation: &str,
        table_name: &str,
        record_id: Option<Uuid>,
    ) -> Result<ReplicationLogEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct LogRow {
            id: Uuid,
            replica_id: Uuid,
            operation: String,
            table_name: String,
            record_id: Option<Uuid>,
            status: String,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, LogRow>(
            r#"INSERT INTO database_replication_logs (replica_id, operation, table_name, record_id)
             VALUES ($1, $2, $3, $4)
             RETURNING id, replica_id, operation, table_name, record_id, status, created_at"#,
        )
        .bind(replica_id)
        .bind(operation)
        .bind(table_name)
        .bind(record_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(ReplicationLogEntry {
            id: row.id,
            replica_id: row.replica_id,
            operation: row.operation,
            table_name: row.table_name,
            record_id: row.record_id,
            status: row.status,
            created_at: row.created_at,
        })
    }

    pub async fn record_stats(
        &self,
        replica_id: Uuid,
        period_start: DateTime<Utc>,
        operations_count: i32,
        avg_lag_ms: f64,
    ) -> Result<ReplicationStatsEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct StatsRow {
            id: Uuid,
            replica_id: Uuid,
            period_start: DateTime<Utc>,
            operations_count: i32,
            avg_lag_ms: f64,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, StatsRow>(
            r#"INSERT INTO database_replication_stats (replica_id, period_start, operations_count, avg_lag_ms)
             VALUES ($1, $2, $3, $4)
             RETURNING id, replica_id, period_start, operations_count, avg_lag_ms, created_at"#,
        )
        .bind(replica_id)
        .bind(period_start)
        .bind(operations_count)
        .bind(avg_lag_ms)
        .fetch_one(&self.pool)
        .await?;

        Ok(ReplicationStatsEntry {
            id: row.id,
            replica_id: row.replica_id,
            period_start: row.period_start,
            operations_count: row.operations_count,
            avg_lag_ms: row.avg_lag_ms,
            created_at: row.created_at,
        })
    }

    pub async fn monitor_lag(&self) -> Result<Vec<ReplicationMonitorResult>, sqlx::Error> {
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
        .fetch_all(&self.pool)
        .await?;

        Ok(replicas
            .into_iter()
            .map(|r| ReplicationMonitorResult {
                replica_id: r.id,
                replica_name: r.name,
                lag_ms: r.lag_ms,
                status: r.status,
                healthy: r.lag_ms < 1000,
            })
            .collect())
    }
}
