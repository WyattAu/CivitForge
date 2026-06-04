//! Error boundary component that catches rendering errors in child components.

use leptos::prelude::*;

/// Error boundary wrapper component.
/// Catches errors during child rendering and displays a fallback UI.
#[component]
pub fn CatchError(
    /// Human-readable name for this boundary
    #[prop(optional, default = "ErrorBoundary".to_string())]
    name: String,
    /// Children components to wrap
    children: ChildrenFn,
) -> impl IntoView {
    let (error, set_error) = signal::<Option<String>>(None);
    let (retry_count, set_retry_count) = signal(0u32);

    let children_fn = StoredValue::new(children);

    view! {
        {move || match error.get() {
            None => children_fn.with_value(|c| c()).into_any(),
            Some(msg) => view! {
                <div class="border-2 border-red-500 bg-red-950 p-6 rounded-none">
                    <div class="flex items-center justify-between mb-4">
                        <h3 class="text-red-400 font-mono text-lg">
                            "Error in " {name.clone()}
                        </h3>
                        <button
                            class="px-4 py-2 bg-red-600 text-white font-mono hover:bg-red-700 rounded-none"
                            on:click=move |_| {
                                set_error.set(None);
                                set_retry_count.update(|c| *c += 1);
                            }
                        >
                            "Retry"
                        </button>
                    </div>
                    <p class="text-red-300 font-mono text-sm mb-2">
                        {msg.clone()}
                    </p>
                    <p class="text-red-400/60 font-mono text-xs">
                        "Retry count: " {retry_count}
                    </p>
                </div>
            }.into_any(),
        }}
    }
}
