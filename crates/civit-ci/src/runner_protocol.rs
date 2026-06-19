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

    #[test]
    fn test_step_spec_serialize() {
        let step = StepSpec {
            step_id: "s1".into(),
            name: "compile".into(),
            step_index: 0,
            step_type: "run".into(),
            commands: Some(serde_json::json!({"run": "cargo build"})),
            action: None,
            action_params: None,
            image: Some("rust:latest".into()),
            workdir: None,
            env: None,
            secrets: None,
            continue_on_error: false,
            timeout: None,
        };
        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("compile"));
        assert!(json.contains("cargo build"));
    }

    #[test]
    fn test_step_spec_all_optionals_none() {
        let step = StepSpec {
            step_id: "s1".into(),
            name: "test".into(),
            step_index: 1,
            step_type: "run".into(),
            commands: None,
            action: None,
            action_params: None,
            image: None,
            workdir: None,
            env: None,
            secrets: None,
            continue_on_error: false,
            timeout: None,
        };
        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("test"));
    }

    #[test]
    fn test_log_entry_serialize() {
        let entry = LogEntry {
            run_job_id: "j1".into(),
            step_index: Some(0),
            line: "Compiling...".into(),
            timestamp: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("Compiling"));
    }

    #[test]
    fn test_log_entry_no_step_index() {
        let entry = LogEntry {
            run_job_id: "j1".into(),
            step_index: None,
            line: "Starting job".into(),
            timestamp: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("Starting job"));
    }

    #[test]
    fn test_step_complete_serialize() {
        let complete = StepComplete {
            run_job_id: "j1".into(),
            step_id: "s1".into(),
            status: "success".into(),
            exit_code: Some(0),
            output: Some("Build successful".into()),
        };
        let json = serde_json::to_string(&complete).unwrap();
        assert!(json.contains("success"));
        assert!(json.contains("Build successful"));
    }

    #[test]
    fn test_step_complete_no_output() {
        let complete = StepComplete {
            run_job_id: "j1".into(),
            step_id: "s1".into(),
            status: "failed".into(),
            exit_code: Some(1),
            output: None,
        };
        let json = serde_json::to_string(&complete).unwrap();
        assert!(json.contains("failed"));
        assert!(json.contains("null"));
    }

    #[test]
    fn test_job_assignment_with_steps() {
        let job = JobAssignment {
            run_job_id: "j1".into(),
            run_id: "r1".into(),
            repo_id: "repo1".into(),
            ref_name: "main".into(),
            commit_sha: "abc123".into(),
            steps: vec![
                StepSpec {
                    step_id: "s1".into(),
                    name: "build".into(),
                    step_index: 0,
                    step_type: "run".into(),
                    commands: None,
                    action: None,
                    action_params: None,
                    image: None,
                    workdir: None,
                    env: None,
                    secrets: None,
                    continue_on_error: false,
                    timeout: None,
                },
                StepSpec {
                    step_id: "s2".into(),
                    name: "test".into(),
                    step_index: 1,
                    step_type: "uses".into(),
                    commands: None,
                    action: Some("checkout".into()),
                    action_params: Some(serde_json::json!({"path": "."})),
                    image: None,
                    workdir: None,
                    env: None,
                    secrets: None,
                    continue_on_error: true,
                    timeout: Some("30m".into()),
                },
            ],
        };
        let json = serde_json::to_string(&job).unwrap();
        assert!(json.contains("build"));
        assert!(json.contains("test"));
        assert!(json.contains("checkout"));
    }

    #[test]
    fn test_status_update_no_runner() {
        let update = StatusUpdate {
            run_job_id: "j1".into(),
            status: "completed".into(),
            runner_id: None,
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("null"));
    }

    #[test]
    fn test_log_entry_empty_line() {
        let entry = LogEntry {
            run_job_id: "j1".into(),
            step_index: Some(0),
            line: "".into(),
            timestamp: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"line\":\"\""));
    }

    #[test]
    fn test_step_complete_negative_exit_code() {
        let complete = StepComplete {
            run_job_id: "j1".into(),
            step_id: "s1".into(),
            status: "failed".into(),
            exit_code: Some(-1),
            output: Some("signal killed".into()),
        };
        let json = serde_json::to_string(&complete).unwrap();
        assert!(json.contains("-1"));
    }
}
