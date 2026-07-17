#![forbid(unsafe_code)]

use crate::http_signatures::{
    HttpSigningConfig, SignatureAlgorithm, SignatureVerifier,
};
use crate::inbox_outbox::{BackoffStrategy, FederatedActivity, OutboxProcessor};
use crate::webfinger::resolve_webfinger;
use base64::Engine;
use base64::prelude::BASE64_STANDARD as BASE64;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, watch};
use tracing::{error, info, warn};

/// Configuration for the federation delivery service.
#[derive(Debug, Clone)]
pub struct FederationDeliveryConfig {
    /// Ed25519 private key bytes (PKCS#8 DER).
    pub private_key: Vec<u8>,
    /// Key ID for HTTP Signature headers (e.g., "main-key-2024").
    pub key_id: String,
    /// Maximum concurrent deliveries.
    pub max_concurrent: usize,
    /// Maximum delivery attempts before permanent failure.
    pub max_attempts: u32,
    /// Base backoff in milliseconds.
    pub backoff_base_ms: u64,
    /// Maximum backoff in milliseconds.
    pub backoff_max_ms: u64,
    /// Whether to add jitter to backoff delays.
    pub jitter_enabled: bool,
    /// Jitter factor (0.0-1.0). 0.5 = ±25%.
    pub jitter_factor: f64,
    /// HTTP request timeout.
    pub http_timeout: Duration,
    /// How many entries to drain per delivery cycle.
    pub batch_size: usize,
}

impl Default for FederationDeliveryConfig {
    fn default() -> Self {
        Self {
            private_key: Vec::new(),
            key_id: "main-key".into(),
            max_concurrent: 10,
            max_attempts: 5,
            backoff_base_ms: 1000,
            backoff_max_ms: 300_000,
            jitter_enabled: true,
            jitter_factor: 0.5,
            http_timeout: Duration::from_secs(30),
            batch_size: 25,
        }
    }
}

impl FederationDeliveryConfig {
    /// Build the signing config used by `SignatureVerifier`.
    pub fn signing_config(&self) -> HttpSigningConfig {
        HttpSigningConfig {
            algorithm: SignatureAlgorithm::Ed25519,
            required_headers: vec![
                "(request-target)".into(),
                "host".into(),
                "date".into(),
                "digest".into(),
            ],
            expires_in_secs: 300,
        }
    }
}

/// Errors from federation delivery operations.
#[derive(Debug, thiserror::Error)]
pub enum DeliveryError {
    #[error("webfinger resolution failed for {target}: {detail}")]
    WebFingerFailed { target: String, detail: String },

    #[error("no inbox URL found for {target}")]
    NoInboxUrl { target: String },

    #[error("HTTP signature error: {0}")]
    SignatureError(String),

    #[error("HTTP delivery failed: {status} for {inbox}")]
    HttpError { status: u16, inbox: String },

    #[error("HTTP client error: {0}")]
    HttpClientError(String),

    #[error("no private key configured")]
    NoPrivateKey,
}

/// Result of a single delivery attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum DeliveryResult {
    /// Activity was accepted by the remote inbox (2xx).
    Accepted,
    /// Remote returned 4xx — permanent failure, do not retry.
    Rejected,
    /// Remote returned 5xx — transient failure, retry with backoff.
    TransientFailure,
    /// Network error — retry with backoff.
    NetworkError,
    /// Activity was already delivered (idempotent check).
    AlreadyDelivered,
}

/// Cached remote actor information resolved via WebFinger.
#[derive(Debug, Clone)]
struct CachedActor {
    /// The inbox URL for this actor.
    inbox_url: String,
    /// When this cache entry was created.
    cached_at: std::time::Instant,
    /// TTL for this cache entry.
    ttl: Duration,
}

impl CachedActor {
    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }
}

/// Federation delivery service — the transport glue between the outbox queue
/// and remote ActivityPub inboxes.
pub struct FederationDeliveryService {
    config: FederationDeliveryConfig,
    http_client: reqwest::Client,
    outbox: Arc<Mutex<OutboxProcessor>>,
    actor_cache: Arc<Mutex<HashMap<String, CachedActor>>>,
    actor_cache_ttl: Duration,
    verifier: SignatureVerifier,
}

