#![forbid(unsafe_code)]

use leptos::prelude::*;
use crate::api::client::ApiClient;
use crate::components::{Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Spinner};
use crate::state::auth::use_auth;

#[derive(Debug, Clone, serde::Deserialize)]
struct AdminFeatureFlag {
    id: String,
    name: String,
    description: String,
    enabled: bool,
    enabled_for_users: Vec<String>,
    enabled_for_percentage: i32,
    enabled_for_orgs: Vec<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct AdminFeatureFlagsResponse {
    flags: Vec<AdminFeatureFlag>,
    total: usize,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct AdminWidget {
    id: String,
    widget_name: String,
    widget_config: serde_json::Value,
    position: i32,
    enabled: bool,
    created_at: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct AdminDashboardResponse {
    widgets: Vec<AdminWidget>,
    total: usize,
}

#[component]
pub fn AdminFeatureFlagsPage() -> impl IntoView {
    let auth = use_auth();
    let (flags, set_flags) = signal(Vec::<AdminFeatureFlag>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    let load_flags = Callback::new(move |_| {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|s| s.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            match client.get("/admin/feature-flags").await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<AdminFeatureFlagsResponse>().await {
                        Ok(data) => {
                            set_flags.set(data.flags);
                            set_loading.set(false);
                        }
                        Err(e) => {
                            set_error.set(Some(format!("Failed to parse response: {e}")));
                            set_loading.set(false);
                        }
                    }
                }
                Ok(resp) => {
                    set_error.set(Some(format!("Error: {}", resp.status())));
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(format!("Request failed: {e}")));
                    set_loading.set(false);
                }
            }
        });
    });

    load_flags.run(());

    let toggle_flag = Callback::new(move |flag_id: String| {
        let token = auth.0.with(|s| s.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let _ = client
                .post(
                    &format!("/admin/feature-flags/{flag_id}/toggle"),
                    &serde_json::json!({}),
                )
                .await;
            load_flags.run(());
        });
    });

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold">"Feature Flags"</h1>
            </div>

            <Show when=move || !loading.get() fallback=|| view! { <Spinner /> }>
                <Show when=move || error.get().is_none() fallback=move || {
                    let err_msg = error.get().unwrap_or_default();
                    view! { <ErrorBanner message=move || err_msg.clone() /> }
                }>
                    <Card>
                        <div class="overflow-x-auto">
                            <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                                <thead class="bg-gray-50 dark:bg-gray-800">
                                    <tr>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Name"</th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Description"</th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Status"</th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Rollout"</th>
                                        <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Actions"</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
                                    <For each=move || flags.get() key=|f| f.id.clone() let:flag>
                                        {
                                            let flag_id = flag.id.clone();
                                            let flag_name = flag.name.clone();
                                            let flag_description = flag.description.clone();
                                            let flag_enabled = flag.enabled;
                                            let flag_percentage = flag.enabled_for_percentage;
                                            let flag_user_count = flag.enabled_for_users.len();
                                            let flag_org_count = flag.enabled_for_orgs.len();
                                            let rollout_text = format!("{}% | Users: {} | Orgs: {}", flag_percentage, flag_user_count, flag_org_count);
                                            let status_text = if flag_enabled { "Enabled".to_string() } else { "Disabled".to_string() };
                                            view! {
                                                <tr class="hover:bg-gray-50 dark:hover:bg-gray-750">
                                                    <td class="px-4 py-3 text-sm font-medium text-gray-900 dark:text-gray-100">{flag_name}</td>
                                                    <td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-400">{flag_description}</td>
                                                    <td class="px-4 py-3">
                                                        <Badge
                                                            color=if flag_enabled { BadgeColor::Success } else { BadgeColor::Neutral }
                                                            text=status_text
                                                        />
                                                    </td>
                                                    <td class="px-4 py-3 text-xs text-gray-400 dark:text-gray-500">
                                                        {rollout_text}
                                                    </td>
                                                    <td class="px-4 py-3 text-right">
                                                        <Button
                                                            variant=ButtonVariant::Secondary
                                                            on:click=move |_| toggle_flag.run(flag_id.clone())
                                                        >
                                                            "Toggle"
                                                        </Button>
                                                    </td>
                                                </tr>
                                            }
                                        }
                                    </For>
                                </tbody>
                            </table>
                        </div>
                    </Card>
                </Show>
            </Show>
        </div>
    }
}

#[component]
pub fn AdminDashboardPage() -> impl IntoView {
    let auth = use_auth();
    let (widgets, set_widgets) = signal(Vec::<AdminWidget>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    let load_widgets = Callback::new(move |_| {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|s| s.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            match client.get("/admin/dashboard").await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<AdminDashboardResponse>().await {
                        Ok(data) => {
                            set_widgets.set(data.widgets);
                            set_loading.set(false);
                        }
                        Err(e) => {
                            set_error.set(Some(format!("Failed to parse response: {e}")));
                            set_loading.set(false);
                        }
                    }
                }
                Ok(resp) => {
                    set_error.set(Some(format!("Error: {}", resp.status())));
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(format!("Request failed: {e}")));
                    set_loading.set(false);
                }
            }
        });
    });

    load_widgets.run(());

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold">"Admin Dashboard"</h1>
            </div>

            <Show when=move || !loading.get() fallback=|| view! { <Spinner /> }>
                <Show when=move || error.get().is_none() fallback=move || {
                    let err_msg = error.get().unwrap_or_default();
                    view! { <ErrorBanner message=move || err_msg.clone() /> }
                }>
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        <For each=move || widgets.get() key=|w| w.id.clone() let:widget>
                            {
                                let widget_name = widget.widget_name.clone();
                                let widget_enabled = widget.enabled;
                                let widget_position = widget.position;
                                let config_text = if widget.widget_config == serde_json::json!({}) {
                                    "No configuration".to_string()
                                } else {
                                    serde_json::to_string_pretty(&widget.widget_config).unwrap_or_default()
                                };
                                let status_text = if widget_enabled { "Active".to_string() } else { "Disabled".to_string() };
                                let pos_text = widget_position.to_string();
                                view! {
                                    <Card>
                                        <div class="space-y-2">
                                            <div class="flex items-center justify-between">
                                                <h3 class="font-semibold">{move || widget_name.clone()}</h3>
                                                <Badge
                                                    color=if widget_enabled { BadgeColor::Success } else { BadgeColor::Neutral }
                                                    text=status_text
                                                />
                                            </div>
                                            <div class="text-xs text-gray-400">
                                                "Position: " {pos_text}
                                            </div>
                                            <div class="text-sm text-gray-600">
                                                {config_text}
                                            </div>
                                        </div>
                                    </Card>
                                }
                            }
                        </For>
                    </div>
                </Show>
            </Show>
        </div>
    }
}
