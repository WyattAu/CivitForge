#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub id: String,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub components: Vec<BackupComponent>,
    pub total_size_bytes: u64,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupComponent {
    pub name: String,
    pub component_type: BackupType,
    pub size_bytes: u64,
    pub checksum: String,
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackupType {
    Database,
    Repositories,
    Artifacts,
    Configuration,
    Secrets,
    Logs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub retention_count: u32,
    pub backup_dir: PathBuf,
    pub compression_enabled: bool,
    pub checksum_algorithm: ChecksumAlgorithm,
    pub include_repositories: bool,
    pub include_artifacts: bool,
    pub include_database: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChecksumAlgorithm {
    Sha256,
    Sha512,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            retention_count: 10,
            backup_dir: PathBuf::from("/var/lib/civitforge/backups"),
            compression_enabled: true,
            checksum_algorithm: ChecksumAlgorithm::Sha256,
            include_repositories: true,
            include_artifacts: true,
            include_database: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResult {
    pub manifest: BackupManifest,
    pub duration_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResult {
    pub backup_id: String,
    pub components_restored: Vec<String>,
    pub duration_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}

pub trait BackupManager: Send + Sync {
    fn create_backup(&self, config: &BackupConfig) -> Result<BackupResult, String>;
    fn restore_backup(
        &self,
        backup_id: &str,
        config: &BackupConfig,
    ) -> Result<RestoreResult, String>;
    fn list_backups(&self) -> Vec<BackupManifest>;
    fn delete_backup(&self, backup_id: &str) -> Result<(), String>;
}

pub struct InMemoryBackupManager {
    backups: std::sync::Mutex<Vec<BackupManifest>>,
}

impl InMemoryBackupManager {
    pub fn new() -> Self {
        Self {
            backups: std::sync::Mutex::new(Vec::new()),
        }
    }
}

impl Default for InMemoryBackupManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BackupManager for InMemoryBackupManager {
    fn create_backup(&self, config: &BackupConfig) -> Result<BackupResult, String> {
        let start = std::time::Instant::now();
        let id = uuid::Uuid::new_v4().to_string();

        let mut components = Vec::new();
        if config.include_database {
            components.push(BackupComponent {
                name: "database".to_string(),
                component_type: BackupType::Database,
                size_bytes: 1024,
                checksum: "db_checksum".to_string(),
                path: format!("{}/{}", config.backup_dir.display(), id),
            });
        }
        if config.include_repositories {
            components.push(BackupComponent {
                name: "repositories".to_string(),
                component_type: BackupType::Repositories,
                size_bytes: 2048,
                checksum: "repo_checksum".to_string(),
                path: format!("{}/{}", config.backup_dir.display(), id),
            });
        }
        if config.include_artifacts {
            components.push(BackupComponent {
                name: "artifacts".to_string(),
                component_type: BackupType::Artifacts,
                size_bytes: 512,
                checksum: "artifact_checksum".to_string(),
                path: format!("{}/{}", config.backup_dir.display(), id),
            });
        }

        let total_size_bytes: u64 = components.iter().map(|c| c.size_bytes).sum();
        let manifest = BackupManifest {
            id: id.clone(),
            version: "1.0".to_string(),
            created_at: Utc::now(),
            components,
            total_size_bytes,
            checksum: format!("sha256_{id}"),
        };

        let mut backups = self.backups.lock().unwrap();
        backups.push(manifest.clone());

        while backups.len() > config.retention_count as usize {
            backups.remove(0);
        }

        Ok(BackupResult {
            manifest,
            duration_ms: start.elapsed().as_millis() as u64,
            success: true,
            error: None,
        })
    }

    fn restore_backup(
        &self,
        backup_id: &str,
        _config: &BackupConfig,
    ) -> Result<RestoreResult, String> {
        let start = std::time::Instant::now();
        let backups = self.backups.lock().unwrap();
        let manifest = backups
            .iter()
            .find(|b| b.id == backup_id)
            .ok_or_else(|| format!("backup {backup_id} not found"))?;

        let components_restored: Vec<String> =
            manifest.components.iter().map(|c| c.name.clone()).collect();

        Ok(RestoreResult {
            backup_id: backup_id.to_string(),
            components_restored,
            duration_ms: start.elapsed().as_millis() as u64,
            success: true,
            error: None,
        })
    }

    fn list_backups(&self) -> Vec<BackupManifest> {
        self.backups.lock().unwrap().clone()
    }

    fn delete_backup(&self, backup_id: &str) -> Result<(), String> {
        let mut backups = self.backups.lock().unwrap();
        let before = backups.len();
        backups.retain(|b| b.id != backup_id);
        if backups.len() < before {
            Ok(())
        } else {
            Err(format!("backup {backup_id} not found"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_config_default() {
        let config = BackupConfig::default();
        assert_eq!(config.retention_count, 10);
        assert!(config.compression_enabled);
        assert!(config.include_database);
        assert!(config.include_repositories);
        assert!(config.include_artifacts);
    }

    #[test]
    fn test_backup_type_equality() {
        assert_eq!(BackupType::Database, BackupType::Database);
        assert_ne!(BackupType::Database, BackupType::Logs);
    }

    #[test]
    fn test_checksum_algorithm_equality() {
        assert_eq!(ChecksumAlgorithm::Sha256, ChecksumAlgorithm::Sha256);
        assert_ne!(ChecksumAlgorithm::Sha256, ChecksumAlgorithm::Sha512);
    }

    #[test]
    fn test_in_memory_create_backup() {
        let mgr = InMemoryBackupManager::new();
        let config = BackupConfig::default();
        let result = mgr.create_backup(&config).unwrap();
        assert!(result.success);
        assert!(result.error.is_none());
        assert_eq!(result.manifest.version, "1.0");
        assert!(!result.manifest.id.is_empty());
    }

    #[test]
    fn test_in_memory_backup_components() {
        let mgr = InMemoryBackupManager::new();
        let config = BackupConfig::default();
        let result = mgr.create_backup(&config).unwrap();
        assert_eq!(result.manifest.components.len(), 3);
    }

    #[test]
    fn test_in_memory_backup_selective_components() {
        let mgr = InMemoryBackupManager::new();
        let config = BackupConfig {
            include_database: true,
            include_repositories: false,
            include_artifacts: false,
            ..BackupConfig::default()
        };
        let result = mgr.create_backup(&config).unwrap();
        assert_eq!(result.manifest.components.len(), 1);
        assert_eq!(
            result.manifest.components[0].component_type,
            BackupType::Database
        );
    }

    #[test]
    fn test_in_memory_list_backups() {
        let mgr = InMemoryBackupManager::new();
        assert!(mgr.list_backups().is_empty());
        mgr.create_backup(&BackupConfig::default()).unwrap();
        mgr.create_backup(&BackupConfig::default()).unwrap();
        assert_eq!(mgr.list_backups().len(), 2);
    }

    #[test]
    fn test_in_memory_restore_backup() {
        let mgr = InMemoryBackupManager::new();
        let config = BackupConfig::default();
        let result = mgr.create_backup(&config).unwrap();
        let backup_id = &result.manifest.id;
        let restore = mgr.restore_backup(backup_id, &config).unwrap();
        assert!(restore.success);
        assert_eq!(restore.backup_id, *backup_id);
        assert!(!restore.components_restored.is_empty());
    }

    #[test]
    fn test_in_memory_restore_nonexistent() {
        let mgr = InMemoryBackupManager::new();
        let config = BackupConfig::default();
        let result = mgr.restore_backup("nonexistent", &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_in_memory_delete_backup() {
        let mgr = InMemoryBackupManager::new();
        let config = BackupConfig::default();
        let result = mgr.create_backup(&config).unwrap();
        let backup_id = result.manifest.id.clone();
        assert!(mgr.delete_backup(&backup_id).is_ok());
        assert_eq!(mgr.list_backups().len(), 0);
    }

    #[test]
    fn test_in_memory_delete_nonexistent() {
        let mgr = InMemoryBackupManager::new();
        assert!(mgr.delete_backup("nope").is_err());
    }

    #[test]
    fn test_retention_limit() {
        let mgr = InMemoryBackupManager::new();
        let config = BackupConfig {
            retention_count: 2,
            ..BackupConfig::default()
        };
        mgr.create_backup(&config).unwrap();
        mgr.create_backup(&config).unwrap();
        mgr.create_backup(&config).unwrap();
        assert_eq!(mgr.list_backups().len(), 2);
    }

    #[test]
    fn test_backup_manifest_serialization() {
        let manifest = BackupManifest {
            id: "bk-1".to_string(),
            version: "1.0".to_string(),
            created_at: Utc::now(),
            components: vec![],
            total_size_bytes: 0,
            checksum: "abc".to_string(),
        };
        let json = serde_json::to_string(&manifest).unwrap();
        let back: BackupManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "bk-1");
    }

    #[test]
    fn test_restore_result_serialization() {
        let result = RestoreResult {
            backup_id: "bk-1".to_string(),
            components_restored: vec!["db".to_string()],
            duration_ms: 100,
            success: true,
            error: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: RestoreResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.backup_id, "bk-1");
        assert!(back.success);
    }

    #[test]
    fn test_backup_result_serialization() {
        let result = BackupResult {
            manifest: BackupManifest {
                id: "bk-2".to_string(),
                version: "1.0".to_string(),
                created_at: Utc::now(),
                components: vec![],
                total_size_bytes: 0,
                checksum: "c".to_string(),
            },
            duration_ms: 50,
            success: true,
            error: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: BackupResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.manifest.id, "bk-2");
    }

    #[test]
    fn test_in_memory_default() {
        let mgr = InMemoryBackupManager::default();
        assert_eq!(mgr.list_backups().len(), 0);
    }

    #[test]
    fn test_total_size_bytes() {
        let mgr = InMemoryBackupManager::new();
        let config = BackupConfig::default();
        let result = mgr.create_backup(&config).unwrap();
        assert_eq!(result.manifest.total_size_bytes, 1024 + 2048 + 512);
    }

    #[test]
    fn test_duration_ms_present() {
        let mgr = InMemoryBackupManager::new();
        let config = BackupConfig::default();
        let result = mgr.create_backup(&config).unwrap();
        assert!(result.duration_ms < 1000);
    }
}
