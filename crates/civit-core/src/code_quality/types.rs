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
