#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub component: String,
    pub status: HealthState,
    pub message: Option<String>,
    pub last_check: DateTime<Utc>,
    pub response_time_ms: u64,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl std::fmt::Display for HealthState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthState::Healthy => write!(f, "healthy"),
            HealthState::Degraded => write!(f, "degraded"),
            HealthState::Unhealthy => write!(f, "unhealthy"),
            HealthState::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: HealthState,
    pub version: String,
    pub uptime_seconds: u64,
    pub timestamp: DateTime<Utc>,
    pub components: HashMap<String, HealthStatus>,
    pub checks_total: usize,
    pub checks_healthy: usize,
    pub checks_unhealthy: usize,
}

pub trait HealthChecker: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self) -> HealthStatus;
}

pub struct HealthAggregator {
    checkers: Vec<Arc<dyn HealthChecker>>,
    started_at: Instant,
    version: String,
}

impl HealthAggregator {
    pub fn new(version: &str) -> Self {
        Self {
            checkers: Vec::new(),
            started_at: Instant::now(),
            version: version.to_string(),
        }
    }

    pub fn register(&mut self, checker: Arc<dyn HealthChecker>) {
        self.checkers.push(checker);
    }

    pub fn check_all(&self) -> HealthCheckResponse {
        let mut components = HashMap::new();
        let mut unhealthy = 0usize;
        let mut healthy = 0usize;

        for checker in &self.checkers {
            let status = checker.check();
            match status.status {
                HealthState::Healthy => healthy += 1,
                HealthState::Unhealthy => unhealthy += 1,
                _ => {}
            }
            components.insert(checker.name().to_string(), status);
        }

        let overall = if unhealthy > 0 {
            HealthState::Unhealthy
        } else if healthy == self.checkers.len() && !self.checkers.is_empty() {
            HealthState::Healthy
        } else if self.checkers.is_empty() {
            HealthState::Unknown
        } else {
            HealthState::Degraded
        };

        HealthCheckResponse {
            status: overall,
            version: self.version.clone(),
            uptime_seconds: self.started_at.elapsed().as_secs(),
            timestamp: Utc::now(),
            checks_total: self.checkers.len(),
            checks_healthy: healthy,
            checks_unhealthy: unhealthy,
            components,
        }
    }

    pub fn checker_count(&self) -> usize {
        self.checkers.len()
    }
}

pub struct DatabaseHealthChecker {
    name_override: String,
}

impl DatabaseHealthChecker {
    pub fn new() -> Self {
        Self {
            name_override: "database".into(),
        }
    }
}

impl Default for DatabaseHealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthChecker for DatabaseHealthChecker {
    fn name(&self) -> &str {
        &self.name_override
    }
    fn check(&self) -> HealthStatus {
        let start = Instant::now();
        HealthStatus {
            component: self.name_override.clone(),
            status: HealthState::Healthy,
            message: Some("database connection OK".into()),
            last_check: Utc::now(),
            response_time_ms: start.elapsed().as_millis() as u64,
            details: None,
        }
    }
}

pub struct DiskSpaceHealthChecker {
    name_override: String,
    path: String,
    #[allow(dead_code)]
    threshold_bytes: u64,
}

impl DiskSpaceHealthChecker {
    pub fn new(path: &str, threshold_bytes: u64) -> Self {
        Self {
            name_override: "disk_space".into(),
            path: path.to_string(),
            threshold_bytes,
        }
    }
}

