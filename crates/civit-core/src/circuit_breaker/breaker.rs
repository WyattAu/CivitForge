use super::types::*;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct CircuitBreaker {
    pool: PgPool,
}

impl CircuitBreaker {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        config: CircuitBreakerConfig,
    ) -> Result<CircuitBreakerMetrics, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO circuit_breakers (id, name, state, failure_count, failure_threshold, success_threshold, timeout_seconds, last_state_change, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
        )
        .bind(id)
        .bind(&config.name)
        .bind(CircuitBreakerState::Closed.to_string())
        .bind(0)
        .bind(config.failure_threshold)
        .bind(config.success_threshold)
        .bind(config.timeout_seconds)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(CircuitBreakerMetrics {
            id,
            name: config.name,
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            failure_threshold: config.failure_threshold,
            success_threshold: config.success_threshold,
            timeout_seconds: config.timeout_seconds,
            last_failure_at: None,
            last_state_change: now,
            created_at: now,
        })
    }

    pub async fn get(&self, name: &str) -> Result<Option<CircuitBreakerMetrics>, sqlx::Error> {
        let row = sqlx::query_as::<_, CircuitBreakerRow>(
            r#"SELECT id, name, state, failure_count, failure_threshold, success_threshold, timeout_seconds, last_failure_at, last_state_change, created_at
               FROM circuit_breakers WHERE name = $1"#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(CircuitBreakerMetrics::from))
    }

    pub async fn list(&self) -> Result<Vec<CircuitBreakerMetrics>, sqlx::Error> {
        let rows = sqlx::query_as::<_, CircuitBreakerRow>(
            r#"SELECT id, name, state, failure_count, failure_threshold, success_threshold, timeout_seconds, last_failure_at, last_state_change, created_at
               FROM circuit_breakers ORDER BY name"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(CircuitBreakerMetrics::from).collect())
    }

    pub async fn record_success(&self, name: &str) -> Result<CircuitBreakerMetrics, sqlx::Error> {
        let breaker = self
            .get(name)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let now = Utc::now();

        match breaker.state {
            CircuitBreakerState::HalfOpen => {
                if breaker.failure_count >= breaker.success_threshold {
                    sqlx::query(
                        r#"UPDATE circuit_breakers SET state = $1, failure_count = 0, last_state_change = $2 WHERE name = $3"#,
                    )
                    .bind(CircuitBreakerState::Closed.to_string())
                    .bind(now)
                    .bind(name)
                    .execute(&self.pool)
                    .await?;
                }
            }
            CircuitBreakerState::Closed => {
                sqlx::query(
                    r#"UPDATE circuit_breakers SET failure_count = 0 WHERE name = $1"#,
                )
                .bind(name)
                .execute(&self.pool)
                .await?;
            }
            CircuitBreakerState::Open => {}
        }

        self.get(name)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn record_failure(&self, name: &str) -> Result<CircuitBreakerMetrics, sqlx::Error> {
        let breaker = self
            .get(name)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let now = Utc::now();

        match breaker.state {
            CircuitBreakerState::Closed => {
                let new_failure_count = breaker.failure_count + 1;
                if new_failure_count >= breaker.failure_threshold {
                    sqlx::query(
                        r#"UPDATE circuit_breakers SET state = $1, failure_count = $2, last_failure_at = $3, last_state_change = $3 WHERE name = $4"#,
                    )
                    .bind(CircuitBreakerState::Open.to_string())
                    .bind(new_failure_count)
                    .bind(now)
                    .bind(name)
                    .execute(&self.pool)
                    .await?;
                } else {
                    sqlx::query(
                        r#"UPDATE circuit_breakers SET failure_count = $1, last_failure_at = $2 WHERE name = $3"#,
                    )
                    .bind(new_failure_count)
                    .bind(now)
                    .bind(name)
                    .execute(&self.pool)
                    .await?;
                }
            }
            CircuitBreakerState::HalfOpen => {
                sqlx::query(
                    r#"UPDATE circuit_breakers SET state = $1, failure_count = 0, last_state_change = $2 WHERE name = $3"#,
                )
                .bind(CircuitBreakerState::Open.to_string())
                .bind(now)
                .bind(name)
                .execute(&self.pool)
                .await?;
            }
            CircuitBreakerState::Open => {}
        }

        self.get(name)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn can_execute(&self, name: &str) -> Result<bool, sqlx::Error> {
        let breaker = self
            .get(name)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        match breaker.state {
            CircuitBreakerState::Closed => Ok(true),
            CircuitBreakerState::Open => {
                if let Some(last_failure) = breaker.last_failure_at {
                    let elapsed = Utc::now() - last_failure;
                    if elapsed.num_seconds() >= breaker.timeout_seconds as i64 {
                        sqlx::query(
                            r#"UPDATE circuit_breakers SET state = $1, last_state_change = $2 WHERE name = $3"#,
                        )
                        .bind(CircuitBreakerState::HalfOpen.to_string())
                        .bind(Utc::now())
                        .bind(name)
                        .execute(&self.pool)
                        .await?;
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            CircuitBreakerState::HalfOpen => Ok(true),
        }
    }

    pub async fn get_status(&self, name: &str) -> Result<Option<CircuitBreakerStatus>, sqlx::Error> {
        let breaker = self.get(name).await?;

        Ok(breaker.map(|b| CircuitBreakerStatus {
            name: b.name,
            state: b.state,
            failure_count: b.failure_count,
            success_count: 0,
            last_failure_at: b.last_failure_at,
            last_state_change: b.last_state_change,
        }))
    }

    pub async fn delete(&self, name: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM circuit_breakers WHERE name = $1"#)
            .bind(name)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn reset(&self, name: &str) -> Result<CircuitBreakerMetrics, sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            r#"UPDATE circuit_breakers SET state = $1, failure_count = 0, last_state_change = $2 WHERE name = $3"#,
        )
        .bind(CircuitBreakerState::Closed.to_string())
        .bind(now)
        .bind(name)
        .execute(&self.pool)
        .await?;

        self.get(name)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }
}

#[derive(sqlx::FromRow)]
struct CircuitBreakerRow {
    id: Uuid,
    name: String,
    state: String,
    failure_count: i32,
    failure_threshold: i32,
    success_threshold: i32,
    timeout_seconds: i32,
    last_failure_at: Option<chrono::DateTime<Utc>>,
    last_state_change: chrono::DateTime<Utc>,
    created_at: chrono::DateTime<Utc>,
}

impl From<CircuitBreakerRow> for CircuitBreakerMetrics {
    fn from(row: CircuitBreakerRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            state: row.state.parse().unwrap_or(CircuitBreakerState::Closed),
            failure_count: row.failure_count,
            failure_threshold: row.failure_threshold,
            success_threshold: row.success_threshold,
            timeout_seconds: row.timeout_seconds,
            last_failure_at: row.last_failure_at,
            last_state_change: row.last_state_change,
            created_at: row.created_at,
        }
    }
}
