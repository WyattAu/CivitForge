#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::api::client::ApiClient;
use crate::components::{Card, ErrorBanner, Spinner};
use crate::state::auth::use_auth;
use crate::utils::*;

// ── Shared types for blame and file-commits ──

#[derive(Clone, serde::Deserialize)]
struct BlameLine {
    line_number: usize,
    content: String,
    #[serde(default)]
    commit_id: String,
    #[serde(default)]
    commit_message: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    time: String,
}

#[derive(Clone, serde::Deserialize)]
#[allow(dead_code)]
struct BlameData {
    lines: Vec<BlameLine>,
    #[serde(default)]
    path: String,
    #[serde(default)]
    language: String,
}

#[derive(Clone, serde::Deserialize)]
struct FileCommitEntry {
    #[serde(default)]
    id: String,
    #[serde(default)]
    message: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    time: String,
}

#[derive(Clone, serde::Deserialize)]
struct FileCommitsData {
    commits: Vec<FileCommitEntry>,
    #[serde(default)]
    path: String,
    #[serde(default)]
    total: usize,
}

// ── Blame View ──

const BLAME_AUTHOR_BG_COLORS: &[&str] = &[
    "#eef2ff", "#fef2f2", "#f0fdf4", "#fffbeb", "#f5f3ff",
    "#fdf2f8", "#ecfeff", "#fff7ed", "#f0fdfa", "#faf5ff",
];

const BLAME_AUTHOR_BG_COLORS_DARK: &[&str] = &[
    "#1e1b4b20", "#450a0a20", "#052e1620", "#451a0320", "#2e106520",
    "#4a044e20", "#08334420", "#43140720", "#042f2e20", "#3b076420",
];

fn author_bg_color(author: &str, dark: bool) -> String {
    let hash: usize = author.bytes().fold(0, |acc, b| acc.wrapping_add(b as usize));
    let idx = hash % BLAME_AUTHOR_BG_COLORS.len();
    if dark {
        BLAME_AUTHOR_BG_COLORS_DARK[idx].to_string()
    } else {
        BLAME_AUTHOR_BG_COLORS[idx].to_string()
    }
}

fn author_text_color(author: &str) -> String {
    let hash: usize = author.bytes().fold(0, |acc, b| acc.wrapping_add(b as usize));
    let colors = &[
        "#3b82f6", "#ef4444", "#22c55e", "#f59e0b", "#8b5cf6",
        "#ec4899", "#06b6d4", "#f97316", "#14b8a6", "#a855f7",
    ];
    colors[hash % colors.len()].to_string()
}

