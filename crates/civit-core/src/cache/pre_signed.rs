#![forbid(unsafe_code)]

use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct PreSignedUrlConfig {
    pub secret_key: String,
    pub base_url: String,
    pub default_ttl_secs: i64,
}

#[derive(Debug, Clone)]
pub struct PreSignedToken {
    pub artifact_id: String,
    pub user_id: String,
    pub expires: DateTime<Utc>,
    pub signature: String,
}

pub struct PreSignedUrlGenerator {
    secret_key: Vec<u8>,
    default_ttl: Duration,
    base_url: String,
}

impl PreSignedUrlGenerator {
    pub fn new(config: PreSignedUrlConfig) -> Self {
        Self {
            secret_key: config.secret_key.into_bytes(),
            default_ttl: Duration::seconds(config.default_ttl_secs),
            base_url: config.base_url,
        }
    }

    pub fn generate_url(
        &self,
        artifact_id: &str,
        user_id: &str,
        ttl: Option<Duration>,
    ) -> Result<String, anyhow::Error> {
        let effective_ttl = ttl.unwrap_or(self.default_ttl);
        let expires = Utc::now() + effective_ttl;
        let expires_ts = expires.timestamp();

        let payload = format!("{artifact_id}\0{user_id}\0{expires_ts}");
        let signature = hmac_sign(&self.secret_key, payload.as_bytes());

        let token_data = format!("{artifact_id}\0{user_id}\0{expires_ts}\0{signature}");
        let encoded = URL_SAFE_NO_PAD.encode(token_data.as_bytes());

        Ok(format!(
            "{}/artifacts/{}/download?token={encoded}",
            self.base_url.trim_end_matches('/'),
            artifact_id
        ))
    }

    pub fn validate_token(&self, token: &PreSignedToken) -> Result<bool, anyhow::Error> {
        if token.expires < Utc::now() {
            return Ok(false);
        }

        let expires_ts = token.expires.timestamp();
        let payload = format!("{}\0{}\0{}", token.artifact_id, token.user_id, expires_ts);
        let expected_sig = hmac_sign(&self.secret_key, payload.as_bytes());

        let mut mac = HmacSha256::new_from_slice(&self.secret_key)?;
        mac.update(token.signature.as_bytes());
        let provided_mac = mac.clone().finalize().into_bytes();

        let mut mac2 = HmacSha256::new_from_slice(&self.secret_key)?;
        mac2.update(expected_sig.as_bytes());
        let expected_mac = mac2.clone().finalize().into_bytes();

        Ok(provided_mac == expected_mac)
    }

    pub fn parse_from_query(query: &str) -> Option<PreSignedToken> {
        let token_value = query.split('&').find_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            if k == "token" { Some(v) } else { None }
        })?;

        let decoded = URL_SAFE_NO_PAD.decode(token_value).ok()?;
        let decoded_str = String::from_utf8(decoded).ok()?;

        let mut parts = decoded_str.splitn(4, '\0');
        let artifact_id = parts.next()?.to_string();
        let user_id = parts.next()?.to_string();
        let expires_ts: i64 = parts.next()?.parse().ok()?;
        let signature = parts.next()?.to_string();

        Some(PreSignedToken {
            artifact_id,
            user_id,
            expires: DateTime::<Utc>::from_timestamp(expires_ts, 0)?,
            signature,
        })
    }

    pub fn parse_token_from_bytes(&self, raw: &str) -> Result<PreSignedToken, anyhow::Error> {
        let decoded = URL_SAFE_NO_PAD.decode(raw)?;
        let decoded_str = String::from_utf8(decoded)?;
        let mut parts = decoded_str.splitn(4, '\0');

        let artifact_id = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing artifact_id"))?
            .to_string();
        let user_id = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing user_id"))?
            .to_string();
        let expires_ts: i64 = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing expires"))?
            .parse()?;
        let signature = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing signature"))?
            .to_string();

        Ok(PreSignedToken {
            artifact_id,
            user_id,
            expires: DateTime::<Utc>::from_timestamp(expires_ts, 0)
                .ok_or_else(|| anyhow::anyhow!("invalid timestamp"))?,
            signature,
        })
    }
}

