//! Pipeline model types matching the YAML spec.

use chrono::Duration;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Top-level Pipeline
// ---------------------------------------------------------------------------

/// Parsed `.civit/pipeline.yaml` document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    /// Schema version. Must be "1".
    pub version: String,
    /// Trigger configuration.
    pub on: Option<TriggerConfig>,
    /// Global environment variables.
    pub env: Option<Vec<EnvVar>>,
    /// Concurrency control.
    pub concurrency: Option<Concurrency>,
    /// Workspace sharing mode.
    pub workspace: Option<Workspace>,
    /// Pipeline-level variables (template references).
    pub variables: Option<Vec<VariableDef>>,
    /// Secrets needed (resolved from pipeline_variables DB).
    pub secrets: Option<Vec<String>>,
    /// Pipeline jobs.
    pub jobs: Vec<Job>,
}

// ---------------------------------------------------------------------------
// Triggers
// ---------------------------------------------------------------------------

/// Trigger configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerConfig {
    pub push: Option<PushTrigger>,
    pub pull_request: Option<PrTrigger>,
    pub schedule: Option<Vec<ScheduleTrigger>>,
    pub workflow_dispatch: Option<WorkflowDispatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushTrigger {
    pub branches: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub paths: Option<Vec<String>>,
    #[serde(rename = "paths_ignore")]
    pub paths_ignore: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrTrigger {
    pub branches: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTrigger {
    /// Cron expression (e.g. "0 6 * * 1" = every Monday 6am).
    pub cron: String,
    /// Human-readable name.
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDispatch {
    pub inputs: Option<Vec<DispatchInput>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchInput {
    pub name: String,
    #[serde(rename = "type")]
    pub input_type: DispatchInputType,
    pub required: Option<bool>,
    pub default: Option<serde_yaml::Value>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DispatchInputType {
    String,
    Boolean,
    Number,
    Selection,
}

// ---------------------------------------------------------------------------
// Concurrency
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concurrency {
    /// Group key (supports expressions).
    pub group: Option<String>,
    /// Cancel in-progress runs in same group.
    #[serde(default)]
    pub cancel_in_progress: bool,
}

// ---------------------------------------------------------------------------
// Workspace
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WorkspaceSharing {
    #[serde(rename = "shared")]
    Shared,
    #[serde(rename = "isolated")]
    #[default]
    Isolated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    #[serde(default)]
    pub sharing: WorkspaceSharing,
}

// ---------------------------------------------------------------------------
// Environment Variables
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub value: Option<String>,
    /// Secret name to resolve from pipeline_variables.
    #[serde(rename = "from_secret")]
    pub from_secret: Option<String>,
    pub description: Option<String>,
}

// ---------------------------------------------------------------------------
// Variables (template references)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDef {
    pub name: String,
    pub value: Option<serde_yaml::Value>,
    pub description: Option<String>,
}

// ---------------------------------------------------------------------------
// Jobs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Job identifier.
    pub name: String,
    /// Job-level condition (CEL expression).
    #[serde(rename = "if")]
    pub condition: Option<String>,
    /// Dependencies (other job names).
    pub needs: Option<Vec<String>>,
    /// Target runner.
    #[serde(rename = "runs-on")]
    pub runs_on: Option<RunsOn>,
    /// Matrix strategy for expanding this job into multiple variants.
    pub strategy: Option<Strategy>,
    /// Time limit.
    pub timeout: Option<JobTimeout>,
    /// Job-level environment.
    pub env: Option<Vec<EnvVar>>,
    /// Job-level secrets.
    pub secrets: Option<Vec<String>>,
    /// Service containers (sidecars).
    pub services: Option<Vec<Service>>,
    /// Job steps.
    pub steps: Vec<Step>,
}

/// Target runner specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunsOn {
    pub labels: Option<Vec<String>>,
    pub group: Option<String>,
}

/// Job timeout — parsed into Duration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JobTimeout {
    /// Parsed from string like "30m", "2h", "300s".
    String(String),
}

impl std::fmt::Display for JobTimeout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "{s}"),
        }
    }
}

impl JobTimeout {
    /// Parse into a Duration. Returns None on parse failure.
    pub fn to_duration(&self) -> Option<Duration> {
        let s = match self {
            Self::String(s) => s.as_str(),
        };
        let s = s.trim();

        // Try seconds: "300s"
        if let Some(n) = s.strip_suffix('s') {
            let secs: i64 = n.trim().parse().ok()?;
            return Duration::try_seconds(secs);
        }
        // Try minutes: "30m"
        if let Some(n) = s.strip_suffix('m') {
            let mins: i64 = n.trim().parse().ok()?;
            return Duration::try_minutes(mins);
        }
        // Try hours: "2h"
        if let Some(n) = s.strip_suffix('h') {
            let hours: i64 = n.trim().parse().ok()?;
            return Duration::try_hours(hours);
        }

        None
    }
}

