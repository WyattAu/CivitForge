#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::api::client::ApiClient;
use crate::components::{Card, ErrorBanner, SkeletonBlock, SkeletonCard};
use crate::state::auth::use_auth;
use crate::utils::*;

#[derive(Clone, serde::Deserialize)]
pub struct TreeEntry {
    pub path: String,
    pub entry_type: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub last_commit: Option<CommitInfo>,
    #[serde(default)]
    pub submodule_url: String,
}

#[derive(Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct CommitInfo {
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
struct BranchInfo {
    name: String,
    #[serde(default)]
    is_default: bool,
}

#[derive(Clone, serde::Deserialize)]
#[allow(dead_code)]
struct PaginatedTreeResponse {
    entries: Vec<TreeEntry>,
    total: usize,
    page: usize,
    per_page: usize,
    total_pages: usize,
}

#[derive(Clone, serde::Deserialize)]
struct BlobData {
    path: String,
    content: String,
    size: u64,
    #[serde(default)]
    encoding: String,
    #[serde(default)]
    language: String,
}

#[derive(Clone, serde::Deserialize)]
#[allow(dead_code)]
struct ReadmeData {
    path: String,
    content: String,
    #[serde(default)]
    encoding: String,
}

#[derive(Clone, serde::Deserialize)]
struct LanguageStatsData {
    languages: Vec<LanguageInfo>,
    #[serde(default)]
    total_bytes: u64,
}

#[derive(Clone, serde::Deserialize)]
#[allow(dead_code)]
struct LanguageInfo {
    name: String,
    bytes: u64,
    percentage: f64,
    #[serde(default)]
    color: String,
}

fn file_icon(entry_type: &str) -> &'static str {
    match entry_type {
        "tree" | "dir" => "dir",
        "submodule" => "submodule",
        _ => "file",
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    if bytes < KB {
        format!("{bytes} B")
    } else if bytes < MB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    }
}

