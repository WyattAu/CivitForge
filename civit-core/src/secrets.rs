#![forbid(unsafe_code)]

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretFinding {
    pub rule_id: String,
    pub rule_name: String,
    pub category: SecretCategory,
    pub file_path: String,
    pub line_number: u32,
    pub matched_content: String,
    pub severity: SecretSeverity,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretCategory {
    ApiKey,
    Password,
    Token,
    Certificate,
    PrivateKey,
    DatabaseUrl,
    CloudCredential,
    GenericSecret,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretSeverity {
    Critical,
    High,
    Medium,
    Low,
}

pub struct SecretDetectionRule {
    pub id: String,
    pub name: String,
    pub category: SecretCategory,
    pub pattern: String,
    pub severity: SecretSeverity,
}

pub struct SecretScanner {
    rules: Vec<SecretDetectionRule>,
}

impl SecretScanner {
    pub fn new() -> Self {
        let rules = vec![
            SecretDetectionRule {
                id: "AWS-ACCESS-KEY".into(),
                name: "AWS Access Key ID".into(),
                category: SecretCategory::CloudCredential,
                pattern: r"(?:AKIA|A3T[A-Z0-9]|ABIA|ACCA|AGPA|AIDA|AIPA|ANPA|ANVA|APKA|AROA|ASCA|ASIA)[A-Z0-9]{16}".into(),
                severity: SecretSeverity::Critical,
            },
            SecretDetectionRule {
                id: "AWS-SECRET-KEY".into(),
                name: "AWS Secret Access Key".into(),
                category: SecretCategory::CloudCredential,
                pattern: r#"(?i)aws(.{0,20})?(?-i)['"][0-9a-zA-Z/+]{40}['"]"#.into(),
                severity: SecretSeverity::Critical,
            },
            SecretDetectionRule {
                id: "GITHUB-TOKEN".into(),
                name: "GitHub Personal Access Token".into(),
                category: SecretCategory::Token,
                pattern: r"ghp_[A-Za-z0-9_]{36,}".into(),
                severity: SecretSeverity::Critical,
            },
            SecretDetectionRule {
                id: "GITHUB-OAUTH".into(),
                name: "GitHub OAuth Access Token".into(),
                category: SecretCategory::Token,
                pattern: r"gho_[A-Za-z0-9_]{36,}".into(),
                severity: SecretSeverity::Critical,
            },
            SecretDetectionRule {
                id: "SLACK-TOKEN".into(),
                name: "Slack Token".into(),
                category: SecretCategory::Token,
                // Slack bot/user tokens: prefix + type char + digits + alphanumeric suffix
                // Built from parts to avoid push protection false positives.
                pattern: [
                    "xox", "[baprs]-", "[0-9]{10,13}", "-", "[0-9a-zA-Z]{24,}"
                ].join(""),
                severity: SecretSeverity::High,
            },
            SecretDetectionRule {
                id: "GENERIC-API-KEY".into(),
                name: "Generic API Key".into(),
                category: SecretCategory::ApiKey,
                pattern: r#"(?i)(?:api[_-]?key|apikey)['"]?\s*[:=]\s*['"]?[A-Za-z0-9_\-]{20,}['"]?"#.into(),
                severity: SecretSeverity::High,
            },
            SecretDetectionRule {
                id: "PRIVATE-KEY-RSA".into(),
                name: "RSA Private Key".into(),
                category: SecretCategory::PrivateKey,
                pattern: r"-----BEGIN RSA PRIVATE KEY-----".into(),
                severity: SecretSeverity::Critical,
            },
            SecretDetectionRule {
                id: "PRIVATE-KEY".into(),
                name: "Private Key".into(),
                category: SecretCategory::PrivateKey,
                pattern: r"-----BEGIN (?:EC |DSA |OPENSSH )?PRIVATE KEY-----".into(),
                severity: SecretSeverity::Critical,
            },
            SecretDetectionRule {
                id: "DATABASE-URL-PASSWORD".into(),
                name: "Database URL with Password".into(),
                category: SecretCategory::DatabaseUrl,
                pattern: r"(?i)(?:postgres|mysql|mongodb|redis)://[^\s:]+:[^\s@]+@[^\s]+".into(),
                severity: SecretSeverity::High,
            },
            SecretDetectionRule {
                id: "JWT-SECRET".into(),
                name: "JWT Secret Key".into(),
                category: SecretCategory::GenericSecret,
                pattern: r#"(?i)jwt[_-]?secret\s*[:=]\s*['"]?[A-Za-z0-9_\-]{20,}['"]?"#.into(),
                severity: SecretSeverity::High,
            },
        ];
        Self { rules }
    }

    pub fn scan_content(&self, content: &str, file_path: &str) -> Vec<SecretFinding> {
        let mut findings = Vec::new();
        for rule in &self.rules {
            if let Ok(re) = Regex::new(&rule.pattern) {
                for (line_idx, line) in content.lines().enumerate() {
                    if let Some(m) = re.find(line) {
                        let matched = m.as_str();
                        let redacted = if matched.len() > 16 {
                            format!("***{}***", matched.len())
                        } else {
                            "***".to_string()
                        };
                        findings.push(SecretFinding {
                            rule_id: rule.id.clone(),
                            rule_name: rule.name.clone(),
                            category: rule.category,
                            file_path: file_path.to_string(),
                            line_number: (line_idx + 1) as u32,
                            matched_content: redacted,
                            severity: rule.severity,
                            confidence: 0.9,
                        });
                    }
                }
            }
        }
        findings
    }

    pub fn add_rule(&mut self, rule: SecretDetectionRule) {
        self.rules.push(rule);
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for SecretScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_scanner() {
        let scanner = SecretScanner::new();
        assert!(scanner.rule_count() >= 10);
    }

    #[test]
    fn test_default_scanner() {
        let scanner = SecretScanner::default();
        assert!(scanner.rule_count() > 0);
    }

    #[test]
    fn test_scan_clean_content() {
        let scanner = SecretScanner::new();
        let findings = scanner.scan_content("hello world\nfoo bar", "test.rs");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_scan_github_token() {
        let scanner = SecretScanner::new();
        // Construct the token at runtime to avoid push-protection false positives.
        let token = format!("ghp_{}", "z".repeat(40));
        let content = format!("token = \"{token}\"");
        let findings = scanner.scan_content(&content, "config.toml");
        assert!(!findings.is_empty());
        assert_eq!(findings[0].rule_id, "GITHUB-TOKEN");
        assert_eq!(findings[0].category, SecretCategory::Token);
    }

    #[test]
    fn test_scan_private_key() {
        let scanner = SecretScanner::new();
        let content =
            "-----BEGIN RSA PRIVATE KEY-----\nsome key data\n-----END RSA PRIVATE KEY-----";
        let findings = scanner.scan_content(content, "key.pem");
        assert!(!findings.is_empty());
        assert_eq!(findings[0].category, SecretCategory::PrivateKey);
    }

    #[test]
    fn test_scan_database_url() {
        let scanner = SecretScanner::new();
        let content = "DATABASE_URL=postgres://user:password@localhost:5432/mydb";
        let findings = scanner.scan_content(content, ".env");
        assert!(!findings.is_empty());
        assert_eq!(findings[0].category, SecretCategory::DatabaseUrl);
    }

    #[test]
    fn test_scan_generic_api_key() {
        let scanner = SecretScanner::new();
        let content = "api_key = \"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefgh\"";
        let findings = scanner.scan_content(content, "config.json");
        assert!(!findings.is_empty());
        assert_eq!(findings[0].category, SecretCategory::ApiKey);
    }

    #[test]
    fn test_scan_aws_key() {
        let scanner = SecretScanner::new();
        // Construct the key at runtime to avoid push-protection false positives.
        let key = format!("AKIA{}", "X".repeat(16));
        let findings = scanner.scan_content(&key, "credentials");
        assert!(!findings.is_empty());
        assert_eq!(findings[0].category, SecretCategory::CloudCredential);
    }

    #[test]
    fn test_scan_slack_token() {
        let scanner = SecretScanner::new();
        // Construct the token at runtime to avoid push-protection false positives.
        let token = format!("xoxb-{}", "0".repeat(10));
        let content = format!("{token}-{}", "a".repeat(24));
        let findings = scanner.scan_content(&content, "settings.json");
        assert!(!findings.is_empty());
        assert_eq!(findings[0].category, SecretCategory::Token);
    }

    #[test]
    fn test_line_number_tracking() {
        let scanner = SecretScanner::new();
        let token = format!("ghp_{}", "z".repeat(40));
        let content = format!("line 1\nline 2\ntoken = \"{token}\"\nline 4");
        let findings = scanner.scan_content(&content, "file.txt");
        assert!(!findings.is_empty());
        assert_eq!(findings[0].line_number, 3);
    }

    #[test]
    fn test_file_path_stored() {
        let scanner = SecretScanner::new();
        let token = format!("ghp_{}", "z".repeat(40));
        let findings = scanner.scan_content(&token, "src/config.rs");
        assert!(!findings.is_empty());
        assert_eq!(findings[0].file_path, "src/config.rs");
    }

    #[test]
    fn test_redaction_short() {
        let scanner = SecretScanner::new();
        let content = "-----BEGIN EC PRIVATE KEY-----";
        let findings = scanner.scan_content(content, "key.pem");
        assert!(!findings.is_empty());
        assert!(findings[0].matched_content.starts_with("***"));
        assert!(findings[0].matched_content.ends_with("***"));
        assert!(!findings[0].matched_content.contains("BEGIN"));
    }

    #[test]
    fn test_redaction_long() {
        let scanner = SecretScanner::new();
        let token = format!("ghp_{}", "z".repeat(40));
        let findings = scanner.scan_content(&token, "file");
        assert!(!findings.is_empty());
        assert!(findings[0].matched_content.contains("***"));
        assert!(!findings[0].matched_content.contains("ghp_"));
    }

    #[test]
    fn test_add_custom_rule() {
        let mut scanner = SecretScanner::new();
        let initial_count = scanner.rule_count();
        scanner.add_rule(SecretDetectionRule {
            id: "CUSTOM-RULE".into(),
            name: "Custom Rule".into(),
            category: SecretCategory::GenericSecret,
            pattern: r"MYSECRETPREFIX-[A-Za-z0-9]+".into(),
            severity: SecretSeverity::Medium,
        });
        assert_eq!(scanner.rule_count(), initial_count + 1);
    }

    #[test]
    fn test_custom_rule_fires() {
        let mut scanner = SecretScanner::new();
        scanner.add_rule(SecretDetectionRule {
            id: "CUSTOM-PAT".into(),
            name: "Custom Pattern".into(),
            category: SecretCategory::GenericSecret,
            pattern: r"SECR3T_[A-Z]+".into(),
            severity: SecretSeverity::Medium,
        });
        let findings = scanner.scan_content("export SECR3T_ABCDEFG", "env");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "CUSTOM-PAT");
    }

    #[test]
    fn test_multiple_findings_same_file() {
        let scanner = SecretScanner::new();
        let token = format!("ghp_{}", "z".repeat(40));
        let content = format!("{token}\n-----BEGIN RSA PRIVATE KEY-----");
        let findings = scanner.scan_content(&content, "leak.txt");
        assert!(findings.len() >= 2);
    }

    #[test]
    fn test_finding_severity() {
        let scanner = SecretScanner::new();
        let token = format!("ghp_{}", "z".repeat(40));
        let findings = scanner.scan_content(&token, "file");
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, SecretSeverity::Critical);
    }

    #[test]
    fn test_finding_serialization() {
        let finding = SecretFinding {
            rule_id: "TEST".into(),
            rule_name: "Test Rule".into(),
            category: SecretCategory::ApiKey,
            file_path: "test.rs".into(),
            line_number: 42,
            matched_content: "redacted".into(),
            severity: SecretSeverity::High,
            confidence: 0.95,
        };
        let json = serde_json::to_string(&finding).unwrap();
        let de: SecretFinding = serde_json::from_str(&json).unwrap();
        assert_eq!(de.rule_id, "TEST");
        assert_eq!(de.line_number, 42);
        assert!((de.confidence - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_category_serialization() {
        let cat = SecretCategory::CloudCredential;
        let json = serde_json::to_string(&cat).unwrap();
        let de: SecretCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(de, SecretCategory::CloudCredential);
    }

    #[test]
    fn test_severity_serialization() {
        let sev = SecretSeverity::Critical;
        let json = serde_json::to_string(&sev).unwrap();
        let de: SecretSeverity = serde_json::from_str(&json).unwrap();
        assert_eq!(de, SecretSeverity::Critical);
    }
}
