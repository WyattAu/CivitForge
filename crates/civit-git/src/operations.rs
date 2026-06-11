use anyhow::{Context, Result};
use chrono::DateTime;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MergeStrategy {
    Merge,
    Squash,
    #[serde(rename = "fast-forward")]
    FastForward,
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
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "merge" | "recursive" => Ok(Self::Merge),
            "squash" => Ok(Self::Squash),
            "fast-forward" | "ff" | "ff-only" => Ok(Self::FastForward),
            "rebase" => Ok(Self::Rebase),
            _ => Err(anyhow::anyhow!("unknown merge strategy: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub commit_sha: String,
    pub strategy_used: String,
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
            return Err(anyhow::anyhow!("repository already exists: {name}"));
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        gix::init_bare(&path).context("failed to init bare repo")?;
        let head_path = path.join("HEAD");
        if !head_path.exists() {
            std::fs::write(&head_path, "ref: refs/heads/main\n")?;
        }
        info!(path = %path.display(), "initialized bare repository");
        Ok(path)
    }

    pub fn repo_exists(&self, owner: &str, name: &str) -> bool {
        self.repo_path(owner, name).join("HEAD").exists()
    }

    pub fn list_commits(&self, owner: &str, name: &str, limit: usize) -> Result<Vec<CommitInfo>> {
        let path = self.repo_path(owner, name);
        let repo = gix::open(&path).context("failed to open repo")?;

        let head_id = repo.head_id().context("failed to get HEAD")?;

        let mut commits = Vec::new();
        let mut current_id = head_id;

        while commits.len() < limit {
            let commit_obj = current_id.object()?;
            let commit = commit_obj.try_into_commit()?;

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
                        DateTime::from_timestamp(t.seconds, 0)
                            .map(|dt| dt.to_rfc3339())
                            .unwrap_or_default()
                    })
                    .unwrap_or_default(),
                parents,
            });

            match parent_ids.first() {
                Some(first_parent) => current_id = *first_parent,
                None => break,
            }
        }

        debug!(repo = %name, count = commits.len(), "listed commits");
        Ok(commits)
    }

    pub fn get_default_branch(&self, owner: &str, name: &str) -> Result<String> {
        let path = self.repo_path(owner, name);
        let repo = gix::open(&path).context("failed to open repo")?;
        let head_ref = repo
            .head_ref()
            .context("failed to get HEAD ref")?
            .ok_or_else(|| anyhow::anyhow!("no HEAD reference"))?;
        let branch_name = head_ref.name().shorten().to_string();
        debug!(branch = %branch_name, "got default branch");
        Ok(branch_name)
    }

    pub fn clone(&self, owner: &str, name: &str, remote_url: &str) -> Result<CloneResult> {
        let path = self.repo_path(owner, name);

        if path.exists() {
            return Err(anyhow::anyhow!("repository already exists: {name}"));
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        gix::init_bare(&path).context("failed to init bare repo")?;
        let repo = gix::open(&path).context("failed to open repo")?;
        let _remote = repo
            .remote_at(remote_url)
            .context("failed to create remote")?;

        info!(remote = %remote_url, path = %path.display(), "initialized clone repo with remote");

        let commit_count = self.count_commits(owner, name);
        let branch_count = 0usize;

        Ok(CloneResult {
            path,
            commit_count,
            branch_count,
        })
    }

    pub fn prepare_receive(&self, owner: &str, name: &str) -> Result<PathBuf> {
        let path = self.repo_path(owner, name);
        if !path.exists() {
            return Err(anyhow::anyhow!("repository does not exist: {name}"));
        }
        gix::open(&path).context("failed to open repo")?;
        Ok(path)
    }

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
            return Err(anyhow::anyhow!(
                "repository {owner}/{name} does not exist"
            ));
        }

        let tmp_dir = tempfile::tempdir().context("failed to create temp dir")?;
        let work_path = tmp_dir.path();

        run_git(
            work_path,
            &[
                "clone",
                "--no-checkout",
                bare_path.to_str().unwrap_or(""),
                work_path.to_str().unwrap_or(""),
            ],
        )?;

        run_git(
            work_path,
            &["config", "user.name", committer_name],
        )?;
        run_git(
            work_path,
            &["config", "user.email", committer_email],
        )?;

        run_git(work_path, &["checkout", target_branch])?;
        run_git(work_path, &["fetch", "origin"])?;

        run_git(
            work_path,
            &[
                "branch",
                "-f",
                "source-temp",
                &format!("origin/{source_branch}"),
            ],
        )?;

        let can_ff = is_fast_forward(work_path, source_branch, target_branch)?;

        let result = match strategy {
            MergeStrategy::FastForward => {
                if !can_ff {
                    return Err(anyhow::anyhow!(
                        "fast-forward merge not possible: {target_branch} has diverged from {source_branch}"
                    ));
                }
                let _output = run_git(work_path, &["merge", "--ff-only", "source-temp"])?;
                let sha = get_head_sha(work_path)?;
                MergeResult {
                    commit_sha: sha,
                    strategy_used: "fast-forward".into(),
                    was_ff: true,
                }
            }
            MergeStrategy::Squash => {
                let _ =
                    run_git(work_path, &["merge", "--squash", "source-temp"])?;
                let _ = run_git(
                    work_path,
                    &[
                        "commit",
                        "-m",
                        &format!("Merge branch '{source_branch}' into {target_branch} (squash)"),
                    ],
                )?;
                let sha = get_head_sha(work_path)?;
                MergeResult {
                    commit_sha: sha,
                    strategy_used: "squash".into(),
                    was_ff: false,
                }
            }
            MergeStrategy::Rebase => {
                run_git(work_path, &["checkout", "source-temp"])?;
                run_git(work_path, &["rebase", target_branch])?;
                run_git(work_path, &["checkout", target_branch])?;
                run_git(work_path, &["merge", "--ff-only", "source-temp"])?;
                let sha = get_head_sha(work_path)?;
                MergeResult {
                    commit_sha: sha,
                    strategy_used: "rebase".into(),
                    was_ff: false,
                }
            }
            MergeStrategy::Merge => {
                if can_ff {
                    let output =
                        run_git(work_path, &["merge", "--ff", "source-temp"])?;
                    let sha = get_head_sha(work_path)?;
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
                        work_path,
                        &[
                            "merge",
                            "--no-ff",
                            "source-temp",
                            "-m",
                            &format!("Merge branch '{source_branch}' into {target_branch}"),
                        ],
                    )?;
                    let sha = get_head_sha(work_path)?;
                    MergeResult {
                        commit_sha: sha,
                        strategy_used: "merge".into(),
                        was_ff: false,
                    }
                }
            }
        };

        run_git(
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

        Ok(result)
    }

    pub fn open_repo(&self, owner: &str, name: &str) -> Result<gix::Repository> {
        let path = self.repo_path(owner, name);
        if !path.join("HEAD").exists() {
            return Err(anyhow::anyhow!("repository not found: {owner}/{name}"));
        }
        gix::open(&path).context("failed to open repo")
    }

    fn count_commits(&self, owner: &str, name: &str) -> usize {
        match self.list_commits(owner, name, usize::MAX) {
            Ok(commits) => commits.len(),
            Err(_) => 0,
        }
    }
}

