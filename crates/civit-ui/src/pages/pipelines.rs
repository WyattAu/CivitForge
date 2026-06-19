#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::components::{
    Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Input, InputType, Modal, Spinner,
};
use crate::state::auth::use_auth;
use crate::utils::*;

#[derive(Clone, serde::Deserialize)]
pub struct PipelineRunResponse {
    pub id: String,
    #[allow(dead_code)]
    pub repo_id: String,
    pub trigger: String,
    pub ref_name: Option<String>,
    pub commit_sha: String,
    pub status: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Clone, serde::Deserialize)]
pub struct PipelineSchedule {
    pub id: String,
    pub cron: String,
    pub branch: String,
    pub enabled: bool,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
    pub created_at: String,
}

#[derive(Clone, serde::Deserialize)]
pub struct PipelineSecret {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Clone, serde::Deserialize)]
pub struct PipelineCache {
    pub id: String,
    pub key: String,
    pub size_bytes: u64,
    pub expires_at: Option<String>,
    pub created_at: String,
}

#[derive(Clone, serde::Deserialize)]
pub struct PipelineVariable {
    pub id: String,
    pub key: String,
    pub value: String,
    pub masked: bool,
    pub protected: bool,
}

#[derive(Clone, serde::Serialize)]
struct CreateScheduleBody {
    cron: String,
    branch: String,
    enabled: bool,
}

#[derive(Clone, serde::Serialize)]
struct CreateSecretBody {
    name: String,
    value: String,
}

#[derive(Clone, serde::Serialize)]
struct CreateVariableBody {
    key: String,
    value: String,
    masked: bool,
    protected: bool,
}

#[derive(Clone, PartialEq)]
enum PipelineTab {
    Runs,
    Schedules,
    Secrets,
    Caches,
    Variables,
}

fn pipeline_status_color(status: &str) -> BadgeColor {
    match status {
        "success" | "completed" => BadgeColor::Success,
        "failed" | "failure" => BadgeColor::Danger,
        "running" | "in_progress" => BadgeColor::Warning,
        "pending" | "queued" => BadgeColor::Neutral,
        "canceled" | "cancelled" => BadgeColor::Danger,
        _ => BadgeColor::Neutral,
    }
}

fn pipeline_status_label(status: &str) -> String {
    match status {
        "in_progress" => "Running".to_string(),
        s => {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        }
    }
}

fn truncate_sha(sha: &str) -> String {
    if sha.len() > 7 {
        sha[..7].to_string()
    } else {
        sha.to_string()
    }
}

fn format_pipeline_duration(started: Option<&str>, finished: Option<&str>) -> String {
    let start = match started.and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()) {
        Some(dt) => dt.with_timezone(&chrono::Utc),
        None => return "-".to_string(),
    };
    let end = match finished.and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()) {
        Some(dt) => dt.with_timezone(&chrono::Utc),
        None => return "-".to_string(),
    };
    let diff = end.signed_duration_since(start);
    if diff.num_seconds() < 60 {
        format!("{}s", diff.num_seconds())
    } else if diff.num_minutes() < 60 {
        format!("{}m {}s", diff.num_minutes(), diff.num_seconds() % 60)
    } else {
        format!("{}h {}m", diff.num_hours(), diff.num_minutes() % 60)
    }
}

