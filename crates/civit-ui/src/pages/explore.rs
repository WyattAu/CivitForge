#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;

use crate::api::client::ApiClient;
use crate::api::repos::list_repos;
use crate::components::{
    Badge, BadgeColor, Button, ButtonVariant, Card, Input, InputType, Pagination, Spinner,
};
use crate::state::auth::use_auth;
use civit_shared::pagination::PaginationParams;
use civit_shared::visibility::Visibility;

fn truncate_uuid(s: &str) -> String {
    if s.len() > 8 {
        format!("{}...", &s[..8])
    } else {
        s.to_string()
    }
}

fn format_datetime(dt: &chrono::DateTime<chrono::Utc>) -> String {
    dt.format("%b %d, %Y").to_string()
}

#[component]
pub fn ExplorePage() -> impl IntoView {
    let auth = use_auth();
    let (query, _set_query) = signal(String::new());
    let (repos_sig, set_repos) = signal(vec![]);
    let (loading, set_loading) = signal(true);
    let (page, set_page) = signal(1u32);
    let (total_pages, set_total_pages) = signal(1u32);
    let (error, set_error) = signal(None::<String>);

    let fetch_repos = move || {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        let current_page = page.get();
        let params = PaginationParams {
            per_page: Some(50),
            page: Some(current_page),
            offset: None,
        };

        leptos::task::spawn_local(async move {
            match list_repos(&client, params).await {
                Ok(resp) => {
                    set_repos.set(resp.data);
                    set_total_pages.set(resp.pagination.total_pages);
                }
                Err(_) => {
                    set_error.set(Some("Failed to load repositories.".to_string()));
                }
            }
            set_loading.set(false);
        });
    };

    fetch_repos();

    let handle_page_change = Callback::new(move |new_page: u32| {
        set_page.set(new_page);
        fetch_repos();
    });

    let handle_search = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        fetch_repos();
    };

    let has_repos = move || !loading.get() && !repos_sig.with(|r| r.is_empty());

    view! {
        <div class="space-y-6">
            <div class="text-center">
                <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Explore Repositories"</h1>
                <p class="mt-2 text-gray-600 dark:text-gray-400">
                    "Discover open-source projects and repositories."
                </p>
            </div>

            <div class="max-w-xl mx-auto">
                <form class="flex gap-2" on:submit=handle_search>
                    <div class="flex-1">
                        <Input
                            input_type=InputType::Text
                            name="q"
                            placeholder="Search repositories..."
                            value=query.get()
                        ></Input>
                    </div>
                    <Button variant=ButtonVariant::Primary extra_class="btn-search">
                        "Search"
                    </Button>
                </form>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div></div> }>
                <div class="max-w-4xl mx-auto p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md">
                    <p class="text-sm text-red-700 dark:text-red-400">{move || error.get().unwrap_or_default()}</p>
                </div>
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div></div> }>
                <div class="flex items-center justify-center py-12">
                    <Spinner />
                </div>
            </Show>

            <Show when=move || !loading.get() && repos_sig.with(|r| r.is_empty()) && error.get().is_none() fallback=|| view! { <div></div> }>
                <Card>
                    <div class="py-12 text-center">
                        <p class="text-gray-500 dark:text-gray-400 text-lg">
                            "No repositories found. Be the first to create one!"
                        </p>
                    </div>
                </Card>
            </Show>

            <Show when=move || has_repos() fallback=|| view! { <div></div> }>
                <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                    <For each=move || repos_sig.get() key=|r| r.id let:repo>
                        {
                            let repo = repo.clone();
                            let card_class = "hover:border-blue-300 dark:hover:border-blue-700 transition-colors cursor-pointer".to_string();
                            view! {
                                <A href=format!("/repos/{}/{}", repo.owner_id, repo.name)>
                                    <Card class=card_class>
                                        <div class="flex items-start justify-between gap-2">
                                            <h3 class="font-semibold text-blue-600 dark:text-blue-400 hover:underline truncate">
                                                {repo.name.clone()}
                                            </h3>
                                            <Badge
                                                color=match repo.visibility {
                                                    Visibility::Public => BadgeColor::Success,
                                                    Visibility::Internal => BadgeColor::Info,
                                                    Visibility::Private => BadgeColor::Neutral,
                                                }
                                                text=repo.visibility.to_string()
                                            />
                                        </div>
                                        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400 line-clamp-2">
                                            {repo.description.clone().unwrap_or_else(|| "No description provided.".to_string())}
                                        </p>
                                        <div class="mt-3 flex flex-wrap gap-x-4 gap-y-1 text-xs text-gray-400 dark:text-gray-500">
                                            <span title="Owner">"Owner: "{truncate_uuid(&repo.owner_id.to_string())}</span>
                                            <span>"Created: "{format_datetime(&repo.created_at)}</span>
                                            <span>"Updated: "{format_datetime(&repo.updated_at)}</span>
                                        </div>
                                    </Card>
                                </A>
                            }
                        }
                    </For>
                </div>

                <div class="mt-6">
                    <Pagination
                        current_page=page.get()
                        total_pages=total_pages.get()
                        on_page_change=handle_page_change
                    />
                </div>
            </Show>
        </div>
    }
}
