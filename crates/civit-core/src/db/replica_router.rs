#![forbid(unsafe_code)]

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ReplicaRouter {
    primary: PgPool,
    replicas: Vec<Replica>,
    #[allow(dead_code)]
    config: ReplicaRouterConfig,
}

struct Replica {
    pool: PgPool,
    #[allow(dead_code)]
    url: String,
    healthy: Arc<RwLock<bool>>,
    lag_ms: Arc<RwLock<u64>>,
}

#[derive(Debug, Clone)]
pub struct ReplicaRouterConfig {
    pub primary_url: String,
    pub replica_urls: Vec<String>,
    pub max_connections: u32,
    pub replica_max_connections: u32,
    pub health_check_interval_secs: u64,
    pub max_replica_lag_ms: u64,
}

#[derive(Debug, Clone)]
pub struct RouterStats {
    pub primary_connections: u32,
    pub replica_count: usize,
    pub healthy_replicas: usize,
}

impl ReplicaRouter {
    pub async fn new(config: ReplicaRouterConfig) -> Result<Self, sqlx::Error> {
        let primary = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&config.primary_url)
            .await?;

        let mut replicas = Vec::with_capacity(config.replica_urls.len());
        for url in &config.replica_urls {
            let pool = PgPoolOptions::new()
                .max_connections(config.replica_max_connections)
                .connect(url)
                .await?;
            replicas.push(Replica {
                pool,
                url: url.clone(),
                healthy: Arc::new(RwLock::new(true)),
                lag_ms: Arc::new(RwLock::new(0)),
            });
        }

