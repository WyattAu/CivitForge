#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
#[cfg(feature = "csr")]
use wasm_bindgen::JsCast;

use crate::api::client::ApiClient;
use crate::api::types::AuthResponse;
use crate::components::{Button, ButtonVariant, Input, InputType};
use crate::state::auth::{login, use_auth};

#[derive(Debug, Clone, serde::Serialize)]
struct LoginRequest {
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let (is_register, set_is_register) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (loading, set_loading) = signal(false);
    let auth = use_auth();
    let navigate = use_navigate();

    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);

        let username_val = get_value("username");
        let email_val = get_value("email");
        let display_name_val = get_value("display_name");

        if email_val.is_empty() {
            set_error.set(Some("Email is required.".to_string()));
            return;
        }

        let body = LoginRequest {
            email: email_val,
            username: if is_register.get() {
                Some(username_val)
            } else {
                None
            },
            display_name: if is_register.get() {
                let dn = display_name_val;
                if dn.is_empty() { None } else { Some(dn) }
            } else {
                None
            },
        };

        set_loading.set(true);

        let auth_clone = auth;
        let navigate_clone = navigate.clone();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(None);
            let result = client.post("/auth/login", &body).await;

            match result {
                Ok(resp) if resp.status().is_success() => match resp.json::<AuthResponse>().await {
                    Ok(data) => {
                        login(&auth_clone, data.user.id, data.user.username, data.token);
                        navigate_clone("/repos", Default::default());
                    }
                    Err(_) => {
                        set_error.set(Some("Failed to process response.".to_string()));
                    }
                },
                Ok(_) => {
                    set_error.set(Some(
                        "Login failed. Please check your credentials.".to_string(),
                    ));
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="flex min-h-screen items-center justify-center py-12 px-4 sm:px-6 lg:px-8">
            <div class="w-full max-w-md">
                <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-8">
                    <div class="text-center mb-8">
                        <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                            {move || if is_register.get() { "Create Account" } else { "Sign In" }}
                        </h1>
                        <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
                            {move || if is_register.get() { "Join the CivitForge community." } else { "Sign in to your CivitForge account." }}
                        </p>
                    </div>

                    <Show when=move || error.get().is_some()>
                        <div class="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md">
                            <p class="text-sm text-red-700 dark:text-red-400">{move || error.get().unwrap_or_default()}</p>
                        </div>
                    </Show>

                    <form on:submit=handle_submit class="space-y-5">
                        <Show when=move || is_register.get()>
                            <Input
                                label="Username"
                                name="username"
                                id="username"
                                input_type=InputType::Text
                                placeholder="johndoe"
                                required=true
                            ></Input>
                            <Input
                                label="Display Name"
                                name="display_name"
                                id="display_name"
                                input_type=InputType::Text
                                placeholder="John Doe"
                                required=false
                            ></Input>
                        </Show>
                        <Input
                            label="Email"
                            name="email"
                            id="email"
                            input_type=InputType::Email
                            placeholder="you@example.com"
                            required=true
                        ></Input>
                        <Button
                            variant=ButtonVariant::Primary
                            extra_class="w-full justify-center"
                            disabled=loading.get()
                        >
                            {move || if loading.get() { "Signing in..." } else if is_register.get() { "Register" } else { "Sign In" }}
                        </Button>
                    </form>

                    <div class="mt-6 text-center text-sm">
                        <button
                            class="text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300"
                            on:click=move |_| set_is_register.set(!is_register.get())
                        >
                            {move || if is_register.get() { "Already have an account? Sign In" } else { "Don't have an account? Register" }}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}

fn get_value(name: &str) -> String {
    #[cfg(feature = "csr")]
    {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return String::new(),
        };
        let doc = match window.document() {
            Some(d) => d,
            None => return String::new(),
        };
        let el = match doc.get_element_by_id(name) {
            Some(el) => el,
            None => return String::new(),
        };
        match el.dyn_into::<web_sys::HtmlInputElement>() {
            Ok(input) => input.value(),
            Err(_) => String::new(),
        }
    }
    #[cfg(not(feature = "csr"))]
    {
        let _ = name;
        String::new()
    }
}
