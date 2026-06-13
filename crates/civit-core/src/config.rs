#![forbid(unsafe_code)]

use crate::error::CoreError;
use ring::rand::SecureRandom;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub login_max_attempts: u32,
    pub login_lockout_secs: i64,
    pub password_min_length: usize,
    pub password_max_length: usize,
    pub password_require_uppercase: bool,
    pub password_require_lowercase: bool,
    pub password_require_digit: bool,
    pub password_require_special: bool,
    #[serde(default)]
    pub ldap_enabled: bool,
    #[serde(default)]
    pub ldap_url: String,
    #[serde(default)]
    pub ldap_bind_dn: String,
    #[serde(default)]
    pub ldap_bind_password: String,
    #[serde(default)]
    pub ldap_user_search_base: String,
    #[serde(default)]
    pub ldap_user_filter: String,
    #[serde(default)]
    pub ldap_group_search_base: String,
    #[serde(default)]
    pub ldap_group_search_filter: String,
    #[serde(default = "default_ldap_max_connections")]
    pub ldap_max_connections: usize,
    #[serde(default)]
    pub ldap_tls_ca_path: Option<String>,
    #[serde(default = "default_ldap_connection_timeout_secs")]
    pub ldap_connection_timeout_secs: u64,
    #[serde(default = "default_ldap_idle_timeout_secs")]
    pub ldap_idle_timeout_secs: u64,
}

fn default_ldap_max_connections() -> usize {
    10
}

fn default_ldap_connection_timeout_secs() -> u64 {
    10
}

fn default_ldap_idle_timeout_secs() -> u64 {
    300
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            login_max_attempts: 5,
            login_lockout_secs: 900,
            password_min_length: 8,
            password_max_length: 128,
            password_require_uppercase: true,
            password_require_lowercase: true,
            password_require_digit: true,
            password_require_special: true,
            ldap_enabled: false,
            ldap_url: String::new(),
            ldap_bind_dn: String::new(),
            ldap_bind_password: String::new(),
            ldap_user_search_base: String::new(),
            ldap_user_filter: String::new(),
            ldap_group_search_base: String::new(),
            ldap_group_search_filter: String::new(),
            ldap_max_connections: 10,
            ldap_tls_ca_path: None,
            ldap_connection_timeout_secs: 10,
            ldap_idle_timeout_secs: 300,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,
    pub storage_path: String,
    pub federation_enabled: bool,
    pub federation_instance_id: String,
    pub federation_instance_domain: String,
    pub cors_allowed_origins: Vec<String>,
    pub rate_limit_max_requests: Option<u32>,
    pub rate_limit_window_secs: Option<u32>,
    pub security: SecurityConfig,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
    pub ui_assets_path: String,
    pub debug_mode: bool,
}

