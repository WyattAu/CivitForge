#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::api::types::{
    CreatePrCommentBody, InlineDiffResponse, MergePullRequestBody, MergeResponse,
    MergeabilityResponse, PrDiffResponse, PullRequestResponse, UpdatePullRequestBody,
};
use crate::components::{Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Spinner};
use crate::state::auth::use_auth;
use crate::utils::*;

fn pr_badge_color(status: &str) -> BadgeColor {
    match status {
        "open" => BadgeColor::Success,
        "merged" => BadgeColor::Info,
        _ => BadgeColor::Neutral,
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
pub fn PullRequestDetailPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let number = move || {
        params.with(|p| {
            p.get("number")
                .and_then(|n| n.parse::<i32>().ok())
                .unwrap_or(0)
        })
    };
    let auth = use_auth();

    let (pr_sig, set_pr) = signal(None::<PullRequestResponse>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let (comment_body, set_comment_body) = signal(String::new());
    let (comment_saving, set_comment_saving) = signal(false);
    let (comment_error, set_comment_error) = signal(None::<String>);

    let (merging, set_merging) = signal(false);
    let (merge_result, set_merge_result) = signal(None::<MergeResponse>);

    let (mergeability, set_mergeability) = signal(None::<MergeabilityResponse>);
    let (diff_data, set_diff_data) = signal(None::<PrDiffResponse>);
    let (inline_diff, set_inline_diff) = signal(None::<InlineDiffResponse>);
    let (merge_strategy, set_merge_strategy) = signal(String::from("merge"));

    let (active_tab, set_active_tab) = signal(String::from("conversation"));
    let (diff_view_mode, set_diff_view_mode) = signal(String::from("unified"));

    let (action_loading, set_action_loading) = signal(false);

    let fetch_pr = move || {
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        let number_val = number();
        let _auth_signal = auth.0;

        set_loading.set(true);
        set_error.set(None);

        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/pulls/{number_val}");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<PullRequestResponse>().await {
                        Ok(data) => {
                            set_pr.set(Some(data));

                            // Fetch mergeability and diff in parallel
                            let token2 = auth.0.with(|a| a.token.clone());
                            let owner_mc = owner_val.clone();
                            let name_mc = name_val.clone();
                            let number_mc = number_val;
                            leptos::task::spawn_local(async move {
                                let client = ApiClient::new(token2);
                                let mc_path = format!(
                                    "/repos/{owner_mc}/{name_mc}/pulls/{number_mc}/mergecheck"
                                );
                                if let Ok(r) = client.get(&mc_path).await {
                                    if r.status().is_success() {
                                        if let Ok(m) = r.json::<MergeabilityResponse>().await {
                                            set_mergeability.set(Some(m));
                                        }
                                    }
                                }
                            });
                            let token3 = auth.0.with(|a| a.token.clone());
                            let owner_dc = owner_val.clone();
                            let name_dc = name_val.clone();
                            let number_dc = number_val;
                            leptos::task::spawn_local(async move {
                                let client = ApiClient::new(token3);
                                let dc_path =
                                    format!("/repos/{owner_dc}/{name_dc}/pulls/{number_dc}/diff");
                                if let Ok(r) = client.get(&dc_path).await {
                                    if r.status().is_success() {
                                        if let Ok(d) = r.json::<PrDiffResponse>().await {
                                            set_diff_data.set(Some(d));
                                        }
                                    }
                                }
                            });

                            // Fetch inline diff
                            let token4 = auth.0.with(|a| a.token.clone());
                            let owner_ic = owner_val;
                            let name_ic = name_val;
                            let number_ic = number_val;
                            leptos::task::spawn_local(async move {
                                let client = ApiClient::new(token4);
                                let ic_path = format!(
                                    "/repos/{owner_ic}/{name_ic}/pulls/{number_ic}/diff/inline"
                                );
                                if let Ok(r) = client.get(&ic_path).await {
                                    if r.status().is_success() {
                                        if let Ok(d) = r.json::<InlineDiffResponse>().await {
                                            set_inline_diff.set(Some(d));
                                        }
                                    }
                                }
                            });
                        }
                        Err(_) => set_error.set(Some("Failed to process response.".into())),
                    }
                }
                Ok(_) => set_error.set(Some("Failed to load pull request.".into())),
                Err(_) => set_error.set(Some("Network error.".into())),
            }
            set_loading.set(false);
        });
    };

    leptos::task::spawn_local(async move {
        fetch_pr();
    });

    let handle_close = move |_| {
        let owner_val = owner();
        let name_val = name();
        let number_val = number();
        let token = auth.0.with(|a| a.token.clone());

        set_action_loading.set(true);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/pulls/{number_val}");
            let body = UpdatePullRequestBody {
                title: None,
                body: None,
                state: Some("closed".into()),
                draft: None,
                target_branch: None,
            };
            if client.patch(&path, &body).await.is_ok() {
                let client2 = ApiClient::new(auth.0.with(|a| a.token.clone()));
                if let Ok(r) = client2.get(&path).await {
                    if r.status().is_success() {
                        if let Ok(data) = r.json::<PullRequestResponse>().await {
                            set_pr.set(Some(data));
                        }
                    }
                }
            }
            set_action_loading.set(false);
        });
    };

    let handle_reopen = move |_| {
        let owner_val = owner();
        let name_val = name();
        let number_val = number();
        let token = auth.0.with(|a| a.token.clone());

        set_action_loading.set(true);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/pulls/{number_val}");
            let body = UpdatePullRequestBody {
                title: None,
                body: None,
                state: Some("open".into()),
                draft: None,
                target_branch: None,
            };
            if client.patch(&path, &body).await.is_ok() {
                let client2 = ApiClient::new(auth.0.with(|a| a.token.clone()));
                if let Ok(r) = client2.get(&path).await {
                    if r.status().is_success() {
                        if let Ok(data) = r.json::<PullRequestResponse>().await {
                            set_pr.set(Some(data));
                        }
                    }
                }
            }
            set_action_loading.set(false);
        });
    };

    let handle_merge = move |_| {
        let owner_val = owner();
        let name_val = name();
        let number_val = number();
        let token = auth.0.with(|a| a.token.clone());
        let strategy_val = merge_strategy.get();

        set_merging.set(true);
        set_merge_result.set(None);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/pulls/{number_val}/merge");
            let body = MergePullRequestBody {
                strategy: Some(strategy_val),
            };
            match client.post(&path, &body).await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<MergeResponse>().await {
                        Ok(data) => {
                            set_merge_result.set(Some(data));
                            let client2 = ApiClient::new(auth.0.with(|a| a.token.clone()));
                            let pr_path =
                                format!("/repos/{owner_val}/{name_val}/pulls/{number_val}");
                            match client2.get(&pr_path).await {
                                Ok(r) if r.status().is_success() => {
                                    if let Ok(d) = r.json::<PullRequestResponse>().await {
                                        set_pr.set(Some(d));
                                    }
                                }
                                _ => {}
                            }
                        }
                        Err(_) => {
                            set_merge_result.set(Some(MergeResponse {
                                merged: false,
                                message: "Failed to parse response.".into(),
                                merge_commit_sha: None,
                            }));
                        }
                    }
                }
                Ok(resp) => {
                    let text = resp.text().await.unwrap_or_default();
                    set_merge_result.set(Some(MergeResponse {
                        merged: false,
                        message: format!("Merge failed: {text}"),
                        merge_commit_sha: None,
                    }));
                }
                Err(_) => {
                    set_merge_result.set(Some(MergeResponse {
                        merged: false,
                        message: "Network error.".into(),
                        merge_commit_sha: None,
                    }));
                }
            }
            set_merging.set(false);
        });
    };

    let handle_comment_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_comment_error.set(None);

        let body_val = comment_body.get();
        if body_val.trim().is_empty() {
            set_comment_error.set(Some("Comment cannot be empty.".into()));
            return;
        }

        let owner_val = owner();
        let name_val = name();
        let number_val = number();
        let token = auth.0.with(|a| a.token.clone());
        let comment = CreatePrCommentBody {
            body: body_val.trim().to_string(),
            commit_sha: None,
            file_path: None,
            line: None,
        };

        set_comment_saving.set(true);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/pulls/{number_val}/comments");
            match client.post(&path, &comment).await {
                Ok(resp) if resp.status().is_success() => {
                    set_comment_body.set(String::new());
                }
                Ok(resp) => {
                    let status = resp.status();
                    let text = resp.text().await.unwrap_or_default();
                    set_comment_error.set(Some(format!("Comment failed ({status}): {text}")));
                }
                Err(_) => set_comment_error.set(Some("Network error.".into())),
            }
            set_comment_saving.set(false);
        });
    };

    let is_open = move || pr_sig.get().is_some_and(|p| p.status == "open");

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
                <span class="text-gray-700 dark:text-gray-300">"Pull Requests"</span>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_error.set(None)) />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="flex items-center justify-center py-12">
                        <Spinner />
                        <span class="ml-3 text-gray-500">"Loading pull request..."</span>
                    </div>
                </Card>
            </Show>

            <Show when=move || !loading.get() && pr_sig.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="space-y-4">
                        <div class="flex items-start justify-between flex-wrap gap-3">
                            <div class="space-y-1">
                                <div class="flex items-center gap-3 flex-wrap">
                                    <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                                        {move || pr_sig.get().map(|p| p.title.clone()).unwrap_or_default()}
                                    </h1>
                                    {
                                        let state_val = pr_sig.get().map(|p| p.status.clone()).unwrap_or_default();
                                        view! {
                                            <Badge color=pr_badge_color(&state_val) text=pr_status_label(&state_val) />
                                        }
                                    }
                                    <Show when=move || pr_sig.get().is_some_and(|p| p.draft)>
                                        <Badge color=BadgeColor::Neutral text="Draft".into() />
                                    </Show>
                                    <Show when=move || mergeability.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                                        {
                                            let m = mergeability.get().unwrap();
                                            view! {
                                                <Badge
                                                    color=if m.mergeable { BadgeColor::Success } else { BadgeColor::Danger }
                                                    text=if m.mergeable { "Mergeable".into() } else { "Conflicts".into() }
                                                />
                                            }
                                        }
                                    </Show>
                                    <span class="text-sm text-gray-400 font-mono">
                                        {move || format!("#{}", pr_sig.get().map(|p| p.number).unwrap_or(0))}
                                    </span>
                                </div>
                                <div class="flex items-center gap-3 text-sm text-gray-500 dark:text-gray-400">
                                    <span class="text-green-600 dark:text-green-400 font-mono">
                                        {move || truncate_uuid(&pr_sig.get().map(|p| p.source_branch.clone()).unwrap_or_default(), 24)}
                                    </span>
                                    <span>"\u{2192}"</span>
                                    <span class="text-blue-600 dark:text-blue-400 font-mono">
                                        {move || truncate_uuid(&pr_sig.get().map(|p| p.target_branch.clone()).unwrap_or_default(), 24)}
                                    </span>
                                    <span>
                                        {move || relative_time(&pr_sig.get().map(|p| p.created_at.clone()).unwrap_or_default())}
                                    </span>
                                </div>
                            </div>
                            <div class="flex items-center gap-2">
                                <Show when=move || is_open() && !action_loading.get() && !merging.get() fallback=|| view! { <div class="hidden"></div> }>
                                    <select
                                        class="px-2 py-1 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm bg-white dark:bg-gray-800 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                        on:change=move |ev| set_merge_strategy.set(event_target_value(&ev))
                                    >
                                        <option value="merge" selected={move || merge_strategy.get() == "merge"}>
                                            "Create merge commit"
                                        </option>
                                        <option value="squash" selected={move || merge_strategy.get() == "squash"}>
                                            "Squash and merge"
                                        </option>
                                        <option value="rebase" selected={move || merge_strategy.get() == "rebase"}>
                                            "Rebase and merge"
                                        </option>
                                        <option value="fast-forward" selected={move || merge_strategy.get() == "fast-forward"}>
                                            "Fast-forward only"
                                        </option>
                                    </select>
                                    <Button variant=ButtonVariant::Primary on:click=handle_merge disabled=merging.get()>
                                        {move || if merging.get() { "Merging..." } else { "Merge Pull Request" }}
                                    </Button>
                                </Show>
                                <Show when=move || is_open() && !action_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                                    <Button variant=ButtonVariant::Danger on:click=handle_close>
                                        "Close"
                                    </Button>
                                </Show>
                                <Show when=move || pr_sig.get().is_some_and(|p| p.status == "closed") && !action_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                                    <Button variant=ButtonVariant::Secondary on:click=handle_reopen>
                                        "Reopen"
                                    </Button>
                                </Show>
                            </div>
                        </div>

                        <Show when=move || pr_sig.get().is_some_and(|p| !p.body.is_empty()) fallback=|| view! { <div class="hidden"></div> }>
                            <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
                                <div class="whitespace-pre-wrap text-sm text-gray-700 dark:text-gray-300 leading-relaxed">
                                    {move || pr_sig.get().map(|p| p.body.clone()).unwrap_or_default()}
                                </div>
                            </div>
                        </Show>

                        <Show when=move || pr_sig.get().is_some_and(|p| p.status == "merged") fallback=|| view! { <div class="hidden"></div> }>
                            <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
                                <div class="flex items-center gap-3 text-sm">
                                    <Badge color=BadgeColor::Info text="Merged".into() />
                                    <span class="text-gray-500">
                                        {move || format!(
                                            "merge_commit: {}",
                                            pr_sig.get().and_then(|p| p.merge_commit_id.clone()).unwrap_or_else(|| "N/A".into())
                                        )}
                                    </span>
                                </div>
                            </div>
                        </Show>

                        <Show when=move || merge_result.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                            <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
                                {
                                    let mr = merge_result.get().unwrap();
                                    view! {
                                        <div class={
                                            if mr.merged { "rounded border border-green-300 bg-green-50 dark:bg-green-900/20 dark:border-green-700 p-3 text-sm text-green-800 dark:text-green-200" }
                                            else { "rounded border border-red-300 bg-red-50 dark:bg-red-900/20 dark:border-red-700 p-3 text-sm text-red-800 dark:text-red-200" }
                                        }>
                                            {mr.message.clone()}
                                        </div>
                                    }
                                }
                            </div>
                        </Show>

                        // Reviewers section
                        <Show when=move || pr_sig.get().is_some_and(|p| !p.reviewers.is_empty()) fallback=|| view! { <div class="hidden"></div> }>
                            <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
                                <h3 class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">"Reviewers"</h3>
                                <For each=move || pr_sig.get().map(|p| p.reviewers.clone()).unwrap_or_default() key=|r| r.user_id.clone() let:reviewer>
                                    {
                                        let badge_color = match reviewer.review_status.as_str() {
                                            "approved" => BadgeColor::Success,
                                            "changes_requested" => BadgeColor::Danger,
                                            _ => BadgeColor::Neutral,
                                        };
                                        let label = match reviewer.review_status.as_str() {
                                            "approved" => "Approved",
                                            "changes_requested" => "Changes Requested",
                                            "commented" => "Commented",
                                            _ => "Pending",
                                        };
                                        view! {
                                            <div class="flex items-center gap-2 text-sm">
                                                <span class="font-mono text-gray-500">{truncate_uuid(&reviewer.user_id, 8)}</span>
                                                <Badge color=badge_color text=label.into() />
                                            </div>
                                        }
                                    }
                                </For>
                            </div>
                        </Show>

                        // Labels
                        <Show when=move || pr_sig.get().is_some_and(|p| !p.labels.is_empty()) fallback=|| view! { <div class="hidden"></div> }>
                            <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
                                <h3 class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">"Labels"</h3>
                                <div class="flex flex-wrap gap-2">
                                    <For each=move || pr_sig.get().map(|p| p.labels.clone()).unwrap_or_default() key=|l| l.id.clone() let:label>
                                        {
                                            let color = label.color.clone().unwrap_or_default();
                                            let style_str = format!("border-color: #{color}; color: #{color}; background-color: {color}15");
                                            view! {
                                                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border" style=style_str>
                                                    {label.name.clone()}
                                                </span>
                                            }
                                        }
                                    </For>
                                </div>
                            </div>
                        </Show>
                    </div>
                </Card>

                // Tab navigation
                <div class="border-b border-gray-200 dark:border-gray-700">
                    <nav class="-mb-px flex space-x-8" aria-label="Tabs">
                        <button
                            class=move || format!(
                                "px-4 py-2 text-sm font-medium border-b-2 transition-colors {}",
                                if active_tab.get() == "conversation" {
                                    "border-blue-600 text-blue-600 dark:border-blue-400 dark:text-blue-400"
                                } else {
                                    "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-200"
                                }
                            )
                            on:click=move |_| set_active_tab.set("conversation".into())
                        >
                            "Conversation"
                        </button>
                        <button
                            class=move || format!(
                                "px-4 py-2 text-sm font-medium border-b-2 transition-colors {}",
                                if active_tab.get() == "files" {
                                    "border-blue-600 text-blue-600 dark:border-blue-400 dark:text-blue-400"
                                } else {
                                    "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-200"
                                }
                            )
                            on:click=move |_| set_active_tab.set("files".into())
                        >
                            {move || {
                                let file_count = diff_data.get().map(|d| d.files.len()).unwrap_or(0);
                                if file_count > 0 {
                                    format!("Files Changed ({file_count})")
                                } else {
                                    "Files Changed".into()
                                }
                            }}
                        </button>
                    </nav>
                </div>

                // Conversation tab
                <Show when=move || active_tab.get() == "conversation" fallback=|| view! { <div class="hidden"></div> }>
                    <Card>
                        <form on:submit=handle_comment_submit class="space-y-3">
                            <Show when=move || comment_error.get().is_some()>
                                <ErrorBanner message=move || comment_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_comment_error.set(None)) />
                            </Show>
                            <textarea
                                id="pr-comment-input"
                                aria-label="Write a comment on this pull request"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                placeholder="Write a comment..."
                                rows="3"
                                on:input=move |ev| set_comment_body.set(event_target_value(&ev))
                                prop:value=comment_body.get()
                            ></textarea>
                            <Button variant=ButtonVariant::Primary disabled=comment_saving.get() || comment_body.get().trim().is_empty()>
                                {move || if comment_saving.get() { "Posting..." } else { "Add Comment" }}
                            </Button>
                        </form>
                    </Card>
                </Show>

                // Files Changed tab
                <Show when=move || active_tab.get() == "files" fallback=|| view! { <div class="hidden"></div> }>
                    <div class="space-y-4">
                        // Diff view toggle + file tree sidebar toggle
                        <div class="flex items-center justify-between">
                            <Show when=move || diff_data.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                                <span class="text-xs text-gray-500 font-mono">
                                    {move || {
                                        let d = diff_data.get().unwrap();
                                        format!(
                                            "+{} -{} \u{00b7} {} file{}",
                                            d.total_additions,
                                            d.total_deletions,
                                            d.files.len(),
                                            if d.files.len() != 1 { "s" } else { "" },
                                        )
                                    }}
                                </span>
                            </Show>
                            <div class="flex items-center gap-1 bg-gray-100 dark:bg-gray-700 rounded-lg p-1">
                                <button
                                    class=move || format!(
                                        "px-3 py-1 text-xs font-medium rounded-md transition-colors {}",
                                        if diff_view_mode.get() == "unified" {
                                            "bg-white dark:bg-gray-600 text-gray-900 dark:text-gray-100 shadow-sm"
                                        } else {
                                            "text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300"
                                        }
                                    )
                                    on:click=move |_| set_diff_view_mode.set("unified".into())
                                >
                                    "Unified"
                                </button>
                                <button
                                    class=move || format!(
                                        "px-3 py-1 text-xs font-medium rounded-md transition-colors {}",
                                        if diff_view_mode.get() == "side-by-side" {
                                            "bg-white dark:bg-gray-600 text-gray-900 dark:text-gray-100 shadow-sm"
                                        } else {
                                            "text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300"
                                        }
                                    )
                                    on:click=move |_| set_diff_view_mode.set("side-by-side".into())
                                >
                                    "Side-by-side"
                                </button>
                            </div>
                        </div>

                        // Content with optional file tree sidebar
                        {move || {
                            let d = diff_data.get();
                            let idiff = inline_diff.get();
                            let files = if let Some(ref id) = idiff {
                                id.files.iter().map(|f| (f.path.clone(), f.status.clone(), f.additions, f.deletions)).collect::<Vec<_>>()
                            } else if let Some(ref d) = d {
                                d.files.iter().map(|f| (f.path.clone(), f.status.clone(), f.additions, f.deletions)).collect::<Vec<_>>()
                            } else {
                                Vec::new()
                            };
                            if files.len() <= 1 {
                                // Single file: no sidebar needed
                                view! {
                                    <Show when=move || inline_diff.get().is_some() fallback=move || view! {
                                        <Show when=move || diff_data.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                                            <Card>
                                                <div class="space-y-1">
                                                    <For
                                                        each=move || diff_data.get().map(|d| d.files.clone()).unwrap_or_default()
                                                        key=|f| f.path.clone()
                                                        let:file
                                                    >
                                                        {
                                                            let status_bg = match file.status.as_str() {
                                                                "added" => "#dcfce7",
                                                                "removed" => "#fecaca",
                                                                _ => "#e5e7eb",
                                                            };
                                                            let status_icon = match file.status.as_str() {
                                                                "added" => "A",
                                                                "removed" => "D",
                                                                _ => "M",
                                                            };
                                                            view! {
                                                                <div class="flex items-center justify-between text-sm py-1.5 border-b border-gray-100 dark:border-gray-800 last:border-b-0">
                                                                    <div class="flex items-center gap-2 min-w-0">
                                                                        <span
                                                                            class="inline-flex items-center justify-center w-4 h-4 rounded text-[10px] font-mono font-bold dark:text-gray-400"
                                                                            style=format!("background-color: {status_bg}")
                                                                        >
                                                                            {status_icon}
                                                                        </span>
                                                                        <span class="truncate font-mono text-xs">{file.path.clone()}</span>
                                                                    </div>
                                                                    <div class="flex items-center gap-3 shrink-0">
                                                                        <span class="text-green-600 dark:text-green-400 text-xs font-mono">+{file.additions}</span>
                                                                        <span class="text-red-600 dark:text-red-400 text-xs font-mono">-{file.deletions}</span>
                                                                    </div>
                                                                </div>
                                                            }
                                                        }
                                                    </For>
                                                </div>
                                            </Card>
                                        </Show>
                                    }>
                                        <For
                                            each=move || inline_diff.get().map(|d| d.files.clone()).unwrap_or_default()
                                            key=|f| f.path.clone()
                                            let:file
                                        >
                                            {
                                                let file_path = file.path.clone();
                                                let file_status = file.status.clone();
                                                let file_additions = file.additions;
                                                let file_deletions = file.deletions;
                                                let file_hunks = file.hunks.clone();
                                                let is_unified = Signal::derive(move || diff_view_mode.get() == "unified");

                                                view! {
                                                    <CollapsibleDiffFile
                                                        path=file_path
                                                        status=file_status
                                                        additions=file_additions
                                                        deletions=file_deletions
                                                        hunks=file_hunks
                                                        is_unified=is_unified
                                                    />
                                                }
                                            }
                                        </For>
                                    </Show>
                                }.into_any()
                            } else {
                                // Multiple files: show sidebar + diffs
                                let files_for_sidebar = StoredValue::new(files);
                                view! {
                                    <div class="flex gap-4">
                                        // File tree sidebar
                                        <div class="hidden lg:block w-64 shrink-0">
                                            <div class="sticky top-4 bg-white dark:bg-gray-800 rounded-md border border-gray-200 dark:border-gray-700 overflow-hidden max-h-[70vh] overflow-y-auto">
                                                <div class="px-3 py-2 border-b border-gray-200 dark:border-gray-700 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                                                    "Files Changed"
                                                </div>
                                                <div class="divide-y divide-gray-100 dark:divide-gray-700/50">
                                                    <For
                                                        each=move || files_for_sidebar.get_value()
                                                        key=|f| f.0.clone()
                                                        let:file_info
                                                    >
                                                        {
                                                            let fp = file_info.0.clone();
                                                            let status_char = match file_info.1.as_str() {
                                                                "added" => "A",
                                                                "removed" => "D",
                                                                "renamed" | "copied" => "R",
                                                                _ => "M",
                                                            };
                                                            let status_color = match file_info.1.as_str() {
                                                                "added" => "text-green-600 dark:text-green-400",
                                                                "removed" => "text-red-600 dark:text-red-400",
                                                                _ => "text-gray-500 dark:text-gray-400",
                                                            };
                                                            let anchor_id = format!("diff-{}", fp.replace('/', "-"));
                                                            let sc = format!("font-mono font-bold {status_color}");
                                                            view! {
                                                                <a
                                                                    href=format!("#{anchor_id}")
                                                                    class="flex items-center justify-between px-3 py-1.5 text-xs hover:bg-gray-50 dark:hover:bg-gray-750 transition-colors"
                                                                >
                                                                    <div class="flex items-center gap-1.5 min-w-0">
                                                                        <span class=sc>{status_char}</span>
                                                                        <span class="truncate font-mono text-gray-700 dark:text-gray-300">{fp}</span>
                                                                    </div>
                                                                    <div class="flex items-center gap-1.5 shrink-0 ml-2">
                                                                        <span class="text-green-600 dark:text-green-400">+{file_info.2}</span>
                                                                        <span class="text-red-600 dark:text-red-400">-{file_info.3}</span>
                                                                    </div>
                                                                </a>
                                                            }
                                                        }
                                                    </For>
                                                </div>
                                            </div>
                                        </div>
                                        // Diff content
                                        <div class="flex-1 min-w-0 space-y-4">
                                            <Show when=move || inline_diff.get().is_some() fallback=move || view! {
                                                <Show when=move || diff_data.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                                                    <Card>
                                                        <div class="space-y-1">
                                                            <For
                                                                each=move || diff_data.get().map(|d| d.files.clone()).unwrap_or_default()
                                                                key=|f| f.path.clone()
                                                                let:file
                                                            >
                                                                {
                                                                    let status_bg = match file.status.as_str() {
                                                                        "added" => "#dcfce7",
                                                                        "removed" => "#fecaca",
                                                                        _ => "#e5e7eb",
                                                                    };
                                                                    let status_icon = match file.status.as_str() {
                                                                        "added" => "A",
                                                                        "removed" => "D",
                                                                        _ => "M",
                                                                    };
                                                                    view! {
                                                                        <div class="flex items-center justify-between text-sm py-1.5 border-b border-gray-100 dark:border-gray-800 last:border-b-0">
                                                                            <div class="flex items-center gap-2 min-w-0">
                                                                                <span
                                                                                    class="inline-flex items-center justify-center w-4 h-4 rounded text-[10px] font-mono font-bold dark:text-gray-400"
                                                                                    style=format!("background-color: {status_bg}")
                                                                                >
                                                                                    {status_icon}
                                                                                </span>
                                                                                <span class="truncate font-mono text-xs">{file.path.clone()}</span>
                                                                            </div>
                                                                            <div class="flex items-center gap-3 shrink-0">
                                                                                <span class="text-green-600 dark:text-green-400 text-xs font-mono">+{file.additions}</span>
                                                                                <span class="text-red-600 dark:text-red-400 text-xs font-mono">-{file.deletions}</span>
                                                                            </div>
                                                                        </div>
                                                                    }
                                                                }
                                                            </For>
                                                        </div>
                                                    </Card>
                                                </Show>
                                            }>
                                                <For
                                                    each=move || inline_diff.get().map(|d| d.files.clone()).unwrap_or_default()
                                                    key=|f| f.path.clone()
                                                    let:file
                                                >
                                                    {
                                                        let file_path = file.path.clone();
                                                        let anchor_id = format!("diff-{}", file_path.replace('/', "-"));
                                                        let file_status = file.status.clone();
                                                        let file_additions = file.additions;
                                                        let file_deletions = file.deletions;
                                                        let file_hunks = file.hunks.clone();
                                                        let is_unified = Signal::derive(move || diff_view_mode.get() == "unified");

                                                        view! {
                                                            <div id=anchor_id>
                                                                <CollapsibleDiffFile
                                                                    path=file_path
                                                                    status=file_status
                                                                    additions=file_additions
                                                                    deletions=file_deletions
                                                                    hunks=file_hunks
                                                                    is_unified=is_unified
                                                                />
                                                            </div>
                                                        }
                                                    }
                                                </For>
                                            </Show>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                </Show>
            </Show>
        </div>
    }
}

#[component]
fn DiffLineRow(
    old_line_no: Option<u32>,
    new_line_no: Option<u32>,
    content: String,
    kind: String,
    file_path: String,
) -> impl IntoView {
    let (bg_style, prefix) = match kind.as_str() {
        "addition" => ("background-color: #e6ffec;", "+"),
        "deletion" => ("background-color: #ffebe9;", "-"),
        "context" => ("background-color: #f6f8fa;", " "),
        _ => ("", " "),
    };
    let row_class = "flex items-center text-xs font-mono border-b border-gray-100 dark:border-gray-800 group/line".to_string();

    let (show_comment_box, set_show_comment_box) = signal(false);
    let (comment_text, set_comment_text) = signal(String::new());
    let (comment_saving, set_comment_saving) = signal(false);
    let line_no = new_line_no.or(old_line_no).unwrap_or(0);
    let fp = StoredValue::new(file_path);

    let submit_inline_comment = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let body = comment_text.get();
        if body.trim().is_empty() {
            return;
        }
        let fp_c = fp.get_value();
        set_comment_saving.set(true);
        // Comment will be posted when auth context is available in scope
        // For now, reset the form
        set_comment_text.set(String::new());
        set_show_comment_box.set(false);
        set_comment_saving.set(false);
        let _ = fp_c;
        let _ = line_no;
    };

    view! {
        <div>
            <div class=row_class style=bg_style>
                <span class="w-12 text-right pr-2 text-gray-400 select-none shrink-0">
                    {old_line_no.map(|n| n.to_string()).unwrap_or_default()}
                </span>
                <span class="w-12 text-right pr-2 text-gray-400 select-none shrink-0">
                    {new_line_no.map(|n| n.to_string()).unwrap_or_default()}
                </span>
                <span class="w-4 text-center text-gray-400 select-none shrink-0">{prefix}</span>
                <span class="flex-1 whitespace-pre px-2">{content}</span>
                <button
                    class="opacity-0 group-hover/line:opacity-100 shrink-0 px-1 py-0.5 text-[10px] text-gray-500 hover:text-blue-600 dark:hover:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded transition-all"
                    title="Add comment"
                    on:click=move |_| set_show_comment_box.update(|c| *c = !*c)
                >
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                    </svg>
                </button>
            </div>
            <Show when=move || show_comment_box.get()>
                <div class="border-l-2 border-blue-400 bg-gray-50 dark:bg-gray-800/50 px-4 py-3 ml-24">
                    <form on:submit=submit_inline_comment class="space-y-2">
                        <textarea
                            class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                            placeholder="Write a comment..."
                            rows="3"
                            on:input=move |ev| set_comment_text.set(event_target_value(&ev))
                            prop:value=comment_text.get()
                        ></textarea>
                        <div class="flex items-center gap-2">
                            <button
                                type="submit"
                                class="px-3 py-1.5 text-xs font-medium rounded bg-blue-600 hover:bg-blue-700 text-white disabled:opacity-50"
                                disabled=comment_saving.get() || comment_text.get().trim().is_empty()
                            >
                                {move || if comment_saving.get() { "Posting..." } else { "Comment" }}
                            </button>
                            <button
                                type="button"
                                class="px-3 py-1.5 text-xs font-medium rounded text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700"
                                on:click=move |_| set_show_comment_box.set(false)
                            >
                                "Cancel"
                            </button>
                        </div>
                    </form>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn DiffLineRowSide(
    line_no: Option<u32>,
    content: String,
    kind: String,
    side: String,
) -> impl IntoView {
    let (bg_style, prefix) = match (kind.as_str(), side.as_str()) {
        ("addition", "new") => ("background-color: #e6ffec;", "+"),
        ("deletion", "old") => ("background-color: #ffebe9;", "-"),
        ("context", _) => ("background-color: #f6f8fa;", " "),
        _ => ("background-color: #f6f8fa;", " "),
    };
    let row_class = "flex items-center text-xs font-mono border-b border-gray-100 dark:border-gray-800 group/line".to_string();

    let (show_comment_box, set_show_comment_box) = signal(false);
    let (comment_text, set_comment_text) = signal(String::new());
    let (comment_saving, set_comment_saving) = signal(false);

    let submit_inline_comment = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let body = comment_text.get();
        if body.trim().is_empty() {
            return;
        }
        set_comment_text.set(String::new());
        set_show_comment_box.set(false);
        set_comment_saving.set(false);
    };

    view! {
        <div>
            <div class=row_class style=bg_style>
                <span class="w-12 text-right pr-2 text-gray-400 select-none shrink-0">
                    {line_no.map(|n| n.to_string()).unwrap_or_default()}
                </span>
                <span class="w-4 text-center text-gray-400 select-none shrink-0">{prefix}</span>
                <span class="flex-1 whitespace-pre px-2">{content}</span>
                <button
                    class="opacity-0 group-hover/line:opacity-100 shrink-0 px-1 py-0.5 text-[10px] text-gray-500 hover:text-blue-600 dark:hover:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded transition-all"
                    title="Add comment"
                    on:click=move |_| set_show_comment_box.update(|c| *c = !*c)
                >
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                    </svg>
                </button>
            </div>
            <Show when=move || show_comment_box.get()>
                <div class="border-l-2 border-blue-400 bg-gray-50 dark:bg-gray-800/50 px-4 py-3 ml-12">
                    <form on:submit=submit_inline_comment class="space-y-2">
                        <textarea
                            class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                            placeholder="Write a comment..."
                            rows="3"
                            on:input=move |ev| set_comment_text.set(event_target_value(&ev))
                            prop:value=comment_text.get()
                        ></textarea>
                        <div class="flex items-center gap-2">
                            <button
                                type="submit"
                                class="px-3 py-1.5 text-xs font-medium rounded bg-blue-600 hover:bg-blue-700 text-white disabled:opacity-50"
                                disabled=comment_saving.get() || comment_text.get().trim().is_empty()
                            >
                                {move || if comment_saving.get() { "Posting..." } else { "Comment" }}
                            </button>
                            <button
                                type="button"
                                class="px-3 py-1.5 text-xs font-medium rounded text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700"
                                on:click=move |_| set_show_comment_box.set(false)
                            >
                                "Cancel"
                            </button>
                        </div>
                    </form>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn HunkView(hunk: crate::api::types::DiffHunk, file_path: String) -> impl IntoView {
    let lines = hunk.lines;
    let header = hunk.header;
    view! {
        <div class="bg-gray-50 dark:bg-gray-800/50 px-3 py-1 text-xs text-gray-500 font-mono border-b border-gray-200 dark:border-gray-700">
            {header}
        </div>
        <For
            each=move || lines.clone()
            key=|l| format!("{}-{}-{}", l.old_line_no.unwrap_or(0), l.new_line_no.unwrap_or(0), l.content)
            let:line
        >
            {
                let fp = file_path.clone();
                view! {
                    <DiffLineRow
                        old_line_no=line.old_line_no
                        new_line_no=line.new_line_no
                        content=line.content
                        kind=line.kind
                        file_path=fp
                    />
                }
            }
        </For>
    }
}

#[component]
fn HunkViewSide(
    hunk: crate::api::types::DiffHunk,
    side: String,
    file_path: String,
) -> impl IntoView {
    let lines = hunk.lines;
    let header = hunk.header;
    let side_key = side.clone();
    view! {
        <div class="bg-gray-50 dark:bg-gray-800/50 px-3 py-1 text-xs text-gray-500 font-mono border-b border-gray-200 dark:border-gray-700">
            {header}
        </div>
        <For
            each=move || lines.clone()
            key=move |l: &crate::api::types::DiffLine| {
                let line_no = if side_key == "old" { l.old_line_no } else { l.new_line_no };
                format!("{}-{}", line_no.unwrap_or(0), l.content)
            }
            let:line
        >
            {
                let line_no = if side == "old" { line.old_line_no } else { line.new_line_no };
                let kind = line.kind.clone();
                let content = line.content.clone();
                let side_str = side.clone();
                let _fp = file_path.clone();
                view! {
                    <DiffLineRowSide
                        line_no=line_no
                        content=content
                        kind=kind
                        side=side_str
                    />
                }
            }
        </For>
    }
}

#[component]
fn SideBySideDiffView(hunks: Vec<crate::api::types::DiffHunk>, file_path: String) -> impl IntoView {
    let hunks_left = hunks.clone();
    let hunks_right = hunks;
    let fp_left = file_path.clone();
    let fp_right = file_path;
    view! {
        <div class="overflow-x-auto">
            <div class="flex min-w-max">
                <div class="w-1/2 border-r border-gray-200 dark:border-gray-700">
                    <For
                        each=move || hunks_left.clone()
                        key=|h| h.header.clone()
                        let:hunk
                    >
                        {
                            let fp = fp_left.clone();
                            view! { <HunkViewSide hunk=hunk side="old".into() file_path=fp /> }
                        }
                    </For>
                </div>
                <div class="w-1/2">
                    <For
                        each=move || hunks_right.clone()
                        key=|h| h.header.clone()
                        let:hunk
                    >
                        {
                            let fp = fp_right.clone();
                            view! { <HunkViewSide hunk=hunk side="new".into() file_path=fp /> }
                        }
                    </For>
                </div>
            </div>
        </div>
    }
}

#[component]
fn UnifiedDiffView(hunks: Vec<crate::api::types::DiffHunk>, file_path: String) -> impl IntoView {
    view! {
        <div class="overflow-x-auto">
            <For
                each=move || hunks.clone()
                key=|h| h.header.clone()
                let:hunk
            >
                {
                let fp = file_path.clone();
                    view! { <HunkView hunk=hunk file_path=fp /> }
                }
            </For>
        </div>
    }
}

#[component]
fn DiffContent(
    hunks: Vec<crate::api::types::DiffHunk>,
    is_unified: Signal<bool>,
    file_path: String,
) -> impl IntoView {
    let hunks_left = StoredValue::new(hunks.clone());
    let hunks_right = StoredValue::new(hunks);
    let fp_left = StoredValue::new(file_path.clone());
    let fp_right = StoredValue::new(file_path);
    view! {
        <div class="mt-2 border border-gray-200 dark:border-gray-700 rounded-md overflow-hidden">
            <Show when=move || is_unified.get() fallback=move || view! {
                <SideBySideDiffView hunks=hunks_left.get_value() file_path=fp_left.get_value() />
            }>
                <UnifiedDiffView hunks=hunks_right.get_value() file_path=fp_right.get_value() />
            </Show>
        </div>
    }
}

#[component]
fn CollapsibleDiffFile(
    path: String,
    status: String,
    additions: u32,
    deletions: u32,
    hunks: Vec<crate::api::types::DiffHunk>,
    is_unified: Signal<bool>,
) -> impl IntoView {
    let (collapsed, set_collapsed) = signal(false);

    let (status_bg, status_icon, _status_label_text) = match status.as_str() {
        "added" => ("bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300", "A", "Added"),
        "removed" => ("bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300", "D", "Deleted"),
        "renamed" | "copied" => ("bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300", "R", "Renamed"),
        "modified" => ("bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-300", "M", "Modified"),
        _ => ("bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300", "M", "Modified"),
    };

    let total = additions + deletions;
    let add_pct = if total > 0 { (additions as f64 / total as f64 * 100.0) as u32 } else { 50 };
    let del_pct = if total > 0 { 100 - add_pct } else { 50 };

    let chevron_class = move || format!("w-4 h-4 text-gray-400 transition-transform {}", if collapsed.get() { "" } else { "rotate-90" });
    let badge_class = StoredValue::new(format!("inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[11px] font-mono font-bold {status_bg}"));
    let hunks_sv = StoredValue::new(hunks);
    let path_sv = StoredValue::new(path);

    view! {
        <Card>
            // File header (collapsible)
            <button
                class="w-full flex items-center justify-between py-2 px-1 -my-2 -mx-1 rounded hover:bg-gray-50 dark:hover:bg-gray-750 transition-colors"
                on:click=move |_| set_collapsed.update(|c| *c = !*c)
            >
                <div class="flex items-center gap-2 min-w-0">
                    <svg
                        class=chevron_class
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                    </svg>
                    <span class={badge_class.get_value()}>
                        {status_icon}
                    </span>
                    <span class="font-mono text-sm text-gray-700 dark:text-gray-300 truncate">{path_sv.get_value()}</span>
                </div>
                <div class="flex items-center gap-3 shrink-0">
                    // File statistics bar
                    <div class="flex items-center gap-2">
                        <div class="flex items-center h-1.5 w-24 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
                            <div class="bg-green-500 dark:bg-green-400 h-full" style:width={format!("{add_pct}%")}></div>
                            <div class="bg-red-500 dark:bg-red-400 h-full" style:width={format!("{del_pct}%")}></div>
                        </div>
                        <span class="text-green-600 dark:text-green-400 text-xs font-mono">+{additions}</span>
                        <span class="text-red-600 dark:text-red-400 text-xs font-mono">-{deletions}</span>
                    </div>
                </div>
            </button>

            // Diff content (collapsible)
            <Show when=move || !collapsed.get() fallback=|| view! { <div class="hidden"></div> }>
                <DiffContent hunks=hunks_sv.get_value() is_unified=is_unified file_path=path_sv.get_value() />
            </Show>
        </Card>
    }
}
