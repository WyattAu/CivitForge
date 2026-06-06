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
use crate::api::types::{AuthResponse, LoginRequest, RegisterRequest};
use crate::components::{Button, ButtonVariant, ErrorBanner, Input, InputType};
use crate::state::auth::{login, use_auth};

#[component]
pub fn LoginPage() -> impl IntoView {
    let (is_register, set_is_register) = signal(false);
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

        let auth_clone = auth;
        let nav_clone = nav_sig.get();
        let register_mode = is_register.get();

        set_loading.set(true);

        leptos::task::spawn_local(async move {
            let client = ApiClient::new(None);
            let result = if register_mode {
                let email_val = get_value("email");
                let display_name_val = get_value("display_name");
                let confirm_val = get_value("confirm_password");

                if email_val.is_empty() {
                    set_error.set(Some("Email is required.".to_string()));
                    set_loading.set(false);
                    return;
                }
                if password_val.len() < 8 {
                    set_error.set(Some("Password must be at least 8 characters.".to_string()));
                    set_loading.set(false);
                    return;
                }
                if password_val != confirm_val {
                    set_error.set(Some("Passwords do not match.".to_string()));
                    set_loading.set(false);
                    return;
                }

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
                client.post("/auth/register", &body).await
            } else {
                let body = LoginRequest {
                    username: username_val,
                    password: password_val,
                };
                client.post("/auth/login", &body).await
            };

            match result {
                Ok(resp) => {
                    let status = resp.status();
                    let text = resp.text().await.unwrap_or_default();
                    if status.is_success() {
                        match serde_json::from_str::<AuthResponse>(&text) {
                            Ok(data) => {
                                login(&auth_clone, data.user.id, data.user.username, data.token);
                                nav_clone("/repos", Default::default());
                            }
                            Err(e) => {
                                set_error.set(Some(format!("Failed to process response: {e}")));
                            }
                        }
                    } else {
                        set_error.set(Some(format!("Login failed ({status}): {text}")));
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

    // Auto-login: check URL hash fragment for credentials injected by Tauri CLI args
    // Format: #auto=base64({json})
    #[cfg(feature = "csr")]
    {
        leptos::task::spawn_local(async move {
            let window = web_sys::window().unwrap();
            let hash = window.location().hash().unwrap_or_default();
            if let Some(auto_data) = hash.strip_prefix("#auto=") {
                if let Ok(decoded) = base64_decode(auto_data) {
                    // Clear the hash fragment from URL
                    let _ = window.location().set_hash("");
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&decoded) {
                        let username = data
                            .get("username")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let password = data
                            .get("password")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let email = data
                            .get("email")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let display_name = data
                            .get("display_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if !username.is_empty() && !password.is_empty() {
                            set_loading.set(true);
                            let client = ApiClient::new(None);
                            let login_body = LoginRequest {
                                username: username.clone(),
                                password: password.clone(),
                            };
                            match client.post("/auth/login", &login_body).await {
                                Ok(resp) if resp.status().is_success() => {
                                    let text = resp.text().await.unwrap_or_default();
                                    if let Ok(data) = serde_json::from_str::<AuthResponse>(&text) {
                                        login(&auth, data.user.id, data.user.username, data.token);
                                        nav_sig.get()("/repos", Default::default());
                                    }
                                }
                                _ => {
                                    let reg_body = RegisterRequest {
                                        username: username.clone(),
                                        email: if email.is_empty() {
                                            format!("{username}@localhost")
                                        } else {
                                            email
                                        },
                                        display_name: if display_name.is_empty() {
                                            username.clone()
                                        } else {
                                            display_name
                                        },
                                        password,
                                    };
                                    match client.post("/auth/register", &reg_body).await {
                                        Ok(resp) if resp.status().is_success() => {
                                            let text = resp.text().await.unwrap_or_default();
                                            if let Ok(data) =
                                                serde_json::from_str::<AuthResponse>(&text)
                                            {
                                                login(
                                                    &auth,
                                                    data.user.id,
                                                    data.user.username,
                                                    data.token,
                                                );
                                                nav_sig.get()("/repos", Default::default());
                                            }
                                        }
                                        Ok(resp) => {
                                            let status = resp.status();
                                            let text = resp.text().await.unwrap_or_default();
                                            set_error.set(Some(format!(
                                                "Auto-login failed ({status}): {text}"
                                            )));
                                        }
                                        Err(e) => {
                                            set_error.set(Some(format!(
                                                "Auto-login network error: {e}"
                                            )));
                                        }
                                    }
                                }
                            }
                            set_loading.set(false);
                        }
                    }
                }
            }
        });
    }

    view! {
        <div class="flex min-h-screen items-center justify-center py-12 px-4 sm:px-6 lg:px-8">
            <div class="w-full max-w-md">
                <div class="bg-white dark:bg-gray-800 rounded-none shadow-sm border-2 border-gray-200 dark:border-gray-700 p-8">
                    <div class="text-center mb-8">
                        <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                            {move || if is_register.get() { "Create Account" } else { "Sign In" }}
                        </h1>
                        <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
                            {move || if is_register.get() { "Join the CivitForge community." } else { "Sign in to your CivitForge account." }}
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
                        <Show when=move || is_register.get()>
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
                        </Show>
                        <Input
                            label="Password"
                            name="password"
                            id="password"
                            input_type=InputType::Password
                            placeholder="••••••••"
                            required=true
                        ></Input>
                        <Show when=move || is_register.get()>
                            <Input
                                label="Confirm Password"
                                name="confirm_password"
                                id="confirm_password"
                                input_type=InputType::Password
                                placeholder="••••••••"
                                required=true
                            ></Input>
                        </Show>
                        <Button
                            variant=ButtonVariant::Primary
                            extra_class="w-full justify-center"
                            disabled=loading.get()
                        >
                            {move || if loading.get() { "Working..." } else if is_register.get() { "Register" } else { "Sign In" }}
                        </Button>
                    </form>

                    <div class="mt-6 text-center text-sm">
                        <button
                            type="button"
                            class="text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300"
                            on:click=move |_| set_is_register.set(!is_register.get())
                            aria-label=move || if is_register.get() { "Switch to sign in form" } else { "Switch to registration form" }
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

#[cfg(feature = "csr")]
fn base64_decode(data: &str) -> Result<String, String> {
    let win = global();
    let func_val = Reflect::get(&win, &JsValue::from_str("atob"))
        .map_err(|e| format!("atob not found: {e:?}"))?;
    let func: Function = func_val
        .dyn_into()
        .map_err(|e| format!("not a function: {e:?}"))?;
    match func.call1(&JsValue::null(), &JsValue::from_str(data)) {
        Ok(v) => match v.as_string() {
            Some(s) => Ok(s),
            None => Err("atob returned non-string".into()),
        },
        Err(e) => Err(format!("atob error: {e:?}")),
    }
}
