#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::api::types::{ListResponse, WikiPageResponse};
use crate::components::{Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Spinner};
use crate::state::auth::use_auth;
use civit_shared::repo::RepoResponse;

#[derive(Clone, serde::Deserialize)]
struct CommitEntry {
    sha: String,
    message: Option<String>,
    author: Option<String>,
    #[allow(dead_code)]
    created_at: Option<String>,
}

fn truncate_sha(sha: &str) -> String {
    if sha.len() > 7 {
        sha[..7].to_string()
    } else {
        sha.to_string()
    }
}

#[component]
pub fn RepoDetailPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let auth = use_auth();

    let (repo_sig, set_repo) = signal(None::<RepoResponse>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let (wiki_content, set_wiki_content) = signal(None::<WikiPageResponse>);
    let (wiki_loading, set_wiki_loading) = signal(true);

    let (commits_sig, set_commits) = signal(Vec::<CommitEntry>::new());
    let (commits_loading, set_commits_loading) = signal(true);

    leptos::task::spawn_local(async move {
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        let owner_val = owner();
        let name_val = name();
        let path = format!("/repos/{owner_val}/{name_val}");
        match client.get(&path).await {
            Ok(resp) if resp.status().is_success() => match resp.json::<RepoResponse>().await {
                Ok(data) => set_repo.set(Some(data)),
                Err(_) => set_error.set(Some("Failed to process response.".to_string())),
            },
            Ok(_) => {
                set_error.set(Some("Failed to load repository.".to_string()));
            }
            Err(_) => {
                set_error.set(Some("Network error. Check your connection.".to_string()));
            }
        }
        set_loading.set(false);
    });

    leptos::task::spawn_local(async move {
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        let owner_val = owner();
        let name_val = name();
        let path = format!("/repos/{owner_val}/{name_val}/wiki/home");
        match client.get(&path).await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(data) = resp.json::<WikiPageResponse>().await {
                    set_wiki_content.set(Some(data));
                }
            }
            _ => {}
        }
        set_wiki_loading.set(false);
    });

    leptos::task::spawn_local(async move {
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        let owner_val = owner();
        let name_val = name();
        let path = format!("/repos/{owner_val}/{name_val}/commits");
        match client.get(&path).await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(data) = resp.json::<ListResponse<CommitEntry>>().await {
                    set_commits.set(data.data);
                }
            }
            _ => {}
        }
        set_commits_loading.set(false);
    });

    let repo_loaded = move || !loading.get() && repo_sig.get().is_some();
    let has_error = move || error.get().is_some();

    let dismiss_error = Callback::new(move |_: ()| set_error.set(None));

    view! {
        <div class="space-y-6">
            <Show when=has_error fallback=|| view! { <div></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=dismiss_error />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div></div> }>
                <div class="flex items-center gap-2">
                    <Spinner />
                    <span class="text-gray-500 dark:text-gray-400">"Loading repository..."</span>
                </div>
            </Show>

            <Show when=repo_loaded fallback=|| view! { <div></div> }>
                <RepoHeader owner=owner repo_sig=repo_sig />
                <RepoTabs owner=owner name=name />
                <RepoContent
                    repo_sig=repo_sig
                    wiki_content=wiki_content
                    wiki_loading=wiki_loading
                    commits_sig=commits_sig
                    commits_loading=commits_loading
                />
            </Show>
        </div>
    }
}

