#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::error::CoreError;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Query / request param structs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct CreateWikiPageRequest {
    pub slug: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct UpdateWikiPageRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub edit_message: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct DiffParams {
    pub sha1: Option<String>,
    pub sha2: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct SearchWikiParams {
    pub q: Option<String>,
}

// ---------------------------------------------------------------------------
// Response structs
// ---------------------------------------------------------------------------

#[derive(Debug, sqlx::FromRow, Serialize)]
#[allow(dead_code)]
pub struct WikiPageSummary {
    pub slug: String,
    pub title: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
#[allow(dead_code)]
pub struct WikiPageResponse {
    pub id: i64,
    pub repo_id: i64,
    pub slug: String,
    pub title: String,
    pub format: String,
    pub content: String,
    pub latest_commit: String,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
#[allow(dead_code)]
pub struct WikiRevisionResponse {
    pub id: i64,
    pub page_id: i64,
    pub commit_sha: String,
    pub author_id: String,
    pub edit_message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct DiffResponse {
    sha1: String,
    sha2: String,
    diff: String,
}

// ---------------------------------------------------------------------------
// Helper: get repo id
// ---------------------------------------------------------------------------

async fn get_repo_id(pool: &sqlx::PgPool, owner: &str, name: &str) -> Option<i64> {
    sqlx::query_scalar::<_, i64>(
        "SELECT id FROM repositories WHERE owner_id::text = $1 AND name = $2",
    )
    .bind(owner)
    .bind(name)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

// ---------------------------------------------------------------------------
// Helper: slug sanitization
// ---------------------------------------------------------------------------

pub fn sanitize_slug(slug: &str) -> String {
    slug.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
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
// Helper: generate pseudo SHA (since no git backend for v1.0)
// ---------------------------------------------------------------------------

fn generate_pseudo_sha() -> String {
    format!("{:040x}", chrono::Utc::now().timestamp_millis())
}

// ---------------------------------------------------------------------------
// 1. GET /wiki — list all wiki pages
// ---------------------------------------------------------------------------

pub async fn list_wiki_pages(
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

    match sqlx::query_as::<_, WikiPageSummary>(
        "SELECT slug, title, updated_at FROM wiki_pages WHERE repo_id = $1 ORDER BY title",
    )
    .bind(repo_id)
    .fetch_all(pool)
    .await
    {
        Ok(pages) => (StatusCode::OK, Json(pages)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 2. POST /wiki — create wiki page
// ---------------------------------------------------------------------------

pub async fn create_wiki_page(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Json(req): Json<CreateWikiPageRequest>,
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

    if req.slug.trim().is_empty() {
        return err_response(StatusCode::BAD_REQUEST, "slug is required");
    }
    if req.title.trim().is_empty() {
        return err_response(StatusCode::BAD_REQUEST, "title is required");
    }

    let slug = sanitize_slug(&req.slug);
    let sha = generate_pseudo_sha();

    match sqlx::query_as::<_, WikiPageResponse>(
        "INSERT INTO wiki_pages (repo_id, slug, title, format, content, latest_commit, created_by, created_at, updated_at) VALUES ($1, $2, $3, 'markdown', $4, $5, 'system', NOW(), NOW()) RETURNING id, repo_id, slug, title, format, content, latest_commit, created_by, created_at, updated_at",
    )
    .bind(repo_id)
    .bind(&slug)
    .bind(&req.title)
    .bind(&req.content)
    .bind(&sha)
    .fetch_one(pool)
    .await
    {
        Ok(page) => (StatusCode::CREATED, Json(page)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 3. GET /wiki/:slug — get wiki page
// ---------------------------------------------------------------------------

pub async fn get_wiki_page(
    State(state): State<AppState>,
    Path((owner, name, slug)): Path<(String, String, String)>,
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

    match sqlx::query_as::<_, WikiPageResponse>(
        "SELECT id, repo_id, slug, title, format, content, latest_commit, created_by, created_at, updated_at FROM wiki_pages WHERE repo_id = $1 AND slug = $2",
    )
    .bind(repo_id)
    .bind(&slug)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(page)) => (StatusCode::OK, Json(page)).into_response(),
        Ok(None) => err_response(
            StatusCode::NOT_FOUND,
            &format!("wiki page '{slug}' not found"),
        ),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 4. PUT /wiki/:slug — update wiki page
// ---------------------------------------------------------------------------

pub async fn update_wiki_page(
    State(state): State<AppState>,
    Path((owner, name, slug)): Path<(String, String, String)>,
    Json(req): Json<UpdateWikiPageRequest>,
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

    let existing = match sqlx::query_as::<_, WikiPageResponse>(
        "SELECT id, repo_id, slug, title, format, content, latest_commit, created_by, created_at, updated_at FROM wiki_pages WHERE repo_id = $1 AND slug = $2",
    )
    .bind(repo_id)
    .bind(&slug)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(p)) => p,
        Ok(None) => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("wiki page '{slug}' not found"),
            );
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    let title = req.title.as_deref().unwrap_or(&existing.title);
    let content = req.content.as_deref().unwrap_or(&existing.content);
    let edit_msg = req.edit_message.as_deref().unwrap_or("updated page");
    let sha = generate_pseudo_sha();

    let row = match sqlx::query_as::<_, WikiPageResponse>(
        "UPDATE wiki_pages SET title = $1, content = $2, latest_commit = $3, updated_at = NOW() WHERE id = $4 RETURNING id, repo_id, slug, title, format, content, latest_commit, created_by, created_at, updated_at",
    )
    .bind(title)
    .bind(content)
    .bind(&sha)
    .bind(existing.id)
    .fetch_one(pool)
    .await
    {
        Ok(r) => r,
        Err(e) => return internal_err(&e.to_string()),
    };

    let _ = sqlx::query(
        "INSERT INTO wiki_revisions (page_id, commit_sha, author_id, edit_message, created_at) VALUES ($1, $2, 'system', $3, NOW())",
    )
    .bind(existing.id)
    .bind(&sha)
    .bind(edit_msg)
    .execute(pool)
    .await;

    (StatusCode::OK, Json(row)).into_response()
}

// ---------------------------------------------------------------------------
// 5. DELETE /wiki/:slug — delete wiki page
// ---------------------------------------------------------------------------

pub async fn delete_wiki_page(
    State(state): State<AppState>,
    Path((owner, name, slug)): Path<(String, String, String)>,
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

    let result = sqlx::query("DELETE FROM wiki_pages WHERE repo_id = $1 AND slug = $2")
        .bind(repo_id)
        .bind(&slug)
        .execute(pool)
        .await;

    match result {
        Ok(rows) if rows.rows_affected() == 0 => err_response(
            StatusCode::NOT_FOUND,
            &format!("wiki page '{slug}' not found"),
        ),
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 6. GET /wiki/:slug/history — page edit history
// ---------------------------------------------------------------------------

pub async fn wiki_page_history(
    State(state): State<AppState>,
    Path((owner, name, slug)): Path<(String, String, String)>,
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

    let page_id = match sqlx::query_scalar::<_, i64>(
        "SELECT id FROM wiki_pages WHERE repo_id = $1 AND slug = $2",
    )
    .bind(repo_id)
    .bind(&slug)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("wiki page '{slug}' not found"),
            );
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    match sqlx::query_as::<_, WikiRevisionResponse>(
        "SELECT id, page_id, commit_sha, author_id, edit_message, created_at FROM wiki_revisions WHERE page_id = $1 ORDER BY created_at DESC",
    )
    .bind(page_id)
    .fetch_all(pool)
    .await
    {
        Ok(revisions) => (StatusCode::OK, Json(revisions)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 7. GET /wiki/:slug/diff — diff between two revisions
// ---------------------------------------------------------------------------

pub async fn wiki_page_diff(
    State(state): State<AppState>,
    Path((owner, name, slug)): Path<(String, String, String)>,
    Query(params): Query<DiffParams>,
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

    let page_id = match sqlx::query_scalar::<_, i64>(
        "SELECT id FROM wiki_pages WHERE repo_id = $1 AND slug = $2",
    )
    .bind(repo_id)
    .bind(&slug)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => {
            return err_response(
                StatusCode::NOT_FOUND,
                &format!("wiki page '{slug}' not found"),
            );
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    let sha1 = match &params.sha1 {
        Some(s) => s.clone(),
        None => {
            return err_response(StatusCode::BAD_REQUEST, "sha1 query param is required");
        }
    };
    let sha2 = match &params.sha2 {
        Some(s) => s.clone(),
        None => {
            return err_response(StatusCode::BAD_REQUEST, "sha2 query param is required");
        }
    };

    let rev1 = match sqlx::query_scalar::<_, String>(
        "SELECT commit_sha FROM wiki_revisions WHERE page_id = $1 AND commit_sha = $2",
    )
    .bind(page_id)
    .bind(&sha1)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => {
            return err_response(StatusCode::NOT_FOUND, "revision sha1 not found");
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    let _ = rev1;

    let rev2 = match sqlx::query_scalar::<_, String>(
        "SELECT commit_sha FROM wiki_revisions WHERE page_id = $1 AND commit_sha = $2",
    )
    .bind(page_id)
    .bind(&sha2)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(_)) => true,
        Ok(None) => {
            return err_response(StatusCode::NOT_FOUND, "revision sha2 not found");
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    let _ = rev2;

    let diff_response = DiffResponse {
        sha1: sha1.clone(),
        sha2: sha2.clone(),
        diff: format!("diff between {sha1} and {sha2} (git-backed diff deferred to v2)"),
    };

    (StatusCode::OK, Json(diff_response)).into_response()
}

// ---------------------------------------------------------------------------
// 8. GET /wiki/:slug/raw — raw Markdown content
// ---------------------------------------------------------------------------

pub async fn wiki_page_raw(
    State(state): State<AppState>,
    Path((owner, name, slug)): Path<(String, String, String)>,
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
        "SELECT content FROM wiki_pages WHERE repo_id = $1 AND slug = $2",
    )
    .bind(repo_id)
    .bind(&slug)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(content)) => (StatusCode::OK, content).into_response(),
        Ok(None) => err_response(
            StatusCode::NOT_FOUND,
            &format!("wiki page '{slug}' not found"),
        ),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// 9. GET /wiki/search — search wiki pages
// ---------------------------------------------------------------------------

pub async fn search_wiki_pages(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(params): Query<SearchWikiParams>,
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

    let query = match &params.q {
        Some(q) if !q.trim().is_empty() => q.trim(),
        _ => return err_response(StatusCode::BAD_REQUEST, "query param 'q' is required"),
    };

    let pattern = format!("%{query}%");

    match sqlx::query_as::<_, WikiPageSummary>(
        "SELECT slug, title, updated_at FROM wiki_pages WHERE repo_id = $1 AND (title ILIKE $2 OR slug ILIKE $2 OR content ILIKE $2) ORDER BY updated_at DESC",
    )
    .bind(repo_id)
    .bind(&pattern)
    .fetch_all(pool)
    .await
    {
        Ok(pages) => (StatusCode::OK, Json(pages)).into_response(),
        Err(e) => internal_err(&e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// Route builder
// ---------------------------------------------------------------------------

pub fn wiki_routes() -> axum::Router<AppState> {
    use axum::routing::get;

    axum::Router::new()
        .route(
            "/api/v1/repos/{owner}/{name}/wiki",
            get(list_wiki_pages).post(create_wiki_page),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/wiki/search",
            get(search_wiki_pages),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/wiki/{slug}",
            get(get_wiki_page)
                .put(update_wiki_page)
                .delete(delete_wiki_page),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/wiki/{slug}/history",
            get(wiki_page_history),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/wiki/{slug}/diff",
            get(wiki_page_diff),
        )
        .route(
            "/api/v1/repos/{owner}/{name}/wiki/{slug}/raw",
            get(wiki_page_raw),
        )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_wiki_page_request_parse() {
        let json_str = "{\"slug\":\"getting-started\",\"title\":\"Getting Started\",\"content\":\"# Hello\\n\\nWelcome!\"}";
        let req: CreateWikiPageRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(req.slug, "getting-started");
        assert_eq!(req.title, "Getting Started");
        assert_eq!(req.content, "# Hello\n\nWelcome!");
    }

    #[test]
    fn test_create_wiki_page_request_defaults() {
        let req: CreateWikiPageRequest =
            serde_json::from_str(r#"{"slug":"a","title":"A","content":"x"}"#).unwrap();
        assert_eq!(req.slug, "a");
    }

    #[test]
    fn test_update_wiki_page_request_partial() {
        let req: UpdateWikiPageRequest =
            serde_json::from_str(r#"{"title":"New Title","edit_message":"fixed typo"}"#).unwrap();
        assert_eq!(req.title.as_deref(), Some("New Title"));
        assert!(req.content.is_none());
        assert_eq!(req.edit_message.as_deref(), Some("fixed typo"));
    }

    #[test]
    fn test_update_wiki_page_request_empty() {
        let req: UpdateWikiPageRequest = serde_json::from_str("{}").unwrap();
        assert!(req.title.is_none());
        assert!(req.content.is_none());
        assert!(req.edit_message.is_none());
    }

    #[test]
    fn test_diff_params_parse() {
        let p: DiffParams = serde_json::from_str(r#"{"sha1":"abc123","sha2":"def456"}"#).unwrap();
        assert_eq!(p.sha1.as_deref(), Some("abc123"));
        assert_eq!(p.sha2.as_deref(), Some("def456"));
    }

    #[test]
    fn test_diff_params_defaults() {
        let p: DiffParams = serde_json::from_str("{}").unwrap();
        assert!(p.sha1.is_none());
        assert!(p.sha2.is_none());
    }

    #[test]
    fn test_search_wiki_params_parse() {
        let p: SearchWikiParams = serde_json::from_str(r#"{"q":"installation"}"#).unwrap();
        assert_eq!(p.q.as_deref(), Some("installation"));
    }

    #[test]
    fn test_search_wiki_params_defaults() {
        let p: SearchWikiParams = serde_json::from_str("{}").unwrap();
        assert!(p.q.is_none());
    }

    #[test]
    fn test_slug_sanitization_basic() {
        assert_eq!(sanitize_slug("Getting Started"), "getting-started");
        assert_eq!(sanitize_slug("API Reference"), "api-reference");
        assert_eq!(sanitize_slug("hello-world"), "hello-world");
    }

    #[test]
    fn test_slug_sanitization_special_chars() {
        assert_eq!(sanitize_slug("Hello!! World??"), "hello-world");
        assert_eq!(sanitize_slug("a   b   c"), "a-b-c");
        assert_eq!(sanitize_slug("test_page"), "test_page");
        assert_eq!(sanitize_slug("CamelCase"), "camelcase");
    }

    #[test]
    fn test_slug_sanitization_empty_and_whitespace() {
        assert_eq!(sanitize_slug(""), "");
        assert_eq!(sanitize_slug("   "), "");
        assert_eq!(sanitize_slug("---"), "");
    }

    #[test]
    fn test_wiki_routes_compile() {
        let router = wiki_routes();
        let _ = router;
    }

    #[test]
    fn test_wiki_page_summary_fields() {
        let summary = WikiPageSummary {
            slug: "home".into(),
            title: "Home".into(),
            updated_at: chrono::Utc::now(),
        };
        assert_eq!(summary.slug, "home");
        assert_eq!(summary.title, "Home");
    }

    #[test]
    fn test_generate_pseudo_sha_format() {
        let sha = generate_pseudo_sha();
        assert_eq!(sha.len(), 40);
        assert!(sha.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_diff_response_fields() {
        let resp = DiffResponse {
            sha1: "abc".into(),
            sha2: "def".into(),
            diff: "some diff".into(),
        };
        assert_eq!(resp.sha1, "abc");
        assert_eq!(resp.sha2, "def");
        assert!(resp.diff.contains("some diff"));
    }

    #[test]
    fn test_search_query_pattern() {
        let pattern = format!("%{query}%", query = "install");
        assert_eq!(pattern, "%install%");
    }
}
