#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorConfig {
    pub id: String,
    pub source_url: String,
    pub direction: MirrorDirection,
    pub interval_secs: u32,
    pub enabled: bool,
    pub auth_method: MirrorAuthMethod,
    pub last_sync: Option<DateTime<Utc>>,
    pub next_sync: Option<DateTime<Utc>>,
    pub status: MirrorStatus,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MirrorDirection {
    Pull,
    Push,
    Bidirectional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MirrorStatus {
    Idle,
    Syncing,
    Success,
    Failed,
    Scheduled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MirrorAuthMethod {
    SshKey { key_id: String },
    HttpBasic { username: String },
    Token { token_name: String },
    None,
}

pub struct MirrorManager {
    mirrors: std::sync::Mutex<Vec<MirrorConfig>>,
}

impl MirrorManager {
    pub fn new() -> Self {
        Self {
            mirrors: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn add_mirror(&self, config: MirrorConfig) -> Result<(), String> {
        let mut mirrors = self.mirrors.lock().unwrap();
        if mirrors.iter().any(|m| m.id == config.id) {
            return Err(format!("mirror with id '{}' already exists", config.id));
        }
        if mirrors.iter().any(|m| m.source_url == config.source_url) {
            return Err(format!(
                "mirror for '{}' already configured",
                config.source_url
            ));
        }
        mirrors.push(config);
        Ok(())
    }

    pub fn remove_mirror(&self, id: &str) -> bool {
        let mut mirrors = self.mirrors.lock().unwrap();
        let len_before = mirrors.len();
        mirrors.retain(|m| m.id != id);
        mirrors.len() < len_before
    }

    pub fn get_mirror(&self, id: &str) -> Option<MirrorConfig> {
        let mirrors = self.mirrors.lock().unwrap();
        mirrors.iter().find(|m| m.id == id).cloned()
    }

    pub fn list_mirrors(&self) -> Vec<MirrorConfig> {
        let mirrors = self.mirrors.lock().unwrap();
        mirrors.clone()
    }

    pub fn update_status(&self, id: &str, status: MirrorStatus) -> bool {
        let mut mirrors = self.mirrors.lock().unwrap();
        if let Some(m) = mirrors.iter_mut().find(|m| m.id == id) {
            m.status = status;
            true
        } else {
            false
        }
    }

    pub fn mark_synced(&self, id: &str) -> bool {
        let mut mirrors = self.mirrors.lock().unwrap();
        if let Some(m) = mirrors.iter_mut().find(|m| m.id == id) {
            let now = Utc::now();
            m.last_sync = Some(now);
            m.next_sync = Some(now + chrono::Duration::seconds(m.interval_secs as i64));
            m.status = MirrorStatus::Success;
            m.error = None;
            true
        } else {
            false
        }
    }

    pub fn mirrors_due_for_sync(&self) -> Vec<MirrorConfig> {
        let mirrors = self.mirrors.lock().unwrap();
        let now = Utc::now();
        mirrors
            .iter()
            .filter(|m| {
                if !m.enabled {
                    return false;
                }
                match m.next_sync {
                    Some(next) => next <= now,
                    None => true,
                }
            })
            .cloned()
            .collect()
    }

    pub fn count(&self) -> usize {
        let mirrors = self.mirrors.lock().unwrap();
        mirrors.len()
    }
}

impl Default for MirrorManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
fn test_mirror(id: &str, url: &str) -> MirrorConfig {
    MirrorConfig {
        id: id.into(),
        source_url: url.into(),
        direction: MirrorDirection::Pull,
        interval_secs: 3600,
        enabled: true,
        auth_method: MirrorAuthMethod::None,
        last_sync: None,
        next_sync: None,
        status: MirrorStatus::Idle,
        error: None,
        created_at: Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_manager() {
        let mgr = MirrorManager::new();
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn test_default_manager() {
        let mgr = MirrorManager::default();
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn test_add_mirror() {
        let mgr = MirrorManager::new();
        let result = mgr.add_mirror(test_mirror("m1", "https://github.com/example/repo.git"));
        assert!(result.is_ok());
        assert_eq!(mgr.count(), 1);
    }

    #[test]
    fn test_add_duplicate_id() {
        let mgr = MirrorManager::new();
        assert!(
            mgr.add_mirror(test_mirror("m1", "https://a.com/r.git"))
                .is_ok()
        );
        let result = mgr.add_mirror(test_mirror("m1", "https://b.com/r.git"));
        assert!(result.is_err());
        assert_eq!(mgr.count(), 1);
    }

    #[test]
    fn test_add_duplicate_url() {
        let mgr = MirrorManager::new();
        assert!(
            mgr.add_mirror(test_mirror("m1", "https://a.com/r.git"))
                .is_ok()
        );
        let result = mgr.add_mirror(test_mirror("m2", "https://a.com/r.git"));
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_mirror() {
        let mgr = MirrorManager::new();
        mgr.add_mirror(test_mirror("m1", "https://a.com/r.git"))
            .unwrap();
        assert!(mgr.remove_mirror("m1"));
        assert!(!mgr.remove_mirror("m1"));
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn test_get_mirror() {
        let mgr = MirrorManager::new();
        mgr.add_mirror(test_mirror("m1", "https://a.com/r.git"))
            .unwrap();
        let mirror = mgr.get_mirror("m1").unwrap();
        assert_eq!(mirror.id, "m1");
        assert!(mgr.get_mirror("nonexistent").is_none());
    }

    #[test]
    fn test_list_mirrors() {
        let mgr = MirrorManager::new();
        mgr.add_mirror(test_mirror("m1", "https://a.com/r1.git"))
            .unwrap();
        mgr.add_mirror(test_mirror("m2", "https://a.com/r2.git"))
            .unwrap();
        let list = mgr.list_mirrors();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_update_status() {
        let mgr = MirrorManager::new();
        mgr.add_mirror(test_mirror("m1", "https://a.com/r.git"))
            .unwrap();
        assert!(mgr.update_status("m1", MirrorStatus::Syncing));
        assert!(mgr.update_status("m1", MirrorStatus::Success));
        assert!(!mgr.update_status("nonexistent", MirrorStatus::Syncing));
    }

    #[test]
    fn test_mark_synced() {
        let mgr = MirrorManager::new();
        mgr.add_mirror(test_mirror("m1", "https://a.com/r.git"))
            .unwrap();
        assert!(mgr.mark_synced("m1"));
        let mirror = mgr.get_mirror("m1").unwrap();
        assert!(mirror.last_sync.is_some());
        assert!(mirror.next_sync.is_some());
        assert_eq!(mirror.status, MirrorStatus::Success);
        assert!(mirror.error.is_none());
    }

    #[test]
    fn test_mark_synced_nonexistent() {
        let mgr = MirrorManager::new();
        assert!(!mgr.mark_synced("nonexistent"));
    }

    #[test]
    fn test_mirrors_due_no_next_sync() {
        let mgr = MirrorManager::new();
        mgr.add_mirror(test_mirror("m1", "https://a.com/r.git"))
            .unwrap();
        let due = mgr.mirrors_due_for_sync();
        assert_eq!(due.len(), 1);
    }

    #[test]
    fn test_mirrors_due_disabled() {
        let mgr = MirrorManager::new();
        let mut mirror = test_mirror("m1", "https://a.com/r.git");
        mirror.enabled = false;
        mgr.add_mirror(mirror).unwrap();
        let due = mgr.mirrors_due_for_sync();
        assert!(due.is_empty());
    }

    #[test]
    fn test_mirror_directions() {
        let mgr = MirrorManager::new();
        let configs = vec![
            (MirrorDirection::Pull, "p1"),
            (MirrorDirection::Push, "p2"),
            (MirrorDirection::Bidirectional, "p3"),
        ];
        for (dir, id) in configs {
            let mut mirror = test_mirror(id, &format!("https://a.com/{id}.git"));
            mirror.direction = dir;
            mgr.add_mirror(mirror).unwrap();
        }
        assert_eq!(mgr.count(), 3);
        assert_eq!(
            mgr.get_mirror("p1").unwrap().direction,
            MirrorDirection::Pull
        );
        assert_eq!(
            mgr.get_mirror("p2").unwrap().direction,
            MirrorDirection::Push
        );
        assert_eq!(
            mgr.get_mirror("p3").unwrap().direction,
            MirrorDirection::Bidirectional
        );
    }

    #[test]
    fn test_auth_methods() {
        let mgr = MirrorManager::new();
        let mut m1 = test_mirror("ssh", "https://a.com/r.git");
        m1.auth_method = MirrorAuthMethod::SshKey {
            key_id: "k1".into(),
        };
        let mut m2 = test_mirror("basic", "https://b.com/r.git");
        m2.auth_method = MirrorAuthMethod::HttpBasic {
            username: "admin".into(),
        };
        let mut m3 = test_mirror("token", "https://c.com/r.git");
        m3.auth_method = MirrorAuthMethod::Token {
            token_name: "deploy".into(),
        };
        mgr.add_mirror(m1).unwrap();
        mgr.add_mirror(m2).unwrap();
        mgr.add_mirror(m3).unwrap();
        assert!(matches!(
            mgr.get_mirror("ssh").unwrap().auth_method,
            MirrorAuthMethod::SshKey { .. }
        ));
        assert!(matches!(
            mgr.get_mirror("basic").unwrap().auth_method,
            MirrorAuthMethod::HttpBasic { .. }
        ));
        assert!(matches!(
            mgr.get_mirror("token").unwrap().auth_method,
            MirrorAuthMethod::Token { .. }
        ));
    }

    #[test]
    fn test_mirror_serialization() {
        let config = test_mirror("m1", "https://github.com/example/repo.git");
        let json = serde_json::to_string(&config).unwrap();
        let de: MirrorConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(de.id, "m1");
        assert_eq!(de.source_url, "https://github.com/example/repo.git");
    }

    #[test]
    fn test_direction_serialization() {
        let d = MirrorDirection::Bidirectional;
        let json = serde_json::to_string(&d).unwrap();
        let de: MirrorDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(de, MirrorDirection::Bidirectional);
    }

    #[test]
    fn test_status_serialization() {
        let s = MirrorStatus::Syncing;
        let json = serde_json::to_string(&s).unwrap();
        let de: MirrorStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(de, MirrorStatus::Syncing);
    }
}
