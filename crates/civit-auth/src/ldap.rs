use crate::error::{AuthError, Result};
use ldap3::{LdapConn, LdapConnSettings, Scope, SearchEntry};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapUserInfo {
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub groups: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LdapConfig {
    pub enabled: bool,
    pub url: String,
    pub bind_dn: String,
    pub bind_password: String,
    pub user_search_base: String,
    pub user_filter: String,
    pub group_search_base: String,
    pub group_search_filter: String,
    pub max_connections: usize,
    pub tls_ca_path: Option<String>,
    pub connection_timeout_secs: u64,
    pub idle_timeout_secs: u64,
}

impl Default for LdapConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: String::new(),
            bind_dn: String::new(),
            bind_password: String::new(),
            user_search_base: String::new(),
            user_filter: String::new(),
            group_search_base: String::new(),
            group_search_filter: String::new(),
            max_connections: 10,
            tls_ca_path: None,
            connection_timeout_secs: 10,
            idle_timeout_secs: 300,
        }
    }
}

#[derive(Debug, Default)]
pub struct LdapPoolMetrics {
    pub connections_created: AtomicU64,
    pub connections_reused: AtomicU64,
    pub connections_failed: AtomicU64,
}

impl LdapPoolMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> (u64, u64, u64) {
        (
            self.connections_created.load(Ordering::Relaxed),
            self.connections_reused.load(Ordering::Relaxed),
            self.connections_failed.load(Ordering::Relaxed),
        )
    }
}

struct PooledConnection {
    conn: LdapConn,
    last_used: Instant,
}

pub struct LdapPool {
    connections: Mutex<Vec<PooledConnection>>,
    max_size: usize,
    url: String,
    bind_dn: String,
    bind_password: String,
    tls_ca_path: Option<String>,
    connection_timeout_secs: u64,
    idle_timeout_secs: u64,
    metrics: Arc<LdapPoolMetrics>,
}

impl LdapPool {
    pub fn new(config: &LdapConfig) -> Self {
        Self {
            connections: Mutex::new(Vec::new()),
            max_size: config.max_connections,
            url: config.url.clone(),
            bind_dn: config.bind_dn.clone(),
            bind_password: config.bind_password.clone(),
            tls_ca_path: config.tls_ca_path.clone(),
            connection_timeout_secs: config.connection_timeout_secs,
            idle_timeout_secs: config.idle_timeout_secs,
            metrics: Arc::new(LdapPoolMetrics::new()),
        }
    }

    pub fn metrics(&self) -> &Arc<LdapPoolMetrics> {
        &self.metrics
    }

    pub fn get_connection(&self) -> Result<LdapConn> {
        {
            let mut pool = self.connections.lock().map_err(|e| {
                AuthError::Internal(format!("LDAP pool lock poisoned: {e}"))
            })?;
            let idle_timeout = Duration::from_secs(self.idle_timeout_secs);
            let now = Instant::now();
            pool.retain(|pc| now.duration_since(pc.last_used) < idle_timeout);
            while let Some(pc) = pool.pop() {
                if now.duration_since(pc.last_used) < idle_timeout {
                    return Ok(pc.conn);
                }
            }
        }

        self.create_connection()
    }

    pub fn return_connection(&self, conn: LdapConn) {
        let mut pool = match self.connections.lock() {
            Ok(p) => p,
            Err(_) => return,
        };
        if pool.len() < self.max_size {
            pool.push(PooledConnection {
                conn,
                last_used: Instant::now(),
            });
        }
    }

