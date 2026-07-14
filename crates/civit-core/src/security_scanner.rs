#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanType {
    Sast,
    Dast,
    Container,
    Dependency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub id: String,
    pub severity: FindingSeverity,
    pub category: String,
    pub title: String,
    pub description: String,
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
    pub recommendation: String,
    pub cwe_id: Option<String>,
    pub cvss_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanV2 {
    pub id: String,
    pub repo_id: String,
    pub scan_type: ScanType,
    pub status: ScanStatus,
    pub findings: Vec<SecurityFinding>,
    pub score: u32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub id: String,
    pub repo_id: String,
    pub policy_name: String,
    pub rules: PolicyRules,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRules {
    pub max_critical: Option<u32>,
    pub max_high: Option<u32>,
    pub max_medium: Option<u32>,
    pub max_low: Option<u32>,
    pub required_scan_types: Vec<ScanType>,
    pub block_on_failure: bool,
    pub auto_remediate: bool,
    pub custom_rules: HashMap<String, serde_json::Value>,
}

impl Default for PolicyRules {
    fn default() -> Self {
        Self {
            max_critical: Some(0),
            max_high: Some(5),
            max_medium: Some(20),
            max_low: Some(100),
            required_scan_types: vec![ScanType::Sast, ScanType::Dependency],
            block_on_failure: true,
            auto_remediate: false,
            custom_rules: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    pub critical: u32,
    pub high: u32,
    pub medium: u32,
    pub low: u32,
    pub informational: u32,
}

impl ScanSummary {
    pub fn from_findings(findings: &[SecurityFinding]) -> Self {
        let mut summary = Self {
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
            informational: 0,
        };
        for f in findings {
            match f.severity {
                FindingSeverity::Critical => summary.critical += 1,
                FindingSeverity::High => summary.high += 1,
                FindingSeverity::Medium => summary.medium += 1,
                FindingSeverity::Low => summary.low += 1,
                FindingSeverity::Informational => summary.informational += 1,
            }
        }
        summary
    }

    pub fn total(&self) -> u32 {
        self.critical + self.high + self.medium + self.low + self.informational
    }
}

pub trait SecurityScanner: Send + Sync {
    fn scan_type(&self) -> ScanType;
    fn scan(&self, target: &ScanTarget) -> Result<SecurityScanV2, String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanTarget {
    pub repo_id: String,
    pub ref_name: Option<String>,
    pub paths: Vec<String>,
}

pub struct SecurityPolicyEvaluator;

impl SecurityPolicyEvaluator {
    pub fn evaluate(scan: &SecurityScanV2, policy: &SecurityPolicy) -> PolicyEvaluationResult {
        if !policy.enabled {
            return PolicyEvaluationResult {
                passed: true,
                violations: Vec::new(),
            };
        }

        let mut violations = Vec::new();
        let summary = ScanSummary::from_findings(&scan.findings);

        if let Some(max) = policy.rules.max_critical
            && summary.critical > max
        {
            violations.push(PolicyViolation {
                rule: "max_critical".into(),
                expected: max,
                actual: summary.critical,
                message: format!(
                    "Critical findings {} exceeds limit {}",
                    summary.critical, max
                ),
            });
        }

        if let Some(max) = policy.rules.max_high
            && summary.high > max
        {
            violations.push(PolicyViolation {
                rule: "max_high".into(),
                expected: max,
                actual: summary.high,
                message: format!(
                    "High findings {} exceeds limit {}",
                    summary.high, max
                ),
            });
        }

        if let Some(max) = policy.rules.max_medium
            && summary.medium > max
        {
            violations.push(PolicyViolation {
                rule: "max_medium".into(),
                expected: max,
                actual: summary.medium,
                message: format!(
                    "Medium findings {} exceeds limit {}",
                    summary.medium, max
                ),
            });
        }

        if let Some(max) = policy.rules.max_low
            && summary.low > max
        {
            violations.push(PolicyViolation {
                rule: "max_low".into(),
                expected: max,
                actual: summary.low,
                message: format!(
                    "Low findings {} exceeds limit {}",
                    summary.low, max
                ),
            });
        }

        PolicyEvaluationResult {
            passed: violations.is_empty(),
            violations,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvaluationResult {
    pub passed: bool,
    pub violations: Vec<PolicyViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub rule: String,
    pub expected: u32,
    pub actual: u32,
    pub message: String,
}

pub struct StubSecurityScanner {
    scan_type: ScanType,
}

impl StubSecurityScanner {
    pub fn new(scan_type: ScanType) -> Self {
        Self { scan_type }
    }
}

impl SecurityScanner for StubSecurityScanner {
    fn scan_type(&self) -> ScanType {
        self.scan_type
    }

    fn scan(&self, target: &ScanTarget) -> Result<SecurityScanV2, String> {
        Ok(SecurityScanV2 {
            id: uuid::Uuid::new_v4().to_string(),
            repo_id: target.repo_id.clone(),
            scan_type: self.scan_type,
            status: ScanStatus::Completed,
            findings: Vec::new(),
            score: 100,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_finding(severity: FindingSeverity) -> SecurityFinding {
        SecurityFinding {
            id: uuid::Uuid::new_v4().to_string(),
            severity,
            category: "test".into(),
            title: "Test finding".into(),
            description: "A test finding".into(),
            file_path: None,
            line_number: None,
            recommendation: "Fix it".into(),
            cwe_id: None,
            cvss_score: None,
        }
    }

    fn make_scan(findings: Vec<SecurityFinding>) -> SecurityScanV2 {
        SecurityScanV2 {
            id: uuid::Uuid::new_v4().to_string(),
            repo_id: "repo-1".into(),
            scan_type: ScanType::Sast,
            status: ScanStatus::Completed,
            findings,
            score: 100,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
        }
    }

    fn make_policy(max_critical: Option<u32>, max_high: Option<u32>) -> SecurityPolicy {
        SecurityPolicy {
            id: uuid::Uuid::new_v4().to_string(),
            repo_id: "repo-1".into(),
            policy_name: "test-policy".into(),
            rules: PolicyRules {
                max_critical,
                max_high,
                ..PolicyRules::default()
            },
            enabled: true,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_scan_type_serialization() {
        assert_eq!(
            serde_json::to_string(&ScanType::Sast).unwrap(),
            "\"sast\""
        );
        assert_eq!(
            serde_json::to_string(&ScanType::Dast).unwrap(),
            "\"dast\""
        );
    }

    #[test]
    fn test_scan_status_serialization() {
        assert_eq!(
            serde_json::to_string(&ScanStatus::Pending).unwrap(),
            "\"pending\""
        );
        assert_eq!(
            serde_json::to_string(&ScanStatus::Completed).unwrap(),
            "\"completed\""
        );
    }

    #[test]
    fn test_finding_severity_ordering() {
        assert!(FindingSeverity::Critical > FindingSeverity::High);
        assert!(FindingSeverity::High > FindingSeverity::Medium);
        assert!(FindingSeverity::Medium > FindingSeverity::Low);
        assert!(FindingSeverity::Low > FindingSeverity::Informational);
    }

    #[test]
    fn test_scan_summary_from_findings() {
        let findings = vec![
            make_finding(FindingSeverity::Critical),
            make_finding(FindingSeverity::High),
            make_finding(FindingSeverity::High),
            make_finding(FindingSeverity::Medium),
            make_finding(FindingSeverity::Low),
            make_finding(FindingSeverity::Informational),
        ];
        let summary = ScanSummary::from_findings(&findings);
        assert_eq!(summary.critical, 1);
        assert_eq!(summary.high, 2);
        assert_eq!(summary.medium, 1);
        assert_eq!(summary.low, 1);
        assert_eq!(summary.informational, 1);
        assert_eq!(summary.total(), 6);
    }

    #[test]
    fn test_scan_summary_empty() {
        let summary = ScanSummary::from_findings(&[]);
        assert_eq!(summary.total(), 0);
    }

    #[test]
    fn test_policy_rules_default() {
        let rules = PolicyRules::default();
        assert_eq!(rules.max_critical, Some(0));
        assert_eq!(rules.max_high, Some(5));
        assert!(rules.block_on_failure);
        assert!(!rules.auto_remediate);
    }

    #[test]
    fn test_policy_evaluation_pass() {
        let scan = make_scan(vec![make_finding(FindingSeverity::Low)]);
        let policy = make_policy(Some(0), Some(5));
        let result = SecurityPolicyEvaluator::evaluate(&scan, &policy);
        assert!(result.passed);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_policy_evaluation_fail_critical() {
        let scan = make_scan(vec![
            make_finding(FindingSeverity::Critical),
            make_finding(FindingSeverity::Critical),
        ]);
        let policy = make_policy(Some(0), Some(5));
        let result = SecurityPolicyEvaluator::evaluate(&scan, &policy);
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].rule, "max_critical");
    }

    #[test]
    fn test_policy_evaluation_fail_high() {
        let scan = make_scan(vec![
            make_finding(FindingSeverity::High),
            make_finding(FindingSeverity::High),
            make_finding(FindingSeverity::High),
            make_finding(FindingSeverity::High),
            make_finding(FindingSeverity::High),
            make_finding(FindingSeverity::High),
        ]);
        let policy = make_policy(Some(0), Some(5));
        let result = SecurityPolicyEvaluator::evaluate(&scan, &policy);
        assert!(!result.passed);
        assert!(result.violations.iter().any(|v| v.rule == "max_high"));
    }

    #[test]
    fn test_policy_evaluation_disabled() {
        let scan = make_scan(vec![make_finding(FindingSeverity::Critical)]);
        let mut policy = make_policy(Some(0), Some(0));
        policy.enabled = false;
        let result = SecurityPolicyEvaluator::evaluate(&scan, &policy);
        assert!(result.passed);
    }

    #[test]
    fn test_stub_scanner() {
        let scanner = StubSecurityScanner::new(ScanType::Sast);
        assert_eq!(scanner.scan_type(), ScanType::Sast);
        let target = ScanTarget {
            repo_id: "repo-1".into(),
            ref_name: None,
            paths: Vec::new(),
        };
        let scan = scanner.scan(&target).unwrap();
        assert_eq!(scan.status, ScanStatus::Completed);
        assert!(scan.findings.is_empty());
        assert_eq!(scan.score, 100);
    }

    #[test]
    fn test_default_policy_rules_required_scan_types() {
        let rules = PolicyRules::default();
        assert!(rules.required_scan_types.contains(&ScanType::Sast));
        assert!(rules
            .required_scan_types
            .contains(&ScanType::Dependency));
    }
}
