#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests: u32,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RateLimitDecision {
    Allowed { remaining: u32, reset_at: Instant },
    Rejected { retry_after: Duration },
}

#[derive(Debug, Clone)]
struct RateLimitBucket {
    count: u32,
    window_start: Instant,
}

pub struct RateLimiter {
    config: RateLimitConfig,
    buckets: Mutex<HashMap<String, RateLimitBucket>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            buckets: Mutex::new(HashMap::new()),
        }
    }

    pub fn check(&self, key: &str) -> RateLimitDecision {
        let now = Instant::now();
        let mut buckets = self.buckets.lock().unwrap();

        let bucket = buckets.entry(key.to_string()).or_insert(RateLimitBucket {
            count: 0,
            window_start: now,
        });

        if now.duration_since(bucket.window_start) >= self.config.window {
            bucket.count = 0;
            bucket.window_start = now;
        }

        if bucket.count >= self.config.max_requests {
            let reset_at = bucket.window_start + self.config.window;
            let retry_after = reset_at.duration_since(now);
            RateLimitDecision::Rejected { retry_after }
        } else {
            bucket.count += 1;
            let remaining = self.config.max_requests - bucket.count;
            RateLimitDecision::Allowed {
                remaining,
                reset_at: bucket.window_start + self.config.window,
            }
        }
    }

    pub fn reset(&self, key: &str) {
        let mut buckets = self.buckets.lock().unwrap();
        buckets.remove(key);
    }

    pub fn bucket_count(&self) -> usize {
        self.buckets.lock().unwrap().len()
    }

    pub fn cleanup_expired(&self) -> usize {
        let now = Instant::now();
        let mut buckets = self.buckets.lock().unwrap();
        let before = buckets.len();
        buckets.retain(|_, b| now.duration_since(b.window_start) < self.config.window);
        before - buckets.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RateLimitConfig::default();
        assert_eq!(config.max_requests, 100);
        assert_eq!(config.window, Duration::from_secs(60));
    }

    #[test]
    fn test_allow_within_limit() {
        let limiter = RateLimiter::new(RateLimitConfig {
            max_requests: 5,
            window: Duration::from_secs(60),
        });
        for i in 0..5 {
            match limiter.check("user1") {
                RateLimitDecision::Allowed { remaining, .. } => {
                    assert_eq!(remaining, 5 - i - 1);
                }
                RateLimitDecision::Rejected { .. } => {
                    panic!("should be allowed at request {}", i + 1)
                }
            }
        }
    }

    #[test]
    fn test_reject_over_limit() {
        let limiter = RateLimiter::new(RateLimitConfig {
            max_requests: 2,
            window: Duration::from_secs(60),
        });
        assert!(matches!(
            limiter.check("user1"),
            RateLimitDecision::Allowed { .. }
        ));
        assert!(matches!(
            limiter.check("user1"),
            RateLimitDecision::Allowed { .. }
        ));
        assert!(matches!(
            limiter.check("user1"),
            RateLimitDecision::Rejected { .. }
        ));
    }

    #[test]
    fn test_separate_keys_independent() {
        let limiter = RateLimiter::new(RateLimitConfig {
            max_requests: 1,
            window: Duration::from_secs(60),
        });
        assert!(matches!(
            limiter.check("a"),
            RateLimitDecision::Allowed { .. }
        ));
        assert!(matches!(
            limiter.check("a"),
            RateLimitDecision::Rejected { .. }
        ));
        assert!(matches!(
            limiter.check("b"),
            RateLimitDecision::Allowed { .. }
        ));
    }

    #[test]
    fn test_reset_key() {
        let limiter = RateLimiter::new(RateLimitConfig {
            max_requests: 1,
            window: Duration::from_secs(60),
        });
        limiter.check("user1");
        assert!(matches!(
            limiter.check("user1"),
            RateLimitDecision::Rejected { .. }
        ));
        limiter.reset("user1");
        assert!(matches!(
            limiter.check("user1"),
            RateLimitDecision::Allowed { .. }
        ));
    }

    #[test]
    fn test_reset_nonexistent_key() {
        let limiter = RateLimiter::new(RateLimitConfig::default());
        limiter.reset("nonexistent");
        assert_eq!(limiter.bucket_count(), 0);
    }

    #[test]
    fn test_bucket_count() {
        let limiter = RateLimiter::new(RateLimitConfig::default());
        assert_eq!(limiter.bucket_count(), 0);
        limiter.check("a");
        limiter.check("b");
        limiter.check("c");
        assert_eq!(limiter.bucket_count(), 3);
    }

    #[test]
    fn test_cleanup_expired() {
        let limiter = RateLimiter::new(RateLimitConfig {
            max_requests: 1,
            window: Duration::from_millis(50),
        });
        limiter.check("a");
        limiter.check("b");
        std::thread::sleep(Duration::from_millis(60));
        let removed = limiter.cleanup_expired();
        assert_eq!(removed, 2);
        assert_eq!(limiter.bucket_count(), 0);
    }

    #[test]
    fn test_cleanup_none_expired() {
        let limiter = RateLimiter::new(RateLimitConfig {
            max_requests: 5,
            window: Duration::from_secs(60),
        });
        limiter.check("a");
        let removed = limiter.cleanup_expired();
        assert_eq!(removed, 0);
    }

    #[test]
    fn test_rejected_retry_after() {
        let limiter = RateLimiter::new(RateLimitConfig {
            max_requests: 1,
            window: Duration::from_secs(10),
        });
        limiter.check("user1");
        if let RateLimitDecision::Rejected { retry_after } = limiter.check("user1") {
            assert!(retry_after <= Duration::from_secs(10));
        } else {
            panic!("expected rejected");
        }
    }

    #[test]
    fn test_allowed_remaining_zero() {
        let limiter = RateLimiter::new(RateLimitConfig {
            max_requests: 1,
            window: Duration::from_secs(60),
        });
        if let RateLimitDecision::Allowed { remaining, .. } = limiter.check("user1") {
            assert_eq!(remaining, 0);
        } else {
            panic!("expected allowed");
        }
    }

    #[test]
    fn test_multiple_rejections() {
        let limiter = RateLimiter::new(RateLimitConfig {
            max_requests: 1,
            window: Duration::from_secs(60),
        });
        limiter.check("user1");
        for _ in 0..10 {
            assert!(matches!(
                limiter.check("user1"),
                RateLimitDecision::Rejected { .. }
            ));
        }
    }

    #[test]
    fn test_config_clone() {
        let config = RateLimitConfig {
            max_requests: 50,
            window: Duration::from_secs(30),
        };
        let config2 = config.clone();
        assert_eq!(config.max_requests, config2.max_requests);
        assert_eq!(config.window, config2.window);
    }

    #[test]
    fn test_decision_equality() {
        let now = Instant::now();
        let a = RateLimitDecision::Allowed {
            remaining: 5,
            reset_at: now,
        };
        let b = RateLimitDecision::Allowed {
            remaining: 5,
            reset_at: now,
        };
        assert_eq!(a, b);
    }

    #[test]
    fn test_rejected_equality() {
        let a = RateLimitDecision::Rejected {
            retry_after: Duration::from_secs(1),
        };
        let b = RateLimitDecision::Rejected {
            retry_after: Duration::from_secs(1),
        };
        assert_eq!(a, b);
    }
}
