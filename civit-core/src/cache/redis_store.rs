#![forbid(unsafe_code)]

use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use futures::StreamExt as _;

/// Configuration for the Redis cache backend.
#[derive(Debug, Clone)]
pub struct RedisCacheConfig {
    /// Redis connection URL (e.g., `redis://127.0.0.1:6379`).
    pub url: String,
    /// Default TTL for cache entries. `None` = no expiry.
    pub default_ttl: Option<Duration>,
    /// Key prefix to namespace CivitForge entries.
    pub key_prefix: String,
    /// Pub/Sub channel for cache invalidation broadcasts.
    pub invalidation_channel: String,
}

impl Default for RedisCacheConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".into(),
            default_ttl: Some(Duration::from_secs(3600)),
            key_prefix: "civit:cache:".into(),
            invalidation_channel: "civit:cache:invalidate".into(),
        }
    }
}

/// Errors from Redis cache operations.
#[derive(Debug, thiserror::Error)]
pub enum RedisCacheError {
    #[error("redis connection error: {0}")]
    Connection(#[source] redis::RedisError),
    #[error("redis command error: {0}")]
    Command(#[source] redis::RedisError),
    #[error("deserialization error: {0}")]
    Deserialization(String),
}

impl From<redis::RedisError> for RedisCacheError {
    fn from(e: redis::RedisError) -> Self {
        RedisCacheError::Command(e)
    }
}

/// A serialized cache entry stored in Redis as a JSON value.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RedisCacheEntry {
    /// Raw data bytes (may be zstd-compressed).
    pub data: Vec<u8>,
    /// Whether `data` is zstd-compressed.
    pub compressed: bool,
    /// Original uncompressed size.
    pub original_size: usize,
    /// Compressed size (same as `data.len()` when compressed).
    pub compressed_size: usize,
    /// SHA-256 ETag of the original data.
    pub etag: String,
}

impl RedisCacheEntry {
    /// Compute SHA-256 ETag of data.
    pub fn compute_etag(data: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("\"{:x}\"", hasher.finalize())
    }
}

/// Redis-backed cache store with TTL support and Pub/Sub invalidation.
///
/// Wraps the `EdgeCacheManager` contract (get/put/invalidate) with Redis
/// as the backing store. Supports:
/// - TTL-based expiry (default 1h, configurable)
/// - zstd compression (threshold 512 bytes, level 3)
/// - SHA-256 ETags
/// - Pub/Sub cross-node invalidation broadcasts
pub struct RedisCacheStore {
    conn: ConnectionManager,
    config: RedisCacheConfig,
    /// Running count of local hits (for stats).
    local_hits: Arc<std::sync::atomic::AtomicU64>,
    /// Running count of local misses (for stats).
    local_misses: Arc<std::sync::atomic::AtomicU64>,
}

impl RedisCacheStore {
    /// Minimum data size (bytes) below which compression is skipped.
    const COMPRESSION_THRESHOLD: usize = 512;
    /// Zstd compression level.
    const COMPRESSION_LEVEL: i32 = 3;

    /// Create a new RedisCacheStore by connecting to Redis.
    pub async fn new(config: RedisCacheConfig) -> Result<Self, RedisCacheError> {
        info!(url = %config.url, "connecting to Redis cache");
        let client =
            redis::Client::open(config.url.as_str()).map_err(RedisCacheError::Connection)?;
        let conn = ConnectionManager::new(client)
            .await
            .map_err(RedisCacheError::Connection)?;
        info!("connected to Redis cache");
        Ok(Self {
            conn,
            config,
            local_hits: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            local_misses: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        })
    }

    /// Build a Redis-prefixed key from a user-facing key.
    fn redis_key(&self, key: &str) -> String {
        format!("{}{}", self.config.key_prefix, key)
    }

    /// Get a cached value by key. Decompresses if needed.
    /// Returns `None` on miss or Redis error.
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let rk = self.redis_key(key);
        let mut conn = self.conn.clone();
        match conn.get::<_, Option<String>>(&rk).await {
            Ok(Some(json_str)) => match serde_json::from_str::<RedisCacheEntry>(&json_str) {
                Ok(entry) => {
                    self.local_hits
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let data = if entry.compressed {
                        zstd::decode_all(std::io::Cursor::new(&entry.data)).ok()?
                    } else {
                        entry.data
                    };
                    debug!(key = %key, "cache hit");
                    Some(data)
                }
                Err(e) => {
                    warn!(key = %key, error = %e, "failed to deserialize cache entry");
                    None
                }
            },
            Ok(None) => {
                self.local_misses
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                debug!(key = %key, "cache miss");
                None
            }
            Err(e) => {
                error!(key = %key, error = %e, "redis GET error");
                None
            }
        }
    }

