#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirementRecord {
    pub id: String,
    pub framework_id: String,
    pub requirement_id: String,
    pub description: String,
    pub severity: RequirementSeverity,
    pub automated_check: bool,
    pub check_config: HashMap<String, serde_json::Value>,
    pub evidence_config: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ComplianceRequirementRecord {
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
            severity: RequirementSeverity::Medium,
            automated_check: false,
            check_config: HashMap::new(),
            evidence_config: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_severity(mut self, severity: RequirementSeverity) -> Self {
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
pub enum RequirementSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceEvidence {
    pub id: String,
    pub requirement_id: String,
    pub assessment_id: String,
    pub evidence_type: EvidenceType,
    pub content: HashMap<String, serde_json::Value>,
    pub collected_by: Option<String>,
    pub collected_at: DateTime<Utc>,
}

impl ComplianceEvidence {
    pub fn new(
        requirement_id: String,
        assessment_id: String,
        evidence_type: EvidenceType,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    Manual,
    Automated,
    SystemGenerated,
    External,
    Inherited,
}

pub type EvidenceTypeV22 = EvidenceType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheckResult {
    pub id: String,
    pub requirement_id: String,
    pub assessment_id: String,
    pub status: CheckStatus,
    pub result_details: HashMap<String, serde_json::Value>,
    pub score: u32,
    pub executed_at: DateTime<Utc>,
}

impl ComplianceCheckResult {
    pub fn new(
        requirement_id: String,
        assessment_id: String,
        status: CheckStatus,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Pending,
    Running,
    Passed,
    Failed,
    Skipped,
    Inherited,
}

pub type CheckStatusV22 = CheckStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomatedCheckExecutor;

impl AutomatedCheckExecutor {
    pub fn execute_check(
        requirement: &ComplianceRequirementRecord,
        target_data: &str,
    ) -> ComplianceCheckResult {
        if !requirement.automated_check {
            return ComplianceCheckResult::new(
                requirement.id.clone(),
                String::new(),
                CheckStatus::Skipped,
            );
        }

        let passed = Self::evaluate_check(requirement, target_data);
        let status = if passed {
            CheckStatus::Passed
        } else {
            CheckStatus::Failed
        };

        let mut result = ComplianceCheckResult::new(
            requirement.id.clone(),
            String::new(),
            status,
        );
        result.score = if passed { 100 } else { 0 };
        result.result_details.insert(
            "automated_check".into(),
            serde_json::Value::Bool(true),
        );
        result
    }

    fn evaluate_check(requirement: &ComplianceRequirementRecord, target_data: &str) -> bool {
        if let Some(pattern) = requirement.check_config.get("pattern") {
            if let Some(pat_str) = pattern.as_str() {
                match regex::Regex::new(pat_str) {
                    Ok(re) => return re.is_match(target_data),
                    Err(_) => return false,
                }
            }
        }
        if let Some(keyword) = requirement.check_config.get("keyword") {
            if let Some(kw_str) = keyword.as_str() {
                return target_data.contains(kw_str);
            }
        }
        !target_data.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceScoringEngine;

impl ComplianceScoringEngine {
    pub fn calculate_framework_score(
        requirements: &[ComplianceRequirementRecord],
        check_results: &[ComplianceCheckResult],
    ) -> ComplianceScore {
        let total = requirements.len() as u32;
        if total == 0 {
            return ComplianceScore {
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
        let severity_scores: HashMap<String, (u32, u32)> = HashMap::new();

        for result in check_results {
            match result.status {
                CheckStatus::Passed => passed += 1,
                CheckStatus::Failed => failed += 1,
                CheckStatus::Skipped => skipped += 1,
                _ => {}
            }
        }

        let applicable = total - skipped;
        let overall = if applicable > 0 {
            ((passed as f64 / applicable as f64) * 100.0) as u32
        } else {
            100
        };

        ComplianceScore {
            overall,
            by_severity: severity_scores,
            total_requirements: total,
            passed,
            failed,
            skipped,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceScore {
    pub overall: u32,
    pub by_severity: HashMap<String, (u32, u32)>,
    pub total_requirements: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationMethod {
    Automated,
    Manual,
    Hybrid,
    Inherited,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkV2 {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub requirements: Vec<ComplianceRequirement>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

impl ComplianceFrameworkV2 {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            version: "1.0".into(),
            description,
            requirements: Vec::new(),
            enabled: true,
            created_at: Utc::now(),
        }
    }

    pub fn with_version(mut self, version: &str) -> Self {
        self.version = version.into();
        self
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

    pub fn increment_version(&mut self) {
        let parts: Vec<&str> = self.version.split('.').collect();
        if parts.len() == 2 {
            if let Ok(minor) = parts[1].parse::<u32>() {
                self.version = format!("{}.{}", parts[0], minor + 1);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAssessmentV2 {
    pub id: String,
    pub framework_id: String,
    pub repo_id: Option<String>,
    pub status: AssessmentStatus,
    pub findings: Vec<ComplianceFinding>,
    pub score: u32,
    pub assessor_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub history: Vec<AssessmentSnapshot>,
}

impl ComplianceAssessmentV2 {
    pub fn new(framework_id: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            framework_id,
            repo_id: None,
            status: AssessmentStatus::Pending,
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

    pub fn add_finding(&mut self, finding: ComplianceFinding) {
        self.findings.push(finding);
        self.snapshot_history();
    }

    pub fn complete(&mut self, score: u32) {
        self.status = if self.findings.iter().any(|f| f.status == FindingStatus::NonCompliant) {
            AssessmentStatus::Failed
        } else if self.findings.iter().any(|f| f.status == FindingStatus::Partial) {
            AssessmentStatus::Partial
        } else {
            AssessmentStatus::Passed
        };
        self.score = score;
        self.completed_at = Some(Utc::now());
        self.snapshot_history();
    }

    fn snapshot_history(&mut self) {
        self.history.push(AssessmentSnapshot {
            status: self.status,
            score: self.score,
            findings_count: self.findings.len() as u32,
            timestamp: Utc::now(),
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentSnapshot {
    pub status: AssessmentStatus,
    pub score: u32,
    pub findings_count: u32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingTracker {
    pub findings: Vec<ComplianceFinding>,
    pub remediation_deadlines: std::collections::HashMap<String, DateTime<Utc>>,
    pub assigned_to: std::collections::HashMap<String, String>,
}

impl FindingTracker {
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
            remediation_deadlines: std::collections::HashMap::new(),
            assigned_to: std::collections::HashMap::new(),
        }
    }

    pub fn add_finding(&mut self, finding: ComplianceFinding) {
        self.findings.push(finding);
    }

    pub fn set_deadline(&mut self, finding_id: &str, deadline: DateTime<Utc>) {
        self.remediation_deadlines.insert(finding_id.into(), deadline);
    }

    pub fn assign(&mut self, finding_id: &str, user_id: &str) {
        self.assigned_to.insert(finding_id.into(), user_id.into());
    }

    pub fn overdue_findings(&self) -> Vec<&ComplianceFinding> {
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

    pub fn open_findings(&self) -> Vec<&ComplianceFinding> {
        self.findings
            .iter()
            .filter(|f| f.status == FindingStatus::NonCompliant || f.status == FindingStatus::Partial)
            .collect()
    }
}

impl Default for FindingTracker {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ComplianceScoringEngineV2;

impl ComplianceScoringEngineV2 {
    pub fn calculate_framework_score(
        requirements: &[ComplianceRequirementRecord],
        check_results: &[ComplianceCheckResult],
    ) -> ComplianceScore {
        let total = requirements.len() as u32;
        if total == 0 {
            return ComplianceScore {
                overall: 100,
                by_severity: std::collections::HashMap::new(),
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
                CheckStatus::Passed => passed += 1,
                CheckStatus::Failed => failed += 1,
                CheckStatus::Skipped => skipped += 1,
                _ => {}
            }
        }

        let applicable = total - skipped;
        let overall = if applicable > 0 {
            ((passed as f64 / applicable as f64) * 100.0) as u32
        } else {
            100
        };

        ComplianceScore {
            overall,
            by_severity: std::collections::HashMap::new(),
            total_requirements: total,
            passed,
            failed,
            skipped,
        }
    }

    pub fn calculate_weighted_score(
        requirements: &[ComplianceRequirementRecord],
        check_results: &[ComplianceCheckResult],
    ) -> f64 {
        let total_weight: f64 = requirements.iter().map(|r| {
            match r.severity {
                RequirementSeverity::Critical => 4.0,
                RequirementSeverity::High => 3.0,
                RequirementSeverity::Medium => 2.0,
                RequirementSeverity::Low => 1.0,
            }
        }).sum();

        if total_weight == 0.0 {
            return 100.0;
        }

        let earned_weight: f64 = requirements.iter().zip(check_results.iter()).map(|(req, res)| {
            let weight = match req.severity {
                RequirementSeverity::Critical => 4.0,
                RequirementSeverity::High => 3.0,
                RequirementSeverity::Medium => 2.0,
                RequirementSeverity::Low => 1.0,
            };
            if res.status == CheckStatus::Passed {
                weight
            } else {
                0.0
            }
        }).sum();

        (earned_weight / total_weight) * 100.0
    }
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

    #[test]
    fn test_compliance_requirement_record_new() {
        let req = ComplianceRequirementRecord::new(
            "fw-1".into(),
            "R1".into(),
            "Test requirement".into(),
        );
        assert_eq!(req.framework_id, "fw-1");
        assert_eq!(req.requirement_id, "R1");
        assert_eq!(req.severity, RequirementSeverity::Medium);
        assert!(!req.automated_check);
    }

    #[test]
    fn test_compliance_requirement_record_with_severity() {
        let req = ComplianceRequirementRecord::new(
            "fw-1".into(),
            "R1".into(),
            "Desc".into(),
        )
        .with_severity(RequirementSeverity::Critical);
        assert_eq!(req.severity, RequirementSeverity::Critical);
    }

    #[test]
    fn test_compliance_requirement_record_with_automated_check() {
        let mut config = std::collections::HashMap::new();
        config.insert("pattern".into(), serde_json::Value::String("test".into()));
        let req = ComplianceRequirementRecord::new(
            "fw-1".into(),
            "R1".into(),
            "Desc".into(),
        )
        .with_automated_check(config);
        assert!(req.automated_check);
        assert!(req.check_config.contains_key("pattern"));
    }

    #[test]
    fn test_compliance_evidence_new() {
        let evidence = ComplianceEvidence::new(
            "req-1".into(),
            "assess-1".into(),
            EvidenceType::Automated,
        );
        assert_eq!(evidence.requirement_id, "req-1");
        assert_eq!(evidence.evidence_type, EvidenceType::Automated);
        assert!(evidence.collected_by.is_none());
    }

    #[test]
    fn test_compliance_evidence_with_collected_by() {
        let evidence = ComplianceEvidence::new(
            "req-1".into(),
            "assess-1".into(),
            EvidenceType::Manual,
        )
        .with_collected_by("user-1");
        assert_eq!(evidence.collected_by.as_deref(), Some("user-1"));
    }

    #[test]
    fn test_compliance_check_result_new() {
        let result = ComplianceCheckResult::new(
            "req-1".into(),
            "assess-1".into(),
            CheckStatus::Passed,
        );
        assert_eq!(result.status, CheckStatus::Passed);
        assert_eq!(result.score, 0);
    }

    #[test]
    fn test_compliance_check_result_with_score() {
        let result = ComplianceCheckResult::new(
            "req-1".into(),
            "assess-1".into(),
            CheckStatus::Passed,
        )
        .with_score(85);
        assert_eq!(result.score, 85);
    }

    #[test]
    fn test_automated_check_executor_pass() {
        let mut config = std::collections::HashMap::new();
        config.insert("keyword".into(), serde_json::Value::String("compliant".into()));
        let req = ComplianceRequirementRecord::new(
            "fw-1".into(),
            "R1".into(),
            "Desc".into(),
        )
        .with_automated_check(config);
        let result = AutomatedCheckExecutor::execute_check(&req, "this is compliant data");
        assert_eq!(result.status, CheckStatus::Passed);
        assert_eq!(result.score, 100);
    }

    #[test]
    fn test_automated_check_executor_fail() {
        let mut config = std::collections::HashMap::new();
        config.insert("keyword".into(), serde_json::Value::String("missing".into()));
        let req = ComplianceRequirementRecord::new(
            "fw-1".into(),
            "R1".into(),
            "Desc".into(),
        )
        .with_automated_check(config);
        let result = AutomatedCheckExecutor::execute_check(&req, "this does not contain it");
        assert_eq!(result.status, CheckStatus::Failed);
        assert_eq!(result.score, 0);
    }

    #[test]
    fn test_automated_check_executor_skip() {
        let req = ComplianceRequirementRecord::new(
            "fw-1".into(),
            "R1".into(),
            "Desc".into(),
        );
        let result = AutomatedCheckExecutor::execute_check(&req, "data");
        assert_eq!(result.status, CheckStatus::Skipped);
    }

    #[test]
    fn test_compliance_scoring_engine() {
        let requirements = vec![
            ComplianceRequirementRecord::new("fw".into(), "R1".into(), "".into()),
            ComplianceRequirementRecord::new("fw".into(), "R2".into(), "".into()),
        ];
        let check_results = vec![
            ComplianceCheckResult::new("R1".into(), "a".into(), CheckStatus::Passed),
            ComplianceCheckResult::new("R2".into(), "a".into(), CheckStatus::Failed),
        ];
        let score = ComplianceScoringEngine::calculate_framework_score(&requirements, &check_results);
        assert_eq!(score.total_requirements, 2);
        assert_eq!(score.passed, 1);
        assert_eq!(score.failed, 1);
        assert_eq!(score.overall, 50);
    }

    #[test]
    fn test_compliance_scoring_engine_all_passed() {
        let requirements = vec![
            ComplianceRequirementRecord::new("fw".into(), "R1".into(), "".into()),
        ];
        let check_results = vec![
            ComplianceCheckResult::new("R1".into(), "a".into(), CheckStatus::Passed),
        ];
        let score = ComplianceScoringEngine::calculate_framework_score(&requirements, &check_results);
        assert_eq!(score.overall, 100);
    }

    #[test]
    fn test_requirement_severity_serialization() {
        assert_eq!(
            serde_json::to_string(&RequirementSeverity::Critical).unwrap(),
            "\"critical\""
        );
    }

    #[test]
    fn test_evidence_type_serialization() {
        assert_eq!(
            serde_json::to_string(&EvidenceType::Automated).unwrap(),
            "\"automated\""
        );
    }

    #[test]
    fn test_check_status_serialization() {
        assert_eq!(
            serde_json::to_string(&CheckStatus::Passed).unwrap(),
            "\"passed\""
        );
    }

    #[test]
    fn test_compliance_framework_v2_new() {
        let fw = ComplianceFrameworkV2::new("Test V2".into(), "Description".into());
        assert_eq!(fw.name, "Test V2");
        assert_eq!(fw.version, "1.0");
        assert!(fw.enabled);
        assert!(fw.requirements.is_empty());
    }

    #[test]
    fn test_compliance_framework_v2_with_version() {
        let fw = ComplianceFrameworkV2::new("Test".into(), "Desc".into())
            .with_version("2.1");
        assert_eq!(fw.version, "2.1");
    }

    #[test]
    fn test_compliance_framework_v2_increment_version() {
        let mut fw = ComplianceFrameworkV2::new("Test".into(), "Desc".into());
        fw.increment_version();
        assert_eq!(fw.version, "1.1");
        fw.increment_version();
        assert_eq!(fw.version, "1.2");
    }

    #[test]
    fn test_compliance_assessment_v2_new() {
        let assessment = ComplianceAssessmentV2::new("fw-1".into());
        assert_eq!(assessment.framework_id, "fw-1");
        assert_eq!(assessment.status, AssessmentStatus::Pending);
        assert!(assessment.findings.is_empty());
        assert_eq!(assessment.score, 0);
        assert!(assessment.history.is_empty());
    }

    #[test]
    fn test_compliance_assessment_v2_with_repo() {
        let assessment = ComplianceAssessmentV2::new("fw-1".into())
            .with_repo("repo-1")
            .with_assessor("user-1");
        assert_eq!(assessment.repo_id.as_deref(), Some("repo-1"));
        assert_eq!(assessment.assessor_id.as_deref(), Some("user-1"));
    }

    #[test]
    fn test_compliance_assessment_v2_add_finding() {
        let mut assessment = ComplianceAssessmentV2::new("fw-1".into());
        let finding = ComplianceFinding {
            id: uuid::Uuid::new_v4().to_string(),
            requirement_id: "req-1".into(),
            status: FindingStatus::Compliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        assessment.add_finding(finding);
        assert_eq!(assessment.findings.len(), 1);
        assert_eq!(assessment.history.len(), 1);
    }

    #[test]
    fn test_compliance_assessment_v2_complete() {
        let mut assessment = ComplianceAssessmentV2::new("fw-1".into());
        let finding = ComplianceFinding {
            id: uuid::Uuid::new_v4().to_string(),
            requirement_id: "req-1".into(),
            status: FindingStatus::Compliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        assessment.add_finding(finding);
        assessment.complete(100);
        assert_eq!(assessment.status, AssessmentStatus::Passed);
        assert_eq!(assessment.score, 100);
        assert!(assessment.completed_at.is_some());
    }

    #[test]
    fn test_compliance_assessment_v2_complete_failed() {
        let mut assessment = ComplianceAssessmentV2::new("fw-1".into());
        let finding = ComplianceFinding {
            id: uuid::Uuid::new_v4().to_string(),
            requirement_id: "req-1".into(),
            status: FindingStatus::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        assessment.add_finding(finding);
        assessment.complete(0);
        assert_eq!(assessment.status, AssessmentStatus::Failed);
    }

    #[test]
    fn test_finding_tracker_new() {
        let tracker = FindingTracker::new();
        assert!(tracker.findings.is_empty());
        assert!(tracker.remediation_deadlines.is_empty());
        assert!(tracker.assigned_to.is_empty());
    }

    #[test]
    fn test_finding_tracker_add_finding() {
        let mut tracker = FindingTracker::new();
        let finding = ComplianceFinding {
            id: "f-1".into(),
            requirement_id: "req-1".into(),
            status: FindingStatus::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        tracker.add_finding(finding);
        assert_eq!(tracker.findings.len(), 1);
    }

    #[test]
    fn test_finding_tracker_overdue() {
        let mut tracker = FindingTracker::new();
        let finding = ComplianceFinding {
            id: "f-1".into(),
            requirement_id: "req-1".into(),
            status: FindingStatus::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        tracker.add_finding(finding);
        tracker.set_deadline("f-1", Utc::now() - chrono::Duration::hours(1));
        assert_eq!(tracker.overdue_findings().len(), 1);
    }

    #[test]
    fn test_finding_tracker_open_findings() {
        let mut tracker = FindingTracker::new();
        let f1 = ComplianceFinding {
            id: "f-1".into(),
            requirement_id: "req-1".into(),
            status: FindingStatus::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        let f2 = ComplianceFinding {
            id: "f-2".into(),
            requirement_id: "req-2".into(),
            status: FindingStatus::Compliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        tracker.add_finding(f1);
        tracker.add_finding(f2);
        assert_eq!(tracker.open_findings().len(), 1);
    }

    #[test]
    fn test_finding_tracker_assign() {
        let mut tracker = FindingTracker::new();
        let finding = ComplianceFinding {
            id: "f-1".into(),
            requirement_id: "req-1".into(),
            status: FindingStatus::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        };
        tracker.add_finding(finding);
        tracker.assign("f-1", "user-1");
        assert_eq!(tracker.assigned_to.get("f-1").map(|s| s.as_str()), Some("user-1"));
    }

    #[test]
    fn test_compliance_scoring_engine_v2_weighted() {
        let requirements = vec![
            ComplianceRequirementRecord::new("fw".into(), "R1".into(), "".into())
                .with_severity(RequirementSeverity::Critical),
            ComplianceRequirementRecord::new("fw".into(), "R2".into(), "".into())
                .with_severity(RequirementSeverity::Low),
        ];
        let check_results = vec![
            ComplianceCheckResult::new("R1".into(), "a".into(), CheckStatus::Passed),
            ComplianceCheckResult::new("R2".into(), "a".into(), CheckStatus::Failed),
        ];
        let score = ComplianceScoringEngineV2::calculate_weighted_score(&requirements, &check_results);
        assert!((score - 80.0).abs() < 0.01);
    }
}

// ============================================================================
// V22 Types
// ============================================================================

pub type RequirementSeverityV22 = RequirementSeverity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirementRecordV22 {
    pub id: String,
    pub framework_id: String,
    pub requirement_id: String,
    pub description: String,
    pub severity: RequirementSeverityV22,
    pub automated_check: bool,
    pub check_config: HashMap<String, serde_json::Value>,
    pub evidence_config: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ComplianceRequirementRecordV22 {
    pub fn new(framework_id: String, requirement_id: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            framework_id,
            requirement_id,
            description,
            severity: RequirementSeverityV22::Medium,
            automated_check: false,
            check_config: HashMap::new(),
            evidence_config: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_severity(mut self, severity: RequirementSeverityV22) -> Self {
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
            RequirementSeverityV22::Critical => 4.0,
            RequirementSeverityV22::High => 3.0,
            RequirementSeverityV22::Medium => 2.0,
            RequirementSeverityV22::Low => 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceEvidenceV22 {
    pub id: String,
    pub requirement_id: String,
    pub assessment_id: String,
    pub evidence_type: EvidenceTypeV22,
    pub content: HashMap<String, serde_json::Value>,
    pub collected_by: Option<String>,
    pub collected_at: DateTime<Utc>,
}

impl ComplianceEvidenceV22 {
    pub fn new(
        requirement_id: String,
        assessment_id: String,
        evidence_type: EvidenceTypeV22,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheckResultV22 {
    pub id: String,
    pub requirement_id: String,
    pub assessment_id: String,
    pub status: CheckStatusV22,
    pub result_details: HashMap<String, serde_json::Value>,
    pub score: u32,
    pub executed_at: DateTime<Utc>,
}

impl ComplianceCheckResultV22 {
    pub fn new(requirement_id: String, assessment_id: String, status: CheckStatusV22) -> Self {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkV22 {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub requirements: Vec<ComplianceRequirementV22>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ComplianceFrameworkV22 {
    pub fn new(name: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            version: "20.0".into(),
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

    pub fn add_requirement(&mut self, requirement: ComplianceRequirementV22) {
        self.requirements.push(requirement);
    }

    pub fn mandatory_requirements(&self) -> Vec<&ComplianceRequirementV22> {
        self.requirements.iter().filter(|r| r.mandatory).collect()
    }

    pub fn requirements_by_category(&self, category: &str) -> Vec<&ComplianceRequirementV22> {
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
pub struct ComplianceRequirementV22 {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub mandatory: bool,
    pub verification_method: VerificationMethodV22,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationMethodV22 {
    Automated,
    Manual,
    Hybrid,
    Inherited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentStatusV22 {
    Pending,
    Running,
    Passed,
    Failed,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAssessmentV22 {
    pub id: String,
    pub framework_id: String,
    pub repo_id: Option<String>,
    pub status: AssessmentStatusV22,
    pub findings: Vec<ComplianceFindingV22>,
    pub score: u32,
    pub assessor_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub history: Vec<AssessmentSnapshotV22>,
}

impl ComplianceAssessmentV22 {
    pub fn new(framework_id: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            framework_id,
            repo_id: None,
            status: AssessmentStatusV22::Pending,
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

    pub fn add_finding(&mut self, finding: ComplianceFindingV22) {
        self.findings.push(finding);
        self.snapshot_history();
    }

    pub fn complete(&mut self, score: u32) {
        self.status = if self
            .findings
            .iter()
            .any(|f| f.status == FindingStatusV22::NonCompliant)
        {
            AssessmentStatusV22::Failed
        } else if self
            .findings
            .iter()
            .any(|f| f.status == FindingStatusV22::Partial)
        {
            AssessmentStatusV22::Partial
        } else {
            AssessmentStatusV22::Passed
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
            .filter(|f| {
                f.status == FindingStatusV22::Compliant
                    || f.status == FindingStatusV22::NotApplicable
            })
            .count();
        (compliant as f64 / self.findings.len() as f64) * 100.0
    }

    pub fn non_compliant_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.status == FindingStatusV22::NonCompliant)
            .count()
    }

    fn snapshot_history(&mut self) {
        self.history.push(AssessmentSnapshotV22 {
            status: self.status,
            score: self.score,
            findings_count: self.findings.len() as u32,
            timestamp: Utc::now(),
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentSnapshotV22 {
    pub status: AssessmentStatusV22,
    pub score: u32,
    pub findings_count: u32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFindingV22 {
    pub id: String,
    pub requirement_id: String,
    pub status: FindingStatusV22,
    pub details: String,
    pub evidence: Option<String>,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatusV22 {
    Compliant,
    NonCompliant,
    Partial,
    NotApplicable,
}

impl ComplianceFindingV22 {
    pub fn is_compliant(&self) -> bool {
        self.status == FindingStatusV22::Compliant
            || self.status == FindingStatusV22::NotApplicable
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingTrackerV22 {
    pub findings: Vec<ComplianceFindingV22>,
    pub remediation_deadlines: HashMap<String, DateTime<Utc>>,
    pub assigned_to: HashMap<String, String>,
    pub severity_scores: HashMap<String, u32>,
    pub history: Vec<FindingHistoryEntryV22>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingHistoryEntryV22 {
    pub finding_id: String,
    pub old_status: FindingStatusV22,
    pub new_status: FindingStatusV22,
    pub changed_by: Option<String>,
    pub changed_at: DateTime<Utc>,
}

impl FindingTrackerV22 {
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
            remediation_deadlines: HashMap::new(),
            assigned_to: HashMap::new(),
            severity_scores: HashMap::new(),
            history: Vec::new(),
        }
    }

    pub fn add_finding(&mut self, finding: ComplianceFindingV22) {
        self.findings.push(finding);
    }

    pub fn set_deadline(&mut self, finding_id: &str, deadline: DateTime<Utc>) {
        self.remediation_deadlines
            .insert(finding_id.into(), deadline);
    }

    pub fn assign(&mut self, finding_id: &str, user_id: &str) {
        self.assigned_to.insert(finding_id.into(), user_id.into());
    }

    pub fn update_finding_status(
        &mut self,
        finding_id: &str,
        new_status: FindingStatusV22,
        changed_by: Option<String>,
    ) -> Result<(), String> {
        let finding = self
            .findings
            .iter_mut()
            .find(|f| f.id == finding_id)
            .ok_or("Finding not found")?;
        let old_status = finding.status;
        finding.status = new_status;
        self.history.push(FindingHistoryEntryV22 {
            finding_id: finding_id.into(),
            old_status,
            new_status,
            changed_by,
            changed_at: Utc::now(),
        });
        Ok(())
    }

    pub fn overdue_findings(&self) -> Vec<&ComplianceFindingV22> {
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

    pub fn open_findings(&self) -> Vec<&ComplianceFindingV22> {
        self.findings
            .iter()
            .filter(|f| {
                f.status == FindingStatusV22::NonCompliant || f.status == FindingStatusV22::Partial
            })
            .collect()
    }

    pub fn findings_by_severity(&self, status: FindingStatusV22) -> Vec<&ComplianceFindingV22> {
        self.findings.iter().filter(|f| f.status == status).collect()
    }

    pub fn compute_severity_scores(&mut self) -> HashMap<String, u32> {
        let mut scores: HashMap<String, u32> = HashMap::new();
        for finding in &self.findings {
            let category = finding
                .requirement_id
                .split('.')
                .next()
                .unwrap_or("unknown")
                .to_string();
            *scores.entry(category).or_insert(0) += match finding.status {
                FindingStatusV22::NonCompliant => 10,
                FindingStatusV22::Partial => 5,
                FindingStatusV22::Compliant => 0,
                FindingStatusV22::NotApplicable => 0,
            };
        }
        self.severity_scores = scores.clone();
        scores
    }

    pub fn total_findings(&self) -> usize {
        self.findings.len()
    }

    pub fn finding_history(&self, finding_id: &str) -> Vec<&FindingHistoryEntryV22> {
        self.history
            .iter()
            .filter(|h| h.finding_id == finding_id)
            .collect()
    }
}

impl Default for FindingTrackerV22 {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceScoreV22 {
    pub overall: u32,
    pub by_severity: HashMap<String, (u32, u32)>,
    pub total_requirements: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub inherited: u32,
}

pub struct ComplianceScoringEngineV22;

impl ComplianceScoringEngineV22 {
    pub fn calculate_framework_score(
        requirements: &[ComplianceRequirementRecordV22],
        check_results: &[ComplianceCheckResultV22],
    ) -> ComplianceScoreV22 {
        let total = requirements.len() as u32;
        if total == 0 {
            return ComplianceScoreV22 {
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
                CheckStatusV22::Passed => passed += 1,
                CheckStatusV22::Failed => failed += 1,
                CheckStatusV22::Skipped => skipped += 1,
                CheckStatusV22::Inherited => inherited += 1,
                _ => {}
            }
        }

        let applicable = total - skipped - inherited;
        let overall = if applicable > 0 {
            ((passed as f64 / applicable as f64) * 100.0) as u32
        } else {
            100
        };

        ComplianceScoreV22 {
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
        requirements: &[ComplianceRequirementRecordV22],
        check_results: &[ComplianceCheckResultV22],
    ) -> f64 {
        let total_weight: f64 = requirements.iter().map(|r| r.risk_weight()).sum();

        if total_weight == 0.0 {
            return 100.0;
        }

        let earned_weight: f64 = requirements
            .iter()
            .zip(check_results.iter())
            .map(|(req, res)| {
                let weight = req.risk_weight();
                if res.status == CheckStatusV22::Passed
                    || res.status == CheckStatusV22::Inherited
                {
                    weight
                } else {
                    0.0
                }
            })
            .sum();

        (earned_weight / total_weight) * 100.0
    }
}

pub struct ComplianceAssessorV22;

impl ComplianceAssessorV22 {
    pub fn assess(
        framework: &ComplianceFrameworkV22,
        findings: &[ComplianceFindingV22],
    ) -> AssessmentResultV22 {
        let total_requirements = framework.requirements.len() as u32;
        if total_requirements == 0 {
            return AssessmentResultV22 {
                score: 100,
                status: AssessmentStatusV22::Passed,
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
                FindingStatusV22::Compliant => compliant += 1,
                FindingStatusV22::NonCompliant => non_compliant += 1,
                FindingStatusV22::Partial => partial += 1,
                FindingStatusV22::NotApplicable => not_applicable += 1,
            }
        }

        let applicable = total_requirements - not_applicable;
        let score = if applicable > 0 {
            ((compliant as f64 / applicable as f64) * 100.0) as u32
        } else {
            100
        };

        let status = if non_compliant > 0 {
            AssessmentStatusV22::Failed
        } else if partial > 0 {
            AssessmentStatusV22::Partial
        } else {
            AssessmentStatusV22::Passed
        };

        AssessmentResultV22 {
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
pub struct AssessmentResultV22 {
    pub score: u32,
    pub status: AssessmentStatusV22,
    pub compliant_count: u32,
    pub non_compliant_count: u32,
    pub partial_count: u32,
    pub not_applicable_count: u32,
}

pub fn create_soc2_framework_v22() -> ComplianceFrameworkV22 {
    let mut framework =
        ComplianceFrameworkV22::new("SOC 2".into(), "Service Organization Control 2".into());
    framework.add_requirement(ComplianceRequirementV22 {
        id: "CC6.1".into(),
        name: "Logical Access Controls".into(),
        description: "Implement logical access security controls".into(),
        category: "Security".into(),
        mandatory: true,
        verification_method: VerificationMethodV22::Automated,
    });
    framework.add_requirement(ComplianceRequirementV22 {
        id: "CC6.6".into(),
        name: "System Boundaries".into(),
        description: "Restrict logical access to system boundaries".into(),
        category: "Security".into(),
        mandatory: true,
        verification_method: VerificationMethodV22::Hybrid,
    });
    framework.add_requirement(ComplianceRequirementV22 {
        id: "CC7.1".into(),
        name: "Vulnerability Management".into(),
        description: "Detect and monitor for vulnerabilities".into(),
        category: "Security".into(),
        mandatory: true,
        verification_method: VerificationMethodV22::Automated,
    });
    framework.add_requirement(ComplianceRequirementV22 {
        id: "CC8.1".into(),
        name: "Change Management".into(),
        description: "Authorize and manage changes to systems".into(),
        category: "Availability".into(),
        mandatory: true,
        verification_method: VerificationMethodV22::Manual,
    });
    framework
}

pub fn create_iso27001_framework_v22() -> ComplianceFrameworkV22 {
    let mut framework = ComplianceFrameworkV22::new(
        "ISO 27001".into(),
        "Information Security Management System".into(),
    );
    framework.add_requirement(ComplianceRequirementV22 {
        id: "A.12.6.1".into(),
        name: "Technical Vulnerability Management".into(),
        description: "Obtain information about technical vulnerabilities".into(),
        category: "Operations Security".into(),
        mandatory: true,
        verification_method: VerificationMethodV22::Automated,
    });
    framework.add_requirement(ComplianceRequirementV22 {
        id: "A.14.2.1".into(),
        name: "Secure Development Policy".into(),
        description: "Establish secure development lifecycle".into(),
        category: "Acquisition Development".into(),
        mandatory: true,
        verification_method: VerificationMethodV22::Hybrid,
    });
    framework.add_requirement(ComplianceRequirementV22 {
        id: "A.18.2.1".into(),
        name: "Independent Review".into(),
        description: "Independent review of organization's ISMS".into(),
        category: "Compliance".into(),
        mandatory: true,
        verification_method: VerificationMethodV22::Manual,
    });
    framework
}

// ============================================================================
// V23 Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

// ============================================================================
// V24 Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomatedCheckV24 {
    pub id: String,
    pub requirement_id: String,
    pub check_type: ComplianceCheckTypeV23,
    pub check_config: HashMap<String, serde_json::Value>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub last_result: CheckResultStatusV24,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

impl AutomatedCheckV24 {
    pub fn new(requirement_id: String, check_type: ComplianceCheckTypeV23) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            requirement_id,
            check_type,
            check_config: HashMap::new(),
            last_run_at: None,
            last_result: CheckResultStatusV24::Pending,
            enabled: true,
            created_at: Utc::now(),
        }
    }

    pub fn with_config(mut self, config: HashMap<String, serde_json::Value>) -> Self {
        self.check_config = config;
        self
    }

    pub fn record_result(&mut self, result: CheckResultStatusV24) {
        self.last_result = result;
        self.last_run_at = Some(Utc::now());
    }

    pub fn is_due(&self, interval_minutes: u32) -> bool {
        match self.last_run_at {
            Some(last) => {
                let elapsed = Utc::now() - last;
                elapsed >= chrono::Duration::minutes(interval_minutes as i64)
            }
            None => true,
        }
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckResultStatusV24 {
    Pending,
    Passed,
    Failed,
    Warning,
    Error,
    Skipped,
}

impl CheckResultStatusV24 {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Skipped => "skipped",
        }
    }

    pub fn is_passing(&self) -> bool {
        matches!(self, Self::Passed | Self::Skipped)
    }

    pub fn is_failing(&self) -> bool {
        matches!(self, Self::Failed | Self::Error)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResultV24 {
    pub id: String,
    pub check_id: String,
    pub result: CheckResultStatusV24,
    pub details: HashMap<String, serde_json::Value>,
    pub run_at: DateTime<Utc>,
}

impl CheckResultV24 {
    pub fn new(check_id: String, result: CheckResultStatusV24) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            check_id,
            result,
            details: HashMap::new(),
            run_at: Utc::now(),
        }
    }

    pub fn with_details(mut self, details: HashMap<String, serde_json::Value>) -> Self {
        self.details = details;
        self
    }

    pub fn is_passing(&self) -> bool {
        self.result.is_passing()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceScoreV24 {
    pub framework_id: String,
    pub total_checks: u32,
    pub passed: u32,
    pub failed: u32,
    pub warnings: u32,
    pub pending: u32,
    pub score_percentage: f64,
    pub last_calculated: DateTime<Utc>,
}

impl ComplianceScoreV24 {
    pub fn new(framework_id: String) -> Self {
        Self {
            framework_id,
            total_checks: 0,
            passed: 0,
            failed: 0,
            warnings: 0,
            pending: 0,
            score_percentage: 0.0,
            last_calculated: Utc::now(),
        }
    }

    pub fn calculate(&mut self, results: &[CheckResultV24]) {
        self.total_checks = results.len() as u32;
        self.passed = results.iter().filter(|r| r.result == CheckResultStatusV24::Passed).count() as u32;
        self.failed = results.iter().filter(|r| r.result == CheckResultStatusV24::Failed).count() as u32;
        self.warnings = results.iter().filter(|r| r.result == CheckResultStatusV24::Warning).count() as u32;
        self.pending = results.iter().filter(|r| r.result == CheckResultStatusV24::Pending).count() as u32;

        let applicable = self.total_checks - self.pending;
        self.score_percentage = if applicable == 0 {
            100.0
        } else {
            (self.passed as f64 / applicable as f64) * 100.0
        };
        self.last_calculated = Utc::now();
    }

    pub fn grade(&self) -> &'static str {
        if self.score_percentage >= 95.0 {
            "A+"
        } else if self.score_percentage >= 90.0 {
            "A"
        } else if self.score_percentage >= 80.0 {
            "B"
        } else if self.score_percentage >= 70.0 {
            "C"
        } else if self.score_percentage >= 60.0 {
            "D"
        } else {
            "F"
        }
    }

    pub fn is_compliant(&self) -> bool {
        self.score_percentage >= 80.0 && self.failed == 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceGapItemV24 {
    pub requirement_id: String,
    pub description: String,
    pub severity: RequirementSeverityV22,
    pub gap_type: GapTypeV24,
    pub recommendation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GapTypeV24 {
    MissingCheck,
    CheckFailing,
    NoEvidence,
    InsufficientEvidence,
    ExpiredEvidence,
}

impl GapTypeV24 {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::MissingCheck => "missing_check",
            Self::CheckFailing => "check_failing",
            Self::NoEvidence => "no_evidence",
            Self::InsufficientEvidence => "insufficient_evidence",
            Self::ExpiredEvidence => "expired_evidence",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceGapAnalysisV24 {
    pub framework_id: String,
    pub gaps: Vec<ComplianceGapItemV24>,
    pub total_gaps: u32,
    pub critical_gaps: u32,
    pub high_gaps: u32,
    pub medium_gaps: u32,
    pub low_gaps: u32,
    pub generated_at: DateTime<Utc>,
}

impl ComplianceGapAnalysisV24 {
    pub fn new(framework_id: String) -> Self {
        Self {
            framework_id,
            gaps: Vec::new(),
            total_gaps: 0,
            critical_gaps: 0,
            high_gaps: 0,
            medium_gaps: 0,
            low_gaps: 0,
            generated_at: Utc::now(),
        }
    }

    pub fn add_gap(&mut self, gap: ComplianceGapItemV24) {
        match gap.severity {
            RequirementSeverityV22::Critical => self.critical_gaps += 1,
            RequirementSeverityV22::High => self.high_gaps += 1,
            RequirementSeverityV22::Medium => self.medium_gaps += 1,
            RequirementSeverityV22::Low => self.low_gaps += 1,
        }
        self.gaps.push(gap);
        self.total_gaps = self.gaps.len() as u32;
    }

    pub fn has_critical_gaps(&self) -> bool {
        self.critical_gaps > 0
    }

    pub fn gaps_by_severity(&self, severity: RequirementSeverityV22) -> Vec<&ComplianceGapItemV24> {
        self.gaps.iter().filter(|g| g.severity == severity).collect()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AutomatedCheckRunnerV24 {
    checks: Vec<AutomatedCheckV24>,
    by_requirement: HashMap<String, Vec<usize>>,
}

impl AutomatedCheckRunnerV24 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_check(&mut self, check: AutomatedCheckV24) {
        let idx = self.checks.len();
        self.by_requirement
            .entry(check.requirement_id.clone())
            .or_default()
            .push(idx);
        self.checks.push(check);
    }

    pub fn get_checks_for_requirement(&self, req_id: &str) -> Vec<&AutomatedCheckV24> {
        self.by_requirement
            .get(req_id)
            .map(|indices| indices.iter().map(|&idx| &self.checks[idx]).collect())
            .unwrap_or_default()
    }

    pub fn get_enabled_checks(&self) -> Vec<&AutomatedCheckV24> {
        self.checks.iter().filter(|c| c.enabled).collect()
    }

    pub fn get_due_checks(&self, interval_minutes: u32) -> Vec<&AutomatedCheckV24> {
        self.checks
            .iter()
            .filter(|c| c.enabled && c.is_due(interval_minutes))
            .collect()
    }

    pub fn total_checks(&self) -> usize {
        self.checks.len()
    }

    pub fn record_result(&mut self, check_id: &str, result: CheckResultStatusV24) {
        if let Some(check) = self.checks.iter_mut().find(|c| c.id == check_id) {
            check.record_result(result);
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CheckResultHistoryV24 {
    results: Vec<CheckResultV24>,
    by_check: HashMap<String, Vec<usize>>,
}

impl CheckResultHistoryV24 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, result: CheckResultV24) {
        let idx = self.results.len();
        self.by_check
            .entry(result.check_id.clone())
            .or_default()
            .push(idx);
        self.results.push(result);
    }

    pub fn get_results_for_check(&self, check_id: &str) -> Vec<&CheckResultV24> {
        self.by_check
            .get(check_id)
            .map(|indices| indices.iter().map(|&idx| &self.results[idx]).collect())
            .unwrap_or_default()
    }

    pub fn latest_result_for_check(&self, check_id: &str) -> Option<&CheckResultV24> {
        self.by_check.get(check_id).and_then(|indices| {
            indices
                .iter()
                .map(|&idx| &self.results[idx])
                .max_by_key(|r| r.run_at)
        })
    }

    pub fn total_results(&self) -> usize {
        self.results.len()
    }

    pub fn pass_rate(&self) -> f64 {
        if self.results.is_empty() {
            return 100.0;
        }
        let passing = self.results.iter().filter(|r| r.is_passing()).count();
        (passing as f64 / self.results.len() as f64) * 100.0
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplianceScoringEngineV24 {
    scores: HashMap<String, ComplianceScoreV24>,
}

impl ComplianceScoringEngineV24 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn calculate_score(
        &mut self,
        framework_id: &str,
        results: &[CheckResultV24],
    ) -> ComplianceScoreV24 {
        let mut score = ComplianceScoreV24::new(framework_id.into());
        score.calculate(results);
        self.scores.insert(framework_id.into(), score.clone());
        score
    }

    pub fn get_score(&self, framework_id: &str) -> Option<&ComplianceScoreV24> {
        self.scores.get(framework_id)
    }

    pub fn all_scores(&self) -> &HashMap<String, ComplianceScoreV24> {
        &self.scores
    }

    pub fn compliant_frameworks(&self) -> Vec<&ComplianceScoreV24> {
        self.scores.values().filter(|s| s.is_compliant()).collect()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GapAnalysisEngineV24 {
    analyses: Vec<ComplianceGapAnalysisV24>,
}

impl GapAnalysisEngineV24 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run_analysis(
        &mut self,
        framework_id: &str,
        checks: &[AutomatedCheckV24],
        evidence: &[ComplianceEvidenceItemV23],
    ) -> ComplianceGapAnalysisV24 {
        let mut analysis = ComplianceGapAnalysisV24::new(framework_id.into());

        for check in checks {
            if !check.enabled {
                continue;
            }
            match check.last_result {
                CheckResultStatusV24::Failed | CheckResultStatusV24::Error => {
                    analysis.add_gap(ComplianceGapItemV24 {
                        requirement_id: check.requirement_id.clone(),
                        description: format!("Check '{}' is failing", check.id),
                        severity: RequirementSeverityV22::High,
                        gap_type: GapTypeV24::CheckFailing,
                        recommendation: "Investigate and remediate the failing check".into(),
                    });
                }
                CheckResultStatusV24::Pending => {
                    analysis.add_gap(ComplianceGapItemV24 {
                        requirement_id: check.requirement_id.clone(),
                        description: format!("Check '{}' has not been run", check.id),
                        severity: RequirementSeverityV22::Medium,
                        gap_type: GapTypeV24::MissingCheck,
                        recommendation: "Run the automated check".into(),
                    });
                }
                _ => {}
            }
        }

        let req_ids: Vec<String> = checks.iter().map(|c| c.requirement_id.clone()).collect();
        for req_id in &req_ids {
            let has_evidence = evidence.iter().any(|e| &e.requirement_id == req_id);
            if !has_evidence {
                analysis.add_gap(ComplianceGapItemV24 {
                    requirement_id: req_id.clone(),
                    description: format!("No evidence for requirement {}", req_id),
                    severity: RequirementSeverityV22::High,
                    gap_type: GapTypeV24::NoEvidence,
                    recommendation: "Collect and submit evidence for this requirement".into(),
                });
            }
        }

        self.analyses.push(analysis.clone());
        analysis
    }

    pub fn latest_analysis(&self, framework_id: &str) -> Option<&ComplianceGapAnalysisV24> {
        self.analyses
            .iter()
            .filter(|a| a.framework_id == framework_id)
            .max_by_key(|a| a.generated_at)
    }
}

#[cfg(test)]
mod consolidated_tests {
    use super::*;

    #[test]
    fn smoke_test_v22_types() {
        let req = ComplianceRequirementRecordV22::new("fw".into(), "R1".into(), "desc".into());
        assert_eq!(req.risk_weight(), 2.0);
        let fw = ComplianceFrameworkV22::new("Test".into(), "Desc".into());
        assert_eq!(fw.version, "20.0");
        let mut assessment = ComplianceAssessmentV22::new("fw-1".into());
        assessment.add_finding(ComplianceFindingV22 {
            id: "f1".into(),
            requirement_id: "r1".into(),
            status: FindingStatusV22::Compliant,
            details: "ok".into(),
            evidence: None,
            remediation: None,
        });
        assert_eq!(assessment.compliance_rate(), 100.0);
    }

    #[test]
    fn smoke_test_v23_types() {
        let rule_set = ComplianceRuleSetV23::new("Test".into(), "SOC 2".into());
        assert_eq!(rule_set.name, "Test");
        let evidence = ComplianceEvidenceItemV23::new("req-1".into(), EvidenceTypeV22::Automated);
        assert!(!evidence.is_verified());
        let report = AuditReadinessReportV23::new("fw".into(), "FW".into(), "SOC2".into());
        assert!(!report.is_audit_ready());
    }

    #[test]
    fn smoke_test_v24_types() {
        let check = AutomatedCheckV24::new("req-1".into(), ComplianceCheckTypeV23::Automated);
        assert!(check.enabled);
        assert_eq!(check.last_result, CheckResultStatusV24::Pending);
        let mut score = ComplianceScoreV24::new("fw".into());
        let results = vec![
            CheckResultV24::new("c1".into(), CheckResultStatusV24::Passed),
            CheckResultV24::new("c2".into(), CheckResultStatusV24::Failed),
        ];
        score.calculate(&results);
        assert_eq!(score.total_checks, 2);
        assert!(!score.is_compliant());
        let mut analysis = ComplianceGapAnalysisV24::new("fw".into());
        analysis.add_gap(ComplianceGapItemV24 {
            requirement_id: "r1".into(),
            description: "Missing".into(),
            severity: RequirementSeverityV22::Critical,
            gap_type: GapTypeV24::NoEvidence,
            recommendation: "Add evidence".into(),
        });
        assert!(analysis.has_critical_gaps());
    }

    #[test]
    fn smoke_test_enhanced_base_enums() {
        assert!(CheckStatus::Inherited != CheckStatus::Passed);
        assert!(EvidenceType::Inherited != EvidenceType::Automated);
        assert!(VerificationMethod::Inherited != VerificationMethod::Hybrid);
    }

    #[test]
    fn smoke_test_v22_scoring_with_inherited() {
        let requirements = vec![
            ComplianceRequirementRecordV22::new("fw".into(), "R1".into(), "".into()),
            ComplianceRequirementRecordV22::new("fw".into(), "R2".into(), "".into()),
        ];
        let check_results = vec![
            ComplianceCheckResultV22::new("R1".into(), "a".into(), CheckStatusV22::Passed),
            ComplianceCheckResultV22::new("R2".into(), "a".into(), CheckStatusV22::Inherited),
        ];
        let score =
            ComplianceScoringEngineV22::calculate_framework_score(&requirements, &check_results);
        assert_eq!(score.overall, 100);
        assert_eq!(score.inherited, 1);
    }

    #[test]
    fn smoke_test_v22_finding_tracker_history() {
        let mut tracker = FindingTrackerV22::new();
        tracker.add_finding(ComplianceFindingV22 {
            id: "f-1".into(),
            requirement_id: "req-1".into(),
            status: FindingStatusV22::NonCompliant,
            details: "Test".into(),
            evidence: None,
            remediation: None,
        });
        tracker
            .update_finding_status("f-1", FindingStatusV22::Compliant, Some("user-1".into()))
            .unwrap();
        assert_eq!(tracker.findings[0].status, FindingStatusV22::Compliant);
        assert_eq!(tracker.history.len(), 1);
    }
}
