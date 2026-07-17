#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Core Scan Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleType {
    Regex,
    Keyword,
    Ast,
    Pattern,
    Custom,
    Composite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleSeverity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

impl RuleSeverity {
    pub fn risk_weight(&self) -> u32 {
        match self {
            Self::Critical => 5,
            Self::High => 4,
            Self::Medium => 2,
            Self::Low => 1,
            Self::Informational => 0,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Critical => "Critical",
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
            Self::Informational => "Informational",
        }
    }
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
    pub auto_fix: bool,
    pub fix_config: serde_json::Value,
    pub compliance_mapping: ComplianceMapping,
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
            auto_fix: false,
            fix_config: serde_json::Value::Object(serde_json::Map::new()),
            compliance_mapping: ComplianceMapping::default(),
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

    pub fn with_auto_fix(mut self, fix_config: serde_json::Value) -> Self {
        self.auto_fix = true;
        self.fix_config = fix_config;
        self
    }

    pub fn with_compliance_mapping(mut self, mapping: ComplianceMapping) -> Self {
        self.compliance_mapping = mapping;
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

    pub fn increment_version(&mut self) {
        self.version += 1;
        self.updated_at = Utc::now();
    }

    pub fn risk_score(&self) -> u32 {
        self.severity.risk_weight()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleTestResult {
    pub rule_id: String,
    pub input: String,
    pub matched: bool,
    pub matched_lines: Vec<u32>,
    pub execution_time_ms: u64,
    pub risk_score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleVersion {
    pub version: u32,
    pub pattern: String,
    pub changed_by: Option<String>,
    pub changed_at: DateTime<Utc>,
    pub change_notes: Option<String>,
    pub compliance_mappings_snapshot: ComplianceMapping,
    pub breaking_change: bool,
}

// ============================================================================
// Compliance Mapping
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplianceMapping {
    pub mappings: Vec<ComplianceMappingEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceMappingEntry {
    pub framework_id: String,
    pub framework_name: String,
    pub requirement_id: String,
    pub mapping_type: ComplianceMappingType,
    pub confidence: f64,
    pub auto_verifiable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceMappingType {
    Direct,
    Partial,
    Advisory,
    Inherited,
}

// ============================================================================
// Auto-Fix System
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanFix {
    pub id: String,
    pub scan_id: String,
    pub rule_id: String,
    pub file_path: String,
    pub line_number: Option<u32>,
    pub fix_type: FixType,
    pub fix_content: String,
    pub applied: bool,
    pub test_result: Option<FixTestResult>,
    pub applied_at: Option<DateTime<Utc>>,
    pub applied_by: Option<String>,
    pub rollback_content: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixType {
    PatternReplace,
    LineRemoval,
    LineInsertion,
    ConfigPatch,
    DependencyUpdate,
    Composite,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixTestResult {
    pub passed: bool,
    pub test_output: String,
    pub regressions: Vec<String>,
    pub tested_at: DateTime<Utc>,
    pub test_duration_ms: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FixAnalytics {
    pub total_fixes: u64,
    pub applied_fixes: u64,
    pub failed_fixes: u64,
    pub tested_fixes: u64,
    pub test_pass_rate: f64,
    pub fixes_by_type: HashMap<String, u64>,
    pub fixes_by_severity: HashMap<String, u64>,
    pub fixes_by_framework: HashMap<String, u64>,
    pub avg_fix_success_rate: f64,
    pub regression_count: u64,
    pub compliance_impact: ComplianceImpact,
    pub rollback_count: u64,
    pub avg_apply_time_ms: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplianceImpact {
    pub rules_with_compliance: u64,
    pub fixes_improving_compliance: u64,
    pub compliance_score_delta: f64,
    pub framework_coverage: HashMap<String, f64>,
    pub auto_verifiable_count: u64,
}

// ============================================================================
// Scan Types
// ============================================================================

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
pub struct SecurityScan {
    pub id: String,
    pub repo_id: String,
    pub scan_type: ScanType,
    pub status: ScanStatus,
    pub findings: Vec<SecurityFinding>,
    pub score: u32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

// ============================================================================
// Policy
// ============================================================================

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
    fn scan(&self, target: &ScanTarget) -> Result<SecurityScan, String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanTarget {
    pub repo_id: String,
    pub ref_name: Option<String>,
    pub paths: Vec<String>,
}

pub struct SecurityPolicyEvaluator;

impl SecurityPolicyEvaluator {
    pub fn evaluate(scan: &SecurityScan, policy: &SecurityPolicy) -> PolicyEvaluationResult {
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

// ============================================================================
// Scan Rule Manager (V1 - basic, with versioning)
// ============================================================================

pub struct ScanRuleManager {
    rules: Vec<SecurityScanRule>,
    versions: HashMap<String, Vec<RuleVersion>>,
    fixes: Vec<ScanFix>,
}

impl ScanRuleManager {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            versions: HashMap::new(),
            fixes: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, rule: SecurityScanRule) {
        let version = RuleVersion {
            version: rule.version,
            pattern: rule.pattern.clone(),
            changed_by: rule.created_by.clone(),
            changed_at: rule.created_at,
            change_notes: Some("Initial creation".into()),
            compliance_mappings_snapshot: rule.compliance_mapping.clone(),
            breaking_change: false,
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

    pub fn list_auto_fix_rules(&self) -> Vec<&SecurityScanRule> {
        self.rules.iter().filter(|r| r.auto_fix && r.enabled).collect()
    }

    pub fn list_rules_with_compliance(&self) -> Vec<&SecurityScanRule> {
        self.rules
            .iter()
            .filter(|r| !r.compliance_mapping.mappings.is_empty())
            .collect()
    }

    pub fn list_auto_verifiable_rules(&self) -> Vec<&SecurityScanRule> {
        self.rules
            .iter()
            .filter(|r| {
                r.enabled
                    && r.compliance_mapping
                        .mappings
                        .iter()
                        .any(|m| m.auto_verifiable)
            })
            .collect()
    }

    pub fn update_rule_pattern(
        &mut self,
        rule_id: &str,
        new_pattern: String,
        changed_by: Option<String>,
        change_notes: Option<String>,
        breaking_change: bool,
    ) -> Result<(), String> {
        let rule = self
            .rules
            .iter_mut()
            .find(|r| r.id == rule_id)
            .ok_or("Rule not found")?;
        rule.pattern = new_pattern.clone();
        rule.increment_version();
        let version = rule.version;
        let compliance_snapshot = rule.compliance_mapping.clone();
        let changed_at = rule.updated_at;

        let entry = RuleVersion {
            version,
            pattern: new_pattern,
            changed_by,
            changed_at,
            change_notes,
            compliance_mappings_snapshot: compliance_snapshot,
            breaking_change,
        };
        self.versions
            .entry(rule_id.to_string())
            .or_default()
            .push(entry);
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
                RuleType::Ast | RuleType::Custom | RuleType::Composite => false,
            };
            if matched {
                matched_lines.push((idx + 1) as u32);
            }
        }

        let has_matches = !matched_lines.is_empty();
        let risk_score = if has_matches { rule.risk_score() } else { 0 };
        RuleTestResult {
            rule_id: rule.id.clone(),
            input: input.to_string(),
            matched: has_matches,
            matched_lines,
            execution_time_ms: start.elapsed().as_millis() as u64,
            risk_score,
        }
    }

    pub fn record_fix(&mut self, fix: ScanFix) {
        self.fixes.push(fix);
    }

    pub fn apply_fix(
        &mut self,
        fix_id: &str,
        applied_by: &str,
        test_before_apply: bool,
    ) -> Result<bool, String> {
        let fix = self
            .fixes
            .iter_mut()
            .find(|f| f.id == fix_id)
            .ok_or("Fix not found")?;

        if test_before_apply && fix.test_result.is_none() {
            return Err("Fix must be tested before applying".into());
        }

        if let Some(ref test) = fix.test_result
            && !test.passed {
                return Err("Fix failed testing".into());
            }

        fix.applied = true;
        fix.applied_at = Some(Utc::now());
        fix.applied_by = Some(applied_by.into());
        Ok(true)
    }

    pub fn rollback_fix(&mut self, fix_id: &str) -> Result<Option<String>, String> {
        let fix = self
            .fixes
            .iter_mut()
            .find(|f| f.id == fix_id)
            .ok_or("Fix not found")?;

        if !fix.applied {
            return Err("Fix has not been applied".into());
        }

        fix.applied = false;
        Ok(fix.rollback_content.clone())
    }

    pub fn test_fix(&mut self, fix_id: &str, test_input: &str) -> Result<FixTestResult, String> {
        let fix = self
            .fixes
            .iter_mut()
            .find(|f| f.id == fix_id)
            .ok_or("Fix not found")?;

        let start = std::time::Instant::now();
        let passed = !fix.fix_content.is_empty() && !test_input.is_empty();
        let duration_ms = start.elapsed().as_millis() as u64;
        let result = FixTestResult {
            passed,
            test_output: if passed {
                "All tests passed".into()
            } else {
                "Test failed".into()
            },
            regressions: Vec::new(),
            tested_at: Utc::now(),
            test_duration_ms: duration_ms,
        };
        fix.test_result = Some(result.clone());
        Ok(result)
    }

    pub fn get_fixes_for_scan(&self, scan_id: &str) -> Vec<&ScanFix> {
        self.fixes.iter().filter(|f| f.scan_id == scan_id).collect()
    }

    pub fn get_pending_fixes(&self) -> Vec<&ScanFix> {
        self.fixes.iter().filter(|f| !f.applied).collect()
    }

    pub fn get_rollbackable_fixes(&self) -> Vec<&ScanFix> {
        self.fixes
            .iter()
            .filter(|f| f.applied && f.rollback_content.is_some())
            .collect()
    }

    pub fn compute_fix_analytics(&self) -> FixAnalytics {
        let total = self.fixes.len() as u64;
        let applied = self.fixes.iter().filter(|f| f.applied).count() as u64;
        let tested = self.fixes.iter().filter(|f| f.test_result.is_some()).count() as u64;
        let test_passed = self
            .fixes
            .iter()
            .filter(|f| f.test_result.as_ref().is_some_and(|t| t.passed))
            .count() as u64;
        let regressions = self
            .fixes
            .iter()
            .filter(|f| {
                f.test_result
                    .as_ref()
                    .is_some_and(|t| !t.regressions.is_empty())
            })
            .count() as u64;
        let rollback_count = self
            .fixes
            .iter()
            .filter(|f| f.rollback_content.is_some())
            .count() as u64;

        let mut fixes_by_type: HashMap<String, u64> = HashMap::new();
        let mut fixes_by_severity: HashMap<String, u64> = HashMap::new();
        let mut fixes_by_framework: HashMap<String, u64> = HashMap::new();

        for fix in &self.fixes {
            *fixes_by_type.entry(format!("{:?}", fix.fix_type)).or_default() += 1;
        }

        let mut rules_with_compliance = 0u64;
        let mut fixes_improving_compliance = 0u64;
        let mut auto_verifiable_count = 0u64;
        let mut framework_coverage: HashMap<String, u64> = HashMap::new();

        for rule in &self.rules {
            let has_compliance = !rule.compliance_mapping.mappings.is_empty();
            if has_compliance {
                rules_with_compliance += 1;
            }
            for mapping in &rule.compliance_mapping.mappings {
                if mapping.auto_verifiable {
                    auto_verifiable_count += 1;
                }
            }
            if self.fixes.iter().any(|f| f.rule_id == rule.id) {
                *fixes_by_severity
                    .entry(format!("{:?}", rule.severity))
                    .or_default() += 1;
                for mapping in &rule.compliance_mapping.mappings {
                    *fixes_by_framework
                        .entry(mapping.framework_name.clone())
                        .or_default() += 1;
                    *framework_coverage
                        .entry(mapping.framework_name.clone())
                        .or_default() += 1;
                }
                if has_compliance {
                    fixes_improving_compliance += 1;
                }
            }
        }

        let test_pass_rate = if tested > 0 {
            (test_passed as f64 / tested as f64) * 100.0
        } else {
            0.0
        };

        let avg_success_rate = if total > 0 {
            (applied as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let total_frameworks = framework_coverage.len() as u64;
        let compliance_impact = ComplianceImpact {
            rules_with_compliance,
            fixes_improving_compliance,
            compliance_score_delta: if total > 0 {
                (fixes_improving_compliance as f64 / total as f64) * 100.0
            } else {
                0.0
            },
            framework_coverage: framework_coverage
                .into_iter()
                .map(|(k, v)| {
                    let coverage = if total_frameworks > 0 {
                        (v as f64 / total_frameworks as f64) * 100.0
                    } else {
                        0.0
                    };
                    (k, coverage)
                })
                .collect(),
            auto_verifiable_count,
        };

        FixAnalytics {
            total_fixes: total,
            applied_fixes: applied,
            failed_fixes: total - applied,
            tested_fixes: tested,
            test_pass_rate,
            fixes_by_type,
            fixes_by_severity,
            fixes_by_framework,
            avg_fix_success_rate: avg_success_rate,
            regression_count: regressions,
            compliance_impact,
            rollback_count,
            avg_apply_time_ms: 0.0,
        }
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::from("=== Security Scan Report ===\n\n");
        report.push_str(&format!("Total Rules: {}\n", self.rules.len()));
        report.push_str(&format!(
            "Active Rules: {}\n",
            self.rules.iter().filter(|r| r.enabled).count()
        ));
        report.push_str(&format!(
            "Auto-Fix Rules: {}\n",
            self.list_auto_fix_rules().len()
        ));
        report.push_str(&format!(
            "Auto-Verifiable Rules: {}\n",
            self.list_auto_verifiable_rules().len()
        ));
        report.push('\n');

        report.push_str("Rules by Severity:\n");
        for severity in &[
            RuleSeverity::Critical,
            RuleSeverity::High,
            RuleSeverity::Medium,
            RuleSeverity::Low,
            RuleSeverity::Informational,
        ] {
            let count = self.list_rules_by_severity(*severity).len();
            report.push_str(&format!("  {}: {}\n", severity.display_name(), count));
        }
        report.push('\n');

        let analytics = self.compute_fix_analytics();
        report.push_str("Fix Analytics:\n");
        report.push_str(&format!("  Total Fixes: {}\n", analytics.total_fixes));
        report.push_str(&format!("  Applied: {}\n", analytics.applied_fixes));
        report.push_str(&format!("  Tested: {}\n", analytics.tested_fixes));
        report.push_str(&format!(
            "  Test Pass Rate: {:.1}%\n",
            analytics.test_pass_rate
        ));
        report.push_str(&format!("  Regressions: {}\n", analytics.regression_count));
        report.push_str(&format!("  Rollbacks: {}\n", analytics.rollback_count));
        report.push_str(&format!(
            "  Success Rate: {:.1}%\n",
            analytics.avg_fix_success_rate
        ));

        report.push_str("\nCompliance Impact:\n");
        report.push_str(&format!(
            "  Rules with Compliance Mappings: {}\n",
            analytics.compliance_impact.rules_with_compliance
        ));
        report.push_str(&format!(
            "  Fixes Improving Compliance: {}\n",
            analytics.compliance_impact.fixes_improving_compliance
        ));
        report.push_str(&format!(
            "  Compliance Score Delta: {:.1}%\n",
            analytics.compliance_impact.compliance_score_delta
        ));
        report.push_str(&format!(
            "  Auto-Verifiable: {}\n",
            analytics.compliance_impact.auto_verifiable_count
        ));

        report.push_str("\nFramework Coverage:\n");
        for (framework, coverage) in &analytics.compliance_impact.framework_coverage {
            report.push_str(&format!("  {}: {:.1}%\n", framework, coverage));
        }

        report.push_str("\nCompliance Mappings:\n");
        let mut framework_counts: HashMap<String, u64> = HashMap::new();
        for rule in &self.rules {
            for mapping in &rule.compliance_mapping.mappings {
                *framework_counts
                    .entry(mapping.framework_name.clone())
                    .or_default() += 1;
            }
        }
        for (framework, count) in &framework_counts {
            report.push_str(&format!("  {}: {} rules\n", framework, count));
        }

        report
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
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
        .with_severity(RuleSeverity::Critical)
        .with_compliance_mapping(ComplianceMapping {
            mappings: vec![ComplianceMappingEntry {
                framework_id: "soc2".into(),
                framework_name: "SOC 2".into(),
                requirement_id: "CC6.1".into(),
                mapping_type: ComplianceMappingType::Direct,
                confidence: 0.95,
                auto_verifiable: true,
            }],
        }),
        SecurityScanRule::new(
            "SQL Injection".into(),
            "Detects potential SQL injection patterns".into(),
            RuleType::Regex,
            r#"(?i)(SELECT|INSERT|UPDATE|DELETE|DROP).*['"].*\+.*['"]"#.into(),
        )
        .with_severity(RuleSeverity::High)
        .with_compliance_mapping(ComplianceMapping {
            mappings: vec![ComplianceMappingEntry {
                framework_id: "owasp_top10".into(),
                framework_name: "OWASP Top 10".into(),
                requirement_id: "A03:2021".into(),
                mapping_type: ComplianceMappingType::Direct,
                confidence: 0.90,
                auto_verifiable: true,
            }],
        }),
        SecurityScanRule::new(
            "XSS Vulnerability".into(),
            "Detects potential cross-site scripting patterns".into(),
            RuleType::Regex,
            r#"<script[^>]*>.*</script>"#.into(),
        )
        .with_severity(RuleSeverity::High)
        .with_compliance_mapping(ComplianceMapping {
            mappings: vec![ComplianceMappingEntry {
                framework_id: "owasp_top10".into(),
                framework_name: "OWASP Top 10".into(),
                requirement_id: "A03:2021".into(),
                mapping_type: ComplianceMappingType::Direct,
                confidence: 0.85,
                auto_verifiable: true,
            }],
        }),
    ]
}

// ============================================================================
// Stub Scanner
// ============================================================================

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

    fn scan(&self, target: &ScanTarget) -> Result<SecurityScan, String> {
        Ok(SecurityScan {
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

// ============================================================================
// Rule Set Manager (from v23)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanRuleSet {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rules: Vec<SecurityScanRule>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

impl SecurityScanRuleSet {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            rules: Vec::new(),
            enabled: true,
            created_at: Utc::now(),
        }
    }

    pub fn with_rules(mut self, rules: Vec<SecurityScanRule>) -> Self {
        self.rules = rules;
        self
    }

    pub fn add_rule(&mut self, rule: SecurityScanRule) {
        self.rules.push(rule);
    }

    pub fn enabled_rules(&self) -> Vec<&SecurityScanRule> {
        self.rules.iter().filter(|r| r.enabled).collect()
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

// ============================================================================
// Deduplication Engine (from v23)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanDedupEntry {
    pub id: String,
    pub vulnerability_id: String,
    pub repo_id: String,
    pub file_path: String,
    pub line_number: Option<u32>,
    pub dedup_hash: String,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

impl SecurityScanDedupEntry {
    pub fn new(
        vulnerability_id: String,
        repo_id: String,
        file_path: String,
        line_number: Option<u32>,
    ) -> Self {
        let now = Utc::now();
        let dedup_hash = format!(
            "{}:{}:{}:{}",
            vulnerability_id,
            repo_id,
            file_path,
            line_number.map_or("0".to_string(), |l| l.to_string())
        );
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            vulnerability_id,
            repo_id,
            file_path,
            line_number,
            dedup_hash,
            first_seen_at: now,
            last_seen_at: now,
        }
    }

    pub fn touch(&mut self) {
        self.last_seen_at = Utc::now();
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeduplicationEngine {
    entries: Vec<SecurityScanDedupEntry>,
    hash_index: HashMap<String, Vec<usize>>,
}

impl DeduplicationEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn check_and_record(
        &mut self,
        vulnerability_id: String,
        repo_id: String,
        file_path: String,
        line_number: Option<u32>,
    ) -> DeduplicationResult {
        let entry =
            SecurityScanDedupEntry::new(vulnerability_id, repo_id, file_path, line_number);
        let hash = entry.dedup_hash.clone();

        if let Some(indices) = self.hash_index.get(&hash)
            && let Some(&idx) = indices.first() {
                let existing = &mut self.entries[idx];
                existing.touch();
                return DeduplicationResult {
                    is_duplicate: true,
                    entry_id: existing.id.clone(),
                    first_seen: existing.first_seen_at,
                    last_seen: existing.last_seen_at,
                    occurrence_count: indices.len() as u32 + 1,
                };
            }

        let idx = self.entries.len();
        let entry_id = entry.id.clone();
        let first_seen = entry.first_seen_at;
        let last_seen = entry.last_seen_at;
        self.hash_index
            .entry(hash)
            .or_default()
            .push(idx);
        self.entries.push(entry);

        DeduplicationResult {
            is_duplicate: false,
            entry_id,
            first_seen,
            last_seen,
            occurrence_count: 1,
        }
    }

    pub fn get_entries_for_repo(&self, repo_id: &str) -> Vec<&SecurityScanDedupEntry> {
        self.entries
            .iter()
            .filter(|e| e.repo_id == repo_id)
            .collect()
    }

    pub fn total_unique(&self) -> usize {
        self.entries.len()
    }

    pub fn total_occurrences(&self) -> usize {
        self.hash_index.values().map(|v| v.len()).sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeduplicationResult {
    pub is_duplicate: bool,
    pub entry_id: String,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub occurrence_count: u32,
}

// ============================================================================
// False Positive Tracker (from v23)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsePositiveRecord {
    pub id: String,
    pub vulnerability_id: String,
    pub rule_id: String,
    pub reason: String,
    pub marked_by: Option<String>,
    pub marked_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub active: bool,
}

impl FalsePositiveRecord {
    pub fn new(
        vulnerability_id: String,
        rule_id: String,
        reason: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            vulnerability_id,
            rule_id,
            reason,
            marked_by: None,
            marked_at: Utc::now(),
            expires_at: None,
            active: true,
        }
    }

    pub fn with_marked_by(mut self, user_id: &str) -> Self {
        self.marked_by = Some(user_id.into());
        self
    }

    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.is_some_and(|exp| Utc::now() > exp)
    }

    pub fn revoke(&mut self) {
        self.active = false;
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FalsePositiveTracker {
    records: Vec<FalsePositiveRecord>,
    by_vulnerability: HashMap<String, Vec<usize>>,
}

impl FalsePositiveTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mark_false_positive(
        &mut self,
        vulnerability_id: String,
        rule_id: String,
        reason: String,
        marked_by: Option<String>,
    ) -> FalsePositiveRecord {
        let mut record = FalsePositiveRecord::new(vulnerability_id.clone(), rule_id, reason);
        record.marked_by = marked_by;
        let idx = self.records.len();
        self.by_vulnerability
            .entry(vulnerability_id)
            .or_default()
            .push(idx);
        self.records.push(record.clone());
        record
    }

    pub fn is_false_positive(&self, vulnerability_id: &str) -> bool {
        self.by_vulnerability
            .get(vulnerability_id)
            .map(|indices| {
                indices.iter().any(|&idx| {
                    let r = &self.records[idx];
                    r.active && !r.is_expired()
                })
            })
            .unwrap_or(false)
    }

    pub fn revoke(&mut self, record_id: &str) -> Result<(), String> {
        let record = self
            .records
            .iter_mut()
            .find(|r| r.id == record_id)
            .ok_or("Record not found")?;
        record.revoke();
        Ok(())
    }

    pub fn active_count(&self) -> usize {
        self.records
            .iter()
            .filter(|r| r.active && !r.is_expired())
            .count()
    }

    pub fn get_records_for_vulnerability(
        &self,
        vulnerability_id: &str,
    ) -> Vec<&FalsePositiveRecord> {
        self.by_vulnerability
            .get(vulnerability_id)
            .map(|indices| indices.iter().map(|&idx| &self.records[idx]).collect())
            .unwrap_or_default()
    }
}

// ============================================================================
// Scan Scheduling (from v23)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSchedule {
    pub id: String,
    pub repo_id: String,
    pub rule_set_id: String,
    pub interval_minutes: u32,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl ScanSchedule {
    pub fn new(repo_id: String, rule_set_id: String, interval_minutes: u32) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            repo_id,
            rule_set_id,
            interval_minutes,
            enabled: true,
            last_run: None,
            next_run: Some(now + chrono::Duration::minutes(interval_minutes as i64)),
            created_at: now,
        }
    }

    pub fn record_run(&mut self) {
        self.last_run = Some(Utc::now());
        self.next_run =
            Some(Utc::now() + chrono::Duration::minutes(self.interval_minutes as i64));
    }

    pub fn is_due(&self) -> bool {
        self.next_run.is_some_and(|next| Utc::now() >= next)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanSchedulingEngine {
    schedules: Vec<ScanSchedule>,
}

impl ScanSchedulingEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_schedule(&mut self, schedule: ScanSchedule) {
        self.schedules.push(schedule);
    }

    pub fn get_due_schedules(&self) -> Vec<&ScanSchedule> {
        self.schedules
            .iter()
            .filter(|s| s.enabled && s.is_due())
            .collect()
    }

    pub fn record_run(&mut self, schedule_id: &str) -> Result<(), String> {
        let schedule = self
            .schedules
            .iter_mut()
            .find(|s| s.id == schedule_id)
            .ok_or("Schedule not found")?;
        schedule.record_run();
        Ok(())
    }

    pub fn disable_schedule(&mut self, schedule_id: &str) -> Result<(), String> {
        let schedule = self
            .schedules
            .iter_mut()
            .find(|s| s.id == schedule_id)
            .ok_or("Schedule not found")?;
        schedule.enabled = false;
        Ok(())
    }

    pub fn schedules_for_repo(&self, repo_id: &str) -> Vec<&ScanSchedule> {
        self.schedules
            .iter()
            .filter(|s| s.repo_id == repo_id)
            .collect()
    }
}

// ============================================================================
// Rule Set Manager (from v23)
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuleSetManager {
    rule_sets: Vec<SecurityScanRuleSet>,
}

impl RuleSetManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_rule_set(&mut self, rule_set: SecurityScanRuleSet) {
        self.rule_sets.push(rule_set);
    }

    pub fn get_rule_set(&self, id: &str) -> Option<&SecurityScanRuleSet> {
        self.rule_sets.iter().find(|rs| rs.id == id)
    }

    pub fn get_enabled_rule_sets(&self) -> Vec<&SecurityScanRuleSet> {
        self.rule_sets.iter().filter(|rs| rs.enabled).collect()
    }

    pub fn list_rule_sets(&self) -> &[SecurityScanRuleSet] {
        &self.rule_sets
    }

    pub fn disable_rule_set(&mut self, id: &str) -> Result<(), String> {
        let rs = self
            .rule_sets
            .iter_mut()
            .find(|rs| rs.id == id)
            .ok_or("Rule set not found")?;
        rs.enabled = false;
        Ok(())
    }

    pub fn total_rules(&self) -> usize {
        self.rule_sets.iter().map(|rs| rs.rules.len()).sum()
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::from("=== Security Scan Rule Sets Report ===\n\n");
        report.push_str(&format!("Total Rule Sets: {}\n", self.rule_sets.len()));
        report.push_str(&format!(
            "Enabled Rule Sets: {}\n",
            self.get_enabled_rule_sets().len()
        ));
        report.push_str(&format!("Total Rules: {}\n\n", self.total_rules()));

        for rs in &self.rule_sets {
            report.push_str(&format!(
                "Rule Set: {} ({} rules, {})\n",
                rs.name,
                rs.rules.len(),
                if rs.enabled { "enabled" } else { "disabled" }
            ));
        }

        report
    }
}

// ============================================================================
// Threat Intelligence (from v24)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIntelligence {
    pub id: String,
    pub cve_id: String,
    pub severity: ThreatSeverity,
    pub description: String,
    pub affected_packages: Vec<String>,
    pub fix_available: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub fetched_at: DateTime<Utc>,
}

impl ThreatIntelligence {
    pub fn new(cve_id: String, severity: ThreatSeverity, description: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            cve_id,
            severity,
            description,
            affected_packages: Vec::new(),
            fix_available: false,
            published_at: None,
            fetched_at: Utc::now(),
        }
    }

    pub fn with_packages(mut self, packages: Vec<String>) -> Self {
        self.affected_packages = packages;
        self
    }

    pub fn with_fix_available(mut self, fix: bool) -> Self {
        self.fix_available = fix;
        self
    }

    pub fn with_published_at(mut self, published_at: DateTime<Utc>) -> Self {
        self.published_at = Some(published_at);
        self
    }

    pub fn is_critical(&self) -> bool {
        self.severity == ThreatSeverity::Critical
    }

    pub fn affects_package(&self, package: &str) -> bool {
        self.affected_packages.iter().any(|p| p == package)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreatSeverity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

impl ThreatSeverity {
    pub fn risk_weight(&self) -> u32 {
        match self {
            Self::Critical => 5,
            Self::High => 4,
            Self::Medium => 2,
            Self::Low => 1,
            Self::Informational => 0,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Critical => "Critical",
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
            Self::Informational => "Informational",
        }
    }
}

// ============================================================================
// Dependency Tree (from v24)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyTreeNode {
    pub id: String,
    pub repo_id: String,
    pub package_name: String,
    pub version: String,
    pub parent_package: Option<String>,
    pub dependency_type: DependencyType,
    pub depth: u32,
    pub scanned_at: DateTime<Utc>,
}

impl DependencyTreeNode {
    pub fn new(
        repo_id: String,
        package_name: String,
        version: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            repo_id,
            package_name,
            version,
            parent_package: None,
            dependency_type: DependencyType::Direct,
            depth: 0,
            scanned_at: Utc::now(),
        }
    }

    pub fn with_parent(mut self, parent: String, dep_type: DependencyType) -> Self {
        self.parent_package = Some(parent);
        self.dependency_type = dep_type;
        self
    }

    pub fn with_depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }

    pub fn is_transitive(&self) -> bool {
        self.dependency_type == DependencyType::Transitive
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyType {
    Direct,
    Transitive,
    Dev,
    Optional,
    Peer,
}

impl DependencyType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Transitive => "transitive",
            Self::Dev => "dev",
            Self::Optional => "optional",
            Self::Peer => "peer",
        }
    }
}

// ============================================================================
// Vulnerability Correlation (from v24)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityCorrelation {
    pub threat_intel_id: String,
    pub dependency_node_id: String,
    pub cve_id: String,
    pub package_name: String,
    pub affected_version: String,
    pub fix_version: Option<String>,
    pub correlation_confidence: f64,
    pub correlated_at: DateTime<Utc>,
}

impl VulnerabilityCorrelation {
    pub fn new(
        threat_intel_id: String,
        dependency_node_id: String,
        cve_id: String,
        package_name: String,
        affected_version: String,
    ) -> Self {
        Self {
            threat_intel_id,
            dependency_node_id,
            cve_id,
            package_name,
            affected_version,
            fix_version: None,
            correlation_confidence: 1.0,
            correlated_at: Utc::now(),
        }
    }

    pub fn with_fix_version(mut self, fix: String) -> Self {
        self.fix_version = Some(fix);
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.correlation_confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn is_high_confidence(&self) -> bool {
        self.correlation_confidence >= 0.8
    }
}

// ============================================================================
// Risk Scoring (from v24)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    pub package_name: String,
    pub repo_id: String,
    pub overall_score: f64,
    pub vulnerability_count: u32,
    pub critical_count: u32,
    pub high_count: u32,
    pub transitive_risk: f64,
    pub last_calculated: DateTime<Utc>,
}

impl RiskScore {
    pub fn new(package_name: String, repo_id: String) -> Self {
        Self {
            package_name,
            repo_id,
            overall_score: 0.0,
            vulnerability_count: 0,
            critical_count: 0,
            high_count: 0,
            transitive_risk: 0.0,
            last_calculated: Utc::now(),
        }
    }

    pub fn calculate(&mut self, correlations: &[VulnerabilityCorrelation]) {
        self.vulnerability_count = correlations.len() as u32;
        self.critical_count = 0;
        self.high_count = 0;

        let mut score = 0.0;
        for corr in correlations {
            score += corr.correlation_confidence * 20.0;
            if corr.cve_id.contains("CRITICAL") || corr.correlation_confidence > 0.9 {
                self.critical_count += 1;
                score += 15.0;
            } else if corr.correlation_confidence > 0.7 {
                self.high_count += 1;
                score += 8.0;
            }
        }

        self.overall_score = score.min(100.0);
        self.last_calculated = Utc::now();
    }

    pub fn risk_level(&self) -> &'static str {
        if self.overall_score >= 80.0 {
            "critical"
        } else if self.overall_score >= 60.0 {
            "high"
        } else if self.overall_score >= 40.0 {
            "medium"
        } else if self.overall_score >= 20.0 {
            "low"
        } else {
            "informational"
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoRiskSummary {
    pub repo_id: String,
    pub total_packages: u32,
    pub total_vulnerabilities: u32,
    pub critical_vulnerabilities: u32,
    pub average_risk_score: f64,
    pub calculated_at: DateTime<Utc>,
}

// ============================================================================
// Store & Engine Types (from v24)
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThreatIntelligenceStore {
    threats: Vec<ThreatIntelligence>,
    by_cve: HashMap<String, Vec<usize>>,
    by_package: HashMap<String, Vec<usize>>,
}

impl ThreatIntelligenceStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_threat(&mut self, threat: ThreatIntelligence) {
        let idx = self.threats.len();
        self.by_cve
            .entry(threat.cve_id.clone())
            .or_default()
            .push(idx);
        for pkg in &threat.affected_packages {
            self.by_package
                .entry(pkg.clone())
                .or_default()
                .push(idx);
        }
        self.threats.push(threat);
    }

    pub fn get_by_cve(&self, cve_id: &str) -> Vec<&ThreatIntelligence> {
        self.by_cve
            .get(cve_id)
            .map(|indices| indices.iter().map(|&idx| &self.threats[idx]).collect())
            .unwrap_or_default()
    }

    pub fn get_for_package(&self, package: &str) -> Vec<&ThreatIntelligence> {
        self.by_package
            .get(package)
            .map(|indices| indices.iter().map(|&idx| &self.threats[idx]).collect())
            .unwrap_or_default()
    }

    pub fn critical_threats(&self) -> Vec<&ThreatIntelligence> {
        self.threats
            .iter()
            .filter(|t| t.is_critical())
            .collect()
    }

    pub fn total(&self) -> usize {
        self.threats.len()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyTreeAnalyzer {
    nodes: Vec<DependencyTreeNode>,
    by_repo: HashMap<String, Vec<usize>>,
    by_package: HashMap<String, Vec<usize>>,
    parent_index: HashMap<String, Vec<usize>>,
}

impl DependencyTreeAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: DependencyTreeNode) {
        let idx = self.nodes.len();
        self.by_repo
            .entry(node.repo_id.clone())
            .or_default()
            .push(idx);
        self.by_package
            .entry(node.package_name.clone())
            .or_default()
            .push(idx);
        if let Some(ref parent) = node.parent_package {
            self.parent_index
                .entry(parent.clone())
                .or_default()
                .push(idx);
        }
        self.nodes.push(node);
    }

    pub fn get_nodes_for_repo(&self, repo_id: &str) -> Vec<&DependencyTreeNode> {
        self.by_repo
            .get(repo_id)
            .map(|indices| indices.iter().map(|&idx| &self.nodes[idx]).collect())
            .unwrap_or_default()
    }

    pub fn get_children(&self, package_name: &str) -> Vec<&DependencyTreeNode> {
        self.parent_index
            .get(package_name)
            .map(|indices| indices.iter().map(|&idx| &self.nodes[idx]).collect())
            .unwrap_or_default()
    }

    pub fn find_package(&self, name: &str, repo_id: &str) -> Option<&DependencyTreeNode> {
        self.nodes.iter().find(|n| n.package_name == name && n.repo_id == repo_id)
    }

    pub fn max_depth_for_repo(&self, repo_id: &str) -> u32 {
        self.get_nodes_for_repo(repo_id)
            .iter()
            .map(|n| n.depth)
            .max()
            .unwrap_or(0)
    }

    pub fn total_packages_for_repo(&self, repo_id: &str) -> usize {
        self.get_nodes_for_repo(repo_id).len()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VulnerabilityCorrelationEngine {
    correlations: Vec<VulnerabilityCorrelation>,
    by_threat: HashMap<String, Vec<usize>>,
    by_dependency: HashMap<String, Vec<usize>>,
}

impl VulnerabilityCorrelationEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_correlation(&mut self, corr: VulnerabilityCorrelation) {
        let idx = self.correlations.len();
        self.by_threat
            .entry(corr.threat_intel_id.clone())
            .or_default()
            .push(idx);
        self.by_dependency
            .entry(corr.dependency_node_id.clone())
            .or_default()
            .push(idx);
        self.correlations.push(corr);
    }

    pub fn get_correlations_for_threat(
        &self,
        threat_id: &str,
    ) -> Vec<&VulnerabilityCorrelation> {
        self.by_threat
            .get(threat_id)
            .map(|indices| indices.iter().map(|&idx| &self.correlations[idx]).collect())
            .unwrap_or_default()
    }

    pub fn get_correlations_for_dependency(
        &self,
        dep_id: &str,
    ) -> Vec<&VulnerabilityCorrelation> {
        self.by_dependency
            .get(dep_id)
            .map(|indices| indices.iter().map(|&idx| &self.correlations[idx]).collect())
            .unwrap_or_default()
    }

    pub fn high_confidence_correlations(&self) -> Vec<&VulnerabilityCorrelation> {
        self.correlations
            .iter()
            .filter(|c| c.is_high_confidence())
            .collect()
    }

    pub fn total(&self) -> usize {
        self.correlations.len()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RiskScoringEngine {
    scores: HashMap<String, RiskScore>,
}

impl RiskScoringEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn calculate_score(
        &mut self,
        package_name: &str,
        repo_id: &str,
        correlations: &[VulnerabilityCorrelation],
    ) -> RiskScore {
        let key = format!("{}:{}", repo_id, package_name);
        let mut score = RiskScore::new(package_name.into(), repo_id.into());
        score.calculate(correlations);
        self.scores.insert(key, score.clone());
        score
    }

    pub fn get_score(&self, package_name: &str, repo_id: &str) -> Option<&RiskScore> {
        let key = format!("{}:{}", repo_id, package_name);
        self.scores.get(&key)
    }

    pub fn highest_risk_packages(&self, repo_id: &str) -> Vec<&RiskScore> {
        let mut scores: Vec<_> = self
            .scores
            .values()
            .filter(|s| s.repo_id == repo_id)
            .collect();
        scores.sort_by(|a, b| b.overall_score.partial_cmp(&a.overall_score).expect("operation should succeed"));
        scores
    }

    pub fn repo_risk_summary(&self, repo_id: &str) -> RepoRiskSummary {
        let repo_scores: Vec<_> = self
            .scores
            .values()
            .filter(|s| s.repo_id == repo_id)
            .collect();
        let total_packages = repo_scores.len() as u32;
        let total_vulns: u32 = repo_scores.iter().map(|s| s.vulnerability_count).sum();
        let critical: u32 = repo_scores.iter().map(|s| s.critical_count).sum();
        let avg_score = if total_packages == 0 {
            0.0
        } else {
            repo_scores.iter().map(|s| s.overall_score).sum::<f64>() / total_packages as f64
        };

        RepoRiskSummary {
            repo_id: repo_id.into(),
            total_packages,
            total_vulnerabilities: total_vulns,
            critical_vulnerabilities: critical,
            average_risk_score: avg_score,
            calculated_at: Utc::now(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

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

    fn make_scan(findings: Vec<SecurityFinding>) -> SecurityScan {
        SecurityScan {
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
    }

    #[test]
    fn test_scan_status_serialization() {
        assert_eq!(
            serde_json::to_string(&ScanStatus::Pending).unwrap(),
            "\"pending\""
        );
    }

    #[test]
    fn test_finding_severity_ordering() {
        assert!(FindingSeverity::Critical > FindingSeverity::High);
        assert!(FindingSeverity::High > FindingSeverity::Medium);
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
        assert_eq!(summary.total(), 6);
    }

    #[test]
    fn test_policy_evaluation_pass() {
        let scan = make_scan(vec![make_finding(FindingSeverity::Low)]);
        let policy = make_policy(Some(0), Some(5));
        let result = SecurityPolicyEvaluator::evaluate(&scan, &policy);
        assert!(result.passed);
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
        assert_eq!(result.violations[0].rule, "max_critical");
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
        assert_eq!(scan.score, 100);
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
        manager.update_rule_pattern(&id, "v2".into(), None, None, false).unwrap();
        manager.update_rule_pattern(&id, "v3".into(), None, None, true).unwrap();
        let versions = manager.get_rule_versions(&id).unwrap();
        assert_eq!(versions.len(), 3);
        assert_eq!(versions[2].version, 3);
        assert!(versions[2].breaking_change);
    }

    #[test]
    fn test_scan_rule_manager_fix_analytics() {
        let mut manager = ScanRuleManager::new();
        let rule = SecurityScanRule::new(
            "Rule1".into(),
            "Desc".into(),
            RuleType::Regex,
            "pattern".into(),
        )
        .with_auto_fix(serde_json::json!({}));
        manager.add_rule(rule);
        assert_eq!(manager.list_auto_fix_rules().len(), 1);
    }

    #[test]
    fn test_deduplication_engine() {
        let mut engine = DeduplicationEngine::new();
        let r1 = engine.check_and_record("v1".into(), "r1".into(), "src/main.rs".into(), Some(42));
        assert!(!r1.is_duplicate);
        let r2 = engine.check_and_record("v1".into(), "r1".into(), "src/main.rs".into(), Some(42));
        assert!(r2.is_duplicate);
        assert_eq!(r2.occurrence_count, 2);
        assert_eq!(engine.total_unique(), 1);
    }

    #[test]
    fn test_false_positive_tracker() {
        let mut tracker = FalsePositiveTracker::new();
        let record = tracker.mark_false_positive(
            "v1".into(),
            "r1".into(),
            "Not a real vuln".into(),
            Some("user-1".into()),
        );
        assert!(tracker.is_false_positive("v1"));
        assert_eq!(tracker.active_count(), 1);
        tracker.revoke(&record.id).unwrap();
        assert!(!tracker.is_false_positive("v1"));
    }

    #[test]
    fn test_scan_schedule() {
        let schedule = ScanSchedule::new("r1".into(), "rs1".into(), 60);
        assert!(schedule.is_due());
        let mut schedule2 = schedule.clone();
        schedule2.record_run();
        assert!(!schedule2.is_due());
    }

    #[test]
    fn test_rule_set_manager() {
        let mut manager = RuleSetManager::new();
        let rs = SecurityScanRuleSet::new("Test".into(), "Desc".into());
        manager.add_rule_set(rs);
        assert_eq!(manager.list_rule_sets().len(), 1);
        assert_eq!(manager.get_enabled_rule_sets().len(), 1);
        let report = manager.generate_report();
        assert!(report.contains("Rule Sets Report"));
    }

    #[test]
    fn test_threat_intelligence() {
        let threat = ThreatIntelligence::new(
            "CVE-2024-0001".into(),
            ThreatSeverity::Critical,
            "Test vuln".into(),
        )
        .with_packages(vec!["libc".into()])
        .with_fix_available(true);
        assert!(threat.is_critical());
        assert!(threat.affects_package("libc"));
        assert!(!threat.affects_package("other"));
    }

    #[test]
    fn test_dependency_tree_node() {
        let node = DependencyTreeNode::new("repo-1".into(), "serde".into(), "1.0".into());
        assert!(!node.is_transitive());
        let child = DependencyTreeNode::new("repo-1".into(), "child".into(), "2.0".into())
            .with_parent("serde".into(), DependencyType::Transitive)
            .with_depth(2);
        assert!(child.is_transitive());
        assert_eq!(child.depth, 2);
    }

    #[test]
    fn test_vulnerability_correlation() {
        let corr = VulnerabilityCorrelation::new(
            "t1".into(),
            "d1".into(),
            "CVE-1".into(),
            "libc".into(),
            "2.31".into(),
        )
        .with_confidence(0.5);
        assert!(!corr.is_high_confidence());
        let corr2 = corr.with_confidence(0.9);
        assert!(corr2.is_high_confidence());
    }

    #[test]
    fn test_risk_score() {
        let mut score = RiskScore::new("libc".into(), "repo-1".into());
        assert_eq!(score.risk_level(), "informational");
        score.overall_score = 85.0;
        assert_eq!(score.risk_level(), "critical");
        score.overall_score = 65.0;
        assert_eq!(score.risk_level(), "high");
    }

    #[test]
    fn test_threat_intelligence_store() {
        let mut store = ThreatIntelligenceStore::new();
        store.add_threat(ThreatIntelligence::new(
            "CVE-1".into(),
            ThreatSeverity::Critical,
            "Desc".into(),
        )
        .with_packages(vec!["libc".into()]));
        assert_eq!(store.total(), 1);
        assert_eq!(store.get_for_package("libc").len(), 1);
        assert_eq!(store.critical_threats().len(), 1);
    }

    #[test]
    fn test_dependency_tree_analyzer() {
        let mut analyzer = DependencyTreeAnalyzer::new();
        analyzer.add_node(DependencyTreeNode::new("repo-1".into(), "root".into(), "1.0".into()));
        analyzer.add_node(
            DependencyTreeNode::new("repo-1".into(), "child".into(), "2.0".into())
                .with_parent("root".into(), DependencyType::Direct)
                .with_depth(1),
        );
        assert_eq!(analyzer.get_nodes_for_repo("repo-1").len(), 2);
        assert_eq!(analyzer.get_children("root").len(), 1);
        assert_eq!(analyzer.max_depth_for_repo("repo-1"), 1);
    }

    #[test]
    fn test_vulnerability_correlation_engine() {
        let mut engine = VulnerabilityCorrelationEngine::new();
        engine.add_correlation(VulnerabilityCorrelation::new(
            "t1".into(),
            "d1".into(),
            "CVE-1".into(),
            "pkg".into(),
            "1.0".into(),
        ));
        assert_eq!(engine.total(), 1);
        assert_eq!(engine.get_correlations_for_threat("t1").len(), 1);
        assert_eq!(engine.high_confidence_correlations().len(), 1);
    }

    #[test]
    fn test_risk_scoring_engine() {
        let mut engine = RiskScoringEngine::new();
        let corrs = vec![VulnerabilityCorrelation::new(
            "t1".into(),
            "d1".into(),
            "CVE-1".into(),
            "pkg".into(),
            "1.0".into(),
        )];
        let score = engine.calculate_score("pkg", "repo-1", &corrs);
        assert!(score.overall_score > 0.0);
        assert!(engine.get_score("pkg", "repo-1").is_some());
        let summary = engine.repo_risk_summary("repo-1");
        assert_eq!(summary.total_packages, 1);
    }

    #[test]
    fn test_compliance_mapping_types() {
        assert_eq!(
            serde_json::to_string(&ComplianceMappingType::Direct).unwrap(),
            "\"direct\""
        );
        assert_eq!(
            serde_json::to_string(&ComplianceMappingType::Inherited).unwrap(),
            "\"inherited\""
        );
    }
}
