#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: uuid::Uuid,
    pub timestamp: DateTime<Utc>,
    pub actor_id: String,
    pub actor_type: ActorType,
    pub action: AuditAction,
    pub resource_type: String,
    pub resource_id: String,
    pub outcome: AuditOutcome,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorType {
    User,
    Service,
    System,
    Anonymous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditAction {
    Login,
    Logout,
    Create,
    Read,
    Update,
    Delete,
    Approve,
    Reject,
    Export,
    Import,
    Configure,
    SystemEvent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditOutcome {
    Success,
    Failure,
    Denied,
}

pub struct AuditQueryBuilder {
    actor_id: Option<String>,
    action: Option<AuditAction>,
    resource_type: Option<String>,
    resource_id: Option<String>,
    outcome: Option<AuditOutcome>,
    since: Option<DateTime<Utc>>,
    until: Option<DateTime<Utc>>,
    #[allow(dead_code)]
    limit: u32,
    #[allow(dead_code)]
    offset: u32,
}

impl AuditQueryBuilder {
    pub fn new() -> Self {
        Self {
            actor_id: None,
            action: None,
            resource_type: None,
            resource_id: None,
            outcome: None,
            since: None,
            until: None,
            limit: 100,
            offset: 0,
        }
    }

    pub fn actor_id(mut self, id: &str) -> Self {
        self.actor_id = Some(id.to_string());
        self
    }

    pub fn action(mut self, action: AuditAction) -> Self {
        self.action = Some(action);
        self
    }

    pub fn resource_type(mut self, rt: &str) -> Self {
        self.resource_type = Some(rt.to_string());
        self
    }

    pub fn resource_id(mut self, id: &str) -> Self {
        self.resource_id = Some(id.to_string());
        self
    }

    pub fn outcome(mut self, outcome: AuditOutcome) -> Self {
        self.outcome = Some(outcome);
        self
    }

    pub fn since(mut self, dt: DateTime<Utc>) -> Self {
        self.since = Some(dt);
        self
    }

    pub fn until(mut self, dt: DateTime<Utc>) -> Self {
        self.until = Some(dt);
        self
    }

    pub fn limit(mut self, n: u32) -> Self {
        self.limit = n;
        self
    }

    pub fn offset(mut self, n: u32) -> Self {
        self.offset = n;
        self
    }

    pub fn matches(&self, event: &AuditEvent) -> bool {
        if let Some(ref actor) = self.actor_id {
            if event.actor_id != *actor {
                return false;
            }
        }
        if let Some(action) = self.action {
            if event.action != action {
                return false;
            }
        }
        if let Some(ref rt) = self.resource_type {
            if event.resource_type != *rt {
                return false;
            }
        }
        if let Some(ref rid) = self.resource_id {
            if event.resource_id != *rid {
                return false;
            }
        }
        if let Some(outcome) = self.outcome {
            if event.outcome != outcome {
                return false;
            }
        }
        if let Some(since) = self.since {
            if event.timestamp < since {
                return false;
            }
        }
        if let Some(until) = self.until {
            if event.timestamp > until {
                return false;
            }
        }
        true
    }
}

impl Default for AuditQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(
        actor_id: &str,
        action: AuditAction,
        resource_type: &str,
        resource_id: &str,
        outcome: AuditOutcome,
        timestamp: DateTime<Utc>,
    ) -> AuditEvent {
        AuditEvent {
            id: uuid::Uuid::new_v4(),
            timestamp,
            actor_id: actor_id.to_string(),
            actor_type: ActorType::User,
            action,
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            outcome,
            details: serde_json::Value::Object(serde_json::Map::new()),
            ip_address: None,
            user_agent: None,
            session_id: None,
        }
    }

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            actor_id: "user-1".to_string(),
            actor_type: ActorType::User,
            action: AuditAction::Login,
            resource_type: "session".to_string(),
            resource_id: "sess-1".to_string(),
            outcome: AuditOutcome::Success,
            details: serde_json::Value::Object(serde_json::Map::new()),
            ip_address: Some("10.0.0.1".to_string()),
            user_agent: Some("test-agent".to_string()),
            session_id: Some("sess-abc".to_string()),
        };
        assert_eq!(event.actor_id, "user-1");
        assert_eq!(event.action, AuditAction::Login);
        assert_eq!(event.outcome, AuditOutcome::Success);
        assert_eq!(event.ip_address.as_deref(), Some("10.0.0.1"));
    }

    #[test]
    fn test_audit_event_serialization_roundtrip() {
        let event = make_event(
            "user-1",
            AuditAction::Create,
            "repo",
            "repo-1",
            AuditOutcome::Success,
            Utc::now(),
        );
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: AuditEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event.id, deserialized.id);
        assert_eq!(event.actor_id, deserialized.actor_id);
        assert_eq!(event.action, deserialized.action);
    }

    #[test]
    fn test_actor_type_serialization() {
        assert_eq!(serde_json::to_string(&ActorType::User).unwrap(), "\"User\"");
        assert_eq!(
            serde_json::to_string(&ActorType::Service).unwrap(),
            "\"Service\""
        );
        assert_eq!(
            serde_json::to_string(&ActorType::System).unwrap(),
            "\"System\""
        );
        assert_eq!(
            serde_json::to_string(&ActorType::Anonymous).unwrap(),
            "\"Anonymous\""
        );
    }

    #[test]
    fn test_actor_type_deserialization() {
        let user: ActorType = serde_json::from_str("\"User\"").unwrap();
        assert_eq!(user, ActorType::User);
        let anon: ActorType = serde_json::from_str("\"Anonymous\"").unwrap();
        assert_eq!(anon, ActorType::Anonymous);
    }

    #[test]
    fn test_audit_action_serialization() {
        assert_eq!(
            serde_json::to_string(&AuditAction::Login).unwrap(),
            "\"Login\""
        );
        assert_eq!(
            serde_json::to_string(&AuditAction::Create).unwrap(),
            "\"Create\""
        );
        assert_eq!(
            serde_json::to_string(&AuditAction::SystemEvent).unwrap(),
            "\"SystemEvent\""
        );
    }

    #[test]
    fn test_audit_outcome_serialization() {
        assert_eq!(
            serde_json::to_string(&AuditOutcome::Success).unwrap(),
            "\"Success\""
        );
        assert_eq!(
            serde_json::to_string(&AuditOutcome::Failure).unwrap(),
            "\"Failure\""
        );
        assert_eq!(
            serde_json::to_string(&AuditOutcome::Denied).unwrap(),
            "\"Denied\""
        );
    }

    #[test]
    fn test_query_builder_default() {
        let qb = AuditQueryBuilder::new();
        let event = make_event(
            "user-1",
            AuditAction::Read,
            "file",
            "f1",
            AuditOutcome::Success,
            Utc::now(),
        );
        assert!(qb.matches(&event));
    }

    #[test]
    fn test_query_by_actor_id() {
        let qb = AuditQueryBuilder::new().actor_id("user-1");
        let match_event = make_event(
            "user-1",
            AuditAction::Read,
            "file",
            "f1",
            AuditOutcome::Success,
            Utc::now(),
        );
        let no_match = make_event(
            "user-2",
            AuditAction::Read,
            "file",
            "f2",
            AuditOutcome::Success,
            Utc::now(),
        );
        assert!(qb.matches(&match_event));
        assert!(!qb.matches(&no_match));
    }

    #[test]
    fn test_query_by_action() {
        let qb = AuditQueryBuilder::new().action(AuditAction::Create);
        let match_event = make_event(
            "user-1",
            AuditAction::Create,
            "repo",
            "r1",
            AuditOutcome::Success,
            Utc::now(),
        );
        let no_match = make_event(
            "user-1",
            AuditAction::Delete,
            "repo",
            "r1",
            AuditOutcome::Success,
            Utc::now(),
        );
        assert!(qb.matches(&match_event));
        assert!(!qb.matches(&no_match));
    }

    #[test]
    fn test_query_by_resource_type() {
        let qb = AuditQueryBuilder::new().resource_type("repo");
        let match_event = make_event(
            "user-1",
            AuditAction::Read,
            "repo",
            "r1",
            AuditOutcome::Success,
            Utc::now(),
        );
        let no_match = make_event(
            "user-1",
            AuditAction::Read,
            "file",
            "f1",
            AuditOutcome::Success,
            Utc::now(),
        );
        assert!(qb.matches(&match_event));
        assert!(!qb.matches(&no_match));
    }

    #[test]
    fn test_query_by_resource_id() {
        let qb = AuditQueryBuilder::new().resource_id("repo-42");
        let match_event = make_event(
            "user-1",
            AuditAction::Read,
            "repo",
            "repo-42",
            AuditOutcome::Success,
            Utc::now(),
        );
        let no_match = make_event(
            "user-1",
            AuditAction::Read,
            "repo",
            "repo-99",
            AuditOutcome::Success,
            Utc::now(),
        );
        assert!(qb.matches(&match_event));
        assert!(!qb.matches(&no_match));
    }

    #[test]
    fn test_query_by_outcome() {
        let qb = AuditQueryBuilder::new().outcome(AuditOutcome::Denied);
        let match_event = make_event(
            "user-1",
            AuditAction::Delete,
            "repo",
            "r1",
            AuditOutcome::Denied,
            Utc::now(),
        );
        let no_match = make_event(
            "user-1",
            AuditAction::Delete,
            "repo",
            "r1",
            AuditOutcome::Success,
            Utc::now(),
        );
        assert!(qb.matches(&match_event));
        assert!(!qb.matches(&no_match));
    }

    #[test]
    fn test_query_by_since() {
        let since = Utc::now() - chrono::Duration::hours(1);
        let qb = AuditQueryBuilder::new().since(since);
        let match_event = make_event(
            "user-1",
            AuditAction::Read,
            "file",
            "f1",
            AuditOutcome::Success,
            Utc::now(),
        );
        let old_event = make_event(
            "user-1",
            AuditAction::Read,
            "file",
            "f1",
            AuditOutcome::Success,
            Utc::now() - chrono::Duration::hours(2),
        );
        assert!(qb.matches(&match_event));
        assert!(!qb.matches(&old_event));
    }

    #[test]
    fn test_query_by_until() {
        let until = Utc::now() - chrono::Duration::hours(1);
        let qb = AuditQueryBuilder::new().until(until);
        let old_event = make_event(
            "user-1",
            AuditAction::Read,
            "file",
            "f1",
            AuditOutcome::Success,
            Utc::now() - chrono::Duration::hours(2),
        );
        let new_event = make_event(
            "user-1",
            AuditAction::Read,
            "file",
            "f1",
            AuditOutcome::Success,
            Utc::now(),
        );
        assert!(qb.matches(&old_event));
        assert!(!qb.matches(&new_event));
    }

    #[test]
    fn test_query_multiple_filters() {
        let qb = AuditQueryBuilder::new()
            .actor_id("admin")
            .action(AuditAction::Delete)
            .outcome(AuditOutcome::Success);
        let match_event = make_event(
            "admin",
            AuditAction::Delete,
            "repo",
            "r1",
            AuditOutcome::Success,
            Utc::now(),
        );
        let wrong_action = make_event(
            "admin",
            AuditAction::Read,
            "repo",
            "r1",
            AuditOutcome::Success,
            Utc::now(),
        );
        let wrong_actor = make_event(
            "user-1",
            AuditAction::Delete,
            "repo",
            "r1",
            AuditOutcome::Success,
            Utc::now(),
        );
        let wrong_outcome = make_event(
            "admin",
            AuditAction::Delete,
            "repo",
            "r1",
            AuditOutcome::Denied,
            Utc::now(),
        );
        assert!(qb.matches(&match_event));
        assert!(!qb.matches(&wrong_action));
        assert!(!qb.matches(&wrong_actor));
        assert!(!qb.matches(&wrong_outcome));
    }

    #[test]
    fn test_query_time_range() {
        let since = Utc::now() - chrono::Duration::days(2);
        let until = Utc::now();
        let qb = AuditQueryBuilder::new().since(since).until(until);
        let in_range = make_event(
            "user-1",
            AuditAction::Read,
            "f",
            "1",
            AuditOutcome::Success,
            Utc::now() - chrono::Duration::days(1),
        );
        let too_old = make_event(
            "user-1",
            AuditAction::Read,
            "f",
            "1",
            AuditOutcome::Success,
            Utc::now() - chrono::Duration::days(5),
        );
        assert!(qb.matches(&in_range));
        assert!(!qb.matches(&too_old));
    }

    #[test]
    fn test_query_with_details_field() {
        let event = AuditEvent {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            actor_id: "svc-1".to_string(),
            actor_type: ActorType::Service,
            action: AuditAction::SystemEvent,
            resource_type: "system".to_string(),
            resource_id: "sys-1".to_string(),
            outcome: AuditOutcome::Success,
            details: serde_json::json!({"reason": "health_check"}),
            ip_address: None,
            user_agent: None,
            session_id: None,
        };
        assert_eq!(event.details["reason"], "health_check");
    }

    #[test]
    fn test_query_limit_and_offset_builders() {
        let qb = AuditQueryBuilder::new().limit(50).offset(10);
        assert_eq!(qb.limit, 50);
        assert_eq!(qb.offset, 10);
    }

    #[test]
    fn test_actor_type_equality() {
        assert_eq!(ActorType::User, ActorType::User);
        assert_ne!(ActorType::User, ActorType::Service);
        assert_ne!(ActorType::System, ActorType::Anonymous);
    }

    #[test]
    fn test_audit_action_equality() {
        assert_eq!(AuditAction::Login, AuditAction::Login);
        assert_ne!(AuditAction::Create, AuditAction::Delete);
        assert_ne!(AuditAction::Approve, AuditAction::Reject);
    }

    #[test]
    fn test_audit_outcome_equality() {
        assert_eq!(AuditOutcome::Success, AuditOutcome::Success);
        assert_ne!(AuditOutcome::Success, AuditOutcome::Failure);
        assert_ne!(AuditOutcome::Failure, AuditOutcome::Denied);
    }
}
