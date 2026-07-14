//! Pipeline Action Categories types and logic.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionCategoryResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parent_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ActionCategoryRow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub parent_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<ActionCategoryRow> for ActionCategoryResponse {
    fn from(r: ActionCategoryRow) -> Self {
        Self {
            id: r.id.to_string(),
            name: r.name,
            description: r.description,
            parent_id: r.parent_id.map(|id| id.to_string()),
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionCategoryMemberResponse {
    pub id: String,
    pub action_id: String,
    pub category_id: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ActionCategoryMemberRow {
    pub id: Uuid,
    pub action_id: Uuid,
    pub category_id: Uuid,
}

impl From<ActionCategoryMemberRow> for ActionCategoryMemberResponse {
    fn from(r: ActionCategoryMemberRow) -> Self {
        Self {
            id: r.id.to_string(),
            action_id: r.action_id.to_string(),
            category_id: r.category_id.to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// DB operations
// ---------------------------------------------------------------------------

/// Create a new action category.
pub async fn create_category(
    pool: &sqlx::PgPool,
    name: &str,
    description: &str,
    parent_id: Option<Uuid>,
) -> std::result::Result<ActionCategoryResponse, sqlx::Error> {
    sqlx::query_as::<_, ActionCategoryRow>(
        "INSERT INTO pipeline_action_categories (name, description, parent_id) \
         VALUES ($1, $2, $3) \
         RETURNING *",
    )
    .bind(name)
    .bind(description)
    .bind(parent_id)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Get an action category by ID.
pub async fn get_category(
    pool: &sqlx::PgPool,
    category_id: Uuid,
) -> std::result::Result<Option<ActionCategoryResponse>, sqlx::Error> {
    sqlx::query_as::<_, ActionCategoryRow>(
        "SELECT * FROM pipeline_action_categories WHERE id = $1",
    )
    .bind(category_id)
    .fetch_optional(pool)
    .await
    .map(|r| r.map(|r| r.into()))
}

/// Update an action category.
pub async fn update_category(
    pool: &sqlx::PgPool,
    category_id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    parent_id: Option<Option<Uuid>>,
) -> std::result::Result<ActionCategoryResponse, sqlx::Error> {
    sqlx::query_as::<_, ActionCategoryRow>(
        "UPDATE pipeline_action_categories \
         SET name = COALESCE($2, name), \
             description = COALESCE($3, description), \
             parent_id = $4 \
         WHERE id = $1 \
         RETURNING *",
    )
    .bind(category_id)
    .bind(name)
    .bind(description)
    .bind(parent_id)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Delete an action category.
pub async fn delete_category(
    pool: &sqlx::PgPool,
    category_id: Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM pipeline_action_categories WHERE id = $1")
        .bind(category_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// List all action categories.
pub async fn list_categories(
    pool: &sqlx::PgPool,
) -> std::result::Result<Vec<ActionCategoryResponse>, sqlx::Error> {
    sqlx::query_as::<_, ActionCategoryRow>(
        "SELECT * FROM pipeline_action_categories ORDER BY name ASC",
    )
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Search categories by name.
pub async fn search_categories(
    pool: &sqlx::PgPool,
    search: &str,
    limit: i64,
) -> std::result::Result<Vec<ActionCategoryResponse>, sqlx::Error> {
    let pattern = format!("%{search}%");
    sqlx::query_as::<_, ActionCategoryRow>(
        "SELECT * FROM pipeline_action_categories \
         WHERE name ILIKE $1 OR description ILIKE $1 \
         ORDER BY name ASC LIMIT $2",
    )
    .bind(pattern)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Add an action to a category.
pub async fn add_action_to_category(
    pool: &sqlx::PgPool,
    action_id: Uuid,
    category_id: Uuid,
) -> std::result::Result<ActionCategoryMemberResponse, sqlx::Error> {
    sqlx::query_as::<_, ActionCategoryMemberRow>(
        "INSERT INTO pipeline_action_category_members (action_id, category_id) \
         VALUES ($1, $2) \
         ON CONFLICT (action_id, category_id) DO NOTHING \
         RETURNING *",
    )
    .bind(action_id)
    .bind(category_id)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Remove an action from a category.
pub async fn remove_action_from_category(
    pool: &sqlx::PgPool,
    action_id: Uuid,
    category_id: Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM pipeline_action_category_members WHERE action_id = $1 AND category_id = $2",
    )
    .bind(action_id)
    .bind(category_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// List categories for an action.
pub async fn list_action_categories(
    pool: &sqlx::PgPool,
    action_id: Uuid,
) -> std::result::Result<Vec<ActionCategoryResponse>, sqlx::Error> {
    sqlx::query_as::<_, ActionCategoryRow>(
        "SELECT c.* FROM pipeline_action_categories c \
         JOIN pipeline_action_category_members m ON c.id = m.category_id \
         WHERE m.action_id = $1 \
         ORDER BY c.name ASC",
    )
    .bind(action_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// List actions in a category.
pub async fn list_category_actions(
    pool: &sqlx::PgPool,
    category_id: Uuid,
) -> std::result::Result<Vec<super::actions::PipelineActionResponse>, sqlx::Error> {
    sqlx::query_as::<_, super::actions::PipelineActionRow>(
        "SELECT a.* FROM pipeline_actions a \
         JOIN pipeline_action_category_members m ON a.id = m.action_id \
         WHERE m.category_id = $1 \
         ORDER BY a.downloads DESC",
    )
    .bind(category_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Get category analytics (action count, total downloads).
pub async fn get_category_analytics(
    pool: &sqlx::PgPool,
    category_id: Uuid,
) -> std::result::Result<serde_json::Value, sqlx::Error> {
    let row: (i64, i64) = sqlx::query_as(
        "SELECT \
            COUNT(m.id), \
            COALESCE(SUM(a.downloads), 0) \
         FROM pipeline_action_category_members m \
         LEFT JOIN pipeline_actions a ON m.action_id = a.id \
         WHERE m.category_id = $1",
    )
    .bind(category_id)
    .fetch_one(pool)
    .await?;

    Ok(serde_json::json!({
        "category_id": category_id.to_string(),
        "action_count": row.0,
        "total_downloads": row.1
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_response_serialize() {
        let resp = ActionCategoryResponse {
            id: "00000000-0000-0000-0000-000000000001".to_string(),
            name: "docker".to_string(),
            description: "Docker-related actions".to_string(),
            parent_id: None,
            created_at: "2025-01-01T00:00:00+00:00".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("docker"));
    }

    #[test]
    fn test_create_category_request() {
        let json = r#"{"name": "security", "description": "Security scanning actions"}"#;
        let req: CreateCategoryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "security");
        assert!(req.parent_id.is_none());
    }

    #[test]
    fn test_member_response_serialize() {
        let resp = ActionCategoryMemberResponse {
            id: "00000000-0000-0000-0000-000000000001".to_string(),
            action_id: "00000000-0000-0000-0000-000000000002".to_string(),
            category_id: "00000000-0000-0000-0000-000000000003".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("action_id"));
    }
}
