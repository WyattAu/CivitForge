use crate::ring::ConsistentRing;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShardRouterError {
    #[error("no healthy shards available")]
    NoHealthyShards,

    #[error("shard not found: {0}")]
    ShardNotFound(String),

    #[error("ring is empty")]
    EmptyRing,
}

/// Configuration for a single shard.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShardConfig {
    /// Unique identifier for the shard.
    pub id: String,
    /// Connection URL for the shard's primary (write) endpoint.
    pub url: String,
    /// Weight for weighted routing (higher weight = more keys routed here).
    pub weight: u32,
    /// Whether the shard is currently healthy and accepting traffic.
    pub is_healthy: bool,
    /// Optional region identifier for geo-aware routing.
    pub region: Option<String>,
    /// Maximum number of connections to this shard.
    pub max_connections: u32,
}

impl ShardConfig {
    pub fn new(id: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            url: url.into(),
            weight: 1,
            is_healthy: true,
            region: None,
            max_connections: 200,
        }
    }

    pub fn with_weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }

    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    pub fn with_max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Mark this shard as unhealthy (e.g., after a health check failure).
    pub fn mark_unhealthy(&mut self) {
        self.is_healthy = false;
    }

    /// Mark this shard as healthy again.
    pub fn mark_healthy(&mut self) {
        self.is_healthy = true;
    }
}

/// Routes keys to shards using a consistent hash ring with health-aware failover.
pub struct ShardRouter {
    ring: ConsistentRing<String>,
    shards: HashMap<String, ShardConfig>,
}

impl ShardRouter {
    /// Create a new shard router from a list of shard configurations.
    ///
    /// Only healthy shards are added to the routing ring.
    pub fn new(shards: Vec<ShardConfig>, vnodes_per_shard: u32) -> Self {
        let mut router = Self {
            ring: ConsistentRing::new(vnodes_per_shard),
            shards: HashMap::new(),
        };

        for shard in shards {
            if shard.is_healthy {
                router.ring.add_node(shard.id.clone());
            }
            router.shards.insert(shard.id.clone(), shard);
        }

        router
    }

    /// Route a key to the responsible shard.
    ///
    /// Returns an error if the ring is empty or no healthy shard is found.
    pub fn route(&self, key: &str) -> Result<&ShardConfig, ShardRouterError> {
        let shard_id = self.ring.get_node(key).ok_or(ShardRouterError::EmptyRing)?;

        self.shards
            .get(shard_id)
            .filter(|s| s.is_healthy)
            .ok_or_else(|| ShardRouterError::ShardNotFound(shard_id.clone()))
    }

    /// Route a key with fallback shards for replication or failover.
    ///
    /// Returns the primary shard and a list of fallback shards (distinct from
    /// the primary). Falls back to unhealthy shards if not enough healthy
    /// replicas exist.
    pub fn route_with_fallback(
        &self,
        key: &str,
        replicas: usize,
    ) -> Result<(&ShardConfig, Vec<&ShardConfig>), ShardRouterError> {
        let node_ids = self.ring.get_nodes(key, replicas);
        if node_ids.is_empty() {
            return Err(ShardRouterError::EmptyRing);
        }

        let primary_id = node_ids[0];
        let primary = self
            .shards
            .get(primary_id)
            .filter(|s| s.is_healthy)
            .ok_or_else(|| ShardRouterError::ShardNotFound(primary_id.clone()))?;

        let mut fallbacks = Vec::new();
        for &nid in &node_ids[1..] {
            if let Some(shard) = self.shards.get(nid)
                && shard.is_healthy
                && shard.id != primary.id
            {
                fallbacks.push(shard);
            }
        }

        // If we don't have enough healthy fallbacks, include unhealthy ones
        if fallbacks.len() < replicas - 1 {
            for &nid in &node_ids[1..] {
                if let Some(shard) = self.shards.get(nid)
                    && !shard.is_healthy
                    && shard.id != primary.id
                    && !fallbacks.iter().any(|f| f.id == shard.id)
                {
                    fallbacks.push(shard);
                }
            }
        }

        Ok((primary, fallbacks))
    }

