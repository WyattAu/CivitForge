use crate::error::{AuthError, Result};
use sha2::{Digest, Sha256};

const PAT_PREFIX: &str = "cf_pat_";

/// Hash a token for storage (SHA-256)
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Generate a random token string with cf_pat_ prefix (40 bytes hex = 80 chars)
pub fn generate_token() -> String {
    let mut random_bytes = [0u8; 40];
    rand::fill(&mut random_bytes);
    format!("{PAT_PREFIX}{}", hex::encode(random_bytes))
}

/// Check if a token is a PAT (starts with cf_pat_)
pub fn is_pat_token(token: &str) -> bool {
    token.starts_with(PAT_PREFIX)
}

/// Validate scope names against allowed set
pub fn validate_scopes(scopes: &[String]) -> Result<()> {
    let allowed = [
        "read",
        "write",
        "admin",
        "repo:read",
        "repo:write",
        "user:read",
        "org:read",
        "org:write",
        "ci:read",
        "ci:write",
        "issues:read",
        "issues:write",
        "packages:read",
        "packages:write",
    ];
    for s in scopes {
        if !allowed.contains(&s.as_str()) {
            return Err(AuthError::BadRequest(format!("invalid scope: {s}")));
        }
    }
    Ok(())
}

/// Parse user_id from string
pub fn parse_user_id(user_id: &str) -> std::result::Result<uuid::Uuid, AuthError> {
    uuid::Uuid::parse_str(user_id)
        .map_err(|_| AuthError::BadRequest("invalid user ID format".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_token_deterministic() {
        let token = "cf_pat_abc123";
        let h1 = hash_token(token);
        let h2 = hash_token(token);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn test_hash_token_different_inputs() {
        let h1 = hash_token("cf_pat_abc");
        let h2 = hash_token("cf_pat_def");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_generate_token_has_prefix() {
        let token = generate_token();
        assert!(token.starts_with(PAT_PREFIX));
        assert_eq!(token.len(), PAT_PREFIX.len() + 80); // prefix + 80 hex chars
    }

    #[test]
    fn test_is_pat_token() {
        assert!(is_pat_token("cf_pat_abc123def456"));
        assert!(!is_pat_token("eyJhbGciOiJIUzI1NiJ9.test"));
    }

    #[test]
    fn test_validate_scopes_valid() {
        let scopes = vec!["read".to_string(), "write".to_string()];
        assert!(validate_scopes(&scopes).is_ok());
    }

    #[test]
    fn test_validate_scopes_invalid() {
        let scopes = vec!["read".to_string(), "invalid_scope".to_string()];
        assert!(validate_scopes(&scopes).is_err());
    }

    #[test]
    fn test_validate_scopes_empty() {
        let scopes = vec![];
        assert!(validate_scopes(&scopes).is_ok());
    }

    #[test]
    fn test_parse_user_id_valid() {
        let id = parse_user_id("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(id.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_parse_user_id_invalid() {
        assert!(parse_user_id("not-a-uuid").is_err());
    }
}
