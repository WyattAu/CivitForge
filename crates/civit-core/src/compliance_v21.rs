#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirementRecordV21 {
    pub id: String,
    pub framework_id: String,
    pub requirement_id: String,
    pub description: String,
    pub severity: RequirementSeverityV21,
    pub automated_check: bool,
    pub check_config: HashMap<String, serde_json::Value>,
    pub evidence_config: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ComplianceRequirementRecordV21 {
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
            severity: RequirementSeverityV21::Medium,
            automated_check: false,
            check_config: HashMap::new(),
            evidence_config: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_severity(mut self, severity: RequirementSeverityV21) -> Self {
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
            RequirementSeverityV21::Critical => 4.0,
            RequirementSeverityV21::High => 3.0,
            RequirementSeverityV21::Medium => 2.0,
            RequirementSeverityV21::Low => 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementSeverityV21 {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceEvidenceV21 {
    pub id: String,
    pub requirement_id: String,
    pub assessment_id: String,
    pub evidence_type: EvidenceTypeV21,
    pub content: HashMap<String, serde_json::Value>,
    pub collected_by: Option<String>,
    pub collected_at: DateTime<Utc>,
}

impl ComplianceEvidenceV21 {
    pub fn new(
        requirement_id: String,
        assessment_id: String,
        evidence_type: EvidenceTypeV21,
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
pub enum EvidenceTypeV21 {
    Manual,
    Automated,
    SystemGenerated,
    External,
    Inherited,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheckResultV21 {
    pub id: String,
    pub requirement_id: String,
    pub assessment_id: String,
    pub status: CheckStatusV21,
    pub result_details: HashMap<String, serde_json::Value>,
    pub score: u32,
    pub executed_at: DateTime<Utc>,
}

impl ComplianceCheckResultV21 {
    pub fn new(
        requirement_id: String,
        assessment_id: String,
        status: CheckStatusV21,
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
pub enum CheckStatusV21 {
    Pending,
    Running,
    Passed,
    Failed,
    Skipped,
    Inherited,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkV21 {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub requirements: Vec<ComplianceRequirementV21>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ComplianceFrameworkV21 {
    pub fn new(name: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            version: "19.0".into(),
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

    pub fn add_requirement(&mut self, requirement: ComplianceRequirementV21) {
        self.requirements.push(requirement);
    }

    pub fn mandatory_requirements(&self) -> Vec<&ComplianceRequirementV21> {
        self.requirements.iter().filter(|r| r.mandatory).collect()
    }

    pub fn requirements_by_category(&self, category: &str) -> Vec<&ComplianceRequirementV21> {
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
pub struct ComplianceRequirementV21 {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub mandatory: bool,
    pub verification_method: VerificationMethodV21,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationMethodV21 {
    Automated,
    Manual,
    Hybrid,
    Inherited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentStatusV21 {
    Pending,
    Running,
    Passed,
    Failed,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAssessmentV21 {
    pub id: String,
    pub framework_id: String,
    pub repo_id: Option<String>,
    pub status: AssessmentStatusV21,
    pub findings: Vec<ComplianceFindingV21>,
    pub score: u32,
    pub assessor_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub history: Vec<AssessmentSnapshotV21>,
}

impl ComplianceAssessmentV21 {
    pub fn new(framework_id: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            framework_id,
            repo_id: None,
            status: AssessmentStatusV21::Pending,
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

    pub fn add_finding(&mut self, finding: ComplianceFindingV21) {
        self.findings.push(finding);
        self.snapshot_history();
    }

    pub fn complete(&mut self, score: u32) {
        self.status = if self.findings.iter().any(|f| f.status == FindingStatusV21::NonCompliant) {
            AssessmentStatusV21::Failed
        } else if self.findings.iter().any(|f| f.status == FindingStatusV21::Partial) {
            AssessmentStatusV21::Partial
        } else {
            AssessmentStatusV21::Passed
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
            .filter(|f| f.status == FindingStatusV21::Compliant || f.status == FindingStatusV21::NotApplicable)
            .count();
        (compliant as f64 / self.findings.len() as f64) * 100.0
    }

    pub fn non_compliant_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.status == FindingStatusV21::NonCompliant)
            .count()
    }

    fn snapshot_history(&mut self) {
        self.history.push(AssessmentSnapshotV21 {
            status: self.status,
            score: self.score,
            findings_count: self.findings.len() as u32,
            timestamp: Utc::now(),
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentSnapshotV21 {
    pub status: AssessmentStatusV21,
    pub score: u32,
    pub findings_count: u32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFindingV21 {
    pub id: String,
    pub requirement_id: String,
    pub status: FindingStatusV21,
    pub details: String,
    pub evidence: Option<String>,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatusV21 {
    Compliant,
    NonCompliant,
    Partial,
    NotApplicable,
}

impl ComplianceFindingV21 {
    pub fn is_compliant(&self) -> bool {
        self.status == FindingStatusV21::Compliant || self.status == FindingStatusV21::NotApplicable
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingTrackerV21 {
    pub findings: Vec<ComplianceFindingV21>,
    pub remediation_deadlines: HashMap<String, DateTime<Utc>>,
    pub assigned_to: HashMap<String, String>,
    pub severity_scores: HashMap<String, u32>,
    pub history: Vec<FindingHistoryEntryV21>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingHistoryEntryV21 {
    pub finding_id: String,
    pub old_status: FindingStatusV21,
    pub new_status: FindingStatusV21,
    pub changed_by: Option<String>,
    pub changed_at: DateTime<Utc>,
}

impl FindingTrackerV21 {
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
            remediation_deadlines: HashMap::new(),
            assigned_to: HashMap::new(),
            severity_scores: HashMap::new(),
            history: Vec::new(),
        }
    }

    pub fn add_finding(&mut self, finding: ComplianceFindingV21) {
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
        new_status: FindingStatusV21,
        changed_by: Option<String>,
    ) -> Result<(), String> {
        let finding = self
            .findings
            .iter_mut()
            .find(|f| f.id == finding_id)
            .ok_or("Finding not found")?;
        let old_status = finding.status;
        finding.status = new_status;
        self.history.push(FindingHistoryEntryV21 {
            finding_id: finding_id.into(),
            old_status,
            new_status,
            changed_by,
            changed_at: Utc::now(),
        });
        Ok(())
    }

    pub fn overdue_findings(&self) -> Vec<&ComplianceFindingV21> {
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

    pub fn open_findings(&self) -> Vec<&ComplianceFindingV21> {
        self.findings
            .iter()
            .filter(|f| f.status == FindingStatusV21::NonCompliant || f.status == FindingStatusV21::Partial)
            .collect()
    }

    pub fn findings_by_severity(&self, status: FindingStatusV21) -> Vec<&ComplianceFindingV21> {
        self.findings.iter().filter(|f| f.status == status).collect()
    }

    pub fn compute_severity_scores(&mut self) -> HashMap<String, u32> {
        let mut scores: HashMap<String, u32> = HashMap::new();
        for finding in &self.findings {
            let category = finding.requirement_id.split('.').next().unwrap_or("unknown").to_string();
            *scores.entry(category).or_insert(0) += match finding.status {
                FindingStatusV21::NonCompliant => 10,
                FindingStatusV21::Partial => 5,
                FindingStatusV21::Compliant => 0,
                FindingStatusV21::NotApplicable => 0,
            };
        }
        self.severity_scores = scores.clone();
        scores
    }

    pub fn total_findings(&self) -> usize {
        self.findings.len()
    }

    pub fn finding_history(&self, finding_id: &str) -> Vec<&FindingHistoryEntryV21> {
        self.history
            .iter()
            .filter(|h| h.finding_id == finding_id)
            .collect()
    }
}

impl Default for FindingTrackerV21 {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceScoreV21 {
    pub overall: u32,
    pub by_severity: HashMap<String, (u32, u32)>,
    pub total_requirements: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub inherited: u32,
}

pub struct ComplianceScoringEngineV21;

impl ComplianceScoringEngineV21 {
    pub fn calculate_framework_score(
        requirements: &[ComplianceRequirementRecordV21],
        check_results: &[ComplianceCheckResultV21],
    ) -> ComplianceScoreV21 {
        let total = requirements.len() as u32;
        if total == 0 {
            return ComplianceScoreV21 {
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
                CheckStatusV21::Passed => passed += 1,
                CheckStatusV21::Failed => failed += 1,
                CheckStatusV21::Skipped => skipped += 1,
                CheckStatusV21::Inherited => inherited += 1,
                _ => {}
            }
        }

        let applicable = total - skipped - inherited;
        let overall = if applicable > 0 {
            ((passed as f64 / applicable as f64) * 100.0) as u32
        } else {
            100
        };

        ComplianceScoreV21 {
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
        requirements: &[ComplianceRequirementRecordV21],
        check_results: &[ComplianceCheckResultV21],
    ) -> f64 {
        let total_weight: f64 = requirements.iter().map(|r| r.risk_weight()).sum();

        if total_weight == 0.0 {
            return 100.0;
        }

        let earned_weight: f64 = requirements.iter().zip(check_results.iter()).map(|(req, res)| {
            let weight = req.risk_weight();
            if res.status == CheckStatusV21::Passed || res.status == CheckStatusV21::Inherited {
                weight
            } else {
                0.0
            }
        }).sum();

        (earned_weight / total_weight) * 100.0
    }
}

pub struct ComplianceAssessorV21;

impl ComplianceAssessorV21 {
    pub fn assess(
        framework: &ComplianceFrameworkV21,
        findings: &[ComplianceFindingV21],
    ) -> AssessmentResultV21 {
        let total_requirements = framework.requirements.len() as u32;
        if total_requirements == 0 {
            return AssessmentResultV21 {
                score: 100,
                status: AssessmentStatusV21::Passed,
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
                FindingStatusV21::Compliant => compliant += 1,
                FindingStatusV21::NonCompliant => non_compliant += 1,
                FindingStatusV21::Partial => partial += 1,
                FindingStatusV21::NotApplicable => not_applicable += 1,
            }
        }

        let applicable = total_requirements - not_applicable;
        let score = if applicable > 0 {
            ((compliant as f64 / applicable as f64) * 100.0) as u32
        } else {
            100
        };

        let status = if non_compliant > 0 {
            AssessmentStatusV21::Failed
        } else if partial > 0 {
            AssessmentStatusV21::Partial
        } else {
            AssessmentStatusV21::Passed
        };

        AssessmentResultV21 {
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
pub struct AssessmentResultV21 {
    pub score: u32,
    pub status: AssessmentStatusV21,
    pub compliant_count: u32,
    pub non_compliant_count: u32,
    pub partial_count: u32,
    pub not_applicable_count: u32,
}

pub fn create_soc2_framework_v21() -> ComplianceFrameworkV21 {
    let mut framework = ComplianceFrameworkV21::new(
        "SOC 2".into(),
        "Service Organization Control 2".into(),
    );
    framework.add_requirement(ComplianceRequirementV21 {
        id: "CC6.1".into(),
        name: "Logical Access Controls".into(),
        description: "Implement logical access security controls".into(),
        category: "Security".into(),
        mandatory: true,
        verification_method: VerificationMethodV21::Automated,
    });
    framework.add_requirement(ComplianceRequirementV21 {
        id: "CC6.6".into(),
        name: "System Boundaries".into(),
        description: "Restrict logical access to system boundaries".into(),
        category: "Security".into(),
        mandatory: true,
        verification_method: VerificationMethodV21::Hybrid,
    });
    framework.add_requirement(ComplianceRequirementV21 {
        id: "CC7.1".into(),
        name: "Vulnerability Management".into(),
        description: "Detect and monitor for vulnerabilities".into(),
        category: "Security".into(),
        mandatory: true,
        verification_method: VerificationMethodV21::Automated,
    });
    framework.add_requirement(ComplianceRequirementV21 {
        id: "CC8.1".into(),
        name: "Change Management".into(),
        description: "Authorize and manage changes to systems".into(),
        category: "Availability".into(),
        mandatory: true,
        verification_method: VerificationMethodV21::Manual,
    });
    framework
}

pub fn create_iso27001_framework_v21() -> ComplianceFrameworkV21 {
    let mut framework = ComplianceFrameworkV21::new(
        "ISO 27001".into(),
        "Information Security Management System".into(),
    );
    framework.add_requirement(ComplianceRequirementV21 {
        id: "A.12.6.1".into(),
        name: "Technical Vulnerability Management".into(),
        description: "Obtain information about technical vulnerabilities".into(),
        category: "Operations Security".into(),
        mandatory: true,
        verification_method: VerificationMethodV21::Automated,
    });
    framework.add_requirement(ComplianceRequirementV21 {
        id: "A.14.2.1".into(),
        name: "Secure Development Policy".into(),
        description: "Establish secure development lifecycle".into(),
        category: "Acquisition Development".into(),
        mandatory: true,
        verification_method: VerificationMethodV21::Hybrid,
    });
    framework.add_requirement(ComplianceRequirementV21 {
        id: "A.18.2.1".into(),
        name: "Independent Review".into(),
        description: "Independent review of organization's ISMS".into(),
        category: "Compliance".into(),
        mandatory: true,
        verification_method: VerificationMethodV21::Manual,
    });
    framework
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_finding(requirement_id: &str, status: FindingStatusV21) -> ComplianceFindingV21 {
        ComplianceFindingV21 {
            id: uuid::Uuid::new_v4().to_string(),
            requirement_id: requirement_id.into(),
            status,
            details: "Test finding".into(),
            evidence: None,
            remediation: None,
        }
    }

    #[test]
    fn test_compliance_framework_v21_new() {
        let fw = ComplianceFrameworkV21::new("Test".into(), "Description".into());
        assert_eq!(fw.name, "Test");
        assert!(fw.enabled);
        assert!(fw.requirements.is_empty());
        assert_eq!(fw.version, "19.0");
    }

    #[test]
    fn test_compliance_framework_v21_with_version() {
        let fw = ComplianceFrameworkV21::new("Test".into(), "Desc".into())
            .with_version("20.0");
        assert_eq!(fw.version, "20.0");
    }

    #[test]
    fn test_compliance_framework_v21_increment_version() {
        let mut fw = ComplianceFrameworkV21::new("Test".into(), "Desc".into());
        fw.increment_version();
        assert_eq!(fw.version, "19.1");
        fw.increment_version();
        assert_eq!(fw.version, "19.2");
    }

    #[test]
    fn test_add_requirement() {
        let mut fw = ComplianceFrameworkV21::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV21 {
            id: "R1".into(),
            name: "Req 1".into(),
            description: "Desc".into(),
            category: "Security".into(),
            mandatory: true,
            verification_method: VerificationMethodV21::Automated,
        });
        assert_eq!(fw.requirements.len(), 1);
    }

    #[test]
    fn test_mandatory_requirements() {
        let mut fw = ComplianceFrameworkV21::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV21 {
            id: "R1".into(),
            name: "Mandatory".into(),
            description: "".into(),
            category: "Security".into(),
            mandatory: true,
            verification_method: VerificationMethodV21::Automated,
        });
        fw.add_requirement(ComplianceRequirementV21 {
            id: "R2".into(),
            name: "Optional".into(),
            description: "".into(),
            category: "Security".into(),
            mandatory: false,
            verification_method: VerificationMethodV21::Manual,
        });
        assert_eq!(fw.mandatory_requirements().len(), 1);
        assert_eq!(fw.mandatory_requirements()[0].id, "R1");
    }

    #[test]
    fn test_requirements_by_category() {
        let mut fw = ComplianceFrameworkV21::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV21 {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Security".into(),
            mandatory: true,
            verification_method: VerificationMethodV21::Automated,
        });
        fw.add_requirement(ComplianceRequirementV21 {
            id: "R2".into(),
            name: "B".into(),
            description: "".into(),
            category: "Privacy".into(),
            mandatory: true,
            verification_method: VerificationMethodV21::Automated,
        });
        assert_eq!(fw.requirements_by_category("Security").len(), 1);
        assert_eq!(fw.requirements_by_category("Privacy").len(), 1);
        assert!(fw.requirements_by_category("Nonexistent").is_empty());
    }

    #[test]
    fn test_framework_counts() {
        let mut fw = ComplianceFrameworkV21::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV21 {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV21::Automated,
        });
        fw.add_requirement(ComplianceRequirementV21 {
            id: "R2".into(),
            name: "B".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: false,
            verification_method: VerificationMethodV21::Automated,
        });
        assert_eq!(fw.requirement_count(), 2);
        assert_eq!(fw.mandatory_count(), 1);
    }

    #[test]
    fn test_assessment_v21_new() {
        let assessment = ComplianceAssessmentV21::new("fw-1".into());
        assert_eq!(assessment.framework_id, "fw-1");
        assert_eq!(assessment.status, AssessmentStatusV21::Pending);
        assert!(assessment.findings.is_empty());
        assert_eq!(assessment.score, 0);
        assert!(assessment.history.is_empty());
    }

    #[test]
    fn test_assessment_v21_with_repo() {
        let assessment = ComplianceAssessmentV21::new("fw-1".into())
            .with_repo("repo-1")
            .with_assessor("user-1");
        assert_eq!(assessment.repo_id.as_deref(), Some("repo-1"));
        assert_eq!(assessment.assessor_id.as_deref(), Some("user-1"));
    }

    #[test]
    fn test_assessment_v21_add_finding() {
        let mut assessment = ComplianceAssessmentV21::new("fw-1".into());
        let finding = ComplianceFindingV21 {
            id: uuid::Uuid::new_v4().to_string(),
            requirement_id: "req-1".into(),
            status: FindingStatusV21::Compliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        assessment.add_finding(finding);
        assert_eq!(assessment.findings.len(), 1);
        assert_eq!(assessment.history.len(), 1);
    }

    #[test]
    fn test_assessment_v21_complete() {
        let mut assessment = ComplianceAssessmentV21::new("fw-1".into());
        let finding = ComplianceFindingV21 {
            id: uuid::Uuid::new_v4().to_string(),
            requirement_id: "req-1".into(),
            status: FindingStatusV21::Compliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        assessment.add_finding(finding);
        assessment.complete(100);
        assert_eq!(assessment.status, AssessmentStatusV21::Passed);
        assert_eq!(assessment.score, 100);
        assert!(assessment.completed_at.is_some());
    }

    #[test]
    fn test_assessment_v21_complete_failed() {
        let mut assessment = ComplianceAssessmentV21::new("fw-1".into());
        let finding = ComplianceFindingV21 {
            id: uuid::Uuid::new_v4().to_string(),
            requirement_id: "req-1".into(),
            status: FindingStatusV21::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        assessment.add_finding(finding);
        assessment.complete(0);
        assert_eq!(assessment.status, AssessmentStatusV21::Failed);
    }

    #[test]
    fn test_assessment_compliance_rate() {
        let mut assessment = ComplianceAssessmentV21::new("fw-1".into());
        assessment.add_finding(make_finding("R1", FindingStatusV21::Compliant));
        assessment.add_finding(make_finding("R2", FindingStatusV21::NonCompliant));
        assert_eq!(assessment.compliance_rate(), 50.0);
    }

    #[test]
    fn test_assessment_non_compliant_count() {
        let mut assessment = ComplianceAssessmentV21::new("fw-1".into());
        assessment.add_finding(make_finding("R1", FindingStatusV21::Compliant));
        assessment.add_finding(make_finding("R2", FindingStatusV21::NonCompliant));
        assessment.add_finding(make_finding("R3", FindingStatusV21::NonCompliant));
        assert_eq!(assessment.non_compliant_count(), 2);
    }

    #[test]
    fn test_finding_is_compliant() {
        let f = make_finding("R1", FindingStatusV21::Compliant);
        assert!(f.is_compliant());
        let f = make_finding("R1", FindingStatusV21::NotApplicable);
        assert!(f.is_compliant());
        let f = make_finding("R1", FindingStatusV21::NonCompliant);
        assert!(!f.is_compliant());
    }

    #[test]
    fn test_finding_tracker_v21_new() {
        let tracker = FindingTrackerV21::new();
        assert!(tracker.findings.is_empty());
        assert!(tracker.remediation_deadlines.is_empty());
        assert!(tracker.assigned_to.is_empty());
        assert!(tracker.severity_scores.is_empty());
        assert!(tracker.history.is_empty());
    }

    #[test]
    fn test_finding_tracker_v21_add_finding() {
        let mut tracker = FindingTrackerV21::new();
        let finding = ComplianceFindingV21 {
            id: "f-1".into(),
            requirement_id: "req-1".into(),
            status: FindingStatusV21::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        tracker.add_finding(finding);
        assert_eq!(tracker.findings.len(), 1);
    }

    #[test]
    fn test_finding_tracker_v21_overdue() {
        let mut tracker = FindingTrackerV21::new();
        let finding = ComplianceFindingV21 {
            id: "f-1".into(),
            requirement_id: "req-1".into(),
            status: FindingStatusV21::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        tracker.add_finding(finding);
        tracker.set_deadline("f-1", Utc::now() - chrono::Duration::hours(1));
        assert_eq!(tracker.overdue_findings().len(), 1);
    }

    #[test]
    fn test_finding_tracker_v21_open_findings() {
        let mut tracker = FindingTrackerV21::new();
        let f1 = ComplianceFindingV21 {
            id: "f-1".into(),
            requirement_id: "req-1".into(),
            status: FindingStatusV21::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        let f2 = ComplianceFindingV21 {
            id: "f-2".into(),
            requirement_id: "req-2".into(),
            status: FindingStatusV21::Compliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        tracker.add_finding(f1);
        tracker.add_finding(f2);
        assert_eq!(tracker.open_findings().len(), 1);
    }

    #[test]
    fn test_finding_tracker_v21_assign() {
        let mut tracker = FindingTrackerV21::new();
        let finding = ComplianceFindingV21 {
            id: "f-1".into(),
            requirement_id: "req-1".into(),
            status: FindingStatusV21::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        tracker.add_finding(finding);
        tracker.assign("f-1", "user-1");
        assert_eq!(tracker.assigned_to.get("f-1").map(|s| s.as_str()), Some("user-1"));
    }

    #[test]
    fn test_finding_tracker_v21_update_status() {
        let mut tracker = FindingTrackerV21::new();
        let finding = ComplianceFindingV21 {
            id: "f-1".into(),
            requirement_id: "req-1".into(),
            status: FindingStatusV21::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        tracker.add_finding(finding);
        tracker
            .update_finding_status("f-1", FindingStatusV21::Compliant, Some("user-1".into()))
            .unwrap();
        assert_eq!(tracker.findings[0].status, FindingStatusV21::Compliant);
        assert_eq!(tracker.history.len(), 1);
        assert_eq!(tracker.history[0].old_status, FindingStatusV21::NonCompliant);
        assert_eq!(tracker.history[0].new_status, FindingStatusV21::Compliant);
    }

    #[test]
    fn test_finding_tracker_v21_finding_history() {
        let mut tracker = FindingTrackerV21::new();
        let finding = ComplianceFindingV21 {
            id: "f-1".into(),
            requirement_id: "req-1".into(),
            status: FindingStatusV21::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        tracker.add_finding(finding);
        tracker
            .update_finding_status("f-1", FindingStatusV21::Partial, None)
            .unwrap();
        tracker
            .update_finding_status("f-1", FindingStatusV21::Compliant, None)
            .unwrap();
        let history = tracker.finding_history("f-1");
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_finding_tracker_v21_findings_by_status() {
        let mut tracker = FindingTrackerV21::new();
        tracker.add_finding(make_finding("R1", FindingStatusV21::NonCompliant));
        tracker.add_finding(make_finding("R2", FindingStatusV21::Compliant));
        tracker.add_finding(make_finding("R3", FindingStatusV21::NonCompliant));
        assert_eq!(tracker.findings_by_severity(FindingStatusV21::NonCompliant).len(), 2);
        assert_eq!(tracker.findings_by_severity(FindingStatusV21::Compliant).len(), 1);
    }

    #[test]
    fn test_finding_tracker_v21_compute_severity_scores() {
        let mut tracker = FindingTrackerV21::new();
        let f1 = ComplianceFindingV21 {
            id: "f-1".into(),
            requirement_id: "CC6.1".into(),
            status: FindingStatusV21::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        let f2 = ComplianceFindingV21 {
            id: "f-2".into(),
            requirement_id: "CC6.1".into(),
            status: FindingStatusV21::Partial,
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
        let mut fw = ComplianceFrameworkV21::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV21 {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV21::Automated,
        });
        fw.add_requirement(ComplianceRequirementV21 {
            id: "R2".into(),
            name: "B".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV21::Automated,
        });
        let findings = vec![
            make_finding("R1", FindingStatusV21::Compliant),
            make_finding("R2", FindingStatusV21::Compliant),
        ];
        let result = ComplianceAssessorV21::assess(&fw, &findings);
        assert_eq!(result.score, 100);
        assert_eq!(result.status, AssessmentStatusV21::Passed);
        assert_eq!(result.compliant_count, 2);
    }

    #[test]
    fn test_assess_some_non_compliant() {
        let mut fw = ComplianceFrameworkV21::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV21 {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV21::Automated,
        });
        fw.add_requirement(ComplianceRequirementV21 {
            id: "R2".into(),
            name: "B".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV21::Automated,
        });
        let findings = vec![
            make_finding("R1", FindingStatusV21::Compliant),
            make_finding("R2", FindingStatusV21::NonCompliant),
        ];
        let result = ComplianceAssessorV21::assess(&fw, &findings);
        assert_eq!(result.score, 50);
        assert_eq!(result.status, AssessmentStatusV21::Failed);
        assert_eq!(result.non_compliant_count, 1);
    }

    #[test]
    fn test_assess_with_not_applicable() {
        let mut fw = ComplianceFrameworkV21::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV21 {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV21::Automated,
        });
        fw.add_requirement(ComplianceRequirementV21 {
            id: "R2".into(),
            name: "B".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV21::Automated,
        });
        let findings = vec![
            make_finding("R1", FindingStatusV21::Compliant),
            make_finding("R2", FindingStatusV21::NotApplicable),
        ];
        let result = ComplianceAssessorV21::assess(&fw, &findings);
        assert_eq!(result.score, 100);
        assert_eq!(result.status, AssessmentStatusV21::Passed);
        assert_eq!(result.not_applicable_count, 1);
    }

    #[test]
    fn test_assess_empty_requirements() {
        let fw = ComplianceFrameworkV21::new("Test".into(), "Desc".into());
        let result = ComplianceAssessorV21::assess(&fw, &[]);
        assert_eq!(result.score, 100);
        assert_eq!(result.status, AssessmentStatusV21::Passed);
    }

    #[test]
    fn test_assess_partial() {
        let mut fw = ComplianceFrameworkV21::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV21 {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethodV21::Automated,
        });
        let findings = vec![make_finding("R1", FindingStatusV21::Partial)];
        let result = ComplianceAssessorV21::assess(&fw, &findings);
        assert_eq!(result.status, AssessmentStatusV21::Partial);
    }

    #[test]
    fn test_soc2_framework_v21() {
        let fw = create_soc2_framework_v21();
        assert_eq!(fw.name, "SOC 2");
        assert!(!fw.requirements.is_empty());
        assert!(fw.mandatory_requirements().len() > 0);
    }

    #[test]
    fn test_iso27001_framework_v21() {
        let fw = create_iso27001_framework_v21();
        assert_eq!(fw.name, "ISO 27001");
        assert!(!fw.requirements.is_empty());
    }

    #[test]
    fn test_compliance_scoring_engine_v21() {
        let requirements = vec![
            ComplianceRequirementRecordV21::new("fw".into(), "R1".into(), "".into()),
            ComplianceRequirementRecordV21::new("fw".into(), "R2".into(), "".into()),
        ];
        let check_results = vec![
            ComplianceCheckResultV21::new("R1".into(), "a".into(), CheckStatusV21::Passed),
            ComplianceCheckResultV21::new("R2".into(), "a".into(), CheckStatusV21::Failed),
        ];
        let score = ComplianceScoringEngineV21::calculate_framework_score(&requirements, &check_results);
        assert_eq!(score.total_requirements, 2);
        assert_eq!(score.passed, 1);
        assert_eq!(score.failed, 1);
        assert_eq!(score.overall, 50);
    }

    #[test]
    fn test_compliance_scoring_engine_v21_all_passed() {
        let requirements = vec![
            ComplianceRequirementRecordV21::new("fw".into(), "R1".into(), "".into()),
        ];
        let check_results = vec![
            ComplianceCheckResultV21::new("R1".into(), "a".into(), CheckStatusV21::Passed),
        ];
        let score = ComplianceScoringEngineV21::calculate_framework_score(&requirements, &check_results);
        assert_eq!(score.overall, 100);
    }

    #[test]
    fn test_compliance_scoring_engine_v21_weighted() {
        let requirements = vec![
            ComplianceRequirementRecordV21::new("fw".into(), "R1".into(), "".into())
                .with_severity(RequirementSeverityV21::Critical),
            ComplianceRequirementRecordV21::new("fw".into(), "R2".into(), "".into())
                .with_severity(RequirementSeverityV21::Low),
        ];
        let check_results = vec![
            ComplianceCheckResultV21::new("R1".into(), "a".into(), CheckStatusV21::Passed),
            ComplianceCheckResultV21::new("R2".into(), "a".into(), CheckStatusV21::Failed),
        ];
        let score = ComplianceScoringEngineV21::calculate_weighted_score(&requirements, &check_results);
        assert!((score - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_compliance_scoring_engine_v21_inherited() {
        let requirements = vec![
            ComplianceRequirementRecordV21::new("fw".into(), "R1".into(), "".into()),
            ComplianceRequirementRecordV21::new("fw".into(), "R2".into(), "".into()),
        ];
        let check_results = vec![
            ComplianceCheckResultV21::new("R1".into(), "a".into(), CheckStatusV21::Passed),
            ComplianceCheckResultV21::new("R2".into(), "a".into(), CheckStatusV21::Inherited),
        ];
        let score = ComplianceScoringEngineV21::calculate_framework_score(&requirements, &check_results);
        assert_eq!(score.overall, 100);
        assert_eq!(score.inherited, 1);
    }

    #[test]
    fn test_requirement_risk_weight() {
        let req = ComplianceRequirementRecordV21::new("fw".into(), "R1".into(), "".into())
            .with_severity(RequirementSeverityV21::Critical);
        assert_eq!(req.risk_weight(), 4.0);
        let req = ComplianceRequirementRecordV21::new("fw".into(), "R1".into(), "".into())
            .with_severity(RequirementSeverityV21::Low);
        assert_eq!(req.risk_weight(), 1.0);
    }
}
