//! Pipeline Action Reviews v3: Advanced review system with helpfulness tracking,
//! analytics, moderation, and recommendations.

#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineActionReviewV3 {
    pub id: Uuid,
    pub action_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub review: String,
    pub helpful_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewHelpfulnessV2 {
    pub id: Uuid,
    pub review_id: Uuid,
    pub user_id: Uuid,
    pub helpful: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewModerationQueueV2 {
    pub id: Uuid,
    pub review_id: Uuid,
    pub status: String,
    pub moderator_id: Option<Uuid>,
    pub reason: Option<String>,
    pub moderated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewAnalyticsV2 {
    pub id: Uuid,
    pub action_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub total_reviews: i32,
    pub avg_rating: f64,
    pub rating_distribution: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRecommendationV2 {
    pub id: Uuid,
    pub action_id: Uuid,
    pub user_id: Uuid,
    pub reason: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReviewRequest {
    pub action_id: Uuid,
    pub rating: i32,
    pub review: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReviewRequest {
    pub rating: Option<i32>,
    pub review: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerateReviewRequest {
    pub status: String,
    pub reason: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct ReviewRowV3 {
    id: Uuid,
    action_id: Uuid,
    user_id: Uuid,
    rating: i32,
    review: String,
    helpful_count: i32,
    created_at: DateTime<Utc>,
}

impl From<ReviewRowV3> for PipelineActionReviewV3 {
    fn from(row: ReviewRowV3) -> Self {
        PipelineActionReviewV3 {
            id: row.id,
            action_id: row.action_id,
            user_id: row.user_id,
            rating: row.rating,
            review: row.review,
            helpful_count: row.helpful_count,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct HelpfulnessRowV2 {
    id: Uuid,
    review_id: Uuid,
    user_id: Uuid,
    helpful: bool,
    created_at: DateTime<Utc>,
}

impl From<HelpfulnessRowV2> for ReviewHelpfulnessV2 {
    fn from(row: HelpfulnessRowV2) -> Self {
        ReviewHelpfulnessV2 {
            id: row.id,
            review_id: row.review_id,
            user_id: row.user_id,
            helpful: row.helpful,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ModerationRowV2 {
    id: Uuid,
    review_id: Uuid,
    status: String,
    moderator_id: Option<Uuid>,
    reason: Option<String>,
    moderated_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<ModerationRowV2> for ReviewModerationQueueV2 {
    fn from(row: ModerationRowV2) -> Self {
        ReviewModerationQueueV2 {
            id: row.id,
            review_id: row.review_id,
            status: row.status,
            moderator_id: row.moderator_id,
            reason: row.reason,
            moderated_at: row.moderated_at,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct AnalyticsRowV2 {
    id: Uuid,
    action_id: Uuid,
    period_start: DateTime<Utc>,
    total_reviews: i32,
    avg_rating: f64,
    rating_distribution: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl From<AnalyticsRowV2> for ReviewAnalyticsV2 {
    fn from(row: AnalyticsRowV2) -> Self {
        ReviewAnalyticsV2 {
            id: row.id,
            action_id: row.action_id,
            period_start: row.period_start,
            total_reviews: row.total_reviews,
            avg_rating: row.avg_rating,
            rating_distribution: row.rating_distribution,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct RecommendationRowV2 {
    id: Uuid,
    action_id: Uuid,
    user_id: Uuid,
    reason: String,
    confidence: f64,
    created_at: DateTime<Utc>,
}

impl From<RecommendationRowV2> for ReviewRecommendationV2 {
    fn from(row: RecommendationRowV2) -> Self {
        ReviewRecommendationV2 {
            id: row.id,
            action_id: row.action_id,
            user_id: row.user_id,
            reason: row.reason,
            confidence: row.confidence,
            created_at: row.created_at,
        }
    }
}

pub struct PipelineActionReviewsServiceV3 {
    pool: PgPool,
}

impl PipelineActionReviewsServiceV3 {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_review(
        &self,
        user_id: Uuid,
        request: CreateReviewRequest,
    ) -> Result<PipelineActionReviewV3, sqlx::Error> {
        let rating = request.rating.clamp(1, 5);
        let review = request.review.unwrap_or_default();

        let row = sqlx::query_as::<_, ReviewRowV3>(
            "INSERT INTO pipeline_action_reviews_v3 (action_id, user_id, rating, review, helpful_count, created_at)
             VALUES ($1, $2, $3, $4, 0, NOW())
             ON CONFLICT (action_id, user_id) DO UPDATE
             SET rating = EXCLUDED.rating, review = EXCLUDED.review
             RETURNING id, action_id, user_id, rating, review, helpful_count, created_at",
        )
        .bind(request.action_id)
        .bind(user_id)
        .bind(rating)
        .bind(&review)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_review(
        &self,
        action_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<PipelineActionReviewV3>, sqlx::Error> {
        let row = sqlx::query_as::<_, ReviewRowV3>(
            "SELECT id, action_id, user_id, rating, review, helpful_count, created_at
             FROM pipeline_action_reviews_v3
             WHERE action_id = $1 AND user_id = $2",
        )
        .bind(action_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn list_reviews_for_action(
        &self,
        action_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PipelineActionReviewV3>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReviewRowV3>(
            "SELECT id, action_id, user_id, rating, review, helpful_count, created_at
             FROM pipeline_action_reviews_v3
             WHERE action_id = $1
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3",
        )
        .bind(action_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_review(
        &self,
        action_id: Uuid,
        user_id: Uuid,
        request: UpdateReviewRequest,
    ) -> Result<PipelineActionReviewV3, sqlx::Error> {
        let row = sqlx::query_as::<_, ReviewRowV3>(
            "UPDATE pipeline_action_reviews_v3
             SET rating = COALESCE($3, rating),
                 review = COALESCE($4, review)
             WHERE action_id = $1 AND user_id = $2
             RETURNING id, action_id, user_id, rating, review, helpful_count, created_at",
        )
        .bind(action_id)
        .bind(user_id)
        .bind(request.rating)
        .bind(request.review)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_review(
        &self,
        action_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM pipeline_action_reviews_v3 WHERE action_id = $1 AND user_id = $2",
        )
        .bind(action_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn toggle_helpfulness(
        &self,
        review_id: Uuid,
        user_id: Uuid,
        helpful: bool,
    ) -> Result<ReviewHelpfulnessV2, sqlx::Error> {
        let row = sqlx::query_as::<_, HelpfulnessRowV2>(
            "INSERT INTO review_helpfulness_v2 (review_id, user_id, helpful, created_at)
             VALUES ($1, $2, $3, NOW())
             ON CONFLICT (review_id, user_id) DO UPDATE
             SET helpful = EXCLUDED.helpful
             RETURNING id, review_id, user_id, helpful, created_at",
        )
        .bind(review_id)
        .bind(user_id)
        .bind(helpful)
        .fetch_one(&self.pool)
        .await?;

        // Update helpful count on the review
        if helpful {
            sqlx::query(
                "UPDATE pipeline_action_reviews_v3 SET helpful_count = helpful_count + 1 WHERE id = $1",
            )
            .bind(review_id)
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query(
                "UPDATE pipeline_action_reviews_v3 SET helpful_count = GREATEST(helpful_count - 1, 0) WHERE id = $1",
            )
            .bind(review_id)
            .execute(&self.pool)
            .await?;
        }

        Ok(row.into())
    }

    pub async fn moderate_review(
        &self,
        review_id: Uuid,
        moderator_id: Uuid,
        request: ModerateReviewRequest,
    ) -> Result<ReviewModerationQueueV2, sqlx::Error> {
        let row = sqlx::query_as::<_, ModerationRowV2>(
            "INSERT INTO review_moderation_queue_v2 (review_id, status, moderator_id, reason, moderated_at, created_at)
             VALUES ($1, $2, $3, $4, NOW(), NOW())
             RETURNING id, review_id, status, moderator_id, reason, moderated_at, created_at",
        )
        .bind(review_id)
        .bind(&request.status)
        .bind(moderator_id)
        .bind(&request.reason)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_pending_moderations(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ReviewModerationQueueV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ModerationRowV2>(
            "SELECT id, review_id, status, moderator_id, reason, moderated_at, created_at
             FROM review_moderation_queue_v2
             WHERE status = 'pending'
             ORDER BY created_at ASC
             LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_review_analytics(
        &self,
        action_id: Uuid,
    ) -> Result<Vec<ReviewAnalyticsV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AnalyticsRowV2>(
            "SELECT id, action_id, period_start, total_reviews, avg_rating, rating_distribution, created_at
             FROM review_analytics_v2
             WHERE action_id = $1
             ORDER BY period_start DESC",
        )
        .bind(action_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn generate_review_analytics(
        &self,
        action_id: Uuid,
        period_start: DateTime<Utc>,
    ) -> Result<ReviewAnalyticsV2, sqlx::Error> {
        let row = sqlx::query_as::<_, AnalyticsRowV2>(
            "INSERT INTO review_analytics_v2 (action_id, period_start, total_reviews, avg_rating, rating_distribution, created_at)
             SELECT
                 $1 as action_id,
                 $2 as period_start,
                 COUNT(*)::INTEGER as total_reviews,
                 COALESCE(AVG(rating), 0)::NUMERIC(3,2) as avg_rating,
                 COALESCE(
                     jsonb_build_object(
                         '1', COUNT(*) FILTER (WHERE rating = 1),
                         '2', COUNT(*) FILTER (WHERE rating = 2),
                         '3', COUNT(*) FILTER (WHERE rating = 3),
                         '4', COUNT(*) FILTER (WHERE rating = 4),
                         '5', COUNT(*) FILTER (WHERE rating = 5)
                     ),
                     '{}'::jsonb
                 ) as rating_distribution,
                 NOW() as created_at
             FROM pipeline_action_reviews_v3
             WHERE action_id = $1 AND created_at >= $2
             RETURNING id, action_id, period_start, total_reviews, avg_rating, rating_distribution, created_at",
        )
        .bind(action_id)
        .bind(period_start)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn create_recommendation(
        &self,
        action_id: Uuid,
        user_id: Uuid,
        reason: &str,
        confidence: f64,
    ) -> Result<ReviewRecommendationV2, sqlx::Error> {
        let row = sqlx::query_as::<_, RecommendationRowV2>(
            "INSERT INTO review_recommendations_v2 (action_id, user_id, reason, confidence, created_at)
             VALUES ($1, $2, $3, $4, NOW())
             ON CONFLICT (action_id, user_id) DO UPDATE
             SET reason = EXCLUDED.reason, confidence = EXCLUDED.confidence
             RETURNING id, action_id, user_id, reason, confidence, created_at",
        )
        .bind(action_id)
        .bind(user_id)
        .bind(reason)
        .bind(confidence)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn get_recommendations_for_action(
        &self,
        action_id: Uuid,
    ) -> Result<Vec<ReviewRecommendationV2>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RecommendationRowV2>(
            "SELECT id, action_id, user_id, reason, confidence, created_at
             FROM review_recommendations_v2
             WHERE action_id = $1
             ORDER BY confidence DESC",
        )
        .bind(action_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_serialize() {
        let review = PipelineActionReviewV3 {
            id: Uuid::new_v4(),
            action_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            rating: 5,
            review: "Great action!".to_string(),
            helpful_count: 10,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&review).unwrap();
        assert!(json.contains("Great action!"));
    }

    #[test]
    fn test_create_review_request_deserialize() {
        let json = r#"{"action_id": "550e8400-e29b-41d4-a716-446655440000", "rating": 4, "review": "Good"}"#;
        let req: CreateReviewRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.rating, 4);
        assert_eq!(req.review, Some("Good".to_string()));
    }

    #[test]
    fn test_moderate_review_request_deserialize() {
        let json = r#"{"status": "approved", "reason": "Helpful review"}"#;
        let req: ModerateReviewRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.status, "approved");
        assert_eq!(req.reason, Some("Helpful review".to_string()));
    }
}
