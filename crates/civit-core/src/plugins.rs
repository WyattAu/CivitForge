#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Plugin {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub plugin_type: String,
    pub config_schema: Value,
    pub enabled: bool,
    pub installed_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PluginHook {
    pub id: Uuid,
    pub plugin_id: Uuid,
    pub hook_name: String,
    pub hook_type: String,
    pub endpoint_url: Option<String>,
    pub config: Value,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PluginLog {
    pub id: Uuid,
    pub plugin_id: Uuid,
    pub level: String,
    pub message: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

pub struct PluginService {
    pool: PgPool,
}

impl PluginService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn install_plugin(
        &self,
        name: &str,
        description: &str,
        version: &str,
        author: &str,
        plugin_type: &str,
        config_schema: Value,
    ) -> Result<Plugin, sqlx::Error> {
        let row = sqlx::query_as::<_, Plugin>(
            r#"
            INSERT INTO plugins_v1 (name, description, version, author, plugin_type, config_schema)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, description, version, author, plugin_type, config_schema,
                      enabled, installed_at, updated_at
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(version)
        .bind(author)
        .bind(plugin_type)
        .bind(config_schema)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn uninstall_plugin(&self, plugin_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM plugins_v1 WHERE id = $1")
            .bind(plugin_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn enable_plugin(&self, plugin_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE plugins_v1 SET enabled = true, updated_at = NOW() WHERE id = $1",
        )
        .bind(plugin_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn disable_plugin(&self, plugin_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE plugins_v1 SET enabled = false, updated_at = NOW() WHERE id = $1",
        )
        .bind(plugin_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn register_hook(
        &self,
        plugin_id: Uuid,
        hook_name: &str,
        hook_type: &str,
        endpoint_url: Option<&str>,
        config: Value,
    ) -> Result<PluginHook, sqlx::Error> {
        let row = sqlx::query_as::<_, PluginHook>(
            r#"
            INSERT INTO plugin_hooks_v1 (plugin_id, hook_name, hook_type, endpoint_url, config)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, plugin_id, hook_name, hook_type, endpoint_url, config,
                      priority, enabled, created_at
            "#,
        )
        .bind(plugin_id)
        .bind(hook_name)
        .bind(hook_type)
        .bind(endpoint_url)
        .bind(config)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn execute_hook(
        &self,
        hook_name: &str,
        payload: &Value,
    ) -> Result<Vec<PluginHook>, sqlx::Error> {
        let hooks = sqlx::query_as::<_, PluginHook>(
            r#"
            SELECT h.id, h.plugin_id, h.hook_name, h.hook_type, h.endpoint_url, h.config,
                   h.priority, h.enabled, h.created_at
            FROM plugin_hooks_v1 h
            JOIN plugins_v1 p ON h.plugin_id = p.id
            WHERE h.hook_name = $1 AND h.enabled = true AND p.enabled = true
            ORDER BY h.priority ASC
            "#,
        )
        .bind(hook_name)
        .fetch_all(&self.pool)
        .await?;

        for hook in &hooks {
            if let Some(ref url) = hook.endpoint_url {
                let client = reqwest::Client::new();
                let _ = client.post(url).json(payload).send().await;
            }
        }

        Ok(hooks)
    }

    pub async fn list_plugins(&self) -> Result<Vec<Plugin>, sqlx::Error> {
        let rows = sqlx::query_as::<_, Plugin>(
            r#"
            SELECT id, name, description, version, author, plugin_type, config_schema,
                   enabled, installed_at, updated_at
            FROM plugins_v1
            ORDER BY name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_plugin(&self, plugin_id: Uuid) -> Result<Option<Plugin>, sqlx::Error> {
        let row = sqlx::query_as::<_, Plugin>(
            r#"
            SELECT id, name, description, version, author, plugin_type, config_schema,
                   enabled, installed_at, updated_at
            FROM plugins_v1
            WHERE id = $1
            "#,
        )
        .bind(plugin_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn log_plugin_event(
        &self,
        plugin_id: Uuid,
        level: &str,
        message: &str,
        metadata: Value,
    ) -> Result<PluginLog, sqlx::Error> {
        let row = sqlx::query_as::<_, PluginLog>(
            r#"
            INSERT INTO plugin_logs_v1 (plugin_id, level, message, metadata)
            VALUES ($1, $2, $3, $4)
            RETURNING id, plugin_id, level, message, metadata, created_at
            "#,
        )
        .bind(plugin_id)
        .bind(level)
        .bind(message)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn get_plugin_logs(
        &self,
        plugin_id: Uuid,
        limit: i64,
    ) -> Result<Vec<PluginLog>, sqlx::Error> {
        let rows = sqlx::query_as::<_, PluginLog>(
            r#"
            SELECT id, plugin_id, level, message, metadata, created_at
            FROM plugin_logs_v1
            WHERE plugin_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(plugin_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
}
