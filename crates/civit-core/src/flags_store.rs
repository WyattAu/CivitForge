#![forbid(unsafe_code)]

//! flag-kit `FlagStore` adapter over the CivitForge DB tables
//! (ADR-0007 step 2).
//!
//! Read model: the kit `Evaluator` evaluates flags through this store;
//! mutations continue to flow through the admin API
//! (`api/feature_flags.rs`), which owns full-fidelity writes (targeting
//! lists, descriptions) that the kit `Flag` shape cannot express.

use std::sync::Arc;

use flag_kit::{Flag, FlagError, FlagName, FlagStore};

use crate::db::DbRepository;

/// Read-model store bridging `feature_flags` DB rows into the kit.
#[derive(Clone)]
pub struct DbFlagStore {
    db: Arc<DbRepository>,
}

impl DbFlagStore {
    pub fn new(db: Arc<DbRepository>) -> Self {
        Self { db }
    }
}

fn to_kit_flag(f: &civit_db::models::FeatureFlag) -> Option<Flag> {
    // Read-path names bypass `FlagName::new` validation: legacy rows may
    // predate the snake_case convention, and evaluation must not break
    // because of a historical name.
    let name = FlagName::new_unchecked(f.name.clone());
    let percentage = f.enabled_for_percentage.clamp(0, 100) as u8;
    Flag::with_created_at(name, f.enabled, percentage, f.created_at).ok()
}

#[async_trait::async_trait]
impl FlagStore for DbFlagStore {
    async fn get(&self, name: &FlagName) -> Option<Flag> {
        let all = self.db.list_feature_flags().await.ok()?;
        all.iter()
            .find(|f| f.name == name.as_str())
            .and_then(to_kit_flag)
    }

    async fn list(&self) -> Vec<Flag> {
        self.db
            .list_feature_flags()
            .await
            .unwrap_or_default()
            .iter()
            .filter_map(to_kit_flag)
            .collect()
    }

    /// Mutations are intentionally unsupported on the read model: kit
    /// `Flag` cannot express targeting lists or descriptions, so a `set`
    /// here would silently clobber admin-managed rows. Writes go through
    /// the admin API.
    async fn set(&self, _flag: Flag) -> flag_kit::error::Result<()> {
        Err(FlagError::Storage(
            "DbFlagStore is a read model; mutate flags via the admin API".into(),
        ))
    }

    async fn delete(&self, _name: &FlagName) -> flag_kit::error::Result<bool> {
        Err(FlagError::Storage(
            "DbFlagStore is a read model; delete flags via the admin API".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    /// Row factory: governance columns default to their schema defaults so
    /// these tests state only what they are about.
    fn row(name: &str, percentage: i32) -> civit_db::models::FeatureFlag {
        civit_db::models::FeatureFlag {
            id: uuid::Uuid::nil(),
            name: name.into(),
            description: "test".into(),
            enabled: true,
            enabled_for_users: Vec::new(),
            enabled_for_percentage: percentage,
            enabled_for_orgs: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            kind: "release".into(),
            owner: "team".into(),
            ticket: "T-1".into(),
            salt: String::new(),
            last_evaluated_at: Some(Utc::now()),
            last_changed_at: Utc::now(),
        }
    }

    #[test]
    fn db_row_maps_to_kit_flag() {
        let row = row("my_flag", 50);
        let flag = to_kit_flag(&row).expect("valid row maps");
        assert_eq!(flag.name.as_str(), "my_flag");
        assert!(flag.enabled);
        assert_eq!(flag.percentage, 50);
    }

    #[test]
    fn percentage_clamped_to_u8_range() {
        let flag = to_kit_flag(&row("over", 250)).expect("clamped high");
        assert_eq!(flag.percentage, 100);

        let flag = to_kit_flag(&row("under", -5)).expect("clamped low");
        assert_eq!(flag.percentage, 0);
    }
}