// ---------------------------------------------------------------------------
// Steps
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    /// Step name.
    pub name: String,
    /// Step description (used in UI).
    pub description: Option<String>,
    /// Container image.
    pub image: Option<String>,
    /// Shell commands to run.
    pub run: Option<Vec<String>>,
    /// Single command string.
    #[serde(rename = "shell")]
    pub shell: Option<String>,
    /// Working directory.
    #[serde(default)]
    pub workdir: String,
    /// Step-level environment.
    pub env: Option<Vec<EnvVar>>,
    /// Step-level secrets.
    pub secrets: Option<Vec<String>>,
    /// Continue on error.
    #[serde(default)]
    #[serde(rename = "continue_on_error")]
    pub continue_on_error: bool,
    /// Time limit for this step.
    pub timeout: Option<JobTimeout>,
    /// Step condition (CEL expression).
    #[serde(rename = "if")]
    pub condition: Option<String>,
    /// Use action.
    pub uses: Option<StepUses>,
    /// Checkout action.
    pub checkout: Option<CheckoutConfig>,
    /// Cache action.
    pub cache: Option<CacheConfig>,
    /// Artifact action.
    pub artifact: Option<ArtifactConfig>,
    /// Retry policy.
    pub retry: Option<RetryConfig>,
}

/// Built-in step actions (uses:).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepUses {
    pub action: StepAction,
    pub with: Option<serde_yaml::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepAction {
    Checkout,
    Cache,
    Artifact,
}

/// Checkout configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutConfig {
    pub fetch_depth: Option<u32>,
    pub submodules: Option<bool>,
    #[serde(rename = "lfs")]
    pub lfs: Option<bool>,
}

/// Cache configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Cache key (supports expressions).
    pub key: Option<String>,
    /// Path(s) to cache.
    pub path: Option<Vec<String>>,
    /// Upload/download mode.
    pub action: Option<String>,
}

/// Artifact configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactConfig {
    pub name: String,
    pub path: Option<Vec<String>>,
    pub retention: Option<String>,
    #[serde(rename = "if_no_files_found")]
    pub if_no_files_found: Option<String>,
}

/// Retry configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max: Option<u32>,
    pub wait: Option<String>,
}

// ---------------------------------------------------------------------------
// Services (sidecar containers)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    /// Service identifier (DNS name within job network).
    pub name: String,
    /// Container image.
    pub image: String,
    /// Service environment.
    pub env: Option<Vec<EnvVar>>,
    /// Service secrets.
    pub secrets: Option<Vec<String>>,
    /// Port mappings.
    pub ports: Option<Vec<ServicePort>>,
    /// Health check.
    pub health_check: Option<HealthCheck>,
    /// Additional container options.
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePort {
    pub port: u16,
    pub protocol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub command: Option<Vec<String>>,
    pub interval: Option<String>,
    pub timeout: Option<String>,
    pub retries: Option<u32>,
}

// ---------------------------------------------------------------------------
// Expression (CEL subset)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Matrix Builds
// ---------------------------------------------------------------------------

/// Strategy configuration for matrix builds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    /// Matrix dimensions defining the cross-product of values.
    #[serde(default)]
    pub matrix: MatrixConfig,
    /// Whether to cancel in-progress jobs when a matrix job fails (default: true).
    #[serde(default = "default_true")]
    pub fail_fast: bool,
    /// Maximum number of concurrent matrix jobs (None = unlimited).
    pub max_parallel: Option<u32>,
}

fn default_true() -> bool {
    true
}

/// Matrix dimension values and inclusion/exclusion rules.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MatrixConfig {
    /// Dynamic dimension keys mapping to lists of values.
    /// e.g. `os: [ubuntu-latest, macos-latest]`, `rust: [stable, nightly]`
    #[serde(flatten)]
    pub dimensions: std::collections::HashMap<String, Vec<String>>,
    /// Extra combinations to add after the cross-product.
    #[serde(default)]
    pub include: Vec<serde_yaml::Value>,
    /// Combinations to remove from the cross-product.
    #[serde(default)]
    pub exclude: Vec<serde_yaml::Value>,
}

// ---------------------------------------------------------------------------
// Pipeline Variables (extended)
// ---------------------------------------------------------------------------

/// Variable scope — determines when a variable is available.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum VariableScope {
    /// Available to all jobs in the repository.
    #[default]
    Repo,
    /// Available only on matching branches.
    Branch,
    /// Available only on matching pull requests.
    Pr,
}

// ---------------------------------------------------------------------------
// Expression (CEL subset)
// ---------------------------------------------------------------------------

/// A CEL-like expression used in `if:` conditions and triggers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PipelineExpression {
    String(String),
}
