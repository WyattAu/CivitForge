#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::components::{Badge, BadgeColor, Button, ButtonVariant, Card, Spinner, TabItem, Tabs};
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

    leptos::task::spawn_local(async move {
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        let owner_val = owner();
        let name_val = name();
        let path = format!("/repos/{owner_val}/{name_val}");
        match client.get(&path).await {
            Ok(resp) if resp.status().is_success() => match resp.json::<RepoResponse>().await {
                Ok(data) => set_repo.set(Some(data)),
                Err(e) => set_error.set(Some(format!("Failed to parse repo: {e}"))),
            },
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                set_error.set(Some(format!("Failed to load repo ({status}): {body}")));
            }
            Err(e) => {
                set_error.set(Some(format!("Network error: {e}")));
            }
        }
        set_loading.set(false);
    });

    let (active_tab, set_active_tab) = signal("code".to_string());
    let (tabs_sig, _) = signal(vec![
        TabItem {
            id: "code".into(),
            label: "Code".into(),
        },
        TabItem {
            id: "issues".into(),
            label: "Issues".into(),
        },
        TabItem {
            id: "wiki".into(),
            label: "Wiki".into(),
        },
        TabItem {
            id: "settings".into(),
            label: "Settings".into(),
        },
    ]);

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between flex-wrap gap-4">
                <div>
                    <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1">
                        <A href="/repos"><span class="hover:text-blue-600 dark:hover:text-blue-400">"Repositories"</span></A>
                        <span>"/"</span>
                        <span class="text-gray-700 dark:text-gray-300">{owner}</span>
                    </div>

                    <Show when=move || loading.get() fallback=|| view! { <div></div> }>
                        <div class="flex items-center gap-2">
                            <Spinner />
                            <span class="text-gray-500 dark:text-gray-400">"Loading repository..."</span>
                        </div>
                    </Show>

                    <Show when=move || !loading.get() && repo_sig.get().is_some() fallback=|| view! { <div></div> }>
                        {move || {
                            repo_sig.get().map(|repo| {
                                let visibility_text = repo.visibility.to_string();
                                let badge_color = match repo.visibility {
                                    civit_shared::visibility::Visibility::Public => BadgeColor::Success,
                                    civit_shared::visibility::Visibility::Internal => BadgeColor::Info,
                                    civit_shared::visibility::Visibility::Private => BadgeColor::Neutral,
                                };
                                view! {
                                    <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100 flex items-center gap-3">
                                        {repo.full_name.clone()}
                                        <Badge color=badge_color text=visibility_text />
                                    </h1>
                                    <p class="mt-2 text-gray-600 dark:text-gray-400">
                                        {repo.description.clone().unwrap_or_else(|| "No description provided.".to_string())}
                                    </p>
                                }
                            })
                        }}
                    </Show>

                    <Show when=move || error.get().is_some() fallback=|| view! { <div></div> }>
                        <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md mt-2">
                            <p class="text-sm text-red-700 dark:text-red-400">{move || error.get().unwrap_or_default()}</p>
                        </div>
                    </Show>
                </div>
                <div class="flex gap-2">
                    <Button variant=ButtonVariant::Secondary>"Fork"</Button>
                    <Button variant=ButtonVariant::Primary>"Code"</Button>
                </div>
            </div>

            <Show when=move || !loading.get() && repo_sig.get().is_some() fallback=|| view! { <div></div> }>
                <Tabs
                    tabs=tabs_sig.get()
                    active_tab=active_tab.get()
                    on_change=Callback::new(move |id: String| set_active_tab.set(id))
                >
                    <Show when=move || active_tab.get() == "code" fallback=|| view! { <div class="hidden"></div> }>
                        {move || {
                            repo_sig.get().map(|repo| {
                                view! {
                                    <Card title="README" description="Project documentation">
                                        <div class="prose dark:prose-invert max-w-none text-gray-600 dark:text-gray-400">
                                            <p>{format!("# {}", repo.name)}</p>
                                            <p>{repo.description.clone().unwrap_or_else(|| "No README yet.".to_string())}</p>
                                        </div>
                                    </Card>

                                    <div class="mt-4 grid grid-cols-2 sm:grid-cols-4 gap-4">
                                        <Card>
                                            <div class="text-center">
                                                <div class="text-2xl font-bold text-gray-900 dark:text-gray-100">{repo.visibility.to_string()}</div>
                                                <div class="text-sm text-gray-500 dark:text-gray-400">"Visibility"</div>
                                            </div>
                                        </Card>
                                        <Card>
                                            <div class="text-center">
                                                <div class="text-2xl font-bold text-gray-900 dark:text-gray-100">{repo.default_branch.clone()}</div>
                                                <div class="text-sm text-gray-500 dark:text-gray-400">"Default Branch"</div>
                                            </div>
                                        </Card>
                                        <Card>
                                            <div class="text-center">
                                                <div class="text-sm font-bold text-gray-900 dark:text-gray-100">{repo.created_at.format("%b %d, %Y").to_string()}</div>
                                                <div class="text-sm text-gray-500 dark:text-gray-400">"Created"</div>
                                            </div>
                                        </Card>
                                        <Card>
                                            <div class="text-center">
                                                <div class="text-sm font-bold text-gray-900 dark:text-gray-100">{repo.updated_at.format("%b %d, %Y").to_string()}</div>
                                                <div class="text-sm text-gray-500 dark:text-gray-400">"Updated"</div>
                                            </div>
                                        </Card>
                                    </div>
                                }
                            })
                        }}
                    </Show>
                </Tabs>
            </Show>
        </div>
    }
}
