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
pub const M_115_CODE_INTELLIGENCE_UP: &str = include_str!("115_add_code_intelligence.sql");
pub const M_115_CODE_INTELLIGENCE_DOWN: &str =
    "DROP TABLE IF EXISTS code_intelligence_references; DROP TABLE IF EXISTS code_intelligence_symbols;";
pub const M_116_CODE_SEARCH_V2_UP: &str = include_str!("116_add_code_search_v2.sql");
pub const M_116_CODE_SEARCH_V2_DOWN: &str =
    "DROP TABLE IF EXISTS code_search_queries; DROP TABLE IF EXISTS code_search_index_v2;";
pub const M_117_CODE_FORMATTERS_UP: &str = include_str!("117_add_code_formatters.sql");
pub const M_117_CODE_FORMATTERS_DOWN: &str = "DROP TABLE IF EXISTS code_formatters;";
pub const M_118_PIPELINE_TEMPLATES_UP: &str = include_str!("118_add_pipeline_templates.sql");
pub const M_118_PIPELINE_TEMPLATES_DOWN: &str = "DROP TABLE IF EXISTS pipeline_templates;";
pub const M_119_PIPELINE_ANALYTICS_UP: &str = include_str!("119_add_pipeline_analytics.sql");
pub const M_119_PIPELINE_ANALYTICS_DOWN: &str = "DROP TABLE IF EXISTS pipeline_analytics;";
pub const M_120_MULTI_PROJECT_PIPELINES_UP: &str = include_str!("120_add_multi_project_pipelines.sql");
pub const M_120_MULTI_PROJECT_PIPELINES_DOWN: &str = "DROP TABLE IF EXISTS multi_project_pipeline_runs; DROP TABLE IF EXISTS multi_project_pipelines;";
pub const M_121_SECURITY_SCANS_V2_UP: &str = include_str!("121_add_security_scans_v2.sql");
pub const M_121_SECURITY_SCANS_V2_DOWN: &str = "DROP TABLE IF EXISTS security_policies; DROP TABLE IF EXISTS security_scans_v2;";
pub const M_122_COMPLIANCE_FRAMEWORKS_UP: &str = include_str!("122_add_compliance_frameworks.sql");
pub const M_122_COMPLIANCE_FRAMEWORKS_DOWN: &str = "DROP TABLE IF EXISTS compliance_assessments; DROP TABLE IF EXISTS compliance_frameworks;";
pub const M_123_AUDIT_TRAIL_UP: &str = include_str!("123_add_audit_trail.sql");
pub const M_123_AUDIT_TRAIL_DOWN: &str = "DROP TABLE IF EXISTS audit_trail;";
pub const M_124_API_DOCUMENTATION_UP: &str = include_str!("124_add_api_documentation.sql");
pub const M_124_API_DOCUMENTATION_DOWN: &str = "DROP TABLE IF EXISTS api_documentation;";
pub const M_125_API_VERSIONS_UP: &str = include_str!("125_add_api_versions.sql");
pub const M_125_API_VERSIONS_DOWN: &str = "DROP TABLE IF EXISTS api_versions;";
pub const M_126_API_ANALYTICS_V2_UP: &str = include_str!("126_add_api_analytics_v2.sql");
pub const M_126_API_ANALYTICS_V2_DOWN: &str = "DROP TABLE IF EXISTS api_analytics_v2;";
pub const M_127_DEPLOYMENT_STRATEGIES_UP: &str = include_str!("127_add_deployment_strategies.sql");
pub const M_127_DEPLOYMENT_STRATEGIES_DOWN: &str = "DROP TABLE IF EXISTS deployment_strategies;";
pub const M_128_INFRASTRUCTURE_UP: &str = include_str!("128_add_infrastructure.sql");
pub const M_128_INFRASTRUCTURE_DOWN: &str = "DROP TABLE IF EXISTS infrastructure_deployments; DROP TABLE IF EXISTS infrastructure_templates;";
pub const M_129_SERVICE_MESH_UP: &str = include_str!("129_add_service_mesh.sql");
pub const M_129_SERVICE_MESH_DOWN: &str = "DROP TABLE IF EXISTS service_mesh_routes; DROP TABLE IF EXISTS service_mesh_services;";
pub const M_130_TEST_COVERAGE_UP: &str = include_str!("130_add_test_coverage.sql");
pub const M_130_TEST_COVERAGE_DOWN: &str = "DROP TABLE IF EXISTS test_coverage;";
pub const M_131_CODE_QUALITY_METRICS_UP: &str = include_str!("131_add_code_quality_metrics.sql");
pub const M_131_CODE_QUALITY_METRICS_DOWN: &str = "DROP TABLE IF EXISTS code_quality_metrics;";
pub const M_132_PERFORMANCE_TESTS_UP: &str = include_str!("132_add_performance_tests.sql");
pub const M_132_PERFORMANCE_TESTS_DOWN: &str = "DROP TABLE IF EXISTS performance_tests;";
pub const M_133_PIPELINE_ACTIONS_UP: &str = include_str!("133_add_pipeline_actions.sql");
pub const M_133_PIPELINE_ACTIONS_DOWN: &str = "DROP TABLE IF EXISTS pipeline_actions;";
pub const M_134_PIPELINE_ENVIRONMENTS_V2_UP: &str = include_str!("134_add_pipeline_environments_v2.sql");
pub const M_134_PIPELINE_ENVIRONMENTS_V2_DOWN: &str = "DROP TABLE IF EXISTS pipeline_environments_v2;";
pub const M_135_PIPELINE_CACHES_V2_UP: &str = include_str!("135_add_pipeline_caches_v2.sql");
pub const M_135_PIPELINE_CACHES_V2_DOWN: &str = "DROP TABLE IF EXISTS pipeline_caches_v2;";
pub const M_136_DATABASE_BACKUP_RECOVERY_UP: &str =
    include_str!("136_add_database_backup_recovery.sql");
pub const M_136_DATABASE_BACKUP_RECOVERY_DOWN: &str =
    "DROP TABLE IF EXISTS database_recovery_points; DROP TABLE IF EXISTS database_backups;";
pub const M_137_DATA_ARCHIVES_UP: &str = include_str!("137_add_data_archives.sql");
pub const M_137_DATA_ARCHIVES_DOWN: &str = "DROP TABLE IF EXISTS data_archives;";
pub const M_138_DATA_MIGRATIONS_UP: &str = include_str!("138_add_data_migrations.sql");
pub const M_138_DATA_MIGRATIONS_DOWN: &str = "DROP TABLE IF EXISTS data_migrations;";
pub const M_139_NETWORK_POLICIES_UP: &str = include_str!("139_add_network_policies.sql");
pub const M_139_NETWORK_POLICIES_DOWN: &str = "DROP TABLE IF EXISTS network_policies;";
pub const M_140_ENCRYPTION_AT_REST_UP: &str = include_str!("140_add_encryption_at_rest.sql");
pub const M_140_ENCRYPTION_AT_REST_DOWN: &str = "DROP TABLE IF EXISTS encrypted_data; DROP TABLE IF EXISTS encryption_keys;";
pub const M_141_ACCESS_CONTROL_LISTS_UP: &str = include_str!("141_add_access_control_lists.sql");
pub const M_141_ACCESS_CONTROL_LISTS_DOWN: &str = "DROP TABLE IF EXISTS access_control_lists;";
pub const M_142_WORKFLOWS_UP: &str = include_str!("142_add_workflows.sql");
pub const M_142_WORKFLOWS_DOWN: &str =
    "DROP TABLE IF EXISTS workflow_runs; DROP TABLE IF EXISTS workflows;";
pub const M_143_AUTOMATION_RULES_UP: &str = include_str!("143_add_automation_rules.sql");
pub const M_143_AUTOMATION_RULES_DOWN: &str = "DROP TABLE IF EXISTS automation_rules;";
pub const M_144_SCHEDULED_TASKS_UP: &str = include_str!("144_add_scheduled_tasks.sql");
pub const M_144_SCHEDULED_TASKS_DOWN: &str = "DROP TABLE IF EXISTS scheduled_tasks;";
pub const M_145_LOG_AGGREGATION_UP: &str = include_str!("145_add_log_aggregation.sql");
pub const M_145_LOG_AGGREGATION_DOWN: &str = "DROP TABLE IF EXISTS log_entries;";
pub const M_146_TRACE_SAMPLING_RULES_UP: &str = include_str!("146_add_trace_sampling_rules.sql");
pub const M_146_TRACE_SAMPLING_RULES_DOWN: &str = "DROP TABLE IF EXISTS trace_sampling_rules;";
pub const M_147_DASHBOARD_REPORTING_UP: &str = include_str!("147_add_dashboard_reporting.sql");
pub const M_147_DASHBOARD_REPORTING_DOWN: &str = "DROP TABLE IF EXISTS reports; DROP TABLE IF EXISTS dashboards;";
pub const M_148_PIPELINE_SECRETS_V2_UP: &str = include_str!("148_add_pipeline_secrets_v2.sql");
pub const M_148_PIPELINE_SECRETS_V2_DOWN: &str = "DROP TABLE IF EXISTS secret_access_log; DROP TABLE IF EXISTS secret_rotation_log; DROP TABLE IF EXISTS pipeline_secrets_v2;";
pub const M_149_PIPELINE_RUNNERS_V2_UP: &str = include_str!("149_add_pipeline_runners_v2.sql");
pub const M_149_PIPELINE_RUNNERS_V2_DOWN: &str = "DROP TABLE IF EXISTS runner_metrics; DROP TABLE IF EXISTS pipeline_runners_v2;";
pub const M_150_ENVIRONMENT_VARIABLES_UP: &str = include_str!("150_add_environment_variables.sql");
pub const M_150_ENVIRONMENT_VARIABLES_DOWN: &str = "DROP TABLE IF EXISTS environment_variable_inheritance; DROP TABLE IF EXISTS environment_variables;";
pub const M_154_API_DOCS_V2_UP: &str = include_str!("154_add_api_docs_v2.sql");
pub const M_154_API_DOCS_V2_DOWN: &str = "DROP TABLE IF EXISTS api_docs_v2;";
pub const M_155_RATE_LIMIT_TIERS_UP: &str = include_str!("155_add_rate_limit_tiers.sql");
pub const M_155_RATE_LIMIT_TIERS_DOWN: &str = "DROP TABLE IF EXISTS rate_limit_tiers;";
pub const M_156_API_ANALYTICS_V3_UP: &str = include_str!("156_add_api_analytics_v3.sql");
pub const M_156_API_ANALYTICS_V3_DOWN: &str = "DROP TABLE IF EXISTS api_analytics_v3;";
pub const M_157_DATABASE_REPLICATION_UP: &str = include_str!("157_add_database_replication.sql");
pub const M_157_DATABASE_REPLICATION_DOWN: &str =
    "DROP TABLE IF EXISTS database_replicas;";
pub const M_158_ENCRYPTION_POLICIES_UP: &str = include_str!("158_add_encryption_policies.sql");
pub const M_158_ENCRYPTION_POLICIES_DOWN: &str = "DROP TABLE IF EXISTS encryption_policies;";
pub const M_159_DATA_RESIDENCY_UP: &str = include_str!("159_add_data_residency.sql");
pub const M_159_DATA_RESIDENCY_DOWN: &str =
    "DROP TABLE IF EXISTS data_residency_violations; DROP TABLE IF EXISTS data_residency_rules;";
pub const M_160_SECURITY_SCAN_RULES_UP: &str = include_str!("160_add_security_scan_rules.sql");
pub const M_160_SECURITY_SCAN_RULES_DOWN: &str = "DROP TABLE IF EXISTS security_scan_rules;";
pub const M_161_COMPLIANCE_REQUIREMENTS_UP: &str =
    include_str!("161_add_compliance_requirements.sql");
pub const M_161_COMPLIANCE_REQUIREMENTS_DOWN: &str =
    "DROP TABLE IF EXISTS compliance_check_results; DROP TABLE IF EXISTS compliance_evidence; DROP TABLE IF EXISTS compliance_requirements;";
pub const M_162_AUDIT_TRAIL_V2_UP: &str = include_str!("162_add_audit_trail_v2.sql");
pub const M_162_AUDIT_TRAIL_V2_DOWN: &str = "DROP TABLE IF EXISTS audit_trail_v2;";
pub const M_166_FIREWALL_RULES_UP: &str = include_str!("166_add_firewall_rules.sql");
pub const M_166_FIREWALL_RULES_DOWN: &str =
    "DROP TABLE IF EXISTS firewall_rule_logs; DROP TABLE IF EXISTS firewall_rules;";
pub const M_167_INTRUSION_DETECTIONS_UP: &str = include_str!("167_add_intrusion_detections.sql");
pub const M_167_INTRUSION_DETECTIONS_DOWN: &str =
    "DROP TABLE IF EXISTS intrusion_incidents; DROP TABLE IF EXISTS intrusion_detection_rules; DROP TABLE IF EXISTS intrusion_detections;";
pub const M_168_DDOS_PROTECTION_UP: &str = include_str!("168_add_ddos_protection.sql");
pub const M_168_DDOS_PROTECTION_DOWN: &str =
    "DROP TABLE IF EXISTS ddos_events; DROP TABLE IF EXISTS ddos_protection;";
pub const M_169_OBJECT_STORAGE_UP: &str = include_str!("169_add_object_storage.sql");
pub const M_169_OBJECT_STORAGE_DOWN: &str =
    "DROP TABLE IF EXISTS object_storage_objects; DROP TABLE IF EXISTS object_storage_buckets;";
pub const M_170_BACKUP_ENCRYPTION_UP: &str = include_str!("170_add_backup_encryption.sql");
pub const M_170_BACKUP_ENCRYPTION_DOWN: &str = "DROP TABLE IF EXISTS backup_encryption_keys;";
pub const M_171_DATA_RETENTION_UP: &str = include_str!("171_add_data_retention.sql");
pub const M_171_DATA_RETENTION_DOWN: &str =
    "DROP TABLE IF EXISTS data_retention_actions; DROP TABLE IF EXISTS data_retention_policies;";
pub const M_172_API_DOCS_V3_UP: &str = include_str!("172_add_api_docs_v3.sql");
pub const M_172_API_DOCS_V3_DOWN: &str = "DROP TABLE IF EXISTS api_docs_v3;";
pub const M_173_API_WEBHOOKS_V2_UP: &str = include_str!("173_add_api_webhooks_v2.sql");
pub const M_173_API_WEBHOOKS_V2_DOWN: &str =
    "DROP TABLE IF EXISTS api_webhook_deliveries_v2; DROP TABLE IF EXISTS api_webhooks_v2;";
