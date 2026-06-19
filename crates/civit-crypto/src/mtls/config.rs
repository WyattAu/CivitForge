#![deny(unsafe_code)]

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, thiserror::Error)]
pub enum MtlsConfigError {
    #[error("missing required environment variable: {0}")]
    MissingEnv(String),

    #[error("invalid path for {field}: {path}")]
    InvalidPath {
        field: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("certificate file not found: {0}")]
    CertNotFound(PathBuf),

    #[error("key file not found: {0}")]
    KeyNotFound(PathBuf),

    #[error("CA certificate file not found: {0}")]
    CaNotFound(PathBuf),

    #[error("configuration validation failed: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtlsConfig {
    pub ca_cert_path: PathBuf,
    pub ca_key_path: PathBuf,
    pub server_cert_path: PathBuf,
    pub server_key_path: PathBuf,
    pub client_ca_path: Option<PathBuf>,
}

impl MtlsConfig {
    pub fn from_env() -> Result<Self, MtlsConfigError> {
        let ca_cert_path = env_path("MTLS_CA_CERT_PATH")?;
        let ca_key_path = env_path("MTLS_CA_KEY_PATH")?;
        let server_cert_path = env_path("MTLS_SERVER_CERT_PATH")?;
        let server_key_path = env_path("MTLS_SERVER_KEY_PATH")?;
        let client_ca_path = env_opt_path("MTLS_CLIENT_CA_PATH")?;

        let config = Self {
            ca_cert_path,
            ca_key_path,
            server_cert_path,
            server_key_path,
            client_ca_path,
        };

        info!("mTLS configuration loaded from environment");
        Ok(config)
    }

    pub fn validate_paths(&self) -> Result<(), MtlsConfigError> {
        check_file_exists(&self.ca_cert_path, "ca_cert_path")?;
        check_file_exists(&self.ca_key_path, "ca_key_path")?;
        check_file_exists(&self.server_cert_path, "server_cert_path")?;
        check_file_exists(&self.server_key_path, "server_key_path")?;
        if let Some(ref path) = self.client_ca_path {
            check_file_exists(path, "client_ca_path")?;
        }
        Ok(())
    }

    pub fn validate_contents(&self) -> Result<(), MtlsConfigError> {
        validate_pem_file(&self.ca_cert_path, "CA certificate")?;
        validate_pem_file(&self.ca_key_path, "CA key")?;
        validate_pem_file(&self.server_cert_path, "server certificate")?;
        validate_pem_file(&self.server_key_path, "server key")?;
        if let Some(ref path) = self.client_ca_path {
            validate_pem_file(path, "client CA certificate")?;
        }
        Ok(())
    }

    pub fn client_verification_enabled(&self) -> bool {
        self.client_ca_path.is_some()
    }
}

fn env_path(name: &str) -> Result<PathBuf, MtlsConfigError> {
    std::env::var(name)
        .map(PathBuf::from)
        .map_err(|_| MtlsConfigError::MissingEnv(name.to_string()))
}

fn env_opt_path(name: &str) -> Result<Option<PathBuf>, MtlsConfigError> {
    match std::env::var(name) {
        Ok(val) if !val.is_empty() => Ok(Some(PathBuf::from(val))),
        _ => Ok(None),
    }
}

fn check_file_exists(path: &Path, field: &'static str) -> Result<(), MtlsConfigError> {
    if !path.exists() {
        return Err(MtlsConfigError::Validation(format!(
            "{field}: file not found at {}",
            path.display()
        )));
    }
    Ok(())
}