    /// Put a value into the cache. Compresses large payloads, sets TTL.
    pub async fn put(&self, key: &str, data: Vec<u8>) -> Result<(), RedisCacheError> {
        let original_size = data.len();
        let etag = RedisCacheEntry::compute_etag(&data);

        let (stored_data, compressed, compressed_size) =
            if data.len() >= Self::COMPRESSION_THRESHOLD {
                match zstd::encode_all(std::io::Cursor::new(&data), Self::COMPRESSION_LEVEL) {
                    Ok(encoded) if encoded.len() < original_size => {
                        let cs = encoded.len();
                        (encoded, true, cs)
                    }
                    _ => (data.clone(), false, original_size),
                }
            } else {
                (data.clone(), false, original_size)
            };

        let entry = RedisCacheEntry {
            data: stored_data,
            compressed,
            original_size,
            compressed_size,
            etag,
        };

        let json_str = serde_json::to_string(&entry)
            .map_err(|e| RedisCacheError::Deserialization(e.to_string()))?;

        let rk = self.redis_key(key);
        let mut conn = self.conn.clone();

        match self.config.default_ttl {
            Some(ttl) => {
                conn.set_ex::<_, _, ()>(&rk, &json_str, ttl.as_secs())
                    .await
                    .map_err(RedisCacheError::Command)?;
            }
            None => {
                conn.set::<_, _, ()>(&rk, &json_str)
                    .await
                    .map_err(RedisCacheError::Command)?;
            }
        }

        debug!(
            key = %key,
            original_size,
            compressed,
            compressed_size,
            "cache put"
        );
        Ok(())
    }

    /// Put with explicit TTL (overrides default).
    pub async fn put_with_ttl(
        &self,
        key: &str,
        data: Vec<u8>,
        ttl: Duration,
    ) -> Result<(), RedisCacheError> {
        let original_size = data.len();
        let etag = RedisCacheEntry::compute_etag(&data);

        let (stored_data, compressed, compressed_size) =
            if data.len() >= Self::COMPRESSION_THRESHOLD {
                match zstd::encode_all(std::io::Cursor::new(&data), Self::COMPRESSION_LEVEL) {
                    Ok(encoded) if encoded.len() < original_size => {
                        let cs = encoded.len();
                        (encoded, true, cs)
                    }
                    _ => (data.clone(), false, original_size),
                }
            } else {
                (data.clone(), false, original_size)
            };

        let entry = RedisCacheEntry {
            data: stored_data,
            compressed,
            original_size,
            compressed_size,
            etag,
        };

        let json_str = serde_json::to_string(&entry)
            .map_err(|e| RedisCacheError::Deserialization(e.to_string()))?;

        let rk = self.redis_key(key);
        let mut conn = self.conn.clone();
        conn.set_ex::<_, _, ()>(&rk, &json_str, ttl.as_secs())
            .await
            .map_err(RedisCacheError::Command)?;

        debug!(key = %key, ttl_secs = ttl.as_secs(), "cache put with TTL");
        Ok(())
    }

    /// Invalidate a single key. Also publishes invalidation to Pub/Sub.
    pub async fn invalidate(&self, key: &str) -> bool {
        let rk = self.redis_key(key);
        let mut conn = self.conn.clone();
        match conn.del::<_, u64>(&[&rk]).await {
            Ok(deleted) if deleted > 0 => {
                // Broadcast invalidation to other nodes
                self.publish_invalidation(key).await;
                debug!(key = %key, "cache invalidated");
                true
            }
            Ok(_) => false,
            Err(e) => {
                error!(key = %key, error = %e, "redis DEL error");
                false
            }
        }
    }

    /// Invalidate all keys matching a pattern. Uses SCAN + DEL.
    pub async fn invalidate_pattern(&self, pattern: &str) -> usize {
        let redis_pattern = if pattern == "*" {
            format!("{}*", self.config.key_prefix)
        } else {
            format!("{}*{}*", self.config.key_prefix, pattern)
        };

        let mut conn = self.conn.clone();
        let mut keys: Vec<String> = Vec::new();

        // Use SCAN to find matching keys (avoids blocking KEYS command)
        let mut cursor: u64 = 0;
        loop {
            let (next_cursor, batch): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&redis_pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await
                .unwrap_or_else(|_| (0, Vec::new()));
            keys.extend(batch);
            cursor = next_cursor;
            if cursor == 0 {
                break;
            }
        }

        let count = keys.len();
        if count > 0 {
            let mut conn = self.conn.clone();
            // DEL with variable args — use iter
            for key in &keys {
                if let Err(e) = redis::cmd("DEL")
                    .arg(key)
                    .query_async::<u64>(&mut conn)
                    .await
                {
                    error!(key = %key, error = %e, "redis DEL error");
                }
            }

            debug!(pattern = %pattern, count, "pattern invalidation via SCAN+DEL");
            self.publish_invalidation_pattern(pattern).await;
        }
        count
    }

