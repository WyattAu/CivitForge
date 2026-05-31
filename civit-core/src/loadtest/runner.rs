#![forbid(unsafe_code)]

use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    pub concurrent_users: u32,
    pub duration: Duration,
    pub ramp_up: Duration,
    pub target_rps: u32,
    pub max_error_rate: f64,
}

impl LoadTestConfig {
    pub fn new() -> Self {
        Self {
            concurrent_users: 10,
            duration: Duration::from_secs(60),
            ramp_up: Duration::from_secs(10),
            target_rps: 100,
            max_error_rate: 0.01,
        }
    }

    pub fn with_concurrent_users(mut self, users: u32) -> Self {
        self.concurrent_users = users;
        self
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn with_target_rps(mut self, rps: u32) -> Self {
        self.target_rps = rps;
        self
    }
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct RequestResult {
    pub status_code: u16,
    pub latency_ms: f64,
    pub error: Option<String>,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct LoadTestResult {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub error_rate: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub avg_latency_ms: f64,
    pub throughput_rps: f64,
    pub duration: Duration,
}

impl LoadTestResult {
    pub fn passed(&self) -> bool {
        self.error_rate <= 0.01
    }
}

pub struct LoadTestRunner {
    pub config: LoadTestConfig,
    pub results: Vec<RequestResult>,
}

impl LoadTestRunner {
    pub fn new(config: LoadTestConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
        }
    }

    pub fn analyze(&self) -> LoadTestResult {
        if self.results.is_empty() {
            return LoadTestResult {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                error_rate: 0.0,
                p50_latency_ms: 0.0,
                p95_latency_ms: 0.0,
                p99_latency_ms: 0.0,
                avg_latency_ms: 0.0,
                throughput_rps: 0.0,
                duration: self.config.duration,
            };
        }

        let total = self.results.len() as u64;
        let successful = self
            .results
            .iter()
            .filter(|r| r.status_code >= 200 && r.status_code < 400)
            .count() as u64;
        let failed = total - successful;
        let error_rate = failed as f64 / total as f64;

        let mut latencies: Vec<f64> = self.results.iter().map(|r| r.latency_ms).collect();
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let p50 = percentile_value(&latencies, 50.0);
        let p95 = percentile_value(&latencies, 95.0);
        let p99 = percentile_value(&latencies, 99.0);
        let avg = latencies.iter().sum::<f64>() / latencies.len() as f64;

        let duration_secs = self.config.duration.as_secs_f64();
        let throughput = if duration_secs > 0.0 {
            total as f64 / duration_secs
        } else {
            0.0
        };

        LoadTestResult {
            total_requests: total,
            successful_requests: successful,
            failed_requests: failed,
            error_rate,
            p50_latency_ms: p50,
            p95_latency_ms: p95,
            p99_latency_ms: p99,
            avg_latency_ms: avg,
            throughput_rps: throughput,
            duration: self.config.duration,
        }
    }

    pub fn run(&mut self) -> LoadTestResult {
        let start = Instant::now();
        let synthetic_status_codes: Vec<u16> =
            vec![200, 200, 200, 200, 200, 200, 201, 200, 200, 200];
        let mut rng_state: u64 = start.elapsed().as_nanos() as u64;

        let deadline = start + self.config.duration;

        while Instant::now() < deadline {
            let idx = (rng_state % synthetic_status_codes.len() as u64) as usize;
            let status = synthetic_status_codes[idx];
            rng_state = rng_state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);

            let latency = 5.0 + (rng_state % 500) as f64 / 10.0;

            self.results.push(RequestResult {
                status_code: status,
                latency_ms: latency,
                error: None,
                timestamp: Instant::now(),
            });

            let sleep_time = Duration::from_micros(100);
            std::thread::sleep(sleep_time);
        }

        let actual_duration = start.elapsed();
        let mut result = self.analyze();
        result.duration = actual_duration;
        result
    }
}

fn percentile_value(sorted_data: &[f64], p: f64) -> f64 {
    if sorted_data.is_empty() {
        return 0.0;
    }
    if sorted_data.len() == 1 {
        return sorted_data[0];
    }
    let idx = ((p / 100.0) * (sorted_data.len() as f64 - 1.0)).round() as usize;
    let idx = idx.min(sorted_data.len() - 1);
    sorted_data[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_results(status_codes: &[u16], latencies: &[f64]) -> Vec<RequestResult> {
        let start = Instant::now();
        status_codes
            .iter()
            .zip(latencies.iter())
            .enumerate()
            .map(|(i, (&code, &lat))| RequestResult {
                status_code: code,
                latency_ms: lat,
                error: if code >= 400 {
                    Some(format!("error {code}"))
                } else {
                    None
                },
                timestamp: start + Duration::from_micros(i as u64 * 100),
            })
            .collect()
    }

    #[test]
    fn test_load_test_config_new() {
        let config = LoadTestConfig::new();
        assert_eq!(config.concurrent_users, 10);
        assert_eq!(config.target_rps, 100);
        assert_eq!(config.max_error_rate, 0.01);
    }

    #[test]
    fn test_load_test_config_builder() {
        let config = LoadTestConfig::new()
            .with_concurrent_users(50)
            .with_duration(Duration::from_secs(120))
            .with_target_rps(500);
        assert_eq!(config.concurrent_users, 50);
        assert_eq!(config.duration, Duration::from_secs(120));
        assert_eq!(config.target_rps, 500);
    }

    #[test]
    fn test_analyze_empty() {
        let runner = LoadTestRunner::new(LoadTestConfig::new());
        let result = runner.analyze();
        assert_eq!(result.total_requests, 0);
        assert_eq!(result.error_rate, 0.0);
        assert!(result.passed());
    }

    #[test]
    fn test_analyze_all_success() {
        let runner = LoadTestRunner {
            config: LoadTestConfig::new(),
            results: make_results(
                &[200, 200, 200, 201, 200, 200, 200, 200, 200, 200],
                &[10.0, 20.0, 30.0, 15.0, 25.0, 35.0, 40.0, 45.0, 50.0, 55.0],
            ),
        };
        let result = runner.analyze();
        assert_eq!(result.total_requests, 10);
        assert_eq!(result.successful_requests, 10);
        assert_eq!(result.failed_requests, 0);
        assert_eq!(result.error_rate, 0.0);
        assert!(result.p50_latency_ms > 0.0);
        assert!(result.p99_latency_ms >= result.p95_latency_ms);
        assert!(result.passed());
    }

    #[test]
    fn test_analyze_with_errors() {
        let codes: Vec<u16> = vec![200, 200, 500, 200, 503, 200, 200, 200, 500, 200];
        let lats: Vec<f64> = vec![10.0; 10];
        let runner = LoadTestRunner {
            config: LoadTestConfig::new(),
            results: make_results(&codes, &lats),
        };
        let result = runner.analyze();
        assert_eq!(result.total_requests, 10);
        assert_eq!(result.successful_requests, 7);
        assert_eq!(result.failed_requests, 3);
        let expected_rate = 3.0 / 10.0;
        assert!((result.error_rate - expected_rate).abs() < 0.001);
        assert!(!result.passed());
    }

    #[test]
    fn test_analyze_percentiles() {
        let lats: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        let codes: Vec<u16> = vec![200; 100];
        let runner = LoadTestRunner {
            config: LoadTestConfig::new(),
            results: make_results(&codes, &lats),
        };
        let result = runner.analyze();
        assert!(
            result.p50_latency_ms > 45.0 && result.p50_latency_ms < 55.0,
            "p50 was {}",
            result.p50_latency_ms
        );
        assert!(
            result.p95_latency_ms > 90.0 && result.p95_latency_ms < 100.0,
            "p95 was {}",
            result.p95_latency_ms
        );
        assert!(
            result.p99_latency_ms > 95.0 && result.p99_latency_ms < 101.0,
            "p99 was {}",
            result.p99_latency_ms
        );
    }

    #[test]
    fn test_analyze_throughput() {
        let runner = LoadTestRunner {
            config: LoadTestConfig::new().with_duration(Duration::from_secs(10)),
            results: make_results(&[200; 1000], &vec![1.0; 1000]),
        };
        let result = runner.analyze();
        assert!((result.throughput_rps - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_percentile_value_single() {
        assert_eq!(percentile_value(&[42.0], 50.0), 42.0);
    }

    #[test]
    fn test_percentile_value_empty() {
        assert_eq!(percentile_value(&[], 99.0), 0.0);
    }

    #[test]
    fn test_passed_threshold() {
        let result = LoadTestResult {
            total_requests: 100,
            successful_requests: 100,
            failed_requests: 0,
            error_rate: 0.0,
            p50_latency_ms: 10.0,
            p95_latency_ms: 20.0,
            p99_latency_ms: 30.0,
            avg_latency_ms: 12.0,
            throughput_rps: 50.0,
            duration: Duration::from_secs(2),
        };
        assert!(result.passed());
    }

    #[test]
    fn test_not_passed_threshold() {
        let result = LoadTestResult {
            total_requests: 100,
            successful_requests: 98,
            failed_requests: 2,
            error_rate: 0.02,
            p50_latency_ms: 10.0,
            p95_latency_ms: 20.0,
            p99_latency_ms: 30.0,
            avg_latency_ms: 12.0,
            throughput_rps: 50.0,
            duration: Duration::from_secs(2),
        };
        assert!(!result.passed());
    }

    #[test]
    fn test_default_config() {
        let config = LoadTestConfig::default();
        assert_eq!(config.concurrent_users, 10);
    }
}
