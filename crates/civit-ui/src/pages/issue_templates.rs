#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::api::types::CreateIssueBody;
use crate::components::{Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Spinner};
use crate::state::auth::use_auth;
use crate::utils::*;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
struct IssueTemplate {
    id: String,
    name: String,
    description: String,
    title: String,
    body: String,
    labels: Vec<String>,
    custom_fields: Vec<CustomField>,
    required_fields: Vec<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
struct CustomField {
    name: String,
    label: String,
    field_type: String,
    required: bool,
    default_value: Option<String>,
    options: Option<Vec<String>>,
}

fn field_type_icon(field_type: &str) -> &'static str {
    match field_type {
        "text" => "T",
        "textarea" => "¶",
        "select" => "▾",
        "checkbox" => "X",
        "number" => "#",
        "date" => "D",
        _ => "?",
    }
}

fn demo_templates() -> Vec<IssueTemplate> {
    vec![
        IssueTemplate {
            id: "tpl-bug".into(),
            name: "Bug Report".into(),
            description: "Report a bug or unexpected behavior".into(),
            title: "[Bug] ".into(),
            body: "## Description\n\n## Steps to Reproduce\n\n## Expected Behavior\n\n## Actual Behavior\n".into(),
            labels: vec!["bug".into()],
            custom_fields: vec![
                CustomField {
                    name: "severity".into(),
                    label: "Severity".into(),
                    field_type: "select".into(),
                    required: true,
                    default_value: None,
                    options: Some(vec!["Low".into(), "Medium".into(), "High".into(), "Critical".into()]),
                },
            ],
            required_fields: vec!["title".into(), "severity".into()],
            created_at: "2025-01-01T00:00:00Z".into(),
            updated_at: "2025-01-01T00:00:00Z".into(),
        },
        IssueTemplate {
            id: "tpl-feature".into(),
            name: "Feature Request".into(),
            description: "Suggest a new feature or enhancement".into(),
            title: "[Feature] ".into(),
            body: "## Summary\n\n## Motivation\n\n## Proposed Solution\n".into(),
            labels: vec!["enhancement".into()],
            custom_fields: vec![],
            required_fields: vec!["title".into()],
            created_at: "2025-01-01T00:00:00Z".into(),
            updated_at: "2025-01-01T00:00:00Z".into(),
        },
    ]
}

