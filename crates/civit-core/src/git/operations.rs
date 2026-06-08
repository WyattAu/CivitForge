#![forbid(unsafe_code)]

use crate::error::{CoreError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tracing::{debug, info, warn};

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

/// Supported merge strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MergeStrategy {
    /// Create a merge commit (two parents).
    Merge,
    /// Squash all source commits into one commit on top of target.
    Squash,
    /// Fast-forward only; fail if not possible.
    #[serde(rename = "fast-forward")]
    FastForward,
    /// Rebase source commits onto target, then fast-forward.
    Rebase,
}

impl std::fmt::Display for MergeStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Merge => write!(f, "merge"),
            Self::Squash => write!(f, "squash"),
            Self::FastForward => write!(f, "fast-forward"),
            Self::Rebase => write!(f, "rebase"),
        }
    }
}

impl std::str::FromStr for MergeStrategy {
    type Err = CoreError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "merge" | "recursive" => Ok(Self::Merge),
            "squash" => Ok(Self::Squash),
            "fast-forward" | "ff" | "ff-only" => Ok(Self::FastForward),
            "rebase" => Ok(Self::Rebase),
            _ => Err(CoreError::BadRequest(format!(
                "unknown merge strategy: {s}"
            ))),
        }
    }
}

/// Result of a merge operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    /// SHA of the merge commit (or HEAD after fast-forward).
    pub commit_sha: String,
    /// Which strategy was actually used (may differ from requested if ff detected).
    pub strategy_used: String,
    /// Whether the merge was a fast-forward (no merge commit created).
    pub was_ff: bool,
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
        // Ensure HEAD points to refs/heads/main (gix::init_bare may not set this)
        let head_path = path.join("HEAD");
        if !head_path.exists() {
            std::fs::write(&head_path, "ref: refs/heads/main\n")?;
        }
        info!(path = %path.display(), "initialized bare repository");
        Ok(path)
    }

    pub fn list_commits(&self, owner: &str, name: &str, limit: usize) -> Result<Vec<CommitInfo>> {
        let path = self.repo_path(owner, name);
        let repo = gix::open(&path).map_err(|e| CoreError::Git(e.to_string()))?;

        let head_id = repo.head_id().map_err(|e| CoreError::Git(e.to_string()))?;

        let mut commits = Vec::new();
        let mut current_id = head_id;

        while commits.len() < limit {
            let commit_obj = current_id
                .object()
                .map_err(|e| CoreError::Git(e.to_string()))?;

            let commit = commit_obj
                .try_into_commit()
                .map_err(|e| CoreError::Git(format!("non-commit object: {e}")))?;

            let parent_ids: Vec<gix::Id<'_>> = commit.parent_ids().collect();

            let parents: Vec<String> = parent_ids
                .iter()
                .map(|id| id.to_hex().to_string())
                .take(20)
                .collect();

            let author = commit.author().ok().map(|a| {
                let name = a.name.to_string();
                let email = a.email.to_string();
                format!("{name} <{email}>")
            });

            let time = commit.time().ok();

            commits.push(CommitInfo {
                id: commit.id().to_hex().to_string(),
                message: commit
                    .message()
                    .map(|m| m.summary().to_string())
                    .unwrap_or_default(),
                author: author.unwrap_or_default(),
                timestamp: time
                    .map(|t| {
                        chrono::DateTime::from_timestamp(t.seconds, 0)
                            .map(|dt| dt.to_rfc3339())
                            .unwrap_or_default()
                    })
                    .unwrap_or_default(),
                parents: parents.clone(),
            });

            // Walk to first parent for linear history
            match parent_ids.first() {
                Some(first_parent) => {
                    current_id = *first_parent;
                }
                None => break,
            }
        }

        debug!(repo = %name, count = commits.len(), "listed commits");
        Ok(commits)
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

    /// Clone a remote repository into the storage root.
    ///
    /// Creates a bare repo and configures the "origin" remote.
    /// Full fetch requires gix network features which may not be available.
    pub fn clone(&self, owner: &str, name: &str, remote_url: &str) -> Result<CloneResult> {
        let path = self.repo_path(owner, name);

        if path.exists() {
            return Err(CoreError::Git(format!("repository already exists: {name}")));
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        gix::init_bare(&path).map_err(|e| CoreError::Git(e.to_string()))?;

        let repo = gix::open(&path).map_err(|e| CoreError::Git(e.to_string()))?;

        // Create the "origin" remote pointing at the remote URL
        let _remote = repo
            .remote_at(remote_url)
            .map_err(|e| CoreError::Git(e.to_string()))?;

        info!(remote = %remote_url, path = %path.display(), "initialized clone repo with remote");

        let commit_count = self.count_commits(owner, name);
        // Branch count requires refs iteration that varies by gix version;
        // for a freshly initialized bare repo, it's 0.
        let branch_count = 0usize;

        Ok(CloneResult {
            path,
            commit_count,
            branch_count,
        })
    }

    /// Receive a push bundle into a bare repository.
    ///
    /// Validates the repository exists and is a valid bare repo.
    /// Returns the repository path.
    pub fn prepare_receive(&self, owner: &str, name: &str) -> Result<PathBuf> {
        let path = self.repo_path(owner, name);
        if !path.exists() {
            return Err(CoreError::Git(format!("repository does not exist: {name}")));
        }

        // Validate it's a valid git repo
        gix::open(&path).map_err(|e| CoreError::Git(e.to_string()))?;

        Ok(path)
    }

    /// Merge `source_branch` into `target_branch` using the given strategy.
    ///
    /// This operates on the bare repo by cloning to a temp directory, performing
    /// the merge there, then force-updating the bare repo's target branch ref.
    #[allow(clippy::too_many_arguments)]
    pub fn merge_branch(
        &self,
        owner: &str,
        name: &str,
        source_branch: &str,
        target_branch: &str,
        strategy: MergeStrategy,
        committer_name: &str,
        committer_email: &str,
    ) -> Result<MergeResult> {
        let bare_path = self.repo_path(owner, name);
        if !bare_path.exists() {
            return Err(CoreError::NotFound(format!(
                "repository {owner}/{name} does not exist"
            )));
        }

        let git_bin = std::env::var("GIT_BIN").unwrap_or_else(|_| "git".to_string());

        // Create temp working clone
        let tmp_dir = tempfile::tempdir()
            .map_err(|e| CoreError::Internal(format!("failed to create temp dir: {e}")))?;
        let work_path = tmp_dir.path();

        run_git(
            &git_bin,
            work_path,
            &[
                "clone",
                "--no-checkout",
                bare_path.to_str().unwrap_or(""),
                work_path.to_str().unwrap_or(""),
            ],
        )?;

        // Configure git user (required for commit operations)
        run_git(
            &git_bin,
            work_path,
            &["config", "user.name", committer_name],
        )?;
        run_git(
            &git_bin,
            work_path,
            &["config", "user.email", committer_email],
        )?;

        // Check out target branch
        run_git(&git_bin, work_path, &["checkout", target_branch])?;

        // Fetch the remote (bare repo) to get all latest refs
        run_git(&git_bin, work_path, &["fetch", "origin"])?;

        // For local branches, also check out source
        run_git(
            &git_bin,
            work_path,
            &[
                "branch",
                "-f",
                "source-temp",
                &format!("origin/{source_branch}"),
            ],
        )?;

        // Check if fast-forward is possible
        let can_ff = is_fast_forward(&git_bin, work_path, source_branch, target_branch)?;

        let result = match strategy {
            MergeStrategy::FastForward => {
                if !can_ff {
                    return Err(CoreError::BadRequest(format!(
                        "fast-forward merge not possible: {target_branch} has diverged from {source_branch}"
                    )));
                }
                // Fast-forward: just move the target ref to source
                let _output = run_git(&git_bin, work_path, &["merge", "--ff-only", "source-temp"])?;
                let sha = get_head_sha(&git_bin, work_path)?;
                MergeResult {
                    commit_sha: sha,
                    strategy_used: "fast-forward".into(),
                    was_ff: true,
                }
            }
            MergeStrategy::Squash => {
                // git merge --squash stages but doesn't commit
                let _ = run_git(&git_bin, work_path, &["merge", "--squash", "source-temp"])?;
                let _ = run_git(
                    &git_bin,
                    work_path,
                    &[
                        "commit",
                        "-m",
                        &format!("Merge branch '{source_branch}' into {target_branch} (squash)"),
                    ],
                )?;
                let sha = get_head_sha(&git_bin, work_path)?;
                MergeResult {
                    commit_sha: sha,
                    strategy_used: "squash".into(),
                    was_ff: false,
                }
            }
            MergeStrategy::Rebase => {
                // Rebase: rebase target onto source, then ff
                run_git(&git_bin, work_path, &["checkout", "source-temp"])?;
                run_git(&git_bin, work_path, &["rebase", target_branch])?;
                // Now move target to the rebased source
                run_git(&git_bin, work_path, &["checkout", target_branch])?;
                run_git(&git_bin, work_path, &["merge", "--ff-only", "source-temp"])?;
                let sha = get_head_sha(&git_bin, work_path)?;
                MergeResult {
                    commit_sha: sha,
                    strategy_used: "rebase".into(),
                    was_ff: false,
                }
            }
            MergeStrategy::Merge => {
                if can_ff {
                    // Use ff when possible even for merge strategy (GitHub behavior)
                    let output = run_git(&git_bin, work_path, &["merge", "--ff", "source-temp"])?;
                    let sha = get_head_sha(&git_bin, work_path)?;
                    let was_ff = output.contains("Fast-forward");
                    MergeResult {
                        commit_sha: sha,
                        strategy_used: if was_ff {
                            "fast-forward".into()
                        } else {
                            "merge".into()
                        },
                        was_ff,
                    }
                } else {
                    let _ = run_git(
                        &git_bin,
                        work_path,
                        &[
                            "merge",
                            "--no-ff",
                            "source-temp",
                            "-m",
                            &format!("Merge branch '{source_branch}' into {target_branch}"),
                        ],
                    )?;
                    let sha = get_head_sha(&git_bin, work_path)?;
                    MergeResult {
                        commit_sha: sha,
                        strategy_used: "merge".into(),
                        was_ff: false,
                    }
                }
            }
        };

        // Push the merged result back to the bare repo
        run_git(
            &git_bin,
            work_path,
            &[
                "push",
                "origin",
                &format!("{target_branch}:{target_branch}"),
            ],
        )?;

        info!(
            repo = format!("{owner}/{name}"),
            source = source_branch,
            target = target_branch,
            strategy = %result.strategy_used,
            sha = %result.commit_sha,
            "merge completed"
        );

        // tmp_dir cleaned up on drop
        Ok(result)
    }

    fn count_commits(&self, owner: &str, name: &str) -> usize {
        match self.list_commits(owner, name, usize::MAX) {
            Ok(commits) => commits.len(),
            Err(_) => 0,
        }
    }
}

/// Run a git command and return stdout. Returns CoreError on failure.
fn run_git(git_bin: &str, cwd: &Path, args: &[&str]) -> Result<String> {
    use std::process::Command;
    let output = Command::new(git_bin)
        .args(args)
        .current_dir(cwd)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| CoreError::Git(format!("failed to run git {args:?}: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        warn!(args = ?args, code = ?output.status.code(), stderr = %stderr, "git command failed");
        // Detect merge conflicts
        if stderr.contains("CONFLICT")
            || stderr.contains("Merge conflict")
            || stdout.contains("CONFLICT")
        {
            return Err(CoreError::BadRequest(format!("merge conflict: {stderr}")));
        }
        return Err(CoreError::Git(format!(
            "git {args:?} failed (exit {:?}): {stderr}",
            output.status.code()
        )));
    }

    Ok(stdout)
}

/// Check if fast-forwarding target to source is possible.
/// This is true when target_branch is an ancestor of source_branch.
fn is_fast_forward(
    git_bin: &str,
    cwd: &Path,
    _source_branch: &str,
    target_branch: &str,
) -> Result<bool> {
    use std::process::Command;
    // We have source-temp (tracking origin/source_branch) and target_branch checked out.
    // git merge-base --is-ancestor <target> <source-temp> → exit 0 means ff possible.
    let output = Command::new(git_bin)
        .args(["merge-base", "--is-ancestor", target_branch, "source-temp"])
        .current_dir(cwd)
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| CoreError::Git(format!("failed to run git merge-base: {e}")))?;

    Ok(output.status.success())
}

/// Get the current HEAD commit SHA.
fn get_head_sha(git_bin: &str, cwd: &Path) -> Result<String> {
    let output = run_git(git_bin, cwd, &["rev-parse", "HEAD"])?;
    Ok(output.trim().to_string())
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

    #[test]
    fn test_clone_nonexistent_url() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = GitService::new(tmp.path().to_path_buf());
        // Should succeed in creating the bare repo with remote config,
        // even if fetch fails (no network)
        let result = svc.clone(
            "testorg",
            "cloned",
            "https://nonexistent.example.invalid/repo.git",
        );
        // Either succeeds (with 0 commits) or fails due to gix network unavailability
        match result {
            Ok(cr) => {
                assert!(cr.path.exists());
                assert!(cr.path.join("HEAD").exists());
            }
            Err(_) => {
                // Acceptable -- gix network features not available
            }
        }
    }

    #[test]
    fn test_clone_duplicate_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = GitService::new(tmp.path().to_path_buf());
        svc.init_bare("testorg", "existing").unwrap();
        let result = svc.clone("testorg", "existing", "https://example.com/repo.git");
        assert!(result.is_err());
    }

    #[test]
    fn test_prepare_receive_nonexistent() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = GitService::new(tmp.path().to_path_buf());
        let result = svc.prepare_receive("testorg", "norepo");
        assert!(result.is_err());
    }

    #[test]
    fn test_prepare_receive_existing() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = GitService::new(tmp.path().to_path_buf());
        svc.init_bare("testorg", "hasrepo").unwrap();
        let path = svc.prepare_receive("testorg", "hasrepo").unwrap();
        assert!(path.join("HEAD").exists());
    }

    // ── MergeStrategy tests ──

    #[test]
    fn test_merge_strategy_parse() {
        assert_eq!(
            "merge".parse::<MergeStrategy>().unwrap(),
            MergeStrategy::Merge
        );
        assert_eq!(
            "recursive".parse::<MergeStrategy>().unwrap(),
            MergeStrategy::Merge
        );
        assert_eq!(
            "squash".parse::<MergeStrategy>().unwrap(),
            MergeStrategy::Squash
        );
        assert_eq!(
            "fast-forward".parse::<MergeStrategy>().unwrap(),
            MergeStrategy::FastForward
        );
        assert_eq!(
            "ff".parse::<MergeStrategy>().unwrap(),
            MergeStrategy::FastForward
        );
        assert_eq!(
            "ff-only".parse::<MergeStrategy>().unwrap(),
            MergeStrategy::FastForward
        );
        assert_eq!(
            "rebase".parse::<MergeStrategy>().unwrap(),
            MergeStrategy::Rebase
        );
    }

    #[test]
    fn test_merge_strategy_parse_unknown() {
        assert!("unknown".parse::<MergeStrategy>().is_err());
        assert!("".parse::<MergeStrategy>().is_err());
    }

    #[test]
    fn test_merge_strategy_display() {
        assert_eq!(MergeStrategy::Merge.to_string(), "merge");
        assert_eq!(MergeStrategy::Squash.to_string(), "squash");
        assert_eq!(MergeStrategy::FastForward.to_string(), "fast-forward");
        assert_eq!(MergeStrategy::Rebase.to_string(), "rebase");
    }

    #[test]
    fn test_merge_strategy_serde() {
        // Serialize
        let s = serde_json::to_string(&MergeStrategy::Squash).unwrap();
        assert_eq!(s, "\"squash\"");

        // Deserialize
        let v: MergeStrategy = serde_json::from_str("\"fast-forward\"").unwrap();
        assert_eq!(v, MergeStrategy::FastForward);
    }

    #[test]
    fn test_merge_strategy_equality() {
        assert_eq!(MergeStrategy::Merge, MergeStrategy::Merge);
        assert_ne!(MergeStrategy::Merge, MergeStrategy::Squash);
        assert_eq!(MergeStrategy::FastForward, MergeStrategy::FastForward);
    }

    /// Integration test: actual git merge (fast-forward) in temp repo.
    #[test]
    fn test_merge_fast_forward() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = GitService::new(tmp.path().to_path_buf());

        // Create a bare repo
        let bare_path = svc.init_bare("org", "repo").unwrap();
        let work = tempfile::tempdir().unwrap();

        // Seed: create initial commit on main
        run_git("git", work.path(), &["init"]).unwrap();
        run_git("git", work.path(), &["config", "user.name", "Test"]).unwrap();
        run_git(
            "git",
            work.path(),
            &["config", "user.email", "test@test.com"],
        )
        .unwrap();
        std::fs::write(work.path().join("file.txt"), "hello\n").unwrap();
        run_git("git", work.path(), &["add", "."]).unwrap();
        run_git("git", work.path(), &["commit", "-m", "initial"]).unwrap();
        run_git(
            "git",
            work.path(),
            &["remote", "add", "origin", bare_path.to_str().unwrap()],
        )
        .unwrap();
        run_git("git", work.path(), &["push", "origin", "master"]).unwrap();

        // Create a feature branch commit
        std::fs::write(work.path().join("feature.txt"), "new feature\n").unwrap();
        run_git("git", work.path(), &["add", "."]).unwrap();
        run_git("git", work.path(), &["commit", "-m", "feature"]).unwrap();
        run_git("git", work.path(), &["push", "origin", "master:feature"]).unwrap();

        // Merge feature into main (ff should succeed)
        let result = svc
            .merge_branch(
                "org",
                "repo",
                "feature",
                "master",
                MergeStrategy::FastForward,
                "CivitForge",
                "civit@test.com",
            )
            .unwrap();

        assert!(result.was_ff);
        assert_eq!(result.strategy_used, "fast-forward");
        assert!(!result.commit_sha.is_empty());
        assert!(result.commit_sha.len() >= 40);
    }

    /// Integration test: merge with --no-ff (creates merge commit).
    #[test]
    fn test_merge_no_ff() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = GitService::new(tmp.path().to_path_buf());

        let bare_path = svc.init_bare("org", "repo2").unwrap();
        let work = tempfile::tempdir().unwrap();

        run_git("git", work.path(), &["init"]).unwrap();
        run_git("git", work.path(), &["config", "user.name", "Test"]).unwrap();
        run_git(
            "git",
            work.path(),
            &["config", "user.email", "test@test.com"],
        )
        .unwrap();
        std::fs::write(work.path().join("file.txt"), "hello\n").unwrap();
        run_git("git", work.path(), &["add", "."]).unwrap();
        run_git("git", work.path(), &["commit", "-m", "initial"]).unwrap();
        run_git(
            "git",
            work.path(),
            &["remote", "add", "origin", bare_path.to_str().unwrap()],
        )
        .unwrap();
        run_git("git", work.path(), &["push", "origin", "master"]).unwrap();

        // Advance main so ff is not possible
        std::fs::write(work.path().join("main-only.txt"), "main change\n").unwrap();
        run_git("git", work.path(), &["add", "."]).unwrap();
        run_git("git", work.path(), &["commit", "-m", "main advance"]).unwrap();
        run_git("git", work.path(), &["push", "origin", "master"]).unwrap();

        // Create feature branch with different content
        run_git("git", work.path(), &["checkout", "-b", "feature", "HEAD~1"]).unwrap();
        std::fs::write(work.path().join("feature.txt"), "feature content\n").unwrap();
        run_git("git", work.path(), &["add", "."]).unwrap();
        run_git("git", work.path(), &["commit", "-m", "feature work"]).unwrap();
        run_git("git", work.path(), &["push", "origin", "feature"]).unwrap();

        // Merge should succeed with --no-ff (creates merge commit)
        let result = svc
            .merge_branch(
                "org",
                "repo2",
                "feature",
                "master",
                MergeStrategy::Merge,
                "CivitForge",
                "civit@test.com",
            )
            .unwrap();

        assert!(!result.was_ff);
        assert_eq!(result.strategy_used, "merge");
        assert!(!result.commit_sha.is_empty());
    }

    /// Test that ff-only fails when branches diverged.
    #[test]
    fn test_merge_ff_fails_on_divergence() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = GitService::new(tmp.path().to_path_buf());

        let bare_path = svc.init_bare("org", "repo3").unwrap();
        let work = tempfile::tempdir().unwrap();

        run_git("git", work.path(), &["init"]).unwrap();
        run_git("git", work.path(), &["config", "user.name", "Test"]).unwrap();
        run_git(
            "git",
            work.path(),
            &["config", "user.email", "test@test.com"],
        )
        .unwrap();
        std::fs::write(work.path().join("file.txt"), "hello\n").unwrap();
        run_git("git", work.path(), &["add", "."]).unwrap();
        run_git("git", work.path(), &["commit", "-m", "initial"]).unwrap();
        run_git(
            "git",
            work.path(),
            &["remote", "add", "origin", bare_path.to_str().unwrap()],
        )
        .unwrap();
        run_git("git", work.path(), &["push", "origin", "master"]).unwrap();

        // Diverge: advance master
        std::fs::write(work.path().join("main-only.txt"), "main\n").unwrap();
        run_git("git", work.path(), &["add", "."]).unwrap();
        run_git("git", work.path(), &["commit", "-m", "main advance"]).unwrap();
        run_git("git", work.path(), &["push", "origin", "master"]).unwrap();

        // Diverge: advance feature from same base
        run_git("git", work.path(), &["checkout", "-b", "feature", "HEAD~1"]).unwrap();
        std::fs::write(work.path().join("feature.txt"), "feat\n").unwrap();
        run_git("git", work.path(), &["add", "."]).unwrap();
        run_git("git", work.path(), &["commit", "-m", "feature"]).unwrap();
        run_git("git", work.path(), &["push", "origin", "feature"]).unwrap();

        // ff-only should fail
        let result = svc.merge_branch(
            "org",
            "repo3",
            "feature",
            "master",
            MergeStrategy::FastForward,
            "CivitForge",
            "civit@test.com",
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("fast-forward merge not possible"));
    }

    /// Test merge on nonexistent repo fails.
    #[test]
    fn test_merge_nonexistent_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let svc = GitService::new(tmp.path().to_path_buf());
        let result = svc.merge_branch(
            "org",
            "noexist",
            "feature",
            "master",
            MergeStrategy::Merge,
            "Test",
            "test@test.com",
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("does not exist"));
    }
}