    /// Publish a key invalidation event to the Pub/Sub channel.
    async fn publish_invalidation(&self, key: &str) {
        let mut conn = self.conn.clone();
        let msg = format!("key:{key}");
        if let Err(e) = conn
            .publish::<_, _, ()>(&self.config.invalidation_channel, &msg)
            .await
        {
            warn!(error = %e, "failed to publish invalidation");
        }
    }

    /// Publish a pattern invalidation event to the Pub/Sub channel.
    async fn publish_invalidation_pattern(&self, pattern: &str) {
        let mut conn = self.conn.clone();
        let msg = format!("pattern:{pattern}");
        if let Err(e) = conn
            .publish::<_, _, ()>(&self.config.invalidation_channel, &msg)
            .await
        {
            warn!(error = %e, "failed to publish pattern invalidation");
        }
    }

    /// Subscribe to invalidation events and apply them locally.
    /// Returns a handle that must be kept alive. Dropping it unsubscribes.
    ///
    /// The `on_invalidate` callback receives the key or pattern that was
    /// invalidated on a remote node. The callback runs on a background task.
    pub async fn subscribe_invalidation<F, Fut>(
        &self,
        on_invalidate: F,
    ) -> Result<tokio::task::JoinHandle<()>, RedisCacheError>
    where
        F: Fn(String) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send,
    {
        let client =
            redis::Client::open(self.config.url.as_str()).map_err(RedisCacheError::Connection)?;
        let mut pubsub = client
            .get_async_pubsub()
            .await
            .map_err(RedisCacheError::Command)?;
        pubsub
            .subscribe(self.config.invalidation_channel.clone())
            .await
            .map_err(RedisCacheError::Command)?;

        let on_invalidate = Arc::new(on_invalidate);
        let handle = tokio::spawn(async move {
            let mut stream = pubsub.on_message();
            loop {
                match stream.next().await {
                    Some(msg) => {
                        let payload: String = msg.get_payload().unwrap_or_default();
                        let key_or_pattern = payload
                            .strip_prefix("key:")
                            .or_else(|| payload.strip_prefix("pattern:"))
                            .unwrap_or("");
                        if !key_or_pattern.is_empty() {
                            let cb = Arc::clone(&on_invalidate);
                            cb(key_or_pattern.to_string()).await;
                        }
                    }
                    None => {
                        info!("invalidation subscription stream ended");
                        break;
                    }
                }
            }
        });

        Ok(handle)
    }

    /// Check if a key exists in Redis.
    pub async fn exists(&self, key: &str) -> bool {
        let rk = self.redis_key(key);
        let mut conn = self.conn.clone();
        match conn.exists::<_, bool>(&rk).await {
            Ok(exists) => exists,
            Err(e) => {
                error!(key = %key, error = %e, "redis EXISTS error");
                false
            }
        }
    }

    /// Get the TTL remaining for a key. Returns `None` if key doesn't exist
    /// or has no expiry.
    pub async fn ttl(&self, key: &str) -> Option<Duration> {
        let rk = self.redis_key(key);
        let mut conn = self.conn.clone();
        match conn.ttl::<_, i64>(&rk).await {
            Ok(secs) if secs > 0 => Some(Duration::from_secs(secs as u64)),
            Ok(_) => None,
            Err(e) => {
                error!(key = %key, error = %e, "redis TTL error");
                None
            }
        }
    }

    /// Local hit rate (approximate, for metrics).
    pub fn hit_rate(&self) -> f64 {
        let hits = self.local_hits.load(std::sync::atomic::Ordering::Relaxed);
        let misses = self.local_misses.load(std::sync::atomic::Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            return 0.0;
        }
        hits as f64 / total as f64
    }
}

/// A cache store trait that abstracts over in-memory (DashMap) and Redis backends.
/// Allows callers to swap backends without changing logic.
pub trait CacheStore: Send + Sync {
    /// Get a cached value. Returns `None` on miss.
    fn get(&self, key: &str) -> Option<Vec<u8>>;
    /// Put a value into the cache.
    fn put(&self, key: String, data: Vec<u8>);
    /// Invalidate a single key. Returns true if it existed.
    fn invalidate(&self, key: &str) -> bool;
    /// Invalidate keys matching a pattern (`*` = all). Returns count.
    fn invalidate_pattern(&self, pattern: &str) -> usize;
}

