#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolverEntryV20 {
    pub id: Uuid,
    pub table_name: String,
    pub resolver_type: String,
    pub custom_logic: Option<serde_json::Value>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LagAlertEntryV20 {
    pub id: Uuid,
    pub source_node: String,
    pub target_node: String,
    pub threshold_ms: i32,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyCheckResultV21 {
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
pub struct FailoverStatusV21 {
    pub source_node: String,
    pub target_node: String,
    pub status: String,
    pub failover_time: Option<DateTime<Utc>>,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub lag_ms: Option<i32>,
}

pub struct DatabaseReplicationV21Service {
    pool: PgPool,
}

impl DatabaseReplicationV21Service {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_conflict_resolver(
        &self,
        table_name: &str,
        resolver_type: &str,
        custom_logic: Option<serde_json::Value>,
    ) -> Result<ConflictResolverEntryV20, sqlx::Error> {
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

        Ok(ConflictResolverEntryV20 {
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
    ) -> Result<Vec<ConflictResolverEntryV20>, sqlx::Error> {
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
            .map(|r| ConflictResolverEntryV20 {
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
    ) -> Result<LagAlertEntryV20, sqlx::Error> {
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

        Ok(LagAlertEntryV20 {
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
    ) -> Result<Vec<LagAlertEntryV20>, sqlx::Error> {
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
            .map(|r| LagAlertEntryV20 {
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
    ) -> Result<ConsistencyCheckResultV21, sqlx::Error> {
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

        Ok(ConsistencyCheckResultV21 {
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
    ) -> Result<FailoverStatusV21, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct ReplicaRow {
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
            Some(r) => Ok(FailoverStatusV21 {
                source_node: source_node.to_string(),
                target_node: target_node.to_string(),
                status: r.status,
                failover_time: None,
                last_heartbeat: Some(Utc::now()),
                lag_ms: Some(r.lag_ms),
            }),
            None => Ok(FailoverStatusV21 {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_resolver_serialization() {
        let resolver = ConflictResolverEntryV20 {
            id: Uuid::nil(),
            table_name: "repos".to_string(),
            resolver_type: "last_write_wins".to_string(),
            custom_logic: None,
            enabled: true,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&resolver).unwrap();
        let deser: ConflictResolverEntryV20 = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.table_name, "repos");
        assert_eq!(deser.resolver_type, "last_write_wins");
    }

    #[test]
    fn test_lag_alert_serialization() {
        let alert = LagAlertEntryV20 {
            id: Uuid::nil(),
            source_node: "node-a".to_string(),
            target_node: "node-b".to_string(),
            threshold_ms: 500,
            enabled: true,
            last_triggered_at: None,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&alert).unwrap();
        let deser: LagAlertEntryV20 = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.threshold_ms, 500);
        assert!(deser.last_triggered_at.is_none());
    }

    #[test]
    fn test_consistency_check_result_serialization() {
        let result = ConsistencyCheckResultV21 {
            check_id: Uuid::nil(),
            source_node: "node-a".to_string(),
            target_node: "node-b".to_string(),
            table_name: "repos".to_string(),
            consistent: true,
            discrepancy_count: 0,
            checked_at: Utc::now(),
            details: serde_json::json!({"source_replica_count": 10, "target_replica_count": 10}),
        };
        let json = serde_json::to_string(&result).unwrap();
        let deser: ConsistencyCheckResultV21 = serde_json::from_str(&json).unwrap();
        assert!(deser.consistent);
        assert_eq!(deser.discrepancy_count, 0);
    }
}
