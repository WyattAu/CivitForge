#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;

use crate::api::client::ApiClient;
use crate::api::types::ListResponse;
use crate::components::{Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Spinner};
use crate::state::auth::use_auth;
use civit_shared::repo::RepoResponse;
use civit_shared::visibility::Visibility;

fn format_datetime(dt: &chrono::DateTime<chrono::Utc>) -> String {
    dt.format("%b %d, %Y").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_datetime_basic() {
        let dt = chrono::DateTime::parse_from_rfc3339("2024-03-15T10:30:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        assert_eq!(format_datetime(&dt), "Mar 15, 2024");
    }

    #[test]
    fn format_datetime_month_names() {
        let cases = [
            ("2024-01-01T00:00:00Z", "Jan 01, 2024"),
            ("2024-06-15T00:00:00Z", "Jun 15, 2024"),
            ("2024-12-25T00:00:00Z", "Dec 25, 2024"),
        ];
        for (input, expected) in cases {
            let dt = chrono::DateTime::parse_from_rfc3339(input)
                .unwrap()
                .with_timezone(&chrono::Utc);
            assert_eq!(format_datetime(&dt), expected, "failed for {input}");
        }
    }
}

#[component]
pub fn ReposPage() -> impl IntoView {
    let auth = use_auth();
    let (repos_sig, set_repos) = signal(vec![]);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let fetch_repos = move || {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        leptos::task::spawn_local(async move {
            match client.get("/repos?per_page=50&offset=0").await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<ListResponse<RepoResponse>>().await {
                        Ok(data) => set_repos.set(data.data),
                        Err(_) => set_error.set(Some("Failed to load repositories.".to_string())),
                    }
                }
                Ok(_) => {
                    set_error.set(Some("Failed to load repositories.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_loading.set(false);
        });
    };

    fetch_repos();

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1">
                        <span class="text-gray-700 dark:text-gray-300">"Repositories"</span>
                    </div>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Repositories"</h1>
                    <p class="mt-1 text-gray-600 dark:text-gray-400">"Browse and manage your repositories."</p>
                </div>
                <A href="/new-repo">
                    <Button variant=ButtonVariant::Primary>"New Repository"</Button>
                </A>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_error.set(None)) />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center justify-center py-12">
                    <Spinner />
                </div>
            </Show>

            <Show when=move || !loading.get() && repos_sig.with(|r| r.is_empty()) && error.get().is_none() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="py-12 text-center">
                        <p class="text-gray-500 dark:text-gray-400 text-lg">
                            "No repositories yet. Create one to get started!"
                        </p>
                    </div>
                </Card>
            </Show>

            <Show when=move || !loading.get() && !repos_sig.with(|r| r.is_empty()) fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="divide-y divide-gray-100 dark:divide-gray-700">
                        <For each=move || repos_sig.get() key=|r| r.id let:repo>
                            {
                                let full_name_v = repo.full_name.clone();
                                let updated_v = format_datetime(&repo.updated_at);
                                view! {
                                    <A href=format!("/repos/{}", full_name_v)>
                                        <div class="flex items-center justify-between py-3 px-1 hover:bg-gray-50 dark:hover:bg-gray-750 -mx-1 rounded transition-colors cursor-pointer">
                                            <div class="min-w-0">
                                                <div class="flex items-center gap-2">
                                                    <span class="font-medium text-blue-600 dark:text-blue-400">{full_name_v.clone()}</span>
                                                    <Badge
                                                        color=match repo.visibility {
                                                            Visibility::Public => BadgeColor::Success,
                                                            Visibility::Internal => BadgeColor::Info,
                                                            Visibility::Private => BadgeColor::Neutral,
                                                        }
                                                        text=repo.visibility.to_string()
                                                    />
                                                </div>
                                                {repo.description.as_ref().map(|desc| {
                                                    view! {
                                                        <p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5 truncate">{desc.clone()}</p>
                                                    }
                                                })}
                                            </div>
                                            <span class="text-xs text-gray-400 dark:text-gray-500 shrink-0 ml-2">{updated_v}</span>
                                        </div>
                                    </A>
                                }
                            }
                        </For>
                    </div>
                </Card>
            </Show>
        </div>
    }
}
