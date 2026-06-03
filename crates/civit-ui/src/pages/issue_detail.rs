#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;
#[cfg(feature = "csr")]
use wasm_bindgen::JsCast;

use crate::api::client::ApiClient;
use crate::api::types::{CreateCommentBody, IssueResponse, UpdateIssueBody};
use crate::components::{Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Spinner};
use crate::state::auth::use_auth;

fn relative_time(created_at: &str) -> String {
    #[cfg(feature = "csr")]
    {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(created_at) {
            let now = chrono::Utc::now();
            let diff = now.signed_duration_since(dt);
            if diff.num_seconds() < 60 {
                return "just now".to_string();
            } else if diff.num_minutes() < 60 {
                return format!("{}m ago", diff.num_minutes());
            } else if diff.num_hours() < 24 {
                return format!("{}h ago", diff.num_hours());
            } else if diff.num_days() < 30 {
                return format!("{}d ago", diff.num_days());
            } else {
                return dt.format("%b %d, %Y").to_string();
            }
        }
    }
    created_at.to_string()
}

fn truncate_uuid(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len])
    } else {
        s.to_string()
    }
}

fn status_badge_color(state: &str) -> BadgeColor {
    match state {
        "open" => BadgeColor::Success,
        "in_progress" => BadgeColor::Info,
        "closed" => BadgeColor::Neutral,
        _ => BadgeColor::Neutral,
    }
}

fn status_label(state: &str) -> String {
    match state {
        "in_progress" => "In Progress".to_string(),
        s => {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        }
    }
}

fn get_input_value(name: &str) -> String {
    #[cfg(feature = "csr")]
    {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return String::new(),
        };
        let doc = match window.document() {
            Some(d) => d,
            None => return String::new(),
        };
        let el = match doc.get_element_by_id(name) {
            Some(el) => el,
            None => return String::new(),
        };
        let tag = el.tag_name().to_lowercase();
        if tag == "textarea" {
            match el.dyn_into::<web_sys::HtmlTextAreaElement>() {
                Ok(ta) => return ta.value(),
                Err(_) => return String::new(),
            }
        }
        match el.dyn_into::<web_sys::HtmlInputElement>() {
            Ok(input) => input.value(),
            Err(_) => String::new(),
        }
    }
    #[cfg(not(feature = "csr"))]
    {
        let _ = name;
        String::new()
    }
}

