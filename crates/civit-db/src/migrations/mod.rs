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
pub const M_062_PR_TEMPLATES_UP: &str = include_str!("062_add_pr_templates.sql");
pub const M_063_DISCUSSIONS_UP: &str = include_str!("063_add_discussions.sql");
pub const M_066_BOARDS_V2_UP: &str = include_str!("066_add_boards_v2.sql");
pub const M_051_ENVIRONMENTS_DEPLOYMENTS_UP: &str =
    include_str!("051_add_environments_deployments.sql");
pub const M_054_MERGE_QUEUE_UP: &str = include_str!("054_add_merge_queue.sql");
pub const M_057_CODEOWNERS_REVIEWS_UP: &str = include_str!("057_add_codeowners_reviews.sql");
pub const M_067_REVIEW_THREADS_UP: &str = include_str!("067_add_review_threads.sql");
pub const M_068_RATE_LIMITS_UP: &str = include_str!("068_add_rate_limits.sql");
pub const M_069_NPM_PACKAGES_UP: &str = include_str!("069_add_npm_packages.sql");
pub const M_070_MAVEN_PACKAGES_UP: &str = include_str!("070_add_maven_packages.sql");
pub const M_071_PAGES_SITES_UP: &str = include_str!("071_add_pages_sites.sql");
pub const M_072_DISCUSSION_LABELS_REACTIONS_UP: &str =
    include_str!("072_add_discussion_labels_reactions.sql");
pub const M_071_SEARCH_HISTORY_UP: &str = include_str!("071_add_search_history.sql");
pub const M_071_SEARCH_HISTORY_DOWN: &str = "DROP TABLE IF EXISTS search_history;";
pub const M_072_CODE_SUGGESTIONS_UP: &str = include_str!("072_add_code_suggestions.sql");
pub const M_072_CODE_SUGGESTIONS_DOWN: &str = "DROP TABLE IF EXISTS code_suggestions;";
pub const M_075_LICENSE_REPORTS_UP: &str = include_str!("075_add_license_reports.sql");
pub const M_075_LICENSE_REPORTS_DOWN: &str = "DROP TABLE IF EXISTS license_reports;";
pub const M_076_ENHANCE_AUDIT_LOG_UP: &str = include_str!("076_enhance_audit_log.sql");
pub const M_076_ENHANCE_AUDIT_LOG_DOWN: &str =
    "ALTER TABLE audit_events DROP COLUMN IF EXISTS request_id;";
pub const M_080_ADD_SAML_PROVIDERS_UP: &str = include_str!("080_add_saml_providers.sql");
pub const M_080_ADD_SAML_PROVIDERS_DOWN: &str = "DROP TABLE IF EXISTS saml_providers;";
pub const M_081_ADD_SCIM_TOKENS_UP: &str = include_str!("081_add_scim_tokens.sql");
pub const M_081_ADD_SCIM_TOKENS_DOWN: &str = "DROP TABLE IF EXISTS scim_tokens;";
pub const M_082_ADD_SSO_GROUPS_SESSIONS_LOGIN_HISTORY_UP: &str =
    include_str!("082_add_sso_groups_sessions_login_history.sql");
pub const M_082_ADD_SSO_GROUPS_SESSIONS_LOGIN_HISTORY_DOWN: &str =
    "DROP TABLE IF EXISTS login_history; DROP TABLE IF EXISTS active_sessions; DROP TABLE IF EXISTS sso_group_mappings;";
pub const M_077_ENHANCE_PAGES_SITES_UP: &str = include_str!("077_enhance_pages_sites.sql");
pub const M_077_ENHANCE_PAGES_SITES_DOWN: &str =
    "DROP TABLE IF EXISTS pages_deployments; ALTER TABLE pages_sites DROP COLUMN IF EXISTS custom_domain; ALTER TABLE pages_sites DROP COLUMN IF EXISTS https_enabled; ALTER TABLE pages_sites DROP COLUMN IF EXISTS last_built_at;";
pub const M_078_ENHANCE_DEPLOYMENT_PROTECTIONS_UP: &str =
    include_str!("078_enhance_deployment_protections.sql");
pub const M_078_ENHANCE_DEPLOYMENT_PROTECTIONS_DOWN: &str =
    "DROP TABLE IF EXISTS deployment_locks; ALTER TABLE deployment_protections DROP COLUMN IF EXISTS allowed_branches;";
pub const M_083_CONTAINER_REPO_POLICIES_UP: &str =
    include_str!("083_add_container_repository_policies.sql");
pub const M_083_CONTAINER_REPO_POLICIES_DOWN: &str =
    "DROP TABLE IF EXISTS container_pull_through_cache; DROP TABLE IF EXISTS container_image_signatures; DROP TABLE IF EXISTS container_vulnerability_scans; DROP TABLE IF EXISTS container_repository_policies;";
pub const M_084_OBSERVABILITY_TABLES_UP: &str =
    include_str!("084_add_observability_tables.sql");
pub const M_084_OBSERVABILITY_TABLES_DOWN: &str =
    "DROP TABLE IF EXISTS metrics; DROP TABLE IF EXISTS trace_spans;";
pub const M_085_PERFORMANCE_INDEXES_UP: &str =
    include_str!("085_add_performance_indexes.sql");
