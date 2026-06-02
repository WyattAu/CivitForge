#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::{Button, ButtonVariant};

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center py-20">
            <h1 class="text-6xl font-bold text-gray-300 dark:text-gray-600">"404"</h1>
            <p class="mt-4 text-xl text-gray-600 dark:text-gray-400">"Page not found"</p>
            <p class="mt-2 text-gray-500 dark:text-gray-500">
                "The page you're looking for doesn't exist or has been moved."
            </p>
            <div class="mt-6">
                <A href="/">
                    <Button variant=ButtonVariant::Primary extra_class="btn-go-home">
                        "Go Home"
                    </Button>
                </A>
            </div>
        </div>
    }
}
