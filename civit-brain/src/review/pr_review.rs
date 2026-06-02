#![forbid(unsafe_code)]

use crate::review::analyzer::DiffAnalyzer;
use crate::review::rules::RuleEngine;
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

// ---------------------------------------------------------------------------
// DiffAnalyzerReviewAgent — real static analysis bridged to ReviewAgent trait
// ---------------------------------------------------------------------------

/// Production review agent backed by [`DiffAnalyzer`].
///
/// Performs real static analysis: security (SQL injection, command injection,
/// path traversal, SSRF), lint rules (unwrap, println, secrets, todo, unsafe,
/// clone-heavy, error-swallow, dead-code, naming, large-function), performance
/// suggestions (excessive allocations, nested loops, lock scope).
pub struct DiffAnalyzerReviewAgent {
    analyzer: DiffAnalyzer,
}

impl DiffAnalyzerReviewAgent {
    /// Create a new agent with the given [`RuleEngine`].
    pub fn new(rule_engine: RuleEngine) -> Self {
        Self {
            analyzer: DiffAnalyzer::new(rule_engine),
        }
    }

    /// Create a new agent with the default built-in rule set.
    pub fn with_default_rules() -> Self {
        Self::new(RuleEngine::default_rules())
    }
}

impl Default for DiffAnalyzerReviewAgent {
    fn default() -> Self {
        Self::with_default_rules()
    }
}

impl ReviewAgent for DiffAnalyzerReviewAgent {
    fn review(&self, diff: &str, context: &ReviewContext) -> Result<PullRequestReview, String> {
        let mut all_findings = Vec::new();

        // Split unified diff into per-file hunks.
        // A unified diff file chunk starts with "--- a/path" followed by "+++ b/path".
        let file_chunks = split_diff_by_file(diff);

        for (file_path, file_diff) in file_chunks {
            let result = self.analyzer.analyze_diff(&file_diff, &file_path);

            // Map RuleViolation → ReviewFinding
            for v in &result.violations {
                all_findings.push(ReviewFinding {
                    file_path: file_path.clone(),
                    line: v.line as u32,
                    rule: v.rule_id.clone(),
                    severity: map_severity(v.severity),
                    message: v.message.clone(),
                    suggestion: v.suggestion.clone(),
                });
            }

            // Map CodeSuggestion → ReviewFinding (Warning level)
            for s in &result.suggestions {
                all_findings.push(ReviewFinding {
                    file_path: file_path.clone(),
                    line: s.line as u32,
                    rule: "perf-suggestion".into(),
                    severity: ReviewSeverity::Warning,
                    message: s.description.clone(),
                    suggestion: Some(s.replacement.clone()),
                });
            }
        }

        // Sort by severity (Critical > Error > Warning > Info) then by line.
        all_findings.sort_by(
            |a, b| match (severity_rank(b.severity), severity_rank(a.severity)) {
                (rb, ra) if rb != ra => rb.cmp(&ra),
                _ => a.line.cmp(&b.line),
            },
        );

        // Cap findings at max_findings, keeping highest severity.
        let capped: Vec<ReviewFinding> = all_findings
            .into_iter()
            .take(context.max_findings as usize)
            .collect();

        // Compute approval and score.
        let (approved, score) = compute_verdict(&capped);

        let summary = build_summary(context, &capped, approved, score);

        Ok(PullRequestReview {
            findings: capped,
            summary,
            approved,
            score,
        })
    }
}

/// Severity rank for sorting (higher = more severe).
fn severity_rank(sev: ReviewSeverity) -> u8 {
    match sev {
        ReviewSeverity::Critical => 4,
        ReviewSeverity::Error => 3,
        ReviewSeverity::Warning => 2,
        ReviewSeverity::Info => 1,
    }
}

/// Map analyzer `RuleSeverity` → `ReviewSeverity`. The enums are identical.
fn map_severity(sev: crate::review::rules::RuleSeverity) -> ReviewSeverity {
    match sev {
        crate::review::rules::RuleSeverity::Critical => ReviewSeverity::Critical,
        crate::review::rules::RuleSeverity::Error => ReviewSeverity::Error,
        crate::review::rules::RuleSeverity::Warning => ReviewSeverity::Warning,
        crate::review::rules::RuleSeverity::Info => ReviewSeverity::Info,
    }
}

