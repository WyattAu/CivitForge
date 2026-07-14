use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TestRunStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "passed")]
    Passed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "skipped")]
    Skipped,
    #[serde(rename = "error")]
    Error,
}

impl std::fmt::Display for TestRunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Passed => write!(f, "passed"),
            Self::Failed => write!(f, "failed"),
            Self::Skipped => write!(f, "skipped"),
            Self::Error => write!(f, "error"),
        }
    }
}

impl std::str::FromStr for TestRunStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "passed" => Ok(Self::Passed),
            "failed" => Ok(Self::Failed),
            "skipped" => Ok(Self::Skipped),
            "error" => Ok(Self::Error),
            _ => Err(format!("unknown test run status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TestSuiteConfig {
    pub framework: Option<String>,
    pub command: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub parallel: Option<bool>,
    pub max_workers: Option<u32>,
    pub environment: Option<serde_json::Value>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub description: String,
    pub test_type: String,
    pub config: TestSuiteConfig,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRun {
    pub id: Uuid,
    pub suite_id: Uuid,
    pub status: TestRunStatus,
    pub total_tests: i32,
    pub passed_tests: i32,
    pub failed_tests: i32,
    pub skipped_tests: i32,
    pub duration_ms: i32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunResult {
    pub run_id: Uuid,
    pub suite_id: Uuid,
    pub status: TestRunStatus,
    pub total_tests: i32,
    pub passed_tests: i32,
    pub failed_tests: i32,
    pub skipped_tests: i32,
    pub duration_ms: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTestSuiteRequest {
    pub name: String,
    pub description: Option<String>,
    pub test_type: String,
    pub config: Option<TestSuiteConfig>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTestSuiteRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub test_type: Option<String>,
    pub config: Option<TestSuiteConfig>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTestRunRequest {
    pub suite_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteSummary {
    pub total_suites: i64,
    pub enabled_suites: i64,
    pub total_runs: i64,
    pub passed_runs: i64,
    pub failed_runs: i64,
    pub by_type: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunHistory {
    pub suite_id: Uuid,
    pub runs: Vec<TestRun>,
    pub pass_rate: f64,
    pub avg_duration_ms: f64,
    pub trend: Vec<TestRunTrend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunTrend {
    pub date: chrono::NaiveDate,
    pub total_runs: i64,
    pub passed_runs: i64,
    pub failed_runs: i64,
    pub avg_duration_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteConfiguration {
    pub id: Uuid,
    pub suite_id: Uuid,
    pub config_key: String,
    pub config_value: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTestSuiteConfigRequest {
    pub config_key: String,
    pub config_value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTestSuiteConfigRequest {
    pub config_value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteNotification {
    pub id: Uuid,
    pub suite_id: Uuid,
    pub notification_type: String,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTestSuiteNotificationRequest {
    pub notification_type: String,
    pub config: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTestSuiteNotificationRequest {
    pub notification_type: Option<String>,
    pub config: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteAnalytics {
    pub total_suites: i64,
    pub total_runs: i64,
    pub avg_pass_rate: f64,
    pub avg_duration_ms: f64,
    pub most_active_suites: Vec<SuiteActivity>,
    pub failure_trends: Vec<FailureTrend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteActivity {
    pub suite_id: Uuid,
    pub suite_name: String,
    pub run_count: i64,
    pub last_run_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureTrend {
    pub date: chrono::NaiveDate,
    pub failure_count: i64,
    pub failure_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSchedule {
    pub suite_id: Uuid,
    pub cron_expression: String,
    pub timezone: Option<String>,
    pub enabled: bool,
    pub next_run_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteTag {
    pub id: Uuid,
    pub suite_id: Uuid,
    pub tag: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTestSuiteTagRequest {
    pub tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteDependency {
    pub id: Uuid,
    pub suite_id: Uuid,
    pub depends_on_suite_id: Uuid,
    pub dependency_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTestSuiteDependencyRequest {
    pub depends_on_suite_id: Uuid,
    pub dependency_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecutionOrder {
    pub suite_id: Uuid,
    pub suite_name: String,
    pub order: i32,
    pub dependencies: Vec<Uuid>,
    pub can_run_parallel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub repo_id: Uuid,
    pub execution_groups: Vec<Vec<TestExecutionOrder>>,
    pub total_suites: i32,
    pub estimated_duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteDependencySummary {
    pub total_dependencies: i64,
    pub circular_dependencies_detected: i64,
    pub suites_with_dependencies: i64,
    pub suites_without_dependencies: i64,
}
