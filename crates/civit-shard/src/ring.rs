use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::hash::Hash;

/// A consistent hash ring that distributes keys across nodes using
/// virtual nodes (vnodes) for improved uniformity.
///
/// Generic over the node identifier type `T`. Nodes are placed on the ring
/// at multiple positions (vnodes) to ensure even key distribution.
#[derive(Debug, Clone)]
pub struct ConsistentRing<T> {
    ring: BTreeMap<u64, T>,
    vnodes_per_shard: u32,
}

impl<T: Clone + Eq + Hash + Debug> ConsistentRing<T> {
    /// Create a new empty consistent hash ring.
    ///
    /// `vnodes_per_shard` controls how many virtual nodes each physical node
    /// occupies on the ring. Higher values improve distribution uniformity at
    /// the cost of memory and lookup time. The design doc recommends 256.
    pub fn new(vnodes_per_shard: u32) -> Self {
        Self {
            ring: BTreeMap::new(),
            vnodes_per_shard,
        }
    }

    /// Add a node to the ring, placing it at `vnodes_per_shard` positions.
    pub fn add_node(&mut self, node: T) {
        for i in 0..self.vnodes_per_shard {
            let key = format!("{}#{i}", Self::node_to_prefix(&node));
            let hash = Self::hash_key(&key);
            self.ring.insert(hash, node.clone());
        }
    }

    /// Remove a node from the ring, removing all its virtual nodes.
    pub fn remove_node(&mut self, node: &T) {
        for i in 0..self.vnodes_per_shard {
            let key = format!("{}#{i}", Self::node_to_prefix(node));
            let hash = Self::hash_key(&key);
            self.ring.remove(&hash);
        }
    }

    /// Find the node responsible for the given key.
    ///
    /// Walks clockwise from the key's hash position on the ring.
    /// Returns `None` if the ring is empty.
    pub fn get_node(&self, key: &str) -> Option<&T> {
        if self.ring.is_empty() {
            return None;
        }
        let hash = Self::hash_key(key);
        // Find the first node with hash >= key hash, wrapping around
        self.ring
            .range(hash..)
            .next()
            .or_else(|| self.ring.iter().next())
            .map(|(_, node)| node)
    }

    /// Find `replicas` distinct nodes responsible for the given key.
    ///
    /// Walks clockwise from the key's hash position, collecting distinct nodes.
    /// Returns fewer nodes if the ring has fewer distinct nodes than requested.
    pub fn get_nodes(&self, key: &str, replicas: usize) -> Vec<&T> {
        if self.ring.is_empty() {
            return Vec::new();
        }
        let hash = Self::hash_key(key);
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();

        // Walk clockwise from the hash position, wrapping around
        for (_, node) in self.ring.range(hash..).chain(self.ring.iter()) {
            if seen.insert(node) {
                result.push(node);
                if result.len() >= replicas {
                    break;
                }
            }
        }
        result
    }

    /// Returns the number of distinct nodes in the ring.
    pub fn node_count(&self) -> usize {
        if self.ring.is_empty() {
            return 0;
        }
        // Count distinct nodes by collecting into a set using value equality
        let mut seen = std::collections::HashSet::new();
        for node in self.ring.values() {
            seen.insert(node);
        }
        seen.len()
    }

    /// Returns the total number of virtual nodes on the ring.
    pub fn vnode_count(&self) -> usize {
        self.ring.len()
    }

    /// Hash a string key to a u64 using SHA-256, taking the first 8 bytes.
    fn hash_key(key: &str) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let result = hasher.finalize();
        u64::from_be_bytes(result[..8].try_into().expect("SHA-256 produces >= 8 bytes"))
    }

    /// Convert a node to a string prefix for vnode key generation.
    fn node_to_prefix(node: &T) -> String {
        format!("{node:?}")
    }
}

