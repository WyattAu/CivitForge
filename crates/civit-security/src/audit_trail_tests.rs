#![cfg(test)]

use super::audit_trail::*;
use chrono::{Duration, Utc};

fn test_event() -> AuditTrailEvent {
    AuditTrailEvent::new("push", "repository", "repo-1", "push")
}

fn test_event_with_risk(score: u32) -> AuditTrailEvent {
    let mut event = test_event();
    event.risk_score = score;
    event
}

// --- EventRiskScore ---

#[test]
fn test_event_risk_score_new() {
    let score = EventRiskScore::new("evt-1".into(), 65.0);
    assert_eq!(score.risk_score, 65.0);
    assert_eq!(score.risk_level(), RiskLevel::High);
}

#[test]
fn test_event_risk_score_clamping() {
    let score = EventRiskScore::new("evt-1".into(), 150.0);
    assert_eq!(score.risk_score, 100.0);

    let score = EventRiskScore::new("evt-1".into(), -10.0);
    assert_eq!(score.risk_score, 0.0);
}

#[test]
fn test_event_risk_score_levels() {
    assert_eq!(EventRiskScore::new("e".into(), 80.0).risk_level(), RiskLevel::Critical);
    assert_eq!(EventRiskScore::new("e".into(), 60.0).risk_level(), RiskLevel::High);
    assert_eq!(EventRiskScore::new("e".into(), 30.0).risk_level(), RiskLevel::Medium);
    assert_eq!(EventRiskScore::new("e".into(), 10.0).risk_level(), RiskLevel::Low);
}

#[test]
fn test_event_risk_score_with_factors() {
    let score = EventRiskScore::new("e".into(), 50.0)
        .with_factors(vec![
            RiskFactor::new("f1".into(), "desc".into(), RiskLevel::High, 0.5),
        ]);
    assert_eq!(score.risk_factors.len(), 1);
}

#[test]
fn test_event_risk_score_has_critical_factors() {
    let score = EventRiskScore::new("e".into(), 50.0)
        .with_factors(vec![
            RiskFactor::new("f1".into(), "d".into(), RiskLevel::Critical, 1.0),
        ]);
    assert!(score.has_critical_factors());

    let score = EventRiskScore::new("e".into(), 50.0)
        .with_factors(vec![
            RiskFactor::new("f1".into(), "d".into(), RiskLevel::Low, 0.1),
        ]);
    assert!(!score.has_critical_factors());
}

#[test]
fn test_event_risk_score_with_mitigations() {
    let score = EventRiskScore::new("e".into(), 50.0)
        .with_mitigations(vec![
            MitigationSuggestion::new("Fix it".into(), "Apply patch".into(), MitigationPriority::Immediate),
        ]);
    assert_eq!(score.mitigation_suggestions.len(), 1);
}

// --- RiskFactor ---

#[test]
fn test_risk_factor_new() {
    let factor = RiskFactor::new("auth".into(), "Weak auth".into(), RiskLevel::High, 0.7);
    assert_eq!(factor.name, "auth");
    assert_eq!(factor.weight, 0.7);
    assert_eq!(factor.severity, RiskLevel::High);
}

#[test]
fn test_risk_factor_weight_clamping() {
    let factor = RiskFactor::new("f".into(), "d".into(), RiskLevel::Low, 1.5);
    assert_eq!(factor.weight, 1.0);

    let factor = RiskFactor::new("f".into(), "d".into(), RiskLevel::Low, -0.5);
    assert_eq!(factor.weight, 0.0);
}

// --- MitigationSuggestion ---

#[test]
fn test_mitigation_suggestion_new() {
    let sug = MitigationSuggestion::new(
        "Patch vuln".into(),
        "Update to latest version".into(),
        MitigationPriority::High,
    );
    assert_eq!(sug.title, "Patch vuln");
    assert_eq!(sug.priority, MitigationPriority::High);
    assert!(sug.estimated_effort.is_none());
}

#[test]
fn test_mitigation_suggestion_with_effort() {
    let sug = MitigationSuggestion::new(
        "Audit".into(),
        "Review access".into(),
        MitigationPriority::Medium,
    )
    .with_effort("2 hours".into());

    assert_eq!(sug.estimated_effort.as_deref(), Some("2 hours"));
}

// --- MitigationPriority ---

