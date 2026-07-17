#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- Core Event Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
}

impl GeoLocation {
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

impl Default for GeoLocation {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
    Partial,
    Unknown,
}

impl ComplianceStatus {
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
pub struct AuditTrailEvent {
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
    pub geo_location: Option<GeoLocation>,
    pub risk_score: u32,
    pub compliance_status: ComplianceStatus,
    pub created_at: DateTime<Utc>,
}

impl AuditTrailEvent {
    pub fn new(event_type: &str, resource_type: &str, resource_id: &str, action: &str) -> Self {
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
            compliance_status: ComplianceStatus::Unknown,
            created_at: Utc::now(),
        }
    }

    pub fn with_risk_score(mut self, score: u32) -> Self {
        self.risk_score = score;
        self
    }

    pub fn with_compliance_status(mut self, status: ComplianceStatus) -> Self {
        self.compliance_status = status;
        self
    }

    pub fn is_high_risk(&self, threshold: u32) -> bool {
        self.risk_score >= threshold
    }

    pub fn is_compliant(&self) -> bool {
        self.compliance_status == ComplianceStatus::Compliant
    }
}

// --- Query Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailQuery {
    pub event_type: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub actor_id: Option<String>,
    pub action: Option<String>,
    pub request_id: Option<String>,
    pub session_id: Option<String>,
    pub compliance_status: Option<ComplianceStatus>,
    pub min_risk_score: Option<u32>,
    pub max_risk_score: Option<u32>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: u32,
    pub offset: u32,
}

impl Default for AuditTrailQuery {
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

impl AuditTrailQuery {
    pub fn matches(&self, event: &AuditTrailEvent) -> bool {
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

// --- Export Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditExport {
    pub format: ExportFormat,
    pub total_events: usize,
    pub events: Vec<AuditTrailEvent>,
    pub exported_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Json,
    Csv,
}

// --- Builder ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailBuilder {
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
    geo_location: Option<GeoLocation>,
    risk_score: u32,
    compliance_status: ComplianceStatus,
}

impl AuditTrailBuilder {
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
            compliance_status: ComplianceStatus::Unknown,
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

    pub fn geo_location(mut self, geo: GeoLocation) -> Self {
        self.geo_location = Some(geo);
        self
    }

    pub fn risk_score(mut self, score: u32) -> Self {
        self.risk_score = score;
        self
    }

    pub fn compliance_status(mut self, status: ComplianceStatus) -> Self {
        self.compliance_status = status;
        self
    }

