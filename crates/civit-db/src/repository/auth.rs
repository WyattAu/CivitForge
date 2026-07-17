#![forbid(unsafe_code)]
#![allow(clippy::too_many_arguments)]

use crate::error::{DbError, Result};
use crate::models::EmailVerificationCode;
use chrono::{DateTime, Utc};
use sqlx::postgres::PgPool;
use serde_json;
use uuid::Uuid;

impl super::DbRepository {
    // --- Access Tokens ---

    pub async fn create_access_token(
        &self,
        user_id: Uuid,
        name: &str,
        token_hash: &str,
        scopes: &[String],
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Uuid> {
        let row: (Uuid,) = sqlx::query_as(
            r#"INSERT INTO access_tokens (user_id, name, token_hash, scopes, expires_at)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING id"#,
        )
        .bind(user_id)
        .bind(name)
        .bind(token_hash)
        .bind(scopes)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_access_token: {e}")))?;
        Ok(row.0)
    }

    pub async fn validate_access_token(&self, token_hash: &str) -> Result<Uuid> {
        let row: (Uuid, Option<DateTime<Utc>>) = sqlx::query_as(
            r#"SELECT user_id, expires_at FROM access_tokens
               WHERE token_hash = $1"#,
        )
        .bind(token_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("validate_access_token: {e}")))?;

        if let Some(exp) = row.1
            && Utc::now() > exp
        {
            return Err(DbError::Auth("access token expired".into()));
        }

        Ok(row.0)
    }

    pub async fn revoke_access_token(&self, token_hash: &str) -> Result<()> {
        sqlx::query("DELETE FROM access_tokens WHERE token_hash = $1")
            .bind(token_hash)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("revoke_access_token: {e}")))?;
        Ok(())
    }

    pub async fn validate_pat_token(&self, token_hash: &str) -> Result<(Uuid, Vec<String>, Uuid)> {
        let row: (Uuid, serde_json::Value, Uuid, Option<DateTime<Utc>>) = sqlx::query_as(
            r#"SELECT user_id, scopes, id, expires_at FROM access_tokens
               WHERE token_hash = $1"#,
        )
        .bind(token_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("validate_pat_token: {e}")))?;

        if let Some(exp) = row.3
            && Utc::now() > exp
        {
            return Err(DbError::Auth("access token expired".into()));
        }

        let scopes: Vec<String> = serde_json::from_value(row.1).unwrap_or_default();

        Ok((row.0, scopes, row.2))
    }

    pub async fn touch_access_token(&self, token_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE access_tokens SET last_used_at = NOW() WHERE id = $1")
            .bind(token_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("touch_access_token: {e}")))?;
        Ok(())
    }

    pub async fn set_email_verified(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE users SET email_verified = true, updated_at = NOW() WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("set_email_verified: {e}")))?;
        Ok(())
    }

    pub async fn store_verification_code(
        &self,
        user_id: Uuid,
        email: &str,
        code: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<EmailVerificationCode> {
        let row = sqlx::query_as::<_, EmailVerificationCode>(
            r#"INSERT INTO email_verification_codes (user_id, email, code, expires_at)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(email)
        .bind(code)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("store_verification_code: {e}")))?;
        Ok(row)
    }

    pub async fn validate_verification_code(
        &self,
        email: &str,
        code: &str,
    ) -> Result<EmailVerificationCode> {
        let row = sqlx::query_as::<_, EmailVerificationCode>(
            r#"SELECT * FROM email_verification_codes
               WHERE email = $1 AND code = $2 AND used = false
               ORDER BY created_at DESC LIMIT 1"#,
        )
        .bind(email)
        .bind(code)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("validate_verification_code: {e}")))?;

        if Utc::now() > row.expires_at {
            return Err(DbError::Auth("verification code expired".into()));
        }

        Ok(row)
    }

    pub async fn mark_verification_code_used(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE email_verification_codes SET used = true WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("mark_verification_code_used: {e}")))?;
        Ok(())
    }

    // --- Audit Events ---

    #[allow(clippy::too_many_arguments)]
    pub async fn record_audit_event(
        &self,
        actor_id: Uuid,
        action: &str,
        resource_type: &str,
        resource_id: Option<Uuid>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        outcome: &str,
    ) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            r#"INSERT INTO audit_events (actor_id, action, resource_type, resource_id, ip_address, user_agent, outcome)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING id"#,
        )
        .bind(actor_id)
        .bind(action)
        .bind(resource_type)
        .bind(resource_id)
        .bind(ip_address)
        .bind(user_agent)
        .bind(outcome)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_audit_event: {e}")))?;
        Ok(row.0)
    }

    pub async fn query_audit_events(
        &self,
        actor_id: Option<Uuid>,
        resource_type: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<(i64, Uuid, String, String, Option<Uuid>, DateTime<Utc>)>> {
        let rows = sqlx::query_as(
            r#"SELECT id, actor_id, action, resource_type, resource_id, created_at
               FROM audit_events
               WHERE ($1::uuid IS NULL OR actor_id = $1)
                 AND ($2::varchar IS NULL OR resource_type = $2)
               ORDER BY created_at DESC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(actor_id)
        .bind(resource_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("query_audit_events: {e}")))?;
        Ok(rows)
    }


}
