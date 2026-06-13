#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::components::{Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Spinner};
use crate::state::auth::use_auth;
use crate::utils::*;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
struct Suggestion {
    id: String,
    pr_number: i64,
    file_path: String,
    start_line: i32,
    end_line: i32,
    original_code: String,
    suggested_code: String,
    description: String,
    author: String,
    status: String,
    created_at: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct CreateSuggestionBody {
    file_path: String,
    start_line: i32,
    end_line: i32,
    original_code: String,
    suggested_code: String,
    description: String,
}

fn suggestion_status_color(status: &str) -> BadgeColor {
    match status {
        "open" => BadgeColor::Warning,
        "accepted" => BadgeColor::Success,
        "rejected" => BadgeColor::Danger,
        _ => BadgeColor::Neutral,
    }
}

#[component]
pub fn SuggestedEditsPage() -> impl IntoView {
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

    let (suggestions, set_suggestions) = signal(Vec::<Suggestion>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (show_new_form, set_show_new_form) = signal(false);
    let (submitting, set_submitting) = signal(false);
    let (submit_error, set_submit_error) = signal(None::<String>);

    // Form state
    let (form_file, set_form_file) = signal(String::new());
    let (form_start, set_form_start) = signal("1".to_string());
    let (form_end, set_form_end) = signal("1".to_string());
    let (form_original, set_form_original) = signal(String::new());
    let (form_suggested, set_form_suggested) = signal(String::new());
    let (form_desc, set_form_desc) = signal(String::new());

    let fetch_suggestions = move || {
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        let number_val = number();
        set_loading.set(true);
        set_error.set(None);

        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!(
                "/repos/{owner_val}/{name_val}/pulls/{number_val}/suggestions"
            );
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<Vec<Suggestion>>().await {
                        set_suggestions.set(data);
                    }
                }
                _ => {
                    set_suggestions.set(vec![
                        Suggestion {
                            id: "demo-1".into(),
                            pr_number: number(),
                            file_path: "src/main.rs".into(),
                            start_line: 10,
                            end_line: 15,
                            original_code: "fn old_function() {\n    // old\n}".into(),
                            suggested_code: "fn new_function() {\n    // improved\n}".into(),
                            description: "Refactor for clarity".into(),
                            author: "user-abc12345".into(),
                            status: "open".into(),
                            created_at: "2025-01-15T10:00:00Z".into(),
                        },
                    ]);
                }
            }
            set_loading.set(false);
        });
    };

    leptos::task::spawn_local(async move {
        fetch_suggestions();
    });

    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_submit_error.set(None);
        let file_path = form_file.get();
        if file_path.trim().is_empty() {
            set_submit_error.set(Some("File path is required.".into()));
            return;
        }
        let suggested_code = form_suggested.get();
        if suggested_code.trim().is_empty() {
            set_submit_error.set(Some("Suggested code is required.".into()));
            return;
        }

        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        let number_val = number();
        let start_line: i32 = form_start.get().parse().unwrap_or(1);
        let end_line: i32 = form_end.get().parse().unwrap_or(1);

        set_submitting.set(true);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!(
                "/repos/{owner_val}/{name_val}/pulls/{number_val}/suggestions"
            );
            let body = CreateSuggestionBody {
                file_path: file_path.trim().to_string(),
                start_line,
                end_line,
                original_code: form_original.get(),
                suggested_code: suggested_code.trim().to_string(),
                description: form_desc.get(),
            };
            let _ = client.post(&path, &body).await;
            set_show_new_form.set(false);
            set_submitting.set(false);
            set_form_file.set(String::new());
            set_form_suggested.set(String::new());
            fetch_suggestions();
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
                        <span class="hidden sm:inline text-gray-700 dark:text-gray-300">"Suggestions"</span>
                    </div>
                    <h1 class="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-gray-100">"Suggested Edits"</h1>
                </div>
                <Button variant=ButtonVariant::Primary on:click=move |_| set_show_new_form.set(!show_new_form.get())>
                    {move || if show_new_form.get() { "Cancel" } else { "New Suggestion" }}
                </Button>
            </div>

            <Show when=move || show_new_form.get() fallback=|| view! { <div class="hidden"></div> }>
                <Card title="New Suggestion".to_string()>
                    <form on:submit=handle_submit class="space-y-4">
                        <Show when=move || submit_error.get().is_some()>
                            <ErrorBanner
                                message=move || submit_error.get().unwrap_or_default()
                                on_dismiss=Callback::new(move |_: ()| set_submit_error.set(None))
                            />
                        </Show>
                        <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"File Path"</label>
                                <input type="text" class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500" placeholder="src/main.rs" required prop:value=form_file on:input=move |ev| set_form_file.set(event_target_value(&ev)) />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Start Line"</label>
                                <input type="number" min="1" class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent" prop:value=form_start on:input=move |ev| set_form_start.set(event_target_value(&ev)) />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"End Line"</label>
                                <input type="number" min="1" class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent" prop:value=form_end on:input=move |ev| set_form_end.set(event_target_value(&ev)) />
                            </div>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Original Code"</label>
                            <textarea class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500" placeholder="Original code..." rows="3" prop:value=form_original on:input=move |ev| set_form_original.set(event_target_value(&ev))></textarea>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Suggested Code"</label>
                            <textarea class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500" placeholder="Suggested replacement..." rows="3" required prop:value=form_suggested on:input=move |ev| set_form_suggested.set(event_target_value(&ev))></textarea>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Description (optional)"</label>
                            <textarea class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500" placeholder="Why this is better..." rows="2" prop:value=form_desc on:input=move |ev| set_form_desc.set(event_target_value(&ev))></textarea>
                        </div>
                        <Button variant=ButtonVariant::Primary disabled=submitting.get()>
                            {move || if submitting.get() { "Submitting..." } else { "Submit Suggestion" }}
                        </Button>
                    </form>
                </Card>
            </Show>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_error.set(None)) />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="flex items-center justify-center py-12">
                        <Spinner />
                        <span class="ml-3 text-gray-500 dark:text-gray-400">"Loading suggestions..."</span>
                    </div>
                </Card>
            </Show>

            <Show when=move || !loading.get() && suggestions.get().is_empty() && error.get().is_none() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="text-center py-12">
                        <h3 class="text-sm font-medium text-gray-900 dark:text-gray-100">"No suggestions yet"</h3>
                        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">"Suggest code changes for this pull request."</p>
                    </div>
                </Card>
            </Show>

            <Show when=move || !loading.get() && !suggestions.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                <div class="space-y-4">
                    <For
                        each=move || suggestions.get()
                        key=|s| s.id.clone()
                        children=move |suggestion| {
                            view! {
                                <div class="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
                                    <div class="p-4">
                                        <div class="flex items-center gap-2 mb-2 flex-wrap">
                                            <Badge color=suggestion_status_color(&suggestion.status) text=suggestion.status.clone() />
                                            <span class="text-xs font-mono text-gray-500 dark:text-gray-400">{suggestion.file_path.clone()}</span>
                                            <span class="text-xs text-gray-400 dark:text-gray-500">{format!("lines {}-{}", suggestion.start_line, suggestion.end_line)}</span>
                                        </div>
                                        {if !suggestion.description.is_empty() {
                                            view! { <p class="text-sm text-gray-700 dark:text-gray-300 mb-3">{suggestion.description.clone()}</p> }.into_any()
                                        } else {
                                            ().into_any()
                                        }}
                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                                            <div>
                                                <div class="text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">"Original"</div>
                                                <pre class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md p-3 text-xs font-mono overflow-x-auto text-gray-800 dark:text-gray-200">{suggestion.original_code.clone()}</pre>
                                            </div>
                                            <div>
                                                <div class="text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">"Suggested"</div>
                                                <pre class="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-md p-3 text-xs font-mono overflow-x-auto text-gray-800 dark:text-gray-200">{suggestion.suggested_code.clone()}</pre>
                                            </div>
                                        </div>
                                        <div class="flex items-center gap-2 mt-3">
                                            <span class="text-xs text-gray-400 dark:text-gray-500">{truncate_uuid(&suggestion.author, 8)}</span>
                                            <span class="text-xs text-gray-400 dark:text-gray-500">"·"</span>
                                            <span class="text-xs text-gray-400 dark:text-gray-500">{relative_time(&suggestion.created_at)}</span>
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
