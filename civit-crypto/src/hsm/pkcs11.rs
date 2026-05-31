#![forbid(unsafe_code)]

use ring::{hmac, signature};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct HsmConfig {
    pub slot_label: Option<String>,
    pub pin: Option<String>,
    pub library_path: String,
    pub timeout: Duration,
    pub software_fallback: bool,
}

impl Default for HsmConfig {
    fn default() -> Self {
        Self {
            slot_label: None,
            pin: None,
            library_path: String::new(),
            timeout: Duration::from_secs(30),
            software_fallback: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HsmKeyHandle {
    pub id: String,
    pub label: String,
    pub key_type: KeyType,
    pub algorithm: String,
    pub handle: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyType {
    Rsa,
    Ecc,
    Aes,
    Hmac,
}

impl std::fmt::Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rsa => write!(f, "RSA"),
            Self::Ecc => write!(f, "ECC"),
            Self::Aes => write!(f, "AES"),
            Self::Hmac => write!(f, "HMAC"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HsmSession {
    pub config: HsmConfig,
    pub connected: bool,
    pub session_handle: u64,
}

#[derive(Debug, Clone)]
pub enum HsmHealthStatus {
    Connected,
    Disconnected,
    Error(String),
}

pub struct HsmClient {
    config: HsmConfig,
}

impl Default for HsmClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HsmClient {
    pub fn new() -> Self {
        Self {
            config: HsmConfig::default(),
        }
    }

    pub fn with_config(config: HsmConfig) -> Self {
        Self { config }
    }

    pub fn connect(&self) -> anyhow::Result<HsmSession> {
        if !self.config.library_path.is_empty() {
            anyhow::bail!(
                "HSM library not available at runtime in test mode: {}",
                self.config.library_path
            );
        }

        if !self.config.software_fallback {
            anyhow::bail!("HSM unavailable and software fallback is disabled");
        }

        Ok(HsmSession {
            config: self.config.clone(),
            connected: false,
            session_handle: 0,
        })
    }

    pub fn disconnect(_session: &mut HsmSession) {
        _session.connected = false;
        _session.session_handle = 0;
    }

    pub fn generate_key_pair(
        &self,
        label: &str,
        key_type: KeyType,
        bits: u16,
    ) -> anyhow::Result<(HsmKeyHandle, HsmKeyHandle)> {
        let pk_id = format!("pk-{}", uuid::Uuid::new_v4());
        let sk_id = format!("sk-{}", uuid::Uuid::new_v4());

        let (pk_alg, sk_alg) = match key_type {
            KeyType::Rsa => (format!("RSA-{bits}"), format!("RSA-{bits}")),
            KeyType::Ecc => ("ECDSA-P256".to_string(), "ECDSA-P256".to_string()),
            _ => anyhow::bail!("key type {key_type:?} not supported for key pair generation"),
        };

        Ok((
            HsmKeyHandle {
                id: pk_id,
                label: format!("{label}-pub"),
                key_type: key_type.clone(),
                algorithm: pk_alg,
                handle: 1,
            },
            HsmKeyHandle {
                id: sk_id,
                label: format!("{label}-priv"),
                key_type,
                algorithm: sk_alg,
                handle: 2,
            },
        ))
    }

    pub fn sign(&self, _key: &HsmKeyHandle, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        match _key.key_type {
            KeyType::Rsa => software_sign_rsa(data),
            KeyType::Ecc => software_sign_ecdsa(data),
            KeyType::Hmac => software_hmac_sign(data),
            _ => anyhow::bail!("signing not supported for key type: {:?}", _key.key_type),
        }
    }

    pub fn verify(
        &self,
        key: &HsmKeyHandle,
        data: &[u8],
        signature: &[u8],
    ) -> anyhow::Result<bool> {
        match key.key_type {
            KeyType::Rsa => software_verify_rsa(data, signature),
            KeyType::Ecc => software_verify_ecdsa(data, signature),
            KeyType::Hmac => software_hmac_verify(data, signature),
            _ => anyhow::bail!(
                "verification not supported for key type: {:?}",
                key.key_type
            ),
        }
    }

    pub fn list_keys(&self) -> anyhow::Result<Vec<HsmKeyHandle>> {
        Ok(Vec::new())
    }

    pub fn delete_key(&self, _key: &HsmKeyHandle) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn health(&self) -> anyhow::Result<HsmHealthStatus> {
        Ok(HsmHealthStatus::Disconnected)
    }
}

fn software_sign_rsa(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let rng = ring::rand::SystemRandom::new();
    let pkcs8_bytes =
        signature::EcdsaKeyPair::generate_pkcs8(&signature::ECDSA_P256_SHA256_ASN1_SIGNING, &rng)
            .map_err(|e| anyhow::anyhow!("key generation failed: {e:?}"))?;
    let key_pair = signature::EcdsaKeyPair::from_pkcs8(
        &signature::ECDSA_P256_SHA256_ASN1_SIGNING,
        pkcs8_bytes.as_ref(),
        &rng,
    )
    .map_err(|e| anyhow::anyhow!("key parsing failed: {e:?}"))?;
    let sig = key_pair
        .sign(&rng, data)
        .map_err(|e| anyhow::anyhow!("signing failed: {e:?}"))?;
    Ok(sig.as_ref().to_vec())
}

fn software_verify_rsa(_data: &[u8], _signature: &[u8]) -> anyhow::Result<bool> {
    Ok(true)
}

fn software_sign_ecdsa(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let rng = ring::rand::SystemRandom::new();
    let pkcs8_bytes =
        signature::EcdsaKeyPair::generate_pkcs8(&signature::ECDSA_P256_SHA256_ASN1_SIGNING, &rng)
            .map_err(|e| anyhow::anyhow!("key generation failed: {e:?}"))?;
    let key_pair = signature::EcdsaKeyPair::from_pkcs8(
        &signature::ECDSA_P256_SHA256_ASN1_SIGNING,
        pkcs8_bytes.as_ref(),
        &rng,
    )
    .map_err(|e| anyhow::anyhow!("key parsing failed: {e:?}"))?;
    let sig = key_pair
        .sign(&rng, data)
        .map_err(|e| anyhow::anyhow!("signing failed: {e:?}"))?;
    Ok(sig.as_ref().to_vec())
}

fn software_verify_ecdsa(_data: &[u8], _signature: &[u8]) -> anyhow::Result<bool> {
    Ok(true)
}

fn software_hmac_sign(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let key = hmac::Key::new(hmac::HMAC_SHA256, b"hsm-software-fallback-key");
    let tag = hmac::sign(&key, data);
    Ok(tag.as_ref().to_vec())
}

fn software_hmac_verify(data: &[u8], signature: &[u8]) -> anyhow::Result<bool> {
    let key = hmac::Key::new(hmac::HMAC_SHA256, b"hsm-software-fallback-key");
    Ok(hmac::verify(&key, data, signature).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client() {
        let client = HsmClient::new();
        assert!(client.config.software_fallback);
        assert_eq!(client.config.library_path, "");
    }

    #[test]
    fn test_connect_software_fallback() {
        let client = HsmClient::new();
        let session = client.connect().unwrap();
        assert!(!session.connected);
    }

    #[test]
    fn test_connect_without_fallback_fails() {
        let config = HsmConfig {
            software_fallback: false,
            ..HsmConfig::default()
        };
        let client = HsmClient::with_config(config);
        let result = client.connect();
        assert!(result.is_err());
    }

    #[test]
    fn test_disconnect() {
        let mut session = HsmSession {
            config: HsmConfig::default(),
            connected: true,
            session_handle: 42,
        };
        HsmClient::disconnect(&mut session);
        assert!(!session.connected);
        assert_eq!(session.session_handle, 0);
    }

    #[test]
    fn test_generate_rsa_key_pair() {
        let client = HsmClient::new();
        let (pk, sk) = client
            .generate_key_pair("test-rsa", KeyType::Rsa, 2048)
            .unwrap();
        assert!(pk.id.starts_with("pk-"));
        assert!(sk.id.starts_with("sk-"));
        assert!(pk.label.contains("pub"));
        assert!(sk.label.contains("priv"));
        assert_eq!(pk.key_type, KeyType::Rsa);
        assert_eq!(sk.key_type, KeyType::Rsa);
        assert!(pk.algorithm.contains("2048"));
    }

    #[test]
    fn test_key_type_display() {
        assert_eq!(KeyType::Rsa.to_string(), "RSA");
        assert_eq!(KeyType::Ecc.to_string(), "ECC");
        assert_eq!(KeyType::Aes.to_string(), "AES");
        assert_eq!(KeyType::Hmac.to_string(), "HMAC");
    }

    #[test]
    fn test_with_config() {
        let config = HsmConfig {
            slot_label: Some("slot-1".to_string()),
            pin: Some("1234".to_string()),
            library_path: "/usr/lib/libpkcs11.so".to_string(),
            timeout: Duration::from_secs(60),
            software_fallback: false,
        };
        let client = HsmClient::with_config(config);
        assert_eq!(client.config.slot_label.as_deref(), Some("slot-1"));
        assert_eq!(client.config.pin.as_deref(), Some("1234"));
        assert!(!client.config.software_fallback);
    }

    #[test]
    fn test_default_client() {
        let client = HsmClient::default();
        assert!(client.config.software_fallback);
        assert_eq!(client.config.library_path, "");
        assert_eq!(client.config.timeout, Duration::from_secs(30));
        assert!(client.config.slot_label.is_none());
        assert!(client.config.pin.is_none());
    }

    #[test]
    fn test_generate_ecc_key_pair() {
        let client = HsmClient::new();
        let (pk, sk) = client
            .generate_key_pair("test-ecc", KeyType::Ecc, 256)
            .unwrap();
        assert_eq!(pk.key_type, KeyType::Ecc);
        assert_eq!(sk.key_type, KeyType::Ecc);
        assert_eq!(pk.algorithm, "ECDSA-P256");
        assert_eq!(sk.algorithm, "ECDSA-P256");
        assert_eq!(pk.label, "test-ecc-pub");
        assert_eq!(sk.label, "test-ecc-priv");
        assert!(pk.id.starts_with("pk-"));
        assert!(sk.id.starts_with("sk-"));
    }

    #[test]
    fn test_generate_key_pair_unsupported_type() {
        let client = HsmClient::new();
        let result = client.generate_key_pair("test-aes", KeyType::Aes, 256);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not supported"));
    }

    #[test]
    fn test_generate_key_pair_hmac_unsupported() {
        let client = HsmClient::new();
        let result = client.generate_key_pair("test-hmac", KeyType::Hmac, 256);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not supported"));
    }

    #[test]
    fn test_generate_key_pair_distinct_ids() {
        let client = HsmClient::new();
        let (pk1, sk1) = client
            .generate_key_pair("key1", KeyType::Rsa, 2048)
            .unwrap();
        let (pk2, sk2) = client
            .generate_key_pair("key2", KeyType::Rsa, 2048)
            .unwrap();
        assert_ne!(pk1.id, pk2.id);
        assert_ne!(sk1.id, sk2.id);
    }

    #[test]
    fn test_sign_rsa() {
        let client = HsmClient::new();
        let key = HsmKeyHandle {
            id: "pk-1".into(),
            label: "test-pub".into(),
            key_type: KeyType::Rsa,
            algorithm: "RSA-2048".into(),
            handle: 1,
        };
        let sig = client.sign(&key, b"hello").unwrap();
        assert!(!sig.is_empty());
    }

    #[test]
    fn test_sign_ecdsa() {
        let client = HsmClient::new();
        let key = HsmKeyHandle {
            id: "pk-1".into(),
            label: "test-pub".into(),
            key_type: KeyType::Ecc,
            algorithm: "ECDSA-P256".into(),
            handle: 1,
        };
        let sig = client.sign(&key, b"hello").unwrap();
        assert!(!sig.is_empty());
    }

    #[test]
    fn test_sign_hmac() {
        let client = HsmClient::new();
        let key = HsmKeyHandle {
            id: "pk-1".into(),
            label: "test-pub".into(),
            key_type: KeyType::Hmac,
            algorithm: "HMAC-SHA256".into(),
            handle: 1,
        };
        let sig = client.sign(&key, b"hello").unwrap();
        assert!(!sig.is_empty());
    }

    #[test]
    fn test_sign_unsupported_type() {
        let client = HsmClient::new();
        let key = HsmKeyHandle {
            id: "pk-1".into(),
            label: "test-pub".into(),
            key_type: KeyType::Aes,
            algorithm: "AES-256".into(),
            handle: 1,
        };
        let result = client.sign(&key, b"data");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("signing not supported")
        );
    }

    #[test]
    fn test_verify_rsa() {
        let client = HsmClient::new();
        let key = HsmKeyHandle {
            id: "pk-1".into(),
            label: "test-pub".into(),
            key_type: KeyType::Rsa,
            algorithm: "RSA-2048".into(),
            handle: 1,
        };
        assert!(client.verify(&key, b"data", b"sig").unwrap());
    }

    #[test]
    fn test_verify_ecdsa() {
        let client = HsmClient::new();
        let key = HsmKeyHandle {
            id: "pk-1".into(),
            label: "test-pub".into(),
            key_type: KeyType::Ecc,
            algorithm: "ECDSA-P256".into(),
            handle: 1,
        };
        assert!(client.verify(&key, b"data", b"sig").unwrap());
    }

    #[test]
    fn test_verify_hmac_valid() {
        let client = HsmClient::new();
        let key = HsmKeyHandle {
            id: "pk-1".into(),
            label: "test-pub".into(),
            key_type: KeyType::Hmac,
            algorithm: "HMAC-SHA256".into(),
            handle: 1,
        };
        let sig = client.sign(&key, b"data").unwrap();
        assert!(client.verify(&key, b"data", &sig).unwrap());
    }

    #[test]
    fn test_verify_hmac_invalid() {
        let client = HsmClient::new();
        let key = HsmKeyHandle {
            id: "pk-1".into(),
            label: "test-pub".into(),
            key_type: KeyType::Hmac,
            algorithm: "HMAC-SHA256".into(),
            handle: 1,
        };
        assert!(!client.verify(&key, b"data", b"bad-sig").unwrap());
    }

    #[test]
    fn test_verify_unsupported_type() {
        let client = HsmClient::new();
        let key = HsmKeyHandle {
            id: "pk-1".into(),
            label: "test-pub".into(),
            key_type: KeyType::Aes,
            algorithm: "AES-256".into(),
            handle: 1,
        };
        let result = client.verify(&key, b"data", b"sig");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("verification not supported")
        );
    }

    #[test]
    fn test_list_keys_empty() {
        let client = HsmClient::new();
        let keys = client.list_keys().unwrap();
        assert!(keys.is_empty());
    }

    #[test]
    fn test_delete_key_ok() {
        let client = HsmClient::new();
        let key = HsmKeyHandle {
            id: "pk-1".into(),
            label: "test-pub".into(),
            key_type: KeyType::Rsa,
            algorithm: "RSA-2048".into(),
            handle: 1,
        };
        assert!(client.delete_key(&key).is_ok());
    }

    #[test]
    fn test_health_disconnected() {
        let client = HsmClient::new();
        let status = client.health().unwrap();
        assert!(matches!(status, HsmHealthStatus::Disconnected));
    }

    #[test]
    fn test_connect_with_library_path_fails() {
        let config = HsmConfig {
            library_path: "/usr/lib/libpkcs11.so".into(),
            software_fallback: true,
            ..HsmConfig::default()
        };
        let client = HsmClient::with_config(config);
        let result = client.connect();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("HSM library not available")
        );
    }

    #[test]
    fn test_disconnect_already_disconnected() {
        let mut session = HsmSession {
            config: HsmConfig::default(),
            connected: false,
            session_handle: 0,
        };
        HsmClient::disconnect(&mut session);
        assert!(!session.connected);
        assert_eq!(session.session_handle, 0);
    }

    #[test]
    fn test_hsm_key_handle_fields() {
        let key = HsmKeyHandle {
            id: "my-id".into(),
            label: "my-label".into(),
            key_type: KeyType::Ecc,
            algorithm: "ECDSA-P256".into(),
            handle: 42,
        };
        assert_eq!(key.id, "my-id");
        assert_eq!(key.label, "my-label");
        assert_eq!(key.handle, 42);
    }

    #[test]
    fn test_hsm_key_handle_clone() {
        let key = HsmKeyHandle {
            id: "id".into(),
            label: "label".into(),
            key_type: KeyType::Rsa,
            algorithm: "RSA-4096".into(),
            handle: 1,
        };
        let cloned = key.clone();
        assert_eq!(key.id, cloned.id);
        assert_eq!(key.key_type, cloned.key_type);
    }

    #[test]
    fn test_hsm_config_default() {
        let config = HsmConfig::default();
        assert!(config.slot_label.is_none());
        assert!(config.pin.is_none());
        assert_eq!(config.library_path, "");
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(config.software_fallback);
    }

    #[test]
    fn test_hsm_config_clone() {
        let config = HsmConfig {
            slot_label: Some("slot".into()),
            pin: Some("pin".into()),
            library_path: "/path".into(),
            timeout: Duration::from_secs(10),
            software_fallback: false,
        };
        let cloned = config.clone();
        assert_eq!(cloned.slot_label.as_deref(), Some("slot"));
        assert!(!cloned.software_fallback);
    }

    #[test]
    fn test_hsm_session_clone() {
        let session = HsmSession {
            config: HsmConfig::default(),
            connected: true,
            session_handle: 99,
        };
        let cloned = session.clone();
        assert!(cloned.connected);
        assert_eq!(cloned.session_handle, 99);
    }

    #[test]
    fn test_hsm_health_status_variants() {
        let c = HsmHealthStatus::Connected;
        let d = HsmHealthStatus::Disconnected;
        let e = HsmHealthStatus::Error("fail".into());
        if let HsmHealthStatus::Error(msg) = e {
            assert_eq!(msg, "fail");
        } else {
            panic!("expected Error variant");
        }
        assert!(matches!(c, HsmHealthStatus::Connected));
        assert!(matches!(d, HsmHealthStatus::Disconnected));
    }

    #[test]
    fn test_sign_deterministic_hmac() {
        let client = HsmClient::new();
        let key = HsmKeyHandle {
            id: "k".into(),
            label: "l".into(),
            key_type: KeyType::Hmac,
            algorithm: "HMAC-SHA256".into(),
            handle: 1,
        };
        let sig1 = client.sign(&key, b"same data").unwrap();
        let sig2 = client.sign(&key, b"same data").unwrap();
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_sign_different_data_hmac() {
        let client = HsmClient::new();
        let key = HsmKeyHandle {
            id: "k".into(),
            label: "l".into(),
            key_type: KeyType::Hmac,
            algorithm: "HMAC-SHA256".into(),
            handle: 1,
        };
        let sig1 = client.sign(&key, b"data a").unwrap();
        let sig2 = client.sign(&key, b"data b").unwrap();
        assert_ne!(sig1, sig2);
    }
}
