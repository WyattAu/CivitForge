#![forbid(unsafe_code)]

use crate::config::SecurityConfig;
use crate::error::{CoreError, Result};
use ldap3::{LdapConn, LdapConnSettings, Scope, SearchEntry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapUserInfo {
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub groups: Vec<String>,
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

        let settings = LdapConnSettings::new().set_no_tls_verify(true);
        let mut conn = LdapConn::with_settings(settings.clone(), &config.ldap_url)
            .map_err(|e| CoreError::Internal(format!("LDAP connection failed: {e}")))?;

        // Bind with service account
        conn.simple_bind(&config.ldap_bind_dn, &config.ldap_bind_password)
            .map_err(|e| CoreError::Auth(format!("LDAP bind failed: {e}")))?
            .success()
            .map_err(|e| CoreError::Auth(format!("LDAP bind rejected: {e}")))?;

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
        let mut user_conn = LdapConn::with_settings(settings, &config.ldap_url)
            .map_err(|e| CoreError::Internal(format!("LDAP connection failed: {e}")))?;

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

        let settings = LdapConnSettings::new().set_no_tls_verify(true);
        let mut conn = LdapConn::with_settings(settings.clone(), &config.ldap_url)
            .map_err(|e| CoreError::Internal(format!("LDAP connection failed: {e}")))?;

        conn.simple_bind(&config.ldap_bind_dn, &config.ldap_bind_password)
            .map_err(|e| CoreError::Auth(format!("LDAP bind failed: {e}")))?
            .success()
            .map_err(|e| CoreError::Auth(format!("LDAP bind rejected: {e}")))?;

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
}
