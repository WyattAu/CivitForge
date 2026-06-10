#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::{A, Outlet};
use leptos_router::hooks::{use_location, use_params_map};

use crate::api::client::ApiClient;
use crate::components::{Badge, BadgeColor, ErrorBanner, Modal, Spinner};
use crate::state::auth::use_auth;
use civit_shared::repo::RepoResponse;

#[component]
pub fn RepoDetailPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let auth = use_auth();

    let (repo_sig, set_repo) = signal(None::<RepoResponse>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (starred, set_starred) = signal(false);
    let (watching, set_watching) = signal(false);
    let (show_clone, set_show_clone) = signal(false);

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
            Ok(_) => set_error.set(Some("Failed to load repository.".to_string())),
            Err(_) => set_error.set(Some("Network error. Check your connection.".to_string())),
        }
        set_loading.set(false);
    });

    let toggle_star = Callback::new(move |_: ()| {
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        let is_starred = starred.get();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/star");
            let _ = client.post(&path, &()).await;
            set_starred.set(!is_starred);
        });
    });

    let toggle_watch = Callback::new(move |_: ()| {
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        let is_watching = watching.get();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/watch");
            let _ = client.post(&path, &()).await;
            set_watching.set(!is_watching);
        });
    });

    let fork_repo = Callback::new(move |_: ()| {
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/fork");
            let _ = client.post(&path, &()).await;
        });
    });

    let close_clone = Callback::new(move |_: ()| set_show_clone.set(false));
    let open_clone = Callback::new(move |_: ()| set_show_clone.set(true));
    let dismiss_error = Callback::new(move |_: ()| set_error.set(None));

    let repo_loaded = move || !loading.get() && repo_sig.get().is_some();
    let has_error = move || error.get().is_some();

    view! {
        <div class="space-y-6">
            <Show when=has_error fallback=|| view! { <div></div> }>
                <ErrorBanner
                    message=move || error.get().unwrap_or_default()
                    on_dismiss=dismiss_error
                />
            </Show>
            <Show when=move || loading.get() fallback=|| view! { <div></div> }>
                <div class="flex items-center gap-2">
                    <Spinner />
                    <span class="text-gray-500 dark:text-gray-400">
                        "Loading repository..."
                    </span>
                </div>
            </Show>
            <Show when=repo_loaded fallback=|| view! { <div></div> }>
                <RepoHeader
                    owner=owner
                    repo_sig=repo_sig
                    starred=starred
                    watching=watching
                    toggle_star=toggle_star
                    toggle_watch=toggle_watch
                    fork_repo=fork_repo
                    open_clone=open_clone
                />
                <RepoTabs owner=owner name=name />
                <div class="mt-6">
                    <Outlet />
                </div>
            </Show>
            <Show
                when=move || show_clone.get()
                fallback=|| view! { <div></div> }
            >
                <Modal
                    show=true
                    title="Clone repository".to_string()
                    on_close=close_clone
                >
                    <CloneContent repo_sig=repo_sig />
                </Modal>
            </Show>
        </div>
    }
}

#[component]
fn RepoHeader(
    owner: impl Fn() -> String + Send + Sync + Copy + 'static,
    repo_sig: ReadSignal<Option<RepoResponse>>,
    starred: ReadSignal<bool>,
    watching: ReadSignal<bool>,
    toggle_star: Callback<()>,
    toggle_watch: Callback<()>,
    fork_repo: Callback<()>,
    open_clone: Callback<()>,
) -> impl IntoView {
    let btn_base = "inline-flex items-center justify-center gap-2 px-4 py-2 rounded-none text-sm font-medium transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900";
    let btn_secondary = "bg-gray-200 hover:bg-gray-300 text-gray-900 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-100";
    let btn_active_star = "bg-amber-100 hover:bg-amber-200 text-amber-800 dark:bg-amber-900 dark:hover:bg-amber-800 dark:text-amber-200";
    let btn_active_watch = "bg-emerald-100 hover:bg-emerald-200 text-emerald-800 dark:bg-emerald-900 dark:hover:bg-emerald-800 dark:text-emerald-200";

    view! {
        <div class="flex items-start justify-between flex-wrap gap-4">
            <div class="min-w-0">
                <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1">
                    <A href="/repos">
                        <span class="hover:text-blue-600 dark:hover:text-blue-400">
                            "Repositories"
                        </span>
                    </A>
                    <span>"/"</span>
                    <span class="text-gray-700 dark:text-gray-300 font-mono">{owner()}</span>
                </div>
                <h1 class="text-3xl font-bold font-mono text-gray-900 dark:text-gray-100 flex items-center gap-3 flex-wrap">
                    {move || repo_sig.get().map(|r| r.full_name.clone()).unwrap_or_default()}
                    {move || {
                        repo_sig.get().map(|repo| {
                            let color = match repo.visibility {
                                civit_shared::visibility::Visibility::Public => {
                                    BadgeColor::Success
                                }
                                civit_shared::visibility::Visibility::Internal => {
                                    BadgeColor::Info
                                }
                                civit_shared::visibility::Visibility::Private => {
                                    BadgeColor::Neutral
                                }
                            };
                            view! { <Badge color=color text=repo.visibility.to_string() /> }
                        })
                    }}
                </h1>
                <p class="mt-2 text-gray-600 dark:text-gray-400">
                    {move || {
                        repo_sig
                            .get()
                            .and_then(|r| r.description.clone())
                            .unwrap_or_else(|| "No description provided.".to_string())
                    }}
                </p>
                <div class="mt-3 flex items-center gap-3 text-sm text-gray-500 dark:text-gray-400">
                    <span class="font-mono">"0 stars"</span>
                    <span>"|"</span>
                    <span class="font-mono">"0 forks"</span>
                    <span>"|"</span>
                    <span class="font-mono">"0 watchers"</span>
                </div>
            </div>
            <div class="flex items-center gap-2 flex-shrink-0">
                <button
                    on:click=move |_| toggle_star.run(())
                    class=move || {
                        format!(
                            "{} {}",
                            btn_base,
                            if starred.get() { btn_active_star } else { btn_secondary }
                        )
                    }
                >
                    {move || if starred.get() { "Starred" } else { "Star" }}
                </button>
                <button
                    on:click=move |_| toggle_watch.run(())
                    class=move || {
                        format!(
                            "{} {}",
                            btn_base,
                            if watching.get() { btn_active_watch } else { btn_secondary }
                        )
                    }
                >
                    {move || if watching.get() { "Watching" } else { "Watch" }}
                </button>
                <button
                    on:click=move |_| fork_repo.run(())
                    class=format!("{btn_base} {btn_secondary}")
                >
                    "Fork"
                </button>
                <button
                    on:click=move |_| open_clone.run(())
                    class=format!("{btn_base} {btn_secondary}")
                >
                    "Clone"
                </button>
            </div>
        </div>
    }
}

