#![forbid(unsafe_code)]

//! Import page — Migration API v1 frontend (ADR-0006).
//!
//! Single-URL imports, Forgejo/Gitea bulk imports, and the live job board.

use leptos::prelude::*;

use crate::api::client::ApiClient;
use crate::components::{Button, ButtonVariant, Card, ErrorBanner};
use crate::state::auth::use_auth;

#[derive(Debug, Clone, serde::Deserialize)]
struct ImportJobView {
    #[serde(default)]
    id: String,
    #[serde(default)]
    forge: String,
    #[serde(default)]
    dest_owner: String,
    #[serde(default)]
    dest_name: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    commit_count: Option<i64>,
}

fn status_badge_class(status: &str) -> &'static str {
    match status {
        "completed" => "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
        "failed" => "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
        "queued" | "cloning" | "verifying" => {
            "bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200"
        }
        _ => "bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200",
    }
}

#[component]
pub fn ImportPage() -> impl IntoView {
    let auth = use_auth();
    let (jobs, set_jobs) = signal(Vec::<ImportJobView>::new());
    let (jobs_loading, set_jobs_loading) = signal(true);
    // True while any job is queued/cloning/verifying — drives auto-refresh
    let (jobs_active, set_jobs_active) = signal(false);
    let (form_error, set_form_error) = signal(None::<String>);
    let (url_loading, set_url_loading) = signal(false);
    let (bulk_loading, set_bulk_loading) = signal(false);
    let (bulk_result, set_bulk_result) = signal(None::<String>);

    // Job board loader — sets jobs_active for the auto-refresh Effect.
    let fetch_jobs = move || {
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        leptos::task::spawn_local(async move {
            #[derive(serde::Deserialize)]
            struct JobsResponse {
                #[serde(default)]
                jobs: Vec<ImportJobView>,
            }
            let mut has_active = false;
            if let Ok(resp) = client.get("/import/jobs").await {
                if resp.status().is_success() {
                    if let Ok(d) = resp.json::<JobsResponse>().await {
                        has_active = d.jobs.iter().any(|j| {
                            matches!(j.status.as_str(), "queued" | "cloning" | "verifying")
                        });
                        set_jobs.set(d.jobs);
                    }
                }
            }
            set_jobs_loading.set(false);
            set_jobs_active.set(has_active);
        })
    };

    // Initial load
    fetch_jobs();

    // Auto-refresh every 5s while jobs are in flight
    Effect::new(move |_| {
        if jobs_active.get() {
            leptos::task::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(5_000).await;
                fetch_jobs();
            });
        }
    });

    // Single URL import
    let handle_url_import = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_form_error.set(None);
        let url = crate::utils::get_input_value("import-url");
        let name = crate::utils::get_input_value("import-name");
        let token = crate::utils::get_input_value("import-token");

        if url.trim().is_empty() {
            set_form_error.set(Some("Repository URL is required.".into()));
            return;
        }

        set_url_loading.set(true);
        let token_auth = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token_auth);
        leptos::task::spawn_local(async move {
            #[derive(serde::Serialize)]
            struct UrlImportBody {
                url: String,
                #[serde(skip_serializing_if = "Option::is_none")]
                name: Option<String>,
                #[serde(skip_serializing_if = "Option::is_none")]
                token: Option<String>,
            }
            let body = UrlImportBody {
                url: url.trim().to_string(),
                name: if name.trim().is_empty() {
                    None
                } else {
                    Some(name.trim().to_string())
                },
                token: if token.trim().is_empty() {
                    None
                } else {
                    Some(token.trim().to_string())
                },
            };
            match client.post("/import/url", &body).await {
                Ok(resp) if resp.status().is_success() || resp.status() == 202 => {
                    set_url_loading.set(false);
                    fetch_jobs();
                }
                Ok(resp) => {
                    let status = resp.status();
                    let msg = if status == 401 {
                        "Session expired. Please sign in again.".to_string()
                    } else {
                        format!("Import failed (HTTP {status}).")
                    };
                    set_form_error.set(Some(msg));
                    set_url_loading.set(false);
                }
                Err(_) => {
                    set_form_error.set(Some("Network error. Check your connection.".into()));
                    set_url_loading.set(false);
                }
            }
        });
    };

    // Forgejo bulk import
    let handle_bulk_import = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_form_error.set(None);
        set_bulk_result.set(None);
        let host_url = crate::utils::get_input_value("bulk-host");
        let owner = crate::utils::get_input_value("bulk-owner");
        let token = crate::utils::get_input_value("bulk-token");

        if host_url.trim().is_empty() || owner.trim().is_empty() {
            set_form_error.set(Some("Host URL and owner are required.".into()));
            return;
        }

        set_bulk_loading.set(true);
        let token_auth = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token_auth);
        leptos::task::spawn_local(async move {
            #[derive(serde::Serialize)]
            struct BulkBody {
                host_url: String,
                owner: String,
                #[serde(skip_serializing_if = "Option::is_none")]
                token: Option<String>,
            }
            let body = BulkBody {
                host_url: host_url.trim().trim_end_matches('/').to_string(),
                owner: owner.trim().to_string(),
                token: if token.trim().is_empty() {
                    None
                } else {
                    Some(token.trim().to_string())
                },
            };
            match client.post("/import/forgejo/bulk", &body).await {
                Ok(resp) if resp.status().is_success() => {
                    #[derive(serde::Deserialize)]
                    struct BulkResponse {
                        #[serde(default)]
                        count: usize,
                        #[serde(default)]
                        skipped: Vec<String>,
                    }
                    match resp.json::<BulkResponse>().await {
                        Ok(d) => {
                            let mut msg = format!("Queued {} repos for import.", d.count);
                            if !d.skipped.is_empty() {
                                msg.push_str(&format!(" Skipped: {}.", d.skipped.join(", ")));
                            }
                            set_bulk_result.set(Some(msg));
                        }
                        Err(_) => set_bulk_result.set(Some("Bulk import queued.".into())),
                    }
                    set_bulk_loading.set(false);
                    fetch_jobs();
                }
                Ok(resp) => {
                    let status = resp.status();
                    set_form_error.set(Some(format!("Bulk import failed (HTTP {status}).")));
                    set_bulk_loading.set(false);
                }
                Err(_) => {
                    set_form_error.set(Some("Network error. Could not reach the server.".into()));
                    set_bulk_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Import Repositories"</h1>
                <p class="mt-2 text-gray-600 dark:text-gray-400">
                    "Migrate repositories from Forgejo, Gitea, GitHub, or any Git URL. Track progress on the job board below."
                </p>
            </div>

            <div class="grid gap-6 lg:grid-cols-2">
                // ── Single URL import ──
                <Card title="Import from Git URL".to_string()>
                    <form on:submit=handle_url_import class="space-y-4">
                        <Show when=move || form_error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                            <ErrorBanner message=move || form_error.get().unwrap_or_default() />
                        </Show>
                        <div>
                            <label for="import-url" class="block text-sm font-medium text-gray-700 dark:text-gray-300">"Repository URL"</label>
                            <input
                                type="text" id="import-url" placeholder="https://forgejo.example.com/owner/repo"
                                class="mt-1 block w-full rounded-md border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                            />
                        </div>
                        <div>
                            <label for="import-name" class="block text-sm font-medium text-gray-700 dark:text-gray-300">"Name (optional)"</label>
                            <input
                                type="text" id="import-name" placeholder="Defaults to the repo name"
                                class="mt-1 block w-full rounded-md border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                            />
                        </div>
                        <div>
                            <label for="import-token" class="block text-sm font-medium text-gray-700 dark:text-gray-300">"Access token (optional, for private repos)"</label>
                            <input
                                type="password" id="import-token" placeholder="token"
                                class="mt-1 block w-full rounded-md border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                            />
                        </div>
                        <Button
                            variant=ButtonVariant::Primary
                            disabled=url_loading.get()
                        >
                            {move || if url_loading.get() { "Importing..." } else { "Import" }}
                        </Button>
                    </form>
                </Card>

                // ── Forgejo bulk import ──
                <Card title="Bulk import from Forgejo / Gitea".to_string()>
                    <form on:submit=handle_bulk_import class="space-y-4">
                        <div>
                            <label for="bulk-host" class="block text-sm font-medium text-gray-700 dark:text-gray-300">"Instance URL"</label>
                            <input
                                type="text" id="bulk-host" placeholder="https://forgejo.example.com"
                                class="mt-1 block w-full rounded-md border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                            />
                        </div>
                        <div>
                            <label for="bulk-owner" class="block text-sm font-medium text-gray-700 dark:text-gray-300">"User or organization"</label>
                            <input
                                type="text" id="bulk-owner" placeholder="owner"
                                class="mt-1 block w-full rounded-md border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                            />
                        </div>
                        <div>
                            <label for="bulk-token" class="block text-sm font-medium text-gray-700 dark:text-gray-300">"Access token (optional, for private repos)"</label>
                            <input
                                type="password" id="bulk-token" placeholder="token"
                                class="mt-1 block w-full rounded-md border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                            />
                        </div>
                        <Show when=move || bulk_result.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                            <div class="rounded-md bg-green-50 dark:bg-green-900/30 p-3 text-sm text-green-800 dark:text-green-200">
                                {move || bulk_result.get().unwrap_or_default()}
                            </div>
                        </Show>
                        <Button
                            variant=ButtonVariant::Primary
                            disabled=bulk_loading.get()
                        >
                            {move || if bulk_loading.get() { "Importing..." } else { "Import all repos" }}
                        </Button>
                    </form>
                </Card>
            </div>

            // ── Job board ──
            <Card title="Import Jobs".to_string()>
                <Show when=move || !jobs_loading.get() fallback=|| view! {
                    <div class="py-8 text-center text-gray-500 dark:text-gray-400">"Loading jobs..."</div>
                }>
                    <Show when=move || !jobs.get().is_empty() fallback=|| view! {
                        <div class="py-8 text-center text-gray-500 dark:text-gray-400">
                            "No import jobs yet. Start one above."
                        </div>
                    }>
                        <div class="overflow-x-auto">
                            <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                                <thead>
                                    <tr>
                                        <th class="px-4 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">"Repository"</th>
                                        <th class="px-4 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">"Forge"</th>
                                        <th class="px-4 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">"Status"</th>
                                        <th class="px-4 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">"Commits"</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-gray-100 dark:divide-gray-700/50">
                                    <For each=move || jobs.get() key=|j| j.id.clone() let:job>
                                        {
                                            let job_error = job.error.clone().unwrap_or_default();
                                            let job_error_for_check = job_error.clone();
                                            view! {
                                                <tr>
                                                    <td class="px-4 py-2 text-sm text-gray-900 dark:text-gray-100">
                                                        {format!("{}/{}", job.dest_owner, job.dest_name)}
                                                    </td>
                                                    <td class="px-4 py-2 text-sm text-gray-500 dark:text-gray-400">{job.forge.clone()}</td>
                                                    <td class="px-4 py-2">
                                                        <span class={format!("inline-flex px-2 py-0.5 rounded-sm text-xs font-medium {}", status_badge_class(&job.status))}>
                                                            {job.status.clone()}
                                                        </span>
                                                        <Show when=move || !job_error_for_check.is_empty()>
                                                            <p class="mt-1 text-xs text-red-600 dark:text-red-400">{job_error.clone()}</p>
                                                        </Show>
                                                    </td>
                                                    <td class="px-4 py-2 text-sm text-gray-500 dark:text-gray-400">
                                                        {job.commit_count.map(|c| c.to_string()).unwrap_or_else(|| "-".into())}
                                                    </td>
                                                </tr>
                                            }
                                        }
                                    </For>
                                </tbody>
                            </table>
                        </div>
                    </Show>
                </Show>
            </Card>
        </div>
    }
}
