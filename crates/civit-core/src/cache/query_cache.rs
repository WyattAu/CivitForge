#![forbid(unsafe_code)]

use std::time::{Duration, Instant};
use dashmap::DashMap;
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, warn};

/// Default TTL values for different query categories (in seconds).
pub mod ttl {
    use std::time::Duration;

    /// Repository list queries - moderate staleness tolerance.
    pub const REPO_LIST: Duration = Duration::from_secs(60);
    /// Issue list with filters - short cache for freshness.
    pub const ISSUE_LIST: Duration = Duration::from_secs(30);
    /// User permissions - longer cache, changes infrequently.
    pub const USER_PERMISSIONS: Duration = Duration::from_secs(300);
    /// Pipeline status - very short cache for real-time feel.
    pub const PIPELINE_STATUS: Duration = Duration::from_secs(10);
    /// Default TTL for unclassified queries.
    pub const DEFAULT: Duration = Duration::from_secs(60);
}

/// A single cache entry with expiration tracking.
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}

/// In-memory LRU query cache backed by DashMap.
///
/// Provides typed get/set/invalidation for hot database query results.
/// Each entry has a per-key TTL. Expired entries are lazily evicted
/// on access and periodically via `evict_expired`.
pub struct QueryCache {
    /// The actual cache storage, keyed by query cache key.
    store: DashMap<String, CacheEntry<Vec<u8>>>,
    /// Maximum number of entries before eviction pressure starts.
    max_entries: usize,
    /// Total gets for hit-rate stats.
    hits: std::sync::atomic::AtomicU64,
    /// Total gets for hit-rate stats.
    misses: std::sync::atomic::AtomicU64,
}

impl QueryCache {
    /// Create a new QueryCache with the given capacity.
    pub fn new(max_entries: usize) -> Self {
        Self {
            store: DashMap::with_capacity(max_entries.min(4096)),
            max_entries,
            hits: std::sync::atomic::AtomicU64::new(0),
            misses: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Get a cached value by key, deserializing from JSON.
    /// Returns `None` on miss or if the entry has expired.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        if let Some(entry) = self.store.get(key) {
            if entry.is_expired() {
                // Lazy eviction
                drop(entry);
                self.store.remove(key);
                self.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                return None;
            }
            self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            match serde_json::from_slice(&entry.value) {
                Ok(v) => {
                    debug!(key = %key, "query cache hit");
                    Some(v)
                }
                Err(e) => {
                    warn!(key = %key, error = %e, "failed to deserialize cached query");
                    None
                }
            }
        } else {
            self.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            None
        }
    }

    /// Get raw bytes by key (for callers that handle serialization themselves).
    pub fn get_raw(&self, key: &str) -> Option<Vec<u8>> {
        if let Some(entry) = self.store.get(key) {
            if entry.is_expired() {
                drop(entry);
                self.store.remove(key);
                self.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                return None;
            }
            self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Some(entry.value.clone())
        } else {
            self.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            None
        }
    }

