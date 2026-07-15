//! Pipeline Action Reviews v14: Enhanced review system with helpfulness v14,
//! analytics v17, moderation v17, and recommendations v17.

#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineActionReviewV14 {
    pub id: Uuid,
    pub action_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub review: String,
    pub helpful_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewHelpfulnessV14 {
    pub id: Uuid,
    pub review_id: Uuid,
    pub user_id: Uuid,
    pub helpful: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewModerationQueueV14 {
    pub id: Uuid,
    pub review_id: Uuid,
    pub status: String,
    pub moderator_id: Option<Uuid>,
    pub reason: Option<String>,
    pub moderated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewAnalyticsV14 {
    pub id: Uuid,
    pub action_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub total_reviews: i32,
    pub avg_rating: f64,
    pub rating_distribution: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRecommendationV14 {
    pub id: Uuid,
    pub action_id: Uuid,
    pub user_id: Uuid,
    pub reason: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReviewRequestV14 {
    pub action_id: Uuid,
    pub rating: i32,
    pub review: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReviewRequestV14 {
    pub rating: Option<i32>,
    pub review: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerateReviewRequestV14 {
    pub status: String,
    pub reason: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct ReviewRowV14 {
    id: Uuid,
    action_id: Uuid,
    user_id: Uuid,
    rating: i32,
    review: String,
    helpful_count: i32,
    created_at: DateTime<Utc>,
}

impl From<ReviewRowV14> for PipelineActionReviewV14 {
    fn from(row: ReviewRowV14) -> Self {
        PipelineActionReviewV14 {
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
struct HelpfulnessRowV14 {
    id: Uuid,
    review_id: Uuid,
    user_id: Uuid,
    helpful: bool,
    created_at: DateTime<Utc>,
}

impl From<HelpfulnessRowV14> for ReviewHelpfulnessV14 {
    fn from(row: HelpfulnessRowV14) -> Self {
        ReviewHelpfulnessV14 {
            id: row.id,
            review_id: row.review_id,
            user_id: row.user_id,
            helpful: row.helpful,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ModerationRowV14 {
    id: Uuid,
    review_id: Uuid,
    status: String,
    moderator_id: Option<Uuid>,
    reason: Option<String>,
    moderated_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<ModerationRowV14> for ReviewModerationQueueV14 {
    fn from(row: ModerationRowV14) -> Self {
        ReviewModerationQueueV14 {
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
struct AnalyticsRowV14 {
    id: Uuid,
    action_id: Uuid,
    period_start: DateTime<Utc>,
    total_reviews: i32,
    avg_rating: f64,
    rating_distribution: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl From<AnalyticsRowV14> for ReviewAnalyticsV14 {
    fn from(row: AnalyticsRowV14) -> Self {
        ReviewAnalyticsV14 {
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
struct RecommendationRowV14 {
    id: Uuid,
    action_id: Uuid,
    user_id: Uuid,
    reason: String,
    confidence: f64,
    created_at: DateTime<Utc>,
}

impl From<RecommendationRowV14> for ReviewRecommendationV14 {
    fn from(row: RecommendationRowV14) -> Self {
        ReviewRecommendationV14 {
            id: row.id,
            action_id: row.action_id,
            user_id: row.user_id,
            reason: row.reason,
            confidence: row.confidence,
            created_at: row.created_at,
        }
    }
}

pub struct PipelineActionReviewsServiceV14 {
    pool: PgPool,
}

impl PipelineActionReviewsServiceV14 {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_review(
        &self,
        user_id: Uuid,
        request: CreateReviewRequestV14,
    ) -> Result<PipelineActionReviewV14, sqlx::Error> {
        let rating = request.rating.clamp(1, 5);
        let review = request.review.unwrap_or_default();

        let row = sqlx::query_as::<_, ReviewRowV14>(
            "INSERT INTO pipeline_action_reviews_v14 (action_id, user_id, rating, review, helpful_count, created_at)
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
    ) -> Result<Option<PipelineActionReviewV14>, sqlx::Error> {
        let row = sqlx::query_as::<_, ReviewRowV14>(
            "SELECT id, action_id, user_id, rating, review, helpful_count, created_at
             FROM pipeline_action_reviews_v14
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
    ) -> Result<Vec<PipelineActionReviewV14>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ReviewRowV14>(
            "SELECT id, action_id, user_id, rating, review, helpful_count, created_at
             FROM pipeline_action_reviews_v14
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
        request: UpdateReviewRequestV14,
    ) -> Result<PipelineActionReviewV14, sqlx::Error> {
        let row = sqlx::query_as::<_, ReviewRowV14>(
            "UPDATE pipeline_action_reviews_v14
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
            "DELETE FROM pipeline_action_reviews_v14 WHERE action_id = $1 AND user_id = $2",
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
    ) -> Result<ReviewHelpfulnessV14, sqlx::Error> {
        let row = sqlx::query_as::<_, HelpfulnessRowV14>(
            "INSERT INTO review_helpfulness_v14 (review_id, user_id, helpful, created_at)
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

        if helpful {
            sqlx::query(
                "UPDATE pipeline_action_reviews_v14 SET helpful_count = helpful_count + 1 WHERE id = $1",
            )
            .bind(review_id)
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query(
                "UPDATE pipeline_action_reviews_v14 SET helpful_count = GREATEST(helpful_count - 1, 0) WHERE id = $1",
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
        request: ModerateReviewRequestV14,
    ) -> Result<ReviewModerationQueueV14, sqlx::Error> {
        let row = sqlx::query_as::<_, ModerationRowV14>(
            "INSERT INTO review_moderation_queue_v14 (review_id, status, moderator_id, reason, moderated_at, created_at)
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
    ) -> Result<Vec<ReviewModerationQueueV14>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ModerationRowV14>(
            "SELECT id, review_id, status, moderator_id, reason, moderated_at, created_at
             FROM review_moderation_queue_v14
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
    ) -> Result<Vec<ReviewAnalyticsV14>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AnalyticsRowV14>(
            "SELECT id, action_id, period_start, total_reviews, avg_rating, rating_distribution, created_at
             FROM review_analytics_v14
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
    ) -> Result<ReviewAnalyticsV14, sqlx::Error> {
        let row = sqlx::query_as::<_, AnalyticsRowV14>(
            "INSERT INTO review_analytics_v14 (action_id, period_start, total_reviews, avg_rating, rating_distribution, created_at)
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
             FROM pipeline_action_reviews_v14
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
    ) -> Result<ReviewRecommendationV14, sqlx::Error> {
        let row = sqlx::query_as::<_, RecommendationRowV14>(
            "INSERT INTO review_recommendations_v14 (action_id, user_id, reason, confidence, created_at)
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
    ) -> Result<Vec<ReviewRecommendationV14>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RecommendationRowV14>(
            "SELECT id, action_id, user_id, reason, confidence, created_at
             FROM review_recommendations_v14
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
    fn test_review_v14_serialize() {
        let review = PipelineActionReviewV14 {
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
    fn test_create_review_request_v14_deserialize() {
        let json = r#"{"action_id": "550e8400-e29b-41d4-a716-446655440000", "rating": 4, "review": "Good"}"#;
        let req: CreateReviewRequestV14 = serde_json::from_str(json).unwrap();
        assert_eq!(req.rating, 4);
        assert_eq!(req.review, Some("Good".to_string()));
    }

    #[test]
    fn test_moderate_review_request_v14_deserialize() {
        let json = r#"{"status": "approved", "reason": "Helpful review"}"#;
        let req: ModerateReviewRequestV14 = serde_json::from_str(json).unwrap();
        assert_eq!(req.status, "approved");
        assert_eq!(req.reason, Some("Helpful review".to_string()));
    }

    #[test]
    fn test_helpfulness_v14_serialize() {
        let h = ReviewHelpfulnessV14 {
            id: Uuid::new_v4(),
            review_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            helpful: true,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&h).unwrap();
        assert!(json.contains("helpful"));
    }

    #[test]
    fn test_recommendation_v14_serialize() {
        let r = ReviewRecommendationV14 {
            id: Uuid::new_v4(),
            action_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            reason: "Based on your usage".to_string(),
            confidence: 0.85,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("0.85"));
    }
}