    pub fn build(self) -> AuditTrailEvent {
        AuditTrailEvent {
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

// --- Recorder ---

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditTrailRecorder {
    events: std::sync::Mutex<Vec<AuditTrailEvent>>,
}

impl AuditTrailRecorder {
    pub fn new() -> Self {
        Self {
            events: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn record(&self, event: AuditTrailEvent) {
        let mut events = self.events.lock().unwrap();
        events.push(event);
    }

    pub fn search(&self, query: &AuditTrailQuery) -> Vec<AuditTrailEvent> {
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

    pub fn get_session_events(&self, session_id: &str) -> Vec<AuditTrailEvent> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.session_id.as_deref() == Some(session_id))
            .cloned()
            .collect()
    }

    pub fn get_request_events(&self, request_id: &str) -> Vec<AuditTrailEvent> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.request_id.as_deref() == Some(request_id))
            .cloned()
            .collect()
    }

    pub fn get_events_by_geo_country(&self, country: &str) -> Vec<AuditTrailEvent> {
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

    pub fn get_events_by_compliance_status(
        &self,
        status: ComplianceStatus,
    ) -> Vec<AuditTrailEvent> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.compliance_status == status)
            .cloned()
            .collect()
    }

    pub fn get_high_risk_events(&self, threshold: u32) -> Vec<AuditTrailEvent> {
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
    ) -> ComplianceAuditResult {
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
            .filter(|e| e.compliance_status == ComplianceStatus::Compliant)
            .count() as u32;

        let compliance_rate = if relevant.is_empty() {
            100.0
        } else {
            (compliant_count as f64 / relevant.len() as f64) * 100.0
        };

        ComplianceAuditResult {
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
    ) -> ForensicsExport {
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

        ForensicsExport {
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

    pub fn detect_anomalies(&self) -> Vec<AnomalyDetectionResult> {
        let events = self.events.lock().unwrap();
        let mut anomalies = Vec::new();

        let mut actor_event_counts: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();
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
                anomalies.push(AnomalyDetectionResult {
                    anomaly_type: AnomalyType::HighActivity,
                    actor_id: Some(actor.clone()),
                    description: format!(
                        "Actor {} has {} events, which is {:.1}x the average ({:.0})",
                        actor,
                        count,
                        *count as f64 / avg_events_per_actor.max(1.0),
                        avg_events_per_actor
                    ),
                    risk_level: if *count as f64 > avg_events_per_actor * 5.0 {
                        RiskLevel::Critical
                    } else if *count as f64 > avg_events_per_actor * 3.0 {
                        RiskLevel::High
                    } else {
                        RiskLevel::Medium
                    },
                    detected_at: Utc::now(),
                });
            }
        }

        let mut failed_count = 0u32;
        let mut total_count = 0u32;
        for event in events.iter() {
            if event.compliance_status == ComplianceStatus::NonCompliant {
                failed_count += 1;
            }
            total_count += 1;
        }

        if total_count > 10 && (failed_count as f64 / total_count as f64) > 0.3 {
            anomalies.push(AnomalyDetectionResult {
                anomaly_type: AnomalyType::ComplianceDrift,
                actor_id: None,
                description: format!(
                    "High non-compliance rate: {:.1}% ({}/{})",
                    (failed_count as f64 / total_count as f64) * 100.0,
                    failed_count,
                    total_count
                ),
                risk_level: RiskLevel::High,
                detected_at: Utc::now(),
            });
        }

        let mut ip_counts: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();
        let mut actor_ips: std::collections::HashMap<String, std::collections::HashSet<String>> =
            std::collections::HashMap::new();
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
                anomalies.push(AnomalyDetectionResult {
                    anomaly_type: AnomalyType::UnusualAccess,
                    actor_id: Some(actor.clone()),
                    description: format!(
                        "Actor {} accessed from {} different IP addresses",
                        actor,
                        ips.len()
                    ),
                    risk_level: if ips.len() > 10 {
                        RiskLevel::Critical
                    } else if ips.len() > 5 {
                        RiskLevel::High
                    } else {
                        RiskLevel::Medium
                    },
                    detected_at: Utc::now(),
                });
            }
        }

        anomalies
    }

    pub fn risk_assessment(&self) -> RiskAssessmentResult {
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
            RiskLevel::Critical
        } else if avg_risk >= 50.0 {
            RiskLevel::High
        } else if avg_risk >= 25.0 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        RiskAssessmentResult {
            total_events: total,
            high_risk_count: high_risk,
            medium_risk_count: medium_risk,
            low_risk_count: low_risk,
            avg_risk_score: avg_risk,
            max_risk_score: max_risk,
            risk_level,
        }
    }

    pub fn export(&self, query: &AuditTrailQuery) -> AuditExport {
        let events = self.search(query);
        AuditExport {
            format: ExportFormat::Json,
            total_events: events.len(),
            events,
            exported_at: Utc::now(),
        }
    }
}

impl Default for AuditTrailRecorder {
    fn default() -> Self {
        Self::new()
    }
}

