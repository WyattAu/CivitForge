#![forbid(unsafe_code)]

use leptos::prelude::*;

use crate::api::client::ApiClient;
use crate::components::{
    Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Spinner,
};
use crate::state::auth::use_auth;
use crate::utils::*;

// ── Types ──

#[derive(Debug, Clone, serde::Deserialize)]
struct AuditEvent {
    id: i64,
    actor_id: String,
    action: String,
    resource_type: String,
    resource_id: Option<String>,
    ip_address: Option<String>,
    #[allow(dead_code)]
    user_agent: Option<String>,
    outcome: String,
    created_at: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct AuditLogResponse {
    data: Vec<AuditEvent>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct AuditStats {
    #[allow(dead_code)]
    total_events: i64,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct UserItem {
    id: String,
    username: String,
    email: String,
    #[allow(dead_code)]
    display_name: Option<String>,
    role: String,
    created_at: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct RepoItem {
    id: String,
    #[allow(dead_code)]
    name: String,
    full_name: String,
    description: Option<String>,
    visibility: String,
    created_at: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct SecretScanResult {
    id: String,
    #[allow(dead_code)]
    repo_id: String,
    filename: String,
    secret_type: String,
    severity: String,
    status: String,
    detected_at: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct SecretScanResponse {
    data: Vec<SecretScanResult>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct SlsaCheck {
    name: String,
    passed: bool,
    description: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct SlsaScorecard {
    #[allow(dead_code)]
    repo_id: String,
    score: f64,
    level: String,
    checks: Vec<SlsaCheck>,
    scanned_at: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct AdminEnvironment {
    id: String,
    name: String,
    repo_id: String,
    #[serde(default)]
    repo_full_name: Option<String>,
    #[serde(default)]
    variable_count: i32,
    created_at: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct AdminDeployment {
    id: String,
    repo_id: String,
    #[serde(default)]
    repo_full_name: Option<String>,
    sha: String,
    environment: String,
    status: String,
    created_at: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct AdminTeam {
    id: String,
    name: String,
    #[serde(default)]
    org_id: String,
    #[serde(default)]
    org_name: Option<String>,
    permission_level: String,
    #[serde(default)]
    member_count: i32,
    created_at: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct MergeQueueEntry {
    id: String,
    pr_number: i64,
    #[serde(default)]
    pr_title: String,
    #[serde(default)]
    repo_full_name: Option<String>,
    #[serde(default)]
    branch: String,
    status: String,
    position: i32,
    enqueued_at: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct MergeQueueListResponse {
    items: Vec<MergeQueueEntry>,
    total: i64,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct OidcProviderItem {
    id: String,
    name: String,
    issuer: String,
    client_id: String,
    jwks_uri: String,
    enabled: bool,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct CreateOidcProviderBody {
    name: String,
    issuer: String,
    client_id: String,
    jwks_uri: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct UpdateOidcProviderBody {
    name: Option<String>,
    issuer: Option<String>,
    client_id: Option<String>,
    jwks_uri: Option<String>,
    enabled: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct LdapStatusResponse {
    enabled: bool,
    connected: bool,
    server_url: String,
    bind_dn: String,
    search_base: String,
    group_search_base: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct LdapSyncResult {
    groups_synced: i32,
    users_mapped: i32,
    message: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct LdapTestRequest {
    server_url: String,
    bind_dn: String,
    bind_password: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct LdapSyncGroupRequest {}

#[derive(Clone, PartialEq)]
enum AdminTab {
    AuditLog,
    Users,
    Repos,
    Security,
    Environments,
    Deployments,
    Teams,
    MergeQueue,
    OidcProviders,
    Ldap,
}

// ── Page ──

#[component]
pub fn AdminPage() -> impl IntoView {
    let auth = use_auth();
    let (active_tab, set_active_tab) = signal(AdminTab::AuditLog);

    // Audit log state
    let (audit_events, set_audit_events) = signal(Vec::<AuditEvent>::new());
    let (audit_loading, set_audit_loading) = signal(true);
    let (audit_error, set_audit_error) = signal(None::<String>);
    let (audit_action_filter, set_audit_action_filter) = signal(String::new());
    let (audit_resource_filter, set_audit_resource_filter) = signal(String::new());
    let (audit_actor_filter, set_audit_actor_filter) = signal(String::new());
    let (audit_date_from, set_audit_date_from) = signal(String::new());
    let (audit_date_to, set_audit_date_to) = signal(String::new());

    // Users state
    let (users, set_users) = signal(Vec::<UserItem>::new());
    let (users_loading, set_users_loading) = signal(false);
    let (users_error, set_users_error) = signal(None::<String>);

    // Repos state
    let (repos, set_repos) = signal(Vec::<RepoItem>::new());
    let (repos_loading, set_repos_loading) = signal(false);
    let (repos_error, set_repos_error) = signal(None::<String>);
    let (repo_search, set_repo_search) = signal(String::new());

    // Security state
    let (scan_results, set_scan_results) = signal(Vec::<SecretScanResult>::new());
    let (scan_loading, set_scan_loading) = signal(false);
    let (scan_error, set_scan_error) = signal(None::<String>);
    let (scorecard, set_scorecard) = signal(None::<SlsaScorecard>);
    let (scorecard_loading, set_scorecard_loading) = signal(false);
    let (scorecard_error, set_scorecard_error) = signal(None::<String>);

    // Environments state
    let (env_list, set_env_list) = signal(Vec::<AdminEnvironment>::new());
    let (env_loading, set_env_loading) = signal(false);
    let (env_error, set_env_error) = signal(None::<String>);

    // Deployments state
    let (deploy_list, set_deploy_list) = signal(Vec::<AdminDeployment>::new());
    let (deploy_loading, set_deploy_loading) = signal(false);
    let (deploy_error, set_deploy_error) = signal(None::<String>);

    // Teams state
    let (team_list, set_team_list) = signal(Vec::<AdminTeam>::new());
    let (team_loading, set_team_loading) = signal(false);
    let (team_error, set_team_error) = signal(None::<String>);

    // Merge queue state
    let (merge_queue, set_merge_queue) = signal(Vec::<MergeQueueEntry>::new());
    let (mq_loading, set_mq_loading) = signal(false);
    let (mq_error, set_mq_error) = signal(None::<String>);

    // OIDC state
    let (oidc_providers, set_oidc_providers) = signal(Vec::<OidcProviderItem>::new());
    let (oidc_loading, set_oidc_loading) = signal(false);
    let (oidc_error, set_oidc_error) = signal(None::<String>);
    let (oidc_form_error, set_oidc_form_error) = signal(None::<String>);
    let (show_oidc_form, set_show_oidc_form) = signal(false);
    let (editing_oidc_id, set_editing_oidc_id) = signal(None::<String>);
    let (oidc_test_loading, set_oidc_test_loading) = signal(false);
    let (oidc_test_result, set_oidc_test_result) = signal(None::<String>);
    let (oidc_connected_users, set_oidc_connected_users) = signal(0i64);

    // LDAP state
    let (ldap_status, set_ldap_status) = signal(None::<LdapStatusResponse>);
    let (ldap_loading, set_ldap_loading) = signal(false);
    let (ldap_error, set_ldap_error) = signal(None::<String>);
    let (ldap_test_loading, set_ldap_test_loading) = signal(false);
    let (ldap_test_result, set_ldap_test_result) = signal(None::<String>);
    let (ldap_sync_loading, set_ldap_sync_loading) = signal(false);
    let (ldap_sync_result, set_ldap_sync_result) = signal(None::<LdapSyncResult>);

    // Fetch audit log
    let fetch_audit = move || {
        let is_admin_check = auth.0.with(|a| a.is_authenticated && a.is_admin);
        if !is_admin_check {
            set_audit_error.set(Some("Admin access required. Please sign in as an administrator.".to_string()));
            set_audit_loading.set(false);
            return;
        }
        set_audit_loading.set(true);
        set_audit_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        let action = audit_action_filter.get();
        let resource = audit_resource_filter.get();
        let actor = audit_actor_filter.get();
        let date_from = audit_date_from.get();
        let date_to = audit_date_to.get();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let mut path = "/audit-log?per_page=50".to_string();
            if !action.is_empty() {
                path.push_str(&format!("&action={action}"));
            }
            if !resource.is_empty() {
                path.push_str(&format!("&resource_type={resource}"));
            }
            if !actor.is_empty() {
                path.push_str(&format!("&actor_id={actor}"));
            }
            if !date_from.is_empty() {
                path.push_str(&format!("&from={date_from}"));
            }
            if !date_to.is_empty() {
                path.push_str(&format!("&to={date_to}"));
            }
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<AuditLogResponse>().await {
                        Ok(data) => set_audit_events.set(data.data),
                        Err(_) => set_audit_error.set(Some("Failed to load audit log.".to_string())),
                    }
                }
                Ok(_) => {
                    set_audit_error.set(Some("Failed to load audit log.".to_string()));
                }
                Err(_) => {
                    set_audit_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_audit_loading.set(false);
        });
    };

    // Fetch users
    let fetch_users = move || {
        let is_admin_check = auth.0.with(|a| a.is_authenticated && a.is_admin);
        if !is_admin_check {
            set_users_error.set(Some("Admin access required.".to_string()));
            set_users_loading.set(false);
            return;
        }
        set_users_loading.set(true);
        set_users_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            match client.get("/users?limit=100&offset=0").await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<Vec<UserItem>>().await {
                        Ok(data) => set_users.set(data),
                        Err(_) => set_users_error.set(Some("Failed to load users.".to_string())),
                    }
                }
                Ok(_) => {
                    set_users_error.set(Some("Failed to load users.".to_string()));
                }
                Err(_) => {
                    set_users_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_users_loading.set(false);
        });
    };

    // Fetch repos
    let fetch_repos = move || {
        let is_admin_check = auth.0.with(|a| a.is_authenticated && a.is_admin);
        if !is_admin_check {
            set_repos_error.set(Some("Admin access required.".to_string()));
            set_repos_loading.set(false);
            return;
        }
        set_repos_loading.set(true);
        set_repos_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        let search = repo_search.get();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let mut path = "/admin/repos?limit=50&offset=0".to_string();
            if !search.is_empty() {
                path.push_str(&format!("&search={search}"));
            }
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<Vec<RepoItem>>().await {
                        Ok(data) => set_repos.set(data),
                        Err(_) => set_repos_error.set(Some("Failed to load repos.".to_string())),
                    }
                }
                Ok(_) => {
                    set_repos_error.set(Some("Failed to load repos.".to_string()));
                }
                Err(_) => {
                    set_repos_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_repos_loading.set(false);
        });
    };

    // Fetch security data
    let fetch_security = move || {
        let is_admin_check = auth.0.with(|a| a.is_authenticated && a.is_admin);
        if !is_admin_check {
            set_scan_error.set(Some("Admin access required.".to_string()));
            set_scan_loading.set(false);
            set_scorecard_loading.set(false);
            return;
        }
        set_scan_loading.set(true);
        set_scan_error.set(None);
        set_scorecard_loading.set(true);
        set_scorecard_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            match client.get("/admin/security/secret-scans?limit=50").await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<SecretScanResponse>().await {
                        Ok(data) => set_scan_results.set(data.data),
                        Err(_) => set_scan_error.set(Some("Failed to load secret scan results.".to_string())),
                    }
                }
                Ok(_) => {
                    set_scan_error.set(Some("Failed to load secret scan results.".to_string()));
                }
                Err(_) => {
                    set_scan_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_scan_loading.set(false);
        });
        let token2 = auth.0.with(|a| a.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token2);
            match client.get("/admin/security/slsa-scorecard").await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<SlsaScorecard>().await {
                        Ok(data) => set_scorecard.set(Some(data)),
                        Err(_) => set_scorecard_error.set(Some("Failed to load SLSA scorecard.".to_string())),
                    }
                }
                Ok(_) => {
                    set_scorecard_error.set(Some("Failed to load SLSA scorecard.".to_string()));
                }
                Err(_) => {
                    set_scorecard_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_scorecard_loading.set(false);
        });
    };

    // Fetch environments
    let fetch_environments = move || {
        let is_admin_check = auth.0.with(|a| a.is_authenticated && a.is_admin);
        if !is_admin_check {
            set_env_error.set(Some("Admin access required.".to_string()));
            set_env_loading.set(false);
            return;
        }
        set_env_loading.set(true);
        set_env_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            match client.get("/admin/environments?limit=50").await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<Vec<AdminEnvironment>>().await {
                        set_env_list.set(data);
                    }
                }
                Ok(_) => {
                    set_env_error.set(Some("Failed to load environments.".to_string()));
                }
                Err(_) => {
                    set_env_error.set(Some("Network error.".to_string()));
                }
            }
            set_env_loading.set(false);
        });
    };

    // Fetch deployments
    let fetch_deployments = move || {
        let is_admin_check = auth.0.with(|a| a.is_authenticated && a.is_admin);
        if !is_admin_check {
            set_deploy_error.set(Some("Admin access required.".to_string()));
            set_deploy_loading.set(false);
            return;
        }
        set_deploy_loading.set(true);
        set_deploy_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            match client.get("/admin/deployments?limit=50").await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<Vec<AdminDeployment>>().await {
                        set_deploy_list.set(data);
                    }
                }
                Ok(_) => {
                    set_deploy_error.set(Some("Failed to load deployments.".to_string()));
                }
                Err(_) => {
                    set_deploy_error.set(Some("Network error.".to_string()));
                }
            }
            set_deploy_loading.set(false);
        });
    };

    // Fetch teams
    let fetch_teams = move || {
        let is_admin_check = auth.0.with(|a| a.is_authenticated && a.is_admin);
        if !is_admin_check {
            set_team_error.set(Some("Admin access required.".to_string()));
            set_team_loading.set(false);
            return;
        }
        set_team_loading.set(true);
        set_team_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            match client.get("/admin/teams?limit=50").await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<Vec<AdminTeam>>().await {
                        set_team_list.set(data);
                    }
                }
                Ok(_) => {
                    set_team_error.set(Some("Failed to load teams.".to_string()));
                }
                Err(_) => {
                    set_team_error.set(Some("Network error.".to_string()));
                }
            }
            set_team_loading.set(false);
        });
    };

    // Fetch merge queue
    let fetch_merge_queue = move || {
        let is_admin_check = auth.0.with(|a| a.is_authenticated && a.is_admin);
        if !is_admin_check {
            set_mq_error.set(Some("Admin access required.".to_string()));
            set_mq_loading.set(false);
            return;
        }
        set_mq_loading.set(true);
        set_mq_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            match client.get("/admin/merge-queue?limit=50").await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<MergeQueueListResponse>().await {
                        set_merge_queue.set(data.items);
                    }
                }
                Ok(_) => {
                    set_mq_error.set(Some("Failed to load merge queue.".to_string()));
                }
                Err(_) => {
                    set_mq_error.set(Some("Network error.".to_string()));
                }
            }
            set_mq_loading.set(false);
        });
    };

    // Fetch OIDC providers
    let fetch_oidc = move || {
        let is_admin_check = auth.0.with(|a| a.is_authenticated && a.is_admin);
        if !is_admin_check {
            set_oidc_error.set(Some("Admin access required.".to_string()));
            set_oidc_loading.set(false);
            return;
        }
        set_oidc_loading.set(true);
        set_oidc_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            match client.get("/admin/oidc-providers").await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<Vec<OidcProviderItem>>().await {
                        set_oidc_providers.set(data);
                    }
                }
                Ok(_) => {
                    set_oidc_error.set(Some("Failed to load OIDC providers.".to_string()));
                }
                Err(_) => {
                    set_oidc_error.set(Some("Network error.".to_string()));
                }
            }
            set_oidc_loading.set(false);
        });
    };

    // Fetch LDAP status
    let fetch_ldap = move || {
        let is_admin_check = auth.0.with(|a| a.is_authenticated && a.is_admin);
        if !is_admin_check {
            set_ldap_error.set(Some("Admin access required.".to_string()));
            set_ldap_loading.set(false);
            return;
        }
        set_ldap_loading.set(true);
        set_ldap_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            match client.get("/admin/ldap/status").await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<LdapStatusResponse>().await {
                        set_ldap_status.set(Some(data));
                    }
                }
                Ok(_) => {
                    set_ldap_error.set(Some("Failed to load LDAP status.".to_string()));
                }
                Err(_) => {
                    set_ldap_error.set(Some("Network error.".to_string()));
                }
            }
            set_ldap_loading.set(false);
        });
    };

    // Test OIDC connection
    let test_oidc_connection = move |provider_id: String| {
        set_oidc_test_loading.set(true);
        set_oidc_test_result.set(None);
        let token = auth.0.with(|a| a.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/admin/oidc-providers/{provider_id}/test");
            match client.post(&path, &serde_json::json!({})).await {
                Ok(resp) if resp.status().is_success() => {
                    set_oidc_test_result.set(Some("Connection successful".to_string()));
                }
                Ok(_) => {
                    set_oidc_test_result.set(Some("Connection failed".to_string()));
                }
                Err(_) => {
                    set_oidc_test_result.set(Some("Network error".to_string()));
                }
            }
            set_oidc_test_loading.set(false);
        });
    };

    // Fetch OIDC connected users count
    let fetch_oidc_users_count = move || {
        let token = auth.0.with(|a| a.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            match client.get("/admin/oidc-providers/users-count").await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<serde_json::Value>().await {
                        if let Some(count) = data.get("count").and_then(|v| v.as_i64()) {
                            set_oidc_connected_users.set(count);
                        }
                    }
                }
                _ => {}
            }
        });
    };

    // Test LDAP connection
    let test_ldap_connection = move || {
        set_ldap_test_loading.set(true);
        set_ldap_test_result.set(None);
        let token = auth.0.with(|a| a.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            match client.post("/admin/ldap/test", &serde_json::json!({})).await {
                Ok(resp) if resp.status().is_success() => {
                    set_ldap_test_result.set(Some("LDAP connection successful".to_string()));
                }
                Ok(_) => {
                    set_ldap_test_result.set(Some("LDAP connection failed".to_string()));
                }
                Err(_) => {
                    set_ldap_test_result.set(Some("Network error".to_string()));
                }
            }
            set_ldap_test_loading.set(false);
        });
    };

    // Sync LDAP groups
    let sync_ldap_groups = move || {
        set_ldap_sync_loading.set(true);
        set_ldap_sync_result.set(None);
        let token = auth.0.with(|a| a.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            match client
                .post("/admin/ldap/sync", &LdapSyncGroupRequest {})
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<LdapSyncResult>().await {
                        set_ldap_sync_result.set(Some(data));
                    }
                }
                Ok(_) => {
                    set_ldap_sync_result.set(Some(LdapSyncResult {
                        groups_synced: 0,
                        users_mapped: 0,
                        message: "Sync failed".to_string(),
                    }));
                }
                Err(_) => {
                    set_ldap_sync_result.set(Some(LdapSyncResult {
                        groups_synced: 0,
                        users_mapped: 0,
                        message: "Network error".to_string(),
                    }));
                }
            }
            set_ldap_sync_loading.set(false);
        });
    };

    // Remove from merge queue
    let remove_from_queue = move |entry_id: String| {
        let token = auth.0.with(|a| a.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/admin/merge-queue/{entry_id}");
            let _ = client.delete(&path).await;
            fetch_merge_queue();
        });
    };

    // Reorder merge queue entry
    let reorder_entry = move |entry_id: String, new_position: i32| {
        let token = auth.0.with(|a| a.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/admin/merge-queue/{entry_id}/reorder");
            let body = serde_json::json!({ "new_position": new_position });
            let _ = client.patch(&path, &body).await;
            fetch_merge_queue();
        });
    };

    // Delete OIDC provider
    let delete_oidc_provider = move |provider_id: String| {
        let token = auth.0.with(|a| a.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/admin/oidc-providers/{provider_id}");
            let _ = client.delete(&path).await;
            fetch_oidc();
        });
    };

    // Initial load
    fetch_audit();

    let switch_tab = move |tab: AdminTab| {
        set_active_tab.set(tab.clone());
        match tab {
            AdminTab::AuditLog => fetch_audit(),
            AdminTab::Users => fetch_users(),
            AdminTab::Repos => fetch_repos(),
            AdminTab::Security => fetch_security(),
            AdminTab::Environments => fetch_environments(),
            AdminTab::Deployments => fetch_deployments(),
            AdminTab::Teams => fetch_teams(),
            AdminTab::MergeQueue => fetch_merge_queue(),
            AdminTab::OidcProviders => {
                fetch_oidc();
                fetch_oidc_users_count();
            }
            AdminTab::Ldap => fetch_ldap(),
        }
    };

    // Ban/unban user
    let toggle_ban = move |user_id: String, currently_banned: bool| {
        let token = auth.0.with(|a| a.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let action = if currently_banned { "unban" } else { "ban" };
            let path = format!("/admin/users/{user_id}/{action}");
            let _ = client.patch(&path, &serde_json::json!({})).await;
            fetch_users();
        });
    };

    // Delete repo
    let delete_repo = move |full_name: String| {
        let token = auth.0.with(|a| a.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/admin/repos/{full_name}");
            let _ = client.delete(&path).await;
            fetch_repos();
        });
    };

    let tab_class = |tab: AdminTab, active: AdminTab| -> &'static str {
        if tab == active {
            "px-4 py-2 text-sm font-semibold border-b-2 border-blue-500 text-blue-600 dark:text-blue-400"
        } else {
            "px-4 py-2 text-sm font-medium border-b-2 border-transparent text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300 hover:border-gray-300 dark:hover:border-gray-600"
        }
    };

    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Admin Panel"</h1>
                <p class="mt-1 text-gray-600 dark:text-gray-400">"System administration and monitoring."</p>
            </div>

            // -- Tab Navigation --
            <div class="border-b border-gray-200 dark:border-gray-700">
                <nav class="flex gap-0 -mb-px">
                    <button class=tab_class(AdminTab::AuditLog, active_tab.get()) on:click=move |_| switch_tab(AdminTab::AuditLog)>
                        "Audit Log"
                    </button>
                    <button class=tab_class(AdminTab::Users, active_tab.get()) on:click=move |_| switch_tab(AdminTab::Users)>
                        "Users"
                    </button>
                    <button class=tab_class(AdminTab::Repos, active_tab.get()) on:click=move |_| switch_tab(AdminTab::Repos)>
                        "Repositories"
                    </button>
                    <button class=tab_class(AdminTab::Security, active_tab.get()) on:click=move |_| switch_tab(AdminTab::Security)>
                        "Security"
                    </button>
                    <button class=tab_class(AdminTab::Environments, active_tab.get()) on:click=move |_| switch_tab(AdminTab::Environments)>
                        "Environments"
                    </button>
                    <button class=tab_class(AdminTab::Deployments, active_tab.get()) on:click=move |_| switch_tab(AdminTab::Deployments)>
                        "Deployments"
                    </button>
                    <button class=tab_class(AdminTab::Teams, active_tab.get()) on:click=move |_| switch_tab(AdminTab::Teams)>
                        "Teams"
                    </button>
                    <button class=tab_class(AdminTab::MergeQueue, active_tab.get()) on:click=move |_| switch_tab(AdminTab::MergeQueue)>
                        "Merge Queue"
                    </button>
                    <button class=tab_class(AdminTab::OidcProviders, active_tab.get()) on:click=move |_| switch_tab(AdminTab::OidcProviders)>
                        "OIDC Providers"
                    </button>
                    <button class=tab_class(AdminTab::Ldap, active_tab.get()) on:click=move |_| switch_tab(AdminTab::Ldap)>
                        "LDAP"
                    </button>
                </nav>
            </div>

            // -- Audit Log Tab --
            <Show when=move || active_tab.get() == AdminTab::AuditLog fallback=|| view! { <div class="hidden"></div> }>
                <div class="space-y-4">
                    <div class="flex items-center gap-3 flex-wrap">
                        <input
                            type="text"
                            placeholder="Filter by action..."
                            aria-label="Filter by action"
                            class="px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                            prop:value=move || audit_action_filter.get()
                            on:input=move |ev| set_audit_action_filter.set(event_target_value(&ev))
                        />
                        <input
                            type="text"
                            placeholder="Filter by resource type..."
                            aria-label="Filter by resource type"
                            class="px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                            prop:value=move || audit_resource_filter.get()
                            on:input=move |ev| set_audit_resource_filter.set(event_target_value(&ev))
                        />
                        <input
                            type="text"
                            placeholder="Filter by actor ID..."
                            aria-label="Filter by actor ID"
                            class="px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                            prop:value=move || audit_actor_filter.get()
                            on:input=move |ev| set_audit_actor_filter.set(event_target_value(&ev))
                        />
                        <input
                            type="date"
                            placeholder="From..."
                            aria-label="Filter from date"
                            class="px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                            prop:value=move || audit_date_from.get()
                            on:input=move |ev| set_audit_date_from.set(event_target_value(&ev))
                        />
                        <input
                            type="date"
                            placeholder="To..."
                            aria-label="Filter to date"
                            class="px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                            prop:value=move || audit_date_to.get()
                            on:input=move |ev| set_audit_date_to.set(event_target_value(&ev))
                        />
                        <Button variant=ButtonVariant::Primary on:click=move |_| fetch_audit()>
                            "Apply"
                        </Button>
                    </div>

                    <Show when=move || audit_error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                        <ErrorBanner message=move || audit_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_audit_error.set(None)) />
                    </Show>

                    <Show when=move || audit_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        <div class="flex items-center justify-center py-12">
                            <Spinner />
                        </div>
                    </Show>

                    <Show when=move || !audit_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        <Card>
                            {move || if audit_events.with(|e| e.is_empty()) {
                                view! {
                                    <div class="py-8 text-center text-gray-400 dark:text-gray-500">
                                        "No audit events found."
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="overflow-x-auto">
                                        <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                                            <thead class="bg-gray-50 dark:bg-gray-750">
                                                <tr>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Time"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Actor"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Action"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Resource"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Outcome"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"IP"</th>
                                                </tr>
                                            </thead>
                                            <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
                                                <For each=move || audit_events.get() key=|e| e.id.to_string() let:event>
                                                    {
                                                        let actor_id = event.actor_id.clone();
                                                        let resource_id = event.resource_id.clone();
                                                        let ip_address = event.ip_address.clone();
                                                        let created_at = event.created_at.clone();
                                                        let action = event.action.clone();
                                                        let resource_type = event.resource_type.clone();
                                                        let outcome = event.outcome.clone();
                                                        view! {
                                                            <tr class="hover:bg-gray-50 dark:hover:bg-gray-750">
                                                                <td class="px-4 py-3 text-xs text-gray-500 dark:text-gray-400 whitespace-nowrap">
                                                                    {created_at}
                                                                </td>
                                                                 <td class="px-4 py-3 text-xs font-mono text-gray-700 dark:text-gray-300">
                                                                     {actor_id[..8.min(actor_id.len())].to_string()}
                                                                 </td>
                                                                <td class="px-4 py-3">
                                                                    <Badge color=BadgeColor::Info text=action />
                                                                </td>
                                                                <td class="px-4 py-3 text-xs text-gray-600 dark:text-gray-400">
                                                                    <span>{resource_type}</span>
                                                                    {resource_id.map(|rid| {
                                                                        let rid_short = rid[..8.min(rid.len())].to_string();
                                                                        view! {
                                                                            <span class="text-gray-400 dark:text-gray-500"> " / " </span>
                                                                            <span class="font-mono">{rid_short}</span>
                                                                        }
                                                                    })}
                                                                </td>
                                                                <td class="px-4 py-3">
                                                                    <Badge
                                                                        color=if outcome == "success" { BadgeColor::Success } else { BadgeColor::Danger }
                                                                        text=outcome
                                                                    />
                                                                </td>
                                                                <td class="px-4 py-3 text-xs text-gray-400 dark:text-gray-500 font-mono">
                                                                    {ip_address.unwrap_or_else(|| "-".to_string())}
                                                                </td>
                                                            </tr>
                                                        }
                                                    }
                                                </For>
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }}
                        </Card>
                    </Show>
                </div>
            </Show>

            // -- Users Tab --
            <Show when=move || active_tab.get() == AdminTab::Users fallback=|| view! { <div class="hidden"></div> }>
                <div class="space-y-4">
                    <Show when=move || users_error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                        <ErrorBanner message=move || users_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_users_error.set(None)) />
                    </Show>

                    <Show when=move || users_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        <div class="flex items-center justify-center py-12">
                            <Spinner />
                        </div>
                    </Show>

                    <Show when=move || !users_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        <Card>
                            {move || if users.with(|u| u.is_empty()) {
                                view! {
                                    <div class="py-8 text-center text-gray-400 dark:text-gray-500">
                                        "No users found."
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="overflow-x-auto">
                                        <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                                            <thead class="bg-gray-50 dark:bg-gray-750">
                                                <tr>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"User"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Email"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Role"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Created"</th>
                                                    <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Actions"</th>
                                                </tr>
                                            </thead>
                                            <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
                                                <For each=move || users.get() key=|u| u.id.clone() let:user>
                                                    {
                                                        let username = user.username.clone();
                                                        let user_id = user.id.clone();
                                                        let email = user.email.clone();
                                                        let role = user.role.clone();
                                                        let created_at = user.created_at.clone();
                                                        let user_id_ban = user.id.clone();
                                                        let user_id_short = user_id[..8.min(user_id.len())].to_string();
                                                        view! {
                                                            <tr class="hover:bg-gray-50 dark:hover:bg-gray-750">
                                                                <td class="px-4 py-3">
                                                                    <div class="text-sm font-medium text-gray-900 dark:text-gray-100">{username}</div>
                                                                    <div class="text-xs text-gray-500 dark:text-gray-400 font-mono">{user_id_short}</div>
                                                                </td>
                                                                <td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-400">{email}</td>
                                                                <td class="px-4 py-3">
                                                                    <Badge color=BadgeColor::Info text=role />
                                                                </td>
                                                                <td class="px-4 py-3 text-xs text-gray-400 dark:text-gray-500">{created_at}</td>
                                                                <td class="px-4 py-3 text-right">
                                                                    <button
                                                                        class="text-sm text-red-600 dark:text-red-400 hover:underline"
                                                                        on:click=move |_| toggle_ban(user_id_ban.clone(), false)
                                                                    >
                                                                        "Ban"
                                                                    </button>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }
                                                </For>
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }}
                        </Card>
                    </Show>
                </div>
            </Show>

            // -- Repos Tab --
            <Show when=move || active_tab.get() == AdminTab::Repos fallback=|| view! { <div class="hidden"></div> }>
                <div class="space-y-4">
                    <div class="flex items-center gap-3">
                        <input
                            type="text"
                            placeholder="Search repositories..."
                            aria-label="Search repositories"
                            class="flex-1 px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                            prop:value=move || repo_search.get()
                            on:input=move |ev| set_repo_search.set(event_target_value(&ev))
                        />
                        <Button variant=ButtonVariant::Primary on:click=move |_| fetch_repos()>
                            "Search"
                        </Button>
                    </div>

                    <Show when=move || repos_error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                        <ErrorBanner message=move || repos_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_repos_error.set(None)) />
                    </Show>

                    <Show when=move || repos_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        <div class="flex items-center justify-center py-12">
                            <Spinner />
                        </div>
                    </Show>

                    <Show when=move || !repos_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        <Card>
                            {move || if repos.with(|r| r.is_empty()) {
                                view! {
                                    <div class="py-8 text-center text-gray-400 dark:text-gray-500">
                                        "No repositories found."
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="overflow-x-auto">
                                        <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                                            <thead class="bg-gray-50 dark:bg-gray-750">
                                                <tr>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Repository"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Description"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Visibility"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Created"</th>
                                                    <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Actions"</th>
                                                </tr>
                                            </thead>
                                            <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
                                                <For each=move || repos.get() key=|r| r.id.clone() let:repo>
                                                    {
                                                         let full_name = repo.full_name.clone();
                                                         let description = repo.description.clone().unwrap_or_else(|| "-".to_string());
                                                         let visibility = repo.visibility.clone();
                                                         let created_at = repo.created_at.clone();
                                                         let repo_full = full_name.clone();
                                                         let full_name_display = full_name.clone();
                                                         let vis_color = match visibility.as_str() {
                                                             "public" => BadgeColor::Success,
                                                             "internal" => BadgeColor::Info,
                                                             _ => BadgeColor::Neutral,
                                                         };
                                                         view! {
                                                             <tr class="hover:bg-gray-50 dark:hover:bg-gray-750">
                                                                 <td class="px-4 py-3">
                                                                     <a href=format!("/repos/{full_name}") class="text-sm font-medium text-blue-600 dark:text-blue-400 hover:underline">
                                                                         {full_name_display}
                                                                     </a>
                                                                </td>
                                                                <td class="px-4 py-3 text-sm text-gray-500 dark:text-gray-400 max-w-xs truncate">
                                                                    {description}
                                                                </td>
                                                                <td class="px-4 py-3">
                                                                    <Badge
                                                                        color=vis_color
                                                                        text=visibility
                                                                    />
                                                                </td>
                                                                <td class="px-4 py-3 text-xs text-gray-400 dark:text-gray-500">{created_at}</td>
                                                                <td class="px-4 py-3 text-right">
                                                                    <button
                                                                        class="text-sm text-red-600 dark:text-red-400 hover:underline"
                                                                        on:click=move |_| delete_repo(repo_full.clone())
                                                                    >
                                                                        "Force Delete"
                                                                    </button>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }
                                                </For>
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }}
                        </Card>
                    </Show>
                </div>
            </Show>

            // -- Security Tab --
            <Show when=move || active_tab.get() == AdminTab::Security fallback=|| view! { <div class="hidden"></div> }>
                <div class="space-y-6">
                    // Secret Scan Results
                    <div>
                        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-3">"Secret Scan Results"</h3>
                        <Show when=move || scan_error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                            <ErrorBanner message=move || scan_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_scan_error.set(None)) />
                        </Show>
                        <Show when=move || scan_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                            <div class="flex items-center justify-center py-8">
                                <Spinner />
                            </div>
                        </Show>
                        <Show when=move || !scan_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                            <Card>
                                {move || if scan_results.with(|r| r.is_empty()) {
                                    view! {
                                        <div class="py-8 text-center text-green-600 dark:text-green-400">
                                            "No secrets detected. All clear!"
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="overflow-x-auto">
                                            <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                                                <thead class="bg-gray-50 dark:bg-gray-750">
                                                    <tr>
                                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"File"</th>
                                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Secret Type"</th>
                                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Severity"</th>
                                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Status"</th>
                                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Detected"</th>
                                                    </tr>
                                                </thead>
                                                <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
                                                    <For each=move || scan_results.get() key=|r| r.id.clone() let:result>
                                                        {
                                                            let filename = result.filename.clone();
                                                            let secret_type = result.secret_type.clone();
                                                            let severity = result.severity.clone();
                                                            let status = result.status.clone();
                                                            let detected_at = result.detected_at.clone();
                                                            let sev_color = match severity.as_str() {
                                                                "critical" | "high" => BadgeColor::Danger,
                                                                "medium" => BadgeColor::Warning,
                                                                _ => BadgeColor::Info,
                                                            };
                                                            let stat_color = match status.as_str() {
                                                                "resolved" => BadgeColor::Success,
                                                                "ignored" => BadgeColor::Neutral,
                                                                _ => BadgeColor::Danger,
                                                            };
                                                            view! {
                                                                <tr class="hover:bg-gray-50 dark:hover:bg-gray-750">
                                                                    <td class="px-4 py-3 text-sm font-mono text-gray-900 dark:text-gray-100">{filename}</td>
                                                                    <td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-400">{secret_type}</td>
                                                                    <td class="px-4 py-3"><Badge color=sev_color text=severity /></td>
                                                                    <td class="px-4 py-3"><Badge color=stat_color text=status /></td>
                                                                    <td class="px-4 py-3 text-xs text-gray-400 dark:text-gray-500">{detected_at}</td>
                                                                </tr>
                                                            }
                                                        }
                                                    </For>
                                                </tbody>
                                            </table>
                                        </div>
                                    }.into_any()
                                }}
                            </Card>
                        </Show>
                    </div>

                    // SLSA Scorecard
                    <div>
                        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-3">"SLSA Scorecard"</h3>
                        <Show when=move || scorecard_error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                            <ErrorBanner message=move || scorecard_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_scorecard_error.set(None)) />
                        </Show>
                        <Show when=move || scorecard_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                            <div class="flex items-center justify-center py-8">
                                <Spinner />
                            </div>
                        </Show>
                        <Show when=move || !scorecard_loading.get() && scorecard.get().is_some() && scorecard_error.get().is_none() fallback=|| view! { <div class="hidden"></div> }>
                            {move || scorecard.get().map(|sc| {
                                let level_color = match sc.level.as_str() {
                                    "SLSA 3" | "SLSA 4" => BadgeColor::Success,
                                    "SLSA 2" => BadgeColor::Warning,
                                    _ => BadgeColor::Info,
                                };
                                let checks = StoredValue::new(sc.checks.clone());
                                view! {
                                    <Card>
                                        <div class="space-y-4">
                                            <div class="flex items-center gap-4">
                                                <div class="text-center">
                                                    <div class="text-3xl font-bold text-gray-900 dark:text-gray-100">
                                                        {format!("{:.1}", sc.score)}
                                                    </div>
                                                    <div class="text-xs text-gray-500 dark:text-gray-400">"Score"</div>
                                                </div>
                                                <Badge color=level_color text=sc.level.clone() />
                                                <span class="text-xs text-gray-400 dark:text-gray-500">
                                                    "Scanned: " {sc.scanned_at.clone()}
                                                </span>
                                            </div>
                                            <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
                                                <h4 class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">"Checks"</h4>
                                                <div class="space-y-2">
                                                    <For each=move || checks.get_value() key=|c| c.name.clone() let:check>
                                                        {
                                                            let check_name = check.name.clone();
                                                            let check_desc = check.description.clone();
                                                            let check_passed = check.passed;
                                                            view! {
                                                                <div class="flex items-center justify-between py-1">
                                                                    <div class="flex items-center gap-2">
                                                                        {if check_passed {
                                                                            view! { <span class="text-green-500">"\u{2713}"</span> }.into_any()
                                                                        } else {
                                                                            view! { <span class="text-red-500">"\u{2717}"</span> }.into_any()
                                                                        }}
                                                                        <span class="text-sm text-gray-900 dark:text-gray-100">{check_name}</span>
                                                                    </div>
                                                                    <span class="text-xs text-gray-500 dark:text-gray-400">{check_desc}</span>
                                                                </div>
                                                            }
                                                        }
                                                    </For>
                                                </div>
                                            </div>
                                        </div>
                                    </Card>
                                }
                            })}
                        </Show>
                        <Show when=move || !scorecard_loading.get() && scorecard.get().is_none() && scorecard_error.get().is_none() fallback=|| view! { <div class="hidden"></div> }>
                            <Card>
                                <div class="py-8 text-center">
                                    <p class="text-gray-500 dark:text-gray-400">"No SLSA scorecard data available."</p>
                                    <div class="mt-4">
                                        <Button variant=ButtonVariant::Primary on:click=move |_| fetch_security()>
                                            "Run Scan"
                                        </Button>
                                    </div>
                                </div>
                            </Card>
                        </Show>
                    </div>
                </div>
            </Show>

            // -- Environments Tab --
            <Show when=move || active_tab.get() == AdminTab::Environments fallback=|| view! { <div class="hidden"></div> }>
                <div class="space-y-4">
                    <Show when=move || env_error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                        <ErrorBanner message=move || env_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_env_error.set(None)) />
                    </Show>
                    <Show when=move || env_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        <div class="flex items-center justify-center py-12"><Spinner /></div>
                    </Show>
                    <Show when=move || !env_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        <Card>
                            {move || if env_list.get().is_empty() {
                                view! { <div class="py-8 text-center text-gray-400 dark:text-gray-500">"No environments found."</div> }.into_any()
                            } else {
                                view! {
                                    <div class="overflow-x-auto">
                                        <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                                            <thead class="bg-gray-50 dark:bg-gray-750">
                                                <tr>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Name"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Repository"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Variables"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Created"</th>
                                                </tr>
                                            </thead>
                                            <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
                                                <For each=move || env_list.get() key=|e| e.id.clone() let:env>
                                                    {
                                                        let name = env.name.clone();
                                                        let repo = env.repo_full_name.clone().unwrap_or_default();
                                                        let var_count = env.variable_count;
                                                        let created_at = env.created_at.clone();
                                                        view! {
                                                            <tr class="hover:bg-gray-50 dark:hover:bg-gray-750">
                                                                <td class="px-4 py-3 text-sm font-medium text-gray-900 dark:text-gray-100">{name}</td>
                                                                <td class="px-4 py-3 text-xs text-gray-600 dark:text-gray-400 font-mono">{repo}</td>
                                                                <td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-400">{var_count.to_string()}</td>
                                                                <td class="px-4 py-3 text-xs text-gray-400 dark:text-gray-500">{created_at}</td>
                                                            </tr>
                                                        }
                                                    }
                                                </For>
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }}
                        </Card>
                    </Show>
                </div>
            </Show>

            // -- Deployments Tab --
            <Show when=move || active_tab.get() == AdminTab::Deployments fallback=|| view! { <div class="hidden"></div> }>
                <div class="space-y-4">
                    <Show when=move || deploy_error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                        <ErrorBanner message=move || deploy_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_deploy_error.set(None)) />
                    </Show>
                    <Show when=move || deploy_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        <div class="flex items-center justify-center py-12"><Spinner /></div>
                    </Show>
                    <Show when=move || !deploy_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        <Card>
                            {move || if deploy_list.get().is_empty() {
                                view! { <div class="py-8 text-center text-gray-400 dark:text-gray-500">"No deployments found."</div> }.into_any()
                            } else {
                                view! {
                                    <div class="overflow-x-auto">
                                        <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                                            <thead class="bg-gray-50 dark:bg-gray-750">
                                                <tr>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Status"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Environment"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"SHA"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Repository"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Created"</th>
                                                </tr>
                                            </thead>
                                            <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
                                                <For each=move || deploy_list.get() key=|d| d.id.clone() let:deploy>
                                                    {
                                                        let status = deploy.status.clone();
                                                        let environment = deploy.environment.clone();
                                                        let sha_short = deploy.sha[..7.min(deploy.sha.len())].to_string();
                                                        let repo = deploy.repo_full_name.clone().unwrap_or_default();
                                                        let created_at = deploy.created_at.clone();
                                                        let status_color_val = match status.as_str() {
                                                            "success" => BadgeColor::Success,
                                                            "in_progress" | "pending" => BadgeColor::Warning,
                                                            "failure" | "error" | "cancelled" => BadgeColor::Danger,
                                                            _ => BadgeColor::Neutral,
                                                        };
                                                        view! {
                                                            <tr class="hover:bg-gray-50 dark:hover:bg-gray-750">
                                                                <td class="px-4 py-3"><Badge color=status_color_val text=status /></td>
                                                                <td class="px-4 py-3 text-sm font-medium text-gray-900 dark:text-gray-100">{environment}</td>
                                                                <td class="px-4 py-3 text-xs font-mono text-gray-600 dark:text-gray-400">{sha_short}</td>
                                                                <td class="px-4 py-3 text-xs text-gray-600 dark:text-gray-400 font-mono">{repo}</td>
                                                                <td class="px-4 py-3 text-xs text-gray-400 dark:text-gray-500">{created_at}</td>
                                                            </tr>
                                                        }
                                                    }
                                                </For>
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }}
                        </Card>
                    </Show>
                </div>
            </Show>

            // -- Teams Tab --
            <Show when=move || active_tab.get() == AdminTab::Teams fallback=|| view! { <div class="hidden"></div> }>
                <div class="space-y-4">
                    <Show when=move || team_error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                        <ErrorBanner message=move || team_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_team_error.set(None)) />
                    </Show>
                    <Show when=move || team_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        <div class="flex items-center justify-center py-12"><Spinner /></div>
                    </Show>
                    <Show when=move || !team_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        <Card>
                            {move || if team_list.get().is_empty() {
                                view! { <div class="py-8 text-center text-gray-400 dark:text-gray-500">"No teams found."</div> }.into_any()
                            } else {
                                view! {
                                    <div class="overflow-x-auto">
                                        <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                                            <thead class="bg-gray-50 dark:bg-gray-750">
                                                <tr>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Team"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Organization"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Permission"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Members"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Created"</th>
                                                </tr>
                                            </thead>
                                            <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
                                                <For each=move || team_list.get() key=|t| t.id.clone() let:team>
                                                    {
                                                        let name = team.name.clone();
                                                        let org = team.org_name.clone().unwrap_or_default();
                                                        let permission = team.permission_level.clone();
                                                        let member_count = team.member_count;
                                                        let created_at = team.created_at.clone();
                                                        let perm_color = match permission.as_str() {
                                                            "admin" => BadgeColor::Danger,
                                                            "write" => BadgeColor::Warning,
                                                            "read" => BadgeColor::Info,
                                                            _ => BadgeColor::Neutral,
                                                        };
                                                        view! {
                                                            <tr class="hover:bg-gray-50 dark:hover:bg-gray-750">
                                                                <td class="px-4 py-3 text-sm font-medium text-gray-900 dark:text-gray-100">{name}</td>
                                                                <td class="px-4 py-3 text-xs text-gray-600 dark:text-gray-400 font-mono">{org}</td>
                                                                <td class="px-4 py-3"><Badge color=perm_color text=permission /></td>
                                                                <td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-400">{member_count.to_string()}</td>
                                                                <td class="px-4 py-3 text-xs text-gray-400 dark:text-gray-500">{created_at}</td>
                                                            </tr>
                                                        }
                                                    }
                                                </For>
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }}
                        </Card>
                    </Show>
                </div>
            </Show>

            // -- Merge Queue Tab --
            <Show when=move || active_tab.get() == AdminTab::MergeQueue fallback=|| view! { <div class="hidden"></div> }>
                <div class="space-y-4">
                    <Show when=move || mq_error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                        <ErrorBanner message=move || mq_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_mq_error.set(None)) />
                    </Show>
                    <Show when=move || mq_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        <div class="flex items-center justify-center py-12"><Spinner /></div>
                    </Show>
                    <Show when=move || !mq_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        <Card>
                            {move || if merge_queue.get().is_empty() {
                                view! { <div class="py-8 text-center text-gray-400 dark:text-gray-500">"No entries in the merge queue."</div> }.into_any()
                            } else {
                                view! {
                                    <div class="overflow-x-auto">
                                        <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                                            <thead class="bg-gray-50 dark:bg-gray-750">
                                                <tr>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Position"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"PR"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Repository"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Branch"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Status"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Enqueued"</th>
                                                    <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Actions"</th>
                                                </tr>
                                            </thead>
                                            <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
                                                <For each=move || merge_queue.get() key=|e| e.id.clone() let:entry>
                                                    {
                                                        let pr_title = entry.pr_title.clone();
                                                        let pr_number = entry.pr_number;
                                                        let repo = entry.repo_full_name.clone().unwrap_or_default();
                                                        let branch = entry.branch.clone();
                                                        let status = entry.status.clone();
                                                        let position = entry.position;
                                                        let enqueued_at = entry.enqueued_at.clone();
                                                        let entry_id = entry.id.clone();
                                                        let entry_id2 = entry.id.clone();
                                                        let entry_id3 = entry.id.clone();
                                                        let status_color_val = match status.as_str() {
                                                            "merged" => BadgeColor::Success,
                                                            "running" | "queued" => BadgeColor::Warning,
                                                            "failed" | "cancelled" => BadgeColor::Danger,
                                                            _ => BadgeColor::Neutral,
                                                        };
                                                        view! {
                                                            <tr class="hover:bg-gray-50 dark:hover:bg-gray-750">
                                                                <td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-400">
                                                                    <div class="flex items-center gap-1">
                                                                        <button
                                                                            class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 disabled:opacity-30"
                                                                            disabled=move || position <= 1
                                                                            on:click=move |_| reorder_entry(entry_id3.clone(), position - 1)
                                                                        >
                                                                            "\u{25B2}"
                                                                        </button>
                                                                        <span class="w-6 text-center">{position.to_string()}</span>
                                                                        <button
                                                                            class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                                                                            on:click=move |_| reorder_entry(entry_id2.clone(), position + 1)
                                                                        >
                                                                            "\u{25BC}"
                                                                        </button>
                                                                    </div>
                                                                </td>
                                                                <td class="px-4 py-3 text-sm font-medium text-gray-900 dark:text-gray-100">
                                                                    {format!("PR #{pr_number} {pr_title}")}
                                                                </td>
                                                                <td class="px-4 py-3 text-xs text-gray-600 dark:text-gray-400 font-mono">{repo}</td>
                                                                <td class="px-4 py-3 text-xs font-mono text-gray-600 dark:text-gray-400">{branch}</td>
                                                                <td class="px-4 py-3"><Badge color=status_color_val text=status /></td>
                                                                <td class="px-4 py-3 text-xs text-gray-400 dark:text-gray-500">{enqueued_at}</td>
                                                                <td class="px-4 py-3 text-right">
                                                                    <button
                                                                        class="text-sm text-red-600 dark:text-red-400 hover:underline"
                                                                        on:click=move |_| remove_from_queue(entry_id.clone())
                                                                    >
                                                                        "Remove"
                                                                    </button>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }
                                                </For>
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }}
                        </Card>
                    </Show>
                </div>
            </Show>

            // -- OIDC Providers Tab --
            <Show when=move || active_tab.get() == AdminTab::OidcProviders fallback=|| view! { <div class="hidden"></div> }>
                <div class="space-y-4">
                    <div class="flex items-center justify-between">
                        <div>
                            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">"OIDC Providers"</h3>
                            <p class="text-sm text-gray-500 dark:text-gray-400">
                                {move || format!("{} connected users", oidc_connected_users.get())}
                            </p>
                        </div>
                        <Button variant=ButtonVariant::Primary on:click=move |_| {
                            set_show_oidc_form.set(!show_oidc_form.get());
                            set_editing_oidc_id.set(None);
                        }>
                            {move || if show_oidc_form.get() { "Cancel" } else { "Add Provider" }}
                        </Button>
                    </div>

                    // OIDC Test Connection Result
                    <Show when=move || oidc_test_result.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                        {move || {
                            let msg = oidc_test_result.get().unwrap_or_default();
                            let is_success = msg.contains("successful");
                            view! {
                                <div class=format!("p-3 border-l-4 text-sm {}", if is_success { "bg-green-50 dark:bg-green-900/20 border-green-500 dark:border-green-400 text-green-700 dark:text-green-400" } else { "bg-red-50 dark:bg-red-900/20 border-red-500 dark:border-red-400 text-red-700 dark:text-red-400" })>
                                    {msg}
                                </div>
                            }
                        }}
                    </Show>

                    // OIDC Add/Edit Form
                    <Show when=move || show_oidc_form.get() fallback=|| view! { <div class="hidden"></div> }>
                        {move || {
                            let title = if editing_oidc_id.get().is_some() { "Edit Provider".to_string() } else { "Add OIDC Provider".to_string() };
                            Some(view! {
                                <Card title=title>
                                    <form on:submit=move |ev: leptos::ev::SubmitEvent| {
                                        ev.prevent_default();
                                        set_oidc_form_error.set(None);
                                        let name = get_input_value("oidc-name");
                                        let issuer = get_input_value("oidc-issuer");
                                        let client_id = get_input_value("oidc-client-id");
                                        let jwks_uri = get_input_value("oidc-jwks-uri");

                                        if name.trim().is_empty() || issuer.trim().is_empty() {
                                            set_oidc_form_error.set(Some("Name and issuer are required.".to_string()));
                                            return;
                                        }

                                        let token = auth.0.with(|a| a.token.clone());
                                        let is_edit = editing_oidc_id.get();
                                        leptos::task::spawn_local(async move {
                                            let client = ApiClient::new(token);
                                            if let Some(edit_id) = is_edit {
                                                let body = UpdateOidcProviderBody {
                                                    name: Some(name.trim().to_string()),
                                                    issuer: Some(issuer.trim().to_string()),
                                                    client_id: Some(client_id.trim().to_string()),
                                                    jwks_uri: Some(jwks_uri.trim().to_string()),
                                                    enabled: None,
                                                };
                                                let path = format!("/admin/oidc-providers/{edit_id}");
                                                let _ = client.patch(&path, &body).await;
                                            } else {
                                                let body = CreateOidcProviderBody {
                                                    name: name.trim().to_string(),
                                                    issuer: issuer.trim().to_string(),
                                                    client_id: client_id.trim().to_string(),
                                                    jwks_uri: jwks_uri.trim().to_string(),
                                                };
                                                let _ = client.post("/admin/oidc-providers", &body).await;
                                            }
                                            set_show_oidc_form.set(false);
                                            set_editing_oidc_id.set(None);
                                            fetch_oidc();
                                        });
                                    } class="space-y-4">
                                        <Show when=move || oidc_form_error.get().is_some()>
                                            <ErrorBanner message=move || oidc_form_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_oidc_form_error.set(None)) />
                                        </Show>
                                        <crate::components::Input
                                            label="Provider Name"
                                            name="oidc-name"
                                            id="oidc-name"
                                            input_type=crate::components::InputType::Text
                                            placeholder="e.g. google, github"
                                            required=true
                                        />
                                        <crate::components::Input
                                            label="Issuer URL"
                                            name="oidc-issuer"
                                            id="oidc-issuer"
                                            input_type=crate::components::InputType::Text
                                            placeholder="https://accounts.google.com"
                                            required=true
                                        />
                                        <crate::components::Input
                                            label="Client ID"
                                            name="oidc-client-id"
                                            id="oidc-client-id"
                                            input_type=crate::components::InputType::Text
                                            placeholder="OAuth2 client ID"
                                            required=true
                                        />
                                        <crate::components::Input
                                            label="JWKS URI"
                                            name="oidc-jwks-uri"
                                            id="oidc-jwks-uri"
                                            input_type=crate::components::InputType::Text
                                            placeholder="https://example.com/.well-known/jwks.json"
                                        />
                                        <div>
                                            <Button variant=ButtonVariant::Primary>
                                                {move || if editing_oidc_id.get().is_some() { "Update Provider" } else { "Add Provider" }}
                                            </Button>
                                        </div>
                                    </form>
                                </Card>
                            })
                        }}
                    </Show>

                    <Show when=move || oidc_error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                        <ErrorBanner message=move || oidc_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_oidc_error.set(None)) />
                    </Show>
                    <Show when=move || oidc_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        <div class="flex items-center justify-center py-12"><Spinner /></div>
                    </Show>
                    <Show when=move || !oidc_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        <Card>
                            {move || if oidc_providers.get().is_empty() {
                                view! { <div class="py-8 text-center text-gray-400 dark:text-gray-500">"No OIDC providers configured."</div> }.into_any()
                            } else {
                                view! {
                                    <div class="overflow-x-auto">
                                        <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                                            <thead class="bg-gray-50 dark:bg-gray-750">
                                                <tr>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Name"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Issuer"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Client ID"</th>
                                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Status"</th>
                                                    <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Actions"</th>
                                                </tr>
                                            </thead>
                                            <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
                                                <For each=move || oidc_providers.get() key=|p| p.id.clone() let:provider>
                                                    {
                                                        let p_name = provider.name.clone();
                                                        let p_issuer = provider.issuer.clone();
                                                        let p_client_id = provider.client_id.clone();
                                                        let p_enabled = provider.enabled;
                                                        let p_id = provider.id.clone();
                                                        let p_id2 = provider.id.clone();
                                                        view! {
                                                            <tr class="hover:bg-gray-50 dark:hover:bg-gray-750">
                                                                <td class="px-4 py-3 text-sm font-medium text-gray-900 dark:text-gray-100">{p_name}</td>
                                                                <td class="px-4 py-3 text-xs text-gray-600 dark:text-gray-400 font-mono max-w-xs truncate">{p_issuer}</td>
                                                                <td class="px-4 py-3 text-xs text-gray-600 dark:text-gray-400 font-mono">{p_client_id}</td>
                                                                <td class="px-4 py-3">
                                                                    <Badge color=if p_enabled { BadgeColor::Success } else { BadgeColor::Neutral } text=if p_enabled { "Enabled".to_string() } else { "Disabled".to_string() } />
                                                                </td>
                                                                <td class="px-4 py-3 text-right">
                                                                    <button
                                                                        class="text-sm text-blue-600 dark:text-blue-400 hover:underline mr-3"
                                                                        disabled=oidc_test_loading.get()
                                                                        on:click=move |_| test_oidc_connection(p_id.clone())
                                                                    >
                                                                        "Test"
                                                                    </button>
                                                                    <button
                                                                        class="text-sm text-red-600 dark:text-red-400 hover:underline"
                                                                        on:click=move |_| delete_oidc_provider(p_id2.clone())
                                                                    >
                                                                        "Delete"
                                                                    </button>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }
                                                </For>
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }}
                        </Card>
                    </Show>
                </div>
            </Show>

            // -- LDAP Tab --
            <Show when=move || active_tab.get() == AdminTab::Ldap fallback=|| view! { <div class="hidden"></div> }>
                <div class="space-y-6">
                    <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">"LDAP Group Sync"</h3>

                    <Show when=move || ldap_error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                        <ErrorBanner message=move || ldap_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_ldap_error.set(None)) />
                    </Show>

                    <Show when=move || ldap_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        <div class="flex items-center justify-center py-8">
                            <Spinner />
                        </div>
                    </Show>

                    <Show when=move || !ldap_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        // Connection Status Card
                        <Card>
                            <div class="space-y-4">
                                <div class="flex items-center justify-between">
                                    <h4 class="text-sm font-semibold text-gray-900 dark:text-gray-100">"Connection Status"</h4>
                                    {move || {
                                        let connected = ldap_status.get().map(|s| s.connected).unwrap_or(false);
                                        let enabled = ldap_status.get().map(|s| s.enabled).unwrap_or(false);
                                        if !enabled {
                                            view! { <Badge color=BadgeColor::Neutral text="LDAP Disabled".to_string() /> }.into_any()
                                        } else if connected {
                                            view! { <Badge color=BadgeColor::Success text="Connected".to_string() /> }.into_any()
                                        } else {
                                            view! { <Badge color=BadgeColor::Danger text="Disconnected".to_string() /> }.into_any()
                                        }
                                    }}
                                </div>

                                {move || ldap_status.get().map(|s| {
                                    view! {
                                        <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 text-sm">
                                            <div>
                                                <span class="text-gray-500 dark:text-gray-400">"Server URL"</span>
                                                <p class="font-mono text-gray-900 dark:text-gray-100">{s.server_url}</p>
                                            </div>
                                            <div>
                                                <span class="text-gray-500 dark:text-gray-400">"Bind DN"</span>
                                                <p class="font-mono text-gray-900 dark:text-gray-100">{s.bind_dn}</p>
                                            </div>
                                            <div>
                                                <span class="text-gray-500 dark:text-gray-400">"Search Base"</span>
                                                <p class="font-mono text-gray-900 dark:text-gray-100">{s.search_base}</p>
                                            </div>
                                        </div>
                                    }
                                })}

                                <div class="flex gap-3">
                                    <Button variant=ButtonVariant::Primary on:click=move |_| test_ldap_connection() disabled=ldap_test_loading.get()>
                                        {move || if ldap_test_loading.get() { "Testing..." } else { "Test Connection" }}
                                    </Button>
                                </div>

                                // Test Connection Result
                                <Show when=move || ldap_test_result.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                                    {move || {
                                        let msg = ldap_test_result.get().unwrap_or_default();
                                        let is_success = msg.contains("successful");
                                        view! {
                                            <div class=format!("p-3 border-l-4 text-sm {}", if is_success { "bg-green-50 dark:bg-green-900/20 border-green-500 dark:border-green-400 text-green-700 dark:text-green-400" } else { "bg-red-50 dark:bg-red-900/20 border-red-500 dark:border-red-400 text-red-700 dark:text-red-400" })>
                                                {msg}
                                            </div>
                                        }
                                    }}
                                </Show>
                            </div>
                        </Card>

                        // Sync Card
                        <Card>
                            <div class="space-y-4">
                                <div class="flex items-center justify-between">
                                    <div>
                                        <h4 class="text-sm font-semibold text-gray-900 dark:text-gray-100">"Group Synchronization"</h4>
                                        <p class="text-xs text-gray-500 dark:text-gray-400">"Sync LDAP groups to CivitForge teams"</p>
                                    </div>
                                    <Button variant=ButtonVariant::Primary on:click=move |_| sync_ldap_groups() disabled=ldap_sync_loading.get()>
                                        {move || if ldap_sync_loading.get() { "Syncing..." } else { "Sync Groups" }}
                                    </Button>
                                </div>

                                // Sync Result
                                <Show when=move || ldap_sync_result.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                                    {move || {
                                        let result = ldap_sync_result.get().unwrap();
                                        view! {
                                            <div class="p-3 bg-blue-50 dark:bg-blue-900/20 border-l-4 border-blue-500 dark:border-blue-400 text-sm text-blue-700 dark:text-blue-400">
                                                <p class="font-medium">{result.message}</p>
                                                <div class="mt-2 flex gap-4 text-xs">
                                                    <span>"Groups synced: " <strong>{result.groups_synced.to_string()}</strong></span>
                                                    <span>"Users mapped: " <strong>{result.users_mapped.to_string()}</strong></span>
                                                </div>
                                            </div>
                                        }
                                    }}
                                </Show>
                            </div>
                        </Card>
                    </Show>
                </div>
            </Show>
        </div>
    }
}
