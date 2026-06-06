#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;

use crate::api::client::ApiClient;
use crate::api::repos::list_repos;
use crate::api::types::ListResponse;
use crate::components::{Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Spinner};
use crate::state::auth::use_auth;
use civit_shared::pagination::PaginationParams;
use civit_shared::visibility::Visibility;

fn format_datetime(dt: &chrono::DateTime<chrono::Utc>) -> String {
    dt.format("%b %d, %Y").to_string()
}

#[component]
pub fn HomePage() -> impl IntoView {
    let auth = use_auth();
    let is_authenticated = move || auth.0.with(|a| a.is_authenticated);

    view! {
        <Show when=is_authenticated fallback=|| view! { <HomeNotLoggedIn /> }>
            <HomeLoggedIn />
        </Show>
    }
}

#[component]
fn HomeNotLoggedIn() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center py-20">
            <div class="max-w-2xl text-center">
                <h1 class="text-5xl font-extrabold text-gray-900 dark:text-gray-100 tracking-tight">
                    "Welcome to CivitForge"
                </h1>
                <p class="mt-6 text-lg text-gray-600 dark:text-gray-400 leading-relaxed">
                    "A modern, lightweight forge for hosting Git repositories, tracking issues, and collaborating with your team."
                </p>
                <div class="mt-10 flex items-center justify-center gap-4">
                    <A href="/login">
                        <Button variant=ButtonVariant::Primary extra_class="btn-get-started">
                            "Get Started"
                        </Button>
                    </A>
                    <A href="/explore">
                        <Button variant=ButtonVariant::Secondary>"Explore Repos"</Button>
                    </A>
                </div>

                <div class="mt-16 grid grid-cols-1 sm:grid-cols-3 gap-8 text-center">
                    <div>
                        <div class="text-3xl font-bold text-blue-600 dark:text-blue-400">"Git Hosting"</div>
                        <p class="mt-2 text-sm text-gray-500 dark:text-gray-400">
                            "Host your repositories with SSH and HTTPS support."
                        </p>
                    </div>
                    <div>
                        <div class="text-3xl font-bold text-blue-600 dark:text-blue-400">"Issue Tracking"</div>
                        <p class="mt-2 text-sm text-gray-500 dark:text-gray-400">
                            "Track bugs, features, and tasks with full issue management."
                        </p>
                    </div>
                    <div>
                        <div class="text-3xl font-bold text-blue-600 dark:text-blue-400">"Wiki"</div>
                        <p class="mt-2 text-sm text-gray-500 dark:text-gray-400">
                            "Document your projects with built-in wiki pages."
                        </p>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn HomeLoggedIn() -> impl IntoView {
    let auth = use_auth();
    let (repos_sig, set_repos) = signal(None::<ListResponse<civit_shared::repo::RepoResponse>>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    leptos::task::spawn_local(async move {
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        let params = PaginationParams {
            per_page: Some(5),
            page: Some(1),
            offset: None,
        };
        match list_repos(&client, params).await {
            Ok(resp) => set_repos.set(Some(resp)),
            Err(_) => set_error.set(Some("Failed to load repositories.".to_string())),
        }
        set_loading.set(false);
    });

    let username = move || auth.0.with(|a| a.username.clone().unwrap_or_default());

    view! {
        <div class="space-y-8">
            <div>
                <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">
                    "Welcome back, "{username}
                </h1>
                <p class="mt-2 text-gray-600 dark:text-gray-400">
                    "Here's an overview of your projects."
                </p>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
                <A href="/new-repo">
                    <Card class="hover:border-blue-300 dark:hover:border-blue-700 transition-colors cursor-pointer".to_string()>
                        <div class="text-center">
                            <div class="text-3xl mb-2">"➕"</div>
                            <div class="font-semibold text-gray-900 dark:text-gray-100">"New Repository"</div>
                            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">"Create a new project"</p>
                        </div>
                    </Card>
                </A>
                <A href="/explore">
                    <Card class="hover:border-blue-300 dark:hover:border-blue-700 transition-colors cursor-pointer".to_string()>
                        <div class="text-center">
                            <div class="text-3xl mb-2">"🔍"</div>
                            <div class="font-semibold text-gray-900 dark:text-gray-100">"Explore"</div>
                            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">"Discover repositories"</p>
                        </div>
                    </Card>
                </A>
                <A href="/repos">
                    <Card class="hover:border-blue-300 dark:hover:border-blue-700 transition-colors cursor-pointer".to_string()>
                        <div class="text-center">
                            <div class="text-3xl mb-2">"📁"</div>
                            <div class="font-semibold text-gray-900 dark:text-gray-100">"All Repos"</div>
                            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">"Browse your repositories"</p>
                        </div>
                    </Card>
                </A>
            </div>

            <Card title="Your Repositories" description="Your latest repositories">
                <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                    <div class="flex items-center justify-center py-8">
                        <Spinner />
                    </div>
                </Show>

                <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                    <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_error.set(None)) />
                </Show>

                <Show when=move || !loading.get() && repos_sig.get().is_some_and(|r| r.data.is_empty()) fallback=|| view! { <div class="hidden"></div> }>
                    <div class="py-8 text-center">
                        <p class="text-gray-500 dark:text-gray-400">"No repositories yet."</p>
                        <A href="/new-repo">
                            <Button variant=ButtonVariant::Primary extra_class="mt-4">"Create your first repository"</Button>
                        </A>
                    </div>
                </Show>

                <Show when=move || !loading.get() && repos_sig.get().is_some_and(|r| !r.data.is_empty()) fallback=|| view! { <div class="hidden"></div> }>
                    <div class="space-y-0">
                        <For each=move || repos_sig.get().map(|r| r.data.clone()).unwrap_or_default() key=|r| r.id let:repo>
                            {
                                let repo = repo.clone();
                                view! {
                                    <A href=format!("/repos/{}", repo.full_name)>
                                        <div class="flex items-center justify-between py-3 border-b border-gray-100 dark:border-gray-700 last:border-0 hover:bg-gray-50 dark:hover:bg-gray-750 px-2 -mx-2 rounded transition-colors">
                                            <div class="min-w-0">
                                                <div class="flex items-center gap-2">
                                                    <span class="font-medium text-blue-600 dark:text-blue-400 truncate">
                                                        {repo.name.clone()}
                                                    </span>
                                                    <Badge
                                                        color=match repo.visibility {
                                                            Visibility::Public => BadgeColor::Success,
                                                            Visibility::Internal => BadgeColor::Info,
                                                            Visibility::Private => BadgeColor::Neutral,
                                                        }
                                                        text=repo.visibility.to_string()
                                                    />
                                                </div>
                                                <p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5 truncate">
                                                    {repo.description.clone().unwrap_or_else(|| "No description".to_string())}
                                                </p>
                                            </div>
                                            <span class="text-xs text-gray-400 dark:text-gray-500 shrink-0 ml-4">
                                                {format_datetime(&repo.updated_at)}
                                            </span>
                                        </div>
                                    </A>
                                }
                            }
                        </For>
                    </div>
                </Show>
            </Card>

        </div>
    }
}
