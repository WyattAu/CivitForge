#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct SshKeyRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key_type: String,
    pub public_key: String,
    pub fingerprint: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SshKeyType {
    Ed25519,
    EcdsaP256,
    Rsa4096,
    Unknown(String),
}

impl SshKeyType {
    pub fn identify(key_type: &str) -> Self {
        match key_type {
            "ssh-ed25519" => SshKeyType::Ed25519,
            "ecdsa-sha2-nistp256" => SshKeyType::EcdsaP256,
            "rsa" => SshKeyType::Rsa4096,
            other => SshKeyType::Unknown(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            SshKeyType::Ed25519 => "ssh-ed25519",
            SshKeyType::EcdsaP256 => "ecdsa-sha2-nistp256",
            SshKeyType::Rsa4096 => "rsa",
            SshKeyType::Unknown(s) => s.as_str(),
        }
    }
}

pub trait SshKeyStore: Send + Sync {
    fn lookup_by_fingerprint(&self, fingerprint: &str) -> Result<Option<SshKeyRecord>, String>;
    fn lookup_by_user(&self, user_id: Uuid) -> Result<Vec<SshKeyRecord>, String>;
    fn add_key(&self, record: SshKeyRecord) -> Result<(), String>;
    fn remove_key(&self, id: Uuid) -> Result<bool, String>;
    fn list_keys(&self) -> Result<Vec<SshKeyRecord>, String>;
}

#[derive(Debug, Clone)]
pub struct InMemorySshKeyStore {
    keys: Arc<DashMap<Uuid, SshKeyRecord>>,
    fingerprints: Arc<DashMap<String, Uuid>>,
    user_keys: Arc<DashMap<Uuid, Vec<Uuid>>>,
}

impl InMemorySshKeyStore {
    pub fn new() -> Self {
        Self {
            keys: Arc::new(DashMap::new()),
            fingerprints: Arc::new(DashMap::new()),
            user_keys: Arc::new(DashMap::new()),
        }
    }
}

impl Default for InMemorySshKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SshKeyStore for InMemorySshKeyStore {
    fn lookup_by_fingerprint(&self, fingerprint: &str) -> Result<Option<SshKeyRecord>, String> {
        Ok(self
            .fingerprints
            .get(fingerprint)
            .and_then(|id| self.keys.get(&id).map(|r| r.clone())))
    }

    fn lookup_by_user(&self, user_id: Uuid) -> Result<Vec<SshKeyRecord>, String> {
        Ok(self
            .user_keys
            .get(&user_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.keys.get(id).map(|r| r.clone()))
                    .collect()
            })
            .unwrap_or_default())
    }

    fn add_key(&self, record: SshKeyRecord) -> Result<(), String> {
        if self.fingerprints.contains_key(&record.fingerprint) {
            return Err("key with this fingerprint already exists".to_string());
        }
        self.keys.insert(record.id, record.clone());
        self.fingerprints.insert(record.fingerprint.clone(), record.id);
        self.user_keys
            .entry(record.user_id)
            .or_default()
            .push(record.id);
        Ok(())
    }

    fn remove_key(&self, id: Uuid) -> Result<bool, String> {
        if let Some((_, record)) = self.keys.remove(&id) {
            self.fingerprints.remove(&record.fingerprint);
            if let Some(mut ids) = self.user_keys.get_mut(&record.user_id) {
                ids.retain(|k| *k != id);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn list_keys(&self) -> Result<Vec<SshKeyRecord>, String> {
        Ok(self.keys.iter().map(|r| r.clone()).collect())
    }
}

pub struct RateLimiter {
    attempts: DashMap<String, Vec<Instant>>,
    bans: DashMap<String, Instant>,
    max_attempts: u32,
    window: Duration,
    ban_duration: Duration,
}

impl RateLimiter {
    pub fn new(max_attempts: u32, window: Duration, ban_duration: Duration) -> Self {
        Self {
            attempts: DashMap::new(),
            bans: DashMap::new(),
            max_attempts,
            window,
            ban_duration,
        }
    }

    pub fn check(&self, ip: &str) -> bool {
        if let Some(banned_at) = self.bans.get(ip) {
            if banned_at.elapsed() < self.ban_duration {
                return false;
            }
            self.bans.remove(ip);
        }

        let now = Instant::now();
        if let Some(mut entries) = self.attempts.get_mut(ip) {
            entries.retain(|t| now.duration_since(*t) < self.window);
            return entries.len() < self.max_attempts as usize;
        }
        true
    }

    pub fn record_failure(&self, ip: &str) {
        let now = Instant::now();
        let mut entries = self.attempts.entry(ip.to_string()).or_default();
        entries.retain(|t| now.duration_since(*t) < self.window);
        entries.push(now);

        if entries.len() >= self.max_attempts as usize {
            self.bans.insert(ip.to_string(), now);
        }
    }

    pub fn record_success(&self, ip: &str) {
        self.attempts.remove(ip);
    }

    pub fn is_banned(&self, ip: &str) -> bool {
        if let Some(banned_at) = self.bans.get(ip) {
            if banned_at.elapsed() < self.ban_duration {
                return true;
            }
            self.bans.remove(ip);
        }
        false
    }
}

pub fn fingerprint_sha256(key_data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key_data);
    let hash = hasher.finalize();
    let hex = hex::encode(hash);
    format!("SHA256:{}", base64_encode_fingerprint(&hex))
}

fn base64_encode_fingerprint(hex_str: &str) -> String {
    let bytes: Vec<u8> = (0..hex_str.len())
        .step_by(2)
        .filter_map(|i| {
            u8::from_str_radix(&hex_str[i..i.saturating_add(2).min(hex_str.len())], 16).ok()
        })
        .collect();
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD_NO_PAD, &bytes)
}

