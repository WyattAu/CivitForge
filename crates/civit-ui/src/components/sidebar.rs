#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::Avatar;
use crate::state::auth::use_auth;

#[derive(Clone)]
struct NavItem {
    href: String,
    label: String,
    icon: &'static str,
}

#[component]
pub fn Sidebar() -> impl IntoView {
    let (mobile_open, set_mobile_open) = signal(false);
    let auth = use_auth();

    let main_nav_items = vec![
        NavItem {
            href: "/".into(),
            label: "Home".into(),
            icon: "\u{1f3e0}\u{fe0f}",
        },
        NavItem {
            href: "/repos".into(),
            label: "Repositories".into(),
            icon: "\u{1f4c1}",
        },
        NavItem {
            href: "/activity".into(),
            label: "Activity".into(),
            icon: "activity",
        },
        NavItem {
            href: "/explore".into(),
            label: "Explore".into(),
            icon: "\u{1f50d}",
        },
        NavItem {
            href: "/orgs".into(),
            label: "Organizations".into(),
            icon: "\u{1f3eb}",
        },
        NavItem {
            href: "/search".into(),
            label: "Search".into(),
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
            <div class="h-16 flex items-center px-6 border-b border-gray-200 dark:border-gray-700 shrink-0">
                <A href="/">
                    <span class="text-xl font-bold text-blue-600 dark:text-blue-400">"CivitForge"</span>
                </A>
            </div>

            <nav class="px-3 py-4 space-y-1 flex-1 overflow-y-auto">
                <For each=move || main_nav_sig.get() key=|item| item.href.clone() let:item>
                    {
                        let label = item.label.clone();
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
                                {label}
                            </A>
                        }
                    }
                </For>

                <Show when=move || auth.0.with(|a| a.is_authenticated) fallback=|| view! { <div class="hidden"></div> }>
                    <div class="pt-3 mt-3 border-t border-gray-200 dark:border-gray-700">
                        <div class="px-3 py-1 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                            "Create"
                        </div>
                        <A href="/new-repo" attr:class=link_class>
                            <span class="mr-2">"\u{2795}"</span>
                            "New Repo"
                        </A>
                    </div>
                </Show>
            </nav>

            <div class="border-t border-gray-200 dark:border-gray-700 p-3 shrink-0">
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
                        <div class="block px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-100 hover:text-gray-900 dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-white">"Sign In"</div>
                    </A>
                }>
                    <div class="space-y-1">
                        <div class="flex items-center gap-2 px-3 py-2">
                            <Avatar name=username() size=28 />
                            <span class="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                                {username}
                            </span>
                        </div>
                        <A href="/settings" attr:class=link_class>
                            <span class="mr-2">"\u{2699}\u{fe0f}"</span>
                            "Settings"
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
                            "\u{1f6aa} Sign Out"
                        </a>
                    </div>
                </Show>
            </div>
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
