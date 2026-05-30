#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSpec {
    pub name: String,
    pub triggers: Vec<String>,
    pub steps: Vec<PipelineStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    pub name: String,
    pub image: String,
    pub commands: Vec<String>,
    pub env: HashMap<String, String>,
    pub condition: Option<StepCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StepCondition {
    Always,
    OnSuccess,
    OnFailure,
    Branch { branches: Vec<String> },
    EnvVar { key: String, value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStatus {
    pub pipeline: String,
    pub status: String,
    pub step_results: Vec<StepStatus>,
    pub started_at: String,
    pub finished_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepStatus {
    pub name: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: String,
    pub output: String,
}

impl PipelineStatus {
    pub fn is_success(&self) -> bool {
        self.status == "success"
    }

    pub fn is_failed(&self) -> bool {
        self.status == "failed"
    }

    pub fn duration(&self) -> std::result::Result<chrono::Duration, chrono::ParseError> {
        let start = chrono::DateTime::parse_from_rfc3339(&self.started_at)?;
        let end = chrono::DateTime::parse_from_rfc3339(&self.finished_at)?;
        Ok(end.signed_duration_since(start))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_status(status: &str) -> PipelineStatus {
        PipelineStatus {
            pipeline: "test".into(),
            status: status.into(),
            step_results: vec![],
            started_at: "2025-01-01T00:00:00+00:00".into(),
            finished_at: "2025-01-01T00:05:00+00:00".into(),
        }
    }

    #[test]
    fn test_pipeline_status_success() {
        assert!(make_status("success").is_success());
        assert!(!make_status("success").is_failed());
    }

    #[test]
    fn test_pipeline_status_failed() {
        assert!(make_status("failed").is_failed());
        assert!(!make_status("failed").is_success());
    }

    #[test]
    fn test_pipeline_status_duration() {
        let status = make_status("success");
        let dur = status.duration().unwrap();
        assert_eq!(dur.num_minutes(), 5);
    }

    #[test]
    fn test_pipeline_spec_serialization() {
        let spec = PipelineSpec {
            name: "ci".into(),
            triggers: vec!["push".into()],
            steps: vec![PipelineStep {
                name: "build".into(),
                image: "alpine".into(),
                commands: vec!["make".into()],
                env: HashMap::new(),
                condition: None,
            }],
        };
        let json = serde_json::to_string(&spec).unwrap();
        let de: PipelineSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(de.name, "ci");
        assert_eq!(de.steps[0].name, "build");
    }

    #[test]
    fn test_step_condition_branch() {
        let cond = StepCondition::Branch {
            branches: vec!["main".into(), "dev".into()],
        };
        let json = serde_json::to_string(&cond).unwrap();
        let de: StepCondition = serde_json::from_str(&json).unwrap();
        match de {
            StepCondition::Branch { branches } => {
                assert_eq!(branches, vec!["main", "dev"]);
            }
            _ => panic!("expected Branch"),
        }
    }
}
