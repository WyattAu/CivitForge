#![forbid(unsafe_code)]

use leptos::prelude::*;

#[component]
pub fn Pagination(
    current_page: u32,
    total_pages: u32,
    #[prop(optional)] on_page_change: Option<Callback<u32>>,
) -> impl IntoView {
    let has_prev = current_page > 1;
    let has_next = current_page < total_pages;

    let (on_change_sig, _) = signal(on_page_change);

    let visible_pages = move || {
        (1..=total_pages)
            .filter(|p| {
                let p = *p;
                p == 1 || p == total_pages || (p as i32 - current_page as i32).unsigned_abs() <= 2
            })
            .collect::<Vec<_>>()
    };

    let prev = move |_| {
        if let Some(cb) = on_change_sig.get() {
            cb.run(current_page.saturating_sub(1));
        }
    };
    let next = move |_| {
        if let Some(cb) = on_change_sig.get() {
            cb.run(current_page + 1);
        }
    };

    view! {
        <nav class="flex items-center justify-center space-x-2" aria-label="Pagination">
            <button
                class="px-3 py-1 text-sm rounded-md border border-gray-300 dark:border-gray-600 \
                        disabled:opacity-40 disabled:cursor-not-allowed \
                        hover:bg-gray-100 dark:hover:bg-gray-700"
                disabled=!has_prev
                on:click=prev
                aria-disabled=move || !has_prev
            >
                "Previous"
            </button>
            <For each=visible_pages key=|p| *p let:page>
                {
                    let is_current = page == current_page;
                    let on_change_sig = on_change_sig;
                    move || {
                        let btn_class = if is_current {
                            "px-3 py-1 text-sm rounded-md bg-blue-600 text-white dark:bg-blue-500".to_string()
                        } else {
                            "px-3 py-1 text-sm rounded-md border border-gray-300 dark:border-gray-600 \
                             hover:bg-gray-100 dark:hover:bg-gray-700".to_string()
                        };
                        view! {
                            <button
                                class=btn_class
                                disabled=is_current
                                on:click=move |_| {
                                    if let Some(cb) = on_change_sig.get() {
                                        cb.run(page);
                                    }
                                }
                            >
                                {page}
                            </button>
                        }
                    }
                }
            </For>
            <button
                class="px-3 py-1 text-sm rounded-md border border-gray-300 dark:border-gray-600 \
                        disabled:opacity-40 disabled:cursor-not-allowed \
                        hover:bg-gray-100 dark:hover:bg-gray-700"
                disabled=!has_next
                on:click=next
                aria-disabled=move || !has_next
            >
                "Next"
            </button>
        </nav>
    }
}
