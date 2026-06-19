#![forbid(unsafe_code)]

use leptos::prelude::*;

#[derive(Clone, PartialEq)]
pub struct TabItem {
    pub id: String,
    pub label: String,
}

#[component]
pub fn Tabs(
    tabs: Vec<TabItem>,
    #[prop(optional)] active_tab: String,
    #[prop(optional)] on_change: Option<Callback<String>>,
    children: ChildrenFn,
) -> impl IntoView {
    let btn_class = "px-4 py-2 text-sm font-medium border-b-2 transition-colors whitespace-nowrap";
    let active_class = "border-blue-600 text-blue-600 dark:border-blue-400 dark:text-blue-400";
    let inactive_class = "border-transparent text-gray-500 hover:text-gray-700 \
                          hover:border-gray-300 dark:text-gray-400 \
                          dark:hover:text-gray-200 dark:hover:border-gray-600";

    let (active_tab_sig, _) = signal(active_tab);
    let (on_change_sig, _) = signal(on_change);

    view! {
        <div>
            <div class="border-b border-gray-200 dark:border-gray-700">
                <nav class="-mb-px flex space-x-8" role="tablist" aria-label="Tabs">
                    <For each=move || tabs.clone() key=|t| t.id.clone() let:tab>
                        {
                            let active_tab_sig = active_tab_sig;
                            let on_change_sig = on_change_sig;
                            let (tab_id_sig, _) = signal(tab.id.clone());
                            move || {
                                let current = active_tab_sig.get();
                                let active = current == tab.id;
                                let tab_class = format!(
                                    "{} {}",
                                    btn_class,
                                    if active { active_class } else { inactive_class }
                                );
                                let panel_id = format!("tabpanel-{}", tab_id_sig.get());
                                view! {
                                    <button
                                        class=tab_class
                                        role="tab"
                                        id=tab_id_sig.get()
                                        aria-selected=active.to_string()
                                        aria-controls=panel_id
                                        tabindex=if active { "0" } else { "-1" }
                                        on:click=move |_| {
                                            if let Some(cb) = on_change_sig.get() {
                                                cb.run(tab_id_sig.get());
                                            }
                                        }
                                    >
                                        {tab.label.clone()}
                                    </button>
                                }
                            }
                        }
                    </For>
                </nav>
            </div>
            <div class="mt-4" role="tabpanel" aria-label="Tab content">
                {children()}
            </div>
        </div>
    }
}