    /// Mark a shard as unhealthy and remove it from the routing ring.
    pub fn mark_shard_unhealthy(&mut self, shard_id: &str) -> Result<(), ShardRouterError> {
        let shard = self
            .shards
            .get_mut(shard_id)
            .ok_or_else(|| ShardRouterError::ShardNotFound(shard_id.to_string()))?;

        shard.mark_unhealthy();
        self.ring.remove_node(&shard_id.to_string());
        Ok(())
    }

    /// Mark a shard as healthy and add it back to the routing ring.
    pub fn mark_shard_healthy(&mut self, shard_id: &str) -> Result<(), ShardRouterError> {
        let shard = self
            .shards
            .get_mut(shard_id)
            .ok_or_else(|| ShardRouterError::ShardNotFound(shard_id.to_string()))?;

        shard.mark_healthy();
        self.ring.add_node(shard_id.to_string());
        Ok(())
    }

    /// Get a shard config by ID.
    pub fn get_shard(&self, shard_id: &str) -> Option<&ShardConfig> {
        self.shards.get(shard_id)
    }

    /// Get all shard configs.
    pub fn all_shards(&self) -> Vec<&ShardConfig> {
        self.shards.values().collect()
    }

    /// Get all healthy shard configs.
    pub fn healthy_shards(&self) -> Vec<&ShardConfig> {
        self.shards.values().filter(|s| s.is_healthy).collect()
    }

    /// Get the number of healthy shards.
    pub fn healthy_shard_count(&self) -> usize {
        self.healthy_shards().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_shards() -> Vec<ShardConfig> {
        vec![
            ShardConfig::new("shard-0", "postgres://localhost:5432/shard0"),
            ShardConfig::new("shard-1", "postgres://localhost:5433/shard1"),
            ShardConfig::new("shard-2", "postgres://localhost:5434/shard2"),
            ShardConfig::new("shard-3", "postgres://localhost:5435/shard3"),
        ]
    }

    #[test]
    fn test_route_returns_consistent_results() {
        let router = ShardRouter::new(test_shards(), 256);
        let node1 = router.route("my-repo-key").unwrap().id.clone();
        let node2 = router.route("my-repo-key").unwrap().id.clone();
        assert_eq!(node1, node2);
    }

    #[test]
    fn test_route_distributes_across_shards() {
        let router = ShardRouter::new(test_shards(), 256);
        let mut counts = std::collections::HashMap::new();

        for i in 0..10_000 {
            let key = format!("repo-{i}");
            let shard_id = router.route(&key).unwrap().id.clone();
            *counts.entry(shard_id).or_insert(0) += 1;
        }

        // All 4 shards should receive keys
        assert_eq!(counts.len(), 4);
    }

    #[test]
    fn test_route_empty_ring() {
        let router = ShardRouter::new(vec![], 256);
        assert!(matches!(
            router.route("key"),
            Err(ShardRouterError::EmptyRing)
        ));
    }

    #[test]
    fn test_route_around_unhealthy_shard() {
        let mut shards = test_shards();
        shards[1].mark_unhealthy();

        let router = ShardRouter::new(shards, 256);

        // No key should route to the unhealthy shard
        for i in 0..1_000 {
            let key = format!("repo-{i}");
            let shard = router.route(&key).unwrap();
            assert_ne!(shard.id, "shard-1");
        }
    }

    #[test]
    fn test_mark_shard_unhealthy_removes_from_ring() {
        let mut router = ShardRouter::new(test_shards(), 256);
        assert_eq!(router.healthy_shard_count(), 4);

        router.mark_shard_unhealthy("shard-1").unwrap();
        assert_eq!(router.healthy_shard_count(), 3);

        for i in 0..1_000 {
            let key = format!("repo-{i}");
            let shard = router.route(&key).unwrap();
            assert_ne!(shard.id, "shard-1");
        }
    }

    #[test]
    fn test_mark_shard_healthy_restores_to_ring() {
        let mut router = ShardRouter::new(test_shards(), 256);
        router.mark_shard_unhealthy("shard-1").unwrap();
        assert_eq!(router.healthy_shard_count(), 3);

        router.mark_shard_healthy("shard-1").unwrap();
        assert_eq!(router.healthy_shard_count(), 4);

        // Now shard-1 should receive some keys again
        let mut found_shard1 = false;
        for i in 0..10_000 {
            let key = format!("repo-{i}");
            if router.route(&key).unwrap().id == "shard-1" {
                found_shard1 = true;
                break;
            }
        }
        assert!(found_shard1);
    }

    #[test]
    fn test_route_with_fallback_returns_multiple_shards() {
        let router = ShardRouter::new(test_shards(), 256);
        let (primary, fallbacks) = router.route_with_fallback("key", 3).unwrap();
        assert!(!primary.id.is_empty());
        assert!(fallbacks.len() <= 2);

        // All returned shards should be distinct
        let mut ids = vec![primary.id.clone()];
        for f in &fallbacks {
            ids.push(f.id.clone());
        }
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len());
    }