pub const M_174_API_ANALYTICS_V4_UP: &str = include_str!("174_add_api_analytics_v4.sql");
pub const M_174_API_ANALYTICS_V4_DOWN: &str = "DROP TABLE IF EXISTS api_analytics_v4;";
pub const M_175_DEPLOYMENT_STRATEGY_CONFIGS_LOGS_UP: &str = include_str!("175_add_deployment_strategy_configs_logs.sql");
pub const M_175_DEPLOYMENT_STRATEGY_CONFIGS_LOGS_DOWN: &str = "DROP TABLE IF EXISTS deployment_strategy_logs; DROP TABLE IF EXISTS deployment_strategy_configs;";
pub const M_176_INFRASTRUCTURE_MODULES_UP: &str = include_str!("176_add_infrastructure_modules.sql");
pub const M_176_INFRASTRUCTURE_MODULES_DOWN: &str = "DROP TABLE IF EXISTS infrastructure_module_deps; DROP TABLE IF EXISTS infrastructure_modules;";
pub const M_177_SERVICE_MESH_POLICIES_METRICS_UP: &str = include_str!("177_add_service_mesh_policies_metrics.sql");
pub const M_177_SERVICE_MESH_POLICIES_METRICS_DOWN: &str = "DROP TABLE IF EXISTS service_mesh_metrics; DROP TABLE IF EXISTS service_mesh_policies;";
pub const M_178_TEST_COVERAGE_V2_UP: &str = include_str!("178_add_test_coverage_v2.sql");
pub const M_178_TEST_COVERAGE_V2_DOWN: &str = "DROP TABLE IF EXISTS test_coverage_v2;";
pub const M_179_CODE_QUALITY_RULES_UP: &str = include_str!("179_add_code_quality_rules.sql");
pub const M_179_CODE_QUALITY_RULES_DOWN: &str = "DROP TABLE IF EXISTS code_quality_rules;";
pub const M_180_PERF_TEST_CONFIGS_RESULTS_UP: &str = include_str!("180_add_performance_test_configs_results.sql");
pub const M_180_PERF_TEST_CONFIGS_RESULTS_DOWN: &str = "DROP TABLE IF EXISTS performance_test_results; DROP TABLE IF EXISTS performance_test_configs;";
pub const M_187_WORKFLOW_TEMPLATES_UP: &str = include_str!("187_add_workflow_templates.sql");
pub const M_187_WORKFLOW_TEMPLATES_DOWN: &str = "DROP TABLE IF EXISTS workflow_template_usage; DROP TABLE IF EXISTS workflow_templates;";
pub const M_188_AUTOMATION_RULES_V3_UP: &str = include_str!("188_add_automation_rules_v3.sql");
pub const M_188_AUTOMATION_RULES_V3_DOWN: &str = "DROP TABLE IF EXISTS automation_rules_v3;";
pub const M_189_SCHEDULED_TASK_TEMPLATES_UP: &str = include_str!("189_add_scheduled_task_templates.sql");
pub const M_189_SCHEDULED_TASK_TEMPLATES_DOWN: &str = "DROP TABLE IF EXISTS scheduled_task_templates;";
pub const M_190_LOG_AGGREGATION_V2_UP: &str = include_str!("190_add_log_aggregation_v2.sql");
pub const M_190_LOG_AGGREGATION_V2_DOWN: &str = "DROP TABLE IF EXISTS log_retention_policies; DROP TABLE IF EXISTS log_entries_v2;";
pub const M_191_DISTRIBUTED_TRACING_V3_UP: &str = include_str!("191_add_distributed_tracing_v3.sql");
pub const M_191_DISTRIBUTED_TRACING_V3_DOWN: &str = "DROP TABLE IF EXISTS trace_dependencies; DROP TABLE IF EXISTS trace_sampling_rules_v2;";
pub const M_192_DASHBOARD_REPORTING_V2_UP: &str = include_str!("192_add_dashboard_reporting_v2.sql");
pub const M_192_DASHBOARD_REPORTING_V2_DOWN: &str = "DROP TABLE IF EXISTS report_schedules; DROP TABLE IF EXISTS dashboard_widgets_v2;";
pub const M_193_PIPELINE_ACTION_CATEGORIES_UP: &str = include_str!("193_add_pipeline_action_categories.sql");
pub const M_193_PIPELINE_ACTION_CATEGORIES_DOWN: &str = "DROP TABLE IF EXISTS pipeline_action_category_members; DROP TABLE IF EXISTS pipeline_action_categories;";
pub const M_194_ENVIRONMENT_WEBHOOKS_NOTIFICATIONS_UP: &str = include_str!("194_add_environment_webhooks_notifications.sql");
pub const M_194_ENVIRONMENT_WEBHOOKS_NOTIFICATIONS_DOWN: &str = "DROP TABLE IF EXISTS environment_webhook_deliveries; DROP TABLE IF EXISTS environment_notifications; DROP TABLE IF EXISTS environment_webhooks;";
pub const M_195_CACHE_WARMING_RULES_UP: &str = include_str!("195_add_cache_warming_rules.sql");
pub const M_195_CACHE_WARMING_RULES_DOWN: &str = "DROP TABLE IF EXISTS cache_warming_logs; DROP TABLE IF EXISTS cache_warming_rules;";
pub const M_196_TEST_SUITE_CONFIG_NOTIFICATIONS_UP: &str = include_str!("196_add_test_suite_config_notifications.sql");
pub const M_196_TEST_SUITE_CONFIG_NOTIFICATIONS_DOWN: &str = "DROP TABLE IF EXISTS test_suite_notifications; DROP TABLE IF EXISTS test_suite_configurations;";
pub const M_197_CODE_QUALITY_RULES_V2_UP: &str = include_str!("197_add_code_quality_rules_v2.sql");
pub const M_197_CODE_QUALITY_RULES_V2_DOWN: &str = "DROP TABLE IF EXISTS code_quality_rule_test_results; DROP TABLE IF EXISTS code_quality_rule_versions; DROP TABLE IF EXISTS code_quality_rules_v2;";
pub const M_198_PERF_BASELINES_REGRESSIONS_UP: &str = include_str!("198_add_performance_baselines_regressions.sql");
pub const M_198_PERF_BASELINES_REGRESSIONS_DOWN: &str = "DROP TABLE IF EXISTS performance_trend_data; DROP TABLE IF EXISTS performance_regressions; DROP TABLE IF EXISTS performance_baselines;";
pub const M_199_API_DOCS_V4_UP: &str = include_str!("199_add_api_docs_v4.sql");
pub const M_199_API_DOCS_V4_DOWN: &str = "DROP TABLE IF EXISTS api_docs_v4;";
pub const M_200_RATE_LIMIT_TIERS_V2_UP: &str = include_str!("200_add_rate_limit_tiers_v2.sql");
pub const M_200_RATE_LIMIT_TIERS_V2_DOWN: &str = "DROP TABLE IF EXISTS rate_limit_usage_v2; DROP TABLE IF EXISTS rate_limit_tiers_v2;";
pub const M_201_API_ANALYTICS_V5_UP: &str = include_str!("201_add_api_analytics_v5.sql");
pub const M_201_API_ANALYTICS_V5_DOWN: &str = "DROP TABLE IF EXISTS api_analytics_v5;";
pub const M_202_DATABASE_REPLICATION_V2_UP: &str = include_str!("202_add_database_replication_v2.sql");
pub const M_202_DATABASE_REPLICATION_V2_DOWN: &str = "DROP TABLE IF EXISTS database_replication_stats; DROP TABLE IF EXISTS database_replication_logs;";
pub const M_203_ENCRYPTION_V3_UP: &str = include_str!("203_add_encryption_v3.sql");
pub const M_203_ENCRYPTION_V3_DOWN: &str = "DROP TABLE IF EXISTS encryption_audit_logs; DROP TABLE IF EXISTS encryption_key_rotations;";
pub const M_204_DATA_RESIDENCY_V2_UP: &str = include_str!("204_add_data_residency_v2.sql");
pub const M_204_DATA_RESIDENCY_V2_DOWN: &str = "DROP TABLE IF EXISTS data_residency_migrations; DROP TABLE IF EXISTS data_residency_audits;";
pub const M_205_SECURITY_SCAN_V3_UP: &str = include_str!("205_add_security_scan_rules_v3_fixes.sql");
pub const M_205_SECURITY_SCAN_V3_DOWN: &str = "DROP TABLE IF EXISTS security_scan_fixes; DROP TABLE IF EXISTS security_scan_rules_v3;";
pub const M_206_COMPLIANCE_V3_UP: &str = include_str!("206_add_compliance_frameworks_v3_evidence_v2.sql");
pub const M_206_COMPLIANCE_V3_DOWN: &str = "DROP TABLE IF EXISTS compliance_evidence_v2; DROP TABLE IF EXISTS compliance_frameworks_v3;";
pub const M_207_AUDIT_TRAIL_V4_UP: &str = include_str!("207_add_audit_trail_v4.sql");
pub const M_207_AUDIT_TRAIL_V4_DOWN: &str = "DROP TABLE IF EXISTS audit_trail_v4;";
pub const M_208_WORKFLOW_EXECUTION_V4_UP: &str = include_str!("208_add_workflow_execution_v4.sql");
pub const M_208_WORKFLOW_EXECUTION_V4_DOWN: &str = "DROP TABLE IF EXISTS workflow_execution_steps; DROP TABLE IF EXISTS workflow_executions;";
pub const M_209_AUTOMATION_RULES_V4_UP: &str = include_str!("209_add_automation_rules_v4.sql");
pub const M_209_AUTOMATION_RULES_V4_DOWN: &str = "DROP TABLE IF EXISTS automation_rules_v4;";
pub const M_210_SCHEDULED_TASK_EXECUTION_V4_UP: &str = include_str!("210_add_scheduled_task_execution_v4.sql");
pub const M_210_SCHEDULED_TASK_EXECUTION_V4_DOWN: &str = "DROP TABLE IF EXISTS scheduled_task_executions;";
pub const M_211_LOG_AGGREGATION_V3_UP: &str = include_str!("211_add_log_aggregation_v3.sql");
pub const M_211_LOG_AGGREGATION_V3_DOWN: &str = "DROP TABLE IF EXISTS log_search_index; DROP TABLE IF EXISTS log_entries_v3;";
pub const M_212_DISTRIBUTED_TRACING_V4_UP: &str = include_str!("212_add_distributed_tracing_v4.sql");
pub const M_212_DISTRIBUTED_TRACING_V4_DOWN: &str = "DROP TABLE IF EXISTS trace_service_map; DROP TABLE IF EXISTS trace_sampling_rules_v3;";
pub const M_213_DASHBOARD_REPORTING_V3_UP: &str = include_str!("213_add_dashboard_reporting_v3.sql");
pub const M_213_DASHBOARD_REPORTING_V3_DOWN: &str = "DROP TABLE IF EXISTS report_templates; DROP TABLE IF EXISTS dashboard_templates;";
pub const M_214_PIPELINE_ACTION_INSTALLATIONS_UP: &str = include_str!("214_add_pipeline_action_installations.sql");
pub const M_214_PIPELINE_ACTION_INSTALLATIONS_DOWN: &str = "DROP TABLE IF EXISTS pipeline_action_installations;";
pub const M_215_ENVIRONMENT_HEALTH_CHECKS_UP: &str = include_str!("215_add_environment_health_checks.sql");
pub const M_215_ENVIRONMENT_HEALTH_CHECKS_DOWN: &str = "DROP TABLE IF EXISTS environment_health_checks;";
pub const M_216_CACHE_EVICTION_POLICIES_LOGS_UP: &str = include_str!("216_add_cache_eviction_policies_logs.sql");
pub const M_216_CACHE_EVICTION_POLICIES_LOGS_DOWN: &str = "DROP TABLE IF EXISTS cache_eviction_logs; DROP TABLE IF EXISTS cache_eviction_policies;";
pub const M_217_TEST_SUITE_TAGS_DEPS_UP: &str = include_str!("217_add_test_suite_tags_dependencies.sql");
pub const M_217_TEST_SUITE_TAGS_DEPS_DOWN: &str = "DROP TABLE IF EXISTS test_suite_dependencies; DROP TABLE IF EXISTS test_suite_tags;";
pub const M_218_CODE_QUALITY_RULES_V3_UP: &str = include_str!("218_add_code_quality_rules_v3_enforcement.sql");
pub const M_218_CODE_QUALITY_RULES_V3_DOWN: &str = "DROP TABLE IF EXISTS code_quality_enforcement_logs; DROP TABLE IF EXISTS code_quality_rules_v3;";
pub const M_219_PERF_TEST_ALERTS_UP: &str = include_str!("219_add_performance_test_alerts.sql");
pub const M_219_PERF_TEST_ALERTS_DOWN: &str = "DROP TABLE IF EXISTS performance_test_alerts;";
pub const M_220_API_DOCS_V5_UP: &str = include_str!("220_add_api_docs_v5.sql");
pub const M_220_API_DOCS_V5_DOWN: &str = "DROP TABLE IF EXISTS api_docs_v5;";
pub const M_221_RATE_LIMIT_TIERS_V3_UP: &str = include_str!("221_add_rate_limit_tiers_v3.sql");
pub const M_221_RATE_LIMIT_TIERS_V3_DOWN: &str = "DROP TABLE IF EXISTS rate_limit_alerts; DROP TABLE IF EXISTS rate_limit_overages; DROP TABLE IF EXISTS rate_limit_tiers_v3;";
pub const M_222_API_ANALYTICS_V6_UP: &str = include_str!("222_add_api_analytics_v6.sql");
pub const M_222_API_ANALYTICS_V6_DOWN: &str = "DROP TABLE IF EXISTS api_analytics_capacity_plans; DROP TABLE IF EXISTS api_analytics_correlations; DROP TABLE IF EXISTS api_analytics_v6;";
pub const M_223_DATABASE_REPLICATION_V3_UP: &str = include_str!("223_add_database_replication_v3.sql");
pub const M_223_DATABASE_REPLICATION_V3_DOWN: &str = "DROP TABLE IF EXISTS database_replication_alerts; DROP TABLE IF EXISTS database_replication_config;";
pub const M_224_ENCRYPTION_V4_UP: &str = include_str!("224_add_encryption_v4.sql");
pub const M_224_ENCRYPTION_V4_DOWN: &str = "DROP TABLE IF EXISTS encryption_compliance_checks; DROP TABLE IF EXISTS encryption_key_versions;";
pub const M_225_DATA_RESIDENCY_V3_UP: &str = include_str!("225_add_data_residency_v3.sql");
pub const M_225_DATA_RESIDENCY_V3_DOWN: &str = "DROP TABLE IF EXISTS data_residency_compliance; DROP TABLE IF EXISTS data_residency_reports;";
pub const M_232_LOG_AGGREGATION_V4_UP: &str = include_str!("232_add_log_aggregation_v4.sql");
pub const M_232_LOG_AGGREGATION_V4_DOWN: &str =
    "DROP TABLE IF EXISTS log_alert_rules; DROP TABLE IF EXISTS log_entries_v4;";
pub const M_233_DISTRIBUTED_TRACING_V4_UP: &str =
    include_str!("233_add_distributed_tracing_v4.sql");
pub const M_233_DISTRIBUTED_TRACING_V4_DOWN: &str =
    "DROP TABLE IF EXISTS trace_service_dependencies; DROP TABLE IF EXISTS trace_sampling_rules_v4;";
pub const M_234_DASHBOARD_REPORTING_V4_UP: &str =
    include_str!("234_add_dashboard_reporting_v4.sql");
pub const M_234_DASHBOARD_REPORTING_V4_DOWN: &str =
    "DROP TABLE IF EXISTS report_schedules_v2; DROP TABLE IF EXISTS dashboard_shares;";
pub const M_238_TEST_SUITE_METRICS_BASELINES_UP: &str =
    include_str!("238_add_test_suite_metrics_baselines.sql");
pub const M_238_TEST_SUITE_METRICS_BASELINES_DOWN: &str =
    "DROP TABLE IF EXISTS test_suite_baselines; DROP TABLE IF EXISTS test_suite_metrics;";
pub const M_239_CODE_QUALITY_METRICS_THRESHOLDS_UP: &str =
    include_str!("239_add_code_quality_metrics_thresholds.sql");
pub const M_239_CODE_QUALITY_METRICS_THRESHOLDS_DOWN: &str =
    "DROP TABLE IF EXISTS code_quality_thresholds; DROP TABLE IF EXISTS code_quality_metrics_v2;";
pub const M_240_PERF_TEST_ALERTS_V2_UP: &str =
    include_str!("240_add_performance_test_alerts_v2.sql");
pub const M_240_PERF_TEST_ALERTS_V2_DOWN: &str =
    "DROP TABLE IF EXISTS performance_test_alert_history_v2; DROP TABLE IF EXISTS performance_test_alerts_v2;";
pub const M_244_DATABASE_REPLICATION_V4_UP: &str =
    include_str!("244_add_database_replication_v4.sql");
pub const M_244_DATABASE_REPLICATION_V4_DOWN: &str =
    "DROP TABLE IF EXISTS database_replication_alerts_v2; DROP TABLE IF EXISTS database_replication_config_v2;";
pub const M_245_ENCRYPTION_V5_UP: &str = include_str!("245_add_encryption_v5.sql");
pub const M_245_ENCRYPTION_V5_DOWN: &str =
    "DROP TABLE IF EXISTS encryption_compliance_checks_v2; DROP TABLE IF EXISTS encryption_key_versions_v2;";
pub const M_246_DATA_RESIDENCY_V4_UP: &str = include_str!("246_add_data_residency_v4.sql");
pub const M_246_DATA_RESIDENCY_V4_DOWN: &str =
    "DROP TABLE IF EXISTS data_residency_compliance_v2; DROP TABLE IF EXISTS data_residency_reports_v2;";
pub const M_247_SECURITY_SCAN_RULES_V5_UP: &str = include_str!("247_add_security_scan_rules_v5.sql");
pub const M_247_SECURITY_SCAN_RULES_V5_DOWN: &str =
    "DROP TABLE IF EXISTS security_scan_fixes_v3; DROP TABLE IF EXISTS security_scan_rules_v5;";
pub const M_248_COMPLIANCE_FRAMEWORKS_V5_UP: &str = include_str!("248_add_compliance_frameworks_v5.sql");
pub const M_248_COMPLIANCE_FRAMEWORKS_V5_DOWN: &str =
    "DROP TABLE IF EXISTS compliance_assessments_v4; DROP TABLE IF EXISTS compliance_frameworks_v5;";
pub const M_249_AUDIT_TRAIL_V6_UP: &str = include_str!("249_add_audit_trail_v6.sql");
pub const M_249_AUDIT_TRAIL_V6_DOWN: &str = "DROP TABLE IF EXISTS audit_trail_v6;";
pub const M_253_LOG_AGGREGATION_V5_UP: &str = include_str!("253_add_log_aggregation_v5.sql");
pub const M_253_LOG_AGGREGATION_V5_DOWN: &str =
    "DROP TABLE IF EXISTS log_alert_rules_v2; DROP TABLE IF EXISTS log_entries_v5;";
pub const M_254_DISTRIBUTED_TRACING_V6_UP: &str = include_str!("254_add_distributed_tracing_v6.sql");
pub const M_254_DISTRIBUTED_TRACING_V6_DOWN: &str =
    "DROP TABLE IF EXISTS trace_service_dependencies_v2; DROP TABLE IF EXISTS trace_sampling_rules_v5;";
pub const M_255_DASHBOARD_REPORTING_V5_UP: &str = include_str!("255_add_dashboard_reporting_v5.sql");
pub const M_255_DASHBOARD_REPORTING_V5_DOWN: &str =
    "DROP TABLE IF EXISTS report_schedules_v3; DROP TABLE IF EXISTS dashboard_shares_v2;";
pub const M_256_PIPELINE_ACTION_REVIEWS_V3_UP: &str = include_str!("256_add_pipeline_action_reviews_v3.sql");
pub const M_256_PIPELINE_ACTION_REVIEWS_V3_DOWN: &str =
    "DROP TABLE IF EXISTS review_recommendations_v2; DROP TABLE IF EXISTS review_analytics_v2; DROP TABLE IF EXISTS review_moderation_queue_v2; DROP TABLE IF EXISTS review_helpfulness_v2; DROP TABLE IF EXISTS pipeline_action_reviews_v3;";
pub const M_257_ENVIRONMENT_DEPLOYMENT_V3_UP: &str = include_str!("257_add_environment_deployment_history_v3.sql");
pub const M_257_ENVIRONMENT_DEPLOYMENT_V3_DOWN: &str =
    "DROP TABLE IF EXISTS deployment_analytics_v3; DROP TABLE IF EXISTS deployment_comparison_v3; DROP TABLE IF EXISTS environment_deployment_history_v3;";
pub const M_258_CACHE_HIT_ANALYSIS_V2_UP: &str = include_str!("258_add_cache_hit_analysis_v2.sql");
pub const M_258_CACHE_HIT_ANALYSIS_V2_DOWN: &str =
    "DROP TABLE IF EXISTS cache_performance_insights_v2; DROP TABLE IF EXISTS cache_cost_optimization_v2; DROP TABLE IF EXISTS cache_size_tracking_v2; DROP TABLE IF EXISTS cache_hit_analysis_v2;";
pub const M_259_TEST_SUITE_METRICS_BASELINES_V2_UP: &str =
    include_str!("259_add_test_suite_metrics_baselines_v2.sql");
pub const M_259_TEST_SUITE_METRICS_BASELINES_V2_DOWN: &str =
    "DROP TABLE IF EXISTS test_suite_baselines_v2; DROP TABLE IF EXISTS test_suite_metrics_v2;";
pub const M_260_CODE_QUALITY_METRICS_THRESHOLDS_V3_UP: &str =
    include_str!("260_add_code_quality_metrics_thresholds_v3.sql");
pub const M_260_CODE_QUALITY_METRICS_THRESHOLDS_V3_DOWN: &str =
    "DROP TABLE IF EXISTS code_quality_thresholds_v2; DROP TABLE IF EXISTS code_quality_metrics_v3;";
pub const M_261_PERF_TEST_ALERTS_V3_UP: &str =
    include_str!("261_add_performance_test_alerts_v3.sql");
pub const M_261_PERF_TEST_ALERTS_V3_DOWN: &str =
    "DROP TABLE IF EXISTS performance_test_alert_history_v3; DROP TABLE IF EXISTS performance_test_alerts_v3;";
pub const M_262_API_DOCS_V7_UP: &str = include_str!("262_add_api_docs_v7.sql");
pub const M_262_API_DOCS_V7_DOWN: &str = "DROP TABLE IF EXISTS api_docs_v7;";
pub const M_263_RATE_LIMIT_TIERS_V5_UP: &str = include_str!("263_add_rate_limit_tiers_v5.sql");
pub const M_263_RATE_LIMIT_TIERS_V5_DOWN: &str = "DROP TABLE IF EXISTS rate_limit_alerts_v2; DROP TABLE IF EXISTS rate_limit_tiers_v5;";
pub const M_264_API_ANALYTICS_V8_UP: &str = include_str!("264_add_api_analytics_v8.sql");
pub const M_264_API_ANALYTICS_V8_DOWN: &str = "DROP TABLE IF EXISTS api_analytics_v8;";
pub const M_265_DATABASE_REPLICATION_V5_UP: &str = include_str!("265_add_database_replication_v5.sql");
pub const M_265_DATABASE_REPLICATION_V5_DOWN: &str =
    "DROP TABLE IF EXISTS database_replication_alerts_v3; DROP TABLE IF EXISTS database_replication_config_v3;";
pub const M_266_ENCRYPTION_V6_UP: &str = include_str!("266_add_encryption_v6.sql");
pub const M_266_ENCRYPTION_V6_DOWN: &str =
    "DROP TABLE IF EXISTS encryption_compliance_checks_v3; DROP TABLE IF EXISTS encryption_key_versions_v3;";
pub const M_267_DATA_RESIDENCY_V5_UP: &str = include_str!("267_add_data_residency_v5.sql");
pub const M_267_DATA_RESIDENCY_V5_DOWN: &str =
    "DROP TABLE IF EXISTS data_residency_compliance_v3; DROP TABLE IF EXISTS data_residency_reports_v3;";