fn breadcrumb_parts(path: &str) -> Vec<String> {
    path.split('/')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn make_code_url(o: &str, n: &str, p: &str, r: &str, page: usize) -> String {
    let mut url = if p.is_empty() {
        format!("/repos/{o}/{n}/code?page={page}")
    } else {
        format!("/repos/{o}/{n}/code?path={p}&page={page}")
    };
    if !r.is_empty() {
        url.push_str(&format!("&ref={r}"));
    }
    url
}

/// Sub-component: repo root overview (language stats bar + README).
/// Extracted from CodePage to keep inner_html closures in a separate view! scope.
#[component]
fn CodeRepoOverview(
    lang_bar: Signal<String>,
    lang_legend: Signal<String>,
    readme: Signal<Option<ReadmeData>>,
    readme_html: Signal<String>,
    is_root: Signal<bool>,
) -> impl IntoView {
    let (show_rendered, set_show_rendered) = signal(true);

    view! {
        // LANGUAGE STATS BAR (root tree view only)
        <Show when=move || !lang_bar.get().is_empty() && is_root.get() fallback=|| view! { <div class="hidden"></div> }>
            <div class="bg-white dark:bg-gray-800 rounded-none shadow-sm border-2 border-gray-200 dark:border-gray-700 px-6 py-4 space-y-2">
                <div class="flex rounded overflow-hidden h-2.5" inner_html=move || lang_bar.get()></div>
                <div class="flex flex-wrap gap-x-3 gap-y-1 text-xs text-gray-600 dark:text-gray-400" inner_html=move || lang_legend.get()></div>
            </div>
        </Show>

        // README RENDERING (root tree view only)
        <Show when=move || readme.get().is_some() && is_root.get() fallback=|| view! { <div class="hidden"></div> }>
            <Card>
                <div class="flex items-center justify-between mb-3 pb-2 border-b border-gray-200 dark:border-gray-700">
                    <div class="flex items-center gap-2">
                        <svg class="w-4 h-4 text-gray-500 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                        </svg>
                        <span class="text-sm font-medium text-gray-700 dark:text-gray-300">{move || readme.get().map(|r| r.path.clone()).unwrap_or_default()}</span>
                    </div>
                    <div class="flex items-center gap-1">
                        <button
                            on:click=move |_| set_show_rendered.set(true)
                            class=move || {
                                if show_rendered.get() {
                                    "px-2 py-1 text-xs font-medium rounded bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300"
                                } else {
                                    "px-2 py-1 text-xs font-medium rounded text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700"
                                }
                            }
                        >
                            "Rendered"
                        </button>
                        <button
                            on:click=move |_| set_show_rendered.set(false)
                            class=move || {
                                if !show_rendered.get() {
                                    "px-2 py-1 text-xs font-medium rounded bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300"
                                } else {
                                    "px-2 py-1 text-xs font-medium rounded text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700"
                                }
                            }
                        >
                            "Raw"
                        </button>
                    </div>
                </div>
                <div class="markdown-body">
                    <Show when=move || show_rendered.get() fallback=move || {
                        view! {
                            <pre class="whitespace-pre-wrap text-sm text-gray-800 dark:text-gray-200 break-words font-mono">{move || readme.get().map(|r| r.content.clone()).unwrap_or_default()}</pre>
                        }.into_any()
                    }>
                        <div inner_html=move || readme_html.get()></div>
                    </Show>
                </div>
            </Card>
        </Show>
    }
}

/// Sub-component: blame line data for inline blame view.
#[derive(Clone, PartialEq, serde::Deserialize)]
struct BlameLineData {
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

const BLAME_AUTHOR_COLORS: &[&str] = &[
    "#3b82f6", "#ef4444", "#22c55e", "#f59e0b", "#8b5cf6", "#ec4899", "#06b6d4", "#f97316",
    "#14b8a6", "#a855f7",
];

/// Sub-component: single blame row with author color coding.
#[component]
fn BlameRow(line: BlameLineData, owner: String, repo: String) -> impl IntoView {
    let short_id = if line.commit_id.len() >= 8 {
        line.commit_id[..8].to_string()
    } else {
        line.commit_id.clone()
    };
    let msg_title = line.commit_message.clone();
    let msg = line.commit_message.clone();
    let author = line.author.clone();
    let time = line.time.clone();
    let content = line.content.clone();
    let line_number = line.line_number;
    let commit_url = format!("/repos/{}/{}/commit/{}", owner, repo, line.commit_id);

    // Compute consistent color from author name using simple hash
    let author_color = {
        let hash: usize = author
            .bytes()
            .fold(0, |acc, b| acc.wrapping_add(b as usize));
        BLAME_AUTHOR_COLORS[hash % BLAME_AUTHOR_COLORS.len()].to_string()
    };

    let (copied, set_copied) = signal(false);
    let content_for_copy = content.clone();

    let copy_line = move |_: leptos::ev::MouseEvent| {
        let text = content_for_copy.clone();
        leptos::task::spawn_local(async move {
            // Use js_sys eval for clipboard write
            let escaped = text
                .replace('\\', "\\\\")
                .replace('\'', "\\'")
                .replace('\n', "\\n")
                .replace('\r', "\\r");
            let js_code = format!("navigator.clipboard.writeText('{escaped}')");
            let _ = js_sys::eval(&js_code);
            set_copied.set(true);
            let _ = js_sys::Promise::resolve(
                &js_sys::eval("new Promise(r => setTimeout(r, 1500))").unwrap(),
            )
            .await;
            set_copied.set(false);
        });
    };

    view! {
        <tr class="hover:bg-gray-100 dark:hover:bg-gray-800/50 group/line">
            <td class="py-0.5 pl-4 pr-2 text-gray-400 dark:text-gray-500 text-right select-none">
                {line_number}
            </td>
            <td class="py-0.5 pr-2 text-xs">
                <a
                    class="font-mono hover:underline cursor-pointer"
                    style=format!("color: {author_color}")
                    href=commit_url
                    title=msg_title
                >
                    {short_id}
                </a>
                <span class="ml-1 text-gray-500 dark:text-gray-400 truncate block max-w-[200px]">{msg}</span>
            </td>
            <td class="py-0.5 pr-2 text-xs whitespace-nowrap">
                <span style=format!("color: {author_color}")>{author}</span>
                <span class="ml-1 text-gray-400 dark:text-gray-500">{time}</span>
            </td>
            <td class="py-0.5 pr-4 whitespace-pre relative">
                <code>{content}</code>
                <button
                    class="opacity-0 group-hover/line:opacity-100 absolute right-2 top-1/2 -translate-y-1/2 px-1.5 py-0.5 text-[10px] font-medium rounded transition-all bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600"
                    on:click=copy_line
                    aria-label="Copy to clipboard"
                >
                    {move || if copied.get() { "Copied!" } else { "Copy" }}
                </button>
            </td>
        </tr>
    }
}

/// Sub-component: collapsible blame section grouped by commit.
#[derive(Clone, PartialEq)]
struct BlameSectionData {
    commit_id: String,
    commit_message: String,
    author: String,
    time: String,
    lines: Vec<BlameLineData>,
}

#[component]
fn BlameSectionView(
    lines: Signal<Vec<BlameLineData>>,
    owner: Signal<String>,
    repo: Signal<String>,
) -> impl IntoView {
    let sections = Memo::new(move |_| {
        let all_lines = lines.get();
        let mut groups: Vec<BlameSectionData> = Vec::new();
        let mut last_commit: Option<String> = None;

        for line in all_lines {
            if last_commit.as_ref() != Some(&line.commit_id) {
                groups.push(BlameSectionData {
                    commit_id: line.commit_id.clone(),
                    commit_message: line.commit_message.clone(),
                    author: line.author.clone(),
                    time: line.time.clone(),
                    lines: Vec::new(),
                });
                last_commit = Some(line.commit_id.clone());
            }
            if let Some(group) = groups.last_mut() {
                group.lines.push(line);
            }
        }
        groups
    });

    view! {
        <div class="bg-gray-50 dark:bg-gray-900/50 rounded-md border border-gray-200 dark:border-gray-700 overflow-x-auto">
            <table class="w-full text-sm font-mono">
                <thead>
                    <tr class="border-b border-gray-200 dark:border-gray-700 text-gray-500 dark:text-gray-400 text-xs">
                        <th class="pb-2 pl-4 pr-2 text-left w-8 font-medium">"#"</th>
                        <th class="pb-2 pr-2 text-left font-medium">"Commit"</th>
                        <th class="pb-2 pr-2 text-left font-medium">"Author"</th>
                        <th class="pb-2 pr-4 text-left font-medium">"Code"</th>
                    </tr>
                </thead>
                <For
                    each=move || {
                        sections.get().into_iter().enumerate().collect::<Vec<_>>()
                    }
                    key=|s| format!("{}-{}", s.0, s.1.commit_id)
                    let:item>
                    <BlameSectionRow item=item owner=owner repo=repo />
                </For>
            </table>
        </div>
    }
}

/// Sub-component: single blame section row with header and collapsible lines.
#[component]
fn BlameSectionRow(
    item: (usize, BlameSectionData),
    owner: Signal<String>,
    repo: Signal<String>,
) -> impl IntoView {
    let (_idx, section) = item;
    let owner_val = owner.get();
    let repo_val = repo.get();
    let short_id = if section.commit_id.len() >= 8 {
        section.commit_id[..8].to_string()
    } else {
        section.commit_id.clone()
    };
    let commit_url = format!(
        "/repos/{}/{}/commit/{}",
        owner_val, repo_val, section.commit_id
    );
    let (collapsed, set_collapsed) = signal(false);
    let author_color = {
        let hash: usize = section
            .author
            .bytes()
            .fold(0, |acc, b| acc.wrapping_add(b as usize));
        BLAME_AUTHOR_COLORS[hash % BLAME_AUTHOR_COLORS.len()].to_string()
    };
    let line_count = section.lines.len();
    let section_lines_sv = StoredValue::new(section.lines);

    view! {
        <BlameSectionRowHeader
            collapsed=collapsed
            set_collapsed=set_collapsed
            commit_url=commit_url
            short_id=short_id
            commit_message=section.commit_message
            author=section.author
            time=section.time
            author_color=author_color
            line_count=line_count
        />
        <BlameSectionRowLines
            collapsed=collapsed
            section_lines_sv=section_lines_sv
            owner_val=owner_val
            repo_val=repo_val
        />
    }
}

#[component]
fn BlameSectionRowHeader(
    collapsed: ReadSignal<bool>,
    set_collapsed: WriteSignal<bool>,
    commit_url: String,
    short_id: String,
    commit_message: String,
    author: String,
    time: String,
    author_color: String,
    line_count: usize,
) -> impl IntoView {
    view! {
        <tr
            class="border-t border-gray-200 dark:border-gray-700 cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800/50"
            on:click=move |_| set_collapsed.update(|c| *c = !*c)
        >
            <td class="py-1 pl-4 pr-2" colspan="4">
                <div class="flex items-center gap-3">
                    <svg class={move || format!("w-3 h-3 text-gray-400 transition-transform {}", if collapsed.get() { "" } else { "rotate-90" })} fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                    </svg>
                    <a
                        class="font-mono text-xs font-semibold hover:underline"
                        style=format!("color: {author_color}")
                        href=commit_url
                    >
                        {short_id}
                    </a>
                    <span class="text-xs text-gray-700 dark:text-gray-300 truncate max-w-xs">{commit_message}</span>
                    <span class="text-xs whitespace-nowrap" style=format!("color: {author_color}")>{author}</span>
                    <span class="text-xs text-gray-400 dark:text-gray-500 whitespace-nowrap">{time}</span>
                    <span class="text-xs text-gray-400 dark:text-gray-500 whitespace-nowrap">
                        {format!("{} line{}", line_count, if line_count != 1 { "s" } else { "" })}
                    </span>
                </div>
            </td>
        </tr>
    }
}

#[component]
fn BlameSectionRowLines(
    collapsed: ReadSignal<bool>,
    section_lines_sv: StoredValue<Vec<BlameLineData>>,
    owner_val: String,
    repo_val: String,
) -> impl IntoView {
    let owner_sv = StoredValue::new(owner_val);
    let repo_sv = StoredValue::new(repo_val);
    view! {
        <Show when=move || !collapsed.get() fallback=|| view! { <div class="hidden"></div> }>
            <For
                each=move || section_lines_sv.get_value()
                key=|l| l.line_number
                let:line>
                <BlameRow line=line owner=owner_sv.get_value() repo=repo_sv.get_value() />
            </For>
        </Show>
    }
}

/// Sub-component: file viewer with syntax highlighting and blame tab.
/// Extracted from CodePage to reduce closure count in main view! macro.
#[component]
fn CodeFileViewer(
    owner: Signal<String>,
    name: Signal<String>,
    file_path: Signal<String>,
    file_size: Signal<u64>,
    file_lang: Signal<String>,
    file_content: Signal<String>,
    file_is_binary: Signal<bool>,
    current_ref: Signal<String>,
    auth_token: Signal<String>,
) -> impl IntoView {
    let (active_view, set_active_view) = signal(String::from("code"));
    let (blame_lines, set_blame_lines) = signal(Vec::<BlameLineData>::new());
    let (blame_loading, set_blame_loading) = signal(false);
    let (blame_error, set_blame_error) = signal(None::<String>);

    let fetch_blame = move || {
        let owner_val = owner.get();
        let name_val = name.get();
        let path_val = file_path.get();
        let ref_val = current_ref.get();
        let token_val = auth_token.get();

        set_blame_loading.set(true);
        set_blame_error.set(None);

        leptos::task::spawn_local(async move {
            let client = crate::api::client::ApiClient::new(Some(token_val));
            let mut url = format!("/repos/{owner_val}/{name_val}/blame?path={path_val}");
            if !ref_val.is_empty() {
                url.push_str(&format!("&ref={ref_val}"));
            }
            match client.get(&url).await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<serde_json::Value>().await {
                        if let Some(lines) = data.get("lines").and_then(|l| l.as_array()) {
                            let parsed: Vec<BlameLineData> = lines
                                .iter()
                                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                                .collect();
                            set_blame_lines.set(parsed);
                        }
                    } else {
                        set_blame_error.set(Some("Failed to parse blame data.".into()));
                    }
                }
                Ok(resp) => {
                    let status = resp.status();
                    set_blame_error.set(Some(format!("Blame request failed (HTTP {status}).")));
                }
                Err(e) => {
                    set_blame_error.set(Some(format!("Network error: {e}")));
                }
            }
            set_blame_loading.set(false);
        });
    };

    let on_blame_click = move |_| {
        set_active_view.set("blame".into());
        if blame_lines.get().is_empty() {
            fetch_blame();
        }
    };

    view! {
        <Card>
            <div class="mb-3 flex items-center justify-between">
                <div class="text-sm text-gray-500 dark:text-gray-400 font-mono truncate">
                    {move || file_path.get()}
                </div>
                <div class="flex items-center gap-2">
                    <a
                        class="text-xs text-blue-600 dark:text-blue-400 hover:underline"
                        href=move || format!(
                            "/repos/{}/{}/commits?path={}&ref={}",
                            owner.get(),
                            name.get(),
                            file_path.get(),
                            current_ref.get(),
                        )
                    >
                        "History"
                    </a>
                </div>
            </div>

            // Tab navigation
            <div class="border-b border-gray-200 dark:border-gray-700 mb-0">
                <nav class="-mb-px flex space-x-6" aria-label="File view tabs">
                    <button
                        class=move || format!(
                            "px-3 py-2 text-sm font-medium border-b-2 transition-colors {}",
                            if active_view.get() == "code" {
                                "border-blue-600 text-blue-600 dark:border-blue-400 dark:text-blue-400"
                            } else {
                                "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-200"
                            }
                        )
                        on:click=move |_| set_active_view.set("code".into())
                    >
                        "Code"
                    </button>
                    <button
                        class=move || format!(
                            "px-3 py-2 text-sm font-medium border-b-2 transition-colors {}",
                            if active_view.get() == "blame" {
                                "border-blue-600 text-blue-600 dark:border-blue-400 dark:text-blue-400"
                            } else {
                                "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-200"
                            }
                        )
                        on:click=on_blame_click
                    >
                        "Blame"
                    </button>
                </nav>
            </div>

            // Code view
            <Show when=move || active_view.get() == "code" fallback=|| view! { <div class="hidden"></div> }>
                <div class="bg-gray-50 dark:bg-gray-900/50 rounded-md border border-gray-200 dark:border-gray-700 overflow-x-auto mt-3">
                    <div class="flex items-center justify-between px-4 py-2 border-b border-gray-200 dark:border-gray-700">
                        <div class="flex items-center gap-2">
                            <span class="text-sm text-gray-500 dark:text-gray-400 truncate">{move || file_path.get()}</span>
                            <Show when=move || !file_lang.get().is_empty()>
                                <span class="px-1.5 py-0.5 text-[10px] font-medium rounded bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-300 uppercase">
                                    {move || file_lang.get()}
                                </span>
                            </Show>
                        </div>
                        <span class="text-xs text-gray-400 dark:text-gray-500">{move || format!(
                            "{}{}",
                            format_size(file_size.get()),
                            if file_is_binary.get() { " (binary)" } else { "" }
                        )}</span>
                    </div>
                    <pre class="p-4 text-sm text-gray-800 dark:text-gray-200 font-mono whitespace-pre-wrap leading-relaxed tab-size-4"><code class=move || format!("language-{}", file_lang.get()) data-lang=move || file_lang.get()>{move || file_content.get()}</code></pre>
                </div>
            </Show>

            // Blame view
            <Show when=move || active_view.get() == "blame" fallback=|| view! { <div class="hidden"></div> }>
                <div class="mt-3">
                    <Show when=move || blame_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        <div class="flex items-center gap-2 py-8 justify-center">
                            <svg class="animate-spin h-5 w-5 text-gray-400" fill="none" viewBox="0 0 24 24">
                                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                            </svg>
                            <span class="text-sm text-gray-500">"Loading blame..."</span>
                        </div>
                    </Show>
                    <Show when=move || blame_error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                        <div class="rounded border border-red-300 bg-red-50 dark:bg-red-900/20 dark:border-red-700 p-3 text-sm text-red-800 dark:text-red-200">
                            {move || blame_error.get().unwrap_or_default()}
                        </div>
                    </Show>
                    <Show when=move || !blame_loading.get() && blame_error.get().is_none() && !blame_lines.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                        <BlameSectionView
                            lines=blame_lines.into()
                            owner=owner
                            repo=name
                        />
                    </Show>
                </div>
            </Show>
        </Card>
    }
}

