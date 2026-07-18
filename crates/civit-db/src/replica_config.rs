#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReplicaError {
    #[error("replica not found: {0}")]
    ReplicaNotFound(String),

    #[error("all replicas unavailable")]
    AllReplicasUnavailable,

    #[error("primary failover failed: {0}")]
    FailoverFailed(String),

    #[error("no healthy replicas for read routing")]
    NoHealthyReplicas,

    #[error("configuration error: {0}")]
    Config(String),
}

/// Configuration for a single database replica endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReplicaEndpoint {
    /// Unique identifier for this replica.
    pub id: String,
    /// Connection URL for this replica.
    pub url: String,
    /// Whether this is the primary (write) endpoint.
    pub is_primary: bool,
    /// Geographic region for geo-aware routing.
    pub region: Option<String>,
    /// Maximum number of connections to this replica.
    pub max_connections: u32,
    /// Weight for weighted read routing (higher = more reads).
    pub weight: u32,
}

impl ReplicaEndpoint {
    pub fn new(id: impl Into<String>, url: impl Into<String>, is_primary: bool) -> Self {
        Self {
            id: id.into(),
            url: url.into(),
            is_primary,
            region: None,
            max_connections: 200,
            weight: 1,
        }
    }

    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    pub fn with_max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    pub fn with_weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }
}

impl std::fmt::Display for ReadRoutingStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadRoutingStrategy::RoundRobin => write!(f, "RoundRobin"),
            ReadRoutingStrategy::LeastLag => write!(f, "LeastLag"),
            ReadRoutingStrategy::NearestReplica => write!(f, "NearestReplica"),
            ReadRoutingStrategy::PrimaryOnly => write!(f, "PrimaryOnly"),
            ReadRoutingStrategy::WeightedRoundRobin => write!(f, "WeightedRoundRobin"),
        }
    }
}

/// Routing strategy for read queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadRoutingStrategy {
    /// Round-robin across all healthy replicas.
    RoundRobin,
    /// Route to the replica with lowest replication lag.
    LeastLag,
    /// Route to the replica closest to the reader (by region).
    NearestReplica,
    /// Route reads to the primary (no replica reads).
    PrimaryOnly,
    /// Weighted round-robin based on replica weights.
    WeightedRoundRobin,
}

/// Top-level configuration for primary + replica database setup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaConfig {
    /// The primary (write) endpoint.
    pub primary: ReplicaEndpoint,
    /// Read replicas (excludes primary).
    pub replicas: Vec<ReplicaEndpoint>,
    /// Strategy for routing read queries.
    pub routing: ReadRoutingStrategy,
    /// Maximum acceptable replication lag before routing away.
    pub max_replication_lag: Duration,
    /// Health check interval for replicas.
    pub health_check_interval: Duration,
    /// Failover timeout — how long to wait before declaring primary dead.
    pub failover_timeout: Duration,
    /// Whether automatic failover is enabled.
    pub auto_failover: bool,
}

impl Default for ReplicaConfig {
    fn default() -> Self {
        Self {
            primary: ReplicaEndpoint::new("primary", "postgres://localhost:5432/civitforge", true),
            replicas: Vec::new(),
            routing: ReadRoutingStrategy::RoundRobin,
            max_replication_lag: Duration::from_secs(10),
            health_check_interval: Duration::from_secs(10),
            failover_timeout: Duration::from_secs(30),
            auto_failover: true,
        }
    }
}

impl ReplicaConfig {
    /// Create a configuration with a primary and N read replicas.
    pub fn with_replicas(primary_url: &str, replica_urls: &[&str]) -> Self {
        let primary = ReplicaEndpoint::new("primary", primary_url, true);
        let replicas: Vec<ReplicaEndpoint> = replica_urls
            .iter()
            .enumerate()
            .map(|(i, url)| ReplicaEndpoint::new(format!("replica-{i}"), *url, false))
            .collect();
        Self {
            primary,
            replicas,
            ..Default::default()
        }
    }

