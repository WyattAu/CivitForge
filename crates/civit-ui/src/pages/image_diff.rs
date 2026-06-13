#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::components::{Button, ButtonVariant, Card, Spinner};
use crate::state::auth::use_auth;

#[derive(Debug, Clone, PartialEq)]
enum DiffMode {
    SideBySide,
    Overlay,
    Slider,
}

impl DiffMode {
    fn label(&self) -> &'static str {
        match self {
            DiffMode::SideBySide => "Side by Side",
            DiffMode::Overlay => "Overlay",
            DiffMode::Slider => "Slider",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ImageStatus {
    Loading,
    Loaded,
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
struct ImageInfo {
    url: String,
    status: ImageStatus,
}

#[component]
fn SliderOverlay(
    before_url: Signal<String>,
    after_url: Signal<String>,
    slider_pos: Signal<f64>,
    set_slider_pos: Callback<f64>,
) -> impl IntoView {
    let on_input = move |ev: leptos::ev::InputEvent| {
        if let Ok(val) = event_target_value(&ev).parse::<f64>() {
            set_slider_pos.run(val);
        }
    };

    view! {
        <div class="relative overflow-hidden rounded-lg border-2 border-gray-300 dark:border-gray-600 bg-gray-50 dark:bg-gray-800">
            <div class="relative">
                <img src=move || after_url.get() alt="After" class="w-full h-auto block" />
                <div
                    class="absolute inset-0 overflow-hidden"
                    style:width=move || format!("{}%", slider_pos.get())
                >
                    <img
                        src=move || before_url.get()
                        alt="Before"
                        class="w-full h-auto block"
                        style:width=move || {
                            let pos = slider_pos.get();
                            let div = if pos > 0.01 { pos } else { 1.0 };
                            format!("{}vw", 100.0 / (div / 100.0))
                        }
                    />
                </div>
                <div
                    class="absolute top-0 bottom-0 w-1 bg-blue-500 cursor-ew-resize z-10"
                    style:left=move || format!("{}%", slider_pos.get())
                >
                    <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-8 h-8 bg-blue-500 rounded-full border-2 border-white flex items-center justify-center shadow-lg">
                        <svg class="w-4 h-4 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l4-4 4 4m0 6l-4 4-4-4"/>
                        </svg>
                    </div>
                </div>
            </div>
            <div class="p-3 flex items-center gap-3">
                <span class="text-xs text-gray-500 dark:text-gray-400">"Before"</span>
                <input
                    type="range"
                    min="0"
                    max="100"
                    step="0.5"
                    class="flex-1 accent-blue-500"
                    prop:value=move || slider_pos.get()
                    on:input=on_input
                />
                <span class="text-xs text-gray-500 dark:text-gray-400">"After"</span>
                <span class="text-xs font-mono text-gray-500 dark:text-gray-400 w-12 text-right">
                    {move || format!("{:.0}%", slider_pos.get())}
                </span>
            </div>
        </div>
    }
}

#[component]
pub fn ImageDiffPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let number = move || {
        params.with(|p| {
            p.get("number")
                .and_then(|n| n.parse::<i64>().ok())
                .unwrap_or(0)
        })
    };
    let auth = use_auth();

    let (before_url, set_before_url) = signal(String::new());
    let (after_url, set_after_url) = signal(String::new());
    let (mode, set_mode) = signal(DiffMode::SideBySide);
    let (slider_pos, set_slider_pos) = signal(50.0_f64);
    let (highlight_diff, set_highlight_diff) = signal(true);
    let (loading, set_loading) = signal(false);

    let set_slider_cb = Callback::new(move |val: f64| set_slider_pos.set(val));

    let fetch_images = move || {
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        let number_val = number();
        set_loading.set(true);

        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let path = format!("/repos/{owner_val}/{name_val}/pulls/{number_val}/files");
            if let Ok(resp) = client.get(&path).await {
                if resp.status().is_success() {
                    if let Ok(data) = resp.json::<serde_json::Value>().await {
                        if let Some(files) = data.as_array() {
                            if let Some(first_image) = files.iter().find(|f| {
                                let name = f["path"].as_str().unwrap_or("");
                                name.ends_with(".png")
                                    || name.ends_with(".jpg")
                                    || name.ends_with(".jpeg")
                                    || name.ends_with(".gif")
                                    || name.ends_with(".svg")
                                    || name.ends_with(".webp")
                            }) {
                                let path_str = first_image["path"].as_str().unwrap_or("");
                                let before = format!(
                                    "/repos/{owner_val}/{name_val}/raw/HEAD/{path_str}"
                                );
                                let after = format!(
                                    "/repos/{owner_val}/{name_val}/raw/HEAD/{path_str}"
                                );
                                set_before_url.set(before);
                                set_after_url.set(after);
                            }
                        }
                    }
                }
            }
            set_loading.set(false);
        });
    };

    leptos::task::spawn_local(async move {
        fetch_images();
    });

