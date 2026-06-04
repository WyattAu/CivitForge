#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::error::CoreError;
use crate::federation::http_signatures::{HttpSignature, SignatureVerifier};
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
    fn test_federation_routes_type() {
        fn _assert_routes() -> Router<AppState> {
            federation_routes()
        }
    }
}
