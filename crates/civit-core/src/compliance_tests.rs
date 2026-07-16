#![cfg(test)]

use super::compliance_v24::*;
use super::compliance_v22::{RequirementSeverityV22, EvidenceTypeV22};
use super::compliance_v23::{ComplianceCheckTypeV23, ComplianceEvidenceItemV23};
use std::collections::HashMap;

// --- AutomatedCheckV24 ---

#[test]
fn test_automated_check_new() {
    let check = AutomatedCheckV24::new("req-1".into(), ComplianceCheckTypeV23::Automated);
    assert_eq!(check.requirement_id, "req-1");
    assert!(check.enabled);
    assert_eq!(check.last_result, CheckResultStatusV24::Pending);
    assert!(check.last_run_at.is_none());
    assert!(check.check_config.is_empty());
}

#[test]
fn test_automated_check_with_config() {
    let mut config = HashMap::new();
    config.insert("url".into(), serde_json::json!("https://example.com"));
    let check = AutomatedCheckV24::new("req-2".into(), ComplianceCheckTypeV23::Manual)
        .with_config(config);

    assert_eq!(check.check_config.len(), 1);
    assert_eq!(check.check_config.get("url").unwrap(), &serde_json::json!("https://example.com"));
}

#[test]
fn test_automated_check_record_result() {
    let mut check = AutomatedCheckV24::new("req-1".into(), ComplianceCheckTypeV23::Automated);
    assert!(check.last_run_at.is_none());

    check.record_result(CheckResultStatusV24::Passed);
    assert_eq!(check.last_result, CheckResultStatusV24::Passed);
    assert!(check.last_run_at.is_some());
}

#[test]
fn test_automated_check_is_due() {
    let mut check = AutomatedCheckV24::new("req-1".into(), ComplianceCheckTypeV23::Automated);
    assert!(check.is_due(60));

    check.record_result(CheckResultStatusV24::Passed);
    assert!(!check.is_due(60));
}

#[test]
fn test_automated_check_enable_disable() {
    let mut check = AutomatedCheckV24::new("req-1".into(), ComplianceCheckTypeV23::Automated);
    assert!(check.enabled);

    check.disable();
    assert!(!check.enabled);

    check.enable();
    assert!(check.enabled);
}

// --- CheckResultStatusV24 ---

#[test]
fn test_check_result_status_display_names() {
    assert_eq!(CheckResultStatusV24::Pending.display_name(), "pending");
    assert_eq!(CheckResultStatusV24::Passed.display_name(), "passed");
    assert_eq!(CheckResultStatusV24::Failed.display_name(), "failed");
    assert_eq!(CheckResultStatusV24::Warning.display_name(), "warning");
    assert_eq!(CheckResultStatusV24::Error.display_name(), "error");
    assert_eq!(CheckResultStatusV24::Skipped.display_name(), "skipped");
}

#[test]
fn test_check_result_status_is_passing() {
    assert!(CheckResultStatusV24::Passed.is_passing());
    assert!(CheckResultStatusV24::Skipped.is_passing());
    assert!(!CheckResultStatusV24::Pending.is_passing());
    assert!(!CheckResultStatusV24::Failed.is_passing());
    assert!(!CheckResultStatusV24::Warning.is_passing());
    assert!(!CheckResultStatusV24::Error.is_passing());
}

#[test]
fn test_check_result_status_is_failing() {
    assert!(CheckResultStatusV24::Failed.is_failing());
    assert!(CheckResultStatusV24::Error.is_failing());
    assert!(!CheckResultStatusV24::Passed.is_failing());
    assert!(!CheckResultStatusV24::Skipped.is_failing());
    assert!(!CheckResultStatusV24::Warning.is_failing());
    assert!(!CheckResultStatusV24::Pending.is_failing());
}

// --- CheckResultV24 ---

#[test]
fn test_check_result_new() {
    let result = CheckResultV24::new("check-1".into(), CheckResultStatusV24::Passed);
    assert!(result.is_passing());
    assert!(result.details.is_empty());
}

#[test]
fn test_check_result_with_details() {
    let mut details = HashMap::new();
    details.insert("output".into(), serde_json::json!("ok"));
    let result = CheckResultV24::new("c1".into(), CheckResultStatusV24::Failed)
        .with_details(details);

    assert!(!result.is_passing());
    assert_eq!(result.details.len(), 1);
}

// --- ComplianceScoreV24 ---

#[test]
fn test_compliance_score_new() {
    let score = ComplianceScoreV24::new("soc2".into());
    assert_eq!(score.framework_id, "soc2");
    assert_eq!(score.total_checks, 0);
    assert_eq!(score.score_percentage, 0.0);
}

