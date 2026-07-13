#![forbid(unsafe_code)]

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlagState {
    pub flags: Vec<FeatureFlag>,
    pub loading: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeatureFlag {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub enabled_for_users: Vec<String>,
    pub enabled_for_percentage: i32,
    pub enabled_for_orgs: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl FeatureFlagState {
    pub fn new() -> Self {
        Self {
            flags: Vec::new(),
            loading: false,
            error: None,
        }
    }

    pub fn is_enabled(&self, flag_name: &str) -> bool {
        self.flags.iter().any(|f| f.name == flag_name && f.enabled)
    }
}

impl Default for FeatureFlagState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct FeatureFlagContext {
    pub state: FeatureFlag,
}

impl FeatureFlagContext {
    pub fn is_enabled(&self) -> bool {
        self.state.enabled
    }
}

pub fn use_feature_flag(flags: Signal<FeatureFlagState>, flag_name: &str) -> bool {
    flags.with(|f| f.is_enabled(flag_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_flag_state_new() {
        let state = FeatureFlagState::new();
        assert!(state.flags.is_empty());
        assert!(!state.loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn test_feature_flag_is_enabled() {
        let mut state = FeatureFlagState::new();
        state.flags.push(FeatureFlag {
            id: "1".into(),
            name: "dark-mode".into(),
            description: "Dark mode".into(),
            enabled: true,
            enabled_for_users: vec![],
            enabled_for_percentage: 100,
            enabled_for_orgs: vec![],
            created_at: "".into(),
            updated_at: "".into(),
        });
        assert!(state.is_enabled("dark-mode"));
        assert!(!state.is_enabled("unknown"));
    }

    #[test]
    fn test_feature_flag_disabled() {
        let mut state = FeatureFlagState::new();
        state.flags.push(FeatureFlag {
            id: "1".into(),
            name: "beta".into(),
            description: "Beta feature".into(),
            enabled: false,
            enabled_for_users: vec![],
            enabled_for_percentage: 0,
            enabled_for_orgs: vec![],
            created_at: "".into(),
            updated_at: "".into(),
        });
        assert!(!state.is_enabled("beta"));
    }
}
