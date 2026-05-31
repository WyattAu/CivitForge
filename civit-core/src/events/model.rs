#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrAction {
    Opened,
    Closed,
    Merged,
    Reopened,
    Approved,
    Commented,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueAction {
    Opened,
    Closed,
    Reopened,
    Assigned,
    Labeled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CiStatus {
    Pending,
    Running,
    Success,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemLevel {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum EventPayload {
    PushEvent {
        repo_id: String,
        branch: String,
        commit_sha: String,
        author: String,
        message: String,
        added: usize,
        removed: usize,
        modified: usize,
    },
    PrEvent {
        repo_id: String,
        pr_number: u32,
        action: PrAction,
        title: String,
        author: String,
        source_branch: String,
        target_branch: String,
    },
    IssueEvent {
        repo_id: String,
        issue_number: u32,
        action: IssueAction,
        title: String,
        author: String,
    },
    CiEvent {
        repo_id: String,
        pipeline_id: String,
        status: CiStatus,
        step: Option<String>,
        duration_ms: Option<u64>,
    },
    FederationEvent {
        remote_instance: String,
        event_type: String,
        payload: String,
    },
    AdminEvent {
        action: String,
        actor: String,
        target_type: String,
        target_id: String,
    },
    SystemEvent {
        level: SystemLevel,
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventCategory {
    Push,
    PullRequest,
    Issue,
    CI,
    Federation,
    Admin,
    System,
}

impl From<&EventPayload> for EventCategory {
    fn from(payload: &EventPayload) -> Self {
        match payload {
            EventPayload::PushEvent { .. } => EventCategory::Push,
            EventPayload::PrEvent { .. } => EventCategory::PullRequest,
            EventPayload::IssueEvent { .. } => EventCategory::Issue,
            EventPayload::CiEvent { .. } => EventCategory::CI,
            EventPayload::FederationEvent { .. } => EventCategory::Federation,
            EventPayload::AdminEvent { .. } => EventCategory::Admin,
            EventPayload::SystemEvent { .. } => EventCategory::System,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub category: EventCategory,
    pub payload: EventPayload,
    pub timestamp: DateTime<Utc>,
    pub source_instance: String,
}

impl Event {
    pub fn new(category: EventCategory, payload: EventPayload, source_instance: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            category,
            payload,
            timestamp: Utc::now(),
            source_instance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_event_roundtrip() {
        let event = Event::new(
            EventCategory::Push,
            EventPayload::PushEvent {
                repo_id: "repo-1".to_string(),
                branch: "main".to_string(),
                commit_sha: "abc123".to_string(),
                author: "alice".to_string(),
                message: "fix typo".to_string(),
                added: 3,
                removed: 1,
                modified: 2,
            },
            "civitforge.local".to_string(),
        );

        let json = serde_json::to_string(&event).expect("serialize");
        let deserialized: Event = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(event.id, deserialized.id);
        assert_eq!(event.category, deserialized.category);
        assert_eq!(event.source_instance, deserialized.source_instance);
    }

    #[test]
    fn pr_event_roundtrip() {
        let event = Event::new(
            EventCategory::PullRequest,
            EventPayload::PrEvent {
                repo_id: "repo-2".to_string(),
                pr_number: 42,
                action: PrAction::Opened,
                title: "Add feature".to_string(),
                author: "bob".to_string(),
                source_branch: "feat/x".to_string(),
                target_branch: "main".to_string(),
            },
            "civitforge.local".to_string(),
        );

        let json = serde_json::to_string(&event).expect("serialize");
        let deserialized: Event = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(event.category, deserialized.category);
    }

    #[test]
    fn ci_event_roundtrip() {
        let event = Event::new(
            EventCategory::CI,
            EventPayload::CiEvent {
                repo_id: "repo-3".to_string(),
                pipeline_id: "pipe-1".to_string(),
                status: CiStatus::Running,
                step: Some("build".to_string()),
                duration_ms: Some(5000),
            },
            "civitforge.local".to_string(),
        );

        let json = serde_json::to_string(&event).expect("serialize");
        let deserialized: Event = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(event.category, deserialized.category);
    }

    #[test]
    fn category_from_payload() {
        let push = EventPayload::PushEvent {
            repo_id: "r".into(),
            branch: "b".into(),
            commit_sha: "s".into(),
            author: "a".into(),
            message: "m".into(),
            added: 0,
            removed: 0,
            modified: 0,
        };
        assert_eq!(EventCategory::from(&push), EventCategory::Push);

        let sys = EventPayload::SystemEvent {
            level: SystemLevel::Info,
            message: "hello".into(),
        };
        assert_eq!(EventCategory::from(&sys), EventCategory::System);
    }

    #[test]
    fn all_action_variants_serializable() {
        let actions = [
            PrAction::Opened,
            PrAction::Closed,
            PrAction::Merged,
            PrAction::Reopened,
            PrAction::Approved,
            PrAction::Commented,
        ];
        for a in &actions {
            let s = serde_json::to_string(a).expect("serialize pr action");
            let back: PrAction = serde_json::from_str(&s).expect("deserialize pr action");
            assert_eq!(*a, back);
        }

        let issue_actions = [
            IssueAction::Opened,
            IssueAction::Closed,
            IssueAction::Reopened,
            IssueAction::Assigned,
            IssueAction::Labeled,
        ];
        for a in &issue_actions {
            let s = serde_json::to_string(a).expect("serialize issue action");
            let back: IssueAction = serde_json::from_str(&s).expect("deserialize issue action");
            assert_eq!(*a, back);
        }

        let ci_statuses = [
            CiStatus::Pending,
            CiStatus::Running,
            CiStatus::Success,
            CiStatus::Failed,
            CiStatus::Cancelled,
        ];
        for s in &ci_statuses {
            let j = serde_json::to_string(s).expect("serialize ci status");
            let back: CiStatus = serde_json::from_str(&j).expect("deserialize ci status");
            assert_eq!(*s, back);
        }
    }
}
