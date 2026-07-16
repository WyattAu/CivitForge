#![forbid(unsafe_code)]

//! Encryption at rest for CivitForge.
//!
//! Provides key management, data encryption/decryption, key rotation,
//! and envelope encryption capabilities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionKey {
    pub id: Uuid,
    pub name: String,
    pub algorithm: String,
    pub rotation_date: Option<DateTime<Utc>>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEncryptionKey {
    pub name: String,
    pub algorithm: Option<String>,
    pub key_material: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPayload {
    pub id: Uuid,
    pub key_id: Uuid,
    pub data_type: String,
    pub data_id: Uuid,
    pub encrypted_data: Vec<u8>,
    pub iv: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct EncryptionKeyRow {
    id: Uuid,
    name: String,
    algorithm: String,
    rotation_date: Option<DateTime<Utc>>,
    enabled: bool,
    created_at: DateTime<Utc>,
}

impl From<EncryptionKeyRow> for EncryptionKey {
    fn from(row: EncryptionKeyRow) -> Self {
        EncryptionKey {
            id: row.id,
            name: row.name,
            algorithm: row.algorithm,
            rotation_date: row.rotation_date,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct EncryptedPayloadRow {
    id: Uuid,
    key_id: Uuid,
    data_type: String,
    data_id: Uuid,
    encrypted_data: Vec<u8>,
    iv: Vec<u8>,
    created_at: DateTime<Utc>,
}

impl From<EncryptedPayloadRow> for EncryptedPayload {
    fn from(row: EncryptedPayloadRow) -> Self {
        EncryptedPayload {
            id: row.id,
            key_id: row.key_id,
            data_type: row.data_type,
            data_id: row.data_id,
            encrypted_data: row.encrypted_data,
            iv: row.iv,
            created_at: row.created_at,
        }
    }
}

pub struct EncryptionService {
    pool: PgPool,
}

impl EncryptionService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_key(&self, input: CreateEncryptionKey) -> Result<EncryptionKey, sqlx::Error> {
        let algorithm = input.algorithm.unwrap_or_else(|| "AES-256-GCM".into());

        let row = sqlx::query_as::<_, EncryptionKeyRow>(
            r#"INSERT INTO encryption_keys (name, algorithm, key_material)
             VALUES ($1, $2, $3)
             RETURNING id, name, algorithm, rotation_date, enabled, created_at"#,
        )
        .bind(&input.name)
        .bind(&algorithm)
        .bind(&input.key_material)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_key(&self, id: Uuid) -> Result<Option<EncryptionKey>, sqlx::Error> {
        let row = sqlx::query_as::<_, EncryptionKeyRow>(
            r#"SELECT id, name, algorithm, rotation_date, enabled, created_at
             FROM encryption_keys WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_keys(&self) -> Result<Vec<EncryptionKey>, sqlx::Error> {
        let rows = sqlx::query_as::<_, EncryptionKeyRow>(
            r#"SELECT id, name, algorithm, rotation_date, enabled, created_at
             FROM encryption_keys ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn rotate_key(&self, id: Uuid, new_material: Vec<u8>) -> Result<EncryptionKey, sqlx::Error> {
        let row = sqlx::query_as::<_, EncryptionKeyRow>(
            r#"UPDATE encryption_keys SET
             key_material = $2,
             rotation_date = NOW()
             WHERE id = $1
             RETURNING id, name, algorithm, rotation_date, enabled, created_at"#,
        )
        .bind(id)
        .bind(new_material)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn disable_key(&self, id: Uuid) -> Result<EncryptionKey, sqlx::Error> {
        let row = sqlx::query_as::<_, EncryptionKeyRow>(
            r#"UPDATE encryption_keys SET enabled = false
             WHERE id = $1
             RETURNING id, name, algorithm, rotation_date, enabled, created_at"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn encrypt_data(
        &self,
        key_id: Uuid,
        data_type: &str,
        data_id: Uuid,
        plaintext: &[u8],
    ) -> Result<EncryptedPayload, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct KeyRow {
            key_material: Vec<u8>,
            #[allow(dead_code)]
            algorithm: String,
        }

        let key_row = sqlx::query_as::<_, KeyRow>(
            r#"SELECT key_material, algorithm FROM encryption_keys WHERE id = $1 AND enabled = true"#,
        )
        .bind(key_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let iv = generate_iv();
        let encrypted = encrypt_aes_gcm(plaintext, &key_row.key_material, &iv);

        let row = sqlx::query_as::<_, EncryptedPayloadRow>(
            r#"INSERT INTO encrypted_data (key_id, data_type, data_id, encrypted_data, iv)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, key_id, data_type, data_id, encrypted_data, iv, created_at"#,
        )
        .bind(key_id)
        .bind(data_type)
        .bind(data_id)
        .bind(encrypted)
        .bind(iv)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn decrypt_data(
        &self,
        payload_id: Uuid,
    ) -> Result<Vec<u8>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct DecryptRow {
            encrypted_data: Vec<u8>,
            iv: Vec<u8>,
            key_material: Vec<u8>,
        }

        let row = sqlx::query_as::<_, DecryptRow>(
            r#"SELECT ed.encrypted_data, ed.iv, ek.key_material
             FROM encrypted_data ed
             JOIN encryption_keys ek ON ek.id = ed.key_id
             WHERE ed.id = $1 AND ek.enabled = true"#,
        )
        .bind(payload_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)?;

        Ok(decrypt_aes_gcm(&row.encrypted_data, &row.key_material, &row.iv))
    }

    pub async fn get_encrypted_data_by_reference(
        &self,
        data_type: &str,
        data_id: Uuid,
    ) -> Result<Option<EncryptedPayload>, sqlx::Error> {
        let row = sqlx::query_as::<_, EncryptedPayloadRow>(
            r#"SELECT id, key_id, data_type, data_id, encrypted_data, iv, created_at
             FROM encrypted_data
             WHERE data_type = $1 AND data_id = $2"#,
        )
        .bind(data_type)
        .bind(data_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn needs_rotation(&self, key_id: Uuid, max_age_days: i64) -> Result<bool, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct RotationRow {
            rotation_date: Option<DateTime<Utc>>,
            created_at: DateTime<Utc>,
        }

        let row = sqlx::query_as::<_, RotationRow>(
            r#"SELECT rotation_date, created_at FROM encryption_keys WHERE id = $1"#,
        )
        .bind(key_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let last_rotation = row.rotation_date.unwrap_or(row.created_at);
        let age = Utc::now() - last_rotation;
        Ok(age.num_days() > max_age_days)
    }
}

fn generate_iv() -> Vec<u8> {
    use rand::RngExt;
    let mut iv = vec![0u8; 12];
    rand::rng().fill(&mut iv[..]);
    iv
}

fn encrypt_aes_gcm(plaintext: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
    use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};

    let cipher = Aes256Gcm::new_from_slice(key).expect("invalid key length");
    let nonce = Nonce::from_slice(iv);
    cipher.encrypt(nonce, plaintext).expect("encryption failed")
}

fn decrypt_aes_gcm(ciphertext: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
    use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};

    let cipher = Aes256Gcm::new_from_slice(key).expect("invalid key length");
    let nonce = Nonce::from_slice(iv);
    cipher.decrypt(nonce, ciphertext).expect("decryption failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iv_generation() {
        let iv1 = generate_iv();
        let iv2 = generate_iv();
        assert_eq!(iv1.len(), 12);
        assert_ne!(iv1, iv2);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = vec![0u8; 32];
        let iv = generate_iv();
        let plaintext = b"hello world";
        let encrypted = encrypt_aes_gcm(plaintext, &key, &iv);
        let decrypted = decrypt_aes_gcm(&encrypted, &key, &iv);
        assert_eq!(plaintext.to_vec(), decrypted);
    }
}
