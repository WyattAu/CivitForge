#![forbid(unsafe_code)]

pub mod activity;
pub mod admin_dashboard;
pub mod api_analytics;
pub mod api_analytics_v2;
pub mod api_analytics_v3;
pub mod api_analytics_v5;
pub mod api_analytics_v6;
pub mod api_analytics_v9;
pub mod api_analytics_v11;
pub mod api_documentation;
pub mod api_docs_v2;
pub mod api_docs_v4;
pub mod api_docs_v5;
pub mod api_docs_v8;
pub mod api_docs_v10;
pub mod api_gateway;
pub mod api_transforms;
pub mod api_versioning;
pub mod artifact_serving;
pub mod audit_admin;
pub mod auth_routes;
pub mod auth;
pub mod badges;
pub mod boards;
pub mod branch_protection;
pub mod circuit_breaker;
pub mod chaos;
pub mod code_browser;
pub mod codeowners;
pub mod compliance;
pub mod deployment_strategy;
pub mod infrastructure;
pub mod service_mesh;
pub mod data_export;
pub mod deploy_keys;
pub mod deployments;
pub mod deployment_history;
pub mod diagnostics;
pub mod discussions;
pub mod edit;
pub mod environments;
pub mod error_reports;
pub mod event_queues;
pub mod events;
pub mod feature_flags;
pub mod federation_routes;
pub mod git_http;
pub mod graphql;
pub mod import;
pub mod issues;
pub mod issue_templates;
pub mod lfs;
pub mod license_compliance;
pub mod marketplace;
pub mod maven;
pub mod mentions;
pub mod npm;
pub mod observability;
pub mod merge_queue;
pub mod mirrors;
pub mod monitoring;
pub mod notifications;
pub mod oci;
pub mod oidc;
pub mod openapi_handler;
pub mod orgs;
pub mod pages;
pub mod password;
pub mod performance_metrics;
pub mod pipeline_actions;
pub mod pipeline_caches;
pub mod pipeline_log_stream;
pub mod pipeline_schedules;
pub mod pipeline_secrets;
pub mod pipeline_artifacts;
pub mod pipeline_environments;
pub mod pipeline_secrets_v2_api;
pub mod runners_v2_api;
pub mod environment_variables_api;
pub mod pipelines;
pub mod pr_templates;
pub mod pull_requests;
pub mod releases;
pub mod repos;
pub mod rate_limiting_v2;
pub mod rate_limiting_v3;
pub mod rate_limiting_v4;
pub mod rate_limiting_v6;
pub mod rate_limiting_v7;
pub mod resilience;
pub mod runners;
pub mod saml;
pub mod scim;
pub mod search;
pub mod secret_scanning;
pub mod security_dashboard;
pub mod dependencies;
pub mod site_settings;
pub mod slsa_dashboard;
pub mod sso;
pub mod ssh_keys;
pub mod teams;
pub mod tokens;
pub mod usage_quotas;
pub mod users;
#[cfg(feature = "webauthn")]
pub mod webauthn;
pub mod webhooks;
pub mod wiki;
pub mod database_replication_v2_api;
pub mod database_replication_v5_api;
pub mod encryption_v3_api;
pub mod encryption_v6_api;
pub mod data_residency_v2_api;
pub mod test_suite_management_v5;
pub mod test_suite_management_v6;
pub mod code_quality_rules_v5;
pub mod code_quality_rules_v6;
pub mod performance_testing_v6;
pub mod performance_testing_v7;
pub mod data_residency_v5_api;
pub mod database_replication_v8_api;
pub mod encryption_v9_api;
pub mod data_residency_v8_api;

