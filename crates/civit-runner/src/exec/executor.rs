//! Step executor — runs pipeline steps in Podman containers.
//!
//! Each step gets its own container with workspace mounted, env vars injected,
//! and a timeout enforced. Output is captured and returned.

#![forbid(unsafe_code)]

use crate::exec::client::JobStepSpec;
use crate::podman::{PodmanRunSpec, PodmanService, VolumeMount};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

/// Result of executing a single step.
#[derive(Debug, Clone)]
pub struct StepResult {
    pub step_name: String,
    pub status: String,
    pub exit_code: i32,
    pub output: String,
    pub duration: std::time::Duration,
}

/// Executes pipeline steps using Podman.
pub struct StepExecutor {
    podman: PodmanService,
    default_image: String,
    default_timeout: Duration,
    default_memory_mb: u64,
}

impl StepExecutor {
    /// Create a new step executor.
    pub fn new(
        podman: PodmanService,
        default_image: &str,
        default_timeout_secs: u64,
        default_memory_mb: u64,
    ) -> Self {
        Self {
            podman,
            default_image: default_image.to_string(),
            default_timeout: Duration::from_secs(default_timeout_secs),
            default_memory_mb,
        }
    }

    /// Execute a single step in a container.
    ///
    /// The workspace directory is mounted at `/workspace` inside the container.
    pub async fn execute_step(
        &self,
        step: &JobStepSpec,
        workspace: &Path,
        env: &HashMap<String, String>,
        secrets: &HashMap<String, String>,
        client: &crate::exec::client::RunnerClient,
        step_id: &str,
    ) -> anyhow::Result<StepResult> {
        let start = std::time::Instant::now();

        // Determine the image (step-level overrides job-level)
        let image = step.image.as_deref().unwrap_or(&self.default_image);

        // Build command
        let commands = match &step.commands {
            Some(cmds) if !cmds.is_empty() => cmds.join("\n"),
            _ => "true".to_string(), // no-op if no commands
        };

        // Merge env: step env > secrets > job env
        let mut merged_env = env.clone();
        merged_env.extend(secrets.clone());
        if let serde_json::Value::Object(map) = &step.env {
            for (k, v) in map {
                if let serde_json::Value::String(s) = v {
                    merged_env.insert(k.clone(), s.clone());
                }
            }
        }

        // Parse timeout
        let timeout = parse_timeout(step.timeout.as_deref()).unwrap_or(self.default_timeout);

        // Determine workdir (relative to workspace)
        let workdir = if step.workdir.is_empty() {
            "/workspace".to_string()
        } else if step.workdir.starts_with('/') {
            step.workdir.clone()
        } else {
            format!("/workspace/{}", step.workdir)
        };

        // Build Podman spec
        let spec = PodmanRunSpec {
            image: image.to_string(),
            command: vec!["sh".to_string(), "-c".to_string(), commands],
            env: merged_env,
            memory_mb: self.default_memory_mb,
            cpu_quota: 100_000,
            network_disabled: true, // Isolated by default
            read_only_fs: false,    // Need writable workspace
            workdir,
            timeout_secs: timeout.as_secs() as u64,
            labels: HashMap::from([
                ("civit.step".to_string(), step.name.clone()),
                ("civit.type".to_string(), "pipeline".to_string()),
            ]),
            volumes: vec![VolumeMount {
                host_path: workspace.to_string_lossy().to_string(),
                container_path: "/workspace".to_string(),
                options: "rw,z".to_string(),
            }],
        };

        tracing::info!(
            step = %step.name,
            image = %image,
            "executing step"
        );

        // Run with timeout
        let result =
            tokio::time::timeout(timeout, self.run_container(&spec, client, step_id)).await;

        let (exit_code, output) = match result {
            Ok(inner) => match inner {
                Ok(pair) => pair,
                Err(e) => {
                    return Ok(StepResult {
                        step_name: step.name.clone(),
                        status: "failure".to_string(),
                        exit_code: -1,
                        output: format!("Container error: {e}"),
                        duration: start.elapsed(),
                    });
                }
            },
            Err(_) => {
                // Timeout
                return Ok(StepResult {
                    step_name: step.name.clone(),
                    status: "failure".to_string(),
                    exit_code: -1,
                    output: format!("Step timed out after {timeout:?}"),
                    duration: start.elapsed(),
                });
            }
        };

        let status = if exit_code == 0 { "success" } else { "failure" };

        tracing::info!(
            step = %step.name,
            status = %status,
            exit_code,
            duration = ?start.elapsed(),
            "step completed"
        );

        Ok(StepResult {
            step_name: step.name.clone(),
            status: status.to_string(),
            exit_code,
            output,
            duration: start.elapsed(),
        })
    }

