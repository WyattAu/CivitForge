#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};

use crate::api::client::ApiClient;
use crate::api::types::{AuthUser, SshKeyResponse, UpdateRepoRequest};
use crate::components::{
    Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Input, InputType, Modal, Spinner,
};
use crate::state::auth::use_auth;
use crate::utils::*;
use civit_shared::user::UserResponse;

#[derive(Debug, Clone, serde::Serialize)]
struct UpdateProfileBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bio: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct AddSshKeyBody {
    key_type: String,
    public_key: String,
    fingerprint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ChangePasswordBody {
    current_password: String,
    new_password: String,
}

#[derive(Clone, PartialEq)]
enum SettingsSection {
    General,
    Collaborators,
    Branches,
    Labels,
    Danger,
}

#[component]
pub fn RepoSettingsPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let auth = use_auth();
    let navigate = use_navigate();
    let (navigate_sig, _) = signal(navigate);

    let (active_section, set_active_section) = signal(SettingsSection::General);
    let (repo_name_sig, set_repo_name) = signal(String::new());
    let (repo_desc_sig, set_repo_desc) = signal(String::new());
    let (repo_visibility_sig, set_repo_visibility) = signal(String::from("public"));
    let (repo_branch_sig, set_repo_branch) = signal(String::from("main"));
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (saving, set_saving) = signal(false);
    let (success, set_success) = signal(false);
    let (deleting, set_deleting) = signal(false);

    let fetch_repo = move || {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<civit_shared::repo::RepoResponse>().await {
                        Ok(repo) => {
                            set_repo_name.set(repo.name);
                            set_repo_desc.set(repo.description.unwrap_or_default());
                            set_repo_visibility.set(repo.visibility.to_string());
                            set_repo_branch.set(repo.default_branch);
                        }
                        Err(_) => set_error.set(Some("Failed to process response.".to_string())),
                    }
                }
                Ok(_) => {
                    set_error.set(Some("Failed to load repository.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_loading.set(false);
        });
    };

    fetch_repo();

    let nav = navigate_sig;
    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        set_success.set(false);

        let name_val = get_input_value("repo-settings-name");
        let desc_val = get_input_value("repo-settings-description");
        let vis_public = get_input_value("visibility-public");
        let vis_internal = get_input_value("visibility-internal");
        let branch_val = get_input_value("repo-settings-branch");

        if name_val.trim().is_empty() {
            set_error.set(Some("Repository name is required.".to_string()));
            return;
        }

        let visibility = if !vis_public.is_empty() && vis_public == "on" {
            "public".to_string()
        } else if !vis_internal.is_empty() && vis_internal == "on" {
            "internal".to_string()
        } else {
            "private".to_string()
        };

        let body = UpdateRepoRequest {
            name: Some(name_val.trim().to_string()),
            description: if desc_val.trim().is_empty() {
                None
            } else {
                Some(desc_val.trim().to_string())
            },
            visibility: Some(visibility),
            default_branch: if branch_val.trim().is_empty() {
                None
            } else {
                Some(branch_val.trim().to_string())
            },
        };

        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();

