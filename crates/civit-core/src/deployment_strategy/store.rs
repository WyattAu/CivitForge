use super::types::*;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct DeploymentStrategyStore {
    pool: PgPool,
}

impl DeploymentStrategyStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: CreateStrategyRequest) -> Result<DeploymentStrategy, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let config = req.config.unwrap_or(serde_json::json!({}));
        let enabled = req.enabled.unwrap_or(true);

        sqlx::query(
            r#"INSERT INTO deployment_strategies (id, repo_id, name, strategy_type, config, enabled, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        )
        .bind(id)
        .bind(req.repo_id)
        .bind(&req.name)
        .bind(req.strategy_type.to_string())
        .bind(&config)
        .bind(enabled)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(DeploymentStrategy {
            id,
            repo_id: req.repo_id,
            name: req.name,
            strategy_type: req.strategy_type,
            config,
            enabled,
            created_at: now,
        })
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<DeploymentStrategy>, sqlx::Error> {
        let row = sqlx::query_as::<_, StrategyRow>(
            r#"SELECT id, repo_id, name, strategy_type, config, enabled, created_at
               FROM deployment_strategies WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(DeploymentStrategy::from))
    }

    pub async fn list_by_repo(&self, repo_id: Uuid, limit: i64, offset: i64) -> Result<Vec<DeploymentStrategy>, sqlx::Error> {
        let rows = sqlx::query_as::<_, StrategyRow>(
            r#"SELECT id, repo_id, name, strategy_type, config, enabled, created_at
               FROM deployment_strategies WHERE repo_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"#,
        )
        .bind(repo_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(DeploymentStrategy::from).collect())
    }

    pub async fn update(&self, id: Uuid, req: UpdateStrategyRequest) -> Result<DeploymentStrategy, sqlx::Error> {
        let mut strategy = self.get(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;

        if let Some(name) = req.name {
            sqlx::query(r#"UPDATE deployment_strategies SET name = $1 WHERE id = $2"#)
                .bind(&name)
                .bind(id)
                .execute(&self.pool)
                .await?;
            strategy.name = name;
        }
        if let Some(strategy_type) = req.strategy_type {
            sqlx::query(r#"UPDATE deployment_strategies SET strategy_type = $1 WHERE id = $2"#)
                .bind(strategy_type.to_string())
                .bind(id)
                .execute(&self.pool)
                .await?;
            strategy.strategy_type = strategy_type;
        }
        if let Some(config) = req.config {
            sqlx::query(r#"UPDATE deployment_strategies SET config = $1 WHERE id = $2"#)
                .bind(&config)
                .bind(id)
                .execute(&self.pool)
                .await?;
            strategy.config = config;
        }
        if let Some(enabled) = req.enabled {
            sqlx::query(r#"UPDATE deployment_strategies SET enabled = $1 WHERE id = $2"#)
                .bind(enabled)
                .bind(id)
                .execute(&self.pool)
                .await?;
            strategy.enabled = enabled;
        }

        Ok(strategy)
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM deployment_strategies WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}

#[derive(sqlx::FromRow)]
struct StrategyRow {
    id: Uuid,
    repo_id: Uuid,
    name: String,
    strategy_type: String,
    config: serde_json::Value,
    enabled: bool,
    created_at: chrono::DateTime<Utc>,
}

impl From<StrategyRow> for DeploymentStrategy {
    fn from(row: StrategyRow) -> Self {
        Self {
            id: row.id,
            repo_id: row.repo_id,
            name: row.name,
            strategy_type: row.strategy_type.parse().unwrap_or(StrategyType::Rolling),
            config: row.config,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}
