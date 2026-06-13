#![forbid(unsafe_code)]

#[cfg(feature = "csr")]
use js_sys::{Function, Reflect, global};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
#[cfg(feature = "csr")]
use wasm_bindgen::JsCast;
#[cfg(feature = "csr")]
use wasm_bindgen::JsValue;

use crate::api::client::ApiClient;
use crate::api::types::{AuthResponse, RegisterRequest};
use crate::components::{Button, ButtonVariant, ErrorBanner, Input, InputType};
use crate::state::auth::{login, use_auth};

#[component]
pub fn RegisterPage() -> impl IntoView {
    let (error, set_error) = signal(None::<String>);
    let (loading, set_loading) = signal(false);
    let auth = use_auth();
    let nav = use_navigate();
    let (nav_sig, _) = signal(nav);

    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);

        let username_val = get_value("username");
        let password_val = get_value("password");

        if username_val.is_empty() {
            set_error.set(Some("Username is required.".to_string()));
            return;
        }
        if password_val.is_empty() {
            set_error.set(Some("Password is required.".to_string()));
            return;
        }

        let email_val = get_value("email");
        let display_name_val = get_value("display_name");
        let confirm_val = get_value("confirm_password");

        if email_val.is_empty() {
            set_error.set(Some("Email is required.".to_string()));
            return;
        }
        if password_val.len() < 8 {
            set_error.set(Some("Password must be at least 8 characters.".to_string()));
            return;
        }
        if password_val != confirm_val {
            set_error.set(Some("Passwords do not match.".to_string()));
            return;
        }

        let auth_clone = auth;
        let nav_clone = nav_sig.get();

        set_loading.set(true);

        leptos::task::spawn_local(async move {
            let client = ApiClient::new(None);
            let uname = username_val.clone();
            let body = RegisterRequest {
                username: uname.clone(),
                email: email_val,
                display_name: if display_name_val.is_empty() {
                    uname
                } else {
                    display_name_val
                },
                password: password_val,
            };
            let result = client.post("/auth/register", &body).await;

            match result {
                Ok(resp) => {
                    let status = resp.status();
                    let text = resp.text().await.unwrap_or_default();
                    if status.is_success() {
                        match serde_json::from_str::<AuthResponse>(&text) {
                            Ok(data) => {
                                login(&auth_clone, data.user.id, data.user.username, data.token, data.user.is_admin);
                                nav_clone("/repos", Default::default());
                            }
                            Err(e) => {
                                set_error.set(Some(format!("Failed to process response: {e}")));
                            }
                        }
                    } else {
                        set_error.set(Some(format!("Registration failed ({status}): {text}")));
                    }
                }
                Err(e) => {
                    set_error.set(Some(format!("Network error: {e}")));
                }
            }
            set_loading.set(false);
        });
    };

    let dismiss_error = Callback::new(move |_: ()| set_error.set(None));

    view! {
        <div class="flex min-h-screen items-center justify-center py-12 px-4 sm:px-6 lg:px-8">
            <div class="w-full max-w-md">
                <div class="bg-white dark:bg-gray-800 rounded-none shadow-sm border-2 border-gray-200 dark:border-gray-700 p-8">
                    <div class="text-center mb-8">
                        <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                            "Create Account"
                        </h1>
                        <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
                            "Join the CivitForge community."
                        </p>
                    </div>

                    <Show when=move || error.get().is_some()>
                        <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=dismiss_error />
                    </Show>

                    <form on:submit=handle_submit class="space-y-5">
                        <Input
                            label="Username"
                            name="username"
                            id="username"
                            input_type=InputType::Text
                            placeholder="johndoe"
                            required=true
                        ></Input>
                        <Input
                            label="Email"
                            name="email"
                            id="email"
                            input_type=InputType::Email
                            placeholder="you@example.com"
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
                        <Input
                            label="Password"
                            name="password"
                            id="password"
                            input_type=InputType::Password
                            placeholder="••••••••"
                            required=true
                        ></Input>
                        <Input
                            label="Confirm Password"
                            name="confirm_password"
                            id="confirm_password"
                            input_type=InputType::Password
                            placeholder="••••••••"
                            required=true
                        ></Input>
                        <Button
                            variant=ButtonVariant::Primary
                            extra_class="w-full justify-center"
                            disabled=loading.get()
                        >
                            {move || if loading.get() { "Working..." } else { "Create Account" }}
                        </Button>
                    </form>

                    <div class="mt-6 text-center text-sm">
                        <a
                            href="/login"
                            class="text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300"
                        >
                            "Already have an account? Sign In"
                        </a>
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
