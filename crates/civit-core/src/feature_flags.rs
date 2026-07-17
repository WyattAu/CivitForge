#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use parking_lot::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub key: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub variant: Option<String>,
    pub rollout_percentage: u8,
    pub target_users: Vec<String>,
    pub target_orgs: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct EvaluationContext {
    pub user_id: String,
    pub org_id: Option<String>,
    pub attributes: HashMap<String, String>,
}

impl EvaluationContext {
    pub fn new(user_id: &str) -> Self {
        Self {
            user_id: user_id.into(),
            org_id: None,
            attributes: HashMap::new(),
        }
    }

    pub fn with_org(user_id: &str, org_id: &str) -> Self {
        Self {
            user_id: user_id.into(),
            org_id: Some(org_id.into()),
            attributes: HashMap::new(),
        }
    }
}

pub struct FeatureFlagService {
    flags: Mutex<HashMap<String, FeatureFlag>>,
}

impl FeatureFlagService {
    pub fn new() -> Self {
        Self {
            flags: Mutex::new(HashMap::new()),
        }
    }

    pub fn set_flag(&self, flag: FeatureFlag) {
        let mut flags = self.flags.lock();
        flags.insert(flag.key.clone(), flag);
    }

    pub fn remove_flag(&self, key: &str) -> bool {
        let mut flags = self.flags.lock();
        flags.remove(key).is_some()
    }

    pub fn is_enabled(&self, key: &str, context: &EvaluationContext) -> bool {
        let flags = self.flags.lock();
        let Some(flag) = flags.get(key) else {
            return false;
        };
        if !flag.enabled {
            return false;
        }
        if !flag.target_users.is_empty() && !flag.target_users.iter().any(|u| u == &context.user_id)
        {
            return false;
        }
        if let Some(ref org_id) = context.org_id
            && !flag.target_orgs.is_empty()
            && !flag.target_orgs.iter().any(|o| o == org_id)
        {
            return false;
        }
        if flag.rollout_percentage < 100 {
            let hash = simple_hash(&context.user_id);
            if (hash % 100u64) >= flag.rollout_percentage as u64 {
                return false;
            }
        }
        true
    }

    pub fn get_variant(&self, key: &str, context: &EvaluationContext) -> Option<String> {
        if self.is_enabled(key, context) {
            let flags = self.flags.lock();
            flags.get(key).and_then(|f| f.variant.clone())
        } else {
            None
        }
    }

    pub fn get_flag(&self, key: &str) -> Option<FeatureFlag> {
        let flags = self.flags.lock();
        flags.get(key).cloned()
    }

    pub fn list_flags(&self) -> Vec<FeatureFlag> {
        let flags = self.flags.lock();
        flags.values().cloned().collect()
    }

    pub fn flag_count(&self) -> usize {
        let flags = self.flags.lock();
        flags.len()
    }
}

impl Default for FeatureFlagService {
    fn default() -> Self {
        Self::new()
    }
}

fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 0;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    hash
}

