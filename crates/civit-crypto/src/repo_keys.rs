#![forbid(unsafe_code)]

use ring::aead;
use ring::rand::SecureRandom;
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RepoKeyError {
    #[error("key derivation failed: {0}")]
    Derivation(String),

    #[error("encryption failed: {0}")]
    Encryption(String),

    #[error("decryption failed: {0}")]
    Decryption(String),

    #[error("key not found: {repo_id}")]
    KeyNotFound { repo_id: String },

    #[error("invalid key length: {0}")]
    InvalidKeyLength(usize),
}

#[derive(Debug, Clone)]
pub struct RepoEncryptionKey {
    repo_id: Uuid,
    key: aead::LessSafeKey,
    key_bytes: [u8; 32],
}

impl RepoEncryptionKey {
    pub fn derive(master_key: &[u8; 32], repo_id: Uuid) -> Result<Self, RepoKeyError> {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(master_key)
            .map_err(|e| RepoKeyError::Derivation(format!("HMAC init failed: {e}")))?;
        mac.update(repo_id.as_bytes());
        let _result = mac.finalize();

        let mut expanded = [0u8; 32];
        let prk = Sha256::digest(master_key);
        let info = repo_id.as_bytes();
        hmac_expand(&prk, info, &mut expanded)
            .map_err(|e| RepoKeyError::Derivation(format!("HKDF expand failed: {e}")))?;

        let unbound = aead::UnboundKey::new(&aead::AES_256_GCM, &expanded)
            .map_err(|e| RepoKeyError::Derivation(format!("key creation failed: {e}")))?;
        let key = aead::LessSafeKey::new(unbound);

        Ok(Self {
            repo_id,
            key,
            key_bytes: expanded,
        })
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), RepoKeyError> {
        let mut nonce_bytes = [0u8; aead::NONCE_LEN];
        ring::rand::SystemRandom::new()
            .fill(&mut nonce_bytes)
            .map_err(|e| RepoKeyError::Encryption(format!("nonce generation failed: {e:?}")))?;
        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);
        let mut ciphertext = plaintext.to_vec();
        ciphertext.extend_from_slice(&[0u8; 16]);

        self.key
            .seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut ciphertext)
            .map_err(|e| RepoKeyError::Encryption(format!("{e}")))?;

        Ok((nonce_bytes.to_vec(), ciphertext))
    }

    pub fn decrypt(
        &self,
        nonce: &[u8],
        ciphertext_with_tag: &mut [u8],
    ) -> Result<(), RepoKeyError> {
        self.key
            .open_in_place(
                aead::Nonce::try_assume_unique_for_key(nonce)
                    .map_err(|e| RepoKeyError::Decryption(format!("invalid nonce: {e}")))?,
                aead::Aad::empty(),
                ciphertext_with_tag,
            )
            .map_err(|e| RepoKeyError::Decryption(format!("{e}")))?;

        Ok(())
    }

    pub fn repo_id(&self) -> Uuid {
        self.repo_id
    }

    pub fn key_bytes(&self) -> &[u8; 32] {
        &self.key_bytes
    }

    pub fn from_bytes(key_bytes: [u8; 32], repo_id: Uuid) -> Self {
        let unbound =
            aead::UnboundKey::new(&aead::AES_256_GCM, &key_bytes).expect("invalid key bytes");
        let key = aead::LessSafeKey::new(unbound);
        Self {
            repo_id,
            key,
            key_bytes,
        }
    }
}

