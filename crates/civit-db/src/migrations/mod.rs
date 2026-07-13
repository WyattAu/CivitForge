#![forbid(unsafe_code)]

pub const M_001_INITIAL_SCHEMA_UP: &str = include_str!("001_initial_schema.sql");
pub const M_001_INITIAL_SCHEMA_DOWN: &str = include_str!("down/002_initial_schema_down.sql");
pub const M_003_PHASE1_UP: &str = include_str!("003_add_ssh_keys_branches_steps_events.sql");
pub const M_003_PHASE1_DOWN: &str =
    include_str!("down/004_add_ssh_keys_branches_steps_events_down.sql");
pub const M_005_AUTH_UP: &str = include_str!("005_add_auth_identity_tables.sql");
pub const M_005_AUTH_DOWN: &str = include_str!("down/006_add_auth_identity_tables_down.sql");
pub const M_007_PERMISSIONS_UP: &str = include_str!("007_add_permissions_tables.sql");
pub const M_007_PERMISSIONS_DOWN: &str = include_str!("down/007_add_permissions_tables_down.sql");
pub const M_009_PIPELINE_UP: &str = include_str!("009_add_ci_cd_pipeline_tables.sql");
pub const M_009_PIPELINE_DOWN: &str = include_str!("down/010_add_ci_cd_pipeline_tables_down.sql");
pub const M_011_OCI_REGISTRY_UP: &str = include_str!("011_add_oci_registry_tables.sql");
pub const M_011_OCI_REGISTRY_DOWN: &str = include_str!("down/012_add_oci_registry_tables_down.sql");
pub const M_013_ISSUES_UP: &str = include_str!("013_add_issue_tracking_tables.sql");
pub const M_013_ISSUES_DOWN: &str = include_str!("down/014_add_issue_tracking_tables_down.sql");
pub const M_015_WIKI_UP: &str = include_str!("015_add_wiki_tables.sql");
pub const M_015_WIKI_DOWN: &str = include_str!("down/016_add_wiki_tables_down.sql");
pub const M_017_SEARCH_UP: &str = include_str!("017_add_code_search_tables.sql");
pub const M_017_SEARCH_DOWN: &str = include_str!("down/018_add_code_search_tables_down.sql");
pub const M_019_WIKI_SNAPSHOT_UP: &str = include_str!("019_add_wiki_content_snapshot.sql");
pub const M_019_WIKI_SNAPSHOT_DOWN: &str =
    include_str!("down/020_add_wiki_content_snapshot_down.sql");
pub const M_021_FTS_UP: &str = include_str!("021_add_fulltext_search.sql");
pub const M_021_FTS_DOWN: &str = include_str!("down/022_add_fulltext_search_down.sql");
pub const M_023_WIKI_FK_UP: &str = include_str!("023_drop_wiki_created_by_fk.sql");
pub const M_023_WIKI_FK_DOWN: &str = include_str!("down/024_drop_wiki_created_by_fk_down.sql");
pub const M_025_ACTIVITY_FED_UP: &str = include_str!("025_add_activity_federation.sql");
pub const M_025_ACTIVITY_FED_DOWN: &str = include_str!("down/026_add_activity_federation_down.sql");
pub const M_027_WIKI_GIT_UP: &str = include_str!("027_add_wiki_git_enabled.sql");
pub const M_027_WIKI_GIT_DOWN: &str = include_str!("down/028_add_wiki_git_enabled_down.sql");
pub const M_029_PASSWORD_HASH_UP: &str = include_str!("029_add_password_hash.sql");
pub const M_031_PR_TRACKING_UP: &str = include_str!("031_add_pr_tracking.sql");
pub const M_031_PR_TRACKING_DOWN: &str = include_str!("down/032_add_pr_tracking_down.sql");
pub const M_033_STAR_WATCH_COUNTS_UP: &str = include_str!("033_add_star_watch_counts.sql");
pub const M_033_STAR_WATCH_COUNTS_DOWN: &str =
    include_str!("down/034_add_star_watch_counts_down.sql");
pub const M_035_LOGIN_ATTEMPTS_UP: &str = include_str!("035_add_login_attempts.sql");
pub const M_036_EMAIL_VERIFICATION_UP: &str = include_str!("036_add_email_verification.sql");
pub const M_038_REPO_SECRETS_CACHES_UP: &str =
    include_str!("038_add_repo_secrets_and_pipeline_caches.sql");
