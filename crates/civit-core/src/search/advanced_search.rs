#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SearchIndexV2 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub file_path: String,
    pub content: String,
    pub language: String,
    pub line_count: i32,
    pub indexed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SearchQuery {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub query: String,
    pub result_count: i32,
    pub execution_time_ms: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file_path: String,
    pub content: String,
    pub language: String,
    pub line_number: i32,
    pub score: f64,
    pub highlighted_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchAnalytics {
    pub total_queries: i64,
    pub unique_users: i64,
    pub avg_execution_time_ms: f64,
    pub top_queries: Vec<(String, i64)>,
}

pub struct AdvancedSearchService {
    db: sqlx::PgPool,
}

impl AdvancedSearchService {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn index_file(
        &self,
        repo_id: Uuid,
        file_path: &str,
        content: &str,
        language: &str,
    ) -> Result<SearchIndexV2, sqlx::Error> {
        let id = Uuid::new_v4();
        let line_count = content.lines().count() as i32;

        let index = sqlx::query_as::<_, SearchIndexV2>(
            r#"
            INSERT INTO code_search_index_v2 (id, repo_id, file_path, content, language, line_count)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (repo_id, file_path) DO UPDATE
            SET content = EXCLUDED.content, language = EXCLUDED.language, line_count = EXCLUDED.line_count, indexed_at = NOW()
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(repo_id)
        .bind(file_path)
        .bind(content)
        .bind(language)
        .bind(line_count)
        .fetch_one(&self.db)
        .await?;

        Ok(index)
    }

    pub async fn search(
        &self,
        query: &str,
        repo_id: Option<Uuid>,
        language: Option<&str>,
        user_id: Option<Uuid>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<SearchResult>, sqlx::Error> {
        let start = std::time::Instant::now();

        let results = if let Some(repo_id) = repo_id {
            sqlx::query_as::<_, SearchIndexV2>(
                r#"
                SELECT * FROM code_search_index_v2
                WHERE repo_id = $1
                AND content ILIKE $2
                ORDER BY indexed_at DESC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(repo_id)
            .bind(format!("%{}%", query))
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.db)
            .await?
        } else if let Some(lang) = language {
            sqlx::query_as::<_, SearchIndexV2>(
                r#"
                SELECT * FROM code_search_index_v2
                WHERE language = $1
                AND content ILIKE $2
                ORDER BY indexed_at DESC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(lang)
            .bind(format!("%{}%", query))
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as::<_, SearchIndexV2>(
                r#"
                SELECT * FROM code_search_index_v2
                WHERE content ILIKE $1
                ORDER BY indexed_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(format!("%{}%", query))
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.db)
            .await?
        };

        let elapsed = start.elapsed().as_millis() as i32;

        let result_count = results.len() as i32;
        self.record_query(user_id, query, result_count, elapsed)
            .await?;

        let search_results = results
            .into_iter()
            .enumerate()
            .map(|(i, r)| {
                let highlighted = Self::highlight_match(&r.content, query);
                SearchResult {
                    file_path: r.file_path,
                    content: r.content,
                    language: r.language,
                    line_number: 0,
                    score: (result_count as usize - i) as f64,
                    highlighted_content: highlighted,
                }
            })
            .collect();

        Ok(search_results)
    }

    fn highlight_match(content: &str, query: &str) -> String {
        let lower_content = content.to_lowercase();
        let lower_query = query.to_lowercase();

        if let Some(pos) = lower_content.find(&lower_query) {
            let before = &content[..pos];
            let matched = &content[pos..pos + query.len()];
            let after = &content[pos + query.len()..];
            format!("{}**{}**{}", before, matched, after)
        } else {
            content.to_string()
        }
    }

    async fn record_query(
        &self,
        user_id: Option<Uuid>,
        query: &str,
        result_count: i32,
        execution_time_ms: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO code_search_queries (id, user_id, query, result_count, execution_time_ms)
            VALUES (gen_random_uuid(), $1, $2, $3, $4)
            "#,
        )
        .bind(user_id)
        .bind(query)
        .bind(result_count)
        .bind(execution_time_ms)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn get_analytics(&self) -> Result<SearchAnalytics, sqlx::Error> {
        let stats = sqlx::query_as::<_, (i64, i64, f64)>(
            r#"
            SELECT
                COUNT(*) as total_queries,
                COUNT(DISTINCT user_id) as unique_users,
                COALESCE(AVG(execution_time_ms), 0.0) as avg_execution_time_ms
            FROM code_search_queries
            "#,
        )
        .fetch_one(&self.db)
        .await?;

        let top_queries = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT query, COUNT(*) as count
            FROM code_search_queries
            GROUP BY query
            ORDER BY count DESC
            LIMIT 10
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(SearchAnalytics {
            total_queries: stats.0,
            unique_users: stats.1,
            avg_execution_time_ms: stats.2,
            top_queries,
        })
    }
}