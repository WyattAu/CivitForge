#![forbid(unsafe_code)]

use crate::error::CoreError;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,
    pub federation_enabled: bool,
    pub federation_instance_id: String,
    pub federation_instance_domain: String,
}

impl AppConfig {
    pub fn from_env() -> crate::error::Result<Self> {
        Ok(Self {
            host: std::env::var("CIVIT_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            port: std::env::var("CIVIT_PORT")
                .unwrap_or_else(|_| "8080".into())
                .parse()
                .map_err(|_| CoreError::Config("invalid port".into()))?,
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| CoreError::Config("DATABASE_URL required".into()))?,
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            jwt_secret: std::env::var("JWT_SECRET")
                .map_err(|_| CoreError::Config("JWT_SECRET required".into()))?,
            jwt_expiry_hours: std::env::var("JWT_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".into())
                .parse()
                .map_err(|_| CoreError::Config("invalid JWT expiry".into()))?,
            federation_enabled: std::env::var("FEDERATION_ENABLED")
                .unwrap_or_else(|_| "false".into())
                .parse()
                .map_err(|_| CoreError::Config("invalid federation flag".into()))?,
            federation_instance_id: std::env::var("FEDERATION_INSTANCE_ID")
                .unwrap_or_else(|_| "default-instance".into()),
            federation_instance_domain: std::env::var("FEDERATION_INSTANCE_DOMAIN")
                .unwrap_or_else(|_| "localhost".into()),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl FromStr for LogLevel {
    type Err = CoreError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(Self::Trace),
            "debug" => Ok(Self::Debug),
            "info" => Ok(Self::Info),
            "warn" => Ok(Self::Warn),
            "error" => Ok(Self::Error),
            _ => Err(CoreError::Config(format!("unknown log level: {s}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("trace").unwrap(), LogLevel::Trace);
        assert_eq!(LogLevel::from_str("INFO").unwrap(), LogLevel::Info);
        assert_eq!(LogLevel::from_str("Warn").unwrap(), LogLevel::Warn);
        assert!(LogLevel::from_str("unknown").is_err());
    }

    #[test]
    fn test_log_level_roundtrip() {
        for level in [
            LogLevel::Trace,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ] {
            let s = format!("{level:?}");
            assert_eq!(LogLevel::from_str(&s.to_lowercase()).unwrap(), level);
        }
    }
}
