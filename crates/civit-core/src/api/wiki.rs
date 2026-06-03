#![forbid(unsafe_code)]

use crate::api::AppState;
use crate::api::auth::AuthUser;
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
    pub id: uuid::Uuid,
    pub repo_id: uuid::Uuid,
    pub slug: String,
    pub title: String,
    pub format: String,
    pub content: String,
    pub latest_commit: String,
    pub created_by: uuid::Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
#[allow(dead_code)]
pub struct WikiRevisionResponse {
    pub id: uuid::Uuid,
    pub page_id: uuid::Uuid,
    pub commit_sha: String,
    pub author_id: uuid::Uuid,
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
// Helper: generate unified diff between two text contents
// ---------------------------------------------------------------------------

/// Generate a unified diff between old and new content.
/// Uses a simple line-based diff algorithm (longest common subsequence of lines).
fn unified_diff(old: &str, new: &str) -> String {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    let hunks = compute_line_diff(&old_lines, &new_lines);

    let mut output = String::new();
    for hunk in &hunks {
        output.push_str(&hunk.to_string());
    }

    if output.is_empty() {
        output.push_str("--- No differences ---\n");
    }

    output
}

/// A single line in a diff hunk.
#[derive(Debug, Clone, PartialEq, Eq)]
enum DiffLine<'a> {
    Context(&'a str),
    Added(&'a str),
    Removed(&'a str),
}

/// A contiguous block of changes.
#[derive(Debug)]
struct DiffHunk<'a> {
    old_start: usize,
    old_count: usize,
    new_start: usize,
    new_count: usize,
    lines: Vec<DiffLine<'a>>,
}

impl<'a> std::fmt::Display for DiffHunk<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "@@ -{},{} +{},{} @@",
            self.old_start + 1,
            self.old_count,
            self.new_start + 1,
            self.new_count
        )?;
        for line in &self.lines {
            match line {
                DiffLine::Context(s) => writeln!(f, " {s}")?,
                DiffLine::Added(s) => writeln!(f, "+{s}")?,
                DiffLine::Removed(s) => writeln!(f, "-{s}")?,
            }
        }
        Ok(())
    }
}

/// Simple LCS-based line diff. Groups changes into hunks with up to 3 lines
/// of context between changes.
fn compute_line_diff<'a>(old: &[&'a str], new: &[&'a str]) -> Vec<DiffHunk<'a>> {
    if old.is_empty() && new.is_empty() {
        return vec![];
    }

    // Build edit script via LCS
    let lcs = lcs_lines(old, new);
    let mut hunks: Vec<DiffHunk<'a>> = Vec::new();
    let mut current_hunk: Option<DiffHunk<'a>> = None;

    let mut oi = 0usize;
    let mut ni = 0usize;
    let mut li = 0usize; // index into LCS

    while oi < old.len() || ni < new.len() {
        let old_done = oi >= old.len();
        let new_done = ni >= new.len();
        let lcs_done = li >= lcs.len();

        if !old_done && !new_done && !lcs_done && old[oi] == lcs[li] && new[ni] == lcs[li] {
            // Common line
            let line = DiffLine::Context(old[oi]);
            if let Some(ref mut h) = current_hunk {
                h.lines.push(line);
                h.old_count += 1;
                h.new_count += 1;
            }
            oi += 1;
            ni += 1;
            li += 1;
        } else {
            // Divergence
            let removed_start = oi;
            let added_start = ni;

            // Count removed lines (in old but not in LCS going forward)
            while oi < old.len() && (li >= lcs.len() || old[oi] != lcs[li]) {
                oi += 1;
            }

            // Count added lines (in new but not in LCS going forward)
            while ni < new.len() && (li >= lcs.len() || new[ni] != lcs[li]) {
                ni += 1;
            }

            // Advance LCS pointer past consumed common lines
            if li < lcs.len() {
                li += 1;
            }

            // Create or extend hunk
            if current_hunk.is_none() {
                current_hunk = Some(DiffHunk {
                    old_start: removed_start.saturating_sub(3).min(removed_start),
                    old_count: 0,
                    new_start: added_start.saturating_sub(3).min(added_start),
                    new_count: 0,
                    lines: Vec::new(),
                });
            }

            if let Some(ref mut h) = current_hunk {
                for line in &old[removed_start..oi] {
                    h.lines.push(DiffLine::Removed(line));
                    h.old_count += 1;
                }
                for line in &new[added_start..ni] {
                    h.lines.push(DiffLine::Added(line));
                    h.new_count += 1;
                }
            }
        }
    }

    if let Some(h) = current_hunk {
        hunks.push(h);
    }

    hunks
}

/// Compute the Longest Common Subsequence of two slices of string references.
fn lcs_lines<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<&'a str> {
    let m = a.len();
    let n = b.len();

    // For very large files, cap to avoid O(m*n) memory blowup
    if m > 5000 || n > 5000 {
        // Fall back to simple matching for large files
        let mut result = Vec::new();
        let mut bi = 0usize;
        for line in a {
            if bi < n && b[bi] == *line {
                result.push(*line);
                bi += 1;
            }
        }
        return result;
    }

    // Standard DP LCS
    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if a[i - 1] == b[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    // Backtrack to find the actual LCS
    let mut result = Vec::new();
    let (mut i, mut j) = (m, n);
    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            result.push(a[i - 1]);
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] > dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    result.reverse();
    result
}

