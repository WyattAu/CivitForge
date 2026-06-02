#![forbid(unsafe_code)]

use sha2::{Digest, Sha256, Sha512};
use tracing::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Sha256,
    Sha512,
}

impl HashAlgorithm {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
            Self::Sha512 => "sha512",
        }
    }

    pub fn digest_length(&self) -> usize {
        match self {
            Self::Sha256 => 32,
            Self::Sha512 => 64,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HashResult {
    pub algorithm: HashAlgorithm,
    pub hex: String,
    pub base64: String,
    pub bytes: Vec<u8>,
}

pub struct HashService;

impl Default for HashService {
    fn default() -> Self {
        Self::new()
    }
}

impl HashService {
    pub fn new() -> Self {
        Self
    }

    pub fn hash(algorithm: HashAlgorithm, data: &[u8]) -> HashResult {
        let bytes = match algorithm {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                hasher.finalize().to_vec()
            }
            HashAlgorithm::Sha512 => {
                let mut hasher = Sha512::new();
                hasher.update(data);
                hasher.finalize().to_vec()
            }
        };
        let hex = hex::encode(&bytes);
        let base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
        debug!(algo = %algorithm.name(), len = data.len(), "computed hash");
        HashResult {
            algorithm,
            hex,
            base64,
            bytes,
        }
    }

    pub fn hash_string(algorithm: HashAlgorithm, text: &str) -> HashResult {
        Self::hash(algorithm, text.as_bytes())
    }

    pub fn hash_file(
        algorithm: HashAlgorithm,
        path: &std::path::Path,
    ) -> std::io::Result<HashResult> {
        let data = std::fs::read(path)?;
        Ok(Self::hash(algorithm, &data))
    }

    pub fn verify(data: &[u8], expected_hex: &str) -> bool {
        let algorithms = [HashAlgorithm::Sha256, HashAlgorithm::Sha512];
        for algo in algorithms {
            let result = Self::hash(algo, data);
            if result.hex == expected_hex {
                return true;
            }
        }
        false
    }

    pub fn verify_with_algorithm(
        algorithm: HashAlgorithm,
        data: &[u8],
        expected_hex: &str,
    ) -> bool {
        let result = Self::hash(algorithm, data);
        result.hex == expected_hex
    }

    pub fn merkle_root(hashes: &[&str]) -> Option<String> {
        if hashes.is_empty() {
            return None;
        }
        let mut current: Vec<String> = hashes.iter().map(|h| h.to_string()).collect();
        while current.len() > 1 {
            let mut next = Vec::new();
            for chunk in current.chunks(2) {
                if chunk.len() == 2 {
                    let combined = format!("{}{}", chunk[0], chunk[1]);
                    let result = Self::hash(HashAlgorithm::Sha256, combined.as_bytes());
                    next.push(result.hex);
                } else {
                    next.push(chunk[0].clone());
                }
            }
            current = next;
        }
        current.into_iter().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hash() {
        let result = HashService::hash(HashAlgorithm::Sha256, b"hello");
        assert_eq!(result.hex.len(), 64);
        assert_eq!(result.bytes.len(), 32);
        assert!(!result.hex.is_empty());
    }

    #[test]
    fn test_sha512_hash() {
        let result = HashService::hash(HashAlgorithm::Sha512, b"hello");
        assert_eq!(result.hex.len(), 128);
        assert_eq!(result.bytes.len(), 64);
    }

    #[test]
    fn test_deterministic() {
        let r1 = HashService::hash(HashAlgorithm::Sha256, b"test");
        let r2 = HashService::hash(HashAlgorithm::Sha256, b"test");
        assert_eq!(r1.hex, r2.hex);
    }

    #[test]
    fn test_different_inputs() {
        let r1 = HashService::hash(HashAlgorithm::Sha256, b"a");
        let r2 = HashService::hash(HashAlgorithm::Sha256, b"b");
        assert_ne!(r1.hex, r2.hex);
    }

    #[test]
    fn test_hash_string() {
        let r1 = HashService::hash(HashAlgorithm::Sha256, b"hello");
        let r2 = HashService::hash_string(HashAlgorithm::Sha256, "hello");
        assert_eq!(r1.hex, r2.hex);
    }

    #[test]
    fn test_verify() {
        let expected = HashService::hash(HashAlgorithm::Sha256, b"verify me").hex;
        assert!(HashService::verify(b"verify me", &expected));
        assert!(!HashService::verify(b"not me", &expected));
    }

    #[test]
    fn test_verify_with_algorithm() {
        let expected = HashService::hash(HashAlgorithm::Sha512, b"data").hex;
        assert!(HashService::verify_with_algorithm(
            HashAlgorithm::Sha512,
            b"data",
            &expected
        ));
    }

    #[test]
    fn test_merkle_root_single() {
        let hash = HashService::hash(HashAlgorithm::Sha256, b"leaf").hex;
        let root = HashService::merkle_root(&[&hash]);
        assert!(root.is_some());
        assert_eq!(root.unwrap(), hash);
    }

    #[test]
    fn test_merkle_root_multiple() {
        let h1 = HashService::hash(HashAlgorithm::Sha256, b"a").hex;
        let h2 = HashService::hash(HashAlgorithm::Sha256, b"b").hex;
        let root = HashService::merkle_root(&[&h1, &h2]);
        assert!(root.is_some());
        assert_ne!(root.unwrap(), h1);
    }

    #[test]
    fn test_merkle_root_empty() {
        assert!(HashService::merkle_root(&[]).is_none());
    }

    #[test]
    fn test_algorithm_properties() {
        assert_eq!(HashAlgorithm::Sha256.name(), "sha256");
        assert_eq!(HashAlgorithm::Sha256.digest_length(), 32);
        assert_eq!(HashAlgorithm::Sha512.digest_length(), 64);
    }

    #[test]
    fn test_hash_file() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), b"file content").unwrap();
        let result = HashService::hash_file(HashAlgorithm::Sha256, tmp.path()).unwrap();
        let direct = HashService::hash(HashAlgorithm::Sha256, b"file content");
        assert_eq!(result.hex, direct.hex);
    }
}
