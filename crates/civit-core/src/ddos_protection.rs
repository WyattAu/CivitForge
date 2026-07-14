#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdosProtection {
    pub id: Uuid,
    pub name: String,
    pub enabled: bool,
    pub threshold_rps: i32,
    pub threshold_bps: i64,
    pub action: String,
    pub duration_seconds: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDdosProtection {
    pub name: String,
    pub enabled: Option<bool>,
    pub threshold_rps: Option<i32>,
    pub threshold_bps: Option<i64>,
    pub action: Option<String>,
    pub duration_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDdosProtection {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub threshold_rps: Option<i32>,
    pub threshold_bps: Option<i64>,
    pub action: Option<String>,
    pub duration_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdosEvent {
    pub id: Uuid,
    pub protection_id: Uuid,
    pub source_ip: String,
    pub request_rate: f64,
    pub action_taken: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficSample {
    pub source_ip: String,
    pub requests_per_second: f64,
    pub bytes_per_second: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdosMitigation {
    pub triggered: bool,
    pub protection_id: Option<Uuid>,
    pub action_taken: String,
    pub blocked_ip: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct ProtectionRow {
    id: Uuid,
    name: String,
    enabled: bool,
    threshold_rps: i32,
    threshold_bps: i64,
    action: String,
    duration_seconds: i32,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<ProtectionRow> for DdosProtection {
    fn from(row: ProtectionRow) -> Self {
        DdosProtection {
            id: row.id,
            name: row.name,
            enabled: row.enabled,
            threshold_rps: row.threshold_rps,
            threshold_bps: row.threshold_bps,
            action: row.action,
            duration_seconds: row.duration_seconds,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct EventRow {
    id: Uuid,
    protection_id: Uuid,
    source_ip: String,
    request_rate: f64,
    action_taken: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<EventRow> for DdosEvent {
    fn from(row: EventRow) -> Self {
        DdosEvent {
            id: row.id,
            protection_id: row.protection_id,
            source_ip: row.source_ip,
            request_rate: row.request_rate,
            action_taken: row.action_taken,
            created_at: row.created_at,
        }
    }
}

pub struct DdosProtectionService {
    pool: PgPool,
}

impl DdosProtectionService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_protection(
        &self,
        input: CreateDdosProtection,
    ) -> Result<DdosProtection, sqlx::Error> {
        let row = sqlx::query_as::<_, ProtectionRow>(
            r#"INSERT INTO ddos_protection (name, enabled, threshold_rps, threshold_bps, action, duration_seconds)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, name, enabled, threshold_rps, threshold_bps, action, duration_seconds, created_at"#,
        )
        .bind(&input.name)
        .bind(input.enabled.unwrap_or(true))
        .bind(input.threshold_rps.unwrap_or(1000))
        .bind(input.threshold_bps.unwrap_or(1_000_000_000))
        .bind(input.action.as_deref().unwrap_or("block"))
        .bind(input.duration_seconds.unwrap_or(300))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_protection(
        &self,
        id: Uuid,
    ) -> Result<Option<DdosProtection>, sqlx::Error> {
        let row = sqlx::query_as::<_, ProtectionRow>(
            r#"SELECT id, name, enabled, threshold_rps, threshold_bps, action, duration_seconds, created_at
             FROM ddos_protection WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_protections(&self) -> Result<Vec<DdosProtection>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ProtectionRow>(
            r#"SELECT id, name, enabled, threshold_rps, threshold_bps, action, duration_seconds, created_at
             FROM ddos_protection ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_protection(
        &self,
        id: Uuid,
        input: UpdateDdosProtection,
    ) -> Result<DdosProtection, sqlx::Error> {
        let row = sqlx::query_as::<_, ProtectionRow>(
            r#"UPDATE ddos_protection SET
             name = COALESCE($2, name),
             enabled = COALESCE($3, enabled),
             threshold_rps = COALESCE($4, threshold_rps),
             threshold_bps = COALESCE($5, threshold_bps),
             action = COALESCE($6, action),
             duration_seconds = COALESCE($7, duration_seconds)
             WHERE id = $1
             RETURNING id, name, enabled, threshold_rps, threshold_bps, action, duration_seconds, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(input.enabled)
        .bind(input.threshold_rps)
        .bind(input.threshold_bps)
        .bind(&input.action)
        .bind(input.duration_seconds)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_protection(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM ddos_protection WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn monitor_traffic(
        &self,
        sample: &TrafficSample,
    ) -> Result<DdosMitigation, sqlx::Error> {
        let protections = sqlx::query_as::<_, ProtectionRow>(
            r#"SELECT id, name, enabled, threshold_rps, threshold_bps, action, duration_seconds, created_at
             FROM ddos_protection WHERE enabled = true"#,
        )
        .fetch_all(&self.pool)
        .await?;

        for prot in protections {
            let rps_exceeded = sample.requests_per_second >= prot.threshold_rps as f64;
            let bps_exceeded = sample.bytes_per_second >= prot.threshold_bps;

            if rps_exceeded || bps_exceeded {
                self.log_event(prot.id, &sample.source_ip, sample.requests_per_second, &prot.action)
                    .await?;

                return Ok(DdosMitigation {
                    triggered: true,
                    protection_id: Some(prot.id),
                    action_taken: prot.action,
                    blocked_ip: Some(sample.source_ip.clone()),
                });
            }
        }

        Ok(DdosMitigation {
            triggered: false,
            protection_id: None,
            action_taken: "none".into(),
            blocked_ip: None,
        })
    }

    pub async fn log_event(
        &self,
        protection_id: Uuid,
        source_ip: &str,
        request_rate: f64,
        action_taken: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO ddos_events (protection_id, source_ip, request_rate, action_taken)
             VALUES ($1, $2, $3, $4)"#,
        )
        .bind(protection_id)
        .bind(source_ip)
        .bind(request_rate)
        .bind(action_taken)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_events(
        &self,
        protection_id: Option<Uuid>,
    ) -> Result<Vec<DdosEvent>, sqlx::Error> {
        let rows = if let Some(pid) = protection_id {
            sqlx::query_as::<_, EventRow>(
                r#"SELECT id, protection_id, source_ip, request_rate, action_taken, created_at
                 FROM ddos_events WHERE protection_id = $1 ORDER BY created_at DESC"#,
            )
            .bind(pid)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, EventRow>(
                r#"SELECT id, protection_id, source_ip, request_rate, action_taken, created_at
                 FROM ddos_events ORDER BY created_at DESC"#,
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn detect_and_mitigate(
        &self,
        source_ip: &str,
        current_rps: f64,
        current_bps: i64,
    ) -> Result<DdosMitigation, sqlx::Error> {
        self.monitor_traffic(&TrafficSample {
            source_ip: source_ip.to_string(),
            requests_per_second: current_rps,
            bytes_per_second: current_bps,
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protection_serialization() {
        let prot = DdosProtection {
            id: Uuid::new_v4(),
            name: "default".into(),
            enabled: true,
            threshold_rps: 1000,
            threshold_bps: 1_000_000_000,
            action: "block".into(),
            duration_seconds: 300,
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&prot).unwrap();
        assert!(json.contains("default"));
        assert!(json.contains("block"));
    }

    #[test]
    fn test_event_serialization() {
        let event = DdosEvent {
            id: Uuid::new_v4(),
            protection_id: Uuid::new_v4(),
            source_ip: "1.2.3.4".into(),
            request_rate: 5000.0,
            action_taken: "block".into(),
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("1.2.3.4"));
    }

    #[test]
    fn test_traffic_sample() {
        let sample = TrafficSample {
            source_ip: "10.0.0.1".into(),
            requests_per_second: 2000.0,
            bytes_per_second: 5_000_000_000,
        };
        assert_eq!(sample.requests_per_second, 2000.0);
        assert_eq!(sample.bytes_per_second, 5_000_000_000);
    }

    #[test]
    fn test_mitigation_result() {
        let mit = DdosMitigation {
            triggered: true,
            protection_id: Some(Uuid::new_v4()),
            action_taken: "block".into(),
            blocked_ip: Some("1.2.3.4".into()),
        };
        assert!(mit.triggered);
        assert!(mit.blocked_ip.is_some());
    }
}
