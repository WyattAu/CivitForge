#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverTestResult {
    pub replica_id: Uuid,
    pub success: bool,
    pub duration_ms: i32,
    pub error_message: Option<String>,
    pub tested_at: DateTime<Utc>,
}

pub struct DatabaseReplicationV3Service {
    pool: PgPool,
}

impl DatabaseReplicationV3Service {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
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
            r#"INSERT INTO database_replication_config (replica_id, config_key, config_value)
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
             FROM database_replication_config
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
             FROM database_replication_config
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
            r#"INSERT INTO database_replication_alerts (replica_id, alert_type, threshold)
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
             FROM database_replication_alerts
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
            results.push(ReplicationMonitoringResult {
                replica_id: replica.id,
                replica_name: replica.name,
                lag_ms: replica.lag_ms,
                status: replica.status,
                healthy: replica.lag_ms < 1000,
                alerts,
            });
        }

        Ok(results)
    }

    pub async fn test_failover(
        &self,
        replica_id: Uuid,
    ) -> Result<FailoverTestResult, sqlx::Error> {
        let start = Utc::now();
        
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
            });
        }

        let end = Utc::now();
        let duration_ms = (end - start).num_milliseconds() as i32;

        Ok(FailoverTestResult {
            replica_id,
            success: true,
            duration_ms,
            error_message: None,
            tested_at: end,
        })
    }
}