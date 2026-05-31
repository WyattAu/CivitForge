#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    #[serde(default = "default_image")]
    pub image: String,
    #[serde(default = "default_cpu_limit")]
    pub cpu_limit_milli: u32,
    #[serde(default = "default_memory_limit")]
    pub memory_limit_bytes: u64,
    #[serde(default = "default_timeout")]
    pub timeout: Duration,
    #[serde(default = "default_network")]
    pub network: String,
    #[serde(default = "default_true")]
    pub rootless: bool,
    #[serde(default = "default_workspace")]
    pub workspace_path: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

fn default_image() -> String {
    "rust:1.88-slim".into()
}
fn default_cpu_limit() -> u32 {
    2000
}
fn default_memory_limit() -> u64 {
    4 * 1024 * 1024 * 1024
}
fn default_timeout() -> Duration {
    Duration::from_secs(3600)
}
fn default_network() -> String {
    "none".into()
}
fn default_true() -> bool {
    true
}
fn default_workspace() -> String {
    "/workspace".into()
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            image: default_image(),
            cpu_limit_milli: default_cpu_limit(),
            memory_limit_bytes: default_memory_limit(),
            timeout: default_timeout(),
            network: default_network(),
            rootless: default_true(),
            workspace_path: default_workspace(),
            env: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecution {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub workdir: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub continue_on_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub name: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub steps: Vec<StepResult>,
    pub total_duration: Duration,
    pub success: bool,
}

pub trait SandboxBackend: Send + Sync {
    fn execute_step(
        &self,
        step: &StepExecution,
        config: &SandboxConfig,
    ) -> Result<StepResult, String>;

    fn is_available(&self) -> bool;

    fn backend_name(&self) -> &str;
}

pub struct LocalProcessSandbox {
    workspace: PathBuf,
}

impl LocalProcessSandbox {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

impl SandboxBackend for LocalProcessSandbox {
    fn execute_step(
        &self,
        step: &StepExecution,
        _config: &SandboxConfig,
    ) -> Result<StepResult, String> {
        use std::process::Command;

        let start = std::time::Instant::now();

        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(&step.command);
        cmd.current_dir(&self.workspace);
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        for (k, v) in &step.env {
            cmd.env(k, v);
        }

        let output = cmd
            .output()
            .map_err(|e| format!("failed to execute step '{}': {}", step.name, e))?;

        let duration = start.elapsed();
        let success = output.status.success();
        let exit_code = output.status.code().unwrap_or(-1);

        Ok(StepResult {
            name: step.name.clone(),
            exit_code,
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            duration,
            success,
        })
    }

    fn is_available(&self) -> bool {
        true
    }

    fn backend_name(&self) -> &str {
        "local-process"
    }
}

pub struct PipelineExecutor {
    backend: Box<dyn SandboxBackend>,
}

impl PipelineExecutor {
    pub fn new(backend: Box<dyn SandboxBackend>) -> Self {
        Self { backend }
    }

    pub fn execute(
        &self,
        steps: &[StepExecution],
        config: &SandboxConfig,
    ) -> Result<PipelineResult, String> {
        let start = std::time::Instant::now();
        let mut results = Vec::new();
        let mut pipeline_success = true;

        for step in steps {
            let step_config = Self::merge_env(config, step);
            match self.backend.execute_step(step, &step_config) {
                Ok(result) => {
                    if !result.success && !step.continue_on_error {
                        pipeline_success = false;
                        results.push(result);
                        break;
                    }
                    results.push(result);
                }
                Err(e) => {
                    pipeline_success = false;
                    results.push(StepResult {
                        name: step.name.clone(),
                        exit_code: -1,
                        stdout: String::new(),
                        stderr: e.clone(),
                        duration: Duration::ZERO,
                        success: false,
                    });
                    break;
                }
            }
        }

        Ok(PipelineResult {
            steps: results,
            total_duration: start.elapsed(),
            success: pipeline_success,
        })
    }

    fn merge_env(config: &SandboxConfig, step: &StepExecution) -> SandboxConfig {
        let mut merged = config.clone();
        for (k, v) in &step.env {
            merged.env.insert(k.clone(), v.clone());
        }
        merged
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sandbox() -> (LocalProcessSandbox, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("tempdir");
        let sandbox = LocalProcessSandbox::new(dir.path().to_path_buf());
        (sandbox, dir)
    }

    fn default_config() -> SandboxConfig {
        SandboxConfig::default()
    }

    #[test]
    fn test_local_sandbox_echo() {
        let (sandbox, _dir) = make_sandbox();
        let step = StepExecution {
            name: "echo-test".into(),
            command: "echo hello".into(),
            workdir: None,
            env: HashMap::new(),
            continue_on_error: false,
        };
        let result = sandbox.execute_step(&step, &default_config()).unwrap();
        assert!(result.success);
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello"));
        assert_eq!(result.name, "echo-test");
    }

    #[test]
    fn test_local_sandbox_failing_command() {
        let (sandbox, _dir) = make_sandbox();
        let step = StepExecution {
            name: "fail-test".into(),
            command: "exit 42".into(),
            workdir: None,
            env: HashMap::new(),
            continue_on_error: false,
        };
        let result = sandbox.execute_step(&step, &default_config()).unwrap();
        assert!(!result.success);
        assert_eq!(result.exit_code, 42);
    }

    #[test]
    fn test_merge_env() {
        let mut config = SandboxConfig::default();
        config.env.insert("BASE".to_string(), "val1".to_string());

        let step = StepExecution {
            name: "env-test".into(),
            command: "echo $BASE $OVERLAY".into(),
            workdir: None,
            env: {
                let mut m = HashMap::new();
                m.insert("OVERLAY".to_string(), "val2".to_string());
                m
            },
            continue_on_error: false,
        };

        let merged = PipelineExecutor::merge_env(&config, &step);
        assert_eq!(merged.env.get("BASE").unwrap(), "val1");
        assert_eq!(merged.env.get("OVERLAY").unwrap(), "val2");
    }

    #[test]
    fn test_pipeline_sequential_execution() {
        let (sandbox, _dir) = make_sandbox();
        let executor = PipelineExecutor::new(Box::new(sandbox));
        let steps = vec![
            StepExecution {
                name: "step1".into(),
                command: "echo first".into(),
                workdir: None,
                env: HashMap::new(),
                continue_on_error: false,
            },
            StepExecution {
                name: "step2".into(),
                command: "echo second".into(),
                workdir: None,
                env: HashMap::new(),
                continue_on_error: false,
            },
        ];
        let result = executor.execute(&steps, &default_config()).unwrap();
        assert!(result.success);
        assert_eq!(result.steps.len(), 2);
        assert!(result.steps[0].stdout.contains("first"));
        assert!(result.steps[1].stdout.contains("second"));
    }

    #[test]
    fn test_pipeline_stop_on_failure() {
        let (sandbox, _dir) = make_sandbox();
        let executor = PipelineExecutor::new(Box::new(sandbox));
        let steps = vec![
            StepExecution {
                name: "ok".into(),
                command: "echo ok".into(),
                workdir: None,
                env: HashMap::new(),
                continue_on_error: false,
            },
            StepExecution {
                name: "fail".into(),
                command: "exit 1".into(),
                workdir: None,
                env: HashMap::new(),
                continue_on_error: false,
            },
            StepExecution {
                name: "never".into(),
                command: "echo never".into(),
                workdir: None,
                env: HashMap::new(),
                continue_on_error: false,
            },
        ];
        let result = executor.execute(&steps, &default_config()).unwrap();
        assert!(!result.success);
        assert_eq!(result.steps.len(), 2);
        assert!(result.steps[0].success);
        assert!(!result.steps[1].success);
    }

    #[test]
    fn test_pipeline_continue_on_failure() {
        let (sandbox, _dir) = make_sandbox();
        let executor = PipelineExecutor::new(Box::new(sandbox));
        let steps = vec![
            StepExecution {
                name: "ok".into(),
                command: "echo ok".into(),
                workdir: None,
                env: HashMap::new(),
                continue_on_error: false,
            },
            StepExecution {
                name: "fail".into(),
                command: "exit 1".into(),
                workdir: None,
                env: HashMap::new(),
                continue_on_error: true,
            },
            StepExecution {
                name: "after".into(),
                command: "echo after".into(),
                workdir: None,
                env: HashMap::new(),
                continue_on_error: false,
            },
        ];
        let result = executor.execute(&steps, &default_config()).unwrap();
        assert!(result.success);
        assert_eq!(result.steps.len(), 3);
        assert!(result.steps[0].success);
        assert!(!result.steps[1].success);
        assert!(result.steps[2].success);
    }

    #[test]
    fn test_sandbox_config_defaults() {
        let config = SandboxConfig::default();
        assert_eq!(config.image, "rust:1.88-slim");
        assert_eq!(config.cpu_limit_milli, 2000);
        assert_eq!(config.memory_limit_bytes, 4 * 1024 * 1024 * 1024);
        assert_eq!(config.timeout, Duration::from_secs(3600));
        assert_eq!(config.network, "none");
        assert!(config.rootless);
        assert_eq!(config.workspace_path, "/workspace");
        assert!(config.env.is_empty());
    }
}
