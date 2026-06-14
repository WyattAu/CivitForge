#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::api::types::AuthUser;
use crate::components::{
    Avatar, Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Input, InputType, Spinner,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    website: Option<String>,
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    let params = use_params_map();
    let username_param = move || params.with(|p| p.get("username").unwrap_or_default());
    let auth = use_auth();

    let (user_sig, set_user) = signal(None::<UserResponse>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (editing, set_editing) = signal(false);

    let (saving, set_saving) = signal(false);
    let (save_error, set_save_error) = signal(None::<String>);
    let (save_success, set_save_success) = signal(false);

    let is_own_profile = Signal::derive(move || {
        let param_user = username_param();
        let auth_user = auth.0.with(|a| a.username.clone().unwrap_or_default());
        param_user == auth_user || param_user.is_empty()
    });

    let fetch_user = move || {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        let uname = username_param();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            if uname.is_empty() {
                match client.get("/auth/me").await {
                    Ok(resp) if resp.status().is_success() => {
                        if let Ok(auth_user) = resp.json::<AuthUser>().await {
                            let client2 = ApiClient::new(None);
                            match client2.get(&format!("/users/{}", auth_user.id)).await {
                                Ok(r) if r.status().is_success() => {
                                    if let Ok(u) = r.json::<UserResponse>().await {
                                        set_user.set(Some(u));
                                    }
                                }
                                _ => {
                                    set_error.set(Some("Failed to load user profile.".to_string()))
                                }
                            }
                        }
                    }
                    _ => set_error.set(Some("auth_required".to_string())),
                }
            } else {
                match client.get("/users").await {
                    Ok(resp) if resp.status().is_success() => {
                        if let Ok(users) = resp.json::<Vec<UserResponse>>().await {
                            let found = users.into_iter().find(|u| u.username == uname);
                            if let Some(u) = found {
                                set_user.set(Some(u));
                            } else {
                                set_error.set(Some("User not found.".to_string()));
                            }
                        }
                    }
                    _ => set_error.set(Some("Failed to load user profile.".to_string())),
                }
            }
            set_loading.set(false);
        });
    };

    fetch_user();

    let handle_save = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_save_error.set(None);
        set_save_success.set(false);

        let display_name_val = get_input_value("profile-display-name");
        let bio_val = get_input_value("profile-bio");
        let avatar_url_val = get_input_value("profile-avatar-url");
        let location_val = get_input_value("profile-location");
        let website_val = get_input_value("profile-website");

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
            avatar_url: if avatar_url_val.trim().is_empty() {
                None
            } else {
                Some(avatar_url_val.trim().to_string())
            },
            location: if location_val.trim().is_empty() {
                None
            } else {
                Some(location_val.trim().to_string())
            },
            website: if website_val.trim().is_empty() {
                None
            } else {
                Some(website_val.trim().to_string())
            },
        };

        set_saving.set(true);
        let auth_clone = auth;
        leptos::task::spawn_local(async move {
            let token = auth_clone.0.with(|a| a.token.clone());
            let client = ApiClient::new(token);
            match client.patch("/user/profile", &body).await {
                Ok(resp) if resp.status().is_success() => {
                    set_save_success.set(true);
                    set_editing.set(false);
                    fetch_user();
                }
                Ok(_) => {
                    set_save_error.set(Some("Failed to update profile.".to_string()));
                }
                Err(_) => {
                    set_save_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_saving.set(false);
        });
    };

    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-gray-100">"User Profile"</h1>
            </div>

            <Show when=move || error.get().is_some() && error.get().as_deref() == Some("auth_required") fallback=|| view! { <div class="hidden"></div> }>
                <Card title="Sign in required".to_string() description="You must be signed in to view your profile".to_string()>
                    <div class="py-8 text-center">
                        <p class="text-gray-600 dark:text-gray-400 mb-4">"Please sign in to view your profile."</p>
                        <A href="/login">
                            <span class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium bg-blue-600 hover:bg-blue-700 text-white dark:bg-blue-500 dark:hover:bg-blue-600 transition-colors">
                                "Sign In"
                            </span>
                        </A>
                    </div>
                </Card>
            </Show>

            <Show when=move || error.get().is_some() && error.get().as_deref() != Some("auth_required") fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_error.set(None)) />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center justify-center py-12">
                    <Spinner />
                    <span class="ml-3 text-gray-500 dark:text-gray-400">"Loading profile..."</span>
                </div>
            </Show>

            <Show when=move || !loading.get() && user_sig.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    // Left column: avatar + info card
                    <div class="lg:col-span-1 space-y-4">
                        <Show when=move || user_sig.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                            <ProfileUserInfo user_sig=user_sig />
                            <ProfileDetails user_sig=user_sig />
                        </Show>
                        <Show when=move || is_own_profile.get()>
                            <Card>
                                <button
                                    class="w-full inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900 bg-blue-600 hover:bg-blue-700 text-white dark:bg-blue-500 dark:hover:bg-blue-600"
                                    on:click=move |_| set_editing.set(true)
                                >
                                    "Edit Profile"
                                </button>
                            </Card>
                        </Show>
                    </div>

                    // Right column: activity / repos placeholder
                    <div class="lg:col-span-2 space-y-4">
                        <Show when=move || is_own_profile.get() && editing.get() fallback=|| view! { <div class="hidden"></div> }>
                            <Card title="Edit Profile".to_string() description="Update your public profile information".to_string()>
                                <form on:submit=handle_save class="space-y-5">
                                    <Show when=move || save_success.get() fallback=|| view! { <div class="hidden"></div> }>
                                        <div class="p-3 bg-green-50 dark:bg-green-900/20 border-l-4 border-green-500 dark:border-green-400 rounded-r-sm text-sm text-green-700 dark:text-green-400">
                                            "Profile updated successfully."
                                        </div>
                                    </Show>
                                    <Show when=move || save_error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                                        <ErrorBanner message=move || save_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_save_error.set(None)) />
                                    </Show>
                                    <ProfileEditForm user_sig=user_sig />
                                    <div class="flex gap-3">
                                        <Button variant=ButtonVariant::Primary disabled=saving.get()>
                                            {move || if saving.get() { "Saving..." } else { "Save Changes" }}
                                        </Button>
                                        <button
                                            type="button"
                                            class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900 bg-gray-200 hover:bg-gray-300 text-gray-900 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-100"
                                            on:click=move |_| {
                                                set_editing.set(false);
                                                set_save_error.set(None);
                                                set_save_success.set(false);
                                            }
                                        >
                                            "Cancel"
                                        </button>
                                    </div>
                                </form>
                            </Card>
                        </Show>

                        <Card title="Repositories".to_string() description="Public repositories owned by this user".to_string()>
                            <div class="py-8 text-center text-gray-400 dark:text-gray-500">
                                <p class="text-sm">"Repository list coming soon"</p>
                            </div>
                        </Card>

                        <Card title="Activity".to_string() description="Recent contributions and activity".to_string()>
                            <div class="py-8 text-center text-gray-400 dark:text-gray-500">
                                <p class="text-sm">"Activity feed coming soon"</p>
                            </div>
                        </Card>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn ProfileUserInfo(user_sig: ReadSignal<Option<UserResponse>>) -> impl IntoView {
    let has_bio = Signal::derive(move || user_sig.get().is_some_and(|u| u.bio.is_some()));

    view! {
        <Card>
            <div class="flex flex-col items-center text-center">
                {move || user_sig.get().map(|u| {
                    view! {
                        <Avatar src=u.avatar_url.unwrap_or_default() name=u.username.clone() size=96 />
                        <h2 class="mt-4 text-xl font-bold text-gray-900 dark:text-gray-100">
                            {u.display_name.unwrap_or_else(|| u.username.clone())}
                        </h2>
                        <p class="text-sm text-gray-500 dark:text-gray-400 font-mono">
                            {format!("@{}", u.username)}
                        </p>
                    }
                })}
                <Show when=move || has_bio.get() fallback=|| view! { <div class="hidden"></div> }>
                    <p class="mt-3 text-sm text-gray-600 dark:text-gray-300 max-w-xs">
                        {move || user_sig.get().and_then(|u| u.bio).unwrap_or_default()}
                    </p>
                </Show>
                {move || user_sig.get().map(|u| {
                    view! {
                        <div class="mt-3">
                            <Badge color=BadgeColor::Info text=format!("{:?}", u.role).to_lowercase() />
                        </div>
                    }
                })}
            </div>
        </Card>
    }
}