#[component]
fn RepoHeader(
    owner: impl Fn() -> String + 'static + Copy,
    repo_sig: ReadSignal<Option<RepoResponse>>,
) -> impl IntoView {
    let visibility_badge = move || {
        repo_sig.get().map(|repo| {
            let color = match repo.visibility {
                civit_shared::visibility::Visibility::Public => BadgeColor::Success,
                civit_shared::visibility::Visibility::Internal => BadgeColor::Info,
                civit_shared::visibility::Visibility::Private => BadgeColor::Neutral,
            };
            (repo.visibility.to_string(), color)
        })
    };

    view! {
        <div class="flex items-center justify-between flex-wrap gap-4">
            <div>
                <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1">
                    <A href="/repos"><span class="hover:text-blue-600 dark:hover:text-blue-400">"Repositories"</span></A>
                    <span>"/"</span>
                    <span class="text-gray-700 dark:text-gray-300">{owner()}</span>
                </div>
                <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100 flex items-center gap-3">
                    {move || repo_sig.get().map(|r| r.full_name.clone()).unwrap_or_default()}
                    {move || visibility_badge().map(|(text, color)| view! { <Badge color=color text=text /> })}
                </h1>
                <p class="mt-2 text-gray-600 dark:text-gray-400">
                    {move || repo_sig.get().and_then(|r| r.description.clone()).unwrap_or_else(|| "No description provided.".to_string())}
                </p>
            </div>
            <div class="flex gap-2">
                <Button variant=ButtonVariant::Secondary disabled=true>"Star"</Button>
                <Button variant=ButtonVariant::Secondary disabled=true>"Fork"</Button>
                <Button variant=ButtonVariant::Primary>
                    "Code"
                </Button>
            </div>
        </div>
    }
}

#[component]
fn RepoTabs(
    owner: impl Fn() -> String + 'static + Copy,
    name: impl Fn() -> String + 'static + Copy,
) -> impl IntoView {
    view! {
        <div class="flex items-center gap-4 text-sm text-gray-500 dark:text-gray-400 border-b border-gray-200 dark:border-gray-700 pb-4">
            <span>"0 issues"</span>
            <span>"0 wiki pages"</span>
        </div>

        <div class="flex gap-1 text-sm font-medium border-b border-gray-200 dark:border-gray-700 -mb-px">
            <A href=format!("/repos/{}/{}/code", owner(), name())>
                <span class="px-4 py-3 border-b-2 border-blue-600 dark:border-blue-400 text-blue-600 dark:text-blue-400">"Code"</span>
            </A>
            <A href=format!("/repos/{}/{}/issues", owner(), name())>
                <span class="px-4 py-3 border-b-2 border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-200 dark:hover:border-gray-600">"Issues"</span>
            </A>
            <A href=format!("/repos/{}/{}/wiki", owner(), name())>
                <span class="px-4 py-3 border-b-2 border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-200 dark:hover:border-gray-600">"Wiki"</span>
            </A>
            <A href=format!("/repos/{}/{}/pipelines", owner(), name())>
                <span class="px-4 py-3 border-b-2 border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-200 dark:hover:border-gray-600">"Pipelines"</span>
            </A>
            <A href=format!("/repos/{}/{}/settings", owner(), name())>
                <span class="px-4 py-3 border-b-2 border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-200 dark:hover:border-gray-600">"Settings"</span>
            </A>
        </div>
    }
}