#[component]
pub fn BlamePage() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let auth = use_auth();

    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let path_param = move || {
        let qp = query.with(|q| q.get("path").unwrap_or_default());
        if qp.is_empty() {
            params.with(|p| p.get("path").unwrap_or_default())
        } else {
            qp
        }
    };
    let ref_param = move || query.with(|q| q.get("ref").unwrap_or_default());

    let (blame_data, set_blame_data) = signal(None::<BlameData>);
    let _blame_data = blame_data; // kept for potential future use
    let (blame_lines, set_blame_lines) = signal(Vec::<BlameLine>::new());
    let (blame_lang, set_blame_lang) = signal(String::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    leptos::task::spawn_local(async move {
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        let path_val = path_param();
        let ref_val = ref_param();

        if path_val.is_empty() {
            set_error.set(Some("No file path specified.".to_string()));
            set_loading.set(false);
            return;
        }

        let mut url = format!("/repos/{}/{}/blame?path={}", owner(), name(), path_val);
        if !ref_val.is_empty() {
            url.push_str(&format!("&ref={ref_val}"));
        }

        match client.get(&url).await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(data) = resp.json::<BlameData>().await {
                    set_blame_lines.set(data.lines.clone());
                    set_blame_lang.set(data.language.clone());
                    set_blame_data.set(Some(data));
                } else {
                    set_error.set(Some(sanitize_error("Failed to parse blame data.")));
                }
            }
            Ok(resp) => {
                let status = resp.status();
                set_error.set(Some(sanitize_error(&format!(
                    "Failed to load blame (HTTP {status})."
                ))));
            }
            Err(e) => {
                set_error.set(Some(sanitize_error(&format!("{e}"))));
            }
        }
        set_loading.set(false);
    });

    let dismiss_error = Callback::new(move |_: ()| set_error.set(None));

    view! {
        <div class="space-y-4">
            // Header
            <div>
                <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1">
                    <A href="/repos"><span class="hover:text-blue-600 dark:hover:text-blue-400">"Repositories"</span></A>
                    <span>"/"</span>
                    <A href=format!("/repos/{}/{}", owner(), name())><span class="hover:text-blue-600 dark:hover:text-blue-400">{format!("{}/{}", owner(), name())}</span></A>
                    <span>"/"</span>
                    <A href=format!("/repos/{}/{}/code/{}", owner(), name(), path_param())><span class="hover:text-blue-600 dark:hover:text-blue-400">{path_param()}</span></A>
                    <span>"/"</span>
                    <span class="text-gray-700 dark:text-gray-300">"Blame"</span>
                </div>
                <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                    {move || path_param()}
                </h1>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=dismiss_error />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center gap-2">
                    <Spinner />
                    <span class="text-gray-500 dark:text-gray-400">"Loading blame..."</span>
                </div>
            </Show>

            <Show when=move || !blame_lines.with(|l| l.is_empty()) fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="overflow-x-auto">
                        <table class="w-full text-sm font-mono">
                            <thead>
                                <tr class="border-b border-gray-200 dark:border-gray-700 text-gray-500 dark:text-gray-400 text-xs">
                                    <th class="pb-2 pr-4 text-left w-8 font-medium">"#"</th>
                                    <th class="pb-2 pr-4 text-left font-medium">"Commit"</th>
                                    <th class="pb-2 pr-4 text-left font-medium">"Author"</th>
                                    <th class="pb-2 text-left font-medium">"Code"</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-gray-100 dark:divide-gray-700/50">
                                <For each=move || blame_lines.get() key=|l| l.line_number let:line>
                                    {
                                        let msg = line.commit_message.clone();
                                        let msg_title = line.commit_message.clone();
                                        let short_id = if line.commit_id.len() >= 8 {
                                            line.commit_id[..8].to_string()
                                        } else {
                                            line.commit_id.clone()
                                        };
                                        let commit_id = line.commit_id.clone();
                                        let author = line.author.clone();
                                        let time = line.time.clone();
                                        let lang = blame_lang.get();
                                        let bg = author_bg_color(&author, false);
                                        let text_c = author_text_color(&author);
                                        let owner_v = owner();
                                        let name_v = name();
                                        view! {
                                            <tr class="hover:brightness-95 dark:hover:brightness-110 transition-all">
                                                <td class="py-0.5 pr-4 text-gray-400 dark:text-gray-500 text-right select-none">
                                                    {line.line_number}
                                                </td>
                                                <td class="py-0.5 pr-4 text-xs" style=format!("background-color: {bg}")>
                                                    <a
                                                        class="font-mono hover:underline"
                                                        style=format!("color: {text_c}")
                                                        href=format!("/repos/{}/{}/commit/{}", owner_v, name_v, commit_id)
                                                        title=msg_title
                                                    >
                                                        {short_id}
                                                    </a>
                                                    <span class="ml-1 text-gray-500 dark:text-gray-400 truncate block max-w-[200px]">{msg}</span>
                                                </td>
                                                <td class="py-0.5 pr-4 text-xs whitespace-nowrap" style=format!("background-color: {bg}")>
                                                    <span style=format!("color: {text_c}")>{author}</span>
                                                    <span class="ml-1 text-gray-400 dark:text-gray-500">{time}</span>
                                                </td>
                                                <td class="py-0.5 whitespace-pre">
                                                    <code data-lang=lang>{line.content.clone()}</code>
                                                </td>
                                            </tr>
                                        }
                                    }
                                </For>
                            </tbody>
                        </table>
                    </div>
                </Card>
            </Show>
        </div>
    }
}

// ── File Commits History View ──

