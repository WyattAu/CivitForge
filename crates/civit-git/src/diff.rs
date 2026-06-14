use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEntry {
    pub status: String,
    pub path: String,
    pub old_path: Option<String>,
    pub additions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub entries: Vec<DiffEntry>,
    pub total_additions: usize,
    pub total_deletions: usize,
    pub base_ref: String,
    pub head_ref: String,
}

pub fn generate_diff(repo_path: &Path, base_ref: &str, head_ref: &str) -> Result<DiffResult> {
    if !repo_path.join("HEAD").exists() {
        return Err(anyhow::anyhow!(
            "repository not found at {}",
            repo_path.display()
        ));
    }

    let git_bin = std::env::var("GIT_BIN").unwrap_or_else(|_| "/usr/bin/git".to_string());

    let output = std::process::Command::new(&git_bin)
        .current_dir(repo_path)
        .args([
            "diff",
            "--numstat",
            "--no-renames",
            base_ref,
            "--",
            head_ref,
        ])
        .output()
        .context("failed to run git diff")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("git diff failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();
    let mut total_additions = 0usize;
    let mut total_deletions = 0usize;

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 {
            continue;
        }

        let additions = parts[0].parse::<usize>().unwrap_or(0);
        let deletions = parts[1].parse::<usize>().unwrap_or(0);
        let path = parts[2].to_string();

        let status = if parts[0] == "-" {
            "deleted"
        } else if parts[1] == "-" {
            "added"
        } else {
            "modified"
        };

        total_additions += additions;
        total_deletions += deletions;

        entries.push(DiffEntry {
            status: status.to_string(),
            path,
            old_path: None,
            additions,
            deletions,
        });
    }

    Ok(DiffResult {
        entries,
        total_additions,
        total_deletions,
        base_ref: base_ref.to_string(),
        head_ref: head_ref.to_string(),
    })
}

pub fn generate_commit_diff(repo_path: &Path, commit_sha: &str) -> Result<DiffResult> {
    if !repo_path.join("HEAD").exists() {
        return Err(anyhow::anyhow!(
            "repository not found at {}",
            repo_path.display()
        ));
    }

    let git_bin = std::env::var("GIT_BIN").unwrap_or_else(|_| "/usr/bin/git".to_string());

    let output = std::process::Command::new(&git_bin)
        .current_dir(repo_path)
        .args([
            "diff",
            "--numstat",
            "--no-renames",
            &format!("{commit_sha}~1"),
            commit_sha,
        ])
        .output()
        .context("failed to run git diff")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("git diff failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();
    let mut total_additions = 0usize;
    let mut total_deletions = 0usize;

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 {
            continue;
        }

        let additions = parts[0].parse::<usize>().unwrap_or(0);
        let deletions = parts[1].parse::<usize>().unwrap_or(0);
        let path = parts[2].to_string();

        let status = if parts[0] == "-" {
            "deleted"
        } else if parts[1] == "-" {
            "added"
        } else {
            "modified"
        };

        total_additions += additions;
        total_deletions += deletions;

        entries.push(DiffEntry {
            status: status.to_string(),
            path,
            old_path: None,
            additions,
            deletions,
        });
    }

    Ok(DiffResult {
        entries,
        total_additions,
        total_deletions,
        base_ref: format!("{commit_sha}~1"),
        head_ref: commit_sha.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_result_serialization() {
        let result = DiffResult {
            entries: vec![DiffEntry {
                status: "modified".into(),
                path: "src/main.rs".into(),
                old_path: None,
                additions: 10,
                deletions: 5,
            }],
            total_additions: 10,
            total_deletions: 5,
            base_ref: "abc1234".into(),
            head_ref: "def5678".into(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"total_additions\":10"));
        assert!(json.contains("\"total_deletions\":5"));
    }

    fn create_repo_with_commits(path: &Path) -> Vec<String> {
        // Create a work directory, then convert to bare (since diff functions check for HEAD at root)
        let work_tmp = tempfile::tempdir().unwrap();
        let work = work_tmp.path();
        std::process::Command::new("git")
            .args(["init", "-b", "main", work.to_str().unwrap()])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(work)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(work)
            .output()
            .unwrap();

        // Commit 1: initial
        std::fs::write(work.join("file.txt"), "line1\nline2\n").unwrap();
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(work)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(work)
            .output()
            .unwrap();
        let sha1 = String::from_utf8(
            std::process::Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(work)
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
        .trim()
        .to_string();

        // Commit 2: modify
        std::fs::write(work.join("file.txt"), "line1\nchanged\nline3\n").unwrap();
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(work)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "modify"])
            .current_dir(work)
            .output()
            .unwrap();
        let sha2 = String::from_utf8(
            std::process::Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(work)
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
        .trim()
        .to_string();

        // Commit 3: add new file
        std::fs::write(work.join("new.txt"), "new content").unwrap();
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(work)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "add file"])
            .current_dir(work)
            .output()
            .unwrap();
        let sha3 = String::from_utf8(
            std::process::Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(work)
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
        .trim()
        .to_string();

        // Convert to bare
        std::fs::create_dir_all(path).unwrap();
        std::process::Command::new("git")
            .args([
                "clone",
                "--bare",
                work.to_str().unwrap(),
                path.to_str().unwrap(),
            ])
            .output()
            .unwrap();

        vec![sha1, sha2, sha3]
    }

    #[test]
    fn test_generate_commit_diff_basic() {
        let tmp = tempfile::tempdir().unwrap();
        let shas = create_repo_with_commits(tmp.path());
        let result = generate_commit_diff(tmp.path(), &shas[1]).unwrap();
        assert!(!result.entries.is_empty());
        assert_eq!(result.head_ref, shas[1]);
    }

    #[test]
    fn test_generate_commit_diff_additions() {
        let tmp = tempfile::tempdir().unwrap();
        let shas = create_repo_with_commits(tmp.path());
        let result = generate_commit_diff(tmp.path(), &shas[2]).unwrap();
        // Commit 3 adds new.txt - diff --numstat shows additions > 0
        let new_file = result.entries.iter().find(|e| e.path == "new.txt");
        assert!(new_file.is_some());
        assert!(new_file.unwrap().additions > 0);
    }

    #[test]
    fn test_generate_commit_diff_between_commits() {
        let tmp = tempfile::tempdir().unwrap();
        let shas = create_repo_with_commits(tmp.path());
        // Use generate_commit_diff on sha2 which compares sha2~1 (sha1) to sha2
        let result = generate_commit_diff(tmp.path(), &shas[1]).unwrap();
        assert!(!result.entries.is_empty());
        assert!(result.total_additions > 0 || result.total_deletions > 0);
        assert_eq!(result.base_ref, format!("{}~1", shas[1]));
        assert_eq!(result.head_ref, shas[1]);
    }

    #[test]
    fn test_generate_diff_nonexistent_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let result = generate_diff(tmp.path(), "a", "b");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_commit_diff_nonexistent_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let result = generate_commit_diff(tmp.path(), "abc1234");
        assert!(result.is_err());
    }
}
