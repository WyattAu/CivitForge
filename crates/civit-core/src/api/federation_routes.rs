#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::error::CoreError;
use crate::federation::http_signatures::{HttpSignature, SignatureVerifier};
use crate::federation::{
    FederatedIssue, FederatedPR, FederatedPRReview, FederatedRepo, ForgeFedActivity, IssueState,
    PRReviewState, PRState,
};
use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

struct CachedKeyPair {
    public_key_pem: String,
    #[allow(dead_code)]
    private_key_bytes: Vec<u8>,
}

static FEDERATION_KEY: OnceLock<CachedKeyPair> = OnceLock::new();

fn get_federation_key() -> &'static CachedKeyPair {
    FEDERATION_KEY.get_or_init(|| {
        let (private_key, public_key) =
            crate::federation::http_signatures::generate_ed25519_keypair();
        let prefix: &[u8] = &[
            0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
        ];
        let mut der = Vec::with_capacity(prefix.len() + 32);
        der.extend_from_slice(prefix);
        der.extend_from_slice(&public_key);
        let b64 = BASE64.encode(&der);
        CachedKeyPair {
            public_key_pem: format!("-----BEGIN PUBLIC KEY-----\n{b64}\n-----END PUBLIC KEY-----"),
            private_key_bytes: private_key,
        }
    })
}

#[derive(Debug, Deserialize)]
pub struct WebFingerQuery {
    pub resource: String,
}

#[derive(Debug, Serialize)]
pub struct WebFingerResponse {
    pub subject: String,
    pub aliases: Vec<String>,
    pub links: Vec<WebFingerLink>,
}

#[derive(Debug, Serialize)]
pub struct WebFingerLink {
    pub rel: String,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub href: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ActorResponse {
    #[serde(rename = "@context")]
    pub context: String,
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub preferred_username: String,
    pub name: String,
    pub inbox: String,
    pub outbox: String,
    pub public_key: ActorPublicKey,
}

#[derive(Debug, Serialize)]
pub struct ActorPublicKey {
    pub id: String,
    pub owner: String,
    pub public_key_pem: String,
}

#[derive(Debug, Serialize)]
pub struct InboxResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct OutboxResponse {
    #[serde(rename = "@context")]
    pub context: String,
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub total_items: u64,
    pub ordered_items: Vec<serde_json::Value>,
}

pub fn federation_routes() -> Router<AppState> {
    Router::new()
        .route("/.well-known/webfinger", get(webfinger))
        .route("/api/v1/federation/actor", get(actor_endpoint))
        .route("/api/v1/federation/inbox", post(inbox))
        .route("/api/v1/federation/outbox", get(outbox))
}

pub async fn webfinger(
    State(state): State<AppState>,
    Query(_query): Query<WebFingerQuery>,
) -> impl IntoResponse {
    if !state.config.federation_enabled {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("federation disabled".into()).error_response()),
        )
            .into_response();
    }

    let domain = &state.config.federation_instance_domain;
    let instance_id = &state.config.federation_instance_id;

    let expected_subject = format!("acct:{instance_id}@{domain}");

    let resp = WebFingerResponse {
        subject: expected_subject.clone(),
        aliases: vec![format!("https://{domain}/api/v1/federation/actor")],
        links: vec![WebFingerLink {
            rel: "self".into(),
            type_: Some("application/activity+json".into()),
            href: Some(format!("https://{domain}/api/v1/federation/actor")),
        }],
    };

    (StatusCode::OK, Json(resp)).into_response()
}

pub async fn actor_endpoint(State(state): State<AppState>) -> impl IntoResponse {
    if !state.config.federation_enabled {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("federation disabled".into()).error_response()),
        )
            .into_response();
    }

    let domain = &state.config.federation_instance_domain;
    let instance_id = &state.config.federation_instance_id;
    let actor_url = format!("https://{domain}/api/v1/federation/actor");

    let resp = ActorResponse {
        context: "https://www.w3.org/ns/activitystreams".into(),
        id: actor_url.clone(),
        type_: "Application".into(),
        preferred_username: instance_id.clone(),
        name: format!("CivitForge - {instance_id}"),
        inbox: format!("https://{domain}/api/v1/federation/inbox"),
        outbox: format!("https://{domain}/api/v1/federation/outbox"),
        public_key: ActorPublicKey {
            id: format!("{actor_url}#main-key"),
            owner: actor_url,
            public_key_pem: get_federation_key().public_key_pem.clone(),
        },
    };

    (StatusCode::OK, Json(resp)).into_response()
}

