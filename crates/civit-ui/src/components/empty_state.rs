#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn EmptyState(
    icon: AnyView,
    title: String,
    description: String,
    #[prop(optional)] action_text: Option<String>,
    #[prop(optional)] action_href: Option<String>,
) -> impl IntoView {
    view! {
        <div class="text-center py-12">
            <div class="mx-auto w-16 h-16 text-gray-400 dark:text-gray-500 mb-4">{icon}</div>
            <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-2">{title}</h2>
            <p class="text-gray-500 dark:text-gray-400 mb-6 max-w-md mx-auto">{description}</p>
            {move || {
                action_text.clone().map(|text| {
                    let href = action_href.clone().unwrap_or_default();
                    view! {
                        <A href=href>
                            <span class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900 bg-blue-600 hover:bg-blue-700 text-white dark:bg-blue-500 dark:hover:bg-blue-600">
                                {text}
                            </span>
                        </A>
                    }
                })
            }}
        </div>
    }
}
