//! Standalone CI/CD Runner — HTTP client for the CivitForge API.
//!
//! Handles runner registration, job polling, claiming, heartbeat,
//! step status updates, and job completion.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// API types matching the server responses
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollJobResponse {
    pub job_id: String,
    pub run_id: String,
    pub name: String,
    pub steps: Vec<JobStepSpec>,
    pub repo_url: String,
    pub commit_sha: String,
    pub ref_name: String,
    pub env: serde_json::Value,
    pub secret_names: Vec<String>,
    pub services: serde_json::Value,
    pub timeout: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStepSpec {
    pub name: String,
    pub step_index: i32,
    pub step_type: String,
    pub commands: Option<Vec<String>>,
    pub action: Option<String>,
    pub action_params: Option<serde_json::Value>,
    pub image: Option<String>,
    pub workdir: String,
    pub env: serde_json::Value,
    pub continue_on_error: bool,
    pub timeout: Option<String>,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateStepRequest {
    pub status: String,
    pub exit_code: Option<i32>,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateJobRequest {
    pub status: String,
    pub outputs: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRunnerRequest {
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_scope")]
    pub scope: String,
    pub labels: Vec<String>,
    pub runner_group: Option<String>,
    pub token: Option<String>,
}

fn default_scope() -> String {
    "global".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRunnerResponse {
    pub id: String,
    pub name: String,
    pub token: String,
}

// ---------------------------------------------------------------------------
// Runner client
// ---------------------------------------------------------------------------

/// HTTP client for the CivitForge runner API.
#[derive(Debug, Clone)]
pub struct RunnerClient {
    base_url: String,
    http: reqwest::Client,
    runner_id: String,
    token: String,
}

impl RunnerClient {
    /// Create a new runner client.
    pub fn new(
        base_url: impl Into<String>,
        runner_id: impl Into<String>,
        token: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            http: reqwest::Client::new(),
            runner_id: runner_id.into(),
            token: token.into(),
        }
    }

    /// Register a new runner with the server. Returns (runner_id, token).
    pub async fn register(
        base_url: &str,
        name: &str,
        labels: &[&str],
        group: Option<&str>,
    ) -> anyhow::Result<(String, String)> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/runners", base_url.trim_end_matches('/'));

        let body = RegisterRunnerRequest {
            name: name.to_string(),
            description: None,
            scope: "global".to_string(),
            labels: labels.iter().map(|s| s.to_string()).collect(),
            runner_group: group.map(|s| s.to_string()),
            token: None,
        };

        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let result: RegisterRunnerResponse = resp.json().await?;
        Ok((result.id, result.token))
    }

    /// Poll for an available job.
    pub async fn poll_job(&self) -> anyhow::Result<Option<PollJobResponse>> {
        let url = format!("{}/api/v1/runners/poll", self.base_url);

        let body = serde_json::json!({
            "token": self.token,
        });

        let resp = self.http.post(&url).json(&body).send().await?;

        let status = resp.status();
        if status == reqwest::StatusCode::NO_CONTENT {
            return Ok(None);
        }

        let resp = resp.error_for_status()?;
        let job: PollJobResponse = resp.json().await?;
        Ok(Some(job))
    }

    /// Claim a specific job.
    pub async fn claim_job(&self, job_id: &str) -> anyhow::Result<bool> {
        let url = format!(
            "{}/api/v1/runners/{}/claim/{}",
            self.base_url, self.runner_id, job_id
        );

        let resp = self.http.post(&url).send().await?;
        let status = resp.status();

        if status == reqwest::StatusCode::CONFLICT {
            return Ok(false);
        }

        resp.error_for_status()?;
        Ok(true)
    }

    /// Send heartbeat.
    pub async fn heartbeat(&self) -> anyhow::Result<()> {
        let url = format!(
            "{}/api/v1/runners/{}/heartbeat",
            self.base_url, self.runner_id
        );

        self.http.post(&url).send().await?.error_for_status()?;
        Ok(())
    }

    /// Update step status.
    pub async fn update_step(
        &self,
        step_id: &str,
        status: &str,
        exit_code: Option<i32>,
        output: Option<String>,
    ) -> anyhow::Result<()> {
        let url = format!(
            "{}/api/v1/runners/{}/steps/{}",
            self.base_url, self.runner_id, step_id
        );

        let body = UpdateStepRequest {
            status: status.to_string(),
            exit_code,
            output,
        };

        self.http
            .post(&url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Mark a job as completed.
    pub async fn complete_job(
        &self,
        job_id: &str,
        status: &str,
        outputs: Option<serde_json::Value>,
    ) -> anyhow::Result<()> {
        let url = format!(
            "{}/api/v1/runners/{}/jobs/{}/complete",
            self.base_url, self.runner_id, job_id
        );

        let body = UpdateJobRequest {
            status: status.to_string(),
            outputs,
        };

        self.http
            .post(&url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Resolve secrets for a job.
    ///
    /// Fetches secret values from the server for the given repo and secret names.
    pub async fn resolve_secrets(
        &self,
        repo_url: &str,
        secret_names: &[String],
    ) -> anyhow::Result<std::collections::HashMap<String, String>> {
        if secret_names.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let url = format!(
            "{}/api/v1/runners/{}/secrets",
            self.base_url, self.runner_id
        );

        let body = serde_json::json!({
            "repo_id": repo_url,
            "secret_names": secret_names,
            "token": self.token,
        });

        let resp = self.http.post(&url).json(&body).send().await?;
        let data: serde_json::Value = resp.error_for_status()?.json().await?;

        let mut secrets = std::collections::HashMap::new();
        if let serde_json::Value::Object(map) = data {
            for (k, v) in map {
                if let serde_json::Value::String(s) = v {
                    secrets.insert(k.clone(), s.clone());
                }
            }
        }

        Ok(secrets)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_scope() {
        assert_eq!(default_scope(), "global");
    }

    #[test]
    fn test_register_runner_request_serialize() {
        let req = RegisterRunnerRequest {
            name: "linux-runner".to_string(),
            description: Some("CI runner".to_string()),
            scope: default_scope(),
            labels: vec!["linux".to_string(), "amd64".to_string()],
            runner_group: Some("default".to_string()),
            token: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("linux-runner"));
        assert!(json.contains("linux"));
        assert!(json.contains("default"));
    }

    #[test]
    fn test_update_step_request_serialize() {
        let req = UpdateStepRequest {
            status: "success".to_string(),
            exit_code: Some(0),
            output: Some("done".to_string()),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("success"));
    }

    #[test]
    fn test_update_job_request_serialize() {
        let req = UpdateJobRequest {
            status: "failure".to_string(),
            outputs: Some(serde_json::json!({"key": "val"})),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("failure"));
    }

    #[test]
    fn test_poll_job_response_deserialize() {
        let json = r#"{
            "job_id": "00000000-0000-0000-0000-000000000001",
            "run_id": "00000000-0000-0000-0000-000000000002",
            "name": "build",
            "steps": [],
            "repo_url": "/alice/repo.git",
            "commit_sha": "abc123",
            "ref_name": "main",
            "env": {},
            "secret_names": [],
            "services": null,
            "timeout": null
        }"#;
        let job: PollJobResponse = serde_json::from_str(json).unwrap();
        assert_eq!(job.name, "build");
        assert_eq!(job.commit_sha, "abc123");
        assert_eq!(job.ref_name, "main");
    }

    #[test]
    fn test_job_step_spec_roundtrip() {
        let spec = JobStepSpec {
            name: "test".to_string(),
            step_index: 0,
            step_type: "run".to_string(),
            commands: Some(vec!["cargo test".to_string()]),
            action: None,
            action_params: None,
            image: Some("rust:1.75".to_string()),
            workdir: "/src".to_string(),
            env: serde_json::json!({}),
            continue_on_error: false,
            timeout: Some("30m".to_string()),
            condition: None,
        };
        let json = serde_json::to_string(&spec).unwrap();
        let decoded: JobStepSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, "test");
        assert_eq!(decoded.commands.unwrap().len(), 1);
    }

    #[test]
    fn test_runner_client_base_url_trim() {
        let client = RunnerClient::new("http://localhost:8080/", "runner-id", "token");
        assert_eq!(client.base_url, "http://localhost:8080");
    }

    #[test]
    fn test_register_runner_response_deserialize() {
        let json = r#"{"id": "runner-123", "name": "test-runner", "token": "secret-token"}"#;
        let resp: RegisterRunnerResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.id, "runner-123");
        assert_eq!(resp.token, "secret-token");
    }
}