/// Apply diff hunks to reconstruct content at revision sha2 from sha1.
#[allow(dead_code)]
fn apply_diff_hunks(old: &str, hunks: &[DiffHunk]) -> String {
    let old_lines: Vec<&str> = old.lines().collect();
    let mut new_lines = Vec::new();
    let mut old_idx = 0;

    for hunk in hunks {
        // Copy context lines before changes
        while old_idx < hunk.old_start && old_idx < old_lines.len() {
            new_lines.push(old_lines[old_idx].to_string());
            old_idx += 1;
        }

        for line in &hunk.lines {
            match line {
                DiffLine::Context(s) => new_lines.push((*s).to_string()),
                DiffLine::Removed(_) => {
                    old_idx += 1;
                }
                DiffLine::Added(s) => new_lines.push((*s).to_string()),
            }
        }

        // Advance old_idx past the consumed lines
        old_idx = hunk.old_start + hunk.old_count;
    }

    // Copy remaining lines
    while old_idx < old_lines.len() {
        new_lines.push(old_lines[old_idx].to_string());
        old_idx += 1;
    }

    new_lines.join("\n")
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
    _auth: AuthUser,
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
    _auth: AuthUser,
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
        "INSERT INTO wiki_revisions (page_id, commit_sha, author_id, edit_message, content_snapshot, created_at) VALUES ($1, $2, 'system', $3, $4, NOW())",
    )
    .bind(existing.id)
    .bind(&sha)
    .bind(edit_msg)
    .bind(content) // snapshot of content AFTER this edit
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

    let page_id = match sqlx::query_scalar::<_, uuid::Uuid>(
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

    let page_id = match sqlx::query_scalar::<_, uuid::Uuid>(
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

    let rev1_content: String = match sqlx::query_scalar::<_, Option<String>>(
        "SELECT content_snapshot FROM wiki_revisions WHERE page_id = $1 AND commit_sha = $2",
    )
    .bind(page_id)
    .bind(&sha1)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(Some(c))) => c,
        Ok(Some(None)) => {
            return err_response(
                StatusCode::NOT_FOUND,
                "revision sha1 has no content snapshot",
            );
        }
        Ok(None) => {
            return err_response(StatusCode::NOT_FOUND, "revision sha1 not found");
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    let rev2_content: String = match sqlx::query_scalar::<_, Option<String>>(
        "SELECT content_snapshot FROM wiki_revisions WHERE page_id = $1 AND commit_sha = $2",
    )
    .bind(page_id)
    .bind(&sha2)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(Some(c))) => c,
        Ok(Some(None)) => {
            return err_response(
                StatusCode::NOT_FOUND,
                "revision sha2 has no content snapshot",
            );
        }
        Ok(None) => {
            return err_response(StatusCode::NOT_FOUND, "revision sha2 not found");
        }
        Err(e) => return internal_err(&e.to_string()),
    };

    let diff = unified_diff(&rev1_content, &rev2_content);

    let diff_response = DiffResponse {
        sha1: sha1.clone(),
        sha2: sha2.clone(),
        diff,
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

    match sqlx::query_as::<_, WikiPageSummary>(
        "SELECT slug, title, updated_at FROM wiki_pages WHERE repo_id = $1 AND search_vector @@ plainto_tsquery($2) ORDER BY ts_rank(search_vector, plainto_tsquery($2)) DESC",
    )
    .bind(repo_id)
    .bind(query)
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

    #[test]
    fn test_unified_diff_no_changes() {
        let diff = unified_diff("line1\nline2\nline3", "line1\nline2\nline3");
        assert!(diff.contains("No differences"));
    }

    #[test]
    fn test_unified_diff_added_lines() {
        let old = "line1\nline3";
        let new = "line1\nline2\nline3";
        let diff = unified_diff(old, new);
        assert!(diff.contains("+line2"));
        assert!(diff.contains("@@"));
    }

    #[test]
    fn test_unified_diff_removed_lines() {
        let old = "line1\nline2\nline3";
        let new = "line1\nline3";
        let diff = unified_diff(old, new);
        assert!(diff.contains("-line2"));
    }

    #[test]
    fn test_unified_diff_replaced_lines() {
        let old = "line1\nold\nline3";
        let new = "line1\nnew\nline3";
        let diff = unified_diff(old, new);
        assert!(diff.contains("-old"));
        assert!(diff.contains("+new"));
    }

    #[test]
    fn test_unified_diff_empty_old() {
        let diff = unified_diff("", "hello\nworld");
        assert!(diff.contains("+hello"));
        assert!(diff.contains("+world"));
    }

    #[test]
    fn test_unified_diff_empty_new() {
        let diff = unified_diff("hello\nworld", "");
        assert!(diff.contains("-hello"));
        assert!(diff.contains("-world"));
    }

    #[test]
    fn test_unified_diff_both_empty() {
        let diff = unified_diff("", "");
        assert!(diff.contains("No differences"));
    }

    #[test]
    fn test_lcs_basic() {
        let a: Vec<&str> = vec!["a", "b", "c", "d"];
        let b: Vec<&str> = vec!["a", "c", "d", "e"];
        let lcs = lcs_lines(&a, &b);
        assert_eq!(lcs, vec!["a", "c", "d"]);
    }

    #[test]
    fn test_lcs_empty() {
        let a: Vec<&str> = vec![];
        let b: Vec<&str> = vec!["x", "y"];
        let lcs = lcs_lines(&a, &b);
        assert!(lcs.is_empty());
    }

    #[test]
    fn test_lcs_identical() {
        let a: Vec<&str> = vec!["a", "b", "c"];
        let b: Vec<&str> = vec!["a", "b", "c"];
        let lcs = lcs_lines(&a, &b);
        assert_eq!(lcs, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_lcs_large_fallback() {
        let a: Vec<&str> = (0..6000).map(|_| "line").collect();
        let b: Vec<&str> = (0..6000).map(|_| "line").collect();
        let lcs = lcs_lines(&a, &b);
        assert!(!lcs.is_empty());
    }

    #[test]
    fn test_diff_hunk_format() {
        let hunk = DiffHunk {
            old_start: 0,
            old_count: 1,
            new_start: 0,
            new_count: 1,
            lines: vec![DiffLine::Removed("old"), DiffLine::Added("new")],
        };
        let s = format!("{hunk}");
        assert!(s.starts_with("@@"));
        assert!(s.contains("-old"));
        assert!(s.contains("+new"));
    }
}
