#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RealtimeChannel {
    pub id: Uuid,
    pub channel_name: String,
    pub subscriber_count: i32,
    pub last_message_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RealtimeMessage {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub payload: serde_json::Value,
    pub sender_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceInfo {
    pub user_id: Uuid,
    pub username: Option<String>,
    pub status: PresenceStatus,
    pub joined_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PresenceStatus {
    Online,
    Away,
    Busy,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingIndicator {
    pub user_id: Uuid,
    pub username: Option<String>,
    pub channel: String,
    pub started_at: DateTime<Utc>,
}

struct ChannelEntry {
    channel: RealtimeChannel,
    senders: Vec<tokio::sync::mpsc::UnboundedSender<RealtimeMessage>>,
}

pub struct RealtimeChannelService {
    channels: Arc<RwLock<HashMap<String, ChannelEntry>>>,
    presence: Arc<RwLock<HashMap<String, HashMap<Uuid, PresenceInfo>>>>,
    typing: Arc<RwLock<HashMap<String, HashMap<Uuid, TypingIndicator>>>>,
}

impl Default for RealtimeChannelService {
    fn default() -> Self {
        Self::new()
    }
}

impl RealtimeChannelService {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            presence: Arc::new(RwLock::new(HashMap::new())),
            typing: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_or_get_channel(
        &self,
        pool: &sqlx::PgPool,
        channel_name: &str,
    ) -> Result<RealtimeChannel, sqlx::Error> {
        let existing = sqlx::query_as::<_, RealtimeChannel>(
            "SELECT id, channel_name, subscriber_count, last_message_at, created_at \
             FROM realtime_channels \
             WHERE channel_name = $1",
        )
        .bind(channel_name)
        .fetch_optional(pool)
        .await?;

        match existing {
            Some(channel) => Ok(channel),
            None => {
                sqlx::query_as::<_, RealtimeChannel>(
                    "INSERT INTO realtime_channels (id, channel_name) \
                     VALUES ($1, $2) \
                     RETURNING id, channel_name, subscriber_count, last_message_at, created_at",
                )
                .bind(Uuid::new_v4())
                .bind(channel_name)
                .fetch_one(pool)
                .await
            }
        }
    }

    pub async fn subscribe(
        &self,
        pool: &sqlx::PgPool,
        channel_name: &str,
    ) -> Result<(RealtimeChannel, tokio::sync::mpsc::UnboundedReceiver<RealtimeMessage>), sqlx::Error> {
        let channel = self.create_or_get_channel(pool, channel_name).await?;

        sqlx::query(
            "UPDATE realtime_channels \
             SET subscriber_count = subscriber_count + 1 \
             WHERE id = $1",
        )
        .bind(channel.id)
        .execute(pool)
        .await?;

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let mut channels = self.channels.write().await;
        let entry = channels
            .entry(channel_name.to_string())
            .or_insert_with(|| ChannelEntry {
                channel: channel.clone(),
                senders: Vec::new(),
            });

        entry.senders.push(tx);

        Ok((channel, rx))
    }

    pub async fn unsubscribe(
        &self,
        pool: &sqlx::PgPool,
        channel_name: &str,
    ) -> Result<(), sqlx::Error> {
        if let Some(entry) = self.channels.write().await.get_mut(channel_name) {
            entry.channel.subscriber_count = entry.channel.subscriber_count.saturating_sub(1);

            sqlx::query(
                "UPDATE realtime_channels \
                 SET subscriber_count = GREATEST(subscriber_count - 1, 0) \
                 WHERE channel_name = $1",
            )
            .bind(channel_name)
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    pub async fn publish_message(
        &self,
        pool: &sqlx::PgPool,
        channel_name: &str,
        payload: serde_json::Value,
        sender_id: Option<Uuid>,
    ) -> Result<RealtimeMessage, sqlx::Error> {
        let channel = self.create_or_get_channel(pool, channel_name).await?;

        let message = sqlx::query_as::<_, RealtimeMessage>(
            "INSERT INTO realtime_messages (id, channel_id, payload, sender_id) \
             VALUES ($1, $2, $3, $4) \
             RETURNING id, channel_id, payload, sender_id, created_at",
        )
        .bind(Uuid::new_v4())
        .bind(channel.id)
        .bind(&payload)
        .bind(sender_id)
        .fetch_one(pool)
        .await?;

        sqlx::query(
            "UPDATE realtime_channels \
             SET last_message_at = NOW() \
             WHERE id = $1",
        )
        .bind(channel.id)
        .execute(pool)
        .await?;

        let channels = self.channels.read().await;
        if let Some(entry) = channels.get(channel_name) {
            for sender in &entry.senders {
                let _ = sender.send(message.clone());
            }
        }

        Ok(message)
    }

    pub async fn get_channel_history(
        &self,
        pool: &sqlx::PgPool,
        channel_name: &str,
        limit: i64,
    ) -> Result<Vec<RealtimeMessage>, sqlx::Error> {
        sqlx::query_as::<_, RealtimeMessage>(
            "SELECT rm.id, rm.channel_id, rm.payload, rm.sender_id, rm.created_at \
             FROM realtime_messages rm \
             JOIN realtime_channels rc ON rm.channel_id = rc.id \
             WHERE rc.channel_name = $1 \
             ORDER BY rm.created_at DESC \
             LIMIT $2",
        )
        .bind(channel_name)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    pub async fn update_presence(
        &self,
        channel_name: &str,
        user_id: Uuid,
        username: Option<String>,
        status: PresenceStatus,
    ) {
        let mut presence = self.presence.write().await;
        let channel_presence = presence
            .entry(channel_name.to_string())
            .or_insert_with(HashMap::new);

        let now = Utc::now();
        channel_presence.insert(
            user_id,
            PresenceInfo {
                user_id,
                username,
                status,
                joined_at: now,
                last_seen: now,
            },
        );
    }

    pub async fn remove_presence(&self, channel_name: &str, user_id: Uuid) {
        let mut presence = self.presence.write().await;
        if let Some(channel_presence) = presence.get_mut(channel_name) {
            channel_presence.remove(&user_id);
        }
    }

    pub async fn get_channel_presence(&self, channel_name: &str) -> Vec<PresenceInfo> {
        let presence = self.presence.read().await;
        presence
            .get(channel_name)
            .map(|p| p.values().cloned().collect())
            .unwrap_or_default()
    }

    pub async fn update_typing_indicator(
        &self,
        channel_name: &str,
        user_id: Uuid,
        username: Option<String>,
    ) {
        let mut typing = self.typing.write().await;
        let channel_typing = typing
            .entry(channel_name.to_string())
            .or_insert_with(HashMap::new);

        channel_typing.insert(
            user_id,
            TypingIndicator {
                user_id,
                username,
                channel: channel_name.to_string(),
                started_at: Utc::now(),
            },
        );
    }

    pub async fn remove_typing_indicator(&self, channel_name: &str, user_id: Uuid) {
        let mut typing = self.typing.write().await;
        if let Some(channel_typing) = typing.get_mut(channel_name) {
            channel_typing.remove(&user_id);
        }
    }

    pub async fn get_typing_users(&self, channel_name: &str) -> Vec<TypingIndicator> {
        let typing = self.typing.read().await;
        typing
            .get(channel_name)
            .map(|t| t.values().cloned().collect())
            .unwrap_or_default()
    }

    pub async fn get_active_channels(&self) -> Vec<String> {
        self.channels.read().await.keys().cloned().collect()
    }

    pub async fn get_channel_subscriber_count(&self, channel_name: &str) -> usize {
        self.channels
            .read()
            .await
            .get(channel_name)
            .map(|e| e.senders.len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realtime_channel_creation() {
        let channel = RealtimeChannel {
            id: Uuid::new_v4(),
            channel_name: "test".to_string(),
            subscriber_count: 0,
            last_message_at: None,
            created_at: Utc::now(),
        };

        assert_eq!(channel.channel_name, "test");
        assert_eq!(channel.subscriber_count, 0);
        assert!(channel.last_message_at.is_none());
    }

    #[test]
    fn test_realtime_message_creation() {
        let message = RealtimeMessage {
            id: Uuid::new_v4(),
            channel_id: Uuid::new_v4(),
            payload: serde_json::json!({"text": "hello"}),
            sender_id: Some(Uuid::new_v4()),
            created_at: Utc::now(),
        };

        assert!(message.payload.get("text").is_some());
        assert!(message.sender_id.is_some());
    }

    #[test]
    fn test_presence_info_creation() {
        let presence = PresenceInfo {
            user_id: Uuid::new_v4(),
            username: Some("testuser".to_string()),
            status: PresenceStatus::Online,
            joined_at: Utc::now(),
            last_seen: Utc::now(),
        };

        assert_eq!(presence.status, PresenceStatus::Online);
        assert_eq!(presence.username, Some("testuser".to_string()));
    }

    #[test]
    fn test_typing_indicator_creation() {
        let typing = TypingIndicator {
            user_id: Uuid::new_v4(),
            username: Some("testuser".to_string()),
            channel: "test".to_string(),
            started_at: Utc::now(),
        };

        assert_eq!(typing.channel, "test");
    }

    #[test]
    fn test_presence_status_variants() {
        assert_eq!(PresenceStatus::Online, PresenceStatus::Online);
        assert_eq!(PresenceStatus::Away, PresenceStatus::Away);
        assert_eq!(PresenceStatus::Busy, PresenceStatus::Busy);
        assert_eq!(PresenceStatus::Offline, PresenceStatus::Offline);
    }

    #[test]
    fn test_service_new() {
        let service = RealtimeChannelService::new();
        assert_eq!(service.channels.blocking_read().len(), 0);
        assert_eq!(service.presence.blocking_read().len(), 0);
        assert_eq!(service.typing.blocking_read().len(), 0);
    }

    #[tokio::test]
    async fn test_get_active_channels_empty() {
        let service = RealtimeChannelService::new();
        let channels = service.get_active_channels().await;
        assert!(channels.is_empty());
    }

    #[tokio::test]
    async fn test_get_channel_subscriber_count_empty() {
        let service = RealtimeChannelService::new();
        let count = service.get_channel_subscriber_count("test").await;
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_update_and_get_presence() {
        let service = RealtimeChannelService::new();
        let user_id = Uuid::new_v4();

        service
            .update_presence("test", user_id, Some("user".to_string()), PresenceStatus::Online)
            .await;

        let presence = service.get_channel_presence("test").await;
        assert_eq!(presence.len(), 1);
        assert_eq!(presence[0].user_id, user_id);
        assert_eq!(presence[0].status, PresenceStatus::Online);
    }

    #[tokio::test]
    async fn test_remove_presence() {
        let service = RealtimeChannelService::new();
        let user_id = Uuid::new_v4();

        service
            .update_presence("test", user_id, None, PresenceStatus::Online)
            .await;
        assert_eq!(service.get_channel_presence("test").await.len(), 1);

        service.remove_presence("test", user_id).await;
        assert_eq!(service.get_channel_presence("test").await.len(), 0);
    }

    #[tokio::test]
    async fn test_typing_indicator() {
        let service = RealtimeChannelService::new();
        let user_id = Uuid::new_v4();

        service
            .update_typing_indicator("test", user_id, Some("user".to_string()))
            .await;

        let typing = service.get_typing_users("test").await;
        assert_eq!(typing.len(), 1);
        assert_eq!(typing[0].user_id, user_id);

        service.remove_typing_indicator("test", user_id).await;
        assert_eq!(service.get_typing_users("test").await.len(), 0);
    }
}