#[test]
fn test_compliance_score_calculate_all_pass() {
    let mut score = ComplianceScoreV24::new("soc2".into());
    let results = vec![
        CheckResultV24::new("c1".into(), CheckResultStatusV24::Passed),
        CheckResultV24::new("c2".into(), CheckResultStatusV24::Passed),
    ];
    score.calculate(&results);

    assert_eq!(score.total_checks, 2);
    assert_eq!(score.passed, 2);
    assert_eq!(score.failed, 0);
    assert_eq!(score.score_percentage, 100.0);
    assert!(score.is_compliant());
}

#[test]
fn test_compliance_score_calculate_mixed() {
    let mut score = ComplianceScoreV24::new("soc2".into());
    let results = vec![
        CheckResultV24::new("c1".into(), CheckResultStatusV24::Passed),
        CheckResultV24::new("c2".into(), CheckResultStatusV24::Passed),
        CheckResultV24::new("c3".into(), CheckResultStatusV24::Failed),
        CheckResultV24::new("c4".into(), CheckResultStatusV24::Warning),
    ];
    score.calculate(&results);

    assert_eq!(score.total_checks, 4);
    assert_eq!(score.passed, 2);
    assert_eq!(score.failed, 1);
    assert_eq!(score.warnings, 1);
    assert!(!score.is_compliant());
}

#[test]
fn test_compliance_score_calculate_with_pending() {
    let mut score = ComplianceScoreV24::new("soc2".into());
    let results = vec![
        CheckResultV24::new("c1".into(), CheckResultStatusV24::Passed),
        CheckResultV24::new("c2".into(), CheckResultStatusV24::Pending),
    ];
    score.calculate(&results);

    assert_eq!(score.pending, 1);
    assert_eq!(score.total_checks, 2);
    // 1 passed out of 1 applicable = 100%
    assert_eq!(score.score_percentage, 100.0);
}

#[test]
fn test_compliance_score_all_pending() {
    let mut score = ComplianceScoreV24::new("soc2".into());
    let results = vec![
        CheckResultV24::new("c1".into(), CheckResultStatusV24::Pending),
        CheckResultV24::new("c2".into(), CheckResultStatusV24::Pending),
    ];
    score.calculate(&results);
    assert_eq!(score.score_percentage, 100.0);
}

#[test]
fn test_compliance_score_grades() {
    let mut score = ComplianceScoreV24::new("fw".into());
    score.score_percentage = 96.0;
    assert_eq!(score.grade(), "A+");

    score.score_percentage = 92.0;
    assert_eq!(score.grade(), "A");

    score.score_percentage = 85.0;
    assert_eq!(score.grade(), "B");

    score.score_percentage = 75.0;
    assert_eq!(score.grade(), "C");

    score.score_percentage = 65.0;
    assert_eq!(score.grade(), "D");

    score.score_percentage = 50.0;
    assert_eq!(score.grade(), "F");
}

#[test]
fn test_compliance_score_not_compliant_if_failures() {
    let mut score = ComplianceScoreV24::new("fw".into());
    score.score_percentage = 95.0;
    score.failed = 1;
    assert!(!score.is_compliant());
}

#[test]
fn test_compliance_score_compliant_if_no_failures() {
    let mut score = ComplianceScoreV24::new("fw".into());
    score.score_percentage = 85.0;
    score.failed = 0;
    assert!(score.is_compliant());
}

// --- ComplianceGapItemV24 ---

#[test]
fn test_compliance_gap_item() {
    let gap = ComplianceGapItemV24 {
        requirement_id: "req-1".into(),
        description: "Missing evidence".into(),
        severity: RequirementSeverityV22::High,
        gap_type: GapTypeV24::NoEvidence,
        recommendation: "Collect evidence".into(),
    };
    assert_eq!(gap.requirement_id, "req-1");
    assert_eq!(gap.gap_type, GapTypeV24::NoEvidence);
}

// --- GapTypeV24 ---

#[test]
fn test_gap_type_display_names() {
    assert_eq!(GapTypeV24::MissingCheck.display_name(), "missing_check");
    assert_eq!(GapTypeV24::CheckFailing.display_name(), "check_failing");
    assert_eq!(GapTypeV24::NoEvidence.display_name(), "no_evidence");
    assert_eq!(GapTypeV24::InsufficientEvidence.display_name(), "insufficient_evidence");
    assert_eq!(GapTypeV24::ExpiredEvidence.display_name(), "expired_evidence");
}

// --- ComplianceGapAnalysisV24 ---

