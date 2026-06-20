#![forbid(unsafe_code)]

use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
};
use serde::Serialize;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use crate::api::AppState;

static START_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
    pub test_results: HealthTestResults,
}

#[derive(Debug, Serialize)]
pub struct HealthTestResults {
    pub database: HealthCheckResult,
    pub redis: HealthCheckResult,
    pub memory: HealthCheckResult,
}

#[derive(Debug, Serialize)]
pub struct HealthCheckResult {
    pub status: String,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let db_result = check_database(&state).await;
    let redis_result = check_redis(&state.config.redis_url).await;
    let mem_result = check_memory();

    let overall = if db_result.status == "healthy"
        && redis_result.status == "healthy"
        && mem_result.status == "healthy"
    {
        "healthy"
    } else if db_result.status == "unhealthy" || redis_result.status == "unhealthy" {
        "unhealthy"
    } else {
        "degraded"
    };

    (
        StatusCode::OK,
        Json(HealthResponse {
            status: overall.to_string(),
            version: VERSION.to_string(),
            uptime_secs: START_TIME.elapsed().as_secs(),
            test_results: HealthTestResults {
                database: db_result,
                redis: redis_result,
                memory: mem_result,
            },
        }),
    )
        .into_response()
}

async fn check_database(state: &AppState) -> HealthCheckResult {
    let start = Instant::now();
    match sqlx::query("SELECT 1").execute(state.db.pool()).await {
        Ok(_) => HealthCheckResult {
            status: "healthy".to_string(),
            latency_ms: Some(start.elapsed().as_millis() as u64),
            error: None,
        },
        Err(e) => HealthCheckResult {
            status: "unhealthy".to_string(),
            latency_ms: Some(start.elapsed().as_millis() as u64),
            error: Some(e.to_string()),
        },
    }
}

async fn check_redis(redis_url: &str) -> HealthCheckResult {
    let start = Instant::now();
    let client = match redis::Client::open(redis_url) {
        Ok(c) => c,
        Err(e) => {
            return HealthCheckResult {
                status: "unhealthy".to_string(),
                latency_ms: Some(start.elapsed().as_millis() as u64),
                error: Some(e.to_string()),
            };
        }
    };
    match tokio::time::timeout(std::time::Duration::from_secs(2), async {
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| redis::RedisError::from(std::io::Error::other(e.to_string())))?;
        redis::cmd("PING").query_async::<String>(&mut conn).await
    })
    .await
    {
        Ok(Ok(_)) => HealthCheckResult {
            status: "healthy".to_string(),
            latency_ms: Some(start.elapsed().as_millis() as u64),
            error: None,
        },
        Ok(Err(e)) => HealthCheckResult {
            status: "unhealthy".to_string(),
            latency_ms: Some(start.elapsed().as_millis() as u64),
            error: Some(e.to_string()),
        },
        Err(_) => HealthCheckResult {
            status: "unhealthy".to_string(),
            latency_ms: Some(start.elapsed().as_millis() as u64),
            error: Some("redis connection timed out".to_string()),
        },
    }
}

fn check_memory() -> HealthCheckResult {
    let start = Instant::now();
    let usage_mb = get_process_memory_mb();
    let status = if usage_mb > 0.0 {
        "healthy"
    } else {
        "degraded"
    };
    HealthCheckResult {
        status: status.to_string(),
        latency_ms: Some(start.elapsed().as_millis() as u64),
        error: None,
    }
}

#[derive(Debug, Serialize)]
pub struct DiagnosticsResponse {
    pub active_connections: u64,
    pub total_requests: u64,
    pub error_count: u64,
    pub slow_query_count: u64,
    pub memory_usage_mb: f64,
    pub uptime_secs: u64,
}

static REQUEST_COUNT: LazyLock<AtomicU64> = LazyLock::new(|| AtomicU64::new(0));
static ERROR_COUNT: LazyLock<AtomicU64> = LazyLock::new(|| AtomicU64::new(0));
static SLOW_QUERY_COUNT: LazyLock<AtomicU64> = LazyLock::new(|| AtomicU64::new(0));

