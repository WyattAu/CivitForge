#![forbid(unsafe_code)]

use crate::events::bus::EventBus;
use crate::events::model::{Event, EventCategory, EventPayload, PresenceAction};
use axum::{
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum WsCommand {
    #[serde(rename = "subscribe")]
    Subscribe { topic: String },
    #[serde(rename = "unsubscribe")]
    Unsubscribe { topic: String },
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "presence")]
    Presence {
        resource_type: String,
        resource_id: String,
        action: PresenceAction,
    },
}

pub struct WsConnection {
    pub id: Uuid,
    pub user_id: Option<String>,
    pub subscriptions: HashSet<String>,
    pub last_ping: Instant,
    pub connected_at: Instant,
    pub tx: mpsc::UnboundedSender<String>,
    pub current_resource: Option<(String, String)>,
    pub entered_at: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct PresenceInfo {
    pub user_id: Option<String>,
    pub connection_id: Uuid,
    pub entered_at: Instant,
}

impl WsConnection {
    pub fn new(id: Uuid, user_id: Option<String>, tx: mpsc::UnboundedSender<String>) -> Self {
        Self {
            id,
            user_id,
            subscriptions: HashSet::new(),
            last_ping: Instant::now(),
            connected_at: Instant::now(),
            tx,
            current_resource: None,
            entered_at: None,
        }
    }

    pub fn record_ping(&mut self) {
        self.last_ping = Instant::now();
    }

    pub fn is_stale(&self, timeout: Duration) -> bool {
        self.last_ping.elapsed() > timeout
    }