#[component]
fn ProfileDetails(user_sig: ReadSignal<Option<UserResponse>>) -> impl IntoView {
    let loc = Signal::derive(move || {
        user_sig
            .get()
            .and_then(|u| u.location.clone())
            .unwrap_or_default()
    });
    let website = Signal::derive(move || {
        user_sig
            .get()
            .and_then(|u| u.website.clone())
            .unwrap_or_default()
    });
    let created = Signal::derive(move || {
        user_sig
            .get()
            .map(|u| u.created_at.format("%b %d, %Y").to_string())
            .unwrap_or_default()
    });
    let has_loc = Signal::derive(move || !loc.get().is_empty());
    let has_website = Signal::derive(move || !website.get().is_empty());

    view! {
        <Card title="Details".to_string()>
            <div class="space-y-3 text-sm">
                <Show when=move || has_loc.get() fallback=|| view! { <div class="hidden"></div> }>
                    <div class="flex items-center gap-2 text-gray-600 dark:text-gray-400">
                        <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z"/>
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z"/>
                        </svg>
                        <span>{move || loc.get()}</span>
                    </div>
                </Show>
                <Show when=move || has_website.get() fallback=|| view! { <div class="hidden"></div> }>
                    <div class="flex items-center gap-2 text-gray-600 dark:text-gray-400">
                        <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"/>
                        </svg>
                        <a href=move || website.get() target="_blank" class="text-blue-600 dark:text-blue-400 hover:underline truncate">
                            {move || website.get()}
                        </a>
                    </div>
                </Show>
                <div class="flex items-center gap-2 text-gray-600 dark:text-gray-400">
                    <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"/>
                    </svg>
                    <span>{move || format!("Joined {}", created.get())}</span>
                </div>
            </div>
        </Card>
    }
}

#[component]
fn ProfileEditForm(user_sig: ReadSignal<Option<UserResponse>>) -> impl IntoView {
    view! {
        {move || user_sig.get().map(|u| {
            view! {
                <div class="space-y-5">
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-5">
                        <Input label="Display Name" name="profile-display-name" id="profile-display-name" input_type=InputType::Text value=u.display_name.unwrap_or_default() />
                        <Input label="Avatar URL" name="profile-avatar-url" id="profile-avatar-url" input_type=InputType::Text placeholder="https://example.com/avatar.png" value=u.avatar_url.unwrap_or_default() />
                    </div>
                    <Input label="Bio" name="profile-bio" id="profile-bio" input_type=InputType::Textarea placeholder="Tell us about yourself..." value=u.bio.unwrap_or_default() />
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-5">
                        <Input label="Location" name="profile-location" id="profile-location" input_type=InputType::Text placeholder="San Francisco, CA" value=u.location.unwrap_or_default() />
                        <Input label="Website" name="profile-website" id="profile-website" input_type=InputType::Text placeholder="https://example.com" value=u.website.unwrap_or_default() />
                    </div>
                </div>
            }
        })}
    }
}
