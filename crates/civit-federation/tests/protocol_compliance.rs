use civit_federation::activitypub::{
    Activity, ActivityObject, ActivityType, Actor, ActorType, InboxHandler, ProcessingResult,
    PublicKey,
};
use civit_federation::forgefed::{
    CrossInstanceIdentityResolver, FederatedIssue, FederatedPR, FederatedPRReview, FederatedRepo,
    ForgeFedActivity, ForgeFedProcessor, IssueState, PRReviewState, PRState,
};
use civit_federation::http_signatures::{
    generate_ed25519_keypair, generate_hmac_key, HttpSignature, HttpSigningConfig,
    SignatureAlgorithm, SignatureVerifier,
};
use civit_federation::inbox_outbox::{
    BackoffStrategy, DeliveryStatus, InboxProcessor, OutboxProcessor,
};
use civit_federation::webfinger::{resolve_webfinger, Link, WebFingerResponse};
use serde_json::Value;
use std::collections::HashMap;

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn load_fixture(name: &str) -> Value {
    let path = format!("tests/fixtures/{name}");
    let content = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
    serde_json::from_str(&content).unwrap_or_else(|e| panic!("failed to parse {path}: {e}"))
}

fn make_actor() -> Actor {
    Actor {
        id: "https://forge.example.com/users/alice".into(),
        r#type: ActorType::Person,
        preferred_username: "alice".into(),
        inbox: "https://forge.example.com/users/alice/inbox".into(),
        outbox: "https://forge.example.com/users/alice/outbox".into(),
        public_key: PublicKey {
            id: "https://forge.example.com/users/alice#main-key".into(),
            owner: "https://forge.example.com/users/alice".into(),
            public_key_pem: "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8A\n-----END PUBLIC KEY-----".into(),
        },
        endpoints: HashMap::new(),
    }
}

fn make_create_activity() -> Activity {
    Activity {
        r#type: ActivityType::Create,
        id: "https://forge.example.com/activities/create-1".into(),
        actor: "https://forge.example.com/users/alice".into(),
        object: ActivityObject::Repository {
            id: "https://forge.example.com/repos/alice/test-repo".into(),
            name: "test-repo".into(),
            attributed_to: "https://forge.example.com/users/alice".into(),
        },
        target: None,
        published: "2025-01-15T10:30:00Z".into(),
        to: vec!["https://www.w3.org/ns/activitystreams#Public".into()],
        cc: vec![],
    }
}

fn make_handler() -> InboxHandler {
    InboxHandler::new("inst-1".into(), "forge.example.com".into())
}

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

// ─── 1. ActivityPub Object Serialization ─────────────────────────────────────

#[test]
fn test_create_activity_serialization_roundtrip() {
    let activity = make_create_activity();
    let json = serde_json::to_string(&activity).unwrap();
    let deserialized: Activity = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.r#type, ActivityType::Create);
    assert_eq!(deserialized.id, activity.id);
    assert_eq!(deserialized.actor, activity.actor);
    assert_eq!(deserialized.to, activity.to);
}

