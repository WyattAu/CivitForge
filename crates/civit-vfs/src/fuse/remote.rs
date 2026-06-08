#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockFetchRequest {
    pub block_id: String,
    pub repo: String,
    pub offset: u64,
    pub length: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockFetchResult {
    pub block_id: String,
    pub data: Vec<u8>,
    pub cache_hit: bool,
}

pub trait RemoteBlockProvider: Send + Sync {
    fn fetch_block(&self, request: &BlockFetchRequest) -> Result<BlockFetchResult, String>;
    fn prefetch_blocks(&self, requests: &[BlockFetchRequest]) -> Vec<BlockFetchResult>;
}

pub struct OnDemandFetcher {
    cache: std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>,
    max_cache_size: usize,
}

impl OnDemandFetcher {
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            cache: std::sync::Mutex::new(std::collections::HashMap::new()),
            max_cache_size,
        }
    }

    pub fn fetch_cached(&self, block_id: &str) -> Option<Vec<u8>> {
        let cache = self.cache.lock().ok()?;
        cache.get(block_id).cloned()
    }

    pub fn store_cached(&self, block_id: &str, data: &[u8]) {
        if let Ok(mut cache) = self.cache.lock() {
            if cache.len() < self.max_cache_size {
                cache.insert(block_id.to_string(), data.to_vec());
            }
        }
    }

    pub fn cache_len(&self) -> usize {
        self.cache.lock().map(|c| c.len()).unwrap_or(0)
    }

    pub fn invalidate_cached(&self, block_id: &str) -> bool {
        self.cache
            .lock()
            .map(|mut c| c.remove(block_id).is_some())
            .unwrap_or(false)
    }
}

#[cfg(test)]
pub struct MockRemoteProvider {
    blocks: std::collections::HashMap<String, Vec<u8>>,
}

#[cfg(test)]
impl MockRemoteProvider {
    pub fn new() -> Self {
        Self {
            blocks: std::collections::HashMap::new(),
        }
    }

    pub fn add_block(&mut self, block_id: &str, data: Vec<u8>) {
        self.blocks.insert(block_id.to_string(), data);
    }
}

#[cfg(test)]
impl RemoteBlockProvider for MockRemoteProvider {
    fn fetch_block(&self, request: &BlockFetchRequest) -> Result<BlockFetchResult, String> {
        if let Some(data) = self.blocks.get(&request.block_id) {
            let start = request.offset as usize;
            if start >= data.len() {
                return Ok(BlockFetchResult {
                    block_id: request.block_id.clone(),
                    data: Vec::new(),
                    cache_hit: false,
                });
            }
            let end = (start + request.length as usize).min(data.len());
            let result = BlockFetchResult {
                block_id: request.block_id.clone(),
                data: data[start..end].to_vec(),
                cache_hit: false,
            };
            Ok(result)
        } else {
            Err(format!("block not found: {}", request.block_id))
        }
    }

    fn prefetch_blocks(&self, requests: &[BlockFetchRequest]) -> Vec<BlockFetchResult> {
        requests
            .iter()
            .filter_map(|req| self.fetch_block(req).ok())
            .collect()
    }
}