#[test]
fn test_mitigation_priority_display() {
    assert_eq!(MitigationPriority::Immediate.display_name(), "immediate");
    assert_eq!(MitigationPriority::High.display_name(), "high");
    assert_eq!(MitigationPriority::Medium.display_name(), "medium");
    assert_eq!(MitigationPriority::Low.display_name(), "low");
}

// --- RetentionPolicy ---

#[test]
fn test_retention_policy_new() {
    let policy = RetentionPolicy::new("security".into(), 365);
    assert!(policy.enabled);
    assert_eq!(policy.retention_days, 365);
    assert!(policy.archive_after_days.is_none());
    assert!(policy.delete_after_days.is_none());
}

#[test]
fn test_retention_policy_is_expired() {
    let policy = RetentionPolicy::new("security".into(), 365);
    assert!(policy.is_expired(366));
    assert!(policy.is_expired(365));
    assert!(!policy.is_expired(364));
}

#[test]
fn test_retention_policy_archive_delete() {
    let policy = RetentionPolicy::new("security".into(), 365)
        .with_archive_after(90)
        .with_delete_after(180);

    assert!(policy.should_archive(91));
    assert!(!policy.should_archive(89));
    assert!(policy.should_delete(181));
    assert!(!policy.should_delete(179));
}

#[test]
fn test_retention_policy_no_archive_delete() {
    let policy = RetentionPolicy::new("security".into(), 365);
    assert!(!policy.should_archive(100));
    assert!(!policy.should_delete(100));
}

// --- ForensicTimeline ---

#[test]
fn test_forensic_timeline_new() {
    let timeline = ForensicTimeline::new("user-1".into());
    assert_eq!(timeline.actor_id, "user-1");
    assert_eq!(timeline.total_events, 0);
    assert_eq!(timeline.average_risk_score, 0.0);
}

#[test]
fn test_forensic_timeline_add_entry() {
    let mut timeline = ForensicTimeline::new("user-1".into());
    timeline.add_entry(ForensicTimelineEntryV24 {
        timestamp: Utc::now(),
        event: test_event(),
        risk_score: Some(EventRiskScore::new("e1".into(), 30.0)),
        correlation_ids: vec![],
        risk_level: RiskLevel::Medium,
        notes: None,
    });

    assert_eq!(timeline.total_events, 1);
    assert!(timeline.time_span_start.is_some());
    assert!(timeline.time_span_end.is_some());
}

#[test]
fn test_forensic_timeline_multiple_entries() {
    let mut timeline = ForensicTimeline::new("user-1".into());
    for i in 0..5 {
        timeline.add_entry(ForensicTimelineEntryV24 {
            timestamp: Utc::now() + Duration::minutes(i),
            event: test_event(),
            risk_score: Some(EventRiskScore::new(
                format!("e{}", i),
                (i as f64) * 20.0,
            )),
            correlation_ids: vec![],
            risk_level: if i < 2 { RiskLevel::Low } else { RiskLevel::High },
            notes: None,
        });
    }

    assert_eq!(timeline.total_events, 5);
    assert!(timeline.high_risk_count > 0);
}

#[test]
fn test_forensic_timeline_high_risk_entries() {
    let mut timeline = ForensicTimeline::new("user-1".into());
    timeline.add_entry(ForensicTimelineEntryV24 {
        timestamp: Utc::now(),
        event: test_event_with_risk(80),
        risk_score: None,
        correlation_ids: vec![],
        risk_level: RiskLevel::Critical,
        notes: None,
    });
    timeline.add_entry(ForensicTimelineEntryV24 {
        timestamp: Utc::now(),
        event: test_event_with_risk(10),
        risk_score: None,
        correlation_ids: vec![],
        risk_level: RiskLevel::Low,
        notes: None,
    });

    assert_eq!(timeline.high_risk_entries().len(), 1);
}

#[test]
fn test_forensic_timeline_entries_in_window() {
    let mut timeline = ForensicTimeline::new("user-1".into());
    let now = Utc::now();
    timeline.add_entry(ForensicTimelineEntryV24 {
        timestamp: now - Duration::hours(2),
        event: test_event(),
        risk_score: None,
        correlation_ids: vec![],
        risk_level: RiskLevel::Low,
        notes: None,
    });
    timeline.add_entry(ForensicTimelineEntryV24 {
        timestamp: now,
        event: test_event(),
        risk_score: None,
        correlation_ids: vec![],
        risk_level: RiskLevel::Low,
        notes: None,
    });

    let in_window = timeline.entries_in_window(now - Duration::hours(1), now + Duration::hours(1));
    assert_eq!(in_window.len(), 1);
}

