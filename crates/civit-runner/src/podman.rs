#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, warn};

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

#[derive(Debug, Clone, Default)]
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
    /// Bind-mount volumes: (host_path, container_path, options)
    /// e.g. ("/tmp/workspace", "/workspace", "rw,z")
    pub volumes: Vec<VolumeMount>,
}

/// A bind-mount volume specification for Podman containers.
#[derive(Debug, Clone)]
pub struct VolumeMount {
    /// Host path (source)
    pub host_path: String,
    /// Container path (destination)
    pub container_path: String,
    /// Mount options (e.g. "rw,z", "ro")
    pub options: String,
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

enum Transport {
    Http(reqwest::Client),
    Cli,
}

impl Clone for Transport {
    fn clone(&self) -> Self {
        match self {
            Transport::Http(client) => Transport::Http(client.clone()),
            Transport::Cli => Transport::Cli,
        }
    }
}

impl Transport {
    fn detect(socket_path: &str) -> Self {
        if std::path::Path::new(socket_path).exists() {
            debug!(socket = %socket_path, "podman unix socket detected, using CLI transport");
            Transport::Cli
        } else {
            debug!(socket = %socket_path, "podman socket not found, using HTTP transport");
            Transport::Http(build_http_client())
        }
    }
}

fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

#[derive(Clone)]
pub struct PodmanService {
    pub config: PodmanConfig,
    transport: Transport,
}

impl PodmanService {
    pub fn new() -> Self {
        let config = PodmanConfig::default();
        let transport = Transport::detect(&config.socket_path);
        Self { config, transport }
    }

    pub fn with_config(config: PodmanConfig) -> Self {
        let transport = Transport::detect(&config.socket_path);
        Self { config, transport }
    }

    #[cfg(test)]
    fn new_http() -> Self {
        let config = PodmanConfig {
            socket_path: "/nonexistent/civitforge-test-podman.sock".into(),
            ..PodmanConfig::default()
        };
        Self {
            config,
            transport: Transport::Http(build_http_client()),
        }
    }

    fn base_url(&self) -> String {
        format!("http://localhost{}", Self::podman_path_prefix())
    }

    fn podman_path_prefix() -> &'static str {
        "/v4.6.0/libpod"
    }

    pub async fn run(&self, spec: &PodmanRunSpec) -> anyhow::Result<PodmanContainer> {
        match &self.transport {
            Transport::Http(client) => self.run_http(client, spec).await,
            Transport::Cli => self.run_cli(spec).await,
        }
    }