    pub(crate) fn create_connection(&self) -> Result<LdapConn> {
        let mut settings = LdapConnSettings::new()
            .set_conn_timeout(Duration::from_secs(self.connection_timeout_secs));

        if self.tls_ca_path.is_some() {
            settings = settings.set_no_tls_verify(false);
        } else {
            settings = settings.set_no_tls_verify(true);
        }

        let mut conn = LdapConn::with_settings(settings, &self.url).map_err(|e| {
            self.metrics.connections_failed.fetch_add(1, Ordering::Relaxed);
            AuthError::Internal(format!("LDAP connection failed: {e}"))
        })?;

        match conn.simple_bind(&self.bind_dn, &self.bind_password) {
            Ok(result) => {
                if let Err(e) = result.success() {
                    self.metrics.connections_failed.fetch_add(1, Ordering::Relaxed);
                    return Err(AuthError::Auth(format!("LDAP bind rejected: {e}")));
                }
            }
            Err(e) => {
                self.metrics.connections_failed.fetch_add(1, Ordering::Relaxed);
                return Err(AuthError::Auth(format!("LDAP bind failed: {e}")));
            }
        }

        self.metrics.connections_created.fetch_add(1, Ordering::Relaxed);
        Ok(conn)
    }
}

pub struct LdapAuth;

impl LdapAuth {
    pub async fn authenticate(
        config: &LdapConfig,
        username: &str,
        password: &str,
    ) -> Result<LdapUserInfo> {
        if !config.enabled {
            return Err(AuthError::Auth(
                "LDAP authentication is not enabled".into(),
            ));
        }

        let pool = LdapPool::new(config);
        let mut conn = pool.get_connection()?;

        let filter = config.user_filter.replace("{}", username);
        let search_result = conn
            .search(
                &config.user_search_base,
                Scope::Subtree,
                &filter,
                vec![
                    "uid",
                    "cn",
                    "mail",
                    "displayName",
                    "memberOf",
                    "dn",
                ],
            )
            .map_err(|e| AuthError::Internal(format!("LDAP search failed: {e}")))?;

        let entries: Vec<SearchEntry> = search_result
            .0
            .into_iter()
            .map(SearchEntry::construct)
            .collect();

        if entries.is_empty() {
            return Err(AuthError::Auth("User not found in LDAP".into()));
        }

        let entry = &entries[0];
        let user_dn = entry.dn.clone();

        let mut user_conn = pool.create_connection()?;

        user_conn
            .simple_bind(&user_dn, password)
            .map_err(|e| AuthError::Auth(format!("Invalid credentials: {e}")))?
            .success()
            .map_err(|e| AuthError::Auth(format!("Authentication failed: {e}")))?;

        let uid = entry
            .attrs
            .get("uid")
            .and_then(|v| v.first())
            .cloned()
            .unwrap_or_else(|| username.to_string());

        let cn = entry
            .attrs
            .get("cn")
            .and_then(|v| v.first())
            .cloned()
            .unwrap_or_else(|| username.to_string());

        let email = entry
            .attrs
            .get("mail")
            .and_then(|v| v.first())
            .cloned()
            .unwrap_or_else(|| format!("{username}@ldap.local"));

        let display_name = entry
            .attrs
            .get("displayName")
            .and_then(|v| v.first())
            .cloned()
            .unwrap_or_else(|| cn.clone());

        let groups = Self::extract_groups_from_entry(entry);

        pool.return_connection(conn);

        Ok(LdapUserInfo {
            username: uid,
            email,
            display_name,
            groups,
        })
    }

    pub async fn sync_groups(
        config: &LdapConfig,
        username: &str,
    ) -> Result<Vec<String>> {
        if !config.enabled {
            return Err(AuthError::Auth(
                "LDAP authentication is not enabled".into(),
            ));
        }

        let pool = LdapPool::new(config);
        let mut conn = pool.get_connection()?;

        let filter = config.group_search_filter.replace("{}", username);
        let search_result = conn
            .search(
                &config.group_search_base,
                Scope::Subtree,
                &filter,
                vec!["cn", "dn"],
            )
            .map_err(|e| AuthError::Internal(format!("LDAP group search failed: {e}")))?;

        let entries: Vec<SearchEntry> = search_result
            .0
            .into_iter()
            .map(SearchEntry::construct)
            .collect();

        let groups: Vec<String> = entries
            .iter()
            .filter_map(|e| {
                e.attrs
                    .get("cn")
                    .and_then(|v| v.first())
                    .cloned()
            })
            .collect();

        pool.return_connection(conn);

        Ok(groups)
    }

