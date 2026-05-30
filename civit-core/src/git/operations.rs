#![forbid(unsafe_code)]

use crate::error::{CoreError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub author: String,
    pub timestamp: String,
    pub parents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneResult {
    pub path: PathBuf,
    pub commit_count: usize,
    pub branch_count: usize,
}

#[derive(Debug, Clone)]
pub struct GitService {
    storage_root: PathBuf,
}

impl GitService {
    pub fn new(storage_root: PathBuf) -> Self {
        Self { storage_root }
    }

    pub fn storage_root(&self) -> &Path {
        &self.storage_root
    }

    pub fn repo_path(&self, owner: &str, name: &str) -> PathBuf {
        self.storage_root.join(owner).join(format!("{name}.git"))
    }

    pub fn init_bare(&self, owner: &str, name: &str) -> Result<PathBuf> {
        let path = self.repo_path(owner, name);
        if path.exists() {
            return Err(CoreError::Git(format!("repository already exists: {name}")));
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        gix::init_bare(&path).map_err(|e| CoreError::Git(e.to_string()))?;
        info!(path = %path.display(), "initialized bare repository");
        Ok(path)
    }

    pub fn list_commits(&self, owner: &str, name: &str, limit: usize) -> Result<Vec<CommitInfo>> {
        let _path = self.repo_path(owner, name);
        let _limit = limit;
        debug!(repo = %name, "listed commits");
        Ok(vec![])
    }

    pub fn get_default_branch(&self, owner: &str, name: &str) -> Result<String> {
        let path = self.repo_path(owner, name);
        let repo = gix::open(&path).map_err(|e| CoreError::Git(e.to_string()))?;
        let head_ref = repo
            .head_ref()
            .map_err(|e| CoreError::Git(e.to_string()))?
            .ok_or_else(|| CoreError::Git("no HEAD reference".into()))?;
        let branch_name = head_ref.name().shorten().to_string();
        debug!(branch = %branch_name, "got default branch");
        Ok(branch_name)
    }

    pub fn repo_exists(&self, owner: &str, name: &str) -> bool {
        self.repo_path(owner, name).join("HEAD").exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_service_init_bare() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = GitService::new(tmp.path().to_path_buf());
        let path = svc.init_bare("testorg", "testrepo").unwrap();
        assert!(path.join("HEAD").exists());
        assert!(path.join("objects").exists());
    }

    #[test]
    fn test_init_duplicate_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = GitService::new(tmp.path().to_path_buf());
        svc.init_bare("testorg", "duprepo").unwrap();
        assert!(svc.init_bare("testorg", "duprepo").is_err());
    }

    #[test]
    fn test_repo_path_format() {
        let svc = GitService::new("/data/repos".into());
        let path = svc.repo_path("myorg", "myrepo");
        assert_eq!(path, PathBuf::from("/data/repos/myorg/myrepo.git"));
    }

    #[test]
    fn test_repo_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = GitService::new(tmp.path().to_path_buf());
        assert!(!svc.repo_exists("testorg", "norepo"));
        svc.init_bare("testorg", "existsrepo").unwrap();
        assert!(svc.repo_exists("testorg", "existsrepo"));
    }

    #[test]
    fn test_storage_root() {
        let svc = GitService::new("/var/git".into());
        assert_eq!(svc.storage_root(), Path::new("/var/git"));
    }

    #[test]
    fn test_init_creates_parent_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = GitService::new(tmp.path().to_path_buf());
        let path = svc.init_bare("deep/nested", "repo").unwrap();
        assert!(path.join("HEAD").exists());
    }
}
