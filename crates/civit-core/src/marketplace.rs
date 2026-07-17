#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MarketplaceListing {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: String,
    pub listing_type: String,
    pub author_id: Uuid,
    pub version: String,
    pub config: Value,
    pub downloads: i32,
    pub rating: f64,
    pub rating_count: i32,
    pub verified: bool,
    pub featured: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MarketplaceReview {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub review: String,
    pub helpful_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MarketplaceDownload {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub user_id: Uuid,
    pub version: String,
    pub downloaded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceAnalytics {
    pub total_downloads: i32,
    pub avg_rating: f64,
    pub rating_count: i32,
    pub review_count: i64,
    pub recent_downloads: i64,
    pub unique_downloaders: i64,
}

pub struct MarketplaceService {
    pool: PgPool,
}

impl MarketplaceService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn publish_listing(
        &self,
        name: &str,
        description: &str,
        category: &str,
        listing_type: &str,
        author_id: Uuid,
        version: &str,
        config: Value,
    ) -> Result<MarketplaceListing, sqlx::Error> {
        let row = sqlx::query_as::<_, MarketplaceListing>(
            r#"
            INSERT INTO marketplace_listings_v1 (name, description, category, listing_type, author_id, version, config)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, name, description, category, listing_type, author_id, version, config,
                      downloads, rating, rating_count, verified, featured, created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(category)
        .bind(listing_type)
        .bind(author_id)
        .bind(version)
        .bind(config)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn update_listing(
        &self,
        listing_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        category: Option<&str>,
        version: Option<&str>,
        config: Option<Value>,
    ) -> Result<MarketplaceListing, sqlx::Error> {
        let row = sqlx::query_as::<_, MarketplaceListing>(
            r#"
            UPDATE marketplace_listings_v1
            SET name = COALESCE($2, name),
                description = COALESCE($3, description),
                category = COALESCE($4, category),
                version = COALESCE($5, version),
                config = COALESCE($6, config),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, description, category, listing_type, author_id, version, config,
                      downloads, rating, rating_count, verified, featured, created_at, updated_at
            "#,
        )
        .bind(listing_id)
        .bind(name)
        .bind(description)
        .bind(category)
        .bind(version)
        .bind(config)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn uninstall_listing(&self, listing_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM marketplace_listings_v1 WHERE id = $1")
            .bind(listing_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn search_listings(
        &self,
        query: &str,
        category: Option<&str>,
        listing_type: Option<&str>,
    ) -> Result<Vec<MarketplaceListing>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MarketplaceListing>(
            r#"
            SELECT id, name, description, category, listing_type, author_id, version, config,
                   downloads, rating, rating_count, verified, featured, created_at, updated_at
            FROM marketplace_listings_v1
            WHERE ($1 = '' OR name ILIKE '%' || $1 || '%' OR description ILIKE '%' || $1 || '%')
              AND ($2 IS NULL OR category = $2)
              AND ($3 IS NULL OR listing_type = $3)
            ORDER BY downloads DESC, rating DESC
            "#,
        )
        .bind(query)
        .bind(category)
        .bind(listing_type)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_featured_listings(&self) -> Result<Vec<MarketplaceListing>, sqlx::Error> {
        let rows = sqlx::query_as::<_, MarketplaceListing>(
            r#"
            SELECT id, name, description, category, listing_type, author_id, version, config,
                   downloads, rating, rating_count, verified, featured, created_at, updated_at
            FROM marketplace_listings_v1
            WHERE featured = true
            ORDER BY downloads DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn rate_listing(
        &self,
        listing_id: Uuid,
        user_id: Uuid,
        rating: i32,
        review: &str,
    ) -> Result<MarketplaceReview, sqlx::Error> {
        let row = sqlx::query_as::<_, MarketplaceReview>(
            r#"
            INSERT INTO marketplace_reviews_v1 (listing_id, user_id, rating, review)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (listing_id, user_id)
            DO UPDATE SET rating = $3, review = $4
            RETURNING id, listing_id, user_id, rating, review, helpful_count, created_at
            "#,
        )
        .bind(listing_id)
        .bind(user_id)
        .bind(rating)
        .bind(review)
        .fetch_one(&self.pool)
        .await?;

        sqlx::query(
            r#"
            UPDATE marketplace_listings_v1
            SET rating = (SELECT COALESCE(AVG(rating::double precision), 0.0) FROM marketplace_reviews_v1 WHERE listing_id = $1),
                rating_count = (SELECT COUNT(*)::integer FROM marketplace_reviews_v1 WHERE listing_id = $1),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(listing_id)
        .execute(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn record_download(
        &self,
        listing_id: Uuid,
        user_id: Uuid,
        version: &str,
    ) -> Result<MarketplaceDownload, sqlx::Error> {
        let row = sqlx::query_as::<_, MarketplaceDownload>(
            r#"
            INSERT INTO marketplace_downloads_v1 (listing_id, user_id, version)
            VALUES ($1, $2, $3)
            RETURNING id, listing_id, user_id, version, downloaded_at
            "#,
        )
        .bind(listing_id)
        .bind(user_id)
        .bind(version)
        .fetch_one(&self.pool)
        .await?;

        sqlx::query(
            r#"
            UPDATE marketplace_listings_v1
            SET downloads = downloads + 1, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(listing_id)
        .execute(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_listing_analytics(
        &self,
        listing_id: Uuid,
    ) -> Result<MarketplaceAnalytics, sqlx::Error> {
        let row = sqlx::query_as::<_, (i32, f64, i32)>(
            r#"
            SELECT downloads, rating, rating_count
            FROM marketplace_listings_v1
            WHERE id = $1
            "#,
        )
        .bind(listing_id)
        .fetch_one(&self.pool)
        .await?;

        let (total_downloads, avg_rating, rating_count) = row;

        let review_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*)::bigint FROM marketplace_reviews_v1 WHERE listing_id = $1",
        )
        .bind(listing_id)
        .fetch_one(&self.pool)
        .await?;

        let recent_downloads: (i64,) = sqlx::query_as(
            "SELECT COUNT(*)::bigint FROM marketplace_downloads_v1 WHERE listing_id = $1 AND downloaded_at > NOW() - INTERVAL '30 days'",
        )
        .bind(listing_id)
        .fetch_one(&self.pool)
        .await?;

        let unique_downloaders: (i64,) = sqlx::query_as(
            "SELECT COUNT(DISTINCT user_id)::bigint FROM marketplace_downloads_v1 WHERE listing_id = $1",
        )
        .bind(listing_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(MarketplaceAnalytics {
            total_downloads,
            avg_rating,
            rating_count,
            review_count: review_count.0,
            recent_downloads: recent_downloads.0,
            unique_downloaders: unique_downloaders.0,
        })
    }

    pub async fn verify_listing(&self, listing_id: Uuid) -> Result<MarketplaceListing, sqlx::Error> {
        let row = sqlx::query_as::<_, MarketplaceListing>(
            r#"
            UPDATE marketplace_listings_v1
            SET verified = true, updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, description, category, listing_type, author_id, version, config,
                      downloads, rating, rating_count, verified, featured, created_at, updated_at
            "#,
        )
        .bind(listing_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }
}