pub const M_271_WORKFLOW_TEMPLATES_V4_UP: &str = include_str!("271_add_workflow_templates_v4.sql");
pub const M_271_WORKFLOW_TEMPLATES_V4_DOWN: &str = "DROP TABLE IF EXISTS workflow_template_reviews_v3; DROP TABLE IF EXISTS workflow_templates_v4;";
pub const M_272_AUTOMATION_RULES_V7_UP: &str = include_str!("272_add_automation_rules_v7.sql");
pub const M_272_AUTOMATION_RULES_V7_DOWN: &str = "DROP TABLE IF EXISTS automation_rules_v7;";
pub const M_273_SCHEDULED_TASK_TEMPLATES_V4_UP: &str = include_str!("273_add_scheduled_task_templates_v4.sql");
pub const M_273_SCHEDULED_TASK_TEMPLATES_V4_DOWN: &str = "DROP TABLE IF EXISTS scheduled_task_templates_v4;";
pub const M_274_LOG_AGGREGATION_V6_UP: &str = include_str!("274_add_log_aggregation_v6.sql");
pub const M_274_LOG_AGGREGATION_V6_DOWN: &str =
    "DROP TABLE IF EXISTS log_alert_rules_v3; DROP TABLE IF EXISTS log_entries_v6;";
pub const M_275_DISTRIBUTED_TRACING_V7_UP: &str = include_str!("275_add_distributed_tracing_v7.sql");
pub const M_275_DISTRIBUTED_TRACING_V7_DOWN: &str =
    "DROP TABLE IF EXISTS trace_service_dependencies_v3; DROP TABLE IF EXISTS trace_sampling_rules_v6;";
pub const M_276_DASHBOARD_REPORTING_V6_UP: &str = include_str!("276_add_dashboard_reporting_v6.sql");
pub const M_276_DASHBOARD_REPORTING_V6_DOWN: &str =
    "DROP TABLE IF EXISTS report_schedules_v4; DROP TABLE IF EXISTS dashboard_shares_v3;";
pub const M_277_PIPELINE_ACTION_REVIEWS_V4_UP: &str = include_str!("277_add_pipeline_action_reviews_v4.sql");
pub const M_277_PIPELINE_ACTION_REVIEWS_V4_DOWN: &str =
    "DROP TABLE IF EXISTS review_recommendations_v3; DROP TABLE IF EXISTS review_analytics_v3; DROP TABLE IF EXISTS review_moderation_queue_v3; DROP TABLE IF EXISTS review_helpfulness_v3; DROP TABLE IF EXISTS pipeline_action_reviews_v4;";
pub const M_278_ENVIRONMENT_DEPLOYMENT_V4_UP: &str = include_str!("278_add_environment_deployment_history_v4.sql");
pub const M_278_ENVIRONMENT_DEPLOYMENT_V4_DOWN: &str =
    "DROP TABLE IF EXISTS deployment_analytics_v4; DROP TABLE IF EXISTS deployment_comparison_v4; DROP TABLE IF EXISTS environment_deployment_history_v4;";
pub const M_279_CACHE_HIT_ANALYSIS_V3_UP: &str = include_str!("279_add_cache_hit_analysis_v3.sql");
pub const M_279_CACHE_HIT_ANALYSIS_V3_DOWN: &str =
    "DROP TABLE IF EXISTS cache_performance_insights_v3; DROP TABLE IF EXISTS cache_cost_optimization_v3; DROP TABLE IF EXISTS cache_size_tracking_v3; DROP TABLE IF EXISTS cache_hit_analysis_v3;";
pub const M_280_TEST_SUITE_METRICS_BASELINES_V3_UP: &str = include_str!("280_add_test_suite_metrics_baselines_v3.sql");
pub const M_280_TEST_SUITE_METRICS_BASELINES_V3_DOWN: &str =
    "DROP TABLE IF EXISTS test_suite_baselines_v3; DROP TABLE IF EXISTS test_suite_metrics_v3;";
pub const M_281_CODE_QUALITY_METRICS_THRESHOLDS_V4_UP: &str = include_str!("281_add_code_quality_metrics_thresholds_v4.sql");
pub const M_281_CODE_QUALITY_METRICS_THRESHOLDS_V4_DOWN: &str =
    "DROP TABLE IF EXISTS code_quality_thresholds_v3; DROP TABLE IF EXISTS code_quality_metrics_v4;";
pub const M_282_PERF_TEST_ALERTS_V4_UP: &str = include_str!("282_add_performance_test_alerts_v4.sql");
pub const M_282_PERF_TEST_ALERTS_V4_DOWN: &str =
    "DROP TABLE IF EXISTS performance_test_alert_history_v4; DROP TABLE IF EXISTS performance_test_alerts_v4;";
pub const M_283_API_DOCS_V8_UP: &str = include_str!("283_add_api_docs_v8.sql");
pub const M_283_API_DOCS_V8_DOWN: &str = "DROP TABLE IF EXISTS api_docs_v8;";
pub const M_284_RATE_LIMIT_TIERS_V6_UP: &str = include_str!("284_add_rate_limit_tiers_v6.sql");
pub const M_284_RATE_LIMIT_TIERS_V6_DOWN: &str =
    "DROP TABLE IF EXISTS rate_limit_alerts_v3; DROP TABLE IF EXISTS rate_limit_tiers_v6;";
pub const M_285_API_ANALYTICS_V9_UP: &str = include_str!("285_add_api_analytics_v9.sql");
pub const M_285_API_ANALYTICS_V9_DOWN: &str = "DROP TABLE IF EXISTS api_analytics_v9;";
pub const M_286_DATABASE_REPLICATION_V6_UP: &str =
    include_str!("286_add_database_replication_v6.sql");
pub const M_286_DATABASE_REPLICATION_V6_DOWN: &str =
    "DROP TABLE IF EXISTS database_replication_alerts_v4; DROP TABLE IF EXISTS database_replication_config_v4;";
pub const M_287_ENCRYPTION_V7_UP: &str = include_str!("287_add_encryption_v7.sql");
pub const M_287_ENCRYPTION_V7_DOWN: &str =
    "DROP TABLE IF EXISTS encryption_compliance_checks_v4; DROP TABLE IF EXISTS encryption_key_versions_v4;";
pub const M_288_DATA_RESIDENCY_V6_UP: &str = include_str!("288_add_data_residency_v6.sql");
pub const M_288_DATA_RESIDENCY_V6_DOWN: &str =
    "DROP TABLE IF EXISTS data_residency_compliance_v4; DROP TABLE IF EXISTS data_residency_reports_v4;";
pub const M_295_LOG_AGGREGATION_V7_UP: &str = include_str!("295_add_log_aggregation_v7.sql");
pub const M_295_LOG_AGGREGATION_V7_DOWN: &str =
    "DROP TABLE IF EXISTS log_alert_rules_v4; DROP TABLE IF EXISTS log_entries_v7;";
pub const M_296_DISTRIBUTED_TRACING_V8_UP: &str =
    include_str!("296_add_distributed_tracing_v8.sql");
pub const M_296_DISTRIBUTED_TRACING_V8_DOWN: &str =
    "DROP TABLE IF EXISTS trace_service_dependencies_v4; DROP TABLE IF EXISTS trace_sampling_rules_v7;";
pub const M_297_DASHBOARD_REPORTING_V7_UP: &str = include_str!("297_add_dashboard_reporting_v7.sql");
pub const M_297_DASHBOARD_REPORTING_V7_DOWN: &str =
    "DROP TABLE IF EXISTS report_schedules_v5; DROP TABLE IF EXISTS dashboard_shares_v4;";
pub const M_304_API_DOCS_V9_UP: &str = include_str!("304_add_api_docs_v9.sql");
pub const M_304_API_DOCS_V9_DOWN: &str = "DROP TABLE IF EXISTS api_docs_v9;";
pub const M_305_RATE_LIMIT_TIERS_V7_UP: &str = include_str!("305_add_rate_limit_tiers_v7.sql");
pub const M_305_RATE_LIMIT_TIERS_V7_DOWN: &str =
    "DROP TABLE IF EXISTS rate_limit_alerts_v4; DROP TABLE IF EXISTS rate_limit_tiers_v7;";
pub const M_306_API_ANALYTICS_V10_UP: &str = include_str!("306_add_api_analytics_v10.sql");
pub const M_306_API_ANALYTICS_V10_DOWN: &str = "DROP TABLE IF EXISTS api_analytics_v10;";
pub const M_307_DATABASE_REPLICATION_V7_UP: &str =
    include_str!("307_add_database_replication_v7.sql");
pub const M_307_DATABASE_REPLICATION_V7_DOWN: &str =
    "DROP TABLE IF EXISTS database_replication_alerts_v5; DROP TABLE IF EXISTS database_replication_config_v5;";
pub const M_308_ENCRYPTION_V8_UP: &str = include_str!("308_add_encryption_v8.sql");
pub const M_308_ENCRYPTION_V8_DOWN: &str =
    "DROP TABLE IF EXISTS encryption_compliance_checks_v5; DROP TABLE IF EXISTS encryption_key_versions_v5;";
pub const M_309_DATA_RESIDENCY_V7_UP: &str = include_str!("309_add_data_residency_v7.sql");
pub const M_309_DATA_RESIDENCY_V7_DOWN: &str =
    "DROP TABLE IF EXISTS data_residency_compliance_v5; DROP TABLE IF EXISTS data_residency_reports_v5;";
pub const M_310_SECURITY_SCAN_V9_UP: &str = include_str!("310_add_security_scan_v9.sql");
pub const M_310_SECURITY_SCAN_V9_DOWN: &str =
    "DROP TABLE IF EXISTS security_scan_fixes_v6; DROP TABLE IF EXISTS security_scan_rules_v8;";
pub const M_311_COMPLIANCE_FRAMEWORKS_V9_UP: &str = include_str!("311_add_compliance_frameworks_v9.sql");
pub const M_311_COMPLIANCE_FRAMEWORKS_V9_DOWN: &str =
    "DROP TABLE IF EXISTS compliance_assessments_v7; DROP TABLE IF EXISTS compliance_frameworks_v8;";
pub const M_312_AUDIT_TRAIL_V9_UP: &str = include_str!("312_add_audit_trail_v9.sql");
pub const M_312_AUDIT_TRAIL_V9_DOWN: &str = "DROP TABLE IF EXISTS audit_trail_v9;";
pub const M_325_API_DOCS_V10_UP: &str = include_str!("325_add_api_docs_v10.sql");
pub const M_325_API_DOCS_V10_DOWN: &str = "DROP TABLE IF EXISTS api_docs_v10;";
pub const M_326_RATE_LIMIT_TIERS_V8_UP: &str = include_str!("326_add_rate_limit_tiers_v8.sql");
pub const M_326_RATE_LIMIT_TIERS_V8_DOWN: &str =
    "DROP TABLE IF EXISTS rate_limit_alerts_v5; DROP TABLE IF EXISTS rate_limit_tiers_v8;";
pub const M_327_API_ANALYTICS_V11_UP: &str = include_str!("327_add_api_analytics_v11.sql");
pub const M_327_API_ANALYTICS_V11_DOWN: &str = "DROP TABLE IF EXISTS api_analytics_v11;";
pub const M_328_DATABASE_REPLICATION_V8_UP: &str =
    include_str!("328_add_database_replication_v8.sql");
pub const M_328_DATABASE_REPLICATION_V8_DOWN: &str =
    "DROP TABLE IF EXISTS database_replication_alerts_v6; DROP TABLE IF EXISTS database_replication_config_v6;";
pub const M_329_ENCRYPTION_V9_UP: &str = include_str!("329_add_encryption_v9.sql");
pub const M_329_ENCRYPTION_V9_DOWN: &str =
    "DROP TABLE IF EXISTS encryption_compliance_checks_v6; DROP TABLE IF EXISTS encryption_key_versions_v6;";
pub const M_330_DATA_RESIDENCY_V8_UP: &str = include_str!("330_add_data_residency_v8.sql");
pub const M_330_DATA_RESIDENCY_V8_DOWN: &str =
    "DROP TABLE IF EXISTS data_residency_compliance_v6; DROP TABLE IF EXISTS data_residency_reports_v6;";
pub const M_331_SECURITY_SCAN_V10_UP: &str = include_str!("331_add_security_scan_v10.sql");
pub const M_331_SECURITY_SCAN_V10_DOWN: &str =
    "DROP TABLE IF EXISTS security_scan_fixes_v7; DROP TABLE IF EXISTS security_scan_rules_v9;";
pub const M_332_COMPLIANCE_FRAMEWORKS_V10_UP: &str =
    include_str!("332_add_compliance_frameworks_v10.sql");
pub const M_332_COMPLIANCE_FRAMEWORKS_V10_DOWN: &str =
    "DROP TABLE IF EXISTS compliance_assessments_v8; DROP TABLE IF EXISTS compliance_frameworks_v9;";
pub const M_333_AUDIT_TRAIL_V10_UP: &str = include_str!("333_add_audit_trail_v10.sql");
pub const M_333_AUDIT_TRAIL_V10_DOWN: &str = "DROP TABLE IF EXISTS audit_trail_v10;";
pub const M_334_WORKFLOW_TEMPLATES_V7_UP: &str = include_str!("334_add_workflow_templates_v7.sql");
pub const M_334_WORKFLOW_TEMPLATES_V7_DOWN: &str = "DROP TABLE IF EXISTS workflow_template_reviews_v6; DROP TABLE IF EXISTS workflow_templates_v7;";
pub const M_335_AUTOMATION_RULES_V10_UP: &str = include_str!("335_add_automation_rules_v10.sql");
pub const M_335_AUTOMATION_RULES_V10_DOWN: &str = "DROP TABLE IF EXISTS automation_rules_v10;";
pub const M_336_SCHEDULED_TASK_TEMPLATES_V7_UP: &str = include_str!("336_add_scheduled_task_templates_v7.sql");
pub const M_336_SCHEDULED_TASK_TEMPLATES_V7_DOWN: &str = "DROP TABLE IF EXISTS scheduled_task_templates_v7;";
pub const M_337_LOG_AGGREGATION_V9_UP: &str = include_str!("337_add_log_aggregation_v9.sql");
pub const M_337_LOG_AGGREGATION_V9_DOWN: &str = "DROP TABLE IF EXISTS log_alert_rules_v6; DROP TABLE IF EXISTS log_entries_v9;";
pub const M_338_DISTRIBUTED_TRACING_V10_UP: &str = include_str!("338_add_distributed_tracing_v10.sql");
pub const M_338_DISTRIBUTED_TRACING_V10_DOWN: &str = "DROP TABLE IF EXISTS trace_service_dependencies_v6; DROP TABLE IF EXISTS trace_sampling_rules_v9;";
pub const M_339_DASHBOARD_REPORTING_V9_UP: &str = include_str!("339_add_dashboard_reporting_v9.sql");
pub const M_339_DASHBOARD_REPORTING_V9_DOWN: &str = "DROP TABLE IF EXISTS report_schedules_v7; DROP TABLE IF EXISTS dashboard_shares_v6;";
pub const M_340_PIPELINE_ACTION_REVIEWS_V7_UP: &str = include_str!("340_add_pipeline_action_reviews_v7.sql");
pub const M_340_PIPELINE_ACTION_REVIEWS_V7_DOWN: &str =
    "DROP TABLE IF EXISTS review_recommendations_v7; DROP TABLE IF EXISTS review_analytics_v7; DROP TABLE IF EXISTS review_moderation_queue_v7; DROP TABLE IF EXISTS review_helpfulness_v7; DROP TABLE IF EXISTS pipeline_action_reviews_v7;";
pub const M_341_ENVIRONMENT_DEPLOYMENT_V7_UP: &str = include_str!("341_add_environment_deployment_history_v7.sql");
pub const M_341_ENVIRONMENT_DEPLOYMENT_V7_DOWN: &str =
    "DROP TABLE IF EXISTS deployment_analytics_v7; DROP TABLE IF EXISTS deployment_comparison_v7; DROP TABLE IF EXISTS environment_deployment_history_v7;";
pub const M_342_CACHE_HIT_ANALYSIS_V6_UP: &str = include_str!("342_add_cache_hit_analysis_v6.sql");
pub const M_342_CACHE_HIT_ANALYSIS_V6_DOWN: &str =
    "DROP TABLE IF EXISTS cache_performance_insights_v6; DROP TABLE IF EXISTS cache_cost_optimization_v6; DROP TABLE IF EXISTS cache_size_tracking_v6; DROP TABLE IF EXISTS cache_hit_analysis_v6;";
pub const M_343_TEST_SUITE_MANAGEMENT_V9_UP: &str = include_str!("343_add_test_suite_management_v9.sql");
pub const M_343_TEST_SUITE_MANAGEMENT_V9_DOWN: &str =
    "DROP TABLE IF EXISTS test_suite_baselines_v6; DROP TABLE IF EXISTS test_suite_metrics_v6;";
pub const M_344_CODE_QUALITY_RULES_V9_UP: &str = include_str!("344_add_code_quality_rules_v9.sql");
pub const M_344_CODE_QUALITY_RULES_V9_DOWN: &str =
    "DROP TABLE IF EXISTS code_quality_thresholds_v6; DROP TABLE IF EXISTS code_quality_metrics_v7;";
pub const M_345_PERFORMANCE_TESTING_V10_UP: &str = include_str!("345_add_performance_testing_v10.sql");
pub const M_345_PERFORMANCE_TESTING_V10_DOWN: &str =
    "DROP TABLE IF EXISTS performance_test_alert_history_v7; DROP TABLE IF EXISTS performance_test_alerts_v7;";
pub const M_346_API_DOCS_V11_UP: &str = include_str!("346_add_api_docs_v11.sql");
pub const M_346_API_DOCS_V11_DOWN: &str = "DROP TABLE IF EXISTS api_docs_v11;";
pub const M_347_RATE_LIMIT_TIERS_V9_UP: &str = include_str!("347_add_rate_limit_tiers_v9.sql");
pub const M_347_RATE_LIMIT_TIERS_V9_DOWN: &str =
    "DROP TABLE IF EXISTS rate_limit_alerts_v6; DROP TABLE IF EXISTS rate_limit_tiers_v9;";
pub const M_348_API_ANALYTICS_V12_UP: &str = include_str!("348_add_api_analytics_v12.sql");
pub const M_348_API_ANALYTICS_V12_DOWN: &str = "DROP TABLE IF EXISTS api_analytics_v12;";
pub const M_361_PIPELINE_ACTION_REVIEWS_V8_UP: &str = include_str!("361_add_pipeline_action_reviews_v8.sql");
pub const M_361_PIPELINE_ACTION_REVIEWS_V8_DOWN: &str =
    "DROP TABLE IF EXISTS review_recommendations_v8; DROP TABLE IF EXISTS review_analytics_v8; DROP TABLE IF EXISTS review_moderation_queue_v8; DROP TABLE IF EXISTS review_helpfulness_v8; DROP TABLE IF EXISTS pipeline_action_reviews_v8;";
pub const M_362_ENVIRONMENT_DEPLOYMENT_V8_UP: &str = include_str!("362_add_environment_deployment_history_v8.sql");
pub const M_362_ENVIRONMENT_DEPLOYMENT_V8_DOWN: &str =
    "DROP TABLE IF EXISTS deployment_analytics_v8; DROP TABLE IF EXISTS deployment_comparison_v8; DROP TABLE IF EXISTS environment_deployment_history_v8;";
pub const M_363_CACHE_HIT_ANALYSIS_V7_UP: &str = include_str!("363_add_cache_hit_analysis_v7.sql");
pub const M_363_CACHE_HIT_ANALYSIS_V7_DOWN: &str =
    "DROP TABLE IF EXISTS cache_performance_insights_v7; DROP TABLE IF EXISTS cache_cost_optimization_v7; DROP TABLE IF EXISTS cache_size_tracking_v7; DROP TABLE IF EXISTS cache_hit_analysis_v7;";
