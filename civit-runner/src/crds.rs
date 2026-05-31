#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "civitforge.io",
    version = "v1",
    kind = "PipelineRun",
    namespaced,
    singular = "pipelinerun",
    plural = "pipelineruns",
    shortname = "prun",
    status = "PipelineRunStatus"
)]
pub struct PipelineRunSpec {
    #[serde(default)]
    pub repo_url: String,
    #[serde(default)]
    pub ref_field: String,
    pub steps: Vec<PipelineStep>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,
    #[serde(default)]
    pub node_selector: Option<HashMap<String, String>>,
    #[serde(default)]
    pub resources: Option<ResourceRequirements>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct PipelineStep {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub workdir: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub continue_on_error: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct ResourceRequirements {
    #[serde(default)]
    pub cpu_limit: String,
    #[serde(default)]
    pub memory_limit: String,
}

fn default_timeout() -> u32 {
    3600
}

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "civitforge.io",
    version = "v1",
    kind = "TaskSpec",
    namespaced,
    singular = "taskspec",
    plural = "taskspecs",
    shortname = "tspec"
)]
pub struct TaskSpecSpec {
    pub name: String,
    pub steps: Vec<PipelineStep>,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,
    #[serde(default)]
    pub resources: Option<ResourceRequirements>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PipelineRunStatus {
    #[serde(default)]
    pub phase: PipelinePhase,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_statuses: Option<Vec<StepStatus>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion_time: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PipelinePhase {
    #[default]
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl PipelinePhase {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            PipelinePhase::Succeeded | PipelinePhase::Failed | PipelinePhase::Cancelled
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct StepStatus {
    pub name: String,
    pub phase: PipelinePhase,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_pipeline_run_spec() -> PipelineRunSpec {
        PipelineRunSpec {
            repo_url: "https://github.com/example/repo".into(),
            ref_field: "refs/heads/main".into(),
            steps: vec![PipelineStep {
                name: "build".into(),
                command: "cargo build --release".into(),
                workdir: Some("/workspace".into()),
                env: HashMap::new(),
                continue_on_error: false,
            }],
            env: HashMap::new(),
            timeout_seconds: 600,
            node_selector: Some(HashMap::new()),
            resources: Some(ResourceRequirements {
                cpu_limit: "500m".into(),
                memory_limit: "256Mi".into(),
            }),
        }
    }

    #[test]
    fn test_pipeline_run_spec_serialization() {
        let spec = sample_pipeline_run_spec();
        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("repo_url"));
        assert!(json.contains("ref_field"));
        let de: PipelineRunSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(de.repo_url, "https://github.com/example/repo");
        assert_eq!(de.ref_field, "refs/heads/main");
        assert_eq!(de.steps.len(), 1);
    }

    #[test]
    fn test_pipeline_run_spec_defaults() {
        let spec = PipelineRunSpec {
            repo_url: String::new(),
            ref_field: String::new(),
            steps: vec![],
            env: HashMap::new(),
            timeout_seconds: 3600,
            node_selector: None,
            resources: None,
        };
        assert_eq!(spec.timeout_seconds, 3600);
        assert!(spec.node_selector.is_none());
        assert!(spec.resources.is_none());
    }

    #[test]
    fn test_pipeline_step_serialization() {
        let step = PipelineStep {
            name: "test".into(),
            command: "cargo test".into(),
            workdir: Some("/src".into()),
            env: HashMap::new(),
            continue_on_error: true,
        };
        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("continue_on_error"));
        let de: PipelineStep = serde_json::from_str(&json).unwrap();
        assert!(de.continue_on_error);
        assert_eq!(de.workdir.as_deref(), Some("/src"));
    }

    #[test]
    fn test_resource_requirements_defaults() {
        let rr = ResourceRequirements {
            cpu_limit: "2".into(),
            memory_limit: "4Gi".into(),
        };
        let json = serde_json::to_string(&rr).unwrap();
        let de: ResourceRequirements = serde_json::from_str(&json).unwrap();
        assert_eq!(de.cpu_limit, "2");
        assert_eq!(de.memory_limit, "4Gi");
    }

    #[test]
    fn test_taskspec_spec_serialization() {
        let spec = TaskSpecSpec {
            name: "rust-build".into(),
            steps: vec![PipelineStep {
                name: "compile".into(),
                command: "cargo build".into(),
                workdir: None,
                env: HashMap::new(),
                continue_on_error: false,
            }],
            timeout_seconds: 1800,
            resources: None,
        };
        let json = serde_json::to_string(&spec).unwrap();
        let de: TaskSpecSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(de.name, "rust-build");
        assert_eq!(de.steps.len(), 1);
    }

    #[test]
    fn test_pipeline_status_phase_transitions() {
        let mut status = PipelineRunStatus::default();
        assert_eq!(status.phase, PipelinePhase::Pending);
        assert!(!status.phase.is_terminal());

        status.phase = PipelinePhase::Running;
        assert!(!status.phase.is_terminal());

        status.phase = PipelinePhase::Succeeded;
        assert!(status.phase.is_terminal());

        status.phase = PipelinePhase::Failed;
        assert!(status.phase.is_terminal());

        status.phase = PipelinePhase::Cancelled;
        assert!(status.phase.is_terminal());
    }

    #[test]
    fn test_pipeline_phase_serialization() {
        let phases = vec![
            PipelinePhase::Pending,
            PipelinePhase::Running,
            PipelinePhase::Succeeded,
            PipelinePhase::Failed,
            PipelinePhase::Cancelled,
        ];
        for phase in phases {
            let json = serde_json::to_string(&phase).unwrap();
            let de: PipelinePhase = serde_json::from_str(&json).unwrap();
            assert_eq!(phase, de);
        }
    }

    #[test]
    fn test_step_status_construction() {
        let step_status = StepStatus {
            name: "build".into(),
            phase: PipelinePhase::Succeeded,
            exit_code: Some(0),
            message: None,
        };
        let json = serde_json::to_string(&step_status).unwrap();
        let de: StepStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(de.name, "build");
        assert_eq!(de.exit_code, Some(0));
        assert!(de.message.is_none());
    }

    #[test]
    fn test_pipeline_run_status_new() {
        let status = PipelineRunStatus::default();
        assert_eq!(status.phase, PipelinePhase::Pending);
        assert!(status.step_statuses.is_none());
        assert!(status.message.is_none());
        assert!(status.start_time.is_none());
        assert!(status.completion_time.is_none());
    }

    #[test]
    fn test_pipeline_run_status_with_steps() {
        let status = PipelineRunStatus {
            phase: PipelinePhase::Running,
            step_statuses: Some(vec![StepStatus {
                name: "build".into(),
                phase: PipelinePhase::Succeeded,
                exit_code: Some(0),
                message: None,
            }]),
            message: Some("building".into()),
            start_time: Some(Utc::now()),
            completion_time: None,
        };
        assert_eq!(status.step_statuses.as_ref().unwrap().len(), 1);
        assert_eq!(status.message.as_deref(), Some("building"));
    }

    #[test]
    fn test_pipeline_run_crd_json_schema() {
        let _spec = sample_pipeline_run_spec();
        let schema = schemars::r#gen::SchemaSettings::openapi3()
            .with(|s| {
                s.inline_subschemas = true;
                s.meta_schema = None;
            })
            .into_generator()
            .into_root_schema_for::<PipelineRunSpec>();
        assert!(schema.schema.object.is_some());
    }
}
