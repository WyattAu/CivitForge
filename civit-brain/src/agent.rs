#![forbid(unsafe_code)]

use crate::rag::RAGPipeline;

use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewComment {
    pub file_path: String,
    pub line: usize,
    pub severity: Severity,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub comments: Vec<ReviewComment>,
    pub score: f32,
    pub summary: String,
}

pub struct ReviewAgent {
    rag: RAGPipeline,
}

impl ReviewAgent {
    pub fn new(rag: RAGPipeline) -> Self {
        Self { rag }
    }

    pub async fn review_diff(
        &self,
        diff_content: &str,
        file_path: &str,
    ) -> anyhow::Result<ReviewResult> {
        let context = self.rag.retrieve(diff_content).await?;
        let _prompt = self
            .rag
            .build_prompt(&context, &format!("Review this diff:\n{diff_content}"));

        let comments = self.analyze_diff(diff_content, file_path, &context);
        let score = self.calculate_score(&comments);
        let summary = self.generate_summary(&comments, score);

        info!(
            file = %file_path,
            comments = comments.len(),
            score = score,
            "review complete"
        );

        Ok(ReviewResult {
            comments,
            score,
            summary,
        })
    }

    fn analyze_diff(
        &self,
        diff: &str,
        file_path: &str,
        _context: &crate::rag::RAGContext,
    ) -> Vec<ReviewComment> {
        let mut comments = Vec::new();
        let mut line_num = 1usize;

        for line in diff.lines() {
            if line.starts_with("@@") {
                if let Some(hunk) = parse_hunk_header(line) {
                    line_num = hunk;
                }
                continue;
            }
            if line.starts_with('+') && !line.starts_with("++") {
                let code = line.strip_prefix('+').unwrap_or("");
                if let Some(comment) = self.check_line(code, file_path, line_num) {
                    comments.push(comment);
                }
            }
            if !line.starts_with('-') && !line.starts_with('+') && !line.starts_with('\\') {
                line_num += 1;
            }
        }

        comments
    }

    fn check_line(&self, code: &str, file_path: &str, line: usize) -> Option<ReviewComment> {
        let trimmed = code.trim();

        if trimmed.contains("unwrap()") && file_path.ends_with(".rs") {
            return Some(ReviewComment {
                file_path: file_path.into(),
                line,
                severity: Severity::Warning,
                message: "Use of unwrap() detected. Consider using proper error handling.".into(),
                suggestion: Some("Replace with .map_err()? or .unwrap_or_default()".into()),
            });
        }

        if trimmed.contains("clone()") && file_path.ends_with(".rs") {
            return Some(ReviewComment {
                file_path: file_path.into(),
                line,
                severity: Severity::Info,
                message: "clone() detected. Verify this is necessary for ownership.".into(),
                suggestion: None,
            });
        }

        if trimmed.starts_with("println!") && file_path.ends_with(".rs") {
            return Some(ReviewComment {
                file_path: file_path.into(),
                line,
                severity: Severity::Warning,
                message: "println! found. Use tracing macros in production code.".into(),
                suggestion: Some("Replace with tracing::info! or tracing::debug!".into()),
            });
        }

        if (trimmed.contains("password")
            || trimmed.contains("secret")
            || trimmed.contains("api_key"))
            && !trimmed.contains("env::")
            && !trimmed.contains("ENV")
        {
            return Some(ReviewComment {
                file_path: file_path.into(),
                line,
                severity: Severity::Error,
                message: "Potential secret/credential in code. Use environment variables.".into(),
                suggestion: Some("Move to environment variable or secrets manager.".into()),
            });
        }

        if trimmed.contains("TODO") || trimmed.contains("FIXME") {
            return Some(ReviewComment {
                file_path: file_path.into(),
                line,
                severity: Severity::Info,
                message: format!(
                    "Note: {} found in code",
                    trimmed.split_whitespace().next().unwrap()
                ),
                suggestion: None,
            });
        }

        None
    }

