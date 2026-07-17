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

const OAUTH_CLIENT_ID: &str = "civitforge-web";
const PKCE_VERIFIER_KEY: &str = "civitforge_pkce_verifier";
const OAUTH_REDIRECT_STATE_KEY: &str = "civitforge_oauth_state";

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
                // For sign-in, use OAuth2/PKCE flow
                let code_verifier = generate_pkce_verifier();
                let code_challenge = generate_pkce_challenge(&code_verifier);
                
                // Store verifier in session storage
                session_storage_set(PKCE_VERIFIER_KEY, &code_verifier);
                
                // Generate random state for CSRF protection
                let state = generate_random_state();
                session_storage_set(OAUTH_REDIRECT_STATE_KEY, &state);
                
                // First, we need to authenticate the user to get a session
                // We'll use the login endpoint to verify credentials, then redirect to OAuth
                let login_body = LoginRequest {
                    username: username_val,
                    password: password_val,
                };
                
                match client.post("/auth/login", &login_body).await {
                    Ok(resp) if resp.status().is_success() => {
                        let text = resp.text().await.unwrap_or_default();
                        if let Ok(data) = serde_json::from_str::<AuthResponse>(&text) {
                            // Store the token temporarily to use for OAuth authorize
                            login(
                                &auth_clone,
                                data.user.id,
                                data.user.username,
                                data.token.clone(),
                                data.user.is_admin,
                            );
                            
                            // Redirect to OAuth authorize endpoint
                            let redirect_uri = get_current_origin();
                            let authorize_url = format!(
                                "/api/v1/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&code_challenge={}&code_challenge_method=S256&state={}",
                                OAUTH_CLIENT_ID,
                                urlencoding::encode(&redirect_uri),
                                code_challenge,
                                state
                            );
                            
                            // Use the token to make an authenticated request
                            let auth_client = ApiClient::new(Some(data.token));
                            match auth_client.get(&authorize_url).await {
                                Ok(_) => {
                                    // The redirect will happen automatically
                                }
                                Err(e) => {
                                    set_error.set(Some(format!("OAuth redirect failed: {e}")));
                                    set_loading.set(false);
                                }
                            }
                        } else {
                            set_error.set(Some("Failed to process login response".to_string()));
                            set_loading.set(false);
                        }
                    }
                    Ok(resp) => {
                        let text = resp.text().await.unwrap_or_default();
                        set_error.set(Some(format!("Login failed: {text}")));
                        set_loading.set(false);
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Network error: {e}")));
                        set_loading.set(false);
                    }
                }
                return;
            };

            match result {
                Ok(resp) => {
                    let status = resp.status();
                    let text = resp.text().await.unwrap_or_default();
                    if status.is_success() {
                        match serde_json::from_str::<AuthResponse>(&text) {
                            Ok(data) => {
                                login(
                                    &auth_clone,
                                    data.user.id,
                                    data.user.username,
                                    data.token,
                                    data.user.is_admin,
                                );
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

    // OAuth2/PKCE: Check for callback code parameter on page load
    #[cfg(feature = "csr")]
    {
        leptos::task::spawn_local(async move {
            let window = web_sys::window().expect("browser window available");
            let location = window.location();
            let search = location.search().unwrap_or_default();
            
            // Check if this is an OAuth callback with ?code= parameter
            if search.contains("code=") {
                let params = parse_url_params(&search);
                if let Some(code) = params.get("code") {
                    set_loading.set(true);
                    
                    // Get stored PKCE code_verifier
                    let verifier = session_storage_get(PKCE_VERIFIER_KEY);
                    
                    if let Some(code_verifier) = verifier {
                        let client = ApiClient::new(None);
                        
                        // Exchange code for tokens
                        let token_body = serde_json::json!({
                            "grant_type": "authorization_code",
                            "code": code,
                            "redirect_uri": get_current_origin(),
                            "code_verifier": code_verifier,
                            "client_id": OAUTH_CLIENT_ID
                        });
                        
                        match client.post("/oauth/token", &token_body).await {
                            Ok(resp) if resp.status().is_success() => {
                                let text = resp.text().await.unwrap_or_default();
                                if let Ok(token_data) = serde_json::from_str::<serde_json::Value>(&text) {
                                    if let Some(access_token) = token_data.get("access_token").and_then(|v| v.as_str()) {
                                        // Get user info with the token
                                        let user_client = ApiClient::new(Some(access_token.to_string()));
                                        match user_client.get("/auth/me").await {
                                            Ok(me_resp) if me_resp.status().is_success() => {
                                                if let Ok(user) = me_resp.json::<crate::api::types::AuthUser>().await {
                                                    login(
                                                        &auth,
                                                        user.id,
                                                        user.username,
                                                        access_token.to_string(),
                                                        user.is_admin,
                                                    );
                                                    // Clean up URL and session storage
                                                    let _ = location.set_search("");
                                                    session_storage_remove(PKCE_VERIFIER_KEY);
                                                    session_storage_remove(OAUTH_REDIRECT_STATE_KEY);
                                                    nav_sig.get()("/repos", Default::default());
                                                }
                                            }
                                            _ => {
                                                set_error.set(Some("Failed to fetch user info".to_string()));
                                            }
                                        }
                                    }
                                }
                            }
                            Ok(resp) => {
                                let text = resp.text().await.unwrap_or_default();
                                set_error.set(Some(format!("Token exchange failed: {text}")));
                            }
                            Err(e) => {
                                set_error.set(Some(format!("Network error: {e}")));
                            }
                        }
                        set_loading.set(false);
                    } else {
                        set_error.set(Some("Missing PKCE verifier. Please try signing in again.".to_string()));
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

// ── PKCE Helper Functions ──

/// Generate a random PKCE code verifier (43-128 characters)
#[cfg(feature = "csr")]
fn generate_pkce_verifier() -> String {
    use js_sys::Math;
    let length = 64;
    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
    let mut verifier = String::with_capacity(length);
    for _ in 0..length {
        let idx = (Math::random() * chars.len() as f64) as usize;
        verifier.push(chars.chars().nth(idx).unwrap_or('A'));
    }
    verifier
}

/// Generate PKCE code challenge from verifier using SHA256 + base64url
#[cfg(feature = "csr")]
fn generate_pkce_challenge(verifier: &str) -> String {
    use sha2::{Digest, Sha256};
    use base64::Engine;
    
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let result = hasher.finalize();
    
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(result)
}

/// Generate a random state parameter for CSRF protection
#[cfg(feature = "csr")]
fn generate_random_state() -> String {
    use js_sys::Math;
    let length = 32;
    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut state = String::with_capacity(length);
    for _ in 0..length {
        let idx = (Math::random() * chars.len() as f64) as usize;
        state.push(chars.chars().nth(idx).unwrap_or('A'));
    }
    state
}

/// Get the current page origin (protocol + host)
#[cfg(feature = "csr")]
fn get_current_origin() -> String {
    let window = web_sys::window().expect("browser window available");
    let location = window.location();
    let protocol = location.protocol().unwrap_or_default();
    let host = location.host().unwrap_or_default();
    format!("{}//{}", protocol, host)
}

/// Parse URL search parameters into a HashMap
#[cfg(feature = "csr")]
fn parse_url_params(search: &str) -> std::collections::HashMap<String, String> {
    let mut params = std::collections::HashMap::new();
    let search = search.trim_start_matches('?');
    for pair in search.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            params.insert(
                key.to_string(),
                urlencoding::decode(value).unwrap_or_default().to_string(),
            );
        }
    }
    params
}

/// Session storage helper functions
#[cfg(feature = "csr")]
fn session_storage_get(key: &str) -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.session_storage().ok()??;
    storage.get_item(key).ok()?
}

#[cfg(feature = "csr")]
fn session_storage_set(key: &str, value: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.session_storage() {
            let _ = storage.set_item(key, value);
        }
    }
}

#[cfg(feature = "csr")]
fn session_storage_remove(key: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.session_storage() {
            let _ = storage.remove_item(key);
        }
    }
}
