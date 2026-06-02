#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployKey {
    pub id: String,
    pub title: String,
    pub public_key: String,
    pub fingerprint: String,
    pub key_type: DeployKeyType,
    pub repository_id: String,
    pub read_only: bool,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeployKeyType {
    Rsa,
    Ed25519,
    Ecdsa,
}

pub struct DeployKeyManager {
    keys: std::sync::Mutex<Vec<DeployKey>>,
}

impl DeployKeyManager {
    pub fn new() -> Self {
        Self {
            keys: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn add_key(&self, key: DeployKey) -> Result<(), String> {
        let mut keys = self.keys.lock().unwrap();
        if keys
            .iter()
            .any(|k| k.fingerprint == key.fingerprint && k.repository_id == key.repository_id)
        {
            return Err("deploy key with this fingerprint already exists for repository".into());
        }
        keys.push(key);
        Ok(())
    }

    pub fn get_by_fingerprint(&self, fingerprint: &str) -> Vec<DeployKey> {
        let keys = self.keys.lock().unwrap();
        keys.iter()
            .filter(|k| k.fingerprint == fingerprint)
            .cloned()
            .collect()
    }

    pub fn get_by_repository(&self, repo_id: &str) -> Vec<DeployKey> {
        let keys = self.keys.lock().unwrap();
        keys.iter()
            .filter(|k| k.repository_id == repo_id)
            .cloned()
            .collect()
    }

    pub fn remove_key(&self, id: &str) -> bool {
        let mut keys = self.keys.lock().unwrap();
        let before = keys.len();
        keys.retain(|k| k.id != id);
        keys.len() < before
    }

    pub fn deactivate_key(&self, id: &str) -> bool {
        let mut keys = self.keys.lock().unwrap();
        if let Some(key) = keys.iter_mut().find(|k| k.id == id) {
            key.active = false;
            return true;
        }
        false
    }

    pub fn record_usage(&self, fingerprint: &str) {
        let mut keys = self.keys.lock().unwrap();
        if let Some(key) = keys.iter_mut().find(|k| k.fingerprint == fingerprint) {
            key.last_used_at = Some(Utc::now());
        }
    }

    pub fn count(&self) -> usize {
        self.keys.lock().unwrap().len()
    }
}

impl Default for DeployKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key(id: &str, repo: &str, key_type: DeployKeyType) -> DeployKey {
        DeployKey {
            id: id.to_string(),
            title: format!("Key {id}"),
            public_key: format!("ssh-{id} AAAA..."),
            fingerprint: format!("fp-{id}"),
            key_type,
            repository_id: repo.to_string(),
            read_only: true,
            created_at: Utc::now(),
            last_used_at: None,
            expires_at: None,
            active: true,
        }
    }

    #[test]
    fn test_add_key() {
        let mgr = DeployKeyManager::new();
        mgr.add_key(make_key("k1", "repo-1", DeployKeyType::Ed25519))
            .unwrap();
        assert_eq!(mgr.count(), 1);
    }

    #[test]
    fn test_duplicate_fingerprint_same_repo_rejected() {
        let mgr = DeployKeyManager::new();
        mgr.add_key(make_key("k1", "repo-1", DeployKeyType::Ed25519))
            .unwrap();
        let mut dup = make_key("k2", "repo-1", DeployKeyType::Ed25519);
        dup.fingerprint = "fp-k1".to_string();
        assert!(mgr.add_key(dup).is_err());
        assert_eq!(mgr.count(), 1);
    }

    #[test]
    fn test_same_fingerprint_different_repo_allowed() {
        let mgr = DeployKeyManager::new();
        mgr.add_key(make_key("k1", "repo-1", DeployKeyType::Ed25519))
            .unwrap();
        let mut key2 = make_key("k2", "repo-2", DeployKeyType::Ed25519);
        key2.fingerprint = "fp-k1".to_string();
        assert!(mgr.add_key(key2).is_ok());
        assert_eq!(mgr.count(), 2);
    }

    #[test]
    fn test_get_by_fingerprint() {
        let mgr = DeployKeyManager::new();
        mgr.add_key(make_key("k1", "repo-1", DeployKeyType::Rsa))
            .unwrap();
        let results = mgr.get_by_fingerprint("fp-k1");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "k1");
    }

    #[test]
    fn test_get_by_fingerprint_missing() {
        let mgr = DeployKeyManager::new();
        let results = mgr.get_by_fingerprint("nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn test_get_by_repository() {
        let mgr = DeployKeyManager::new();
        mgr.add_key(make_key("k1", "repo-1", DeployKeyType::Ed25519))
            .unwrap();
        mgr.add_key(make_key("k2", "repo-1", DeployKeyType::Rsa))
            .unwrap();
        mgr.add_key(make_key("k3", "repo-2", DeployKeyType::Ed25519))
            .unwrap();
        let results = mgr.get_by_repository("repo-1");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_remove_key() {
        let mgr = DeployKeyManager::new();
        mgr.add_key(make_key("k1", "repo-1", DeployKeyType::Ed25519))
            .unwrap();
        assert!(mgr.remove_key("k1"));
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn test_remove_nonexistent_key() {
        let mgr = DeployKeyManager::new();
        assert!(!mgr.remove_key("nonexistent"));
    }

    #[test]
    fn test_deactivate_key() {
        let mgr = DeployKeyManager::new();
        mgr.add_key(make_key("k1", "repo-1", DeployKeyType::Ed25519))
            .unwrap();
        assert!(mgr.deactivate_key("k1"));
        let keys = mgr.get_by_repository("repo-1");
        assert!(!keys[0].active);
    }

    #[test]
    fn test_deactivate_nonexistent_key() {
        let mgr = DeployKeyManager::new();
        assert!(!mgr.deactivate_key("nonexistent"));
    }

    #[test]
    fn test_record_usage() {
        let mgr = DeployKeyManager::new();
        mgr.add_key(make_key("k1", "repo-1", DeployKeyType::Ed25519))
            .unwrap();
        mgr.record_usage("fp-k1");
        let keys = mgr.get_by_fingerprint("fp-k1");
        assert!(keys[0].last_used_at.is_some());
    }

    #[test]
    fn test_record_usage_nonexistent() {
        let mgr = DeployKeyManager::new();
        mgr.record_usage("nonexistent");
    }

    #[test]
    fn test_key_serialization_roundtrip() {
        let key = make_key("k1", "repo-1", DeployKeyType::Ecdsa);
        let json = serde_json::to_string(&key).unwrap();
        let de: DeployKey = serde_json::from_str(&json).unwrap();
        assert_eq!(de.id, "k1");
        assert_eq!(de.repository_id, "repo-1");
        assert_eq!(de.key_type, DeployKeyType::Ecdsa);
    }

    #[test]
    fn test_key_type_serialization() {
        assert_eq!(
            serde_json::to_string(&DeployKeyType::Rsa).unwrap(),
            "\"Rsa\""
        );
        assert_eq!(
            serde_json::to_string(&DeployKeyType::Ed25519).unwrap(),
            "\"Ed25519\""
        );
        assert_eq!(
            serde_json::to_string(&DeployKeyType::Ecdsa).unwrap(),
            "\"Ecdsa\""
        );
    }

    #[test]
    fn test_key_type_equality() {
        assert_eq!(DeployKeyType::Rsa, DeployKeyType::Rsa);
        assert_ne!(DeployKeyType::Ed25519, DeployKeyType::Ecdsa);
    }

    #[test]
    fn test_default_is_empty() {
        let mgr = DeployKeyManager::default();
        assert_eq!(mgr.count(), 0);
    }
}
