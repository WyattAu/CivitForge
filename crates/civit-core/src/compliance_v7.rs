#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirementRecordV7 {
    pub id: String,
    pub framework_id: String,
    pub requirement_id: String,
    pub description: String,
    pub severity: RequirementSeverityV7,
    pub automated_check: bool,
    pub check_config: HashMap<String, serde_json::Value>,
    pub evidence_config: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ComplianceRequirementRecordV7 {
    pub fn new(
        framework_id: String,
        requirement_id: String,
        description: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            framework_id,
            requirement_id,
            description,
            severity: RequirementSeverityV7::Medium,
            automated_check: false,
            check_config: HashMap::new(),
            evidence_config: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_severity(mut self, severity: RequirementSeverityV7) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_automated_check(mut self, config: HashMap<String, serde_json::Value>) -> Self {
        self.automated_check = true;
        self.check_config = config;
        self
    }

    pub fn with_evidence_config(mut self, config: HashMap<String, serde_json::Value>) -> Self {
        self.evidence_config = config;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementSeverityV7 {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceEvidenceV7 {
    pub id: String,
    pub requirement_id: String,
    pub assessment_id: String,
    pub evidence_type: EvidenceTypeV7,
    pub content: HashMap<String, serde_json::Value>,
    pub collected_by: Option<String>,
    pub collected_at: DateTime<Utc>,
}

impl ComplianceEvidenceV7 {
    pub fn new(
        requirement_id: String,
        assessment_id: String,
        evidence_type: EvidenceTypeV7,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            requirement_id,
            assessment_id,
            evidence_type,
            content: HashMap::new(),
            collected_by: None,
            collected_at: Utc::now(),
        }
    }

    pub fn with_content(mut self, content: HashMap<String, serde_json::Value>) -> Self {
        self.content = content;
        self
    }

    pub fn with_collected_by(mut self, user_id: &str) -> Self {
        self.collected_by = Some(user_id.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceTypeV7 {
    Manual,
    Automated,
    SystemGenerated,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheckResultV7 {
    pub id: String,
    pub requirement_id: String,
    pub assessment_id: String,
    pub status: CheckStatusV7,
    pub result_details: HashMap<String, serde_json::Value>,
    pub score: u32,
    pub executed_at: DateTime<Utc>,
}

impl ComplianceCheckResultV7 {
    pub fn new(
        requirement_id: String,
        assessment_id: String,
        status: CheckStatusV7,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            requirement_id,
            assessment_id,
            status,
            result_details: HashMap::new(),
            score: 0,
            executed_at: Utc::now(),
        }
    }

    pub fn with_score(mut self, score: u32) -> Self {
        self.score = score;
        self
    }

    pub fn with_details(mut self, details: HashMap<String, serde_json::Value>) -> Self {
        self.result_details = details;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatusV7 {
    Pending,
    Running,
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkV9 {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub requirements: Vec<ComplianceRequirementV7>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ComplianceFrameworkV9 {
    pub fn new(name: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            version: "7.0".into(),
            description,
            requirements: Vec::new(),
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_version(mut self, version: &str) -> Self {
        self.version = version.into();
        self
    }

    pub fn add_requirement(&mut self, requirement: ComplianceRequirementV7) {
        self.requirements.push(requirement);
    }

    pub fn mandatory_requirements(&self) -> Vec<&ComplianceRequirementV7> {
        self.requirements.iter().filter(|r| r.mandatory).collect()
    }

    pub fn requirements_by_category(&self, category: &str) -> Vec<&ComplianceRequirementV7> {
        self.requirements
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn increment_version(&mut self) {
        let parts: Vec<&str> = self.version.split('.').collect();
        if parts.len() == 2 {
            if let Ok(minor) = parts[1].parse::<u32>() {
                self.version = format!("{}.{}", parts[0], minor + 1);
            }
        }
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirementV7 {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub mandatory: bool,
    pub verification_method: VerificationMethodV7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationMethodV7 {
    Automated,
    Manual,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentStatusV7 {
    Pending,
    Running,
    Passed,
    Failed,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAssessmentV7 {
    pub id: String,
    pub framework_id: String,
    pub repo_id: Option<String>,
    pub status: AssessmentStatusV7,
    pub findings: Vec<ComplianceFindingV7>,
    pub score: u32,
    pub assessor_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub history: Vec<AssessmentSnapshotV7>,
}

impl ComplianceAssessmentV7 {
    pub fn new(framework_id: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            framework_id,
            repo_id: None,
            status: AssessmentStatusV7::Pending,
            findings: Vec::new(),
            score: 0,
            assessor_id: None,
            started_at: Utc::now(),
            completed_at: None,
            history: Vec::new(),
        }
    }

    pub fn with_repo(mut self, repo_id: &str) -> Self {
        self.repo_id = Some(repo_id.into());
        self
    }

    pub fn with_assessor(mut self, assessor_id: &str) -> Self {
        self.assessor_id = Some(assessor_id.into());
        self
    }

    pub fn add_finding(&mut self, finding: ComplianceFindingV7) {
        self.findings.push(finding);
        self.snapshot_history();
    }

    pub fn complete(&mut self, score: u32) {
        self.status = if self.findings.iter().any(|f| f.status == FindingStatusV7::NonCompliant) {
            AssessmentStatusV7::Failed
        } else if self.findings.iter().any(|f| f.status == FindingStatusV7::Partial) {
            AssessmentStatusV7::Partial
        } else {
            AssessmentStatusV7::Passed
        };
        self.score = score;
        self.completed_at = Some(Utc::now());
        self.snapshot_history();
    }

    fn snapshot_history(&mut self) {
        self.history.push(AssessmentSnapshotV7 {
            status: self.status,
            score: self.score,
            findings_count: self.findings.len() as u32,
            timestamp: Utc::now(),
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentSnapshotV7 {
    pub status: AssessmentStatusV7,
    pub score: u32,
    pub findings_count: u32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFindingV7 {
    pub id: String,
    pub requirement_id: String,
    pub status: FindingStatusV7,
    pub details: String,
    pub evidence: Option<String>,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatusV7 {
    Compliant,
    NonCompliant,
    Partial,
    NotApplicable,
}

impl ComplianceFindingV7 {
    pub fn is_compliant(&self) -> bool {
        self.status == FindingStatusV7::Compliant || self.status == FindingStatusV7::NotApplicable
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingTrackerV9 {
    pub findings: Vec<ComplianceFindingV7>,
    pub remediation_deadlines: HashMap<String, DateTime<Utc>>,
    pub assigned_to: HashMap<String, String>,
    pub severity_scores: HashMap<String, u32>,
}

impl FindingTrackerV9 {
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
            remediation_deadlines: HashMap::new(),
            assigned_to: HashMap::new(),
            severity_scores: HashMap::new(),
        }
    }

    pub fn add_finding(&mut self, finding: ComplianceFindingV7) {
        self.findings.push(finding);
    }

    pub fn set_deadline(&mut self, finding_id: &str, deadline: DateTime<Utc>) {
        self.remediation_deadlines.insert(finding_id.into(), deadline);
    }

    pub fn assign(&mut self, finding_id: &str, user_id: &str) {
        self.assigned_to.insert(finding_id.into(), user_id.into());
    }

    pub fn overdue_findings(&self) -> Vec<&ComplianceFindingV7> {
        let now = Utc::now();
        self.findings
            .iter()
            .filter(|f| {
                self.remediation_deadlines
                    .get(&f.id)
                    .map(|d| *d < now)
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn open_findings(&self) -> Vec<&ComplianceFindingV7> {
        self.findings
            .iter()
            .filter(|f| f.status == FindingStatusV7::NonCompliant || f.status == FindingStatusV7::Partial)
            .collect()
    }

    pub fn compute_severity_scores(&mut self) -> HashMap<String, u32> {
        let mut scores: HashMap<String, u32> = HashMap::new();
        for finding in &self.findings {
            let category = finding.requirement_id.split('.').next().unwrap_or("unknown").to_string();
            *scores.entry(category).or_insert(0) += match finding.status {
                FindingStatusV7::NonCompliant => 10,
                FindingStatusV7::Partial => 5,
                FindingStatusV7::Compliant => 0,
                FindingStatusV7::NotApplicable => 0,
            };
        }
        self.severity_scores = scores.clone();
        scores
    }
}

impl Default for FindingTrackerV9 {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceScoreV9 {
    pub overall: u32,
    pub by_severity: HashMap<String, (u32, u32)>,
    pub total_requirements: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
}

pub struct ComplianceScoringEngineV4;

impl ComplianceScoringEngineV4 {
    pub fn calculate_framework_score(
        requirements: &[ComplianceRequirementRecordV7],
        check_results: &[ComplianceCheckResultV7],
    ) -> ComplianceScoreV9 {
        let total = requirements.len() as u32;
        if total == 0 {
            return ComplianceScoreV9 {
                overall: 100,
                by_severity: HashMap::new(),
                total_requirements: 0,
                passed: 0,
                failed: 0,
                skipped: 0,
            };
        }

        let mut passed = 0u32;
        let mut failed = 0u32;
        let mut skipped = 0u32;

        for result in check_results {
            match result.status {
                CheckStatusV7::Passed => passed += 1,
                CheckStatusV7::Failed => failed += 1,
                CheckStatusV7::Skipped => skipped += 1,
                _ => {}
            }
        }

        let applicable = total - skipped;
        let overall = if applicable > 0 {
            ((passed as f64 / applicable as f64) * 100.0) as u32
        } else {
            100
        };

        ComplianceScoreV9 {
            overall,
            by_severity: HashMap::new(),
            total_requirements: total,
            passed,
            failed,
            skipped,
        }
    }

    pub fn calculate_weighted_score(
        requirements: &[ComplianceRequirementRecordV7],
        check_results: &[ComplianceCheckResultV7],
    ) -> f64 {
        let total_weight: f64 = requirements.iter().map(|r| {
            match r.severity {
                RequirementSeverityV7::Critical => 4.0,
                RequirementSeverityV7::High => 3.0,
                RequirementSeverityV7::Medium => 2.0,
                RequirementSeverityV7::Low => 1.0,
            }
        }).sum();

        if total_weight == 0.0 {
            return 100.0;
        }

        let earned_weight: f64 = requirements.iter().zip(check_results.iter()).map(|(req, res)| {
            let weight = match req.severity {
                RequirementSeverityV7::Critical => 4.0,
                RequirementSeverityV7::High => 3.0,
                RequirementSeverityV7::Medium => 2.0,
                RequirementSeverityV7::Low => 1.0,
            };
            if res.status == CheckStatusV7::Passed {
                weight
            } else {
                0.0
            }
        }).sum();

        (earned_weight / total_weight) * 100.0
    }
}

pub struct ComplianceAssessorV4;

impl ComplianceAssessorV4 {
    pub fn assess(
        framework: &ComplianceFrameworkV9,
        findings: &[ComplianceFindingV7],
    ) -> AssessmentResultV4 {
        let total_requirements = framework.requirements.len() as u32;
        if total_requirements == 0 {
            return AssessmentResultV4 {
                score: 100,
                status: AssessmentStatusV7::Passed,
                compliant_count: 0,
                non_compliant_count: 0,
                partial_count: 0,
                not_applicable_count: 0,
            };
        }

        let mut compliant = 0u32;
        let mut non_compliant = 0u32;
        let mut partial = 0u32;
        let mut not_applicable = 0u32;

        for finding in findings {
            match finding.status {
                FindingStatusV7::Compliant => compliant += 1,
                FindingStatusV7::NonCompliant => non_compliant += 1,
                FindingStatusV7::Partial => partial += 1,
                FindingStatusV7::NotApplicable => not_applicable += 1,
            }
        }

        let applicable = total_requirements - not_applicable;
        let score = if applicable > 0 {
            ((compliant as f64 / applicable as f64) * 100.0) as u32
        } else {
            100
        };

        let status = if non_compliant > 0 {
            AssessmentStatusV7::Failed
        } else if partial > 0 {
            AssessmentStatusV7::Partial
        } else {
            AssessmentStatusV7::Passed
        };

        AssessmentResultV4 {
            score,
            status,
            compliant_count: compliant,
            non_compliant_count: non_compliant,
            partial_count: partial,
            not_applicable_count: not_applicable,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentResultV4 {
    pub score: u32,
    pub status: AssessmentStatusV7,
    pub compliant_count: u32,
    pub non_compliant_count: u32,
    pub partial_count: u32,
    pub not_applicable_count: u32,
}

pub fn create_soc2_framework_v9() -> ComplianceFrameworkV9 {
    let mut framework = ComplianceFrameworkV9::new(
        "SOC 2".into(),
        "Service Organization Control 2".into(),
    );
    framework.add_requirement(ComplianceRequirementV7 {
        id: "CC6.1".into(),
        name: "Logical Access Controls".into(),
        description: "Implement logical access security controls".into(),
        category: "Security".into(),
        mandatory: true,
        verification_method: VerificationMethodV7::Automated,
    });
    framework.add_requirement(ComplianceRequirementV7 {
        id: "CC6.6".into(),
        name: "System Boundaries".into(),
        description: "Restrict logical access to system boundaries".into(),
        category: "Security".into(),
        mandatory: true,
        verification_method: VerificationMethodV7::Hybrid,
    });
    framework.add_requirement(ComplianceRequirementV7 {
        id: "CC7.1".into(),
        name: "Vulnerability Management".into(),
        description: "Detect and monitor for vulnerabilities".into(),
        category: "Security".into(),
        mandatory: true,
        verification_method: VerificationMethodV7::Automated,
    });
    framework.add_requirement(ComplianceRequirementV7 {
        id: "CC8.1".into(),
        name: "Change Management".into(),
        description: "Authorize and manage changes to systems".into(),
        category: "Availability".into(),
        mandatory: true,
        verification_method: VerificationMethodV7::Manual,
    });
    framework
}

pub fn create_iso27001_framework_v9() -> ComplianceFrameworkV9 {
    let mut framework = ComplianceFrameworkV9::new(
        "ISO 27001".into(),
        "Information Security Management System".into(),
    );
    framework.add_requirement(ComplianceRequirementV7 {
        id: "A.12.6.1".into(),
        name: "Technical Vulnerability Management".into(),
        description: "Obtain information about technical vulnerabilities".into(),
        category: "Operations Security".into(),
        mandatory: true,
        verification_method: VerificationMethodV7::Automated,
    });
    framework.add_requirement(ComplianceRequirementV7 {
        id: "A.14.2.1".into(),
        name: "Secure Development Policy".into(),
        description: "Establish secure development lifecycle".into(),
        category: "Acquisition Development".into(),
        mandatory: true,
        verification_method: VerificationMethodV7::Hybrid,
    });
    framework.add_requirement(ComplianceRequirementV7 {
        id: "A.18.2.1".into(),
        name: "Independent Review".into(),
        description: "Independent review of organization's ISMS".into(),
        category: "Compliance".into(),
        mandatory: true,
        verification_method: VerificationMethodV7::Manual,
    });
    framework
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_finding(requirement_id: &str, status: FindingStatusV7) -> ComplianceFindingV7 {
        ComplianceFindingV7 {
            id: uuid::Uuid::new_v4().to_string(),
            requirement_id: requirement_id.into(),
            status,
            details: "Test finding".into(),
            evidence: None,
            remediation: None,
        }
    }

    #[test]
    fn test_compliance_framework_v9_new() {
        let fw = ComplianceFrameworkV9::new("Test".into(), "Description".into());
        assert_eq!(fw.name, "Test");
        assert!(fw.enabled);
        assert!(fw.requirements.is_empty());
        assert_eq!(fw.version, "7.0");
    }

    #[test]
    fn test_compliance_framework_v9_with_version() {
        let fw = ComplianceFrameworkV9::new("Test".into(), "Desc".into())
            .with_version("8.0");
        assert_eq!(fw.version, "8.0");
    }

    #[test]
    fn test_compliance_framework_v9_increment_version() {
        let mut fw = ComplianceFrameworkV9::new("Test".into(), "Desc".into());
        fw.increment_version();
        assert_eq!(fw.version, "7.1");
        fw.increment_version();
        assert_eq!(fw.version, "7.2");
    }

    #[test]
    fn test_add_requirement() {
        let mut fw = ComplianceFrameworkV9::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV7 {
            id: "R1".into(),
            name: "Req 1".into(),
            description: "Desc".into(),
            category: "Security".into(),
            mandatory: true,
            verification_method: VerificationMethodV7::Automated,
        });
        assert_eq!(fw.requirements.len(), 1);
    }

    #[test]
    fn test_mandatory_requirements() {
        let mut fw = ComplianceFrameworkV9::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV7 {
            id: "R1".into(),
            name: "Mandatory".into(),
            description: "".into(),
            category: "Security".into(),
            mandatory: true,
            verification_method: VerificationMethodV7::Automated,
        });
        fw.add_requirement(ComplianceRequirementV7 {
            id: "R2".into(),
            name: "Optional".into(),
            description: "".into(),
            category: "Security".into(),
            mandatory: false,
            verification_method: VerificationMethodV7::Manual,
        });
        assert_eq!(fw.mandatory_requirements().len(), 1);
        assert_eq!(fw.mandatory_requirements()[0].id, "R1");
    }

    #[test]
    fn test_requirements_by_category() {
        let mut fw = ComplianceFrameworkV9::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV7 {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Security".into(),
            mandatory: true,
            verification_method: VerificationMethodV7::Automated,
        });
        fw.add_requirement(ComplianceRequirementV7 {
            id: "R2".into(),
            name: "B".into(),
            description: "".into(),
            category: "Privacy".into(),
            mandatory: true,
            verification_method: VerificationMethodV7::Automated,
        });
        assert_eq!(fw.requirements_by_category("Security").len(), 1);
        assert_eq!(fw.requirements_by_category("Privacy").len(), 1);
        assert!(fw.requirements_by_category("Nonexistent").is_empty());
    }

    #[test]
    fn test_assessment_v7_new() {
        let assessment = ComplianceAssessmentV7::new("fw-1".into());
        assert_eq!(assessment.framework_id, "fw-1");
        assert_eq!(assessment.status, AssessmentStatusV7::Pending);
        assert!(assessment.findings.is_empty());
        assert_eq!(assessment.score, 0);
        assert!(assessment.history.is_empty());
    }

    #[test]
    fn test_assessment_v7_with_repo() {
        let assessment = ComplianceAssessmentV7::new("fw-1".into())
            .with_repo("repo-1")
            .with_assessor("user-1");
        assert_eq!(assessment.repo_id.as_deref(), Some("repo-1"));
        assert_eq!(assessment.assessor_id.as_deref(), Some("user-1"));
    }

    #[test]
    fn test_assessment_v7_add_finding() {
        let mut assessment = ComplianceAssessmentV7::new("fw-1".into());
        let finding = ComplianceFindingV7 {
            id: uuid::Uuid::new_v4().to_string(),
            requirement_id: "req-1".into(),
            status: FindingStatusV7::Compliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        assessment.add_finding(finding);
        assert_eq!(assessment.findings.len(), 1);
        assert_eq!(assessment.history.len(), 1);
    }

    #[test]
    fn test_assessment_v7_complete() {
        let mut assessment = ComplianceAssessmentV7::new("fw-1".into());
        let finding = ComplianceFindingV7 {
            id: uuid::Uuid::new_v4().to_string(),
            requirement_id: "req-1".into(),
            status: FindingStatusV7::Compliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        assessment.add_finding(finding);
        assessment.complete(100);
        assert_eq!(assessment.status, AssessmentStatusV7::Passed);
        assert_eq!(assessment.score, 100);
        assert!(assessment.completed_at.is_some());
    }

    #[test]
    fn test_assessment_v7_complete_failed() {
        let mut assessment = ComplianceAssessmentV7::new("fw-1".into());
        let finding = ComplianceFindingV7 {
            id: uuid::Uuid::new_v4().to_string(),
            requirement_id: "req-1".into(),
            status: FindingStatusV7::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        assessment.add_finding(finding);
        assessment.complete(0);
        assert_eq!(assessment.status, AssessmentStatusV7::Failed);
    }

    #[test]
    fn test_finding_is_compliant() {
        let f = make_finding("R1", FindingStatusV7::Compliant);
        assert!(f.is_compliant());
        let f = make_finding("R1", FindingStatusV7::NotApplicable);
        assert!(f.is_compliant());
        let f = make_finding("R1", FindingStatusV7::NonCompliant);
        assert!(!f.is_compliant());
    }

    #[test]
    fn test_finding_tracker_v9_new() {
        let tracker = FindingTrackerV9::new();
        assert!(tracker.findings.is_empty());
        assert!(tracker.remediation_deadlines.is_empty());
        assert!(tracker.assigned_to.is_empty());
        assert!(tracker.severity_scores.is_empty());
    }

    #[test]
    fn test_finding_tracker_v9_add_finding() {
        let mut tracker = FindingTrackerV9::new();
        let finding = ComplianceFindingV7 {
            id: "f-1".into(),
            requirement_id: "req-1".into(),
            status: FindingStatusV7::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        tracker.add_finding(finding);
        assert_eq!(tracker.findings.len(), 1);
    }

    #[test]
    fn test_finding_tracker_v9_overdue() {
        let mut tracker = FindingTrackerV9::new();
        let finding = ComplianceFindingV7 {
            id: "f-1".into(),
            requirement_id: "req-1".into(),
            status: FindingStatusV7::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        tracker.add_finding(finding);
        tracker.set_deadline("f-1", Utc::now() - chrono::Duration::hours(1));
        assert_eq!(tracker.overdue_findings().len(), 1);
    }

    #[test]
    fn test_finding_tracker_v9_open_findings() {
        let mut tracker = FindingTrackerV9::new();
        let f1 = ComplianceFindingV7 {
            id: "f-1".into(),
            requirement_id: "req-1".into(),
            status: FindingStatusV7::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        let f2 = ComplianceFindingV7 {
            id: "f-2".into(),
            requirement_id: "req-2".into(),
            status: FindingStatusV7::Compliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        tracker.add_finding(f1);
        tracker.add_finding(f2);
        assert_eq!(tracker.open_findings().len(), 1);
    }

    #[test]
    fn test_finding_tracker_v9_assign() {
        let mut tracker = FindingTrackerV9::new();
        let finding = ComplianceFindingV7 {
            id: "f-1".into(),
            requirement_id: "req-1".into(),
            status: FindingStatusV7::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        tracker.add_finding(finding);
        tracker.assign("f-1", "user-1");
        assert_eq!(tracker.assigned_to.get("f-1").map(|s| s.as_str()), Some("user-1"));
    }

    #[test]
    fn test_finding_tracker_v9_compute_severity_scores() {
        let mut tracker = FindingTrackerV9::new();
        let f1 = ComplianceFindingV7 {
            id: "f-1".into(),
            requirement_id: "CC6.1".into(),
            status: FindingStatusV7::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        let f2 = ComplianceFindingV7 {
            id: "f-2".into(),
            requirement_id: "CC6.1".into(),
            status: FindingStatusV7::Partial,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        tracker.add_finding(f1);
        tracker.add_finding(f2);
        let scores = tracker.compute_severity_scores();
        assert_eq!(scores.get("CC6"), Some(&15));
    }

    #[test]
    fn test_assess_all_compliant() {
        let mut fw = ComplianceFrameworkV9::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV7 {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV7::Automated,
        });
        fw.add_requirement(ComplianceRequirementV7 {
            id: "R2".into(),
            name: "B".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV7::Automated,
        });
        let findings = vec![
            make_finding("R1", FindingStatusV7::Compliant),
            make_finding("R2", FindingStatusV7::Compliant),
        ];
        let result = ComplianceAssessorV4::assess(&fw, &findings);
        assert_eq!(result.score, 100);
        assert_eq!(result.status, AssessmentStatusV7::Passed);
        assert_eq!(result.compliant_count, 2);
    }

    #[test]
    fn test_assess_some_non_compliant() {
        let mut fw = ComplianceFrameworkV9::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV7 {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV7::Automated,
        });
        fw.add_requirement(ComplianceRequirementV7 {
            id: "R2".into(),
            name: "B".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV7::Automated,
        });
        let findings = vec![
            make_finding("R1", FindingStatusV7::Compliant),
            make_finding("R2", FindingStatusV7::NonCompliant),
        ];
        let result = ComplianceAssessorV4::assess(&fw, &findings);
        assert_eq!(result.score, 50);
        assert_eq!(result.status, AssessmentStatusV7::Failed);
        assert_eq!(result.non_compliant_count, 1);
    }

    #[test]
    fn test_assess_with_not_applicable() {
        let mut fw = ComplianceFrameworkV9::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV7 {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV7::Automated,
        });
        fw.add_requirement(ComplianceRequirementV7 {
            id: "R2".into(),
            name: "B".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV7::Automated,
        });
        let findings = vec![
            make_finding("R1", FindingStatusV7::Compliant),
            make_finding("R2", FindingStatusV7::NotApplicable),
        ];
        let result = ComplianceAssessorV4::assess(&fw, &findings);
        assert_eq!(result.score, 100);
        assert_eq!(result.status, AssessmentStatusV7::Passed);
        assert_eq!(result.not_applicable_count, 1);
    }

    #[test]
    fn test_assess_empty_requirements() {
        let fw = ComplianceFrameworkV9::new("Test".into(), "Desc".into());
        let result = ComplianceAssessorV4::assess(&fw, &[]);
        assert_eq!(result.score, 100);
        assert_eq!(result.status, AssessmentStatusV7::Passed);
    }

    #[test]
    fn test_assess_partial() {
        let mut fw = ComplianceFrameworkV9::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV7 {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV7::Automated,
        });
        let findings = vec![make_finding("R1", FindingStatusV7::Partial)];
        let result = ComplianceAssessorV4::assess(&fw, &findings);
        assert_eq!(result.status, AssessmentStatusV7::Partial);
    }

    #[test]
    fn test_soc2_framework_v9() {
        let fw = create_soc2_framework_v9();
        assert_eq!(fw.name, "SOC 2");
        assert!(!fw.requirements.is_empty());
        assert!(fw.mandatory_requirements().len() > 0);
    }

    #[test]
    fn test_iso27001_framework_v9() {
        let fw = create_iso27001_framework_v9();
        assert_eq!(fw.name, "ISO 27001");
        assert!(!fw.requirements.is_empty());
    }

    #[test]
    fn test_compliance_scoring_engine_v4() {
        let requirements = vec![
            ComplianceRequirementRecordV7::new("fw".into(), "R1".into(), "".into()),
            ComplianceRequirementRecordV7::new("fw".into(), "R2".into(), "".into()),
        ];
        let check_results = vec![
            ComplianceCheckResultV7::new("R1".into(), "a".into(), CheckStatusV7::Passed),
            ComplianceCheckResultV7::new("R2".into(), "a".into(), CheckStatusV7::Failed),
        ];
        let score = ComplianceScoringEngineV4::calculate_framework_score(&requirements, &check_results);
        assert_eq!(score.total_requirements, 2);
        assert_eq!(score.passed, 1);
        assert_eq!(score.failed, 1);
        assert_eq!(score.overall, 50);
    }

    #[test]
    fn test_compliance_scoring_engine_v4_all_passed() {
        let requirements = vec![
            ComplianceRequirementRecordV7::new("fw".into(), "R1".into(), "".into()),
        ];
        let check_results = vec![
            ComplianceCheckResultV7::new("R1".into(), "a".into(), CheckStatusV7::Passed),
        ];
        let score = ComplianceScoringEngineV4::calculate_framework_score(&requirements, &check_results);
        assert_eq!(score.overall, 100);
    }

    #[test]
    fn test_compliance_scoring_engine_v4_weighted() {
        let requirements = vec![
            ComplianceRequirementRecordV7::new("fw".into(), "R1".into(), "".into())
                .with_severity(RequirementSeverityV7::Critical),
            ComplianceRequirementRecordV7::new("fw".into(), "R2".into(), "".into())
                .with_severity(RequirementSeverityV7::Low),
        ];
        let check_results = vec![
            ComplianceCheckResultV7::new("R1".into(), "a".into(), CheckStatusV7::Passed),
            ComplianceCheckResultV7::new("R2".into(), "a".into(), CheckStatusV7::Failed),
        ];
        let score = ComplianceScoringEngineV4::calculate_weighted_score(&requirements, &check_results);
        assert!((score - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_verification_method_v7_serialization() {
        assert_eq!(
            serde_json::to_string(&VerificationMethodV7::Automated).unwrap(),
            "\"automated\""
        );
        assert_eq!(
            serde_json::to_string(&VerificationMethodV7::Manual).unwrap(),
            "\"manual\""
        );
    }

    #[test]
    fn test_assessment_status_v7_serialization() {
        assert_eq!(
            serde_json::to_string(&AssessmentStatusV7::Passed).unwrap(),
            "\"passed\""
        );
        assert_eq!(
            serde_json::to_string(&AssessmentStatusV7::Failed).unwrap(),
            "\"failed\""
        );
    }

    #[test]
    fn test_finding_status_v7_serialization() {
        assert_eq!(
            serde_json::to_string(&FindingStatusV7::Compliant).unwrap(),
            "\"compliant\""
        );
        assert_eq!(
            serde_json::to_string(&FindingStatusV7::NonCompliant).unwrap(),
            "\"non_compliant\""
        );
    }
}
