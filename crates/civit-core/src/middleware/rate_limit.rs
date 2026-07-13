//! Per-user rate limiting middleware with tiered limits and security headers.
//!
//! Supports three tiers:
//! - Anonymous: 60 requests/minute (by IP)
//! - Authenticated: 300 requests/minute (by user ID)
//! - Admin: 1000 requests/minute (by user ID)
//!
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

/// Per-key (IP or user ID) sliding window state. Thread-safe via tokio Mutex.
#[derive(Debug, Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    buckets: Arc<Mutex<HashMap<String, Bucket>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            buckets: Arc::new(Mutex::new(HashMap::new())),
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
                    limit.to_string().into(),
                ),
                (
                    HeaderName::from_static("x-ratelimit-remaining"),
                    "0".into(),
                ),
                (
                    HeaderName::from_static("x-ratelimit-reset"),
                    reset_epoch.to_string().into(),
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
        limit.to_string().parse().unwrap(),
    );
    headers.insert(
        HeaderName::from_static("x-ratelimit-remaining"),
        remaining.to_string().parse().unwrap(),
    );
    headers.insert(
        HeaderName::from_static("x-ratelimit-reset"),
        reset_epoch.to_string().parse().unwrap(),
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
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);
        let key = "ip:1.2.3.4";

        // Exhaust the limit
        for _ in 0..60 {
            limiter.check(key, RateLimitTier::Anonymous).await;
        }

        let (allowed, _, _, _, _) = limiter.check(key, RateLimitTier::Anonymous).await;
        assert!(!allowed);

        // Wait for window to expire
        tokio::time::sleep(Duration::from_millis(1050)).await;

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
