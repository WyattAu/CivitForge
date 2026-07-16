#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::compliance_v23::{
    ComplianceCheckTypeV23, ComplianceEvidenceItemV23, ComplianceRuleV23, EvidenceCollectionV23,
};
use crate::compliance_v22::{CheckStatusV22, RequirementSeverityV22};

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
mod tests {
    use super::*;

    #[test]
    fn test_automated_check_v24_new() {
        let check = AutomatedCheckV24::new("req-1".into(), ComplianceCheckTypeV23::Automated);
        assert!(check.enabled);
        assert_eq!(check.last_result, CheckResultStatusV24::Pending);
    }

    #[test]
    fn test_automated_check_v24_record_result() {
        let mut check = AutomatedCheckV24::new("req-1".into(), ComplianceCheckTypeV23::Automated);
        check.record_result(CheckResultStatusV24::Passed);
        assert_eq!(check.last_result, CheckResultStatusV24::Passed);
        assert!(check.last_run_at.is_some());
    }

    #[test]
    fn test_automated_check_v24_is_due() {
        let mut check = AutomatedCheckV24::new("req-1".into(), ComplianceCheckTypeV23::Automated);
        assert!(check.is_due(60));
        check.record_result(CheckResultStatusV24::Passed);
        assert!(!check.is_due(60));
    }

    #[test]
    fn test_check_result_status_v24() {
        assert!(CheckResultStatusV24::Passed.is_passing());
        assert!(CheckResultStatusV24::Skipped.is_passing());
        assert!(!CheckResultStatusV24::Failed.is_passing());
        assert!(CheckResultStatusV24::Failed.is_failing());
        assert!(CheckResultStatusV24::Error.is_failing());
    }

    #[test]
    fn test_check_result_v24_new() {
        let result = CheckResultV24::new("c1".into(), CheckResultStatusV24::Passed);
        assert!(result.is_passing());
    }

    #[test]
    fn test_compliance_score_v24() {
        let mut score = ComplianceScoreV24::new("fw-1".into());
        let results = vec![
            CheckResultV24::new("c1".into(), CheckResultStatusV24::Passed),
            CheckResultV24::new("c2".into(), CheckResultStatusV24::Passed),
            CheckResultV24::new("c3".into(), CheckResultStatusV24::Failed),
        ];
        score.calculate(&results);
        assert_eq!(score.total_checks, 3);
        assert_eq!(score.passed, 2);
        assert_eq!(score.failed, 1);
        assert!(score.score_percentage > 60.0);
        assert_eq!(score.grade(), "D");
    }

    #[test]
    fn test_compliance_score_v24_compliant() {
        let mut score = ComplianceScoreV24::new("fw-1".into());
        let results = vec![
            CheckResultV24::new("c1".into(), CheckResultStatusV24::Passed),
            CheckResultV24::new("c2".into(), CheckResultStatusV24::Passed),
        ];
        score.calculate(&results);
        assert!(score.is_compliant());
    }

    #[test]
    fn test_gap_analysis_v24() {
        let mut analysis = ComplianceGapAnalysisV24::new("fw-1".into());
        analysis.add_gap(ComplianceGapItemV24 {
            requirement_id: "r1".into(),
            description: "Missing".into(),
            severity: RequirementSeverityV22::Critical,
            gap_type: GapTypeV24::NoEvidence,
            recommendation: "Add evidence".into(),
        });
        assert!(analysis.has_critical_gaps());
        assert_eq!(analysis.total_gaps, 1);
    }

    #[test]
    fn test_automated_check_runner_v24() {
        let mut runner = AutomatedCheckRunnerV24::new();
        let check = AutomatedCheckV24::new("req-1".into(), ComplianceCheckTypeV23::Automated);
        runner.register_check(check);
        assert_eq!(runner.total_checks(), 1);
        assert_eq!(runner.get_enabled_checks().len(), 1);
    }

    #[test]
    fn test_check_result_history_v24() {
        let mut history = CheckResultHistoryV24::new();
        let r1 = CheckResultV24::new("c1".into(), CheckResultStatusV24::Passed);
        let r2 = CheckResultV24::new("c1".into(), CheckResultStatusV24::Failed);
        history.record(r1);
        history.record(r2);
        assert_eq!(history.total_results(), 2);
        assert_eq!(history.get_results_for_check("c1").len(), 2);
        assert!((history.pass_rate() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_compliance_scoring_engine_v24() {
        let mut engine = ComplianceScoringEngineV24::new();
        let results = vec![CheckResultV24::new("c1".into(), CheckResultStatusV24::Passed)];
        let score = engine.calculate_score("fw-1", &results);
        assert_eq!(score.passed, 1);
        assert!(engine.get_score("fw-1").is_some());
    }

    #[test]
    fn test_gap_type_v24_display() {
        assert_eq!(GapTypeV24::MissingCheck.display_name(), "missing_check");
        assert_eq!(GapTypeV24::CheckFailing.display_name(), "check_failing");
        assert_eq!(GapTypeV24::NoEvidence.display_name(), "no_evidence");
    }
}
