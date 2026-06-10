#![forbid(unsafe_code)]

use leptos::prelude::*;

use crate::api::client::ApiClient;
use crate::components::{
    Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Spinner,
};
use crate::state::auth::use_auth;

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

#[derive(Clone, PartialEq)]
enum AdminTab {
    AuditLog,
    Users,
    Repos,
    Security,
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

    // Fetch audit log
    let fetch_audit = move || {
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

    // Initial load
    fetch_audit();

    let switch_tab = move |tab: AdminTab| {
        set_active_tab.set(tab.clone());
        match tab {
            AdminTab::AuditLog => fetch_audit(),
            AdminTab::Users => fetch_users(),
            AdminTab::Repos => fetch_repos(),
            AdminTab::Security => fetch_security(),
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
                </nav>
            </div>

            // -- Audit Log Tab --
            <Show when=move || active_tab.get() == AdminTab::AuditLog fallback=|| view! { <div class="hidden"></div> }>
                <div class="space-y-4">
                    <div class="flex items-center gap-3 flex-wrap">
                        <input
                            type="text"
                            placeholder="Filter by action..."
                            class="px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                            prop:value=move || audit_action_filter.get()
                            on:input=move |ev| set_audit_action_filter.set(event_target_value(&ev))
                        />
                        <input
                            type="text"
                            placeholder="Filter by resource type..."
                            class="px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                            prop:value=move || audit_resource_filter.get()
                            on:input=move |ev| set_audit_resource_filter.set(event_target_value(&ev))
                        />
                        <input
                            type="text"
                            placeholder="Filter by actor ID..."
                            class="px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                            prop:value=move || audit_actor_filter.get()
                            on:input=move |ev| set_audit_actor_filter.set(event_target_value(&ev))
                        />
                        <input
                            type="date"
                            placeholder="From..."
                            class="px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                            prop:value=move || audit_date_from.get()
                            on:input=move |ev| set_audit_date_from.set(event_target_value(&ev))
                        />
                        <input
                            type="date"
                            placeholder="To..."
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
        </div>
    }
}
