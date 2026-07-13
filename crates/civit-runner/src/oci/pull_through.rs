#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A cached pull-through entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub repo_id: String,
    pub upstream_url: String,
    pub upstream_ref: String,
    pub local_digest: String,
    pub cached_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl CacheEntry {
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }

    pub fn remaining_ttl(&self) -> Duration {
        let now = chrono::Utc::now();
        if now >= self.expires_at {
            Duration::ZERO
        } else {
            (self.expires_at - now).to_std().unwrap_or(Duration::ZERO)
        }
    }
}

/// Configuration for pull-through caching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullThroughConfig {
    /// Default TTL for cached blobs.
    pub default_ttl: Duration,
    /// Maximum cache size in bytes.
    pub max_cache_bytes: u64,
    /// Upstream registry URL.
    pub upstream_url: String,
}

impl Default for PullThroughConfig {
    fn default() -> Self {
        Self {
            default_ttl: Duration::from_secs(3600), // 1 hour
            max_cache_bytes: 10 * 1024 * 1024 * 1024, // 10 GB
            upstream_url: "https://registry-1.docker.io".to_string(),
        }
    }
}

/// Pull-through cache for container registry.
///
/// Caches blobs from upstream registries to reduce bandwidth and improve
/// pull performance for frequently accessed images.
pub struct PullThroughCache {
    config: PullThroughConfig,
    entries: dashmap::DashMap<String, CacheEntry>,
    current_bytes: std::sync::atomic::AtomicU64,
}

impl Default for PullThroughCache {
    fn default() -> Self {
        Self::new(PullThroughConfig::default())
    }
}

impl PullThroughCache {
    pub fn new(config: PullThroughConfig) -> Self {
        Self {
            config,
            entries: dashmap::DashMap::new(),
            current_bytes: std::sync::atomic::AtomicU64::new(0),
        }
    }

    fn cache_key(repo_id: &str, upstream_ref: &str) -> String {
        format!("{repo_id}:{upstream_ref}")
    }

    /// Check if a reference is cached and not expired.
    pub fn get(&self, repo_id: &str, upstream_ref: &str) -> Option<CacheEntry> {
        let key = Self::cache_key(repo_id, upstream_ref);
        let entry = self.entries.get(&key)?;
        if entry.is_expired() {
            drop(entry);
            self.evict(repo_id, upstream_ref);
            None
        } else {
            Some(entry.value().clone())
        }
    }

    /// Store a new cache entry.
    pub fn put(
        &self,
        repo_id: &str,
        upstream_url: &str,
        upstream_ref: &str,
        local_digest: &str,
    ) -> CacheEntry {
        let now = chrono::Utc::now();
        let entry = CacheEntry {
            repo_id: repo_id.to_string(),
            upstream_url: upstream_url.to_string(),
            upstream_ref: upstream_ref.to_string(),
            local_digest: local_digest.to_string(),
            cached_at: now,
            expires_at: now + chrono::Duration::from_std(self.config.default_ttl).unwrap(),
        };

        let key = Self::cache_key(repo_id, upstream_ref);
        self.entries.insert(key, entry.clone());
        entry
    }

    /// Evict a specific cache entry.
    pub fn evict(&self, repo_id: &str, upstream_ref: &str) -> bool {
        let key = Self::cache_key(repo_id, upstream_ref);
        self.entries.remove(&key).is_some()
    }

    /// Evict all expired entries.
    pub fn evict_expired(&self) -> usize {
        let mut expired = Vec::new();
        for entry in self.entries.iter() {
            if entry.value().is_expired() {
                expired.push(entry.key().clone());
            }
        }
        let count = expired.len();
        for key in expired {
            self.entries.remove(&key);
        }
        count
    }

    /// Number of active cache entries.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// List all non-expired cache entries.
    pub fn list_entries(&self) -> Vec<CacheEntry> {
        self.entries
            .iter()
            .filter(|r| !r.value().is_expired())
            .map(|r| r.value().clone())
            .collect()
    }

    /// Get cache configuration.
    pub fn config(&self) -> &PullThroughConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_is_expired() {
        let entry = CacheEntry {
            repo_id: "r".into(),
            upstream_url: "u".into(),
            upstream_ref: "v1".into(),
            local_digest: "sha256:abc".into(),
            cached_at: chrono::Utc::now() - chrono::Duration::hours(2),
            expires_at: chrono::Utc::now() - chrono::Duration::hours(1),
        };
        assert!(entry.is_expired());
    }

    #[test]
    fn test_cache_entry_not_expired() {
        let entry = CacheEntry {
            repo_id: "r".into(),
            upstream_url: "u".into(),
            upstream_ref: "v1".into(),
            local_digest: "sha256:abc".into(),
            cached_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        };
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_pull_through_put_and_get() {
        let cache = PullThroughCache::new(PullThroughConfig {
            default_ttl: Duration::from_secs(60),
            ..Default::default()
        });
        cache.put("repo1", "https://upstream.io", "v1", "sha256:abc");
        let entry = cache.get("repo1", "v1");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().local_digest, "sha256:abc");
    }

    #[test]
    fn test_pull_through_get_expired() {
        let cache = PullThroughCache::new(PullThroughConfig {
            default_ttl: Duration::from_secs(0), // immediate expiry
            ..Default::default()
        });
        cache.put("repo1", "https://upstream.io", "v1", "sha256:abc");
        // expired
        assert!(cache.get("repo1", "v1").is_none());
    }

    #[test]
    fn test_pull_through_evict() {
        let cache = PullThroughCache::default();
        cache.put("r", "u", "v1", "d");
        assert!(cache.evict("r", "v1"));
        assert!(!cache.evict("r", "v1"));
    }

    #[test]
    fn test_pull_through_evict_expired() {
        let cache = PullThroughCache::new(PullThroughConfig {
            default_ttl: Duration::from_secs(0),
            ..Default::default()
        });
        cache.put("r", "u", "v1", "d");
        cache.put("r", "u", "v2", "d");
        let evicted = cache.evict_expired();
        assert_eq!(evicted, 2);
    }

    #[test]
    fn test_pull_through_entry_count() {
        let cache = PullThroughCache::default();
        assert_eq!(cache.entry_count(), 0);
        cache.put("r", "u", "v1", "d");
        assert_eq!(cache.entry_count(), 1);
    }

    #[test]
    fn test_pull_through_list_entries() {
        let cache = PullThroughCache::default();
        cache.put("r", "u", "v1", "d");
        cache.put("r", "u", "v2", "d2");
        let entries = cache.list_entries();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_config_default() {
        let config = PullThroughConfig::default();
        assert_eq!(config.default_ttl, Duration::from_secs(3600));
        assert_eq!(config.max_cache_bytes, 10 * 1024 * 1024 * 1024);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let entry = CacheEntry {
            repo_id: "r".into(),
            upstream_url: "u".into(),
            upstream_ref: "v1".into(),
            local_digest: "d".into(),
            cached_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let de: CacheEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(de.repo_id, "r");
    }
}
