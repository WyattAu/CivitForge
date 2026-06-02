#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq)]
pub struct ShardConfig {
    pub total_shards: u32,
    pub replication_factor: u32,
}

#[derive(Debug, Clone)]
pub struct RepositorySharder {
    config: ShardConfig,
}

impl RepositorySharder {
    pub fn new(config: ShardConfig) -> Self {
        Self { config }
    }

    pub fn get_shard(&self, repo_path: &str) -> u32 {
        let hash = Sha256::digest(repo_path.as_bytes());
        let hash_val = u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]);
        hash_val % self.config.total_shards
    }

    pub fn get_replicas(&self, repo_path: &str) -> Vec<u32> {
        let primary = self.get_shard(repo_path);
        let mut replicas = Vec::with_capacity(self.config.replication_factor as usize);
        replicas.push(primary);

        for i in 1..self.config.replication_factor {
            let mut hasher = Sha256::new();
            hasher.update(repo_path.as_bytes());
            hasher.update(i.to_le_bytes());
            let hash = hasher.finalize();
            let replica_shard =
                u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]) % self.config.total_shards;
            if !replicas.contains(&replica_shard) {
                replicas.push(replica_shard);
            }
        }

        replicas
    }

    pub fn total_shards(&self) -> u32 {
        self.config.total_shards
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sharder(shards: u32, replication: u32) -> RepositorySharder {
        RepositorySharder::new(ShardConfig {
            total_shards: shards,
            replication_factor: replication,
        })
    }

    #[test]
    fn test_deterministic_sharding() {
        let sharder = make_sharder(16, 1);
        let shard1 = sharder.get_shard("users/alice/repo");
        let shard2 = sharder.get_shard("users/alice/repo");
        assert_eq!(shard1, shard2);
    }

    #[test]
    fn test_different_repos_different_shards() {
        let sharder = make_sharder(16, 1);
        let shard_a = sharder.get_shard("users/alice/repo");
        let shard_b = sharder.get_shard("users/bob/repo");
        assert_ne!(shard_a, shard_b);
    }

    #[test]
    fn test_shard_within_range() {
        let sharder = make_sharder(8, 1);
        for i in 0..100 {
            let repo = format!("users/user-{i}/repo");
            let shard = sharder.get_shard(&repo);
            assert!(shard < 8);
        }
    }

    #[test]
    fn test_replicas_include_primary() {
        let sharder = make_sharder(16, 3);
        let replicas = sharder.get_replicas("users/alice/repo");
        let primary = sharder.get_shard("users/alice/repo");
        assert_eq!(replicas[0], primary);
        assert_eq!(replicas.len(), 3);
    }

    #[test]
    fn test_no_duplicate_replicas() {
        let sharder = make_sharder(16, 3);
        let replicas = sharder.get_replicas("users/alice/repo");
        let unique: std::collections::HashSet<u32> = replicas.iter().copied().collect();
        assert_eq!(unique.len(), replicas.len());
    }

    #[test]
    fn test_replicas_deterministic() {
        let sharder = make_sharder(16, 3);
        let r1 = sharder.get_replicas("users/bob/repo");
        let r2 = sharder.get_replicas("users/bob/repo");
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_total_shards() {
        let sharder = make_sharder(64, 2);
        assert_eq!(sharder.total_shards(), 64);
    }

    #[test]
    fn test_replication_one() {
        let sharder = make_sharder(16, 1);
        let replicas = sharder.get_replicas("users/alice/repo");
        assert_eq!(replicas.len(), 1);
    }

    #[test]
    fn test_single_shard() {
        let sharder = make_sharder(1, 1);
        let shard = sharder.get_shard("any/repo");
        assert_eq!(shard, 0);
    }

    #[test]
    fn test_many_repos_distribute() {
        let sharder = make_sharder(8, 1);
        let mut counts = vec![0u32; 8];
        for i in 0..200 {
            let repo = format!("users/user-{i}/repo-{i}");
            let shard = sharder.get_shard(&repo) as usize;
            counts[shard] += 1;
        }
        for count in &counts {
            assert!(*count > 0);
        }
    }
}
