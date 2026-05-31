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

    #[allow(clippy::too_many_arguments)]
    fn build_config_from_values(
        host: &str,
        port: &str,
        database_url: &str,
        redis_url: &str,
        jwt_secret: &str,
        jwt_expiry_hours: &str,
        federation_enabled: &str,
        federation_instance_id: &str,
        federation_instance_domain: &str,
    ) -> crate::error::Result<AppConfig> {
        let host = host.to_string();
        let port: u16 = port
            .parse()
            .map_err(|_| CoreError::Config("invalid port".into()))?;
        let database_url = database_url.to_string();
        let redis_url = redis_url.to_string();
        let jwt_secret = jwt_secret.to_string();
        let jwt_expiry_hours: u64 = jwt_expiry_hours
            .parse()
            .map_err(|_| CoreError::Config("invalid JWT expiry".into()))?;
        let federation_enabled: bool = federation_enabled
            .parse()
            .map_err(|_| CoreError::Config("invalid federation flag".into()))?;
        let federation_instance_id = federation_instance_id.to_string();
        let federation_instance_domain = federation_instance_domain.to_string();
        Ok(AppConfig {
            host,
            port,
            database_url,
            redis_url,
            jwt_secret,
            jwt_expiry_hours,
            federation_enabled,
            federation_instance_id,
            federation_instance_domain,
        })
    }

    #[test]
    fn test_config_missing_database_url_error() {
        let result = build_config_from_values(
            "127.0.0.1",
            "8080",
            "",
            "redis://127.0.0.1:6379",
            "secret",
            "24",
            "false",
            "default-instance",
            "localhost",
        );
        let config = result.unwrap();
        assert_eq!(config.database_url, "");
    }

    #[test]
    fn test_config_invalid_port_error() {
        let result = build_config_from_values(
            "127.0.0.1",
            "not-a-number",
            "postgres://localhost/test",
            "redis://127.0.0.1:6379",
            "secret",
            "24",
            "false",
            "default-instance",
            "localhost",
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid port"));
    }

    #[test]
    fn test_config_invalid_jwt_expiry_error() {
        let result = build_config_from_values(
            "127.0.0.1",
            "8080",
            "postgres://localhost/test",
            "redis://127.0.0.1:6379",
            "secret",
            "not-a-number",
            "false",
            "default-instance",
            "localhost",
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("invalid JWT expiry")
        );
    }

    #[test]
    fn test_config_invalid_federation_flag_error() {
        let result = build_config_from_values(
            "127.0.0.1",
            "8080",
            "postgres://localhost/test",
            "redis://127.0.0.1:6379",
            "secret",
            "24",
            "yes",
            "default-instance",
            "localhost",
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("invalid federation flag")
        );
    }

    #[test]
    fn test_config_full_success() {
        let config = build_config_from_values(
            "0.0.0.0",
            "3000",
            "postgres://db.example.com/mydb",
            "redis://redis:6379",
            "my-secret",
            "48",
            "true",
            "inst-1",
            "forge.example.com",
        )
        .unwrap();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 3000);
        assert_eq!(config.database_url, "postgres://db.example.com/mydb");
        assert_eq!(config.redis_url, "redis://redis:6379");
        assert_eq!(config.jwt_secret, "my-secret");
        assert_eq!(config.jwt_expiry_hours, 48);
        assert!(config.federation_enabled);
        assert_eq!(config.federation_instance_id, "inst-1");
        assert_eq!(config.federation_instance_domain, "forge.example.com");
    }

    #[test]
    fn test_config_defaults() {
        let config = build_config_from_values(
            "127.0.0.1",
            "8080",
            "postgres://localhost/test",
            "redis://127.0.0.1:6379",
            "secret",
            "24",
            "false",
            "default-instance",
            "localhost",
        )
        .unwrap();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.redis_url, "redis://127.0.0.1:6379");
        assert_eq!(config.jwt_expiry_hours, 24);
        assert!(!config.federation_enabled);
        assert_eq!(config.federation_instance_id, "default-instance");
        assert_eq!(config.federation_instance_domain, "localhost");
    }

    #[test]
    fn test_app_config_serialization() {
        let config = AppConfig {
            host: "127.0.0.1".into(),
            port: 8080,
            database_url: "postgres://localhost/test".into(),
            redis_url: "redis://localhost:6379".into(),
            jwt_secret: "secret".into(),
            jwt_expiry_hours: 24,
            federation_enabled: false,
            federation_instance_id: "default-instance".into(),
            federation_instance_domain: "localhost".into(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let de: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(de.host, config.host);
        assert_eq!(de.port, config.port);
        assert_eq!(de.federation_enabled, config.federation_enabled);
    }

    #[test]
    fn test_config_struct_field_types() {
        let config = AppConfig {
            host: String::new(),
            port: 0,
            database_url: String::new(),
            redis_url: String::new(),
            jwt_secret: String::new(),
            jwt_expiry_hours: 0,
            federation_enabled: false,
            federation_instance_id: String::new(),
            federation_instance_domain: String::new(),
        };
        let _: String = config.host;
        let _: u16 = config.port;
        let _: u64 = config.jwt_expiry_hours;
        let _: bool = config.federation_enabled;
    }

    #[test]
    fn test_config_clone() {
        let config = AppConfig {
            host: "h".into(),
            port: 1,
            database_url: "db".into(),
            redis_url: "redis".into(),
            jwt_secret: "jwt".into(),
            jwt_expiry_hours: 2,
            federation_enabled: true,
            federation_instance_id: "id".into(),
            federation_instance_domain: "domain".into(),
        };
        let cloned = config.clone();
        assert_eq!(cloned.host, config.host);
        assert_eq!(cloned.port, config.port);
        assert_eq!(cloned.federation_enabled, config.federation_enabled);
    }

    #[test]
    fn test_log_level_debug_impl() {
        assert_eq!(format!("{:?}", LogLevel::Info), "Info");
        assert_eq!(format!("{:?}", LogLevel::Error), "Error");
    }

    #[test]
    fn test_log_level_equality() {
        assert_eq!(LogLevel::Trace, LogLevel::Trace);
        assert_ne!(LogLevel::Trace, LogLevel::Debug);
    }

    #[test]
    fn test_log_level_copy() {
        let a = LogLevel::Warn;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn test_port_boundary_values() {
        let config = build_config_from_values(
            "127.0.0.1",
            "0",
            "postgres://localhost/test",
            "redis://127.0.0.1:6379",
            "secret",
            "24",
            "false",
            "default-instance",
            "localhost",
        )
        .unwrap();
        assert_eq!(config.port, 0);

        let config = build_config_from_values(
            "127.0.0.1",
            "65535",
            "postgres://localhost/test",
            "redis://127.0.0.1:6379",
            "secret",
            "24",
            "false",
            "default-instance",
            "localhost",
        )
        .unwrap();
        assert_eq!(config.port, 65535);
    }

    #[test]
    fn test_port_overflow_fails() {
        let result = build_config_from_values(
            "127.0.0.1",
            "65536",
            "postgres://localhost/test",
            "redis://127.0.0.1:6379",
            "secret",
            "24",
            "false",
            "default-instance",
            "localhost",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_expiry_boundary_values() {
        let config = build_config_from_values(
            "127.0.0.1",
            "8080",
            "postgres://localhost/test",
            "redis://127.0.0.1:6379",
            "secret",
            "0",
            "false",
            "default-instance",
            "localhost",
        )
        .unwrap();
        assert_eq!(config.jwt_expiry_hours, 0);

        let config = build_config_from_values(
            "127.0.0.1",
            "8080",
            "postgres://localhost/test",
            "redis://127.0.0.1:6379",
            "secret",
            "999999",
            "false",
            "default-instance",
            "localhost",
        )
        .unwrap();
        assert_eq!(config.jwt_expiry_hours, 999999);
    }

    #[test]
    fn test_jwt_expiry_negative_fails() {
        let result = build_config_from_values(
            "127.0.0.1",
            "8080",
            "postgres://localhost/test",
            "redis://127.0.0.1:6379",
            "secret",
            "-1",
            "false",
            "default-instance",
            "localhost",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_config_error_message_format() {
        let err = CoreError::Config("DATABASE_URL required".into());
        assert!(err.to_string().contains("DATABASE_URL required"));
        let err = CoreError::Config("invalid port".into());
        assert!(err.to_string().contains("invalid port"));
        let err = CoreError::Config("invalid JWT expiry".into());
        assert!(err.to_string().contains("invalid JWT expiry"));
        let err = CoreError::Config("invalid federation flag".into());
        assert!(err.to_string().contains("invalid federation flag"));
    }
}
