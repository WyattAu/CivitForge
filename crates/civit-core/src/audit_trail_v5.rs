#![forbid(unsafe_code)]

use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventTypeV6 {
    Auth,
    Security,
    DataAccess,
    Admin,
    Compliance,
    System,
    User,
    Repository,
}

impl AuditEventTypeV6 {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auth => "auth",
            Self::Security => "security",
            Self::DataAccess => "data_access",
            Self::Admin => "admin",
            Self::Compliance => "compliance",
            Self::System => "system",
            Self::User => "user",
            Self::Repository => "repository",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatusV6 {
    Compliant,
    NonCompliant,
    Partial,
    UnderReview,
    NotApplicable,
}

impl ComplianceStatusV6 {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Compliant => "compliant",
            Self::NonCompliant => "non_compliant",
            Self::Partial => "partial",
            Self::UnderReview => "under_review",
            Self::NotApplicable => "not_applicable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevelV6 {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevelV6 {
    pub fn from_score(score: u32) -> Self {
        match score {
            0..=25 => Self::Low,
            26..=50 => Self::Medium,
            51..=75 => Self::High,
            _ => Self::Critical,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyTypeV6 {
    UnusualAccessTime,
    UnusualLocation,
    ExcessiveFailedAttempts,
    PrivilegeEscalation,
    BulkDataAccess,
    SuspiciousPattern,
    ImpossibleTravel,
    NewDeviceAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailEventV6 {
    pub id: String,
    pub event_type: AuditEventTypeV6,
    pub resource_type: String,
    pub resource_id: String,
    pub actor_id: Option<String>,
    pub action: String,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    pub session_id: Option<String>,
    pub geo_location: Option<GeoLocationV6>,
    pub risk_score: u32,
    pub compliance_status: ComplianceStatusV6,
    pub created_at: DateTime<Utc>,
}

impl AuditTrailEventV6 {
    pub fn new(
        event_type: AuditEventTypeV6,
        resource_type: &str,
        resource_id: &str,
        action: &str,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            actor_id: None,
            action: action.into(),
            details: serde_json::Value::Object(serde_json::Map::new()),
            ip_address: None,
            user_agent: None,
            request_id: None,
            session_id: None,
            geo_location: None,
            risk_score: 0,
            compliance_status: ComplianceStatusV6::Compliant,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeoLocationV6 {
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
}

impl GeoLocationV6 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_country(mut self, country: &str) -> Self {
        self.country = Some(country.into());
        self
    }

    pub fn with_region(mut self, region: &str) -> Self {
        self.region = Some(region.into());
        self
    }

    pub fn with_city(mut self, city: &str) -> Self {
        self.city = Some(city.into());
        self
    }

    pub fn with_coordinates(mut self, lat: f64, lon: f64) -> Self {
        self.latitude = Some(lat);
        self.longitude = Some(lon);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionV6 {
    pub id: String,
    pub entry_id: String,
    pub anomaly_type: AnomalyTypeV6,
    pub confidence: f64,
    pub description: String,
    pub risk_contribution: u32,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicAnalysisV6 {
    pub actor_id: String,
    pub time_range_start: DateTime<Utc>,
    pub time_range_end: DateTime<Utc>,
    pub total_events: u64,
    pub risk_events: u64,
    pub anomalies: Vec<AnomalyDetectionV6>,
    pub risk_summary: HashMap<RiskLevelV6, u64>,
    pub compliance_summary: HashMap<ComplianceStatusV6, u64>,
    pub timeline: Vec<AuditTrailEventV6>,
    pub geo_summary: HashMap<String, u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditAnalyticsV6 {
    pub total_events: u64,
    pub events_by_type: HashMap<String, u64>,
    pub events_by_risk_level: HashMap<RiskLevelV6, u64>,
    pub events_by_compliance_status: HashMap<ComplianceStatusV6, u64>,
    pub avg_risk_score: f64,
    pub high_risk_events: u64,
    pub compliance_rate: f64,
}

#[derive(Debug, Clone)]
pub struct AuditTrailV6 {
    entries: Vec<AuditTrailEventV6>,
    anomalies: Vec<AnomalyDetectionV6>,
}

impl Default for AuditTrailV6 {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditTrailV6 {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            anomalies: Vec::new(),
        }
    }

    pub fn append(&mut self, entry: AuditTrailEventV6) {
        self.entries.push(entry);
    }

    pub fn query_by_actor(&self, actor_id: &str) -> Vec<&AuditTrailEventV6> {
        self.entries
            .iter()
            .filter(|e| e.actor_id.as_deref() == Some(actor_id))
            .collect()
    }

    pub fn query_by_event_type(&self, event_type: &AuditEventTypeV6) -> Vec<&AuditTrailEventV6> {
        self.entries
            .iter()
            .filter(|e| &e.event_type == event_type)
            .collect()
    }

    pub fn query_by_time_range(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Vec<&AuditTrailEventV6> {
        self.entries
            .iter()
            .filter(|e| e.created_at >= from && e.created_at <= to)
            .collect()
    }

    pub fn query_high_risk(&self, threshold: u32) -> Vec<&AuditTrailEventV6> {
        self.entries
            .iter()
            .filter(|e| e.risk_score >= threshold)
            .collect()
    }

    pub fn query_non_compliant(&self) -> Vec<&AuditTrailEventV6> {
        self.entries
            .iter()
            .filter(|e| e.compliance_status == ComplianceStatusV6::NonCompliant)
            .collect()
    }

    pub fn compute_risk_score(
        event_type: &AuditEventTypeV6,
        action: &str,
        details: &serde_json::Value,
    ) -> u32 {
        let mut score = 0u32;

        match event_type {
            AuditEventTypeV6::Security => score += 30,
            AuditEventTypeV6::Auth => score += 20,
            AuditEventTypeV6::DataAccess => score += 25,
            AuditEventTypeV6::Admin => score += 35,
            AuditEventTypeV6::Compliance => score += 15,
            AuditEventTypeV6::System => score += 10,
            AuditEventTypeV6::User => score += 10,
            AuditEventTypeV6::Repository => score += 12,
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

    pub fn detect_anomalies(&mut self, entry: &AuditTrailEventV6) -> Vec<AnomalyDetectionV6> {
        let mut detected = Vec::new();

        let hour = entry.created_at.time().hour();
        if hour < 6 || hour > 22 {
            let anomaly = AnomalyDetectionV6 {
                id: format!("anomaly-{}", uuid::Uuid::new_v4()),
                entry_id: entry.id.clone(),
                anomaly_type: AnomalyTypeV6::UnusualAccessTime,
                confidence: 0.7,
                description: format!("Access at unusual hour: {}:00", hour),
                risk_contribution: 15,
                detected_at: Utc::now(),
            };
            detected.push(anomaly);
        }

        if entry.risk_score > 70 {
            let anomaly = AnomalyDetectionV6 {
                id: format!("anomaly-{}", uuid::Uuid::new_v4()),
                entry_id: entry.id.clone(),
                anomaly_type: AnomalyTypeV6::SuspiciousPattern,
                confidence: 0.8,
                description: format!("High risk score: {}", entry.risk_score),
                risk_contribution: 20,
                detected_at: Utc::now(),
            };
            detected.push(anomaly);
        }

        if entry.compliance_status == ComplianceStatusV6::NonCompliant {
            let anomaly = AnomalyDetectionV6 {
                id: format!("anomaly-{}", uuid::Uuid::new_v4()),
                entry_id: entry.id.clone(),
                anomaly_type: AnomalyTypeV6::PrivilegeEscalation,
                confidence: 0.6,
                description: "Non-compliant action detected".into(),
                risk_contribution: 25,
                detected_at: Utc::now(),
            };
            detected.push(anomaly);
        }

        self.anomalies.extend(detected.clone());
        detected
    }

    pub fn generate_compliance_report(&self) -> String {
        let mut report = String::from("=== Audit Trail V6 Compliance Report ===\n\n");
        let analytics = self.compute_analytics();

        report.push_str(&format!("Total Events: {}\n", analytics.total_events));
        report.push_str(&format!("Avg Risk Score: {:.1}\n", analytics.avg_risk_score));
        report.push_str(&format!("High Risk Events: {}\n", analytics.high_risk_events));
        report.push_str(&format!(
            "Compliance Rate: {:.1}%\n",
            analytics.compliance_rate
        ));
        report.push('\n');

        report.push_str("Events by Type:\n");
        for (event_type, count) in &analytics.events_by_type {
            report.push_str(&format!("  {}: {}\n", event_type, count));
        }
        report.push('\n');

        report.push_str("Events by Risk Level:\n");
        for level in &[
            RiskLevelV6::Critical,
            RiskLevelV6::High,
            RiskLevelV6::Medium,
            RiskLevelV6::Low,
        ] {
            if let Some(count) = analytics.events_by_risk_level.get(level) {
                report.push_str(&format!("  {:?}: {}\n", level, count));
            }
        }
        report.push('\n');

        report.push_str("Compliance Status:\n");
        for (status, count) in &analytics.events_by_compliance_status {
            report.push_str(&format!("  {:?}: {}\n", status, count));
        }

        report
    }

    pub fn perform_forensic_analysis(
        &self,
        actor_id: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> ForensicAnalysisV6 {
        let events: Vec<&AuditTrailEventV6> = self
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

        let anomalies: Vec<AnomalyDetectionV6> = self
            .anomalies
            .iter()
            .filter(|a| events.iter().any(|e| e.id == a.entry_id))
            .cloned()
            .collect();

        let mut risk_summary: HashMap<RiskLevelV6, u64> = HashMap::new();
        let mut compliance_summary: HashMap<ComplianceStatusV6, u64> = HashMap::new();
        let mut geo_summary: HashMap<String, u64> = HashMap::new();

        for event in &events {
            let level = RiskLevelV6::from_score(event.risk_score);
            *risk_summary.entry(level).or_default() += 1;

            *compliance_summary
                .entry(event.compliance_status.clone())
                .or_default() += 1;

            if let Some(ref geo) = event.geo_location {
                if let Some(ref country) = geo.country {
                    *geo_summary.entry(country.clone()).or_default() += 1;
                }
            }
        }

        let timeline: Vec<AuditTrailEventV6> = events.into_iter().cloned().collect();

        ForensicAnalysisV6 {
            actor_id: actor_id.to_string(),
            time_range_start: from,
            time_range_end: to,
            total_events,
            risk_events,
            anomalies,
            risk_summary,
            compliance_summary,
            timeline,
            geo_summary,
        }
    }

    pub fn compute_analytics(&self) -> AuditAnalyticsV6 {
        let total = self.entries.len() as u64;

        let mut events_by_type: HashMap<String, u64> = HashMap::new();
        let mut events_by_risk_level: HashMap<RiskLevelV6, u64> = HashMap::new();
        let mut events_by_compliance_status: HashMap<ComplianceStatusV6, u64> = HashMap::new();
        let mut total_risk_score = 0u64;
        let mut high_risk = 0u64;
        let mut compliant_count = 0u64;

        for entry in &self.entries {
            *events_by_type
                .entry(entry.event_type.as_str().to_string())
                .or_default() += 1;

            let level = RiskLevelV6::from_score(entry.risk_score);
            *events_by_risk_level.entry(level).or_default() += 1;

            *events_by_compliance_status
                .entry(entry.compliance_status.clone())
                .or_default() += 1;

            total_risk_score += entry.risk_score as u64;
            if entry.risk_score > 70 {
                high_risk += 1;
            }
            if entry.compliance_status == ComplianceStatusV6::Compliant {
                compliant_count += 1;
            }
        }

        let avg_risk_score = if total > 0 {
            total_risk_score as f64 / total as f64
        } else {
            0.0
        };

        let compliance_rate = if total > 0 {
            (compliant_count as f64 / total as f64) * 100.0
        } else {
            100.0
        };

        AuditAnalyticsV6 {
            total_events: total,
            events_by_type,
            events_by_risk_level,
            events_by_compliance_status,
            avg_risk_score,
            high_risk_events: high_risk,
            compliance_rate,
        }
    }

    pub fn entries(&self) -> &[AuditTrailEventV6] {
        &self.entries
    }

    pub fn anomalies(&self) -> &[AnomalyDetectionV6] {
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

    fn sample_entry(
        id: &str,
        event_type: AuditEventTypeV6,
        action: &str,
        risk_score: u32,
    ) -> AuditTrailEventV6 {
        AuditTrailEventV6 {
            id: id.to_string(),
            event_type,
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
            compliance_status: ComplianceStatusV6::Compliant,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_append_entry() {
        let mut trail = AuditTrailV6::new();
        trail.append(sample_entry(
            "e1",
            AuditEventTypeV6::Auth,
            "login",
            10,
        ));
        assert_eq!(trail.len(), 1);
    }

    #[test]
    fn test_query_by_actor() {
        let mut trail = AuditTrailV6::new();
        trail.append(sample_entry(
            "e1",
            AuditEventTypeV6::Auth,
            "login",
            10,
        ));
        trail.append(sample_entry(
            "e2",
            AuditEventTypeV6::Auth,
            "login",
            10,
        ));
        trail.entries.last_mut().unwrap().actor_id = Some("user-2".to_string());
        let results = trail.query_by_actor("user-1");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_by_event_type() {
        let mut trail = AuditTrailV6::new();
        trail.append(sample_entry(
            "e1",
            AuditEventTypeV6::Auth,
            "login",
            10,
        ));
        trail.append(sample_entry(
            "e2",
            AuditEventTypeV6::Security,
            "scan",
            30,
        ));
        let results = trail.query_by_event_type(&AuditEventTypeV6::Auth);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_high_risk() {
        let mut trail = AuditTrailV6::new();
        trail.append(sample_entry(
            "e1",
            AuditEventTypeV6::Auth,
            "login",
            10,
        ));
        trail.append(sample_entry(
            "e2",
            AuditEventTypeV6::Security,
            "delete",
            80,
        ));
        trail.append(sample_entry(
            "e3",
            AuditEventTypeV6::Admin,
            "config",
            90,
        ));
        let results = trail.query_high_risk(70);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_non_compliant() {
        let mut trail = AuditTrailV6::new();
        let mut e1 = sample_entry("e1", AuditEventTypeV6::Auth, "login", 10);
        e1.compliance_status = ComplianceStatusV6::Compliant;
        let mut e2 = sample_entry("e2", AuditEventTypeV6::Security, "scan", 30);
        e2.compliance_status = ComplianceStatusV6::NonCompliant;
        trail.append(e1);
        trail.append(e2);
        let results = trail.query_non_compliant();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_compute_risk_score() {
        let score = AuditTrailV6::compute_risk_score(
            &AuditEventTypeV6::Security,
            "delete",
            &serde_json::json!({}),
        );
        assert!(score >= 50);
        let score = AuditTrailV6::compute_risk_score(
            &AuditEventTypeV6::Auth,
            "read",
            &serde_json::json!({}),
        );
        assert!(score < 30);
    }

    #[test]
    fn test_detect_anomalies() {
        let mut trail = AuditTrailV6::new();
        let mut entry = sample_entry("e1", AuditEventTypeV6::Security, "delete", 80);
        let naive_time = chrono::NaiveDateTime::new(
            entry.created_at.date_naive(),
            chrono::NaiveTime::from_hms_opt(3, 0, 0).unwrap(),
        );
        entry.created_at = DateTime::from_naive_utc_and_offset(naive_time, Utc);
        let anomalies = trail.detect_anomalies(&entry);
        assert!(!anomalies.is_empty());
    }

    #[test]
    fn test_detect_anomalies_non_compliant() {
        let mut trail = AuditTrailV6::new();
        let mut entry = sample_entry("e1", AuditEventTypeV6::Security, "delete", 40);
        entry.compliance_status = ComplianceStatusV6::NonCompliant;
        let anomalies = trail.detect_anomalies(&entry);
        assert!(anomalies
            .iter()
            .any(|a| a.anomaly_type == AnomalyTypeV6::PrivilegeEscalation));
    }

    #[test]
    fn test_compute_analytics() {
        let mut trail = AuditTrailV6::new();
        trail.append(sample_entry(
            "e1",
            AuditEventTypeV6::Auth,
            "login",
            10,
        ));
        trail.append(sample_entry(
            "e2",
            AuditEventTypeV6::Security,
            "scan",
            80,
        ));
        let analytics = trail.compute_analytics();
        assert_eq!(analytics.total_events, 2);
        assert_eq!(analytics.high_risk_events, 1);
        assert_eq!(analytics.compliance_rate, 100.0);
    }

    #[test]
    fn test_generate_compliance_report() {
        let mut trail = AuditTrailV6::new();
        trail.append(sample_entry(
            "e1",
            AuditEventTypeV6::Auth,
            "login",
            10,
        ));
        let report = trail.generate_compliance_report();
        assert!(report.contains("Audit Trail V6 Compliance Report"));
        assert!(report.contains("Total Events: 1"));
        assert!(report.contains("Compliance Rate:"));
    }

    #[test]
    fn test_perform_forensic_analysis() {
        let mut trail = AuditTrailV6::new();
        let mut e1 = sample_entry("e1", AuditEventTypeV6::Auth, "login", 10);
        e1.geo_location = Some(GeoLocationV6::new().with_country("US"));
        let mut e2 = sample_entry("e2", AuditEventTypeV6::Security, "delete", 80);
        e2.geo_location = Some(GeoLocationV6::new().with_country("UK"));
        trail.append(e1);
        trail.append(e2);
        let from = Utc::now() - chrono::Duration::hours(1);
        let to = Utc::now() + chrono::Duration::hours(1);
        let analysis = trail.perform_forensic_analysis("user-1", from, to);
        assert_eq!(analysis.total_events, 2);
        assert_eq!(analysis.risk_events, 1);
        assert_eq!(analysis.geo_summary.get("US"), Some(&1));
        assert_eq!(analysis.geo_summary.get("UK"), Some(&1));
    }

    #[test]
    fn test_risk_level_from_score() {
        assert_eq!(RiskLevelV6::from_score(10), RiskLevelV6::Low);
        assert_eq!(RiskLevelV6::from_score(30), RiskLevelV6::Medium);
        assert_eq!(RiskLevelV6::from_score(60), RiskLevelV6::High);
        assert_eq!(RiskLevelV6::from_score(90), RiskLevelV6::Critical);
    }

    #[test]
    fn test_geo_location() {
        let geo = GeoLocationV6::new()
            .with_country("US")
            .with_region("CA")
            .with_city("San Francisco")
            .with_coordinates(37.7749, -122.4194);
        assert_eq!(geo.country.as_deref(), Some("US"));
        assert_eq!(geo.region.as_deref(), Some("CA"));
        assert_eq!(geo.city.as_deref(), Some("San Francisco"));
        assert_eq!(geo.latitude, Some(37.7749));
        assert_eq!(geo.longitude, Some(-122.4194));
    }

    #[test]
    fn test_empty_trail() {
        let trail = AuditTrailV6::new();
        assert!(trail.is_empty());
        assert_eq!(trail.len(), 0);
        let analytics = trail.compute_analytics();
        assert_eq!(analytics.total_events, 0);
    }

    #[test]
    fn test_event_type_as_str() {
        assert_eq!(AuditEventTypeV6::Auth.as_str(), "auth");
        assert_eq!(AuditEventTypeV6::Security.as_str(), "security");
        assert_eq!(AuditEventTypeV6::Admin.as_str(), "admin");
    }

    #[test]
    fn test_compliance_status_as_str() {
        assert_eq!(ComplianceStatusV6::Compliant.as_str(), "compliant");
        assert_eq!(
            ComplianceStatusV6::NonCompliant.as_str(),
            "non_compliant"
        );
    }
}
