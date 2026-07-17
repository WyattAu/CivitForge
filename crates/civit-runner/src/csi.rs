#![forbid(unsafe_code)]

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsiConfig {
    pub driver_name: String,
    pub endpoint: String,
    pub s3_endpoint: String,
    pub s3_region: String,
    pub s3_bucket: String,
    pub mount_base_path: String,
}

impl Default for CsiConfig {
    fn default() -> Self {
        Self {
            driver_name: "civitforge.csi.s3".into(),
            endpoint: "unix:///csi/csi.sock".into(),
            s3_endpoint: "https://s3.amazonaws.com".into(),
            s3_region: "us-east-1".into(),
            s3_bucket: "civitforge-volumes".into(),
            mount_base_path: "/var/lib/civitforge/mounts".into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessMode {
    ReadWriteOnce,
    ReadOnlyMany,
    ReadWriteMany,
}

impl std::fmt::Display for AccessMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessMode::ReadWriteOnce => write!(f, "ReadWriteOnce"),
            AccessMode::ReadOnlyMany => write!(f, "ReadOnlyMany"),
            AccessMode::ReadWriteMany => write!(f, "ReadWriteMany"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsiVolume {
    pub volume_id: String,
    pub bucket: String,
    pub prefix: String,
    pub capacity_bytes: i64,
    pub access_mode: AccessMode,
    pub mount_options: HashMap<String, String>,
}

impl CsiVolume {
    pub fn s3_path(&self) -> String {
        if self.prefix.is_empty() {
            format!("s3://{}/", self.bucket)
        } else {
            format!("s3://{}/{}", self.bucket, self.prefix)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsiMountRequest {
    pub volume_id: String,
    pub target_path: String,
    pub stage_secret_ref: Option<String>,
    pub publish_context: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsiMountResult {
    pub device_path: String,
    pub staging_target: String,
    pub publish_context: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum CsiError {
    VolumeNotFound(String),
    VolumeAlreadyExists(String),
    InvalidRequest(String),
    MountFailed(String),
    Internal(String),
}

impl std::fmt::Display for CsiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsiError::VolumeNotFound(id) => write!(f, "volume not found: {id}"),
            CsiError::VolumeAlreadyExists(id) => write!(f, "volume already exists: {id}"),
            CsiError::InvalidRequest(msg) => write!(f, "invalid request: {msg}"),
            CsiError::MountFailed(msg) => write!(f, "mount failed: {msg}"),
            CsiError::Internal(msg) => write!(f, "internal error: {msg}"),
        }
    }
}

impl std::error::Error for CsiError {}

#[derive(Debug, Clone)]
pub struct MountInfo {
    pub volume_id: String,
    pub target_path: String,
    pub staging_target: String,
    pub mount_options: HashMap<String, String>,
}

pub struct CsiDriver {
    pub config: CsiConfig,
    pub volumes: DashMap<String, CsiVolume>,
    pub mounts: DashMap<String, MountInfo>,
}

impl CsiDriver {
    pub fn new(config: CsiConfig) -> Self {
        Self {
            config,
            volumes: DashMap::new(),
            mounts: DashMap::new(),
        }
    }

    pub fn create_volume(
        &self,
        volume_id: &str,
        bucket: &str,
        prefix: &str,
        capacity_bytes: i64,
        access_mode: AccessMode,
        mount_options: HashMap<String, String>,
    ) -> Result<CsiVolume, CsiError> {
        if volume_id.is_empty() {
            return Err(CsiError::InvalidRequest("volume_id must not be empty".into()));
        }
        if bucket.is_empty() {
            return Err(CsiError::InvalidRequest("bucket must not be empty".into()));
        }
        if capacity_bytes <= 0 {
            return Err(CsiError::InvalidRequest("capacity_bytes must be positive".into()));
        }

        if self.volumes.contains_key(volume_id) {
            return Err(CsiError::VolumeAlreadyExists(volume_id.into()));
        }

        let volume = CsiVolume {
            volume_id: volume_id.into(),
            bucket: bucket.into(),
            prefix: prefix.into(),
            capacity_bytes,
            access_mode,
            mount_options,
        };

        self.volumes.insert(volume_id.into(), volume.clone());
        debug!(volume_id, bucket, "CSI volume created");
        Ok(volume)
    }

    pub fn delete_volume(&self, volume_id: &str) -> Result<(), CsiError> {
        if !self.volumes.contains_key(volume_id) {
            return Err(CsiError::VolumeNotFound(volume_id.into()));
        }

        let mut mounts_to_remove = Vec::new();
        for entry in self.mounts.iter_mut() {
            if entry.value().volume_id == volume_id {
                mounts_to_remove.push(entry.key().clone());
            }
        }

        for mount_key in mounts_to_remove {
            self.mounts.remove(&mount_key);
        }

        self.volumes.remove(volume_id);
        debug!(volume_id, "CSI volume deleted");
        Ok(())
    }

    pub fn stage_volume(&self, request: &CsiMountRequest) -> Result<CsiMountResult, CsiError> {
        if !self.volumes.contains_key(&request.volume_id) {
            return Err(CsiError::VolumeNotFound(request.volume_id.clone()));
        }

        let staging_target = format!("{}/staging/{}", self.config.mount_base_path, request.volume_id);
        let device_path = format!("{}/{}", staging_target, request.volume_id);

        let mut publish_context = request.publish_context.clone();
        publish_context.insert("staging_target".into(), staging_target.clone());
        publish_context.insert("device_path".into(), device_path.clone());

        debug!(volume_id = %request.volume_id, %staging_target, "CSI volume staged");
        Ok(CsiMountResult {
            device_path,
            staging_target,
            publish_context,
        })
    }

    pub fn publish_volume(&self, request: &CsiMountRequest) -> Result<CsiMountResult, CsiError> {
        if !self.volumes.contains_key(&request.volume_id) {
            return Err(CsiError::VolumeNotFound(request.volume_id.clone()));
        }

        if self.mounts.contains_key(&request.target_path) {
            return Err(CsiError::MountFailed(format!(
                "target path already mounted: {}",
                request.target_path
            )));
        }

        let staging_target = format!("{}/staging/{}", self.config.mount_base_path, request.volume_id);
        let device_path = format!(
            "{}/publish/{}",
            self.config.mount_base_path, request.volume_id
        );

        let volume = self
            .volumes
            .get(&request.volume_id)
            .map(|v| v.clone())
            .expect("operation should succeed");

        let mount_info = MountInfo {
            volume_id: request.volume_id.clone(),
            target_path: request.target_path.clone(),
            staging_target: staging_target.clone(),
            mount_options: volume.mount_options.clone(),
        };

        self.mounts.insert(request.target_path.clone(), mount_info);

        let mut publish_context = request.publish_context.clone();
        publish_context.insert("device_path".into(), device_path.clone());
        publish_context.insert("staging_target".into(), staging_target.clone());

        debug!(
            volume_id = %request.volume_id,
            target = %request.target_path,
            "CSI volume published"
        );
        Ok(CsiMountResult {
            device_path,
            staging_target,
            publish_context,
        })
    }

    pub fn unpublish_volume(&self, volume_id: &str, target_path: &str) -> Result<(), CsiError> {
        if !self.mounts.contains_key(target_path) {
            return Err(CsiError::MountFailed(format!(
                "no mount found at target path: {target_path}"
            )));
        }

        if let Some(entry) = self.mounts.get_mut(target_path) {
            if entry.value().volume_id != volume_id {
                return Err(CsiError::MountFailed(format!(
                    "mount at {target_path} belongs to different volume: {}",
                    entry.value().volume_id
                )));
            }
            drop(entry);
        }

        self.mounts.remove(target_path);
        debug!(volume_id, target_path, "CSI volume unpublished");
        Ok(())
    }

    pub fn list_volumes(&self) -> Vec<CsiVolume> {
        self.volumes.iter().map(|v| v.value().clone()).collect()
    }

    pub fn get_volume(&self, volume_id: &str) -> Option<CsiVolume> {
        self.volumes.get(volume_id).map(|v| v.clone())
    }

    pub fn mount_count(&self) -> usize {
        self.mounts.len()
    }

    pub fn volume_count(&self) -> usize {
        self.volumes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_driver() -> CsiDriver {
        CsiDriver::new(CsiConfig::default())
    }

    fn test_mount_options() -> HashMap<String, String> {
        let mut opts = HashMap::new();
        opts.insert("fsType".into(), "fuse".into());
        opts
    }

    #[test]
    fn test_csi_config_default() {
        let config = CsiConfig::default();
        assert_eq!(config.driver_name, "civitforge.csi.s3");
        assert_eq!(config.s3_region, "us-east-1");
        assert_eq!(config.s3_bucket, "civitforge-volumes");
        assert_eq!(config.mount_base_path, "/var/lib/civitforge/mounts");
    }

    #[test]
    fn test_csi_config_serialization() {
        let config = CsiConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let de: CsiConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(de.driver_name, config.driver_name);
        assert_eq!(de.s3_region, config.s3_region);
    }

    #[test]
    fn test_access_mode_display() {
        assert_eq!(AccessMode::ReadWriteOnce.to_string(), "ReadWriteOnce");
        assert_eq!(AccessMode::ReadOnlyMany.to_string(), "ReadOnlyMany");
        assert_eq!(AccessMode::ReadWriteMany.to_string(), "ReadWriteMany");
    }

    #[test]
    fn test_csi_volume_s3_path() {
        let vol = CsiVolume {
            volume_id: "v1".into(),
            bucket: "my-bucket".into(),
            prefix: "data/".into(),
            capacity_bytes: 1024,
            access_mode: AccessMode::ReadWriteOnce,
            mount_options: HashMap::new(),
        };
        assert_eq!(vol.s3_path(), "s3://my-bucket/data/");
    }

    #[test]
    fn test_csi_volume_s3_path_no_prefix() {
        let vol = CsiVolume {
            volume_id: "v2".into(),
            bucket: "my-bucket".into(),
            prefix: String::new(),
            capacity_bytes: 1024,
            access_mode: AccessMode::ReadWriteOnce,
            mount_options: HashMap::new(),
        };
        assert_eq!(vol.s3_path(), "s3://my-bucket/");
    }

    #[test]
    fn test_create_volume() {
        let driver = test_driver();
        let vol = driver
            .create_volume("vol-1", "my-bucket", "prefix", 1073741824, AccessMode::ReadWriteOnce, test_mount_options())
            .unwrap();
        assert_eq!(vol.volume_id, "vol-1");
        assert_eq!(vol.bucket, "my-bucket");
        assert_eq!(vol.capacity_bytes, 1073741824);
        assert_eq!(driver.volume_count(), 1);
    }

    #[test]
    fn test_create_volume_duplicate() {
        let driver = test_driver();
        driver
            .create_volume("vol-1", "b", "p", 100, AccessMode::ReadWriteOnce, HashMap::new())
            .unwrap();
        let result = driver.create_volume("vol-1", "b", "p", 100, AccessMode::ReadWriteOnce, HashMap::new());
        assert!(matches!(result, Err(CsiError::VolumeAlreadyExists(_))));
    }

    #[test]
    fn test_create_volume_empty_id() {
        let driver = test_driver();
        let result = driver.create_volume("", "b", "p", 100, AccessMode::ReadWriteOnce, HashMap::new());
        assert!(matches!(result, Err(CsiError::InvalidRequest(_))));
    }

    #[test]
    fn test_create_volume_empty_bucket() {
        let driver = test_driver();
        let result = driver.create_volume("v1", "", "p", 100, AccessMode::ReadWriteOnce, HashMap::new());
        assert!(matches!(result, Err(CsiError::InvalidRequest(_))));
    }

    #[test]
    fn test_create_volume_negative_capacity() {
        let driver = test_driver();
        let result = driver.create_volume("v1", "b", "p", -1, AccessMode::ReadWriteOnce, HashMap::new());
        assert!(matches!(result, Err(CsiError::InvalidRequest(_))));
    }

    #[test]
    fn test_create_volume_zero_capacity() {
        let driver = test_driver();
        let result = driver.create_volume("v1", "b", "p", 0, AccessMode::ReadWriteOnce, HashMap::new());
        assert!(matches!(result, Err(CsiError::InvalidRequest(_))));
    }

    #[test]
    fn test_delete_volume() {
        let driver = test_driver();
        driver
            .create_volume("vol-1", "b", "p", 100, AccessMode::ReadWriteOnce, HashMap::new())
            .unwrap();
        driver.delete_volume("vol-1").unwrap();
        assert_eq!(driver.volume_count(), 0);
    }

    #[test]
    fn test_delete_volume_not_found() {
        let driver = test_driver();
        let result = driver.delete_volume("nonexistent");
        assert!(matches!(result, Err(CsiError::VolumeNotFound(_))));
    }

    #[test]
    fn test_delete_volume_cleans_up_mounts() {
        let driver = test_driver();
        driver
            .create_volume("vol-1", "b", "p", 100, AccessMode::ReadWriteOnce, HashMap::new())
            .unwrap();
        let req = CsiMountRequest {
            volume_id: "vol-1".into(),
            target_path: "/mnt/data".into(),
            stage_secret_ref: None,
            publish_context: HashMap::new(),
        };
        driver.publish_volume(&req).unwrap();
        assert_eq!(driver.mount_count(), 1);

        driver.delete_volume("vol-1").unwrap();
        assert_eq!(driver.mount_count(), 0);
        assert_eq!(driver.volume_count(), 0);
    }

    #[test]
    fn test_stage_volume() {
        let driver = test_driver();
        driver
            .create_volume("vol-1", "b", "p", 100, AccessMode::ReadWriteOnce, HashMap::new())
            .unwrap();
        let req = CsiMountRequest {
            volume_id: "vol-1".into(),
            target_path: "/mnt/data".into(),
            stage_secret_ref: None,
            publish_context: HashMap::new(),
        };
        let result = driver.stage_volume(&req).unwrap();
        assert!(result.device_path.contains("vol-1"));
        assert!(result.staging_target.contains("staging"));
        assert!(result.publish_context.contains_key("staging_target"));
    }

    #[test]
    fn test_stage_volume_not_found() {
        let driver = test_driver();
        let req = CsiMountRequest {
            volume_id: "nonexistent".into(),
            target_path: "/mnt/data".into(),
            stage_secret_ref: None,
            publish_context: HashMap::new(),
        };
        let result = driver.stage_volume(&req);
        assert!(matches!(result, Err(CsiError::VolumeNotFound(_))));
    }

    #[test]
    fn test_publish_volume() {
        let driver = test_driver();
        driver
            .create_volume("vol-1", "b", "p", 100, AccessMode::ReadWriteOnce, test_mount_options())
            .unwrap();
        let req = CsiMountRequest {
            volume_id: "vol-1".into(),
            target_path: "/mnt/data".into(),
            stage_secret_ref: None,
            publish_context: HashMap::new(),
        };
        let result = driver.publish_volume(&req).unwrap();
        assert_eq!(result.publish_context["device_path"], result.device_path);
        assert_eq!(driver.mount_count(), 1);
    }

    #[test]
    fn test_publish_volume_already_mounted() {
        let driver = test_driver();
        driver
            .create_volume("vol-1", "b", "p", 100, AccessMode::ReadWriteOnce, HashMap::new())
            .unwrap();
        let req = CsiMountRequest {
            volume_id: "vol-1".into(),
            target_path: "/mnt/data".into(),
            stage_secret_ref: None,
            publish_context: HashMap::new(),
        };
        driver.publish_volume(&req).unwrap();
        let result = driver.publish_volume(&req);
        assert!(matches!(result, Err(CsiError::MountFailed(_))));
    }

    #[test]
    fn test_publish_volume_not_found() {
        let driver = test_driver();
        let req = CsiMountRequest {
            volume_id: "nonexistent".into(),
            target_path: "/mnt/data".into(),
            stage_secret_ref: None,
            publish_context: HashMap::new(),
        };
        let result = driver.publish_volume(&req);
        assert!(matches!(result, Err(CsiError::VolumeNotFound(_))));
    }

    #[test]
    fn test_unpublish_volume() {
        let driver = test_driver();
        driver
            .create_volume("vol-1", "b", "p", 100, AccessMode::ReadWriteOnce, HashMap::new())
            .unwrap();
        let req = CsiMountRequest {
            volume_id: "vol-1".into(),
            target_path: "/mnt/data".into(),
            stage_secret_ref: None,
            publish_context: HashMap::new(),
        };
        driver.publish_volume(&req).unwrap();
        assert_eq!(driver.mount_count(), 1);

        driver.unpublish_volume("vol-1", "/mnt/data").unwrap();
        assert_eq!(driver.mount_count(), 0);
    }

    #[test]
    fn test_unpublish_volume_not_mounted() {
        let driver = test_driver();
        let result = driver.unpublish_volume("vol-1", "/mnt/nonexistent");
        assert!(matches!(result, Err(CsiError::MountFailed(_))));
    }

    #[test]
    fn test_unpublish_volume_wrong_volume() {
        let driver = test_driver();
        driver
            .create_volume("vol-1", "b", "p", 100, AccessMode::ReadWriteOnce, HashMap::new())
            .unwrap();
        let req = CsiMountRequest {
            volume_id: "vol-1".into(),
            target_path: "/mnt/data".into(),
            stage_secret_ref: None,
            publish_context: HashMap::new(),
        };
        driver.publish_volume(&req).unwrap();

        let result = driver.unpublish_volume("vol-2", "/mnt/data");
        assert!(matches!(result, Err(CsiError::MountFailed(_))));
    }

    #[test]
    fn test_list_volumes() {
        let driver = test_driver();
        assert!(driver.list_volumes().is_empty());

        driver
            .create_volume("vol-1", "b1", "p1", 100, AccessMode::ReadWriteOnce, HashMap::new())
            .unwrap();
        driver
            .create_volume("vol-2", "b2", "p2", 200, AccessMode::ReadOnlyMany, HashMap::new())
            .unwrap();

        let volumes = driver.list_volumes();
        assert_eq!(volumes.len(), 2);
    }

    #[test]
    fn test_get_volume() {
        let driver = test_driver();
        driver
            .create_volume("vol-1", "b", "p", 100, AccessMode::ReadWriteMany, HashMap::new())
            .unwrap();
        let vol = driver.get_volume("vol-1").unwrap();
        assert_eq!(vol.access_mode, AccessMode::ReadWriteMany);
        assert!(driver.get_volume("nonexistent").is_none());
    }

    #[test]
    fn test_csi_error_display() {
        let err = CsiError::VolumeNotFound("v1".into());
        assert_eq!(format!("{err}"), "volume not found: v1");

        let err = CsiError::InvalidRequest("bad".into());
        assert_eq!(format!("{err}"), "invalid request: bad");
    }

    #[test]
    fn test_access_mode_serialization() {
        let modes = vec![
            AccessMode::ReadWriteOnce,
            AccessMode::ReadOnlyMany,
            AccessMode::ReadWriteMany,
        ];
        for mode in modes {
            let json = serde_json::to_string(&mode).unwrap();
            let de: AccessMode = serde_json::from_str(&json).unwrap();
            assert_eq!(mode, de);
        }
    }
}
