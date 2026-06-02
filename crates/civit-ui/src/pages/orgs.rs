#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::components::{Badge, BadgeColor, Button, ButtonVariant, Card};

#[component]
pub fn OrgsPage() -> impl IntoView {
    let (orgs_sig, _) = signal(vec![
        (
            "civitforge",
            "CivitForge Platform",
            "public",
            "12 members",
            "5 repos",
        ),
        (
            "rustdev",
            "Rust Developer Community",
            "public",
            "48 members",
            "23 repos",
        ),
        (
            "myteam",
            "My Team Workspace",
            "private",
            "6 members",
            "3 repos",
        ),
    ]);

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between flex-wrap gap-4">
                <div>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Organizations"</h1>
                    <p class="mt-1 text-gray-600 dark:text-gray-400">"Manage your organizations."</p>
                </div>
                <Button variant=ButtonVariant::Primary extra_class="btn-new-org">
                    "New Organization"
                </Button>
            </div>

            <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                <For each=move || orgs_sig.get() key=|o| o.0.to_string() let:org>
                    {
                        view! {
                            <A href=format!("/orgs/{}", org.0)>
                                <Card>
                                    <div class="flex items-center gap-3">
                                        <div class="w-12 h-12 rounded-full bg-blue-600 dark:bg-blue-500 flex items-center justify-center text-white font-bold text-lg select-none">
                                            {org.0.chars().next().unwrap_or_default().to_uppercase().collect::<String>()}
                                        </div>
                                        <div>
                                            <div class="flex items-center gap-2">
                                                <span class="font-semibold text-gray-900 dark:text-gray-100">{org.0}</span>
                                                <Badge
                                                    color=if org.2 == "public" { BadgeColor::Success } else { BadgeColor::Neutral }
                                                    text=org.2.to_string()
                                                />
                                            </div>
                                            <p class="text-sm text-gray-500 dark:text-gray-400">{org.1}</p>
                                        </div>
                                    </div>
                                    <div class="mt-4 flex gap-4 text-sm text-gray-500 dark:text-gray-400">
                                        <span>{org.3}</span>
                                        <span>{org.4}</span>
                                    </div>
                                </Card>
                            </A>
                        }
                    }
                </For>
            </div>
        </div>
    }
}

#[component]
pub fn OrgDetailPage() -> impl IntoView {
    let params = use_params_map();
    let org_id = move || params.with(|p| p.get("id").unwrap_or_default());

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between flex-wrap gap-4">
                <div>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">
                        {move || format!("Organization {}", org_id())}
                    </h1>
                    <p class="mt-1 text-gray-600 dark:text-gray-400">
                        "Organization details and management."
                    </p>
                </div>
                <Button variant=ButtonVariant::Primary extra_class="btn-edit-org">
                    "Edit Organization"
                </Button>
            </div>

            <div class="grid gap-4 sm:grid-cols-3">
                <Card>
                    <div class="text-center">
                        <div class="text-2xl font-bold text-gray-900 dark:text-gray-100">"12"</div>
                        <div class="text-sm text-gray-500 dark:text-gray-400">"Members"</div>
                    </div>
                </Card>
                <Card>
                    <div class="text-center">
                        <div class="text-2xl font-bold text-gray-900 dark:text-gray-100">"5"</div>
                        <div class="text-sm text-gray-500 dark:text-gray-400">"Repositories"</div>
                    </div>
                </Card>
                <Card>
                    <div class="text-center">
                        <div class="text-2xl font-bold text-gray-900 dark:text-gray-100">"3"</div>
                        <div class="text-sm text-gray-500 dark:text-gray-400">"Teams"</div>
                    </div>
                </Card>
            </div>

            <Card title="Repositories">
                <div class="py-8 text-center text-gray-400 dark:text-gray-500">
                    "Organization repositories will be listed here."
                </div>
            </Card>
        </div>
    }
}