fn parse_repo(obj: &serde_json::Value) -> Option<FederatedRepo> {
    Some(FederatedRepo {
        id: obj.get("id")?.as_str()?.to_string(),
        name: obj.get("name")?.as_str()?.to_string(),
        description: obj
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        owner: obj
            .get("attributedTo")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        visibility: obj
            .get("visibility")
            .and_then(|v| v.as_str())
            .unwrap_or("public")
            .to_string(),
    })
}

fn parse_incoming_activity(body: &serde_json::Value) -> crate::error::Result<ForgeFedActivity> {
    let activity_type = body
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CoreError::Federation("missing 'type' field".into()))?;
    let actor = body
        .get("actor")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let activity = match activity_type {
        "Create" => {
            let obj = body
                .get("object")
                .ok_or_else(|| CoreError::Federation("missing 'object' field".into()))?;
            let obj_type = obj
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            match obj_type {
                "Repository" => {
                    let repo = parse_repo(obj)
                        .ok_or_else(|| CoreError::Federation("invalid repository object".into()))?;
                    ForgeFedActivity::CreateRepository { actor, repo }
                }
                "Issue" => {
                    let issue = FederatedIssue {
                        id: obj
                            .get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        title: obj
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        body: obj
                            .get("content")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        state: match obj.get("state").and_then(|v| v.as_str()).unwrap_or("open") {
                            "closed" => IssueState::Closed,
                            _ => IssueState::Open,
                        },
                        author: obj
                            .get("attributedTo")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&actor)
                            .to_string(),
                    };
                    let repo_obj = body
                        .get("target")
                        .or_else(|| body.get("context"))
                        .and_then(|v| v.as_object())
                        .map(|o| serde_json::Value::Object(o.clone()));
                    let repo = match repo_obj {
                        Some(ref v) => parse_repo(v).ok_or_else(|| {
                            CoreError::Federation("invalid target repository".into())
                        })?,
                        None => FederatedRepo {
                            id: String::new(),
                            name: String::new(),
                            description: String::new(),
                            owner: String::new(),
                            visibility: String::new(),
                        },
                    };
                    ForgeFedActivity::CreateIssue { actor, repo, issue }
                }
                "PullRequest" => {
                    let pr = FederatedPR {
                        id: obj
                            .get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        title: obj
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        body: obj
                            .get("content")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        source_branch: obj
                            .get("sourceBranch")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        target_branch: obj
                            .get("targetBranch")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        state: match obj.get("state").and_then(|v| v.as_str()).unwrap_or("open") {
                            "closed" => PRState::Closed,
                            "merged" => PRState::Merged,
                            _ => PRState::Open,
                        },
                        author: obj
                            .get("attributedTo")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&actor)
                            .to_string(),
                    };
                    let repo_obj = body
                        .get("target")
                        .or_else(|| body.get("context"))
                        .and_then(|v| v.as_object())
                        .map(|o| serde_json::Value::Object(o.clone()));
                    let repo = match repo_obj {
                        Some(ref v) => parse_repo(v).ok_or_else(|| {
                            CoreError::Federation("invalid target repository".into())
                        })?,
                        None => FederatedRepo {
                            id: String::new(),
                            name: String::new(),
                            description: String::new(),
                            owner: String::new(),
                            visibility: String::new(),
                        },
                    };
                    ForgeFedActivity::CreatePullRequest { actor, repo, pr }
                }
                other => {
                    return Err(CoreError::Federation(format!(
                        "unknown Create object type: {other}"
                    )));
                }
            }
        }
        "Fork" => {
            let obj = body
                .get("object")
                .ok_or_else(|| CoreError::Federation("missing 'object' field".into()))?;
            let source = parse_repo(obj)
                .ok_or_else(|| CoreError::Federation("invalid source repository".into()))?;
            let target_obj = body
                .get("target")
                .ok_or_else(|| CoreError::Federation("missing 'target' field".into()))?;
            let target = parse_repo(target_obj)
                .ok_or_else(|| CoreError::Federation("invalid target repository".into()))?;
            ForgeFedActivity::ForkRepository {
                actor,
                source,
                target,
            }
        }
        "Like" => {
            let target_type = body
                .get("object")
                .and_then(|v| v.get("type"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            let target_id = body
                .get("object")
                .and_then(|v| v.get("id"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            ForgeFedActivity::Like {
                actor,
                target_type,
                target_id,
            }
        }
        "Follow" => {
            let target = body
                .get("object")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            ForgeFedActivity::FollowUser { actor, target }
        }
        "Accept" => {
            let target = body
                .get("object")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            ForgeFedActivity::Accept { actor, target }
        }
        "Reject" => {
            let target = body
                .get("object")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            ForgeFedActivity::Reject { actor, target }
        }
        "Undo" => {
            let obj = body
                .get("object")
                .ok_or_else(|| CoreError::Federation("missing 'object' field for Undo".into()))?;
            let target_type = obj
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            let target_id = obj
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            ForgeFedActivity::Undo {
                actor,
                target_type,
                target_id,
            }
        }
        "Add" => {
            let obj = body
                .get("object")
                .ok_or_else(|| CoreError::Federation("missing 'object' field for Add".into()))?;
            let repo_obj = body
                .get("target")
                .ok_or_else(|| CoreError::Federation("missing 'target' for Add review".into()))?;
            let repo = parse_repo(repo_obj).ok_or_else(|| {
                CoreError::Federation("invalid target repository for review".into())
            })?;
            let review = FederatedPRReview {
                id: obj
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                pr_id: obj
                    .get("inReplyTo")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                reviewer: actor.clone(),
                state: match obj
                    .get("result")
                    .and_then(|v| v.as_str())
                    .unwrap_or("pending")
                {
                    "approved" => PRReviewState::Approved,
                    "changesRequested" => PRReviewState::ChangesRequested,
                    "comment" => PRReviewState::Comment,
                    _ => PRReviewState::Pending,
                },
                body: obj
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            };
            ForgeFedActivity::ReviewPullRequest {
                actor,
                repo,
                review,
            }
        }
        other => {
            return Err(CoreError::Federation(format!(
                "unknown activity type: {other}"
            )));
        }
    };

    Ok(activity)
}

pub async fn inbox(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    if !state.config.federation_enabled {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("federation disabled".into()).error_response()),
        )
            .into_response();
    }

    let activity_type = body
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let actor = body
        .get("actor")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    tracing::info!(activity_type = %activity_type, actor = %actor, activity = %body, "received federation inbox activity");

    if let Some(sig_header) = headers.get("Signature") {
        let sig_str = match sig_header.to_str() {
            Ok(s) => s,
            Err(_) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(CoreError::Auth("Invalid HTTP signature".into()).error_response()),
                )
                    .into_response();
            }
        };

        let http_sig = match HttpSignature::from_header_value(sig_str) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(error = %e, "failed to parse Signature header");
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(CoreError::Auth("Invalid HTTP signature".into()).error_response()),
                )
                    .into_response();
            }
        };

        let mut header_map = std::collections::HashMap::new();
        for (k, v) in headers.iter() {
            if let Ok(val) = v.to_str() {
                header_map.insert(k.as_str().to_lowercase(), val.to_string());
            }
        }

        let verifier = SignatureVerifier::new();
        let body_bytes = serde_json::to_vec(&body).unwrap_or_default();

        if !verifier.verify_http_signature(&http_sig, &header_map, &body_bytes, &[]) {
            tracing::warn!(key_id = %http_sig.key_id, "HTTP signature verification failed");
            return (
                StatusCode::UNAUTHORIZED,
                Json(CoreError::Auth("Invalid HTTP signature".into()).error_response()),
            )
                .into_response();
        }
    } else {
        tracing::warn!("no Signature header present, accepting for compatibility");
    }

    match parse_incoming_activity(&body) {
        Ok(activity) => {
            let processor = state.forgefed_processor.clone();
            tokio::spawn(async move {
                match processor.process_incoming(activity) {
                    Ok(outcome) => {
                        tracing::info!(outcome = ?outcome, "forgefed processing complete");
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "forgefed processing failed");
                    }
                }
            });
        }
        Err(e) => {
            tracing::warn!(error = %e, "failed to parse incoming activity, still accepting");
        }
    }

    let resp = InboxResponse {
        status: "accepted".into(),
        message: "Activity received and queued for processing".into(),
    };

    (StatusCode::ACCEPTED, Json(resp)).into_response()
}