#[component]
fn RepoContent(
    repo_sig: ReadSignal<Option<RepoResponse>>,
    wiki_content: ReadSignal<Option<WikiPageResponse>>,
    wiki_loading: ReadSignal<bool>,
    commits_sig: ReadSignal<Vec<CommitEntry>>,
    commits_loading: ReadSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
            <div class="lg:col-span-3 space-y-6">
                <Card title="README" description="Project documentation">
                    <Show when=move || wiki_loading.get() fallback=|| view! { <div></div> }>
                        <div class="flex items-center justify-center py-6">
                            <Spinner />
                        </div>
                    </Show>
                    <Show when=move || !wiki_loading.get() && wiki_content.get().is_some() fallback=|| view! { <div></div> }>
                        <div class="prose dark:prose-invert max-w-none text-gray-700 dark:text-gray-300 whitespace-pre-wrap leading-relaxed text-sm bg-gray-50 dark:bg-gray-900/50 rounded-md p-4 overflow-x-auto">
                            {move || wiki_content.get().map(|w| w.content.clone()).unwrap_or_default()}
                        </div>
                    </Show>
                    <Show when=move || !wiki_loading.get() && wiki_content.get().is_none() fallback=|| view! { <div></div> }>
                        <div class="prose dark:prose-invert max-w-none text-gray-600 dark:text-gray-400">
                            <p>{move || format!("# {}", repo_sig.get().map(|r| r.name.clone()).unwrap_or_default())}</p>
                            <p>{move || repo_sig.get().and_then(|r| r.description.clone()).unwrap_or_else(|| "No README yet.".to_string())}</p>
                        </div>
                    </Show>
                </Card>

                <Card title="Latest Commits" description="Recent commit history">
                    <Show when=move || commits_loading.get() fallback=|| view! { <div></div> }>
                        <div class="flex items-center justify-center py-6">
                            <Spinner />
                        </div>
                    </Show>
                    <Show when=move || !commits_loading.get() && commits_sig.with(|c| c.is_empty()) fallback=|| view! { <div></div> }>
                        <div class="py-6 text-center text-gray-500 dark:text-gray-400">
                            "No commits yet."
                        </div>
                    </Show>
                    <Show when=move || !commits_loading.get() && !commits_sig.with(|c| c.is_empty()) fallback=|| view! { <div></div> }>
                        <div class="divide-y divide-gray-100 dark:divide-gray-700">
                            <For each=move || commits_sig.get() key=|c| c.sha.clone() let:commit>
                                {
                                    view! {
                                        <div class="flex items-center gap-4 py-3">
                                            <span class="font-mono text-sm text-blue-600 dark:text-blue-400 shrink-0">
                                                {truncate_sha(&commit.sha)}
                                            </span>
                                            <span class="text-sm text-gray-700 dark:text-gray-300 truncate flex-1">
                                                {commit.message.clone().unwrap_or_else(|| "No commit message".to_string())}
                                            </span>
                                            {commit.author.as_ref().map(|a| {
                                                view! {
                                                    <span class="text-xs text-gray-400 dark:text-gray-500 shrink-0">{a.clone()}</span>
                                                }
                                            })}
                                        </div>
                                    }
                                }
                            </For>
                        </div>
                    </Show>
                </Card>
            </div>

            <div class="lg:col-span-1 space-y-4">
                <Card>
                    <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-3">"About"</h3>
                    <div class="space-y-3 text-sm">
                        <div>
                            <span class="text-gray-500 dark:text-gray-400">"Owner"</span>
                            <div class="text-gray-900 dark:text-gray-100">{move || repo_sig.get().map(|r| r.owner_id.to_string()).unwrap_or_default()}</div>
                        </div>
                        <div>
                            <span class="text-gray-500 dark:text-gray-400">"Default Branch"</span>
                            <div class="text-gray-900 dark:text-gray-100 font-mono">{move || repo_sig.get().map(|r| r.default_branch.clone()).unwrap_or_default()}</div>
                        </div>
                        <div>
                            <span class="text-gray-500 dark:text-gray-400">"Created"</span>
                            <div class="text-gray-900 dark:text-gray-100">{move || repo_sig.get().map(|r| r.created_at.format("%b %d, %Y").to_string()).unwrap_or_default()}</div>
                        </div>
                        <div>
                            <span class="text-gray-500 dark:text-gray-400">"Updated"</span>
                            <div class="text-gray-900 dark:text-gray-100">{move || repo_sig.get().map(|r| r.updated_at.format("%b %d, %Y").to_string()).unwrap_or_default()}</div>
                        </div>
                        <div>
                            <span class="text-gray-500 dark:text-gray-400">"License"</span>
                            <div class="text-gray-400 dark:text-gray-500 italic">"Not specified"</div>
                        </div>
                    </div>
                </Card>

                <Card>
                    <div class="grid grid-cols-2 gap-4">
                        <div class="text-center">
                            <div class="text-2xl font-bold text-gray-900 dark:text-gray-100">{move || repo_sig.get().map(|r| r.visibility.to_string()).unwrap_or_default()}</div>
                            <div class="text-xs text-gray-500 dark:text-gray-400">"Visibility"</div>
                        </div>
                        <div class="text-center">
                            <div class="text-2xl font-bold text-gray-900 dark:text-gray-100 font-mono">{move || repo_sig.get().map(|r| r.default_branch.clone()).unwrap_or_default()}</div>
                            <div class="text-xs text-gray-500 dark:text-gray-400">"Branch"</div>
                        </div>
                    </div>
                </Card>
            </div>
        </div>
    }
}
