#![forbid(unsafe_code)]

use leptos::prelude::*;
use crate::api::client::ApiClient;
use crate::components::{Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Spinner};
use crate::state::auth::use_auth;

#[derive(Debug, Clone, serde::Deserialize)]
struct AdminFeatureFlag {
    id: String,
    name: String,
    description: String,
    enabled: bool,
    enabled_for_users: Vec<String>,
    enabled_for_percentage: i32,
    enabled_for_orgs: Vec<String>,
    created_at: String,
    updated_at: String,
    /// Lifecycle category; sets the staleness deadline (ADR-0008).
    #[serde(default)]
    kind: String,
    /// Accountable party.
    #[serde(default)]
    owner: String,
    /// Rollout-cycle identifier.
    #[serde(default)]
    salt: String,
    /// Staleness clock origin.
    #[serde(default)]
    last_changed_at: String,
}

/// One staleness verdict from the audit endpoint.
#[derive(Debug, Clone, serde::Deserialize)]
struct FlagStaleness {
    id: String,
    name: String,
    kind: String,
    #[serde(default)]
    owner: String,
    staleness: String,
    is_removal_candidate: bool,
    age_days: i64,
    #[serde(default)]
    signals: Vec<FlagSignal>,
}

/// One signal behind a verdict.
#[derive(Debug, Clone, serde::Deserialize)]
struct FlagSignal {
    signal: String,
    #[serde(default)]
    detail: String,
}

/// One in-flight rollout as the controller reports it.
#[derive(Debug, Clone, serde::Deserialize)]
struct FlagRolloutRow {
    flag_id: String,
    name: String,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    owner: String,
    enabled_for_percentage: i32,
    stage_index: i32,
    consecutive_failures: i32,
    #[serde(default)]
    last_decision: String,
    #[serde(default)]
    last_reason: String,
    last_error_rate: Option<f64>,
    last_latency_p99_ms: Option<f64>,
}

