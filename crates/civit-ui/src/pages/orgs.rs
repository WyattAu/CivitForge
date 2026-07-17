#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::api::types::ListResponse;
use crate::components::{
    Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Input, InputType, Modal, Spinner,
};
use crate::state::auth::use_auth;
use crate::utils::*;
use civit_shared::org::OrgResponse;
use civit_shared::visibility::Visibility;

#[derive(Debug, Clone, serde::Serialize)]
struct CreateOrgBody {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    visibility: Visibility,
}

#[component]
pub fn OrgsPage() -> impl IntoView {
    let auth = use_auth();
    let (orgs_sig, set_orgs) = signal(vec![]);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (show_create, set_show_create) = signal(false);
    let (create_error, set_create_error) = signal(None::<String>);
    let (create_loading, set_create_loading) = signal(false);

    let fetch_orgs = move || {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        leptos::task::spawn_local(async move {
            match client.get("/orgs?per_page=100&offset=0").await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<ListResponse<OrgResponse>>().await {
                        Ok(data) => set_orgs.set(data.data),
                        Err(_) => set_error.set(Some("Failed to process response.".to_string())),
                    }
                }
                Ok(_) => {
                    set_error.set(Some("Failed to load organizations.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_loading.set(false);
        });
    };

    fetch_orgs();

    let open_create = move |_| set_show_create.set(true);
    let close_create = Callback::new(move |_: ()| set_show_create.set(false));

    let handle_create_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_create_error.set(None);

        let name_val = get_input_value("org-name");
        let display_name_val = get_input_value("org-display-name");
        let desc_val = get_input_value("org-description");
        let vis_public = get_input_value("org-vis-public");
        let vis_private = get_input_value("org-vis-private");

        if name_val.trim().is_empty() {
            set_create_error.set(Some("Organization name is required.".to_string()));
            return;
        }

        let vis = if !vis_public.is_empty() && vis_public == "on" {
            Visibility::Public
        } else if !vis_private.is_empty() && vis_private == "on" {
            Visibility::Private
        } else {
            Visibility::Public
        };

        let body = CreateOrgBody {
            name: name_val.trim().to_string(),
            display_name: if display_name_val.trim().is_empty() {
                None
            } else {
                Some(display_name_val.trim().to_string())
            },
            description: if desc_val.trim().is_empty() {
                None
            } else {
                Some(desc_val.trim().to_string())
            },
            visibility: vis,
        };

        set_create_loading.set(true);
        leptos::task::spawn_local(async move {
            let token = auth.0.with(|a| a.token.clone());
            let client = ApiClient::new(token);
            match client.post("/orgs", &body).await {
                Ok(resp) if resp.status().is_success() => {
                    set_show_create.set(false);
                    fetch_orgs();
                }
                Ok(_) => {
                    set_create_error.set(Some("Failed to create organization.".to_string()));
                }
                Err(_) => {
                    set_create_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_create_loading.set(false);
        });
    };

    let dismiss_error = Callback::new(move |_: ()| set_error.set(None));

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between flex-wrap gap-4">
                <div>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Organizations"</h1>
                    <p class="mt-1 text-gray-600 dark:text-gray-400">"Manage your organizations."</p>
                </div>
                <Button variant=ButtonVariant::Primary on:click=open_create extra_class="btn-new-org">
                    "New Organization"
                </Button>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=dismiss_error />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div></div> }>
                <div class="flex items-center justify-center py-12">
                    <Spinner />
                </div>
            </Show>

            <Show when=move || !loading.get() && orgs_sig.with(|o| o.is_empty()) && error.get().is_none() fallback=|| view! { <div></div> }>
                <Card>
                    <div class="py-12 text-center">
                        <p class="text-gray-500 dark:text-gray-400 text-lg">
                            "No organizations yet. Create one to collaborate with your team!"
                        </p>
                    </div>
                </Card>
            </Show>

            <Show when=move || !loading.get() && !orgs_sig.with(|o| o.is_empty()) fallback=|| view! { <div></div> }>
                <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                    <For each=move || orgs_sig.get() key=|o| o.id let:org>
                        {
                            let org = org.clone();
                            let card_class = "hover:border-blue-300 dark:hover:border-blue-700 transition-colors cursor-pointer".to_string();
                            view! {
                                <A href=format!("/orgs/{}", org.id)>
                                    <Card class=card_class>
                                        <div class="flex items-center gap-3">
                                            <div class="w-12 h-12 rounded-full bg-blue-600 dark:bg-blue-500 flex items-center justify-center text-white font-bold text-lg select-none">
                                                {org.name.chars().next().unwrap_or_default().to_uppercase().collect::<String>()}
                                            </div>
                                            <div class="min-w-0">
                                                <div class="flex items-center gap-2">
                                                    <span class="font-semibold text-gray-900 dark:text-gray-100 truncate">
                                                        {org.display_name.clone().unwrap_or(org.name.clone())}
                                                    </span>
                                                    <Badge
                                                        color=match org.visibility {
                                                            Visibility::Public => BadgeColor::Success,
                                                            Visibility::Internal => BadgeColor::Info,
                                                            Visibility::Private => BadgeColor::Neutral,
                                                        }
                                                        text=org.visibility.to_string()
                                                    />
                                                </div>
                                                <p class="text-sm text-gray-500 dark:text-gray-400 truncate">
                                                    {org.name.clone()}
                                                </p>
                                            </div>
                                        </div>

                                        {org.description.as_ref().map(|desc| {
                                            view! {
                                                <p class="mt-2 text-sm text-gray-500 dark:text-gray-400 line-clamp-2">
                                                    {desc.clone()}
                                                </p>
                                            }
                                        })}

                                        <div class="mt-4 flex gap-4 text-sm text-gray-500 dark:text-gray-400">
                                            <span>{format!("{} members", org.member_count)}</span>
                                            <span>{format!("{} repos", org.repo_count)}</span>
                                        </div>
                                    </Card>
                                </A>
                            }
                        }
                    </For>
                </div>
            </Show>

            <Modal
                show=show_create.get()
                title="Create Organization".to_string()
                on_close=close_create
            >
                <form on:submit=handle_create_submit class="space-y-4">
                    <Show when=move || create_error.get().is_some()>
                        <ErrorBanner message=move || create_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_create_error.set(None)) />
                    </Show>

                    <Input
                        label="Organization Name"
                        name="org-name"
                        id="org-name"
                        input_type=InputType::Text
                        placeholder="my-org"
                        required=true
                    />

                    <Input
                        label="Display Name (optional)"
                        name="org-display-name"
                        id="org-display-name"
                        input_type=InputType::Text
                        placeholder="My Organization"
                    />

                    <Input
                        label="Description (optional)"
                        name="org-description"
                        id="org-description"
                        input_type=InputType::Textarea
                        placeholder="What is this organization about?"
                    />

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            "Visibility"
                        </label>
                        <div class="space-y-2">
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input type="radio" name="org-visibility" id="org-vis-public" value="public" checked
                                    class="w-4 h-4 text-blue-600 border-gray-300 focus:ring-blue-500 dark:border-gray-600" />
                                <span class="text-sm text-gray-900 dark:text-gray-100">"Public"</span>
                            </label>
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input type="radio" name="org-visibility" id="org-vis-private" value="private"
                                    class="w-4 h-4 text-blue-600 border-gray-300 focus:ring-blue-500 dark:border-gray-600" />
                                <span class="text-sm text-gray-900 dark:text-gray-100">"Private"</span>
                            </label>
                        </div>
                    </div>

                    <div class="flex gap-3 pt-2">
                        <Button variant=ButtonVariant::Primary disabled=create_loading.get()>
                            {move || if create_loading.get() { "Creating..." } else { "Create" }}
                        </Button>
                        <Button variant=ButtonVariant::Secondary on:click=move |_| set_show_create.set(false)>
                            "Cancel"
                        </Button>
                    </div>
                </form>
            </Modal>
        </div>
    }
}