pub const M_364_TEST_SUITE_MANAGEMENT_V10_UP: &str = include_str!("364_add_test_suite_management_v10.sql");
pub const M_364_TEST_SUITE_MANAGEMENT_V10_DOWN: &str =
    "DROP TABLE IF EXISTS test_suite_baselines_v7; DROP TABLE IF EXISTS test_suite_metrics_v7;";
pub const M_365_CODE_QUALITY_RULES_V10_UP: &str = include_str!("365_add_code_quality_rules_v10.sql");
pub const M_365_CODE_QUALITY_RULES_V10_DOWN: &str =
    "DROP TABLE IF EXISTS code_quality_thresholds_v7; DROP TABLE IF EXISTS code_quality_metrics_v8;";
pub const M_366_PERFORMANCE_TESTING_V11_UP: &str = include_str!("366_add_performance_testing_v11.sql");
pub const M_366_PERFORMANCE_TESTING_V11_DOWN: &str =
    "DROP TABLE IF EXISTS performance_test_alert_history_v8; DROP TABLE IF EXISTS performance_test_alerts_v8;";
pub const M_370_DATABASE_REPLICATION_V10_UP: &str =
    include_str!("370_add_database_replication_v10.sql");
pub const M_370_DATABASE_REPLICATION_V10_DOWN: &str =
    "DROP TABLE IF EXISTS database_replication_alerts_v8; DROP TABLE IF EXISTS database_replication_config_v8;";
pub const M_371_ENCRYPTION_V11_UP: &str =
    include_str!("371_add_encryption_v11.sql");
pub const M_371_ENCRYPTION_V11_DOWN: &str =
    "DROP TABLE IF EXISTS encryption_compliance_checks_v8; DROP TABLE IF EXISTS encryption_key_versions_v8;";
pub const M_372_DATA_RESIDENCY_V10_UP: &str =
    include_str!("372_add_data_residency_v10.sql");
pub const M_372_DATA_RESIDENCY_V10_DOWN: &str =
    "DROP TABLE IF EXISTS data_residency_compliance_v8; DROP TABLE IF EXISTS data_residency_reports_v8;";

pub const M_352_SECURITY_SCAN_V11_UP: &str = include_str!("352_add_security_scan_v11.sql");
pub const M_352_SECURITY_SCAN_V11_DOWN: &str =
    "DROP TABLE IF EXISTS security_scan_fixes_v8; DROP TABLE IF EXISTS security_scan_rules_v10;";
pub const M_353_COMPLIANCE_FRAMEWORKS_V11_UP: &str =
    include_str!("353_add_compliance_frameworks_v11.sql");
pub const M_353_COMPLIANCE_FRAMEWORKS_V11_DOWN: &str =
    "DROP TABLE IF EXISTS compliance_assessments_v9; DROP TABLE IF EXISTS compliance_frameworks_v10;";
pub const M_354_AUDIT_TRAIL_V11_UP: &str = include_str!("354_add_audit_trail_v11.sql");
pub const M_354_AUDIT_TRAIL_V11_DOWN: &str = "DROP TABLE IF EXISTS audit_trail_v11;";
pub const M_355_WORKFLOW_TEMPLATES_V8_UP: &str = include_str!("355_add_workflow_templates_v8.sql");
pub const M_355_WORKFLOW_TEMPLATES_V8_DOWN: &str = "DROP TABLE IF EXISTS workflow_template_reviews_v7; DROP TABLE IF EXISTS workflow_templates_v8;";
pub const M_356_AUTOMATION_RULES_V11_UP: &str = include_str!("356_add_automation_rules_v11.sql");
pub const M_356_AUTOMATION_RULES_V11_DOWN: &str = "DROP TABLE IF EXISTS automation_rules_v11;";
pub const M_357_SCHEDULED_TASK_TEMPLATES_V8_UP: &str = include_str!("357_add_scheduled_task_templates_v8.sql");
pub const M_357_SCHEDULED_TASK_TEMPLATES_V8_DOWN: &str = "DROP TABLE IF EXISTS scheduled_task_templates_v8;";
pub const M_376_WORKFLOW_TEMPLATES_V9_UP: &str = include_str!("376_add_workflow_templates_v9.sql");
pub const M_376_WORKFLOW_TEMPLATES_V9_DOWN: &str = "DROP TABLE IF EXISTS workflow_template_reviews_v8; DROP TABLE IF EXISTS workflow_templates_v9;";
pub const M_377_AUTOMATION_RULES_V12_UP: &str = include_str!("377_add_automation_rules_v12.sql");
pub const M_377_AUTOMATION_RULES_V12_DOWN: &str = "DROP TABLE IF EXISTS automation_rules_v12;";
pub const M_378_SCHEDULED_TASK_TEMPLATES_V9_UP: &str = include_str!("378_add_scheduled_task_templates_v9.sql");
pub const M_378_SCHEDULED_TASK_TEMPLATES_V9_DOWN: &str = "DROP TABLE IF EXISTS scheduled_task_templates_v9;";
pub const M_388_API_DOCS_V13_UP: &str = include_str!("388_add_api_docs_v13.sql");
pub const M_388_API_DOCS_V13_DOWN: &str = "DROP TABLE IF EXISTS api_docs_v13;";
pub const M_389_RATE_LIMIT_TIERS_V11_UP: &str = include_str!("389_add_rate_limit_tiers_v11.sql");
pub const M_389_RATE_LIMIT_TIERS_V11_DOWN: &str = "DROP TABLE IF EXISTS rate_limit_alerts_v8; DROP TABLE IF EXISTS rate_limit_tiers_v11;";
pub const M_390_API_ANALYTICS_V14_UP: &str = include_str!("390_add_api_analytics_v14.sql");
pub const M_390_API_ANALYTICS_V14_DOWN: &str = "DROP TABLE IF EXISTS api_analytics_v14;";
pub const M_400_LOG_AGGREGATION_V12_UP: &str = include_str!("400_add_log_aggregation_v12.sql");
pub const M_400_LOG_AGGREGATION_V12_DOWN: &str =
    "DROP TABLE IF EXISTS log_alert_rules_v9; DROP TABLE IF EXISTS log_entries_v12;";
pub const M_401_DISTRIBUTED_TRACING_V13_UP: &str =
    include_str!("401_add_distributed_tracing_v13.sql");
pub const M_401_DISTRIBUTED_TRACING_V13_DOWN: &str =
    "DROP TABLE IF EXISTS trace_service_dependencies_v9; DROP TABLE IF EXISTS trace_sampling_rules_v12;";
pub const M_402_DASHBOARD_REPORTING_V12_UP: &str =
    include_str!("402_add_dashboard_reporting_v12.sql");
pub const M_402_DASHBOARD_REPORTING_V12_DOWN: &str =
    "DROP TABLE IF EXISTS report_schedules_v10; DROP TABLE IF EXISTS dashboard_shares_v9;";
pub const M_406_TEST_SUITE_MANAGEMENT_V12_UP: &str =
    include_str!("406_add_test_suite_management_v12.sql");
pub const M_406_TEST_SUITE_MANAGEMENT_V12_DOWN: &str =
    "DROP TABLE IF EXISTS test_suite_baselines_v9; DROP TABLE IF EXISTS test_suite_metrics_v9;";
pub const M_407_CODE_QUALITY_RULES_V12_UP: &str =
    include_str!("407_add_code_quality_rules_v12.sql");
pub const M_407_CODE_QUALITY_RULES_V12_DOWN: &str =
    "DROP TABLE IF EXISTS code_quality_thresholds_v9; DROP TABLE IF EXISTS code_quality_metrics_v10;";
pub const M_408_PERFORMANCE_TESTING_V13_UP: &str =
    include_str!("408_add_performance_testing_v13.sql");
pub const M_408_PERFORMANCE_TESTING_V13_DOWN: &str =
    "DROP TABLE IF EXISTS performance_test_alert_history_v10; DROP TABLE IF EXISTS performance_test_alerts_v10;";
pub const M_409_API_DOCS_V14_UP: &str = include_str!("409_add_api_docs_v14.sql");
pub const M_409_API_DOCS_V14_DOWN: &str = "DROP TABLE IF EXISTS api_docs_v14;";
pub const M_410_RATE_LIMIT_TIERS_V12_UP: &str = include_str!("410_add_rate_limit_tiers_v12.sql");
pub const M_410_RATE_LIMIT_TIERS_V12_DOWN: &str = "DROP TABLE IF EXISTS rate_limit_alerts_v9; DROP TABLE IF EXISTS rate_limit_tiers_v12;";
pub const M_411_API_ANALYTICS_V15_UP: &str = include_str!("411_add_api_analytics_v15.sql");
pub const M_411_API_ANALYTICS_V15_DOWN: &str = "DROP TABLE IF EXISTS api_analytics_v15;";
pub const M_430_API_DOCS_V15_UP: &str = include_str!("430_add_api_docs_v15.sql");
pub const M_430_API_DOCS_V15_DOWN: &str = "DROP TABLE IF EXISTS api_docs_v15;";
pub const M_431_RATE_LIMIT_TIERS_V13_UP: &str = include_str!("431_add_rate_limit_tiers_v13.sql");
pub const M_431_RATE_LIMIT_TIERS_V13_DOWN: &str = "DROP TABLE IF EXISTS rate_limit_alerts_v10; DROP TABLE IF EXISTS rate_limit_tiers_v13;";
pub const M_432_API_ANALYTICS_V16_UP: &str = include_str!("432_add_api_analytics_v16.sql");
pub const M_432_API_ANALYTICS_V16_DOWN: &str = "DROP TABLE IF EXISTS api_analytics_v16;";
pub const M_451_API_DOCS_V16_UP: &str = include_str!("451_add_api_docs_v16.sql");
pub const M_451_API_DOCS_V16_DOWN: &str = "DROP TABLE IF EXISTS api_docs_v16;";
pub const M_452_RATE_LIMIT_TIERS_V14_UP: &str = include_str!("452_add_rate_limit_tiers_v14.sql");
pub const M_452_RATE_LIMIT_TIERS_V14_DOWN: &str = "DROP TABLE IF EXISTS rate_limit_alerts_v11; DROP TABLE IF EXISTS rate_limit_tiers_v14;";
pub const M_453_API_ANALYTICS_V17_UP: &str = include_str!("453_add_api_analytics_v17.sql");
pub const M_453_API_ANALYTICS_V17_DOWN: &str = "DROP TABLE IF EXISTS api_analytics_v17;";
pub const M_433_DATABASE_REPLICATION_V13_UP: &str =
    include_str!("433_add_database_replication_v13.sql");
pub const M_433_DATABASE_REPLICATION_V13_DOWN: &str =
    "DROP TABLE IF EXISTS database_replication_alerts_v11; DROP TABLE IF EXISTS database_replication_config_v11;";
pub const M_434_ENCRYPTION_V14_UP: &str = include_str!("434_add_encryption_v14.sql");
pub const M_434_ENCRYPTION_V14_DOWN: &str =
    "DROP TABLE IF EXISTS encryption_compliance_checks_v11; DROP TABLE IF EXISTS encryption_key_versions_v11;";
pub const M_435_DATA_RESIDENCY_V13_UP: &str = include_str!("435_add_data_residency_v13.sql");
pub const M_435_DATA_RESIDENCY_V13_DOWN: &str =
    "DROP TABLE IF EXISTS data_residency_compliance_v11; DROP TABLE IF EXISTS data_residency_reports_v11;";
pub const M_412_DATABASE_REPLICATION_V12_UP: &str =
    include_str!("412_add_database_replication_v12.sql");
pub const M_412_DATABASE_REPLICATION_V12_DOWN: &str =
    "DROP TABLE IF EXISTS database_replication_alerts_v10; DROP TABLE IF EXISTS database_replication_config_v10;";
pub const M_413_ENCRYPTION_V13_UP: &str = include_str!("413_add_encryption_v13.sql");
pub const M_413_ENCRYPTION_V13_DOWN: &str =
    "DROP TABLE IF EXISTS encryption_compliance_checks_v10; DROP TABLE IF EXISTS encryption_key_versions_v10;";
pub const M_414_DATA_RESIDENCY_V12_UP: &str = include_str!("414_add_data_residency_v12.sql");
pub const M_414_DATA_RESIDENCY_V12_DOWN: &str =
    "DROP TABLE IF EXISTS data_residency_compliance_v10; DROP TABLE IF EXISTS data_residency_reports_v10;";
pub const M_418_WORKFLOW_TEMPLATES_V11_UP: &str = include_str!("418_add_workflow_templates_v11.sql");
pub const M_418_WORKFLOW_TEMPLATES_V11_DOWN: &str = "DROP TABLE IF EXISTS workflow_template_reviews_v10; DROP TABLE IF EXISTS workflow_templates_v11;";
pub const M_419_AUTOMATION_RULES_V14_UP: &str = include_str!("419_add_automation_rules_v14.sql");
pub const M_419_AUTOMATION_RULES_V14_DOWN: &str = "DROP TABLE IF EXISTS automation_rules_v14;";
pub const M_420_SCHEDULED_TASK_TEMPLATES_V11_UP: &str = include_str!("420_add_scheduled_task_templates_v11.sql");
pub const M_420_SCHEDULED_TASK_TEMPLATES_V11_DOWN: &str = "DROP TABLE IF EXISTS scheduled_task_templates_v11;";
pub const M_421_LOG_AGGREGATION_V13_UP: &str = include_str!("421_add_log_aggregation_v13.sql");
pub const M_421_LOG_AGGREGATION_V13_DOWN: &str =
    "DROP TABLE IF EXISTS log_alert_rules_v10; DROP TABLE IF EXISTS log_entries_v13;";
pub const M_422_DISTRIBUTED_TRACING_V14_UP: &str = include_str!("422_add_distributed_tracing_v14.sql");
pub const M_422_DISTRIBUTED_TRACING_V14_DOWN: &str =
    "DROP TABLE IF EXISTS trace_service_dependencies_v10; DROP TABLE IF EXISTS trace_sampling_rules_v13;";
pub const M_423_DASHBOARD_REPORTING_V13_UP: &str = include_str!("423_add_dashboard_reporting_v13.sql");
pub const M_423_DASHBOARD_REPORTING_V13_DOWN: &str =
    "DROP TABLE IF EXISTS report_schedules_v11; DROP TABLE IF EXISTS dashboard_shares_v10;";
pub const M_427_TEST_SUITE_MANAGEMENT_V13_UP: &str =
    include_str!("427_add_test_suite_management_v13.sql");
pub const M_427_TEST_SUITE_MANAGEMENT_V13_DOWN: &str =
    "DROP TABLE IF EXISTS test_suite_baselines_v10; DROP TABLE IF EXISTS test_suite_metrics_v10;";
pub const M_428_CODE_QUALITY_RULES_V13_UP: &str =
    include_str!("428_add_code_quality_rules_v13.sql");
pub const M_428_CODE_QUALITY_RULES_V13_DOWN: &str =
    "DROP TABLE IF EXISTS code_quality_thresholds_v10; DROP TABLE IF EXISTS code_quality_metrics_v11;";
pub const M_429_PERFORMANCE_TESTING_V14_UP: &str =
    include_str!("429_add_performance_testing_v14.sql");
pub const M_429_PERFORMANCE_TESTING_V14_DOWN: &str =
    "DROP TABLE IF EXISTS performance_test_alert_history_v11; DROP TABLE IF EXISTS performance_test_alerts_v11;";
pub const M_442_LOG_AGGREGATION_V14_UP: &str = include_str!("442_add_log_aggregation_v14.sql");
pub const M_442_LOG_AGGREGATION_V14_DOWN: &str =
    "DROP TABLE IF EXISTS log_alert_rules_v11; DROP TABLE IF EXISTS log_entries_v14;";
pub const M_443_DISTRIBUTED_TRACING_V15_UP: &str = include_str!("443_add_distributed_tracing_v15.sql");
pub const M_443_DISTRIBUTED_TRACING_V15_DOWN: &str =
    "DROP TABLE IF EXISTS trace_service_dependencies_v11; DROP TABLE IF EXISTS trace_sampling_rules_v14;";
pub const M_444_DASHBOARD_REPORTING_V14_UP: &str = include_str!("444_add_dashboard_reporting_v14.sql");
pub const M_444_DASHBOARD_REPORTING_V14_DOWN: &str =
    "DROP TABLE IF EXISTS report_schedules_v12; DROP TABLE IF EXISTS dashboard_shares_v11;";

pub const M_448_TEST_SUITE_METRICS_BASELINES_V11_UP: &str =
    include_str!("448_add_test_suite_metrics_baselines_v11.sql");
pub const M_448_TEST_SUITE_METRICS_BASELINES_V11_DOWN: &str =
    "DROP TABLE IF EXISTS test_suite_baselines_v11; DROP TABLE IF EXISTS test_suite_metrics_v11;";

pub const M_449_CODE_QUALITY_METRICS_V12_THRESHOLDS_V11_UP: &str =
    include_str!("449_add_code_quality_metrics_v12_thresholds_v11.sql");
pub const M_449_CODE_QUALITY_METRICS_V12_THRESHOLDS_V11_DOWN: &str =
    "DROP TABLE IF EXISTS code_quality_thresholds_v11; DROP TABLE IF EXISTS code_quality_metrics_v12;";

pub const M_450_PERFORMANCE_TEST_ALERTS_V12_UP: &str =
    include_str!("450_add_performance_test_alerts_v12.sql");
pub const M_450_PERFORMANCE_TEST_ALERTS_V12_DOWN: &str =
    "DROP TABLE IF EXISTS performance_test_alert_history_v12; DROP TABLE IF EXISTS performance_test_alerts_v12;";
pub const M_460_WORKFLOW_TEMPLATES_V13_UP: &str = include_str!("460_add_workflow_templates_v13.sql");
pub const M_460_WORKFLOW_TEMPLATES_V13_DOWN: &str = "DROP TABLE IF EXISTS workflow_template_reviews_v12; DROP TABLE IF EXISTS workflow_templates_v13;";
pub const M_461_AUTOMATION_RULES_V16_UP: &str = include_str!("461_add_automation_rules_v16.sql");
pub const M_461_AUTOMATION_RULES_V16_DOWN: &str = "DROP TABLE IF EXISTS automation_rules_v16;";
pub const M_462_SCHEDULED_TASK_TEMPLATES_V13_UP: &str = include_str!("462_add_scheduled_task_templates_v13.sql");
pub const M_462_SCHEDULED_TASK_TEMPLATES_V13_DOWN: &str = "DROP TABLE IF EXISTS scheduled_task_templates_v13;";

pub const M_481_WORKFLOW_TEMPLATES_V14_UP: &str = include_str!("481_add_workflow_templates_v14.sql");
pub const M_481_WORKFLOW_TEMPLATES_V14_DOWN: &str = "DROP TABLE IF EXISTS workflow_template_reviews_v13; DROP TABLE IF EXISTS workflow_templates_v14;";
pub const M_482_AUTOMATION_RULES_V17_UP: &str = include_str!("482_add_automation_rules_v17.sql");
pub const M_482_AUTOMATION_RULES_V17_DOWN: &str = "DROP TABLE IF EXISTS automation_rules_v17;";
pub const M_483_SCHEDULED_TASK_TEMPLATES_V14_UP: &str = include_str!("483_add_scheduled_task_templates_v14.sql");
pub const M_483_SCHEDULED_TASK_TEMPLATES_V14_DOWN: &str = "DROP TABLE IF EXISTS scheduled_task_templates_v14;";

pub const M_469_TEST_SUITE_MANAGEMENT_V15_UP: &str =
    include_str!("469_add_test_suite_management_v15.sql");