        set_saving.set(true);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}");
            match client.patch(&path, &body).await {
                Ok(resp) if resp.status().is_success() => {
                    set_success.set(true);
                }
                Ok(_) => {
                    set_error.set(Some("Failed to update repository.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_saving.set(false);
        });
    };

    let delete_repo = move |_: leptos::ev::MouseEvent| {
        set_error.set(None);
        set_deleting.set(true);

        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        let nav_cl = nav.get();

        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}");
            match client.delete(&path).await {
                Ok(resp)
                    if resp.status().is_success()
                        || resp.status() == reqwest::StatusCode::NO_CONTENT =>
                {
                    nav_cl("/repos", Default::default());
                }
                Ok(_) => {
                    set_error.set(Some("Failed to delete repository.".to_string()));
                    set_deleting.set(false);
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                    set_deleting.set(false);
                }
            }
        });
    };

    let sidebar_item_class =
        "block w-full text-left px-4 py-2 text-sm font-medium border-2 transition-colors";
    let sidebar_item_active = "bg-gray-100 dark:bg-gray-700 border-gray-300 dark:border-gray-600 text-gray-900 dark:text-gray-100";
    let sidebar_item_inactive = "bg-white dark:bg-gray-800 border-transparent text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-750 hover:border-gray-200 dark:hover:border-gray-600 hover:text-gray-900 dark:hover:text-gray-100";

    let owner_disp = move || owner();
    let name_disp = move || name();
    let current_vis = move || repo_visibility_sig.get();

    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-3xl font-bold font-mono text-gray-900 dark:text-gray-100">"SETTINGS"</h1>
                <p class="mt-1 text-sm text-gray-500 dark:text-gray-400 font-mono">
                    {move || format!("{}/{}", owner_disp(), name_disp())}
                </p>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_error.set(None)) />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center justify-center py-12">
                    <Spinner />
                    <span class="ml-3 text-gray-500 dark:text-gray-400">"Loading repository..."</span>
                </div>
            </Show>

            <Show when=move || !loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex gap-6">
                    // -- Sidebar --
                    <nav class="w-56 shrink-0">
                        <div class="bg-white dark:bg-gray-800 rounded-none border-2 border-gray-200 dark:border-gray-700 p-2 space-y-1">
                            <button
                                class=format!("{sidebar_item_class} {}", if active_section.get() == SettingsSection::General { sidebar_item_active } else { sidebar_item_inactive })
                                on:click=move |_| set_active_section.set(SettingsSection::General)
                            >
                                "General"
                            </button>
                            <button
                                class=format!("{sidebar_item_class} {}", if active_section.get() == SettingsSection::Collaborators { sidebar_item_active } else { sidebar_item_inactive })
                                on:click=move |_| set_active_section.set(SettingsSection::Collaborators)
                            >
                                "Collaborators"
                            </button>
                            <button
                                class=format!("{sidebar_item_class} {}", if active_section.get() == SettingsSection::Branches { sidebar_item_active } else { sidebar_item_inactive })
                                on:click=move |_| set_active_section.set(SettingsSection::Branches)
                            >
                                "Branches"
                            </button>
                            <button
                                class=format!("{sidebar_item_class} {}", if active_section.get() == SettingsSection::Labels { sidebar_item_active } else { sidebar_item_inactive })
                                on:click=move |_| set_active_section.set(SettingsSection::Labels)
                            >
                                "Labels"
                            </button>

                            <div class="my-2 border-t border-gray-200 dark:border-gray-700"></div>

                            <button
                                class=format!("{sidebar_item_class} text-red-600 dark:text-red-400 {}", if active_section.get() == SettingsSection::Danger { sidebar_item_active } else { sidebar_item_inactive })
                                on:click=move |_| set_active_section.set(SettingsSection::Danger)
                            >
                                "Danger Zone"
                            </button>
                        </div>
                    </nav>

                    // -- Content --
                    <div class="flex-1 min-w-0 space-y-6">
                        <Show when=move || active_section.get() == SettingsSection::General fallback=|| view! { <div class="hidden"></div> }>
                            <Card title="General".to_string() description="Basic repository settings".to_string()>
                                <form on:submit=handle_submit class="space-y-5">
                                    <Show when=move || success.get() fallback=|| view! { <div class="hidden"></div> }>
                                        <div class="p-3 bg-green-50 dark:bg-green-900/20 border-l-4 border-green-500 dark:border-green-400 text-sm text-green-700 dark:text-green-400">
                                            "Settings updated successfully."
                                        </div>
                                    </Show>
                                    <Input
                                        label="Repository Name"
                                        name="repo-settings-name"
                                        id="repo-settings-name"
                                        input_type=InputType::Text
                                        value=repo_name_sig.get()
                                        required=true
                                    />
                                    <Input
                                        label="Description"
                                        name="repo-settings-description"
                                        id="repo-settings-description"
                                        input_type=InputType::Textarea
                                        placeholder="A brief description..."
                                        value=repo_desc_sig.get()
                                    />
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
                                                    checked=move || current_vis() == "public"
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
                                                    checked=move || current_vis() == "internal"
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
                                                    checked=move || current_vis() == "private"
                                                    class="w-4 h-4 text-blue-600 border-gray-300 focus:ring-blue-500 dark:border-gray-600"
                                                />
                                                <div>
                                                    <span class="text-sm font-medium text-gray-900 dark:text-gray-100">"Private"</span>
                                                    <p class="text-xs text-gray-500 dark:text-gray-400">"Only authorized users can see this repository."</p>
                                                </div>
                                            </label>
                                        </div>
                                    </div>
                                    <Input
                                        label="Default Branch"
                                        name="repo-settings-branch"
                                        id="repo-settings-branch"
                                        input_type=InputType::Text
                                        value=repo_branch_sig.get()
                                    />
                                    <div>
                                        <Button variant=ButtonVariant::Primary extra_class="btn-save-settings" disabled=saving.get()>
                                            {move || if saving.get() { "Saving..." } else { "Save Changes" }}
                                        </Button>
                                    </div>
                                </form>
                            </Card>
                        </Show>

                        <Show when=move || active_section.get() == SettingsSection::Collaborators fallback=|| view! { <div class="hidden"></div> }>
                            <Card title="Collaborators".to_string() description="Manage repository collaborators".to_string()>
                                <div class="py-8 text-center text-gray-400 dark:text-gray-500">
                                    <p class="text-sm">"Coming soon"</p>
                                </div>
                            </Card>
                        </Show>

                        <Show when=move || active_section.get() == SettingsSection::Branches fallback=|| view! { <div class="hidden"></div> }>
                            <Card title="Branches".to_string() description="Manage repository branches and defaults".to_string()>
                                <div class="py-8 text-center text-gray-400 dark:text-gray-500">
                                    <p class="text-sm">"Coming soon"</p>
                                </div>
                            </Card>
                        </Show>

                        <Show when=move || active_section.get() == SettingsSection::Labels fallback=|| view! { <div class="hidden"></div> }>
                            <Card title="Labels".to_string() description="Manage issue and pull request labels".to_string()>
                                <div class="py-8 text-center text-gray-400 dark:text-gray-500">
                                    <p class="text-sm">"Coming soon"</p>
                                </div>
                            </Card>
                        </Show>

                        <Show when=move || active_section.get() == SettingsSection::Danger fallback=|| view! { <div class="hidden"></div> }>
                            <Card title="Danger Zone".to_string() description="Irreversible and destructive actions".to_string()>
                                <div class="space-y-6">
                                    <div class="border-2 border-red-200 dark:border-red-800 p-4">
                                        <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100">"Change Visibility"</h3>
                                        <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
                                            "Changing repository visibility will immediately affect who can access this repository."
                                        </p>
                                        <p class="mt-1">
                                            <Button variant=ButtonVariant::Secondary>"Change Visibility"</Button>
                                        </p>
                                    </div>

                                    <div class="border-2 border-red-200 dark:border-red-800 p-4">
                                        <h3 class="text-sm font-semibold text-red-600 dark:text-red-400">"Delete this repository"</h3>
                                        <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
                                            "Once you delete a repository, there is no going back. This action permanently deletes the repository, wiki, issues, comments, packages, secrets, workflow runs, and all other associated data."
                                        </p>
                                        <div class="mt-3">
                                            <Button variant=ButtonVariant::Danger extra_class="btn-delete-repo" disabled=deleting.get() on:click=delete_repo>
                                                {move || if deleting.get() { "Deleting..." } else { "Delete this repository" }}
                                            </Button>
                                        </div>
                                    </div>
                                </div>
                            </Card>
                        </Show>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
