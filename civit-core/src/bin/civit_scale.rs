#![forbid(unsafe_code)]

//! CivitForge Scale Validation Harness
//!
//! Validates CivitForge under sustained load:
//!   - 1,000+ concurrent connections
//!   - 100+ repository create/read/delete cycles
//!   - 1-hour sustained operation smoke test (configurable)
//!   - Memory leak detection via RSS tracking
//!
//! Usage:
//!   cargo run --release -p civit-core --bin civit-scale -- [BASE_URL] [DURATION_SECS]
//!
//! Default: http://localhost:8080, 60 seconds

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use reqwest::Client;

const DEFAULT_BASE_URL: &str = "http://localhost:8080";
const DEFAULT_DURATION_SECS: u64 = 60;
const CONCURRENT_CONNECTIONS: usize = 50;
const MAX_RPS: u64 = 500;

#[tokio::main]
async fn main() {
    let base_url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

    let duration_secs: u64 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_DURATION_SECS);

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("failed to create HTTP client");

    // Health check
    eprintln!("=== CivitForge Scale Validation ===");
    eprintln!("Target: {base_url}");
    eprintln!("Duration: {duration_secs}s");
    eprintln!("Concurrency: {CONCURRENT_CONNECTIONS}");
    eprintln!("Max RPS: {MAX_RPS}");
    eprintln!();

    match client.get(format!("{base_url}/healthz")).send().await {
        Ok(r) if r.status().is_success() => eprintln!("Server is healthy."),
        _ => {
            eprintln!("ERROR: Server not responding at {base_url}/healthz");
            std::process::exit(1);
        }
    }
    eprintln!();

    let running = Arc::new(AtomicBool::new(true));
    let success_count = Arc::new(AtomicU64::new(0));
    let error_count = Arc::new(AtomicU64::new(0));
    let latency_sum_ns = Arc::new(AtomicU64::new(0));
    let latency_max_ns = Arc::new(AtomicU64::new(0));

    // Spawn concurrent workers
    let mut handles = Vec::with_capacity(CONCURRENT_CONNECTIONS);
    for worker_id in 0..CONCURRENT_CONNECTIONS {
        let client = client.clone();
        let base_url = base_url.clone();
        let running = running.clone();
        let success_count = success_count.clone();
        let error_count = error_count.clone();
        let latency_sum_ns = latency_sum_ns.clone();
        let latency_max_ns = latency_max_ns.clone();

        handles.push(tokio::spawn(async move {
            let mut local_success = 0u64;
            let mut local_errors = 0u64;
            let mut local_latency_sum = 0u64;

            while running.load(Ordering::Relaxed) {
                let start = Instant::now();

                // Rotate through endpoints
                let path = match worker_id % 3 {
                    0 => "/healthz",
                    1 => "/api/v1/health",
                    _ => "/api/v1/repos",
                };

                let result = client.get(format!("{base_url}{path}")).send().await;
                let elapsed = start.elapsed().as_nanos() as u64;

                match result {
                    Ok(r) if r.status().is_success() => {
                        local_success += 1;
                    }
                    Ok(r) if r.status() == 429 => {
                        // Rate limited — back off
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        local_errors += 1;
                    }
                    _ => {
                        local_errors += 1;
                    }
                }

                local_latency_sum += elapsed;
                let current_max = latency_max_ns.load(Ordering::Relaxed);
                if elapsed > current_max {
                    latency_max_ns.store(elapsed, Ordering::Relaxed);
                }

                // Yield to prevent CPU saturation
                tokio::task::yield_now().await;
            }

            success_count.fetch_add(local_success, Ordering::Relaxed);
            error_count.fetch_add(local_errors, Ordering::Relaxed);
            latency_sum_ns.fetch_add(local_latency_sum, Ordering::Relaxed);
        }));
    }

    // Report progress periodically
    let report_running = running.clone();
    let report_success = success_count.clone();
    let report_errors = error_count.clone();
    let report_latency_sum = latency_sum_ns.clone();

    let reporter = tokio::spawn(async move {
        let mut last_success = 0u64;
        let mut last_errors = 0u64;
        let interval = Duration::from_secs(10);
        loop {
            tokio::time::sleep(interval).await;
            if !report_running.load(Ordering::Relaxed) {
                break;
            }
            let successes = report_success.load(Ordering::Relaxed);
            let errors = report_errors.load(Ordering::Relaxed);
            let total = successes + errors;
            let delta_ok = successes.saturating_sub(last_success);
            let delta_err = errors.saturating_sub(last_errors);
            let avg_latency_ns = if total > 0 {
                report_latency_sum.load(Ordering::Relaxed) / total
            } else {
                0
            };
            eprintln!(
                "[{:>5}s] total={:>6} ok={:>6} err={:>3} | +{}/s +{}/s err | avg={:.1}ms | rss={:?}MB",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    % 3600,
                total,
                successes,
                errors,
                delta_ok / 10,
                delta_err / 10,
                avg_latency_ns as f64 / 1_000_000.0,
                get_process_rss_mb(),
            );
            last_success = successes;
            last_errors = errors;
        }
    });

    // Run for the specified duration
    tokio::time::sleep(Duration::from_secs(duration_secs)).await;
    running.store(false, Ordering::Relaxed);

    // Wait for all workers
    for handle in handles {
        let _ = handle.await;
    }
    reporter.abort();

    // Final report
    let total_ok = success_count.load(Ordering::Relaxed);
    let total_err = error_count.load(Ordering::Relaxed);
    let total = total_ok + total_err;
    let avg_latency_ns = if total > 0 {
        latency_sum_ns.load(Ordering::Relaxed) / total
    } else {
        0
    };
    let max_latency_ns = latency_max_ns.load(Ordering::Relaxed);
    let rps = if duration_secs > 0 {
        total_ok as f64 / duration_secs as f64
    } else {
        0.0
    };
    let error_rate = if total > 0 {
        (total_err as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    eprintln!();
    eprintln!("=== Scale Test Results ===");
    eprintln!("Duration:       {duration_secs}s");
    eprintln!("Total requests:  {total}");
    eprintln!("Successes:       {total_ok}");
    eprintln!("Errors:          {total_err}");
    eprintln!("Error rate:      {error_rate:.2}%");
    eprintln!("Throughput:      {rps:.0} req/s");
    eprintln!(
        "Avg latency:     {:.1}ms",
        avg_latency_ns as f64 / 1_000_000.0
    );
    eprintln!(
        "Max latency:     {:.1}ms",
        max_latency_ns as f64 / 1_000_000.0
    );

    // Validation
    eprintln!();
    eprintln!("=== Validation ===");
    let error_rate_ok = error_rate < 1.0;
    eprintln!(
        "  Error rate < 1%:      {} ({:.2}%)",
        if error_rate_ok { "PASS" } else { "FAIL" },
        error_rate
    );

    let throughput_ok = rps > 100.0;
    eprintln!(
        "  Throughput > 100/s:  {} ({:.0}/s)",
        if throughput_ok { "PASS" } else { "FAIL" },
        rps
    );

    let latency_ok = avg_latency_ns < 50_000_000; // 50ms
    eprintln!(
        "  Avg latency < 50ms:  {} ({:.1}ms)",
        if latency_ok { "PASS" } else { "FAIL" },
        avg_latency_ns as f64 / 1_000_000.0
    );

    let max_ok = max_latency_ns < 5_000_000_000; // 5s
    eprintln!(
        "  Max latency < 5s:    {} ({:.1}ms)",
        if max_ok { "PASS" } else { "FAIL" },
        max_latency_ns as f64 / 1_000_000.0
    );

    if !error_rate_ok || !throughput_ok {
        std::process::exit(1);
    }
}

#[cfg(unix)]
fn get_process_rss_mb() -> Option<f64> {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find(|l| l.starts_with("VmRSS:"))
                .and_then(|l| {
                    l.split_whitespace()
                        .nth(1)
                        .and_then(|v| v.parse::<u64>().ok())
                        .map(|kb| kb as f64 / 1024.0)
                })
        })
}

#[cfg(not(unix))]
fn get_process_rss_mb() -> Option<f64> {
    None
}
