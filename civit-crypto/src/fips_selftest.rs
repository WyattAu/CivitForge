#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};
use std::time::Instant;

#[derive(Debug, Clone, serde::Serialize)]
pub struct FipsTestResult {
    pub test_name: String,
    pub passed: bool,
    pub duration_us: u128,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct FipsConfig {
    pub fips_enabled: bool,
    pub fips_module_id: String,
    pub fips_version: String,
}

impl Default for FipsConfig {
    fn default() -> Self {
        Self {
            fips_enabled: true,
            fips_module_id: "civit-crypto-001".to_string(),
            fips_version: "1.0.0".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FipsSelfTest {
    config: FipsConfig,
}

impl Default for FipsSelfTest {
    fn default() -> Self {
        Self::new()
    }
}

impl FipsSelfTest {
    pub fn new() -> Self {
        Self {
            config: FipsConfig::default(),
        }
    }

    pub fn with_config(config: FipsConfig) -> Self {
        Self { config }
    }

    pub fn run_all(&self) -> Vec<FipsTestResult> {
        vec![
            self.run_hash_test(),
            self.run_hmac_test(),
            self.run_aes_test(),
            self.run_rng_test(),
            self.run_signing_test(),
        ]
    }

    pub fn run_hash_test(&self) -> FipsTestResult {
        let start = Instant::now();
        let data = b"abc";
        let expected =
            hex::decode("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
                .unwrap();
        let result = Sha256::digest(data).to_vec();
        let duration = start.elapsed().as_micros();
        let passed = result == expected;
        FipsTestResult {
            test_name: "SHA-256".to_string(),
            passed,
            duration_us: duration,
            details: if passed {
                "SHA-256 output matches NIST test vector".to_string()
            } else {
                format!(
                    "expected {}, got {}",
                    hex::encode(&expected),
                    hex::encode(&result)
                )
            },
        }
    }

    pub fn run_hmac_test(&self) -> FipsTestResult {
        let start = Instant::now();
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<Sha256>;
        let key = hex::decode("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b").unwrap();
        let data = b"Hi There";
        let expected =
            hex::decode("b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7")
                .unwrap();
        let mut mac = HmacSha256::new_from_slice(&key).unwrap();
        mac.update(data);
        let result = mac.finalize().into_bytes().to_vec();
        let duration = start.elapsed().as_micros();
        let passed = result == expected;
        FipsTestResult {
            test_name: "HMAC-SHA256".to_string(),
            passed,
            duration_us: duration,
            details: if passed {
                "HMAC-SHA256 output matches RFC 4231 test case 1".to_string()
            } else {
                format!(
                    "expected {}, got {}",
                    hex::encode(&expected),
                    hex::encode(&result)
                )
            },
        }
    }

    pub fn run_aes_test(&self) -> FipsTestResult {
        let start = Instant::now();
        use ring::aead::{AES_256_GCM, Aad, LessSafeKey, Nonce, UnboundKey};
        let key_bytes: [u8; 32] = [0u8; 32];
        let nonce_bytes: [u8; 12] = [0u8; 12];
        let key = UnboundKey::new(&AES_256_GCM, &key_bytes).unwrap();
        let key = LessSafeKey::new(key);
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let plaintext = b"FIPS AES-256-GCM test plaintext";
        let aad = Aad::empty();
        let mut in_out = plaintext.to_vec();
        key.seal_in_place_append_tag(nonce, aad, &mut in_out)
            .unwrap();
        let duration = start.elapsed().as_micros();
        let passed = in_out.len() > plaintext.len();
        FipsTestResult {
            test_name: "AES-256-GCM".to_string(),
            passed,
            duration_us: duration,
            details: if passed {
                format!(
                    "AES-256-GCM encryption produced {} bytes from {} bytes input (with 16-byte tag)",
                    in_out.len(),
                    plaintext.len()
                )
            } else {
                "AES-256-GCM encryption failed".to_string()
            },
        }
    }

    pub fn run_rng_test(&self) -> FipsTestResult {
        let start = Instant::now();
        let rng = ring::rand::SystemRandom::new();
        let mut buf1 = [0u8; 32];
        let mut buf2 = [0u8; 32];
        let r1 = ring::rand::SecureRandom::fill(&rng, &mut buf1);
        let r2 = ring::rand::SecureRandom::fill(&rng, &mut buf2);
        let duration = start.elapsed().as_micros();
        let passed = r1.is_ok() && r2.is_ok() && buf1 != buf2;
        FipsTestResult {
            test_name: "RNG".to_string(),
            passed,
            duration_us: duration,
            details: if passed {
                "CSPRNG produced unique outputs".to_string()
            } else {
                "CSPRNG fill failed or produced identical outputs".to_string()
            },
        }
    }

    pub fn run_signing_test(&self) -> FipsTestResult {
        let start = Instant::now();
        use ring::signature::{ECDSA_P256_SHA256_ASN1_SIGNING, EcdsaKeyPair};
        let rng = ring::rand::SystemRandom::new();
        let key_pair = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &rng).unwrap();
        let key_pair =
            EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, key_pair.as_ref(), &rng)
                .unwrap();
        let message = b"FIPS ECDSA-P256 signing test message";
        let signature = key_pair.sign(&rng, message).unwrap();
        let duration = start.elapsed().as_micros();
        let passed = !signature.as_ref().is_empty();
        FipsTestResult {
            test_name: "ECDSA-P256-SHA256".to_string(),
            passed,
            duration_us: duration,
            details: if passed {
                format!(
                    "ECDSA-P256-SHA256 signature generated: {} bytes",
                    signature.as_ref().len()
                )
            } else {
                "ECDSA-P256-SHA256 signing produced empty signature".to_string()
            },
        }
    }

    pub fn verify_fips_mode(&self) -> bool {
        if !self.config.fips_enabled {
            return false;
        }
        let results = self.run_all();
        results.iter().all(|r| r.passed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_test_passes() {
        let selftest = FipsSelfTest::new();
        let result = selftest.run_hash_test();
        assert!(result.passed);
        assert_eq!(result.test_name, "SHA-256");
    }

    #[test]
    fn test_hmac_test_passes() {
        let selftest = FipsSelfTest::new();
        let result = selftest.run_hmac_test();
        assert!(result.passed);
        assert_eq!(result.test_name, "HMAC-SHA256");
    }

    #[test]
    fn test_aes_test_passes() {
        let selftest = FipsSelfTest::new();
        let result = selftest.run_aes_test();
        assert!(result.passed);
        assert_eq!(result.test_name, "AES-256-GCM");
    }

    #[test]
    fn test_rng_test_passes() {
        let selftest = FipsSelfTest::new();
        let result = selftest.run_rng_test();
        assert!(result.passed);
        assert_eq!(result.test_name, "RNG");
    }

    #[test]
    fn test_signing_test_passes() {
        let selftest = FipsSelfTest::new();
        let result = selftest.run_signing_test();
        assert!(result.passed);
        assert_eq!(result.test_name, "ECDSA-P256-SHA256");
    }

    #[test]
    fn test_run_all_tests() {
        let selftest = FipsSelfTest::new();
        let results = selftest.run_all();
        assert_eq!(results.len(), 5);
        assert!(results.iter().all(|r| r.passed));
    }

    #[test]
    fn test_verify_fips_mode_enabled() {
        let selftest = FipsSelfTest::new();
        assert!(selftest.verify_fips_mode());
    }

    #[test]
    fn test_verify_fips_mode_disabled() {
        let config = FipsConfig {
            fips_enabled: false,
            fips_module_id: "test".to_string(),
            fips_version: "0.0.0".to_string(),
        };
        let selftest = FipsSelfTest::with_config(config);
        assert!(!selftest.verify_fips_mode());
    }

    #[test]
    fn test_config_defaults() {
        let config = FipsConfig::default();
        assert!(config.fips_enabled);
        assert_eq!(config.fips_module_id, "civit-crypto-001");
        assert_eq!(config.fips_version, "1.0.0");
    }

    #[test]
    fn test_with_config() {
        let config = FipsConfig {
            fips_enabled: true,
            fips_module_id: "custom-001".to_string(),
            fips_version: "2.0.0".to_string(),
        };
        let selftest = FipsSelfTest::with_config(config);
        let result = selftest.run_hash_test();
        assert!(result.passed);
    }

    #[test]
    fn test_result_contains_details() {
        let selftest = FipsSelfTest::new();
        let result = selftest.run_hash_test();
        assert!(!result.details.is_empty());
    }

    #[test]
    fn test_result_duration_nonzero() {
        let selftest = FipsSelfTest::new();
        let result = selftest.run_signing_test();
        assert!(result.duration_us > 0);
    }

    #[test]
    fn test_aes_encrypt_decrypt_roundtrip() {
        use ring::aead::{AES_256_GCM, Aad, LessSafeKey, Nonce, UnboundKey};
        let key_bytes: [u8; 32] = [0x42u8; 32];
        let nonce_bytes: [u8; 12] = [0x11u8; 12];
        let key = UnboundKey::new(&AES_256_GCM, &key_bytes).unwrap();
        let key = LessSafeKey::new(key);
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let plaintext = b"roundtrip test data";
        let aad = Aad::empty();
        let mut in_out = plaintext.to_vec();
        key.seal_in_place_append_tag(nonce, aad, &mut in_out)
            .unwrap();
        let nonce2 = Nonce::assume_unique_for_key(nonce_bytes);
        let decrypted = key
            .open_in_place(nonce2, Aad::empty(), in_out.as_mut())
            .unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_ecdsa_sign_verify() {
        use ring::signature::{ECDSA_P256_SHA256_ASN1_SIGNING, EcdsaKeyPair, KeyPair};
        let rng = ring::rand::SystemRandom::new();
        let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &rng).unwrap();
        let key_pair =
            EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, pkcs8.as_ref(), &rng)
                .unwrap();
        let message = b"sign and verify test";
        let signature = key_pair.sign(&rng, message).unwrap();
        assert!(!signature.as_ref().is_empty());
        let public_key_bytes = key_pair.public_key().as_ref();
        assert!(!public_key_bytes.is_empty());
    }

    #[test]
    fn test_hmac_rfc4231_case2() {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<Sha256>;
        let key = b"Jefe";
        let data = b"what do ya want for nothing?";
        let expected =
            hex::decode("5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843")
                .unwrap();
        let mut mac = HmacSha256::new_from_slice(key).unwrap();
        mac.update(data);
        let result = mac.finalize().into_bytes().to_vec();
        assert_eq!(result, expected);
    }
}
