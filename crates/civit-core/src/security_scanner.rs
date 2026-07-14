#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleType {
    Regex,
    Keyword,
    Ast,
    Pattern,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleSeverity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_type: RuleType,
    pub severity: RuleSeverity,
    pub pattern: String,
    pub enabled: bool,
    pub version: u32,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SecurityScanRule {
    pub fn new(name: String, description: String, rule_type: RuleType, pattern: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            rule_type,
            severity: RuleSeverity::Medium,
            pattern,
            enabled: true,
            version: 1,
            created_by: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_severity(mut self, severity: RuleSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_created_by(mut self, user_id: &str) -> Self {
        self.created_by = Some(user_id.into());
        self
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        self.updated_at = Utc::now();
    }

    pub fn enable(&mut self) {
        self.enabled = true;
        self.updated_at = Utc::now();
    }

    pub fn update_pattern(&mut self, new_pattern: String) {
        self.pattern = new_pattern;
        self.version += 1;
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleTestResult {
    pub rule_id: String,
    pub input: String,
    pub matched: bool,
    pub matched_lines: Vec<u32>,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleVersion {
    pub version: u32,
    pub pattern: String,
    pub changed_by: Option<String>,
    pub changed_at: DateTime<Utc>,
    pub change_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRuleManager {
    rules: Vec<SecurityScanRule>,
    versions: HashMap<String, Vec<RuleVersion>>,
}

impl ScanRuleManager {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            versions: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule: SecurityScanRule) {
        let version = RuleVersion {
            version: rule.version,
            pattern: rule.pattern.clone(),
            changed_by: rule.created_by.clone(),
            changed_at: rule.created_at,
            change_notes: Some("Initial creation".into()),
        };
        self.versions
            .entry(rule.id.clone())
            .or_default()
            .push(version);
        self.rules.push(rule);
    }

    pub fn get_rule(&self, rule_id: &str) -> Option<&SecurityScanRule> {
        self.rules.iter().find(|r| r.id == rule_id)
    }

    pub fn get_rule_mut(&mut self, rule_id: &str) -> Option<&mut SecurityScanRule> {
        self.rules.iter_mut().find(|r| r.id == rule_id)
    }

    pub fn list_rules(&self) -> &[SecurityScanRule] {
        &self.rules
    }

    pub fn list_enabled_rules(&self) -> Vec<&SecurityScanRule> {
        self.rules.iter().filter(|r| r.enabled).collect()
    }

    pub fn list_rules_by_type(&self, rule_type: RuleType) -> Vec<&SecurityScanRule> {
        self.rules.iter().filter(|r| r.rule_type == rule_type).collect()
    }

    pub fn list_rules_by_severity(&self, severity: RuleSeverity) -> Vec<&SecurityScanRule> {
        self.rules.iter().filter(|r| r.severity == severity).collect()
    }

    pub fn update_rule_pattern(&mut self, rule_id: &str, new_pattern: String, changed_by: Option<String>) -> Result<(), String> {
        let (version_num, version_entry) = {
            let rule = self.get_rule_mut(rule_id).ok_or("Rule not found")?;
            let version_num = rule.version + 1;
            rule.update_pattern(new_pattern.clone());
            (version_num, RuleVersion {
                version: version_num,
                pattern: new_pattern,
                changed_by: changed_by.clone(),
                changed_at: Utc::now(),
                change_notes: None,
            })
        };
        self.versions
            .entry(rule_id.to_string())
            .or_default()
            .push(version_entry);
        Ok(())
    }

    pub fn get_rule_versions(&self, rule_id: &str) -> Option<&Vec<RuleVersion>> {
        self.versions.get(rule_id)
    }

    pub fn test_rule(&self, rule: &SecurityScanRule, input: &str) -> RuleTestResult {
        let start = std::time::Instant::now();
        let mut matched_lines = Vec::new();

        for (idx, line) in input.lines().enumerate() {
            let matched = match rule.rule_type {
                RuleType::Regex => {
                    regex::Regex::new(&rule.pattern)
                        .map(|re| re.is_match(line))
                        .unwrap_or(false)
                }
                RuleType::Keyword => line.contains(&rule.pattern),
                RuleType::Pattern => {
                    line.to_lowercase().contains(&rule.pattern.to_lowercase())
                }
                RuleType::Ast | RuleType::Custom => false,
            };
            if matched {
                matched_lines.push((idx + 1) as u32);
            }
        }

        RuleTestResult {
            rule_id: rule.id.clone(),
            input: input.to_string(),
            matched: !matched_lines.is_empty(),
            matched_lines,
            execution_time_ms: start.elapsed().as_millis() as u64,
        }
    }
}

impl Default for ScanRuleManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_default_scan_rules() -> Vec<SecurityScanRule> {
    vec![
        SecurityScanRule::new(
            "Hardcoded Secret".into(),
            "Detects hardcoded secrets and API keys".into(),
            RuleType::Regex,
            r#"(?i)(api[_-]?key|secret|password|token)\s*[:=]\s*['"][^'"]+['"]"#.into(),
        )
        .with_severity(RuleSeverity::Critical),
        SecurityScanRule::new(
            "SQL Injection".into(),
            "Detects potential SQL injection patterns".into(),
            RuleType::Regex,
            r#"(?i)(SELECT|INSERT|UPDATE|DELETE|DROP).*['"].*\+.*['"]"#.into(),
        )
        .with_severity(RuleSeverity::High),
        SecurityScanRule::new(
            "XSS Vulnerability".into(),
            "Detects potential cross-site scripting patterns".into(),
            RuleType::Regex,
            r#"<script[^>]*>.*</script>"#.into(),
        )
        .with_severity(RuleSeverity::High),
    ]
}

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

    #[test]
    fn test_security_scan_rule_new() {
        let rule = SecurityScanRule::new(
            "Test Rule".into(),
            "A test rule".into(),
            RuleType::Regex,
            r#"test_pattern"#.into(),
        );
        assert_eq!(rule.name, "Test Rule");
        assert_eq!(rule.rule_type, RuleType::Regex);
        assert_eq!(rule.severity, RuleSeverity::Medium);
        assert!(rule.enabled);
        assert_eq!(rule.version, 1);
    }

    #[test]
    fn test_security_scan_rule_with_severity() {
        let rule = SecurityScanRule::new(
            "Critical Rule".into(),
            "Critical".into(),
            RuleType::Keyword,
            "secret".into(),
        )
        .with_severity(RuleSeverity::Critical);
        assert_eq!(rule.severity, RuleSeverity::Critical);
    }

    #[test]
    fn test_security_scan_rule_disable_enable() {
        let mut rule = SecurityScanRule::new(
            "Test".into(),
            "Desc".into(),
            RuleType::Regex,
            "pattern".into(),
        );
        rule.disable();
        assert!(!rule.enabled);
        rule.enable();
        assert!(rule.enabled);
    }

    #[test]
    fn test_security_scan_rule_update_pattern() {
        let mut rule = SecurityScanRule::new(
            "Test".into(),
            "Desc".into(),
            RuleType::Regex,
            "old".into(),
        );
        rule.update_pattern("new".into());
        assert_eq!(rule.pattern, "new");
        assert_eq!(rule.version, 2);
    }

    #[test]
    fn test_scan_rule_manager_add_and_get() {
        let mut manager = ScanRuleManager::new();
        let rule = SecurityScanRule::new(
            "Rule1".into(),
            "Desc".into(),
            RuleType::Regex,
            "pattern".into(),
        );
        let id = rule.id.clone();
        manager.add_rule(rule);
        assert!(manager.get_rule(&id).is_some());
        assert!(manager.get_rule("nonexistent").is_none());
    }

    #[test]
    fn test_scan_rule_manager_list_enabled() {
        let mut manager = ScanRuleManager::new();
        let mut rule1 = SecurityScanRule::new(
            "Rule1".into(),
            "Desc".into(),
            RuleType::Regex,
            "pattern".into(),
        );
        rule1.disable();
        manager.add_rule(rule1);
        manager.add_rule(SecurityScanRule::new(
            "Rule2".into(),
            "Desc".into(),
            RuleType::Regex,
            "pattern".into(),
        ));
        assert_eq!(manager.list_enabled_rules().len(), 1);
    }

    #[test]
    fn test_scan_rule_manager_list_by_type() {
        let mut manager = ScanRuleManager::new();
        manager.add_rule(SecurityScanRule::new(
            "R1".into(),
            "".into(),
            RuleType::Regex,
            "p".into(),
        ));
        manager.add_rule(SecurityScanRule::new(
            "R2".into(),
            "".into(),
            RuleType::Keyword,
            "p".into(),
        ));
        assert_eq!(manager.list_rules_by_type(RuleType::Regex).len(), 1);
        assert_eq!(manager.list_rules_by_type(RuleType::Keyword).len(), 1);
    }

    #[test]
    fn test_scan_rule_manager_update_pattern() {
        let mut manager = ScanRuleManager::new();
        let rule = SecurityScanRule::new(
            "Rule1".into(),
            "Desc".into(),
            RuleType::Regex,
            "old".into(),
        );
        let id = rule.id.clone();
        manager.add_rule(rule);
        manager.update_rule_pattern(&id, "new".into(), Some("user1".into())).unwrap();
        let updated = manager.get_rule(&id).unwrap();
        assert_eq!(updated.pattern, "new");
        assert_eq!(updated.version, 2);
        let versions = manager.get_rule_versions(&id).unwrap();
        assert_eq!(versions.len(), 2);
    }

    #[test]
    fn test_scan_rule_manager_test_rule_keyword() {
        let manager = ScanRuleManager::new();
        let rule = SecurityScanRule::new(
            "Keyword Rule".into(),
            "Desc".into(),
            RuleType::Keyword,
            "password".into(),
        );
        let result = manager.test_rule(&rule, "line with password here\nsafe line");
        assert!(result.matched);
        assert_eq!(result.matched_lines, vec![1]);
    }

    #[test]
    fn test_scan_rule_manager_test_rule_no_match() {
        let manager = ScanRuleManager::new();
        let rule = SecurityScanRule::new(
            "Keyword Rule".into(),
            "Desc".into(),
            RuleType::Keyword,
            "password".into(),
        );
        let result = manager.test_rule(&rule, "safe line\nanother safe line");
        assert!(!result.matched);
        assert!(result.matched_lines.is_empty());
    }

    #[test]
    fn test_create_default_scan_rules() {
        let rules = create_default_scan_rules();
        assert_eq!(rules.len(), 3);
        assert!(rules.iter().any(|r| r.severity == RuleSeverity::Critical));
        assert!(rules.iter().all(|r| r.enabled));
    }

    #[test]
    fn test_rule_versioning() {
        let mut manager = ScanRuleManager::new();
        let rule = SecurityScanRule::new(
            "Rule1".into(),
            "Desc".into(),
            RuleType::Regex,
            "v1".into(),
        );
        let id = rule.id.clone();
        manager.add_rule(rule);
        manager.update_rule_pattern(&id, "v2".into(), None).unwrap();
        manager.update_rule_pattern(&id, "v3".into(), None).unwrap();
        let versions = manager.get_rule_versions(&id).unwrap();
        assert_eq!(versions.len(), 3);
        assert_eq!(versions[0].version, 1);
        assert_eq!(versions[2].version, 3);
    }

    #[test]
    fn test_rule_type_serialization() {
        assert_eq!(serde_json::to_string(&RuleType::Regex).unwrap(), "\"regex\"");
        assert_eq!(serde_json::to_string(&RuleType::Keyword).unwrap(), "\"keyword\"");
    }

    #[test]
    fn test_rule_severity_serialization() {
        assert_eq!(
            serde_json::to_string(&RuleSeverity::Critical).unwrap(),
            "\"critical\""
        );
        assert_eq!(
            serde_json::to_string(&RuleSeverity::Low).unwrap(),
            "\"low\""
        );
    }
}
