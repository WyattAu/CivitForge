use crate::error::{AuthError, Result};
use serde::{Deserialize, Serialize};
use tokenkit::service::{JwtAlgorithm, JwtConfig, JwtService as TokenKitService};
use tracing::info;
use zeroize::Zeroizing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub role: String,
    pub org_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    pub iat: u64,
    pub exp: u64,
}

pub struct JwtService {
    inner: TokenKitService,
    expiry_hours: u64,
}

impl std::fmt::Debug for JwtService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtService")
            .field("expiry_hours", &self.expiry_hours)
            .finish()
    }
}

impl JwtService {
    pub fn new(secret: &str, expiry_hours: u64) -> Result<Self> {
        if secret.len() < 32 {
            return Err(AuthError::Config(
                "JWT secret must be at least 32 bytes".into(),
            ));
        }
        let issuer = "civitforge".to_string();
        let config = JwtConfig {
            algorithm: JwtAlgorithm::HS256,
            secret: Zeroizing::new(secret.to_string()),
            issuer: Some(issuer),
            audience: None,
            access_token_ttl: expiry_hours as i64 * 3600,
            refresh_token_ttl: 604800,
        };
        Ok(Self {
            inner: TokenKitService::new(config),
            expiry_hours,
        })
    }

    pub fn generate_token(
        &self,
        sub: &str,
        username: &str,
        role: &str,
        org_id: Option<&str>,
    ) -> Result<String> {
        let now = chrono::Utc::now().timestamp() as u64;
        let claims = Claims {
            sub: sub.to_string(),
            username: username.to_string(),
            role: role.to_string(),
            org_id: org_id.map(String::from),
            iss: Some("civitforge".to_string()),
            iat: now,
            exp: now + (self.expiry_hours * 3600),
        };
        let token = self.inner.encode(&claims).map_err(|e| {
            AuthError::Internal(format!("Failed to encode JWT: {e}"))
        })?;
        info!(user = %username, "generated JWT token");
        Ok(token)
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let claims: Claims = self.inner.decode(token).map_err(|e| {
            AuthError::Internal(format!("Failed to decode JWT: {e}"))
        })?;
        info!(sub = %claims.sub, "validated JWT token");
        Ok(claims)
    }

    pub fn extract_bearer(header: &str) -> Option<&str> {
        header
            .strip_prefix("Bearer ")
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }

    /// Returns the token expiry in seconds.
    pub fn expiry_seconds(&self) -> u64 {
        self.expiry_hours * 3600
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_service() -> JwtService {
        JwtService::new("test-secret-key-32bytes-minimums", 24).unwrap()
    }

    #[test]
    fn test_generate_and_validate_token() {
        let svc = make_service();
        let token = svc
            .generate_token("user-1", "alice", "admin", Some("org-1"))
            .unwrap();
        let claims = svc.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "user-1");
        assert_eq!(claims.username, "alice");
        assert_eq!(claims.role, "admin");
        assert_eq!(claims.org_id.as_deref(), Some("org-1"));
    }

