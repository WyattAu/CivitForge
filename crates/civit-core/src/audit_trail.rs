#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
}
