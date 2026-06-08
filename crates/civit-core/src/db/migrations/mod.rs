#![forbid(unsafe_code)]

pub const M_001_INITIAL_SCHEMA_UP: &str = include_str!("001_initial_schema.sql");
pub const M_001_INITIAL_SCHEMA_DOWN: &str = include_str!("002_initial_schema_down.sql");
pub const M_003_PHASE1_UP: &str = include_str!("003_add_ssh_keys_branches_steps_events.sql");
pub const M_003_PHASE1_DOWN: &str = include_str!("004_add_ssh_keys_branches_steps_events_down.sql");
pub const M_005_AUTH_UP: &str = include_str!("005_add_auth_identity_tables.sql");
pub const M_005_AUTH_DOWN: &str = include_str!("006_add_auth_identity_tables_down.sql");
pub const M_007_PERMISSIONS_UP: &str = include_str!("007_add_permissions_tables.sql");
pub const M_007_PERMISSIONS_DOWN: &str = include_str!("007_add_permissions_tables_down.sql");
pub const M_009_PIPELINE_UP: &str = include_str!("009_add_ci_cd_pipeline_tables.sql");
pub const M_009_PIPELINE_DOWN: &str = include_str!("010_add_ci_cd_pipeline_tables_down.sql");
pub const M_011_OCI_REGISTRY_UP: &str = include_str!("011_add_oci_registry_tables.sql");
pub const M_011_OCI_REGISTRY_DOWN: &str = include_str!("012_add_oci_registry_tables_down.sql");
pub const M_013_ISSUES_UP: &str = include_str!("013_add_issue_tracking_tables.sql");
pub const M_013_ISSUES_DOWN: &str = include_str!("014_add_issue_tracking_tables_down.sql");
pub const M_015_WIKI_UP: &str = include_str!("015_add_wiki_tables.sql");
pub const M_015_WIKI_DOWN: &str = include_str!("016_add_wiki_tables_down.sql");
pub const M_017_SEARCH_UP: &str = include_str!("017_add_code_search_tables.sql");
pub const M_017_SEARCH_DOWN: &str = include_str!("018_add_code_search_tables_down.sql");
pub const M_019_WIKI_SNAPSHOT_UP: &str = include_str!("019_add_wiki_content_snapshot.sql");
pub const M_019_WIKI_SNAPSHOT_DOWN: &str = include_str!("020_add_wiki_content_snapshot_down.sql");
pub const M_021_FTS_UP: &str = include_str!("021_add_fulltext_search.sql");
pub const M_021_FTS_DOWN: &str = include_str!("022_add_fulltext_search_down.sql");
pub const M_023_WIKI_FK_UP: &str = include_str!("023_drop_wiki_created_by_fk.sql");
pub const M_023_WIKI_FK_DOWN: &str = include_str!("024_drop_wiki_created_by_fk_down.sql");
pub const M_025_ACTIVITY_FED_UP: &str = include_str!("025_add_activity_federation.sql");
pub const M_025_ACTIVITY_FED_DOWN: &str = include_str!("026_add_activity_federation_down.sql");
pub const M_027_WIKI_GIT_UP: &str = include_str!("027_add_wiki_git_enabled.sql");
pub const M_027_WIKI_GIT_DOWN: &str = include_str!("028_add_wiki_git_enabled_down.sql");
pub const M_029_PASSWORD_HASH_UP: &str = include_str!("029_add_password_hash.sql");
pub const M_031_PR_TRACKING_UP: &str = include_str!("031_add_pr_tracking.sql");
pub const M_031_PR_TRACKING_DOWN: &str = include_str!("032_add_pr_tracking_down.sql");
pub const M_033_STAR_WATCH_COUNTS_UP: &str = include_str!("033_add_star_watch_counts.sql");
pub const M_033_STAR_WATCH_COUNTS_DOWN: &str = include_str!("034_add_star_watch_counts_down.sql");
pub const M_035_LOGIN_ATTEMPTS_UP: &str = include_str!("035_add_login_attempts.sql");

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
        assert_eq!(mgr.all().len(), 18);
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
    }

    #[test]
    fn test_add_migration_sequential() {
        let mut mgr = MigrationManager::new();
        mgr.add_migration(Migration {
            version: 37,
            name: "add_index".into(),
            up_sql: "CREATE INDEX test;".into(),
            down_sql: "DROP INDEX test;".into(),
        });
        assert_eq!(mgr.all().len(), 19);
        assert_eq!(mgr.all()[18].version, 37);
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
        assert_eq!(pending.len(), 18);
    }

    #[test]
    fn test_get_pending_all_applied() {
        let mgr = MigrationManager::new();
        let pending = mgr.get_pending(29);
        assert_eq!(pending.len(), 3);
    }

    #[test]
    fn test_get_pending_partial() {
        let mgr = MigrationManager::new();
        let pending = mgr.get_pending(1);
        assert_eq!(pending.len(), 17);
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
        // issues table already exists from migration 001, so 013 only adds the related tables
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
}
