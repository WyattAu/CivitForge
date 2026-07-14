#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkV6 {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub requirements: Vec<ComplianceRequirementV6>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ComplianceFrameworkV6 {
    pub fn new(name: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            version: "4.0".into(),
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

    pub fn add_requirement(&mut self, requirement: ComplianceRequirementV6) {
        self.requirements.push(requirement);
    }

    pub fn mandatory_requirements(&self) -> Vec<&ComplianceRequirementV6> {
        self.requirements.iter().filter(|r| r.mandatory).collect()
    }

    pub fn requirements_by_category(&self, category: &str) -> Vec<&ComplianceRequirementV6> {
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
                self.updated_at = Utc::now();
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirementV6 {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub mandatory: bool,
    pub weight: u32,
    pub verification_method: VerificationMethodV6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationMethodV6 {
    Automated,
    Manual,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentStatusV6 {
    Pending,
    Running,
    Passed,
    Failed,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAssessmentV6 {
    pub id: String,
    pub framework_id: String,
    pub repo_id: Option<String>,
    pub status: AssessmentStatusV6,
    pub findings: Vec<ComplianceFindingV6>,
    pub score: u32,
    pub assessor_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub history: Vec<AssessmentSnapshotV6>,
}

impl ComplianceAssessmentV6 {
    pub fn new(framework_id: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            framework_id,
            repo_id: None,
            status: AssessmentStatusV6::Pending,
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

    pub fn add_finding(&mut self, finding: ComplianceFindingV6) {
        self.findings.push(finding);
        self.snapshot_history();
    }

    pub fn complete(&mut self, score: u32) {
        self.status = if self
            .findings
            .iter()
            .any(|f| f.status == FindingStatusV6::NonCompliant)
        {
            AssessmentStatusV6::Failed
        } else if self
            .findings
            .iter()
            .any(|f| f.status == FindingStatusV6::Partial)
        {
            AssessmentStatusV6::Partial
        } else {
            AssessmentStatusV6::Passed
        };
        self.score = score;
        self.completed_at = Some(Utc::now());
        self.snapshot_history();
    }

    fn snapshot_history(&mut self) {
        self.history.push(AssessmentSnapshotV6 {
            status: self.status,
            score: self.score,
            findings_count: self.findings.len() as u32,
            timestamp: Utc::now(),
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentSnapshotV6 {
    pub status: AssessmentStatusV6,
    pub score: u32,
    pub findings_count: u32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFindingV6 {
    pub id: String,
    pub requirement_id: String,
    pub status: FindingStatusV6,
    pub details: String,
    pub evidence: Option<String>,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatusV6 {
    Compliant,
    NonCompliant,
    Partial,
    NotApplicable,
}

impl ComplianceFindingV6 {
    pub fn is_compliant(&self) -> bool {
        self.status == FindingStatusV6::Compliant || self.status == FindingStatusV6::NotApplicable
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingTrackerV6 {
    pub findings: Vec<ComplianceFindingV6>,
    pub remediation_deadlines: HashMap<String, DateTime<Utc>>,
    pub assigned_to: HashMap<String, String>,
    pub finding_history: Vec<FindingHistoryEntryV6>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingHistoryEntryV6 {
    pub finding_id: String,
    pub previous_status: FindingStatusV6,
    pub new_status: FindingStatusV6,
    pub changed_by: Option<String>,
    pub changed_at: DateTime<Utc>,
}

impl FindingTrackerV6 {
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
            remediation_deadlines: HashMap::new(),
            assigned_to: HashMap::new(),
            finding_history: Vec::new(),
        }
    }

    pub fn add_finding(&mut self, finding: ComplianceFindingV6) {
        self.findings.push(finding);
    }

    pub fn update_finding_status(
        &mut self,
        finding_id: &str,
        new_status: FindingStatusV6,
        changed_by: Option<String>,
    ) -> Result<(), String> {
        let finding = self
            .findings
            .iter_mut()
            .find(|f| f.id == finding_id)
            .ok_or("Finding not found")?;
        let previous_status = finding.status;
        finding.status = new_status;
        self.finding_history.push(FindingHistoryEntryV6 {
            finding_id: finding_id.into(),
            previous_status,
            new_status,
            changed_by,
            changed_at: Utc::now(),
        });
        Ok(())
    }

    pub fn set_deadline(&mut self, finding_id: &str, deadline: DateTime<Utc>) {
        self.remediation_deadlines
            .insert(finding_id.into(), deadline);
    }

    pub fn assign(&mut self, finding_id: &str, user_id: &str) {
        self.assigned_to
            .insert(finding_id.into(), user_id.into());
    }

    pub fn overdue_findings(&self) -> Vec<&ComplianceFindingV6> {
        let now = Utc::now();
        self.findings
            .iter()
            .filter(|f| {
                self.remediation_deadlines
                    .get(&f.id)
                    .is_some_and(|d| *d < now)
            })
            .collect()
    }

    pub fn open_findings(&self) -> Vec<&ComplianceFindingV6> {
        self.findings
            .iter()
            .filter(|f| {
                f.status == FindingStatusV6::NonCompliant || f.status == FindingStatusV6::Partial
            })
            .collect()
    }

    pub fn get_finding_history(&self, finding_id: &str) -> Vec<&FindingHistoryEntryV6> {
        self.finding_history
            .iter()
            .filter(|h| h.finding_id == finding_id)
            .collect()
    }
}

impl Default for FindingTrackerV6 {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceScoringEngineV6;

impl ComplianceScoringEngineV6 {
    pub fn calculate_framework_score(
        requirements: &[ComplianceRequirementV6],
        findings: &[ComplianceFindingV6],
    ) -> ComplianceScoreV6 {
        let total = requirements.len() as u32;
        if total == 0 {
            return ComplianceScoreV6 {
                overall: 100,
                weighted_score: 100.0,
                by_category: HashMap::new(),
                total_requirements: 0,
                passed: 0,
                failed: 0,
                partial: 0,
                skipped: 0,
            };
        }

        let mut passed = 0u32;
        let mut failed = 0u32;
        let mut partial = 0u32;
        let mut not_applicable = 0u32;
        let mut total_weight = 0u32;
        let mut earned_weight = 0u32;
        let mut by_category: HashMap<String, (u32, u32)> = HashMap::new();

        for finding in findings {
            let req_weight = requirements
                .iter()
                .find(|r| r.id == finding.requirement_id)
                .map(|r| r.weight)
                .unwrap_or(1);
            let req_category = requirements
                .iter()
                .find(|r| r.id == finding.requirement_id)
                .map(|r| r.category.clone())
                .unwrap_or_default();

            match finding.status {
                FindingStatusV6::Compliant => {
                    passed += 1;
                    earned_weight += req_weight;
                }
                FindingStatusV6::NonCompliant => {
                    failed += 1;
                }
                FindingStatusV6::Partial => {
                    partial += 1;
                    earned_weight += req_weight / 2;
                }
                FindingStatusV6::NotApplicable => {
                    not_applicable += 1;
                }
            }

            total_weight += req_weight;

            let entry = by_category
                .entry(req_category)
                .or_insert((0, 0));
            match finding.status {
                FindingStatusV6::Compliant => entry.0 += 1,
                FindingStatusV6::NonCompliant => entry.1 += 1,
                FindingStatusV6::Partial => {
                    entry.0 += 1;
                    entry.1 += 1;
                }
                FindingStatusV6::NotApplicable => {}
            }
        }

        let applicable = total - not_applicable;
        let overall = if applicable > 0 {
            ((passed as f64 / applicable as f64) * 100.0) as u32
        } else {
            100
        };

        let weighted_score = if total_weight > 0 {
            (earned_weight as f64 / total_weight as f64) * 100.0
        } else {
            100.0
        };

        ComplianceScoreV6 {
            overall,
            weighted_score,
            by_category,
            total_requirements: total,
            passed,
            failed,
            partial,
            skipped: not_applicable,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceScoreV6 {
    pub overall: u32,
    pub weighted_score: f64,
    pub by_category: HashMap<String, (u32, u32)>,
    pub total_requirements: u32,
    pub passed: u32,
    pub failed: u32,
    pub partial: u32,
    pub skipped: u32,
}

pub struct ComplianceAssessorV6;

impl ComplianceAssessorV6 {
    pub fn assess(
        framework: &ComplianceFrameworkV6,
        findings: &[ComplianceFindingV6],
    ) -> AssessmentResultV6 {
        let total_requirements = framework.requirements.len() as u32;
        if total_requirements == 0 {
            return AssessmentResultV6 {
                score: 100,
                status: AssessmentStatusV6::Passed,
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
                FindingStatusV6::Compliant => compliant += 1,
                FindingStatusV6::NonCompliant => non_compliant += 1,
                FindingStatusV6::Partial => partial += 1,
                FindingStatusV6::NotApplicable => not_applicable += 1,
            }
        }

        let applicable = total_requirements - not_applicable;
        let score = if applicable > 0 {
            ((compliant as f64 / applicable as f64) * 100.0) as u32
        } else {
            100
        };

        let status = if non_compliant > 0 {
            AssessmentStatusV6::Failed
        } else if partial > 0 {
            AssessmentStatusV6::Partial
        } else {
            AssessmentStatusV6::Passed
        };

        AssessmentResultV6 {
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
pub struct AssessmentResultV6 {
    pub score: u32,
    pub status: AssessmentStatusV6,
    pub compliant_count: u32,
    pub non_compliant_count: u32,
    pub partial_count: u32,
    pub not_applicable_count: u32,
}

pub fn create_soc2_framework_v6() -> ComplianceFrameworkV6 {
    let mut framework = ComplianceFrameworkV6::new(
        "SOC 2".into(),
        "Service Organization Control 2".into(),
    );
    framework.add_requirement(ComplianceRequirementV6 {
        id: "CC6.1".into(),
        name: "Logical Access Controls".into(),
        description: "Implement logical access security controls".into(),
        category: "Security".into(),
        mandatory: true,
        weight: 4,
        verification_method: VerificationMethodV6::Automated,
    });
    framework.add_requirement(ComplianceRequirementV6 {
        id: "CC6.6".into(),
        name: "System Boundaries".into(),
        description: "Restrict logical access to system boundaries".into(),
        category: "Security".into(),
        mandatory: true,
        weight: 3,
        verification_method: VerificationMethodV6::Hybrid,
    });
    framework.add_requirement(ComplianceRequirementV6 {
        id: "CC7.1".into(),
        name: "Vulnerability Management".into(),
        description: "Detect and monitor for vulnerabilities".into(),
        category: "Security".into(),
        mandatory: true,
        weight: 4,
        verification_method: VerificationMethodV6::Automated,
    });
    framework.add_requirement(ComplianceRequirementV6 {
        id: "CC8.1".into(),
        name: "Change Management".into(),
        description: "Authorize and manage changes to systems".into(),
        category: "Availability".into(),
        mandatory: true,
        weight: 3,
        verification_method: VerificationMethodV6::Manual,
    });
    framework
}

pub fn create_iso27001_framework_v6() -> ComplianceFrameworkV6 {
    let mut framework = ComplianceFrameworkV6::new(
        "ISO 27001".into(),
        "Information Security Management System".into(),
    );
    framework.add_requirement(ComplianceRequirementV6 {
        id: "A.12.6.1".into(),
        name: "Technical Vulnerability Management".into(),
        description: "Obtain information about technical vulnerabilities".into(),
        category: "Operations Security".into(),
        mandatory: true,
        weight: 4,
        verification_method: VerificationMethodV6::Automated,
    });
    framework.add_requirement(ComplianceRequirementV6 {
        id: "A.14.2.1".into(),
        name: "Secure Development Policy".into(),
        description: "Establish secure development lifecycle".into(),
        category: "Acquisition Development".into(),
        mandatory: true,
        weight: 4,
        verification_method: VerificationMethodV6::Hybrid,
    });
    framework.add_requirement(ComplianceRequirementV6 {
        id: "A.18.2.1".into(),
        name: "Independent Review".into(),
        description: "Independent review of organization's ISMS".into(),
        category: "Compliance".into(),
        mandatory: true,
        weight: 3,
        verification_method: VerificationMethodV6::Manual,
    });
    framework
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_finding(requirement_id: &str, status: FindingStatusV6) -> ComplianceFindingV6 {
        ComplianceFindingV6 {
            id: uuid::Uuid::new_v4().to_string(),
            requirement_id: requirement_id.into(),
            status,
            details: "Test finding".into(),
            evidence: None,
            remediation: None,
        }
    }

    #[test]
    fn test_framework_new() {
        let fw = ComplianceFrameworkV6::new("Test".into(), "Description".into());
        assert_eq!(fw.name, "Test");
        assert_eq!(fw.version, "4.0");
        assert!(fw.enabled);
        assert!(fw.requirements.is_empty());
    }

    #[test]
    fn test_framework_with_version() {
        let fw = ComplianceFrameworkV6::new("Test".into(), "Desc".into()).with_version("2.1");
        assert_eq!(fw.version, "2.1");
    }

    #[test]
    fn test_framework_increment_version() {
        let mut fw = ComplianceFrameworkV6::new("Test".into(), "Desc".into());
        fw.increment_version();
        assert_eq!(fw.version, "4.1");
        fw.increment_version();
        assert_eq!(fw.version, "4.2");
    }

    #[test]
    fn test_add_requirement() {
        let mut fw = ComplianceFrameworkV6::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV6 {
            id: "R1".into(),
            name: "Req 1".into(),
            description: "Desc".into(),
            category: "Security".into(),
            mandatory: true,
            weight: 3,
            verification_method: VerificationMethodV6::Automated,
        });
        assert_eq!(fw.requirements.len(), 1);
    }

    #[test]
    fn test_mandatory_requirements() {
        let mut fw = ComplianceFrameworkV6::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV6 {
            id: "R1".into(),
            name: "Mandatory".into(),
            description: "".into(),
            category: "Security".into(),
            mandatory: true,
            weight: 3,
            verification_method: VerificationMethodV6::Automated,
        });
        fw.add_requirement(ComplianceRequirementV6 {
            id: "R2".into(),
            name: "Optional".into(),
            description: "".into(),
            category: "Security".into(),
            mandatory: false,
            weight: 1,
            verification_method: VerificationMethodV6::Manual,
        });
        assert_eq!(fw.mandatory_requirements().len(), 1);
        assert_eq!(fw.mandatory_requirements()[0].id, "R1");
    }

    #[test]
    fn test_assessment_new() {
        let assessment = ComplianceAssessmentV6::new("fw-1".into());
        assert_eq!(assessment.framework_id, "fw-1");
        assert_eq!(assessment.status, AssessmentStatusV6::Pending);
        assert!(assessment.findings.is_empty());
        assert_eq!(assessment.score, 0);
        assert!(assessment.history.is_empty());
    }

    #[test]
    fn test_assessment_with_repo() {
        let assessment = ComplianceAssessmentV6::new("fw-1".into())
            .with_repo("repo-1")
            .with_assessor("user-1");
        assert_eq!(assessment.repo_id.as_deref(), Some("repo-1"));
        assert_eq!(assessment.assessor_id.as_deref(), Some("user-1"));
    }

    #[test]
    fn test_assessment_add_finding() {
        let mut assessment = ComplianceAssessmentV6::new("fw-1".into());
        let finding = make_finding("req-1", FindingStatusV6::Compliant);
        assessment.add_finding(finding);
        assert_eq!(assessment.findings.len(), 1);
        assert_eq!(assessment.history.len(), 1);
    }

    #[test]
    fn test_assessment_complete() {
        let mut assessment = ComplianceAssessmentV6::new("fw-1".into());
        let finding = make_finding("req-1", FindingStatusV6::Compliant);
        assessment.add_finding(finding);
        assessment.complete(100);
        assert_eq!(assessment.status, AssessmentStatusV6::Passed);
        assert_eq!(assessment.score, 100);
        assert!(assessment.completed_at.is_some());
    }

    #[test]
    fn test_assessment_complete_failed() {
        let mut assessment = ComplianceAssessmentV6::new("fw-1".into());
        let finding = make_finding("req-1", FindingStatusV6::NonCompliant);
        assessment.add_finding(finding);
        assessment.complete(0);
        assert_eq!(assessment.status, AssessmentStatusV6::Failed);
    }

    #[test]
    fn test_finding_is_compliant() {
        let f = make_finding("R1", FindingStatusV6::Compliant);
        assert!(f.is_compliant());
        let f = make_finding("R1", FindingStatusV6::NotApplicable);
        assert!(f.is_compliant());
        let f = make_finding("R1", FindingStatusV6::NonCompliant);
        assert!(!f.is_compliant());
    }

    #[test]
    fn test_finding_tracker_new() {
        let tracker = FindingTrackerV6::new();
        assert!(tracker.findings.is_empty());
        assert!(tracker.remediation_deadlines.is_empty());
        assert!(tracker.assigned_to.is_empty());
    }

    #[test]
    fn test_finding_tracker_add_finding() {
        let mut tracker = FindingTrackerV6::new();
        let finding = make_finding("req-1", FindingStatusV6::NonCompliant);
        tracker.add_finding(finding);
        assert_eq!(tracker.findings.len(), 1);
    }

    #[test]
    fn test_finding_tracker_overdue() {
        let mut tracker = FindingTrackerV6::new();
        let finding = make_finding("f-1", FindingStatusV6::NonCompliant);
        tracker.add_finding(finding);
        tracker.set_deadline("f-1", Utc::now() - chrono::Duration::hours(1));
        assert_eq!(tracker.overdue_findings().len(), 1);
    }

    #[test]
    fn test_finding_tracker_open_findings() {
        let mut tracker = FindingTrackerV6::new();
        tracker.add_finding(make_finding("f-1", FindingStatusV6::NonCompliant));
        tracker.add_finding(make_finding("f-2", FindingStatusV6::Compliant));
        assert_eq!(tracker.open_findings().len(), 1);
    }

    #[test]
    fn test_finding_tracker_assign() {
        let mut tracker = FindingTrackerV6::new();
        tracker.add_finding(make_finding("f-1", FindingStatusV6::NonCompliant));
        tracker.assign("f-1", "user-1");
        assert_eq!(
            tracker.assigned_to.get("f-1").map(|s| s.as_str()),
            Some("user-1")
        );
    }

    #[test]
    fn test_finding_tracker_update_status() {
        let mut tracker = FindingTrackerV6::new();
        let mut finding = make_finding("f-1", FindingStatusV6::NonCompliant);
        finding.id = "f-1".into();
        tracker.add_finding(finding);
        tracker
            .update_finding_status("f-1", FindingStatusV6::Compliant, Some("user-1".into()))
            .unwrap();
        assert_eq!(tracker.findings[0].status, FindingStatusV6::Compliant);
        assert_eq!(tracker.finding_history.len(), 1);
    }

    #[test]
    fn test_finding_tracker_get_history() {
        let mut tracker = FindingTrackerV6::new();
        let mut finding = make_finding("f-1", FindingStatusV6::NonCompliant);
        finding.id = "f-1".into();
        tracker.add_finding(finding);
        tracker
            .update_finding_status("f-1", FindingStatusV6::Partial, None)
            .unwrap();
        tracker
            .update_finding_status("f-1", FindingStatusV6::Compliant, None)
            .unwrap();
        let history = tracker.get_finding_history("f-1");
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_assess_all_compliant() {
        let mut fw = ComplianceFrameworkV6::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV6 {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            weight: 3,
            verification_method: VerificationMethodV6::Automated,
        });
        let findings = vec![make_finding("R1", FindingStatusV6::Compliant)];
        let result = ComplianceAssessorV6::assess(&fw, &findings);
        assert_eq!(result.score, 100);
        assert_eq!(result.status, AssessmentStatusV6::Passed);
        assert_eq!(result.compliant_count, 1);
    }

    #[test]
    fn test_assess_some_non_compliant() {
        let mut fw = ComplianceFrameworkV6::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirementV6 {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            weight: 3,
            verification_method: VerificationMethodV6::Automated,
        });
        fw.add_requirement(ComplianceRequirementV6 {
            id: "R2".into(),
            name: "B".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            weight: 3,
            verification_method: VerificationMethodV6::Automated,
        });
        let findings = vec![
            make_finding("R1", FindingStatusV6::Compliant),
            make_finding("R2", FindingStatusV6::NonCompliant),
        ];
        let result = ComplianceAssessorV6::assess(&fw, &findings);
        assert_eq!(result.score, 50);
        assert_eq!(result.status, AssessmentStatusV6::Failed);
        assert_eq!(result.non_compliant_count, 1);
    }

    #[test]
    fn test_scoring_engine_weighted() {
        let requirements = vec![
            ComplianceRequirementV6 {
                id: "R1".into(),
                name: "A".into(),
                description: "".into(),
                category: "Sec".into(),
                mandatory: true,
                weight: 4,
                verification_method: VerificationMethodV6::Automated,
            },
            ComplianceRequirementV6 {
                id: "R2".into(),
                name: "B".into(),
                description: "".into(),
                category: "Sec".into(),
                mandatory: true,
                weight: 1,
                verification_method: VerificationMethodV6::Automated,
            },
        ];
        let findings = vec![
            make_finding("R1", FindingStatusV6::Compliant),
            make_finding("R2", FindingStatusV6::NonCompliant),
        ];
        let score =
            ComplianceScoringEngineV6::calculate_framework_score(&requirements, &findings);
        assert_eq!(score.total_requirements, 2);
        assert_eq!(score.passed, 1);
        assert_eq!(score.failed, 1);
        assert_eq!(score.overall, 50);
        assert!((score.weighted_score - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_soc2_framework() {
        let fw = create_soc2_framework_v6();
        assert_eq!(fw.name, "SOC 2");
        assert_eq!(fw.version, "4.0");
        assert!(!fw.requirements.is_empty());
        assert!(fw.mandatory_requirements().len() > 0);
    }

    #[test]
    fn test_iso27001_framework() {
        let fw = create_iso27001_framework_v6();
        assert_eq!(fw.name, "ISO 27001");
        assert!(!fw.requirements.is_empty());
    }
}