#[cfg(test)]
fn test_flag(key: &str, name: &str, enabled: bool) -> FeatureFlag {
    FeatureFlag {
        key: key.into(),
        name: name.into(),
        description: "test".into(),
        enabled,
        variant: None,
        rollout_percentage: 100,
        target_users: Vec::new(),
        target_orgs: Vec::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_service() {
        let svc = FeatureFlagService::new();
        assert_eq!(svc.flag_count(), 0);
    }

    #[test]
    fn test_default_service() {
        let svc = FeatureFlagService::default();
        assert_eq!(svc.flag_count(), 0);
    }

    #[test]
    fn test_set_and_get_flag() {
        let svc = FeatureFlagService::new();
        let flag = test_flag("dark-mode", "Dark Mode", true);
        svc.set_flag(flag);
        let retrieved = svc.get_flag("dark-mode").unwrap();
        assert_eq!(retrieved.key, "dark-mode");
        assert!(retrieved.enabled);
    }

    #[test]
    fn test_remove_flag() {
        let svc = FeatureFlagService::new();
        svc.set_flag(test_flag("feat1", "Feature 1", true));
        assert!(svc.remove_flag("feat1"));
        assert!(!svc.remove_flag("feat1"));
        assert_eq!(svc.flag_count(), 0);
    }

    #[test]
    fn test_is_enabled_true() {
        let svc = FeatureFlagService::new();
        svc.set_flag(test_flag("on-flag", "On Flag", true));
        let ctx = EvaluationContext::new("user1");
        assert!(svc.is_enabled("on-flag", &ctx));
    }

    #[test]
    fn test_is_enabled_false() {
        let svc = FeatureFlagService::new();
        svc.set_flag(test_flag("off-flag", "Off Flag", false));
        let ctx = EvaluationContext::new("user1");
        assert!(!svc.is_enabled("off-flag", &ctx));
    }

    #[test]
    fn test_is_enabled_missing_flag() {
        let svc = FeatureFlagService::new();
        let ctx = EvaluationContext::new("user1");
        assert!(!svc.is_enabled("nonexistent", &ctx));
    }

    #[test]
    fn test_target_user() {
        let svc = FeatureFlagService::new();
        let mut flag = test_flag("beta", "Beta", true);
        flag.target_users = vec!["user1".into(), "user2".into()];
        svc.set_flag(flag);
        let ctx1 = EvaluationContext::new("user1");
        let ctx3 = EvaluationContext::new("user3");
        assert!(svc.is_enabled("beta", &ctx1));
        assert!(!svc.is_enabled("beta", &ctx3));
    }

    #[test]
    fn test_target_org() {
        let svc = FeatureFlagService::new();
        let mut flag = test_flag("org-feat", "Org Feature", true);
        flag.target_orgs = vec!["org1".into()];
        svc.set_flag(flag);
        let ctx_ok = EvaluationContext::with_org("user1", "org1");
        let ctx_no = EvaluationContext::with_org("user2", "org2");
        assert!(svc.is_enabled("org-feat", &ctx_ok));
        assert!(!svc.is_enabled("org-feat", &ctx_no));
    }

    #[test]
    fn test_rollout_percentage() {
        let svc = FeatureFlagService::new();
        let mut flag = test_flag("rollout", "Rollout", true);
        flag.rollout_percentage = 0;
        svc.set_flag(flag);
        let ctx = EvaluationContext::new("user1");
        assert!(!svc.is_enabled("rollout", &ctx));
    }

    #[test]
    fn test_rollout_100_percent() {
        let svc = FeatureFlagService::new();
        let mut flag = test_flag("full-rollout", "Full Rollout", true);
        flag.rollout_percentage = 100;
        svc.set_flag(flag);
        let ctx = EvaluationContext::new("any-user");
        assert!(svc.is_enabled("full-rollout", &ctx));
    }

    #[test]
    fn test_get_variant() {
        let svc = FeatureFlagService::new();
        let mut flag = test_flag("ab-test", "AB Test", true);
        flag.variant = Some("variant-a".into());
        svc.set_flag(flag);
        let ctx = EvaluationContext::new("user1");
        assert_eq!(svc.get_variant("ab-test", &ctx), Some("variant-a".into()));
    }

    #[test]
    fn test_get_variant_disabled() {
        let svc = FeatureFlagService::new();
        let mut flag = test_flag("ab-test-off", "AB Test Off", false);
        flag.variant = Some("variant-b".into());
        svc.set_flag(flag);
        let ctx = EvaluationContext::new("user1");
        assert_eq!(svc.get_variant("ab-test-off", &ctx), None);
    }

    #[test]
    fn test_get_variant_none() {
        let svc = FeatureFlagService::new();
        svc.set_flag(test_flag("no-variant", "No Variant", true));
        let ctx = EvaluationContext::new("user1");
        assert_eq!(svc.get_variant("no-variant", &ctx), None);
    }

    #[test]
    fn test_list_flags() {
        let svc = FeatureFlagService::new();
        svc.set_flag(test_flag("a", "A", true));
        svc.set_flag(test_flag("b", "B", false));
        let flags = svc.list_flags();
        assert_eq!(flags.len(), 2);
    }

    #[test]
    fn test_flag_count() {
        let svc = FeatureFlagService::new();
        assert_eq!(svc.flag_count(), 0);
        svc.set_flag(test_flag("x", "X", true));
        assert_eq!(svc.flag_count(), 1);
        svc.set_flag(test_flag("y", "Y", false));
        assert_eq!(svc.flag_count(), 2);
    }

    #[test]
    fn test_evaluation_context_new() {
        let ctx = EvaluationContext::new("user1");
        assert_eq!(ctx.user_id, "user1");
        assert!(ctx.org_id.is_none());
    }

    #[test]
    fn test_evaluation_context_with_org() {
        let ctx = EvaluationContext::with_org("user1", "org1");
        assert_eq!(ctx.org_id, Some("org1".into()));
    }

    #[test]
    fn test_flag_serialization() {
        let flag = test_flag("test-key", "Test Flag", true);
        let json = serde_json::to_string(&flag).unwrap();
        let de: FeatureFlag = serde_json::from_str(&json).unwrap();
        assert_eq!(de.key, "test-key");
        assert!(de.enabled);
    }

    #[test]
    fn test_update_flag() {
        let svc = FeatureFlagService::new();
        svc.set_flag(test_flag("toggle", "Toggle", true));
        assert!(svc.is_enabled("toggle", &EvaluationContext::new("u")));
        svc.set_flag(test_flag("toggle", "Toggle", false));
        assert!(!svc.is_enabled("toggle", &EvaluationContext::new("u")));
    }

    #[test]
    fn test_no_org_context_passes_org_target() {
        let svc = FeatureFlagService::new();
        let mut flag = test_flag("org-only", "Org Only", true);
        flag.target_orgs = vec!["org1".into()];
        svc.set_flag(flag);
        let ctx = EvaluationContext::new("user1");
        assert!(svc.is_enabled("org-only", &ctx));
    }
}
