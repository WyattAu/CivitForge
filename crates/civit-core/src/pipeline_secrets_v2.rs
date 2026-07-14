//! Pipeline Secrets v2: Advanced secrets management with environment support,
//! rotation tracking, and audit logging.

#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSecretV2 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub description: String,
    pub environment: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSecretV2Request {
    pub name: String,
    pub value: String,
    pub description: Option<String>,
    pub environment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSecretV2Request {
    pub value: Option<String>,
    pub description: Option<Option<String>>,
    pub environment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretRotationLog {
    pub id: Uuid,
    pub secret_id: Uuid,
    pub rotated_by: Option<Uuid>,
    pub rotated_at: DateTime<Utc>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretAccessLog {
    pub id: Uuid,
    pub secret_id: Uuid,
    pub accessed_by: Option<Uuid>,
    pub access_type: String,
    pub accessed_at: DateTime<Utc>,
    pub ip_address: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct SecretRow {
    id: Uuid,
    repo_id: Uuid,
    name: String,
    description: String,
    environment: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<SecretRow> for PipelineSecretV2 {
    fn from(row: SecretRow) -> Self {
        PipelineSecretV2 {
            id: row.id,
            repo_id: row.repo_id,
            name: row.name,
            description: row.description,
            environment: row.environment,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

pub struct PipelineSecretsV2Service {
    pool: PgPool,
}

impl PipelineSecretsV2Service {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_secrets(
        &self,
        repo_id: Uuid,
        environment: Option<&str>,
    ) -> Result<Vec<PipelineSecretV2>, sqlx::Error> {
        let rows = if let Some(env) = environment {
            sqlx::query_as::<_, SecretRow>(
                "SELECT id, repo_id, name, description, environment, created_at, updated_at
                 FROM pipeline_secrets_v2
                 WHERE repo_id = $1 AND environment = $2
                 ORDER BY name",
            )
            .bind(repo_id)
            .bind(env)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, SecretRow>(
                "SELECT id, repo_id, name, description, environment, created_at, updated_at
                 FROM pipeline_secrets_v2
                 WHERE repo_id = $1
                 ORDER BY name",
            )
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_secret(
        &self,
        repo_id: Uuid,
        name: &str,
        environment: &str,
    ) -> Result<Option<PipelineSecretV2>, sqlx::Error> {
        let row = sqlx::query_as::<_, SecretRow>(
            "SELECT id, repo_id, name, description, environment, created_at, updated_at
             FROM pipeline_secrets_v2
             WHERE repo_id = $1 AND name = $2 AND environment = $3",
        )
        .bind(repo_id)
        .bind(name)
        .bind(environment)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn create_secret(
        &self,
        repo_id: Uuid,
        request: CreateSecretV2Request,
        encrypted_value: Vec<u8>,
        created_by: Option<Uuid>,
    ) -> Result<PipelineSecretV2, sqlx::Error> {
        let environment = request.environment.unwrap_or_else(|| "all".to_string());
        let description = request.description.unwrap_or_default();
        let now = Utc::now();

        let row = sqlx::query_as::<_, SecretRow>(
            "INSERT INTO pipeline_secrets_v2 (repo_id, name, encrypted_value, description, environment, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, repo_id, name, description, environment, created_at, updated_at",
        )
        .bind(repo_id)
        .bind(&request.name)
        .bind(&encrypted_value)
        .bind(&description)
        .bind(&environment)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        // Log the creation
        self.log_access(
            row.id,
            created_by,
            "create",
            None,
        )
        .await?;

        Ok(row.into())
    }

    pub async fn update_secret(
        &self,
        repo_id: Uuid,
        name: &str,
        environment: &str,
        request: UpdateSecretV2Request,
        encrypted_value: Option<Vec<u8>>,
        updated_by: Option<Uuid>,
    ) -> Result<PipelineSecretV2, sqlx::Error> {
        let now = Utc::now();

        let row = if let Some(value) = encrypted_value {
            sqlx::query_as::<_, SecretRow>(
                "UPDATE pipeline_secrets_v2
                 SET encrypted_value = $3, description = COALESCE($4, description), updated_at = $5
                 WHERE repo_id = $1 AND name = $2 AND environment = $6
                 RETURNING id, repo_id, name, description, environment, created_at, updated_at",
            )
            .bind(repo_id)
            .bind(name)
            .bind(&value)
            .bind(request.description)
            .bind(now)
            .bind(environment)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, SecretRow>(
                "UPDATE pipeline_secrets_v2
                 SET description = COALESCE($3, description), updated_at = $4
                 WHERE repo_id = $1 AND name = $2 AND environment = $5
                 RETURNING id, repo_id, name, description, environment, created_at, updated_at",
            )
            .bind(repo_id)
            .bind(name)
            .bind(request.description)
            .bind(now)
            .bind(environment)
            .fetch_one(&self.pool)
            .await?
        };

        // Log the update
        self.log_access(
            row.id,
            updated_by,
            "update",
            None,
        )
        .await?;

        Ok(row.into())
    }

    pub async fn delete_secret(
        &self,
        repo_id: Uuid,
        name: &str,
        environment: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM pipeline_secrets_v2 WHERE repo_id = $1 AND name = $2 AND environment = $3",
        )
        .bind(repo_id)
        .bind(name)
        .bind(environment)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn rotate_secret(
        &self,
        repo_id: Uuid,
        name: &str,
        environment: &str,
        new_encrypted_value: Vec<u8>,
        rotated_by: Option<Uuid>,
        reason: &str,
    ) -> Result<PipelineSecretV2, sqlx::Error> {
        let now = Utc::now();

        // Update the secret value
        let row = sqlx::query_as::<_, SecretRow>(
            "UPDATE pipeline_secrets_v2
             SET encrypted_value = $3, updated_at = $4
             WHERE repo_id = $1 AND name = $2 AND environment = $5
             RETURNING id, repo_id, name, description, environment, created_at, updated_at",
        )
        .bind(repo_id)
        .bind(name)
        .bind(&new_encrypted_value)
        .bind(now)
        .bind(environment)
        .fetch_one(&self.pool)
        .await?;

        // Log the rotation
        sqlx::query(
            "INSERT INTO secret_rotation_log (secret_id, rotated_by, rotated_at, reason)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(row.id)
        .bind(rotated_by)
        .bind(now)
        .bind(reason)
        .execute(&self.pool)
        .await?;

        // Log access
        self.log_access(row.id, rotated_by, "rotate", None).await?;

        Ok(row.into())
    }

    pub async fn get_rotation_log(
        &self,
        secret_id: Uuid,
    ) -> Result<Vec<SecretRotationLog>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (Uuid, Uuid, Option<Uuid>, DateTime<Utc>, String)>(
            "SELECT id, secret_id, rotated_by, rotated_at, reason
             FROM secret_rotation_log
             WHERE secret_id = $1
             ORDER BY rotated_at DESC",
        )
        .bind(secret_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, secret_id, rotated_by, rotated_at, reason)| SecretRotationLog {
                id,
                secret_id,
                rotated_by,
                rotated_at,
                reason,
            })
            .collect())
    }

    pub async fn get_access_log(
        &self,
        secret_id: Uuid,
    ) -> Result<Vec<SecretAccessLog>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (Uuid, Uuid, Option<Uuid>, String, DateTime<Utc>, Option<String>)>(
            "SELECT id, secret_id, accessed_by, access_type, accessed_at, ip_address
             FROM secret_access_log
             WHERE secret_id = $1
             ORDER BY accessed_at DESC
             LIMIT 100",
        )
        .bind(secret_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, secret_id, accessed_by, access_type, accessed_at, ip_address)| {
                SecretAccessLog {
                    id,
                    secret_id,
                    accessed_by,
                    access_type,
                    accessed_at,
                    ip_address,
                }
            })
            .collect())
    }

    async fn log_access(
        &self,
        secret_id: Uuid,
        accessed_by: Option<Uuid>,
        access_type: &str,
        ip_address: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO secret_access_log (secret_id, accessed_by, access_type, accessed_at, ip_address)
             VALUES ($1, $2, $3, NOW(), $4)",
        )
        .bind(secret_id)
        .bind(accessed_by)
        .bind(access_type)
        .bind(ip_address)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_secret_v2_serialize() {
        let secret = PipelineSecretV2 {
            id: Uuid::new_v4(),
            repo_id: Uuid::new_v4(),
            name: "TEST_SECRET".to_string(),
            description: "Test secret".to_string(),
            environment: "staging".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&secret).unwrap();
        assert!(json.contains("TEST_SECRET"));
        assert!(json.contains("staging"));
    }

    #[test]
    fn test_create_secret_request_deserialize() {
        let json = r#"{"name": "MY_TOKEN", "value": "abc123", "description": "Token for API", "environment": "production"}"#;
        let req: CreateSecretV2Request = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "MY_TOKEN");
        assert_eq!(req.value, "abc123");
        assert_eq!(req.description.as_deref(), Some("Token for API"));
        assert_eq!(req.environment.as_deref(), Some("production"));
    }

    #[test]
    fn test_secret_rotation_log_serialize() {
        let log = SecretRotationLog {
            id: Uuid::new_v4(),
            secret_id: Uuid::new_v4(),
            rotated_by: Some(Uuid::new_v4()),
            rotated_at: Utc::now(),
            reason: "Security policy".to_string(),
        };
        let json = serde_json::to_string(&log).unwrap();
        assert!(json.contains("Security policy"));
    }

    #[test]
    fn test_secret_access_log_serialize() {
        let log = SecretAccessLog {
            id: Uuid::new_v4(),
            secret_id: Uuid::new_v4(),
            accessed_by: Some(Uuid::new_v4()),
            access_type: "read".to_string(),
            accessed_at: Utc::now(),
            ip_address: Some("192.168.1.1".to_string()),
        };
        let json = serde_json::to_string(&log).unwrap();
        assert!(json.contains("read"));
        assert!(json.contains("192.168.1.1"));
    }
}