#[component]
pub fn PipelinesPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let auth = use_auth();

    let (active_tab, set_active_tab) = signal(PipelineTab::Runs);

    let (pipelines_sig, set_pipelines) = signal(vec![]);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let (schedules, set_schedules) = signal(Vec::<PipelineSchedule>::new());
    let (schedules_loading, set_schedules_loading) = signal(false);
    let (show_schedule_form, set_show_schedule_form) = signal(false);
    let (schedule_saving, set_schedule_saving) = signal(false);

    let (secrets, set_secrets) = signal(Vec::<PipelineSecret>::new());
    let (secrets_loading, set_secrets_loading) = signal(false);
    let (show_secret_form, set_show_secret_form) = signal(false);
    let (secret_saving, set_secret_saving) = signal(false);
    let (revealed_secrets, set_revealed_secrets) = signal(Vec::<String>::new());

    let (caches, set_caches) = signal(Vec::<PipelineCache>::new());
    let (caches_loading, set_caches_loading) = signal(false);

    let (variables, set_variables) = signal(Vec::<PipelineVariable>::new());
    let (variables_loading, set_variables_loading) = signal(false);
    let (show_var_form, set_show_var_form) = signal(false);
    let (var_saving, set_var_saving) = signal(false);

    let fetch_pipelines = move || {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/pipelines?limit=50&offset=0");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<Vec<PipelineRunResponse>>().await {
                        Ok(data) => set_pipelines.set(data),
                        Err(_) => set_error.set(Some("Failed to parse pipeline data.".to_string())),
                    }
                }
                Ok(_) => {
                    set_error.set(Some("Failed to load pipelines.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_loading.set(false);
        });
    };

    let fetch_schedules = move || {
        set_schedules_loading.set(true);
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/pipeline-schedules");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<Vec<PipelineSchedule>>().await {
                        set_schedules.set(data);
                    }
                }
                _ => {}
            }
            set_schedules_loading.set(false);
        });
    };

    let fetch_secrets = move || {
        set_secrets_loading.set(true);
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/pipeline-secrets");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<Vec<PipelineSecret>>().await {
                        set_secrets.set(data);
                    }
                }
                _ => {}
            }
            set_secrets_loading.set(false);
        });
    };

    let fetch_caches = move || {
        set_caches_loading.set(true);
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/pipeline-caches");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<Vec<PipelineCache>>().await {
                        set_caches.set(data);
                    }
                }
                _ => {}
            }
            set_caches_loading.set(false);
        });
    };

    let fetch_variables = move || {
        set_variables_loading.set(true);
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/pipeline-variables");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<Vec<PipelineVariable>>().await {
                        set_variables.set(data);
                    }
                }
                _ => {}
            }
            set_variables_loading.set(false);
        });
    };

    fetch_pipelines();

    let switch_tab = move |tab: PipelineTab| {
        set_active_tab.set(tab.clone());
        match tab {
            PipelineTab::Runs => fetch_pipelines(),
            PipelineTab::Schedules => fetch_schedules(),
            PipelineTab::Secrets => fetch_secrets(),
            PipelineTab::Caches => fetch_caches(),
            PipelineTab::Variables => fetch_variables(),
        }
    };

    let dismiss_error = Callback::new(move |_| set_error.set(None));

    let owner_v = owner();
    let name_v = name();

    let tab_class = |active: bool| {
        if active {
            "px-4 py-2 text-sm font-medium border-b-2 border-blue-600 dark:border-blue-400 text-blue-600 dark:text-blue-400"
        } else {
            "px-4 py-2 text-sm font-medium border-b-2 border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
        }
    };

    view! {
        <div class="space-y-6">
            <div>
                <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1 overflow-x-auto">
                    <A href="/repos"><span class="hover:text-blue-600 dark:hover:text-blue-400 shrink-0">"Repositories"</span></A>
                    <span class="shrink-0">"/"</span>
                    <A href=format!("/repos/{owner_v}/{name_v}")><span class="hover:text-blue-600 dark:hover:text-blue-400 shrink-0">{format!("{owner_v}/{name_v}")}</span></A>
                </div>
                <h1 class="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-gray-100">"Pipelines"</h1>
                <p class="mt-1 text-gray-600 dark:text-gray-400">"CI/CD pipeline configuration and runs."</p>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=dismiss_error />
            </Show>

            <div class="border-b border-gray-200 dark:border-gray-700 overflow-x-auto">
                <div class="flex gap-1 -mb-px min-w-max">
                    <button class=move || tab_class(active_tab.get() == PipelineTab::Runs) on:click=move |_| switch_tab(PipelineTab::Runs)>"Runs"</button>
                    <button class=move || tab_class(active_tab.get() == PipelineTab::Schedules) on:click=move |_| switch_tab(PipelineTab::Schedules)>"Schedules"</button>
                    <button class=move || tab_class(active_tab.get() == PipelineTab::Secrets) on:click=move |_| switch_tab(PipelineTab::Secrets)>"Secrets"</button>
                    <button class=move || tab_class(active_tab.get() == PipelineTab::Caches) on:click=move |_| switch_tab(PipelineTab::Caches)>"Caches"</button>
                    <button class=move || tab_class(active_tab.get() == PipelineTab::Variables) on:click=move |_| switch_tab(PipelineTab::Variables)>"Variables"</button>
                </div>
            </div>

            // ── Runs Tab ──
            <Show when=move || active_tab.get() == PipelineTab::Runs fallback=|| view! { <div class="hidden"></div> }>
                <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                    <div class="flex items-center justify-center py-12"><Spinner /></div>
                </Show>
                <Show when=move || !loading.get() && pipelines_sig.with(|p| p.is_empty()) && error.get().is_none() fallback=|| view! { <div class="hidden"></div> }>
                    <Card>
                        <div class="py-12 text-center">
                            <p class="text-gray-500 dark:text-gray-400 text-lg">"No pipeline runs yet."</p>
                            <p class="text-sm text-gray-400 dark:text-gray-500 mt-1">"Push a commit to trigger your first pipeline."</p>
                        </div>
                    </Card>
                </Show>
                <Show when=move || !loading.get() && !pipelines_sig.with(|p| p.is_empty()) fallback=|| view! { <div class="hidden"></div> }>
                    <Card>
                        <div class="divide-y divide-gray-100 dark:divide-gray-700">
                            <For each=move || pipelines_sig.get() key=|p| p.id.clone() let:pipeline>
                                {
                                    let status_color = pipeline_status_color(&pipeline.status);
                                    let status_text = pipeline_status_label(&pipeline.status);
                                    let duration = format_pipeline_duration(pipeline.started_at.as_deref(), pipeline.finished_at.as_deref());
                                    let time_str = relative_time(&pipeline.created_at);
                                    let ref_name = pipeline.ref_name.clone().unwrap_or_default();
                                    let commit_short = truncate_sha(&pipeline.commit_sha);
                                    let owner_link = owner();
                                    let name_link = name();
                                    view! {
                                        <A href=format!("/repos/{owner_link}/{name_link}/pipelines/{}", pipeline.id)>
                                            <div class="flex items-center gap-2 sm:gap-4 py-3 px-2 hover:bg-gray-50 dark:hover:bg-gray-750 -mx-1 rounded transition-colors cursor-pointer">
                                                <Badge color=status_color text=status_text />
                                                <span class="font-mono text-sm text-blue-600 dark:text-blue-400 shrink-0 hidden sm:inline">{commit_short}</span>
                                                <span class="text-sm text-gray-700 dark:text-gray-300 truncate flex-1">{pipeline.trigger.clone()}</span>
                                                <span class="text-xs text-gray-500 dark:text-gray-400 font-mono shrink-0 hidden md:inline">{ref_name}</span>
                                                <span class="text-xs text-gray-400 dark:text-gray-500 shrink-0 w-16 text-right hidden sm:inline">{duration}</span>
                                                <span class="text-xs text-gray-400 dark:text-gray-500 shrink-0 w-20 text-right">{time_str}</span>
                                            </div>
                                        </A>
                                    }
                                }
                            </For>
                        </div>
                    </Card>
                </Show>
            </Show>

            // ── Schedules Tab ──
            <Show when=move || active_tab.get() == PipelineTab::Schedules fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center justify-between">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">"Pipeline Schedules"</h2>
                    <button class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors bg-blue-600 hover:bg-blue-700 text-white" on:click=move |_| set_show_schedule_form.set(true)>"New Schedule"</button>
                </div>
                <Show when=move || schedules_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                    <div class="flex items-center justify-center py-12"><Spinner /></div>
                </Show>
                <Show when=move || !schedules_loading.get() && schedules.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                    <Card><div class="py-12 text-center text-gray-400 dark:text-gray-500">"No schedules configured."</div></Card>
                </Show>
                <Show when=move || !schedules_loading.get() && !schedules.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                    <Card>
                        <div class="divide-y divide-gray-100 dark:divide-gray-700">
                            <For each=move || schedules.get() key=|s| s.id.clone() let:sched>
                                <ScheduleRow sched=sched owner=owner() repo_name=name() auth=auth on_refresh=fetch_schedules />
                            </For>
                        </div>
                    </Card>
                </Show>
                <Modal show=show_schedule_form.get() title="Create Schedule".to_string() on_close=Callback::new(move |_: ()| set_show_schedule_form.set(false))>
                    <div class="space-y-4">
                        <Input label="Cron Expression" name="schedule-cron" id="schedule-cron" input_type=InputType::Text placeholder="0 * * * *" required=true />
                        <Input label="Branch" name="schedule-branch" id="schedule-branch" input_type=InputType::Text placeholder="main" required=true />
                        <div class="flex gap-3 pt-2">
                            <Button variant=ButtonVariant::Primary disabled=schedule_saving.get() on:click=move |_| {
                                let cron_val = get_input_value("schedule-cron");
                                let branch_val = get_input_value("schedule-branch");
                                if cron_val.trim().is_empty() || branch_val.trim().is_empty() { return; }
                                let body = CreateScheduleBody { cron: cron_val, branch: branch_val, enabled: true };
                                let token = auth.0.with(|a| a.token.clone());
                                let owner_val = owner();
                                let name_val = name();
                                set_schedule_saving.set(true);
                                leptos::task::spawn_local(async move {
                                    let client = ApiClient::new(token);
                                    let _ = client.post(&format!("/repos/{owner_val}/{name_val}/pipeline-schedules"), &body).await;
                                    set_show_schedule_form.set(false);
                                    set_schedule_saving.set(false);
                                    fetch_schedules();
                                });
                            }>{move || if schedule_saving.get() { "Creating..." } else { "Create" }}</Button>
                            <button class="px-4 py-2 rounded-md text-sm font-medium bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600" on:click=move |_| set_show_schedule_form.set(false)>"Cancel"</button>
                        </div>
                    </div>
                </Modal>
            </Show>

            // ── Secrets Tab ──
            <Show when=move || active_tab.get() == PipelineTab::Secrets fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center justify-between">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">"Pipeline Secrets"</h2>
                    <button class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors bg-blue-600 hover:bg-blue-700 text-white" on:click=move |_| set_show_secret_form.set(true)>"Add Secret"</button>
                </div>
                <Show when=move || secrets_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                    <div class="flex items-center justify-center py-12"><Spinner /></div>
                </Show>
                <Show when=move || !secrets_loading.get() && secrets.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                    <Card><div class="py-12 text-center text-gray-400 dark:text-gray-500">"No secrets configured."</div></Card>
                </Show>
                <Show when=move || !secrets_loading.get() && !secrets.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                    <Card>
                        <div class="divide-y divide-gray-100 dark:divide-gray-700">
                            <For each=move || secrets.get() key=|s| s.id.clone() let:sec>
                                <SecretRow sec=sec owner=owner() repo_name=name() auth=auth on_refresh=fetch_secrets revealed=revealed_secrets set_revealed=set_revealed_secrets />
                            </For>
                        </div>
                    </Card>
                </Show>
                <Modal show=show_secret_form.get() title="Add Secret".to_string() on_close=Callback::new(move |_: ()| set_show_secret_form.set(false))>
                    <div class="space-y-4">
                        <Input label="Secret Name" name="secret-name" id="secret-name" input_type=InputType::Text placeholder="MY_SECRET_TOKEN" required=true />
                        <Input label="Secret Value" name="secret-value" id="secret-value" input_type=InputType::Password placeholder="Enter secret value" required=true />
                        <div class="flex gap-3 pt-2">
                            <Button variant=ButtonVariant::Primary disabled=secret_saving.get() on:click=move |_| {
                                let name_val = get_input_value("secret-name");
                                let value_val = get_input_value("secret-value");
                                if name_val.trim().is_empty() || value_val.trim().is_empty() { return; }
                                let body = CreateSecretBody { name: name_val, value: value_val };
                                let token = auth.0.with(|a| a.token.clone());
                                let owner_val = owner();
                                let name_val = name();
                                set_secret_saving.set(true);
                                leptos::task::spawn_local(async move {
                                    let client = ApiClient::new(token);
                                    let _ = client.post(&format!("/repos/{owner_val}/{name_val}/pipeline-secrets"), &body).await;
                                    set_show_secret_form.set(false);
                                    set_secret_saving.set(false);
                                    fetch_secrets();
                                });
                            }>{move || if secret_saving.get() { "Adding..." } else { "Add Secret" }}</Button>
                            <button class="px-4 py-2 rounded-md text-sm font-medium bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600" on:click=move |_| set_show_secret_form.set(false)>"Cancel"</button>
                        </div>
                    </div>
                </Modal>
            </Show>

            // ── Caches Tab ──
            <Show when=move || active_tab.get() == PipelineTab::Caches fallback=|| view! { <div class="hidden"></div> }>
                <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">"Pipeline Caches"</h2>
                <Show when=move || caches_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                    <div class="flex items-center justify-center py-12"><Spinner /></div>
                </Show>
                <Show when=move || !caches_loading.get() && caches.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                    <Card><div class="py-12 text-center text-gray-400 dark:text-gray-500">"No caches found."</div></Card>
                </Show>
                <Show when=move || !caches_loading.get() && !caches.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                    <Card>
                        <div class="divide-y divide-gray-100 dark:divide-gray-700">
                            <For each=move || caches.get() key=|c| c.id.clone() let:cache>
                                <CacheRow cache=cache owner=owner() repo_name=name() auth=auth on_refresh=fetch_caches />
                            </For>
                        </div>
                    </Card>
                </Show>
            </Show>

            // ── Variables Tab ──
            <Show when=move || active_tab.get() == PipelineTab::Variables fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center justify-between">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">"Pipeline Variables"</h2>
                    <button class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors bg-blue-600 hover:bg-blue-700 text-white" on:click=move |_| set_show_var_form.set(true)>"Add Variable"</button>
                </div>
                <Show when=move || variables_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                    <div class="flex items-center justify-center py-12"><Spinner /></div>
                </Show>
                <Show when=move || !variables_loading.get() && variables.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                    <Card><div class="py-12 text-center text-gray-400 dark:text-gray-500">"No variables configured."</div></Card>
                </Show>
                <Show when=move || !variables_loading.get() && !variables.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                    <Card>
                        <div class="divide-y divide-gray-100 dark:divide-gray-700">
                            <For each=move || variables.get() key=|v| v.id.clone() let:var>
                                <VariableRow var=var owner=owner() repo_name=name() auth=auth on_refresh=fetch_variables />
                            </For>
                        </div>
                    </Card>
                </Show>
                <Modal show=show_var_form.get() title="Add Variable".to_string() on_close=Callback::new(move |_: ()| set_show_var_form.set(false))>
                    <div class="space-y-4">
                        <Input label="Key" name="var-key" id="var-key" input_type=InputType::Text placeholder="MY_VAR" required=true />
                        <Input label="Value" name="var-value" id="var-value" input_type=InputType::Text placeholder="Enter value" required=true />
                        <label class="flex items-center gap-2 cursor-pointer">
                            <input type="checkbox" id="var-masked" class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500" />
                            <span class="text-sm font-medium text-gray-700 dark:text-gray-300">"Mask value in logs"</span>
                        </label>
                        <label class="flex items-center gap-2 cursor-pointer">
                            <input type="checkbox" id="var-protected" class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500" />
                            <span class="text-sm font-medium text-gray-700 dark:text-gray-300">"Protect from forked pipelines"</span>
                        </label>
                        <div class="flex gap-3 pt-2">
                            <Button variant=ButtonVariant::Primary disabled=var_saving.get() on:click=move |_| {
                                let key_val = get_input_value("var-key");
                                let value_val = get_input_value("var-value");
                                let masked = get_input_value("var-masked");
                                let protected = get_input_value("var-protected");
                                if key_val.trim().is_empty() || value_val.trim().is_empty() { return; }
                                let body = CreateVariableBody {
                                    key: key_val,
                                    value: value_val,
                                    masked: masked == "on",
                                    protected: protected == "on",
                                };
                                let token = auth.0.with(|a| a.token.clone());
                                let owner_val = owner();
                                let name_val = name();
                                set_var_saving.set(true);
                                leptos::task::spawn_local(async move {
                                    let client = ApiClient::new(token);
                                    let _ = client.post(&format!("/repos/{owner_val}/{name_val}/pipeline-variables"), &body).await;
                                    set_show_var_form.set(false);
                                    set_var_saving.set(false);
                                    fetch_variables();
                                });
                            }>{move || if var_saving.get() { "Adding..." } else { "Add Variable" }}</Button>
                            <button class="px-4 py-2 rounded-md text-sm font-medium bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600" on:click=move |_| set_show_var_form.set(false)>"Cancel"</button>
                        </div>
                    </div>
                </Modal>
            </Show>
        </div>
    }
}

