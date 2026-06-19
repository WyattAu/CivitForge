use crate::error::{AuthError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyInfo {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub key_type: String,
    pub public_key: String,
    pub fingerprint: String,
    pub label: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSshKeyRequest {
    pub key_type: String,
    pub public_key: String,
    pub fingerprint: String,
    pub label: Option<String>,
}

pub fn validate_ssh_key_type(key_type: &str) -> Result<()> {
    let valid_types = [
        "ssh-ed25519",
        "ssh-rsa",
        "ecdsa-sha2-nistp256",
        "ecdsa-sha2-nistp384",
        "ecdsa-sha2-nistp521",
    ];
    if valid_types.contains(&key_type) {
        Ok(())
    } else {
        Err(AuthError::BadRequest(format!(
            "unsupported key type: {key_type}"
        )))
    }
}

pub fn validate_public_key(key: &str) -> Result<()> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return Err(AuthError::BadRequest("public_key required".into()));
    }
    if trimmed.len() > 10000 {
        return Err(AuthError::BadRequest("public_key too long".into()));
    }
    Ok(())
}

pub fn validate_fingerprint(fingerprint: &str) -> Result<()> {
    let trimmed = fingerprint.trim();
    if trimmed.is_empty() {
        return Err(AuthError::BadRequest("fingerprint required".into()));
    }
    Ok(())
}

pub fn validate_label(label: &str) -> Result<()> {
    if label.len() > 255 {
        return Err(AuthError::BadRequest("label too long".into()));
    }
    Ok(())
}

pub fn from_db_key(key: civit_db::models::SshKey) -> SshKeyInfo {
    SshKeyInfo {
        id: key.id,
        user_id: key.user_id,
        key_type: key.key_type,
        public_key: key.public_key,
        fingerprint: key.fingerprint,
        label: key.label,
        created_at: key.created_at,
    }
}

