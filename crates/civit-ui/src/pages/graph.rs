#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::api::types::{GraphNode, GraphResponse};
use crate::components::{Card, ErrorBanner, Spinner};
use crate::state::auth::use_auth;

const BRANCH_COLORS: &[&str] = &[
    "#3b82f6", "#ef4444", "#22c55e", "#f59e0b", "#8b5cf6",
    "#ec4899", "#06b6d4", "#f97316", "#14b8a6", "#a855f7",
];

fn branch_color(branch: &str, all_branches: &[String]) -> &'static str {
    let idx = all_branches.iter().position(|b| b == branch).unwrap_or(0);
    BRANCH_COLORS[idx % BRANCH_COLORS.len()]
}

fn truncate_sha(sha: &str) -> String {
    if sha.len() > 8 {
        sha[..8].to_string()
    } else {
        sha.to_string()
    }
}

fn truncate_msg(msg: &str, max: usize) -> String {
    let first_line = msg.lines().next().unwrap_or("");
    if first_line.len() > max {
        format!("{}...", &first_line[..max])
    } else {
        first_line.to_string()
    }
}

#[derive(Clone, PartialEq)]
struct GraphCommit {
    sha: String,
    short_sha: String,
    message: String,
    first_line: String,
    author: String,
    date: String,
    parents: Vec<String>,
    branch: Option<String>,
    lane: usize,
    parent_lanes: Vec<usize>,
}

