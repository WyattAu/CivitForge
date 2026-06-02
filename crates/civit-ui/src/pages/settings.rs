#![forbid(unsafe_code)]

use leptos::prelude::*;

use crate::components::{Button, ButtonVariant, Card, Input, InputType};

#[component]
pub fn RepoSettingsPage() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Repository Settings"</h1>

            <Card title="General" description="Basic repository settings">
                <form class="space-y-5">
                    <Input label="Repository name" name="name" input_type=InputType::Text placeholder="my-repo" required=true></Input>
                    <Input label="Description" name="description" input_type=InputType::Textarea placeholder="A brief description..."></Input>
                    <div>
                        <Button variant=ButtonVariant::Primary extra_class="btn-save-settings">
                            "Save Changes"
                        </Button>
                    </div>
                </form>
            </Card>

            <Card title="Danger Zone" description="Irreversible and destructive actions">
                <div class="border border-red-200 dark:border-red-800 rounded-md p-4">
                    <h3 class="text-sm font-medium text-red-600 dark:text-red-400">"Delete this repository"</h3>
                    <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
                        "Once you delete a repository, there is no going back."
                    </p>
                    <div class="mt-3">
                        <Button variant=ButtonVariant::Danger extra_class="btn-delete-repo">
                            "Delete Repository"
                        </Button>
                    </div>
                </div>
            </Card>
        </div>
    }
}

#[component]
pub fn SettingsPage() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"User Settings"</h1>

            <Card title="Profile" description="Manage your public profile information">
                <form class="space-y-5">
                    <Input label="Display Name" name="display_name" input_type=InputType::Text placeholder="John Doe"></Input>
                    <Input label="Email" name="email" input_type=InputType::Email placeholder="you@example.com"></Input>
                    <Input label="Bio" name="bio" input_type=InputType::Textarea placeholder="Tell us about yourself..."></Input>
                    <Button variant=ButtonVariant::Primary extra_class="btn-save-profile">
                        "Save Profile"
                    </Button>
                </form>
            </Card>

            <Card title="SSH Keys" description="Manage your SSH keys for repository access">
                <div class="py-4 text-center text-gray-400 dark:text-gray-500">
                    "No SSH keys configured."
                </div>
                <Button variant=ButtonVariant::Secondary extra_class="btn-add-ssh">
                    "Add SSH Key"
                </Button>
            </Card>
        </div>
    }
}
