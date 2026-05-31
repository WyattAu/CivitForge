#![forbid(unsafe_code)]

#[cfg(feature = "ssh-server")]
use crate::error::Result;
#[cfg(feature = "ssh-server")]
use crate::git::GitService;
#[cfg(feature = "ssh-server")]
use crate::ssh::auth::SshAuthService;
#[cfg(feature = "ssh-server")]
use crate::ssh::server::SshConfig;
#[cfg(feature = "ssh-server")]
use russh::ChannelId;
#[cfg(feature = "ssh-server")]
use russh::keys::{Algorithm, PrivateKey};
#[cfg(feature = "ssh-server")]
use russh::server::{self, Auth, Handler, Server};
#[cfg(feature = "ssh-server")]
use std::sync::Arc;
#[cfg(feature = "ssh-server")]
use tokio::sync::RwLock;
#[cfg(feature = "ssh-server")]
use tracing::{error, info, warn};

#[cfg(feature = "ssh-server")]
#[derive(Clone)]
pub struct SshDaemon {
    config: SshConfig,
    git_service: Arc<GitService>,
    auth_service: Arc<SshAuthService>,
    pub server_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

#[cfg(feature = "ssh-server")]
impl SshDaemon {
    pub fn new(
        config: SshConfig,
        git_service: Arc<GitService>,
        auth_service: Arc<SshAuthService>,
    ) -> Self {
        Self {
            config,
            git_service,
            auth_service,
            server_handle: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let addr_display = addr.clone();
        let config = Arc::new(self.config.clone());
        let git_service = self.git_service.clone();
        let auth_service = self.auth_service.clone();

        let handle = tokio::spawn(async move {
            let host_key_path = std::path::Path::new(&config.host_keys_path);
            let secret_key = Self::load_or_generate_host_key(host_key_path);

            let mut server = SshDaemonServer {
                config: config.clone(),
                git_service,
                auth_service,
            };

            let addr_parsed: std::net::SocketAddr = addr
                .parse()
                .unwrap_or_else(|_| "0.0.0.0:2222".parse().unwrap());

            let russh_config = Arc::new(russh::server::Config {
                keys: vec![secret_key],
                ..Default::default()
            });

            info!(addr = %addr, "SSH server listening");
            match server.run_on_address(russh_config, addr_parsed).await {
                Ok(()) => info!("SSH server stopped"),
                Err(e) => error!(error = %e, "SSH server error"),
            }
        });

        let mut lock = self.server_handle.write().await;
        *lock = Some(handle);
        info!(addr = %addr_display, "SSH daemon started");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut lock = self.server_handle.write().await;
        if let Some(handle) = lock.take() {
            handle.abort();
            info!("SSH daemon stopped");
        }
        Ok(())
    }

    fn load_or_generate_host_key(path: &std::path::Path) -> PrivateKey {
        if path.exists() {
            match PrivateKey::read_openssh_file(path) {
                Ok(key) => {
                    info!("loaded existing SSH host key");
                    return key;
                }
                Err(e) => {
                    warn!("failed to parse existing host key: {e}, generating new one");
                }
            }
        }

        let mut rng = rand::rng();
        let key = PrivateKey::random(&mut rng, Algorithm::Ed25519).unwrap();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = key.write_openssh_file(path, Default::default());
        info!("generated new SSH host key");
        key
    }
}

#[cfg(feature = "ssh-server")]
struct SshDaemonServer {
    config: Arc<SshConfig>,
    git_service: Arc<GitService>,
    auth_service: Arc<SshAuthService>,
}

#[cfg(feature = "ssh-server")]
#[derive(Debug)]
pub struct DaemonError(String);

impl std::fmt::Display for DaemonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DaemonError {}

impl From<russh::Error> for DaemonError {
    fn from(e: russh::Error) -> Self {
        DaemonError(e.to_string())
    }
}

#[cfg(feature = "ssh-server")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GitCommand {
    UploadPack { repo_path: String },
    ReceivePack { repo_path: String },
}

#[cfg(feature = "ssh-server")]
pub(crate) fn parse_git_command(command: &str) -> std::result::Result<GitCommand, DaemonError> {
    let cmd = command.trim();

    let (service_name, arg) = if let Some(rest) = cmd.strip_prefix("git-upload-pack ") {
        ("upload-pack", rest.trim())
    } else if let Some(rest) = cmd.strip_prefix("git-receive-pack ") {
        ("receive-pack", rest.trim())
    } else {
        return Err(DaemonError(format!("unknown command: {cmd}")));
    };

    let repo_path = arg.strip_prefix('\'').and_then(|s| s.strip_suffix('\''));

    let repo_path = match repo_path {
        Some(p) => p,
        None => {
            return Err(DaemonError(format!("invalid repo path argument: {arg}")));
        }
    };

    let repo_path = repo_path
        .strip_suffix(".git")
        .unwrap_or(repo_path)
        .to_string();

    match service_name {
        "upload-pack" => Ok(GitCommand::UploadPack { repo_path }),
        "receive-pack" => Ok(GitCommand::ReceivePack { repo_path }),
        _ => Err(DaemonError(format!("unknown git service: {service_name}"))),
    }
}

#[cfg(feature = "ssh-server")]
pub(crate) fn parse_repo_path(repo_path: &str) -> std::result::Result<(&str, &str), DaemonError> {
    let mut parts = repo_path.splitn(2, '/');
    let owner = parts
        .next()
        .ok_or_else(|| DaemonError("missing owner".into()))?;
    let name = parts
        .next()
        .ok_or_else(|| DaemonError("missing repo name".into()))?;

    if owner.is_empty() || name.is_empty() {
        return Err(DaemonError("invalid repo path format".into()));
    }

    Ok((owner, name))
}

#[cfg(feature = "ssh-server")]
pub(crate) fn compute_fingerprint(public_key: &russh::keys::PublicKey) -> String {
    public_key
        .fingerprint(russh::keys::HashAlg::Sha256)
        .to_string()
}

#[cfg(feature = "ssh-server")]
impl Server for SshDaemonServer {
    type Handler = Self;

