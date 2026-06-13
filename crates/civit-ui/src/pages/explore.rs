#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;

use crate::api::client::ApiClient;
use crate::api::types::ListResponse;
use crate::components::{
    Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Input, InputType, Pagination,
    Spinner,
};
use crate::state::auth::use_auth;
use crate::utils::*;
use civit_shared::repo::RepoResponse;
use civit_shared::visibility::Visibility;

#[component]
pub fn ExplorePage() -> impl IntoView {
    let auth = use_auth();
    let (query, set_query) = signal(String::new());
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
        let query_val = query.get();

        leptos::task::spawn_local(async move {
            let mut path = format!("/repos?per_page=50&page={current_page}");
            if !query_val.trim().is_empty() {
                let encoded: String = query_val
                    .trim()
                    .chars()
                    .map(|c| match c {
                        ' ' => "+".to_string(),
                        c if c.is_alphanumeric() => c.to_string(),
                        _ => format!("%{:02X}", c as u8),
                    })
                    .collect();
                path.push_str(&format!("&q={encoded}"));
            }
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<ListResponse<RepoResponse>>().await {
                        Ok(data) => {
                            set_repos.set(data.data);
                            set_total_pages.set(data.pagination.total_pages);
                        }
                        Err(_) => {
                            set_error.set(Some("Failed to load repositories.".to_string()));
                        }
                    }
                }
                _ => {
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
        let q = get_input_value("q");
        set_query.set(q);
        set_page.set(1);
        fetch_repos();
    };

    let has_repos = move || !loading.get() && !repos_sig.with(|r| r.is_empty());

    view! {
        <div class="space-y-6">
            <div class="text-center">
                <div class="flex items-center justify-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1">
                    <span class="text-gray-700 dark:text-gray-300">"Explore"</span>
                </div>
                <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Explore Repositories"</h1>
                <p class="mt-2 text-gray-600 dark:text-gray-400">
                    "Discover open-source projects and repositories."
                </p>
            </div>

            <div class="max-w-xl mx-auto">
                <form class="flex gap-2" on:submit=handle_search>
                    <div class="flex-1">
                        <Input
                            label="Search"
                            input_type=InputType::Text
                            name="q"
                            id="q"
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
                <div class="max-w-4xl mx-auto">
                    <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_error.set(None)) />
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
                                <A href=format!("/repos/{}", repo.full_name)>
                                    <Card class=card_class>
                                        <div class="flex items-start justify-between gap-2">
                                            <h2 class="font-semibold text-blue-600 dark:text-blue-400 hover:underline truncate">
                                                {repo.name.clone()}
                                            </h2>
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
                                            <span title="Owner">"Owner: "{truncate_uuid(&repo.owner_id.to_string(), 8)}</span>
                                            <span>"Created: "{format!("{}", repo.created_at.format("%b %d, %Y"))}</span>
                                            <span>"Updated: "{format!("{}", repo.updated_at.format("%b %d, %Y"))}</span>
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
