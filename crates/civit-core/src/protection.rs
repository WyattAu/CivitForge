#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use parking_lot::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchProtectionRule {
    pub id: String,
    pub pattern: String,
    pub required_reviews: u32,
    pub require_up_to_date: bool,
    pub require_status_checks: bool,
    pub required_status_checks: Vec<String>,
    pub enforce_admins: bool,
    pub require_signed_commits: bool,
    pub allow_force_pushes: bool,
    pub allow_deletions: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushEvaluation {
    pub allowed: bool,
    pub rule_id: Option<String>,
    pub violations: Vec<String>,
}

pub struct BranchProtectionEvaluator {
    rules: Mutex<Vec<BranchProtectionRule>>,
}

impl BranchProtectionEvaluator {
    pub fn new() -> Self {
        Self {
            rules: Mutex::new(Vec::new()),
        }
    }

    pub fn add_rule(&self, rule: BranchProtectionRule) {
        self.rules.lock().push(rule);
    }

    pub fn remove_rule(&self, id: &str) -> bool {
        let mut rules = self.rules.lock();
        let before = rules.len();
        rules.retain(|r| r.id != id);
        rules.len() < before
    }

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate_push(
        &self,
        branch: &str,
        reviews_count: u32,
        is_up_to_date: bool,
        status_checks_passed: &[String],
        is_admin: bool,
        is_force_push: bool,
        is_signed: bool,
    ) -> PushEvaluation {
        let rules = self.rules.lock();
        let matching: Vec<&BranchProtectionRule> = rules
            .iter()
            .filter(|r| Self::pattern_matches(&r.pattern, branch))
            .collect();

        if matching.is_empty() {
            return PushEvaluation {
                allowed: true,
                rule_id: None,
                violations: Vec::new(),
            };
        }

        let rule = &matching[0];
        let mut violations = Vec::new();

        if !is_admin || rule.enforce_admins {
            if reviews_count < rule.required_reviews {
                violations.push(format!(
                    "requires {} review(s), got {}",
                    rule.required_reviews, reviews_count
                ));
            }
            if rule.require_up_to_date && !is_up_to_date {
                violations.push("branch must be up to date".into());
            }
            if rule.require_signed_commits && !is_signed {
                violations.push("commits must be signed".into());
            }
            if !rule.allow_force_pushes && is_force_push {
                violations.push("force pushes are not allowed".into());
            }
            if rule.require_status_checks {
                for check in &rule.required_status_checks {
                    if !status_checks_passed.contains(check) {
                        violations.push(format!("required status check '{check}' not passed"));
                    }
                }
            }
        }

        PushEvaluation {
            allowed: violations.is_empty(),
            rule_id: Some(rule.id.clone()),
            violations,
        }
    }

    fn pattern_matches(pattern: &str, branch: &str) -> bool {
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return branch.starts_with(parts[0]) && branch.ends_with(parts[1]);
            }
            let prefix = pattern.trim_end_matches('*');
            branch.starts_with(prefix)
        } else {
            pattern == branch
        }
    }

    pub fn get_rules(&self) -> Vec<BranchProtectionRule> {
        self.rules.lock().clone()
    }

    pub fn rule_count(&self) -> usize {
        self.rules.lock().len()
    }
}

