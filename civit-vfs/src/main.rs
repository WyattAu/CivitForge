#![forbid(unsafe_code)]

use anyhow::Result;
use civit_vfs::LruCache;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("civit_vfs=info")),
        )
        .with_target(false)
        .init();

    info!("CivitForge VFS starting");

    let mut cache = LruCache::new(1024 * 1024 * 256); // 256MB
    cache.insert("test.txt".into(), vec![b'h', b'e', b'l', b'l', b'o']);

    if let Some(data) = cache.get("test.txt") {
        info!(size = data.len(), "cache hit");
    }

    info!("CivitForge VFS ready");
    Ok(())
}
