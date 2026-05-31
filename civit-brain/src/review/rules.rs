#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RulePattern {
    ExactMatch(String),
    Regex(String),
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: RuleSeverity,
    pub language: Option<String>,
    pub enabled: bool,
    pub pattern: RulePattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleViolation {
    pub rule_id: String,
    pub line: usize,
    pub severity: RuleSeverity,
    pub message: String,
    pub suggestion: Option<String>,
}

pub struct RuleEngine {
    pub rules: Vec<ReviewRule>,
    pub repo_config: HashMap<String, Vec<ReviewRule>>,
}

impl RuleEngine {
    pub fn default_rules() -> Self {
        let rules = vec![
            ReviewRule {
                id: "unwrap-usage".into(),
                name: "Unwrap Usage".into(),
                description: "Detect unwrap() calls that may panic".into(),
                severity: RuleSeverity::Warning,
                language: Some("rust".into()),
                enabled: true,
                pattern: RulePattern::ExactMatch("unwrap()".into()),
            },
            ReviewRule {
                id: "println-usage".into(),
                name: "Println Usage".into(),
                description: "Detect println!/dbg! macros in production code".into(),
                severity: RuleSeverity::Warning,
                language: Some("rust".into()),
                enabled: true,
                pattern: RulePattern::Regex(r"(println!|dbg!)".into()),
            },
            ReviewRule {
                id: "secret-detection".into(),
                name: "Secret Detection".into(),
                description: "Detect passwords, API keys, and tokens in code".into(),
                severity: RuleSeverity::Critical,
                language: None,
                enabled: true,
                pattern: RulePattern::Regex(
                    r#"(?i)(password|secret|api_key|token)\s*[:=]\s*["'][^"']+["']"#.into(),
                ),
            },
            ReviewRule {
                id: "todo-fixme".into(),
                name: "TODO/FIXME".into(),
                description: "Detect TODO, FIXME, and HACK comments".into(),
                severity: RuleSeverity::Info,
                language: None,
                enabled: true,
                pattern: RulePattern::Regex(r"(?i)\b(TODO|FIXME|HACK)\b".into()),
            },
            ReviewRule {
                id: "unsafe-usage".into(),
                name: "Unsafe Usage".into(),
                description: "Detect unsafe blocks".into(),
                severity: RuleSeverity::Error,
                language: Some("rust".into()),
                enabled: true,
                pattern: RulePattern::Regex(r"\bunsafe\b".into()),
            },
            ReviewRule {
                id: "large-function".into(),
                name: "Large Function".into(),
                description: "Functions exceeding 100 lines".into(),
                severity: RuleSeverity::Warning,
                language: Some("rust".into()),
                enabled: true,
                pattern: RulePattern::Custom("large-function".into()),
            },
            ReviewRule {
                id: "clone-heavy".into(),
                name: "Heavy Clone".into(),
                description: "Excessive clone() usage".into(),
                severity: RuleSeverity::Info,
                language: Some("rust".into()),
                enabled: true,
                pattern: RulePattern::Regex(r"\.clone\(\)".into()),
            },
            ReviewRule {
                id: "error-swallow".into(),
                name: "Error Swallowing".into(),
                description: "let _ = expr patterns that discard errors".into(),
                severity: RuleSeverity::Warning,
                language: Some("rust".into()),
                enabled: true,
                pattern: RulePattern::Regex(r"let\s+_\s*=".into()),
            },
            ReviewRule {
                id: "dead-code".into(),
                name: "Dead Code".into(),
                description: "Unused imports and variables".into(),
                severity: RuleSeverity::Info,
                language: Some("rust".into()),
                enabled: true,
                pattern: RulePattern::Regex(
                    r"#\[(?:allow|warn)\(dead_code\)\]".into(),
                ),
            },
            ReviewRule {
                id: "naming-convention".into(),
                name: "Naming Convention".into(),
                description: "Non-standard naming patterns".into(),
                severity: RuleSeverity::Info,
                language: Some("rust".into()),
                enabled: true,
                pattern: RulePattern::Regex(r"\b(fn|struct|enum|mod)\s+[A-Z][a-z]+\b".into()),
            },
        ];
        Self {
            rules,
            repo_config: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule: ReviewRule) {
        self.rules.push(rule);
    }

    pub fn remove_rule(&mut self, rule_id: &str) -> bool {
        let idx = self.rules.iter().position(|r| r.id == rule_id);
        if let Some(i) = idx {
            self.rules.remove(i);
            true
        } else {
            false
        }
    }

    pub fn get_rules_for_repo(&self, _repo_path: &str) -> Vec<&ReviewRule> {
        let repo_rules = self
            .repo_config
            .get(_repo_path)
            .map(|r| r.iter().collect::<Vec<_>>())
            .unwrap_or_default();

        let mut result = repo_rules;
        for rule in &self.rules {
            if rule.enabled
                && !result.iter().any(|r| r.id == rule.id)
                && result.iter().all(|r| r.id != rule.id)
            {
                result.push(rule);
            }
        }
        result
    }

    pub fn enable_rule(&mut self, rule_id: &str) -> bool {
        for rule in &mut self.rules {
            if rule.id == rule_id {
                rule.enabled = true;
                return true;
            }
        }
        false
    }

    pub fn disable_rule(&mut self, rule_id: &str) -> bool {
        for rule in &mut self.rules {
            if rule.id == rule_id {
                rule.enabled = false;
                return true;
            }
        }
        false
    }

    pub fn evaluate_line(&self, line: &str, file_path: &str) -> Vec<RuleViolation> {
        let mut violations = Vec::new();
        let lang = detect_language(file_path);

        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }
            if let Some(ref rule_lang) = rule.language {
                if rule_lang != &lang {
                    continue;
                }
            }
            if matches_rule(rule, line) {
                let suggestion = match rule.id.as_str() {
                    "unwrap-usage" => Some("Replace with .map_err()? or .unwrap_or_default()".into()),
                    "println-usage" => Some("Replace with tracing::info! or tracing::debug!".into()),
                    "secret-detection" => Some("Use environment variables or secrets manager.".into()),
                    "unsafe-usage" => Some("Review if unsafe is truly necessary.".into()),
                    "error-swallow" => Some("Handle the error explicitly or use #[allow(unused_must_use)].".into()),
                    "naming-convention" => Some("Use snake_case for functions and modules.".into()),
                    _ => None,
                };
                violations.push(RuleViolation {
                    rule_id: rule.id.clone(),
                    line: 0,
                    severity: rule.severity,
                    message: rule.description.clone(),
                    suggestion,
                });
            }
        }
        violations
    }
}

