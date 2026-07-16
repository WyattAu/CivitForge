#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::audit_trail_v22::{
    AnomalyDetectionResultV22, AnomalyTypeV22, AuditTrailEventV22, AuditTrailRecorderV22,
    RiskLevelV22,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventCategoryV23 {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: AuditCategorySeverityV23,
    pub retention_days: u32,
    pub created_at: DateTime<Utc>,
}

impl AuditEventCategoryV23 {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            severity: AuditCategorySeverityV23::Info,
            retention_days: 365,
            created_at: Utc::now(),
        }
    }

    pub fn with_severity(mut self, severity: AuditCategorySeverityV23) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_retention(mut self, days: u32) -> Self {
        self.retention_days = days;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditCategorySeverityV23 {
    Info,
    Warning,
    Critical,
}

impl AuditCategorySeverityV23 {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Warning => "Warning",
            Self::Critical => "Critical",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventCorrelationV23 {
    pub id: String,
    pub event_id: String,
    pub correlated_event_id: String,
    pub correlation_type: CorrelationTypeV23,
    pub created_at: DateTime<Utc>,
}

impl AuditEventCorrelationV23 {
    pub fn new(event_id: String, correlated_event_id: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_id,
            correlated_event_id,
            correlation_type: CorrelationTypeV23::Related,
            created_at: Utc::now(),
        }
    }

    pub fn with_type(mut self, correlation_type: CorrelationTypeV23) -> Self {
        self.correlation_type = correlation_type;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrelationTypeV23 {
    Related,
    Causal,
    Temporal,
    Actor,
    Resource,
}

impl CorrelationTypeV23 {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Related => "Related",
            Self::Causal => "Causal",
            Self::Temporal => "Temporal",
            Self::Actor => "Actor",
            Self::Resource => "Resource",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionRuleV23 {
    pub id: String,
    pub name: String,
    pub anomaly_type: AnomalyTypeV22,
    pub threshold: f64,
    pub window_minutes: u32,
    pub enabled: bool,
    pub severity: RiskLevelV22,
}

impl AnomalyDetectionRuleV23 {
    pub fn new(
        name: String,
        anomaly_type: AnomalyTypeV22,
        threshold: f64,
        window_minutes: u32,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            anomaly_type,
            threshold,
            window_minutes,
            enabled: true,
            severity: RiskLevelV22::Medium,
        }
    }

    pub fn with_severity(mut self, severity: RiskLevelV22) -> Self {
        self.severity = severity;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicTimelineEntryV23 {
    pub timestamp: DateTime<Utc>,
    pub event: AuditTrailEventV22,
    pub correlation_ids: Vec<String>,
    pub risk_level: RiskLevelV22,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicAnalysisResultV23 {
    pub actor_id: String,
    pub analysis_period_start: DateTime<Utc>,
    pub analysis_period_end: DateTime<Utc>,
    pub timeline: Vec<ForensicTimelineEntryV23>,
    pub anomalies_detected: Vec<AnomalyDetectionResultV22>,
    pub risk_summary: ForensicRiskSummaryV23,
    pub recommendations: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicRiskSummaryV23 {
    pub total_events: u32,
    pub high_risk_events: u32,
    pub medium_risk_events: u32,
    pub low_risk_events: u32,
    pub average_risk_score: f64,
    pub max_risk_score: u32,
    pub risk_trend: RiskTrendV23,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskTrendV23 {
    Increasing,
    Decreasing,
    Stable,
}

impl RiskTrendV23 {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Increasing => "Increasing",
            Self::Decreasing => "Decreasing",
            Self::Stable => "Stable",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditEventCategoryManagerV23 {
    categories: Vec<AuditEventCategoryV23>,
}

impl AuditEventCategoryManagerV23 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_category(&mut self, category: AuditEventCategoryV23) {
        self.categories.push(category);
    }

    pub fn get_category(&self, id: &str) -> Option<&AuditEventCategoryV23> {
        self.categories.iter().find(|c| c.id == id)
    }

    pub fn get_category_by_name(&self, name: &str) -> Option<&AuditEventCategoryV23> {
        self.categories.iter().find(|c| c.name == name)
    }

    pub fn list_categories(&self) -> &[AuditEventCategoryV23] {
        &self.categories
    }

    pub fn categories_by_severity(
        &self,
        severity: AuditCategorySeverityV23,
    ) -> Vec<&AuditEventCategoryV23> {
        self.categories
            .iter()
            .filter(|c| c.severity == severity)
            .collect()
    }

    pub fn disable_category(&mut self, id: &str) -> Result<(), String> {
        // Categories don't have an enabled field in schema, but we can remove them
        self.categories.retain(|c| c.id != id);
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventCorrelationEngineV23 {
    correlations: Vec<AuditEventCorrelationV23>,
    by_event: HashMap<String, Vec<usize>>,
    by_correlated: HashMap<String, Vec<usize>>,
}

impl EventCorrelationEngineV23 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_correlation(&mut self, correlation: AuditEventCorrelationV23) {
        let idx = self.correlations.len();
        self.by_event
            .entry(correlation.event_id.clone())
            .or_default()
            .push(idx);
        self.by_correlated
            .entry(correlation.correlated_event_id.clone())
            .or_default()
            .push(idx);
        self.correlations.push(correlation);
    }

    pub fn get_correlations_for_event(
        &self,
        event_id: &str,
    ) -> Vec<&AuditEventCorrelationV23> {
        self.by_event
            .get(event_id)
            .map(|indices| indices.iter().map(|&idx| &self.correlations[idx]).collect())
            .unwrap_or_default()
    }

    pub fn get_correlated_events(&self, event_id: &str) -> Vec<&AuditEventCorrelationV23> {
        self.by_correlated
            .get(event_id)
            .map(|indices| indices.iter().map(|&idx| &self.correlations[idx]).collect())
            .unwrap_or_default()
    }

    pub fn total_correlations(&self) -> usize {
        self.correlations.len()
    }

    pub fn remove_correlations_for_event(&mut self, event_id: &str) {
        self.correlations
            .retain(|c| c.event_id != event_id && c.correlated_event_id != event_id);
        // Rebuild indices
        self.by_event.clear();
        self.by_correlated.clear();
        for (idx, c) in self.correlations.iter().enumerate() {
            self.by_event
                .entry(c.event_id.clone())
                .or_default()
                .push(idx);
            self.by_correlated
                .entry(c.correlated_event_id.clone())
                .or_default()
                .push(idx);
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnomalyDetectionEngineV23 {
    rules: Vec<AnomalyDetectionRuleV23>,
}

impl AnomalyDetectionEngineV23 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_rule(&mut self, rule: AnomalyDetectionRuleV23) {
        self.rules.push(rule);
    }

    pub fn get_enabled_rules(&self) -> Vec<&AnomalyDetectionRuleV23> {
        self.rules.iter().filter(|r| r.enabled).collect()
    }

    pub fn detect(
        &self,
        recorder: &AuditTrailRecorderV22,
    ) -> Vec<AnomalyDetectionResultV22> {
        let mut anomalies = Vec::new();
        let base_anomalies = recorder.detect_anomalies();
        anomalies.extend(base_anomalies);

        let enabled_rules = self.get_enabled_rules();
        if enabled_rules.is_empty() {
            return anomalies;
        }

        let events = recorder.search(&crate::audit_trail_v22::AuditTrailQueryV22::default());
        let now = Utc::now();

        for rule in &enabled_rules {
            let window_start =
                now - chrono::Duration::minutes(rule.window_minutes as i64);
            let recent_events: Vec<_> = events
                .iter()
                .filter(|e| e.created_at >= window_start)
                .collect();

            let rate = recent_events.len() as f64 / rule.window_minutes as f64;
            if rate > rule.threshold {
                anomalies.push(AnomalyDetectionResultV22 {
                    anomaly_type: rule.anomaly_type,
                    actor_id: None,
                    description: format!(
                        "Rule '{}': rate {:.2} events/min exceeds threshold {:.2}",
                        rule.name, rate, rule.threshold
                    ),
                    risk_level: rule.severity,
                    detected_at: now,
                });
            }
        }

        anomalies
    }

    pub fn list_rules(&self) -> &[AnomalyDetectionRuleV23] {
        &self.rules
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForensicAnalysisEngineV23 {
    timeline: Vec<ForensicTimelineEntryV23>,
}

impl ForensicAnalysisEngineV23 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build_timeline(
        &mut self,
        recorder: &AuditTrailRecorderV22,
        actor_id: &str,
        since: DateTime<Utc>,
        until: DateTime<Utc>,
        correlation_engine: &EventCorrelationEngineV23,
    ) {
        self.timeline.clear();
        let events = recorder.search(&crate::audit_trail_v22::AuditTrailQueryV22 {
            actor_id: Some(actor_id.into()),
            since: Some(since),
            until: Some(until),
            limit: 10000,
            ..Default::default()
        });

        for event in events {
            let correlation_ids = correlation_engine
                .get_correlations_for_event(&event.id)
                .iter()
                .map(|c| c.correlated_event_id.clone())
                .collect();

            let risk_level = if event.risk_score >= 75 {
                RiskLevelV22::Critical
            } else if event.risk_score >= 50 {
                RiskLevelV22::High
            } else if event.risk_score >= 25 {
                RiskLevelV22::Medium
            } else {
                RiskLevelV22::Low
            };

            self.timeline.push(ForensicTimelineEntryV23 {
                timestamp: event.created_at,
                event,
                correlation_ids,
                risk_level,
                notes: None,
            });
        }

        self.timeline.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    }

    pub fn analyze(
        &self,
        actor_id: &str,
        anomaly_engine: &AnomalyDetectionEngineV23,
        recorder: &AuditTrailRecorderV22,
    ) -> ForensicAnalysisResultV23 {
        let total = self.timeline.len() as u32;
        let high_risk = self
            .timeline
            .iter()
            .filter(|e| e.risk_level == RiskLevelV22::Critical || e.risk_level == RiskLevelV22::High)
            .count() as u32;
        let medium_risk = self
            .timeline
            .iter()
            .filter(|e| e.risk_level == RiskLevelV22::Medium)
            .count() as u32;
        let low_risk = self
            .timeline
            .iter()
            .filter(|e| e.risk_level == RiskLevelV22::Low)
            .count() as u32;

        let avg_risk = if total == 0 {
            0.0
        } else {
            self.timeline
                .iter()
                .map(|e| e.event.risk_score as f64)
                .sum::<f64>()
                / total as f64
        };

        let max_risk = self
            .timeline
            .iter()
            .map(|e| e.event.risk_score)
            .max()
            .unwrap_or(0);

        let risk_trend = if self.timeline.len() >= 2 {
            let mid = self.timeline.len() / 2;
            let first_half_avg: f64 = self.timeline[..mid]
                .iter()
                .map(|e| e.event.risk_score as f64)
                .sum::<f64>()
                / mid as f64;
            let second_half_avg: f64 = self.timeline[mid..]
                .iter()
                .map(|e| e.event.risk_score as f64)
                .sum::<f64>()
                / (self.timeline.len() - mid) as f64;
            if second_half_avg > first_half_avg * 1.1 {
                RiskTrendV23::Increasing
            } else if second_half_avg < first_half_avg * 0.9 {
                RiskTrendV23::Decreasing
            } else {
                RiskTrendV23::Stable
            }
        } else {
            RiskTrendV23::Stable
        };

        let anomalies = anomaly_engine.detect(recorder);
        let mut recommendations = Vec::new();

        if high_risk > 0 {
            recommendations.push(format!(
                "Review {} high-risk events for potential security incidents",
                high_risk
            ));
        }
        if risk_trend == RiskTrendV23::Increasing {
            recommendations
                .push("Risk trend is increasing - investigate recent changes".into());
        }
        if anomalies
            .iter()
            .any(|a| a.risk_level == RiskLevelV22::Critical)
        {
            recommendations.push("Critical anomalies detected - immediate review required".into());
        }

        let period_start = self
            .timeline
            .first()
            .map(|e| e.timestamp)
            .unwrap_or_else(Utc::now);
        let period_end = self
            .timeline
            .last()
            .map(|e| e.timestamp)
            .unwrap_or_else(Utc::now);

        ForensicAnalysisResultV23 {
            actor_id: actor_id.into(),
            analysis_period_start: period_start,
            analysis_period_end: period_end,
            timeline: self.timeline.clone(),
            anomalies_detected: anomalies,
            risk_summary: ForensicRiskSummaryV23 {
                total_events: total,
                high_risk_events: high_risk,
                medium_risk_events: medium_risk,
                low_risk_events: low_risk,
                average_risk_score: avg_risk,
                max_risk_score: max_risk,
                risk_trend,
            },
            recommendations,
            generated_at: Utc::now(),
        }
    }

    pub fn timeline(&self) -> &[ForensicTimelineEntryV23] {
        &self.timeline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_category_v23_new() {
        let cat = AuditEventCategoryV23::new("security".into(), "Security events".into());
        assert_eq!(cat.name, "security");
        assert_eq!(cat.severity, AuditCategorySeverityV23::Info);
        assert_eq!(cat.retention_days, 365);
    }

    #[test]
    fn test_audit_event_category_v23_with_fields() {
        let cat = AuditEventCategoryV23::new("test".into(), "Desc".into())
            .with_severity(AuditCategorySeverityV23::Critical)
            .with_retention(730);
        assert_eq!(cat.severity, AuditCategorySeverityV23::Critical);
        assert_eq!(cat.retention_days, 730);
    }

    #[test]
    fn test_category_severity_display() {
        assert_eq!(AuditCategorySeverityV23::Info.display_name(), "Info");
        assert_eq!(
            AuditCategorySeverityV23::Warning.display_name(),
            "Warning"
        );
        assert_eq!(
            AuditCategorySeverityV23::Critical.display_name(),
            "Critical"
        );
    }

    #[test]
    fn test_correlation_type_display() {
        assert_eq!(CorrelationTypeV23::Related.display_name(), "Related");
        assert_eq!(CorrelationTypeV23::Causal.display_name(), "Causal");
        assert_eq!(CorrelationTypeV23::Temporal.display_name(), "Temporal");
    }

    #[test]
    fn test_event_correlation_v23() {
        let corr = AuditEventCorrelationV23::new("e1".into(), "e2".into());
        assert_eq!(corr.event_id, "e1");
        assert_eq!(corr.correlated_event_id, "e2");
        assert_eq!(corr.correlation_type, CorrelationTypeV23::Related);
    }

    #[test]
    fn test_anomaly_detection_rule_v23() {
        let rule = AnomalyDetectionRuleV23::new(
            "High rate".into(),
            AnomalyTypeV22::HighActivity,
            10.0,
            60,
        );
        assert_eq!(rule.name, "High rate");
        assert!(rule.enabled);
        assert_eq!(rule.threshold, 10.0);
    }

    #[test]
    fn test_risk_trend_display() {
        assert_eq!(RiskTrendV23::Increasing.display_name(), "Increasing");
        assert_eq!(RiskTrendV23::Decreasing.display_name(), "Decreasing");
        assert_eq!(RiskTrendV23::Stable.display_name(), "Stable");
    }

    #[test]
    fn test_audit_event_category_manager_v23() {
        let mut manager = AuditEventCategoryManagerV23::new();
        let cat = AuditEventCategoryV23::new("test".into(), "Desc".into());
        manager.add_category(cat);
        assert_eq!(manager.list_categories().len(), 1);
    }

    #[test]
    fn test_audit_event_category_manager_v23_by_severity() {
        let mut manager = AuditEventCategoryManagerV23::new();
        manager.add_category(
            AuditEventCategoryV23::new("a".into(), "".into())
                .with_severity(AuditCategorySeverityV23::Critical),
        );
        manager.add_category(
            AuditEventCategoryV23::new("b".into(), "".into())
                .with_severity(AuditCategorySeverityV23::Info),
        );
        assert_eq!(
            manager
                .categories_by_severity(AuditCategorySeverityV23::Critical)
                .len(),
            1
        );
    }

    #[test]
    fn test_event_correlation_engine_v23() {
        let mut engine = EventCorrelationEngineV23::new();
        let corr = AuditEventCorrelationV23::new("e1".into(), "e2".into());
        engine.add_correlation(corr);
        assert_eq!(engine.total_correlations(), 1);
        assert_eq!(engine.get_correlations_for_event("e1").len(), 1);
    }

    #[test]
    fn test_event_correlation_engine_v23_remove() {
        let mut engine = EventCorrelationEngineV23::new();
        engine.add_correlation(AuditEventCorrelationV23::new("e1".into(), "e2".into()));
        engine.add_correlation(AuditEventCorrelationV23::new("e1".into(), "e3".into()));
        engine.remove_correlations_for_event("e1");
        assert_eq!(engine.total_correlations(), 0);
    }

    #[test]
    fn test_anomaly_detection_engine_v23() {
        let mut engine = AnomalyDetectionEngineV23::new();
        engine.add_rule(AnomalyDetectionRuleV23::new(
            "Test".into(),
            AnomalyTypeV22::HighActivity,
            10.0,
            60,
        ));
        assert_eq!(engine.get_enabled_rules().len(), 1);
    }

    #[test]
    fn test_forensic_analysis_engine_v23_empty() {
        let engine = ForensicAnalysisEngineV23::new();
        let anomaly_engine = AnomalyDetectionEngineV23::new();
        let recorder = AuditTrailRecorderV22::new();
        let result = engine.analyze("user-1", &anomaly_engine, &recorder);
        assert_eq!(result.actor_id, "user-1");
        assert_eq!(result.risk_summary.total_events, 0);
    }
}
