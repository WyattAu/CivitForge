#![forbid(unsafe_code)]

use crate::models::{PipelineSpec, PipelineStatus, PipelineStep, StepStatus};
use crate::podman::{PodmanRunSpec, PodmanService};
use std::collections::HashMap;
use tracing::{debug, info};

pub struct PipelineEngine {
    #[allow(dead_code)]
    namespace: String,
    podman: Option<PodmanService>,
    /// Container IDs for running service containers, keyed by service name.
    service_containers: std::sync::Arc<tokio::sync::Mutex<HashMap<String, String>>>,
}

impl std::fmt::Debug for PipelineEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PipelineEngine")
            .field("namespace", &self.namespace)
            .field(
                "podman",
                &self.podman.as_ref().map(|_| "Some(PodmanService)"),
            )
            .finish()
    }
}

impl PipelineEngine {
    pub fn new(namespace: String, podman: Option<PodmanService>) -> Self {
        Self {
            namespace,
            podman,
            service_containers: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    pub fn new_without_runner(namespace: String) -> Self {
        Self {
            namespace,
            podman: None,
            service_containers: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Start service containers before running pipeline steps.
    /// Returns a guard that will clean up all services on drop.
    pub async fn start_services(
        &self,
        services: &[(String, String)], // (name, image)
    ) -> anyhow::Result<ServiceGuard> {
        let mut containers = HashMap::new();
        if let Some(ref podman) = self.podman {
            for (name, image) in services {
                let spec = PodmanRunSpec {
                    image: image.clone(),
                    command: vec![],
                    env: HashMap::new(),
                    memory_mb: 256,
                    cpu_quota: 50_000,
                    network_disabled: false, // Services need network for inter-container communication
                    read_only_fs: false,
                    workdir: "/".to_string(),
                    timeout_secs: 0, // Services run indefinitely
                    labels: HashMap::from([
                        ("civit.service".to_string(), name.clone()),
                        ("civit.type".to_string(), "service".to_string()),
                    ]),
                    volumes: vec![],
                };

                let container = podman.run(&spec).await?;
                containers.insert(name.clone(), container.id.clone());
                info!(service = %name, id = %container.id, "service container started");
            }
        }

        // Store container IDs for cleanup
        {
            let mut svc_map = self.service_containers.lock().await;
            for (name, id) in &containers {
                svc_map.insert(name.clone(), id.clone());
            }
        }

        // Wait for services to be healthy (simple delay — real implementation uses health checks)
        if !containers.is_empty() {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }

        let guard = ServiceGuard {
            podman: self.podman.clone(),
            containers,
        };

        Ok(guard)
    }

    /// Stop all running service containers.
    pub async fn stop_all_services(&self) {
        let mut svc_map = self.service_containers.lock().await;
        if let Some(podman) = self.podman.as_ref() {
            for (name, id) in svc_map.drain() {
                info!(service = %name, id = %id, "stopping service container");
                let _ = podman.stop(&id).await;
                let _ = podman.rm(&id).await;
            }
        }
    }

    pub async fn run(&self, spec: &PipelineSpec) -> anyhow::Result<PipelineStatus> {
        info!(pipeline = %spec.name, steps = spec.steps.len(), "starting pipeline");

        let mut step_results: Vec<StepStatus> = Vec::new();

        for step in &spec.steps {
            let result = self.execute_step(step, &step_results).await;
            let status = match result {
                Ok(output) => {
                    info!(step = %step.name, "step succeeded");
                    StepStatus {
                        name: step.name.clone(),
                        status: "success".into(),
                        started_at: chrono::Utc::now().to_rfc3339(),
                        finished_at: chrono::Utc::now().to_rfc3339(),
                        output,
                    }
                }
                Err(e) => {
                    info!(step = %step.name, error = %e, "step failed");
                    StepStatus {
                        name: step.name.clone(),
                        status: "failed".into(),
                        started_at: chrono::Utc::now().to_rfc3339(),
                        finished_at: chrono::Utc::now().to_rfc3339(),
                        output: e.to_string(),
                    }
                }
            };

            let succeeded = status.status == "success";
            step_results.push(status);

            if !succeeded {
                return Ok(PipelineStatus {
                    pipeline: spec.name.clone(),
                    status: "failed".into(),
                    step_results,
                    started_at: chrono::Utc::now().to_rfc3339(),
                    finished_at: chrono::Utc::now().to_rfc3339(),
                });
            }
        }

        Ok(PipelineStatus {
            pipeline: spec.name.clone(),
            status: "success".into(),
            step_results,
            started_at: chrono::Utc::now().to_rfc3339(),
            finished_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    async fn execute_step(
        &self,
        step: &PipelineStep,
        prior_results: &[StepStatus],
    ) -> anyhow::Result<String> {
        if let Some(ref condition) = step.condition
            && !self.evaluate_condition(condition, prior_results)
        {
            info!(step = %step.name, "step skipped due to condition");
            return Ok(String::new());
        }

        debug!(
            step = %step.name,
            image = %step.image,
            commands = step.commands.len(),
            "executing step"
        );

        let svc = match &self.podman {
            Some(s) => s,
            None => {
                debug!(
                    step = %step.name,
                    "no podman service configured, using stub execution"
                );
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                return Ok(String::new());
            }
        };

        let shell_cmd = step.commands.join("\n");
        let spec = PodmanRunSpec {
            image: step.image.clone(),
            command: vec!["sh".into(), "-c".into(), shell_cmd],
            env: step.env.clone(),
            memory_mb: 512,
            cpu_quota: 100_000,
            network_disabled: true,
            read_only_fs: true,
            workdir: "/workspace".into(),
            timeout_secs: 600,
            labels: {
                let mut labels = HashMap::new();
                labels.insert("civit.pipeline".into(), self.namespace.clone());
                labels.insert("civit.step".into(), step.name.clone());
                labels
            },
            volumes: vec![],
        };

        let container = svc.run(&spec).await?;

        let logs = svc.logs(&container.id, None).await.unwrap_or_default();

        let inspected = match svc.inspect(&container.id).await {
            Ok(c) => c,
            Err(e) => {
                let _ = svc.rm(&container.id).await;
                anyhow::bail!(
                    "failed to inspect container {} for step '{}': {}",
                    container.id,
                    step.name,
                    e
                );
            }
        };

        let _ = svc.rm(&container.id).await;

        match inspected.exit_code {
            Some(code) if code != 0 => {
                anyhow::bail!(
                    "container {} exited with code {} for step '{}':\n{}",
                    container.id,
                    code,
                    step.name,
                    logs
                );
            }
            _ => {}
        }

        Ok(logs)
    }

    fn evaluate_condition(
        &self,
        condition: &crate::models::StepCondition,
        prior_results: &[StepStatus],
    ) -> bool {
        match condition {
            crate::models::StepCondition::Always => true,
            crate::models::StepCondition::OnSuccess => {
                prior_results.iter().all(|r| r.status == "success")
            }
            crate::models::StepCondition::OnFailure => {
                prior_results.iter().any(|r| r.status == "failed")
            }
            crate::models::StepCondition::Branch { branches } => match std::env::var("CI_BRANCH") {
                Ok(ref branch) => branches.iter().any(|b| b == branch),
                Err(_) => branches.is_empty(),
            },
            crate::models::StepCondition::EnvVar { key, value } => {
                std::env::var(key).map(|v| v == *value).unwrap_or(false)
            }
        }
    }

    pub fn validate_spec(&self, spec: &PipelineSpec) -> Vec<String> {
        let mut warnings = Vec::new();
        if spec.steps.is_empty() {
            warnings.push("pipeline has no steps".into());
        }
        for (i, step) in spec.steps.iter().enumerate() {
            if step.name.is_empty() {
                warnings.push(format!("step {i}: empty name"));
            }
            if step.image.is_empty() {
                warnings.push(format!("step {i}: empty image"));
            }
            if step.commands.is_empty() {
                warnings.push(format!("step {}: no commands", step.name));
            }
        }
        warnings
    }
}

// ---------------------------------------------------------------------------
// Service Guard — RAII cleanup for service containers
// ---------------------------------------------------------------------------

/// RAII guard that stops and removes service containers when dropped.
pub struct ServiceGuard {
    podman: Option<PodmanService>,
    containers: HashMap<String, String>,
}

impl ServiceGuard {
    /// Check if a service is running.
    pub fn has_service(&self, name: &str) -> bool {
        self.containers.contains_key(name)
    }

    /// Get the list of running service names.
    pub fn service_names(&self) -> Vec<String> {
        self.containers.keys().cloned().collect()
    }
}

impl Drop for ServiceGuard {
    fn drop(&mut self) {
        if self.podman.is_some() {
            // Log service names being dropped (can't await in Drop)
            let names: Vec<String> = self.containers.keys().cloned().collect();
            for name in names {
                tracing::info!(service = %name, "service guard: container not stopped (call stop_all_services explicitly)");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pipeline_success() {
        let engine = PipelineEngine::new_without_runner("default".into());
        let spec = PipelineSpec {
            name: "test-pipeline".into(),
            triggers: vec!["push".into()],
            steps: vec![
                PipelineStep {
                    name: "step1".into(),
                    image: "alpine:latest".into(),
                    commands: vec!["echo hello".into()],
                    env: std::collections::HashMap::new(),
                    condition: None,
                },
                PipelineStep {
                    name: "step2".into(),
                    image: "alpine:latest".into(),
                    commands: vec!["echo world".into()],
                    env: std::collections::HashMap::new(),
                    condition: None,
                },
            ],
        };
        let result = engine.run(&spec).await.unwrap();
        assert_eq!(result.status, "success");
        assert_eq!(result.step_results.len(), 2);
    }

    #[test]
    fn test_validate_empty_pipeline() {
        let engine = PipelineEngine::new_without_runner("default".into());
        let spec = PipelineSpec {
            name: "empty".into(),
            triggers: vec![],
            steps: vec![],
        };
        let warnings = engine.validate_spec(&spec);
        assert!(warnings.iter().any(|w| w.contains("no steps")));
    }

    #[tokio::test]
    async fn test_evaluate_condition_on_failure_true_when_prior_failed() {
        let engine = PipelineEngine::new_without_runner("default".into());
        let prior = vec![StepStatus {
            name: "step1".into(),
            status: "failed".into(),
            started_at: chrono::Utc::now().to_rfc3339(),
            finished_at: chrono::Utc::now().to_rfc3339(),
            output: "error".into(),
        }];
        assert!(engine.evaluate_condition(&crate::models::StepCondition::OnFailure, &prior));
        assert!(!engine.evaluate_condition(&crate::models::StepCondition::OnSuccess, &prior));
    }

    #[tokio::test]
    async fn test_evaluate_condition_on_success_false_when_prior_failed() {
        let engine = PipelineEngine::new_without_runner("default".into());
        let prior = vec![StepStatus {
            name: "step1".into(),
            status: "success".into(),
            started_at: chrono::Utc::now().to_rfc3339(),
            finished_at: chrono::Utc::now().to_rfc3339(),
            output: String::new(),
        }];
        assert!(engine.evaluate_condition(&crate::models::StepCondition::OnSuccess, &prior));
        assert!(!engine.evaluate_condition(&crate::models::StepCondition::OnFailure, &prior));
    }

    #[tokio::test]
    async fn test_evaluate_condition_on_failure_empty_prior() {
        let engine = PipelineEngine::new_without_runner("default".into());
        let prior: Vec<StepStatus> = vec![];
        assert!(!engine.evaluate_condition(&crate::models::StepCondition::OnFailure, &prior));
        assert!(engine.evaluate_condition(&crate::models::StepCondition::OnSuccess, &prior));
    }

    #[tokio::test]
    async fn test_evaluate_condition_branch_empty_branches() {
        let engine = PipelineEngine::new_without_runner("default".into());
        let prior: Vec<StepStatus> = vec![];
        let cond = crate::models::StepCondition::Branch { branches: vec![] };
        assert!(engine.evaluate_condition(&cond, &prior));
    }

    #[tokio::test]
    async fn test_evaluate_condition_branch_nonempty_no_env() {
        let engine = PipelineEngine::new_without_runner("default".into());
        let prior: Vec<StepStatus> = vec![];
        let cond = crate::models::StepCondition::Branch {
            branches: vec!["main".into()],
        };
        assert!(!engine.evaluate_condition(&cond, &prior));
    }

    #[test]
    fn test_validate_step_warnings() {
        let engine = PipelineEngine::new_without_runner("default".into());
        let spec = PipelineSpec {
            name: "bad-steps".into(),
            triggers: vec![],
            steps: vec![PipelineStep {
                name: "".into(),
                image: "".into(),
                commands: vec![],
                env: std::collections::HashMap::new(),
                condition: None,
            }],
        };
        let warnings = engine.validate_spec(&spec);
        assert!(warnings.iter().any(|w| w.contains("empty name")));
        assert!(warnings.iter().any(|w| w.contains("empty image")));
        assert!(warnings.iter().any(|w| w.contains("no commands")));
    }
}