#[component]
pub fn IssueTemplatesPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let auth = use_auth();

    let (templates, set_templates) = signal(demo_templates());
    let (loading, set_loading) = signal(true);
    let (show_form, set_show_form) = signal(false);
    let (selected_template_id, set_selected_template_id) = signal(None::<String>);
    let (show_management, set_show_management) = signal(false);

    // Template form state
    let (form_title, set_form_title) = signal(String::new());
    let (form_body, set_form_body) = signal(String::new());
    let (form_field_values, set_form_field_values) = signal(std::collections::HashMap::<String, String>::new());
    let (submitting, set_submitting) = signal(false);
    let (submit_error, set_submit_error) = signal(None::<String>);

    // Management form state
    let (mgmt_name, set_mgmt_name) = signal(String::new());
    let (mgmt_desc, set_mgmt_desc) = signal(String::new());
    let (show_create_form, set_show_create_form) = signal(false);

    let fetch_templates = move || {
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        set_loading.set(true);

        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/issue-templates");
            if let Ok(resp) = client.get(&path).await {
                if resp.status().is_success() {
                    if let Ok(data) = resp.json::<Vec<IssueTemplate>>().await {
                        set_templates.set(data);
                    }
                }
            }
            set_loading.set(false);
        });
    };

    leptos::task::spawn_local(async move {
        fetch_templates();
    });

    let select_template = move |template_id: String| {
        let tmpl = templates.get().iter().find(|t| t.id == template_id).cloned();
        if let Some(tmpl) = tmpl {
            set_form_title.set(tmpl.title.clone());
            set_form_body.set(tmpl.body.clone());
            let mut field_vals = std::collections::HashMap::new();
            for cf in &tmpl.custom_fields {
                if let Some(default) = &cf.default_value {
                    field_vals.insert(cf.name.clone(), default.clone());
                }
            }
            set_form_field_values.set(field_vals);
            set_selected_template_id.set(Some(template_id));
            set_show_form.set(true);
        }
    };

    let handle_submit_issue = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_submit_error.set(None);

        let title_val = form_title.get();
        if title_val.trim().is_empty() {
            set_submit_error.set(Some("Title is required.".into()));
            return;
        }

        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        let body_val = form_body.get();

        set_submitting.set(true);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/issues");
            let body = CreateIssueBody {
                title: title_val.trim().to_string(),
                description: if body_val.trim().is_empty() {
                    None
                } else {
                    Some(body_val.trim().to_string())
                },
            };
            match client.post(&path, &body).await {
                Ok(resp) if resp.status().is_success() => {
                    set_show_form.set(false);
                    set_selected_template_id.set(None);
                }
                Ok(_) => {
                    set_submit_error.set(Some("Failed to create issue.".into()));
                }
                Err(_) => {
                    set_submit_error.set(Some("Network error.".into()));
                }
            }
            set_submitting.set(false);
        });
    };

    let handle_create_template = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let tmpl_name = mgmt_name.get();
        if tmpl_name.trim().is_empty() {
            return;
        }
        let new_template = IssueTemplate {
            id: format!("tpl-{}", js_sys::Date::now() as u64),
            name: tmpl_name.trim().to_string(),
            description: mgmt_desc.get(),
            title: String::new(),
            body: String::new(),
            labels: vec![],
            custom_fields: vec![],
            required_fields: vec!["title".into()],
            created_at: "2025-01-01T00:00:00Z".into(),
            updated_at: "2025-01-01T00:00:00Z".into(),
        };
        set_templates.update(|ts| ts.push(new_template));
        set_show_create_form.set(false);
        set_mgmt_name.set(String::new());
        set_mgmt_desc.set(String::new());
    };

    let handle_delete_template = move |template_id: String| {
        set_templates.update(|ts| ts.retain(|t| t.id != template_id));
    };

    let selected_template = move || {
        selected_template_id.get().and_then(|id| {
            templates.get().iter().find(|t| t.id == id).cloned()
        })
    };

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between flex-wrap gap-4">
                <div>
                    <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1">
                        <A href=format!("/repos/{}/{}", owner(), name())>
                            <span class="hover:text-blue-600 dark:hover:text-blue-400">
                                {move || format!("{}/{}", owner(), name())}
                            </span>
                        </A>
                        <span class="hidden sm:inline">"/"</span>
                        <span class="hidden sm:inline text-gray-700 dark:text-gray-300">"Issue Templates"</span>
                    </div>
                    <h1 class="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-gray-100">"Issue Templates"</h1>
                </div>
                <div class="flex gap-2">
                    <Button variant=ButtonVariant::Primary on:click=move |_| {
                        set_show_form.set(false);
                        set_show_management.set(!show_management.get());
                    }>
                        {move || if show_management.get() { "Back to Templates" } else { "Manage Templates" }}
                    </Button>
                </div>
            </div>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="flex items-center justify-center py-12">
                        <Spinner />
                        <span class="ml-3 text-gray-500 dark:text-gray-400">"Loading templates..."</span>
                    </div>
                </Card>
            </Show>

            <Show when=move || !loading.get() fallback=|| view! { <div class="hidden"></div> }>
                // Management view
                {move || if show_management.get() {
                    view! {
                        <div class="space-y-4">
                            <Button variant=ButtonVariant::Primary on:click=move |_| set_show_create_form.set(true)>
                                "Create Template"
                            </Button>

                            <Show when=move || show_create_form.get() fallback=|| view! { <div class="hidden"></div> }>
                                <Card title="New Template".to_string()>
                                    <form on:submit=handle_create_template class="space-y-4">
                                        <div>
                                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Template Name"</label>
                                            <input
                                                type="text"
                                                class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500"
                                                placeholder="e.g. Bug Report"
                                                prop:value=mgmt_name
                                                on:input=move |ev| set_mgmt_name.set(event_target_value(&ev))
                                                required
                                            />
                                        </div>
                                        <div>
                                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Description"</label>
                                            <input
                                                type="text"
                                                class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500"
                                                placeholder="Short description"
                                                prop:value=mgmt_desc
                                                on:input=move |ev| set_mgmt_desc.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div class="flex gap-3">
                                            <Button variant=ButtonVariant::Primary>"Create"</Button>
                                            <Button variant=ButtonVariant::Secondary on:click=move |_| set_show_create_form.set(false)>"Cancel"</Button>
                                        </div>
                                    </form>
                                </Card>
                            </Show>

                            <For
                                each=move || templates.get()
                                key=|t| t.id.clone()
                                children=move |template| {
                                    let tid = template.id.clone();
                                    let tname = template.name.clone();
                                    let tdesc = template.description.clone();
                                    let handle_delete = handle_delete_template.clone();
                                    view! {
                                        <div class="flex items-center justify-between p-4 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg">
                                            <div>
                                                <h4 class="text-sm font-medium text-gray-900 dark:text-gray-100">{tname}</h4>
                                                <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">{tdesc}</p>
                                                <div class="flex flex-wrap gap-1 mt-2">
                                                    {template.labels.iter().map(|l| {
                                                        view! { <Badge color=BadgeColor::Info text=l.clone() /> }
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            </div>
                                            <button
                                                class="text-gray-400 hover:text-red-500 dark:hover:text-red-400 text-sm"
                                                on:click=move |_| handle_delete(tid.clone())
                                            >
                                                "Delete"
                                            </button>
                                        </div>
                                    }
                                }
                            />
                        </div>
                    }.into_any()
                } else if show_form.get() {
                    // Template form view
                    match selected_template() {
                        Some(tmpl) => {
                            let tmpl_name = tmpl.name.clone();
                            view! {
                                <div class="space-y-4">
                                    <Button variant=ButtonVariant::Ghost on:click=move |_| {
                                        set_show_form.set(false);
                                        set_selected_template_id.set(None);
                                    }>
                                        "\u{2190} Back"
                                    </Button>
                                    <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
                                        {format!("New Issue from: {}", tmpl_name)}
                                    </h3>

                                    <form on:submit=handle_submit_issue class="space-y-4">
                                        <Show when=move || submit_error.get().is_some()>
                                            <ErrorBanner
                                                message=move || submit_error.get().unwrap_or_default()
                                                on_dismiss=Callback::new(move |_: ()| set_submit_error.set(None))
                                            />
                                        </Show>

                                        <Card title="Issue Details".to_string()>
                                            <div class="space-y-4">
                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                                        "Title"
                                                        {if tmpl.required_fields.contains(&"title".to_string()) {
                                                            view! { <span class="text-red-500">" *"</span> }.into_any()
                                                        } else {
                                                            ().into_any()
                                                        }}
                                                    </label>
                                                    <input
                                                        type="text"
                                                        class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500"
                                                        prop:value=form_title
                                                        on:input=move |ev| set_form_title.set(event_target_value(&ev))
                                                        required
                                                    />
                                                </div>
                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Description"</label>
                                                    <textarea
                                                        class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500"
                                                        rows="6"
                                                        prop:value=form_body
                                                        on:input=move |ev| set_form_body.set(event_target_value(&ev))
                                                    ></textarea>
                                                </div>
                                            </div>
                                        </Card>

                                        {if !tmpl.custom_fields.is_empty() {
                                            let fields = tmpl.custom_fields.clone();
                                            view! {
                                                <Card title="Custom Fields".to_string()>
                                                    <div class="space-y-4">
                                                        {fields.into_iter().map(|field| {
                                                            let field_name = field.name.clone();
                                                            let field_label = field.label.clone();
                                                            let field_type = field.field_type.clone();
                                                            let field_required = field.required;
                                                            let field_id = format!("cf-{}", field.name);

                                                            view! {
                                                                <div>
                                                                    <label for=field_id.clone() class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                                                        {format!("{} {}", field_type_icon(&field_type), field_label)}
                                                                        {if field_required {
                                                                            view! { <span class="text-red-500">" *"</span> }.into_any()
                                                                        } else {
                                                                            ().into_any()
                                                                        }}
                                                                    </label>
                                                                    <input
                                                                        type="text"
                                                                        id=field_id.clone()
                                                                        class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500"
                                                                        on:input=move |ev| {
                                                                            let val = event_target_value(&ev);
                                                                            set_form_field_values.update(|vals| { vals.insert(field_name.clone(), val); });
                                                                        }
                                                                    />
                                                                </div>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                </Card>
                                            }.into_any()
                                        } else {
                                            ().into_any()
                                        }}

                                        <div class="flex gap-3">
                                            <Button variant=ButtonVariant::Primary disabled=submitting.get()>
                                                {move || if submitting.get() { "Creating..." } else { "Create Issue" }}
                                            </Button>
                                        </div>
                                    </form>
                                </div>
                            }.into_any()
                        }
                        None => view! { <div class="hidden"></div> }.into_any(),
                    }
                } else {
                    // Template list view
                    view! {
                        <div class="space-y-4">
                            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">"Choose a Template"</h3>
                            <For
                                each=move || templates.get()
                                key=|t| t.id.clone()
                                children=move |template| {
                                    let tid = template.id.clone();
                                    let tname = template.name.clone();
                                    let tdesc = template.description.clone();
                                    let tlabels = template.labels.clone();
                                    let select_fn = select_template.clone();
                                    view! {
                                        <div
                                            class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-5 cursor-pointer hover:border-blue-400 dark:hover:border-blue-500 transition-colors"
                                            on:click=move |_| select_fn(tid.clone())
                                        >
                                            <h4 class="text-base font-semibold text-gray-900 dark:text-gray-100">{tname}</h4>
                                            <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{tdesc}</p>
                                            <div class="flex flex-wrap gap-1 mt-3">
                                                {tlabels.into_iter().map(|label| {
                                                    view! { <Badge color=BadgeColor::Info text=label /> }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        </div>
                                    }
                                }
                            />
                        </div>
                    }.into_any()
                }}
            </Show>
        </div>
    }
}
