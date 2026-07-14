#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailEventV2 {
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
    pub created_at: DateTime<Utc>,
}

impl AuditTrailEventV2 {
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
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailQueryV2 {
    pub event_type: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub actor_id: Option<String>,
    pub action: Option<String>,
    pub request_id: Option<String>,
    pub session_id: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: u32,
    pub offset: u32,
}

impl Default for AuditTrailQueryV2 {
    fn default() -> Self {
        Self {
            event_type: None,
            resource_type: None,
            resource_id: None,
            actor_id: None,
            action: None,
            request_id: None,
            session_id: None,
            since: None,
            until: None,
            limit: 100,
            offset: 0,
        }
    }
}

impl AuditTrailQueryV2 {
    pub fn matches(&self, event: &AuditTrailEventV2) -> bool {
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
pub struct AuditTrailRecorderV2 {
    events: std::sync::Mutex<Vec<AuditTrailEventV2>>,
}

impl AuditTrailRecorderV2 {
    pub fn new() -> Self {
        Self {
            events: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn record(&self, event: AuditTrailEventV2) {
        let mut events = self.events.lock().unwrap();
        events.push(event);
    }

    pub fn search(&self, query: &AuditTrailQueryV2) -> Vec<AuditTrailEventV2> {
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

    pub fn get_session_events(&self, session_id: &str) -> Vec<AuditTrailEventV2> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.session_id.as_deref() == Some(session_id))
            .cloned()
            .collect()
    }

    pub fn get_request_events(&self, request_id: &str) -> Vec<AuditTrailEventV2> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.request_id.as_deref() == Some(request_id))
            .cloned()
            .collect()
    }

    pub fn export(&self, query: &AuditTrailQueryV2) -> AuditExportV2 {
        let events = self.search(query);
        AuditExportV2 {
            format: ExportFormatV2::Json,
            total_events: events.len(),
            events,
            exported_at: Utc::now(),
        }
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

        ComplianceAuditResult {
            event_type: event_type.into(),
            period_start: since,
            period_end: until,
            total_events: relevant.len() as u32,
            unique_actors: actor_count as u32,
            unique_resources: resource_count as u32,
            action_counts,
        }
    }
}

impl Default for AuditTrailRecorderV2 {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAuditResult {
    pub event_type: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_events: u32,
    pub unique_actors: u32,
    pub unique_resources: u32,
    pub action_counts: std::collections::HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditExportV2 {
    pub format: ExportFormatV2,
    pub total_events: usize,
    pub events: Vec<AuditTrailEventV2>,
    pub exported_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormatV2 {
    Json,
    Csv,
}

pub struct AuditTrailBuilderV2 {
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
}

impl AuditTrailBuilderV2 {
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

    pub fn build(self) -> AuditTrailEventV2 {
        AuditTrailEventV2 {
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
            created_at: Utc::now(),
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
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailQuery {
    pub event_type: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub actor_id: Option<String>,
    pub action: Option<String>,
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

pub struct AuditTrailBuilder {
    event_type: String,
    resource_type: String,
    resource_id: String,
    actor_id: Option<String>,
    action: String,
    details: serde_json::Value,
    ip_address: Option<String>,
    user_agent: Option<String>,
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
            created_at: Utc::now(),
        }
    }
}

pub fn record_compliance_event(
    recorder: &AuditTrailRecorder,
    repo_id: &str,
    action: &str,
    framework_name: &str,
    details: serde_json::Value,
) {
    let event = AuditTrailBuilder::new("compliance", "repository", repo_id, action)
        .details(serde_json::json!({
            "framework": framework_name,
            "details": details,
        }))
        .build();
    recorder.record(event);
}

pub fn record_security_event(
    recorder: &AuditTrailRecorder,
    repo_id: &str,
    action: &str,
    scan_type: &str,
    details: serde_json::Value,
) {
    let event = AuditTrailBuilder::new("security", "repository", repo_id, action)
        .details(serde_json::json!({
            "scan_type": scan_type,
            "details": details,
        }))
        .build();
    recorder.record(event);
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailEventV3 {
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
    pub created_at: DateTime<Utc>,
}

impl AuditTrailEventV3 {
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
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailQueryV3 {
    pub event_type: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub actor_id: Option<String>,
    pub action: Option<String>,
    pub request_id: Option<String>,
    pub session_id: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: u32,
    pub offset: u32,
}

impl Default for AuditTrailQueryV3 {
    fn default() -> Self {
        Self {
            event_type: None,
            resource_type: None,
            resource_id: None,
            actor_id: None,
            action: None,
            request_id: None,
            session_id: None,
            since: None,
            until: None,
            limit: 100,
            offset: 0,
        }
    }
}

impl AuditTrailQueryV3 {
    pub fn matches(&self, event: &AuditTrailEventV3) -> bool {
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
pub struct AuditTrailRecorderV3 {
    events: std::sync::Mutex<Vec<AuditTrailEventV3>>,
}

impl AuditTrailRecorderV3 {
    pub fn new() -> Self {
        Self {
            events: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn record(&self, event: AuditTrailEventV3) {
        let mut events = self.events.lock().unwrap();
        events.push(event);
    }

    pub fn search(&self, query: &AuditTrailQueryV3) -> Vec<AuditTrailEventV3> {
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

    pub fn get_session_events(&self, session_id: &str) -> Vec<AuditTrailEventV3> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.session_id.as_deref() == Some(session_id))
            .cloned()
            .collect()
    }

    pub fn get_request_events(&self, request_id: &str) -> Vec<AuditTrailEventV3> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.request_id.as_deref() == Some(request_id))
            .cloned()
            .collect()
    }

    pub fn get_events_by_geo_country(&self, country: &str) -> Vec<AuditTrailEventV3> {
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

        ComplianceAuditResult {
            event_type: event_type.into(),
            period_start: since,
            period_end: until,
            total_events: relevant.len() as u32,
            unique_actors: actor_count as u32,
            unique_resources: resource_count as u32,
            action_counts,
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

        ForensicsExport {
            actor_id: actor_id.into(),
            period_start: since,
            period_end: until,
            total_events: relevant.len() as u32,
            session_ids,
            ip_addresses,
            event_type_counts,
            events: relevant,
        }
    }

    pub fn export(&self, query: &AuditTrailQueryV3) -> AuditExportV3 {
        let events = self.search(query);
        AuditExportV3 {
            format: ExportFormatV3::Json,
            total_events: events.len(),
            events,
            exported_at: Utc::now(),
        }
    }
}

impl Default for AuditTrailRecorderV3 {
    fn default() -> Self {
        Self::new()
    }
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
    pub events: Vec<AuditTrailEventV3>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditExportV3 {
    pub format: ExportFormatV3,
    pub total_events: usize,
    pub events: Vec<AuditTrailEventV3>,
    pub exported_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormatV3 {
    Json,
    Csv,
}

pub struct AuditTrailBuilderV3 {
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
}

impl AuditTrailBuilderV3 {
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

    pub fn build(self) -> AuditTrailEventV3 {
        AuditTrailEventV3 {
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
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(event_type: &str, resource_type: &str, resource_id: &str, action: &str) -> AuditTrailEvent {
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
    fn test_search_by_resource_type() {
        let recorder = AuditTrailRecorder::new();
        recorder.record(make_event("access", "repo", "r1", "read"));
        recorder.record(make_event("access", "pipeline", "p1", "run"));
        let query = AuditTrailQuery {
            resource_type: Some("repo".into()),
            ..AuditTrailQuery::default()
        };
        let results = recorder.search(&query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].resource_type, "repo");
    }

    #[test]
    fn test_search_by_actor_id() {
        let recorder = AuditTrailRecorder::new();
        let mut e1 = make_event("access", "repo", "r1", "read");
        e1.actor_id = Some("user-1".into());
        let mut e2 = make_event("access", "repo", "r2", "read");
        e2.actor_id = Some("user-2".into());
        recorder.record(e1);
        recorder.record(e2);
        let query = AuditTrailQuery {
            actor_id: Some("user-1".into()),
            ..AuditTrailQuery::default()
        };
        let results = recorder.search(&query);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_by_action() {
        let recorder = AuditTrailRecorder::new();
        recorder.record(make_event("access", "repo", "r1", "read"));
        recorder.record(make_event("access", "repo", "r1", "write"));
        let query = AuditTrailQuery {
            action: Some("read".into()),
            ..AuditTrailQuery::default()
        };
        let results = recorder.search(&query);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_by_time_range() {
        let recorder = AuditTrailRecorder::new();
        recorder.record(make_event("access", "repo", "r1", "read"));
        let query = AuditTrailQuery {
            since: Some(Utc::now() + chrono::Duration::hours(1)),
            ..AuditTrailQuery::default()
        };
        let results = recorder.search(&query);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_limit() {
        let recorder = AuditTrailRecorder::new();
        for _ in 0..10 {
            recorder.record(make_event("access", "repo", "r1", "read"));
        }
        let query = AuditTrailQuery {
            limit: 5,
            ..AuditTrailQuery::default()
        };
        let results = recorder.search(&query);
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_search_offset() {
        let recorder = AuditTrailRecorder::new();
        for _ in 0..10 {
            recorder.record(make_event("access", "repo", "r1", "read"));
        }
        let query = AuditTrailQuery {
            limit: 5,
            offset: 5,
            ..AuditTrailQuery::default()
        };
        let results = recorder.search(&query);
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_export() {
        let recorder = AuditTrailRecorder::new();
        recorder.record(make_event("access", "repo", "r1", "read"));
        let export = recorder.export(&AuditTrailQuery::default());
        assert_eq!(export.total_events, 1);
        assert_eq!(export.format, ExportFormat::Json);
    }

    #[test]
    fn test_builder() {
        let event = AuditTrailBuilder::new("access", "repo", "r1", "read")
            .actor_id("user-1")
            .ip_address("10.0.0.1")
            .user_agent("test-agent")
            .details(serde_json::json!({"key": "value"}))
            .build();
        assert_eq!(event.event_type, "access");
        assert_eq!(event.actor_id.as_deref(), Some("user-1"));
        assert_eq!(event.ip_address.as_deref(), Some("10.0.0.1"));
        assert_eq!(event.details["key"], "value");
    }

    #[test]
    fn test_record_compliance_event() {
        let recorder = AuditTrailRecorder::new();
        record_compliance_event(
            &recorder,
            "repo-1",
            "assessment_complete",
            "SOC 2",
            serde_json::json!({"score": 100}),
        );
        assert_eq!(recorder.count(), 1);
        let query = AuditTrailQuery {
            event_type: Some("compliance".into()),
            ..AuditTrailQuery::default()
        };
        let results = recorder.search(&query);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].resource_id, "repo-1");
    }

    #[test]
    fn test_record_security_event() {
        let recorder = AuditTrailRecorder::new();
        record_security_event(
            &recorder,
            "repo-1",
            "scan_completed",
            "sast",
            serde_json::json!({"score": 100}),
        );
        assert_eq!(recorder.count(), 1);
        let query = AuditTrailQuery {
            event_type: Some("security".into()),
            ..AuditTrailQuery::default()
        };
        let results = recorder.search(&query);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_multiple_filters() {
        let recorder = AuditTrailRecorder::new();
        let mut e1 = make_event("security", "repo", "r1", "scan");
        e1.actor_id = Some("user-1".into());
        let mut e2 = make_event("security", "repo", "r2", "scan");
        e2.actor_id = Some("user-1".into());
        let mut e3 = make_event("compliance", "repo", "r1", "assess");
        e3.actor_id = Some("user-2".into());
        recorder.record(e1);
        recorder.record(e2);
        recorder.record(e3);
        let query = AuditTrailQuery {
            event_type: Some("security".into()),
            actor_id: Some("user-1".into()),
            ..AuditTrailQuery::default()
        };
        let results = recorder.search(&query);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_export_format_serialization() {
        assert_eq!(
            serde_json::to_string(&ExportFormat::Json).unwrap(),
            "\"json\""
        );
        assert_eq!(
            serde_json::to_string(&ExportFormat::Csv).unwrap(),
            "\"csv\""
        );
    }

    // V2 Tests

    #[test]
    fn test_audit_trail_event_v2_new() {
        let event = AuditTrailEventV2::new("access", "repo", "r1", "read");
        assert_eq!(event.event_type, "access");
        assert_eq!(event.resource_type, "repo");
        assert_eq!(event.resource_id, "r1");
        assert_eq!(event.action, "read");
        assert!(event.request_id.is_none());
        assert!(event.session_id.is_none());
    }

    #[test]
    fn test_audit_trail_query_v2_matches_request_id() {
        let mut event = AuditTrailEventV2::new("access", "repo", "r1", "read");
        event.request_id = Some("req-1".into());
        let query = AuditTrailQueryV2 {
            request_id: Some("req-1".into()),
            ..AuditTrailQueryV2::default()
        };
        assert!(query.matches(&event));
        let query2 = AuditTrailQueryV2 {
            request_id: Some("req-2".into()),
            ..AuditTrailQueryV2::default()
        };
        assert!(!query2.matches(&event));
    }

    #[test]
    fn test_audit_trail_query_v2_matches_session_id() {
        let mut event = AuditTrailEventV2::new("access", "repo", "r1", "read");
        event.session_id = Some("sess-1".into());
        let query = AuditTrailQueryV2 {
            session_id: Some("sess-1".into()),
            ..AuditTrailQueryV2::default()
        };
        assert!(query.matches(&event));
        let query2 = AuditTrailQueryV2 {
            session_id: Some("sess-2".into()),
            ..AuditTrailQueryV2::default()
        };
        assert!(!query2.matches(&event));
    }

    #[test]
    fn test_audit_trail_recorder_v2_record_and_search() {
        let recorder = AuditTrailRecorderV2::new();
        let event = AuditTrailEventV2::new("access", "repo", "r1", "read");
        recorder.record(event);
        assert_eq!(recorder.count(), 1);
        let results = recorder.search(&AuditTrailQueryV2::default());
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_audit_trail_recorder_v2_session_tracking() {
        let recorder = AuditTrailRecorderV2::new();
        let mut e1 = AuditTrailEventV2::new("access", "repo", "r1", "read");
        e1.session_id = Some("sess-1".into());
        let mut e2 = AuditTrailEventV2::new("access", "repo", "r1", "read");
        e2.session_id = Some("sess-1".into());
        let mut e3 = AuditTrailEventV2::new("access", "repo", "r1", "read");
        e3.session_id = Some("sess-2".into());
        recorder.record(e1);
        recorder.record(e2);
        recorder.record(e3);
        assert_eq!(recorder.count_by_session("sess-1"), 2);
        assert_eq!(recorder.count_by_session("sess-2"), 1);
        assert_eq!(recorder.get_session_events("sess-1").len(), 2);
    }

    #[test]
    fn test_audit_trail_recorder_v2_request_events() {
        let recorder = AuditTrailRecorderV2::new();
        let mut e1 = AuditTrailEventV2::new("security", "repo", "r1", "scan");
        e1.request_id = Some("req-1".into());
        let mut e2 = AuditTrailEventV2::new("security", "repo", "r1", "scan");
        e2.request_id = Some("req-1".into());
        let mut e3 = AuditTrailEventV2::new("security", "repo", "r1", "scan");
        e3.request_id = Some("req-2".into());
        recorder.record(e1);
        recorder.record(e2);
        recorder.record(e3);
        assert_eq!(recorder.get_request_events("req-1").len(), 2);
        assert_eq!(recorder.get_request_events("req-2").len(), 1);
    }

    #[test]
    fn test_audit_trail_recorder_v2_compliance_audit() {
        let recorder = AuditTrailRecorderV2::new();
        let mut e1 = AuditTrailEventV2::new("security", "repo", "r1", "scan");
        e1.actor_id = Some("user-1".into());
        let mut e2 = AuditTrailEventV2::new("security", "repo", "r2", "scan");
        e2.actor_id = Some("user-1".into());
        let mut e3 = AuditTrailEventV2::new("security", "repo", "r1", "fix");
        e3.actor_id = Some("user-2".into());
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
    }

    #[test]
    fn test_audit_trail_builder_v2() {
        let event = AuditTrailBuilderV2::new("security", "repo", "r1", "scan")
            .actor_id("user-1")
            .ip_address("10.0.0.1")
            .user_agent("test-agent")
            .request_id("req-1")
            .session_id("sess-1")
            .details(serde_json::json!({"key": "value"}))
            .build();
        assert_eq!(event.event_type, "security");
        assert_eq!(event.actor_id.as_deref(), Some("user-1"));
        assert_eq!(event.ip_address.as_deref(), Some("10.0.0.1"));
        assert_eq!(event.request_id.as_deref(), Some("req-1"));
        assert_eq!(event.session_id.as_deref(), Some("sess-1"));
        assert_eq!(event.details["key"], "value");
    }

    #[test]
    fn test_audit_trail_export_v2() {
        let recorder = AuditTrailRecorderV2::new();
        recorder.record(AuditTrailEventV2::new("access", "repo", "r1", "read"));
        let export = recorder.export(&AuditTrailQueryV2::default());
        assert_eq!(export.total_events, 1);
        assert_eq!(export.format, ExportFormatV2::Json);
    }

    #[test]
    fn test_export_format_v2_serialization() {
        assert_eq!(
            serde_json::to_string(&ExportFormatV2::Json).unwrap(),
            "\"json\""
        );
        assert_eq!(
            serde_json::to_string(&ExportFormatV2::Csv).unwrap(),
            "\"csv\""
        );
    }

    // V3 Tests

    #[test]
    fn test_geo_location_new() {
        let geo = GeoLocation::new();
        assert!(geo.country.is_none());
        assert!(geo.region.is_none());
        assert!(geo.city.is_none());
        assert!(geo.latitude.is_none());
        assert!(geo.longitude.is_none());
    }

    #[test]
    fn test_geo_location_with_fields() {
        let geo = GeoLocation::new()
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
    fn test_audit_trail_event_v3_new() {
        let event = AuditTrailEventV3::new("access", "repo", "r1", "read");
        assert_eq!(event.event_type, "access");
        assert_eq!(event.resource_type, "repo");
        assert_eq!(event.resource_id, "r1");
        assert_eq!(event.action, "read");
        assert!(event.request_id.is_none());
        assert!(event.session_id.is_none());
        assert!(event.geo_location.is_none());
    }

    #[test]
    fn test_audit_trail_query_v3_matches() {
        let mut event = AuditTrailEventV3::new("access", "repo", "r1", "read");
        event.request_id = Some("req-1".into());
        event.session_id = Some("sess-1".into());
        let query = AuditTrailQueryV3 {
            request_id: Some("req-1".into()),
            ..AuditTrailQueryV3::default()
        };
        assert!(query.matches(&event));
        let query2 = AuditTrailQueryV3 {
            request_id: Some("req-2".into()),
            ..AuditTrailQueryV3::default()
        };
        assert!(!query2.matches(&event));
    }

    #[test]
    fn test_audit_trail_recorder_v3_record_and_search() {
        let recorder = AuditTrailRecorderV3::new();
        let event = AuditTrailEventV3::new("access", "repo", "r1", "read");
        recorder.record(event);
        assert_eq!(recorder.count(), 1);
        let results = recorder.search(&AuditTrailQueryV3::default());
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_audit_trail_recorder_v3_session_tracking() {
        let recorder = AuditTrailRecorderV3::new();
        let mut e1 = AuditTrailEventV3::new("access", "repo", "r1", "read");
        e1.session_id = Some("sess-1".into());
        let mut e2 = AuditTrailEventV3::new("access", "repo", "r1", "read");
        e2.session_id = Some("sess-1".into());
        let mut e3 = AuditTrailEventV3::new("access", "repo", "r1", "read");
        e3.session_id = Some("sess-2".into());
        recorder.record(e1);
        recorder.record(e2);
        recorder.record(e3);
        assert_eq!(recorder.count_by_session("sess-1"), 2);
        assert_eq!(recorder.count_by_session("sess-2"), 1);
        assert_eq!(recorder.get_session_events("sess-1").len(), 2);
    }

    #[test]
    fn test_audit_trail_recorder_v3_geo_tracking() {
        let recorder = AuditTrailRecorderV3::new();
        let mut e1 = AuditTrailEventV3::new("access", "repo", "r1", "read");
        e1.geo_location = Some(GeoLocation::new().with_country("US"));
        let mut e2 = AuditTrailEventV3::new("access", "repo", "r2", "read");
        e2.geo_location = Some(GeoLocation::new().with_country("UK"));
        let mut e3 = AuditTrailEventV3::new("access", "repo", "r3", "read");
        e3.geo_location = Some(GeoLocation::new().with_country("US"));
        recorder.record(e1);
        recorder.record(e2);
        recorder.record(e3);
        assert_eq!(recorder.get_events_by_geo_country("US").len(), 2);
        assert_eq!(recorder.get_events_by_geo_country("UK").len(), 1);
    }

    #[test]
    fn test_audit_trail_recorder_v3_compliance_audit() {
        let recorder = AuditTrailRecorderV3::new();
        let mut e1 = AuditTrailEventV3::new("security", "repo", "r1", "scan");
        e1.actor_id = Some("user-1".into());
        let mut e2 = AuditTrailEventV3::new("security", "repo", "r2", "scan");
        e2.actor_id = Some("user-1".into());
        let mut e3 = AuditTrailEventV3::new("security", "repo", "r1", "fix");
        e3.actor_id = Some("user-2".into());
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
    }

    #[test]
    fn test_audit_trail_recorder_v3_forensics_export() {
        let recorder = AuditTrailRecorderV3::new();
        let mut e1 = AuditTrailEventV3::new("access", "repo", "r1", "read");
        e1.actor_id = Some("user-1".into());
        e1.session_id = Some("sess-1".into());
        e1.ip_address = Some("10.0.0.1".into());
        let mut e2 = AuditTrailEventV3::new("access", "repo", "r2", "write");
        e2.actor_id = Some("user-1".into());
        e2.session_id = Some("sess-1".into());
        e2.ip_address = Some("10.0.0.1".into());
        let mut e3 = AuditTrailEventV3::new("access", "repo", "r3", "read");
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
    }

    #[test]
    fn test_audit_trail_builder_v3() {
        let geo = GeoLocation::new()
            .with_country("US")
            .with_city("New York");
        let event = AuditTrailBuilderV3::new("security", "repo", "r1", "scan")
            .actor_id("user-1")
            .ip_address("10.0.0.1")
            .user_agent("test-agent")
            .request_id("req-1")
            .session_id("sess-1")
            .geo_location(geo)
            .details(serde_json::json!({"key": "value"}))
            .build();
        assert_eq!(event.event_type, "security");
        assert_eq!(event.actor_id.as_deref(), Some("user-1"));
        assert_eq!(event.ip_address.as_deref(), Some("10.0.0.1"));
        assert_eq!(event.request_id.as_deref(), Some("req-1"));
        assert_eq!(event.session_id.as_deref(), Some("sess-1"));
        assert!(event.geo_location.is_some());
        assert_eq!(event.geo_location.as_ref().unwrap().country.as_deref(), Some("US"));
        assert_eq!(event.details["key"], "value");
    }

    #[test]
    fn test_audit_trail_export_v3() {
        let recorder = AuditTrailRecorderV3::new();
        recorder.record(AuditTrailEventV3::new("access", "repo", "r1", "read"));
        let export = recorder.export(&AuditTrailQueryV3::default());
        assert_eq!(export.total_events, 1);
        assert_eq!(export.format, ExportFormatV3::Json);
    }

    #[test]
    fn test_export_format_v3_serialization() {
        assert_eq!(
            serde_json::to_string(&ExportFormatV3::Json).unwrap(),
            "\"json\""
        );
        assert_eq!(
            serde_json::to_string(&ExportFormatV3::Csv).unwrap(),
            "\"csv\""
        );
    }
}
