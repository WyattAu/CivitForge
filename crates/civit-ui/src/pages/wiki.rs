#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::components::Card;

#[component]
pub fn WikiPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());

    let (wiki_sig, _) = signal(vec![
        (
            "Home",
            "Project overview and getting started",
            "Updated 3 days ago",
        ),
        (
            "Architecture",
            "System design and component overview",
            "Updated 1 week ago",
        ),
        (
            "Contributing",
            "How to contribute to the project",
            "Updated 2 weeks ago",
        ),
        (
            "API Reference",
            "REST API documentation",
            "Updated 1 month ago",
        ),
    ]);

    view! {
        <div class="space-y-6">
            <div>
                <div class="text-sm text-gray-500 dark:text-gray-400 mb-1">
                    {move || format!("{}/{}", owner(), name())}
                </div>
                <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Wiki"</h1>
                <p class="mt-1 text-gray-600 dark:text-gray-400">"Documentation for this repository."</p>
            </div>

            <Card title="Home" description="Project overview and getting started">
                <div class="prose dark:prose-invert max-w-none text-gray-600 dark:text-gray-400">
                    <p>"This is the wiki home page."</p>
                </div>
            </Card>

            <Card title="All Pages">
                <div class="space-y-0">
                    <For each=move || wiki_sig.get() key=|p| p.0.to_string() let:page>
                        {
                            view! {
                                <div class="flex items-center justify-between py-3 border-b border-gray-100 dark:border-gray-700 last:border-0">
                                    <div>
                                        <span class="font-medium text-gray-900 dark:text-gray-100">{page.0}</span>
                                        <p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5">{page.1}</p>
                                    </div>
                                    <span class="text-xs text-gray-400 dark:text-gray-500">{page.2}</span>
                                </div>
                            }
                        }
                    </For>
                </div>
            </Card>
        </div>
    }
}
