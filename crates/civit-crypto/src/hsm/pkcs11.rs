#![forbid(unsafe_code)]

use ring::{
    aead, hmac,
    rand::SecureRandom,
    signature::{self, KeyPair},
};
use std::collections::HashMap;
use std::fmt;
use std::sync::RwLock;
use std::time::Duration;
use tracing::warn;

/// Software key material stored in memory (fallback when no HSM hardware).
/// Audit warning: keys in process memory. Real HSM keeps keys inside the module.
struct SoftwareKeyEntry {
    public_key_der: Vec<u8>,
    key_pair: SoftwareKeyPair,
    key_type: KeyType,
    algorithm: String,
}

/// Enumerates the key pair types we support in software fallback.
enum SoftwareKeyPair {
    Ecdsa(signature::EcdsaKeyPair),
    Aes(Box<aead::LessSafeKey>),
    Hmac(hmac::Key),
}

impl fmt::Debug for SoftwareKeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ecdsa(_) => write!(f, "SoftwareKeyPair::Ecdsa"),
            Self::Aes(_) => write!(f, "SoftwareKeyPair::Aes"),
            Self::Hmac(_) => write!(f, "SoftwareKeyPair::Hmac"),
        }
    }
}

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
    /// Software key store: key_id -> key material.
    /// Only populated when software_fallback is true and no HSM hardware is present.
    software_keys: RwLock<HashMap<String, SoftwareKeyEntry>>,
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
            software_keys: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_config(config: HsmConfig) -> Self {
        Self {
            config,
            software_keys: RwLock::new(HashMap::new()),
        }
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

        warn!("HSM software fallback active — keys held in process memory");
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
        _bits: u16,
    ) -> anyhow::Result<(HsmKeyHandle, HsmKeyHandle)> {
        let pk_id = format!("pk-{}", uuid::Uuid::new_v4());
        let sk_id = format!("sk-{}", uuid::Uuid::new_v4());

        match key_type {
            KeyType::Rsa => {
                // RSA signing is not supported in ring (software fallback).
                // RSA requires real HSM hardware or a different crypto library.
                anyhow::bail!(
                    "RSA key pair generation requires HSM hardware; \
                     ring does not support RSA signing in software fallback"
                );
            }
            KeyType::Ecc => {
                let rng = ring::rand::SystemRandom::new();
                let pkcs8_bytes = signature::EcdsaKeyPair::generate_pkcs8(
                    &signature::ECDSA_P256_SHA256_ASN1_SIGNING,
                    &rng,
                )
                .map_err(|e| anyhow::anyhow!("ECDSA key generation failed: {e:?}"))?;
                let key_pair = signature::EcdsaKeyPair::from_pkcs8(
                    &signature::ECDSA_P256_SHA256_ASN1_SIGNING,
                    pkcs8_bytes.as_ref(),
                    &rng,
                )
                .map_err(|e| anyhow::anyhow!("ECDSA key parsing failed: {e:?}"))?;

                let public_der = key_pair.public_key().as_ref().to_vec();

                let mut keys = self.software_keys.write().unwrap();
                keys.insert(
                    sk_id.clone(),
                    SoftwareKeyEntry {
                        public_key_der: public_der.clone(),
                        key_pair: SoftwareKeyPair::Ecdsa(key_pair),
                        key_type: KeyType::Ecc,
                        algorithm: "ECDSA-P256-SHA256".to_string(),
                    },
                );

                let pk_handle = HsmKeyHandle {
                    id: pk_id.clone(),
                    label: format!("{label}-pub"),
                    key_type: KeyType::Ecc,
                    algorithm: "ECDSA-P256-SHA256".to_string(),
                    handle: 1,
                };
                let sk_handle = HsmKeyHandle {
                    id: sk_id.clone(),
                    label: format!("{label}-priv"),
                    key_type: KeyType::Ecc,
                    algorithm: "ECDSA-P256-SHA256".to_string(),
                    handle: 2,
                };
                Ok((pk_handle, sk_handle))
            }
            _ => anyhow::bail!("key type {key_type:?} not supported for key pair generation"),
        }
    }

    /// Generate a symmetric key (AES or HMAC).
    pub fn generate_symmetric_key(
        &self,
        label: &str,
        key_type: KeyType,
        bits: u16,
    ) -> anyhow::Result<HsmKeyHandle> {
        let key_id = format!("sk-{}", uuid::Uuid::new_v4());

        let algorithm_str = match key_type {
            KeyType::Aes => {
                if bits != 128 && bits != 256 {
                    anyhow::bail!("AES key must be 128 or 256 bits, got {bits}");
                }
                let key_bytes = bits / 8;
                let mut raw_key = vec![0u8; key_bytes as usize];
                ring::rand::SystemRandom::new()
                    .fill(&mut raw_key)
                    .map_err(|e| anyhow::anyhow!("random generation failed: {e:?}"))?;
                let algorithm = match bits {
                    128 => &aead::AES_128_GCM,
                    256 => &aead::AES_256_GCM,
                    other => anyhow::bail!("AES key must be 128 or 256 bits, got {other}"),
                };
                let aead_key = aead::LessSafeKey::new(
                    aead::UnboundKey::new(algorithm, &raw_key)
                        .map_err(|e| anyhow::anyhow!("AES key creation failed: {e:?}"))?,
                );

                let mut keys = self.software_keys.write().unwrap();
                keys.insert(
                    key_id.clone(),
                    SoftwareKeyEntry {
                        public_key_der: Vec::new(),
                        key_pair: SoftwareKeyPair::Aes(Box::new(aead_key)),
                        key_type: KeyType::Aes,
                        algorithm: format!("AES-{bits}-GCM"),
                    },
                );
                format!("AES-{bits}-GCM")
            }
            KeyType::Hmac => {
                let mut hmac_raw = [0u8; 32];
                ring::rand::SystemRandom::new()
                    .fill(&mut hmac_raw)
                    .map_err(|e| anyhow::anyhow!("random generation failed: {e:?}"))?;
                let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &hmac_raw);

                let mut keys = self.software_keys.write().unwrap();
                keys.insert(
                    key_id.clone(),
                    SoftwareKeyEntry {
                        public_key_der: Vec::new(),
                        key_pair: SoftwareKeyPair::Hmac(hmac_key),
                        key_type: KeyType::Hmac,
                        algorithm: "HMAC-SHA256".to_string(),
                    },
                );
                "HMAC-SHA256".to_string()
            }
            _ => anyhow::bail!("symmetric key generation not supported for {key_type:?}"),
        };

        Ok(HsmKeyHandle {
            id: key_id,
            label: format!("{label}-sym"),
            key_type,
            algorithm: algorithm_str,
            handle: 3,
        })
    }

    pub fn sign(&self, key: &HsmKeyHandle, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        warn!(
            key_id = %key.id,
            key_type = %key.key_type,
            "signing with software fallback (keys in memory)"
        );

        let keys = self.software_keys.read().unwrap();
        let entry = keys
            .get(&key.id)
            .ok_or_else(|| anyhow::anyhow!("key {} not found in software key store", key.id))?;

        match &entry.key_pair {
            SoftwareKeyPair::Ecdsa(key_pair) => {
                let rng = ring::rand::SystemRandom::new();
                let sig = key_pair
                    .sign(&rng, data)
                    .map_err(|e| anyhow::anyhow!("ECDSA signing failed: {e:?}"))?;
                Ok(sig.as_ref().to_vec())
            }
            SoftwareKeyPair::Aes(_) => {
                anyhow::bail!("AES keys cannot be used for signing; use HMAC or ECDSA")
            }
            SoftwareKeyPair::Hmac(hmac_key) => {
                let tag = hmac::sign(hmac_key, data);
                Ok(tag.as_ref().to_vec())
            }
        }
    }

    pub fn verify(
        &self,
        key: &HsmKeyHandle,
        data: &[u8],
        signature_bytes: &[u8],
    ) -> anyhow::Result<bool> {
        let keys = self.software_keys.read().unwrap();
        let entry = keys
            .get(&key.id)
            .ok_or_else(|| anyhow::anyhow!("key {} not found in software key store", key.id))?;

        match &entry.key_pair {
            SoftwareKeyPair::Ecdsa(_) => {
                let public_key = signature::UnparsedPublicKey::new(
                    &signature::ECDSA_P256_SHA256_ASN1,
                    &entry.public_key_der,
                );
                Ok(public_key.verify(data, signature_bytes).is_ok())
            }
            SoftwareKeyPair::Aes(_) => {
                anyhow::bail!("AES keys cannot be used for verification; use HMAC or ECDSA")
            }
            SoftwareKeyPair::Hmac(hmac_key) => {
                Ok(hmac::verify(hmac_key, data, signature_bytes).is_ok())
            }
        }
    }

    /// Encrypt data with AES-GCM. Returns nonce || ciphertext || tag.
    pub fn encrypt(&self, key: &HsmKeyHandle, plaintext: &[u8]) -> anyhow::Result<Vec<u8>> {
        let keys = self.software_keys.read().unwrap();
        let entry = keys
            .get(&key.id)
            .ok_or_else(|| anyhow::anyhow!("key {} not found in software key store", key.id))?;

        match &entry.key_pair {
            SoftwareKeyPair::Aes(aead_key) => {
                let mut nonce_bytes = [0u8; aead::NONCE_LEN];
                ring::rand::SystemRandom::new()
                    .fill(&mut nonce_bytes)
                    .map_err(|e| anyhow::anyhow!("nonce generation failed: {e:?}"))?;
                let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);
                let aad = aead::Aad::empty();

                // seal_in_place_append_tag writes ciphertext in-place, then appends tag.
                // We need a buffer of plaintext.len() bytes (it overwrites) plus room for tag.
                let mut in_out = vec![0u8; plaintext.len()];
                in_out.copy_from_slice(plaintext);

                aead_key
                    .seal_in_place_append_tag(nonce, aad, &mut in_out)
                    .map_err(|e| anyhow::anyhow!("AES-GCM encryption failed: {e:?}"))?;

                // in_out now contains: ciphertext || tag (exactly)
                let mut result = nonce_bytes.to_vec();
                result.extend_from_slice(&in_out);
                Ok(result)
            }
            SoftwareKeyPair::Hmac(_) | SoftwareKeyPair::Ecdsa(_) => {
                anyhow::bail!("only AES keys support encryption; got {:?}", key.key_type)
            }
        }
    }

    /// Decrypt data with AES-GCM. Expects nonce (12 bytes) || ciphertext || tag (16 bytes).
    pub fn decrypt(&self, key: &HsmKeyHandle, ciphertext: &[u8]) -> anyhow::Result<Vec<u8>> {
        let nonce_len = aead::NONCE_LEN;
        let tag_len = 16; // AES-GCM tag is always 16 bytes
        let min_len = nonce_len + tag_len;
        if ciphertext.len() < min_len {
            anyhow::bail!(
                "ciphertext too short: expected >= {min_len} bytes (nonce+tag), got {}",
                ciphertext.len()
            );
        }

        let keys = self.software_keys.read().unwrap();
        let entry = keys
            .get(&key.id)
            .ok_or_else(|| anyhow::anyhow!("key {} not found in software key store", key.id))?;

        match &entry.key_pair {
            SoftwareKeyPair::Aes(aead_key) => {
                let nonce = aead::Nonce::try_assume_unique_for_key(&ciphertext[..nonce_len])
                    .map_err(|_| anyhow::anyhow!("invalid nonce"))?;
                let aad = aead::Aad::empty();
                let mut in_out = ciphertext[nonce_len..].to_vec();

                let plaintext_ref = aead_key
                    .open_in_place(nonce, aad, &mut in_out)
                    .map_err(|e| anyhow::anyhow!("AES-GCM decryption failed: {e:?}"))?;

                Ok(plaintext_ref.to_vec())
            }
            SoftwareKeyPair::Hmac(_) | SoftwareKeyPair::Ecdsa(_) => {
                anyhow::bail!("only AES keys support decryption; got {:?}", key.key_type)
            }
        }
    }

    pub fn list_keys(&self) -> anyhow::Result<Vec<HsmKeyHandle>> {
        let keys = self.software_keys.read().unwrap();
        Ok(keys
            .iter()
            .map(|(id, entry)| HsmKeyHandle {
                id: id.clone(),
                label: id.clone(),
                key_type: entry.key_type.clone(),
                algorithm: entry.algorithm.clone(),
                handle: 1,
            })
            .collect())
    }

    pub fn delete_key(&self, key: &HsmKeyHandle) -> anyhow::Result<()> {
        let mut keys = self.software_keys.write().unwrap();
        keys.remove(&key.id);
        Ok(())
    }

    pub fn health(&self) -> anyhow::Result<HsmHealthStatus> {
        if self.config.software_fallback {
            Ok(HsmHealthStatus::Disconnected) // software fallback, no real HSM
        } else {
            Ok(HsmHealthStatus::Error(
                "HSM unavailable and software fallback disabled".into(),
            ))
        }
    }
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
    fn test_generate_rsa_key_pair_fails() {
        let client = HsmClient::new();
        let result = client.generate_key_pair("test-rsa", KeyType::Rsa, 2048);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("HSM hardware"));
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
        assert_eq!(pk.algorithm, "ECDSA-P256-SHA256");
        assert_eq!(sk.algorithm, "ECDSA-P256-SHA256");
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
    fn test_generate_key_pair_distinct_ids() {
        let client = HsmClient::new();
        let (pk1, sk1) = client.generate_key_pair("key1", KeyType::Ecc, 256).unwrap();
        let (pk2, sk2) = client.generate_key_pair("key2", KeyType::Ecc, 256).unwrap();
        assert_ne!(pk1.id, pk2.id);
        assert_ne!(sk1.id, sk2.id);
    }

    #[test]
    fn test_ecdsa_sign_and_verify_roundtrip() {
        let client = HsmClient::new();
        let (_pk, sk) = client
            .generate_key_pair("test-ecc", KeyType::Ecc, 256)
            .unwrap();

        let sig = client.sign(&sk, b"hello world").unwrap();
        assert!(!sig.is_empty());

        // Verify with the same key (uses stored public key)
        let valid = client.verify(&sk, b"hello world", &sig).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_ecdsa_verify_wrong_data_fails() {
        let client = HsmClient::new();
        let (_pk, sk) = client
            .generate_key_pair("test-ecc", KeyType::Ecc, 256)
            .unwrap();

        let sig = client.sign(&sk, b"hello world").unwrap();
        let valid = client.verify(&sk, b"wrong data", &sig).unwrap();
        assert!(!valid);
    }

    #[test]
    fn test_ecdsa_verify_wrong_signature_fails() {
        let client = HsmClient::new();
        let (_pk, sk) = client
            .generate_key_pair("test-ecc", KeyType::Ecc, 256)
            .unwrap();

        let valid = client
            .verify(&sk, b"hello world", b"bogus_signature_bytes")
            .unwrap();
        assert!(!valid);
    }

    #[test]
    fn test_generate_aes_key() {
        let client = HsmClient::new();
        let key = client
            .generate_symmetric_key("test-aes", KeyType::Aes, 256)
            .unwrap();
        assert!(key.id.starts_with("sk-"));
        assert_eq!(key.key_type, KeyType::Aes);
        assert!(key.algorithm.contains("AES-256"));
    }

    #[test]
    fn test_generate_aes_128_key() {
        let client = HsmClient::new();
        let key = client
            .generate_symmetric_key("test-aes128", KeyType::Aes, 128)
            .unwrap();
        assert!(key.algorithm.contains("AES-128"));
    }

    #[test]
    fn test_generate_aes_key_invalid_bits() {
        let client = HsmClient::new();
        let result = client.generate_symmetric_key("bad", KeyType::Aes, 192);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_hmac_key() {
        let client = HsmClient::new();
        let key = client
            .generate_symmetric_key("test-hmac", KeyType::Hmac, 256)
            .unwrap();
        assert!(key.id.starts_with("sk-"));
        assert_eq!(key.key_type, KeyType::Hmac);
        assert!(key.algorithm.contains("HMAC"));
    }

    #[test]
    fn test_hmac_sign_and_verify_roundtrip() {
        let client = HsmClient::new();
        let key = client
            .generate_symmetric_key("test-hmac", KeyType::Hmac, 256)
            .unwrap();

        let sig = client.sign(&key, b"hello world").unwrap();
        assert!(!sig.is_empty());

        let valid = client.verify(&key, b"hello world", &sig).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_hmac_verify_wrong_data_fails() {
        let client = HsmClient::new();
        let key = client
            .generate_symmetric_key("test-hmac", KeyType::Hmac, 256)
            .unwrap();

        let sig = client.sign(&key, b"hello world").unwrap();
        let valid = client.verify(&key, b"wrong data", &sig).unwrap();
        assert!(!valid);
    }

    #[test]
    fn test_hmac_sign_deterministic() {
        let client = HsmClient::new();
        let key = client
            .generate_symmetric_key("test-hmac", KeyType::Hmac, 256)
            .unwrap();

        let sig1 = client.sign(&key, b"same data").unwrap();
        let sig2 = client.sign(&key, b"same data").unwrap();
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_aes_encrypt_decrypt_roundtrip() {
        let client = HsmClient::new();
        let key = client
            .generate_symmetric_key("test-aes", KeyType::Aes, 256)
            .unwrap();

        let plaintext = b"hello world, this is a secret message";
        let ciphertext = client.encrypt(&key, plaintext).unwrap();

        // Ciphertext must be longer (nonce + tag)
        assert!(ciphertext.len() > plaintext.len());

        let decrypted = client.decrypt(&key, &ciphertext).unwrap();
        assert_eq!(&decrypted, plaintext);
    }

    #[test]
    fn test_aes_encrypt_decrypt_empty() {
        let client = HsmClient::new();
        let key = client
            .generate_symmetric_key("test-aes", KeyType::Aes, 256)
            .unwrap();

        let ciphertext = client.encrypt(&key, b"").unwrap();
        let decrypted = client.decrypt(&key, &ciphertext).unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_aes_decrypt_wrong_key_fails() {
        let client = HsmClient::new();
        let key1 = client
            .generate_symmetric_key("aes-1", KeyType::Aes, 256)
            .unwrap();
        let key2 = client
            .generate_symmetric_key("aes-2", KeyType::Aes, 256)
            .unwrap();

        let ciphertext = client.encrypt(&key1, b"secret data").unwrap();
        let result = client.decrypt(&key2, &ciphertext);
        assert!(result.is_err());
    }

    #[test]
    fn test_aes_decrypt_tampered_fails() {
        let client = HsmClient::new();
        let key = client
            .generate_symmetric_key("aes", KeyType::Aes, 256)
            .unwrap();

        let mut ciphertext = client.encrypt(&key, b"secret data").unwrap();
        // Tamper with last byte
        let last = ciphertext.len() - 1;
        ciphertext[last] ^= 0xFF;
        let result = client.decrypt(&key, &ciphertext);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_with_ecdsa_key_fails() {
        let client = HsmClient::new();
        let (_pk, sk) = client
            .generate_key_pair("test-ecc", KeyType::Ecc, 256)
            .unwrap();
        let result = client.encrypt(&sk, b"plaintext");
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_with_aes_key_fails() {
        let client = HsmClient::new();
        let key = client
            .generate_symmetric_key("test-aes", KeyType::Aes, 256)
            .unwrap();
        let result = client.sign(&key, b"data");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_nonexistent_key() {
        let client = HsmClient::new();
        let key = HsmKeyHandle {
            id: "nonexistent".into(),
            label: "missing".into(),
            key_type: KeyType::Ecc,
            algorithm: "ECDSA-P256".into(),
            handle: 0,
        };
        let result = client.verify(&key, b"data", b"sig");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_list_keys_empty() {
        let client = HsmClient::new();
        let keys = client.list_keys().unwrap();
        assert!(keys.is_empty());
    }

    #[test]
    fn test_list_keys_after_generate() {
        let client = HsmClient::new();
        client.generate_key_pair("ecc1", KeyType::Ecc, 256).unwrap();
        client.generate_key_pair("ecc2", KeyType::Ecc, 256).unwrap();
        let keys = client.list_keys().unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_delete_key() {
        let client = HsmClient::new();
        let (_pk, sk) = client
            .generate_key_pair("test-ecc", KeyType::Ecc, 256)
            .unwrap();
        assert!(client.delete_key(&sk).is_ok());
        // Verify it's gone
        let result = client.sign(&sk, b"data");
        assert!(result.is_err());
    }

    #[test]
    fn test_health_disconnected() {
        let client = HsmClient::new();
        let status = client.health().unwrap();
        assert!(matches!(status, HsmHealthStatus::Disconnected));
    }

    #[test]
    fn test_health_no_fallback() {
        let config = HsmConfig {
            software_fallback: false,
            ..HsmConfig::default()
        };
        let client = HsmClient::with_config(config);
        let status = client.health().unwrap();
        assert!(matches!(status, HsmHealthStatus::Error(_)));
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
    fn test_aes_128_encrypt_decrypt_roundtrip() {
        let client = HsmClient::new();
        let key = client
            .generate_symmetric_key("aes128", KeyType::Aes, 128)
            .unwrap();
        let plaintext = b"short msg";
        let ct = client.encrypt(&key, plaintext).unwrap();
        let pt = client.decrypt(&key, &ct).unwrap();
        assert_eq!(&pt, plaintext);
    }
}
