#![forbid(unsafe_code)]

use anyhow::Result;
use civit_core::{api::create_router, config::AppConfig, db::DatabasePool};
use std::net::SocketAddr;
use tokio::signal;
use tracing::info;
use tracing_subscriber::EnvFilter;

async fn shutdown_signal() {
    signal::ctrl_c().await.expect("failed to listen for ctrl+c");
    info!("received shutdown signal");
}

#[tokio::main]
async fn main() -> Result<()> {
    let debug_mode = std::env::args().any(|arg| arg == "--debug");

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            if debug_mode {
                EnvFilter::new("civit_core=debug,tower_http=debug")
            } else {
                EnvFilter::new("civit_core=info,tower_http=debug")
            }
        }))
        .with_target(false)
        .init();

    let mut config = AppConfig::from_env()?;
    if debug_mode {
        config.debug_mode = true;
    }

    info!("connecting to database");
    let db_pool = DatabasePool::from_config(&config).await?;
    let pool = db_pool.pool().clone();

    let migration_mgr = civit_core::db::migrations::MigrationManager::new();
    let current_version: i64 =
        sqlx::query_as::<_, (i64,)>("SELECT COALESCE(MAX(version), 0) FROM schema_migrations")
            .fetch_one(&pool)
            .await
            .map(|r| r.0)
            .unwrap_or(0);

    let pending = migration_mgr.get_pending(current_version);
    if !pending.is_empty() {
        info!(
            current = current_version,
            pending = pending.len(),
            "running pending migrations"
        );
        for migration in &pending {
            info!(
                version = migration.version,
                name = %migration.name,
                "applying migration"
            );
            for stmt in migration.up_sql.split(';') {
                let s = stmt.trim();
                if !s.is_empty() {
                    sqlx::query(s).execute(&pool).await?;
                }
            }
            sqlx::query(
                "INSERT INTO schema_migrations (version, name, applied_at) VALUES ($1, $2, NOW())",
            )
            .bind(migration.version)
            .bind(&migration.name)
            .execute(&pool)
            .await?;
            info!(version = migration.version, "migration applied");
        }
        info!("all migrations applied");
    } else {
        info!(current = current_version, "database schema is up to date");
    }

    let router = create_router(config.clone(), pool)?;

    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .expect("invalid bind address");

    if config.tls_enabled() {
        let cert_path = config.tls_cert_path.as_ref().unwrap();
        let key_path = config.tls_key_path.as_ref().unwrap();

        let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(cert_path, key_path)
            .await
            .expect("failed to load TLS certificate/key");

        info!("CivitForge API listening on {} (TLS)", addr);
        let handle = axum_server::Handle::new();
        let shutdown_handle = handle.clone();
        tokio::spawn(async move {
            shutdown_signal().await;
            shutdown_handle.graceful_shutdown(Some(std::time::Duration::from_secs(30)));
        });
        axum_server::bind_rustls(addr, tls_config)
            .handle(handle)
            .serve(router.into_make_service())
            .await?;
    } else {
        info!("CivitForge API listening on {} (HTTP)", addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal())
            .await?;
    }

    info!("server shutdown complete");
    Ok(())
}
