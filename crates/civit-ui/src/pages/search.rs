#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos::ev::SubmitEvent;

use crate::api::client::ApiClient;
use crate::api::types::{SearchSuggestResponse, SearchHistoryResponse};
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

    // Advanced search state
    let (suggestions, set_suggestions) = signal(Vec::<crate::api::types::SearchSuggestionItem>::new());
    let (show_suggestions, set_show_suggestions) = signal(false);
    let (show_history, set_show_history) = signal(false);
    let (history_items, set_history_items) = signal(Vec::<crate::api::types::SearchHistoryItem>::new());
    let (show_help, set_show_help) = signal(false);

    let fetch_results = move |q: String| {
        if q.trim().is_empty() {
            set_results.set(None);
            set_searched.set(false);
            return;
        }
        set_loading.set(true);
        set_error.set(None);
        set_show_suggestions.set(false);
        set_show_history.set(false);

        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        let path = format!("/search?q={}&page=1&per_page=30", q.replace(' ', "+"));
        leptos::task::spawn_local(async move {
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<SearchEnvelope>().await {
                        Ok(data) => {
                            // Record search in history
                            let result_count = data.total;
                            let token_h = auth.0.with(|a| a.token.clone());
                            let client_h = ApiClient::new(token_h);
                            let q_clone = q.clone();
                            leptos::task::spawn_local(async move {
                                let _ = client_h.post_json("/search/history", &serde_json::json!({
                                    "query": q_clone,
                                    "result_count": result_count
                                })).await;
                            });
                            set_results.set(Some(data));
                        }
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

    let fetch_suggestions = move |q: String| {
        if q.trim().len() < 2 {
            set_suggestions.set(Vec::new());
            set_show_suggestions.set(false);
            return;
        }
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        let path = format!("/search/suggest?q={}", q.replace(' ', "+"));
        leptos::task::spawn_local(async move {
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<SearchSuggestResponse>().await {
                        set_suggestions.set(data.suggestions);
                        set_show_suggestions.set(true);
                    }
                }
                _ => {}
            }
        });
    };

    let fetch_history = move || {
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        leptos::task::spawn_local(async move {
            match client.get("/search/history?page=1&per_page=10").await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<SearchHistoryResponse>().await {
                        set_history_items.set(data.items);
                        set_show_history.set(true);
                    }
                }
                _ => {}
            }
        });
    };

    let on_input = move |ev: leptos::ev::Event| {
        let value = event_target_value(&ev);
        set_query.set(value.clone());
        fetch_suggestions(value);
    };

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let q = query_sig.get();
        fetch_results(q);
    };

    let on_search_click = move |_| {
        let q = query_sig.get();
        fetch_results(q);
    };

    let on_input_focus = move |_| {
        let q = query_sig.get();
        if q.trim().is_empty() {
            fetch_history();
        } else if q.trim().len() >= 2 {
            fetch_suggestions(q);
        }
    };

    let dismiss_error = move |_| set_error.set(None);

    let apply_suggestion = move |text: String| {
        set_query.set(text.clone());
        set_show_suggestions.set(false);
        fetch_results(text);
    };

    let apply_history = move |q: String| {
        set_query.set(q.clone());
        set_show_history.set(false);
        fetch_results(q);
    };

    let has_results = move || results_sig.get().is_some_and(|r| !r.results.is_empty());
    let no_results =
        move || searched.get() && results_sig.get().is_some_and(|r| r.results.is_empty());
    let show_initial = move || !searched.get() && !loading.get();

    view! {
        <div class="space-y-6">
            <div>
                <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1">
                    <span class="text-gray-700 dark:text-gray-300">"Search"</span>
                </div>
                <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Search"</h1>
                <p class="mt-1 text-gray-600 dark:text-gray-400">"Search code across all repositories."</p>
            </div>

            <Card class="p-6".to_string()>
                <form on:submit=on_submit class="flex gap-3">
                    <div class="relative flex-1">
                        <input
                            type="text"
                            id="search-input"
                            aria-label="Search code"
                            placeholder=r#"Search code... (supports repo:name, user:name, is:issue|pr, status:open|closed, language:rust, created:>2024-01-01)"#
                            class="w-full px-4 py-2 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent text-sm"
                            prop:value=move || query_sig.get()
                            on:input=on_input
                            on:focus=on_input_focus
                        />
                        <Show when=move || show_suggestions.get() && !suggestions.get().is_empty()>
                            <div class="absolute z-10 top-full left-0 right-0 mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-md shadow-lg max-h-60 overflow-y-auto">
                                {move || suggestions.get().iter().map(|s| {
                                    let text = s.text.clone();
                                    let category = s.category.clone();
                                    let apply = apply_suggestion.clone();
                                    view! {
                                        <div
                                            class="px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer flex items-center justify-between"
                                            on:click=move |ev| {
                                                ev.prevent_default();
                                                apply(text.clone());
                                            }
                                        >
                                            <span class="text-sm text-gray-900 dark:text-gray-100">{s.text.clone()}</span>
                                            <span class="text-xs text-gray-400 dark:text-gray-500">{category.clone()}</span>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </Show>
                        <Show when=move || show_history.get() && !history_items.get().is_empty()>
                            <div class="absolute z-10 top-full left-0 right-0 mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-md shadow-lg max-h-60 overflow-y-auto">
                                <div class="px-4 py-1.5 text-xs text-gray-400 dark:text-gray-500 border-b border-gray-100 dark:border-gray-700">"Recent searches"</div>
                                {move || history_items.get().iter().map(|h| {
                                    let query = h.query.clone();
                                    let apply = apply_history.clone();
                                    view! {
                                        <div
                                            class="px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer flex items-center justify-between"
                                            on:click=move |ev| {
                                                ev.prevent_default();
                                                apply(query.clone());
                                            }
                                        >
                                            <span class="text-sm text-gray-900 dark:text-gray-100">{h.query.clone()}</span>
                                            <span class="text-xs text-gray-400 dark:text-gray-500">{h.result_count.to_string()} " results"</span>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </Show>
                    </div>
                    <button
                        type="button"
                        on:click=move |_| set_show_help.update(|v| *v = !*v)
                        class="px-3 py-2 rounded-md border border-gray-300 dark:border-gray-600 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 text-sm font-medium transition-colors"
                        title="Search syntax help"
                    >
                        "?"
                    </button>
                    <button
                        type="button"
                        on:click=on_search_click
                        class="px-4 py-2 rounded-md bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                        disabled=move || query_sig.get().trim().is_empty() || loading.get()
                    >
                        "Search"
                    </button>
                </form>
                <Show when=move || show_help.get()>
                    <div class="mt-3 p-3 bg-gray-50 dark:bg-gray-750 border border-gray-200 dark:border-gray-700 rounded-md text-sm">
                        <p class="font-medium text-gray-700 dark:text-gray-300 mb-2">"Advanced Search Syntax"</p>
                        <ul class="space-y-1 text-gray-600 dark:text-gray-400">
                            <li><code class="bg-gray-200 dark:bg-gray-600 px-1 rounded">"repo:owner/name"</code> " — Search in a specific repository"</li>
                            <li><code class="bg-gray-200 dark:bg-gray-600 px-1 rounded">"user:username"</code> " — Filter by author"</li>
                            <li><code class="bg-gray-200 dark:bg-gray-600 px-1 rounded">"is:issue"</code> " or " <code class="bg-gray-200 dark:bg-gray-600 px-1 rounded">"is:pr"</code> " — Filter by type"</li>
                            <li><code class="bg-gray-200 dark:bg-gray-600 px-1 rounded">"status:open"</code> " or " <code class="bg-gray-200 dark:bg-gray-600 px-1 rounded">"status:closed"</code> " — Filter by status"</li>
                            <li><code class="bg-gray-200 dark:bg-gray-600 px-1 rounded">"language:rust"</code> " — Filter by language"</li>
                            <li><code class="bg-gray-200 dark:bg-gray-600 px-1 rounded">"created:>2024-01-01"</code> " — Filter by creation date"</li>
                        </ul>
                    </div>
                </Show>
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
                        <p class="text-xs text-gray-400 dark:text-gray-500 mt-2">"Use advanced syntax for precise filtering. Click ? for help."</p>
                    </div>
                </Card>
            </Show>

            <Show when=no_results fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="py-12 text-center">
                        <p class="text-gray-500 dark:text-gray-400 text-lg">"No results found."</p>
                        <p class="text-sm text-gray-400 dark:text-gray-500 mt-1">"Try adjusting your search query or filters."</p>
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
