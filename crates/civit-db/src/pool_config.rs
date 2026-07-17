#![forbid(unsafe_code)]

use std::time::Duration;

/// Configuration for database connection pool tuning.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of connections in the pool.
    pub max_connections: u32,
    /// Minimum idle connections to maintain.
    pub min_idle: u32,
    /// Maximum time to wait for a connection from the pool.
    pub connect_timeout: Duration,
    /// Maximum time a connection can be idle before being closed.
    pub idle_timeout: Duration,
    /// Maximum lifetime of a connection in the pool.
    pub max_lifetime: Duration,
    /// Interval between connection health checks.
    pub health_check_interval: Duration,
    /// Whether to enable connection recycling (re-validate on checkout).
    pub test_on_check_out: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self::calculate_optimal()
    }
}

impl PoolConfig {
    /// Calculate optimal pool configuration based on system characteristics.
    ///
    /// Formula: max_connections = (CPU cores * 2) + effective_spindle_count
    /// For SSDs, effective_spindle_count = 0, so: cores * 2 + 1
    /// Capped at reasonable bounds for a web application.
    pub fn calculate_optimal() -> Self {
        let cpu_cores = std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(4);

        // Rule of thumb: 2 * cores + 1 for SSD-backed databases
        // Minimum 5, maximum 100 for web apps
        let max_connections = ((cpu_cores * 2) + 1).clamp(5, 100);

        // Keep at least 20% of max as idle for burst traffic
        let min_idle = (max_connections / 5).max(2);

        Self {
            max_connections,
            min_idle,
            connect_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(1800),
            health_check_interval: Duration::from_secs(30),
            test_on_check_out: true,
        }
    }

    /// Create a configuration for high-throughput servers (many cores).
    pub fn high_throughput() -> Self {
        let cpu_cores = std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(8);

        let max_connections = ((cpu_cores * 2) + 2).clamp(20, 200);

        Self {
            max_connections,
            min_idle: (max_connections / 4).max(5),
            connect_timeout: Duration::from_secs(3),
            idle_timeout: Duration::from_secs(120),
            max_lifetime: Duration::from_secs(900),
            health_check_interval: Duration::from_secs(15),
            test_on_check_out: false,
        }
    }

    /// Create a configuration for low-resource environments.
    pub fn low_resource() -> Self {
        Self {
            max_connections: 5,
            min_idle: 1,
            connect_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(3600),
            health_check_interval: Duration::from_secs(60),
            test_on_check_out: false,
        }
    }

    /// Validate the configuration and return any warnings.
    pub fn validate(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        if self.max_connections > 100 {
            warnings.push(format!(
                "max_connections={} is very high; may cause database resource contention",
                self.max_connections
            ));
        }

        if self.max_connections < 3 {
            warnings.push(format!(
                "max_connections={} is very low; may cause connection starvation",
                self.max_connections
            ));
        }

        if self.min_idle > self.max_connections / 2 {
            warnings.push(format!(
                "min_idle={} is more than half of max_connections={}; consider reducing",
                self.min_idle, self.max_connections
            ));
        }

        if self.connect_timeout < Duration::from_secs(1) {
            warnings.push("connect_timeout < 1s may cause spurious failures".into());
        }

        if self.max_lifetime < Duration::from_secs(60) {
            warnings.push("max_lifetime < 60s will cause excessive connection cycling".into());
        }

        warnings
    }
}

/// Calculate the recommended pool size for a given workload profile.
pub fn recommended_pool_size(
    concurrent_requests: u32,
    avg_query_duration_ms: u64,
    cpu_cores: u32,
) -> u32 {
    // Little's Law: L = λ * W
    // L = pool size needed
    // λ = request arrival rate (requests/sec)
    // W = avg query duration (seconds)
    //
    // Conservative: pool_size = (requests_per_second * avg_duration_seconds) * safety_factor
    let arrival_rate = concurrent_requests as f64;
    let avg_duration_secs = avg_query_duration_ms as f64 / 1000.0;
    let calculated = (arrival_rate * avg_duration_secs * 1.5) as u32;

    // Also cap by CPU cores * 2 as a ceiling
    let ceiling = cpu_cores * 2;

    calculated.clamp(5, ceiling.max(10))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = PoolConfig::default();
        let warnings = config.validate();
        // Default should have no warnings
        assert!(
            warnings.is_empty(),
            "Default config produced warnings: {:?}",
            warnings
        );
    }

    #[test]
    fn test_high_throughput_config() {
        let config = PoolConfig::high_throughput();
        assert!(config.max_connections >= 20);
        let warnings = config.validate();
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_low_resource_config() {
        let config = PoolConfig::low_resource();
        assert_eq!(config.max_connections, 5);
        assert_eq!(config.min_idle, 1);
    }

    #[test]
    fn test_validate_warns_on_high_connections() {
        let config = PoolConfig {
            max_connections: 200,
            min_idle: 1,
            connect_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(1800),
            health_check_interval: Duration::from_secs(30),
            test_on_check_out: true,
        };
        let warnings = config.validate();
        assert!(warnings.iter().any(|w| w.contains("very high")));
    }

    #[test]
    fn test_validate_warns_on_low_connections() {
        let config = PoolConfig {
            max_connections: 2,
            min_idle: 1,
            connect_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(1800),
            health_check_interval: Duration::from_secs(30),
            test_on_check_out: true,
        };
        let warnings = config.validate();
        assert!(warnings.iter().any(|w| w.contains("very low")));
    }

    #[test]
    fn test_recommended_pool_size() {
        // 50 concurrent requests, 100ms avg query, 4 cores
        let size = recommended_pool_size(50, 100, 4);
        assert!(size >= 5);
        assert!(size <= 8); // 4 * 2 = 8
    }

    #[test]
    fn test_recommended_pool_size_high_load() {
        // 200 concurrent, 50ms avg, 16 cores
        let size = recommended_pool_size(200, 50, 16);
        assert!(size >= 5);
        assert!(size <= 32);
    }
}
