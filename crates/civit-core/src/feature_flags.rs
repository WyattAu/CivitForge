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

    /// Insert or replace a flag. Keys are validated by the flag-kit
    /// `FlagName` rules (ADR-0006 Phase 4).
    ///
    /// # Errors
    /// Returns an error string when the key is not a valid flag name.
    pub fn set_flag(&self, flag: FeatureFlag) -> Result<(), String> {
        flag_kit::FlagName::new(flag.key.clone())
            .map_err(|e| format!("invalid flag key: {e}"))?;
        let mut flags = self.flags.lock();
        flags.insert(flag.key.clone(), flag);
        Ok(())
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
            // Kit deterministic bucketing (0-99) — consistent distribution
            // across every CivitForge service using flag-kit.
            let bucket = flag_kit::bucket(key, &context.user_id);
            if u16::from(bucket) >= u16::from(flag.rollout_percentage) {
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
        let flag = test_flag("dark_mode", "Dark Mode", true);
        svc.set_flag(flag).unwrap();
        let retrieved = svc.get_flag("dark_mode").unwrap();
        assert_eq!(retrieved.key, "dark_mode");
        assert!(retrieved.enabled);
    }

    #[test]
    fn test_remove_flag() {
        let svc = FeatureFlagService::new();
        svc.set_flag(test_flag("feat1", "Feature 1", true)).unwrap();
        assert!(svc.remove_flag("feat1"));
        assert!(!svc.remove_flag("feat1"));
        assert_eq!(svc.flag_count(), 0);
    }

    #[test]
    fn test_is_enabled_true() {
        let svc = FeatureFlagService::new();
        svc.set_flag(test_flag("on_flag", "On Flag", true)).unwrap();
        let ctx = EvaluationContext::new("user1");
        assert!(svc.is_enabled("on_flag", &ctx));
    }

    #[test]
    fn test_is_enabled_false() {
        let svc = FeatureFlagService::new();
        svc.set_flag(test_flag("off_flag", "Off Flag", false)).unwrap();
        let ctx = EvaluationContext::new("user1");
        assert!(!svc.is_enabled("off_flag", &ctx));
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
        svc.set_flag(flag).unwrap();
        let ctx1 = EvaluationContext::new("user1");
        let ctx3 = EvaluationContext::new("user3");
        assert!(svc.is_enabled("beta", &ctx1));
        assert!(!svc.is_enabled("beta", &ctx3));
    }

    #[test]
    fn test_target_org() {
        let svc = FeatureFlagService::new();
        let mut flag = test_flag("org_feat", "Org Feature", true);
        flag.target_orgs = vec!["org1".into()];
        svc.set_flag(flag).unwrap();
        let ctx_ok = EvaluationContext::with_org("user1", "org1");
        let ctx_no = EvaluationContext::with_org("user2", "org2");
        assert!(svc.is_enabled("org_feat", &ctx_ok));
        assert!(!svc.is_enabled("org_feat", &ctx_no));
    }

    #[test]
    fn test_rollout_percentage() {
        let svc = FeatureFlagService::new();
        let mut flag = test_flag("rollout", "Rollout", true);
        flag.rollout_percentage = 0;
        svc.set_flag(flag).unwrap();
        let ctx = EvaluationContext::new("user1");
        assert!(!svc.is_enabled("rollout", &ctx));
    }

    #[test]
    fn test_rollout_100_percent() {
        let svc = FeatureFlagService::new();
        let mut flag = test_flag("full_rollout", "Full Rollout", true);
        flag.rollout_percentage = 100;
        svc.set_flag(flag).unwrap();
        let ctx = EvaluationContext::new("any_user");
        assert!(svc.is_enabled("full_rollout", &ctx));
    }

    #[test]
    fn test_get_variant() {
        let svc = FeatureFlagService::new();
        let mut flag = test_flag("ab_test", "AB Test", true);
        flag.variant = Some("variant_a".into());
        svc.set_flag(flag).unwrap();
        let ctx = EvaluationContext::new("user1");
        assert_eq!(svc.get_variant("ab_test", &ctx), Some("variant_a".into()));
    }

    #[test]
    fn test_get_variant_disabled() {
        let svc = FeatureFlagService::new();
        let mut flag = test_flag("ab_test_off", "AB Test Off", false);
        flag.variant = Some("variant_b".into());
        svc.set_flag(flag).unwrap();
        let ctx = EvaluationContext::new("user1");
        assert_eq!(svc.get_variant("ab_test_off", &ctx), None);
    }

    #[test]
    fn test_get_variant_none() {
        let svc = FeatureFlagService::new();
        svc.set_flag(test_flag("no_variant", "No Variant", true)).unwrap();
        let ctx = EvaluationContext::new("user1");
        assert_eq!(svc.get_variant("no_variant", &ctx), None);
    }

    #[test]
    fn test_list_flags() {
        let svc = FeatureFlagService::new();
        svc.set_flag(test_flag("a", "A", true)).unwrap();
        svc.set_flag(test_flag("b", "B", false)).unwrap();
        let flags = svc.list_flags();
        assert_eq!(flags.len(), 2);
    }

    #[test]
    fn test_flag_count() {
        let svc = FeatureFlagService::new();
        assert_eq!(svc.flag_count(), 0);
        svc.set_flag(test_flag("x", "X", true)).unwrap();
        assert_eq!(svc.flag_count(), 1);
        svc.set_flag(test_flag("y", "Y", false)).unwrap();
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
        let flag = test_flag("test_key", "Test Flag", true);
        let json = serde_json::to_string(&flag).unwrap();
        let de: FeatureFlag = serde_json::from_str(&json).unwrap();
        assert_eq!(de.key, "test_key");
        assert!(de.enabled);
    }

    #[test]
    fn test_update_flag() {
        let svc = FeatureFlagService::new();
        svc.set_flag(test_flag("toggle", "Toggle", true)).unwrap();
        assert!(svc.is_enabled("toggle", &EvaluationContext::new("u")));
        svc.set_flag(test_flag("toggle", "Toggle", false)).unwrap();
        assert!(!svc.is_enabled("toggle", &EvaluationContext::new("u")));
    }

    #[test]
    fn test_no_org_context_passes_org_target() {
        let svc = FeatureFlagService::new();
        let mut flag = test_flag("org_only", "Org Only", true);
        flag.target_orgs = vec!["org1".into()];
        svc.set_flag(flag).unwrap();
        let ctx = EvaluationContext::new("user1");
        assert!(svc.is_enabled("org_only", &ctx));
    }
}
