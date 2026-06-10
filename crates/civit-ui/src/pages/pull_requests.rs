#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::api::types::{CreatePullRequestBody, PullRequestListResponse};
use crate::components::{Badge, Button, ButtonVariant, Card, ErrorBanner, Spinner, TabItem, Tabs};
use crate::state::auth::use_auth;
use crate::utils::*;

fn truncate_title(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len])
    } else {
        s.to_string()
    }
}

fn pr_badge_color(status: &str) -> crate::components::badge::BadgeColor {
    match status {
        "open" => crate::components::badge::BadgeColor::Success,
        "merged" => crate::components::badge::BadgeColor::Info,
        "closed" => crate::components::badge::BadgeColor::Neutral,
        _ => crate::components::badge::BadgeColor::Neutral,
    }
}

fn pr_status_label(status: &str) -> String {
    match status {
        "open" => "Open".into(),
        "merged" => "Merged".into(),
        "closed" => "Closed".into(),
        s if !s.is_empty() => {
            let mut c = s.chars();
            match c.next() {
                Some(f) => format!("{}{}", f.to_ascii_uppercase(), c.as_str()),
                None => String::new(),
            }
        }
        _ => String::new(),
    }
}

#[component]
pub fn PullRequestsPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let auth = use_auth();

    let (page, set_page) = signal(1u32);
    let (filter, set_filter) = signal("all".to_string());
    let (prs_sig, set_prs) = signal(None::<PullRequestListResponse>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (show_new_form, set_show_new_form) = signal(false);
    let (submitting, set_submitting) = signal(false);
    let (submit_error, set_submit_error) = signal(None::<String>);

    let fetch_prs = move || {
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
                format!("/repos/{owner_val}/{name_val}/pulls?per_page=20&page={page_val}");
            if filter_val != "all" {
                path.push_str(&format!("&state={filter_val}"));
            }
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<PullRequestListResponse>().await {
                        Ok(data) => set_prs.set(Some(data)),
                        Err(_) => set_error.set(Some("Failed to process response.".into())),
                    }
                }
                Ok(_) => set_error.set(Some("Failed to load pull requests.".into())),
                Err(_) => set_error.set(Some("Network error.".into())),
            }
            set_loading.set(false);
        });
    };

    leptos::task::spawn_local(async move {
        fetch_prs();
    });

    let _handle_page_change = move |new_page: u32| {
        set_page.set(new_page);
        fetch_prs();
    };

    let handle_filter_change = move |new_filter: String| {
        set_page.set(1);
        set_filter.set(new_filter);
        fetch_prs();
    };

    let handle_new_pr_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_submit_error.set(None);

        let title_val = get_input_value("new-pr-title");
        let body_val = get_input_value("new-pr-body");
        let source_val = get_input_value("new-pr-source");
        let target_val = get_input_value("new-pr-target");

        if title_val.trim().is_empty() {
            set_submit_error.set(Some("Title is required.".into()));
            return;
        }
        if source_val.trim().is_empty() {
            set_submit_error.set(Some("Source branch is required.".into()));
            return;
        }
        if target_val.trim().is_empty() {
            set_submit_error.set(Some("Target branch is required.".into()));
            return;
        }

        let body = CreatePullRequestBody {
            title: title_val.trim().to_string(),
            body: if body_val.trim().is_empty() {
                None
            } else {
                Some(body_val.trim().to_string())
            },
            source_branch: source_val.trim().to_string(),
            target_branch: target_val.trim().to_string(),
            draft: None,
            assignees: Vec::new(),
            reviewers: Vec::new(),
            labels: Vec::new(),
        };

        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();

        set_submitting.set(true);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/pulls");
            match client.post(&path, &body).await {
                Ok(resp) if resp.status().is_success() => {
                    set_show_new_form.set(false);
                    set_page.set(1);
                }
                Ok(resp) => {
                    let status = resp.status();
                    let text = resp.text().await.unwrap_or_default();
                    set_submit_error.set(Some(format!("Create failed ({status}): {text}")));
                }
                Err(_) => set_submit_error.set(Some("Network error.".into())),
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
            id: "closed".into(),
            label: "Closed".into(),
        },
        TabItem {
            id: "merged".into(),
            label: "Merged".into(),
        },
    ]);

    let has_prs = move || prs_sig.get().is_some_and(|d| !d.items.is_empty());

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
                        <span class="text-gray-700 dark:text-gray-300">"Pull Requests"</span>
                    </div>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Pull Requests"</h1>
                </div>
                <Button variant=ButtonVariant::Primary on:click=move |_| set_show_new_form.set(!show_new_form.get())>
                    {move || if show_new_form.get() { "Cancel" } else { "New Pull Request" }}
                </Button>
            </div>

            <Show when=move || show_new_form.get() fallback=|| view! { <div class="hidden"></div> }>
                <Card title="New Pull Request".to_string()>
                    <form on:submit=handle_new_pr_submit class="space-y-4">
                        <Show when=move || submit_error.get().is_some()>
                            <ErrorBanner message=move || submit_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_submit_error.set(None)) />
                        </Show>
                        <div>
                            <label for="new-pr-title" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Title"</label>
                            <input id="new-pr-title" type="text" class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent" placeholder="PR title" required />
                        </div>
                        <div>
                            <label for="new-pr-body" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Description"</label>
                            <textarea id="new-pr-body" class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent" placeholder="Describe changes..." rows="4"></textarea>
                        </div>
                        <div class="grid grid-cols-2 gap-4">
                            <div>
                                <label for="new-pr-source" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Source Branch"</label>
                                <input id="new-pr-source" type="text" class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent" placeholder="feature-branch" required />
                            </div>
                            <div>
                                <label for="new-pr-target" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Target Branch"</label>
                                <input id="new-pr-target" type="text" class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent" placeholder="main" required />
                            </div>
                        </div>
                        <Button variant=ButtonVariant::Primary disabled=submitting.get()>
                            {move || if submitting.get() { "Creating..." } else { "Create Pull Request" }}
                        </Button>
                    </form>
                </Card>
            </Show>

            <Tabs tabs=filter_tabs.get() active_tab=filter.get() on_change=Callback::new(move |id: String| handle_filter_change(id))>
                <div class="hidden"></div>
            </Tabs>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_error.set(None)) />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="flex items-center justify-center py-12">
                        <Spinner />
                        <span class="ml-3 text-gray-500">"Loading pull requests..."</span>
                    </div>
                </Card>
            </Show>

            <Show when=move || !loading.get() && !has_prs() && error.get().is_none() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="text-center py-12">
                        <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-gray-100">"No pull requests yet"</h3>
                        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">"Create the first pull request for this repository."</p>
                    </div>
                </Card>
            </Show>

            <Show when=has_prs fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="divide-y divide-gray-100 dark:divide-gray-700 overflow-x-auto">
                        <For each=move || prs_sig.get().map(|d| d.items.clone()).unwrap_or_default() key=|pr| pr.id.clone() let:pr>
                            {
                                let owner_v = owner();
                                let name_v = name();
                                view! {
                                    <A href=format!("/repos/{owner_v}/{name_v}/pulls/{}", pr.number)>
                                        <div class="flex items-center justify-between py-3 px-1 hover:bg-gray-50 dark:hover:bg-gray-750 -mx-1 rounded transition-colors cursor-pointer">
                                            <div class="flex items-center gap-3 min-w-0">
                                                <span class="text-sm font-mono text-gray-400 shrink-0">
                                                    {format!("#{}", pr.number)}
                                                </span>
                                                <span class="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                                                    {truncate_title(&pr.title, 80)}
                                                </span>
                                                <Badge color=pr_badge_color(&pr.status) text=pr_status_label(&pr.status) />
                                                {pr.draft.then(|| view! { <Badge color=crate::components::badge::BadgeColor::Neutral text="Draft".into() /> })}
                                            </div>
                                            <div class="flex items-center gap-3 shrink-0">
                                                <span class="text-xs text-gray-400 font-mono hidden sm:inline-flex">
                                                    {truncate_uuid(&pr.source_branch, 20)}
                                                </span>
                                                <span class="text-gray-400">"\u{2192}"</span>
                                                <span class="text-xs text-gray-400 font-mono hidden sm:inline-flex">
                                                    {truncate_uuid(&pr.target_branch, 20)}
                                                </span>
                                                <span class="text-xs text-gray-400 hidden sm:inline-flex">
                                                    {relative_time(&pr.created_at)}
                                                </span>
                                            </div>
                                        </div>
                                    </A>
                                }
                            }
                        </For>
                    </div>
                </Card>
            </Show>
        </div>
    }
}
