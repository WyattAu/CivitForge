#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;

use crate::api::client::ApiClient;
use crate::components::Avatar;
use crate::i18n::{self, t, LOCALES};
use crate::state::auth::use_auth;

#[derive(Clone)]
struct NavItem {
    href: String,
    label: String,
    icon: &'static str,
}

#[derive(Clone, serde::Deserialize)]
struct SiteSettingsCache {
    site_name: String,
    logo_url: String,
    footer_text: String,
}

#[component]
pub fn Sidebar() -> impl IntoView {
    let (mobile_open, set_mobile_open) = signal(false);
    let auth = use_auth();
    let (current_locale, set_current_locale) = signal(i18n::get_locale());
    let (site_settings, set_site_settings) = signal(None::<SiteSettingsCache>);

    // Fetch site settings for sidebar display
    leptos::task::spawn_local(async move {
        let client = ApiClient::new(None);
        if let Ok(resp) = client.get("/admin/settings").await {
            if resp.status().is_success() {
                if let Ok(data) = resp.json::<SiteSettingsCache>().await {
                    set_site_settings.set(Some(data));
                }
            }
        }
    });

    let main_nav_items = vec![
        NavItem {
            href: "/".into(),
            label: "nav.home".to_string(),
            icon: "\u{1f3e0}\u{fe0f}",
        },
        NavItem {
            href: "/repos".into(),
            label: "nav.repos".to_string(),
            icon: "\u{1f4c1}",
        },
        NavItem {
            href: "/activity".into(),
            label: "nav.activity".to_string(),
            icon: "activity",
        },
        NavItem {
            href: "/explore".into(),
            label: "nav.explore".to_string(),
            icon: "\u{1f50d}",
        },
        NavItem {
            href: "/orgs".into(),
            label: "nav.orgs".to_string(),
            icon: "\u{1f3eb}",
        },
        NavItem {
            href: "/search".into(),
            label: "nav.search".to_string(),
            icon: "svg",
        },
    ];

    let (main_nav_sig, _) = signal(main_nav_items);

    let link_class = "block px-3 py-2 rounded-md text-sm font-medium \
                      text-gray-700 hover:bg-gray-100 hover:text-gray-900 \
                      dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-white";

    let username = move || auth.0.with(|a| a.username.clone().unwrap_or_default());

    view! {
        <aside
            class="fixed inset-y-0 left-0 z-40 w-64 \
                   bg-white dark:bg-gray-800 \
                   border-r border-gray-200 dark:border-gray-700 \
                   transition-transform duration-200 ease-in-out \
                   -translate-x-full lg:translate-x-0 flex flex-col"
            class:translate-x-0=move || mobile_open.get()
        >
            <header class="h-16 flex items-center px-6 border-b border-gray-200 dark:border-gray-700 shrink-0" role="banner">
                <A href="/">
                    {move || {
                        let s = site_settings.get();
                        let name = s.as_ref().map(|s| s.site_name.clone()).unwrap_or_else(|| "CivitForge".into());
                        let logo = s.and_then(|s| {
                            if s.logo_url.is_empty() { None } else { Some(s.logo_url) }
                        });
                        view! {
                            {if let Some(url) = logo {
                                view! { <img src=url class="h-7 w-7 mr-2 rounded" alt="Logo" /> }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }}
                            <span class="text-xl font-bold text-blue-600 dark:text-blue-400">{name}</span>
                        }
                    }}
                </A>
            </header>

            <nav class="px-3 py-4 space-y-1 flex-1 overflow-y-auto">
                <For each=move || main_nav_sig.get() key=|item| item.href.clone() let:item>
                    {
                        let label_key = item.label.clone();
                        let href = item.href.clone();
                        view! {
                            <A href=href attr:class=link_class>
                                <Show when=move || item.icon == "svg" fallback=move || view! {
                                    <Show when=move || item.icon == "activity" fallback=move || view! {
                                        <span class="mr-2">{item.icon}</span>
                                    }>
                                        <svg class="w-4 h-4 mr-2 inline-block" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
                                        </svg>
                                    </Show>
                                }>
                                    <svg class="w-4 h-4 mr-2 inline-block" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
                                    </svg>
                                </Show>
                                {move || t(&label_key)}
                            </A>
                        }
                    }
                </For>

                <Show when=move || auth.0.with(|a| a.is_authenticated && a.username.as_deref() != Some("")) fallback=|| view! { <div class="hidden"></div> }>
                    <div class="pt-3 mt-3 border-t border-gray-200 dark:border-gray-700">
                        <div class="px-3 py-1 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                            {move || t("nav.create")}
                        </div>
                        <A href="/new-repo" attr:class=link_class>
                            <span class="mr-2">"\u{2795}"</span>
                            {move || t("nav.new_repo")}
                        </A>
                    </div>
                </Show>

                <Show when=move || auth.0.with(|a| a.is_authenticated && a.is_admin) fallback=|| view! { <div class="hidden"></div> }>
                    <div class="pt-3 mt-3 border-t border-gray-200 dark:border-gray-700">
                        <div class="px-3 py-1 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                            "Admin"
                        </div>
                        <A href="/admin" attr:class=link_class>
                            <span class="mr-2">"\u{2699}\u{fe0f}"</span>
                            "Admin Panel"
                        </A>
                        <A href="/admin/site-settings" attr:class=link_class>
                            <span class="mr-2">"\u{1f527}"</span>
                            "Site Settings"
                        </A>
                    </div>
                </Show>
            </nav>

            <div class="border-t border-gray-200 dark:border-gray-700 p-3 shrink-0">
                // Locale switcher
                <div class="mb-2">
                    <select
                        aria-label="Select language"
                        class="w-full px-2 py-1 rounded text-xs \
                               bg-gray-100 dark:bg-gray-700 \
                               text-gray-700 dark:text-gray-300 \
                               border border-gray-200 dark:border-gray-600"
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            i18n::save_locale_to_storage(&val);
                            set_current_locale.set(val);
                        }
                    >
                        {LOCALES.iter().map(|(code, name)| {
                            let code = *code;
                            let name = *name;
                            view! {
                                <option value=code selected=move || current_locale.get() == code>{name}</option>
                            }
                        }).collect_view()}
                    </select>
                </div>
                // Theme toggle — uses data-theme-toggle attribute with JS-attached
                // click handler to avoid Leptos on:click WebKit auto-fire bug
                <div
                    data-theme-toggle=""
                    class="block w-full px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-100 hover:text-gray-900 dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-white cursor-pointer select-none"
                    role="button"
                    tabindex="0"
                    aria-label="Toggle dark mode"
                >
                    <span data-theme-toggle-icon="">"\u{1f319}"</span>
                    " Toggle Theme"
                </div>
                <Show when=move || auth.0.with(|a| a.is_authenticated) fallback=|| view! {
                    <A href="/login">
                        <div class="block px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-100 hover:text-gray-900 dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-white">
                            {move || t("auth.sign_in")}
                        </div>
                    </A>
                }>
                    <div class="space-y-1">
                        <div class="flex items-center gap-2 px-3 py-2">
                            <Avatar name=username() size=28 />
                            <span class="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                                {username}
                            </span>
                        </div>
                        <A href="/profile" attr:class=link_class>
                            <span class="mr-2">"\u{1f464}"</span>
                            "Profile"
                        </A>
                        <A href="/settings" attr:class=link_class>
                            <span class="mr-2">"\u{2699}\u{fe0f}"</span>
                            {move || t("settings.title")}
                        </A>
                        // Sign out uses <a href> instead of <button on:click> to
                        // avoid WebKit auto-fire bug. Link triggers JS logout function.
                        <a
                            href="javascript:void(0)"
                            data-action-logout=""
                            class="block w-full text-left px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-100 hover:text-gray-900 dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-white cursor-pointer"
                            role="button"
                            aria-label="Sign out of CivitForge"
                        >
                            "\u{1f6aa} "
                            {move || t("auth.sign_out")}
                        </a>
                    </div>
                </Show>
            </div>

            // Footer text from site settings
            {move || {
                let footer = site_settings.get().and_then(|s| {
                    if s.footer_text.is_empty() { None } else { Some(s.footer_text) }
                });
                if let Some(text) = footer {
                    view! {
                        <div class="border-t border-gray-200 dark:border-gray-700 px-3 py-2 text-xs text-gray-400 dark:text-gray-500 shrink-0">
                            {text}
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </aside>

        <div
            class="lg:hidden fixed inset-0 z-30 bg-black/50 transition-opacity"
            class:opacity-0=move || !mobile_open.get()
            class:opacity-100=move || mobile_open.get()
            class:pointer-events-none=move || !mobile_open.get()
            class:pointer-events-auto=move || mobile_open.get()
            on:click=move |_| set_mobile_open.set(false)
        ></div>

        <button
            class="lg:hidden fixed top-4 left-4 z-50 p-2 rounded-md \
                   bg-white dark:bg-gray-800 shadow-md \
                   border border-gray-200 dark:border-gray-700"
            on:click=move |_| set_mobile_open.set(true)
            aria-label="Toggle sidebar"
        >
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M4 6h16M4 12h16M4 18h16"
                />
            </svg>
        </button>
    }
}