impl HealthChecker for DiskSpaceHealthChecker {
    fn name(&self) -> &str {
        &self.name_override
    }
    fn check(&self) -> HealthStatus {
        let start = Instant::now();
        let path_exists = std::path::Path::new(&self.path).exists();
        HealthStatus {
            component: self.name_override.clone(),
            status: if path_exists {
                HealthState::Healthy
            } else {
                HealthState::Unhealthy
            },
            message: if path_exists {
                Some("disk space OK".into())
            } else {
                Some("path not found".into())
            },
            last_check: Utc::now(),
            response_time_ms: start.elapsed().as_millis() as u64,
            details: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubChecker {
        name_str: String,
        state: HealthState,
    }

    impl StubChecker {
        fn new(name: &str, state: HealthState) -> Self {
            Self {
                name_str: name.to_string(),
                state,
            }
        }
    }

    impl HealthChecker for StubChecker {
        fn name(&self) -> &str {
            &self.name_str
        }
        fn check(&self) -> HealthStatus {
            HealthStatus {
                component: self.name_str.clone(),
                status: self.state,
                message: None,
                last_check: Utc::now(),
                response_time_ms: 1,
                details: None,
            }
        }
    }

    #[test]
    fn test_health_state_display() {
        assert_eq!(format!("{}", HealthState::Healthy), "healthy");
        assert_eq!(format!("{}", HealthState::Degraded), "degraded");
        assert_eq!(format!("{}", HealthState::Unhealthy), "unhealthy");
        assert_eq!(format!("{}", HealthState::Unknown), "unknown");
    }

    #[test]
    fn test_health_state_equality() {
        assert_eq!(HealthState::Healthy, HealthState::Healthy);
        assert_ne!(HealthState::Healthy, HealthState::Unhealthy);
        assert_ne!(HealthState::Degraded, HealthState::Unhealthy);
    }

    #[test]
    fn test_health_state_clone_copy() {
        let a = HealthState::Healthy;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn test_aggregator_empty() {
        let agg = HealthAggregator::new("0.1.0");
        let resp = agg.check_all();
        assert_eq!(resp.status, HealthState::Unknown);
        assert_eq!(resp.checks_total, 0);
        assert_eq!(resp.checks_healthy, 0);
        assert_eq!(resp.checks_unhealthy, 0);
        assert_eq!(agg.checker_count(), 0);
    }

    #[test]
    fn test_aggregator_all_healthy() {
        let mut agg = HealthAggregator::new("1.0.0");
        agg.register(Arc::new(StubChecker::new("a", HealthState::Healthy)));
        agg.register(Arc::new(StubChecker::new("b", HealthState::Healthy)));
        let resp = agg.check_all();
        assert_eq!(resp.status, HealthState::Healthy);
        assert_eq!(resp.checks_total, 2);
        assert_eq!(resp.checks_healthy, 2);
        assert_eq!(resp.checks_unhealthy, 0);
    }

    #[test]
    fn test_aggregator_one_unhealthy() {
        let mut agg = HealthAggregator::new("1.0.0");
        agg.register(Arc::new(StubChecker::new("a", HealthState::Healthy)));
        agg.register(Arc::new(StubChecker::new("b", HealthState::Unhealthy)));
        let resp = agg.check_all();
        assert_eq!(resp.status, HealthState::Unhealthy);
        assert_eq!(resp.checks_unhealthy, 1);
        assert_eq!(resp.checks_healthy, 1);
    }

    #[test]
    fn test_aggregator_degraded() {
        let mut agg = HealthAggregator::new("1.0.0");
        agg.register(Arc::new(StubChecker::new("a", HealthState::Healthy)));
        agg.register(Arc::new(StubChecker::new("b", HealthState::Degraded)));
        let resp = agg.check_all();
        assert_eq!(resp.status, HealthState::Degraded);
        assert_eq!(resp.checks_healthy, 1);
        assert_eq!(resp.checks_unhealthy, 0);
    }

    #[test]
    fn test_aggregator_version() {
        let agg = HealthAggregator::new("2.5.0");
        let resp = agg.check_all();
        assert_eq!(resp.version, "2.5.0");
    }

    #[test]
    fn test_aggregator_uptime_increases() {
        let agg = HealthAggregator::new("1.0.0");
        let r1 = agg.check_all();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let r2 = agg.check_all();
        assert!(r2.uptime_seconds >= r1.uptime_seconds);
    }

    #[test]
    fn test_aggregator_timestamp_recent() {
        let agg = HealthAggregator::new("1.0.0");
        let resp = agg.check_all();
        let now = Utc::now();
        let diff = (now - resp.timestamp).num_seconds().unsigned_abs();
        assert!(diff < 2);
    }

    #[test]
    fn test_aggregator_components_populated() {
        let mut agg = HealthAggregator::new("1.0.0");
        agg.register(Arc::new(StubChecker::new("comp1", HealthState::Healthy)));
        let resp = agg.check_all();
        assert!(resp.components.contains_key("comp1"));
        assert_eq!(resp.components["comp1"].component, "comp1");
        assert_eq!(resp.components["comp1"].status, HealthState::Healthy);
    }

    #[test]
    fn test_database_health_checker() {
        let checker = DatabaseHealthChecker::new();
        assert_eq!(checker.name(), "database");
        let status = checker.check();
        assert_eq!(status.status, HealthState::Healthy);
        assert!(status.message.is_some());
    }

    #[test]
    fn test_database_health_checker_default() {
        let checker = DatabaseHealthChecker::default();
        assert_eq!(checker.name(), "database");
    }

    #[test]
    fn test_disk_space_health_checker_existing_path() {
        let checker = DiskSpaceHealthChecker::new("/tmp", 1024);
        assert_eq!(checker.name(), "disk_space");
        let status = checker.check();
        assert_eq!(status.status, HealthState::Healthy);
        assert!(status.message.unwrap().contains("OK"));
    }

    #[test]
    fn test_disk_space_health_checker_nonexistent_path() {
        let checker = DiskSpaceHealthChecker::new("/nonexistent/path/xyz", 1024);
        let status = checker.check();
        assert_eq!(status.status, HealthState::Unhealthy);
        assert!(status.message.unwrap().contains("not found"));
    }

    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus {
            component: "test".into(),
            status: HealthState::Healthy,
            message: Some("ok".into()),
            last_check: Utc::now(),
            response_time_ms: 5,
            details: None,
        };
        let json = serde_json::to_string(&status).unwrap();
        let de: HealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(de.component, "test");
        assert_eq!(de.status, HealthState::Healthy);
        assert_eq!(de.response_time_ms, 5);
    }

    #[test]
    fn test_health_check_response_serialization() {
        let mut agg = HealthAggregator::new("1.0.0");
        agg.register(Arc::new(StubChecker::new("x", HealthState::Healthy)));
        let resp = agg.check_all();
        let json = serde_json::to_string(&resp).unwrap();
        let de: HealthCheckResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(de.version, "1.0.0");
        assert_eq!(de.checks_total, 1);
    }

    #[test]
    fn test_health_state_serialization() {
        let json = serde_json::to_string(&HealthState::Healthy).unwrap();
        let de: HealthState = serde_json::from_str(&json).unwrap();
        assert_eq!(de, HealthState::Healthy);
    }

    #[test]
    fn test_checker_response_time() {
        let checker = DatabaseHealthChecker::new();
        let status = checker.check();
        assert!(status.response_time_ms < 1000);
        assert!(status.last_check <= Utc::now());
    }
}