pub struct SshAuthService {
    pub store: Arc<dyn SshKeyStore>,
    pub rate_limiter: RateLimiter,
}

impl SshAuthService {
    pub fn new(store: Arc<dyn SshKeyStore>, rate_limiter: RateLimiter) -> Self {
        Self {
            store,
            rate_limiter,
        }
    }

    pub fn authenticate(&self, fingerprint: &str, ip: &str) -> Result<Option<SshKeyRecord>, String> {
        if !self.rate_limiter.check(ip) {
            return Err("too many authentication failures".to_string());
        }

        match self.store.lookup_by_fingerprint(fingerprint) {
            Ok(Some(record)) => {
                self.rate_limiter.record_success(ip);
                Ok(Some(record))
            }
            Ok(None) => {
                self.rate_limiter.record_failure(ip);
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key_record(id: &str, user_id: &str, fingerprint: &str) -> SshKeyRecord {
        SshKeyRecord {
            id: Uuid::parse_str(id).unwrap(),
            user_id: Uuid::parse_str(user_id).unwrap(),
            key_type: "ssh-ed25519".to_string(),
            public_key: "AAAAC3NzaC1lZDI1NTE5AAAAI".to_string(),
            fingerprint: fingerprint.to_string(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_ssh_key_type_identify() {
        assert_eq!(SshKeyType::identify("ssh-ed25519"), SshKeyType::Ed25519);
        assert_eq!(
            SshKeyType::identify("ecdsa-sha2-nistp256"),
            SshKeyType::EcdsaP256
        );
        assert_eq!(SshKeyType::identify("rsa"), SshKeyType::Rsa4096);
        assert_eq!(
            SshKeyType::identify("ssh-rsa"),
            SshKeyType::Unknown("ssh-rsa".to_string())
        );
    }

    #[test]
    fn test_ssh_key_type_as_str() {
        assert_eq!(SshKeyType::Ed25519.as_str(), "ssh-ed25519");
        assert_eq!(SshKeyType::EcdsaP256.as_str(), "ecdsa-sha2-nistp256");
        assert_eq!(SshKeyType::Rsa4096.as_str(), "rsa");
        assert_eq!(
            SshKeyType::Unknown("custom".to_string()).as_str(),
            "custom"
        );
    }

    #[test]
    fn test_in_memory_store_add_and_lookup() {
        let store = InMemorySshKeyStore::new();
        let record = make_key_record(
            "00000000-0000-0000-0000-000000000001",
            "00000000-0000-0000-0000-0000000000a0",
            "SHA256:abc123",
        );

        store.add_key(record.clone()).unwrap();

        let found = store
            .lookup_by_fingerprint("SHA256:abc123")
            .unwrap()
            .unwrap();
        assert_eq!(found.id, record.id);
        assert_eq!(found.user_id, record.user_id);
    }

    #[test]
    fn test_in_memory_store_duplicate_fingerprint_rejected() {
        let store = InMemorySshKeyStore::new();
        let record = make_key_record(
            "00000000-0000-0000-0000-000000000001",
            "00000000-0000-0000-0000-0000000000a0",
            "SHA256:abc123",
        );
        store.add_key(record).unwrap();

        let record2 = make_key_record(
            "00000000-0000-0000-0000-000000000002",
            "00000000-0000-0000-0000-0000000000a0",
            "SHA256:abc123",
        );
        assert!(store.add_key(record2).is_err());
    }

    #[test]
    fn test_in_memory_store_lookup_by_user() {
        let store = InMemorySshKeyStore::new();
        let user_id = Uuid::parse_str("00000000-0000-0000-0000-0000000000a0").unwrap();
        let r1 = make_key_record(
            "00000000-0000-0000-0000-000000000001",
            "00000000-0000-0000-0000-0000000000a0",
            "SHA256:fp1",
        );
        let r2 = make_key_record(
            "00000000-0000-0000-0000-000000000002",
            "00000000-0000-0000-0000-0000000000a0",
            "SHA256:fp2",
        );
        store.add_key(r1).unwrap();
        store.add_key(r2).unwrap();

        let keys = store.lookup_by_user(user_id).unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_in_memory_store_remove_key() {
        let store = InMemorySshKeyStore::new();
        let id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let record = make_key_record(
            "00000000-0000-0000-0000-000000000001",
            "00000000-0000-0000-0000-0000000000a0",
            "SHA256:fp1",
        );
        store.add_key(record).unwrap();

        assert!(store.remove_key(id).unwrap());
        assert!(store.lookup_by_fingerprint("SHA256:fp1").unwrap().is_none());
        assert!(!store.remove_key(id).unwrap());
    }

    #[test]
    fn test_in_memory_store_list_keys() {
        let store = InMemorySshKeyStore::new();
        let r1 = make_key_record(
            "00000000-0000-0000-0000-000000000001",
            "00000000-0000-0000-0000-0000000000a0",
            "SHA256:fp1",
        );
        let r2 = make_key_record(
            "00000000-0000-0000-0000-000000000002",
            "00000000-0000-0000-0000-0000000000b0",
            "SHA256:fp2",
        );
        store.add_key(r1).unwrap();
        store.add_key(r2).unwrap();

        assert_eq!(store.list_keys().unwrap().len(), 2);
    }

    #[test]
    fn test_in_memory_store_missing_fingerprint() {
        let store = InMemorySshKeyStore::new();
        assert!(store
            .lookup_by_fingerprint("SHA256:nonexistent")
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_rate_limiter_allows_initial() {
        let rl = RateLimiter::new(3, Duration::from_secs(60), Duration::from_secs(300));
        assert!(rl.check("1.2.3.4"));
    }

    #[test]
    fn test_rate_limiter_blocks_after_max() {
        let rl = RateLimiter::new(2, Duration::from_secs(60), Duration::from_secs(300));
        rl.record_failure("1.2.3.4");
        assert!(rl.check("1.2.3.4"));
        rl.record_failure("1.2.3.4");
        assert!(!rl.check("1.2.3.4"));
    }

    #[test]
    fn test_rate_limiter_success_clears() {
        let rl = RateLimiter::new(3, Duration::from_secs(60), Duration::from_secs(300));
        rl.record_failure("1.2.3.4");
        rl.record_failure("1.2.3.4");
        rl.record_success("1.2.3.4");
        assert!(rl.check("1.2.3.4"));
        rl.record_failure("1.2.3.4");
        assert!(rl.check("1.2.3.4"));
    }

    #[test]
    fn test_rate_limiter_different_ips() {
        let rl = RateLimiter::new(1, Duration::from_secs(60), Duration::from_secs(300));
        rl.record_failure("1.2.3.4");
        assert!(!rl.check("1.2.3.4"));
        assert!(rl.check("5.6.7.8"));
    }

    #[test]
    fn test_rate_limiter_ban_expires() {
        let rl = RateLimiter::new(1, Duration::from_secs(0), Duration::from_millis(10));
        rl.record_failure("1.2.3.4");
        assert!(!rl.check("1.2.3.4"));
        std::thread::sleep(Duration::from_millis(20));
        assert!(rl.check("1.2.3.4"));
    }

    #[test]
    fn test_rate_limiter_is_banned() {
        let rl = RateLimiter::new(1, Duration::from_secs(60), Duration::from_secs(300));
        rl.record_failure("1.2.3.4");
        assert!(rl.is_banned("1.2.3.4"));
        assert!(!rl.is_banned("5.6.7.8"));
    }

    #[test]
    fn test_fingerprint_sha256_deterministic() {
        let fp1 = fingerprint_sha256(b"test-key-data");
        let fp2 = fingerprint_sha256(b"test-key-data");
        assert_eq!(fp1, fp2);
        assert!(fp1.starts_with("SHA256:"));
    }

    #[test]
    fn test_fingerprint_sha256_different_keys() {
        let fp1 = fingerprint_sha256(b"key-one");
        let fp2 = fingerprint_sha256(b"key-two");
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_authenticate_success() {
        let store = Arc::new(InMemorySshKeyStore::new());
        let rl = RateLimiter::new(5, Duration::from_secs(60), Duration::from_secs(300));
        let record = make_key_record(
            "00000000-0000-0000-0000-000000000001",
            "00000000-0000-0000-0000-0000000000a0",
            "SHA256:fp1",
        );
        store.add_key(record.clone()).unwrap();

        let service = SshAuthService::new(store, rl);
        let result = service.authenticate("SHA256:fp1", "1.2.3.4").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, record.id);
    }

    #[test]
    fn test_authenticate_unknown_key() {
        let store = Arc::new(InMemorySshKeyStore::new());
        let rl = RateLimiter::new(5, Duration::from_secs(60), Duration::from_secs(300));
        let service = SshAuthService::new(store, rl);

        let result = service.authenticate("SHA256:nonexistent", "1.2.3.4").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_authenticate_rate_limited() {
        let store = Arc::new(InMemorySshKeyStore::new());
        let rl = RateLimiter::new(2, Duration::from_secs(60), Duration::from_secs(300));
        let service = SshAuthService::new(store, rl);

        assert!(service
            .authenticate("SHA256:nonexistent", "1.2.3.4")
            .unwrap()
            .is_none());
        assert!(service
            .authenticate("SHA256:nonexistent", "1.2.3.4")
            .unwrap()
            .is_none());
        assert!(service
            .authenticate("SHA256:nonexistent", "1.2.3.4")
            .is_err());
    }
}
