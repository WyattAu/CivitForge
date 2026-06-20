#![forbid(unsafe_code)]

use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Danger,
    Ghost,
}

impl ButtonVariant {
    pub fn class(&self) -> &'static str {
        match self {
            Self::Primary => {
                "bg-blue-600 hover:bg-blue-700 text-white \
                 dark:bg-blue-500 dark:hover:bg-blue-600"
            }
            Self::Secondary => {
                "bg-gray-200 hover:bg-gray-300 text-gray-900 \
                 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-100"
            }
            Self::Danger => {
                "bg-red-600 hover:bg-red-700 text-white \
                 dark:bg-red-500 dark:hover:bg-red-600"
            }
            Self::Ghost => {
                "bg-transparent hover:bg-gray-100 text-gray-700 \
                 dark:hover:bg-gray-800 dark:text-gray-300"
            }
        }
    }
}

#[component]
pub fn Button(
    #[prop(optional)] variant: ButtonVariant,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] extra_class: &'static str,
    children: ChildrenFn,
) -> impl IntoView {
    let base = "inline-flex items-center justify-center px-4 py-2 rounded-none \
                text-sm font-medium transition-colors focus:outline-none \
                focus-visible:ring-2 focus-visible:ring-offset-2 \
                dark:focus-visible:ring-offset-gray-900 \
                disabled:opacity-50 disabled:cursor-not-allowed";
    let full_class = if extra_class.is_empty() {
        format!("{base} {}", variant.class())
    } else {
        format!("{base} {} {}", variant.class(), extra_class)
    };

    view! {
        <button class=full_class disabled=disabled>
            {children()}
        </button>
    }
}