impl FederationDeliveryService {
    /// Create a new delivery service.
    pub fn new(config: FederationDeliveryConfig) -> std::result::Result<Self, DeliveryError> {
        let verifier = SignatureVerifier::new();
        let backoff = BackoffStrategy::Exponential {
            base_ms: config.backoff_base_ms,
            max_ms: config.backoff_max_ms,
        };
        let http_client = reqwest::Client::builder()
            .timeout(config.http_timeout)
            .user_agent("CivitForge/0.1 (ForgeFed)")
            .build()
            .map_err(|e| DeliveryError::HttpClientError(e.to_string()))?;

        Ok(Self {
            config,
            http_client,
            outbox: Arc::new(Mutex::new(OutboxProcessor::with_backoff(backoff))),
            actor_cache: Arc::new(Mutex::new(HashMap::new())),
            actor_cache_ttl: Duration::from_secs(300),
            verifier,
        })
    }

    /// Enqueue an activity for delivery to a remote instance.
    pub async fn enqueue(&self, activity: FederatedActivity, target_instance: &str) -> String {
        let mut outbox = self.outbox.lock().await;
        outbox.enqueue(activity, target_instance.to_string())
    }

    /// Run the delivery loop until cancelled.
    pub async fn run_until_cancelled(&self, mut cancel: watch::Receiver<bool>) {
        info!("federation delivery service started");

        loop {
            tokio::select! {
                _ = cancel.changed() => {
                    info!("federation delivery service shutting down");
                    break;
                }
                _ = tokio::time::sleep(Duration::from_secs(5)) => {
                    if let Err(e) = self.deliver_batch().await {
                        error!(error = %e, "delivery batch error");
                    }
                }
            }
        }
    }

    /// Deliver a batch of ready entries.
    async fn deliver_batch(&self) -> std::result::Result<(), DeliveryError> {
        let pending: Vec<(String, String)>;
        let retry_ready: Vec<(String, String)>;
        {
            let mut outbox = self.outbox.lock().await;
            pending = outbox.drain_pending(self.config.batch_size);
            retry_ready = outbox.drain_retry_ready(self.config.batch_size);
        }

        for (activity_id, target) in &pending {
            if let Err(e) = self.deliver_one(activity_id, target).await {
                error!(activity = %activity_id, target = %target, error = %e, "delivery failed");
            }
        }

        for (activity_id, target) in &retry_ready {
            if let Err(e) = self.deliver_one(activity_id, target).await {
                error!(activity = %activity_id, target = %target, error = %e, "retry delivery failed");
            }
        }

        Ok(())
    }

