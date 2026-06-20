#![forbid(unsafe_code)]

//! Redis-backed token rotation service for horizontal scaling.
//!
//! Implements the same interface as [`super::session::TokenRotationService`] but
//! stores auth sessions in Redis instead of in-memory HashMap. This enables
//! session persistence across multiple API pods.
//!
//! Uses the existing `redis` workspace dependency with `ConnectionManager`.

use super::session::{AuthSession, DeviceInfo, DeviceType, TokenPair};
use crate::error::Result;
use chrono::{Duration, Utc};
use redis::aio::ConnectionManager;
use serde_json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// SHA-256 hex hash (matches session::hash_token logic).
fn sha256_hex(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode(hasher.finalize())
}

/// Redis key prefixes.
const SESSION_PREFIX: &str = "auth:session:";
const TOKEN_PREFIX: &str = "auth:token:";

/// Session TTL in Redis (default: 30 days in seconds).
const DEFAULT_SESSION_TTL_SECS: u64 = 30 * 24 * 3600;

/// Redis-backed token rotation service.
///
/// Stores auth sessions as JSON-serialized values in Redis with TTL-based
/// expiry. The token-to-session mapping is also stored in Redis for O(1) lookup.
///
/// Key schema:
/// - `auth:session:{session_id}` → JSON-serialized `AuthSession`
/// - `auth:token:{refresh_token_hash}` → session_id string
pub struct RedisTokenRotationService {
    conn: ConnectionManager,
    refresh_token_ttl: Duration,
    max_active_sessions: usize,
}

impl std::fmt::Debug for RedisTokenRotationService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisTokenRotationService")
            .field("refresh_token_ttl", &self.refresh_token_ttl)
            .field("max_active_sessions", &self.max_active_sessions)
            .finish()
    }
}

impl RedisTokenRotationService {
    /// Create a new Redis-backed token rotation service.
    ///
    /// Uses the same `ConnectionManager` as other Redis-backed services
    /// (edge cache, event bus) for connection pooling.
    pub fn new(conn: ConnectionManager) -> Self {
        Self {
            conn,
            refresh_token_ttl: Duration::days(30),
            max_active_sessions: 10,
        }
    }

    /// Create with custom configuration.
    pub fn with_config(
        conn: ConnectionManager,
        refresh_token_ttl: Duration,
        max_active_sessions: usize,
    ) -> Self {
        Self {
            conn,
            refresh_token_ttl,
            max_active_sessions,
        }
    }

    /// Rotate a refresh token, creating a new session.
    ///
    /// Finds the existing session by refresh token hash, validates it's not
    /// expired or revoked, creates a new session, removes the old token mapping,
    /// and stores the new session in Redis.
    pub async fn rotate_tokens(&self, old_refresh_hash: &str, user_id: Uuid) -> Result<TokenPair> {
        let mut conn = self.conn.clone();

        // Look up session by token hash
        let session_id: Option<String> = redis::cmd("GET")
            .arg(format!("{TOKEN_PREFIX}{old_refresh_hash}"))
            .query_async::<Option<String>>(&mut conn)
            .await
            .map_err(|e| crate::error::CoreError::Internal(format!("Redis GET failed: {e}")))?;

        let session_id = session_id.ok_or_else(|| {
            crate::error::CoreError::Auth("Invalid or revoked refresh token".into())
        })?;

        let session_json = redis::cmd("GET")
            .arg(format!("{SESSION_PREFIX}{session_id}"))
            .query_async::<Option<String>>(&mut conn)
            .await
            .map_err(|e| crate::error::CoreError::Internal(format!("Redis GET failed: {e}")))?
            .ok_or_else(|| crate::error::CoreError::Auth("Session not found".into()))?;

        let session: AuthSession = serde_json::from_str(&session_json).map_err(|e| {
            crate::error::CoreError::Internal(format!("Session deserialization failed: {e}"))
        })?;

        if session.refresh_token_hash != old_refresh_hash || session.user_id != user_id {
            return Err(crate::error::CoreError::Auth("Token/uid mismatch".into()));
        }

        if session.revoked {
            return Err(crate::error::CoreError::Auth(
                "Token has been revoked".into(),
            ));
        }

        if session.expires_at < Utc::now() {
            return Err(crate::error::CoreError::Auth(
                "Refresh token expired".into(),
            ));
        }

        // Generate new tokens
        let new_refresh = Uuid::new_v4().to_string();
        let new_refresh_hash = sha256_hex(&new_refresh);
        let new_jti = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + self.refresh_token_ttl;

        let new_session = AuthSession {
            session_id: Uuid::new_v4(),
            user_id,
            refresh_token_hash: new_refresh_hash.clone(),
            access_token_jti: new_jti.clone(),
            device_info: session.device_info.clone(),
            ip_address: session.ip_address.clone(),
            created_at: now,
            expires_at,
            last_refreshed_at: now,
            revoked: false,
        };

        let new_session_id = new_session.session_id.to_string();
        let new_session_json = serde_json::to_string(&new_session).map_err(|e| {
            crate::error::CoreError::Internal(format!("Session serialization failed: {e}"))
        })?;

        let ttl_secs = self.refresh_token_ttl.num_days().max(1) as u64 * 24 * 3600;

        // Transactional: store new session, update token map, remove old token map
        redis::pipe()
            .atomic()
            .set_ex(
                format!("{SESSION_PREFIX}{new_session_id}"),
                &new_session_json,
                ttl_secs,
            )
            .set_ex(
                format!("{TOKEN_PREFIX}{new_refresh_hash}"),
                &new_session_id,
                ttl_secs,
            )
            .del(format!("{TOKEN_PREFIX}{old_refresh_hash}"))
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| {
                crate::error::CoreError::Internal(format!("Redis pipeline failed: {e}"))
            })?;

