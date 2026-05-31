#![forbid(unsafe_code)]

pub mod repos;

use crate::config::AppConfig;
use crate::db::DbRepository;
use crate::error::Result;
use axum::Router;
use axum::routing::get;
use sqlx::postgres::PgPool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn create_router(config: AppConfig, db: PgPool) -> Result<Router> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api = Router::new()
        .route("/healthz", get(health))
        .route("/ready", get(health))
        .route("/api/v1/health", get(health))
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
        );

    let router = Router::new()
        .merge(api)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(AppState::new(config, db));

    Ok(router)
}

async fn health() -> &'static str {
    "OK"
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: DbRepository,
}

impl AppState {
    pub fn new(config: AppConfig, db: PgPool) -> Self {
        Self {
            config,
            db: DbRepository::new(db),
        }
    }
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
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        assert_eq!(health().await, "OK");
    }
}
