#![forbid(unsafe_code)]

use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnomalyType {
    UnusualAccessTime,
    UnusualLocation,
    ExcessiveFailedAttempts,
    PrivilegeEscalation,
    BulkDataAccess,
    SuspiciousPattern,
}

impl fmt::Display for AnomalyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnusualAccessTime => write!(f, "unusual_access_time"),
            Self::UnusualLocation => write!(f, "unusual_location"),
            Self::ExcessiveFailedAttempts => write!(f, "excessive_failed_attempts"),
            Self::PrivilegeEscalation => write!(f, "privilege_escalation"),
            Self::BulkDataAccess => write!(f, "bulk_data_access"),
            Self::SuspiciousPattern => write!(f, "suspicious_pattern"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

impl RiskLevel {
    pub fn from_score(score: u32) -> Self {
        match score {
            0..=25 => Self::Low,
            26..=50 => Self::Medium,
            51..=75 => Self::High,
            _ => Self::Critical,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntryV4 {
    pub id: String,
    pub event_type: String,
    pub resource_type: String,
    pub resource_id: String,
    pub actor_id: Option<String>,
    pub action: String,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    pub session_id: Option<String>,
    pub geo_location: Option<serde_json::Value>,
    pub risk_score: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetection {
    pub id: String,
    pub entry_id: String,
    pub anomaly_type: AnomalyType,
    pub confidence: f64,
    pub description: String,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicAnalysis {
    pub actor_id: String,
    pub time_range_start: DateTime<Utc>,
    pub time_range_end: DateTime<Utc>,
    pub total_events: u64,
    pub risk_events: u64,
    pub anomalies: Vec<AnomalyDetection>,
    pub risk_summary: HashMap<RiskLevel, u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditAnalytics {
    pub total_events: u64,
    pub events_by_type: HashMap<String, u64>,
    pub events_by_risk_level: HashMap<RiskLevel, u64>,
    pub avg_risk_score: f64,
    pub high_risk_events: u64,
}

#[derive(Debug, Clone)]
pub struct AuditTrailV4 {
    entries: Vec<AuditEntryV4>,
    anomalies: Vec<AnomalyDetection>,
}

impl Default for AuditTrailV4 {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditTrailV4 {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            anomalies: Vec::new(),
        }
    }

    pub fn append(&mut self, entry: AuditEntryV4) {
        self.entries.push(entry);
    }

    pub fn query_by_actor(&self, actor_id: &str) -> Vec<&AuditEntryV4> {
        self.entries
            .iter()
            .filter(|e| e.actor_id.as_deref() == Some(actor_id))
            .collect()
    }

    pub fn query_by_event_type(&self, event_type: &str) -> Vec<&AuditEntryV4> {
        self.entries
            .iter()
            .filter(|e| e.event_type == event_type)
            .collect()
    }

    pub fn query_by_time_range(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Vec<&AuditEntryV4> {
        self.entries
            .iter()
            .filter(|e| e.created_at >= from && e.created_at <= to)
            .collect()
    }

    pub fn query_high_risk(&self, threshold: u32) -> Vec<&AuditEntryV4> {
        self.entries
            .iter()
            .filter(|e| e.risk_score >= threshold)
            .collect()
    }

    pub fn compute_risk_score(
        event_type: &str,
        action: &str,
        details: &serde_json::Value,
    ) -> u32 {
        let mut score = 0u32;

        match event_type {
            "security" => score += 30,
            "auth" => score += 20,
            "data_access" => score += 25,
            "admin" => score += 35,
            "compliance" => score += 15,
            _ => score += 10,
        }

        match action {
            "delete" | "destroy" => score += 20,
            "modify" | "update" => score += 10,
            "create" => score += 5,
            "read" => score += 2,
            _ => score += 5,
        }

        if let Some(obj) = details.as_object() {
            if obj.contains_key("admin_action") {
                score += 15;
            }
            if obj.contains_key("bulk_operation") {
                score += 10;
            }
            if obj.contains_key("failed") {
                score += 5;
            }
        }

        score.min(100)
    }

    pub fn detect_anomalies(&mut self, entry: &AuditEntryV4) -> Vec<AnomalyDetection> {
        let mut detected = Vec::new();

        let hour = entry.created_at.time().hour();
        if !(6..=22).contains(&hour) {
            let anomaly = AnomalyDetection {
                id: format!("anomaly-{}", uuid::Uuid::new_v4()),
                entry_id: entry.id.clone(),
                anomaly_type: AnomalyType::UnusualAccessTime,
                confidence: 0.7,
                description: format!("Access at unusual hour: {}:00", hour),
                detected_at: Utc::now(),
            };
            detected.push(anomaly);
        }

        if entry.risk_score > 70 {
            let anomaly = AnomalyDetection {
                id: format!("anomaly-{}", uuid::Uuid::new_v4()),
                entry_id: entry.id.clone(),
                anomaly_type: AnomalyType::SuspiciousPattern,
                confidence: 0.8,
                description: format!("High risk score: {}", entry.risk_score),
                detected_at: Utc::now(),
            };
            detected.push(anomaly);
        }

        self.anomalies.extend(detected.clone());
        detected
    }

    pub fn generate_compliance_report(&self) -> String {
        let mut report = String::from("=== Audit Trail V4 Compliance Report ===\n\n");
        let analytics = self.compute_analytics();

        report.push_str(&format!("Total Events: {}\n", analytics.total_events));
        report.push_str(&format!("Avg Risk Score: {:.1}\n", analytics.avg_risk_score));
        report.push_str(&format!("High Risk Events: {}\n", analytics.high_risk_events));
        report.push('\n');

        report.push_str("Events by Type:\n");
        for (event_type, count) in &analytics.events_by_type {
            report.push_str(&format!("  {}: {}\n", event_type, count));
        }
        report.push('\n');

        report.push_str("Events by Risk Level:\n");
        for level in &[
            RiskLevel::Critical,
            RiskLevel::High,
            RiskLevel::Medium,
            RiskLevel::Low,
        ] {
            if let Some(count) = analytics.events_by_risk_level.get(level) {
                report.push_str(&format!("  {}: {}\n", level, count));
            }
        }

        report
    }

    pub fn perform_forensic_analysis(
        &self,
        actor_id: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> ForensicAnalysis {
        let events: Vec<&AuditEntryV4> = self
            .entries
            .iter()
            .filter(|e| {
                e.actor_id.as_deref() == Some(actor_id)
                    && e.created_at >= from
                    && e.created_at <= to
            })
            .collect();

        let total_events = events.len() as u64;
        let risk_events = events.iter().filter(|e| e.risk_score > 50).count() as u64;

        let anomalies: Vec<AnomalyDetection> = self
            .anomalies
            .iter()
            .filter(|a| {
                events.iter().any(|e| e.id == a.entry_id)
            })
            .cloned()
            .collect();

        let mut risk_summary: HashMap<RiskLevel, u64> = HashMap::new();
        for event in &events {
            let level = RiskLevel::from_score(event.risk_score);
            *risk_summary.entry(level).or_default() += 1;
        }

        ForensicAnalysis {
            actor_id: actor_id.to_string(),
            time_range_start: from,
            time_range_end: to,
            total_events,
            risk_events,
            anomalies,
            risk_summary,
        }
    }

    pub fn compute_analytics(&self) -> AuditAnalytics {
        let total = self.entries.len() as u64;

        let mut events_by_type: HashMap<String, u64> = HashMap::new();
        let mut events_by_risk_level: HashMap<RiskLevel, u64> = HashMap::new();
        let mut total_risk_score = 0u64;
        let mut high_risk = 0u64;

        for entry in &self.entries {
            *events_by_type
                .entry(entry.event_type.clone())
                .or_default() += 1;

            let level = RiskLevel::from_score(entry.risk_score);
            *events_by_risk_level.entry(level).or_default() += 1;

            total_risk_score += entry.risk_score as u64;
            if entry.risk_score > 70 {
                high_risk += 1;
            }
        }

        let avg_risk_score = if total > 0 {
            total_risk_score as f64 / total as f64
        } else {
            0.0
        };

        AuditAnalytics {
            total_events: total,
            events_by_type,
            events_by_risk_level,
            avg_risk_score,
            high_risk_events: high_risk,
        }
    }

    pub fn entries(&self) -> &[AuditEntryV4] {
        &self.entries
    }

    pub fn anomalies(&self) -> &[AnomalyDetection] {
        &self.anomalies
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(id: &str, event_type: &str, action: &str, risk_score: u32) -> AuditEntryV4 {
        AuditEntryV4 {
            id: id.to_string(),
            event_type: event_type.to_string(),
            resource_type: "repo".to_string(),
            resource_id: "repo-1".to_string(),
            actor_id: Some("user-1".to_string()),
            action: action.to_string(),
            details: serde_json::json!({}),
            ip_address: Some("10.0.0.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            request_id: Some("req-1".to_string()),
            session_id: Some("sess-1".to_string()),
            geo_location: None,
            risk_score,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_append_entry() {
        let mut trail = AuditTrailV4::new();
        trail.append(sample_entry("e1", "auth", "login", 10));
        assert_eq!(trail.len(), 1);
    }

    #[test]
    fn test_query_by_actor() {
        let mut trail = AuditTrailV4::new();
        trail.append(sample_entry("e1", "auth", "login", 10));
        trail.append(sample_entry("e2", "auth", "login", 10));
        trail.entries.last_mut().unwrap().actor_id = Some("user-2".to_string());
        let results = trail.query_by_actor("user-1");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_by_event_type() {
        let mut trail = AuditTrailV4::new();
        trail.append(sample_entry("e1", "auth", "login", 10));
        trail.append(sample_entry("e2", "security", "scan", 30));
        let results = trail.query_by_event_type("auth");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_high_risk() {
        let mut trail = AuditTrailV4::new();
        trail.append(sample_entry("e1", "auth", "login", 10));
        trail.append(sample_entry("e2", "security", "delete", 80));
        trail.append(sample_entry("e3", "admin", "config", 90));
        let results = trail.query_high_risk(70);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_compute_risk_score() {
        let score = AuditTrailV4::compute_risk_score("security", "delete", &serde_json::json!({}));
        assert!(score >= 50);
        let score = AuditTrailV4::compute_risk_score("auth", "read", &serde_json::json!({}));
        assert!(score < 30);
    }

    #[test]
    fn test_detect_anomalies() {
        let mut trail = AuditTrailV4::new();
        let mut entry = sample_entry("e1", "security", "delete", 80);
        let naive_time = chrono::NaiveDateTime::new(
            entry.created_at.date_naive(),
            chrono::NaiveTime::from_hms_opt(3, 0, 0).unwrap(),
        );
        entry.created_at = DateTime::from_naive_utc_and_offset(naive_time, Utc);
        let anomalies = trail.detect_anomalies(&entry);
        assert!(!anomalies.is_empty());
    }

    #[test]
    fn test_compute_analytics() {
        let mut trail = AuditTrailV4::new();
        trail.append(sample_entry("e1", "auth", "login", 10));
        trail.append(sample_entry("e2", "security", "scan", 80));
        let analytics = trail.compute_analytics();
        assert_eq!(analytics.total_events, 2);
        assert_eq!(analytics.high_risk_events, 1);
    }

    #[test]
    fn test_generate_compliance_report() {
        let mut trail = AuditTrailV4::new();
        trail.append(sample_entry("e1", "auth", "login", 10));
        let report = trail.generate_compliance_report();
        assert!(report.contains("Audit Trail V4 Compliance Report"));
        assert!(report.contains("Total Events: 1"));
    }

    #[test]
    fn test_perform_forensic_analysis() {
        let mut trail = AuditTrailV4::new();
        trail.append(sample_entry("e1", "auth", "login", 10));
        trail.append(sample_entry("e2", "security", "delete", 80));
        let from = Utc::now() - chrono::Duration::hours(1);
        let to = Utc::now() + chrono::Duration::hours(1);
        let analysis = trail.perform_forensic_analysis("user-1", from, to);
        assert_eq!(analysis.total_events, 2);
        assert_eq!(analysis.risk_events, 1);
    }

    #[test]
    fn test_risk_level_from_score() {
        assert_eq!(RiskLevel::from_score(10), RiskLevel::Low);
        assert_eq!(RiskLevel::from_score(30), RiskLevel::Medium);
        assert_eq!(RiskLevel::from_score(60), RiskLevel::High);
        assert_eq!(RiskLevel::from_score(90), RiskLevel::Critical);
    }

    #[test]
    fn test_empty_trail() {
        let trail = AuditTrailV4::new();
        assert!(trail.is_empty());
        assert_eq!(trail.len(), 0);
        let analytics = trail.compute_analytics();
        assert_eq!(analytics.total_events, 0);
    }
}
