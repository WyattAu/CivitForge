#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use parking_lot::Mutex;

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
    backups: Mutex<Vec<BackupManifest>>,
}

impl InMemoryBackupManager {
    pub fn new() -> Self {
        Self {
            backups: Mutex::new(Vec::new()),
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

        let mut backups = self.backups.lock();
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
        let backups = self.backups.lock();
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
        self.backups.lock().clone()
    }

    fn delete_backup(&self, backup_id: &str) -> Result<(), String> {
        let mut backups = self.backups.lock();
        let before = backups.len();
        backups.retain(|b| b.id != backup_id);
        if backups.len() < before {
            Ok(())
        } else {
            Err(format!("backup {backup_id} not found"))
        }
    }
}

// --- Database Backup / Recovery ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseBackupType {
    Full,
    Incremental,
    Differential,
}

impl DatabaseBackupType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseBackupType::Full => "full",
            DatabaseBackupType::Incremental => "incremental",
            DatabaseBackupType::Differential => "differential",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseBackupRequest {
    pub backup_type: DatabaseBackupType,
    pub recovery_point_name: Option<String>,
    pub recovery_point_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseBackupResult {
    pub backup_id: String,
    pub file_path: String,
    pub file_size_bytes: u64,
    pub checksum: String,
    pub duration_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPoint {
    pub id: String,
    pub backup_id: String,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreFromBackupRequest {
    pub backup_id: String,
    pub target_recovery_point: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreFromBackupResult {
    pub backup_id: String,
    pub tables_restored: Vec<String>,
    pub duration_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSchedule {
    pub backup_type: DatabaseBackupType,
    pub cron_expression: String,
    pub retention_count: u32,
    pub enabled: bool,
}

pub trait DatabaseBackupManager: Send + Sync {
    fn create_backup(
        &self,
        request: &DatabaseBackupRequest,
    ) -> Result<DatabaseBackupResult, String>;
    fn restore_from_backup(
        &self,
        request: &RestoreFromBackupRequest,
    ) -> Result<RestoreFromBackupResult, String>;
    fn create_recovery_point(
        &self,
        backup_id: &str,
        name: &str,
        description: &str,
    ) -> Result<RecoveryPoint, String>;
    fn list_recovery_points(&self, backup_id: &str) -> Vec<RecoveryPoint>;
    fn schedule_backup(&self, schedule: &BackupSchedule) -> Result<(), String>;
}

pub struct InMemoryDatabaseBackupManager {
    #[allow(clippy::type_complexity)]
    backups: Mutex<Vec<(String, DatabaseBackupType, u64, String, DateTime<Utc>)>>,
    recovery_points: Mutex<Vec<RecoveryPoint>>,
}

impl InMemoryDatabaseBackupManager {
    pub fn new() -> Self {
        Self {
            backups: Mutex::new(Vec::new()),
            recovery_points: Mutex::new(Vec::new()),
        }
    }
}

impl Default for InMemoryDatabaseBackupManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseBackupManager for InMemoryDatabaseBackupManager {
    fn create_backup(
        &self,
        request: &DatabaseBackupRequest,
    ) -> Result<DatabaseBackupResult, String> {
        let start = std::time::Instant::now();
        let id = uuid::Uuid::new_v4().to_string();
        let file_path = format!("/var/lib/civitforge/backups/db_{}.dump", id);

        let mut backups = self.backups.lock();
        backups.push((
            id.clone(),
            request.backup_type.clone(),
            1024,
            file_path.clone(),
            Utc::now(),
        ));

        Ok(DatabaseBackupResult {
            backup_id: id,
            file_path,
            file_size_bytes: 1024,
            checksum: "sha256_placeholder".to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
            success: true,
            error: None,
        })
    }

    fn restore_from_backup(
        &self,
        request: &RestoreFromBackupRequest,
    ) -> Result<RestoreFromBackupResult, String> {
        let start = std::time::Instant::now();
        let backups = self.backups.lock();
        let _ = backups
            .iter()
            .find(|(id, _, _, _, _)| id == &request.backup_id)
            .ok_or_else(|| format!("backup {} not found", request.backup_id))?;

        Ok(RestoreFromBackupResult {
            backup_id: request.backup_id.clone(),
            tables_restored: vec!["users".into(), "repositories".into(), "issues".into()],
            duration_ms: start.elapsed().as_millis() as u64,
            success: true,
            error: None,
        })
    }

    fn create_recovery_point(
        &self,
        backup_id: &str,
        name: &str,
        description: &str,
    ) -> Result<RecoveryPoint, String> {
        let point = RecoveryPoint {
            id: uuid::Uuid::new_v4().to_string(),
            backup_id: backup_id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            created_at: Utc::now(),
        };

        let mut points = self.recovery_points.lock();
        points.push(point.clone());

        Ok(point)
    }

    fn list_recovery_points(&self, backup_id: &str) -> Vec<RecoveryPoint> {
        let points = self.recovery_points.lock();
        points
            .iter()
            .filter(|p| p.backup_id == backup_id)
            .cloned()
            .collect()
    }

    fn schedule_backup(&self, _schedule: &BackupSchedule) -> Result<(), String> {
        Ok(())
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

    // --- Database Backup Tests ---

    #[test]
    fn test_database_backup_type_as_str() {
        assert_eq!(DatabaseBackupType::Full.as_str(), "full");
        assert_eq!(DatabaseBackupType::Incremental.as_str(), "incremental");
        assert_eq!(DatabaseBackupType::Differential.as_str(), "differential");
    }

    #[test]
    fn test_in_memory_db_create_backup() {
        let mgr = InMemoryDatabaseBackupManager::new();
        let request = DatabaseBackupRequest {
            backup_type: DatabaseBackupType::Full,
            recovery_point_name: None,
            recovery_point_description: None,
        };
        let result = mgr.create_backup(&request).unwrap();
        assert!(result.success);
        assert!(!result.backup_id.is_empty());
    }

    #[test]
    fn test_in_memory_db_restore_backup() {
        let mgr = InMemoryDatabaseBackupManager::new();
        let request = DatabaseBackupRequest {
            backup_type: DatabaseBackupType::Full,
            recovery_point_name: None,
            recovery_point_description: None,
        };
        let backup = mgr.create_backup(&request).unwrap();
        let restore = RestoreFromBackupRequest {
            backup_id: backup.backup_id,
            target_recovery_point: None,
        };
        let result = mgr.restore_from_backup(&restore).unwrap();
        assert!(result.success);
        assert!(!result.tables_restored.is_empty());
    }

    #[test]
    fn test_in_memory_db_restore_nonexistent() {
        let mgr = InMemoryDatabaseBackupManager::new();
        let restore = RestoreFromBackupRequest {
            backup_id: "nonexistent".to_string(),
            target_recovery_point: None,
        };
        assert!(mgr.restore_from_backup(&restore).is_err());
    }

    #[test]
    fn test_in_memory_recovery_point() {
        let mgr = InMemoryDatabaseBackupManager::new();
        let request = DatabaseBackupRequest {
            backup_type: DatabaseBackupType::Full,
            recovery_point_name: None,
            recovery_point_description: None,
        };
        let backup = mgr.create_backup(&request).unwrap();
        let point = mgr
            .create_recovery_point(&backup.backup_id, "pre-deploy", "Before v1.0")
            .unwrap();
        assert_eq!(point.name, "pre-deploy");
        assert_eq!(point.backup_id, backup.backup_id);
    }

    #[test]
    fn test_in_memory_list_recovery_points() {
        let mgr = InMemoryDatabaseBackupManager::new();
        let request = DatabaseBackupRequest {
            backup_type: DatabaseBackupType::Full,
            recovery_point_name: None,
            recovery_point_description: None,
        };
        let backup = mgr.create_backup(&request).unwrap();
        mgr.create_recovery_point(&backup.backup_id, "rp1", "desc1")
            .unwrap();
        mgr.create_recovery_point(&backup.backup_id, "rp2", "desc2")
            .unwrap();
        let points = mgr.list_recovery_points(&backup.backup_id);
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn test_in_memory_schedule_backup() {
        let mgr = InMemoryDatabaseBackupManager::new();
        let schedule = BackupSchedule {
            backup_type: DatabaseBackupType::Full,
            cron_expression: "0 2 * * *".to_string(),
            retention_count: 7,
            enabled: true,
        };
        assert!(mgr.schedule_backup(&schedule).is_ok());
    }

    #[test]
    fn test_db_backup_manager_default() {
        let mgr = InMemoryDatabaseBackupManager::default();
        let points = mgr.list_recovery_points("nonexistent");
        assert!(points.is_empty());
    }
}
