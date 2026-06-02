#![forbid(unsafe_code)]

use either_of::Either;
use leptos::prelude::*;

#[component]
pub fn Avatar(
    #[prop(optional)] src: String,
    #[prop(optional)] alt: String,
    #[prop(optional)] size: usize,
    #[prop(optional)] name: String,
) -> impl IntoView {
    let size_px = if size == 0 { 40 } else { size };
    let (name_sig, _) = signal(name);
    let font_size = (size_px as f64 * 0.4) as usize;

    if !src.is_empty() {
        Either::Left(view! {
            <img
                src=src
                alt=alt
                class="rounded-full object-cover"
                style=format!("width:{size_px}px;height:{size_px}px")
            />
        })
    } else {
        Either::Right(view! {
            <div
                class="rounded-full bg-blue-600 text-white flex items-center justify-center font-medium select-none dark:bg-blue-500"
                style=format!("width:{size_px}px;height:{size_px}px;font-size:{font_size}px")
                title=name_sig.get()
            >
                {move || {
                    name_sig.get()
                        .split_whitespace()
                        .take(2)
                        .map(|w| w.chars().next().unwrap_or_default())
                        .collect::<String>()
                        .to_uppercase()
                }}
            </div>
        })
    }
}
