#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
use crate::error::CoreError;
use crate::search::tantivy_index::SearchHit as TantivySearchHit;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::Row;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Query / request param structs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
pub struct GlobalSearchParams {
    pub q: Option<String>,
    pub repo: Option<String>,
    pub language: Option<String>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

#[derive(Debug, Deserialize, Default)]
pub struct RepoSearchParams {
    pub q: Option<String>,
    pub language: Option<String>,
    pub path: Option<String>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    30
}

#[derive(Debug, Deserialize, Default)]
pub struct CodeSearchParams {
    pub q: Option<String>,
    pub repo: Option<String>,
    pub lang: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default = "default_offset")]
    pub offset: usize,
}

#[derive(Debug, Deserialize, Default)]
pub struct RepoCodeSearchParams {
    pub q: Option<String>,
    pub lang: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default = "default_offset")]
    pub offset: usize,
}

fn default_limit() -> usize {
    30
}

fn default_offset() -> usize {
    0
}

// ---------------------------------------------------------------------------
// Response structs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct SearchHit {
    pub file_path: String,
    pub language: Option<String>,
    pub line_number: i32,
    pub line_content: String,
}

#[derive(Debug, Serialize)]
struct SearchEnvelope {
    pub results: Vec<SearchHit>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Serialize)]
struct LanguagesResponse {
    pub languages: Vec<String>,
}

#[derive(Debug, Serialize)]
struct CodeSearchResponse {
    pub results: Vec<TantivySearchHit>,
    pub query: String,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Serialize)]
