#![forbid(unsafe_code)]

pub mod breaker;
pub mod types;

pub use breaker::CircuitBreaker;
pub use types::{CircuitBreakerConfig, CircuitBreakerMetrics, CircuitBreakerState};
