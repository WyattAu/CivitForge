#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::{Badge, BadgeColor, Card};

#[component]
pub fn HomePage() -> impl IntoView {
    let (repos_sig, _) = signal(vec![
        ("civitforge/civitforge", "Core platform", "public"),
        ("civitforge/cli", "Command-line interface", "public"),
        ("civitforge/runner", "CI/CD runner", "private"),
    ]);

    view! {
        <div class="space-y-8">
            <div>
                <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Dashboard"</h1>
                <p class="mt-2 text-gray-600 dark:text-gray-400">
                    "Welcome to CivitForge. Here's an overview of your recent activity."
                </p>
            </div>

            <Card title="Recent Repositories" description="Your latest repositories">
                <div class="space-y-3">
                    <For each=move || repos_sig.get() key=|r| r.0.to_string() let:repo>
                        {
                            view! {
                                <div class="flex items-center justify-between py-2 border-b border-gray-100 dark:border-gray-700 last:border-0">
                                    <A href=format!("/repos/{}", repo.0)>
                                        <span class="text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300 font-medium">{repo.0}</span>
                                    </A>
                                    <span class="text-sm text-gray-500 dark:text-gray-400">{repo.1}</span>
                                    <Badge
                                        color=if repo.2 == "public" { BadgeColor::Success } else { BadgeColor::Neutral }
                                        text=repo.2.to_string()
                                    />
                                </div>
                            }
                        }
                    </For>
                </div>
            </Card>

            <Card title="Activity" description="Recent events across your projects">
                <div class="py-8 text-center text-gray-400 dark:text-gray-500">
                    "No recent activity to display."
                </div>
            </Card>
        </div>
    }
}