// --- AuditFindingType ---

#[test]
fn test_audit_finding_type_display() {
    assert_eq!(AuditFindingType::UnscoredEvents.display_name(), "un_scored_events");
    assert_eq!(AuditFindingType::HighRiskConcentration.display_name(), "high_risk_concentration");
    assert_eq!(AuditFindingType::RetentionViolation.display_name(), "retention_violation");
    assert_eq!(AuditFindingType::AnomalousPattern.display_name(), "anomalous_pattern");
    assert_eq!(AuditFindingType::MissingMitigation.display_name(), "missing_mitigation");
}

// --- ComplianceAuditReport ---

#[test]
fn test_compliance_audit_report_new() {
    let report = ComplianceAuditReport::new("soc2".into(), "SOC 2 Type II".into());
    assert_eq!(report.framework_id, "soc2");
    assert_eq!(report.framework_name, "SOC 2 Type II");
    assert_eq!(report.total_events_audited, 0);
    assert!(!report.has_critical_findings());
}

#[test]
fn test_compliance_audit_report_add_finding() {
    let mut report = ComplianceAuditReport::new("fw".into(), "Framework".into());
    report.add_finding(ComplianceAuditFinding {
        finding_type: AuditFindingType::UnscoredEvents,
        description: "5 events unscored".into(),
        severity: RiskLevel::Critical,
        event_count: 5,
        recommendation: "Apply scoring".into(),
    });

    assert!(report.has_critical_findings());
    assert_eq!(report.findings.len(), 1);
}

#[test]
fn test_compliance_audit_report_no_critical_findings() {
    let mut report = ComplianceAuditReport::new("fw".into(), "F".into());
    report.add_finding(ComplianceAuditFinding {
        finding_type: AuditFindingType::HighRiskConcentration,
        description: "Some finding".into(),
        severity: RiskLevel::Medium,
        event_count: 3,
        recommendation: "Review".into(),
    });

    assert!(!report.has_critical_findings());
}

// --- EventRiskScoringEngine ---

#[test]
fn test_scoring_engine_empty() {
    let engine = EventRiskScoringEngine::new();
    assert_eq!(engine.total_scored(), 0);
    assert_eq!(engine.average_score(), 0.0);
    assert!(engine.high_risk_scores().is_empty());
}

#[test]
fn test_scoring_engine_score_event() {
    let mut engine = EventRiskScoringEngine::new();
    let score = engine.score_event(&test_event(), vec![]);
    assert!(score.risk_score > 30.0);
    assert_eq!(engine.total_scored(), 1);
    assert!(engine.get_score_for_event(&score.event_id).is_some());
}

#[test]
fn test_scoring_engine_with_custom_factors() {
    let mut engine = EventRiskScoringEngine::new();
    let event = test_event_with_risk(20);
    let score = engine.score_event(&event, vec![
        RiskFactor::new("custom".into(), "Custom risk".into(), RiskLevel::High, 0.8),
    ]);
    assert!(score.risk_score > 20.0);
    assert_eq!(score.risk_factors.len(), 1);
}

#[test]
fn test_scoring_engine_high_base_risk() {
    let mut engine = EventRiskScoringEngine::new();
    let event = test_event_with_risk(80);
    let score = engine.score_event(&event, vec![]);
    assert!(score.risk_score >= 80.0);
    assert_eq!(engine.high_risk_scores().len(), 1);
}

// --- RetentionPolicyManager ---

#[test]
fn test_retention_manager_empty() {
    let manager = RetentionPolicyManager::new();
    assert!(manager.get_policy_for_category("security").is_none());
    assert!(manager.get_enabled_policies().is_empty());
}

#[test]
fn test_retention_manager_add_policy() {
    let mut manager = RetentionPolicyManager::new();
    manager.add_policy(RetentionPolicy::new("security".into(), 365));

    assert!(manager.get_policy_for_category("security").is_some());
    assert_eq!(manager.get_all_policies().len(), 1);
    assert_eq!(manager.get_enabled_policies().len(), 1);
}