#[component]
fn ScheduleRow(
    sched: PipelineSchedule,
    owner: String,
    repo_name: String,
    auth: (
        ReadSignal<crate::state::auth::AuthState>,
        WriteSignal<crate::state::auth::AuthState>,
    ),
    on_refresh: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let del_id = sched.id.clone();
    let sched_c = sched.clone();
    view! {
        <div class="flex items-center justify-between py-3">
            <div class="space-y-1">
                <div class="flex items-center gap-3">
                    <span class="font-mono text-sm text-gray-900 dark:text-gray-100">{sched_c.cron.clone()}</span>
                    <span class="font-mono text-xs text-gray-500 dark:text-gray-400">{sched_c.branch.clone()}</span>
                    <Badge color=if sched_c.enabled { BadgeColor::Success } else { BadgeColor::Neutral } text=if sched_c.enabled { "Enabled".to_string() } else { "Disabled".to_string() } />
                </div>
                <div class="text-xs text-gray-400 dark:text-gray-500">
                    {sched_c.last_run.as_deref().map(|l| format!("Last run: {}", relative_time(l))).unwrap_or_else(|| "Never run".to_string())}
                </div>
            </div>
            <button class="text-xs text-red-600 dark:text-red-400 hover:text-red-800 dark:hover:text-red-200 font-medium" on:click=move |_| {
                let token = auth.0.with(|a| a.token.clone());
                let id = del_id.clone();
                let o = owner.clone();
                let r = repo_name.clone();
                let cb = on_refresh.clone();
                leptos::task::spawn_local(async move {
                    let client = ApiClient::new(token);
                    let _ = client.delete(&format!("/repos/{o}/{r}/pipeline-schedules/{id}")).await;
                    cb();
                });
            }>"Delete"</button>
        </div>
    }
}

