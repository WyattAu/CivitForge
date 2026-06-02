#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MountStatus {
    Pending,
    Mounted,
    Unmounted,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountPoint {
    pub id: String,
    pub repo_id: String,
    pub commit_sha: String,
    pub mount_path: String,
    pub status: MountStatus,
    pub mounted_at: Option<String>,
    pub error: Option<String>,
}

pub struct MountManager {
    mounts: HashMap<String, MountPoint>,
    base_path: String,
}

impl MountManager {
    pub fn new(base_path: String) -> Self {
        Self {
            mounts: HashMap::new(),
            base_path,
        }
    }

    pub fn mount_path(&self, repo_id: &str) -> String {
        format!("{}/{}", self.base_path, repo_id)
    }

    pub fn create_mount(
        &mut self,
        repo_id: &str,
        commit_sha: &str,
        mount_path: Option<&str>,
    ) -> anyhow::Result<MountPoint> {
        let id = uuid::Uuid::new_v4().to_string();
        let path = mount_path
            .map(String::from)
            .unwrap_or_else(|| self.mount_path(repo_id));

        if self
            .mounts
            .values()
            .any(|m| m.mount_path == path && m.status == MountStatus::Mounted)
        {
            anyhow::bail!("mount path already in use: {path}");
        }

        let mount = MountPoint {
            id: id.clone(),
            repo_id: repo_id.into(),
            commit_sha: commit_sha.into(),
            mount_path: path,
            status: MountStatus::Pending,
            mounted_at: None,
            error: None,
        };

        info!(id = %id, repo = %repo_id, path = %mount.mount_path, "created mount point");
        self.mounts.insert(id.clone(), mount.clone());
        Ok(mount)
    }

    pub fn mark_mounted(&mut self, id: &str) -> anyhow::Result<()> {
        let mount = self
            .mounts
            .get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("mount not found: {id}"))?;
        mount.status = MountStatus::Mounted;
        mount.mounted_at = Some(chrono::Utc::now().to_rfc3339());
        debug!(id = %id, "marked as mounted");
        Ok(())
    }

    pub fn mark_failed(&mut self, id: &str, error: &str) -> anyhow::Result<()> {
        let mount = self
            .mounts
            .get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("mount not found: {id}"))?;
        mount.status = MountStatus::Failed;
        mount.error = Some(error.into());
        debug!(id = %id, error = %error, "marked as failed");
        Ok(())
    }

    pub fn unmount(&mut self, id: &str) -> anyhow::Result<MountPoint> {
        let mut mount = self
            .mounts
            .remove(id)
            .ok_or_else(|| anyhow::anyhow!("mount not found: {id}"))?;
        mount.status = MountStatus::Unmounted;
        info!(id = %id, "unmounted");
        Ok(mount)
    }

    pub fn get(&self, id: &str) -> Option<&MountPoint> {
        self.mounts.get(id)
    }

    pub fn list_by_repo(&self, repo_id: &str) -> Vec<&MountPoint> {
        self.mounts
            .values()
            .filter(|m| m.repo_id == repo_id)
            .collect()
    }

    pub fn list_mounted(&self) -> Vec<&MountPoint> {
        self.mounts
            .values()
            .filter(|m| m.status == MountStatus::Mounted)
            .collect()
    }

    pub fn count(&self) -> usize {
        self.mounts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_manager() -> MountManager {
        MountManager::new("/mnt/civit".into())
    }

    #[test]
    fn test_create_mount() {
        let mut mgr = make_manager();
        let mount = mgr.create_mount("repo-1", "sha123", None).unwrap();
        assert_eq!(mount.status, MountStatus::Pending);
        assert_eq!(mount.mount_path, "/mnt/civit/repo-1");
    }

    #[test]
    fn test_mark_mounted() {
        let mut mgr = make_manager();
        let mount = mgr.create_mount("repo-1", "sha123", None).unwrap();
        mgr.mark_mounted(&mount.id).unwrap();
        let mount = mgr.get(&mount.id).unwrap();
        assert_eq!(mount.status, MountStatus::Mounted);
        assert!(mount.mounted_at.is_some());
    }

    #[test]
    fn test_mark_failed() {
        let mut mgr = make_manager();
        let mount = mgr.create_mount("repo-1", "sha123", None).unwrap();
        mgr.mark_failed(&mount.id, "permission denied").unwrap();
        let mount = mgr.get(&mount.id).unwrap();
        assert_eq!(mount.status, MountStatus::Failed);
        assert_eq!(mount.error.as_deref(), Some("permission denied"));
    }

    #[test]
    fn test_unmount() {
        let mut mgr = make_manager();
        let mount = mgr.create_mount("repo-1", "sha123", None).unwrap();
        mgr.mark_mounted(&mount.id).unwrap();
        let removed = mgr.unmount(&mount.id).unwrap();
        assert_eq!(removed.status, MountStatus::Unmounted);
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn test_duplicate_mount_path() {
        let mut mgr = make_manager();
        let m1 = mgr
            .create_mount("repo-1", "sha1", Some("/mnt/custom"))
            .unwrap();
        mgr.mark_mounted(&m1.id).unwrap();
        assert!(
            mgr.create_mount("repo-2", "sha2", Some("/mnt/custom"))
                .is_err()
        );
    }

    #[test]
    fn test_list_by_repo() {
        let mut mgr = make_manager();
        mgr.create_mount("repo-1", "sha1", None).unwrap();
        mgr.create_mount("repo-1", "sha2", None).unwrap();
        mgr.create_mount("repo-2", "sha3", None).unwrap();
        assert_eq!(mgr.list_by_repo("repo-1").len(), 2);
        assert_eq!(mgr.list_by_repo("repo-2").len(), 1);
    }

    #[test]
    fn test_list_mounted() {
        let mut mgr = make_manager();
        let m1 = mgr.create_mount("repo-1", "sha1", None).unwrap();
        let _m2 = mgr.create_mount("repo-2", "sha2", None).unwrap();
        mgr.mark_mounted(&m1.id).unwrap();
        assert_eq!(mgr.list_mounted().len(), 1);
    }
}
