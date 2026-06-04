#![forbid(unsafe_code)]

use axum::{extract::Request, middleware::Next, response::Response};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

pub async fn debug_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let start = Instant::now();

    let response = next.run(request).await;

    let elapsed = start.elapsed();
    let status = response.status();

    if status.is_server_error() {
        error!(
            method = %method,
            path,
            status = status.as_u16(),
            elapsed_ms = elapsed.as_millis() as u64,
            "SERVER ERROR"
        );
    } else if status.is_client_error() {
        warn!(
            method = %method,
            path,
            status = status.as_u16(),
            elapsed_ms = elapsed.as_millis() as u64,
            "CLIENT ERROR"
        );
    } else if elapsed.as_millis() > 1000 {
        warn!(
            method = %method,
            path,
            status = status.as_u16(),
            elapsed_ms = elapsed.as_millis() as u64,
            "SLOW REQUEST"
        );
    } else {
        info!(
            method = %method,
            path,
            status = status.as_u16(),
            elapsed_ms = elapsed.as_millis() as u64,
            "REQUEST"
        );
    }

    response
}

pub struct SlowQueryDetector {
    threshold_ms: AtomicU64,
}

impl SlowQueryDetector {
    pub fn new(threshold_ms: u64) -> Self {
        Self {
            threshold_ms: AtomicU64::new(threshold_ms),
        }
    }

    pub fn check_and_log(&self, query: &str, duration: Duration) {
        let threshold = self.threshold_ms.load(Ordering::Relaxed);
        let ms = duration.as_millis() as u64;
        if ms > threshold {
            warn!(query, elapsed_ms = ms, "SLOW QUERY");
        }
    }

    #[cfg(test)]
    pub fn threshold(&self) -> u64 {
        self.threshold_ms.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slow_query_detector_new() {
        let detector = SlowQueryDetector::new(500);
        assert_eq!(detector.threshold(), 500);
    }

    #[test]
    fn test_slow_query_detector_triggers_above_threshold() {
        let detector = SlowQueryDetector::new(100);
        let duration = Duration::from_millis(200);
        detector.check_and_log("SELECT 1", duration);
        assert_eq!(detector.threshold(), 100);
    }

    #[test]
    fn test_slow_query_detector_below_threshold() {
        let detector = SlowQueryDetector::new(1000);
        let duration = Duration::from_millis(50);
        detector.check_and_log("SELECT 1", duration);
    }

    #[test]
    fn test_slow_query_detector_zero_threshold() {
        let detector = SlowQueryDetector::new(0);
        let duration = Duration::from_nanos(1);
        detector.check_and_log("SELECT 1", duration);
    }
}
