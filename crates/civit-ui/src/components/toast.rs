#![forbid(unsafe_code)]

use leptos::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub enum ToastLevel {
    Success,
    Warning,
    Danger,
    Info,
}

impl ToastLevel {
    fn class(&self) -> &'static str {
        match self {
            Self::Success => "bg-green-600 dark:bg-green-500",
            Self::Warning => "bg-yellow-600 dark:bg-yellow-500",
            Self::Danger => "bg-red-600 dark:bg-red-500",
            Self::Info => "bg-blue-600 dark:bg-blue-500",
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct ToastMessage {
    pub id: uuid::Uuid,
    pub level: ToastLevel,
    pub message: String,
}

#[component]
pub fn ToastContainer(
    toasts: Vec<ToastMessage>,
    #[prop(optional)] on_dismiss: Option<Callback<uuid::Uuid>>,
) -> impl IntoView {
    let (toasts_sig, _) = signal(toasts);
    let (on_dismiss_sig, _) = signal(on_dismiss);

    view! {
        <div class="fixed bottom-4 right-4 z-50 space-y-2 max-w-sm">
            <For
                each=move || toasts_sig.get()
                key=|t| t.id
                let:toast
            >
                {
                    let on_dismiss_sig = on_dismiss_sig;
                    let (toast_id_sig, _) = signal(toast.id);
                    move || view! {
                        <div class=format!(
                            "rounded-lg px-4 py-3 text-white text-sm shadow-lg flex items-center justify-between {}",
                            toast.level.class()
                        )>
                            <span>{toast.message.clone()}</span>
                            <button
                                class="ml-2 text-white/80 hover:text-white"
                                on:click=move |_| {
                                    if let Some(cb) = on_dismiss_sig.get() {
                                        cb.run(toast_id_sig.get());
                                    }
                                }
                                aria-label="Dismiss notification"
                            >
                                "\u{00d7}"
                            </button>
                        </div>
                    }
                }
            </For>
        </div>
    }
}
