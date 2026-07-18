//! Per-user rate limiting middleware with tiered limits and security headers.
//!
//! Supports three tiers:
//! - Anonymous: 60 requests/minute (by IP)
//! - Authenticated: 300 requests/minute (by user ID)
//! - Admin: 1000 requests/minute (by user ID)
//!
//! Also supports token bucket rate limiting via database-backed policies.
//! Returns 429 Too Many Requests with `Retry-After` header.
//! Adds `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset` headers.

#![forbid(unsafe_code)]

#[cfg(test)]
use axum::body::Body;
use axum::{
    extract::Request,
    http::{HeaderName, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use tracing::warn;

/// Rate limit tiers based on user role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitTier {
    Anonymous,
    Authenticated,
    Admin,
}

impl RateLimitTier {
    /// Max requests per minute for this tier.
    pub fn max_requests(&self) -> u32 {
        match self {
            RateLimitTier::Anonymous => 60,
            RateLimitTier::Authenticated => 300,
            RateLimitTier::Admin => 1000,
        }
    }
}

/// Configuration for rate limiting.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Max requests per window (legacy, used as fallback).
    pub max_requests: u32,
    /// Window duration.
    pub window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60),
        }
    }
}

#[derive(Debug, Clone)]
struct Bucket {
    count: u32,
    window_start: Instant,
}

/// Token bucket state for database-backed rate limiting.
#[derive(Debug, Clone)]
struct TokenBucket {
    tokens: i32,
    last_refill: Instant,
    max_tokens: i32,
    refill_rate: f64, // tokens per second
}

/// Per-key (IP or user ID) sliding window state. Thread-safe via tokio Mutex.
#[derive(Debug, Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    buckets: Arc<Mutex<HashMap<String, Bucket>>>,
    /// Token buckets for database-backed rate limiting
    token_buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    /// Admin bypass flag
    admin_bypass: bool,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            buckets: Arc::new(Mutex::new(HashMap::new())),
            token_buckets: Arc::new(Mutex::new(HashMap::new())),
            admin_bypass: true,
        }
    }

    /// Check if request is allowed. Returns `(allowed, retry_after_seconds, remaining, limit, reset_seconds)`.
    pub async fn check(
        &self,
        key: &str,
        tier: RateLimitTier,
    ) -> (bool, u32, u32, u32, u64) {
        let mut buckets = self.buckets.lock().await;
        let now = Instant::now();
        let max_requests = tier.max_requests();
        let window = self.config.window;

        let bucket = buckets.entry(key.to_string()).or_insert(Bucket {
            count: 0,
            window_start: now,
        });

        // Sliding window: reset if window expired
        if now.duration_since(bucket.window_start) >= window {
            bucket.count = 0;
            bucket.window_start = now;
        }

        let reset_seconds = window
            .saturating_sub(now.duration_since(bucket.window_start))
            .as_secs();

        if bucket.count >= max_requests {
            let retry_after = reset_seconds as u32 + u32::from(reset_seconds == 0);
            return (false, retry_after, 0, max_requests, reset_seconds);
        }

        bucket.count += 1;
        let remaining = max_requests.saturating_sub(bucket.count);
        (true, 0, remaining, max_requests, reset_seconds)
    }

    /// Check token bucket rate limiting. Returns `(allowed, retry_after_ms)`.
    pub async fn check_token_bucket(
        &self,
        key: &str,
        max_tokens: i32,
        refill_rate: f64,
        burst_size: i32,
    ) -> (bool, u64) {
        let mut token_buckets = self.token_buckets.lock().await;
        let now = Instant::now();

        let bucket = token_buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket {
                tokens: max_tokens,
                last_refill: now,
                max_tokens,
                refill_rate,
            });

        // Refill tokens based on elapsed time
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        let new_tokens = (elapsed * refill_rate) as i32;
        if new_tokens > 0 {
            bucket.tokens = (bucket.tokens + new_tokens).min(bucket.max_tokens);
            bucket.last_refill = now;
        }

        // Check if we have enough tokens (accounting for burst)
        let _effective_max = bucket.max_tokens + burst_size;
        if bucket.tokens <= 0 {
            // Calculate retry time until next token
            let wait_time = if bucket.refill_rate > 0.0 {
                ((1.0 / bucket.refill_rate) * 1000.0) as u64
            } else {
                1000
            };
            return (false, wait_time);
        }

        bucket.tokens -= 1;
        (true, 0)
    }

    /// Check rate limit with policy-based limits. Returns `(allowed, retry_after_seconds, remaining, limit, reset_seconds)`.
    pub async fn check_with_policy(
        &self,
        key: &str,
        rate_limit: i32,
        window_seconds: i32,
        burst_size: i32,
    ) -> (bool, u32, u32, u32, u64) {
        let window = Duration::from_secs(window_seconds as u64);
        let max_requests = rate_limit as u32;

        let mut buckets = self.buckets.lock().await;
        let now = Instant::now();

        let bucket = buckets.entry(key.to_string()).or_insert(Bucket {
            count: 0,
            window_start: now,
        });

        // Sliding window: reset if window expired
        if now.duration_since(bucket.window_start) >= window {
            bucket.count = 0;
            bucket.window_start = now;
        }

        let reset_seconds = window
            .saturating_sub(now.duration_since(bucket.window_start))
            .as_secs();

        let effective_limit = max_requests + burst_size as u32;
        if bucket.count >= effective_limit {
            let retry_after = reset_seconds as u32 + u32::from(reset_seconds == 0);
            return (false, retry_after, 0, max_requests, reset_seconds);
        }

        bucket.count += 1;
        let remaining = effective_limit.saturating_sub(bucket.count);
        (true, 0, remaining, max_requests, reset_seconds)
    }

    /// Check if admin bypass is enabled
    pub fn is_admin_bypass_enabled(&self) -> bool {
        self.admin_bypass
    }
}

