#![forbid(unsafe_code)]

use leptos::prelude::*;

use crate::api::client::ApiClient;
use crate::components::{Card, Spinner};
use crate::state::auth::use_auth;

#[derive(Clone, serde::Deserialize)]
pub struct SearchHit {
    pub file_path: String,
    pub language: Option<String>,
    pub line_number: i32,
    pub line_content: String,
}

#[derive(Clone, serde::Deserialize)]
pub struct SearchEnvelope {
    pub results: Vec<SearchHit>,
    pub total: i64,
    #[allow(dead_code)]
    pub page: i64,
    #[allow(dead_code)]
    pub per_page: i64,
}

#[component]
pub fn SearchPage() -> impl IntoView {
    let auth = use_auth();
    let (query_sig, set_query) = signal(String::new());
    let (results_sig, set_results) = signal(None::<SearchEnvelope>);
    let (loading, set_loading) = signal(false);
    let (searched, set_searched) = signal(false);
    let (error, set_error) = signal(None::<String>);

    let fetch_results = move |q: String| {
        if q.trim().is_empty() {
            set_results.set(None);
            set_searched.set(false);
            return;
        }
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        let path = format!("/search?q={}&page=1&per_page=30", q.replace(' ', "+"));
        leptos::task::spawn_local(async move {
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<SearchEnvelope>().await {
                        Ok(data) => set_results.set(Some(data)),
                        Err(_) => {
                            set_error.set(Some("Failed to parse search results.".to_string()))
                        }
                    }
                }
                Ok(_) => {
                    set_error.set(Some("Search request failed.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_loading.set(false);
            set_searched.set(true);
        });
    };

    let on_input = move |ev: leptos::ev::Event| {
        let value = event_target_value(&ev);
        set_query.set(value.clone());
    };

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let q = query_sig.get();
        fetch_results(q);
    };

    let on_search_click = move |_| {
        let q = query_sig.get();
        fetch_results(q);
    };

    let dismiss_error = move |_| set_error.set(None);

    let has_results = move || results_sig.get().is_some_and(|r| !r.results.is_empty());
    let no_results =
        move || searched.get() && results_sig.get().is_some_and(|r| r.results.is_empty());
    let show_initial = move || !searched.get() && !loading.get();

    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Search"</h1>
                <p class="mt-1 text-gray-600 dark:text-gray-400">"Search code across all repositories."</p>
            </div>

            <Card class="p-6".to_string()>
                <form on:submit=on_submit class="flex gap-3">
                    <input
                        type="text"
                        id="search-input"
                        placeholder="Search code..."
                        class="flex-1 px-4 py-2 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent text-sm"
                        prop:value=move || query_sig.get()
                        on:input=on_input
                    />
                    <button
                        type="button"
                        on:click=on_search_click
                        class="px-4 py-2 rounded-md bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                        disabled=move || query_sig.get().trim().is_empty() || loading.get()
                    >
                        "Search"
                    </button>
                </form>
            </Card>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md flex items-center justify-between">
                    <p class="text-sm text-red-700 dark:text-red-400">{move || error.get().unwrap_or_default()}</p>
                    <button on:click=dismiss_error class="text-red-500 hover:text-red-700 dark:text-red-400 dark:hover:text-red-200 text-sm font-medium">"Dismiss"</button>
                </div>
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center justify-center py-12">
                    <Spinner />
                </div>
            </Show>

            <Show when=show_initial fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="py-12 text-center">
                        <svg class="w-12 h-12 mx-auto text-gray-300 dark:text-gray-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
                        </svg>
                        <p class="text-gray-500 dark:text-gray-400">"Enter a search query to find code across repositories."</p>
                    </div>
                </Card>
            </Show>

            <Show when=no_results fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="py-12 text-center">
                        <p class="text-gray-500 dark:text-gray-400 text-lg">"No results found."</p>
                        <p class="text-sm text-gray-400 dark:text-gray-500 mt-1">"Try adjusting your search query."</p>
                    </div>
                </Card>
            </Show>

            <Show when=has_results fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="flex items-center justify-between mb-4">
                        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">"Code Search Results"</h3>
                        <span class="text-sm text-gray-500 dark:text-gray-400">{move || results_sig.get().map(|r| r.total.to_string()).unwrap_or_default()} "results"</span>
                    </div>
                    <div class="space-y-3">
                        <For each=move || results_sig.get().map(|r| r.results).unwrap_or_default() key=|h| format!("{}:{}", h.file_path, h.line_number) let:hit>
                            {
                                let lang = hit.language.clone().unwrap_or_default();
                                let file_path = hit.file_path.clone();
                                view! {
                                    <div class="border border-gray-100 dark:border-gray-700 rounded-md overflow-hidden">
                                        <div class="flex items-center gap-2 px-3 py-1.5 bg-gray-50 dark:bg-gray-750 border-b border-gray-100 dark:border-gray-700">
                                            <span class="text-xs font-mono text-blue-600 dark:text-blue-400 truncate">{file_path}</span>
                                            <span class="text-xs text-gray-400 dark:text-gray-500 shrink-0">":"</span>
                                            <span class="text-xs font-mono text-gray-500 dark:text-gray-400">{hit.line_number}</span>
                                            {(!lang.is_empty()).then(|| view! {
                                                <span class="ml-auto text-xs text-gray-400 dark:text-gray-500 shrink-0">{lang}</span>
                                            })}
                                        </div>
                                        <pre class="px-3 py-2 text-sm font-mono text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 overflow-x-auto whitespace-pre-wrap break-all">{hit.line_content.clone()}</pre>
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