    /// Cache a value with the given TTL.
    pub fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) {
        match serde_json::to_vec(value) {
            Ok(bytes) => {
                // Evict oldest if at capacity
                if self.store.len() >= self.max_entries {
                    self.evict_expired();
                    // If still at capacity, remove a random entry
                    if self.store.len() >= self.max_entries {
                        if let Some(entry) = self.store.iter().next() {
                            let k = entry.key().clone();
                            drop(entry);
                            self.store.remove(&k);
                        }
                    }
                }
                self.store.insert(key.to_string(), CacheEntry::new(bytes, ttl));
                debug!(key = %key, ttl_ms = ttl.as_millis(), "query cache set");
            }
            Err(e) => {
                warn!(key = %key, error = %e, "failed to serialize query for cache");
            }
        }
    }

    /// Cache raw bytes with the given TTL.
    pub fn set_raw(&self, key: &str, data: Vec<u8>, ttl: Duration) {
        if self.store.len() >= self.max_entries {
            self.evict_expired();
            if self.store.len() >= self.max_entries {
                if let Some(entry) = self.store.iter().next() {
                    let k = entry.key().clone();
                    drop(entry);
                    self.store.remove(&k);
                }
            }
        }
        self.store.insert(key.to_string(), CacheEntry::new(data, ttl));
    }

    /// Invalidate a single key.
    pub fn invalidate(&self, key: &str) -> bool {
        let removed = self.store.remove(key).is_some();
        if removed {
            debug!(key = %key, "query cache invalidated");
        }
        removed
    }

    /// Invalidate all keys whose key contains the given pattern string.
    pub fn invalidate_pattern(&self, pattern: &str) -> usize {
        let keys_to_remove: Vec<String> = self
            .store
            .iter()
            .filter(|entry| entry.key().contains(pattern))
            .map(|entry| entry.key().clone())
            .collect();
        let count = keys_to_remove.len();
        for key in &keys_to_remove {
            self.store.remove(key);
        }
        if count > 0 {
            debug!(pattern = %pattern, count, "query cache pattern invalidation");
        }
        count
    }

    /// Remove all expired entries. Returns the number evicted.
    pub fn evict_expired(&self) -> usize {
        let mut evicted = 0usize;
        let mut keys_to_remove = Vec::new();
        for entry in self.store.iter() {
            if entry.value().is_expired() {
                keys_to_remove.push(entry.key().clone());
            }
        }
        for key in keys_to_remove {
            self.store.remove(&key);
            evicted += 1;
        }
        if evicted > 0 {
            debug!(evicted, "expired query cache entries evicted");
        }
        evicted
    }

    /// Clear all entries.
    pub fn clear(&self) {
        self.store.clear();
        debug!("query cache cleared");
    }

    /// Current number of entries (including expired not yet evicted).
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// Hit rate as a fraction (0.0 - 1.0).
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(std::sync::atomic::Ordering::Relaxed);
        let misses = self.misses.load(std::sync::atomic::Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            return 0.0;
        }
        hits as f64 / total as f64
    }

    /// Reset hit/miss counters.
    pub fn reset_stats(&self) {
        self.hits.store(0, std::sync::atomic::Ordering::Relaxed);
        self.misses.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Default for QueryCache {
    fn default() -> Self {
        Self::new(10_000)
    }
}

/// Build a cache key for repository list queries.
pub fn repo_list_key(owner_id: &str, page: u64, per_page: u64) -> String {
    format!("repo:list:{owner_id}:{page}:{per_page}")
}

/// Build a cache key for issue list queries.
pub fn issue_list_key(repo_id: &str, status: Option<&str>, page: u64, per_page: u64) -> String {
    match status {
        Some(s) => format!("issue:list:{repo_id}:{s}:{page}:{per_page}"),
        None => format!("issue:list:{repo_id}:all:{page}:{per_page}"),
    }
}

/// Build a cache key for user permissions.
pub fn user_permissions_key(user_id: &str) -> String {
    format!("user:perms:{user_id}")
}

/// Build a cache key for pipeline status.
pub fn pipeline_status_key(repo_id: &str) -> String {
    format!("pipeline:status:{repo_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_cache_set_and_get() {
        let cache = QueryCache::new(100);
        cache.set("key1", &"hello".to_string(), Duration::from_secs(60));
        let result: Option<String> = cache.get("key1");
        assert_eq!(result, Some("hello".to_string()));
    }

    #[test]
    fn test_query_cache_miss() {
        let cache = QueryCache::new(100);
        let result: Option<String> = cache.get("nonexistent");
        assert_eq!(result, None);
    }

    #[test]
    fn test_query_cache_expiry() {
        let cache = QueryCache::new(100);
        cache.set("key1", &"hello".to_string(), Duration::from_millis(1));
        // Wait for expiry
        std::thread::sleep(Duration::from_millis(10));
        let result: Option<String> = cache.get("key1");
        assert_eq!(result, None);
    }

    #[test]
    fn test_query_cache_invalidate() {
        let cache = QueryCache::new(100);
        cache.set("key1", &"hello".to_string(), Duration::from_secs(60));
        assert!(cache.invalidate("key1"));
        let result: Option<String> = cache.get("key1");
        assert_eq!(result, None);
    }

    #[test]
    fn test_query_cache_invalidate_pattern() {
        let cache = QueryCache::new(100);
        cache.set("repo:list:1:0:10", &"data1".to_string(), Duration::from_secs(60));
        cache.set("repo:list:2:0:10", &"data2".to_string(), Duration::from_secs(60));
        cache.set("issue:list:1:0:10", &"data3".to_string(), Duration::from_secs(60));
        let count = cache.invalidate_pattern("repo:list");
        assert_eq!(count, 2);
        let result: Option<String> = cache.get("repo:list:1:0:10");
        assert_eq!(result, None);
        let result: Option<String> = cache.get("issue:list:1:0:10");
        assert_eq!(result, Some("data3".to_string()));
    }

    #[test]
    fn test_query_cache_evict_expired() {
        let cache = QueryCache::new(100);
        cache.set("key1", &"v1".to_string(), Duration::from_millis(1));
        cache.set("key2", &"v2".to_string(), Duration::from_secs(60));
        std::thread::sleep(Duration::from_millis(10));
        let evicted = cache.evict_expired();
        assert_eq!(evicted, 1);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_query_cache_hit_rate() {
        let cache = QueryCache::new(100);
        cache.set("key1", &"hello".to_string(), Duration::from_secs(60));
        let _: Option<String> = cache.get("key1"); // hit
        let _: Option<String> = cache.get("key1"); // hit
        let _: Option<String> = cache.get("miss"); // miss
        assert!((cache.hit_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_cache_key_builders() {
        assert_eq!(repo_list_key("user1", 0, 10), "repo:list:user1:0:10");
        assert_eq!(issue_list_key("repo1", Some("open"), 0, 10), "issue:list:repo1:open:0:10");
        assert_eq!(issue_list_key("repo1", None, 0, 10), "issue:list:repo1:all:0:10");
        assert_eq!(user_permissions_key("user1"), "user:perms:user1");
        assert_eq!(pipeline_status_key("repo1"), "pipeline:status:repo1");
    }

    #[test]
    fn test_raw_get_set() {
        let cache = QueryCache::new(100);
        let data = b"raw bytes".to_vec();
        cache.set_raw("raw1", data.clone(), Duration::from_secs(60));
        let result = cache.get_raw("raw1");
        assert_eq!(result, Some(data));
    }
}
