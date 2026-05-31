#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewFinding {
    pub file_path: String,
    pub line: u32,
    pub rule: String,
    pub severity: ReviewSeverity,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestReview {
    pub findings: Vec<ReviewFinding>,
    pub summary: String,
    pub approved: bool,
    pub score: i32,
}

pub trait ReviewAgent: Send + Sync {
    fn review(&self, diff: &str, context: &ReviewContext) -> Result<PullRequestReview, String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewContext {
    pub repo_name: String,
    pub pr_number: u32,
    pub language: String,
    pub max_findings: u32,
}

pub struct StubReviewAgent;

impl StubReviewAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StubReviewAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl ReviewAgent for StubReviewAgent {
    fn review(&self, diff: &str, context: &ReviewContext) -> Result<PullRequestReview, String> {
        let line_count = diff.lines().count();
        let finding_count = (line_count / 50).min(context.max_findings as usize);
        let mut findings = Vec::new();

        for i in 0..finding_count {
            findings.push(ReviewFinding {
                file_path: "src/main.rs".into(),
                line: ((i + 1) * 50) as u32,
                rule: "STUB-001".into(),
                severity: ReviewSeverity::Info,
                message: format!("Stub review finding #{} for {}", i + 1, context.repo_name),
                suggestion: Some("Review this code carefully.".into()),
            });
        }

        Ok(PullRequestReview {
            findings,
            summary: format!(
                "Stub review of PR #{} in {}",
                context.pr_number, context.repo_name
            ),
            approved: true,
            score: 80,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context() -> ReviewContext {
        ReviewContext {
            repo_name: "test-repo".into(),
            pr_number: 42,
            language: "rust".into(),
            max_findings: 10,
        }
    }

    #[test]
    fn test_stub_review_agent_new() {
        let _agent = StubReviewAgent::new();
    }

    #[test]
    fn test_stub_review_agent_default_trait() {
        let _agent: Box<dyn ReviewAgent> = Box::new(StubReviewAgent);
        let ctx = make_context();
        let result = _agent.review("", &ctx).unwrap();
        assert!(result.findings.is_empty());
    }

    #[test]
    fn test_stub_review_agent_review() {
        let agent = StubReviewAgent::new();
        let lines: Vec<String> = (0..100).map(|i| format!("line {i}")).collect();
        let diff = lines.join("\n");
        let result = agent.review(&diff, &make_context()).unwrap();
        assert!(!result.findings.is_empty());
        assert!(result.summary.contains("PR #42"));
        assert!(result.summary.contains("test-repo"));
    }

    #[test]
    fn test_stub_review_agent_empty_diff() {
        let agent = StubReviewAgent::new();
        let result = agent.review("", &make_context()).unwrap();
        assert!(result.findings.is_empty());
    }

    #[test]
    fn test_stub_review_agent_small_diff() {
        let agent = StubReviewAgent::new();
        let diff = "@@ -1,10 +1,10 @@\njust a few lines";
        let result = agent.review(diff, &make_context()).unwrap();
        assert!(result.findings.is_empty());
    }

    #[test]
    fn test_stub_review_agent_large_diff() {
        let agent = StubReviewAgent::new();
        let lines: Vec<String> = (0..500).map(|i| format!("line {i}")).collect();
        let diff = lines.join("\n");
        let result = agent.review(&diff, &make_context()).unwrap();
        assert!(!result.findings.is_empty());
        assert!(result.findings.len() <= 10);
    }

    #[test]
    fn test_review_finding_fields() {
        let finding = ReviewFinding {
            file_path: "src/lib.rs".into(),
            line: 100,
            rule: "CLIPPY-001".into(),
            severity: ReviewSeverity::Warning,
            message: "Consider using clone_from".into(),
            suggestion: Some("Use clone_from instead".into()),
        };
        assert_eq!(finding.file_path, "src/lib.rs");
        assert_eq!(finding.line, 100);
        assert_eq!(finding.rule, "CLIPPY-001");
        assert_eq!(finding.severity, ReviewSeverity::Warning);
    }

    #[test]
    fn test_review_finding_with_suggestion() {
        let finding = ReviewFinding {
            file_path: "f".into(),
            line: 1,
            rule: "r".into(),
            severity: ReviewSeverity::Info,
            message: "m".into(),
            suggestion: Some("fix it".into()),
        };
        assert!(finding.suggestion.is_some());
        assert_eq!(finding.suggestion.unwrap(), "fix it");
    }

    #[test]
    fn test_review_finding_no_suggestion() {
        let finding = ReviewFinding {
            file_path: "f".into(),
            line: 1,
            rule: "r".into(),
            severity: ReviewSeverity::Info,
            message: "m".into(),
            suggestion: None,
        };
        assert!(finding.suggestion.is_none());
    }

    #[test]
    fn test_review_severity_all_variants() {
        let severities = [
            ReviewSeverity::Info,
            ReviewSeverity::Warning,
            ReviewSeverity::Error,
            ReviewSeverity::Critical,
        ];
        for sev in severities {
            let _ = sev;
        }
    }

    #[test]
    fn test_review_severity_equality() {
        assert_eq!(ReviewSeverity::Info, ReviewSeverity::Info);
        assert_ne!(ReviewSeverity::Info, ReviewSeverity::Critical);
    }

    #[test]
    fn test_pull_request_review_approved() {
        let agent = StubReviewAgent::new();
        let diff = "@@ -1,50 +1,50 @@\n".repeat(3);
        let result = agent.review(&diff, &make_context()).unwrap();
        assert!(result.approved);
    }

    #[test]
    fn test_pull_request_review_score() {
        let agent = StubReviewAgent::new();
        let diff = "@@ -1,50 +1,50 @@\n".repeat(3);
        let result = agent.review(&diff, &make_context()).unwrap();
        assert_eq!(result.score, 80);
    }

    #[test]
    fn test_pull_request_review_summary() {
        let agent = StubReviewAgent::new();
        let result = agent
            .review("@@ -1,100 +1,100 @@\n", &make_context())
            .unwrap();
        assert!(result.summary.contains("Stub review"));
    }

    #[test]
    fn test_review_context_fields() {
        let ctx = ReviewContext {
            repo_name: "my-org/my-repo".into(),
            pr_number: 123,
            language: "typescript".into(),
            max_findings: 5,
        };
        assert_eq!(ctx.repo_name, "my-org/my-repo");
        assert_eq!(ctx.pr_number, 123);
        assert_eq!(ctx.language, "typescript");
        assert_eq!(ctx.max_findings, 5);
    }

    #[test]
    fn test_review_finding_serialization() {
        let finding = ReviewFinding {
            file_path: "src/main.rs".into(),
            line: 42,
            rule: "SEC-001".into(),
            severity: ReviewSeverity::Error,
            message: "Hardcoded secret".into(),
            suggestion: Some("Use env var".into()),
        };
        let json = serde_json::to_string(&finding).unwrap();
        let de: ReviewFinding = serde_json::from_str(&json).unwrap();
        assert_eq!(de.file_path, "src/main.rs");
        assert_eq!(de.line, 42);
        assert_eq!(de.severity, ReviewSeverity::Error);
    }

    #[test]
    fn test_pull_request_review_serialization() {
        let review = PullRequestReview {
            findings: vec![],
            summary: "LGTM".into(),
            approved: true,
            score: 100,
        };
        let json = serde_json::to_string(&review).unwrap();
        let de: PullRequestReview = serde_json::from_str(&json).unwrap();
        assert_eq!(de.summary, "LGTM");
        assert!(de.approved);
        assert_eq!(de.score, 100);
    }

    #[test]
    fn test_review_severity_serialization() {
        let sev = ReviewSeverity::Critical;
        let json = serde_json::to_string(&sev).unwrap();
        let de: ReviewSeverity = serde_json::from_str(&json).unwrap();
        assert_eq!(sev, de);
    }

    #[test]
    fn test_review_context_serialization() {
        let ctx = make_context();
        let json = serde_json::to_string(&ctx).unwrap();
        let de: ReviewContext = serde_json::from_str(&json).unwrap();
        assert_eq!(de.repo_name, "test-repo");
        assert_eq!(de.pr_number, 42);
    }

    #[test]
    fn test_stub_agent_max_findings_limit() {
        let agent = StubReviewAgent::new();
        let ctx = ReviewContext {
            repo_name: "repo".into(),
            pr_number: 1,
            language: "rust".into(),
            max_findings: 2,
        };
        let diff = "line\n".repeat(200);
        let result = agent.review(&diff, &ctx).unwrap();
        assert!(result.findings.len() <= 2);
    }

    #[test]
    fn test_stub_agent_finding_file_path() {
        let agent = StubReviewAgent::new();
        let diff = "line\n".repeat(100);
        let result = agent.review(&diff, &make_context()).unwrap();
        for finding in &result.findings {
            assert_eq!(finding.file_path, "src/main.rs");
            assert_eq!(finding.rule, "STUB-001");
        }
    }

    #[test]
    fn test_stub_agent_finding_severity() {
        let agent = StubReviewAgent::new();
        let diff = "line\n".repeat(100);
        let result = agent.review(&diff, &make_context()).unwrap();
        for finding in &result.findings {
            assert_eq!(finding.severity, ReviewSeverity::Info);
        }
    }

    #[test]
    fn test_stub_agent_finding_line_numbers() {
        let agent = StubReviewAgent::new();
        let diff = "line\n".repeat(100);
        let result = agent.review(&diff, &make_context()).unwrap();
        for (i, finding) in result.findings.iter().enumerate() {
            assert_eq!(finding.line, ((i + 1) * 50) as u32);
        }
    }

    #[test]
    fn test_pull_request_review_empty_findings() {
        let review = PullRequestReview {
            findings: vec![],
            summary: "No issues".into(),
            approved: true,
            score: 100,
        };
        assert!(review.findings.is_empty());
        assert!(review.approved);
    }
}
