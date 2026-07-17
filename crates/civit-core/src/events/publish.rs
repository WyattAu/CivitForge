#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PublishedEvent {
    pub id: Uuid,
    pub event_type: String,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub actor_id: Option<Uuid>,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventSubscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub event_type: String,
    pub callback_url: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventDelivery {
    pub id: Uuid,
    pub event_id: Uuid,
    pub subscription_id: Uuid,
    pub status: String,
    pub attempts: i32,
    pub last_error: Option<String>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub struct EventPublisher {
    http_client: Client,
}

impl Default for EventPublisher {
    fn default() -> Self {
        Self::new()
    }
}

impl EventPublisher {
    pub fn new() -> Self {
        Self {
            http_client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    pub async fn publish_event(
        &self,
        pool: &sqlx::PgPool,
        event_type: &str,
        resource_type: &str,
        resource_id: &str,
        actor_id: Option<&str>,
        payload: serde_json::Value,
    ) -> Result<PublishedEvent, sqlx::Error> {
        let event_id = Uuid::new_v4();
        let resource_uuid = Uuid::parse_str(resource_id).unwrap_or_default();
        let actor_uuid = actor_id.and_then(|id| Uuid::parse_str(id).ok());

        sqlx::query(
            "INSERT INTO events (id, event_type, resource_type, resource_id, actor_id, payload) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(event_id)
        .bind(event_type)
        .bind(resource_type)
        .bind(resource_uuid)
        .bind(actor_uuid)
        .bind(&payload)
        .execute(pool)
        .await?;

        let event = PublishedEvent {
            id: event_id,
            event_type: event_type.to_string(),
            resource_type: resource_type.to_string(),
            resource_id: resource_uuid,
            actor_id: actor_uuid,
            payload,
            created_at: Utc::now(),
        };

        self.deliver_to_subscribers(pool, &event).await;

        Ok(event)
    }

    async fn deliver_to_subscribers(&self, pool: &sqlx::PgPool, event: &PublishedEvent) {
        let subscriptions = match sqlx::query_as::<_, EventSubscription>(
            "SELECT id, user_id, event_type, callback_url, enabled, created_at \
             FROM event_subscriptions \
             WHERE event_type = $1 AND enabled = true",
        )
        .bind(&event.event_type)
        .fetch_all(pool)
        .await
        {
            Ok(subs) => subs,
            Err(e) => {
                tracing::error!("Failed to query event subscriptions: {e}");
                return;
            }
        };

        for subscription in subscriptions {
            if let Some(ref callback_url) = subscription.callback_url {
                let delivery_id = Uuid::new_v4();
                let _payload = serde_json::json!({
                    "event": event,
                    "subscription": subscription,
                });

                let _ = sqlx::query(
                    "INSERT INTO event_deliveries (id, event_id, subscription_id, status, attempts) \
                     VALUES ($1, $2, $3, 'pending', 0)",
                )
                .bind(delivery_id)
                .bind(event.id)
                .bind(subscription.id)
                .execute(pool)
                .await;

                let client = self.http_client.clone();
                let callback_url = callback_url.clone();
                let event_clone = event.clone();

                tokio::spawn(async move {
                    if let Err(e) = Self::deliver_event(&client, &callback_url, &event_clone).await {
                        tracing::error!("Failed to deliver event to {}: {e}", callback_url);
                    }
                });
            }
        }
    }

    async fn deliver_event(
        client: &Client,
        callback_url: &str,
        event: &PublishedEvent,
    ) -> Result<(), DeliveryError> {
        let body = serde_json::to_vec(event)
            .map_err(|e| DeliveryError::Serialization(e.to_string()))?;

        let resp = client
            .post(callback_url)
            .header("Content-Type", "application/json")
            .header("X-CivitForge-Event", &event.event_type)
            .header("X-CivitForge-Delivery", event.id.to_string())
            .body(body)
            .send()
            .await
            .map_err(|e| DeliveryError::Network(e.to_string()))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(DeliveryError::Http(resp.status().as_u16()))
        }
    }

    pub async fn create_subscription(
        &self,
        pool: &sqlx::PgPool,
        user_id: &str,
        event_type: &str,
        callback_url: Option<&str>,
    ) -> Result<EventSubscription, sqlx::Error> {
        let id = Uuid::new_v4();
        let user_uuid = Uuid::parse_str(user_id).unwrap_or_default();

        sqlx::query_as::<_, EventSubscription>(
            "INSERT INTO event_subscriptions (id, user_id, event_type, callback_url) \
             VALUES ($1, $2, $3, $4) \
             RETURNING id, user_id, event_type, callback_url, enabled, created_at",
        )
        .bind(id)
        .bind(user_uuid)
        .bind(event_type)
        .bind(callback_url)
        .fetch_one(pool)
        .await
    }

    pub async fn list_subscriptions(
        &self,
        pool: &sqlx::PgPool,
        user_id: &str,
    ) -> Result<Vec<EventSubscription>, sqlx::Error> {
        let user_uuid = Uuid::parse_str(user_id).unwrap_or_default();

        sqlx::query_as::<_, EventSubscription>(
            "SELECT id, user_id, event_type, callback_url, enabled, created_at \
             FROM event_subscriptions \
             WHERE user_id = $1 \
             ORDER BY created_at DESC",
        )
        .bind(user_uuid)
        .fetch_all(pool)
        .await
    }

    pub async fn delete_subscription(
        &self,
        pool: &sqlx::PgPool,
        subscription_id: &str,
        user_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let sub_uuid = Uuid::parse_str(subscription_id).unwrap_or_default();
        let user_uuid = Uuid::parse_str(user_id).unwrap_or_default();

        let result = sqlx::query(
            "DELETE FROM event_subscriptions WHERE id = $1 AND user_id = $2",
        )
        .bind(sub_uuid)
        .bind(user_uuid)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_event_history(
        &self,
        pool: &sqlx::PgPool,
        resource_type: &str,
        resource_id: &str,
        limit: i64,
    ) -> Result<Vec<PublishedEvent>, sqlx::Error> {
        let resource_uuid = Uuid::parse_str(resource_id).unwrap_or_default();

        sqlx::query_as::<_, PublishedEvent>(
            "SELECT id, event_type, resource_type, resource_id, actor_id, payload, created_at \
             FROM events \
             WHERE resource_type = $1 AND resource_id = $2 \
             ORDER BY created_at DESC \
             LIMIT $3",
        )
        .bind(resource_type)
        .bind(resource_uuid)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    pub async fn replay_event(
        &self,
        pool: &sqlx::PgPool,
        event_id: &str,
    ) -> Result<PublishedEvent, sqlx::Error> {
        let event_uuid = Uuid::parse_str(event_id).unwrap_or_default();

        let event = sqlx::query_as::<_, PublishedEvent>(
            "SELECT id, event_type, resource_type, resource_id, actor_id, payload, created_at \
             FROM events \
             WHERE id = $1",
        )
        .bind(event_uuid)
        .fetch_one(pool)
        .await?;

        self.deliver_to_subscribers(pool, &event).await;

        Ok(event)
    }
}

#[derive(Debug)]
enum DeliveryError {
    Serialization(String),
    Http(u16),
    Network(String),
}

impl std::fmt::Display for DeliveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeliveryError::Serialization(e) => write!(f, "serialization: {e}"),
            DeliveryError::Http(code) => write!(f, "HTTP {code}"),
            DeliveryError::Network(e) => write!(f, "network: {e}"),
        }
    }
}

impl std::error::Error for DeliveryError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_published_event_creation() {
        let event = PublishedEvent {
            id: Uuid::new_v4(),
            event_type: "push".to_string(),
            resource_type: "repository".to_string(),
            resource_id: Uuid::new_v4(),
            actor_id: Some(Uuid::new_v4()),
            payload: serde_json::json!({"ref": "main"}),
            created_at: Utc::now(),
        };

        assert_eq!(event.event_type, "push");
        assert_eq!(event.resource_type, "repository");
        assert!(event.actor_id.is_some());
    }

    #[test]
    fn test_event_subscription_creation() {
        let subscription = EventSubscription {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            event_type: "push".to_string(),
            callback_url: Some("https://example.com/webhook".to_string()),
            enabled: true,
            created_at: Utc::now(),
        };

        assert_eq!(subscription.event_type, "push");
        assert!(subscription.enabled);
        assert!(subscription.callback_url.is_some());
    }

    #[test]
    fn test_event_publisher_new() {
        let _publisher = EventPublisher::new();
    }

    #[test]
    fn test_delivery_error_display() {
        let err = DeliveryError::Http(500);
        assert_eq!(err.to_string(), "HTTP 500");

        let err = DeliveryError::Network("timeout".into());
        assert!(err.to_string().contains("timeout"));

        let err = DeliveryError::Serialization("bad json".into());
        assert!(err.to_string().contains("serialization"));
    }

    #[test]
    fn test_event_serialization_roundtrip() {
        let event = PublishedEvent {
            id: Uuid::new_v4(),
            event_type: "push".to_string(),
            resource_type: "repository".to_string(),
            resource_id: Uuid::new_v4(),
            actor_id: None,
            payload: serde_json::json!({"data": "test"}),
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: PublishedEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event.id, deserialized.id);
        assert_eq!(event.event_type, deserialized.event_type);
    }

    #[test]
    fn test_event_subscription_serialization_roundtrip() {
        let subscription = EventSubscription {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            event_type: "issue".to_string(),
            callback_url: None,
            enabled: false,
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&subscription).unwrap();
        let deserialized: EventSubscription = serde_json::from_str(&json).unwrap();
        assert_eq!(subscription.id, deserialized.id);
        assert!(!deserialized.enabled);
    }
}