// --- Result Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAuditResult {
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
pub struct ForensicsExport {
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
    pub events: Vec<AuditTrailEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessmentResult {
    pub total_events: u32,
    pub high_risk_count: u32,
    pub medium_risk_count: u32,
    pub low_risk_count: u32,
    pub avg_risk_score: f64,
    pub max_risk_score: u32,
    pub risk_level: RiskLevel,
}

// --- Enums ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyType {
    HighActivity,
    ComplianceDrift,
    UnusualAccess,
    RiskEscalation,
    GeographicAnomaly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
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
pub struct AnomalyDetectionResult {
    pub anomaly_type: AnomalyType,
    pub actor_id: Option<String>,
    pub description: String,
    pub risk_level: RiskLevel,
    pub detected_at: DateTime<Utc>,
}

// --- Helper Functions ---

pub fn record_compliance_event(
    recorder: &AuditTrailRecorder,
    repo_id: &str,
    action: &str,
    framework_name: &str,
    details: serde_json::Value,
    risk_score: u32,
    compliance_status: ComplianceStatus,
) {
    let event = AuditTrailBuilder::new("compliance", "repository", repo_id, action)
        .details(serde_json::json!({
            "framework": framework_name,
            "details": details,
        }))
        .risk_score(risk_score)
        .compliance_status(compliance_status)
        .build();
    recorder.record(event);
}

pub fn record_security_event(
    recorder: &AuditTrailRecorder,
    repo_id: &str,
    action: &str,
    scan_type: &str,
    details: serde_json::Value,
    risk_score: u32,
    compliance_status: ComplianceStatus,
) {
    let event = AuditTrailBuilder::new("security", "repository", repo_id, action)
        .details(serde_json::json!({
            "scan_type": scan_type,
            "details": details,
        }))
        .risk_score(risk_score)
        .compliance_status(compliance_status)
        .build();
    recorder.record(event);
}

// --- Forensic Analysis Types (from v23) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventCategory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: AuditCategorySeverity,
    pub retention_days: u32,
    pub created_at: DateTime<Utc>,
}

