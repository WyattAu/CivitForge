#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GraphqlSubscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub query: String,
    pub variables: serde_json::Value,
    pub channel: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionEvent {
    pub subscription_id: Uuid,
    pub channel: String,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub query: String,
    #[serde(default)]
    pub variables: Option<serde_json::Value>,
    pub channel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionFilter {
    pub channel: Option<String>,
    pub enabled: Option<bool>,
}

/// Per-subscriber queue depth for subscription fan-out.
///
/// This queue was unbounded, so one subscriber that stopped reading grew the
/// server's heap until the host ran out of memory.
pub const GRAPHQL_SUBSCRIPTION_CAPACITY: usize = 256;

struct SubscriptionEntry {
    subscription: GraphqlSubscription,
    tx: tokio::sync::mpsc::Sender<SubscriptionEvent>,
}

pub struct GraphqlSubscriptionService {
    subscriptions: Arc<RwLock<HashMap<Uuid, SubscriptionEntry>>>,
    dropped: Arc<std::sync::atomic::AtomicU64>,
    channel_subscribers: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
}

impl Default for GraphqlSubscriptionService {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphqlSubscriptionService {
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            dropped: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            channel_subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_subscription(
        &self,
        pool: &sqlx::PgPool,
        user_id: Uuid,
        request: CreateSubscriptionRequest,
    ) -> Result<
        (
            GraphqlSubscription,
            tokio::sync::mpsc::Receiver<SubscriptionEvent>,
        ),
        sqlx::Error,
    > {
        let id = Uuid::new_v4();
        let variables = request.variables.unwrap_or(serde_json::json!({}));

        let subscription = sqlx::query_as::<_, GraphqlSubscription>(
            "INSERT INTO graphql_subscriptions (id, user_id, query, variables, channel, enabled) \
             VALUES ($1, $2, $3, $4, $5, true) \
             RETURNING id, user_id, query, variables, channel, enabled, created_at",
        )
        .bind(id)
        .bind(user_id)
        .bind(&request.query)
        .bind(&variables)
        .bind(&request.channel)
        .fetch_one(pool)
        .await?;

        let (tx, rx) = tokio::sync::mpsc::channel(GRAPHQL_SUBSCRIPTION_CAPACITY);

        let entry = SubscriptionEntry {
            subscription: subscription.clone(),
            tx,
        };

        self.subscriptions.write().await.insert(id, entry);

        self.channel_subscribers
            .write()
            .await
            .entry(request.channel)
            .or_default()
            .push(id);

        Ok((subscription, rx))
    }

    pub async fn get_subscription(
        &self,
        pool: &sqlx::PgPool,
        subscription_id: Uuid,
    ) -> Result<Option<GraphqlSubscription>, sqlx::Error> {
        sqlx::query_as::<_, GraphqlSubscription>(
            "SELECT id, user_id, query, variables, channel, enabled, created_at \
             FROM graphql_subscriptions \
             WHERE id = $1",
        )
        .bind(subscription_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn list_user_subscriptions(
        &self,
        pool: &sqlx::PgPool,
        user_id: Uuid,
        filter: Option<SubscriptionFilter>,
    ) -> Result<Vec<GraphqlSubscription>, sqlx::Error> {
        use sqlx::QueryBuilder;

        let mut builder = QueryBuilder::new(
            "SELECT id, user_id, query, variables, channel, enabled, created_at \
             FROM graphql_subscriptions \
             WHERE user_id = ",
        );

        builder.push_bind(user_id);

        if let Some(f) = filter {
            if let Some(channel) = f.channel {
                builder.push(" AND channel = ");
                builder.push_bind(channel);
            }
            if let Some(enabled) = f.enabled {
                builder.push(" AND enabled = ");
                builder.push_bind(enabled);
            }
        }

        builder.push(" ORDER BY created_at DESC");

        builder
            .build_query_as::<GraphqlSubscription>()
            .fetch_all(pool)
            .await
    }

    pub async fn enable_subscription(
        &self,
        pool: &sqlx::PgPool,
        subscription_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE graphql_subscriptions \
             SET enabled = true \
             WHERE id = $1 AND user_id = $2",
        )
        .bind(subscription_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        if result.rows_affected() > 0 {
            if let Some(entry) = self.subscriptions.write().await.get_mut(&subscription_id) {
                entry.subscription.enabled = true;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn disable_subscription(
        &self,
        pool: &sqlx::PgPool,
        subscription_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE graphql_subscriptions \
             SET enabled = false \
             WHERE id = $1 AND user_id = $2",
        )
        .bind(subscription_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        if result.rows_affected() > 0 {
            if let Some(entry) = self.subscriptions.write().await.get_mut(&subscription_id) {
                entry.subscription.enabled = false;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn delete_subscription(
        &self,
        pool: &sqlx::PgPool,
        subscription_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM graphql_subscriptions \
             WHERE id = $1 AND user_id = $2",
        )
        .bind(subscription_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        if result.rows_affected() > 0 {
            self.subscriptions.write().await.remove(&subscription_id);

            let mut channels = self.channel_subscribers.write().await;
            for subscribers in channels.values_mut() {
                subscribers.retain(|id| *id != subscription_id);
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn publish_event(&self, channel: &str, payload: serde_json::Value) -> usize {
        let event = SubscriptionEvent {
            subscription_id: Uuid::new_v4(),
            channel: channel.to_string(),
            payload,
            timestamp: Utc::now(),
        };

        let subscribers = self.channel_subscribers.read().await;
        let channel_ids = subscribers.get(channel);

        match channel_ids {
            Some(ids) => {
                let subs = self.subscriptions.read().await;
                let mut sent = 0;
                for sub_id in ids {
                    if let Some(entry) = subs.get(sub_id)
                        && entry.subscription.enabled
                    {
                        match entry.tx.try_send(event.clone()) {
                            Ok(()) => sent += 1,
                            Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                                self.dropped
                                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }
                            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {}
                        }
                    }
                }
                sent
            }
            None => 0,
        }
    }

    /// Events dropped because a subscriber's queue was full.
    pub fn dropped_count(&self) -> u64 {
        self.dropped.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn get_channel_subscriber_count(&self, channel: &str) -> usize {
        self.channel_subscribers
            .read()
            .await
            .get(channel)
            .map(|ids| ids.len())
            .unwrap_or(0)
    }

    pub async fn get_total_subscriber_count(&self) -> usize {
        self.subscriptions.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_event_creation() {
        let event = SubscriptionEvent {
            subscription_id: Uuid::new_v4(),
            channel: "test_channel".to_string(),
            payload: serde_json::json!({"data": "test"}),
            timestamp: Utc::now(),
        };

        assert_eq!(event.channel, "test_channel");
        assert!(event.payload.get("data").is_some());
    }

    #[test]
    fn test_create_subscription_request() {
        let req = CreateSubscriptionRequest {
            query: "{ users { id } }".to_string(),
            variables: Some(serde_json::json!({"limit": 10})),
            channel: "users".to_string(),
        };

        assert_eq!(req.query, "{ users { id } }");
        assert!(req.variables.is_some());
        assert_eq!(req.channel, "users");
    }

    #[test]
    fn test_subscription_service_new() {
        let service = GraphqlSubscriptionService::new();
        assert_eq!(service.subscriptions.blocking_read().len(), 0);
        assert_eq!(service.channel_subscribers.blocking_read().len(), 0);
    }

    #[tokio::test]
    async fn test_publish_event_no_subscribers() {
        let service = GraphqlSubscriptionService::new();
        let sent = service
            .publish_event("test", serde_json::json!({"data": "test"}))
            .await;
        assert_eq!(sent, 0);
    }

    #[tokio::test]
    async fn test_get_channel_subscriber_count_empty() {
        let service = GraphqlSubscriptionService::new();
        let count = service.get_channel_subscriber_count("test").await;
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_get_total_subscriber_count_empty() {
        let service = GraphqlSubscriptionService::new();
        let count = service.get_total_subscriber_count().await;
        assert_eq!(count, 0);
    }

    #[test]
    fn test_subscription_filter_defaults() {
        let filter = SubscriptionFilter {
            channel: None,
            enabled: None,
        };
        assert!(filter.channel.is_none());
        assert!(filter.enabled.is_none());
    }
}
