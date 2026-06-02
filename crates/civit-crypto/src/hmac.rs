#![forbid(unsafe_code)]

use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::debug;

type HmacSha256 = Hmac<Sha256>;

pub struct HmacService;

impl Default for HmacService {
    fn default() -> Self {
        Self::new()
    }
}

impl HmacService {
    pub fn new() -> Self {
        Self
    }

    pub fn sign(key: &[u8], data: &[u8]) -> String {
        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key length error");
        mac.update(data);
        let result = mac.finalize();
        let bytes = result.into_bytes();
        let hex = hex::encode(bytes);
        debug!(data_len = data.len(), "generated HMAC signature");
        hex
    }

    pub fn sign_string(key: &str, message: &str) -> String {
        Self::sign(key.as_bytes(), message.as_bytes())
    }

    pub fn verify(key: &[u8], data: &[u8], expected_hex: &str) -> bool {
        let expected_bytes = match hex::decode(expected_hex) {
            Ok(b) => b,
            Err(_) => return false,
        };

        let mut mac = match HmacSha256::new_from_slice(key) {
            Ok(m) => m,
            Err(_) => return false,
        };
        mac.update(data);
        mac.verify_slice(&expected_bytes).is_ok()
    }

    pub fn verify_string(key: &str, message: &str, expected_hex: &str) -> bool {
        Self::verify(key.as_bytes(), message.as_bytes(), expected_hex)
    }

    pub fn sign_base64(key: &[u8], data: &[u8]) -> String {
        let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key length error");
        mac.update(data);
        let result = mac.finalize();
        let bytes = result.into_bytes();
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
    }

    pub fn timed_sign(key: &[u8], data: &[u8], timestamp: u64) -> String {
        let timed_data = format!("{}:{}", timestamp, String::from_utf8_lossy(data));
        Self::sign(key, timed_data.as_bytes())
    }

    pub fn timed_verify(
        key: &[u8],
        data: &[u8],
        timestamp: u64,
        signature: &str,
        max_age_secs: u64,
    ) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if timestamp + max_age_secs < now {
            debug!("timestamp expired");
            return false;
        }
        if timestamp > now + 60 {
            debug!("timestamp too far in future");
            return false;
        }

        let timed_data = format!("{}:{}", timestamp, String::from_utf8_lossy(data));
        Self::verify(key, timed_data.as_bytes(), signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let key = b"secret-key";
        let data = b"message to sign";
        let sig = HmacService::sign(key, data);
        assert!(HmacService::verify(key, data, &sig));
    }

    #[test]
    fn test_verify_wrong_key() {
        let sig = HmacService::sign(b"key1", b"data");
        assert!(!HmacService::verify(b"key2", b"data", &sig));
    }

    #[test]
    fn test_verify_wrong_data() {
        let sig = HmacService::sign(b"key", b"original");
        assert!(!HmacService::verify(b"key", b"modified", &sig));
    }

    #[test]
    fn test_sign_string() {
        let sig = HmacService::sign_string("my-key", "my-message");
        assert!(HmacService::verify_string("my-key", "my-message", &sig));
    }

    #[test]
    fn test_sign_base64_format() {
        let sig = HmacService::sign_base64(b"key", b"data");
        assert!(base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &sig).is_ok());
    }

    #[test]
    fn test_timed_sign_and_verify() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let sig = HmacService::timed_sign(b"key", b"data", now);
        assert!(HmacService::timed_verify(b"key", b"data", now, &sig, 300));
    }

    #[test]
    fn test_timed_verify_expired() {
        let old_timestamp = 1000000u64;
        let sig = HmacService::timed_sign(b"key", b"data", old_timestamp);
        assert!(!HmacService::timed_verify(
            b"key",
            b"data",
            old_timestamp,
            &sig,
            300
        ));
    }

    #[test]
    fn test_deterministic() {
        let s1 = HmacService::sign(b"k", b"d");
        let s2 = HmacService::sign(b"k", b"d");
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_different_keys_different_sigs() {
        let s1 = HmacService::sign(b"key_a", b"data");
        let s2 = HmacService::sign(b"key_b", b"data");
        assert_ne!(s1, s2);
    }
}