/// Compute approval decision and score from findings.
fn compute_verdict(findings: &[ReviewFinding]) -> (bool, i32) {
    if findings.is_empty() {
        return (true, 100);
    }

    let critical = findings
        .iter()
        .filter(|f| f.severity == ReviewSeverity::Critical)
        .count();
    let errors = findings
        .iter()
        .filter(|f| f.severity == ReviewSeverity::Error)
        .count();
    let warnings = findings
        .iter()
        .filter(|f| f.severity == ReviewSeverity::Warning)
        .count();
    let infos = findings
        .iter()
        .filter(|f| f.severity == ReviewSeverity::Info)
        .count();

    let approved;
    let score;

    if critical > 0 {
        approved = false;
        // Each Critical: -25
        score = 40_i32.saturating_sub((critical as i32) * 25);
    } else if errors > 0 {
        approved = false;
        // Each Error: -15
        score = 70_i32.saturating_sub((errors as i32) * 15);
    } else if warnings > 0 {
        // Warnings only: approve with reduced score
        approved = true;
        score = 95_i32.saturating_sub((warnings as i32) * 5);
    } else {
        // Info only: approve with minor reduction
        approved = true;
        score = 98_i32.saturating_sub((infos as i32) * 2);
    }

    (approved, score.max(0))
}

/// Build a human-readable review summary.
fn build_summary(
    context: &ReviewContext,
    findings: &[ReviewFinding],
    approved: bool,
    score: i32,
) -> String {
    let critical = findings
        .iter()
        .filter(|f| f.severity == ReviewSeverity::Critical)
        .count();
    let errors = findings
        .iter()
        .filter(|f| f.severity == ReviewSeverity::Error)
        .count();
    let warnings = findings
        .iter()
        .filter(|f| f.severity == ReviewSeverity::Warning)
        .count();
    let infos = findings
        .iter()
        .filter(|f| f.severity == ReviewSeverity::Info)
        .count();

    let status = if approved {
        "APPROVED"
    } else {
        "CHANGES_REQUESTED"
    };

    format!(
        "Static analysis review of PR #{} in {} — {} (score: {}). \
         {} critical, {} errors, {} warnings, {} info findings.",
        context.pr_number, context.repo_name, status, score, critical, errors, warnings, infos,
    )
}