#[component]
pub fn IssueDetailPage() -> impl IntoView {
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

    let (issue_sig, set_issue) = signal(None::<IssueResponse>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let (editing, set_editing) = signal(false);
    let (edit_saving, set_edit_saving) = signal(false);
    let (edit_error, set_edit_error) = signal(None::<String>);

    let (comment_body, set_comment_body) = signal(String::new());
    let (comment_saving, set_comment_saving) = signal(false);
    let (comment_error, set_comment_error) = signal(None::<String>);

    let (action_loading, set_action_loading) = signal(false);

    let fetch_issue = move || {
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        let number_val = number();
        let _auth_signal = auth.0;

        set_loading.set(true);
        set_error.set(None);

        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/issues/{number_val}");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<IssueResponse>().await {
                        Ok(data) => {
                            set_issue.set(Some(data));
                        }
                        Err(_) => set_error.set(Some("Failed to process response.".to_string())),
                    }
                }
                Ok(_) => {
                    set_error.set(Some("Failed to load issue.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_loading.set(false);
        });
    };

    leptos::task::spawn_local(async move {
        fetch_issue();
    });

    let handle_edit_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_edit_error.set(None);

        let title_val = get_input_value("edit-issue-title");
        let body_val = get_input_value("edit-issue-body");
        let owner_val = owner();
        let name_val = name();
        let number_val = number();
        let token = auth.0.with(|a| a.token.clone());

        if title_val.trim().is_empty() {
            set_edit_error.set(Some("Title is required.".to_string()));
            return;
        }

        let update_body = UpdateIssueBody {
            title: Some(title_val.trim().to_string()),
            body: Some(body_val),
            state: None,
        };

        set_edit_saving.set(true);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/issues/{number_val}");
            match client.put(&path, &update_body).await {
                Ok(resp) if resp.status().is_success() => {
                    set_editing.set(false);
                }
                Ok(_) => {
                    set_edit_error.set(Some("Failed to update issue.".to_string()));
                }
                Err(_) => {
                    set_edit_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_edit_saving.set(false);
        });
    };

    let handle_state_change = move |new_state: &str| {
        let owner_val = owner();
        let name_val = name();
        let number_val = number();
        let token = auth.0.with(|a| a.token.clone());
        let state_str = new_state.to_string();

        set_action_loading.set(true);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/issues/{number_val}");
            let body = UpdateIssueBody {
                title: None,
                body: None,
                state: Some(state_str),
            };
            match client.put(&path, &body).await {
                Ok(resp) if resp.status().is_success() => {
                    let client2 = ApiClient::new(auth.0.with(|a| a.token.clone()));
                    match client2.get(&path).await {
                        Ok(r) if r.status().is_success() => {
                            if let Ok(data) = r.json::<IssueResponse>().await {
                                set_issue.set(Some(data));
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            set_action_loading.set(false);
        });
    };

    let handle_comment_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_comment_error.set(None);

        let body_val = comment_body.get();
        if body_val.trim().is_empty() {
            set_comment_error.set(Some("Comment cannot be empty.".to_string()));
            return;
        }

        let owner_val = owner();
        let name_val = name();
        let number_val = number();
        let token = auth.0.with(|a| a.token.clone());
        let comment = CreateCommentBody {
            body: body_val.trim().to_string(),
        };

        set_comment_saving.set(true);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/issues/{number_val}/comments");
            match client.post(&path, &comment).await {
                Ok(resp) if resp.status().is_success() => {
                    set_comment_body.set(String::new());
                    let client2 = ApiClient::new(auth.0.with(|a| a.token.clone()));
                    let issue_path = format!("/repos/{owner_val}/{name_val}/issues/{number_val}");
                    match client2.get(&issue_path).await {
                        Ok(r) if r.status().is_success() => {
                            if let Ok(data) = r.json::<IssueResponse>().await {
                                set_issue.set(Some(data));
                            }
                        }
                        _ => {}
                    }
                }
                Ok(_) => {
                    set_comment_error.set(Some("Failed to add comment.".to_string()));
                }
                Err(_) => {
                    set_comment_error
                        .set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_comment_saving.set(false);
        });
    };

    let is_open = move || issue_sig.get().is_some_and(|i| i.state == "open");
    let is_in_progress = move || issue_sig.get().is_some_and(|i| i.state == "in_progress");

    view! {
        <div class="space-y-6">
            <div class="text-sm text-gray-500 dark:text-gray-400 mb-1">
                <A href="/repos">
                    <span class="hover:text-blue-600 dark:hover:text-blue-400">"Repositories"</span>
                </A>
                <span>" / "</span>
                <A href=format!("/repos/{}/{}", owner(), name())>
                    <span class="hover:text-blue-600 dark:hover:text-blue-400">
                        {move || format!("{}/{}", owner(), name())}
                    </span>
                </A>
                <span>" / "</span>
                <span class="text-gray-700 dark:text-gray-300">"Issues"</span>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_error.set(None)) />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="flex items-center justify-center py-12">
                        <Spinner />
                        <span class="ml-3 text-gray-500 dark:text-gray-400">"Loading issue..."</span>
                    </div>
                </Card>
            </Show>

            <Show when=move || !loading.get() && issue_sig.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="space-y-4">
                        <div class="flex items-start justify-between flex-wrap gap-3">
                            <div class="space-y-1">
                                <div class="flex items-center gap-3 flex-wrap">
                                    <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                                        {move || issue_sig.get().map(|i| i.title.clone()).unwrap_or_default()}
                                    </h1>
                                    {
                                        let state_val = issue_sig.get().map(|i| i.state.clone()).unwrap_or_default();
                                        let badge_color = status_badge_color(&state_val);
                                        let badge_text = status_label(&state_val);
                                        view! {
                                            <Badge color=badge_color text=badge_text />
                                        }
                                    }
                                    <span class="text-sm text-gray-400 dark:text-gray-500 font-mono">
                                        {move || format!("#{}", issue_sig.get().map(|i| i.number.unwrap_or(i.id)).unwrap_or(0))}
                                    </span>
                                </div>
                                <div class="flex items-center gap-3 text-sm text-gray-500 dark:text-gray-400">
                                    <span>
                                        {move || truncate_uuid(&issue_sig.get().map(|i| i.author.clone()).unwrap_or_default(), 8)}
                                    </span>
                                    <span>"opened this issue"</span>
                                    <span>
                                        {move || relative_time(&issue_sig.get().map(|i| i.created_at.clone()).unwrap_or_default())}
                                    </span>
                                </div>
                            </div>
                            <div class="flex items-center gap-2">
                                <Button
                                    variant=ButtonVariant::Secondary
                                    on:click=move |_| set_editing.set(!editing.get())
                                >
                                    {move || if editing.get() { "Cancel" } else { "Edit" }}
                                </Button>
                                <Show when=move || is_open() && !is_in_progress() && !action_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                                    <Button
                                        variant=ButtonVariant::Secondary
                                        on:click=move |_| handle_state_change("in_progress")
                                    >
                                        "Start Progress"
                                    </Button>
                                </Show>
                                <Show when=move || (is_open() || is_in_progress()) && !action_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                                    <Button
                                        variant=ButtonVariant::Danger
                                        on:click=move |_| handle_state_change("closed")
                                    >
                                        "Close Issue"
                                    </Button>
                                </Show>
                                <Show when=move || issue_sig.get().is_some_and(|i| i.state == "closed") && !action_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                                    <Button
                                        variant=ButtonVariant::Primary
                                        on:click=move |_| handle_state_change("open")
                                    >
                                        "Reopen Issue"
                                    </Button>
                                </Show>
                            </div>
                        </div>

                        <Show when=move || editing.get() fallback=|| view! { <div class="hidden"></div> }>
                            <div class="border-t border-gray-200 dark:border-gray-700 pt-4 space-y-3">
                                <Show when=move || edit_error.get().is_some()>
                                    <ErrorBanner message=move || edit_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_edit_error.set(None)) />
                                </Show>
                                <form on:submit=handle_edit_submit class="space-y-3">
                                    <div>
                                        <label for="edit-issue-title" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                            "Title"
                                        </label>
                                        <input
                                            id="edit-issue-title"
                                            type="text"
                                            class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                            value=move || issue_sig.get().map(|i| i.title.clone()).unwrap_or_default()
                                            required
                                        />
                                    </div>
                                    <div>
                                        <label for="edit-issue-body" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                            "Body"
                                        </label>
                                        <textarea
                                            id="edit-issue-body"
                                            class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                            rows="6"
                                        >
                                            {move || issue_sig.get().and_then(|i| i.body.clone()).unwrap_or_default()}
                                        </textarea>
                                    </div>
                                    <Button variant=ButtonVariant::Primary disabled=edit_saving.get()>
                                        {move || if edit_saving.get() { "Saving..." } else { "Save Changes" }}
                                    </Button>
                                </form>
                            </div>
                        </Show>

                        <Show when=move || !editing.get() && issue_sig.get().is_some_and(|i| i.body.as_ref().is_some_and(|b| !b.is_empty())) fallback=|| view! { <div class="hidden"></div> }>
                            <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
                                <div class="whitespace-pre-wrap text-sm text-gray-700 dark:text-gray-300 leading-relaxed">
                                    {move || issue_sig.get().and_then(|i| i.body.clone()).unwrap_or_default()}
                                </div>
                            </div>
                        </Show>
                    </div>
                </Card>

                <div class="mt-6 space-y-4">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
                        "Comments"
                    </h2>

                    <Card>
                        <form on:submit=handle_comment_submit class="space-y-3">
                            <Show when=move || comment_error.get().is_some()>
                                <ErrorBanner message=move || comment_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_comment_error.set(None)) />
                            </Show>
                            <label for="comment-input" class="sr-only">"Write a comment"</label>
                            <textarea
                                id="comment-input"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500"
                                placeholder="Write a comment..."
                                rows="3"
                                on:input=move |ev| {
                                    let val = event_target_value(&ev);
                                    set_comment_body.set(val);
                                }
                                prop:value=comment_body.get()
                            ></textarea>
                            <Button variant=ButtonVariant::Primary disabled=comment_saving.get() || comment_body.get().trim().is_empty()>
                                {move || if comment_saving.get() { "Posting..." } else { "Add Comment" }}
                            </Button>
                        </form>
                    </Card>

                    <For each=move || issue_sig.get().map(|i| i.comments.clone()).unwrap_or_default() key=|c| c.id let:comment>
                        {
                            let is_edited = comment.updated_at != comment.created_at;
                            view! {
                                <Card class="mt-3".to_string()>
                                    <div class="space-y-2">
                                        <div class="flex items-center gap-2 flex-wrap">
                                            <span class="text-sm font-mono text-gray-500 dark:text-gray-400">
                                                {truncate_uuid(&comment.author, 8)}
                                            </span>
                                            <span class="text-xs text-gray-400 dark:text-gray-500">
                                                {relative_time(&comment.created_at)}
                                            </span>
                                            {is_edited.then(|| view! {
                                                <Badge color=BadgeColor::Info text="Edited".to_string() />
                                            })}
                                        </div>
                                        <div class="whitespace-pre-wrap text-sm text-gray-700 dark:text-gray-300 leading-relaxed">
                                            {comment.body.clone()}
                                        </div>
                                    </div>
                                </Card>
                            }
                        }
                    </For>
                </div>
            </Show>
        </div>
    }
}