pub fn parse_user_id(user_id: &str) -> Result<uuid::Uuid> {
    uuid::Uuid::parse_str(user_id).map_err(|_| AuthError::BadRequest("invalid user id".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_ssh_key_type_valid() {
        assert!(validate_ssh_key_type("ssh-ed25519").is_ok());
        assert!(validate_ssh_key_type("ssh-rsa").is_ok());
    }

    #[test]
    fn test_validate_ssh_key_type_invalid() {
        assert!(validate_ssh_key_type("ssh-dss").is_err());
    }

    #[test]
    fn test_validate_public_key_empty() {
        assert!(validate_public_key("").is_err());
        assert!(validate_public_key("  ").is_err());
    }

    #[test]
    fn test_validate_public_key_valid() {
        assert!(validate_public_key("AAAAC3NzaC1lZDI1NTE5AAAAI...").is_ok());
    }

    #[test]
    fn test_validate_fingerprint_empty() {
        assert!(validate_fingerprint("").is_err());
        assert!(validate_fingerprint("  ").is_err());
    }

    #[test]
    fn test_validate_fingerprint_valid() {
        assert!(validate_fingerprint("SHA256:abc123def456").is_ok());
    }

    #[test]
    fn test_validate_label_too_long() {
        let long = "a".repeat(256);
        assert!(validate_label(&long).is_err());
    }

    #[test]
    fn test_validate_label_valid() {
        assert!(validate_label("my-laptop").is_ok());
        assert!(validate_label("").is_ok());
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

    #[test]
    fn test_from_db_key() {
        let db_key = civit_db::models::SshKey {
            id: uuid::Uuid::nil(),
            user_id: uuid::Uuid::nil(),
            key_type: "ssh-ed25519".into(),
            public_key: "AAAAC3NzaC1lZDI1NTE5AAAAI...".into(),
            fingerprint: "SHA256:abc123def456".into(),
            label: "my-laptop".into(),
            created_at: chrono::Utc::now(),
        };
        let info = from_db_key(db_key);
        assert_eq!(info.key_type, "ssh-ed25519");
        assert_eq!(info.fingerprint, "SHA256:abc123def456");
    }

    #[test]
    fn test_validate_ssh_key_type_ecdsa() {
        assert!(validate_ssh_key_type("ecdsa-sha2-nistp256").is_ok());
        assert!(validate_ssh_key_type("ecdsa-sha2-nistp384").is_ok());
        assert!(validate_ssh_key_type("ecdsa-sha2-nistp521").is_ok());
    }

    #[test]
    fn test_validate_ssh_key_type_all_valid() {
        let valid = [
            "ssh-ed25519",
            "ssh-rsa",
            "ecdsa-sha2-nistp256",
            "ecdsa-sha2-nistp384",
            "ecdsa-sha2-nistp521",
        ];
        for k in valid {
            assert!(validate_ssh_key_type(k).is_ok(), "expected ok for {k}");
        }
    }

    #[test]
    fn test_validate_ssh_key_type_invalid_various() {
        let invalid = [
            "ssh-dss",
            "ssh-dsa",
            "ecdsa-sha2-nistp192",
            "",
            "ssh-ed25519-cert-v01@openssh.com",
        ];
        for k in invalid {
            assert!(validate_ssh_key_type(k).is_err(), "expected err for {k}");
        }
    }

    #[test]
    fn test_validate_public_key_max_length() {
        let long_key = "a".repeat(10001);
        assert!(validate_public_key(&long_key).is_err());
    }

    #[test]
    fn test_validate_public_key_exact_max() {
        let max_key = "a".repeat(10000);
        assert!(validate_public_key(&max_key).is_ok());
    }

    #[test]
    fn test_validate_label_at_boundary() {
        let label_255 = "a".repeat(255);
        assert!(validate_label(&label_255).is_ok());
    }

    #[test]
    fn test_validate_fingerprint_whitespace_only() {
        assert!(validate_fingerprint("   ").is_err());
        assert!(validate_fingerprint("\t\n").is_err());
    }

    #[test]
    fn test_from_db_key_rsa() {
        let db_key = civit_db::models::SshKey {
            id: uuid::Uuid::new_v4(),
            user_id: uuid::Uuid::new_v4(),
            key_type: "ssh-rsa".into(),
            public_key: "AAAAB3NzaC1yc2EAAAADAQAB...".into(),
            fingerprint: "MD5:ab:cd:ef:12".into(),
            label: "server-key".into(),
            created_at: chrono::Utc::now(),
        };
        let info = from_db_key(db_key);
        assert_eq!(info.key_type, "ssh-rsa");
        assert_eq!(info.label, "server-key");
    }

    #[test]
    fn test_parse_user_id_valid_various() {
        let id = parse_user_id("00000000-0000-0000-0000-000000000000").unwrap();
        assert_eq!(id, uuid::Uuid::nil());
        let id2 = parse_user_id("ffffffff-ffff-ffff-ffff-ffffffffffff").unwrap();
        assert_eq!(id2.to_string(), "ffffffff-ffff-ffff-ffff-ffffffffffff");
    }

    #[test]
    fn test_validate_ssh_key_type_empty() {
        assert!(validate_ssh_key_type("").is_err());
    }

    #[test]
    fn test_validate_ssh_key_type_whitespace() {
        assert!(validate_ssh_key_type("ssh-ed25519 ").is_err());
        assert!(validate_ssh_key_type(" ssh-ed25519").is_err());
    }

    #[test]
    fn test_validate_public_key_with_newlines() {
        let key = "AAAAC3\nzaC1l\nZDI1NTE5AAAAI...";
        assert!(validate_public_key(key).is_ok());
    }

    #[test]
    fn test_validate_public_key_special_chars() {
        let key = "AAAAC3zaC1lZDI1NTE5AAAAI!@#$%^&*()";
        assert!(validate_public_key(key).is_ok());
    }

    #[test]
    fn test_validate_fingerprint_hex_format() {
        assert!(validate_fingerprint("ab:cd:ef:12:34:56").is_ok());
    }

    #[test]
    fn test_validate_fingerprint_sha256_format() {
        assert!(validate_fingerprint("SHA256:base64hash").is_ok());
    }

    #[test]
    fn test_validate_label_exact_max() {
        let label = "a".repeat(255);
        assert!(validate_label(&label).is_ok());
    }

    #[test]
    fn test_validate_label_one_over_max() {
        let label = "a".repeat(256);
        assert!(validate_label(&label).is_err());
    }

    #[test]
    fn test_parse_user_id_with_dashes() {
        let id = parse_user_id("12345678-1234-1234-1234-123456789012").unwrap();
        assert_eq!(id.to_string(), "12345678-1234-1234-1234-123456789012");
    }

    #[test]
    fn test_parse_user_id_uppercase_hex() {
        let id = parse_user_id("550E8400-E29B-41D4-A716-446655440000").unwrap();
        assert_eq!(id.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_parse_user_id_with_braces() {
        // uuid crate actually accepts braced UUIDs
        let id = parse_user_id("{550e8400-e29b-41d4-a716-446655440000}").unwrap();
        assert_eq!(id.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_add_ssh_key_request_deserialize() {
        let json = r#"{
            "key_type": "ssh-ed25519",
            "public_key": "AAAAC3NzaC1lZDI1NTE5AAAAI...",
            "fingerprint": "SHA256:abc123def456",
            "label": "my-key"
        }"#;
        let req: AddSshKeyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.key_type, "ssh-ed25519");
        assert_eq!(req.label, Some("my-key".into()));
    }

    #[test]
    fn test_add_ssh_key_request_no_label() {
        let json = r#"{
            "key_type": "ssh-rsa",
            "public_key": "AAAAB3NzaC1yc2E...",
            "fingerprint": "MD5:ab:cd:ef"
        }"#;
        let req: AddSshKeyRequest = serde_json::from_str(json).unwrap();
        assert!(req.label.is_none());
    }

    #[test]
    fn test_ssh_key_info_serialize() {
        let info = SshKeyInfo {
            id: uuid::Uuid::nil(),
            user_id: uuid::Uuid::nil(),
            key_type: "ssh-ed25519".into(),
            public_key: "key".into(),
            fingerprint: "fp".into(),
            label: "label".into(),
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("ssh-ed25519"));
    }
}