#[component]
fn RepoTabs(
    owner: impl Fn() -> String + Send + Sync + Copy + 'static,
    name: impl Fn() -> String + Send + Sync + Copy + 'static,
) -> impl IntoView {
    let location = use_location();
    let icon_code = "{ }";
    let icon_issues = "!";
    let icon_prs = "><";
    let icon_pipelines = ">";
    let icon_wiki = "W";
    let icon_settings = "S";
    let tab_active = "px-4 py-3 border-b-2 border-blue-600 dark:border-blue-400 text-blue-600 dark:text-blue-400 font-mono flex items-center gap-2";
    let tab_inactive = "px-4 py-3 border-b-2 border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-200 dark:hover:border-gray-600 font-mono flex items-center gap-2";

    let active_tab = move || {
        let pathname = location.pathname.with(|p| p.clone());
        let base = format!("/repos/{}/{}", owner(), name());
        let rest = pathname.strip_prefix(&base).unwrap_or("");
        if rest.is_empty() || rest.starts_with("/code") {
            "code"
        } else if rest.starts_with("/issues") {
            "issues"
        } else if rest.starts_with("/pulls") {
            "pulls"
        } else if rest.starts_with("/boards") {
            "boards"
        } else if rest.starts_with("/wiki") {
            "wiki"
        } else if rest.starts_with("/pipelines") {
            "pipelines"
        } else if rest.starts_with("/settings") {
            "settings"
        } else {
            "code"
        }
    };

    view! {
        <div class="flex gap-1 text-sm font-medium border-b border-gray-200 dark:border-gray-700 -mb-px">
            <A href=format!("/repos/{}/{}", owner(), name())>
                <span class=move || if active_tab() == "code" { tab_active } else { tab_inactive }>
                    <span class="text-xs opacity-60">{icon_code}</span>
                    <span>"Code"</span>
                </span>
            </A>
            <A href=format!("/repos/{}/{}/issues", owner(), name())>
                <span class=move || if active_tab() == "issues" { tab_active } else { tab_inactive }>
                    <span class="text-xs opacity-60">{icon_issues}</span>
                    <span>"Issues"</span>
                </span>
            </A>
            <A href=format!("/repos/{}/{}/pulls", owner(), name())>
                <span class=move || if active_tab() == "pulls" { tab_active } else { tab_inactive }>
                    <span class="text-xs opacity-60">{icon_prs}</span>
                    <span>"Pull Requests"</span>
                </span>
            </A>
            <A href=format!("/repos/{}/{}/boards", owner(), name())>
                <span class=move || if active_tab() == "boards" { tab_active } else { tab_inactive }>
                    <span class="text-xs opacity-60">"K"</span>
                    <span>"Boards"</span>
                </span>
            </A>
            <A href=format!("/repos/{}/{}/pipelines", owner(), name())>
                <span class=move || if active_tab() == "pipelines" { tab_active } else { tab_inactive }>
                    <span class="text-xs opacity-60">{icon_pipelines}</span>
                    <span>"Pipelines"</span>
                </span>
            </A>
            <A href=format!("/repos/{}/{}/wiki", owner(), name())>
                <span class=move || if active_tab() == "wiki" { tab_active } else { tab_inactive }>
                    <span class="text-xs opacity-60">{icon_wiki}</span>
                    <span>"Wiki"</span>
                </span>
            </A>
            <A href=format!("/repos/{}/{}/settings", owner(), name())>
                <span class=move || if active_tab() == "settings" { tab_active } else { tab_inactive }>
                    <span class="text-xs opacity-60">{icon_settings}</span>
                    <span>"Settings"</span>
                </span>
            </A>
        </div>
    }
}

#[component]
fn CloneContent(repo_sig: ReadSignal<Option<RepoResponse>>) -> impl IntoView {
    let url_box = "w-full px-3 py-2 rounded-none border-2 border-gray-300 dark:border-gray-600 bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100 font-mono text-sm break-all select-all";

    view! {
        <div class="space-y-4">
            <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2 font-mono">
                    "HTTPS"
                </label>
                <div class=url_box>
                    {move || {
                        repo_sig
                            .get()
                            .and_then(|r| r.http_clone_url.clone())
                            .unwrap_or_default()
                    }}
                </div>
            </div>
            <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2 font-mono">
                    "SSH"
                </label>
                <div class=url_box>
                    {move || {
                        repo_sig
                            .get()
                            .and_then(|r| r.ssh_clone_url.clone())
                            .unwrap_or_default()
                    }}
                </div>
            </div>
        </div>
    }
}
