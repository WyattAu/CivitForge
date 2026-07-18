use crate::coordination::AssignmentStatus;
use crate::migration::{MigrationPhase, MigrationState};
use crate::router::{ShardConfig, ShardRouter};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShardConfigError {
    #[error("shard not found: {0}")]
    ShardNotFound(String),

    #[error("resharding already in progress")]
    ReshardInProgress,

    #[error("insufficient healthy shards: need {need}, have {have}")]
    InsufficientShards { need: usize, have: usize },

    #[error("invalid shard count: {0}")]
    InvalidShardCount(u32),

    #[error("migration error: {0}")]
    Migration(String),
}

/// Top-level shard configuration for the entire cluster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterShardConfig {
    /// Number of logical shards.
    pub shard_count: u32,
    /// Maps shard IDs to their connection/endpoint info.
    pub shard_map: HashMap<String, ShardConfig>,
    /// Replication factor per shard (1 = no replication).
    pub replication_factor: u32,
    /// Vnodes per shard for consistent hashing.
    pub vnodes_per_shard: u32,
    /// Health check interval.
    pub health_check_interval: Duration,
    /// Maximum acceptable replication lag before routing away.
    pub max_replication_lag: Duration,
}

impl Default for ClusterShardConfig {
    fn default() -> Self {
        Self {
            shard_count: 4,
            shard_map: HashMap::new(),
            replication_factor: 1,
            vnodes_per_shard: 256,
            health_check_interval: Duration::from_secs(10),
            max_replication_lag: Duration::from_secs(10),
        }
    }
}

impl ClusterShardConfig {
    /// Create a new cluster configuration with the given number of shards.
    pub fn new(shard_count: u32) -> Self {
        let mut shard_map = HashMap::new();
        for i in 0..shard_count {
            let id = format!("shard-{i}");
            let url = format!("postgres://localhost:{}/civitforge_shard_{i}", 5432 + i);
            shard_map.insert(id.clone(), ShardConfig::new(id, url));
        }
        Self {
            shard_count,
            shard_map,
            ..Default::default()
        }
    }

    /// Add a shard to the cluster.
    pub fn add_shard(&mut self, shard: ShardConfig) {
        self.shard_map.insert(shard.id.clone(), shard);
        self.shard_count = self.shard_map.len() as u32;
    }

    /// Remove a shard from the cluster.
    pub fn remove_shard(&mut self, shard_id: &str) -> Option<ShardConfig> {
        let removed = self.shard_map.remove(shard_id);
        if removed.is_some() {
            self.shard_count = self.shard_map.len() as u32;
        }
        removed
    }

    /// Get all shard IDs.
    pub fn shard_ids(&self) -> Vec<&str> {
        self.shard_map.keys().map(|s| s.as_str()).collect()
    }

    /// Build a ShardRouter from this configuration.
    pub fn build_router(&self) -> ShardRouter {
        let shards: Vec<ShardConfig> = self.shard_map.values().cloned().collect();
        ShardRouter::new(shards, self.vnodes_per_shard)
    }
}

/// Monitors the health of individual shards in the cluster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardHealth {
    /// Shard ID being monitored.
    pub shard_id: String,
    /// Current health status.
    pub status: ShardHealthStatus,
    /// Last successful health check timestamp.
    pub last_healthy_at: Option<DateTime<Utc>>,
    /// Number of consecutive failed health checks.
    pub consecutive_failures: u32,
    /// Average query latency (ms) over the monitoring window.
    pub avg_latency_ms: f64,
    /// Current replication lag (if this is a replica).
    pub replication_lag: Option<Duration>,
    /// Disk usage percentage.
    pub disk_usage_pct: Option<f64>,
    /// Connection pool utilization percentage.
    pub connection_pool_pct: Option<f64>,
    /// Error rate (errors per second).
    pub error_rate: f64,
    /// When this health record was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Health status of a shard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShardHealthStatus {
    /// Shard is healthy and accepting traffic.
    Healthy,
    /// Shard is degraded (high latency, partial failures).
    Degraded,
    /// Shard is unhealthy and should not receive traffic.
    Unhealthy,
    /// Shard health is unknown (not yet checked).
    Unknown,
}

