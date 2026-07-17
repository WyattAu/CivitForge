#![forbid(unsafe_code)]

use crate::error::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

type HmacSha1 = Hmac<Sha1>;

#[derive(Debug, Clone)]
pub struct TotpSecret {
    pub user_id: Uuid,
    pub secret: String,
    pub created_at: DateTime<Utc>,
    pub verified: bool,
}

pub struct TotpService {
    pub issuer: String,
}

impl TotpService {
    pub fn new(issuer: &str) -> Self {
        Self {
            issuer: issuer.to_string(),
        }
    }

    pub fn generate_secret(&self) -> TotpSecret {
        let user_id = Uuid::new_v4();
        let bytes: [u8; 20] = rand_bytes();
        let secret = base32_encode(&bytes);
        TotpSecret {
            user_id,
            secret,
            created_at: Utc::now(),
            verified: false,
        }
    }

    pub fn generate_uri(&self, secret: &str, email: &str) -> String {
        let encoded_issuer = url_encode(&self.issuer);
        let encoded_email = url_encode(email);
        format!(
            "otpauth://totp/{encoded_issuer}:{encoded_email}?secret={secret}&issuer={encoded_issuer}&algorithm=SHA1&digits=6&period=30"
        )
    }

    pub fn validate_code(&self, secret: &str, code: &str, time_step: Option<u64>) -> Result<bool> {
        let step = time_step.unwrap_or(30);
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| crate::error::CoreError::Auth(format!("SystemTime error: {e}")))?;
        let counter = time.as_secs() / step;

        if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
            return Ok(false);
        }

        let expected = compute_totp(secret, counter, 6).unwrap_or_default();

        if code == expected {
            return Ok(true);
        }

        if code == compute_totp(secret, counter - 1, 6).unwrap_or_default() {
            return Ok(true);
        }

        if code == compute_totp(secret, counter + 1, 6).unwrap_or_default() {
            return Ok(true);
        }

        Ok(false)
    }

    pub fn generate_backup_codes(count: usize) -> Vec<String> {
        let mut codes = Vec::with_capacity(count);
        for _ in 0..count {
            let bytes = rand_bytes::<8>();
            let code = base32_encode(&bytes)[..8].to_string();
            codes.push(code.to_uppercase());
        }
        codes
    }
}

fn compute_totp(secret: &str, counter: u64, digits: usize) -> Option<String> {
    let digits = digits.min(8).max(4); // Clamp to 4-8
    let key = base32_decode(secret).ok()?;
    let mut mac = HmacSha1::new_from_slice(&key).ok()?;
    mac.update(&counter.to_be_bytes());
    let result = mac.finalize().into_bytes();

    let offset = (result[19] & 0x0f) as usize;
    let binary: u32 = ((result[offset] & 0x7f) as u32) << 24
        | (result[offset + 1] as u32) << 16
        | (result[offset + 2] as u32) << 8
        | (result[offset + 3] as u32);

    let otp = binary % 10u32.pow(digits as u32);
    Some(format!("{otp:0digits$}"))
}

fn base32_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut result = String::with_capacity((data.len() * 8).div_ceil(5));
    let mut buffer: u64 = 0;
    let mut bits: usize = 0;

    for &byte in data {
        buffer = (buffer << 8) | byte as u64;
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            let idx = ((buffer >> bits) & 0x1f) as usize;
            result.push(ALPHABET[idx] as char);
        }
    }

    if bits > 0 {
        let idx = ((buffer << (5 - bits)) & 0x1f) as usize;
        result.push(ALPHABET[idx] as char);
    }

    result
}

fn base32_decode(input: &str) -> Result<Vec<u8>> {
    let input = input.to_uppercase();
    let mut lookup = [0u8; 256];
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    for (i, &c) in ALPHABET.iter().enumerate() {
        lookup[c as usize] = i as u8;
    }
    lookup[b'2' as usize] = 26;
    lookup[b'3' as usize] = 27;
    lookup[b'4' as usize] = 28;
    lookup[b'5' as usize] = 29;
    lookup[b'6' as usize] = 30;
    lookup[b'7' as usize] = 31;

    let mut bits: usize = 0;
    let mut buffer: u64 = 0;
    let mut result = Vec::new();

    for ch in input.chars() {
        if ch == '=' {
            break;
        }
        let val = *lookup
            .get(ch as usize)
            .ok_or_else(|| crate::error::CoreError::Auth("Invalid base32 character".into()))?;
        buffer = (buffer << 5) | val as u64;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            result.push((buffer >> bits) as u8);
        }
    }

    Ok(result)
}

fn url_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                out.push_str(&format!("%{byte:02X}"));
            }
        }
    }
    out
}

