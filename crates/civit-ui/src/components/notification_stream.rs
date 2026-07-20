#![forbid(unsafe_code)]

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[derive(Clone, PartialEq, Debug, serde::Deserialize)]
pub struct NotificationEvent {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub body: Option<String>,
    pub repo_name: Option<String>,
    pub created_at: String,
}

fn get_base_url() -> String {
    if let Some(window) = web_sys::window() {
        let api_url = js_sys::eval(
            "typeof window !== 'undefined' && window.__CIVIT_API_URL ? window.__CIVIT_API_URL : ''",
        );
        if let Ok(val) = api_url
            && !val.is_undefined()
            && val.is_string()
        {
            let url: String = val.as_string().unwrap_or_default();
            if !url.is_empty() {
                return url;
            }
        }

        let origin = window.location().origin().unwrap_or_default();
        if !origin.is_empty() {
            return origin;
        }
    }
    "http://127.0.0.1:9091".to_string()
}

#[component]
pub fn NotificationBell() -> impl IntoView {
    let (notifications, set_notifications) = signal(Vec::<NotificationEvent>::new());
    let (unread_count, set_unread_count) = signal(0u32);
    let (dropdown_open, set_dropdown_open) = signal(false);
    let (connected, set_connected) = signal(false);
    let (es_resource, set_es) = signal(None::<web_sys::EventSource>);
    let (reconnecting, set_reconnecting) = signal(false);

    let connect_sse = {
        let set_connected = set_connected.clone();
        let set_unread_count = set_unread_count.clone();
        let set_notifications = set_notifications.clone();
        let set_es = set_es.clone();
        move || {
            if let Some(old_es) = es_resource.get_untracked() {
                let _ = old_es.close();
            }
            set_es.set(None);

            let base_url = get_base_url();
            let token = window_local_storage()
                .and_then(|s| s.get_item("auth_token").ok())
                .flatten();

            let Some(token) = token else { return };

            let url = format!("{base_url}/api/v1/notifications/stream?token={token}");

            leptos::task::spawn_local(async move {
                let Ok(event_source) = web_sys::EventSource::new(&url) else {
                    set_reconnecting.set(true);
                    TimeoutFuture::new(5000).await;
                    set_reconnecting.set(false);
                    return;
                };

                set_es.set(Some(event_source.clone()));
                set_connected.set(true);

                let on_message = Closure::wrap(Box::new(move |ev: web_sys::MessageEvent| {
                    if let Some(data) = ev.data().as_string() {
                        if let Ok(event) = serde_json::from_str::<NotificationEvent>(&data) {
                            set_unread_count.update(|c| *c += 1);
                            set_notifications.update(|n| {
                                n.insert(0, event);
                                n.truncate(50);
                            });
                        }
                    }
                }) as Box<dyn FnMut(_)>);

                let on_error = Closure::wrap(Box::new(move |_: web_sys::Event| {
                    set_reconnecting.set(true);
                    set_connected.set(false);
                    leptos::task::spawn_local(async move {
                        TimeoutFuture::new(5000).await;
                        set_reconnecting.set(false);
                    });
                }) as Box<dyn FnMut(_)>);

                let _ = event_source.add_event_listener_with_callback(
                    "message",
                    on_message.as_ref().unchecked_ref(),
                );
                let _ = event_source.add_event_listener_with_callback(
                    "error",
                    on_error.as_ref().unchecked_ref(),
                );

                let _ = on_message.into_js_value();
                let _ = on_error.into_js_value();
            });
        }
    };

    Effect::new(move |_| {
        let is_connected = connected.get();
        let is_reconnecting = reconnecting.get();
        if !is_connected && !is_reconnecting {
            connect_sse();
        }
    });

    let mark_all_read = move |_| {
        set_unread_count.set(0);
        set_dropdown_open.set(false);
    };

    let toggle_dropdown = move |_: leptos::ev::MouseEvent| {
        set_dropdown_open.update(|o| *o = !*o);
    };

    view! {
        <div class="relative">
            <button
                class="relative p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 transition-colors"
                on:click=toggle_dropdown
                aria-label="Notifications"
            >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"/>
                </svg>
                {move || if unread_count.get() > 0 {
                    view! {
                        <span class="absolute -top-1 -right-1 inline-flex items-center justify-center px-1.5 py-0.5 text-xs font-bold text-white bg-red-600 rounded-full min-w-[18px] h-[18px]">
                            {move || {
                                let c = unread_count.get();
                                if c > 99 { "99+".to_string() } else { c.to_string() }
                            }}
                        </span>
                    }.into_any()
                } else {
                    view! { <div class="hidden"></div> }.into_any()
                }}
                {move || if !connected.get() {
                    view! {
                        <span class="absolute top-1 right-1 w-2 h-2 bg-yellow-500 rounded-full"></span>
                    }.into_any()
                } else {
                    view! { <div class="hidden"></div> }.into_any()
                }}
            </button>

            <Show when=move || dropdown_open.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="absolute right-0 mt-2 w-80 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg z-50 max-h-96 overflow-hidden">
                    <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-700">
                        <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100">"Notifications"</h3>
                        <button
                            class="text-xs text-blue-600 dark:text-blue-400 hover:underline"
                            on:click=mark_all_read
                        >
                            "Mark all read"
                        </button>
                    </div>

                    <div class="overflow-y-auto max-h-80">
                        <Show
                            when=move || !notifications.get().is_empty()
                            fallback=|| view! {
                                <div class="px-4 py-8 text-center text-sm text-gray-500 dark:text-gray-400">
                                    "No notifications yet"
                                </div>
                            }
                        >
                            <For
                                each=move || notifications.get()
                                key=|n| n.id.clone()
                                let:notif
                            >
                                <div class="px-4 py-3 border-b border-gray-100 dark:border-gray-700/50 hover:bg-gray-50 dark:hover:bg-gray-750 transition-colors">
                                    <div class="flex items-start gap-3">
                                        <div class="shrink-0 mt-0.5">
                                            {move || match notif.kind.as_str() {
                                                "issue" => view! {
                                                    <svg class="w-4 h-4 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                                                    </svg>
                                                }.into_any(),
                                                "pull_request" | "pr_merged" => view! {
                                                    <svg class="w-4 h-4 text-purple-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 3v12M18 9a3 3 0 100-6 3 3 0 000 6zM6 21a3 3 0 100-6 3 3 0 000 6zM9 12l3-3 3 3"/>
                                                    </svg>
                                                }.into_any(),
                                                "pipeline" => view! {
                                                    <svg class="w-4 h-4 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>
                                                    </svg>
                                                }.into_any(),
                                                _ => view! {
                                                    <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"/>
                                                    </svg>
                                                }.into_any(),
                                            }}
                                        </div>
                                        <div class="min-w-0 flex-1">
                                            <p class="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                                                {notif.title}
                                            </p>
                                            {move || notif.body.as_ref().map(|b| {
                                                let display = if b.len() > 80 { format!("{}...", &b[..80]) } else { b.clone() };
                                                view! {
                                                    <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5 line-clamp-2">{display}</p>
                                                }
                                            })}
                                            <div class="flex items-center gap-2 mt-1 text-xs text-gray-400 dark:text-gray-500">
                                                {move || notif.repo_name.as_ref().map(|r| {
                                                    view! { <span class="font-mono">{r.clone()}</span> }
                                                })}
                                                <span>{notif.created_at.clone()}</span>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </For>
                        </Show>
                    </div>
                </div>
            </Show>
        </div>
    }
}

fn window_local_storage() -> Option<web_sys::Storage> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
}