pub struct CacheHeaders;

impl CacheHeaders {
    pub fn public_cache(max_age_secs: u64) -> String {
        format!("public, max-age={max_age_secs}")
    }

    pub fn private_cache() -> String {
        "private, no-cache, must-revalidate".to_string()
    }

    pub fn etag_from_hash(sha256_hex: &str) -> String {
        format!("\"{sha256_hex}\"")
    }

    pub fn parse_if_none_match(header: &str) -> Vec<String> {
        header
            .split(',')
            .map(|v| v.trim().trim_matches('"').to_string())
            .filter(|v| !v.is_empty())
            .collect()
    }

    pub fn is_not_modified(if_none_match: &[String], etag: &str) -> bool {
        if if_none_match.iter().any(|v| v == "*") {
            return true;
        }
        let etag_inner = etag.trim_matches('"');
        if_none_match
            .iter()
            .any(|v| v.trim_matches('"') == etag_inner)
    }
}

fn hmac_sign(key: &[u8], data: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key length is always valid");
    mac.update(data);
    hex::encode(mac.finalize().into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> PreSignedUrlConfig {
        PreSignedUrlConfig {
            secret_key: "test-secret-key-for-signing-pre-signed-urls!".to_string(),
            base_url: "https://cdn.example.com".to_string(),
            default_ttl_secs: 3600,
        }
    }

    fn test_generator() -> PreSignedUrlGenerator {
        PreSignedUrlGenerator::new(test_config())
    }

    #[test]
    fn test_generate_url_format() {
        let generator = test_generator();
        let url = generator.generate_url("art-123", "user-1", None).unwrap();
        assert!(url.starts_with("https://cdn.example.com/artifacts/art-123/download?token="));
        let token_part = url.split("token=").nth(1).unwrap();
        assert!(!token_part.is_empty());
    }

    #[test]
    fn test_generate_and_validate_token() {
        let generator = test_generator();
        let url = generator.generate_url("art-456", "user-2", None).unwrap();
        let token_raw = url.split("token=").nth(1).unwrap();
        let token = generator.parse_token_from_bytes(token_raw).unwrap();
        assert!(generator.validate_token(&token).unwrap());
        assert_eq!(token.artifact_id, "art-456");
        assert_eq!(token.user_id, "user-2");
    }

    #[test]
    fn test_generate_with_custom_ttl() {
        let generator = test_generator();
        let url = generator
            .generate_url("art-789", "user-3", Some(Duration::seconds(60)))
            .unwrap();
        let token_raw = url.split("token=").nth(1).unwrap();
        let token = generator.parse_token_from_bytes(token_raw).unwrap();
        let expected = Utc::now() + Duration::seconds(60);
        let diff = (token.expires - expected).num_seconds().abs();
        assert!(diff <= 1);
    }

    #[test]
    fn test_different_artifacts_different_urls() {
        let generator = test_generator();
        let url1 = generator.generate_url("art-a", "user-1", None).unwrap();
        let url2 = generator.generate_url("art-b", "user-1", None).unwrap();
        assert_ne!(url1, url2);
    }

    #[test]
    fn test_different_users_different_urls() {
        let generator = test_generator();
        let url1 = generator.generate_url("art-1", "user-a", None).unwrap();
        let url2 = generator.generate_url("art-1", "user-b", None).unwrap();
        assert_ne!(url1, url2);
    }

    #[test]
    fn test_tampered_signature_fails() {
        let generator = test_generator();
        let url = generator
            .generate_url("art-tamper", "user-x", None)
            .unwrap();
        let token_raw = url.split("token=").nth(1).unwrap();
        let mut token = generator.parse_token_from_bytes(token_raw).unwrap();
        token.signature = "deadbeef00".to_string();
        assert!(!generator.validate_token(&token).unwrap());
    }

    #[test]
    fn test_tampered_artifact_id_fails() {
        let generator = test_generator();
        let url = generator
            .generate_url("art-original", "user-y", None)
            .unwrap();
        let token_raw = url.split("token=").nth(1).unwrap();
        let mut token = generator.parse_token_from_bytes(token_raw).unwrap();
        token.artifact_id = "art-forged".to_string();
        assert!(!generator.validate_token(&token).unwrap());
    }

    #[test]
    fn test_tampered_user_id_fails() {
        let generator = test_generator();
        let url = generator
            .generate_url("art-real", "user-real", None)
            .unwrap();
        let token_raw = url.split("token=").nth(1).unwrap();
        let mut token = generator.parse_token_from_bytes(token_raw).unwrap();
        token.user_id = "user-fake".to_string();
        assert!(!generator.validate_token(&token).unwrap());
    }

    #[test]
    fn test_expired_token_fails() {
        let generator = test_generator();
        let token = PreSignedToken {
            artifact_id: "art-expired".to_string(),
            user_id: "user-z".to_string(),
            expires: Utc::now() - Duration::seconds(10),
            signature: String::new(),
        };
        assert!(!generator.validate_token(&token).unwrap());
    }

    #[test]
    fn test_parse_from_query() {
        let generator = test_generator();
        let url = generator.generate_url("art-parse", "user-p", None).unwrap();
        let query = url.split('?').nth(1).unwrap();
        let token = PreSignedUrlGenerator::parse_from_query(query).unwrap();
        assert_eq!(token.artifact_id, "art-parse");
        assert_eq!(token.user_id, "user-p");
    }

    #[test]
    fn test_parse_from_query_missing_token() {
        let result = PreSignedUrlGenerator::parse_from_query("foo=bar");
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_from_query_garbled_token() {
        let result = PreSignedUrlGenerator::parse_from_query("token=!!invalid!!");
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_headers_public() {
        let header = CacheHeaders::public_cache(86400);
        assert_eq!(header, "public, max-age=86400");
    }

    #[test]
    fn test_cache_headers_private() {
        let header = CacheHeaders::private_cache();
        assert_eq!(header, "private, no-cache, must-revalidate");
    }

    #[test]
    fn test_etag_from_hash() {
        let etag = CacheHeaders::etag_from_hash("abc123");
        assert_eq!(etag, "\"abc123\"");
    }

    #[test]
    fn test_parse_if_none_match_single() {
        let etags = CacheHeaders::parse_if_none_match("\"abc\"");
        assert_eq!(etags, vec!["abc"]);
    }

    #[test]
    fn test_parse_if_none_match_multiple() {
        let etags = CacheHeaders::parse_if_none_match("\"abc\", \"def\"");
        assert_eq!(etags, vec!["abc", "def"]);
    }

    #[test]
    fn test_parse_if_none_match_wildcard() {
        let etags = CacheHeaders::parse_if_none_match("*");
        assert_eq!(etags, vec!["*"]);
    }

    #[test]
    fn test_is_not_modified_match() {
        let etags = vec!["abc".to_string()];
        assert!(CacheHeaders::is_not_modified(&etags, "\"abc\""));
    }

    #[test]
    fn test_is_not_modified_no_match() {
        let etags = vec!["xyz".to_string()];
        assert!(!CacheHeaders::is_not_modified(&etags, "\"abc\""));
    }

    #[test]
    fn test_is_not_modified_wildcard() {
        let etags = vec!["*".to_string()];
        assert!(CacheHeaders::is_not_modified(&etags, "\"anything\""));
    }

    #[test]
    fn test_is_not_modified_empty_list() {
        let etags: Vec<String> = Vec::new();
        assert!(!CacheHeaders::is_not_modified(&etags, "\"abc\""));
    }

    #[test]
    fn test_base_url_trailing_slash_stripped() {
        let cfg = PreSignedUrlConfig {
            secret_key: "secret".to_string(),
            base_url: "https://cdn.example.com/".to_string(),
            default_ttl_secs: 3600,
        };
        let generator = PreSignedUrlGenerator::new(cfg);
        let url = generator.generate_url("a", "u", None).unwrap();
        assert!(url.starts_with("https://cdn.example.com/artifacts/"));
        assert!(!url.contains("//artifacts"));
    }
}