impl std::fmt::Display for ShardHealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShardHealthStatus::Healthy => write!(f, "healthy"),
            ShardHealthStatus::Degraded => write!(f, "degraded"),
            ShardHealthStatus::Unhealthy => write!(f, "unhealthy"),
            ShardHealthStatus::Unknown => write!(f, "unknown"),
        }
    }
}

impl ShardHealth {
    /// Create a new health record for a shard.
    pub fn new(shard_id: impl Into<String>) -> Self {
        Self {
            shard_id: shard_id.into(),
            status: ShardHealthStatus::Unknown,
            last_healthy_at: None,
            consecutive_failures: 0,
            avg_latency_ms: 0.0,
            replication_lag: None,
            disk_usage_pct: None,
            connection_pool_pct: None,
            error_rate: 0.0,
            updated_at: Utc::now(),
        }
    }

    /// Record a successful health check.
    pub fn record_success(&mut self, latency: Duration) {
        self.consecutive_failures = 0;
        self.last_healthy_at = Some(Utc::now());
        self.avg_latency_ms = latency.as_millis() as f64;
        self.status = if self.avg_latency_ms < 100.0 {
            ShardHealthStatus::Healthy
        } else {
            ShardHealthStatus::Degraded
        };
        self.updated_at = Utc::now();
    }

    /// Record a failed health check.
    pub fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        self.status = if self.consecutive_failures >= 3 {
            ShardHealthStatus::Unhealthy
        } else {
            ShardHealthStatus::Degraded
        };
        self.updated_at = Utc::now();
    }

    /// Record replication lag measurement.
    pub fn set_replication_lag(&mut self, lag: Duration) {
        self.replication_lag = Some(lag);
        if lag > Duration::from_secs(10) {
            self.status = ShardHealthStatus::Degraded;
        }
        self.updated_at = Utc::now();
    }

    /// Record disk usage.
    pub fn set_disk_usage(&mut self, pct: f64) {
        self.disk_usage_pct = Some(pct);
        if pct > 95.0 {
            self.status = ShardHealthStatus::Unhealthy;
        } else if pct > 80.0 {
            self.status = ShardHealthStatus::Degraded;
        }
        self.updated_at = Utc::now();
    }

    /// Record connection pool utilization.
    pub fn set_connection_pool_usage(&mut self, pct: f64) {
        self.connection_pool_pct = Some(pct);
        if pct > 90.0 {
            self.status = ShardHealthStatus::Degraded;
        }
        self.updated_at = Utc::now();
    }

    /// Record error rate.
    pub fn set_error_rate(&mut self, rate: f64) {
        self.error_rate = rate;
        if rate > 1.0 {
            self.status = ShardHealthStatus::Unhealthy;
        } else if rate > 0.1 {
            self.status = ShardHealthStatus::Degraded;
        }
        self.updated_at = Utc::now();
    }

    /// Returns true if the shard should receive traffic.
    pub fn is_usable(&self) -> bool {
        self.status == ShardHealthStatus::Healthy || self.status == ShardHealthStatus::Degraded
    }

    /// Returns true if replication lag exceeds the threshold.
    pub fn is_lagging(&self, max_lag: Duration) -> bool {
        self.replication_lag
            .map(|lag| lag > max_lag)
            .unwrap_or(false)
    }
}

/// Tracks health for all shards in the cluster.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShardHealthMap {
    shards: HashMap<String, ShardHealth>,
}

impl ShardHealthMap {
    pub fn new() -> Self {
        Self {
            shards: HashMap::new(),
        }
    }

