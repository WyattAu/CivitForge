#![forbid(unsafe_code)]

use leptos::prelude::*;

use crate::components::{Avatar, Badge, BadgeColor, Button, ButtonVariant, Card, Input, InputType};

#[component]
pub fn ExplorePage() -> impl IntoView {
    let (query, _set_query) = signal(String::new());

    let (repos_sig, _) = signal(vec![
        (
            "rust-lang/rust",
            "The Rust programming language",
            "public",
            "72.4k",
            "rust",
        ),
        (
            "tokio-rs/tokio",
            "A runtime for writing async Rust",
            "public",
            "28.1k",
            "async",
        ),
        (
            "leptos-rs/leptos",
            "Build fast web apps with Rust",
            "public",
            "15.2k",
            "web",
        ),
        (
            "serde-rs/serde",
            "Serialization framework for Rust",
            "public",
            "8.9k",
            "serde",
        ),
    ]);

    let handle_search = move |_| {};

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

            <Card title="Trending" description="Popular repositories on CivitForge">
                <div class="space-y-0">
                    <For each=move || repos_sig.get() key=|r| r.0.to_string() let:repo>
                        {
                            view! {
                                <div class="flex items-center justify-between py-3 border-b border-gray-100 dark:border-gray-700 last:border-0">
                                    <div class="flex items-center gap-3">
                                        <Avatar name=repo.0.to_string() size=32 />
                                        <div>
                                            <div class="flex items-center gap-2">
                                                <span class="font-medium text-blue-600 dark:text-blue-400">{repo.0}</span>
                                                <Badge color=BadgeColor::Success text=repo.2.to_string() />
                                            </div>
                                            <p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5">{repo.1}</p>
                                        </div>
                                    </div>
                                    <div class="flex items-center gap-4 text-sm text-gray-500 dark:text-gray-400">
                                        <Badge color=BadgeColor::Info text=repo.3.to_string() />
                                        <Badge color=BadgeColor::Neutral text=repo.4.to_string() />
                                    </div>
                                </div>
                            }
                        }
                    </For>
                </div>
            </Card>
        </div>
    }
}
