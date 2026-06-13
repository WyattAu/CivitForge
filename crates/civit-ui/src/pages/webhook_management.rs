#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::components::{
    Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Input, InputType, Modal, Spinner,
};
use crate::state::auth::use_auth;
use crate::utils::*;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct WebhookItem {
    pub id: String,
    pub repo_id: String,
    pub url: String,
    pub events: Vec<String>,
    pub active: bool,
    pub created_at: String,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct DeliveryItem {
    pub id: String,
    pub event: String,
    pub status: String,
    pub attempts: i32,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub created_at: String,
}

const EVENT_OPTIONS: &[&str] = &[
    "push",
    "tag",
    "delete",
    "issue",
    "issue_comment",
    "pull_request",
    "pull_request_review",
    "wiki",
    "release",
    "fork",
    "member",
    "repository",
    "star",
    "watch",
    "pipeline",
    "deploy",
];

#[component]
pub fn WebhookManagementPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let auth = use_auth();

    let (webhooks, set_webhooks) = signal(Vec::<WebhookItem>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (show_create, set_show_create) = signal(false);
    let (editing, set_editing) = signal(None::<WebhookItem>);
    let (show_deliveries, set_show_deliveries) = signal(None::<WebhookItem>);
    let (deliveries, set_deliveries) = signal(Vec::<DeliveryItem>::new());
    let (delivery_loading, set_delivery_loading) = signal(false);
    let (testing_id, set_testing_id) = signal(None::<String>);

    let fetch_webhooks = move || {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        let o = owner();
        let n = name();
        leptos::task::spawn_local(async move {
            let path = format!("/repos/{o}/{n}/webhooks");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<Vec<WebhookItem>>().await {
                        set_webhooks.set(data);
                    }
                }
                Ok(_) => {
                    set_error.set(Some(sanitize_error("Failed to load webhooks.")));
                }
                Err(e) => {
                    set_error.set(Some(sanitize_error(&e.to_string())));
                }
            }
            set_loading.set(false);
        });
    };

    fetch_webhooks();

    let dismiss_error = Callback::new(move |_: ()| set_error.set(None));

    let handle_test = move |wh: WebhookItem| {
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        let o = owner();
        let n = name();
        let wh_id = wh.id.clone();
        set_testing_id.set(Some(wh_id.clone()));
        leptos::task::spawn_local(async move {
            let path = format!("/repos/{o}/{n}/webhooks/{wh_id}/test");
            let _ = client.post(&path, &serde_json::json!({})).await;
            set_testing_id.set(None);
        });
    };

    let handle_toggle = move |wh: WebhookItem| {
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        let o = owner();
        let n = name();
        let wh_id = wh.id.clone();
        let new_active = !wh.active;
        leptos::task::spawn_local(async move {
            let path = format!("/repos/{o}/{n}/webhooks/{wh_id}");
            let body = serde_json::json!({ "active": new_active });
            let _ = client.patch(&path, &body).await;
            fetch_webhooks();
        });
    };

    let handle_delete = move |wh: WebhookItem| {
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        let o = owner();
        let n = name();
        let wh_id = wh.id.clone();
        leptos::task::spawn_local(async move {
            let path = format!("/repos/{o}/{n}/webhooks/{wh_id}");
            let _ = client.delete(&path).await;
            fetch_webhooks();
        });
    };

    let load_deliveries = move |wh: WebhookItem| {
        set_show_deliveries.set(Some(wh.clone()));
        set_delivery_loading.set(true);
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        let o = owner();
        let n = name();
        let wh_id = wh.id.clone();
        leptos::task::spawn_local(async move {
            let path = format!("/repos/{o}/{n}/webhooks/{wh_id}/deliveries");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<Vec<DeliveryItem>>().await {
                        set_deliveries.set(data);
                    }
                }
                _ => {}
            }
            set_delivery_loading.set(false);
        });
    };

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl sm:text-3xl font-bold font-mono text-gray-900 dark:text-gray-100">"WEBHOOKS"</h1>
                    <p class="mt-1 text-sm text-gray-500 dark:text-gray-400 font-mono">
                        {move || format!("{}/{}", owner(), name())}
                    </p>
                </div>
                <Button variant=ButtonVariant::Primary on:click=move |_| set_show_create.set(true)>
                    "New Webhook"
                </Button>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=dismiss_error />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center justify-center py-12">
                    <Spinner />
                </div>
            </Show>

            <Show when=move || !loading.get() && webhooks.get().is_empty() && error.get().is_none() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="py-12 text-center">
                        <p class="text-gray-500 dark:text-gray-400">"No webhooks configured."</p>
                    </div>
                </Card>
            </Show>

            <Show when=move || !webhooks.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="divide-y divide-gray-100 dark:divide-gray-700">
                        <For each=move || webhooks.get() key=|w| w.id.clone() let:wh>
                            {
                                let wh_clone = wh.clone();
                                let wh_clone2 = wh.clone();
                                let wh_clone3 = wh.clone();
                                let wh_clone4 = wh.clone();
                                let wh_clone5 = wh.clone();
                                let testing = testing_id.get() == Some(wh.id.clone());
                                view! {
                                    <div class="flex items-center justify-between py-4 px-2 gap-4">
                                        <div class="min-w-0 flex-1">
                                            <div class="flex items-center gap-2 flex-wrap">
                                                <span class="font-mono text-sm text-gray-900 dark:text-gray-100 truncate">
                                                    {wh.url.clone()}
                                                </span>
                                                <Badge
                                                    color=if wh.active { BadgeColor::Success } else { BadgeColor::Neutral }
                                                    text=if wh.active { "active" } else { "disabled" }.to_string()
                                                />
                                            </div>
                                            <div class="flex items-center gap-1 mt-1 flex-wrap">
                                                {wh.events.iter().map(|ev| {
                                                    view! {
                                                        <span class="inline-block px-1.5 py-0.5 text-xs font-mono bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 rounded">
                                                            {ev.clone()}
                                                        </span>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                            <div class="text-xs text-gray-400 dark:text-gray-500 mt-1">
                                                {relative_time(&wh.created_at)}
                                            </div>
                                        </div>
                                        <div class="flex items-center gap-1 shrink-0">
                                            <button
                                                class="px-2 py-1 text-xs font-medium text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors"
                                                on:click=move |_| handle_test(wh_clone.clone())
                                                disabled=testing
                                            >
                                                {if testing { "Testing..." } else { "Test" }}
                                            </button>
                                            <button
                                                class="px-2 py-1 text-xs font-medium text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors"
                                                on:click=move |_| load_deliveries(wh_clone2.clone())
                                            >
                                                "Deliveries"
                                            </button>
                                            <button
                                                class="px-2 py-1 text-xs font-medium text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors"
                                                on:click=move |_| set_editing.set(Some(wh_clone3.clone()))
                                            >
                                                "Edit"
                                            </button>
                                            <button
                                                class="px-2 py-1 text-xs font-medium text-yellow-600 dark:text-yellow-400 hover:bg-yellow-50 dark:hover:bg-yellow-900/20 rounded transition-colors"
                                                on:click=move |_| handle_toggle(wh_clone4.clone())
                                            >
                                                {if wh_clone4.active { "Disable" } else { "Enable" }}
                                            </button>
                                            <button
                                                class="px-2 py-1 text-xs font-medium text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors"
                                                on:click=move |_| handle_delete(wh_clone5.clone())
                                            >
                                                "Delete"
                                            </button>
                                        </div>
                                    </div>
                                }
                            }
                        </For>
                    </div>
                </Card>
            </Show>
        </div>
    }
}
