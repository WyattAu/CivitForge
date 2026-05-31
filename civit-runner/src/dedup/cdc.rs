#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdcConfig {
    pub min_chunk_size: usize,
    pub max_chunk_size: usize,
    pub target_chunk_size: usize,
    pub window_size: usize,
}

impl Default for CdcConfig {
    fn default() -> Self {
        Self {
            min_chunk_size: 4 * 1024,
            max_chunk_size: 64 * 1024,
            target_chunk_size: 32 * 1024,
            window_size: 48,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Chunk {
    pub id: String,
    pub offset: u64,
    pub size: usize,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkResult {
    pub chunks: Vec<Chunk>,
    pub total_size: u64,
}

pub struct FastCdc {
    config: CdcConfig,
    mask: u64,
}

impl FastCdc {
    pub fn new(config: CdcConfig) -> Self {
        let target = config.target_chunk_size as u64;
        let bits = 64 - target.leading_zeros();
        let mask = (1u64 << bits) - 1;
        Self { config, mask }
    }

    pub fn chunk_data(&self, data: &[u8]) -> ChunkResult {
        let mut chunks = Vec::new();
        let data_len = data.len();
        if data_len == 0 {
            return ChunkResult {
                chunks,
                total_size: 0,
            };
        }

        let mut pos = 0usize;
        while pos < data_len {
            let remaining = data_len - pos;
            if remaining <= self.config.min_chunk_size {
                let chunk_data = &data[pos..];
                let id = compute_digest(chunk_data);
                chunks.push(Chunk {
                    id,
                    offset: pos as u64,
                    size: chunk_data.len(),
                    digest: compute_digest(chunk_data),
                });
                break;
            }

            let cut_point = self.find_boundary(&data[pos..], remaining);
            let chunk_data = &data[pos..pos + cut_point];
            let id = compute_digest(chunk_data);
            chunks.push(Chunk {
                id,
                offset: pos as u64,
                size: cut_point,
                digest: compute_digest(chunk_data),
            });
            pos += cut_point;
        }

        let total_size = chunks.iter().map(|c| c.size as u64).sum();
        ChunkResult { chunks, total_size }
    }

    fn find_boundary(&self, data: &[u8], remaining: usize) -> usize {
        let mut hash: u64 = 0;
        let max = self.config.max_chunk_size.min(remaining);

        for i in 0..max {
            hash = hash.wrapping_add(GEAR_TABLE[data[i] as usize]);

            if i >= self.config.min_chunk_size && hash & self.mask == 0 {
                return i;
            }
        }

        max
    }
}

impl Default for FastCdc {
    fn default() -> Self {
        Self::new(CdcConfig::default())
    }
}

#[allow(dead_code)]
fn gear_hash(data: &[u8], mask: u64) -> u64 {
    let mut hash: u64 = 0;
    for &byte in data {
        hash = hash.wrapping_add(GEAR_TABLE[byte as usize]);
    }
    hash & mask
}

pub fn compute_digest(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

const GEAR_TABLE: [u64; 256] = {
    let mut table = [0u64; 256];
    let mut i = 0u32;
    while i < 256 {
        let mut h: u64 = (i as u64).wrapping_mul(0x5bd1e995);
        h ^= h >> 24;
        h = h.wrapping_mul(0x5bd1e995);
        h ^= h >> 24;
        h = h.wrapping_mul(0x5bd1e995);
        h ^= h >> 24;
        table[i as usize] = h;
        i += 1;
    }
    table
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_chunking() {
        let cdc = FastCdc::default();
        let data = vec![0u8; 128 * 1024];
        let result1 = cdc.chunk_data(&data);
        let result2 = cdc.chunk_data(&data);
        assert_eq!(result1.chunks.len(), result2.chunks.len());
        for (a, b) in result1.chunks.iter().zip(result2.chunks.iter()) {
            assert_eq!(a.offset, b.offset);
            assert_eq!(a.size, b.size);
            assert_eq!(a.digest, b.digest);
        }
    }

    #[test]
    fn test_identical_data_identical_chunks() {
        let cdc = FastCdc::default();
        let data = b"hello world this is a test of content defined chunking".repeat(100);
        let copy = data.clone();
        let r1 = cdc.chunk_data(&data);
        let r2 = cdc.chunk_data(&copy);
        assert_eq!(r1.chunks, r2.chunks);
    }

    #[test]
    fn test_different_data_different_boundaries() {
        let config = CdcConfig {
            min_chunk_size: 256,
            max_chunk_size: 2 * 1024,
            target_chunk_size: 512,
            window_size: 48,
        };
        let cdc = FastCdc::new(config);
        let text_a = b"The quick brown fox jumps over the lazy dog.";
        let data_a: Vec<u8> = text_a.iter().cycle().take(64 * 1024).copied().collect();
        let text_b = b"Sphinx of black quartz, judge my vow.";
        let data_b: Vec<u8> = text_b.iter().cycle().take(64 * 1024).copied().collect();
        let r_a = cdc.chunk_data(&data_a);
        let r_b = cdc.chunk_data(&data_b);
        let offsets_a: Vec<u64> = r_a.chunks.iter().map(|c| c.offset).collect();
        let offsets_b: Vec<u64> = r_b.chunks.iter().map(|c| c.offset).collect();
        assert_ne!(
            offsets_a, offsets_b,
            "different data should produce different boundaries"
        );
        assert!(r_a.chunks.len() > 2, "should produce multiple chunks");
    }

    #[test]
    fn test_min_chunk_size_enforced() {
        let config = CdcConfig {
            min_chunk_size: 8 * 1024,
            max_chunk_size: 64 * 1024,
            target_chunk_size: 32 * 1024,
            window_size: 48,
        };
        let cdc = FastCdc::new(config);
        let data = vec![0u8; 128 * 1024];
        let result = cdc.chunk_data(&data);
        for chunk in &result.chunks {
            assert!(chunk.size >= 8 * 1024 || chunk.offset as usize + chunk.size == data.len());
        }
    }

    #[test]
    fn test_max_chunk_size_enforced() {
        let config = CdcConfig {
            min_chunk_size: 1024,
            max_chunk_size: 8 * 1024,
            target_chunk_size: 4 * 1024,
            window_size: 48,
        };
        let cdc = FastCdc::new(config);
        let data = vec![0u8; 64 * 1024];
        let result = cdc.chunk_data(&data);
        for chunk in &result.chunks {
            assert!(chunk.size <= 8 * 1024);
        }
    }

    #[test]
    fn test_empty_data() {
        let cdc = FastCdc::default();
        let result = cdc.chunk_data(&[]);
        assert!(result.chunks.is_empty());
        assert_eq!(result.total_size, 0);
    }

    #[test]
    fn test_small_data() {
        let cdc = FastCdc::default();
        let data = b"hello";
        let result = cdc.chunk_data(data);
        assert_eq!(result.chunks.len(), 1);
        assert_eq!(result.chunks[0].size, 5);
        assert_eq!(result.total_size, 5);
    }

    #[test]
    fn test_gear_hash_deterministic() {
        let data = b"test data";
        let mask = 0x7FFF;
        let h1 = gear_hash(data, mask);
        let h2 = gear_hash(data, mask);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_compute_digest_deterministic() {
        let data = b"test";
        let d1 = compute_digest(data);
        let d2 = compute_digest(data);
        assert_eq!(d1, d2);
        assert_eq!(d1.len(), 64);
    }

    #[test]
    fn test_total_size_matches() {
        let cdc = FastCdc::default();
        let data = vec![42u8; 100 * 1024];
        let result = cdc.chunk_data(&data);
        assert_eq!(result.total_size, data.len() as u64);
    }
}
