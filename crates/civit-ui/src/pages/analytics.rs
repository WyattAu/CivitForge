#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::components::{Card, ErrorBanner, Spinner};
use crate::state::auth::use_auth;
use crate::utils::*;

#[derive(Clone, serde::Deserialize)]
struct StarHistory {
    date: String,
    count: i64,
}

#[derive(Clone, serde::Deserialize)]
struct ForkHistory {
    date: String,
    count: i64,
}

#[derive(Clone, serde::Deserialize)]
struct CommitActivity {
    week: String,
    count: i64,
}

#[derive(Clone, serde::Deserialize)]
struct IssueRate {
    opened: i64,
    closed: i64,
}

#[derive(Clone, serde::Deserialize)]
struct LanguageStat {
    name: String,
    percentage: f64,
}

const LANGUAGE_COLORS: &[(&str, &str)] = &[
    ("Rust", "#dea584"),
    ("TypeScript", "#3178c6"),
    ("JavaScript", "#f1e05a"),
    ("Python", "#3572A5"),
    ("Go", "#00ADD8"),
    ("Java", "#b07219"),
    ("C", "#555555"),
    ("C++", "#f34b7d"),
    ("Ruby", "#701516"),
    ("PHP", "#4F5D95"),
    ("Shell", "#89e051"),
    ("Other", "#8b8b8b"),
];

fn language_color(name: &str) -> &'static str {
    LANGUAGE_COLORS
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, c)| *c)
        .unwrap_or("#8b8b8b")
}

fn bar_height(value: i64, max: i64) -> String {
    if max == 0 {
        return "0%".to_string();
    }
    let pct = (value as f64 / max as f64 * 100.0).round() as u32;
    format!("{}%", pct.min(100))
}