fn hmac_expand(prk: &[u8], info: &[u8], out: &mut [u8]) -> Result<(), String> {
    use hmac::{Hmac, Mac};
    type HmacSha256 = Hmac<Sha256>;

    if out.len() > 32 * 255 {
        return Err("output too long".into());
    }

    let mut t = [0u8; 32];
    let mut hmac = HmacSha256::new_from_slice(prk).map_err(|e| format!("{e}"))?;
    hmac.update(info);
    hmac.update(&[1u8]);
    let result = hmac.finalize();
    t.copy_from_slice(&result.into_bytes());

    let mut offset = 0;
    let mut counter = 1u8;
    while offset < out.len() {
        let copy_len = std::cmp::min(32, out.len() - offset);
        out[offset..offset + copy_len].copy_from_slice(&t[..copy_len]);
        offset += copy_len;

        if offset < out.len() {
            counter += 1;
            let mut hmac2 = HmacSha256::new_from_slice(prk).map_err(|e| format!("{e}"))?;
            hmac2.update(&t);
            hmac2.update(info);
            hmac2.update(&[counter]);
            let result2 = hmac2.finalize();
            for (i, b) in result2.into_bytes().iter().enumerate() {
                t[i] ^= b;
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct KeyRotation {
    pub repo_id: Uuid,
    pub current_key: RepoEncryptionKey,
    pub previous_key: Option<RepoEncryptionKey>,
    pub rotated_at: chrono::DateTime<chrono::Utc>,
}

impl KeyRotation {
    pub fn new(master_key: &[u8; 32], repo_id: Uuid) -> Result<Self, RepoKeyError> {
        let current_key = RepoEncryptionKey::derive(master_key, repo_id)?;
        Ok(Self {
            repo_id,
            current_key,
            previous_key: None,
            rotated_at: chrono::Utc::now(),
        })
    }

    pub fn rotate(&mut self, master_key: &[u8; 32]) -> Result<(), RepoKeyError> {
        let new_key = RepoEncryptionKey::derive(master_key, self.repo_id)?;
        self.previous_key = Some(std::mem::replace(&mut self.current_key, new_key));
        self.rotated_at = chrono::Utc::now();
        Ok(())
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), RepoKeyError> {
        self.current_key.encrypt(plaintext)
    }

    pub fn decrypt(&self, nonce: &[u8], ciphertext: &mut [u8]) -> Result<(), RepoKeyError> {
        match self.current_key.decrypt(nonce, ciphertext) {
            Ok(()) => Ok(()),
            Err(_) => {
                if let Some(ref prev) = self.previous_key {
                    prev.decrypt(nonce, ciphertext)
                } else {
                    Err(RepoKeyError::Decryption("decryption failed".into()))
                }
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RepoKeyStore {
    keys: std::collections::HashMap<Uuid, KeyRotation>,
    master_key: Option<[u8; 32]>,
}

impl RepoKeyStore {
    pub fn new(master_key: [u8; 32]) -> Self {
        Self {
            keys: std::collections::HashMap::new(),
            master_key: Some(master_key),
        }
    }

    pub fn get_or_create(&mut self, repo_id: Uuid) -> Result<&KeyRotation, RepoKeyError> {
        if !self.keys.contains_key(&repo_id) {
            let master = self.master_key.as_ref().ok_or(RepoKeyError::KeyNotFound {
                repo_id: repo_id.to_string(),
            })?;
            let rotation = KeyRotation::new(master, repo_id)?;
            self.keys.insert(repo_id, rotation);
        }
        Ok(self.keys.get(&repo_id).unwrap())
    }

    pub fn rotate_key(&mut self, repo_id: Uuid) -> Result<(), RepoKeyError> {
        let master = self.master_key.as_ref().ok_or(RepoKeyError::KeyNotFound {
            repo_id: repo_id.to_string(),
        })?;
        let rotation = self
            .keys
            .get_mut(&repo_id)
            .ok_or(RepoKeyError::KeyNotFound {
                repo_id: repo_id.to_string(),
            })?;
        rotation.rotate(master)
    }

    pub fn list_repos(&self) -> Vec<Uuid> {
        self.keys.keys().copied().collect()
    }

    pub fn remove(&mut self, repo_id: Uuid) {
        self.keys.remove(&repo_id);
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_master_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        for (i, b) in key.iter_mut().enumerate() {
            *b = (i as u8).wrapping_add(42);
        }
        key
    }

    #[test]
    fn test_derive_repo_key() {
        let master = test_master_key();
        let repo_id = Uuid::new_v4();
        let key = RepoEncryptionKey::derive(&master, repo_id).unwrap();
        assert_eq!(key.repo_id(), repo_id);
        assert_eq!(key.key_bytes().len(), 32);
    }

    #[test]
    fn test_different_repos_different_keys() {
        let master = test_master_key();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let key1 = RepoEncryptionKey::derive(&master, id1).unwrap();
        let key2 = RepoEncryptionKey::derive(&master, id2).unwrap();
        assert_ne!(key1.key_bytes(), key2.key_bytes());
    }

    #[test]
    fn test_same_repo_same_key() {
        let master = test_master_key();
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let key1 = RepoEncryptionKey::derive(&master, id).unwrap();
        let key2 = RepoEncryptionKey::derive(&master, id).unwrap();
        assert_eq!(key1.key_bytes(), key2.key_bytes());
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let master = test_master_key();
        let repo_id = Uuid::new_v4();
        let key = RepoEncryptionKey::derive(&master, repo_id).unwrap();

        let plaintext = b"hello world, this is a secret pipeline variable";
        let (nonce, mut ciphertext) = key.encrypt(plaintext).unwrap();
        assert_eq!(nonce.len(), 12);
        assert!(ciphertext.len() > plaintext.len());

        key.decrypt(&nonce, &mut ciphertext).unwrap();
        assert_eq!(&ciphertext[..plaintext.len()], plaintext);
    }

    #[test]
    fn test_from_bytes() {
        let master = test_master_key();
        let repo_id = Uuid::new_v4();
        let key1 = RepoEncryptionKey::derive(&master, repo_id).unwrap();
        let key_bytes = *key1.key_bytes();
        let key2 = RepoEncryptionKey::from_bytes(key_bytes, repo_id);
        assert_eq!(key1.key_bytes(), key2.key_bytes());
    }

    #[test]
    fn test_key_rotation_new() {
        let master = test_master_key();
        let repo_id = Uuid::new_v4();
        let rotation = KeyRotation::new(&master, repo_id).unwrap();
        assert!(rotation.previous_key.is_none());
    }

    #[test]
    fn test_key_rotation_rotate() {
        let master = test_master_key();
        let repo_id = Uuid::new_v4();
        let mut rotation = KeyRotation::new(&master, repo_id).unwrap();

        let plaintext = b"test secret";
        let (nonce, mut ciphertext) = rotation.encrypt(plaintext).unwrap();

        rotation.rotate(&master).unwrap();
        assert!(rotation.previous_key.is_some());

        rotation.decrypt(&nonce, &mut ciphertext).unwrap();
        assert_eq!(&ciphertext[..plaintext.len()], plaintext);
    }

    #[test]
    fn test_key_store_get_or_create() {
        let master = test_master_key();
        let mut store = RepoKeyStore::new(master);
        let repo_id = Uuid::new_v4();
        let rotation = store.get_or_create(repo_id).unwrap();
        assert_eq!(rotation.repo_id, repo_id);
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_key_store_same_repo() {
        let master = test_master_key();
        let mut store = RepoKeyStore::new(master);
        let repo_id = Uuid::new_v4();
        let _ = store.get_or_create(repo_id).unwrap();
        let _ = store.get_or_create(repo_id).unwrap();
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_key_store_rotate() {
        let master = test_master_key();
        let mut store = RepoKeyStore::new(master);
        let repo_id = Uuid::new_v4();
        store.get_or_create(repo_id).unwrap();
        store.rotate_key(repo_id).unwrap();
    }

    #[test]
    fn test_key_store_remove() {
        let master = test_master_key();
        let mut store = RepoKeyStore::new(master);
        let repo_id = Uuid::new_v4();
        store.get_or_create(repo_id).unwrap();
        store.remove(repo_id);
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_key_store_list() {
        let master = test_master_key();
        let mut store = RepoKeyStore::new(master);
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        store.get_or_create(id1).unwrap();
        store.get_or_create(id2).unwrap();
        let repos = store.list_repos();
        assert_eq!(repos.len(), 2);
    }

    #[test]
    fn test_key_store_default_no_master() {
        let store = RepoKeyStore::default();
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_encrypt_empty_plaintext() {
        let master = test_master_key();
        let repo_id = Uuid::new_v4();
        let key = RepoEncryptionKey::derive(&master, repo_id).unwrap();
        let (nonce, mut ciphertext) = key.encrypt(b"").unwrap();
        key.decrypt(&nonce, &mut ciphertext).unwrap();
    }

    #[test]
    fn test_large_plaintext() {
        let master = test_master_key();
        let repo_id = Uuid::new_v4();
        let key = RepoEncryptionKey::derive(&master, repo_id).unwrap();
        let plaintext = vec![0xABu8; 100_000];
        let (nonce, mut ciphertext) = key.encrypt(&plaintext).unwrap();
        key.decrypt(&nonce, &mut ciphertext).unwrap();
        assert_eq!(&ciphertext[..100_000], &plaintext[..]);
    }
}