#[test]
fn test_retention_manager_disable_policy() {
    let mut manager = RetentionPolicyManager::new();
    let policy = RetentionPolicy::new("security".into(), 365);
    let policy_id = policy.id.clone();
    manager.add_policy(policy);

    manager.disable_policy(&policy_id).unwrap();
    assert!(manager.get_policy_for_category("security").is_none());
    assert!(manager.get_enabled_policies().is_empty());
}

#[test]
fn test_retention_manager_disable_not_found() {
    let mut manager = RetentionPolicyManager::new();
    assert!(manager.disable_policy("nonexistent").is_err());
}

#[test]
fn test_retention_manager_archive_delete_queries() {
    let mut manager = RetentionPolicyManager::new();
    manager.add_policy(
        RetentionPolicy::new("security".into(), 365)
            .with_archive_after(90)
            .with_delete_after(180),
    );

    assert!(manager.events_to_archive("security", 91));
    assert!(!manager.events_to_archive("security", 89));
    assert!(manager.events_to_delete("security", 181));
    assert!(!manager.events_to_delete("security", 179));
}

#[test]
fn test_retention_manager_unknown_category() {
    let manager = RetentionPolicyManager::new();
    assert!(!manager.events_to_archive("unknown", 100));
    assert!(!manager.events_to_delete("unknown", 100));
}

// --- ForensicTimelineBuilder ---

#[test]
fn test_timeline_builder_empty() {
    let builder = ForensicTimelineBuilder::new();
    assert_eq!(builder.actor_count(), 0);
    assert!(builder.get_timeline("user-1").is_none());
}

#[test]
fn test_timeline_builder_get_or_create() {
    let mut builder = ForensicTimelineBuilder::new();
    {
        let timeline = builder.get_or_create_timeline("user-1");
        timeline.add_entry(ForensicTimelineEntryV24 {
            timestamp: Utc::now(),
            event: test_event(),
            risk_score: None,
            correlation_ids: vec![],
            risk_level: RiskLevel::Low,
            notes: None,
        });
    }

    assert_eq!(builder.actor_count(), 1);
    assert!(builder.get_timeline("user-1").is_some());
    assert!(builder.get_timeline("user-2").is_none());
}

#[test]
fn test_timeline_builder_multiple_actors() {
    let mut builder = ForensicTimelineBuilder::new();
    builder.get_or_create_timeline("user-1");
    builder.get_or_create_timeline("user-2");
    builder.get_or_create_timeline("user-3");

    assert_eq!(builder.actor_count(), 3);
    assert_eq!(builder.all_timelines().len(), 3);
}

// --- ComplianceReportingEngine ---

#[test]
fn test_reporting_engine_empty() {
    let engine = ComplianceReportingEngine::new();
    assert!(engine.get_reports().is_empty());
    assert!(engine.latest_report("fw-1").is_none());
}

#[test]
fn test_reporting_engine_generate_report() {
    let mut engine = ComplianceReportingEngine::new();
    let mut scoring = EventRiskScoringEngine::new();
    scoring.score_event(&test_event_with_risk(80), vec![]);
    scoring.score_event(&test_event_with_risk(10), vec![]);

    let policies = RetentionPolicyManager::new();
    let report = engine.generate_report("soc2", "SOC 2", &scoring, &policies);

    assert_eq!(report.risk_scores_counted, 2);
    assert_eq!(report.total_events_audited, 2);
    assert!(report.average_risk_score > 0.0);
    assert!(engine.latest_report("soc2").is_some());
    assert_eq!(engine.get_reports().len(), 1);
}

#[test]
fn test_reporting_engine_critical_events_finding() {
    let mut engine = ComplianceReportingEngine::new();
    let mut scoring = EventRiskScoringEngine::new();
    scoring.score_event(&test_event_with_risk(80), vec![]);

    let policies = RetentionPolicyManager::new();
    let report = engine.generate_report("fw", "Framework", &scoring, &policies);

    assert!(report.critical_events > 0);
    assert!(report.findings.iter().any(|f| f.finding_type == AuditFindingType::HighRiskConcentration));
}

#[test]
fn test_reporting_engine_with_retention_policies() {
    let mut engine = ComplianceReportingEngine::new();
    let scoring = EventRiskScoringEngine::new();
    let mut policies = RetentionPolicyManager::new();
    policies.add_policy(RetentionPolicy::new("security".into(), 365));

    let report = engine.generate_report("fw", "F", &scoring, &policies);
    assert_eq!(report.retention_policies_applied, 1);
}
