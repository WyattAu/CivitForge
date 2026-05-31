#![forbid(unsafe_code)]

use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRotationConfig {
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,
    pub absolute_max_age: Duration,
    pub rotation_grace_period: Duration,
    pub max_active_sessions: u32,
    pub enable_issuer_check: bool,
}

impl Default for TokenRotationConfig {
    fn default() -> Self {
        Self {
            access_token_ttl: Duration::minutes(15),
            refresh_token_ttl: Duration::days(7),
            absolute_max_age: Duration::days(90),
            rotation_grace_period: Duration::seconds(30),
            max_active_sessions: 10,
            enable_issuer_check: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenValidationResult {
    pub is_valid: bool,
    pub reason: Option<String>,
    pub needs_rotation: bool,
    pub session_count: u32,
}

pub struct TokenRotationService {
    config: TokenRotationConfig,
    active_sessions: std::sync::Mutex<HashMap<String, DateTime<Utc>>>,
}

impl TokenRotationService {
    pub fn new(config: TokenRotationConfig) -> Self {
        Self {
            config,
            active_sessions: std::sync::Mutex::new(HashMap::new()),
        }
    }

    pub fn validate_token(&self, issued_at: DateTime<Utc>, issuer: &str) -> TokenValidationResult {
        let now = Utc::now();

        if self.config.enable_issuer_check && issuer.is_empty() {
            return TokenValidationResult {
                is_valid: false,
                reason: Some("issuer check enabled but issuer is empty".into()),
                needs_rotation: false,
                session_count: 0,
            };
        }

        if now - issued_at > self.config.access_token_ttl {
            return TokenValidationResult {
                is_valid: false,
                reason: Some("token expired".into()),
                needs_rotation: false,
                session_count: 0,
            };
        }

        if now - issued_at > self.config.absolute_max_age {
            return TokenValidationResult {
                is_valid: false,
                reason: Some("token beyond absolute max age".into()),
                needs_rotation: false,
                session_count: 0,
            };
        }

        let rotation_threshold = self.config.access_token_ttl.num_seconds() * 4 / 5;
        let needs_rotation = (now - issued_at).num_seconds() > rotation_threshold;

        let session_count = self.active_sessions.lock().unwrap().len() as u32;

        TokenValidationResult {
            is_valid: true,
            reason: None,
            needs_rotation,
            session_count,
        }
    }

    pub fn register_session(&self, token_id: &str) -> Result<(), String> {
        let mut sessions = self.active_sessions.lock().unwrap();
        if sessions.len() >= self.config.max_active_sessions as usize {
            return Err(format!(
                "max active sessions ({}) exceeded",
                self.config.max_active_sessions
            ));
        }
        sessions.insert(token_id.to_string(), Utc::now());
        Ok(())
    }

    pub fn revoke_session(&self, token_id: &str) -> bool {
        let mut sessions = self.active_sessions.lock().unwrap();
        sessions.remove(token_id).is_some()
    }

    pub fn active_session_count(&self) -> usize {
        self.active_sessions.lock().unwrap().len()
    }

    pub fn cleanup_expired(&self) -> usize {
        let mut sessions = self.active_sessions.lock().unwrap();
        let now = Utc::now();
        let before = sessions.len();
        sessions.retain(|_, created| now - *created < self.config.refresh_token_ttl);
        before - sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_service() -> TokenRotationService {
        TokenRotationService::new(TokenRotationConfig::default())
    }

    #[test]
    fn test_default_config() {
        let cfg = TokenRotationConfig::default();
        assert_eq!(cfg.access_token_ttl, Duration::minutes(15));
        assert_eq!(cfg.refresh_token_ttl, Duration::days(7));
        assert_eq!(cfg.absolute_max_age, Duration::days(90));
        assert_eq!(cfg.rotation_grace_period, Duration::seconds(30));
        assert_eq!(cfg.max_active_sessions, 10);
        assert!(cfg.enable_issuer_check);
    }

    #[test]
    fn test_custom_config() {
        let cfg = TokenRotationConfig {
            access_token_ttl: Duration::hours(1),
            refresh_token_ttl: Duration::days(30),
            absolute_max_age: Duration::days(365),
            rotation_grace_period: Duration::minutes(5),
            max_active_sessions: 5,
            enable_issuer_check: false,
        };
        assert_eq!(cfg.access_token_ttl, Duration::hours(1));
        assert_eq!(cfg.max_active_sessions, 5);
        assert!(!cfg.enable_issuer_check);
    }

    #[test]
    fn test_config_serialization() {
        let cfg = TokenRotationConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let deserialized: TokenRotationConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.max_active_sessions, deserialized.max_active_sessions);
        assert_eq!(cfg.enable_issuer_check, deserialized.enable_issuer_check);
    }

    #[test]
    fn test_validate_fresh_token() {
        let svc = default_service();
        let result = svc.validate_token(Utc::now(), "civitforge");
        assert!(result.is_valid);
        assert!(result.reason.is_none());
        assert!(!result.needs_rotation);
    }

    #[test]
    fn test_validate_expired_token() {
        let svc = default_service();
        let issued_at = Utc::now() - Duration::minutes(20);
        let result = svc.validate_token(issued_at, "civitforge");
        assert!(!result.is_valid);
        assert_eq!(result.reason.as_deref(), Some("token expired"));
        assert!(!result.needs_rotation);
    }

    #[test]
    fn test_validate_max_age_exceeded() {
        let svc = TokenRotationService::new(TokenRotationConfig {
            access_token_ttl: Duration::days(365),
            absolute_max_age: Duration::days(1),
            ..Default::default()
        });
        let issued_at = Utc::now() - Duration::days(2);
        let result = svc.validate_token(issued_at, "civitforge");
        assert!(!result.is_valid);
        assert_eq!(
            result.reason.as_deref(),
            Some("token beyond absolute max age")
        );
    }

    #[test]
    fn test_validate_needs_rotation() {
        let svc = default_service();
        let issued_at = Utc::now() - Duration::minutes(13);
        let result = svc.validate_token(issued_at, "civitforge");
        assert!(result.is_valid);
        assert!(result.needs_rotation);
    }

    #[test]
    fn test_validate_no_rotation_needed() {
        let svc = default_service();
        let issued_at = Utc::now() - Duration::minutes(5);
        let result = svc.validate_token(issued_at, "civitforge");
        assert!(result.is_valid);
        assert!(!result.needs_rotation);
    }

    #[test]
    fn test_validate_empty_issuer_with_check_enabled() {
        let svc = default_service();
        let result = svc.validate_token(Utc::now(), "");
        assert!(!result.is_valid);
        assert!(result.reason.as_ref().unwrap().contains("issuer"));
    }

    #[test]
    fn test_validate_empty_issuer_with_check_disabled() {
        let svc = TokenRotationService::new(TokenRotationConfig {
            enable_issuer_check: false,
            ..Default::default()
        });
        let result = svc.validate_token(Utc::now(), "");
        assert!(result.is_valid);
    }

    #[test]
    fn test_register_session() {
        let svc = default_service();
        assert!(svc.register_session("token-1").is_ok());
        assert_eq!(svc.active_session_count(), 1);
    }

    #[test]
    fn test_register_multiple_sessions() {
        let svc = default_service();
        for i in 0..5 {
            assert!(svc.register_session(&format!("token-{i}")).is_ok());
        }
        assert_eq!(svc.active_session_count(), 5);
    }

    #[test]
    fn test_register_session_max_exceeded() {
        let svc = TokenRotationService::new(TokenRotationConfig {
            max_active_sessions: 2,
            ..Default::default()
        });
        assert!(svc.register_session("t1").is_ok());
        assert!(svc.register_session("t2").is_ok());
        let err = svc.register_session("t3").unwrap_err();
        assert!(err.contains("max active sessions"));
    }

    #[test]
    fn test_revoke_session() {
        let svc = default_service();
        svc.register_session("token-1").unwrap();
        assert_eq!(svc.active_session_count(), 1);
        assert!(svc.revoke_session("token-1"));
        assert_eq!(svc.active_session_count(), 0);
    }

    #[test]
    fn test_revoke_nonexistent_session() {
        let svc = default_service();
        assert!(!svc.revoke_session("nonexistent"));
    }

    #[test]
    fn test_active_session_count_empty() {
        let svc = default_service();
        assert_eq!(svc.active_session_count(), 0);
    }

    #[test]
    fn test_cleanup_expired_sessions() {
        let svc = TokenRotationService::new(TokenRotationConfig {
            refresh_token_ttl: Duration::days(1),
            ..Default::default()
        });
        svc.register_session("fresh").unwrap();
        let expired = svc.register_session("expired");
        assert!(expired.is_ok());

        {
            let mut sessions = svc.active_sessions.lock().unwrap();
            if let Some(ts) = sessions.get_mut("expired") {
                *ts = Utc::now() - Duration::days(2);
            }
        }

        let removed = svc.cleanup_expired();
        assert_eq!(removed, 1);
        assert_eq!(svc.active_session_count(), 1);
    }

    #[test]
    fn test_cleanup_no_expired_sessions() {
        let svc = default_service();
        svc.register_session("t1").unwrap();
        let removed = svc.cleanup_expired();
        assert_eq!(removed, 0);
        assert_eq!(svc.active_session_count(), 1);
    }

    #[test]
    fn test_validation_result_serialization() {
        let result = TokenValidationResult {
            is_valid: true,
            reason: None,
            needs_rotation: false,
            session_count: 3,
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: TokenValidationResult = serde_json::from_str(&json).unwrap();
        assert!(deserialized.is_valid);
        assert!(deserialized.reason.is_none());
        assert!(!deserialized.needs_rotation);
        assert_eq!(deserialized.session_count, 3);
    }

    #[test]
    fn test_validation_result_with_reason() {
        let result = TokenValidationResult {
            is_valid: false,
            reason: Some("token expired".into()),
            needs_rotation: false,
            session_count: 0,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("token expired"));
        let deserialized: TokenValidationResult = serde_json::from_str(&json).unwrap();
        assert!(!deserialized.is_valid);
        assert_eq!(deserialized.reason.as_deref(), Some("token expired"));
    }

    #[test]
    fn test_new_service_zero_sessions() {
        let svc = default_service();
        assert_eq!(svc.active_session_count(), 0);
        assert_eq!(svc.cleanup_expired(), 0);
    }

    #[test]
    fn test_validate_session_count_reflected() {
        let svc = default_service();
        svc.register_session("t1").unwrap();
        svc.register_session("t2").unwrap();
        svc.register_session("t3").unwrap();
        let result = svc.validate_token(Utc::now(), "civitforge");
        assert!(result.is_valid);
        assert_eq!(result.session_count, 3);
    }
}
