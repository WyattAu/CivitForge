#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Chunk produced by content-defined chunking.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub offset: usize,
    pub length: usize,
    pub content: Vec<u8>,
    pub hash: String,
}

/// File manifest for reconstruction.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChunkManifest {
    pub file_hash: String,
    pub file_size: u64,
    pub chunks: Vec<ChunkInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChunkInfo {
    pub chunk_hash: String,
    pub offset: usize,
    pub length: usize,
}

/// Configuration for content-defined chunking.
#[derive(Debug, Clone)]
pub struct ChunkerConfig {
    pub min_chunk_size: usize,
    pub max_chunk_size: usize,
    pub target_chunk_size: usize,
    pub mask: u32,
}

impl Default for ChunkerConfig {
    fn default() -> Self {
        Self {
            min_chunk_size: 4 * 1024,
            max_chunk_size: 64 * 1024,
            target_chunk_size: 16 * 1024,
            mask: 0x1FFF,
        }
    }
}

/// Content-defined chunker using a simple rolling hash.
pub struct ContentDefinedChunker {
    config: ChunkerConfig,
}

impl ContentDefinedChunker {
    pub fn new(config: ChunkerConfig) -> Self {
        Self { config }
    }

    /// Split data into variable-length chunks.
    pub fn chunk(&self, data: &[u8]) -> Vec<Chunk> {
        if data.is_empty() {
            return Vec::new();
        }

        let mut chunks = Vec::new();
        let mut pos = 0;
        let data_len = data.len();

        while pos < data_len {
            let remaining = data_len - pos;
            let min_end = pos + self.config.min_chunk_size.min(remaining);

            if min_end >= data_len {
                chunks.push(self.make_chunk(data, pos, data_len));
                break;
            }

            let max_end = pos + self.config.max_chunk_size.min(remaining);
            let boundary = self.find_boundary(data, min_end, max_end);
            chunks.push(self.make_chunk(data, pos, boundary));
            pos = boundary;
        }

        chunks
    }

    fn find_boundary(&self, data: &[u8], min_pos: usize, max_pos: usize) -> usize {
        let mut hash: u32 = 0;
        let window_size = 48;

        let start = min_pos.saturating_sub(window_size);
        for byte in data.iter().take(min_pos).skip(start) {
            hash = hash.wrapping_mul(31).wrapping_add(*byte as u32);
        }

        let mut pos = min_pos;
        while pos < max_pos {
            hash = hash.wrapping_mul(31).wrapping_add(data[pos] as u32);

            if (hash & self.config.mask) == 0 {
                return pos + 1;
            }

            if pos >= start + window_size {
                let old_byte = data[pos - window_size] as u32;
                let power = 31u32.wrapping_pow(window_size as u32 - 1);
                hash = hash.wrapping_sub(old_byte.wrapping_mul(power));
            }

            pos += 1;
        }

        max_pos
    }

    fn make_chunk(&self, data: &[u8], start: usize, end: usize) -> Chunk {
        let content = data[start..end].to_vec();
        let hash = Self::hash_content(&content);
        Chunk {
            offset: start,
            length: end - start,
            content,
            hash,
        }
    }

