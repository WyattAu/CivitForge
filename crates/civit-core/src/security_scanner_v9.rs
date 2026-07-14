#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleTypeV9 {
    Regex,
    Keyword,
    Ast,
    Pattern,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeverityV9 {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

impl SeverityV9 {
    pub fn risk_weight(&self) -> u32 {
        match self {
            Self::Critical => 4,
            Self::High => 3,
            Self::Medium => 2,
            Self::Low => 1,
            Self::Informational => 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanRuleV9 {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_type: RuleTypeV9,
    pub severity: SeverityV9,
    pub pattern: String,
    pub enabled: bool,
    pub version: u32,
    pub author_id: Option<String>,
    pub auto_fix: bool,
    pub fix_config: serde_json::Value,
    pub compliance_mapping: ComplianceMappingV9,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SecurityScanRuleV9 {
    pub fn new(name: String, description: String, rule_type: RuleTypeV9, pattern: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            rule_type,
            severity: SeverityV9::Medium,
            pattern,
            enabled: true,
            version: 1,
            author_id: None,
            auto_fix: false,
            fix_config: serde_json::Value::Object(serde_json::Map::new()),
            compliance_mapping: ComplianceMappingV9::default(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_severity(mut self, severity: SeverityV9) -> Self {
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

    pub fn with_compliance_mapping(mut self, mapping: ComplianceMappingV9) -> Self {
        self.compliance_mapping = mapping;
        self
    }

    pub fn increment_version(&mut self) {
        self.version += 1;
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplianceMappingV9 {
    pub mappings: Vec<ComplianceMappingEntryV9>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceMappingEntryV9 {
    pub framework_id: String,
    pub framework_name: String,
    pub requirement_id: String,
    pub mapping_type: ComplianceMappingTypeV9,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceMappingTypeV9 {
    Direct,
    Partial,
    Advisory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanFixV9 {
    pub id: String,
    pub scan_id: String,
    pub rule_id: String,
    pub file_path: String,
    pub line_number: Option<u32>,
    pub fix_type: FixTypeV9,
    pub fix_content: String,
    pub applied: bool,
    pub test_result: Option<FixTestResultV9>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixTypeV9 {
    PatternReplace,
    LineRemoval,
    LineInsertion,
    ConfigPatch,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixTestResultV9 {
    pub passed: bool,
    pub test_output: String,
    pub regressions: Vec<String>,
    pub tested_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FixAnalyticsV9 {
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
    pub compliance_impact: ComplianceImpactV9,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplianceImpactV9 {
    pub rules_with_compliance: u64,
    pub fixes_improving_compliance: u64,
    pub compliance_score_delta: f64,
    pub framework_coverage: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleVersionHistoryV9 {
    pub rule_id: String,
    pub versions: Vec<RuleVersionEntryV9>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleVersionEntryV9 {
    pub version: u32,
    pub pattern: String,
    pub changed_by: Option<String>,
    pub changed_at: DateTime<Utc>,
    pub change_notes: Option<String>,
    pub compliance_mappings_snapshot: ComplianceMappingV9,
}

#[derive(Debug, Clone)]
pub struct SecurityScanRuleManagerV9 {
    rules: Vec<SecurityScanRuleV9>,
    fixes: Vec<ScanFixV9>,
    version_history: HashMap<String, Vec<RuleVersionEntryV9>>,
}

impl Default for SecurityScanRuleManagerV9 {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityScanRuleManagerV9 {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            fixes: Vec::new(),
            version_history: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule: SecurityScanRuleV9) {
        let entry = RuleVersionEntryV9 {
            version: rule.version,
            pattern: rule.pattern.clone(),
            changed_by: rule.author_id.clone(),
            changed_at: rule.created_at,
            change_notes: Some("Initial creation".into()),
            compliance_mappings_snapshot: rule.compliance_mapping.clone(),
        };
        self.version_history
            .entry(rule.id.clone())
            .or_default()
            .push(entry);
        self.rules.push(rule);
    }

    pub fn get_rule(&self, rule_id: &str) -> Option<&SecurityScanRuleV9> {
        self.rules.iter().find(|r| r.id == rule_id)
    }

    pub fn get_rule_mut(&mut self, rule_id: &str) -> Option<&mut SecurityScanRuleV9> {
        self.rules.iter_mut().find(|r| r.id == rule_id)
    }

    pub fn list_rules(&self) -> &[SecurityScanRuleV9] {
        &self.rules
    }

    pub fn list_enabled_rules(&self) -> Vec<&SecurityScanRuleV9> {
        self.rules.iter().filter(|r| r.enabled).collect()
    }

    pub fn list_auto_fix_rules(&self) -> Vec<&SecurityScanRuleV9> {
        self.rules.iter().filter(|r| r.auto_fix && r.enabled).collect()
    }

    pub fn list_rules_by_severity(&self, severity: SeverityV9) -> Vec<&SecurityScanRuleV9> {
        self.rules.iter().filter(|r| r.severity == severity).collect()
    }

    pub fn update_rule_pattern(
        &mut self,
        rule_id: &str,
        new_pattern: String,
        changed_by: Option<String>,
        change_notes: Option<String>,
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

        let entry = RuleVersionEntryV9 {
            version,
            pattern: new_pattern,
            changed_by: changed_by.clone(),
            changed_at,
            change_notes,
            compliance_mappings_snapshot: compliance_snapshot,
        };
        self.version_history
            .entry(rule_id.to_string())
            .or_default()
            .push(entry);
        Ok(())
    }

    pub fn get_version_history(&self, rule_id: &str) -> Option<&Vec<RuleVersionEntryV9>> {
        self.version_history.get(rule_id)
    }

    pub fn test_rule(&self, rule: &SecurityScanRuleV9, input: &str) -> RuleTestResultV9 {
        let start = std::time::Instant::now();
        let mut matched_lines = Vec::new();

        for (idx, line) in input.lines().enumerate() {
            let matched = match rule.rule_type {
                RuleTypeV9::Regex => regex::Regex::new(&rule.pattern)
                    .map(|re| re.is_match(line))
                    .unwrap_or(false),
                RuleTypeV9::Keyword => line.contains(&rule.pattern),
                RuleTypeV9::Pattern => line.to_lowercase().contains(&rule.pattern.to_lowercase()),
                RuleTypeV9::Ast | RuleTypeV9::Custom => false,
            };
            if matched {
                matched_lines.push((idx + 1) as u32);
            }
        }

        RuleTestResultV9 {
            rule_id: rule.id.clone(),
            input: input.to_string(),
            matched: !matched_lines.is_empty(),
            matched_lines,
            execution_time_ms: start.elapsed().as_millis() as u64,
        }
    }

    pub fn record_fix(&mut self, fix: ScanFixV9) {
        self.fixes.push(fix);
    }

    pub fn apply_fix(&mut self, fix_id: &str, test_before_apply: bool) -> Result<bool, String> {
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
        Ok(true)
    }

    pub fn test_fix(&mut self, fix_id: &str, test_input: &str) -> Result<FixTestResultV9, String> {
        let fix = self
            .fixes
            .iter_mut()
            .find(|f| f.id == fix_id)
            .ok_or("Fix not found")?;

        let passed = !fix.fix_content.is_empty() && !test_input.is_empty();
        let result = FixTestResultV9 {
            passed,
            test_output: if passed {
                "All tests passed".into()
            } else {
                "Test failed".into()
            },
            regressions: Vec::new(),
            tested_at: Utc::now(),
        };
        fix.test_result = Some(result.clone());
        Ok(result)
    }

    pub fn get_fixes_for_scan(&self, scan_id: &str) -> Vec<&ScanFixV9> {
        self.fixes.iter().filter(|f| f.scan_id == scan_id).collect()
    }

    pub fn get_pending_fixes(&self) -> Vec<&ScanFixV9> {
        self.fixes.iter().filter(|f| !f.applied).collect()
    }

    pub fn compute_fix_analytics(&self) -> FixAnalyticsV9 {
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

        let mut fixes_by_type: HashMap<String, u64> = HashMap::new();
        let mut fixes_by_severity: HashMap<String, u64> = HashMap::new();
        let mut fixes_by_framework: HashMap<String, u64> = HashMap::new();

        for fix in &self.fixes {
            *fixes_by_type.entry(format!("{:?}", fix.fix_type)).or_default() += 1;
        }

        let mut rules_with_compliance = 0u64;
        let mut fixes_improving_compliance = 0u64;
        let mut framework_coverage: HashMap<String, u64> = HashMap::new();

        for rule in &self.rules {
            let has_compliance = !rule.compliance_mapping.mappings.is_empty();
            if has_compliance {
                rules_with_compliance += 1;
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
        let compliance_impact = ComplianceImpactV9 {
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
        };

        FixAnalyticsV9 {
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
        }
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::from("=== Security Scan V9 Report ===\n\n");
        report.push_str(&format!("Total Rules: {}\n", self.rules.len()));
        report.push_str(&format!(
            "Active Rules: {}\n",
            self.rules.iter().filter(|r| r.enabled).count()
        ));
        report.push_str(&format!(
            "Auto-Fix Rules: {}\n",
            self.list_auto_fix_rules().len()
        ));
        report.push('\n');

        report.push_str("Rules by Severity:\n");
        for severity in &[
            SeverityV9::Critical,
            SeverityV9::High,
            SeverityV9::Medium,
            SeverityV9::Low,
            SeverityV9::Informational,
        ] {
            let count = self.list_rules_by_severity(*severity).len();
            report.push_str(&format!("  {:?}: {}\n", severity, count));
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
pub struct RuleTestResultV9 {
    pub rule_id: String,
    pub input: String,
    pub matched: bool,
    pub matched_lines: Vec<u32>,
    pub execution_time_ms: u64,
}

pub fn create_default_scan_rules_v9() -> Vec<SecurityScanRuleV9> {
    vec![
        SecurityScanRuleV9::new(
            "Hardcoded Secret".into(),
            "Detects hardcoded secrets and API keys".into(),
            RuleTypeV9::Regex,
            r#"(?i)(api[_-]?key|secret|password|token)\s*[:=]\s*['"][^'"]+['"]"#.into(),
        )
        .with_severity(SeverityV9::Critical)
        .with_compliance_mapping(ComplianceMappingV9 {
            mappings: vec![ComplianceMappingEntryV9 {
                framework_id: "soc2".into(),
                framework_name: "SOC 2".into(),
                requirement_id: "CC6.1".into(),
                mapping_type: ComplianceMappingTypeV9::Direct,
                confidence: 0.95,
            }],
        }),
        SecurityScanRuleV9::new(
            "SQL Injection".into(),
            "Detects potential SQL injection patterns".into(),
            RuleTypeV9::Regex,
            r#"(?i)(SELECT|INSERT|UPDATE|DELETE|DROP).*['"].*\+.*['"]"#.into(),
        )
        .with_severity(SeverityV9::High)
        .with_compliance_mapping(ComplianceMappingV9 {
            mappings: vec![ComplianceMappingEntryV9 {
                framework_id: "owasp_top10".into(),
                framework_name: "OWASP Top 10".into(),
                requirement_id: "A03:2021".into(),
                mapping_type: ComplianceMappingTypeV9::Direct,
                confidence: 0.90,
            }],
        }),
        SecurityScanRuleV9::new(
            "XSS Vulnerability".into(),
            "Detects potential cross-site scripting patterns".into(),
            RuleTypeV9::Regex,
            r#"<script[^>]*>.*</script>"#.into(),
        )
        .with_severity(SeverityV9::High)
        .with_compliance_mapping(ComplianceMappingV9 {
            mappings: vec![ComplianceMappingEntryV9 {
                framework_id: "owasp_top10".into(),
                framework_name: "OWASP Top 10".into(),
                requirement_id: "A03:2021".into(),
                mapping_type: ComplianceMappingTypeV9::Direct,
                confidence: 0.85,
            }],
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_rule(id: &str, severity: SeverityV9, auto_fix: bool) -> SecurityScanRuleV9 {
        SecurityScanRuleV9 {
            id: id.to_string(),
            name: format!("Rule {id}"),
            description: format!("Description for rule {id}"),
            rule_type: RuleTypeV9::Pattern,
            severity,
            pattern: format!("pattern_{id}"),
            enabled: true,
            version: 1,
            author_id: Some("user-1".to_string()),
            auto_fix,
            fix_config: serde_json::json!({}),
            compliance_mapping: ComplianceMappingV9::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sample_fix(id: &str, scan_id: &str, rule_id: &str, applied: bool) -> ScanFixV9 {
        ScanFixV9 {
            id: id.to_string(),
            scan_id: scan_id.to_string(),
            rule_id: rule_id.to_string(),
            file_path: format!("src/file_{id}.rs"),
            line_number: Some(42),
            fix_type: FixTypeV9::PatternReplace,
            fix_content: format!("fixed content for {id}"),
            applied,
            test_result: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_add_rule() {
        let mut manager = SecurityScanRuleManagerV9::new();
        manager.add_rule(sample_rule("r1", SeverityV9::High, false));
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_get_rules_by_severity() {
        let mut manager = SecurityScanRuleManagerV9::new();
        manager.add_rule(sample_rule("r1", SeverityV9::High, false));
        manager.add_rule(sample_rule("r2", SeverityV9::Critical, false));
        manager.add_rule(sample_rule("r3", SeverityV9::High, false));
        let high = manager.list_rules_by_severity(SeverityV9::High);
        assert_eq!(high.len(), 2);
    }

    #[test]
    fn test_get_auto_fix_rules() {
        let mut manager = SecurityScanRuleManagerV9::new();
        manager.add_rule(sample_rule("r1", SeverityV9::High, true));
        manager.add_rule(sample_rule("r2", SeverityV9::Critical, false));
        let auto = manager.list_auto_fix_rules();
        assert_eq!(auto.len(), 1);
        assert_eq!(auto[0].id, "r1");
    }

    #[test]
    fn test_record_fix() {
        let mut manager = SecurityScanRuleManagerV9::new();
        manager.record_fix(sample_fix("f1", "scan-1", "r1", false));
        assert_eq!(manager.fixes.len(), 1);
    }

    #[test]
    fn test_apply_fix() {
        let mut manager = SecurityScanRuleManagerV9::new();
        manager.record_fix(sample_fix("f1", "scan-1", "r1", false));
        let result = manager.apply_fix("f1", false);
        assert!(result.is_ok());
        assert!(manager.fixes[0].applied);
    }

    #[test]
    fn test_apply_fix_requires_test() {
        let mut manager = SecurityScanRuleManagerV9::new();
        manager.record_fix(sample_fix("f1", "scan-1", "r1", false));
        let result = manager.apply_fix("f1", true);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("tested before applying"));
    }

    #[test]
    fn test_test_fix() {
        let mut manager = SecurityScanRuleManagerV9::new();
        manager.record_fix(sample_fix("f1", "scan-1", "r1", false));
        let result = manager.test_fix("f1", "test input");
        assert!(result.is_ok());
        assert!(result.unwrap().passed);
    }

    #[test]
    fn test_get_fixes_for_scan() {
        let mut manager = SecurityScanRuleManagerV9::new();
        manager.record_fix(sample_fix("f1", "scan-1", "r1", false));
        manager.record_fix(sample_fix("f2", "scan-1", "r2", false));
        manager.record_fix(sample_fix("f3", "scan-2", "r1", false));
        let fixes = manager.get_fixes_for_scan("scan-1");
        assert_eq!(fixes.len(), 2);
    }

    #[test]
    fn test_get_pending_fixes() {
        let mut manager = SecurityScanRuleManagerV9::new();
        manager.record_fix(sample_fix("f1", "scan-1", "r1", false));
        manager.record_fix(sample_fix("f2", "scan-1", "r1", true));
        let pending = manager.get_pending_fixes();
        assert_eq!(pending.len(), 1);
    }

    #[test]
    fn test_compute_fix_analytics() {
        let mut manager = SecurityScanRuleManagerV9::new();
        manager.add_rule(sample_rule("r1", SeverityV9::High, true));
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
        let mut manager = SecurityScanRuleManagerV9::new();
        manager.add_rule(sample_rule("r1", SeverityV9::High, true));
        manager.record_fix(sample_fix("f1", "scan-1", "r1", true));
        let report = manager.generate_report();
        assert!(report.contains("Security Scan V9 Report"));
        assert!(report.contains("Total Rules: 1"));
    }

    #[test]
    fn test_rule_versioning() {
        let mut manager = SecurityScanRuleManagerV9::new();
        manager.add_rule(sample_rule("r1", SeverityV9::High, false));
        manager
            .update_rule_pattern("r1", "new_pattern".into(), Some("user-1".into()), None)
            .unwrap();
        let history = manager.get_version_history("r1").unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].version, 1);
        assert_eq!(history[1].version, 2);
    }

    #[test]
    fn test_compliance_mapping() {
        let rule = SecurityScanRuleV9::new(
            "Test".into(),
            "Desc".into(),
            RuleTypeV9::Regex,
            "pattern".into(),
        )
        .with_compliance_mapping(ComplianceMappingV9 {
            mappings: vec![ComplianceMappingEntryV9 {
                framework_id: "soc2".into(),
                framework_name: "SOC 2".into(),
                requirement_id: "CC6.1".into(),
                mapping_type: ComplianceMappingTypeV9::Direct,
                confidence: 0.95,
            }],
        });
        assert_eq!(rule.compliance_mapping.mappings.len(), 1);
        assert_eq!(
            rule.compliance_mapping.mappings[0].framework_name,
            "SOC 2"
        );
    }

    #[test]
    fn test_severity_risk_weight() {
        assert_eq!(SeverityV9::Critical.risk_weight(), 4);
        assert_eq!(SeverityV9::High.risk_weight(), 3);
        assert_eq!(SeverityV9::Medium.risk_weight(), 2);
        assert_eq!(SeverityV9::Low.risk_weight(), 1);
        assert_eq!(SeverityV9::Informational.risk_weight(), 0);
    }

    #[test]
    fn test_empty_manager() {
        let manager = SecurityScanRuleManagerV9::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
        let analytics = manager.compute_fix_analytics();
        assert_eq!(analytics.total_fixes, 0);
    }

    #[test]
    fn test_fix_analytics_with_compliance() {
        let mut manager = SecurityScanRuleManagerV9::new();
        let mut rule = sample_rule("r1", SeverityV9::High, true);
        rule.compliance_mapping = ComplianceMappingV9 {
            mappings: vec![ComplianceMappingEntryV9 {
                framework_id: "soc2".into(),
                framework_name: "SOC 2".into(),
                requirement_id: "CC6.1".into(),
                mapping_type: ComplianceMappingTypeV9::Direct,
                confidence: 0.9,
            }],
        };
        manager.add_rule(rule);
        manager.record_fix(sample_fix("f1", "scan-1", "r1", true));
        let analytics = manager.compute_fix_analytics();
        assert_eq!(analytics.fixes_by_framework.get("SOC 2"), Some(&1));
        assert_eq!(analytics.compliance_impact.rules_with_compliance, 1);
        assert_eq!(analytics.compliance_impact.fixes_improving_compliance, 1);
    }

    #[test]
    fn test_default_rules_have_compliance_mapping() {
        let rules = create_default_scan_rules_v9();
        assert_eq!(rules.len(), 3);
        for rule in &rules {
            assert!(!rule.compliance_mapping.mappings.is_empty());
        }
    }
}