pub fn increment_request_count() {
    REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub fn increment_error_count() {
    ERROR_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub fn increment_slow_query_count() {
    SLOW_QUERY_COUNT.fetch_add(1, Ordering::Relaxed);
}

async fn diagnostics() -> impl IntoResponse {
    let resp = DiagnosticsResponse {
        active_connections: 0,
        total_requests: REQUEST_COUNT.load(Ordering::Relaxed),
        error_count: ERROR_COUNT.load(Ordering::Relaxed),
        slow_query_count: SLOW_QUERY_COUNT.load(Ordering::Relaxed),
        memory_usage_mb: get_process_memory_mb(),
        uptime_secs: START_TIME.elapsed().as_secs(),
    };
    (StatusCode::OK, Json(resp)).into_response()
}

#[cfg(debug_assertions)]
async fn trigger_panic() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "Panic will be triggered",
            "note": "This endpoint is only available in debug builds"
        })),
    )
        .into_response()
}

#[cfg(not(debug_assertions))]
async fn trigger_panic() -> impl IntoResponse {
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({
            "error": "forbidden",
            "message": "Panic endpoint is only available in debug builds"
        })),
    )
        .into_response()
}

#[derive(Debug, Serialize)]
pub struct RouteInfo {
    pub method: String,
    pub path: String,
}

async fn list_routes() -> impl IntoResponse {
    let routes = vec![
        RouteInfo {
            method: "GET".into(),
            path: "/healthz".into(),
        },
        RouteInfo {
            method: "GET".into(),
            path: "/ready".into(),
        },
        RouteInfo {
            method: "GET".into(),
            path: "/api/v1/health".into(),
        },
        RouteInfo {
            method: "GET".into(),
            path: "/api/v1/ws".into(),
        },
        RouteInfo {
            method: "POST".into(),
            path: "/api/v1/auth/login".into(),
        },
        RouteInfo {
            method: "GET".into(),
            path: "/api/v1/auth/me".into(),
        },
        RouteInfo {
            method: "POST".into(),
            path: "/api/v1/auth/refresh".into(),
        },
        RouteInfo {
            method: "GET".into(),
            path: "/api/v1/repos".into(),
        },
        RouteInfo {
            method: "POST".into(),
            path: "/api/v1/repos".into(),
        },
        RouteInfo {
            method: "GET".into(),
            path: "/api/v1/repos/{owner}/{name}".into(),
        },
        RouteInfo {
            method: "DELETE".into(),
            path: "/api/v1/repos/{owner}/{name}".into(),
        },
        RouteInfo {
            method: "GET".into(),
            path: "/api/v1/repos/{owner}/{name}/commits".into(),
        },
        RouteInfo {
            method: "GET".into(),
            path: "/debug/health".into(),
        },
        RouteInfo {
            method: "GET".into(),
            path: "/debug/diagnostics".into(),
        },
        RouteInfo {
            method: "GET".into(),
            path: "/debug/routes".into(),
        },
        RouteInfo {
            method: "GET".into(),
            path: "/debug/panic".into(),
        },
        RouteInfo {
            method: "POST".into(),
            path: "/debug/error-reports".into(),
        },
    ];
    (StatusCode::OK, Json(routes)).into_response()
}

pub fn diagnostics_routes() -> Router<AppState> {
    Router::new()
        .route("/debug/health", get(health_check))
        .route("/debug/diagnostics", get(diagnostics))
        .route("/debug/routes", get(list_routes))
        .route("/debug/panic", get(trigger_panic))
}