/// Sub-component: tree table with pagination.
/// Extracted from CodePage to stay under Leptos view! macro closure limit.
/// Takes signals (not raw values) to avoid FnOnce issues in For loops.
#[component]
pub fn CodeTreeTable(
    owner: Signal<String>,
    name: Signal<String>,
    path: Signal<String>,
    current_ref: Signal<String>,
    entries: Signal<Vec<TreeEntry>>,
    total_entries: Signal<usize>,
    current_page: Signal<usize>,
    total_pages: Signal<usize>,
) -> impl IntoView {
    let show_pagination = move || total_pages.get() > 1;

    view! {
        <Card>
            // Breadcrumb
            <div class="mb-3 flex items-center gap-1 text-sm text-gray-500 dark:text-gray-400 font-mono overflow-x-auto">
                <span class="shrink-0">{move || owner.get()}</span>
                <span class="shrink-0">"/"</span>
                <span class="shrink-0">{move || name.get()}</span>
                <Show when=move || !path.get().is_empty()>
                    <For each=move || breadcrumb_parts(&path.get()) key=|s| s.clone() let:seg>
                        {
                            view! {
                                <span class="shrink-0">"/"</span>
                                <span class="text-gray-700 dark:text-gray-300 shrink-0">{seg}</span>
                            }
                        }
                    </For>
                </Show>
            </div>

            <div class="overflow-x-auto">
                <table class="w-full text-sm text-left">
                    <thead>
                        <tr class="border-b border-gray-200 dark:border-gray-700 text-gray-500 dark:text-gray-400">
                            <th class="pb-2 pr-4 font-medium">"Name"</th>
                            <th class="pb-2 pr-4 font-medium hidden md:table-cell">"Last Commit"</th>
                            <th class="pb-2 pr-4 font-medium text-right hidden sm:table-cell">"Size"</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
                        <For each=move || entries.get() key=|e| e.path.clone() let:entry>
                            {
                                let entry_type = entry.entry_type.clone();
                                let path_c = path.get();
                                let full_path = if path_c.is_empty() {
                                    entry.path.clone()
                                } else {
                                    format!("{}/{}", path_c, entry.path)
                                };
                                let entry_path = entry.path.clone();
                                let icon = file_icon(&entry_type);
                                let is_dir = entry_type == "tree" || entry_type == "dir";
                                let cur_ref = current_ref.get();
                                let commit_info = entry.last_commit.clone();
                                let commit_msg = commit_info
                                    .as_ref()
                                    .map(|c| c.message.clone())
                                    .unwrap_or_default();
                                let commit_author = commit_info
                                    .as_ref()
                                    .map(|c| c.author.clone())
                                    .unwrap_or_else(|| "-".to_string());
                                let commit_time = commit_info
                                    .as_ref()
                                    .map(|c| c.time.clone())
                                    .unwrap_or_default();
                                let entry_size = entry.size;
                                view! {
                                    <A href=format!(
                                        "/repos/{}/{}/code/{}{}",
                                        owner.get(),
                                        name.get(),
                                        full_path,
                                        if cur_ref.is_empty() {
                                            String::new()
                                        } else {
                                            format!("?ref={cur_ref}")
                                        }
                                    )>
                                        <tr class="hover:bg-gray-50 dark:hover:bg-gray-750 cursor-pointer">
                                            <td class="py-2 pr-4">
                                                            <div class="flex items-center gap-2">
                                                                 {move || match icon {
                                                                     "dir" => view! {
                                                                         <svg class="w-4 h-4 text-blue-500 dark:text-blue-400 shrink-0" fill="currentColor" viewBox="0 0 20 20">
                                                                             <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"/>
                                                                         </svg>
                                                                     }.into_any(),
                                                                     "submodule" => view! {
                                                                         <svg class="w-4 h-4 text-purple-500 dark:text-purple-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                             <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
                                                                         </svg>
                                                                     }.into_any(),
                                                                     _ => view! {
                                                                         <svg class="w-4 h-4 text-gray-400 dark:text-gray-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                             <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                                                                         </svg>
                                                                     }.into_any(),
                                                                 }}
                                                                <span class="text-gray-700 dark:text-gray-300 truncate">{entry_path}</span>
                                                            </div>
                                            </td>
                                                        <td class="py-2 pr-4 hidden md:table-cell">
                                                            <div class="text-xs">
                                                                <span class="text-gray-700 dark:text-gray-300 truncate block max-w-xs">{commit_msg}</span>
                                                                <span class="text-gray-400 dark:text-gray-500">{commit_author}</span>
                                                                <span class="text-gray-400 dark:text-gray-500 ml-2">{commit_time}</span>
                                                            </div>
                                                        </td>
                                                        <td class="py-2 pr-4 text-right text-gray-400 dark:text-gray-500 font-mono text-xs hidden sm:table-cell">
                                                            {if is_dir { "-".to_string() } else { format_size(entry_size) }}
                                                        </td>
                                        </tr>
                                    </A>
                                }
                            }
                        </For>
                    </tbody>
                </table>
            </div>

            <Show when=show_pagination fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center justify-between mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
                    <span class="text-sm text-gray-500 dark:text-gray-400">
                        {move || format!(
                            "Showing {} of {} entries (page {}/{})",
                            entries.with(|t| t.len()),
                            total_entries.get(),
                            current_page.get(),
                            total_pages.get()
                        )}
                    </span>
                    <div class="flex items-center gap-2">
                        {move || {
                            let page = current_page.get();
                            let prev = page.saturating_sub(1);
                            if prev >= 1 {
                                let href = make_code_url(&owner.get(), &name.get(), &path.get(), &current_ref.get(), prev);
                                view! {
                                    <a class="px-3 py-1.5 text-sm rounded border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors cursor-pointer" href=href>"← Prev"</a>
                                }.into_any()
                            } else {
                                view! { <span class="px-3 py-1.5 text-sm rounded border border-gray-300 dark:border-gray-600 opacity-40 cursor-not-allowed">"← Prev"</span> }.into_any()
                            }
                        }}
                        <span class="text-sm text-gray-600 dark:text-gray-300">
                            {move || format!("{}", current_page.get())}
                        </span>
                        {move || {
                            let page = current_page.get();
                            let tp = total_pages.get();
                            let next = page + 1;
                            if next <= tp {
                                let href = make_code_url(&owner.get(), &name.get(), &path.get(), &current_ref.get(), next);
                                view! {
                                    <a class="px-3 py-1.5 text-sm rounded border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors cursor-pointer" href=href>"Next →"</a>
                                }.into_any()
                            } else {
                                view! { <span class="px-3 py-1.5 text-sm rounded border border-gray-300 dark:border-gray-600 opacity-40 cursor-not-allowed">"Next →"</span> }.into_any()
                            }
                        }}
                    </div>
                </div>
            </Show>
        </Card>
    }
}

