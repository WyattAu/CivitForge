#![forbid(unsafe_code)]

use crate::ssh::auth::{RateLimiter, SshKeyStore};
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub host_keys_path: String,
    pub max_connections: u32,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_auth_attempts: u32,
    pub banner: String,
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 2222,
            host_keys_path: "/etc/civit/ssh/host_keys".to_string(),
            max_connections: 100,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300),
            max_auth_attempts: 5,
            banner: "CivitForge SSH Server\r\n".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub id: u64,
    pub remote_addr: String,
    pub username: Option<String>,
    pub authenticated: bool,
    pub connected_at: Instant,
    pub last_activity: Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GitRef {
    pub name: String,
    pub sha: String,
    pub ref_type: GitRefType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GitRefType {
    Branch,
    Tag,
    HEAD,
}

pub trait GitProtocolHandler: Send + Sync {
    fn handle_upload_pack(&self, repo_path: &str, input: &[u8]) -> Result<Vec<u8>, String>;
    fn handle_receive_pack(&self, repo_path: &str, input: &[u8]) -> Result<Vec<u8>, String>;
    fn handle_ls_remote(&self, repo_path: &str) -> Result<Vec<GitRef>, String>;
}

static CONNECTION_COUNTER: AtomicU64 = AtomicU64::new(1);

pub struct SshServer {
    pub config: SshConfig,
    pub auth_service: Arc<dyn SshKeyStore>,
    pub rate_limiter: RateLimiter,
    pub connections: DashMap<u64, ConnectionInfo>,
}

impl SshServer {
    pub fn new(
        config: SshConfig,
        auth_service: Arc<dyn SshKeyStore>,
        rate_limiter: RateLimiter,
    ) -> Self {
        Self {
            config,
            auth_service,
            rate_limiter,
            connections: DashMap::new(),
        }
    }

    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    pub fn disconnect(&self, conn_id: u64) -> bool {
        self.connections.remove(&conn_id).is_some()
    }

    pub fn list_connections(&self) -> Vec<ConnectionInfo> {
        self.connections.iter().map(|r| r.value().clone()).collect()
    }

    pub fn track_connection(
        &self,
        remote_addr: String,
        username: Option<String>,
        authenticated: bool,
    ) -> u64 {
        let id = CONNECTION_COUNTER.fetch_add(1, Ordering::SeqCst);
        let now = Instant::now();
        self.connections.insert(
            id,
            ConnectionInfo {
                id,
                remote_addr,
                username,
                authenticated,
                connected_at: now,
                last_activity: now,
            },
        );
        id
    }

    pub fn is_at_capacity(&self) -> bool {
        self.connections.len() >= self.config.max_connections as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssh::auth::InMemorySshKeyStore;
    use std::thread;

    #[test]
    fn test_ssh_config_defaults() {
        let config = SshConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 2222);
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.connection_timeout, Duration::from_secs(30));
        assert_eq!(config.idle_timeout, Duration::from_secs(300));
        assert_eq!(config.max_auth_attempts, 5);
        assert!(config.banner.contains("CivitForge"));
    }

    #[test]
    fn test_ssh_server_new() {
        let config = SshConfig::default();
        let store: Arc<dyn SshKeyStore> = Arc::new(InMemorySshKeyStore::new());
        let rl = RateLimiter::new(5, Duration::from_secs(60), Duration::from_secs(300));
        let server = SshServer::new(config, store, rl);
        assert_eq!(server.connection_count(), 0);
        assert!(!server.is_at_capacity());
    }

    #[test]
    fn test_track_and_disconnect() {
        let config = SshConfig::default();
        let store: Arc<dyn SshKeyStore> = Arc::new(InMemorySshKeyStore::new());
        let rl = RateLimiter::new(5, Duration::from_secs(60), Duration::from_secs(300));
        let server = SshServer::new(config, store, rl);

        let id = server.track_connection("1.2.3.4:12345".to_string(), None, false);
        assert_eq!(server.connection_count(), 1);

        assert!(server.disconnect(id));
        assert_eq!(server.connection_count(), 0);
        assert!(!server.disconnect(id));
    }

    #[test]
    fn test_track_with_user() {
        let config = SshConfig::default();
        let store: Arc<dyn SshKeyStore> = Arc::new(InMemorySshKeyStore::new());
        let rl = RateLimiter::new(5, Duration::from_secs(60), Duration::from_secs(300));
        let server = SshServer::new(config, store, rl);

        let id = server.track_connection(
            "10.0.0.1:54321".to_string(),
            Some("alice".to_string()),
            true,
        );
        let conns = server.list_connections();
        assert_eq!(conns.len(), 1);
        assert_eq!(conns[0].id, id);
        assert_eq!(conns[0].remote_addr, "10.0.0.1:54321");
        assert_eq!(conns[0].username, Some("alice".to_string()));
        assert!(conns[0].authenticated);
    }

    #[test]
    fn test_capacity_tracking() {
        let config = SshConfig {
            max_connections: 2,
            ..SshConfig::default()
        };
        let store: Arc<dyn SshKeyStore> = Arc::new(InMemorySshKeyStore::new());
        let rl = RateLimiter::new(5, Duration::from_secs(60), Duration::from_secs(300));
        let server = SshServer::new(config, store, rl);

        server.track_connection("1.2.3.4:1".to_string(), None, false);
        assert!(!server.is_at_capacity());

        server.track_connection("5.6.7.8:2".to_string(), None, false);
        assert!(server.is_at_capacity());
    }

    #[test]
    fn test_multiple_connections_unique_ids() {
        let config = SshConfig::default();
        let store: Arc<dyn SshKeyStore> = Arc::new(InMemorySshKeyStore::new());
        let rl = RateLimiter::new(5, Duration::from_secs(60), Duration::from_secs(300));
        let server = SshServer::new(config, store, rl);

        let id1 = server.track_connection("1.2.3.4:1".to_string(), None, false);
        let id2 = server.track_connection("5.6.7.8:2".to_string(), None, false);
        assert_ne!(id1, id2);
        assert_eq!(server.connection_count(), 2);
    }

    #[test]
    fn test_rate_limiter_on_server() {
        let config = SshConfig::default();
        let store: Arc<dyn SshKeyStore> = Arc::new(InMemorySshKeyStore::new());
        let rl = RateLimiter::new(2, Duration::from_secs(60), Duration::from_secs(300));
        let server = SshServer::new(config, store, rl);

        assert!(server.rate_limiter.check("9.8.7.6"));
        server.rate_limiter.record_failure("9.8.7.6");
        assert!(server.rate_limiter.check("9.8.7.6"));
        server.rate_limiter.record_failure("9.8.7.6");
        assert!(!server.rate_limiter.check("9.8.7.6"));
    }

    #[test]
    fn test_git_ref_type_equality() {
        assert_eq!(GitRefType::Branch, GitRefType::Branch);
        assert_ne!(GitRefType::Branch, GitRefType::Tag);
        assert_ne!(GitRefType::Tag, GitRefType::HEAD);
    }

    #[test]
    fn test_git_ref_fields() {
        let git_ref = GitRef {
            name: "refs/heads/main".to_string(),
            sha: "abc123".to_string(),
            ref_type: GitRefType::Branch,
        };
        assert_eq!(git_ref.name, "refs/heads/main");
        assert_eq!(git_ref.sha, "abc123");
        assert_eq!(git_ref.ref_type, GitRefType::Branch);
    }

    struct MockGitHandler;

    impl GitProtocolHandler for MockGitHandler {
        fn handle_upload_pack(&self, _repo_path: &str, _input: &[u8]) -> Result<Vec<u8>, String> {
            Ok(vec![0x00, 0x01, 0x02])
        }

        fn handle_receive_pack(&self, _repo_path: &str, _input: &[u8]) -> Result<Vec<u8>, String> {
            Ok(vec![0x03, 0x04])
        }

        fn handle_ls_remote(&self, _repo_path: &str) -> Result<Vec<GitRef>, String> {
            Ok(vec![
                GitRef {
                    name: "refs/heads/main".to_string(),
                    sha: "deadbeef".to_string(),
                    ref_type: GitRefType::Branch,
                },
                GitRef {
                    name: "HEAD".to_string(),
                    sha: "deadbeef".to_string(),
                    ref_type: GitRefType::HEAD,
                },
            ])
        }
    }

    #[test]
    fn test_mock_git_handler_upload_pack() {
        let handler = MockGitHandler;
        let result = handler.handle_upload_pack("repo", &[]).unwrap();
        assert_eq!(result, vec![0x00, 0x01, 0x02]);
    }

    #[test]
    fn test_mock_git_handler_receive_pack() {
        let handler = MockGitHandler;
        let result = handler.handle_receive_pack("repo", &[]).unwrap();
        assert_eq!(result, vec![0x03, 0x04]);
    }

    #[test]
    fn test_mock_git_handler_ls_remote() {
        let handler = MockGitHandler;
        let refs = handler.handle_ls_remote("repo").unwrap();
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].name, "refs/heads/main");
        assert_eq!(refs[1].ref_type, GitRefType::HEAD);
    }
}