    fn new_client(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> Self::Handler {
        SshDaemonServer {
            config: self.config.clone(),
            git_service: self.git_service.clone(),
            auth_service: self.auth_service.clone(),
        }
    }

    fn handle_session_error(&mut self, _error: <Self::Handler as Handler>::Error) {
        tracing::error!("SSH client session error");
    }
}

#[cfg(feature = "ssh-server")]
impl Handler for SshDaemonServer {
    type Error = DaemonError;

    async fn authentication_banner(&mut self) -> std::result::Result<Option<String>, Self::Error> {
        Ok(Some(self.config.banner.clone()))
    }

    async fn auth_publickey(
        &mut self,
        user: &str,
        public_key: &russh::keys::PublicKey,
    ) -> std::result::Result<Auth, Self::Error> {
        if user == "git" {
            return Ok(Auth::Accept);
        }

        let fingerprint = compute_fingerprint(public_key);
        info!(user = %user, fingerprint = %fingerprint, "SSH pubkey auth attempt");

        match self.auth_service.authenticate(&fingerprint, "ssh") {
            Ok(Some(_record)) => {
                info!(user = %user, fingerprint = %fingerprint, "SSH pubkey auth accepted");
                Ok(Auth::Accept)
            }
            Ok(None) => {
                info!(user = %user, fingerprint = %fingerprint, "SSH pubkey auth rejected: key not found");
                Ok(Auth::Reject {
                    proceed_with_methods: None,
                    partial_success: false,
                })
            }
            Err(e) => {
                warn!(user = %user, error = %e, "SSH pubkey auth error");
                Ok(Auth::Reject {
                    proceed_with_methods: None,
                    partial_success: false,
                })
            }
        }
    }

    async fn channel_open_session(
        &mut self,
        _channel: russh::Channel<server::Msg>,
        _session: &mut russh::server::Session,
    ) -> std::result::Result<bool, Self::Error> {
        Ok(true)
    }

