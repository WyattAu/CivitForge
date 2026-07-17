#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Region {
    Us,
    Eu,
    Apac,
}

impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Region::Us => write!(f, "us"),
            Region::Eu => write!(f, "eu"),
            Region::Apac => write!(f, "apac"),
        }
    }
}

impl std::str::FromStr for Region {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "us" => Ok(Region::Us),
            "eu" => Ok(Region::Eu),
            "apac" => Ok(Region::Apac),
            _ => Err(format!("unknown region: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RegionStatus {
    Healthy,
    Degraded,
    Unavailable,
    Maintenance,
}

impl std::fmt::Display for RegionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegionStatus::Healthy => write!(f, "healthy"),
            RegionStatus::Degraded => write!(f, "degraded"),
            RegionStatus::Unavailable => write!(f, "unavailable"),
            RegionStatus::Maintenance => write!(f, "maintenance"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FailoverStrategy {
    Automatic,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RegionConfig {
    pub id: Uuid,
    pub region: String,
    pub endpoint: String,
    pub status: String,
    pub failover_strategy: String,
    pub data_residency_required: bool,
    pub compliance_frameworks: serde_json::Value,
    pub max_latency_ms: i64,
    pub capacity_weight: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RegionConfig {
    pub fn compliance_frameworks_list(&self) -> Vec<String> {
        self.compliance_frameworks
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReplicationStatus {
    pub id: Uuid,
    pub source_region: String,
    pub target_region: String,
    pub status: String,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub lag_bytes: i64,
    pub lag_seconds: f64,
    pub items_pending: i64,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FailoverRecord {
    pub id: Uuid,
    pub source_region: String,
    pub target_region: String,
    pub reason: String,
    pub status: String,
    pub initiated_by: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ComplianceRule {
    pub id: Uuid,
    pub region: String,
    pub framework: String,
    pub rule_name: String,
    pub rule_description: String,
    pub enabled: bool,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub last_result: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LatencyRoute {
    pub source_region: String,
    pub target_region: String,
    pub latency_ms: f64,
    pub last_measured_at: DateTime<Utc>,
    pub healthy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionOverview {
    pub regions: Vec<RegionConfig>,
    pub replication_links: Vec<ReplicationStatus>,
    pub total_regions: usize,
    pub healthy_regions: usize,
    pub degraded_regions: usize,
    pub unavailable_regions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverRequest {
    pub source_region: String,
    pub target_region: String,
    pub reason: String,
}

pub struct MultiRegionService {
    pool: PgPool,
}

impl MultiRegionService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_region_config(
        &self,
        region: &str,
        endpoint: &str,
        failover_strategy: &str,
        data_residency_required: bool,
        compliance_frameworks: Vec<String>,
        max_latency_ms: u64,
        capacity_weight: f64,
    ) -> Result<RegionConfig, sqlx::Error> {
        let id = Uuid::new_v4();
        let frameworks = serde_json::to_value(&compliance_frameworks).unwrap_or_default();

        let row = sqlx::query_as::<_, RegionConfig>(
            r#"INSERT INTO multi_region_configs
               (id, region, endpoint, status, failover_strategy, data_residency_required,
                compliance_frameworks, max_latency_ms, capacity_weight)
               VALUES ($1, $2, $3, 'healthy', $4, $5, $6, $7, $8)
               RETURNING *"#,
        )
        .bind(id)
        .bind(region)
        .bind(endpoint)
        .bind(failover_strategy)
        .bind(data_residency_required)
        .bind(&frameworks)
        .bind(max_latency_ms as i64)
        .bind(capacity_weight)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_region_config(&self, region: &str) -> Result<Option<RegionConfig>, sqlx::Error> {
        let row = sqlx::query_as::<_, RegionConfig>(
            "SELECT * FROM multi_region_configs WHERE region = $1",
        )
        .bind(region)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list_region_configs(&self) -> Result<Vec<RegionConfig>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RegionConfig>(
            "SELECT * FROM multi_region_configs ORDER BY region",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn update_region_status(
        &self,
        region: &str,
        status: &str,
    ) -> Result<RegionConfig, sqlx::Error> {
        let row = sqlx::query_as::<_, RegionConfig>(
            r#"UPDATE multi_region_configs
               SET status = $2, updated_at = NOW()
               WHERE region = $1
               RETURNING *"#,
        )
        .bind(region)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn create_replication_link(
        &self,
        source_region: &str,
        target_region: &str,
    ) -> Result<ReplicationStatus, sqlx::Error> {
        let id = Uuid::new_v4();

        let row = sqlx::query_as::<_, ReplicationStatus>(
            r#"INSERT INTO multi_region_replication
               (id, source_region, target_region, status, lag_bytes, lag_seconds, items_pending)
               VALUES ($1, $2, $3, 'active', 0, 0.0, 0)
               RETURNING *"#,
        )
        .bind(id)
        .bind(source_region)
        .bind(target_region)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_replication_status(
        &self,
        source_region: Option<&str>,
        target_region: Option<&str>,
    ) -> Result<Vec<ReplicationStatus>, sqlx::Error> {
        let rows = match (source_region, target_region) {
            (Some(s), Some(t)) => {
                sqlx::query_as::<_, ReplicationStatus>(
                    r#"SELECT * FROM multi_region_replication
                       WHERE source_region = $1 AND target_region = $2
                       ORDER BY updated_at DESC"#,
                )
                .bind(s)
                .bind(t)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(s), None) => {
                sqlx::query_as::<_, ReplicationStatus>(
                    r#"SELECT * FROM multi_region_replication
                       WHERE source_region = $1
                       ORDER BY updated_at DESC"#,
                )
                .bind(s)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(t)) => {
                sqlx::query_as::<_, ReplicationStatus>(
                    r#"SELECT * FROM multi_region_replication
                       WHERE target_region = $1
                       ORDER BY updated_at DESC"#,
                )
                .bind(t)
                .fetch_all(&self.pool)
                .await?
            }
            (None, None) => {
                sqlx::query_as::<_, ReplicationStatus>(
                    "SELECT * FROM multi_region_replication ORDER BY updated_at DESC",
                )
                .fetch_all(&self.pool)
                .await?
            }
        };
        Ok(rows)
    }

    pub async fn update_replication_status(
        &self,
        id: Uuid,
        status: &str,
        lag_bytes: i64,
        lag_seconds: f64,
        items_pending: i64,
        error: Option<&str>,
    ) -> Result<ReplicationStatus, sqlx::Error> {
        let row = sqlx::query_as::<_, ReplicationStatus>(
            r#"UPDATE multi_region_replication
               SET status = $2, lag_bytes = $3, lag_seconds = $4, items_pending = $5,
                   error_message = $6, updated_at = NOW(),
                   last_synced_at = CASE WHEN $2 = 'active' THEN NOW() ELSE last_synced_at END
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(status)
        .bind(lag_bytes)
        .bind(lag_seconds)
        .bind(items_pending)
        .bind(error)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn create_failover_record(
        &self,
        source_region: &str,
        target_region: &str,
        reason: &str,
        initiated_by: Uuid,
    ) -> Result<FailoverRecord, sqlx::Error> {
        let id = Uuid::new_v4();

        let row = sqlx::query_as::<_, FailoverRecord>(
            r#"INSERT INTO multi_region_failovers
               (id, source_region, target_region, reason, status, initiated_by)
               VALUES ($1, $2, $3, $4, 'in_progress', $5)
               RETURNING *"#,
        )
        .bind(id)
        .bind(source_region)
        .bind(target_region)
        .bind(reason)
        .bind(initiated_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn complete_failover(
        &self,
        id: Uuid,
        status: &str,
        error: Option<&str>,
    ) -> Result<FailoverRecord, sqlx::Error> {
        let row = sqlx::query_as::<_, FailoverRecord>(
            r#"UPDATE multi_region_failovers
               SET status = $2, completed_at = NOW(), error_message = $3
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(status)
        .bind(error)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn get_failover_history(
        &self,
        source_region: Option<&str>,
        limit: i64,
    ) -> Result<Vec<FailoverRecord>, sqlx::Error> {
        let rows = match source_region {
            Some(s) => {
                sqlx::query_as::<_, FailoverRecord>(
                    r#"SELECT * FROM multi_region_failovers
                       WHERE source_region = $1
                       ORDER BY started_at DESC
                       LIMIT $2"#,
                )
                .bind(s)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, FailoverRecord>(
                    r#"SELECT * FROM multi_region_failovers
                       ORDER BY started_at DESC
                       LIMIT $1"#,
                )
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
        };
        Ok(rows)
    }

    pub async fn create_compliance_rule(
        &self,
        region: &str,
        framework: &str,
        rule_name: &str,
        rule_description: &str,
    ) -> Result<ComplianceRule, sqlx::Error> {
        let id = Uuid::new_v4();

        let row = sqlx::query_as::<_, ComplianceRule>(
            r#"INSERT INTO multi_region_compliance_rules
               (id, region, framework, rule_name, rule_description, enabled)
               VALUES ($1, $2, $3, $4, $5, true)
               RETURNING *"#,
        )
        .bind(id)
        .bind(region)
        .bind(framework)
        .bind(rule_name)
        .bind(rule_description)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_compliance_rules(
        &self,
        region: &str,
        framework: Option<&str>,
    ) -> Result<Vec<ComplianceRule>, sqlx::Error> {
        let rows = match framework {
            Some(fw) => {
                sqlx::query_as::<_, ComplianceRule>(
                    r#"SELECT * FROM multi_region_compliance_rules
                       WHERE region = $1 AND framework = $2
                       ORDER BY rule_name"#,
                )
                .bind(region)
                .bind(fw)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, ComplianceRule>(
                    r#"SELECT * FROM multi_region_compliance_rules
                       WHERE region = $1
                       ORDER BY rule_name"#,
                )
                .bind(region)
                .fetch_all(&self.pool)
                .await?
            }
        };
        Ok(rows)
    }

    pub async fn update_compliance_check(
        &self,
        id: Uuid,
        result: &str,
    ) -> Result<ComplianceRule, sqlx::Error> {
        let row = sqlx::query_as::<_, ComplianceRule>(
            r#"UPDATE multi_region_compliance_rules
               SET last_result = $2, last_checked_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(result)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn record_latency(
        &self,
        source_region: &str,
        target_region: &str,
        latency_ms: f64,
    ) -> Result<LatencyRoute, sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO multi_region_latency_routes
               (source_region, target_region, latency_ms, healthy)
               VALUES ($1, $2, $3, $4)"#,
        )
        .bind(source_region)
        .bind(target_region)
        .bind(latency_ms)
        .bind(latency_ms < 200.0)
        .execute(&self.pool)
        .await?;

        Ok(LatencyRoute {
            source_region: source_region.to_string(),
            target_region: target_region.to_string(),
            latency_ms,
            last_measured_at: Utc::now(),
            healthy: latency_ms < 200.0,
        })
    }

    pub async fn get_best_region(
        &self,
        client_region: &str,
    ) -> Result<String, sqlx::Error> {
        let routes = sqlx::query_as::<_, LatencyRoute>(
            r#"SELECT source_region, target_region, latency_ms, last_measured_at, healthy
               FROM multi_region_latency_routes
               WHERE source_region = $1
               ORDER BY latency_ms ASC
               LIMIT 1"#,
        )
        .bind(client_region)
        .fetch_optional(&self.pool)
        .await?;

        Ok(match routes {
            Some(r) => r.target_region,
            None => client_region.to_string(),
        })
    }

    pub async fn get_region_overview(&self) -> Result<RegionOverview, sqlx::Error> {
        let regions = self.list_region_configs().await?;
        let replication = self.get_replication_status(None, None).await?;

        let total = regions.len();
        let healthy = regions.iter().filter(|r| r.status == "healthy").count();
        let degraded = regions.iter().filter(|r| r.status == "degraded").count();
        let unavailable = regions.iter().filter(|r| r.status == "unavailable").count();

        Ok(RegionOverview {
            regions,
            replication_links: replication,
            total_regions: total,
            healthy_regions: healthy,
            degraded_regions: degraded,
            unavailable_regions: unavailable,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_display() {
        assert_eq!(Region::Us.to_string(), "us");
        assert_eq!(Region::Eu.to_string(), "eu");
        assert_eq!(Region::Apac.to_string(), "apac");
    }

    #[test]
    fn test_region_parse() {
        assert_eq!("us".parse::<Region>().unwrap(), Region::Us);
        assert_eq!("eu".parse::<Region>().unwrap(), Region::Eu);
        assert_eq!("apac".parse::<Region>().unwrap(), Region::Apac);
        assert_eq!("US".parse::<Region>().unwrap(), Region::Us);
        assert!("invalid".parse::<Region>().is_err());
    }

    #[test]
    fn test_region_status_display() {
        assert_eq!(RegionStatus::Healthy.to_string(), "healthy");
        assert_eq!(RegionStatus::Degraded.to_string(), "degraded");
        assert_eq!(RegionStatus::Unavailable.to_string(), "unavailable");
        assert_eq!(RegionStatus::Maintenance.to_string(), "maintenance");
    }

    #[test]
    fn test_region_config_serialization() {
        let config = RegionConfig {
            id: Uuid::new_v4(),
            region: "us".into(),
            endpoint: "https://us.civitforge.com".into(),
            status: "healthy".into(),
            failover_strategy: "automatic".into(),
            data_residency_required: true,
            compliance_frameworks: serde_json::json!(["SOC2", "GDPR"]),
            max_latency_ms: 100,
            capacity_weight: 1.0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"region\":\"us\""));
        assert!(json.contains("\"status\":\"healthy\""));
    }

    #[test]
    fn test_replication_status_serialization() {
        let status = ReplicationStatus {
            id: Uuid::new_v4(),
            source_region: "us".into(),
            target_region: "eu".into(),
            status: "active".into(),
            last_synced_at: Some(Utc::now()),
            lag_bytes: 0,
            lag_seconds: 0.5,
            items_pending: 10,
            error_message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"source_region\":\"us\""));
        assert!(json.contains("\"lag_seconds\":0.5"));
    }

    #[test]
    fn test_failover_request_serialization() {
        let req = FailoverRequest {
            source_region: "us".into(),
            target_region: "eu".into(),
            reason: "region outage".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"reason\":\"region outage\""));
    }

    #[test]
    fn test_latency_route_healthy() {
        let route = LatencyRoute {
            source_region: "us".into(),
            target_region: "eu".into(),
            latency_ms: 50.0,
            last_measured_at: Utc::now(),
            healthy: true,
        };
        assert!(route.healthy);

        let unhealthy = LatencyRoute {
            source_region: "us".into(),
            target_region: "apac".into(),
            latency_ms: 300.0,
            last_measured_at: Utc::now(),
            healthy: false,
        };
        assert!(!unhealthy.healthy);
    }

    #[test]
    fn test_region_overview_serialization() {
        let overview = RegionOverview {
            regions: vec![],
            replication_links: vec![],
            total_regions: 3,
            healthy_regions: 2,
            degraded_regions: 1,
            unavailable_regions: 0,
        };
        let json = serde_json::to_string(&overview).unwrap();
        assert!(json.contains("\"total_regions\":3"));
        assert!(json.contains("\"healthy_regions\":2"));
    }
}
