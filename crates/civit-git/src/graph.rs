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
}
