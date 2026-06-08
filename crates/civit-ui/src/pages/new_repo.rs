#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

use crate::api::client::ApiClient;
use crate::components::{Button, ButtonVariant, Card, ErrorBanner, Input, InputType};
use crate::state::auth::use_auth;
use crate::utils::get_input_value;
use civit_shared::visibility::Visibility;

#[derive(Debug, Clone, serde::Serialize)]
struct CreateRepoBody {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    visibility: Visibility,
}

#[component]
pub fn NewRepoPage() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();
    let (error, set_error) = signal(None::<String>);
    let (loading, set_loading) = signal(false);
    let (navigate_sig, _) = signal(navigate);

    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);

        let name_val = get_input_value("repo-name");
        let desc_val = get_input_value("repo-description");

        if name_val.trim().is_empty() {
            set_error.set(Some("Repository name is required.".to_string()));
            return;
        }

        let vis_val = get_input_value("visibility-public");
        let vis_val2 = get_input_value("visibility-internal");
        let visibility_val = if !vis_val.is_empty() && vis_val == "on" {
            Visibility::Public
        } else if !vis_val2.is_empty() && vis_val2 == "on" {
            Visibility::Internal
        } else {
            Visibility::Private
        };

        let body = CreateRepoBody {
            name: name_val.trim().to_string(),
            description: if desc_val.trim().is_empty() {
                None
            } else {
                Some(desc_val.trim().to_string())
            },
            visibility: visibility_val,
        };

        set_loading.set(true);
        let nav = navigate_sig.get();
        leptos::task::spawn_local(async move {
            let token = auth.0.with(|a| a.token.clone());
            let client = ApiClient::new(token);
            let result = client.post("/repos", &body).await;

            match result {
                Ok(resp) if resp.status().is_success() => {
                    let owner_id = auth.0.with(|a| a.user_id.clone().unwrap_or_default());
                    let repo_name = body.name.clone();
                    nav(
                        &format!("/repos/{owner_id}/{repo_name}"),
                        Default::default(),
                    );
                }
                Ok(_) => {
                    set_error.set(Some("Failed to create repository.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_loading.set(false);
        });
    };

    let username = move || auth.0.with(|a| a.username.clone().unwrap_or_default());

    view! {
        <div class="max-w-2xl mx-auto space-y-6">
            <div>
                <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"New Repository"</h1>
                <p class="mt-1 text-gray-600 dark:text-gray-400">
                    "Create a new repository on CivitForge."
                </p>
            </div>

            <Card>
                <form on:submit=handle_submit class="space-y-5">
                    <Show when=move || error.get().is_some()>
                        <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_error.set(None)) />
                    </Show>

                    <Input
                        label="Repository Name"
                        name="repo-name"
                        id="repo-name"
                        input_type=InputType::Text
                        placeholder="my-awesome-project"
                        required=true
                    />

                    <Input
                        label="Description (optional)"
                        name="repo-description"
                        id="repo-description"
                        input_type=InputType::Textarea
                        placeholder="A brief description of your project..."
                    />

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            "Owner"
                        </label>
                        <div class="px-3 py-2 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 rounded-md text-sm text-gray-500 dark:text-gray-400">
                            {username}
                        </div>
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                            "Visibility"
                        </label>
                        <div class="space-y-2">
                            <label class="flex items-center gap-3 cursor-pointer">
                                <input
                                    type="radio"
                                    name="visibility"
                                    id="visibility-public"
                                    value="public"
                                    checked
                                    class="w-4 h-4 text-blue-600 border-gray-300 focus:ring-blue-500 dark:border-gray-600"
                                />
                                <div>
                                    <span class="text-sm font-medium text-gray-900 dark:text-gray-100">"Public"</span>
                                    <p class="text-xs text-gray-500 dark:text-gray-400">"Anyone can see this repository."</p>
                                </div>
                            </label>
                            <label class="flex items-center gap-3 cursor-pointer">
                                <input
                                    type="radio"
                                    name="visibility"
                                    id="visibility-internal"
                                    value="internal"
                                    class="w-4 h-4 text-blue-600 border-gray-300 focus:ring-blue-500 dark:border-gray-600"
                                />
                                <div>
                                    <span class="text-sm font-medium text-gray-900 dark:text-gray-100">"Internal"</span>
                                    <p class="text-xs text-gray-500 dark:text-gray-400">"Only authenticated users can see this repository."</p>
                                </div>
                            </label>
                            <label class="flex items-center gap-3 cursor-pointer">
                                <input
                                    type="radio"
                                    name="visibility"
                                    id="visibility-private"
                                    value="private"
                                    class="w-4 h-4 text-blue-600 border-gray-300 focus:ring-blue-500 dark:border-gray-600"
                                />
                                <div>
                                    <span class="text-sm font-medium text-gray-900 dark:text-gray-100">"Private"</span>
                                    <p class="text-xs text-gray-500 dark:text-gray-400">"Only authorized users can see this repository."</p>
                                </div>
                            </label>
                        </div>
                    </div>

                    <div class="flex gap-3 pt-2">
                        <Button
                            variant=ButtonVariant::Primary
                            extra_class="btn-create-repo"
                            disabled=loading.get()
                        >
                            {move || if loading.get() { "Creating..." } else { "Create Repository" }}
                        </Button>
                        <A href="/repos">
                            <Button variant=ButtonVariant::Secondary>"Cancel"</Button>
                        </A>
                    </div>
                </form>
            </Card>
        </div>
    }
}