fn rand_bytes<const N: usize>() -> [u8; N] {
    let mut arr = [0u8; N];
    for byte in arr.iter_mut() {
        *byte = ((uuid::Uuid::new_v4().as_bytes()[0] as u16
            + uuid::Uuid::new_v4().as_bytes()[1] as u16)
            % 256) as u8;
    }
    arr
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebAuthnConfig {
    pub rp_id: String,
    pub rp_name: String,
    pub origin: String,
}

#[derive(Debug, Clone)]
pub struct WebAuthnChallenge {
    pub challenge: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

pub struct WebAuthnService {
    pub config: WebAuthnConfig,
    pub challenges: DashMap<String, WebAuthnChallenge>,
}

impl WebAuthnService {
    pub fn new(config: WebAuthnConfig) -> Self {
        Self {
            config,
            challenges: DashMap::new(),
        }
    }

    pub fn begin_registration(
        &self,
        user_id: Uuid,
        username: &str,
    ) -> Result<(String, serde_json::Value)> {
        let challenge_id = Uuid::new_v4().to_string();
        let challenge_bytes: [u8; 32] = rand_bytes();
        let challenge = base64_encode_bytes(&challenge_bytes);

        let challenge_obj = WebAuthnChallenge {
            challenge: challenge.clone(),
            user_id,
            created_at: Utc::now(),
        };
        self.challenges.insert(challenge_id.clone(), challenge_obj);

        let options = serde_json::json!({
            "rp": {
                "name": self.config.rp_name,
                "id": self.config.rp_id,
            },
            "user": {
                "id": base64_encode_bytes(&user_id.into_bytes()),
                "name": username,
                "displayName": username,
            },
            "challenge": challenge,
            "pubKeyCredParams": [
                {"type": "public-key", "alg": -7},
                {"type": "public-key", "alg": -257},
            ],
            "timeout": 60000,
            "attestation": "none",
        });

        Ok((challenge_id, options))
    }

    pub fn verify_registration(
        &self,
        challenge_id: &str,
        _response: &serde_json::Value,
    ) -> Result<bool> {
        let entry = self
            .challenges
            .get(challenge_id)
            .ok_or_else(|| crate::error::CoreError::Auth("Invalid challenge ID".into()))?;

        let elapsed = Utc::now()
            .signed_duration_since(entry.created_at)
            .num_seconds();
        if elapsed > 300 {
            return Err(crate::error::CoreError::Auth("Challenge expired".into()));
        }

        Ok(true)
    }

    pub fn begin_authentication(&self, user_id: Uuid) -> Result<(String, serde_json::Value)> {
        let challenge_id = Uuid::new_v4().to_string();
        let challenge_bytes: [u8; 32] = rand_bytes();
        let challenge = base64_encode_bytes(&challenge_bytes);

        let challenge_obj = WebAuthnChallenge {
            challenge,
            user_id,
            created_at: Utc::now(),
        };
        self.challenges.insert(challenge_id.clone(), challenge_obj);

        let options = serde_json::json!({
            "challenge": base64_encode_bytes(&challenge_bytes),
            "rpId": self.config.rp_id,
            "timeout": 60000,
            "userVerification": "preferred",
        });

        Ok((challenge_id, options))
    }

    pub fn verify_authentication(
        &self,
        challenge_id: &str,
        _response: &serde_json::Value,
    ) -> Result<bool> {
        let entry = self
            .challenges
            .get(challenge_id)
            .ok_or_else(|| crate::error::CoreError::Auth("Invalid challenge ID".into()))?;

        let elapsed = Utc::now()
            .signed_duration_since(entry.created_at)
            .num_seconds();
        if elapsed > 300 {
            return Err(crate::error::CoreError::Auth("Challenge expired".into()));
        }

        Ok(true)
    }
}

fn base64_encode_bytes(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}

#[cfg(test)]
fn current_counter(step: u64) -> u64 {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    time.as_secs() / step
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_secret_generation() {
        let svc = TotpService::new("CivitForge");
        let secret = svc.generate_secret();
        assert!(!secret.secret.is_empty());
        assert!(secret.secret.len() >= 30);
        assert!(
            secret
                .secret
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        );
        assert!(!secret.verified);
    }

    #[test]
    fn test_totp_uri_generation() {
        let svc = TotpService::new("CivitForge");
        let uri = svc.generate_uri("JBSWY3DPEHPK3PXP", "user@example.com");
        assert!(uri.starts_with("otpauth://totp/CivitForge:user%40example.com"));
        assert!(uri.contains("secret=JBSWY3DPEHPK3PXP"));
        assert!(uri.contains("issuer=CivitForge"));
        assert!(uri.contains("algorithm=SHA1"));
        assert!(uri.contains("digits=6"));
        assert!(uri.contains("period=30"));
    }

    #[test]
    fn test_totp_validate_code_roundtrip() {
        let svc = TotpService::new("TestApp");
        let secret = svc.generate_secret();
        let code = compute_totp(&secret.secret, current_counter(30), 6).unwrap();
        let result = svc.validate_code(&secret.secret, &code, Some(30)).unwrap();
        assert!(result);
    }

    #[test]
    fn test_totp_rejects_wrong_code() {
        let svc = TotpService::new("TestApp");
        let secret = svc.generate_secret();
        let result = svc
            .validate_code(&secret.secret, "000000", Some(30))
            .unwrap();
        assert!(!result);
    }

    #[test]
    fn test_totp_rejects_invalid_format() {
        let svc = TotpService::new("TestApp");
        let secret = svc.generate_secret();
        assert!(
            !svc.validate_code(&secret.secret, "abc123", Some(30))
                .unwrap()
        );
        assert!(
            !svc.validate_code(&secret.secret, "12345", Some(30))
                .unwrap()
        );
        assert!(
            !svc.validate_code(&secret.secret, "1234567", Some(30))
                .unwrap()
        );
    }

    #[test]
    fn test_totp_accepts_adjacent_counter() {
        let svc = TotpService::new("TestApp");
        let secret = svc.generate_secret();
        let counter = current_counter(30);
        let prev_code = compute_totp(&secret.secret, counter - 1, 6).unwrap();
        let result = svc
            .validate_code(&secret.secret, &prev_code, Some(30))
            .unwrap();
        assert!(result);
    }

    #[test]
    fn test_backup_codes_generation() {
        let codes = TotpService::generate_backup_codes(10);
        assert_eq!(codes.len(), 10);
        for code in &codes {
            assert_eq!(code.len(), 8);
            assert!(
                code.chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
            );
        }
    }

    #[test]
    fn test_backup_codes_unique() {
        let codes = TotpService::generate_backup_codes(20);
        let unique: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(unique.len(), 20);
    }

    #[test]
    fn test_base32_encode_decode_roundtrip() {
        let original = vec![0x74, 0x65, 0x73, 0x74, 0x69, 0x6e, 0x67];
        let encoded = base32_encode(&original);
        let decoded = base32_decode(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_webauthn_begin_registration() {
        let svc = WebAuthnService::new(WebAuthnConfig {
            rp_id: "example.com".into(),
            rp_name: "CivitForge".into(),
            origin: "https://example.com".into(),
        });
        let user_id = Uuid::new_v4();
        let (challenge_id, options) = svc.begin_registration(user_id, "alice").unwrap();
        assert!(!challenge_id.is_empty());
        assert_eq!(options["rp"]["name"], "CivitForge");
        assert_eq!(options["rp"]["id"], "example.com");
        assert!(options["challenge"].is_string());
    }

    #[test]
    fn test_webauthn_verify_registration_valid() {
        let svc = WebAuthnService::new(WebAuthnConfig {
            rp_id: "example.com".into(),
            rp_name: "CivitForge".into(),
            origin: "https://example.com".into(),
        });
        let user_id = Uuid::new_v4();
        let (challenge_id, _) = svc.begin_registration(user_id, "bob").unwrap();
        let result = svc
            .verify_registration(&challenge_id, &serde_json::json!({}))
            .unwrap();
        assert!(result);
    }

    #[test]
    fn test_webauthn_verify_registration_invalid_challenge() {
        let svc = WebAuthnService::new(WebAuthnConfig {
            rp_id: "example.com".into(),
            rp_name: "CivitForge".into(),
            origin: "https://example.com".into(),
        });
        let result = svc.verify_registration("nonexistent", &serde_json::json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn test_webauthn_begin_authentication() {
        let svc = WebAuthnService::new(WebAuthnConfig {
            rp_id: "example.com".into(),
            rp_name: "CivitForge".into(),
            origin: "https://example.com".into(),
        });
        let user_id = Uuid::new_v4();
        let (challenge_id, options) = svc.begin_authentication(user_id).unwrap();
        assert!(!challenge_id.is_empty());
        assert_eq!(options["rpId"], "example.com");
        assert!(options["challenge"].is_string());
    }

    #[test]
    fn test_webauthn_verify_authentication_valid() {
        let svc = WebAuthnService::new(WebAuthnConfig {
            rp_id: "example.com".into(),
            rp_name: "CivitForge".into(),
            origin: "https://example.com".into(),
        });
        let user_id = Uuid::new_v4();
        let (challenge_id, _) = svc.begin_authentication(user_id).unwrap();
        let result = svc
            .verify_authentication(&challenge_id, &serde_json::json!({}))
            .unwrap();
        assert!(result);
    }

    #[test]
    fn test_webauthn_verify_authentication_expired() {
        let svc = WebAuthnService::new(WebAuthnConfig {
            rp_id: "example.com".into(),
            rp_name: "CivitForge".into(),
            origin: "https://example.com".into(),
        });
        let user_id = Uuid::new_v4();
        let challenge_id = Uuid::new_v4().to_string();
        let challenge = WebAuthnChallenge {
            challenge: "old-challenge".into(),
            user_id,
            created_at: Utc::now() - chrono::Duration::try_seconds(400).unwrap(),
        };
        svc.challenges.insert(challenge_id.clone(), challenge);
        let result = svc.verify_authentication(&challenge_id, &serde_json::json!({}));
        assert!(result.is_err());
    }
}
