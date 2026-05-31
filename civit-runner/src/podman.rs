#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodmanConfig {
    pub runtime: String,
    pub socket_path: String,
    pub default_image: String,
    pub default_memory_mb: u64,
    pub default_cpu_quota: i64,
    pub network_mode: String,
    pub user_ns_mode: String,
}

impl Default for PodmanConfig {
    fn default() -> Self {
        Self {
            runtime: "podman".into(),
            socket_path: "/run/podman/podman.sock".into(),
            default_image: "alpine:latest".into(),
            default_memory_mb: 512,
            default_cpu_quota: 100_000,
            network_mode: "slirp4netns".into(),
            user_ns_mode: "keep-id".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContainerStatus {
    Created,
    Running,
    Exited,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodmanContainer {
    pub id: String,
    pub image: String,
    pub status: ContainerStatus,
    pub exit_code: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct PodmanRunSpec {
    pub image: String,
    pub command: Vec<String>,
    pub env: HashMap<String, String>,
    pub memory_mb: u64,
    pub cpu_quota: i64,
    pub network_disabled: bool,
    pub read_only_fs: bool,
    pub workdir: String,
    pub timeout_secs: u64,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ExecResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkPolicy {
    AllowAll,
    DenyAll,
    AllowOnly(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermeticConfig {
    pub allowed_images: Vec<String>,
    pub allowed_registries: Vec<String>,
    pub network_policy: NetworkPolicy,
    pub read_only_root_fs: bool,
    pub no_new_privileges: bool,
}

impl Default for HermeticConfig {
    fn default() -> Self {
        Self {
            allowed_images: vec![],
            allowed_registries: vec!["docker.io".into()],
            network_policy: NetworkPolicy::DenyAll,
            read_only_root_fs: true,
            no_new_privileges: true,
        }
    }
}

impl HermeticConfig {
    pub fn is_image_allowed(&self, image: &str) -> bool {
        if self
            .allowed_images
            .iter()
            .any(|a| image.starts_with(a.as_str()))
        {
            return true;
        }
        let registry = image.split('/').next().unwrap_or(image);
        self.allowed_registries
            .iter()
            .any(|r| registry == r.as_str())
    }
}

pub struct PodmanService {
    pub config: PodmanConfig,
}

impl PodmanService {
    pub fn new() -> Self {
        Self {
            config: PodmanConfig::default(),
        }
    }

    pub fn with_config(config: PodmanConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, spec: &PodmanRunSpec) -> anyhow::Result<PodmanContainer> {
        let container_id = uuid::Uuid::new_v4().to_string();
        Ok(PodmanContainer {
            id: container_id,
            image: spec.image.clone(),
            status: ContainerStatus::Created,
            exit_code: None,
            created_at: Utc::now(),
        })
    }

    pub fn exec(&self, _container_id: &str, command: &str) -> anyhow::Result<ExecResult> {
        Ok(ExecResult {
            exit_code: 0,
            stdout: format!("executed: {command}"),
            stderr: String::new(),
        })
    }

    pub fn inspect(&self, container_id: &str) -> anyhow::Result<PodmanContainer> {
        Ok(PodmanContainer {
            id: container_id.into(),
            image: self.config.default_image.clone(),
            status: ContainerStatus::Exited,
            exit_code: Some(0),
            created_at: Utc::now(),
        })
    }

    pub fn rm(&self, _container_id: &str) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn logs(&self, _container_id: &str, tail: Option<usize>) -> anyhow::Result<String> {
        let limit = tail.unwrap_or(100);
        let lines: Vec<String> = (0..limit).map(|i| format!("log line {i}")).collect();
        Ok(lines.join("\n"))
    }

    pub fn list(&self) -> anyhow::Result<Vec<PodmanContainer>> {
        Ok(vec![])
    }

    pub fn health(&self) -> anyhow::Result<bool> {
        Ok(true)
    }
}

impl Default for PodmanService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_podman_config_defaults() {
        let cfg = PodmanConfig::default();
        assert_eq!(cfg.runtime, "podman");
        assert_eq!(cfg.socket_path, "/run/podman/podman.sock");
        assert_eq!(cfg.default_image, "alpine:latest");
        assert_eq!(cfg.default_memory_mb, 512);
        assert_eq!(cfg.default_cpu_quota, 100_000);
        assert_eq!(cfg.network_mode, "slirp4netns");
        assert_eq!(cfg.user_ns_mode, "keep-id");
    }

    #[test]
    fn test_podman_config_serialization() {
        let cfg = PodmanConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let de: PodmanConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(de.runtime, cfg.runtime);
    }

    #[test]
    fn test_run_container() {
        let svc = PodmanService::new();
        let spec = PodmanRunSpec {
            image: "alpine:latest".into(),
            command: vec!["echo".into(), "hello".into()],
            env: HashMap::new(),
            memory_mb: 256,
            cpu_quota: 50_000,
            network_disabled: true,
            read_only_fs: true,
            workdir: "/workspace".into(),
            timeout_secs: 60,
            labels: HashMap::new(),
        };
        let container = svc.run(&spec).unwrap();
        assert_eq!(container.status, ContainerStatus::Created);
        assert_eq!(container.image, "alpine:latest");
        assert!(container.exit_code.is_none());
    }

    #[test]
    fn test_exec_command() {
        let svc = PodmanService::new();
        let result = svc.exec("test-id", "ls /").unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("executed: ls /"));
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn test_inspect_container() {
        let svc = PodmanService::new();
        let container = svc.inspect("some-id").unwrap();
        assert_eq!(container.status, ContainerStatus::Exited);
        assert_eq!(container.exit_code, Some(0));
    }

    #[test]
    fn test_rm_container() {
        let svc = PodmanService::new();
        assert!(svc.rm("any-id").is_ok());
    }

    #[test]
    fn test_logs_with_tail() {
        let svc = PodmanService::new();
        let logs = svc.logs("id", Some(5)).unwrap();
        let lines: Vec<&str> = logs.lines().collect();
        assert_eq!(lines.len(), 5);
    }

    #[test]
    fn test_logs_default_tail() {
        let svc = PodmanService::new();
        let logs = svc.logs("id", None).unwrap();
        let lines: Vec<&str> = logs.lines().collect();
        assert_eq!(lines.len(), 100);
    }

    #[test]
    fn test_health_check() {
        let svc = PodmanService::new();
        assert!(svc.health().unwrap());
    }

    #[test]
    fn test_list_empty() {
        let svc = PodmanService::new();
        let containers = svc.list().unwrap();
        assert!(containers.is_empty());
    }

    #[test]
    fn test_hermetic_config_defaults() {
        let cfg = HermeticConfig::default();
        assert!(cfg.read_only_root_fs);
        assert!(cfg.no_new_privileges);
        assert_eq!(cfg.network_policy, NetworkPolicy::DenyAll);
        assert!(cfg.allowed_registries.contains(&"docker.io".into()));
    }

    #[test]
    fn test_hermetic_image_allowed() {
        let cfg = HermeticConfig::default();
        assert!(cfg.is_image_allowed("docker.io/library/alpine:latest"));
        assert!(cfg.is_image_allowed("docker.io/rust:1.75"));
    }

    #[test]
    fn test_hermetic_image_blocked() {
        let cfg = HermeticConfig::default();
        assert!(!cfg.is_image_allowed("ghcr.io/some/image:latest"));
    }

    #[test]
    fn test_hermetic_custom_allowed_images() {
        let cfg = HermeticConfig {
            allowed_images: vec!["ghcr.io/civitforge/".into()],
            allowed_registries: vec![],
            network_policy: NetworkPolicy::AllowAll,
            read_only_root_fs: false,
            no_new_privileges: false,
        };
        assert!(cfg.is_image_allowed("ghcr.io/civitforge/builder:latest"));
    }

    #[test]
    fn test_network_policy_serialization() {
        let np = NetworkPolicy::AllowOnly(vec!["10.0.0.0/8".into(), "192.168.0.0/16".into()]);
        let json = serde_json::to_string(&np).unwrap();
        let de: NetworkPolicy = serde_json::from_str(&json).unwrap();
        match de {
            NetworkPolicy::AllowOnly(ref cidrs) => {
                assert_eq!(cidrs.len(), 2);
            }
            _ => panic!("expected AllowOnly"),
        }
    }

    #[test]
    fn test_container_status_serialization() {
        let statuses = vec![
            ContainerStatus::Created,
            ContainerStatus::Running,
            ContainerStatus::Exited,
            ContainerStatus::Error,
        ];
        for s in statuses {
            let json = serde_json::to_string(&s).unwrap();
            let de: ContainerStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, de);
        }
    }
}
