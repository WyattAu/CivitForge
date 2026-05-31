#![forbid(unsafe_code)]

use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

static HTTP_REQUESTS_TOTAL: LazyLock<AtomicU64> = LazyLock::new(|| AtomicU64::new(0));
static HTTP_REQUEST_DURATION_SUM_NS: LazyLock<AtomicU64> = LazyLock::new(|| AtomicU64::new(0));
static HTTP_REQUEST_DURATION_COUNT: LazyLock<AtomicU64> = LazyLock::new(|| AtomicU64::new(0));
static DB_QUERY_DURATION_SUM_NS: LazyLock<AtomicU64> = LazyLock::new(|| AtomicU64::new(0));
static DB_QUERY_DURATION_COUNT: LazyLock<AtomicU64> = LazyLock::new(|| AtomicU64::new(0));
static DB_QUERIES_TOTAL: LazyLock<AtomicU64> = LazyLock::new(|| AtomicU64::new(0));
static GIT_OPERATIONS_TOTAL: LazyLock<AtomicU64> = LazyLock::new(|| AtomicU64::new(0));
static ACTIVE_WS_CONNECTIONS: LazyLock<AtomicU64> = LazyLock::new(|| AtomicU64::new(0));
static PIPELINE_RUNS_TOTAL: LazyLock<AtomicU64> = LazyLock::new(|| AtomicU64::new(0));

#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub service_name: String,
    pub exporter_endpoint: Option<String>,
    pub trace_level: String,
    pub metrics_enabled: bool,
}

impl TelemetryConfig {
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_owned(),
            exporter_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
            trace_level: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
            metrics_enabled: true,
        }
    }

    pub fn init_tracing(&self) -> std::result::Result<(), String> {
        let level_filter = match self.trace_level.to_lowercase().as_str() {
            "trace" => tracing::level_filters::LevelFilter::TRACE,
            "debug" => tracing::level_filters::LevelFilter::DEBUG,
            "info" => tracing::level_filters::LevelFilter::INFO,
            "warn" => tracing::level_filters::LevelFilter::WARN,
            "error" => tracing::level_filters::LevelFilter::ERROR,
            _ => return Err(format!("unknown trace level: {}", self.trace_level)),
        };

        let subscriber = tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::builder()
                    .with_default_directive(level_filter.into())
                    .from_env_lossy(),
            )
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .map_err(|e| format!("failed to set tracing subscriber: {e}"))?;

        tracing::info!(
            service_name = %self.service_name,
            "tracing initialized"
        );

        Ok(())
    }

    pub fn init_metrics(&self) {
        if !self.metrics_enabled {
            tracing::info!("metrics collection disabled");
        }
        tracing::info!(
            service_name = %self.service_name,
            "metrics initialized"
        );
    }

    pub fn shutdown(&self) {
        tracing::info!(
            service_name = %self.service_name,
            "telemetry shutdown complete"
        );
    }
}