impl Default for BranchProtectionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rule(id: &str, pattern: &str, required_reviews: u32) -> BranchProtectionRule {
        BranchProtectionRule {
            id: id.to_string(),
            pattern: pattern.to_string(),
            required_reviews,
            require_up_to_date: true,
            require_status_checks: false,
            required_status_checks: Vec::new(),
            enforce_admins: true,
            require_signed_commits: false,
            allow_force_pushes: false,
            allow_deletions: false,
            created_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_no_rules_allows_push() {
        let ev = BranchProtectionEvaluator::new();
        let result = ev.evaluate_push("main", 0, true, &[], false, false, false);
        assert!(result.allowed);
        assert!(result.rule_id.is_none());
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_exact_pattern_match() {
        let ev = BranchProtectionEvaluator::new();
        ev.add_rule(make_rule("r1", "main", 2));
        let result = ev.evaluate_push("main", 2, true, &[], false, false, false);
        assert!(result.allowed);
        assert_eq!(result.rule_id.as_deref(), Some("r1"));
    }

    #[test]
    fn test_exact_pattern_no_match() {
        let ev = BranchProtectionEvaluator::new();
        ev.add_rule(make_rule("r1", "main", 2));
        let result = ev.evaluate_push("develop", 0, true, &[], false, false, false);
        assert!(result.allowed);
        assert!(result.rule_id.is_none());
    }

    #[test]
    fn test_wildcard_pattern() {
        let ev = BranchProtectionEvaluator::new();
        ev.add_rule(make_rule("r1", "release/*", 1));
        let result = ev.evaluate_push("release/v1.0", 1, true, &[], false, false, false);
        assert!(result.allowed);
        assert_eq!(result.rule_id.as_deref(), Some("r1"));
    }

    #[test]
    fn test_insufficient_reviews() {
        let ev = BranchProtectionEvaluator::new();
        ev.add_rule(make_rule("r1", "main", 2));
        let result = ev.evaluate_push("main", 1, true, &[], false, false, false);
        assert!(!result.allowed);
        assert!(result.violations.iter().any(|v| v.contains("review")));
    }

    #[test]
    fn test_stale_branch_rejected() {
        let ev = BranchProtectionEvaluator::new();
        ev.add_rule(make_rule("r1", "main", 0));
        let result = ev.evaluate_push("main", 0, false, &[], false, false, false);
        assert!(!result.allowed);
        assert!(result.violations.iter().any(|v| v.contains("up to date")));
    }

    #[test]
    fn test_unsigned_commits_rejected() {
        let ev = BranchProtectionEvaluator::new();
        let mut rule = make_rule("r1", "main", 0);
        rule.require_signed_commits = true;
        ev.add_rule(rule);
        let result = ev.evaluate_push("main", 0, true, &[], false, false, false);
        assert!(!result.allowed);
        assert!(result.violations.iter().any(|v| v.contains("signed")));
    }

    #[test]
    fn test_force_push_rejected() {
        let ev = BranchProtectionEvaluator::new();
        ev.add_rule(make_rule("r1", "main", 0));
        let result = ev.evaluate_push("main", 0, true, &[], false, true, false);
        assert!(!result.allowed);
        assert!(result.violations.iter().any(|v| v.contains("force")));
    }

    #[test]
    fn test_force_push_allowed_when_configured() {
        let ev = BranchProtectionEvaluator::new();
        let mut rule = make_rule("r1", "main", 0);
        rule.allow_force_pushes = true;
        ev.add_rule(rule);
        let result = ev.evaluate_push("main", 0, true, &[], false, true, false);
        assert!(result.allowed);
    }

    #[test]
    fn test_status_check_required() {
        let ev = BranchProtectionEvaluator::new();
        let mut rule = make_rule("r1", "main", 0);
        rule.require_status_checks = true;
        rule.required_status_checks = vec!["ci".to_string(), "lint".to_string()];
        ev.add_rule(rule);
        let result = ev.evaluate_push("main", 0, true, &["ci".to_string()], false, false, false);
        assert!(!result.allowed);
        assert!(result.violations.iter().any(|v| v.contains("lint")));
    }

    #[test]
    fn test_status_checks_all_passed() {
        let ev = BranchProtectionEvaluator::new();
        let mut rule = make_rule("r1", "main", 0);
        rule.require_status_checks = true;
        rule.required_status_checks = vec!["ci".to_string(), "lint".to_string()];
        ev.add_rule(rule);
        let result = ev.evaluate_push(
            "main",
            0,
            true,
            &["ci".to_string(), "lint".to_string()],
            false,
            false,
            false,
        );
        assert!(result.allowed);
    }

    #[test]
    fn test_admin_bypass_when_not_enforced() {
        let ev = BranchProtectionEvaluator::new();
        let mut rule = make_rule("r1", "main", 2);
        rule.enforce_admins = false;
        ev.add_rule(rule);
        let result = ev.evaluate_push("main", 0, true, &[], true, false, false);
        assert!(result.allowed);
    }

    #[test]
    fn test_admin_enforced_still_checked() {
        let ev = BranchProtectionEvaluator::new();
        let mut rule = make_rule("r1", "main", 2);
        rule.enforce_admins = true;
        ev.add_rule(rule);
        let result = ev.evaluate_push("main", 0, true, &[], true, false, false);
        assert!(!result.allowed);
    }

    #[test]
    fn test_remove_rule() {
        let ev = BranchProtectionEvaluator::new();
        ev.add_rule(make_rule("r1", "main", 2));
        assert_eq!(ev.rule_count(), 1);
        assert!(ev.remove_rule("r1"));
        assert_eq!(ev.rule_count(), 0);
    }

    #[test]
    fn test_remove_nonexistent_rule() {
        let ev = BranchProtectionEvaluator::new();
        assert!(!ev.remove_rule("nonexistent"));
    }

    #[test]
    fn test_rule_serialization_roundtrip() {
        let rule = make_rule("r1", "main", 2);
        let json = serde_json::to_string(&rule).unwrap();
        let de: BranchProtectionRule = serde_json::from_str(&json).unwrap();
        assert_eq!(de.id, "r1");
        assert_eq!(de.pattern, "main");
        assert_eq!(de.required_reviews, 2);
    }

    #[test]
    fn test_evaluation_serialization_roundtrip() {
        let eval = PushEvaluation {
            allowed: false,
            rule_id: Some("r1".to_string()),
            violations: vec!["needs review".to_string()],
        };
        let json = serde_json::to_string(&eval).unwrap();
        let de: PushEvaluation = serde_json::from_str(&json).unwrap();
        assert!(!de.allowed);
        assert_eq!(de.violations.len(), 1);
    }

    #[test]
    fn test_default_is_empty() {
        let ev = BranchProtectionEvaluator::default();
        assert_eq!(ev.rule_count(), 0);
        assert!(ev.get_rules().is_empty());
    }
}
