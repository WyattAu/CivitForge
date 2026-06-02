#![forbid(unsafe_code)]

//! SLSA provenance signing using ECDSA P-256-SHA256 via ring.
//!
//! [`SignedProvenance`] is an in-toto envelope: JSON-serialized `SlsaProvenance`
//! + base64-encoded ECDSA signature + PEM-encoded public key.
//!
//! [`ProvenanceSigner`] holds an ECDSA key pair and can sign/verify provenance
//! statements. The key pair is generated with ring's `SystemRandom` and persists
//! across multiple signing operations within the same process.

use super::{SlsaProvenance, sha256_digest};
use base64::{Engine, prelude::BASE64_STANDARD};
use ring::{
    rand::{SecureRandom, SystemRandom},
    signature::{
        ECDSA_P256_SHA256_ASN1, ECDSA_P256_SHA256_ASN1_SIGNING, EcdsaKeyPair, KeyPair,
        UnparsedPublicKey,
    },
};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum SignatureVerificationError {
    #[error("base64 decode failed: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("signature verification failed: {0}")]
    VerificationFailed(ring::error::Unspecified),
    #[error("provenance serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("malformed public key PEM: {0}")]
    MalformedPublicKey(String),
    #[error("no signature present")]
    NoSignature,
    #[error("key generation failed: {0}")]
    KeyGeneration(String),
}

// ---------------------------------------------------------------------------
// Signing key pair — wraps ring EcdsaKeyPair
// ---------------------------------------------------------------------------

/// ECDSA P-256-SHA256 key pair for provenance signing.
pub struct SigningKeyPair {
    key_pair: EcdsaKeyPair,
}

impl std::fmt::Debug for SigningKeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SigningKeyPair")
            .field("algorithm", &"ECDSA_P256_SHA256")
            .finish()
    }
}

impl SigningKeyPair {
    /// Generate a new ECDSA P-256 key pair using the system random source.
    pub fn generate() -> Result<Self, SignatureVerificationError> {
        let rng = SystemRandom::new();
        let pkcs8_doc = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &rng)
            .map_err(|e| SignatureVerificationError::KeyGeneration(e.to_string()))?;
        let key_pair = EcdsaKeyPair::from_pkcs8(
            &ECDSA_P256_SHA256_ASN1_SIGNING,
            pkcs8_doc.as_ref(),
            &rng as &dyn SecureRandom,
        )
        .map_err(|e| SignatureVerificationError::KeyGeneration(e.to_string()))?;
        Ok(Self { key_pair })
    }

    /// Serialize the public key as DER SubjectPublicKeyInfo, then base64-wrap
    /// in a PEM envelope.
    pub fn public_key_pem(&self) -> String {
        let der = self.key_pair.public_key().as_ref();
        pem_encode(der, "PUBLIC KEY")
    }

    /// Sign `message` and return the DER-encoded ECDSA signature.
    fn sign(&self, message: &[u8]) -> Vec<u8> {
        let rng = SystemRandom::new();
        self.key_pair
            .sign(&rng, message)
            .expect("ECDSA signing failed")
            .as_ref()
            .to_vec()
    }
}

// ---------------------------------------------------------------------------
// PEM encoding
// ---------------------------------------------------------------------------

/// Minimal PEM encoder — wraps `der_bytes` with the given label.
fn pem_encode(der_bytes: &[u8], label: &str) -> String {
    let b64 = BASE64_STANDARD.encode(der_bytes);
    let mut pem = format!("-----BEGIN {label}-----\n");
    // Wrap at 64 characters
    for chunk in b64.as_bytes().chunks(64) {
        pem.push_str(std::str::from_utf8(chunk).unwrap());
        pem.push('\n');
    }
    pem.push_str(&format!("-----END {label}-----\n"));
    pem
}

