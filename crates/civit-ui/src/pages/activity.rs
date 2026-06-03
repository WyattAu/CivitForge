#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;

use crate::api::client::ApiClient;
use crate::components::{Avatar, Badge, BadgeColor, Card, ErrorBanner, Spinner};
use crate::state::auth::use_auth;

#[derive(Clone, serde::Deserialize)]
pub struct ActivityItem {
    pub id: String,
    pub actor: String,
    pub action_type: String,
    pub target: String,
    pub repo_name: String,
    pub created_at: String,
}

fn relative_time(ts: &str) -> String {
    #[cfg(feature = "csr")]
    {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
            let now = chrono::Utc::now();
            let diff = now.signed_duration_since(dt);
            if diff.num_seconds() < 60 {
                return "just now".to_string();
            } else if diff.num_minutes() < 60 {
                return format!("{}m ago", diff.num_minutes());
            } else if diff.num_hours() < 24 {
                return format!("{}h ago", diff.num_hours());
            } else if diff.num_days() < 30 {
                return format!("{}d ago", diff.num_days());
            } else {
                return dt.format("%b %d, %Y").to_string();
            }
        }
    }
    ts.to_string()
}

fn sanitize_error(raw: &str) -> String {
    raw.chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '.' || *c == '-')
        .collect::<String>()
        .trim()
        .to_string()
}

fn action_badge(action: &str) -> (BadgeColor, String) {
    match action {
        "push" => (BadgeColor::Success, "pushed".into()),
        "commit" => (BadgeColor::Success, "commit".into()),
        "issue_open" | "opened_issue" => (BadgeColor::Info, "opened issue".into()),
        "issue_close" | "closed_issue" => (BadgeColor::Neutral, "closed issue".into()),
        "wiki_edit" | "edited_wiki" => (BadgeColor::Warning, "edited wiki".into()),
        "wiki_create" | "created_wiki" => (BadgeColor::Warning, "created wiki".into()),
        _ => (BadgeColor::Neutral, action.to_string()),
    }
}

#[component]
pub fn ActivityPage() -> impl IntoView {
    let auth = use_auth();
    let (activities, set_activities) = signal(Vec::<ActivityItem>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    leptos::task::spawn_local(async move {
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        match client.get("/repos").await {
            Ok(resp) if resp.status().is_success() => {
                let items: Vec<ActivityItem> = Vec::new();
                set_activities.set(items);
            }
            Ok(_) => {
                set_error.set(Some(sanitize_error("Failed to load activity feed.")));
            }
            Err(e) => {
                let msg = format!("{e}");
                set_error.set(Some(sanitize_error(&msg)));
            }
        }
        set_loading.set(false);
    });

    let has_activities = move || !activities.with(|a| a.is_empty());

    let dismiss_error = Callback::new(move |_: ()| set_error.set(None));

    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Activity"</h1>
                <p class="mt-1 text-gray-600 dark:text-gray-400">
                    "Recent activity across your repositories."
                </p>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=dismiss_error />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center gap-2">
                    <Spinner />
                    <span class="text-gray-500 dark:text-gray-400">"Loading activity..."</span>
                </div>
            </Show>

            <Show when=move || !loading.get() && !has_activities() && error.get().is_none() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="text-center py-12">
                        <svg class="mx-auto h-12 w-12 text-gray-300 dark:text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
                        </svg>
                        <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-gray-100">"No recent activity"</h3>
                        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                            "Activity from your repositories will appear here."
                        </p>
                    </div>
                </Card>
            </Show>

            <Show when=has_activities fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="divide-y divide-gray-100 dark:divide-gray-700">
                        <For each=move || activities.get() key=|a| a.id.clone() let:item>
                            {
                                let (badge_color, badge_text) = action_badge(&item.action_type);
                                let time_str = relative_time(&item.created_at);
                                view! {
                                    <div class="flex items-center gap-3 py-3 px-1 hover:bg-gray-50 dark:hover:bg-gray-750 -mx-1 rounded transition-colors">
                                        <Avatar name=item.actor.clone() size=28 />
                                        <div class="min-w-0 flex-1">
                                            <div class="flex items-center gap-2 flex-wrap">
                                                <span class="text-sm font-medium text-gray-900 dark:text-gray-100">
                                                    {item.actor.clone()}
                                                </span>
                                                <Badge color=badge_color text=badge_text />
                                                <span class="text-sm text-gray-700 dark:text-gray-300 truncate">
                                                    {item.target.clone()}
                                                </span>
                                            </div>
                                            <div class="flex items-center gap-2 mt-0.5 text-xs text-gray-400 dark:text-gray-500">
                                                <A href=format!("/repos/{}", item.repo_name.clone())>
                                                    <span class="hover:text-blue-600 dark:hover:text-blue-400 font-mono">
                                                        {item.repo_name.clone()}
                                                    </span>
                                                </A>
                                                <span class="text-gray-300 dark:text-gray-600">"-"</span>
                                                <span>{time_str}</span>
                                            </div>
                                        </div>
                                    </div>
                                }
                            }
                        </For>
                    </div>
                </Card>
            </Show>
        </div>
    }
}
