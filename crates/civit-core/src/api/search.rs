#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::error::CoreError;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Query / request param structs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
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
#[allow(dead_code)]
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

// ---------------------------------------------------------------------------
// Response structs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct SearchHit {
    pub file_path: String,
    pub language: Option<String>,
    pub line_number: i32,
    pub line_content: String,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct SearchEnvelope {
    pub results: Vec<SearchHit>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct LanguagesResponse {
    pub languages: Vec<String>,
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
         WHERE i.search_vector @@ plainto_tsquery($1)",
    );
    let mut count_str = String::from(
        "SELECT COUNT(DISTINCT i.file_path) \
         FROM code_search_tokens t \
         INNER JOIN code_search_index i ON i.id = t.index_id \
         WHERE i.search_vector @@ plainto_tsquery($1)",
    );

    let mut bind_idx = 2i32;

    if let Some(ref lang) = params.language {
        if !lang.is_empty() {
            let clause = format!(" AND i.language = ${bind_idx}");
            query_str.push_str(&clause);
            count_str.push_str(&clause);
            bind_idx += 1;
        }
    }

    if let Some(ref repo) = params.repo {
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

    let mut query = sqlx::query_as::<_, SearchHitRow>(&query_str).bind(&q);
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_str).bind(&q);

    if let Some(ref lang) = params.language {
        if !lang.is_empty() {
            query = query.bind(lang);
            count_query = count_query.bind(lang);
        }
    }

    if let Some(ref repo) = params.repo {
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
#[allow(dead_code)]
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

    if let Some(ref lang) = params.language {
        if !lang.is_empty() {
            let clause = format!(" AND i.language = ${bind_idx}");
            query_str.push_str(&clause);
            count_str.push_str(&clause);
            bind_idx += 1;
        }
    }

    if let Some(ref path_glob) = params.path {
        if !path_glob.is_empty() {
            let clause = format!(" AND i.file_path ILIKE ${bind_idx}");
            query_str.push_str(&clause);
            count_str.push_str(&clause);
            bind_idx += 1;
        }
    }

    query_str.push_str(&format!(
        " ORDER BY i.file_path, t.line_number LIMIT ${idx} OFFSET ${idx2}",
        idx = bind_idx,
        idx2 = bind_idx + 1,
    ));

    let mut query = sqlx::query_as::<_, SearchHitRow>(&query_str)
        .bind(repo_id)
        .bind(&q);
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_str)
        .bind(repo_id)
        .bind(&q);

    if let Some(ref lang) = params.language {
        if !lang.is_empty() {
            query = query.bind(lang);
            count_query = count_query.bind(lang);
        }
    }

    let mut path_pattern: Option<String> = None;
    if let Some(ref path_glob) = params.path {
        if !path_glob.is_empty() {
            path_pattern = Some(format!("%{path_glob}%"));
        }
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
// Route builder
// ---------------------------------------------------------------------------

pub fn search_routes() -> axum::Router<AppState> {
    use axum::routing::get;

    axum::Router::new()
        .route("/api/v1/search", get(global_search))
        .route("/api/v1/repos/{owner}/{name}/search", get(repo_search))
        .route(
            "/api/v1/repos/{owner}/{name}/search/languages",
            get(list_repo_search_languages),
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
}