fn layout_commits(nodes: &[GraphNode]) -> (Vec<GraphCommit>, Vec<String>) {
    let mut branches: Vec<String> = nodes
        .iter()
        .filter_map(|n| n.branch.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    branches.sort();

    let mut sha_to_lane: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut next_lane = 0usize;

    let mut commits = Vec::new();
    for node in nodes.iter() {
        let lane = if let Some(&l) = sha_to_lane.get(&node.sha) {
            l
        } else {
            let l = next_lane;
            next_lane += 1;
            l
        };
        sha_to_lane.insert(node.sha.clone(), lane);

        let parent_lanes: Vec<usize> = node
            .parents
            .iter()
            .filter_map(|p| sha_to_lane.get(p).copied())
            .collect();

        let short_sha = truncate_sha(&node.sha);
        let first_line = truncate_msg(&node.message, 80);

        commits.push(GraphCommit {
            sha: node.sha.clone(),
            short_sha,
            message: node.message.clone(),
            first_line,
            author: node.author.clone(),
            date: node.date.clone(),
            parents: node.parents.clone(),
            branch: node.branch.clone(),
            lane,
            parent_lanes,
        });
    }

    (commits, branches)
}

#[component]
fn GraphCommitRow(
    commit: GraphCommit,
    total_lanes: usize,
    color: String,
) -> impl IntoView {
    let lane_dots: Vec<usize> = (0..total_lanes).collect();
    let commit_sha = StoredValue::new(commit.sha.clone());
    let short_sha = commit.short_sha.clone();
    let commit_msg = commit.first_line.clone();
    let commit_author = commit.author.clone();
    let commit_date = commit.date.clone();
    let branch_label = StoredValue::new(commit.branch.clone());
    let color2 = StoredValue::new(color.clone());
    let color_stored = StoredValue::new(color);
    let parent_lanes_stored = StoredValue::new(commit.parent_lanes.clone());
    let current_lane = commit.lane;

    view! {
        <div class="flex items-center border-b border-gray-100 dark:border-gray-800 hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors">
            <div class="flex items-center shrink-0" style="width: 160px; padding-left: 8px;">
                <For
                    each=move || lane_dots.clone()
                    key=move |l| format!("{}-{}", commit_sha.get_value(), l)
                    let:lane_idx
                >
                    {
                        let lane = lane_idx;
                        let is_current = lane == current_lane;

                        view! {
                            <div class="relative flex items-center" style="width: 24px; height: 40px;">
                                // Vertical line through this lane
                                <div
                                    class="absolute w-0.5"
                                    style=move || {
                                        let left = format!("{}px", 11);
                                        if is_current {
                                            format!("left: {left}; top: 0; bottom: 0; background-color: {}", color_stored.get_value())
                                        } else {
                                            format!("left: {left}; top: 0; bottom: 0; background-color: #d1d5db")
                                        }
                                    }
                                ></div>
                                // Horizontal connection to parent in a different lane
                                <Show when=move || {
                                    let parents = parent_lanes_stored.get_value();
                                    is_current && parents.iter().any(|&p| p != current_lane)
                                }>
                                    <div
                                        class="absolute h-0.5"
                                        style=move || {
                                            let parents = parent_lanes_stored.get_value();
                                            let current_left = 11;
                                            if let Some(&target_lane) = parents.iter().find(|&&p| p != current_lane) {
                                                let x1 = current_left;
                                                let x2 = (target_lane as i32 - current_lane as i32) * 24 + current_left;
                                                let (left, width) = if x2 > x1 {
                                                    (x1, x2 - x1)
                                                } else {
                                                    (x2, x1 - x2)
                                                };
                                                format!("left: {left}px; top: 50%; width: {width}px; background-color: #d1d5db; transform: translateY(-50%);")
                                            } else {
                                                "display: none;".to_string()
                                            }
                                        }
                                    ></div>
                                </Show>
                                // Commit dot
                                <Show when=move || is_current>
                                    <div
                                        class="absolute w-3 h-3 rounded-full border-2 border-white dark:border-gray-900 z-10 shadow-sm"
                                        style=move || format!("left: 7px; top: 50%; transform: translateY(-50%); background-color: {}", color_stored.get_value())
                                        title=move || commit_sha.get_value()
                                    ></div>
                                </Show>
                            </div>
                        }
                    }
                </For>
            </div>

            <div class="flex-1 min-w-0 px-3 py-2">
                <div class="flex items-center gap-2">
                    <span class="text-sm text-gray-900 dark:text-gray-100 truncate">
                        {commit_msg}
                    </span>
                    <Show when=move || branch_label.get_value().is_some()>
                        <span
                            class="shrink-0 px-1.5 py-0.5 text-[10px] font-medium rounded-full text-white shadow-sm"
                            style=move || format!("background-color: {}", color2.get_value())
                        >
                            {move || branch_label.get_value().unwrap_or_default()}
                        </span>
                    </Show>
                </div>
                <div class="flex items-center gap-3 mt-0.5 text-xs text-gray-500 dark:text-gray-400">
                    <span class="font-mono">{short_sha}</span>
                    <span>{commit_author}</span>
                    <span>{commit_date}</span>
                </div>
            </div>

            <div class="shrink-0 pr-4 flex items-center gap-2">
                <a
                    class="text-xs text-blue-600 dark:text-blue-400 hover:underline"
                    href=move || format!("/repos/commit/{}", commit_sha.get_value())
                >
                    "View"
                </a>
            </div>
        </div>
    }
}

#[component]
pub fn GraphPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let auth = use_auth();

    let (graph_data, set_graph_data) = signal(None::<GraphResponse>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (all_commits, set_all_commits) = signal(Vec::<GraphNode>::new());
    let (page, set_page) = signal(1usize);
    let (has_more, set_has_more) = signal(true);
    let (loading_more, set_loading_more) = signal(false);

    let fetch_graph = move |pg: usize| {
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();

        if pg == 1 {
            set_loading.set(true);
        } else {
            set_loading_more.set(true);
        }
        set_error.set(None);

        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/graph?page={pg}");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<GraphResponse>().await {
                        let new_nodes = data.nodes.clone();
                        if pg == 1 {
                            set_graph_data.set(Some(data));
                        }
                        if new_nodes.is_empty() {
                            set_has_more.set(false);
                        } else {
                            set_all_commits.update(|c| {
                                if pg == 1 {
                                    *c = new_nodes;
                                } else {
                                    c.extend(new_nodes);
                                }
                            });
                        }
                    } else {
                        set_error.set(Some("Failed to parse graph data.".into()));
                    }
                }
                Ok(_) => {
                    set_error.set(Some("Failed to load commit graph.".into()));
                }
                Err(_) => {
                    set_error.set(Some("Network error.".into()));
                }
            }
            set_loading.set(false);
            set_loading_more.set(false);
        });
    };

    leptos::task::spawn_local(async move {
        fetch_graph(1);
    });

    let load_more = move |_| {
        let next = page.get() + 1;
        set_page.set(next);
        fetch_graph(next);
    };

    let dismiss_error = Callback::new(move |_: ()| set_error.set(None));

    let computed_data = Memo::new(move |_| {
        graph_data.get().map(|data| {
            let (commits, branches) = layout_commits(&data.nodes);
            let total_lanes = branches.len().max(1);
            let branch_colors: Vec<(String, String)> = branches
                .iter()
                .map(|b| {
                    let c = branch_color(b, &branches);
                    (b.clone(), c.to_string())
                })
                .collect();
            (commits, total_lanes, branch_colors)
        })
    });

    view! {
        <div class="space-y-6">
            <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1">
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
                <span class="text-gray-700 dark:text-gray-300">"Graph"</span>
            </div>

            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">"Commit Graph"</h1>
                <A href=format!("/repos/{}/{}/commits", owner(), name())>
                    <span class="text-sm text-blue-600 dark:text-blue-400 hover:underline">
                        "View as list"
                    </span>
                </A>
            </div>

            <Show when=move || error.get().is_some()>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=dismiss_error />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="flex items-center justify-center py-12">
                        <Spinner />
                        <span class="ml-3 text-gray-500">"Loading commit graph..."</span>
                    </div>
                </Card>
            </Show>

            <Show when=move || !loading.get() && computed_data.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                {move || {
                    computed_data.get().map(|(commits, total_lanes, branch_colors)| {
                        let commits_sv = StoredValue::new(commits);
                        let branch_colors_sv = StoredValue::new(branch_colors);
                        view! {
                            <Card>
                                <div class="overflow-x-auto">
                                    <For
                                        each=move || commits_sv.get_value()
                                        key=|c| c.sha.clone()
                                        let:commit
                                    >
                                        {
                                            let color = branch_colors_sv
                                                .get_value()
                                                .iter()
                                                .find(|(b, _)| commit.branch.as_deref() == Some(b))
                                                .map(|(_, c)| c.clone())
                                                .unwrap_or_else(|| "#3b82f6".to_string());
                                            view! {
                                                <GraphCommitRow
                                                    commit=commit
                                                    total_lanes=total_lanes
                                                    color=color
                                                />
                                            }
                                        }
                                    </For>
                                </div>
                            </Card>
                        }
                    })
                }}
            </Show>

            <Card>
                <h3 class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-3">
                    "Legend"
                </h3>
                <div class="flex flex-wrap gap-4 text-xs text-gray-600 dark:text-gray-400">
                    <div class="flex items-center gap-2">
                        <div class="w-3 h-3 rounded-full bg-blue-500"></div>
                        <span>"Commit node"</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <div class="w-0.5 h-4 bg-gray-300 dark:bg-gray-600"></div>
                        <span>"Parent link"</span>
                    </div>
                    {move || {
                        computed_data.get().map(|(_, _, branch_colors)| {
                            branch_colors.into_iter().map(|(branch, color)| {
                                view! {
                                    <div class="flex items-center gap-2">
                                        <span
                                            class="px-1.5 py-0.5 text-[10px] font-medium rounded-full text-white"
                                            style=format!("background-color: {color}")
                                        >
                                            {branch}
                                        </span>
                                    </div>
                                }
                            }).collect::<Vec<_>>()
                        }).unwrap_or_default()
                    }}
                </div>
            </Card>

            <Show when=move || !loading.get() && has_more.get() && !loading_more.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex justify-center">
                    <button
                        class="px-4 py-2 text-sm font-medium rounded-md border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                        on:click=load_more
                    >
                        "Load more commits"
                    </button>
                </div>
            </Show>

            <Show when=move || loading_more.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex justify-center py-4">
                    <Spinner />
                    <span class="ml-2 text-sm text-gray-500">"Loading more..."</span>
                </div>
            </Show>

            <Show when=move || !loading.get() && !has_more.get() && all_commits.with(|c| !c.is_empty()) fallback=|| view! { <div class="hidden"></div> }>
                <div class="text-center text-xs text-gray-400 dark:text-gray-500 py-2">
                    "All commits loaded"
                </div>
            </Show>
        </div>
    }
}
