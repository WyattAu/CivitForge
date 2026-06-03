#![forbid(unsafe_code)]

use leptos::prelude::*;

use crate::api::client::ApiClient;
use crate::api::types::{AuthUser, SshKeyResponse};
use crate::components::{
    Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Input, InputType, Spinner,
};
use crate::state::auth::use_auth;
use civit_shared::user::UserResponse;

#[component]
pub fn RepoSettingsPage() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Repository Settings"</h1>

            <Card title="General" description="Basic repository settings">
                <form class="space-y-5">
                    <Input label="Repository name" name="name" input_type=InputType::Text placeholder="my-repo" required=true></Input>
                    <Input label="Description" name="description" input_type=InputType::Textarea placeholder="A brief description..."></Input>
                    <div>
                        <Button variant=ButtonVariant::Primary extra_class="btn-save-settings">
                            "Save Changes"
                        </Button>
                    </div>
                </form>
            </Card>

            <Card title="Danger Zone" description="Irreversible and destructive actions">
                <div class="border border-red-200 dark:border-red-800 rounded-md p-4">
                    <h3 class="text-sm font-medium text-red-600 dark:text-red-400">"Delete this repository"</h3>
                    <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
                        "Once you delete a repository, there is no going back."
                    </p>
                    <div class="mt-3">
                        <Button variant=ButtonVariant::Danger extra_class="btn-delete-repo">
                            "Delete Repository"
                        </Button>
                    </div>
                </div>
            </Card>
        </div>
    }
}

#[component]
pub fn SettingsPage() -> impl IntoView {
    let auth = use_auth();
    let (user_sig, set_user) = signal(None::<UserResponse>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let (ssh_keys, set_ssh_keys) = signal(Vec::<SshKeyResponse>::new());
    let (ssh_loading, set_ssh_loading) = signal(true);

    leptos::task::spawn_local(async move {
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
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

    leptos::task::spawn_local(async move {
        let token = auth.0.with(|a| a.token.clone());
        let user_id = auth.0.with(|a| a.user_id.map(|id| id.to_string()));
        if let Some(uid) = user_id {
            let client = ApiClient::new(token);
            match client.get(&format!("/users/{uid}/ssh-keys")).await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(keys) = resp.json::<Vec<SshKeyResponse>>().await {
                        set_ssh_keys.set(keys);
                    }
                }
                _ => {}
            }
        }
        set_ssh_loading.set(false);
    });

    let username = move || auth.0.with(|a| a.username.clone().unwrap_or_default());

    view! {
        <div class="space-y-6">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"User Settings"</h1>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_error.set(None)) />
            </Show>

            <Card title="Profile" description="Manage your public profile information">
                <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                    <div class="flex items-center justify-center py-6">
                        <Spinner />
                    </div>
                </Show>
                <Show when=move || !loading.get() && user_sig.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                    <form class="space-y-5">
                        {move || user_sig.get().map(|u| {
                            view! {
                                <div class="grid grid-cols-1 sm:grid-cols-2 gap-5">
                                    <Input label="Username" name="username" input_type=InputType::Text value=username() required=true></Input>
                                    <Input label="Display Name" name="display_name" input_type=InputType::Text value=u.display_name.clone().unwrap_or_default()></Input>
                                </div>
                                <Input label="Email" name="email" input_type=InputType::Email value=u.email.clone() required=true></Input>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Role"</label>
                                    <div class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 dark:bg-gray-700 rounded-md text-sm text-gray-500 dark:text-gray-400 bg-gray-50 dark:bg-gray-800">
                                        <Badge color=BadgeColor::Info text=format!("{:?}", u.role).to_lowercase() />
                                    </div>
                                </div>
                                <Input label="Bio" name="bio" input_type=InputType::Textarea placeholder="Tell us about yourself..." value=u.bio.clone().unwrap_or_default()></Input>
                                <div>
                                    <Button variant=ButtonVariant::Primary extra_class="btn-save-profile">
                                        "Update Profile"
                                    </Button>
                                </div>
                            }
                        })}
                    </form>
                </Show>
            </Card>

            <Card title="SSH Keys" description="Manage your SSH keys for repository access">
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
                                        <span class="text-xs text-gray-400 dark:text-gray-500 shrink-0 ml-4">
                                            {key.created_at.clone()}
                                        </span>
                                    </div>
                                }
                            }
                        </For>
                    </div>
                </Show>
                <div class="mt-4">
                    <Button variant=ButtonVariant::Secondary extra_class="btn-add-ssh">
                        "Add SSH Key"
                    </Button>
                </div>
            </Card>

            <Card title="Danger Zone" description="Irreversible and destructive actions">
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
        </div>
    }
}
