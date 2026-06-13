#![forbid(unsafe_code)]

use leptos::prelude::*;

use crate::i18n::{self, LOCALES};

#[component]
pub fn Footer() -> impl IntoView {
    let (current_locale, set_current_locale) = signal(i18n::get_locale());

    view! {
        <footer class="bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 mt-auto" role="contentinfo">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
                <div class="flex flex-col md:flex-row items-center justify-between gap-4">
                    // Left: Copyright + version
                    <div class="text-sm text-gray-500 dark:text-gray-400 text-center md:text-left">
                        <span>"Copyright "</span>
                        <span class="font-medium text-gray-700 dark:text-gray-300">"CivitForge"</span>
                        <span class="mx-1">"\u{00a9}"</span>
                        <span>{move || {
                            let year = js_sys::Date::new_0().get_full_year();
                            year.to_string()
                        }}</span>
                        <span class="ml-2 px-1.5 py-0.5 text-[10px] font-mono rounded bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400">
                            "v0.1.0"
                        </span>
                    </div>

                    // Center: Links
                    <div class="flex items-center gap-4 text-sm">
                        <a
                            href="/docs"
                            class="text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 transition-colors"
                        >
                            "Documentation"
                        </a>
                        <a
                            href="/api/v1"
                            class="text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 transition-colors"
                        >
                            "API"
                        </a>
                        <a
                            href="/status"
                            class="text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 transition-colors"
                        >
                            "Status"
                        </a>
                    </div>

                    // Right: Language selector + theme toggle
                    <div class="flex items-center gap-3">
                        <select
                            aria-label="Select language"
                            class="px-2 py-1 rounded text-xs \
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
                        <div
                            data-theme-toggle=""
                            class="px-2 py-1 rounded text-xs cursor-pointer select-none \
                                   bg-gray-100 dark:bg-gray-700 \
                                   text-gray-700 dark:text-gray-300 \
                                   border border-gray-200 dark:border-gray-600 \
                                   hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                            role="button"
                            tabindex="0"
                            aria-label="Toggle dark mode"
                        >
                            <span data-theme-toggle-icon="">"\u{1f319}"</span>
                        </div>
                    </div>
                </div>
            </div>
        </footer>
    }
}
