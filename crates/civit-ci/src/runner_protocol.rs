//! Runner communication protocol for CI/CD pipelines.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobAssignment {
    pub run_job_id: String,
    pub run_id: String,
    pub repo_id: String,
    pub ref_name: String,
    pub commit_sha: String,
    pub steps: Vec<StepSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepSpec {
    pub step_id: String,
    pub name: String,
    pub step_index: i32,
    pub step_type: String,
    pub commands: Option<serde_json::Value>,
    pub action: Option<String>,
    pub action_params: Option<serde_json::Value>,
    pub image: Option<String>,
    pub workdir: Option<String>,
    pub env: Option<serde_json::Value>,
    pub secrets: Option<serde_json::Value>,
    pub continue_on_error: bool,
    pub timeout: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusUpdate {
    pub run_job_id: String,
    pub status: String,
    pub runner_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub run_job_id: String,
    pub step_index: Option<i32>,
    pub line: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepComplete {
    pub run_job_id: String,
    pub step_id: String,
    pub status: String,
    pub exit_code: Option<i32>,
    pub output: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_assignment_serialize() {
        let job = JobAssignment {
            run_job_id: "j1".into(),
            run_id: "r1".into(),
            repo_id: "repo1".into(),
            ref_name: "main".into(),
            commit_sha: "abc123".into(),
            steps: vec![],
        };
        let json = serde_json::to_string(&job).unwrap();
        assert!(json.contains("run_job_id"));
    }

    #[test]
    fn test_status_update_serialize() {
        let update = StatusUpdate {
            run_job_id: "j1".into(),
            status: "running".into(),
            runner_id: Some("runner-1".into()),
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("running"));
    }
}