use crate::config::AppConfig;
use crate::db::DbRepository;
use crate::error::Result;
use crate::federation::ForgeFedProcessor;
use crate::middleware::csrf::csrf_middleware;
use crate::middleware::rate_limit::{RateLimitConfig, RateLimiter, rate_limit_middleware};
use crate::search::tantivy_index::CodeSearchIndex;
use crate::wiki::WikiGitBackend;
use axum::Router;
use axum::extract::{Request, State};
use axum::extract::ws::WebSocketUpgrade;
use axum::http::{HeaderName, HeaderValue};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post};
use sqlx::postgres::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

pub fn create_router(config: AppConfig, db: PgPool) -> Result<Router> {
    let state = AppState::new(config, db);

    let cors = if state.config.cors_allowed_origins.is_empty()
        || state.config.cors_allowed_origins.iter().any(|o| o == "*")
    {
        CorsLayer::permissive()
    } else {
        let origins = state
            .config
            .cors_allowed_origins
            .iter()
            .filter_map(|o| o.parse::<axum::http::HeaderValue>().ok())
            .collect::<Vec<_>>();
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods(AllowMethods::any())
            .allow_headers(AllowHeaders::any())
    };

    let mut api = Router::new()
        .route("/healthz", get(health))
        .route("/ready", get(health))
        .route("/api/v1/health", get(health))
        .route("/api/v1/ws", get(ws_handler))
        .merge(pipeline_log_stream::log_stream_routes())
        .route("/api/v1/auth/login", post(auth_routes::login))
        .route("/api/v1/auth/register", post(auth_routes::register))
        .route("/api/v1/auth/verify-email", post(auth_routes::verify_email))
        .route("/api/v1/auth/me", get(auth_routes::me))
        .route("/api/v1/auth/refresh", post(auth_routes::refresh))
        .route("/api/v1/auth/ldap/sync", post(auth_routes::ldap_sync))
        .route(
            "/api/v1/auth/change-password",
            post(auth_routes::change_password_auth),
        )
        .route(
            "/api/v1/admin/ldap/status",
            get(auth_routes::ldap_admin_status),
        )
        .route(
            "/api/v1/admin/ldap/test",
            post(auth_routes::ldap_admin_test),
        )
        .route(
            "/api/v1/admin/ldap/sync",
            post(auth_routes::ldap_admin_sync),
        )
        .route(
            "/api/v1/admin/oidc-providers/{id}/test",
            post(auth_routes::oidc_admin_test),
        )
        .route(
            "/api/v1/admin/oidc-providers/users-count",
            get(auth_routes::oidc_admin_users_count),
        )
        .route(
            "/api/v1/repos",
            get(repos::list_repos).post(repos::create_repo),
        )
        .route(
            "/api/v1/repos/{owner}/{name}",
            get(repos::get_repo)
                .patch(repos::update_repo)
                .delete(repos::delete_repo),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/commits",
            get(repos::list_commits),
        )
        .merge(repos::repo_routes())
        .merge(pipelines::pipeline_routes())
        .merge(pipeline_artifacts::artifact_routes())
        .merge(pipeline_environments::environment_routes())
        .merge(pipeline_schedules::schedule_routes())
        .merge(badges::badge_routes())
        .merge(pipeline_secrets::pipeline_secret_routes())
        .merge(pipeline_actions::pipeline_action_routes())
        .merge(pipeline_caches::pipeline_cache_routes())
        .merge(runners::runner_routes())
        .merge(pipeline_secrets_v2_api::pipeline_secrets_v2_routes())
        .merge(runners_v2_api::runners_v2_routes())
        .merge(environment_variables_api::environment_variables_routes())
        .merge(oci::registry_routes())
        .merge(issues::issue_routes())
        .merge(issue_templates::issue_template_routes())
        .merge(pr_templates::pr_template_routes())
        .merge(discussions::discussion_routes())
        .merge(boards::board_routes())
        .merge(edit::edit_routes())
        .merge(pull_requests::pr_routes())
        .merge(merge_queue::merge_queue_routes())
        .merge(releases::release_routes())
        .merge(branch_protection::branch_protection_routes())
        .merge(wiki::wiki_routes())
        .merge(search::search_routes())
        .merge(activity::activity_routes())
        .merge(code_browser::code_browser_routes())
        .merge(secret_scanning::secret_scanning_routes())
        .merge(security_dashboard::security_dashboard_routes())
        .merge(dependencies::dependency_routes())
        .merge(slsa_dashboard::slsa_dashboard_routes())
        .merge(federation_routes::federation_routes())
        .merge(mirrors::mirror_routes())
        .merge(lfs::lfs_routes())
        .merge(password::password_routes())
        .merge(artifact_serving::artifact_serving_routes())
        .merge(openapi_handler::openapi_routes())
        .merge(marketplace::marketplace_routes())
        .merge(npm::npm_routes())
        .merge(maven::maven_routes())
        .merge(license_compliance::license_routes())
        .merge(pages::pages_routes())
        .merge(tokens::token_routes())
        .merge(webhooks::webhook_routes())
        .merge(events::event_routes())
        .merge(event_queues::event_queue_routes())
        .route(
            "/api/v1/repos/{owner}/{name}/webhooks",
            get(webhooks::list_webhooks).post(webhooks::create_webhook),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/deploy-keys",
            get(deploy_keys::list_deploy_keys).post(deploy_keys::create_deploy_key),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/codeowners",
            get(codeowners::get_codeowners).put(codeowners::update_codeowners),
        )
        .route("/graphql", post(graphql::graphql_endpoint))
        .route("/graphql/playground", get(graphql::graphql_playground))
        .route("/graphql/subscribe", get(graphql::graphql_subscribe))
        .merge(deploy_keys::deploy_key_routes())
        .merge(deployments::deployment_routes())
        .merge(deployment_history::deployment_history_routes())
        .merge(environments::environment_routes())
        .merge(monitoring::monitoring_routes())
        .merge(performance_metrics::performance_metric_routes())
        .merge(notifications::notification_routes())
        .route("/api/v1/user/tokens", post(tokens::create_token))
        .route(
            "/api/v1/users",
            get(users::list_users).post(users::create_user),
        )
        .route(
            "/api/v1/users/{id}",
            get(users::get_user)
                .patch(users::update_user)
                .delete(users::delete_user),
        )
        .route("/api/v1/user/profile", patch(users::update_profile))
        .route(
            "/api/v1/auth/oidc/exchange",
            post(oidc::exchange_oidc_token),
        )
        .route(
            "/api/v1/oauth/authorize",
            get(auth_routes::oauth_authorize),
        )
        .route(
            "/api/v1/oauth/token",
            post(auth_routes::oauth_token),
        )
        .route(
            "/api/v1/oauth/refresh",
            post(auth_routes::oauth_refresh),
        )
        .route(
            "/api/v1/oauth/clients",
            post(auth_routes::oauth_register_client),
        )
        .route("/api/v1/orgs", get(orgs::list_orgs).post(orgs::create_org))
        .route(
            "/api/v1/orgs/{id}",
            get(orgs::get_org).patch(orgs::update_org),
        )
        .merge(teams::team_routes())
        .merge(audit_admin::audit_admin_routes())
        .route(
            "/api/v1/admin/settings",
            get(site_settings::get_site_settings).put(site_settings::update_site_settings),
        )
        .merge(oidc::oidc_routes())
        .merge(saml::saml_routes())
        .merge(scim::scim_routes())
        .merge(sso::sso_routes())
        .merge(feature_flags::feature_flag_routes())
        .merge(admin_dashboard::admin_dashboard_routes())
        .merge(api_analytics::api_analytics_routes())
        .merge(api_analytics_v2::api_analytics_v2_routes())
        .merge(api_analytics_v3::api_analytics_v3_routes())
        .merge(api_analytics_v5::api_analytics_v5_routes())
        .merge(api_analytics_v6::api_analytics_v6_routes())
        .merge(api_analytics_v9::api_analytics_v9_routes())
        .merge(api_analytics_v11::api_analytics_v11_routes())
        .merge(api_documentation::api_documentation_routes())
        .merge(api_docs_v2::api_docs_v2_routes())
        .merge(api_docs_v4::api_docs_v4_routes())
        .merge(api_docs_v5::api_docs_v5_routes())
        .merge(api_docs_v8::api_docs_v8_routes())
        .merge(api_docs_v10::api_docs_v10_routes())
        .merge(api_versioning::api_version_routes())
        .merge(rate_limiting_v2::rate_limiting_v2_routes())
        .merge(rate_limiting_v3::rate_limiting_v3_routes())
        .merge(rate_limiting_v4::rate_limiting_v4_routes())
        .merge(rate_limiting_v6::rate_limiting_v6_routes())
        .merge(rate_limiting_v7::rate_limiting_v7_routes())
        .merge(usage_quotas::usage_quota_routes())
        .merge(data_export::export_routes())
        .merge(compliance::compliance_routes())
        .merge(observability_routes())
        .merge(chaos::chaos_routes())
        .merge(resilience::resilience_routes())
        .merge(circuit_breaker::circuit_breaker_routes())
        .merge(api_gateway::gateway_routes())
        .merge(api_transforms::transform_routes())
        .merge(deployment_strategy::deployment_strategy_routes())
        .merge(infrastructure::infrastructure_routes())
        .merge(service_mesh::service_mesh_routes())
        .merge(database_replication_v2_api::replication_v2_routes())
        .merge(database_replication_v5_api::replication_v5_routes())
        .merge(database_replication_v8_api::replication_v8_routes())
        .merge(encryption_v3_api::encryption_v3_routes())
        .merge(encryption_v6_api::encryption_v6_routes())
        .merge(encryption_v9_api::encryption_v9_routes())
        .merge(data_residency_v2_api::data_residency_v2_routes())
        .merge(data_residency_v5_api::data_residency_v5_routes())
        .merge(data_residency_v8_api::data_residency_v8_routes())
        .merge(test_suite_management_v5::test_suite_v5_routes())
        .merge(test_suite_management_v6::test_suite_v6_routes())
        .merge(code_quality_rules_v5::code_quality_v5_routes())
        .merge(code_quality_rules_v6::code_quality_v6_routes())
        .merge(performance_testing_v6::performance_testing_v6_routes())
        .merge(performance_testing_v7::performance_testing_v7_routes())
        .route("/api/v1/orgs/{id}/profile", get(orgs::get_org_profile))
        .route("/api/v1/import/github", post(import::import_github))
        .route("/api/v1/import/gitlab", post(import::import_gitlab))
        .route("/api/v1/import/url", post(import::import_url))
        .route(
            "/api/v1/users/{user_id}/ssh-keys",
            get(ssh_keys::list_ssh_keys).post(ssh_keys::add_ssh_key),
        )
        .route(
            "/api/v1/ssh-keys/{key_id}",
            delete(ssh_keys::delete_ssh_key),
        );

    #[cfg(feature = "webauthn")]
    {
        api = api.merge(webauthn::webauthn_routes());
    }

    // Git smart HTTP routes — large body limit for pack data (up to 10 GB)
    let git_routes = Router::new()
        .route("/{owner}/{name}/info/refs", get(git_http::info_refs))
        .route(
            "/{owner}/{name}/git-upload-pack",
            post(git_http::upload_pack),
        )
        .route(
            "/{owner}/{name}/git-receive-pack",
            post(git_http::receive_pack),
        )
        .layer(axum::extract::DefaultBodyLimit::max(
            10 * 1024 * 1024 * 1024, // 10 GB
        ));

    api = api.merge(git_routes);

    if state.config.debug_mode {
        api = api
            .merge(diagnostics::diagnostics_routes())
            .merge(error_reports::error_reports_routes());
    }

    let rate_limiter = Arc::new(RateLimiter::new(RateLimitConfig {
        max_requests: state.config.rate_limit_max_requests.unwrap_or(100),
        window: Duration::from_secs(state.config.rate_limit_window_secs.unwrap_or(60) as u64),
    }));

    let state_jwt_service = state.jwt_service.clone();

    let ui_dir = std::path::PathBuf::from(&state.config.ui_assets_path);
    let index_path = ui_dir.join("index.html");
    let has_ui = ui_dir.is_dir() && index_path.is_file();

    let debug_mode = state.config.debug_mode;

    let mut router = Router::new()
        .merge(api)
        // API v2 routes (backward compatible, with deprecation notice on v1)
        .merge(api_v2_routes())
        .layer(cors)
        .layer(middleware::from_fn(rate_limit_middleware))
        .layer(middleware::from_fn(csrf_middleware))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            axum::http::header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-xss-protection"),
            HeaderValue::from_static("0"),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            axum::http::header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            axum::http::header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                 "default-src 'self'; \
                  script-src 'self' 'unsafe-inline' 'unsafe-eval' 'wasm-unsafe-eval' https://cdn.jsdelivr.net; \
                  style-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net; \
                  img-src 'self' data:; font-src 'self'; \
                  connect-src *; \
                  frame-ancestors 'none'; \
                  upgrade-insecure-requests;",
            ),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            axum::http::header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone())
        .layer(axum::Extension(rate_limiter))
        .layer(axum::Extension(state_jwt_service))
        .layer(axum::Extension(std::sync::Arc::new(state.db.clone())))
        .layer(middleware::from_fn(crate::middleware::api_analytics::api_analytics_middleware));

    if debug_mode {
        router = router
            .layer(middleware::from_fn(
                crate::middleware::error_reporter::panic_catcher,
            ))
            .layer(middleware::from_fn(
                crate::middleware::debug::debug_middleware,
            ));
    }

    // SPA fallback: serve static files, fall back to index.html for client-side routing
    if has_ui {
        router =
            router.fallback_service(ServeDir::new(&ui_dir).fallback(ServeFile::new(&index_path)));
    } else {
        tracing::warn!(
            "UI assets directory not found at {:?}, web UI will not be served",
            ui_dir
        );
        router = router.fallback_service(ServeDir::new("/tmp/nonexistent-civit-ui"));
    }

    // Automation endpoints for Playwright testing
    router = router.route("/__navigate__", get(|| async { "" }));
    router = router.route("/__capture__", post(|| async { "" }));
    router = router.route("/__logout__", get(|| async { "" }));

    Ok(router)
}

