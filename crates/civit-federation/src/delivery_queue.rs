#![forbid(unsafe_code)]

use crate::delivery::{DeliveryError, DeliveryResult, FederationDeliveryConfig};
use crate::http_signatures::{HttpSigningConfig, SignatureAlgorithm, SignatureVerifier};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeliveryStatus {
    Pending,
    InFlight,
    Delivered,
    Failed,
    Retrying,
}

impl DeliveryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InFlight => "in_flight",
            Self::Delivered => "delivered",
            Self::Failed => "failed",
            Self::Retrying => "retrying",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryEntry {
    pub id: String,
    pub source_actor_url: String,
    pub target_inbox_url: String,
    pub activity_type: String,
    pub payload: serde_json::Value,
    pub status: DeliveryStatus,
    pub attempts: u32,
    pub max_attempts: u32,
    pub last_error: Option<String>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerState {
    pub instance_url: String,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub delivery_success_count: u64,
    pub delivery_failure_count: u64,
    pub avg_latency_ms: f64,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct QueueStatus {
    pub pending: usize,
    pub in_flight: usize,
    pub delivered: usize,
    pub failed: usize,
    pub total: usize,
}

pub struct DeliveryQueueService {
    config: FederationDeliveryConfig,
    http_client: reqwest::Client,
    queue: Arc<Mutex<Vec<DeliveryEntry>>>,
    peer_state: Arc<Mutex<HashMap<String, PeerState>>>,
    verifier: SignatureVerifier,
}

impl DeliveryQueueService {
    pub fn new(config: FederationDeliveryConfig) -> Result<Self, DeliveryError> {
        let http_client = reqwest::Client::builder()
            .timeout(config.http_timeout)
            .user_agent("CivitForge/0.1 (ForgeFed)")
            .build()
            .map_err(|e| DeliveryError::HttpClientError(e.to_string()))?;

        Ok(Self {
            config,
            http_client,
            queue: Arc::new(Mutex::new(Vec::new())),
            peer_state: Arc::new(Mutex::new(HashMap::new())),
            verifier: SignatureVerifier::new(),
        })
    }

    pub async fn enqueue_delivery(
        &self,
        source_actor_url: &str,
        target_inbox_url: &str,
        activity_type: &str,
        payload: serde_json::Value,
    ) -> String {
        let entry = DeliveryEntry {
            id: uuid(),
            source_actor_url: source_actor_url.to_string(),
            target_inbox_url: target_inbox_url.to_string(),
            activity_type: activity_type.to_string(),
            payload,
            status: DeliveryStatus::Pending,
            attempts: 0,
            max_attempts: self.config.max_attempts,
            last_error: None,
            next_retry_at: None,
            created_at: Utc::now(),
            delivered_at: None,
        };
        let id = entry.id.clone();
        self.queue.lock().await.push(entry);
        info!(delivery_id = %id, "enqueued federation delivery");
        id
    }

    pub async fn process_queue(&self) -> Vec<DeliveryResult> {
        let entries: Vec<DeliveryEntry> = {
            let queue = self.queue.lock().await;
            let now = Utc::now();
            let ready: Vec<usize> = queue
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    e.status == DeliveryStatus::Pending
                        || e.status == DeliveryStatus::Retrying
                            && e.next_retry_at.map_or(true, |t| t <= now)
                })
                .map(|(i, _)| i)
                .take(self.config.batch_size)
                .collect();
            ready
                .iter()
                .rev()
                .filter_map(|&i| queue.get(i).cloned())
                .collect()
        };

        let mut results = Vec::new();
        for entry in &entries {
            let result = self.deliver_entry(entry).await;
            results.push(result.clone());
            self.update_entry_status(entry.id.clone(), result).await;
        }
        results
    }

    pub async fn retry_failed(&self, entry_id: &str) -> Result<DeliveryEntry, DeliveryError> {
        let mut queue = self.queue.lock().await;
        let entry = queue
            .iter_mut()
            .find(|e| e.id == entry_id)
            .ok_or_else(|| DeliveryError::HttpClientError(format!("entry {entry_id} not found")))?;

        if entry.status != DeliveryStatus::Failed {
            return Err(DeliveryError::HttpClientError(
                "entry is not in failed state".into(),
            ));
        }
        entry.status = DeliveryStatus::Pending;
        entry.next_retry_at = None;
        entry.last_error = None;
        info!(delivery_id = %entry_id, "retrying failed delivery");
        Ok(entry.clone())
    }

    pub async fn get_queue_status(&self) -> QueueStatus {
        let queue = self.queue.lock().await;
        let mut pending = 0;
        let mut in_flight = 0;
        let mut delivered = 0;
        let mut failed = 0;
        for entry in queue.iter() {
            match entry.status {
                DeliveryStatus::Pending | DeliveryStatus::Retrying => pending += 1,
                DeliveryStatus::InFlight => in_flight += 1,
                DeliveryStatus::Delivered => delivered += 1,
                DeliveryStatus::Failed => failed += 1,
            }
        }
        QueueStatus {
            pending,
            in_flight,
            delivered,
            failed,
            total: queue.len(),
        }
    }

    pub async fn update_peer_state(
        &self,
        instance_url: &str,
        success: bool,
        latency_ms: f64,
    ) {
        let mut peers = self.peer_state.lock().await;
        let peer = peers
            .entry(instance_url.to_string())
            .or_insert_with(|| PeerState {
                instance_url: instance_url.to_string(),
                last_seen_at: None,
                delivery_success_count: 0,
                delivery_failure_count: 0,
                avg_latency_ms: 0.0,
                status: "active".into(),
                created_at: Utc::now(),
            });
        peer.last_seen_at = Some(Utc::now());
        if success {
            let total = peer.delivery_success_count + peer.delivery_failure_count;
            let new_total = total + 1;
            peer.avg_latency_ms =
                (peer.avg_latency_ms * total as f64 + latency_ms) / new_total as f64;
            peer.delivery_success_count += 1;
        } else {
            peer.delivery_failure_count += 1;
        }
    }

    async fn deliver_entry(&self, entry: &DeliveryEntry) -> DeliveryResult {
        let body = match serde_json::to_string(&entry.payload) {
            Ok(b) => b,
            Err(_) => return DeliveryResult::NetworkError,
        };

        let digest = compute_digest(body.as_bytes());
        let date = httpdate_format();

        let parsed_url = match reqwest::Url::parse(&entry.target_inbox_url) {
            Ok(u) => u,
            Err(_) => return DeliveryResult::Rejected,
        };
        let host = parsed_url.host_str().unwrap_or("").to_string();
        let path = parsed_url.path().to_string();

        let mut sig_headers = HashMap::new();
        sig_headers.insert("(method)".into(), "POST".into());
        sig_headers.insert("(path)".into(), path.clone());
        sig_headers.insert("host".into(), host.clone());
        sig_headers.insert("date".into(), date.clone());
        sig_headers.insert("digest".into(), digest.clone());

        let signing_config = HttpSigningConfig {
            algorithm: SignatureAlgorithm::Ed25519,
            required_headers: vec![
                "(request-target)".into(),
                "host".into(),
                "date".into(),
                "digest".into(),
            ],
            expires_in_secs: 300,
        };

        let signature_header = if self.config.private_key.is_empty() {
            None
        } else {
            match self.verifier.sign_request(
                &signing_config,
                &sig_headers,
                body.as_bytes(),
                &self.config.private_key,
                &self.config.key_id,
            ) {
                Ok(sig) => Some(sig.to_header_value()),
                Err(_) => {
                    return DeliveryResult::NetworkError;
                }
            }
        };

        let mut req = self
            .http_client
            .post(&entry.target_inbox_url)
            .header("Content-Type", "application/activity+json")
            .header("Digest", &digest)
            .header("Date", &date);

        if let Some(ref sig) = signature_header {
            req = req.header("Signature", sig);
        }

        match req.body(body).send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                if (200..300).contains(&status) {
                    DeliveryResult::Accepted
                } else if (400..500).contains(&status) {
                    DeliveryResult::Rejected
                } else {
                    DeliveryResult::TransientFailure
                }
            }
            Err(_) => DeliveryResult::NetworkError,
        }
    }

    async fn update_entry_status(&self, entry_id: String, result: DeliveryResult) {
        let mut queue = self.queue.lock().await;
        if let Some(entry) = queue.iter_mut().find(|e| e.id == entry_id) {
            entry.attempts += 1;
            match result {
                DeliveryResult::Accepted => {
                    entry.status = DeliveryStatus::Delivered;
                    entry.delivered_at = Some(Utc::now());
                    info!(delivery_id = %entry_id, "delivery accepted");
                }
                DeliveryResult::Rejected => {
                    entry.status = DeliveryStatus::Failed;
                    entry.last_error = Some("rejected by remote".into());
                    warn!(delivery_id = %entry_id, "delivery permanently rejected");
                }
                DeliveryResult::TransientFailure | DeliveryResult::NetworkError => {
                    if entry.attempts >= entry.max_attempts {
                        entry.status = DeliveryStatus::Failed;
                        entry.last_error = Some(format!("failed after {} attempts", entry.attempts));
                        warn!(delivery_id = %entry_id, attempts = entry.attempts, "delivery permanently failed");
                    } else {
                        entry.status = DeliveryStatus::Retrying;
                        let backoff_ms =
                            1000u64 * 2u64.pow(entry.attempts.saturating_sub(1));
                        entry.next_retry_at =
                            Some(Utc::now() + Duration::from_millis(backoff_ms));
                        entry.last_error = Some("transient failure, will retry".into());
                        info!(
                            delivery_id = %entry_id,
                            next_retry = ?entry.next_retry_at,
                            "delivery will retry"
                        );
                    }
                }
                DeliveryResult::AlreadyDelivered => {
                    entry.status = DeliveryStatus::Delivered;
                    entry.delivered_at = Some(Utc::now());
                }
            }
        }
    }
}

