//! HTTP cache middleware for GET requests with TTL-based expiration.
//!
//! Caches GET responses for 5 minutes by default.
//! Supports cache invalidation on mutations via cache busting.
//! Provides cache statistics endpoint.

#![forbid(unsafe_code)]

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderName, HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::debug;

/// Default cache TTL (5 minutes).
const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300);

/// Cache entry metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// The cached response body.
    pub body: Vec<u8>,
    /// Response status code.
    pub status: u16,
    /// Response headers.
    pub headers: Vec<(String, String)>,
    /// When this entry was created.
    pub created_at: u64,
    /// When this entry expires.
    pub expires_at: u64,
}

/// Cache statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total cache hits.
    pub hits: u64,
    /// Total cache misses.
    pub misses: u64,
    /// Total cache invalidations.
    pub invalidations: u64,
    /// Current number of entries in cache.
    pub entries: usize,
}

/// In-memory cache store with TTL expiration.
#[derive(Debug)]
pub struct CacheStore {
    entries: RwLock<std::collections::HashMap<String, CacheEntry>>,
    stats: RwLock<CacheStats>,
    ttl: Duration,
}

impl CacheStore {
    /// Create a new cache store with default TTL.
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(std::collections::HashMap::new()),
            stats: RwLock::new(CacheStats::default()),
            ttl: DEFAULT_CACHE_TTL,
        }
    }

    /// Create a new cache store with custom TTL.
    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            entries: RwLock::new(std::collections::HashMap::new()),
            stats: RwLock::new(CacheStats::default()),
            ttl,
        }
    }

    /// Get a cached entry if it exists and hasn't expired.
    pub async fn get(&self, key: &str) -> Option<CacheEntry> {
        let entries = self.entries.read().await;
        if let Some(entry) = entries.get(key) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if entry.expires_at > now {
                debug!(key = %key, "Cache hit");
                self.stats.write().await.hits += 1;
                return Some(entry.clone());
            }
        }
        debug!(key = %key, "Cache miss");
        self.stats.write().await.misses += 1;
        None
    }

    /// Store a value in the cache.
    pub async fn set(&self, key: String, entry: CacheEntry) {
        let mut entries = self.entries.write().await;
        entries.insert(key, entry);
    }

    /// Invalidate a specific cache entry.
    pub async fn invalidate(&self, key: &str) {
        let mut entries = self.entries.write().await;
        if entries.remove(key).is_some() {
            debug!(key = %key, "Cache entry invalidated");
            self.stats.write().await.invalidations += 1;
        }
    }

    /// Invalidate all cache entries matching a prefix.
    pub async fn invalidate_prefix(&self, prefix: &str) {
        let mut entries = self.entries.write().await;
        let keys_to_remove: Vec<String> = entries
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        let count = keys_to_remove.len();
        for key in keys_to_remove {
            entries.remove(&key);
        }
        if count > 0 {
            debug!(prefix = %prefix, count = count, "Cache prefix invalidated");
            self.stats.write().await.invalidations += count as u64;
        }
    }

    /// Clear all expired entries.
    pub async fn cleanup(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut entries = self.entries.write().await;
        entries.retain(|_, entry| entry.expires_at > now);
    }

    /// Get cache statistics.
    pub async fn stats(&self) -> CacheStats {
        let mut stats = self.stats.read().await.clone();
        stats.entries = self.entries.read().await.len();
        stats
    }

    /// Get the cache TTL.
    pub fn ttl(&self) -> Duration {
        self.ttl
    }
}

impl Default for CacheStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a cache key from request method, path, and query string.
fn generate_cache_key(req: &Request) -> Option<String> {
    if req.method() != axum::http::Method::GET {
        return None;
    }
    let path = req.uri().path().to_string();
    let query = req.uri().query().map(|q| format!("?{q}")).unwrap_or_default();
    Some(format!("http:{path}{query}"))
}

/// Check if the request should be cached.
fn should_cache(req: &Request) -> bool {
    // Only cache GET requests
    if req.method() != axum::http::Method::GET {
        return false;
    }
    // Don't cache if Cache-Control: no-cache is set
    if let Some(cc) = req.headers().get(header::CACHE_CONTROL) {
        if let Ok(val) = cc.to_str() {
            if val.contains("no-cache") || val.contains("no-store") {
                return false;
            }
        }
    }
    true
}

/// Check if the response should be cached.
#[allow(dead_code)]
fn should_cache_response(req: &Request, status: StatusCode) -> bool {
    if !should_cache(req) {
        return false;
    }
    // Only cache successful responses
    status.is_success()
}

/// State passed through request extensions for the cache middleware.
#[derive(Clone)]
pub struct CacheState {
    pub store: Arc<CacheStore>,
}

