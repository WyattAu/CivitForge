//! Pipeline Actions Marketplace types and logic.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineActionResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub action_type: String,
    pub config: serde_json::Value,
    pub version: String,
    pub author_id: Option<String>,
    pub downloads: i32,
    pub rating: f64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct PipelineActionRow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub action_type: String,
    pub config: serde_json::Value,
    pub version: String,
    pub author_id: Option<Uuid>,
    pub downloads: i32,
    pub rating: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<PipelineActionRow> for PipelineActionResponse {
    fn from(r: PipelineActionRow) -> Self {
        Self {
            id: r.id.to_string(),
            name: r.name,
            description: r.description,
            action_type: r.action_type,
            config: r.config,
            version: r.version,
            author_id: r.author_id.map(|id| id.to_string()),
            downloads: r.downloads,
            rating: r.rating,
            created_at: r.created_at.to_rfc3339(),
            updated_at: r.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreatePipelineActionRequest {
    pub name: String,
    pub description: Option<String>,
    pub action_type: String,
    pub config: Option<serde_json::Value>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePipelineActionRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub action_type: Option<String>,
    pub config: Option<serde_json::Value>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PipelineActionListParams {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub action_type: Option<String>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
}

pub fn default_limit() -> i64 {
    50
}

// ---------------------------------------------------------------------------
// DB operations
// ---------------------------------------------------------------------------

/// Create a new pipeline action.
pub async fn create_pipeline_action(
    pool: &sqlx::PgPool,
    name: &str,
    description: &str,
    action_type: &str,
    config: &serde_json::Value,
    version: &str,
    author_id: Option<Uuid>,
) -> std::result::Result<PipelineActionResponse, sqlx::Error> {
    sqlx::query_as::<_, PipelineActionRow>(
        "INSERT INTO pipeline_actions (name, description, action_type, config, version, author_id) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING *",
    )
    .bind(name)
    .bind(description)
    .bind(action_type)
    .bind(config)
    .bind(version)
    .bind(author_id)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Get a pipeline action by ID.
pub async fn get_pipeline_action(
    pool: &sqlx::PgPool,
    action_id: Uuid,
) -> std::result::Result<Option<PipelineActionResponse>, sqlx::Error> {
    sqlx::query_as::<_, PipelineActionRow>("SELECT * FROM pipeline_actions WHERE id = $1")
        .bind(action_id)
        .fetch_optional(pool)
        .await
        .map(|r| r.map(|r| r.into()))
}

/// Update a pipeline action.
pub async fn update_pipeline_action(
    pool: &sqlx::PgPool,
    action_id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    action_type: Option<&str>,
    config: Option<&serde_json::Value>,
    version: Option<&str>,
) -> std::result::Result<PipelineActionResponse, sqlx::Error> {
    sqlx::query_as::<_, PipelineActionRow>(
        "UPDATE pipeline_actions \
         SET name = COALESCE($2, name), \
             description = COALESCE($3, description), \
             action_type = COALESCE($4, action_type), \
             config = COALESCE($5, config), \
             version = COALESCE($6, version), \
             updated_at = NOW() \
         WHERE id = $1 \
         RETURNING *",
    )
    .bind(action_id)
    .bind(name)
    .bind(description)
    .bind(action_type)
    .bind(config)
    .bind(version)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Delete a pipeline action.
pub async fn delete_pipeline_action(
    pool: &sqlx::PgPool,
    action_id: Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM pipeline_actions WHERE id = $1")
        .bind(action_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// List pipeline actions with filtering.
pub async fn list_pipeline_actions(
    pool: &sqlx::PgPool,
    limit: i64,
    offset: i64,
    action_type: Option<&str>,
    search: Option<&str>,
    sort_by: Option<&str>,
) -> std::result::Result<Vec<PipelineActionResponse>, sqlx::Error> {
    let order_clause = match sort_by {
        Some("downloads") => "downloads DESC",
        Some("rating") => "rating DESC",
        Some("name") => "name ASC",
        _ => "created_at DESC",
    };

    let mut builder = sqlx::QueryBuilder::new(format!(
        "SELECT id, name, description, action_type, config, version, author_id, downloads, rating, created_at, updated_at \
         FROM pipeline_actions WHERE 1=1"
    ));

    if let Some(at) = action_type {
        builder.push(" AND action_type = ").push_bind(at);
    }
    if let Some(s) = search {
        let pattern = format!("%{s}%");
        builder
            .push(" AND (name ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR description ILIKE ")
            .push_bind(pattern)
            .push(")");
    }

    builder.push(format!(" ORDER BY {order_clause} LIMIT "));
    builder.push_bind(limit);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    let rows: Vec<PipelineActionRow> = builder
        .build_query_as()
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(|r| r.into()).collect())
}

/// Track a download for a pipeline action.
pub async fn track_download(
    pool: &sqlx::PgPool,
    action_id: Uuid,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query("UPDATE pipeline_actions SET downloads = downloads + 1, updated_at = NOW() WHERE id = $1")
        .bind(action_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Update rating for a pipeline action.
pub async fn update_rating(
    pool: &sqlx::PgPool,
    action_id: Uuid,
    rating: f64,
) -> std::result::Result<PipelineActionResponse, sqlx::Error> {
    sqlx::query_as::<_, PipelineActionRow>(
        "UPDATE pipeline_actions SET rating = $2, updated_at = NOW() WHERE id = $1 RETURNING *",
    )
    .bind(action_id)
    .bind(rating)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_action_response_serialize() {
        let resp = PipelineActionResponse {
            id: "00000000-0000-0000-0000-000000000001".to_string(),
            name: "docker-build".to_string(),
            description: "Build Docker images".to_string(),
            action_type: "docker".to_string(),
            config: serde_json::json!({"registry": "ghcr.io"}),
            version: "1.0.0".to_string(),
            author_id: Some("00000000-0000-0000-0000-000000000002".to_string()),
            downloads: 42,
            rating: 4.5,
            created_at: "2025-01-01T00:00:00+00:00".to_string(),
            updated_at: "2025-01-01T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("docker-build"));
        assert!(json.contains("4.5"));
    }

    #[test]
    fn test_create_request_deserialize() {
        let json = r#"{"name": "test-action", "action_type": "script", "description": "A test action"}"#;
        let req: CreatePipelineActionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "test-action");
        assert_eq!(req.action_type, "script");
    }

    #[test]
    fn test_list_params_defaults() {
        let json = r#"{}"#;
        let params: PipelineActionListParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.limit, 50);
        assert_eq!(params.offset, 0);
        assert!(params.action_type.is_none());
        assert!(params.search.is_none());
    }

    #[test]
    fn test_action_type_variants() {
        let types = ["docker", "script", "composite", "security", "deploy"];
        for t in types {
            let resp = PipelineActionResponse {
                id: "id".into(),
                name: "name".into(),
                description: "desc".into(),
                action_type: t.to_string(),
                config: serde_json::json!({}),
                version: "1.0.0".into(),
                author_id: None,
                downloads: 0,
                rating: 0.0,
                created_at: "2025-01-01T00:00:00Z".into(),
                updated_at: "2025-01-01T00:00:00Z".into(),
            };
            let json = serde_json::to_string(&resp).unwrap();
            assert!(json.contains(t));
        }
    }
}
