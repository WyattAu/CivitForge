use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TestStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
}

impl std::fmt::Display for TestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for TestStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => Err(format!("unknown test status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TestType {
    #[serde(rename = "retry")]
    Retry,
    #[serde(rename = "timeout")]
    Timeout,
    #[serde(rename = "fallback")]
    Fallback,
    #[serde(rename = "bulkhead")]
    Bulkhead,
    #[serde(rename = "rate_limit")]
    RateLimit,
}

impl std::fmt::Display for TestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Retry => write!(f, "retry"),
            Self::Timeout => write!(f, "timeout"),
            Self::Fallback => write!(f, "fallback"),
            Self::Bulkhead => write!(f, "bulkhead"),
            Self::RateLimit => write!(f, "rate_limit"),
        }
    }
}

impl std::str::FromStr for TestType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "retry" => Ok(Self::Retry),
            "timeout" => Ok(Self::Timeout),
            "fallback" => Ok(Self::Fallback),
            "bulkhead" => Ok(Self::Bulkhead),
            "rate_limit" => Ok(Self::RateLimit),
            _ => Err(format!("unknown test type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilienceTest {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub test_type: TestType,
    pub target: String,
    pub parameters: serde_json::Value,
    pub status: TestStatus,
    pub score: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTestRequest {
    pub name: String,
    pub description: Option<String>,
    pub test_type: TestType,
    pub target: String,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_id: Uuid,
    pub score: i32,
    pub recommendations: Vec<String>,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilienceScore {
    pub overall: i32,
    pub retry_score: i32,
    pub timeout_score: i32,
    pub fallback_score: i32,
    pub bulkhead_score: i32,
    pub rate_limit_score: i32,
}
