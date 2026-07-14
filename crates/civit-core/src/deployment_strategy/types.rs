use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StrategyType {
    #[serde(rename = "rolling")]
    Rolling,
    #[serde(rename = "blue_green")]
    BlueGreen,
    #[serde(rename = "canary")]
    Canary,
    #[serde(rename = "recreate")]
    Recreate,
}

impl std::fmt::Display for StrategyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rolling => write!(f, "rolling"),
            Self::BlueGreen => write!(f, "blue_green"),
            Self::Canary => write!(f, "canary"),
            Self::Recreate => write!(f, "recreate"),
        }
    }
}

impl std::str::FromStr for StrategyType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rolling" => Ok(Self::Rolling),
            "blue_green" => Ok(Self::BlueGreen),
            "canary" => Ok(Self::Canary),
            "recreate" => Ok(Self::Recreate),
            _ => Err(format!("unknown strategy type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStrategy {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub strategy_type: StrategyType,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStrategyRequest {
    pub repo_id: Uuid,
    pub name: String,
    pub strategy_type: StrategyType,
    pub config: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStrategyRequest {
    pub name: Option<String>,
    pub strategy_type: Option<StrategyType>,
    pub config: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub id: Uuid,
    pub strategy_id: Uuid,
    pub config_key: String,
    pub config_value: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStrategyConfigRequest {
    pub config_key: String,
    pub config_value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyLog {
    pub id: Uuid,
    pub strategy_id: Uuid,
    pub action: String,
    pub status: String,
    pub details: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStrategyLogRequest {
    pub action: String,
    pub status: Option<String>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyTestResult {
    pub strategy_id: Uuid,
    pub test_name: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyComparison {
    pub strategy_a_id: Uuid,
    pub strategy_b_id: Uuid,
    pub metrics: Vec<StrategyComparisonMetric>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyComparisonMetric {
    pub metric_name: String,
    pub strategy_a_value: f64,
    pub strategy_b_value: f64,
    pub winner: String,
}
