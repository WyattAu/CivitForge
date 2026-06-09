//! Pipeline status badge API endpoints.

#![forbid(unsafe_code)]

use crate::api::AppState;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde::Deserialize;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct BadgeQueryParams {
    pub branch: Option<String>,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn badge_routes() -> Router<AppState> {
    Router::new().route("/api/v1/repos/{owner}/{name}/badge.svg", get(badge_svg))
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Returns an SVG badge showing the latest pipeline status for a repository.
pub async fn badge_svg(
    State(state): State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    Query(params): Query<BadgeQueryParams>,
) -> Response {
    let pool = state.db.pool();

    let repo_id = match resolve_repo_id(pool, &owner, &repo_name).await {
        Ok(Some(id)) => id,
        _ => return badge_response("no pipelines", "#959595", "#ccc"),
    };

    let status = match get_latest_pipeline_status(pool, repo_id, params.branch.as_deref()).await {
        Ok(Some(s)) => s,
        _ => return badge_response("no pipelines", "#959595", "#ccc"),
    };

    let (label, fg, bg) = match status.as_str() {
        "success" => ("passing", "#4c1", "#333"),
        "failure" | "failed" | "error" => ("failing", "#e05d44", "#333"),
        "pending" | "queued" | "running" => ("pending", "#dfb317", "#333"),
        _ => ("unknown", "#959595", "#ccc"),
    };

    badge_response(label, fg, bg)
}

// ---------------------------------------------------------------------------
// SVG generation
// ---------------------------------------------------------------------------

fn badge_response(label: &str, fg: &str, bg: &str) -> Response {
    let width = label.len() * 7 + 24;
    let svg = format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="20">
  <rect width="{width}" height="20" fill="{bg}"/>
  <rect x="0" y="0" width="70" height="20" fill="#555"/>
  <text x="35" y="14" font-family="DejaVu Sans,Verdana,Geneva,sans-serif" font-size="11" fill="#fff" text-anchor="middle">build</text>
  <text x="{text_x}" y="14" font-family="DejaVu Sans,Verdana,Geneva,sans-serif" font-size="11" fill="{fg}" text-anchor="middle">{label}</text>
</svg>"##,
        width = width,
        bg = bg,
        text_x = 70 + (width - 70) / 2,
        fg = fg,
        label = label,
    );

    (
        [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
        svg,
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

async fn resolve_repo_id(
    pool: &sqlx::PgPool,
    owner: &str,
    repo_name: &str,
) -> std::result::Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_as::<_, (Uuid,)>(
        "SELECT r.id FROM repositories r JOIN users u ON r.owner_id = u.id WHERE u.username = $1 AND r.name = $2",
    )
    .bind(owner)
    .bind(repo_name)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}

async fn get_latest_pipeline_status(
    pool: &sqlx::PgPool,
    repo_id: Uuid,
    branch: Option<&str>,
) -> std::result::Result<Option<String>, sqlx::Error> {
    let sql = if branch.is_some() {
        "SELECT status FROM pipeline_runs WHERE repo_id = $1 AND ref_name = $2 ORDER BY created_at DESC LIMIT 1"
    } else {
        "SELECT status FROM pipeline_runs WHERE repo_id = $1 ORDER BY created_at DESC LIMIT 1"
    };

    let mut query = sqlx::query_scalar::<_, String>(sql);
    query = query.bind(repo_id);
    if let Some(b) = branch {
        query = query.bind(b);
    }

    query.fetch_optional(pool).await
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_badge_svg_passing() {
        let resp = badge_response("passing", "#4c1", "#333");
        assert_eq!(
            resp.headers()
                .get(axum::http::header::CONTENT_TYPE)
                .unwrap(),
            "image/svg+xml; charset=utf-8"
        );
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let svg = String::from_utf8(body.to_vec()).unwrap();
        assert!(svg.contains("passing"));
        assert!(svg.contains("#4c1"));
    }

    #[tokio::test]
    async fn test_badge_svg_failing() {
        let resp = badge_response("failing", "#e05d44", "#333");
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let svg = String::from_utf8(body.to_vec()).unwrap();
        assert!(svg.contains("failing"));
        assert!(svg.contains("#e05d44"));
    }

    #[tokio::test]
    async fn test_badge_svg_pending() {
        let resp = badge_response("pending", "#dfb317", "#333");
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let svg = String::from_utf8(body.to_vec()).unwrap();
        assert!(svg.contains("pending"));
        assert!(svg.contains("#dfb317"));
    }

    #[tokio::test]
    async fn test_badge_svg_unknown() {
        let resp = badge_response("unknown", "#959595", "#ccc");
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let svg = String::from_utf8(body.to_vec()).unwrap();
        assert!(svg.contains("unknown"));
    }

    #[tokio::test]
    async fn test_badge_svg_dimensions() {
        let resp = badge_response("passing", "#4c1", "#333");
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let svg = String::from_utf8(body.to_vec()).unwrap();
        // "passing" = 7 chars, width = 7*7 + 24 = 73
        assert!(svg.contains("width=\"73\""));
    }
}
