#![forbid(unsafe_code)]

#[cfg(feature = "ssh-server")]
use crate::error::{CoreError, Result};
#[cfg(feature = "ssh-server")]
use crate::git::GitService;
#[cfg(feature = "ssh-server")]
use crate::ssh::server::SshConfig;
#[cfg(feature = "ssh-server")]
use russh::ChannelId;
#[cfg(feature = "ssh-server")]
use russh::keys::{self, Algorithm, PrivateKey};
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
    pub server_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

#[cfg(feature = "ssh-server")]
impl SshDaemon {
    pub fn new(config: SshConfig, git_service: Arc<GitService>) -> Self {
        Self {
            config,
            git_service,
            server_handle: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let config = Arc::new(self.config.clone());
        let git_service = self.git_service.clone();

        let handle = tokio::spawn(async move {
            let host_key_path = std::path::Path::new(&config.host_keys_path);
            let secret_key = Self::load_or_generate_host_key(host_key_path);

            let mut server = SshDaemonServer {
                config: config.clone(),
                git_service,
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
        info!(addr = %addr, "SSH daemon started");
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
impl Server for SshDaemonServer {
    type Handler = Self;

    fn new_client(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> Self::Handler {
        SshDaemonServer {
            config: self.config.clone(),
            git_service: self.git_service.clone(),
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
        _user: &str,
        _public_key: &russh::keys::PublicKey,
    ) -> std::result::Result<Auth, Self::Error> {
        Ok(Auth::Reject {
            proceed_with_methods: None,
        })
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
        let _ = (channel, session);
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