pub const M_085_PERFORMANCE_INDEXES_DOWN: &str = "DROP INDEX IF EXISTS idx_repositories_owner_id; DROP INDEX IF EXISTS idx_repositories_visibility; DROP INDEX IF EXISTS idx_issues_repo_id_status; DROP INDEX IF EXISTS idx_pull_requests_repo_id_status; DROP INDEX IF EXISTS idx_pipeline_runs_repo_id; DROP INDEX IF EXISTS idx_audit_events_created_at; DROP INDEX IF EXISTS idx_audit_events_user_id; DROP INDEX IF EXISTS idx_stars_user_id; DROP INDEX IF EXISTS idx_watchers_user_id; DROP INDEX IF EXISTS idx_comments_pr_id; DROP INDEX IF EXISTS idx_comments_issue_id;";
pub const M_086_CACHE_ENTRIES_UP: &str = include_str!("086_add_cache_entries.sql");
pub const M_086_CACHE_ENTRIES_DOWN: &str = "DROP TABLE IF EXISTS cache_entries;";
pub const M_087_CDN_CONFIG_UP: &str = include_str!("087_add_cdn_config.sql");
pub const M_087_CDN_CONFIG_DOWN: &str = "DROP TABLE IF EXISTS cdn_config;";
pub const M_088_SERVER_INSTANCES_UP: &str = include_str!("088_add_server_instances.sql");
pub const M_088_SERVER_INSTANCES_DOWN: &str =
    "DROP TABLE IF EXISTS sticky_sessions; DROP TABLE IF EXISTS server_instances;";
pub const M_089_WEBSOCKET_CONNECTIONS_UP: &str =
    include_str!("089_add_websocket_connections.sql");
pub const M_089_WEBSOCKET_CONNECTIONS_DOWN: &str = "DROP TABLE IF EXISTS websocket_connections;";
pub const M_090_POOL_CONFIG_UP: &str = include_str!("090_add_pool_config.sql");
pub const M_090_POOL_CONFIG_DOWN: &str = "DROP TABLE IF EXISTS pool_config;";
pub const M_091_FEATURE_FLAGS_UP: &str = include_str!("091_add_feature_flags.sql");
pub const M_091_FEATURE_FLAGS_DOWN: &str =
    include_str!("down/091_feature_flags_down.sql");
pub const M_092_ADMIN_DASHBOARD_CONFIG_UP: &str = include_str!("092_add_admin_dashboard_config.sql");
pub const M_092_ADMIN_DASHBOARD_CONFIG_DOWN: &str =
    include_str!("down/092_admin_dashboard_config_down.sql");
pub const M_093_API_ANALYTICS_UP: &str = include_str!("093_add_api_analytics.sql");
pub const M_093_API_ANALYTICS_DOWN: &str = "DROP TABLE IF EXISTS api_usage_summary; DROP TABLE IF EXISTS api_analytics;";
pub const M_094_USAGE_QUOTAS_UP: &str = include_str!("094_add_usage_quotas.sql");
pub const M_094_USAGE_QUOTAS_DOWN: &str = "DROP TABLE IF EXISTS usage_quotas;";
pub const M_095_EXPORT_JOBS_UP: &str = include_str!("095_add_export_jobs.sql");
pub const M_095_EXPORT_JOBS_DOWN: &str = "DROP TABLE IF EXISTS export_jobs;";
pub const M_096_COMPLIANCE_REPORTS_UP: &str = include_str!("096_add_compliance_reports.sql");
pub const M_096_COMPLIANCE_REPORTS_DOWN: &str = "DROP TABLE IF EXISTS compliance_reports;";
pub const M_097_DEPLOYMENT_HISTORY_UP: &str = include_str!("097_add_deployment_history.sql");
pub const M_097_DEPLOYMENT_HISTORY_DOWN: &str =
    "DROP TABLE IF EXISTS deployment_history;";
pub const M_098_MONITORING_ALERTS_UP: &str = include_str!("098_add_monitoring_alerts.sql");
pub const M_098_MONITORING_ALERTS_DOWN: &str =
    "DROP TABLE IF EXISTS monitoring_incidents; DROP TABLE IF EXISTS monitoring_alerts;";