struct IndexTriggerResponse {
    pub status: String,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Advanced Search Filters
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct ParsedFilters {
    pub free_text: String,
    pub repo_name: Option<String>,
    pub user_name: Option<String>,
    pub item_type: Option<String>,
    pub status: Option<String>,
    pub language: Option<String>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,
}

/// Parse advanced search syntax from a query string.
/// Supports: repo:name, user:name, is:issue|pr, status:open|closed, language:x, created:>date / created:<date
pub fn parse_advanced_filters(query: &str) -> ParsedFilters {
    let mut filters = ParsedFilters::default();
    let mut free_parts = Vec::new();

    for token in query.split_whitespace() {
        if let Some(val) = token.strip_prefix("repo:") {
            filters.repo_name = Some(val.to_string());
        } else if let Some(val) = token.strip_prefix("user:") {
            filters.user_name = Some(val.to_string());
        } else if let Some(val) = token.strip_prefix("is:") {
            match val {
                "issue" | "pr" => filters.item_type = Some(val.to_string()),
                _ => free_parts.push(token),
            }
        } else if let Some(val) = token.strip_prefix("status:") {
            match val {
                "open" | "closed" => filters.status = Some(val.to_string()),
                _ => free_parts.push(token),
            }
        } else if let Some(val) = token.strip_prefix("language:") {
            filters.language = Some(val.to_string());
        } else if let Some(val) = token.strip_prefix("created:") {
            if let Some(date) = val.strip_prefix('>') {
                filters.created_after = Some(date.to_string());
            } else if let Some(date) = val.strip_prefix('<') {
                filters.created_before = Some(date.to_string());
            } else {
                free_parts.push(token);
            }
        } else {
            free_parts.push(token);
        }
    }

    filters.free_text = free_parts.join(" ");
    filters
}

// ---------------------------------------------------------------------------
// Search Suggestions
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SearchSuggestParams {
    pub q: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchSuggestion {
    pub text: String,
    pub category: String,
}

#[derive(Debug, Serialize)]
struct SearchSuggestResponse {
    pub suggestions: Vec<SearchSuggestion>,
}

/// GET /api/v1/search/suggest — auto-complete suggestions for search queries
pub async fn search_suggestions(
    State(state): State<AppState>,
    Query(params): Query<SearchSuggestParams>,
) -> impl IntoResponse {
    let q = match &params.q {
        Some(q) if !q.trim().is_empty() => q.trim().to_string(),
        _ => {
            return (
                StatusCode::OK,
                Json(SearchSuggestResponse {
                    suggestions: Vec::new(),
                }),
            )
                .into_response();
        }
    };

    let pool = state.db.pool();
    let mut suggestions = Vec::new();

    // Suggest repos matching prefix
    if let Ok(repos) = sqlx::query_scalar::<_, String>(
        "SELECT CONCAT(u.username, '/', r.name) FROM repositories r \
         JOIN users u ON r.owner_id = u.id \
         WHERE CONCAT(u.username, '/', r.name) ILIKE $1 LIMIT 5",
    )
    .bind(format!("{q}%"))
    .fetch_all(pool)
    .await
    {
        for repo in repos {
            suggestions.push(SearchSuggestion {
                text: repo,
                category: "repo".into(),
            });
        }
    }

    // Suggest users matching prefix
    if let Ok(users) = sqlx::query_scalar::<_, String>(
        "SELECT username FROM users WHERE username ILIKE $1 LIMIT 5",
    )
    .bind(format!("{q}%"))
    .fetch_all(pool)
    .await
    {
        for user in users {
            suggestions.push(SearchSuggestion {
                text: user,
                category: "user".into(),
            });
        }
    }

    // Suggest recent search queries matching prefix (from search_history)
    if let Ok(queries) = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT query FROM search_history \
         WHERE query ILIKE $1 ORDER BY created_at DESC LIMIT 5",
    )
    .bind(format!("{q}%"))
    .fetch_all(pool)
    .await
    {
        for query in queries {
            suggestions.push(SearchSuggestion {
                text: query,
                category: "recent".into(),
            });
        }
    }

    (
        StatusCode::OK,
        Json(SearchSuggestResponse { suggestions }),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Search History
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SearchHistoryParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

#[derive(Debug, Deserialize)]
pub struct AddSearchHistoryBody {
    pub query: String,
    pub result_count: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SearchHistoryItem {
    pub id: String,
    pub query: String,
    pub result_count: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
struct SearchHistoryResponse {
    pub items: Vec<SearchHistoryItem>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

/// GET /api/v1/search/history — list recent searches for the authenticated user
pub async fn get_search_history(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<SearchHistoryParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let user_id = match uuid::Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => return err_response(StatusCode::UNAUTHORIZED, "invalid user id"),
    };

    let offset = (params.page - 1) * params.per_page;

    let items = match sqlx::query_as::<_, SearchHistoryRow>(
        "SELECT id, query, result_count, created_at::text \
         FROM search_history WHERE user_id = $1 \
         ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(user_id)
    .bind(params.per_page)
    .bind(offset)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => return internal_err(&e.to_string()),
    };

    let total = match sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM search_history WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    {
        Ok(t) => t,
        Err(e) => return internal_err(&e.to_string()),
    };

    let items: Vec<SearchHistoryItem> = items
        .into_iter()
        .map(|r| SearchHistoryItem {
            id: r.id,
            query: r.query,
            result_count: r.result_count,
            created_at: r.created_at,
        })
        .collect();

    (
        StatusCode::OK,
        Json(SearchHistoryResponse {
            items,
            total,
            page: params.page,
            per_page: params.per_page,
        }),
    )
        .into_response()
}

#[derive(Debug, sqlx::FromRow)]
struct SearchHistoryRow {
    id: String,
    query: String,
    result_count: i64,
    created_at: String,
}

/// POST /api/v1/search/history — record a search query
pub async fn add_search_history(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<AddSearchHistoryBody>,
) -> impl IntoResponse {
    let pool = state.db.pool();
    let user_id = match uuid::Uuid::parse_str(&auth.user_id) {
        Ok(id) => id,
        Err(_) => return err_response(StatusCode::UNAUTHORIZED, "invalid user id"),
    };

    let query = sanitize_query(&body.query);
    if query.is_empty() {
        return err_response(StatusCode::BAD_REQUEST, "query is required");
    }

    let result_count = body.result_count.unwrap_or(0);

    match sqlx::query(
        "INSERT INTO search_history (user_id, query, result_count) VALUES ($1, $2, $3) RETURNING id::text",
    )
    .bind(user_id)
    .bind(&query)
    .bind(result_count)
    .fetch_one(pool)
    .await
    {
        Ok(row) => {
            let id: String = row.get("id");
            (
                StatusCode::CREATED,
                Json(serde_json::json!({ "id": id, "query": query, "result_count": result_count })),
            )
                .into_response()
        }
        Err(e) => internal_err(&e.to_string()),
    }
}

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
// Helper: query sanitization
// ---------------------------------------------------------------------------

fn sanitize_query(q: &str) -> String {
    let q = q.trim();
    let max_len = 256usize;
    if q.len() > max_len {
        q[..max_len].to_string()
    } else {
        q.to_string()
    }
}

// ---------------------------------------------------------------------------
// Helper: error response shorthand
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
// 1. GET /api/v1/search — global search across repos
// ---------------------------------------------------------------------------

pub async fn global_search(
    State(state): State<AppState>,
    Query(params): Query<GlobalSearchParams>,
) -> impl IntoResponse {
    let pool = state.db.pool();

    let raw_q = match &params.q {
        Some(q) if !q.trim().is_empty() => q.trim().to_string(),
        _ => {
            return err_response(StatusCode::BAD_REQUEST, "query parameter 'q' is required");
        }
    };

    // Parse advanced filters from the query
    let filters = parse_advanced_filters(&raw_q);

    // Merge explicit params with parsed filters (explicit params take precedence)
    let effective_repo = params
        .repo
        .as_ref()
        .or(filters.repo_name.as_ref())
        .cloned();
    let effective_language = params
        .language
        .as_ref()
        .or(filters.language.as_ref())
        .cloned();
    let q = sanitize_query(&filters.free_text);

    let offset = (params.page - 1) * params.per_page;

    let mut query_str = String::from(
        "SELECT DISTINCT i.file_path, i.language, t.line_number, t.line_content \
         FROM code_search_tokens t \
         INNER JOIN code_search_index i ON i.id = t.index_id \
         WHERE i.search_vector @@ plainto_tsquery($1)",
    );
    let mut count_str = String::from(
        "SELECT COUNT(DISTINCT i.file_path) \
         FROM code_search_tokens t \
         INNER JOIN code_search_index i ON i.id = t.index_id \
         WHERE i.search_vector @@ plainto_tsquery($1)",
    );

    let mut bind_idx = 2i32;

    if let Some(ref lang) = effective_language
        && !lang.is_empty()
    {
        let clause = format!(" AND i.language = ${bind_idx}");
        query_str.push_str(&clause);
        count_str.push_str(&clause);
        bind_idx += 1;
    }

    if let Some(ref repo) = effective_repo {
        let parts: Vec<&str> = repo.splitn(2, '/').collect();
        if parts.len() == 2 {
            let clause = format!(
                " AND i.repo_id = (SELECT r.id FROM repositories r JOIN users u ON r.owner_id = u.id WHERE u.username = ${bind_idx} AND r.name = ${bind_idx_plus})",
                bind_idx = bind_idx,
                bind_idx_plus = bind_idx + 1,
            );
            query_str.push_str(&clause);
            count_str.push_str(&clause);
            bind_idx += 2;
        }
    }

    query_str.push_str(&format!(
        " ORDER BY i.file_path, t.line_number LIMIT ${idx} OFFSET ${idx2}",
        idx = bind_idx,
        idx2 = bind_idx + 1,
    ));

    let mut query = sqlx::query_as::<_, SearchHitRow>(sqlx::AssertSqlSafe(query_str)).bind(&q);
    let mut count_query = sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(count_str)).bind(&q);

    if let Some(ref lang) = effective_language
        && !lang.is_empty()
    {
        query = query.bind(lang);
        count_query = count_query.bind(lang);
    }

    if let Some(ref repo) = effective_repo {
        let parts: Vec<&str> = repo.splitn(2, '/').collect();
        if parts.len() == 2 {
            query = query.bind(parts[0]).bind(parts[1]);
            count_query = count_query.bind(parts[0]).bind(parts[1]);
        }
    }

    query = query.bind(params.per_page).bind(offset);

    let rows = match query.fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return internal_err(&e.to_string()),
    };

    let total = match count_query.fetch_one(pool).await {
        Ok(t) => t,
        Err(e) => return internal_err(&e.to_string()),
    };

    let results: Vec<SearchHit> = rows
        .into_iter()
        .map(|r| SearchHit {
            file_path: r.file_path,
            language: r.language,
            line_number: r.line_number,
            line_content: r.line_content,
        })
        .collect();

    (
        StatusCode::OK,
        Json(SearchEnvelope {
            results,
            total,
            page: params.page,
            per_page: params.per_page,
        }),
    )
        .into_response()
}

#[derive(Debug, sqlx::FromRow)]
struct SearchHitRow {
    file_path: String,
    language: Option<String>,
    line_number: i32,
    line_content: String,
}

// ---------------------------------------------------------------------------
// 2. GET /api/v1/repos/{owner}/{name}/search — search within a repo
// ---------------------------------------------------------------------------

pub async fn repo_search(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(params): Query<RepoSearchParams>,
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

    let q = match &params.q {
        Some(q) if !q.trim().is_empty() => sanitize_query(q),
        _ => {
            return err_response(StatusCode::BAD_REQUEST, "query parameter 'q' is required");
        }
    };

    let offset = (params.page - 1) * params.per_page;

    let mut query_str = String::from(
        "SELECT DISTINCT i.file_path, i.language, t.line_number, t.line_content \
         FROM code_search_tokens t \
         INNER JOIN code_search_index i ON i.id = t.index_id \
         WHERE i.repo_id = $1 AND i.search_vector @@ plainto_tsquery($2)",
    );
    let mut count_str = String::from(
        "SELECT COUNT(DISTINCT i.file_path) \
         FROM code_search_tokens t \
         INNER JOIN code_search_index i ON i.id = t.index_id \
         WHERE i.repo_id = $1 AND i.search_vector @@ plainto_tsquery($2)",
    );

    let mut bind_idx = 3i32;

    if let Some(ref lang) = params.language
        && !lang.is_empty()
    {
        let clause = format!(" AND i.language = ${bind_idx}");
        query_str.push_str(&clause);
        count_str.push_str(&clause);
        bind_idx += 1;
    }

    if let Some(ref path_glob) = params.path
        && !path_glob.is_empty()
    {
        let clause = format!(" AND i.file_path ILIKE ${bind_idx}");
        query_str.push_str(&clause);
        count_str.push_str(&clause);
        bind_idx += 1;
    }

    query_str.push_str(&format!(
        " ORDER BY i.file_path, t.line_number LIMIT ${idx} OFFSET ${idx2}",
        idx = bind_idx,
        idx2 = bind_idx + 1,
    ));

    let mut query = sqlx::query_as::<_, SearchHitRow>(sqlx::AssertSqlSafe(query_str))
        .bind(repo_id)
        .bind(&q);
    let mut count_query = sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(count_str))
        .bind(repo_id)
        .bind(&q);

    if let Some(ref lang) = params.language
        && !lang.is_empty()
    {
        query = query.bind(lang);
        count_query = count_query.bind(lang);
    }

    let mut path_pattern: Option<String> = None;
    if let Some(ref path_glob) = params.path
        && !path_glob.is_empty()
    {
        path_pattern = Some(format!("%{path_glob}%"));
    }
    if let Some(ref pp) = path_pattern {
        query = query.bind(pp.as_str());
        count_query = count_query.bind(pp.as_str());
    }

    let query = query.bind(params.per_page).bind(offset);

    let rows = match query.fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => return internal_err(&e.to_string()),
    };

    let total = match count_query.fetch_one(pool).await {
        Ok(t) => t,
        Err(e) => return internal_err(&e.to_string()),
    };

    let results: Vec<SearchHit> = rows
        .into_iter()
        .map(|r| SearchHit {
            file_path: r.file_path,
            language: r.language,
            line_number: r.line_number,
            line_content: r.line_content,
        })
        .collect();

    (
        StatusCode::OK,
        Json(SearchEnvelope {
            results,
            total,
            page: params.page,
            per_page: params.per_page,
        }),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// 3. GET /api/v1/repos/{owner}/{name}/search/languages — list indexed languages
// ---------------------------------------------------------------------------

pub async fn list_repo_search_languages(
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

    match sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT language FROM code_search_index WHERE repo_id = $1 AND language IS NOT NULL ORDER BY language",
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await
    {
        Ok(languages) => (
            StatusCode::OK,
            Json(LanguagesResponse { languages }),
        )
            .into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 4. GET /api/v1/search/code — global Tantivy code search
// ---------------------------------------------------------------------------

pub async fn global_code_search(
    State(state): State<AppState>,
    Query(params): Query<CodeSearchParams>,
) -> impl IntoResponse {
    let q = match &params.q {
        Some(q) if !q.trim().is_empty() => q.trim().to_string(),
        _ => {
            return err_response(StatusCode::BAD_REQUEST, "query parameter 'q' is required");
        }
    };

    let limit = params.limit.min(100);
    let offset = params.offset;

    let idx = state.code_search_index.read().await;
    match idx.search_global(&q, limit, offset) {
        Ok(results) => (
            StatusCode::OK,
            Json(CodeSearchResponse {
                results,
                query: q,
                limit,
                offset,
            }),
        )
            .into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 5. GET /api/v1/repos/{owner}/{name}/search/code — repo-scoped Tantivy code search
// ---------------------------------------------------------------------------

pub async fn repo_code_search(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(params): Query<RepoCodeSearchParams>,
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

    let q = match &params.q {
        Some(q) if !q.trim().is_empty() => q.trim().to_string(),
        _ => {
            return err_response(StatusCode::BAD_REQUEST, "query parameter 'q' is required");
        }
    };

    let limit = params.limit.min(100);
    let offset = params.offset;
    let repo_id_str = repo_id.to_string();

    let idx = state.code_search_index.read().await;
    match idx.search(
        &q,
        Some(&repo_id_str),
        params.lang.as_deref(),
        limit,
        offset,
    ) {
        Ok(results) => (
            StatusCode::OK,
            Json(CodeSearchResponse {
                results,
                query: q,
                limit,
                offset,
            }),
        )
            .into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 6. POST /api/v1/repos/{owner}/{name}/search/index — trigger re-index
// ---------------------------------------------------------------------------

pub async fn trigger_repo_index(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
) -> axum::response::Response {
    let pool = state.db.pool().clone();

    let repo_id = match get_repo_id(&pool, &owner, &name).await {
        Some(id) => id,
        None => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("repository {owner}/{name} not found"),
            );
        }
    };

    let repo_path: PathBuf = state.git_service.repo_path(&owner, &name);

    let (commit_sha, files) = match collect_repo_files(&repo_path) {
        Ok(f) => f,
        Err(e) => return internal_err(&e.to_string()),
    };

    match index_collected_files(&pool, &repo_id, &commit_sha, &files).await {
        Ok(count) => (
            StatusCode::OK,
            Json(IndexTriggerResponse {
                status: "ok".to_string(),
                message: format!("indexed {count} files for {owner}/{name}"),
            }),
        )
            .into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// Helper: index_repository — walk a bare git repo and populate search tables
// ---------------------------------------------------------------------------

fn language_from_extension(path: &str) -> Option<&'static str> {
    let ext = std::path::Path::new(path)
        .extension()?
        .to_str()?
        .to_ascii_lowercase();
    Some(match ext.as_str() {
        "rs" => "rust",
        "go" => "go",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "tsx" => "typescript",
        "jsx" => "javascript",
        "rb" => "ruby",
        "java" => "java",
        "kt" => "kotlin",
        "scala" => "scala",
        "c" => "c",
        "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "cpp",
        "cs" => "csharp",
        "fs" | "fsx" | "fsi" => "fsharp",
        "swift" => "swift",
        "m" => "objc",
        "php" => "php",
        "pl" | "pm" => "perl",
        "r" => "r",
        "R" => "r",
        "lua" => "lua",
        "zig" => "zig",
        "nim" => "nim",
        "ex" | "exs" => "elixir",
        "erl" | "hrl" => "erlang",
        "hs" => "haskell",
        "ml" | "mli" => "ocaml",
        "clj" | "cljs" => "clojure",
        "dart" => "dart",
        "vue" => "vue",
        "svelte" => "svelte",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" => "scss",
        "less" => "less",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "xml" => "xml",
        "sql" => "sql",
        "sh" | "bash" | "zsh" => "shell",
        "ps1" => "powershell",
        "bat" | "cmd" => "batch",
        "md" | "markdown" => "markdown",
        "txt" => "text",
        "proto" => "protobuf",
        "graphql" | "gql" => "graphql",
        "tf" | "hcl" => "hcl",
        "dockerfile" => "dockerfile",
        "makefile" => "makefile",
        "cmake" => "cmake",
        "gradle" => "gradle",
        _ => return None,
    })
}

fn is_binary(data: &[u8]) -> bool {
    let check_len = data.len().min(512);
    data[..check_len].contains(&0)
}

struct CollectedFile {
    path: String,
    content: String,
    language: String,
    byte_size: i64,
    line_count: i32,
}

/// Phase 1: Walk the git tree synchronously and collect file metadata + content.
/// Returns (commit_sha, files). No gix types escape this function.
fn collect_repo_files(
    repo_path: &std::path::Path,
) -> Result<(String, Vec<CollectedFile>), CoreError> {
    let repo = gix::open(repo_path).map_err(|e| CoreError::Git(format!("open repository: {e}")))?;

    let head_id = repo
        .head_id()
        .map_err(|e| CoreError::Git(format!("read HEAD: {e}")))?;

    let commit_obj = head_id
        .object()
        .map_err(|e| CoreError::Git(format!("read commit object: {e}")))?;

    let commit = commit_obj
        .try_into_commit()
        .map_err(|e| CoreError::Git(format!("parse commit: {e}")))?;

    let commit_sha = head_id.to_hex().to_string();

    let tree_id = commit
        .tree_id()
        .map_err(|e| CoreError::Git(format!("read tree id: {e}")))?;

    let root_tree_id = tree_id.detach();

    let mut files = Vec::new();
    let mut stack: Vec<(gix::hash::ObjectId, String)> = vec![(root_tree_id, String::new())];

    while let Some((current_tree_id, prefix)) = stack.pop() {
        let tree_obj = match repo.find_object(current_tree_id) {
            Ok(o) => o,
            Err(_) => continue,
        };
        let tree = match tree_obj.try_into_tree() {
            Ok(t) => t,
            Err(_) => continue,
        };

        for entry_result in tree.iter() {
            let entry = match entry_result {
                Ok(e) => e,
                Err(_) => continue,
            };

            let name = entry.filename().to_string();
            let path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}/{name}")
            };

            let mode = entry.mode();
            if mode.is_tree() {
                let subtree_oid = entry.oid().to_owned();
                stack.push((subtree_oid, path));
                continue;
            }

            if !mode.is_blob() {
                continue;
            }

            let blob_obj = match entry.object() {
                Ok(o) => o,
                Err(_) => continue,
            };
            let blob = match blob_obj.try_into_blob() {
                Ok(b) => b,
                Err(_) => continue,
            };

            if is_binary(&blob.data) {
                continue;
            }

            let content = String::from_utf8_lossy(&blob.data).to_string();
            let language = language_from_extension(&path)
                .map(|s| s.to_string())
                .unwrap_or_default();
            let line_count = content.lines().count() as i32;
            let byte_size = blob.data.len() as i64;

            files.push(CollectedFile {
                path,
                content,
                language,
                byte_size,
                line_count,
            });
        }
    }

    Ok((commit_sha, files))
}

/// Phase 2: Insert collected files into the database.
async fn index_collected_files(
    pool: &PgPool,
    repo_id: &uuid::Uuid,
    commit_sha: &str,
    files: &[CollectedFile],
) -> Result<usize, CoreError> {
    // Delete existing entries for this repo
    sqlx::query("DELETE FROM code_search_tokens WHERE index_id IN (SELECT id FROM code_search_index WHERE repo_id = $1)")
        .bind(repo_id)
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

    sqlx::query("DELETE FROM code_search_index WHERE repo_id = $1")
        .bind(repo_id)
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

    let mut indexed = 0usize;

    for file in files {
        // Insert into code_search_index
        let index_id = sqlx::query_scalar::<_, uuid::Uuid>(
            "INSERT INTO code_search_index \
             (repo_id, file_path, language, content, line_count, byte_size, commit_sha) \
             VALUES ($1, $2, NULLIF($3, ''), $4, $5, $6, $7) \
             RETURNING id",
        )
        .bind(repo_id)
        .bind(&file.path)
        .bind(&file.language)
        .bind(&file.content)
        .bind(file.line_count)
        .bind(file.byte_size)
        .bind(commit_sha)
        .fetch_one(pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        // Insert search tokens (one row per line)
        for (line_number, line) in file.content.lines().enumerate() {
            let line_content = line.trim();
            if line_content.is_empty() {
                continue;
            }
            sqlx::query(
                "INSERT INTO code_search_tokens (index_id, token, line_number, line_content) \
                 VALUES ($1, $2, $3, $4)",
            )
            .bind(index_id)
            .bind(line_content)
            .bind((line_number + 1) as i32)
            .bind(line_content)
            .execute(pool)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;
        }

        // Update search_vector for full-text search
        sqlx::query(
            "UPDATE code_search_index \
             SET search_vector = to_tsvector('english', coalesce(file_path, '') || ' ' || coalesce(content, '')) \
             WHERE id = $1",
        )
        .bind(index_id)
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        indexed += 1;
    }

    Ok(indexed)
}

/// Incrementally re-index a repository after a push.
/// Collects all files from the repo and updates the search index.
pub async fn reindex_repo_after_push(
    pool: &PgPool,
    repo_id: &uuid::Uuid,
    repo_path: &std::path::Path,
) -> Result<usize, CoreError> {
    let (commit_sha, files) = collect_repo_files(repo_path)?;
    index_collected_files(pool, repo_id, &commit_sha, &files).await
}

/// Compute the set of file paths that differ between two tree IDs.
/// Returns (added_or_modified, deleted) where each is a set of paths.
fn diff_trees(
    repo: &gix::Repository,
    old_tree_id: Option<gix::hash::ObjectId>,
    new_tree_id: gix::hash::ObjectId,
) -> Result<
    (
        std::collections::HashSet<String>,
        std::collections::HashSet<String>,
    ),
    CoreError,
> {
    use std::collections::{HashMap, HashSet};

    let mut old_files: HashMap<String, gix::hash::ObjectId> = HashMap::new();
    if let Some(old_id) = old_tree_id {
        collect_tree_entries(repo, old_id, "", &mut old_files)?;
    }

    let mut new_files: HashMap<String, gix::hash::ObjectId> = HashMap::new();
    collect_tree_entries(repo, new_tree_id, "", &mut new_files)?;

    let mut added_modified = HashSet::new();
    let mut deleted = HashSet::new();

    for (path, oid) in &new_files {
        match old_files.get(path) {
            Some(old_oid) if old_oid == oid => {}
            _ => {
                added_modified.insert(path.clone());
            }
        }
    }
    for path in old_files.keys() {
        if !new_files.contains_key(path) {
            deleted.insert(path.clone());
        }
    }

    Ok((added_modified, deleted))
}

fn collect_tree_entries(
    repo: &gix::Repository,
    tree_id: gix::hash::ObjectId,
    prefix: &str,
    entries: &mut std::collections::HashMap<String, gix::hash::ObjectId>,
) -> Result<(), CoreError> {
    let tree_obj = repo
        .find_object(tree_id)
        .map_err(|e| CoreError::Git(format!("find tree object: {e}")))?;
    let tree = tree_obj
        .try_into_tree()
        .map_err(|e| CoreError::Git(format!("parse tree: {e}")))?;

    for entry_result in tree.iter() {
        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue,
        };

        let name = entry.filename().to_string();
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };

        let mode = entry.mode();
        if mode.is_tree() {
            let subtree_oid = entry.oid().to_owned();
            collect_tree_entries(repo, subtree_oid, &path, entries)?;
        } else if mode.is_blob() {
            entries.insert(path, entry.oid().to_owned());
        }
    }
    Ok(())
}

/// Collect files from the repo tree, optionally filtering to only changed paths.
fn collect_changed_files(
    repo_path: &std::path::Path,
    changed_paths: &std::collections::HashSet<String>,
) -> Result<(String, Vec<CollectedFile>), CoreError> {
    let repo = gix::open(repo_path).map_err(|e| CoreError::Git(format!("open repository: {e}")))?;

    let head_id = repo
        .head_id()
        .map_err(|e| CoreError::Git(format!("read HEAD: {e}")))?;

    let commit_obj = head_id
        .object()
        .map_err(|e| CoreError::Git(format!("read commit object: {e}")))?;

    let commit = commit_obj
        .try_into_commit()
        .map_err(|e| CoreError::Git(format!("parse commit: {e}")))?;

    let commit_sha = head_id.to_hex().to_string();

    let tree_id = commit
        .tree_id()
        .map_err(|e| CoreError::Git(format!("read tree id: {e}")))?;

    let root_tree_id = tree_id.detach();

    let mut files = Vec::new();
    let mut stack: Vec<(gix::hash::ObjectId, String)> = vec![(root_tree_id, String::new())];

    while let Some((current_tree_id, prefix)) = stack.pop() {
        let tree_obj = match repo.find_object(current_tree_id) {
            Ok(o) => o,
            Err(_) => continue,
        };
        let tree = match tree_obj.try_into_tree() {
            Ok(t) => t,
            Err(_) => continue,
        };

        for entry_result in tree.iter() {
            let entry = match entry_result {
                Ok(e) => e,
                Err(_) => continue,
            };

            let name = entry.filename().to_string();
            let path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}/{name}")
            };

            let mode = entry.mode();
            if mode.is_tree() {
                let subtree_oid = entry.oid().to_owned();
                stack.push((subtree_oid, path));
                continue;
            }

            if !mode.is_blob() {
                continue;
            }

            if !changed_paths.contains(&path) {
                continue;
            }

            let blob_obj = match entry.object() {
                Ok(o) => o,
                Err(_) => continue,
            };
            let blob = match blob_obj.try_into_blob() {
                Ok(b) => b,
                Err(_) => continue,
            };

            if is_binary(&blob.data) {
                continue;
            }

            let content = String::from_utf8_lossy(&blob.data).to_string();
            let language = language_from_extension(&path)
                .map(|s| s.to_string())
                .unwrap_or_default();
            let line_count = content.lines().count() as i32;
            let byte_size = blob.data.len() as i64;

            files.push(CollectedFile {
                path,
                content,
                language,
                byte_size,
                line_count,
            });
        }
    }

    Ok((commit_sha, files))
}

/// Data collected from a git diff for indexing.
struct DiffIndexData {
    commit_sha: String,
    added_modified_files: Vec<CollectedFile>,
    deleted_paths: Vec<String>,
}

/// Collect diff data synchronously (no gix types escape this function).
fn collect_diff_data(repo_path: &std::path::Path) -> Result<DiffIndexData, CoreError> {
    let repo = gix::open(repo_path).map_err(|e| CoreError::Git(format!("open repository: {e}")))?;

    let head_id = repo
        .head_id()
        .map_err(|e| CoreError::Git(format!("read HEAD: {e}")))?;

    let commit_obj = head_id
        .object()
        .map_err(|e| CoreError::Git(format!("read commit object: {e}")))?;

    let commit = commit_obj
        .try_into_commit()
        .map_err(|e| CoreError::Git(format!("parse commit: {e}")))?;

    let new_tree_id = commit
        .tree_id()
        .map_err(|e| CoreError::Git(format!("read tree id: {e}")))?;

    let new_root = new_tree_id.detach();

    let mut old_tree_id = None;
    if let Some(parent_id) = commit.parent_ids().next()
        && let Ok(parent_obj) = parent_id.object()
        && let Ok(parent_commit) = parent_obj.try_into_commit()
        && let Ok(parent_tree_id) = parent_commit.tree_id()
    {
        old_tree_id = Some(parent_tree_id.detach());
    }

    let (added_modified, deleted) = diff_trees(&repo, old_tree_id, new_root)?;

    let (commit_sha, files) = collect_changed_files(repo_path, &added_modified)?;

    Ok(DiffIndexData {
        commit_sha,
        added_modified_files: files,
        deleted_paths: deleted.into_iter().collect(),
    })
}

/// Incrementally index only changed files after a push.
/// Computes the diff between old and new HEAD trees and indexes only added/modified files,
/// while removing deleted files from the index.
pub async fn reindex_changed_files_after_push(
    pool: &PgPool,
    repo_id: &uuid::Uuid,
    repo_path: &std::path::Path,
) -> Result<usize, CoreError> {
    let data = collect_diff_data(repo_path)?;

    tracing::info!(
        repo_id = %repo_id,
        added_modified = data.added_modified_files.len(),
        deleted = data.deleted_paths.len(),
        "diff-based search indexing"
    );

    for path in &data.deleted_paths {
        sqlx::query(
            "DELETE FROM code_search_tokens WHERE index_id IN \
             (SELECT id FROM code_search_index WHERE repo_id = $1 AND file_path = $2)",
        )
        .bind(repo_id)
        .bind(path)
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        sqlx::query("DELETE FROM code_search_index WHERE repo_id = $1 AND file_path = $2")
            .bind(repo_id)
            .bind(path)
            .execute(pool)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;
    }

    if data.added_modified_files.is_empty() {
        tracing::info!(repo_id = %repo_id, "no changed files to index");
        return Ok(0);
    }

    let mut indexed = 0usize;
    for file in &data.added_modified_files {
        sqlx::query(
            "DELETE FROM code_search_tokens WHERE index_id IN \
             (SELECT id FROM code_search_index WHERE repo_id = $1 AND file_path = $2)",
        )
        .bind(repo_id)
        .bind(&file.path)
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        sqlx::query("DELETE FROM code_search_index WHERE repo_id = $1 AND file_path = $2")
            .bind(repo_id)
            .bind(&file.path)
            .execute(pool)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        let index_id = sqlx::query_scalar::<_, uuid::Uuid>(
            "INSERT INTO code_search_index \
             (repo_id, file_path, language, content, line_count, byte_size, commit_sha) \
             VALUES ($1, $2, NULLIF($3, ''), $4, $5, $6, $7) \
             RETURNING id",
        )
        .bind(repo_id)
        .bind(&file.path)
        .bind(&file.language)
        .bind(&file.content)
        .bind(file.line_count)
        .bind(file.byte_size)
        .bind(&data.commit_sha)
        .fetch_one(pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        for (line_number, line) in file.content.lines().enumerate() {
            let line_content = line.trim();
            if line_content.is_empty() {
                continue;
            }
            sqlx::query(
                "INSERT INTO code_search_tokens (index_id, token, line_number, line_content) \
                 VALUES ($1, $2, $3, $4)",
            )
            .bind(index_id)
            .bind(line_content)
            .bind((line_number + 1) as i32)
            .bind(line_content)
            .execute(pool)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;
        }

        sqlx::query(
            "UPDATE code_search_index \
             SET search_vector = to_tsvector('english', coalesce(file_path, '') || ' ' || coalesce(content, '')) \
             WHERE id = $1",
        )
        .bind(index_id)
        .execute(pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        indexed += 1;
    }

    tracing::info!(
        repo_id = %repo_id,
        indexed,
        "diff-based search indexing complete"
    );

    Ok(indexed)
}

/// Background search indexing for a repository.
/// Uses diff-based indexing to only process changed files.
pub async fn trigger_repo_index_background(
    state: &AppState,
    owner: &str,
    name: &str,
) -> Result<(), String> {
    let pool = state.db.pool();
    let repo_id = get_repo_id(pool, owner, name)
        .await
        .ok_or_else(|| format!("repository {owner}/{name} not found"))?;
    let repo_path: PathBuf = state.git_service.repo_path(owner, name);

    let indexed = reindex_changed_files_after_push(pool, &repo_id, &repo_path)
        .await
        .map_err(|e| format!("failed to index files: {e}"))?;

    tracing::info!(owner, name, indexed, "background search indexing complete");
    Ok(())
}

// ---------------------------------------------------------------------------
// Route builder
// ---------------------------------------------------------------------------

pub fn search_routes() -> axum::Router<AppState> {
    use axum::routing::{get, post};

    axum::Router::new()
        .route("/api/v1/search", get(global_search))
        .route("/api/v1/search/code", get(global_code_search))
        .route("/api/v1/search/suggest", get(search_suggestions))
        .route(
            "/api/v1/search/history",
            get(get_search_history).post(add_search_history),
        )
        .route("/api/v1/repos/{owner}/{name}/search", get(repo_search))
        .route(
            "/api/v1/repos/{owner}/{name}/search/code",
            get(repo_code_search),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/search/languages",
            get(list_repo_search_languages),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/search/index",
            post(trigger_repo_index),
        )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_search_params_default() {
        let p: GlobalSearchParams = serde_json::from_str("{}").unwrap();
        assert!(p.q.is_none());
        assert!(p.repo.is_none());
        assert!(p.language.is_none());
        assert_eq!(p.page, 1);
        assert_eq!(p.per_page, 30);
    }

    #[test]
    fn test_global_search_params_with_values() {
        let p: GlobalSearchParams = serde_json::from_str(
            r#"{"q":"fn main","repo":"acme/repo","language":"rust","page":2,"per_page":10}"#,
        )
        .unwrap();
        assert_eq!(p.q.as_deref(), Some("fn main"));
        assert_eq!(p.repo.as_deref(), Some("acme/repo"));
        assert_eq!(p.language.as_deref(), Some("rust"));
        assert_eq!(p.page, 2);
        assert_eq!(p.per_page, 10);
    }

    #[test]
    fn test_repo_search_params_default() {
        let p: RepoSearchParams = serde_json::from_str("{}").unwrap();
        assert!(p.q.is_none());
        assert!(p.language.is_none());
        assert!(p.path.is_none());
        assert_eq!(p.page, 1);
        assert_eq!(p.per_page, 30);
    }

    #[test]
    fn test_repo_search_params_with_values() {
        let p: RepoSearchParams = serde_json::from_str(
            r#"{"q":"impl","language":"go","path":"src/","page":3,"per_page":50}"#,
        )
        .unwrap();
        assert_eq!(p.q.as_deref(), Some("impl"));
        assert_eq!(p.language.as_deref(), Some("go"));
        assert_eq!(p.path.as_deref(), Some("src/"));
        assert_eq!(p.page, 3);
        assert_eq!(p.per_page, 50);
    }

    #[test]
    fn test_search_routes_compile() {
        let router = search_routes();
        let _ = router;
    }

    #[test]
    fn test_sanitize_query_normal() {
        assert_eq!(sanitize_query("fn main"), "fn main");
    }

    #[test]
    fn test_sanitize_query_trims_whitespace() {
        assert_eq!(sanitize_query("  hello world  "), "hello world");
    }

    #[test]
    fn test_sanitize_query_truncates_long_input() {
        let long = "a".repeat(300);
        let sanitized = sanitize_query(&long);
        assert_eq!(sanitized.len(), 256);
    }

    #[test]
    fn test_sanitize_query_empty() {
        assert_eq!(sanitize_query(""), "");
        assert_eq!(sanitize_query("   "), "");
    }

    #[test]
    fn test_sanitize_query_preserves_special_chars_for_parametric_queries() {
        let input = "'; DROP TABLE code_search_tokens; --";
        let sanitized = sanitize_query(input);
        assert_eq!(sanitized, "'; DROP TABLE code_search_tokens; --");
    }

    #[test]
    fn test_language_filter_parsed() {
        let p: RepoSearchParams =
            serde_json::from_str(r#"{"q":"test","language":"python"}"#).unwrap();
        assert_eq!(p.language.as_deref(), Some("python"));
    }

    #[test]
    fn test_code_search_params_default() {
        let p: CodeSearchParams = serde_json::from_str("{}").unwrap();
        assert!(p.q.is_none());
        assert!(p.repo.is_none());
        assert!(p.lang.is_none());
        assert_eq!(p.limit, 30);
        assert_eq!(p.offset, 0);
    }

    #[test]
    fn test_code_search_params_with_values() {
        let p: CodeSearchParams = serde_json::from_str(
            r#"{"q":"fn main","repo":"acme/repo","lang":"rust","limit":10,"offset":5}"#,
        )
        .unwrap();
        assert_eq!(p.q.as_deref(), Some("fn main"));
        assert_eq!(p.repo.as_deref(), Some("acme/repo"));
        assert_eq!(p.lang.as_deref(), Some("rust"));
        assert_eq!(p.limit, 10);
        assert_eq!(p.offset, 5);
    }

    #[test]
    fn test_repo_code_search_params_default() {
        let p: RepoCodeSearchParams = serde_json::from_str("{}").unwrap();
        assert!(p.q.is_none());
        assert!(p.lang.is_none());
        assert_eq!(p.limit, 30);
        assert_eq!(p.offset, 0);
    }

    #[test]
    fn test_repo_code_search_params_with_values() {
        let p: RepoCodeSearchParams =
            serde_json::from_str(r#"{"q":"impl","lang":"go","limit":50,"offset":10}"#).unwrap();
        assert_eq!(p.q.as_deref(), Some("impl"));
        assert_eq!(p.lang.as_deref(), Some("go"));
        assert_eq!(p.limit, 50);
        assert_eq!(p.offset, 10);
    }

    #[test]
    fn test_tantivy_routes_compile() {
        let router = search_routes();
        let _ = router;
    }
}