/// Minimal PEM decoder — extracts the base64 payload between the headers.
fn pem_decode(pem: &str, expected_label: &str) -> Result<Vec<u8>, SignatureVerificationError> {
    let header = format!("-----BEGIN {expected_label}-----");
    let footer = format!("-----END {expected_label}-----");

    let start = pem.find(&header).ok_or_else(|| {
        SignatureVerificationError::MalformedPublicKey(format!("missing PEM header: {header}"))
    })?;
    let after_header = &pem[start + header.len()..];

    let end = after_header.find(&footer).ok_or_else(|| {
        SignatureVerificationError::MalformedPublicKey(format!("missing PEM footer: {footer}"))
    })?;

    let b64_body: String = after_header[..end]
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    BASE64_STANDARD
        .decode(&b64_body)
        .map_err(SignatureVerificationError::Base64)
}

// ---------------------------------------------------------------------------
// SignedProvenance — in-toto envelope
// ---------------------------------------------------------------------------

/// Envelope containing a serialized SLSA provenance statement, its ECDSA
/// signature, and the signer's public key (PEM).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignedProvenance {
    /// JSON-serialized `SlsaProvenance`.
    pub payload: String,
    /// Base64-encoded DER ECDSA signature.
    pub signature: String,
    /// PEM-encoded SubjectPublicKeyInfo.
    pub public_key_pem: String,
}

impl SignedProvenance {
    /// Decode the payload back into `SlsaProvenance`.
    pub fn decode_payload(&self) -> Result<SlsaProvenance, SignatureVerificationError> {
        serde_json::from_str(&self.payload).map_err(SignatureVerificationError::Serialization)
    }

    /// Verify the signature against the embedded payload using the embedded
    /// public key.
    ///
    /// The signature covers the SHA-256 digest of the payload (matching how
    /// [`ProvenanceSigner::sign`] works).
    pub fn verify_signature(&self) -> Result<(), SignatureVerificationError> {
        let signature_bytes = BASE64_STANDARD.decode(&self.signature)?;
        let public_key_der = pem_decode(&self.public_key_pem, "PUBLIC KEY")?;

        let public_key = UnparsedPublicKey::new(&ECDSA_P256_SHA256_ASN1, &public_key_der);
        let digest = sha256_digest(self.payload.as_bytes());

        public_key
            .verify(digest.as_bytes(), &signature_bytes)
            .map_err(SignatureVerificationError::VerificationFailed)
    }

    /// Verify the signature against the embedded payload using an *external*
    /// trusted public key (PEM).
    pub fn verify_with_public_key(
        &self,
        trusted_pem: &str,
    ) -> Result<(), SignatureVerificationError> {
        let signature_bytes = BASE64_STANDARD.decode(&self.signature)?;
        let public_key_der = pem_decode(trusted_pem, "PUBLIC KEY")?;

        let public_key = UnparsedPublicKey::new(&ECDSA_P256_SHA256_ASN1, &public_key_der);
        let digest = sha256_digest(self.payload.as_bytes());

        public_key
            .verify(digest.as_bytes(), &signature_bytes)
            .map_err(SignatureVerificationError::VerificationFailed)
    }

    /// Full verification: structural checks + cryptographic signature.
    pub fn verify_full(&self) -> Result<(), SignatureVerificationError> {
        self.verify_signature()?;
        let provenance = self.decode_payload()?;
        let result = super::ProvenanceGenerator::verify(&provenance)
            .map_err(SignatureVerificationError::MalformedPublicKey)?;
        if !result.passed {
            return Err(SignatureVerificationError::MalformedPublicKey(
                "structural verification failed".into(),
            ));
        }
        if !provenance.kind.starts_with("https://in-toto.io/Statement") {
            return Err(SignatureVerificationError::MalformedPublicKey(
                "wrong _type predicate".into(),
            ));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ProvenanceSigner — high-level API
// ---------------------------------------------------------------------------

/// Holds an ECDSA P-256 key pair and signs provenance statements.
pub struct ProvenanceSigner {
    key_pair: SigningKeyPair,
}

impl std::fmt::Debug for ProvenanceSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProvenanceSigner")
            .field("key_pair", &self.key_pair)
            .finish()
    }
}

impl ProvenanceSigner {
    /// Create a new signer with a freshly generated key pair.
    pub fn new() -> Result<Self, SignatureVerificationError> {
        Ok(Self {
            key_pair: SigningKeyPair::generate()?,
        })
    }

    /// Create a signer from an existing key pair.
    pub fn with_key_pair(key_pair: SigningKeyPair) -> Self {
        Self { key_pair }
    }