fn matches_rule(rule: &ReviewRule, line: &str) -> bool {
    match &rule.id {
        "unwrap-usage" => line.contains("unwrap()"),
        "println-usage" => line.contains("println!") || line.contains("dbg!"),
        "secret-detection" => {
            let lower = line.to_lowercase();
            (lower.contains("password")
                || lower.contains("secret")
                || lower.contains("api_key")
                || lower.contains("token"))
                && (line.contains(":=\"") || line.contains("='") || line.contains("=\"") || line.contains("='"))
        }
        "todo-fixme" => {
            let upper = line.to_uppercase();
            upper.contains("TODO") || upper.contains("FIXME") || upper.contains("HACK")
        }
        "unsafe-usage" => line.contains("unsafe"),
        "large-function" => false,
        "clone-heavy" => line.contains(".clone()"),
        "error-swallow" => {
            let trimmed = line.trim();
            trimmed.starts_with("let _ =")
        }
        "dead-code" => line.contains("#[allow(dead_code)]") || line.contains("#[warn(dead_code)]"),
        "naming-convention" => {
            line.contains("fn ") && {
                let parts: Vec<&str> = line.split("fn ").collect();
                parts.len() > 1 && {
                    let name_part: String = parts[1]
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    !name_part.is_empty()
                        && name_part.chars().next().map_or(false, |c| c.is_uppercase())
                        && !name_part.starts_with('_')
                }
            }
        }
        _ => match &rule.pattern {
            RulePattern::ExactMatch(s) => line.contains(s),
            RulePattern::Regex(pat) => {
                let clean = pat.replace("(?i)", "").replace("\\b", "").replace("\\s*", " ").replace("[^\"']+", "");
                let lower_text = line.to_lowercase();
                lower_text.contains(&clean.to_lowercase())
            }
            RulePattern::Custom(_) => false,
        },
    }
}

