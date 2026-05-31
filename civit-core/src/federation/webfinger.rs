#![forbid(unsafe_code)]

use crate::error::{CoreError, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct WebFingerResponse {
    pub subject: String,
    pub aliases: Vec<String>,
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    pub rel: String,
    pub type_: String,
    pub href: String,
}

pub fn resolve_webfinger(domain: &str, username: &str) -> Result<WebFingerResponse> {
    if domain.is_empty() || username.is_empty() {
        return Err(CoreError::Federation(
            "domain and username must be non-empty".into(),
        ));
    }

    Ok(WebFingerResponse {
        subject: format!("acct:{username}@{domain}"),
        aliases: vec![
            format!("https://{domain}/users/{username}"),
            format!("https://{domain}/u/{username}"),
        ],
        links: vec![
            Link {
                rel: "self".into(),
                type_: "application/activity+json".into(),
                href: format!("https://{domain}/users/{username}"),
            },
            Link {
                rel: "http://webfinger.net/rel/profile-page".into(),
                type_: "text/html".into(),
                href: format!("https://{domain}/{username}"),
            },
        ],
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct HttpSignature {
    pub key_id: String,
    pub algorithm: String,
    pub headers: Vec<String>,
    pub signature: String,
}

/// Verify HTTP signature using SHA-256 hash comparison (insecure, legacy).
///
/// WARNING: This is NOT cryptographic signature verification. It compares a
/// hash of key_id+method+path+headers against the signature. Use
/// `verify_http_signature_ed25519` for real cryptographic verification.
#[deprecated(note = "Use verify_http_signature_ed25519 for real crypto verification")]
pub fn verify_http_signature(
    signature: &HttpSignature,
    _public_key_pem: &str,
    method: &str,
    path: &str,
    headers: &HashMap<String, String>,
) -> Result<bool> {
    if signature.key_id.is_empty() || signature.signature.is_empty() {
        return Err(CoreError::Federation(
            "key_id and signature must be non-empty".into(),
        ));
    }

    let mut hasher = Sha256::new();
    hasher.update(signature.key_id.as_bytes());
    hasher.update(method.as_bytes());
    hasher.update(path.as_bytes());
    for header_name in &signature.headers {
        if let Some(value) = headers.get(header_name) {
            hasher.update(header_name.as_bytes());
            hasher.update(value.as_bytes());
        }
    }
    let expected = hex::encode(hasher.finalize());

    Ok(signature.signature == expected)
}

/// Verify an HTTP signature using Ed25519 cryptographic verification.
///
/// `public_key_bytes` must be the raw 32-byte Ed25519 public key of the signer.
/// The signature in `HttpSignature.signature` must be base64-encoded 64 bytes.
pub fn verify_http_signature_ed25519(
    signature: &HttpSignature,
    public_key_bytes: &[u8],
    method: &str,
    path: &str,
    headers: &HashMap<String, String>,
) -> Result<bool> {
    if signature.key_id.is_empty() || signature.signature.is_empty() {
        return Err(CoreError::Federation(
            "key_id and signature must be non-empty".into(),
        ));
    }

    let mut message = Vec::new();
    message.extend_from_slice(signature.key_id.as_bytes());
    message.extend_from_slice(method.as_bytes());
    message.extend_from_slice(path.as_bytes());
    for header_name in &signature.headers {
        if let Some(value) = headers.get(header_name) {
            message.extend_from_slice(header_name.as_bytes());
            message.extend_from_slice(value.as_bytes());
        }
    }

    let signature_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &signature.signature,
    )
    .map_err(|_| CoreError::Federation("invalid base64 signature".into()))?;

    let public_key =
        ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, public_key_bytes);

    match public_key.verify(&message, &signature_bytes) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

pub fn create_http_signature(
    key_id: &str,
    private_key: &[u8],
    method: &str,
    path: &str,
    headers: &HashMap<String, String>,
) -> Result<HttpSignature> {
    if key_id.is_empty() || private_key.is_empty() {
        return Err(CoreError::Federation(
            "key_id and private_key must be non-empty".into(),
        ));
    }

    let mut hasher = Sha256::new();
    hasher.update(key_id.as_bytes());
    hasher.update(method.as_bytes());
    hasher.update(path.as_bytes());

    let mut signed_headers = vec!["(request-target)".to_string()];
    for (name, value) in headers {
        hasher.update(name.as_bytes());
        hasher.update(value.as_bytes());
        signed_headers.push(name.clone());
    }

    let sig_input = hasher.finalize();
    let signature = hex::encode(sig_input);

    Ok(HttpSignature {
        key_id: key_id.to_string(),
        algorithm: "hs2019".into(),
        headers: signed_headers,
        signature,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_webfinger() {
        let response = resolve_webfinger("forge.example.com", "alice").unwrap();
        assert_eq!(response.subject, "acct:alice@forge.example.com");
        assert_eq!(response.aliases.len(), 2);
        assert_eq!(response.links.len(), 2);

        let self_link = response.links.iter().find(|l| l.rel == "self").unwrap();
        assert_eq!(self_link.type_, "application/activity+json");
        assert_eq!(self_link.href, "https://forge.example.com/users/alice");
    }

    #[test]
    fn test_resolve_webfinger_empty_domain() {
        let result = resolve_webfinger("", "alice");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_webfinger_empty_username() {
        let result = resolve_webfinger("forge.example.com", "");
        assert!(result.is_err());
    }

    #[test]
    fn test_http_signature_verify_valid() {
        let key_id = "key-1";
        let method = "POST";
        let path = "/inbox";
        let mut headers: HashMap<String, String> = HashMap::new();
        headers.insert("host".into(), "forge.example.com".into());
        headers.insert("date".into(), "2025-01-01T00:00:00Z".into());

        let sig =
            create_http_signature(key_id, b"test-private-key", method, path, &headers).unwrap();
        assert!(verify_http_signature(&sig, "unused-pem", method, path, &headers).unwrap());
    }

    #[test]
    fn test_http_signature_verify_invalid() {
        let sig = HttpSignature {
            key_id: "key-1".into(),
            algorithm: "hs2019".into(),
            headers: vec![],
            signature: "wrong-signature".into(),
        };
        assert!(!verify_http_signature(&sig, "pem", "POST", "/inbox", &HashMap::new()).unwrap());
    }

    #[test]
    fn test_http_signature_verify_empty() {
        let sig = HttpSignature {
            key_id: "".into(),
            algorithm: "hs2019".into(),
            headers: vec![],
            signature: "".into(),
        };
        let result = verify_http_signature(&sig, "pem", "POST", "/", &HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_create_http_signature() {
        let mut headers = HashMap::new();
        headers.insert("host".into(), "forge.example.com".into());
        headers.insert("date".into(), "2025-01-01T00:00:00Z".into());

        let sig = create_http_signature("key-1", b"private-key-bytes", "POST", "/inbox", &headers)
            .unwrap();

        assert_eq!(sig.key_id, "key-1");
        assert_eq!(sig.algorithm, "hs2019");
        assert!(sig.headers.contains(&"(request-target)".to_string()));
        assert!(!sig.signature.is_empty());
    }

    #[test]
    fn test_create_http_signature_empty_key() {
        let result = create_http_signature("", b"", "POST", "/", &HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_create_and_verify_roundtrip() {
        let key_id = "key-rt-1";
        let private_key = b"test-private-key";
        let method = "PUT";
        let path = "/repo/1";
        let mut headers = HashMap::new();
        headers.insert("content-type".into(), "application/json".into());

        let sig = create_http_signature(key_id, private_key, method, path, &headers).unwrap();
        let valid = verify_http_signature(&sig, "unused-pem", method, path, &headers).unwrap();
        assert!(valid);
    }
}
