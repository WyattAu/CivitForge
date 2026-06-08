#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::api::types::{CreateIssueBody, IssueResponse, ListResponse};
use crate::components::{
    Badge, Button, ButtonVariant, Card, ErrorBanner, Pagination, Spinner, TabItem, Tabs,
};
use crate::state::auth::use_auth;
use crate::utils::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::badge::BadgeColor;

    #[test]
    fn truncate_uuid_short() {
        assert_eq!(truncate_uuid("abc", 8), "abc");
    }

    #[test]
    fn truncate_uuid_exact() {
        assert_eq!(truncate_uuid("abcdefgh", 8), "abcdefgh");
    }

    #[test]
    fn truncate_uuid_long() {
        assert_eq!(truncate_uuid("abcdefghijk", 8), "abcdefgh...");
    }

    #[test]
    fn truncate_uuid_zero_max() {
        assert_eq!(truncate_uuid("anything", 0), "...");
    }

    #[test]
    fn relative_time_non_csr_passthrough() {
        assert_eq!(relative_time("some-bad-format"), "some-bad-format");
    }

    #[test]
    fn status_badge_color_known_states() {
        assert_eq!(status_badge_color("open"), BadgeColor::Success);
        assert_eq!(status_badge_color("in_progress"), BadgeColor::Info);
        assert_eq!(status_badge_color("closed"), BadgeColor::Neutral);
    }

    #[test]
    fn status_badge_color_unknown() {
        assert_eq!(status_badge_color("merged"), BadgeColor::Neutral);
        assert_eq!(status_badge_color(""), BadgeColor::Neutral);
    }

    #[test]
    fn status_label_capitalize() {
        assert_eq!(status_label("open"), "Open");
        assert_eq!(status_label("closed"), "Closed");
    }

    #[test]
    fn status_label_in_progress_special() {
        assert_eq!(status_label("in_progress"), "In Progress");
    }

    #[test]
    fn status_label_empty() {
        assert_eq!(status_label(""), "");
    }

    #[test]
    fn truncate_title_short() {
        assert_eq!(truncate_title("short", 80), "short");
    }

    #[test]
    fn truncate_title_long() {
        assert_eq!(
            truncate_title("a very long title that exceeds the max", 10),
            "a very lon..."
        );
    }
}

fn truncate_title(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len])
    } else {
        s.to_string()
    }
}

