#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::api::types::{
    CreateWikiPageBody, UpdateWikiPageBody, WikiPageListItem, WikiPageResponse, WikiRevision,
};
use crate::components::{Button, ButtonVariant, Card, EmptyState, ErrorBanner, Modal, Spinner};
use crate::state::auth::use_auth;
use crate::utils::*;

#[component]
pub fn WikiPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let auth = use_auth();

    let (pages_sig, set_pages) = signal(Vec::<WikiPageListItem>::new());
    let (pages_loading, set_pages_loading) = signal(true);

    let (current_page_sig, set_current_page) = signal(None::<WikiPageResponse>);
    let (page_loading, set_page_loading) = signal(false);

    let (error, set_error) = signal(None::<String>);

    let (show_new_form, set_show_new_form) = signal(false);
    let (editing, set_editing) = signal(false);

    let (submit_error, set_submit_error) = signal(None::<String>);
    let (submitting, set_submitting) = signal(false);

    let (show_history, set_show_history) = signal(false);
    let (history_sig, set_history) = signal(Vec::<WikiRevision>::new());
    let (history_loading, set_history_loading) = signal(false);

    let (show_delete_confirm, set_show_delete_confirm) = signal(false);
    let (deleting, set_deleting) = signal(false);

    let (search_query, set_search_query) = signal(String::new());
    let (searching, set_searching) = signal(false);
    let (search_results, set_search_results) = signal(Vec::<WikiPageListItem>::new());

    let (active_slug, set_active_slug) = signal(String::new());

    let fetch_pages = move || {
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();

        set_pages_loading.set(true);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/wiki");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<Vec<WikiPageListItem>>().await {
                        Ok(data) => set_pages.set(data),
                        Err(_) => set_error.set(Some("Failed to process response.".to_string())),
                    }
                }
                Ok(resp) => {
                    let body = resp.text().await.unwrap_or_default();
                    if body == "[]" {
                        set_pages.set(Vec::new());
                    } else {
                        set_error.set(Some("Failed to load wiki.".to_string()));
                    }
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_pages_loading.set(false);
        });
    };

    leptos::task::spawn_local(async move {
        fetch_pages();
    });

    let fetch_page_content = move |slug: String| {
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();

        set_page_loading.set(true);
        set_current_page.set(None);
        set_editing.set(false);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/wiki/{slug}");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<WikiPageResponse>().await {
                        Ok(data) => set_current_page.set(Some(data)),
                        Err(_) => set_error.set(Some("Failed to process response.".to_string())),
                    }
                }
                Ok(_) => {
                    set_error.set(Some("Failed to load page.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_page_loading.set(false);
        });
    };

    let _slug_effect = Effect::new(move |_| {
        let slug = active_slug.get();
        if !slug.is_empty() {
            fetch_page_content(slug);
        } else {
            set_current_page.set(None);
            set_page_loading.set(false);
        }
    });

    let handle_new_page_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_submit_error.set(None);

        let slug_val = get_input_value("wiki-new-slug");
        let title_val = get_input_value("wiki-new-title");
        let content_val = get_input_value("wiki-new-content");

        if slug_val.trim().is_empty() {
            set_submit_error.set(Some("Slug is required.".to_string()));
            return;
        }
        if title_val.trim().is_empty() {
            set_submit_error.set(Some("Title is required.".to_string()));
            return;
        }

        let body = CreateWikiPageBody {
            slug: slug_val.trim().to_string(),
            title: title_val.trim().to_string(),
            content: content_val,
        };

        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();

        set_submitting.set(true);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/wiki");
            match client.post(&path, &body).await {
                Ok(resp) if resp.status().is_success() => {
                    set_show_new_form.set(false);
                    fetch_pages();
                }
                Ok(_) => {
                    set_submit_error.set(Some("Failed to create page.".to_string()));
                }
                Err(_) => {
                    set_submit_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_submitting.set(false);
        });
    };

    let handle_edit_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_submit_error.set(None);

        let title_val = get_input_value("wiki-edit-title");
        let content_val = get_input_value("wiki-edit-content");

        if title_val.trim().is_empty() {
            set_submit_error.set(Some("Title is required.".to_string()));
            return;
        }

        let slug_val = active_slug.get();
        let body = UpdateWikiPageBody {
            title: Some(title_val.trim().to_string()),
            content: content_val,
        };

        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();

        set_submitting.set(true);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/wiki/{slug_val}");
            match client.put(&path, &body).await {
                Ok(resp) if resp.status().is_success() => {
                    set_editing.set(false);
                }
                Ok(_) => {
                    set_submit_error.set(Some("Failed to update page.".to_string()));
                }
                Err(_) => {
                    set_submit_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_submitting.set(false);
        });
    };

    let handle_delete_click = move |_| {
        set_show_delete_confirm.set(true);
    };

    let confirm_delete = move |_| {
        let slug_val = active_slug.get();
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();

        set_deleting.set(true);
        set_show_delete_confirm.set(false);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/wiki/{slug_val}");
            match client.delete(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    set_current_page.set(None);
                    set_active_slug.set(String::new());
                    fetch_pages();
                }
                Ok(_) => {
                    set_error.set(Some("Failed to delete page.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_deleting.set(false);
        });
    };

    let handle_search = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let query = search_query.get();
        if query.trim().is_empty() {
            set_search_results.set(Vec::new());
            return;
        }

        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();

        set_searching.set(true);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let encoded = query
                .trim()
                .chars()
                .map(|c| match c {
                    ' ' => "+".to_string(),
                    c if c.is_alphanumeric() => c.to_string(),
                    _ => format!("%{:02X}", c as u8),
                })
                .collect::<String>();
            let path = format!("/repos/{owner_val}/{name_val}/wiki/search?q={encoded}");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<Vec<WikiPageListItem>>().await {
                        Ok(data) => set_search_results.set(data),
                        Err(_) => {
                            set_error.set(Some("Failed to process search results.".to_string()))
                        }
                    }
                }
                _ => {
                    set_search_results.set(Vec::new());
                }
            }
            set_searching.set(false);
        });
    };

    let handle_history_click = move |_| {
        let slug_val = active_slug.get();
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();

        set_show_history.set(true);
        set_history_loading.set(true);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/wiki/{slug_val}/history");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<Vec<WikiRevision>>().await {
                        Ok(data) => set_history.set(data),
                        Err(_) => set_error.set(Some("Failed to process history.".to_string())),
                    }
                }
                _ => {
                    set_history.set(Vec::new());
                }
            }
            set_history_loading.set(false);
        });
    };

    let page_has_content = move || {
        current_page_sig
            .get()
            .is_some_and(|p| !p.content.is_empty())
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
                        <span>"/"</span>
                        <span class="text-gray-700 dark:text-gray-300">"Wiki"</span>
                    </div>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Wiki"</h1>
                    <p class="mt-1 text-gray-600 dark:text-gray-400">"Documentation for this repository."</p>
                </div>
                <div class="flex gap-2">
                    <Button
                        variant=ButtonVariant::Primary
                        on:click=move |_| set_show_new_form.set(!show_new_form.get())
                    >
                        {move || if show_new_form.get() { "Cancel" } else { "New Page" }}
                    </Button>
                </div>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_error.set(None)) />
            </Show>

            <Show when=move || show_new_form.get() fallback=|| view! { <div class="hidden"></div> }>
                <Card title="Create New Page".to_string()>
                    <form on:submit=handle_new_page_submit class="space-y-4">
                        <Show when=move || submit_error.get().is_some()>
                            <ErrorBanner message=move || submit_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_submit_error.set(None)) />
                        </Show>
                        <div>
                            <label for="wiki-new-slug" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Slug (URL path)"
                            </label>
                            <input
                                id="wiki-new-slug"
                                type="text"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500"
                                placeholder="getting-started"
                                required
                            />
                        </div>
                        <div>
                            <label for="wiki-new-title" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Title"
                            </label>
                            <input
                                id="wiki-new-title"
                                type="text"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500"
                                placeholder="Getting Started"
                                required
                            />
                        </div>
                        <div>
                            <label for="wiki-new-content" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Content"
                            </label>
                            <textarea
                                id="wiki-new-content"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500"
                                placeholder="Write your wiki content..."
                                rows="8"
                            ></textarea>
                        </div>
                        <Button variant=ButtonVariant::Primary disabled=submitting.get()>
                            {move || if submitting.get() { "Creating..." } else { "Create Page" }}
                        </Button>
                    </form>
                </Card>
            </Show>

            <div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
                <div class="lg:col-span-1">
                    <Card title="Pages".to_string() class="p-0".to_string()>
                        <div class="divide-y divide-gray-100 dark:divide-gray-700">
                            <Show when=move || pages_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                                <div class="p-4 flex items-center justify-center">
                                    <Spinner />
                                </div>
                            </Show>

                            <Show when=move || !pages_loading.get() && pages_sig.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                                <div class="p-4">
                                    <EmptyState
                                        icon=view! {
                                            <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"/>
                                            </svg>
                                        }.into_any()
                                        title="No wiki pages".to_string()
                                        description="Create the first wiki page to document this repository.".to_string()
                                        action_text="New Page".to_string()
                                    />
                                </div>
                            </Show>

                            <For each=move || pages_sig.get() key=|p| p.slug.clone() let:page_item>
                                {
                                    let slug = page_item.slug.clone();
                                    view! {
                                        <button
                                            role="option"
                                            aria-selected=active_slug.get() == slug
                                            class=format!(
                                                "w-full text-left px-4 py-2.5 text-sm transition-colors {}",
                                                if active_slug.get() == slug {
                                                    "bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300 font-medium border-l-2 border-blue-600 dark:border-blue-400"
                                                } else {
                                                    "text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-750 border-l-2 border-transparent"
                                                }
                                            )
                                            on:click=move |_| set_active_slug.set(slug.clone())
                                        >
                                            <div class="truncate">{page_item.title.clone()}</div>
                                            <div class="text-xs text-gray-400 dark:text-gray-500 mt-0.5">
                                                {relative_time(&page_item.updated_at)}
                                            </div>
                                        </button>
                                    }
                                }
                            </For>
                        </div>
                    </Card>
                </div>

                <div class="lg:col-span-3 space-y-6">
                    <form on:submit=handle_search class="flex gap-2">
                        <label for="wiki-search" class="sr-only">"Search wiki"</label>
                        <input
                            id="wiki-search"
                            type="text"
                            class="flex-1 px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500"
                            placeholder="Search wiki..."
                            on:input=move |ev| {
                                set_search_query.set(event_target_value(&ev));
                            }
                            prop:value=search_query.get()
                        />
                        <Button variant=ButtonVariant::Primary disabled=searching.get()>
                            {move || if searching.get() { "..." } else { "Search" }}
                        </Button>
                    </form>

                    <Show when=move || !search_results.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                        <Card title="Search Results".to_string()>
                            <div class="divide-y divide-gray-100 dark:divide-gray-700">
                                <For each=move || search_results.get() key=|p| p.slug.clone() let:result>
                                    {
                                        let slug = result.slug.clone();
                                        view! {
                                            <button
                                                class="w-full text-left py-2 px-1 hover:bg-gray-50 dark:hover:bg-gray-750 rounded transition-colors"
                                                on:click=move |_| set_active_slug.set(slug.clone())
                                            >
                                                <span class="font-medium text-gray-900 dark:text-gray-100">
                                                    {result.title.clone()}
                                                </span>
                                                <span class="text-xs text-gray-400 dark:text-gray-500 ml-2 font-mono">
                                                    {result.slug.clone()}
                                                </span>
                                            </button>
                                        }
                                    }
                                </For>
                            </div>
                        </Card>
                    </Show>

                    <Show when=move || page_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        <Card>
                            <div class="flex items-center justify-center py-12">
                                <Spinner />
                                <span class="ml-3 text-gray-500 dark:text-gray-400">"Loading page..."</span>
                            </div>
                        </Card>
                    </Show>

                    <Show when=move || !page_loading.get() && current_page_sig.get().is_none() fallback=|| view! { <div class="hidden"></div> }>
                        <Card>
                            <EmptyState
                                icon=view! {
                                    <svg class="w-16 h-16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"/>
                                    </svg>
                                }.into_any()
                                title="Select a page".to_string()
                                description="Choose a page from the sidebar to view its content, or create a new one.".to_string()
                                action_text="New Page".to_string()
                                action_href=format!("/repos/{}/{}/wiki", owner(), name())
                            />
                        </Card>
                    </Show>

                    <Show when=move || !page_loading.get() && current_page_sig.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                        <Card>
                            <Show when=move || editing.get() fallback=|| view! { <div class="hidden"></div> }>
                                <div class="space-y-4">
                                    <Show when=move || submit_error.get().is_some()>
                                        <ErrorBanner message=move || submit_error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_submit_error.set(None)) />
                                    </Show>
                                    <form on:submit=handle_edit_submit class="space-y-4">
                                        <div>
                                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                                "Title"
                                            </label>
                                            <input
                                                id="wiki-edit-title"
                                                type="text"
                                                class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                                value=move || current_page_sig.get().map(|p| p.title.clone()).unwrap_or_default()
                                                required
                                            />
                                        </div>
                                        <div>
                                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                                "Content"
                                            </label>
                                            <textarea
                                                id="wiki-edit-content"
                                                class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                                rows="12"
                                            >
                                                {move || current_page_sig.get().map(|p| p.content.clone()).unwrap_or_default()}
                                            </textarea>
                                        </div>
                                        <div class="flex gap-3">
                                            <Button variant=ButtonVariant::Primary disabled=submitting.get()>
                                                {move || if submitting.get() { "Saving..." } else { "Save Changes" }}
                                            </Button>
                                            <Button
                                                variant=ButtonVariant::Secondary
                                                on:click=move |_| set_editing.set(false)
                                            >
                                                "Cancel"
                                            </Button>
                                        </div>
                                    </form>
                                </div>
                            </Show>

                            <Show when=move || !editing.get() fallback=|| view! { <div class="hidden"></div> }>
                                <div class="space-y-4">
                                    <div class="flex items-center justify-between flex-wrap gap-3">
                                        <div>
                                            <h2 class="text-xl font-bold text-gray-900 dark:text-gray-100">
                                                {move || current_page_sig.get().map(|p| p.title.clone()).unwrap_or_default()}
                                            </h2>
                                            <div class="flex items-center gap-3 mt-1 text-sm text-gray-500 dark:text-gray-400">
                                                <span class="font-mono">
                                                    {move || format!("/ {}", current_page_sig.get().map(|p| p.slug.clone()).unwrap_or_default())}
                                                </span>
                                                <span>
                                                    {move || {
                                                        let ts = current_page_sig.get().map(|p| p.updated_at.clone()).unwrap_or_default();
                                                        if ts.is_empty() { String::new() } else { format!("Updated {}", relative_time(&ts)) }
                                                    }}
                                                </span>
                                            </div>
                                        </div>
                                        <div class="flex gap-2">
                                            <Button
                                                variant=ButtonVariant::Secondary
                                                on:click=handle_history_click
                                            >
                                                "History"
                                            </Button>
                                            <Button
                                                variant=ButtonVariant::Secondary
                                                on:click=move |_| set_editing.set(true)
                                            >
                                                "Edit"
                                            </Button>
                                            <Button
                                                variant=ButtonVariant::Danger
                                                on:click=handle_delete_click
                                                disabled=deleting.get()
                                            >
                                                {move || if deleting.get() { "Deleting..." } else { "Delete" }}
                                            </Button>
                                        </div>
                                    </div>

                                    <Show when=page_has_content fallback=|| view! { <div class="hidden"></div> }>
                                        <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
                                            <div class="whitespace-pre-wrap text-sm text-gray-700 dark:text-gray-300 leading-relaxed font-mono bg-gray-50 dark:bg-gray-900/50 rounded-md p-4 overflow-x-auto">
                                                {move || current_page_sig.get().map(|p| p.content.clone()).unwrap_or_default()}
                                            </div>
                                        </div>
                                    </Show>
                                </div>
                            </Show>
                        </Card>
                    </Show>
                </div>
            </div>

            <Modal
                show=show_delete_confirm.get()
                title="Delete Page".to_string()
                on_close=Callback::new(move |_| set_show_delete_confirm.set(false))
            >
                <div class="space-y-4">
                    <p class="text-sm text-gray-600 dark:text-gray-400">
                        "Are you sure you want to delete this page? This action cannot be undone."
                    </p>
                    <div class="flex gap-3 justify-end">
                        <Button
                            variant=ButtonVariant::Secondary
                            on:click=move |_| set_show_delete_confirm.set(false)
                        >
                            "Cancel"
                        </Button>
                        <Button
                            variant=ButtonVariant::Danger
                            on:click=confirm_delete
                        >
                            "Delete"
                        </Button>
                    </div>
                </div>
            </Modal>

            <Modal
                show=show_history.get()
                title="Page History".to_string()
                on_close=Callback::new(move |_| set_show_history.set(false))
            >
                <div class="space-y-3 max-h-96 overflow-y-auto">
                    <Show when=move || history_loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        <div class="flex items-center justify-center py-6">
                            <Spinner />
                        </div>
                    </Show>

                    <Show when=move || !history_loading.get() && history_sig.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                        <p class="text-sm text-gray-500 dark:text-gray-400 text-center py-4">
                            "No history available."
                        </p>
                    </Show>

                    <For each=move || history_sig.get() key=|r| r.revision let:rev>
                        {
                            view! {
                                <div class="flex items-center justify-between py-2 border-b border-gray-100 dark:border-gray-700 last:border-0">
                                    <div>
                                        <span class="text-sm font-medium text-gray-900 dark:text-gray-100">
                                            {format!("Revision #{}", rev.revision)}
                                        </span>
                                        {(!rev.title.is_empty()).then(|| view! {
                                            <span class="text-sm text-gray-600 dark:text-gray-400 ml-2">
                                                {rev.title.clone()}
                                            </span>
                                        })}
                                    </div>
                                    <div class="flex items-center gap-2 text-xs text-gray-400 dark:text-gray-500">
                                        <span class="font-mono">{truncate_uuid(&rev.author, 8)}</span>
                                        <span>{relative_time(&rev.created_at)}</span>
                                    </div>
                                </div>
                            }
                        }
                    </For>
                </div>
            </Modal>
        </div>
    }
}
