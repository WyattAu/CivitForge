#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::components::{Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Spinner};
use crate::state::auth::use_auth;
use crate::utils::*;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
struct ReviewThread {
    id: String,
    file_path: Option<String>,
    line: Option<i32>,
    body: String,
    author: String,
    resolved: bool,
    created_at: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct CreateReviewThreadBody {
    file_path: Option<String>,
    line: Option<i32>,
    body: String,
}

fn review_status_color(resolved: bool) -> BadgeColor {
    if resolved {
        BadgeColor::Success
    } else {
        BadgeColor::Warning
    }
}

#[component]
pub fn CodeReviewThreadPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let number = move || {
        params.with(|p| {
            p.get("number")
                .and_then(|n| n.parse::<i64>().ok())
                .unwrap_or(0)
        })
    };
    let auth = use_auth();

    let (threads, set_threads) = signal(Vec::<ReviewThread>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (show_new_thread, set_show_new_thread) = signal(false);
    let (new_file, set_new_file) = signal(String::new());
    let (new_line, set_new_line) = signal(String::new());
    let (new_body, set_new_body) = signal(String::new());
    let (submitting, set_submitting) = signal(false);
    let (submit_error, set_submit_error) = signal(None::<String>);

    let fetch_threads = move || {
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        let number_val = number();
        set_loading.set(true);
        set_error.set(None);

        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!(
                "/repos/{owner_val}/{name_val}/pulls/{number_val}/threads"
            );
            if let Ok(resp) = client.get(&path).await {
                if resp.status().is_success() {
                    if let Ok(data) = resp.json::<Vec<ReviewThread>>().await {
                        set_threads.set(data);
                        set_loading.set(false);
                        return;
                    }
                }
            }
            // Demo data
            set_threads.set(vec![
                ReviewThread {
                    id: "thread-1".into(),
                    file_path: Some("src/main.rs".into()),
                    line: Some(42),
                    body: "This could be simplified using iterator combinators.".into(),
                    author: "user-abc12345".into(),
                    resolved: false,
                    created_at: "2025-01-15T10:00:00Z".into(),
                },
                ReviewThread {
                    id: "thread-2".into(),
                    file_path: Some("src/lib.rs".into()),
                    line: Some(15),
                    body: "Missing error handling here.".into(),
                    author: "user-ghi11111".into(),
                    resolved: true,
                    created_at: "2025-01-14T09:00:00Z".into(),
                },
            ]);
            set_loading.set(false);
        });
    };

    leptos::task::spawn_local(async move {
        fetch_threads();
    });

    let handle_new_thread = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_submit_error.set(None);
        let body_text = new_body.get();
        if body_text.trim().is_empty() {
            set_submit_error.set(Some("Comment is required.".into()));
            return;
        }
        let file_path = if new_file.get().trim().is_empty() {
            None
        } else {
            Some(new_file.get().trim().to_string())
        };
        let line: Option<i32> = new_line.get().parse().ok();

        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        let number_val = number();
        set_submitting.set(true);

        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!(
                "/repos/{owner_val}/{name_val}/pulls/{number_val}/threads"
            );
            let body = CreateReviewThreadBody {
                file_path,
                line,
                body: body_text.trim().to_string(),
            };
            let _ = client.post(&path, &body).await;
            set_show_new_thread.set(false);
            set_new_body.set(String::new());
            set_submitting.set(false);
            fetch_threads();
        });
    };

    let toggle_resolve = move |thread_id: String, currently_resolved: bool| {
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        let number_val = number();
        let new_resolved = !currently_resolved;
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!(
                "/repos/{owner_val}/{name_val}/pulls/{number_val}/threads/{thread_id}"
            );
            let body = serde_json::json!({ "resolved": new_resolved });
            let _ = client.patch(&path, &body).await;
            fetch_threads();
        });
    };

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between flex-wrap gap-4">
                <div>
                    <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1">
                        <A href=format!("/repos/{}/{}", owner(), name())>
                            <span class="hover:text-blue-600 dark:hover:text-blue-400">
                                {move || format!("{}/{}", owner(), name())}
                            </span>
                        </A>
                        <span class="hidden sm:inline">"/"</span>
                        <A href=format!("/repos/{}/{}/pulls/{}", owner(), name(), number())>
                            <span class="hover:text-blue-600 dark:hover:text-blue-400">
                                {move || format!("PR #{}", number())}
                            </span>
                        </A>
                        <span class="hidden sm:inline">"/"</span>
                        <span class="hidden sm:inline text-gray-700 dark:text-gray-300">"Review"</span>
                    </div>
                    <h1 class="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-gray-100">"Code Review"</h1>
                </div>
                <Button variant=ButtonVariant::Primary on:click=move |_| set_show_new_thread.set(!show_new_thread.get())>
                    {move || if show_new_thread.get() { "Cancel" } else { "New Thread" }}
                </Button>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_error.set(None)) />
            </Show>

            <Show when=move || show_new_thread.get() fallback=|| view! { <div class="hidden"></div> }>
                <Card title="New Review Thread".to_string()>
                    <form on:submit=handle_new_thread class="space-y-4">
                        <Show when=move || submit_error.get().is_some()>
                            <ErrorBanner message=move || submit_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_submit_error.set(None)) />
                        </Show>
                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"File Path (optional)"</label>
                                <input type="text" class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500" placeholder="src/main.rs" prop:value=new_file on:input=move |ev| set_new_file.set(event_target_value(&ev)) />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Line Number (optional)"</label>
                                <input type="number" min="1" class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500" placeholder="42" prop:value=new_line on:input=move |ev| set_new_line.set(event_target_value(&ev)) />
                            </div>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Comment"</label>
                            <textarea class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500" placeholder="Your review comment..." rows="4" required prop:value=new_body on:input=move |ev| set_new_body.set(event_target_value(&ev))></textarea>
                        </div>
                        <Button variant=ButtonVariant::Primary disabled=submitting.get()>
                            {move || if submitting.get() { "Creating..." } else { "Create Thread" }}
                        </Button>
                    </form>
                </Card>
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="flex items-center justify-center py-12">
                        <Spinner />
                        <span class="ml-3 text-gray-500 dark:text-gray-400">"Loading threads..."</span>
                    </div>
                </Card>
            </Show>

            <Show when=move || !loading.get() && threads.get().is_empty() && error.get().is_none() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="text-center py-12">
                        <h3 class="text-sm font-medium text-gray-900 dark:text-gray-100">"No review threads yet"</h3>
                        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">"Start a review discussion on this pull request."</p>
                    </div>
                </Card>
            </Show>

            <Show when=move || !loading.get() && !threads.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                <div class="space-y-4">
                    <For
                        each=move || threads.get()
                        key=|t| t.id.clone()
                        children=move |thread| {
                            let tid = thread.id.clone();
                            let resolved = thread.resolved;
                            let toggle = toggle_resolve.clone();
                            view! {
                                <div class={
                                    if thread.resolved {
                                        "border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/30 rounded-lg overflow-hidden"
                                    } else {
                                        "border border-yellow-300 dark:border-yellow-600 bg-white dark:bg-gray-800 rounded-lg overflow-hidden"
                                    }
                                }>
                                    <div class="p-4">
                                        <div class="flex items-center gap-2 mb-2 flex-wrap">
                                            <Badge color=review_status_color(thread.resolved) text=if thread.resolved { "Resolved".into() } else { "Unresolved".into() } />
                                            {thread.file_path.as_ref().map(|fp| {
                                                view! { <span class="text-xs font-mono text-gray-500 dark:text-gray-400">{fp.clone()}</span> }
                                            })}
                                            {thread.line.map(|l| {
                                                view! { <span class="text-xs text-gray-400 dark:text-gray-500">{format!("line {l}")}</span> }
                                            })}
                                        </div>
                                        <div class="flex gap-3">
                                            <div class="w-8 h-8 rounded-full bg-gray-300 dark:bg-gray-600 flex items-center justify-center text-xs font-medium text-gray-700 dark:text-gray-300 shrink-0">
                                                {thread.author.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default()}
                                            </div>
                                            <div class="flex-1 min-w-0">
                                                <div class="flex items-center gap-2">
                                                    <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{truncate_uuid(&thread.author, 8)}</span>
                                                    <span class="text-xs text-gray-400 dark:text-gray-500">{relative_time(&thread.created_at)}</span>
                                                </div>
                                                <p class="text-sm text-gray-700 dark:text-gray-300 mt-1 whitespace-pre-wrap">{thread.body}</p>
                                            </div>
                                        </div>
                                        <div class="mt-3">
                                            <Button
                                                variant=if thread.resolved { ButtonVariant::Secondary } else { ButtonVariant::Primary }
                                                on:click=move |_| toggle(tid.clone(), resolved)
                                            >
                                                {move || if thread.resolved { "Unresolve" } else { "Resolve" }}
                                            </Button>
                                        </div>
                                    </div>
                                </div>
                            }
                        }
                    />
                </div>
            </Show>
        </div>
    }
}
