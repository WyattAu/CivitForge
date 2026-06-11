use crate::error::Result;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub role: String,
    pub org_id: Option<String>,
    pub iat: u64,
    pub exp: u64,
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiry_hours: u64,
}

impl JwtService {
    pub fn new(secret: &str, expiry_hours: u64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            expiry_hours,
        }
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
            iat: now,
            exp: now + (self.expiry_hours * 3600),
        };
        let token = encode(&Header::default(), &claims, &self.encoding_key)?;
        info!(user = %username, "generated JWT token");
        Ok(token)
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let validation = Validation::default();
        let data = decode::<Claims>(token, &self.decoding_key, &validation)?;
        info!(sub = %data.claims.sub, "validated JWT token");
        Ok(data.claims)
    }

    pub fn extract_bearer(header: &str) -> Option<&str> {
        header
            .strip_prefix("Bearer ")
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_service() -> JwtService {
        JwtService::new("test-secret-key-32bytes-minimums", 24)
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
        let svc1 = JwtService::new("secret-one-32bytes-minimum-padding", 24);
        let svc2 = JwtService::new("secret-two-32bytes-minimum-padding", 24);
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
        let svc = JwtService::new("test-secret-key-32bytes-minimums", 48);
        let token = svc.generate_token("u1", "charlie", "guest", None).unwrap();
        let claims = svc.validate_token(&token).unwrap();
        let now = chrono::Utc::now().timestamp() as u64;
        assert!(claims.exp > now);
        assert!(claims.exp <= now + (48 * 3600));
    }
}
