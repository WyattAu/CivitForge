#![forbid(unsafe_code)]

pub mod auth;
pub mod auth_routes;
pub mod git_http;
pub mod issues;
pub mod oci;
pub mod orgs;
pub mod pipelines;
pub mod repos;
pub mod runners;
pub mod search;
pub mod ssh_keys;
pub mod users;
pub mod wiki;

use crate::config::AppConfig;
use crate::db::DbRepository;
use crate::error::Result;
use axum::Router;
use axum::extract::State;
use axum::extract::ws::WebSocketUpgrade;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use sqlx::postgres::PgPool;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn create_router(config: AppConfig, db: PgPool) -> Result<Router> {
    let state = AppState::new(config, db);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api = Router::new()
        .route("/healthz", get(health))
        .route("/ready", get(health))
        .route("/api/v1/health", get(health))
        .route("/api/v1/ws", get(ws_handler))
        .route("/api/v1/auth/login", post(auth_routes::login))
        .route("/api/v1/auth/me", get(auth_routes::me))
        .route("/api/v1/auth/refresh", post(auth_routes::refresh))
        .route(
            "/api/v1/repos",
            get(repos::list_repos).post(repos::create_repo),
        )
        .route(
            "/api/v1/repos/{owner}/{name}",
            get(repos::get_repo).delete(repos::delete_repo),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/commits",
            get(repos::list_commits),
        )
        .merge(pipelines::pipeline_routes())
        .merge(runners::runner_routes())
        .merge(oci::registry_routes())
        .merge(issues::issue_routes())
        .merge(wiki::wiki_routes())
        .merge(search::search_routes())
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
        .route("/api/v1/orgs", get(orgs::list_orgs).post(orgs::create_org))
        .route(
            "/api/v1/orgs/{id}",
            get(orgs::get_org).patch(orgs::update_org),
        )
        .route(
            "/api/v1/users/{user_id}/ssh-keys",
            get(ssh_keys::list_ssh_keys).post(ssh_keys::add_ssh_key),
        )
        .route(
            "/api/v1/ssh-keys/{key_id}",
            delete(ssh_keys::delete_ssh_key),
        )
        .route("/{owner}/{name}/info/refs", get(git_http::info_refs))
        .route(
            "/{owner}/{name}/git-upload-pack",
            post(git_http::upload_pack),
        )
        .route(
            "/{owner}/{name}/git-receive-pack",
            post(git_http::receive_pack),
        );

    let router = Router::new()
        .merge(api)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    Ok(router)
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
    pub session_manager: Arc<crate::db::SessionManager>,
    pub git_service: Arc<crate::git::GitService>,
}

impl AppState {
    pub fn new(config: AppConfig, db: PgPool) -> Self {
        let jwt_service = Arc::new(crate::auth::jwt::JwtService::new(
            &config.jwt_secret,
            config.jwt_expiry_hours,
        ));
        let event_bus = Arc::new(crate::events::EventBus::new(1000));
        let ws_manager = Arc::new(tokio::sync::RwLock::new(
            crate::events::WebSocketManager::new(event_bus.clone()),
        ));
        let session_manager = Arc::new(crate::db::SessionManager::new(
            db.clone(),
            std::time::Duration::from_secs(config.jwt_expiry_hours * 3600),
        ));
        let git_service = Arc::new(crate::git::GitService::new(std::path::PathBuf::from(
            &config.storage_path,
        )));
        Self {
            config,
            db: DbRepository::new(db),
            jwt_service,
            event_bus,
            ws_manager,
            session_manager,
            git_service,
        }
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    crate::events::websocket::ws_upgrade_handler(ws, State(state.ws_manager)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> AppConfig {
        AppConfig {
            host: "127.0.0.1".into(),
            port: 8080,
            database_url: "postgres://localhost/test".into(),
            redis_url: "redis://localhost:6379".into(),
            jwt_secret: "test-secret-key-32bytes-minimum".into(),
            jwt_expiry_hours: 24,
            federation_enabled: false,
            federation_instance_id: "test".into(),
            federation_instance_domain: "localhost".into(),
            storage_path: "/tmp/repos".into(),
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
