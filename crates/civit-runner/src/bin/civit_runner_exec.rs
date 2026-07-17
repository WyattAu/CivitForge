//! CivitForge Standalone Runner
//!
//! A lightweight CI/CD runner that:
//! 1. Registers with the CivitForge server
//! 2. Polls for available pipeline jobs
//! 3. Claims and executes jobs using Podman containers
//! 4. Reports step/job status back to the server
//!
//! Usage:
//!   civit-runner-exec --url https://forge.example.com --name my-runner [--labels linux,amd64]

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use clap::Parser;

use civit_runner::exec::client::{PollJobResponse, RunnerClient};
use civit_runner::exec::executor::StepExecutor;
use civit_runner::exec::executor::StepResult;
use civit_runner::exec::workspace::WorkspaceManager;
use civit_runner::podman::PodmanService;

/// CivitForge standalone CI/CD runner
#[derive(Debug, Parser)]
#[command(name = "civit-runner-exec", version)]
struct Args {
    /// CivitForge server URL
    #[arg(long, env = "CIVITFORGE_URL", default_value = "http://localhost:8080")]
    url: String,

    /// Runner name
    #[arg(long, env = "RUNNER_NAME", default_value = "standalone-runner")]
    name: String,

    /// Runner labels (comma-separated)
    #[arg(long, env = "RUNNER_LABELS", default_value = "linux,amd64")]
    labels: String,

    /// Runner group
    #[arg(long, env = "RUNNER_GROUP")]
    group: Option<String>,

    /// Poll interval in seconds
    #[arg(long, env = "RUNNER_POLL_INTERVAL", default_value = "5")]
    poll_interval: u64,

    /// Heartbeat interval in seconds
    #[arg(long, env = "RUNNER_HEARTBEAT_INTERVAL", default_value = "30")]
    heartbeat_interval: u64,

    /// Default container image for steps without explicit image
    #[arg(
        long,
        env = "RUNNER_DEFAULT_IMAGE",
        default_value = "wolfi-base:latest"
    )]
    default_image: String,

    /// Default step timeout in seconds
    #[arg(long, env = "RUNNER_DEFAULT_TIMEOUT", default_value = "600")]
    default_timeout: u64,

    /// Default memory limit in MB per container
    #[arg(long, env = "RUNNER_MEMORY_MB", default_value = "512")]
    memory_mb: u64,

    /// Workspace directory
    #[arg(
        long,
        env = "RUNNER_WORKSPACE",
        default_value = "/tmp/civit-runner-workspaces"
    )]
    workspace: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "civit_runner=info".parse().expect("invalid value")),
        )
        .init();

    let args = Args::parse();

    // Check Podman is available
    let podman = PodmanService::new();
    if !podman.health().await.unwrap_or(false) {
        anyhow::bail!("Podman is not available. Please install and start Podman.");
    }

    tracing::info!(
        name = %args.name,
        url = %args.url,
        labels = %args.labels,
        "starting CivitForge runner"
    );

    // Register runner
    let labels: Vec<&str> = args.labels.split(',').collect();
    let (runner_id, token) =
        RunnerClient::register(&args.url, &args.name, &labels, args.group.as_deref()).await?;

    tracing::info!(
        id = %runner_id,
        "runner registered successfully"
    );

    let client = RunnerClient::new(&args.url, &runner_id, &token);
    let workspace_mgr = WorkspaceManager::new(&args.workspace);
    workspace_mgr.ensure_root()?;

    let executor = StepExecutor::new(
        PodmanService::new(),
        &args.default_image,
        args.default_timeout,
        args.memory_mb,
    );

    let shutdown = Arc::new(AtomicBool::new(false));

    // Graceful shutdown handler
    let shutdown_clone = shutdown.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        tracing::info!("shutdown signal received, finishing current job...");
        shutdown_clone.store(true, Ordering::SeqCst);
    });

    // Heartbeat task
    let heartbeat_client = client.clone();
    let heartbeat_shutdown = shutdown.clone();
    let heartbeat_interval = args.heartbeat_interval;
    tokio::spawn(async move {
        loop {
            if heartbeat_shutdown.load(Ordering::SeqCst) {
                break;
            }
            if let Err(e) = heartbeat_client.heartbeat().await {
                tracing::warn!(error = %e, "heartbeat failed");
            }
            tokio::time::sleep(Duration::from_secs(heartbeat_interval)).await;
        }
    });

    // Main poll-execute loop
    loop {
        if shutdown.load(Ordering::SeqCst) {
            tracing::info!("shutting down");
            break;
        }

        match client.poll_job().await {
            Ok(Some(job)) => {
                tracing::info!(
                    job_id = %job.job_id,
                    run_id = %job.run_id,
                    name = %job.name,
                    "job received"
                );

                // Claim the job
                match client.claim_job(&job.job_id).await {
                    Ok(true) => {
                        tracing::info!(job_id = %job.job_id, "job claimed");
                    }
                    Ok(false) => {
                        tracing::warn!(job_id = %job.job_id, "job already claimed by another runner");
                        continue;
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "failed to claim job");
                        continue;
                    }
                }

                // Execute the job
                let _job_status = execute_job(&client, &executor, &workspace_mgr, &job).await;

                // Clean up workspace
                let _ = workspace_mgr.cleanup(&job.job_id).await;
            }
            Ok(None) => {
                // No jobs available — wait and poll again
                tokio::time::sleep(Duration::from_secs(args.poll_interval)).await;
            }
            Err(e) => {
                tracing::error!(error = %e, "poll failed, retrying...");
                tokio::time::sleep(Duration::from_secs(args.poll_interval)).await;
            }
        }
    }

    Ok(())
}

