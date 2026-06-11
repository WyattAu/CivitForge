use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitGraphNode {
    pub id: String,
    pub message: String,
    pub author: String,
    pub date: String,
    pub parents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitGraphEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphBranchInfo {
    pub name: String,
    pub head: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitGraph {
    pub nodes: Vec<CommitGraphNode>,
    pub edges: Vec<CommitGraphEdge>,
    pub branches: Vec<GraphBranchInfo>,
}

pub fn generate_commit_graph(repo_path: &Path, max_commits: usize) -> Result<CommitGraph> {
    if !repo_path.join("HEAD").exists() {
        return Err(anyhow::anyhow!(
            "repository not found at {}",
            repo_path.display()
        ));
    }

    let git_bin = std::env::var("GIT_BIN").unwrap_or_else(|_| "/usr/bin/git".to_string());

    let limit = format!("{max_commits}");
    let log_output = std::process::Command::new(&git_bin)
        .current_dir(repo_path)
        .args(["log", "--all", "--format=%H|%s|%an|%aI|%P", "-n", &limit])
        .output()
        .context("failed to run git log")?;

    if !log_output.status.success() {
        return Ok(CommitGraph {
            nodes: Vec::new(),
            edges: Vec::new(),
            branches: Vec::new(),
        });
    }

    let stdout = String::from_utf8_lossy(&log_output.stdout);
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.splitn(5, '|').collect();
        if parts.len() < 4 {
            continue;
        }
        let id = parts[0].to_string();
        let short_id: String = id.chars().take(7).collect();
        let message = parts[1].to_string();
        let author = parts[2].to_string();
        let date = parts[3].to_string();
        let parents: Vec<String> = parts
            .get(4)
            .map(|s| s.split_whitespace().map(|p| p.to_string()).collect())
            .unwrap_or_default();

        if !seen.contains(&id) {
            seen.insert(id.clone());
            nodes.push(CommitGraphNode {
                id: short_id,
                message,
                author,
                date,
                parents: parents
                    .iter()
                    .map(|p| p.chars().take(7).collect())
                    .collect(),
            });
        }

        for parent_id in &parents {
            let parent_short: String = parent_id.chars().take(7).collect();
            let child_short: String = id.chars().take(7).collect();
            edges.push(CommitGraphEdge {
                from: parent_short,
                to: child_short,
            });
        }
    }

    let branch_output = std::process::Command::new(&git_bin)
        .current_dir(repo_path)
        .args(["branch", "--format=%(refname:short)|%(objectname)"])
        .output();

    let branches = match branch_output {
        Ok(o) if o.status.success() => {
            let out = String::from_utf8_lossy(&o.stdout);
            out.lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.splitn(2, '|').collect();
                    if parts.len() == 2 {
                        Some(GraphBranchInfo {
                            name: parts[0].to_string(),
                            head: parts[1].chars().take(7).collect(),
                        })
                    } else {
                        None
                    }
                })
                .collect()
        }
        _ => Vec::new(),
    };

    Ok(CommitGraph {
        nodes,
        edges,
        branches,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_serialization() {
        let graph = CommitGraph {
            nodes: vec![CommitGraphNode {
                id: "abc1234".into(),
                message: "test".into(),
                author: "user".into(),
                date: "2024-01-01T00:00:00+00:00".into(),
                parents: vec![],
            }],
            edges: vec![CommitGraphEdge {
                from: "abc1234".into(),
                to: "def5678".into(),
            }],
            branches: vec![GraphBranchInfo {
                name: "main".into(),
                head: "abc1234".into(),
            }],
        };
        let json = serde_json::to_string(&graph).unwrap();
        assert!(json.contains("\"nodes\""));
        assert!(json.contains("\"edges\""));
        assert!(json.contains("\"branches\""));
    }

    fn create_repo_with_commits(path: &Path, count: usize) {
        let work_tmp = tempfile::tempdir().unwrap();
        let work = work_tmp.path();
        std::process::Command::new("git").args(["init", "-b", "main", work.to_str().unwrap()]).output().unwrap();
        std::process::Command::new("git").args(["config", "user.name", "Test"]).current_dir(work).output().unwrap();
        std::process::Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(work).output().unwrap();

        for i in 0..count {
            std::fs::write(work.join(format!("file{i}.txt")), format!("content {i}")).unwrap();
            std::process::Command::new("git").args(["add", "."]).current_dir(work).output().unwrap();
            std::process::Command::new("git").args(["commit", "-m", &format!("commit {i}")]).current_dir(work).output().unwrap();
        }
        std::fs::create_dir_all(path).unwrap();
        std::process::Command::new("git").args(["clone", "--bare", work.to_str().unwrap(), path.to_str().unwrap()]).output().unwrap();
    }

    #[test]
    fn test_generate_commit_graph_basic() {
        let tmp = tempfile::tempdir().unwrap();
        create_repo_with_commits(tmp.path(), 3);
        let graph = generate_commit_graph(tmp.path(), 10).unwrap();
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2); // each commit has one parent edge
        assert!(!graph.branches.is_empty());
    }

    #[test]
    fn test_generate_commit_graph_with_branches() {
        let tmp = tempfile::tempdir().unwrap();
        create_repo_with_commits(tmp.path(), 1);
        // Create a branch
        let work_tmp = tempfile::tempdir().unwrap();
        let work = work_tmp.path();
        // Clone from the bare repo
        std::process::Command::new("git")
            .args(["clone", tmp.path().to_str().unwrap(), "."])
            .current_dir(work)
            .output()
            .unwrap();
        // The bare repo's HEAD points to the initial branch (master)
        std::process::Command::new("git").args(["config", "user.name", "Test"]).current_dir(work).output().unwrap();
        std::process::Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(work).output().unwrap();
        // Detect default branch name from the bare repo's HEAD
        let head_content = std::fs::read_to_string(tmp.path().join("HEAD")).unwrap();
        let _default_branch = head_content.trim().strip_prefix("ref: refs/heads/").unwrap_or("master").to_string();
        std::process::Command::new("git").args(["checkout", "-b", "feature"]).current_dir(work).output().unwrap();
        std::fs::write(work.join("feat.txt"), "feat").unwrap();
        std::process::Command::new("git").args(["add", "."]).current_dir(work).output().unwrap();
        std::process::Command::new("git").args(["commit", "-m", "feature"]).current_dir(work).output().unwrap();
        std::process::Command::new("git").args(["push", "origin", "feature"]).current_dir(work).output().unwrap();

        let graph = generate_commit_graph(tmp.path(), 10).unwrap();
        assert!(graph.nodes.len() >= 2);
        let branch_names: Vec<&str> = graph.branches.iter().map(|b| b.name.as_str()).collect();
        assert!(branch_names.contains(&"main") || branch_names.contains(&"master"), "expected main or master in {branch_names:?}");
        assert!(branch_names.contains(&"feature"));
    }

    #[test]
    fn test_generate_commit_graph_empty_repo() {
        let tmp = tempfile::tempdir().unwrap();
        std::process::Command::new("git")
            .args(["init", "--bare", tmp.path().to_str().unwrap()])
            .output()
            .unwrap();
        let graph = generate_commit_graph(tmp.path(), 10).unwrap();
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_generate_commit_graph_nonexistent_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let result = generate_commit_graph(tmp.path(), 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_commit_graph_respects_limit() {
        let tmp = tempfile::tempdir().unwrap();
        create_repo_with_commits(tmp.path(), 5);
        let graph = generate_commit_graph(tmp.path(), 2).unwrap();
        assert_eq!(graph.nodes.len(), 2);
    }
}