pub const M_469_TEST_SUITE_MANAGEMENT_V15_DOWN: &str =
    "DROP TABLE IF EXISTS test_suite_baselines_v12; DROP TABLE IF EXISTS test_suite_metrics_v12;";
pub const M_470_CODE_QUALITY_RULES_V15_UP: &str =
    include_str!("470_add_code_quality_rules_v15.sql");
pub const M_470_CODE_QUALITY_RULES_V15_DOWN: &str =
    "DROP TABLE IF EXISTS code_quality_thresholds_v12; DROP TABLE IF EXISTS code_quality_metrics_v13;";
pub const M_471_PERFORMANCE_TESTING_V16_UP: &str =
    include_str!("471_add_performance_testing_v16.sql");
pub const M_471_PERFORMANCE_TESTING_V16_DOWN: &str =
    "DROP TABLE IF EXISTS performance_test_alert_history_v13; DROP TABLE IF EXISTS performance_test_alerts_v13;";
pub const M_487_PIPELINE_ACTION_REVIEWS_V14_UP: &str =
    include_str!("487_add_pipeline_action_reviews_v14.sql");
pub const M_487_PIPELINE_ACTION_REVIEWS_V14_DOWN: &str =
    "DROP TABLE IF EXISTS pipeline_action_reviews_v14;";
pub const M_488_ENVIRONMENT_DEPLOYMENT_V14_UP: &str =
    include_str!("488_add_environment_deployment_history_v14.sql");
pub const M_488_ENVIRONMENT_DEPLOYMENT_V14_DOWN: &str =
    "DROP TABLE IF EXISTS environment_deployment_history_v14;";
pub const M_489_CACHE_HIT_ANALYSIS_V13_UP: &str =
    include_str!("489_add_cache_hit_analysis_v13.sql");
