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
    /// Special action steps (checkout, cache, artifact) are handled without containers.
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

        // Handle built-in actions without spawning containers
        if let Some(ref action) = step.action
            && action != "run"
        {
            return self
                .execute_action(step, workspace, env, secrets, client, step_id, action)
                .await;
        }

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
            timeout_secs: timeout.as_secs(),
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

    /// Execute a built-in action step (checkout, cache, artifact).
    #[allow(clippy::too_many_arguments)]
    async fn execute_action(
        &self,
        step: &JobStepSpec,
        workspace: &Path,
        env: &HashMap<String, String>,
        _secrets: &HashMap<String, String>,
        client: &crate::exec::client::RunnerClient,
        step_id: &str,
        action: &str,
    ) -> anyhow::Result<StepResult> {
        let start = std::time::Instant::now();

        let result = match action {
            "checkout" => self.action_checkout(step, workspace, env).await,
            "cache" => self.action_cache(step, workspace).await,
            "artifact" => self.action_artifact(step, workspace).await,
            other => {
                // Let _ = client.update_step(step_id, "running", None, Some(format!("Unknown action: {other}"))).await;
                Err(anyhow::anyhow!("unknown action: {other}"))
            }
        };

        let (status, exit_code, output) = match result {
            Ok(output) => ("success", 0i32, output),
            Err(e) => ("failure", -1, format!("{e:#}")),
        };

        let _ = client
            .update_step(step_id, status, Some(exit_code), Some(output.clone()))
            .await;

        Ok(StepResult {
            step_name: step.name.clone(),
            status: status.to_string(),
            exit_code,
            output,
            duration: start.elapsed(),
        })
    }

    /// Checkout: clone or fetch the repo into the workspace.
    async fn action_checkout(
        &self,
        step: &JobStepSpec,
        workspace: &Path,
        env: &HashMap<String, String>,
    ) -> anyhow::Result<String> {
        let repo_url = env.get("CIVIT_REPO_URL").map(|s| s.as_str()).unwrap_or("");
        let commit_sha = env
            .get("CIVIT_COMMIT_SHA")
            .map(|s| s.as_str())
            .unwrap_or("HEAD");
        let fetch_depth = step
            .action_params
            .as_ref()
            .and_then(|p| p.get("fetch_depth"))
            .and_then(|v| v.as_u64())
            .unwrap_or(1);

        // Build git clone command
        let mut cmd_parts = vec!["git".to_string(), "clone".to_string()];

        if fetch_depth > 0 {
            cmd_parts.push("--depth".to_string());
            cmd_parts.push(fetch_depth.to_string());
        }

        // Check for submodules flag
        if step
            .action_params
            .as_ref()
            .and_then(|p| p.get("submodules"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            cmd_parts.push("--recurse-submodules".to_string());
        }

        cmd_parts.push(repo_url.to_string());
        cmd_parts.push(workspace.to_string_lossy().to_string());

        let output = tokio::process::Command::new(&cmd_parts[0])
            .args(&cmd_parts[1..])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // If clone failed, try fetch into existing dir
            if workspace.exists() {
                let fetch_output = tokio::process::Command::new("git")
                    .args([
                        "fetch",
                        "--depth",
                        &fetch_depth.to_string(),
                        "origin",
                        commit_sha,
                    ])
                    .current_dir(workspace)
                    .output()
                    .await?;
                if !fetch_output.status.success() {
                    return Err(anyhow::anyhow!(
                        "git fetch failed: {}",
                        String::from_utf8_lossy(&fetch_output.stderr)
                    ));
                }
                let checkout_output = tokio::process::Command::new("git")
                    .args(["checkout", "FETCH_HEAD"])
                    .current_dir(workspace)
                    .output()
                    .await?;
                if !checkout_output.status.success() {
                    return Err(anyhow::anyhow!(
                        "git checkout failed: {}",
                        String::from_utf8_lossy(&checkout_output.stderr)
                    ));
                }
                return Ok(format!("fetched {commit_sha} (depth={fetch_depth})"));
            }
            return Err(anyhow::anyhow!("git clone failed: {stderr}"));
        }

        Ok(format!("cloned {repo_url} (depth={fetch_depth})"))
    }

    /// Cache: upload or download cached files.
    async fn action_cache(&self, step: &JobStepSpec, workspace: &Path) -> anyhow::Result<String> {
        let cache_action = step
            .action_params
            .as_ref()
            .and_then(|p| p.get("action"))
            .and_then(|v| v.as_str())
            .unwrap_or("upload");

        let cache_key = step
            .action_params
            .as_ref()
            .and_then(|p| p.get("key"))
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let cache_dir = workspace.join(".civit-cache");

        match cache_action {
            "upload" => {
                // Save workspace state to cache directory (simplified: record timestamp)
                std::fs::create_dir_all(&cache_dir)?;
                std::fs::write(cache_dir.join("cache_key"), cache_key)?;
                std::fs::write(cache_dir.join("timestamp"), chrono::Utc::now().to_rfc3339())?;
                Ok(format!("cache uploaded: {cache_key}"))
            }
            "download" => {
                if cache_dir.join("cache_key").exists() {
                    let ts =
                        std::fs::read_to_string(cache_dir.join("timestamp")).unwrap_or_default();
                    Ok(format!("cache restored: {cache_key} (cached at {ts})"))
                } else {
                    Ok(format!("cache miss: {cache_key}"))
                }
            }
            _ => Err(anyhow::anyhow!("unknown cache action: {cache_action}")),
        }
    }

    /// Artifact: upload or download build artifacts.
    async fn action_artifact(
        &self,
        step: &JobStepSpec,
        workspace: &Path,
    ) -> anyhow::Result<String> {
        let artifact_action = step
            .action_params
            .as_ref()
            .and_then(|p| p.get("action"))
            .and_then(|v| v.as_str())
            .unwrap_or("upload");

        let artifact_name = step
            .action_params
            .as_ref()
            .and_then(|p| p.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("artifact");

        let artifact_dir = workspace.join(".civit-artifacts").join(artifact_name);

        match artifact_action {
            "upload" => {
                let paths: Vec<String> = step
                    .action_params
                    .as_ref()
                    .and_then(|p| p.get("path"))
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                std::fs::create_dir_all(&artifact_dir)?;

                // Copy specified paths to artifact directory
                let mut uploaded = Vec::new();
                for path in &paths {
                    let src = workspace.join(path);
                    if src.exists() {
                        let dest = artifact_dir.join(path);
                        if let Some(parent) = dest.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        if src.is_dir() {
                            copy_dir_recursive(&src, &dest)?;
                        } else {
                            std::fs::copy(&src, &dest)?;
                        }
                        uploaded.push(path.clone());
                    }
                }

                if uploaded.is_empty() && !paths.is_empty() {
                    let if_none = step
                        .action_params
                        .as_ref()
                        .and_then(|p| p.get("if_no_files_found"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("warn");
                    if if_none == "ignore" {
                        return Ok(format!(
                            "artifact '{artifact_name}': no files found (ignored)"
                        ));
                    }
                    return Ok(format!("artifact '{artifact_name}': no files found"));
                }

                Ok(format!(
                    "artifact uploaded: {artifact_name} ({} files)",
                    uploaded.len()
                ))
            }
            "download" => {
                if artifact_dir.exists() {
                    // Copy artifact files back to workspace
                    if artifact_dir.is_dir() {
                        copy_dir_recursive(&artifact_dir, workspace)?;
                    }
                    Ok(format!("artifact downloaded: {artifact_name}"))
                } else {
                    Ok(format!("artifact not found: {artifact_name}"))
                }
            }
            _ => Err(anyhow::anyhow!(
                "unknown artifact action: {artifact_action}"
            )),
        }
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

/// Recursively copy a directory.
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
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