pub async fn outbox(State(state): State<AppState>) -> impl IntoResponse {
    if !state.config.federation_enabled {
        return (
            StatusCode::NOT_FOUND,
            Json(CoreError::NotFound("federation disabled".into()).error_response()),
        )
            .into_response();
    }

    let domain = &state.config.federation_instance_domain;

    let resp = OutboxResponse {
        context: "https://www.w3.org/ns/activitystreams".into(),
        id: format!("https://{domain}/api/v1/federation/outbox"),
        type_: "OrderedCollection".into(),
        total_items: 0,
        ordered_items: Vec::new(),
    };

    (StatusCode::OK, Json(resp)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webfinger_response_serialization() {
        let resp = WebFingerResponse {
            subject: "acct:test@localhost".into(),
            aliases: vec!["https://localhost/actor".into()],
            links: vec![WebFingerLink {
                rel: "self".into(),
                type_: Some("application/activity+json".into()),
                href: Some("https://localhost/actor".into()),
            }],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"subject\":\"acct:test@localhost\""));
        assert!(json.contains("\"rel\":\"self\""));
    }

    #[test]
    fn test_webfinger_link_serialization() {
        let link = WebFingerLink {
            rel: "self".into(),
            type_: Some("application/activity+json".into()),
            href: Some("https://example.com/actor".into()),
        };
        let json = serde_json::to_string(&link).unwrap();
        assert!(json.contains("\"type\":\"application/activity+json\""));
        assert!(json.contains("\"href\":\"https://example.com/actor\""));
    }

    #[test]
    fn test_actor_response_serialization() {
        let resp = ActorResponse {
            context: "https://www.w3.org/ns/activitystreams".into(),
            id: "https://example.com/actor".into(),
            type_: "Application".into(),
            preferred_username: "test-instance".into(),
            name: "CivitForge - test-instance".into(),
            inbox: "https://example.com/inbox".into(),
            outbox: "https://example.com/outbox".into(),
            public_key: ActorPublicKey {
                id: "https://example.com/actor#main-key".into(),
                owner: "https://example.com/actor".into(),
                public_key_pem: "KEY".into(),
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"@context\":\"https://www.w3.org/ns/activitystreams\""));
        assert!(json.contains("\"type\":\"Application\""));
    }

    #[test]
    fn test_inbox_response_serialization() {
        let resp = InboxResponse {
            status: "accepted".into(),
            message: "queued".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"status\":\"accepted\""));
    }

    #[test]
    fn test_outbox_response_serialization() {
        let resp = OutboxResponse {
            context: "https://www.w3.org/ns/activitystreams".into(),
            id: "https://example.com/outbox".into(),
            type_: "OrderedCollection".into(),
            total_items: 0,
            ordered_items: Vec::new(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"total_items\":0"));
        assert!(json.contains("\"type\":\"OrderedCollection\""));
    }

    #[test]
    fn test_webfinger_query_parse() {
        let json = r#"{"resource":"acct:test@localhost"}"#;
        let query: WebFingerQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.resource, "acct:test@localhost");
    }

    #[test]
    fn test_parse_create_repository() {
        let body = serde_json::json!({
            "type": "Create",
            "actor": "https://other.forge/users/alice",
            "object": {
                "type": "Repository",
                "id": "repo-1",
                "name": "test-repo",
                "description": "a repo",
                "attributedTo": "alice",
                "visibility": "public"
            }
        });
        let activity = parse_incoming_activity(&body).unwrap();
        assert!(matches!(
            activity,
            ForgeFedActivity::CreateRepository { .. }
        ));
    }

    #[test]
    fn test_parse_create_repository_missing_object() {
        let body = serde_json::json!({
            "type": "Create",
            "actor": "https://other.forge/users/alice"
        });
        assert!(parse_incoming_activity(&body).is_err());
    }

    #[test]
    fn test_parse_create_issue() {
        let body = serde_json::json!({
            "type": "Create",
            "actor": "https://other.forge/users/alice",
            "object": {
                "type": "Issue",
                "id": "issue-1",
                "name": "Bug report",
                "content": "Something broke",
                "state": "open"
            },
            "target": {
                "id": "repo-1",
                "name": "test-repo",
                "attributedTo": "alice"
            }
        });
        let activity = parse_incoming_activity(&body).unwrap();
        assert!(matches!(activity, ForgeFedActivity::CreateIssue { .. }));
        if let ForgeFedActivity::CreateIssue { ref issue, .. } = activity {
            assert_eq!(issue.title, "Bug report");
            assert_eq!(issue.state, IssueState::Open);
        }
    }

    #[test]
    fn test_parse_create_issue_closed_state() {
        let body = serde_json::json!({
            "type": "Create",
            "actor": "https://other.forge/users/alice",
            "object": {
                "type": "Issue",
                "id": "issue-2",
                "name": "Old bug",
                "content": "Already fixed",
                "state": "closed"
            },
            "target": {
                "id": "repo-1",
                "name": "test-repo",
                "attributedTo": "alice"
            }
        });
        let activity = parse_incoming_activity(&body).unwrap();
        if let ForgeFedActivity::CreateIssue { ref issue, .. } = activity {
            assert_eq!(issue.state, IssueState::Closed);
        } else {
            panic!("expected CreateIssue");
        }
    }

    #[test]
    fn test_parse_create_pull_request() {
        let body = serde_json::json!({
            "type": "Create",
            "actor": "https://other.forge/users/alice",
            "object": {
                "type": "PullRequest",
                "id": "pr-1",
                "name": "Fix bug",
                "content": "Fixes issue-1",
                "sourceBranch": "fix-branch",
                "targetBranch": "main",
                "state": "open"
            },
            "target": {
                "id": "repo-1",
                "name": "test-repo",
                "attributedTo": "alice"
            }
        });
        let activity = parse_incoming_activity(&body).unwrap();
        assert!(matches!(
            activity,
            ForgeFedActivity::CreatePullRequest { .. }
        ));
        if let ForgeFedActivity::CreatePullRequest { ref pr, .. } = activity {
            assert_eq!(pr.title, "Fix bug");
            assert_eq!(pr.state, PRState::Open);
        }
    }

    #[test]
    fn test_parse_create_pull_request_merged_state() {
        let body = serde_json::json!({
            "type": "Create",
            "actor": "https://other.forge/users/alice",
            "object": {
                "type": "PullRequest",
                "id": "pr-2",
                "name": "Merged fix",
                "content": "done",
                "sourceBranch": "fix-branch",
                "targetBranch": "main",
                "state": "merged"
            },
            "target": {
                "id": "repo-1",
                "name": "test-repo",
                "attributedTo": "alice"
            }
        });
        let activity = parse_incoming_activity(&body).unwrap();
        if let ForgeFedActivity::CreatePullRequest { ref pr, .. } = activity {
            assert_eq!(pr.state, PRState::Merged);
        } else {
            panic!("expected CreatePullRequest");
        }
    }

    #[test]
    fn test_parse_fork_repository() {
        let body = serde_json::json!({
            "type": "Fork",
            "actor": "https://other.forge/users/bob",
            "object": {
                "id": "repo-1",
                "name": "original",
                "attributedTo": "alice"
            },
            "target": {
                "id": "repo-2",
                "name": "forked",
                "attributedTo": "bob"
            }
        });
        let activity = parse_incoming_activity(&body).unwrap();
        assert!(matches!(activity, ForgeFedActivity::ForkRepository { .. }));
    }

    #[test]
    fn test_parse_like() {
        let body = serde_json::json!({
            "type": "Like",
            "actor": "https://other.forge/users/alice",
            "object": {
                "type": "Comment",
                "id": "comment-1"
            }
        });
        let activity = parse_incoming_activity(&body).unwrap();
        assert!(matches!(activity, ForgeFedActivity::Like { .. }));
        if let ForgeFedActivity::Like {
            ref target_type,
            ref target_id,
            ..
        } = activity
        {
            assert_eq!(target_type, "Comment");
            assert_eq!(target_id, "comment-1");
        }
    }

    #[test]
    fn test_parse_follow() {
        let body = serde_json::json!({
            "type": "Follow",
            "actor": "https://other.forge/users/alice",
            "object": "https://forge.example.com/users/bob"
        });
        let activity = parse_incoming_activity(&body).unwrap();
        assert!(matches!(activity, ForgeFedActivity::FollowUser { .. }));
        if let ForgeFedActivity::FollowUser { ref target, .. } = activity {
            assert_eq!(target, "https://forge.example.com/users/bob");
        }
    }

    #[test]
    fn test_parse_accept() {
        let body = serde_json::json!({
            "type": "Accept",
            "actor": "https://forge.example.com/users/bob",
            "object": "https://other.forge/users/alice"
        });
        let activity = parse_incoming_activity(&body).unwrap();
        assert!(matches!(activity, ForgeFedActivity::Accept { .. }));
    }

    #[test]
    fn test_parse_reject() {
        let body = serde_json::json!({
            "type": "Reject",
            "actor": "https://forge.example.com/users/bob",
            "object": "https://other.forge/users/alice"
        });
        let activity = parse_incoming_activity(&body).unwrap();
        assert!(matches!(activity, ForgeFedActivity::Reject { .. }));
    }

    #[test]
    fn test_parse_undo() {
        let body = serde_json::json!({
            "type": "Undo",
            "actor": "https://other.forge/users/alice",
            "object": {
                "type": "Like",
                "id": "comment-1"
            }
        });
        let activity = parse_incoming_activity(&body).unwrap();
        assert!(matches!(activity, ForgeFedActivity::Undo { .. }));
        if let ForgeFedActivity::Undo {
            ref target_type,
            ref target_id,
            ..
        } = activity
        {
            assert_eq!(target_type, "Like");
            assert_eq!(target_id, "comment-1");
        }
    }

    #[test]
    fn test_parse_add_review() {
        let body = serde_json::json!({
            "type": "Add",
            "actor": "https://other.forge/users/alice",
            "object": {
                "type": "Review",
                "id": "rev-1",
                "inReplyTo": "pr-1",
                "result": "approved",
                "content": "LGTM"
            },
            "target": {
                "id": "repo-1",
                "name": "test-repo",
                "attributedTo": "alice"
            }
        });
        let activity = parse_incoming_activity(&body).unwrap();
        assert!(matches!(
            activity,
            ForgeFedActivity::ReviewPullRequest { .. }
        ));
        if let ForgeFedActivity::ReviewPullRequest { ref review, .. } = activity {
            assert_eq!(review.state, PRReviewState::Approved);
            assert_eq!(review.pr_id, "pr-1");
        }
    }

    #[test]
    fn test_parse_add_review_changes_requested() {
        let body = serde_json::json!({
            "type": "Add",
            "actor": "https://other.forge/users/alice",
            "object": {
                "type": "Review",
                "id": "rev-2",
                "inReplyTo": "pr-2",
                "result": "changesRequested",
                "content": "needs work"
            },
            "target": {
                "id": "repo-1",
                "name": "test-repo",
                "attributedTo": "alice"
            }
        });
        let activity = parse_incoming_activity(&body).unwrap();
        if let ForgeFedActivity::ReviewPullRequest { ref review, .. } = activity {
            assert_eq!(review.state, PRReviewState::ChangesRequested);
        } else {
            panic!("expected ReviewPullRequest");
        }
    }

    #[test]
    fn test_parse_unknown_type() {
        let body = serde_json::json!({
            "type": "Delete",
            "actor": "https://other.forge/users/alice",
            "object": { "id": "x" }
        });
        let err = parse_incoming_activity(&body).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("unknown activity type"));
    }

    #[test]
    fn test_parse_create_unknown_object_type() {
        let body = serde_json::json!({
            "type": "Create",
            "actor": "https://other.forge/users/alice",
            "object": { "type": "Note", "id": "note-1" }
        });
        let err = parse_incoming_activity(&body).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("unknown Create object type"));
    }

    #[test]
    fn test_parse_missing_type_field() {
        let body = serde_json::json!({
            "actor": "https://other.forge/users/alice"
        });
        assert!(parse_incoming_activity(&body).is_err());
    }

    #[test]
    fn test_parse_missing_actor_defaults_empty() {
        let body = serde_json::json!({
            "type": "Like",
            "object": { "type": "Comment", "id": "c1" }
        });
        let activity = parse_incoming_activity(&body).unwrap();
        if let ForgeFedActivity::Like { ref actor, .. } = activity {
            assert_eq!(actor, "");
        } else {
            panic!("expected Like");
        }
    }

    #[test]
    fn test_parse_fork_missing_target() {
        let body = serde_json::json!({
            "type": "Fork",
            "actor": "alice",
            "object": { "id": "r1", "name": "orig", "attributedTo": "a" }
        });
        assert!(parse_incoming_activity(&body).is_err());
    }

    #[test]
    fn test_parse_undo_missing_object() {
        let body = serde_json::json!({
            "type": "Undo",
            "actor": "alice"
        });
        assert!(parse_incoming_activity(&body).is_err());
    }
}