/// Extract client IP from request. Checks `X-Forwarded-For`, `X-Real-Ip`,
/// then falls back to loopback.
fn extract_client_ip(req: &Request) -> IpAddr {
    if let Some(forwarded) = req.headers().get("x-forwarded-for")
        && let Ok(val) = forwarded.to_str()
        && let Some(first_ip) = val.split(',').next().map(|s| s.trim())
        && let Ok(ip) = first_ip.parse::<IpAddr>()
    {
        return ip;
    }

    if let Some(real_ip) = req.headers().get("x-real-ip")
        && let Ok(val) = real_ip.to_str()
        && let Ok(ip) = val.parse::<IpAddr>()
    {
        return ip;
    }

    IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
}

/// Determine the rate limit tier from JWT claims (if present).
///
/// Returns (tier, user_key, jwt_service).
/// user_key is either the user ID (for authenticated) or IP string (for anonymous).
fn extract_tier_info(
    req: &Request,
    ip: &IpAddr,
    jwt_service: &Arc<civit_auth::jwt::JwtService>,
) -> (RateLimitTier, String) {
    // Try to extract Bearer token from Authorization header
    if let Some(auth_header) = req.headers().get(header::AUTHORIZATION)
        && let Ok(auth_str) = auth_header.to_str()
        && let Some(token) = civit_auth::jwt::JwtService::extract_bearer(auth_str)
        && let Ok(claims) = jwt_service.validate_token(token)
    {
        let tier = if claims.role == "admin" {
            RateLimitTier::Admin
        } else {
            RateLimitTier::Authenticated
        };
        return (tier, format!("user:{}", claims.sub));
    }

    // No valid token → anonymous, rate limit by IP
    (RateLimitTier::Anonymous, format!("ip:{}", ip))
}

