#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::api::types::{
    CreatePrCommentBody, MergePullRequestBody, MergeResponse, MergeabilityResponse, PrDiffResponse,
    PullRequestResponse, UpdatePullRequestBody,
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
    let (merge_strategy, set_merge_strategy) = signal(String::from("merge"));

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
                            let owner_dc = owner_val;
                            let name_dc = name_val;
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

                        // Files Changed
                        <Show when=move || diff_data.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                            <Card>
                                <div class="space-y-3">
                                    <div class="flex items-center justify-between">
                                        <h3 class="text-sm font-semibold text-gray-700 dark:text-gray-300">
                                            "Files Changed"
                                        </h3>
                                        <span class="text-xs text-gray-500 font-mono">
                                            {move || {
                                                let d = diff_data.get().unwrap();
                                                format!(
                                                    "+{} -{} · {} file{} · {} commit{}",
                                                    d.total_additions,
                                                    d.total_deletions,
                                                    d.files.len(),
                                                    if d.files.len() != 1 { "s" } else { "" },
                                                    d.commit_count,
                                                    if d.commit_count != 1 { "s" } else { "" },
                                                )
                                            }}
                                        </span>
                                    </div>
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

                // Comments
                <div class="mt-6 space-y-4">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">"Comments"</h2>
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
                </div>
            </Show>
        </div>
    }
}
