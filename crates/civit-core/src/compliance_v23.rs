#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::compliance_v22::{
    ComplianceFindingV22, EvidenceTypeV22,
    RequirementSeverityV22,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRuleSetV23 {
    pub id: String,
    pub name: String,
    pub standard: String,
    pub rules: Vec<ComplianceRuleV23>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

impl ComplianceRuleSetV23 {
    pub fn new(name: String, standard: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            standard,
            rules: Vec::new(),
            enabled: true,
            created_at: Utc::now(),
        }
    }

    pub fn with_rules(mut self, rules: Vec<ComplianceRuleV23>) -> Self {
        self.rules = rules;
        self
    }

    pub fn add_rule(&mut self, rule: ComplianceRuleV23) {
        self.rules.push(rule);
    }

    pub fn enabled_rules(&self) -> Vec<&ComplianceRuleV23> {
        self.rules.iter().filter(|r| r.enabled).collect()
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRuleV23 {
    pub id: String,
    pub name: String,
    pub description: String,
    pub requirement_id: String,
    pub check_type: ComplianceCheckTypeV23,
    pub check_config: HashMap<String, serde_json::Value>,
    pub severity: RequirementSeverityV22,
    pub enabled: bool,
}

impl ComplianceRuleV23 {
    pub fn new(
        name: String,
        description: String,
        requirement_id: String,
        check_type: ComplianceCheckTypeV23,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            requirement_id,
            check_type,
            check_config: HashMap::new(),
            severity: RequirementSeverityV22::Medium,
            enabled: true,
        }
    }

    pub fn with_severity(mut self, severity: RequirementSeverityV22) -> Self {
        self.severity = severity;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceCheckTypeV23 {
    Automated,
    Manual,
    Hybrid,
    Inherited,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceEvidenceItemV23 {
    pub id: String,
    pub requirement_id: String,
    pub evidence_type: EvidenceTypeV22,
    pub evidence_data: HashMap<String, serde_json::Value>,
    pub verified: bool,
    pub verified_by: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl ComplianceEvidenceItemV23 {
    pub fn new(requirement_id: String, evidence_type: EvidenceTypeV22) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            requirement_id,
            evidence_type,
            evidence_data: HashMap::new(),
            verified: false,
            verified_by: None,
            verified_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_data(mut self, data: HashMap<String, serde_json::Value>) -> Self {
        self.evidence_data = data;
        self
    }

    pub fn verify(&mut self, user_id: &str) {
        self.verified = true;
        self.verified_by = Some(user_id.into());
        self.verified_at = Some(Utc::now());
    }

    pub fn unverify(&mut self) {
        self.verified = false;
        self.verified_by = None;
        self.verified_at = None;
    }

    pub fn is_verified(&self) -> bool {
        self.verified
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReadinessReportV23 {
    pub framework_id: String,
    pub framework_name: String,
    pub standard: String,
    pub total_requirements: u32,
    pub verified_evidence: u32,
    pub unverified_evidence: u32,
    pub missing_evidence: u32,
    pub readiness_score: f64,
    pub findings: Vec<ComplianceFindingV22>,
    pub generated_at: DateTime<Utc>,
}

impl AuditReadinessReportV23 {
    pub fn new(framework_id: String, framework_name: String, standard: String) -> Self {
        Self {
            framework_id,
            framework_name,
            standard,
            total_requirements: 0,
            verified_evidence: 0,
            unverified_evidence: 0,
            missing_evidence: 0,
            readiness_score: 0.0,
            findings: Vec::new(),
            generated_at: Utc::now(),
        }
    }

    pub fn calculate_readiness(&mut self) {
        self.readiness_score = if self.total_requirements == 0 {
            100.0
        } else {
            (self.verified_evidence as f64 / self.total_requirements as f64) * 100.0
        };
    }

    pub fn add_finding(&mut self, finding: ComplianceFindingV22) {
        self.findings.push(finding);
    }

    pub fn is_audit_ready(&self) -> bool {
        self.readiness_score >= 90.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceCollectionV23 {
    pub requirement_id: String,
    pub evidence_items: Vec<ComplianceEvidenceItemV23>,
    pub all_verified: bool,
    pub verification_rate: f64,
}

impl EvidenceCollectionV23 {
    pub fn new(requirement_id: String) -> Self {
        Self {
            requirement_id,
            evidence_items: Vec::new(),
            all_verified: false,
            verification_rate: 0.0,
        }
    }

    pub fn add_evidence(&mut self, evidence: ComplianceEvidenceItemV23) {
        self.evidence_items.push(evidence);
        self.recalculate();
    }

    pub fn recalculate(&mut self) {
        let total = self.evidence_items.len() as f64;
        let verified = self
            .evidence_items
            .iter()
            .filter(|e| e.is_verified())
            .count() as f64;
        self.all_verified = total > 0.0 && verified == total;
        self.verification_rate = if total > 0.0 {
            (verified / total) * 100.0
        } else {
            0.0
        };
    }

    pub fn verified_count(&self) -> usize {
        self.evidence_items
            .iter()
            .filter(|e| e.is_verified())
            .count()
    }

    pub fn unverified_count(&self) -> usize {
        self.evidence_items
            .iter()
            .filter(|e| !e.is_verified())
            .count()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplianceRuleSetManagerV23 {
    rule_sets: Vec<ComplianceRuleSetV23>,
}

impl ComplianceRuleSetManagerV23 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_rule_set(&mut self, rule_set: ComplianceRuleSetV23) {
        self.rule_sets.push(rule_set);
    }

    pub fn get_rule_set(&self, id: &str) -> Option<&ComplianceRuleSetV23> {
        self.rule_sets.iter().find(|rs| rs.id == id)
    }

    pub fn get_enabled_rule_sets(&self) -> Vec<&ComplianceRuleSetV23> {
        self.rule_sets.iter().filter(|rs| rs.enabled).collect()
    }

    pub fn list_rule_sets(&self) -> &[ComplianceRuleSetV23] {
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
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceCollectorV23 {
    collections: HashMap<String, EvidenceCollectionV23>,
}

impl EvidenceCollectorV23 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_evidence(
        &mut self,
        requirement_id: String,
        evidence: ComplianceEvidenceItemV23,
    ) {
        let rid = requirement_id.clone();
        self.collections
            .entry(requirement_id)
            .or_insert_with(|| EvidenceCollectionV23::new(rid))
            .add_evidence(evidence);
    }

    pub fn get_collection(&self, requirement_id: &str) -> Option<&EvidenceCollectionV23> {
        self.collections.get(requirement_id)
    }

    pub fn get_unverified_requirements(&self) -> Vec<&EvidenceCollectionV23> {
        self.collections
            .values()
            .filter(|c| !c.all_verified)
            .collect()
    }

    pub fn get_fully_verified_requirements(&self) -> Vec<&EvidenceCollectionV23> {
        self.collections
            .values()
            .filter(|c| c.all_verified)
            .collect()
    }

    pub fn overall_verification_rate(&self) -> f64 {
        let total: u32 = self
            .collections
            .values()
            .map(|c| c.evidence_items.len() as u32)
            .sum();
        let verified: u32 = self
            .collections
            .values()
            .map(|c| c.verified_count() as u32)
            .sum();
        if total == 0 {
            100.0
        } else {
            (verified as f64 / total as f64) * 100.0
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceVerificationEngineV23 {
    pending_verification: Vec<String>,
    verified: Vec<String>,
    rejected: Vec<String>,
}

impl EvidenceVerificationEngineV23 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn submit_for_verification(&mut self, evidence_id: &str) {
        if !self.pending_verification.contains(&evidence_id.to_string()) {
            self.pending_verification.push(evidence_id.into());
        }
    }

    pub fn approve(&mut self, evidence_id: &str) -> Result<(), String> {
        let idx = self
            .pending_verification
            .iter()
            .position(|id| id == evidence_id)
            .ok_or("Evidence not pending verification")?;
        self.pending_verification.remove(idx);
        self.verified.push(evidence_id.into());
        Ok(())
    }

    pub fn reject(&mut self, evidence_id: &str) -> Result<(), String> {
        let idx = self
            .pending_verification
            .iter()
            .position(|id| id == evidence_id)
            .ok_or("Evidence not pending verification")?;
        self.pending_verification.remove(idx);
        self.rejected.push(evidence_id.into());
        Ok(())
    }

    pub fn pending_count(&self) -> usize {
        self.pending_verification.len()
    }

    pub fn verified_count(&self) -> usize {
        self.verified.len()
    }

    pub fn rejected_count(&self) -> usize {
        self.rejected.len()
    }

    pub fn verification_rate(&self) -> f64 {
        let total = self.verified.len() + self.rejected.len();
        if total == 0 {
            100.0
        } else {
            (self.verified.len() as f64 / total as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_rule_set_v23_new() {
        let rs = ComplianceRuleSetV23::new("Test".into(), "SOC 2".into());
        assert_eq!(rs.name, "Test");
        assert_eq!(rs.standard, "SOC 2");
        assert!(rs.enabled);
    }

    #[test]
    fn test_compliance_rule_v23_new() {
        let rule = ComplianceRuleV23::new(
            "R1".into(),
            "Desc".into(),
            "req-1".into(),
            ComplianceCheckTypeV23::Automated,
        );
        assert_eq!(rule.name, "R1");
        assert!(rule.enabled);
    }

    #[test]
    fn test_compliance_evidence_item_v23_new() {
        let evidence =
            ComplianceEvidenceItemV23::new("req-1".into(), EvidenceTypeV22::Automated);
        assert!(!evidence.is_verified());
    }

    #[test]
    fn test_compliance_evidence_item_v23_verify() {
        let mut evidence =
            ComplianceEvidenceItemV23::new("req-1".into(), EvidenceTypeV22::Automated);
        evidence.verify("user-1");
        assert!(evidence.is_verified());
        assert_eq!(evidence.verified_by.as_deref(), Some("user-1"));
        assert!(evidence.verified_at.is_some());
    }

    #[test]
    fn test_compliance_evidence_item_v23_unverify() {
        let mut evidence =
            ComplianceEvidenceItemV23::new("req-1".into(), EvidenceTypeV22::Automated);
        evidence.verify("user-1");
        evidence.unverify();
        assert!(!evidence.is_verified());
    }

    #[test]
    fn test_evidence_collection_v23() {
        let mut collection = EvidenceCollectionV23::new("req-1".into());
        let e1 = ComplianceEvidenceItemV23::new("req-1".into(), EvidenceTypeV22::Automated);
        let mut e2 = ComplianceEvidenceItemV23::new("req-1".into(), EvidenceTypeV22::Manual);
        e2.verify("user-1");
        collection.add_evidence(e1);
        collection.add_evidence(e2);
        assert_eq!(collection.verified_count(), 1);
        assert_eq!(collection.unverified_count(), 1);
        assert!(!collection.all_verified);
        assert!((collection.verification_rate - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_audit_readiness_report_v23() {
        let mut report =
            AuditReadinessReportV23::new("fw-1".into(), "SOC 2".into(), "SOC2".into());
        report.total_requirements = 10;
        report.verified_evidence = 8;
        report.unverified_evidence = 2;
        report.calculate_readiness();
        assert!((report.readiness_score - 80.0).abs() < 0.01);
        assert!(!report.is_audit_ready());
    }

    #[test]
    fn test_audit_readiness_report_v23_ready() {
        let mut report =
            AuditReadinessReportV23::new("fw-1".into(), "SOC 2".into(), "SOC2".into());
        report.total_requirements = 10;
        report.verified_evidence = 10;
        report.calculate_readiness();
        assert!(report.is_audit_ready());
    }

    #[test]
    fn test_compliance_rule_set_manager_v23() {
        let mut manager = ComplianceRuleSetManagerV23::new();
        let rs = ComplianceRuleSetV23::new("Test".into(), "SOC 2".into());
        manager.add_rule_set(rs);
        assert_eq!(manager.list_rule_sets().len(), 1);
    }

    #[test]
    fn test_evidence_collector_v23() {
        let mut collector = EvidenceCollectorV23::new();
        let evidence = ComplianceEvidenceItemV23::new("req-1".into(), EvidenceTypeV22::Automated);
        collector.add_evidence("req-1".into(), evidence);
        assert!(collector.get_collection("req-1").is_some());
        assert_eq!(collector.overall_verification_rate(), 0.0);
    }

    #[test]
    fn test_evidence_verification_engine_v23() {
        let mut engine = EvidenceVerificationEngineV23::new();
        engine.submit_for_verification("e1");
        assert_eq!(engine.pending_count(), 1);
        engine.approve("e1").unwrap();
        assert_eq!(engine.verified_count(), 1);
        assert_eq!(engine.pending_count(), 0);
    }

    #[test]
    fn test_evidence_verification_engine_v23_reject() {
        let mut engine = EvidenceVerificationEngineV23::new();
        engine.submit_for_verification("e1");
        engine.reject("e1").unwrap();
        assert_eq!(engine.rejected_count(), 1);
        assert_eq!(engine.verification_rate(), 0.0);
    }

    #[test]
    fn test_evidence_verification_engine_v23_nonexistent() {
        let mut engine = EvidenceVerificationEngineV23::new();
        assert!(engine.approve("nonexistent").is_err());
        assert!(engine.reject("nonexistent").is_err());
    }
}
