#![forbid(unsafe_code)]

use anyhow::Result;
use civit_core::{api::create_router, config::AppConfig};
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
    let router = create_router(config.clone())?;

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("CivitForge API listening on {}", addr);

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("server shutdown complete");
    Ok(())
}
