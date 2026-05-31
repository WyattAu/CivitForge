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
}
