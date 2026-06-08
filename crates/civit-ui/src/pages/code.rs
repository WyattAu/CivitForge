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

#[allow(dead_code)]
#[derive(Clone, serde::Deserialize)]
struct BlobData {
    path: String,
    content: String,
    size: u64,
    #[serde(default)]
    encoding: String,
}

fn file_icon(entry_type: &str) -> &'static str {
    match entry_type {
        "tree" | "dir" => "dir",
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

#[component]
pub fn CodePage() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    // Path from query param (?path=src) takes priority over wildcard path segment (/code/src)
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
    let (file_content, set_file_content) = signal(None::<BlobData>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (total_pages, set_total_pages) = signal(0usize);
    let (current_page, set_current_page) = signal(1usize);
    let (total_entries, set_total_entries) = signal(0usize);
    let (branches, set_branches) = signal(Vec::<BranchInfo>::new());
    let (current_ref, set_current_ref) = signal(String::new());

    // Read ref from query param (?ref=branch-name)
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
                _ => {} // Non-critical, don't show error for branch list failure
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
                    Ok(blob) => set_file_content.set(Some(blob)),
                    Err(_) => {
                        set_error.set(Some(sanitize_error("Failed to parse file data.")));
                    }
                },
                Ok(_) => {
                    set_error.set(Some(sanitize_error("Failed to load file content.")));
                }
                Err(e) => {
                    let msg = format!("{e}");
                    set_error.set(Some(sanitize_error(&msg)));
                }
            }
        } else {
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
                    let msg = format!("{e}");
                    set_error.set(Some(sanitize_error(&msg)));
                }
            }
        }
        set_loading.set(false);
    });

    let has_tree = move || !tree_entries.with(|t| t.is_empty());
    let showing_file = move || file_content.get().is_some();
    let has_path = move || !path_param().is_empty();
    let show_pagination = move || total_pages.get() > 1;

    // Helper to build code URL with ref preserved
    let code_url_with_ref = move |extra_path: String, page: usize| {
        let o = owner();
        let n = name();
        let r = current_ref.get();
        let p = if extra_path.is_empty() {
            path_param()
        } else {
            extra_path
        };
        let mut url = if p.is_empty() {
            format!("/repos/{o}/{n}/code?page={page}")
        } else {
            format!("/repos/{o}/{n}/code?path={p}&page={page}")
        };
        if !r.is_empty() {
            url.push_str(&format!("&ref={r}"));
        }
        url
    };

    let dismiss_error = Callback::new(move |_: ()| set_error.set(None));

    view! {
        <div class="space-y-6">
            <div>
                <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1">
                    <A href="/repos"><span class="hover:text-blue-600 dark:hover:text-blue-400">"Repositories"</span></A>
                    <span>"/"</span>
                    <A href=format!("/repos/{}/{}", owner(), name())><span class="hover:text-blue-600 dark:hover:text-blue-400">{format!("{}/{}", owner(), name())}</span></A>
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

            // Branch selector dropdown — uses data-branch-selector attribute
            // with JS-attached onchange handler (WebKit safe, avoids on:click)
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
                    <Show when=move || !loading.get() && showing_file() fallback=|| view! { <div class="hidden"></div> }>
                        <Card>
                            <div class="mb-3 flex items-center gap-1 text-sm text-gray-500 dark:text-gray-400 font-mono overflow-x-auto">
                                <span>{owner()}</span>
                                <span>"/"</span>
                                <span>{name()}</span>
                                <Show when=has_path>
                                    <For each=move || breadcrumb_parts(&path_param()) key=|s| s.clone() let:seg>
                                        {
                                            view! {
                                                <span>"/"</span>
                                                <span class="text-gray-700 dark:text-gray-300">{seg}</span>
                                            }
                                        }
                                    </For>
                                </Show>
                            </div>
                            {file_content.get().map(|blob| {
                                let path = blob.path.clone();
                                let size = blob.size;
                                let content = blob.content.clone();
                                view! {
                                    <div class="bg-gray-50 dark:bg-gray-900/50 rounded-md border border-gray-200 dark:border-gray-700 overflow-x-auto">
                                        <div class="flex items-center justify-between px-4 py-2 border-b border-gray-200 dark:border-gray-700">
                                            <span class="text-sm text-gray-500 dark:text-gray-400 truncate">{path}</span>
                                            <span class="text-xs text-gray-400 dark:text-gray-500">{format_size(size)}</span>
                                        </div>
                                        <pre class="p-4 text-sm text-gray-800 dark:text-gray-200 font-mono whitespace-pre-wrap leading-relaxed tab-size-4">{content}</pre>
                                    </div>
                                }
                            })}
                        </Card>
                    </Show>

                    <Show when=move || !loading.get() && !showing_file() && !tree_entries.with(|t| t.is_empty()) fallback=|| view! { <div class="hidden"></div> }>
                        <Card>
                            <div class="mb-3 flex items-center gap-1 text-sm text-gray-500 dark:text-gray-400 font-mono overflow-x-auto">
                                <span>{owner()}</span>
                                <span>"/"</span>
                                <span>{name()}</span>
                                <Show when=has_path>
                                    <For each=move || breadcrumb_parts(&path_param()) key=|s| s.clone() let:seg>
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
                                        <For each=move || tree_entries.get() key=|e| e.path.clone() let:entry>
                                            {
                                                let entry_type = entry.entry_type.clone();
                                                let owner_c = owner();
                                                let name_c = name();
                                                let path_c = path_param();
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
                                                view! {
                                                    <A href=format!(
                                                        "/repos/{owner_c}/{name_c}/code/{}{}",
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
                                                                    {match icon {
                                                                        "dir" => view! {
                                                                            <svg class="w-4 h-4 text-blue-500 dark:text-blue-400 shrink-0" fill="currentColor" viewBox="0 0 20 20">
                                                                                <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"/>
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
                                                                {move || commit_info.as_ref().map(|c| {
                                                                    view! {
                                                                        <div class="text-xs">
                                                                            <span class="text-gray-700 dark:text-gray-300 truncate block max-w-xs">{c.message.clone()}</span>
                                                                            <span class="text-gray-400 dark:text-gray-500">{c.author.clone()}</span>
                                                                            <span class="text-gray-400 dark:text-gray-500 ml-2">{c.time.clone()}</span>
                                                                        </div>
                                                                    }.into_any()
                                                                }).unwrap_or_else(|| view! { <span class="text-gray-400 dark:text-gray-500 text-xs">"-"</span> }.into_any())}
                                                            </td>
                                                            <td class="py-2 pr-4 text-right text-gray-400 dark:text-gray-500 font-mono text-xs">
                                                                {if is_dir {
                                                                    view! { <span>"-"</span> }.into_any()
                                                                } else {
                                                                    view! { <span>{format_size(entry.size)}</span> }.into_any()
                                                                }}
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
                                        {format!(
                                            "Showing {} of {} entries (page {}/{})",
                                            tree_entries.with(|t| t.len()),
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
                                                let href = code_url_with_ref(String::new(), prev);
                                                view! {
                                                    <a
                                                        class="px-3 py-1.5 text-sm rounded border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors cursor-pointer"
                                                        href=href
                                                    >"← Prev"</a>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <span class="px-3 py-1.5 text-sm rounded border border-gray-300 dark:border-gray-600 opacity-40 cursor-not-allowed">"← Prev"</span>
                                                }.into_any()
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
                                                let href = code_url_with_ref(String::new(), next);
                                                view! {
                                                    <a
                                                        class="px-3 py-1.5 text-sm rounded border border-gray-300 dark:border-gray-600 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors cursor-pointer"
                                                        href=href
                                                    >"Next →"</a>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <span class="px-3 py-1.5 text-sm rounded border border-gray-300 dark:border-gray-600 opacity-40 cursor-not-allowed">"Next →"</span>
                                                }.into_any()
                                            }
                                        }}
                                    </div>
                                </div>
                            </Show>
                        </Card>
                    </Show>
            </div>
        </div>
    }
}
