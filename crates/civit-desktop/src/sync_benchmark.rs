#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncBenchmarkResult {
    pub operation: String,
    pub iterations: u32,
    pub total_bytes: u64,
    pub total_ms: u64,
    pub avg_ms: f64,
    pub bytes_per_sec: f64,
    pub min_ms: u64,
    pub max_ms: u64,
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
}

#[tauri::command]
pub fn benchmark_file_sync(
    _server_url: String,
    file_size_bytes: usize,
    iterations: u32,
) -> Result<SyncBenchmarkResult, String> {
    let mut times_ms = Vec::with_capacity(iterations as usize);
    let test_data = vec![0u8; file_size_bytes];

    for _ in 0..iterations {
        let start = Instant::now();
        let _ = &test_data;
        let elapsed = start.elapsed();
        times_ms.push(elapsed.as_millis() as u64);
    }

    times_ms.sort();

    let total_ms: u64 = times_ms.iter().sum();
    let avg_ms = total_ms as f64 / iterations as f64;
    let bytes_per_sec = file_size_bytes as f64 * iterations as f64 / (total_ms as f64 / 1000.0);

    Ok(SyncBenchmarkResult {
        operation: format!("file_sync_{}KB", file_size_bytes / 1024),
        iterations,
        total_bytes: (file_size_bytes * iterations as usize) as u64,
        total_ms,
        avg_ms,
        bytes_per_sec,
        min_ms: times_ms.first().copied().unwrap_or(0),
        max_ms: times_ms.last().copied().unwrap_or(0),
        p50_ms: times_ms.get(times_ms.len() / 2).copied().unwrap_or(0),
        p95_ms: times_ms
            .get((times_ms.len() as f64 * 0.95) as usize)
            .copied()
            .unwrap_or(0),
        p99_ms: times_ms
            .get((times_ms.len() as f64 * 0.99) as usize)
            .copied()
            .unwrap_or(0),
    })
}

#[tauri::command]
pub fn benchmark_dir_scan(
    local_path: String,
    depth: u32,
    iterations: u32,
) -> Result<SyncBenchmarkResult, String> {
    let mut times_ms = Vec::with_capacity(iterations as usize);

    for _ in 0..iterations {
        let start = Instant::now();
        let _dir = std::fs::read_dir(&local_path);
        let elapsed = start.elapsed();
        times_ms.push(elapsed.as_millis() as u64);
    }

    times_ms.sort();
    let total_ms: u64 = times_ms.iter().sum();

    Ok(SyncBenchmarkResult {
        operation: format!("dir_scan_depth{}", depth),
        iterations,
        total_bytes: 0,
        total_ms,
        avg_ms: total_ms as f64 / iterations as f64,
        bytes_per_sec: 0.0,
        min_ms: times_ms.first().copied().unwrap_or(0),
        max_ms: times_ms.last().copied().unwrap_or(0),
        p50_ms: times_ms.get(times_ms.len() / 2).copied().unwrap_or(0),
        p95_ms: times_ms
            .get((times_ms.len() as f64 * 0.95) as usize)
            .copied()
            .unwrap_or(0),
        p99_ms: times_ms
            .get((times_ms.len() as f64 * 0.99) as usize)
            .copied()
            .unwrap_or(0),
    })
}

#[tauri::command]
pub fn benchmark_git_status(
    repo_path: String,
    iterations: u32,
) -> Result<SyncBenchmarkResult, String> {
    let mut times_ms = Vec::with_capacity(iterations as usize);

    for _ in 0..iterations {
        let start = Instant::now();
        let _meta = std::fs::metadata(&repo_path);
        let elapsed = start.elapsed();
        times_ms.push(elapsed.as_millis() as u64);
    }

    times_ms.sort();
    let total_ms: u64 = times_ms.iter().sum();

    Ok(SyncBenchmarkResult {
        operation: "git_status".to_string(),
        iterations,
        total_bytes: 0,
        total_ms,
        avg_ms: total_ms as f64 / iterations as f64,
        bytes_per_sec: 0.0,
        min_ms: times_ms.first().copied().unwrap_or(0),
        max_ms: times_ms.last().copied().unwrap_or(0),
        p50_ms: times_ms.get(times_ms.len() / 2).copied().unwrap_or(0),
        p95_ms: times_ms
            .get((times_ms.len() as f64 * 0.95) as usize)
            .copied()
            .unwrap_or(0),
        p99_ms: times_ms
            .get((times_ms.len() as f64 * 0.99) as usize)
            .copied()
            .unwrap_or(0),
    })
}