    /// Run a container, stream logs to API, return exit code + final output.
    async fn run_container(
        &self,
        spec: &PodmanRunSpec,
        client: &crate::exec::client::RunnerClient,
        step_id: &str,
    ) -> anyhow::Result<(i32, String)> {
        // Create and start container
        let container = self.podman.run(spec).await?;

        // Stream logs periodically while container is running
        let container_id = container.id.clone();
        let mut final_output = String::new();
        let poll_interval = Duration::from_secs(2);
        let max_wait = Duration::from_secs(spec.timeout_secs);
        let deadline = std::time::Instant::now() + max_wait;

        loop {
            if std::time::Instant::now() >= deadline {
                let _ = self.podman.stop(&container_id).await;
                let _ = self.podman.rm(&container_id).await;
                return Ok((-1, final_output));
            }

            // Check if container has finished
            match self.podman.inspect(&container_id).await {
                Ok(inspected) => {
                    // Stream available logs
                    if let Ok(logs) = self.podman.logs(&container_id, None).await {
                        let new_output = if final_output.len() <= logs.len()
                            && logs.starts_with(&final_output)
                        {
                            &logs[final_output.len()..]
                        } else {
                            &logs
                        };
                        if !new_output.is_empty() {
                            final_output = logs.clone();
                            // Stream incremental output to API
                            let _ = client
                                .update_step(step_id, "running", None, Some(final_output.clone()))
                                .await;
                        }
                    }

                    if inspected.status != crate::podman::ContainerStatus::Running {
                        let exit_code = inspected.exit_code.unwrap_or(-1);
                        // Get final logs
                        if let Ok(final_logs) = self.podman.logs(&container_id, None).await {
                            final_output = final_logs;
                        }
                        let _ = self.podman.rm(&container_id).await;
                        return Ok((exit_code, final_output));
                    }
                }
                Err(_) => {
                    break;
                }
            }

            tokio::time::sleep(poll_interval).await;
        }

        // Fallback: get final logs and clean up
        let _ = self.podman.rm(&container_id).await;
        Ok((-1, final_output))
    }
}

/// Parse a timeout string like "30m", "2h", "300s" into a Duration.
fn parse_timeout(s: Option<&str>) -> Option<Duration> {
    let s = s?.trim();

    if let Some(n) = s.strip_suffix('s') {
        let secs: u64 = n.trim().parse().ok()?;
        return Some(Duration::from_secs(secs));
    }
    if let Some(n) = s.strip_suffix('m') {
        let mins: u64 = n.trim().parse().ok()?;
        return Some(Duration::from_secs(mins * 60));
    }
    if let Some(n) = s.strip_suffix('h') {
        let hours: u64 = n.trim().parse().ok()?;
        return Some(Duration::from_secs(hours * 3600));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timeout_seconds() {
        assert_eq!(parse_timeout(Some("300s")), Some(Duration::from_secs(300)));
    }

    #[test]
    fn test_parse_timeout_minutes() {
        assert_eq!(parse_timeout(Some("30m")), Some(Duration::from_secs(1800)));
    }

    #[test]
    fn test_parse_timeout_hours() {
        assert_eq!(parse_timeout(Some("2h")), Some(Duration::from_secs(7200)));
    }

    #[test]
    fn test_parse_timeout_none() {
        assert_eq!(parse_timeout(None), None);
    }

    #[test]
    fn test_parse_timeout_invalid() {
        assert_eq!(parse_timeout(Some("abc")), None);
    }

    #[test]
    fn test_parse_timeout_empty() {
        assert_eq!(parse_timeout(Some("")), None);
    }

    #[test]
    fn test_step_result_fields() {
        let result = StepResult {
            step_name: "build".to_string(),
            status: "success".to_string(),
            exit_code: 0,
            output: "done".to_string(),
            duration: Duration::from_millis(500),
        };
        assert_eq!(result.step_name, "build");
        assert_eq!(result.status, "success");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_step_executor_constructor() {
        let podman = PodmanService::new();
        let executor = StepExecutor::new(podman, "alpine:3.18", 300, 512);
        assert_eq!(executor.default_image, "alpine:3.18");
        assert_eq!(executor.default_timeout, Duration::from_secs(300));
    }
}