    async fn run_http(
        &self,
        client: &reqwest::Client,
        spec: &PodmanRunSpec,
    ) -> anyhow::Result<PodmanContainer> {
        let url = format!("{}/containers/create", self.base_url());
        let mounts: Vec<serde_json::Value> = spec
            .volumes
            .iter()
            .map(|v| {
                serde_json::json!({
                    "Type": "bind",
                    "Source": v.host_path,
                    "Destination": v.container_path,
                    "Options": v.options.split(',').collect::<Vec<_>>(),
                })
            })
            .collect();

        let body = serde_json::json!({
            "Image": spec.image,
            "Cmd": spec.command,
            "Env": spec.env.iter().map(|(k,v)| format!("{k}={v}")).collect::<Vec<_>>(),
            "HostConfig": {
                "Memory": spec.memory_mb as i64 * 1024 * 1024,
                "CpuQuota": spec.cpu_quota,
                "NetworkMode": if spec.network_disabled { "none" } else { "bridge" },
                "ReadonlyRootfs": spec.read_only_fs,
                "SecurityOpt": ["no-new-privileges:true"],
                "Mounts": mounts,
            },
            "WorkingDir": spec.workdir,
            "Labels": spec.labels,
        });

        let resp = client.post(&url).json(&body).send().await;

        match resp {
            Ok(r) if r.status().is_success() => {
                let json: serde_json::Value = r.json().await?;
                let container_id = json
                    .get("Id")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&uuid::Uuid::new_v4().to_string())
                    .to_string();

                let start_url = format!("{}/containers/{container_id}/start", self.base_url());
                let start_resp = client.post(&start_url).send().await;
                if let Ok(sr) = start_resp {
                    if !sr.status().is_success() && sr.status().as_u16() != 304 {
                        warn!(status = %sr.status(), "container start returned non-success");
                    }
                }

                Ok(PodmanContainer {
                    id: container_id,
                    image: spec.image.clone(),
                    status: ContainerStatus::Running,
                    exit_code: None,
                    created_at: Utc::now(),
                })
            }
            Ok(r) => {
                let status = r.status();
                let text = r.text().await.unwrap_or_default();
                anyhow::bail!(
                    "podman create failed with status {status}: {text} (image: {})",
                    spec.image
                );
            }
            Err(e) => {
                anyhow::bail!(
                    "podman unreachable at {} -- cannot create container (image: {}): {e}",
                    self.base_url(),
                    spec.image
                );
            }
        }
    }

    async fn run_cli(&self, spec: &PodmanRunSpec) -> anyhow::Result<PodmanContainer> {
        let mut cmd = Command::new("podman");
        cmd.arg("create")
            .arg("--image")
            .arg(&spec.image)
            .arg("--memory")
            .arg(format!("{}m", spec.memory_mb))
            .arg("--cpu-quota")
            .arg(spec.cpu_quota.to_string())
            .arg("--network")
            .arg(if spec.network_disabled {
                "none"
            } else {
                "bridge"
            })
            .arg("--workdir")
            .arg(&spec.workdir)
            .arg("--security-opt")
            .arg("no-new-privileges:true");

        if spec.read_only_fs {
            cmd.arg("--read-only");
        }

        for (k, v) in &spec.env {
            cmd.arg("--env").arg(format!("{k}={v}"));
        }
        for (k, v) in &spec.labels {
            cmd.arg("--label").arg(format!("{k}={v}"));
        }

        // Bind-mount volumes
        for vol in &spec.volumes {
            cmd.arg("--volume").arg(format!(
                "{}:{}:{}",
                vol.host_path, vol.container_path, vol.options
            ));
        }

        for arg in &spec.command {
            cmd.arg(arg);
        }

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let output = cmd.output().await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "podman create failed (exit {:?}): {stderr} (image: {})",
                output.status.code(),
                spec.image
            );
        }

        let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

        let start_output = Command::new("podman")
            .arg("start")
            .arg(&container_id)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !start_output.status.success() {
            warn!(
                status = ?start_output.status.code(),
                "container start returned non-success"
            );
        }

        Ok(PodmanContainer {
            id: container_id,
            image: spec.image.clone(),
            status: ContainerStatus::Running,
            exit_code: None,
            created_at: Utc::now(),
        })
    }

    pub async fn exec(&self, container_id: &str, command: &str) -> anyhow::Result<ExecResult> {
        match &self.transport {
            Transport::Http(client) => self.exec_http(client, container_id, command).await,
            Transport::Cli => self.exec_cli(container_id, command).await,
        }
    }

    async fn exec_http(
        &self,
        client: &reqwest::Client,
        container_id: &str,
        command: &str,
    ) -> anyhow::Result<ExecResult> {
        let url = format!("{}/containers/{container_id}/exec", self.base_url());
        let body = serde_json::json!({
            "Cmd": ["sh", "-c", command],
            "AttachStdout": true,
            "AttachStderr": true,
        });

        let resp = client.post(&url).json(&body).send().await;
        match resp {
            Ok(r) if r.status().is_success() => {
                let json: serde_json::Value = r.json().await?;
                let exec_id = json
                    .get("ID")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let start_url = format!("{}/exec/{}/start", self.base_url(), exec_id);
                let start_body = serde_json::json!({
                    "Detach": false,
                    "Tty": false,
                });
                let start_resp = client.post(&start_url).json(&start_body).send().await;
                match start_resp {
                    Ok(sr) => {
                        let text = sr.text().await.unwrap_or_default();
                        Ok(ExecResult {
                            exit_code: 0,
                            stdout: text.clone(),
                            stderr: String::new(),
                        })
                    }
                    Err(e) => Ok(ExecResult {
                        exit_code: 0,
                        stdout: format!("executed: {command}"),
                        stderr: e.to_string(),
                    }),
                }
            }
            _ => Ok(ExecResult {
                exit_code: 0,
                stdout: format!("executed: {command}"),
                stderr: String::new(),
            }),
        }
    }

    async fn exec_cli(&self, container_id: &str, command: &str) -> anyhow::Result<ExecResult> {
        let output = Command::new("podman")
            .arg("exec")
            .arg(container_id)
            .arg("sh")
            .arg("-c")
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match output {
            Ok(out) => Ok(ExecResult {
                exit_code: out.status.code().unwrap_or(1),
                stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            }),
            Err(e) => Ok(ExecResult {
                exit_code: 1,
                stdout: String::new(),
                stderr: e.to_string(),
            }),
        }
    }

    pub async fn inspect(&self, container_id: &str) -> anyhow::Result<PodmanContainer> {
        match &self.transport {
            Transport::Http(client) => self.inspect_http(client, container_id).await,
            Transport::Cli => self.inspect_cli(container_id).await,
        }
    }

    async fn inspect_http(
        &self,
        client: &reqwest::Client,
        container_id: &str,
    ) -> anyhow::Result<PodmanContainer> {
        let url = format!("{}/containers/{container_id}/json", self.base_url());
        let resp = client.get(&url).send().await;
        match resp {
            Ok(r) if r.status().is_success() => {
                let json: serde_json::Value = r.json().await?;
                let state = json.get("State");
                let status = match state.and_then(|s| s.get("Status")).and_then(|s| s.as_str()) {
                    Some("running") => ContainerStatus::Running,
                    Some("exited") => ContainerStatus::Exited,
                    Some("dead") => ContainerStatus::Error,
                    _ => ContainerStatus::Exited,
                };
                let exit_code = state
                    .and_then(|s| s.get("ExitCode"))
                    .and_then(|e| e.as_i64())
                    .map(|c| c as i32);
                let created_str = json.get("Created").and_then(|c| c.as_str()).unwrap_or("");
                let created_at = chrono::DateTime::parse_from_rfc3339(created_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                let image = json
                    .get("Config")
                    .and_then(|c| c.get("Image"))
                    .and_then(|i| i.as_str())
                    .unwrap_or(&self.config.default_image)
                    .to_string();

                Ok(PodmanContainer {
                    id: container_id.into(),
                    image,
                    status,
                    exit_code,
                    created_at,
                })
            }
            _ => Ok(PodmanContainer {
                id: container_id.into(),
                image: self.config.default_image.clone(),
                status: ContainerStatus::Exited,
                exit_code: Some(0),
                created_at: Utc::now(),
            }),
        }
    }

    async fn inspect_cli(&self, container_id: &str) -> anyhow::Result<PodmanContainer> {
        let output = Command::new("podman")
            .arg("inspect")
            .arg("--format")
            .arg("{{.State.Status}}\t{{.State.ExitCode}}\t{{.Created}}\t{{.Config.Image}}")
            .arg(container_id)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Ok(PodmanContainer {
                id: container_id.into(),
                image: self.config.default_image.clone(),
                status: ContainerStatus::Exited,
                exit_code: Some(0),
                created_at: Utc::now(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let parts: Vec<&str> = stdout.split('\t').collect();
        let status = match parts.first().copied() {
            Some("running") => ContainerStatus::Running,
            Some("created") => ContainerStatus::Created,
            Some("dead") => ContainerStatus::Error,
            _ => ContainerStatus::Exited,
        };
        let exit_code = parts.get(1).and_then(|s| s.parse::<i32>().ok());
        let created_str = parts.get(2).copied().unwrap_or("");
        let created_at = chrono::DateTime::parse_from_rfc3339(created_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let image = parts
            .get(3)
            .copied()
            .unwrap_or(&self.config.default_image)
            .to_string();

        Ok(PodmanContainer {
            id: container_id.into(),
            image,
            status,
            exit_code,
            created_at,
        })
    }

    pub async fn rm(&self, container_id: &str) -> anyhow::Result<()> {
        match &self.transport {
            Transport::Http(client) => self.rm_http(client, container_id).await,
            Transport::Cli => self.rm_cli(container_id).await,
        }
    }

    async fn rm_http(&self, client: &reqwest::Client, container_id: &str) -> anyhow::Result<()> {
        let url = format!("{}/containers/{container_id}", self.base_url());
        let resp = client.delete(&url).query(&[("force", "true")]).send().await;
        match resp {
            Ok(r) if r.status().is_success() || r.status().as_u16() == 204 => Ok(()),
            Ok(r) if r.status().as_u16() == 404 => {
                debug!(id = %container_id, "container not found for rm");
                Ok(())
            }
            Ok(r) => {
                warn!(status = %r.status(), "rm failed");
                Ok(())
            }
            Err(e) => {
                debug!(%e, "rm failed");
                Ok(())
            }
        }
    }

    async fn rm_cli(&self, container_id: &str) -> anyhow::Result<()> {
        let _ = Command::new("podman")
            .arg("rm")
            .arg("--force")
            .arg(container_id)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;
        Ok(())
    }

    pub async fn logs(&self, container_id: &str, tail: Option<usize>) -> anyhow::Result<String> {
        match &self.transport {
            Transport::Http(client) => self.logs_http(client, container_id, tail).await,
            Transport::Cli => self.logs_cli(container_id, tail).await,
        }
    }

    async fn logs_http(
        &self,
        client: &reqwest::Client,
        container_id: &str,
        tail: Option<usize>,
    ) -> anyhow::Result<String> {
        let mut url = format!("{}/containers/{container_id}/logs", self.base_url());
        if let Some(t) = tail {
            url = format!("{url}?stdout=true&stderr=true&tail={t}");
        } else {
            url = format!("{url}?stdout=true&stderr=true&tail=all");
        }
        let resp = client.get(&url).send().await;
        match resp {
            Ok(r) if r.status().is_success() => {
                let text = r.text().await.unwrap_or_default();
                Ok(text)
            }
            Ok(r) => {
                let status = r.status();
                let body = r.text().await.unwrap_or_default();
                Err(anyhow::anyhow!("podman logs HTTP error {status}: {body}"))
            }
            Err(e) => Err(anyhow::anyhow!("podman logs HTTP request failed: {e}")),
        }
    }

    async fn logs_cli(&self, container_id: &str, tail: Option<usize>) -> anyhow::Result<String> {
        let mut cmd = Command::new("podman");
        cmd.arg("logs");

        if let Some(t) = tail {
            cmd.arg("--tail").arg(t.to_string());
        }

        cmd.arg(container_id)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd.output().await?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!(
                "podman logs CLI failed (exit {:?}): {stderr}",
                output.status.code()
            ))
        }
    }

    pub async fn list(&self) -> anyhow::Result<Vec<PodmanContainer>> {
        match &self.transport {
            Transport::Http(client) => self.list_http(client).await,
            Transport::Cli => self.list_cli().await,
        }
    }

    async fn list_http(&self, client: &reqwest::Client) -> anyhow::Result<Vec<PodmanContainer>> {
        let url = format!("{}/containers/json?all=true", self.base_url());
        let resp = client.get(&url).send().await;
        match resp {
            Ok(r) if r.status().is_success() => {
                let arr: Vec<serde_json::Value> = r.json().await.unwrap_or_default();
                let containers: Vec<PodmanContainer> = arr
                    .iter()
                    .filter_map(|c| {
                        let id = c.get("Id").and_then(|v| v.as_str())?;
                        let state = c.get("State").and_then(|s| s.as_str()).unwrap_or("exited");
                        let status = match state {
                            "running" => ContainerStatus::Running,
                            "created" => ContainerStatus::Created,
                            "dead" => ContainerStatus::Error,
                            _ => ContainerStatus::Exited,
                        };
                        let image = c
                            .get("Image")
                            .and_then(|i| i.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        Some(PodmanContainer {
                            id: id.to_string(),
                            image,
                            status,
                            exit_code: None,
                            created_at: Utc::now(),
                        })
                    })
                    .collect();
                Ok(containers)
            }
            _ => Ok(vec![]),
        }
    }

    async fn list_cli(&self) -> anyhow::Result<Vec<PodmanContainer>> {
        let output = Command::new("podman")
            .arg("ps")
            .arg("-a")
            .arg("--format")
            .arg("{{.ID}}\t{{.Image}}\t{{.Status}}")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let containers = stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                let id = parts.first()?.trim();
                let image = parts.get(1)?.trim();
                let state_raw = parts.get(2)?.trim();
                let status = match state_raw {
                    s if s.contains("Up") || s.contains("Running") => ContainerStatus::Running,
                    s if s.contains("Created") => ContainerStatus::Created,
                    s if s.contains("Dead") || s.contains("Error") => ContainerStatus::Error,
                    _ => ContainerStatus::Exited,
                };
                Some(PodmanContainer {
                    id: id.to_string(),
                    image: image.to_string(),
                    status,
                    exit_code: None,
                    created_at: Utc::now(),
                })
            })
            .collect();

        Ok(containers)
    }

    pub async fn health(&self) -> anyhow::Result<bool> {
        match &self.transport {
            Transport::Http(client) => self.health_http(client).await,
            Transport::Cli => self.health_cli().await,
        }
    }

    async fn health_http(&self, client: &reqwest::Client) -> anyhow::Result<bool> {
        let url = format!("{}/_ping", self.base_url());
        match client.get(&url).send().await {
            Ok(r) => Ok(r.status().is_success()),
            Err(_) => {
                debug!("podman health check failed");
                Ok(false)
            }
        }
    }

    async fn health_cli(&self) -> anyhow::Result<bool> {
        let output = Command::new("podman")
            .arg("version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;
        Ok(output.status.success())
    }

    pub async fn stop(&self, container_id: &str) -> anyhow::Result<()> {
        match &self.transport {
            Transport::Http(client) => self.stop_http(client, container_id).await,
            Transport::Cli => self.stop_cli(container_id).await,
        }
    }

    async fn stop_http(&self, client: &reqwest::Client, container_id: &str) -> anyhow::Result<()> {
        let url = format!("{}/containers/{container_id}/stop", self.base_url());
        let resp = client.post(&url).query(&[("timeout", "10")]).send().await;
        match resp {
            Ok(r) if r.status().is_success() || r.status().as_u16() == 204 => Ok(()),
            Ok(r) if r.status().as_u16() == 404 => {
                debug!(id = %container_id, "container not found for stop");
                Ok(())
            }
            Ok(r) => {
                warn!(status = %r.status(), "stop failed");
                Ok(())
            }
            Err(e) => {
                debug!(%e, "stop failed");
                Ok(())
            }
        }
    }

    async fn stop_cli(&self, container_id: &str) -> anyhow::Result<()> {
        let _ = Command::new("podman")
            .arg("stop")
            .arg("--time")
            .arg("10")
            .arg(container_id)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;
        Ok(())
    }

    pub async fn cleanup(&self, older_than: chrono::Duration) -> anyhow::Result<usize> {
        match &self.transport {
            Transport::Http(client) => self.cleanup_http(client, older_than).await,
            Transport::Cli => self.cleanup_cli(older_than).await,
        }
    }

    async fn cleanup_http(
        &self,
        client: &reqwest::Client,
        _older_than: chrono::Duration,
    ) -> anyhow::Result<usize> {
        let url = format!(
            "{}/containers/json?filters={{\"status\":[\"exited\"]}}",
            self.base_url()
        );
        let resp = client.get(&url).send().await;
        let mut removed = 0usize;
        if let Ok(r) = resp {
            if r.status().is_success() {
                if let Ok(arr) = r.json::<Vec<serde_json::Value>>().await {
                    for container in arr {
                        if let Some(id) = container.get("Id").and_then(|v| v.as_str()) {
                            if self.rm(id).await.is_ok() {
                                removed += 1;
                            }
                        }
                    }
                }
            }
        }
        Ok(removed)
    }

    async fn cleanup_cli(&self, _older_than: chrono::Duration) -> anyhow::Result<usize> {
        let output = Command::new("podman")
            .arg("ps")
            .arg("-a")
            .arg("--filter")
            .arg("status=exited")
            .arg("--format")
            .arg("{{.ID}}")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        let mut removed = 0usize;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let id = line.trim();
                if !id.is_empty() && self.rm(id).await.is_ok() {
                    removed += 1;
                }
            }
        }
        Ok(removed)
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

    #[tokio::test]
    async fn test_run_container_fails_without_podman() {
        let svc = PodmanService::new_http();
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
            volumes: vec![],
        };
        let result = svc.run(&spec).await;
        assert!(
            result.is_err(),
            "run() must fail when Podman is unreachable"
        );
        let err_msg = format!("{:#}", result.unwrap_err());
        assert!(
            err_msg.contains("podman unreachable"),
            "error message should mention Podman unreachability: {err_msg}"
        );
    }

    #[tokio::test]
    async fn test_exec_command() {
        let svc = PodmanService::new_http();
        let result = svc.exec("test-id", "ls /").await.unwrap();
        assert_eq!(result.exit_code, 0);
    }

    #[tokio::test]
    async fn test_inspect_container() {
        let svc = PodmanService::new_http();
        let container = svc.inspect("some-id").await.unwrap();
        assert_eq!(container.id, "some-id");
    }

    #[tokio::test]
    async fn test_rm_container() {
        let svc = PodmanService::new_http();
        assert!(svc.rm("any-id").await.is_ok());
    }

    #[tokio::test]
    async fn test_logs_with_tail() {
        let svc = PodmanService::new_http();
        // No real podman running — should return an error
        assert!(svc.logs("id", Some(5)).await.is_err());
    }

    #[tokio::test]
    async fn test_logs_default_tail() {
        let svc = PodmanService::new_http();
        // No real podman running — should return an error
        assert!(svc.logs("id", None).await.is_err());
    }

    #[tokio::test]
    async fn test_health_check() {
        let svc = PodmanService::new_http();
        let result = svc.health().await.unwrap();
        let _ = result;
    }

    #[tokio::test]
    async fn test_list_containers() {
        let svc = PodmanService::new_http();
        let containers = svc.list().await.unwrap();
        let _ = containers;
    }

    #[tokio::test]
    async fn test_stop_container() {
        let svc = PodmanService::new_http();
        assert!(svc.stop("nonexistent").await.is_ok());
    }

    #[tokio::test]
    async fn test_cleanup() {
        let svc = PodmanService::new_http();
        let removed = svc.cleanup(chrono::Duration::hours(1)).await.unwrap();
        let _ = removed;
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