pub const M_489_CACHE_HIT_ANALYSIS_V13_DOWN: &str =
    "DROP TABLE IF EXISTS cache_hit_analysis_v13;";

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
        self.add_migration(Migration {
            version: 115,
            name: "add_code_intelligence".into(),
            up_sql: M_115_CODE_INTELLIGENCE_UP.into(),
            down_sql: M_115_CODE_INTELLIGENCE_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 116,
            name: "add_code_search_v2".into(),
            up_sql: M_116_CODE_SEARCH_V2_UP.into(),
            down_sql: M_116_CODE_SEARCH_V2_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 117,
            name: "add_code_formatters".into(),
            up_sql: M_117_CODE_FORMATTERS_UP.into(),
            down_sql: M_117_CODE_FORMATTERS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 118,
            name: "add_pipeline_templates".into(),
            up_sql: M_118_PIPELINE_TEMPLATES_UP.into(),
            down_sql: M_118_PIPELINE_TEMPLATES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 119,
            name: "add_pipeline_analytics".into(),
            up_sql: M_119_PIPELINE_ANALYTICS_UP.into(),
            down_sql: M_119_PIPELINE_ANALYTICS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 120,
            name: "add_multi_project_pipelines".into(),
            up_sql: M_120_MULTI_PROJECT_PIPELINES_UP.into(),
            down_sql: M_120_MULTI_PROJECT_PIPELINES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 121,
            name: "add_security_scans_v2".into(),
            up_sql: M_121_SECURITY_SCANS_V2_UP.into(),
            down_sql: M_121_SECURITY_SCANS_V2_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 122,
            name: "add_compliance_frameworks".into(),
            up_sql: M_122_COMPLIANCE_FRAMEWORKS_UP.into(),
            down_sql: M_122_COMPLIANCE_FRAMEWORKS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 123,
            name: "add_audit_trail".into(),
            up_sql: M_123_AUDIT_TRAIL_UP.into(),
            down_sql: M_123_AUDIT_TRAIL_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 124,
            name: "add_api_documentation".into(),
            up_sql: M_124_API_DOCUMENTATION_UP.into(),
            down_sql: M_124_API_DOCUMENTATION_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 125,
            name: "add_api_versions".into(),
            up_sql: M_125_API_VERSIONS_UP.into(),
            down_sql: M_125_API_VERSIONS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 126,
            name: "add_api_analytics_v2".into(),
            up_sql: M_126_API_ANALYTICS_V2_UP.into(),
            down_sql: M_126_API_ANALYTICS_V2_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 127,
            name: "add_deployment_strategies".into(),
            up_sql: M_127_DEPLOYMENT_STRATEGIES_UP.into(),
            down_sql: M_127_DEPLOYMENT_STRATEGIES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 128,
            name: "add_infrastructure".into(),
            up_sql: M_128_INFRASTRUCTURE_UP.into(),
            down_sql: M_128_INFRASTRUCTURE_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 129,
            name: "add_service_mesh".into(),
            up_sql: M_129_SERVICE_MESH_UP.into(),
            down_sql: M_129_SERVICE_MESH_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 130,
            name: "add_test_coverage".into(),
            up_sql: M_130_TEST_COVERAGE_UP.into(),
            down_sql: M_130_TEST_COVERAGE_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 131,
            name: "add_code_quality_metrics".into(),
            up_sql: M_131_CODE_QUALITY_METRICS_UP.into(),
            down_sql: M_131_CODE_QUALITY_METRICS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 132,
            name: "add_performance_tests".into(),
            up_sql: M_132_PERFORMANCE_TESTS_UP.into(),
            down_sql: M_132_PERFORMANCE_TESTS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 133,
            name: "add_pipeline_actions".into(),
            up_sql: M_133_PIPELINE_ACTIONS_UP.into(),
            down_sql: M_133_PIPELINE_ACTIONS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 134,
            name: "add_pipeline_environments_v2".into(),
            up_sql: M_134_PIPELINE_ENVIRONMENTS_V2_UP.into(),
            down_sql: M_134_PIPELINE_ENVIRONMENTS_V2_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 135,
            name: "add_pipeline_caches_v2".into(),
            up_sql: M_135_PIPELINE_CACHES_V2_UP.into(),
            down_sql: M_135_PIPELINE_CACHES_V2_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 136,
            name: "add_database_backup_recovery".into(),
            up_sql: M_136_DATABASE_BACKUP_RECOVERY_UP.into(),
            down_sql: M_136_DATABASE_BACKUP_RECOVERY_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 137,
            name: "add_data_archives".into(),
            up_sql: M_137_DATA_ARCHIVES_UP.into(),
            down_sql: M_137_DATA_ARCHIVES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 138,
            name: "add_data_migrations".into(),
            up_sql: M_138_DATA_MIGRATIONS_UP.into(),
            down_sql: M_138_DATA_MIGRATIONS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 139,
            name: "add_network_policies".into(),
            up_sql: M_139_NETWORK_POLICIES_UP.into(),
            down_sql: M_139_NETWORK_POLICIES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 140,
            name: "add_encryption_at_rest".into(),
            up_sql: M_140_ENCRYPTION_AT_REST_UP.into(),
            down_sql: M_140_ENCRYPTION_AT_REST_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 141,
            name: "add_access_control_lists".into(),
            up_sql: M_141_ACCESS_CONTROL_LISTS_UP.into(),
            down_sql: M_141_ACCESS_CONTROL_LISTS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 142,
            name: "add_workflows".into(),
            up_sql: M_142_WORKFLOWS_UP.into(),
            down_sql: M_142_WORKFLOWS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 143,
            name: "add_automation_rules".into(),
            up_sql: M_143_AUTOMATION_RULES_UP.into(),
            down_sql: M_143_AUTOMATION_RULES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 144,
            name: "add_scheduled_tasks".into(),
            up_sql: M_144_SCHEDULED_TASKS_UP.into(),
            down_sql: M_144_SCHEDULED_TASKS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 145,
            name: "add_log_aggregation".into(),
            up_sql: M_145_LOG_AGGREGATION_UP.into(),
            down_sql: M_145_LOG_AGGREGATION_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 146,
            name: "add_trace_sampling_rules".into(),
            up_sql: M_146_TRACE_SAMPLING_RULES_UP.into(),
            down_sql: M_146_TRACE_SAMPLING_RULES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 147,
            name: "add_dashboard_reporting".into(),
            up_sql: M_147_DASHBOARD_REPORTING_UP.into(),
            down_sql: M_147_DASHBOARD_REPORTING_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 148,
            name: "add_pipeline_secrets_v2".into(),
            up_sql: M_148_PIPELINE_SECRETS_V2_UP.into(),
            down_sql: M_148_PIPELINE_SECRETS_V2_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 149,
            name: "add_pipeline_runners_v2".into(),
            up_sql: M_149_PIPELINE_RUNNERS_V2_UP.into(),
            down_sql: M_149_PIPELINE_RUNNERS_V2_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 150,
            name: "add_environment_variables".into(),
            up_sql: M_150_ENVIRONMENT_VARIABLES_UP.into(),
            down_sql: M_150_ENVIRONMENT_VARIABLES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 154,
            name: "add_api_docs_v2".into(),
            up_sql: M_154_API_DOCS_V2_UP.into(),
            down_sql: M_154_API_DOCS_V2_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 155,
            name: "add_rate_limit_tiers".into(),
            up_sql: M_155_RATE_LIMIT_TIERS_UP.into(),
            down_sql: M_155_RATE_LIMIT_TIERS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 156,
            name: "add_api_analytics_v3".into(),
            up_sql: M_156_API_ANALYTICS_V3_UP.into(),
            down_sql: M_156_API_ANALYTICS_V3_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 157,
            name: "add_database_replication".into(),
            up_sql: M_157_DATABASE_REPLICATION_UP.into(),
            down_sql: M_157_DATABASE_REPLICATION_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 158,
            name: "add_encryption_policies".into(),
            up_sql: M_158_ENCRYPTION_POLICIES_UP.into(),
            down_sql: M_158_ENCRYPTION_POLICIES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 159,
            name: "add_data_residency".into(),
            up_sql: M_159_DATA_RESIDENCY_UP.into(),
            down_sql: M_159_DATA_RESIDENCY_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 160,
            name: "add_security_scan_rules".into(),
            up_sql: M_160_SECURITY_SCAN_RULES_UP.into(),
            down_sql: M_160_SECURITY_SCAN_RULES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 161,
            name: "add_compliance_requirements".into(),
            up_sql: M_161_COMPLIANCE_REQUIREMENTS_UP.into(),
            down_sql: M_161_COMPLIANCE_REQUIREMENTS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 162,
            name: "add_audit_trail_v2".into(),
            up_sql: M_162_AUDIT_TRAIL_V2_UP.into(),
            down_sql: M_162_AUDIT_TRAIL_V2_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 166,
            name: "add_firewall_rules".into(),
            up_sql: M_166_FIREWALL_RULES_UP.into(),
            down_sql: M_166_FIREWALL_RULES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 167,
            name: "add_intrusion_detections".into(),
            up_sql: M_167_INTRUSION_DETECTIONS_UP.into(),
            down_sql: M_167_INTRUSION_DETECTIONS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 168,
            name: "add_ddos_protection".into(),
            up_sql: M_168_DDOS_PROTECTION_UP.into(),
            down_sql: M_168_DDOS_PROTECTION_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 169,
            name: "add_object_storage".into(),
            up_sql: M_169_OBJECT_STORAGE_UP.into(),
            down_sql: M_169_OBJECT_STORAGE_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 170,
            name: "add_backup_encryption".into(),
            up_sql: M_170_BACKUP_ENCRYPTION_UP.into(),
            down_sql: M_170_BACKUP_ENCRYPTION_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 171,
            name: "add_data_retention".into(),
            up_sql: M_171_DATA_RETENTION_UP.into(),
            down_sql: M_171_DATA_RETENTION_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 172,
            name: "add_api_docs_v3".into(),
            up_sql: M_172_API_DOCS_V3_UP.into(),
            down_sql: M_172_API_DOCS_V3_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 173,
            name: "add_api_webhooks_v2".into(),
            up_sql: M_173_API_WEBHOOKS_V2_UP.into(),
            down_sql: M_173_API_WEBHOOKS_V2_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 174,
            name: "add_api_analytics_v4".into(),
            up_sql: M_174_API_ANALYTICS_V4_UP.into(),
            down_sql: M_174_API_ANALYTICS_V4_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 175,
            name: "add_deployment_strategy_configs_logs".into(),
            up_sql: M_175_DEPLOYMENT_STRATEGY_CONFIGS_LOGS_UP.into(),
            down_sql: M_175_DEPLOYMENT_STRATEGY_CONFIGS_LOGS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 176,
            name: "add_infrastructure_modules".into(),
            up_sql: M_176_INFRASTRUCTURE_MODULES_UP.into(),
            down_sql: M_176_INFRASTRUCTURE_MODULES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 177,
            name: "add_service_mesh_policies_metrics".into(),
            up_sql: M_177_SERVICE_MESH_POLICIES_METRICS_UP.into(),
            down_sql: M_177_SERVICE_MESH_POLICIES_METRICS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 178,
            name: "add_test_coverage_v2".into(),
            up_sql: M_178_TEST_COVERAGE_V2_UP.into(),
            down_sql: M_178_TEST_COVERAGE_V2_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 179,
            name: "add_code_quality_rules".into(),
            up_sql: M_179_CODE_QUALITY_RULES_UP.into(),
            down_sql: M_179_CODE_QUALITY_RULES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 180,
            name: "add_performance_test_configs_results".into(),
            up_sql: M_180_PERF_TEST_CONFIGS_RESULTS_UP.into(),
            down_sql: M_180_PERF_TEST_CONFIGS_RESULTS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 187,
            name: "add_workflow_templates".into(),
            up_sql: M_187_WORKFLOW_TEMPLATES_UP.into(),
            down_sql: M_187_WORKFLOW_TEMPLATES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 188,
            name: "add_automation_rules_v3".into(),
            up_sql: M_188_AUTOMATION_RULES_V3_UP.into(),
            down_sql: M_188_AUTOMATION_RULES_V3_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 189,
            name: "add_scheduled_task_templates".into(),
            up_sql: M_189_SCHEDULED_TASK_TEMPLATES_UP.into(),
            down_sql: M_189_SCHEDULED_TASK_TEMPLATES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 190,
            name: "add_log_aggregation_v2".into(),
            up_sql: M_190_LOG_AGGREGATION_V2_UP.into(),
            down_sql: M_190_LOG_AGGREGATION_V2_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 191,
            name: "add_distributed_tracing_v3".into(),
            up_sql: M_191_DISTRIBUTED_TRACING_V3_UP.into(),
            down_sql: M_191_DISTRIBUTED_TRACING_V3_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 192,
            name: "add_dashboard_reporting_v2".into(),
            up_sql: M_192_DASHBOARD_REPORTING_V2_UP.into(),
            down_sql: M_192_DASHBOARD_REPORTING_V2_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 193,
            name: "add_pipeline_action_categories".into(),
            up_sql: M_193_PIPELINE_ACTION_CATEGORIES_UP.into(),
            down_sql: M_193_PIPELINE_ACTION_CATEGORIES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 194,
            name: "add_environment_webhooks_notifications".into(),
            up_sql: M_194_ENVIRONMENT_WEBHOOKS_NOTIFICATIONS_UP.into(),
            down_sql: M_194_ENVIRONMENT_WEBHOOKS_NOTIFICATIONS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 195,
            name: "add_cache_warming_rules".into(),
            up_sql: M_195_CACHE_WARMING_RULES_UP.into(),
            down_sql: M_195_CACHE_WARMING_RULES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 196,
            name: "add_test_suite_config_notifications".into(),
            up_sql: M_196_TEST_SUITE_CONFIG_NOTIFICATIONS_UP.into(),
            down_sql: M_196_TEST_SUITE_CONFIG_NOTIFICATIONS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 197,
            name: "add_code_quality_rules_v2".into(),
            up_sql: M_197_CODE_QUALITY_RULES_V2_UP.into(),
            down_sql: M_197_CODE_QUALITY_RULES_V2_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 198,
            name: "add_performance_baselines_regressions".into(),
            up_sql: M_198_PERF_BASELINES_REGRESSIONS_UP.into(),
            down_sql: M_198_PERF_BASELINES_REGRESSIONS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 199,
            name: "add_api_docs_v4".into(),
            up_sql: M_199_API_DOCS_V4_UP.into(),
            down_sql: M_199_API_DOCS_V4_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 200,
            name: "add_rate_limit_tiers_v2".into(),
            up_sql: M_200_RATE_LIMIT_TIERS_V2_UP.into(),
            down_sql: M_200_RATE_LIMIT_TIERS_V2_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 201,
            name: "add_api_analytics_v5".into(),
            up_sql: M_201_API_ANALYTICS_V5_UP.into(),
            down_sql: M_201_API_ANALYTICS_V5_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 202,
            name: "add_database_replication_v2".into(),
            up_sql: M_202_DATABASE_REPLICATION_V2_UP.into(),
            down_sql: M_202_DATABASE_REPLICATION_V2_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 203,
            name: "add_encryption_v3".into(),
            up_sql: M_203_ENCRYPTION_V3_UP.into(),
            down_sql: M_203_ENCRYPTION_V3_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 204,
            name: "add_data_residency_v2".into(),
            up_sql: M_204_DATA_RESIDENCY_V2_UP.into(),
            down_sql: M_204_DATA_RESIDENCY_V2_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 205,
            name: "add_security_scan_rules_v3_fixes".into(),
            up_sql: M_205_SECURITY_SCAN_V3_UP.into(),
            down_sql: M_205_SECURITY_SCAN_V3_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 206,
            name: "add_compliance_frameworks_v3_evidence_v2".into(),
            up_sql: M_206_COMPLIANCE_V3_UP.into(),
            down_sql: M_206_COMPLIANCE_V3_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 207,
            name: "add_audit_trail_v4".into(),
            up_sql: M_207_AUDIT_TRAIL_V4_UP.into(),
            down_sql: M_207_AUDIT_TRAIL_V4_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 208,
            name: "add_workflow_execution_v4".into(),
            up_sql: M_208_WORKFLOW_EXECUTION_V4_UP.into(),
            down_sql: M_208_WORKFLOW_EXECUTION_V4_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 209,
            name: "add_automation_rules_v4".into(),
            up_sql: M_209_AUTOMATION_RULES_V4_UP.into(),
            down_sql: M_209_AUTOMATION_RULES_V4_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 210,
            name: "add_scheduled_task_execution_v4".into(),
            up_sql: M_210_SCHEDULED_TASK_EXECUTION_V4_UP.into(),
            down_sql: M_210_SCHEDULED_TASK_EXECUTION_V4_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 211,
            name: "add_log_aggregation_v3".into(),
            up_sql: M_211_LOG_AGGREGATION_V3_UP.into(),
            down_sql: M_211_LOG_AGGREGATION_V3_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 212,
            name: "add_distributed_tracing_v4".into(),
            up_sql: M_212_DISTRIBUTED_TRACING_V4_UP.into(),
            down_sql: M_212_DISTRIBUTED_TRACING_V4_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 213,
            name: "add_dashboard_reporting_v3".into(),
            up_sql: M_213_DASHBOARD_REPORTING_V3_UP.into(),
            down_sql: M_213_DASHBOARD_REPORTING_V3_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 217,
            name: "add_test_suite_tags_dependencies".into(),
            up_sql: M_217_TEST_SUITE_TAGS_DEPS_UP.into(),
            down_sql: M_217_TEST_SUITE_TAGS_DEPS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 218,
            name: "add_code_quality_rules_v3_enforcement".into(),
            up_sql: M_218_CODE_QUALITY_RULES_V3_UP.into(),
            down_sql: M_218_CODE_QUALITY_RULES_V3_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 219,
            name: "add_performance_test_alerts".into(),
            up_sql: M_219_PERF_TEST_ALERTS_UP.into(),
            down_sql: M_219_PERF_TEST_ALERTS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 220,
            name: "add_api_docs_v5".into(),
            up_sql: M_220_API_DOCS_V5_UP.into(),
            down_sql: M_220_API_DOCS_V5_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 221,
            name: "add_rate_limit_tiers_v3".into(),
            up_sql: M_221_RATE_LIMIT_TIERS_V3_UP.into(),
            down_sql: M_221_RATE_LIMIT_TIERS_V3_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 222,
            name: "add_api_analytics_v6".into(),
            up_sql: M_222_API_ANALYTICS_V6_UP.into(),
            down_sql: M_222_API_ANALYTICS_V6_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 223,
            name: "add_database_replication_v3".into(),
            up_sql: M_223_DATABASE_REPLICATION_V3_UP.into(),
            down_sql: M_223_DATABASE_REPLICATION_V3_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 224,
            name: "add_encryption_v4".into(),
            up_sql: M_224_ENCRYPTION_V4_UP.into(),
            down_sql: M_224_ENCRYPTION_V4_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 225,
            name: "add_data_residency_v3".into(),
            up_sql: M_225_DATA_RESIDENCY_V3_UP.into(),
            down_sql: M_225_DATA_RESIDENCY_V3_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 232,
            name: "add_log_aggregation_v4".into(),
            up_sql: M_232_LOG_AGGREGATION_V4_UP.into(),
            down_sql: M_232_LOG_AGGREGATION_V4_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 233,
            name: "add_distributed_tracing_v4".into(),
            up_sql: M_233_DISTRIBUTED_TRACING_V4_UP.into(),
            down_sql: M_233_DISTRIBUTED_TRACING_V4_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 234,
            name: "add_dashboard_reporting_v4".into(),
            up_sql: M_234_DASHBOARD_REPORTING_V4_UP.into(),
            down_sql: M_234_DASHBOARD_REPORTING_V4_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 238,
            name: "add_test_suite_metrics_baselines".into(),
            up_sql: M_238_TEST_SUITE_METRICS_BASELINES_UP.into(),
            down_sql: M_238_TEST_SUITE_METRICS_BASELINES_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 239,
            name: "add_code_quality_metrics_thresholds".into(),
            up_sql: M_239_CODE_QUALITY_METRICS_THRESHOLDS_UP.into(),
            down_sql: M_239_CODE_QUALITY_METRICS_THRESHOLDS_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 240,
            name: "add_performance_test_alerts_v2".into(),
            up_sql: M_240_PERF_TEST_ALERTS_V2_UP.into(),
            down_sql: M_240_PERF_TEST_ALERTS_V2_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 244,
            name: "add_database_replication_v4".into(),
            up_sql: M_244_DATABASE_REPLICATION_V4_UP.into(),
            down_sql: M_244_DATABASE_REPLICATION_V4_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 245,
            name: "add_encryption_v5".into(),
            up_sql: M_245_ENCRYPTION_V5_UP.into(),
            down_sql: M_245_ENCRYPTION_V5_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 246,
            name: "add_data_residency_v4".into(),
            up_sql: M_246_DATA_RESIDENCY_V4_UP.into(),
            down_sql: M_246_DATA_RESIDENCY_V4_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 247,
            name: "add_security_scan_rules_v5".into(),
            up_sql: M_247_SECURITY_SCAN_RULES_V5_UP.into(),
            down_sql: M_247_SECURITY_SCAN_RULES_V5_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 248,
            name: "add_compliance_frameworks_v5".into(),
            up_sql: M_248_COMPLIANCE_FRAMEWORKS_V5_UP.into(),
            down_sql: M_248_COMPLIANCE_FRAMEWORKS_V5_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 249,
            name: "add_audit_trail_v6".into(),
            up_sql: M_249_AUDIT_TRAIL_V6_UP.into(),
            down_sql: M_249_AUDIT_TRAIL_V6_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 253,
            name: "add_log_aggregation_v5".into(),
            up_sql: M_253_LOG_AGGREGATION_V5_UP.into(),
            down_sql: M_253_LOG_AGGREGATION_V5_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 254,
            name: "add_distributed_tracing_v6".into(),
            up_sql: M_254_DISTRIBUTED_TRACING_V6_UP.into(),
            down_sql: M_254_DISTRIBUTED_TRACING_V6_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 255,
            name: "add_dashboard_reporting_v5".into(),
            up_sql: M_255_DASHBOARD_REPORTING_V5_UP.into(),
            down_sql: M_255_DASHBOARD_REPORTING_V5_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 256,
            name: "add_pipeline_action_reviews_v3".into(),
            up_sql: M_256_PIPELINE_ACTION_REVIEWS_V3_UP.into(),
            down_sql: M_256_PIPELINE_ACTION_REVIEWS_V3_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 257,
            name: "add_environment_deployment_history_v3".into(),
            up_sql: M_257_ENVIRONMENT_DEPLOYMENT_V3_UP.into(),
            down_sql: M_257_ENVIRONMENT_DEPLOYMENT_V3_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 258,
            name: "add_cache_hit_analysis_v2".into(),
            up_sql: M_258_CACHE_HIT_ANALYSIS_V2_UP.into(),
            down_sql: M_258_CACHE_HIT_ANALYSIS_V2_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 259,
            name: "add_test_suite_metrics_baselines_v2".into(),
            up_sql: M_259_TEST_SUITE_METRICS_BASELINES_V2_UP.into(),
            down_sql: M_259_TEST_SUITE_METRICS_BASELINES_V2_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 260,
            name: "add_code_quality_metrics_thresholds_v3".into(),
            up_sql: M_260_CODE_QUALITY_METRICS_THRESHOLDS_V3_UP.into(),
            down_sql: M_260_CODE_QUALITY_METRICS_THRESHOLDS_V3_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 261,
            name: "add_performance_test_alerts_v3".into(),
            up_sql: M_261_PERF_TEST_ALERTS_V3_UP.into(),
            down_sql: M_261_PERF_TEST_ALERTS_V3_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 262,
            name: "add_api_docs_v7".into(),
            up_sql: M_262_API_DOCS_V7_UP.into(),
            down_sql: M_262_API_DOCS_V7_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 263,
            name: "add_rate_limit_tiers_v5".into(),
            up_sql: M_263_RATE_LIMIT_TIERS_V5_UP.into(),
            down_sql: M_263_RATE_LIMIT_TIERS_V5_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 264,
            name: "add_api_analytics_v8".into(),
            up_sql: M_264_API_ANALYTICS_V8_UP.into(),
            down_sql: M_264_API_ANALYTICS_V8_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 265,
            name: "add_database_replication_v5".into(),
            up_sql: M_265_DATABASE_REPLICATION_V5_UP.into(),
            down_sql: M_265_DATABASE_REPLICATION_V5_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 266,
            name: "add_encryption_v6".into(),
            up_sql: M_266_ENCRYPTION_V6_UP.into(),
            down_sql: M_266_ENCRYPTION_V6_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 267,
            name: "add_data_residency_v5".into(),
            up_sql: M_267_DATA_RESIDENCY_V5_UP.into(),
            down_sql: M_267_DATA_RESIDENCY_V5_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 271,
            name: "add_workflow_templates_v4".into(),
            up_sql: M_271_WORKFLOW_TEMPLATES_V4_UP.into(),
            down_sql: M_271_WORKFLOW_TEMPLATES_V4_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 272,
            name: "add_automation_rules_v7".into(),
            up_sql: M_272_AUTOMATION_RULES_V7_UP.into(),
            down_sql: M_272_AUTOMATION_RULES_V7_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 273,
            name: "add_scheduled_task_templates_v4".into(),
            up_sql: M_273_SCHEDULED_TASK_TEMPLATES_V4_UP.into(),
            down_sql: M_273_SCHEDULED_TASK_TEMPLATES_V4_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 274,
            name: "add_log_aggregation_v6".into(),
            up_sql: M_274_LOG_AGGREGATION_V6_UP.into(),
            down_sql: M_274_LOG_AGGREGATION_V6_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 275,
            name: "add_distributed_tracing_v7".into(),
            up_sql: M_275_DISTRIBUTED_TRACING_V7_UP.into(),
            down_sql: M_275_DISTRIBUTED_TRACING_V7_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 276,
            name: "add_dashboard_reporting_v6".into(),
            up_sql: M_276_DASHBOARD_REPORTING_V6_UP.into(),
            down_sql: M_276_DASHBOARD_REPORTING_V6_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 277,
            name: "add_pipeline_action_reviews_v4".into(),
            up_sql: M_277_PIPELINE_ACTION_REVIEWS_V4_UP.into(),
            down_sql: M_277_PIPELINE_ACTION_REVIEWS_V4_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 278,
            name: "add_environment_deployment_history_v4".into(),
            up_sql: M_278_ENVIRONMENT_DEPLOYMENT_V4_UP.into(),
            down_sql: M_278_ENVIRONMENT_DEPLOYMENT_V4_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 279,
            name: "add_cache_hit_analysis_v3".into(),
            up_sql: M_279_CACHE_HIT_ANALYSIS_V3_UP.into(),
            down_sql: M_279_CACHE_HIT_ANALYSIS_V3_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 280,
            name: "add_test_suite_metrics_baselines_v3".into(),
            up_sql: M_280_TEST_SUITE_METRICS_BASELINES_V3_UP.into(),
            down_sql: M_280_TEST_SUITE_METRICS_BASELINES_V3_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 281,
            name: "add_code_quality_metrics_thresholds_v4".into(),
            up_sql: M_281_CODE_QUALITY_METRICS_THRESHOLDS_V4_UP.into(),
            down_sql: M_281_CODE_QUALITY_METRICS_THRESHOLDS_V4_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 282,
            name: "add_performance_test_alerts_v4".into(),
            up_sql: M_282_PERF_TEST_ALERTS_V4_UP.into(),
            down_sql: M_282_PERF_TEST_ALERTS_V4_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 283,
            name: "add_api_docs_v8".into(),
            up_sql: M_283_API_DOCS_V8_UP.into(),
            down_sql: M_283_API_DOCS_V8_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 284,
            name: "add_rate_limit_tiers_v6".into(),
            up_sql: M_284_RATE_LIMIT_TIERS_V6_UP.into(),
            down_sql: M_284_RATE_LIMIT_TIERS_V6_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 285,
            name: "add_api_analytics_v9".into(),
            up_sql: M_285_API_ANALYTICS_V9_UP.into(),
            down_sql: M_285_API_ANALYTICS_V9_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 295,
            name: "add_log_aggregation_v7".into(),
            up_sql: M_295_LOG_AGGREGATION_V7_UP.into(),
            down_sql: M_295_LOG_AGGREGATION_V7_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 296,
            name: "add_distributed_tracing_v8".into(),
            up_sql: M_296_DISTRIBUTED_TRACING_V8_UP.into(),
            down_sql: M_296_DISTRIBUTED_TRACING_V8_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 297,
            name: "add_dashboard_reporting_v7".into(),
            up_sql: M_297_DASHBOARD_REPORTING_V7_UP.into(),
            down_sql: M_297_DASHBOARD_REPORTING_V7_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 304,
            name: "add_api_docs_v9".into(),
            up_sql: M_304_API_DOCS_V9_UP.into(),
            down_sql: M_304_API_DOCS_V9_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 305,
            name: "add_rate_limit_tiers_v7".into(),
            up_sql: M_305_RATE_LIMIT_TIERS_V7_UP.into(),
            down_sql: M_305_RATE_LIMIT_TIERS_V7_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 306,
            name: "add_api_analytics_v10".into(),
            up_sql: M_306_API_ANALYTICS_V10_UP.into(),
            down_sql: M_306_API_ANALYTICS_V10_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 307,
            name: "add_database_replication_v7".into(),
            up_sql: M_307_DATABASE_REPLICATION_V7_UP.into(),
            down_sql: M_307_DATABASE_REPLICATION_V7_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 308,
            name: "add_encryption_v8".into(),
            up_sql: M_308_ENCRYPTION_V8_UP.into(),
            down_sql: M_308_ENCRYPTION_V8_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 309,
            name: "add_data_residency_v7".into(),
            up_sql: M_309_DATA_RESIDENCY_V7_UP.into(),
            down_sql: M_309_DATA_RESIDENCY_V7_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 310,
            name: "add_security_scan_v9".into(),
            up_sql: M_310_SECURITY_SCAN_V9_UP.into(),
            down_sql: M_310_SECURITY_SCAN_V9_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 311,
            name: "add_compliance_frameworks_v9".into(),
            up_sql: M_311_COMPLIANCE_FRAMEWORKS_V9_UP.into(),
            down_sql: M_311_COMPLIANCE_FRAMEWORKS_V9_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 312,
            name: "add_audit_trail_v9".into(),
            up_sql: M_312_AUDIT_TRAIL_V9_UP.into(),
            down_sql: M_312_AUDIT_TRAIL_V9_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 325,
            name: "add_api_docs_v10".into(),
            up_sql: M_325_API_DOCS_V10_UP.into(),
            down_sql: M_325_API_DOCS_V10_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 326,
            name: "add_rate_limit_tiers_v8".into(),
            up_sql: M_326_RATE_LIMIT_TIERS_V8_UP.into(),
            down_sql: M_326_RATE_LIMIT_TIERS_V8_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 327,
            name: "add_api_analytics_v11".into(),
            up_sql: M_327_API_ANALYTICS_V11_UP.into(),
            down_sql: M_327_API_ANALYTICS_V11_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 328,
            name: "add_database_replication_v8".into(),
            up_sql: M_328_DATABASE_REPLICATION_V8_UP.into(),
            down_sql: M_328_DATABASE_REPLICATION_V8_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 329,
            name: "add_encryption_v9".into(),
            up_sql: M_329_ENCRYPTION_V9_UP.into(),
            down_sql: M_329_ENCRYPTION_V9_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 330,
            name: "add_data_residency_v8".into(),
            up_sql: M_330_DATA_RESIDENCY_V8_UP.into(),
            down_sql: M_330_DATA_RESIDENCY_V8_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 331,
            name: "add_security_scan_v10".into(),
            up_sql: M_331_SECURITY_SCAN_V10_UP.into(),
            down_sql: M_331_SECURITY_SCAN_V10_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 332,
            name: "add_compliance_frameworks_v10".into(),
            up_sql: M_332_COMPLIANCE_FRAMEWORKS_V10_UP.into(),
            down_sql: M_332_COMPLIANCE_FRAMEWORKS_V10_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 333,
            name: "add_audit_trail_v10".into(),
            up_sql: M_333_AUDIT_TRAIL_V10_UP.into(),
            down_sql: M_333_AUDIT_TRAIL_V10_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 334,
            name: "add_workflow_templates_v7".into(),
            up_sql: M_334_WORKFLOW_TEMPLATES_V7_UP.into(),
            down_sql: M_334_WORKFLOW_TEMPLATES_V7_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 335,
            name: "add_automation_rules_v10".into(),
            up_sql: M_335_AUTOMATION_RULES_V10_UP.into(),
            down_sql: M_335_AUTOMATION_RULES_V10_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 336,
            name: "add_scheduled_task_templates_v7".into(),
            up_sql: M_336_SCHEDULED_TASK_TEMPLATES_V7_UP.into(),
            down_sql: M_336_SCHEDULED_TASK_TEMPLATES_V7_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 337,
            name: "add_log_aggregation_v9".into(),
            up_sql: M_337_LOG_AGGREGATION_V9_UP.into(),
            down_sql: M_337_LOG_AGGREGATION_V9_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 338,
            name: "add_distributed_tracing_v10".into(),
            up_sql: M_338_DISTRIBUTED_TRACING_V10_UP.into(),
            down_sql: M_338_DISTRIBUTED_TRACING_V10_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 339,
            name: "add_dashboard_reporting_v9".into(),
            up_sql: M_339_DASHBOARD_REPORTING_V9_UP.into(),
            down_sql: M_339_DASHBOARD_REPORTING_V9_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 340,
            name: "add_pipeline_action_reviews_v7".into(),
            up_sql: M_340_PIPELINE_ACTION_REVIEWS_V7_UP.into(),
            down_sql: M_340_PIPELINE_ACTION_REVIEWS_V7_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 341,
            name: "add_environment_deployment_history_v7".into(),
            up_sql: M_341_ENVIRONMENT_DEPLOYMENT_V7_UP.into(),
            down_sql: M_341_ENVIRONMENT_DEPLOYMENT_V7_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 342,
            name: "add_cache_hit_analysis_v6".into(),
            up_sql: M_342_CACHE_HIT_ANALYSIS_V6_UP.into(),
            down_sql: M_342_CACHE_HIT_ANALYSIS_V6_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 343,
            name: "add_test_suite_management_v9".into(),
            up_sql: M_343_TEST_SUITE_MANAGEMENT_V9_UP.into(),
            down_sql: M_343_TEST_SUITE_MANAGEMENT_V9_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 344,
            name: "add_code_quality_rules_v9".into(),
            up_sql: M_344_CODE_QUALITY_RULES_V9_UP.into(),
            down_sql: M_344_CODE_QUALITY_RULES_V9_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 345,
            name: "add_performance_testing_v10".into(),
            up_sql: M_345_PERFORMANCE_TESTING_V10_UP.into(),
            down_sql: M_345_PERFORMANCE_TESTING_V10_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 346,
            name: "add_api_docs_v11".into(),
            up_sql: M_346_API_DOCS_V11_UP.into(),
            down_sql: M_346_API_DOCS_V11_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 347,
            name: "add_rate_limit_tiers_v9".into(),
            up_sql: M_347_RATE_LIMIT_TIERS_V9_UP.into(),
            down_sql: M_347_RATE_LIMIT_TIERS_V9_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 348,
            name: "add_api_analytics_v12".into(),
            up_sql: M_348_API_ANALYTICS_V12_UP.into(),
            down_sql: M_348_API_ANALYTICS_V12_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 352,
            name: "add_security_scan_v11".into(),
            up_sql: M_352_SECURITY_SCAN_V11_UP.into(),
            down_sql: M_352_SECURITY_SCAN_V11_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 353,
            name: "add_compliance_frameworks_v11".into(),
            up_sql: M_353_COMPLIANCE_FRAMEWORKS_V11_UP.into(),
            down_sql: M_353_COMPLIANCE_FRAMEWORKS_V11_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 354,
            name: "add_audit_trail_v11".into(),
            up_sql: M_354_AUDIT_TRAIL_V11_UP.into(),
            down_sql: M_354_AUDIT_TRAIL_V11_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 355,
            name: "add_workflow_templates_v8".into(),
            up_sql: M_355_WORKFLOW_TEMPLATES_V8_UP.into(),
            down_sql: M_355_WORKFLOW_TEMPLATES_V8_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 356,
            name: "add_automation_rules_v11".into(),
            up_sql: M_356_AUTOMATION_RULES_V11_UP.into(),
            down_sql: M_356_AUTOMATION_RULES_V11_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 357,
            name: "add_scheduled_task_templates_v8".into(),
            up_sql: M_357_SCHEDULED_TASK_TEMPLATES_V8_UP.into(),
            down_sql: M_357_SCHEDULED_TASK_TEMPLATES_V8_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 376,
            name: "add_workflow_templates_v9".into(),
            up_sql: M_376_WORKFLOW_TEMPLATES_V9_UP.into(),
            down_sql: M_376_WORKFLOW_TEMPLATES_V9_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 377,
            name: "add_automation_rules_v12".into(),
            up_sql: M_377_AUTOMATION_RULES_V12_UP.into(),
            down_sql: M_377_AUTOMATION_RULES_V12_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 378,
            name: "add_scheduled_task_templates_v9".into(),
            up_sql: M_378_SCHEDULED_TASK_TEMPLATES_V9_UP.into(),
            down_sql: M_378_SCHEDULED_TASK_TEMPLATES_V9_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 388,
            name: "add_api_docs_v13".into(),
            up_sql: M_388_API_DOCS_V13_UP.into(),
            down_sql: M_388_API_DOCS_V13_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 389,
            name: "add_rate_limit_tiers_v11".into(),
            up_sql: M_389_RATE_LIMIT_TIERS_V11_UP.into(),
            down_sql: M_389_RATE_LIMIT_TIERS_V11_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 390,
            name: "add_api_analytics_v14".into(),
            up_sql: M_390_API_ANALYTICS_V14_UP.into(),
            down_sql: M_390_API_ANALYTICS_V14_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 361,
            name: "add_pipeline_action_reviews_v8".into(),
            up_sql: M_361_PIPELINE_ACTION_REVIEWS_V8_UP.into(),
            down_sql: M_361_PIPELINE_ACTION_REVIEWS_V8_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 362,
            name: "add_environment_deployment_history_v8".into(),
            up_sql: M_362_ENVIRONMENT_DEPLOYMENT_V8_UP.into(),
            down_sql: M_362_ENVIRONMENT_DEPLOYMENT_V8_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 363,
            name: "add_cache_hit_analysis_v7".into(),
            up_sql: M_363_CACHE_HIT_ANALYSIS_V7_UP.into(),
            down_sql: M_363_CACHE_HIT_ANALYSIS_V7_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 364,
            name: "add_test_suite_management_v10".into(),
            up_sql: M_364_TEST_SUITE_MANAGEMENT_V10_UP.into(),
            down_sql: M_364_TEST_SUITE_MANAGEMENT_V10_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 365,
            name: "add_code_quality_rules_v10".into(),
            up_sql: M_365_CODE_QUALITY_RULES_V10_UP.into(),
            down_sql: M_365_CODE_QUALITY_RULES_V10_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 366,
            name: "add_performance_testing_v11".into(),
            up_sql: M_366_PERFORMANCE_TESTING_V11_UP.into(),
            down_sql: M_366_PERFORMANCE_TESTING_V11_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 370,
            name: "add_database_replication_v10".into(),
            up_sql: M_370_DATABASE_REPLICATION_V10_UP.into(),
            down_sql: M_370_DATABASE_REPLICATION_V10_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 371,
            name: "add_encryption_v11".into(),
            up_sql: M_371_ENCRYPTION_V11_UP.into(),
            down_sql: M_371_ENCRYPTION_V11_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 372,
            name: "add_data_residency_v10".into(),
            up_sql: M_372_DATA_RESIDENCY_V10_UP.into(),
            down_sql: M_372_DATA_RESIDENCY_V10_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 400,
            name: "add_log_aggregation_v12".into(),
            up_sql: M_400_LOG_AGGREGATION_V12_UP.into(),
            down_sql: M_400_LOG_AGGREGATION_V12_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 401,
            name: "add_distributed_tracing_v13".into(),
            up_sql: M_401_DISTRIBUTED_TRACING_V13_UP.into(),
            down_sql: M_401_DISTRIBUTED_TRACING_V13_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 402,
            name: "add_dashboard_reporting_v12".into(),
            up_sql: M_402_DASHBOARD_REPORTING_V12_UP.into(),
            down_sql: M_402_DASHBOARD_REPORTING_V12_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 406,
            name: "add_test_suite_management_v12".into(),
            up_sql: M_406_TEST_SUITE_MANAGEMENT_V12_UP.into(),
            down_sql: M_406_TEST_SUITE_MANAGEMENT_V12_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 407,
            name: "add_code_quality_rules_v12".into(),
            up_sql: M_407_CODE_QUALITY_RULES_V12_UP.into(),
            down_sql: M_407_CODE_QUALITY_RULES_V12_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 408,
            name: "add_performance_testing_v13".into(),
            up_sql: M_408_PERFORMANCE_TESTING_V13_UP.into(),
            down_sql: M_408_PERFORMANCE_TESTING_V13_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 409,
            name: "add_api_docs_v14".into(),
            up_sql: M_409_API_DOCS_V14_UP.into(),
            down_sql: M_409_API_DOCS_V14_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 410,
            name: "add_rate_limit_tiers_v12".into(),
            up_sql: M_410_RATE_LIMIT_TIERS_V12_UP.into(),
            down_sql: M_410_RATE_LIMIT_TIERS_V12_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 411,
            name: "add_api_analytics_v15".into(),
            up_sql: M_411_API_ANALYTICS_V15_UP.into(),
            down_sql: M_411_API_ANALYTICS_V15_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 412,
            name: "add_database_replication_v12".into(),
            up_sql: M_412_DATABASE_REPLICATION_V12_UP.into(),
            down_sql: M_412_DATABASE_REPLICATION_V12_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 413,
            name: "add_encryption_v13".into(),
            up_sql: M_413_ENCRYPTION_V13_UP.into(),
            down_sql: M_413_ENCRYPTION_V13_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 414,
            name: "add_data_residency_v12".into(),
            up_sql: M_414_DATA_RESIDENCY_V12_UP.into(),
            down_sql: M_414_DATA_RESIDENCY_V12_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 418,
            name: "add_workflow_templates_v11".into(),
            up_sql: M_418_WORKFLOW_TEMPLATES_V11_UP.into(),
            down_sql: M_418_WORKFLOW_TEMPLATES_V11_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 419,
            name: "add_automation_rules_v14".into(),
            up_sql: M_419_AUTOMATION_RULES_V14_UP.into(),
            down_sql: M_419_AUTOMATION_RULES_V14_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 420,
            name: "add_scheduled_task_templates_v11".into(),
            up_sql: M_420_SCHEDULED_TASK_TEMPLATES_V11_UP.into(),
            down_sql: M_420_SCHEDULED_TASK_TEMPLATES_V11_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 421,
            name: "add_log_aggregation_v13".into(),
            up_sql: M_421_LOG_AGGREGATION_V13_UP.into(),
            down_sql: M_421_LOG_AGGREGATION_V13_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 422,
            name: "add_distributed_tracing_v14".into(),
            up_sql: M_422_DISTRIBUTED_TRACING_V14_UP.into(),
            down_sql: M_422_DISTRIBUTED_TRACING_V14_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 423,
            name: "add_dashboard_reporting_v13".into(),
            up_sql: M_423_DASHBOARD_REPORTING_V13_UP.into(),
            down_sql: M_423_DASHBOARD_REPORTING_V13_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 427,
            name: "add_test_suite_management_v13".into(),
            up_sql: M_427_TEST_SUITE_MANAGEMENT_V13_UP.into(),
            down_sql: M_427_TEST_SUITE_MANAGEMENT_V13_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 428,
            name: "add_code_quality_rules_v13".into(),
            up_sql: M_428_CODE_QUALITY_RULES_V13_UP.into(),
            down_sql: M_428_CODE_QUALITY_RULES_V13_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 429,
            name: "add_performance_testing_v14".into(),
            up_sql: M_429_PERFORMANCE_TESTING_V14_UP.into(),
            down_sql: M_429_PERFORMANCE_TESTING_V14_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 430,
            name: "add_api_docs_v15".into(),
            up_sql: M_430_API_DOCS_V15_UP.into(),
            down_sql: M_430_API_DOCS_V15_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 431,
            name: "add_rate_limit_tiers_v13".into(),
            up_sql: M_431_RATE_LIMIT_TIERS_V13_UP.into(),
            down_sql: M_431_RATE_LIMIT_TIERS_V13_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 432,
            name: "add_api_analytics_v16".into(),
            up_sql: M_432_API_ANALYTICS_V16_UP.into(),
            down_sql: M_432_API_ANALYTICS_V16_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 433,
            name: "add_database_replication_v13".into(),
            up_sql: M_433_DATABASE_REPLICATION_V13_UP.into(),
            down_sql: M_433_DATABASE_REPLICATION_V13_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 434,
            name: "add_encryption_v14".into(),
            up_sql: M_434_ENCRYPTION_V14_UP.into(),
            down_sql: M_434_ENCRYPTION_V14_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 435,
            name: "add_data_residency_v13".into(),
            up_sql: M_435_DATA_RESIDENCY_V13_UP.into(),
            down_sql: M_435_DATA_RESIDENCY_V13_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 448,
            name: "add_test_suite_metrics_baselines_v11".into(),
            up_sql: M_448_TEST_SUITE_METRICS_BASELINES_V11_UP.into(),
            down_sql: M_448_TEST_SUITE_METRICS_BASELINES_V11_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 449,
            name: "add_code_quality_metrics_v12_thresholds_v11".into(),
            up_sql: M_449_CODE_QUALITY_METRICS_V12_THRESHOLDS_V11_UP.into(),
            down_sql: M_449_CODE_QUALITY_METRICS_V12_THRESHOLDS_V11_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 450,
            name: "add_performance_test_alerts_v12".into(),
            up_sql: M_450_PERFORMANCE_TEST_ALERTS_V12_UP.into(),
            down_sql: M_450_PERFORMANCE_TEST_ALERTS_V12_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 451,
            name: "add_api_docs_v16".into(),
            up_sql: M_451_API_DOCS_V16_UP.into(),
            down_sql: M_451_API_DOCS_V16_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 452,
            name: "add_rate_limit_tiers_v14".into(),
            up_sql: M_452_RATE_LIMIT_TIERS_V14_UP.into(),
            down_sql: M_452_RATE_LIMIT_TIERS_V14_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 453,
            name: "add_api_analytics_v17".into(),
            up_sql: M_453_API_ANALYTICS_V17_UP.into(),
            down_sql: M_453_API_ANALYTICS_V17_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 460,
            name: "add_workflow_templates_v13".into(),
            up_sql: M_460_WORKFLOW_TEMPLATES_V13_UP.into(),
            down_sql: M_460_WORKFLOW_TEMPLATES_V13_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 461,
            name: "add_automation_rules_v16".into(),
            up_sql: M_461_AUTOMATION_RULES_V16_UP.into(),
            down_sql: M_461_AUTOMATION_RULES_V16_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 462,
            name: "add_scheduled_task_templates_v13".into(),
            up_sql: M_462_SCHEDULED_TASK_TEMPLATES_V13_UP.into(),
            down_sql: M_462_SCHEDULED_TASK_TEMPLATES_V13_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 481,
            name: "add_workflow_templates_v14".into(),
            up_sql: M_481_WORKFLOW_TEMPLATES_V14_UP.into(),
            down_sql: M_481_WORKFLOW_TEMPLATES_V14_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 482,
            name: "add_automation_rules_v17".into(),
            up_sql: M_482_AUTOMATION_RULES_V17_UP.into(),
            down_sql: M_482_AUTOMATION_RULES_V17_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 483,
            name: "add_scheduled_task_templates_v14".into(),
            up_sql: M_483_SCHEDULED_TASK_TEMPLATES_V14_UP.into(),
            down_sql: M_483_SCHEDULED_TASK_TEMPLATES_V14_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 469,
            name: "add_test_suite_management_v15".into(),
            up_sql: M_469_TEST_SUITE_MANAGEMENT_V15_UP.into(),
            down_sql: M_469_TEST_SUITE_MANAGEMENT_V15_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 470,
            name: "add_code_quality_rules_v15".into(),
            up_sql: M_470_CODE_QUALITY_RULES_V15_UP.into(),
            down_sql: M_470_CODE_QUALITY_RULES_V15_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 471,
            name: "add_performance_testing_v16".into(),
            up_sql: M_471_PERFORMANCE_TESTING_V16_UP.into(),
            down_sql: M_471_PERFORMANCE_TESTING_V16_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 487,
            name: "add_pipeline_action_reviews_v14".into(),
            up_sql: M_487_PIPELINE_ACTION_REVIEWS_V14_UP.into(),
            down_sql: M_487_PIPELINE_ACTION_REVIEWS_V14_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 488,
            name: "add_environment_deployment_history_v14".into(),
            up_sql: M_488_ENVIRONMENT_DEPLOYMENT_V14_UP.into(),
            down_sql: M_488_ENVIRONMENT_DEPLOYMENT_V14_DOWN.into(),
        });
        self.add_migration(Migration {
            version: 489,
            name: "add_cache_hit_analysis_v13".into(),
            up_sql: M_489_CACHE_HIT_ANALYSIS_V13_UP.into(),
            down_sql: M_489_CACHE_HIT_ANALYSIS_V13_DOWN.into(),
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
        assert_eq!(mgr.all().len(), 289);
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
        assert_eq!(mgr.all()[115].version, 139);
        assert_eq!(mgr.all()[115].name, "add_network_policies");
        assert_eq!(mgr.all()[116].version, 140);
        assert_eq!(mgr.all()[116].name, "add_encryption_at_rest");
        assert_eq!(mgr.all()[117].version, 141);
        assert_eq!(mgr.all()[117].name, "add_access_control_lists");
        assert_eq!(mgr.all()[118].version, 142);
        assert_eq!(mgr.all()[118].name, "add_workflows");
        assert_eq!(mgr.all()[119].version, 143);
        assert_eq!(mgr.all()[119].name, "add_automation_rules");
        assert_eq!(mgr.all()[120].version, 144);
        assert_eq!(mgr.all()[120].name, "add_scheduled_tasks");
        assert_eq!(mgr.all()[121].version, 145);
        assert_eq!(mgr.all()[121].name, "add_log_aggregation");
        assert_eq!(mgr.all()[122].version, 146);
        assert_eq!(mgr.all()[122].name, "add_trace_sampling_rules");
        assert_eq!(mgr.all()[123].version, 147);
        assert_eq!(mgr.all()[123].name, "add_dashboard_reporting");
        assert_eq!(mgr.all()[124].version, 148);
        assert_eq!(mgr.all()[124].name, "add_pipeline_secrets_v2");
        assert_eq!(mgr.all()[125].version, 149);
        assert_eq!(mgr.all()[125].name, "add_pipeline_runners_v2");
        assert_eq!(mgr.all()[126].version, 150);
        assert_eq!(mgr.all()[126].name, "add_environment_variables");
        assert_eq!(mgr.all()[127].version, 154);
        assert_eq!(mgr.all()[127].name, "add_api_docs_v2");
        assert_eq!(mgr.all()[128].version, 155);
        assert_eq!(mgr.all()[128].name, "add_rate_limit_tiers");
        assert_eq!(mgr.all()[129].version, 156);
        assert_eq!(mgr.all()[129].name, "add_api_analytics_v3");
        assert_eq!(mgr.all()[130].version, 157);
        assert_eq!(mgr.all()[130].name, "add_database_replication");
        assert_eq!(mgr.all()[131].version, 158);
        assert_eq!(mgr.all()[131].name, "add_encryption_policies");
        assert_eq!(mgr.all()[132].version, 159);
        assert_eq!(mgr.all()[132].name, "add_data_residency");
        assert_eq!(mgr.all()[133].version, 160);
        assert_eq!(mgr.all()[133].name, "add_security_scan_rules");
        assert_eq!(mgr.all()[134].version, 161);
        assert_eq!(mgr.all()[134].name, "add_compliance_requirements");
        assert_eq!(mgr.all()[135].version, 162);
        assert_eq!(mgr.all()[135].name, "add_audit_trail_v2");
        assert_eq!(mgr.all()[136].version, 166);
        assert_eq!(mgr.all()[136].name, "add_firewall_rules");
        assert_eq!(mgr.all()[137].version, 167);
        assert_eq!(mgr.all()[137].name, "add_intrusion_detections");
        assert_eq!(mgr.all()[138].version, 168);
        assert_eq!(mgr.all()[138].name, "add_ddos_protection");
        assert_eq!(mgr.all()[139].version, 169);
        assert_eq!(mgr.all()[139].name, "add_object_storage");
        assert_eq!(mgr.all()[140].version, 170);
        assert_eq!(mgr.all()[140].name, "add_backup_encryption");
        assert_eq!(mgr.all()[141].version, 171);
        assert_eq!(mgr.all()[141].name, "add_data_retention");
        assert_eq!(mgr.all()[142].version, 172);
        assert_eq!(mgr.all()[142].name, "add_api_docs_v3");
        assert_eq!(mgr.all()[143].version, 173);
        assert_eq!(mgr.all()[143].name, "add_api_webhooks_v2");
        assert_eq!(mgr.all()[144].version, 174);
        assert_eq!(mgr.all()[144].name, "add_api_analytics_v4");
        assert_eq!(mgr.all()[148].version, 178);
        assert_eq!(mgr.all()[148].name, "add_test_coverage_v2");
        assert_eq!(mgr.all()[149].version, 179);
        assert_eq!(mgr.all()[149].name, "add_code_quality_rules");
        assert_eq!(mgr.all()[150].version, 180);
        assert_eq!(mgr.all()[150].name, "add_performance_test_configs_results");
    }

    #[test]
    fn test_add_migration_sequential() {
        let mut mgr = MigrationManager::new();
        mgr.add_migration(Migration {
            version: 403,
            name: "add_index".into(),
            up_sql: "CREATE INDEX test;".into(),
            down_sql: "DROP INDEX test;".into(),
        });
        assert_eq!(mgr.all().len(), 278);
        assert_eq!(mgr.all()[269].version, 403);
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
        assert_eq!(pending.len(), 268);
    }

    #[test]
    fn test_get_pending_all_applied() {
        let mgr = MigrationManager::new();
        let pending = mgr.get_pending(192);
        assert_eq!(pending.len(), 102);
    }

    #[test]
    fn test_get_pending_partial() {
        let mgr = MigrationManager::new();
        let pending = mgr.get_pending(1);
        assert_eq!(pending.len(), 261);
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

    #[test]
    fn test_test_suite_config_notifications_sql_not_empty() {
        assert_ne!(M_196_TEST_SUITE_CONFIG_NOTIFICATIONS_UP, "");
        assert!(M_196_TEST_SUITE_CONFIG_NOTIFICATIONS_UP.contains("CREATE TABLE IF NOT EXISTS test_suite_configurations"));
        assert!(M_196_TEST_SUITE_CONFIG_NOTIFICATIONS_UP.contains("suite_id"));
        assert!(M_196_TEST_SUITE_CONFIG_NOTIFICATIONS_UP.contains("config_key"));
        assert!(M_196_TEST_SUITE_CONFIG_NOTIFICATIONS_UP.contains("config_value"));
        assert!(M_196_TEST_SUITE_CONFIG_NOTIFICATIONS_UP.contains("CREATE TABLE IF NOT EXISTS test_suite_notifications"));
        assert!(M_196_TEST_SUITE_CONFIG_NOTIFICATIONS_UP.contains("notification_type"));
    }

    #[test]
    fn test_test_suite_config_notifications_down_sql_not_empty() {
        assert_ne!(M_196_TEST_SUITE_CONFIG_NOTIFICATIONS_DOWN, "");
        assert!(M_196_TEST_SUITE_CONFIG_NOTIFICATIONS_DOWN.contains("DROP TABLE IF EXISTS test_suite_notifications"));
        assert!(M_196_TEST_SUITE_CONFIG_NOTIFICATIONS_DOWN.contains("DROP TABLE IF EXISTS test_suite_configurations"));
    }

    #[test]
    fn test_code_quality_rules_v2_sql_not_empty() {
        assert_ne!(M_197_CODE_QUALITY_RULES_V2_UP, "");
        assert!(M_197_CODE_QUALITY_RULES_V2_UP.contains("CREATE TABLE IF NOT EXISTS code_quality_rules_v2"));
        assert!(M_197_CODE_QUALITY_RULES_V2_UP.contains("auto_fix"));
        assert!(M_197_CODE_QUALITY_RULES_V2_UP.contains("fix_config"));
        assert!(M_197_CODE_QUALITY_RULES_V2_UP.contains("CREATE TABLE IF NOT EXISTS code_quality_rule_versions"));
        assert!(M_197_CODE_QUALITY_RULES_V2_UP.contains("CREATE TABLE IF NOT EXISTS code_quality_rule_test_results"));
    }

    #[test]
    fn test_code_quality_rules_v2_down_sql_not_empty() {
        assert_ne!(M_197_CODE_QUALITY_RULES_V2_DOWN, "");
        assert!(M_197_CODE_QUALITY_RULES_V2_DOWN.contains("DROP TABLE IF EXISTS code_quality_rule_test_results"));
        assert!(M_197_CODE_QUALITY_RULES_V2_DOWN.contains("DROP TABLE IF EXISTS code_quality_rule_versions"));
        assert!(M_197_CODE_QUALITY_RULES_V2_DOWN.contains("DROP TABLE IF EXISTS code_quality_rules_v2"));
    }

    #[test]
    fn test_performance_baselines_regressions_sql_not_empty() {
        assert_ne!(M_198_PERF_BASELINES_REGRESSIONS_UP, "");
        assert!(M_198_PERF_BASELINES_REGRESSIONS_UP.contains("CREATE TABLE IF NOT EXISTS performance_baselines"));
        assert!(M_198_PERF_BASELINES_REGRESSIONS_UP.contains("baseline_value"));
        assert!(M_198_PERF_BASELINES_REGRESSIONS_UP.contains("threshold_percent"));
        assert!(M_198_PERF_BASELINES_REGRESSIONS_UP.contains("CREATE TABLE IF NOT EXISTS performance_regressions"));
        assert!(M_198_PERF_BASELINES_REGRESSIONS_UP.contains("regression_percent"));
        assert!(M_198_PERF_BASELINES_REGRESSIONS_UP.contains("CREATE TABLE IF NOT EXISTS performance_trend_data"));
    }

    #[test]
    fn test_performance_baselines_regressions_down_sql_not_empty() {
        assert_ne!(M_198_PERF_BASELINES_REGRESSIONS_DOWN, "");
        assert!(M_198_PERF_BASELINES_REGRESSIONS_DOWN.contains("DROP TABLE IF EXISTS performance_trend_data"));
        assert!(M_198_PERF_BASELINES_REGRESSIONS_DOWN.contains("DROP TABLE IF EXISTS performance_regressions"));
        assert!(M_198_PERF_BASELINES_REGRESSIONS_DOWN.contains("DROP TABLE IF EXISTS performance_baselines"));
    }

    #[test]
    fn test_api_docs_v7_sql_not_empty() {
        assert_ne!(M_262_API_DOCS_V7_UP, "");
        assert!(M_262_API_DOCS_V7_UP.contains("CREATE TABLE IF NOT EXISTS api_docs_v7"));
        assert!(M_262_API_DOCS_V7_UP.contains("endpoint"));
        assert!(M_262_API_DOCS_V7_UP.contains("method"));
        assert!(M_262_API_DOCS_V7_UP.contains("version"));
        assert!(M_262_API_DOCS_V7_UP.contains("security_schemes"));
        assert!(M_262_API_DOCS_V7_UP.contains("rate_limits"));
        assert!(M_262_API_DOCS_V7_UP.contains("changelog"));
        assert!(M_262_API_DOCS_V7_UP.contains("deprecated"));
    }

    #[test]
    fn test_api_docs_v7_down_sql_not_empty() {
        assert_ne!(M_262_API_DOCS_V7_DOWN, "");
        assert!(M_262_API_DOCS_V7_DOWN.contains("DROP TABLE IF EXISTS api_docs_v7"));
    }

    #[test]
    fn test_rate_limit_tiers_v5_sql_not_empty() {
        assert_ne!(M_263_RATE_LIMIT_TIERS_V5_UP, "");
        assert!(M_263_RATE_LIMIT_TIERS_V5_UP.contains("CREATE TABLE IF NOT EXISTS rate_limit_tiers_v5"));
        assert!(M_263_RATE_LIMIT_TIERS_V5_UP.contains("CREATE TABLE IF NOT EXISTS rate_limit_alerts_v2"));
        assert!(M_263_RATE_LIMIT_TIERS_V5_UP.contains("features"));
        assert!(M_263_RATE_LIMIT_TIERS_V5_UP.contains("limits"));
        assert!(M_263_RATE_LIMIT_TIERS_V5_UP.contains("threshold"));
    }

    #[test]
    fn test_rate_limit_tiers_v5_down_sql_not_empty() {
        assert_ne!(M_263_RATE_LIMIT_TIERS_V5_DOWN, "");
        assert!(M_263_RATE_LIMIT_TIERS_V5_DOWN.contains("DROP TABLE IF EXISTS rate_limit_alerts_v2"));
        assert!(M_263_RATE_LIMIT_TIERS_V5_DOWN.contains("DROP TABLE IF EXISTS rate_limit_tiers_v5"));
    }

    #[test]
    fn test_api_analytics_v8_sql_not_empty() {
        assert_ne!(M_264_API_ANALYTICS_V8_UP, "");
        assert!(M_264_API_ANALYTICS_V8_UP.contains("CREATE TABLE IF NOT EXISTS api_analytics_v8"));
        assert!(M_264_API_ANALYTICS_V8_UP.contains("endpoint"));
        assert!(M_264_API_ANALYTICS_V8_UP.contains("cost_cents"));
        assert!(M_264_API_ANALYTICS_V8_UP.contains("cache_hit"));
        assert!(M_264_API_ANALYTICS_V8_UP.contains("region"));
        assert!(M_264_API_ANALYTICS_V8_UP.contains("request_id"));
    }

    #[test]
    fn test_api_analytics_v8_down_sql_not_empty() {
        assert_ne!(M_264_API_ANALYTICS_V8_DOWN, "");
        assert!(M_264_API_ANALYTICS_V8_DOWN.contains("DROP TABLE IF EXISTS api_analytics_v8"));
    }

    #[test]
    fn test_api_docs_v10_sql_not_empty() {
        assert_ne!(M_325_API_DOCS_V10_UP, "");
        assert!(M_325_API_DOCS_V10_UP.contains("CREATE TABLE IF NOT EXISTS api_docs_v10"));
        assert!(M_325_API_DOCS_V10_UP.contains("endpoint"));
        assert!(M_325_API_DOCS_V10_UP.contains("method"));
        assert!(M_325_API_DOCS_V10_UP.contains("version"));
        assert!(M_325_API_DOCS_V10_UP.contains("security_schemes"));
        assert!(M_325_API_DOCS_V10_UP.contains("rate_limits"));
        assert!(M_325_API_DOCS_V10_UP.contains("changelog"));
        assert!(M_325_API_DOCS_V10_UP.contains("deprecated"));
    }

    #[test]
    fn test_api_docs_v10_down_sql_not_empty() {
        assert_ne!(M_325_API_DOCS_V10_DOWN, "");
        assert!(M_325_API_DOCS_V10_DOWN.contains("DROP TABLE IF EXISTS api_docs_v10"));
    }

    #[test]
    fn test_rate_limit_tiers_v8_sql_not_empty() {
        assert_ne!(M_326_RATE_LIMIT_TIERS_V8_UP, "");
        assert!(M_326_RATE_LIMIT_TIERS_V8_UP.contains("CREATE TABLE IF NOT EXISTS rate_limit_tiers_v8"));
        assert!(M_326_RATE_LIMIT_TIERS_V8_UP.contains("CREATE TABLE IF NOT EXISTS rate_limit_alerts_v5"));
        assert!(M_326_RATE_LIMIT_TIERS_V8_UP.contains("features"));
        assert!(M_326_RATE_LIMIT_TIERS_V8_UP.contains("limits"));
        assert!(M_326_RATE_LIMIT_TIERS_V8_UP.contains("threshold"));
    }

    #[test]
    fn test_rate_limit_tiers_v8_down_sql_not_empty() {
        assert_ne!(M_326_RATE_LIMIT_TIERS_V8_DOWN, "");
        assert!(M_326_RATE_LIMIT_TIERS_V8_DOWN.contains("DROP TABLE IF EXISTS rate_limit_alerts_v5"));
        assert!(M_326_RATE_LIMIT_TIERS_V8_DOWN.contains("DROP TABLE IF EXISTS rate_limit_tiers_v8"));
    }

    #[test]
    fn test_api_analytics_v11_sql_not_empty() {
        assert_ne!(M_327_API_ANALYTICS_V11_UP, "");
        assert!(M_327_API_ANALYTICS_V11_UP.contains("CREATE TABLE IF NOT EXISTS api_analytics_v11"));
        assert!(M_327_API_ANALYTICS_V11_UP.contains("endpoint"));
        assert!(M_327_API_ANALYTICS_V11_UP.contains("cost_cents"));
        assert!(M_327_API_ANALYTICS_V11_UP.contains("cache_hit"));
        assert!(M_327_API_ANALYTICS_V11_UP.contains("region"));
        assert!(M_327_API_ANALYTICS_V11_UP.contains("request_id"));
    }

    #[test]
    fn test_api_analytics_v11_down_sql_not_empty() {
        assert_ne!(M_327_API_ANALYTICS_V11_DOWN, "");
        assert!(M_327_API_ANALYTICS_V11_DOWN.contains("DROP TABLE IF EXISTS api_analytics_v11"));
    }

    #[test]
    fn test_log_aggregation_v14_up_sql_not_empty() {
        assert_ne!(M_442_LOG_AGGREGATION_V14_UP, "");
        assert!(M_442_LOG_AGGREGATION_V14_UP.contains("CREATE TABLE IF NOT EXISTS log_entries_v14"));
        assert!(M_442_LOG_AGGREGATION_V14_UP.contains("CREATE TABLE IF NOT EXISTS log_alert_rules_v11"));
        assert!(M_442_LOG_AGGREGATION_V14_UP.contains("level"));
        assert!(M_442_LOG_AGGREGATION_V14_UP.contains("message"));
        assert!(M_442_LOG_AGGREGATION_V14_UP.contains("source"));
        assert!(M_442_LOG_AGGREGATION_V14_UP.contains("service"));
        assert!(M_442_LOG_AGGREGATION_V14_UP.contains("trace_id"));
        assert!(M_442_LOG_AGGREGATION_V14_UP.contains("span_id"));
        assert!(M_442_LOG_AGGREGATION_V14_UP.contains("metadata"));
        assert!(M_442_LOG_AGGREGATION_V14_UP.contains("retention_days"));
        assert!(M_442_LOG_AGGREGATION_V14_UP.contains("indexed"));
    }

    #[test]
    fn test_log_aggregation_v14_down_sql_not_empty() {
        assert_ne!(M_442_LOG_AGGREGATION_V14_DOWN, "");
        assert!(M_442_LOG_AGGREGATION_V14_DOWN.contains("DROP TABLE IF EXISTS log_alert_rules_v11"));
        assert!(M_442_LOG_AGGREGATION_V14_DOWN.contains("DROP TABLE IF EXISTS log_entries_v14"));
    }

    #[test]
    fn test_distributed_tracing_v15_up_sql_not_empty() {
        assert_ne!(M_443_DISTRIBUTED_TRACING_V15_UP, "");
        assert!(M_443_DISTRIBUTED_TRACING_V15_UP.contains("CREATE TABLE IF NOT EXISTS trace_sampling_rules_v14"));
        assert!(M_443_DISTRIBUTED_TRACING_V15_UP.contains("CREATE TABLE IF NOT EXISTS trace_service_dependencies_v11"));
        assert!(M_443_DISTRIBUTED_TRACING_V15_UP.contains("service_name"));
        assert!(M_443_DISTRIBUTED_TRACING_V15_UP.contains("endpoint"));
        assert!(M_443_DISTRIBUTED_TRACING_V15_UP.contains("sample_rate"));
        assert!(M_443_DISTRIBUTED_TRACING_V15_UP.contains("max_traces_per_second"));
        assert!(M_443_DISTRIBUTED_TRACING_V15_UP.contains("priority"));
        assert!(M_443_DISTRIBUTED_TRACING_V15_UP.contains("depends_on_service"));
        assert!(M_443_DISTRIBUTED_TRACING_V15_UP.contains("call_count"));
        assert!(M_443_DISTRIBUTED_TRACING_V15_UP.contains("avg_duration_ms"));
        assert!(M_443_DISTRIBUTED_TRACING_V15_UP.contains("error_rate"));
    }

    #[test]
    fn test_distributed_tracing_v15_down_sql_not_empty() {
        assert_ne!(M_443_DISTRIBUTED_TRACING_V15_DOWN, "");
        assert!(M_443_DISTRIBUTED_TRACING_V15_DOWN.contains("DROP TABLE IF EXISTS trace_service_dependencies_v11"));
        assert!(M_443_DISTRIBUTED_TRACING_V15_DOWN.contains("DROP TABLE IF EXISTS trace_sampling_rules_v14"));
    }

    #[test]
    fn test_dashboard_reporting_v14_up_sql_not_empty() {
        assert_ne!(M_444_DASHBOARD_REPORTING_V14_UP, "");
        assert!(M_444_DASHBOARD_REPORTING_V14_UP.contains("CREATE TABLE IF NOT EXISTS dashboard_shares_v11"));
        assert!(M_444_DASHBOARD_REPORTING_V14_UP.contains("CREATE TABLE IF NOT EXISTS report_schedules_v12"));
        assert!(M_444_DASHBOARD_REPORTING_V14_UP.contains("dashboard_id"));
        assert!(M_444_DASHBOARD_REPORTING_V14_UP.contains("user_id"));
        assert!(M_444_DASHBOARD_REPORTING_V14_UP.contains("permission"));
        assert!(M_444_DASHBOARD_REPORTING_V14_UP.contains("report_id"));
        assert!(M_444_DASHBOARD_REPORTING_V14_UP.contains("cron_expression"));
        assert!(M_444_DASHBOARD_REPORTING_V14_UP.contains("next_run_at"));
    }

    #[test]
    fn test_dashboard_reporting_v14_down_sql_not_empty() {
        assert_ne!(M_444_DASHBOARD_REPORTING_V14_DOWN, "");
        assert!(M_444_DASHBOARD_REPORTING_V14_DOWN.contains("DROP TABLE IF EXISTS report_schedules_v12"));
        assert!(M_444_DASHBOARD_REPORTING_V14_DOWN.contains("DROP TABLE IF EXISTS dashboard_shares_v11"));
    }

    #[test]
    fn test_test_suite_metrics_baselines_v11_up_sql_not_empty() {
        assert_ne!(M_448_TEST_SUITE_METRICS_BASELINES_V11_UP, "");
        assert!(M_448_TEST_SUITE_METRICS_BASELINES_V11_UP.contains("CREATE TABLE IF NOT EXISTS test_suite_metrics_v11"));
        assert!(M_448_TEST_SUITE_METRICS_BASELINES_V11_UP.contains("CREATE TABLE IF NOT EXISTS test_suite_baselines_v11"));
        assert!(M_448_TEST_SUITE_METRICS_BASELINES_V11_UP.contains("suite_id"));
        assert!(M_448_TEST_SUITE_METRICS_BASELINES_V11_UP.contains("metric_name"));
        assert!(M_448_TEST_SUITE_METRICS_BASELINES_V11_UP.contains("metric_value"));
        assert!(M_448_TEST_SUITE_METRICS_BASELINES_V11_UP.contains("baseline_value"));
        assert!(M_448_TEST_SUITE_METRICS_BASELINES_V11_UP.contains("threshold_percent"));
    }

    #[test]
    fn test_test_suite_metrics_baselines_v11_down_sql_not_empty() {
        assert_ne!(M_448_TEST_SUITE_METRICS_BASELINES_V11_DOWN, "");
        assert!(M_448_TEST_SUITE_METRICS_BASELINES_V11_DOWN.contains("DROP TABLE IF EXISTS test_suite_baselines_v11"));
        assert!(M_448_TEST_SUITE_METRICS_BASELINES_V11_DOWN.contains("DROP TABLE IF EXISTS test_suite_metrics_v11"));
    }

    #[test]
    fn test_code_quality_metrics_v12_thresholds_v11_up_sql_not_empty() {
        assert_ne!(M_449_CODE_QUALITY_METRICS_V12_THRESHOLDS_V11_UP, "");
        assert!(M_449_CODE_QUALITY_METRICS_V12_THRESHOLDS_V11_UP.contains("CREATE TABLE IF NOT EXISTS code_quality_metrics_v12"));
        assert!(M_449_CODE_QUALITY_METRICS_V12_THRESHOLDS_V11_UP.contains("CREATE TABLE IF NOT EXISTS code_quality_thresholds_v11"));
        assert!(M_449_CODE_QUALITY_METRICS_V12_THRESHOLDS_V11_UP.contains("repo_id"));
        assert!(M_449_CODE_QUALITY_METRICS_V12_THRESHOLDS_V11_UP.contains("file_path"));
        assert!(M_449_CODE_QUALITY_METRICS_V12_THRESHOLDS_V11_UP.contains("metric_name"));
        assert!(M_449_CODE_QUALITY_METRICS_V12_THRESHOLDS_V11_UP.contains("threshold_value"));
        assert!(M_449_CODE_QUALITY_METRICS_V12_THRESHOLDS_V11_UP.contains("enabled"));
    }

    #[test]
    fn test_code_quality_metrics_v12_thresholds_v11_down_sql_not_empty() {
        assert_ne!(M_449_CODE_QUALITY_METRICS_V12_THRESHOLDS_V11_DOWN, "");
        assert!(M_449_CODE_QUALITY_METRICS_V12_THRESHOLDS_V11_DOWN.contains("DROP TABLE IF EXISTS code_quality_thresholds_v11"));
        assert!(M_449_CODE_QUALITY_METRICS_V12_THRESHOLDS_V11_DOWN.contains("DROP TABLE IF EXISTS code_quality_metrics_v12"));
    }

    #[test]
    fn test_performance_test_alerts_v12_up_sql_not_empty() {
        assert_ne!(M_450_PERFORMANCE_TEST_ALERTS_V12_UP, "");
        assert!(M_450_PERFORMANCE_TEST_ALERTS_V12_UP.contains("CREATE TABLE IF NOT EXISTS performance_test_alerts_v12"));
        assert!(M_450_PERFORMANCE_TEST_ALERTS_V12_UP.contains("CREATE TABLE IF NOT EXISTS performance_test_alert_history_v12"));
        assert!(M_450_PERFORMANCE_TEST_ALERTS_V12_UP.contains("baseline_id"));
        assert!(M_450_PERFORMANCE_TEST_ALERTS_V12_UP.contains("alert_type"));
        assert!(M_450_PERFORMANCE_TEST_ALERTS_V12_UP.contains("threshold"));
        assert!(M_450_PERFORMANCE_TEST_ALERTS_V12_UP.contains("enabled"));
        assert!(M_450_PERFORMANCE_TEST_ALERTS_V12_UP.contains("metric_name"));
    }

    #[test]
    fn test_performance_test_alerts_v12_down_sql_not_empty() {
        assert_ne!(M_450_PERFORMANCE_TEST_ALERTS_V12_DOWN, "");
        assert!(M_450_PERFORMANCE_TEST_ALERTS_V12_DOWN.contains("DROP TABLE IF EXISTS performance_test_alert_history_v12"));
        assert!(M_450_PERFORMANCE_TEST_ALERTS_V12_DOWN.contains("DROP TABLE IF EXISTS performance_test_alerts_v12"));
    }
}