    /// Get all endpoint URLs (primary + replicas).
    pub fn all_urls(&self) -> Vec<&str> {
        let mut urls = vec![self.primary.url.as_str()];
        urls.extend(self.replicas.iter().map(|r| r.url.as_str()));
        urls
    }

    /// Get the primary endpoint.
    pub fn primary(&self) -> &ReplicaEndpoint {
        &self.primary
    }

    /// Get all read replica endpoints.
    pub fn read_replicas(&self) -> &[ReplicaEndpoint] {
        &self.replicas
    }

    /// Total endpoint count (primary + replicas).
    pub fn endpoint_count(&self) -> usize {
        1 + self.replicas.len()
    }
}

/// Routes read queries to replicas based on the configured strategy.
pub struct ReadReplicaRouter {
    config: ReplicaConfig,
    round_robin_index: AtomicUsize,
    replica_health: HashMap<String, ReplicaHealth>,
}

/// Health status for a single replica.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaHealth {
    /// Replica endpoint ID.
    pub id: String,
    /// Whether the replica is responding to health checks.
    pub is_healthy: bool,
    /// Current replication lag.
    pub replication_lag: Duration,
    /// Average read latency (ms).
    pub avg_latency_ms: f64,
    /// Number of active connections.
    pub active_connections: u32,
    /// Last successful health check.
    pub last_healthy_at: Option<DateTime<Utc>>,
    /// Number of consecutive failures.
    pub consecutive_failures: u32,
}

impl ReplicaHealth {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            is_healthy: true,
            replication_lag: Duration::ZERO,
            avg_latency_ms: 0.0,
            active_connections: 0,
            last_healthy_at: None,
            consecutive_failures: 0,
        }
    }

    /// Record a successful health check.
    pub fn record_success(&mut self, lag: Duration, latency: Duration) {
        self.is_healthy = true;
        self.replication_lag = lag;
        self.avg_latency_ms = latency.as_millis() as f64;
        self.last_healthy_at = Some(Utc::now());
        self.consecutive_failures = 0;
    }

    /// Record a failed health check.
    pub fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        if self.consecutive_failures >= 3 {
            self.is_healthy = false;
        }
    }
}

impl ReadReplicaRouter {
    /// Create a new router from configuration.
    pub fn new(config: ReplicaConfig) -> Self {
        let mut replica_health = HashMap::new();
        for replica in &config.replicas {
            replica_health.insert(replica.id.clone(), ReplicaHealth::new(&replica.id));
        }
        Self {
            config,
            round_robin_index: AtomicUsize::new(0),
            replica_health,
        }
    }

    /// Route a read query to the best available replica.
    pub fn route_read(&self) -> Result<&ReplicaEndpoint, ReplicaError> {
        match self.config.routing {
            ReadRoutingStrategy::PrimaryOnly => Ok(&self.config.primary),
            ReadRoutingStrategy::RoundRobin => self.route_round_robin(),
            ReadRoutingStrategy::WeightedRoundRobin => self.route_weighted_round_robin(),
            ReadRoutingStrategy::LeastLag => self.route_least_lag(),
            ReadRoutingStrategy::NearestReplica => self.route_nearest(),
        }
    }

    /// Get the primary endpoint for writes.
    pub fn route_write(&self) -> &ReplicaEndpoint {
        &self.config.primary
    }

    /// Get health for a replica.
    pub fn replica_health(&self, id: &str) -> Option<&ReplicaHealth> {
        self.replica_health.get(id)
    }

    /// Get all replica health statuses.
    pub fn all_health(&self) -> &HashMap<String, ReplicaHealth> {
        &self.replica_health
    }

    /// Update health for a replica.
    pub fn update_health(&mut self, id: &str, health: ReplicaHealth) {
        self.replica_health.insert(id.to_string(), health);
    }