/// Observability routes for traces and metrics.
fn observability_routes() -> axum::Router<AppState> {
    let obs_state = Arc::new(observability::ObservabilityState {
        provider: Arc::new(crate::telemetry::opentelemetry::InstrumentationProvider::new(
            crate::telemetry::opentelemetry::Resource::default(),
        )),
    });

    Router::new()
        .route(
            "/api/v1/observability/traces",
            get(observability::list_traces),
        )
        .route(
            "/api/v1/observability/metrics",
            get(observability::list_metrics),
        )
        .route(
            "/api/v1/observability/traces/export",
            post(observability::export_traces),
        )
        .with_state(obs_state)
}

/// API v2 routes with forward-compatible versioning.
///
/// v2 endpoints are aliases for v1 endpoints. When breaking changes are introduced,
/// v2 will diverge while v1 remains stable. v1 endpoints include a deprecation header.
fn api_v2_routes() -> axum::Router<AppState> {
    Router::new()
        .route("/api/v2/health", get(health))
        .route("/api/v2/repos", get(repos::list_repos).post(repos::create_repo))
        .route(
            "/api/v2/repos/{owner}/{name}",
            get(repos::get_repo)
                .patch(repos::update_repo)
                .delete(repos::delete_repo),
        )
        .route(
            "/api/v2/repos/{owner}/{name}/commits",
            get(repos::list_commits),
        )
        .route("/api/v2/auth/login", post(auth_routes::login))
        .route("/api/v2/auth/register", post(auth_routes::register))
        .route("/api/v2/auth/me", get(auth_routes::me))
        .route("/api/v2/auth/refresh", post(auth_routes::refresh))
        .route("/api/v2/users", get(users::list_users).post(users::create_user))
        .route(
            "/api/v2/users/{id}",
            get(users::get_user)
                .patch(users::update_user)
                .delete(users::delete_user),
        )
        .route("/api/v2/user/profile", patch(users::update_profile))
        .route("/api/v2/orgs", get(orgs::list_orgs).post(orgs::create_org))
        .route(
            "/api/v2/orgs/{id}",
            get(orgs::get_org).patch(orgs::update_org),
        )
        .route("/api/v2/search", get(search::global_search))
        .layer(middleware::from_fn(deprecation_header_middleware))
}