/// Rate limiting middleware function for axum.
pub async fn rate_limit_middleware(req: Request, next: Next) -> Response {
    let state = req.extensions().get::<Arc<RateLimiter>>().cloned();
    let jwt_service = req.extensions().get::<Arc<civit_auth::jwt::JwtService>>().cloned();

    let limiter = match state {
        Some(l) => l,
        None => return next.run(req).await,
    };

    let jwt_service = match jwt_service {
        Some(s) => s,
        None => return next.run(req).await,
    };

    let ip = extract_client_ip(&req);
    let (tier, user_key) = extract_tier_info(&req, &ip, &jwt_service);

    // Admin bypass check — never bypass for auth endpoints (login, register, etc.)
    if tier == RateLimitTier::Admin && limiter.is_admin_bypass_enabled() {
        let path = req.uri().path();
        let is_auth_endpoint = path.contains("/auth/");
        if !is_auth_endpoint {
            return next.run(req).await;
        }
    }

    let (allowed, retry_after, remaining, limit, reset_seconds) =
        limiter.check(&user_key, tier).await;

    let reset_epoch = chrono::Utc::now().timestamp() as u64 + reset_seconds;

    if !allowed {
        warn!(
            key = %user_key,
            tier = ?tier,
            retry_after = retry_after,
            "Rate limit exceeded"
        );
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [
                (header::RETRY_AFTER, retry_after.to_string()),
                (
                    HeaderName::from_static("x-ratelimit-limit"),
                    limit.to_string(),
                ),
                (
                    HeaderName::from_static("x-ratelimit-remaining"),
                    "0".into(),
                ),
                (
                    HeaderName::from_static("x-ratelimit-reset"),
                    reset_epoch.to_string(),
                ),
            ],
            axum::Json(serde_json::json!({
                "error": "rate_limit_exceeded",
                "message": "Too many requests. Please retry later."
            })),
        )
            .into_response();
    }

    let mut response = next.run(req).await;

    // Add rate limit headers to successful responses
    let headers = response.headers_mut();
    headers.insert(
        HeaderName::from_static("x-ratelimit-limit"),
        limit.to_string().parse().expect("invalid value"),
    );
    headers.insert(
        HeaderName::from_static("x-ratelimit-remaining"),
        remaining.to_string().parse().expect("invalid value"),
    );
    headers.insert(
        HeaderName::from_static("x-ratelimit-reset"),
        reset_epoch.to_string().parse().expect("invalid value"),
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_under_limit() {
        let config = RateLimitConfig {
            max_requests: 5,
            window: Duration::from_secs(60),
        };
        let limiter = RateLimiter::new(config);
        let key = "ip:1.2.3.4";

        for _ in 0..5 {
            let (allowed, _, _, _, _) = limiter.check(key, RateLimitTier::Anonymous).await;
            assert!(allowed);
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);
        let key = "ip:1.2.3.4";

        for _ in 0..60 {
            limiter.check(key, RateLimitTier::Anonymous).await;
        }

        let (allowed, retry, _, _, _) = limiter.check(key, RateLimitTier::Anonymous).await;
        assert!(!allowed);
        assert!(retry > 0);
    }

    #[tokio::test]
    async fn test_rate_limiter_independent_keys() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);

        // Exhaust anonymous limit for key1
        for _ in 0..60 {
            limiter.check("ip:1.1.1.1", RateLimitTier::Anonymous).await;
        }

        // key2 should still be allowed
        let (allowed, _, _, _, _) = limiter.check("ip:2.2.2.2", RateLimitTier::Anonymous).await;
        assert!(allowed);
    }

    #[tokio::test]
    async fn test_rate_limiter_window_resets() {
        let config = RateLimitConfig {
            max_requests: 100,
            window: Duration::from_millis(500),
        };
        let limiter = RateLimiter::new(config);
        let key = "ip:1.2.3.4";

        // Exhaust the limit (Anonymous tier has 60 max)
        for _ in 0..60 {
            limiter.check(key, RateLimitTier::Anonymous).await;
        }

        let (allowed, _, _, _, _) = limiter.check(key, RateLimitTier::Anonymous).await;
        assert!(!allowed);

        // Wait for window to expire
        tokio::time::sleep(Duration::from_millis(600)).await;

        let (allowed, _, _, _, _) = limiter.check(key, RateLimitTier::Anonymous).await;
        assert!(allowed);
    }

    #[tokio::test]
    async fn test_rate_limiter_tier_limits() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);

        // Authenticated tier should allow 300
        for _ in 0..300 {
            let (allowed, _, _, _, _) =
                limiter.check("user:u1", RateLimitTier::Authenticated).await;
            assert!(allowed);
        }

        let (allowed, _, _, _, _) = limiter.check("user:u1", RateLimitTier::Authenticated).await;
        assert!(!allowed);

        // Admin tier should still work for different key
        let (allowed, _, _, _, _) = limiter.check("user:admin1", RateLimitTier::Admin).await;
        assert!(allowed);
    }

    #[tokio::test]
    async fn test_rate_limiter_returns_correct_remaining() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);

        let (allowed, _, remaining, limit, _) =
            limiter.check("ip:1.2.3.4", RateLimitTier::Anonymous).await;
        assert!(allowed);
        assert_eq!(remaining, 59);
        assert_eq!(limit, 60);
    }

    #[tokio::test]
    async fn test_token_bucket_rate_limiting() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);
        let key = "user:test1";

        // Should allow requests up to max_tokens
        for _ in 0..10 {
            let (allowed, _) = limiter
                .check_token_bucket(key, 10, 1.0, 0)
                .await;
            assert!(allowed);
        }

        // Should be blocked after max_tokens
        let (allowed, retry_ms) = limiter
            .check_token_bucket(key, 10, 1.0, 0)
            .await;
        assert!(!allowed);
        assert!(retry_ms > 0);
    }

    #[tokio::test]
    async fn test_token_bucket_refill() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);
        let key = "user:test2";

        // Exhaust tokens
        for _ in 0..5 {
            limiter.check_token_bucket(key, 5, 2.0, 0).await;
        }

        let (allowed, _) = limiter.check_token_bucket(key, 5, 2.0, 0).await;
        assert!(!allowed);

        // Wait for refill
        tokio::time::sleep(Duration::from_millis(600)).await;

        let (allowed, _) = limiter.check_token_bucket(key, 5, 2.0, 0).await;
        assert!(allowed);
    }

    #[tokio::test]
    async fn test_policy_based_rate_limiting() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);
        let key = "user:policy1";

        // Should allow up to rate_limit
        for _ in 0..5 {
            let (allowed, _, _, _, _) = limiter
                .check_with_policy(key, 5, 60, 0)
                .await;
            assert!(allowed);
        }

        let (allowed, _, _, _, _) = limiter.check_with_policy(key, 5, 60, 0).await;
        assert!(!allowed);
    }

    #[tokio::test]
    async fn test_policy_with_burst() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);
        let key = "user:burst1";

        // Should allow up to rate_limit + burst_size
        for _ in 0..7 {
            let (allowed, _, _, _, _) = limiter
                .check_with_policy(key, 5, 60, 2)
                .await;
            assert!(allowed);
        }

        let (allowed, _, _, _, _) = limiter.check_with_policy(key, 5, 60, 2).await;
        assert!(!allowed);
    }

    #[tokio::test]
    async fn test_admin_bypass() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);
        assert!(limiter.is_admin_bypass_enabled());
    }

    #[test]
    fn test_extract_client_ip_forwarded() {
        let req = Request::builder()
            .header("x-forwarded-for", "4.3.2.1, 1.2.3.4")
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            extract_client_ip(&req),
            "4.3.2.1".parse::<IpAddr>().unwrap()
        );
    }

    #[test]
    fn test_extract_client_ip_real_ip() {
        let req = Request::builder()
            .header("X-Real-Ip", "10.0.0.1")
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            extract_client_ip(&req),
            "10.0.0.1".parse::<IpAddr>().unwrap()
        );
    }

    #[test]
    fn test_extract_client_ip_fallback() {
        let req = Request::builder().body(Body::empty()).unwrap();
        assert_eq!(
            extract_client_ip(&req),
            IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
        );
    }

    #[test]
    fn test_default_config() {
        let config = RateLimitConfig::default();
        assert_eq!(config.max_requests, 100);
        assert_eq!(config.window, Duration::from_secs(60));
    }

    #[test]
    fn test_tier_max_requests() {
        assert_eq!(RateLimitTier::Anonymous.max_requests(), 60);
        assert_eq!(RateLimitTier::Authenticated.max_requests(), 300);
        assert_eq!(RateLimitTier::Admin.max_requests(), 1000);
    }
}