    /// Get or create health record for a shard.
    pub fn get_or_create(&mut self, shard_id: &str) -> &mut ShardHealth {
        self.shards
            .entry(shard_id.to_string())
            .or_insert_with(|| ShardHealth::new(shard_id))
    }

    /// Get health for a shard.
    pub fn get(&self, shard_id: &str) -> Option<&ShardHealth> {
        self.shards.get(shard_id)
    }

    /// Get all unhealthy shard IDs.
    pub fn unhealthy_shards(&self) -> Vec<&str> {
        self.shards
            .iter()
            .filter(|(_, h)| h.status == ShardHealthStatus::Unhealthy)
            .map(|(id, _)| id.as_str())
            .collect()
    }

    /// Get all healthy shard IDs.
    pub fn healthy_shards(&self) -> Vec<&str> {
        self.shards
            .iter()
            .filter(|(_, h)| h.is_usable())
            .map(|(id, _)| id.as_str())
            .collect()
    }

    /// Update the ShardRouter based on current health status.
    pub fn apply_to_router(&self, router: &mut ShardRouter) {
        for (id, health) in &self.shards {
            if health.status == ShardHealthStatus::Unhealthy {
                let _ = router.mark_shard_unhealthy(id);
            } else if health.is_usable() {
                let _ = router.mark_shard_healthy(id);
            }
        }
    }
}

/// Represents an ongoing or completed resharding operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReshardOperation {
    /// Unique operation ID.
    pub id: String,
    /// Source shard count.
    pub from_shard_count: u32,
    /// Target shard count.
    pub to_shard_count: u32,
    /// Current migration state.
    pub migration_state: MigrationState,
    /// Mapping of repository IDs to their source and target shards.
    pub shard_map: HashMap<String, ReshardMapping>,
    /// When the operation started.
    pub started_at: DateTime<Utc>,
    /// When the operation completed (if it has).
    pub completed_at: Option<DateTime<Utc>>,
    /// Whether the operation is paused.
    pub is_paused: bool,
    /// Operator who initiated the reshard.
    pub initiated_by: String,
}

/// Mapping for a single repository during resharding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReshardMapping {
    /// Repository ID being moved.
    pub repo_id: String,
    /// Source shard ID.
    pub source_shard: String,
    /// Target shard ID.
    pub target_shard: String,
    /// Current status of this repo's migration.
    pub status: AssignmentStatus,
    /// When data copy started.
    pub copy_started_at: Option<DateTime<Utc>>,
    /// When data copy completed.
    pub copy_completed_at: Option<DateTime<Utc>>,
    /// Error message if copy failed.
    pub error: Option<String>,
}

impl ReshardOperation {
    /// Create a new resharding operation.
    pub fn new(
        id: impl Into<String>,
        from_shard_count: u32,
        to_shard_count: u32,
        total_repos: u64,
        initiated_by: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            from_shard_count,
            to_shard_count,
            migration_state: MigrationState::new(total_repos),
            shard_map: HashMap::new(),
            started_at: Utc::now(),
            completed_at: None,
            is_paused: false,
            initiated_by: initiated_by.into(),
        }
    }

    /// Add a repository mapping to this operation.
    pub fn add_mapping(&mut self, mapping: ReshardMapping) {
        self.shard_map.insert(mapping.repo_id.clone(), mapping);
    }

    /// Get the current phase of the resharding.
    pub fn current_phase(&self) -> MigrationPhase {
        self.migration_state.current_phase
    }

    /// Progress fraction (0.0 to 1.0).
    pub fn progress(&self) -> f64 {
        self.migration_state.progress
    }

    /// Returns true if the operation is complete.
    pub fn is_complete(&self) -> bool {
        self.completed_at.is_some()
    }

    /// Mark the operation as complete.
    pub fn complete(&mut self) {
        self.migration_state.complete();
        self.completed_at = Some(Utc::now());
    }

    /// Pause the operation.
    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    /// Resume the operation.
    pub fn resume(&mut self) {
        self.is_paused = false;
    }

    /// Get repos pending migration.
    pub fn pending_repos(&self) -> Vec<&ReshardMapping> {
        self.shard_map
            .values()
            .filter(|m| m.status == AssignmentStatus::Pending)
            .collect()
    }

    /// Get repos currently being migrated.
    pub fn migrating_repos(&self) -> Vec<&ReshardMapping> {
        self.shard_map
            .values()
            .filter(|m| m.status == AssignmentStatus::Migrating)
            .collect()
    }

    /// Get repos that failed migration.
    pub fn failed_repos(&self) -> Vec<&ReshardMapping> {
        self.shard_map
            .values()
            .filter(|m| m.error.is_some())
            .collect()
    }
}