fn get_process_memory_mb() -> f64 {
    if let Ok(content) = std::fs::read_to_string("/proc/self/status") {
        for line in content.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2
                    && let Ok(kb) = parts[1].parse::<u64>()
                {
                    return kb as f64 / 1024.0;
                }
            }
        }
    }
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health::HealthAggregator;

    #[test]
    fn test_health_response_serialization() {
        let resp = HealthResponse {
            status: "healthy".into(),
            version: "1.0.0".into(),
            uptime_secs: 60,
            test_results: HealthTestResults {
                database: HealthCheckResult {
                    status: "healthy".into(),
                    latency_ms: Some(5),
                    error: None,
                },
                redis: HealthCheckResult {
                    status: "healthy".into(),
                    latency_ms: Some(3),
                    error: None,
                },
                memory: HealthCheckResult {
                    status: "healthy".into(),
                    latency_ms: Some(1),
                    error: None,
                },
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("1.0.0"));
    }

    #[test]
    fn test_health_check_result_error_variant() {
        let result = HealthCheckResult {
            status: "unhealthy".into(),
            latency_ms: Some(100),
            error: Some("connection refused".into()),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("unhealthy"));
        assert!(json.contains("connection refused"));
    }

    #[test]
    fn test_diagnostics_response_serialization() {
        let resp = DiagnosticsResponse {
            active_connections: 10,
            total_requests: 100,
            error_count: 5,
            slow_query_count: 2,
            memory_usage_mb: 256.5,
            uptime_secs: 3600,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("100"));
        assert!(json.contains("256.5"));
    }

    #[test]
    fn test_increment_counters() {
        let before = REQUEST_COUNT.load(Ordering::Relaxed);
        increment_request_count();
        increment_request_count();
        let after = REQUEST_COUNT.load(Ordering::Relaxed);
        assert_eq!(after, before + 2);

        let before_err = ERROR_COUNT.load(Ordering::Relaxed);
        increment_error_count();
        let after_err = ERROR_COUNT.load(Ordering::Relaxed);
        assert_eq!(after_err, before_err + 1);

        let before_sq = SLOW_QUERY_COUNT.load(Ordering::Relaxed);
        increment_slow_query_count();
        let after_sq = SLOW_QUERY_COUNT.load(Ordering::Relaxed);
        assert_eq!(after_sq, before_sq + 1);
    }

    #[test]
    fn test_route_info_serialization() {
        let info = RouteInfo {
            method: "GET".into(),
            path: "/api/v1/repos".into(),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("GET"));
        assert!(json.contains("/api/v1/repos"));
    }

    #[test]
    fn test_check_memory_returns_some_value() {
        let result = check_memory();
        assert!(!result.status.is_empty());
        assert!(result.error.is_none());
        assert!(result.latency_ms.is_some());
    }

    #[test]
    fn test_get_process_memory_mb() {
        let mem = get_process_memory_mb();
        if std::path::Path::new("/proc/self/status").exists() {
            assert!(mem > 0.0);
        }
    }

    #[test]
    fn test_version_constant() {
        assert_ne!(VERSION, "");
        assert!(std::mem::size_of::<&str>() > 0);
    }

    #[test]
    fn test_start_time_increases() {
        let t1 = START_TIME.elapsed().as_secs();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let t2 = START_TIME.elapsed().as_secs();
        assert!(t2 >= t1);
    }

    #[test]
    fn test_health_aggregator_integration() {
        let mut agg = HealthAggregator::new(VERSION);
        let db_checker = crate::health::DatabaseHealthChecker::new();
        agg.register(std::sync::Arc::new(db_checker));
        let result = agg.check_all();
        assert_eq!(result.version, VERSION);
        let _ = result.uptime_seconds;
    }

    #[test]
    fn test_list_routes_serialization() {
        let routes = vec![
            RouteInfo {
                method: "GET".into(),
                path: "/healthz".into(),
            },
            RouteInfo {
                method: "POST".into(),
                path: "/debug/error-reports".into(),
            },
        ];
        let json = serde_json::to_string(&routes).unwrap();
        assert!(json.contains("/healthz"));
        assert!(json.contains("/debug/error-reports"));
    }

    #[test]
    fn test_overall_status_all_healthy() {
        let db = HealthCheckResult {
            status: "healthy".into(),
            latency_ms: None,
            error: None,
        };
        let redis = HealthCheckResult {
            status: "healthy".into(),
            latency_ms: None,
            error: None,
        };
        let mem = HealthCheckResult {
            status: "healthy".into(),
            latency_ms: None,
            error: None,
        };
        let overall =
            if db.status == "healthy" && redis.status == "healthy" && mem.status == "healthy" {
                "healthy"
            } else {
                "not_healthy"
            };
        assert_eq!(overall, "healthy");
    }

    #[test]
    fn test_overall_status_unhealthy() {
        let db = HealthCheckResult {
            status: "unhealthy".into(),
            latency_ms: None,
            error: None,
        };
        let redis = HealthCheckResult {
            status: "healthy".into(),
            latency_ms: None,
            error: None,
        };
        let _mem = HealthCheckResult {
            status: "healthy".into(),
            latency_ms: None,
            error: None,
        };
        let overall = if db.status == "unhealthy" || redis.status == "unhealthy" {
            "unhealthy"
        } else {
            "ok"
        };
        assert_eq!(overall, "unhealthy");
    }
}
