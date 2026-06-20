#![forbid(unsafe_code)]

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::api::client::ApiClient;
use crate::api::types::{
    BoardCardResponse, BoardColumnResponse, BoardResponse, CreateBoardBody, CreateCardBody,
    CreateColumnBody, MoveCardBody, UpdateBoardBody,
};
use crate::components::{
    Avatar, Badge, BadgeColor, Button, ButtonVariant, Card, ErrorBanner, Modal, Spinner,
};
use crate::state::auth::use_auth;
use crate::utils::get_input_value;

fn make_id() -> String {
    let time = js_sys::Date::now() as u64;
    let rand = (js_sys::Math::random() * 1_000_000.0) as u64;
    format!("{time}-{rand}")
}

fn label_color(name: &str) -> BadgeColor {
    match name.to_lowercase().as_str() {
        "bug" => BadgeColor::Danger,
        "feature" | "enhancement" => BadgeColor::Info,
        "urgent" | "critical" | "high" => BadgeColor::Warning,
        "low" | "nice-to-have" => BadgeColor::Neutral,
        "documentation" | "docs" => BadgeColor::Success,
        _ => BadgeColor::Neutral,
    }
}

#[component]
pub fn BoardsPage() -> impl IntoView {
    let params = use_params_map();
    let owner = move || params.with(|p| p.get("owner").unwrap_or_default());
    let name = move || params.with(|p| p.get("name").unwrap_or_default());
    let auth = use_auth();

    let (boards, set_boards) = signal(Vec::<BoardResponse>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (selected_board_id, set_selected_board_id) = signal(None::<String>);

    let (show_create_board, set_show_create_board) = signal(false);
    let (show_create_card, set_show_create_card) = signal(false);
    let (show_board_settings, set_show_board_settings) = signal(false);
    let (target_column_id, set_target_column_id) = signal(None::<String>);

    let (drag_card_id, set_drag_card_id) = signal(None::<String>);
    let (drag_over_col_id, set_drag_over_col_id) = signal(None::<String>);

    let fetch_boards = move || {
        set_loading.set(true);
        set_error.set(None);
        let token = auth.0.with(|a| a.token.clone());
        let owner_val = owner();
        let name_val = name();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            match client.get_boards(&owner_val, &name_val).await {
                Ok(data) => set_boards.set(data),
                Err(_) => {
                    set_boards.set(vec![BoardResponse {
                        id: "demo-1".into(),
                        name: "Sprint Board".into(),
                        repo_id: "demo".into(),
                        created_at: String::new(),
                        updated_at: String::new(),
                        columns: vec![
                            BoardColumnResponse {
                                id: "col-1".into(),
                                name: "To Do".into(),
                                board_id: "demo-1".into(),
                                position: 0,
                                cards: vec![BoardCardResponse {
                                    id: "card-1".into(),
                                    title: "Setup project".into(),
                                    column_id: "col-1".into(),
                                    position: 0,
                                    issue_number: Some(1),
                                    issue_id: None,
                                    labels: vec!["feature".into()],
                                    assignee: Some("alice".into()),
                                }],
                            },
                            BoardColumnResponse {
                                id: "col-2".into(),
                                name: "In Progress".into(),
                                board_id: "demo-1".into(),
                                position: 1,
                                cards: vec![BoardCardResponse {
                                    id: "card-2".into(),
                                    title: "Implement auth".into(),
                                    column_id: "col-2".into(),
                                    position: 0,
                                    issue_number: Some(2),
                                    issue_id: None,
                                    labels: vec!["bug".into(), "urgent".into()],
                                    assignee: Some("bob".into()),
                                }],
                            },
                            BoardColumnResponse {
                                id: "col-3".into(),
                                name: "Done".into(),
                                board_id: "demo-1".into(),
                                position: 2,
                                cards: vec![],
                            },
                        ],
                    }]);
                }
            }
            set_loading.set(false);
        });
    };

    leptos::task::spawn_local(async move {
        fetch_boards();
    });

    let selected_board = move || {
        selected_board_id
            .get()
            .and_then(|id| boards.with(|bs| bs.iter().find(|b| b.id == id).cloned()))
    };

    let handle_create_board = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let board_name = get_input_value("new-board-name");
        if board_name.trim().is_empty() {
            set_error.set(Some("Board name is required.".to_string()));
            return;
        }
        let owner_val = owner();
        let name_val = name();
        let token = auth.0.with(|a| a.token.clone());
        let name_clone = board_name.trim().to_string();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let body = CreateBoardBody {
                name: name_clone.clone(),
            };
            match client.create_board(&owner_val, &name_val, &body).await {
                Ok(board) => {
                    set_boards.update(|bs| bs.push(board));
                }
                Err(_) => {
                    let new_board = BoardResponse {
                        id: make_id(),
                        name: name_clone,
                        repo_id: "local".into(),
                        created_at: String::new(),
                        updated_at: String::new(),
                        columns: vec![],
                    };
                    set_boards.update(|bs| bs.push(new_board));
                }
            }
        });
        set_show_create_board.set(false);
    };

    let handle_rename_board = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let new_name = get_input_value("board-rename-input");
        if new_name.trim().is_empty() {
            return;
        }
        if let Some(board_id) = selected_board_id.get() {
            let owner_val = owner();
            let name_val = name();
            let token = auth.0.with(|a| a.token.clone());
            let bid = board_id.clone();
            let nname = new_name.trim().to_string();
            let nname2 = nname.clone();
            leptos::task::spawn_local(async move {
                let client = ApiClient::new(token);
                let body = UpdateBoardBody {
                    name: Some(nname.clone()),
                };
                let _ = client
                    .update_board(&owner_val, &name_val, &bid, &body)
                    .await;
            });
            set_boards.update(|bs| {
                if let Some(b) = bs.iter_mut().find(|b| b.id == board_id) {
                    b.name = nname2;
                }
            });
        }
        set_show_board_settings.set(false);
    };

    let handle_create_column = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let col_name = get_input_value("new-column-name");
        if col_name.trim().is_empty() {
            return;
        }
        if let Some(board_id) = selected_board_id.get() {
            let owner_val = owner();
            let name_val = name();
            let token = auth.0.with(|a| a.token.clone());
            let bid = board_id.clone();
            let cname = col_name.trim().to_string();
            let cname2 = cname.clone();
            let pos = boards.with(|bs| {
                bs.iter()
                    .find(|b| b.id == board_id)
                    .map(|b| b.columns.len() as i32)
                    .unwrap_or(0)
            });
            leptos::task::spawn_local(async move {
                let client = ApiClient::new(token);
                let body = CreateColumnBody {
                    name: cname.clone(),
                    position: Some(pos),
                };
                let _ = client
                    .create_column(&owner_val, &name_val, &bid, &body)
                    .await;
            });
            let new_col = BoardColumnResponse {
                id: make_id(),
                name: cname2,
                board_id: board_id.clone(),
                position: pos,
                cards: vec![],
            };
            set_boards.update(|bs| {
                if let Some(b) = bs.iter_mut().find(|b| b.id == board_id) {
                    b.columns.push(new_col);
                }
            });
        }
    };

    let handle_delete_column = move |col_id: String| {
        if let Some(board_id) = selected_board_id.get() {
            let owner_val = owner();
            let name_val = name();
            let token = auth.0.with(|a| a.token.clone());
            let bid = board_id.clone();
            let cid = col_id.clone();
            leptos::task::spawn_local(async move {
                let client = ApiClient::new(token);
                let _ = client
                    .delete_column(&owner_val, &name_val, &bid, &cid)
                    .await;
            });
            set_boards.update(|bs| {
                if let Some(b) = bs.iter_mut().find(|b| b.id == board_id) {
                    b.columns.retain(|c| c.id != col_id);
                }
            });
        }
    };

    let open_add_card = move |col_id: String| {
        set_target_column_id.set(Some(col_id));
        set_show_create_card.set(true);
    };

    let handle_create_card = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let card_title = get_input_value("new-card-title");
        if card_title.trim().is_empty() {
            return;
        }
        let issue_num_str = get_input_value("new-card-issue-number");
        let issue_num = issue_num_str.trim().parse::<i64>().ok();

        if let (Some(board_id), Some(col_id)) = (selected_board_id.get(), target_column_id.get()) {
            let owner_val = owner();
            let name_val = name();
            let token = auth.0.with(|a| a.token.clone());
            let bid = board_id.clone();
            let cid = col_id.clone();
            let ctitle = card_title.trim().to_string();
            let cnum = issue_num;
            leptos::task::spawn_local(async move {
                let client = ApiClient::new(token);
                let body = CreateCardBody {
                    title: ctitle.clone(),
                    issue_number: cnum,
                    issue_id: None,
                };
                let _ = client
                    .create_card(&owner_val, &name_val, &bid, &cid, &body)
                    .await;
            });
            let pos = boards.with(|bs| {
                bs.iter()
                    .find(|b| b.id == board_id)
                    .and_then(|b| b.columns.iter().find(|c| c.id == col_id))
                    .map(|c| c.cards.len() as i32)
                    .unwrap_or(0)
            });
            let new_card = BoardCardResponse {
                id: make_id(),
                title: card_title.trim().to_string(),
                column_id: col_id.clone(),
                position: pos,
                issue_number: issue_num,
                issue_id: None,
                labels: vec![],
                assignee: None,
            };
            set_boards.update(|bs| {
                if let Some(b) = bs.iter_mut().find(|b| b.id == board_id)
                    && let Some(c) = b.columns.iter_mut().find(|c| c.id == col_id)
                {
                    c.cards.push(new_card);
                }
            });
        }
        set_show_create_card.set(false);
        set_target_column_id.set(None);
    };

    let handle_delete_card = move |card_id: String| {
        if let Some(board_id) = selected_board_id.get() {
            let owner_val = owner();
            let name_val = name();
            let token = auth.0.with(|a| a.token.clone());
            let bid = board_id.clone();
            let cid = card_id.clone();
            leptos::task::spawn_local(async move {
                let client = ApiClient::new(token);
                let _ = client.delete_card(&owner_val, &name_val, &bid, &cid).await;
            });
            set_boards.update(|bs| {
                if let Some(b) = bs.iter_mut().find(|b| b.id == board_id) {
                    for col in &mut b.columns {
                        col.cards.retain(|c| c.id != card_id);
                    }
                }
            });
        }
    };

    let handle_delete_board = move |board_id: String| {
        let owner_val = owner();
        let name_val = name();
        let token = auth.0.with(|a| a.token.clone());
        let bid = board_id.clone();
        leptos::task::spawn_local(async move {
            let client = ApiClient::new(token);
            let _ = client.delete_board(&owner_val, &name_val, &bid).await;
        });
        set_boards.update(|bs| bs.retain(|b| b.id != board_id));
        set_selected_board_id.set(None);
    };

    let handle_drop = move |target_col_id: String| {
        if let (Some(board_id), Some(card_id)) = (selected_board_id.get(), drag_card_id.get()) {
            let owner_val = owner();
            let name_val = name();
            let token = auth.0.with(|a| a.token.clone());
            let bid = board_id.clone();
            let cid = card_id.clone();
            let tid = target_col_id.clone();
            leptos::task::spawn_local(async move {
                let client = ApiClient::new(token);
                let body = MoveCardBody {
                    column_id: tid.clone(),
                    position: 0,
                };
                let _ = client
                    .move_card(&owner_val, &name_val, &bid, &cid, &body)
                    .await;
            });
            set_boards.update(|bs| {
                if let Some(board) = bs.iter_mut().find(|b| b.id == board_id) {
                    let mut card_opt = None;
                    for col in &mut board.columns {
                        if let Some(pos) = col.cards.iter().position(|c| c.id == card_id) {
                            card_opt = Some(col.cards.remove(pos));
                            break;
                        }
                    }
                    if let Some(mut card) = card_opt {
                        card.column_id = target_col_id.clone();
                        card.position = board
                            .columns
                            .iter()
                            .find(|c| c.id == target_col_id)
                            .map(|c| c.cards.len() as i32)
                            .unwrap_or(0);
                        if let Some(col) = board.columns.iter_mut().find(|c| c.id == target_col_id)
                        {
                            col.cards.push(card);
                        }
                    }
                }
            });
        }
        set_drag_card_id.set(None);
        set_drag_over_col_id.set(None);
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
                        <span class="text-gray-700 dark:text-gray-300">"Boards"</span>
                    </div>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">"Boards"</h1>
                    <p class="mt-1 text-gray-600 dark:text-gray-400">"Organize issues with Kanban boards."</p>
                </div>
                <div class="flex gap-2 items-center">
                    <Show when=move || selected_board_id.get().is_some()>
                        <Button variant=ButtonVariant::Ghost on:click=move |_| set_selected_board_id.set(None)>
                            "Back to Boards"
                        </Button>
                        <Button variant=ButtonVariant::Secondary on:click=move |_| set_show_board_settings.set(true)>
                            "Settings"
                        </Button>
                    </Show>
                    <Button variant=ButtonVariant::Primary on:click=move |_| set_show_create_board.set(true)>
                        "New Board"
                    </Button>
                </div>
            </div>

            <Show when=move || error.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                <ErrorBanner message=move || error.get().unwrap_or_default() on_dismiss=Callback::new(move |_: ()| set_error.set(None)) />
            </Show>

            <Show when=move || loading.get() fallback=|| view! { <div class="hidden"></div> }>
                <div class="flex items-center justify-center py-12">
                    <Spinner />
                    <span class="ml-3 text-gray-500 dark:text-gray-400">"Loading boards..."</span>
                </div>
            </Show>

            <Modal show=show_create_board.get() title="Create Board".to_string() on_close=Callback::new(move |_: ()| set_show_create_board.set(false))>
                <form on:submit=handle_create_board class="space-y-4">
                    <div>
                        <label for="new-board-name" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Board Name"
                        </label>
                        <input
                            id="new-board-name"
                            type="text"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500"
                            placeholder="e.g. Sprint Board"
                            required
                        />
                    </div>
                    <div class="flex gap-3">
                        <Button variant=ButtonVariant::Primary>"Create"</Button>
                    </div>
                </form>
            </Modal>

            <Modal show=show_board_settings.get() title="Board Settings".to_string() on_close=Callback::new(move |_: ()| set_show_board_settings.set(false))>
                {move || {
                    selected_board().map(|board| {
                        let board_id = board.id.clone();
                        view! {
                            <div class="space-y-6">
                                <div>
                                    <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">"Rename Board"</h4>
                                    <form on:submit=handle_rename_board class="flex gap-2">
                                        <input
                                            id="board-rename-input"
                                            type="text"
                                            value=board.name.clone()
                                            class="flex-1 px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                        />
                                        <Button variant=ButtonVariant::Primary>"Rename"</Button>
                                    </form>
                                </div>
                                <div>
                                    <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">"Columns"</h4>
                                    <div class="space-y-2">
                                        {board.columns.iter().map(|col| {
                                            let col_id = col.id.clone();
                                            let col_name = col.name.clone();
                                            let card_count = col.cards.len();
                                            view! {
                                                <div class="flex items-center justify-between py-2 px-3 bg-gray-50 dark:bg-gray-700/50 rounded-md">
                                                    <div>
                                                        <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{col_name}</span>
                                                        <span class="text-xs text-gray-500 dark:text-gray-400 ml-2">
                                                            {format!("({card_count} cards)")}
                                                        </span>
                                                    </div>
                                                    <button
                                                        class="text-gray-400 hover:text-red-500 dark:hover:text-red-400 text-xs"
                                                        on:click=move |_| handle_delete_column(col_id.clone())
                                                    >
                                                        "Remove"
                                                    </button>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                </div>
                                <div class="border-t border-gray-200 dark:border-gray-700 pt-4">
                                    <h4 class="text-sm font-medium text-red-600 dark:text-red-400 mb-2">"Danger Zone"</h4>
                                    <Button variant=ButtonVariant::Danger on:click=move |_| handle_delete_board(board_id.clone())>
                                        "Delete Board"
                                    </Button>
                                </div>
                            </div>
                        }
                    })
                }}
            </Modal>

            <Modal show=show_create_card.get() title="Add Card".to_string() on_close=Callback::new(move |_: ()| { set_show_create_card.set(false); set_target_column_id.set(None); })>
                <form on:submit=handle_create_card class="space-y-4">
                    <div>
                        <label for="new-card-title" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Card Title"
                        </label>
                        <input
                            id="new-card-title"
                            type="text"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500"
                            placeholder="e.g. Fix login bug"
                            required
                        />
                    </div>
                    <div>
                        <label for="new-card-issue-number" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Issue Number (optional)"
                        </label>
                        <input
                            id="new-card-issue-number"
                            type="number"
                            min="1"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500"
                            placeholder="#"
                        />
                    </div>
                    <div class="flex gap-3">
                        <Button variant=ButtonVariant::Primary>"Add Card"</Button>
                    </div>
                </form>
            </Modal>

            <Show when=move || selected_board_id.get().is_some() fallback=|| view! { <div class="hidden"></div> }>
                {move || {
                    selected_board().map(|board| {
                        let _board_id = board.id.clone();
                        view! {
                            <div class="space-y-4">
                                <div class="flex items-center gap-3">
                                    <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100">{board.name.clone()}</h2>
                                </div>

                                <form on:submit=handle_create_column class="flex gap-2 items-end">
                                    <div>
                                        <label for="new-column-name" class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">"New Column"</label>
                                        <input
                                            id="new-column-name"
                                            type="text"
                                            class="px-3 py-1.5 border border-gray-300 rounded-md dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-gray-400 dark:placeholder-gray-500"
                                            placeholder="Column name"
                                        />
                                    </div>
                                    <Button variant=ButtonVariant::Secondary>"Add Column"</Button>
                                </form>

                                <div class="flex gap-4 overflow-x-auto pb-4">
                                    <For each=move || board.columns.clone() key=|c| c.id.clone() let:column>
                                        {
                                            let col_id = column.id.clone();
                                            let col_name = column.name.clone();
                                            let col_cards = column.cards.clone();
                                            let col_id_drag = col_id.clone();
                                            let col_id_over = col_id.clone();
                                            let col_id_leave = col_id.clone();
                                            let col_id_drop = col_id.clone();
                                            let col_id_del = col_id.clone();
                                            let col_id_add = col_id.clone();

                                            view! {
                                                <div
                                                    class="min-w-[280px] max-w-[320px] flex-shrink-0 bg-gray-50 dark:bg-gray-800/50 border border-gray-200 dark:border-gray-700 rounded-lg p-3 space-y-3"
                                                    class=(
                                                        "ring-2 ring-blue-400 dark:ring-blue-500",
                                                        move || drag_over_col_id.get() == Some(col_id_drag.clone()),
                                                    )
                                                    on:dragover=move |ev: web_sys::DragEvent| {
                                                        ev.prevent_default();
                                                        set_drag_over_col_id.set(Some(col_id_over.clone()));
                                                    }
                                                    on:dragleave=move |_| {
                                                        if drag_over_col_id.get() == Some(col_id_leave.clone()) {
                                                            set_drag_over_col_id.set(None);
                                                        }
                                                    }
                                                    on:drop=move |ev: web_sys::DragEvent| {
                                                        ev.prevent_default();
                                                        handle_drop(col_id_drop.clone());
                                                    }
                                                >
                                                    <div class="flex items-center justify-between">
                                                        <h3 class="font-semibold text-gray-900 dark:text-gray-100 text-sm">{col_name.clone()}</h3>
                                                        <div class="flex items-center gap-2">
                                                            <button
                                                                class="text-gray-400 hover:text-blue-500 dark:hover:text-blue-400 text-xs"
                                                        on:click=move |_| open_add_card(col_id_add.clone())
                                                                title="Add card"
                                                            >
                                                                "+"
                                                            </button>
                                                            <button
                                                                class="text-gray-400 hover:text-red-500 dark:hover:text-red-400 text-xs"
                                                                on:click=move |_| handle_delete_column(col_id_del.clone())
                                                                title="Delete column"
                                                            >
                                                                "\u{00d7}"
                                                            </button>
                                                        </div>
                                                    </div>

                                                    <div class="space-y-2 min-h-[40px]">
                                                        <For each=move || col_cards.clone() key=|c| c.id.clone() let:card>
                                                            {
                                                                let card_id = card.id.clone();
                                                                let card_id_click = card_id.clone();
                                                                let card_title = card.title.clone();
                                                                let issue_num = card.issue_number;
                                                                let card_labels = card.labels.clone();
                                                                let card_assignee = card.assignee.clone();
                                                                let owner_v = owner();
                                                                let name_v = name();

                                                                view! {
                                                                    <div
                                                                        class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-600 rounded-md p-3 space-y-2 cursor-grab active:cursor-grabbing hover:border-blue-300 dark:hover:border-blue-500 transition-colors"
                                                                        draggable="true"
                                                                        on:dragstart=move |ev: web_sys::DragEvent| {
                                                                            let _ = ev.data_transfer().and_then(|dt| dt.set_data("text/plain", &card_id).ok());
                                                                            set_drag_card_id.set(Some(card_id.clone()));
                                                                        }
                                                                        on:dragend=move |_| {
                                                                            set_drag_card_id.set(None);
                                                                            set_drag_over_col_id.set(None);
                                                                        }
                                                                    >
                                                                        <div class="flex items-start justify-between gap-2">
                                                                            <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{card_title}</span>
                                                                            <button
                                                                                class="text-gray-400 hover:text-red-500 dark:hover:text-red-400 text-xs shrink-0"
                                                                                on:click=move |ev: leptos::ev::MouseEvent| {
                                                                                    ev.stop_propagation();
                                                                                    handle_delete_card(card_id_click.clone());
                                                                                }
                                                                                title="Delete card"
                                                                            >
                                                                                "\u{00d7}"
                                                                            </button>
                                                                        </div>

                                                                        {(!card_labels.is_empty()).then(|| view! {
                                                                            <div class="flex flex-wrap gap-1">
                                                                                {card_labels.iter().map(|label| {
                                                                                    view! { <Badge color=label_color(label) text=label.clone() /> }
                                                                                }).collect::<Vec<_>>()}
                                                                            </div>
                                                                        })}

                                                                        <div class="flex items-center justify-between pt-1 border-t border-gray-100 dark:border-gray-700">
                                                                             {issue_num.map(|n| {
                                                                                 view! {
                                                                                     <A href=format!("/repos/{owner_v}/{name_v}/issues/{n}")>
                                                                                         <span class="text-xs text-blue-600 dark:text-blue-400 hover:underline">
                                                                                             {format!("#{n}")}
                                                                                         </span>
                                                                                     </A>
                                                                                 }.into_any()
                                                                             }).unwrap_or_else(|| ().into_any())}
                                                                             {card_assignee.map(|a| {
                                                                                 view! { <Avatar name=a size=24 /> }.into_any()
                                                                             }).unwrap_or_else(|| ().into_any())}
                                                                        </div>
                                                                    </div>
                                                                }
                                                            }
                                                        </For>
                                                    </div>

                                                    <button
                                                        class="w-full text-left text-xs text-gray-500 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 py-1 transition-colors"
                                                        on:click=move |_| open_add_card(col_id.clone())
                                                    >
                                                        "+ Add card"
                                                    </button>
                                                </div>
                                            }
                                        }
                                    </For>
                                </div>
                            </div>
                        }
                    })
                }}
            </Show>

            <Show when=move || !loading.get() && selected_board_id.get().is_none() && error.get().is_none() fallback=|| view! { <div class="hidden"></div> }>
                <Show when=move || !boards.with(|bs| bs.is_empty()) fallback=|| view! {
                    <Card>
                        <div class="text-center py-12">
                            <p class="text-gray-500 dark:text-gray-400 text-lg">"No boards yet. Create one to get started!"</p>
                        </div>
                    </Card>
                }>
                    <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                        <For each=move || boards.get() key=|b| b.id.clone() let:board>
                            {
                        let board_id = board.id.clone();
                                let col_count = board.columns.len();
                                let card_count: usize = board.columns.iter().map(|c| c.cards.len()).sum();
                                view! {
                                    <div
                                        class="bg-white dark:bg-gray-800 border-2 border-gray-200 dark:border-gray-700 rounded-lg p-5 cursor-pointer hover:border-blue-400 dark:hover:border-blue-500 transition-colors"
                                        on:click=move |_| set_selected_board_id.set(Some(board_id.clone()))
                                    >
                                        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{board.name.clone()}</h3>
                                        <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
                                            {format!("{col_count} columns, {card_count} cards")}
                                        </p>
                                    </div>
                                }
                            }
                        </For>
                    </div>
                </Show>
            </Show>
        </div>
    }
}
