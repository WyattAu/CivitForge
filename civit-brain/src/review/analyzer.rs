#![forbid(unsafe_code)]

use crate::review::rules::{RuleEngine, RuleSeverity, RuleViolation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSuggestion {
    pub line: usize,
    pub original: String,
    pub replacement: String,
    pub description: String,
    pub auto_applicable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub file_path: String,
    pub violations: Vec<RuleViolation>,
    pub added_lines: usize,
    pub removed_lines: usize,
    pub complexity_score: f32,
    pub suggestions: Vec<CodeSuggestion>,
}

pub struct DiffAnalyzer {
    pub rule_engine: RuleEngine,
}

impl DiffAnalyzer {
    pub fn new(rule_engine: RuleEngine) -> Self {
        Self { rule_engine }
    }

    pub fn analyze_diff(&self, diff: &str, file_path: &str) -> AnalysisResult {
        let mut violations = Vec::new();
        let mut added_lines = 0usize;
        let mut removed_lines = 0usize;
        let mut suggestions = Vec::new();
        let mut line_num = 0usize;

        for raw_line in diff.lines() {
            if raw_line.starts_with("@@") {
                if let Some(hunk_start) = parse_hunk_start(raw_line) {
                    line_num = hunk_start;
                }
                continue;
            }
            if raw_line.starts_with("+++") || raw_line.starts_with("---") {
                continue;
            }
            if raw_line.starts_with('+') {
                let code = &raw_line[1..];
                added_lines += 1;
                line_num += 1;

                let line_violations = self.rule_engine.evaluate_line(code, file_path);
                for mut v in line_violations {
                    v.line = line_num;
                    violations.push(v);
                }

                self.check_security(code, line_num, &mut violations);
                self.check_performance(code, line_num, &mut suggestions);
            } else if raw_line.starts_with('-') {
                removed_lines += 1;
            } else if !raw_line.starts_with('\\') {
                line_num += 1;
            }
        }

        let complexity_score = self.estimate_complexity(diff);

        AnalysisResult {
            file_path: file_path.into(),
            violations,
            added_lines,
            removed_lines,
            complexity_score,
            suggestions,
        }
    }

    pub fn analyze_file(&self, source: &str, file_path: &str) -> AnalysisResult {
        let mut violations = Vec::new();
        let mut suggestions = Vec::new();

        for (i, line) in source.lines().enumerate() {
            let line_num = i + 1;
            let line_violations = self.rule_engine.evaluate_line(line, file_path);
            for mut v in line_violations {
                v.line = line_num;
                violations.push(v);
            }
            self.check_security(line, line_num, &mut violations);
            self.check_performance(line, line_num, &mut suggestions);
        }

        let total_lines = source.lines().count();
        let complexity_score = self.estimate_complexity(source);

        AnalysisResult {
            file_path: file_path.into(),
            violations,
            added_lines: total_lines,
            removed_lines: 0,
            complexity_score,
            suggestions,
        }
    }

    fn estimate_complexity(&self, source: &str) -> f32 {
        let branches: usize = source
            .lines()
            .map(|l| {
                l.matches("if ")
                    .count()
                    + l.matches("match ")
                    .count()
                    + l.matches("else")
                    .count()
                    + l.matches("for ")
                    .count()
                    + l.matches("while ")
                    .count()
                    + l.matches("&&")
                    .count()
                    + l.matches("||")
                    .count()
            })
            .sum();

        let nesting = source
            .lines()
            .filter(|l| l.trim_start().starts_with('{') || l.contains('{'))
            .count();

        let lines = source.lines().count().max(1);
        let raw_score = (branches as f32 + nesting as f32 * 0.5) / lines as f32;
        raw_score.min(1.0)
    }

    fn check_security(&self, line: &str, line_num: usize, violations: &mut Vec<RuleViolation>) {
        let sql_injection_patterns = ["format!(\"SELECT", "format!(\"INSERT", "format!(\"UPDATE", "format!(\"DELETE"];
        for pat in &sql_injection_patterns {
            if line.contains(pat) {
                violations.push(RuleViolation {
                    rule_id: "sql-injection".into(),
                    line: line_num,
                    severity: RuleSeverity::Critical,
                    message: "Potential SQL injection via string interpolation".into(),
                    suggestion: Some("Use parameterized queries.".into()),
                });
                return;
            }
        }

        if line.contains("Command::new") || line.contains("std::process::Command") {
            if line.contains("format!") || line.contains("+") {
                violations.push(RuleViolation {
                    rule_id: "command-injection".into(),
                    line: line_num,
                    severity: RuleSeverity::Critical,
                    message: "Potential command injection via string interpolation".into(),
                    suggestion: Some("Use argument vectors instead of string concatenation.".into()),
                });
                return;
            }
        }

        if line.contains("../") || line.contains("..\\") {
            violations.push(RuleViolation {
                rule_id: "path-traversal".into(),
                line: line_num,
                severity: RuleSeverity::Error,
                message: "Potential path traversal vulnerability".into(),
                suggestion: Some("Validate and sanitize file paths.".into()),
            });
            return;
        }

        let ssrf_patterns = ["reqwest::get", "HttpClient::new", "ureq::get"];
        for pat in &ssrf_patterns {
            if line.contains(pat) && (line.contains("format!") || line.contains("+")) {
                violations.push(RuleViolation {
                    rule_id: "ssrf".into(),
                    line: line_num,
                    severity: RuleSeverity::Critical,
                    message: "Potential SSRF via user-controlled URL".into(),
                    suggestion: Some("Validate and allowlist URLs.".into()),
                });
                return;
            }
        }
    }

    fn check_performance(&self, line: &str, line_num: usize, suggestions: &mut Vec<CodeSuggestion>) {
        if line.contains(".to_string()")
            || line.contains("String::from")
            || line.contains("format!(")
        {
            let count = line.matches(".to_string()").count()
                + line.matches("String::from").count()
                + line.matches("format!(").count();
            if count > 2 {
                suggestions.push(CodeSuggestion {
                    line: line_num,
                    original: line.to_owned(),
                    replacement: "Consider reducing allocations".into(),
                    description: "Multiple string allocations on a single line".into(),
                    auto_applicable: false,
                });
            }
        }

        let nested_loops = ["for"]
            .iter()
            .filter(|p| line.contains(**p))
            .count();
        if nested_loops > 1 && line.contains("for") {
            suggestions.push(CodeSuggestion {
                line: line_num,
                original: line.to_owned(),
                replacement: "Consider using a more efficient algorithm or data structure".into(),
                description: "Nested loop detected; potential O(n^2) pattern".into(),
                auto_applicable: false,
            });
        }

        if line.contains(".lock()")
            || line.contains(".write()")
            || line.contains(".read()")
        {
            suggestions.push(CodeSuggestion {
                line: line_num,
                original: line.to_owned(),
                replacement: "Consider lock duration minimization".into(),
                description: "Lock acquisition detected; ensure scope is minimal".into(),
                auto_applicable: false,
            });
        }
    }
}

fn parse_hunk_start(hunk_line: &str) -> Option<usize> {
    let parts: Vec<&str> = hunk_line.split('+').collect();
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

    fn make_analyzer() -> DiffAnalyzer {
        DiffAnalyzer::new(RuleEngine::default_rules())
    }

    #[test]
    fn test_analyze_clean_diff() {
        let analyzer = make_analyzer();
        let diff = "@@ -1,3 +1,3 @@\n let x = 5;\n let y = 10;\n";
        let result = analyzer.analyze_diff(diff, "src/main.rs");
        assert!(result.violations.is_empty());
        assert_eq!(result.added_lines, 2);
    }

    #[test]
    fn test_analyze_unwrap_in_diff() {
        let analyzer = make_analyzer();
        let diff = "@@ -1,1 +1,1 @@\n+let val = data.unwrap();\n";
        let result = analyzer.analyze_diff(diff, "src/main.rs");
        assert!(result.violations.iter().any(|v| v.rule_id == "unwrap-usage"));
    }

    #[test]
    fn test_analyze_sql_injection() {
        let analyzer = make_analyzer();
        let diff = "@@ -1,1 +1,1 @@\n+let q = format!(\"SELECT * FROM users WHERE id = {}\", id);\n";
        let result = analyzer.analyze_diff(diff, "src/main.rs");
        assert!(result.violations.iter().any(|v| v.rule_id == "sql-injection"));
        assert!(result
            .violations
            .iter()
            .any(|v| v.severity == RuleSeverity::Critical));
    }

    #[test]
    fn test_analyze_command_injection() {
        let analyzer = make_analyzer();
        let diff = "@@ -1,1 +1,1 @@\n+Command::new(format!(\"rm {}\", path));\n";
        let result = analyzer.analyze_diff(diff, "src/main.rs");
        assert!(result
            .violations
            .iter()
            .any(|v| v.rule_id == "command-injection"));
    }

    #[test]
    fn test_analyze_path_traversal() {
        let analyzer = make_analyzer();
        let diff = "@@ -1,1 +1,1 @@\n+let path = format!(\"../{}\", user_input);\n";
        let result = analyzer.analyze_diff(diff, "src/main.rs");
        assert!(result.violations.iter().any(|v| v.rule_id == "path-traversal"));
    }

    #[test]
    fn test_analyze_println() {
        let analyzer = make_analyzer();
        let diff = "@@ -1,1 +1,1 @@\n+println!(\"debugging\");\n";
        let result = analyzer.analyze_diff(diff, "src/main.rs");
        assert!(result.violations.iter().any(|v| v.rule_id == "println-usage"));
    }

    #[test]
    fn test_analyze_file() {
        let analyzer = make_analyzer();
        let source = "fn main() {\n    let x = data.unwrap();\n    println!(\"hi\");\n}\n";
        let result = analyzer.analyze_file(source, "src/main.rs");
        assert!(result.violations.iter().any(|v| v.rule_id == "unwrap-usage"));
        assert!(result.violations.iter().any(|v| v.rule_id == "println-usage"));
        assert_eq!(result.added_lines, 4);
    }

    #[test]
    fn test_complexity_score_range() {
        let analyzer = make_analyzer();
        let diff = "@@ -1,1 +1,3 @@\n+if x { a }\n+else { b }\n+match y { _ => {} }\n";
        let result = analyzer.analyze_diff(diff, "src/main.rs");
        assert!(result.complexity_score >= 0.0);
        assert!(result.complexity_score <= 1.0);
    }

    #[test]
    fn test_removed_lines_count() {
        let analyzer = make_analyzer();
        let diff = "@@ -1,3 +1,0 @@\n-let a = 1;\n-let b = 2;\n-let c = 3;\n";
        let result = analyzer.analyze_diff(diff, "src/main.rs");
        assert_eq!(result.removed_lines, 3);
    }

    #[test]
    fn test_suggestions_generated() {
        let analyzer = make_analyzer();
        let diff = "@@ -1,1 +1,1 @@\n+let x = format!(\"{}\", a).to_string();\n";
        let result = analyzer.analyze_diff(diff, "src/main.rs");
        assert_eq!(result.file_path, "src/main.rs");
        assert_eq!(result.added_lines, 1);
    }

    #[test]
    fn test_parse_hunk_start() {
        assert_eq!(parse_hunk_start("@@ -1,3 +5,7 @@"), Some(5));
        assert_eq!(parse_hunk_start("@@ -0,0 +1 @@"), Some(1));
    }

    #[test]
    fn test_analyze_file_with_lock_contention() {
        let analyzer = make_analyzer();
        let source = "fn main() {\n    let data = map.lock();\n}\n";
        let result = analyzer.analyze_file(source, "src/main.rs");
        assert!(!result.suggestions.is_empty());
    }
}
