#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScanRuleSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for ScanRuleSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

impl ScanRuleSeverity {
    pub fn risk_weight(&self) -> u32 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FixType {
    PatternReplace,
    LineRemoval,
    LineInsertion,
    ConfigPatch,
    Custom,
}

impl fmt::Display for FixType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PatternReplace => write!(f, "pattern_replace"),
            Self::LineRemoval => write!(f, "line_removal"),
            Self::LineInsertion => write!(f, "line_insertion"),
            Self::ConfigPatch => write!(f, "config_patch"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRuleV3 {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_type: String,
    pub severity: ScanRuleSeverity,
    pub pattern: String,
    pub enabled: bool,
    pub version: u32,
    pub author_id: Option<String>,
    pub auto_fix: bool,
    pub fix_config: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

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
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FixAnalytics {
    pub total_fixes: u64,
    pub applied_fixes: u64,
    pub failed_fixes: u64,
    pub fixes_by_type: HashMap<FixType, u64>,
    pub fixes_by_severity: HashMap<ScanRuleSeverity, u64>,
    pub avg_fix_success_rate: f64,
}

#[derive(Debug, Clone)]
pub struct SecurityScannerV3 {
    rules: Vec<ScanRuleV3>,
    fixes: Vec<ScanFix>,
}

impl Default for SecurityScannerV3 {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityScannerV3 {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            fixes: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, rule: ScanRuleV3) {
        self.rules.push(rule);
    }

    pub fn get_rules_by_severity(&self, severity: &ScanRuleSeverity) -> Vec<&ScanRuleV3> {
        self.rules
            .iter()
            .filter(|r| &r.severity == severity && r.enabled)
            .collect()
    }

    pub fn get_auto_fix_rules(&self) -> Vec<&ScanRuleV3> {
        self.rules
            .iter()
            .filter(|r| r.auto_fix && r.enabled)
            .collect()
    }

    pub fn record_fix(&mut self, fix: ScanFix) {
        self.fixes.push(fix);
    }

    pub fn apply_fix(&mut self, fix_id: &str) -> bool {
        if let Some(fix) = self.fixes.iter_mut().find(|f| f.id == fix_id) {
            fix.applied = true;
            true
        } else {
            false
        }
    }

    pub fn get_fixes_for_scan(&self, scan_id: &str) -> Vec<&ScanFix> {
        self.fixes
            .iter()
            .filter(|f| f.scan_id == scan_id)
            .collect()
    }

    pub fn get_pending_fixes(&self) -> Vec<&ScanFix> {
        self.fixes.iter().filter(|f| !f.applied).collect()
    }

    pub fn compute_fix_analytics(&self) -> FixAnalytics {
        let total = self.fixes.len() as u64;
        let applied = self.fixes.iter().filter(|f| f.applied).count() as u64;
        let failed = total - applied;

        let mut fixes_by_type: HashMap<FixType, u64> = HashMap::new();
        let mut fixes_by_severity: HashMap<ScanRuleSeverity, u64> = HashMap::new();

        for fix in &self.fixes {
            *fixes_by_type.entry(fix.fix_type.clone()).or_default() += 1;
        }

        for rule in &self.rules {
            if self.fixes.iter().any(|f| f.rule_id == rule.id) {
                *fixes_by_severity
                    .entry(rule.severity.clone())
                    .or_default() += 1;
            }
        }

        let avg_success_rate = if total > 0 {
            (applied as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        FixAnalytics {
            total_fixes: total,
            applied_fixes: applied,
            failed_fixes: failed,
            fixes_by_type,
            fixes_by_severity,
            avg_fix_success_rate: avg_success_rate,
        }
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::from("=== Security Scan V3 Report ===\n\n");
        report.push_str(&format!("Total Rules: {}\n", self.rules.len()));
        report.push_str(&format!("Active Rules: {}\n", self.rules.iter().filter(|r| r.enabled).count()));
        report.push_str(&format!("Auto-Fix Rules: {}\n", self.get_auto_fix_rules().len()));
        report.push('\n');

        report.push_str("Rules by Severity:\n");
        for severity in &[
            ScanRuleSeverity::Critical,
            ScanRuleSeverity::High,
            ScanRuleSeverity::Medium,
            ScanRuleSeverity::Low,
        ] {
            let count = self.get_rules_by_severity(severity).len();
            report.push_str(&format!("  {}: {}\n", severity, count));
        }
        report.push('\n');

        let analytics = self.compute_fix_analytics();
        report.push_str(&format!("Fix Analytics:\n"));
        report.push_str(&format!("  Total Fixes: {}\n", analytics.total_fixes));
        report.push_str(&format!("  Applied: {}\n", analytics.applied_fixes));
        report.push_str(&format!("  Failed: {}\n", analytics.failed_fixes));
        report.push_str(&format!("  Success Rate: {:.1}%\n", analytics.avg_fix_success_rate));

        report
    }

    pub fn rules(&self) -> &[ScanRuleV3] {
        &self.rules
    }

    pub fn fixes(&self) -> &[ScanFix] {
        &self.fixes
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_rule(id: &str, severity: ScanRuleSeverity, auto_fix: bool) -> ScanRuleV3 {
        ScanRuleV3 {
            id: id.to_string(),
            name: format!("Rule {id}"),
            description: format!("Description for rule {id}"),
            rule_type: "pattern".to_string(),
            severity,
            pattern: format!("pattern_{id}"),
            enabled: true,
            version: 1,
            author_id: Some("user-1".to_string()),
            auto_fix,
            fix_config: serde_json::json!({}),
            created_at: Utc::now(),
        }
    }

    fn sample_fix(id: &str, scan_id: &str, rule_id: &str, applied: bool) -> ScanFix {
        ScanFix {
            id: id.to_string(),
            scan_id: scan_id.to_string(),
            rule_id: rule_id.to_string(),
            file_path: format!("src/file_{id}.rs"),
            line_number: Some(42),
            fix_type: FixType::PatternReplace,
            fix_content: format!("fixed content for {id}"),
            applied,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_add_rule() {
        let mut scanner = SecurityScannerV3::new();
        scanner.add_rule(sample_rule("r1", ScanRuleSeverity::High, false));
        assert_eq!(scanner.len(), 1);
    }

    #[test]
    fn test_get_rules_by_severity() {
        let mut scanner = SecurityScannerV3::new();
        scanner.add_rule(sample_rule("r1", ScanRuleSeverity::High, false));
        scanner.add_rule(sample_rule("r2", ScanRuleSeverity::Critical, false));
        scanner.add_rule(sample_rule("r3", ScanRuleSeverity::High, false));
        let high = scanner.get_rules_by_severity(&ScanRuleSeverity::High);
        assert_eq!(high.len(), 2);
    }

    #[test]
    fn test_get_auto_fix_rules() {
        let mut scanner = SecurityScannerV3::new();
        scanner.add_rule(sample_rule("r1", ScanRuleSeverity::High, true));
        scanner.add_rule(sample_rule("r2", ScanRuleSeverity::Critical, false));
        let auto = scanner.get_auto_fix_rules();
        assert_eq!(auto.len(), 1);
        assert_eq!(auto[0].id, "r1");
    }

    #[test]
    fn test_record_fix() {
        let mut scanner = SecurityScannerV3::new();
        scanner.record_fix(sample_fix("f1", "scan-1", "r1", false));
        assert_eq!(scanner.fixes().len(), 1);
    }

    #[test]
    fn test_apply_fix() {
        let mut scanner = SecurityScannerV3::new();
        scanner.record_fix(sample_fix("f1", "scan-1", "r1", false));
        assert!(scanner.apply_fix("f1"));
        assert!(scanner.fixes()[0].applied);
        assert!(!scanner.apply_fix("nonexistent"));
    }

    #[test]
    fn test_get_fixes_for_scan() {
        let mut scanner = SecurityScannerV3::new();
        scanner.record_fix(sample_fix("f1", "scan-1", "r1", false));
        scanner.record_fix(sample_fix("f2", "scan-1", "r2", false));
        scanner.record_fix(sample_fix("f3", "scan-2", "r1", false));
        let fixes = scanner.get_fixes_for_scan("scan-1");
        assert_eq!(fixes.len(), 2);
    }

    #[test]
    fn test_get_pending_fixes() {
        let mut scanner = SecurityScannerV3::new();
        scanner.record_fix(sample_fix("f1", "scan-1", "r1", false));
        scanner.record_fix(sample_fix("f2", "scan-1", "r1", true));
        let pending = scanner.get_pending_fixes();
        assert_eq!(pending.len(), 1);
    }

    #[test]
    fn test_compute_fix_analytics() {
        let mut scanner = SecurityScannerV3::new();
        scanner.add_rule(sample_rule("r1", ScanRuleSeverity::High, true));
        scanner.record_fix(sample_fix("f1", "scan-1", "r1", true));
        scanner.record_fix(sample_fix("f2", "scan-1", "r1", false));
        let analytics = scanner.compute_fix_analytics();
        assert_eq!(analytics.total_fixes, 2);
        assert_eq!(analytics.applied_fixes, 1);
        assert_eq!(analytics.failed_fixes, 1);
        assert_eq!(analytics.avg_fix_success_rate, 50.0);
    }

    #[test]
    fn test_generate_report() {
        let mut scanner = SecurityScannerV3::new();
        scanner.add_rule(sample_rule("r1", ScanRuleSeverity::High, true));
        scanner.record_fix(sample_fix("f1", "scan-1", "r1", true));
        let report = scanner.generate_report();
        assert!(report.contains("Security Scan V3 Report"));
        assert!(report.contains("Total Rules: 1"));
    }

    #[test]
    fn test_severity_risk_weight() {
        assert_eq!(ScanRuleSeverity::Low.risk_weight(), 1);
        assert_eq!(ScanRuleSeverity::Medium.risk_weight(), 2);
        assert_eq!(ScanRuleSeverity::High.risk_weight(), 3);
        assert_eq!(ScanRuleSeverity::Critical.risk_weight(), 4);
    }

    #[test]
    fn test_empty_scanner() {
        let scanner = SecurityScannerV3::new();
        assert!(scanner.is_empty());
        assert_eq!(scanner.len(), 0);
        let analytics = scanner.compute_fix_analytics();
        assert_eq!(analytics.total_fixes, 0);
    }
}