    /// Deliver a single activity to a remote inbox.
    async fn deliver_one(
        &self,
        activity_id: &str,
        target_instance: &str,
    ) -> std::result::Result<(), DeliveryError> {
        let inbox_url = self.resolve_inbox_url(target_instance).await?;
        if inbox_url.is_empty() {
            return Err(DeliveryError::NoInboxUrl {
                target: target_instance.into(),
            });
        }

        let body;
        let digest_header;
        {
            let outbox = self.outbox.lock().await;
            body = outbox.get_activity_json(activity_id).unwrap_or_default();
            digest_header = compute_digest(body.as_bytes());
        }

        {
            let mut outbox = self.outbox.lock().await;
            outbox.mark_in_flight(activity_id, target_instance);
        }

        let signature = if self.config.private_key.is_empty() {
            None
        } else {
            let mut headers = HashMap::new();
            headers.insert("(method)".into(), "POST".into());
            headers.insert("(path)".into(), parse_path(&inbox_url));
            headers.insert("host".into(), parse_host(&inbox_url));
            headers.insert("date".into(), httpdate_format());
            headers.insert("digest".into(), digest_header.clone());

            let signing_config = self.config.signing_config();
            match self.verifier.sign_request(
                &signing_config,
                &headers,
                body.as_bytes(),
                &self.config.private_key,
                &self.config.key_id,
            ) {
                Ok(sig) => Some(sig),
                Err(e) => {
                    {
                        let mut outbox = self.outbox.lock().await;
                        outbox.mark_failed(activity_id, target_instance, false);
                    }
                    return Err(DeliveryError::SignatureError(e));
                }
            }
        };

        let mut req = self
            .http_client
            .post(&inbox_url)
            .header("Content-Type", "application/activity+json")
            .header("Digest", &digest_header)
            .header("Date", httpdate_format());

        if let Some(sig) = &signature {
            req = req.header("Signature", sig.to_header_value());
        }

        match req.body(body.clone()).send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                if (200..300).contains(&status) {
                    info!(
                        activity = %activity_id,
                        target = %target_instance,
                        status,
                        "activity delivered"
                    );
                    let mut outbox = self.outbox.lock().await;
                    outbox.mark_delivered(activity_id, target_instance);
                    Ok(())
                } else if (400..500).contains(&status) {
                    warn!(
                        activity = %activity_id,
                        target = %target_instance,
                        status,
                        "permanent rejection"
                    );
                    let mut outbox = self.outbox.lock().await;
                    outbox.mark_failed(activity_id, target_instance, true);
                    Ok(())
                } else {
                    warn!(
                        activity = %activity_id,
                        target = %target_instance,
                        status,
                        "transient failure"
                    );
                    let mut outbox = self.outbox.lock().await;
                    outbox.mark_failed(activity_id, target_instance, false);
                    Ok(())
                }
            }
            Err(e) => {
                warn!(
                    activity = %activity_id,
                    target = %target_instance,
                    error = %e,
                    "network error"
                );
                let mut outbox = self.outbox.lock().await;
                outbox.mark_failed(activity_id, target_instance, false);
                Ok(())
            }
        }
    }

    /// Resolve an inbox URL for a target instance.
    async fn resolve_inbox_url(&self, target_instance: &str) -> std::result::Result<String, DeliveryError> {
        let (username, domain) = parse_instance_target(target_instance);
        let cache_key = format!("acct:{username}@{domain}");

        {
            let cache = self.actor_cache.lock().await;
            if let Some(cached) = cache.get(&cache_key)
                && !cached.is_expired()
            {
                return Ok(cached.inbox_url.clone());
            }
        }

        let wf_result = resolve_webfinger(&domain, &username).await.map_err(|e| {
            DeliveryError::WebFingerFailed {
                target: target_instance.into(),
                detail: e.to_string(),
            }
        })?;

        let inbox_url = wf_result
            .links
            .iter()
            .find(|l| {
                l.rel == "http://www.w3.org/ns/ldp#inbox" || l.type_ == "application/activity+json"
            })
            .map(|l| l.href.clone())
            .unwrap_or_else(|| {
                wf_result
                    .links
                    .first()
                    .map(|l| format!("{}/inbox", l.href))
                    .unwrap_or_default()
            });

        {
            let mut cache = self.actor_cache.lock().await;
            cache.insert(
                cache_key,
                CachedActor {
                    inbox_url: inbox_url.clone(),
                    cached_at: std::time::Instant::now(),
                    ttl: self.actor_cache_ttl,
                },
            );
        }

        Ok(inbox_url)
    }

    /// Get delivery statistics.
    pub async fn stats(&self) -> DeliveryStats {
        let outbox = self.outbox.lock().await;
        DeliveryStats {
            pending: outbox.pending_count(),
            in_flight: outbox.in_flight_count(),
            delivered: outbox.delivered_count(),
            total: outbox.entry_count(),
        }
    }

    /// Purge expired entries from the actor cache.
    pub async fn purge_actor_cache(&self) -> usize {
        let mut cache = self.actor_cache.lock().await;
        let before = cache.len();
        cache.retain(|_, v| !v.is_expired());
        before - cache.len()
    }
}

/// Delivery statistics.
#[derive(Debug, Clone)]
pub struct DeliveryStats {
    pub pending: usize,
    pub in_flight: usize,
    pub delivered: usize,
    pub total: usize,
}

/// Parse a target instance string into (username, domain).
fn parse_instance_target(target: &str) -> (String, String) {
    if let Some((user, domain)) = target.split_once('@') {
        (user.to_string(), domain.to_string())
    } else {
        ("actor".to_string(), target.to_string())
    }
}

/// Parse the path from a URL.
fn parse_path(url: &str) -> String {
    url.find("//")
        .and_then(|scheme_end| {
            url[scheme_end + 2..]
                .find('/')
                .map(|path_start| url[scheme_end + 2 + path_start..].to_string())
        })
        .unwrap_or_else(|| "/".to_string())
}

/// Parse the host from a URL.
fn parse_host(url: &str) -> String {
    url.find("//")
        .and_then(|scheme_end| {
            let rest = &url[scheme_end + 2..];
            rest.find('/')
                .map(|path_start| rest[..path_start].to_string())
        })
        .unwrap_or_else(|| url.to_string())
}

/// Compute SHA-256 digest for HTTP content.
fn compute_digest(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("SHA-256={}", BASE64.encode(hasher.finalize()))
}

/// Format current time as HTTP-date (RFC 7231).
fn httpdate_format() -> String {
    chrono::Utc::now()
        .format("%a, %d %b %Y %H:%M:%S GMT")
        .to_string()
}

