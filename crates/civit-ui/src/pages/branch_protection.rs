#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::components::{Button, ButtonVariant, Card, ErrorBanner, Input, InputType, Spinner};
use crate::state::auth::use_auth;
use crate::utils::get_input_value;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BranchProtectionRule {
    pub id: String,
    pub branch_pattern: String,
    pub require_pull_request: bool,
    pub required_approving_reviews: i32,
    pub enforce_admins: bool,
    pub allow_force_pushes: bool,
    pub allow_deletions: bool,
    pub required_status_checks: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct CreateProtectionBody {
    branch_pattern: String,
    require_pull_request: bool,
    required_approving_reviews: i32,
    enforce_admins: bool,
    allow_force_pushes: bool,
    allow_deletions: bool,
    required_status_checks: Vec<String>,
}

#[component]
pub fn BranchProtectionPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let auth = use_auth();

    let (rules, set_rules) = signal(Vec::<BranchProtectionRule>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (saving, set_saving) = signal(false);
    let (show_form, set_show_form) = signal(false);
    let (editing_id, set_editing_id) = signal(None::<String>);

    let (req_pr, set_req_pr) = signal(true);
    let (num_reviews, set_num_reviews) = signal(1);
    let (enforce_admins, set_enforce_admins) = signal(false);
    let (allow_force, set_allow_force) = signal(false);
    let (allow_delete, set_allow_delete) = signal(false);
    let (status_checks, set_status_checks) = signal(Vec::<String>::new());
    let (new_check, set_new_check) = signal(String::new());

    let fetch_rules = move || {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/branch-protection");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<Vec<BranchProtectionRule>>().await {
                        set_rules.set(data);
                    }
                }
                Ok(_) => {
                    set_rules.set(Vec::new());
                }
                Err(_) => {
                    set_error.set(Some("Network error.".to_string()));
                }
            }
            set_loading.set(false);
        });
    };

    fetch_rules();

    let reset_form = Callback::new(move |_: ()| {
        set_req_pr.set(true);
        set_num_reviews.set(1);
        set_enforce_admins.set(false);
        set_allow_force.set(false);
        set_allow_delete.set(false);
        set_status_checks.set(Vec::new());
        set_new_check.set(String::new());
        set_editing_id.set(None);
    });

    let open_create = Callback::new(move |_: ()| {
        reset_form.run(());
        set_show_form.set(true);
    });

    let close_form = Callback::new(move |_: ()| {
        set_show_form.set(false);
        reset_form.run(());
    });

    let add_check = Callback::new(move |_: ()| {
        let val = new_check.get_untracked();
        if !val.trim().is_empty() {
            let mut checks = status_checks.get_untracked();
            if !checks.contains(&val.trim().to_string()) {
                checks.push(val.trim().to_string());
                set_status_checks.set(checks);
            }
            set_new_check.set(String::new());
        }
    });

    let save_rule = move |_: leptos::ev::MouseEvent| {
        let branch_pattern = get_input_value("bp-branch-pattern");
        if branch_pattern.trim().is_empty() {
            set_error.set(Some("Branch pattern is required.".to_string()));
            return;
        }

        let body = CreateProtectionBody {
            branch_pattern: branch_pattern.trim().to_string(),
            require_pull_request: req_pr.get(),
            required_approving_reviews: num_reviews.get(),
            enforce_admins: enforce_admins.get(),
            allow_force_pushes: allow_force.get(),
            allow_deletions: allow_delete.get(),
            required_status_checks: status_checks.get(),
        };

        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        let is_edit = editing_id.get();

        set_saving.set(true);
        set_error.set(None);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = if let Some(ref id) = is_edit {
                format!("/repos/{owner_val}/{name_val}/branch-protection/{id}")
            } else {
                format!("/repos/{owner_val}/{name_val}/branch-protection")
            };
            let result = if is_edit.is_some() {
                client.patch(&path, &body).await
            } else {
                client.post(&path, &body).await
            };
            match result {
                Ok(resp) if resp.status().is_success() => {
                    set_show_form.set(false);
                    reset_form.run(());
                    fetch_rules();
                }
                Ok(_) => {
                    set_error.set(Some("Failed to save branch protection rule.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error.".to_string()));
                }
            }
            set_saving.set(false);
        });
    };

    let delete_rule = move |rule_id: String| {
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/branch-protection/{rule_id}");
            let _ = client.delete(&path).await;
            fetch_rules();
        });
    };

    let dismiss_error = Callback::new(move |_: ()| set_error.set(None));

    let owner_v = owner();
    let name_v = name();

    view! {
        <div class="space-y-6">
            <div class="flex items-start justify-between">
                <div>
                    <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1">
                        <span class="text-gray-700 dark:text-gray-300 font-mono">{format!("{owner_v}/{name_v}")}</span>
                    </div>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Branch Protection"</h1>
                    <p class="mt-1 text-gray-600 dark:text-gray-400">"Configure rules to protect branches from unintended changes."</p>
                </div>
                <button
                    class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors bg-blue-600 hover:bg-blue-700 text-white"
                    on:click=move |_| open_create.run(())
                >
                    "Add Rule"
                </button>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=dismiss_error />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center justify-center py-12">
                    <Spinner />
                </div>
            </Show>

            <Show when=move || !loading.get() && rules.get().is_empty() && error.get().is_none() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="py-12 text-center">
                        <p class="text-gray-500 dark:text-gray-400 text-lg">"No branch protection rules."</p>
                        <p class="text-sm text-gray-400 dark:text-gray-500 mt-1">"Add a rule to protect branches from force pushes, deletions, and require reviews."</p>
                    </div>
                </Card>
            </Show>

            <Show when=move || !loading.get() && !rules.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                <div class="space-y-4">
                    <For each=move || rules.get() key=|r| r.id.clone() let:rule>
                        {
                            let del_id = StoredValue::new(rule.id.clone());
                            let status_checks = StoredValue::new(rule.required_status_checks.clone());
                            view! {
                                <Card>
                                    <div class="flex items-start justify-between">
                                        <div class="space-y-2">
                                            <div class="flex items-center gap-3">
                                                <span class="font-mono text-sm font-semibold text-gray-900 dark:text-gray-100">
                                                    {rule.branch_pattern.clone()}
                                                </span>
                                            </div>
                                            <div class="flex flex-wrap gap-2 text-xs">
                                                 {move || {
                                                    status_checks.get_value().iter().map(|c| {
                                                        view! {
                                                            <span class="inline-flex items-center px-2 py-0.5 rounded-full font-medium bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200">
                                                                {c.clone()}
                                                            </span>
                                                        }
                                                    }).collect_view()
                                                }}
                                            </div>
                                            <div class="flex flex-wrap gap-x-4 gap-y-1 text-xs text-gray-500 dark:text-gray-400">
                                                {if rule.require_pull_request { Some("Require PR") } else { None }}
                                                {if rule.required_approving_reviews > 0 { Some(format!("{} reviews", rule.required_approving_reviews)) } else { None }}
                                                {if rule.enforce_admins { Some("Enforce admins") } else { None }}
                                                {if rule.allow_force_pushes { Some("Force push allowed") } else { None }}
                                                {if rule.allow_deletions { Some("Deletion allowed") } else { None }}
                                            </div>
                                        </div>
                                        <div class="flex items-center gap-2">
                                            <button
                                                class="text-xs text-red-600 dark:text-red-400 hover:text-red-800 dark:hover:text-red-200 font-medium"
                                                on:click=move |_| delete_rule(del_id.get_value())
                                            >
                                                "Delete"
                                            </button>
                                        </div>
                                    </div>
                                </Card>
                            }
                        }
                    </For>
                </div>
            </Show>

            <Show when=move || show_form.get() fallback=|| view! { <div class="hidden"></div> }>
                <Card title={if editing_id.get().is_some() { "Edit Branch Protection Rule".to_string() } else { "New Branch Protection Rule".to_string() }} description="Configure protection settings for a branch pattern".to_string()>
                    <div class="space-y-5">
                        <Input label="Branch Pattern (e.g. main, release/*, **)" name="bp-branch-pattern" id="bp-branch-pattern" input_type=InputType::Text placeholder="main" required=true />

                        <label class="flex items-center gap-3 cursor-pointer">
                            <input
                                type="checkbox"
                                checked=move || req_pr.get()
                                on:change=move |ev| {
                                    let checked = event_target_checked(&ev);
                                    set_req_pr.set(checked);
                                }
                                class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                            />
                            <div>
                                <span class="text-sm font-medium text-gray-900 dark:text-gray-100">"Require pull request before merging"</span>
                            </div>
                        </label>

                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Required approving reviews"</label>
                            <input
                                type="number"
                                min="0"
                                max="10"
                                prop:value=move || num_reviews.get().to_string()
                                on:change=move |ev| {
                                    if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                        set_num_reviews.set(v);
                                    }
                                }
                                class="w-24 px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                            />
                        </div>

                        <label class="flex items-center gap-3 cursor-pointer">
                            <input
                                type="checkbox"
                                checked=move || enforce_admins.get()
                                on:change=move |ev| {
                                    let checked = event_target_checked(&ev);
                                    set_enforce_admins.set(checked);
                                }
                                class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                            />
                            <span class="text-sm font-medium text-gray-900 dark:text-gray-100">"Enforce rules for administrators"</span>
                        </label>

                        <label class="flex items-center gap-3 cursor-pointer">
                            <input
                                type="checkbox"
                                checked=move || allow_force.get()
                                on:change=move |ev| {
                                    let checked = event_target_checked(&ev);
                                    set_allow_force.set(checked);
                                }
                                class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                            />
                            <span class="text-sm font-medium text-gray-900 dark:text-gray-100">"Allow force pushes"</span>
                        </label>

                        <label class="flex items-center gap-3 cursor-pointer">
                            <input
                                type="checkbox"
                                checked=move || allow_delete.get()
                                on:change=move |ev| {
                                    let checked = event_target_checked(&ev);
                                    set_allow_delete.set(checked);
                                }
                                class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                            />
                            <span class="text-sm font-medium text-gray-900 dark:text-gray-100">"Allow deletions"</span>
                        </label>

                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Required status checks"</label>
                            <div class="flex gap-2 mb-2">
                                <input
                                    type="text"
                                    prop:value=move || new_check.get()
                                    on:input=move |ev| set_new_check.set(event_target_value(&ev))
                                    placeholder="Check name (e.g. ci/build)"
                                    class="flex-1 px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                                    on:keydown=move |ev| {
                                        if ev.key() == "Enter" {
                                            ev.prevent_default();
                                            add_check.run(());
                                        }
                                    }
                                />
                                <button
                                    class="px-3 py-2 text-sm font-medium bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 rounded-md"
                                    on:click=move |_| add_check.run(())
                                >
                                    "Add"
                                </button>
                            </div>
                            <div class="flex flex-wrap gap-2">
                                <For each=move || status_checks.get() key=|c| c.clone() let:check>
                                    {
                                        let check_val = check.clone();
                                        view! {
                                            <span class="inline-flex items-center gap-1 px-2 py-1 rounded-md bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 text-xs font-mono">
                                                {check.clone()}
                                                <button
                                                    class="text-blue-600 dark:text-blue-300 hover:text-blue-800 dark:hover:text-blue-100"
                                                    on:click=move |_| {
                                                        let current = status_checks.get_untracked();
                                                        let filtered: Vec<String> = current.into_iter().filter(|c| c != &check_val).collect();
                                                        set_status_checks.set(filtered);
                                                    }
                                                >
                                                    "\u{00d7}"
                                                </button>
                                            </span>
                                        }
                                    }
                                </For>
                            </div>
                        </div>

                        <div class="flex gap-3 pt-2">
                            <Button variant=ButtonVariant::Primary disabled=saving.get() on:click=save_rule>
                                {move || if saving.get() { "Saving..." } else { "Save Rule" }}
                            </Button>
                            <button
                                class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors bg-gray-200 hover:bg-gray-300 text-gray-900 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-100"
                                on:click=move |_| close_form.run(())
                            >
                                "Cancel"
                            </button>
                        </div>
                    </div>
                </Card>
            </Show>
        </div>
    }
}
