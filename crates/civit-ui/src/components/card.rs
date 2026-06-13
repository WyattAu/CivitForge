#![forbid(unsafe_code)]

use leptos::prelude::*;

#[component]
pub fn Card(
    #[prop(optional)] title: String,
    #[prop(optional)] description: String,
    #[prop(optional)] class: String,
    children: ChildrenFn,
) -> impl IntoView {
    let base = "bg-white dark:bg-gray-800 rounded-none shadow-sm \
                border-2 border-gray-200 dark:border-gray-700";
    let full_class = if class.is_empty() {
        base.to_string()
    } else {
        format!("{base} {class}")
    };

    view! {
        <div class=full_class>
            {(!title.is_empty() || !description.is_empty()).then(|| view! {
                <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                    {(!title.is_empty()).then(|| view! {
                        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{title}</h2>
                    })}
                    {(!description.is_empty()).then(|| view! {
                        <p class="mt-1 text-sm text-gray-600 dark:text-gray-400">{description}</p>
                    })}
                </div>
            })}
            <div class="px-6 py-4">
                {children()}
            </div>
        </div>
    }
}
