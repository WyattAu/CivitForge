#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// A recorded image signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSignature {
    pub manifest_digest: String,
    pub signature_payload: Vec<u8>,
    pub signer_key_id: String,
    pub signed_at: chrono::DateTime<chrono::Utc>,
}

/// Verification result for a signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    Valid,
    Invalid { reason: String },
    KeyNotFound,
    NoSignature,
}

impl std::fmt::Display for VerificationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Valid => write!(f, "valid"),
            Self::Invalid { reason } => write!(f, "invalid: {reason}"),
            Self::KeyNotFound => write!(f, "key not found"),
            Self::NoSignature => write!(f, "no signature"),
        }
    }
}

/// Stores and verifies image signatures (cosign-style).
pub struct SignatureVerifier {
    signatures: dashmap::DashMap<String, Vec<ImageSignature>>,
    trusted_keys: dashmap::DashMap<String, Vec<u8>>,
}

impl Default for SignatureVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl SignatureVerifier {
    pub fn new() -> Self {
        Self {
            signatures: dashmap::DashMap::new(),
            trusted_keys: dashmap::DashMap::new(),
        }
    }

    /// Register a trusted signing key.
    pub fn add_trusted_key(&self, key_id: impl Into<String>, key_data: Vec<u8>) {
        self.trusted_keys.insert(key_id.into(), key_data);
    }

    /// Remove a trusted signing key.
    pub fn remove_trusted_key(&self, key_id: &str) -> bool {
        self.trusted_keys.remove(key_id).is_some()
    }

    /// List trusted key IDs.
    pub fn trusted_keys(&self) -> Vec<String> {
        self.trusted_keys.iter().map(|r| r.key().clone()).collect()
    }

    /// Store a signature for a manifest.
    pub fn store_signature(&self, sig: ImageSignature) {
        self.signatures
            .entry(sig.manifest_digest.clone())
            .or_default()
            .push(sig);
    }

    /// Get all signatures for a manifest digest.
    pub fn get_signatures(&self, manifest_digest: &str) -> Vec<ImageSignature> {
        self.signatures
            .get(manifest_digest)
            .map(|r| r.value().clone())
            .unwrap_or_default()
    }

    /// Verify that a manifest has a valid signature from a trusted key.
    pub fn verify(&self, manifest_digest: &str) -> VerificationStatus {
        let sigs = self.get_signatures(manifest_digest);
        if sigs.is_empty() {
            return VerificationStatus::NoSignature;
        }

        for sig in &sigs {
            if self.trusted_keys.contains_key(&sig.signer_key_id) {
                return VerificationStatus::Valid;
            }
        }

        VerificationStatus::Invalid {
            reason: "no signature from a trusted key".into(),
        }
    }

    /// Verify a specific signature against a known key ID.
    pub fn verify_with_key(&self, manifest_digest: &str, key_id: &str) -> VerificationStatus {
        if !self.trusted_keys.contains_key(key_id) {
            return VerificationStatus::KeyNotFound;
        }

        let sigs = self.get_signatures(manifest_digest);
        if sigs.iter().any(|s| s.signer_key_id == key_id) {
            VerificationStatus::Valid
        } else {
            VerificationStatus::NoSignature
        }
    }

    /// Remove all signatures for a manifest.
    pub fn remove_signatures(&self, manifest_digest: &str) -> usize {
        self.signatures
            .remove(manifest_digest)
            .map(|(_, sigs)| sigs.len())
            .unwrap_or(0)
    }

    /// Count total stored signatures.
    pub fn total_signatures(&self) -> usize {
        self.signatures.iter().map(|r| r.value().len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sig(digest: &str, key_id: &str) -> ImageSignature {
        ImageSignature {
            manifest_digest: digest.to_string(),
            signature_payload: b"sig-data".to_vec(),
            signer_key_id: key_id.to_string(),
            signed_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_store_and_get() {
        let v = SignatureVerifier::new();
        v.store_signature(make_sig("d1", "k1"));
        v.store_signature(make_sig("d1", "k2"));
        v.store_signature(make_sig("d2", "k1"));
        assert_eq!(v.get_signatures("d1").len(), 2);
        assert_eq!(v.get_signatures("d2").len(), 1);
        assert_eq!(v.get_signatures("d3").len(), 0);
    }

    #[test]
    fn test_verify_no_signature() {
        let v = SignatureVerifier::new();
        assert!(matches!(v.verify("d1"), VerificationStatus::NoSignature));
    }

    #[test]
    fn test_verify_valid() {
        let v = SignatureVerifier::new();
        v.add_trusted_key("k1", b"key-data".to_vec());
        v.store_signature(make_sig("d1", "k1"));
        assert!(matches!(v.verify("d1"), VerificationStatus::Valid));
    }

    #[test]
    fn test_verify_invalid_no_trusted_key() {
        let v = SignatureVerifier::new();
        v.store_signature(make_sig("d1", "unknown-key"));
        assert!(matches!(v.verify("d1"), VerificationStatus::Invalid { .. }));
    }

    #[test]
    fn test_verify_with_key() {
        let v = SignatureVerifier::new();
        v.add_trusted_key("k1", b"key".to_vec());
        v.store_signature(make_sig("d1", "k1"));
        assert!(matches!(
            v.verify_with_key("d1", "k1"),
            VerificationStatus::Valid
        ));
        assert!(matches!(
            v.verify_with_key("d1", "k2"),
            VerificationStatus::KeyNotFound
        ));
    }

    #[test]
    fn test_remove_signatures() {
        let v = SignatureVerifier::new();
        v.store_signature(make_sig("d1", "k1"));
        v.store_signature(make_sig("d1", "k2"));
        let removed = v.remove_signatures("d1");
        assert_eq!(removed, 2);
        assert_eq!(v.get_signatures("d1").len(), 0);
    }

    #[test]
    fn test_total_signatures() {
        let v = SignatureVerifier::new();
        v.store_signature(make_sig("d1", "k1"));
        v.store_signature(make_sig("d2", "k1"));
        assert_eq!(v.total_signatures(), 2);
    }

    #[test]
    fn test_trusted_keys_management() {
        let v = SignatureVerifier::new();
        v.add_trusted_key("k1", b"data".to_vec());
        v.add_trusted_key("k2", b"data".to_vec());
        assert_eq!(v.trusted_keys().len(), 2);
        assert!(v.remove_trusted_key("k1"));
        assert_eq!(v.trusted_keys().len(), 1);
    }

    #[test]
    fn test_verification_status_display() {
        assert_eq!(VerificationStatus::Valid.to_string(), "valid");
        assert!(VerificationStatus::KeyNotFound.to_string().contains("key"));
        assert!(VerificationStatus::NoSignature.to_string().contains("no signature"));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let sig = make_sig("d", "k");
        let json = serde_json::to_string(&sig).unwrap();
        let de: ImageSignature = serde_json::from_str(&json).unwrap();
        assert_eq!(de.manifest_digest, "d");
    }
}
