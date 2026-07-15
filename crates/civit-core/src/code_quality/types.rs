use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricCategory {
    #[serde(rename = "complexity")]
    Complexity,
    #[serde(rename = "duplication")]
    Duplication,
    #[serde(rename = "code_smells")]
    CodeSmells,
    #[serde(rename = "technical_debt")]
    TechnicalDebt,
    #[serde(rename = "custom")]
    Custom,
}

impl std::fmt::Display for MetricCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Complexity => write!(f, "complexity"),
            Self::Duplication => write!(f, "duplication"),
            Self::CodeSmells => write!(f, "code_smells"),
            Self::TechnicalDebt => write!(f, "technical_debt"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

impl std::str::FromStr for MetricCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "complexity" => Ok(Self::Complexity),
            "duplication" => Ok(Self::Duplication),
            "code_smells" => Ok(Self::CodeSmells),
            "technical_debt" => Ok(Self::TechnicalDebt),
            "custom" => Ok(Self::Custom),
            _ => Err(format!("unknown metric category: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RuleType {
    #[serde(rename = "pattern")]
    Pattern,
    #[serde(rename = "regex")]
    Regex,
    #[serde(rename = "ast")]
    Ast,
    #[serde(rename = "metric")]
    Metric,
    #[serde(rename = "custom")]
    Custom,
}

impl std::fmt::Display for RuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pattern => write!(f, "pattern"),
            Self::Regex => write!(f, "regex"),
            Self::Ast => write!(f, "ast"),
            Self::Metric => write!(f, "metric"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

impl std::str::FromStr for RuleType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pattern" => Ok(Self::Pattern),
            "regex" => Ok(Self::Regex),
            "ast" => Ok(Self::Ast),
            "metric" => Ok(Self::Metric),
            "custom" => Ok(Self::Custom),
            _ => Err(format!("unknown rule type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "critical")]
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

impl std::str::FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "info" => Ok(Self::Info),
            "warning" => Ok(Self::Warning),
            "error" => Ok(Self::Error),
            "critical" => Ok(Self::Critical),
            _ => Err(format!("unknown severity: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetricReport {
    pub repo_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetricSummary {
    pub metric_name: String,
    pub latest_value: f64,
    pub avg_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub measurement_count: i64,
    pub files_affected: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrend {
    pub date: chrono::NaiveDate,
    pub avg_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub measurement_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityAnalysis {
    pub avg_complexity: f64,
    pub max_complexity: f64,
    pub avg_cognitive_complexity: f64,
    pub high_complexity_files: i64,
    pub total_measurements: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationReport {
    pub duplication_ratio: f64,
    pub total_duplicated_lines: f64,
    pub files_with_duplication: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSmellsReport {
    pub total_smells: f64,
    pub smell_density: f64,
    pub files_with_smells: i64,
    pub critical_smells: f64,
    pub major_smells: f64,
    pub minor_smells: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDebtReport {
    pub total_debt_hours: f64,
    pub debt_ratio: f64,
    pub debt_per_file: f64,
    pub remediation_time_priority: f64,
    pub files_with_debt: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMetricRequest {
    pub metric_name: String,
    pub metric_value: f64,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRule {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub description: String,
    pub rule_type: RuleType,
    pub severity: Severity,
    pub pattern: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQualityRuleRequest {
    pub name: String,
    pub description: String,
    pub rule_type: RuleType,
    pub severity: Severity,
    pub pattern: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateQualityRuleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub rule_type: Option<RuleType>,
    pub severity: Option<Severity>,
    pub pattern: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRuleEnforcementResult {
    pub rule_id: Uuid,
    pub rule_name: String,
    pub violations: f64,
    pub files_checked: i64,
    pub files_violating: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRuleV2 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub description: String,
    pub rule_type: RuleType,
    pub severity: Severity,
    pub pattern: Option<String>,
    pub auto_fix: bool,
    pub fix_config: serde_json::Value,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQualityRuleV2Request {
    pub name: String,
    pub description: String,
    pub rule_type: RuleType,
    pub severity: Severity,
    pub pattern: Option<String>,
    pub auto_fix: Option<bool>,
    pub fix_config: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateQualityRuleV2Request {
    pub name: Option<String>,
    pub description: Option<String>,
    pub rule_type: Option<RuleType>,
    pub severity: Option<Severity>,
    pub pattern: Option<String>,
    pub auto_fix: Option<bool>,
    pub fix_config: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRuleVersion {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub version: i32,
    pub config_snapshot: serde_json::Value,
    pub change_description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRuleTestResult {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub test_file: String,
    pub expected_violations: i32,
    pub actual_violations: i32,
    pub passed: bool,
    pub tested_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleTestRequest {
    pub test_file: String,
    pub expected_violations: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleVersionDiff {
    pub from_version: i32,
    pub to_version: i32,
    pub changes: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleAnalytics {
    pub rule_id: Uuid,
    pub total_enforcements: i64,
    pub total_violations: i64,
    pub avg_violations_per_run: f64,
    pub last_enforced_at: Option<DateTime<Utc>>,
    pub trend: Vec<RuleEnforcementTrend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEnforcementTrend {
    pub date: chrono::NaiveDate,
    pub enforcement_count: i64,
    pub violation_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRuleV3 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub description: String,
    pub rule_type: RuleType,
    pub severity: Severity,
    pub pattern: Option<String>,
    pub auto_fix: bool,
    pub fix_config: serde_json::Value,
    pub enabled: bool,
    pub version: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQualityRuleV3Request {
    pub name: String,
    pub description: String,
    pub rule_type: RuleType,
    pub severity: Severity,
    pub pattern: Option<String>,
    pub auto_fix: Option<bool>,
    pub fix_config: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateQualityRuleV3Request {
    pub name: Option<String>,
    pub description: Option<String>,
    pub rule_type: Option<RuleType>,
    pub severity: Option<Severity>,
    pub pattern: Option<String>,
    pub auto_fix: Option<bool>,
    pub fix_config: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EnforcementType {
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "block")]
    Block,
    #[serde(rename = "audit")]
    Audit,
}

impl std::fmt::Display for EnforcementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Warn => write!(f, "warn"),
            Self::Block => write!(f, "block"),
            Self::Audit => write!(f, "audit"),
        }
    }
}

impl std::str::FromStr for EnforcementType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "warn" => Ok(Self::Warn),
            "block" => Ok(Self::Block),
            "audit" => Ok(Self::Audit),
            _ => Err(format!("unknown enforcement type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRuleEnforcement {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub enforcement_type: EnforcementType,
    pub threshold: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEnforcementRequest {
    pub enforcement_type: Option<String>,
    pub threshold: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEnforcementRequest {
    pub enforcement_type: Option<String>,
    pub threshold: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementAnalytics {
    pub rule_id: Uuid,
    pub total_enforcements: i64,
    pub blocked_count: i64,
    pub warned_count: i64,
    pub audited_count: i64,
    pub avg_violations_per_run: f64,
    pub last_enforced_at: Option<DateTime<Utc>>,
    pub trend: Vec<EnforcementTrend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementTrend {
    pub date: chrono::NaiveDate,
    pub enforcement_count: i64,
    pub blocked_count: i64,
    pub warned_count: i64,
    pub audited_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementThresholdResult {
    pub rule_id: Uuid,
    pub rule_name: String,
    pub enforcement_type: String,
    pub threshold: i32,
    pub current_violations: i64,
    pub would_block: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityMetricV3 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub file_path: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMetricV3Request {
    pub file_path: String,
    pub metric_name: String,
    pub metric_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityThresholdV2 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub metric_name: String,
    pub threshold_value: f64,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCodeQualityThresholdV2Request {
    pub metric_name: String,
    pub threshold_value: f64,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCodeQualityThresholdV2Request {
    pub threshold_value: Option<f64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityViolation {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub file_path: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold_value: f64,
    pub severity: String,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityEnforcementReportV2 {
    pub repo_id: Uuid,
    pub total_thresholds: i64,
    pub active_thresholds: i64,
    pub total_violations: i64,
    pub violations_by_severity: serde_json::Value,
    pub violations_by_metric: serde_json::Value,
    pub violations: Vec<CodeQualityViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityScoreV2 {
    pub repo_id: Uuid,
    pub overall_score: f64,
    pub metrics_evaluated: i64,
    pub thresholds_passed: i64,
    pub thresholds_failed: i64,
    pub score_breakdown: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityMetricSummaryV2 {
    pub metric_name: String,
    pub latest_value: f64,
    pub avg_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub measurement_count: i64,
    pub files_affected: i64,
    pub threshold: Option<f64>,
    pub threshold_exceeded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityMetricV5 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub file_path: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMetricV5Request {
    pub file_path: String,
    pub metric_name: String,
    pub metric_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityThresholdV4 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub metric_name: String,
    pub threshold_value: f64,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCodeQualityThresholdV4Request {
    pub metric_name: String,
    pub threshold_value: f64,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCodeQualityThresholdV4Request {
    pub threshold_value: Option<f64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityViolationV2 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub file_path: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold_value: f64,
    pub severity: String,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityEnforcementReportV3 {
    pub repo_id: Uuid,
    pub total_thresholds: i64,
    pub active_thresholds: i64,
    pub total_violations: i64,
    pub violations_by_severity: serde_json::Value,
    pub violations_by_metric: serde_json::Value,
    pub violations: Vec<CodeQualityViolationV2>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityScoreV3 {
    pub repo_id: Uuid,
    pub overall_score: f64,
    pub metrics_evaluated: i64,
    pub thresholds_passed: i64,
    pub thresholds_failed: i64,
    pub score_breakdown: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityMetricSummaryV3 {
    pub metric_name: String,
    pub latest_value: f64,
    pub avg_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub measurement_count: i64,
    pub files_affected: i64,
    pub threshold: Option<f64>,
    pub threshold_exceeded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityMetricV9 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub file_path: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMetricV9Request {
    pub file_path: String,
    pub metric_name: String,
    pub metric_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityThresholdV8 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub metric_name: String,
    pub threshold_value: f64,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCodeQualityThresholdV8Request {
    pub metric_name: String,
    pub threshold_value: f64,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCodeQualityThresholdV8Request {
    pub threshold_value: Option<f64>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityViolationV3 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub file_path: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold_value: f64,
    pub severity: String,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityEnforcementReportV4 {
    pub repo_id: Uuid,
    pub total_thresholds: i64,
    pub active_thresholds: i64,
    pub total_violations: i64,
    pub violations_by_severity: serde_json::Value,
    pub violations_by_metric: serde_json::Value,
    pub violations: Vec<CodeQualityViolationV3>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityScoreV4 {
    pub repo_id: Uuid,
    pub overall_score: f64,
    pub metrics_evaluated: i64,
    pub thresholds_passed: i64,
    pub thresholds_failed: i64,
    pub score_breakdown: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityMetricSummaryV4 {
    pub metric_name: String,
    pub latest_value: f64,
    pub avg_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub measurement_count: i64,
    pub files_affected: i64,
    pub threshold: Option<f64>,
    pub threshold_exceeded: bool,
}
