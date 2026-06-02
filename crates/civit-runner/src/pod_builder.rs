#![forbid(unsafe_code)]

use crate::crds::{PipelineRunSpec, ResourceRequirements};
use k8s_openapi::api::core::v1::{
    Container, EnvVar, Pod, PodSpec, ResourceRequirements as K8sResourceRequirements,
    SecurityContext, Volume, VolumeMount,
};
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use std::collections::BTreeMap;
#[cfg(test)]
use std::collections::HashMap;

pub const DEFAULT_SANDBOX_IMAGE: &str = "rust:1.88-slim";
pub const WORKSPACE_VOLUME_NAME: &str = "workspace";
pub const WORKSPACE_MOUNT_PATH: &str = "/workspace";

pub struct PodBuilder {
    image: String,
}

impl Default for PodBuilder {
    fn default() -> Self {
        Self {
            image: DEFAULT_SANDBOX_IMAGE.into(),
        }
    }
}

impl PodBuilder {
    pub fn new(image: impl Into<String>) -> Self {
        Self {
            image: image.into(),
        }
    }

    pub fn build_sandbox_pod(
        &self,
        name: &str,
        namespace: &str,
        spec: &PipelineRunSpec,
        owner_uid: &str,
    ) -> Pod {
        let script = build_entrypoint_script(spec);

        let mut env_vars: Vec<EnvVar> = spec
            .env
            .iter()
            .map(|(k, v)| EnvVar {
                name: k.clone(),
                value: Some(v.clone()),
                value_from: None,
            })
            .collect();

        env_vars.push(EnvVar {
            name: "REPO_URL".into(),
            value: Some(spec.repo_url.clone()),
            value_from: None,
        });
        env_vars.push(EnvVar {
            name: "GIT_REF".into(),
            value: Some(spec.ref_field.clone()),
            value_from: None,
        });

        let resource_limits = spec
            .resources
            .as_ref()
            .map(build_resource_requirements)
            .unwrap_or_default();

        let container = Container {
            name: "pipeline".into(),
            image: Some(self.image.clone()),
            command: Some(vec!["/bin/sh".into(), "-c".into()]),
            args: Some(vec![script]),
            env: Some(env_vars),
            volume_mounts: Some(vec![VolumeMount {
                name: WORKSPACE_VOLUME_NAME.into(),
                mount_path: WORKSPACE_MOUNT_PATH.into(),
                ..Default::default()
            }]),
            resources: Some(K8sResourceRequirements {
                limits: Some(resource_limits),
                requests: None,
                claims: None,
            }),
            security_context: Some(SecurityContext {
                run_as_non_root: Some(true),
                allow_privilege_escalation: Some(false),
                read_only_root_filesystem: Some(false),
                ..Default::default()
            }),
            ..Default::default()
        };

        let mut labels = BTreeMap::new();
        labels.insert("app.kubernetes.io/name".into(), "civit-operator".into());
        labels.insert("app.kubernetes.io/component".into(), "pipeline-run".into());
        labels.insert("civitforge.io/pipelinerun".into(), name.into());

        let mut annotations = BTreeMap::new();
        annotations.insert("civitforge.io/repo-url".into(), spec.repo_url.clone());
        annotations.insert("civitforge.io/git-ref".into(), spec.ref_field.clone());

        Pod {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some(name.into()),
                namespace: Some(namespace.into()),
                labels: Some(labels),
                annotations: Some(annotations),
                owner_references: Some(vec![
                    k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference {
                        api_version: "civitforge.io/v1".into(),
                        kind: "PipelineRun".into(),
                        name: name.into(),
                        uid: owner_uid.into(),
                        controller: Some(true),
                        block_owner_deletion: Some(true),
                    },
                ]),
                ..Default::default()
            },
            spec: Some(PodSpec {
                restart_policy: Some("Never".into()),
                containers: vec![container],
                volumes: Some(vec![Volume {
                    name: WORKSPACE_VOLUME_NAME.into(),
                    empty_dir: Some(k8s_openapi::api::core::v1::EmptyDirVolumeSource::default()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

fn build_entrypoint_script(spec: &PipelineRunSpec) -> String {
    let mut commands = String::new();
    commands.push_str("set -e\n");
    commands.push_str("set -o pipefail\n");

    for step in &spec.steps {
        let workdir = step.workdir.as_deref().unwrap_or(WORKSPACE_MOUNT_PATH);

        if !step.env.is_empty() {
            for (k, v) in &step.env {
                commands.push_str(&format!("export {k}={v}\n"));
            }
        }

        if step.workdir.is_some() {
            commands.push_str(&format!("cd {workdir}\n"));
        }

        if step.continue_on_error {
            commands.push_str(&format!(
                "echo '--- Step: {} ---'\n{} || true\n",
                step.name, step.command
            ));
        } else {
            commands.push_str(&format!(
                "echo '--- Step: {} ---'\n{}\n",
                step.name, step.command
            ));
        }
    }

    commands
}

fn build_resource_requirements(rr: &ResourceRequirements) -> BTreeMap<String, Quantity> {
    let mut limits = BTreeMap::new();
    if !rr.cpu_limit.is_empty() {
        limits.insert("cpu".into(), Quantity(rr.cpu_limit.clone()));
    }
    if !rr.memory_limit.is_empty() {
        limits.insert("memory".into(), Quantity(rr.memory_limit.clone()));
    }
    limits
}

pub fn build_resource_requirements_map(rr: &ResourceRequirements) -> BTreeMap<String, Quantity> {
    let mut limits = BTreeMap::new();
    if !rr.cpu_limit.is_empty() {
        limits.insert("cpu".into(), Quantity(rr.cpu_limit.clone()));
    }
    if !rr.memory_limit.is_empty() {
        limits.insert("memory".into(), Quantity(rr.memory_limit.clone()));
    }
    limits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crds::PipelineStep;

    fn sample_spec() -> PipelineRunSpec {
        let mut env = HashMap::new();
        env.insert("RUST_LOG".into(), "info".into());
        PipelineRunSpec {
            repo_url: "https://github.com/example/repo".into(),
            ref_field: "main".into(),
            steps: vec![
                PipelineStep {
                    name: "checkout".into(),
                    command: "git clone $REPO_URL .".into(),
                    workdir: None,
                    env: HashMap::new(),
                    continue_on_error: false,
                },
                PipelineStep {
                    name: "build".into(),
                    command: "cargo build --release".into(),
                    workdir: Some("/workspace".into()),
                    env: HashMap::new(),
                    continue_on_error: false,
                },
            ],
            env,
            timeout_seconds: 3600,
            node_selector: Some(HashMap::new()),
            resources: Some(ResourceRequirements {
                cpu_limit: "2".into(),
                memory_limit: "4Gi".into(),
            }),
        }
    }

    #[test]
    fn test_build_entrypoint_script() {
        let spec = sample_spec();
        let script = build_entrypoint_script(&spec);
        assert!(script.contains("set -e"));
        assert!(script.contains("checkout"));
        assert!(script.contains("build"));
        assert!(script.contains("git clone"));
        assert!(script.contains("cargo build --release"));
    }

    #[test]
    fn test_build_entrypoint_script_continue_on_error() {
        let spec = PipelineRunSpec {
            repo_url: String::new(),
            ref_field: String::new(),
            steps: vec![PipelineStep {
                name: "flaky".into(),
                command: "cargo test".into(),
                workdir: None,
                env: HashMap::new(),
                continue_on_error: true,
            }],
            env: HashMap::new(),
            timeout_seconds: 3600,
            node_selector: None,
            resources: None,
        };
        let script = build_entrypoint_script(&spec);
        assert!(script.contains("|| true"));
    }

    #[test]
    fn test_build_entrypoint_script_with_step_env() {
        let mut env = HashMap::new();
        env.insert("TARGET".into(), "x86_64".into());
        let spec = PipelineRunSpec {
            repo_url: String::new(),
            ref_field: String::new(),
            steps: vec![PipelineStep {
                name: "build".into(),
                command: "make".into(),
                workdir: None,
                env,
                continue_on_error: false,
            }],
            env: HashMap::new(),
            timeout_seconds: 3600,
            node_selector: None,
            resources: None,
        };
        let script = build_entrypoint_script(&spec);
        assert!(script.contains("export TARGET=x86_64"));
    }

    #[test]
    fn test_pod_builder_basic() {
        let builder = PodBuilder::new("rust:1.88-slim");
        let spec = sample_spec();
        let pod = builder.build_sandbox_pod("prun-test-abc", "default", &spec, "uid-123");

        assert_eq!(pod.metadata.name.as_deref(), Some("prun-test-abc"));
        assert_eq!(pod.metadata.namespace.as_deref(), Some("default"));
    }

    #[test]
    fn test_pod_builder_labels() {
        let builder = PodBuilder::new("alpine:latest");
        let spec = sample_spec();
        let pod = builder.build_sandbox_pod("prun-labels", "ns", &spec, "uid");

        let labels = pod.metadata.labels.as_ref().unwrap();
        assert_eq!(
            labels.get("app.kubernetes.io/name").unwrap(),
            "civit-operator"
        );
        assert_eq!(
            labels.get("app.kubernetes.io/component").unwrap(),
            "pipeline-run"
        );
        assert_eq!(
            labels.get("civitforge.io/pipelinerun").unwrap(),
            "prun-labels"
        );
    }

    #[test]
    fn test_pod_builder_owner_reference() {
        let builder = PodBuilder::new("alpine:latest");
        let spec = sample_spec();
        let pod = builder.build_sandbox_pod("prun-own", "default", &spec, "uid-abc");

        let owners = pod.metadata.owner_references.as_ref().unwrap();
        assert_eq!(owners.len(), 1);
        assert_eq!(owners[0].api_version, "civitforge.io/v1");
        assert_eq!(owners[0].kind, "PipelineRun");
        assert_eq!(owners[0].uid, "uid-abc");
        assert_eq!(owners[0].controller, Some(true));
        assert_eq!(owners[0].block_owner_deletion, Some(true));
    }

    #[test]
    fn test_pod_builder_container_image() {
        let builder = PodBuilder::new("custom/image:latest");
        let spec = sample_spec();
        let pod = builder.build_sandbox_pod("prun-img", "default", &spec, "uid");

        let container = pod.spec.as_ref().unwrap().containers.first().unwrap();
        assert_eq!(container.image.as_deref(), Some("custom/image:latest"));
    }

    #[test]
    fn test_pod_builder_resource_limits() {
        let builder = PodBuilder::new("alpine:latest");
        let spec = sample_spec();
        let pod = builder.build_sandbox_pod("prun-res", "default", &spec, "uid");

        let container = pod.spec.as_ref().unwrap().containers.first().unwrap();
        let resources = container.resources.as_ref().unwrap();
        let limits = resources.limits.as_ref().unwrap();
        assert_eq!(limits.get("cpu").unwrap().0, "2");
        assert_eq!(limits.get("memory").unwrap().0, "4Gi");
    }

    #[test]
    fn test_pod_builder_workspace_volume() {
        let builder = PodBuilder::new("alpine:latest");
        let spec = sample_spec();
        let pod = builder.build_sandbox_pod("prun-vol", "default", &spec, "uid");

        let volumes = pod.spec.as_ref().unwrap().volumes.as_ref().unwrap();
        assert_eq!(volumes.len(), 1);
        assert_eq!(volumes[0].name, WORKSPACE_VOLUME_NAME);
        assert!(volumes[0].empty_dir.is_some());

        let container = pod.spec.as_ref().unwrap().containers.first().unwrap();
        let mounts = container.volume_mounts.as_ref().unwrap();
        assert_eq!(mounts.len(), 1);
        assert_eq!(mounts[0].mount_path, WORKSPACE_MOUNT_PATH);
    }

    #[test]
    fn test_pod_builder_env_vars() {
        let builder = PodBuilder::new("alpine:latest");
        let spec = sample_spec();
        let pod = builder.build_sandbox_pod("prun-env", "default", &spec, "uid");

        let container = pod.spec.as_ref().unwrap().containers.first().unwrap();
        let env = container.env.as_ref().unwrap();
        let env_map: HashMap<&str, &str> = env
            .iter()
            .filter_map(|e| e.value.as_ref().map(|v| (e.name.as_str(), v.as_str())))
            .collect();
        assert_eq!(*env_map.get("REPO_URL").unwrap(), spec.repo_url.as_str());
        assert_eq!(*env_map.get("GIT_REF").unwrap(), spec.ref_field.as_str());
        assert_eq!(*env_map.get("RUST_LOG").unwrap(), "info");
    }

    #[test]
    fn test_pod_builder_security_context() {
        let builder = PodBuilder::new("alpine:latest");
        let spec = sample_spec();
        let pod = builder.build_sandbox_pod("prun-sec", "default", &spec, "uid");

        let container = pod.spec.as_ref().unwrap().containers.first().unwrap();
        let sec = container.security_context.as_ref().unwrap();
        assert_eq!(sec.run_as_non_root, Some(true));
        assert_eq!(sec.allow_privilege_escalation, Some(false));
    }

    #[test]
    fn test_pod_builder_restart_policy() {
        let builder = PodBuilder::new("alpine:latest");
        let spec = sample_spec();
        let pod = builder.build_sandbox_pod("prun-resp", "default", &spec, "uid");

        assert_eq!(
            pod.spec.as_ref().unwrap().restart_policy.as_deref(),
            Some("Never")
        );
    }

    #[test]
    fn test_build_resource_requirements_map() {
        let rr = ResourceRequirements {
            cpu_limit: "500m".into(),
            memory_limit: "128Mi".into(),
        };
        let map = build_resource_requirements_map(&rr);
        assert_eq!(map.get("cpu").unwrap().0, "500m");
        assert_eq!(map.get("memory").unwrap().0, "128Mi");
    }

    #[test]
    fn test_default_builder_image() {
        let builder = PodBuilder::default();
        assert_eq!(builder.image, DEFAULT_SANDBOX_IMAGE);
    }
}
