use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlameLine {
    pub line_number: usize,
    pub content: String,
    pub commit_id: String,
    pub commit_message: String,
    pub author: String,
    pub time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlameResult {
    pub lines: Vec<BlameLine>,
    pub path: String,
    pub language: String,
}

pub fn git_blame(repo_path: &Path, ref_name: &str, file_path: &str) -> Result<BlameResult> {
    if !repo_path.join("HEAD").exists() {
        return Err(anyhow::anyhow!("repository not found at {}", repo_path.display()));
    }

    let git_bin = std::env::var("GIT_BIN").unwrap_or_else(|_| "/usr/bin/git".to_string());

    let output = std::process::Command::new(&git_bin)
        .current_dir(repo_path)
        .args(["blame", "--porcelain", ref_name, "--", file_path])
        .output()
        .context("failed to run git blame")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("git blame failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = Vec::new();
    let mut current_commit_id = String::new();
    let mut current_message = String::new();
    let mut current_author = String::new();
    let mut current_time = String::new();

    for line_str in stdout.lines() {
        if let Some(author) = line_str.strip_prefix("author ") {
            current_author = author.to_string();
        } else if let Some(ts_str) = line_str.strip_prefix("author-time ") {
            let ts: i64 = ts_str.parse().unwrap_or(0);
            let dt = chrono::DateTime::from_timestamp(ts, 0).unwrap_or_default();
            current_time = dt.format("%Y-%m-%d").to_string();
        } else if let Some(msg) = line_str.strip_prefix("summary ") {
            current_message = msg.to_string();
        } else if !line_str.is_empty()
            && !line_str.starts_with("author")
            && !line_str.starts_with("committer")
            && !line_str.starts_with("previous ")
            && !line_str.starts_with("filename ")
            && !line_str.contains('\t')
        {
            let parts: Vec<&str> = line_str.split_whitespace().collect();
            if !parts.is_empty() {
                current_commit_id = parts[0].chars().take(7).collect();
            }
        } else if let Some(content) = line_str.strip_prefix('\t') {
            lines.push(BlameLine {
                line_number: 0,
                content: content.to_string(),
                commit_id: current_commit_id.clone(),
                commit_message: current_message.clone(),
                author: current_author.clone(),
                time: current_time.clone(),
            });
        }
    }

    for (i, blame_line) in lines.iter_mut().enumerate() {
        blame_line.line_number = i + 1;
    }

    let language = crate::tree::detect_language(file_path);

    Ok(BlameResult {
        lines,
        path: file_path.to_string(),
        language,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blame_result_serialization() {
        let result = BlameResult {
            lines: vec![BlameLine {
                line_number: 1,
                content: "hello".into(),
                commit_id: "abc1234".into(),
                commit_message: "init".into(),
                author: "test".into(),
                time: "2024-01-01".into(),
            }],
            path: "file.txt".into(),
            language: "text".into(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"commit_id\":\"abc1234\""));
    }
}
