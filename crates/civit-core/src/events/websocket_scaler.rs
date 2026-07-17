#![forbid(unsafe_code)]

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsConnectionRecord {
    pub id: Uuid,
    pub user_id: Option<String>,
    pub instance_id: Uuid,
    pub channel: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct RedisChannel {
    pub name: String,
    pub subscribers: DashMap<Uuid, WsConnectionRecord>,
}

impl RedisChannel {
    pub fn new(name: String) -> Self {
        Self {
            name,
            subscribers: DashMap::new(),
        }
    }

    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }

    pub fn add_subscriber(&self, record: WsConnectionRecord) {
        self.subscribers.insert(record.id, record);
    }

    pub fn remove_subscriber(&self, id: Uuid) -> bool {
        self.subscribers.remove(&id).is_some()
    }

    pub fn has_subscriber(&self, id: Uuid) -> bool {
        self.subscribers.contains_key(&id)
    }
}

#[derive(Clone)]
pub struct WebSocketScaler {
    channels: Arc<DashMap<String, RedisChannel>>,
    instance_id: Uuid,
    message_log: Arc<DashMap<String, Vec<LoggedMessage>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggedMessage {
    pub channel: String,
    pub message_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source_instance: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStats {
    pub channel: String,
    pub subscriber_count: usize,
    pub instance_id: Uuid,
}

impl WebSocketScaler {
    pub fn new(instance_id: Uuid) -> Self {
        Self {
            channels: Arc::new(DashMap::new()),
            instance_id,
            message_log: Arc::new(DashMap::new()),
        }
    }

    pub fn instance_id(&self) -> Uuid {
        self.instance_id
    }

    pub fn subscribe(&self, channel: &str, user_id: Option<String>) -> Uuid {
        let connection_id = Uuid::new_v4();
        let record = WsConnectionRecord {
            id: connection_id,
            user_id,
            instance_id: self.instance_id,
            channel: channel.to_string(),
            created_at: chrono::Utc::now(),
        };

        self.channels
            .entry(channel.to_string())
            .or_insert_with(|| RedisChannel::new(channel.to_string()))
            .add_subscriber(record);

        connection_id
    }

    pub fn unsubscribe(&self, connection_id: Uuid, channel: &str) -> bool {
        if let Some(ch) = self.channels.get(channel) {
            ch.remove_subscriber(connection_id)
        } else {
            false
        }
    }

    pub fn unsubscribe_all(&self, connection_id: Uuid) -> usize {
        let mut removed = 0;
        for ch in self.channels.iter() {
            if ch.remove_subscriber(connection_id) {
                removed += 1;
            }
        }
        removed
    }

    pub fn broadcast(&self, channel: &str, _message: &str, message_type: &str) -> usize {
        let logged = LoggedMessage {
            channel: channel.to_string(),
            message_type: message_type.to_string(),
            timestamp: chrono::Utc::now(),
            source_instance: self.instance_id,
        };
        self.message_log
            .entry(channel.to_string())
            .or_default()
            .push(logged);

        if let Some(ch) = self.channels.get(channel) {
            ch.subscriber_count()
        } else {
            0
        }
    }

    pub fn channel_subscribers(&self, channel: &str) -> Vec<WsConnectionRecord> {
        if let Some(ch) = self.channels.get(channel) {
            ch.subscribers.iter().map(|r| r.value().clone()).collect()
        } else {
            Vec::new()
        }
    }

    pub fn local_subscribers(&self, channel: &str) -> Vec<WsConnectionRecord> {
        self.channel_subscribers(channel)
            .into_iter()
            .filter(|r| r.instance_id == self.instance_id)
            .collect()
    }

    pub fn channel_stats(&self) -> Vec<ChannelStats> {
        self.channels
            .iter()
            .map(|ch| ChannelStats {
                channel: ch.key().clone(),
                subscriber_count: ch.subscriber_count(),
                instance_id: self.instance_id,
            })
            .collect()
    }

    pub fn total_connections(&self) -> usize {
        self.channels.iter().map(|ch| ch.subscriber_count()).sum()
    }

    pub fn cleanup_stale(&self, timeout: Duration) -> usize {
        let mut removed = 0;
        let now = chrono::Utc::now();
        for ch in self.channels.iter_mut() {
            let stale: Vec<Uuid> = ch
                .subscribers
                .iter()
                .filter(|r| {
                    let elapsed = now
                        .signed_duration_since(r.value().created_at)
                        .num_seconds()
                        .unsigned_abs();
                    Duration::from_secs(elapsed) > timeout
                })
                .map(|r| *r.key())
                .collect();
            for id in stale {
                ch.remove_subscriber(id);
                removed += 1;
            }
        }
        removed
    }

    pub fn remove_empty_channels(&self) -> usize {
        let empty: Vec<String> = self
            .channels
            .iter()
            .filter(|ch| ch.subscriber_count() == 0)
            .map(|ch| ch.key().clone())
            .collect();
        let count = empty.len();
        for name in empty {
            self.channels.remove(&name);
        }
        count
    }

    pub fn message_history(&self, channel: &str, limit: usize) -> Vec<LoggedMessage> {
        self.message_log
            .get(channel)
            .map(|log| {
                let len = log.len();
                let start = len.saturating_sub(limit);
                log[start..].to_vec()
            })
            .unwrap_or_default()
    }
}

pub struct CrossInstanceBroadcaster {
    scaler: Arc<RwLock<WebSocketScaler>>,
}

impl CrossInstanceBroadcaster {
    pub fn new(scaler: Arc<RwLock<WebSocketScaler>>) -> Self {
        Self { scaler }
    }

    pub async fn publish(&self, channel: &str, message: &str, message_type: &str) {
        let scaler = self.scaler.read().await;
        scaler.broadcast(channel, message, message_type);
    }

    pub async fn subscribe(&self, channel: &str, user_id: Option<String>) -> Uuid {
        let scaler = self.scaler.read().await;
        scaler.subscribe(channel, user_id)
    }

    pub async fn unsubscribe(&self, connection_id: Uuid, channel: &str) -> bool {
        let scaler = self.scaler.read().await;
        scaler.unsubscribe(connection_id, channel)
    }

    pub async fn stats(&self) -> Vec<ChannelStats> {
        let scaler = self.scaler.read().await;
        scaler.channel_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_scaler() -> WebSocketScaler {
        WebSocketScaler::new(Uuid::new_v4())
    }

    #[test]
    fn test_subscribe_and_unsubscribe() {
        let scaler = make_scaler();
        let id = scaler.subscribe("repo:123", Some("user1".into()));
        assert_eq!(scaler.channel_subscribers("repo:123").len(), 1);
        assert!(scaler.unsubscribe(id, "repo:123"));
        assert_eq!(scaler.channel_subscribers("repo:123").len(), 0);
    }

    #[test]
    fn test_broadcast_returns_subscriber_count() {
        let scaler = make_scaler();
        scaler.subscribe("repo:123", None);
        scaler.subscribe("repo:123", None);
        let count = scaler.broadcast("repo:123", "hello", "message");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_local_subscribers_filters_by_instance() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let s1 = WebSocketScaler::new(id1);
        let s2 = WebSocketScaler::new(id2);

        s1.subscribe("global", None);
        s2.subscribe("global", None);

        assert_eq!(s1.local_subscribers("global").len(), 1);
        assert_eq!(s2.local_subscribers("global").len(), 1);
        assert_eq!(s1.channel_subscribers("global").len(), 2);
    }

    #[test]
    fn test_channel_stats() {
        let scaler = make_scaler();
        scaler.subscribe("repo:1", None);
        scaler.subscribe("repo:1", None);
        scaler.subscribe("repo:2", None);

        let stats = scaler.channel_stats();
        assert_eq!(stats.len(), 2);
    }

    #[test]
    fn test_total_connections() {
        let scaler = make_scaler();
        scaler.subscribe("repo:1", None);
        scaler.subscribe("repo:2", None);
        scaler.subscribe("repo:2", None);
        assert_eq!(scaler.total_connections(), 3);
    }

    #[test]
    fn test_cleanup_stale() {
        let scaler = make_scaler();
        scaler.subscribe("global", None);
        let removed = scaler.cleanup_stale(Duration::from_secs(0));
        assert_eq!(removed, 1);
    }

    #[test]
    fn test_remove_empty_channels() {
        let scaler = make_scaler();
        let id = scaler.subscribe("empty", None);
        scaler.unsubscribe(id, "empty");
        let removed = scaler.remove_empty_channels();
        assert_eq!(removed, 1);
    }

    #[test]
    fn test_message_history() {
        let scaler = make_scaler();
        scaler.broadcast("repo:1", "msg1", "event");
        scaler.broadcast("repo:1", "msg2", "event");
        let history = scaler.message_history("repo:1", 1);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].message_type, "event");
    }

    #[test]
    fn test_unsubscribe_nonexistent_returns_false() {
        let scaler = make_scaler();
        assert!(!scaler.unsubscribe(Uuid::new_v4(), "nope"));
    }

    #[test]
    fn test_unsubscribe_all() {
        let scaler = make_scaler();
        let id = scaler.subscribe("repo:1", None);
        scaler.subscribe("repo:2", None);
        let removed = scaler.unsubscribe_all(id);
        assert_eq!(removed, 2);
    }
}