    pub fn send(&self, msg: &str) -> bool {
        self.tx.send(msg.to_string()).is_ok()
    }
}

#[derive(Clone)]
pub struct WebSocketManager {
    connections: Arc<DashMap<Uuid, WsConnection>>,
    #[allow(dead_code)]
    event_bus: Arc<EventBus>,
}

impl WebSocketManager {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
            event_bus,
        }
    }

    pub fn register(&self, user_id: Option<String>) -> Uuid {
        let id = Uuid::new_v4();
        let (tx, _rx) = mpsc::unbounded_channel::<String>();
        let conn = WsConnection::new(id, user_id, tx);
        self.connections.insert(id, conn);
        id
    }

    /// Register a connection with a real sender channel (used by the WebSocket handler).
    pub fn register_with_channel(
        &self,
        user_id: Option<String>,
        tx: mpsc::UnboundedSender<String>,
    ) -> Uuid {
        let id = Uuid::new_v4();
        let conn = WsConnection::new(id, user_id, tx);
        self.connections.insert(id, conn);
        id
    }

    pub fn unregister(&self, id: Uuid) {
        self.connections.remove(&id);
    }

    pub fn subscribe(&self, conn_id: Uuid, topic: &str) -> bool {
        if let Some(mut conn) = self.connections.get_mut(&conn_id) {
            conn.subscriptions.insert(topic.to_string());
            true
        } else {
            false
        }
    }

    pub fn unsubscribe(&self, conn_id: Uuid, topic: &str) -> bool {
        if let Some(mut conn) = self.connections.get_mut(&conn_id) {
            conn.subscriptions.remove(topic);
            true
        } else {
            false
        }
    }

    /// Broadcast an event to all matching connections. Actually sends
    /// the serialized event through each connection's mpsc channel.
    pub fn broadcast_event(&self, event: &Event) {
        let msg = match serde_json::to_string(event) {
            Ok(m) => m,
            Err(_) => return,
        };

        let mut stale_ids: Vec<Uuid> = Vec::new();

        for mut conn in self.connections.iter_mut() {
            let mut should_send = conn.subscriptions.contains("global");
            if !should_send {
                if let Some(repo_id) = self.extract_repo_id(event) {
                    should_send = conn.subscriptions.contains(&format!("repo:{repo_id}"));
                }
            }
            if should_send {
                if !conn.send(&msg) {
                    // Channel closed -- connection is dead
                    stale_ids.push(conn.id);
                } else {
                    conn.record_ping();
                }
            }
        }

        // Clean up dead connections
        for id in stale_ids {
            self.connections.remove(&id);
        }
    }

    pub fn send_to_connection(&self, conn_id: Uuid, msg: &str) -> anyhow::Result<()> {
        let conn = self
            .connections
            .get(&conn_id)
            .ok_or_else(|| anyhow::anyhow!("connection not found: {conn_id}"))?;
        if conn.send(msg) {
            Ok(())
        } else {
            Err(anyhow::anyhow!("send failed: channel closed"))
        }
    }

    pub fn active_count(&self) -> usize {
        self.connections.len()
    }

    pub fn connection_info(&self, id: Uuid) -> Option<WsConnection> {
        self.connections.get(&id).map(|c| WsConnection {
            id: c.id,
            user_id: c.user_id.clone(),
            subscriptions: c.subscriptions.clone(),
            last_ping: c.last_ping,
            connected_at: c.connected_at,
            tx: c.tx.clone(),
            current_resource: c.current_resource.clone(),
            entered_at: c.entered_at,
        })
    }

    pub fn cleanup_stale(&self, timeout: Duration) -> usize {
        let stale_ids: Vec<Uuid> = self
            .connections
            .iter()
            .filter(|c| c.value().is_stale(timeout))
            .map(|c| *c.key())
            .collect();
        let count = stale_ids.len();
        for id in stale_ids {
            self.connections.remove(&id);
        }
        count
    }

    pub fn presence_for_resource(
        &self,
        resource_type: &str,
        resource_id: &str,
    ) -> Vec<PresenceInfo> {
        self.connections
            .iter()
            .filter_map(|c| {
                let conn = c.value();
                if conn.current_resource.as_ref()
                    == Some(&(resource_type.to_string(), resource_id.to_string()))
                {
                    Some(PresenceInfo {
                        user_id: conn.user_id.clone(),
                        connection_id: conn.id,
                        entered_at: conn.entered_at.unwrap_or(conn.connected_at),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn presence_summary(
        &self,
        resources: &[(String, String)],
    ) -> HashMap<String, Vec<PresenceInfo>> {
        let mut result: HashMap<String, Vec<PresenceInfo>> = HashMap::new();
        for (rt, ri) in resources {
            let key = format!("{rt}:{ri}");
            let users = self.presence_for_resource(rt, ri);
            result.insert(key, users);
        }
        result
    }

    fn broadcast_presence(
        &self,
        resource_type: &str,
        resource_id: &str,
        user_id: Option<&str>,
        action: PresenceAction,
    ) {
        let topic = format!("presence:{resource_type}:{resource_id}");
        let active_users: Vec<String> = self
            .presence_for_resource(resource_type, resource_id)
            .iter()
            .filter_map(|p| p.user_id.clone())
            .collect();

        let event = Event::new(
            EventCategory::Presence,
            EventPayload::PresenceEvent {
                resource_type: resource_type.to_string(),
                resource_id: resource_id.to_string(),
                user_id: user_id.map(String::from),
                action,
                active_users,
            },
            "civitforge.local".to_string(),
        );

        let msg = match serde_json::to_string(&event) {
            Ok(m) => m,
            Err(_) => return,
        };

        let mut stale_ids: Vec<Uuid> = Vec::new();
        for conn in self.connections.iter_mut() {
            if (conn.subscriptions.contains(&topic) || conn.subscriptions.contains("global"))
                && !conn.send(&msg)
            {
                stale_ids.push(conn.id);
            }
        }
        for id in stale_ids {
            self.connections.remove(&id);
        }
    }

    pub(crate) fn enter_presence(&self, conn_id: Uuid, resource_type: &str, resource_id: &str) {
        let user_id = if let Some(mut conn) = self.connections.get_mut(&conn_id) {
            conn.current_resource = Some((resource_type.to_string(), resource_id.to_string()));
            conn.entered_at = Some(Instant::now());
            conn.user_id.clone()
        } else {
            return;
        };
        self.broadcast_presence(
            resource_type,
            resource_id,
            user_id.as_deref(),
            PresenceAction::Enter,
        );
    }

    pub(crate) fn leave_presence(&self, conn_id: Uuid) {
        let (resource_type, resource_id, user_id) = {
            if let Some(mut conn) = self.connections.get_mut(&conn_id) {
                let old = conn.current_resource.take();
                conn.entered_at = None;
                let uid = conn.user_id.clone();
                match old {
                    Some((rt, ri)) => (Some(rt), Some(ri), uid),
                    None => return,
                }
            } else {
                return;
            }
        };
        if let (Some(rt), Some(ri)) = (resource_type, resource_id) {
            self.broadcast_presence(&rt, &ri, user_id.as_deref(), PresenceAction::Leave);
        }
    }

    fn extract_repo_id<'a>(&self, event: &'a Event) -> Option<&'a str> {
        use crate::events::model::EventPayload;
        match &event.payload {
            EventPayload::PushEvent { repo_id, .. } => Some(repo_id),
            EventPayload::PrEvent { repo_id, .. } => Some(repo_id),
            EventPayload::IssueEvent { repo_id, .. } => Some(repo_id),
            EventPayload::CiEvent { repo_id, .. } => Some(repo_id),
            _ => None,
        }
    }
}

/// Axum WebSocket upgrade handler.
///
/// Upgrades the HTTP connection to WebSocket, spawns read/write tasks,
/// and integrates with `WebSocketManager` for pub/sub event delivery.
pub async fn ws_upgrade_handler(
    ws: WebSocketUpgrade,
    State(manager): State<Arc<RwLock<WebSocketManager>>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, manager))
}

async fn handle_socket(socket: WebSocket, manager: Arc<RwLock<WebSocketManager>>) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // Register connection
    let conn_id = {
        let mgr = manager.read().await;
        mgr.register_with_channel(None, tx)
    };

    // Write task: forward messages from channel to WebSocket
    let write_handle = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Read task: process incoming commands
    let mgr_read = manager.clone();
    let cid = conn_id;
    while let Some(Ok(msg)) = ws_receiver.next().await {
        if let Ok(text) = msg.to_text() {
            if let Ok(cmd) = serde_json::from_str::<WsCommand>(text) {
                let mgr = mgr_read.read().await;
                match cmd {
                    WsCommand::Subscribe { topic } => {
                        mgr.subscribe(cid, &topic);
                    }
                    WsCommand::Unsubscribe { topic } => {
                        mgr.unsubscribe(cid, &topic);
                    }
                    WsCommand::Ping => {
                        mgr.subscribe(cid, "global");
                    }
                    WsCommand::Presence {
                        resource_type,
                        resource_id,
                        action,
                    } => match action {
                        PresenceAction::Enter => {
                            mgr.enter_presence(cid, &resource_type, &resource_id);
                        }
                        PresenceAction::Leave => {
                            mgr.leave_presence(cid);
                        }
                    },
                }
            }
        }
    }

    // Cleanup on disconnect
    write_handle.abort();
    let mgr = manager.write().await;
    mgr.leave_presence(conn_id);
    mgr.unregister(conn_id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::model::{EventCategory, EventPayload, SystemLevel};

    fn make_bus() -> Arc<EventBus> {
        Arc::new(EventBus::new(100))
    }

    fn make_system_event(msg: &str) -> Event {
        Event::new(
            EventCategory::System,
            EventPayload::SystemEvent {
                level: SystemLevel::Info,
                message: msg.to_string(),
            },
            "test.local".to_string(),
        )
    }

    #[test]
    fn register_and_unregister() {
        let bus = make_bus();
        let mgr = WebSocketManager::new(bus);
        let id = mgr.register(Some("user-1".to_string()));
        assert_eq!(mgr.active_count(), 1);
        assert!(mgr.connection_info(id).is_some());
        mgr.unregister(id);
        assert_eq!(mgr.active_count(), 0);
        assert!(mgr.connection_info(id).is_none());
    }

    #[test]
    fn subscribe_topic() {
        let bus = make_bus();
        let mgr = WebSocketManager::new(bus);
        let id = mgr.register(None);
        assert!(mgr.subscribe(id, "global"));
        assert!(mgr.subscribe(id, "repo:abc"));

        let info = mgr.connection_info(id).unwrap();
        assert!(info.subscriptions.contains("global"));
        assert!(info.subscriptions.contains("repo:abc"));

        assert!(!mgr.subscribe(Uuid::nil(), "global"));
    }

    #[test]
    fn unsubscribe_topic() {
        let bus = make_bus();
        let mgr = WebSocketManager::new(bus);
        let id = mgr.register(None);
        mgr.subscribe(id, "repo:xyz");
        assert!(mgr.unsubscribe(id, "repo:xyz"));

        let info = mgr.connection_info(id).unwrap();
        assert!(!info.subscriptions.contains("repo:xyz"));
    }

    #[test]
    fn send_to_unknown_connection_fails() {
        let bus = make_bus();
        let mgr = WebSocketManager::new(bus);
        let result = mgr.send_to_connection(Uuid::nil(), "hello");
        assert!(result.is_err());
    }

    #[test]
    fn cleanup_stale_connections() {
        let bus = make_bus();
        let mgr = WebSocketManager::new(bus);

        let id1 = mgr.register(None);
        let _id2 = mgr.register(None);

        let info = mgr.connection_info(id1).unwrap();
        assert!(!info.is_stale(Duration::from_secs(60)));

        let removed = mgr.cleanup_stale(Duration::from_secs(0));
        assert_eq!(removed, 2);
        assert_eq!(mgr.active_count(), 0);
    }

    #[test]
    fn broadcast_to_global_subscribers() {
        let bus = make_bus();
        let mgr = WebSocketManager::new(bus);
        let (tx1, _rx1) = mpsc::unbounded_channel::<String>();
        let (tx2, _rx2) = mpsc::unbounded_channel::<String>();
        let id1 = mgr.register_with_channel(None, tx1);
        let _id2 = mgr.register_with_channel(None, tx2);
        mgr.subscribe(id1, "global");

        let event = make_system_event("broadcast test");
        mgr.broadcast_event(&event);

        // id1 has alive receiver, should remain
        assert_eq!(mgr.active_count(), 2);
    }

    #[test]
    fn register_with_channel() {
        let bus = make_bus();
        let mgr = WebSocketManager::new(bus);
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        let id = mgr.register_with_channel(Some("user-2".to_string()), tx);
        assert_eq!(mgr.active_count(), 1);

        // Send via manager
        assert!(mgr.send_to_connection(id, "hello").is_ok());
        let received = rx.try_recv().unwrap();
        assert_eq!(received, "hello");
    }

    #[test]
    fn broadcast_sends_to_matching_channels() {
        let bus = make_bus();
        let mgr = WebSocketManager::new(bus);

        let (tx1, mut rx1) = mpsc::unbounded_channel::<String>();
        let (tx2, mut rx2) = mpsc::unbounded_channel::<String>();
        let id1 = mgr.register_with_channel(None, tx1);
        let id2 = mgr.register_with_channel(None, tx2);

        mgr.subscribe(id1, "global");
        mgr.subscribe(id2, "repo:abc");

        let event = make_system_event("test broadcast send");
        mgr.broadcast_event(&event);

        // id1 subscribed to global, should receive
        let msg1 = rx1.try_recv().unwrap();
        assert!(msg1.contains("test broadcast send"));

        // id2 not subscribed to global, should not receive
        assert!(rx2.try_recv().is_err());
    }

    #[test]
    fn ws_command_deserialization() {
        let sub: WsCommand =
            serde_json::from_str(r#"{"type":"subscribe","topic":"repo:123"}"#).unwrap();
        match sub {
            WsCommand::Subscribe { topic } => assert_eq!(topic, "repo:123"),
            _ => panic!("expected Subscribe"),
        }

        let unsub: WsCommand =
            serde_json::from_str(r#"{"type":"unsubscribe","topic":"global"}"#).unwrap();
        match unsub {
            WsCommand::Unsubscribe { topic } => assert_eq!(topic, "global"),
            _ => panic!("expected Unsubscribe"),
        }

        let ping: WsCommand = serde_json::from_str(r#"{"type":"ping"}"#).unwrap();
        assert!(matches!(ping, WsCommand::Ping));
    }

    #[test]
    fn presence_command_deserialization() {
        let enter: WsCommand = serde_json::from_str(
            r#"{"type":"presence","resource_type":"pr","resource_id":"pr-1","action":"enter"}"#,
        )
        .unwrap();
        match enter {
            WsCommand::Presence {
                resource_type,
                resource_id,
                action,
            } => {
                assert_eq!(resource_type, "pr");
                assert_eq!(resource_id, "pr-1");
                assert_eq!(action, PresenceAction::Enter);
            }
            _ => panic!("expected Presence"),
        }

        let leave: WsCommand = serde_json::from_str(
            r#"{"type":"presence","resource_type":"repo","resource_id":"repo-1","action":"leave"}"#,
        )
        .unwrap();
        match leave {
            WsCommand::Presence { action, .. } => assert_eq!(action, PresenceAction::Leave),
            _ => panic!("expected Presence Leave"),
        }
    }

    #[test]
    fn presence_enter_leave_tracking() {
        let bus = make_bus();
        let mgr = WebSocketManager::new(bus);

        let (tx1, _rx1) = mpsc::unbounded_channel::<String>();
        let (tx2, _rx2) = mpsc::unbounded_channel::<String>();
        let id1 = mgr.register_with_channel(Some("alice".to_string()), tx1);
        let id2 = mgr.register_with_channel(Some("bob".to_string()), tx2);

        mgr.enter_presence(id1, "pr", "pr-123");
        mgr.enter_presence(id2, "pr", "pr-123");

        let present = mgr.presence_for_resource("pr", "pr-123");
        assert_eq!(present.len(), 2);

        mgr.leave_presence(id1);
        let present = mgr.presence_for_resource("pr", "pr-123");
        assert_eq!(present.len(), 1);
        assert_eq!(present[0].user_id.as_deref(), Some("bob"));
    }

    #[test]
    fn presence_for_resource_empty() {
        let bus = make_bus();
        let mgr = WebSocketManager::new(bus);
        let present = mgr.presence_for_resource("repo", "repo-999");
        assert!(present.is_empty());
    }

    #[test]
    fn presence_summary_multiple_resources() {
        let bus = make_bus();
        let mgr = WebSocketManager::new(bus);

        let (tx1, _) = mpsc::unbounded_channel::<String>();
        let (tx2, _) = mpsc::unbounded_channel::<String>();
        let id1 = mgr.register_with_channel(Some("alice".to_string()), tx1);
        let id2 = mgr.register_with_channel(Some("bob".to_string()), tx2);

        mgr.enter_presence(id1, "repo", "repo-1");
        mgr.enter_presence(id2, "pr", "pr-456");

        let summary = mgr.presence_summary(&[
            ("repo".to_string(), "repo-1".to_string()),
            ("pr".to_string(), "pr-456".to_string()),
            ("issue".to_string(), "issue-789".to_string()),
        ]);

        assert_eq!(summary["repo:repo-1"].len(), 1);
        assert_eq!(summary["pr:pr-456"].len(), 1);
        assert!(summary["issue:issue-789"].is_empty());
    }

    #[test]
    fn broadcast_presence_event_on_enter() {
        let bus = make_bus();
        let mgr = WebSocketManager::new(bus);

        let (tx_enterer, _) = mpsc::unbounded_channel::<String>();
        let (tx_watcher, mut rx_watcher) = mpsc::unbounded_channel::<String>();
        let id_enterer = mgr.register_with_channel(Some("alice".to_string()), tx_enterer);
        let id_watcher = mgr.register_with_channel(None, tx_watcher);

        let presence_topic = "presence:pr:pr-10";
        mgr.subscribe(id_watcher, presence_topic);

        mgr.enter_presence(id_enterer, "pr", "pr-10");

        let received = rx_watcher.try_recv().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&received).unwrap();
        // EventPayload uses serde(tag="type", content="data") so fields are nested under .payload.data
        assert_eq!(parsed["category"], "presence");
        assert_eq!(parsed["payload"]["data"]["resource_type"], "pr");
        assert_eq!(parsed["payload"]["data"]["resource_id"], "pr-10");
        assert_eq!(parsed["payload"]["data"]["action"], "enter");
        assert_eq!(parsed["payload"]["data"]["user_id"], "alice");
        assert!(
            parsed["payload"]["data"]["active_users"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!("alice"))
        );
    }

    #[test]
    fn broadcast_presence_event_on_leave() {
        let bus = make_bus();
        let mgr = WebSocketManager::new(bus);

        let (tx_enterer, _) = mpsc::unbounded_channel::<String>();
        let (tx_watcher, mut rx_watcher) = mpsc::unbounded_channel::<String>();
        let id_enterer = mgr.register_with_channel(Some("bob".to_string()), tx_enterer);
        let id_watcher = mgr.register_with_channel(None, tx_watcher);

        let presence_topic = "presence:issue:issue-5";
        mgr.subscribe(id_watcher, presence_topic);

        mgr.enter_presence(id_enterer, "issue", "issue-5");
        rx_watcher.try_recv().unwrap();

        mgr.leave_presence(id_enterer);
        let received = rx_watcher.try_recv().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&received).unwrap();
        assert_eq!(parsed["payload"]["data"]["action"], "leave");
        assert_eq!(
            parsed["payload"]["data"]["active_users"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
    }

    #[test]
    fn presence_cleared_on_disconnect() {
        let bus = make_bus();
        let mgr = WebSocketManager::new(bus);

        let (tx1, _) = mpsc::unbounded_channel::<String>();
        let (tx2, mut rx2) = mpsc::unbounded_channel::<String>();
        let id1 = mgr.register_with_channel(Some("charlie".to_string()), tx1);
        let id2 = mgr.register_with_channel(None, tx2);

        let presence_topic = "presence:repo:repo-1";
        mgr.subscribe(id2, presence_topic);

        mgr.enter_presence(id1, "repo", "repo-1");
        rx2.try_recv().unwrap();

        // leave_presence broadcasts a leave event and clears the resource
        mgr.leave_presence(id1);
        assert!(mgr.presence_for_resource("repo", "repo-1").is_empty());

        let received = rx2.try_recv().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&received).unwrap();
        assert_eq!(parsed["payload"]["data"]["action"], "leave");
    }

    #[test]
    fn dead_channel_cleaned_on_broadcast() {
        let bus = make_bus();
        let mgr = WebSocketManager::new(bus);

        let (tx, rx) = mpsc::unbounded_channel::<String>();
        let id = mgr.register_with_channel(None, tx);
        assert_eq!(mgr.active_count(), 1);

        // Drop receiver to simulate dead connection
        drop(rx);

        let event = make_system_event("stale");
        mgr.subscribe(id, "global");
        mgr.broadcast_event(&event);

        // Dead connection should be cleaned up
        assert_eq!(mgr.active_count(), 0);
    }
}
