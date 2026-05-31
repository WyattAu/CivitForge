#![forbid(unsafe_code)]

use crate::error::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    Desktop,
    Mobile,
    Tablet,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub user_agent: String,
    pub device_type: DeviceType,
    pub os: String,
    pub browser: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub refresh_token_hash: String,
    pub access_token_jti: String,
    pub device_info: DeviceInfo,
    pub ip_address: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_refreshed_at: DateTime<Utc>,
    pub revoked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

pub struct TokenRotationService {
    pub refresh_token_ttl: Duration,
    pub max_active_sessions: usize,
    sessions: Mutex<HashMap<String, AuthSession>>,
    token_session_map: Mutex<HashMap<String, String>>,
}

impl TokenRotationService {
    pub fn new() -> Self {
        Self {
            refresh_token_ttl: Duration::days(30),
            max_active_sessions: 10,
            sessions: Mutex::new(HashMap::new()),
            token_session_map: Mutex::new(HashMap::new()),
        }
    }

    pub fn with_config(refresh_token_ttl: Duration, max_active_sessions: usize) -> Self {
        Self {
            refresh_token_ttl,
            max_active_sessions,
            sessions: Mutex::new(HashMap::new()),
            token_session_map: Mutex::new(HashMap::new()),
        }
    }

    pub fn rotate_tokens(&self, old_refresh_hash: &str, user_id: Uuid) -> Result<TokenPair> {
        let sessions = self
            .sessions
            .lock()
            .map_err(|e| crate::error::CoreError::Internal(format!("Lock poisoned: {e}")))?;

        let session = sessions
            .values()
            .find(|s| {
                s.refresh_token_hash == old_refresh_hash && s.user_id == user_id && !s.revoked
            })
            .ok_or_else(|| {
                crate::error::CoreError::Auth("Invalid or revoked refresh token".into())
            })?;

        if session.expires_at < Utc::now() {
            return Err(crate::error::CoreError::Auth(
                "Refresh token expired".into(),
            ));
        }

        drop(sessions);

        let new_refresh = Uuid::new_v4().to_string();
        let new_refresh_hash = hash_token(&new_refresh);
        let new_jti = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + self.refresh_token_ttl;

        let new_session = AuthSession {
            session_id: Uuid::new_v4(),
            user_id,
            refresh_token_hash: new_refresh_hash.clone(),
            access_token_jti: new_jti.clone(),
            device_info: {
                let locked = self.sessions.lock().unwrap();
                sessions::get_existing(&locked, old_refresh_hash)
                    .map(|s| s.device_info.clone())
                    .unwrap_or(DeviceInfo {
                        user_agent: String::new(),
                        device_type: DeviceType::Unknown,
                        os: String::new(),
                        browser: None,
                    })
            },
            ip_address: String::new(),
            created_at: now,
            expires_at,
            last_refreshed_at: now,
            revoked: false,
        };

        let new_session_id = new_session.session_id.to_string();

        {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(new_session_id.clone(), new_session);

            let mut token_map = self.token_session_map.lock().unwrap();
            token_map.remove(old_refresh_hash);
            token_map.insert(new_refresh_hash.clone(), new_session_id.clone());
        }

        Ok(TokenPair {
            access_token: new_jti,
            refresh_token: new_refresh,
            expires_in: 3600,
        })
    }

    pub fn validate_refresh_token(&self, token_hash: &str, user_id: Uuid) -> Result<bool> {
        let sessions = self
            .sessions
            .lock()
            .map_err(|e| crate::error::CoreError::Internal(format!("Lock poisoned: {e}")))?;

        let valid = sessions.values().any(|s| {
            s.refresh_token_hash == token_hash
                && s.user_id == user_id
                && !s.revoked
                && s.expires_at > Utc::now()
        });
        Ok(valid)
    }

    pub fn revoke_session(&self, session_id: &str) {
        let uuid = Uuid::parse_str(session_id).unwrap_or_else(|_| {
            let mut sessions = self.sessions.lock().unwrap();
            if let Some(s) = sessions.get_mut(session_id) {
                s.revoked = true;
            }
            Uuid::nil()
        });
        if !uuid.is_nil() {
            let mut sessions = self.sessions.lock().unwrap();
            if let Some(s) = sessions.get_mut(&uuid.to_string()) {
                s.revoked = true;
            }
        }
    }

    pub fn revoke_all_user_sessions(&self, user_id: &Uuid) {
        let mut sessions = self.sessions.lock().unwrap();
        for session in sessions.values_mut() {
            if session.user_id == *user_id {
                session.revoked = true;
            }
        }
    }

    pub fn list_active_devices(&self, user_id: &Uuid) -> Vec<DeviceInfo> {
        let sessions = self.sessions.lock().unwrap();
        sessions
            .values()
            .filter(|s| s.user_id == *user_id && !s.revoked && s.expires_at > Utc::now())
            .map(|s| s.device_info.clone())
            .collect()
    }