    #[test]
    fn test_invalid_token_rejected() {
        let svc = make_service();
        let result = svc.validate_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_secret_rejected() {
        let svc1 = JwtService::new("secret-one-32bytes-minimum-padding", 24).unwrap();
        let svc2 = JwtService::new("secret-two-32bytes-minimum-padding", 24).unwrap();
        let token = svc1.generate_token("u1", "bob", "member", None).unwrap();
        assert!(svc2.validate_token(&token).is_err());
    }

    #[test]
    fn test_extract_bearer() {
        assert_eq!(JwtService::extract_bearer("Bearer abc123"), Some("abc123"));
        assert_eq!(
            JwtService::extract_bearer("Bearer  abc123  "),
            Some("abc123")
        );
        assert_eq!(JwtService::extract_bearer("Basic abc123"), None);
        assert_eq!(JwtService::extract_bearer("Bearer "), None);
    }

    #[test]
    fn test_token_expiry_set() {
        let svc = JwtService::new("test-secret-key-32bytes-minimums", 48).unwrap();
        let token = svc.generate_token("u1", "charlie", "guest", None).unwrap();
        let claims = svc.validate_token(&token).unwrap();
        let now = chrono::Utc::now().timestamp() as u64;
        assert!(claims.exp > now);
        assert!(claims.exp <= now + (48 * 3600));
    }

    #[test]
    fn test_token_expiry_short_lived() {
        let svc = JwtService::new("test-secret-key-32bytes-minimums", 1).unwrap();
        let token = svc.generate_token("u1", "charlie", "guest", None).unwrap();
        let claims = svc.validate_token(&token).unwrap();
        // exp should be iat + 1 hour = iat + 3600
        assert_eq!(claims.exp, claims.iat + 3600);
        let now = chrono::Utc::now().timestamp() as u64;
        assert!(claims.exp > now);
    }

    #[test]
    fn test_malformed_token_rejected() {
        let svc = make_service();
        assert!(svc.validate_token("not-a-jwt").is_err());
        assert!(svc.validate_token("").is_err());
        assert!(svc.validate_token("aaa.bbb").is_err());
        assert!(svc.validate_token("aaa.bbb.ccc.ddd").is_err());
    }

    #[test]
    fn test_claims_extraction() {
        let svc = make_service();
        let token = svc
            .generate_token("user-42", "dave", "admin", Some("org-99"))
            .unwrap();
        let claims = svc.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "user-42");
        assert_eq!(claims.username, "dave");
        assert_eq!(claims.role, "admin");
        assert_eq!(claims.org_id.as_deref(), Some("org-99"));
        assert!(claims.iat > 0);
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_claims_no_org() {
        let svc = make_service();
        let token = svc.generate_token("u1", "eve", "viewer", None).unwrap();
        let claims = svc.validate_token(&token).unwrap();
        assert!(claims.org_id.is_none());
    }

    #[test]
    fn test_different_roles() {
        let svc = make_service();
        for role in &["admin", "member", "guest", "viewer", "owner"] {
            let token = svc.generate_token("u1", "user", role, None).unwrap();
            let claims = svc.validate_token(&token).unwrap();
            assert_eq!(claims.role, *role);
        }
    }

    #[test]
    fn test_extract_bearer_full_header() {
        assert_eq!(JwtService::extract_bearer("Bearer tok123"), Some("tok123"));
    }

    #[test]
    fn test_extract_bearer_empty() {
        assert_eq!(JwtService::extract_bearer(""), None);
    }

    #[test]
    fn test_extract_bearer_no_prefix() {
        assert_eq!(JwtService::extract_bearer("Token abc"), None);
    }

    #[test]
    fn test_token_uses_correct_secret() {
        let svc_a = JwtService::new("secret-a-32-bytes-padding-pad!!!", 24).unwrap();
        let svc_b = JwtService::new("secret-b-32-bytes-padding-pad!!!", 24).unwrap();
        let token = svc_a.generate_token("u1", "alice", "admin", None).unwrap();
        assert!(svc_a.validate_token(&token).is_ok());
        assert!(svc_b.validate_token(&token).is_err());
    }

    #[test]
    fn test_generate_token_empty_sub() {
        let svc = make_service();
        let token = svc.generate_token("", "alice", "admin", None).unwrap();
        let claims = svc.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "");
    }

    #[test]
    fn test_generate_token_empty_username() {
        let svc = make_service();
        let token = svc.generate_token("u1", "", "admin", None).unwrap();
        let claims = svc.validate_token(&token).unwrap();
        assert_eq!(claims.username, "");
    }

    #[test]
    fn test_generate_token_empty_role() {
        let svc = make_service();
        let token = svc.generate_token("u1", "alice", "", None).unwrap();
        let claims = svc.validate_token(&token).unwrap();
        assert_eq!(claims.role, "");
    }

    #[test]
    fn test_generate_token_long_strings() {
        let svc = make_service();
        let long_str = "x".repeat(10000);
        let token = svc
            .generate_token(&long_str, &long_str, &long_str, None)
            .unwrap();
        let claims = svc.validate_token(&token).unwrap();
        assert_eq!(claims.sub.len(), 10000);
    }

    #[test]
    fn test_extract_bearer_only_bearer() {
        assert_eq!(JwtService::extract_bearer("Bearer"), None);
    }

    #[test]
    fn test_extract_bearer_multiple_spaces() {
        assert_eq!(JwtService::extract_bearer("Bearer   token"), Some("token"));
    }

    #[test]
    fn test_token_expiry_zero_hours() {
        let svc = JwtService::new("test-secret-key-32bytes-minimums", 0).unwrap();
        let token = svc.generate_token("u1", "alice", "admin", None).unwrap();
        let claims = svc.validate_token(&token).unwrap();
        assert_eq!(claims.exp, claims.iat);
    }

    #[test]
    fn test_different_users_same_secret() {
        let svc = make_service();
        let token1 = svc.generate_token("u1", "alice", "admin", None).unwrap();
        let token2 = svc.generate_token("u2", "bob", "member", None).unwrap();
        let claims1 = svc.validate_token(&token1).unwrap();
        let claims2 = svc.validate_token(&token2).unwrap();
        assert_eq!(claims1.sub, "u1");
        assert_eq!(claims2.sub, "u2");
    }

    #[test]
    fn test_token_with_org_id() {
        let svc = make_service();
        let token = svc
            .generate_token("u1", "alice", "admin", Some("org-123"))
            .unwrap();
        let claims = svc.validate_token(&token).unwrap();
        assert_eq!(claims.org_id.as_deref(), Some("org-123"));
    }

    #[test]
    fn test_token_without_org_id() {
        let svc = make_service();
        let token = svc.generate_token("u1", "alice", "admin", None).unwrap();
        let claims = svc.validate_token(&token).unwrap();
        assert!(claims.org_id.is_none());
    }

    #[test]
    fn test_short_secret_rejected() {
        let result = JwtService::new("short", 24);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("at least 32 bytes"));
    }

    #[test]
    fn test_exactly_32_byte_secret_accepted() {
        let result = JwtService::new("12345678901234567890123456789012", 24);
        assert!(result.is_ok());
    }
}
