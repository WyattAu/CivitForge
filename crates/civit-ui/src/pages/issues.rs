#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::components::{Badge, BadgeColor, Button, ButtonVariant, Card, Pagination};

#[component]
pub fn IssuesPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());

    let (page, set_page) = signal(1u32);

    let (issues_sig, _) = signal(vec![
        ("Bug: CI pipeline fails on main branch", "open", "bug"),
        ("Feature: Add webhook support", "open", "enhancement"),
        ("Docs: Update API documentation", "open", "documentation"),
        ("Fix: Handle empty repository clone", "closed", "bug"),
    ]);

    let open_count = 8;

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between flex-wrap gap-4">
                <div>
                    <div class="text-sm text-gray-500 dark:text-gray-400 mb-1">
                        {move || format!("{}/{}", owner(), name())}
                    </div>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Issues"</h1>
                </div>
                <Button variant=ButtonVariant::Primary extra_class="btn-new-issue">
                    "New Issue"
                </Button>
            </div>

            <div class="flex gap-3">
                <Badge color=BadgeColor::Info text=format!("{open_count} Open") />
                <Badge color=BadgeColor::Neutral text="3 Closed".to_string() />
            </div>

            <Card>
                <div class="divide-y divide-gray-100 dark:divide-gray-700">
                    <For each=move || issues_sig.get() key=|i| i.0.to_string() let:issue>
                        {
                            view! {
                                <div class="flex items-center justify-between py-3 px-1">
                                    <div class="flex items-center gap-3">
                                        <svg class="w-5 h-5 text-gray-400" fill="currentColor" viewBox="0 0 20 20">
                                            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-12a1 1 0 10-2 0v4a1 1 0 00.293.707l2.828 2.829a1 1 0 101.415-1.415L11 9.586V6z" clip-rule="evenodd"/>
                                        </svg>
                                        <span class="text-sm font-medium text-gray-900 dark:text-gray-100">
                                            {issue.0}
                                        </span>
                                        <Badge
                                            color=match issue.2 {
                                                "bug" => BadgeColor::Danger,
                                                "enhancement" => BadgeColor::Info,
                                                _ => BadgeColor::Neutral,
                                            }
                                            text=issue.2.to_string()
                                        />
                                    </div>
                                    <div class="flex items-center gap-2">
                                        <Badge color=if issue.1 == "open" { BadgeColor::Success } else { BadgeColor::Neutral } text=issue.1.to_string() />
                                    </div>
                                </div>
                            }
                        }
                    </For>
                </div>
            </Card>

            <Pagination
                current_page=page.get()
                total_pages=5
                on_page_change=Callback::new(move |p: u32| set_page.set(p))
            />
        </div>
    }
}