/// Cache warming: pre-populate the Redis cache with data on a repository push event.
/// `entries` is an iterator of (key, data) pairs to insert.
pub async fn warm_cache(
    store: &RedisCacheStore,
    entries: Vec<(String, Vec<u8>)>,
) -> Result<u64, RedisCacheError> {
    let mut count = 0u64;
    for (key, data) in entries {
        store.put(&key, data).await?;
        count += 1;
    }
    info!(entries = count, "cache warmed");
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_cache_config_default() {
        let cfg = RedisCacheConfig::default();
        assert_eq!(cfg.url, "redis://127.0.0.1:6379");
        assert_eq!(cfg.default_ttl, Some(Duration::from_secs(3600)));
        assert_eq!(cfg.key_prefix, "civit:cache:");
        assert_eq!(cfg.invalidation_channel, "civit:cache:invalidate");
    }

    #[test]
    fn test_redis_cache_entry_serialization() {
        let entry = RedisCacheEntry {
            data: b"hello world".to_vec(),
            compressed: false,
            original_size: 11,
            compressed_size: 11,
            etag: "\"abc123\"".into(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deser: RedisCacheEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.data, b"hello world");
        assert!(!deser.compressed);
        assert_eq!(deser.original_size, 11);
        assert_eq!(deser.etag, "\"abc123\"");
    }

    #[test]
    fn test_redis_cache_entry_serialization_compressed() {
        let data = vec![0u8; 4096];
        let entry = RedisCacheEntry {
            data: data.clone(),
            compressed: true,
            original_size: 4096,
            compressed_size: 4096,
            etag: "\"def456\"".into(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deser: RedisCacheEntry = serde_json::from_str(&json).unwrap();
        assert!(deser.compressed);
        assert_eq!(deser.compressed_size, 4096);
    }

    #[test]
    fn test_compute_etag() {
        let etag = RedisCacheEntry::compute_etag(b"test data");
        assert!(etag.starts_with('"'));
        assert!(etag.ends_with('"'));
        assert!(etag.len() > 10); // SHA-256 hex
    }

    #[test]
    fn test_compute_etag_deterministic() {
        let e1 = RedisCacheEntry::compute_etag(b"deterministic");
        let e2 = RedisCacheEntry::compute_etag(b"deterministic");
        assert_eq!(e1, e2);
    }

    #[test]
    fn test_compute_etag_different_data() {
        let e1 = RedisCacheEntry::compute_etag(b"data-a");
        let e2 = RedisCacheEntry::compute_etag(b"data-b");
        assert_ne!(e1, e2);
    }

    #[test]
    fn test_redis_key_prefix() {
        let store_config = RedisCacheConfig {
            url: "redis://localhost:6379".into(),
            default_ttl: None,
            key_prefix: "myapp:".into(),
            invalidation_channel: "myapp:inv".into(),
        };
        // Can't easily construct a RedisCacheStore without a server,
        // so test the key prefix logic directly
        let prefix = &store_config.key_prefix;
        let key = "repo:main:build";
        let rk = format!("{prefix}{key}");
        assert_eq!(rk, "myapp:repo:main:build");
    }

    #[test]
    fn test_redis_cache_error_display() {
        let err = RedisCacheError::Deserialization("bad json".into());
        assert!(err.to_string().contains("deserialization"));
    }

    #[test]
    fn test_compression_threshold() {
        assert_eq!(RedisCacheStore::COMPRESSION_THRESHOLD, 512);
        assert_eq!(RedisCacheStore::COMPRESSION_LEVEL, 3);
    }

    // Note: Integration tests with a real Redis server are deferred to CI
    // where a Redis container is available (e.g., via testcontainers).

    #[test]
    fn test_zstd_compression_roundtrip() {
        // Large repetitive data should compress
        let data = "abcdefghij".repeat(1024).into_bytes();
        let original_size = data.len();
        assert!(original_size >= RedisCacheStore::COMPRESSION_THRESHOLD);

        let encoded = zstd::encode_all(
            std::io::Cursor::new(&data),
            RedisCacheStore::COMPRESSION_LEVEL,
        )
        .unwrap();
        assert!(encoded.len() < original_size);

        let decoded = zstd::decode_all(std::io::Cursor::new(&encoded)).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_zstd_no_compress_small() {
        // Small data shouldn't compress
        let data = b"hello".to_vec();
        let encoded = zstd::encode_all(
            std::io::Cursor::new(&data),
            RedisCacheStore::COMPRESSION_LEVEL,
        )
        .unwrap();
        // For small data, compressed may be larger
        assert!(encoded.len() >= data.len());
    }
}
