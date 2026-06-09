#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::Duration;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WebhookEvent {
    Push,
    PullRequest,
    Issue,
    IssueComment,
    Pipeline,
    Release,
    Star,
    Fork,
}

impl WebhookEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            WebhookEvent::Push => "push",
            WebhookEvent::PullRequest => "pull_request",
            WebhookEvent::Issue => "issue",
            WebhookEvent::IssueComment => "issue_comment",
            WebhookEvent::Pipeline => "pipeline",
            WebhookEvent::Release => "release",
            WebhookEvent::Star => "star",
            WebhookEvent::Fork => "fork",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDelivery {
    pub id: String,
    pub event: WebhookEvent,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

pub struct WebhookDispatcher {
    http_client: Client,
    max_retries: u32,
}

impl Default for WebhookDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl WebhookDispatcher {
    pub fn new() -> Self {
        Self {
            http_client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            max_retries: 3,
        }
    }

    pub async fn dispatch(
        &self,
        pool: &sqlx::PgPool,
        repo_id: Uuid,
        event: &WebhookEvent,
        payload: serde_json::Value,
    ) {
        let webhooks = match self.query_active_webhooks(pool, repo_id, event).await {
            Ok(w) => w,
            Err(e) => {
                tracing::error!("Failed to query webhooks for repo {repo_id}: {e}");
                return;
            }
        };

        for webhook in webhooks {
            let delivery = WebhookDelivery {
                id: Uuid::new_v4().to_string(),
                event: event.clone(),
                payload: payload.clone(),
                timestamp: Utc::now(),
            };

            if let Err(e) = self.deliver_with_retry(&webhook, &delivery).await {
                tracing::error!(
                    "Webhook delivery failed for {} after {} attempts: {e}",
                    webhook.url,
                    self.max_retries
                );
            }
        }
    }

    async fn query_active_webhooks(
        &self,
        pool: &sqlx::PgPool,
        repo_id: Uuid,
        event: &WebhookEvent,
    ) -> Result<Vec<StoredWebhook>, sqlx::Error> {
        let rows = sqlx::query_as::<_, StoredWebhook>(
            "SELECT id, url, events, secret, active FROM webhooks WHERE repo_id = $1 AND active = true",
        )
        .bind(repo_id)
        .fetch_all(pool)
        .await?;

        let event_name = event.as_str();
        Ok(rows
            .into_iter()
            .filter(|wh| {
                let events: Vec<String> =
                    serde_json::from_value(wh.events.clone()).unwrap_or_default();
                events.iter().any(|e| e == event_name)
            })
            .collect())
    }

    async fn deliver_with_retry(
        &self,
        webhook: &StoredWebhook,
        delivery: &WebhookDelivery,
    ) -> Result<(), DeliveryError> {
        let body = serde_json::to_vec(&delivery.payload)
            .map_err(|e| DeliveryError::Serialization(e.to_string()))?;

        let mut last_err = None;
        for attempt in 0..self.max_retries {
            if attempt > 0 {
                let backoff_ms = 1000 * 2u64.pow(attempt - 1);
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            }

            let mut req = self
                .http_client
                .post(&webhook.url)
                .header("Content-Type", "application/json")
                .header("X-CivitForge-Event", delivery.event.as_str())
                .header("X-CivitForge-Delivery", &delivery.id)
                .body(body.clone());

            if let Some(ref secret) = webhook.secret {
                let signature = compute_hmac_signature(secret, &body);
                req = req.header("X-CivitForge-Signature", signature);
            }

            match req.send().await {
                Ok(resp) if resp.status().is_success() => {
                    tracing::info!(
                        "Webhook delivered: event={}, delivery_id={}, url={}, status={}",
                        delivery.event.as_str(),
                        delivery.id,
                        webhook.url,
                        resp.status()
                    );
                    return Ok(());
                }
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    tracing::warn!(
                        "Webhook delivery returned {status} on attempt {attempt}: url={}",
                        webhook.url
                    );
                    last_err = Some(DeliveryError::Http(status));
                }
                Err(e) => {
                    tracing::warn!(
                        "Webhook delivery error on attempt {attempt}: url={}, error={e}",
                        webhook.url
                    );
                    last_err = Some(DeliveryError::Network(e.to_string()));
                }
            }
        }

        Err(last_err.unwrap_or(DeliveryError::MaxRetriesExceeded))
    }
}