#[component]
pub fn AnalyticsPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let auth = use_auth();

    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let (stars, set_stars) = signal(Vec::<StarHistory>::new());
    let (forks, set_forks) = signal(Vec::<ForkHistory>::new());
    let (commits, set_commits) = signal(Vec::<CommitActivity>::new());
    let (issue_rate, set_issue_rate) = signal(None::<IssueRate>);
    let (languages, set_languages) = signal(Vec::<LanguageStat>::new());

    leptos::task::spawn_local(async move {
        let token = auth.0.with(|a| a.token.clone());
        let client = ApiClient::new(token);
        let o = owner();
        let n = name();

        let stars_path = format!("/repos/{o}/{n}/stars-history");
        if let Ok(resp) = client.get(&stars_path).await {
            if resp.status().is_success() {
                if let Ok(data) = resp.json::<Vec<StarHistory>>().await {
                    set_stars.set(data);
                }
            }
        }

        let forks_path = format!("/repos/{o}/{n}/forks-history");
        if let Ok(resp) = client.get(&forks_path).await {
            if resp.status().is_success() {
                if let Ok(data) = resp.json::<Vec<ForkHistory>>().await {
                    set_forks.set(data);
                }
            }
        }

        let commits_path = format!("/repos/{o}/{n}/commit-activity");
        if let Ok(resp) = client.get(&commits_path).await {
            if resp.status().is_success() {
                if let Ok(data) = resp.json::<Vec<CommitActivity>>().await {
                    set_commits.set(data);
                }
            }
        }

        let issues_path = format!("/repos/{o}/{n}/issue-rates");
        if let Ok(resp) = client.get(&issues_path).await {
            if resp.status().is_success() {
                if let Ok(data) = resp.json::<IssueRate>().await {
                    set_issue_rate.set(Some(data));
                }
            }
        }

        let lang_path = format!("/repos/{o}/{n}/languages");
        if let Ok(resp) = client.get(&lang_path).await {
            if resp.status().is_success() {
                if let Ok(data) = resp.json::<Vec<LanguageStat>>().await {
                    set_languages.set(data);
                }
            }
        }

        set_loading.set(false);
    });

    let max_stars = move || stars.get().iter().map(|s| s.count).max().unwrap_or(1);
    let max_forks = move || forks.get().iter().map(|f| f.count).max().unwrap_or(1);
    let max_commits = move || commits.get().iter().map(|c| c.count).max().unwrap_or(1);

    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-2xl sm:text-3xl font-bold font-mono text-gray-900 dark:text-gray-100">"ANALYTICS"</h1>
                <p class="mt-1 text-sm text-gray-500 dark:text-gray-400 font-mono">
                    {move || format!("{}/{}", owner(), name())}
                </p>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_error.set(None)) />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center justify-center py-12"><Spinner /></div>
            </Show>

            <Show when=move || !loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    // Star History
                    <Card title="Stars Over Time".to_string() description="Star count trend".to_string()>
                        {move || {
                            let s = stars.get();
                            if s.is_empty() {
                                view! { <div class="py-8 text-center text-gray-400 dark:text-gray-500 text-sm">"No star data yet"</div> }.into_view()
                            } else {
                                let mx = max_stars();
                                view! {
                                    <div class="flex items-end gap-1 h-40 px-2">
                                        {s.iter().map(|item| {
                                            let h = bar_height(item.count, mx);
                                            view! {
                                                <div class="flex-1 flex flex-col items-center justify-end h-full" title=format!("{}: {}", item.date, item.count)>
                                                    <div class="w-full bg-blue-500 dark:bg-blue-400 rounded-t transition-all" style:height=h></div>
                                                    <span class="text-[10px] text-gray-400 dark:text-gray-500 mt-1 truncate w-full text-center">{item.date.split('-').last().unwrap_or("")}</span>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_view()
                            }
                        }}
                    </Card>

                    // Fork History
                    <Card title="Forks Over Time".to_string() description="Fork count trend".to_string()>
                        {move || {
                            let f = forks.get();
                            if f.is_empty() {
                                view! { <div class="py-8 text-center text-gray-400 dark:text-gray-500 text-sm">"No fork data yet"</div> }.into_view()
                            } else {
                                let mx = max_forks();
                                view! {
                                    <div class="flex items-end gap-1 h-40 px-2">
                                        {f.iter().map(|item| {
                                            let h = bar_height(item.count, mx);
                                            view! {
                                                <div class="flex-1 flex flex-col items-center justify-end h-full" title=format!("{}: {}", item.date, item.count)>
                                                    <div class="w-full bg-green-500 dark:bg-green-400 rounded-t transition-all" style:height=h></div>
                                                    <span class="text-[10px] text-gray-400 dark:text-gray-500 mt-1 truncate w-full text-center">{item.date.split('-').last().unwrap_or("")}</span>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_view()
                            }
                        }}
                    </Card>

                    // Commit Activity
                    <Card title="Commit Activity".to_string() description="Commits per week".to_string()>
                        {move || {
                            let c = commits.get();
                            if c.is_empty() {
                                view! { <div class="py-8 text-center text-gray-400 dark:text-gray-500 text-sm">"No commit data yet"</div> }.into_view()
                            } else {
                                let mx = max_commits();
                                view! {
                                    <div class="flex items-end gap-1 h-40 px-2">
                                        {c.iter().map(|item| {
                                            let h = bar_height(item.count, mx);
                                            view! {
                                                <div class="flex-1 flex flex-col items-center justify-end h-full" title=format!("{}: {} commits", item.week, item.count)>
                                                    <div class="w-full bg-purple-500 dark:bg-purple-400 rounded-t transition-all" style:height=h></div>
                                                    <span class="text-[10px] text-gray-400 dark:text-gray-500 mt-1 truncate w-full text-center">{item.week.split('-').last().unwrap_or("")}</span>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_view()
                            }
                        }}
                    </Card>

                    // Issue/PR Open/Close Rates
                    <Card title="Issue & PR Rates".to_string() description="Open vs close counts".to_string()>
                        {move || {
                            match issue_rate.get() {
                                Some(rate) => {
                                    let total = rate.opened + rate.closed;
                                    let open_pct = if total > 0 { (rate.opened as f64 / total as f64 * 100.0).round() as u32 } else { 0 };
                                    let close_pct = 100 - open_pct;
                                    view! {
                                        <div class="space-y-4 py-2">
                                            <div>
                                                <div class="flex justify-between text-sm mb-1">
                                                    <span class="text-gray-600 dark:text-gray-400">"Opened"</span>
                                                    <span class="font-mono text-gray-900 dark:text-gray-100">{rate.opened} " (" {open_pct} "%)"</span>
                                                </div>
                                                <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3">
                                                    <div class="bg-blue-500 dark:bg-blue-400 h-3 rounded-full transition-all" style:width=format!("{open_pct}%")></div>
                                                </div>
                                            </div>
                                            <div>
                                                <div class="flex justify-between text-sm mb-1">
                                                    <span class="text-gray-600 dark:text-gray-400">"Closed"</span>
                                                    <span class="font-mono text-gray-900 dark:text-gray-100">{rate.closed} " (" {close_pct} "%)"</span>
                                                </div>
                                                <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3">
                                                    <div class="bg-green-500 dark:bg-green-400 h-3 rounded-full transition-all" style:width=format!("{close_pct}%")></div>
                                                </div>
                                            </div>
                                        </div>
                                    }.into_view()
                                }
                                None => {
                                    view! { <div class="py-8 text-center text-gray-400 dark:text-gray-500 text-sm">"No issue data yet"</div> }.into_view()
                                }
                            }
                        }}
                    </Card>

                    // Language Distribution
                    <Card title="Languages".to_string() description="Language distribution".to_string()>
                        {move || {
                            let langs = languages.get();
                            if langs.is_empty() {
                                view! { <div class="py-8 text-center text-gray-400 dark:text-gray-500 text-sm">"No language data"</div> }.into_view()
                            } else {
                                let total: f64 = langs.iter().map(|l| l.percentage).sum();
                                view! {
                                    <div class="space-y-2">
                                        <div class="flex h-4 rounded-full overflow-hidden">
                                            {langs.iter().map(|l| {
                                                let w = if total > 0.0 { l.percentage / total * 100.0 } else { 0.0 };
                                                let color = language_color(&l.name);
                                                view! {
                                                    <div class="h-full transition-all" style:width=format!("{w:.1}%") style:background-color=color title="{} ({:.1}%)", l.name, w></div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                        <div class="flex flex-wrap gap-3 mt-2">
                                            {langs.iter().map(|l| {
                                                let color = language_color(&l.name);
                                                view! {
                                                    <div class="flex items-center gap-1.5 text-xs text-gray-600 dark:text-gray-400">
                                                        <div class="w-2.5 h-2.5 rounded-full" style:background-color=color></div>
                                                        <span>{l.name.clone()}</span>
                                                        <span class="font-mono text-gray-400 dark:text-gray-500">{format!("{:.1}%", l.percentage)}</span>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    </div>
                                }.into_view()
                            }
                        }}
                    </Card>

                    // Contributor Heatmap placeholder
                    <Card title="Contributor Activity".to_string() description="Contribution heatmap".to_string()>
                        <div class="py-8 text-center text-gray-400 dark:text-gray-500 text-sm">
                            "Contribution heatmap — coming soon"
                        </div>
                    </Card>
                </div>
            </Show>
        </div>
    }
}
