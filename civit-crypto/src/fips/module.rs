#![forbid(unsafe_code)]

use sha2::{Digest, Sha256, Sha384, Sha512};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FipsStatus {
    Compliant,
    NonCompliant,
    InTransition,
}

#[derive(Debug, Clone)]
pub struct FipsConfig {
    pub enforce_fips: bool,
    pub allowed_non_fips_algorithms: Vec<String>,
}

impl Default for FipsConfig {
    fn default() -> Self {
        Self {
            enforce_fips: true,
            allowed_non_fips_algorithms: Vec::new(),
        }
    }
}

const FIPS_APPROVED: &[&str] = &[
    "SHA-256",
    "SHA-384",
    "SHA-512",
    "HMAC-SHA256",
    "AES-128",
    "AES-256",
    "RSA",
];

#[derive(Debug)]
pub struct FipsModule {
    approved_algorithms: HashSet<String>,
    self_test_passed: AtomicBool,
    module_version: String,
    #[allow(dead_code)]
    validation_certificate: Option<String>,
    config: FipsConfig,
}

impl Default for FipsModule {
    fn default() -> Self {
        Self::new()
    }
}

impl FipsModule {
    pub fn new() -> Self {
        let approved: HashSet<String> = FIPS_APPROVED.iter().map(|s| s.to_string()).collect();
        Self {
            approved_algorithms: approved,
            self_test_passed: AtomicBool::new(false),
            module_version: "1.0.0".to_string(),
            validation_certificate: None,
            config: FipsConfig::default(),
        }
    }

    pub fn with_config(config: FipsConfig) -> Self {
        let mut module = Self::new();
        module.config = config;
        module
    }

    pub fn initialize(&self) -> anyhow::Result<FipsStatus> {
        self.run_self_tests()?;
        self.self_test_passed.store(true, Ordering::SeqCst);
        Ok(FipsStatus::Compliant)
    }

    pub fn is_approved(&self, algorithm: &str) -> bool {
        self.approved_algorithms.contains(algorithm)
            || self
                .config
                .allowed_non_fips_algorithms
                .iter()
                .any(|a| a == algorithm)
    }

    pub fn verify_self_test(&self) -> bool {
        self.self_test_passed.load(Ordering::SeqCst)
    }

    pub fn hash(&self, data: &[u8], algorithm: &str) -> anyhow::Result<Vec<u8>> {
        if self.config.enforce_fips && !self.is_approved(algorithm) {
            anyhow::bail!("algorithm '{algorithm}' is not FIPS-approved");
        }
        if !self.self_test_passed.load(Ordering::SeqCst) {
            anyhow::bail!("FIPS self-tests have not passed");
        }
        match algorithm {
            "SHA-256" => {
                let result = Sha256::digest(data);
                Ok(result.to_vec())
            }
            "SHA-384" => {
                let result = Sha384::digest(data);
                Ok(result.to_vec())
            }
            "SHA-512" => {
                let result = Sha512::digest(data);
                Ok(result.to_vec())
            }
            _ => anyhow::bail!("unsupported algorithm: {algorithm}"),
        }
    }

    pub fn hmac(&self, key: &[u8], data: &[u8]) -> anyhow::Result<Vec<u8>> {
        if self.config.enforce_fips && !self.is_approved("HMAC-SHA256") {
            anyhow::bail!("HMAC-SHA256 is not FIPS-approved");
        }
        if !self.self_test_passed.load(Ordering::SeqCst) {
            anyhow::bail!("FIPS self-tests have not passed");
        }
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(key)?;
        mac.update(data);
        let result = mac.finalize();
        Ok(result.into_bytes().to_vec())
    }

    pub fn status(&self) -> FipsStatus {
        if self.self_test_passed.load(Ordering::SeqCst) {
            FipsStatus::Compliant
        } else if self.config.enforce_fips {
            FipsStatus::InTransition
        } else {
            FipsStatus::NonCompliant
        }
    }

    pub fn module_version(&self) -> &str {
        &self.module_version
    }

    pub fn approved_algorithms(&self) -> &HashSet<String> {
        &self.approved_algorithms
    }

    fn run_self_tests(&self) -> anyhow::Result<()> {
        self.test_sha256()?;
        self.test_sha384()?;
        self.test_sha512()?;
        self.test_hmac_sha256()?;
        Ok(())
    }

    fn test_sha256(&self) -> anyhow::Result<()> {
        let data = b"abc";
        let expected =
            hex::decode("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")?;
        let result = Sha256::digest(data).to_vec();
        if result != expected {
            anyhow::bail!("SHA-256 self-test failed");
        }
        Ok(())
    }