#[derive(Debug, sqlx::FromRow)]
struct StoredWebhook {
    #[allow(dead_code)]
    id: Uuid,
    url: String,
    events: serde_json::Value,
    secret: Option<String>,
    #[allow(dead_code)]
    active: bool,
}

#[derive(Debug)]
enum DeliveryError {
    Serialization(String),
    Http(u16),
    Network(String),
    MaxRetriesExceeded,
}

impl std::fmt::Display for DeliveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeliveryError::Serialization(e) => write!(f, "serialization: {e}"),
            DeliveryError::Http(code) => write!(f, "HTTP {code}"),
            DeliveryError::Network(e) => write!(f, "network: {e}"),
            DeliveryError::MaxRetriesExceeded => write!(f, "max retries exceeded"),
        }
    }
}

pub fn compute_hmac_signature(secret: &str, body: &[u8]) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
    mac.update(body);
    let result = mac.finalize();
    let bytes = result.into_bytes();
    format!("sha256={}", hex::encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_signature_format() {
        let sig = compute_hmac_signature("secret", b"payload");
        assert!(sig.starts_with("sha256="));
        let hex_part = &sig[7..];
        assert_eq!(hex_part.len(), 64);
        assert!(hex::decode(hex_part).is_ok());
    }

    #[test]
    fn test_hmac_signature_deterministic() {
        let sig1 = compute_hmac_signature("key", b"data");
        let sig2 = compute_hmac_signature("key", b"data");
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_hmac_signature_different_secrets() {
        let sig1 = compute_hmac_signature("secret1", b"data");
        let sig2 = compute_hmac_signature("secret2", b"data");
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_hmac_signature_different_payloads() {
        let sig1 = compute_hmac_signature("key", b"data1");
        let sig2 = compute_hmac_signature("key", b"data2");
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_webhook_event_as_str() {
        assert_eq!(WebhookEvent::Push.as_str(), "push");
        assert_eq!(WebhookEvent::PullRequest.as_str(), "pull_request");
        assert_eq!(WebhookEvent::Issue.as_str(), "issue");
        assert_eq!(WebhookEvent::IssueComment.as_str(), "issue_comment");
        assert_eq!(WebhookEvent::Pipeline.as_str(), "pipeline");
        assert_eq!(WebhookEvent::Release.as_str(), "release");
        assert_eq!(WebhookEvent::Star.as_str(), "star");
        assert_eq!(WebhookEvent::Fork.as_str(), "fork");
    }

    #[test]
    fn test_webhook_event_serialization_roundtrip() {
        let events = vec![
            WebhookEvent::Push,
            WebhookEvent::PullRequest,
            WebhookEvent::Issue,
            WebhookEvent::IssueComment,
            WebhookEvent::Pipeline,
            WebhookEvent::Release,
            WebhookEvent::Star,
            WebhookEvent::Fork,
        ];
        for event in &events {
            let json = serde_json::to_string(event).unwrap();
            let back: WebhookEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(*event, back);
        }
    }

    #[test]
    fn test_webhook_delivery_serialization() {
        let delivery = WebhookDelivery {
            id: Uuid::new_v4().to_string(),
            event: WebhookEvent::Push,
            payload: serde_json::json!({"ref": "main", "commits": []}),
            timestamp: Utc::now(),
        };
        let json = serde_json::to_string(&delivery).unwrap();
        assert!(json.contains("Push"));
        assert!(json.contains("main"));
    }

    #[test]
    fn test_webhook_dispatcher_new() {
        let dispatcher = WebhookDispatcher::new();
        assert_eq!(dispatcher.max_retries, 3);
    }

    #[test]
    fn test_delivery_error_display() {
        let err = DeliveryError::Http(500);
        assert_eq!(err.to_string(), "HTTP 500");

        let err = DeliveryError::Network("timeout".into());
        assert!(err.to_string().contains("timeout"));

        let err = DeliveryError::MaxRetriesExceeded;
        assert!(err.to_string().contains("max retries"));

        let err = DeliveryError::Serialization("bad json".into());
        assert!(err.to_string().contains("serialization"));
    }
}