    pub fn detect_device_type(user_agent: &str) -> DeviceType {
        let ua = user_agent.to_lowercase();
        if ua.contains("ipad") || ua.contains("tablet") {
            DeviceType::Tablet
        } else if ua.contains("mobile") || ua.contains("android") || ua.contains("iphone") {
            DeviceType::Mobile
        } else if !ua.is_empty() {
            DeviceType::Desktop
        } else {
            DeviceType::Unknown
        }
    }

    pub fn register_session(&self, session: AuthSession) {
        let session_id = session.session_id.to_string();
        let hash = session.refresh_token_hash.clone();
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session_id.clone(), session);
        let mut token_map = self.token_session_map.lock().unwrap();
        token_map.insert(hash, session_id);
    }

    pub fn session_count(&self) -> usize {
        self.sessions.lock().unwrap().len()
    }
}

impl Default for TokenRotationService {
    fn default() -> Self {
        Self::new()
    }
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

mod sessions {
    use super::AuthSession;

    pub(crate) fn get_existing<'a>(
        sessions: &'a std::collections::HashMap<String, AuthSession>,
        refresh_hash: &str,
    ) -> Option<&'a AuthSession> {
        sessions
            .values()
            .find(|s| s.refresh_token_hash == refresh_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_device_type_desktop() {
        assert_eq!(
            TokenRotationService::detect_device_type(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0"
            ),
            DeviceType::Desktop
        );
    }

    #[test]
    fn test_detect_device_type_mobile() {
        assert_eq!(
            TokenRotationService::detect_device_type(
                "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) Mobile/15E148"
            ),
            DeviceType::Mobile
        );
    }

    #[test]
    fn test_detect_device_type_android_mobile() {
        assert_eq!(
            TokenRotationService::detect_device_type(
                "Mozilla/5.0 (Linux; Android 14) AppleWebKit/537.36 Chrome/120.0 Mobile"
            ),
            DeviceType::Mobile
        );
    }

    #[test]
    fn test_detect_device_type_tablet() {
        assert_eq!(
            TokenRotationService::detect_device_type(
                "Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X) AppleWebKit/605.1.15"
            ),
            DeviceType::Tablet
        );
    }

    #[test]
    fn test_detect_device_type_tablet_generic() {
        assert_eq!(
            TokenRotationService::detect_device_type("Mozilla/5.0 (Linux; Android 14; Tablet)"),
            DeviceType::Tablet
        );
    }

    #[test]
    fn test_detect_device_type_unknown_empty() {
        assert_eq!(
            TokenRotationService::detect_device_type(""),
            DeviceType::Unknown
        );
    }

    #[test]
    fn test_new_service_default() {
        let svc = TokenRotationService::new();
        assert_eq!(svc.session_count(), 0);
    }

    #[test]
    fn test_register_and_list_devices() {
        let svc = TokenRotationService::new();
        let user_id = Uuid::new_v4();
        let session = AuthSession {
            session_id: Uuid::new_v4(),
            user_id,
            refresh_token_hash: hash_token("refresh-token-1"),
            access_token_jti: Uuid::new_v4().to_string(),
            device_info: DeviceInfo {
                user_agent: "Mozilla/5.0 (Windows NT 10.0) Chrome".into(),
                device_type: DeviceType::Desktop,
                os: "Windows".into(),
                browser: Some("Chrome".into()),
            },
            ip_address: "192.168.1.1".into(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(30),
            last_refreshed_at: Utc::now(),
            revoked: false,
        };
        svc.register_session(session);
        let devices = svc.list_active_devices(&user_id);
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].device_type, DeviceType::Desktop);
        assert_eq!(devices[0].browser.as_deref(), Some("Chrome"));
    }

