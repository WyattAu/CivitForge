#![forbid(unsafe_code)]

use std::fmt;

use serde::{Deserialize, Serialize};

/// Common status for workflow runs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Skipped,
}

impl fmt::Display for RunStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunStatus::Pending => write!(f, "pending"),
            RunStatus::Running => write!(f, "running"),
            RunStatus::Completed => write!(f, "completed"),
            RunStatus::Failed => write!(f, "failed"),
            RunStatus::Cancelled => write!(f, "cancelled"),
            RunStatus::Skipped => write!(f, "skipped"),
        }
    }
}

impl std::str::FromStr for RunStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(RunStatus::Pending),
            "running" => Ok(RunStatus::Running),
            "completed" => Ok(RunStatus::Completed),
            "failed" => Ok(RunStatus::Failed),
            "cancelled" => Ok(RunStatus::Cancelled),
            "skipped" => Ok(RunStatus::Skipped),
            other => Err(format!("unknown run status: {other}")),
        }
    }
}

impl RunStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            RunStatus::Completed | RunStatus::Failed | RunStatus::Cancelled
        )
    }
}

/// Common status for rule execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionResult {
    Matched,
    NotMatched,
    Error,
}

impl fmt::Display for ExecutionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionResult::Matched => write!(f, "matched"),
            ExecutionResult::NotMatched => write!(f, "not_matched"),
            ExecutionResult::Error => write!(f, "error"),
        }
    }
}

impl std::str::FromStr for ExecutionResult {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "matched" => Ok(ExecutionResult::Matched),
            "not_matched" => Ok(ExecutionResult::NotMatched),
            "error" => Ok(ExecutionResult::Error),
            other => Err(format!("unknown execution result: {other}")),
        }
    }
}

impl ExecutionResult {
    pub fn is_success(&self) -> bool {
        matches!(self, ExecutionResult::Matched)
    }
}

/// Common status for compliance checks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CheckStatus {
    Pending,
    Running,
    Passed,
    Failed,
    Skipped,
}

impl fmt::Display for CheckStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckStatus::Pending => write!(f, "pending"),
            CheckStatus::Running => write!(f, "running"),
            CheckStatus::Passed => write!(f, "passed"),
            CheckStatus::Failed => write!(f, "failed"),
            CheckStatus::Skipped => write!(f, "skipped"),
        }
    }
}

/// Severity levels used across modules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Low => write!(f, "low"),
            Severity::Medium => write!(f, "medium"),
            Severity::High => write!(f, "high"),
            Severity::Critical => write!(f, "critical"),
        }
    }
}