#[test]
fn test_gap_analysis_new() {
    let analysis = ComplianceGapAnalysisV24::new("soc2".into());
    assert_eq!(analysis.framework_id, "soc2");
    assert_eq!(analysis.total_gaps, 0);
    assert!(!analysis.has_critical_gaps());
}

#[test]
fn test_gap_analysis_add_gap() {
    let mut analysis = ComplianceGapAnalysisV24::new("soc2".into());
    analysis.add_gap(ComplianceGapItemV24 {
        requirement_id: "r1".into(),
        description: "Missing".into(),
        severity: RequirementSeverityV22::Critical,
        gap_type: GapTypeV24::NoEvidence,
        recommendation: "Add evidence".into(),
    });

    assert!(analysis.has_critical_gaps());
    assert_eq!(analysis.total_gaps, 1);
    assert_eq!(analysis.critical_gaps, 1);
    assert_eq!(analysis.gaps_by_severity(RequirementSeverityV22::Critical).len(), 1);
}

#[test]
fn test_gap_analysis_severity_breakdown() {
    let mut analysis = ComplianceGapAnalysisV24::new("fw".into());
    analysis.add_gap(ComplianceGapItemV24 {
        requirement_id: "r1".into(),
        description: "d".into(),
        severity: RequirementSeverityV22::Critical,
        gap_type: GapTypeV24::MissingCheck,
        recommendation: "r".into(),
    });
    analysis.add_gap(ComplianceGapItemV24 {
        requirement_id: "r2".into(),
        description: "d".into(),
        severity: RequirementSeverityV22::High,
        gap_type: GapTypeV24::CheckFailing,
        recommendation: "r".into(),
    });
    analysis.add_gap(ComplianceGapItemV24 {
        requirement_id: "r3".into(),
        description: "d".into(),
        severity: RequirementSeverityV22::Medium,
        gap_type: GapTypeV24::NoEvidence,
        recommendation: "r".into(),
    });
    analysis.add_gap(ComplianceGapItemV24 {
        requirement_id: "r4".into(),
        description: "d".into(),
        severity: RequirementSeverityV22::Low,
        gap_type: GapTypeV24::MissingCheck,
        recommendation: "r".into(),
    });

    assert_eq!(analysis.total_gaps, 4);
    assert_eq!(analysis.critical_gaps, 1);
    assert_eq!(analysis.high_gaps, 1);
    assert_eq!(analysis.medium_gaps, 1);
    assert_eq!(analysis.low_gaps, 1);
}

// --- AutomatedCheckRunnerV24 ---

#[test]
fn test_check_runner_empty() {
    let runner = AutomatedCheckRunnerV24::new();
    assert_eq!(runner.total_checks(), 0);
    assert!(runner.get_enabled_checks().is_empty());
    assert!(runner.get_due_checks(60).is_empty());
}

#[test]
fn test_check_runner_register_and_query() {
    let mut runner = AutomatedCheckRunnerV24::new();
    let check1 = AutomatedCheckV24::new("req-1".into(), ComplianceCheckTypeV23::Automated);
    let check2 = AutomatedCheckV24::new("req-2".into(), ComplianceCheckTypeV23::Manual);
    runner.register_check(check1);
    runner.register_check(check2);

    assert_eq!(runner.total_checks(), 2);
    assert_eq!(runner.get_enabled_checks().len(), 2);
    assert_eq!(runner.get_checks_for_requirement("req-1").len(), 1);
    assert_eq!(runner.get_checks_for_requirement("req-2").len(), 1);
    assert!(runner.get_checks_for_requirement("req-3").is_empty());
}

#[test]
fn test_check_runner_disabled_check() {
    let mut runner = AutomatedCheckRunnerV24::new();
    let mut check = AutomatedCheckV24::new("req-1".into(), ComplianceCheckTypeV23::Automated);
    check.disable();
    runner.register_check(check);

    assert_eq!(runner.total_checks(), 1);
    assert!(runner.get_enabled_checks().is_empty());
}

#[test]
fn test_check_runner_record_result() {
    let mut runner = AutomatedCheckRunnerV24::new();
    let check = AutomatedCheckV24::new("req-1".into(), ComplianceCheckTypeV23::Automated);
    runner.register_check(check);

    let check_id = runner.get_checks_for_requirement("req-1")[0].id.clone();
    runner.record_result(&check_id, CheckResultStatusV24::Passed);

    let updated = &runner.get_checks_for_requirement("req-1")[0];
    assert_eq!(updated.last_result, CheckResultStatusV24::Passed);
}

// --- CheckResultHistoryV24 ---

