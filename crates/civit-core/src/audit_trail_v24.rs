#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::audit_trail_v23::RiskTrendV23;
use crate::audit_trail_v22::{AuditTrailEventV22, RiskLevelV22};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRiskScoreV24 {
    pub id: String,
    pub event_id: String,
    pub risk_score: f64,
    pub risk_factors: Vec<RiskFactorV24>,
    pub mitigation_suggestions: Vec<MitigationSuggestionV24>,
    pub scored_at: DateTime<Utc>,
}

impl EventRiskScoreV24 {
    pub fn new(event_id: String, risk_score: f64) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_id,
            risk_score: risk_score.clamp(0.0, 100.0),
            risk_factors: Vec::new(),
            mitigation_suggestions: Vec::new(),
            scored_at: Utc::now(),
        }
    }

    pub fn with_factors(mut self, factors: Vec<RiskFactorV24>) -> Self {
        self.risk_factors = factors;
        self
    }

    pub fn with_mitigations(mut self, mitigations: Vec<MitigationSuggestionV24>) -> Self {
        self.mitigation_suggestions = mitigations;
        self
    }

    pub fn risk_level(&self) -> RiskLevelV22 {
        if self.risk_score >= 75.0 {
            RiskLevelV22::Critical
        } else if self.risk_score >= 50.0 {
            RiskLevelV22::High
        } else if self.risk_score >= 25.0 {
            RiskLevelV22::Medium
        } else {
            RiskLevelV22::Low
        }
    }

    pub fn has_critical_factors(&self) -> bool {
        self.risk_factors
            .iter()
            .any(|f| f.severity == RiskLevelV22::Critical)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactorV24 {
    pub name: String,
    pub description: String,
    pub severity: RiskLevelV22,
    pub weight: f64,
}

impl RiskFactorV24 {
    pub fn new(name: String, description: String, severity: RiskLevelV22, weight: f64) -> Self {
        Self {
            name,
            description,
            severity,
            weight: weight.clamp(0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationSuggestionV24 {
    pub title: String,
    pub description: String,
    pub priority: MitigationPriorityV24,
    pub estimated_effort: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MitigationPriorityV24 {
    Immediate,
    High,
    Medium,
    Low,
}

impl MitigationPriorityV24 {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Immediate => "immediate",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

impl MitigationSuggestionV24 {
    pub fn new(title: String, description: String, priority: MitigationPriorityV24) -> Self {
        Self {
            title,
            description,
            priority,
            estimated_effort: None,
        }
    }

    pub fn with_effort(mut self, effort: String) -> Self {
        self.estimated_effort = Some(effort);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicyV24 {
    pub id: String,
    pub event_category: String,
    pub retention_days: u32,
    pub archive_after_days: Option<u32>,
    pub delete_after_days: Option<u32>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

impl RetentionPolicyV24 {
    pub fn new(event_category: String, retention_days: u32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_category,
            retention_days,
            archive_after_days: None,
            delete_after_days: None,
            enabled: true,
            created_at: Utc::now(),
        }
    }

    pub fn with_archive_after(mut self, days: u32) -> Self {
        self.archive_after_days = Some(days);
        self
    }

    pub fn with_delete_after(mut self, days: u32) -> Self {
        self.delete_after_days = Some(days);
        self
    }

    pub fn should_archive(&self, event_age_days: u32) -> bool {
        self.archive_after_days
            .map_or(false, |days| event_age_days >= days)
    }

    pub fn should_delete(&self, event_age_days: u32) -> bool {
        self.delete_after_days
            .map_or(false, |days| event_age_days >= days)
    }

    pub fn is_expired(&self, event_age_days: u32) -> bool {
        event_age_days >= self.retention_days
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicTimelineEntryV24 {
    pub timestamp: DateTime<Utc>,
    pub event: AuditTrailEventV22,
    pub risk_score: Option<EventRiskScoreV24>,
    pub correlation_ids: Vec<String>,
    pub risk_level: RiskLevelV22,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicTimelineV24 {
    pub actor_id: String,
    pub entries: Vec<ForensicTimelineEntryV24>,
    pub time_span_start: Option<DateTime<Utc>>,
    pub time_span_end: Option<DateTime<Utc>>,
    pub total_events: u32,
    pub high_risk_count: u32,
    pub average_risk_score: f64,
    pub risk_trend: RiskTrendV23,
}

impl ForensicTimelineV24 {
    pub fn new(actor_id: String) -> Self {
        Self {
            actor_id,
            entries: Vec::new(),
            time_span_start: None,
            time_span_end: None,
            total_events: 0,
            high_risk_count: 0,
            average_risk_score: 0.0,
            risk_trend: RiskTrendV23::Stable,
        }
    }

    pub fn add_entry(&mut self, entry: ForensicTimelineEntryV24) {
        self.entries.push(entry);
        self.recalculate();
    }

    fn recalculate(&mut self) {
        self.entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        self.total_events = self.entries.len() as u32;
        self.time_span_start = self.entries.first().map(|e| e.timestamp);
        self.time_span_end = self.entries.last().map(|e| e.timestamp);

        self.high_risk_count = self
            .entries
            .iter()
            .filter(|e| e.risk_level == RiskLevelV22::Critical || e.risk_level == RiskLevelV22::High)
            .count() as u32;

        if self.total_events > 0 {
            self.average_risk_score = self
                .entries
                .iter()
                .map(|e| {
                    e.risk_score
                        .as_ref()
                        .map(|s| s.risk_score)
                        .unwrap_or(0.0)
                })
                .sum::<f64>()
                / self.total_events as f64;
        }

        if self.entries.len() >= 2 {
            let mid = self.entries.len() / 2;
            let first_half: f64 = self.entries[..mid]
                .iter()
                .map(|e| {
                    e.risk_score
                        .as_ref()
                        .map(|s| s.risk_score)
                        .unwrap_or(0.0)
                })
                .sum::<f64>()
                / mid as f64;
            let second_half: f64 = self.entries[mid..]
                .iter()
                .map(|e| {
                    e.risk_score
                        .as_ref()
                        .map(|s| s.risk_score)
                        .unwrap_or(0.0)
                })
                .sum::<f64>()
                / (self.entries.len() - mid) as f64;
            self.risk_trend = if second_half > first_half * 1.1 {
                RiskTrendV23::Increasing
            } else if second_half < first_half * 0.9 {
                RiskTrendV23::Decreasing
            } else {
                RiskTrendV23::Stable
            };
        }
    }

    pub fn entries_in_window(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&ForensicTimelineEntryV24> {
        self.entries
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect()
    }

    pub fn high_risk_entries(&self) -> Vec<&ForensicTimelineEntryV24> {
        self.entries
            .iter()
            .filter(|e| e.risk_level == RiskLevelV22::Critical || e.risk_level == RiskLevelV22::High)
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAuditReportV24 {
    pub framework_id: String,
    pub framework_name: String,
    pub report_period_start: DateTime<Utc>,
    pub report_period_end: DateTime<Utc>,
    pub total_events_audited: u32,
    pub risk_scores_counted: u32,
    pub average_risk_score: f64,
    pub critical_events: u32,
    pub high_risk_events: u32,
    pub retention_policies_applied: u32,
    pub findings: Vec<ComplianceAuditFindingV24>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAuditFindingV24 {
    pub finding_type: AuditFindingTypeV24,
    pub description: String,
    pub severity: RiskLevelV22,
    pub event_count: u32,
    pub recommendation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditFindingTypeV24 {
    UnscoredEvents,
    HighRiskConcentration,
    RetentionViolation,
    AnomalousPattern,
    MissingMitigation,
}

impl AuditFindingTypeV24 {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::UnscoredEvents => "un_scored_events",
            Self::HighRiskConcentration => "high_risk_concentration",
            Self::RetentionViolation => "retention_violation",
            Self::AnomalousPattern => "anomalous_pattern",
            Self::MissingMitigation => "missing_mitigation",
        }
    }
}

impl ComplianceAuditReportV24 {
    pub fn new(framework_id: String, framework_name: String) -> Self {
        let now = Utc::now();
        Self {
            framework_id,
            framework_name,
            report_period_start: now - chrono::Duration::days(30),
            report_period_end: now,
            total_events_audited: 0,
            risk_scores_counted: 0,
            average_risk_score: 0.0,
            critical_events: 0,
            high_risk_events: 0,
            retention_policies_applied: 0,
            findings: Vec::new(),
            generated_at: now,
        }
    }

    pub fn add_finding(&mut self, finding: ComplianceAuditFindingV24) {
        self.findings.push(finding);
    }

    pub fn has_critical_findings(&self) -> bool {
        self.findings
            .iter()
            .any(|f| f.severity == RiskLevelV22::Critical)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventRiskScoringEngineV24 {
    scores: Vec<EventRiskScoreV24>,
    by_event: HashMap<String, Vec<usize>>,
}

impl EventRiskScoringEngineV24 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn score_event(
        &mut self,
        event: &AuditTrailEventV22,
        custom_factors: Vec<RiskFactorV24>,
    ) -> EventRiskScoreV24 {
        let mut base_score = event.risk_score as f64;
        let mut factors = Vec::new();

        if event.risk_score >= 75 {
            factors.push(RiskFactorV24::new(
                "high_base_risk".into(),
                "Event has high base risk score".into(),
                RiskLevelV22::Critical,
                0.4,
            ));
            base_score += 15.0;
        }

        factors.extend(custom_factors);
        for factor in &factors {
            base_score += factor.weight * 20.0;
        }

        let score = EventRiskScoreV24::new(event.id.clone(), base_score.min(100.0))
            .with_factors(factors);

        let idx = self.scores.len();
        self.by_event
            .entry(score.event_id.clone())
            .or_default()
            .push(idx);
        self.scores.push(score.clone());
        score
    }

    pub fn get_score_for_event(&self, event_id: &str) -> Option<&EventRiskScoreV24> {
        self.by_event
            .get(event_id)
            .and_then(|indices| indices.first().map(|&idx| &self.scores[idx]))
    }

    pub fn high_risk_scores(&self) -> Vec<&EventRiskScoreV24> {
        self.scores
            .iter()
            .filter(|s| s.risk_score >= 75.0)
            .collect()
    }

    pub fn average_score(&self) -> f64 {
        if self.scores.is_empty() {
            return 0.0;
        }
        self.scores.iter().map(|s| s.risk_score).sum::<f64>() / self.scores.len() as f64
    }

    pub fn total_scored(&self) -> usize {
        self.scores.len()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RetentionPolicyManagerV24 {
    policies: Vec<RetentionPolicyV24>,
    by_category: HashMap<String, Vec<usize>>,
}

impl RetentionPolicyManagerV24 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_policy(&mut self, policy: RetentionPolicyV24) {
        let idx = self.policies.len();
        self.by_category
            .entry(policy.event_category.clone())
            .or_default()
            .push(idx);
        self.policies.push(policy);
    }

    pub fn get_policy_for_category(&self, category: &str) -> Option<&RetentionPolicyV24> {
        self.by_category
            .get(category)
            .and_then(|indices| {
                indices
                    .iter()
                    .map(|&idx| &self.policies[idx])
                    .find(|p| p.enabled)
            })
    }

    pub fn get_all_policies(&self) -> &[RetentionPolicyV24] {
        &self.policies
    }

    pub fn get_enabled_policies(&self) -> Vec<&RetentionPolicyV24> {
        self.policies.iter().filter(|p| p.enabled).collect()
    }

    pub fn disable_policy(&mut self, id: &str) -> Result<(), String> {
        let policy = self
            .policies
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or("Policy not found")?;
        policy.enabled = false;
        Ok(())
    }

    pub fn events_to_archive(&self, category: &str, event_age_days: u32) -> bool {
        self.get_policy_for_category(category)
            .map_or(false, |p| p.should_archive(event_age_days))
    }

    pub fn events_to_delete(&self, category: &str, event_age_days: u32) -> bool {
        self.get_policy_for_category(category)
            .map_or(false, |p| p.should_delete(event_age_days))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForensicTimelineBuilderV24 {
    timelines: HashMap<String, ForensicTimelineV24>,
}

impl ForensicTimelineBuilderV24 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_create_timeline(&mut self, actor_id: &str) -> &mut ForensicTimelineV24 {
        self.timelines
            .entry(actor_id.into())
            .or_insert_with(|| ForensicTimelineV24::new(actor_id.into()))
    }

    pub fn get_timeline(&self, actor_id: &str) -> Option<&ForensicTimelineV24> {
        self.timelines.get(actor_id)
    }

    pub fn all_timelines(&self) -> &HashMap<String, ForensicTimelineV24> {
        &self.timelines
    }

    pub fn actor_count(&self) -> usize {
        self.timelines.len()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplianceReportingEngineV24 {
    reports: Vec<ComplianceAuditReportV24>,
}

impl ComplianceReportingEngineV24 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate_report(
        &mut self,
        framework_id: &str,
        framework_name: &str,
        risk_scores: &EventRiskScoringEngineV24,
        policies: &RetentionPolicyManagerV24,
    ) -> ComplianceAuditReportV24 {
        let mut report = ComplianceAuditReportV24::new(framework_id.into(), framework_name.into());

        report.risk_scores_counted = risk_scores.total_scored() as u32;
        report.average_risk_score = risk_scores.average_score();
        report.total_events_audited = risk_scores.total_scored() as u32;
        report.critical_events = risk_scores.high_risk_scores().len() as u32;
        report.retention_policies_applied = policies.get_enabled_policies().len() as u32;

        let unscored = report.total_events_audited - report.risk_scores_counted;
        if unscored > 0 {
            report.add_finding(ComplianceAuditFindingV24 {
                finding_type: AuditFindingTypeV24::UnscoredEvents,
                description: format!("{} events have not been risk-scored", unscored),
                severity: RiskLevelV22::Medium,
                event_count: unscored,
                recommendation: "Apply risk scoring to all audit events".into(),
            });
        }

        if report.critical_events > 0 {
            report.add_finding(ComplianceAuditFindingV24 {
                finding_type: AuditFindingTypeV24::HighRiskConcentration,
                description: format!("{} critical/high-risk events detected", report.critical_events),
                severity: RiskLevelV22::High,
                event_count: report.critical_events,
                recommendation: "Review critical events and apply mitigations".into(),
            });
        }

        self.reports.push(report.clone());
        report
    }

    pub fn get_reports(&self) -> &[ComplianceAuditReportV24] {
        &self.reports
    }

    pub fn latest_report(&self, framework_id: &str) -> Option<&ComplianceAuditReportV24> {
        self.reports
            .iter()
            .filter(|r| r.framework_id == framework_id)
            .max_by_key(|r| r.generated_at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_event() -> AuditTrailEventV22 {
        AuditTrailEventV22 {
            id: "evt-1".into(),
            action: "push".into(),
            actor_id: "user-1".into(),
            resource_type: "repository".into(),
            resource_id: "repo-1".into(),
            risk_score: 30,
            metadata: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_event_risk_score_v24_new() {
        let score = EventRiskScoreV24::new("evt-1".into(), 65.0);
        assert_eq!(score.risk_score, 65.0);
        assert_eq!(score.risk_level(), RiskLevelV22::High);
    }

    #[test]
    fn test_event_risk_score_v24_clamping() {
        let score = EventRiskScoreV24::new("evt-1".into(), 150.0);
        assert_eq!(score.risk_score, 100.0);
        let score = EventRiskScoreV24::new("evt-1".into(), -10.0);
        assert_eq!(score.risk_score, 0.0);
    }

    #[test]
    fn test_event_risk_score_v24_levels() {
        assert_eq!(EventRiskScoreV24::new("e".into(), 80.0).risk_level(), RiskLevelV22::Critical);
        assert_eq!(EventRiskScoreV24::new("e".into(), 60.0).risk_level(), RiskLevelV22::High);
        assert_eq!(EventRiskScoreV24::new("e".into(), 30.0).risk_level(), RiskLevelV22::Medium);
        assert_eq!(EventRiskScoreV24::new("e".into(), 10.0).risk_level(), RiskLevelV22::Low);
    }

    #[test]
    fn test_risk_factor_v24_new() {
        let factor = RiskFactorV24::new("test".into(), "desc".into(), RiskLevelV22::High, 0.5);
        assert_eq!(factor.weight, 0.5);
    }

    #[test]
    fn test_mitigation_suggestion_v24_new() {
        let sug = MitigationSuggestionV24::new("title".into(), "desc".into(), MitigationPriorityV24::High);
        assert_eq!(sug.priority, MitigationPriorityV24::High);
        assert!(sug.estimated_effort.is_none());
    }

    #[test]
    fn test_retention_policy_v24_new() {
        let policy = RetentionPolicyV24::new("security".into(), 365);
        assert!(policy.enabled);
        assert!(policy.is_expired(366));
        assert!(!policy.is_expired(364));
    }

    #[test]
    fn test_retention_policy_v24_archive_delete() {
        let policy = RetentionPolicyV24::new("security".into(), 365)
            .with_archive_after(90)
            .with_delete_after(180);
        assert!(policy.should_archive(91));
        assert!(!policy.should_archive(89));
        assert!(policy.should_delete(181));
        assert!(!policy.should_delete(179));
    }

    #[test]
    fn test_forensic_timeline_v24() {
        let mut timeline = ForensicTimelineV24::new("user-1".into());
        let entry = ForensicTimelineEntryV24 {
            timestamp: Utc::now(),
            event: test_event(),
            risk_score: None,
            correlation_ids: Vec::new(),
            risk_level: RiskLevelV22::Medium,
            notes: None,
        };
        timeline.add_entry(entry);
        assert_eq!(timeline.total_events, 1);
        assert!(timeline.time_span_start.is_some());
    }

    #[test]
    fn test_compliance_audit_report_v24() {
        let mut report = ComplianceAuditReportV24::new("fw-1".into(), "SOC 2".into());
        report.add_finding(ComplianceAuditFindingV24 {
            finding_type: AuditFindingTypeV24::UnscoredEvents,
            description: "Test".into(),
            severity: RiskLevelV22::Critical,
            event_count: 5,
            recommendation: "Fix".into(),
        });
        assert!(report.has_critical_findings());
        assert_eq!(report.findings.len(), 1);
    }

    #[test]
    fn test_event_risk_scoring_engine_v24() {
        let mut engine = EventRiskScoringEngineV24::new();
        let score = engine.score_event(&test_event(), Vec::new());
        assert!(score.risk_score > 30.0);
        assert_eq!(engine.total_scored(), 1);
        assert!(engine.get_score_for_event("evt-1").is_some());
    }

    #[test]
    fn test_retention_policy_manager_v24() {
        let mut manager = RetentionPolicyManagerV24::new();
        let policy = RetentionPolicyV24::new("security".into(), 365);
        manager.add_policy(policy);
        assert!(manager.get_policy_for_category("security").is_some());
        assert!(manager.events_to_archive("security", 91));
        assert!(!manager.events_to_archive("security", 89));
    }

    #[test]
    fn test_forensic_timeline_builder_v24() {
        let mut builder = ForensicTimelineBuilderV24::new();
        {
            let timeline = builder.get_or_create_timeline("user-1");
            timeline.add_entry(ForensicTimelineEntryV24 {
                timestamp: Utc::now(),
                event: test_event(),
                risk_score: None,
                correlation_ids: Vec::new(),
                risk_level: RiskLevelV22::Low,
                notes: None,
            });
        }
        assert_eq!(builder.actor_count(), 1);
        assert!(builder.get_timeline("user-1").is_some());
    }

    #[test]
    fn test_compliance_reporting_engine_v24() {
        let mut engine = ComplianceReportingEngineV24::new();
        let mut scoring = EventRiskScoringEngineV24::new();
        scoring.score_event(&test_event(), Vec::new());
        let policies = RetentionPolicyManagerV24::new();
        let report = engine.generate_report("fw-1", "SOC 2", &scoring, &policies);
        assert_eq!(report.risk_scores_counted, 1);
        assert!(engine.latest_report("fw-1").is_some());
    }

    #[test]
    fn test_mitigation_priority_v24_display() {
        assert_eq!(MitigationPriorityV24::Immediate.display_name(), "immediate");
        assert_eq!(MitigationPriorityV24::High.display_name(), "high");
    }

    #[test]
    fn test_audit_finding_type_v24_display() {
        assert_eq!(AuditFindingTypeV24::UnscoredEvents.display_name(), "un_scored_events");
        assert_eq!(AuditFindingTypeV24::RetentionViolation.display_name(), "retention_violation");
    }
}
