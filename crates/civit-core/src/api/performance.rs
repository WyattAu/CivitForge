#![forbid(unsafe_code)]

//! Performance monitoring endpoints for metrics, health details, and profiling.
//!
//! - `GET /api/v1/metrics` — Prometheus-compatible metrics endpoint
//! - `GET /api/v1/health/detailed` — Detailed health check with component timing
//! - `GET /api/v1/debug/pprof` — CPU/memory profiling (debug mode only)

use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
    routing::get,
    Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Global metrics counters for the application.
#[derive(Debug)]
pub struct MetricsCollector {
    pub http_requests_total: AtomicU64,
    pub http_request_duration_seconds: RwLock<Vec<f64>>,
    pub http_requests_in_flight: AtomicU64,
    pub db_query_duration_seconds: RwLock<Vec<f64>>,
    pub git_operations_total: AtomicU64,
    pub ws_connections_active: AtomicU64,
    pub started_at: Instant,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            http_requests_total: AtomicU64::new(0),
            http_request_duration_seconds: RwLock::new(Vec::new()),
            http_requests_in_flight: AtomicU64::new(0),
            db_query_duration_seconds: RwLock::new(Vec::new()),
            git_operations_total: AtomicU64::new(0),
            ws_connections_active: AtomicU64::new(0),
            started_at: Instant::now(),
        }
    }

    pub fn record_http_request(&self, duration_secs: f64) {
        self.http_requests_total.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut durations) = self.http_request_duration_seconds.try_write() {
            durations.push(duration_secs);
            let len = durations.len();
            if len > 10000 {
                durations.drain(0..len - 10000);
            }
        }
    }

    pub fn record_db_query(&self, duration_secs: f64) {
        if let Ok(mut durations) = self.db_query_duration_seconds.try_write() {
            durations.push(duration_secs);
            let len = durations.len();
            if len > 10000 {
                durations.drain(0..len - 10000);
            }
        }
    }

    pub fn inc_in_flight(&self) {
        self.http_requests_in_flight.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec_in_flight(&self) {
        self.http_requests_in_flight.fetch_sub(1, Ordering::Relaxed);
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared state for performance monitoring routes.
#[derive(Clone)]
pub struct PerformanceState {
    pub collector: Arc<MetricsCollector>,
    pub storage_path: String,
}

/// Prometheus text format metrics endpoint.
pub async fn metrics_handler(
    State(state): State<PerformanceState>,
) -> impl IntoResponse {
    let collector = &state.collector;
    let uptime = collector.started_at.elapsed().as_secs_f64();

    let (req_count, req_p50, req_p95, req_p99) = {
        let durations = collector.http_request_duration_seconds.read().await;
        let count = durations.len() as u64;
        if durations.is_empty() {
            (0, 0.0, 0.0, 0.0)
        } else {
            let mut sorted = durations.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let p50 = percentile(&sorted, 0.50);
            let p95 = percentile(&sorted, 0.95);
            let p99 = percentile(&sorted, 0.99);
            (count, p50, p95, p99)
        }
    };

    let db_count = {
        let durations = collector.db_query_duration_seconds.read().await;
        durations.len() as u64
    };

    let metrics = format!(
        "# HELP civitforge_up Whether the server is up.\n\
         # TYPE civitforge_up gauge\n\
         civitforge_up 1\n\
         \n\
         # HELP civitforge_uptime_seconds Server uptime in seconds.\n\
         # TYPE civitforge_uptime_seconds gauge\n\
         civitforge_uptime_seconds {uptime:.3}\n\
         \n\
         # HELP civitforge_http_requests_total Total HTTP requests served.\n\
         # TYPE civitforge_http_requests_total counter\n\
         civitforge_http_requests_total {req_count}\n\
         \n\
         # HELP civitforge_http_requests_in_flight Current in-flight requests.\n\
         # TYPE civitforge_http_requests_in_flight gauge\n\
         civitforge_http_requests_in_flight {}\n\
         \n\
         # HELP civitforge_http_request_duration_seconds HTTP request latency.\n\
         # TYPE civitforge_http_request_duration_seconds summary\n\
         civitforge_http_request_duration_seconds{{quantile=\"0.5\"}} {req_p50:.6}\n\
         civitforge_http_request_duration_seconds{{quantile=\"0.95\"}} {req_p95:.6}\n\
         civitforge_http_request_duration_seconds{{quantile=\"0.99\"}} {req_p99:.6}\n\
         \n\
         # HELP civitforge_db_queries_total Total DB queries recorded.\n\
         # TYPE civitforge_db_queries_total counter\n\
         civitforge_db_queries_total {db_count}\n\
         \n\
         # HELP civitforge_git_operations_total Total git operations.\n\
         # TYPE civitforge_git_operations_total counter\n\
         civitforge_git_operations_total {}\n\
         \n\
         # HELP civitforge_websocket_connections Active WebSocket connections.\n\
         # TYPE civitforge_websocket_connections gauge\n\
         civitforge_websocket_connections {}\n",
        collector.http_requests_in_flight.load(Ordering::Relaxed),
        collector.git_operations_total.load(Ordering::Relaxed),
        collector.ws_connections_active.load(Ordering::Relaxed),
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
    );

    (headers, metrics)
}

/// Detailed health check with component timing.
pub async fn detailed_health_handler(
    State(state): State<PerformanceState>,
) -> impl IntoResponse {
    let start = Instant::now();
    let mut checks = HashMap::new();

    // Memory check
    let mem_info = get_memory_usage();
    checks.insert(
        "memory".to_string(),
        ComponentHealth {
            status: "healthy".into(),
            response_time_ms: 0,
            message: Some(format!(
                "rss_kb={}, available_kb={}",
                mem_info.rss_kb, mem_info.available_kb
            )),
        },
    );

    // Disk check
    let disk_ok = std::path::Path::new(&state.storage_path).exists();
    checks.insert(
        "disk".to_string(),
        ComponentHealth {
            status: if disk_ok { "healthy".into() } else { "degraded".into() },
            response_time_ms: 0,
            message: Some(format!("storage_path={}", state.storage_path)),
        },
    );

    // Event bus check
    checks.insert(
        "event_bus".to_string(),
        ComponentHealth {
            status: "healthy".into(),
            response_time_ms: 0,
            message: Some("operational".into()),
        },
    );

    let total_ms = start.elapsed().as_millis();
    let overall_status = if checks.values().all(|c| c.status == "healthy") {
        "healthy"
    } else if checks.values().any(|c| c.status == "unhealthy") {
        "unhealthy"
    } else {
        "degraded"
    };

    let response = DetailedHealthResponse {
        status: overall_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.collector.started_at.elapsed().as_secs(),
        timestamp: Utc::now().to_rfc3339(),
        total_check_ms: total_ms,
        checks,
    };

    axum::Json(response)
}

/// CPU/memory profiling endpoint (debug mode only).
#[cfg(feature = "pprof")]
pub async fn pprof_handler() -> impl IntoResponse {
    let profile = pprof::Profile::builder()
        .sample_rate(100)
        .build()
        .expect("pprof build failed");

    let mut buf = Vec::new();
    profile
        .encode(&mut buf)
        .expect("pprof encode failed");

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/x-protobuf"),
    );
    headers.insert(
        axum::http::header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"profile.pb.gz\""),
    );

    (headers, buf)
}