#[test]
fn test_update_activity_serialization() {
    let activity = Activity {
        r#type: ActivityType::Update,
        id: "https://forge.example.com/activities/update-1".into(),
        actor: "https://forge.example.com/users/alice".into(),
        object: ActivityObject::Repository {
            id: "repo-1".into(),
            name: "test-repo".into(),
            attributed_to: "https://forge.example.com/users/alice".into(),
        },
        target: None,
        published: "2025-01-16T14:00:00Z".into(),
        to: vec!["https://www.w3.org/ns/activitystreams#Public".into()],
        cc: vec![],
    };
    let json = serde_json::to_string(&activity).unwrap();
    assert!(json.contains(r#""type":"Update""#));
    let deserialized: Activity = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.r#type, ActivityType::Update);
}

#[test]
fn test_delete_activity_serialization() {
    let activity = Activity {
        r#type: ActivityType::Delete,
        id: "https://forge.example.com/activities/delete-1".into(),
        actor: "https://forge.example.com/users/alice".into(),
        object: ActivityObject::Repository {
            id: "repo-2".into(),
            name: "old-repo".into(),
            attributed_to: "https://forge.example.com/users/alice".into(),
        },
        target: None,
        published: "2025-01-17T09:00:00Z".into(),
        to: vec!["https://www.w3.org/ns/activitystreams#Public".into()],
        cc: vec![],
    };
    let json = serde_json::to_string(&activity).unwrap();
    assert!(json.contains(r#""type":"Delete""#));
    let deserialized: Activity = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.r#type, ActivityType::Delete);
}

#[test]
fn test_activity_object_variants_serialize() {
    let objects = vec![
        ActivityObject::Repository {
            id: "r1".into(),
            name: "repo".into(),
            attributed_to: "a1".into(),
        },
        ActivityObject::Commit {
            id: "c1".into(),
            message: "fix bug".into(),
            attributed_to: "a1".into(),
        },
        ActivityObject::Issue {
            id: "i1".into(),
            name: "issue title".into(),
            attributed_to: "a1".into(),
        },
        ActivityObject::PullRequest {
            id: "p1".into(),
            name: "pr title".into(),
            attributed_to: "a1".into(),
        },
        ActivityObject::Note {
            id: "n1".into(),
            content: "hello".into(),
            attributed_to: "a1".into(),
        },
        ActivityObject::Unknown(serde_json::json!({"type": "Custom"})),
    ];

    for obj in objects {
        let json = serde_json::to_string(&obj).unwrap();
        let deserialized: ActivityObject = serde_json::from_str(&json).unwrap();
        assert!(!json.is_empty());
        // Verify the roundtrip produced a valid object
        match deserialized {
            ActivityObject::Unknown(v) => assert_eq!(v["type"], "Custom"),
            _ => {}
        }
    }
}

#[test]
fn test_activity_type_serialization_names() {
    assert_eq!(
        serde_json::to_string(&ActivityType::Create).unwrap(),
        r#""Create""#
    );
    assert_eq!(
        serde_json::to_string(&ActivityType::Update).unwrap(),
        r#""Update""#
    );
    assert_eq!(
        serde_json::to_string(&ActivityType::Delete).unwrap(),
        r#""Delete""#
    );
    assert_eq!(
        serde_json::to_string(&ActivityType::Follow).unwrap(),
        r#""Follow""#
    );
    assert_eq!(
        serde_json::to_string(&ActivityType::Undo).unwrap(),
        r#""Undo""#
    );
    assert_eq!(
        serde_json::to_string(&ActivityType::Accept).unwrap(),
        r#""Accept""#
    );
    assert_eq!(
        serde_json::to_string(&ActivityType::Reject).unwrap(),
        r#""Reject""#
    );
    assert_eq!(
        serde_json::to_string(&ActivityType::Like).unwrap(),
        r#""Like""#
    );
}

#[test]
fn test_fixture_create_activity_parsed() {
    let fixture = load_fixture("activitypub_create.json");
    assert_eq!(fixture["type"], "Create");
    assert_eq!(fixture["actor"], "https://forge.example.com/users/alice");
    assert_eq!(fixture["object"]["type"], "Repository");
    assert_eq!(fixture["object"]["name"], "test-repo");
}

#[test]
fn test_fixture_update_activity_parsed() {
    let fixture = load_fixture("activitypub_update.json");
    assert_eq!(fixture["type"], "Update");
    assert_eq!(
        fixture["object"]["content"],
        "Updated description for federation testing"
    );
}

#[test]
fn test_fixture_delete_activity_parsed() {
    let fixture = load_fixture("activitypub_delete.json");
    assert_eq!(fixture["type"], "Delete");
    assert_eq!(fixture["object"]["name"], "old-repo");
}

// ─── 2. HTTP Signature Generation and Verification ───────────────────────────

#[test]
fn test_ed25519_sign_and_verify_roundtrip() {
    let (private_key, public_key) = generate_ed25519_keypair();
    let verifier = SignatureVerifier::new();
    let config = HttpSigningConfig {
        required_headers: vec![
            "(request-target)".into(),
            "host".into(),
            "date".into(),
        ],
        algorithm: SignatureAlgorithm::Ed25519,
        expires_in_secs: 300,
    };

    let mut headers = HashMap::new();
    headers.insert("(method)".into(), "POST".into());
    headers.insert("(path)".into(), "/inbox".into());
    headers.insert("host".into(), "forge.example.com".into());
    headers.insert("date".into(), "2025-01-15T10:30:00Z".into());

    let sig = verifier
        .sign_request(&config, &headers, b"{\"type\":\"Create\"}", &private_key, "key-1")
        .unwrap();

    assert!(!sig.signature.is_empty());
    assert!(verifier.verify_http_signature(&sig, &headers, b"{\"type\":\"Create\"}", &public_key));
}

#[test]
fn test_hmac_sha256_sign_and_verify_roundtrip() {
    let hmac_key = generate_hmac_key();
    let verifier = SignatureVerifier::new();
    let config = HttpSigningConfig {
        required_headers: vec!["(request-target)".into(), "host".into()],
        algorithm: SignatureAlgorithm::HmacSha256,
        expires_in_secs: 60,
    };

    let mut headers = HashMap::new();
    headers.insert("(method)".into(), "GET".into());
    headers.insert("(path)".into(), "/actor".into());
    headers.insert("host".into(), "forge.example.com".into());

    let sig = verifier
        .sign_request(&config, &headers, &[], &hmac_key, "hmac-key-1")
        .unwrap();

    assert!(verifier.verify_http_signature(&sig, &headers, &[], &hmac_key));
}

#[test]
fn test_signature_rejects_tampered_body() {
    let (private_key, public_key) = generate_ed25519_keypair();
    let verifier = SignatureVerifier::new();
    let config = HttpSigningConfig {
        required_headers: vec!["(request-target)".into()],
        algorithm: SignatureAlgorithm::Ed25519,
        expires_in_secs: 300,
    };

    let mut headers = HashMap::new();
    headers.insert("(method)".into(), "POST".into());
    headers.insert("(path)".into(), "/inbox".into());

    let sig = verifier
        .sign_request(&config, &headers, b"original", &private_key, "key-1")
        .unwrap();

    // Verification with different body should fail
    assert!(!verifier.verify_http_signature(&sig, &headers, b"tampered", &public_key));
}

#[test]
fn test_signature_rejects_wrong_key() {
    let (_, public_key) = generate_ed25519_keypair();
    let (wrong_private_key, _) = generate_ed25519_keypair();
    let verifier = SignatureVerifier::new();
    let config = HttpSigningConfig {
        required_headers: vec!["(request-target)".into()],
        algorithm: SignatureAlgorithm::Ed25519,
        expires_in_secs: 300,
    };

    let mut headers = HashMap::new();
    headers.insert("(method)".into(), "POST".into());
    headers.insert("(path)".into(), "/inbox".into());

    let sig = verifier
        .sign_request(&config, &headers, b"{}", &wrong_private_key, "key-1")
        .unwrap();

    assert!(!verifier.verify_http_signature(&sig, &headers, b"{}", &public_key));
}

#[test]
fn test_signature_header_value_roundtrip() {
    let sig = HttpSignature {
        key_id: "https://forge.example.com/users/alice#main-key".into(),
        algorithm: SignatureAlgorithm::Ed25519,
        created: chrono::DateTime::from_timestamp(1700000000, 0).unwrap(),
        expires: chrono::DateTime::from_timestamp(1700000300, 0).unwrap(),
        headers: vec!["(request-target)".into(), "host".into(), "date".into()],
        signature: "abc123signature".into(),
    };

    let header_val = sig.to_header_value();
    let parsed = HttpSignature::from_header_value(&header_val).unwrap();

    assert_eq!(parsed.key_id, sig.key_id);
    assert_eq!(parsed.algorithm, sig.algorithm);
    assert_eq!(parsed.headers, sig.headers);
    assert_eq!(parsed.signature, sig.signature);
}

#[test]
fn test_signature_expired_rejects() {
    let (_, public_key) = generate_ed25519_keypair();
    let verifier = SignatureVerifier::new();

    let sig = HttpSignature {
        key_id: "key-1".into(),
        algorithm: SignatureAlgorithm::Ed25519,
        created: chrono::Utc::now() - chrono::Duration::hours(2),
        expires: chrono::Utc::now() - chrono::Duration::hours(1),
        headers: vec!["(request-target)".into()],
        signature: "expired".into(),
    };

    assert!(!verifier.verify_http_signature(&sig, &HashMap::new(), &[], &public_key));
}

#[test]
fn test_signature_algorithm_parse_roundtrip() {
    let algos = vec![
        SignatureAlgorithm::RsaSha256,
        SignatureAlgorithm::EcdsaP256,
        SignatureAlgorithm::HmacSha256,
        SignatureAlgorithm::Ed25519,
    ];
    for algo in algos {
        let s = algo.to_string();
        let parsed: SignatureAlgorithm = s.parse().unwrap();
        assert_eq!(parsed, algo);
    }
}

// ─── 3. WebFinger Discovery ──────────────────────────────────────────────────

#[tokio::test]
async fn test_webfinger_fallback_response() {
    let response = resolve_webfinger("nonexistent.invalid.tld", "alice")
        .await
        .unwrap();
    assert_eq!(response.subject, "acct:alice@nonexistent.invalid.tld");
    assert!(!response.aliases.is_empty());
    assert!(!response.links.is_empty());
}

#[tokio::test]
async fn test_webfinger_rejects_empty_domain() {
    let result = resolve_webfinger("", "alice").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_webfinger_rejects_empty_username() {
    let result = resolve_webfinger("forge.example.com", "").await;
    assert!(result.is_err());
}

#[test]
fn test_webfinger_response_structure() {
    let response = WebFingerResponse {
        subject: "acct:bob@forge.example.com".into(),
        aliases: vec!["https://forge.example.com/users/bob".into()],
        links: vec![Link {
            rel: "self".into(),
            type_: "application/activity+json".into(),
            href: "https://forge.example.com/users/bob".into(),
        }],
    };

    assert_eq!(response.subject, "acct:bob@forge.example.com");
    assert_eq!(response.aliases.len(), 1);
    assert_eq!(response.links.len(), 1);
    assert_eq!(response.links[0].rel, "self");
    assert_eq!(response.links[0].type_, "application/activity+json");
}

#[test]
fn test_webfinger_fixture_parsed() {
    let fixture = load_fixture("webfinger_response.json");
    assert_eq!(fixture["subject"], "acct:alice@forge.example.com");
    let links = fixture["links"].as_array().unwrap();
    assert!(!links.is_empty());
    let self_link = &links[0];
    assert_eq!(self_link["rel"], "self");
    assert_eq!(self_link["type"], "application/activity+json");
}

#[test]
fn test_handler_builds_webfinger_response() {
    let handler = make_handler();
    let response = handler.build_webfinger_response("alice");
    let json = serde_json::to_value(&response).unwrap();
    assert_eq!(json["subject"], "acct:alice@forge.example.com");
    let links = json["links"].as_array().unwrap();
    assert!(!links.is_empty());
    assert_eq!(links[0]["rel"], "self");
    assert_eq!(links[0]["type"], "application/activity+json");
}

// ─── 4. Inbox/Outbox Message Handling ────────────────────────────────────────

#[test]
fn test_inbox_receives_and_processes_create() {
    let mut inbox = InboxProcessor::new();
    let activity = make_create_activity();
    let federated = civit_federation::inbox_outbox::FederatedActivity {
        id: activity.id.clone(),
        type_: "Create".into(),
        actor: activity.actor.clone(),
        object: Some("https://forge.example.com/repos/alice/test-repo".into()),
        target: None,
        to: activity.to.clone(),
        cc: activity.cc.clone(),
        published: None,
        raw_json: serde_json::to_string(&activity).unwrap(),
    };

    assert!(inbox.receive(federated, "key-create-1".into()));
    assert_eq!(inbox.pending_count(), 1);

    let result = inbox.process_next(|_| {
        civit_federation::inbox_outbox::ProcessingResult::Success
    });
    assert!(result.is_none());
    assert!(inbox.is_empty());
}

#[test]
fn test_inbox_idempotency_rejects_duplicate() {
    let mut inbox = InboxProcessor::new();
    let activity = civit_federation::inbox_outbox::FederatedActivity {
        id: "act-1".into(),
        type_: "Create".into(),
        actor: "actor-1".into(),
        object: None,
        target: None,
        to: vec![],
        cc: vec![],
        published: None,
        raw_json: "{}".into(),
    };

    assert!(inbox.receive(activity.clone(), "dup-key".into()));
    assert!(!inbox.receive(activity, "dup-key".into()));
    assert_eq!(inbox.pending_count(), 1);
}

#[test]
fn test_inbox_retry_on_failure() {
    let mut inbox = InboxProcessor::with_max_retries(3);
    let activity = civit_federation::inbox_outbox::FederatedActivity {
        id: "act-fail".into(),
        type_: "Create".into(),
        actor: "actor-1".into(),
        object: None,
        target: None,
        to: vec![],
        cc: vec![],
        published: None,
        raw_json: "{}".into(),
    };

    inbox.receive(activity, "fail-key".into());
    inbox.process_next(|_| {
        civit_federation::inbox_outbox::ProcessingResult::Failed("timeout".into())
    });
    assert_eq!(inbox.pending_count(), 1);

    let retried = inbox.retry_failed();
    assert_eq!(retried, 1);
}

#[test]
fn test_outbox_enqueue_and_deliver() {
    let mut outbox = OutboxProcessor::new();
    let activity = civit_federation::inbox_outbox::FederatedActivity {
        id: "act-out".into(),
        type_: "Create".into(),
        actor: "actor-1".into(),
        object: None,
        target: None,
        to: vec![],
        cc: vec![],
        published: None,
        raw_json: "{}".into(),
    };

    outbox.enqueue(activity, "https://remote.forge".into());
    assert_eq!(outbox.pending_count(), 1);

    outbox.mark_in_flight("act-out", "https://remote.forge");
    assert_eq!(outbox.in_flight_count(), 1);

    outbox.mark_delivered("act-out", "https://remote.forge");
    assert_eq!(outbox.delivered_count(), 1);
}

#[test]
fn test_outbox_retry_backoff() {
    let mut outbox = OutboxProcessor::with_backoff(BackoffStrategy::Exponential {
        base_ms: 1000,
        max_ms: 60_000,
    });
    let activity = civit_federation::inbox_outbox::FederatedActivity {
        id: "act-retry".into(),
        type_: "Create".into(),
        actor: "actor-1".into(),
        object: None,
        target: None,
        to: vec![],
        cc: vec![],
        published: None,
        raw_json: "{}".into(),
    };

    outbox.enqueue(activity, "https://remote.forge".into());
    outbox.mark_failed("act-retry", "https://remote.forge", false);
    assert_eq!(outbox.pending_count(), 0);
}

#[test]
fn test_outbox_permanent_failure() {
    let mut outbox = OutboxProcessor::new();
    let activity = civit_federation::inbox_outbox::FederatedActivity {
        id: "act-perm".into(),
        type_: "Create".into(),
        actor: "actor-1".into(),
        object: None,
        target: None,
        to: vec![],
        cc: vec![],
        published: None,
        raw_json: "{}".into(),
    };

    outbox.enqueue(activity, "https://remote.forge".into());
    outbox.mark_failed("act-perm", "https://remote.forge", true);
    let ready = outbox.retry_ready();
    assert!(ready.is_empty());
}

// ─── 5. Actor Profile Resolution ─────────────────────────────────────────────

#[test]
fn test_actor_validation_valid() {
    let handler = make_handler();
    let actor = make_actor();
    assert!(handler.validate_actor(&actor).is_ok());
}

#[test]
fn test_actor_validation_rejects_empty_id() {
    let handler = make_handler();
    let mut actor = make_actor();
    actor.id = "".into();
    assert!(handler.validate_actor(&actor).is_err());
}

#[test]
fn test_actor_validation_rejects_empty_inbox() {
    let handler = make_handler();
    let mut actor = make_actor();
    actor.inbox = "".into();
    assert!(handler.validate_actor(&actor).is_err());
}

#[test]
fn test_actor_validation_rejects_empty_username() {
    let handler = make_handler();
    let mut actor = make_actor();
    actor.preferred_username = "".into();
    assert!(handler.validate_actor(&actor).is_err());
}

#[test]
fn test_actor_profile_fixture() {
    let fixture = load_fixture("actor_profile.json");
    assert_eq!(fixture["type"], "Person");
    assert_eq!(fixture["preferredUsername"], "alice");
    assert_eq!(
        fixture["inbox"],
        "https://forge.example.com/users/alice/inbox"
    );
    assert_eq!(
        fixture["outbox"],
        "https://forge.example.com/users/alice/outbox"
    );
    assert!(fixture["publicKey"].is_object());
    assert!(fixture["publicKey"]["publicKeyPem"].is_string());
}

#[test]
fn test_actor_serialization_roundtrip() {
    let actor = make_actor();
    let json = serde_json::to_string(&actor).unwrap();
    let deserialized: Actor = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.r#type, ActorType::Person);
    assert_eq!(deserialized.preferred_username, "alice");
    assert_eq!(deserialized.inbox, actor.inbox);
}

#[test]
fn test_actor_type_variants() {
    let types = vec![
        ActorType::Person,
        ActorType::Organization,
        ActorType::Application,
        ActorType::Service,
        ActorType::Group,
    ];
    for actor_type in types {
        let json = serde_json::to_string(&actor_type).unwrap();
        let deserialized: ActorType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, actor_type);
    }
}

// ─── 6. Collection Pages ─────────────────────────────────────────────────────

#[test]
fn test_cross_instance_identity_resolver() {
    let resolver = CrossInstanceIdentityResolver::new("forge.example.com".into());
    assert_eq!(
        resolver.resolve_local("alice"),
        "https://forge.example.com/users/alice"
    );
    assert!(resolver.is_local("https://forge.example.com/users/alice"));
    assert!(!resolver.is_local("https://other.forge/users/bob"));
}

#[test]
fn test_identity_cache_operations() {
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
fn test_federation_uri_format() {
    let resolver = CrossInstanceIdentityResolver::new("forge.example.com".into());
    let uri = resolver.federation_uri("other.forge", "bob");
    assert_eq!(uri, "acct:bob@other.forge");
}

// ─── 7. LD+JSON Context and Types ────────────────────────────────────────────

#[test]
fn test_create_activity_ld_context() {
    let fixture = load_fixture("activitypub_create.json");
    let context = fixture["@context"].as_array().unwrap();
    assert!(context.contains(&Value::String("https://www.w3.org/ns/activitystreams".into())));
    assert!(context.contains(&Value::String("https://forgefed.org/ns".into())));
}

#[test]
fn test_actor_profile_ld_context() {
    let fixture = load_fixture("actor_profile.json");
    let context = fixture["@context"].as_array().unwrap();
    assert!(context.contains(&Value::String("https://www.w3.org/ns/activitystreams".into())));
    assert!(context.contains(&Value::String("https://w3id.org/security/v1".into())));
}

#[test]
fn test_activity_object_types() {
    let fixture = load_fixture("activitypub_create.json");
    assert_eq!(fixture["object"]["type"], "Repository");

    let fixture = load_fixture("activitypub_update.json");
    assert_eq!(fixture["object"]["type"], "Repository");

    let fixture = load_fixture("activitypub_delete.json");
    assert_eq!(fixture["object"]["type"], "Repository");
}

#[test]
fn test_webfinger_self_link_has_activity_json_type() {
    let fixture = load_fixture("webfinger_response.json");
    let links = fixture["links"].as_array().unwrap();
    let self_link = links.iter().find(|l| l["rel"] == "self").unwrap();
    assert_eq!(self_link["type"], "application/activity+json");
}

// ─── 8. Error Handling ───────────────────────────────────────────────────────

#[test]
fn test_handler_rejects_empty_actor_activity() {
    let handler = make_handler();
    let activity = Activity {
        r#type: ActivityType::Create,
        id: "act-1".into(),
        actor: "".into(),
        object: ActivityObject::Note {
            id: "n1".into(),
            content: "test".into(),
            attributed_to: "a1".into(),
        },
        target: None,
        published: "2025-01-01T00:00:00Z".into(),
        to: vec!["https://www.w3.org/ns/activitystreams#Public".into()],
        cc: vec![],
    };
    let result = handler.validate_activity(&activity);
    assert!(result.is_err());
}

#[test]
fn test_handler_rejects_no_recipient() {
    let handler = make_handler();
    let activity = Activity {
        r#type: ActivityType::Create,
        id: "act-1".into(),
        actor: "https://forge.example.com/users/alice".into(),
        object: ActivityObject::Note {
            id: "n1".into(),
            content: "test".into(),
            attributed_to: "a1".into(),
        },
        target: None,
        published: "2025-01-01T00:00:00Z".into(),
        to: vec![],
        cc: vec![],
    };
    let result = handler.validate_activity(&activity);
    assert!(result.is_err());
}

#[test]
fn test_forgefed_rejects_missing_actor() {
    let proc = make_processor();
    let activity = ForgeFedActivity::CreateRepository {
        actor: "".into(),
        repo: make_repo(),
    };
    let outcome = proc.process_incoming(activity).unwrap();
    match outcome {
        civit_federation::forgefed::ProcessingOutcome::Rejected { reason, .. } => {
            assert!(reason.contains("missing"));
        }
        _ => panic!("expected Rejected for missing actor"),
    }
}

#[test]
fn test_forgefed_rejects_missing_repo_id() {
    let proc = make_processor();
    let activity = ForgeFedActivity::StarRepository {
        actor: "alice".into(),
        repo: FederatedRepo {
            id: "".into(),
            name: "repo".into(),
            description: "".into(),
            owner: "alice".into(),
            visibility: "public".into(),
        },
    };
    let outcome = proc.process_incoming(activity).unwrap();
    match outcome {
        civit_federation::forgefed::ProcessingOutcome::Rejected { .. } => {}
        _ => panic!("expected Rejected for missing repo id"),
    }
}

#[test]
fn test_forgefed_rejects_missing_pr_reviewer() {
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
        civit_federation::forgefed::ProcessingOutcome::Rejected { .. } => {}
        _ => panic!("expected Rejected for missing reviewer"),
    }
}

#[test]
fn test_processing_result_variants() {
    let results = vec![
        ProcessingResult::Accepted {
            message: "ok".into(),
            id: "1".into(),
        },
        ProcessingResult::Pending {
            message: "pending".into(),
            id: "2".into(),
        },
        ProcessingResult::Rejected {
            message: "no".into(),
            id: "3".into(),
        },
    ];
    assert_eq!(results.len(), 3);
}

#[test]
fn test_inbox_handler_processes_all_activity_types() {
    let handler = make_handler();
    let types_and_expected = vec![
        (ActivityType::Create, "resource created"),
        (ActivityType::Update, "resource updated"),
        (ActivityType::Delete, "resource deleted"),
        (ActivityType::Follow, "follow request pending"),
        (ActivityType::Accept, "follow accepted"),
        (ActivityType::Reject, "activity rejected"),
        (ActivityType::Like, "activity processed"),
        (ActivityType::Undo, "activity processed"),
        (ActivityType::Announce, "activity processed"),
        (ActivityType::Add, "activity processed"),
    ];

    for (activity_type, expected_msg) in types_and_expected {
        let activity = Activity {
            r#type: activity_type.clone(),
            id: "act-1".into(),
            actor: "https://forge.example.com/users/alice".into(),
            object: ActivityObject::Note {
                id: "n1".into(),
                content: "test".into(),
                attributed_to: "a1".into(),
            },
            target: None,
            published: "2025-01-01T00:00:00Z".into(),
            to: vec!["https://www.w3.org/ns/activitystreams#Public".into()],
            cc: vec![],
        };
        let result = handler.process_incoming(activity).unwrap();
        match result {
            ProcessingResult::Accepted { message, .. }
            | ProcessingResult::Pending { message, .. }
            | ProcessingResult::Rejected { message, .. } => {
                assert_eq!(message, expected_msg);
            }
        }
    }
}

// ─── 9. ForgeFed Activity Processing ─────────────────────────────────────────

#[test]
fn test_forgefed_create_repository_accepted() {
    let proc = make_processor();
    let activity = ForgeFedActivity::CreateRepository {
        actor: "https://other.forge/users/alice".into(),
        repo: make_repo(),
    };
    let outcome = proc.process_incoming(activity).unwrap();
    match outcome {
        civit_federation::forgefed::ProcessingOutcome::Accepted { message, .. } => {
            assert!(message.contains("repository test-repo created"));
        }
        _ => panic!("expected Accepted"),
    }
}

#[test]
fn test_forgefed_fork_repository() {
    let proc = make_processor();
    let target = FederatedRepo {
        id: "repo-2".into(),
        name: "fork-repo".into(),
        description: "a fork".into(),
        owner: "bob".into(),
        visibility: "public".into(),
    };
    let activity = ForgeFedActivity::ForkRepository {
        actor: "https://other.forge/users/bob".into(),
        source: make_repo(),
        target,
    };
    let outcome = proc.process_incoming(activity).unwrap();
    match outcome {
        civit_federation::forgefed::ProcessingOutcome::Accepted { message, .. } => {
            assert!(message.contains("forked"));
        }
        _ => panic!("expected Accepted"),
    }
}

#[test]
fn test_forgefed_create_issue() {
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
        civit_federation::forgefed::ProcessingOutcome::Accepted { message, .. } => {
            assert!(message.contains("issue 'Bug report'"));
            assert!(message.contains("Open"));
        }
        _ => panic!("expected Accepted"),
    }
}

#[test]
fn test_forgefed_create_pr() {
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
        civit_federation::forgefed::ProcessingOutcome::Accepted { message, .. } => {
            assert!(message.contains("PR 'Fix bug' created"));
        }
        _ => panic!("expected Accepted"),
    }
}

#[test]
fn test_forgefed_idempotency_duplicate_rejection() {
    let proc = make_processor();
    let activity = ForgeFedActivity::StarRepository {
        actor: "alice".into(),
        repo: make_repo(),
    };
    let _ = proc.process_incoming(activity.clone()).unwrap();
    let outcome = proc.process_incoming(activity).unwrap();
    match outcome {
        civit_federation::forgefed::ProcessingOutcome::Duplicate { .. } => {}
        _ => panic!("expected Duplicate"),
    }
}

#[test]
fn test_forgefed_outbox_activity_mapping() {
    let proc = make_processor();

    let repo_activity = ForgeFedActivity::CreateRepository {
        actor: "https://forge.example.com/users/alice".into(),
        repo: make_repo(),
    };
    let outbox = proc.build_outbox_activity(repo_activity);
    assert_eq!(outbox.r#type, ActivityType::Create);
    assert!(!outbox.id.is_empty());
    assert_eq!(
        outbox.to,
        vec!["https://www.w3.org/ns/activitystreams#Public"]
    );

    let follow_activity = ForgeFedActivity::FollowUser {
        actor: "https://forge.example.com/users/alice".into(),
        target: "https://other.forge/users/bob".into(),
    };
    let outbox = proc.build_outbox_activity(follow_activity);
    assert_eq!(outbox.r#type, ActivityType::Follow);

    let star_activity = ForgeFedActivity::StarRepository {
        actor: "https://forge.example.com/users/alice".into(),
        repo: make_repo(),
    };
    let outbox = proc.build_outbox_activity(star_activity);
    assert_eq!(outbox.r#type, ActivityType::Like);
}