#[test]
fn test_check_result_history_empty() {
    let history = CheckResultHistoryV24::new();
    assert_eq!(history.total_results(), 0);
    assert_eq!(history.pass_rate(), 100.0);
}

#[test]
fn test_check_result_history_record() {
    let mut history = CheckResultHistoryV24::new();
    history.record(CheckResultV24::new("c1".into(), CheckResultStatusV24::Passed));
    history.record(CheckResultV24::new("c1".into(), CheckResultStatusV24::Failed));

    assert_eq!(history.total_results(), 2);
    assert_eq!(history.get_results_for_check("c1").len(), 2);
    assert!((history.pass_rate() - 50.0).abs() < 0.01);
}

#[test]
fn test_check_result_history_latest_result() {
    let mut history = CheckResultHistoryV24::new();
    history.record(CheckResultV24::new("c1".into(), CheckResultStatusV24::Failed));
    history.record(CheckResultV24::new("c1".into(), CheckResultStatusV24::Passed));

    let latest = history.latest_result_for_check("c1");
    assert!(latest.is_some());
    assert_eq!(latest.unwrap().result, CheckResultStatusV24::Passed);
}

// --- ComplianceScoringEngineV24 ---

#[test]
fn test_scoring_engine_empty() {
    let engine = ComplianceScoringEngineV24::new();
    assert!(engine.get_score("fw-1").is_none());
    assert!(engine.all_scores().is_empty());
    assert!(engine.compliant_frameworks().is_empty());
}

#[test]
fn test_scoring_engine_calculate_and_query() {
    let mut engine = ComplianceScoringEngineV24::new();
    let results = vec![
        CheckResultV24::new("c1".into(), CheckResultStatusV24::Passed),
        CheckResultV24::new("c2".into(), CheckResultStatusV24::Passed),
    ];
    let score = engine.calculate_score("soc2", &results);

    assert_eq!(score.passed, 2);
    assert!(engine.get_score("soc2").is_some());
    assert_eq!(engine.compliant_frameworks().len(), 1);
}

#[test]
fn test_scoring_engine_non_compliant() {
    let mut engine = ComplianceScoringEngineV24::new();
    let results = vec![
        CheckResultV24::new("c1".into(), CheckResultStatusV24::Failed),
    ];
    engine.calculate_score("soc2", &results);
    assert!(engine.compliant_frameworks().is_empty());
}

// --- GapAnalysisEngineV24 ---

#[test]
fn test_gap_analysis_engine_empty() {
    let engine = GapAnalysisEngineV24::new();
    assert!(engine.latest_analysis("fw-1").is_none());
}

#[test]
fn test_gap_analysis_engine_run_analysis() {
    let mut engine = GapAnalysisEngineV24::new();
    let checks = vec![
        AutomatedCheckV24::new("req-1".into(), ComplianceCheckTypeV23::Automated),
        AutomatedCheckV24::new("req-2".into(), ComplianceCheckTypeV23::Manual),
    ];
    let evidence = vec![ComplianceEvidenceItemV23::new(
        "req-1".into(),
        EvidenceTypeV22::Manual,
    )];

    let analysis = engine.run_analysis("soc2", &checks, &evidence);
    assert_eq!(analysis.framework_id, "soc2");
    assert!(analysis.total_gaps > 0);

    // req-2 has no evidence → gap
    assert!(analysis.gaps.iter().any(|g| g.requirement_id == "req-2"));
    // Both checks are pending → gaps
    assert!(analysis.gaps.iter().any(|g| g.gap_type == GapTypeV24::MissingCheck));

    assert!(engine.latest_analysis("soc2").is_some());
}

#[test]
fn test_gap_analysis_engine_failing_check() {
    let mut engine = GapAnalysisEngineV24::new();
    let mut check = AutomatedCheckV24::new("req-1".into(), ComplianceCheckTypeV23::Automated);
    check.record_result(CheckResultStatusV24::Failed);

    let analysis = engine.run_analysis("fw", &[check], &[]);
    assert!(analysis.gaps.iter().any(|g| g.gap_type == GapTypeV24::CheckFailing));
}

#[test]
fn test_gap_analysis_engine_passing_check_no_gap() {
    let mut engine = GapAnalysisEngineV24::new();
    let mut check = AutomatedCheckV24::new("req-1".into(), ComplianceCheckTypeV23::Automated);
    check.record_result(CheckResultStatusV24::Passed);

    let evidence = vec![ComplianceEvidenceItemV23::new(
        "req-1".into(),
        EvidenceTypeV22::Manual,
    )];
    let analysis = engine.run_analysis("fw", &[check], &evidence);
    assert_eq!(analysis.total_gaps, 0);
}