/// Manages online resharding operations with safety checks.
pub struct ReshardManager {
    active_operation: Option<ReshardOperation>,
    completed_operations: Vec<ReshardOperation>,
}

impl ReshardManager {
    pub fn new() -> Self {
        Self {
            active_operation: None,
            completed_operations: Vec::new(),
        }
    }

    /// Start a new resharding operation.
    pub fn start_operation(
        &mut self,
        operation: ReshardOperation,
    ) -> Result<(), ShardConfigError> {
        if self.active_operation.is_some() {
            return Err(ShardConfigError::ReshardInProgress);
        }
        self.active_operation = Some(operation);
        Ok(())
    }

    /// Get the currently active operation.
    pub fn active_operation(&self) -> Option<&ReshardOperation> {
        self.active_operation.as_ref()
    }

    /// Get a mutable reference to the active operation.
    pub fn active_operation_mut(&mut self) -> Option<&mut ReshardOperation> {
        self.active_operation.as_mut()
    }

    /// Complete the current operation and archive it.
    pub fn complete_operation(&mut self) -> Result<(), ShardConfigError> {
        let mut op = self
            .active_operation
            .take()
            .ok_or(ShardConfigError::ReshardInProgress)?; // using as "no operation"

        op.complete();
        self.completed_operations.push(op);
        Ok(())
    }

    /// Cancel the current operation.
    pub fn cancel_operation(&mut self) -> Result<(), ShardConfigError> {
        self.active_operation
            .take()
            .ok_or(ShardConfigError::ReshardInProgress)?;
        Ok(())
    }

    /// Get list of completed operations.
    pub fn completed_operations(&self) -> &[ReshardOperation] {
        &self.completed_operations
    }
}

