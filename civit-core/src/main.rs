#![forbid(unsafe_code)]

use anyhow::Result;
use civit_core::{api::create_router, config::AppConfig, db::DatabasePool};
use tokio::signal;
use tracing::info;
use tracing_subscriber::EnvFilter;

async fn shutdown_signal() {
    signal::ctrl_c().await.expect("failed to listen for ctrl+c");
    info!("received shutdown signal");
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("civit_core=info,tower_http=debug")),
        )
        .with_target(false)
        .init();

    let config = AppConfig::from_env()?;

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

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("CivitForge API listening on {}", addr);

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("server shutdown complete");
    Ok(())
}
