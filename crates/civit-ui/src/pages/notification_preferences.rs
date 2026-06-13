#![forbid(unsafe_code)]

use leptos::prelude::*;

use crate::api::client::ApiClient;
use crate::components::{
    Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Input, InputType, Spinner,
};
use crate::state::auth::use_auth;
use crate::utils::*;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct NotificationSettings {
    email_enabled: bool,
    web_enabled: bool,
    frequency: String,
    #[serde(default)]
    per_repo_overrides: Vec<RepoNotificationOverride>,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct RepoNotificationOverride {
    repo_full_name: String,
    email_enabled: Option<bool>,
    web_enabled: Option<bool>,
    frequency: Option<String>,
}

const FREQUENCY_OPTIONS: &[(&str, &str)] = &[
    ("instant", "Instant"),
    ("daily", "Daily Digest"),
    ("weekly", "Weekly"),
];

#[component]
pub fn NotificationPreferencesPage() -> impl IntoView {
    let auth = use_auth();

    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (saving, set_saving) = signal(false);
    let (success, set_success) = signal(false);

    let (email_enabled, set_email_enabled) = signal(true);
    let (web_enabled, set_web_enabled) = signal(true);
    let (frequency, set_frequency) = signal("instant".to_string());
    let (overrides, set_overrides) = signal(Vec::<RepoNotificationOverride>::new());

    leptos::task::spawn_local(async move {
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        match client.get("/user/notification-settings").await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(settings) = resp.json::<NotificationSettings>().await {
                    set_email_enabled.set(settings.email_enabled);
                    set_web_enabled.set(settings.web_enabled);
                    set_frequency.set(settings.frequency);
                    set_overrides.set(settings.per_repo_overrides);
                }
            }
            _ => {
                set_email_enabled.set(true);
                set_web_enabled.set(true);
                set_frequency.set("instant".to_string());
            }
        }
        set_loading.set(false);
    });

    let handle_save = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        set_success.set(false);
        set_saving.set(true);

        let body = serde_json::json!({
            "email_enabled": email_enabled.get(),
            "web_enabled": web_enabled.get(),
            "frequency": frequency.get(),
            "per_repo_overrides": overrides.get(),
        });

        leptos::task::spawn_local(async move {
            let token = auth.0.with(|a| a.token.clone());
            let client = ApiClient::new(token);
            match client.put("/user/notification-settings", &body).await {
                Ok(resp) if resp.status().is_success() => {
                    set_success.set(true);
                }
                Ok(_) => {
                    set_error.set(Some("Failed to save notification settings.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_saving.set(false);
        });
    };

    let dismiss_error = Callback::new(move |_: ()| set_error.set(None));

    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-gray-100">
                    "Notification Preferences"
                </h1>
                <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                    "Configure how and when you receive notifications."
                </p>
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
                <form on:submit=handle_save class="space-y-6">
                    <Show when=move || success.get() fallback=|| view! { <div class="hidden"></div> }>
                        <div class="p-3 bg-green-50 dark:bg-green-900/20 border-l-4 border-green-500 dark:border-green-400 text-sm text-green-700 dark:text-green-400">
                            "Notification settings saved successfully."
                        </div>
                    </Show>

                    // Email Notifications
                    <Card
                        title="Email Notifications".to_string()
                        description="Configure email notification delivery".to_string()
                    >
                        <div class="space-y-4">
                            <label class="flex items-center gap-3 cursor-pointer">
                                <input
                                    type="checkbox"
                                    prop:checked=email_enabled
                                    on:change=move |ev| {
                                        let checked = event_target_checked(&ev);
                                        set_email_enabled.set(checked);
                                    }
                                    class="w-5 h-5 text-blue-600 border-gray-300 rounded focus:ring-blue-500 dark:border-gray-600"
                                />
                                <div>
                                    <span class="text-sm font-medium text-gray-900 dark:text-gray-100">
                                        "Enable email notifications"
                                    </span>
                                    <p class="text-xs text-gray-500 dark:text-gray-400">
                                        "Receive notifications via email for repository activity."
                                    </p>
                                </div>
                            </label>
                        </div>
                    </Card>

                    // Web Notifications
                    <Card
                        title="Web Notifications".to_string()
                        description="Configure browser/push notifications".to_string()
                    >
                        <div class="space-y-4">
                            <label class="flex items-center gap-3 cursor-pointer">
                                <input
                                    type="checkbox"
                                    prop:checked=web_enabled
                                    on:change=move |ev| {
                                        let checked = event_target_checked(&ev);
                                        set_web_enabled.set(checked);
                                    }
                                    class="w-5 h-5 text-blue-600 border-gray-300 rounded focus:ring-blue-500 dark:border-gray-600"
                                />
                                <div>
                                    <span class="text-sm font-medium text-gray-900 dark:text-gray-100">
                                        "Enable web notifications"
                                    </span>
                                    <p class="text-xs text-gray-500 dark:text-gray-400">
                                        "Show browser notifications for repository activity."
                                    </p>
                                </div>
                            </label>
                        </div>
                    </Card>

                    // Frequency
                    <Card
                        title="Notification Frequency".to_string()
                        description="How often to receive digest notifications".to_string()
                    >
                        <div class="space-y-3">
                            {FREQUENCY_OPTIONS.iter().map(|(value, label)| {
                                let v = (*value).to_string();
                                let l = (*label).to_string();
                                let checked_value = v.clone();
                                view! {
                                    <label class="flex items-center gap-3 cursor-pointer">
                                        <input
                                            type="radio"
                                            name="notification-frequency"
                                            value=v.clone()
                                            prop:checked=move || frequency.get() == checked_value
                                            on:change=move |_| set_frequency.set(v.clone())
                                            class="w-4 h-4 text-blue-600 border-gray-300 focus:ring-blue-500 dark:border-gray-600"
                                        />
                                        <div>
                                            <span class="text-sm font-medium text-gray-900 dark:text-gray-100">
                                                {l.clone()}
                                            </span>
                                        </div>
                                    </label>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </Card>

                    // Per-repo overrides
                    <Card
                        title="Per-Repository Overrides".to_string()
                        description="Override notification settings for specific repositories".to_string()
                    >
                        {move || {
                            let ovs = overrides.get();
                            if ovs.is_empty() {
                                view! {
                                    <div class="py-6 text-center text-gray-400 dark:text-gray-500 text-sm">
                                        "No repository-specific overrides configured."
                                    </div>
                                }.into_view()
                            } else {
                                view! {
                                    <div class="divide-y divide-gray-100 dark:divide-gray-700">
                                        {ovs.into_iter().map(|ov| {
                                            let repo = ov.repo_full_name.clone();
                                            view! {
                                                <div class="flex items-center justify-between py-3">
                                                    <span class="text-sm font-mono text-gray-900 dark:text-gray-100">
                                                        {repo}
                                                    </span>
                                                    <div class="flex items-center gap-2">
                                                        <Badge
                                                            color=if ov.email_enabled.unwrap_or(true) { BadgeColor::Success } else { BadgeColor::Neutral }
                                                            text="email".to_string()
                                                        />
                                                        <Badge
                                                            color=if ov.web_enabled.unwrap_or(true) { BadgeColor::Success } else { BadgeColor::Neutral }
                                                            text="web".to_string()
                                                        />
                                                        <Badge
                                                            color=BadgeColor::Info
                                                            text=ov.frequency.unwrap_or_else(|| "instant".to_string())
                                                        />
                                                    </div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_view()
                            }
                        }}
                    </Card>

                    <div class="flex gap-3">
                        <Button variant=ButtonVariant::Primary disabled=saving.get()>
                            {move || if saving.get() { "Saving..." } else { "Save Preferences" }}
                        </Button>
                    </div>
                </form>
            </Show>
        </div>
    }
}
