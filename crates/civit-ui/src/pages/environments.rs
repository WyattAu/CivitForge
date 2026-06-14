#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::components::{
    Button, ButtonVariant, Card, ErrorBanner, Input, InputType, Modal, Spinner,
};
use crate::state::auth::use_auth;
use crate::utils::get_input_value;

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
struct Environment {
    id: String,
    name: String,
    repo_id: String,
    protection_rules: Option<String>,
    variables: Option<Vec<EnvVariable>>,
    created_at: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct EnvVariable {
    name: String,
    value: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct CreateEnvironmentBody {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    protection_rules: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct UpdateEnvironmentBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    protection_rules: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct UpsertVariableBody {
    name: String,
    value: String,
}

#[component]
pub fn EnvironmentsPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let repo_name = move || params.with(|p| p.get("name").unwrap_or_default());
    let auth = use_auth();

    let (environments, set_environments) = signal(Vec::<Environment>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let (show_create, set_show_create) = signal(false);
    let (show_edit, set_show_edit) = signal(false);
    let (editing_env, set_editing_env) = signal(None::<Environment>);
    let (creating, set_creating) = signal(false);

    let (show_var_modal, set_show_var_modal) = signal(false);
    let (selected_env_id, set_selected_env_id) = signal(None::<String>);

    let fetch_environments = move || {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        let o = owner();
        let r = repo_name();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{o}/{r}/environments");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<Vec<Environment>>().await {
                        set_environments.set(data);
                    }
                }
                Ok(_) => {
                    set_error.set(Some("Failed to load environments.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error.".to_string()));
                }
            }
            set_loading.set(false);
        });
    };

    fetch_environments();

    let handle_create = move |_: leptos::ev::MouseEvent| {
        let name = get_input_value("env-name");
        let rules = get_input_value("env-protection-rules");

        if name.trim().is_empty() {
            set_error.set(Some("Environment name is required.".to_string()));
            return;
        }

        let body = CreateEnvironmentBody {
            name: name.trim().to_string(),
            protection_rules: if rules.trim().is_empty() {
                None
            } else {
                Some(rules.trim().to_string())
            },
        };

        let token = auth.0.with(|a| a.token.clone());
        let o = owner();
        let r = repo_name();
        set_creating.set(true);
        set_error.set(None);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{o}/{r}/environments");
            match client.post(&path, &body).await {
                Ok(resp) if resp.status().is_success() => {
                    set_show_create.set(false);
                    fetch_environments();
                }
                Ok(_) => {
                    set_error.set(Some("Failed to create environment.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error.".to_string()));
                }
            }
            set_creating.set(false);
        });
    };

    let handle_edit = move |env_id: String| {
        let name = get_input_value("edit-env-name");
        let rules = get_input_value("edit-env-protection-rules");

        let body = UpdateEnvironmentBody {
            name: if name.trim().is_empty() {
                None
            } else {
                Some(name.trim().to_string())
            },
            protection_rules: if rules.trim().is_empty() {
                None
            } else {
                Some(rules.trim().to_string())
            },
        };

        let token = auth.0.with(|a| a.token.clone());
        let o = owner();
        let r = repo_name();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{o}/{r}/environments/{env_id}");
            let _ = client.patch(&path, &body).await;
            set_show_edit.set(false);
            set_editing_env.set(None);
            fetch_environments();
        });
    };

    let delete_environment = move |env_id: String| {
        let token = auth.0.with(|a| a.token.clone());
        let o = owner();
        let r = repo_name();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{o}/{r}/environments/{env_id}");
            let _ = client.delete(&path).await;
            fetch_environments();
        });
    };

    let handle_add_var = move |env_id: String| {
        let var_name = get_input_value("var-name");
        let var_value = get_input_value("var-value");

        if var_name.trim().is_empty() || var_value.trim().is_empty() {
            set_error.set(Some("Variable name and value are required.".to_string()));
            return;
        }

        let body = UpsertVariableBody {
            name: var_name.trim().to_string(),
            value: var_value.trim().to_string(),
        };

        let token = auth.0.with(|a| a.token.clone());
        let o = owner();
        let r = repo_name();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{o}/{r}/environments/{env_id}/variables");
            match client.post(&path, &body).await {
                Ok(resp) if resp.status().is_success() => {
                    set_show_var_modal.set(false);
                    set_selected_env_id.set(None);
                    fetch_environments();
                }
                Ok(_) => {
                    set_error.set(Some("Failed to add variable.".to_string()));
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
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Environments"</h1>
                    <p class="mt-1 text-gray-600 dark:text-gray-400">"Manage deployment environments and variables."</p>
                </div>
                <Button variant=ButtonVariant::Primary on:click=move |_| set_show_create.set(true)>
                    "Create Environment"
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
                <Show
                    when=move || !environments.get().is_empty()
                    fallback=|| view! {
                        <Card>
                            <div class="py-12 text-center text-gray-400 dark:text-gray-500">
                                "No environments configured. Create one to get started."
                            </div>
                        </Card>
                    }
                >
                    <div class="grid gap-4 md:grid-cols-2">
                        <For each=move || environments.get() key=|e| e.id.clone() let:env>
                            {
                                let env_for_edit = StoredValue::new(env.clone());
                                let env_for_delete = StoredValue::new(env.clone());
                                let env_for_vars_btn = StoredValue::new(env.clone());
                                let env_for_prot = StoredValue::new(env.clone());
                                let env_for_vars = StoredValue::new(env.clone());
                                view! {
                                    <Card>
                                        <div class="flex items-start justify-between">
                                            <div>
                                                <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{env.name.clone()}</h3>
                                                <p class="text-xs text-gray-400 dark:text-gray-500 mt-1">"Created " {env.created_at.clone()}</p>
                                            </div>
                                            <div class="flex gap-2">
                                                <button
                                                    class="text-xs text-blue-600 dark:text-blue-400 hover:underline"
                                                    on:click=move |_| {
                                                        set_editing_env.set(Some(env_for_edit.get_value()));
                                                        set_show_edit.set(true);
                                                    }
                                                >
                                                    "Edit"
                                                </button>
                                                <button
                                                    class="text-xs text-red-600 dark:text-red-400 hover:underline"
                                                    on:click=move |_| delete_environment(env_for_delete.get_value().id)
                                                >
                                                    "Delete"
                                                </button>
                                            </div>
                                        </div>

                                        {move || {
                                            let prot = env_for_prot.get_value().protection_rules.clone().unwrap_or_default();
                                            if prot.is_empty() {
                                                view! { <div class="hidden"></div> }.into_any()
                                            } else {
                                                view! {
                                                    <div class="mt-3">
                                                        <span class="text-xs font-medium text-gray-500 dark:text-gray-400">"Protection: "</span>
                                                        <span class="text-xs text-gray-700 dark:text-gray-300">{prot}</span>
                                                    </div>
                                                }.into_any()
                                            }
                                        }}

                                        <div class="mt-4">
                                            <div class="flex items-center justify-between mb-2">
                                                <span class="text-sm font-medium text-gray-700 dark:text-gray-300">"Variables"</span>
                                                <button
                                                    class="text-xs text-blue-600 dark:text-blue-400 hover:underline"
                                                    on:click=move |_| {
                                                        set_selected_env_id.set(Some(env_for_vars_btn.get_value().id));
                                                        set_show_var_modal.set(true);
                                                    }
                                                >
                                                    "+ Add Variable"
                                                </button>
                                            </div>
                                            {move || {
                                                let vars = env_for_vars.get_value().variables.clone().unwrap_or_default();
                                                if vars.is_empty() {
                                                    view! {
                                                        <p class="text-xs text-gray-400 dark:text-gray-500">"No variables set."</p>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <div class="space-y-1">
                                                            {vars.into_iter().map(|v| {
                                                                let vname = v.name.clone();
                                                                let vval = v.value.clone();
                                                                view! {
                                                                    <div class="flex items-center justify-between bg-gray-50 dark:bg-gray-750 rounded px-2 py-1">
                                                                        <span class="text-xs font-mono text-gray-700 dark:text-gray-300">{vname}</span>
                                                                        <span class="text-xs font-mono text-gray-400 dark:text-gray-500 truncate max-w-[150px]">{vval}</span>
                                                                    </div>
                                                                }
                                                            }).collect::<Vec<_>>()}
                                                        </div>
                                                    }.into_any()
                                                }
                                            }}
                                        </div>
                                    </Card>
                                }
                            }
                        </For>
                    </div>
                </Show>
            </Show>

            <Modal show=show_create.get() title="Create Environment".to_string() on_close=Callback::new(move |_: ()| set_show_create.set(false))>
                <div class="space-y-4">
                    <Input label="Environment Name" name="env-name" id="env-name" input_type=InputType::Text placeholder="e.g. production" required=true />
                    <Input label="Protection Rules" name="env-protection-rules" id="env-protection-rules" input_type=InputType::Textarea placeholder="e.g. review requirements JSON" />
                    <div class="flex gap-3 pt-2">
                        <Button variant=ButtonVariant::Primary disabled=creating.get() on:click=handle_create>
                            {move || if creating.get() { "Creating..." } else { "Create" }}
                        </Button>
                        <button class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors bg-gray-200 hover:bg-gray-300 text-gray-900 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-100" on:click=move |_| set_show_create.set(false)>
                            "Cancel"
                        </button>
                    </div>
                </div>
            </Modal>

            <Modal show=show_edit.get() title="Edit Environment".to_string() on_close=Callback::new(move |_: ()| { set_show_edit.set(false); set_editing_env.set(None); })>
                {move || editing_env.get().map(|env| {
                    let eid = env.id.clone();
                    view! {
                        <div class="space-y-4">
                            <Input label="Environment Name" name="edit-env-name" id="edit-env-name" input_type=InputType::Text value=env.name.clone() />
                            <Input label="Protection Rules" name="edit-env-protection-rules" id="edit-env-protection-rules" input_type=InputType::Textarea value=env.protection_rules.clone().unwrap_or_default() />
                            <div class="flex gap-3 pt-2">
                                <Button variant=ButtonVariant::Primary on:click=move |_| handle_edit(eid.clone())>
                                    "Save"
                                </Button>
                                <button class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors bg-gray-200 hover:bg-gray-300 text-gray-900 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-100" on:click=move |_| { set_show_edit.set(false); set_editing_env.set(None); }>
                                    "Cancel"
                                </button>
                            </div>
                        </div>
                    }
                })}
            </Modal>

            <Modal show=show_var_modal.get() title="Add Environment Variable".to_string() on_close=Callback::new(move |_: ()| { set_show_var_modal.set(false); set_selected_env_id.set(None); })>
                <div class="space-y-4">
                    <Input label="Variable Name" name="var-name" id="var-name" input_type=InputType::Text placeholder="e.g. API_KEY" required=true />
                    <Input label="Value" name="var-value" id="var-value" input_type=InputType::Text placeholder="e.g. secret-value" required=true />
                    <div class="flex gap-3 pt-2">
                        <Button variant=ButtonVariant::Primary on:click=move |_| {
                            if let Some(eid) = selected_env_id.get() {
                                handle_add_var(eid);
                            }
                        }>
                            "Add Variable"
                        </Button>
                        <button class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors bg-gray-200 hover:bg-gray-300 text-gray-900 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-100" on:click=move |_| { set_show_var_modal.set(false); set_selected_env_id.set(None); }>
                            "Cancel"
                        </button>
                    </div>
                </div>
            </Modal>
        </div>
    }
}