pub const M_099_PERFORMANCE_METRICS_UP: &str = include_str!("099_add_performance_metrics.sql");
pub const M_099_PERFORMANCE_METRICS_DOWN: &str = "DROP TABLE IF EXISTS performance_metrics;";
pub const M_100_WEBHOOK_DELIVERIES_V2_UP: &str = include_str!("100_add_webhook_deliveries_v2.sql");
pub const M_100_WEBHOOK_DELIVERIES_V2_DOWN: &str = "DROP TABLE IF EXISTS webhook_deliveries_v2;";
pub const M_101_EVENTS_AND_SUBSCRIPTIONS_UP: &str = include_str!("101_add_events_and_subscriptions.sql");
pub const M_101_EVENTS_AND_SUBSCRIPTIONS_DOWN: &str = "DROP TABLE IF EXISTS event_subscriptions; DROP TABLE IF EXISTS events;";
pub const M_102_EVENT_QUEUES_UP: &str = include_str!("102_add_event_queues.sql");
pub const M_102_EVENT_QUEUES_DOWN: &str = "DROP TABLE IF EXISTS event_queue_messages; DROP TABLE IF EXISTS event_queues;";
pub const M_103_CHAOS_ENGINEERING_UP: &str = include_str!("103_add_chaos_engineering.sql");
pub const M_103_CHAOS_ENGINEERING_DOWN: &str = "DROP TABLE IF EXISTS chaos_results; DROP TABLE IF EXISTS chaos_experiments;";
pub const M_104_RESILIENCE_TESTS_UP: &str = include_str!("104_add_resilience_tests.sql");
pub const M_104_RESILIENCE_TESTS_DOWN: &str = "DROP TABLE IF EXISTS resilience_tests;";
pub const M_105_CIRCUIT_BREAKERS_UP: &str = include_str!("105_add_circuit_breakers.sql");
pub const M_105_CIRCUIT_BREAKERS_DOWN: &str = "DROP TABLE IF EXISTS circuit_breakers;";
pub const M_106_DISTRIBUTED_TRACES_UP: &str = include_str!("106_add_distributed_traces.sql");
pub const M_106_DISTRIBUTED_TRACES_DOWN: &str = "DROP TABLE IF EXISTS distributed_traces;";
pub const M_107_APM_TRANSACTIONS_SPANS_UP: &str = include_str!("107_add_apm_transactions_spans.sql");
pub const M_107_APM_TRANSACTIONS_SPANS_DOWN: &str = "DROP TABLE IF EXISTS apm_spans; DROP TABLE IF EXISTS apm_transactions;";
pub const M_108_ERROR_TRACKING_UP: &str = include_str!("108_add_error_tracking.sql");
pub const M_108_ERROR_TRACKING_DOWN: &str = "DROP TABLE IF EXISTS error_tracking;";
pub const M_109_API_GATEWAY_UP: &str = include_str!("109_add_api_gateway.sql");
pub const M_109_API_GATEWAY_DOWN: &str =
    "DROP TABLE IF EXISTS api_gateway_keys; DROP TABLE IF EXISTS api_gateway_routes;";
pub const M_110_RATE_LIMIT_POLICIES_UP: &str = include_str!("110_add_rate_limit_policies.sql");
pub const M_110_RATE_LIMIT_POLICIES_DOWN: &str =
    "DROP TABLE IF EXISTS rate_limit_buckets; DROP TABLE IF EXISTS rate_limit_policies;";
