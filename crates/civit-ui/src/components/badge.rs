#![forbid(unsafe_code)]

use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BadgeColor {
    Success,
    Warning,
    Danger,
    Info,
    Neutral,
}

impl BadgeColor {
    pub fn class(&self) -> &'static str {
        match self {
            Self::Success => "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
            Self::Warning => {
                "bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200"
            }
            Self::Danger => "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
            Self::Info => "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
            Self::Neutral => "bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200",
        }
    }
}

#[component]
pub fn Badge(color: BadgeColor, text: String) -> impl IntoView {
    view! {
        <span class=format!(
            "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {}",
            color.class()
        )>
            {text}
        </span>
    }
}
