//! Workspace management for CI/CD runner.
//!
//! Handles git clone + checkout at specific SHA for pipeline execution.

#![forbid(unsafe_code)]

use std::path::PathBuf;

/// Manages the workspace directory for a pipeline job.
pub struct WorkspaceManager {
    workspaces_root: PathBuf,
}

impl WorkspaceManager {
    /// Create a new workspace manager.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            workspaces_root: root.into(),
        }
    }

    /// Ensure the workspaces root directory exists.
    pub fn ensure_root(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.workspaces_root)
    }

    /// Get the workspace path for a specific job.
    pub fn job_path(&self, job_id: &str) -> PathBuf {
        self.workspaces_root.join(job_id)
    }

    /// Prepare a workspace for a job: clone the repo and checkout at SHA.
    ///
    /// Returns the workspace directory path.
    pub async fn prepare(
        &self,
        job_id: &str,
        repo_url: &str,
        commit_sha: &str,
    ) -> anyhow::Result<PathBuf> {
        let workspace = self.job_path(job_id);

        // Clean up any previous workspace
        if workspace.exists() {
            tokio::fs::remove_dir_all(&workspace).await?;
        }
        tokio::fs::create_dir_all(&workspace).await?;

        // Clone the repository (shallow clone for speed)
        let status = tokio::process::Command::new("git")
            .arg("clone")
            .arg("--depth=1")
            .arg(repo_url)
            .arg(&workspace)
            .output()
            .await?;

        if !status.status.success() {
            return Err(anyhow::anyhow!(
                "git clone failed: {}",
                String::from_utf8_lossy(&status.stderr)
            ));
        }

        // Checkout the specific commit
        let status = tokio::process::Command::new("git")
            .arg("--git-dir")
            .arg(workspace.join(".git"))
            .arg("--work-tree")
            .arg(&workspace)
            .arg("checkout")
            .arg(commit_sha)
            .output()
            .await?;

        if !status.status.success() {
            return Err(anyhow::anyhow!(
                "git checkout failed: {}",
                String::from_utf8_lossy(&status.stderr)
            ));
        }

        tracing::info!(job_id, commit = commit_sha, "workspace prepared");

        Ok(workspace)
    }

    /// Clean up a workspace directory.
    pub async fn cleanup(&self, job_id: &str) -> std::io::Result<()> {
        let workspace = self.job_path(job_id);
        if workspace.exists() {
            tokio::fs::remove_dir_all(&workspace).await
        } else {
            Ok(())
        }
    }

    /// Clean up all workspaces.
    pub async fn cleanup_all(&self) -> std::io::Result<()> {
        if self.workspaces_root.exists() {
            tokio::fs::remove_dir_all(&self.workspaces_root).await?;
            self.ensure_root()?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_manager_job_path() {
        let mgr = WorkspaceManager::new("/tmp/civit-workspaces");
        let path = mgr.job_path("job-123");
        assert_eq!(path, PathBuf::from("/tmp/civit-workspaces/job-123"));
    }

    #[tokio::test]
    async fn test_workspace_manager_cleanup_nonexistent() {
        let dir = std::env::temp_dir().join("civit-test-workspaces");
        let mgr = WorkspaceManager::new(&dir);
        // Should not error on nonexistent dir
        assert!(mgr.cleanup("nonexistent").await.is_ok());
    }

    #[tokio::test]
    async fn test_workspace_manager_cleanup_all() {
        let dir = std::env::temp_dir().join("civit-test-cleanup-all");
        let mgr = WorkspaceManager::new(&dir);
        // Create then clean
        std::fs::create_dir_all(dir.join("subdir")).ok();
        assert!(mgr.cleanup_all().await.is_ok());
        assert!(dir.exists()); // Root recreated
    }
}
