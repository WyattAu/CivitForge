#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvidenceType {
    Document,
    Configuration,
    Log,
    Screenshot,
    Audit,
    Automated,
    Manual,
}

impl fmt::Display for EvidenceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Document => write!(f, "document"),
            Self::Configuration => write!(f, "configuration"),
            Self::Log => write!(f, "log"),
            Self::Screenshot => write!(f, "screenshot"),
            Self::Audit => write!(f, "audit"),
            Self::Automated => write!(f, "automated"),
            Self::Manual => write!(f, "manual"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComplianceScore {
    NotStarted,
    InProgress,
    PartiallyCompliant,
    Compliant,
    NonCompliant,
    NotApplicable,
}

impl fmt::Display for ComplianceScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotStarted => write!(f, "not_started"),
            Self::InProgress => write!(f, "in_progress"),
            Self::PartiallyCompliant => write!(f, "partially_compliant"),
            Self::Compliant => write!(f, "compliant"),
            Self::NonCompliant => write!(f, "non_compliant"),
            Self::NotApplicable => write!(f, "not_applicable"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkRequirement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub control_family: String,
    pub weight: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkV3 {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub requirements: Vec<FrameworkRequirement>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceEvidence {
    pub id: String,
    pub assessment_id: String,
    pub requirement_id: String,
    pub evidence_type: EvidenceType,
    pub evidence_data: serde_json::Value,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerificationTracking {
    pub total_evidence: u64,
    pub verified_count: u64,
    pub pending_count: u64,
    pub evidence_by_type: HashMap<EvidenceType, u64>,
    pub verification_rate: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplianceScoring {
    pub total_requirements: u64,
    pub met_requirements: u64,
    pub partial_requirements: u64,
    pub unmet_requirements: u64,
    pub compliance_percentage: f64,
    pub score_by_control_family: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct ComplianceManagerV3 {
    frameworks: Vec<ComplianceFrameworkV3>,
    evidence: Vec<ComplianceEvidence>,
}

impl Default for ComplianceManagerV3 {
    fn default() -> Self {
        Self::new()
    }
}

impl ComplianceManagerV3 {
    pub fn new() -> Self {
        Self {
            frameworks: Vec::new(),
            evidence: Vec::new(),
        }
    }

    pub fn add_framework(&mut self, framework: ComplianceFrameworkV3) {
        self.frameworks.push(framework);
    }

    pub fn get_framework_by_name(&self, name: &str) -> Option<&ComplianceFrameworkV3> {
        self.frameworks.iter().find(|f| f.name == name)
    }

    pub fn add_evidence(&mut self, evidence: ComplianceEvidence) {
        self.evidence.push(evidence);
    }

    pub fn verify_evidence(&mut self, evidence_id: &str) -> bool {
        if let Some(evidence) = self.evidence.iter_mut().find(|e| e.id == evidence_id) {
            evidence.verified = true;
            true
        } else {
            false
        }
    }

    pub fn get_evidence_for_assessment(&self, assessment_id: &str) -> Vec<&ComplianceEvidence> {
        self.evidence
            .iter()
            .filter(|e| e.assessment_id == assessment_id)
            .collect()
    }

    pub fn get_evidence_for_requirement(&self, requirement_id: &str) -> Vec<&ComplianceEvidence> {
        self.evidence
            .iter()
            .filter(|e| e.requirement_id == requirement_id)
            .collect()
    }

    pub fn compute_verification_tracking(&self) -> VerificationTracking {
        let total = self.evidence.len() as u64;
        let verified = self.evidence.iter().filter(|e| e.verified).count() as u64;
        let pending = total - verified;

        let mut evidence_by_type: HashMap<EvidenceType, u64> = HashMap::new();
        for evidence in &self.evidence {
            *evidence_by_type
                .entry(evidence.evidence_type.clone())
                .or_default() += 1;
        }

        let verification_rate = if total > 0 {
            (verified as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        VerificationTracking {
            total_evidence: total,
            verified_count: verified,
            pending_count: pending,
            evidence_by_type,
            verification_rate,
        }
    }

    pub fn compute_compliance_scoring(
        &self,
        met_requirements: &[String],
        partial_requirements: &[String],
    ) -> ComplianceScoring {
        let total_requirements: u64 = self
            .frameworks
            .iter()
            .filter(|f| f.enabled)
            .map(|f| f.requirements.len() as u64)
            .sum();

        let met = met_requirements.len() as u64;
        let partial = partial_requirements.len() as u64;
        let unmet = total_requirements.saturating_sub(met + partial);

        let compliance_percentage = if total_requirements > 0 {
            ((met as f64 + (partial as f64 * 0.5)) / total_requirements as f64) * 100.0
        } else {
            0.0
        };

        let mut score_by_control_family: HashMap<String, f64> = HashMap::new();
        for framework in &self.frameworks {
            if !framework.enabled {
                continue;
            }
            let family_groups: HashMap<String, Vec<&FrameworkRequirement>> = framework
                .requirements
                .iter()
                .fold(HashMap::new(), |mut acc, req| {
                    acc.entry(req.control_family.clone()).or_default().push(req);
                    acc
                });

            for (family, reqs) in family_groups {
                let family_met = reqs
                    .iter()
                    .filter(|r| met_requirements.contains(&r.id))
                    .count() as f64;
                let family_total = reqs.len() as f64;
                if family_total > 0.0 {
                    let score = (family_met / family_total) * 100.0;
                    *score_by_control_family.entry(family).or_insert(0.0) += score;
                }
            }
        }

        ComplianceScoring {
            total_requirements,
            met_requirements: met,
            partial_requirements: partial,
            unmet_requirements: unmet,
            compliance_percentage,
            score_by_control_family,
        }
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::from("=== Compliance Frameworks V3 Report ===\n\n");
        report.push_str(&format!("Total Frameworks: {}\n", self.frameworks.len()));
        report.push_str(&format!(
            "Active Frameworks: {}\n",
            self.frameworks.iter().filter(|f| f.enabled).count()
        ));

        let tracking = self.compute_verification_tracking();
        report.push_str(&format!("\nEvidence Tracking:\n"));
        report.push_str(&format!("  Total Evidence: {}\n", tracking.total_evidence));
        report.push_str(&format!("  Verified: {}\n", tracking.verified_count));
        report.push_str(&format!("  Pending: {}\n", tracking.pending_count));
        report.push_str(&format!(
            "  Verification Rate: {:.1}%\n",
            tracking.verification_rate
        ));

        report.push_str("\nFrameworks:\n");
        for framework in &self.frameworks {
            report.push_str(&format!(
                "  {} v{}: {} requirements (enabled={})\n",
                framework.name,
                framework.version,
                framework.requirements.len(),
                framework.enabled
            ));
        }

        report
    }

    pub fn frameworks(&self) -> &[ComplianceFrameworkV3] {
        &self.frameworks
    }

    pub fn evidence(&self) -> &[ComplianceEvidence] {
        &self.evidence
    }

    pub fn len(&self) -> usize {
        self.frameworks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frameworks.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_framework(id: &str, name: &str, version: &str) -> ComplianceFrameworkV3 {
        ComplianceFrameworkV3 {
            id: id.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            description: format!("Framework {name}"),
            requirements: vec![
                FrameworkRequirement {
                    id: format!("{name}-1"),
                    title: "Req 1".to_string(),
                    description: "Description 1".to_string(),
                    control_family: "Access Control".to_string(),
                    weight: 1,
                },
                FrameworkRequirement {
                    id: format!("{name}-2"),
                    title: "Req 2".to_string(),
                    description: "Description 2".to_string(),
                    control_family: "Logging".to_string(),
                    weight: 1,
                },
            ],
            enabled: true,
            created_at: Utc::now(),
        }
    }

    fn sample_evidence(id: &str, assessment_id: &str, requirement_id: &str) -> ComplianceEvidence {
        ComplianceEvidence {
            id: id.to_string(),
            assessment_id: assessment_id.to_string(),
            requirement_id: requirement_id.to_string(),
            evidence_type: EvidenceType::Document,
            evidence_data: serde_json::json!({"file": "policy.pdf"}),
            verified: false,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_add_framework() {
        let mut manager = ComplianceManagerV3::new();
        manager.add_framework(sample_framework("f1", "ISO27001", "2.0"));
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_get_framework_by_name() {
        let mut manager = ComplianceManagerV3::new();
        manager.add_framework(sample_framework("f1", "ISO27001", "2.0"));
        manager.add_framework(sample_framework("f2", "SOC2", "1.0"));
        assert!(manager.get_framework_by_name("ISO27001").is_some());
        assert!(manager.get_framework_by_name("NonExistent").is_none());
    }

    #[test]
    fn test_add_evidence() {
        let mut manager = ComplianceManagerV3::new();
        manager.add_evidence(sample_evidence("e1", "a1", "ISO27001-1"));
        assert_eq!(manager.evidence().len(), 1);
    }

    #[test]
    fn test_verify_evidence() {
        let mut manager = ComplianceManagerV3::new();
        manager.add_evidence(sample_evidence("e1", "a1", "ISO27001-1"));
        assert!(manager.verify_evidence("e1"));
        assert!(manager.evidence()[0].verified);
        assert!(!manager.verify_evidence("nonexistent"));
    }

    #[test]
    fn test_get_evidence_for_assessment() {
        let mut manager = ComplianceManagerV3::new();
        manager.add_evidence(sample_evidence("e1", "a1", "ISO27001-1"));
        manager.add_evidence(sample_evidence("e2", "a1", "ISO27001-2"));
        manager.add_evidence(sample_evidence("e3", "a2", "ISO27001-1"));
        let evidence = manager.get_evidence_for_assessment("a1");
        assert_eq!(evidence.len(), 2);
    }

    #[test]
    fn test_get_evidence_for_requirement() {
        let mut manager = ComplianceManagerV3::new();
        manager.add_evidence(sample_evidence("e1", "a1", "ISO27001-1"));
        manager.add_evidence(sample_evidence("e2", "a1", "ISO27001-1"));
        manager.add_evidence(sample_evidence("e3", "a1", "ISO27001-2"));
        let evidence = manager.get_evidence_for_requirement("ISO27001-1");
        assert_eq!(evidence.len(), 2);
    }

    #[test]
    fn test_compute_verification_tracking() {
        let mut manager = ComplianceManagerV3::new();
        manager.add_evidence(sample_evidence("e1", "a1", "ISO27001-1"));
        manager.add_evidence(sample_evidence("e2", "a1", "ISO27001-1"));
        manager.verify_evidence("e1");
        let tracking = manager.compute_verification_tracking();
        assert_eq!(tracking.total_evidence, 2);
        assert_eq!(tracking.verified_count, 1);
        assert_eq!(tracking.pending_count, 1);
        assert_eq!(tracking.verification_rate, 50.0);
    }

    #[test]
    fn test_compute_compliance_scoring() {
        let mut manager = ComplianceManagerV3::new();
        manager.add_framework(sample_framework("f1", "ISO27001", "2.0"));
        let scoring =
            manager.compute_compliance_scoring(&["ISO27001-1".to_string()], &[]);
        assert_eq!(scoring.total_requirements, 2);
        assert_eq!(scoring.met_requirements, 1);
        assert_eq!(scoring.partial_requirements, 0);
        assert_eq!(scoring.unmet_requirements, 1);
        assert_eq!(scoring.compliance_percentage, 50.0);
    }

    #[test]
    fn test_generate_report() {
        let mut manager = ComplianceManagerV3::new();
        manager.add_framework(sample_framework("f1", "ISO27001", "2.0"));
        let report = manager.generate_report();
        assert!(report.contains("Compliance Frameworks V3 Report"));
        assert!(report.contains("Total Frameworks: 1"));
    }

    #[test]
    fn test_empty_manager() {
        let manager = ComplianceManagerV3::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
        let tracking = manager.compute_verification_tracking();
        assert_eq!(tracking.total_evidence, 0);
    }
}
