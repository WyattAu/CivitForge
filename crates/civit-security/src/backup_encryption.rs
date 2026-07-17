#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEncryptionKey {
    pub id: Uuid,
    pub backup_id: Uuid,
    pub key_id: Uuid,
    pub encrypted_key: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBackupEncryptionKey {
    pub backup_id: Uuid,
    pub key_id: Uuid,
    pub encrypted_key: Vec<u8>,
}

#[derive(Debug, sqlx::FromRow)]
struct BackupEncryptionKeyRow {
    id: Uuid,
    backup_id: Uuid,
    key_id: Uuid,
    encrypted_key: Vec<u8>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<BackupEncryptionKeyRow> for BackupEncryptionKey {
    fn from(row: BackupEncryptionKeyRow) -> Self {
        BackupEncryptionKey {
            id: row.id,
            backup_id: row.backup_id,
            key_id: row.key_id,
            encrypted_key: row.encrypted_key,
            created_at: row.created_at,
        }
    }
}

pub struct BackupEncryptionService {
    pool: PgPool,
}

impl BackupEncryptionService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_key(
        &self,
        input: CreateBackupEncryptionKey,
    ) -> Result<BackupEncryptionKey, sqlx::Error> {
        let row = sqlx::query_as::<_, BackupEncryptionKeyRow>(
            r#"INSERT INTO backup_encryption_keys (backup_id, key_id, encrypted_key)
             VALUES ($1, $2, $3)
             RETURNING id, backup_id, key_id, encrypted_key, created_at"#,
        )
        .bind(input.backup_id)
        .bind(input.key_id)
        .bind(&input.encrypted_key)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_key(&self, id: Uuid) -> Result<Option<BackupEncryptionKey>, sqlx::Error> {
        let row = sqlx::query_as::<_, BackupEncryptionKeyRow>(
            r#"SELECT id, backup_id, key_id, encrypted_key, created_at
             FROM backup_encryption_keys WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_keys_for_backup(
        &self,
        backup_id: Uuid,
    ) -> Result<Vec<BackupEncryptionKey>, sqlx::Error> {
        let rows = sqlx::query_as::<_, BackupEncryptionKeyRow>(
            r#"SELECT id, backup_id, key_id, encrypted_key, created_at
             FROM backup_encryption_keys WHERE backup_id = $1 ORDER BY created_at DESC"#,
        )
        .bind(backup_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_keys_for_key(
        &self,
        key_id: Uuid,
    ) -> Result<Vec<BackupEncryptionKey>, sqlx::Error> {
        let rows = sqlx::query_as::<_, BackupEncryptionKeyRow>(
            r#"SELECT id, backup_id, key_id, encrypted_key, created_at
             FROM backup_encryption_keys WHERE key_id = $1 ORDER BY created_at DESC"#,
        )
        .bind(key_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn delete_key(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM backup_encryption_keys WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_keys_for_backup(
        &self,
        backup_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let result =
            sqlx::query("DELETE FROM backup_encryption_keys WHERE backup_id = $1")
                .bind(backup_id)
                .execute(&self.pool)
                .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_serialization() {
        let key = BackupEncryptionKey {
            id: Uuid::new_v4(),
            backup_id: Uuid::new_v4(),
            key_id: Uuid::new_v4(),
            encrypted_key: vec![1, 2, 3, 4],
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&key).unwrap();
        assert!(json.contains("backup_id"));
        assert!(json.contains("key_id"));
    }
}