/// Middleware that adds deprecation headers to v1 API responses.
async fn deprecation_header_middleware(mut req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;

    // Add deprecation headers for v1 endpoints
    let headers = response.headers_mut();
    headers.insert(
        HeaderName::from_static("sunset"),
        HeaderValue::from_static("2027-01-01"),
    );
    headers.insert(
        HeaderName::from_static("deprecation"),
        HeaderValue::from_static("true"),
    );
    headers.insert(
        HeaderName::from_static("link"),
        HeaderValue::from_static("</api/v2>; rel=\"successor-version\""),
    );

    response
}

async fn health() -> &'static str {
    "OK"
}

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: DbRepository,
    pub jwt_service: Arc<crate::auth::jwt::JwtService>,
    pub event_bus: Arc<crate::events::EventBus>,
    pub ws_manager: Arc<tokio::sync::RwLock<crate::events::WebSocketManager>>,
    pub log_broadcaster: Arc<crate::events::LogBroadcaster>,
    pub session_manager: Arc<crate::db::SessionManager>,
    pub git_service: Arc<crate::git::GitService>,
    pub forgefed_processor: Arc<ForgeFedProcessor>,
    pub code_search_index: Arc<RwLock<CodeSearchIndex>>,
    pub wiki_git: Arc<WikiGitBackend>,
    pub notification_broadcaster: Arc<tokio::sync::broadcast::Sender<String>>,
    #[cfg(feature = "webauthn")]
    pub webauthn_service: Option<Arc<civit_auth::webauthn::WebAuthnService>>,
}