pub fn record_http_request(duration: Duration) {
    HTTP_REQUESTS_TOTAL.fetch_add(1, Ordering::Relaxed);
    HTTP_REQUEST_DURATION_SUM_NS.fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    HTTP_REQUEST_DURATION_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub fn record_db_query(duration: Duration) {
    DB_QUERIES_TOTAL.fetch_add(1, Ordering::Relaxed);
    DB_QUERY_DURATION_SUM_NS.fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    DB_QUERY_DURATION_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub fn record_git_operation() {
    GIT_OPERATIONS_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn set_active_ws_connections(count: u64) {
    ACTIVE_WS_CONNECTIONS.store(count, Ordering::Relaxed);
}

pub fn increment_ws_connections() {
    ACTIVE_WS_CONNECTIONS.fetch_add(1, Ordering::Relaxed);
}

pub fn decrement_ws_connections() {
    ACTIVE_WS_CONNECTIONS.fetch_sub(1, Ordering::Relaxed);
}

pub fn record_pipeline_run() {
    PIPELINE_RUNS_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn http_requests_total() -> u64 {
    HTTP_REQUESTS_TOTAL.load(Ordering::Relaxed)
}

pub fn db_queries_total() -> u64 {
    DB_QUERIES_TOTAL.load(Ordering::Relaxed)
}

pub fn git_operations_total() -> u64 {
    GIT_OPERATIONS_TOTAL.load(Ordering::Relaxed)
}

pub fn active_ws_connections() -> u64 {
    ACTIVE_WS_CONNECTIONS.load(Ordering::Relaxed)
}

pub fn pipeline_runs_total() -> u64 {
    PIPELINE_RUNS_TOTAL.load(Ordering::Relaxed)
}

pub fn avg_http_request_duration_ms() -> f64 {
    let count = HTTP_REQUEST_DURATION_COUNT.load(Ordering::Relaxed);
    if count == 0 {
        return 0.0;
    }
    let sum_ns = HTTP_REQUEST_DURATION_SUM_NS.load(Ordering::Relaxed);
    (sum_ns as f64 / count as f64) / 1_000_000.0
}

pub fn avg_db_query_duration_ms() -> f64 {
    let count = DB_QUERY_DURATION_COUNT.load(Ordering::Relaxed);
    if count == 0 {
        return 0.0;
    }
    let sum_ns = DB_QUERY_DURATION_SUM_NS.load(Ordering::Relaxed);
    (sum_ns as f64 / count as f64) / 1_000_000.0
}

pub fn reset_all_metrics() {
    HTTP_REQUESTS_TOTAL.store(0, Ordering::Relaxed);
    HTTP_REQUEST_DURATION_SUM_NS.store(0, Ordering::Relaxed);
    HTTP_REQUEST_DURATION_COUNT.store(0, Ordering::Relaxed);
    DB_QUERY_DURATION_SUM_NS.store(0, Ordering::Relaxed);
    DB_QUERY_DURATION_COUNT.store(0, Ordering::Relaxed);
    DB_QUERIES_TOTAL.store(0, Ordering::Relaxed);
    GIT_OPERATIONS_TOTAL.store(0, Ordering::Relaxed);
    ACTIVE_WS_CONNECTIONS.store(0, Ordering::Relaxed);
    PIPELINE_RUNS_TOTAL.store(0, Ordering::Relaxed);
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub http_requests: u64,
    pub db_queries: u64,
    pub git_ops: u64,
    pub ws_connections: u32,
    pub pipeline_runs: u64,
    pub avg_http_latency_ms: f64,
    pub avg_db_latency_ms: f64,
}

impl MetricsSnapshot {
    pub fn capture() -> Self {
        Self {
            http_requests: http_requests_total(),
            db_queries: db_queries_total(),
            git_ops: git_operations_total(),
            ws_connections: active_ws_connections() as u32,
            pipeline_runs: pipeline_runs_total(),
            avg_http_latency_ms: avg_http_request_duration_ms(),
            avg_db_latency_ms: avg_db_query_duration_ms(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_config_new() {
        let config = TelemetryConfig {
            service_name: "test-service".to_owned(),
            exporter_endpoint: None,
            trace_level: "debug".to_owned(),
            metrics_enabled: true,
        };
        assert_eq!(config.service_name, "test-service");
        assert_eq!(config.trace_level, "debug");
        assert!(config.metrics_enabled);
    }

    #[test]
    fn test_telemetry_config_new_defaults() {
        let config = TelemetryConfig::new("default");
        assert_eq!(config.trace_level, "info");
        assert_eq!(config.service_name, "default");
    }

    #[test]
    fn test_init_metrics() {
        let config = TelemetryConfig::new("test");
        config.init_metrics();
    }

    #[test]
    fn test_shutdown() {
        let config = TelemetryConfig::new("test");
        config.shutdown();
    }

    #[test]
    fn test_record_http_request() {
        reset_all_metrics();
        record_http_request(Duration::from_millis(10));
        record_http_request(Duration::from_millis(20));
        assert_eq!(http_requests_total(), 2);
        let avg = avg_http_request_duration_ms();
        assert!(avg > 14.0 && avg < 16.0, "avg was {avg}");
    }

    #[test]
    fn test_record_db_query() {
        reset_all_metrics();
        record_db_query(Duration::from_micros(500));
        record_db_query(Duration::from_micros(1500));
        assert_eq!(db_queries_total(), 2);
        let avg = avg_db_query_duration_ms();
        assert!(avg > 0.9 && avg < 1.1, "avg was {avg}");
    }

    #[test]
    fn test_record_git_operation() {
        reset_all_metrics();
        record_git_operation();
        record_git_operation();
        record_git_operation();
        assert_eq!(git_operations_total(), 3);
    }

    #[test]
    fn test_ws_connections() {
        reset_all_metrics();
        increment_ws_connections();
        increment_ws_connections();
        assert_eq!(active_ws_connections(), 2);
        decrement_ws_connections();
        assert_eq!(active_ws_connections(), 1);
        set_active_ws_connections(10);
        assert_eq!(active_ws_connections(), 10);
    }

    #[test]
    fn test_pipeline_runs() {
        reset_all_metrics();
        record_pipeline_run();
        record_pipeline_run();
        assert_eq!(pipeline_runs_total(), 2);
    }

    #[test]
    fn test_avg_http_no_requests() {
        reset_all_metrics();
        assert_eq!(avg_http_request_duration_ms(), 0.0);
    }

    #[test]
    fn test_avg_db_no_queries() {
        reset_all_metrics();
        assert_eq!(avg_db_query_duration_ms(), 0.0);
    }

    #[test]
    fn test_metrics_snapshot() {
        reset_all_metrics();
        record_http_request(Duration::from_millis(5));
        record_db_query(Duration::from_micros(200));
        record_git_operation();
        increment_ws_connections();
        record_pipeline_run();

        let snap = MetricsSnapshot::capture();
        assert_eq!(snap.http_requests, 1);
        assert_eq!(snap.db_queries, 1);
        assert_eq!(snap.git_ops, 1);
        assert_eq!(snap.ws_connections, 1);
        assert_eq!(snap.pipeline_runs, 1);
        assert!(snap.avg_http_latency_ms > 4.0);
    }

    #[test]
    fn test_reset_all_metrics() {
        record_http_request(Duration::from_millis(1));
        record_db_query(Duration::from_micros(100));
        record_git_operation();
        increment_ws_connections();
        record_pipeline_run();
        reset_all_metrics();
        assert_eq!(http_requests_total(), 0);
        assert_eq!(db_queries_total(), 0);
        assert_eq!(git_operations_total(), 0);
        assert_eq!(active_ws_connections(), 0);
        assert_eq!(pipeline_runs_total(), 0);
    }
}
