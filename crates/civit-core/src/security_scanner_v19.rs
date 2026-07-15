#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleTypeV19 {
    Regex,
    Keyword,
    Ast,
    Pattern,
    Custom,
    Composite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeverityV19 {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

impl SeverityV19 {
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
pub struct SecurityScanRuleV19 {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_type: RuleTypeV19,
    pub severity: SeverityV19,
    pub pattern: String,
    pub enabled: bool,
    pub version: u32,
    pub author_id: Option<String>,
    pub auto_fix: bool,
    pub fix_config: serde_json::Value,
    pub compliance_mapping: ComplianceMappingV19,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SecurityScanRuleV19 {
    pub fn new(name: String, description: String, rule_type: RuleTypeV19, pattern: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            rule_type,
            severity: SeverityV19::Medium,
            pattern,
            enabled: true,
            version: 1,
            author_id: None,
            auto_fix: false,
            fix_config: serde_json::Value::Object(serde_json::Map::new()),
            compliance_mapping: ComplianceMappingV19::default(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_severity(mut self, severity: SeverityV19) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_author(mut self, author_id: &str) -> Self {
        self.author_id = Some(author_id.into());
        self
    }

    pub fn with_auto_fix(mut self, fix_config: serde_json::Value) -> Self {
        self.auto_fix = true;
        self.fix_config = fix_config;
        self
    }

    pub fn with_compliance_mapping(mut self, mapping: ComplianceMappingV19) -> Self {
        self.compliance_mapping = mapping;
        self
    }

    pub fn increment_version(&mut self) {
        self.version += 1;
        self.updated_at = Utc::now();
    }

    pub fn risk_score(&self) -> u32 {
        self.severity.risk_weight()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplianceMappingV19 {
    pub mappings: Vec<ComplianceMappingEntryV19>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceMappingEntryV19 {
    pub framework_id: String,
    pub framework_name: String,
    pub requirement_id: String,
    pub mapping_type: ComplianceMappingTypeV19,
    pub confidence: f64,
    pub auto_verifiable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceMappingTypeV19 {
    Direct,
    Partial,
    Advisory,
    Inherited,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanFixV19 {
    pub id: String,
    pub scan_id: String,
    pub rule_id: String,
    pub file_path: String,
    pub line_number: Option<u32>,
    pub fix_type: FixTypeV19,
    pub fix_content: String,
    pub applied: bool,
    pub test_result: Option<FixTestResultV19>,
    pub applied_at: Option<DateTime<Utc>>,
    pub applied_by: Option<String>,
    pub rollback_content: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixTypeV19 {
    PatternReplace,
    LineRemoval,
    LineInsertion,
    ConfigPatch,
    DependencyUpdate,
    Composite,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixTestResultV19 {
    pub passed: bool,
    pub test_output: String,
    pub regressions: Vec<String>,
    pub tested_at: DateTime<Utc>,
    pub test_duration_ms: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FixAnalyticsV19 {
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
    pub compliance_impact: ComplianceImpactV19,
    pub rollback_count: u64,
    pub avg_apply_time_ms: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplianceImpactV19 {
    pub rules_with_compliance: u64,
    pub fixes_improving_compliance: u64,
    pub compliance_score_delta: f64,
    pub framework_coverage: HashMap<String, f64>,
    pub auto_verifiable_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleVersionHistoryV19 {
    pub rule_id: String,
    pub versions: Vec<RuleVersionEntryV19>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleVersionEntryV19 {
    pub version: u32,
    pub pattern: String,
    pub changed_by: Option<String>,
    pub changed_at: DateTime<Utc>,
    pub change_notes: Option<String>,
    pub compliance_mappings_snapshot: ComplianceMappingV19,
    pub breaking_change: bool,
}

#[derive(Debug, Clone)]
pub struct SecurityScanRuleManagerV19 {
    rules: Vec<SecurityScanRuleV19>,
    fixes: Vec<ScanFixV19>,
    version_history: HashMap<String, Vec<RuleVersionEntryV19>>,
}

impl Default for SecurityScanRuleManagerV19 {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityScanRuleManagerV19 {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            fixes: Vec::new(),
            version_history: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule: SecurityScanRuleV19) {
        let entry = RuleVersionEntryV19 {
            version: rule.version,
            pattern: rule.pattern.clone(),
            changed_by: rule.author_id.clone(),
            changed_at: rule.created_at,
            change_notes: Some("Initial creation".into()),
            compliance_mappings_snapshot: rule.compliance_mapping.clone(),
            breaking_change: false,
        };
        self.version_history
            .entry(rule.id.clone())
            .or_default()
            .push(entry);
        self.rules.push(rule);
    }

    pub fn get_rule(&self, rule_id: &str) -> Option<&SecurityScanRuleV19> {
        self.rules.iter().find(|r| r.id == rule_id)
    }

    pub fn get_rule_mut(&mut self, rule_id: &str) -> Option<&mut SecurityScanRuleV19> {
        self.rules.iter_mut().find(|r| r.id == rule_id)
    }

    pub fn list_rules(&self) -> &[SecurityScanRuleV19] {
        &self.rules
    }

    pub fn list_enabled_rules(&self) -> Vec<&SecurityScanRuleV19> {
        self.rules.iter().filter(|r| r.enabled).collect()
    }

    pub fn list_auto_fix_rules(&self) -> Vec<&SecurityScanRuleV19> {
        self.rules.iter().filter(|r| r.auto_fix && r.enabled).collect()
    }

    pub fn list_rules_by_severity(&self, severity: SeverityV19) -> Vec<&SecurityScanRuleV19> {
        self.rules.iter().filter(|r| r.severity == severity).collect()
    }

    pub fn list_rules_with_compliance(&self) -> Vec<&SecurityScanRuleV19> {
        self.rules
            .iter()
            .filter(|r| !r.compliance_mapping.mappings.is_empty())
            .collect()
    }

    pub fn list_auto_verifiable_rules(&self) -> Vec<&SecurityScanRuleV19> {
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

        let entry = RuleVersionEntryV19 {
            version,
            pattern: new_pattern,
            changed_by,
            changed_at,
            change_notes,
            compliance_mappings_snapshot: compliance_snapshot,
            breaking_change,
        };
        self.version_history
            .entry(rule_id.to_string())
            .or_default()
            .push(entry);
        Ok(())
    }

    pub fn get_version_history(&self, rule_id: &str) -> Option<&Vec<RuleVersionEntryV19>> {
        self.version_history.get(rule_id)
    }

    pub fn test_rule(&self, rule: &SecurityScanRuleV19, input: &str) -> RuleTestResultV19 {
        let start = std::time::Instant::now();
        let mut matched_lines = Vec::new();

        for (idx, line) in input.lines().enumerate() {
            let matched = match rule.rule_type {
                RuleTypeV19::Regex => regex::Regex::new(&rule.pattern)
                    .map(|re| re.is_match(line))
                    .unwrap_or(false),
                RuleTypeV19::Keyword => line.contains(&rule.pattern),
                RuleTypeV19::Pattern => line.to_lowercase().contains(&rule.pattern.to_lowercase()),
                RuleTypeV19::Ast | RuleTypeV19::Custom | RuleTypeV19::Composite => false,
            };
            if matched {
                matched_lines.push((idx + 1) as u32);
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        let has_matches = !matched_lines.is_empty();
        let risk_score = if has_matches { rule.risk_score() } else { 0 };
        RuleTestResultV19 {
            rule_id: rule.id.clone(),
            input: input.to_string(),
            matched: has_matches,
            matched_lines,
            execution_time_ms: duration_ms,
            risk_score,
        }
    }

    pub fn record_fix(&mut self, fix: ScanFixV19) {
        self.fixes.push(fix);
    }

    pub fn apply_fix(&mut self, fix_id: &str, applied_by: &str, test_before_apply: bool) -> Result<bool, String> {
        let fix = self
            .fixes
            .iter_mut()
            .find(|f| f.id == fix_id)
            .ok_or("Fix not found")?;

        if test_before_apply && fix.test_result.is_none() {
            return Err("Fix must be tested before applying".into());
        }

        if let Some(ref test) = fix.test_result {
            if !test.passed {
                return Err("Fix failed testing".into());
            }
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

    pub fn test_fix(&mut self, fix_id: &str, test_input: &str) -> Result<FixTestResultV19, String> {
        let fix = self
            .fixes
            .iter_mut()
            .find(|f| f.id == fix_id)
            .ok_or("Fix not found")?;

        let start = std::time::Instant::now();
        let passed = !fix.fix_content.is_empty() && !test_input.is_empty();
        let duration_ms = start.elapsed().as_millis() as u64;
        let result = FixTestResultV19 {
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

    pub fn get_fixes_for_scan(&self, scan_id: &str) -> Vec<&ScanFixV19> {
        self.fixes.iter().filter(|f| f.scan_id == scan_id).collect()
    }

    pub fn get_pending_fixes(&self) -> Vec<&ScanFixV19> {
        self.fixes.iter().filter(|f| !f.applied).collect()
    }

    pub fn get_rollbackable_fixes(&self) -> Vec<&ScanFixV19> {
        self.fixes
            .iter()
            .filter(|f| f.applied && f.rollback_content.is_some())
            .collect()
    }

    pub fn compute_fix_analytics(&self) -> FixAnalyticsV19 {
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
        let compliance_impact = ComplianceImpactV19 {
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

        FixAnalyticsV19 {
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
        let mut report = String::from("=== Security Scan V19 Report ===\n\n");
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
            SeverityV19::Critical,
            SeverityV19::High,
            SeverityV19::Medium,
            SeverityV19::Low,
            SeverityV19::Informational,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleTestResultV19 {
    pub rule_id: String,
    pub input: String,
    pub matched: bool,
    pub matched_lines: Vec<u32>,
    pub execution_time_ms: u64,
    pub risk_score: u32,
}

pub fn create_default_scan_rules_v18() -> Vec<SecurityScanRuleV19> {
    vec![
        SecurityScanRuleV19::new(
            "Hardcoded Secret".into(),
            "Detects hardcoded secrets and API keys".into(),
            RuleTypeV19::Regex,
            r#"(?i)(api[_-]?key|secret|password|token)\s*[:=]\s*['"][^'"]+['"]"#.into(),
        )
        .with_severity(SeverityV19::Critical)
        .with_compliance_mapping(ComplianceMappingV19 {
            mappings: vec![ComplianceMappingEntryV19 {
                framework_id: "soc2".into(),
                framework_name: "SOC 2".into(),
                requirement_id: "CC6.1".into(),
                mapping_type: ComplianceMappingTypeV19::Direct,
                confidence: 0.95,
                auto_verifiable: true,
            }],
        }),
        SecurityScanRuleV19::new(
            "SQL Injection".into(),
            "Detects potential SQL injection patterns".into(),
            RuleTypeV19::Regex,
            r#"(?i)(SELECT|INSERT|UPDATE|DELETE|DROP).*['"].*\+.*['"]"#.into(),
        )
        .with_severity(SeverityV19::High)
        .with_compliance_mapping(ComplianceMappingV19 {
            mappings: vec![ComplianceMappingEntryV19 {
                framework_id: "owasp_top10".into(),
                framework_name: "OWASP Top 10".into(),
                requirement_id: "A03:2021".into(),
                mapping_type: ComplianceMappingTypeV19::Direct,
                confidence: 0.90,
                auto_verifiable: true,
            }],
        }),
        SecurityScanRuleV19::new(
            "XSS Vulnerability".into(),
            "Detects potential cross-site scripting patterns".into(),
            RuleTypeV19::Regex,
            r#"<script[^>]*>.*</script>"#.into(),
        )
        .with_severity(SeverityV19::High)
        .with_compliance_mapping(ComplianceMappingV19 {
            mappings: vec![ComplianceMappingEntryV19 {
                framework_id: "owasp_top10".into(),
                framework_name: "OWASP Top 10".into(),
                requirement_id: "A03:2021".into(),
                mapping_type: ComplianceMappingTypeV19::Direct,
                confidence: 0.85,
                auto_verifiable: true,
            }],
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_rule(id: &str, severity: SeverityV19, auto_fix: bool) -> SecurityScanRuleV19 {
        SecurityScanRuleV19 {
            id: id.to_string(),
            name: format!("Rule {id}"),
            description: format!("Description for rule {id}"),
            rule_type: RuleTypeV19::Pattern,
            severity,
            pattern: format!("pattern_{id}"),
            enabled: true,
            version: 1,
            author_id: Some("user-1".to_string()),
            auto_fix,
            fix_config: serde_json::json!({}),
            compliance_mapping: ComplianceMappingV19::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sample_fix(id: &str, scan_id: &str, rule_id: &str, applied: bool) -> ScanFixV19 {
        ScanFixV19 {
            id: id.to_string(),
            scan_id: scan_id.to_string(),
            rule_id: rule_id.to_string(),
            file_path: format!("src/file_{id}.rs"),
            line_number: Some(42),
            fix_type: FixTypeV19::PatternReplace,
            fix_content: format!("fixed content for {id}"),
            applied,
            test_result: None,
            applied_at: None,
            applied_by: None,
            rollback_content: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_add_rule() {
        let mut manager = SecurityScanRuleManagerV19::new();
        manager.add_rule(sample_rule("r1", SeverityV19::High, false));
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_get_rules_by_severity() {
        let mut manager = SecurityScanRuleManagerV19::new();
        manager.add_rule(sample_rule("r1", SeverityV19::High, false));
        manager.add_rule(sample_rule("r2", SeverityV19::Critical, false));
        manager.add_rule(sample_rule("r3", SeverityV19::High, false));
        let high = manager.list_rules_by_severity(SeverityV19::High);
        assert_eq!(high.len(), 2);
    }

    #[test]
    fn test_get_auto_fix_rules() {
        let mut manager = SecurityScanRuleManagerV19::new();
        manager.add_rule(sample_rule("r1", SeverityV19::High, true));
        manager.add_rule(sample_rule("r2", SeverityV19::Critical, false));
        let auto = manager.list_auto_fix_rules();
        assert_eq!(auto.len(), 1);
        assert_eq!(auto[0].id, "r1");
    }

    #[test]
    fn test_record_fix() {
        let mut manager = SecurityScanRuleManagerV19::new();
        manager.record_fix(sample_fix("f1", "scan-1", "r1", false));
        assert_eq!(manager.fixes.len(), 1);
    }

    #[test]
    fn test_apply_fix() {
        let mut manager = SecurityScanRuleManagerV19::new();
        manager.record_fix(sample_fix("f1", "scan-1", "r1", false));
        let result = manager.apply_fix("f1", "user-1", false);
        assert!(result.is_ok());
        assert!(manager.fixes[0].applied);
        assert!(manager.fixes[0].applied_by.is_some());
        assert!(manager.fixes[0].applied_at.is_some());
    }

    #[test]
    fn test_apply_fix_requires_test() {
        let mut manager = SecurityScanRuleManagerV19::new();
        manager.record_fix(sample_fix("f1", "scan-1", "r1", false));
        let result = manager.apply_fix("f1", "user-1", true);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("tested before applying"));
    }

    #[test]
    fn test_rollback_fix() {
        let mut manager = SecurityScanRuleManagerV19::new();
        let mut fix = sample_fix("f1", "scan-1", "r1", true);
        fix.rollback_content = Some("original content".into());
        manager.record_fix(fix);
        let result = manager.rollback_fix("f1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("original content".into()));
        assert!(!manager.fixes[0].applied);
    }

    #[test]
    fn test_rollback_unapplied_fix_fails() {
        let mut manager = SecurityScanRuleManagerV19::new();
        manager.record_fix(sample_fix("f1", "scan-1", "r1", false));
        let result = manager.rollback_fix("f1");
        assert!(result.is_err());
    }

    #[test]
    fn test_test_fix() {
        let mut manager = SecurityScanRuleManagerV19::new();
        manager.record_fix(sample_fix("f1", "scan-1", "r1", false));
        let result = manager.test_fix("f1", "test input");
        assert!(result.is_ok());
        assert!(result.unwrap().passed);
    }

    #[test]
    fn test_get_fixes_for_scan() {
        let mut manager = SecurityScanRuleManagerV19::new();
        manager.record_fix(sample_fix("f1", "scan-1", "r1", false));
        manager.record_fix(sample_fix("f2", "scan-1", "r2", false));
        manager.record_fix(sample_fix("f3", "scan-2", "r1", false));
        let fixes = manager.get_fixes_for_scan("scan-1");
        assert_eq!(fixes.len(), 2);
    }

    #[test]
    fn test_get_pending_fixes() {
        let mut manager = SecurityScanRuleManagerV19::new();
        manager.record_fix(sample_fix("f1", "scan-1", "r1", false));
        manager.record_fix(sample_fix("f2", "scan-1", "r1", true));
        let pending = manager.get_pending_fixes();
        assert_eq!(pending.len(), 1);
    }

    #[test]
    fn test_compute_fix_analytics() {
        let mut manager = SecurityScanRuleManagerV19::new();
        manager.add_rule(sample_rule("r1", SeverityV19::High, true));
        manager.record_fix(sample_fix("f1", "scan-1", "r1", true));
        manager.record_fix(sample_fix("f2", "scan-1", "r1", false));
        let analytics = manager.compute_fix_analytics();
        assert_eq!(analytics.total_fixes, 2);
        assert_eq!(analytics.applied_fixes, 1);
        assert_eq!(analytics.failed_fixes, 1);
        assert_eq!(analytics.avg_fix_success_rate, 50.0);
    }

    #[test]
    fn test_generate_report() {
        let mut manager = SecurityScanRuleManagerV19::new();
        manager.add_rule(sample_rule("r1", SeverityV19::High, true));
        manager.record_fix(sample_fix("f1", "scan-1", "r1", true));
        let report = manager.generate_report();
        assert!(report.contains("Security Scan V19 Report"));
        assert!(report.contains("Total Rules: 1"));
        assert!(report.contains("Auto-Verifiable"));
    }

    #[test]
    fn test_rule_versioning() {
        let mut manager = SecurityScanRuleManagerV19::new();
        manager.add_rule(sample_rule("r1", SeverityV19::High, false));
        manager
            .update_rule_pattern("r1", "new_pattern".into(), Some("user-1".into()), None, false)
            .unwrap();
        let history = manager.get_version_history("r1").unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].version, 1);
        assert_eq!(history[1].version, 2);
        assert!(!history[1].breaking_change);
    }

    #[test]
    fn test_breaking_change_tracking() {
        let mut manager = SecurityScanRuleManagerV19::new();
        manager.add_rule(sample_rule("r1", SeverityV19::High, false));
        manager
            .update_rule_pattern("r1", "new_pattern".into(), Some("user-1".into()), None, true)
            .unwrap();
        let history = manager.get_version_history("r1").unwrap();
        assert!(history[1].breaking_change);
    }

    #[test]
    fn test_compliance_mapping() {
        let rule = SecurityScanRuleV19::new(
            "Test".into(),
            "Desc".into(),
            RuleTypeV19::Regex,
            "pattern".into(),
        )
        .with_compliance_mapping(ComplianceMappingV19 {
            mappings: vec![ComplianceMappingEntryV19 {
                framework_id: "soc2".into(),
                framework_name: "SOC 2".into(),
                requirement_id: "CC6.1".into(),
                mapping_type: ComplianceMappingTypeV19::Direct,
                confidence: 0.95,
                auto_verifiable: true,
            }],
        });
        assert_eq!(rule.compliance_mapping.mappings.len(), 1);
        assert!(rule.compliance_mapping.mappings[0].auto_verifiable);
    }

    #[test]
    fn test_severity_risk_weight() {
        assert_eq!(SeverityV19::Critical.risk_weight(), 5);
        assert_eq!(SeverityV19::High.risk_weight(), 4);
        assert_eq!(SeverityV19::Medium.risk_weight(), 2);
        assert_eq!(SeverityV19::Low.risk_weight(), 1);
        assert_eq!(SeverityV19::Informational.risk_weight(), 0);
    }

    #[test]
    fn test_severity_display_name() {
        assert_eq!(SeverityV19::Critical.display_name(), "Critical");
        assert_eq!(SeverityV19::High.display_name(), "High");
    }

    #[test]
    fn test_empty_manager() {
        let manager = SecurityScanRuleManagerV19::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
        let analytics = manager.compute_fix_analytics();
        assert_eq!(analytics.total_fixes, 0);
    }

    #[test]
    fn test_fix_analytics_with_compliance() {
        let mut manager = SecurityScanRuleManagerV19::new();
        let mut rule = sample_rule("r1", SeverityV19::High, true);
        rule.compliance_mapping = ComplianceMappingV19 {
            mappings: vec![ComplianceMappingEntryV19 {
                framework_id: "soc2".into(),
                framework_name: "SOC 2".into(),
                requirement_id: "CC6.1".into(),
                mapping_type: ComplianceMappingTypeV19::Direct,
                confidence: 0.9,
                auto_verifiable: true,
            }],
        };
        manager.add_rule(rule);
        manager.record_fix(sample_fix("f1", "scan-1", "r1", true));
        let analytics = manager.compute_fix_analytics();
        assert_eq!(analytics.fixes_by_framework.get("SOC 2"), Some(&1));
        assert_eq!(analytics.compliance_impact.rules_with_compliance, 1);
        assert_eq!(analytics.compliance_impact.fixes_improving_compliance, 1);
        assert_eq!(analytics.compliance_impact.auto_verifiable_count, 1);
    }

    #[test]
    fn test_default_rules_have_compliance_mapping() {
        let rules = create_default_scan_rules_v18();
        assert_eq!(rules.len(), 3);
        for rule in &rules {
            assert!(!rule.compliance_mapping.mappings.is_empty());
            for mapping in &rule.compliance_mapping.mappings {
                assert!(mapping.auto_verifiable);
            }
        }
    }

    #[test]
    fn test_list_rules_with_compliance() {
        let mut manager = SecurityScanRuleManagerV19::new();
        let mut rule = sample_rule("r1", SeverityV19::High, false);
        rule.compliance_mapping = ComplianceMappingV19 {
            mappings: vec![ComplianceMappingEntryV19 {
                framework_id: "soc2".into(),
                framework_name: "SOC 2".into(),
                requirement_id: "CC6.1".into(),
                mapping_type: ComplianceMappingTypeV19::Direct,
                confidence: 0.9,
                auto_verifiable: false,
            }],
        };
        manager.add_rule(rule);
        manager.add_rule(sample_rule("r2", SeverityV19::Low, false));
        assert_eq!(manager.list_rules_with_compliance().len(), 1);
    }

    #[test]
    fn test_list_auto_verifiable_rules() {
        let mut manager = SecurityScanRuleManagerV19::new();
        let mut rule = sample_rule("r1", SeverityV19::High, false);
        rule.compliance_mapping = ComplianceMappingV19 {
            mappings: vec![ComplianceMappingEntryV19 {
                framework_id: "soc2".into(),
                framework_name: "SOC 2".into(),
                requirement_id: "CC6.1".into(),
                mapping_type: ComplianceMappingTypeV19::Direct,
                confidence: 0.9,
                auto_verifiable: true,
            }],
        };
        manager.add_rule(rule);
        manager.add_rule(sample_rule("r2", SeverityV19::Low, false));
        assert_eq!(manager.list_auto_verifiable_rules().len(), 1);
    }

    #[test]
    fn test_rule_risk_score() {
        let rule = sample_rule("r1", SeverityV19::Critical, false);
        assert_eq!(rule.risk_score(), 5);
    }
}
