#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyCheckEntryV21 {
    pub id: Uuid,
    pub source_node: String,
    pub target_node: String,
    pub table_name: String,
    pub check_type: String,
    pub status: String,
    pub discrepancy_count: i32,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverHistoryEntryV21 {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationTopologyV22 {
    pub source_node: String,
    pub target_node: String,
    pub status: String,
    pub lag_ms: Option<i32>,
    pub consistency_score: f64,
    pub last_check_at: Option<DateTime<Utc>>,
    pub total_failovers: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPointV22 {
    pub node_name: String,
    pub recovery_point: DateTime<Utc>,
    pub lag_seconds: i64,
    pub last_write_at: Option<DateTime<Utc>>,
    pub consistency_status: String,
}

pub struct DatabaseReplicationV22Service {
    pool: PgPool,
}

impl DatabaseReplicationV22Service {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_consistency_check(
        &self,
        source_node: &str,
        target_node: &str,
        table_name: &str,
        check_type: &str,
    ) -> Result<ConsistencyCheckEntryV21, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct CheckRow {
            id: Uuid,
            source_node: String,
            target_node: String,
            table_name: String,
            check_type: String,
            status: String,
            discrepancy_count: i32,
            checked_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, CheckRow>(
            r#"INSERT INTO replication_consistency_checks_v21 (source_node, target_node, table_name, check_type)
             VALUES ($1, $2, $3, $4)
             RETURNING id, source_node, target_node, table_name, check_type, status, discrepancy_count, checked_at"#,
        )
        .bind(source_node)
        .bind(target_node)
        .bind(table_name)
        .bind(check_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(ConsistencyCheckEntryV21 {
            id: row.id,
            source_node: row.source_node,
            target_node: row.target_node,
            table_name: row.table_name,
            check_type: row.check_type,
            status: row.status,
            discrepancy_count: row.discrepancy_count,
            checked_at: row.checked_at,
        })
    }

    pub async fn get_consistency_check_history(
        &self,
        source_node: Option<&str>,
        target_node: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ConsistencyCheckEntryV21>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct CheckRow {
            id: Uuid,
            source_node: String,
            target_node: String,
            table_name: String,
            check_type: String,
            status: String,
            discrepancy_count: i32,
            checked_at: DateTime<Utc>,
        }

        let rows = match (source_node, target_node) {
            (Some(sn), Some(tn)) => {
                sqlx::query_as::<_, CheckRow>(
                    r#"SELECT id, source_node, target_node, table_name, check_type, status, discrepancy_count, checked_at
                     FROM replication_consistency_checks_v21
                     WHERE source_node = $1 AND target_node = $2
                     ORDER BY checked_at DESC
                     LIMIT $3"#,
                )
                .bind(sn)
                .bind(tn)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(sn), None) => {
                sqlx::query_as::<_, CheckRow>(
                    r#"SELECT id, source_node, target_node, table_name, check_type, status, discrepancy_count, checked_at
                     FROM replication_consistency_checks_v21
                     WHERE source_node = $1
                     ORDER BY checked_at DESC
                     LIMIT $2"#,
                )
                .bind(sn)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(tn)) => {
                sqlx::query_as::<_, CheckRow>(
                    r#"SELECT id, source_node, target_node, table_name, check_type, status, discrepancy_count, checked_at
                     FROM replication_consistency_checks_v21
                     WHERE target_node = $1
                     ORDER BY checked_at DESC
                     LIMIT $2"#,
                )
                .bind(tn)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            (None, None) => {
                sqlx::query_as::<_, CheckRow>(
                    r#"SELECT id, source_node, target_node, table_name, check_type, status, discrepancy_count, checked_at
                     FROM replication_consistency_checks_v21
                     ORDER BY checked_at DESC
                     LIMIT $1"#,
                )
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(rows
            .into_iter()
            .map(|r| ConsistencyCheckEntryV21 {
                id: r.id,
                source_node: r.source_node,
                target_node: r.target_node,
                table_name: r.table_name,
                check_type: r.check_type,
                status: r.status,
                discrepancy_count: r.discrepancy_count,
                checked_at: r.checked_at,
            })
            .collect())
    }

    pub async fn record_failover(
        &self,
        source_node: &str,
        target_node: &str,
        failover_type: &str,
        reason: &str,
        duration_ms: Option<i32>,
        success: bool,
        initiated_by: Option<Uuid>,
    ) -> Result<FailoverHistoryEntryV21, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct FailoverRow {
            id: Uuid,
            source_node: String,
            target_node: String,
            failover_type: String,
            reason: String,
            duration_ms: Option<i32>,
            success: bool,
            initiated_by: Option<Uuid>,
            occurred_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, FailoverRow>(
            r#"INSERT INTO replication_failover_history_v21 (source_node, target_node, failover_type, reason, duration_ms, success, initiated_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, source_node, target_node, failover_type, reason, duration_ms, success, initiated_by, occurred_at"#,
        )
        .bind(source_node)
        .bind(target_node)
        .bind(failover_type)
        .bind(reason)
        .bind(duration_ms)
        .bind(success)
        .bind(initiated_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(FailoverHistoryEntryV21 {
            id: row.id,
            source_node: row.source_node,
            target_node: row.target_node,
            failover_type: row.failover_type,
            reason: row.reason,
            duration_ms: row.duration_ms,
            success: row.success,
            initiated_by: row.initiated_by,
            occurred_at: row.occurred_at,
        })
    }

    pub async fn get_failover_history(
        &self,
        source_node: Option<&str>,
        limit: i64,
    ) -> Result<Vec<FailoverHistoryEntryV21>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct FailoverRow {
            id: Uuid,
            source_node: String,
            target_node: String,
            failover_type: String,
            reason: String,
            duration_ms: Option<i32>,
            success: bool,
            initiated_by: Option<Uuid>,
            occurred_at: DateTime<Utc>,
        }

        let rows = match source_node {
            Some(sn) => {
                sqlx::query_as::<_, FailoverRow>(
                    r#"SELECT id, source_node, target_node, failover_type, reason, duration_ms, success, initiated_by, occurred_at
                     FROM replication_failover_history_v21
                     WHERE source_node = $1
                     ORDER BY occurred_at DESC
                     LIMIT $2"#,
                )
                .bind(sn)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, FailoverRow>(
                    r#"SELECT id, source_node, target_node, failover_type, reason, duration_ms, success, initiated_by, occurred_at
                     FROM replication_failover_history_v21
                     ORDER BY occurred_at DESC
                     LIMIT $1"#,
                )
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(rows
            .into_iter()
            .map(|r| FailoverHistoryEntryV21 {
                id: r.id,
                source_node: r.source_node,
                target_node: r.target_node,
                failover_type: r.failover_type,
                reason: r.reason,
                duration_ms: r.duration_ms,
                success: r.success,
                initiated_by: r.initiated_by,
                occurred_at: r.occurred_at,
            })
            .collect())
    }

    pub async fn get_replication_topology(
        &self,
    ) -> Result<Vec<ReplicationTopologyV22>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct ReplicaRow {
            name: String,
            status: String,
            lag_ms: i32,
        }

        let replicas = sqlx::query_as::<_, ReplicaRow>(
            r#"SELECT name, status, lag_ms FROM database_replicas ORDER BY name"#,
        )
        .fetch_all(&self.pool)
        .await?;

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
                .fetch_one(&self.pool)
                .await?;

                let last_check: Option<(Option<DateTime<Utc>>,)> = sqlx::query_as(
                    r#"SELECT MAX(checked_at) FROM replication_consistency_checks_v21
                     WHERE source_node = $1 AND target_node = $2"#,
                )
                .bind(&replica.name)
                .bind(&target.name)
                .fetch_optional(&self.pool)
                .await?;

                topology.push(ReplicationTopologyV22 {
                    source_node: replica.name.clone(),
                    target_node: target.name.clone(),
                    status: replica.status.clone(),
                    lag_ms: Some(replica.lag_ms),
                    consistency_score: if replica.lag_ms < 100 { 100.0 } else { 50.0 },
                    last_check_at: last_check.and_then(|l| l.0),
                    total_failovers: failover_count.0,
                });
            }
        }

        Ok(topology)
    }

    pub async fn get_recovery_points(
        &self,
    ) -> Result<Vec<RecoveryPointV22>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct ReplicaRow {
            name: String,
            status: String,
            lag_ms: i32,
        }

        let replicas = sqlx::query_as::<_, ReplicaRow>(
            r#"SELECT name, status, lag_ms FROM database_replicas ORDER BY name"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut points = Vec::new();
        let now = Utc::now();

        for replica in replicas {
            let lag_seconds = replica.lag_ms as i64 / 1000;
            let recovery_point = now - chrono::Duration::seconds(lag_seconds);

            points.push(RecoveryPointV22 {
                node_name: replica.name,
                recovery_point,
                lag_seconds,
                last_write_at: Some(recovery_point),
                consistency_status: if replica.status == "healthy" {
                    "consistent".to_string()
                } else {
                    "inconsistent".to_string()
                },
            });
        }

        Ok(points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistency_check_entry_serialization() {
        let entry = ConsistencyCheckEntryV21 {
            id: Uuid::nil(),
            source_node: "node-a".to_string(),
            target_node: "node-b".to_string(),
            table_name: "repos".to_string(),
            check_type: "full".to_string(),
            status: "completed".to_string(),
            discrepancy_count: 0,
            checked_at: Utc::now(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deser: ConsistencyCheckEntryV21 = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.source_node, "node-a");
        assert_eq!(deser.discrepancy_count, 0);
    }

    #[test]
    fn test_failover_history_entry_serialization() {
        let entry = FailoverHistoryEntryV21 {
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
        let json = serde_json::to_string(&entry).unwrap();
        let deser: FailoverHistoryEntryV21 = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.failover_type, "automatic");
        assert!(deser.success);
    }

    #[test]
    fn test_replication_topology_serialization() {
        let topology = ReplicationTopologyV22 {
            source_node: "node-a".to_string(),
            target_node: "node-b".to_string(),
            status: "healthy".to_string(),
            lag_ms: Some(50),
            consistency_score: 100.0,
            last_check_at: Some(Utc::now()),
            total_failovers: 3,
        };
        let json = serde_json::to_string(&topology).unwrap();
        let deser: ReplicationTopologyV22 = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.consistency_score, 100.0);
        assert_eq!(deser.total_failovers, 3);
    }

    #[test]
    fn test_recovery_point_serialization() {
        let point = RecoveryPointV22 {
            node_name: "node-a".to_string(),
            recovery_point: Utc::now(),
            lag_seconds: 5,
            last_write_at: Some(Utc::now()),
            consistency_status: "consistent".to_string(),
        };
        let json = serde_json::to_string(&point).unwrap();
        let deser: RecoveryPointV22 = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.lag_seconds, 5);
        assert_eq!(deser.consistency_status, "consistent");
    }
}
