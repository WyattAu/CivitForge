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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    Manual,
    Automated,
    SystemGenerated,
    External,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Pending,
    Running,
    Passed,
    Failed,
    Skipped,
}

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