#[component]
fn SecretRow(
    sec: PipelineSecret,
    owner: String,
    repo_name: String,
    auth: (
        ReadSignal<crate::state::auth::AuthState>,
        WriteSignal<crate::state::auth::AuthState>,
    ),
    on_refresh: impl Fn() + Clone + 'static,
    revealed: ReadSignal<Vec<String>>,
    set_revealed: WriteSignal<Vec<String>>,
) -> impl IntoView {
    let sec_c = sec.clone();
    let name_clone = sec.name.clone();
    let name_clone_display = name_clone.clone();
    let del_id = sec.id.clone();
    view! {
        <div class="flex items-center justify-between py-3">
            <div class="flex items-center gap-3">
                <span class="font-mono text-sm font-medium text-gray-900 dark:text-gray-100">{sec_c.name.clone()}</span>
                <span class="text-xs text-gray-400 dark:text-gray-500">"Added {relative_time(&sec_c.created_at)}"</span>
            </div>
            <div class="flex items-center gap-2">
                <button class="text-xs text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-200 font-medium" on:click=move |_| {
                    let mut r = revealed.get_untracked();
                    if r.contains(&name_clone) {
                        r.retain(|x| x != &name_clone);
                    } else {
                        r.push(name_clone.clone());
                    }
                    set_revealed.set(r);
                }>{move || if revealed.get().contains(&name_clone_display) { "Hide" } else { "Reveal" }}</button>
                <button class="text-xs text-red-600 dark:text-red-400 hover:text-red-800 dark:hover:text-red-200 font-medium" on:click=move |_| {
                    let token = auth.0.with(|a| a.token.clone());
                    let id = del_id.clone();
                    let o = owner.clone();
                    let r = repo_name.clone();
                    let cb = on_refresh.clone();
                    leptos::task::spawn_local(async move {
                        let client = ApiClient::new(token);
                        let _ = client.delete(&format!("/repos/{o}/{r}/pipeline-secrets/{id}")).await;
                        cb();
                    });
                }>"Delete"</button>
            </div>
        </div>
    }
}

