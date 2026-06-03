#![forbid(unsafe_code)]

use leptos::prelude::*;

#[component]
pub fn Modal(
    show: bool,
    #[prop(optional)] title: String,
    #[prop(optional)] on_close: Option<Callback<()>>,
    children: ChildrenFn,
) -> impl IntoView {
    let (on_close_sig, _) = signal(on_close);

    let close = move |_| {
        if let Some(cb) = on_close_sig.get() {
            cb.run(());
        }
    };

    view! {
        <Show when=move || show fallback=|| view! { <div class="hidden"></div> }>
            <div class="fixed inset-0 z-50 flex items-center justify-center">
                <div class="fixed inset-0 bg-black/50" on:click=close></div>
                <div role="dialog" aria-modal="true" class="relative bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-lg w-full mx-4 p-6">
                    <div class="flex items-center justify-between mb-4">
                        {(!title.is_empty()).then(|| view! {
                            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
                                {title.clone()}
                            </h3>
                        })}
                        <button
                            class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 text-xl leading-none"
                            on:click=close
                            aria-label="Close"
                        >
                            "\u{00d7}"
                        </button>
                    </div>
                    {children()}
                </div>
            </div>
        </Show>
    }
}