impl AuditEventCategory {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            severity: AuditCategorySeverity::Info,
            retention_days: 365,
            created_at: Utc::now(),
        }
    }

    pub fn with_severity(mut self, severity: AuditCategorySeverity) -> Self {
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
pub enum AuditCategorySeverity {
    Info,
    Warning,
    Critical,
}

impl AuditCategorySeverity {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Warning => "Warning",
            Self::Critical => "Critical",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventCorrelation {
    pub id: String,
    pub event_id: String,
    pub correlated_event_id: String,
    pub correlation_type: CorrelationType,
    pub created_at: DateTime<Utc>,
}

impl AuditEventCorrelation {
    pub fn new(event_id: String, correlated_event_id: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_id,
            correlated_event_id,
            correlation_type: CorrelationType::Related,
            created_at: Utc::now(),
        }
    }

    pub fn with_type(mut self, correlation_type: CorrelationType) -> Self {
        self.correlation_type = correlation_type;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrelationType {
    Related,
    Causal,
    Temporal,
    Actor,
    Resource,
}

impl CorrelationType {
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
pub struct AnomalyDetectionRule {
    pub id: String,
    pub name: String,
    pub anomaly_type: AnomalyType,
    pub threshold: f64,
    pub window_minutes: u32,
    pub enabled: bool,
    pub severity: RiskLevel,
}

impl AnomalyDetectionRule {
    pub fn new(
        name: String,
        anomaly_type: AnomalyType,
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
            severity: RiskLevel::Medium,
        }
    }

    pub fn with_severity(mut self, severity: RiskLevel) -> Self {
        self.severity = severity;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicTimelineEntry {
    pub timestamp: DateTime<Utc>,
    pub event: AuditTrailEvent,
    pub correlation_ids: Vec<String>,
    pub risk_level: RiskLevel,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicAnalysisResult {
    pub actor_id: String,
    pub analysis_period_start: DateTime<Utc>,
    pub analysis_period_end: DateTime<Utc>,
    pub timeline: Vec<ForensicTimelineEntry>,
    pub anomalies_detected: Vec<AnomalyDetectionResult>,
    pub risk_summary: ForensicRiskSummary,
    pub recommendations: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicRiskSummary {
    pub total_events: u32,
    pub high_risk_events: u32,
    pub medium_risk_events: u32,
    pub low_risk_events: u32,
    pub average_risk_score: f64,
    pub max_risk_score: u32,
    pub risk_trend: RiskTrend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskTrend {
    Increasing,
    Decreasing,
    Stable,
}

impl RiskTrend {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Increasing => "Increasing",
            Self::Decreasing => "Decreasing",
            Self::Stable => "Stable",
        }
    }
}

// --- Engines ---

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditEventCategoryManager {
    categories: Vec<AuditEventCategory>,
}

impl AuditEventCategoryManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_category(&mut self, category: AuditEventCategory) {
        self.categories.push(category);
    }

    pub fn get_category(&self, id: &str) -> Option<&AuditEventCategory> {
        self.categories.iter().find(|c| c.id == id)
    }

    pub fn get_category_by_name(&self, name: &str) -> Option<&AuditEventCategory> {
        self.categories.iter().find(|c| c.name == name)
    }

    pub fn list_categories(&self) -> &[AuditEventCategory] {
        &self.categories
    }

    pub fn categories_by_severity(
        &self,
        severity: AuditCategorySeverity,
    ) -> Vec<&AuditEventCategory> {
        self.categories
            .iter()
            .filter(|c| c.severity == severity)
            .collect()
    }

    pub fn disable_category(&mut self, id: &str) -> Result<(), String> {
        self.categories.retain(|c| c.id != id);
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventCorrelationEngine {
    correlations: Vec<AuditEventCorrelation>,
    by_event: HashMap<String, Vec<usize>>,
    by_correlated: HashMap<String, Vec<usize>>,
}

impl EventCorrelationEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_correlation(&mut self, correlation: AuditEventCorrelation) {
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

    pub fn get_correlations_for_event(&self, event_id: &str) -> Vec<&AuditEventCorrelation> {
        self.by_event
            .get(event_id)
            .map(|indices| indices.iter().map(|&idx| &self.correlations[idx]).collect())
            .unwrap_or_default()
    }

    pub fn get_correlated_events(&self, event_id: &str) -> Vec<&AuditEventCorrelation> {
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
pub struct AnomalyDetectionEngine {
    rules: Vec<AnomalyDetectionRule>,
}

impl AnomalyDetectionEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_rule(&mut self, rule: AnomalyDetectionRule) {
        self.rules.push(rule);
    }

    pub fn get_enabled_rules(&self) -> Vec<&AnomalyDetectionRule> {
        self.rules.iter().filter(|r| r.enabled).collect()
    }

    pub fn detect(&self, recorder: &AuditTrailRecorder) -> Vec<AnomalyDetectionResult> {
        let mut anomalies = Vec::new();
        let base_anomalies = recorder.detect_anomalies();
        anomalies.extend(base_anomalies);

        let enabled_rules = self.get_enabled_rules();
        if enabled_rules.is_empty() {
            return anomalies;
        }

        let events = recorder.search(&AuditTrailQuery::default());
        let now = Utc::now();

        for rule in &enabled_rules {
            let window_start = now - chrono::Duration::minutes(rule.window_minutes as i64);
            let recent_events: Vec<_> = events
                .iter()
                .filter(|e| e.created_at >= window_start)
                .collect();

            let rate = recent_events.len() as f64 / rule.window_minutes as f64;
            if rate > rule.threshold {
                anomalies.push(AnomalyDetectionResult {
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

    pub fn list_rules(&self) -> &[AnomalyDetectionRule] {
        &self.rules
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForensicAnalysisEngine {
    timeline: Vec<ForensicTimelineEntry>,
}

impl ForensicAnalysisEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build_timeline(
        &mut self,
        recorder: &AuditTrailRecorder,
        actor_id: &str,
        since: DateTime<Utc>,
        until: DateTime<Utc>,
        correlation_engine: &EventCorrelationEngine,
    ) {
        self.timeline.clear();
        let events = recorder.search(&AuditTrailQuery {
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
                RiskLevel::Critical
            } else if event.risk_score >= 50 {
                RiskLevel::High
            } else if event.risk_score >= 25 {
                RiskLevel::Medium
            } else {
                RiskLevel::Low
            };

            self.timeline.push(ForensicTimelineEntry {
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
        anomaly_engine: &AnomalyDetectionEngine,
        recorder: &AuditTrailRecorder,
    ) -> ForensicAnalysisResult {
        let total = self.timeline.len() as u32;
        let high_risk = self
            .timeline
            .iter()
            .filter(|e| e.risk_level == RiskLevel::Critical || e.risk_level == RiskLevel::High)
            .count() as u32;
        let medium_risk = self
            .timeline
            .iter()
            .filter(|e| e.risk_level == RiskLevel::Medium)
            .count() as u32;
        let low_risk = self
            .timeline
            .iter()
            .filter(|e| e.risk_level == RiskLevel::Low)
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
                RiskTrend::Increasing
            } else if second_half_avg < first_half_avg * 0.9 {
                RiskTrend::Decreasing
            } else {
                RiskTrend::Stable
            }
        } else {
            RiskTrend::Stable
        };

        let anomalies = anomaly_engine.detect(recorder);
        let mut recommendations = Vec::new();

        if high_risk > 0 {
            recommendations.push(format!(
                "Review {} high-risk events for potential security incidents",
                high_risk
            ));
        }
        if risk_trend == RiskTrend::Increasing {
            recommendations.push("Risk trend is increasing - investigate recent changes".into());
        }
        if anomalies
            .iter()
            .any(|a| a.risk_level == RiskLevel::Critical)
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

        ForensicAnalysisResult {
            actor_id: actor_id.into(),
            analysis_period_start: period_start,
            analysis_period_end: period_end,
            timeline: self.timeline.clone(),
            anomalies_detected: anomalies,
            risk_summary: ForensicRiskSummary {
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

    pub fn timeline(&self) -> &[ForensicTimelineEntry] {
        &self.timeline
    }
}

// --- Risk Scoring Types (from v24) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRiskScore {
    pub id: String,
    pub event_id: String,
    pub risk_score: f64,
    pub risk_factors: Vec<RiskFactor>,
    pub mitigation_suggestions: Vec<MitigationSuggestion>,
    pub scored_at: DateTime<Utc>,
}

impl EventRiskScore {
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

    pub fn with_factors(mut self, factors: Vec<RiskFactor>) -> Self {
        self.risk_factors = factors;
        self
    }

    pub fn with_mitigations(mut self, mitigations: Vec<MitigationSuggestion>) -> Self {
        self.mitigation_suggestions = mitigations;
        self
    }

    pub fn risk_level(&self) -> RiskLevel {
        if self.risk_score >= 75.0 {
            RiskLevel::Critical
        } else if self.risk_score >= 50.0 {
            RiskLevel::High
        } else if self.risk_score >= 25.0 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    pub fn has_critical_factors(&self) -> bool {
        self.risk_factors
            .iter()
            .any(|f| f.severity == RiskLevel::Critical)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub name: String,
    pub description: String,
    pub severity: RiskLevel,
    pub weight: f64,
}

impl RiskFactor {
    pub fn new(name: String, description: String, severity: RiskLevel, weight: f64) -> Self {
        Self {
            name,
            description,
            severity,
            weight: weight.clamp(0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationSuggestion {
    pub title: String,
    pub description: String,
    pub priority: MitigationPriority,
    pub estimated_effort: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MitigationPriority {
    Immediate,
    High,
    Medium,
    Low,
}

impl MitigationPriority {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Immediate => "immediate",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

impl MitigationSuggestion {
    pub fn new(title: String, description: String, priority: MitigationPriority) -> Self {
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

// --- Retention Policy Types (from v24) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub id: String,
    pub event_category: String,
    pub retention_days: u32,
    pub archive_after_days: Option<u32>,
    pub delete_after_days: Option<u32>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

impl RetentionPolicy {
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

// --- Forensic Timeline Types (from v24) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicTimelineEntryV24 {
    pub timestamp: DateTime<Utc>,
    pub event: AuditTrailEvent,
    pub risk_score: Option<EventRiskScore>,
    pub correlation_ids: Vec<String>,
    pub risk_level: RiskLevel,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicTimeline {
    pub actor_id: String,
    pub entries: Vec<ForensicTimelineEntryV24>,
    pub time_span_start: Option<DateTime<Utc>>,
    pub time_span_end: Option<DateTime<Utc>>,
    pub total_events: u32,
    pub high_risk_count: u32,
    pub average_risk_score: f64,
    pub risk_trend: RiskTrend,
}

impl ForensicTimeline {
    pub fn new(actor_id: String) -> Self {
        Self {
            actor_id,
            entries: Vec::new(),
            time_span_start: None,
            time_span_end: None,
            total_events: 0,
            high_risk_count: 0,
            average_risk_score: 0.0,
            risk_trend: RiskTrend::Stable,
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
            .filter(|e| e.risk_level == RiskLevel::Critical || e.risk_level == RiskLevel::High)
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
                RiskTrend::Increasing
            } else if second_half < first_half * 0.9 {
                RiskTrend::Decreasing
            } else {
                RiskTrend::Stable
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
            .filter(|e| e.risk_level == RiskLevel::Critical || e.risk_level == RiskLevel::High)
            .collect()
    }
}

// --- Compliance Reporting Types (from v24) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAuditReport {
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
    pub findings: Vec<ComplianceAuditFinding>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAuditFinding {
    pub finding_type: AuditFindingType,
    pub description: String,
    pub severity: RiskLevel,
    pub event_count: u32,
    pub recommendation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditFindingType {
    UnscoredEvents,
    HighRiskConcentration,
    RetentionViolation,
    AnomalousPattern,
    MissingMitigation,
}

impl AuditFindingType {
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

impl ComplianceAuditReport {
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

    pub fn add_finding(&mut self, finding: ComplianceAuditFinding) {
        self.findings.push(finding);
    }

    pub fn has_critical_findings(&self) -> bool {
        self.findings
            .iter()
            .any(|f| f.severity == RiskLevel::Critical)
    }
}

// --- Engines (from v24) ---

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventRiskScoringEngine {
    scores: Vec<EventRiskScore>,
    by_event: HashMap<String, Vec<usize>>,
}

impl EventRiskScoringEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn score_event(
        &mut self,
        event: &AuditTrailEvent,
        custom_factors: Vec<RiskFactor>,
    ) -> EventRiskScore {
        let mut base_score = event.risk_score as f64;
        let mut factors = Vec::new();

        if event.risk_score >= 75 {
            factors.push(RiskFactor::new(
                "high_base_risk".into(),
                "Event has high base risk score".into(),
                RiskLevel::Critical,
                0.4,
            ));
            base_score += 15.0;
        }

        factors.extend(custom_factors);
        for factor in &factors {
            base_score += factor.weight * 20.0;
        }

        let score = EventRiskScore::new(event.id.clone(), base_score.min(100.0))
            .with_factors(factors);

        let idx = self.scores.len();
        self.by_event
            .entry(score.event_id.clone())
            .or_default()
            .push(idx);
        self.scores.push(score.clone());
        score
    }

    pub fn get_score_for_event(&self, event_id: &str) -> Option<&EventRiskScore> {
        self.by_event
            .get(event_id)
            .and_then(|indices| indices.first().map(|&idx| &self.scores[idx]))
    }

    pub fn high_risk_scores(&self) -> Vec<&EventRiskScore> {
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
pub struct RetentionPolicyManager {
    policies: Vec<RetentionPolicy>,
    by_category: HashMap<String, Vec<usize>>,
}

impl RetentionPolicyManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_policy(&mut self, policy: RetentionPolicy) {
        let idx = self.policies.len();
        self.by_category
            .entry(policy.event_category.clone())
            .or_default()
            .push(idx);
        self.policies.push(policy);
    }

    pub fn get_policy_for_category(&self, category: &str) -> Option<&RetentionPolicy> {
        self.by_category
            .get(category)
            .and_then(|indices| {
                indices
                    .iter()
                    .map(|&idx| &self.policies[idx])
                    .find(|p| p.enabled)
            })
    }

    pub fn get_all_policies(&self) -> &[RetentionPolicy] {
        &self.policies
    }

    pub fn get_enabled_policies(&self) -> Vec<&RetentionPolicy> {
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
pub struct ForensicTimelineBuilder {
    timelines: HashMap<String, ForensicTimeline>,
}

impl ForensicTimelineBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_create_timeline(&mut self, actor_id: &str) -> &mut ForensicTimeline {
        self.timelines
            .entry(actor_id.into())
            .or_insert_with(|| ForensicTimeline::new(actor_id.into()))
    }

    pub fn get_timeline(&self, actor_id: &str) -> Option<&ForensicTimeline> {
        self.timelines.get(actor_id)
    }

    pub fn all_timelines(&self) -> &HashMap<String, ForensicTimeline> {
        &self.timelines
    }

    pub fn actor_count(&self) -> usize {
        self.timelines.len()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplianceReportingEngine {
    reports: Vec<ComplianceAuditReport>,
}

impl ComplianceReportingEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate_report(
        &mut self,
        framework_id: &str,
        framework_name: &str,
        risk_scores: &EventRiskScoringEngine,
        policies: &RetentionPolicyManager,
    ) -> ComplianceAuditReport {
        let mut report = ComplianceAuditReport::new(framework_id.into(), framework_name.into());

        report.risk_scores_counted = risk_scores.total_scored() as u32;
        report.average_risk_score = risk_scores.average_score();
        report.total_events_audited = risk_scores.total_scored() as u32;
        report.critical_events = risk_scores.high_risk_scores().len() as u32;
        report.retention_policies_applied = policies.get_enabled_policies().len() as u32;

        let unscored = report.total_events_audited - report.risk_scores_counted;
        if unscored > 0 {
            report.add_finding(ComplianceAuditFinding {
                finding_type: AuditFindingType::UnscoredEvents,
                description: format!("{} events have not been risk-scored", unscored),
                severity: RiskLevel::Medium,
                event_count: unscored,
                recommendation: "Apply risk scoring to all audit events".into(),
            });
        }

        if report.critical_events > 0 {
            report.add_finding(ComplianceAuditFinding {
                finding_type: AuditFindingType::HighRiskConcentration,
                description: format!("{} critical/high-risk events detected", report.critical_events),
                severity: RiskLevel::High,
                event_count: report.critical_events,
                recommendation: "Review critical events and apply mitigations".into(),
            });
        }

        self.reports.push(report.clone());
        report
    }

    pub fn get_reports(&self) -> &[ComplianceAuditReport] {
        &self.reports
    }

    pub fn latest_report(&self, framework_id: &str) -> Option<&ComplianceAuditReport> {
        self.reports
            .iter()
            .filter(|r| r.framework_id == framework_id)
            .max_by_key(|r| r.generated_at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(
        event_type: &str,
        resource_type: &str,
        resource_id: &str,
        action: &str,
    ) -> AuditTrailEvent {
        AuditTrailEvent {
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
            compliance_status: ComplianceStatus::Unknown,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_record_event() {
        let recorder = AuditTrailRecorder::new();
        let event = make_event("access", "repo", "r1", "read");
        recorder.record(event);
        assert_eq!(recorder.count(), 1);
    }

    #[test]
    fn test_search_by_event_type() {
        let recorder = AuditTrailRecorder::new();
        recorder.record(make_event("access", "repo", "r1", "read"));
        recorder.record(make_event("security", "repo", "r1", "scan"));
        let query = AuditTrailQuery {
            event_type: Some("access".into()),
            ..AuditTrailQuery::default()
        };
        let results = recorder.search(&query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].event_type, "access");
    }

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
    fn test_retention_policy_new() {
        let policy = RetentionPolicy::new("security".into(), 365);
        assert!(policy.enabled);
        assert!(policy.is_expired(366));
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
    fn test_forensic_timeline_new() {
        let timeline = ForensicTimeline::new("user-1".into());
        assert_eq!(timeline.actor_id, "user-1");
        assert_eq!(timeline.total_events, 0);
    }

    #[test]
    fn test_compliance_audit_report_new() {
        let report = ComplianceAuditReport::new("fw-1".into(), "SOC 2".into());
        assert_eq!(report.framework_id, "fw-1");
        assert!(!report.has_critical_findings());
    }

    #[test]
    fn test_scoring_engine_score_event() {
        let mut engine = EventRiskScoringEngine::new();
        let score = engine.score_event(&make_event("push", "repo", "r1", "push"), Vec::new());
        assert!(score.risk_score > 0.0);
        assert_eq!(engine.total_scored(), 1);
    }

    #[test]
    fn test_retention_manager_add_policy() {
        let mut manager = RetentionPolicyManager::new();
        manager.add_policy(RetentionPolicy::new("security".into(), 365));
        assert!(manager.get_policy_for_category("security").is_some());
    }

    #[test]
    fn test_reporting_engine_generate_report() {
        let mut engine = ComplianceReportingEngine::new();
        let mut scoring = EventRiskScoringEngine::new();
        scoring.score_event(&make_event("push", "repo", "r1", "push"), vec![]);
        let policies = RetentionPolicyManager::new();
        let report = engine.generate_report("fw-1", "SOC 2", &scoring, &policies);
        assert_eq!(report.risk_scores_counted, 1);
        assert!(engine.latest_report("fw-1").is_some());
    }
}
