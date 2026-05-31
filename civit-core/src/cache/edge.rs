#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use tracing::info;

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeNodeStatus {
    Active,
    Draining,
    Offline,
}

pub struct EdgeNode {
    pub id: String,
    pub region: String,
    pub endpoint: String,
    pub status: EdgeNodeStatus,
    pub capacity_bytes: u64,
    pub used_bytes: Arc<AtomicU64>,
    pub last_heartbeat: Arc<AtomicI64>,
}

impl EdgeNode {
    pub fn new(id: String, region: String, endpoint: String, capacity_bytes: u64) -> Self {
        Self {
            id,
            region,
            endpoint,
            status: EdgeNodeStatus::Active,
            capacity_bytes,
            used_bytes: Arc::new(AtomicU64::new(0)),
            last_heartbeat: Arc::new(AtomicI64::new(Utc::now().timestamp())),
        }
    }

    pub fn heartbeat(&self) {
        self.last_heartbeat
            .store(Utc::now().timestamp(), Ordering::Relaxed);
    }

    pub fn add_usage(&self, bytes: u64) {
        self.used_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn remove_usage(&self, bytes: u64) {
        self.used_bytes.fetch_sub(bytes, Ordering::Relaxed);
    }
}

pub struct CacheEntry {
    pub data: Vec<u8>,
    pub compressed: bool,
    pub compressed_size: usize,
    pub original_size: usize,
    pub created_at: DateTime<Utc>,
    pub access_count: Arc<AtomicU64>,
    pub node_id: String,
}

pub struct EdgeCacheManager {
    nodes: DashMap<String, EdgeNode>,
    cache: DashMap<String, CacheEntry>,
    hit_count: AtomicU64,
    miss_count: AtomicU64,
}

pub struct EdgeCacheStats {
    pub total_nodes: usize,
    pub active_nodes: usize,
    pub total_entries: usize,
    pub hit_rate: f64,
    pub total_size_bytes: u64,
    pub dedup_savings_bytes: u64,
}

impl EdgeCacheManager {
    pub fn new() -> Self {
        Self {
            nodes: DashMap::new(),
            cache: DashMap::new(),
            hit_count: AtomicU64::new(0),
            miss_count: AtomicU64::new(0),
        }
    }

    pub fn register_node(&self, id: String, region: String, endpoint: String, capacity_bytes: u64) {
        info!(node_id = %id, region = %region, "registered edge node");
        let node = EdgeNode::new(id, region, endpoint, capacity_bytes);
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn deregister_node(&self, id: &str) -> bool {
        if let Some(mut node) = self.nodes.get_mut(id) {
            node.status = EdgeNodeStatus::Draining;
            info!(node_id = %id, "deregistered edge node");
            drop(node);
            self.nodes.remove(id);
            return true;
        }
        false
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        if let Some(entry) = self.cache.get(key) {
            entry.access_count.fetch_add(1, Ordering::Relaxed);
            self.hit_count.fetch_add(1, Ordering::Relaxed);
            if entry.compressed {
                let cursor = std::io::Cursor::new(&entry.data);
                return zstd::decode_all(cursor).ok();
            }
            return Some(entry.data.clone());
        }
        self.miss_count.fetch_add(1, Ordering::Relaxed);
        None
    }

    /// Minimum data size (bytes) below which compression is skipped.
    const COMPRESSION_THRESHOLD: usize = 512;

    pub fn put(&self, key: String, data: Vec<u8>) {
        let original_size = data.len();

        // Attempt compression only for large payloads.
        if data.len() >= Self::COMPRESSION_THRESHOLD {
            if let Ok(encoded) = zstd::encode_all(std::io::Cursor::new(&data), 3) {
                if encoded.len() < original_size {
                    // Compression saved space -- use it.
                    let encoded_len = encoded.len();
                    let node_id = self.pick_node_id();
                    let node = self.nodes.get(&node_id);
                    if let Some(ref n) = node {
                        n.add_usage(encoded_len as u64);
                    }
                    drop(node);
                    let entry = CacheEntry {
                        data: encoded,
                        compressed: true,
                        compressed_size: encoded_len,
                        original_size,
                        created_at: Utc::now(),
                        access_count: Arc::new(AtomicU64::new(0)),
                        node_id,
                    };
                    self.cache.insert(key, entry);
                    return;
                }
                // Compression didn't help -- fall through to uncompressed.
            }
        }

        // No compression or compression didn't reduce size.
        let node_id = self.pick_node_id();
        let node = self.nodes.get(&node_id);
        if let Some(ref n) = node {
            n.add_usage(data.len() as u64);
        }
        drop(node);
        let entry = CacheEntry {
            data,
            compressed: false,
            compressed_size: original_size,
            original_size,
            created_at: Utc::now(),
            access_count: Arc::new(AtomicU64::new(0)),
            node_id,
        };
        self.cache.insert(key, entry);
    }

    fn pick_node_id(&self) -> String {
        self.nodes
            .iter()
            .next()
            .map(|n| n.id.clone())
            .unwrap_or_default()
    }

    pub fn invalidate(&self, key: &str) -> bool {
        if let Some(entry) = self.cache.remove(key) {
            let node = self.nodes.get(&entry.1.node_id);
            if let Some(ref n) = node {
                n.remove_usage(entry.1.data.len() as u64);
            }
            return true;
        }
        false
    }

    pub fn invalidate_pattern(&self, pattern: &str) -> usize {
        let mut count = 0;
        let keys: Vec<String> = self
            .cache
            .iter()
            .filter(|entry| {
                if pattern == "*" {
                    return true;
                }
                entry.key().contains(pattern)
            })
            .map(|entry| entry.key().clone())
            .collect();

        for key in keys {
            if self.invalidate(&key) {
                count += 1;
            }
        }
        count
    }

    pub fn node_status(&self, id: &str) -> Option<EdgeNodeStatus> {
        self.nodes.get(id).map(|n| n.status.clone())
    }

    pub fn hit_rate(&self) -> f64 {
        let hits = self.hit_count.load(Ordering::Relaxed);
        let misses = self.miss_count.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            return 0.0;
        }
        hits as f64 / total as f64
    }