pub const M_038_REPO_SECRETS_CACHES_DOWN: &str =
    include_str!("down/038_add_repo_secrets_and_pipeline_caches_down.sql");
pub const M_037_MENTIONS_XREFS_UP: &str = include_str!("037_add_mentions_and_crossrefs.sql");
pub const M_043_PIPELINE_SCHEDULES_UP: &str = include_str!("043_add_pipeline_schedules.sql");
pub const M_044_REPO_COLLABORATORS_UP: &str = include_str!("044_add_repo_collaborators.sql");
pub const M_045_DEPLOY_KEYS_UP: &str = include_str!("045_add_deploy_keys.sql");
pub const M_046_NOTIFICATIONS_UP: &str = include_str!("046_add_notifications.sql");
pub const M_047_ADD_USER_BANNED_UP: &str = include_str!("047_add_user_banned.sql");
pub const M_048_ADD_REPO_ARCHIVED_TOPICS_UP: &str =
    include_str!("048_add_repo_archived_topics.sql");
pub const M_049_FIX_TYPE_MISMATCHES_UP: &str = include_str!("049_fix_type_mismatches.sql");
pub const M_050_ADD_ISSUE_PR_FEATURES_UP: &str = include_str!("050_add_issue_pr_features.sql");
pub const M_052_ADD_PROFILE_UP: &str = include_str!("052_add_profile.sql");
pub const M_053_ADD_OIDC_UP: &str = include_str!("053_add_oidc.sql");
pub const M_055_ADD_SITE_SETTINGS_UP: &str = include_str!("055_add_site_settings.sql");
pub const M_056_ADD_OIDC_ADMIN_UP: &str = include_str!("056_add_oidc_admin.sql");
pub const M_058_WEBAUTHN_UP: &str = include_str!("058_webauthn.sql");
pub const M_060_OAUTH2_UP: &str = include_str!("060_add_oauth2.sql");
pub const M_061_ISSUE_TEMPLATES_UP: &str = include_str!("061_add_issue_templates.sql");
pub const M_051_ENVIRONMENTS_DEPLOYMENTS_UP: &str =
    include_str!("051_add_environments_deployments.sql");
pub const M_054_MERGE_QUEUE_UP: &str = include_str!("054_add_merge_queue.sql");
pub const M_057_CODEOWNERS_REVIEWS_UP: &str = include_str!("057_add_codeowners_reviews.sql");

pub const M_040_BOARDS_UP: &str = include_str!("040_add_boards.sql");
pub const M_041_BOARDS_DOWN: &str = include_str!("down/041_add_boards_down.sql");
pub const M_041_WEBHOOKS_UP: &str = include_str!("041_add_webhooks.sql");
pub const M_039_SECRET_SCANNING_SLSA_UP: &str =
    include_str!("039_add_secret_scanning_slsa_tables.sql");
pub const M_039_SECRET_SCANNING_SLSA_DOWN: &str =
    include_str!("down/040_add_secret_scanning_slsa_tables_down.sql");
pub const M_042_WEBHOOK_DELIVERIES_UP: &str = include_str!("042_add_webhook_deliveries.sql");
pub const M_042_WEBHOOK_DELIVERIES_DOWN: &str =
    include_str!("down/042_add_webhook_deliveries_down.sql");

#[derive(Debug, Clone)]
pub struct Migration {
    pub version: i64,
    pub name: String,
    pub up_sql: String,
    pub down_sql: String,
}

#[derive(Debug, Default)]
pub struct MigrationManager {
    migrations: Vec<Migration>,
}

impl MigrationManager {
    pub fn new() -> Self {
        let mut mgr = Self {
            migrations: Vec::new(),
        };
        mgr.register_builtins();
        mgr
    }