#[component]
fn CacheRow(
    cache: PipelineCache,
    owner: String,
    repo_name: String,
    auth: (
        ReadSignal<crate::state::auth::AuthState>,
        WriteSignal<crate::state::auth::AuthState>,
    ),
    on_refresh: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let cache_c = cache.clone();
    let del_id = cache.id.clone();
    view! {
        <div class="flex items-center justify-between py-3">
            <div class="space-y-1">
                <div class="flex items-center gap-3">
                    <span class="font-mono text-sm font-medium text-gray-900 dark:text-gray-100">{cache_c.key.clone()}</span>
                    <span class="text-xs text-gray-500 dark:text-gray-400">{format_bytes(cache_c.size_bytes)}</span>
                </div>
                <div class="text-xs text-gray-400 dark:text-gray-500">
                    {cache_c.expires_at.as_deref().map(|e| format!("Expires: {}", relative_time(e))).unwrap_or_else(|| "No expiry".to_string())}
                </div>
            </div>
            <button class="text-xs text-red-600 dark:text-red-400 hover:text-red-800 dark:hover:text-red-200 font-medium" on:click=move |_| {
                let token = auth.0.with(|a| a.token.clone());
                let id = del_id.clone();
                let o = owner.clone();
                let r = repo_name.clone();
                let cb = on_refresh.clone();
                leptos::task::spawn_local(async move {
                    let client = ApiClient::new(token);
                    let _ = client.delete(&format!("/repos/{o}/{r}/pipeline-caches/{id}")).await;
                    cb();
                });
            }>"Delete"</button>
        </div>
    }
}