pub fn SettingsPage() -> impl IntoView {
    let auth = use_auth();
    let (user_sig, set_user) = signal(None::<UserResponse>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let (profile_saving, set_profile_saving) = signal(false);
    let (profile_error, set_profile_error) = signal(None::<String>);
    let (profile_success, set_profile_success) = signal(false);

    let (ssh_keys, set_ssh_keys) = signal(Vec::<SshKeyResponse>::new());
    let (ssh_loading, set_ssh_loading) = signal(true);

    let (show_add_ssh, set_show_add_ssh) = signal(false);
    let (ssh_add_loading, set_ssh_add_loading) = signal(false);
    let (ssh_add_error, set_ssh_add_error) = signal(None::<String>);

    let (confirm_delete_id, set_confirm_delete_id) = signal(None::<String>);
    let (ssh_delete_loading, set_ssh_delete_loading) = signal(false);

    let (pw_error, set_pw_error) = signal(None::<String>);
    let (pw_success, set_pw_success) = signal(false);
    let (pw_loading, set_pw_loading) = signal(false);

    let fetch_user = move || {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        leptos::task::spawn_local(async move {
            match client.get("/auth/me").await {
                Ok(resp) if resp.status().is_success() => match resp.json::<AuthUser>().await {
                    Ok(auth_user) => {
                        let client2 = ApiClient::new(auth.0.with(|a| a.token.clone()));
                        match client2.get(&format!("/users/{}", auth_user.id)).await {
                            Ok(r) if r.status().is_success() => {
                                if let Ok(u) = r.json::<UserResponse>().await {
                                    set_user.set(Some(u));
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(_) => set_error.set(Some("Failed to process response.".to_string())),
                },
                Ok(_) => {
                    set_error.set(Some("Failed to load user.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_loading.set(false);
        });
    };

    let fetch_ssh_keys = move || {
        set_ssh_loading.set(true);
        let token = auth.0.with(|a| a.token.clone());
        let user_id = auth.0.with(|a| a.user_id.clone());
        if let Some(uid) = user_id {
            let client = ApiClient::new(token);
            leptos::task::spawn_local(async move {
                match client.get(&format!("/users/{uid}/ssh-keys")).await {
                    Ok(resp) if resp.status().is_success() => {
                        if let Ok(keys) = resp.json::<Vec<SshKeyResponse>>().await {
                            set_ssh_keys.set(keys);
                        }
                    }
                    _ => {}
                }
                set_ssh_loading.set(false);
            });
        } else {
            set_ssh_loading.set(false);
        }
    };

    fetch_user();
    fetch_ssh_keys();

    let handle_profile_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_profile_error.set(None);
        set_profile_success.set(false);

        let display_name_val = get_input_value("settings-display-name");
        let bio_val = get_input_value("settings-bio");

        let body = UpdateProfileBody {
            display_name: if display_name_val.trim().is_empty() {
                None
            } else {
                Some(display_name_val.trim().to_string())
            },
            bio: if bio_val.trim().is_empty() {
                None
            } else {
                Some(bio_val.trim().to_string())
            },
        };

        let user_id = match user_sig.get() {
            Some(u) => u.id.to_string(),
            None => {
                set_profile_error.set(Some("User not loaded.".to_string()));
                return;
            }
        };

        set_profile_saving.set(true);
        leptos::task::spawn_local(async move {
            let token = auth.0.with(|a| a.token.clone());
            let client = ApiClient::new(token);
            match client.put(&format!("/users/{user_id}"), &body).await {
                Ok(resp) if resp.status().is_success() => {
                    set_profile_success.set(true);
                }
                Ok(_) => {
                    set_profile_error.set(Some("Failed to update profile.".to_string()));
                }
                Err(_) => {
                    set_profile_error
                        .set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_profile_saving.set(false);
        });
    };

    let open_add_ssh = move |_: leptos::ev::MouseEvent| set_show_add_ssh.set(true);
    let close_add_ssh = Callback::new(move |_: ()| {
        set_show_add_ssh.set(false);
        set_ssh_add_error.set(None);
    });

    let handle_ssh_add_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_ssh_add_error.set(None);

        let label_val = get_input_value("ssh-key-label");
        let public_key_val = get_input_value("ssh-key-public");

        if public_key_val.trim().is_empty() {
            set_ssh_add_error.set(Some("Public key is required.".to_string()));
            return;
        }

        let parts: Vec<&str> = public_key_val.split_whitespace().collect();
        let key_type = parts
            .first()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "ssh-ed25519".to_string());
        let fingerprint = format!(
            "SHA256:{}",
            &parts
                .last()
                .unwrap_or(&"")
                .chars()
                .take(43)
                .collect::<String>()
        );

        let user_id = match auth.0.with(|a| a.user_id.clone()) {
            Some(uid) => uid,
            None => {
                set_ssh_add_error.set(Some("User not authenticated.".to_string()));
                return;
            }
        };

        let body = AddSshKeyBody {
            key_type,
            public_key: public_key_val.trim().to_string(),
            fingerprint,
            label: if label_val.trim().is_empty() {
                None
            } else {
                Some(label_val.trim().to_string())
            },
        };

        set_ssh_add_loading.set(true);
        leptos::task::spawn_local(async move {
            let token = auth.0.with(|a| a.token.clone());
            let client = ApiClient::new(token);
            match client
                .post(&format!("/users/{user_id}/ssh-keys"), &body)
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    set_show_add_ssh.set(false);
                    let token2 = auth.0.with(|a| a.token.clone());
                    let client2 = ApiClient::new(token2);
                    match client2.get(&format!("/users/{user_id}/ssh-keys")).await {
                        Ok(r) if r.status().is_success() => {
                            if let Ok(keys) = r.json::<Vec<SshKeyResponse>>().await {
                                set_ssh_keys.set(keys);
                            }
                        }
                        _ => {}
                    }
                }
                Ok(_) => {
                    set_ssh_add_error.set(Some("Failed to add SSH key.".to_string()));
                }
                Err(_) => {
                    set_ssh_add_error
                        .set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_ssh_add_loading.set(false);
        });
    };

    let request_delete_ssh = move |key_id: String| {
        set_confirm_delete_id.set(Some(key_id));
    };

    let cancel_delete_ssh = Callback::new(move |_: ()| set_confirm_delete_id.set(None));

    let handle_password_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_pw_error.set(None);
        set_pw_success.set(false);

        let current = get_input_value("pw-current");
        let new_pw = get_input_value("pw-new");
        let confirm = get_input_value("pw-confirm");

        if current.is_empty() {
            set_pw_error.set(Some("Current password is required.".to_string()));
            return;
        }
        if new_pw.len() < 8 {
            set_pw_error.set(Some(
                "New password must be at least 8 characters.".to_string(),
            ));
            return;
        }
        if new_pw != confirm {
            set_pw_error.set(Some("New passwords do not match.".to_string()));
            return;
        }

        let user_id = match auth.0.with(|a| a.user_id.clone()) {
            Some(uid) => uid,
            None => {
                set_pw_error.set(Some("User not authenticated.".to_string()));
                return;
            }
        };

        let body = ChangePasswordBody {
            current_password: current,
            new_password: new_pw,
        };

        set_pw_loading.set(true);
        leptos::task::spawn_local(async move {
            let token = auth.0.with(|a| a.token.clone());
            let client = ApiClient::new(token);
            match client
                .post(&format!("/users/{user_id}/password"), &body)
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    set_pw_success.set(true);
                }
                Ok(_) => {
                    set_pw_error.set(Some("Failed to change password.".to_string()));
                }
                Err(_) => {
                    set_pw_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_pw_loading.set(false);
        });
    };

    let username = move || auth.0.with(|a| a.username.clone().unwrap_or_default());

    view! {
        <div class="space-y-6">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"User Settings"</h1>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_error.set(None)) />
            </Show>

            // -- Profile --
            <Card title="Profile".to_string() description="Manage your public profile information".to_string()>
                <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                    <div class="flex items-center justify-center py-6">
                        <Spinner />
                    </div>
                </Show>
                <Show when=move || !loading.get() && user_sig.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                    <form on:submit=handle_profile_submit class="space-y-5">
                        <Show when=move || profile_success.get() fallback=|| view! { <div class="hidden"></div> }>
                            <div class="p-3 bg-green-50 dark:bg-green-900/20 border-l-4 border-green-500 dark:border-green-400 rounded-r-sm text-sm text-green-700 dark:text-green-400">
                                "Profile updated successfully."
                            </div>
                        </Show>
                        <Show when=move || profile_error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                            <ErrorBanner message=move || profile_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_profile_error.set(None)) />
                        </Show>
                        {move || user_sig.get().map(|u| {
                            view! {
                                <div class="grid grid-cols-1 sm:grid-cols-2 gap-5">
                                    <Input label="Username" name="username" input_type=InputType::Text value=username() required=true disabled=true></Input>
                                    <Input label="Display Name" name="settings-display-name" id="settings-display-name" input_type=InputType::Text value=u.display_name.clone().unwrap_or_default()></Input>
                                </div>
                                <Input label="Email" name="email" input_type=InputType::Email value=u.email.clone() required=true disabled=true></Input>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Role"</label>
                                    <div class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 rounded-md text-sm text-gray-500 dark:text-gray-400 bg-gray-50 dark:bg-gray-800">
                                        <Badge color=BadgeColor::Info text=format!("{:?}", u.role).to_lowercase() />
                                    </div>
                                </div>
                                <Input label="Bio" name="settings-bio" id="settings-bio" input_type=InputType::Textarea placeholder="Tell us about yourself..." value=u.bio.clone().unwrap_or_default()></Input>
                                <div>
                                    <Button variant=ButtonVariant::Primary extra_class="btn-save-profile" disabled=profile_saving.get()>
                                        {move || if profile_saving.get() { "Saving..." } else { "Update Profile" }}
                                    </Button>
                                </div>
                            }
                        })}
                    </form>
                </Show>
            </Card>

            // -- SSH Keys --
            <Card title="SSH Keys".to_string() description="Manage your SSH keys for repository access".to_string()>
                <Show when=move || ssh_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                    <div class="flex items-center justify-center py-4">
                        <Spinner />
                    </div>
                </Show>
                <Show when=move || !ssh_loading.get() && ssh_keys.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                    <div class="py-4 text-center text-gray-400 dark:text-gray-500">
                        "No SSH keys configured."
                    </div>
                </Show>
                <Show when=move || !ssh_loading.get() && !ssh_keys.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                    <div class="divide-y divide-gray-100 dark:divide-gray-700">
                        <For each=move || ssh_keys.get() key=|k| k.id.clone() let:key>
                            {
                                let key = key.clone();
                                view! {
                                    <div class="flex items-center justify-between py-3">
                                        <div class="min-w-0">
                                            <div class="flex items-center gap-2">
                                                <span class="text-sm font-medium text-gray-900 dark:text-gray-100">
                                                    {if key.label.is_empty() { "Unnamed key".to_string() } else { key.label.clone() }}
                                                </span>
                                                <Badge color=BadgeColor::Neutral text=key.key_type.clone() />
                                            </div>
                                            <div class="text-xs text-gray-500 dark:text-gray-400 font-mono mt-1 truncate">
                                                {key.fingerprint.clone()}
                                            </div>
                                        </div>
                                        <div class="flex items-center gap-2 shrink-0 ml-4">
                                            <span class="text-xs text-gray-400 dark:text-gray-500">
                                                {key.created_at.clone()}
                                            </span>
                                            <button
                                                class="inline-flex items-center justify-center px-2 py-1 rounded-md text-xs font-medium transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900 bg-red-600 hover:bg-red-700 text-white dark:bg-red-500 dark:hover:bg-red-600 disabled:opacity-50 disabled:cursor-not-allowed"
                                                disabled=ssh_delete_loading.get()
                                                on:click=move |_| request_delete_ssh(key.id.clone())
                                            >
                                                "Delete"
                                            </button>
                                        </div>
                                    </div>
                                }
                            }
                        </For>
                    </div>
                </Show>
                <div class="mt-4">
                    <button
                        class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900 bg-gray-200 hover:bg-gray-300 text-gray-900 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-100 btn-add-ssh"
                        on:click=open_add_ssh
                    >
                        "Add SSH Key"
                    </button>
                </div>
            </Card>

            // -- Password --
            <Card title="Change Password".to_string() description="Update your account password".to_string()>
                <form on:submit=handle_password_submit class="space-y-5">
                    <Show when=move || pw_success.get() fallback=|| view! { <div class="hidden"></div> }>
                        <div class="p-3 bg-green-50 dark:bg-green-900/20 border-l-4 border-green-500 dark:border-green-400 rounded-r-sm text-sm text-green-700 dark:text-green-400">
                            "Password changed successfully."
                        </div>
                    </Show>
                    <Show when=move || pw_error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                        <ErrorBanner message=move || pw_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_pw_error.set(None)) />
                    </Show>
                    <Input label="Current Password" name="pw-current" id="pw-current" input_type=InputType::Password placeholder="Enter current password" required=true></Input>
                    <Input label="New Password" name="pw-new" id="pw-new" input_type=InputType::Password placeholder="Enter new password (min 8 chars)" required=true></Input>
                    <Input label="Confirm New Password" name="pw-confirm" id="pw-confirm" input_type=InputType::Password placeholder="Confirm new password" required=true></Input>
                    <div>
                        <Button variant=ButtonVariant::Primary extra_class="btn-change-password" disabled=pw_loading.get()>
                            {move || if pw_loading.get() { "Changing..." } else { "Change Password" }}
                        </Button>
                    </div>
                </form>
            </Card>

            // -- Danger Zone --
            <Card title="Danger Zone".to_string() description="Irreversible and destructive actions".to_string()>
                <div class="border border-red-200 dark:border-red-800 rounded-md p-4">
                    <h3 class="text-sm font-medium text-red-600 dark:text-red-400">"Delete Account"</h3>
                    <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
                        "Once you delete your account, there is no going back. Please be certain."
                    </p>
                    <div class="mt-3">
                        <Button variant=ButtonVariant::Danger disabled=true>
                            "Delete Account"
                        </Button>
                    </div>
                </div>
            </Card>

            // -- Add SSH Key Modal --
            <Modal
                show=show_add_ssh.get()
                title="Add SSH Key".to_string()
                on_close=close_add_ssh
            >
                <form on:submit=handle_ssh_add_submit class="space-y-4">
                    <Show when=move || ssh_add_error.get().is_some()>
                        <ErrorBanner message=move || ssh_add_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_ssh_add_error.set(None)) />
                    </Show>
                    <Input
                        label="Label (optional)"
                        name="ssh-key-label"
                        id="ssh-key-label"
                        input_type=InputType::Text
                        placeholder="e.g. my-laptop"
                    />
                    <Input
                        label="Public Key"
                        name="ssh-key-public"
                        id="ssh-key-public"
                        input_type=InputType::Textarea
                        placeholder="ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI..."
                        required=true
                    />
                    <div class="flex gap-3 pt-2">
                        <Button variant=ButtonVariant::Primary disabled=ssh_add_loading.get()>
                            {move || if ssh_add_loading.get() { "Adding..." } else { "Add Key" }}
                        </Button>
                        <button
                            class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900 bg-gray-200 hover:bg-gray-300 text-gray-900 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-100"
                            on:click=move |_| {
                                set_show_add_ssh.set(false);
                                set_ssh_add_error.set(None);
                            }
                        >
                            "Cancel"
                        </button>
                    </div>
                </form>
            </Modal>

            // -- Delete SSH Key Confirmation Modal --
            <Modal
                show=confirm_delete_id.get().is_some()
                title="Delete SSH Key".to_string()
                on_close=cancel_delete_ssh
            >
                <div class="space-y-4">
                    <p class="text-sm text-gray-600 dark:text-gray-400">
                        "Are you sure you want to delete this SSH key? This action cannot be undone."
                    </p>
                    <div class="flex gap-3 pt-2">
                        <button
                            class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900 bg-red-600 hover:bg-red-700 text-white dark:bg-red-500 dark:hover:bg-red-600 disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled=ssh_delete_loading.get()
                            on:click=move |_| {
                                let key_id = match confirm_delete_id.get() {
                                    Some(id) => id,
                                    None => return,
                                };
                                set_confirm_delete_id.set(None);
                                set_ssh_delete_loading.set(true);
                                let auth_cl = auth;
                                let set_ssh_keys_cl = set_ssh_keys;
                                let set_ssh_delete_loading_cl = set_ssh_delete_loading;
                                leptos::task::spawn_local(async move {
                                    let token = auth_cl.0.with(|a| a.token.clone());
                                    let client = ApiClient::new(token);
                                    match client.delete(&format!("/ssh-keys/{key_id}")).await {
                                        Ok(resp) if resp.status().is_success() || resp.status() == reqwest::StatusCode::NO_CONTENT => {
                                            let token2 = auth_cl.0.with(|a| a.token.clone());
                                            let uid = auth_cl.0.with(|a| a.user_id.clone());
                                            if let Some(user_id) = uid {
                                                let client2 = ApiClient::new(token2);
                                                match client2.get(&format!("/users/{user_id}/ssh-keys")).await {
                                                    Ok(r) if r.status().is_success() => {
                                                        if let Ok(keys) = r.json::<Vec<SshKeyResponse>>().await {
                                                            set_ssh_keys_cl.set(keys);
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                    set_ssh_delete_loading_cl.set(false);
                                });
                            }
                        >
                            {move || if ssh_delete_loading.get() { "Deleting..." } else { "Delete" }}
                        </button>
                        <button
                            class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900 bg-gray-200 hover:bg-gray-300 text-gray-900 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-100"
                            on:click=move |_| set_confirm_delete_id.set(None)
                        >
                            "Cancel"
                        </button>
                    </div>
                </div>
            </Modal>
        </div>
    }
}
