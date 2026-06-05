#![forbid(unsafe_code)]

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthState {
    pub is_authenticated: bool,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub token: Option<String>,
}

const STORAGE_KEY: &str = "civitforge_token";

fn local_storage_get(key: &str) -> Option<String> {
    #[cfg(feature = "csr")]
    {
        let window = web_sys::window()?;
        let storage = window.local_storage().ok()??;
        storage.get_item(key).ok()?
    }
    #[cfg(not(feature = "csr"))]
    {
        let _ = key;
        None
    }
}

fn local_storage_set(key: &str, value: &str) {
    #[cfg(feature = "csr")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item(key, value);
            }
        }
    }
    #[cfg(not(feature = "csr"))]
    {
        let _ = key;
        let _ = value;
    }
}

fn local_storage_remove(key: &str) {
    #[cfg(feature = "csr")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.remove_item(key);
            }
        }
    }
    #[cfg(not(feature = "csr"))]
    {
        let _ = key;
    }
}

pub fn provide_auth_context() {
    let auth = signal(AuthState::default());
    provide_context(auth);

    #[cfg(feature = "csr")]
    leptos::task::spawn_local(async move {
        if let Some(token) = local_storage_get(STORAGE_KEY) {
            let client = crate::api::client::ApiClient::new(Some(token.clone()));
            if let Ok(resp) = client.get("/auth/me").await {
                if resp.status().is_success() {
                    if let Ok(user) = resp.json::<crate::api::types::AuthUser>().await {
                        auth.1.update(|state| {
                            state.is_authenticated = true;
                            state.user_id = Some(user.id);
                            state.username = Some(user.username);
                            state.token = Some(token);
                        });
                    }
                }
            }
        }
    });
}

pub fn use_auth() -> (ReadSignal<AuthState>, WriteSignal<AuthState>) {
    expect_context::<(ReadSignal<AuthState>, WriteSignal<AuthState>)>()
}

pub fn login(
    auth: &(ReadSignal<AuthState>, WriteSignal<AuthState>),
    user_id: String,
    username: String,
    token: String,
) {
    local_storage_set(STORAGE_KEY, &token);
    auth.1.update(|state| {
        state.is_authenticated = true;
        state.user_id = Some(user_id);
        state.username = Some(username);
        state.token = Some(token);
    });
}

pub fn logout(auth: &(ReadSignal<AuthState>, WriteSignal<AuthState>)) {
    local_storage_remove(STORAGE_KEY);
    auth.1.update(|state| {
        state.is_authenticated = false;
        state.user_id = None;
        state.username = None;
        state.token = None;
    });
}
