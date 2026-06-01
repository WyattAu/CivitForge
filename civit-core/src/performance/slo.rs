#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricType {
    Latency,
    Availability,
    Throughput,
    ErrorRate,
}

impl std::fmt::Display for MetricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricType::Latency => write!(f, "latency"),
            MetricType::Availability => write!(f, "availability"),
            MetricType::Throughput => write!(f, "throughput"),
            MetricType::ErrorRate => write!(f, "error_rate"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloTarget {
    pub p99: Option<f64>,
    pub p95: Option<f64>,
    pub p50: Option<f64>,
    pub percentage: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloDefinition {
    pub name: String,
    pub description: String,
    pub target: SloTarget,
    pub window: Duration,
    pub service: String,
    pub metric_type: MetricType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SloStatus {
    Met,
    Breached,
    AtRisk,
}

impl std::fmt::Display for SloStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SloStatus::Met => write!(f, "Met"),
            SloStatus::Breached => write!(f, "Breached"),
            SloStatus::AtRisk => write!(f, "AtRisk"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloReport {
    pub slo_name: String,
    pub current_value: f64,
    pub target: f64,
    pub compliance_ratio: f64,
    pub status: SloStatus,
}

#[derive(Debug, Clone)]
pub struct SloEvaluator;

impl SloEvaluator {
    pub fn evaluate(slo: &SloDefinition, measurements: &[f64]) -> SloReport {
        let target_val = match slo.metric_type {
            MetricType::Latency => {
                if measurements.is_empty() {
                    0.0
                } else {
                    let sorted = {
                        let mut s = measurements.to_vec();
                        s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                        s
                    };
                    let p99_idx = ((sorted.len() as f64 * 0.99) as usize).min(sorted.len() - 1);
                    sorted[p99_idx]
                }
            }
            MetricType::Availability => {
                let sum: f64 = measurements.iter().sum();
                let count = measurements.len() as f64;
                if count > 0.0 {
                    (sum / count) * 100.0
                } else {
                    100.0
                }
            }
            MetricType::Throughput => {
                let sum: f64 = measurements.iter().sum();
                let count = measurements.len() as f64;
                if count > 0.0 { sum / count } else { 0.0 }
            }
            MetricType::ErrorRate => {
                let sum: f64 = measurements.iter().sum();
                let count = measurements.len() as f64;
                if count > 0.0 { sum / count } else { 0.0 }
            }
        };

        let target = match slo.metric_type {
            MetricType::Latency => slo.target.p99.unwrap_or(f64::MAX),
            MetricType::Availability => slo.target.percentage.unwrap_or(99.9),
            MetricType::Throughput => slo.target.p50.unwrap_or(0.0),
            MetricType::ErrorRate => slo.target.p95.unwrap_or(0.0),
        };

        let compliant = measurements
            .iter()
            .filter(|&&m| match slo.metric_type {
                MetricType::Latency => m <= target,
                MetricType::Availability => m >= target,
                MetricType::Throughput => m >= target,
                MetricType::ErrorRate => m <= target,
            })
            .count();

        let compliance_ratio = if !measurements.is_empty() {
            compliant as f64 / measurements.len() as f64
        } else {
            1.0
        };

        let status = if compliance_ratio >= 0.99 {
            SloStatus::Met
        } else if compliance_ratio >= 0.95 {
            SloStatus::AtRisk
        } else {
            SloStatus::Breached
        };

        SloReport {
            slo_name: slo.name.clone(),
            current_value: target_val,
            target,
            compliance_ratio,
            status,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceBaseline {
    pub api_read_p99: Duration,
    pub api_write_p99: Duration,
    pub git_clone_1m_repo: Duration,
    pub pipeline_schedule_latency: Duration,
    pub memory_rss_api_pod: u64,
    pub db_query_p99: Duration,
}

impl Default for PerformanceBaseline {
    fn default() -> Self {
        Self {
            api_read_p99: Duration::from_millis(50),
            api_write_p99: Duration::from_millis(200),
            git_clone_1m_repo: Duration::from_secs(2),
            pipeline_schedule_latency: Duration::from_millis(500),
            memory_rss_api_pod: 256 * 1024 * 1024,
            db_query_p99: Duration::from_millis(10),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Regression {
    pub metric_name: String,
    pub baseline_value: f64,
    pub current_value: f64,
    pub regression_factor: f64,
}

#[derive(Debug, Clone)]
pub struct ProfilingReport {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub measurements: HashMap<String, f64>,
    pub baseline: PerformanceBaseline,
    pub regressions: Vec<Regression>,
}

#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    measurements: HashMap<String, Vec<f64>>,
    baseline: PerformanceBaseline,
    regression_threshold: f64,
}

impl PerformanceMonitor {
    pub fn new(baseline: PerformanceBaseline) -> Self {
        Self {
            measurements: HashMap::new(),
            baseline,
            regression_threshold: 2.0,
        }
    }

    pub fn with_regression_threshold(mut self, threshold: f64) -> Self {
        self.regression_threshold = threshold;
        self
    }

    pub fn record(&mut self, metric: &str, value: f64) {
        self.measurements
            .entry(metric.to_string())
            .or_default()
            .push(value);
    }

    pub fn check_baseline(&self) -> Vec<Regression> {
        let mut regressions = vec![];

        for (metric, values) in &self.measurements {
            if values.is_empty() {
                continue;
            }
            let avg = values.iter().sum::<f64>() / values.len() as f64;

            let baseline_val = match metric.as_str() {
                "api_read_p99" => self.baseline.api_read_p99.as_millis() as f64,
                "api_write_p99" => self.baseline.api_write_p99.as_millis() as f64,
                "git_clone_1m_repo" => self.baseline.git_clone_1m_repo.as_millis() as f64,
                "pipeline_schedule_latency" => {
                    self.baseline.pipeline_schedule_latency.as_millis() as f64
                }
                "db_query_p99" => self.baseline.db_query_p99.as_millis() as f64,
                _ => continue,
            };

            if baseline_val > 0.0 && avg > baseline_val * self.regression_threshold {
                regressions.push(Regression {
                    metric_name: metric.clone(),
                    baseline_value: baseline_val,
                    current_value: avg,
                    regression_factor: avg / baseline_val,
                });
            }
        }

        regressions
    }

    pub fn detect_regression(&self, metric: &str) -> bool {
        if let Some(values) = self.measurements.get(metric) {
            if values.is_empty() {
                return false;
            }
            let avg = values.iter().sum::<f64>() / values.len() as f64;

            let baseline_val = match metric {
                "api_read_p99" => self.baseline.api_read_p99.as_millis() as f64,
                "api_write_p99" => self.baseline.api_write_p99.as_millis() as f64,
                "git_clone_1m_repo" => self.baseline.git_clone_1m_repo.as_millis() as f64,
                "pipeline_schedule_latency" => {
                    self.baseline.pipeline_schedule_latency.as_millis() as f64
                }
                "db_query_p99" => self.baseline.db_query_p99.as_millis() as f64,
                _ => return false,
            };

            baseline_val > 0.0 && avg > baseline_val * self.regression_threshold
        } else {
            false
        }
    }

    pub fn measurements(&self) -> &HashMap<String, Vec<f64>> {
        &self.measurements
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadTestScenario {
    pub name: String,
    pub description: String,
    pub target_rps: u32,
    pub duration: Duration,
    pub users: u32,
    pub endpoints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StressTestConfig {
    pub max_rps: u32,
    pub ramp_up_time: Duration,
    pub steady_state_duration: Duration,
    pub tear_down_duration: Duration,
}

impl Default for StressTestConfig {
    fn default() -> Self {
        Self {
            max_rps: 10_000,
            ramp_up_time: Duration::from_secs(60),
            steady_state_duration: Duration::from_secs(300),
            tear_down_duration: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slo_definition_creation() {
        let slo = SloDefinition {
            name: "api-latency".to_string(),
            description: "API read latency SLO".to_string(),
            target: SloTarget {
                p99: Some(100.0),
                p95: None,
                p50: None,
                percentage: None,
            },
            window: Duration::from_secs(300),
            service: "api".to_string(),
            metric_type: MetricType::Latency,
        };
        assert_eq!(slo.name, "api-latency");
        assert_eq!(slo.metric_type, MetricType::Latency);
        assert_eq!(slo.target.p99, Some(100.0));
    }

    #[test]
    fn test_metric_type_display() {
        assert_eq!(MetricType::Latency.to_string(), "latency");
        assert_eq!(MetricType::Availability.to_string(), "availability");
        assert_eq!(MetricType::Throughput.to_string(), "throughput");
        assert_eq!(MetricType::ErrorRate.to_string(), "error_rate");
    }

    #[test]
    fn test_slo_status_display() {
        assert_eq!(SloStatus::Met.to_string(), "Met");
        assert_eq!(SloStatus::Breached.to_string(), "Breached");
        assert_eq!(SloStatus::AtRisk.to_string(), "AtRisk");
    }

    #[test]
    fn test_slo_evaluator_latency_met() {
        let slo = SloDefinition {
            name: "api-latency".to_string(),
            description: String::new(),
            target: SloTarget {
                p99: Some(100.0),
                p95: None,
                p50: None,
                percentage: None,
            },
            window: Duration::from_secs(300),
            service: "api".to_string(),
            metric_type: MetricType::Latency,
        };
        let measurements = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let report = SloEvaluator::evaluate(&slo, &measurements);
        assert_eq!(report.slo_name, "api-latency");
        assert_eq!(report.status, SloStatus::Met);
        assert!(report.compliance_ratio > 0.99);
    }

    #[test]
    fn test_slo_evaluator_latency_breached() {
        let slo = SloDefinition {
            name: "api-latency".to_string(),
            description: String::new(),
            target: SloTarget {
                p99: Some(10.0),
                p95: None,
                p50: None,
                percentage: None,
            },
            window: Duration::from_secs(300),
            service: "api".to_string(),
            metric_type: MetricType::Latency,
        };
        let measurements: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        let report = SloEvaluator::evaluate(&slo, &measurements);
        assert_eq!(report.status, SloStatus::Breached);
    }

    #[test]
    fn test_slo_evaluator_availability_met() {
        let slo = SloDefinition {
            name: "availability".to_string(),
            description: String::new(),
            target: SloTarget {
                p99: None,
                p95: None,
                p50: None,
                percentage: Some(99.9),
            },
            window: Duration::from_secs(3600),
            service: "api".to_string(),
            metric_type: MetricType::Availability,
        };
        let measurements = vec![100.0, 99.9, 99.95, 100.0, 99.9];
        let report = SloEvaluator::evaluate(&slo, &measurements);
        assert_eq!(report.status, SloStatus::Met);
    }

    #[test]
    fn test_slo_evaluator_empty_measurements() {
        let slo = SloDefinition {
            name: "test".to_string(),
            description: String::new(),
            target: SloTarget {
                p99: Some(100.0),
                p95: None,
                p50: None,
                percentage: None,
            },
            window: Duration::from_secs(60),
            service: "api".to_string(),
            metric_type: MetricType::Latency,
        };
        let report = SloEvaluator::evaluate(&slo, &[]);
        assert_eq!(report.status, SloStatus::Met);
        assert_eq!(report.compliance_ratio, 1.0);
    }

    #[test]
    fn test_performance_baseline_default() {
        let baseline = PerformanceBaseline::default();
        assert_eq!(baseline.api_read_p99, Duration::from_millis(50));
        assert_eq!(baseline.api_write_p99, Duration::from_millis(200));
        assert_eq!(baseline.git_clone_1m_repo, Duration::from_secs(2));
        assert_eq!(baseline.memory_rss_api_pod, 256 * 1024 * 1024);
    }

    #[test]
    fn test_performance_monitor_record() {
        let mut monitor = PerformanceMonitor::new(PerformanceBaseline::default());
        monitor.record("api_read_p99", 45.0);
        monitor.record("api_read_p99", 55.0);
        assert_eq!(monitor.measurements().get("api_read_p99").unwrap().len(), 2);
    }

    #[test]
    fn test_performance_monitor_no_regression() {
        let mut monitor = PerformanceMonitor::new(PerformanceBaseline::default());
        monitor.record("api_read_p99", 40.0);
        monitor.record("api_read_p99", 45.0);
        let regressions = monitor.check_baseline();
        assert!(regressions.is_empty());
    }

    #[test]
    fn test_performance_monitor_detect_regression() {
        let mut monitor =
            PerformanceMonitor::new(PerformanceBaseline::default()).with_regression_threshold(1.5);
        monitor.record("api_read_p99", 100.0);
        monitor.record("api_read_p99", 120.0);
        assert!(monitor.detect_regression("api_read_p99"));
    }

    #[test]
    fn test_performance_monitor_check_baseline_finds_regressions() {
        let mut monitor =
            PerformanceMonitor::new(PerformanceBaseline::default()).with_regression_threshold(1.5);
        monitor.record("db_query_p99", 25.0);
        monitor.record("db_query_p99", 20.0);
        let regressions = monitor.check_baseline();
        assert_eq!(regressions.len(), 1);
        assert_eq!(regressions[0].metric_name, "db_query_p99");
    }

    #[test]
    fn test_performance_monitor_unknown_metric() {
        let mut monitor = PerformanceMonitor::new(PerformanceBaseline::default());
        monitor.record("unknown_metric", 1000.0);
        assert!(!monitor.detect_regression("unknown_metric"));
        assert!(monitor.check_baseline().is_empty());
    }

    #[test]
    fn test_load_test_scenario() {
        let scenario = LoadTestScenario {
            name: "api-read".to_string(),
            description: "API read load test".to_string(),
            target_rps: 1000,
            duration: Duration::from_secs(300),
            users: 50,
            endpoints: vec!["GET /api/v1/repos".to_string()],
        };
        assert_eq!(scenario.name, "api-read");
        assert_eq!(scenario.target_rps, 1000);
        assert_eq!(scenario.users, 50);
        assert_eq!(scenario.endpoints.len(), 1);
    }

    #[test]
    fn test_stress_test_config_default() {
        let config = StressTestConfig::default();
        assert_eq!(config.max_rps, 10_000);
        assert_eq!(config.ramp_up_time, Duration::from_secs(60));
        assert_eq!(config.steady_state_duration, Duration::from_secs(300));
        assert_eq!(config.tear_down_duration, Duration::from_secs(30));
    }

    #[test]
    fn test_profiling_report_creation() {
        let baseline = PerformanceBaseline::default();
        let mut measurements = HashMap::new();
        measurements.insert("api_read_p99".to_string(), 45.0);
        let report = ProfilingReport {
            timestamp: chrono::Utc::now(),
            measurements,
            baseline,
            regressions: vec![],
        };
        assert!(report.regressions.is_empty());
        assert!(report.measurements.contains_key("api_read_p99"));
    }

    #[test]
    fn test_slo_evaluator_throughput() {
        let slo = SloDefinition {
            name: "throughput".to_string(),
            description: String::new(),
            target: SloTarget {
                p99: None,
                p95: None,
                p50: Some(1000.0),
                percentage: None,
            },
            window: Duration::from_secs(60),
            service: "api".to_string(),
            metric_type: MetricType::Throughput,
        };
        let measurements = vec![2000.0, 1500.0, 1800.0];
        let report = SloEvaluator::evaluate(&slo, &measurements);
        assert_eq!(report.status, SloStatus::Met);
    }

    #[test]
    fn test_slo_evaluator_error_rate() {
        let slo = SloDefinition {
            name: "error-rate".to_string(),
            description: String::new(),
            target: SloTarget {
                p99: None,
                p95: Some(1.0),
                p50: None,
                percentage: None,
            },
            window: Duration::from_secs(300),
            service: "api".to_string(),
            metric_type: MetricType::ErrorRate,
        };
        let measurements = vec![0.1, 0.2, 0.5, 0.3];
        let report = SloEvaluator::evaluate(&slo, &measurements);
        assert_eq!(report.status, SloStatus::Met);
    }

    #[test]
    fn test_slo_report_fields() {
        let report = SloReport {
            slo_name: "test-slo".to_string(),
            current_value: 42.0,
            target: 100.0,
            compliance_ratio: 1.0,
            status: SloStatus::Met,
        };
        assert_eq!(report.slo_name, "test-slo");
        assert_eq!(report.current_value, 42.0);
        assert_eq!(report.compliance_ratio, 1.0);
    }
}