    #[test]
    fn test_route_with_fallback_excludes_primary_from_fallbacks() {
        let router = ShardRouter::new(test_shards(), 256);
        let (primary, fallbacks) = router.route_with_fallback("key", 4).unwrap();
        for f in &fallbacks {
            assert_ne!(f.id, primary.id);
        }
    }

    #[test]
    fn test_route_with_fallback_handles_unhealthy() {
        let mut shards = test_shards();
        shards[2].mark_unhealthy();
        let router = ShardRouter::new(shards, 256);

        let (primary, fallbacks) = router.route_with_fallback("key", 3).unwrap();
        assert!(primary.is_healthy);
        for f in &fallbacks {
            // Fallbacks should be healthy (we have 3 healthy shards total)
            assert!(f.is_healthy);
        }
    }

    #[test]
    fn test_shard_config_builder() {
        let shard = ShardConfig::new("s0", "postgres://localhost/db")
            .with_weight(5)
            .with_region("us-east-1")
            .with_max_connections(500);

        assert_eq!(shard.weight, 5);
        assert_eq!(shard.region.as_deref(), Some("us-east-1"));
        assert_eq!(shard.max_connections, 500);
        assert!(shard.is_healthy);
    }

    #[test]
    fn test_shard_config_mark_unhealthy_healthy() {
        let mut shard = ShardConfig::new("s0", "url");
        assert!(shard.is_healthy);
        shard.mark_unhealthy();
        assert!(!shard.is_healthy);
        shard.mark_healthy();
        assert!(shard.is_healthy);
    }

    #[test]
    fn test_all_shards_and_healthy_shards() {
        let mut shards = test_shards();
        shards[1].mark_unhealthy();
        let router = ShardRouter::new(shards, 256);

        assert_eq!(router.all_shards().len(), 4);
        assert_eq!(router.healthy_shards().len(), 3);
    }

    #[test]
    fn test_get_shard_by_id() {
        let router = ShardRouter::new(test_shards(), 256);
        assert!(router.get_shard("shard-0").is_some());
        assert!(router.get_shard("shard-99").is_none());
    }

    #[test]
    fn test_single_shard_routes_all_keys() {
        let shards = vec![ShardConfig::new("only-shard", "postgres://localhost/db")];
        let router = ShardRouter::new(shards, 256);

        for i in 0..100 {
            let key = format!("key-{i}");
            assert_eq!(router.route(&key).unwrap().id, "only-shard");
        }
    }

    #[test]
    fn test_mark_nonexistent_shard_unhealthy() {
        let mut router = ShardRouter::new(test_shards(), 256);
        assert!(matches!(
            router.mark_shard_unhealthy("shard-99"),
            Err(ShardRouterError::ShardNotFound(_))
        ));
    }

    #[test]
    fn test_weight_does_not_affect_ring_consistency() {
        let shards = vec![
            ShardConfig::new("s0", "url0").with_weight(1),
            ShardConfig::new("s1", "url1").with_weight(10),
        ];
        let router = ShardRouter::new(shards, 256);

        // Same key always routes to the same shard regardless of weights
        for _ in 0..100 {
            let result = router.route("deterministic-key").unwrap().id.clone();
            assert_eq!(result, router.route("deterministic-key").unwrap().id);
        }
    }

    #[test]
    fn test_shard_config_serialization() {
        let shard = ShardConfig::new("s0", "postgres://localhost/db")
            .with_weight(5)
            .with_region("eu-west-1");

        let json = serde_json::to_string(&shard).unwrap();
        let deserialized: ShardConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(shard, deserialized);
    }
}
