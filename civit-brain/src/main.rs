#![forbid(unsafe_code)]

use anyhow::Result;
use civit_brain::ParseEngine;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("civit_brain=info")),
        )
        .with_target(false)
        .init();

    info!("CivitForge Brain starting");

    let engine = ParseEngine::new();

    let sample = r#"
fn main() {
    let x = 42;
    println!("hello {}", x);
    if x > 40 {
        println!("big number");
    }
    for i in 0..10 {
        println!("{}", i);
    }
}
"#;

    let nodes = engine.parse(sample, "rust")?;
    info!(node_count = nodes.len(), "parsed code");
    for node in &nodes {
        info!(
            kind = ?node.node_type,
            name = %node.name,
            line = node.line_range.0,
            "AST node"
        );
    }

    let worker = civit_brain::EmbeddingWorker::new(384);
    let text = "This is a code review comment for a Rust function";
    let vector = worker.embed_text(text).await?;
    info!(dimensions = vector.data.len(), "generated embedding");

    info!("CivitForge Brain complete");
    Ok(())
}
