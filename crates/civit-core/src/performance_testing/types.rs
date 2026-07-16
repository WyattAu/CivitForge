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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub metric_name: String,
    pub baseline_value: f64,
    pub threshold_percent: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePerformanceBaselineRequest {
    pub metric_name: String,
    pub baseline_value: f64,
    pub threshold_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePerformanceBaselineRequest {
    pub baseline_value: Option<f64>,
    pub threshold_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRegression {
    pub id: Uuid,
    pub baseline_id: Uuid,
    pub test_id: Uuid,
    pub regression_percent: f64,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionStatusUpdate {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrendData {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordTrendDataRequest {
    pub metric_name: String,
    pub metric_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrendAnalysis {
    pub metric_name: String,
    pub data_points: Vec<PerformanceTrendData>,
    pub avg_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub trend_direction: String,
    pub change_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub baseline_id: Uuid,
    pub metric_name: String,
    pub baseline_value: f64,
    pub current_value: f64,
    pub regression_percent: f64,
    pub threshold_percent: f64,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaselineSummary {
    pub total_baselines: i64,
    pub active_regressions: i64,
    pub resolved_regressions: i64,
    pub baselines: Vec<PerformanceBaseline>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestAlertConfig {
    pub id: Uuid,
    pub baseline_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAlertConfigRequest {
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAlertConfigRequest {
    pub alert_type: Option<String>,
    pub threshold: Option<f64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlertHistory {
    pub id: Uuid,
    pub alert_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNotification {
    pub alert_id: Uuid,
    pub metric_name: String,
    pub current_value: f64,
    pub threshold: f64,
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertAnalytics {
    pub total_alerts: i64,
    pub active_alerts: i64,
    pub total_triggers: i64,
    pub triggers_by_type: serde_json::Value,
    pub avg_time_between_triggers_ms: f64,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub trigger_trend: Vec<AlertTriggerTrend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertTriggerTrend {
    pub date: chrono::NaiveDate,
    pub trigger_count: i64,
    pub alert_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestAlertConfigV3 {
    pub id: Uuid,
    pub baseline_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAlertConfigV3Request {
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAlertConfigV3Request {
    pub alert_type: Option<String>,
    pub threshold: Option<f64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlertHistoryV3 {
    pub id: Uuid,
    pub alert_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNotificationV3 {
    pub alert_id: Uuid,
    pub metric_name: String,
    pub current_value: f64,
    pub threshold: f64,
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertAnalyticsV3 {
    pub total_alerts: i64,
    pub active_alerts: i64,
    pub total_triggers: i64,
    pub triggers_by_type: serde_json::Value,
    pub avg_time_between_triggers_ms: f64,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub trigger_trend: Vec<AlertTriggerTrendV3>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertTriggerTrendV3 {
    pub date: chrono::NaiveDate,
    pub trigger_count: i64,
    pub alert_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestAlertConfigV5 {
    pub id: Uuid,
    pub baseline_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAlertConfigV5Request {
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAlertConfigV5Request {
    pub alert_type: Option<String>,
    pub threshold: Option<f64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlertHistoryV5 {
    pub id: Uuid,
    pub alert_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNotificationV5 {
    pub alert_id: Uuid,
    pub metric_name: String,
    pub current_value: f64,
    pub threshold: f64,
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertAnalyticsV5 {
    pub total_alerts: i64,
    pub active_alerts: i64,
    pub total_triggers: i64,
    pub triggers_by_type: serde_json::Value,
    pub avg_time_between_triggers_ms: f64,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub trigger_trend: Vec<AlertTriggerTrendV5>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertTriggerTrendV5 {
    pub date: chrono::NaiveDate,
    pub trigger_count: i64,
    pub alert_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestAlertConfigV9 {
    pub id: Uuid,
    pub baseline_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAlertConfigV9Request {
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAlertConfigV9Request {
    pub alert_type: Option<String>,
    pub threshold: Option<f64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlertHistoryV9 {
    pub id: Uuid,
    pub alert_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNotificationV9 {
    pub alert_id: Uuid,
    pub metric_name: String,
    pub current_value: f64,
    pub threshold: f64,
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertAnalyticsV9 {
    pub total_alerts: i64,
    pub active_alerts: i64,
    pub total_triggers: i64,
    pub triggers_by_type: serde_json::Value,
    pub avg_time_between_triggers_ms: f64,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub trigger_trend: Vec<AlertTriggerTrendV9>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertTriggerTrendV9 {
    pub date: chrono::NaiveDate,
    pub trigger_count: i64,
    pub alert_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestAlertConfigV16 {
    pub id: Uuid,
    pub baseline_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAlertConfigV16Request {
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAlertConfigV16Request {
    pub alert_type: Option<String>,
    pub threshold: Option<f64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlertHistoryV16 {
    pub id: Uuid,
    pub alert_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNotificationV16 {
    pub alert_id: Uuid,
    pub metric_name: String,
    pub current_value: f64,
    pub threshold: f64,
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertAnalyticsV16 {
    pub total_alerts: i64,
    pub active_alerts: i64,
    pub total_triggers: i64,
    pub triggers_by_type: serde_json::Value,
    pub avg_time_between_triggers_ms: f64,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub trigger_trend: Vec<AlertTriggerTrendV16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertTriggerTrendV16 {
    pub date: chrono::NaiveDate,
    pub trigger_count: i64,
    pub alert_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestAlertConfigV18 {
    pub id: Uuid,
    pub baseline_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAlertConfigV18Request {
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAlertConfigV18Request {
    pub alert_type: Option<String>,
    pub threshold: Option<f64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlertHistoryV18 {
    pub id: Uuid,
    pub alert_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNotificationV18 {
    pub alert_id: Uuid,
    pub metric_name: String,
    pub current_value: f64,
    pub threshold: f64,
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertAnalyticsV18 {
    pub total_alerts: i64,
    pub active_alerts: i64,
    pub total_triggers: i64,
    pub triggers_by_type: serde_json::Value,
    pub avg_time_between_triggers_ms: f64,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub trigger_trend: Vec<AlertTriggerTrendV18>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertTriggerTrendV18 {
    pub date: chrono::NaiveDate,
    pub trigger_count: i64,
    pub alert_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestComparisonV20 {
    pub id: Uuid,
    pub baseline_id: Uuid,
    pub comparison_id: Uuid,
    pub metric_name: String,
    pub baseline_value: f64,
    pub comparison_value: f64,
    pub percent_change: f64,
    pub is_regression: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePerformanceTestComparisonV20Request {
    pub baseline_id: Uuid,
    pub comparison_id: Uuid,
    pub metric_name: String,
    pub baseline_value: f64,
    pub comparison_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestRegressionsV20 {
    pub id: Uuid,
    pub baseline_id: Uuid,
    pub metric_name: String,
    pub threshold_percent: f64,
    pub enabled: bool,
    pub last_detected_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePerformanceTestRegressionsV20Request {
    pub baseline_id: Uuid,
    pub metric_name: String,
    pub threshold_percent: Option<f64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePerformanceTestRegressionsV20Request {
    pub threshold_percent: Option<f64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonAnalysisResultV20 {
    pub total_comparisons: i64,
    pub regressions_detected: i64,
    pub improvements_detected: i64,
    pub avg_percent_change: f64,
    pub comparisons: Vec<PerformanceTestComparisonV20>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionDetectionResultV20 {
    pub total_baselines: i64,
    pub active_regressions: i64,
    pub regressions: Vec<PerformanceTestRegressionsV20>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBudgetV23 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub metric_name: String,
    pub budget_value: f64,
    pub alert_threshold: f64,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePerformanceBudgetV23Request {
    pub metric_name: String,
    pub budget_value: f64,
    pub alert_threshold: Option<f64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePerformanceBudgetV23Request {
    pub budget_value: Option<f64>,
    pub alert_threshold: Option<f64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBudgetCheckV23 {
    pub budget_id: Uuid,
    pub metric_name: String,
    pub budget_value: f64,
    pub current_value: f64,
    pub within_budget: bool,
    pub utilization_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrendAnalysisV23 {
    pub metric_name: String,
    pub data_points: Vec<PerformanceTrendData>,
    pub avg_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub trend_direction: String,
    pub change_percent: f64,
    pub forecast_next: Option<f64>,
}
