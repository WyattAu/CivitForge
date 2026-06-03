#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::components::{Badge, BadgeColor, Card, Spinner};
use crate::state::auth::use_auth;

#[derive(Clone, serde::Deserialize)]
pub struct PipelineRunResponse {
    pub id: String,
    #[allow(dead_code)]
    pub repo_id: String,
    pub trigger: String,
    pub ref_name: Option<String>,
    pub commit_sha: String,
    pub status: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

fn pipeline_status_color(status: &str) -> BadgeColor {
    match status {
        "success" | "completed" => BadgeColor::Success,
        "failed" | "failure" => BadgeColor::Danger,
        "running" | "in_progress" => BadgeColor::Warning,
        "pending" | "queued" => BadgeColor::Neutral,
        "canceled" | "cancelled" => BadgeColor::Danger,
        _ => BadgeColor::Neutral,
    }
}

fn pipeline_status_label(status: &str) -> String {
    match status {
        "in_progress" => "Running".to_string(),
        s => {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        }
    }
}

fn truncate_sha(sha: &str) -> String {
    if sha.len() > 7 {
        sha[..7].to_string()
    } else {
        sha.to_string()
    }
}

fn format_pipeline_duration(started: Option<&str>, finished: Option<&str>) -> String {
    let start = match started.and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()) {
        Some(dt) => dt.with_timezone(&chrono::Utc),
        None => return "-".to_string(),
    };
    let end = match finished.and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()) {
        Some(dt) => dt.with_timezone(&chrono::Utc),
        None => return "-".to_string(),
    };
    let diff = end.signed_duration_since(start);
    if diff.num_seconds() < 60 {
        format!("{}s", diff.num_seconds())
    } else if diff.num_minutes() < 60 {
        format!("{}m {}s", diff.num_minutes(), diff.num_seconds() % 60)
    } else {
        format!("{}h {}m", diff.num_hours(), diff.num_minutes() % 60)
    }
}

fn relative_time(created_at: &str) -> String {
    #[cfg(feature = "csr")]
    {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(created_at) {
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
    created_at.to_string()
}

#[component]
pub fn PipelinesPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let auth = use_auth();

    let (pipelines_sig, set_pipelines) = signal(vec![]);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let fetch_pipelines = move || {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        let owner_val = owner();
        let name_val = name();
        let path = format!("/repos/{owner_val}/{name_val}/pipelines?limit=50&offset=0");
        leptos::task::spawn_local(async move {
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<Vec<PipelineRunResponse>>().await {
                        Ok(data) => set_pipelines.set(data),
                        Err(_) => set_error.set(Some("Failed to parse pipeline data.".to_string())),
                    }
                }
                Ok(_) => {
                    set_error.set(Some("Failed to load pipelines.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_loading.set(false);
        });
    };

    fetch_pipelines();

    let dismiss_error = move |_| set_error.set(None);

    let owner_v = owner();
    let name_v = name();

    view! {
        <div class="space-y-6">
            <div>
                <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1">
                    <A href="/repos"><span class="hover:text-blue-600 dark:hover:text-blue-400">"Repositories"</span></A>
                    <span>"/"</span>
                    <A href=format!("/repos/{owner_v}/{name_v}")><span class="hover:text-blue-600 dark:hover:text-blue-400">{format!("{owner_v}/{name_v}")}</span></A>
                </div>
                <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Pipelines"</h1>
                <p class="mt-1 text-gray-600 dark:text-gray-400">"CI/CD pipeline runs for this repository."</p>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <div class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md flex items-center justify-between">
                    <p class="text-sm text-red-700 dark:text-red-400">{move || error.get().unwrap_or_default()}</p>
                    <button on:click=dismiss_error class="text-red-500 hover:text-red-700 dark:text-red-400 dark:hover:text-red-200 text-sm font-medium">"Dismiss"</button>
                </div>
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center justify-center py-12">
                    <Spinner />
                </div>
            </Show>

            <Show when=move || !loading.get() && pipelines_sig.with(|p| p.is_empty()) && error.get().is_none() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="py-12 text-center">
                        <p class="text-gray-500 dark:text-gray-400 text-lg">"No pipeline runs yet."</p>
                        <p class="text-sm text-gray-400 dark:text-gray-500 mt-1">"Push a commit to trigger your first pipeline."</p>
                    </div>
                </Card>
            </Show>

            <Show when=move || !loading.get() && !pipelines_sig.with(|p| p.is_empty()) fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="divide-y divide-gray-100 dark:divide-gray-700">
                        <For each=move || pipelines_sig.get() key=|p| p.id.clone() let:pipeline>
                            {
                                let status_color = pipeline_status_color(&pipeline.status);
                                let status_text = pipeline_status_label(&pipeline.status);
                                let duration = format_pipeline_duration(
                                    pipeline.started_at.as_deref(),
                                    pipeline.finished_at.as_deref(),
                                );
                                let time_str = relative_time(&pipeline.created_at);
                                let ref_name = pipeline.ref_name.clone().unwrap_or_default();
                                let commit_short = truncate_sha(&pipeline.commit_sha);
                                let owner_link = owner();
                                let name_link = name();
                                view! {
                                    <A href=format!("/repos/{owner_link}/{name_link}/pipelines/{}", pipeline.id)>
                                        <div class="flex items-center gap-4 py-3 px-1 hover:bg-gray-50 dark:hover:bg-gray-750 -mx-1 rounded transition-colors cursor-pointer">
                                            <Badge color=status_color text=status_text />
                                            <span class="font-mono text-sm text-blue-600 dark:text-blue-400 shrink-0">{commit_short}</span>
                                            <span class="text-sm text-gray-700 dark:text-gray-300 truncate flex-1">{pipeline.trigger.clone()}</span>
                                            <span class="text-xs text-gray-500 dark:text-gray-400 font-mono shrink-0">{ref_name}</span>
                                            <span class="text-xs text-gray-400 dark:text-gray-500 shrink-0 w-16 text-right">{duration}</span>
                                            <span class="text-xs text-gray-400 dark:text-gray-500 shrink-0 w-20 text-right">{time_str}</span>
                                        </div>
                                    </A>
                                }
                            }
                        </For>
                    </div>
                </Card>
            </Show>
        </div>
    }
}