#[component]
pub fn CodePage() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let path_param = move || {
        let query_path = query.with(|q| q.get("path").unwrap_or_default());
        if query_path.is_empty() {
            params.with(|p| p.get("path").unwrap_or_default())
        } else {
            query_path
        }
    };
    let auth = use_auth();
    let page_query = move || {
        query.with(|q| {
            q.get("page")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(1)
        })
    };

    let (tree_entries, set_tree_entries) = signal(Vec::<TreeEntry>::new());
    let (readme_data, set_readme_data) = signal(None::<ReadmeData>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (lang_bar_html, set_lang_bar_html) = signal(String::new());
    let (lang_legend_html, set_lang_legend_html) = signal(String::new());
    let (total_pages, set_total_pages) = signal(0usize);
    let (current_page, set_current_page) = signal(1usize);
    let (total_entries, set_total_entries) = signal(0usize);
    let (branches, set_branches) = signal(Vec::<BranchInfo>::new());
    let (current_ref, set_current_ref) = signal(String::new());
    let (file_path_sig, set_file_path_sig) = signal(String::new());
    let (file_size_sig, set_file_size_sig) = signal(0u64);
    let (file_lang_sig, set_file_lang_sig) = signal(String::new());
    let (file_content_sig, set_file_content_sig) = signal(String::new());
    let (file_is_binary, set_file_is_binary) = signal(false);
    let (readme_html, set_readme_html) = signal(String::new());
    // Derived signals for passing to sub-components
    #[allow(clippy::redundant_closure)]
    let owner_sig = Signal::derive(move || owner());
    #[allow(clippy::redundant_closure)]
    let name_sig = Signal::derive(move || name());
    #[allow(clippy::redundant_closure)]
    let path_sig = Signal::derive(move || path_param());
    let is_root_sig = Signal::derive(move || path_param().is_empty());
    let auth_token_sig =
        Signal::derive(move || auth.0.with(|a| a.token.clone()).unwrap_or_default());

    let ref_query = move || query.with(|q| q.get("ref").unwrap_or_default());

    #[cfg(feature = "csr")]
    Effect::new(move |_| {
        let content = file_content_sig.get();
        let path = file_path_sig.get();
        if !content.is_empty() && !path.is_empty() {
            // Small delay to ensure DOM is updated
            let _ = js_sys::eval("setTimeout(function() { hljs.highlightAll(); }, 50)");
        }
    });

    leptos::task::spawn_local(async move {
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        let owner_val = owner();
        let name_val = name();
        let path_val = path_param();
        let ref_val = ref_query();
        set_current_ref.set(ref_val.clone());

        inject_highlight_js();
        inject_marked_js();
        inject_katex_js();
        inject_mermaid_js();

        // Fetch branch list in background
        let branches_client = client.clone();
        let branches_owner = owner_val.clone();
        let branches_name = name_val.clone();
        leptos::task::spawn_local(async move {
            let url = format!("/repos/{branches_owner}/{branches_name}/branches");
            match branches_client.get(&url).await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(branch_list) = resp.json::<Vec<BranchInfo>>().await {
                        set_branches.set(branch_list);
                    }
                }
                _ => {}
            }
        });

        let is_file = !path_val.is_empty() && path_val.contains('.');

        if is_file {
            let mut blob_url = format!("/repos/{owner_val}/{name_val}/blob?path={path_val}");
            if !ref_val.is_empty() {
                blob_url.push_str(&format!("&ref={ref_val}"));
            }
            match client.get(&blob_url).await {
                Ok(resp) if resp.status().is_success() => match resp.json::<BlobData>().await {
                    Ok(blob) => {
                        let detected = if blob.language.is_empty() {
                            detect_language(&blob.path).to_string()
                        } else {
                            blob.language.clone()
                        };
                        set_file_path_sig.set(blob.path.clone());
                        set_file_size_sig.set(blob.size);
                        set_file_lang_sig.set(detected);
                        set_file_content_sig.set(blob.content.clone());
                        set_file_is_binary.set(blob.encoding == "base64");
                    }
                    Err(_) => {
                        set_error.set(Some(sanitize_error("Failed to parse file data.")));
                    }
                },
                Ok(resp) => {
                    let status = resp.status();
                    let msg = if status == 401 || status == 403 {
                        "Session expired. Please sign in again."
                    } else if status == 404 {
                        "Resource not found."
                    } else if status.as_u16() >= 500 {
                        "Something went wrong. Please try again."
                    } else {
                        "Failed to load file content."
                    };
                    set_error.set(Some(sanitize_error(msg)));
                }
                Err(_) => {
                    set_error.set(Some(sanitize_error("Connection failed. Check your internet.")));
                }
            }
        } else {
            // Tree view — fetch README and language stats for root
            if path_val.is_empty() {
                let readme_client = client.clone();
                let readme_owner = owner_val.clone();
                let readme_name = name_val.clone();
                leptos::task::spawn_local(async move {
                    let url = format!("/repos/{readme_owner}/{readme_name}/readme");
                    match readme_client.get(&url).await {
                        Ok(resp) if resp.status().is_success() => {
                            if let Ok(data) = resp.json::<ReadmeData>().await {
                                let html = render_markdown(&data.content);
                                if !html.is_empty() {
                                    set_readme_html.set(html);
                                }
                                set_readme_data.set(Some(data));
                            }
                        }
                        _ => {}
                    }
                });

                let stats_client = client.clone();
                let stats_owner = owner_val.clone();
                let stats_name = name_val.clone();
                leptos::task::spawn_local(async move {
                    let url = format!("/repos/{stats_owner}/{stats_name}/languages");
                    match stats_client.get(&url).await {
                        Ok(resp) if resp.status().is_success() => {
                            if let Ok(data) = resp.json::<LanguageStatsData>().await {
                                let total = data.total_bytes;
                                let bar_html: String = data
                                    .languages
                                    .iter()
                                    .take(8)
                                    .map(|lang| {
                                        let width = if total > 0 { lang.percentage } else { 0.0 };
                                        let color = if lang.color.is_empty() {
                                            language_color(&lang.name).to_string()
                                        } else {
                                            lang.color.clone()
                                        };
                                        format!(
                                            "<div style=\"width: {:.1}%; background-color: {}\" title=\"{}: {:.1}%\" class=\"h-full\"></div>",
                                            width, color, lang.name, lang.percentage
                                        )
                                    })
                                    .collect::<Vec<_>>()
                                    .join("");
                                let legend_html: String = data
                                    .languages
                                    .iter()
                                    .take(8)
                                    .map(|lang| {
                                        let color = if lang.color.is_empty() {
                                            language_color(&lang.name).to_string()
                                        } else {
                                            lang.color.clone()
                                        };
                                        format!(
                                            "<span class=\"flex items-center gap-1\"><span class=\"w-2.5 h-2.5 rounded-sm inline-block\" style=\"background-color: {}\"></span><span class=\"font-medium\">{}</span><span class=\"text-gray-400 dark:text-gray-500\">{:.1}%</span></span>",
                                            color, lang.name, lang.percentage
                                        )
                                    })
                                    .collect::<Vec<_>>()
                                    .join("");
                                set_lang_bar_html.set(bar_html);
                                set_lang_legend_html.set(legend_html);
                            }
                        }
                        _ => {}
                    }
                });
            }

            let page_val = page_query();
            let mut tree_url = if path_val.is_empty() {
                format!("/repos/{owner_val}/{name_val}/tree?page={page_val}")
            } else {
                format!("/repos/{owner_val}/{name_val}/tree?path={path_val}&page={page_val}")
            };
            if !ref_val.is_empty() {
                tree_url.push_str(&format!("&ref={ref_val}"));
            }
            match client.get(&tree_url).await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<PaginatedTreeResponse>().await {
                        Ok(data) => {
                            set_tree_entries.set(data.entries);
                            set_total_pages.set(data.total_pages);
                            set_total_entries.set(data.total);
                            set_current_page.set(data.page);
                        }
                        Err(_) => set_error.set(Some(sanitize_error("Failed to parse tree data."))),
                    }
                }
                Ok(resp) => {
                    let status = resp.status();
                    let msg = if status == 401 || status == 403 {
                        "Session expired. Please sign in again."
                    } else if status == 404 {
                        "Resource not found."
                    } else if status.as_u16() >= 500 {
                        "Something went wrong. Please try again."
                    } else {
                        "Failed to load repository tree."
                    };
                    set_error.set(Some(sanitize_error(msg)));
                }
                Err(_) => {
                    set_error.set(Some(sanitize_error("Connection failed. Check your internet.")));
                }
            }
        }
        set_loading.set(false);
    });

    let has_tree = move || !tree_entries.with(|t| t.is_empty());
    let showing_file = move || !file_path_sig.with(|p| p.is_empty());
    let has_path = move || !path_param().is_empty();

    let dismiss_error = Callback::new(move |_: ()| set_error.set(None));

    view! {
        <div class="space-y-6">
            // Header
            <div>
                <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1 overflow-x-auto">
                    <A href="/repos"><span class="hover:text-blue-600 dark:hover:text-blue-400 shrink-0">"Repositories"</span></A>
                    <span class="shrink-0">"/"</span>
                    <A href=format!("/repos/{}/{}", owner(), name())><span class="hover:text-blue-600 dark:hover:text-blue-400 shrink-0">{format!("{}/{}", owner(), name())}</span></A>
                    <Show when=has_path>
                        <span class="shrink-0">"/"</span>
                        <span class="text-gray-700 dark:text-gray-300 truncate">{path_param()}</span>
                    </Show>
                    <span class="shrink-0">"/"</span>
                    <span class="text-gray-700 dark:text-gray-300 shrink-0">"Code"</span>
                </div>
                <h1 class="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-gray-100">"Code"</h1>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=dismiss_error />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="space-y-4">
                    <SkeletonBlock class="h-10 w-full".to_string() />
                    <div class="space-y-1">
                        <For each=move || 0..10usize key=|i| *i let:_i>
                            <SkeletonCard />
                        </For>
                    </div>
                </div>
            </Show>

            // Branch selector dropdown
            <Show when=move || !branches.with(|b| b.is_empty()) fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center gap-2">
                    <svg class="w-4 h-4 text-gray-500 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>
                    </svg>
                    <select
                        data-branch-selector=""
                        class="text-sm border border-gray-300 dark:border-gray-600 rounded-md px-3 py-1.5 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                    >
                        <For each=move || branches.get() key=|b| b.name.clone() let:branch>
                            {
                                let selected = branch.name == current_ref.get();
                                let display_name = if branch.is_default {
                                    format!("{} (default)", branch.name)
                                } else {
                                    branch.name.clone()
                                };
                                view! {
                                    <option value=branch.name.clone() selected=selected>
                                        {display_name}
                                    </option>
                                }
                            }
                        </For>
                    </select>
                </div>
            </Show>

            // Empty repo state
            <Show when=move || !loading.get() && !has_tree() && !showing_file() && error.get().is_none() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="text-center py-12">
                        <svg class="mx-auto h-12 w-12 text-gray-300 dark:text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"/>
                        </svg>
                        <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-gray-100">"No files found"</h3>
                        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                            "Push a commit to see the repository file tree."
                        </p>
                    </div>
                </Card>
            </Show>

            <div>
                // === REPO OVERVIEW (lang stats + readme) ===
                <CodeRepoOverview
                    lang_bar=lang_bar_html.into()
                    lang_legend=lang_legend_html.into()
                    readme=readme_data.into()
                    readme_html=readme_html.into()
                    is_root=is_root_sig
                />

                // === FILE VIEWER with syntax highlighting ===
                <Show when=move || !loading.get() && showing_file() fallback=|| view! { <div class="hidden"></div> }>
                    <CodeFileViewer
                        owner=owner_sig
                        name=name_sig
                        file_path=file_path_sig.into()
                        file_size=file_size_sig.into()
                        file_lang=file_lang_sig.into()
                        file_content=file_content_sig.into()
                        file_is_binary=file_is_binary.into()
                        current_ref=current_ref.into()
                        auth_token=auth_token_sig
                    />
                </Show>

                // === TREE VIEW (delegated to sub-component) ===
                <Show when=move || !loading.get() && !showing_file() && !tree_entries.with(|t| t.is_empty()) fallback=|| view! { <div class="hidden"></div> }>
                    <CodeTreeTable
                        owner=owner_sig
                        name=name_sig
                        path=path_sig
                        current_ref=current_ref.into()
                        entries=tree_entries.into()
                        total_entries=total_entries.into()
                        current_page=current_page.into()
                        total_pages=total_pages.into()
                    />
                </Show>
            </div>
        </div>
    }
}