/// Execute a full job: prepare workspace, resolve secrets, run steps, report results.
async fn execute_job(
    client: &RunnerClient,
    executor: &StepExecutor,
    workspace_mgr: &WorkspaceManager,
    job: &PollJobResponse,
) -> String {
    // Prepare workspace
    let workspace = match workspace_mgr
        .prepare(&job.job_id, &job.repo_url, &job.commit_sha)
        .await
    {
        Ok(path) => path,
        Err(e) => {
            tracing::error!(error = %e, "failed to prepare workspace");
            let _ = client.complete_job(&job.job_id, "failure", None).await;
            return "failure".to_string();
        }
    };

    // Parse job-level env
    let mut job_env = HashMap::new();
    if let serde_json::Value::Object(map) = &job.env {
        for (k, v) in map {
            if let serde_json::Value::String(s) = v {
                job_env.insert(k.clone(), s.clone());
            }
        }
    }

    // Resolve secrets from server
    let secrets = client
        .resolve_secrets(&job.repo_url, &job.secret_names)
        .await
        .unwrap_or_default();

    tracing::info!(secrets_count = secrets.len(), "secrets resolved");

    // Execute steps sequentially
    let mut final_status = "success".to_string();
    let outputs = serde_json::json!({});

    for step in &job.steps {
        // Use step name as the step_id for API reporting
        let step_id = &step.name;

        if final_status == "failure" && !step.continue_on_error {
            // Skip remaining steps on failure (unless continue_on_error)
            let _ = client.update_step(step_id, "skipped", None, None).await;
            continue;
        }

        // Report step as running
        let _ = client.update_step(step_id, "running", None, None).await;

        // Execute step with workspace volume mount + streaming
        let result = executor
            .execute_step(step, &workspace, &job_env, &secrets, client, step_id)
            .await
            .unwrap_or_else(|e| StepResult {
                step_name: step.name.clone(),
                status: "failure".to_string(),
                exit_code: -1,
                output: format!("Execution error: {e}"),
                duration: Duration::ZERO,
            });

        // Report step result
        let _ = client
            .update_step(
                step_id,
                &result.status,
                Some(result.exit_code),
                Some(result.output),
            )
            .await;

        if result.status == "failure" && !step.continue_on_error {
            final_status = "failure".to_string();
        }
    }

    // Report job completion
    let _ = client
        .complete_job(&job.job_id, &final_status, Some(outputs))
        .await;

    tracing::info!(
        job_id = %job.job_id,
        status = %final_status,
        "job completed"
    );

    final_status
}
