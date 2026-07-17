use super::types::*;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct ResilienceTester {
    pool: PgPool,
}

impl ResilienceTester {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_test(&self, req: CreateTestRequest) -> Result<ResilienceTest, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let params = req.parameters.unwrap_or(serde_json::json!({}));
        let description = req.description.unwrap_or_default();

        sqlx::query(
            r#"INSERT INTO resilience_tests (id, name, description, test_type, target, parameters, status, score, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
        )
        .bind(id)
        .bind(&req.name)
        .bind(&description)
        .bind(req.test_type.to_string())
        .bind(&req.target)
        .bind(&params)
        .bind(TestStatus::Pending.to_string())
        .bind(0)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(ResilienceTest {
            id,
            name: req.name,
            description,
            test_type: req.test_type,
            target: req.target,
            parameters: params,
            status: TestStatus::Pending,
            score: 0,
            started_at: None,
            completed_at: None,
            created_at: now,
        })
    }

    pub async fn get_test(&self, id: Uuid) -> Result<Option<ResilienceTest>, sqlx::Error> {
        let row = sqlx::query_as::<_, ResilienceTestRow>(
            r#"SELECT id, name, description, test_type, target, parameters, status, score, started_at, completed_at, created_at
               FROM resilience_tests WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(ResilienceTest::from))
    }

    pub async fn list_tests(
        &self,
        status: Option<TestStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ResilienceTest>, sqlx::Error> {
        let rows = if let Some(status) = status {
            sqlx::query_as::<_, ResilienceTestRow>(
                r#"SELECT id, name, description, test_type, target, parameters, status, score, started_at, completed_at, created_at
                   FROM resilience_tests WHERE status = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"#,
            )
            .bind(status.to_string())
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ResilienceTestRow>(
                r#"SELECT id, name, description, test_type, target, parameters, status, score, started_at, completed_at, created_at
                   FROM resilience_tests ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows.into_iter().map(ResilienceTest::from).collect())
    }

    pub async fn start_test(&self, id: Uuid) -> Result<ResilienceTest, sqlx::Error> {
        let now = Utc::now();
        sqlx::query(r#"UPDATE resilience_tests SET status = $1, started_at = $2 WHERE id = $3"#)
            .bind(TestStatus::Running.to_string())
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?;

        self.get_test(id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn complete_test(
        &self,
        id: Uuid,
        score: i32,
        success: bool,
    ) -> Result<ResilienceTest, sqlx::Error> {
        let now = Utc::now();
        let status = if success {
            TestStatus::Completed
        } else {
            TestStatus::Failed
        };

        sqlx::query(
            r#"UPDATE resilience_tests SET status = $1, score = $2, completed_at = $3 WHERE id = $4"#,
        )
        .bind(status.to_string())
        .bind(score)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_test(id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn calculate_score(
        &self,
        test_type: TestType,
        target: &str,
    ) -> Result<TestResult, sqlx::Error> {
        let (score, recommendations) = match test_type {
            TestType::Retry => {
                let score = 85;
                let recs = vec![
                    "Implement exponential backoff for retries".to_string(),
                    "Add jitter to prevent thundering herd".to_string(),
                    "Set maximum retry count".to_string(),
                ];
                (score, recs)
            }
            TestType::Timeout => {
                let score = 90;
                let recs = vec![
                    "Configure appropriate timeout values".to_string(),
                    "Implement circuit breaker for repeated timeouts".to_string(),
                ];
                (score, recs)
            }
            TestType::Fallback => {
                let score = 75;
                let recs = vec![
                    "Implement graceful degradation".to_string(),
                    "Provide default responses for failures".to_string(),
                    "Cache fallback responses".to_string(),
                ];
                (score, recs)
            }
            TestType::Bulkhead => {
                let score = 80;
                let recs = vec![
                    "Isolate critical components".to_string(),
                    "Implement thread pool isolation".to_string(),
                    "Set connection limits per service".to_string(),
                ];
                (score, recs)
            }
            TestType::RateLimit => {
                let score = 88;
                let recs = vec![
                    "Implement token bucket algorithm".to_string(),
                    "Add rate limit headers to responses".to_string(),
                    "Configure per-user rate limits".to_string(),
                ];
                (score, recs)
            }
        };

        let details = serde_json::json!({
            "target": target,
            "test_type": test_type.to_string(),
            "score": score,
            "recommendations_count": recommendations.len(),
        });

        Ok(TestResult {
            test_id: Uuid::new_v4(),
            score,
            recommendations,
            details,
        })
    }

    pub async fn get_resilience_score(&self) -> Result<ResilienceScore, sqlx::Error> {
        let tests = self.list_tests(None, 100, 0).await?;

        let mut retry_score = 0;
        let mut timeout_score = 0;
        let mut fallback_score = 0;
        let mut bulkhead_score = 0;
        let mut rate_limit_score = 0;
        let mut count = 0;

        for test in &tests {
            if test.status == TestStatus::Completed {
                count += 1;
                match test.test_type {
                    TestType::Retry => retry_score = test.score,
                    TestType::Timeout => timeout_score = test.score,
                    TestType::Fallback => fallback_score = test.score,
                    TestType::Bulkhead => bulkhead_score = test.score,
                    TestType::RateLimit => rate_limit_score = test.score,
                }
            }
        }

        let overall = if count > 0 {
            (retry_score + timeout_score + fallback_score + bulkhead_score + rate_limit_score) / count
        } else {
            0
        };

        Ok(ResilienceScore {
            overall,
            retry_score,
            timeout_score,
            fallback_score,
            bulkhead_score,
            rate_limit_score,
        })
    }

    pub async fn delete_test(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM resilience_tests WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

#[derive(sqlx::FromRow)]
struct ResilienceTestRow {
    id: Uuid,
    name: String,
    description: String,
    test_type: String,
    target: String,
    parameters: serde_json::Value,
    status: String,
    score: i32,
    started_at: Option<chrono::DateTime<Utc>>,
    completed_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
}

impl From<ResilienceTestRow> for ResilienceTest {
    fn from(row: ResilienceTestRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            test_type: row.test_type.parse().unwrap_or(TestType::Retry),
            target: row.target,
            parameters: row.parameters,
            status: row.status.parse().unwrap_or(TestStatus::Pending),
            score: row.score,
            started_at: row.started_at,
            completed_at: row.completed_at,
            created_at: row.created_at,
        }
    }
}
