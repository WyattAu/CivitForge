#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::components::{Badge, BadgeColor, Button, ButtonVariant, Card, TabItem, Tabs};

#[component]
pub fn RepoDetailPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());

    let tabs = vec![
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
    ];

    let (active_tab, set_active_tab) = signal("code".to_string());

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between flex-wrap gap-4">
                <div>
                    <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1">
                        <A href="/repos"><span class="hover:text-blue-600 dark:hover:text-blue-400">"Repositories"</span></A>
                        <span>"/"</span>
                        <span class="text-gray-700 dark:text-gray-300">{owner}</span>
                    </div>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100 flex items-center gap-3">
                        {owner} "/" {name}
                        <Badge color=BadgeColor::Success text="public".to_string() />
                    </h1>
                    <p class="mt-2 text-gray-600 dark:text-gray-400">
                        {move || format!("{} / {}", owner(), name())}
                    </p>
                </div>
                <div class="flex gap-2">
                    <Button variant=ButtonVariant::Secondary>"Fork"</Button>
                    <Button variant=ButtonVariant::Primary>"Code"</Button>
                </div>
            </div>

            <Tabs
                tabs=tabs
                active_tab=active_tab.get()
                on_change=Callback::new(move |id: String| set_active_tab.set(id))
            >
                <Show when=move || active_tab.get() == "code" fallback=|| view! { <div class="hidden"></div> }>
                    <Card title="README" description="Project documentation">
                        <div class="prose dark:prose-invert max-w-none text-gray-600 dark:text-gray-400">
                            <p>{move || format!("# {}", name())}</p>
                            <p>"This is the README for the repository."</p>
                        </div>
                    </Card>

                    <div class="mt-4 grid grid-cols-2 sm:grid-cols-4 gap-4">
                        <Card>
                            <div class="text-center">
                                <div class="text-2xl font-bold text-gray-900 dark:text-gray-100">"12"</div>
                                <div class="text-sm text-gray-500 dark:text-gray-400">"Stars"</div>
                            </div>
                        </Card>
                        <Card>
                            <div class="text-center">
                                <div class="text-2xl font-bold text-gray-900 dark:text-gray-100">"3"</div>
                                <div class="text-sm text-gray-500 dark:text-gray-400">"Forks"</div>
                            </div>
                        </Card>
                        <Card>
                            <div class="text-center">
                                <div class="text-2xl font-bold text-gray-900 dark:text-gray-100">"8"</div>
                                <div class="text-sm text-gray-500 dark:text-gray-400">"Issues"</div>
                            </div>
                        </Card>
                        <Card>
                            <div class="text-center">
                                <div class="text-2xl font-bold text-gray-900 dark:text-gray-100">"2"</div>
                                <div class="text-sm text-gray-500 dark:text-gray-400">"Branches"</div>
                            </div>
                        </Card>
                    </div>
                </Show>
            </Tabs>
        </div>
    }
}