impl AppState {
    pub fn new(config: AppConfig, db: PgPool) -> Self {
        let jwt_service = Arc::new(
            crate::auth::jwt::JwtService::new(&config.jwt_secret, config.jwt_expiry_hours)
                .expect("JWT secret must be at least 32 bytes"),
        );
        let event_bus = Arc::new(crate::events::EventBus::new(1000));
        let ws_manager = Arc::new(tokio::sync::RwLock::new(
            crate::events::WebSocketManager::new(event_bus.clone()),
        ));
        let log_broadcaster = Arc::new(crate::events::LogBroadcaster::new(1024));
        let session_manager = Arc::new(crate::db::SessionManager::new(
            db.clone(),
            std::time::Duration::from_secs(config.jwt_expiry_hours * 3600),
        ));
        let git_service = Arc::new(crate::git::GitService::new(std::path::PathBuf::from(
            &config.storage_path,
        )));
        let forgefed_processor = Arc::new(ForgeFedProcessor::new(
            config.federation_instance_domain.clone(),
            config.federation_instance_id.clone(),
        ));

        let tantivy_path = std::path::Path::new(&config.storage_path).join("tantivy-index");
        let code_search_index = match CodeSearchIndex::new(&tantivy_path) {
            Ok(idx) => Arc::new(RwLock::new(idx)),
            Err(e) => {
                tracing::warn!(
                    "failed to open tantivy code search index at {tantivy_path:?}: {e}, using in-memory index"
                );
                match CodeSearchIndex::new_in_memory() {
                    Ok(idx) => Arc::new(RwLock::new(idx)),
                    Err(e) => {
                        tracing::error!("failed to create in-memory tantivy index: {e}");
                        panic!("could not initialize tantivy search index: {e}");
                    }
                }
            }
        };

        let wiki_git_path = std::path::Path::new(&config.storage_path).join("wikis");
        let wiki_git = match WikiGitBackend::new(wiki_git_path) {
            Ok(b) => Arc::new(b),
            Err(e) => {
                tracing::warn!("failed to init wiki git backend: {e}");
                Arc::new(
                    WikiGitBackend::new(tempfile::tempdir().unwrap().path().to_path_buf()).unwrap(),
                )
            }
        };

        Self {
            config,
            db: DbRepository::new(db),
            jwt_service,
            event_bus,
            ws_manager,
            log_broadcaster,
            session_manager,
            git_service,
            forgefed_processor,
            code_search_index,
            wiki_git,
            notification_broadcaster: Arc::new(tokio::sync::broadcast::channel(256).0),
            #[cfg(feature = "webauthn")]
            webauthn_service: {
                let rp_name =
                    std::env::var("WEBAUTHN_RP_NAME").unwrap_or_else(|_| "CivitForge".into());
                let rp_id = std::env::var("WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".into());
                let origin = std::env::var("WEBAUTHN_ORIGIN")
                    .unwrap_or_else(|_| "http://localhost:8080".into());

                match civit_auth::webauthn::WebAuthnService::new(
                    civit_auth::webauthn::WebAuthnConfig {
                        relying_party_name: rp_name,
                        relying_party_id: rp_id,
                        origin,
                    },
                ) {
                    Ok(service) => Some(Arc::new(service)),
                    Err(e) => {
                        tracing::warn!("failed to initialize WebAuthn: {e}");
                        None
                    }
                }
            },
        }
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    crate::events::websocket::ws_upgrade_handler(ws, State(state.ws_manager)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SecurityConfig;

    fn test_config() -> AppConfig {
        AppConfig {
            host: "127.0.0.1".into(),
            port: 8080,
            database_url: "postgres://localhost/test".into(),
            redis_url: "redis://localhost:6379".into(),
            jwt_secret: "test-secret-key-32bytes-minimums".into(),
            jwt_expiry_hours: 24,
            federation_enabled: false,
            federation_instance_id: "test".into(),
            federation_instance_domain: "localhost".into(),
            storage_path: "/tmp/repos".into(),
            cors_allowed_origins: Vec::new(),
            rate_limit_max_requests: None,
            rate_limit_window_secs: None,
            security: SecurityConfig::default(),
            tls_cert_path: None,
            tls_key_path: None,
            ui_assets_path: "./crates/civit-ui/dist".into(),
            debug_mode: false,
        }
    }

    #[tokio::test]
    async fn test_app_state_new() {
        let opts = sqlx::postgres::PgPoolOptions::new().max_connections(1);
        let pool = opts.connect_lazy("postgres://localhost/test").unwrap();
        let state = AppState::new(test_config(), pool);
        assert_eq!(state.config.port, 8080);
        assert!(Arc::strong_count(&state.event_bus) >= 1);
        assert!(Arc::strong_count(&state.ws_manager) >= 1);
        assert!(Arc::strong_count(&state.session_manager) >= 1);
    }

    #[tokio::test]
    async fn test_app_state_jwt_service_works() {
        let opts = sqlx::postgres::PgPoolOptions::new().max_connections(1);
        let pool = opts.connect_lazy("postgres://localhost/test").unwrap();
        let state = AppState::new(test_config(), pool);
        let token = state
            .jwt_service
            .generate_token("u1", "alice", "admin", None)
            .unwrap();
        let claims = state.jwt_service.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "u1");
        assert_eq!(claims.username, "alice");
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        assert_eq!(health().await, "OK");
    }
}