#[component]
pub fn IssuesPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let auth = use_auth();

    let (page, set_page) = signal(1u32);
    let (filter, set_filter) = signal("all".to_string());
    let (issues_sig, set_issues) = signal(None::<ListResponse<IssueResponse>>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (show_new_form, set_show_new_form) = signal(false);
    let (submitting, set_submitting) = signal(false);
    let (submit_error, set_submit_error) = signal(None::<String>);

    let fetch_issues = move || {
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        let page_val = page.get();
        let filter_val = filter.get();
        let _auth_signal = auth.0;

        set_loading.set(true);
        set_error.set(None);

        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let mut path =
                format!("/repos/{owner_val}/{name_val}/issues?per_page=20&page={page_val}");
            if filter_val != "all" {
                path.push_str(&format!("&state={filter_val}"));
            }
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<ListResponse<IssueResponse>>().await {
                        Ok(data) => {
                            set_issues.set(Some(data));
                        }
                        Err(_) => set_error.set(Some("Failed to process response.".to_string())),
                    }
                }
                Ok(_) => {
                    set_error.set(Some("Failed to load issues.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_loading.set(false);
        });
    };

    leptos::task::spawn_local(async move {
        fetch_issues();
    });

    let handle_page_change = move |new_page: u32| {
        set_page.set(new_page);
        fetch_issues();
    };

    let handle_filter_change = move |new_filter: String| {
        set_filter.set(new_filter);
        set_page.set(1);
        fetch_issues();
    };

    let handle_new_issue_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_submit_error.set(None);

        let title_val = get_input_value("new-issue-title");
        let desc_val = get_input_value("new-issue-description");

        if title_val.trim().is_empty() {
            set_submit_error.set(Some("Title is required.".to_string()));
            return;
        }

        let body = CreateIssueBody {
            title: title_val.trim().to_string(),
            description: if desc_val.trim().is_empty() {
                None
            } else {
                Some(desc_val.trim().to_string())
            },
        };

        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();

        set_submitting.set(true);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/issues");
            match client.post(&path, &body).await {
                Ok(resp) if resp.status().is_success() => {
                    set_show_new_form.set(false);
                    set_page.set(1);
                    set_filter.set("all".to_string());
                }
                Ok(_) => {
                    set_submit_error.set(Some("Failed to create issue.".to_string()));
                }
                Err(_) => {
                    set_submit_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_submitting.set(false);
        });
    };

    let (filter_tabs, _) = signal(vec![
        TabItem {
            id: "all".into(),
            label: "All".into(),
        },
        TabItem {
            id: "open".into(),
            label: "Open".into(),
        },
        TabItem {
            id: "in_progress".into(),
            label: "In Progress".into(),
        },
        TabItem {
            id: "closed".into(),
            label: "Closed".into(),
        },
    ]);

    let has_issues = move || issues_sig.get().is_some_and(|data| !data.data.is_empty());

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
                        <span>"/"</span>
                        <span class="text-gray-700 dark:text-gray-300">"Issues"</span>
                    </div>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Issues"</h1>
                </div>
                <Button
                    variant=ButtonVariant::Primary
                    extra_class="btn-new-issue"
                    on:click=move |_| set_show_new_form.set(!show_new_form.get())
                >
                    {move || if show_new_form.get() { "Cancel" } else { "New Issue" }}
                </Button>
            </div>

            <Show when=move || show_new_form.get() fallback=|| view! { <div class="hidden"></div> }>
                <Card title="New Issue">
                    <form on:submit=handle_new_issue_submit class="space-y-4">
                        <Show when=move || submit_error.get().is_some()>
                            <ErrorBanner message=move || submit_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_submit_error.set(None)) />
                        </Show>
                        <div>
                            <label for="new-issue-title" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Title"
                            </label>
                            <input
                                id="new-issue-title"
                                type="text"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500"
                                placeholder="Issue title"
                                required
                            />
                        </div>
                        <div>
                            <label for="new-issue-description" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Description"
                            </label>
                            <textarea
                                id="new-issue-description"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500"
                                placeholder="Describe the issue..."
                                rows="4"
                            ></textarea>
                        </div>
                        <div class="flex gap-3">
                            <Button variant=ButtonVariant::Primary disabled=submitting.get()>
                                {move || if submitting.get() { "Creating..." } else { "Submit Issue" }}
                            </Button>
                        </div>
                    </form>
                </Card>
            </Show>

            <Tabs
                tabs=filter_tabs.get()
                active_tab=filter.get()
                on_change=Callback::new(move |id: String| handle_filter_change(id))
            >
                <div class="hidden"></div>
            </Tabs>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_error.set(None)) />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="flex items-center justify-center py-12">
                        <Spinner />
                        <span class="ml-3 text-gray-500 dark:text-gray-400">"Loading issues..."</span>
                    </div>
                </Card>
            </Show>

            <Show when=move || !loading.get() && !has_issues() && error.get().is_none() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="text-center py-12">
                        <svg class="mx-auto h-12 w-12 text-gray-300 dark:text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                        </svg>
                        <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-gray-100">"No issues yet"</h3>
                        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">"Create the first issue for this repository."</p>
                    </div>
                </Card>
            </Show>

            <Show when=has_issues fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="divide-y divide-gray-100 dark:divide-gray-700 overflow-x-auto">
                        <For each=move || issues_sig.get().map(|d| d.data.clone()).unwrap_or_default() key=|i| i.id.clone() let:issue>
                            {
                                let owner_v = owner();
                                let name_v = name();
                                view! {
                                    <A
                                        href=format!("/repos/{owner_v}/{name_v}/issues/{}", issue.number.map(|n| n.to_string()).unwrap_or_else(|| issue.id.clone()))
                                    >
                                        <div class="flex items-center justify-between py-3 px-1 hover:bg-gray-50 dark:hover:bg-gray-750 -mx-1 rounded transition-colors cursor-pointer">
                                            <div class="flex items-center gap-3 min-w-0">
                                                <span class="text-sm font-mono text-gray-400 dark:text-gray-500 shrink-0">
                                                     {format!("#{}", issue.number.map(|n| n.to_string()).unwrap_or_else(|| issue.id.clone()))}
                                                </span>
                                                <span class="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                                                    {truncate_title(&issue.title, 80)}
                                                </span>
                                                <Badge
                                                    color=status_badge_color(&issue.state)
                                                    text=status_label(&issue.state)
                                                />
                                            </div>
                                            <div class="flex items-center gap-3 shrink-0">
                                                <span class="text-xs text-gray-400 dark:text-gray-500 font-mono hidden sm:inline-flex">
                                                    {truncate_uuid(&issue.author, 8)}
                                                </span>
                                                <span class="text-xs text-gray-400 dark:text-gray-500 hidden sm:inline-flex">
                                                    {relative_time(&issue.created_at)}
                                                </span>
                                            </div>
                                        </div>
                                    </A>
                                }
                            }
                        </For>
                    </div>
                </Card>

                {
                    move || {
                        let tp = issues_sig.get().map(|d| d.pagination.total_pages).unwrap_or(1);
                        view! {
                            <Pagination
                                current_page=page.get()
                                total_pages=tp
                                on_page_change=Callback::new(move |p: u32| handle_page_change(p))
                            />
                        }
                    }
                }
            </Show>
        </div>
    }
}