fn uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:032x}", t as u128)
}

fn compute_digest(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    use base64::Engine;
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!(
        "SHA-256={}",
        base64::prelude::BASE64_STANDARD.encode(hasher.finalize())
    )
}

fn httpdate_format() -> String {
    chrono::Utc::now()
        .format("%a, %d %b %Y %H:%M:%S GMT")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delivery_status_as_str() {
        assert_eq!(DeliveryStatus::Pending.as_str(), "pending");
        assert_eq!(DeliveryStatus::InFlight.as_str(), "in_flight");
        assert_eq!(DeliveryStatus::Delivered.as_str(), "delivered");
        assert_eq!(DeliveryStatus::Failed.as_str(), "failed");
        assert_eq!(DeliveryStatus::Retrying.as_str(), "retrying");
    }

    #[test]
    fn test_delivery_entry_serialization() {
        let entry = DeliveryEntry {
            id: "test-123".into(),
            source_actor_url: "https://example.com/actor".into(),
            target_inbox_url: "https://remote.com/inbox".into(),
            activity_type: "Create".into(),
            payload: serde_json::json!({"type": "Note"}),
            status: DeliveryStatus::Pending,
            attempts: 0,
            max_attempts: 5,
            last_error: None,
            next_retry_at: None,
            created_at: Utc::now(),
            delivered_at: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: DeliveryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "test-123");
        assert_eq!(deserialized.status, DeliveryStatus::Pending);
    }

    #[test]
    fn test_peer_state_defaults() {
        let peer = PeerState {
            instance_url: "https://forge.example.com".into(),
            last_seen_at: None,
            delivery_success_count: 0,
            delivery_failure_count: 0,
            avg_latency_ms: 0.0,
            status: "active".into(),
            created_at: Utc::now(),
        };
        assert_eq!(peer.delivery_success_count, 0);
        assert_eq!(peer.status, "active");
    }

    #[test]
    fn test_queue_status_calculation() {
        let status = QueueStatus {
            pending: 5,
            in_flight: 2,
            delivered: 10,
            failed: 1,
            total: 18,
        };
        assert_eq!(status.total, status.pending + status.in_flight + status.delivered + status.failed);
    }

    #[tokio::test]
    async fn test_enqueue_and_status() {
        let config = FederationDeliveryConfig::default();
        let service = DeliveryQueueService::new(config).unwrap();
        let id = service
            .enqueue_delivery(
                "https://example.com/actor",
                "https://remote.com/inbox",
                "Create",
                serde_json::json!({"type": "Note"}),
            )
            .await;
        assert!(!id.is_empty());
        let status = service.get_queue_status().await;
        assert_eq!(status.pending, 1);
        assert_eq!(status.total, 1);
    }

    #[tokio::test]
    async fn test_process_empty_queue() {
        let config = FederationDeliveryConfig::default();
        let service = DeliveryQueueService::new(config).unwrap();
        let results = service.process_queue().await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_retry_failed_nonexistent() {
        let config = FederationDeliveryConfig::default();
        let service = DeliveryQueueService::new(config).unwrap();
        let result = service.retry_failed("nonexistent-id").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_peer_state_success() {
        let config = FederationDeliveryConfig::default();
        let service = DeliveryQueueService::new(config).unwrap();
        service
            .update_peer_state("https://forge.example.com", true, 42.5)
            .await;
        let peers = service.peer_state.lock().await;
        let peer = peers.get("https://forge.example.com").unwrap();
        assert_eq!(peer.delivery_success_count, 1);
        assert_eq!(peer.delivery_failure_count, 0);
        assert!((peer.avg_latency_ms - 42.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_update_peer_state_failure() {
        let config = FederationDeliveryConfig::default();
        let service = DeliveryQueueService::new(config).unwrap();
        service
            .update_peer_state("https://forge.example.com", false, 100.0)
            .await;
        let peers = service.peer_state.lock().await;
        let peer = peers.get("https://forge.example.com").unwrap();
        assert_eq!(peer.delivery_success_count, 0);
        assert_eq!(peer.delivery_failure_count, 1);
    }

    #[tokio::test]
    async fn test_update_peer_state_averages() {
        let config = FederationDeliveryConfig::default();
        let service = DeliveryQueueService::new(config).unwrap();
        service
            .update_peer_state("https://forge.example.com", true, 100.0)
            .await;
        service
            .update_peer_state("https://forge.example.com", true, 200.0)
            .await;
        let peers = service.peer_state.lock().await;
        let peer = peers.get("https://forge.example.com").unwrap();
        assert_eq!(peer.delivery_success_count, 2);
        assert!((peer.avg_latency_ms - 150.0).abs() < 0.01);
    }

    #[test]
    fn test_compute_digest() {
        let d1 = compute_digest(b"hello");
        let d2 = compute_digest(b"hello");
        assert_eq!(d1, d2);
        assert!(d1.starts_with("SHA-256="));
    }

    #[test]
    fn test_httpdate_format() {
        let formatted = httpdate_format();
        assert!(formatted.contains("GMT"));
    }
}
