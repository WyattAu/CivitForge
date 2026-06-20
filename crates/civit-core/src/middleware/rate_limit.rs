//! IP-based rate limiting middleware using a sliding window counter.
//!
//! Each IP address gets a configurable number of requests per window.
//! When exceeded, returns 429 Too Many Requests with `Retry-After` header.

#![forbid(unsafe_code)]

#[cfg(test)]
use axum::body::Body;
use axum::{
    extract::Request,
    http::{StatusCode, header},
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

/// Configuration for rate limiting.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Max requests per window per IP.
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

/// Per-IP sliding window state. Thread-safe via tokio Mutex.
#[derive(Debug, Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    buckets: Arc<Mutex<HashMap<IpAddr, Bucket>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            buckets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if request is allowed. Returns `(allowed, retry_after_seconds)`.
    pub async fn check(&self, ip: IpAddr) -> (bool, u32) {
        let mut buckets = self.buckets.lock().await;
        let now = Instant::now();

        let bucket = buckets.entry(ip).or_insert(Bucket {
            count: 0,
            window_start: now,
        });

        // Sliding window: reset if window expired
        if now.duration_since(bucket.window_start) >= self.config.window {
            bucket.count = 0;
            bucket.window_start = now;
        }

        if bucket.count >= self.config.max_requests {
            let elapsed = now.duration_since(bucket.window_start);
            let remaining = self.config.window.saturating_sub(elapsed);
            let retry_after = remaining.as_secs() as u32 + u32::from(remaining.subsec_nanos() > 0);
            return (false, retry_after);
        }

        bucket.count += 1;
        (true, 0)
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

/// Rate limiting middleware function for axum.
pub async fn rate_limit_middleware(req: Request, next: Next) -> Response {
    let state = req.extensions().get::<Arc<RateLimiter>>().cloned();

    let limiter = match state {
        Some(l) => l,
        None => return next.run(req).await,
    };

    let ip = extract_client_ip(&req);
    let (allowed, retry_after) = limiter.check(ip).await;

    if !allowed {
        warn!(ip = %ip, retry_after = retry_after, "Rate limit exceeded");
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            axum::Json(serde_json::json!({
                "error": "rate_limit_exceeded",
                "message": "Too many requests. Please retry later."
            })),
        )
            .into_response();
    }

    next.run(req).await
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
        let ip: IpAddr = "1.2.3.4".parse().unwrap();

        for _ in 0..5 {
            let (allowed, _) = limiter.check(ip).await;
            assert!(allowed);
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let config = RateLimitConfig {
            max_requests: 3,
            window: Duration::from_secs(60),
        };
        let limiter = RateLimiter::new(config);
        let ip: IpAddr = "1.2.3.4".parse().unwrap();

        for _ in 0..3 {
            let (allowed, _) = limiter.check(ip).await;
            assert!(allowed);
        }

        let (allowed, retry) = limiter.check(ip).await;
        assert!(!allowed);
        assert!(retry > 0);
    }

    #[tokio::test]
    async fn test_rate_limiter_independent_ips() {
        let config = RateLimitConfig {
            max_requests: 2,
            window: Duration::from_secs(60),
        };
        let limiter = RateLimiter::new(config);
        let ip1: IpAddr = "1.1.1.1".parse().unwrap();
        let ip2: IpAddr = "2.2.2.2".parse().unwrap();

        for _ in 0..2 {
            limiter.check(ip1).await;
        }

        // IP2 should still be allowed
        let (allowed, _) = limiter.check(ip2).await;
        assert!(allowed);
    }

    #[tokio::test]
    async fn test_rate_limiter_window_resets() {
        let config = RateLimitConfig {
            max_requests: 2,
            window: Duration::from_millis(100),
        };
        let limiter = RateLimiter::new(config);
        let ip: IpAddr = "1.2.3.4".parse().unwrap();

        limiter.check(ip).await;
        limiter.check(ip).await;

        let (allowed, _) = limiter.check(ip).await;
        assert!(!allowed);

        tokio::time::sleep(Duration::from_millis(150)).await;

        let (allowed, _) = limiter.check(ip).await;
        assert!(allowed);
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
}
