#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};

use crate::api::client::ApiClient;
use crate::components::{
    Badge, BadgeColor, Button, ButtonVariant, Card, EmptyState, ErrorBanner, Input, InputType,
    Modal, Spinner,
};
use crate::state::auth::use_auth;

#[derive(Debug, Clone, serde::Deserialize)]
struct ReleaseResponse {
    id: String,
    #[allow(dead_code)]
    repo_id: String,
    tag_name: String,
    name: String,
    body: Option<String>,
    draft: bool,
    prerelease: bool,
    #[allow(dead_code)]
    author_id: String,
    #[allow(dead_code)]
    created_at: String,
    #[allow(dead_code)]
    updated_at: String,
    published_at: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ReleaseAssetResponse {
    id: String,
    release_id: String,
    name: String,
    content_type: String,
    size: i64,
    download_count: i64,
    #[allow(dead_code)]
    author_id: String,
    #[allow(dead_code)]
    created_at: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct CreateReleaseBody {
    tag_name: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    draft: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prerelease: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct CreateAssetBody {
    name: String,
    content_type: String,
    size: i64,
}

fn format_size(bytes: i64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

#[component]
pub fn ReleasesPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let auth = use_auth();
    let _navigate = use_navigate();

    let (releases, set_releases) = signal(Vec::<ReleaseResponse>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (show_create, set_show_create) = signal(false);
    let (creating, set_creating) = signal(false);

    let (selected_release, set_selected_release) = signal(None::<ReleaseResponse>);
    let (show_detail, set_show_detail) = signal(false);
    let (assets, set_assets) = signal(Vec::<ReleaseAssetResponse>::new());
    let (assets_loading, set_assets_loading) = signal(false);

    let (show_upload_asset, set_show_upload_asset) = signal(false);
    let (uploading_asset, set_uploading_asset) = signal(false);

    let (delete_confirm, set_delete_confirm) = signal(None::<String>);
    let (deleting, set_deleting) = signal(false);

    let fetch_releases = move || {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/releases");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<Vec<ReleaseResponse>>().await {
                        Ok(data) => set_releases.set(data),
                        Err(_) => set_error.set(Some("Failed to load releases.".to_string())),
                    }
                }
                Ok(_) => {
                    set_error.set(Some("Failed to load releases.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_loading.set(false);
        });
    };

    fetch_releases();

    let handle_create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);

        let tag = crate::utils::get_input_value("release-tag");
        let release_name = crate::utils::get_input_value("release-name");
        let body = crate::utils::get_input_value("release-body");
        let draft = crate::utils::get_input_value("release-draft");
        let prerelease = crate::utils::get_input_value("release-prerelease");

        if tag.trim().is_empty() {
            set_error.set(Some("Tag name is required.".to_string()));
            return;
        }
        if release_name.trim().is_empty() {
            set_error.set(Some("Release name is required.".to_string()));
            return;
        }

        let body = CreateReleaseBody {
            tag_name: tag.trim().to_string(),
            name: release_name.trim().to_string(),
            body: if body.trim().is_empty() {
                None
            } else {
                Some(body.trim().to_string())
            },
            draft: Some(draft == "on"),
            prerelease: Some(prerelease == "on"),
        };

        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();

        set_creating.set(true);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/releases");
            match client.post(&path, &body).await {
                Ok(resp) if resp.status().is_success() => {
                    set_show_create.set(false);
                    fetch_releases();
                }
                Ok(_) => {
                    set_error.set(Some("Failed to create release.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_creating.set(false);
        });
    };

    let open_detail = move |release: ReleaseResponse| {
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        let release_id = release.id.clone();

        set_selected_release.set(Some(release));
        set_show_detail.set(true);
        set_assets_loading.set(true);

        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/releases/{release_id}/assets");
            match client.get(&path).await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(data) = resp.json::<Vec<ReleaseAssetResponse>>().await {
                        set_assets.set(data);
                    }
                }
                _ => {}
            }
            set_assets_loading.set(false);
        });
    };

    let handle_upload_asset = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);

        let asset_name = crate::utils::get_input_value("asset-name");
        let asset_type = crate::utils::get_input_value("asset-type");

        if asset_name.trim().is_empty() {
            set_error.set(Some("Asset name is required.".to_string()));
            return;
        }

        let release_id = match selected_release.get() {
            Some(r) => r.id,
            None => return,
        };

        let body = CreateAssetBody {
            name: asset_name.trim().to_string(),
            content_type: if asset_type.trim().is_empty() {
                "application/octet-stream".to_string()
            } else {
                asset_type.trim().to_string()
            },
            size: 0,
        };

        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();

        set_uploading_asset.set(true);
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/releases/{release_id}/assets");
            match client.post(&path, &body).await {
                Ok(resp) if resp.status().is_success() => {
                    set_show_upload_asset.set(false);
                    // Reload assets
                    let token2 = auth.0.with(|a| a.token.clone());
                    let client2 = ApiClient::new(token2);
                    let assets_path =
                        format!("/repos/{owner_val}/{name_val}/releases/{release_id}/assets");
                    if let Ok(r) = client2.get(&assets_path).await
                        && let Ok(data) = r.json::<Vec<ReleaseAssetResponse>>().await
                    {
                        set_assets.set(data);
                    }
                }
                Ok(_) => {
                    set_error.set(Some("Failed to upload asset.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_uploading_asset.set(false);
        });
    };

    let request_delete = move |release_id: String| {
        set_delete_confirm.set(Some(release_id));
    };

    let confirm_delete = move |_: leptos::ev::MouseEvent| {
        let release_id = match delete_confirm.get() {
            Some(id) => id,
            None => return,
        };
        set_delete_confirm.set(None);
        set_deleting.set(true);
        set_error.set(None);

        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();

        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/releases/{release_id}");
            match client.delete(&path).await {
                Ok(resp)
                    if resp.status().is_success()
                        || resp.status() == reqwest::StatusCode::NO_CONTENT =>
                {
                    fetch_releases();
                }
                Ok(_) => {
                    set_error.set(Some("Failed to delete release.".to_string()));
                }
                Err(_) => {
                    set_error.set(Some("Network error. Check your connection.".to_string()));
                }
            }
            set_deleting.set(false);
        });
    };

    let owner_disp = move || owner();
    let name_disp = move || name();

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1">
                        <span class="text-gray-700 dark:text-gray-300">
                            {move || format!("{}/{}", owner_disp(), name_disp())}
                        </span>
                        <span>"/"</span>
                        <span class="font-semibold text-gray-900 dark:text-gray-100">"Releases"</span>
                    </div>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Releases"</h1>
                    <p class="mt-1 text-gray-600 dark:text-gray-400">"Manage releases for this repository."</p>
                </div>
                <Button variant=ButtonVariant::Primary on:click=move |_| set_show_create.set(true)>
                    "New Release"
                </Button>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_error.set(None)) />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center justify-center py-12">
                    <Spinner />
                </div>
            </Show>

            <Show when=move || !loading.get() && releases.with(|r| r.is_empty()) && error.get().is_none() fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <EmptyState
                        icon=view! {
                            <svg class="w-16 h-16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z"/>
                            </svg>
                        }.into_any()
                        title="No releases".to_string()
                        description="Create the first release for this repository to distribute versions to your users.".to_string()
                        action_text="New Release".to_string()
                    />
                </Card>
            </Show>

            <Show when=move || !loading.get() && !releases.with(|r| r.is_empty()) fallback=|| view! { <div class="hidden"></div> }>
                <Card>
                    <div class="divide-y divide-gray-100 dark:divide-gray-700">
                        <For each=move || releases.get() key=|r| r.id.clone() let:release>
                            {
                                let rel = release.clone();
                                let rel2 = release.clone();
                                let _rel3 = release.clone();
                                view! {
                                    <div class="py-4 px-1">
                                        <div class="flex items-start justify-between">
                                            <div class="min-w-0 flex-1">
                                                <div class="flex items-center gap-2 flex-wrap">
                                                    <button
                                                        class="text-blue-600 dark:text-blue-400 hover:underline font-mono text-sm font-semibold"
                                                        on:click=move |_| open_detail(rel.clone())
                                                    >
                                                        {release.tag_name.clone()}
                                                    </button>
                                                    <span class="font-medium text-gray-900 dark:text-gray-100">
                                                        {release.name.clone()}
                                                    </span>
                                                    {move || if release.draft {
                                                        view! { <Badge color=BadgeColor::Warning text="Draft".to_string() /> }.into_any()
                                                    } else {
                                                        view! { <div class="hidden"></div> }.into_any()
                                                    }}
                                                    {move || if release.prerelease {
                                                        view! { <Badge color=BadgeColor::Info text="Pre-release".to_string() /> }.into_any()
                                                    } else {
                                                        view! { <div class="hidden"></div> }.into_any()
                                                    }}
                                                </div>
                                                {release.body.as_ref().map(|b| {
                                                    let body_text = if b.len() > 120 {
                                                        format!("{}...", &b[..120])
                                                    } else {
                                                        b.clone()
                                                    };
                                                    view! {
                                                        <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{body_text}</p>
                                                    }
                                                })}
                                                <div class="flex items-center gap-3 mt-2 text-xs text-gray-400 dark:text-gray-500">
                                                    <span>
                                                        {release.published_at.clone().unwrap_or_else(|| release.created_at.clone())}
                                                    </span>
                                                    <span>"by"</span>
                                                    <span class="font-mono">{release.author_id[..8].to_string()}</span>
                                                </div>
                                            </div>
                                            <div class="flex items-center gap-2 shrink-0 ml-4">
                                                <button
                                                    class="text-sm text-red-600 dark:text-red-400 hover:underline"
                                                    on:click=move |_| request_delete(rel2.id.clone())
                                                >
                                                    "Delete"
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                }
                            }
                        </For>
                    </div>
                </Card>
            </Show>

            // -- Create Release Modal --
            <Modal
                show=show_create.get()
                title="Create Release".to_string()
                on_close=Callback::new(move |_: ()| set_show_create.set(false))
            >
                <form on:submit=handle_create class="space-y-4">
                    <Input
                        label="Tag Name"
                        name="release-tag"
                        id="release-tag"
                        input_type=InputType::Text
                        placeholder="v1.0.0"
                        required=true
                    />
                    <Input
                        label="Release Title"
                        name="release-name"
                        id="release-name"
                        input_type=InputType::Text
                        placeholder="Version 1.0.0"
                        required=true
                    />
                    <Input
                        label="Description"
                        name="release-body"
                        id="release-body"
                        input_type=InputType::Textarea
                        placeholder="Release notes..."
                    />
                    <div class="flex items-center gap-4">
                        <label class="flex items-center gap-2 cursor-pointer">
                            <input type="checkbox" name="release-draft" id="release-draft" class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500 dark:border-gray-600" />
                            <span class="text-sm text-gray-700 dark:text-gray-300">"Draft"</span>
                        </label>
                        <label class="flex items-center gap-2 cursor-pointer">
                            <input type="checkbox" name="release-prerelease" id="release-prerelease" class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500 dark:border-gray-600" />
                            <span class="text-sm text-gray-700 dark:text-gray-300">"Pre-release"</span>
                        </label>
                    </div>
                    <div class="flex gap-3 pt-2">
                        <Button variant=ButtonVariant::Primary disabled=creating.get()>
                            {move || if creating.get() { "Creating..." } else { "Create Release" }}
                        </Button>
                        <button
                            type="button"
                            class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900 bg-gray-200 hover:bg-gray-300 text-gray-900 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-100"
                            on:click=move |_| set_show_create.set(false)
                        >
                            "Cancel"
                        </button>
                    </div>
                </form>
            </Modal>

            // -- Release Detail Modal --
            <Modal
                show=show_detail.get()
                title=selected_release.get().map(|r| r.name).unwrap_or_default()
                on_close=Callback::new(move |_: ()| {
                    set_show_detail.set(false);
                    set_assets.set(Vec::new());
                })
            >
                <div class="space-y-4">
                    {move || selected_release.get().map(|r| {
                        view! {
                            <div class="space-y-3">
                                <div class="flex items-center gap-2">
                                    <Badge color=BadgeColor::Neutral text=r.tag_name.clone() />
                                    {move || if r.draft {
                                        view! { <Badge color=BadgeColor::Warning text="Draft".to_string() /> }.into_any()
                                    } else {
                                        view! { <div class="hidden"></div> }.into_any()
                                    }}
                                    {move || if r.prerelease {
                                        view! { <Badge color=BadgeColor::Info text="Pre-release".to_string() /> }.into_any()
                                    } else {
                                        view! { <div class="hidden"></div> }.into_any()
                                    }}
                                </div>
                                {r.body.as_ref().map(|b| {
                                    view! {
                                        <div class="text-sm text-gray-600 dark:text-gray-400 whitespace-pre-wrap">
                                            {b.clone()}
                                        </div>
                                    }
                                })}
                                <div class="text-xs text-gray-400 dark:text-gray-500">
                                    "Published: " {r.published_at.clone().unwrap_or_else(|| r.created_at.clone())}
                                </div>
                            </div>
                        }
                    })}

                    <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
                        <div class="flex items-center justify-between mb-3">
                            <h4 class="font-semibold text-gray-900 dark:text-gray-100">"Assets"</h4>
                            <button
                                class="text-sm text-blue-600 dark:text-blue-400 hover:underline"
                                on:click=move |_| set_show_upload_asset.set(true)
                            >
                                "Upload Asset"
                            </button>
                        </div>

                        {move || if assets_loading.get() {
                            view! {
                                <div class="flex items-center justify-center py-4">
                                    <Spinner />
                                </div>
                            }.into_any()
                        } else if assets.with(|a| a.is_empty()) {
                            view! {
                                <p class="text-sm text-gray-400 dark:text-gray-500 text-center py-4">
                                    "No assets uploaded."
                                </p>
                            }.into_any()
                        } else {
                            view! {
                                <div class="divide-y divide-gray-100 dark:divide-gray-700">
                                    <For each=move || assets.get() key=|a| a.id.clone() let:asset>
                                        {
                                            let asset_name = asset.name.clone();
                                            let asset_size = format_size(asset.size);
                                            let asset_type = asset.content_type.clone();
                                            let download_url = format!(
                                                "/api/v1/releases/{}/assets/{}",
                                                asset.release_id, asset.id
                                            );
                                            let dl_name = asset.name.clone();
                                            view! {
                                                <div class="flex items-center justify-between py-2">
                                                    <div class="min-w-0">
                                                        <span class="text-sm font-medium text-gray-900 dark:text-gray-100">
                                                            {asset_name}
                                                        </span>
                                                        <div class="flex items-center gap-2 text-xs text-gray-400 dark:text-gray-500">
                                                            <span>{asset_size}</span>
                                                            <span>"·"</span>
                                                            <span>{asset_type}</span>
                                                            <span>"·"</span>
                                                            <span>{format!("{} downloads", asset.download_count)}</span>
                                                        </div>
                                                    </div>
                                                    <a
                                                        href=download_url
                                                        download=dl_name
                                                        class="text-sm text-blue-600 dark:text-blue-400 hover:underline shrink-0 ml-4"
                                                    >
                                                        "Download"
                                                    </a>
                                                </div>
                                            }
                                        }
                                    </For>
                                </div>
                            }.into_any()
                        }}
                    </div>

                    <div class="flex gap-3 pt-2">
                        <button
                            class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900 bg-gray-200 hover:bg-gray-300 text-gray-900 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-100"
                            on:click=move |_| {
                                set_show_detail.set(false);
                                set_assets.set(Vec::new());
                            }
                        >
                            "Close"
                        </button>
                    </div>
                </div>
            </Modal>

            // -- Upload Asset Modal --
            <Modal
                show=show_upload_asset.get()
                title="Upload Asset".to_string()
                on_close=Callback::new(move |_: ()| set_show_upload_asset.set(false))
            >
                <form on:submit=handle_upload_asset class="space-y-4">
                    <Input
                        label="File Name"
                        name="asset-name"
                        id="asset-name"
                        input_type=InputType::Text
                        placeholder="binary.tar.gz"
                        required=true
                    />
                    <Input
                        label="Content Type"
                        name="asset-type"
                        id="asset-type"
                        input_type=InputType::Text
                        placeholder="application/gzip"
                    />
                    <div class="flex gap-3 pt-2">
                        <Button variant=ButtonVariant::Primary disabled=uploading_asset.get()>
                            {move || if uploading_asset.get() { "Uploading..." } else { "Upload" }}
                        </Button>
                        <button
                            type="button"
                            class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900 bg-gray-200 hover:bg-gray-300 text-gray-900 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-100"
                            on:click=move |_| set_show_upload_asset.set(false)
                        >
                            "Cancel"
                        </button>
                    </div>
                </form>
            </Modal>

            // -- Delete Confirmation Modal --
            <Modal
                show=delete_confirm.get().is_some()
                title="Delete Release".to_string()
                on_close=Callback::new(move |_: ()| set_delete_confirm.set(None))
            >
                <div class="space-y-4">
                    <p class="text-sm text-gray-600 dark:text-gray-400">
                        "Are you sure you want to delete this release? This action cannot be undone."
                    </p>
                    <div class="flex gap-3 pt-2">
                        <button
                            class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900 bg-red-600 hover:bg-red-700 text-white dark:bg-red-500 dark:hover:bg-red-600 disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled=deleting.get()
                            on:click=confirm_delete
                        >
                            {move || if deleting.get() { "Deleting..." } else { "Delete" }}
                        </button>
                        <button
                            class="inline-flex items-center justify-center px-4 py-2 rounded-md text-sm font-medium transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900 bg-gray-200 hover:bg-gray-300 text-gray-900 dark:bg-gray-700 dark:hover:bg-gray-600 dark:text-gray-100"
                            on:click=move |_| set_delete_confirm.set(None)
                        >
                            "Cancel"
                        </button>
                    </div>
                </div>
            </Modal>
        </div>
    }
}