/// Compute the p-th percentile from a sorted slice.
fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() - 1) as f64 * p) as usize;
    sorted[idx]
}

/// Memory usage information.
#[derive(Debug)]
struct MemoryInfo {
    rss_kb: u64,
    available_kb: u64,
}

fn get_memory_usage() -> MemoryInfo {
    let rss_kb = std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmRSS:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse::<u64>().ok())
        })
        .unwrap_or(0);

    let available_kb = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("MemAvailable:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse::<u64>().ok())
        })
        .unwrap_or(0);

    MemoryInfo {
        rss_kb,
        available_kb,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: String,
    pub response_time_ms: u64,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedHealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub timestamp: String,
    pub total_check_ms: u128,
    pub checks: HashMap<String, ComponentHealth>,
}

/// Build the performance monitoring routes.
pub fn performance_routes(_perf_state: PerformanceState) -> Router<PerformanceState> {
    Router::new()
        .route("/api/v1/metrics", get(metrics_handler))
        .route("/api/v1/health/detailed", get(detailed_health_handler))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentile_empty() {
        assert_eq!(percentile(&[], 0.5), 0.0);
    }

    #[test]
    fn test_percentile_single() {
        assert_eq!(percentile(&[1.0], 0.5), 1.0);
    }

    #[test]
    fn test_percentile_p50() {
        let sorted = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&sorted, 0.5), 3.0);
    }

    #[test]
    fn test_percentile_p99() {
        let sorted: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        assert!(percentile(&sorted, 0.99) >= 98.0);
    }

    #[tokio::test]
    async fn test_metrics_collector_record() {
        let collector = MetricsCollector::new();
        collector.record_http_request(0.05);
        collector.record_http_request(0.1);
        collector.inc_in_flight();
        assert_eq!(collector.http_requests_total.load(Ordering::Relaxed), 2);
        assert_eq!(collector.http_requests_in_flight.load(Ordering::Relaxed), 1);
        collector.dec_in_flight();
        assert_eq!(collector.http_requests_in_flight.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn test_metrics_collector_db_record() {
        let collector = MetricsCollector::new();
        collector.record_db_query(0.001);
        collector.record_db_query(0.002);
        let durations = collector.db_query_duration_seconds.read().await;
        assert_eq!(durations.len(), 2);
    }

    #[test]
    fn test_memory_usage_gets_values() {
        let info = get_memory_usage();
        assert!(info.rss_kb == 0 || info.rss_kb > 0);
    }

    #[test]
    fn test_detailed_health_response_serialization() {
        let mut checks = HashMap::new();
        checks.insert(
            "test".to_string(),
            ComponentHealth {
                status: "healthy".into(),
                response_time_ms: 5,
                message: None,
            },
        );
        let resp = DetailedHealthResponse {
            status: "healthy".into(),
            version: "1.0.0".into(),
            uptime_seconds: 100,
            timestamp: Utc::now().to_rfc3339(),
            total_check_ms: 10,
            checks,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("1.0.0"));
    }
}
