#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::components::{
    Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Input, InputType, Modal, Spinner,
};
use crate::state::auth::use_auth;
use crate::utils::get_input_value;

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
struct Deployment {
    id: String,
    repo_id: String,
    sha: String,
    environment: String,
    status: String,
    creator: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct CreateDeploymentBody {
    sha: String,
    environment: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct UpdateDeploymentStatusBody {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

fn status_color(status: &str) -> BadgeColor {
    match status {
        "success" => BadgeColor::Success,
        "in_progress" | "pending" => BadgeColor::Warning,
        "failure" | "error" | "cancelled" => BadgeColor::Danger,
        _ => BadgeColor::Neutral,
    }
}

#[component]
pub fn DeploymentsPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|p| p.get("name").unwrap_or_default());
    let auth = use_auth();

    let (deployments, set_deployments) = signal(Vec::<Deployment>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let (show_create, set_show_create) = signal(false);
    let (creating, set_creating) = signal(false);

    let (show_status_modal, set_show_status_modal) = signal(false);
    let (selected_deploy_id, set_selected_deploy_id) = signal(None::<String>);

    let fetch_deployments = move || {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        let o = owner();
        let r = repo_name();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{o}/{r}/deployments");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<Vec<Deployment>>().await {
                        set_deployments.set(data);
                    }
                }
                Ok(_) => {
                    set_error.set(Some("Failed to load deployments.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error.".to_string()));
                }
            }
            set_loading.set(false);
        });
    };

    fetch_deployments();

    let handle_create = move |_: leptos::ev::MouseEvent| {
        let sha = get_input_value("deploy-sha");
        let environment = get_input_value("deploy-environment");
        let description = get_input_value("deploy-description");

        if sha.trim().is_empty() || environment.trim().is_empty() {
            set_error.set(Some("SHA and environment are required.".to_string()));
            return;
        }

        let body = CreateDeploymentBody {
            sha: sha.trim().to_string(),
            environment: environment.trim().to_string(),
            description: if description.trim().is_empty() {
                None
            } else {
                Some(description.trim().to_string())
            },
        };

        let token = auth.0.with(|a| a.token.clone());
        let o = owner();
        let r = repo_name();
        set_creating.set(true);
        set_error.set(None);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{o}/{r}/deployments");
            match client.post(&path, &body).await {
                Ok(resp) if resp.status().is_success() => {
                    set_show_create.set(false);
                    fetch_deployments();
                }
                Ok(_) => {
                    set_error.set(Some("Failed to create deployment.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error.".to_string()));
                }
            }
            set_creating.set(false);
        });
    };

    let handle_update_status = move |deploy_id: String| {
        let new_status = get_input_value("deploy-status");
        let description = get_input_value("deploy-status-desc");

        if new_status.trim().is_empty() {
            set_error.set(Some("Status is required.".to_string()));
            return;
        }

        let body = UpdateDeploymentStatusBody {
            status: new_status.trim().to_string(),
            description: if description.trim().is_empty() {
                None
            } else {
                Some(description.trim().to_string())
            },
        };

        let token = auth.0.with(|a| a.token.clone());
        let o = owner();
        let r = repo_name();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{o}/{r}/deployments/{deploy_id}/statuses");
            match client.post(&path, &body).await {
                Ok(resp) if resp.status().is_success() => {
                    set_show_status_modal.set(false);
                    set_selected_deploy_id.set(None);
                    fetch_deployments();
                }
                Ok(_) => {
                    set_error.set(Some("Failed to update status.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error.".to_string()));
                }
            }
        });
    };

    let dismiss_error = Callback::new(move |_: ()| set_error.set(None));

    view! {
        <div class="space-y-6">
            <div class="flex items-start justify-between">
                <div>
                    <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1">
                        <a href=format!("/repos/{}/{}", owner(), repo_name()) class="text-blue-600 dark:text-blue-400 hover:underline">
                            {format!("{}/{}", owner(), repo_name())}
                        </a>
                    </div>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Deployments"</h1>
                    <p class="mt-1 text-gray-600 dark:text-gray-400">"View and manage deployments for this repository."</p>
                </div>
                <Button variant=ButtonVariant::Primary on:click=move |_| set_show_create.set(true)>
                    "Create Deployment"
                </Button>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=dismiss_error />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center justify-center py-12">
                    <Spinner />
                </div>
            </Show>

            <Show when=move || !loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    {move || if deployments.get().is_empty() {
                        view! {
                            <div class="py-12 text-center text-gray-400 dark:text-gray-500">
                                "No deployments found."
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="overflow-x-auto">
                                <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                                    <thead class="bg-gray-50 dark:bg-gray-750">
                                        <tr>
                                            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Status"</th>
                                            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Environment"</th>
                                            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"SHA"</th>
                                            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Created"</th>
                                            <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Actions"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
                                        <For each=move || deployments.get() key=|d| d.id.clone() let:deploy>
                                            {
                                                let deploy_id = deploy.id.clone();
                                                let status = deploy.status.clone();
                                                let environment = deploy.environment.clone();
                                                let sha_short = deploy.sha[..7.min(deploy.sha.len())].to_string();
                                                let created_at = deploy.created_at.clone();
                                                let did = deploy_id.clone();
                                                view! {
                                                    <tr class="hover:bg-gray-50 dark:hover:bg-gray-750">
                                                        <td class="px-4 py-3">
                                                            <Badge color=status_color(&status) text=status />
                                                        </td>
                                                        <td class="px-4 py-3 text-sm font-medium text-gray-900 dark:text-gray-100">
                                                            {environment}
                                                        </td>
                                                        <td class="px-4 py-3 text-xs font-mono text-gray-600 dark:text-gray-400">
                                                            {sha_short}
                                                        </td>
                                                        <td class="px-4 py-3 text-xs text-gray-400 dark:text-gray-500">
                                                            {created_at}
                                                        </td>
                                                        <td class="px-4 py-3 text-right">
                                                            <button
                                                                class="text-xs text-blue-600 dark:text-blue-400 hover:underline"
                                                                on:click=move |_| {
                                                                    set_selected_deploy_id.set(Some(did.clone()));
                                                                    set_show_status_modal.set(true);
                                                                }
                                                            >
                                                                "Update Status"
                                                            </button>
                                                        </td>
                                                    </tr>
                                                }
                                            }
                                        </For>
                                    </tbody>
                                </table>
                            </div>
                        }.into_any()
                    }}
                </Card>
            </Show>

            <Modal show=show_create.get() title="Create Deployment".to_string() on_close=Callback::new(move |_: ()| set_show_create.set(false))>
                <div class="space-y-4">
                    <Input label="Commit SHA" name="deploy-sha" id="deploy-sha" input_type=InputType::Text placeholder="e.g. abc1234" required=true />
                    <Input label="Environment" name="deploy-environment" id="deploy-environment" input_type=InputType::Select options=vec![("production", "Production"), ("staging", "Staging"), ("development", "Development")] />
                    <Input label="Description" name="deploy-description" id="deploy-description" input_type=InputType::Textarea placeholder="Optional description" />
                    <div class="flex gap-3 pt-2">
                        <Button variant=ButtonVariant::Primary disabled=creating.get() on:click=handle_create>
                            {move || if creating.get() { "Creating..." } else { "Deploy" }}
                        </Button>
                        <button class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors bg-gray-200 hover:bg-gray-300 text-gray-900 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-100" on:click=move |_| set_show_create.set(false)>
                            "Cancel"
                        </button>
                    </div>
                </div>
            </Modal>

            <Modal show=show_status_modal.get() title="Update Deployment Status".to_string() on_close=Callback::new(move |_: ()| { set_show_status_modal.set(false); set_selected_deploy_id.set(None); })>
                <div class="space-y-4">
                    <Input label="New Status" name="deploy-status" id="deploy-status" input_type=InputType::Select options=vec![("pending", "Pending"), ("in_progress", "In Progress"), ("success", "Success"), ("failure", "Failure"), ("cancelled", "Cancelled")] />
                    <Input label="Description" name="deploy-status-desc" id="deploy-status-desc" input_type=InputType::Textarea placeholder="Optional status description" />
                    <div class="flex gap-3 pt-2">
                        <Button variant=ButtonVariant::Primary on:click=move |_| {
                            if let Some(did) = selected_deploy_id.get() {
                                handle_update_status(did);
                            }
                        }>
                            "Update"
                        </Button>
                        <button class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors bg-gray-200 hover:bg-gray-300 text-gray-900 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-100" on:click=move |_| { set_show_status_modal.set(false); set_selected_deploy_id.set(None); }>
                            "Cancel"
                        </button>
                    </div>
                </div>
            </Modal>
        </div>
    }
}