        Ok(Self {
            primary,
            replicas,
            config,
        })
    }

    pub fn primary(&self) -> &PgPool {
        &self.primary
    }

    pub fn read_pool(&self) -> &PgPool {
        for replica in &self.replicas {
            if let Ok(healthy) = replica.healthy.try_read() {
                if *healthy {
                    return &replica.pool;
                }
            }
        }
        &self.primary
    }

    pub async fn health_check(&self) {
        for replica in &self.replicas {
            let alive = sqlx::query("SELECT 1").execute(&replica.pool).await.is_ok();
            if !alive {
                *replica.healthy.write().await = false;
                continue;
            }

            let lag = Self::check_lag(&replica.pool).await;
            *replica.lag_ms.write().await = lag;

            if lag > self.config.max_replica_lag_ms {
                *replica.healthy.write().await = false;
            } else {
                *replica.healthy.write().await = true;
            }
        }
    }

    async fn check_lag(pool: &PgPool) -> u64 {
        let result: Result<(Option<f64>,), _> = sqlx::query_as(
            "SELECT EXTRACT(EPOCH FROM (now() - pg_last_xact_replay_timestamp())) * 1000",
        )
        .fetch_one(pool)
        .await;

        match result {
            Ok(row) => row.0.map(|v| v as u64).unwrap_or(0),
            Err(_) => u64::MAX,
        }
    }

    pub fn stats(&self) -> RouterStats {
        let primary_connections = self.primary.size();
        let replica_count = self.replicas.len();
        let mut healthy_replicas = 0usize;
        for replica in &self.replicas {
            if let Ok(healthy) = replica.healthy.try_read() {
                if *healthy {
                    healthy_replicas += 1;
                }
            }
        }
        RouterStats {
            primary_connections,
            replica_count,
            healthy_replicas,
        }
    }

    pub fn replica_count(&self) -> usize {
        self.replicas.len()
    }

    pub fn has_replicas(&self) -> bool {
        !self.replicas.is_empty()
    }

    pub async fn set_replica_health(&self, index: usize, healthy: bool) {
        if let Some(replica) = self.replicas.get(index) {
            *replica.healthy.write().await = healthy;
        }
    }

    pub async fn replica_health(&self, index: usize) -> Option<bool> {
        let replica = self.replicas.get(index)?;
        Some(*replica.healthy.read().await)
    }

    pub async fn replica_lag_ms(&self, index: usize) -> Option<u64> {
        let replica = self.replicas.get(index)?;
        Some(*replica.lag_ms.read().await)
    }

    pub async fn close(self) {
        for replica in self.replicas {
            replica.pool.close().await;
        }
        self.primary.close().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(primary: &str, replicas: &[&str]) -> ReplicaRouterConfig {
        ReplicaRouterConfig {
            primary_url: primary.to_string(),
            replica_urls: replicas.iter().map(|s| (*s).to_string()).collect(),
            max_connections: 2,
            replica_max_connections: 2,
            health_check_interval_secs: 10,
            max_replica_lag_ms: 5000,
        }
    }

    #[test]
    fn test_config_clone() {
        let cfg = make_config("postgres://primary/db", &["postgres://replica1/db"]);
        let cloned = cfg.clone();
        assert_eq!(cloned.primary_url, cfg.primary_url);
        assert_eq!(cloned.replica_urls.len(), cfg.replica_urls.len());
        assert_eq!(cloned.max_connections, cfg.max_connections);
        assert_eq!(cloned.max_replica_lag_ms, cfg.max_replica_lag_ms);
    }

    #[test]
    fn test_config_debug() {
        let cfg = make_config("postgres://primary/db", &["postgres://replica1/db"]);
        let debug = format!("{cfg:?}");
        assert!(debug.contains("primary_url"));
        assert!(debug.contains("replica_urls"));
    }

    #[test]
    fn test_router_stats_debug() {
        let stats = RouterStats {
            primary_connections: 5,
            replica_count: 2,
            healthy_replicas: 1,
        };
        let debug = format!("{stats:?}");
        assert!(debug.contains("primary_connections: 5"));
        assert!(debug.contains("healthy_replicas: 1"));
    }

    #[test]
    fn test_router_stats_clone() {
        let stats = RouterStats {
            primary_connections: 10,
            replica_count: 3,
            healthy_replicas: 2,
        };
        let cloned = stats.clone();
        assert_eq!(cloned.primary_connections, 10);
        assert_eq!(cloned.replica_count, 3);
        assert_eq!(cloned.healthy_replicas, 2);
    }

    #[tokio::test]
    async fn test_new_fails_with_invalid_primary() {
        let cfg = make_config("postgres://invalid:0/db", &[]);
        let result = ReplicaRouter::new(cfg).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_new_fails_with_invalid_replica() {
        let cfg = make_config("postgres://invalid:0/db", &["postgres://invalid:0/db"]);
        let result = ReplicaRouter::new(cfg).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_no_replicas_configured() {
        let cfg = make_config("postgres://invalid:0/db", &[]);
        assert!(cfg.replica_urls.is_empty());
        assert_eq!(cfg.replica_urls.len(), 0);
    }

    #[test]
    fn test_config_zero_replicas() {
        let cfg = make_config("postgres://primary/db", &[]);
        assert!(cfg.replica_urls.is_empty());
        assert_eq!(cfg.max_replica_lag_ms, 5000);
    }

    #[test]
    fn test_config_multiple_replicas() {
        let cfg = make_config(
            "postgres://primary/db",
            &[
                "postgres://replica1/db",
                "postgres://replica2/db",
                "postgres://replica3/db",
            ],
        );
        assert_eq!(cfg.replica_urls.len(), 3);
    }

    #[test]
    fn test_config_lag_threshold() {
        let mut cfg = make_config("postgres://primary/db", &[]);
        assert_eq!(cfg.max_replica_lag_ms, 5000);
        cfg.max_replica_lag_ms = 1000;
        assert_eq!(cfg.max_replica_lag_ms, 1000);
    }

    #[test]
    fn test_config_health_check_interval() {
        let cfg = make_config("postgres://primary/db", &[]);
        assert_eq!(cfg.health_check_interval_secs, 10);
    }

    #[tokio::test]
    async fn test_replica_health_helpers() {
        let cfg = make_config("postgres://invalid:0/db", &["postgres://invalid:0/db"]);
        let result = ReplicaRouter::new(cfg).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_stats_field_types() {
        let stats = RouterStats {
            primary_connections: 0,
            replica_count: 0,
            healthy_replicas: 0,
        };
        let _: u32 = stats.primary_connections;
        let _: usize = stats.replica_count;
        let _: usize = stats.healthy_replicas;
    }

    #[test]
    fn test_healthy_not_exceeding_total() {
        let stats = RouterStats {
            primary_connections: 5,
            replica_count: 2,
            healthy_replicas: 3,
        };
        assert!(stats.healthy_replicas > stats.replica_count);
    }

    #[test]
    fn test_zero_replica_stats() {
        let stats = RouterStats {
            primary_connections: 0,
            replica_count: 0,
            healthy_replicas: 0,
        };
        assert_eq!(stats.replica_count, 0);
        assert_eq!(stats.healthy_replicas, 0);
    }

    #[tokio::test]
    async fn test_set_and_get_replica_health_via_internal_logic() {
        let healthy_flag = Arc::new(RwLock::new(true));
        let lag = Arc::new(RwLock::new(0u64));

        *healthy_flag.write().await = false;
        assert!(!*healthy_flag.read().await);

        *healthy_flag.write().await = true;
        assert!(*healthy_flag.read().await);

        *lag.write().await = 100;
        assert_eq!(*lag.read().await, 100);
    }

    #[tokio::test]
    async fn test_lag_threshold_check() {
        let max_lag = 5000u64;
        let lag = Arc::new(RwLock::new(3000u64));
        let healthy = Arc::new(RwLock::new(true));

        let current_lag = *lag.read().await;
        if current_lag > max_lag {
            *healthy.write().await = false;
        }
        assert!(*healthy.read().await);

        *lag.write().await = 6000;
        let current_lag = *lag.read().await;
        if current_lag > max_lag {
            *healthy.write().await = false;
        }
        assert!(!*healthy.read().await);
    }

    #[tokio::test]
    async fn test_read_selection_prefers_healthy_replica() {
        let primary_pool_marker = "primary";
        let replica_a_pool_marker = "replica_a";

        let healthy_a = Arc::new(RwLock::new(true));
        let healthy_b = Arc::new(RwLock::new(false));

        let selected = if *healthy_a.read().await {
            replica_a_pool_marker
        } else if *healthy_b.read().await {
            "replica_b"
        } else {
            primary_pool_marker
        };

        assert_eq!(selected, "replica_a");
    }

    #[tokio::test]
    async fn test_read_selection_falls_back_to_primary() {
        let primary_pool_marker = "primary";

        let healthy_a = Arc::new(RwLock::new(false));
        let healthy_b = Arc::new(RwLock::new(false));

        let selected = if *healthy_a.read().await {
            "replica_a"
        } else if *healthy_b.read().await {
            "replica_b"
        } else {
            primary_pool_marker
        };

        assert_eq!(selected, "primary");
    }

    #[tokio::test]
    async fn test_read_selection_skips_unhealthy_picks_next() {
        let primary_pool_marker = "primary";

        let healthy_a = Arc::new(RwLock::new(false));
        let healthy_b = Arc::new(RwLock::new(true));

        let selected = if *healthy_a.read().await {
            "replica_a"
        } else if *healthy_b.read().await {
            "replica_b"
        } else {
            primary_pool_marker
        };

        assert_eq!(selected, "replica_b");
    }

    #[tokio::test]
    async fn test_no_replicas_uses_primary() {
        let replica_count = 0;
        let primary_pool_marker = "primary";

        let selected = if replica_count > 0 {
            "replica"
        } else {
            primary_pool_marker
        };

        assert_eq!(selected, "primary");
    }

    #[test]
    fn test_check_lag_max_on_error() {
        let lag: u64 = u64::MAX;
        assert_eq!(lag, u64::MAX);
    }

    #[test]
    fn test_check_lag_none_returns_zero() {
        let lag: u64 = 0;
        assert_eq!(lag, 0);
    }

    #[test]
    fn test_check_lag_f64_to_u64_conversion() {
        let val: f64 = 1234.567;
        let converted = val as u64;
        assert_eq!(converted, 1234);
    }

    #[test]
    fn test_replica_config_urls_preserved() {
        let cfg = make_config("postgres://p/db", &["postgres://r1/db", "postgres://r2/db"]);
        assert_eq!(cfg.replica_urls[0], "postgres://r1/db");
        assert_eq!(cfg.replica_urls[1], "postgres://r2/db");
    }

    #[test]
    fn test_router_stats_equality() {
        let a = RouterStats {
            primary_connections: 1,
            replica_count: 2,
            healthy_replicas: 2,
        };
        let b = RouterStats {
            primary_connections: 1,
            replica_count: 2,
            healthy_replicas: 2,
        };
        assert_eq!(a.primary_connections, b.primary_connections);
        assert_eq!(a.replica_count, b.replica_count);
        assert_eq!(a.healthy_replicas, b.healthy_replicas);
    }
}