fn validate_pem_file(path: &Path, label: &str) -> Result<(), MtlsConfigError> {
    let content = std::fs::read_to_string(path).map_err(|e| MtlsConfigError::InvalidPath {
        field: "validate_pem",
        path: path.to_path_buf(),
        source: e,
    })?;

    let trimmed = content.trim();
    let has_pem_header = trimmed.contains("-----BEGIN ");
    let has_pem_footer = trimmed.contains("-----END ");

    if !has_pem_header || !has_pem_footer {
        return Err(MtlsConfigError::Validation(format!(
            "{label}: file at {} does not appear to be valid PEM",
            path.display()
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_validation_rejects_invalid_pem() {
        let dir = tempfile::tempdir().unwrap();

        let bad_pem = dir.path().join("bad.pem");
        std::fs::write(&bad_pem, "not a pem file").unwrap();

        let config = MtlsConfig {
            ca_cert_path: bad_pem.clone(),
            ca_key_path: bad_pem.clone(),
            server_cert_path: bad_pem.clone(),
            server_key_path: bad_pem.clone(),
            client_ca_path: None,
        };

        let result = config.validate_contents();
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_accepts_valid_pem() {
        let dir = tempfile::tempdir().unwrap();

        let pem = dir.path().join("cert.pem");
        let mut f = std::fs::File::create(&pem).unwrap();
        writeln!(
            f,
            "-----BEGIN CERTIFICATE-----\ndata\n-----END CERTIFICATE-----"
        )
        .unwrap();

        let config = MtlsConfig {
            ca_cert_path: pem.clone(),
            ca_key_path: pem.clone(),
            server_cert_path: pem.clone(),
            server_key_path: pem.clone(),
            client_ca_path: None,
        };

        assert!(config.validate_contents().is_ok());
        assert!(!config.client_verification_enabled());
    }

    #[test]
    fn test_client_verification_enabled() {
        let config = MtlsConfig {
            ca_cert_path: PathBuf::from("/a"),
            ca_key_path: PathBuf::from("/b"),
            server_cert_path: PathBuf::from("/c"),
            server_key_path: PathBuf::from("/d"),
            client_ca_path: Some(PathBuf::from("/e")),
        };
        assert!(config.client_verification_enabled());

        let config = MtlsConfig {
            ca_cert_path: PathBuf::from("/a"),
            ca_key_path: PathBuf::from("/b"),
            server_cert_path: PathBuf::from("/c"),
            server_key_path: PathBuf::from("/d"),
            client_ca_path: None,
        };
        assert!(!config.client_verification_enabled());
    }

    #[test]
    fn test_validate_paths_missing_file() {
        let config = MtlsConfig {
            ca_cert_path: PathBuf::from("/nonexistent/ca.pem"),
            ca_key_path: PathBuf::from("/nonexistent/ca-key.pem"),
            server_cert_path: PathBuf::from("/nonexistent/server.pem"),
            server_key_path: PathBuf::from("/nonexistent/server-key.pem"),
            client_ca_path: None,
        };
        assert!(config.validate_paths().is_err());
    }

    #[test]
    fn test_validate_paths_all_exist() {
        let dir = tempfile::tempdir().unwrap();

        let paths = ["ca.pem", "ca-key.pem", "server.pem", "server-key.pem"];
        for name in &paths {
            std::fs::write(dir.path().join(name), "x").unwrap();
        }

        let config = MtlsConfig {
            ca_cert_path: dir.path().join("ca.pem"),
            ca_key_path: dir.path().join("ca-key.pem"),
            server_cert_path: dir.path().join("server.pem"),
            server_key_path: dir.path().join("server-key.pem"),
            client_ca_path: None,
        };
        assert!(config.validate_paths().is_ok());
    }

    #[test]
    fn test_validate_paths_missing_optional_client_ca() {
        let dir = tempfile::tempdir().unwrap();

        let paths = ["ca.pem", "ca-key.pem", "server.pem", "server-key.pem"];
        for name in &paths {
            std::fs::write(dir.path().join(name), "x").unwrap();
        }

        let config = MtlsConfig {
            ca_cert_path: dir.path().join("ca.pem"),
            ca_key_path: dir.path().join("ca-key.pem"),
            server_cert_path: dir.path().join("server.pem"),
            server_key_path: dir.path().join("server-key.pem"),
            client_ca_path: Some(dir.path().join("missing-client-ca.pem")),
        };
        assert!(config.validate_paths().is_err());
    }

    #[test]
    fn test_validate_contents_all_valid() {
        let dir = tempfile::tempdir().unwrap();

        let pem_content = "-----BEGIN CERTIFICATE-----\ndata\n-----END CERTIFICATE-----";
        let paths = ["ca.pem", "ca-key.pem", "server.pem", "server-key.pem"];
        for name in &paths {
            std::fs::write(dir.path().join(name), pem_content).unwrap();
        }

        let config = MtlsConfig {
            ca_cert_path: dir.path().join("ca.pem"),
            ca_key_path: dir.path().join("ca-key.pem"),
            server_cert_path: dir.path().join("server.pem"),
            server_key_path: dir.path().join("server-key.pem"),
            client_ca_path: None,
        };
        assert!(config.validate_contents().is_ok());
    }

    #[test]
    fn test_validate_contents_one_bad_pem() {
        let dir = tempfile::tempdir().unwrap();

        let pem_content = "-----BEGIN CERTIFICATE-----\ndata\n-----END CERTIFICATE-----";
        std::fs::write(dir.path().join("ca.pem"), pem_content).unwrap();
        std::fs::write(dir.path().join("ca-key.pem"), "bad content").unwrap();
        std::fs::write(dir.path().join("server.pem"), pem_content).unwrap();
        std::fs::write(dir.path().join("server-key.pem"), pem_content).unwrap();

        let config = MtlsConfig {
            ca_cert_path: dir.path().join("ca.pem"),
            ca_key_path: dir.path().join("ca-key.pem"),
            server_cert_path: dir.path().join("server.pem"),
            server_key_path: dir.path().join("server-key.pem"),
            client_ca_path: None,
        };
        assert!(config.validate_contents().is_err());
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = MtlsConfig {
            ca_cert_path: PathBuf::from("/ca.pem"),
            ca_key_path: PathBuf::from("/ca-key.pem"),
            server_cert_path: PathBuf::from("/server.pem"),
            server_key_path: PathBuf::from("/server-key.pem"),
            client_ca_path: Some(PathBuf::from("/client-ca.pem")),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: MtlsConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.ca_cert_path, deserialized.ca_cert_path);
        assert_eq!(config.ca_key_path, deserialized.ca_key_path);
        assert_eq!(config.server_cert_path, deserialized.server_cert_path);
        assert_eq!(config.server_key_path, deserialized.server_key_path);
        assert_eq!(config.client_ca_path, deserialized.client_ca_path);
    }
}
