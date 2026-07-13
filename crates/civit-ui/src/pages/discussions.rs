#![forbid(unsafe_code)]

use leptos::prelude::*;

use crate::api::client::ApiClient;
use crate::components::{Avatar, Badge, BadgeColor, Card, ErrorBanner, Spinner};
use crate::state::auth::use_auth;
use crate::utils::*;

#[derive(Clone, serde::Deserialize)]
pub struct DiscussionItem {
    pub id: String,
    pub title: String,
    pub body: String,
    pub category: String,
    pub author_id: String,
    pub is_pinned: bool,
    pub is_locked: bool,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub comment_count: Option<i64>,
    #[serde(default)]
    pub labels: Vec<DiscussionLabelItem>,
}

#[derive(Clone, serde::Deserialize)]
pub struct DiscussionCommentItem {
    pub id: String,
    pub discussion_id: String,
    pub author_id: String,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub reactions: Vec<DiscussionReactionItem>,
}

#[derive(Clone, serde::Deserialize)]
pub struct DiscussionLabelItem {
    pub id: String,
    pub discussion_id: String,
    pub label: String,
    pub color: String,
}

#[derive(Clone, serde::Deserialize)]
pub struct DiscussionReactionItem {
    pub id: String,
    pub comment_id: String,
    pub user_id: String,
    pub emoji: String,
    pub created_at: String,
}

fn category_badge(category: &str) -> (BadgeColor, String) {
    match category {
        "general" => (BadgeColor::Neutral, "general".into()),
        "rfc" => (BadgeColor::Info, "rfc".into()),
        "q&a" => (BadgeColor::Success, "q&a".into()),
        "show-and-tell" => (BadgeColor::Warning, "show & tell".into()),
        _ => (BadgeColor::Neutral, category.to_string()),
    }
}

const DISCUSSION_FILTERS: &[&str] = &["all", "general", "rfc", "q&a", "show-and-tell"];

