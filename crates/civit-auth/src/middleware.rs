use crate::error::{AuthError, Result};
use crate::jwt::JwtService;
use crate::pat;
use serde::Serialize;
use sha2::{Digest, Sha256};

pub type TokenValidator =
    dyn Fn(&str) -> std::result::Result<(String, Vec<String>, uuid::Uuid), AuthError>;

#[derive(Debug, Clone, Serialize)]
pub struct AuthUser {
    pub user_id: String,
    pub username: String,
    pub role: String,
    pub org_id: Option<String>,
}

#[derive(Debug)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

/// Hash a token for comparison (SHA-256)
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Extract and validate authorization from an Authorization header value.
pub fn extract_auth_user(
    auth_header: &str,
    jwt_service: &JwtService,
    token_validator: &TokenValidator,
) -> Result<AuthUser> {
    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        if pat::is_pat_token(token) {
            let (user_id, scopes, _token_id) = token_validator(token)?;
            if !scopes.iter().any(|s| s == "read" || s == "admin") {
                return Err(AuthError::Auth("token lacks required scope".into()));
            }
            return Ok(AuthUser {
                user_id,
                username: String::new(),
                role: String::new(),
                org_id: None,
            });
        }

        let claims = jwt_service.validate_token(token)?;
        return Ok(AuthUser {
            user_id: claims.sub,
            username: claims.username,
            role: claims.role,
            org_id: claims.org_id,
        });
    }

    if let Some(basic) = auth_header.strip_prefix("Basic ") {
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(basic)
            .map_err(|_| AuthError::Auth("invalid basic auth encoding".into()))?;
        let creds =
            String::from_utf8(decoded).map_err(|_| AuthError::Auth("invalid basic auth".into()))?;
        let (_username, password) = creds
            .split_once(':')
            .ok_or_else(|| AuthError::Auth("invalid basic auth format".into()))?;

        if pat::is_pat_token(password) {
            let (user_id, scopes, _token_id) = token_validator(password)?;
            if !scopes.iter().any(|s| s == "read" || s == "admin") {
                return Err(AuthError::Auth("token lacks required scope".into()));
            }
            return Ok(AuthUser {
                user_id,
                username: String::new(),
                role: String::new(),
                org_id: None,
            });
        }

        if let Ok(claims) = jwt_service.validate_token(password) {
            return Ok(AuthUser {
                user_id: claims.sub,
                username: claims.username,
                role: claims.role,
                org_id: claims.org_id,
            });
        }

        return Err(AuthError::Auth(
            "invalid credentials (use token as password)".into(),
        ));
    }

    Err(AuthError::Auth("invalid authorization scheme".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jwt::JwtService;

    fn make_jwt_service() -> JwtService {
        JwtService::new("test-secret-key-32bytes-minimums", 24).unwrap()
    }

    #[test]
    fn test_auth_user_field_access() {
        let user = AuthUser {
            user_id: "u-1".into(),
            username: "alice".into(),
            role: "admin".into(),
            org_id: Some("org-1".into()),
        };
        assert_eq!(user.user_id, "u-1");
        assert_eq!(user.username, "alice");
        assert_eq!(user.role, "admin");
        assert_eq!(user.org_id.as_deref(), Some("org-1"));
    }

    #[test]
    fn test_auth_user_no_org() {
        let user = AuthUser {
            user_id: "u-2".into(),
            username: "bob".into(),
            role: "guest".into(),
            org_id: None,
        };
        assert!(user.org_id.is_none());
        assert_eq!(user.role, "guest");
    }

    #[test]
    fn test_auth_user_serialization() {
        let user = AuthUser {
            user_id: "u-1".into(),
            username: "alice".into(),
            role: "member".into(),
            org_id: None,
        };
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("\"user_id\":\"u-1\""));
        assert!(json.contains("\"username\":\"alice\""));
        assert!(json.contains("\"role\":\"member\""));
        assert!(json.contains("\"org_id\":null"));
    }

    #[test]
    fn test_optional_auth_user_wraps_none() {
        let opt = OptionalAuthUser(None);
        assert!(opt.0.is_none());
    }

    #[test]
    fn test_optional_auth_user_wraps_some() {
        let user = AuthUser {
            user_id: "u-1".into(),
            username: "alice".into(),
            role: "admin".into(),
            org_id: None,
        };
        let opt = OptionalAuthUser(Some(user));
        assert!(opt.0.is_some());
        assert_eq!(opt.0.as_ref().unwrap().username, "alice");
    }

    #[test]
    fn test_hash_token_deterministic() {
        let token = "cf_pat_abc123";
        let h1 = hash_token(token);
        let h2 = hash_token(token);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn test_hash_token_different_inputs() {
        let h1 = hash_token("cf_pat_abc");
        let h2 = hash_token("cf_pat_def");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_pat_token_prefix_detection() {
        let token = "cf_pat_abc123def456";
        assert!(token.starts_with("cf_pat_"));
        let jwt = "eyJhbGciOiJIUzI1NiJ9.test";
        assert!(!jwt.starts_with("cf_pat_"));
    }

    #[test]
    fn test_missing_header_returns_unauthorized() {
        let svc = make_jwt_service();
        let noop_validator = |_token: &str| -> std::result::Result<
            (String, Vec<String>, uuid::Uuid),
            AuthError,
        > { Err(AuthError::Auth("not implemented".into())) };
        let result = extract_auth_user("invalid", &svc, &noop_validator);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_scheme_returns_unauthorized() {
        let svc = make_jwt_service();
        let noop_validator = |_token: &str| -> std::result::Result<
            (String, Vec<String>, uuid::Uuid),
            AuthError,
        > { Err(AuthError::Auth("not implemented".into())) };
        let result = extract_auth_user("Basic abc123", &svc, &noop_validator);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_jwt_token_succeeds() {
        let svc = make_jwt_service();
        let noop_validator = |_token: &str| -> std::result::Result<
            (String, Vec<String>, uuid::Uuid),
            AuthError,
        > { Err(AuthError::Auth("not implemented".into())) };
        let token = svc
            .generate_token("user-1", "alice", "admin", Some("org-1"))
            .unwrap();
        let result = extract_auth_user(&format!("Bearer {token}"), &svc, &noop_validator);
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.user_id, "user-1");
        assert_eq!(user.username, "alice");
        assert_eq!(user.role, "admin");
        assert_eq!(user.org_id.as_deref(), Some("org-1"));
    }

    #[test]
    fn test_basic_auth_invalid_base64() {
        let svc = make_jwt_service();
        let noop_validator = |_token: &str| -> std::result::Result<
            (String, Vec<String>, uuid::Uuid),
            AuthError,
        > { Err(AuthError::Auth("not implemented".into())) };
        let result = extract_auth_user("Basic not-valid-base64!@#", &svc, &noop_validator);
        assert!(result.is_err());
    }

    #[test]
    fn test_basic_auth_no_colon() {
        let svc = make_jwt_service();
        let noop_validator = |_token: &str| -> std::result::Result<
            (String, Vec<String>, uuid::Uuid),
            AuthError,
        > { Err(AuthError::Auth("not implemented".into())) };
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(b"nocolon");
        let result = extract_auth_user(&format!("Basic {encoded}"), &svc, &noop_validator);
        assert!(result.is_err());
    }

    #[test]
    fn test_basic_auth_with_pat_token() {
        use base64::Engine;
        let svc = make_jwt_service();
        let pat_token = "cf_pat_abc123def456";
        let encoded = base64::engine::general_purpose::STANDARD.encode(format!("user:{pat_token}"));
        let validator =
            |_token: &str| -> std::result::Result<(String, Vec<String>, uuid::Uuid), AuthError> {
                Ok(("user-1".into(), vec!["read".into()], uuid::Uuid::nil()))
            };
        let result = extract_auth_user(&format!("Basic {encoded}"), &svc, &validator);
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.user_id, "user-1");
    }

    #[test]
    fn test_basic_auth_pat_token_lacks_scope() {
        use base64::Engine;
        let svc = make_jwt_service();
        let pat_token = "cf_pat_abc123def456";
        let encoded = base64::engine::general_purpose::STANDARD.encode(format!("user:{pat_token}"));
        let validator =
            |_token: &str| -> std::result::Result<(String, Vec<String>, uuid::Uuid), AuthError> {
                Ok(("user-1".into(), vec!["write".into()], uuid::Uuid::nil()))
            };
        let result = extract_auth_user(&format!("Basic {encoded}"), &svc, &validator);
        assert!(result.is_err());
    }

    #[test]
    fn test_bearer_pat_token_succeeds() {
        let svc = make_jwt_service();
        let pat_token = "cf_pat_abc123def456";
        let validator =
            |_token: &str| -> std::result::Result<(String, Vec<String>, uuid::Uuid), AuthError> {
                Ok(("user-1".into(), vec!["read".into()], uuid::Uuid::nil()))
            };
        let result = extract_auth_user(&format!("Bearer {pat_token}"), &svc, &validator);
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.user_id, "user-1");
        assert!(user.username.is_empty());
    }

    #[test]
    fn test_bearer_pat_token_lacks_scope() {
        let svc = make_jwt_service();
        let pat_token = "cf_pat_abc123def456";
        let validator =
            |_token: &str| -> std::result::Result<(String, Vec<String>, uuid::Uuid), AuthError> {
                Ok(("user-1".into(), vec!["write".into()], uuid::Uuid::nil()))
            };
        let result = extract_auth_user(&format!("Bearer {pat_token}"), &svc, &validator);
        assert!(result.is_err());
    }

    #[test]
    fn test_bearer_pat_token_admin_scope() {
        let svc = make_jwt_service();
        let pat_token = "cf_pat_abc123def456";
        let validator =
            |_token: &str| -> std::result::Result<(String, Vec<String>, uuid::Uuid), AuthError> {
                Ok(("user-1".into(), vec!["admin".into()], uuid::Uuid::nil()))
            };
        let result = extract_auth_user(&format!("Bearer {pat_token}"), &svc, &validator);
        assert!(result.is_ok());
    }

    #[test]
    fn test_basic_auth_invalid_jwt_password() {
        use base64::Engine;
        let svc = make_jwt_service();
        let encoded = base64::engine::general_purpose::STANDARD.encode(b"user:invalid-jwt-token");
        let noop_validator = |_token: &str| -> std::result::Result<
            (String, Vec<String>, uuid::Uuid),
            AuthError,
        > { Err(AuthError::Auth("not implemented".into())) };
        let result = extract_auth_user(&format!("Basic {encoded}"), &svc, &noop_validator);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_authorization() {
        let svc = make_jwt_service();
        let noop_validator = |_token: &str| -> std::result::Result<
            (String, Vec<String>, uuid::Uuid),
            AuthError,
        > { Err(AuthError::Auth("not implemented".into())) };
        let result = extract_auth_user("", &svc, &noop_validator);
        assert!(result.is_err());
    }

    #[test]
    fn test_auth_user_all_roles() {
        for role in &["admin", "member", "guest", "viewer", "owner"] {
            let user = AuthUser {
                user_id: "u1".into(),
                username: "user".into(),
                role: role.to_string(),
                org_id: None,
            };
            assert_eq!(user.role, *role);
        }
    }

    #[test]
    fn test_optional_auth_user_debug() {
        let opt = OptionalAuthUser(None);
        let debug_str = format!("{opt:?}");
        assert!(debug_str.contains("None"));

        let user = AuthUser {
            user_id: "u1".into(),
            username: "alice".into(),
            role: "admin".into(),
            org_id: None,
        };
        let opt = OptionalAuthUser(Some(user));
        let debug_str = format!("{opt:?}");
        assert!(debug_str.contains("alice"));
    }
}