    pub fn total_size(&self) -> u64 {
        self.cache
            .iter()
            .map(|entry| entry.value().data.len() as u64)
            .sum()
    }

    pub fn stats(&self) -> EdgeCacheStats {
        let active_nodes = self
            .nodes
            .iter()
            .filter(|n| n.value().status == EdgeNodeStatus::Active)
            .count();

        let total_used: u64 = self
            .nodes
            .iter()
            .map(|n| n.value().used_bytes.load(Ordering::Relaxed))
            .sum();

        let total_capacity: u64 = self.nodes.iter().map(|n| n.value().capacity_bytes).sum();
        let dedup_savings = total_capacity.saturating_sub(total_used);

        EdgeCacheStats {
            total_nodes: self.nodes.len(),
            active_nodes,
            total_entries: self.cache.len(),
            hit_rate: self.hit_rate(),
            total_size_bytes: self.total_size(),
            dedup_savings_bytes: dedup_savings,
        }
    }
}

impl Default for EdgeCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_node() {
        let mgr = EdgeCacheManager::new();
        mgr.register_node(
            "node-1".into(),
            "us-east".into(),
            "http://node1:8080".into(),
            1024,
        );
        assert_eq!(mgr.node_status("node-1"), Some(EdgeNodeStatus::Active));
        let stats = mgr.stats();
        assert_eq!(stats.total_nodes, 1);
        assert_eq!(stats.active_nodes, 1);
    }

    #[test]
    fn test_deregister_node() {
        let mgr = EdgeCacheManager::new();
        mgr.register_node(
            "node-1".into(),
            "us-east".into(),
            "http://node1:8080".into(),
            1024,
        );
        assert!(mgr.deregister_node("node-1"));
        assert_eq!(mgr.node_status("node-1"), None);
        assert!(!mgr.deregister_node("node-1"));
    }

    #[test]
    fn test_put_and_get() {
        let mgr = EdgeCacheManager::new();
        mgr.register_node(
            "node-1".into(),
            "us-east".into(),
            "http://node1:8080".into(),
            1024,
        );
        mgr.put("key-1".into(), b"hello world".to_vec());
        let val = mgr.get("key-1").unwrap();
        assert_eq!(val, b"hello world");
    }

    #[test]
    fn test_cache_miss() {
        let mgr = EdgeCacheManager::new();
        assert!(mgr.get("missing").is_none());
        let stats = mgr.stats();
        assert_eq!(stats.hit_rate, 0.0);
    }

