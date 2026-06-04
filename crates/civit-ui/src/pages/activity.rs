#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;

use crate::api::client::ApiClient;
use crate::components::{Avatar, Badge, BadgeColor, Card, ErrorBanner, Spinner};
use crate::state::auth::use_auth;

#[derive(Clone, serde::Deserialize)]
pub struct ActivityItem {
    pub id: String,
    pub actor_id: String,
    pub action: String,
    pub resource_type: String,
    #[serde(default)]
    pub resource_id: Option<String>,
    #[serde(default)]
    pub repo_id: Option<String>,
    #[serde(default)]
    pub org_id: Option<String>,
    pub description: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
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
        "create_repo" => (BadgeColor::Success, "created repo".into()),
        "open_issue" => (BadgeColor::Info, "opened issue".into()),
        "close_issue" => (BadgeColor::Neutral, "closed issue".into()),
        "merge_pr" => (BadgeColor::Success, "merged PR".into()),
        "open_pr" => (BadgeColor::Info, "opened PR".into()),
        "create_wiki" => (BadgeColor::Warning, "created wiki".into()),
        "edit_wiki" => (BadgeColor::Warning, "edited wiki".into()),
        "fork_repo" => (BadgeColor::Info, "forked".into()),
        "star_repo" => (BadgeColor::Info, "starred".into()),
        "comment" => (BadgeColor::Neutral, "comment".into()),
        "join_org" => (BadgeColor::Info, "joined".into()),
        "leave_org" => (BadgeColor::Neutral, "left".into()),
        _ => (BadgeColor::Neutral, action.to_string()),
    }
}

fn action_verb(action: &str) -> String {
    match action {
        "push" => "pushed to".to_string(),
        "create_repo" => "created repository".to_string(),
        "open_issue" => "opened issue on".to_string(),
        "close_issue" => "closed issue on".to_string(),
        "merge_pr" => "merged PR in".to_string(),
        "open_pr" => "opened PR in".to_string(),
        "create_wiki" => "created wiki page in".to_string(),
        "edit_wiki" => "edited wiki page in".to_string(),
        "fork_repo" => "forked".to_string(),
        "star_repo" => "starred".to_string(),
        "comment" => "commented on".to_string(),
        "join_org" => "joined".to_string(),
        "leave_org" => "left".to_string(),
        _ => action.to_string(),
    }
}

const FILTERS: &[&str] = &["all", "push", "open_issue", "merge_pr", "create_repo"];

#[component]
pub fn ActivityPage() -> impl IntoView {
    let auth = use_auth();
    let (activities, set_activities) = signal(Vec::<ActivityItem>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (filter, set_filter) = signal("all".to_string());

    leptos::task::spawn_local(async move {
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        match client.get("/activity?limit=50").await {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<Vec<ActivityItem>>().await {
                    Ok(items) => set_activities.set(items),
                    Err(_) => set_error.set(Some(sanitize_error("Failed to parse activity data."))),
                }
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

    let filtered = move || {
        let f = filter.get();
        let items = activities.get();
        if f == "all" {
            items
        } else {
            items
                .into_iter()
                .filter(|a| a.action == f)
                .collect::<Vec<_>>()
        }
    };

    let dismiss_error = Callback::new(move |_: ()| set_error.set(None));

    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Activity"</h1>
                <p class="mt-1 text-gray-600 dark:text-gray-400">
                    "Recent activity across your repositories."
                </p>
            </div>

            <div class="flex gap-2 flex-wrap">
                {FILTERS.iter().map(|f| {
                    let f_str = (*f).to_string();
                    let label = (*f).to_uppercase();
                    let on_click_f = f_str.clone();
                    let is_active = move || filter.get() == f_str;
                    view! {
                        <button
                            class=move || if is_active() {
                                "px-3 py-1 text-sm font-mono rounded-sm border-2 border-blue-600 dark:border-blue-400 bg-blue-600 text-white dark:bg-blue-400 dark:text-gray-900"
                            } else {
                                "px-3 py-1 text-sm font-mono rounded-sm border-2 border-gray-300 dark:border-gray-600 bg-transparent text-gray-700 dark:text-gray-300 hover:border-blue-500 dark:hover:border-blue-400"
                            }
                            on:click=move |_| set_filter.set(on_click_f.clone())
                        >
                            {label}
                        </button>
                    }
                }).collect::<Vec<_>>()}
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

            <Show when=move || !loading.get() && filtered().is_empty() && error.get().is_none() fallback=|| view! { <div class="hidden"></div> }>
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

            <Show when=move || !filtered().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="divide-y divide-gray-100 dark:divide-gray-700">
                        <For each=filtered key=|a| a.id.clone() let:item>
                            {
                                let (badge_color, badge_text) = action_badge(&item.action);
                                let time_str = relative_time(&item.created_at);
                                let actor_id = item.actor_id.clone();
                                let actor_short = if actor_id.len() > 8 {
                                    actor_id[..8].to_string()
                                } else {
                                    actor_id.clone()
                                };
                                let has_desc = !item.description.is_empty();
                                let desc = item.description.clone();
                                let verb = action_verb(&item.action);
                                let is_repo = item.resource_type == "repo";
                                let repo_id = item.repo_id.clone();
                                let resource_id = item.resource_id.clone();
                                let res_short = resource_id.as_ref().map(|id| {
                                    if id.len() > 8 { id[..8].to_string() } else { id.clone() }
                                }).unwrap_or_else(|| "repo".to_string());
                                view! {
                                    <div class="flex items-start gap-3 py-3 px-1 hover:bg-gray-50 dark:hover:bg-gray-750 -mx-1 rounded transition-colors">
                                        <Avatar name=actor_id.clone() size=28 />
                                        <div class="min-w-0 flex-1">
                                            <div class="flex items-center gap-2 flex-wrap">
                                                <span class="text-sm font-medium text-gray-900 dark:text-gray-100">
                                                    {actor_short}
                                                </span>
                                                <Badge color=badge_color text=badge_text />
                                                <span class="text-sm text-gray-600 dark:text-gray-400">
                                                    {verb}
                                                </span>
                                            </div>
                                            {has_desc.then(|| view! {
                                                <p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5 truncate">{desc}</p>
                                            })}
                                            <div class="flex items-center gap-2 mt-0.5 text-xs text-gray-400 dark:text-gray-500">
                                                <span>{time_str}</span>
                                                {is_repo.then(|| view! {
                                                    <A href=format!("/repos/_/{}", repo_id.as_deref().unwrap_or(""))>
                                                        <span class="hover:text-blue-600 dark:hover:text-blue-400 font-mono">
                                                            {res_short}
                                                        </span>
                                                    </A>
                                                })}
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
