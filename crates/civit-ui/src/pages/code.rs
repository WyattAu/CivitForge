#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::api::client::ApiClient;
use crate::components::{Card, ErrorBanner, Spinner};
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
    is_root: Signal<bool>,
) -> impl IntoView {
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
                <div class="flex items-center gap-2 mb-3 pb-2 border-b border-gray-200 dark:border-gray-700">
                    <svg class="w-4 h-4 text-gray-500 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                    </svg>
                    <span class="text-sm font-medium text-gray-700 dark:text-gray-300">{move || readme.get().map(|r| r.path.clone()).unwrap_or_default()}</span>
                </div>
                <div class="markdown-body">
                    <pre class="whitespace-pre-wrap text-sm text-gray-800 dark:text-gray-200 break-words">{move || readme.get().map(|r| r.content.clone()).unwrap_or_default()}</pre>
                </div>
            </Card>
        </Show>
    }
}

/// Sub-component: file viewer with syntax highlighting.
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
) -> impl IntoView {
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
                            "/repos/{}/{}/blame?path={}&ref={}",
                            owner.get(),
                            name.get(),
                            file_path.get(),
                            current_ref.get(),
                        )
                    >
                        "Blame"
                    </a>
                    <span class="text-gray-300 dark:text-gray-600">"|"</span>
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
            <div class="bg-gray-50 dark:bg-gray-900/50 rounded-md border border-gray-200 dark:border-gray-700 overflow-x-auto">
                <div class="flex items-center justify-between px-4 py-2 border-b border-gray-200 dark:border-gray-700">
                    <span class="text-sm text-gray-500 dark:text-gray-400 truncate">{move || file_path.get()}</span>
                    <span class="text-xs text-gray-400 dark:text-gray-500">{move || format!(
                        "{}{}",
                        format_size(file_size.get()),
                        if file_is_binary.get() { " (binary)" } else { "" }
                    )}</span>
                </div>
                <pre class="p-4 text-sm text-gray-800 dark:text-gray-200 font-mono whitespace-pre-wrap leading-relaxed tab-size-4"><code data-lang=move || file_lang.get()>{move || file_content.get()}</code></pre>
            </div>
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
                <span>{move || owner.get()}</span>
                <span>"/"</span>
                <span>{move || name.get()}</span>
                <Show when=move || !path.get().is_empty()>
                    <For each=move || breadcrumb_parts(&path.get()) key=|s| s.clone() let:seg>
                        {
                            view! {
                                <span>"/"</span>
                                <span class="text-gray-700 dark:text-gray-300">{seg}</span>
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
                            <th class="pb-2 pr-4 font-medium">"Last Commit"</th>
                            <th class="pb-2 pr-4 font-medium text-right">"Size"</th>
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
                                                        <td class="py-2 pr-4">
                                                            <div class="text-xs">
                                                                <span class="text-gray-700 dark:text-gray-300 truncate block max-w-xs">{commit_msg}</span>
                                                                <span class="text-gray-400 dark:text-gray-500">{commit_author}</span>
                                                                <span class="text-gray-400 dark:text-gray-500 ml-2">{commit_time}</span>
                                                            </div>
                                                        </td>
                                                        <td class="py-2 pr-4 text-right text-gray-400 dark:text-gray-500 font-mono text-xs">
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
    // Derived signals for passing to sub-components
    #[allow(clippy::redundant_closure)]
    let owner_sig = Signal::derive(move || owner());
    #[allow(clippy::redundant_closure)]
    let name_sig = Signal::derive(move || name());
    #[allow(clippy::redundant_closure)]
    let path_sig = Signal::derive(move || path_param());
    let is_root_sig = Signal::derive(move || path_param().is_empty());

    let ref_query = move || query.with(|q| q.get("ref").unwrap_or_default());

    leptos::task::spawn_local(async move {
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        let owner_val = owner();
        let name_val = name();
        let path_val = path_param();
        let ref_val = ref_query();
        set_current_ref.set(ref_val.clone());

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
                        set_file_path_sig.set(blob.path.clone());
                        set_file_size_sig.set(blob.size);
                        set_file_lang_sig.set(blob.language.clone());
                        set_file_content_sig.set(blob.content.clone());
                        set_file_is_binary.set(blob.encoding == "base64");
                    }
                    Err(_) => {
                        set_error.set(Some(sanitize_error("Failed to parse file data.")));
                    }
                },
                Ok(_) => {
                    set_error.set(Some(sanitize_error("Failed to load file content.")));
                }
                Err(e) => {
                    set_error.set(Some(sanitize_error(&format!("{e}"))));
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
                                        format!(
                                            "<div style=\"width: {:.1}%; background-color: {}\" title=\"{}: {:.1}%\" class=\"h-full\"></div>",
                                            width, lang.color, lang.name, lang.percentage
                                        )
                                    })
                                    .collect::<Vec<_>>()
                                    .join("");
                                let legend_html: String = data
                                    .languages
                                    .iter()
                                    .take(8)
                                    .map(|lang| {
                                        format!(
                                            "<span class=\"flex items-center gap-1\"><span class=\"w-2.5 h-2.5 rounded-sm inline-block\" style=\"background-color: {}\"></span><span class=\"font-medium\">{}</span><span class=\"text-gray-400 dark:text-gray-500\">{:.1}%</span></span>",
                                            lang.color, lang.name, lang.percentage
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
                Ok(_) => {
                    set_error.set(Some(sanitize_error("Failed to load repository tree.")));
                }
                Err(e) => {
                    set_error.set(Some(sanitize_error(&format!("{e}"))));
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
                <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1">
                    <A href="/repos"><span class="hover:text-blue-600 dark:hover:text-blue-400">"Repositories"</span></A>
                    <span>"/"</span>
                    <A href=format!("/repos/{}/{}", owner(), name())><span class="hover:text-blue-600 dark:hover:text-blue-400">{format!("{}/{}", owner(), name())}</span></A>
                    <Show when=has_path>
                        <span>"/"</span>
                        <span class="text-gray-700 dark:text-gray-300">{path_param()}</span>
                    </Show>
                    <span>"/"</span>
                    <span class="text-gray-700 dark:text-gray-300">"Code"</span>
                </div>
                <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Code"</h1>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=dismiss_error />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center gap-2">
                    <Spinner />
                    <span class="text-gray-500 dark:text-gray-400">"Loading code..."</span>
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