    async fn exec_request(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut russh::server::Session,
    ) -> std::result::Result<(), Self::Error> {
        let command = String::from_utf8_lossy(data).to_string();
        info!(command = %command, "exec request");

        let git_cmd = parse_git_command(&command)?;

        let repo_path_str = match &git_cmd {
            GitCommand::UploadPack { repo_path } => repo_path.as_str(),
            GitCommand::ReceivePack { repo_path } => repo_path.as_str(),
        };

        let (owner, name) = parse_repo_path(repo_path_str)?;

        if !self.git_service.repo_exists(owner, name) {
            let msg = format!("repository not found: {owner}/{name}");
            let _ = session.data(channel, bytes::Bytes::from(format!("ERR: {msg}\n")));
            let _ = session.close(channel);
            return Err(DaemonError(msg));
        }

        let repo_fs_path = self.git_service.repo_path(owner, name);
        let service = match &git_cmd {
            GitCommand::UploadPack { .. } => "upload-pack",
            GitCommand::ReceivePack { .. } => "receive-pack",
        };

        let response = crate::git::http::info_refs(&repo_fs_path, service)
            .map_err(|e| DaemonError(e.to_string()))?;

        session
            .data(channel, bytes::Bytes::from(response))
            .map_err(|e| DaemonError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(not(feature = "ssh-server"))]
pub struct SshDaemon;

#[cfg(not(feature = "ssh-server"))]
impl SshDaemon {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "ssh-server")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_git_upload_pack() {
        let cmd = parse_git_command("git-upload-pack 'owner/repo.git'");
        assert_eq!(
            cmd.unwrap(),
            GitCommand::UploadPack {
                repo_path: "owner/repo".to_string()
            }
        );
    }

    #[test]
    fn test_parse_git_receive_pack() {
        let cmd = parse_git_command("git-receive-pack 'alice/myproject.git'");
        assert_eq!(
            cmd.unwrap(),
            GitCommand::ReceivePack {
                repo_path: "alice/myproject".to_string()
            }
        );
    }

    #[test]
    fn test_parse_git_command_unknown() {
        let cmd = parse_git_command("ls -la");
        assert!(cmd.is_err());
        let err = cmd.unwrap_err().0;
        assert!(err.contains("unknown command"));
    }

    #[test]
    fn test_parse_git_command_without_quotes() {
        let cmd = parse_git_command("git-upload-pack owner/repo.git");
        assert!(cmd.is_err());
        assert!(cmd.unwrap_err().0.contains("invalid repo path argument"));
    }

    #[test]
    fn test_parse_git_command_empty_arg() {
        let cmd = parse_git_command("git-upload-pack ''");
        assert!(cmd.is_err());
    }

    #[test]
    fn test_parse_repo_path_valid() {
        let (owner, name) = parse_repo_path("acme/widgets").unwrap();
        assert_eq!(owner, "acme");
        assert_eq!(name, "widgets");
    }

    #[test]
    fn test_parse_repo_path_missing_name() {
        let result = parse_repo_path("onlyowner");
        assert!(result.is_err());
        assert!(result.unwrap_err().0.contains("missing repo name"));
    }

    #[test]
    fn test_parse_repo_path_missing_owner() {
        let result = parse_repo_path("/repo");
        assert!(result.is_err());
        assert!(result.unwrap_err().0.contains("missing owner"));
    }

    #[test]
    fn test_parse_repo_path_empty() {
        let result = parse_repo_path("");
        assert!(result.is_err());
    }

    #[test]
    fn test_upload_pack_without_git_suffix() {
        let cmd = parse_git_command("git-upload-pack 'owner/repo'");
        assert_eq!(
            cmd.unwrap(),
            GitCommand::UploadPack {
                repo_path: "owner/repo".to_string()
            }
        );
    }

    #[test]
    fn test_git_command_equality() {
        assert_eq!(
            GitCommand::UploadPack {
                repo_path: "a/b".into()
            },
            GitCommand::UploadPack {
                repo_path: "a/b".into()
            }
        );
        assert_ne!(
            GitCommand::UploadPack {
                repo_path: "a/b".into()
            },
            GitCommand::ReceivePack {
                repo_path: "a/b".into()
            }
        );
    }

    #[test]
    fn test_compute_fingerprint_format() {
        let mut rng = rand::rng();
        let key = PrivateKey::random(&mut rng, Algorithm::Ed25519).unwrap();
        let fp = compute_fingerprint(&key.public_key());
        assert!(fp.starts_with("SHA256:"));
        assert!(fp.len() > 20);
    }

    #[test]
    fn test_compute_fingerprint_deterministic() {
        let mut rng1 = rand::rng();
        let mut rng2 = rand::rng();
        let key1 = PrivateKey::random(&mut rng1, Algorithm::Ed25519).unwrap();
        let key2 = PrivateKey::random(&mut rng2, Algorithm::Ed25519).unwrap();
        let fp1 = compute_fingerprint(&key1.public_key());
        let fp2 = compute_fingerprint(&key2.public_key());
        if key1 == key2 {
            assert_eq!(fp1, fp2);
        } else {
            assert_ne!(fp1, fp2);
        }
    }
}
