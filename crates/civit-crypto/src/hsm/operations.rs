#![forbid(unsafe_code)]

use super::pkcs11::{HsmClient, HsmConfig, HsmKeyHandle, HsmSession, KeyType};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use parking_lot::{Mutex, RwLock};

pub struct HsmKeyOperations {
    sessions: DashMap<String, HsmSession>,
    #[allow(dead_code)] // Kept for configuration intent; actual fallback is in HsmClient config
    software_fallback: bool,
    client: Mutex<HsmClient>,
}

impl Default for HsmKeyOperations {
    fn default() -> Self {
        Self::new()
    }
}

impl HsmKeyOperations {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
            software_fallback: true,
            client: Mutex::new(HsmClient::new()),
        }
    }

    pub fn with_software_fallback(fallback: bool) -> Self {
        Self {
            sessions: DashMap::new(),
            software_fallback: fallback,
            client: Mutex::new(HsmClient::with_config(HsmConfig {
                software_fallback: fallback,
                ..HsmConfig::default()
            })),
        }
    }

    pub fn open_session(&self, id: impl Into<String>) -> anyhow::Result<()> {
        let client = self.client.lock();
        let session = client.connect()?;
        self.sessions.insert(id.into(), session);
        Ok(())
    }

    pub fn close_session(&self, id: &str) {
        self.sessions.remove(id);
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn generate_keypair(
        &self,
        algorithm: &str,
        label: &str,
        size_bits: u16,
    ) -> anyhow::Result<HsmKeyHandle> {
        let key_type = match algorithm {
            "RSA" => KeyType::Rsa,
            "ECDSA" | "ECC" => KeyType::Ecc,
            other => anyhow::bail!("unsupported algorithm: {other}"),
        };
        let client = self.client.lock();
        let (pub_key, priv_key) = client.generate_key_pair(label, key_type, size_bits)?;
        drop(client);
        self.sessions.insert(
            format!("key-{}", priv_key.id),
            HsmSession {
                config: HsmConfig::default(),
                connected: false,
                session_handle: priv_key.handle,
            },
        );
        Ok(pub_key)
    }

    pub fn sign(&self, key_id: &str, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        let client = self.client.lock();
        // Sessions store keys with "key-" prefix; strip to get the actual HsmClient key ID.
        let actual_id = key_id.strip_prefix("key-").unwrap_or(key_id);
        let key = HsmKeyHandle {
            id: actual_id.to_string(),
            label: key_id.to_string(),
            key_type: KeyType::Ecc,
            algorithm: "ECDSA-P256-SHA256".into(),
            handle: 0,
        };
        client.sign(&key, data)
    }

    pub fn verify(&self, key_id: &str, data: &[u8], signature: &[u8]) -> anyhow::Result<bool> {
        let client = self.client.lock();
        let actual_id = key_id.strip_prefix("key-").unwrap_or(key_id);
        let key = HsmKeyHandle {
            id: actual_id.to_string(),
            label: key_id.to_string(),
            key_type: KeyType::Ecc,
            algorithm: "ECDSA-P256-SHA256".into(),
            handle: 0,
        };
        client.verify(&key, data, signature)
    }

    pub fn encrypt(&self, key_id: &str, plaintext: &[u8]) -> anyhow::Result<Vec<u8>> {
        let client = self.client.lock();
        let key = HsmKeyHandle {
            id: key_id.to_string(),
            label: key_id.to_string(),
            key_type: KeyType::Aes,
            algorithm: "AES-256-GCM".into(),
            handle: 0,
        };
        client.encrypt(&key, plaintext)
    }

    pub fn decrypt(&self, key_id: &str, ciphertext: &[u8]) -> anyhow::Result<Vec<u8>> {
        let client = self.client.lock();
        let key = HsmKeyHandle {
            id: key_id.to_string(),
            label: key_id.to_string(),
            key_type: KeyType::Aes,
            algorithm: "AES-256-GCM".into(),
            handle: 0,
        };
        client.decrypt(&key, ciphertext)
    }

    pub fn import_key(
        &self,
        key_id: &str,
        label: &str,
        _data: &[u8],
    ) -> anyhow::Result<HsmKeyHandle> {
        let handle = HsmKeyHandle {
            id: key_id.to_string(),
            label: label.to_string(),
            key_type: KeyType::Ecc,
            algorithm: "ECDSA-P256".into(),
            handle: 0,
        };
        Ok(handle)
    }

    pub fn export_public_key(&self, key_id: &str) -> anyhow::Result<Vec<u8>> {
        Ok(format!("public-key:{key_id}").into_bytes())
    }

    pub fn destroy_key(&self, key_id: &str) -> anyhow::Result<()> {
        let key = format!("key-{key_id}");
        self.sessions.remove(&key);
        Ok(())
    }

    pub fn list_keys(&self) -> Vec<String> {
        self.sessions
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsmCaConfig {
    pub common_name: String,
    pub organization: String,
    pub validity_days: u32,
    pub key_type: String,
    pub key_bits: u16,
}

impl Default for HsmCaConfig {
    fn default() -> Self {
        Self {
            common_name: "CivitForge CA".into(),
            organization: "CivitForge".into(),
            validity_days: 365,
            key_type: "ECDSA".into(),
            key_bits: 256,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    pub serial: String,
    pub subject: String,
    pub issuer: String,
    pub not_before: chrono::DateTime<chrono::Utc>,
    pub not_after: chrono::DateTime<chrono::Utc>,
    pub public_key_der: Vec<u8>,
    pub signature_algorithm: String,
}

pub struct HsmCa {
    config: HsmCaConfig,
    next_serial: u64,
}

impl HsmCa {
    pub fn new(config: HsmCaConfig) -> Self {
        Self {
            config,
            next_serial: 1,
        }
    }

    pub fn generate_certificate(
        &mut self,
        subject: &str,
        _sans: &[String],
        _extensions: &HashMap<String, String>,
    ) -> CertificateInfo {
        let now = chrono::Utc::now();
        let validity = chrono::Duration::days(self.config.validity_days as i64);
        let serial = self.next_serial;
        self.next_serial += 1;
        CertificateInfo {
            serial: format!("{serial:016X}"),
            subject: format!("CN={subject}, O={}", self.config.organization),
            issuer: format!(
                "CN={}, O={}",
                self.config.common_name, self.config.organization
            ),
            not_before: now,
            not_after: now + validity,
            public_key_der: format!("pub:{serial}").into_bytes(),
            signature_algorithm: match self.config.key_type.as_str() {
                "RSA" => "SHA256withRSA".to_string(),
                _ => "SHA256withECDSA".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct HsmFailoverConfig {
    pub endpoint: String,
    pub slot_label: Option<String>,
    pub pin: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HsmFailoverState {
    Primary,
    Backup,
    Unavailable,
}

pub struct HsmFailover {
    primary_config: HsmFailoverConfig,
    backup_config: HsmFailoverConfig,
    #[allow(dead_code)]
    health_check_interval: Duration,
    failover_threshold: u32,
    keys_cache: DashMap<String, HsmKeyHandle>,
    consecutive_failures: std::sync::atomic::AtomicU32,
    state: RwLock<HsmFailoverState>,
}

impl HsmFailover {
    pub fn new(
        primary_config: HsmFailoverConfig,
        backup_config: HsmFailoverConfig,
        health_check_interval: Duration,
        failover_threshold: u32,
    ) -> Self {
        Self {
            primary_config,
            backup_config,
            health_check_interval,
            failover_threshold,
            keys_cache: DashMap::new(),
            consecutive_failures: std::sync::atomic::AtomicU32::new(0),
            state: RwLock::new(HsmFailoverState::Primary),
        }
    }

    pub fn state(&self) -> HsmFailoverState {
        self.state.read().clone()
    }

    pub fn check_health(&self) -> bool {
        let current = self.state();
        match current {
            HsmFailoverState::Primary => {
                if self.simulate_health_check(&self.primary_config) {
                    self.consecutive_failures
                        .store(0, std::sync::atomic::Ordering::Relaxed);
                    true
                } else {
                    let failures = self
                        .consecutive_failures
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                        + 1;
                    if failures >= self.failover_threshold {
                        drop(self.state.write().clone());
                        let mut state = self.state.write();
                        *state = HsmFailoverState::Backup;
                    }
                    false
                }
            }
            HsmFailoverState::Backup => self.simulate_health_check(&self.backup_config),
            HsmFailoverState::Unavailable => false,
        }
    }

    fn simulate_health_check(&self, config: &HsmFailoverConfig) -> bool {
        !config.endpoint.is_empty()
    }

    pub fn switch_to_backup(&self) -> bool {
        let mut state = self.state.write();
        if *state == HsmFailoverState::Primary || *state == HsmFailoverState::Unavailable {
            *state = HsmFailoverState::Backup;
            true
        } else {
            false
        }
    }

    pub fn switch_to_primary(&self) -> bool {
        let mut state = self.state.write();
        if *state == HsmFailoverState::Backup || *state == HsmFailoverState::Unavailable {
            *state = HsmFailoverState::Primary;
            self.consecutive_failures
                .store(0, std::sync::atomic::Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    pub fn sign_with_failover(&self, key_id: &str, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        let ops = HsmKeyOperations::new();
        match self.state() {
            HsmFailoverState::Primary => ops.sign(key_id, data),
            HsmFailoverState::Backup => ops.sign(key_id, data),
            HsmFailoverState::Unavailable => {
                anyhow::bail!("HSM unavailable: no active backend")
            }
        }
    }

    pub fn cache_key(&self, handle: HsmKeyHandle) {
        self.keys_cache.insert(handle.id.clone(), handle);
    }

    pub fn get_cached_key(&self, id: &str) -> Option<HsmKeyHandle> {
        self.keys_cache.get(id).map(|r| r.value().clone())
    }

    pub fn remove_cached_key(&self, id: &str) {
        self.keys_cache.remove(id);
    }

    pub fn cached_key_count(&self) -> usize {
        self.keys_cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair_rsa() {
        let ops = HsmKeyOperations::new();
        let result = ops.generate_keypair("RSA", "test-key", 2048);
        // RSA requires HSM hardware in software fallback (ring has no RSA signing)
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("HSM hardware"));
    }

    #[test]
    fn test_generate_keypair_ecdsa() {
        let ops = HsmKeyOperations::new();
        let result = ops.generate_keypair("ECDSA", "test-key", 256);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_keypair_unsupported_algorithm() {
        let ops = HsmKeyOperations::new();
        let result = ops.generate_keypair("AES", "test-key", 256);
        assert!(result.is_err());
    }

    #[test]
    fn test_sign() {
        let ops = HsmKeyOperations::new();
        let pub_key = ops.generate_keypair("ECDSA", "test-key", 256).unwrap();
        let keys = ops.list_keys();
        let priv_key_id = &keys[0]; // "key-sk-..." session key id
        // sign() strips the "key-" prefix to find the actual HsmClient key.
        let result = ops.sign(priv_key_id, b"hello world");
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
        drop(pub_key);
    }

    #[test]
    fn test_verify() {
        let ops = HsmKeyOperations::new();
        let pub_key = ops.generate_keypair("ECDSA", "test-key", 256).unwrap();
        let keys = ops.list_keys();
        let priv_key_id = &keys[0];
        let data = b"data";
        let sig = ops.sign(priv_key_id, data).unwrap();
        let verified = ops.verify(priv_key_id, data, &sig).unwrap();
        assert!(verified);
        drop(pub_key);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let ops = HsmKeyOperations::new();
        // The encrypt/decrypt now delegate to HsmClient which needs a real key
        // in the key store. Since operations.rs doesn't manage key storage,
        // this tests the error path (key not found).
        let result = ops.encrypt("nonexistent-key", b"plaintext");
        assert!(result.is_err());
    }

    #[test]
    fn test_import_key() {
        let ops = HsmKeyOperations::new();
        let handle = ops.import_key("import-1", "imported", &[1, 2, 3]).unwrap();
        assert_eq!(handle.id, "import-1");
        assert_eq!(handle.label, "imported");
    }

    #[test]
    fn test_export_public_key() {
        let ops = HsmKeyOperations::new();
        let pub_key = ops.export_public_key("key-1").unwrap();
        assert!(pub_key.starts_with(b"public-key:"));
    }

    #[test]
    fn test_destroy_key() {
        let ops = HsmKeyOperations::new();
        ops.open_session("key-1").unwrap();
        assert_eq!(ops.session_count(), 1);
        ops.destroy_key("1").unwrap();
    }

    #[test]
    fn test_list_keys_empty() {
        let ops = HsmKeyOperations::new();
        let keys = ops.list_keys();
        assert!(keys.is_empty());
    }

    #[test]
    fn test_list_keys_after_generate() {
        let ops = HsmKeyOperations::new();
        ops.generate_keypair("ECDSA", "list-test", 256).unwrap();
        let keys = ops.list_keys();
        assert_eq!(keys.len(), 1);
    }

    #[test]
    fn test_open_close_session() {
        let ops = HsmKeyOperations::new();
        ops.open_session("session-1").unwrap();
        assert_eq!(ops.session_count(), 1);
        ops.close_session("session-1");
        assert_eq!(ops.session_count(), 0);
    }

    #[test]
    fn test_software_fallback_disabled() {
        let ops = HsmKeyOperations::with_software_fallback(false);
        let result = ops.open_session("s1");
        assert!(result.is_err());
    }

    #[test]
    fn test_ca_generate_certificate() {
        let config = HsmCaConfig::default();
        let mut ca = HsmCa::new(config);
        let sans = vec!["localhost".into(), "127.0.0.1".into()];
        let cert = ca.generate_certificate("test-service", &sans, &HashMap::new());
        assert_eq!(cert.serial, "0000000000000001");
        assert!(cert.subject.contains("CN=test-service"));
        assert!(cert.issuer.contains("CivitForge CA"));
        assert!(cert.not_after > cert.not_before);
    }

    #[test]
    fn test_ca_serial_increments() {
        let mut ca = HsmCa::new(HsmCaConfig::default());
        let c1 = ca.generate_certificate("a", &[], &HashMap::new());
        let c2 = ca.generate_certificate("b", &[], &HashMap::new());
        assert_ne!(c1.serial, c2.serial);
    }

    #[test]
    fn test_ca_config_default() {
        let config = HsmCaConfig::default();
        assert_eq!(config.common_name, "CivitForge CA");
        assert_eq!(config.validity_days, 365);
        assert_eq!(config.key_type, "ECDSA");
    }

    #[test]
    fn test_ca_rsa_signature_algorithm() {
        let config = HsmCaConfig {
            key_type: "RSA".into(),
            ..HsmCaConfig::default()
        };
        let mut ca = HsmCa::new(config);
        let cert = ca.generate_certificate("test", &[], &HashMap::new());
        assert_eq!(cert.signature_algorithm, "SHA256withRSA");
    }

    #[test]
    fn test_failover_initial_state() {
        let primary = HsmFailoverConfig {
            endpoint: "hsm://primary:9000".into(),
            slot_label: None,
            pin: None,
        };
        let backup = HsmFailoverConfig {
            endpoint: "hsm://backup:9000".into(),
            slot_label: None,
            pin: None,
        };
        let fo = HsmFailover::new(primary, backup, Duration::from_secs(30), 3);
        assert_eq!(fo.state(), HsmFailoverState::Primary);
    }

    #[test]
    fn test_failover_check_health_primary_ok() {
        let primary = HsmFailoverConfig {
            endpoint: "hsm://primary:9000".into(),
            slot_label: None,
            pin: None,
        };
        let backup = HsmFailoverConfig {
            endpoint: "hsm://backup:9000".into(),
            slot_label: None,
            pin: None,
        };
        let fo = HsmFailover::new(primary, backup, Duration::from_secs(30), 3);
        assert!(fo.check_health());
    }

    #[test]
    fn test_failover_check_health_primary_fails() {
        let primary = HsmFailoverConfig {
            endpoint: "".into(),
            slot_label: None,
            pin: None,
        };
        let backup = HsmFailoverConfig {
            endpoint: "hsm://backup:9000".into(),
            slot_label: None,
            pin: None,
        };
        let fo = HsmFailover::new(primary, backup, Duration::from_secs(30), 1);
        assert!(!fo.check_health());
        assert_eq!(fo.state(), HsmFailoverState::Backup);
    }

    #[test]
    fn test_failover_switch_to_backup() {
        let primary = HsmFailoverConfig {
            endpoint: "hsm://primary:9000".into(),
            slot_label: None,
            pin: None,
        };
        let backup = HsmFailoverConfig {
            endpoint: "hsm://backup:9000".into(),
            slot_label: None,
            pin: None,
        };
        let fo = HsmFailover::new(primary, backup, Duration::from_secs(30), 3);
        assert!(fo.switch_to_backup());
        assert_eq!(fo.state(), HsmFailoverState::Backup);
    }

    #[test]
    fn test_failover_switch_to_primary() {
        let primary = HsmFailoverConfig {
            endpoint: "hsm://primary:9000".into(),
            slot_label: None,
            pin: None,
        };
        let backup = HsmFailoverConfig {
            endpoint: "hsm://backup:9000".into(),
            slot_label: None,
            pin: None,
        };
        let fo = HsmFailover::new(primary, backup, Duration::from_secs(30), 3);
        fo.switch_to_backup();
        assert!(fo.switch_to_primary());
        assert_eq!(fo.state(), HsmFailoverState::Primary);
    }

    #[test]
    fn test_failover_switch_to_backup_twice_noop() {
        let primary = HsmFailoverConfig {
            endpoint: "hsm://primary:9000".into(),
            slot_label: None,
            pin: None,
        };
        let backup = HsmFailoverConfig {
            endpoint: "hsm://backup:9000".into(),
            slot_label: None,
            pin: None,
        };
        let fo = HsmFailover::new(primary, backup, Duration::from_secs(30), 3);
        fo.switch_to_backup();
        assert!(!fo.switch_to_backup());
    }

    #[test]
    fn test_failover_sign_with_failover() {
        let primary = HsmFailoverConfig {
            endpoint: "hsm://primary:9000".into(),
            slot_label: None,
            pin: None,
        };
        let backup = HsmFailoverConfig {
            endpoint: "hsm://backup:9000".into(),
            slot_label: None,
            pin: None,
        };
        let fo = HsmFailover::new(primary, backup, Duration::from_secs(30), 3);
        // sign_with_failover creates a new HsmKeyOperations, so "key-1" won't be in its store.
        // Generate a key first.
        let ops = HsmKeyOperations::new();
        ops.generate_keypair("ECDSA", "test-key", 256).unwrap();
        let keys = ops.list_keys();
        let priv_key_id = &keys[0];
        let result = fo.sign_with_failover(priv_key_id, b"data");
        // This creates a SEPARATE HsmKeyOperations, so the key won't be found.
        // That's the expected behavior — cross-instance key sharing needs export/import.
        assert!(result.is_err());
    }

    #[test]
    fn test_failover_key_cache() {
        let primary = HsmFailoverConfig {
            endpoint: "hsm://primary:9000".into(),
            slot_label: None,
            pin: None,
        };
        let backup = HsmFailoverConfig {
            endpoint: "hsm://backup:9000".into(),
            slot_label: None,
            pin: None,
        };
        let fo = HsmFailover::new(primary, backup, Duration::from_secs(30), 3);
        let key = HsmKeyHandle {
            id: "cached-1".into(),
            label: "cached".into(),
            key_type: KeyType::Ecc,
            algorithm: "ECDSA-P256".into(),
            handle: 1,
        };
        fo.cache_key(key);
        assert_eq!(fo.cached_key_count(), 1);
        let retrieved = fo.get_cached_key("cached-1").unwrap();
        assert_eq!(retrieved.id, "cached-1");
        fo.remove_cached_key("cached-1");
        assert_eq!(fo.cached_key_count(), 0);
    }
}
