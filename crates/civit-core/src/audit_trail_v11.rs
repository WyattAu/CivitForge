#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocationV11 {
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
}

impl GeoLocationV11 {
    pub fn new() -> Self {
        Self {
            country: None,
            region: None,
            city: None,
            latitude: None,
            longitude: None,
            timezone: None,
        }
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

impl Default for GeoLocationV11 {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatusV11 {
    Compliant,
    NonCompliant,
    Partial,
    Unknown,
}

impl ComplianceStatusV11 {
    pub fn risk_contribution(&self) -> u32 {
        match self {
            Self::Compliant => 0,
            Self::Partial => 50,
            Self::NonCompliant => 100,
            Self::Unknown => 25,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailEventV11 {
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
    pub geo_location: Option<GeoLocationV11>,
    pub risk_score: u32,
    pub compliance_status: ComplianceStatusV11,
    pub created_at: DateTime<Utc>,
}

impl AuditTrailEventV11 {
    pub fn new(
        event_type: &str,
        resource_type: &str,
        resource_id: &str,
        action: &str,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: event_type.into(),
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
            compliance_status: ComplianceStatusV11::Unknown,
            created_at: Utc::now(),
        }
    }

    pub fn with_risk_score(mut self, score: u32) -> Self {
        self.risk_score = score;
        self
    }

    pub fn with_compliance_status(mut self, status: ComplianceStatusV11) -> Self {
        self.compliance_status = status;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailQueryV11 {
    pub event_type: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub actor_id: Option<String>,
    pub action: Option<String>,
    pub request_id: Option<String>,
    pub session_id: Option<String>,
    pub compliance_status: Option<ComplianceStatusV11>,
    pub min_risk_score: Option<u32>,
    pub max_risk_score: Option<u32>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: u32,
    pub offset: u32,
}

impl Default for AuditTrailQueryV11 {
    fn default() -> Self {
        Self {
            event_type: None,
            resource_type: None,
            resource_id: None,
            actor_id: None,
            action: None,
            request_id: None,
            session_id: None,
            compliance_status: None,
            min_risk_score: None,
            max_risk_score: None,
            since: None,
            until: None,
            limit: 100,
            offset: 0,
        }
    }
}

impl AuditTrailQueryV11 {
    pub fn matches(&self, event: &AuditTrailEventV11) -> bool {
        if let Some(ref et) = self.event_type
            && event.event_type != *et
        {
            return false;
        }
        if let Some(ref rt) = self.resource_type
            && event.resource_type != *rt
        {
            return false;
        }
        if let Some(ref rid) = self.resource_id
            && event.resource_id != *rid
        {
            return false;
        }
        if let Some(ref aid) = self.actor_id
            && event.actor_id.as_deref() != Some(aid.as_str())
        {
            return false;
        }
        if let Some(ref a) = self.action
            && event.action != *a
        {
            return false;
        }
        if let Some(ref req_id) = self.request_id
            && event.request_id.as_deref() != Some(req_id.as_str())
        {
            return false;
        }
        if let Some(ref sess_id) = self.session_id
            && event.session_id.as_deref() != Some(sess_id.as_str())
        {
            return false;
        }
        if let Some(ref cs) = self.compliance_status
            && event.compliance_status != *cs
        {
            return false;
        }
        if let Some(min_risk) = self.min_risk_score
            && event.risk_score < min_risk
        {
            return false;
        }
        if let Some(max_risk) = self.max_risk_score
            && event.risk_score > max_risk
        {
            return false;
        }
        if let Some(since) = self.since
            && event.created_at < since
        {
            return false;
        }
        if let Some(until) = self.until
            && event.created_at > until
        {
            return false;
        }
        true
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditTrailRecorderV11 {
    events: std::sync::Mutex<Vec<AuditTrailEventV11>>,
}

impl AuditTrailRecorderV11 {
    pub fn new() -> Self {
        Self {
            events: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn record(&self, event: AuditTrailEventV11) {
        let mut events = self.events.lock().unwrap();
        events.push(event);
    }

    pub fn search(&self, query: &AuditTrailQueryV11) -> Vec<AuditTrailEventV11> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| query.matches(e))
            .skip(query.offset as usize)
            .take(query.limit as usize)
            .cloned()
            .collect()
    }

    pub fn count(&self) -> usize {
        let events = self.events.lock().unwrap();
        events.len()
    }

    pub fn count_by_session(&self, session_id: &str) -> usize {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.session_id.as_deref() == Some(session_id))
            .count()
    }

    pub fn get_session_events(&self, session_id: &str) -> Vec<AuditTrailEventV11> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.session_id.as_deref() == Some(session_id))
            .cloned()
            .collect()
    }

    pub fn get_request_events(&self, request_id: &str) -> Vec<AuditTrailEventV11> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.request_id.as_deref() == Some(request_id))
            .cloned()
            .collect()
    }

    pub fn get_events_by_geo_country(&self, country: &str) -> Vec<AuditTrailEventV11> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| {
                e.geo_location
                    .as_ref()
                    .and_then(|g| g.country.as_deref())
                    .map(|c| c == country)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    pub fn get_events_by_compliance_status(&self, status: ComplianceStatusV11) -> Vec<AuditTrailEventV11> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.compliance_status == status)
            .cloned()
            .collect()
    }

    pub fn get_high_risk_events(&self, threshold: u32) -> Vec<AuditTrailEventV11> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.risk_score >= threshold)
            .cloned()
            .collect()
    }

    pub fn compliance_audit(
        &self,
        event_type: &str,
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    ) -> ComplianceAuditResultV11 {
        let events = self.events.lock().unwrap();
        let relevant: Vec<_> = events
            .iter()
            .filter(|e| {
                e.event_type == event_type && e.created_at >= since && e.created_at <= until
            })
            .cloned()
            .collect();

        let actor_count = relevant
            .iter()
            .filter_map(|e| e.actor_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .len();

        let resource_count = relevant
            .iter()
            .map(|e| format!("{}:{}", e.resource_type, e.resource_id))
            .collect::<std::collections::HashSet<_>>()
            .len();

        let action_counts = {
            let mut counts = std::collections::HashMap::new();
            for event in &relevant {
                *counts.entry(event.action.clone()).or_insert(0u32) += 1;
            }
            counts
        };

        let compliance_counts = {
            let mut counts = std::collections::HashMap::new();
            for event in &relevant {
                *counts
                    .entry(format!("{:?}", event.compliance_status))
                    .or_insert(0u32) += 1;
            }
            counts
        };

        let avg_risk_score = if relevant.is_empty() {
            0.0
        } else {
            relevant.iter().map(|e| e.risk_score as f64).sum::<f64>() / relevant.len() as f64
        };

        ComplianceAuditResultV11 {
            event_type: event_type.into(),
            period_start: since,
            period_end: until,
            total_events: relevant.len() as u32,
            unique_actors: actor_count as u32,
            unique_resources: resource_count as u32,
            action_counts,
            compliance_counts,
            avg_risk_score,
        }
    }

    pub fn forensics_export(
        &self,
        actor_id: &str,
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    ) -> ForensicsExportV11 {
        let events = self.events.lock().unwrap();
        let relevant: Vec<_> = events
            .iter()
            .filter(|e| {
                e.actor_id.as_deref() == Some(actor_id)
                    && e.created_at >= since
                    && e.created_at <= until
            })
            .cloned()
            .collect();

        let session_ids: Vec<_> = relevant
            .iter()
            .filter_map(|e| e.session_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let ip_addresses: Vec<_> = relevant
            .iter()
            .filter_map(|e| e.ip_address.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let event_type_counts = {
            let mut counts = std::collections::HashMap::new();
            for event in &relevant {
                *counts.entry(event.event_type.clone()).or_insert(0u32) += 1;
            }
            counts
        };

        let compliance_breakdown = {
            let mut counts = std::collections::HashMap::new();
            for event in &relevant {
                *counts
                    .entry(format!("{:?}", event.compliance_status))
                    .or_insert(0u32) += 1;
            }
            counts
        };

        let avg_risk_score = if relevant.is_empty() {
            0.0
        } else {
            relevant.iter().map(|e| e.risk_score as f64).sum::<f64>() / relevant.len() as f64
        };

        ForensicsExportV11 {
            actor_id: actor_id.into(),
            period_start: since,
            period_end: until,
            total_events: relevant.len() as u32,
            session_ids,
            ip_addresses,
            event_type_counts,
            compliance_breakdown,
            avg_risk_score,
            events: relevant,
        }
    }

    pub fn detect_anomalies(&self) -> Vec<AnomalyDetectionResultV11> {
        let events = self.events.lock().unwrap();
        let mut anomalies = Vec::new();

        let mut actor_event_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for event in events.iter() {
            if let Some(ref actor) = event.actor_id {
                *actor_event_counts.entry(actor.clone()).or_insert(0) += 1;
            }
        }

        let avg_events_per_actor = if actor_event_counts.is_empty() {
            0.0
        } else {
            actor_event_counts.values().sum::<u32>() as f64 / actor_event_counts.len() as f64
        };

        for (actor, count) in &actor_event_counts {
            if *count as f64 > avg_events_per_actor * 2.0 && *count > 10 {
                anomalies.push(AnomalyDetectionResultV11 {
                    anomaly_type: AnomalyTypeV11::HighActivity,
                    actor_id: Some(actor.clone()),
                    description: format!(
                        "Actor {} has {} events, which is {:.1}x the average ({:.0})",
                        actor, count, *count as f64 / avg_events_per_actor.max(1.0), avg_events_per_actor
                    ),
                    risk_level: if *count as f64 > avg_events_per_actor * 5.0 {
                        RiskLevelV11::Critical
                    } else if *count as f64 > avg_events_per_actor * 3.0 {
                        RiskLevelV11::High
                    } else {
                        RiskLevelV11::Medium
                    },
                    detected_at: Utc::now(),
                });
            }
        }

        let mut failed_count = 0u32;
        let mut total_count = 0u32;
        for event in events.iter() {
            if event.compliance_status == ComplianceStatusV11::NonCompliant {
                failed_count += 1;
            }
            total_count += 1;
        }

        if total_count > 10 && (failed_count as f64 / total_count as f64) > 0.3 {
            anomalies.push(AnomalyDetectionResultV11 {
                anomaly_type: AnomalyTypeV11::ComplianceDrift,
                actor_id: None,
                description: format!(
                    "High non-compliance rate: {:.1}% ({}/{})",
                    (failed_count as f64 / total_count as f64) * 100.0,
                    failed_count,
                    total_count
                ),
                risk_level: RiskLevelV11::High,
                detected_at: Utc::now(),
            });
        }

        anomalies
    }

    pub fn export(&self, query: &AuditTrailQueryV11) -> AuditExportV11 {
        let events = self.search(query);
        AuditExportV11 {
            format: ExportFormatV11::Json,
            total_events: events.len(),
            events,
            exported_at: Utc::now(),
        }
    }
}

impl Default for AuditTrailRecorderV11 {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAuditResultV11 {
    pub event_type: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_events: u32,
    pub unique_actors: u32,
    pub unique_resources: u32,
    pub action_counts: std::collections::HashMap<String, u32>,
    pub compliance_counts: std::collections::HashMap<String, u32>,
    pub avg_risk_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicsExportV11 {
    pub actor_id: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_events: u32,
    pub session_ids: Vec<String>,
    pub ip_addresses: Vec<String>,
    pub event_type_counts: std::collections::HashMap<String, u32>,
    pub compliance_breakdown: std::collections::HashMap<String, u32>,
    pub avg_risk_score: f64,
    pub events: Vec<AuditTrailEventV11>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditExportV11 {
    pub format: ExportFormatV11,
    pub total_events: usize,
    pub events: Vec<AuditTrailEventV11>,
    pub exported_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormatV11 {
    Json,
    Csv,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailBuilderV11 {
    event_type: String,
    resource_type: String,
    resource_id: String,
    actor_id: Option<String>,
    action: String,
    details: serde_json::Value,
    ip_address: Option<String>,
    user_agent: Option<String>,
    request_id: Option<String>,
    session_id: Option<String>,
    geo_location: Option<GeoLocationV11>,
    risk_score: u32,
    compliance_status: ComplianceStatusV11,
}

impl AuditTrailBuilderV11 {
    pub fn new(event_type: &str, resource_type: &str, resource_id: &str, action: &str) -> Self {
        Self {
            event_type: event_type.into(),
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
            compliance_status: ComplianceStatusV11::Unknown,
        }
    }

    pub fn actor_id(mut self, id: &str) -> Self {
        self.actor_id = Some(id.into());
        self
    }

    pub fn details(mut self, details: serde_json::Value) -> Self {
        self.details = details;
        self
    }

    pub fn ip_address(mut self, ip: &str) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    pub fn user_agent(mut self, ua: &str) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    pub fn request_id(mut self, id: &str) -> Self {
        self.request_id = Some(id.into());
        self
    }

    pub fn session_id(mut self, id: &str) -> Self {
        self.session_id = Some(id.into());
        self
    }

    pub fn geo_location(mut self, geo: GeoLocationV11) -> Self {
        self.geo_location = Some(geo);
        self
    }

    pub fn risk_score(mut self, score: u32) -> Self {
        self.risk_score = score;
        self
    }

    pub fn compliance_status(mut self, status: ComplianceStatusV11) -> Self {
        self.compliance_status = status;
        self
    }

    pub fn build(self) -> AuditTrailEventV11 {
        AuditTrailEventV11 {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: self.event_type,
            resource_type: self.resource_type,
            resource_id: self.resource_id,
            actor_id: self.actor_id,
            action: self.action,
            details: self.details,
            ip_address: self.ip_address,
            user_agent: self.user_agent,
            request_id: self.request_id,
            session_id: self.session_id,
            geo_location: self.geo_location,
            risk_score: self.risk_score,
            compliance_status: self.compliance_status,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyTypeV11 {
    HighActivity,
    ComplianceDrift,
    UnusualAccess,
    RiskEscalation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevelV11 {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionResultV11 {
    pub anomaly_type: AnomalyTypeV11,
    pub actor_id: Option<String>,
    pub description: String,
    pub risk_level: RiskLevelV11,
    pub detected_at: DateTime<Utc>,
}

pub fn record_compliance_event_v11(
    recorder: &AuditTrailRecorderV11,
    repo_id: &str,
    action: &str,
    framework_name: &str,
    details: serde_json::Value,
    risk_score: u32,
    compliance_status: ComplianceStatusV11,
) {
    let event = AuditTrailBuilderV11::new("compliance", "repository", repo_id, action)
        .details(serde_json::json!({
            "framework": framework_name,
            "details": details,
        }))
        .risk_score(risk_score)
        .compliance_status(compliance_status)
        .build();
    recorder.record(event);
}

pub fn record_security_event_v11(
    recorder: &AuditTrailRecorderV11,
    repo_id: &str,
    action: &str,
    scan_type: &str,
    details: serde_json::Value,
    risk_score: u32,
    compliance_status: ComplianceStatusV11,
) {
    let event = AuditTrailBuilderV11::new("security", "repository", repo_id, action)
        .details(serde_json::json!({
            "scan_type": scan_type,
            "details": details,
        }))
        .risk_score(risk_score)
        .compliance_status(compliance_status)
        .build();
    recorder.record(event);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(event_type: &str, resource_type: &str, resource_id: &str, action: &str) -> AuditTrailEventV11 {
        AuditTrailEventV11 {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: event_type.into(),
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            actor_id: Some("user-1".into()),
            action: action.into(),
            details: serde_json::Value::Object(serde_json::Map::new()),
            ip_address: None,
            user_agent: None,
            request_id: None,
            session_id: None,
            geo_location: None,
            risk_score: 0,
            compliance_status: ComplianceStatusV11::Unknown,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_geo_location_v11_new() {
        let geo = GeoLocationV11::new();
        assert!(geo.country.is_none());
        assert!(geo.region.is_none());
        assert!(geo.city.is_none());
        assert!(geo.latitude.is_none());
        assert!(geo.longitude.is_none());
    }

    #[test]
    fn test_geo_location_v11_with_fields() {
        let geo = GeoLocationV11::new()
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
    fn test_audit_trail_event_v11_new() {
        let event = AuditTrailEventV11::new("access", "repo", "r1", "read");
        assert_eq!(event.event_type, "access");
        assert_eq!(event.resource_type, "repo");
        assert_eq!(event.resource_id, "r1");
        assert_eq!(event.action, "read");
        assert!(event.request_id.is_none());
        assert!(event.session_id.is_none());
        assert!(event.geo_location.is_none());
        assert_eq!(event.risk_score, 0);
        assert_eq!(event.compliance_status, ComplianceStatusV11::Unknown);
    }

    #[test]
    fn test_audit_trail_event_v11_with_risk_score() {
        let event = AuditTrailEventV11::new("access", "repo", "r1", "read")
            .with_risk_score(75)
            .with_compliance_status(ComplianceStatusV11::NonCompliant);
        assert_eq!(event.risk_score, 75);
        assert_eq!(event.compliance_status, ComplianceStatusV11::NonCompliant);
    }

    #[test]
    fn test_compliance_status_risk_contribution() {
        assert_eq!(ComplianceStatusV11::Compliant.risk_contribution(), 0);
        assert_eq!(ComplianceStatusV11::Partial.risk_contribution(), 50);
        assert_eq!(ComplianceStatusV11::NonCompliant.risk_contribution(), 100);
        assert_eq!(ComplianceStatusV11::Unknown.risk_contribution(), 25);
    }

    #[test]
    fn test_audit_trail_query_v11_matches() {
        let mut event = AuditTrailEventV11::new("access", "repo", "r1", "read");
        event.request_id = Some("req-1".into());
        event.session_id = Some("sess-1".into());
        event.risk_score = 50;
        event.compliance_status = ComplianceStatusV11::Compliant;
        let query = AuditTrailQueryV11 {
            request_id: Some("req-1".into()),
            ..AuditTrailQueryV11::default()
        };
        assert!(query.matches(&event));
        let query2 = AuditTrailQueryV11 {
            request_id: Some("req-2".into()),
            ..AuditTrailQueryV11::default()
        };
        assert!(!query2.matches(&event));
    }

    #[test]
    fn test_audit_trail_query_v11_matches_risk_score() {
        let mut event = AuditTrailEventV11::new("access", "repo", "r1", "read");
        event.risk_score = 75;
        let query = AuditTrailQueryV11 {
            min_risk_score: Some(50),
            max_risk_score: Some(100),
            ..AuditTrailQueryV11::default()
        };
        assert!(query.matches(&event));
        let query2 = AuditTrailQueryV11 {
            min_risk_score: Some(80),
            ..AuditTrailQueryV11::default()
        };
        assert!(!query2.matches(&event));
    }

    #[test]
    fn test_audit_trail_query_v11_matches_compliance_status() {
        let mut event = AuditTrailEventV11::new("access", "repo", "r1", "read");
        event.compliance_status = ComplianceStatusV11::Compliant;
        let query = AuditTrailQueryV11 {
            compliance_status: Some(ComplianceStatusV11::Compliant),
            ..AuditTrailQueryV11::default()
        };
        assert!(query.matches(&event));
        let query2 = AuditTrailQueryV11 {
            compliance_status: Some(ComplianceStatusV11::NonCompliant),
            ..AuditTrailQueryV11::default()
        };
        assert!(!query2.matches(&event));
    }

    #[test]
    fn test_audit_trail_recorder_v11_record_and_search() {
        let recorder = AuditTrailRecorderV11::new();
        let event = AuditTrailEventV11::new("access", "repo", "r1", "read");
        recorder.record(event);
        assert_eq!(recorder.count(), 1);
        let results = recorder.search(&AuditTrailQueryV11::default());
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_audit_trail_recorder_v11_session_tracking() {
        let recorder = AuditTrailRecorderV11::new();
        let mut e1 = AuditTrailEventV11::new("access", "repo", "r1", "read");
        e1.session_id = Some("sess-1".into());
        let mut e2 = AuditTrailEventV11::new("access", "repo", "r1", "read");
        e2.session_id = Some("sess-1".into());
        let mut e3 = AuditTrailEventV11::new("access", "repo", "r1", "read");
        e3.session_id = Some("sess-2".into());
        recorder.record(e1);
        recorder.record(e2);
        recorder.record(e3);
        assert_eq!(recorder.count_by_session("sess-1"), 2);
        assert_eq!(recorder.count_by_session("sess-2"), 1);
        assert_eq!(recorder.get_session_events("sess-1").len(), 2);
    }

    #[test]
    fn test_audit_trail_recorder_v11_geo_tracking() {
        let recorder = AuditTrailRecorderV11::new();
        let mut e1 = AuditTrailEventV11::new("access", "repo", "r1", "read");
        e1.geo_location = Some(GeoLocationV11::new().with_country("US"));
        let mut e2 = AuditTrailEventV11::new("access", "repo", "r2", "read");
        e2.geo_location = Some(GeoLocationV11::new().with_country("UK"));
        let mut e3 = AuditTrailEventV11::new("access", "repo", "r3", "read");
        e3.geo_location = Some(GeoLocationV11::new().with_country("US"));
        recorder.record(e1);
        recorder.record(e2);
        recorder.record(e3);
        assert_eq!(recorder.get_events_by_geo_country("US").len(), 2);
        assert_eq!(recorder.get_events_by_geo_country("UK").len(), 1);
    }

    #[test]
    fn test_audit_trail_recorder_v11_compliance_tracking() {
        let recorder = AuditTrailRecorderV11::new();
        let mut e1 = AuditTrailEventV11::new("access", "repo", "r1", "read");
        e1.compliance_status = ComplianceStatusV11::Compliant;
        let mut e2 = AuditTrailEventV11::new("access", "repo", "r2", "read");
        e2.compliance_status = ComplianceStatusV11::NonCompliant;
        let mut e3 = AuditTrailEventV11::new("access", "repo", "r3", "read");
        e3.compliance_status = ComplianceStatusV11::Compliant;
        recorder.record(e1);
        recorder.record(e2);
        recorder.record(e3);
        assert_eq!(recorder.get_events_by_compliance_status(ComplianceStatusV11::Compliant).len(), 2);
        assert_eq!(recorder.get_events_by_compliance_status(ComplianceStatusV11::NonCompliant).len(), 1);
    }

    #[test]
    fn test_audit_trail_recorder_v11_high_risk_events() {
        let recorder = AuditTrailRecorderV11::new();
        let mut e1 = AuditTrailEventV11::new("access", "repo", "r1", "read");
        e1.risk_score = 90;
        let mut e2 = AuditTrailEventV11::new("access", "repo", "r2", "read");
        e2.risk_score = 50;
        let mut e3 = AuditTrailEventV11::new("access", "repo", "r3", "read");
        e3.risk_score = 30;
        recorder.record(e1);
        recorder.record(e2);
        recorder.record(e3);
        assert_eq!(recorder.get_high_risk_events(75).len(), 1);
        assert_eq!(recorder.get_high_risk_events(40).len(), 2);
    }

    #[test]
    fn test_audit_trail_recorder_v11_compliance_audit() {
        let recorder = AuditTrailRecorderV11::new();
        let mut e1 = AuditTrailEventV11::new("security", "repo", "r1", "scan");
        e1.actor_id = Some("user-1".into());
        e1.risk_score = 75;
        e1.compliance_status = ComplianceStatusV11::Compliant;
        let mut e2 = AuditTrailEventV11::new("security", "repo", "r2", "scan");
        e2.actor_id = Some("user-1".into());
        e2.risk_score = 25;
        e2.compliance_status = ComplianceStatusV11::NonCompliant;
        let mut e3 = AuditTrailEventV11::new("security", "repo", "r1", "fix");
        e3.actor_id = Some("user-2".into());
        e3.risk_score = 50;
        e3.compliance_status = ComplianceStatusV11::Compliant;
        recorder.record(e1);
        recorder.record(e2);
        recorder.record(e3);
        let result = recorder.compliance_audit(
            "security",
            Utc::now() - chrono::Duration::hours(1),
            Utc::now() + chrono::Duration::hours(1),
        );
        assert_eq!(result.total_events, 3);
        assert_eq!(result.unique_actors, 2);
        assert_eq!(result.unique_resources, 2);
        assert_eq!(result.action_counts.get("scan"), Some(&2));
        assert_eq!(result.action_counts.get("fix"), Some(&1));
        assert!(result.avg_risk_score > 0.0);
    }

    #[test]
    fn test_audit_trail_recorder_v11_forensics_export() {
        let recorder = AuditTrailRecorderV11::new();
        let mut e1 = AuditTrailEventV11::new("access", "repo", "r1", "read");
        e1.actor_id = Some("user-1".into());
        e1.session_id = Some("sess-1".into());
        e1.ip_address = Some("10.0.0.1".into());
        e1.risk_score = 50;
        e1.compliance_status = ComplianceStatusV11::Compliant;
        let mut e2 = AuditTrailEventV11::new("access", "repo", "r2", "write");
        e2.actor_id = Some("user-1".into());
        e2.session_id = Some("sess-1".into());
        e2.ip_address = Some("10.0.0.1".into());
        e2.risk_score = 75;
        e2.compliance_status = ComplianceStatusV11::NonCompliant;
        let mut e3 = AuditTrailEventV11::new("access", "repo", "r3", "read");
        e3.actor_id = Some("user-2".into());
        recorder.record(e1);
        recorder.record(e2);
        recorder.record(e3);
        let export = recorder.forensics_export(
            "user-1",
            Utc::now() - chrono::Duration::hours(1),
            Utc::now() + chrono::Duration::hours(1),
        );
        assert_eq!(export.total_events, 2);
        assert_eq!(export.session_ids, vec!["sess-1"]);
        assert_eq!(export.ip_addresses, vec!["10.0.0.1"]);
        assert_eq!(export.event_type_counts.get("access"), Some(&2));
        assert!(export.avg_risk_score > 0.0);
    }

    #[test]
    fn test_audit_trail_recorder_v11_detect_anomalies() {
        let recorder = AuditTrailRecorderV11::new();
        for i in 0..20 {
            let mut e = AuditTrailEventV11::new("access", "repo", &format!("r{}", i), "read");
            e.actor_id = Some("user-1".into());
            e.risk_score = 10;
            e.compliance_status = ComplianceStatusV11::Compliant;
            recorder.record(e);
        }
        for i in 0..5 {
            let mut e = AuditTrailEventV11::new("access", "repo", &format!("r{}", i + 20), "read");
            e.actor_id = Some("user-2".into());
            e.risk_score = 10;
            e.compliance_status = ComplianceStatusV11::Compliant;
            recorder.record(e);
        }
        let anomalies = recorder.detect_anomalies();
        assert!(!anomalies.is_empty());
        assert!(anomalies.iter().any(|a| a.anomaly_type == AnomalyTypeV11::HighActivity));
    }

    #[test]
    fn test_audit_trail_recorder_v11_detect_anomalies_compliance_drift() {
        let recorder = AuditTrailRecorderV11::new();
        for i in 0..15 {
            let mut e = AuditTrailEventV11::new("security", "repo", &format!("r{}", i), "scan");
            e.actor_id = Some("user-1".into());
            e.compliance_status = ComplianceStatusV11::NonCompliant;
            recorder.record(e);
        }
        let anomalies = recorder.detect_anomalies();
        assert!(!anomalies.is_empty());
        assert!(anomalies.iter().any(|a| a.anomaly_type == AnomalyTypeV11::ComplianceDrift));
    }

    #[test]
    fn test_audit_trail_builder_v11() {
        let geo = GeoLocationV11::new()
            .with_country("US")
            .with_city("New York");
        let event = AuditTrailBuilderV11::new("security", "repo", "r1", "scan")
            .actor_id("user-1")
            .ip_address("10.0.0.1")
            .user_agent("test-agent")
            .request_id("req-1")
            .session_id("sess-1")
            .geo_location(geo)
            .risk_score(75)
            .compliance_status(ComplianceStatusV11::Compliant)
            .details(serde_json::json!({"key": "value"}))
            .build();
        assert_eq!(event.event_type, "security");
        assert_eq!(event.actor_id.as_deref(), Some("user-1"));
        assert_eq!(event.ip_address.as_deref(), Some("10.0.0.1"));
        assert_eq!(event.request_id.as_deref(), Some("req-1"));
        assert_eq!(event.session_id.as_deref(), Some("sess-1"));
        assert!(event.geo_location.is_some());
        assert_eq!(event.geo_location.as_ref().unwrap().country.as_deref(), Some("US"));
        assert_eq!(event.risk_score, 75);
        assert_eq!(event.compliance_status, ComplianceStatusV11::Compliant);
        assert_eq!(event.details["key"], "value");
    }

    #[test]
    fn test_audit_trail_export_v11() {
        let recorder = AuditTrailRecorderV11::new();
        recorder.record(AuditTrailEventV11::new("access", "repo", "r1", "read"));
        let export = recorder.export(&AuditTrailQueryV11::default());
        assert_eq!(export.total_events, 1);
        assert_eq!(export.format, ExportFormatV11::Json);
    }

    #[test]
    fn test_record_compliance_event_v11() {
        let recorder = AuditTrailRecorderV11::new();
        record_compliance_event_v11(
            &recorder,
            "repo-1",
            "assessment_complete",
            "SOC 2",
            serde_json::json!({"score": 100}),
            50,
            ComplianceStatusV11::Compliant,
        );
        assert_eq!(recorder.count(), 1);
        let query = AuditTrailQueryV11 {
            event_type: Some("compliance".into()),
            ..AuditTrailQueryV11::default()
        };
        let results = recorder.search(&query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].resource_id, "repo-1");
        assert_eq!(results[0].risk_score, 50);
        assert_eq!(results[0].compliance_status, ComplianceStatusV11::Compliant);
    }

    #[test]
    fn test_record_security_event_v11() {
        let recorder = AuditTrailRecorderV11::new();
        record_security_event_v11(
            &recorder,
            "repo-1",
            "scan_completed",
            "sast",
            serde_json::json!({"score": 100}),
            25,
            ComplianceStatusV11::Compliant,
        );
        assert_eq!(recorder.count(), 1);
        let query = AuditTrailQueryV11 {
            event_type: Some("security".into()),
            ..AuditTrailQueryV11::default()
        };
        let results = recorder.search(&query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].risk_score, 25);
        assert_eq!(results[0].compliance_status, ComplianceStatusV11::Compliant);
    }

    #[test]
    fn test_export_format_v11_serialization() {
        assert_eq!(
            serde_json::to_string(&ExportFormatV11::Json).unwrap(),
            "\"json\""
        );
        assert_eq!(
            serde_json::to_string(&ExportFormatV11::Csv).unwrap(),
            "\"csv\""
        );
    }

    #[test]
    fn test_anomaly_type_v11_serialization() {
        assert_eq!(
            serde_json::to_string(&AnomalyTypeV11::HighActivity).unwrap(),
            "\"high_activity\""
        );
        assert_eq!(
            serde_json::to_string(&AnomalyTypeV11::ComplianceDrift).unwrap(),
            "\"compliance_drift\""
        );
    }

    #[test]
    fn test_risk_level_v11_serialization() {
        assert_eq!(
            serde_json::to_string(&RiskLevelV11::Critical).unwrap(),
            "\"critical\""
        );
        assert_eq!(
            serde_json::to_string(&RiskLevelV11::Low).unwrap(),
            "\"low\""
        );
    }
}