    fn register_builtins(&mut self) {
        self.add_migration(Migration {
            version: 1,
            name: "initial_schema".into(),
            up_sql: M_001_INITIAL_SCHEMA_UP.into(),
            down_sql: M_001_INITIAL_SCHEMA_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 3,
            name: "add_ssh_keys_branches_steps_events".into(),
            up_sql: M_003_PHASE1_UP.into(),
            down_sql: M_003_PHASE1_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 5,
            name: "add_auth_identity_tables".into(),
            up_sql: M_005_AUTH_UP.into(),
            down_sql: M_005_AUTH_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 7,
            name: "add_permissions_tables".into(),
            up_sql: M_007_PERMISSIONS_UP.into(),
            down_sql: M_007_PERMISSIONS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 9,
            name: "add_ci_cd_pipeline_tables".into(),
            up_sql: M_009_PIPELINE_UP.into(),
            down_sql: M_009_PIPELINE_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 11,
            name: "add_oci_registry_tables".into(),
            up_sql: M_011_OCI_REGISTRY_UP.into(),
            down_sql: M_011_OCI_REGISTRY_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 13,
            name: "add_issue_tracking_tables".into(),
            up_sql: M_013_ISSUES_UP.into(),
            down_sql: M_013_ISSUES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 15,
            name: "add_wiki_tables".into(),
            up_sql: M_015_WIKI_UP.into(),
            down_sql: M_015_WIKI_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 17,
            name: "add_code_search_tables".into(),
            up_sql: M_017_SEARCH_UP.into(),
            down_sql: M_017_SEARCH_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 19,
            name: "add_wiki_content_snapshot".into(),
            up_sql: M_019_WIKI_SNAPSHOT_UP.into(),
            down_sql: M_019_WIKI_SNAPSHOT_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 21,
            name: "add_fulltext_search".into(),
            up_sql: M_021_FTS_UP.into(),
            down_sql: M_021_FTS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 23,
            name: "drop_wiki_created_by_fk".into(),
            up_sql: M_023_WIKI_FK_UP.into(),
            down_sql: M_023_WIKI_FK_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 25,
            name: "add_activity_federation".into(),
            up_sql: M_025_ACTIVITY_FED_UP.into(),
            down_sql: M_025_ACTIVITY_FED_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 27,
            name: "add_wiki_git_enabled".into(),
            up_sql: M_027_WIKI_GIT_UP.into(),
            down_sql: M_027_WIKI_GIT_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 29,
            name: "add_password_hash".into(),
            up_sql: M_029_PASSWORD_HASH_UP.into(),
            down_sql: String::new(),
        });
        self.add_migration(Migration {
            version: 31,
            name: "add_pr_tracking".into(),
            up_sql: M_031_PR_TRACKING_UP.into(),
            down_sql: M_031_PR_TRACKING_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 33,
            name: "add_star_watch_counts".into(),
            up_sql: M_033_STAR_WATCH_COUNTS_UP.into(),
            down_sql: M_033_STAR_WATCH_COUNTS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 35,
            name: "add_login_attempts".into(),
            up_sql: M_035_LOGIN_ATTEMPTS_UP.into(),
            down_sql: "DROP TABLE IF EXISTS login_attempts;".into(),
        });
        self.add_migration(Migration {
            version: 36,
            name: "add_email_verification".into(),
            up_sql: M_036_EMAIL_VERIFICATION_UP.into(),
            down_sql: "ALTER TABLE users DROP COLUMN IF EXISTS email_verified; DROP TABLE IF EXISTS email_verification_codes;".into(),
        });
        self.add_migration(Migration {
            version: 37,
            name: "add_mentions_and_crossrefs".into(),
            up_sql: M_037_MENTIONS_XREFS_UP.into(),
            down_sql: "DROP TABLE IF EXISTS comment_mentions; DROP TABLE IF EXISTS comment_cross_references;".into(),
        });
        self.add_migration(Migration {
            version: 38,
            name: "add_repo_secrets_and_pipeline_caches".into(),
            up_sql: M_038_REPO_SECRETS_CACHES_UP.into(),
            down_sql: M_038_REPO_SECRETS_CACHES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 39,
            name: "add_secret_scanning_slsa".into(),
            up_sql: M_039_SECRET_SCANNING_SLSA_UP.into(),
            down_sql: String::new(),
        });
        self.add_migration(Migration {
            version: 40,
            name: "add_boards".into(),
            up_sql: M_040_BOARDS_UP.into(),
            down_sql: M_041_BOARDS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 41,
            name: "add_webhooks".into(),
            up_sql: M_041_WEBHOOKS_UP.into(),
            down_sql: String::new(),
        });
        self.add_migration(Migration {
            version: 42,
            name: "add_webhook_deliveries".into(),
            up_sql: M_042_WEBHOOK_DELIVERIES_UP.into(),
            down_sql: M_042_WEBHOOK_DELIVERIES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 43,
            name: "add_pipeline_schedules".into(),
            up_sql: M_043_PIPELINE_SCHEDULES_UP.into(),
            down_sql: String::new(),
        });
        self.add_migration(Migration {
            version: 44,
            name: "add_repo_collaborators".into(),
            up_sql: M_044_REPO_COLLABORATORS_UP.into(),
            down_sql: "DROP TABLE IF EXISTS repo_collaborators;".into(),
        });
        self.add_migration(Migration {
            version: 45,
            name: "add_deploy_keys".into(),
            up_sql: M_045_DEPLOY_KEYS_UP.into(),
            down_sql: "DROP TABLE IF EXISTS deploy_keys;".into(),
        });
        self.add_migration(Migration {
            version: 46,
            name: "add_notifications".into(),
            up_sql: M_046_NOTIFICATIONS_UP.into(),
            down_sql: "DROP TABLE IF EXISTS notifications;".into(),
        });
        self.add_migration(Migration {
            version: 47,
            name: "add_user_banned".into(),
            up_sql: M_047_ADD_USER_BANNED_UP.into(),
            down_sql: "ALTER TABLE users DROP COLUMN IF EXISTS banned;".into(),
        });
        self.add_migration(Migration {
            version: 48,
            name: "add_repo_archived_topics".into(),
            up_sql: M_048_ADD_REPO_ARCHIVED_TOPICS_UP.into(),
            down_sql: "ALTER TABLE repositories DROP COLUMN IF EXISTS archived; ALTER TABLE repositories DROP COLUMN IF EXISTS topics;".into(),
        });
        self.add_migration(Migration {
            version: 49,
            name: "fix_type_mismatches".into(),
            up_sql: M_049_FIX_TYPE_MISMATCHES_UP.into(),
            down_sql: String::new(),
        });
        self.add_migration(Migration {
            version: 50,
            name: "add_issue_pr_features".into(),
            up_sql: M_050_ADD_ISSUE_PR_FEATURES_UP.into(),
            down_sql: String::new(),
        });
        self.add_migration(Migration {
            version: 51,
            name: "add_environments_deployments".into(),
            up_sql: M_051_ENVIRONMENTS_DEPLOYMENTS_UP.into(),
            down_sql: "DROP TABLE IF EXISTS deployments; DROP TABLE IF EXISTS environments;".into(),
        });
        self.add_migration(Migration {
            version: 52,
            name: "add_profile".into(),
            up_sql: M_052_ADD_PROFILE_UP.into(),
            down_sql: "ALTER TABLE users DROP COLUMN IF EXISTS avatar_url; ALTER TABLE users DROP COLUMN IF EXISTS location; ALTER TABLE users DROP COLUMN IF EXISTS website;".into(),
        });
        self.add_migration(Migration {
            version: 53,
            name: "add_oidc".into(),
            up_sql: M_053_ADD_OIDC_UP.into(),
            down_sql: "DROP TABLE IF EXISTS oidc_providers; DROP TABLE IF EXISTS oidc_identities;".into(),
        });
        self.add_migration(Migration {
            version: 54,
            name: "add_merge_queue".into(),
            up_sql: M_054_MERGE_QUEUE_UP.into(),
            down_sql: "DROP TABLE IF EXISTS merge_queue;".into(),
        });
        self.add_migration(Migration {
            version: 55,
            name: "add_site_settings".into(),
            up_sql: M_055_ADD_SITE_SETTINGS_UP.into(),
            down_sql: "DROP TABLE IF EXISTS site_settings;".into(),
        });
        self.add_migration(Migration {
            version: 56,
            name: "add_oidc_admin".into(),
            up_sql: M_056_ADD_OIDC_ADMIN_UP.into(),
            down_sql: String::new(),
        });
        self.add_migration(Migration {
            version: 57,
            name: "add_codeowners_reviews".into(),
            up_sql: M_057_CODEOWNERS_REVIEWS_UP.into(),
            down_sql: "DROP TABLE IF EXISTS codeowners_reviews;".into(),
        });
        self.add_migration(Migration {
            version: 58,
            name: "webauthn".into(),
            up_sql: M_058_WEBAUTHN_UP.into(),
            down_sql: "DROP TABLE IF EXISTS webauthn_credentials;".into(),
        });
        self.add_migration(Migration {
            version: 60,
            name: "add_oauth2".into(),
            up_sql: M_060_OAUTH2_UP.into(),
            down_sql: "DROP TABLE IF EXISTS oauth_codes; DROP TABLE IF EXISTS oauth_clients;".into(),
        });
        self.add_migration(Migration {
            version: 61,
            name: "add_issue_templates".into(),
            up_sql: M_061_ISSUE_TEMPLATES_UP.into(),
            down_sql: "DROP TABLE IF EXISTS issue_templates;".into(),
        });
    }

