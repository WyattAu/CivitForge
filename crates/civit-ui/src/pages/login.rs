#![forbid(unsafe_code)]

use leptos::prelude::*;

use crate::components::{Button, ButtonVariant, Input, InputType};

#[component]
pub fn LoginPage() -> impl IntoView {
    let (is_register, set_is_register) = signal(false);

    let handle_submit = move |ev| {
        let _ = event_target_value(&ev);
    };

    view! {
        <div class="flex min-h-screen items-center justify-center py-12 px-4 sm:px-6 lg:px-8">
            <div class="w-full max-w-md">
                <div class="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-8">
                    <div class="text-center mb-8">
                        <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                            {move || if is_register.get() { "Create Account" } else { "Sign In" }}
                        </h1>
                        <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
                            {move || if is_register.get() { "Join the CivitForge community." } else { "Sign in to your CivitForge account." }}
                        </p>
                    </div>

                    <form on:submit=handle_submit class="space-y-5">
                        {is_register.get().then(|| view! {
                            <Input
                                label="Username"
                                name="username"
                                input_type=InputType::Text
                                placeholder="johndoe"
                                required=true
                            ></Input>
                        })}
                        <Input
                            label="Email"
                            name="email"
                            input_type=InputType::Email
                            placeholder="you@example.com"
                            required=true
                        ></Input>
                        <Input
                            label="Password"
                            name="password"
                            input_type=InputType::Password
                            placeholder="••••••••"
                            required=true
                        ></Input>
                        <Button
                            variant=ButtonVariant::Primary
                            extra_class="w-full justify-center"
                        >
                            {move || if is_register.get() { "Register" } else { "Sign In" }}
                        </Button>
                    </form>

                    <div class="mt-6 text-center text-sm">
                        <button
                            class="text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300"
                            on:click=move |_| set_is_register.set(!is_register.get())
                        >
                            {move || if is_register.get() { "Already have an account? Sign In" } else { "Don't have an account? Register" }}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
