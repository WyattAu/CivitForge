#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirementRecordV20 {
    pub id: String,
    pub framework_id: String,
    pub requirement_id: String,
    pub description: String,
    pub severity: RequirementSeverityV20,
    pub automated_check: bool,
    pub check_config: HashMap<String, serde_json::Value>,
    pub evidence_config: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ComplianceRequirementRecordV20 {
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
            severity: RequirementSeverityV20::Medium,
            automated_check: false,
            check_config: HashMap::new(),
            evidence_config: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_severity(mut self, severity: RequirementSeverityV20) -> Self {
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

    pub fn risk_weight(&self) -> f64 {
        match self.severity {
            RequirementSeverityV20::Critical => 4.0,
            RequirementSeverityV20::High => 3.0,
            RequirementSeverityV20::Medium => 2.0,
            RequirementSeverityV20::Low => 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementSeverityV20 {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceEvidenceV20 {
    pub id: String,
    pub requirement_id: String,
    pub assessment_id: String,
    pub evidence_type: EvidenceTypeV20,
    pub content: HashMap<String, serde_json::Value>,
    pub collected_by: Option<String>,
    pub collected_at: DateTime<Utc>,
}

impl ComplianceEvidenceV20 {
    pub fn new(
        requirement_id: String,
        assessment_id: String,
        evidence_type: EvidenceTypeV20,
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
pub enum EvidenceTypeV20 {
    Manual,
    Automated,
    SystemGenerated,
    External,
    Inherited,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheckResultV20 {
    pub id: String,
    pub requirement_id: String,
    pub assessment_id: String,
    pub status: CheckStatusV20,
    pub result_details: HashMap<String, serde_json::Value>,
    pub score: u32,
    pub executed_at: DateTime<Utc>,
}

impl ComplianceCheckResultV20 {
    pub fn new(
        requirement_id: String,
        assessment_id: String,
        status: CheckStatusV20,
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
pub enum CheckStatusV20 {
    Pending,
    Running,
    Passed,
    Failed,
    Skipped,
    Inherited,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkV20 {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub requirements: Vec<ComplianceRequirementV20>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ComplianceFrameworkV20 {
    pub fn new(name: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            version: "18.0".into(),
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

    pub fn add_requirement(&mut self, requirement: ComplianceRequirementV20) {
        self.requirements.push(requirement);
    }

    pub fn mandatory_requirements(&self) -> Vec<&ComplianceRequirementV20> {
        self.requirements.iter().filter(|r| r.mandatory).collect()
    }

    pub fn requirements_by_category(&self, category: &str) -> Vec<&ComplianceRequirementV20> {
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

    pub fn requirement_count(&self) -> usize {
        self.requirements.len()
    }

    pub fn mandatory_count(&self) -> usize {
        self.mandatory_requirements().len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirementV20 {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub mandatory: bool,
    pub verification_method: VerificationMethodV20,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationMethodV20 {
    Automated,
    Manual,
    Hybrid,
    Inherited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentStatusV20 {
    Pending,
    Running,
    Passed,
    Failed,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAssessmentV20 {
    pub id: String,
    pub framework_id: String,
    pub repo_id: Option<String>,
    pub status: AssessmentStatusV20,
    pub findings: Vec<ComplianceFindingV20>,
    pub score: u32,
    pub assessor_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub history: Vec<AssessmentSnapshotV20>,
}

impl ComplianceAssessmentV20 {
    pub fn new(framework_id: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            framework_id,
            repo_id: None,
            status: AssessmentStatusV20::Pending,
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

    pub fn add_finding(&mut self, finding: ComplianceFindingV20) {
        self.findings.push(finding);
        self.snapshot_history();
    }

    pub fn complete(&mut self, score: u32) {
        self.status = if self.findings.iter().any(|f| f.status == FindingStatusV20::NonCompliant) {
            AssessmentStatusV20::Failed
        } else if self.findings.iter().any(|f| f.status == FindingStatusV20::Partial) {
            AssessmentStatusV20::Partial
        } else {
            AssessmentStatusV20::Passed
        };
        self.score = score;
        self.completed_at = Some(Utc::now());
        self.snapshot_history();
    }

    pub fn compliance_rate(&self) -> f64 {
        if self.findings.is_empty() {
            return 100.0;
        }
        let compliant = self
            .findings
            .iter()
            .filter(|f| f.status == FindingStatusV20::Compliant || f.status == FindingStatusV20::NotApplicable)
            .count();
        (compliant as f64 / self.findings.len() as f64) * 100.0
    }

    pub fn non_compliant_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.status == FindingStatusV20::NonCompliant)
            .count()
    }

    fn snapshot_history(&mut self) {
        self.history.push(AssessmentSnapshotV20 {
            status: self.status,
            score: self.score,
            findings_count: self.findings.len() as u32,
            timestamp: Utc::now(),
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentSnapshotV20 {
    pub status: AssessmentStatusV20,
    pub score: u32,
    pub findings_count: u32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFindingV20 {
    pub id: String,
    pub requirement_id: String,
    pub status: FindingStatusV20,
    pub details: String,
    pub evidence: Option<String>,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatusV20 {
    Compliant,
    NonCompliant,
    Partial,
    NotApplicable,
}

impl ComplianceFindingV20 {
    pub fn is_compliant(&self) -> bool {
        self.status == FindingStatusV20::Compliant || self.status == FindingStatusV20::NotApplicable
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingTrackerV20 {
    pub findings: Vec<ComplianceFindingV20>,
    pub remediation_deadlines: HashMap<String, DateTime<Utc>>,
    pub assigned_to: HashMap<String, String>,
    pub severity_scores: HashMap<String, u32>,
    pub history: Vec<FindingHistoryEntryV20>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingHistoryEntryV20 {
    pub finding_id: String,
    pub old_status: FindingStatusV20,
    pub new_status: FindingStatusV20,
    pub changed_by: Option<String>,
    pub changed_at: DateTime<Utc>,
}

impl FindingTrackerV20 {
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
            remediation_deadlines: HashMap::new(),
            assigned_to: HashMap::new(),
            severity_scores: HashMap::new(),
            history: Vec::new(),
        }
    }

    pub fn add_finding(&mut self, finding: ComplianceFindingV20) {
        self.findings.push(finding);
    }

    pub fn set_deadline(&mut self, finding_id: &str, deadline: DateTime<Utc>) {
        self.remediation_deadlines.insert(finding_id.into(), deadline);
    }

    pub fn assign(&mut self, finding_id: &str, user_id: &str) {
        self.assigned_to.insert(finding_id.into(), user_id.into());
    }

    pub fn update_finding_status(
        &mut self,
        finding_id: &str,
        new_status: FindingStatusV20,
        changed_by: Option<String>,
    ) -> Result<(), String> {
        let finding = self
            .findings
            .iter_mut()
            .find(|f| f.id == finding_id)
            .ok_or("Finding not found")?;
        let old_status = finding.status;
        finding.status = new_status;
        self.history.push(FindingHistoryEntryV20 {
            finding_id: finding_id.into(),
            old_status,
            new_status,
            changed_by,
            changed_at: Utc::now(),
        });
        Ok(())
    }

    pub fn overdue_findings(&self) -> Vec<&ComplianceFindingV20> {
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

    pub fn open_findings(&self) -> Vec<&ComplianceFindingV20> {
        self.findings
            .iter()
            .filter(|f| f.status == FindingStatusV20::NonCompliant || f.status == FindingStatusV20::Partial)
            .collect()
    }

    pub fn findings_by_severity(&self, status: FindingStatusV20) -> Vec<&ComplianceFindingV20> {
        self.findings.iter().filter(|f| f.status == status).collect()
    }

    pub fn compute_severity_scores(&mut self) -> HashMap<String, u32> {
        let mut scores: HashMap<String, u32> = HashMap::new();
        for finding in &self.findings {
            let category = finding.requirement_id.split('.').next().unwrap_or("unknown").to_string();
            *scores.entry(category).or_insert(0) += match finding.status {
                FindingStatusV20::NonCompliant => 10,
                FindingStatusV20::Partial => 5,
                FindingStatusV20::Compliant => 0,
                FindingStatusV20::NotApplicable => 0,
            };
        }
        self.severity_scores = scores.clone();
        scores
    }

    pub fn total_findings(&self) -> usize {
        self.findings.len()
    }

    pub fn finding_history(&self, finding_id: &str) -> Vec<&FindingHistoryEntryV20> {
        self.history
            .iter()
            .filter(|h| h.finding_id == finding_id)
            .collect()
    }
}

impl Default for FindingTrackerV20 {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceScoreV20 {
    pub overall: u32,
    pub by_severity: HashMap<String, (u32, u32)>,
    pub total_requirements: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub inherited: u32,
}

pub struct ComplianceScoringEngineV20;

impl ComplianceScoringEngineV20 {
    pub fn calculate_framework_score(
        requirements: &[ComplianceRequirementRecordV20],
        check_results: &[ComplianceCheckResultV20],
    ) -> ComplianceScoreV20 {
        let total = requirements.len() as u32;
        if total == 0 {
            return ComplianceScoreV20 {
                overall: 100,
                by_severity: HashMap::new(),
                total_requirements: 0,
                passed: 0,
                failed: 0,
                skipped: 0,
                inherited: 0,
            };
        }

        let mut passed = 0u32;
        let mut failed = 0u32;
        let mut skipped = 0u32;
        let mut inherited = 0u32;

        for result in check_results {
            match result.status {
                CheckStatusV20::Passed => passed += 1,
                CheckStatusV20::Failed => failed += 1,
                CheckStatusV20::Skipped => skipped += 1,
                CheckStatusV20::Inherited => inherited += 1,
                _ => {}
            }
        }

        let applicable = total - skipped - inherited;
        let overall = if applicable > 0 {
            ((passed as f64 / applicable as f64) * 100.0) as u32
        } else {
            100
        };

        ComplianceScoreV20 {
            overall,
            by_severity: HashMap::new(),
            total_requirements: total,
            passed,
            failed,
            skipped,
            inherited,
        }
    }

    pub fn calculate_weighted_score(
        requirements: &[ComplianceRequirementRecordV20],
        check_results: &[ComplianceCheckResultV20],
    ) -> f64 {
        let total_weight: f64 = requirements.iter().map(|r| r.risk_weight()).sum();

        if total_weight == 0.0 {
            return 100.0;
        }

        let earned_weight: f64 = requirements.iter().zip(check_results.iter()).map(|(req, res)| {
            let weight = req.risk_weight();
            if res.status == CheckStatusV20::Passed || res.status == CheckStatusV20::Inherited {
                weight
            } else {
                0.0
            }
        }).sum();

        (earned_weight / total_weight) * 100.0
    }
}

pub struct ComplianceAssessorV20;

impl ComplianceAssessorV20 {
    pub fn assess(
        framework: &ComplianceFrameworkV20,
        findings: &[ComplianceFindingV20],
    ) -> AssessmentResultV20 {
        let total_requirements = framework.requirements.len() as u32;
        if total_requirements == 0 {
            return AssessmentResultV20 {
                score: 100,
                status: AssessmentStatusV20::Passed,
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
                FindingStatusV20::Compliant => compliant += 1,
                FindingStatusV20::NonCompliant => non_compliant += 1,
                FindingStatusV20::Partial => partial += 1,
                FindingStatusV20::NotApplicable => not_applicable += 1,
            }
        }

        let applicable = total_requirements - not_applicable;
        let score = if applicable > 0 {
            ((compliant as f64 / applicable as f64) * 100.0) as u32
        } else {
            100
        };

        let status = if non_compliant > 0 {
            AssessmentStatusV20::Failed
        } else if partial > 0 {
            AssessmentStatusV20::Partial
        } else {
            AssessmentStatusV20::Passed
        };

        AssessmentResultV20 {
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
pub struct AssessmentResultV20 {
    pub score: u32,
    pub status: AssessmentStatusV20,
    pub compliant_count: u32,
    pub non_compliant_count: u32,
    pub partial_count: u32,
    pub not_applicable_count: u32,
}

pub fn create_soc2_framework_v19() -> ComplianceFrameworkV20 {
    let mut framework = ComplianceFrameworkV20::new(
        "SOC 2".into(),
        "Service Organization Control 2".into(),
    );
    framework.add_requirement(ComplianceRequirementV20 {
        id: "CC6.1".into(),
        name: "Logical Access Controls".into(),
        description: "Implement logical access security controls".into(),
        category: "Security".into(),
        mandatory: true,
        verification_method: VerificationMethodV20::Automated,
    });
    framework.add_requirement(ComplianceRequirementV20 {
        id: "CC6.6".into(),
        name: "System Boundaries".into(),
        description: "Restrict logical access to system boundaries".into(),
        category: "Security".into(),
        mandatory: true,
        verification_method: VerificationMethodV20::Hybrid,
    });
    framework.add_requirement(ComplianceRequirementV20 {
        id: "CC7.1".into(),
        name: "Vulnerability Management".into(),
        description: "Detect and monitor for vulnerabilities".into(),
        category: "Security".into(),
        mandatory: true,
        verification_method: VerificationMethodV20::Automated,
    });
    framework.add_requirement(ComplianceRequirementV20 {
        id: "CC8.1".into(),
        name: "Change Management".into(),
        description: "Authorize and manage changes to systems".into(),
        category: "Availability".into(),
        mandatory: true,
        verification_method: VerificationMethodV20::Manual,
    });
    framework
}

pub fn create_iso27001_framework_v19() -> ComplianceFrameworkV20 {
    let mut framework = ComplianceFrameworkV20::new(
        "ISO 27001".into(),
        "Information Security Management System".into(),
    );
    framework.add_requirement(ComplianceRequirementV20 {
        id: "A.12.6.1".into(),
        name: "Technical Vulnerability Management".into(),
        description: "Obtain information about technical vulnerabilities".into(),
        category: "Operations Security".into(),
        mandatory: true,
        verification_method: VerificationMethodV20::Automated,
    });
    framework.add_requirement(ComplianceRequirementV20 {
        id: "A.14.2.1".into(),
        name: "Secure Development Policy".into(),
        description: "Establish secure development lifecycle".into(),
        category: "Acquisition Development".into(),
        mandatory: true,
        verification_method: VerificationMethodV20::Hybrid,
    });
    framework.add_requirement(ComplianceRequirementV20 {
        id: "A.18.2.1".into(),
        name: "Independent Review".into(),
        description: "Independent review of organization's ISMS".into(),
        category: "Compliance".into(),
        mandatory: true,
        verification_method: VerificationMethodV20::Manual,
    });
    framework
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_finding(requirement_id: &str, status: FindingStatusV20) -> ComplianceFindingV20 {
        ComplianceFindingV20 {
            id: uuid::Uuid::new_v4().to_string(),
            requirement_id: requirement_id.into(),
            status,
            details: "Test finding".into(),
            evidence: None,
            remediation: None,
        }
    }

    #[test]
    fn test_compliance_framework_v19_new() {
        let fw = ComplianceFrameworkV20::new("Test".into(), "Description".into());
        assert_eq!(fw.name, "Test");
        assert!(fw.enabled);
        assert!(fw.requirements.is_empty());
        assert_eq!(fw.version, "18.0");
    }

    #[test]
    fn test_compliance_framework_v19_with_version() {
        let fw = ComplianceFrameworkV20::new("Test".into(), "Desc".into())
            .with_version("19.0");
        assert_eq!(fw.version, "19.0");
    }

    #[test]
    fn test_compliance_framework_v19_increment_version() {
        let mut fw = ComplianceFrameworkV20::new("Test".into(), "Desc".into());
        fw.increment_version();
        assert_eq!(fw.version, "18.1");
        fw.increment_version();
        assert_eq!(fw.version, "18.2");
    }

    #[test]
    fn test_add_requirement() {
        let mut fw = ComplianceFrameworkV20::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV20 {
            id: "R1".into(),
            name: "Req 1".into(),
            description: "Desc".into(),
            category: "Security".into(),
            mandatory: true,
            verification_method: VerificationMethodV20::Automated,
        });
        assert_eq!(fw.requirements.len(), 1);
    }

    #[test]
    fn test_mandatory_requirements() {
        let mut fw = ComplianceFrameworkV20::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV20 {
            id: "R1".into(),
            name: "Mandatory".into(),
            description: "".into(),
            category: "Security".into(),
            mandatory: true,
            verification_method: VerificationMethodV20::Automated,
        });
        fw.add_requirement(ComplianceRequirementV20 {
            id: "R2".into(),
            name: "Optional".into(),
            description: "".into(),
            category: "Security".into(),
            mandatory: false,
            verification_method: VerificationMethodV20::Manual,
        });
        assert_eq!(fw.mandatory_requirements().len(), 1);
        assert_eq!(fw.mandatory_requirements()[0].id, "R1");
    }

    #[test]
    fn test_requirements_by_category() {
        let mut fw = ComplianceFrameworkV20::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV20 {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Security".into(),
            mandatory: true,
            verification_method: VerificationMethodV20::Automated,
        });
        fw.add_requirement(ComplianceRequirementV20 {
            id: "R2".into(),
            name: "B".into(),
            description: "".into(),
            category: "Privacy".into(),
            mandatory: true,
            verification_method: VerificationMethodV20::Automated,
        });
        assert_eq!(fw.requirements_by_category("Security").len(), 1);
        assert_eq!(fw.requirements_by_category("Privacy").len(), 1);
        assert!(fw.requirements_by_category("Nonexistent").is_empty());
    }

    #[test]
    fn test_framework_counts() {
        let mut fw = ComplianceFrameworkV20::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV20 {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV20::Automated,
        });
        fw.add_requirement(ComplianceRequirementV20 {
            id: "R2".into(),
            name: "B".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: false,
            verification_method: VerificationMethodV20::Automated,
        });
        assert_eq!(fw.requirement_count(), 2);
        assert_eq!(fw.mandatory_count(), 1);
    }

    #[test]
    fn test_assessment_v19_new() {
        let assessment = ComplianceAssessmentV20::new("fw-1".into());
        assert_eq!(assessment.framework_id, "fw-1");
        assert_eq!(assessment.status, AssessmentStatusV20::Pending);
        assert!(assessment.findings.is_empty());
        assert_eq!(assessment.score, 0);
        assert!(assessment.history.is_empty());
    }

    #[test]
    fn test_assessment_v19_with_repo() {
        let assessment = ComplianceAssessmentV20::new("fw-1".into())
            .with_repo("repo-1")
            .with_assessor("user-1");
        assert_eq!(assessment.repo_id.as_deref(), Some("repo-1"));
        assert_eq!(assessment.assessor_id.as_deref(), Some("user-1"));
    }

    #[test]
    fn test_assessment_v19_add_finding() {
        let mut assessment = ComplianceAssessmentV20::new("fw-1".into());
        let finding = ComplianceFindingV20 {
            id: uuid::Uuid::new_v4().to_string(),
            requirement_id: "req-1".into(),
            status: FindingStatusV20::Compliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        assessment.add_finding(finding);
        assert_eq!(assessment.findings.len(), 1);
        assert_eq!(assessment.history.len(), 1);
    }

    #[test]
    fn test_assessment_v19_complete() {
        let mut assessment = ComplianceAssessmentV20::new("fw-1".into());
        let finding = ComplianceFindingV20 {
            id: uuid::Uuid::new_v4().to_string(),
            requirement_id: "req-1".into(),
            status: FindingStatusV20::Compliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        assessment.add_finding(finding);
        assessment.complete(100);
        assert_eq!(assessment.status, AssessmentStatusV20::Passed);
        assert_eq!(assessment.score, 100);
        assert!(assessment.completed_at.is_some());
    }

    #[test]
    fn test_assessment_v19_complete_failed() {
        let mut assessment = ComplianceAssessmentV20::new("fw-1".into());
        let finding = ComplianceFindingV20 {
            id: uuid::Uuid::new_v4().to_string(),
            requirement_id: "req-1".into(),
            status: FindingStatusV20::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        assessment.add_finding(finding);
        assessment.complete(0);
        assert_eq!(assessment.status, AssessmentStatusV20::Failed);
    }

    #[test]
    fn test_assessment_compliance_rate() {
        let mut assessment = ComplianceAssessmentV20::new("fw-1".into());
        assessment.add_finding(make_finding("R1", FindingStatusV20::Compliant));
        assessment.add_finding(make_finding("R2", FindingStatusV20::NonCompliant));
        assert_eq!(assessment.compliance_rate(), 50.0);
    }

    #[test]
    fn test_assessment_non_compliant_count() {
        let mut assessment = ComplianceAssessmentV20::new("fw-1".into());
        assessment.add_finding(make_finding("R1", FindingStatusV20::Compliant));
        assessment.add_finding(make_finding("R2", FindingStatusV20::NonCompliant));
        assessment.add_finding(make_finding("R3", FindingStatusV20::NonCompliant));
        assert_eq!(assessment.non_compliant_count(), 2);
    }

    #[test]
    fn test_finding_is_compliant() {
        let f = make_finding("R1", FindingStatusV20::Compliant);
        assert!(f.is_compliant());
        let f = make_finding("R1", FindingStatusV20::NotApplicable);
        assert!(f.is_compliant());
        let f = make_finding("R1", FindingStatusV20::NonCompliant);
        assert!(!f.is_compliant());
    }

    #[test]
    fn test_finding_tracker_v19_new() {
        let tracker = FindingTrackerV20::new();
        assert!(tracker.findings.is_empty());
        assert!(tracker.remediation_deadlines.is_empty());
        assert!(tracker.assigned_to.is_empty());
        assert!(tracker.severity_scores.is_empty());
        assert!(tracker.history.is_empty());
    }

    #[test]
    fn test_finding_tracker_v19_add_finding() {
        let mut tracker = FindingTrackerV20::new();
        let finding = ComplianceFindingV20 {
            id: "f-1".into(),
            requirement_id: "req-1".into(),
            status: FindingStatusV20::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        tracker.add_finding(finding);
        assert_eq!(tracker.findings.len(), 1);
    }

    #[test]
    fn test_finding_tracker_v19_overdue() {
        let mut tracker = FindingTrackerV20::new();
        let finding = ComplianceFindingV20 {
            id: "f-1".into(),
            requirement_id: "req-1".into(),
            status: FindingStatusV20::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        tracker.add_finding(finding);
        tracker.set_deadline("f-1", Utc::now() - chrono::Duration::hours(1));
        assert_eq!(tracker.overdue_findings().len(), 1);
    }

    #[test]
    fn test_finding_tracker_v19_open_findings() {
        let mut tracker = FindingTrackerV20::new();
        let f1 = ComplianceFindingV20 {
            id: "f-1".into(),
            requirement_id: "req-1".into(),
            status: FindingStatusV20::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        let f2 = ComplianceFindingV20 {
            id: "f-2".into(),
            requirement_id: "req-2".into(),
            status: FindingStatusV20::Compliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        tracker.add_finding(f1);
        tracker.add_finding(f2);
        assert_eq!(tracker.open_findings().len(), 1);
    }

    #[test]
    fn test_finding_tracker_v19_assign() {
        let mut tracker = FindingTrackerV20::new();
        let finding = ComplianceFindingV20 {
            id: "f-1".into(),
            requirement_id: "req-1".into(),
            status: FindingStatusV20::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        tracker.add_finding(finding);
        tracker.assign("f-1", "user-1");
        assert_eq!(tracker.assigned_to.get("f-1").map(|s| s.as_str()), Some("user-1"));
    }

    #[test]
    fn test_finding_tracker_v19_update_status() {
        let mut tracker = FindingTrackerV20::new();
        let finding = ComplianceFindingV20 {
            id: "f-1".into(),
            requirement_id: "req-1".into(),
            status: FindingStatusV20::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        tracker.add_finding(finding);
        tracker
            .update_finding_status("f-1", FindingStatusV20::Compliant, Some("user-1".into()))
            .unwrap();
        assert_eq!(tracker.findings[0].status, FindingStatusV20::Compliant);
        assert_eq!(tracker.history.len(), 1);
        assert_eq!(tracker.history[0].old_status, FindingStatusV20::NonCompliant);
        assert_eq!(tracker.history[0].new_status, FindingStatusV20::Compliant);
    }

    #[test]
    fn test_finding_tracker_v19_finding_history() {
        let mut tracker = FindingTrackerV20::new();
        let finding = ComplianceFindingV20 {
            id: "f-1".into(),
            requirement_id: "req-1".into(),
            status: FindingStatusV20::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        tracker.add_finding(finding);
        tracker
            .update_finding_status("f-1", FindingStatusV20::Partial, None)
            .unwrap();
        tracker
            .update_finding_status("f-1", FindingStatusV20::Compliant, None)
            .unwrap();
        let history = tracker.finding_history("f-1");
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_finding_tracker_v19_findings_by_status() {
        let mut tracker = FindingTrackerV20::new();
        tracker.add_finding(make_finding("R1", FindingStatusV20::NonCompliant));
        tracker.add_finding(make_finding("R2", FindingStatusV20::Compliant));
        tracker.add_finding(make_finding("R3", FindingStatusV20::NonCompliant));
        assert_eq!(tracker.findings_by_severity(FindingStatusV20::NonCompliant).len(), 2);
        assert_eq!(tracker.findings_by_severity(FindingStatusV20::Compliant).len(), 1);
    }

    #[test]
    fn test_finding_tracker_v19_compute_severity_scores() {
        let mut tracker = FindingTrackerV20::new();
        let f1 = ComplianceFindingV20 {
            id: "f-1".into(),
            requirement_id: "CC6.1".into(),
            status: FindingStatusV20::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        let f2 = ComplianceFindingV20 {
            id: "f-2".into(),
            requirement_id: "CC6.1".into(),
            status: FindingStatusV20::Partial,
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
        let mut fw = ComplianceFrameworkV20::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV20 {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV20::Automated,
        });
        fw.add_requirement(ComplianceRequirementV20 {
            id: "R2".into(),
            name: "B".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV20::Automated,
        });
        let findings = vec![
            make_finding("R1", FindingStatusV20::Compliant),
            make_finding("R2", FindingStatusV20::Compliant),
        ];
        let result = ComplianceAssessorV20::assess(&fw, &findings);
        assert_eq!(result.score, 100);
        assert_eq!(result.status, AssessmentStatusV20::Passed);
        assert_eq!(result.compliant_count, 2);
    }

    #[test]
    fn test_assess_some_non_compliant() {
        let mut fw = ComplianceFrameworkV20::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV20 {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV20::Automated,
        });
        fw.add_requirement(ComplianceRequirementV20 {
            id: "R2".into(),
            name: "B".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV20::Automated,
        });
        let findings = vec![
            make_finding("R1", FindingStatusV20::Compliant),
            make_finding("R2", FindingStatusV20::NonCompliant),
        ];
        let result = ComplianceAssessorV20::assess(&fw, &findings);
        assert_eq!(result.score, 50);
        assert_eq!(result.status, AssessmentStatusV20::Failed);
        assert_eq!(result.non_compliant_count, 1);
    }

    #[test]
    fn test_assess_with_not_applicable() {
        let mut fw = ComplianceFrameworkV20::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV20 {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV20::Automated,
        });
        fw.add_requirement(ComplianceRequirementV20 {
            id: "R2".into(),
            name: "B".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV20::Automated,
        });
        let findings = vec![
            make_finding("R1", FindingStatusV20::Compliant),
            make_finding("R2", FindingStatusV20::NotApplicable),
        ];
        let result = ComplianceAssessorV20::assess(&fw, &findings);
        assert_eq!(result.score, 100);
        assert_eq!(result.status, AssessmentStatusV20::Passed);
        assert_eq!(result.not_applicable_count, 1);
    }

    #[test]
    fn test_assess_empty_requirements() {
        let fw = ComplianceFrameworkV20::new("Test".into(), "Desc".into());
        let result = ComplianceAssessorV20::assess(&fw, &[]);
        assert_eq!(result.score, 100);
        assert_eq!(result.status, AssessmentStatusV20::Passed);
    }

    #[test]
    fn test_assess_partial() {
        let mut fw = ComplianceFrameworkV20::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV20 {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV20::Automated,
        });
        let findings = vec![make_finding("R1", FindingStatusV20::Partial)];
        let result = ComplianceAssessorV20::assess(&fw, &findings);
        assert_eq!(result.status, AssessmentStatusV20::Partial);
    }

    #[test]
    fn test_soc2_framework_v19() {
        let fw = create_soc2_framework_v19();
        assert_eq!(fw.name, "SOC 2");
        assert!(!fw.requirements.is_empty());
        assert!(fw.mandatory_requirements().len() > 0);
    }

    #[test]
    fn test_iso27001_framework_v19() {
        let fw = create_iso27001_framework_v19();
        assert_eq!(fw.name, "ISO 27001");
        assert!(!fw.requirements.is_empty());
    }

    #[test]
    fn test_compliance_scoring_engine_v19() {
        let requirements = vec![
            ComplianceRequirementRecordV20::new("fw".into(), "R1".into(), "".into()),
            ComplianceRequirementRecordV20::new("fw".into(), "R2".into(), "".into()),
        ];
        let check_results = vec![
            ComplianceCheckResultV20::new("R1".into(), "a".into(), CheckStatusV20::Passed),
            ComplianceCheckResultV20::new("R2".into(), "a".into(), CheckStatusV20::Failed),
        ];
        let score = ComplianceScoringEngineV20::calculate_framework_score(&requirements, &check_results);
        assert_eq!(score.total_requirements, 2);
        assert_eq!(score.passed, 1);
        assert_eq!(score.failed, 1);
        assert_eq!(score.overall, 50);
    }

    #[test]
    fn test_compliance_scoring_engine_v19_all_passed() {
        let requirements = vec![
            ComplianceRequirementRecordV20::new("fw".into(), "R1".into(), "".into()),
        ];
        let check_results = vec![
            ComplianceCheckResultV20::new("R1".into(), "a".into(), CheckStatusV20::Passed),
        ];
        let score = ComplianceScoringEngineV20::calculate_framework_score(&requirements, &check_results);
        assert_eq!(score.overall, 100);
    }

    #[test]
    fn test_compliance_scoring_engine_v19_weighted() {
        let requirements = vec![
            ComplianceRequirementRecordV20::new("fw".into(), "R1".into(), "".into())
                .with_severity(RequirementSeverityV20::Critical),
            ComplianceRequirementRecordV20::new("fw".into(), "R2".into(), "".into())
                .with_severity(RequirementSeverityV20::Low),
        ];
        let check_results = vec![
            ComplianceCheckResultV20::new("R1".into(), "a".into(), CheckStatusV20::Passed),
            ComplianceCheckResultV20::new("R2".into(), "a".into(), CheckStatusV20::Failed),
        ];
        let score = ComplianceScoringEngineV20::calculate_weighted_score(&requirements, &check_results);
        assert!((score - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_compliance_scoring_engine_v19_inherited() {
        let requirements = vec![
            ComplianceRequirementRecordV20::new("fw".into(), "R1".into(), "".into()),
            ComplianceRequirementRecordV20::new("fw".into(), "R2".into(), "".into()),
        ];
        let check_results = vec![
            ComplianceCheckResultV20::new("R1".into(), "a".into(), CheckStatusV20::Passed),
            ComplianceCheckResultV20::new("R2".into(), "a".into(), CheckStatusV20::Inherited),
        ];
        let score = ComplianceScoringEngineV20::calculate_framework_score(&requirements, &check_results);
        assert_eq!(score.overall, 100);
        assert_eq!(score.inherited, 1);
    }

    #[test]
    fn test_requirement_risk_weight() {
        let req = ComplianceRequirementRecordV20::new("fw".into(), "R1".into(), "".into())
            .with_severity(RequirementSeverityV20::Critical);
        assert_eq!(req.risk_weight(), 4.0);
        let req = ComplianceRequirementRecordV20::new("fw".into(), "R1".into(), "".into())
            .with_severity(RequirementSeverityV20::Low);
        assert_eq!(req.risk_weight(), 1.0);
    }
}