    #[test]
    fn test_invalidate() {
        let mgr = EdgeCacheManager::new();
        mgr.register_node(
            "node-1".into(),
            "us-east".into(),
            "http://node1:8080".into(),
            1024,
        );
        mgr.put("key-1".into(), b"data".to_vec());
        assert!(mgr.invalidate("key-1"));
        assert!(mgr.get("key-1").is_none());
        assert!(!mgr.invalidate("key-1"));
    }

    #[test]
    fn test_invalidate_pattern() {
        let mgr = EdgeCacheManager::new();
        mgr.register_node(
            "node-1".into(),
            "us-east".into(),
            "http://node1:8080".into(),
            1024,
        );
        mgr.put("repo-a:data".into(), b"1".to_vec());
        mgr.put("repo-a:meta".into(), b"2".to_vec());
        mgr.put("repo-b:data".into(), b"3".to_vec());
        let count = mgr.invalidate_pattern("repo-a");
        assert_eq!(count, 2);
        assert!(mgr.get("repo-b:data").is_some());
    }

    #[test]
    fn test_invalidate_pattern_wildcard() {
        let mgr = EdgeCacheManager::new();
        mgr.register_node(
            "node-1".into(),
            "us-east".into(),
            "http://node1:8080".into(),
            1024,
        );
        mgr.put("key-1".into(), b"1".to_vec());
        mgr.put("key-2".into(), b"2".to_vec());
        let count = mgr.invalidate_pattern("*");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_hit_rate() {
        let mgr = EdgeCacheManager::new();
        mgr.register_node(
            "node-1".into(),
            "us-east".into(),
            "http://node1:8080".into(),
            1024,
        );
        mgr.put("key-1".into(), b"data".to_vec());
        mgr.get("key-1");
        mgr.get("key-1");
        mgr.get("missing");
        let rate = mgr.hit_rate();
        assert!((rate - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_total_size() {
        let mgr = EdgeCacheManager::new();
        mgr.register_node(
            "node-1".into(),
            "us-east".into(),
            "http://node1:8080".into(),
            10240,
        );
        mgr.put("k1".into(), vec![0u8; 100]);
        mgr.put("k2".into(), vec![0u8; 200]);
        assert_eq!(mgr.total_size(), 300);
    }

    #[test]
    fn test_stats() {
        let mgr = EdgeCacheManager::new();
        mgr.register_node(
            "node-1".into(),
            "us-east".into(),
            "http://node1:8080".into(),
            10240,
        );
        mgr.register_node(
            "node-2".into(),
            "eu-west".into(),
            "http://node2:8080".into(),
            10240,
        );
        mgr.put("k1".into(), vec![0u8; 100]);
        let stats = mgr.stats();
        assert_eq!(stats.total_nodes, 2);
        assert_eq!(stats.active_nodes, 2);
        assert_eq!(stats.total_entries, 1);
    }

    #[test]
    fn test_edge_node_heartbeat() {
        let mgr = EdgeCacheManager::new();
        mgr.register_node(
            "node-1".into(),
            "us-east".into(),
            "http://node1:8080".into(),
            1024,
        );
        if let Some(node) = mgr.nodes.get("node-1") {
            let ts_before = node.last_heartbeat.load(Ordering::Relaxed);
            node.heartbeat();
            let ts_after = node.last_heartbeat.load(Ordering::Relaxed);
            assert!(ts_after >= ts_before);
        }
    }

    #[test]
    fn test_put_compresses_large_data() {
        let mgr = EdgeCacheManager::new();
        mgr.register_node(
            "node-1".into(),
            "us-east".into(),
            "http://node1:8080".into(),
            1_048_576,
        );
        // 8KB of repetitive data -- should compress well
        let data = "abcdefghij".repeat(1024).into_bytes();
        mgr.put("big-key".into(), data.clone());
        let val = mgr.get("big-key").unwrap();
        assert_eq!(val, data);
        if let Some(entry) = mgr.cache.get("big-key") {
            assert!(entry.compressed, "large entry should be compressed");
            assert!(entry.compressed_size < entry.original_size);
        }
    }

    #[test]
    fn test_put_skips_compression_for_small_data() {
        let mgr = EdgeCacheManager::new();
        mgr.register_node(
            "node-1".into(),
            "us-east".into(),
            "http://node1:8080".into(),
            1024,
        );
        mgr.put("tiny".into(), b"hello".to_vec());
        if let Some(entry) = mgr.cache.get("tiny") {
            assert!(!entry.compressed, "small entry should not be compressed");
        }
        assert_eq!(mgr.get("tiny").unwrap(), b"hello");
    }
}
