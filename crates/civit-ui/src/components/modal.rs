#![forbid(unsafe_code)]

use leptos::prelude::*;

const FOCUSABLE_SEL: &str = "a[href], button:not([disabled]), input:not([disabled]), textarea:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex='-1'])";

#[component]
pub fn Modal(
    show: bool,
    #[prop(optional)] title: String,
    #[prop(optional)] on_close: Option<Callback<()>>,
    children: ChildrenFn,
) -> impl IntoView {
    let (on_close_sig, _) = signal(on_close);
    let title_clone = title.clone();

    let close = move |_| {
        if let Some(cb) = on_close_sig.get() {
            cb.run(());
        }
    };

    let close_key = on_close_sig;
    let has_title = !title.is_empty();

    Effect::new(move |_| {
        let is_open = show;
        if is_open {
            // Save the currently focused element so we can restore it on close
            let _ = js_sys::eval(
                "(function() {\
                    var el = document.activeElement;\
                    window.__modal_prev_focus = el;\
                })()",
            );
            // Focus the first focusable element inside the dialog after a tick
            let _ = js_sys::eval(
                "setTimeout(function() {\
                    var d = document.querySelector('[role=\"dialog\"]');\
                    if(!d) return;\
                    var els = d.querySelectorAll('a[href], button:not([disabled]), input:not([disabled]), textarea:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex=\"-1\"])');\
                    if(els.length > 0) els[0].focus();\
                }, 0)",
            );
        } else {
            // Restore focus to the trigger element when modal closes
            let _ = js_sys::eval(
                "(function() {\
                    var prev = window.__modal_prev_focus;\
                    if(prev && prev.focus) prev.focus();\
                    window.__modal_prev_focus = null;\
                })()",
            );
        }
    });

    let handle_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" {
            if let Some(cb) = close_key.get() {
                cb.run(());
            }
            return;
        }
        if ev.key() == "Tab" {
            // Focus trap: if Tab would escape the dialog, wrap focus instead
            let shift = ev.shift_key();
            let should_prevent = js_sys::eval(&format!(
                "(function(shift) {{\
                    var d = document.querySelector('[role=\"dialog\"]');\
                    if(!d) return false;\
                    var els = d.querySelectorAll('{FOCUSABLE_SEL}');\
                    if(els.length === 0) return true;\
                    var first = els[0];\
                    var last = els[els.length - 1];\
                    var focused = document.activeElement;\
                    if(!shift) {{\
                        if(focused === last) {{ first.focus(); return true; }}\
                    }} else {{\
                        if(focused === first) {{ last.focus(); return true; }}\
                    }}\
                    return false;\
                }})({shift})"
            ))
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            if should_prevent {
                ev.prevent_default();
            }
        }
    };

    view! {
        {move || show.then(|| view! {
            <div class="fixed inset-0 z-50 flex items-center justify-center">
                <div class="fixed inset-0 bg-black/50" on:click=close></div>
                <div
                    role="dialog"
                    aria-modal="true"
                    class="relative bg-white dark:bg-gray-800 rounded-none shadow-xl max-w-lg w-full mx-4 p-6"
                    on:keydown=handle_keydown
                >
                    <div class="flex items-center justify-between mb-4">
                        {if has_title {
                            view! {
                                <h3 class="text-lg font-semibold font-mono text-gray-900 dark:text-gray-100">
                                    {title_clone.clone()}
                                </h3>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }}
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
        })}
    }
}