fn run_git(cwd: &Path, args: &[&str]) -> Result<String> {
    let git_bin = std::env::var("GIT_BIN").unwrap_or_else(|_| "git".to_string());
    let output = std::process::Command::new(&git_bin)
        .args(args)
        .current_dir(cwd)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context(format!("failed to run git {args:?}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        warn!(args = ?args, code = ?output.status.code(), stderr = %stderr, "git command failed");
        if stderr.contains("CONFLICT")
            || stderr.contains("Merge conflict")
            || stdout.contains("CONFLICT")
        {
            return Err(anyhow::anyhow!("merge conflict: {stderr}"));
        }
        return Err(anyhow::anyhow!(
            "git {:?} failed (exit {:?}): {stderr}",
            args,
            output.status.code()
        ));
    }

    Ok(stdout)
}

fn is_fast_forward(cwd: &Path, _source_branch: &str, target_branch: &str) -> Result<bool> {
    let git_bin = std::env::var("GIT_BIN").unwrap_or_else(|_| "git".to_string());
    let output = std::process::Command::new(git_bin)
        .args(["merge-base", "--is-ancestor", target_branch, "source-temp"])
        .current_dir(cwd)
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("failed to run git merge-base")?;
    Ok(output.status.success())
}

fn get_head_sha(cwd: &Path) -> Result<String> {
    let output = run_git(cwd, &["rev-parse", "HEAD"])?;
    Ok(output.trim().to_string())
}

pub fn run_git_command(cwd: &Path, args: &[&str]) -> Result<String> {
    run_git(cwd, args)
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
        let result = svc.clone(
            "testorg",
            "cloned",
            "https://nonexistent.example.invalid/repo.git",
        );
        if let Ok(cr) = result {
            assert!(cr.path.exists());
            assert!(cr.path.join("HEAD").exists());
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
    fn test_merge_strategy_parse() {
        assert_eq!(
            "merge".parse::<MergeStrategy>().unwrap(),
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
            "rebase".parse::<MergeStrategy>().unwrap(),
            MergeStrategy::Rebase
        );
    }

    #[test]
    fn test_merge_strategy_parse_unknown() {
        assert!("unknown".parse::<MergeStrategy>().is_err());
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
        let s = serde_json::to_string(&MergeStrategy::Squash).unwrap();
        assert_eq!(s, "\"squash\"");
        let v: MergeStrategy = serde_json::from_str("\"fast-forward\"").unwrap();
        assert_eq!(v, MergeStrategy::FastForward);
    }
}