    fn calculate_score(&self, comments: &[ReviewComment]) -> f32 {
        if comments.is_empty() {
            return 1.0;
        }
        let penalty: f32 = comments
            .iter()
            .map(|c| match c.severity {
                Severity::Info => 0.02,
                Severity::Warning => 0.05,
                Severity::Error => 0.15,
            })
            .sum();
        (1.0 - penalty).max(0.0)
    }

    fn generate_summary(&self, comments: &[ReviewComment], score: f32) -> String {
        let errors = comments
            .iter()
            .filter(|c| c.severity == Severity::Error)
            .count();
        let warnings = comments
            .iter()
            .filter(|c| c.severity == Severity::Warning)
            .count();
        let infos = comments
            .iter()
            .filter(|c| c.severity == Severity::Info)
            .count();

        format!(
            "Review score: {:.0}/100. Found {} error(s), {} warning(s), {} info(s).",
            score * 100.0,
            errors,
            warnings,
            infos,
        )
    }
}

fn parse_hunk_header(line: &str) -> Option<usize> {
    let parts: Vec<&str> = line.split('+').collect();
    if parts.len() >= 2 {
        let start_part = parts.get(1)?;
        let num_str: String = start_part
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        num_str.parse().ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::EmbeddingWorker;
    use crate::vectordb::{DistanceMetric, VectorDbClient, VectorDbConfig};

    fn make_agent() -> ReviewAgent {
        let worker = EmbeddingWorker::new(8);
        let db = VectorDbClient::new(VectorDbConfig {
            collection_name: "test".into(),
            dimension: 8,
            distance_metric: DistanceMetric::Cosine,
        });
        let rag = RAGPipeline::new(worker, db, 5, 0.0);
        ReviewAgent::new(rag)
    }

    #[tokio::test]
    async fn test_review_clean_diff() {
        let agent = make_agent();
        let diff = "@@ -1,3 +1,3 @@\n let x = 5;\n let y = 10;\n let z = x + y;\n";
        let result = agent.review_diff(diff, "src/main.rs").await.unwrap();
        assert_eq!(result.comments.len(), 0);
        assert_eq!(result.score, 1.0);
    }

    #[tokio::test]
    async fn test_review_detects_unwrap() {
        let agent = make_agent();
        let diff = "@@ -1,1 +1,1 @@\n+let val = data.unwrap();\n";
        let result = agent.review_diff(diff, "src/main.rs").await.unwrap();
        assert!(result.comments.iter().any(|c| c.message.contains("unwrap")));
    }

    #[tokio::test]
    async fn test_review_detects_println() {
        let agent = make_agent();
        let diff = "@@ -1,1 +1,1 @@\n+println!(\"hello\");\n";
        let result = agent.review_diff(diff, "src/main.rs").await.unwrap();
        assert!(
            result
                .comments
                .iter()
                .any(|c| c.message.contains("println"))
        );
    }

    #[tokio::test]
    async fn test_review_detects_secrets() {
        let agent = make_agent();
        let diff = "@@ -1,1 +1,1 @@\n+let password = \"supersecret\";\n";
        let result = agent.review_diff(diff, "src/main.rs").await.unwrap();
        assert!(
            result
                .comments
                .iter()
                .any(|c| c.severity == Severity::Error)
        );
    }

    #[test]
    fn test_parse_hunk_header() {
        assert_eq!(parse_hunk_header("@@ -1,3 +5,7 @@"), Some(5));
        assert_eq!(parse_hunk_header("@@ -0,0 +1 @@ @@"), Some(1));
    }

    #[test]
    fn test_calculate_score() {
        let agent = make_agent();
        let comments = vec![
            ReviewComment {
                file_path: "f".into(),
                line: 1,
                severity: Severity::Info,
                message: "m".into(),
                suggestion: None,
            },
            ReviewComment {
                file_path: "f".into(),
                line: 2,
                severity: Severity::Warning,
                message: "m".into(),
                suggestion: None,
            },
            ReviewComment {
                file_path: "f".into(),
                line: 3,
                severity: Severity::Error,
                message: "m".into(),
                suggestion: None,
            },
        ];
        let score = agent.calculate_score(&comments);
        assert!(score < 1.0);
        assert!(score > 0.0);
    }
}
