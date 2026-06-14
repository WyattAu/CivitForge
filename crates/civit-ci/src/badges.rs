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
        assert!(
            headers
                .iter()
                .any(|(k, v)| k == &"content-type" && v.contains("image/svg+xml"))
        );
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

    #[test]
    fn test_badge_svg_pending() {
        let (headers, svg) = badge_response("pending", "#dfb317", "#333");
        assert!(headers.iter().any(|(k, _v)| k == &"content-type"));
        assert!(svg.contains("pending"));
        assert!(svg.contains("#dfb317"));
    }

    #[test]
    fn test_badge_svg_unknown() {
        let (_, svg) = badge_response("unknown", "#9f9f9f", "#333");
        assert!(svg.contains("unknown"));
        assert!(svg.contains("#9f9f9f"));
    }

    #[test]
    fn test_badge_svg_is_valid_xml() {
        let (_, svg) = badge_response("passing", "#4c1", "#333");
        assert!(svg.starts_with("<?xml"));
        assert!(svg.contains("<svg"));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn test_badge_svg_contains_build_text() {
        let (_, svg) = badge_response("passing", "#4c1", "#333");
        assert!(svg.contains(">build<"));
    }

    #[test]
    fn test_badge_svg_width_scales_with_label() {
        // badge_response has a width formula that requires label.len()*7+24 >= 70
        // "passing" = 7*7+24=73, "failing!" = 8*7+24=80
        let (_, svg_short) = badge_response("passing", "#4c1", "#333");
        let (_, svg_long) = badge_response("failing!", "#e05d44", "#333");
        assert!(svg_short.contains("width=\"73\""));
        assert!(svg_long.contains("width=\"80\""));
    }

    #[test]
    fn test_badge_svg_color_params() {
        let (_, svg) = badge_response("passing", "#ff0000", "#0000ff");
        assert!(svg.contains("#ff0000"));
        assert!(svg.contains("#0000ff"));
    }

    #[test]
    fn test_badge_svg_special_chars_label() {
        let (_, svg) = badge_response("100 pct", "#4c1", "#333");
        assert!(svg.contains("100 pct"));
    }

    #[test]
    fn test_badge_response_headers_content_type() {
        let (headers, _) = badge_response("passing", "#4c1", "#333");
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0, "content-type");
        assert!(headers[0].1.contains("image/svg+xml"));
    }

    #[test]
    fn test_badge_query_params() {
        let params = BadgeQueryParams {
            branch: Some("main".into()),
        };
        assert_eq!(params.branch.as_deref(), Some("main"));
    }

    #[test]
    fn test_badge_query_params_no_branch() {
        let params = BadgeQueryParams { branch: None };
        assert!(params.branch.is_none());
    }

    #[test]
    fn test_badge_query_params_deserialize() {
        let json = r#"{"branch": "develop"}"#;
        let params: BadgeQueryParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.branch.as_deref(), Some("develop"));
    }

    #[test]
    fn test_badge_query_params_deserialize_empty() {
        let json = r#"{}"#;
        let params: BadgeQueryParams = serde_json::from_str(json).unwrap();
        assert!(params.branch.is_none());
    }
}