/// One recent controller decision.
#[derive(Debug, Clone, serde::Deserialize)]
struct RolloutEventRow {
    #[serde(default)]
    name: String,
    decision: String,
    #[serde(default)]
    reason: String,
    percentage_before: i32,
    percentage_after: i32,
    #[serde(default)]
    error_rate: Option<f64>,
    #[serde(default, rename = "createdAt")]
    created_at: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct RolloutsResponse {
    controller_enabled: bool,
    rollouts: Vec<FlagRolloutRow>,
    recent_events: Vec<RolloutEventRow>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct StalenessAuditResponse {
    flags: Vec<FlagStaleness>,
    total: usize,
    removal_candidates: usize,
    aging: usize,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct AdminFeatureFlagsResponse {
    flags: Vec<AdminFeatureFlag>,
    total: usize,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct AdminWidget {
    id: String,
    widget_name: String,
    widget_config: serde_json::Value,
    position: i32,
    enabled: bool,
    created_at: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct AdminDashboardResponse {
    widgets: Vec<AdminWidget>,
    total: usize,
}

#[component]
pub fn AdminFeatureFlagsPage() -> impl IntoView {
    let auth = use_auth();
    let (flags, set_flags) = signal(Vec::<AdminFeatureFlag>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    let (audit, set_audit) = signal(Option::<StalenessAuditResponse>::None);
    let (rollouts, set_rollouts) = signal(Option::<RolloutsResponse>::None);
    let load_rollouts = Callback::new(move |_| {
        let token = auth.0.with(|s| s.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            if let Ok(resp) = client.get("/admin/feature-flags/rollouts").await {
                if resp.status().is_success() {
                    if let Ok(data) = resp.json::<RolloutsResponse>().await {
                        set_rollouts.set(Some(data));
                    }
                }
            }
        });
    });
    let load_audit = Callback::new(move |_| {
        let token = auth.0.with(|s| s.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            if let Ok(resp) = client.get("/admin/feature-flags/stale").await {
                if resp.status().is_success() {
                    if let Ok(data) = resp.json::<StalenessAuditResponse>().await {
                        set_audit.set(Some(data));
                    }
                }
            }
        });
    });

    let load_flags = Callback::new(move |_| {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|s| s.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            match client.get("/admin/feature-flags").await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<AdminFeatureFlagsResponse>().await {
                        Ok(data) => {
                            set_flags.set(data.flags);
                            set_loading.set(false);
                        }
                        Err(e) => {
                            set_error.set(Some(format!("Failed to parse response: {e}")));
                            set_loading.set(false);
                        }
                    }
                }
                Ok(resp) => {
                    set_error.set(Some(format!("Error: {}", resp.status())));
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(format!("Request failed: {e}")));
                    set_loading.set(false);
                }
            }
        });
    });

    load_flags.run(());
    load_audit.run(());
    load_rollouts.run(());

    let toggle_flag = Callback::new(move |flag_id: String| {
        let token = auth.0.with(|s| s.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let _ = client
                .post(
                    &format!("/admin/feature-flags/{flag_id}/toggle"),
                    &serde_json::json!({}),
                )
                .await;
            load_flags.run(());
        });
    });

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold">"Feature Flags"</h1>
            </div>

            <Show when=move || !loading.get() fallback=|| view! { <Spinner /> }>
                <Show when=move || error.get().is_none() fallback=move || {
                    let err_msg = error.get().unwrap_or_default();
                    view! { <ErrorBanner message=move || err_msg.clone() /> }
                }>
                    <Card>
                        <div class="overflow-x-auto">
                            <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                                <thead class="bg-gray-50 dark:bg-gray-800">
                                    <tr>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Name"</th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Description"</th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Kind"</th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Owner"</th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Status"</th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Rollout"</th>
                                        <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Actions"</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
                                    <For each=move || flags.get() key=|f| f.id.clone() let:flag>
                                        {
                                            let flag_id = flag.id.clone();
                                            let flag_name = flag.name.clone();
                                            let flag_description = flag.description.clone();
                                            let flag_enabled = flag.enabled;
                                            let flag_percentage = flag.enabled_for_percentage;
                                            let flag_user_count = flag.enabled_for_users.len();
                                            let flag_org_count = flag.enabled_for_orgs.len();
                                            let flag_kind = if flag.kind.is_empty() { "release".to_string() } else { flag.kind.clone() };
                                            let flag_owner = if flag.owner.is_empty() { "unassigned".to_string() } else { flag.owner.clone() };
                                            let rollout_text = format!("{}% | Users: {} | Orgs: {}", flag_percentage, flag_user_count, flag_org_count);
                                            let status_text = if flag_enabled { "Enabled".to_string() } else { "Disabled".to_string() };
                                            view! {
                                                <tr class="hover:bg-gray-50 dark:hover:bg-gray-750">
                                                    <td class="px-4 py-3 text-sm font-medium text-gray-900 dark:text-gray-100">{flag_name}</td>
                                                    <td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-400">{flag_description}</td>
                                                    <td class="px-4 py-3">
                                                        <Badge
                                                            color=match flag_kind.as_str() {
                                                                "permission" => BadgeColor::Warning,
                                                                "experiment" => BadgeColor::Info,
                                                                _ => BadgeColor::Neutral,
                                                            }
                                                            text=flag_kind
                                                        />
                                                    </td>
                                                    <td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-400">{flag_owner}</td>
                                                    <td class="px-4 py-3">
                                                        <Badge
                                                            color=if flag_enabled { BadgeColor::Success } else { BadgeColor::Neutral }
                                                            text=status_text
                                                        />
                                                    </td>
                                                    <td class="px-4 py-3 text-xs text-gray-400 dark:text-gray-500">
                                                        {rollout_text}
                                                    </td>
                                                    <td class="px-4 py-3 text-right">
                                                        <Button
                                                            variant=ButtonVariant::Secondary
                                                            on:click=move |_| toggle_flag.run(flag_id.clone())
                                                        >
                                                            "Toggle"
                                                        </Button>
                                                    </td>
                                                </tr>
                                            }
                                        }
                                    </For>
                                </tbody>
                            </table>
                        </div>
                    </Card>

                    <Card>
                        <div class="space-y-3">
                            <div class="flex items-center justify-between">
                                <h2 class="text-lg font-semibold">"Health-Gated Rollouts"</h2>
                                <Show when=move || rollouts.get().is_some() fallback=|| ()>
                                    {move || {
                                        let r = rollouts.get().expect("guarded by Show");
                                        view! {
                                            <Badge
                                                color=if r.controller_enabled { BadgeColor::Success } else { BadgeColor::Neutral }
                                                text=if r.controller_enabled { "controller running".to_string() } else { "controller disabled".to_string() }
                                            />
                                        }
                                    }}
                                </Show>
                            </div>
                            <div class="overflow-x-auto">
                                <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                                    <thead class="bg-gray-50 dark:bg-gray-800">
                                        <tr>
                                            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Flag"</th>
                                            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Exposure"</th>
                                            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Stage"</th>
                                            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Bad windows"</th>
                                            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Last decision"</th>
                                            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Evidence"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
                                        <For each=move || rollouts.get().map(|r| r.rollouts).unwrap_or_default() key=|r| r.flag_id.clone() let:row>
                                        {
                                            let decision_color = match row.last_decision.as_str() {
                                                "rollback" => BadgeColor::Danger,
                                                "hold" => BadgeColor::Warning,
                                                "promote" | "completed" => BadgeColor::Success,
                                                _ => BadgeColor::Neutral,
                                            };
                                            let evidence = format!(
                                                "error rate {} | p99 {} ms",
                                                row.last_error_rate.map(|r| format!("{:.2}%", r * 100.0)).unwrap_or_else(|| "n/a".into()),
                                                row.last_latency_p99_ms.map(|p| format!("{:.0}", p)).unwrap_or_else(|| "n/a".into()),
                                            );
                                            view! {
                                                <tr class="hover:bg-gray-50 dark:hover:bg-gray-750">
                                                    <td class="px-4 py-3 text-sm font-medium text-gray-900 dark:text-gray-100">{row.name.clone()}</td>
                                                    <td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-400">{format!("{}%", row.enabled_for_percentage)}</td>
                                                    <td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-400">{format!("{}", row.stage_index)}</td>
                                                    <td class="px-4 py-3 text-sm" class=("text-red-600", row.consecutive_failures > 0)>{format!("{}", row.consecutive_failures)}</td>
                                                    <td class="px-4 py-3">
                                                        <Badge color=decision_color text=row.last_decision.clone() />
                                                    </td>
                                                    <td class="px-4 py-3 text-xs text-gray-500 dark:text-gray-400">{evidence}</td>
                                                </tr>
                                            }
                                        }
                                        </For>
                                    </tbody>
                                </table>
                            </div>
                            <div>
                                <h3 class="text-sm font-semibold text-gray-700 dark:text-gray-300">"Recent decisions"</h3>
                                <ul class="mt-1 space-y-1 text-xs text-gray-500 dark:text-gray-400">
                                    <For each=move || rollouts.get().map(|r| r.recent_events).unwrap_or_default() key=|e| format!("{:?}-{}", e.created_at, e.name.clone()) let:ev>
                                        <li>
                                            {format!("[{}] {} {}: {} → {} ({})", ev.created_at.clone().unwrap_or_default(), ev.name, ev.decision, ev.percentage_before, ev.percentage_after, ev.reason)}
                                        </li>
                                    </For>
                                </ul>
                            </div>
                        </div>
                    </Card>

                    <Card>
                        <div class="space-y-3">
                            <div class="flex items-center justify-between">
                                <h2 class="text-lg font-semibold">"Flag Lifecycle Audit"</h2>
                                <Show when=move || audit.get().is_some() fallback=|| ()>
                                    {move || {
                                        let a = audit.get().expect("guarded by Show");
                                        view! {
                                            <div class="flex gap-2 text-xs">
                                                <Badge color=BadgeColor::Danger text=format!("{} removal candidates", a.removal_candidates) />
                                                <Badge color=BadgeColor::Warning text=format!("{} aging", a.aging) />
                                                <Badge color=BadgeColor::Neutral text=format!("{} total", a.total) />
                                            </div>
                                        }
                                    }}
                                </Show>
                            </div>
                            <p class="text-xs text-gray-500 dark:text-gray-400">
                                "Verdicts from the shared staleness policy (flag-kit). Removal candidates are flags past their lifecycle deadline, fully rolled out but still present, or never evaluated. Permission gates are permanently exempt."
                            </p>
                            <div class="overflow-x-auto">
                                <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                                    <thead class="bg-gray-50 dark:bg-gray-800">
                                        <tr>
                                            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Flag"</th>
                                            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Kind"</th>
                                            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Owner"</th>
                                            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Age (days)"</th>
                                            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Verdict"</th>
                                            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Evidence"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
                                        <For each=move || audit.get().map(|a| a.flags).unwrap_or_default() key=|v| v.id.clone() let:entry>
                                        {
                                            let kind = if entry.kind.is_empty() { "release".to_string() } else { entry.kind.clone() };
                                            let owner = if entry.owner.is_empty() { "unassigned".to_string() } else { entry.owner.clone() };
                                            let evidence = if entry.signals.is_empty() {
                                                "-".to_string()
                                            } else {
                                                entry.signals.iter().map(|s| s.detail.as_str()).collect::<Vec<_>>().join("; ")
                                            };
                                            let signals = entry.signals.clone();
                                            let verdict_color = match entry.staleness.as_str() {
                                                "stale" => BadgeColor::Danger,
                                                "aging" => BadgeColor::Warning,
                                                "permanent" => BadgeColor::Info,
                                                _ => BadgeColor::Success,
                                            };
                                            let _ = &signals;
                                            view! {
                                                <tr class="hover:bg-gray-50 dark:hover:bg-gray-750">
                                                    <td class="px-4 py-3 text-sm font-medium text-gray-900 dark:text-gray-100">{entry.name.clone()}</td>
                                                    <td class="px-4 py-3 text-xs text-gray-500 dark:text-gray-400">{kind}</td>
                                                    <td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-400">{owner}</td>
                                                    <td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-400">{entry.age_days.to_string()}</td>
                                                    <td class="px-4 py-3">
                                                        <Badge color=verdict_color text=entry.staleness.clone() />
                                                    </td>
                                                    <td class="px-4 py-3 text-xs text-gray-500 dark:text-gray-400">{evidence}</td>
                                                </tr>
                                            }
                                        }
                                        </For>
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    </Card>
                </Show>
            </Show>
        </div>
    }
}