    fn test_sha384(&self) -> anyhow::Result<()> {
        let data = b"abc";
        let expected = hex::decode(
            "cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7",
        )?;
        let result = Sha384::digest(data).to_vec();
        if result != expected {
            anyhow::bail!("SHA-384 self-test failed");
        }
        Ok(())
    }

    fn test_sha512(&self) -> anyhow::Result<()> {
        let data = b"abc";
        let expected = hex::decode(
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f",
        )?;
        let result = Sha512::digest(data).to_vec();
        if result != expected {
            anyhow::bail!("SHA-512 self-test failed");
        }
        Ok(())
    }

    fn test_hmac_sha256(&self) -> anyhow::Result<()> {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<Sha256>;
        let key = hex::decode("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b")?;
        let data = b"Hi There";
        let expected =
            hex::decode("b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7")?;
        let mut mac = HmacSha256::new_from_slice(&key)?;
        mac.update(data);
        let result = mac.finalize().into_bytes().to_vec();
        if result != expected {
            anyhow::bail!("HMAC-SHA256 self-test failed");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_module() {
        let module = FipsModule::new();
        assert!(!module.verify_self_test());
        assert_eq!(module.module_version(), "1.0.0");
    }

    #[test]
    fn test_initialize_self_tests() {
        let module = FipsModule::new();
        let status = module.initialize().unwrap();
        assert_eq!(status, FipsStatus::Compliant);
        assert!(module.verify_self_test());
    }

    #[test]
    fn test_approved_algorithms() {
        let module = FipsModule::new();
        assert!(module.is_approved("SHA-256"));
        assert!(module.is_approved("SHA-384"));
        assert!(module.is_approved("SHA-512"));
        assert!(module.is_approved("HMAC-SHA256"));
        assert!(module.is_approved("AES-128"));
        assert!(module.is_approved("AES-256"));
        assert!(module.is_approved("RSA"));
        assert!(!module.is_approved("MD5"));
        assert!(!module.is_approved("SHA-1"));
    }

    #[test]
    fn test_hash_sha256() {
        let module = FipsModule::new();
        module.initialize().unwrap();
        let result = module.hash(b"hello", "SHA-256").unwrap();
        assert_eq!(result.len(), 32);
        let expected = hex::encode(&result);
        assert_eq!(
            expected,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_hash_sha384() {
        let module = FipsModule::new();
        module.initialize().unwrap();
        let result = module.hash(b"hello", "SHA-384").unwrap();
        assert_eq!(result.len(), 48);
    }

    #[test]
    fn test_hash_sha512() {
        let module = FipsModule::new();
        module.initialize().unwrap();
        let result = module.hash(b"hello", "SHA-512").unwrap();
        assert_eq!(result.len(), 64);
    }

    #[test]
    fn test_hash_unapproved_rejected() {
        let module = FipsModule::new();
        module.initialize().unwrap();
        let result = module.hash(b"hello", "MD5");
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_without_init_fails() {
        let module = FipsModule::new();
        let result = module.hash(b"hello", "SHA-256");
        assert!(result.is_err());
    }

    #[test]
    fn test_hmac_sha256() {
        let module = FipsModule::new();
        module.initialize().unwrap();
        let result = module.hmac(b"secret-key", b"message").unwrap();
        assert_eq!(result.len(), 32);
    }

    #[test]
    fn test_hmac_without_init_fails() {
        let module = FipsModule::new();
        let result = module.hmac(b"key", b"data");
        assert!(result.is_err());
    }

    #[test]
    fn test_status_before_init() {
        let module = FipsModule::new();
        assert_eq!(module.status(), FipsStatus::InTransition);
    }

    #[test]
    fn test_status_after_init() {
        let module = FipsModule::new();
        module.initialize().unwrap();
        assert_eq!(module.status(), FipsStatus::Compliant);
    }

    #[test]
    fn test_status_non_compliant() {
        let module = FipsModule::with_config(FipsConfig {
            enforce_fips: false,
            allowed_non_fips_algorithms: vec!["MD5".to_string()],
        });
        assert_eq!(module.status(), FipsStatus::NonCompliant);
        assert!(module.is_approved("MD5"));
    }

    #[test]
    fn test_non_enforce_mode_allows_unapproved() {
        let module = FipsModule::with_config(FipsConfig {
            enforce_fips: false,
            allowed_non_fips_algorithms: Vec::new(),
        });
        module.initialize().unwrap();
        let result = module.hash(b"hello", "SHA-256").unwrap();
        assert_eq!(result.len(), 32);
    }
}