#[cfg(test)]
impl Default for MockRemoteProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_fetch_request_construction() {
        let req = BlockFetchRequest {
            block_id: "blk-1".into(),
            repo: "my-repo".into(),
            offset: 0,
            length: 4096,
        };
        assert_eq!(req.block_id, "blk-1");
        assert_eq!(req.repo, "my-repo");
    }

    #[test]
    fn test_block_fetch_request_serialization() {
        let req = BlockFetchRequest {
            block_id: "blk-1".into(),
            repo: "repo".into(),
            offset: 100,
            length: 512,
        };
        let json = serde_json::to_string(&req).unwrap();
        let deser: BlockFetchRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.offset, 100);
        assert_eq!(deser.length, 512);
    }

    #[test]
    fn test_block_fetch_result_construction() {
        let result = BlockFetchResult {
            block_id: "blk-1".into(),
            data: vec![1, 2, 3],
            cache_hit: true,
        };
        assert!(result.cache_hit);
        assert_eq!(result.data.len(), 3);
    }

    #[test]
    fn test_block_fetch_result_serialization() {
        let result = BlockFetchResult {
            block_id: "blk-1".into(),
            data: vec![0u8; 10],
            cache_hit: false,
        };
        let json = serde_json::to_string(&result).unwrap();
        let deser: BlockFetchResult = serde_json::from_str(&json).unwrap();
        assert!(!deser.cache_hit);
        assert_eq!(deser.data.len(), 10);
    }

    #[test]
    fn test_mock_remote_provider_fetch() {
        let mut provider = MockRemoteProvider::new();
        provider.add_block("blk-1", b"hello world".to_vec());
        let req = BlockFetchRequest {
            block_id: "blk-1".into(),
            repo: "repo".into(),
            offset: 0,
            length: 5,
        };
        let result = provider.fetch_block(&req).unwrap();
        assert_eq!(result.data, b"hello");
        assert_eq!(result.block_id, "blk-1");
    }

    #[test]
    fn test_mock_remote_provider_fetch_partial() {
        let mut provider = MockRemoteProvider::new();
        provider.add_block("blk-1", b"hello world".to_vec());
        let req = BlockFetchRequest {
            block_id: "blk-1".into(),
            repo: "repo".into(),
            offset: 6,
            length: 5,
        };
        let result = provider.fetch_block(&req).unwrap();
        assert_eq!(result.data, b"world");
    }

    #[test]
    fn test_mock_remote_provider_fetch_not_found() {
        let provider = MockRemoteProvider::new();
        let req = BlockFetchRequest {
            block_id: "missing".into(),
            repo: "repo".into(),
            offset: 0,
            length: 100,
        };
        assert!(provider.fetch_block(&req).is_err());
    }

    #[test]
    fn test_mock_remote_provider_prefetch() {
        let mut provider = MockRemoteProvider::new();
        provider.add_block("blk-1", b"a".to_vec());
        provider.add_block("blk-2", b"bb".to_vec());
        let reqs = vec![
            BlockFetchRequest {
                block_id: "blk-1".into(),
                repo: "repo".into(),
                offset: 0,
                length: 1,
            },
            BlockFetchRequest {
                block_id: "blk-2".into(),
                repo: "repo".into(),
                offset: 0,
                length: 2,
            },
        ];
        let results = provider.prefetch_blocks(&reqs);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_mock_remote_provider_prefetch_with_missing() {
        let mut provider = MockRemoteProvider::new();
        provider.add_block("blk-1", b"a".to_vec());
        let reqs = vec![
            BlockFetchRequest {
                block_id: "blk-1".into(),
                repo: "repo".into(),
                offset: 0,
                length: 1,
            },
            BlockFetchRequest {
                block_id: "missing".into(),
                repo: "repo".into(),
                offset: 0,
                length: 1,
            },
        ];
        let results = provider.prefetch_blocks(&reqs);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_on_demand_fetcher_store_and_fetch() {
        let fetcher = OnDemandFetcher::new(100);
        fetcher.store_cached("blk-1", b"data");
        let cached = fetcher.fetch_cached("blk-1").unwrap();
        assert_eq!(cached, b"data");
    }

    #[test]
    fn test_on_demand_fetcher_miss() {
        let fetcher = OnDemandFetcher::new(100);
        assert!(fetcher.fetch_cached("missing").is_none());
    }

    #[test]
    fn test_on_demand_fetcher_cache_limit() {
        let fetcher = OnDemandFetcher::new(2);
        fetcher.store_cached("blk-1", b"a");
        fetcher.store_cached("blk-2", b"b");
        fetcher.store_cached("blk-3", b"c");
        assert_eq!(fetcher.cache_len(), 2);
        assert!(fetcher.fetch_cached("blk-3").is_none());
    }

    #[test]
    fn test_on_demand_fetcher_invalidate() {
        let fetcher = OnDemandFetcher::new(100);
        fetcher.store_cached("blk-1", b"data");
        assert!(fetcher.invalidate_cached("blk-1"));
        assert!(!fetcher.invalidate_cached("blk-1"));
        assert_eq!(fetcher.cache_len(), 0);
    }

    #[test]
    fn test_fetch_with_offset_beyond_data() {
        let mut provider = MockRemoteProvider::new();
        provider.add_block("blk-1", b"hi".to_vec());
        let req = BlockFetchRequest {
            block_id: "blk-1".into(),
            repo: "repo".into(),
            offset: 100,
            length: 10,
        };
        let result = provider.fetch_block(&req).unwrap();
        assert!(result.data.is_empty());
    }
}
