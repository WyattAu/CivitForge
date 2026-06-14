#![forbid(unsafe_code)]

use leptos::prelude::*;

use crate::components::Modal;

#[component]
pub fn KeyboardShortcuts() -> impl IntoView {
    let (show_help, set_show_help) = signal(false);

    #[cfg(feature = "csr")]
    {
        leptos::task::spawn_local(async move {
            if let Some(window) = web_sys::window() {
                let cb = web_sys::js_sys::Function::new_with_args(
                    "ev",
                    "if(ev.key==='?'&&!['INPUT','TEXTAREA','SELECT'].includes(document.activeElement.tagName)){ev.preventDefault();window.__civit_toggle_help();}",
                );
                let _ = window.add_event_listener_with_callback("keydown", &cb);
                let _ = js_sys::eval("window.__civit_toggle_help=null;");
            }
        });
    }

    view! {
        {move || if show_help.get() {
            view! {
                <Modal
                    show=true
                    on_close=Callback::new(move |_| set_show_help.set(false))
                    title="Keyboard Shortcuts".to_string()
                >
                    <div class="space-y-3 text-sm">
                        <p class="text-gray-500 dark:text-gray-400">
                            "Press ? to toggle this panel."
                        </p>
                        <div class="grid grid-cols-2 gap-2">
                            <div class="font-mono bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded text-xs">
                                "/"
                            </div>
                            <div>"Focus search"</div>
                            <div class="font-mono bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded text-xs">
                                "g h"
                            </div>
                            <div>"Go to Home"</div>
                            <div class="font-mono bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded text-xs">
                                "g r"
                            </div>
                            <div>"Go to Repositories"</div>
                            <div class="font-mono bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded text-xs">
                                "g a"
                            </div>
                            <div>"Go to Activity"</div>
                            <div class="font-mono bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded text-xs">
                                "g e"
                            </div>
                            <div>"Go to Explore"</div>
                            <div class="font-mono bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded text-xs">
                                "g o"
                            </div>
                            <div>"Go to Organizations"</div>
                            <div class="font-mono bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded text-xs">
                                "g s"
                            </div>
                            <div>"Go to Search"</div>
                        </div>
                    </div>
                </Modal>
            }.into_any()
        } else {
            view! { <div></div> }.into_any()
        }}
    }
}