#[component]
pub fn DiscussionsPage() -> impl IntoView {
    let auth = use_auth();
    let (discussions, set_discussions) = signal(Vec::<DiscussionItem>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (filter, set_filter) = signal("all".to_string());
    let (search_query, set_search_query) = signal(String::new());

    let fetch_discussions = move |query: String| {
        let auth_clone = auth.clone();
        let set_discussions = set_discussions.clone();
        let set_loading = set_loading.clone();
        let set_error = set_error.clone();

        leptos::task::spawn_local(async move {
            set_loading.set(true);
            let token = auth_clone.0.with(|a| a.token.clone());
            let client = ApiClient::new(token);

            let url = if query.is_empty() {
                "/activity?limit=50".to_string()
            } else {
                format!(
                    "/activity?limit=50&search={}",
                    urlencoding::encode(&query)
                )
            };

            match client.get(&url).await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<Vec<DiscussionItem>>().await {
                        Ok(data) => set_discussions.set(data),
                        Err(_) => {
                            set_error.set(Some(sanitize_error("Failed to parse discussions.")))
                        }
                    }
                }
                Ok(_) => {
                    set_error.set(Some(sanitize_error("Failed to load discussions.")));
                }
                Err(e) => {
                    let msg = format!("{e}");
                    set_error.set(Some(sanitize_error(&msg)));
                }
            }
            set_loading.set(false);
        });
    };

    // Initial fetch
    fetch_discussions(String::new());

    let filtered = move || {
        let f = filter.get();
        let items = discussions.get();
        if f == "all" {
            items
        } else {
            items
                .into_iter()
                .filter(|d| d.category == f)
                .collect::<Vec<_>>()
        }
    };

    let dismiss_error = Callback::new(move |_: ()| set_error.set(None));

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Discussions"</h1>
                    <p class="mt-1 text-gray-600 dark:text-gray-400">
                        "Community discussions for this repository."
                    </p>
                </div>
            </div>

            <div class="flex items-center gap-4">
                <div class="flex-1 max-w-md">
                    <input
                        type="text"
                        placeholder="Search discussions..."
                        class="w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                        on:input=move |ev| {
                            let val = event_target_value(&ev);
                            set_search_query.set(val.clone());
                            fetch_discussions(val);
                        }
                        prop:value=move || search_query.get()
                    />
                </div>
            </div>

            <div class="flex gap-2 flex-wrap">
                {DISCUSSION_FILTERS
                    .iter()
                    .map(|f| {
                        let f_str = (*f).to_string();
                        let label = (*f).to_uppercase();
                        let on_click_f = f_str.clone();
                        let is_active = move || filter.get() == f_str;
                        view! {
                            <button
                                class=move || if is_active() {
                                    "px-3 py-1 text-sm font-mono rounded-sm border-2 border-blue-600 dark:border-blue-400 bg-blue-600 text-white dark:bg-blue-400 dark:text-gray-900"
                                } else {
                                    "px-3 py-1 text-sm font-mono rounded-sm border-2 border-gray-300 dark:border-gray-600 bg-transparent text-gray-700 dark:text-gray-300 hover:border-blue-500 dark:hover:border-blue-400"
                                }
                                on:click=move |_| set_filter.set(on_click_f.clone())
                            >
                                {label}
                            </button>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>

            <Show
                when=move || error.get().is_some()
                fallback=|| view! { <div class="hidden"></div> }
            >
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=dismiss_error />
            </Show>

            <Show
                when=move || loading.get()
                fallback=|| view! { <div class="hidden"></div> }
            >
                <div class="flex items-center gap-2">
                    <Spinner />
                    <span class="text-gray-500 dark:text-gray-400">"Loading discussions..."</span>
                </div>
            </Show>

            <Show
                when=move || !loading.get() && filtered().is_empty() && error.get().is_none()
                fallback=|| view! { <div class="hidden"></div> }
            >
                <Card>
                    <div class="text-center py-12">
                        <svg class="mx-auto h-12 w-12 text-gray-300 dark:text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"/>
                        </svg>
                        <h2 class="mt-2 text-sm font-medium text-gray-900 dark:text-gray-100">"No discussions"</h2>
                        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                            "Start a discussion to engage with the community."
                        </p>
                    </div>
                </Card>
            </Show>

            <Show
                when=move || !filtered().is_empty()
                fallback=|| view! { <div class="hidden"></div> }
            >
                <Card>
                    <div class="divide-y divide-gray-100 dark:divide-gray-700">
                        <For
                            each=move || filtered()
                            key=|d| d.id.clone()
                            let:item
                        >
                            {
                                let (badge_color, badge_text) = category_badge(&item.category);
                                let time_str = relative_time(&item.created_at);
                                let author_short = if item.author_id.len() > 8 {
                                    item.author_id[..8].to_string()
                                } else {
                                    item.author_id.clone()
                                };
                                let comment_count = item.comment_count.unwrap_or(0);
                                let labels = item.labels.clone();
                                view! {
                                    <div class="py-3 px-1 hover:bg-gray-50 dark:hover:bg-gray-750 -mx-1 rounded transition-colors">
                                        <div class="flex items-center gap-2 flex-wrap">
                                            {item.is_pinned.then(|| {
                                                view! {
                                                    <Badge color=BadgeColor::Success text="pinned".into() />
                                                }
                                            })}
                                            <span class="text-sm font-medium text-gray-900 dark:text-gray-100">
                                                {item.title}
                                            </span>
                                            <Badge color=badge_color text=badge_text />
                                            {item.is_locked.then(|| {
                                                view! {
                                                    <Badge color=BadgeColor::Neutral text="locked".into() />
                                                }
                                            })}
                                            {labels
                                                .into_iter()
                                                .map(|l| {
                                                    let color = l.color.clone();
                                                    let label = l.label.clone();
                                                    view! {
                                                        <span
                                                            class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium text-white"
                                                            style:background-color=move || color.clone()
                                                        >
                                                            {label}
                                                        </span>
                                                    }
                                                })
                                                .collect::<Vec<_>>()}
                                        </div>
                                        <p class="text-sm text-gray-500 dark:text-gray-400 mt-1 line-clamp-2">{item.body}</p>
                                        <div class="flex items-center gap-3 mt-1 text-xs text-gray-400 dark:text-gray-500">
                                            <Avatar name=item.author_id.clone() size=16 />
                                            <span>{author_short}</span>
                                            <span>{time_str}</span>
                                            <span>{format!("{comment_count} comments")}</span>
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