#[component]
pub fn FileCommitsPage() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let auth = use_auth();

    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let path_param = move || {
        let qp = query.with(|q| q.get("path").unwrap_or_default());
        if qp.is_empty() {
            params.with(|p| p.get("path").unwrap_or_default())
        } else {
            qp
        }
    };
    let ref_param = move || query.with(|q| q.get("ref").unwrap_or_default());

    let (commits_data, set_commits_data) = signal(None::<FileCommitsData>);
    let _commits_data = commits_data; // kept for potential future use
    let (commit_entries, set_commit_entries) = signal(Vec::<FileCommitEntry>::new());
    let (commit_summary, set_commit_summary) = signal(String::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    leptos::task::spawn_local(async move {
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        let path_val = path_param();

        if path_val.is_empty() {
            set_error.set(Some("No file path specified.".to_string()));
            set_loading.set(false);
            return;
        }

        let ref_val = ref_param();
        let mut url = format!(
            "/repos/{}/{}/file-commits?path={}",
            owner(),
            name(),
            path_val
        );
        if !ref_val.is_empty() {
            url.push_str(&format!("&ref={ref_val}"));
        }

        match client.get(&url).await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(data) = resp.json::<FileCommitsData>().await {
                    set_commit_entries.set(data.commits.clone());
                    set_commit_summary
                        .set(format!("{} commits affecting {}", data.total, data.path));
                    set_commits_data.set(Some(data));
                } else {
                    set_error.set(Some(sanitize_error("Failed to parse commit history.")));
                }
            }
            Ok(resp) => {
                let status = resp.status();
                set_error.set(Some(sanitize_error(&format!(
                    "Failed to load commits (HTTP {status})."
                ))));
            }
            Err(e) => {
                set_error.set(Some(sanitize_error(&format!("{e}"))));
            }
        }
        set_loading.set(false);
    });

    let dismiss_error = Callback::new(move |_: ()| set_error.set(None));

    view! {
        <div class="space-y-4">
            // Header
            <div>
                <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1">
                    <A href="/repos"><span class="hover:text-blue-600 dark:hover:text-blue-400">"Repositories"</span></A>
                    <span>"/"</span>
                    <A href=format!("/repos/{}/{}", owner(), name())><span class="hover:text-blue-600 dark:hover:text-blue-400">{format!("{}/{}", owner(), name())}</span></A>
                    <span>"/"</span>
                    <A href=format!("/repos/{}/{}/code/{}", owner(), name(), path_param())><span class="hover:text-blue-600 dark:hover:text-blue-400">{path_param()}</span></A>
                    <span>"/"</span>
                    <span class="text-gray-700 dark:text-gray-300">"History"</span>
                </div>
                <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                    "Commits for "{move || path_param()}
                </h1>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=dismiss_error />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center gap-2">
                    <Spinner />
                    <span class="text-gray-500 dark:text-gray-400">"Loading commits..."</span>
                </div>
            </Show>

            <Show when=move || !commit_entries.with(|c| c.is_empty()) fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="mb-3 text-sm text-gray-500 dark:text-gray-400">
                        {move || commit_summary.get()}
                    </div>
                    <div class="divide-y divide-gray-200 dark:divide-gray-700">
                        <For each=move || commit_entries.get() key=|c| c.id.clone() let:commit>
                            {
                                let short_id = if commit.id.len() >= 8 {
                                    commit.id[..8].to_string()
                                } else {
                                    commit.id.clone()
                                };
                                view! {
                                    <div class="py-3 flex items-start gap-3">
                                        <div class="mt-1 shrink-0">
                                            <div class="w-3 h-3 rounded-full bg-blue-500 dark:bg-blue-400"></div>
                                        </div>
                                        <div class="min-w-0 flex-1">
                                            <div class="flex items-center gap-2">
                                                <span class="text-sm font-medium text-blue-600 dark:text-blue-400 font-mono">
                                                    {short_id.clone()}
                                                </span>
                                                <span class="text-sm text-gray-800 dark:text-gray-200 truncate">
                                                    {commit.message.clone()}
                                                </span>
                                            </div>
                                            <div class="mt-0.5 text-xs text-gray-500 dark:text-gray-400">
                                                {commit.author.clone()}" committed on "{commit.time.clone()}
                                            </div>
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