    /// Get healthy replicas sorted by lag.
    fn healthy_replicas_by_lag(&self) -> Vec<&ReplicaEndpoint> {
        let mut replicas: Vec<&ReplicaEndpoint> = self
            .config
            .replicas
            .iter()
            .filter(|r| {
                self.replica_health
                    .get(&r.id)
                    .map(|h| h.is_healthy && h.replication_lag <= self.config.max_replication_lag)
                    .unwrap_or(false)
            })
            .collect();

        replicas.sort_by_key(|r| {
            self.replica_health
                .get(&r.id)
                .map(|h| h.replication_lag)
                .unwrap_or(Duration::MAX)
        });

        replicas
    }

    fn route_round_robin(&self) -> Result<&ReplicaEndpoint, ReplicaError> {
        let healthy = self.healthy_replicas_by_lag();
        if healthy.is_empty() {
            // Fallback to primary if no healthy replicas
            return Ok(&self.config.primary);
        }
        let idx = self.round_robin_index.fetch_add(1, Ordering::Relaxed);
        Ok(healthy[idx % healthy.len()])
    }

    fn route_weighted_round_robin(&self) -> Result<&ReplicaEndpoint, ReplicaError> {
        let healthy = self.healthy_replicas_by_lag();
        if healthy.is_empty() {
            return Ok(&self.config.primary);
        }
        let total_weight: u32 = healthy.iter().map(|r| r.weight).sum();
        if total_weight == 0 {
            return Ok(&self.config.primary);
        }

        let idx = self.round_robin_index.fetch_add(1, Ordering::Relaxed) as u32;
        let mut accumulated = 0u32;
        for replica in &healthy {
            accumulated += replica.weight;
            if idx % total_weight < accumulated {
                return Ok(replica);
            }
        }
        Ok(healthy.last().unwrap())
    }

    fn route_least_lag(&self) -> Result<&ReplicaEndpoint, ReplicaError> {
        let healthy = self.healthy_replicas_by_lag();
        healthy
            .first()
            .copied()
            .ok_or(ReplicaError::NoHealthyReplicas)
    }

    fn route_nearest(&self) -> Result<&ReplicaEndpoint, ReplicaError> {
        // Without client region info, fall back to round-robin
        self.route_round_robin()
    }
}

/// Monitors replication lag between primary and replicas.
pub struct ReplicationLagMonitor {
    /// Lag measurements per replica, stored as (timestamp, lag) pairs.
    measurements: HashMap<String, Vec<LagMeasurement>>,
    /// Maximum measurements to keep per replica.
    max_measurements: usize,
}

/// A single lag measurement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LagMeasurement {
    /// When the measurement was taken.
    pub timestamp: DateTime<Utc>,
    /// Measured lag duration.
    pub lag: Duration,
}

impl ReplicationLagMonitor {
    pub fn new() -> Self {
        Self {
            measurements: HashMap::new(),
            max_measurements: 1000,
        }
    }

    /// Record a lag measurement for a replica.
    pub fn record_lag(&mut self, replica_id: &str, lag: Duration) {
        let measurements = self
            .measurements
            .entry(replica_id.to_string())
            .or_insert_with(Vec::new);

        measurements.push(LagMeasurement {
            timestamp: Utc::now(),
            lag,
        });

        // Trim old measurements
        if measurements.len() > self.max_measurements {
            let drain_count = measurements.len() - self.max_measurements;
            measurements.drain(..drain_count);
        }
    }

    /// Get the current lag for a replica (most recent measurement).
    pub fn current_lag(&self, replica_id: &str) -> Option<Duration> {
        self.measurements
            .get(replica_id)
            .and_then(|m| m.last())
            .map(|m| m.lag)
    }

    /// Get the average lag over the last N measurements.
    pub fn avg_lag(&self, replica_id: &str, window: usize) -> Option<Duration> {
        let measurements = self.measurements.get(replica_id)?;
        let window_size = window.min(measurements.len());
        if window_size == 0 {
            return None;
        }

        let sum: Duration = measurements
            .iter()
            .rev()
            .take(window_size)
            .map(|m| m.lag)
            .sum();

        Some(sum / window_size as u32)
    }

