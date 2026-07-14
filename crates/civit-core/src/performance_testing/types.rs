use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TestType {
    #[serde(rename = "load")]
    Load,
    #[serde(rename = "stress")]
    Stress,
    #[serde(rename = "soak")]
    Soak,
    #[serde(rename = "benchmark")]
    Benchmark,
}

impl std::fmt::Display for TestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load => write!(f, "load"),
            Self::Stress => write!(f, "stress"),
            Self::Soak => write!(f, "soak"),
            Self::Benchmark => write!(f, "benchmark"),
        }
    }
}

impl std::str::FromStr for TestType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "load" => Ok(Self::Load),
            "stress" => Ok(Self::Stress),
            "soak" => Ok(Self::Soak),
            "benchmark" => Ok(Self::Benchmark),
            _ => Err(format!("unknown test type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TestStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
}

impl std::fmt::Display for TestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for TestStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => Err(format!("unknown test status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceTestConfig {
    pub concurrent_users: Option<u64>,
    pub duration_seconds: Option<u64>,
    pub ramp_up_seconds: Option<u64>,
    pub target_rps: Option<u64>,
    pub threshold_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestResults {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_ms: f64,
    pub min_response_ms: f64,
    pub max_response_ms: f64,
    pub p50_response_ms: f64,
    pub p90_response_ms: f64,
    pub p95_response_ms: f64,
    pub p99_response_ms: f64,
    pub requests_per_second: f64,
    pub error_rate: f64,
    pub total_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePerformanceTestRequest {
    pub name: String,
    pub test_type: TestType,
    pub endpoint: Option<String>,
    pub config: Option<PerformanceTestConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestSummary {
    pub total_tests: i64,
    pub completed_tests: i64,
    pub failed_tests: i64,
    pub running_tests: i64,
    pub pending_tests: i64,
    pub by_type: serde_json::Value,
    pub latest_results: Option<PerformanceTestResults>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestRecord {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub test_type: TestType,
    pub endpoint: Option<String>,
    pub config: serde_json::Value,
    pub status: TestStatus,
    pub results: serde_json::Value,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfigEntry {
    pub id: Uuid,
    pub test_id: Uuid,
    pub config_key: String,
    pub config_value: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTestConfigRequest {
    pub config_key: String,
    pub config_value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResultMetric {
    pub id: Uuid,
    pub test_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub percentile: Option<f64>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordTestResultRequest {
    pub metric_name: String,
    pub metric_value: f64,
    pub percentile: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PercentileAnalysis {
    pub metric_name: String,
    pub p50: Option<f64>,
    pub p90: Option<f64>,
    pub p95: Option<f64>,
    pub p99: Option<f64>,
    pub avg: f64,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceComparison {
    pub test_id_1: Uuid,
    pub test_id_2: Uuid,
    pub test_name_1: String,
    pub test_name_2: String,
    pub metrics: Vec<MetricComparison>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricComparison {
    pub metric_name: String,
    pub value_1: f64,
    pub value_2: f64,
    pub change_percent: f64,
    pub improved: bool,
}