pub const M_111_API_TRANSFORMS_UP: &str = include_str!("111_add_api_transforms.sql");
pub const M_111_API_TRANSFORMS_DOWN: &str = "DROP TABLE IF EXISTS api_transforms;";
pub const M_112_GRAPHQL_SUBSCRIPTIONS_UP: &str = include_str!("112_add_graphql_subscriptions.sql");
pub const M_112_GRAPHQL_SUBSCRIPTIONS_DOWN: &str = "DROP TABLE IF EXISTS graphql_subscriptions;";
pub const M_113_REALTIME_CHANNELS_UP: &str = include_str!("113_add_realtime_channels.sql");
pub const M_113_REALTIME_CHANNELS_DOWN: &str = "DROP TABLE IF EXISTS realtime_messages; DROP TABLE IF EXISTS realtime_channels;";
pub const M_114_LIVE_COLLABORATION_UP: &str = include_str!("114_add_live_collaboration.sql");
pub const M_114_LIVE_COLLABORATION_DOWN: &str = "DROP TABLE IF EXISTS live_collaboration_sessions;";

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
            version: 62,
            name: "add_pr_templates".into(),
            up_sql: M_062_PR_TEMPLATES_UP.into(),
            down_sql: "DROP TABLE IF EXISTS pr_templates;".into(),
        });
        self.add_migration(Migration {
            version: 63,
            name: "add_discussions".into(),
            up_sql: M_063_DISCUSSIONS_UP.into(),
            down_sql: "DROP TABLE IF EXISTS discussion_comments; DROP TABLE IF EXISTS discussions;".into(),
        });
        self.add_migration(Migration {
            version: 66,
            name: "add_boards_v2".into(),
            up_sql: M_066_BOARDS_V2_UP.into(),
            down_sql: "DROP TABLE IF EXISTS board_card_assignees; DROP TABLE IF EXISTS board_card_labels; ALTER TABLE board_cards DROP COLUMN IF EXISTS priority; ALTER TABLE board_cards DROP COLUMN IF EXISTS due_date; ALTER TABLE board_cards DROP COLUMN IF EXISTS sort_order;".into(),
        });
        self.add_migration(Migration {
            version: 67,
            name: "add_review_threads".into(),
            up_sql: M_067_REVIEW_THREADS_UP.into(),
            down_sql: "DROP TABLE IF EXISTS pr_review_assignments; ALTER TABLE pr_comments DROP COLUMN IF EXISTS resolved; ALTER TABLE pr_comments DROP COLUMN IF EXISTS resolved_by;".into(),
        });
        self.add_migration(Migration {
            version: 68,
            name: "add_rate_limits".into(),
            up_sql: M_068_RATE_LIMITS_UP.into(),
            down_sql: "DROP TABLE IF EXISTS rate_limits;".into(),
        });
        self.add_migration(Migration {
            version: 69,
            name: "add_npm_packages".into(),
            up_sql: M_069_NPM_PACKAGES_UP.into(),
            down_sql: "DROP TABLE IF EXISTS npm_versions; DROP TABLE IF EXISTS npm_packages;".into(),
        });
        self.add_migration(Migration {
            version: 70,
            name: "add_maven_packages".into(),
            up_sql: M_070_MAVEN_PACKAGES_UP.into(),
            down_sql: "DROP TABLE IF EXISTS maven_packages;".into(),
        });
        self.add_migration(Migration {
            version: 71,
            name: "add_pages_sites".into(),
            up_sql: M_071_PAGES_SITES_UP.into(),
            down_sql: "DROP TABLE IF EXISTS pages_sites;".into(),
        });
        self.add_migration(Migration {
            version: 72,
            name: "add_discussion_labels_reactions".into(),
            up_sql: M_072_DISCUSSION_LABELS_REACTIONS_UP.into(),
            down_sql: "DROP TABLE IF EXISTS discussion_reactions; DROP TABLE IF EXISTS discussion_labels;".into(),
        });
        self.add_migration(Migration {
            version: 73,
            name: "add_search_history".into(),
            up_sql: M_071_SEARCH_HISTORY_UP.into(),
            down_sql: M_071_SEARCH_HISTORY_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 74,
            name: "add_code_suggestions".into(),
            up_sql: M_072_CODE_SUGGESTIONS_UP.into(),
            down_sql: M_072_CODE_SUGGESTIONS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 75,
            name: "add_license_reports".into(),
            up_sql: M_075_LICENSE_REPORTS_UP.into(),
            down_sql: M_075_LICENSE_REPORTS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 76,
            name: "enhance_audit_log".into(),
            up_sql: M_076_ENHANCE_AUDIT_LOG_UP.into(),
            down_sql: M_076_ENHANCE_AUDIT_LOG_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 77,
            name: "enhance_pages_sites".into(),
            up_sql: M_077_ENHANCE_PAGES_SITES_UP.into(),
            down_sql: M_077_ENHANCE_PAGES_SITES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 80,
            name: "add_saml_providers".into(),
            up_sql: M_080_ADD_SAML_PROVIDERS_UP.into(),
            down_sql: M_080_ADD_SAML_PROVIDERS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 81,
            name: "add_scim_tokens".into(),
            up_sql: M_081_ADD_SCIM_TOKENS_UP.into(),
            down_sql: M_081_ADD_SCIM_TOKENS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 82,
            name: "add_sso_groups_sessions_login_history".into(),
            up_sql: M_082_ADD_SSO_GROUPS_SESSIONS_LOGIN_HISTORY_UP.into(),
            down_sql: M_082_ADD_SSO_GROUPS_SESSIONS_LOGIN_HISTORY_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 83,
            name: "add_container_repository_policies".into(),
            up_sql: M_083_CONTAINER_REPO_POLICIES_UP.into(),
            down_sql: M_083_CONTAINER_REPO_POLICIES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 84,
            name: "add_observability_tables".into(),
            up_sql: M_084_OBSERVABILITY_TABLES_UP.into(),
            down_sql: M_084_OBSERVABILITY_TABLES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 85,
            name: "add_performance_indexes".into(),
            up_sql: M_085_PERFORMANCE_INDEXES_UP.into(),
            down_sql: M_085_PERFORMANCE_INDEXES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 86,
            name: "add_cache_entries".into(),
            up_sql: M_086_CACHE_ENTRIES_UP.into(),
            down_sql: M_086_CACHE_ENTRIES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 87,
            name: "add_cdn_config".into(),
            up_sql: M_087_CDN_CONFIG_UP.into(),
            down_sql: M_087_CDN_CONFIG_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 88,
            name: "add_server_instances".into(),
            up_sql: M_088_SERVER_INSTANCES_UP.into(),
            down_sql: M_088_SERVER_INSTANCES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 89,
            name: "add_websocket_connections".into(),
            up_sql: M_089_WEBSOCKET_CONNECTIONS_UP.into(),
            down_sql: M_089_WEBSOCKET_CONNECTIONS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 90,
            name: "add_pool_config".into(),
            up_sql: M_090_POOL_CONFIG_UP.into(),
            down_sql: M_090_POOL_CONFIG_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 91,
            name: "add_feature_flags".into(),
            up_sql: M_091_FEATURE_FLAGS_UP.into(),
            down_sql: M_091_FEATURE_FLAGS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 92,
            name: "add_admin_dashboard_config".into(),
            up_sql: M_092_ADMIN_DASHBOARD_CONFIG_UP.into(),
            down_sql: M_092_ADMIN_DASHBOARD_CONFIG_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 93,
            name: "add_api_analytics".into(),
            up_sql: M_093_API_ANALYTICS_UP.into(),
            down_sql: M_093_API_ANALYTICS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 94,
            name: "add_usage_quotas".into(),
            up_sql: M_094_USAGE_QUOTAS_UP.into(),
            down_sql: M_094_USAGE_QUOTAS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 95,
            name: "add_export_jobs".into(),
            up_sql: M_095_EXPORT_JOBS_UP.into(),
            down_sql: M_095_EXPORT_JOBS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 96,
            name: "add_compliance_reports".into(),
            up_sql: M_096_COMPLIANCE_REPORTS_UP.into(),
            down_sql: M_096_COMPLIANCE_REPORTS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 97,
            name: "add_deployment_history".into(),
            up_sql: M_097_DEPLOYMENT_HISTORY_UP.into(),
            down_sql: M_097_DEPLOYMENT_HISTORY_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 98,
            name: "add_monitoring_alerts".into(),
            up_sql: M_098_MONITORING_ALERTS_UP.into(),
            down_sql: M_098_MONITORING_ALERTS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 99,
            name: "add_performance_metrics".into(),
            up_sql: M_099_PERFORMANCE_METRICS_UP.into(),
            down_sql: M_099_PERFORMANCE_METRICS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 100,
            name: "add_webhook_deliveries_v2".into(),
            up_sql: M_100_WEBHOOK_DELIVERIES_V2_UP.into(),
            down_sql: M_100_WEBHOOK_DELIVERIES_V2_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 101,
            name: "add_events_and_subscriptions".into(),
            up_sql: M_101_EVENTS_AND_SUBSCRIPTIONS_UP.into(),
            down_sql: M_101_EVENTS_AND_SUBSCRIPTIONS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 102,
            name: "add_event_queues".into(),
            up_sql: M_102_EVENT_QUEUES_UP.into(),
            down_sql: M_102_EVENT_QUEUES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 103,
            name: "add_chaos_engineering".into(),
            up_sql: M_103_CHAOS_ENGINEERING_UP.into(),
            down_sql: M_103_CHAOS_ENGINEERING_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 104,
            name: "add_resilience_tests".into(),
            up_sql: M_104_RESILIENCE_TESTS_UP.into(),
            down_sql: M_104_RESILIENCE_TESTS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 105,
            name: "add_circuit_breakers".into(),
            up_sql: M_105_CIRCUIT_BREAKERS_UP.into(),
            down_sql: M_105_CIRCUIT_BREAKERS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 106,
            name: "add_distributed_traces".into(),
            up_sql: M_106_DISTRIBUTED_TRACES_UP.into(),
            down_sql: M_106_DISTRIBUTED_TRACES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 107,
            name: "add_apm_transactions_spans".into(),
            up_sql: M_107_APM_TRANSACTIONS_SPANS_UP.into(),
            down_sql: M_107_APM_TRANSACTIONS_SPANS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 108,
            name: "add_error_tracking".into(),
            up_sql: M_108_ERROR_TRACKING_UP.into(),
            down_sql: M_108_ERROR_TRACKING_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 109,
            name: "add_api_gateway".into(),
            up_sql: M_109_API_GATEWAY_UP.into(),
            down_sql: M_109_API_GATEWAY_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 110,
            name: "add_rate_limit_policies".into(),
            up_sql: M_110_RATE_LIMIT_POLICIES_UP.into(),
            down_sql: M_110_RATE_LIMIT_POLICIES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 111,
            name: "add_api_transforms".into(),
            up_sql: M_111_API_TRANSFORMS_UP.into(),
            down_sql: M_111_API_TRANSFORMS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 112,
            name: "add_graphql_subscriptions".into(),
            up_sql: M_112_GRAPHQL_SUBSCRIPTIONS_UP.into(),
            down_sql: M_112_GRAPHQL_SUBSCRIPTIONS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 113,
            name: "add_realtime_channels".into(),
            up_sql: M_113_REALTIME_CHANNELS_UP.into(),
            down_sql: M_113_REALTIME_CHANNELS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 114,
            name: "add_live_collaboration".into(),
            up_sql: M_114_LIVE_COLLABORATION_UP.into(),
            down_sql: M_114_LIVE_COLLABORATION_DOWN.into(),
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
        assert_eq!(mgr.all().len(), 91);
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
        assert_eq!(mgr.all()[41].version, 60);
        assert_eq!(mgr.all()[41].name, "add_oauth2");
        assert_eq!(mgr.all()[42].version, 62);
        assert_eq!(mgr.all()[42].name, "add_pr_templates");
        assert_eq!(mgr.all()[43].version, 63);
        assert_eq!(mgr.all()[43].name, "add_discussions");
        assert_eq!(mgr.all()[44].version, 66);
        assert_eq!(mgr.all()[44].name, "add_boards_v2");
        assert_eq!(mgr.all()[45].version, 67);
        assert_eq!(mgr.all()[45].name, "add_review_threads");
        assert_eq!(mgr.all()[47].version, 69);
        assert_eq!(mgr.all()[47].name, "add_npm_packages");
        assert_eq!(mgr.all()[48].version, 70);
        assert_eq!(mgr.all()[48].name, "add_maven_packages");
        assert_eq!(mgr.all()[49].version, 71);
        assert_eq!(mgr.all()[49].name, "add_pages_sites");
        assert_eq!(mgr.all()[50].version, 72);
        assert_eq!(mgr.all()[50].name, "add_discussion_labels_reactions");
        assert_eq!(mgr.all()[51].version, 73);
        assert_eq!(mgr.all()[51].name, "add_search_history");
        assert_eq!(mgr.all()[52].version, 74);
        assert_eq!(mgr.all()[52].name, "add_code_suggestions");
        assert_eq!(mgr.all()[53].version, 75);
        assert_eq!(mgr.all()[53].name, "add_license_reports");
        assert_eq!(mgr.all()[54].version, 76);
        assert_eq!(mgr.all()[54].name, "enhance_audit_log");
        assert_eq!(mgr.all()[55].version, 77);
        assert_eq!(mgr.all()[55].name, "enhance_pages_sites");
        assert_eq!(mgr.all()[56].version, 80);
        assert_eq!(mgr.all()[56].name, "add_saml_providers");
        assert_eq!(mgr.all()[57].version, 81);
        assert_eq!(mgr.all()[57].name, "add_scim_tokens");
        assert_eq!(mgr.all()[58].version, 82);
        assert_eq!(mgr.all()[58].name, "add_sso_groups_sessions_login_history");
        assert_eq!(mgr.all()[59].version, 83);
        assert_eq!(mgr.all()[59].name, "add_container_repository_policies");
        assert_eq!(mgr.all()[60].version, 84);
        assert_eq!(mgr.all()[60].name, "add_observability_tables");
        assert_eq!(mgr.all()[61].version, 85);
        assert_eq!(mgr.all()[61].name, "add_performance_indexes");
        assert_eq!(mgr.all()[62].version, 86);
        assert_eq!(mgr.all()[62].name, "add_cache_entries");
        assert_eq!(mgr.all()[63].version, 87);
        assert_eq!(mgr.all()[63].name, "add_cdn_config");
        assert_eq!(mgr.all()[64].version, 88);
        assert_eq!(mgr.all()[64].name, "add_server_instances");
        assert_eq!(mgr.all()[65].version, 89);
        assert_eq!(mgr.all()[65].name, "add_websocket_connections");
        assert_eq!(mgr.all()[66].version, 90);
        assert_eq!(mgr.all()[66].name, "add_pool_config");
        assert_eq!(mgr.all()[67].version, 91);
        assert_eq!(mgr.all()[67].name, "add_feature_flags");
        assert_eq!(mgr.all()[68].version, 92);
        assert_eq!(mgr.all()[68].name, "add_admin_dashboard_config");
        assert_eq!(mgr.all()[69].version, 93);
        assert_eq!(mgr.all()[69].name, "add_api_analytics");
        assert_eq!(mgr.all()[70].version, 94);
        assert_eq!(mgr.all()[70].name, "add_usage_quotas");
        assert_eq!(mgr.all()[71].version, 95);
        assert_eq!(mgr.all()[71].name, "add_export_jobs");
        assert_eq!(mgr.all()[72].version, 96);
        assert_eq!(mgr.all()[72].name, "add_compliance_reports");
        assert_eq!(mgr.all()[73].version, 97);
        assert_eq!(mgr.all()[73].name, "add_deployment_history");
        assert_eq!(mgr.all()[74].version, 98);
        assert_eq!(mgr.all()[74].name, "add_monitoring_alerts");
        assert_eq!(mgr.all()[75].version, 99);
        assert_eq!(mgr.all()[75].name, "add_performance_metrics");
        assert_eq!(mgr.all()[76].version, 100);
        assert_eq!(mgr.all()[76].name, "add_webhook_deliveries_v2");
        assert_eq!(mgr.all()[77].version, 101);
        assert_eq!(mgr.all()[77].name, "add_events_and_subscriptions");
        assert_eq!(mgr.all()[78].version, 102);
        assert_eq!(mgr.all()[78].name, "add_event_queues");
        assert_eq!(mgr.all()[85].version, 109);
        assert_eq!(mgr.all()[85].name, "add_api_gateway");
        assert_eq!(mgr.all()[86].version, 110);
        assert_eq!(mgr.all()[86].name, "add_rate_limit_policies");
        assert_eq!(mgr.all()[87].version, 111);
        assert_eq!(mgr.all()[87].name, "add_api_transforms");
        assert_eq!(mgr.all()[88].version, 112);
        assert_eq!(mgr.all()[88].name, "add_graphql_subscriptions");
        assert_eq!(mgr.all()[89].version, 113);
        assert_eq!(mgr.all()[89].name, "add_realtime_channels");
        assert_eq!(mgr.all()[90].version, 114);
        assert_eq!(mgr.all()[90].name, "add_live_collaboration");
    }

    #[test]
    fn test_add_migration_sequential() {
        let mut mgr = MigrationManager::new();
        mgr.add_migration(Migration {
            version: 115,
            name: "add_index".into(),
            up_sql: "CREATE INDEX test;".into(),
            down_sql: "DROP INDEX test;".into(),
        });
        assert_eq!(mgr.all().len(), 92);
        assert_eq!(mgr.all()[91].version, 115);
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
        assert_eq!(pending.len(), 91);
    }

    #[test]
    fn test_get_pending_all_applied() {
        let mgr = MigrationManager::new();
        let pending = mgr.get_pending(114);
        assert_eq!(pending.len(), 0);
    }

    #[test]
    fn test_get_pending_partial() {
        let mgr = MigrationManager::new();
        let pending = mgr.get_pending(1);
        assert_eq!(pending.len(), 90);
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

    #[test]
    fn test_container_repo_policies_sql_not_empty() {
        assert_ne!(M_083_CONTAINER_REPO_POLICIES_UP, "");
        assert!(M_083_CONTAINER_REPO_POLICIES_UP.contains("CREATE TABLE IF NOT EXISTS container_repository_policies"));
        assert!(M_083_CONTAINER_REPO_POLICIES_UP.contains("CREATE TABLE IF NOT EXISTS container_vulnerability_scans"));
        assert!(M_083_CONTAINER_REPO_POLICIES_UP.contains("CREATE TABLE IF NOT EXISTS container_image_signatures"));
        assert!(M_083_CONTAINER_REPO_POLICIES_UP.contains("CREATE TABLE IF NOT EXISTS container_pull_through_cache"));
    }

    #[test]
    fn test_container_repo_policies_down_sql_not_empty() {
        assert_ne!(M_083_CONTAINER_REPO_POLICIES_DOWN, "");
        assert!(M_083_CONTAINER_REPO_POLICIES_DOWN.contains("DROP TABLE IF EXISTS container_repository_policies"));
    }

    #[test]
    fn test_observability_tables_sql_not_empty() {
        assert_ne!(M_084_OBSERVABILITY_TABLES_UP, "");
        assert!(M_084_OBSERVABILITY_TABLES_UP.contains("CREATE TABLE IF NOT EXISTS trace_spans"));
        assert!(M_084_OBSERVABILITY_TABLES_UP.contains("CREATE TABLE IF NOT EXISTS metrics"));
    }

    #[test]
    fn test_observability_tables_down_sql_not_empty() {
        assert_ne!(M_084_OBSERVABILITY_TABLES_DOWN, "");
        assert!(M_084_OBSERVABILITY_TABLES_DOWN.contains("DROP TABLE IF EXISTS trace_spans"));
        assert!(M_084_OBSERVABILITY_TABLES_DOWN.contains("DROP TABLE IF EXISTS metrics"));
    }

    #[test]
    fn test_webhook_deliveries_v2_sql_not_empty() {
        assert_ne!(M_100_WEBHOOK_DELIVERIES_V2_UP, "");
        assert!(M_100_WEBHOOK_DELIVERIES_V2_UP.contains("CREATE TABLE IF NOT EXISTS webhook_deliveries_v2"));
        assert!(M_100_WEBHOOK_DELIVERIES_V2_UP.contains("webhook_id"));
        assert!(M_100_WEBHOOK_DELIVERIES_V2_UP.contains("event"));
        assert!(M_100_WEBHOOK_DELIVERIES_V2_UP.contains("payload"));
        assert!(M_100_WEBHOOK_DELIVERIES_V2_UP.contains("status"));
        assert!(M_100_WEBHOOK_DELIVERIES_V2_UP.contains("response_status"));
        assert!(M_100_WEBHOOK_DELIVERIES_V2_UP.contains("response_body"));
        assert!(M_100_WEBHOOK_DELIVERIES_V2_UP.contains("attempts"));
        assert!(M_100_WEBHOOK_DELIVERIES_V2_UP.contains("max_attempts"));
        assert!(M_100_WEBHOOK_DELIVERIES_V2_UP.contains("next_retry_at"));
        assert!(M_100_WEBHOOK_DELIVERIES_V2_UP.contains("created_at"));
    }

    #[test]
    fn test_webhook_deliveries_v2_down_sql_not_empty() {
        assert_ne!(M_100_WEBHOOK_DELIVERIES_V2_DOWN, "");
        assert!(M_100_WEBHOOK_DELIVERIES_V2_DOWN.contains("DROP TABLE IF EXISTS webhook_deliveries_v2"));
    }

    #[test]
    fn test_events_and_subscriptions_sql_not_empty() {
        assert_ne!(M_101_EVENTS_AND_SUBSCRIPTIONS_UP, "");
        assert!(M_101_EVENTS_AND_SUBSCRIPTIONS_UP.contains("CREATE TABLE IF NOT EXISTS events"));
        assert!(M_101_EVENTS_AND_SUBSCRIPTIONS_UP.contains("CREATE TABLE IF NOT EXISTS event_subscriptions"));
        assert!(M_101_EVENTS_AND_SUBSCRIPTIONS_UP.contains("event_type"));
        assert!(M_101_EVENTS_AND_SUBSCRIPTIONS_UP.contains("resource_type"));
        assert!(M_101_EVENTS_AND_SUBSCRIPTIONS_UP.contains("resource_id"));
        assert!(M_101_EVENTS_AND_SUBSCRIPTIONS_UP.contains("actor_id"));
        assert!(M_101_EVENTS_AND_SUBSCRIPTIONS_UP.contains("payload"));
        assert!(M_101_EVENTS_AND_SUBSCRIPTIONS_UP.contains("callback_url"));
        assert!(M_101_EVENTS_AND_SUBSCRIPTIONS_UP.contains("enabled"));
    }

    #[test]
    fn test_events_and_subscriptions_down_sql_not_empty() {
        assert_ne!(M_101_EVENTS_AND_SUBSCRIPTIONS_DOWN, "");
        assert!(M_101_EVENTS_AND_SUBSCRIPTIONS_DOWN.contains("DROP TABLE IF EXISTS event_subscriptions"));
        assert!(M_101_EVENTS_AND_SUBSCRIPTIONS_DOWN.contains("DROP TABLE IF EXISTS events"));
    }

    #[test]
    fn test_event_queues_sql_not_empty() {
        assert_ne!(M_102_EVENT_QUEUES_UP, "");
        assert!(M_102_EVENT_QUEUES_UP.contains("CREATE TABLE IF NOT EXISTS event_queues"));
        assert!(M_102_EVENT_QUEUES_UP.contains("CREATE TABLE IF NOT EXISTS event_queue_messages"));
        assert!(M_102_EVENT_QUEUES_UP.contains("queue_name"));
        assert!(M_102_EVENT_QUEUES_UP.contains("message_count"));
        assert!(M_102_EVENT_QUEUES_UP.contains("payload"));
        assert!(M_102_EVENT_QUEUES_UP.contains("status"));
        assert!(M_102_EVENT_QUEUES_UP.contains("attempts"));
        assert!(M_102_EVENT_QUEUES_UP.contains("max_attempts"));
        assert!(M_102_EVENT_QUEUES_UP.contains("processed_at"));
    }

    #[test]
    fn test_event_queues_down_sql_not_empty() {
        assert_ne!(M_102_EVENT_QUEUES_DOWN, "");
        assert!(M_102_EVENT_QUEUES_DOWN.contains("DROP TABLE IF EXISTS event_queue_messages"));
        assert!(M_102_EVENT_QUEUES_DOWN.contains("DROP TABLE IF EXISTS event_queues"));
    }

    #[test]
    fn test_chaos_engineering_sql_not_empty() {
        assert_ne!(M_103_CHAOS_ENGINEERING_UP, "");
        assert!(M_103_CHAOS_ENGINEERING_UP.contains("CREATE TABLE IF NOT EXISTS chaos_experiments"));
        assert!(M_103_CHAOS_ENGINEERING_UP.contains("CREATE TABLE IF NOT EXISTS chaos_results"));
        assert!(M_103_CHAOS_ENGINEERING_UP.contains("experiment_type"));
        assert!(M_103_CHAOS_ENGINEERING_UP.contains("target"));
        assert!(M_103_CHAOS_ENGINEERING_UP.contains("parameters"));
        assert!(M_103_CHAOS_ENGINEERING_UP.contains("status"));
        assert!(M_103_CHAOS_ENGINEERING_UP.contains("metric_name"));
        assert!(M_103_CHAOS_ENGINEERING_UP.contains("metric_value"));
        assert!(M_103_CHAOS_ENGINEERING_UP.contains("baseline_value"));
        assert!(M_103_CHAOS_ENGINEERING_UP.contains("impact"));
    }

    #[test]
    fn test_chaos_engineering_down_sql_not_empty() {
        assert_ne!(M_103_CHAOS_ENGINEERING_DOWN, "");
        assert!(M_103_CHAOS_ENGINEERING_DOWN.contains("DROP TABLE IF EXISTS chaos_results"));
        assert!(M_103_CHAOS_ENGINEERING_DOWN.contains("DROP TABLE IF EXISTS chaos_experiments"));
    }

    #[test]
    fn test_resilience_tests_sql_not_empty() {
        assert_ne!(M_104_RESILIENCE_TESTS_UP, "");
        assert!(M_104_RESILIENCE_TESTS_UP.contains("CREATE TABLE IF NOT EXISTS resilience_tests"));
        assert!(M_104_RESILIENCE_TESTS_UP.contains("test_type"));
        assert!(M_104_RESILIENCE_TESTS_UP.contains("target"));
        assert!(M_104_RESILIENCE_TESTS_UP.contains("parameters"));
        assert!(M_104_RESILIENCE_TESTS_UP.contains("status"));
        assert!(M_104_RESILIENCE_TESTS_UP.contains("score"));
    }

    #[test]
    fn test_resilience_tests_down_sql_not_empty() {
        assert_ne!(M_104_RESILIENCE_TESTS_DOWN, "");
        assert!(M_104_RESILIENCE_TESTS_DOWN.contains("DROP TABLE IF EXISTS resilience_tests"));
    }

    #[test]
    fn test_circuit_breakers_sql_not_empty() {
        assert_ne!(M_105_CIRCUIT_BREAKERS_UP, "");
        assert!(M_105_CIRCUIT_BREAKERS_UP.contains("CREATE TABLE IF NOT EXISTS circuit_breakers"));
        assert!(M_105_CIRCUIT_BREAKERS_UP.contains("state"));
        assert!(M_105_CIRCUIT_BREAKERS_UP.contains("failure_count"));
        assert!(M_105_CIRCUIT_BREAKERS_UP.contains("failure_threshold"));
        assert!(M_105_CIRCUIT_BREAKERS_UP.contains("success_threshold"));
        assert!(M_105_CIRCUIT_BREAKERS_UP.contains("timeout_seconds"));
        assert!(M_105_CIRCUIT_BREAKERS_UP.contains("last_failure_at"));
        assert!(M_105_CIRCUIT_BREAKERS_UP.contains("last_state_change"));
    }

    #[test]
    fn test_circuit_breakers_down_sql_not_empty() {
        assert_ne!(M_105_CIRCUIT_BREAKERS_DOWN, "");
        assert!(M_105_CIRCUIT_BREAKERS_DOWN.contains("DROP TABLE IF EXISTS circuit_breakers"));
    }
}
