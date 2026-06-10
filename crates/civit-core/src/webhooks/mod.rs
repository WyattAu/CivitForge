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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Pending,
    Delivered,
    Failed,
    Retrying,
}

impl DeliveryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeliveryStatus::Pending => "pending",
            DeliveryStatus::Delivered => "delivered",
            DeliveryStatus::Failed => "failed",
            DeliveryStatus::Retrying => "retrying",
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
            let delivery_id = Uuid::new_v4().to_string();
            let delivery = WebhookDelivery {
                id: delivery_id.clone(),
                event: event.clone(),
                payload: payload.clone(),
                timestamp: Utc::now(),
            };

            if let Err(e) = self
                .persist_delivery(pool, &webhook.id.to_string(), &delivery)
                .await
            {
                tracing::error!("Failed to persist webhook delivery {delivery_id}: {e}");
                continue;
            }

            if let Err(e) = self.deliver_with_retry(pool, &webhook, &delivery).await {
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

    async fn persist_delivery(
        &self,
        pool: &sqlx::PgPool,
        webhook_id: &str,
        delivery: &WebhookDelivery,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO webhook_deliveries (id, webhook_id, event, payload, status, attempts) \
             VALUES ($1, $2, $3, $4, $5, 0)",
        )
        .bind(Uuid::parse_str(&delivery.id).unwrap_or_default())
        .bind(Uuid::parse_str(webhook_id).unwrap_or_default())
        .bind(delivery.event.as_str())
        .bind(&delivery.payload)
        .bind(DeliveryStatus::Pending.as_str())
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn update_delivery_status(
        &self,
        pool: &sqlx::PgPool,
        delivery_id: &str,
        status: DeliveryStatus,
        attempts: i32,
        last_error: Option<&str>,
        next_retry_at: Option<DateTime<Utc>>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE webhook_deliveries \
             SET status = $2, attempts = $3, last_error = $4, next_retry_at = $5 \
             WHERE id = $1",
        )
        .bind(Uuid::parse_str(delivery_id).unwrap_or_default())
        .bind(status.as_str())
        .bind(attempts)
        .bind(last_error)
        .bind(next_retry_at)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn deliver_with_retry(
        &self,
        pool: &sqlx::PgPool,
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
                    let _ = self
                        .update_delivery_status(
                            pool,
                            &delivery.id,
                            DeliveryStatus::Delivered,
                            (attempt + 1) as i32,
                            None,
                            None,
                        )
                        .await;
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

        let error_msg = last_err
            .as_ref()
            .map(|e| e.to_string())
            .unwrap_or_default();
        let next_retry = self.compute_next_retry(self.max_retries);
        let final_status = if next_retry.is_some() {
            DeliveryStatus::Retrying
        } else {
            DeliveryStatus::Failed
        };

        let _ = self
            .update_delivery_status(
                pool,
                &delivery.id,
                final_status,
                self.max_retries as i32,
                Some(&error_msg),
                next_retry,
            )
            .await;

        Err(last_err.unwrap_or(DeliveryError::MaxRetriesExceeded))
    }

    fn compute_next_retry(&self, attempts: u32) -> Option<DateTime<Utc>> {
        if attempts >= self.max_retries {
            None
        } else {
            let backoff_secs = 60 * 2u64.pow(attempts);
            Some(Utc::now() + chrono::Duration::from_std(Duration::from_secs(backoff_secs)).unwrap_or_default())
        }
    }

    pub async fn retry_pending_deliveries(&self, pool: &sqlx::PgPool) -> u32 {
        let pending = match sqlx::query_as::<_, PendingDelivery>(
            "SELECT id, webhook_id, event, payload, attempts \
             FROM webhook_deliveries \
             WHERE status IN ('pending', 'retrying') \
               AND (next_retry_at IS NULL OR next_retry_at <= NOW()) \
             ORDER BY created_at ASC \
             LIMIT 50",
        )
        .fetch_all(pool)
        .await
        {
            Ok(rows) => rows,
            Err(e) => {
                tracing::error!("Failed to query pending webhook deliveries: {e}");
                return 0;
            }
        };

        let mut retried = 0u32;
        for row in pending {
            if row.attempts >= self.max_retries as i32 {
                let _ = self
                    .update_delivery_status(
                        pool,
                        &row.id,
                        DeliveryStatus::Failed,
                        row.attempts,
                        Some("max retries exceeded"),
                        None,
                    )
                    .await;
                continue;
            }

            let webhook = match self.get_webhook(pool, &row.webhook_id).await {
                Some(w) => w,
                None => continue,
            };

            let delivery = WebhookDelivery {
                id: row.id.clone(),
                event: match row.event.as_str() {
                    "push" => WebhookEvent::Push,
                    "pull_request" => WebhookEvent::PullRequest,
                    "issue" => WebhookEvent::Issue,
                    "issue_comment" => WebhookEvent::IssueComment,
                    "pipeline" => WebhookEvent::Pipeline,
                    "release" => WebhookEvent::Release,
                    "star" => WebhookEvent::Star,
                    "fork" => WebhookEvent::Fork,
                    _ => WebhookEvent::Push,
                },
                payload: row.payload,
                timestamp: Utc::now(),
            };

            if self.deliver_with_retry(pool, &webhook, &delivery).await.is_ok() {
                retried += 1;
            }
        }
        retried
    }

    async fn get_webhook(&self, pool: &sqlx::PgPool, webhook_id: &str) -> Option<StoredWebhook> {
        sqlx::query_as::<_, StoredWebhook>(
            "SELECT id, url, events, secret, active FROM webhooks WHERE id = $1",
        )
        .bind(Uuid::parse_str(webhook_id).unwrap_or_default())
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
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

#[derive(Debug, sqlx::FromRow)]
struct PendingDelivery {
    id: String,
    webhook_id: String,
    event: String,
    payload: serde_json::Value,
    attempts: i32,
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

pub fn start_webhook_retry_loop(pool: sqlx::PgPool) {
    tokio::spawn(async move {
        let dispatcher = WebhookDispatcher::new();
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            let retried = dispatcher.retry_pending_deliveries(&pool).await;
            if retried > 0 {
                tracing::info!("Retried {retried} pending webhook deliveries");
            }
        }
    });
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

    #[test]
    fn test_delivery_status_as_str() {
        assert_eq!(DeliveryStatus::Pending.as_str(), "pending");
        assert_eq!(DeliveryStatus::Delivered.as_str(), "delivered");
        assert_eq!(DeliveryStatus::Failed.as_str(), "failed");
        assert_eq!(DeliveryStatus::Retrying.as_str(), "retrying");
    }

    #[test]
    fn test_delivery_status_serialization_roundtrip() {
        let statuses = vec![
            DeliveryStatus::Pending,
            DeliveryStatus::Delivered,
            DeliveryStatus::Failed,
            DeliveryStatus::Retrying,
        ];
        for status in &statuses {
            let json = serde_json::to_string(status).unwrap();
            let back: DeliveryStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(*status, back);
        }
    }

    #[test]
    fn test_compute_next_retry_none_when_max() {
        let dispatcher = WebhookDispatcher::new();
        assert!(dispatcher.compute_next_retry(3).is_none());
    }

    #[test]
    fn test_compute_next_retry_some_when_below_max() {
        let dispatcher = WebhookDispatcher::new();
        let next = dispatcher.compute_next_retry(0);
        assert!(next.is_some());
        let next = dispatcher.compute_next_retry(1);
        assert!(next.is_some());
        let next = dispatcher.compute_next_retry(2);
        assert!(next.is_some());
    }
}
