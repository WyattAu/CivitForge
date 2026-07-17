#![forbid(unsafe_code)]

use crate::error::{FedError, Result};
use crate::activitypub::{Activity, ActivityObject, ActivityType};
use crate::webfinger::WebFingerResponse;
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
    ReviewPullRequest {
        actor: String,
        repo: FederatedRepo,
        review: FederatedPRReview,
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
    pub state: IssueState,
    pub author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum IssueState {
    #[default]
    Open,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FederatedPR {
    pub id: String,
    pub title: String,
    pub body: String,
    pub source_branch: String,
    pub target_branch: String,
    pub state: PRState,
    pub author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum PRState {
    #[default]
    Open,
    Closed,
    Merged,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PRReviewState {
    Approved,
    ChangesRequested,
    Comment,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FederatedPRReview {
    pub id: String,
    pub pr_id: String,
    pub reviewer: String,
    pub state: PRReviewState,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FederatedStar {
    pub id: String,
    pub actor: String,
    pub repo_id: String,
    pub starred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FederatedFork {
    pub id: String,
    pub source_repo_id: String,
    pub target_repo_id: String,
    pub actor: String,
    pub forked_at: DateTime<Utc>,
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
                            "issue '{}' ({:?}) created in {} by {}",
                            issue.title, issue.state, repo.name, actor
                        ),
                    }
                }
            }
            ForgeFedActivity::ReviewPullRequest {
                actor,
                repo,
                review,
            } => {
                if actor.is_empty()
                    || repo.id.is_empty()
                    || review.id.is_empty()
                    || review.reviewer.is_empty()
                {
                    ProcessingOutcome::Rejected {
                        activity_id: id.clone(),
                        reason: "missing required fields for PR review".into(),
                    }
                } else {
                    ProcessingOutcome::Accepted {
                        activity_id: id.clone(),
                        message: format!(
                            "PR review {:?} on PR {} in {} by {}",
                            review.state, review.pr_id, repo.name, actor
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
            ForgeFedActivity::ReviewPullRequest { actor, review, .. } => (
                actor.clone(),
                ActivityObject::Unknown(serde_json::json!({
                    "type": "Review",
                    "id": review.id,
                    "pr": review.pr_id,
                    "state": format!("{:?}", review.state),
                })),
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

    #[deprecated(note = "Use verify_signature_ed25519 for real crypto verification")]
    pub fn verify_signature(payload: &str, signature: &str, key_id: &str) -> Result<bool> {
        if payload.is_empty() || signature.is_empty() || key_id.is_empty() {
            return Err(FedError(
                "payload, signature, and key_id must all be non-empty".into(),
            ));
        }

        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        hasher.update(key_id.as_bytes());
        let expected = hex::encode(hasher.finalize());

        Ok(signature == expected)
    }

    pub fn verify_signature_ed25519(
        payload: &str,
        signature: &str,
        key_id: &str,
        public_key_bytes: &[u8],
    ) -> Result<bool> {
        if payload.is_empty() || signature.is_empty() || key_id.is_empty() {
            return Err(FedError(
                "payload, signature, and key_id must all be non-empty".into(),
            ));
        }

        let mut message = Vec::new();
        message.extend_from_slice(payload.as_bytes());
        message.extend_from_slice(key_id.as_bytes());

        let signature_bytes =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, signature)
                .map_err(|_| FedError("invalid base64 signature".into()))?;

        let public_key =
            ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, public_key_bytes);

        match public_key.verify(&message, &signature_bytes) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

pub struct CrossInstanceIdentityResolver {
    cache: DashMap<String, WebFingerResponse>,
    instance_domain: String,
}

impl CrossInstanceIdentityResolver {
    pub fn new(instance_domain: String) -> Self {
        Self {
            cache: DashMap::new(),
            instance_domain,
        }
    }

    pub fn resolve_local(&self, username: &str) -> String {
        format!("https://{}/users/{}", self.instance_domain, username)
    }

    pub fn cache_identity(&self, acct: &str, response: WebFingerResponse) {
        self.cache.insert(acct.to_string(), response);
    }

    pub fn get_cached(&self, acct: &str) -> Option<WebFingerResponse> {
        self.cache.get(acct).map(|r| r.clone())
    }

    pub fn remove_cached(&self, acct: &str) -> bool {
        self.cache.remove(acct).is_some()
    }

    pub fn cached_count(&self) -> usize {
        self.cache.len()
    }

    pub fn federation_uri(&self, remote_domain: &str, username: &str) -> String {
        format!("acct:{username}@{remote_domain}")
    }

    pub fn is_local(&self, actor_uri: &str) -> bool {
        actor_uri.contains(&self.instance_domain)
    }
}

#[cfg(test)]
#[allow(deprecated)]
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
                state: IssueState::Open,
                author: "alice".into(),
            },
        };
        let outcome = proc.process_incoming(activity).unwrap();
        match outcome {
            ProcessingOutcome::Accepted { ref message, .. } => {
                assert!(message.contains("issue 'Bug report'"));
                assert!(message.contains("Open"));
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
                state: PRState::Open,
                author: "alice".into(),
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

    #[test]
    fn test_process_review_pr_accepted() {
        let proc = make_processor();
        let activity = ForgeFedActivity::ReviewPullRequest {
            actor: "https://other.forge/users/alice".into(),
            repo: make_repo(),
            review: FederatedPRReview {
                id: "rev-1".into(),
                pr_id: "pr-1".into(),
                reviewer: "alice".into(),
                state: PRReviewState::Approved,
                body: "LGTM".into(),
            },
        };
        let outcome = proc.process_incoming(activity).unwrap();
        match outcome {
            ProcessingOutcome::Accepted { ref message, .. } => {
                assert!(message.contains("Approved"));
                assert!(message.contains("pr-1"));
            }
            _ => panic!("expected Accepted"),
        }
    }

    #[test]
    fn test_process_review_pr_missing_reviewer() {
        let proc = make_processor();
        let activity = ForgeFedActivity::ReviewPullRequest {
            actor: "alice".into(),
            repo: make_repo(),
            review: FederatedPRReview {
                id: "rev-1".into(),
                pr_id: "pr-1".into(),
                reviewer: "".into(),
                state: PRReviewState::Approved,
                body: "LGTM".into(),
            },
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
    fn test_issue_state_default() {
        assert_eq!(IssueState::default(), IssueState::Open);
    }

    #[test]
    fn test_pr_state_default() {
        assert_eq!(PRState::default(), PRState::Open);
    }

    #[test]
    fn test_federated_star_construction() {
        let star = FederatedStar {
            id: "star-1".into(),
            actor: "alice".into(),
            repo_id: "repo-1".into(),
            starred_at: Utc::now(),
        };
        assert_eq!(star.actor, "alice");
        assert_eq!(star.repo_id, "repo-1");
    }

    #[test]
    fn test_federated_fork_construction() {
        let fork = FederatedFork {
            id: "fork-1".into(),
            source_repo_id: "repo-1".into(),
            target_repo_id: "repo-2".into(),
            actor: "bob".into(),
            forked_at: Utc::now(),
        };
        assert_eq!(fork.source_repo_id, "repo-1");
        assert_eq!(fork.target_repo_id, "repo-2");
    }

    #[test]
    fn test_cross_instance_identity_resolver_local() {
        let resolver = CrossInstanceIdentityResolver::new("forge.example.com".into());
        assert_eq!(
            resolver.resolve_local("alice"),
            "https://forge.example.com/users/alice"
        );
        assert!(resolver.is_local("https://forge.example.com/users/alice"));
        assert!(!resolver.is_local("https://other.forge/users/bob"));
    }

    #[test]
    fn test_cross_instance_identity_resolver_cache() {
        let resolver = CrossInstanceIdentityResolver::new("forge.example.com".into());
        assert_eq!(resolver.cached_count(), 0);
        let wf = WebFingerResponse {
            subject: "acct:bob@other.forge".into(),
            aliases: vec!["https://other.forge/users/bob".into()],
            links: vec![],
        };
        resolver.cache_identity("acct:bob@other.forge", wf);
        assert_eq!(resolver.cached_count(), 1);
        assert!(resolver.get_cached("acct:bob@other.forge").is_some());
        assert!(resolver.remove_cached("acct:bob@other.forge"));
        assert_eq!(resolver.cached_count(), 0);
    }

    #[test]
    fn test_cross_instance_identity_resolver_federation_uri() {
        let resolver = CrossInstanceIdentityResolver::new("forge.example.com".into());
        let uri = resolver.federation_uri("other.forge", "bob");
        assert_eq!(uri, "acct:bob@other.forge");
    }

    #[test]
    fn test_review_state_variants() {
        assert_eq!(format!("{:?}", PRReviewState::Approved), "Approved");
        assert_eq!(
            format!("{:?}", PRReviewState::ChangesRequested),
            "ChangesRequested"
        );
        assert_eq!(format!("{:?}", PRReviewState::Comment), "Comment");
        assert_eq!(format!("{:?}", PRReviewState::Pending), "Pending");
    }

    #[test]
    fn test_pr_state_variants() {
        assert_eq!(format!("{:?}", PRState::Open), "Open");
        assert_eq!(format!("{:?}", PRState::Closed), "Closed");
        assert_eq!(format!("{:?}", PRState::Merged), "Merged");
    }

    #[test]
    fn test_build_outbox_activity_review_pr() {
        let proc = make_processor();
        let activity = ForgeFedActivity::ReviewPullRequest {
            actor: "alice".into(),
            repo: make_repo(),
            review: FederatedPRReview {
                id: "rev-1".into(),
                pr_id: "pr-1".into(),
                reviewer: "alice".into(),
                state: PRReviewState::ChangesRequested,
                body: "needs work".into(),
            },
        };
        let outbox = proc.build_outbox_activity(activity);
        assert!(!outbox.id.is_empty());
    }

    #[test]
    fn test_federated_issue_serialization() {
        let issue = FederatedIssue {
            id: "i1".into(),
            title: "Bug".into(),
            body: "fix me".into(),
            state: IssueState::Closed,
            author: "alice".into(),
        };
        let json = serde_json::to_string(&issue).unwrap();
        let deser: FederatedIssue = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.state, IssueState::Closed);
    }

    #[test]
    fn test_federated_pr_serialization() {
        let pr = FederatedPR {
            id: "p1".into(),
            title: "Fix".into(),
            body: "fix".into(),
            source_branch: "f".into(),
            target_branch: "main".into(),
            state: PRState::Merged,
            author: "bob".into(),
        };
        let json = serde_json::to_string(&pr).unwrap();
        let deser: FederatedPR = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.state, PRState::Merged);
    }
}