    fn hash_content(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    /// Create a chunk manifest for a file.
    pub fn create_manifest(&self, data: &[u8]) -> ChunkManifest {
        let file_hash = Self::hash_content(data);
        let chunks = self.chunk(data);
        let chunk_infos: Vec<ChunkInfo> = chunks
            .iter()
            .map(|c| ChunkInfo {
                chunk_hash: c.hash.clone(),
                offset: c.offset,
                length: c.length,
            })
            .collect();
        ChunkManifest {
            file_hash,
            file_size: data.len() as u64,
            chunks: chunk_infos,
        }
    }

    /// Reconstruct file data from chunks and manifest.
    pub fn reconstruct(chunks: &[Chunk]) -> Vec<u8> {
        let mut data = Vec::new();
        for chunk in chunks {
            data.extend_from_slice(&chunk.content);
        }
        data
    }

    /// Deduplicate chunks across multiple files.
    /// Returns unique chunks and a mapping for each file.
    pub fn deduplicate(&self, files: &[&[u8]]) -> (HashMap<String, Chunk>, Vec<ChunkManifest>) {
        let mut chunk_store: HashMap<String, Chunk> = HashMap::new();
        let mut manifests = Vec::new();

        for data in files {
            let manifest = self.create_manifest(data);
            for chunk in self.chunk(data) {
                chunk_store
                    .entry(chunk.hash.clone())
                    .or_insert_with(|| chunk);
            }
            manifests.push(manifest);
        }

        (chunk_store, manifests)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_small_file() {
        let chunker = ContentDefinedChunker::new(ChunkerConfig {
            min_chunk_size: 64,
            max_chunk_size: 256,
            target_chunk_size: 128,
            mask: 0x1FFF,
        });
        let data = vec![0u8; 512];
        let chunks = chunker.chunk(&data);
        assert!(!chunks.is_empty());
        let total: usize = chunks.iter().map(|c| c.length).sum();
        assert_eq!(total, data.len());
        for c in &chunks {
            assert!(c.length >= 64 || c.offset + c.length == data.len());
        }
    }

    #[test]
    fn test_chunk_empty_data() {
        let chunker = ContentDefinedChunker::new(ChunkerConfig::default());
        let chunks = chunker.chunk(&[]);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_chunk_below_min_size() {
        let chunker = ContentDefinedChunker::new(ChunkerConfig::default());
        let data = vec![42u8; 128];
        let chunks = chunker.chunk(&data);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].length, 128);
        assert_eq!(chunks[0].offset, 0);
    }

    #[test]
    fn test_hash_consistency() {
        let chunker = ContentDefinedChunker::new(ChunkerConfig {
            min_chunk_size: 4,
            max_chunk_size: 32,
            target_chunk_size: 8,
            mask: 0x3,
        });
        let data = b"hello world this is a test of content defined chunking";
        let chunks1 = chunker.chunk(data);
        let chunks2 = chunker.chunk(data);
        assert_eq!(chunks1.len(), chunks2.len());
        for (a, b) in chunks1.iter().zip(chunks2.iter()) {
            assert_eq!(a.hash, b.hash);
            assert_eq!(a.content, b.content);
        }
    }

    #[test]
    fn test_manifest_creation() {
        let chunker = ContentDefinedChunker::new(ChunkerConfig::default());
        let data = vec![7u8; 8192];
        let manifest = chunker.create_manifest(&data);
        assert_eq!(manifest.file_size, data.len() as u64);
        assert!(!manifest.file_hash.is_empty());
        let total: usize = manifest.chunks.iter().map(|c| c.length).sum();
        assert_eq!(total, data.len());
    }

    #[test]
    fn test_reconstruction() {
        let chunker = ContentDefinedChunker::new(ChunkerConfig {
            min_chunk_size: 32,
            max_chunk_size: 128,
            target_chunk_size: 64,
            mask: 0xFF,
        });
        let data = b"reconstruction test data that is somewhat long enough to be chunked";
        let chunks = chunker.chunk(data);
        let reconstructed = ContentDefinedChunker::reconstruct(&chunks);
        assert_eq!(reconstructed, data);
    }

    #[test]
    fn test_deduplication() {
        let chunker = ContentDefinedChunker::new(ChunkerConfig {
            min_chunk_size: 16,
            max_chunk_size: 64,
            target_chunk_size: 32,
            mask: 0xF,
        });
        let data_a = vec![1u8; 256];
        let data_b = vec![1u8; 256];
        let (store, manifests) = chunker.deduplicate(&[&data_a, &data_b]);
        let chunks_a = chunker.chunk(&data_a);
        let _chunks_b = chunker.chunk(&data_b);
        let expected_unique = chunks_a.len();
        assert!(store.len() <= expected_unique);
        assert_eq!(manifests.len(), 2);
    }

    #[test]
    fn test_custom_config() {
        let small = ContentDefinedChunker::new(ChunkerConfig {
            min_chunk_size: 8,
            max_chunk_size: 16,
            target_chunk_size: 12,
            mask: 0x1,
        });
        let data = vec![0xAA; 128];
        let chunks = small.chunk(&data);
        for c in &chunks {
            if c.offset + c.length < data.len() {
                assert!(c.length >= 8 && c.length <= 16);
            }
        }
    }
}
