#![forbid(unsafe_code)]

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthState {
    pub is_authenticated: bool,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub token: Option<String>,
}

pub fn provide_auth_context() {
    let auth = signal(AuthState::default());
    provide_context(auth);
}

pub fn use_auth() -> (ReadSignal<AuthState>, WriteSignal<AuthState>) {
    expect_context::<(ReadSignal<AuthState>, WriteSignal<AuthState>)>()
}

pub fn login(
    auth: &(ReadSignal<AuthState>, WriteSignal<AuthState>),
    user_id: i64,
    username: String,
    token: String,
) {
    auth.1.update(|state| {
        state.is_authenticated = true;
        state.user_id = Some(user_id);
        state.username = Some(username);
        state.token = Some(token);
    });
}

pub fn logout(auth: &(ReadSignal<AuthState>, WriteSignal<AuthState>)) {
    auth.1.update(|state| {
        state.is_authenticated = false;
        state.user_id = None;
        state.username = None;
        state.token = None;
    });
}
