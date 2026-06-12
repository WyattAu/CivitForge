#![forbid(unsafe_code)]

use leptos::prelude::*;

use crate::api::client::ApiClient;
use crate::components::{Button, ButtonVariant, Card, ErrorBanner, Input, InputType, Spinner};
use crate::state::auth::use_auth;
use crate::utils::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SiteSettingsData {
    pub id: i32,
    pub site_name: String,
    pub site_description: String,
    pub footer_text: String,
    pub logo_url: String,
    pub contact_email: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UpdateSiteSettingsBody {
    pub site_name: String,
    pub site_description: String,
    pub footer_text: String,
    pub logo_url: String,
    pub contact_email: String,
}

#[component]
pub fn SiteSettingsPage() -> impl IntoView {
    let auth = use_auth();
    let (settings, set_settings) = signal(None::<SiteSettingsData>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (saving, set_saving) = signal(false);
    let (success, set_success) = signal(false);

    let fetch_settings = move || {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            match client.get("/admin/settings").await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<SiteSettingsData>().await {
                        Ok(data) => set_settings.set(Some(data)),
                        Err(_) => set_error.set(Some("Failed to load settings.".to_string())),
                    }
                }
                Ok(_) => set_error.set(Some("Failed to load settings.".to_string())),
                Err(_) => set_error.set(Some("Network error.".to_string())),
            }
            set_loading.set(false);
        });
    };

    leptos::task::spawn_local(async move {
        fetch_settings();
    });

    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        set_success.set(false);

        let site_name = get_input_value("site-settings-name");
        let site_description = get_input_value("site-settings-description");
        let footer_text = get_input_value("site-settings-footer");
        let logo_url = get_input_value("site-settings-logo");
        let contact_email = get_input_value("site-settings-email");

        if site_name.trim().is_empty() {
            set_error.set(Some("Site name is required.".to_string()));
            return;
        }

        let body = UpdateSiteSettingsBody {
            site_name: site_name.trim().to_string(),
            site_description: site_description.trim().to_string(),
            footer_text: footer_text.trim().to_string(),
            logo_url: logo_url.trim().to_string(),
            contact_email: contact_email.trim().to_string(),
        };

        let token = auth.0.with(|a| a.token.clone());
        set_saving.set(true);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            match client.put("/admin/settings", &body).await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(updated) = resp.json::<SiteSettingsData>().await {
                        set_settings.set(Some(updated));
                    }
                    set_success.set(true);
                }
                Ok(_) => set_error.set(Some("Failed to update settings.".to_string())),
                Err(_) => set_error.set(Some("Network error.".to_string())),
            }
            set_saving.set(false);
        });
    };

    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Site Settings"</h1>
                <p class="mt-1 text-gray-600 dark:text-gray-400">"Configure your CivitForge instance."</p>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_error.set(None)) />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center justify-center py-12">
                    <Spinner />
                    <span class="ml-3 text-gray-500 dark:text-gray-400">"Loading settings..."</span>
                </div>
            </Show>

            <Show when=move || !loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <Card title="General Settings".to_string() description="Site-wide configuration options".to_string()>
                    <form on:submit=handle_submit class="space-y-5">
                        <Show when=move || success.get() fallback=|| view! { <div class="hidden"></div> }>
                            <div class="p-3 bg-green-50 dark:bg-green-900/20 border-l-4 border-green-500 dark:border-green-400 text-sm text-green-700 dark:text-green-400">
                                "Settings saved successfully."
                            </div>
                        </Show>
                        {move || settings.get().map(|s| view! {
                            <Input
                                label="Site Name"
                                name="site-settings-name"
                                id="site-settings-name"
                                input_type=InputType::Text
                                value=s.site_name
                                required=true
                            />
                            <Input
                                label="Site Description"
                                name="site-settings-description"
                                id="site-settings-description"
                                input_type=InputType::Textarea
                                placeholder="A brief description of your instance..."
                                value=s.site_description
                            />
                            <Input
                                label="Logo URL"
                                name="site-settings-logo"
                                id="site-settings-logo"
                                input_type=InputType::Text
                                placeholder="https://example.com/logo.png"
                                value=s.logo_url
                            />
                            <Input
                                label="Footer Text"
                                name="site-settings-footer"
                                id="site-settings-footer"
                                input_type=InputType::Text
                                placeholder="Displayed in the page footer..."
                                value=s.footer_text
                            />
                            <Input
                                label="Contact Email"
                                name="site-settings-email"
                                id="site-settings-email"
                                input_type=InputType::Email
                                placeholder="admin@example.com"
                                value=s.contact_email
                            />
                        })}
                        <div>
                            <Button variant=ButtonVariant::Primary disabled=saving.get()>
                                {move || if saving.get() { "Saving..." } else { "Save Settings" }}
                            </Button>
                        </div>
                    </form>
                </Card>
            </Show>
        </div>
    }
}