impl Default for ReshardManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_shard_config_default() {
        let config = ClusterShardConfig::default();
        assert_eq!(config.shard_count, 4);
        assert_eq!(config.replication_factor, 1);
        assert_eq!(config.vnodes_per_shard, 256);
    }

    #[test]
    fn test_cluster_shard_config_new() {
        let config = ClusterShardConfig::new(6);
        assert_eq!(config.shard_count, 6);
        assert_eq!(config.shard_map.len(), 6);
        for i in 0..6 {
            assert!(config.shard_map.contains_key(&format!("shard-{i}")));
        }
    }

    #[test]
    fn test_cluster_add_remove_shard() {
        let mut config = ClusterShardConfig::new(2);
        assert_eq!(config.shard_count, 2);

        config.add_shard(ShardConfig::new("shard-new", "postgres://localhost/new"));
        assert_eq!(config.shard_count, 3);

        config.remove_shard("shard-new");
        assert_eq!(config.shard_count, 2);
        assert!(!config.shard_map.contains_key("shard-new"));
    }

    #[test]
    fn test_cluster_shard_ids() {
        let config = ClusterShardConfig::new(3);
        let ids = config.shard_ids();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&"shard-0"));
        assert!(ids.contains(&"shard-1"));
        assert!(ids.contains(&"shard-2"));
    }

    #[test]
    fn test_build_router() {
        let config = ClusterShardConfig::new(4);
        let router = config.build_router();
        assert_eq!(router.healthy_shard_count(), 4);
    }

    #[test]
    fn test_shard_health_new() {
        let health = ShardHealth::new("shard-0");
        assert_eq!(health.shard_id, "shard-0");
        assert_eq!(health.status, ShardHealthStatus::Unknown);
        assert_eq!(health.consecutive_failures, 0);
    }

    #[test]
    fn test_shard_health_success() {
        let mut health = ShardHealth::new("s0");
        health.record_success(Duration::from_millis(50));
        assert_eq!(health.status, ShardHealthStatus::Healthy);
        assert_eq!(health.consecutive_failures, 0);
        assert!(health.last_healthy_at.is_some());
    }

    #[test]
    fn test_shard_health_degraded_latency() {
        let mut health = ShardHealth::new("s0");
        health.record_success(Duration::from_millis(200));
        assert_eq!(health.status, ShardHealthStatus::Degraded);
    }

    #[test]
    fn test_shard_health_failure() {
        let mut health = ShardHealth::new("s0");
        health.record_failure();
        assert_eq!(health.status, ShardHealthStatus::Degraded);
        assert_eq!(health.consecutive_failures, 1);

        health.record_failure();
        health.record_failure();
        assert_eq!(health.status, ShardHealthStatus::Unhealthy);
        assert_eq!(health.consecutive_failures, 3);
    }

    #[test]
    fn test_shard_health_replication_lag() {
        let mut health = ShardHealth::new("s0");
        health.record_success(Duration::from_millis(10));
        assert_eq!(health.status, ShardHealthStatus::Healthy);

        health.set_replication_lag(Duration::from_secs(15));
        assert_eq!(health.status, ShardHealthStatus::Degraded);
    }

    #[test]
    fn test_shard_health_disk_usage() {
        let mut health = ShardHealth::new("s0");
        health.set_disk_usage(50.0);
        assert_eq!(health.status, ShardHealthStatus::Healthy);

        health.set_disk_usage(85.0);
        assert_eq!(health.status, ShardHealthStatus::Degraded);

        health.set_disk_usage(98.0);
        assert_eq!(health.status, ShardHealthStatus::Unhealthy);
    }

    #[test]
    fn test_shard_health_error_rate() {
        let mut health = ShardHealth::new("s0");
        health.set_error_rate(0.05);
        assert_eq!(health.status, ShardHealthStatus::Healthy);

        health.set_error_rate(0.5);
        assert_eq!(health.status, ShardHealthStatus::Degraded);

        health.set_error_rate(2.0);
        assert_eq!(health.status, ShardHealthStatus::Unhealthy);
    }

    #[test]
    fn test_shard_health_is_usable() {
        let mut health = ShardHealth::new("s0");
        health.status = ShardHealthStatus::Healthy;
        assert!(health.is_usable());

        health.status = ShardHealthStatus::Degraded;
        assert!(health.is_usable());

        health.status = ShardHealthStatus::Unhealthy;
        assert!(!health.is_usable());

        health.status = ShardHealthStatus::Unknown;
        assert!(!health.is_usable());
    }

    #[test]
    fn test_shard_health_is_lagging() {
        let mut health = ShardHealth::new("s0");
        assert!(!health.is_lagging(Duration::from_secs(10)));

        health.set_replication_lag(Duration::from_secs(5));
        assert!(!health.is_lagging(Duration::from_secs(10)));

        health.set_replication_lag(Duration::from_secs(15));
        assert!(health.is_lagging(Duration::from_secs(10)));
    }

    #[test]
    fn test_shard_health_map() {
        let mut map = ShardHealthMap::new();
        {
            let h = map.get_or_create("shard-0");
            h.record_success(Duration::from_millis(10));
        }
        {
            let h = map.get_or_create("shard-1");
            h.record_failure();
            h.record_failure();
            h.record_failure();
        }

        assert_eq!(map.healthy_shards().len(), 1);
        assert_eq!(map.unhealthy_shards().len(), 1);
    }

    #[test]
    fn test_shard_health_status_display() {
        assert_eq!(ShardHealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(ShardHealthStatus::Degraded.to_string(), "degraded");
        assert_eq!(ShardHealthStatus::Unhealthy.to_string(), "unhealthy");
        assert_eq!(ShardHealthStatus::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_reshard_operation_new() {
        let op = ReshardOperation::new("op-1", 2, 4, 1000, "admin");
        assert_eq!(op.from_shard_count, 2);
        assert_eq!(op.to_shard_count, 4);
        assert_eq!(op.migrated_count(), 0);
        assert!(!op.is_complete());
        assert!(!op.is_paused);
    }

    #[test]
    fn test_reshard_operation_add_mapping() {
        let mut op = ReshardOperation::new("op-1", 2, 4, 100, "admin");
        op.add_mapping(ReshardMapping {
            repo_id: "repo-1".into(),
            source_shard: "shard-0".into(),
            target_shard: "shard-2".into(),
            status: AssignmentStatus::Pending,
            copy_started_at: None,
            copy_completed_at: None,
            error: None,
        });

        assert_eq!(op.pending_repos().len(), 1);
    }

    #[test]
    fn test_reshard_operation_progress() {
        let mut op = ReshardOperation::new("op-1", 2, 4, 100, "admin");
        assert_eq!(op.progress(), 0.0);

        for _ in 0..50 {
            op.migration_state.record_migration();
        }
        assert!((op.progress() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_reshard_operation_pause_resume() {
        let mut op = ReshardOperation::new("op-1", 2, 4, 100, "admin");
        op.pause();
        assert!(op.is_paused);

        op.resume();
        assert!(!op.is_paused);
    }

    #[test]
    fn test_reshard_operation_complete() {
        let mut op = ReshardOperation::new("op-1", 2, 4, 100, "admin");
        op.complete();
        assert!(op.is_complete());
        assert!(op.completed_at.is_some());
    }

    #[test]
    fn test_reshard_manager_lifecycle() {
        let mut manager = ReshardManager::new();
        assert!(manager.active_operation().is_none());

        let op = ReshardOperation::new("op-1", 2, 4, 100, "admin");
        manager.start_operation(op).unwrap();
        assert!(manager.active_operation().is_some());

        manager.complete_operation().unwrap();
        assert!(manager.active_operation().is_none());
        assert_eq!(manager.completed_operations().len(), 1);
    }

    #[test]
    fn test_reshard_manager_prevents_double_start() {
        let mut manager = ReshardManager::new();
        let op1 = ReshardOperation::new("op-1", 2, 4, 100, "admin");
        let op2 = ReshardOperation::new("op-2", 4, 8, 200, "admin");

        manager.start_operation(op1).unwrap();
        assert!(manager.start_operation(op2).is_err());
    }

    #[test]
    fn test_reshard_manager_cancel() {
        let mut manager = ReshardManager::new();
        let op = ReshardOperation::new("op-1", 2, 4, 100, "admin");
        manager.start_operation(op).unwrap();

        manager.cancel_operation().unwrap();
        assert!(manager.active_operation().is_none());
        assert_eq!(manager.completed_operations().len(), 0);
    }

    #[test]
    fn test_failed_repos() {
        let mut op = ReshardOperation::new("op-1", 2, 4, 100, "admin");
        op.add_mapping(ReshardMapping {
            repo_id: "repo-fail".into(),
            source_shard: "s0".into(),
            target_shard: "s2".into(),
            status: AssignmentStatus::Migrating,
            copy_started_at: Some(Utc::now()),
            copy_completed_at: None,
            error: Some("disk full".into()),
        });

        assert_eq!(op.failed_repos().len(), 1);
        assert_eq!(op.failed_repos()[0].repo_id, "repo-fail");
    }
}
