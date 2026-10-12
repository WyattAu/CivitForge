#![forbid(unsafe_code)]

pub mod store;
pub mod types;

pub use store::DeploymentStrategyStore;
pub use types::{
    CreateStrategyConfigRequest, CreateStrategyLogRequest, CreateStrategyRequest,
    DeploymentStrategy, StrategyComparison, StrategyComparisonMetric, StrategyConfig, StrategyLog,
    StrategyTestResult, StrategyType, UpdateStrategyRequest,
};
