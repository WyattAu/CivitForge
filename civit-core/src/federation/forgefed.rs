#![forbid(unsafe_code)]

use crate::error::{CoreError, Result};
use crate::federation::activitypub::{Activity, ActivityObject, ActivityType};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ForgeFedActivity {
    CreateRepository {
        actor: String,
        repo: FederatedRepo,
    },
    ForkRepository {
        actor: String,
        source: FederatedRepo,
        target: FederatedRepo,
    },
    StarRepository {
        actor: String,
        repo: FederatedRepo,
    },
    FollowUser {
        actor: String,
        target: String,
    },
    CreateIssue {
        actor: String,
        repo: FederatedRepo,
        issue: FederatedIssue,
    },
    CreatePullRequest {
        actor: String,
        repo: FederatedRepo,
        pr: FederatedPR,
    },
    Comment {
        actor: String,
        repo: FederatedRepo,
        target_type: String,
        target_id: String,
        body: String,
    },
    Like {
        actor: String,
        target_type: String,
        target_id: String,
    },
    Accept {
        actor: String,
        target: String,
    },
    Reject {
        actor: String,
        target: String,
    },
    Undo {
        actor: String,
        target_type: String,
        target_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FederatedRepo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner: String,
    pub visibility: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FederatedIssue {
    pub id: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FederatedPR {
    pub id: String,
    pub title: String,
    pub body: String,
    pub source_branch: String,
    pub target_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessingOutcome {
    Accepted {
        activity_id: String,
        message: String,
    },
    Rejected {
        activity_id: String,
        reason: String,
    },
    Duplicate {
        activity_id: String,
    },
}

pub struct IdempotencyTracker {
    processed_ids: DashMap<String, DateTime<Utc>>,
}

impl Default for IdempotencyTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl IdempotencyTracker {
    pub fn new() -> Self {
        Self {
            processed_ids: DashMap::new(),
        }
    }

    pub fn is_duplicate(&self, id: &str) -> bool {
        self.processed_ids.contains_key(id)
    }

    pub fn mark_processed(&self, id: &str) {
        self.processed_ids.insert(id.to_string(), Utc::now());
    }

    pub fn cleanup_older_than(&self, cutoff: DateTime<Utc>) {
        self.processed_ids.retain(|_, ts| *ts > cutoff);
    }

    pub fn len(&self) -> usize {
        self.processed_ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.processed_ids.is_empty()
    }
}

pub struct ForgeFedProcessor {
    pub instance_domain: String,
    pub instance_id: String,
    idempotency: IdempotencyTracker,
}

impl ForgeFedProcessor {
    pub fn new(instance_domain: String, instance_id: String) -> Self {
        Self {
            instance_domain,
            instance_id,
            idempotency: IdempotencyTracker::new(),
        }
    }

    fn activity_id(activity: &ForgeFedActivity) -> String {
        let payload = serde_json::to_string(activity).unwrap_or_default();
        let hash = Sha256::digest(payload.as_bytes());
        format!("forgefed-{}", hex::encode(hash))
    }

    pub fn process_incoming(&self, activity: ForgeFedActivity) -> Result<ProcessingOutcome> {
        let id = Self::activity_id(&activity);

        if self.idempotency.is_duplicate(&id) {
            return Ok(ProcessingOutcome::Duplicate { activity_id: id });
        }

        let outcome = match &activity {
            ForgeFedActivity::CreateRepository { actor, repo } => {
                if actor.is_empty() || repo.id.is_empty() {
                    ProcessingOutcome::Rejected {
                        activity_id: id.clone(),
                        reason: "missing required fields".into(),
                    }
                } else {
                    ProcessingOutcome::Accepted {
                        activity_id: id.clone(),
                        message: format!("repository {} created by {}", repo.name, actor),
                    }
                }
            }
            ForgeFedActivity::ForkRepository {
                actor,
                source,
                target,
            } => {
                if actor.is_empty() || source.id.is_empty() || target.id.is_empty() {
                    ProcessingOutcome::Rejected {
                        activity_id: id.clone(),
                        reason: "missing required fields for fork".into(),
                    }
                } else {
                    ProcessingOutcome::Accepted {
                        activity_id: id.clone(),
                        message: format!("forked {} -> {} by {}", source.name, target.name, actor),
                    }
                }
            }
            ForgeFedActivity::StarRepository { actor, repo } => {
                if actor.is_empty() || repo.id.is_empty() {
                    ProcessingOutcome::Rejected {
                        activity_id: id.clone(),
                        reason: "missing required fields for star".into(),
                    }
                } else {
                    ProcessingOutcome::Accepted {
                        activity_id: id.clone(),
                        message: format!("{} starred {}", actor, repo.name),
                    }
                }
            }
            ForgeFedActivity::FollowUser { actor, target } => {
                if actor.is_empty() || target.is_empty() {
                    ProcessingOutcome::Rejected {
                        activity_id: id.clone(),
                        reason: "missing required fields for follow".into(),
                    }
                } else {
                    ProcessingOutcome::Accepted {
                        activity_id: id.clone(),
                        message: format!("{actor} wants to follow {target}"),
                    }
                }
            }
            ForgeFedActivity::CreateIssue { actor, repo, issue } => {
                if actor.is_empty() || repo.id.is_empty() || issue.id.is_empty() {
                    ProcessingOutcome::Rejected {
                        activity_id: id.clone(),
                        reason: "missing required fields for issue".into(),
                    }
                } else {
                    ProcessingOutcome::Accepted {
                        activity_id: id.clone(),
                        message: format!(
                            "issue '{}' created in {} by {}",
                            issue.title, repo.name, actor
                        ),
                    }
                }
            }
            ForgeFedActivity::CreatePullRequest { actor, repo, pr } => {
                if actor.is_empty() || repo.id.is_empty() || pr.id.is_empty() {
                    ProcessingOutcome::Rejected {
                        activity_id: id.clone(),
                        reason: "missing required fields for PR".into(),
                    }
                } else {
                    ProcessingOutcome::Accepted {
                        activity_id: id.clone(),
                        message: format!("PR '{}' created in {} by {}", pr.title, repo.name, actor),
                    }
                }
            }
            ForgeFedActivity::Comment {
                actor,
                repo,
                target_type,
                target_id,
                body,
            } => {
                if actor.is_empty() || body.is_empty() {
                    ProcessingOutcome::Rejected {
                        activity_id: id.clone(),
                        reason: "missing required fields for comment".into(),
                    }
                } else {
                    ProcessingOutcome::Accepted {
                        activity_id: id.clone(),
                        message: format!(
                            "comment on {} {} in {} by {}",
                            target_type, target_id, repo.name, actor
                        ),
                    }
                }
            }
            ForgeFedActivity::Like {
                actor,
                target_type,
                target_id,
            } => {
                if actor.is_empty() || target_id.is_empty() {
                    ProcessingOutcome::Rejected {
                        activity_id: id.clone(),
                        reason: "missing required fields for like".into(),
                    }
                } else {
                    ProcessingOutcome::Accepted {
                        activity_id: id.clone(),
                        message: format!("{actor} liked {target_type} {target_id}"),
                    }
                }
            }
            ForgeFedActivity::Accept { actor, target } => {
                if actor.is_empty() || target.is_empty() {
                    ProcessingOutcome::Rejected {
                        activity_id: id.clone(),
                        reason: "missing required fields for accept".into(),
                    }
                } else {
                    ProcessingOutcome::Accepted {
                        activity_id: id.clone(),
                        message: format!("{actor} accepted {target}"),
                    }
                }
            }
            ForgeFedActivity::Reject { actor, target } => {
                if actor.is_empty() || target.is_empty() {
                    ProcessingOutcome::Rejected {
                        activity_id: id.clone(),
                        reason: "missing required fields for reject".into(),
                    }
                } else {
                    ProcessingOutcome::Accepted {
                        activity_id: id.clone(),
                        message: format!("{actor} rejected {target}"),
                    }
                }
            }
            ForgeFedActivity::Undo {
                actor,
                target_type,
                target_id,
            } => {
                if actor.is_empty() || target_id.is_empty() {
                    ProcessingOutcome::Rejected {
                        activity_id: id.clone(),
                        reason: "missing required fields for undo".into(),
                    }
                } else {
                    ProcessingOutcome::Accepted {
                        activity_id: id.clone(),
                        message: format!("{actor} undid {target_type} {target_id}"),
                    }
                }
            }
        };

        self.idempotency.mark_processed(&id);

        info!(
            activity_id = %id,
            outcome = ?outcome,
            "processed ForgeFed activity"
        );

        Ok(outcome)
    }

    pub fn build_outbox_activity(&self, activity: ForgeFedActivity) -> Activity {
        let id = Self::activity_id(&activity);
        let (actor, object, activity_type) = match &activity {
            ForgeFedActivity::CreateRepository { actor, repo } => (
                actor.clone(),
                ActivityObject::Repository {
                    id: repo.id.clone(),
                    name: repo.name.clone(),
                    attributed_to: actor.clone(),
                },
                ActivityType::Create,
            ),
            ForgeFedActivity::ForkRepository { actor, target, .. } => (
                actor.clone(),
                ActivityObject::Repository {
                    id: target.id.clone(),
                    name: target.name.clone(),
                    attributed_to: actor.clone(),
                },
                ActivityType::Create,
            ),
            ForgeFedActivity::StarRepository { actor, repo } => (
                actor.clone(),
                ActivityObject::Repository {
                    id: repo.id.clone(),
                    name: repo.name.clone(),
                    attributed_to: actor.clone(),
                },
                ActivityType::Like,
            ),
            ForgeFedActivity::FollowUser { actor, .. } => (
                actor.clone(),
                ActivityObject::Unknown(serde_json::json!({ "type": "Follow" })),
                ActivityType::Follow,
            ),
            ForgeFedActivity::CreateIssue { actor, issue, .. } => (
                actor.clone(),
                ActivityObject::Issue {
                    id: issue.id.clone(),
                    name: issue.title.clone(),
                    attributed_to: actor.clone(),
                },
                ActivityType::Create,
            ),
            ForgeFedActivity::CreatePullRequest { actor, pr, .. } => (
                actor.clone(),
                ActivityObject::PullRequest {
                    id: pr.id.clone(),
                    name: pr.title.clone(),
                    attributed_to: actor.clone(),
                },
                ActivityType::Create,
            ),
            ForgeFedActivity::Comment {
                actor, repo, body, ..
            } => (
                actor.clone(),
                ActivityObject::Note {
                    id: repo.id.clone(),
                    content: body.clone(),
                    attributed_to: actor.clone(),
                },
                ActivityType::Create,
            ),
            ForgeFedActivity::Like {
                actor, target_id, ..
            } => (
                actor.clone(),
                ActivityObject::Unknown(serde_json::json!({
                    "type": "Like",
                    "id": target_id,
                })),
                ActivityType::Like,
            ),
            ForgeFedActivity::Accept { actor, target } => (
                actor.clone(),
                ActivityObject::Unknown(serde_json::json!({
                    "type": "Accept",
                    "id": target,
                })),
                ActivityType::Accept,
            ),
            ForgeFedActivity::Reject { actor, target } => (
                actor.clone(),
                ActivityObject::Unknown(serde_json::json!({
                    "type": "Reject",
                    "id": target,
                })),
                ActivityType::Reject,
            ),
            ForgeFedActivity::Undo {
                actor, target_id, ..
            } => (
                actor.clone(),
                ActivityObject::Unknown(serde_json::json!({
                    "type": "Undo",
                    "id": target_id,
                })),
                ActivityType::Undo,
            ),
        };

        Activity {
            r#type: activity_type,
            id,
            actor,
            object,
            target: None,
            published: Utc::now().to_rfc3339(),
            to: vec!["https://www.w3.org/ns/activitystreams#Public".into()],
            cc: vec![],
        }
    }

    /// Verify signature using SHA-256 hash comparison (insecure, legacy).
    ///
    /// WARNING: This is NOT cryptographic signature verification. It compares a
    /// hash of payload+key_id against the signature. Use `verify_signature_ed25519`
    /// for real cryptographic verification.
    #[deprecated(note = "Use verify_signature_ed25519 for real crypto verification")]
    pub fn verify_signature(payload: &str, signature: &str, key_id: &str) -> Result<bool> {
        if payload.is_empty() || signature.is_empty() || key_id.is_empty() {
            return Err(CoreError::Federation(
                "payload, signature, and key_id must all be non-empty".into(),
            ));
        }

        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        hasher.update(key_id.as_bytes());
        let expected = hex::encode(hasher.finalize());

        Ok(signature == expected)
    }

    /// Verify a ForgeFed activity signature using Ed25519 cryptographic verification.
    ///
    /// `public_key_bytes` must be the raw 32-byte Ed25519 public key of the signing actor.
    /// `signature` must be the base64-encoded 64-byte Ed25519 signature.
    pub fn verify_signature_ed25519(
        payload: &str,
        signature: &str,
        key_id: &str,
        public_key_bytes: &[u8],
    ) -> Result<bool> {
        if payload.is_empty() || signature.is_empty() || key_id.is_empty() {
            return Err(CoreError::Federation(
                "payload, signature, and key_id must all be non-empty".into(),
            ));
        }

        let mut message = Vec::new();
        message.extend_from_slice(payload.as_bytes());
        message.extend_from_slice(key_id.as_bytes());

        let signature_bytes =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, signature)
                .map_err(|_| CoreError::Federation("invalid base64 signature".into()))?;

        let public_key =
            ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, public_key_bytes);

        match public_key.verify(&message, &signature_bytes) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_processor() -> ForgeFedProcessor {
        ForgeFedProcessor::new("forge.example.com".into(), "inst-1".into())
    }

    fn make_repo() -> FederatedRepo {
        FederatedRepo {
            id: "repo-1".into(),
            name: "test-repo".into(),
            description: "a test repo".into(),
            owner: "alice".into(),
            visibility: "public".into(),
        }
    }

    #[test]
    fn test_process_create_repository() {
        let proc = make_processor();
        let activity = ForgeFedActivity::CreateRepository {
            actor: "https://other.forge/users/alice".into(),
            repo: make_repo(),
        };
        let outcome = proc.process_incoming(activity).unwrap();
        match outcome {
            ProcessingOutcome::Accepted { ref message, .. } => {
                assert!(message.contains("repository test-repo created"));
            }
            _ => panic!("expected Accepted"),
        }
    }

    #[test]
    fn test_process_create_repository_missing_actor() {
        let proc = make_processor();
        let activity = ForgeFedActivity::CreateRepository {
            actor: "".into(),
            repo: make_repo(),
        };
        let outcome = proc.process_incoming(activity).unwrap();
        match outcome {
            ProcessingOutcome::Rejected { ref reason, .. } => {
                assert!(reason.contains("missing"));
            }
            _ => panic!("expected Rejected"),
        }
    }

    #[test]
    fn test_process_fork_repository() {
        let proc = make_processor();
        let source = make_repo();
        let target = FederatedRepo {
            id: "repo-2".into(),
            name: "test-repo-fork".into(),
            description: "a fork".into(),
            owner: "bob".into(),
            visibility: "public".into(),
        };
        let activity = ForgeFedActivity::ForkRepository {
            actor: "https://other.forge/users/bob".into(),
            source,
            target,
        };
        let outcome = proc.process_incoming(activity).unwrap();
        match outcome {
            ProcessingOutcome::Accepted { ref message, .. } => {
                assert!(message.contains("forked"));
            }
            _ => panic!("expected Accepted"),
        }
    }

    #[test]
    fn test_process_star_repository() {
        let proc = make_processor();
        let activity = ForgeFedActivity::StarRepository {
            actor: "https://other.forge/users/carol".into(),
            repo: make_repo(),
        };
        let outcome = proc.process_incoming(activity).unwrap();
        match outcome {
            ProcessingOutcome::Accepted { ref message, .. } => {
                assert!(message.contains("starred"));
            }
            _ => panic!("expected Accepted"),
        }
    }

    #[test]
    fn test_process_follow_user() {
        let proc = make_processor();
        let activity = ForgeFedActivity::FollowUser {
            actor: "https://other.forge/users/alice".into(),
            target: "https://forge.example.com/users/bob".into(),
        };
        let outcome = proc.process_incoming(activity).unwrap();
        match outcome {
            ProcessingOutcome::Accepted { ref message, .. } => {
                assert!(message.contains("follow"));
            }
            _ => panic!("expected Accepted"),
        }
    }

    #[test]
    fn test_process_create_issue() {
        let proc = make_processor();
        let activity = ForgeFedActivity::CreateIssue {
            actor: "https://other.forge/users/alice".into(),
            repo: make_repo(),
            issue: FederatedIssue {
                id: "issue-1".into(),
                title: "Bug report".into(),
                body: "Something is broken".into(),
            },
        };
        let outcome = proc.process_incoming(activity).unwrap();
        match outcome {
            ProcessingOutcome::Accepted { ref message, .. } => {
                assert!(message.contains("issue 'Bug report' created"));
            }
            _ => panic!("expected Accepted"),
        }
    }

    #[test]
    fn test_process_create_pr() {
        let proc = make_processor();
        let activity = ForgeFedActivity::CreatePullRequest {
            actor: "https://other.forge/users/alice".into(),
            repo: make_repo(),
            pr: FederatedPR {
                id: "pr-1".into(),
                title: "Fix bug".into(),
                body: "Fixes issue-1".into(),
                source_branch: "fix-branch".into(),
                target_branch: "main".into(),
            },
        };
        let outcome = proc.process_incoming(activity).unwrap();
        match outcome {
            ProcessingOutcome::Accepted { ref message, .. } => {
                assert!(message.contains("PR 'Fix bug' created"));
            }
            _ => panic!("expected Accepted"),
        }
    }

    #[test]
    fn test_process_comment() {
        let proc = make_processor();
        let activity = ForgeFedActivity::Comment {
            actor: "https://other.forge/users/alice".into(),
            repo: make_repo(),
            target_type: "Issue".into(),
            target_id: "issue-1".into(),
            body: "Looks good to me".into(),
        };
        let outcome = proc.process_incoming(activity).unwrap();
        match outcome {
            ProcessingOutcome::Accepted { ref message, .. } => {
                assert!(message.contains("comment on"));
            }
            _ => panic!("expected Accepted"),
        }
    }

    #[test]
    fn test_process_like() {
        let proc = make_processor();
        let activity = ForgeFedActivity::Like {
            actor: "https://other.forge/users/alice".into(),
            target_type: "Comment".into(),
            target_id: "comment-1".into(),
        };
        let outcome = proc.process_incoming(activity).unwrap();
        match outcome {
            ProcessingOutcome::Accepted { ref message, .. } => {
                assert!(message.contains("liked"));
            }
            _ => panic!("expected Accepted"),
        }
    }

    #[test]
    fn test_process_accept() {
        let proc = make_processor();
        let activity = ForgeFedActivity::Accept {
            actor: "https://forge.example.com/users/bob".into(),
            target: "https://other.forge/users/alice".into(),
        };
        let outcome = proc.process_incoming(activity).unwrap();
        match outcome {
            ProcessingOutcome::Accepted { ref message, .. } => {
                assert!(message.contains("accepted"));
            }
            _ => panic!("expected Accepted"),
        }
    }

    #[test]
    fn test_process_reject() {
        let proc = make_processor();
        let activity = ForgeFedActivity::Reject {
            actor: "https://forge.example.com/users/bob".into(),
            target: "https://other.forge/users/alice".into(),
        };
        let outcome = proc.process_incoming(activity).unwrap();
        match outcome {
            ProcessingOutcome::Accepted { ref message, .. } => {
                assert!(message.contains("rejected"));
            }
            _ => panic!("expected Accepted"),
        }
    }

    #[test]
    fn test_process_undo() {
        let proc = make_processor();
        let activity = ForgeFedActivity::Undo {
            actor: "https://other.forge/users/alice".into(),
            target_type: "Like".into(),
            target_id: "comment-1".into(),
        };
        let outcome = proc.process_incoming(activity).unwrap();
        match outcome {
            ProcessingOutcome::Accepted { ref message, .. } => {
                assert!(message.contains("undid"));
            }
            _ => panic!("expected Accepted"),
        }
    }

    #[test]
    fn test_idempotency_duplicate() {
        let proc = make_processor();
        let activity = ForgeFedActivity::StarRepository {
            actor: "alice".into(),
            repo: make_repo(),
        };
        let _ = proc.process_incoming(activity.clone()).unwrap();
        let outcome = proc.process_incoming(activity).unwrap();
        match outcome {
            ProcessingOutcome::Duplicate { .. } => {}
            _ => panic!("expected Duplicate"),
        }
    }

    #[test]
    fn test_idempotency_tracker_cleanup() {
        let tracker = IdempotencyTracker::new();
        tracker.mark_processed("id-1");
        tracker.mark_processed("id-2");
        assert_eq!(tracker.len(), 2);
        let cutoff = Utc::now() + chrono::Duration::seconds(10);
        tracker.cleanup_older_than(cutoff);
        assert_eq!(tracker.len(), 0);
    }

    #[test]
    fn test_build_outbox_activity_create_repo() {
        let proc = make_processor();
        let activity = ForgeFedActivity::CreateRepository {
            actor: "https://forge.example.com/users/alice".into(),
            repo: make_repo(),
        };
        let outbox = proc.build_outbox_activity(activity);
        assert_eq!(outbox.r#type, ActivityType::Create);
        assert!(!outbox.id.is_empty());
    }

    #[test]
    fn test_build_outbox_activity_follow() {
        let proc = make_processor();
        let activity = ForgeFedActivity::FollowUser {
            actor: "https://forge.example.com/users/alice".into(),
            target: "https://other.forge/users/bob".into(),
        };
        let outbox = proc.build_outbox_activity(activity);
        assert_eq!(outbox.r#type, ActivityType::Follow);
    }

    #[test]
    fn test_verify_signature_valid() {
        let payload = "test-payload";
        let key_id = "key-123";
        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        hasher.update(key_id.as_bytes());
        let signature = hex::encode(hasher.finalize());
        assert!(ForgeFedProcessor::verify_signature(payload, &signature, key_id).unwrap());
    }

    #[test]
    fn test_verify_signature_invalid() {
        let payload = "test-payload";
        let signature = "invalid-signature";
        let key_id = "key-123";
        assert!(!ForgeFedProcessor::verify_signature(payload, signature, key_id).unwrap());
    }

    #[test]
    fn test_verify_signature_empty_inputs() {
        let result = ForgeFedProcessor::verify_signature("", "sig", "key");
        assert!(result.is_err());
    }
}
