#![forbid(unsafe_code)]

// Re-export from civit-auth
pub use civit_auth::ldap::{LdapAuth, LdapConfig, LdapPool, LdapUserInfo};

use crate::config::SecurityConfig;

impl From<&SecurityConfig> for LdapConfig {
    fn from(config: &SecurityConfig) -> Self {
        Self {
            enabled: config.ldap_enabled,
            url: config.ldap_url.clone(),
            bind_dn: config.ldap_bind_dn.clone(),
            bind_password: config.ldap_bind_password.clone(),
            user_search_base: config.ldap_user_search_base.clone(),
            user_filter: config.ldap_user_filter.clone(),
            group_search_base: config.ldap_group_search_base.clone(),
            group_search_filter: config.ldap_group_search_filter.clone(),
            max_connections: config.ldap_max_connections,
            tls_ca_path: config.ldap_tls_ca_path.clone(),
            connection_timeout_secs: config.ldap_connection_timeout_secs,
        }
    }
}
