#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocationV15 {
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
}

impl GeoLocationV15 {
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

impl Default for GeoLocationV15 {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatusV15 {
    Compliant,
    NonCompliant,
    Partial,
    Unknown,
}

impl ComplianceStatusV15 {
    pub fn risk_contribution(&self) -> u32 {
        match self {
            Self::Compliant => 0,
            Self::Partial => 50,
            Self::NonCompliant => 100,
            Self::Unknown => 25,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Compliant => "Compliant",
            Self::NonCompliant => "Non-Compliant",
            Self::Partial => "Partial",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailEventV15 {
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
    pub geo_location: Option<GeoLocationV15>,
    pub risk_score: u32,
    pub compliance_status: ComplianceStatusV15,
    pub created_at: DateTime<Utc>,
}

impl AuditTrailEventV15 {
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
            compliance_status: ComplianceStatusV15::Unknown,
            created_at: Utc::now(),
        }
    }

    pub fn with_risk_score(mut self, score: u32) -> Self {
        self.risk_score = score;
        self
    }

    pub fn with_compliance_status(mut self, status: ComplianceStatusV15) -> Self {
        self.compliance_status = status;
        self
    }

    pub fn is_high_risk(&self, threshold: u32) -> bool {
        self.risk_score >= threshold
    }

    pub fn is_compliant(&self) -> bool {
        self.compliance_status == ComplianceStatusV15::Compliant
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailQueryV15 {
    pub event_type: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub actor_id: Option<String>,
    pub action: Option<String>,
    pub request_id: Option<String>,
    pub session_id: Option<String>,
    pub compliance_status: Option<ComplianceStatusV15>,
    pub min_risk_score: Option<u32>,
    pub max_risk_score: Option<u32>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: u32,
    pub offset: u32,
}

impl Default for AuditTrailQueryV15 {
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

impl AuditTrailQueryV15 {
    pub fn matches(&self, event: &AuditTrailEventV15) -> bool {
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
pub struct AuditTrailRecorderV15 {
    events: std::sync::Mutex<Vec<AuditTrailEventV15>>,
}

impl AuditTrailRecorderV15 {
    pub fn new() -> Self {
        Self {
            events: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn record(&self, event: AuditTrailEventV15) {
        let mut events = self.events.lock().unwrap();
        events.push(event);
    }

    pub fn search(&self, query: &AuditTrailQueryV15) -> Vec<AuditTrailEventV15> {
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

    pub fn get_session_events(&self, session_id: &str) -> Vec<AuditTrailEventV15> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.session_id.as_deref() == Some(session_id))
            .cloned()
            .collect()
    }

    pub fn get_request_events(&self, request_id: &str) -> Vec<AuditTrailEventV15> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.request_id.as_deref() == Some(request_id))
            .cloned()
            .collect()
    }

    pub fn get_events_by_geo_country(&self, country: &str) -> Vec<AuditTrailEventV15> {
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

    pub fn get_events_by_compliance_status(&self, status: ComplianceStatusV15) -> Vec<AuditTrailEventV15> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.compliance_status == status)
            .cloned()
            .collect()
    }

    pub fn get_high_risk_events(&self, threshold: u32) -> Vec<AuditTrailEventV15> {
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
    ) -> ComplianceAuditResultV15 {
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

        let max_risk_score = relevant.iter().map(|e| e.risk_score).max().unwrap_or(0);

        let compliant_count = relevant
            .iter()
            .filter(|e| e.compliance_status == ComplianceStatusV15::Compliant)
            .count() as u32;

        let compliance_rate = if relevant.is_empty() {
            100.0
        } else {
            (compliant_count as f64 / relevant.len() as f64) * 100.0
        };

        ComplianceAuditResultV15 {
            event_type: event_type.into(),
            period_start: since,
            period_end: until,
            total_events: relevant.len() as u32,
            unique_actors: actor_count as u32,
            unique_resources: resource_count as u32,
            action_counts,
            compliance_counts,
            avg_risk_score,
            max_risk_score,
            compliance_rate,
        }
    }

    pub fn forensics_export(
        &self,
        actor_id: &str,
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    ) -> ForensicsExportV15 {
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

        let max_risk_score = relevant.iter().map(|e| e.risk_score).max().unwrap_or(0);

        let risk_timeline: Vec<(DateTime<Utc>, u32)> = relevant
            .iter()
            .map(|e| (e.created_at, e.risk_score))
            .collect();

        ForensicsExportV15 {
            actor_id: actor_id.into(),
            period_start: since,
            period_end: until,
            total_events: relevant.len() as u32,
            session_ids,
            ip_addresses,
            event_type_counts,
            compliance_breakdown,
            avg_risk_score,
            max_risk_score,
            risk_timeline,
            events: relevant,
        }
    }

    pub fn detect_anomalies(&self) -> Vec<AnomalyDetectionResultV15> {
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
                anomalies.push(AnomalyDetectionResultV15 {
                    anomaly_type: AnomalyTypeV15::HighActivity,
                    actor_id: Some(actor.clone()),
                    description: format!(
                        "Actor {} has {} events, which is {:.1}x the average ({:.0})",
                        actor, count, *count as f64 / avg_events_per_actor.max(1.0), avg_events_per_actor
                    ),
                    risk_level: if *count as f64 > avg_events_per_actor * 5.0 {
                        RiskLevelV15::Critical
                    } else if *count as f64 > avg_events_per_actor * 3.0 {
                        RiskLevelV15::High
                    } else {
                        RiskLevelV15::Medium
                    },
                    detected_at: Utc::now(),
                });
            }
        }

        let mut failed_count = 0u32;
        let mut total_count = 0u32;
        for event in events.iter() {
            if event.compliance_status == ComplianceStatusV15::NonCompliant {
                failed_count += 1;
            }
            total_count += 1;
        }

        if total_count > 10 && (failed_count as f64 / total_count as f64) > 0.3 {
            anomalies.push(AnomalyDetectionResultV15 {
                anomaly_type: AnomalyTypeV15::ComplianceDrift,
                actor_id: None,
                description: format!(
                    "High non-compliance rate: {:.1}% ({}/{})",
                    (failed_count as f64 / total_count as f64) * 100.0,
                    failed_count,
                    total_count
                ),
                risk_level: RiskLevelV15::High,
                detected_at: Utc::now(),
            });
        }

        let mut ip_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        let mut actor_ips: std::collections::HashMap<String, std::collections::HashSet<String>> = std::collections::HashMap::new();
        for event in events.iter() {
            if let (Some(actor), Some(ip)) = (&event.actor_id, &event.ip_address) {
                *ip_counts.entry(ip.clone()).or_insert(0) += 1;
                actor_ips
                    .entry(actor.clone())
                    .or_default()
                    .insert(ip.clone());
            }
        }

        for (actor, ips) in &actor_ips {
            if ips.len() > 3 {
                anomalies.push(AnomalyDetectionResultV15 {
                    anomaly_type: AnomalyTypeV15::UnusualAccess,
                    actor_id: Some(actor.clone()),
                    description: format!(
                        "Actor {} accessed from {} different IP addresses",
                        actor,
                        ips.len()
                    ),
                    risk_level: if ips.len() > 10 {
                        RiskLevelV15::Critical
                    } else if ips.len() > 5 {
                        RiskLevelV15::High
                    } else {
                        RiskLevelV15::Medium
                    },
                    detected_at: Utc::now(),
                });
            }
        }

        anomalies
    }

    pub fn risk_assessment(&self) -> RiskAssessmentResultV15 {
        let events = self.events.lock().unwrap();
        let total = events.len() as u32;
        let high_risk = events.iter().filter(|e| e.risk_score >= 75).count() as u32;
        let medium_risk = events
            .iter()
            .filter(|e| e.risk_score >= 25 && e.risk_score < 75)
            .count() as u32;
        let low_risk = events.iter().filter(|e| e.risk_score < 25).count() as u32;

        let avg_risk = if total == 0 {
            0.0
        } else {
            events.iter().map(|e| e.risk_score as f64).sum::<f64>() / total as f64
        };

        let max_risk = events.iter().map(|e| e.risk_score).max().unwrap_or(0);

        let risk_level = if avg_risk >= 75.0 {
            RiskLevelV15::Critical
        } else if avg_risk >= 50.0 {
            RiskLevelV15::High
        } else if avg_risk >= 25.0 {
            RiskLevelV15::Medium
        } else {
            RiskLevelV15::Low
        };

        RiskAssessmentResultV15 {
            total_events: total,
            high_risk_count: high_risk,
            medium_risk_count: medium_risk,
            low_risk_count: low_risk,
            avg_risk_score: avg_risk,
            max_risk_score: max_risk,
            risk_level,
        }
    }

    pub fn export(&self, query: &AuditTrailQueryV15) -> AuditExportV15 {
        let events = self.search(query);
        AuditExportV15 {
            format: ExportFormatV15::Json,
            total_events: events.len(),
            events,
            exported_at: Utc::now(),
        }
    }
}

impl Default for AuditTrailRecorderV15 {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAuditResultV15 {
    pub event_type: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_events: u32,
    pub unique_actors: u32,
    pub unique_resources: u32,
    pub action_counts: std::collections::HashMap<String, u32>,
    pub compliance_counts: std::collections::HashMap<String, u32>,
    pub avg_risk_score: f64,
    pub max_risk_score: u32,
    pub compliance_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicsExportV15 {
    pub actor_id: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_events: u32,
    pub session_ids: Vec<String>,
    pub ip_addresses: Vec<String>,
    pub event_type_counts: std::collections::HashMap<String, u32>,
    pub compliance_breakdown: std::collections::HashMap<String, u32>,
    pub avg_risk_score: f64,
    pub max_risk_score: u32,
    pub risk_timeline: Vec<(DateTime<Utc>, u32)>,
    pub events: Vec<AuditTrailEventV15>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditExportV15 {
    pub format: ExportFormatV15,
    pub total_events: usize,
    pub events: Vec<AuditTrailEventV15>,
    pub exported_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormatV15 {
    Json,
    Csv,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailBuilderV15 {
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
    geo_location: Option<GeoLocationV15>,
    risk_score: u32,
    compliance_status: ComplianceStatusV15,
}

impl AuditTrailBuilderV15 {
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
            compliance_status: ComplianceStatusV15::Unknown,
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

    pub fn geo_location(mut self, geo: GeoLocationV15) -> Self {
        self.geo_location = Some(geo);
        self
    }

    pub fn risk_score(mut self, score: u32) -> Self {
        self.risk_score = score;
        self
    }

    pub fn compliance_status(mut self, status: ComplianceStatusV15) -> Self {
        self.compliance_status = status;
        self
    }

    pub fn build(self) -> AuditTrailEventV15 {
        AuditTrailEventV15 {
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
pub enum AnomalyTypeV15 {
    HighActivity,
    ComplianceDrift,
    UnusualAccess,
    RiskEscalation,
    GeographicAnomaly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevelV15 {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevelV15 {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Critical => "Critical",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionResultV15 {
    pub anomaly_type: AnomalyTypeV15,
    pub actor_id: Option<String>,
    pub description: String,
    pub risk_level: RiskLevelV15,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessmentResultV15 {
    pub total_events: u32,
    pub high_risk_count: u32,
    pub medium_risk_count: u32,
    pub low_risk_count: u32,
    pub avg_risk_score: f64,
    pub max_risk_score: u32,
    pub risk_level: RiskLevelV15,
}

pub fn record_compliance_event_v15(
    recorder: &AuditTrailRecorderV15,
    repo_id: &str,
    action: &str,
    framework_name: &str,
    details: serde_json::Value,
    risk_score: u32,
    compliance_status: ComplianceStatusV15,
) {
    let event = AuditTrailBuilderV15::new("compliance", "repository", repo_id, action)
        .details(serde_json::json!({
            "framework": framework_name,
            "details": details,
        }))
        .risk_score(risk_score)
        .compliance_status(compliance_status)
        .build();
    recorder.record(event);
}

pub fn record_security_event_v15(
    recorder: &AuditTrailRecorderV15,
    repo_id: &str,
    action: &str,
    scan_type: &str,
    details: serde_json::Value,
    risk_score: u32,
    compliance_status: ComplianceStatusV15,
) {
    let event = AuditTrailBuilderV15::new("security", "repository", repo_id, action)
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

    fn make_event(event_type: &str, resource_type: &str, resource_id: &str, action: &str) -> AuditTrailEventV15 {
        AuditTrailEventV15 {
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
            compliance_status: ComplianceStatusV15::Unknown,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_geo_location_v15_new() {
        let geo = GeoLocationV15::new();
        assert!(geo.country.is_none());
        assert!(geo.region.is_none());
        assert!(geo.city.is_none());
        assert!(geo.latitude.is_none());
        assert!(geo.longitude.is_none());
    }

    #[test]
    fn test_geo_location_v15_with_fields() {
        let geo = GeoLocationV15::new()
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
    fn test_audit_trail_event_v15_new() {
        let event = AuditTrailEventV15::new("access", "repo", "r1", "read");
        assert_eq!(event.event_type, "access");
        assert_eq!(event.resource_type, "repo");
        assert_eq!(event.resource_id, "r1");
        assert_eq!(event.action, "read");
        assert!(event.request_id.is_none());
        assert!(event.session_id.is_none());
        assert!(event.geo_location.is_none());
        assert_eq!(event.risk_score, 0);
        assert_eq!(event.compliance_status, ComplianceStatusV15::Unknown);
    }

    #[test]
    fn test_audit_trail_event_v15_with_risk_score() {
        let event = AuditTrailEventV15::new("access", "repo", "r1", "read")
            .with_risk_score(75)
            .with_compliance_status(ComplianceStatusV15::NonCompliant);
        assert_eq!(event.risk_score, 75);
        assert_eq!(event.compliance_status, ComplianceStatusV15::NonCompliant);
    }

    #[test]
    fn test_event_helpers() {
        let event = AuditTrailEventV15::new("access", "repo", "r1", "read")
            .with_risk_score(80)
            .with_compliance_status(ComplianceStatusV15::Compliant);
        assert!(event.is_high_risk(75));
        assert!(!event.is_high_risk(90));
        assert!(event.is_compliant());
    }

    #[test]
    fn test_compliance_status_risk_contribution() {
        assert_eq!(ComplianceStatusV15::Compliant.risk_contribution(), 0);
        assert_eq!(ComplianceStatusV15::Partial.risk_contribution(), 50);
        assert_eq!(ComplianceStatusV15::NonCompliant.risk_contribution(), 100);
        assert_eq!(ComplianceStatusV15::Unknown.risk_contribution(), 25);
    }

    #[test]
    fn test_compliance_status_display_name() {
        assert_eq!(ComplianceStatusV15::Compliant.display_name(), "Compliant");
        assert_eq!(ComplianceStatusV15::NonCompliant.display_name(), "Non-Compliant");
    }

    #[test]
    fn test_audit_trail_query_v15_matches() {
        let mut event = AuditTrailEventV15::new("access", "repo", "r1", "read");
        event.request_id = Some("req-1".into());
        event.session_id = Some("sess-1".into());
        event.risk_score = 50;
        event.compliance_status = ComplianceStatusV15::Compliant;
        let query = AuditTrailQueryV15 {
            request_id: Some("req-1".into()),
            ..AuditTrailQueryV15::default()
        };
        assert!(query.matches(&event));
        let query2 = AuditTrailQueryV15 {
            request_id: Some("req-2".into()),
            ..AuditTrailQueryV15::default()
        };
        assert!(!query2.matches(&event));
    }

    #[test]
    fn test_audit_trail_query_v15_matches_risk_score() {
        let mut event = AuditTrailEventV15::new("access", "repo", "r1", "read");
        event.risk_score = 75;
        let query = AuditTrailQueryV15 {
            min_risk_score: Some(50),
            max_risk_score: Some(100),
            ..AuditTrailQueryV15::default()
        };
        assert!(query.matches(&event));
        let query2 = AuditTrailQueryV15 {
            min_risk_score: Some(80),
            ..AuditTrailQueryV15::default()
        };
        assert!(!query2.matches(&event));
    }

    #[test]
    fn test_audit_trail_query_v15_matches_compliance_status() {
        let mut event = AuditTrailEventV15::new("access", "repo", "r1", "read");
        event.compliance_status = ComplianceStatusV15::Compliant;
        let query = AuditTrailQueryV15 {
            compliance_status: Some(ComplianceStatusV15::Compliant),
            ..AuditTrailQueryV15::default()
        };
        assert!(query.matches(&event));
        let query2 = AuditTrailQueryV15 {
            compliance_status: Some(ComplianceStatusV15::NonCompliant),
            ..AuditTrailQueryV15::default()
        };
        assert!(!query2.matches(&event));
    }

    #[test]
    fn test_audit_trail_recorder_v15_record_and_search() {
        let recorder = AuditTrailRecorderV15::new();
        let event = AuditTrailEventV15::new("access", "repo", "r1", "read");
        recorder.record(event);
        assert_eq!(recorder.count(), 1);
        let results = recorder.search(&AuditTrailQueryV15::default());
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_audit_trail_recorder_v15_session_tracking() {
        let recorder = AuditTrailRecorderV15::new();
        let mut e1 = AuditTrailEventV15::new("access", "repo", "r1", "read");
        e1.session_id = Some("sess-1".into());
        let mut e2 = AuditTrailEventV15::new("access", "repo", "r1", "read");
        e2.session_id = Some("sess-1".into());
        let mut e3 = AuditTrailEventV15::new("access", "repo", "r1", "read");
        e3.session_id = Some("sess-2".into());
        recorder.record(e1);
        recorder.record(e2);
        recorder.record(e3);
        assert_eq!(recorder.count_by_session("sess-1"), 2);
        assert_eq!(recorder.count_by_session("sess-2"), 1);
        assert_eq!(recorder.get_session_events("sess-1").len(), 2);
    }

    #[test]
    fn test_audit_trail_recorder_v15_geo_tracking() {
        let recorder = AuditTrailRecorderV15::new();
        let mut e1 = AuditTrailEventV15::new("access", "repo", "r1", "read");
        e1.geo_location = Some(GeoLocationV15::new().with_country("US"));
        let mut e2 = AuditTrailEventV15::new("access", "repo", "r2", "read");
        e2.geo_location = Some(GeoLocationV15::new().with_country("UK"));
        let mut e3 = AuditTrailEventV15::new("access", "repo", "r3", "read");
        e3.geo_location = Some(GeoLocationV15::new().with_country("US"));
        recorder.record(e1);
        recorder.record(e2);
        recorder.record(e3);
        assert_eq!(recorder.get_events_by_geo_country("US").len(), 2);
        assert_eq!(recorder.get_events_by_geo_country("UK").len(), 1);
    }

    #[test]
    fn test_audit_trail_recorder_v15_compliance_tracking() {
        let recorder = AuditTrailRecorderV15::new();
        let mut e1 = AuditTrailEventV15::new("access", "repo", "r1", "read");
        e1.compliance_status = ComplianceStatusV15::Compliant;
        let mut e2 = AuditTrailEventV15::new("access", "repo", "r2", "read");
        e2.compliance_status = ComplianceStatusV15::NonCompliant;
        let mut e3 = AuditTrailEventV15::new("access", "repo", "r3", "read");
        e3.compliance_status = ComplianceStatusV15::Compliant;
        recorder.record(e1);
        recorder.record(e2);
        recorder.record(e3);
        assert_eq!(recorder.get_events_by_compliance_status(ComplianceStatusV15::Compliant).len(), 2);
        assert_eq!(recorder.get_events_by_compliance_status(ComplianceStatusV15::NonCompliant).len(), 1);
    }

    #[test]
    fn test_audit_trail_recorder_v15_high_risk_events() {
        let recorder = AuditTrailRecorderV15::new();
        let mut e1 = AuditTrailEventV15::new("access", "repo", "r1", "read");
        e1.risk_score = 90;
        let mut e2 = AuditTrailEventV15::new("access", "repo", "r2", "read");
        e2.risk_score = 50;
        let mut e3 = AuditTrailEventV15::new("access", "repo", "r3", "read");
        e3.risk_score = 30;
        recorder.record(e1);
        recorder.record(e2);
        recorder.record(e3);
        assert_eq!(recorder.get_high_risk_events(75).len(), 1);
        assert_eq!(recorder.get_high_risk_events(40).len(), 2);
    }

    #[test]
    fn test_audit_trail_recorder_v15_compliance_audit() {
        let recorder = AuditTrailRecorderV15::new();
        let mut e1 = AuditTrailEventV15::new("security", "repo", "r1", "scan");
        e1.actor_id = Some("user-1".into());
        e1.risk_score = 75;
        e1.compliance_status = ComplianceStatusV15::Compliant;
        let mut e2 = AuditTrailEventV15::new("security", "repo", "r2", "scan");
        e2.actor_id = Some("user-1".into());
        e2.risk_score = 25;
        e2.compliance_status = ComplianceStatusV15::NonCompliant;
        let mut e3 = AuditTrailEventV15::new("security", "repo", "r1", "fix");
        e3.actor_id = Some("user-2".into());
        e3.risk_score = 50;
        e3.compliance_status = ComplianceStatusV15::Compliant;
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
        assert!(result.max_risk_score > 0);
        assert!(result.compliance_rate > 0.0);
    }

    #[test]
    fn test_audit_trail_recorder_v15_forensics_export() {
        let recorder = AuditTrailRecorderV15::new();
        let mut e1 = AuditTrailEventV15::new("access", "repo", "r1", "read");
        e1.actor_id = Some("user-1".into());
        e1.session_id = Some("sess-1".into());
        e1.ip_address = Some("10.0.0.1".into());
        e1.risk_score = 50;
        e1.compliance_status = ComplianceStatusV15::Compliant;
        let mut e2 = AuditTrailEventV15::new("access", "repo", "r2", "write");
        e2.actor_id = Some("user-1".into());
        e2.session_id = Some("sess-1".into());
        e2.ip_address = Some("10.0.0.1".into());
        e2.risk_score = 75;
        e2.compliance_status = ComplianceStatusV15::NonCompliant;
        let mut e3 = AuditTrailEventV15::new("access", "repo", "r3", "read");
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
        assert!(export.max_risk_score > 0);
        assert_eq!(export.risk_timeline.len(), 2);
    }

    #[test]
    fn test_audit_trail_recorder_v15_detect_anomalies() {
        let recorder = AuditTrailRecorderV15::new();
        for i in 0..20 {
            let mut e = AuditTrailEventV15::new("access", "repo", &format!("r{}", i), "read");
            e.actor_id = Some("user-1".into());
            e.risk_score = 10;
            e.compliance_status = ComplianceStatusV15::Compliant;
            recorder.record(e);
        }
        for i in 0..5 {
            let mut e = AuditTrailEventV15::new("access", "repo", &format!("r{}", i + 20), "read");
            e.actor_id = Some("user-2".into());
            e.risk_score = 10;
            e.compliance_status = ComplianceStatusV15::Compliant;
            recorder.record(e);
        }
        let anomalies = recorder.detect_anomalies();
        assert!(!anomalies.is_empty());
        assert!(anomalies.iter().any(|a| a.anomaly_type == AnomalyTypeV15::HighActivity));
    }

    #[test]
    fn test_audit_trail_recorder_v15_detect_anomalies_compliance_drift() {
        let recorder = AuditTrailRecorderV15::new();
        for i in 0..15 {
            let mut e = AuditTrailEventV15::new("security", "repo", &format!("r{}", i), "scan");
            e.actor_id = Some("user-1".into());
            e.compliance_status = ComplianceStatusV15::NonCompliant;
            recorder.record(e);
        }
        let anomalies = recorder.detect_anomalies();
        assert!(!anomalies.is_empty());
        assert!(anomalies.iter().any(|a| a.anomaly_type == AnomalyTypeV15::ComplianceDrift));
    }

    #[test]
    fn test_audit_trail_recorder_v15_detect_anomalies_unusual_access() {
        let recorder = AuditTrailRecorderV15::new();
        let mut e1 = AuditTrailEventV15::new("access", "repo", "r1", "read");
        e1.actor_id = Some("user-1".into());
        e1.ip_address = Some("10.0.0.1".into());
        recorder.record(e1);
        for i in 0..5 {
            let mut e = AuditTrailEventV15::new("access", "repo", &format!("r{}", i + 2), "read");
            e.actor_id = Some("user-1".into());
            e.ip_address = Some(format!("10.0.0.{}", i + 2));
            recorder.record(e);
        }
        let anomalies = recorder.detect_anomalies();
        assert!(anomalies.iter().any(|a| a.anomaly_type == AnomalyTypeV15::UnusualAccess));
    }

    #[test]
    fn test_risk_assessment() {
        let recorder = AuditTrailRecorderV15::new();
        let mut e1 = AuditTrailEventV15::new("access", "repo", "r1", "read");
        e1.risk_score = 90;
        let mut e2 = AuditTrailEventV15::new("access", "repo", "r2", "read");
        e2.risk_score = 50;
        let mut e3 = AuditTrailEventV15::new("access", "repo", "r3", "read");
        e3.risk_score = 10;
        recorder.record(e1);
        recorder.record(e2);
        recorder.record(e3);
        let assessment = recorder.risk_assessment();
        assert_eq!(assessment.total_events, 3);
        assert_eq!(assessment.high_risk_count, 1);
        assert_eq!(assessment.medium_risk_count, 1);
        assert_eq!(assessment.low_risk_count, 1);
        assert!(assessment.avg_risk_score > 0.0);
        assert_eq!(assessment.max_risk_score, 90);
    }

    #[test]
    fn test_audit_trail_builder_v15() {
        let geo = GeoLocationV15::new()
            .with_country("US")
            .with_city("New York");
        let event = AuditTrailBuilderV15::new("security", "repo", "r1", "scan")
            .actor_id("user-1")
            .ip_address("10.0.0.1")
            .user_agent("test-agent")
            .request_id("req-1")
            .session_id("sess-1")
            .geo_location(geo)
            .risk_score(75)
            .compliance_status(ComplianceStatusV15::Compliant)
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
        assert_eq!(event.compliance_status, ComplianceStatusV15::Compliant);
        assert_eq!(event.details["key"], "value");
    }

    #[test]
    fn test_audit_trail_export_v15() {
        let recorder = AuditTrailRecorderV15::new();
        recorder.record(AuditTrailEventV15::new("access", "repo", "r1", "read"));
        let export = recorder.export(&AuditTrailQueryV15::default());
        assert_eq!(export.total_events, 1);
        assert_eq!(export.format, ExportFormatV15::Json);
    }

    #[test]
    fn test_record_compliance_event_v15() {
        let recorder = AuditTrailRecorderV15::new();
        record_compliance_event_v15(
            &recorder,
            "repo-1",
            "assessment_complete",
            "SOC 2",
            serde_json::json!({"score": 100}),
            50,
            ComplianceStatusV15::Compliant,
        );
        assert_eq!(recorder.count(), 1);
        let query = AuditTrailQueryV15 {
            event_type: Some("compliance".into()),
            ..AuditTrailQueryV15::default()
        };
        let results = recorder.search(&query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].resource_id, "repo-1");
        assert_eq!(results[0].risk_score, 50);
        assert_eq!(results[0].compliance_status, ComplianceStatusV15::Compliant);
    }

    #[test]
    fn test_record_security_event_v15() {
        let recorder = AuditTrailRecorderV15::new();
        record_security_event_v15(
            &recorder,
            "repo-1",
            "scan_completed",
            "sast",
            serde_json::json!({"score": 100}),
            25,
            ComplianceStatusV15::Compliant,
        );
        assert_eq!(recorder.count(), 1);
        let query = AuditTrailQueryV15 {
            event_type: Some("security".into()),
            ..AuditTrailQueryV15::default()
        };
        let results = recorder.search(&query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].risk_score, 25);
        assert_eq!(results[0].compliance_status, ComplianceStatusV15::Compliant);
    }

    #[test]
    fn test_risk_level_display_name() {
        assert_eq!(RiskLevelV15::Critical.display_name(), "Critical");
        assert_eq!(RiskLevelV15::Low.display_name(), "Low");
    }

    #[test]
    fn test_export_format_v15_serialization() {
        assert_eq!(
            serde_json::to_string(&ExportFormatV15::Json).unwrap(),
            "\"json\""
        );
        assert_eq!(
            serde_json::to_string(&ExportFormatV15::Csv).unwrap(),
            "\"csv\""
        );
    }

    #[test]
    fn test_anomaly_type_v15_serialization() {
        assert_eq!(
            serde_json::to_string(&AnomalyTypeV15::HighActivity).unwrap(),
            "\"high_activity\""
        );
        assert_eq!(
            serde_json::to_string(&AnomalyTypeV15::ComplianceDrift).unwrap(),
            "\"compliance_drift\""
        );
        assert_eq!(
            serde_json::to_string(&AnomalyTypeV15::GeographicAnomaly).unwrap(),
            "\"geographic_anomaly\""
        );
    }

    #[test]
    fn test_risk_level_v15_serialization() {
        assert_eq!(
            serde_json::to_string(&RiskLevelV15::Critical).unwrap(),
            "\"critical\""
        );
        assert_eq!(
            serde_json::to_string(&RiskLevelV15::Low).unwrap(),
            "\"low\""
        );
    }
}