#[component]
pub fn AdminDashboardPage() -> impl IntoView {
    let auth = use_auth();
    let (widgets, set_widgets) = signal(Vec::<AdminWidget>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    let load_widgets = Callback::new(move |_| {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|s| s.token.clone());
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            match client.get("/admin/dashboard").await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<AdminDashboardResponse>().await {
                        Ok(data) => {
                            set_widgets.set(data.widgets);
                            set_loading.set(false);
                        }
                        Err(e) => {
                            set_error.set(Some(format!("Failed to parse response: {e}")));
                            set_loading.set(false);
                        }
                    }
                }
                Ok(resp) => {
                    set_error.set(Some(format!("Error: {}", resp.status())));
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(format!("Request failed: {e}")));
                    set_loading.set(false);
                }
            }
        });
    });

    load_widgets.run(());

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold">"Admin Dashboard"</h1>
            </div>

            <Show when=move || !loading.get() fallback=|| view! { <Spinner /> }>
                <Show when=move || error.get().is_none() fallback=move || {
                    let err_msg = error.get().unwrap_or_default();
                    view! { <ErrorBanner message=move || err_msg.clone() /> }
                }>
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        <For each=move || widgets.get() key=|w| w.id.clone() let:widget>
                            {
                                view! {
                                    <Card>
                                        <div class="space-y-2">
                                            <div class="flex items-center justify-between">
                                                <h3 class="font-semibold">{widget.widget_name.clone()}</h3>
                                                <Badge
                                                    color=if widget.enabled { BadgeColor::Success } else { BadgeColor::Neutral }
                                                    text=if widget.enabled { "Active".to_string() } else { "Disabled".to_string() }
                                                />
                                            </div>
                                            <div class="text-xs text-gray-400">
                                                "Position: " {widget.position.to_string()}
                                            </div>
                                            <div class="text-sm text-gray-600">
                                                {if widget.widget_config == serde_json::json!({}) {
                                                    "No configuration".to_string()
                                                } else {
                                                    serde_json::to_string_pretty(&widget.widget_config).unwrap_or_default()
                                                }}
                                            </div>
                                        </div>
                                    </Card>
                                }
                            }
                        </For>
                    </div>
                </Show>
            </Show>
        </div>
    }
}
