#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInstance {
    pub id: Uuid,
    pub hostname: String,
    pub ip_address: IpAddr,
    pub port: u16,
    pub status: InstanceStatus,
    pub last_heartbeat: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceStatus {
    Active,
    Draining,
    Stopped,
}

impl std::fmt::Display for InstanceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstanceStatus::Active => write!(f, "active"),
            InstanceStatus::Draining => write!(f, "draining"),
            InstanceStatus::Stopped => write!(f, "stopped"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickySession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub instance_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct InstanceManager {
    instances: Arc<DashMap<Uuid, ServerInstance>>,
    sticky_sessions: Arc<DashMap<Uuid, StickySession>>,
    user_sessions: Arc<DashMap<Uuid, Vec<Uuid>>>,
    instance_id: Uuid,
    heartbeat_interval: Duration,
}

impl InstanceManager {
    pub fn new(instance_id: Uuid, heartbeat_interval: Duration) -> Self {
        Self {
            instances: Arc::new(DashMap::new()),
            sticky_sessions: Arc::new(DashMap::new()),
            user_sessions: Arc::new(DashMap::new()),
            instance_id,
            heartbeat_interval,
        }
    }

    pub fn instance_id(&self) -> Uuid {
        self.instance_id
    }

    pub fn register_instance(&self, instance: ServerInstance) {
        self.instances.insert(instance.id, instance);
    }

    pub fn unregister_instance(&self, id: Uuid) {
        self.instances.remove(&id);
    }

    pub fn get_instance(&self, id: Uuid) -> Option<ServerInstance> {
        self.instances.get(&id).map(|r| r.clone())
    }

    pub fn active_instances(&self) -> Vec<ServerInstance> {
        self.instances
            .iter()
            .filter(|r| r.value().status == InstanceStatus::Active)
            .map(|r| r.value().clone())
            .collect()
    }

    pub fn update_heartbeat(&self, id: Uuid) -> bool {
        if let Some(mut instance) = self.instances.get_mut(&id) {
            instance.last_heartbeat = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn stale_instances(&self, timeout: Duration) -> Vec<Uuid> {
        let now = Utc::now();
        self.instances
            .iter()
            .filter(|r| {
                let elapsed = now
                    .signed_duration_since(r.value().last_heartbeat)
                    .num_seconds()
                    .unsigned_abs();
                Duration::from_secs(elapsed) > timeout
            })
            .map(|r| *r.key())
            .collect()
    }

    pub fn heartbeat_interval(&self) -> Duration {
        self.heartbeat_interval
    }

    pub fn create_sticky_session(&self, user_id: Uuid, instance_id: Uuid, ttl: Duration) -> StickySession {
        let session = StickySession {
            id: Uuid::new_v4(),
            user_id,
            instance_id,
            expires_at: Utc::now() + chrono::Duration::from_std(ttl).unwrap_or(chrono::Duration::hours(1)),
            created_at: Utc::now(),
        };
        self.sticky_sessions.insert(session.id, session.clone());
        self.user_sessions
            .entry(user_id)
            .or_default()
            .push(session.id);
        session
    }

    pub fn resolve_sticky_session(&self, user_id: Uuid) -> Option<Uuid> {
        let now = Utc::now();
        if let Some(sessions) = self.user_sessions.get(&user_id) {
            for session_id in sessions.iter() {
                if let Some(session) = self.sticky_sessions.get(session_id)
                    && session.expires_at > now && session.instance_id == self.instance_id {
                        return Some(session.instance_id);
                    }
            }
        }
        None
    }

    pub fn cleanup_expired_sessions(&self) -> usize {
        let now = Utc::now();
        let expired: Vec<Uuid> = self
            .sticky_sessions
            .iter()
            .filter(|r| r.value().expires_at <= now)
            .map(|r| *r.key())
            .collect();
        let count = expired.len();
        for id in &expired {
            if let Some(session) = self.sticky_sessions.remove(id)
                && let Some(mut user_sessions) = self.user_sessions.get_mut(&session.1.user_id) {
                    user_sessions.retain(|s| s != id);
                }
        }
        count
    }

    pub fn health_status(&self) -> HealthStatusResponse {
        let active = self.active_instances();
        let total = self.instances.len();
        let active_sessions: usize = {
            let now = Utc::now();
            self.sticky_sessions
                .iter()
                .filter(|r| r.value().expires_at > now)
                .count()
        };

        HealthStatusResponse {
            instance_id: self.instance_id,
            total_instances: total,
            active_instances: active.len(),
            active_sticky_sessions: active_sessions,
            status: if active.is_empty() {
                InstanceStatus::Stopped
            } else {
                InstanceStatus::Active
            },
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatusResponse {
    pub instance_id: Uuid,
    pub total_instances: usize,
    pub active_instances: usize,
    pub active_sticky_sessions: usize,
    pub status: InstanceStatus,
    pub timestamp: DateTime<Utc>,
}

pub struct HeartbeatTask {
    manager: Arc<RwLock<InstanceManager>>,
    interval: Duration,
}

impl HeartbeatTask {
    pub fn new(manager: Arc<RwLock<InstanceManager>>, interval: Duration) -> Self {
        Self { manager, interval }
    }

    pub fn spawn(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.interval);
            loop {
                interval.tick().await;
                let mgr = self.manager.read().await;
                mgr.update_heartbeat(mgr.instance_id());
                let stale = mgr.stale_instances(Duration::from_secs(90));
                drop(mgr);
                if !stale.is_empty() {
                    let mgr = self.manager.write().await;
                    for id in stale {
                        mgr.unregister_instance(id);
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_manager() -> InstanceManager {
        InstanceManager::new(Uuid::new_v4(), Duration::from_secs(30))
    }

    fn make_instance(hostname: &str) -> ServerInstance {
        ServerInstance {
            id: Uuid::new_v4(),
            hostname: hostname.to_string(),
            ip_address: "127.0.0.1".parse().unwrap(),
            port: 8080,
            status: InstanceStatus::Active,
            last_heartbeat: Utc::now(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_register_and_unregister_instance() {
        let mgr = make_manager();
        let inst = make_instance("host1");
        let id = inst.id;
        mgr.register_instance(inst);
        assert_eq!(mgr.active_instances().len(), 1);
        mgr.unregister_instance(id);
        assert_eq!(mgr.active_instances().len(), 0);
    }

    #[test]
    fn test_update_heartbeat() {
        let mgr = make_manager();
        let inst = make_instance("host1");
        let id = inst.id;
        mgr.register_instance(inst);
        assert!(mgr.update_heartbeat(id));
        assert!(!mgr.update_heartbeat(Uuid::new_v4()));
    }

    #[test]
    fn test_stale_instances_detection() {
        let mgr = make_manager();
        let mut inst = make_instance("host1");
        inst.last_heartbeat = Utc::now() - chrono::Duration::seconds(120);
        let id = inst.id;
        mgr.register_instance(inst);
        let stale = mgr.stale_instances(Duration::from_secs(60));
        assert_eq!(stale.len(), 1);
        assert_eq!(stale[0], id);
    }

    #[test]
    fn test_sticky_session_lifecycle() {
        let mgr = make_manager();
        let user_id = Uuid::new_v4();
        let instance_id = mgr.instance_id();
        let session = mgr.create_sticky_session(user_id, instance_id, Duration::from_secs(3600));
        assert_eq!(session.user_id, user_id);
        assert_eq!(session.instance_id, instance_id);
        let resolved = mgr.resolve_sticky_session(user_id);
        assert!(resolved.is_some());
    }

    #[test]
    fn test_cleanup_expired_sessions() {
        let mgr = make_manager();
        let user_id = Uuid::new_v4();
        let instance_id = Uuid::new_v4();
        let _session = mgr.create_sticky_session(user_id, instance_id, Duration::from_secs(0));
        let cleaned = mgr.cleanup_expired_sessions();
        assert_eq!(cleaned, 1);
    }

    #[test]
    fn test_health_status() {
        let mgr = make_manager();
        let inst = make_instance("host1");
        mgr.register_instance(inst);
        let status = mgr.health_status();
        assert_eq!(status.active_instances, 1);
        assert_eq!(status.total_instances, 1);
        assert_eq!(status.status, InstanceStatus::Active);
    }

    #[test]
    fn test_instance_status_display() {
        assert_eq!(format!("{}", InstanceStatus::Active), "active");
        assert_eq!(format!("{}", InstanceStatus::Draining), "draining");
        assert_eq!(format!("{}", InstanceStatus::Stopped), "stopped");
    }

    #[test]
    fn test_instance_status_equality() {
        assert_eq!(InstanceStatus::Active, InstanceStatus::Active);
        assert_ne!(InstanceStatus::Active, InstanceStatus::Stopped);
    }
}
