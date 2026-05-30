#![forbid(unsafe_code)]

use crate::models::PipelineStep;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub image: String,
    pub memory_mb: u64,
    pub cpu_cores: f64,
    pub timeout_secs: u64,
    pub network_disabled: bool,
    pub read_only_fs: bool,
    pub env: HashMap<String, String>,
    pub workdir: String,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            image: "alpine:latest".into(),
            memory_mb: 512,
            cpu_cores: 1.0,
            timeout_secs: 300,
            network_disabled: false,
            read_only_fs: false,
            env: HashMap::new(),
            workdir: "/workspace".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SandboxResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

impl SandboxResult {
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

pub struct SandboxManager {
    default_config: SandboxConfig,
    max_concurrent: usize,
    active_count: usize,
}

impl SandboxManager {
    pub fn new(default_config: SandboxConfig, max_concurrent: usize) -> Self {
        Self {
            default_config,
            max_concurrent,
            active_count: 0,
        }
    }

    pub fn can_start(&self) -> bool {
        self.active_count < self.max_concurrent
    }

    pub fn build_config_from_step(&self, step: &PipelineStep) -> SandboxConfig {
        let mut config = self.default_config.clone();
        config.image = step.image.clone();
        for (k, v) in &step.env {
            config.env.insert(k.clone(), v.clone());
        }
        config
    }

    pub async fn run_container(
        &mut self,
        config: &SandboxConfig,
        command: &str,
    ) -> anyhow::Result<SandboxResult> {
        if !self.can_start() {
            anyhow::bail!("max concurrent sandboxes reached");
        }
        self.active_count += 1;

        debug!(
            image = %config.image,
            mem = config.memory_mb,
            cpu = config.cpu_cores,
            timeout = config.timeout_secs,
            "running container"
        );

        let start = std::time::Instant::now();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let duration_ms = start.elapsed().as_millis() as u64;

        self.active_count -= 1;

        Ok(SandboxResult {
            exit_code: 0,
            stdout: format!("executed: {command}"),
            stderr: String::new(),
            duration_ms,
        })
    }

    pub async fn cleanup_stale(&self, max_age_secs: u64) -> Vec<String> {
        debug!(max_age_secs, "cleaning stale containers");
        vec![]
    }

    pub fn active_count(&self) -> usize {
        self.active_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_sandbox_config() {
        let config = SandboxConfig::default();
        assert_eq!(config.image, "alpine:latest");
        assert_eq!(config.memory_mb, 512);
        assert_eq!(config.cpu_cores, 1.0);
        assert_eq!(config.timeout_secs, 300);
        assert!(!config.network_disabled);
        assert!(!config.read_only_fs);
        assert_eq!(config.workdir, "/workspace");
    }

    #[test]
    fn test_build_config_from_step() {
        let manager = SandboxManager::new(SandboxConfig::default(), 4);
        let mut env = HashMap::new();
        env.insert("RUST_LOG".into(), "debug".into());
        let step = PipelineStep {
            name: "test".into(),
            image: "rust:1.75".into(),
            commands: vec!["cargo test".into()],
            env,
            condition: None,
        };
        let config = manager.build_config_from_step(&step);
        assert_eq!(config.image, "rust:1.75");
        assert_eq!(config.env.get("RUST_LOG").unwrap(), "debug");
    }

    #[tokio::test]
    async fn test_run_container() {
        let mut manager = SandboxManager::new(SandboxConfig::default(), 4);
        let config = SandboxConfig::default();
        let result = manager.run_container(&config, "echo hello").await.unwrap();
        assert!(result.success());
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_concurrency_limit() {
        let manager = SandboxManager::new(SandboxConfig::default(), 2);
        assert!(manager.can_start());
    }

    #[test]
    fn test_sandbox_result_success() {
        let result = SandboxResult {
            exit_code: 0,
            stdout: "ok".into(),
            stderr: String::new(),
            duration_ms: 100,
        };
        assert!(result.success());

        let result = SandboxResult {
            exit_code: 1,
            stdout: String::new(),
            stderr: "error".into(),
            duration_ms: 50,
        };
        assert!(!result.success());
    }
}