#[component]
pub fn OrgDetailPage() -> impl IntoView {
    let params = use_params_map();
    let org_id = move || params.with(|p| p.get("id").unwrap_or_default());
    let auth = use_auth();

    let (org_sig, set_org) = signal(None::<OrgResponse>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let fetch_org = move || {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        let id = org_id();
        if id.is_empty() {
            set_loading.set(false);
            return;
        }
        leptos::task::spawn_local(async move {
            match client.get(&format!("/orgs/{id}")).await {
                Ok(resp) if resp.status().is_success() => match resp.json::<OrgResponse>().await {
                    Ok(data) => set_org.set(Some(data)),
                    Err(_) => set_error.set(Some("Failed to load organization.".to_string())),
                },
                Ok(_) => {
                    set_error.set(Some("Organization not found.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_loading.set(false);
        });
    };

    fetch_org();

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between flex-wrap gap-4">
                <div>
                    <Show when=move || org_sig.get().is_some() fallback=|| view! { <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Organization"</h1> }>
                        <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">
                            {move || org_sig.get().map(|o| o.display_name.clone().unwrap_or(o.name.clone())).unwrap_or_default()}
                        </h1>
                    </Show>
                    <p class="mt-1 text-gray-600 dark:text-gray-400">
                        "Organization details and management."
                    </p>
                </div>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center justify-center py-12">
                    <Spinner />
                </div>
            </Show>

            <Show when=move || org_sig.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                {move || {
                    let org = org_sig.get().expect("key present");
                    let member_count = org.member_count;
                    let repo_count = org.repo_count;
                    let vis = org.visibility;
                    let desc_text = org.description.clone().unwrap_or_default();
                    let has_desc = org.description.is_some();
                    let (desc_sig, _) = signal(desc_text);
                    view! {
                        <div class="grid gap-4 sm:grid-cols-3">
                            <Card>
                                <div class="text-center">
                                    <div class="text-2xl font-bold text-gray-900 dark:text-gray-100">{member_count}</div>
                                    <div class="text-sm text-gray-500 dark:text-gray-400">"Members"</div>
                                </div>
                            </Card>
                            <Card>
                                <div class="text-center">
                                    <div class="text-2xl font-bold text-gray-900 dark:text-gray-100">{repo_count}</div>
                                    <div class="text-sm text-gray-500 dark:text-gray-400">"Repositories"</div>
                                </div>
                            </Card>
                            <Card>
                                <div class="text-center flex items-center justify-center gap-2">
                                    <Badge
                                        color=match vis {
                                            Visibility::Public => BadgeColor::Success,
                                            Visibility::Internal => BadgeColor::Info,
                                            Visibility::Private => BadgeColor::Neutral,
                                        }
                                        text=vis.to_string()
                                    />
                                </div>
                            </Card>
                        </div>

                        <Show when=move || has_desc fallback=|| view! { <div class="hidden"></div> }>
                            <Card title="Description".to_string()>
                                <p class="text-sm text-gray-700 dark:text-gray-300">{move || desc_sig.get()}</p>
                            </Card>
                        </Show>

                        <Card title="Repositories".to_string()>
                            <div class="py-8 text-center text-gray-400 dark:text-gray-500">
                                "Organization repositories will be listed here."
                            </div>
                        </Card>
                    }
                }}
            </Show>
        </div>
    }
}
