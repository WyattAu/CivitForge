use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CircuitBreakerState {
    #[serde(rename = "closed")]
    Closed,
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "half_open")]
    HalfOpen,
}

impl std::fmt::Display for CircuitBreakerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Closed => write!(f, "closed"),
            Self::Open => write!(f, "open"),
            Self::HalfOpen => write!(f, "half_open"),
        }
    }
}

impl std::str::FromStr for CircuitBreakerState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "closed" => Ok(Self::Closed),
            "open" => Ok(Self::Open),
            "half_open" => Ok(Self::HalfOpen),
            _ => Err(format!("unknown circuit breaker state: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub name: String,
    pub failure_threshold: i32,
    pub success_threshold: i32,
    pub timeout_seconds: i32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            failure_threshold: 5,
            success_threshold: 3,
            timeout_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerMetrics {
    pub id: Uuid,
    pub name: String,
    pub state: CircuitBreakerState,
    pub failure_count: i32,
    pub failure_threshold: i32,
    pub success_threshold: i32,
    pub timeout_seconds: i32,
    pub last_failure_at: Option<DateTime<Utc>>,
    pub last_state_change: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerStatus {
    pub name: String,
    pub state: CircuitBreakerState,
    pub failure_count: i32,
    pub success_count: i32,
    pub last_failure_at: Option<DateTime<Utc>>,
    pub last_state_change: DateTime<Utc>,
}