#[component]
fn VariableRow(
    var: PipelineVariable,
    owner: String,
    repo_name: String,
    auth: (
        ReadSignal<crate::state::auth::AuthState>,
        WriteSignal<crate::state::auth::AuthState>,
    ),
    on_refresh: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let var_c = var.clone();
    let del_id = var.id.clone();
    view! {
        <div class="flex items-center justify-between py-3">
            <div class="flex items-center gap-3">
                <span class="font-mono text-sm font-medium text-gray-900 dark:text-gray-100">{var_c.key.clone()}</span>
                <span class="font-mono text-xs text-gray-500 dark:text-gray-400">
                    {if var_c.masked { "***".to_string() } else { var_c.value.clone() }}
                </span>
                {if var_c.masked { Some(view! { <Badge color=BadgeColor::Warning text="Masked".to_string() /> }) } else { None }}
                {if var_c.protected { Some(view! { <Badge color=BadgeColor::Info text="Protected".to_string() /> }) } else { None }}
            </div>
            <button class="text-xs text-red-600 dark:text-red-400 hover:text-red-800 dark:hover:text-red-200 font-medium" on:click=move |_| {
                let token = auth.0.with(|a| a.token.clone());
                let id = del_id.clone();
                let o = owner.clone();
                let r = repo_name.clone();
                let cb = on_refresh.clone();
                leptos::task::spawn_local(async move {
                    let client = ApiClient::new(token);
                    let _ = client.delete(&format!("/repos/{o}/{r}/pipeline-variables/{id}")).await;
                    cb();
                });
            }>"Delete"</button>
        </div>
    }
}