/// Compute an HTTP Signature (draft-cavage-http-signatures) for the given request components.
pub fn compute_signature(
    _method: &str,
    _path: &str,
    headers: &HashMap<String, String>,
    signing_key: &[u8],
) -> std::result::Result<String, DeliveryError> {
    let config = HttpSigningConfig {
        algorithm: SignatureAlgorithm::Ed25519,
        required_headers: vec![
            "(request-target)".into(),
            "host".into(),
            "date".into(),
            "digest".into(),
        ],
        expires_in_secs: 300,
    };

    let verifier = SignatureVerifier::new();
    let sig = verifier
        .sign_request(&config, headers, b"", signing_key, "main-key")
        .map_err(DeliveryError::SignatureError)?;
    Ok(sig.to_header_value())
}

/// Simple synchronous delivery interface for pushing a single Activity to a target inbox.
pub struct FederationDelivery;

impl FederationDelivery {
    pub async fn deliver_activity(
        activity: &crate::activitypub::Activity,
        target_url: &str,
        signing_key: &[u8],
    ) -> std::result::Result<(), DeliveryError> {
        let body = serde_json::to_vec(activity)
            .map_err(|e| DeliveryError::HttpClientError(format!("serialize activity: {e}")))?;

        let digest = compute_digest(&body);
        let date = httpdate_format();

        let parsed_url = reqwest::Url::parse(target_url)
            .map_err(|e| DeliveryError::HttpClientError(format!("invalid target URL: {e}")))?;
        let host = parsed_url.host_str().unwrap_or("").to_string();
        let path = parsed_url.path().to_string();

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("CivitForge/0.1 (ForgeFed)")
            .build()
            .map_err(|e| DeliveryError::HttpClientError(e.to_string()))?;

        let mut last_err = None;
        for attempt in 0..3u32 {
            if attempt > 0 {
                let backoff_ms = 1000u64 * 2u64.pow(attempt - 1);
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            }

            let mut sig_headers = HashMap::new();
            sig_headers.insert("(method)".into(), "POST".into());
            sig_headers.insert("(path)".into(), path.clone());
            sig_headers.insert("host".into(), host.clone());
            sig_headers.insert("date".into(), date.clone());
            sig_headers.insert("digest".into(), digest.clone());

            let signature_header = if signing_key.is_empty() {
                None
            } else {
                match compute_signature("POST", &path, &sig_headers, signing_key) {
                    Ok(sig) => Some(sig),
                    Err(e) => {
                        tracing::warn!(
                            attempt,
                            error = %e,
                            target = %target_url,
                            "failed to compute HTTP signature"
                        );
                        last_err = Some(e);
                        continue;
                    }
                }
            };

            let mut req = client
                .post(target_url)
                .header("Content-Type", "application/activity+json")
                .header("Date", &date)
                .header("Digest", &digest);

            if let Some(ref sig) = signature_header {
                req = req.header("Signature", sig);
            }

            match req.body(body.clone()).send().await {
                Ok(resp) if resp.status().is_success() => {
                    tracing::info!(
                        target = %target_url,
                        status = %resp.status(),
                        attempt,
                        "activity delivered"
                    );
                    return Ok(());
                }
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    if (400..500).contains(&status) {
                        tracing::warn!(
                            target = %target_url,
                            status,
                            attempt,
                            "activity delivery rejected (permanent)"
                        );
                        return Err(DeliveryError::HttpError {
                            status,
                            inbox: target_url.to_string(),
                        });
                    }
                    tracing::warn!(
                        target = %target_url,
                        status,
                        attempt,
                        "activity delivery returned transient failure"
                    );
                    last_err = Some(DeliveryError::HttpError {
                        status,
                        inbox: target_url.to_string(),
                    });
                }
                Err(e) => {
                    tracing::warn!(
                        target = %target_url,
                        error = %e,
                        attempt,
                        "activity delivery network error"
                    );
                    last_err = Some(DeliveryError::HttpClientError(e.to_string()));
                }
            }
        }

