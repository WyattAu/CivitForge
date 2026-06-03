#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;

use crate::state::auth::{logout, use_auth};

#[derive(Clone)]
struct NavItem {
    href: String,
    label: String,
}

#[component]
pub fn Sidebar() -> impl IntoView {
    let (mobile_open, set_mobile_open) = signal(false);
    let auth = use_auth();

    let nav_items = vec![
        NavItem {
            href: "/".into(),
            label: "Dashboard".into(),
        },
        NavItem {
            href: "/explore".into(),
            label: "Explore".into(),
        },
        NavItem {
            href: "/repos".into(),
            label: "Repositories".into(),
        },
        NavItem {
            href: "/orgs".into(),
            label: "Organizations".into(),
        },
        NavItem {
            href: "/settings".into(),
            label: "Settings".into(),
        },
    ];

    let (nav_sig, _) = signal(nav_items);

    let link_class: &'static str = "block px-3 py-2 rounded-md text-sm font-medium \
                      text-gray-700 hover:bg-gray-100 hover:text-gray-900 \
                      dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-white";

    let handle_logout = move |_| {
        logout(&auth);
    };

    view! {
        <aside
            class="fixed inset-y-0 left-0 z-40 w-64 \
                   bg-white dark:bg-gray-800 \
                   border-r border-gray-200 dark:border-gray-700 \
                   transition-transform duration-200 ease-in-out \
                   -translate-x-full lg:translate-x-0"
            class:translate-x-0=move || mobile_open.get()
        >
            <div class="h-16 flex items-center px-6 border-b border-gray-200 dark:border-gray-700">
                <A href="/">
                    <span class="text-xl font-bold text-blue-600 dark:text-blue-400">"CivitForge"</span>
                </A>
            </div>
            <nav class="px-3 py-4 space-y-1">
                <For each=move || nav_sig.get() key=|item| item.href.clone() let:item>
                    {
                        view! {
                            <A href=item.href.clone()>
                                <span class=link_class>{item.label.clone()}</span>
                            </A>
                        }
                    }
                </For>
            </nav>

            <div class="absolute bottom-0 left-0 right-0 p-4 border-t border-gray-200 dark:border-gray-700">
                <Show when=move || auth.0.with(|a| a.is_authenticated) fallback=|| view! {
                    <A href="/login">
                        <div class="block px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-100 hover:text-gray-900 dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-white">"Sign In"</div>
                    </A>
                }>
                    <div class="flex items-center justify-between">
                        <div class="px-3 py-2">
                            <span class="text-sm font-medium text-gray-900 dark:text-gray-100">
                                {move || auth.0.with(|a| a.username.clone().unwrap_or_default())}
                            </span>
                        </div>
                        <button
                            class="px-3 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 hover:text-gray-900 dark:text-gray-300 dark:hover:bg-gray-700 dark:hover:text-white rounded-md"
                            on:click=handle_logout
                        >
                            "Sign Out"
                        </button>
                    </div>
                </Show>
            </div>
        </aside>

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
