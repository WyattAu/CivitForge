#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectStorageBucket {
    pub id: Uuid,
    pub name: String,
    pub region: String,
    pub endpoint: Option<String>,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateObjectStorageBucket {
    pub name: String,
    pub region: Option<String>,
    pub endpoint: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateObjectStorageBucket {
    pub name: Option<String>,
    pub region: Option<String>,
    pub endpoint: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectStorageObject {
    pub id: Uuid,
    pub bucket_id: Uuid,
    pub key: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub etag: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateObjectStorageObject {
    pub bucket_id: Uuid,
    pub key: String,
    pub content_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub etag: String,
}

#[derive(Debug, sqlx::FromRow)]
struct ObjectStorageBucketRow {
    id: Uuid,
    name: String,
    region: String,
    endpoint: Option<String>,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<ObjectStorageBucketRow> for ObjectStorageBucket {
    fn from(row: ObjectStorageBucketRow) -> Self {
        ObjectStorageBucket {
            id: row.id,
            name: row.name,
            region: row.region,
            endpoint: row.endpoint,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ObjectStorageObjectRow {
    id: Uuid,
    bucket_id: Uuid,
    key: String,
    content_type: String,
    size_bytes: i64,
    etag: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<ObjectStorageObjectRow> for ObjectStorageObject {
    fn from(row: ObjectStorageObjectRow) -> Self {
        ObjectStorageObject {
            id: row.id,
            bucket_id: row.bucket_id,
            key: row.key,
            content_type: row.content_type,
            size_bytes: row.size_bytes,
            etag: row.etag,
            created_at: row.created_at,
        }
    }
}

pub struct ObjectStorageService {
    pool: PgPool,
}

impl ObjectStorageService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_bucket(
        &self,
        input: CreateObjectStorageBucket,
    ) -> Result<ObjectStorageBucket, sqlx::Error> {
        let row = sqlx::query_as::<_, ObjectStorageBucketRow>(
            r#"INSERT INTO object_storage_buckets (name, region, endpoint, enabled)
             VALUES ($1, $2, $3, $4)
             RETURNING id, name, region, endpoint, enabled, created_at"#,
        )
        .bind(&input.name)
        .bind(input.region.as_deref().unwrap_or("us-east-1"))
        .bind(&input.endpoint)
        .bind(input.enabled.unwrap_or(true))
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_bucket(
        &self,
        id: Uuid,
    ) -> Result<Option<ObjectStorageBucket>, sqlx::Error> {
        let row = sqlx::query_as::<_, ObjectStorageBucketRow>(
            r#"SELECT id, name, region, endpoint, enabled, created_at
             FROM object_storage_buckets WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_buckets(&self) -> Result<Vec<ObjectStorageBucket>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ObjectStorageBucketRow>(
            r#"SELECT id, name, region, endpoint, enabled, created_at
             FROM object_storage_buckets ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_bucket(
        &self,
        id: Uuid,
        input: UpdateObjectStorageBucket,
    ) -> Result<ObjectStorageBucket, sqlx::Error> {
        let row = sqlx::query_as::<_, ObjectStorageBucketRow>(
            r#"UPDATE object_storage_buckets SET
             name = COALESCE($2, name),
             region = COALESCE($3, region),
             endpoint = COALESCE($4, endpoint),
             enabled = COALESCE($5, enabled)
             WHERE id = $1
             RETURNING id, name, region, endpoint, enabled, created_at"#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.region)
        .bind(&input.endpoint)
        .bind(input.enabled)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_bucket(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM object_storage_buckets WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn put_object(
        &self,
        input: CreateObjectStorageObject,
    ) -> Result<ObjectStorageObject, sqlx::Error> {
        let row = sqlx::query_as::<_, ObjectStorageObjectRow>(
            r#"INSERT INTO object_storage_objects (bucket_id, key, content_type, size_bytes, etag)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, bucket_id, key, content_type, size_bytes, etag, created_at"#,
        )
        .bind(input.bucket_id)
        .bind(&input.key)
        .bind(input.content_type.as_deref().unwrap_or("application/octet-stream"))
        .bind(input.size_bytes.unwrap_or(0))
        .bind(&input.etag)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_object(
        &self,
        bucket_id: Uuid,
        key: &str,
    ) -> Result<Option<ObjectStorageObject>, sqlx::Error> {
        let row = sqlx::query_as::<_, ObjectStorageObjectRow>(
            r#"SELECT id, bucket_id, key, content_type, size_bytes, etag, created_at
             FROM object_storage_objects WHERE bucket_id = $1 AND key = $2"#,
        )
        .bind(bucket_id)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_objects(
        &self,
        bucket_id: Uuid,
    ) -> Result<Vec<ObjectStorageObject>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ObjectStorageObjectRow>(
            r#"SELECT id, bucket_id, key, content_type, size_bytes, etag, created_at
             FROM object_storage_objects WHERE bucket_id = $1 ORDER BY created_at DESC"#,
        )
        .bind(bucket_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn delete_object(
        &self,
        bucket_id: Uuid,
        key: &str,
    ) -> Result<bool, sqlx::Error> {
        let result =
            sqlx::query("DELETE FROM object_storage_objects WHERE bucket_id = $1 AND key = $2")
                .bind(bucket_id)
                .bind(key)
                .execute(&self.pool)
                .await?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bucket_serialization() {
        let bucket = ObjectStorageBucket {
            id: Uuid::new_v4(),
            name: "my-bucket".into(),
            region: "us-east-1".into(),
            endpoint: None,
            enabled: true,
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&bucket).unwrap();
        assert!(json.contains("my-bucket"));
        assert!(json.contains("us-east-1"));
    }

    #[test]
    fn test_object_serialization() {
        let obj = ObjectStorageObject {
            id: Uuid::new_v4(),
            bucket_id: Uuid::new_v4(),
            key: "path/to/file.bin".into(),
            content_type: "application/octet-stream".into(),
            size_bytes: 1024,
            etag: "abc123".into(),
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&obj).unwrap();
        assert!(json.contains("path/to/file.bin"));
        assert!(json.contains("abc123"));
    }
}
