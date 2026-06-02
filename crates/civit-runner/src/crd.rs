#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineRunSpec {
    pub name: String,
    pub repo_url: String,
    pub commit_sha: String,
    pub branch: String,
    pub steps: Vec<CrdStep>,
    pub triggers: Vec<String>,
    pub timeout_seconds: u32,
    pub resources: ResourceRequirements,
    pub node_selector: HashMap<String, String>,
    pub tolerations: Vec<Toleration>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrdStep {
    pub name: String,
    pub image: String,
    pub commands: Vec<String>,
    pub env: HashMap<String, String>,
    pub condition: Option<String>,
    pub workdir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceRequirements {
    pub cpu: String,
    pub memory: String,
    pub gpu: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Toleration {
    pub key: String,
    pub operator: String,
    pub value: Option<String>,
    pub effect: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineRunStatus {
    pub phase: RunPhase,
    pub message: String,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub step_statuses: Vec<StepPhaseStatus>,
    pub retry_count: u32,
}

impl PipelineRunStatus {
    pub fn new_pending(message: impl Into<String>) -> Self {
        Self {
            phase: RunPhase::Pending,
            message: message.into(),
            started_at: None,
            finished_at: None,
            step_statuses: vec![],
            retry_count: 0,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.phase,
            RunPhase::Succeeded | RunPhase::Failed | RunPhase::Cancelled
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum RunPhase {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepPhaseStatus {
    pub name: String,
    pub phase: RunPhase,
    pub exit_code: Option<i32>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_spec() -> PipelineRunSpec {
        PipelineRunSpec {
            name: "test-run".into(),
            repo_url: "https://github.com/example/repo".into(),
            commit_sha: "abc123".into(),
            branch: "main".into(),
            steps: vec![CrdStep {
                name: "build".into(),
                image: "alpine:latest".into(),
                commands: vec!["make".into()],
                env: HashMap::new(),
                condition: None,
                workdir: Some("/src".into()),
            }],
            triggers: vec!["push".into()],
            timeout_seconds: 600,
            resources: ResourceRequirements {
                cpu: "500m".into(),
                memory: "256Mi".into(),
                gpu: None,
            },
            node_selector: HashMap::new(),
            tolerations: vec![],
        }
    }

    #[test]
    fn test_pipeline_run_spec_serialization() {
        let spec = sample_spec();
        let json = serde_json::to_string(&spec).unwrap();
        let de: PipelineRunSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(de.name, "test-run");
        assert_eq!(de.commit_sha, "abc123");
        assert_eq!(de.steps.len(), 1);
        assert_eq!(de.steps[0].workdir.as_deref(), Some("/src"));
    }

    #[test]
    fn test_resource_requirements_with_gpu() {
        let rr = ResourceRequirements {
            cpu: "2".into(),
            memory: "4Gi".into(),
            gpu: Some("nvidia.com/gpu=1".into()),
        };
        let json = serde_json::to_string(&rr).unwrap();
        let de: ResourceRequirements = serde_json::from_str(&json).unwrap();
        assert_eq!(de.gpu.as_deref(), Some("nvidia.com/gpu=1"));
    }

    #[test]
    fn test_toleration_serialization() {
        let tol = Toleration {
            key: "dedicated".into(),
            operator: "Equal".into(),
            value: Some("gpu".into()),
            effect: "NoSchedule".into(),
        };
        let json = serde_json::to_string(&tol).unwrap();
        let de: Toleration = serde_json::from_str(&json).unwrap();
        assert_eq!(de.value.as_deref(), Some("gpu"));
        assert_eq!(de.effect, "NoSchedule");
    }

    #[test]
    fn test_run_phase_serialization() {
        let phases = vec![
            RunPhase::Pending,
            RunPhase::Running,
            RunPhase::Succeeded,
            RunPhase::Failed,
            RunPhase::Cancelled,
        ];
        for phase in phases {
            let json = serde_json::to_string(&phase).unwrap();
            let de: RunPhase = serde_json::from_str(&json).unwrap();
            assert_eq!(phase, de);
        }
    }

    #[test]
    fn test_pipeline_run_status_new_pending() {
        let status = PipelineRunStatus::new_pending("queued");
        assert_eq!(status.phase, RunPhase::Pending);
        assert_eq!(status.message, "queued");
        assert!(!status.is_terminal());
    }

    #[test]
    fn test_pipeline_run_status_is_terminal() {
        for phase in [RunPhase::Succeeded, RunPhase::Failed, RunPhase::Cancelled] {
            let mut status = PipelineRunStatus::new_pending("");
            status.phase = phase;
            assert!(status.is_terminal());
        }
        let mut status = PipelineRunStatus::new_pending("");
        status.phase = RunPhase::Running;
        assert!(!status.is_terminal());
    }

    #[test]
    fn test_step_phase_status() {
        let s = StepPhaseStatus {
            name: "test".into(),
            phase: RunPhase::Succeeded,
            exit_code: Some(0),
            started_at: Some(Utc::now()),
            finished_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&s).unwrap();
        let de: StepPhaseStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(de.exit_code, Some(0));
    }

    #[test]
    fn test_node_selector_roundtrip() {
        let mut spec = sample_spec();
        spec.node_selector.insert("disktype".into(), "ssd".into());
        let json = serde_json::to_string(&spec).unwrap();
        let de: PipelineRunSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(de.node_selector.get("disktype").unwrap(), "ssd");
    }
}
