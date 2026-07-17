#![forbid(unsafe_code)]

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use ring::rand::SystemRandom;
use ring::signature::{
    ECDSA_P256_SHA256_ASN1, ECDSA_P256_SHA256_ASN1_SIGNING, Ed25519KeyPair, KeyPair,
    RSA_PKCS1_2048_8192_SHA256, UnparsedPublicKey,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    RsaSha256,
    EcdsaP256,
    HmacSha256,
    Ed25519,
}

impl std::fmt::Display for SignatureAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignatureAlgorithm::RsaSha256 => write!(f, "rsa-sha256"),
            SignatureAlgorithm::EcdsaP256 => write!(f, "ecdsa-p256-sha256"),
            SignatureAlgorithm::HmacSha256 => write!(f, "hmac-sha256"),
            SignatureAlgorithm::Ed25519 => write!(f, "ed25519"),
        }
    }
}

impl std::str::FromStr for SignatureAlgorithm {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rsa-sha256" => Ok(SignatureAlgorithm::RsaSha256),
            "ecdsa-p256-sha256" => Ok(SignatureAlgorithm::EcdsaP256),
            "hmac-sha256" => Ok(SignatureAlgorithm::HmacSha256),
            "ed25519" => Ok(SignatureAlgorithm::Ed25519),
            _ => Err(format!("unknown signature algorithm: {s}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpSignature {
    pub key_id: String,
    pub algorithm: SignatureAlgorithm,
    pub created: DateTime<Utc>,
    pub expires: DateTime<Utc>,
    pub headers: Vec<String>,
    pub signature: String,
}

impl HttpSignature {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires
    }

    pub fn to_header_value(&self) -> String {
        let algo = &self.algorithm.to_string();
        let headers_list = self.headers.join(" ");
        format!(
            "keyId=\"{}\",algorithm=\"{}\",created=\"{}\",expires=\"{}\",headers=\"{}\",signature=\"{}\"",
            self.key_id,
            algo,
            self.created.timestamp(),
            self.expires.timestamp(),
            headers_list,
            self.signature,
        )
    }

    pub fn from_header_value(value: &str) -> std::result::Result<Self, String> {
        let mut key_id = String::new();
        let mut algorithm_str = String::new();
        let mut created_str = String::new();
        let mut expires_str = String::new();
        let mut headers_str = String::new();
        let mut signature = String::new();

        for part in value.split(',') {
            let part = part.trim();
            if let Some(val) = part
                .strip_prefix("keyId=\"")
                .and_then(|s| s.strip_suffix('"'))
            {
                key_id = val.to_string();
            } else if let Some(val) = part
                .strip_prefix("algorithm=\"")
                .and_then(|s| s.strip_suffix('"'))
            {
                algorithm_str = val.to_string();
            } else if let Some(val) = part
                .strip_prefix("created=\"")
                .and_then(|s| s.strip_suffix('"'))
            {
                created_str = val.to_string();
            } else if let Some(val) = part
                .strip_prefix("expires=\"")
                .and_then(|s| s.strip_suffix('"'))
            {
                expires_str = val.to_string();
            } else if let Some(val) = part
                .strip_prefix("headers=\"")
                .and_then(|s| s.strip_suffix('"'))
            {
                headers_str = val.to_string();
            } else if let Some(val) = part
                .strip_prefix("signature=\"")
                .and_then(|s| s.strip_suffix('"'))
            {
                signature = val.to_string();
            }
        }

        let created_ts: i64 = created_str
            .parse()
            .map_err(|_| "invalid created timestamp")?;
        let expires_ts: i64 = expires_str
            .parse()
            .map_err(|_| "invalid expires timestamp")?;
        let algorithm: SignatureAlgorithm = algorithm_str.parse()?;

        Ok(HttpSignature {
            key_id,
            algorithm,
            created: DateTime::from_timestamp(created_ts, 0).unwrap_or_default(),
            expires: DateTime::from_timestamp(expires_ts, 0).unwrap_or_default(),
            headers: if headers_str.is_empty() {
                Vec::new()
            } else {
                headers_str.split(' ').map(String::from).collect()
            },
            signature,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpSigningConfig {
    pub required_headers: Vec<String>,
    pub algorithm: SignatureAlgorithm,
    pub expires_in_secs: i64,
}

impl Default for HttpSigningConfig {
    fn default() -> Self {
        Self {
            required_headers: vec![
                "(request-target)".to_string(),
                "host".to_string(),
                "date".to_string(),
            ],
            algorithm: SignatureAlgorithm::Ed25519,
            expires_in_secs: 300,
        }
    }
}

pub struct SignatureVerifier;

impl Default for SignatureVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl SignatureVerifier {
    pub fn new() -> Self {
        Self
    }

    fn build_signing_string(
        headers: &Vec<String>,
        header_map: &HashMap<String, String>,
        method: &str,
        path: &str,
    ) -> String {
        let mut lines: Vec<String> = Vec::new();
        for h in headers {
            if h == "(request-target)" {
                lines.push(format!("(request-target): {method} {path}"));
            } else if let Some(val) = header_map.get(h) {
                lines.push(format!("{}: {val}", h.to_lowercase()));
            }
        }
        lines.join("\n")
    }

    pub fn verify_http_signature(
        &self,
        signature: &HttpSignature,
        headers: &HashMap<String, String>,
        body: &[u8],
        public_key_bytes: &[u8],
    ) -> bool {
        if signature.is_expired() {
            return false;
        }
        if signature.headers.is_empty() || signature.signature.is_empty() {
            return false;
        }

        let method = headers
            .get("(method)")
            .map(|s| s.as_str())
            .unwrap_or("POST");
        let path = headers.get("(path)").map(|s| s.as_str()).unwrap_or("/");

        let signing_string = Self::build_signing_string(&signature.headers, headers, method, path);

        let sig_bytes = match BASE64.decode(&signature.signature) {
            Ok(b) => b,
            Err(_) => return false,
        };

        let payload = if body.is_empty() {
            signing_string.as_bytes().to_vec()
        } else {
            let mut combined = signing_string.as_bytes().to_vec();
            combined.extend_from_slice(b".");
            combined.extend_from_slice(body);
            combined
        };

        match signature.algorithm {
            SignatureAlgorithm::Ed25519 => {
                let pk = UnparsedPublicKey::new(&ring::signature::ED25519, public_key_bytes);
                pk.verify(&payload, &sig_bytes).is_ok()
            }
            SignatureAlgorithm::RsaSha256 => {
                let pk = UnparsedPublicKey::new(&RSA_PKCS1_2048_8192_SHA256, public_key_bytes);
                pk.verify(&payload, &sig_bytes).is_ok()
            }
            SignatureAlgorithm::EcdsaP256 => {
                let pk = UnparsedPublicKey::new(&ECDSA_P256_SHA256_ASN1, public_key_bytes);
                pk.verify(&payload, &sig_bytes).is_ok()
            }
            SignatureAlgorithm::HmacSha256 => {
                if public_key_bytes.len() < 16 {
                    return false;
                }
                let mut mac = Hmac::<Sha256>::new_from_slice(public_key_bytes).unwrap();
                mac.update(&payload);
                let expected = mac.finalize().into_bytes();
                let provided = match BASE64.decode(&signature.signature) {
                    Ok(b) => b,
                    Err(_) => return false,
                };
                constant_time_eq(&expected, &provided)
            }
        }
    }

    pub fn sign_request(
        &self,
        config: &HttpSigningConfig,
        headers: &HashMap<String, String>,
        body: &[u8],
        private_key_bytes: &[u8],
        key_id: &str,
    ) -> std::result::Result<HttpSignature, String> {
        let method = headers
            .get("(method)")
            .map(|s| s.as_str())
            .unwrap_or("POST");
        let path = headers.get("(path)").map(|s| s.as_str()).unwrap_or("/");

        let signing_string =
            Self::build_signing_string(&config.required_headers, headers, method, path);

        let payload = if body.is_empty() {
            signing_string.as_bytes().to_vec()
        } else {
            let mut combined = signing_string.as_bytes().to_vec();
            combined.extend_from_slice(b".");
            combined.extend_from_slice(body);
            combined
        };

        let sig_bytes = match config.algorithm {
            SignatureAlgorithm::Ed25519 => {
                let key_pair = Ed25519KeyPair::from_pkcs8(private_key_bytes)
                    .map_err(|e| format!("failed to parse ed25519 private key: {e}"))?;
                key_pair.sign(&payload).as_ref().to_vec()
            }
            SignatureAlgorithm::HmacSha256 => {
                if private_key_bytes.len() < 16 {
                    return Err("HMAC key must be at least 16 bytes".into());
                }
                let mut mac = Hmac::<Sha256>::new_from_slice(private_key_bytes).unwrap();
                mac.update(&payload);
                mac.finalize().into_bytes().to_vec()
            }
            SignatureAlgorithm::RsaSha256 => {
                let rng = ring::rand::SystemRandom::new();
                let key_pair = ring::signature::RsaKeyPair::from_pkcs8(private_key_bytes)
                    .map_err(|e| format!("failed to parse RSA private key: {e}"))?;
                let mut signature = vec![0u8; key_pair.public().modulus_len()];
                key_pair
                    .sign(
                        &ring::signature::RSA_PKCS1_SHA256,
                        &rng,
                        &payload,
                        &mut signature,
                    )
                    .map_err(|e| format!("RSA signing failed: {e}"))?;
                signature
            }
            SignatureAlgorithm::EcdsaP256 => {
                let rng = ring::rand::SystemRandom::new();
                let key_pair = ring::signature::EcdsaKeyPair::from_pkcs8(
                    &ECDSA_P256_SHA256_ASN1_SIGNING,
                    private_key_bytes,
                    &rng,
                )
                .map_err(|e| format!("failed to parse ECDSA private key: {e}"))?;
                let signature = key_pair
                    .sign(&rng, &payload)
                    .map_err(|e| format!("ECDSA signing failed: {e}"))?;
                signature.as_ref().to_vec()
            }
        };

        let now = Utc::now();
        Ok(HttpSignature {
            key_id: key_id.to_string(),
            algorithm: config.algorithm.clone(),
            created: now,
            expires: now + chrono::Duration::seconds(config.expires_in_secs),
            headers: config.required_headers.clone(),
            signature: BASE64.encode(&sig_bytes),
        })
    }

    pub fn verify_ld_signature(&self, document: &serde_json::Value, public_key_pem: &str) -> bool {
        let sig_value = match document.get("proof").or_else(|| document.get("signature")) {
            Some(v) => v,
            None => return false,
        };

        let created = sig_value
            .get("created")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let sig_bytes_b64 = sig_value
            .get("signatureValue")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if created.is_empty() || sig_bytes_b64.is_empty() {
            return false;
        }

        let verification_method = sig_value
            .get("verificationMethod")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if verification_method.is_empty() {
            return false;
        }

        let canonicalized = serde_json::to_string(document).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(canonicalized.as_bytes());
        let digest = hasher.finalize();

        let pk_bytes = match BASE64.decode(public_key_pem.trim()) {
            Ok(b) => b,
            Err(_) => {
                let re = regex::Regex::new(
                    r"(?s)-----BEGIN PUBLIC KEY-----(.+?)-----END PUBLIC KEY-----",
                )
                .unwrap();
                if let Some(caps) = re.captures(public_key_pem) {
                    match BASE64.decode(caps[1].split_whitespace().collect::<Vec<_>>().join("")) {
                        Ok(b) => b,
                        Err(_) => return false,
                    }
                } else {
                    return false;
                }
            }
        };

        let sig_bytes = match BASE64.decode(sig_bytes_b64) {
            Ok(b) => b,
            Err(_) => return false,
        };

        let pk = UnparsedPublicKey::new(&ring::signature::ED25519, &pk_bytes);
        pk.verify(&digest, &sig_bytes).is_ok()
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    #[allow(deprecated)]
    {
        ring::constant_time::verify_slices_are_equal(a, b).is_ok()
    }
}

pub fn generate_ed25519_keypair() -> (Vec<u8>, Vec<u8>) {
    let rng = SystemRandom::new();
    let pkcs8_bytes =
        Ed25519KeyPair::generate_pkcs8(&rng).expect("failed to generate ed25519 keypair");
    let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
        .expect("failed to parse generated keypair");
    (
        pkcs8_bytes.as_ref().to_vec(),
        key_pair.public_key().as_ref().to_vec(),
    )
}

pub fn generate_hmac_key() -> Vec<u8> {
    use rand::RngExt;
    let mut key = vec![0u8; 32];
    rand::rng().fill(&mut key[..]);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_algorithm_display() {
        assert_eq!(SignatureAlgorithm::RsaSha256.to_string(), "rsa-sha256");
        assert_eq!(
            SignatureAlgorithm::EcdsaP256.to_string(),
            "ecdsa-p256-sha256"
        );
        assert_eq!(SignatureAlgorithm::HmacSha256.to_string(), "hmac-sha256");
        assert_eq!(SignatureAlgorithm::Ed25519.to_string(), "ed25519");
    }

    #[test]
    fn test_signature_algorithm_from_str() {
        assert_eq!(
            "rsa-sha256".parse::<SignatureAlgorithm>().unwrap(),
            SignatureAlgorithm::RsaSha256
        );
        assert_eq!(
            "ecdsa-p256-sha256".parse::<SignatureAlgorithm>().unwrap(),
            SignatureAlgorithm::EcdsaP256
        );
        assert_eq!(
            "hmac-sha256".parse::<SignatureAlgorithm>().unwrap(),
            SignatureAlgorithm::HmacSha256
        );
        assert_eq!(
            "ed25519".parse::<SignatureAlgorithm>().unwrap(),
            SignatureAlgorithm::Ed25519
        );
        assert!("unknown".parse::<SignatureAlgorithm>().is_err());
    }

    #[test]
    fn test_signature_algorithm_from_str_case_insensitive() {
        assert_eq!(
            "RSA-SHA256".parse::<SignatureAlgorithm>().unwrap(),
            SignatureAlgorithm::RsaSha256
        );
        assert_eq!(
            "Ed25519".parse::<SignatureAlgorithm>().unwrap(),
            SignatureAlgorithm::Ed25519
        );
    }

    #[test]
    fn test_http_signature_is_expired() {
        let sig = HttpSignature {
            key_id: "key1".into(),
            algorithm: SignatureAlgorithm::Ed25519,
            created: Utc::now() - chrono::Duration::hours(2),
            expires: Utc::now() - chrono::Duration::hours(1),
            headers: vec!["(request-target)".into()],
            signature: "sig".into(),
        };
        assert!(sig.is_expired());

        let sig_future = HttpSignature {
            key_id: "key1".into(),
            algorithm: SignatureAlgorithm::Ed25519,
            created: Utc::now(),
            expires: Utc::now() + chrono::Duration::hours(1),
            headers: vec!["(request-target)".into()],
            signature: "sig".into(),
        };
        assert!(!sig_future.is_expired());
    }

    #[test]
    fn test_http_signature_to_header_value() {
        let sig = HttpSignature {
            key_id: "https://example.com/actor#main-key".into(),
            algorithm: SignatureAlgorithm::Ed25519,
            created: DateTime::from_timestamp(1700000000, 0).unwrap(),
            expires: DateTime::from_timestamp(1700000300, 0).unwrap(),
            headers: vec!["(request-target)".into(), "host".into()],
            signature: "abc123".into(),
        };
        let val = sig.to_header_value();
        assert!(val.contains("keyId=\"https://example.com/actor#main-key\""));
        assert!(val.contains("algorithm=\"ed25519\""));
        assert!(val.contains("signature=\"abc123\""));
    }

    #[test]
    fn test_http_signature_from_header_value() {
        let val = r#"keyId="test-key",algorithm="ed25519",created="1700000000",expires="1700000300",headers="(request-target) host",signature="abc123""#;
        let sig = HttpSignature::from_header_value(val).unwrap();
        assert_eq!(sig.key_id, "test-key");
        assert_eq!(sig.algorithm, SignatureAlgorithm::Ed25519);
        assert_eq!(sig.headers, vec!["(request-target)", "host"]);
        assert_eq!(sig.signature, "abc123");
    }

    #[test]
    fn test_http_signature_from_header_value_invalid() {
        assert!(HttpSignature::from_header_value("garbage").is_err());
        assert!(HttpSignature::from_header_value(r#"keyId="",algorithm="ed25519",created="notanumber",expires="0",headers="",signature=""#).is_err());
    }

    #[test]
    fn test_signing_config_default() {
        let config = HttpSigningConfig::default();
        assert_eq!(config.algorithm, SignatureAlgorithm::Ed25519);
        assert_eq!(config.expires_in_secs, 300);
        assert!(
            config
                .required_headers
                .contains(&"(request-target)".to_string())
        );
        assert!(config.required_headers.contains(&"host".to_string()));
    }

    #[test]
    fn test_verify_http_signature_expired() {
        let verifier = SignatureVerifier::new();
        let sig = HttpSignature {
            key_id: "key1".into(),
            algorithm: SignatureAlgorithm::Ed25519,
            created: Utc::now() - chrono::Duration::hours(2),
            expires: Utc::now() - chrono::Duration::hours(1),
            headers: vec!["(request-target)".into()],
            signature: "whatever".into(),
        };
        let headers = HashMap::new();
        assert!(!verifier.verify_http_signature(&sig, &headers, &[], &[]));
    }

    #[test]
    fn test_verify_http_signature_empty_headers() {
        let verifier = SignatureVerifier::new();
        let sig = HttpSignature {
            key_id: "key1".into(),
            algorithm: SignatureAlgorithm::Ed25519,
            created: Utc::now(),
            expires: Utc::now() + chrono::Duration::hours(1),
            headers: vec![],
            signature: "whatever".into(),
        };
        let headers = HashMap::new();
        assert!(!verifier.verify_http_signature(&sig, &headers, &[], &[]));
    }

    #[test]
    fn test_verify_http_signature_invalid_base64() {
        let verifier = SignatureVerifier::new();
        let sig = HttpSignature {
            key_id: "key1".into(),
            algorithm: SignatureAlgorithm::Ed25519,
            created: Utc::now(),
            expires: Utc::now() + chrono::Duration::hours(1),
            headers: vec!["(request-target)".into()],
            signature: "not-valid-base64!!!".into(),
        };
        let headers = HashMap::new();
        assert!(!verifier.verify_http_signature(&sig, &headers, &[], &[]));
    }

    #[test]
    fn test_sign_and_verify_ed25519() {
        let (private_key, public_key) = generate_ed25519_keypair();
        let verifier = SignatureVerifier::new();
        let config = HttpSigningConfig {
            required_headers: vec!["(request-target)".into(), "host".into()],
            algorithm: SignatureAlgorithm::Ed25519,
            expires_in_secs: 300,
        };

        let mut headers = HashMap::new();
        headers.insert("(method)".to_string(), "POST".to_string());
        headers.insert("(path)".to_string(), "/inbox".to_string());
        headers.insert("host".to_string(), "example.com".to_string());

        let sig = verifier
            .sign_request(&config, &headers, b"{}", &private_key, "key-1")
            .unwrap();
        assert_eq!(sig.key_id, "key-1");
        assert_eq!(sig.algorithm, SignatureAlgorithm::Ed25519);
        assert!(!sig.signature.is_empty());

        assert!(verifier.verify_http_signature(&sig, &headers, b"{}", &public_key));
    }

    #[test]
    fn test_sign_and_verify_ed25519_wrong_body() {
        let (private_key, public_key) = generate_ed25519_keypair();
        let verifier = SignatureVerifier::new();
        let config = HttpSigningConfig {
            required_headers: vec!["(request-target)".into()],
            algorithm: SignatureAlgorithm::Ed25519,
            expires_in_secs: 300,
        };

        let mut headers = HashMap::new();
        headers.insert("(method)".to_string(), "POST".to_string());
        headers.insert("(path)".to_string(), "/inbox".to_string());

        let sig = verifier
            .sign_request(&config, &headers, b"{}", &private_key, "key-1")
            .unwrap();
        assert!(!verifier.verify_http_signature(&sig, &headers, b"wrong-body", &public_key));
    }

    #[test]
    fn test_sign_and_verify_hmac_sha256() {
        let hmac_key = generate_hmac_key();
        let verifier = SignatureVerifier::new();
        let config = HttpSigningConfig {
            required_headers: vec!["(request-target)".into(), "host".into()],
            algorithm: SignatureAlgorithm::HmacSha256,
            expires_in_secs: 60,
        };

        let mut headers = HashMap::new();
        headers.insert("(method)".to_string(), "GET".to_string());
        headers.insert("(path)".to_string(), "/actor".to_string());
        headers.insert("host".to_string(), "example.com".to_string());

        let sig = verifier
            .sign_request(&config, &headers, &[], &hmac_key, "hmac-key")
            .unwrap();
        assert!(verifier.verify_http_signature(&sig, &headers, &[], &hmac_key));
    }

    #[test]
    fn test_sign_invalid_private_key() {
        let verifier = SignatureVerifier::new();
        let config = HttpSigningConfig {
            required_headers: vec!["(request-target)".into()],
            algorithm: SignatureAlgorithm::Ed25519,
            expires_in_secs: 60,
        };

        let headers = HashMap::new();
        let result = verifier.sign_request(&config, &headers, &[], b"not-a-key", "k");
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_hmac_short_key_errors() {
        let verifier = SignatureVerifier::new();
        let config = HttpSigningConfig {
            required_headers: vec!["(request-target)".into()],
            algorithm: SignatureAlgorithm::HmacSha256,
            expires_in_secs: 60,
        };
        let headers = HashMap::new();
        let result = verifier.sign_request(&config, &headers, &[], b"short", "k");
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_rsa_invalid_key_errors() {
        let verifier = SignatureVerifier::new();
        let config = HttpSigningConfig {
            required_headers: vec!["(request-target)".into()],
            algorithm: SignatureAlgorithm::RsaSha256,
            expires_in_secs: 60,
        };
        let headers = HashMap::new();
        let result = verifier.sign_request(&config, &headers, &[], b"not-a-valid-rsa-key", "k");
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_ecdsa_invalid_key_errors() {
        let verifier = SignatureVerifier::new();
        let config = HttpSigningConfig {
            required_headers: vec!["(request-target)".into()],
            algorithm: SignatureAlgorithm::EcdsaP256,
            expires_in_secs: 60,
        };
        let headers = HashMap::new();
        let result = verifier.sign_request(&config, &headers, &[], b"not-a-valid-ecdsa-key", "k");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_ld_signature_missing_proof() {
        let verifier = SignatureVerifier::new();
        let doc = serde_json::json!({"type": "Note", "content": "hello"});
        assert!(!verifier.verify_ld_signature(&doc, ""));
    }

    #[test]
    fn test_verify_ld_signature_empty_signature_value() {
        let verifier = SignatureVerifier::new();
        let doc = serde_json::json!({
            "proof": {
                "type": "Ed25519Signature2020",
                "created": "2024-01-01T00:00:00Z",
                "verificationMethod": "key-1",
                "signatureValue": ""
            }
        });
        assert!(!verifier.verify_ld_signature(&doc, ""));
    }

    #[test]
    fn test_verify_ld_signature_missing_verification_method() {
        let verifier = SignatureVerifier::new();
        let doc = serde_json::json!({
            "proof": {
                "type": "Ed25519Signature2020",
                "created": "2024-01-01T00:00:00Z",
                "signatureValue": "AAAA"
            }
        });
        assert!(!verifier.verify_ld_signature(&doc, ""));
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"short", b"longer"));
    }

    #[test]
    fn test_generate_keypair() {
        let (priv_key, pub_key) = generate_ed25519_keypair();
        assert!(!priv_key.is_empty());
        assert!(!pub_key.is_empty());
        assert_eq!(pub_key.len(), 32);
    }

    #[test]
    fn test_generate_hmac_key() {
        let key = generate_hmac_key();
        assert_eq!(key.len(), 32);
    }
}