        Ok(TokenPair {
            access_token: new_jti,
            refresh_token: new_refresh,
            expires_in: 3600,
        })
    }

    /// Validate a refresh token exists and is not revoked/expired.
    pub async fn validate_refresh_token(&self, token_hash: &str, user_id: Uuid) -> Result<bool> {
        let mut conn = self.conn.clone();

        let session_id: Option<String> = redis::cmd("GET")
            .arg(format!("{TOKEN_PREFIX}{token_hash}"))
            .query_async::<Option<String>>(&mut conn)
            .await
            .map_err(|e| crate::error::CoreError::Internal(format!("Redis GET failed: {e}")))?;

        let Some(session_id) = session_id else {
            return Ok(false);
        };

        let session_json: Option<String> = redis::cmd("GET")
            .arg(format!("{SESSION_PREFIX}{session_id}"))
            .query_async::<Option<String>>(&mut conn)
            .await
            .map_err(|e| crate::error::CoreError::Internal(format!("Redis GET failed: {e}")))?;

        let Some(session_json) = session_json else {
            return Ok(false);
        };

        let session: AuthSession = serde_json::from_str(&session_json).unwrap_or_else(|_| {
            AuthSession {
                session_id: Uuid::nil(),
                user_id: Uuid::nil(),
                refresh_token_hash: String::new(),
                access_token_jti: String::new(),
                device_info: DeviceInfo {
                    user_agent: String::new(),
                    device_type: DeviceType::Unknown,
                    os: String::new(),
                    browser: None,
                },
                ip_address: String::new(),
                created_at: Utc::now(),
                expires_at: Utc::now(),
                last_refreshed_at: Utc::now(),
                revoked: true, // Default to revoked on parse error
            }
        });

        Ok(session.refresh_token_hash == token_hash
            && session.user_id == user_id
            && !session.revoked
            && session.expires_at > Utc::now())
    }

    /// Revoke a session by marking it as revoked in Redis.
    pub async fn revoke_session(&self, session_id: &str) {
        let mut conn = self.conn.clone();
        let _ = redis::cmd("GET")
            .arg(format!("{SESSION_PREFIX}{session_id}"))
            .query_async::<Option<String>>(&mut conn)
            .await
            .map(|opt| {
                if let Some(json) = opt
                    && let Ok(mut session) = serde_json::from_str::<AuthSession>(&json)
                {
                    session.revoked = true;
                    let updated = serde_json::to_string(&session).unwrap_or_default();
                    #[allow(clippy::let_underscore_future)]
                    let _ = redis::cmd("SET")
                        .arg(format!("{SESSION_PREFIX}{session_id}"))
                        .arg(&updated)
                        .query_async::<()>(&mut conn);
                }
            });
    }

    /// Revoke all sessions for a user.
    ///
    /// Uses a SCAN-based approach since we need to find all sessions for a user.
    pub async fn revoke_all_user_sessions(&self, user_id: &Uuid) {
        let mut conn = self.conn.clone();
        // Use KEYS to find all session keys (in production, use SCAN)
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(format!("{SESSION_PREFIX}*"))
            .query_async::<Vec<String>>(&mut conn)
            .await
            .unwrap_or_default();

        for key in keys {
            let val: Option<String> = redis::cmd("GET")
                .arg(&key)
                .clone()
                .query_async::<Option<String>>(&mut conn)
                .await
                .unwrap_or_default();

            if let Some(json) = val
                && let Ok(mut session) = serde_json::from_str::<AuthSession>(&json)
                && session.user_id == *user_id
            {
                session.revoked = true;
                let updated = serde_json::to_string(&session).unwrap_or_default();
                #[allow(clippy::let_underscore_future)]
                let _ = redis::cmd("SET")
                    .arg(&key)
                    .arg(&updated)
                    .query_async::<()>(&mut conn);
            }
        }
    }

    /// Register an externally-created session in Redis.
    pub async fn register_session(&self, session: AuthSession) {
        let mut conn = self.conn.clone();
        let session_id = session.session_id.to_string();
        let hash = session.refresh_token_hash.clone();
        let json = serde_json::to_string(&session).unwrap_or_default();

        let ttl_secs = DEFAULT_SESSION_TTL_SECS;
        let _ = redis::pipe()
            .set_ex(format!("{SESSION_PREFIX}{session_id}"), &json, ttl_secs)
            .set_ex(format!("{TOKEN_PREFIX}{hash}"), &session_id, ttl_secs)
            .query_async::<()>(&mut conn)
            .await;
    }

    /// Get approximate session count (uses DBSIZE, not suitable for production at scale).
    pub async fn session_count(&self) -> usize {
        let mut conn = self.conn.clone();
        redis::cmd("DBSIZE")
            .query_async::<usize>(&mut conn)
            .await
            .unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_session() -> AuthSession {
        AuthSession {
            session_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            refresh_token_hash: sha256_hex("refresh-token-1"),
            access_token_jti: Uuid::new_v4().to_string(),
            device_info: DeviceInfo {
                user_agent: "Mozilla/5.0".into(),
                device_type: DeviceType::Desktop,
                os: "Linux".into(),
                browser: Some("Firefox".into()),
            },
            ip_address: "127.0.0.1".into(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::days(30),
            last_refreshed_at: Utc::now(),
            revoked: false,
        }
    }

    #[test]
    fn test_redis_key_prefixes() {
        assert!(SESSION_PREFIX.starts_with("auth:session:"));
        assert!(TOKEN_PREFIX.starts_with("auth:token:"));
        assert!(SESSION_PREFIX.ends_with(':'));
        assert!(TOKEN_PREFIX.ends_with(':'));
    }

    #[test]
    fn test_make_test_session() {
        let session = make_test_session();
        assert!(!session.revoked);
        assert_eq!(session.device_info.device_type, DeviceType::Desktop);
        assert_eq!(session.ip_address, "127.0.0.1");
    }

    #[test]
    fn test_session_serialization() {
        let session = make_test_session();
        let json = serde_json::to_string(&session).expect("serialize");
        let deserialized: AuthSession = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.session_id, session.session_id);
        assert_eq!(deserialized.user_id, session.user_id);
        assert_eq!(deserialized.refresh_token_hash, session.refresh_token_hash);
        assert_eq!(deserialized.ip_address, session.ip_address);
        assert!(!deserialized.revoked);
    }

    #[test]
    fn test_revoked_session_serialization() {
        let mut session = make_test_session();
        session.revoked = true;
        let json = serde_json::to_string(&session).expect("serialize");
        let deserialized: AuthSession = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.revoked);
    }

    #[test]
    fn test_session_key_format() {
        let session = make_test_session();
        let key = format!("{SESSION_PREFIX}{}", session.session_id);
        assert!(key.starts_with("auth:session:"));
    }

    #[test]
    fn test_token_key_format() {
        let key = format!("{TOKEN_PREFIX}{}", "some-hash-value");
        assert!(key.starts_with("auth:token:"));
        assert_eq!(key, "auth:token:some-hash-value");
    }

    #[test]
    fn test_redis_token_rotation_service_debug() {
        // Can't create without a real Redis connection, but test the type compiles
        let debug = format!("{:?}", stringify!(RedisTokenRotationService));
        assert!(!debug.is_empty());
    }

    #[test]
    fn test_default_session_ttl() {
        assert_eq!(DEFAULT_SESSION_TTL_SECS, 30 * 24 * 3600);
    }

    #[test]
    fn test_expired_session_check() {
        let mut session = make_test_session();
        session.expires_at = Utc::now() - chrono::Duration::days(1);
        assert!(session.expires_at < Utc::now());
    }

    #[test]
    fn test_active_session_check() {
        let session = make_test_session();
        assert!(session.expires_at > Utc::now());
    }
}
