//! Debug panel for development.
//! Toggle with Ctrl+Shift+D. Shows captured errors.

use crate::error_capture::{
    clear_captured_errors, error_count, get_captured_errors, sync_js_errors,
};
use leptos::*;

/// Debug overlay panel (only compiled with debug-panel feature)
#[component]
pub fn DebugPanel() -> impl IntoView {
    let (visible, set_visible) = create_signal(false);
    let (errors, set_errors) = create_signal(Vec::new());

    // Toggle on Ctrl+Shift+D
    leptos::on_mount(move |_| {
        let set_vis = set_visible;
        let _ = leptos::window_event_listener("keydown", move |ev: web_sys::KeyboardEvent| {
            if ev.ctrl_key() && ev.shift_key() && ev.key() == "D" {
                ev.prevent_default();
                set_vis.update(|v| *v = !*v);
                // Sync errors from JS
                sync_js_errors();
                let errs = get_captured_errors();
                set_errors.set(errs);
            }
        });
    });

    view! {
        // Floating button
        <div
            class="fixed bottom-4 right-4 z-50"
            class:hidden=move || !visible.get()
        >
            <div class="bg-gray-900 border-2 border-green-500 rounded-none shadow-2xl w-96 max-h-[80vh] overflow-hidden font-mono">
                <div class="flex items-center justify-between p-3 bg-green-900 border-b-2 border-green-500">
                    <h2 class="text-green-400 text-sm font-bold">
                        "Debug Panel"
                        <span class="ml-2 px-2 py-0.5 bg-red-600 text-white text-xs rounded-sm">
                            {move || error_count()}
                        </span>
                    </h2>
                    <div class="flex gap-2">
                        <button
                            class="px-2 py-1 text-xs bg-gray-700 text-gray-300 hover:bg-gray-600 rounded-none"
                            on:click=move |_| {
                                sync_js_errors();
                                set_errors.set(get_captured_errors());
                            }
                        >
                            "Refresh"
                        </button>
                        <button
                            class="px-2 py-1 text-xs bg-red-700 text-white hover:bg-red-600 rounded-none"
                            on:click=move |_| {
                                clear_captured_errors();
                                set_errors.set(vec![]);
                            }
                        >
                            "Clear"
                        </button>
                        <button
                            class="px-2 py-1 text-xs bg-gray-700 text-gray-300 hover:bg-gray-600 rounded-none"
                            on:click=move |_| set_visible.set(false)
                        >
                            "X"
                        </button>
                    </div>
                </div>
                <div class="overflow-y-auto max-h-[60vh] p-2">
                    <For
                        each=move || errors.get()
                        key=|e| e.timestamp.clone()
                        let:error
                    >
                        <div class="mb-2 p-2 bg-gray-800 border border-gray-700 rounded-none text-xs">
                            <div class="flex items-center gap-2 mb-1">
                                <span class="px-1.5 py-0.5 rounded-sm text-white text-[10px]"
                                    class=("bg-red-600" => error.source == "unhandled" || error.source == "unhandled_promise" || error.source == "console")
                                    class=("bg-yellow-600" => error.source == "console_warn")
                                    class=("bg-blue-600" => error.source != "unhandled" && error.source != "unhandled_promise" && error.source != "console" && error.source != "console_warn")
                                >
                                    {error.source.clone()}
                                </span>
                                <span class="text-gray-500 truncate flex-1">
                                    {error.url.clone()}
                                </span>
                            </div>
                            <p class="text-red-300 break-all">{error.message.clone()}</p>
                            {error.stack.as_ref().map(|s| view! {
                                <details class="mt-1">
                                    <summary class="text-gray-500 cursor-pointer text-[10px]">"Stack trace"</summary>
                                    <pre class="text-gray-600 text-[10px] mt-1 whitespace-pre-wrap break-all">{s.clone()}</pre>
                                </details>
                            }).into_view()}
                        </div>
                    </For>
                </div>
            </div>
        </div>

        // Always-visible toggle badge (bottom-right)
        <div
            class="fixed bottom-4 right-4 z-40 cursor-pointer select-none"
            class:hidden=move || visible.get()
            on:click=move |_| {
                sync_js_errors();
                set_errors.set(get_captured_errors());
                set_visible.set(true);
            }
        >
            <div class="w-8 h-8 bg-green-600 text-white rounded-full flex items-center justify-center text-xs font-bold shadow-lg hover:bg-green-500 transition-colors"
                class:hidden=move || error_count() == 0
            >
                {move || error_count()}
            </div>
            <div class="w-3 h-3 bg-green-500 rounded-full shadow-lg hover:bg-green-400 transition-colors"
                class:hidden=move || error_count() > 0
            >
            </div>
        </div>
    }
}
