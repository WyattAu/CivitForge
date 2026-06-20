#![forbid(unsafe_code)]

use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum SpinnerSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl SpinnerSize {
    fn dimensions(&self) -> &'static str {
        match self {
            Self::Small => "w-4 h-4 border-2",
            Self::Medium => "w-8 h-8 border-3",
            Self::Large => "w-12 h-12 border-4",
        }
    }
}

#[component]
pub fn Spinner(
    #[prop(optional)] size: SpinnerSize,
    #[prop(optional)] _class: String,
) -> impl IntoView {
    let full_class = format!(
        "animate-spin rounded-full border-gray-300 dark:border-gray-600 \
         border-t-blue-600 dark:border-t-blue-400 {}",
        size.dimensions()
    );

    view! {
        <div class=full_class role="status" aria-label="Loading">
            <span class="sr-only">"Loading..."</span>
        </div>
    }
}

#[component]
pub fn Skeleton(#[prop(optional)] class: String, #[prop(optional)] lines: usize) -> impl IntoView {
    let base = "animate-pulse rounded bg-gray-200 dark:bg-gray-700";
    let full_class = if class.is_empty() {
        base.to_string()
    } else {
        format!("{base} {class}")
    };

    view! {
        <div class="space-y-3">
            <For each=move || 0..lines key=|i| *i let:_line>
                <div class=full_class.clone() style="height:1rem"></div>
            </For>
        </div>
    }
}