    let toggle_mode = move |new_mode: DiffMode| {
        set_mode.set(new_mode);
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
                        <A href=format!("/repos/{}/{}/pulls/{}", owner(), name(), number())>
                            <span class="hover:text-blue-600 dark:hover:text-blue-400">
                                {move || format!("PR #{}", number())}
                            </span>
                        </A>
                        <span class="hidden sm:inline">"/"</span>
                        <span class="hidden sm:inline text-gray-700 dark:text-gray-300">"Image Diff"</span>
                    </div>
                    <h1 class="text-2xl sm:text-3xl font-bold text-gray-900 dark:text-gray-100">"Visual Image Diff"</h1>
                </div>
            </div>

            <Card title="Compare Images".to_string()>
                <div class="space-y-4">
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Before Image URL"</label>
                            <input
                                type="text"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500"
                                placeholder="https://example.com/before.png"
                                prop:value=before_url
                                on:input=move |ev: leptos::ev::InputEvent| {
                                    set_before_url.set(event_target_value(&ev));
                                }
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"After Image URL"</label>
                            <input
                                type="text"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500"
                                placeholder="https://example.com/after.png"
                                prop:value=after_url
                                on:input=move |ev: leptos::ev::InputEvent| {
                                    set_after_url.set(event_target_value(&ev));
                                }
                            />
                        </div>
                    </div>

                    <div class="flex flex-wrap items-center gap-3">
                        <span class="text-sm font-medium text-gray-700 dark:text-gray-300">"Mode:"</span>
                        {move || {
                            let current = mode.get();
                            vec![
                                DiffMode::SideBySide,
                                DiffMode::Overlay,
                                DiffMode::Slider,
                            ]
                            .into_iter()
                            .map(|m| {
                                let is_active = m == current;
                                let btn_class = if is_active {
                                    "px-3 py-1.5 text-sm font-medium rounded-md bg-blue-600 text-white"
                                } else {
                                    "px-3 py-1.5 text-sm font-medium rounded-md bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600"
                                };
                                let label = m.label().to_string();
                                view! {
                                    <button
                                        class=btn_class
                                        on:click=move |_| toggle_mode(m.clone())
                                    >
                                        {label}
                                    </button>
                                }
                            })
                            .collect::<Vec<_>>()
                        }}
                        <div class="flex items-center gap-2 ml-4">
                            <input
                                type="checkbox"
                                id="highlight-diff"
                                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                prop:checked=highlight_diff
                                on:change=move |ev: leptos::ev::Event| {
                                    set_highlight_diff.set(event_target_checked(&ev));
                                }
                            />
                            <label for="highlight-diff" class="text-sm text-gray-700 dark:text-gray-300">
                                "Highlight pixel differences"
                            </label>
                        </div>
                    </div>

                    <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                        <div class="flex items-center justify-center py-12">
                            <Spinner />
                            <span class="ml-3 text-gray-500 dark:text-gray-400">"Loading images..."</span>
                        </div>
                    </Show>

                    <Show when=move || !loading.get() && (before_url.get().is_empty() && after_url.get().is_empty()) fallback=|| view! { <div class="hidden"></div> }>
                        <div class="text-center py-12 text-gray-500 dark:text-gray-400">
                            "Enter image URLs above to compare."
                        </div>
                    </Show>

                    <Show when=move || !before_url.get().is_empty() || !after_url.get().is_empty() fallback=|| view! { <div class="hidden"></div> }>
                        {move || match mode.get() {
                            DiffMode::SideBySide => {
                                let b = before_url.get();
                                let a = after_url.get();
                                view! {
                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                        <div class="relative border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg overflow-hidden bg-gray-50 dark:bg-gray-800">
                                            <div class="absolute top-2 left-2 z-10 bg-black/60 text-white text-xs px-2 py-1 rounded">
                                                "Before"
                                            </div>
                                            {if b.is_empty() {
                                                view! { <div class="flex items-center justify-center h-64"><Spinner /></div> }.into_any()
                                            } else {
                                                view! { <img src=b alt="Before" class="w-full h-auto" /> }.into_any()
                                            }}
                                        </div>
                                        <div class="relative border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg overflow-hidden bg-gray-50 dark:bg-gray-800">
                                            <div class="absolute top-2 left-2 z-10 bg-black/60 text-white text-xs px-2 py-1 rounded">
                                                "After"
                                            </div>
                                            {if a.is_empty() {
                                                view! { <div class="flex items-center justify-center h-64"><Spinner /></div> }.into_any()
                                            } else {
                                                view! { <img src=a alt="After" class="w-full h-auto" /> }.into_any()
                                            }}
                                        </div>
                                    </div>
                                }
                            }
                            DiffMode::Overlay => {
                                let a = after_url.get();
                                view! {
                                    <div class="relative border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg overflow-hidden bg-gray-50 dark:bg-gray-800">
                                        <div class="absolute top-2 left-2 z-10 bg-black/60 text-white text-xs px-2 py-1 rounded">
                                            "Overlay"
                                        </div>
                                        {if a.is_empty() {
                                            view! { <div class="flex items-center justify-center h-64"><Spinner /></div> }.into_any()
                                        } else {
                                            view! { <img src=a alt="Overlay" class="w-full h-auto" style="mix-blend-mode:difference;" /> }.into_any()
                                        }}
                                    </div>
                                }
                            }
                            DiffMode::Slider => view! {
                                <SliderOverlay
                                    before_url=before_url
                                    after_url=after_url
                                    slider_pos=slider_pos
                                    set_slider_pos=set_slider_cb
                                />
                            },
                        }}
                    </Show>

                    <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
                        <div class="flex items-center gap-4 text-xs text-gray-500 dark:text-gray-400">
                            <span>"Supported formats: PNG, JPG, GIF, SVG, WebP"</span>
                            <span>"|"</span>
                            <span>"Pixel-level diff highlighting available in Side-by-Side mode"</span>
                        </div>
                    </div>
                </div>
            </Card>
        </div>
    }
}
