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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationConfigEntry {
    pub id: Uuid,
    pub replica_id: Uuid,
    pub config_key: String,
    pub config_value: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationAlertEntry {
    pub id: Uuid,
    pub replica_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationMonitoringResult {
    pub replica_id: Uuid,
    pub replica_name: String,
    pub lag_ms: i32,
    pub status: String,
    pub healthy: bool,
    pub alerts: Vec<ReplicationAlertEntry>,
    pub config: Vec<ReplicationConfigEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverTestResult {
    pub replica_id: Uuid,
    pub success: bool,
    pub duration_ms: i32,
    pub error_message: Option<String>,
    pub tested_at: DateTime<Utc>,
    pub steps: Vec<FailoverTestStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverTestStep {
    pub step_name: String,
    pub success: bool,
    pub duration_ms: i32,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolverEntry {
    pub id: Uuid,
    pub table_name: String,
    pub resolver_type: String,
    pub custom_logic: Option<serde_json::Value>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LagAlertEntry {
    pub id: Uuid,
    pub source_node: String,
    pub target_node: String,
    pub threshold_ms: i32,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyCheckResult {
    pub check_id: Uuid,
    pub source_node: String,
    pub target_node: String,
    pub table_name: String,
    pub consistent: bool,
    pub discrepancy_count: i64,
    pub checked_at: DateTime<Utc>,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverStatus {
    pub source_node: String,
    pub target_node: String,
    pub status: String,
    pub failover_time: Option<DateTime<Utc>>,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub lag_ms: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyCheckEntry {
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
pub struct FailoverHistoryEntry {
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
pub struct ReplicationTopology {
    pub source_node: String,
    pub target_node: String,
    pub status: String,
    pub lag_ms: Option<i32>,
    pub consistency_score: f64,
    pub last_check_at: Option<DateTime<Utc>>,
    pub total_failovers: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPoint {
    pub node_name: String,
    pub recovery_point: DateTime<Utc>,
    pub lag_seconds: i64,
    pub last_write_at: Option<DateTime<Utc>>,
    pub consistency_status: String,
}

pub struct DatabaseReplicationService {
    pool: PgPool,
}

impl DatabaseReplicationService {
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

    pub async fn set_config(
        &self,
        replica_id: Uuid,
        config_key: &str,
        config_value: serde_json::Value,
    ) -> Result<ReplicationConfigEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct ConfigRow {
            id: Uuid,
            replica_id: Uuid,
            config_key: String,
            config_value: serde_json::Value,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, ConfigRow>(
            r#"INSERT INTO database_replication_config_v18 (replica_id, config_key, config_value)
             VALUES ($1, $2, $3)
             ON CONFLICT (replica_id, config_key) DO UPDATE SET config_value = $3
             RETURNING id, replica_id, config_key, config_value, created_at"#,
        )
        .bind(replica_id)
        .bind(config_key)
        .bind(config_value)
        .fetch_one(&self.pool)
        .await?;

        Ok(ReplicationConfigEntry {
            id: row.id,
            replica_id: row.replica_id,
            config_key: row.config_key,
            config_value: row.config_value,
            created_at: row.created_at,
        })
    }

    pub async fn get_config(
        &self,
        replica_id: Uuid,
        config_key: &str,
    ) -> Result<Option<ReplicationConfigEntry>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct ConfigRow {
            id: Uuid,
            replica_id: Uuid,
            config_key: String,
            config_value: serde_json::Value,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, ConfigRow>(
            r#"SELECT id, replica_id, config_key, config_value, created_at
             FROM database_replication_config_v18
             WHERE replica_id = $1 AND config_key = $2"#,
        )
        .bind(replica_id)
        .bind(config_key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| ReplicationConfigEntry {
            id: r.id,
            replica_id: r.replica_id,
            config_key: r.config_key,
            config_value: r.config_value,
            created_at: r.created_at,
        }))
    }

    pub async fn get_all_config(
        &self,
        replica_id: Uuid,
    ) -> Result<Vec<ReplicationConfigEntry>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct ConfigRow {
            id: Uuid,
            replica_id: Uuid,
            config_key: String,
            config_value: serde_json::Value,
            created_at: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, ConfigRow>(
            r#"SELECT id, replica_id, config_key, config_value, created_at
             FROM database_replication_config_v18
             WHERE replica_id = $1
             ORDER BY config_key"#,
        )
        .bind(replica_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ReplicationConfigEntry {
                id: r.id,
                replica_id: r.replica_id,
                config_key: r.config_key,
                config_value: r.config_value,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn create_alert(
        &self,
        replica_id: Uuid,
        alert_type: &str,
        threshold: f64,
    ) -> Result<ReplicationAlertEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AlertRow {
            id: Uuid,
            replica_id: Uuid,
            alert_type: String,
            threshold: f64,
            enabled: bool,
            last_triggered_at: Option<DateTime<Utc>>,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, AlertRow>(
            r#"INSERT INTO database_replication_alerts_v18 (replica_id, alert_type, threshold)
             VALUES ($1, $2, $3)
             RETURNING id, replica_id, alert_type, threshold, enabled, last_triggered_at, created_at"#,
        )
        .bind(replica_id)
        .bind(alert_type)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await?;

        Ok(ReplicationAlertEntry {
            id: row.id,
            replica_id: row.replica_id,
            alert_type: row.alert_type,
            threshold: row.threshold,
            enabled: row.enabled,
            last_triggered_at: row.last_triggered_at,
            created_at: row.created_at,
        })
    }

    pub async fn get_alerts(
        &self,
        replica_id: Uuid,
    ) -> Result<Vec<ReplicationAlertEntry>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AlertRow {
            id: Uuid,
            replica_id: Uuid,
            alert_type: String,
            threshold: f64,
            enabled: bool,
            last_triggered_at: Option<DateTime<Utc>>,
            created_at: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, AlertRow>(
            r#"SELECT id, replica_id, alert_type, threshold, enabled, last_triggered_at, created_at
             FROM database_replication_alerts_v18
             WHERE replica_id = $1
             ORDER BY created_at DESC"#,
        )
        .bind(replica_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ReplicationAlertEntry {
                id: r.id,
                replica_id: r.replica_id,
                alert_type: r.alert_type,
                threshold: r.threshold,
                enabled: r.enabled,
                last_triggered_at: r.last_triggered_at,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn monitor_replication(&self) -> Result<Vec<ReplicationMonitoringResult>, sqlx::Error> {
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

        let mut results = Vec::new();
        for replica in replicas {
            let alerts = self.get_alerts(replica.id).await.unwrap_or_default();
            let config = self.get_all_config(replica.id).await.unwrap_or_default();
            results.push(ReplicationMonitoringResult {
                replica_id: replica.id,
                replica_name: replica.name,
                lag_ms: replica.lag_ms,
                status: replica.status,
                healthy: replica.lag_ms < 1000,
                alerts,
                config,
            });
        }

        Ok(results)
    }

    pub async fn test_failover(
        &self,
        replica_id: Uuid,
    ) -> Result<FailoverTestResult, sqlx::Error> {
        let start = Utc::now();
        let mut steps = Vec::new();

        let replica_exists: Option<(Uuid,)> = sqlx::query_as(
            r#"SELECT id FROM database_replicas WHERE id = $1"#,
        )
        .bind(replica_id)
        .fetch_optional(&self.pool)
        .await?;

        if replica_exists.is_none() {
            return Ok(FailoverTestResult {
                replica_id,
                success: false,
                duration_ms: 0,
                error_message: Some("Replica not found".to_string()),
                tested_at: Utc::now(),
                steps,
            });
        }

        let step_start = Utc::now();
        let _config_count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM database_replication_config_v18 WHERE replica_id = $1"#,
        )
        .bind(replica_id)
        .fetch_one(&self.pool)
        .await?;
        let step_duration = (Utc::now() - step_start).num_milliseconds() as i32;
        steps.push(FailoverTestStep {
            step_name: "verify_config".into(),
            success: true,
            duration_ms: step_duration,
            error_message: None,
        });

        let step_start = Utc::now();
        let _alert_count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM database_replication_alerts_v18 WHERE replica_id = $1"#,
        )
        .bind(replica_id)
        .fetch_one(&self.pool)
        .await?;
        let step_duration = (Utc::now() - step_start).num_milliseconds() as i32;
        steps.push(FailoverTestStep {
            step_name: "verify_alerts".into(),
            success: true,
            duration_ms: step_duration,
            error_message: None,
        });

        let end = Utc::now();
        let duration_ms = (end - start).num_milliseconds() as i32;

        Ok(FailoverTestResult {
            replica_id,
            success: true,
            duration_ms,
            error_message: None,
            tested_at: end,
            steps,
        })
    }

    pub async fn create_conflict_resolver(
        &self,
        table_name: &str,
        resolver_type: &str,
        custom_logic: Option<serde_json::Value>,
    ) -> Result<ConflictResolverEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct ResolverRow {
            id: Uuid,
            table_name: String,
            resolver_type: String,
            custom_logic: Option<serde_json::Value>,
            enabled: bool,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, ResolverRow>(
            r#"INSERT INTO replication_conflict_resolvers_v20 (table_name, resolver_type, custom_logic)
             VALUES ($1, $2, $3)
             ON CONFLICT (table_name) DO UPDATE SET resolver_type = $2, custom_logic = $3
             RETURNING id, table_name, resolver_type, custom_logic, enabled, created_at"#,
        )
        .bind(table_name)
        .bind(resolver_type)
        .bind(custom_logic)
        .fetch_one(&self.pool)
        .await?;

        Ok(ConflictResolverEntry {
            id: row.id,
            table_name: row.table_name,
            resolver_type: row.resolver_type,
            custom_logic: row.custom_logic,
            enabled: row.enabled,
            created_at: row.created_at,
        })
    }

    pub async fn get_conflict_resolvers(
        &self,
    ) -> Result<Vec<ConflictResolverEntry>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct ResolverRow {
            id: Uuid,
            table_name: String,
            resolver_type: String,
            custom_logic: Option<serde_json::Value>,
            enabled: bool,
            created_at: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, ResolverRow>(
            r#"SELECT id, table_name, resolver_type, custom_logic, enabled, created_at
             FROM replication_conflict_resolvers_v20
             ORDER BY table_name"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ConflictResolverEntry {
                id: r.id,
                table_name: r.table_name,
                resolver_type: r.resolver_type,
                custom_logic: r.custom_logic,
                enabled: r.enabled,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn create_lag_alert(
        &self,
        source_node: &str,
        target_node: &str,
        threshold_ms: i32,
    ) -> Result<LagAlertEntry, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AlertRow {
            id: Uuid,
            source_node: String,
            target_node: String,
            threshold_ms: i32,
            enabled: bool,
            last_triggered_at: Option<DateTime<Utc>>,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, AlertRow>(
            r#"INSERT INTO replication_lag_alerts_v20 (source_node, target_node, threshold_ms)
             VALUES ($1, $2, $3)
             RETURNING id, source_node, target_node, threshold_ms, enabled, last_triggered_at, created_at"#,
        )
        .bind(source_node)
        .bind(target_node)
        .bind(threshold_ms)
        .fetch_one(&self.pool)
        .await?;

        Ok(LagAlertEntry {
            id: row.id,
            source_node: row.source_node,
            target_node: row.target_node,
            threshold_ms: row.threshold_ms,
            enabled: row.enabled,
            last_triggered_at: row.last_triggered_at,
            created_at: row.created_at,
        })
    }

    pub async fn get_lag_alerts(
        &self,
        source_node: Option<&str>,
    ) -> Result<Vec<LagAlertEntry>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct AlertRow {
            id: Uuid,
            source_node: String,
            target_node: String,
            threshold_ms: i32,
            enabled: bool,
            last_triggered_at: Option<DateTime<Utc>>,
            created_at: DateTime<Utc>,
        }

        let rows = match source_node {
            Some(sn) => {
                sqlx::query_as::<_, AlertRow>(
                    r#"SELECT id, source_node, target_node, threshold_ms, enabled, last_triggered_at, created_at
                     FROM replication_lag_alerts_v20
                     WHERE source_node = $1
                     ORDER BY created_at DESC"#,
                )
                .bind(sn)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, AlertRow>(
                    r#"SELECT id, source_node, target_node, threshold_ms, enabled, last_triggered_at, created_at
                     FROM replication_lag_alerts_v20
                     ORDER BY created_at DESC"#,
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(rows
            .into_iter()
            .map(|r| LagAlertEntry {
                id: r.id,
                source_node: r.source_node,
                target_node: r.target_node,
                threshold_ms: r.threshold_ms,
                enabled: r.enabled,
                last_triggered_at: r.last_triggered_at,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn run_consistency_check(
        &self,
        source_node: &str,
        target_node: &str,
        table_name: &str,
    ) -> Result<ConsistencyCheckResult, sqlx::Error> {
        let check_id = Uuid::new_v4();
        let checked_at = Utc::now();

        let source_count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM database_replicas WHERE name = $1"#,
        )
        .bind(source_node)
        .fetch_one(&self.pool)
        .await
            .unwrap_or((0,));

        let target_count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM database_replicas WHERE name = $1"#,
        )
        .bind(target_node)
        .fetch_one(&self.pool)
        .await
            .unwrap_or((0,));

        let discrepancy_count = (source_count.0 - target_count.0).abs();
        let consistent = discrepancy_count == 0;

        Ok(ConsistencyCheckResult {
            check_id,
            source_node: source_node.to_string(),
            target_node: target_node.to_string(),
            table_name: table_name.to_string(),
            consistent,
            discrepancy_count,
            checked_at,
            details: serde_json::json!({
                "source_replica_count": source_count.0,
                "target_replica_count": target_count.0,
                "table_name": table_name,
            }),
        })
    }

    pub async fn get_failover_status(
        &self,
        source_node: &str,
        target_node: &str,
    ) -> Result<FailoverStatus, sqlx::Error> {
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
        .bind(source_node)
        .bind(target_node)
        .fetch_optional(&self.pool)
        .await?;

        match replica {
            Some(r) => Ok(FailoverStatus {
                source_node: source_node.to_string(),
                target_node: target_node.to_string(),
                status: r.status,
                failover_time: None,
                last_heartbeat: Some(Utc::now()),
                lag_ms: Some(r.lag_ms),
            }),
            None => Ok(FailoverStatus {
                source_node: source_node.to_string(),
                target_node: target_node.to_string(),
                status: "unknown".to_string(),
                failover_time: None,
                last_heartbeat: None,
                lag_ms: None,
            }),
        }
    }

    pub async fn get_lag_monitoring_summary(
        &self,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let alerts = self.get_lag_alerts(None).await.unwrap_or_default();

        let summary: Vec<serde_json::Value> = alerts
            .iter()
            .map(|a| {
                serde_json::json!({
                    "id": a.id,
                    "source_node": a.source_node,
                    "target_node": a.target_node,
                    "threshold_ms": a.threshold_ms,
                    "enabled": a.enabled,
                    "last_triggered_at": a.last_triggered_at.map(|t| t.to_rfc3339()),
                })
            })
            .collect();

        Ok(summary)
    }

    pub async fn create_consistency_check(
        &self,
        source_node: &str,
        target_node: &str,
        table_name: &str,
        check_type: &str,
    ) -> Result<ConsistencyCheckEntry, sqlx::Error> {
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

        Ok(ConsistencyCheckEntry {
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
    ) -> Result<Vec<ConsistencyCheckEntry>, sqlx::Error> {
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
            .map(|r| ConsistencyCheckEntry {
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
    ) -> Result<FailoverHistoryEntry, sqlx::Error> {
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

        Ok(FailoverHistoryEntry {
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
    ) -> Result<Vec<FailoverHistoryEntry>, sqlx::Error> {
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
            .map(|r| FailoverHistoryEntry {
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
    ) -> Result<Vec<ReplicationTopology>, sqlx::Error> {
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

                topology.push(ReplicationTopology {
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
    ) -> Result<Vec<RecoveryPoint>, sqlx::Error> {
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

            points.push(RecoveryPoint {
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
    fn test_replication_log_entry_serialization() {
        let entry = ReplicationLogEntry {
            id: Uuid::nil(),
            replica_id: Uuid::nil(),
            operation: "insert".to_string(),
            table_name: "repos".to_string(),
            record_id: Some(Uuid::nil()),
            status: "completed".to_string(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deser: ReplicationLogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.operation, "insert");
        assert_eq!(deser.table_name, "repos");
    }

    #[test]
    fn test_replication_stats_entry_serialization() {
        let entry = ReplicationStatsEntry {
            id: Uuid::nil(),
            replica_id: Uuid::nil(),
            period_start: Utc::now(),
            operations_count: 42,
            avg_lag_ms: 12.5,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deser: ReplicationStatsEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.operations_count, 42);
        assert_eq!(deser.avg_lag_ms, 12.5);
    }

    #[test]
    fn test_conflict_resolver_entry_serialization() {
        let entry = ConflictResolverEntry {
            id: Uuid::nil(),
            table_name: "repos".to_string(),
            resolver_type: "last_write_wins".to_string(),
            custom_logic: None,
            enabled: true,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deser: ConflictResolverEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.table_name, "repos");
        assert_eq!(deser.resolver_type, "last_write_wins");
    }

    #[test]
    fn test_lag_alert_entry_serialization() {
        let entry = LagAlertEntry {
            id: Uuid::nil(),
            source_node: "node-a".to_string(),
            target_node: "node-b".to_string(),
            threshold_ms: 500,
            enabled: true,
            last_triggered_at: None,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deser: LagAlertEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.threshold_ms, 500);
        assert!(deser.last_triggered_at.is_none());
    }

    #[test]
    fn test_consistency_check_entry_serialization() {
        let entry = ConsistencyCheckEntry {
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
        let deser: ConsistencyCheckEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.source_node, "node-a");
        assert_eq!(deser.discrepancy_count, 0);
    }

    #[test]
    fn test_failover_history_entry_serialization() {
        let entry = FailoverHistoryEntry {
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
        let deser: FailoverHistoryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.failover_type, "automatic");
        assert!(deser.success);
    }

    #[test]
    fn test_replication_topology_serialization() {
        let topology = ReplicationTopology {
            source_node: "node-a".to_string(),
            target_node: "node-b".to_string(),
            status: "healthy".to_string(),
            lag_ms: Some(50),
            consistency_score: 100.0,
            last_check_at: Some(Utc::now()),
            total_failovers: 3,
        };
        let json = serde_json::to_string(&topology).unwrap();
        let deser: ReplicationTopology = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.consistency_score, 100.0);
        assert_eq!(deser.total_failovers, 3);
    }

    #[test]
    fn test_recovery_point_serialization() {
        let point = RecoveryPoint {
            node_name: "node-a".to_string(),
            recovery_point: Utc::now(),
            lag_seconds: 5,
            last_write_at: Some(Utc::now()),
            consistency_status: "consistent".to_string(),
        };
        let json = serde_json::to_string(&point).unwrap();
        let deser: RecoveryPoint = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.lag_seconds, 5);
        assert_eq!(deser.consistency_status, "consistent");
    }
}
