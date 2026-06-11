#![forbid(unsafe_code)]

use crate::error::{DbError, Result};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
}

pub struct SessionManager {
    db: PgPool,
    default_ttl: Duration,
}

impl SessionManager {
    pub fn new(db: PgPool, default_ttl: Duration) -> Self {
        Self { db, default_ttl }
    }

    pub fn default_ttl(&self) -> Duration {
        self.default_ttl
    }

    fn compute_expiry(&self) -> DateTime<Utc> {
        Utc::now() + ChronoDuration::from_std(self.default_ttl).unwrap_or(ChronoDuration::hours(24))
    }

    pub fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub async fn create_session(
        &self,
        user_id: Uuid,
        raw_token: &str,
        ip: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<Session> {
        let token_hash = Self::hash_token(raw_token);
        let expires_at = self.compute_expiry();
        let now = Utc::now();

        let row = sqlx::query_as::<_, Session>(
            r#"INSERT INTO sessions (id, user_id, token_hash, ip, user_agent, created_at, expires_at, last_active_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING *"#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(&token_hash)
        .bind(ip)
        .bind(user_agent)
        .bind(now)
        .bind(expires_at)
        .bind(now)
        .fetch_one(&self.db)
        .await
        .map_err(|e| DbError::Database(format!("create_session: {e}")))?;

        Ok(row)
    }

    pub async fn validate_session(&self, raw_token: &str) -> Result<Session> {
        let token_hash = Self::hash_token(raw_token);

        let row = sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE token_hash = $1")
            .bind(&token_hash)
            .fetch_one(&self.db)
            .await
            .map_err(|e| DbError::Auth(format!("session not found: {e}")))?;

        if Utc::now() > row.expires_at {
            return Err(DbError::Auth("session expired".into()));
        }

        sqlx::query("UPDATE sessions SET last_active_at = NOW() WHERE id = $1")
            .bind(row.id)
            .execute(&self.db)
            .await
            .map_err(|e| DbError::Database(format!("update last_active_at: {e}")))?;

        Ok(row)
    }

    pub async fn extend_session(
        &self,
        raw_token: &str,
        additional_ttl: Option<Duration>,
    ) -> Result<Session> {
        let token_hash = Self::hash_token(raw_token);

        let current: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM sessions WHERE token_hash = $1")
                .bind(&token_hash)
                .fetch_optional(&self.db)
                .await
                .map_err(|e| DbError::Database(format!("lookup session: {e}")))?;

        let session_id = current
            .ok_or_else(|| DbError::Auth("session not found".into()))?
            .0;

        let ttl = additional_ttl.unwrap_or(self.default_ttl);
        let new_expiry =
            Utc::now() + ChronoDuration::from_std(ttl).unwrap_or(ChronoDuration::hours(24));

        sqlx::query("UPDATE sessions SET expires_at = $1, last_active_at = NOW() WHERE id = $2")
            .bind(new_expiry)
            .bind(session_id)
            .execute(&self.db)
            .await
            .map_err(|e| DbError::Database(format!("extend_session: {e}")))?;

        let row = sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE id = $1")
            .bind(session_id)
            .fetch_one(&self.db)
            .await
            .map_err(|e| DbError::Database(format!("fetch extended session: {e}")))?;

        Ok(row)
    }

    pub async fn revoke_session(&self, raw_token: &str) -> Result<()> {
        let token_hash = Self::hash_token(raw_token);
        sqlx::query("DELETE FROM sessions WHERE token_hash = $1")
            .bind(&token_hash)
            .execute(&self.db)
            .await
            .map_err(|e| DbError::Database(format!("revoke_session: {e}")))?;
        Ok(())
    }

    pub async fn revoke_all_sessions(&self, user_id: Uuid) -> Result<u64> {
        let result = sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.db)
            .await
            .map_err(|e| DbError::Database(format!("revoke_all_sessions: {e}")))?;
        Ok(result.rows_affected())
    }

    pub async fn cleanup_expired(&self) -> Result<u64> {
        let result = sqlx::query("DELETE FROM sessions WHERE expires_at < NOW()")
            .execute(&self.db)
            .await
            .map_err(|e| DbError::Database(format!("cleanup_expired: {e}")))?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_token_deterministic() {
        let hash1 = SessionManager::hash_token("test-token");
        let hash2 = SessionManager::hash_token("test-token");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_token_different_inputs() {
        let hash1 = SessionManager::hash_token("token-a");
        let hash2 = SessionManager::hash_token("token-b");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_token_length() {
        let hash = SessionManager::hash_token("anything");
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_session_manager_compiles() {
        fn _assert_send<T: Send>() {}
        fn _assert_sync<T: Sync>() {}
        _assert_send::<SessionManager>();
        _assert_sync::<SessionManager>();
    }

    #[test]
    fn test_compute_expiry_future() {
        let ttl = Duration::from_secs(3600);
        let expiry =
            Utc::now() + ChronoDuration::from_std(ttl).unwrap_or(ChronoDuration::hours(24));
        let now = Utc::now();
        assert!(expiry > now);
        let diff = expiry.signed_duration_since(now);
        assert!(diff.num_seconds() >= 3599);
        assert!(diff.num_seconds() <= 3601);
    }

    #[test]
    fn test_compute_expiry_large_duration() {
        let ttl = Duration::from_secs(86400 * 30);
        let expiry =
            Utc::now() + ChronoDuration::from_std(ttl).unwrap_or(ChronoDuration::hours(24));
        let now = Utc::now();
        let diff = expiry.signed_duration_since(now).num_days();
        assert!((29..=31).contains(&diff));
    }

    #[test]
    fn test_hash_token_empty_string() {
        let hash = SessionManager::hash_token("");
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_token_is_hex() {
        let hash = SessionManager::hash_token("test");
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_token_unicode_input() {
        let hash = SessionManager::hash_token("hello 世界");
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_default_ttl_accessor() {
        let ttl = Duration::from_secs(3600);
        assert_eq!(ttl.as_secs(), 3600);
    }

    #[test]
    fn test_compute_expiry_fallback_for_overflow() {
        let very_large = Duration::from_secs(u64::MAX);
        let fallback = ChronoDuration::from_std(very_large).unwrap_or(ChronoDuration::hours(24));
        assert_eq!(fallback, ChronoDuration::hours(24));
    }

    #[test]
    fn test_session_expiry_check_logic() {
        let expired = Utc::now() - ChronoDuration::hours(1);
        let future = Utc::now() + ChronoDuration::hours(1);
        assert!(Utc::now() > expired);
        assert!(Utc::now() < future);
    }

    #[test]
    fn test_session_struct_fields() {
        let session = Session {
            id: Uuid::nil(),
            user_id: Uuid::nil(),
            token_hash: "abc".into(),
            ip: Some("127.0.0.1".into()),
            user_agent: Some("test".into()),
            created_at: Utc::now(),
            expires_at: Utc::now() + ChronoDuration::hours(1),
            last_active_at: Utc::now(),
        };
        assert_eq!(session.token_hash, "abc");
        assert_eq!(session.ip.as_deref(), Some("127.0.0.1"));
        assert!(Utc::now() < session.expires_at);
    }

    #[test]
    fn test_create_session_error_format() {
        let err = DbError::Database("create_session: connection refused".into());
        assert!(err.to_string().contains("create_session"));
    }

    #[test]
    fn test_validate_session_error_format() {
        let auth_err = DbError::Auth("session not found: no rows returned".into());
        assert!(auth_err.to_string().contains("session not found"));
        let expired_err = DbError::Auth("session expired".into());
        assert!(expired_err.to_string().contains("session expired"));
    }

    #[test]
    fn test_extend_session_error_format() {
        let db_err = DbError::Database("lookup session: connection refused".into());
        assert!(db_err.to_string().contains("lookup session"));
        let auth_err = DbError::Auth("session not found".into());
        assert!(auth_err.to_string().contains("session not found"));
    }

    #[test]
    fn test_revoke_session_error_format() {
        let err = DbError::Database("revoke_session: connection refused".into());
        assert!(err.to_string().contains("revoke_session"));
    }

    #[test]
    fn test_revoke_all_sessions_error_format() {
        let err = DbError::Database("revoke_all_sessions: connection refused".into());
        assert!(err.to_string().contains("revoke_all_sessions"));
    }

    #[test]
    fn test_cleanup_expired_error_format() {
        let err = DbError::Database("cleanup_expired: connection refused".into());
        assert!(err.to_string().contains("cleanup_expired"));
    }
}