    /// Get the maximum lag over the last N measurements.
    pub fn max_lag(&self, replica_id: &str, window: usize) -> Option<Duration> {
        let measurements = self.measurements.get(replica_id)?;
        let window_size = window.min(measurements.len());
        if window_size == 0 {
            return None;
        }

        measurements
            .iter()
            .rev()
            .take(window_size)
            .map(|m| m.lag)
            .max()
    }

    /// Get all replica IDs being monitored.
    pub fn monitored_replicas(&self) -> Vec<&str> {
        self.measurements.keys().map(|s| s.as_str()).collect()
    }

    /// Clear measurements for a replica.
    pub fn clear(&mut self, replica_id: &str) {
        self.measurements.remove(replica_id);
    }
}

impl Default for ReplicationLagMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages automatic failover when the primary becomes unavailable.
pub struct FailoverManager {
    /// Whether failover is currently in progress.
    pub failover_in_progress: bool,
    /// The replica that was promoted to primary (if any).
    pub promoted_replica: Option<String>,
    /// When failover was initiated.
    pub failover_started_at: Option<Instant>,
    /// How many times failover has occurred.
    pub failover_count: u32,
    /// Maximum consecutive failures before triggering failover.
    pub failure_threshold: u32,
    /// Current consecutive primary failure count.
    pub consecutive_primary_failures: u32,
    /// Timeout for failover operation.
    pub failover_timeout: Duration,
}

impl FailoverManager {
    pub fn new() -> Self {
        Self {
            failover_in_progress: false,
            promoted_replica: None,
            failover_started_at: None,
            failover_count: 0,
            failure_threshold: 3,
            consecutive_primary_failures: 0,
            failover_timeout: Duration::from_secs(30),
        }
    }

    /// Record a primary failure. Returns true if failover should be triggered.
    pub fn record_primary_failure(&mut self) -> bool {
        self.consecutive_primary_failures += 1;
        self.consecutive_primary_failures >= self.failure_threshold
    }

    /// Record primary recovery. Resets failure count.
    pub fn record_primary_recovery(&mut self) {
        self.consecutive_primary_failures = 0;
        if self.failover_in_progress {
            self.failover_in_progress = false;
            self.promoted_replica = None;
            self.failover_started_at = None;
        }
    }

    /// Initiate failover to a specific replica.
    pub fn initiate_failover(&mut self, replica_id: &str) {
        self.failover_in_progress = true;
        self.promoted_replica = Some(replica_id.to_string());
        self.failover_started_at = Some(Instant::now());
        self.failover_count += 1;
    }

    /// Complete the failover operation.
    pub fn complete_failover(&mut self) {
        self.failover_in_progress = false;
        self.failover_started_at = None;
        self.consecutive_primary_failures = 0;
    }

    /// Check if failover has timed out.
    pub fn is_failover_timed_out(&self) -> bool {
        self.failover_started_at
            .map(|t| t.elapsed() > self.failover_timeout)
            .unwrap_or(false)
    }

    /// Get the replica that should be promoted (lowest lag, highest health).
    pub fn select_promotion_candidate(
        replicas: &[ReplicaEndpoint],
        health: &HashMap<String, ReplicaHealth>,
        max_lag: Duration,
    ) -> Option<String> {
        replicas
            .iter()
            .filter(|r| {
                health
                    .get(&r.id)
                    .map(|h| h.is_healthy && h.replication_lag <= max_lag)
                    .unwrap_or(false)
            })
            .min_by_key(|r| {
                health
                    .get(&r.id)
                    .map(|h| h.replication_lag)
                    .unwrap_or(Duration::MAX)
            })
            .map(|r| r.id.clone())
    }
}