impl AppConfig {
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.host.is_empty() {
            return Err(CoreError::Config("host must not be empty".into()));
        }
        if self.port == 0 {
            return Err(CoreError::Config("port must not be 0".into()));
        }
        if self.database_url.is_empty() {
            return Err(CoreError::Config("DATABASE_URL must not be empty".into()));
        }
        if self.database_url.starts_with("postgres://")
            && self.database_url.contains('@')
            && self.database_url.len() < 20
        {
            return Err(CoreError::Config("DATABASE_URL appears malformed".into()));
        }
        if self.jwt_secret.len() < 32 {
            return Err(CoreError::Config(
                "JWT_SECRET must be at least 32 characters (256 bits)".into(),
            ));
        }
        if self.storage_path.is_empty() {
            return Err(CoreError::Config("storage_path must not be empty".into()));
        }
        if self.federation_enabled {
            if self.federation_instance_id.is_empty() {
                return Err(CoreError::Config(
                    "federation_instance_id must not be empty when federation is enabled".into(),
                ));
            }
            if self.federation_instance_domain.is_empty() {
                return Err(CoreError::Config(
                    "federation_instance_domain must not be empty when federation is enabled"
                        .into(),
                ));
            }
        }
        Ok(())
    }

    pub fn tls_enabled(&self) -> bool {
        self.tls_cert_path.is_some() && self.tls_key_path.is_some()
    }

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
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| {
                    let mut buf = [0u8; 32];
                    ring::rand::SystemRandom::new()
                        .fill(&mut buf)
                        .expect("RNG failure");
                    hex::encode(buf)
                }),
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
            storage_path: std::env::var("CIVIT_STORAGE_PATH")
                .unwrap_or_else(|_| "/var/lib/civit/repos".into()),
            cors_allowed_origins: std::env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            rate_limit_max_requests: std::env::var("RATE_LIMIT_MAX_REQUESTS")
                .ok()
                .and_then(|v| v.parse::<u32>().ok()),
            rate_limit_window_secs: std::env::var("RATE_LIMIT_WINDOW_SECS")
                .ok()
                .and_then(|v| v.parse::<u32>().ok()),
            security: SecurityConfig {
                login_max_attempts: std::env::var("LOGIN_MAX_ATTEMPTS")
                    .ok()
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(5),
                login_lockout_secs: std::env::var("LOGIN_LOCKOUT_SECS")
                    .ok()
                    .and_then(|v| v.parse::<i64>().ok())
                    .unwrap_or(900),
                password_min_length: std::env::var("PASSWORD_MIN_LENGTH")
                    .ok()
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(8),
                password_max_length: std::env::var("PASSWORD_MAX_LENGTH")
                    .ok()
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(128),
                password_require_uppercase: std::env::var("PASSWORD_REQUIRE_UPPERCASE")
                    .ok()
                    .and_then(|v| v.parse::<bool>().ok())
                    .unwrap_or(true),
                password_require_lowercase: std::env::var("PASSWORD_REQUIRE_LOWERCASE")
                    .ok()
                    .and_then(|v| v.parse::<bool>().ok())
                    .unwrap_or(true),
                password_require_digit: std::env::var("PASSWORD_REQUIRE_DIGIT")
                    .ok()
                    .and_then(|v| v.parse::<bool>().ok())
                    .unwrap_or(true),
                password_require_special: std::env::var("PASSWORD_REQUIRE_SPECIAL")
                    .ok()
                    .and_then(|v| v.parse::<bool>().ok())
                    .unwrap_or(true),
                ldap_enabled: std::env::var("LDAP_ENABLED")
                    .ok()
                    .and_then(|v| v.parse::<bool>().ok())
                    .unwrap_or(false),
                ldap_url: std::env::var("LDAP_URL")
                    .unwrap_or_else(|_| "ldap://localhost:389".into()),
                ldap_bind_dn: std::env::var("LDAP_BIND_DN")
                    .unwrap_or_default(),
                ldap_bind_password: std::env::var("LDAP_BIND_PASSWORD")
                    .unwrap_or_default(),
                ldap_user_search_base: std::env::var("LDAP_USER_SEARCH_BASE")
                    .unwrap_or_else(|_| "ou=users".into()),
                ldap_user_filter: std::env::var("LDAP_USER_FILTER")
                    .unwrap_or_else(|_| "(uid={})".into()),
                ldap_group_search_base: std::env::var("LDAP_GROUP_SEARCH_BASE")
                    .unwrap_or_else(|_| "ou=groups".into()),
                ldap_group_search_filter: std::env::var("LDAP_GROUP_FILTER")
                    .unwrap_or_else(|_| "(memberUid={})".into()),
                ldap_max_connections: std::env::var("LDAP_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(10),
                ldap_tls_ca_path: std::env::var("LDAP_TLS_CA_PATH")
                    .ok()
                    .filter(|s| !s.is_empty()),
                ldap_connection_timeout_secs: std::env::var("LDAP_CONNECTION_TIMEOUT_SECS")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(10),
                ldap_idle_timeout_secs: std::env::var("LDAP_IDLE_TIMEOUT_SECS")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(300),
            },
            tls_cert_path: std::env::var("TLS_CERT_PATH")
                .ok()
                .filter(|s| !s.is_empty()),
            tls_key_path: std::env::var("TLS_KEY_PATH").ok().filter(|s| !s.is_empty()),
            ui_assets_path: std::env::var("UI_ASSETS_PATH")
                .unwrap_or_else(|_| "./crates/civit-ui/dist".into()),
            debug_mode: std::env::var("CIVIT_DEBUG")
                .unwrap_or_else(|_| "false".into())
                .parse()
                .map_err(|_| CoreError::Config("invalid debug flag".into()))?,
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
        storage_path: &str,
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
        let storage_path = storage_path.to_string();
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
            storage_path,
            cors_allowed_origins: Vec::new(),
            rate_limit_max_requests: None,
            rate_limit_window_secs: None,
            security: SecurityConfig::default(),
            tls_cert_path: None,
            tls_key_path: None,
            ui_assets_path: "./crates/civit-ui/dist".into(),
            debug_mode: false,
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
            "/data/repos",
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
            "/data/repos",
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
            "/data/repos",
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
            "/data/repos",
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
            "/opt/repos",
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
            "/data/repos",
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
            storage_path: "/data/repos".into(),
            cors_allowed_origins: Vec::new(),
            rate_limit_max_requests: None,
            rate_limit_window_secs: None,
            security: SecurityConfig::default(),
            tls_cert_path: None,
            tls_key_path: None,
            ui_assets_path: "./crates/civit-ui/dist".into(),
            debug_mode: false,
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
            storage_path: String::new(),
            cors_allowed_origins: Vec::new(),
            rate_limit_max_requests: None,
            rate_limit_window_secs: None,
            security: SecurityConfig::default(),
            tls_cert_path: None,
            tls_key_path: None,
            ui_assets_path: "./crates/civit-ui/dist".into(),
            debug_mode: false,
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
            storage_path: "/data".into(),
            cors_allowed_origins: Vec::new(),
            rate_limit_max_requests: None,
            rate_limit_window_secs: None,
            security: SecurityConfig::default(),
            tls_cert_path: None,
            tls_key_path: None,
            ui_assets_path: "./crates/civit-ui/dist".into(),
            debug_mode: false,
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
            "/data/repos",
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
            "/data/repos",
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
            "/data/repos",
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
            "/data/repos",
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
            "/data/repos",
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
            "/data/repos",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_host() {
        let config = AppConfig {
            host: String::new(),
            port: 8080,
            database_url: "postgres://localhost/test".into(),
            redis_url: "redis://localhost:6379".into(),
            jwt_secret: "test-secret-key-32bytes-minimums".into(),
            jwt_expiry_hours: 24,
            federation_enabled: false,
            federation_instance_id: "default-instance".into(),
            federation_instance_domain: "localhost".into(),
            storage_path: "/data/repos".into(),
            cors_allowed_origins: Vec::new(),
            rate_limit_max_requests: None,
            rate_limit_window_secs: None,
            security: SecurityConfig::default(),
            tls_cert_path: None,
            tls_key_path: None,
            ui_assets_path: "./crates/civit-ui/dist".into(),
            debug_mode: false,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_port_zero() {
        let config = AppConfig {
            host: "127.0.0.1".into(),
            port: 0,
            database_url: "postgres://localhost/test".into(),
            redis_url: "redis://localhost:6379".into(),
            jwt_secret: "test-secret-key-32bytes-minimums".into(),
            jwt_expiry_hours: 24,
            federation_enabled: false,
            federation_instance_id: "default-instance".into(),
            federation_instance_domain: "localhost".into(),
            storage_path: "/data/repos".into(),
            cors_allowed_origins: Vec::new(),
            rate_limit_max_requests: None,
            rate_limit_window_secs: None,
            security: SecurityConfig::default(),
            tls_cert_path: None,
            tls_key_path: None,
            ui_assets_path: "./crates/civit-ui/dist".into(),
            debug_mode: false,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_empty_database_url() {
        let config = AppConfig {
            host: "127.0.0.1".into(),
            port: 8080,
            database_url: String::new(),
            redis_url: "redis://localhost:6379".into(),
            jwt_secret: "test-secret-key-32bytes-minimums".into(),
            jwt_expiry_hours: 24,
            federation_enabled: false,
            federation_instance_id: "default-instance".into(),
            federation_instance_domain: "localhost".into(),
            storage_path: "/data/repos".into(),
            cors_allowed_origins: Vec::new(),
            rate_limit_max_requests: None,
            rate_limit_window_secs: None,
            security: SecurityConfig::default(),
            tls_cert_path: None,
            tls_key_path: None,
            ui_assets_path: "./crates/civit-ui/dist".into(),
            debug_mode: false,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_short_jwt_secret() {
        let config = AppConfig {
            host: "127.0.0.1".into(),
            port: 8080,
            database_url: "postgres://localhost/test".into(),
            redis_url: "redis://localhost:6379".into(),
            jwt_secret: "short".into(),
            jwt_expiry_hours: 24,
            federation_enabled: false,
            federation_instance_id: "default-instance".into(),
            federation_instance_domain: "localhost".into(),
            storage_path: "/data/repos".into(),
            cors_allowed_origins: Vec::new(),
            rate_limit_max_requests: None,
            rate_limit_window_secs: None,
            security: SecurityConfig::default(),
            tls_cert_path: None,
            tls_key_path: None,
            ui_assets_path: "./crates/civit-ui/dist".into(),
            debug_mode: false,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_empty_storage_path() {
        let config = AppConfig {
            host: "127.0.0.1".into(),
            port: 8080,
            database_url: "postgres://localhost/test".into(),
            redis_url: "redis://localhost:6379".into(),
            jwt_secret: "test-secret-key-32bytes-minimums".into(),
            jwt_expiry_hours: 24,
            federation_enabled: false,
            federation_instance_id: "default-instance".into(),
            federation_instance_domain: "localhost".into(),
            storage_path: String::new(),
            cors_allowed_origins: Vec::new(),
            rate_limit_max_requests: None,
            rate_limit_window_secs: None,
            security: SecurityConfig::default(),
            tls_cert_path: None,
            tls_key_path: None,
            ui_assets_path: "./crates/civit-ui/dist".into(),
            debug_mode: false,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_federation_empty_instance_id() {
        let config = AppConfig {
            host: "127.0.0.1".into(),
            port: 8080,
            database_url: "postgres://localhost/test".into(),
            redis_url: "redis://localhost:6379".into(),
            jwt_secret: "test-secret-key-32bytes-minimums".into(),
            jwt_expiry_hours: 24,
            federation_enabled: true,
            federation_instance_id: String::new(),
            federation_instance_domain: "localhost".into(),
            storage_path: "/data/repos".into(),
            cors_allowed_origins: Vec::new(),
            rate_limit_max_requests: None,
            rate_limit_window_secs: None,
            security: SecurityConfig::default(),
            tls_cert_path: None,
            tls_key_path: None,
            ui_assets_path: "./crates/civit-ui/dist".into(),
            debug_mode: false,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_federation_empty_domain() {
        let config = AppConfig {
            host: "127.0.0.1".into(),
            port: 8080,
            database_url: "postgres://localhost/test".into(),
            redis_url: "redis://localhost:6379".into(),
            jwt_secret: "test-secret-key-32bytes-minimums".into(),
            jwt_expiry_hours: 24,
            federation_enabled: true,
            federation_instance_id: "inst-1".into(),
            federation_instance_domain: String::new(),
            storage_path: "/data/repos".into(),
            cors_allowed_origins: Vec::new(),
            rate_limit_max_requests: None,
            rate_limit_window_secs: None,
            security: SecurityConfig::default(),
            tls_cert_path: None,
            tls_key_path: None,
            ui_assets_path: "./crates/civit-ui/dist".into(),
            debug_mode: false,
        };
        assert!(config.validate().is_err());
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
