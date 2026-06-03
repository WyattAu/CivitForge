#![forbid(unsafe_code)]

use leptos::prelude::*;

#[component]
pub fn FormField(
    label: &'static str,
    #[prop(optional)] input_type: crate::components::InputType,
    #[prop(optional)] name: &'static str,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] required: bool,
    #[prop(optional)] id: &'static str,
    #[prop(optional)] error: String,
    #[prop(optional)] value: String,
) -> impl IntoView {
    let (error_sig, _) = signal(error);
    let has_error = move || error_sig.with(|e| !e.is_empty());
    let input_id = if id.is_empty() { name } else { id };
    let input_classes = if has_error() {
        "w-full px-3 py-2 border-2 border-red-500 dark:border-red-400 rounded-none \
         dark:bg-gray-700 dark:text-gray-100 text-sm \
         focus:outline-none focus-visible:ring-2 focus-visible:ring-red-500 \
         focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900 \
         placeholder-gray-400 dark:placeholder-gray-500"
    } else {
        "w-full px-3 py-2 border-2 border-gray-300 dark:border-gray-600 rounded-none \
         dark:bg-gray-700 dark:text-gray-100 text-sm \
         focus:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 \
         focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900 \
         focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500"
    };

    let tpe = match input_type {
        crate::components::InputType::Email => "email",
        crate::components::InputType::Password => "password",
        _ => "text",
    };

    view! {
        <div>
            <label for=input_id class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1 font-mono">
                {label}
                {required.then(|| view! { <span class="text-red-500 ml-0.5">"*"</span> })}
            </label>
            <input
                type=tpe
                id=input_id
                name=name
                class=input_classes
                placeholder=placeholder
                value=value
                required=required
                aria-label=label
                aria_invalid=has_error()
            />
            <Show when=move || has_error()>
                <p class="mt-1 text-sm text-red-600 dark:text-red-400 font-mono">
                    {move || error_sig.get()}
                </p>
            </Show>
        </div>
    }
}