    pub fn add_migration(&mut self, migration: Migration) {
        if let Some(last) = self.migrations.last() {
            assert!(
                migration.version > last.version,
                "migration version {} must be greater than last version {}",
                migration.version,
                last.version,
            );
        }
        self.migrations.push(migration);
    }

    pub fn all(&self) -> &[Migration] {
        &self.migrations
    }

    pub fn get_pending(&self, db_version: i64) -> Vec<&Migration> {
        self.migrations
            .iter()
            .filter(|m| m.version > db_version)
            .collect()
    }
}

impl std::ops::Deref for MigrationManager {
    type Target = [Migration];

    fn deref(&self) -> &Self::Target {
        &self.migrations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_manager_has_initial_migration() {
        let mgr = MigrationManager::new();
        assert_eq!(mgr.all().len(), 42);
        assert_eq!(mgr.all()[0].version, 1);
        assert_eq!(mgr.all()[0].name, "initial_schema");
        assert_eq!(mgr.all()[1].version, 3);
        assert_eq!(mgr.all()[1].name, "add_ssh_keys_branches_steps_events");
        assert_eq!(mgr.all()[2].version, 5);
        assert_eq!(mgr.all()[2].name, "add_auth_identity_tables");
        assert_eq!(mgr.all()[3].version, 7);
        assert_eq!(mgr.all()[3].name, "add_permissions_tables");
        assert_eq!(mgr.all()[4].version, 9);
        assert_eq!(mgr.all()[4].name, "add_ci_cd_pipeline_tables");
        assert_eq!(mgr.all()[5].version, 11);
        assert_eq!(mgr.all()[5].name, "add_oci_registry_tables");
        assert_eq!(mgr.all()[6].version, 13);
        assert_eq!(mgr.all()[6].name, "add_issue_tracking_tables");
        assert_eq!(mgr.all()[7].version, 15);
        assert_eq!(mgr.all()[7].name, "add_wiki_tables");
        assert_eq!(mgr.all()[8].version, 17);
        assert_eq!(mgr.all()[8].name, "add_code_search_tables");
        assert_eq!(mgr.all()[9].version, 19);
        assert_eq!(mgr.all()[9].name, "add_wiki_content_snapshot");
        assert_eq!(mgr.all()[10].version, 21);
        assert_eq!(mgr.all()[10].name, "add_fulltext_search");
        assert_eq!(mgr.all()[11].version, 23);
        assert_eq!(mgr.all()[11].name, "drop_wiki_created_by_fk");
        assert_eq!(mgr.all()[12].version, 25);
        assert_eq!(mgr.all()[12].name, "add_activity_federation");
        assert_eq!(mgr.all()[13].version, 27);
        assert_eq!(mgr.all()[13].name, "add_wiki_git_enabled");
        assert_eq!(mgr.all()[14].version, 29);
        assert_eq!(mgr.all()[14].name, "add_password_hash");
        assert_eq!(mgr.all()[15].version, 31);
        assert_eq!(mgr.all()[15].name, "add_pr_tracking");
        assert_eq!(mgr.all()[16].version, 33);
        assert_eq!(mgr.all()[16].name, "add_star_watch_counts");
        assert_eq!(mgr.all()[17].version, 35);
        assert_eq!(mgr.all()[17].name, "add_login_attempts");
        assert_eq!(mgr.all()[18].version, 36);
        assert_eq!(mgr.all()[18].name, "add_email_verification");
        assert_eq!(mgr.all()[19].version, 37);
        assert_eq!(mgr.all()[19].name, "add_mentions_and_crossrefs");
        assert_eq!(mgr.all()[20].version, 38);
        assert_eq!(mgr.all()[20].name, "add_repo_secrets_and_pipeline_caches");
        assert_eq!(mgr.all()[21].version, 39);
        assert_eq!(mgr.all()[21].name, "add_secret_scanning_slsa");
        assert_eq!(mgr.all()[22].version, 40);
        assert_eq!(mgr.all()[22].name, "add_boards");
        assert_eq!(mgr.all()[23].version, 41);
        assert_eq!(mgr.all()[23].name, "add_webhooks");
        assert_eq!(mgr.all()[24].version, 42);
        assert_eq!(mgr.all()[24].name, "add_webhook_deliveries");
        assert_eq!(mgr.all()[25].version, 43);
        assert_eq!(mgr.all()[25].name, "add_pipeline_schedules");
        assert_eq!(mgr.all()[26].version, 44);
        assert_eq!(mgr.all()[26].name, "add_repo_collaborators");
        assert_eq!(mgr.all()[27].version, 45);
        assert_eq!(mgr.all()[27].name, "add_deploy_keys");
        assert_eq!(mgr.all()[28].version, 46);
        assert_eq!(mgr.all()[28].name, "add_notifications");
        assert_eq!(mgr.all()[29].version, 47);
        assert_eq!(mgr.all()[29].name, "add_user_banned");
        assert_eq!(mgr.all()[30].version, 48);
        assert_eq!(mgr.all()[30].name, "add_repo_archived_topics");
        assert_eq!(mgr.all()[31].version, 49);
        assert_eq!(mgr.all()[31].name, "fix_type_mismatches");
        assert_eq!(mgr.all()[32].version, 50);
        assert_eq!(mgr.all()[32].name, "add_issue_pr_features");
        assert_eq!(mgr.all()[33].version, 51);
        assert_eq!(mgr.all()[33].name, "add_environments_deployments");
        assert_eq!(mgr.all()[34].version, 52);
        assert_eq!(mgr.all()[34].name, "add_profile");
        assert_eq!(mgr.all()[35].version, 53);
        assert_eq!(mgr.all()[35].name, "add_oidc");
        assert_eq!(mgr.all()[36].version, 54);
        assert_eq!(mgr.all()[36].name, "add_merge_queue");
        assert_eq!(mgr.all()[37].version, 55);
        assert_eq!(mgr.all()[37].name, "add_site_settings");
        assert_eq!(mgr.all()[38].version, 56);
        assert_eq!(mgr.all()[38].name, "add_oidc_admin");
        assert_eq!(mgr.all()[39].version, 57);
        assert_eq!(mgr.all()[39].name, "add_codeowners_reviews");
        assert_eq!(mgr.all()[40].version, 58);
        assert_eq!(mgr.all()[40].name, "webauthn");
        assert_eq!(mgr.all()[41].version, 61);
        assert_eq!(mgr.all()[41].name, "add_issue_templates");
    }

    #[test]
    fn test_add_migration_sequential() {
        let mut mgr = MigrationManager::new();
        mgr.add_migration(Migration {
            version: 62,
            name: "add_index".into(),
            up_sql: "CREATE INDEX test;".into(),
            down_sql: "DROP INDEX test;".into(),
        });
        assert_eq!(mgr.all().len(), 43);
        assert_eq!(mgr.all()[42].version, 62);
    }

    #[test]
    #[should_panic(expected = "must be greater")]
    fn test_add_migration_out_of_order_panics() {
        let mut mgr = MigrationManager::new();
        mgr.add_migration(Migration {
            version: 0,
            name: "bad".into(),
            up_sql: "".into(),
            down_sql: "".into(),
        });
    }

    #[test]
    fn test_get_pending_none_applied() {
        let mgr = MigrationManager::new();
        let pending = mgr.get_pending(0);
        assert_eq!(pending.len(), 42);
    }

    #[test]
    fn test_get_pending_all_applied() {
        let mgr = MigrationManager::new();
        let pending = mgr.get_pending(61);
        assert_eq!(pending.len(), 0);
    }

    #[test]
    fn test_get_pending_partial() {
        let mgr = MigrationManager::new();
        let pending = mgr.get_pending(1);
        assert_eq!(pending.len(), 41);
    }

    #[test]
    fn test_initial_schema_sql_not_empty() {
        assert_ne!(M_001_INITIAL_SCHEMA_UP, "");
        assert!(M_001_INITIAL_SCHEMA_UP.contains("CREATE TABLE IF NOT EXISTS users"));
        assert!(M_001_INITIAL_SCHEMA_UP.contains("CREATE TABLE IF NOT EXISTS schema_migrations"));
    }

    #[test]
    fn test_initial_schema_down_sql_not_empty() {
        assert_ne!(M_001_INITIAL_SCHEMA_DOWN, "");
        assert!(M_001_INITIAL_SCHEMA_DOWN.contains("DROP TABLE IF EXISTS"));
    }

    #[test]
    fn test_phase1_schema_sql_not_empty() {
        assert_ne!(M_003_PHASE1_UP, "");
        assert!(M_003_PHASE1_UP.contains("CREATE TABLE IF NOT EXISTS sessions"));
        assert!(M_003_PHASE1_UP.contains("CREATE TABLE IF NOT EXISTS ssh_keys"));
        assert!(M_003_PHASE1_UP.contains("CREATE TABLE IF NOT EXISTS branches"));
        assert!(M_003_PHASE1_UP.contains("CREATE TABLE IF NOT EXISTS pipeline_steps"));
        assert!(M_003_PHASE1_UP.contains("CREATE TABLE IF NOT EXISTS event_log"));
    }

    #[test]
    fn test_phase1_schema_down_sql_not_empty() {
        assert_ne!(M_003_PHASE1_DOWN, "");
        assert!(M_003_PHASE1_DOWN.contains("DROP TABLE IF EXISTS"));
    }

    #[test]
    fn test_auth_schema_sql_not_empty() {
        assert_ne!(M_005_AUTH_UP, "");
        assert!(M_005_AUTH_UP.contains("CREATE TABLE IF NOT EXISTS oidc_identities"));
        assert!(M_005_AUTH_UP.contains("CREATE TABLE IF NOT EXISTS webauthn_credentials"));
        assert!(M_005_AUTH_UP.contains("CREATE TABLE IF NOT EXISTS devices"));
        assert!(M_005_AUTH_UP.contains("CREATE TABLE IF NOT EXISTS refresh_tokens"));
    }

    #[test]
    fn test_pipeline_schema_sql_not_empty() {
        assert_ne!(M_009_PIPELINE_UP, "");
        assert!(M_009_PIPELINE_UP.contains("CREATE TABLE IF NOT EXISTS runners"));
        assert!(M_009_PIPELINE_UP.contains("CREATE TABLE IF NOT EXISTS pipeline_definitions"));
        assert!(M_009_PIPELINE_UP.contains("CREATE TABLE IF NOT EXISTS pipeline_jobs"));
        assert!(M_009_PIPELINE_UP.contains("CREATE TABLE IF NOT EXISTS pipeline_job_steps"));
        assert!(M_009_PIPELINE_UP.contains("CREATE TABLE IF NOT EXISTS pipeline_runs"));
        assert!(M_009_PIPELINE_UP.contains("CREATE TABLE IF NOT EXISTS pipeline_run_jobs"));
        assert!(M_009_PIPELINE_UP.contains("CREATE TABLE IF NOT EXISTS pipeline_run_steps"));
    }

    #[test]
    fn test_pipeline_schema_down_sql_not_empty() {
        assert_ne!(M_009_PIPELINE_DOWN, "");
        assert!(M_009_PIPELINE_DOWN.contains("DROP TABLE IF EXISTS"));
    }

    #[test]
    fn test_oci_registry_schema_sql_not_empty() {
        assert_ne!(M_011_OCI_REGISTRY_UP, "");
        assert!(M_011_OCI_REGISTRY_UP.contains("CREATE TABLE IF NOT EXISTS oci_repositories"));
        assert!(M_011_OCI_REGISTRY_UP.contains("CREATE TABLE IF NOT EXISTS oci_blobs"));
        assert!(M_011_OCI_REGISTRY_UP.contains("CREATE TABLE IF NOT EXISTS oci_manifests"));
        assert!(M_011_OCI_REGISTRY_UP.contains("CREATE TABLE IF NOT EXISTS oci_tags"));
        assert!(M_011_OCI_REGISTRY_UP.contains("CREATE TABLE IF NOT EXISTS oci_manifest_layers"));
        assert!(M_011_OCI_REGISTRY_UP.contains("CREATE TABLE IF NOT EXISTS oci_image_signatures"));
        assert!(M_011_OCI_REGISTRY_UP.contains("CREATE TABLE IF NOT EXISTS oci_vuln_scans"));
        assert!(M_011_OCI_REGISTRY_UP.contains("CREATE TABLE IF NOT EXISTS oci_policies"));
    }

    #[test]
    fn test_oci_registry_down_sql_not_empty() {
        assert_ne!(M_011_OCI_REGISTRY_DOWN, "");
        assert!(M_011_OCI_REGISTRY_DOWN.contains("DROP TABLE IF EXISTS"));
    }

    #[test]
    fn test_issues_schema_sql_not_empty() {
        assert_ne!(M_013_ISSUES_UP, "");
        assert!(M_013_ISSUES_UP.contains("CREATE TABLE IF NOT EXISTS issue_comments"));
        assert!(M_013_ISSUES_UP.contains("CREATE TABLE IF NOT EXISTS labels"));
        assert!(M_013_ISSUES_UP.contains("CREATE TABLE IF NOT EXISTS issue_labels"));
        assert!(M_013_ISSUES_UP.contains("CREATE TABLE IF NOT EXISTS issue_assignees"));
        assert!(M_013_ISSUES_UP.contains("CREATE TABLE IF NOT EXISTS milestones"));
        assert!(M_013_ISSUES_UP.contains("CREATE TABLE IF NOT EXISTS issue_timeline"));
        assert!(M_013_ISSUES_UP.contains("CREATE TABLE IF NOT EXISTS issue_reactions"));
    }

    #[test]
    fn test_issues_down_sql_not_empty() {
        assert_ne!(M_013_ISSUES_DOWN, "");
        assert!(M_013_ISSUES_DOWN.contains("DROP TABLE IF EXISTS"));
    }

    #[test]
    fn test_wiki_schema_sql_not_empty() {
        assert_ne!(M_015_WIKI_UP, "");
        assert!(M_015_WIKI_UP.contains("CREATE TABLE IF NOT EXISTS wiki_pages"));
        assert!(M_015_WIKI_UP.contains("CREATE TABLE IF NOT EXISTS wiki_revisions"));
        assert!(M_015_WIKI_UP.contains("content"));
        assert!(M_015_WIKI_UP.contains("latest_commit"));
        assert!(M_015_WIKI_UP.contains("idx_wiki_pages_repo"));
        assert!(M_015_WIKI_UP.contains("idx_wiki_revisions_page"));
    }

    #[test]
    fn test_wiki_down_sql_not_empty() {
        assert_ne!(M_015_WIKI_DOWN, "");
        assert!(M_015_WIKI_DOWN.contains("DROP TABLE IF EXISTS wiki_revisions"));
        assert!(M_015_WIKI_DOWN.contains("DROP TABLE IF EXISTS wiki_pages"));
    }

    #[test]
    fn test_search_schema_sql_not_empty() {
        assert_ne!(M_017_SEARCH_UP, "");
        assert!(M_017_SEARCH_UP.contains("CREATE TABLE IF NOT EXISTS code_search_index"));
        assert!(M_017_SEARCH_UP.contains("CREATE TABLE IF NOT EXISTS code_search_tokens"));
        assert!(M_017_SEARCH_UP.contains("idx_code_search_repo"));
        assert!(M_017_SEARCH_UP.contains("idx_code_tokens_token"));
    }

    #[test]
    fn test_search_down_sql_not_empty() {
        assert_ne!(M_017_SEARCH_DOWN, "");
        assert!(M_017_SEARCH_DOWN.contains("DROP TABLE IF EXISTS code_search_tokens"));
        assert!(M_017_SEARCH_DOWN.contains("DROP TABLE IF EXISTS code_search_index"));
    }

    #[test]
    fn test_webhook_deliveries_sql_not_empty() {
        assert_ne!(M_042_WEBHOOK_DELIVERIES_UP, "");
        assert!(
            M_042_WEBHOOK_DELIVERIES_UP.contains("CREATE TABLE IF NOT EXISTS webhook_deliveries")
        );
        assert!(M_042_WEBHOOK_DELIVERIES_UP.contains("webhook_id"));
        assert!(M_042_WEBHOOK_DELIVERIES_UP.contains("event"));
        assert!(M_042_WEBHOOK_DELIVERIES_UP.contains("payload"));
        assert!(M_042_WEBHOOK_DELIVERIES_UP.contains("status"));
        assert!(M_042_WEBHOOK_DELIVERIES_UP.contains("attempts"));
        assert!(M_042_WEBHOOK_DELIVERIES_UP.contains("last_error"));
        assert!(M_042_WEBHOOK_DELIVERIES_UP.contains("next_retry_at"));
        assert!(M_042_WEBHOOK_DELIVERIES_UP.contains("created_at"));
        assert!(M_042_WEBHOOK_DELIVERIES_UP.contains("idx_webhook_deliveries_webhook_id"));
        assert!(M_042_WEBHOOK_DELIVERIES_UP.contains("idx_webhook_deliveries_status_retry"));
    }

    #[test]
    fn test_webhook_deliveries_down_sql_not_empty() {
        assert_ne!(M_042_WEBHOOK_DELIVERIES_DOWN, "");
        assert!(M_042_WEBHOOK_DELIVERIES_DOWN.contains("DROP TABLE IF EXISTS webhook_deliveries"));
    }
}
