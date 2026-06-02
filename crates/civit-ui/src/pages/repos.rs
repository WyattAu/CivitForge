#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::{Badge, BadgeColor, Button, ButtonVariant, Card};

#[component]
pub fn ReposPage() -> impl IntoView {
    let (repos_sig, _) = signal(vec![
        (
            "civitforge/civitforge",
            "Core platform repository",
            "public",
            "Updated 2 hours ago",
        ),
        (
            "civitforge/cli",
            "Command-line interface",
            "public",
            "Updated 1 day ago",
        ),
        (
            "civitforge/runner",
            "CI/CD runner agent",
            "private",
            "Updated 3 days ago",
        ),
        (
            "myuser/myproject",
            "Personal project",
            "private",
            "Updated 1 week ago",
        ),
    ]);

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Repositories"</h1>
                    <p class="mt-1 text-gray-600 dark:text-gray-400">"Browse and manage your repositories."</p>
                </div>
                <Button variant=ButtonVariant::Primary>"New Repository"</Button>
            </div>

            <Card>
                <div class="space-y-0">
                    <For each=move || repos_sig.get() key=|r| r.0.to_string() let:repo>
                        {
                            view! {
                        <A href=format!("/repos/{}", repo.0)>
                            <span class="flex items-center justify-between py-3 border-b border-gray-100 dark:border-gray-700 last:border-0 hover:bg-gray-50 dark:hover:bg-gray-750 px-2 -mx-2 rounded transition-colors">
                                    <div>
                                        <div class="flex items-center gap-2">
                                            <span class="font-medium text-blue-600 dark:text-blue-400">{repo.0}</span>
                                            <Badge
                                                color=if repo.2 == "public" { BadgeColor::Success } else { BadgeColor::Neutral }
                                                text=repo.2.to_string()
                                            />
                                        </div>
                                        <p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5">{repo.1}</p>
                                    </div>
                                    <span class="text-xs text-gray-400 dark:text-gray-500">{repo.3}</span>
                                </span>
                                </A>
                            }
                        }
                    </For>
                </div>
            </Card>
        </div>
    }
}