        Err(last_err.unwrap_or(DeliveryError::HttpClientError(
            "delivery failed after 3 attempts".into(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delivery_config_default() {
        let cfg = FederationDeliveryConfig::default();
        assert_eq!(cfg.key_id, "main-key");
        assert_eq!(cfg.max_concurrent, 10);
        assert_eq!(cfg.max_attempts, 5);
        assert_eq!(cfg.backoff_base_ms, 1000);
        assert_eq!(cfg.backoff_max_ms, 300_000);
        assert!(cfg.jitter_enabled);
        assert!(cfg.private_key.is_empty());
    }

    #[test]
    fn test_delivery_result_variants() {
        use DeliveryResult::*;
        let results = [
            Accepted,
            Rejected,
            TransientFailure,
            NetworkError,
            AlreadyDelivered,
        ];
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_parse_instance_target_user_domain() {
        let (user, domain) = parse_instance_target("alice@forge.example.com");
        assert_eq!(user, "alice");
        assert_eq!(domain, "forge.example.com");
    }

    #[test]
    fn test_parse_instance_target_bare_domain() {
        let (user, domain) = parse_instance_target("forge.example.com");
        assert_eq!(user, "actor");
        assert_eq!(domain, "forge.example.com");
    }

    #[test]
    fn test_parse_path() {
        assert_eq!(
            parse_path("https://example.com/users/alice/inbox"),
            "/users/alice/inbox"
        );
    }

    #[test]
    fn test_parse_path_root() {
        assert_eq!(parse_path("https://example.com"), "/");
    }

    #[test]
    fn test_parse_host() {
        assert_eq!(
            parse_host("https://example.com/users/alice/inbox"),
            "example.com"
        );
    }

    #[test]
    fn test_parse_host_with_port() {
        assert_eq!(
            parse_host("https://example.com:8080/users/alice"),
            "example.com:8080"
        );
    }

    #[test]
    fn test_compute_digest_deterministic() {
        let d1 = compute_digest(b"hello world");
        let d2 = compute_digest(b"hello world");
        assert_eq!(d1, d2);
        assert!(d1.starts_with("SHA-256="));
    }

    #[test]
    fn test_compute_digest_different_data() {
        let d1 = compute_digest(b"data-a");
        let d2 = compute_digest(b"data-b");
        assert_ne!(d1, d2);
    }

    #[test]
    fn test_delivery_error_display() {
        let err = DeliveryError::NoInboxUrl {
            target: "remote.example.com".into(),
        };
        assert!(err.to_string().contains("no inbox URL"));
        assert!(err.to_string().contains("remote.example.com"));
    }

    #[test]
    fn test_delivery_stats() {
        let stats = DeliveryStats {
            pending: 3,
            in_flight: 1,
            delivered: 42,
            total: 46,
        };
        assert_eq!(stats.total, 46);
        assert_eq!(stats.pending, 3);
    }

    #[test]
    fn test_cached_actor_expiry() {
        let cached = CachedActor {
            inbox_url: "https://example.com/inbox".into(),
            cached_at: std::time::Instant::now(),
            ttl: Duration::from_secs(1),
        };
        assert!(!cached.is_expired());

        let expired = CachedActor {
            inbox_url: "https://example.com/inbox".into(),
            cached_at: std::time::Instant::now() - Duration::from_secs(2),
            ttl: Duration::from_secs(1),
        };
        assert!(expired.is_expired());
    }

    #[test]
    fn test_compute_signature_returns_header_value() {
        let (private_key, _public_key) =
            crate::http_signatures::generate_ed25519_keypair();
        let mut headers = HashMap::new();
        headers.insert("(method)".into(), "POST".into());
        headers.insert("(path)".into(), "/users/alice/inbox".into());
        headers.insert("host".into(), "remote.example.com".into());
        headers.insert("date".into(), "Mon, 10 Jun 2026 12:00:00 GMT".into());
        headers.insert("digest".into(), "SHA-256=abc123".into());

        let result = compute_signature("POST", "/users/alice/inbox", &headers, &private_key);
        assert!(result.is_ok());
        let header_val = result.unwrap();
        assert!(header_val.contains("keyId=\"main-key\""));
        assert!(header_val.contains("algorithm=\"ed25519\""));
        assert!(header_val.contains("signature=\""));
    }

    #[test]
    fn test_compute_signature_empty_key_fails() {
        let mut headers = HashMap::new();
        headers.insert("(method)".into(), "POST".into());
        headers.insert("(path)".into(), "/inbox".into());

        let result = compute_signature("POST", "/inbox", &headers, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_compute_signature_invalid_key_fails() {
        let mut headers = HashMap::new();
        headers.insert("(method)".into(), "POST".into());
        headers.insert("(path)".into(), "/inbox".into());

        let result = compute_signature("POST", "/inbox", &headers, b"not-a-valid-key");
        assert!(result.is_err());
    }
}
