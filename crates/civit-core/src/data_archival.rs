#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use parking_lot::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveConfig {
    pub default_retention_days: i32,
    pub archive_dir: String,
    pub compression_enabled: bool,
    pub max_archive_size_bytes: i64,
}

impl Default for ArchiveConfig {
    fn default() -> Self {
        Self {
            default_retention_days: 365,
            archive_dir: "/var/lib/civitforge/archives".to_string(),
            compression_enabled: true,
            max_archive_size_bytes: 10 * 1024 * 1024 * 1024, // 10 GB
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveRequest {
    pub repo_id: String,
    pub archive_type: String,
    pub retention_days: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveResult {
    pub archive_id: String,
    pub file_path: String,
    pub file_size_bytes: i64,
    pub duration_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub archive_type: String,
    pub retention_days: i32,
    pub max_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveStats {
    pub total_archives: i64,
    pub total_size_bytes: i64,
    pub expired_count: i64,
    pub by_type: Vec<(String, i64)>,
}

pub trait ArchiveManager: Send + Sync {
    fn create_archive(&self, request: &ArchiveRequest) -> Result<ArchiveResult, String>;
    fn delete_archive(&self, archive_id: &str) -> Result<(), String>;
    fn enforce_retention(&self, policies: &[RetentionPolicy]) -> Result<i64, String>;
    fn get_stats(&self) -> Result<ArchiveStats, String>;
}

pub struct InMemoryArchiveManager {
    #[allow(clippy::type_complexity)]
    archives: Mutex<Vec<(String, String, i64, DateTime<Utc>)>>,
}

impl InMemoryArchiveManager {
    pub fn new() -> Self {
        Self {
            archives: Mutex::new(Vec::new()),
        }
    }
}

impl Default for InMemoryArchiveManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchiveManager for InMemoryArchiveManager {
    fn create_archive(&self, request: &ArchiveRequest) -> Result<ArchiveResult, String> {
        let start = std::time::Instant::now();
        let id = uuid::Uuid::new_v4().to_string();
        let file_path = format!("/{}/{}.tar.gz", "/var/lib/civitforge/archives", id);

        let mut archives = self.archives.lock();
        archives.push((
            id.clone(),
            request.archive_type.clone(),
            1024,
            Utc::now(),
        ));

        Ok(ArchiveResult {
            archive_id: id,
            file_path,
            file_size_bytes: 1024,
            duration_ms: start.elapsed().as_millis() as u64,
            success: true,
            error: None,
        })
    }

    fn delete_archive(&self, archive_id: &str) -> Result<(), String> {
        let mut archives = self.archives.lock();
        let before = archives.len();
        archives.retain(|(id, _, _, _)| id != archive_id);
        if archives.len() < before {
            Ok(())
        } else {
            Err(format!("archive {archive_id} not found"))
        }
    }

    fn enforce_retention(&self, _policies: &[RetentionPolicy]) -> Result<i64, String> {
        Ok(0)
    }

    fn get_stats(&self) -> Result<ArchiveStats, String> {
        let archives = self.archives.lock();
        Ok(ArchiveStats {
            total_archives: archives.len() as i64,
            total_size_bytes: archives.iter().map(|(_, _, size, _)| size).sum(),
            expired_count: 0,
            by_type: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_config_default() {
        let config = ArchiveConfig::default();
        assert_eq!(config.default_retention_days, 365);
        assert!(config.compression_enabled);
        assert_eq!(config.max_archive_size_bytes, 10 * 1024 * 1024 * 1024);
    }

    #[test]
    fn test_in_memory_create_archive() {
        let mgr = InMemoryArchiveManager::new();
        let request = ArchiveRequest {
            repo_id: "repo-1".to_string(),
            archive_type: "code".to_string(),
            retention_days: Some(90),
        };
        let result = mgr.create_archive(&request).unwrap();
        assert!(result.success);
        assert!(!result.archive_id.is_empty());
    }

    #[test]
    fn test_in_memory_delete_archive() {
        let mgr = InMemoryArchiveManager::new();
        let request = ArchiveRequest {
            repo_id: "repo-1".to_string(),
            archive_type: "code".to_string(),
            retention_days: None,
        };
        let result = mgr.create_archive(&request).unwrap();
        assert!(mgr.delete_archive(&result.archive_id).is_ok());
    }

    #[test]
    fn test_in_memory_delete_nonexistent() {
        let mgr = InMemoryArchiveManager::new();
        assert!(mgr.delete_archive("nope").is_err());
    }

    #[test]
    fn test_in_memory_stats() {
        let mgr = InMemoryArchiveManager::new();
        let stats = mgr.get_stats().unwrap();
        assert_eq!(stats.total_archives, 0);
    }
}