/// HTTP cache middleware.
///
/// Caches GET responses and serves them on subsequent requests.
/// Invalidates cache on mutations (POST, PUT, DELETE, PATCH).
pub async fn cache_middleware(req: Request, next: Next) -> Response {
    let state = req.extensions().get::<Arc<CacheState>>().cloned();

    let cache_state = match state {
        Some(s) => s,
        None => return next.run(req).await,
    };

    let cache_key = match generate_cache_key(&req) {
        Some(k) => k,
        None => {
            // For non-GET requests, invalidate related cache entries
            let path = req.uri().path().to_string();
            let prefix = format!("http:{path}");
            cache_state.store.invalidate_prefix(&prefix).await;
            return next.run(req).await;
        }
    };

    // Try to serve from cache
    if should_cache(&req) {
        if let Some(entry) = cache_state.store.get(&cache_key).await {
            debug!(key = %cache_key, "Serving cached response");
            let mut response = Response::builder()
                .status(entry.status)
                .body(Body::from(entry.body))
                .unwrap();
            let headers = response.headers_mut();
            for (name, value) in &entry.headers {
                if let (Ok(n), Ok(v)) = (
                    HeaderName::from_bytes(name.as_bytes()),
                    HeaderValue::from_str(value),
                ) {
                    headers.insert(n, v);
                }
            }
            headers.insert(
                HeaderName::from_static("x-cache"),
                HeaderValue::from_static("HIT"),
            );
            return response;
        }
    }

    let start = Instant::now();
    let response = next.run(req).await;
    let duration = start.elapsed();

    // Cache successful GET responses
    // Note: We need to check if we should cache before consuming the response
    let should_cache = response.status().is_success();
    let status = response.status().as_u16();
    let headers = response.headers().clone();

    if should_cache {
        if let Ok(body) = axum::body::to_bytes(response.into_body(), usize::MAX).await {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let entry = CacheEntry {
                body: body.to_vec(),
                status,
                headers: headers
                    .iter()
                    .filter_map(|(k, v)| {
                        v.to_str().ok().map(|val| (k.to_string(), val.to_string()))
                    })
                    .collect(),
                created_at: now,
                expires_at: now + cache_state.store.ttl().as_secs(),
            };

            cache_state.store.set(cache_key.clone(), entry).await;
            debug!(
                key = %cache_key,
                duration_ms = duration.as_secs_f64() * 1000.0,
                "Cached response"
            );

            let mut response = Response::builder()
                .status(status)
                .body(Body::from(body))
                .unwrap();
            let resp_headers = response.headers_mut();
            resp_headers.insert(
                HeaderName::from_static("x-cache"),
                HeaderValue::from_static("MISS"),
            );
            return response;
        }
    }

    // Return original response if caching failed
    let mut response = Response::builder()
        .status(status)
        .body(Body::empty())
        .unwrap();
    *response.headers_mut() = headers;
    response
}

/// Cache statistics endpoint handler.
pub async fn cache_stats_handler(
    axum::extract::State(state): axum::extract::State<Arc<CacheState>>,
) -> impl IntoResponse {
    let stats = state.store.stats().await;
    axum::Json(stats)
}

/// Cache invalidation endpoint handler.
pub async fn cache_invalidate_handler(
    axum::extract::State(state): axum::extract::State<Arc<CacheState>>,
    axum::extract::Path(key): axum::extract::Path<String>,
) -> impl IntoResponse {
    state.store.invalidate(&key).await;
    StatusCode::OK
}

/// Cache clear endpoint handler.
pub async fn cache_clear_handler(
    axum::extract::State(state): axum::extract::State<Arc<CacheState>>,
) -> impl IntoResponse {
    state.store.cleanup().await;
    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Method;

    #[test]
    fn test_cache_store_new() {
        let store = CacheStore::new();
        assert_eq!(store.ttl(), DEFAULT_CACHE_TTL);
    }

    #[test]
    fn test_cache_store_with_ttl() {
        let store = CacheStore::with_ttl(Duration::from_secs(60));
        assert_eq!(store.ttl(), Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_cache_store_set_and_get() {
        let store = CacheStore::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let entry = CacheEntry {
            body: b"test".to_vec(),
            status: 200,
            headers: vec![],
            created_at: now,
            expires_at: now + 300,
        };
        store.set("key1".into(), entry.clone()).await;
        let retrieved = store.get("key1").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().body, b"test");
    }

    #[tokio::test]
    async fn test_cache_store_expired() {
        let store = CacheStore::new();
        let entry = CacheEntry {
            body: b"test".to_vec(),
            status: 200,
            headers: vec![],
            created_at: 0,
            expires_at: 0, // Already expired
        };
        store.set("key1".into(), entry).await;
        let retrieved = store.get("key1").await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_cache_store_invalidate() {
        let store = CacheStore::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let entry = CacheEntry {
            body: b"test".to_vec(),
            status: 200,
            headers: vec![],
            created_at: now,
            expires_at: now + 300,
        };
        store.set("key1".into(), entry).await;
        store.invalidate("key1").await;
        assert!(store.get("key1").await.is_none());
    }

    #[tokio::test]
    async fn test_cache_store_stats() {
        let store = CacheStore::new();
        let stats = store.stats().await;
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_generate_cache_key_get() {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/repos?page=1")
            .body(Body::empty())
            .unwrap();
        let key = generate_cache_key(&req);
        assert_eq!(key, Some("http:/api/repos?page=1".into()));
    }

    #[test]
    fn test_generate_cache_key_post() {
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/repos")
            .body(Body::empty())
            .unwrap();
        let key = generate_cache_key(&req);
        assert!(key.is_none());
    }

    #[test]
    fn test_should_cache_get() {
        let req = Request::builder()
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();
        assert!(should_cache(&req));
    }

    #[test]
    fn test_should_cache_no_cache_header() {
        let req = Request::builder()
            .method(Method::GET)
            .header("Cache-Control", "no-cache")
            .body(Body::empty())
            .unwrap();
        assert!(!should_cache(&req));
    }
}
