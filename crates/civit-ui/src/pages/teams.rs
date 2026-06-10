#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::components::{Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Input, InputType, Modal, Spinner};
use crate::state::auth::use_auth;
use crate::utils::get_input_value;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TeamResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub permission_level: String,
    pub org_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TeamMemberResponse {
    pub user_id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub role: String,
    pub joined_at: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct CreateTeamBody {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    permission_level: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct AddMemberBody {
    user_id: String,
}

fn permission_color(level: &str) -> BadgeColor {
    match level {
        "admin" => BadgeColor::Danger,
        "write" => BadgeColor::Warning,
        "read" => BadgeColor::Info,
        _ => BadgeColor::Neutral,
    }
}

#[component]
pub fn TeamsPage() -> impl IntoView {
    let params = use_params_map();
    let org_id = move || params.with(|p| p.get("id").unwrap_or_default());
    let auth = use_auth();

    let (teams, set_teams) = signal(Vec::<TeamResponse>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    let (show_create, set_show_create) = signal(false);
    let (creating, set_creating) = signal(false);

    let (selected_team, set_selected_team) = signal(None::<TeamResponse>);
    let (members, set_members) = signal(Vec::<TeamMemberResponse>::new());
    let (members_loading, set_members_loading) = signal(false);
    let (show_add_member, set_show_add_member) = signal(false);
    let (add_member_loading, set_add_member_loading) = signal(false);

    let fetch_teams = move || {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        let org_val = org_id();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/orgs/{org_val}/teams");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<Vec<TeamResponse>>().await {
                        set_teams.set(data);
                    }
                }
                Ok(_) => {
                    set_error.set(Some("Failed to load teams.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error.".to_string()));
                }
            }
            set_loading.set(false);
        });
    };

    fetch_teams();

    let fetch_members = move |team: TeamResponse| {
        set_members_loading.set(true);
        let token = auth.0.with(|a| a.token.clone());
        let org_val = org_id();
        let team_id = team.id.clone();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/orgs/{org_val}/teams/{team_id}/members");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<Vec<TeamMemberResponse>>().await {
                        set_members.set(data);
                    }
                }
                _ => {
                    set_members.set(Vec::new());
                }
            }
            set_members_loading.set(false);
        });
    };

    let handle_create = move |_: leptos::ev::MouseEvent| {
        let team_name = get_input_value("team-name");
        let team_desc = get_input_value("team-description");
        let team_perm = get_input_value("team-permission");

        if team_name.trim().is_empty() {
            set_error.set(Some("Team name is required.".to_string()));
            return;
        }

        let body = CreateTeamBody {
            name: team_name.trim().to_string(),
            description: if team_desc.trim().is_empty() { None } else { Some(team_desc.trim().to_string()) },
            permission_level: if team_perm.is_empty() { "read".to_string() } else { team_perm },
        };

        let token = auth.0.with(|a| a.token.clone());
        let org_val = org_id();
        set_creating.set(true);
        set_error.set(None);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/orgs/{org_val}/teams");
            match client.post(&path, &body).await {
                Ok(resp) if resp.status().is_success() => {
                    set_show_create.set(false);
                    fetch_teams();
                }
                Ok(_) => {
                    set_error.set(Some("Failed to create team.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error.".to_string()));
                }
            }
            set_creating.set(false);
        });
    };

    let handle_add_member = move |team_id: String| {
        let user_id = get_input_value("add-member-id");
        if user_id.trim().is_empty() {
            set_error.set(Some("User ID is required.".to_string()));
            return;
        }

        let body = AddMemberBody { user_id: user_id.trim().to_string() };
        let token = auth.0.with(|a| a.token.clone());
        let org_val = org_id();
        set_add_member_loading.set(true);
        set_error.set(None);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/orgs/{org_val}/teams/{team_id}/members");
            match client.post(&path, &body).await {
                Ok(resp) if resp.status().is_success() => {
                    set_show_add_member.set(false);
                    if let Some(team) = selected_team.get() {
                        fetch_members(team);
                    }
                }
                Ok(_) => {
                    set_error.set(Some("Failed to add member.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error.".to_string()));
                }
            }
            set_add_member_loading.set(false);
        });
    };

    let remove_member = move |team_id: String, user_id: String| {
        let token = auth.0.with(|a| a.token.clone());
        let org_val = org_id();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/orgs/{org_val}/teams/{team_id}/members/{user_id}");
            let _ = client.delete(&path).await;
            if let Some(team) = selected_team.get_untracked() {
                fetch_members(team);
            }
        });
    };

    let dismiss_error = Callback::new(move |_: ()| set_error.set(None));

    let select_team = move |team: TeamResponse| {
        let t = team.clone();
        set_selected_team.set(Some(team));
        fetch_members(t);
    };

    let org_v = org_id();

    view! {
        <div class="space-y-6">
            <div class="flex items-start justify-between">
                <div>
                    <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1">
                        <span class="text-gray-700 dark:text-gray-300 font-mono">{format!("org/{org_v}")}</span>
                    </div>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Teams"</h1>
                    <p class="mt-1 text-gray-600 dark:text-gray-400">"Manage teams and their members for this organization."</p>
                </div>
                <button
                    class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors bg-blue-600 hover:bg-blue-700 text-white"
                    on:click=move |_| set_show_create.set(true)
                >
                    "Create Team"
                </button>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=dismiss_error />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center justify-center py-12">
                    <Spinner />
                </div>
            </Show>

            <Show when=move || !loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex gap-6">
                    <div class="w-72 shrink-0">
                        <Card title="Teams".to_string()>
                            <Show when=move || teams.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                                <div class="py-6 text-center text-gray-400 dark:text-gray-500 text-sm">
                                    "No teams yet."
                                </div>
                            </Show>
                            <Show when=move || !teams.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                                <div class="divide-y divide-gray-100 dark:divide-gray-700">
                                    <For each=move || teams.get() key=|t| t.id.clone() let:team>
                                        {
                                            let t = team.clone();
                                            let selected = move || selected_team.get().map(|s| s.id.clone()) == Some(team.id.clone());
                                            view! {
                                                <button
                                                    class=move || {
                                                        let base = "w-full text-left px-3 py-2 text-sm font-medium transition-colors";
                                                        if selected() {
                                                            format!("{base} bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-gray-100")
                                                        } else {
                                                            format!("{base} text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-750")
                                                        }
                                                    }
                                                    on:click=move |_| select_team(t.clone())
                                                >
                                                    <div class="flex items-center justify-between">
                                                        <span>{team.name.clone()}</span>
                                                        <Badge color=permission_color(&team.permission_level) text=team.permission_level.clone() />
                                                    </div>
                                                </button>
                                            }
                                        }
                                    </For>
                                </div>
                            </Show>
                        </Card>
                    </div>

                    <div class="flex-1 min-w-0">
                        <Show
                            when=move || selected_team.get().is_some()
                            fallback=|| view! {
                                <Card>
                                    <div class="py-12 text-center text-gray-400 dark:text-gray-500">
                                        "Select a team to view members."
                                    </div>
                                </Card>
                            }
                        >
                            {move || {
                                selected_team.get().map(|team| {
                                    let team_id = StoredValue::new(team.id.clone());
                                    view! {
                                        <Card title={team.name.clone()} description={team.description.clone().unwrap_or_default()}>
                                            <div class="flex items-center justify-between mb-4">
                                                <Badge color=permission_color(&team.permission_level) text=format!("Permission: {}", team.permission_level) />
                                                <button
                                                    class="inline-flex items-center justify-center px-3 py-1.5 rounded-md text-xs font-medium transition-colors bg-blue-600 hover:bg-blue-700 text-white"
                                                    on:click=move |_| set_show_add_member.set(true)
                                                >
                                                    "Add Member"
                                                </button>
                                            </div>

                                            <Show when=move || members_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                                                <div class="flex items-center justify-center py-6">
                                                    <Spinner />
                                                </div>
                                            </Show>

                                            <Show when=move || !members_loading.get() && members.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                                                <div class="py-6 text-center text-gray-400 dark:text-gray-500 text-sm">
                                                    "No members in this team."
                                                </div>
                                            </Show>

                                            <Show when=move || !members_loading.get() && !members.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                                                <div class="divide-y divide-gray-100 dark:divide-gray-700">
                                                    <For each=move || members.get() key=|m| m.user_id.clone() let:member>
                                                        {
                                                            let uid = member.user_id.clone();
                                                            let tid = team_id.get_value();
                                                            view! {
                                                                <div class="flex items-center justify-between py-3">
                                                                    <div>
                                                                        <span class="text-sm font-medium text-gray-900 dark:text-gray-100">
                                                                            {member.username.clone()}
                                                                        </span>
                                                                        {member.display_name.clone().map(|dn| view! {
                                                                            <span class="text-xs text-gray-500 dark:text-gray-400 ml-2">{dn}</span>
                                                                        })}
                                                                        <div class="text-xs text-gray-400 dark:text-gray-500 mt-0.5">
                                                                            {format!("Joined {}", member.joined_at)}
                                                                        </div>
                                                                    </div>
                                                                    <div class="flex items-center gap-2">
                                                                        <Badge color=permission_color(&member.role) text=member.role.clone() />
                                                                        <button
                                                                            class="text-xs text-red-600 dark:text-red-400 hover:text-red-800 dark:hover:text-red-200 font-medium"
                                                                            on:click=move |_| remove_member(tid.clone(), uid.clone())
                                                                        >
                                                                            "Remove"
                                                                        </button>
                                                                    </div>
                                                                </div>
                                                            }
                                                        }
                                                    </For>
                                                </div>
                                            </Show>
                                        </Card>
                                    }
                                })
                            }}
                        </Show>
                    </div>
                </div>
            </Show>

            <Modal show=show_create.get() title="Create Team".to_string() on_close=Callback::new(move |_: ()| set_show_create.set(false))>
                <div class="space-y-4">
                    <Input label="Team Name" name="team-name" id="team-name" input_type=InputType::Text placeholder="e.g. backend-team" required=true />
                    <Input label="Description" name="team-description" id="team-description" input_type=InputType::Textarea placeholder="What does this team do?" />
                    <Input label="Permission Level" name="team-permission" id="team-permission" input_type=InputType::Select options=vec![("read", "Read"), ("write", "Write"), ("admin", "Admin")] />
                    <div class="flex gap-3 pt-2">
                        <Button variant=ButtonVariant::Primary disabled=creating.get() on:click=handle_create>
                            {move || if creating.get() { "Creating..." } else { "Create Team" }}
                        </Button>
                        <button
                            class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors bg-gray-200 hover:bg-gray-300 text-gray-900 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-100"
                            on:click=move |_| set_show_create.set(false)
                        >
                            "Cancel"
                        </button>
                    </div>
                </div>
            </Modal>

            <Modal show=show_add_member.get() title="Add Team Member".to_string() on_close=Callback::new(move |_: ()| set_show_add_member.set(false))>
                <div class="space-y-4">
                    <Input label="User ID" name="add-member-id" id="add-member-id" input_type=InputType::Text placeholder="Enter user ID to add" required=true />
                    <div class="flex gap-3 pt-2">
                        <Button variant=ButtonVariant::Primary disabled=add_member_loading.get() on:click=move |_| {
                            if let Some(team) = selected_team.get() {
                                handle_add_member(team.id.clone());
                            }
                        }>
                            {move || if add_member_loading.get() { "Adding..." } else { "Add Member" }}
                        </Button>
                        <button
                            class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors bg-gray-200 hover:bg-gray-300 text-gray-900 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-100"
                            on:click=move |_| set_show_add_member.set(false)
                        >
                            "Cancel"
                        </button>
                    </div>
                </div>
            </Modal>
        </div>
    }
}
