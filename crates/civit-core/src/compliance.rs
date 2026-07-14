#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFramework {
    pub id: String,
    pub name: String,
    pub description: String,
    pub requirements: Vec<ComplianceRequirement>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub mandatory: bool,
    pub verification_method: VerificationMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationMethod {
    Automated,
    Manual,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentStatus {
    Pending,
    Running,
    Passed,
    Failed,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAssessment {
    pub id: String,
    pub framework_id: String,
    pub repo_id: Option<String>,
    pub status: AssessmentStatus,
    pub findings: Vec<ComplianceFinding>,
    pub score: u32,
    pub assessor_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFinding {
    pub id: String,
    pub requirement_id: String,
    pub status: FindingStatus,
    pub details: String,
    pub evidence: Option<String>,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatus {
    Compliant,
    NonCompliant,
    Partial,
    NotApplicable,
}

impl ComplianceFinding {
    pub fn is_compliant(&self) -> bool {
        self.status == FindingStatus::Compliant || self.status == FindingStatus::NotApplicable
    }
}

impl ComplianceFramework {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            requirements: Vec::new(),
            enabled: true,
            created_at: Utc::now(),
        }
    }

    pub fn add_requirement(&mut self, requirement: ComplianceRequirement) {
        self.requirements.push(requirement);
    }

    pub fn mandatory_requirements(&self) -> Vec<&ComplianceRequirement> {
        self.requirements.iter().filter(|r| r.mandatory).collect()
    }

    pub fn requirements_by_category(&self, category: &str) -> Vec<&ComplianceRequirement> {
        self.requirements
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }
}

pub struct ComplianceAssessor;

impl ComplianceAssessor {
    pub fn assess(
        framework: &ComplianceFramework,
        findings: &[ComplianceFinding],
    ) -> AssessmentResult {
        let total_requirements = framework.requirements.len() as u32;
        if total_requirements == 0 {
            return AssessmentResult {
                score: 100,
                status: AssessmentStatus::Passed,
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
                FindingStatus::Compliant => compliant += 1,
                FindingStatus::NonCompliant => non_compliant += 1,
                FindingStatus::Partial => partial += 1,
                FindingStatus::NotApplicable => not_applicable += 1,
            }
        }

        let applicable = total_requirements - not_applicable;
        let score = if applicable > 0 {
            ((compliant as f64 / applicable as f64) * 100.0) as u32
        } else {
            100
        };

        let status = if non_compliant > 0 {
            AssessmentStatus::Failed
        } else if partial > 0 {
            AssessmentStatus::Partial
        } else {
            AssessmentStatus::Passed
        };

        AssessmentResult {
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
pub struct AssessmentResult {
    pub score: u32,
    pub status: AssessmentStatus,
    pub compliant_count: u32,
    pub non_compliant_count: u32,
    pub partial_count: u32,
    pub not_applicable_count: u32,
}

pub fn create_soc2_framework() -> ComplianceFramework {
    let mut framework = ComplianceFramework::new(
        "SOC 2".into(),
        "Service Organization Control 2".into(),
    );
    framework.add_requirement(ComplianceRequirement {
        id: "CC6.1".into(),
        name: "Logical Access Controls".into(),
        description: "Implement logical access security controls".into(),
        category: "Security".into(),
        mandatory: true,
        verification_method: VerificationMethod::Automated,
    });
    framework.add_requirement(ComplianceRequirement {
        id: "CC6.6".into(),
        name: "System Boundaries".into(),
        description: "Restrict logical access to system boundaries".into(),
        category: "Security".into(),
        mandatory: true,
        verification_method: VerificationMethod::Hybrid,
    });
    framework.add_requirement(ComplianceRequirement {
        id: "CC7.1".into(),
        name: "Vulnerability Management".into(),
        description: "Detect and monitor for vulnerabilities".into(),
        category: "Security".into(),
        mandatory: true,
        verification_method: VerificationMethod::Automated,
    });
    framework.add_requirement(ComplianceRequirement {
        id: "CC8.1".into(),
        name: "Change Management".into(),
        description: "Authorize and manage changes to systems".into(),
        category: "Availability".into(),
        mandatory: true,
        verification_method: VerificationMethod::Manual,
    });
    framework
}

pub fn create_iso27001_framework() -> ComplianceFramework {
    let mut framework = ComplianceFramework::new(
        "ISO 27001".into(),
        "Information Security Management System".into(),
    );
    framework.add_requirement(ComplianceRequirement {
        id: "A.12.6.1".into(),
        name: "Technical Vulnerability Management".into(),
        description: "Obtain information about technical vulnerabilities".into(),
        category: "Operations Security".into(),
        mandatory: true,
        verification_method: VerificationMethod::Automated,
    });
    framework.add_requirement(ComplianceRequirement {
        id: "A.14.2.1".into(),
        name: "Secure Development Policy".into(),
        description: "Establish secure development lifecycle".into(),
        category: "Acquisition Development".into(),
        mandatory: true,
        verification_method: VerificationMethod::Hybrid,
    });
    framework.add_requirement(ComplianceRequirement {
        id: "A.18.2.1".into(),
        name: "Independent Review".into(),
        description: "Independent review of organization's ISMS".into(),
        category: "Compliance".into(),
        mandatory: true,
        verification_method: VerificationMethod::Manual,
    });
    framework
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_finding(requirement_id: &str, status: FindingStatus) -> ComplianceFinding {
        ComplianceFinding {
            id: uuid::Uuid::new_v4().to_string(),
            requirement_id: requirement_id.into(),
            status,
            details: "Test finding".into(),
            evidence: None,
            remediation: None,
        }
    }

    #[test]
    fn test_compliance_framework_new() {
        let fw = ComplianceFramework::new("Test".into(), "Description".into());
        assert_eq!(fw.name, "Test");
        assert!(fw.enabled);
        assert!(fw.requirements.is_empty());
    }

    #[test]
    fn test_add_requirement() {
        let mut fw = ComplianceFramework::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirement {
            id: "R1".into(),
            name: "Req 1".into(),
            description: "Desc".into(),
            category: "Security".into(),
            mandatory: true,
            verification_method: VerificationMethod::Automated,
        });
        assert_eq!(fw.requirements.len(), 1);
    }

    #[test]
    fn test_mandatory_requirements() {
        let mut fw = ComplianceFramework::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirement {
            id: "R1".into(),
            name: "Mandatory".into(),
            description: "".into(),
            category: "Security".into(),
            mandatory: true,
            verification_method: VerificationMethod::Automated,
        });
        fw.add_requirement(ComplianceRequirement {
            id: "R2".into(),
            name: "Optional".into(),
            description: "".into(),
            category: "Security".into(),
            mandatory: false,
            verification_method: VerificationMethod::Manual,
        });
        assert_eq!(fw.mandatory_requirements().len(), 1);
        assert_eq!(fw.mandatory_requirements()[0].id, "R1");
    }

    #[test]
    fn test_requirements_by_category() {
        let mut fw = ComplianceFramework::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirement {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Security".into(),
            mandatory: true,
            verification_method: VerificationMethod::Automated,
        });
        fw.add_requirement(ComplianceRequirement {
            id: "R2".into(),
            name: "B".into(),
            description: "".into(),
            category: "Privacy".into(),
            mandatory: true,
            verification_method: VerificationMethod::Automated,
        });
        assert_eq!(fw.requirements_by_category("Security").len(), 1);
        assert_eq!(fw.requirements_by_category("Privacy").len(), 1);
        assert!(fw.requirements_by_category("Nonexistent").is_empty());
    }

    #[test]
    fn test_assess_all_compliant() {
        let mut fw = ComplianceFramework::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirement {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethod::Automated,
        });
        fw.add_requirement(ComplianceRequirement {
            id: "R2".into(),
            name: "B".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethod::Automated,
        });
        let findings = vec![
            make_finding("R1", FindingStatus::Compliant),
            make_finding("R2", FindingStatus::Compliant),
        ];
        let result = ComplianceAssessor::assess(&fw, &findings);
        assert_eq!(result.score, 100);
        assert_eq!(result.status, AssessmentStatus::Passed);
        assert_eq!(result.compliant_count, 2);
    }

    #[test]
    fn test_assess_some_non_compliant() {
        let mut fw = ComplianceFramework::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirement {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethod::Automated,
        });
        fw.add_requirement(ComplianceRequirement {
            id: "R2".into(),
            name: "B".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethod::Automated,
        });
        let findings = vec![
            make_finding("R1", FindingStatus::Compliant),
            make_finding("R2", FindingStatus::NonCompliant),
        ];
        let result = ComplianceAssessor::assess(&fw, &findings);
        assert_eq!(result.score, 50);
        assert_eq!(result.status, AssessmentStatus::Failed);
        assert_eq!(result.non_compliant_count, 1);
    }

    #[test]
    fn test_assess_with_not_applicable() {
        let mut fw = ComplianceFramework::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirement {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethod::Automated,
        });
        fw.add_requirement(ComplianceRequirement {
            id: "R2".into(),
            name: "B".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethod::Automated,
        });
        let findings = vec![
            make_finding("R1", FindingStatus::Compliant),
            make_finding("R2", FindingStatus::NotApplicable),
        ];
        let result = ComplianceAssessor::assess(&fw, &findings);
        assert_eq!(result.score, 100);
        assert_eq!(result.status, AssessmentStatus::Passed);
        assert_eq!(result.not_applicable_count, 1);
    }

    #[test]
    fn test_assess_empty_requirements() {
        let fw = ComplianceFramework::new("Test".into(), "Desc".into());
        let result = ComplianceAssessor::assess(&fw, &[]);
        assert_eq!(result.score, 100);
        assert_eq!(result.status, AssessmentStatus::Passed);
    }

    #[test]
    fn test_assess_partial() {
        let mut fw = ComplianceFramework::new("Test".into(), "Desc".into());
        fw.add_requirement(ComplianceRequirement {
            id: "R1".into(),
            name: "A".into(),
            description: "".into(),
            category: "Sec".into(),
            mandatory: true,
            verification_method: VerificationMethod::Automated,
        });
        let findings = vec![make_finding("R1", FindingStatus::Partial)];
        let result = ComplianceAssessor::assess(&fw, &findings);
        assert_eq!(result.status, AssessmentStatus::Partial);
    }

    #[test]
    fn test_finding_is_compliant() {
        let f = make_finding("R1", FindingStatus::Compliant);
        assert!(f.is_compliant());
        let f = make_finding("R1", FindingStatus::NotApplicable);
        assert!(f.is_compliant());
        let f = make_finding("R1", FindingStatus::NonCompliant);
        assert!(!f.is_compliant());
    }

    #[test]
    fn test_soc2_framework() {
        let fw = create_soc2_framework();
        assert_eq!(fw.name, "SOC 2");
        assert!(!fw.requirements.is_empty());
        assert!(fw.mandatory_requirements().len() > 0);
    }

    #[test]
    fn test_iso27001_framework() {
        let fw = create_iso27001_framework();
        assert_eq!(fw.name, "ISO 27001");
        assert!(!fw.requirements.is_empty());
    }

    #[test]
    fn test_verification_method_serialization() {
        assert_eq!(
            serde_json::to_string(&VerificationMethod::Automated).unwrap(),
            "\"automated\""
        );
        assert_eq!(
            serde_json::to_string(&VerificationMethod::Manual).unwrap(),
            "\"manual\""
        );
    }

    #[test]
    fn test_assessment_status_serialization() {
        assert_eq!(
            serde_json::to_string(&AssessmentStatus::Passed).unwrap(),
            "\"passed\""
        );
        assert_eq!(
            serde_json::to_string(&AssessmentStatus::Failed).unwrap(),
            "\"failed\""
        );
    }

    #[test]
    fn test_finding_status_serialization() {
        assert_eq!(
            serde_json::to_string(&FindingStatus::Compliant).unwrap(),
            "\"compliant\""
        );
        assert_eq!(
            serde_json::to_string(&FindingStatus::NonCompliant).unwrap(),
            "\"non_compliant\""
        );
    }
}
