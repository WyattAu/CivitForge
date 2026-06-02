#![forbid(unsafe_code)]

//! CivitForge Performance Benchmark Harness
//!
//! Measures API latency (P50/P95/P99), throughput, and memory usage
//! against a running CivitForge instance. Designed for Phase 6.2
//! performance baseline validation.
//!
//! Usage:
//!   cargo run --release -p civit-core --bin civit-bench -- [BASE_URL]
//!
//! Default BASE_URL: http://localhost:8080

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use reqwest::Client;

const DEFAULT_BASE_URL: &str = "http://localhost:8080";
const CONCURRENT_REQUESTS: usize = 10;
const REQUESTS_PER_ENDPOINT: usize = 100;

#[derive(Debug, Clone)]
struct EndpointTarget {
    method: &'static str,
    path: &'static str,
    body: Option<&'static str>,
}

static TARGETS: &[EndpointTarget] = &[
    EndpointTarget {
        method: "GET",
        path: "/healthz",
        body: None,
    },
    EndpointTarget {
        method: "GET",
        path: "/api/v1/health",
        body: None,
    },
    EndpointTarget {
        method: "POST",
        path: "/api/v1/auth/login",
        body: Some(r#"{"username":"bench","password":"bench-pass"}"#),
    },
    EndpointTarget {
        method: "GET",
        path: "/api/v1/repos",
        body: None,
    },
    EndpointTarget {
        method: "GET",
        path: "/api/v1/users",
        body: None,
    },
    EndpointTarget {
        method: "GET",
        path: "/api/v1/orgs",
        body: None,
    },
];

#[derive(Debug, Clone)]
struct LatencyStats {
    count: usize,
    sum_ns: u64,
    min_ns: u64,
    max_ns: u64,
    sorted_latencies: Vec<u64>,
}

impl LatencyStats {
    fn new() -> Self {
        Self {
            count: 0,
            sum_ns: 0,
            min_ns: u64::MAX,
            max_ns: 0,
            sorted_latencies: Vec::new(),
        }
    }

    fn record(&mut self, latency_ns: u64) {
        self.count += 1;
        self.sum_ns += latency_ns;
        self.min_ns = self.min_ns.min(latency_ns);
        self.max_ns = self.max_ns.max(latency_ns);
        self.sorted_latencies.push(latency_ns);
    }

    fn finalize(&mut self) {
        self.sorted_latencies.sort_unstable();
    }

    fn percentile(&self, pct: f64) -> u64 {
        if self.sorted_latencies.is_empty() {
            return 0;
        }
        let idx = ((pct / 100.0) * (self.count as f64 - 1.0)).round() as usize;
        self.sorted_latencies[idx.min(self.count - 1)]
    }

    fn mean(&self) -> u64 {
        if self.count == 0 {
            0
        } else {
            self.sum_ns / self.count as u64
        }
    }
}

async fn measure_endpoint(
    client: &Client,
    base_url: &str,
    target: &EndpointTarget,
    total_requests: usize,
    concurrency: usize,
) -> LatencyStats {
    let url = format!("{}{}", base_url, target.path);
    let counter = Arc::new(AtomicU64::new(0));
    let latencies = Arc::new(tokio::sync::Mutex::new(Vec::with_capacity(total_requests)));

    let mut handles = Vec::with_capacity(concurrency);
    for _ in 0..concurrency {
        let client = client.clone();
        let url = url.clone();
        let method = target.method;
        let body = target.body.map(String::from);
        let counter = counter.clone();
        let latencies = latencies.clone();

        handles.push(tokio::spawn(async move {
            loop {
                let idx = counter.fetch_add(1, Ordering::Relaxed) as usize;
                if idx >= total_requests {
                    break;
                }

                let start = Instant::now();
                let result = match method {
                    "GET" => client.get(&url).send().await,
                    "POST" => {
                        if let Some(ref b) = body {
                            client
                                .post(&url)
                                .body(b.clone())
                                .header("Content-Type", "application/json")
                                .send()
                                .await
                        } else {
                            client.post(&url).send().await
                        }
                    }
                    _ => client.get(&url).send().await,
                };
                let elapsed = start.elapsed().as_nanos() as u64;

                // We don't care about HTTP errors for latency measurement
                // (e.g., 401 on auth/login is expected without real credentials)
                drop(result);

                latencies.lock().await.push(elapsed);
            }
        }));
    }

    for handle in handles {
        let _ = handle.await;
    }

    let latencies_guard = latencies.lock().await;
    let mut stats = LatencyStats::new();
    for &lat in latencies_guard.iter() {
        stats.record(lat);
    }
    stats.finalize();
    stats
}

fn format_ns(ns: u64) -> String {
    if ns < 1_000 {
        format!("{ns}ns")
    } else if ns < 1_000_000 {
        format!("{:.1}us", ns as f64 / 1_000.0)
    } else {
        format!("{:.1}ms", ns as f64 / 1_000_000.0)
    }
}

fn format_throughput(count: usize, elapsed_ms: u64) -> String {
    if elapsed_ms == 0 {
        "N/A".to_string()
    } else {
        let per_sec = (count as f64 / elapsed_ms as f64) * 1000.0;
        format!("{per_sec:.0} req/s")
    }
}

async fn check_health(client: &Client, base_url: &str) -> bool {
    client
        .get(format!("{base_url}/healthz"))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

fn print_usage() {
    eprintln!("CivitForge Performance Benchmark Harness");
    eprintln!();
    eprintln!("Usage: civit-bench [BASE_URL]");
    eprintln!("  BASE_URL  CivitForge API base URL (default: {DEFAULT_BASE_URL})");
    eprintln!();
    eprintln!("Ensure CivitForge is running (e.g., docker compose up -d)");
}

#[tokio::main]
async fn main() {
    let base_url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

    if base_url == "--help" || base_url == "-h" {
        print_usage();
        std::process::exit(0);
    }

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("failed to create HTTP client");

    // Health check
    eprintln!("Checking CivitForge at {base_url} ...");
    if !check_health(&client, &base_url).await {
        eprintln!("ERROR: CivitForge not responding at {base_url}/healthz");
        eprintln!("Start it first: docker compose up -d");
        std::process::exit(1);
    }
    eprintln!("CivitForge is healthy.");
    eprintln!();

    // Measure memory (RSS) of current process for reference
    let self_rss = get_process_rss_kb();
    if let Some(rss_kb) = self_rss {
        eprintln!("Benchmark harness RSS: {} MB", rss_kb / 1024);
    }
    eprintln!();

    // Header
    eprintln!(
        "{:<45} {:>8} {:>8} {:>8} {:>8} {:>8} {:>10} {:>12}",
        "Endpoint", "Count", "Mean", "P50", "P95", "P99", "Min", "Max"
    );
    eprintln!("{}", "-".repeat(115));

    let total_start = Instant::now();
    let mut all_stats: BTreeMap<&str, (LatencyStats, u64)> = BTreeMap::new();
    let mut total_requests = 0usize;

    for target in TARGETS {
        eprint!("{:<45} ", target.path);
        let start = Instant::now();
        let stats = measure_endpoint(
            &client,
            &base_url,
            target,
            REQUESTS_PER_ENDPOINT,
            CONCURRENT_REQUESTS,
        )
        .await;
        let elapsed = start.elapsed().as_millis() as u64;
        total_requests += stats.count;

        eprintln!(
            "{:>8} {:>8} {:>8} {:>8} {:>8} {:>10} {:>12}",
            stats.count,
            format_ns(stats.mean()),
            format_ns(stats.percentile(50.0)),
            format_ns(stats.percentile(95.0)),
            format_ns(stats.percentile(99.0)),
            format_ns(stats.min_ns),
            format_ns(stats.max_ns),
        );

        all_stats.insert(target.path, (stats, elapsed));
    }

    let total_elapsed = total_start.elapsed().as_millis() as u64;

    eprintln!("{}", "-".repeat(115));
    eprintln!(
        "{:<45} {:>8} {:>41} {:>10} {:>12}",
        "TOTAL",
        total_requests,
        format_throughput(total_requests, total_elapsed),
        format_ns(total_elapsed * 1_000_000 / total_requests.max(1) as u64),
        "",
    );
    eprintln!();
    eprintln!("Total benchmark duration: {total_elapsed}ms");

    // Summary report
    eprintln!();
    eprintln!("=== Performance Baseline Summary ===");
    for target in TARGETS {
        if let Some((stats, _)) = all_stats.get(target.path) {
            let p50_ms = stats.percentile(50.0) as f64 / 1_000_000.0;
            let p99_ms = stats.percentile(99.0) as f64 / 1_000_000.0;
            let ratio = if p50_ms > 0.0 { p99_ms / p50_ms } else { 0.0 };
            let tail_ok = ratio < 2.0;
            eprintln!(
                "  {:<40} P50={:>6.1}ms  P99={:>6.1}ms  ratio={:.1}x  {}",
                target.path,
                p50_ms,
                p99_ms,
                ratio,
                if tail_ok { "OK" } else { "LONG TAIL" },
            );
        }
    }

    // Throughput per endpoint
    eprintln!();
    eprintln!("=== Throughput ===");
    for target in TARGETS {
        if let Some((stats, elapsed_ms)) = all_stats.get(target.path) {
            eprintln!(
                "  {:<40} {}",
                target.path,
                format_throughput(stats.count, *elapsed_ms),
            );
        }
    }
}

#[cfg(unix)]
fn get_process_rss_kb() -> Option<u64> {
    use std::fs;
    fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find(|l| l.starts_with("VmRSS:"))
                .and_then(|l| {
                    l.split_whitespace()
                        .nth(1)
                        .and_then(|v| v.parse::<u64>().ok())
                })
        })
}

#[cfg(not(unix))]
fn get_process_rss_kb() -> Option<u64> {
    None
}
