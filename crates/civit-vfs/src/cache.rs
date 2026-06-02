#![forbid(unsafe_code)]

use std::collections::{HashMap, VecDeque};
use tracing::debug;

#[derive(Debug, Clone)]
struct CacheEntry {
    data: Vec<u8>,
    size: usize,
}

pub struct LruCache {
    max_size: usize,
    current_size: usize,
    entries: HashMap<String, CacheEntry>,
    order: VecDeque<String>,
    hits: usize,
    misses: usize,
}

impl LruCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            current_size: 0,
            entries: HashMap::new(),
            order: VecDeque::new(),
            hits: 0,
            misses: 0,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&[u8]> {
        if let Some(entry) = self.entries.get(key) {
            self.order.retain(|k| k != key);
            self.order.push_back(key.to_string());
            self.hits += 1;
            Some(&entry.data)
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn insert(&mut self, key: String, data: Vec<u8>) {
        let size = data.len();
        if let Some(existing) = self.entries.remove(&key) {
            self.current_size -= existing.size;
            self.order.retain(|k| k != &key);
        }

        while self.current_size + size > self.max_size && !self.order.is_empty() {
            if let Some(evicted) = self.order.pop_front() {
                if let Some(entry) = self.entries.remove(&evicted) {
                    self.current_size -= entry.size;
                    debug!(key = %evicted, size = entry.size, "evicted cache entry");
                }
            }
        }

        self.entries.insert(key.clone(), CacheEntry { data, size });
        self.order.push_back(key);
        self.current_size += size;
    }

    pub fn remove(&mut self, key: &str) -> Option<Vec<u8>> {
        self.order.retain(|k| k != key);
        if let Some(entry) = self.entries.remove(key) {
            self.current_size -= entry.size;
            return Some(entry.data);
        }
        None
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.order.clear();
        self.current_size = 0;
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn size_bytes(&self) -> usize {
        self.current_size
    }

    pub fn utilization(&self) -> f64 {
        if self.max_size == 0 {
            return 0.0;
        }
        self.current_size as f64 / self.max_size as f64
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            return 0.0;
        }
        self.hits as f64 / total as f64
    }

    pub fn stats(&self) -> (usize, usize, f64) {
        (self.hits, self.misses, self.hit_rate())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut cache = LruCache::new(1024);
        cache.insert("key1".into(), b"hello".to_vec());
        let val = cache.get("key1");
        assert!(val.is_some());
        assert_eq!(val.unwrap(), b"hello");
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = LruCache::new(1024);
        assert!(cache.get("missing").is_none());
        assert_eq!(cache.misses, 1);
    }

    #[test]
    fn test_lru_eviction() {
        let mut cache = LruCache::new(10);
        cache.insert("a".into(), vec![0u8; 4]);
        cache.insert("b".into(), vec![0u8; 4]);
        cache.insert("c".into(), vec![0u8; 4]);
        assert!(cache.get("a").is_none()); // evicted (oldest)
        assert!(cache.get("b").is_some());
        assert!(cache.get("c").is_some());
    }

    #[test]
    fn test_lru_ordering() {
        let mut cache = LruCache::new(14);
        cache.insert("a".into(), vec![0u8; 5]);
        cache.insert("b".into(), vec![0u8; 5]);
        cache.get("a"); // touch a to make it most recent
        cache.insert("c".into(), vec![0u8; 5]);
        assert!(cache.get("b").is_none()); // b should be evicted (LRU)
        assert!(cache.get("a").is_some());
        assert!(cache.get("c").is_some());
    }

    #[test]
    fn test_remove() {
        let mut cache = LruCache::new(1024);
        cache.insert("key1".into(), b"data".to_vec());
        let removed = cache.remove("key1");
        assert!(removed.is_some());
        assert!(cache.get("key1").is_none());
    }

    #[test]
    fn test_clear() {
        let mut cache = LruCache::new(1024);
        cache.insert("a".into(), vec![1]);
        cache.insert("b".into(), vec![2]);
        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.size_bytes(), 0);
    }

    #[test]
    fn test_overwrite() {
        let mut cache = LruCache::new(1024);
        cache.insert("key".into(), vec![1, 2, 3]);
        cache.insert("key".into(), vec![4, 5]);
        let val = cache.get("key").unwrap();
        assert_eq!(val, &[4, 5]);
    }

    #[test]
    fn test_stats() {
        let mut cache = LruCache::new(1024);
        cache.insert("k".into(), vec![1]);
        cache.get("k");
        cache.get("missing");
        let (hits, misses, rate) = cache.stats();
        assert_eq!(hits, 1);
        assert_eq!(misses, 1);
        assert!((rate - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_utilization() {
        let mut cache = LruCache::new(100);
        cache.insert("k".into(), vec![0u8; 25]);
        assert!((cache.utilization() - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_empty_size() {
        let cache = LruCache::new(1024);
        assert!(cache.is_empty());
        assert_eq!(cache.size_bytes(), 0);
    }
}
