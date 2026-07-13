#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::routing::{delete, get, patch, post};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Helper: get repo id
// ---------------------------------------------------------------------------

async fn get_repo_id(pool: &sqlx::PgPool, owner: &str, name: &str) -> Option<uuid::Uuid> {
    sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT r.id FROM repositories r JOIN users u ON r.owner_id = u.id WHERE u.username = $1 AND r.name = $2",
    )
    .bind(owner)
    .bind(name)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

// ---------------------------------------------------------------------------
// Helper: error responses
// ---------------------------------------------------------------------------

fn err_response(status: StatusCode, msg: &str) -> axum::response::Response {
    (
        status,
        Json(CoreError::NotFound(msg.to_string()).error_response()),
    )
        .into_response()
}

fn internal_err(msg: &str) -> axum::response::Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(CoreError::Database(msg.to_string()).error_response()),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Request / Response structs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateBoardRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateColumnRequest {
    pub name: String,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateColumnRequest {
    pub name: Option<String>,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCardRequest {
    pub column_id: uuid::Uuid,
    pub issue_id: Option<uuid::Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct MoveCardRequest {
    pub column_id: Option<uuid::Uuid>,
    pub position: Option<i32>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct AddCardLabelRequest {
    pub label: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddCardAssigneeRequest {
    pub user_id: uuid::Uuid,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CardLabelResponse {
    pub id: uuid::Uuid,
    pub card_id: uuid::Uuid,
    pub label: String,
    pub color: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CardAssigneeResponse {
    pub id: uuid::Uuid,
    pub card_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCardRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub due_date: Option<DateTime<Utc>>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BoardResponse {
    pub id: uuid::Uuid,
    pub repo_id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_by: uuid::Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ColumnResponse {
    pub id: uuid::Uuid,
    pub board_id: uuid::Uuid,
    pub name: String,
    pub position: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CardResponse {
    pub id: uuid::Uuid,
    pub column_id: uuid::Uuid,
    pub issue_id: Option<uuid::Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub position: i32,
    pub priority: i32,
    pub due_date: Option<DateTime<Utc>>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CardDetailResponse {
    pub card: CardResponse,
    pub labels: Vec<CardLabelResponse>,
    pub assignees: Vec<CardAssigneeResponse>,
}

#[derive(Debug, Serialize)]
pub struct BoardDetailResponse {
    pub board: BoardResponse,
    pub columns: Vec<ColumnWithCards>,
}

#[derive(Debug, Serialize)]
pub struct ColumnWithCards {
    pub column: ColumnResponse,
    pub cards: Vec<CardDetailResponse>,
}

// ---------------------------------------------------------------------------
// 1. GET /repos/{owner}/{name}/boards — list boards
// ---------------------------------------------------------------------------

pub async fn list_boards(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    match sqlx::query_as::<_, BoardResponse>(
        "SELECT id, repo_id, name, description, created_by, created_at, updated_at FROM boards WHERE repo_id = $1 ORDER BY created_at DESC",
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await
    {
        Ok(boards) => (StatusCode::OK, Json(boards)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 2. POST /repos/{owner}/{name}/boards — create board with default columns
// ---------------------------------------------------------------------------

pub async fn create_board(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    auth: AuthUser,
    Json(req): Json<CreateBoardRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    if req.name.trim().is_empty() {
        return err_response(StatusCode::BAD_REQUEST, "board name is required");
    }

    let user_id = match uuid::Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => return err_response(StatusCode::UNAUTHORIZED, "invalid user id"),
    };

    let board = match sqlx::query_as::<_, BoardResponse>(
        "INSERT INTO boards (repo_id, name, description, created_by, created_at, updated_at) VALUES ($1, $2, $3, $4, NOW(), NOW()) RETURNING id, repo_id, name, description, created_by, created_at, updated_at",
    )
    .bind(repo_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(user_id)
    .fetch_one(pool)
    .await
    {
        Ok(b) => b,
        Err(e) => return internal_err(&e.to_string()),
    };

    let default_columns = ["To Do", "In Progress", "Review", "Done"];
    for (i, col_name) in default_columns.iter().enumerate() {
        let _ = sqlx::query(
            "INSERT INTO board_columns (board_id, name, position, created_at) VALUES ($1, $2, $3, NOW())",
        )
        .bind(board.id)
        .bind(col_name)
        .bind(i as i32)
        .execute(pool)
        .await;
    }

    (StatusCode::CREATED, Json(board)).into_response()
}

// ---------------------------------------------------------------------------
// 3. GET /repos/{owner}/{name}/boards/{id} — get board with columns + cards
// ---------------------------------------------------------------------------

pub async fn get_board(
    State(state): State<AppState>,
    Path((owner, name, board_id)): Path<(String, String, uuid::Uuid)>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let board = match sqlx::query_as::<_, BoardResponse>(
        "SELECT id, repo_id, name, description, created_by, created_at, updated_at FROM boards WHERE id = $1 AND repo_id = $2",
    )
    .bind(board_id)
    .bind(repo_id)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(b)) => b,
        Ok(None) => {
            return err_response(StatusCode::NOT_FOUND, "board not found");
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    let columns = match sqlx::query_as::<_, ColumnResponse>(
        "SELECT id, board_id, name, position, created_at FROM board_columns WHERE board_id = $1 ORDER BY position",
    )
    .bind(board_id)
    .fetch_all(pool)
    .await
    {
        Ok(c) => c,
        Err(e) => return internal_err(&e.to_string()),
    };

    let mut columns_with_cards = Vec::with_capacity(columns.len());
    for col in columns {
        let raw_cards = sqlx::query_as::<_, CardResponse>(
            "SELECT id, column_id, issue_id, title, description, position, priority, due_date, sort_order, created_at, updated_at FROM board_cards WHERE column_id = $1 ORDER BY sort_order, position",
        )
        .bind(col.id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let mut cards = Vec::with_capacity(raw_cards.len());
        for card in raw_cards {
            let labels = sqlx::query_as::<_, CardLabelResponse>(
                "SELECT id, card_id, label, color FROM board_card_labels WHERE card_id = $1 ORDER BY label",
            )
            .bind(card.id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            let assignees = sqlx::query_as::<_, CardAssigneeResponse>(
                "SELECT id, card_id, user_id FROM board_card_assignees WHERE card_id = $1 ORDER BY user_id",
            )
            .bind(card.id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            cards.push(CardDetailResponse { card, labels, assignees });
        }

        columns_with_cards.push(ColumnWithCards { column: col, cards });
    }

    let resp = BoardDetailResponse {
        board,
        columns: columns_with_cards,
    };

    (StatusCode::OK, Json(resp)).into_response()
}

// ---------------------------------------------------------------------------
// 4. POST /repos/{owner}/{name}/boards/{id}/columns — add column
// ---------------------------------------------------------------------------

pub async fn add_column(
    State(state): State<AppState>,
    Path((owner, name, board_id)): Path<(String, String, uuid::Uuid)>,
    _auth: AuthUser,
    Json(req): Json<CreateColumnRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    // Verify board belongs to repo
    let exists: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM boards WHERE id = $1 AND repo_id = $2")
            .bind(board_id)
            .bind(repo_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    if exists.is_none() {
        return err_response(StatusCode::NOT_FOUND, "board not found");
    }

    let position = match req.position {
        Some(p) => p,
        None => {
            let max_pos: Option<(i32,)> = sqlx::query_as(
                "SELECT COALESCE(MAX(position), -1) FROM board_columns WHERE board_id = $1",
            )
            .bind(board_id)
            .fetch_one(pool)
            .await
            .ok();
            max_pos.map(|(p,)| p + 1).unwrap_or(0)
        }
    };

    match sqlx::query_as::<_, ColumnResponse>(
        "INSERT INTO board_columns (board_id, name, position, created_at) VALUES ($1, $2, $3, NOW()) RETURNING id, board_id, name, position, created_at",
    )
    .bind(board_id)
    .bind(&req.name)
    .bind(position)
    .fetch_one(pool)
    .await
    {
        Ok(col) => (StatusCode::CREATED, Json(col)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 5. PATCH /repos/{owner}/{name}/boards/{id}/columns/{col_id} — update column
// ---------------------------------------------------------------------------

pub async fn update_column(
    State(state): State<AppState>,
    Path((owner, name, board_id, col_id)): Path<(String, String, uuid::Uuid, uuid::Uuid)>,
    _auth: AuthUser,
    Json(req): Json<UpdateColumnRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    // Verify board belongs to repo
    let exists: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM boards WHERE id = $1 AND repo_id = $2")
            .bind(board_id)
            .bind(repo_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    if exists.is_none() {
        return err_response(StatusCode::NOT_FOUND, "board not found");
    }

    let existing = match sqlx::query_as::<_, ColumnResponse>(
        "SELECT id, board_id, name, position, created_at FROM board_columns WHERE id = $1 AND board_id = $2",
    )
    .bind(col_id)
    .bind(board_id)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return err_response(StatusCode::NOT_FOUND, "column not found");
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    let name = req.name.as_deref().unwrap_or(&existing.name);
    let position = req.position.unwrap_or(existing.position);

    match sqlx::query_as::<_, ColumnResponse>(
        "UPDATE board_columns SET name = $1, position = $2 WHERE id = $3 RETURNING id, board_id, name, position, created_at",
    )
    .bind(name)
    .bind(position)
    .bind(col_id)
    .fetch_one(pool)
    .await
    {
        Ok(col) => (StatusCode::OK, Json(col)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 6. DELETE /repos/{owner}/{name}/boards/{id}/columns/{col_id} — delete column
// ---------------------------------------------------------------------------

pub async fn delete_column(
    State(state): State<AppState>,
    Path((owner, name, board_id, col_id)): Path<(String, String, uuid::Uuid, uuid::Uuid)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let exists: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM boards WHERE id = $1 AND repo_id = $2")
            .bind(board_id)
            .bind(repo_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    if exists.is_none() {
        return err_response(StatusCode::NOT_FOUND, "board not found");
    }

    let result = sqlx::query("DELETE FROM board_columns WHERE id = $1 AND board_id = $2")
        .bind(col_id)
        .bind(board_id)
        .execute(pool)
        .await;

    match result {
        Ok(rows) if rows.rows_affected() == 0 => {
            err_response(StatusCode::NOT_FOUND, "column not found")
        }
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 7. POST /repos/{owner}/{name}/boards/{id}/cards — add card
// ---------------------------------------------------------------------------

pub async fn add_card(
    State(state): State<AppState>,
    Path((owner, name, board_id)): Path<(String, String, uuid::Uuid)>,
    _auth: AuthUser,
    Json(req): Json<CreateCardRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let exists: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM boards WHERE id = $1 AND repo_id = $2")
            .bind(board_id)
            .bind(repo_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    if exists.is_none() {
        return err_response(StatusCode::NOT_FOUND, "board not found");
    }

    // Verify column belongs to this board
    let col_exists: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM board_columns WHERE id = $1 AND board_id = $2")
            .bind(req.column_id)
            .bind(board_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    if col_exists.is_none() {
        return err_response(StatusCode::NOT_FOUND, "column not found");
    }

    if req.title.trim().is_empty() {
        return err_response(StatusCode::BAD_REQUEST, "card title is required");
    }

    let position = match req.position {
        Some(p) => p,
        None => {
            let max_pos: Option<(i32,)> = sqlx::query_as(
                "SELECT COALESCE(MAX(position), -1) FROM board_cards WHERE column_id = $1",
            )
            .bind(req.column_id)
            .fetch_one(pool)
            .await
            .ok();
            max_pos.map(|(p,)| p + 1).unwrap_or(0)
        }
    };

    match sqlx::query_as::<_, CardResponse>(
        "INSERT INTO board_cards (column_id, issue_id, title, description, position, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, NOW(), NOW()) RETURNING id, column_id, issue_id, title, description, position, priority, due_date, sort_order, created_at, updated_at",
    )
    .bind(req.column_id)
    .bind(req.issue_id)
    .bind(&req.title)
    .bind(&req.description)
    .bind(position)
    .fetch_one(pool)
    .await
    {
        Ok(card) => (StatusCode::CREATED, Json(card)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 8. PATCH /repos/{owner}/{name}/boards/{id}/cards/{card_id} — move card
// ---------------------------------------------------------------------------

pub async fn move_card(
    State(state): State<AppState>,
    Path((owner, name, board_id, card_id)): Path<(String, String, uuid::Uuid, uuid::Uuid)>,
    _auth: AuthUser,
    Json(req): Json<MoveCardRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let exists: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM boards WHERE id = $1 AND repo_id = $2")
            .bind(board_id)
            .bind(repo_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    if exists.is_none() {
        return err_response(StatusCode::NOT_FOUND, "board not found");
    }

    let existing = match sqlx::query_as::<_, CardResponse>(
        "SELECT id, column_id, issue_id, title, description, position, priority, due_date, sort_order, created_at, updated_at FROM board_cards WHERE id = $1",
    )
    .bind(card_id)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return err_response(StatusCode::NOT_FOUND, "card not found");
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    let column_id = req.column_id.unwrap_or(existing.column_id);
    let position = req.position.unwrap_or(existing.position);
    let sort_order = req.sort_order.unwrap_or(existing.sort_order);

    match sqlx::query_as::<_, CardResponse>(
        "UPDATE board_cards SET column_id = $1, position = $2, sort_order = $3, updated_at = NOW() WHERE id = $4 RETURNING id, column_id, issue_id, title, description, position, priority, due_date, sort_order, created_at, updated_at",
    )
    .bind(column_id)
    .bind(position)
    .bind(sort_order)
    .bind(card_id)
    .fetch_one(pool)
    .await
    {
        Ok(card) => (StatusCode::OK, Json(card)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 9. DELETE /repos/{owner}/{name}/boards/{id}/cards/{card_id} — remove card
// ---------------------------------------------------------------------------

pub async fn delete_card(
    State(state): State<AppState>,
    Path((owner, name, board_id, card_id)): Path<(String, String, uuid::Uuid, uuid::Uuid)>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let exists: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM boards WHERE id = $1 AND repo_id = $2")
            .bind(board_id)
            .bind(repo_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    if exists.is_none() {
        return err_response(StatusCode::NOT_FOUND, "board not found");
    }

    let result = sqlx::query("DELETE FROM board_cards WHERE id = $1")
        .bind(card_id)
        .execute(pool)
        .await;

    match result {
        Ok(rows) if rows.rows_affected() == 0 => {
            err_response(StatusCode::NOT_FOUND, "card not found")
        }
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 10. POST /repos/{owner}/{name}/boards/{id}/cards/{card_id}/labels — add label
// ---------------------------------------------------------------------------

pub async fn add_card_label(
    State(state): State<AppState>,
    Path((owner, name, board_id, card_id)): Path<(String, String, uuid::Uuid, uuid::Uuid)>,
    _auth: AuthUser,
    Json(req): Json<AddCardLabelRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let exists: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM boards WHERE id = $1 AND repo_id = $2")
            .bind(board_id)
            .bind(repo_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    if exists.is_none() {
        return err_response(StatusCode::NOT_FOUND, "board not found");
    }

    let card_exists: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM board_cards WHERE id = $1")
            .bind(card_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    if card_exists.is_none() {
        return err_response(StatusCode::NOT_FOUND, "card not found");
    }

    let color = req.color.unwrap_or_else(|| "#3b82f6".to_string());

    match sqlx::query_as::<_, CardLabelResponse>(
        r#"INSERT INTO board_card_labels (card_id, label, color)
           VALUES ($1, $2, $3)
           ON CONFLICT (card_id, label) DO UPDATE SET color = $3
           RETURNING id, card_id, label, color"#,
    )
    .bind(card_id)
    .bind(&req.label)
    .bind(&color)
    .fetch_one(pool)
    .await
    {
        Ok(label) => (StatusCode::CREATED, Json(label)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 11. DELETE /repos/{owner}/{name}/boards/{id}/cards/{card_id}/labels/{label} — remove label
// ---------------------------------------------------------------------------

pub async fn remove_card_label(
    State(state): State<AppState>,
    Path((owner, name, board_id, card_id, label)): Path<(
        String,
        String,
        uuid::Uuid,
        uuid::Uuid,
        String,
    )>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let exists: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM boards WHERE id = $1 AND repo_id = $2")
            .bind(board_id)
            .bind(repo_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    if exists.is_none() {
        return err_response(StatusCode::NOT_FOUND, "board not found");
    }

    let result = sqlx::query("DELETE FROM board_card_labels WHERE card_id = $1 AND label = $2")
        .bind(card_id)
        .bind(&label)
        .execute(pool)
        .await;

    match result {
        Ok(rows) if rows.rows_affected() == 0 => {
            err_response(StatusCode::NOT_FOUND, "label not found")
        }
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 12. POST /repos/{owner}/{name}/boards/{id}/cards/{card_id}/assignees — add assignee
// ---------------------------------------------------------------------------

pub async fn add_card_assignee(
    State(state): State<AppState>,
    Path((owner, name, board_id, card_id)): Path<(String, String, uuid::Uuid, uuid::Uuid)>,
    _auth: AuthUser,
    Json(req): Json<AddCardAssigneeRequest>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let exists: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM boards WHERE id = $1 AND repo_id = $2")
            .bind(board_id)
            .bind(repo_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    if exists.is_none() {
        return err_response(StatusCode::NOT_FOUND, "board not found");
    }

    let card_exists: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM board_cards WHERE id = $1")
            .bind(card_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    if card_exists.is_none() {
        return err_response(StatusCode::NOT_FOUND, "card not found");
    }

    match sqlx::query_as::<_, CardAssigneeResponse>(
        r#"INSERT INTO board_card_assignees (card_id, user_id)
           VALUES ($1, $2)
           ON CONFLICT DO NOTHING
           RETURNING id, card_id, user_id"#,
    )
    .bind(card_id)
    .bind(req.user_id)
    .fetch_one(pool)
    .await
    {
        Ok(assignee) => (StatusCode::CREATED, Json(assignee)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 13. DELETE /repos/{owner}/{name}/boards/{id}/cards/{card_id}/assignees/{user_id} — remove assignee
// ---------------------------------------------------------------------------

pub async fn remove_card_assignee(
    State(state): State<AppState>,
    Path((owner, name, board_id, card_id, user_id)): Path<(
        String,
        String,
        uuid::Uuid,
        uuid::Uuid,
        uuid::Uuid,
    )>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let exists: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM boards WHERE id = $1 AND repo_id = $2")
            .bind(board_id)
            .bind(repo_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    if exists.is_none() {
        return err_response(StatusCode::NOT_FOUND, "board not found");
    }

    let result =
        sqlx::query("DELETE FROM board_card_assignees WHERE card_id = $1 AND user_id = $2")
            .bind(card_id)
            .bind(user_id)
            .execute(pool)
            .await;

    match result {
        Ok(rows) if rows.rows_affected() == 0 => {
            err_response(StatusCode::NOT_FOUND, "assignee not found")
        }
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 14. PATCH /repos/{owner}/{name}/boards/{id}/cards/{card_id}/priority — update priority
// ---------------------------------------------------------------------------

pub async fn update_card_priority(
    State(state): State<AppState>,
    Path((owner, name, board_id, card_id)): Path<(String, String, uuid::Uuid, uuid::Uuid)>,
    _auth: AuthUser,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let exists: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM boards WHERE id = $1 AND repo_id = $2")
            .bind(board_id)
            .bind(repo_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    if exists.is_none() {
        return err_response(StatusCode::NOT_FOUND, "board not found");
    }

    let priority = match body.get("priority").and_then(|v| v.as_i64()) {
        Some(p) => p as i32,
        None => return err_response(StatusCode::BAD_REQUEST, "priority is required"),
    };

    match sqlx::query(
        "UPDATE board_cards SET priority = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(priority)
    .bind(card_id)
    .execute(pool)
    .await
    {
        Ok(rows) if rows.rows_affected() == 0 => {
            err_response(StatusCode::NOT_FOUND, "card not found")
        }
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"priority": priority}))).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 15. PATCH /repos/{owner}/{name}/boards/{id}/cards/{card_id}/due_date — update due date
// ---------------------------------------------------------------------------

pub async fn update_card_due_date(
    State(state): State<AppState>,
    Path((owner, name, board_id, card_id)): Path<(String, String, uuid::Uuid, uuid::Uuid)>,
    _auth: AuthUser,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let exists: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM boards WHERE id = $1 AND repo_id = $2")
            .bind(board_id)
            .bind(repo_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    if exists.is_none() {
        return err_response(StatusCode::NOT_FOUND, "board not found");
    }

    let due_date: Option<DateTime<Utc>> = body
        .get("due_date")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    match sqlx::query(
        "UPDATE board_cards SET due_date = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(due_date)
    .bind(card_id)
    .execute(pool)
    .await
    {
        Ok(rows) if rows.rows_affected() == 0 => {
            err_response(StatusCode::NOT_FOUND, "card not found")
        }
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({"due_date": due_date})),
        )
            .into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 16. PATCH /repos/{owner}/{name}/boards/{id}/cards/{card_id}/sort_order — update sort order
// ---------------------------------------------------------------------------

pub async fn update_card_sort_order(
    State(state): State<AppState>,
    Path((owner, name, board_id, card_id)): Path<(String, String, uuid::Uuid, uuid::Uuid)>,
    _auth: AuthUser,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let repo_id = match get_repo_id(pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let exists: Option<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM boards WHERE id = $1 AND repo_id = $2")
            .bind(board_id)
            .bind(repo_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    if exists.is_none() {
        return err_response(StatusCode::NOT_FOUND, "board not found");
    }

    let sort_order = match body.get("sort_order").and_then(|v| v.as_i64()) {
        Some(p) => p as i32,
        None => return err_response(StatusCode::BAD_REQUEST, "sort_order is required"),
    };

    match sqlx::query(
        "UPDATE board_cards SET sort_order = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(sort_order)
    .bind(card_id)
    .execute(pool)
    .await
    {
        Ok(rows) if rows.rows_affected() == 0 => {
            err_response(StatusCode::NOT_FOUND, "card not found")
        }
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({"sort_order": sort_order})),
        )
            .into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// Route registration
// ---------------------------------------------------------------------------

pub fn board_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/boards",
            get(list_boards).post(create_board),
        )
        .route("/api/v1/repos/{owner}/{name}/boards/{id}", get(get_board))
        .route(
            "/api/v1/repos/{owner}/{name}/boards/{id}/columns",
            post(add_column),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/boards/{id}/columns/{col_id}",
            patch(update_column).delete(delete_column),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/boards/{id}/cards",
            post(add_card),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/boards/{id}/cards/{card_id}",
            patch(move_card).delete(delete_card),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/boards/{id}/cards/{card_id}/labels",
            post(add_card_label),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/boards/{id}/cards/{card_id}/labels/{label}",
            delete(remove_card_label),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/boards/{id}/cards/{card_id}/assignees",
            post(add_card_assignee),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/boards/{id}/cards/{card_id}/assignees/{user_id}",
            delete(remove_card_assignee),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/boards/{id}/cards/{card_id}/priority",
            patch(update_card_priority),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/boards/{id}/cards/{card_id}/due_date",
            patch(update_card_due_date),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/boards/{id}/cards/{card_id}/sort_order",
            patch(update_card_sort_order),
        )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_board_request_deserialize() {
        let json = r#"{"name":"My Board","description":"Sprint 1"}"#;
        let req: CreateBoardRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "My Board");
        assert_eq!(req.description.as_deref(), Some("Sprint 1"));
    }

    #[test]
    fn test_create_board_request_minimal() {
        let json = r#"{"name":"Board"}"#;
        let req: CreateBoardRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Board");
        assert!(req.description.is_none());
    }

    #[test]
    fn test_create_column_request_deserialize() {
        let json = r#"{"name":"Backlog","position":0}"#;
        let req: CreateColumnRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Backlog");
        assert_eq!(req.position, Some(0));
    }

    #[test]
    fn test_update_column_request_deserialize() {
        let json = r#"{"name":"Updated Name","position":2}"#;
        let req: UpdateColumnRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name.as_deref(), Some("Updated Name"));
        assert_eq!(req.position, Some(2));
    }

    #[test]
    fn test_create_card_request_deserialize() {
        let json = r#"{"column_id":"00000000-0000-0000-0000-000000000001","title":"Fix bug","description":"A critical bug"}"#;
        let req: CreateCardRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.title, "Fix bug");
        assert!(req.issue_id.is_none());
    }

    #[test]
    fn test_move_card_request_deserialize() {
        let json = r#"{"column_id":"00000000-0000-0000-0000-000000000002","position":0}"#;
        let req: MoveCardRequest = serde_json::from_str(json).unwrap();
        assert!(req.column_id.is_some());
        assert_eq!(req.position, Some(0));
    }

    #[test]
    fn test_move_card_request_partial() {
        let json = r#"{}"#;
        let req: MoveCardRequest = serde_json::from_str(json).unwrap();
        assert!(req.column_id.is_none());
        assert!(req.position.is_none());
    }

    #[test]
    fn test_board_response_serialization() {
        let resp = BoardResponse {
            id: uuid::Uuid::nil(),
            repo_id: uuid::Uuid::nil(),
            name: "Test Board".into(),
            description: None,
            created_by: uuid::Uuid::nil(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("Test Board"));
    }

    #[test]
    fn test_column_response_serialization() {
        let resp = ColumnResponse {
            id: uuid::Uuid::nil(),
            board_id: uuid::Uuid::nil(),
            name: "To Do".into(),
            position: 0,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("To Do"));
    }

    #[test]
    fn test_card_response_serialization() {
        let resp = CardResponse {
            id: uuid::Uuid::nil(),
            column_id: uuid::Uuid::nil(),
            issue_id: None,
            title: "My Card".into(),
            description: None,
            position: 0,
            priority: 0,
            due_date: None,
            sort_order: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("My Card"));
    }

    #[test]
    fn test_board_detail_response_serialization() {
        let resp = BoardDetailResponse {
            board: BoardResponse {
                id: uuid::Uuid::nil(),
                repo_id: uuid::Uuid::nil(),
                name: "Board".into(),
                description: Some("desc".into()),
                created_by: uuid::Uuid::nil(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            columns: vec![],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("columns"));
        assert!(json.contains("Board"));
    }
}
