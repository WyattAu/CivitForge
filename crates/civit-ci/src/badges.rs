//! Pipeline status badge types and SVG generation.

#![forbid(unsafe_code)]

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BadgeQueryParams {
    pub branch: Option<String>,
}

pub fn badge_response(label: &str, fg: &str, bg: &str) -> (Vec<(&'static str, String)>, String) {
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

    let headers = vec![("content-type", "image/svg+xml; charset=utf-8".to_string())];
    (headers, svg)
}

pub async fn get_latest_pipeline_status(
    pool: &sqlx::PgPool,
    repo_id: uuid::Uuid,
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

pub async fn resolve_repo_id(
    pool: &sqlx::PgPool,
    owner: &str,
    repo_name: &str,
) -> std::result::Result<Option<uuid::Uuid>, sqlx::Error> {
    let row = sqlx::query_as::<_, (uuid::Uuid,)>(
        "SELECT r.id FROM repositories r JOIN users u ON r.owner_id = u.id WHERE u.username = $1 AND r.name = $2",
    )
    .bind(owner)
    .bind(repo_name)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_badge_svg_passing() {
        let (headers, svg) = badge_response("passing", "#4c1", "#333");
        assert!(headers.iter().any(|(k, v)| k == &"content-type" && v.contains("image/svg+xml")));
        assert!(svg.contains("passing"));
        assert!(svg.contains("#4c1"));
    }

    #[test]
    fn test_badge_svg_failing() {
        let (_, svg) = badge_response("failing", "#e05d44", "#333");
        assert!(svg.contains("failing"));
        assert!(svg.contains("#e05d44"));
    }

    #[test]
    fn test_badge_svg_dimensions() {
        let (_, svg) = badge_response("passing", "#4c1", "#333");
        assert!(svg.contains("width=\"73\""));
    }
}
