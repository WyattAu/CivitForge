use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FindingSeverity {
    #[serde(rename = "critical")]
    Critical,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "info")]
    Info,
}

impl std::fmt::Display for FindingSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "critical"),
            Self::High => write!(f, "high"),
            Self::Medium => write!(f, "medium"),
            Self::Low => write!(f, "low"),
            Self::Info => write!(f, "info"),
        }
    }
}

impl std::str::FromStr for FindingSeverity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "critical" => Ok(Self::Critical),
            "high" => Ok(Self::High),
            "medium" => Ok(Self::Medium),
            "low" => Ok(Self::Low),
            "info" => Ok(Self::Info),
            _ => Err(format!("unknown finding severity: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum QualityGateCondition {
    #[serde(rename = "min_test_pass_rate")]
    MinTestPassRate { threshold: f64 },
    #[serde(rename = "max_critical_findings")]
    MaxCriticalFindings { threshold: i32 },
    #[serde(rename = "max_high_findings")]
    MaxHighFindings { threshold: i32 },
    #[serde(rename = "min_code_coverage")]
    MinCodeCoverage { threshold: f64 },
    #[serde(rename = "no_failing_tests")]
    NoFailingTests,
    #[serde(rename = "lint_clean")]
    LintClean,
    #[serde(rename = "security_scan_pass")]
    SecurityScanPass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum QualityGateAction {
    #[serde(rename = "block_merge")]
    BlockMerge,
    #[serde(rename = "add_label")]
    AddLabel { label: String },
    #[serde(rename = "comment")]
    Comment { body: String },
    #[serde(rename = "notify")]
    Notify { channel: String, message: String },
    #[serde(rename = "request_review")]
    RequestReview { reviewer: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGate {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub conditions: Vec<QualityGateCondition>,
    pub actions: Vec<QualityGateAction>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateResult {
    pub id: Uuid,
    pub gate_id: Uuid,
    pub pr_id: Option<Uuid>,
    pub status: String,
    pub findings: Vec<QualityGateFinding>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateFinding {
    pub severity: FindingSeverity,
    pub message: String,
    pub file_path: Option<String>,
    pub line_number: Option<i32>,
    pub rule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQualityGateRequest {
    pub name: String,
    pub conditions: Option<Vec<QualityGateCondition>>,
    pub actions: Option<Vec<QualityGateAction>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateQualityGateRequest {
    pub name: Option<String>,
    pub conditions: Option<Vec<QualityGateCondition>>,
    pub actions: Option<Vec<QualityGateAction>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateCheckResult {
    pub gate_id: Uuid,
    pub gate_name: String,
    pub passed: bool,
    pub conditions_checked: i32,
    pub conditions_passed: i32,
    pub conditions_failed: i32,
    pub findings: Vec<QualityGateFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateEnforcementResult {
    pub pr_id: Uuid,
    pub total_gates: i64,
    pub gates_passed: i64,
    pub gates_failed: i64,
    pub can_merge: bool,
    pub gate_results: Vec<GateCheckResult>,
}
