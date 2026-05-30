#![forbid(unsafe_code)]

use anyhow::Result;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("civit_runner=info")),
        )
        .with_target(false)
        .init();

    info!("CivitForge Runner starting");

    let pipeline_engine = civit_runner::PipelineEngine::new("default".into());

    let spec = civit_runner::models::PipelineSpec {
        name: "ci-pipeline".into(),
        triggers: vec!["push".into()],
        steps: vec![
            civit_runner::models::PipelineStep {
                name: "checkout".into(),
                image: "alpine/git:latest".into(),
                commands: vec!["git clone $REPO_URL .".into()],
                env: Default::default(),
                condition: None,
            },
            civit_runner::models::PipelineStep {
                name: "build".into(),
                image: "rust:1.75".into(),
                commands: vec!["cargo build --release".into()],
                env: Default::default(),
                condition: None,
            },
            civit_runner::models::PipelineStep {
                name: "test".into(),
                image: "rust:1.75".into(),
                commands: vec!["cargo test --release".into()],
                env: Default::default(),
                condition: None,
            },
        ],
    };

    pipeline_engine.run(&spec).await?;

    info!("Pipeline complete");
    Ok(())
}
