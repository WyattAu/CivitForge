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

// ---------------------------------------------------------------------------
// Action Reviews
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionReviewResponse {
    pub id: String,
    pub action_id: String,
    pub user_id: String,
    pub rating: i32,
    pub review: String,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ActionReviewRow {
    pub id: Uuid,
    pub action_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub review: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<ActionReviewRow> for ActionReviewResponse {
    fn from(r: ActionReviewRow) -> Self {
        Self {
            id: r.id.to_string(),
            action_id: r.action_id.to_string(),
            user_id: r.user_id.to_string(),
            rating: r.rating,
            review: r.review,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

/// Create or update a review for a pipeline action.
pub async fn upsert_action_review(
    pool: &sqlx::PgPool,
    action_id: Uuid,
    user_id: Uuid,
    rating: i32,
    review: &str,
) -> std::result::Result<ActionReviewResponse, sqlx::Error> {
    sqlx::query_as::<_, ActionReviewRow>(
        "INSERT INTO pipeline_action_reviews (action_id, user_id, rating, review) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (action_id, user_id) DO UPDATE \
         SET rating = $3, review = $4 \
         RETURNING *",
    )
    .bind(action_id)
    .bind(user_id)
    .bind(rating)
    .bind(review)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Get a user's review for an action.
pub async fn get_action_review(
    pool: &sqlx::PgPool,
    action_id: Uuid,
    user_id: Uuid,
) -> std::result::Result<Option<ActionReviewResponse>, sqlx::Error> {
    sqlx::query_as::<_, ActionReviewRow>(
        "SELECT * FROM pipeline_action_reviews WHERE action_id = $1 AND user_id = $2",
    )
    .bind(action_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map(|r| r.map(|r| r.into()))
}

/// List reviews for an action.
pub async fn list_action_reviews(
    pool: &sqlx::PgPool,
    action_id: Uuid,
    limit: i64,
    offset: i64,
) -> std::result::Result<Vec<ActionReviewResponse>, sqlx::Error> {
    sqlx::query_as::<_, ActionReviewRow>(
        "SELECT * FROM pipeline_action_reviews WHERE action_id = $1 \
         ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(action_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Delete a user's review for an action.
pub async fn delete_action_review(
    pool: &sqlx::PgPool,
    action_id: Uuid,
    user_id: Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM pipeline_action_reviews WHERE action_id = $1 AND user_id = $2",
    )
    .bind(action_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Get average rating for an action from reviews.
pub async fn get_action_average_rating(
    pool: &sqlx::PgPool,
    action_id: Uuid,
) -> std::result::Result<f64, sqlx::Error> {
    let row: (Option<f64>,) = sqlx::query_as(
        "SELECT COALESCE(AVG(rating), 0) FROM pipeline_action_reviews WHERE action_id = $1",
    )
    .bind(action_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0.unwrap_or(0.0))
}

/// Refresh action rating from reviews.
pub async fn refresh_action_rating(
    pool: &sqlx::PgPool,
    action_id: Uuid,
) -> std::result::Result<(), sqlx::Error> {
    let avg = get_action_average_rating(pool, action_id).await?;
    sqlx::query("UPDATE pipeline_actions SET rating = $2, updated_at = NOW() WHERE id = $1")
        .bind(action_id)
        .bind(avg)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Action Forks
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionForkResponse {
    pub id: String,
    pub action_id: String,
    pub forked_by: String,
    pub new_name: String,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ActionForkRow {
    pub id: Uuid,
    pub action_id: Uuid,
    pub forked_by: Uuid,
    pub new_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<ActionForkRow> for ActionForkResponse {
    fn from(r: ActionForkRow) -> Self {
        Self {
            id: r.id.to_string(),
            action_id: r.action_id.to_string(),
            forked_by: r.forked_by.to_string(),
            new_name: r.new_name,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

/// Fork a pipeline action.
pub async fn fork_action(
    pool: &sqlx::PgPool,
    action_id: Uuid,
    user_id: Uuid,
    new_name: &str,
) -> std::result::Result<PipelineActionResponse, sqlx::Error> {
    // Get the source action
    let source = sqlx::query_as::<_, PipelineActionRow>(
        "SELECT * FROM pipeline_actions WHERE id = $1",
    )
    .bind(action_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| sqlx::Error::RowNotFound)?;

    // Create the fork record
    sqlx::query(
        "INSERT INTO pipeline_action_forks (action_id, forked_by, new_name) VALUES ($1, $2, $3)",
    )
    .bind(action_id)
    .bind(user_id)
    .bind(new_name)
    .execute(pool)
    .await?;

    // Create the new action as a copy
    sqlx::query_as::<_, PipelineActionRow>(
        "INSERT INTO pipeline_actions (name, description, action_type, config, version, author_id) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING *",
    )
    .bind(new_name)
    .bind(&source.description)
    .bind(&source.action_type)
    .bind(&source.config)
    .bind(&source.version)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// List forks for an action.
pub async fn list_action_forks(
    pool: &sqlx::PgPool,
    action_id: Uuid,
) -> std::result::Result<Vec<ActionForkResponse>, sqlx::Error> {
    sqlx::query_as::<_, ActionForkRow>(
        "SELECT * FROM pipeline_action_forks WHERE action_id = $1 ORDER BY created_at DESC",
    )
    .bind(action_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Get recommended actions based on type and popularity.
pub async fn get_recommended_actions(
    pool: &sqlx::PgPool,
    action_id: Uuid,
    limit: i64,
) -> std::result::Result<Vec<PipelineActionResponse>, sqlx::Error> {
    let rows: Vec<PipelineActionRow> = sqlx::query_as(
        "SELECT * FROM pipeline_actions \
         WHERE action_type = (SELECT action_type FROM pipeline_actions WHERE id = $1) \
         AND id != $1 \
         ORDER BY downloads DESC, rating DESC \
         LIMIT $2",
    )
    .bind(action_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.into()).collect())
}

/// Get action analytics (downloads, reviews count, average rating).
pub async fn get_action_analytics(
    pool: &sqlx::PgPool,
    action_id: Uuid,
) -> std::result::Result<serde_json::Value, sqlx::Error> {
    let row: (i32, i64, f64) = sqlx::query_as(
        "SELECT \
            COALESCE(downloads, 0), \
            (SELECT COUNT(*) FROM pipeline_action_reviews WHERE action_id = $1), \
            COALESCE(rating, 0) \
         FROM pipeline_actions WHERE id = $1",
    )
    .bind(action_id)
    .fetch_one(pool)
    .await?;

    Ok(serde_json::json!({
        "action_id": action_id.to_string(),
        "downloads": row.0,
        "review_count": row.1,
        "average_rating": row.2
    }))
}

// ---------------------------------------------------------------------------
// Action Reviews V2
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionReviewV2Response {
    pub id: String,
    pub action_id: String,
    pub user_id: String,
    pub rating: i32,
    pub review: String,
    pub helpful_count: i32,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ActionReviewV2Row {
    pub id: Uuid,
    pub action_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub review: String,
    pub helpful_count: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<ActionReviewV2Row> for ActionReviewV2Response {
    fn from(r: ActionReviewV2Row) -> Self {
        Self {
            id: r.id.to_string(),
            action_id: r.action_id.to_string(),
            user_id: r.user_id.to_string(),
            rating: r.rating,
            review: r.review,
            helpful_count: r.helpful_count,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

/// Create or update a review v2 for a pipeline action.
pub async fn upsert_action_review_v2(
    pool: &sqlx::PgPool,
    action_id: Uuid,
    user_id: Uuid,
    rating: i32,
    review: &str,
) -> std::result::Result<ActionReviewV2Response, sqlx::Error> {
    sqlx::query_as::<_, ActionReviewV2Row>(
        "INSERT INTO pipeline_action_reviews_v2 (action_id, user_id, rating, review) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (action_id, user_id) DO UPDATE \
         SET rating = $3, review = $4 \
         RETURNING *",
    )
    .bind(action_id)
    .bind(user_id)
    .bind(rating)
    .bind(review)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Get a user's review v2 for an action.
pub async fn get_action_review_v2(
    pool: &sqlx::PgPool,
    action_id: Uuid,
    user_id: Uuid,
) -> std::result::Result<Option<ActionReviewV2Response>, sqlx::Error> {
    sqlx::query_as::<_, ActionReviewV2Row>(
        "SELECT * FROM pipeline_action_reviews_v2 WHERE action_id = $1 AND user_id = $2",
    )
    .bind(action_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map(|r| r.map(|r| r.into()))
}

/// List reviews v2 for an action.
pub async fn list_action_reviews_v2(
    pool: &sqlx::PgPool,
    action_id: Uuid,
    limit: i64,
    offset: i64,
) -> std::result::Result<Vec<ActionReviewV2Response>, sqlx::Error> {
    sqlx::query_as::<_, ActionReviewV2Row>(
        "SELECT * FROM pipeline_action_reviews_v2 WHERE action_id = $1 \
         ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(action_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Delete a user's review v2 for an action.
pub async fn delete_action_review_v2(
    pool: &sqlx::PgPool,
    action_id: Uuid,
    user_id: Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM pipeline_action_reviews_v2 WHERE action_id = $1 AND user_id = $2",
    )
    .bind(action_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Mark a review as helpful or not helpful.
pub async fn toggle_review_helpfulness(
    pool: &sqlx::PgPool,
    review_id: Uuid,
    user_id: Uuid,
    helpful: bool,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO review_helpfulness (review_id, user_id, helpful) \
         VALUES ($1, $2, $3) \
         ON CONFLICT (review_id, user_id) DO UPDATE \
         SET helpful = $3",
    )
    .bind(review_id)
    .bind(user_id)
    .bind(helpful)
    .execute(pool)
    .await?;

    // Update helpful count on the review
    sqlx::query(
        "UPDATE pipeline_action_reviews_v2 \
         SET helpful_count = (SELECT COUNT(*) FROM review_helpfulness WHERE review_id = $1 AND helpful = true) \
         WHERE id = $1",
    )
    .bind(review_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get review analytics for an action.
pub async fn get_review_analytics(
    pool: &sqlx::PgPool,
    action_id: Uuid,
) -> std::result::Result<serde_json::Value, sqlx::Error> {
    let row: (Option<i64>, Option<f64>, Option<i64>, Option<i64>) = sqlx::query_as(
        "SELECT \
            COUNT(*), \
            COALESCE(AVG(rating), 0), \
            SUM(helpful_count), \
            (SELECT COUNT(*) FROM review_moderation_queue rq \
             JOIN pipeline_action_reviews_v2 rv ON rq.review_id = rv.id \
             WHERE rv.action_id = $1 AND rq.status = 'pending') \
         FROM pipeline_action_reviews_v2 WHERE action_id = $1",
    )
    .bind(action_id)
    .fetch_one(pool)
    .await?;

    Ok(serde_json::json!({
        "action_id": action_id.to_string(),
        "total_reviews": row.0.unwrap_or(0),
        "average_rating": row.1.unwrap_or(0.0),
        "total_helpful_votes": row.2.unwrap_or(0),
        "pending_moderation": row.3.unwrap_or(0)
    }))
}

/// Submit a review to moderation queue.
pub async fn submit_review_for_moderation(
    pool: &sqlx::PgPool,
    review_id: Uuid,
    reason: &str,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO review_moderation_queue (review_id, reason) VALUES ($1, $2)",
    )
    .bind(review_id)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

/// Moderate a review (approve or reject).
pub async fn moderate_review(
    pool: &sqlx::PgPool,
    moderation_id: Uuid,
    moderator_id: Uuid,
    status: &str,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE review_moderation_queue \
         SET status = $2, moderator_id = $3, moderated_at = NOW() \
         WHERE id = $1",
    )
    .bind(moderation_id)
    .bind(status)
    .bind(moderator_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get recommended actions based on user's review history.
pub async fn get_recommendations_for_user(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    limit: i64,
) -> std::result::Result<Vec<PipelineActionResponse>, sqlx::Error> {
    let rows: Vec<PipelineActionRow> = sqlx::query_as(
        "SELECT pa.* FROM pipeline_actions pa \
         WHERE pa.id NOT IN (SELECT action_id FROM pipeline_action_reviews_v2 WHERE user_id = $1) \
         AND pa.action_type IN ( \
             SELECT DISTINCT action_type FROM pipeline_action_reviews_v2 \
             WHERE user_id = $1 AND rating >= 4 \
         ) \
         ORDER BY pa.rating DESC, pa.downloads DESC \
         LIMIT $2",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.into()).collect())
}

// ---------------------------------------------------------------------------
// Action Reviews V19
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionReviewV19Response {
    pub id: String,
    pub action_id: String,
    pub user_id: String,
    pub rating: i32,
    pub review: String,
    pub helpful_count: i32,
    pub created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ActionReviewV19Row {
    pub id: Uuid,
    pub action_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub review: String,
    pub helpful_count: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<ActionReviewV19Row> for ActionReviewV19Response {
    fn from(r: ActionReviewV19Row) -> Self {
        Self {
            id: r.id.to_string(),
            action_id: r.action_id.to_string(),
            user_id: r.user_id.to_string(),
            rating: r.rating,
            review: r.review,
            helpful_count: r.helpful_count,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

/// Create or update a review v19 for a pipeline action.
pub async fn upsert_action_review_v19(
    pool: &sqlx::PgPool,
    action_id: Uuid,
    user_id: Uuid,
    rating: i32,
    review: &str,
) -> std::result::Result<ActionReviewV19Response, sqlx::Error> {
    sqlx::query_as::<_, ActionReviewV19Row>(
        "INSERT INTO pipeline_action_reviews_v19 (action_id, user_id, rating, review) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (action_id, user_id) DO UPDATE \
         SET rating = $3, review = $4 \
         RETURNING *",
    )
    .bind(action_id)
    .bind(user_id)
    .bind(rating)
    .bind(review)
    .fetch_one(pool)
    .await
    .map(|r| r.into())
}

/// Get a user's review v19 for an action.
pub async fn get_action_review_v19(
    pool: &sqlx::PgPool,
    action_id: Uuid,
    user_id: Uuid,
) -> std::result::Result<Option<ActionReviewV19Response>, sqlx::Error> {
    sqlx::query_as::<_, ActionReviewV19Row>(
        "SELECT * FROM pipeline_action_reviews_v19 WHERE action_id = $1 AND user_id = $2",
    )
    .bind(action_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map(|r| r.map(|r| r.into()))
}

/// List reviews v19 for an action.
pub async fn list_action_reviews_v19(
    pool: &sqlx::PgPool,
    action_id: Uuid,
    limit: i64,
    offset: i64,
) -> std::result::Result<Vec<ActionReviewV19Response>, sqlx::Error> {
    sqlx::query_as::<_, ActionReviewV19Row>(
        "SELECT * FROM pipeline_action_reviews_v19 WHERE action_id = $1 \
         ORDER BY helpful_count DESC, created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(action_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
}

/// Delete a user's review v19 for an action.
pub async fn delete_action_review_v19(
    pool: &sqlx::PgPool,
    action_id: Uuid,
    user_id: Uuid,
) -> std::result::Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM pipeline_action_reviews_v19 WHERE action_id = $1 AND user_id = $2",
    )
    .bind(action_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Toggle helpfulness of a review v19.
pub async fn toggle_review_helpfulness_v19(
    pool: &sqlx::PgPool,
    review_id: Uuid,
    user_id: Uuid,
    helpful: bool,
) -> std::result::Result<(), sqlx::Error> {
    let delta = if helpful { 1 } else { -1 };
    sqlx::query(
        "UPDATE pipeline_action_reviews_v19 \
         SET helpful_count = GREATEST(0, helpful_count + $3) \
         WHERE id = $1",
    )
    .bind(review_id)
    .bind(user_id)
    .bind(delta)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get review analytics v22 (distribution, averages, trends).
pub async fn get_review_analytics_v22(
    pool: &sqlx::PgPool,
    action_id: Uuid,
) -> std::result::Result<serde_json::Value, sqlx::Error> {
    let rows: Vec<(i32, i64)> = sqlx::query_as(
        "SELECT rating, COUNT(*) as count \
         FROM pipeline_action_reviews_v19 WHERE action_id = $1 \
         GROUP BY rating ORDER BY rating",
    )
    .bind(action_id)
    .fetch_all(pool)
    .await?;

    let total_reviews: (i64, f64) = sqlx::query_as(
        "SELECT COUNT(*), COALESCE(AVG(rating), 0) \
         FROM pipeline_action_reviews_v19 WHERE action_id = $1",
    )
    .bind(action_id)
    .fetch_one(pool)
    .await?;

    let mut distribution = serde_json::Map::new();
    for (rating, count) in rows {
        distribution.insert(rating.to_string(), serde_json::Value::Number(count.into()));
    }

    Ok(serde_json::json!({
        "action_id": action_id.to_string(),
        "total_reviews": total_reviews.0,
        "average_rating": total_reviews.1,
        "distribution": distribution
    }))
}

/// Moderate a review v22 (flag/approve).
pub async fn moderate_review_v22(
    pool: &sqlx::PgPool,
    review_id: Uuid,
    moderator_id: Uuid,
    status: &str,
) -> std::result::Result<(), sqlx::Error> {
    match status {
        "approved" => {
            sqlx::query(
                "UPDATE pipeline_action_reviews_v19 SET review = review WHERE id = $1",
            )
            .bind(review_id)
            .execute(pool)
            .await?;
        }
        "flagged" | "removed" => {
            sqlx::query(
                "UPDATE pipeline_action_reviews_v19 SET review = '[moderated]' WHERE id = $1",
            )
            .bind(review_id)
            .execute(pool)
            .await?;
        }
        _ => {}
    }
    let _ = moderator_id;
    Ok(())
}

/// Get review-based recommendations v22.
pub async fn get_review_recommendations_v22(
    pool: &sqlx::PgPool,
    action_id: Uuid,
    limit: i64,
) -> std::result::Result<Vec<PipelineActionResponse>, sqlx::Error> {
    let rows: Vec<PipelineActionRow> = sqlx::query_as(
        "SELECT pa.* FROM pipeline_actions pa \
         JOIN pipeline_action_reviews_v19 par ON pa.id = par.action_id \
         WHERE pa.action_type = (SELECT action_type FROM pipeline_actions WHERE id = $1) \
         AND pa.id != $1 \
         GROUP BY pa.id \
         ORDER BY AVG(par.rating) DESC, SUM(par.helpful_count) DESC \
         LIMIT $2",
    )
    .bind(action_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.into()).collect())
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
