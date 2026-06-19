#![forbid(unsafe_code)]

use leptos::prelude::*;

#[component]
pub fn ErrorBanner(
    message: impl Fn() -> String + Send + Sync + 'static,
    #[prop(optional)] on_dismiss: Option<Callback<()>>,
) -> impl IntoView {
    let (on_dismiss_sig, _) = signal(on_dismiss);
    view! {
        <div
            class="p-4 bg-red-50 dark:bg-red-900/20 border-l-4 border-red-500 dark:border-red-400 rounded-r-none flex items-start justify-between gap-2"
            role="alert"
            aria-live="assertive"
        >
            <p class="text-sm font-mono text-red-700 dark:text-red-400">{message}</p>
            <Show when=move || on_dismiss_sig.get().is_some()>
                <button
                    class="text-red-500 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300 text-lg leading-none shrink-0"
                    on:click=move |_| {
                        if let Some(cb) = on_dismiss_sig.get() {
                            cb.run(());
                        }
                    }
                    aria-label="Dismiss error"
                >
                    "\u{00d7}"
                </button>
            </Show>
        </div>
    }
}
