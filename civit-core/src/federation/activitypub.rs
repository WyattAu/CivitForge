#![forbid(unsafe_code)]

use crate::error::{CoreError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    pub id: String,
    pub r#type: ActorType,
    pub preferred_username: String,
    pub inbox: String,
    pub outbox: String,
    pub public_key: PublicKey,
    pub endpoints: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ActorType {
    Person,
    Organization,
    Application,
    Service,
    Group,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKey {
    pub id: String,
    pub owner: String,
    pub public_key_pem: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub r#type: ActivityType,
    pub id: String,
    pub actor: String,
    pub object: ActivityObject,
    pub target: Option<String>,
    pub published: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityObject {
    Repository {
        id: String,
        name: String,
        attributed_to: String,
    },
    Commit {
        id: String,
        message: String,
        attributed_to: String,
    },
    Issue {
        id: String,
        name: String,
        attributed_to: String,
    },
    PullRequest {
        id: String,
        name: String,
        attributed_to: String,
    },
    Note {
        id: String,
        content: String,
        attributed_to: String,
    },
    Unknown(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum ActivityType {
    Create,
    Update,
    Delete,
    Follow,
    Undo,
    Accept,
    Reject,
    Add,
    Like,
    Announce,
}

#[derive(Debug, Clone)]
pub struct InboxHandler {
    #[allow(dead_code)]
    instance_id: String,
    instance_domain: String,
}

impl InboxHandler {
    pub fn new(instance_id: String, instance_domain: String) -> Self {
        Self {
            instance_id,
            instance_domain,
        }
    }

    pub fn validate_actor(&self, actor: &Actor) -> Result<()> {
        if actor.id.is_empty() {
            return Err(CoreError::Federation("actor id required".into()));
        }
        if actor.inbox.is_empty() {
            return Err(CoreError::Federation("actor inbox required".into()));
        }
        if actor.preferred_username.is_empty() {
            return Err(CoreError::Federation("actor username required".into()));
        }
        Ok(())
    }

    pub fn validate_activity(&self, activity: &Activity) -> Result<()> {
        if activity.actor.is_empty() {
            return Err(CoreError::Federation("activity actor required".into()));
        }
        if activity.id.is_empty() {
            return Err(CoreError::Federation("activity id required".into()));
        }
        if activity.to.is_empty() && activity.cc.is_empty() {
            return Err(CoreError::Federation("activity needs recipient".into()));
        }
        Ok(())
    }

    pub fn process_incoming(&self, activity: Activity) -> Result<ProcessingResult> {
        self.validate_activity(&activity)?;

        let result = match activity.r#type {
            ActivityType::Create => ProcessingResult::Accepted {
                message: "resource created".into(),
                id: activity.id.clone(),
            },
            ActivityType::Update => ProcessingResult::Accepted {
                message: "resource updated".into(),
                id: activity.id.clone(),
            },
            ActivityType::Delete => ProcessingResult::Accepted {
                message: "resource deleted".into(),
                id: activity.id.clone(),
            },
            ActivityType::Follow => ProcessingResult::Pending {
                message: "follow request pending".into(),
                id: activity.id.clone(),
            },
            ActivityType::Accept => ProcessingResult::Accepted {
                message: "follow accepted".into(),
                id: activity.id.clone(),
            },
            ActivityType::Reject => ProcessingResult::Rejected {
                message: "activity rejected".into(),
                id: activity.id.clone(),
            },
            _ => ProcessingResult::Accepted {
                message: "activity processed".into(),
                id: activity.id.clone(),
            },
        };

        info!(
            activity_type = ?activity.r#type,
            actor = %activity.actor,
            result = ?result,
            "processed incoming activity"
        );
        Ok(result)
    }

    pub fn build_webfinger_response(&self, username: &str) -> serde_json::Value {
        serde_json::json!({
            "subject": format!("acct:{}@{}", username, self.instance_domain),
            "aliases": [
                format!("https://{}/users/{}", self.instance_domain, username),
            ],
            "links": [
                {
                    "rel": "self",
                    "type": "application/activity+json",
                    "href": format!("https://{}/users/{}", self.instance_domain, username),
                }
            ],
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingResult {
    Accepted { message: String, id: String },
    Pending { message: String, id: String },
    Rejected { message: String, id: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_handler() -> InboxHandler {
        InboxHandler::new("inst-1".into(), "forge.example.com".into())
    }

    fn make_valid_activity() -> Activity {
        Activity {
            r#type: ActivityType::Create,
            id: "act-1".into(),
            actor: "https://other.forge/users/alice".into(),
            object: ActivityObject::Repository {
                id: "repo-1".into(),
                name: "test-repo".into(),
                attributed_to: "https://other.forge/users/alice".into(),
            },
            target: None,
            published: "2025-01-01T00:00:00Z".into(),
            to: vec!["https://www.w3.org/ns/activitystreams#Public".into()],
            cc: vec![],
        }
    }

    #[test]
    fn test_validate_actor_valid() {
        let handler = make_handler();
        let actor = Actor {
            id: "https://forge.example.com/users/alice".into(),
            r#type: ActorType::Person,
            preferred_username: "alice".into(),
            inbox: "https://forge.example.com/users/alice/inbox".into(),
            outbox: "https://forge.example.com/users/alice/outbox".into(),
            public_key: PublicKey {
                id: "key-1".into(),
                owner: "https://forge.example.com/users/alice".into(),
                public_key_pem: "-----BEGIN PUBLIC KEY-----...".into(),
            },
            endpoints: HashMap::new(),
        };
        assert!(handler.validate_actor(&actor).is_ok());
    }

    #[test]
    fn test_validate_actor_missing_fields() {
        let handler = make_handler();
        let actor = Actor {
            id: "".into(),
            r#type: ActorType::Person,
            preferred_username: "alice".into(),
            inbox: "https://forge.example.com/users/alice/inbox".into(),
            outbox: "https://forge.example.com/users/alice/outbox".into(),
            public_key: PublicKey {
                id: "key-1".into(),
                owner: "https://forge.example.com/users/alice".into(),
                public_key_pem: "-----BEGIN PUBLIC KEY-----...".into(),
            },
            endpoints: HashMap::new(),
        };
        assert!(handler.validate_actor(&actor).is_err());
    }

    #[test]
    fn test_process_create_activity() {
        let handler = make_handler();
        let activity = make_valid_activity();
        let result = handler.process_incoming(activity).unwrap();
        match result {
            ProcessingResult::Accepted { ref message, .. } => {
                assert_eq!(message, "resource created")
            }
            _ => panic!("expected Accepted"),
        }
    }

    #[test]
    fn test_process_follow_pending() {
        let handler = make_handler();
        let mut activity = make_valid_activity();
        activity.r#type = ActivityType::Follow;
        let result = handler.process_incoming(activity).unwrap();
        match result {
            ProcessingResult::Pending { ref message, .. } => {
                assert_eq!(message, "follow request pending")
            }
            _ => panic!("expected Pending"),
        }
    }

    #[test]
    fn test_validate_activity_no_recipient_fails() {
        let handler = make_handler();
        let mut activity = make_valid_activity();
        activity.to = vec![];
        activity.cc = vec![];
        assert!(handler.validate_activity(&activity).is_err());
    }

    #[test]
    fn test_webfinger_response() {
        let handler = make_handler();
        let response = handler.build_webfinger_response("alice");
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["subject"], "acct:alice@forge.example.com");
        assert!(!json["links"].as_array().unwrap().is_empty());
    }
}