fn detect_language(file_path: &str) -> String {
    if file_path.ends_with(".rs") {
        "rust".into()
    } else if file_path.ends_with(".py") {
        "python".into()
    } else if file_path.ends_with(".js") || file_path.ends_with(".ts") {
        "javascript".into()
    } else if file_path.ends_with(".go") {
        "go".into()
    } else {
        "unknown".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_rules_count() {
        let engine = RuleEngine::default_rules();
        assert_eq!(engine.rules.len(), 10);
    }

    #[test]
    fn test_add_rule() {
        let mut engine = RuleEngine::default_rules();
        engine.add_rule(ReviewRule {
            id: "custom".into(),
            name: "Custom".into(),
            description: "A custom rule".into(),
            severity: RuleSeverity::Info,
            language: None,
            enabled: true,
            pattern: RulePattern::ExactMatch("FORBIDDEN".into()),
        });
        assert_eq!(engine.rules.len(), 11);
    }

    #[test]
    fn test_remove_rule() {
        let mut engine = RuleEngine::default_rules();
        assert!(engine.remove_rule("unwrap-usage"));
        assert_eq!(engine.rules.len(), 9);
    }

    #[test]
    fn test_remove_rule_nonexistent() {
        let mut engine = RuleEngine::default_rules();
        assert!(!engine.remove_rule("nonexistent"));
    }

    #[test]
    fn test_enable_disable_rule() {
        let mut engine = RuleEngine::default_rules();
        assert!(engine.enable_rule("unwrap-usage"));
        assert!(engine.disable_rule("unwrap-usage"));
        let rule = engine.rules.iter().find(|r| r.id == "unwrap-usage").unwrap();
        assert!(!rule.enabled);
    }

    #[test]
    fn test_evaluate_unwrap() {
        let engine = RuleEngine::default_rules();
        let violations = engine.evaluate_line("let val = data.unwrap();", "src/main.rs");
        assert!(violations.iter().any(|v| v.rule_id == "unwrap-usage"));
    }

    #[test]
    fn test_evaluate_println() {
        let engine = RuleEngine::default_rules();
        let violations = engine.evaluate_line("println!(\"hello\");", "src/main.rs");
        assert!(violations.iter().any(|v| v.rule_id == "println-usage"));
    }

    #[test]
    fn test_evaluate_secret() {
        let engine = RuleEngine::default_rules();
        let violations = engine.evaluate_line("let password = \"supersecret\";", "src/main.rs");
        assert!(violations.iter().any(|v| v.rule_id == "secret-detection"));
        assert!(violations.iter().any(|v| v.severity == RuleSeverity::Critical));
    }

    #[test]
    fn test_evaluate_todo() {
        let engine = RuleEngine::default_rules();
        let violations = engine.evaluate_line("// TODO: fix this later", "src/main.rs");
        assert!(violations.iter().any(|v| v.rule_id == "todo-fixme"));
    }

    #[test]
    fn test_evaluate_unsafe() {
        let engine = RuleEngine::default_rules();
        let violations = engine.evaluate_line("unsafe { ptr.read() }", "src/main.rs");
        assert!(violations.iter().any(|v| v.rule_id == "unsafe-usage"));
        assert!(violations.iter().any(|v| v.severity == RuleSeverity::Error));
    }

    #[test]
    fn test_evaluate_clone() {
        let engine = RuleEngine::default_rules();
        let violations = engine.evaluate_line("let x = y.clone();", "src/main.rs");
        assert!(violations.iter().any(|v| v.rule_id == "clone-heavy"));
    }

    #[test]
    fn test_evaluate_error_swallow() {
        let engine = RuleEngine::default_rules();
        let violations = engine.evaluate_line("let _ = result;", "src/main.rs");
        assert!(violations.iter().any(|v| v.rule_id == "error-swallow"));
    }

    #[test]
    fn test_evaluate_clean_line() {
        let engine = RuleEngine::default_rules();
        let violations = engine.evaluate_line("let x = 5;", "src/main.rs");
        let actionable: Vec<_> = violations
            .iter()
            .filter(|v| v.rule_id != "naming-convention" && v.rule_id != "clone-heavy")
            .collect();
        assert!(actionable.is_empty());
    }

    #[test]
    fn test_language_filter() {
        let engine = RuleEngine::default_rules();
        let violations = engine.evaluate_line("let val = data.unwrap();", "src/main.py");
        assert!(!violations.iter().any(|v| v.rule_id == "unwrap-usage"));
    }

    #[test]
    fn test_disabled_rule_skipped() {
        let mut engine = RuleEngine::default_rules();
        engine.disable_rule("unwrap-usage");
        let violations = engine.evaluate_line("data.unwrap()", "src/main.rs");
        assert!(!violations.iter().any(|v| v.rule_id == "unwrap-usage"));
    }

    #[test]
    fn test_secret_case_insensitive() {
        let engine = RuleEngine::default_rules();
        let violations = engine.evaluate_line("let PASSWORD = \"abc\"", "src/main.rs");
        assert!(violations.iter().any(|v| v.rule_id == "secret-detection"));
    }

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language("src/main.rs"), "rust");
        assert_eq!(detect_language("app.py"), "python");
        assert_eq!(detect_language("index.js"), "javascript");
        assert_eq!(detect_language("main.go"), "go");
        assert_eq!(detect_language("data.txt"), "unknown");
    }

    #[test]
    fn test_rule_serialization() {
        let rule = ReviewRule {
            id: "test".into(),
            name: "Test Rule".into(),
            description: "A test".into(),
            severity: RuleSeverity::Warning,
            language: Some("rust".into()),
            enabled: true,
            pattern: RulePattern::ExactMatch("foo".into()),
        };
        let json = serde_json::to_string(&rule).unwrap();
        let de: ReviewRule = serde_json::from_str(&json).unwrap();
        assert_eq!(de.id, "test");
    }

    #[test]
    fn test_get_rules_for_repo_empty() {
        let engine = RuleEngine::default_rules();
        let rules = engine.get_rules_for_repo("/nonexistent");
        assert_eq!(rules.len(), 10);
    }
}