    /// Access the underlying key pair (e.g. to export public key for storage).
    pub fn key_pair(&self) -> &SigningKeyPair {
        &self.key_pair
    }

    /// Sign a `SlsaProvenance` and return the `SignedProvenance` envelope.
    ///
    /// The provenance is serialized to deterministic JSON before signing.
    pub fn sign(
        &self,
        provenance: &SlsaProvenance,
    ) -> Result<SignedProvenance, SignatureVerificationError> {
        let payload =
            serde_json::to_string(provenance).map_err(SignatureVerificationError::Serialization)?;

        // Sign the SHA-256 digest of the payload (not the raw JSON) for
        // consistency with SLSA best practices and to bound signature size.
        let digest = sha256_digest(payload.as_bytes());
        let signature = self.key_pair.sign(digest.as_bytes());
        let signature_b64 = BASE64_STANDARD.encode(&signature);

        Ok(SignedProvenance {
            payload,
            signature: signature_b64,
            public_key_pem: self.key_pair.public_key_pem(),
        })
    }

    /// Verify a `SignedProvenance` using the signer's trusted public key.
    pub fn verify(&self, signed: &SignedProvenance) -> Result<(), SignatureVerificationError> {
        // Verify cryptographic signature first
        let signature_bytes = BASE64_STANDARD.decode(&signed.signature)?;
        let public_key_der = pem_decode(&signed.public_key_pem, "PUBLIC KEY")?;
        let public_key = UnparsedPublicKey::new(&ECDSA_P256_SHA256_ASN1, &public_key_der);

        let digest = sha256_digest(signed.payload.as_bytes());
        public_key
            .verify(digest.as_bytes(), &signature_bytes)
            .map_err(SignatureVerificationError::VerificationFailed)?;

        // Structural checks
        let provenance = signed.decode_payload()?;
        let result = super::ProvenanceGenerator::verify(&provenance)
            .map_err(SignatureVerificationError::MalformedPublicKey)?;
        if !result.passed {
            return Err(SignatureVerificationError::MalformedPublicKey(
                "structural verification failed".into(),
            ));
        }
        if !provenance.kind.starts_with("https://in-toto.io/Statement") {
            return Err(SignatureVerificationError::MalformedPublicKey(
                "wrong _type predicate".into(),
            ));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance::{ProvenanceGenerator, SlsaProvenance, material_with_digest};
    use chrono::Utc;

    fn make_test_provenance() -> SlsaProvenance {
        let materials = vec![material_with_digest(
            "git+https://example.com/repo",
            b"hello",
        )];
        ProvenanceGenerator::new("civitforge-builder".into())
            .with_version("1.0.0".into())
            .generate("inv-test-1", materials, Utc::now(), Some(Utc::now()), true)
    }

    // -- SigningKeyPair --

    #[test]
    fn test_generate_key_pair() {
        let kp = SigningKeyPair::generate().expect("generate");
        let pem = kp.public_key_pem();
        assert!(pem.starts_with("-----BEGIN PUBLIC KEY-----"));
        assert!(pem.ends_with("-----END PUBLIC KEY-----\n"));
    }

    #[test]
    fn test_key_pair_sign_verify() {
        let kp = SigningKeyPair::generate().expect("generate");
        let msg = b"test message to sign";

        let sig = kp.sign(msg);
        assert!(!sig.is_empty());

        // Verify with the public key
        let pem = kp.public_key_pem();
        let pub_der = pem_decode(&pem, "PUBLIC KEY").unwrap();
        let pub_key = UnparsedPublicKey::new(&ECDSA_P256_SHA256_ASN1, &pub_der);
        pub_key.verify(msg, &sig).expect("signature should verify");
    }

    #[test]
    fn test_key_pair_sign_verify_wrong_message() {
        let kp = SigningKeyPair::generate().expect("generate");
        let sig = kp.sign(b"correct message");

        let pem = kp.public_key_pem();
        let pub_der = pem_decode(&pem, "PUBLIC KEY").unwrap();
        let pub_key = UnparsedPublicKey::new(&ECDSA_P256_SHA256_ASN1, &pub_der);
        // Wrong message should fail
        assert!(pub_key.verify(b"wrong message", &sig).is_err());
    }

    // -- PEM encoding/decoding --

    #[test]
    fn test_pem_roundtrip() {
        let original = b"some DER bytes here";
        let pem = pem_encode(original, "PUBLIC KEY");
        let decoded = pem_decode(&pem, "PUBLIC KEY").unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_pem_decode_missing_header() {
        let pem = "-----END PUBLIC KEY-----\n";
        assert!(pem_decode(pem, "PUBLIC KEY").is_err());
    }

    #[test]
    fn test_pem_decode_missing_footer() {
        let pem = "-----BEGIN PUBLIC KEY-----\n";
        assert!(pem_decode(pem, "PUBLIC KEY").is_err());
    }

    #[test]
    fn test_pem_decode_wrong_label() {
        let pem = pem_encode(b"data", "CERTIFICATE");
        assert!(pem_decode(&pem, "PUBLIC KEY").is_err());
    }

    // -- ProvenanceSigner --

    #[test]
    fn test_sign_provenance() {
        let signer = ProvenanceSigner::new().expect("new");
        let prov = make_test_provenance();
        let signed = signer.sign(&prov).expect("sign");

        assert!(!signed.payload.is_empty());
        assert!(!signed.signature.is_empty());
        assert!(
            signed
                .public_key_pem
                .starts_with("-----BEGIN PUBLIC KEY-----")
        );
    }

    #[test]
    fn test_sign_and_verify_roundtrip() {
        let signer = ProvenanceSigner::new().expect("new");
        let prov = make_test_provenance();
        let signed = signer.sign(&prov).expect("sign");

        // Verify with the same signer
        signer.verify(&signed).expect("verify should succeed");
    }

    #[test]
    fn test_signed_provenance_decode_payload() {
        let signer = ProvenanceSigner::new().expect("new");
        let prov = make_test_provenance();
        let signed = signer.sign(&prov).expect("sign");

        let decoded = signed.decode_payload().expect("decode");
        assert_eq!(decoded.builder.id, prov.builder.id);
        assert_eq!(decoded.metadata.build_invocation_id, "inv-test-1");
        assert_eq!(decoded.materials.len(), 1);
    }

    #[test]
    fn test_signed_provenance_verify_signature() {
        let signer = ProvenanceSigner::new().expect("new");
        let prov = make_test_provenance();
        let signed = signer.sign(&prov).expect("sign");

        // verify_signature uses embedded public key
        signed
            .verify_signature()
            .expect("embedded signature should verify");
    }

    #[test]
    fn test_signed_provenance_verify_with_external_key() {
        let signer = ProvenanceSigner::new().expect("new");
        let prov = make_test_provenance();
        let signed = signer.sign(&prov).expect("sign");

        let trusted_pem = signer.key_pair().public_key_pem();
        signed
            .verify_with_public_key(&trusted_pem)
            .expect("verify with trusted key");
    }

    #[test]
    fn test_signed_provenance_verify_with_wrong_key() {
        let signer1 = ProvenanceSigner::new().expect("new");
        let signer2 = ProvenanceSigner::new().expect("new");
        let prov = make_test_provenance();
        let signed = signer1.sign(&prov).expect("sign");

        let wrong_pem = signer2.key_pair().public_key_pem();
        assert!(signed.verify_with_public_key(&wrong_pem).is_err());
    }

    #[test]
    fn test_tampered_payload_rejected() {
        let signer = ProvenanceSigner::new().expect("new");
        let prov = make_test_provenance();
        let mut signed = signer.sign(&prov).expect("sign");

        // Tamper with the payload
        signed.payload = signed
            .payload
            .replace("civitforge-builder", "attacker-builder");

        assert!(signed.verify_signature().is_err());
        assert!(signer.verify(&signed).is_err());
    }

    #[test]
    fn test_tampered_signature_rejected() {
        let signer = ProvenanceSigner::new().expect("new");
        let prov = make_test_provenance();
        let mut signed = signer.sign(&prov).expect("sign");

        // Corrupt the signature
        let mut sig_bytes = BASE64_STANDARD.decode(&signed.signature).unwrap();
        sig_bytes[0] = sig_bytes[0].wrapping_add(1);
        signed.signature = BASE64_STANDARD.encode(&sig_bytes);

        assert!(signed.verify_signature().is_err());
        assert!(signer.verify(&signed).is_err());
    }

    #[test]
    fn test_tampered_public_key_rejected() {
        let signer1 = ProvenanceSigner::new().expect("new");
        let signer2 = ProvenanceSigner::new().expect("new");
        let prov = make_test_provenance();
        let mut signed = signer1.sign(&prov).expect("sign");

        // Replace public key with a different one
        signed.public_key_pem = signer2.key_pair().public_key_pem();

        assert!(signed.verify_signature().is_err());
    }

    #[test]
    fn test_signed_provenance_serialization_roundtrip() {
        let signer = ProvenanceSigner::new().expect("new");
        let prov = make_test_provenance();
        let signed = signer.sign(&prov).expect("sign");

        let json = serde_json::to_string(&signed).expect("serialize envelope");
        let deserialized: SignedProvenance =
            serde_json::from_str(&json).expect("deserialize envelope");

        assert_eq!(deserialized.payload, signed.payload);
        assert_eq!(deserialized.signature, signed.signature);
        assert_eq!(deserialized.public_key_pem, signed.public_key_pem);

        // Deserialized version should also verify
        deserialized
            .verify_signature()
            .expect("deserialized verify");
    }

    #[test]
    fn test_verify_full_valid() {
        let signer = ProvenanceSigner::new().expect("new");
        let prov = make_test_provenance();
        let signed = signer.sign(&prov).expect("sign");
        signed.verify_full().expect("full verification should pass");
    }

    #[test]
    fn test_verify_full_tampered_payload() {
        let signer = ProvenanceSigner::new().expect("new");
        let prov = make_test_provenance();
        let mut signed = signer.sign(&prov).expect("sign");
        signed.payload.push('"');
        assert!(signed.verify_full().is_err());
    }

    #[test]
    fn test_verify_full_empty_builder() {
        let signer = ProvenanceSigner::new().expect("new");
        let mut prov = make_test_provenance();
        prov.builder.id = String::new();
        let signed = signer.sign(&prov).expect("sign");
        assert!(signed.verify_full().is_err());
    }

    #[test]
    fn test_verify_full_wrong_type() {
        let signer = ProvenanceSigner::new().expect("new");
        let mut prov = make_test_provenance();
        prov.kind = "https://evil.example.com/Fake".into();
        let signed = signer.sign(&prov).expect("sign");
        assert!(signed.verify_full().is_err());
    }

    #[test]
    fn test_sign_multiple_provenance_same_key() {
        let signer = ProvenanceSigner::new().expect("new");

        let prov1 = make_test_provenance();
        let signed1 = signer.sign(&prov1).expect("sign 1");

        let mut prov2 = make_test_provenance();
        prov2.metadata.build_invocation_id = "inv-test-2".into();
        let signed2 = signer.sign(&prov2).expect("sign 2");

        // Both should verify with the same key
        signer.verify(&signed1).expect("verify 1");
        signer.verify(&signed2).expect("verify 2");

        // Different payloads should have different signatures
        assert_ne!(signed1.signature, signed2.signature);
    }

    #[test]
    fn test_signer_debug() {
        let signer = ProvenanceSigner::new().expect("new");
        let debug = format!("{signer:?}");
        assert!(debug.contains("ProvenanceSigner"));
        assert!(debug.contains("ECDSA_P256_SHA256"));
    }

    #[test]
    fn test_signing_key_pair_debug() {
        let kp = SigningKeyPair::generate().expect("generate");
        let debug = format!("{kp:?}");
        assert!(debug.contains("SigningKeyPair"));
    }

    #[test]
    fn test_pem_encode_long_key() {
        // Real P-256 keys produce ~91 bytes of DER; test a larger payload
        let long_data = vec![0xABu8; 512];
        let pem = pem_encode(&long_data, "PUBLIC KEY");
        assert!(
            pem.lines()
                .all(|line| { line.is_empty() || line.starts_with("-----") || line.len() <= 64 })
        );
        let decoded = pem_decode(&pem, "PUBLIC KEY").unwrap();
        assert_eq!(decoded, long_data);
    }
}
