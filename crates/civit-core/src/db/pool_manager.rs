#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub min_connections: usize,
    pub max_connections: usize,
    pub connect_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 5,
            max_connections: 20,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
            max_lifetime_secs: 1800,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub active_connections: usize,
    pub idle_connections: usize,
    pub total_connections: usize,
    pub pending_requests: usize,
    pub config: PoolConfig,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionLeak {
    pub connection_id: u64,
    pub acquired_at: DateTime<Utc>,
    pub owner: String,
    pub elapsed_secs: u64,
}

pub struct ConnectionPool {
    config: RwLock<PoolConfig>,
    active_count: AtomicUsize,
    idle_count: AtomicUsize,
    pending_count: AtomicUsize,
    total_created: AtomicUsize,
    connection_owners: dashmap::DashMap<u64, ConnectionOwner>,
}

struct ConnectionOwner {
    connection_id: u64,
    acquired_at: Instant,
    owner: String,
}

impl ConnectionPool {
    pub fn new(config: PoolConfig) -> Self {
        let min = config.min_connections;
        Self {
            config: RwLock::new(config),
            active_count: AtomicUsize::new(0),
            idle_count: AtomicUsize::new(min),
            pending_count: AtomicUsize::new(0),
            total_created: AtomicUsize::new(min),
            connection_owners: dashmap::DashMap::new(),
        }
    }

    pub async fn acquire(&self, owner: &str) -> u64 {
        self.pending_count.fetch_add(1, Ordering::Relaxed);
        let id = self.total_created.fetch_add(1, Ordering::Relaxed) as u64;
        self.pending_count.fetch_sub(1, Ordering::Relaxed);
        self.active_count.fetch_add(1, Ordering::Relaxed);
        self.connection_owners.insert(
            id,
            ConnectionOwner {
                connection_id: id,
                acquired_at: Instant::now(),
                owner: owner.to_string(),
            },
        );
        id
    }

    pub fn release(&self, connection_id: u64) {
        self.connection_owners.remove(&connection_id);
        let active = self.active_count.load(Ordering::Relaxed);
        if active > 0 {
            self.active_count.fetch_sub(1, Ordering::Relaxed);
        }
        self.idle_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn stats(&self) -> PoolStats {
        let config = self.config.try_read().unwrap().clone();
        PoolStats {
            active_connections: self.active_count.load(Ordering::Relaxed),
            idle_connections: self.idle_count.load(Ordering::Relaxed),
            total_connections: self.total_created.load(Ordering::Relaxed),
            pending_requests: self.pending_count.load(Ordering::Relaxed),
            config,
            timestamp: Utc::now(),
        }
    }

    pub fn detect_leaks(&self, max_duration: Duration) -> Vec<ConnectionLeak> {
        let _now = Instant::now();
        self.connection_owners
            .iter()
            .filter(|r| r.value().acquired_at.elapsed() > max_duration)
            .map(|r| ConnectionLeak {
                connection_id: r.value().connection_id,
                acquired_at: Utc::now()
                    - chrono::Duration::from_std(r.value().acquired_at.elapsed())
                        .unwrap_or(chrono::Duration::seconds(0)),
                owner: r.value().owner.clone(),
                elapsed_secs: r.value().acquired_at.elapsed().as_secs(),
            })
            .collect()
    }

    pub async fn resize(&self, new_config: PoolConfig) {
        let mut config = self.config.write().await;
        *config = new_config;
    }

    pub async fn adjust_for_load(&self, load_factor: f64) {
        let mut config = self.config.write().await;
        let base_max = config.max_connections as f64;
        let adjusted = (base_max * load_factor).ceil() as usize;
        config.max_connections = adjusted
            .max(config.min_connections)
            .min(config.max_connections * 2);
    }

    pub fn active_count(&self) -> usize {
        self.active_count.load(Ordering::Relaxed)
    }

    pub fn idle_count(&self) -> usize {
        self.idle_count.load(Ordering::Relaxed)
    }

    pub fn pending_count(&self) -> usize {
        self.pending_count.load(Ordering::Relaxed)
    }
}

pub struct PoolHealthChecker {
    pool: Arc<ConnectionPool>,
    leak_threshold: Duration,
}

impl PoolHealthChecker {
    pub fn new(pool: Arc<ConnectionPool>, leak_threshold: Duration) -> Self {
        Self {
            pool,
            leak_threshold,
        }
    }

    pub fn check(&self) -> PoolHealthReport {
        let stats = self.pool.stats();
        let leaks = self.pool.detect_leaks(self.leak_threshold);

        let health = if leaks.len() > 3 {
            PoolHealth::Critical
        } else if !leaks.is_empty() || stats.active_connections > stats.config.max_connections {
            PoolHealth::Degraded
        } else {
            PoolHealth::Healthy
        };

        PoolHealthReport {
            health,
            stats,
            leaks,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoolHealth {
    Healthy,
    Degraded,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolHealthReport {
    pub health: PoolHealth,
    pub stats: PoolStats,
    pub leaks: Vec<ConnectionLeak>,
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.min_connections, 5);
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.connect_timeout_secs, 30);
        assert_eq!(config.idle_timeout_secs, 600);
        assert_eq!(config.max_lifetime_secs, 1800);
    }

    #[tokio::test]
    async fn test_pool_acquire_and_release() {
        let pool = ConnectionPool::new(PoolConfig::default());
        let id = pool.acquire("test").await;
        assert_eq!(pool.active_count(), 1);
        pool.release(id);
        assert_eq!(pool.active_count(), 0);
        assert_eq!(pool.idle_count(), 6);
    }

    #[tokio::test]
    async fn test_pool_stats() {
        let pool = ConnectionPool::new(PoolConfig::default());
        let _id = pool.acquire("test").await;
        let stats = pool.stats();
        assert_eq!(stats.active_connections, 1);
        assert_eq!(stats.config.min_connections, 5);
    }

    #[tokio::test]
    async fn test_detect_leaks() {
        let pool = ConnectionPool::new(PoolConfig::default());
        let _id = pool.acquire("leaky").await;
        let leaks = pool.detect_leaks(Duration::from_secs(0));
        assert_eq!(leaks.len(), 1);
        assert_eq!(leaks[0].owner, "leaky");
    }

    #[tokio::test]
    async fn test_pool_resize() {
        let pool = ConnectionPool::new(PoolConfig::default());
        let new_config = PoolConfig {
            max_connections: 50,
            ..Default::default()
        };
        pool.resize(new_config).await;
        let stats = pool.stats();
        assert_eq!(stats.config.max_connections, 50);
    }

    #[test]
    fn test_pool_health_checker() {
        let pool = Arc::new(ConnectionPool::new(PoolConfig::default()));
        let checker = PoolHealthChecker::new(pool, Duration::from_secs(60));
        let report = checker.check();
        assert!(matches!(report.health, PoolHealth::Healthy));
        assert!(report.leaks.is_empty());
    }

    #[tokio::test]
    async fn test_adjust_for_load() {
        let pool = ConnectionPool::new(PoolConfig::default());
        pool.adjust_for_load(2.0).await;
        let stats = pool.stats();
        assert!(stats.config.max_connections >= 20);
    }
}
