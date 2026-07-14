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

    pub async fn create_config(&self, strategy_id: Uuid, req: CreateStrategyConfigRequest) -> Result<StrategyConfig, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"INSERT INTO deployment_strategy_configs (id, strategy_id, config_key, config_value, created_at)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(id)
        .bind(strategy_id)
        .bind(&req.config_key)
        .bind(&req.config_value)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(StrategyConfig {
            id,
            strategy_id,
            config_key: req.config_key,
            config_value: req.config_value,
            created_at: now,
        })
    }

    pub async fn get_config(&self, id: Uuid) -> Result<Option<StrategyConfig>, sqlx::Error> {
        let row = sqlx::query_as::<_, StrategyConfigRow>(
            r#"SELECT id, strategy_id, config_key, config_value, created_at
               FROM deployment_strategy_configs WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(StrategyConfig::from))
    }

    pub async fn list_configs(&self, strategy_id: Uuid) -> Result<Vec<StrategyConfig>, sqlx::Error> {
        let rows = sqlx::query_as::<_, StrategyConfigRow>(
            r#"SELECT id, strategy_id, config_key, config_value, created_at
               FROM deployment_strategy_configs WHERE strategy_id = $1 ORDER BY created_at DESC"#,
        )
        .bind(strategy_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(StrategyConfig::from).collect())
    }

    pub async fn update_config(&self, id: Uuid, config_value: serde_json::Value) -> Result<StrategyConfig, sqlx::Error> {
        sqlx::query(r#"UPDATE deployment_strategy_configs SET config_value = $1 WHERE id = $2"#)
            .bind(&config_value)
            .bind(id)
            .execute(&self.pool)
            .await?;

        self.get_config(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn delete_config(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM deployment_strategy_configs WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn create_log(&self, strategy_id: Uuid, req: CreateStrategyLogRequest) -> Result<StrategyLog, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let status = req.status.unwrap_or_else(|| "success".to_string());
        let details = req.details.unwrap_or(serde_json::json!({}));

        sqlx::query(
            r#"INSERT INTO deployment_strategy_logs (id, strategy_id, action, status, details, created_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(id)
        .bind(strategy_id)
        .bind(&req.action)
        .bind(&status)
        .bind(&details)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(StrategyLog {
            id,
            strategy_id,
            action: req.action,
            status,
            details,
            created_at: now,
        })
    }

    pub async fn list_logs(&self, strategy_id: Uuid, limit: i64, offset: i64) -> Result<Vec<StrategyLog>, sqlx::Error> {
        let rows = sqlx::query_as::<_, StrategyLogRow>(
            r#"SELECT id, strategy_id, action, status, details, created_at
               FROM deployment_strategy_logs WHERE strategy_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"#,
        )
        .bind(strategy_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(StrategyLog::from).collect())
    }

    pub async fn test_strategy(&self, strategy_id: Uuid) -> Result<StrategyTestResult, sqlx::Error> {
        let strategy = self.get(strategy_id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;
        
        let start = std::time::Instant::now();
        let mut passed = true;
        let mut details = serde_json::json!({
            "strategy_type": strategy.strategy_type.to_string(),
            "config": strategy.config,
            "enabled": strategy.enabled
        });

        match strategy.strategy_type {
            StrategyType::Rolling => {
                if strategy.config.get("max_surge").is_none() {
                    passed = false;
                    details["error"] = serde_json::json!("missing max_surge config");
                }
            }
            StrategyType::BlueGreen => {
                if strategy.config.get("switch_over_time").is_none() {
                    passed = false;
                    details["error"] = serde_json::json!("missing switch_over_time config");
                }
            }
            StrategyType::Canary => {
                if strategy.config.get("step_percentage").is_none() {
                    passed = false;
                    details["error"] = serde_json::json!("missing step_percentage config");
                }
            }
            StrategyType::Recreate => {
                if strategy.config.get("termination_grace_period").is_none() {
                    passed = false;
                    details["error"] = serde_json::json!("missing termination_grace_period config");
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(StrategyTestResult {
            strategy_id,
            test_name: "config_validation".to_string(),
            passed,
            duration_ms,
            details,
        })
    }

    pub async fn compare_strategies(&self, strategy_a_id: Uuid, strategy_b_id: Uuid) -> Result<StrategyComparison, sqlx::Error> {
        let strategy_a = self.get(strategy_a_id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;
        let strategy_b = self.get(strategy_b_id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;

        let mut metrics = Vec::new();
        let mut a_score = 0.0;
        let mut b_score = 0.0;

        let complexity_a = match strategy_a.strategy_type {
            StrategyType::Rolling => 1.0,
            StrategyType::BlueGreen => 2.0,
            StrategyType::Canary => 3.0,
            StrategyType::Recreate => 0.5,
        };
        let complexity_b = match strategy_b.strategy_type {
            StrategyType::Rolling => 1.0,
            StrategyType::BlueGreen => 2.0,
            StrategyType::Canary => 3.0,
            StrategyType::Recreate => 0.5,
        };

        metrics.push(StrategyComparisonMetric {
            metric_name: "complexity".to_string(),
            strategy_a_value: complexity_a,
            strategy_b_value: complexity_b,
            winner: if complexity_a < complexity_b { "a".to_string() } else { "b".to_string() },
        });

        if complexity_a < complexity_b { a_score += 1.0; } else { b_score += 1.0; }

        let risk_a = match strategy_a.strategy_type {
            StrategyType::Rolling => 0.3,
            StrategyType::BlueGreen => 0.2,
            StrategyType::Canary => 0.4,
            StrategyType::Recreate => 0.8,
        };
        let risk_b = match strategy_b.strategy_type {
            StrategyType::Rolling => 0.3,
            StrategyType::BlueGreen => 0.2,
            StrategyType::Canary => 0.4,
            StrategyType::Recreate => 0.8,
        };

        metrics.push(StrategyComparisonMetric {
            metric_name: "risk".to_string(),
            strategy_a_value: risk_a,
            strategy_b_value: risk_b,
            winner: if risk_a < risk_b { "a".to_string() } else { "b".to_string() },
        });

        if risk_a < risk_b { a_score += 1.0; } else { b_score += 1.0; }

        let recommendation = if a_score > b_score {
            format!("Strategy A ({}) is recommended", strategy_a.name)
        } else if b_score > a_score {
            format!("Strategy B ({}) is recommended", strategy_b.name)
        } else {
            "Both strategies are equally suitable".to_string()
        };

        Ok(StrategyComparison {
            strategy_a_id,
            strategy_b_id,
            metrics,
            recommendation,
        })
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

#[derive(sqlx::FromRow)]
struct StrategyConfigRow {
    id: Uuid,
    strategy_id: Uuid,
    config_key: String,
    config_value: serde_json::Value,
    created_at: chrono::DateTime<Utc>,
}

impl From<StrategyConfigRow> for StrategyConfig {
    fn from(row: StrategyConfigRow) -> Self {
        Self {
            id: row.id,
            strategy_id: row.strategy_id,
            config_key: row.config_key,
            config_value: row.config_value,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct StrategyLogRow {
    id: Uuid,
    strategy_id: Uuid,
    action: String,
    status: String,
    details: serde_json::Value,
    created_at: chrono::DateTime<Utc>,
}

impl From<StrategyLogRow> for StrategyLog {
    fn from(row: StrategyLogRow) -> Self {
        Self {
            id: row.id,
            strategy_id: row.strategy_id,
            action: row.action,
            status: row.status,
            details: row.details,
            created_at: row.created_at,
        }
    }
}