impl Default for FailoverManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_replica_config() -> ReplicaConfig {
        ReplicaConfig::with_replicas(
            "postgres://localhost:5432/civitforge",
            &[
                "postgres://localhost:5433/civitforge",
                "postgres://localhost:5434/civitforge",
            ],
        )
    }

    #[test]
    fn test_replica_endpoint_new() {
        let ep = ReplicaEndpoint::new("r0", "postgres://localhost/db", false);
        assert_eq!(ep.id, "r0");
        assert!(!ep.is_primary);
        assert_eq!(ep.weight, 1);
    }

    #[test]
    fn test_replica_endpoint_builders() {
        let ep = ReplicaEndpoint::new("r0", "url", false)
            .with_region("us-east-1")
            .with_max_connections(500)
            .with_weight(5);
        assert_eq!(ep.region.as_deref(), Some("us-east-1"));
        assert_eq!(ep.max_connections, 500);
        assert_eq!(ep.weight, 5);
    }

    #[test]
    fn test_replica_config_default() {
        let config = ReplicaConfig::default();
        assert!(config.primary.is_primary);
        assert!(config.replicas.is_empty());
        assert_eq!(config.routing, ReadRoutingStrategy::RoundRobin);
    }

    #[test]
    fn test_replica_config_with_replicas() {
        let config = test_replica_config();
        assert_eq!(config.replicas.len(), 2);
        assert_eq!(config.endpoint_count(), 3);
    }

    #[test]
    fn test_replica_config_all_urls() {
        let config = test_replica_config();
        let urls = config.all_urls();
        assert_eq!(urls.len(), 3);
        assert_eq!(urls[0], "postgres://localhost:5432/civitforge");
    }

    #[test]
    fn test_read_replica_router_round_robin() {
        let config = test_replica_config();
        let router = ReadReplicaRouter::new(config);

        // Should return different replicas on successive calls
        let r1 = router.route_read().unwrap();
        let r2 = router.route_read().unwrap();
        // At least one of them should be a replica (not primary)
        assert!(!r1.is_primary || !r2.is_primary);
    }

    #[test]
    fn test_read_replica_router_primary_only() {
        let mut config = test_replica_config();
        config.routing = ReadRoutingStrategy::PrimaryOnly;
        let router = ReadReplicaRouter::new(config);

        for _ in 0..10 {
            let ep = router.route_read().unwrap();
            assert!(ep.is_primary);
        }
    }

    #[test]
    fn test_read_replica_router_write_always_primary() {
        let config = test_replica_config();
        let router = ReadReplicaRouter::new(config);

        let ep = router.route_write();
        assert!(ep.is_primary);
    }

    #[test]
    fn test_replica_health_success() {
        let mut health = ReplicaHealth::new("r0");
        health.record_success(Duration::from_secs(2), Duration::from_millis(15));
        assert!(health.is_healthy);
        assert_eq!(health.replication_lag, Duration::from_secs(2));
        assert!(health.last_healthy_at.is_some());
        assert_eq!(health.consecutive_failures, 0);
    }

    #[test]
    fn test_replica_health_failure() {
        let mut health = ReplicaHealth::new("r0");
        health.record_failure();
        health.record_failure();
        assert!(health.is_healthy); // still healthy at 2 failures

        health.record_failure();
        assert!(!health.is_healthy); // unhealthy at 3
    }

    #[test]
    fn test_lag_monitor_record_and_query() {
        let mut monitor = ReplicationLagMonitor::new();
        monitor.record_lag("r0", Duration::from_secs(1));
        monitor.record_lag("r0", Duration::from_secs(2));
        monitor.record_lag("r0", Duration::from_secs(3));

        assert_eq!(monitor.current_lag("r0"), Some(Duration::from_secs(3)));
        assert_eq!(
            monitor.avg_lag("r0", 3),
            Some(Duration::from_secs(2))
        );
        assert_eq!(
            monitor.max_lag("r0", 3),
            Some(Duration::from_secs(3))
        );
    }

    #[test]
    fn test_lag_monitor_window() {
        let mut monitor = ReplicationLagMonitor::new();
        for i in 0..10 {
            monitor.record_lag("r0", Duration::from_secs(i));
        }

        // Last 3 measurements: 7, 8, 9 seconds
        let avg = monitor.avg_lag("r0", 3).unwrap();
        assert_eq!(avg, Duration::from_secs(8));
    }

    #[test]
    fn test_lag_monitor_monitored_replicas() {
        let mut monitor = ReplicationLagMonitor::new();
        monitor.record_lag("r0", Duration::from_secs(1));
        monitor.record_lag("r1", Duration::from_secs(2));

        let mut replicas = monitor.monitored_replicas();
        replicas.sort();
        assert_eq!(replicas, vec!["r0", "r1"]);
    }

    #[test]
    fn test_lag_monitor_clear() {
        let mut monitor = ReplicationLagMonitor::new();
        monitor.record_lag("r0", Duration::from_secs(1));
        monitor.clear("r0");
        assert!(monitor.current_lag("r0").is_none());
    }

    #[test]
    fn test_failover_manager_initial_state() {
        let fm = FailoverManager::new();
        assert!(!fm.failover_in_progress);
        assert_eq!(fm.failover_count, 0);
        assert_eq!(fm.consecutive_primary_failures, 0);
    }

    #[test]
    fn test_failover_manager_failure_threshold() {
        let mut fm = FailoverManager::new();
        assert!(!fm.record_primary_failure());
        assert!(!fm.record_primary_failure());
        assert!(fm.record_primary_failure()); // 3rd failure triggers
    }

    #[test]
    fn test_failover_manager_recovery() {
        let mut fm = FailoverManager::new();
        fm.record_primary_failure();
        fm.record_primary_failure();
        fm.record_primary_failure();
        fm.initiate_failover("r0");

        fm.record_primary_recovery();
        assert!(!fm.failover_in_progress);
        assert!(fm.promoted_replica.is_none());
        assert_eq!(fm.consecutive_primary_failures, 0);
    }

    #[test]
    fn test_failover_manager_lifecycle() {
        let mut fm = FailoverManager::new();

        // Trigger failover
        fm.initiate_failover("replica-1");
        assert!(fm.failover_in_progress);
        assert_eq!(fm.promoted_replica.as_deref(), Some("replica-1"));
        assert_eq!(fm.failover_count, 1);

        // Complete failover
        fm.complete_failover();
        assert!(!fm.failover_in_progress);
        assert_eq!(fm.consecutive_primary_failures, 0);
    }

    #[test]
    fn test_select_promotion_candidate() {
        let replicas = vec![
            ReplicaEndpoint::new("r0", "url0", false),
            ReplicaEndpoint::new("r1", "url1", false),
            ReplicaEndpoint::new("r2", "url2", false),
        ];

        let mut health = HashMap::new();
        health.insert(
            "r0".to_string(),
            {
                let mut h = ReplicaHealth::new("r0");
                h.replication_lag = Duration::from_secs(5);
                h
            },
        );
        health.insert(
            "r1".to_string(),
            {
                let mut h = ReplicaHealth::new("r1");
                h.replication_lag = Duration::from_secs(1);
                h
            },
        );
        health.insert(
            "r2".to_string(),
            {
                let mut h = ReplicaHealth::new("r2");
                h.is_healthy = false;
                h
            },
        );

        let candidate =
            FailoverManager::select_promotion_candidate(&replicas, &health, Duration::from_secs(10));
        assert_eq!(candidate.as_deref(), Some("r1"));
    }

    #[test]
    fn test_select_promotion_candidate_no_healthy() {
        let replicas = vec![ReplicaEndpoint::new("r0", "url0", false)];
        let mut health = HashMap::new();
        health.insert("r0".to_string(), {
            let mut h = ReplicaHealth::new("r0");
            h.is_healthy = false;
            h
        });

        let candidate =
            FailoverManager::select_promotion_candidate(&replicas, &health, Duration::from_secs(10));
        assert!(candidate.is_none());
    }

    #[test]
    fn test_read_routing_strategy_display() {
        assert_eq!(
            ReadRoutingStrategy::RoundRobin.to_string(),
            "RoundRobin"
        );
    }
}