    #[test]
    fn test_revoke_session() {
        let svc = TokenRotationService::new();
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let session = AuthSession {
            session_id,
            user_id,
            refresh_token_hash: hash_token("token-to-revoke"),
            access_token_jti: Uuid::new_v4().to_string(),
            device_info: DeviceInfo {
                user_agent: "Test".into(),
                device_type: DeviceType::Desktop,
                os: "Linux".into(),
                browser: None,
            },
            ip_address: "10.0.0.1".into(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(30),
            last_refreshed_at: Utc::now(),
            revoked: false,
        };
        svc.register_session(session);
        svc.revoke_session(&session_id.to_string());
        let devices = svc.list_active_devices(&user_id);
        assert!(devices.is_empty());
    }

    #[test]
    fn test_revoke_all_user_sessions() {
        let svc = TokenRotationService::new();
        let user_id = Uuid::new_v4();
        let other_user = Uuid::new_v4();

        for i in 0..3 {
            let session = AuthSession {
                session_id: Uuid::new_v4(),
                user_id,
                refresh_token_hash: hash_token(&format!("token-{i}")),
                access_token_jti: Uuid::new_v4().to_string(),
                device_info: DeviceInfo {
                    user_agent: "Test".into(),
                    device_type: DeviceType::Desktop,
                    os: "Linux".into(),
                    browser: None,
                },
                ip_address: "10.0.0.1".into(),
                created_at: Utc::now(),
                expires_at: Utc::now() + Duration::days(30),
                last_refreshed_at: Utc::now(),
                revoked: false,
            };
            svc.register_session(session);
        }

        let other_session = AuthSession {
            session_id: Uuid::new_v4(),
            user_id: other_user,
            refresh_token_hash: hash_token("other-token"),
            access_token_jti: Uuid::new_v4().to_string(),
            device_info: DeviceInfo {
                user_agent: "Test".into(),
                device_type: DeviceType::Desktop,
                os: "Linux".into(),
                browser: None,
            },
            ip_address: "10.0.0.2".into(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(30),
            last_refreshed_at: Utc::now(),
            revoked: false,
        };
        svc.register_session(other_session);

        svc.revoke_all_user_sessions(&user_id);

        assert!(svc.list_active_devices(&user_id).is_empty());
        assert_eq!(svc.list_active_devices(&other_user).len(), 1);
    }

    #[test]
    fn test_rotate_tokens_success() {
        let svc = TokenRotationService::new();
        let user_id = Uuid::new_v4();
        let refresh = Uuid::new_v4().to_string();
        let refresh_hash = hash_token(&refresh);

        let session = AuthSession {
            session_id: Uuid::new_v4(),
            user_id,
            refresh_token_hash: refresh_hash.clone(),
            access_token_jti: Uuid::new_v4().to_string(),
            device_info: DeviceInfo {
                user_agent: "Test".into(),
                device_type: DeviceType::Desktop,
                os: "Linux".into(),
                browser: None,
            },
            ip_address: "127.0.0.1".into(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(30),
            last_refreshed_at: Utc::now(),
            revoked: false,
        };
        svc.register_session(session);

        let pair = svc.rotate_tokens(&refresh_hash, user_id).unwrap();
        assert!(!pair.access_token.is_empty());
        assert!(!pair.refresh_token.is_empty());
        assert_eq!(pair.expires_in, 3600);
    }

    #[test]
    fn test_rotate_tokens_invalid_hash() {
        let svc = TokenRotationService::new();
        let user_id = Uuid::new_v4();
        let result = svc.rotate_tokens("nonexistent-hash", user_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_refresh_token_valid() {
        let svc = TokenRotationService::new();
        let user_id = Uuid::new_v4();
        let refresh = Uuid::new_v4().to_string();
        let refresh_hash = hash_token(&refresh);

        let session = AuthSession {
            session_id: Uuid::new_v4(),
            user_id,
            refresh_token_hash: refresh_hash.clone(),
            access_token_jti: Uuid::new_v4().to_string(),
            device_info: DeviceInfo {
                user_agent: "Test".into(),
                device_type: DeviceType::Desktop,
                os: "Linux".into(),
                browser: None,
            },
            ip_address: "127.0.0.1".into(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(30),
            last_refreshed_at: Utc::now(),
            revoked: false,
        };
        svc.register_session(session);

        assert!(svc.validate_refresh_token(&refresh_hash, user_id).unwrap());
    }

    #[test]
    fn test_validate_refresh_token_wrong_user() {
        let svc = TokenRotationService::new();
        let user_id = Uuid::new_v4();
        let other_user = Uuid::new_v4();
        let refresh_hash = hash_token("some-token");

        let session = AuthSession {
            session_id: Uuid::new_v4(),
            user_id,
            refresh_token_hash: refresh_hash.clone(),
            access_token_jti: Uuid::new_v4().to_string(),
            device_info: DeviceInfo {
                user_agent: "Test".into(),
                device_type: DeviceType::Desktop,
                os: "Linux".into(),
                browser: None,
            },
            ip_address: "127.0.0.1".into(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(30),
            last_refreshed_at: Utc::now(),
            revoked: false,
        };
        svc.register_session(session);

        assert!(
            !svc.validate_refresh_token(&refresh_hash, other_user)
                .unwrap()
        );
    }

    #[test]
    fn test_hash_token_deterministic() {
        let h1 = hash_token("my-token");
        let h2 = hash_token("my-token");
        assert_eq!(h1, h2);
        assert_ne!(h1, hash_token("other-token"));
    }

    #[test]
    fn test_with_config() {
        let svc = TokenRotationService::with_config(Duration::hours(1), 5);
        assert_eq!(svc.max_active_sessions, 5);
    }
}