/// Split a unified diff into (file_path, file_diff) pairs.
///
/// Handles diffs with explicit `--- a/...` / `+++ b/...` headers. If no file
/// headers are found, the entire diff is attributed to a single unknown file.
fn split_diff_by_file(diff: &str) -> Vec<(String, String)> {
    let mut chunks: Vec<(String, String)> = Vec::new();
    let lines: Vec<&str> = diff.lines().collect();
    let mut idx = 0;

    while idx < lines.len() {
        // Look for "--- a/..." line
        let file_path = loop {
            if idx >= lines.len() {
                break None;
            }
            let line = lines[idx];
            idx += 1;
            if line.starts_with("--- ") {
                let path = line
                    .trim_start_matches("--- a/")
                    .trim_start_matches("--- b/")
                    .trim_start_matches("--- ");
                let path = path.strip_prefix('/').unwrap_or(path);
                break Some(path.to_string());
            }
        };

        let file_path = match file_path {
            Some(p) => p,
            None => {
                // No more "---" headers; check if we already found chunks
                if chunks.is_empty() && !diff.is_empty() {
                    chunks.push(("(unknown)".to_string(), diff.to_string()));
                }
                return chunks;
            }
        };

        // Consume the matching "+++ b/..." line
        if idx < lines.len() && lines[idx].starts_with("+++ ") {
            idx += 1;
        }

        // Collect all lines until the next "--- " or end of input
        let mut file_diff = String::new();
        while idx < lines.len() {
            if lines[idx].starts_with("--- ") {
                break;
            }
            file_diff.push_str(lines[idx]);
            file_diff.push('\n');
            idx += 1;
        }

        chunks.push((file_path, file_diff));
    }

    chunks
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

    // ===================================================================
    // DiffAnalyzerReviewAgent tests
    // ===================================================================

    fn make_clean_context() -> ReviewContext {
        ReviewContext {
            repo_name: "test-repo".into(),
            pr_number: 1,
            language: "rust".into(),
            max_findings: 20,
        }
    }

    fn clean_diff() -> &'static str {
        "--- a/src/main.rs\n\
         +++ b/src/main.rs\n\
         @@ -1,3 +1,3 @@\n\
         fn main() {\n\
        -    println!(\"hello\");\n\
        +    println!(\"world\");\n\
         }\n"
    }

    #[test]
    fn test_diff_analyzer_agent_clean_diff_approved() {
        let agent = DiffAnalyzerReviewAgent::with_default_rules();
        // Clean diff has only "println!" which is a Warning-level lint
        let result = agent.review(clean_diff(), &make_clean_context()).unwrap();
        assert!(result.approved);
        assert!(result.score > 80);
        assert!(result.summary.contains("PR #1"));
        assert!(result.summary.contains("test-repo"));
    }

    #[test]
    fn test_diff_analyzer_agent_empty_diff() {
        let agent = DiffAnalyzerReviewAgent::with_default_rules();
        let result = agent.review("", &make_clean_context()).unwrap();
        assert!(result.findings.is_empty());
        assert!(result.approved);
        assert_eq!(result.score, 100);
    }

    #[test]
    fn test_diff_analyzer_agent_unwrap_detection() {
        let diff = "--- a/src/lib.rs\n\
                     +++ b/src/lib.rs\n\
                     @@ -1,3 +1,3 @@\n\
                     fn foo() -> Option<i32> {\n\
                     -    None\n\
                     +    some_result.unwrap()\n\
                     }\n";
        let agent = DiffAnalyzerReviewAgent::with_default_rules();
        let result = agent.review(diff, &make_clean_context()).unwrap();
        let unwrap_finding = result.findings.iter().find(|f| f.rule == "unwrap-usage");
        assert!(unwrap_finding.is_some());
        assert_eq!(unwrap_finding.unwrap().severity, ReviewSeverity::Warning);
    }

    #[test]
    fn test_diff_analyzer_agent_sql_injection_rejection() {
        let diff = "--- a/src/danger.rs\n\
                     +++ b/src/danger.rs\n\
                     @@ -1,3 +1,3 @@\n\
                     fn read() {\n\
                     -    let p = \"/safe/path\";\n\
                     +    let p = format!(\"SELECT * FROM users WHERE name = '{}'\", name);\n\
                     }\n";
        let agent = DiffAnalyzerReviewAgent::with_default_rules();
        let result = agent.review(diff, &make_clean_context()).unwrap();
        let sql_finding = result.findings.iter().find(|f| f.rule == "sql-injection");
        assert!(sql_finding.is_some());
        assert_eq!(sql_finding.unwrap().severity, ReviewSeverity::Critical);
        assert!(!result.approved);
        assert!(result.score < 40);
    }

    #[test]
    fn test_diff_analyzer_agent_command_injection() {
        let diff = "--- a/src/cmd.rs\n\
                     +++ b/src/cmd.rs\n\
                     @@ -1,3 +1,4 @@\n\
                     fn run(input: &str) {\n\
                     +    std::process::Command::new(\"sh\").arg(\"-c\").arg(format!(\"echo {}\", input))\n\
                     }\n";
        let agent = DiffAnalyzerReviewAgent::with_default_rules();
        let result = agent.review(diff, &make_clean_context()).unwrap();
        let cmd_finding = result
            .findings
            .iter()
            .find(|f| f.rule == "command-injection");
        assert!(cmd_finding.is_some());
        assert_eq!(cmd_finding.unwrap().severity, ReviewSeverity::Critical);
        assert!(!result.approved);
    }

    #[test]
    fn test_diff_analyzer_agent_secret_detection() {
        let diff = "--- a/src/config.rs\n\
                     +++ b/src/config.rs\n\
                     @@ -1,3 +1,4 @@\n\
                     fn load() {\n\
                     +    let password=\"supersecret\";\n\
                     }\n";
        let agent = DiffAnalyzerReviewAgent::with_default_rules();
        let result = agent.review(diff, &make_clean_context()).unwrap();
        let secret_finding = result
            .findings
            .iter()
            .find(|f| f.rule == "secret-detection");
        assert!(secret_finding.is_some());
        assert_eq!(secret_finding.unwrap().severity, ReviewSeverity::Critical);
        assert!(!result.approved);
    }

    #[test]
    fn test_diff_analyzer_agent_path_traversal() {
        // Path traversal needs ../ in the added line
        let diff = "--- a/src/fs.rs\n\
                     +++ b/src/fs.rs\n\
                     @@ -1,3 +1,3 @@\n\
                     fn read() {\n\
                     -    let p = \"/safe/path\";\n\
                     +    let p = format!(\"/data/{}\", \"../etc/passwd\");\n\
                     }\n";
        let agent = DiffAnalyzerReviewAgent::with_default_rules();
        let result = agent.review(diff, &make_clean_context()).unwrap();
        let pt_finding = result.findings.iter().find(|f| f.rule == "path-traversal");
        assert!(pt_finding.is_some());
        assert_eq!(pt_finding.unwrap().severity, ReviewSeverity::Error);
    }

    #[test]
    fn test_diff_analyzer_agent_ssrf_detection() {
        let diff = "--- a/src/http.rs\n\
                     +++ b/src/http.rs\n\
                     @@ -1,3 +1,4 @@\n\
                     fn fetch(user_url: &str) {\n\
                     +    reqwest::get(format!(\"https://{}\", user_url))\n\
                     }\n";
        let agent = DiffAnalyzerReviewAgent::with_default_rules();
        let result = agent.review(diff, &make_clean_context()).unwrap();
        let ssrf_finding = result.findings.iter().find(|f| f.rule == "ssrf");
        assert!(ssrf_finding.is_some());
        assert_eq!(ssrf_finding.unwrap().severity, ReviewSeverity::Critical);
        assert!(!result.approved);
    }

    #[test]
    fn test_diff_analyzer_agent_multi_file() {
        let diff = "--- a/src/clean.rs\n\
                     +++ b/src/clean.rs\n\
                     @@ -1,2 +1,2 @@\n\
                     fn clean() {\n\
                     -    let x = 1;\n\
                     +    let x = 2;\n\
                     }\n\
                     --- a/src/danger.rs\n\
                     +++ b/src/danger.rs\n\
                     @@ -1,3 +1,4 @@\n\
                     fn danger() {\n\
                     +    format!(\"SELECT * FROM users WHERE id = {}\", input)\n\
                     }\n";
        let agent = DiffAnalyzerReviewAgent::with_default_rules();
        let result = agent.review(diff, &make_clean_context()).unwrap();
        let files: std::collections::HashSet<&str> = result
            .findings
            .iter()
            .map(|f| f.file_path.as_str())
            .collect();
        assert!(files.contains("src/clean.rs") || files.contains("src/danger.rs"));
        // The dangerous file should have a finding
        let danger_finding = result
            .findings
            .iter()
            .find(|f| f.file_path == "src/danger.rs");
        assert!(danger_finding.is_some());
    }

    #[test]
    fn test_diff_analyzer_agent_max_findings_cap() {
        let diff = "--- a/src/lib.rs\n\
                     +++ b/src/lib.rs\n\
                     @@ -1,100 +1,100 @@\n";
        let ctx = ReviewContext {
            repo_name: "repo".into(),
            pr_number: 1,
            language: "rust".into(),
            max_findings: 3,
        };
        let agent = DiffAnalyzerReviewAgent::with_default_rules();
        let result = agent.review(diff, &ctx).unwrap();
        assert!(result.findings.len() <= 3);
    }

    #[test]
    fn test_diff_analyzer_agent_default_trait() {
        let agent: Box<dyn ReviewAgent> = Box::new(DiffAnalyzerReviewAgent::with_default_rules());
        let result = agent.review("", &make_clean_context()).unwrap();
        assert!(result.findings.is_empty());
        assert_eq!(result.score, 100);
    }

    #[test]
    fn test_diff_analyzer_agent_summary_format() {
        let agent = DiffAnalyzerReviewAgent::with_default_rules();
        let result = agent.review(clean_diff(), &make_clean_context()).unwrap();
        assert!(result.summary.contains("Static analysis review"));
        assert!(result.summary.contains("score:"));
    }

    #[test]
    fn test_diff_analyzer_agent_findings_sorted_by_severity() {
        let diff = "--- a/src/lib.rs\n\
                     +++ b/src/lib.rs\n\
                     @@ -1,5 +1,8 @@\n\
                     fn foo() {\n\
                     -    let x = 1;\n\
                     +    let api_key = \"sk-live-abc123\";\n\
                     +    let result = some_val.unwrap();\n\
                     +    format!(\"SELECT * FROM t WHERE id = {}\", id);\n\
                     +    println!(\"debug\");\n\
                     }\n";
        let agent = DiffAnalyzerReviewAgent::with_default_rules();
        let result = agent.review(diff, &make_clean_context()).unwrap();
        // Critical findings should appear before Warning/Info
        let mut prev_rank = u8::MAX;
        for finding in &result.findings {
            let rank = severity_rank(finding.severity);
            assert!(
                rank <= prev_rank,
                "findings not sorted: rank {rank} > prev {prev_rank}"
            );
            prev_rank = rank;
        }
    }

    #[test]
    fn test_diff_analyzer_agent_performance_suggestion() {
        let diff = "--- a/src/alloc.rs\n\
                     +++ b/src/alloc.rs\n\
                     @@ -1,3 +1,4 @@\n\
                     fn heavy() {\n\
                     -    let x = 1;\n\
                     +    let a = x.to_string(); let b = y.to_string(); let c = z.to_string(); let d = w.to_string();\n\
                     }\n";
        let agent = DiffAnalyzerReviewAgent::with_default_rules();
        let result = agent.review(diff, &make_clean_context()).unwrap();
        let perf = result.findings.iter().find(|f| f.rule == "perf-suggestion");
        assert!(perf.is_some());
        assert_eq!(perf.unwrap().severity, ReviewSeverity::Warning);
        assert!(perf.unwrap().suggestion.is_some());
    }

    #[test]
    fn test_compute_verdict_clean() {
        let (approved, score) = compute_verdict(&[]);
        assert!(approved);
        assert_eq!(score, 100);
    }

    #[test]
    fn test_compute_verdict_critical() {
        let findings = vec![ReviewFinding {
            file_path: "f".into(),
            line: 1,
            rule: "r".into(),
            severity: ReviewSeverity::Critical,
            message: "m".into(),
            suggestion: None,
        }];
        let (approved, score) = compute_verdict(&findings);
        assert!(!approved);
        assert!(score <= 40);
    }

    #[test]
    fn test_compute_verdict_error() {
        let findings = vec![ReviewFinding {
            file_path: "f".into(),
            line: 1,
            rule: "r".into(),
            severity: ReviewSeverity::Error,
            message: "m".into(),
            suggestion: None,
        }];
        let (approved, score) = compute_verdict(&findings);
        assert!(!approved);
        assert!(score < 70);
    }

    #[test]
    fn test_compute_verdict_warnings_only() {
        let findings = vec![
            ReviewFinding {
                file_path: "f".into(),
                line: 1,
                rule: "r".into(),
                severity: ReviewSeverity::Warning,
                message: "m".into(),
                suggestion: None,
            },
            ReviewFinding {
                file_path: "f".into(),
                line: 2,
                rule: "r".into(),
                severity: ReviewSeverity::Warning,
                message: "m".into(),
                suggestion: None,
            },
        ];
        let (approved, score) = compute_verdict(&findings);
        assert!(approved);
        assert!(score < 95);
    }

    #[test]
    fn test_compute_verdict_info_only() {
        let findings = vec![ReviewFinding {
            file_path: "f".into(),
            line: 1,
            rule: "r".into(),
            severity: ReviewSeverity::Info,
            message: "m".into(),
            suggestion: None,
        }];
        let (approved, score) = compute_verdict(&findings);
        assert!(approved);
        assert!(score >= 95);
    }

    #[test]
    fn test_split_diff_by_file_multi() {
        let diff = "--- a/file1.rs\n\
                     +++ b/file1.rs\n\
                     @@ -1,1 +1,1 @@\n\
                     -old\n\
                     +new\n\
                     --- a/file2.rs\n\
                     +++ b/file2.rs\n\
                     @@ -1,1 +1,1 @@\n\
                     -a\n\
                     +b\n";
        let chunks = split_diff_by_file(diff);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].0, "file1.rs");
        assert_eq!(chunks[1].0, "file2.rs");
    }

    #[test]
    fn test_split_diff_by_file_no_headers() {
        let diff = "@@ -1,3 +1,3 @@\n-old\n+new\n";
        let chunks = split_diff_by_file(diff);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].0, "(unknown)");
    }

    #[test]
    fn test_split_diff_by_file_empty() {
        let chunks = split_diff_by_file("");
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_split_diff_by_file_with_timestamp() {
        let diff = "--- a/lib.rs\t2026-01-01 00:00:00.000000000 +0000\n\
                     +++ b/lib.rs\t2026-01-01 00:00:00.000000000 +0000\n\
                     @@ -1,1 +1,1 @@\n\
                     -a\n\
                     +b\n";
        let chunks = split_diff_by_file(diff);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].0, "lib.rs\t2026-01-01 00:00:00.000000000 +0000");
    }
}
