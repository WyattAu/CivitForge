#![forbid(unsafe_code)]

use crate::telemetry::tracing_setup::MetricsSnapshot;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

#[derive(Debug)]
pub struct MetricsCollector {
    pub start_time: Instant,
    pub counters: DashMap<String, AtomicU64>,
    pub histograms: DashMap<String, Vec<f64>>,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            counters: DashMap::new(),
            histograms: DashMap::new(),
        }
    }

    pub fn increment_counter(&self, name: &str, value: u64) {
        if let Some(counter) = self.counters.get(name) {
            counter.fetch_add(value, Ordering::Relaxed);
        } else {
            let counter = AtomicU64::new(value);
            self.counters.insert(name.to_owned(), counter);
        }
    }

    pub fn record_timing(&self, name: &str, value_ms: f64) {
        if let Some(mut hist) = self.histograms.get_mut(name) {
            hist.push(value_ms);
        } else {
            self.histograms.insert(name.to_owned(), vec![value_ms]);
        }
    }

    pub fn gauge(&self, name: &str, value: u64) {
        if let Some(counter) = self.counters.get(name) {
            counter.store(value, Ordering::Relaxed);
        } else {
            let counter = AtomicU64::new(value);
            self.counters.insert(name.to_owned(), counter);
        }
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot::capture()
    }

    pub fn reset(&self) {
        self.counters.clear();
        self.histograms.clear();
    }

    pub fn get_counter(&self, name: &str) -> Option<u64> {
        self.counters.get(name).map(|c| c.load(Ordering::Relaxed))
    }

    pub fn get_histogram(&self, name: &str) -> Option<Vec<f64>> {
        self.histograms.get(name).map(|h| h.clone())
    }
}

#[derive(Debug, Clone)]
pub struct SloReport {
    pub api_p99_latency_ms: f64,
    pub meets_target: bool,
    pub error_budget_remaining: f64,
}

impl SloReport {
    pub fn target_p99_ms() -> f64 {
        200.0
    }

    pub fn error_budget_percentage() -> f64 {
        99.9
    }
}

pub fn validate_slo(snapshot: &MetricsSnapshot) -> SloReport {
    let api_p99_latency_ms = snapshot.avg_http_latency_ms * 3.0;
    let meets_target = api_p99_latency_ms <= SloReport::target_p99_ms();
    let error_budget_remaining = if meets_target {
        100.0
    } else {
        let ratio = SloReport::target_p99_ms() / api_p99_latency_ms;
        (ratio * 100.0).max(0.0)
    };

    SloReport {
        api_p99_latency_ms,
        meets_target,
        error_budget_remaining,
    }
}

pub fn percentile(data: &[f64], p: f64) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((p / 100.0) * (sorted.len() as f64 - 1.0)).round() as usize;
    let idx = idx.min(sorted.len() - 1);
    sorted[idx]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_collector_new() {
        let collector = MetricsCollector::new();
        assert!(collector.counters.is_empty());
        assert!(collector.histograms.is_empty());
    }

    #[test]
    fn test_increment_counter() {
        let collector = MetricsCollector::new();
        collector.increment_counter("requests", 1);
        collector.increment_counter("requests", 1);
        collector.increment_counter("requests", 1);
        assert_eq!(collector.get_counter("requests"), Some(3));
    }

    #[test]
    fn test_increment_counter_new_key() {
        let collector = MetricsCollector::new();
        collector.increment_counter("new_counter", 5);
        assert_eq!(collector.get_counter("new_counter"), Some(5));
    }

    #[test]
    fn test_gauge() {
        let collector = MetricsCollector::new();
        collector.gauge("connections", 10);
        assert_eq!(collector.get_counter("connections"), Some(10));
        collector.gauge("connections", 20);
        assert_eq!(collector.get_counter("connections"), Some(20));
    }

    #[test]
    fn test_record_timing() {
        let collector = MetricsCollector::new();
        collector.record_timing("latency", 10.0);
        collector.record_timing("latency", 20.0);
        collector.record_timing("latency", 30.0);
        let hist = collector.get_histogram("latency").unwrap();
        assert_eq!(hist.len(), 3);
    }

    #[test]
    fn test_reset() {
        let collector = MetricsCollector::new();
        collector.increment_counter("test", 100);
        collector.record_timing("timing", 50.0);
        collector.reset();
        assert_eq!(collector.get_counter("test"), None);
        assert_eq!(collector.get_histogram("timing"), None);
    }

    #[test]
    fn test_percentile_empty() {
        assert_eq!(percentile(&[], 50.0), 0.0);
    }

    #[test]
    fn test_percentile_single() {
        assert_eq!(percentile(&[42.0], 50.0), 42.0);
    }

    #[test]
    fn test_percentile_p50() {
        let data = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        assert_eq!(percentile(&data, 50.0), 30.0);
    }

    #[test]
    fn test_percentile_p99() {
        let data: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        assert_eq!(percentile(&data, 99.0), 99.0);
    }

    #[test]
    fn test_percentile_p95() {
        let data: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        assert_eq!(percentile(&data, 95.0), 95.0);
    }

    #[test]
    fn test_validate_slo_meets_target() {
        crate::telemetry::tracing_setup::reset_all_metrics();
        crate::telemetry::tracing_setup::record_http_request(Duration::from_millis(30));
        let snap = MetricsSnapshot::capture();
        let report = validate_slo(&snap);
        assert!(report.meets_target);
        assert!(report.error_budget_remaining > 0.0);
    }

    #[test]
    fn test_validate_slo_exceeds_target() {
        crate::telemetry::tracing_setup::reset_all_metrics();
        crate::telemetry::tracing_setup::record_http_request(Duration::from_millis(200));
        let snap = MetricsSnapshot::capture();
        let report = validate_slo(&snap);
        assert!(!report.meets_target);
        assert!(report.error_budget_remaining < 100.0);
    }

    #[test]
    fn test_slo_report_targets() {
        assert_eq!(SloReport::target_p99_ms(), 200.0);
        assert_eq!(SloReport::error_budget_percentage(), 99.9);
    }
}