    fn extract_groups_from_entry(entry: &SearchEntry) -> Vec<String> {
        entry
            .attrs
            .get("memberOf")
            .map(|dns| {
                dns.iter()
                    .filter_map(|dn| {
                        dn.split(',')
                            .find(|part| part.trim().to_lowercase().starts_with("cn="))
                            .map(|part| part.trim()[3..].to_string())
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_groups_from_dn() {
        let mut attrs = std::collections::HashMap::new();
        attrs.insert(
            "memberOf".to_string(),
            vec![
                "cn=admins,ou=groups,dc=example,dc=com".to_string(),
                "cn=devs,ou=groups,dc=example,dc=com".to_string(),
            ],
        );
        let entry = SearchEntry {
            dn: "cn=test".to_string(),
            attrs,
            bin_attrs: std::collections::HashMap::new(),
        };
        let groups = LdapAuth::extract_groups_from_entry(&entry);
        assert_eq!(groups, vec!["admins", "devs"]);
    }

    #[test]
    fn test_extract_groups_empty() {
        let entry = SearchEntry {
            dn: "cn=test".to_string(),
            attrs: std::collections::HashMap::new(),
            bin_attrs: std::collections::HashMap::new(),
        };
        let groups = LdapAuth::extract_groups_from_entry(&entry);
        assert!(groups.is_empty());
    }

    #[test]
    fn test_ldap_pool_new() {
        let config = LdapConfig::default();
        let pool = LdapPool::new(&config);
        assert_eq!(pool.max_size, 10);
        assert!(pool.tls_ca_path.is_none());
        assert_eq!(pool.connection_timeout_secs, 10);
        assert_eq!(pool.idle_timeout_secs, 300);
    }

    #[test]
    fn test_ldap_pool_custom_config() {
        let config = LdapConfig {
            max_connections: 5,
            tls_ca_path: Some("/etc/ldap/ca.pem".to_string()),
            connection_timeout_secs: 30,
            idle_timeout_secs: 600,
            ..LdapConfig::default()
        };
        let pool = LdapPool::new(&config);
        assert_eq!(pool.max_size, 5);
        assert_eq!(pool.tls_ca_path, Some("/etc/ldap/ca.pem".to_string()));
        assert_eq!(pool.connection_timeout_secs, 30);
        assert_eq!(pool.idle_timeout_secs, 600);
    }

    #[test]
    fn test_ldap_pool_empty_on_creation() {
        let config = LdapConfig::default();
        let pool = LdapPool::new(&config);
        assert!(pool.connections.lock().unwrap().is_empty());
    }

    #[test]
    fn test_ldap_config_default() {
        let config = LdapConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.connection_timeout_secs, 10);
        assert_eq!(config.idle_timeout_secs, 300);
    }

    #[test]
    fn test_ldap_pool_metrics_default() {
        let metrics = LdapPoolMetrics::new();
        let (created, reused, failed) = metrics.snapshot();
        assert_eq!(created, 0);
        assert_eq!(reused, 0);
        assert_eq!(failed, 0);
    }

    #[test]
    fn test_ldap_pool_metrics_incr() {
        let metrics = LdapPoolMetrics::new();
        metrics.connections_created.fetch_add(2, Ordering::Relaxed);
        metrics.connections_reused.fetch_add(5, Ordering::Relaxed);
        metrics.connections_failed.fetch_add(1, Ordering::Relaxed);
        let (created, reused, failed) = metrics.snapshot();
        assert_eq!(created, 2);
        assert_eq!(reused, 5);
        assert_eq!(failed, 1);
    }
}
