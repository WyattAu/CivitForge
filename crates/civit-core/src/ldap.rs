#![forbid(unsafe_code)]

use crate::config::SecurityConfig;
use crate::error::{CoreError, Result};
use ldap3::{LdapConn, LdapConnSettings, Scope, SearchEntry};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapUserInfo {
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub groups: Vec<String>,
}

/// Connection pool for LDAP connections (ldap3 sync connections are not Send).
pub struct LdapPool {
    connections: Mutex<Vec<LdapConn>>,
    max_size: usize,
    url: String,
    bind_dn: String,
    bind_password: String,
    tls_ca_path: Option<String>,
    connection_timeout_secs: u64,
}

impl LdapPool {
    /// Create a new LDAP connection pool.
    pub fn new(config: &SecurityConfig) -> Self {
        Self {
            connections: Mutex::new(Vec::new()),
            max_size: config.ldap_max_connections,
            url: config.ldap_url.clone(),
            bind_dn: config.ldap_bind_dn.clone(),
            bind_password: config.ldap_bind_password.clone(),
            tls_ca_path: config.ldap_tls_ca_path.clone(),
            connection_timeout_secs: config.ldap_connection_timeout_secs,
        }
    }

    /// Get a connection from the pool, or create a new one if the pool is empty.
    pub fn get_connection(&self) -> Result<LdapConn> {
        // Try to reuse an existing connection from the pool
        {
            let mut pool = self.connections.lock().map_err(|e| {
                CoreError::Internal(format!("LDAP pool lock poisoned: {e}"))
            })?;
            while let Some(mut conn) = pool.pop() {
                if self.is_alive(&mut conn) {
                    return Ok(conn);
                }
            }
        }

        // Pool is empty or all connections were dead; create a new one
        self.create_connection()
    }

    /// Return a connection to the pool for reuse.
    pub fn return_connection(&self, conn: LdapConn) {
        let mut pool = match self.connections.lock() {
            Ok(p) => p,
            Err(_) => return,
        };
        if pool.len() < self.max_size {
            pool.push(conn);
        }
    }

    /// Check if a connection is still alive by attempting a no-op search.
    fn is_alive(&self, conn: &mut LdapConn) -> bool {
        conn.search("", Scope::Base, "(objectClass=*)", vec!["dn"])
            .is_ok()
    }

    /// Create a new LDAP connection with configured settings.
    pub(crate) fn create_connection(&self) -> Result<LdapConn> {
        let mut settings = LdapConnSettings::new()
            .set_conn_timeout(Duration::from_secs(self.connection_timeout_secs));

        if self.tls_ca_path.is_some() {
            settings = settings.set_no_tls_verify(false);
        } else {
            settings = settings.set_no_tls_verify(true);
        }

        let mut conn = LdapConn::with_settings(settings, &self.url)
            .map_err(|e| CoreError::Internal(format!("LDAP connection failed: {e}")))?;

        // Bind with service account
        conn.simple_bind(&self.bind_dn, &self.bind_password)
            .map_err(|e| CoreError::Auth(format!("LDAP bind failed: {e}")))?
            .success()
            .map_err(|e| CoreError::Auth(format!("LDAP bind rejected: {e}")))?;

        Ok(conn)
    }
}

pub struct LdapAuth;

impl LdapAuth {
    pub async fn authenticate(
        config: &SecurityConfig,
        username: &str,
        password: &str,
    ) -> Result<LdapUserInfo> {
        if !config.ldap_enabled {
            return Err(CoreError::Auth(
                "LDAP authentication is not enabled".into(),
            ));
        }

        let pool = LdapPool::new(config);
        let mut conn = pool.get_connection()?;

        // Search for user
        let filter = config.ldap_user_filter.replace("{}", username);
        let search_result = conn
            .search(
                &config.ldap_user_search_base,
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
            .map_err(|e| CoreError::Internal(format!("LDAP search failed: {e}")))?;

        let entries: Vec<SearchEntry> = search_result
            .0
            .into_iter()
            .map(SearchEntry::construct)
            .collect();

        if entries.is_empty() {
            return Err(CoreError::Auth("User not found in LDAP".into()));
        }

        let entry = &entries[0];
        let user_dn = entry.dn.clone();

        // Attempt to bind as the found user to verify credentials
        let mut user_conn = pool.create_connection()?;

        user_conn
            .simple_bind(&user_dn, password)
            .map_err(|e| CoreError::Auth(format!("Invalid credentials: {e}")))?
            .success()
            .map_err(|e| CoreError::Auth(format!("Authentication failed: {e}")))?;

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
        config: &SecurityConfig,
        username: &str,
    ) -> Result<Vec<String>> {
        if !config.ldap_enabled {
            return Err(CoreError::Auth(
                "LDAP authentication is not enabled".into(),
            ));
        }

        let pool = LdapPool::new(config);
        let mut conn = pool.get_connection()?;

        let filter = config.ldap_group_search_filter.replace("{}", username);
        let search_result = conn
            .search(
                &config.ldap_group_search_base,
                Scope::Subtree,
                &filter,
                vec!["cn", "dn"],
            )
            .map_err(|e| CoreError::Internal(format!("LDAP group search failed: {e}")))?;

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
        let config = SecurityConfig::default();
        let pool = LdapPool::new(&config);
        assert_eq!(pool.max_size, 10);
        assert!(pool.tls_ca_path.is_none());
        assert_eq!(pool.connection_timeout_secs, 10);
    }

    #[test]
    fn test_ldap_pool_custom_config() {
        let config = SecurityConfig {
            ldap_max_connections: 5,
            ldap_tls_ca_path: Some("/etc/ldap/ca.pem".to_string()),
            ldap_connection_timeout_secs: 30,
            ..SecurityConfig::default()
        };
        let pool = LdapPool::new(&config);
        assert_eq!(pool.max_size, 5);
        assert_eq!(pool.tls_ca_path, Some("/etc/ldap/ca.pem".to_string()));
        assert_eq!(pool.connection_timeout_secs, 30);
    }

    #[test]
    fn test_ldap_pool_return_connection() {
        let config = SecurityConfig::default();
        let pool = LdapPool::new(&config);
        let before = pool.connections.lock().unwrap().len();
        let _ = before; // pool size before return
    }

    #[test]
    fn test_ldap_pool_empty_on_creation() {
        let config = SecurityConfig::default();
        let pool = LdapPool::new(&config);
        assert!(pool.connections.lock().unwrap().is_empty());
    }
}