impl<T: Clone + Eq + Hash + Debug + Default> ConsistentRing<T> {
    /// Create a ring pre-populated with `count` default-initialized nodes.
    pub fn with_default_nodes(count: u32, vnodes_per_shard: u32) -> Self {
        let mut ring = Self::new(vnodes_per_shard);
        for _ in 0..count {
            ring.add_node(T::default());
        }
        ring
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_ring_returns_none() {
        let ring: ConsistentRing<String> = ConsistentRing::new(10);
        assert!(ring.get_node("any-key").is_none());
        assert!(ring.get_nodes("any-key", 3).is_empty());
    }

    #[test]
    fn test_single_node_always_returns_that_node() {
        let mut ring = ConsistentRing::new(10);
        ring.add_node("shard-0".to_string());
        assert_eq!(ring.get_node("any-key"), Some(&"shard-0".to_string()));
        assert_eq!(ring.get_node("another-key"), Some(&"shard-0".to_string()));
    }

    #[test]
    fn test_add_node_increases_vnode_count() {
        let mut ring = ConsistentRing::new(16);
        assert_eq!(ring.vnode_count(), 0);
        ring.add_node("shard-0".to_string());
        assert_eq!(ring.vnode_count(), 16);
        ring.add_node("shard-1".to_string());
        assert_eq!(ring.vnode_count(), 32);
    }

    #[test]
    fn test_remove_node_decreases_vnode_count() {
        let mut ring = ConsistentRing::new(16);
        ring.add_node("shard-0".to_string());
        ring.add_node("shard-1".to_string());
        assert_eq!(ring.vnode_count(), 32);
        ring.remove_node(&"shard-0".to_string());
        assert_eq!(ring.vnode_count(), 16);
        assert_eq!(ring.node_count(), 1);
    }

    #[test]
    fn test_remove_last_node_empty_ring() {
        let mut ring = ConsistentRing::new(10);
        ring.add_node("shard-0".to_string());
        ring.remove_node(&"shard-0".to_string());
        assert!(ring.get_node("key").is_none());
        assert_eq!(ring.vnode_count(), 0);
    }

    #[test]
    fn test_consistent_routing() {
        let mut ring = ConsistentRing::new(64);
        ring.add_node("shard-0".to_string());
        ring.add_node("shard-1".to_string());
        ring.add_node("shard-2".to_string());
        ring.add_node("shard-3".to_string());

        // The same key always routes to the same node
        let first = ring.get_node("test-repo-key").unwrap().clone();
        for _ in 0..100 {
            let node = ring.get_node("test-repo-key").unwrap();
            assert_eq!(node, &first);
        }
    }

    #[test]
    fn test_key_distribution_uniformity() {
        let mut ring = ConsistentRing::new(256);
        for i in 0..4 {
            ring.add_node(format!("shard-{i}"));
        }

        let mut counts = std::collections::HashMap::new();
        let num_keys = 10_000;

        for i in 0..num_keys {
            let key = format!("repo-{i}");
            let node = ring.get_node(&key).unwrap().clone();
            *counts.entry(node).or_insert(0) += 1;
        }

        // With 256 vnodes per shard and 10K keys, each shard should get
        // roughly 2500 keys. Allow 15% deviation.
        let expected = num_keys / 4;
        let tolerance = (expected as f64 * 0.15) as u32;

        for (shard, count) in &counts {
            assert!(
                (*count as i64 - expected as i64).unsigned_abs() <= tolerance as u64,
                "Shard {shard} got {count} keys, expected ~{expected} (tolerance {tolerance})"
            );
        }
        assert_eq!(counts.len(), 4, "All 4 shards should receive keys");
    }

    #[test]
    fn test_replication_returns_distinct_nodes() {
        let mut ring = ConsistentRing::new(64);
        for i in 0..4 {
            ring.add_node(format!("shard-{i}"));
        }

        let nodes = ring.get_nodes("some-key", 3);
        assert_eq!(nodes.len(), 3);

        // All nodes must be distinct
        let mut seen = std::collections::HashSet::new();
        for node in &nodes {
            assert!(seen.insert(*node), "duplicate node in replication: {node}");
        }
    }

    #[test]
    fn test_replication_requests_more_than_available() {
        let mut ring = ConsistentRing::new(10);
        ring.add_node("shard-0".to_string());

        let nodes = ring.get_nodes("key", 5);
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn test_adding_node_only_reassigns_fraction_of_keys() {
        let mut ring = ConsistentRing::new(256);
        ring.add_node("shard-0".to_string());
        ring.add_node("shard-1".to_string());
        ring.add_node("shard-2".to_string());

        // Record assignments before adding shard-3
        let mut before = std::collections::HashMap::new();
        for i in 0..10_000 {
            let key = format!("repo-{i}");
            let node = ring.get_node(&key).unwrap().clone();
            before.insert(key, node);
        }

        ring.add_node("shard-3".to_string());

        let mut changed = 0;
        for i in 0..10_000 {
            let key = format!("repo-{i}");
            let node = ring.get_node(&key).unwrap().clone();
            if node != *before.get(&key).unwrap() {
                changed += 1;
            }
        }

        // With consistent hashing, adding a 4th shard to 3 should reassign
        // roughly 1/4 of keys (25%), allow up to 35%
        let ratio = changed as f64 / 10_000.0;
        assert!(
            ratio > 0.10 && ratio < 0.35,
            "Expected ~25% key reassignment, got {:.1}%",
            ratio * 100.0
        );
    }

    #[test]
    fn test_remove_node_redistributes_keys() {
        let mut ring = ConsistentRing::new(256);
        for i in 0..4 {
            ring.add_node(format!("shard-{i}"));
        }

        let mut before = std::collections::HashMap::new();
        for i in 0..5_000 {
            let key = format!("repo-{i}");
            let node = ring.get_node(&key).unwrap().clone();
            before.insert(key, node);
        }

        ring.remove_node(&"shard-2".to_string());

        // All keys that were on shard-2 must now be on a different shard
        for i in 0..5_000 {
            let key = format!("repo-{i}");
            let node = ring.get_node(&key).unwrap().clone();
            if before.get(&key).unwrap() == &"shard-2".to_string() {
                assert_ne!(node, "shard-2", "Key {key} still on removed shard");
            }
        }

        // No keys should be lost
        for i in 0..5_000 {
            let key = format!("repo-{i}");
            assert!(ring.get_node(&key).is_some());
        }
    }

    #[test]
    fn test_node_count() {
        let mut ring = ConsistentRing::new(10);
        assert_eq!(ring.node_count(), 0);
        ring.add_node("a".to_string());
        assert_eq!(ring.node_count(), 1);
        ring.add_node("b".to_string());
        assert_eq!(ring.node_count(), 2);
        ring.remove_node(&"a".to_string());
        assert_eq!(ring.node_count(), 1);
    }

    #[test]
    fn test_deterministic_hash() {
        let h1 = ConsistentRing::<String>::hash_key("test-key");
        let h2 = ConsistentRing::<String>::hash_key("test-key");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_different_keys_different_hashes() {
        let h1 = ConsistentRing::<String>::hash_key("key-a");
        let h2 = ConsistentRing::<String>::hash_key("key-b");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_ring_with_custom_type() {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        struct ShardId(u32);

        let mut ring = ConsistentRing::new(32);
        ring.add_node(ShardId(0));
        ring.add_node(ShardId(1));

        let node = ring.get_node("my-key").unwrap();
        assert!(node.0 == 0 || node.0 == 1);
    }

    #[test]
    fn test_ring_clone() {
        let mut ring = ConsistentRing::new(16);
        ring.add_node("shard-0".to_string());
        ring.add_node("shard-1".to_string());

        let ring2 = ring.clone();
        assert_eq!(ring.get_node("key"), ring2.get_node("key"));
        assert_eq!(ring.vnode_count(), ring2.vnode_count());
    }
}
