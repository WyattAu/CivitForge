//! Pipeline status badge API endpoints.

#![forbid(unsafe_code)]

use crate::api::AppState;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use civit_ci::badges::{self, BadgeQueryParams};

pub fn badge_routes() -> Router<AppState> {
    Router::new().route("/api/v1/repos/{owner}/{name}/badge.svg", get(badge_svg))
}

pub async fn badge_svg(
    State(state): State<AppState>,
    Path((owner, repo_name)): Path<(String, String)>,
    Query(params): Query<BadgeQueryParams>,
) -> Response {
    let pool = state.db.pool();

    let repo_id = match badges::resolve_repo_id(pool, &owner, &repo_name).await {
        Ok(Some(id)) => id,
        _ => return badge_response("no pipelines", "#959595", "#ccc"),
    };

    let status =
        match badges::get_latest_pipeline_status(pool, repo_id, params.branch.as_deref()).await {
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

fn badge_response(label: &str, fg: &str, bg: &str) -> Response {
    let (_headers, svg) = badges::badge_response(label, fg, bg);

    (
        [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
        svg,
    )
        .into_response()
}

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
}
