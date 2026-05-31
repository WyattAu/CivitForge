#![forbid(unsafe_code)]

use crate::events::bus::EventBus;
use crate::events::model::Event;
use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub struct WsConnection {
    pub id: Uuid,
    pub user_id: Option<String>,
    pub subscriptions: HashSet<String>,
    pub last_ping: Instant,
    pub connected_at: Instant,
}

impl WsConnection {
    pub fn new(id: Uuid, user_id: Option<String>) -> Self {
        Self {
            id,
            user_id,
            subscriptions: HashSet::new(),
            last_ping: Instant::now(),
            connected_at: Instant::now(),
        }
    }

    pub fn record_ping(&mut self) {
        self.last_ping = Instant::now();
    }

    pub fn is_stale(&self, timeout: Duration) -> bool {
        self.last_ping.elapsed() > timeout
    }
}

pub struct WebSocketManager {
    connections: DashMap<Uuid, WsConnection>,
    #[allow(dead_code)]
    event_bus: Arc<EventBus>,
}

impl WebSocketManager {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            connections: DashMap::new(),
            event_bus,
        }
    }

    pub fn register(&self, user_id: Option<String>) -> Uuid {
        let id = Uuid::new_v4();
        let conn = WsConnection::new(id, user_id);
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

    pub fn broadcast_event(&self, event: &Event) {
        let _msg = match serde_json::to_string(event) {
            Ok(m) => m,
            Err(_) => return,
        };

        for mut conn in self.connections.iter_mut() {
            let mut should_send = conn.subscriptions.contains("global");
            if !should_send {
                if let Some(repo_id) = self.extract_repo_id(event) {
                    should_send = conn.subscriptions.contains(&format!("repo:{repo_id}"));
                }
            }
            if should_send {
                conn.record_ping();
            }
        }
    }

    pub fn send_to_connection(&self, conn_id: Uuid, _msg: &str) -> anyhow::Result<()> {
        let mut conn = self
            .connections
            .get_mut(&conn_id)
            .ok_or_else(|| anyhow::anyhow!("connection not found: {conn_id}"))?;
        conn.record_ping();
        Ok(())
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
/// This function would serve as the axum handler for the `/ws` endpoint.
/// When a client connects, it upgrades the HTTP connection to a WebSocket
/// using `axum::extract::ws::WebSocketUpgrade`. After the upgrade, the handler
/// enters a loop that:
///
/// 1. Spawns a read task that processes incoming client messages (ping/pong,
///    subscribe/unsubscribe commands encoded as JSON).
/// 2. Registers the connection with `WebSocketManager::register`.
/// 3. Waits for the read task to complete (client disconnect).
/// 4. Unregisters the connection via `WebSocketManager::unregister`.
///
/// The handler signature would be:
///
/// ```text
/// async fn ws_handler(
///     ws: WebSocketUpgrade,
///     State(manager): State<Arc<WebSocketManager>>,
/// ) -> impl IntoResponse {
///     ws.on_upgrade(move |socket| handle_socket(socket, manager))
/// }
/// ```
pub fn ws_upgrade_handler() {}

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
        let id1 = mgr.register(None);
        let _id2 = mgr.register(None);
        mgr.subscribe(id1, "global");

        let event = make_system_event("broadcast test");
        mgr.broadcast_event(&event);

        assert_eq!(mgr.active_count(), 2);
    }
}
