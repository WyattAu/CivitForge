#![forbid(unsafe_code)]

use leptos::prelude::*;

use crate::api::client::ApiClient;
use crate::components::{Badge, BadgeColor, Card, ErrorBanner, Spinner};
use crate::state::auth::use_auth;
use crate::utils::*;

#[derive(Clone, serde::Deserialize)]
pub struct PrTemplateItem {
    pub id: String,
    pub name: String,
    pub title: String,
    pub body: String,
    pub base_branch: String,
    pub labels: Vec<String>,
    pub created_at: String,
}

#[component]
pub fn PrTemplatesPage() -> impl IntoView {
    let auth = use_auth();
    let (templates, set_templates) = signal(Vec::<PrTemplateItem>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    leptos::task::spawn_local(async move {
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        match client.get("/activity?limit=50").await {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<Vec<PrTemplateItem>>().await {
                    Ok(data) => set_templates.set(data),
                    Err(_) => set_error.set(Some(sanitize_error("Failed to parse templates."))),
                }
            }
            Ok(_) => {
                set_error.set(Some(sanitize_error("Failed to load PR templates.")));
            }
            Err(e) => {
                let msg = format!("{e}");
                set_error.set(Some(sanitize_error(&msg)));
            }
        }
        set_loading.set(false);
    });

    let dismiss_error = Callback::new(move |_: ()| set_error.set(None));

    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"PR Templates"</h1>
                <p class="mt-1 text-gray-600 dark:text-gray-400">
                    "Templates for creating new pull requests."
                </p>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=dismiss_error />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center gap-2">
                    <Spinner />
                    <span class="text-gray-500 dark:text-gray-400">"Loading templates..."</span>
                </div>
            </Show>

            <Show when=move || !loading.get() && templates.get().is_empty() && error.get().is_none() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="text-center py-12">
                        <svg class="mx-auto h-12 w-12 text-gray-300 dark:text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                        </svg>
                        <h2 class="mt-2 text-sm font-medium text-gray-900 dark:text-gray-100">"No PR templates"</h2>
                        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                            "Create a template to standardize pull request descriptions."
                        </p>
                    </div>
                </Card>
            </Show>

            <Show when=move || !templates.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="divide-y divide-gray-100 dark:divide-gray-700">
                        <For each=move || templates.get() key=|t| t.id.clone() let:item>
                            {
                                let time_str = relative_time(&item.created_at);
                                view! {
                                    <div class="py-3 px-1 hover:bg-gray-50 dark:hover:bg-gray-750 -mx-1 rounded transition-colors">
                                        <div class="flex items-center gap-2">
                                            <span class="text-sm font-medium text-gray-900 dark:text-gray-100">
                                                {item.name}
                                            </span>
                                            <Badge color=BadgeColor::Info text=item.base_branch.clone() />
                                        </div>
                                        <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{item.title}</p>
                                        <div class="flex items-center gap-2 mt-1 text-xs text-gray-400 dark:text-gray-500">
                                            <span>{time_str}</span>
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
