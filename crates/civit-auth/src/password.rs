use crate::error::{AuthError, Result};

#[derive(Debug, Clone)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub max_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_digit: bool,
    pub require_special: bool,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: true,
        }
    }
}

pub fn validate_password_policy(password: &str, policy: &PasswordPolicy) -> Vec<String> {
    let mut violations = Vec::new();

    if password.len() < policy.min_length {
        violations.push(format!(
            "Password must be at least {} characters",
            policy.min_length
        ));
    }

    if password.len() > policy.max_length {
        violations.push(format!(
            "Password must be at most {} characters",
            policy.max_length
        ));
    }

    if policy.require_uppercase && !password.chars().any(|c| c.is_ascii_uppercase()) {
        violations.push("Password must contain at least one uppercase letter".into());
    }

    if policy.require_lowercase && !password.chars().any(|c| c.is_ascii_lowercase()) {
        violations.push("Password must contain at least one lowercase letter".into());
    }

    if policy.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
        violations.push("Password must contain at least one digit".into());
    }

    if policy.require_special && !password.chars().any(|c| !c.is_alphanumeric()) {
        violations.push("Password must contain at least one special character".into());
    }

    for ch in password.chars() {
        if ch.is_control() {
            violations.push("Password contains invalid characters".into());
            break;
        }
    }

    violations
}

pub fn hash_password(password: &str) -> Result<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| AuthError::Internal(format!("Failed to hash password: {e}")))
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strict_policy() -> PasswordPolicy {
        PasswordPolicy {
            min_length: 8,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: true,
        }
    }

    fn lenient_policy() -> PasswordPolicy {
        PasswordPolicy {
            min_length: 4,
            max_length: 64,
            require_uppercase: false,
            require_lowercase: false,
            require_digit: false,
            require_special: false,
        }
    }

    #[test]
    fn test_validate_password_valid() {
        let policy = strict_policy();
        let violations = validate_password_policy("Abcdef1!", &policy);
        assert!(violations.is_empty(), "got: {violations:?}");
    }

    #[test]
    fn test_validate_password_too_short() {
        let policy = strict_policy();
        let violations = validate_password_policy("Ab1!xyz", &policy);
        assert!(violations.iter().any(|v| v.contains("at least 8")));
    }

    #[test]
    fn test_validate_password_too_long() {
        let policy = strict_policy();
        let long = "Aa1!".to_string() + &"x".repeat(130);
        let violations = validate_password_policy(&long, &policy);
        assert!(violations.iter().any(|v| v.contains("at most 128")));
    }

    #[test]
    fn test_validate_password_missing_uppercase() {
        let policy = strict_policy();
        let violations = validate_password_policy("abcdef1!", &policy);
        assert!(violations.iter().any(|v| v.contains("uppercase")));
    }

    #[test]
    fn test_validate_password_missing_lowercase() {
        let policy = strict_policy();
        let violations = validate_password_policy("ABCDEF1!", &policy);
        assert!(violations.iter().any(|v| v.contains("lowercase")));
    }

    #[test]
    fn test_validate_password_missing_digit() {
        let policy = strict_policy();
        let violations = validate_password_policy("Abcdefg!", &policy);
        assert!(violations.iter().any(|v| v.contains("digit")));
    }

    #[test]
    fn test_validate_password_missing_special() {
        let policy = strict_policy();
        let violations = validate_password_policy("Abcdefg1", &policy);
        assert!(violations.iter().any(|v| v.contains("special")));
    }

    #[test]
    fn test_validate_password_multiple_violations() {
        let policy = strict_policy();
        let violations = validate_password_policy("short", &policy);
        assert!(
            violations.len() >= 3,
            "expected >=3 violations, got: {violations:?}"
        );
    }

    #[test]
    fn test_validate_password_control_chars() {
        let policy = lenient_policy();
        let violations = validate_password_policy("abc\ndef", &policy);
        assert!(violations.iter().any(|v| v.contains("invalid characters")));
    }

    #[test]
    fn test_validate_password_lenient_policy() {
        let policy = lenient_policy();
        let violations = validate_password_policy("test", &policy);
        assert!(violations.is_empty(), "got: {violations:?}");
    }

    #[test]
    fn test_validate_password_at_min_boundary() {
        let policy = strict_policy();
        let violations = validate_password_policy("Abcde1!x", &policy);
        assert!(violations.is_empty(), "got: {violations:?}");
    }

    #[test]
    fn test_validate_password_at_max_boundary() {
        let policy = strict_policy();
        let middle = "a".repeat(124);
        let pw = format!("Ab{middle}1!");
        assert_eq!(pw.len(), 128);
        let violations = validate_password_policy(&pw, &policy);
        assert!(violations.is_empty(), "got: {violations:?}");
    }

    #[test]
    fn test_password_hash_valid_format() {
        let hash = hash_password("test123").unwrap();
        assert!(hash.starts_with("$2b$"));
        assert_eq!(hash.len(), 60);
    }

    #[test]
    fn test_verify_password_correct() {
        let hash = hash_password("test123").unwrap();
        assert!(verify_password("test123", &hash));
    }

    #[test]
    fn test_verify_password_incorrect() {
        let hash = hash_password("test123").unwrap();
        assert!(!verify_password("wrong", &hash));
    }

    #[test]
    fn test_password_policy_default() {
        let policy = PasswordPolicy::default();
        assert_eq!(policy.min_length, 8);
        assert_eq!(policy.max_length, 128);
        assert!(policy.require_uppercase);
        assert!(policy.require_lowercase);
        assert!(policy.require_digit);
        assert!(policy.require_special);
    }
}
