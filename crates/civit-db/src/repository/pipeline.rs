#![forbid(unsafe_code)]
#![allow(clippy::too_many_arguments)]

use crate::error::{DbError, Result};
use crate::models::{
    MultiProjectPipeline, MultiProjectPipelineRun, Pipeline, PipelineAnalytics, PipelineTemplate,
};
use chrono::{DateTime, Utc};
use sqlx::postgres::PgPool;
use serde_json;
use uuid::Uuid;

impl super::DbRepository {
    // --- Pipelines ---

    pub async fn create_pipeline(
        &self,
        repo_id: Uuid,
        commit_sha: &str,
        trigger: &str,
    ) -> Result<Pipeline> {
        let row = sqlx::query_as::<_, Pipeline>(
            r#"INSERT INTO pipelines (repo_id, commit_sha, trigger)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(commit_sha)
        .bind(trigger)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_pipeline: {e}")))?;
        Ok(row)
    }

    pub async fn get_pipeline(&self, id: Uuid) -> Result<Pipeline> {
        sqlx::query_as::<_, Pipeline>("SELECT * FROM pipelines WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_pipeline: {e}")))
    }

    pub async fn list_pipelines(
        &self,
        repo_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Pipeline>> {
        let rows = sqlx::query_as::<_, Pipeline>(
            "SELECT * FROM pipelines WHERE repo_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(repo_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_pipelines: {e}")))?;
        Ok(rows)
    }

    pub async fn update_pipeline(&self, id: Uuid, status: Option<&str>) -> Result<Pipeline> {
        let row = sqlx::query_as::<_, Pipeline>(
            r#"UPDATE pipelines
               SET status     = COALESCE($2, status),
                   updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_pipeline: {e}")))?;
        Ok(row)
    }

    // --- Pipeline Templates ---

    pub async fn create_pipeline_template(
        &self,
        name: &str,
        description: &str,
        yaml_content: &str,
        category: &str,
        is_public: bool,
        author_id: Option<Uuid>,
    ) -> Result<PipelineTemplate> {
        let row = sqlx::query_as::<_, PipelineTemplate>(
            r#"INSERT INTO pipeline_templates (name, description, yaml_content, category, is_public, author_id)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(yaml_content)
        .bind(category)
        .bind(is_public)
        .bind(author_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_pipeline_template: {e}")))?;
        Ok(row)
    }

    pub async fn get_pipeline_template(&self, id: Uuid) -> Result<PipelineTemplate> {
        sqlx::query_as::<_, PipelineTemplate>("SELECT * FROM pipeline_templates WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_pipeline_template: {e}")))
    }

    pub async fn list_pipeline_templates(
        &self,
        category: Option<&str>,
        public_only: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PipelineTemplate>> {
        let rows = if let Some(cat) = category {
            sqlx::query_as::<_, PipelineTemplate>(
                r#"SELECT * FROM pipeline_templates
                   WHERE category = $1 AND ($2 = false OR is_public = true)
                   ORDER BY usage_count DESC, created_at DESC
                   LIMIT $3 OFFSET $4"#,
            )
            .bind(cat)
            .bind(public_only)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, PipelineTemplate>(
                r#"SELECT * FROM pipeline_templates
                   WHERE ($1 = false OR is_public = true)
                   ORDER BY usage_count DESC, created_at DESC
                   LIMIT $2 OFFSET $3"#,
            )
            .bind(public_only)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_pipeline_templates: {e}")))
    }

    pub async fn search_pipeline_templates(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<PipelineTemplate>> {
        let rows = sqlx::query_as::<_, PipelineTemplate>(
            r#"SELECT * FROM pipeline_templates
               WHERE is_public = true
                 AND (name ILIKE '%' || $1 || '%'
                      OR description ILIKE '%' || $1 || '%')
               ORDER BY usage_count DESC
               LIMIT $2"#,
        )
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("search_pipeline_templates: {e}")))?;
        Ok(rows)
    }

    pub async fn update_pipeline_template(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        yaml_content: Option<&str>,
        category: Option<&str>,
        is_public: Option<bool>,
    ) -> Result<PipelineTemplate> {
        let row = sqlx::query_as::<_, PipelineTemplate>(
            r#"UPDATE pipeline_templates
               SET name         = COALESCE($2, name),
                   description  = COALESCE($3, description),
                   yaml_content = COALESCE($4, yaml_content),
                   category     = COALESCE($5, category),
                   is_public    = COALESCE($6, is_public)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(yaml_content)
        .bind(category)
        .bind(is_public)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_pipeline_template: {e}")))?;
        Ok(row)
    }

    pub async fn delete_pipeline_template(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM pipeline_templates WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_pipeline_template: {e}")))?;
        Ok(())
    }

    pub async fn increment_template_usage(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE pipeline_templates SET usage_count = usage_count + 1 WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("increment_template_usage: {e}")))?;
        Ok(())
    }

    pub async fn create_template_from_pipeline(
        &self,
        name: &str,
        description: &str,
        yaml_content: &str,
        category: &str,
        is_public: bool,
        author_id: Option<Uuid>,
    ) -> Result<PipelineTemplate> {
        self.create_pipeline_template(name, description, yaml_content, category, is_public, author_id)
            .await
    }

    // --- Pipeline Analytics ---

    pub async fn create_pipeline_analytics(
        &self,
        repo_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        total_runs: i32,
        successful_runs: i32,
        failed_runs: i32,
        avg_duration_ms: i32,
        total_duration_ms: i64,
    ) -> Result<PipelineAnalytics> {
        let row = sqlx::query_as::<_, PipelineAnalytics>(
            r#"INSERT INTO pipeline_analytics (repo_id, period_start, period_end, total_runs, successful_runs, failed_runs, avg_duration_ms, total_duration_ms)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(period_start)
        .bind(period_end)
        .bind(total_runs)
        .bind(successful_runs)
        .bind(failed_runs)
        .bind(avg_duration_ms)
        .bind(total_duration_ms)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_pipeline_analytics: {e}")))?;
        Ok(row)
    }

    pub async fn get_pipeline_analytics(
        &self,
        repo_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<Vec<PipelineAnalytics>> {
        let rows = sqlx::query_as::<_, PipelineAnalytics>(
            r#"SELECT * FROM pipeline_analytics
               WHERE repo_id = $1
                 AND period_start >= $2
                 AND period_end <= $3
               ORDER BY period_start DESC"#,
        )
        .bind(repo_id)
        .bind(period_start)
        .bind(period_end)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_pipeline_analytics: {e}")))?;
        Ok(rows)
    }

    pub async fn get_pipeline_run_statistics(
        &self,
        repo_id: Uuid,
    ) -> Result<(i64, i64, i64, f64)> {
        let row: (i64, i64, i64, Option<f64>) = sqlx::query_as(
            r#"SELECT
                   COUNT(*) as total_runs,
                   COUNT(*) FILTER (WHERE status = 'success') as successful_runs,
                   COUNT(*) FILTER (WHERE status = 'failure') as failed_runs,
                   AVG(EXTRACT(EPOCH FROM (finished_at - started_at)) * 1000)::FLOAT as avg_duration_ms
               FROM pipeline_runs
               WHERE repo_id = $1
                 AND finished_at IS NOT NULL"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_pipeline_run_statistics: {e}")))?;
        Ok((row.0, row.1, row.2, row.3.unwrap_or(0.0)))
    }

    pub async fn get_pipeline_success_failure_rates(
        &self,
        repo_id: Uuid,
    ) -> Result<(f64, f64)> {
        let row: (Option<f64>, Option<f64>) = sqlx::query_as(
            r#"SELECT
                   (COUNT(*) FILTER (WHERE status = 'success')::FLOAT / NULLIF(COUNT(*), 0) * 100) as success_rate,
                   (COUNT(*) FILTER (WHERE status = 'failure')::FLOAT / NULLIF(COUNT(*), 0) * 100) as failure_rate
               FROM pipeline_runs
               WHERE repo_id = $1"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_pipeline_success_failure_rates: {e}")))?;
        Ok((row.0.unwrap_or(0.0), row.1.unwrap_or(0.0)))
    }

    pub async fn estimate_pipeline_cost(
        &self,
        repo_id: Uuid,
        cost_per_minute_ms: f64,
    ) -> Result<f64> {
        let row: (Option<f64>,) = sqlx::query_as(
            r#"SELECT
                   SUM(EXTRACT(EPOCH FROM (finished_at - started_at)) * 1000) * $2 as estimated_cost
               FROM pipeline_runs
               WHERE repo_id = $1
                 AND finished_at IS NOT NULL"#,
        )
        .bind(repo_id)
        .bind(cost_per_minute_ms / 60000.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("estimate_pipeline_cost: {e}")))?;
        Ok(row.0.unwrap_or(0.0))
    }

    // --- Multi-project Pipelines ---

    pub async fn create_multi_project_pipeline(
        &self,
        name: &str,
        description: &str,
        project_ids: &[Uuid],
        trigger_rules: &serde_json::Value,
        enabled: bool,
    ) -> Result<MultiProjectPipeline> {
        let row = sqlx::query_as::<_, MultiProjectPipeline>(
            r#"INSERT INTO multi_project_pipelines (name, description, project_ids, trigger_rules, enabled)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(project_ids)
        .bind(trigger_rules)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_multi_project_pipeline: {e}")))?;
        Ok(row)
    }

    pub async fn get_multi_project_pipeline(
        &self,
        id: Uuid,
    ) -> Result<MultiProjectPipeline> {
        sqlx::query_as::<_, MultiProjectPipeline>(
            "SELECT * FROM multi_project_pipelines WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_multi_project_pipeline: {e}")))
    }

    pub async fn list_multi_project_pipelines(
        &self,
        enabled_only: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<MultiProjectPipeline>> {
        let rows = sqlx::query_as::<_, MultiProjectPipeline>(
            r#"SELECT * FROM multi_project_pipelines
               WHERE ($1 = false OR enabled = true)
               ORDER BY created_at DESC
               LIMIT $2 OFFSET $3"#,
        )
        .bind(enabled_only)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_multi_project_pipelines: {e}")))?;
        Ok(rows)
    }

    pub async fn update_multi_project_pipeline(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        project_ids: Option<&[Uuid]>,
        trigger_rules: Option<&serde_json::Value>,
        enabled: Option<bool>,
    ) -> Result<MultiProjectPipeline> {
        let row = sqlx::query_as::<_, MultiProjectPipeline>(
            r#"UPDATE multi_project_pipelines
               SET name          = COALESCE($2, name),
                   description   = COALESCE($3, description),
                   project_ids   = COALESCE($4, project_ids),
                   trigger_rules = COALESCE($5, trigger_rules),
                   enabled       = COALESCE($6, enabled)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(project_ids)
        .bind(trigger_rules)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_multi_project_pipeline: {e}")))?;
        Ok(row)
    }

    pub async fn delete_multi_project_pipeline(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM multi_project_pipelines WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_multi_project_pipeline: {e}")))?;
        Ok(())
    }

    pub async fn create_multi_project_pipeline_run(
        &self,
        pipeline_id: Uuid,
    ) -> Result<MultiProjectPipelineRun> {
        let row = sqlx::query_as::<_, MultiProjectPipelineRun>(
            r#"INSERT INTO multi_project_pipeline_runs (pipeline_id)
               VALUES ($1)
               RETURNING *"#,
        )
        .bind(pipeline_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_multi_project_pipeline_run: {e}")))?;
        Ok(row)
    }

    pub async fn update_multi_project_pipeline_run(
        &self,
        id: Uuid,
        status: Option<&str>,
    ) -> Result<MultiProjectPipelineRun> {
        let row = sqlx::query_as::<_, MultiProjectPipelineRun>(
            r#"UPDATE multi_project_pipeline_runs
               SET status = COALESCE($2, status),
                   completed_at = CASE WHEN $2 IN ('success', 'failure', 'canceled') THEN NOW() ELSE completed_at END
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_multi_project_pipeline_run: {e}")))?;
        Ok(row)
    }

    pub async fn list_multi_project_pipeline_runs(
        &self,
        pipeline_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<MultiProjectPipelineRun>> {
        let rows = sqlx::query_as::<_, MultiProjectPipelineRun>(
            r#"SELECT * FROM multi_project_pipeline_runs
               WHERE pipeline_id = $1
               ORDER BY started_at DESC
               LIMIT $2 OFFSET $3"#,
        )
        .bind(pipeline_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_multi_project_pipeline_runs: {e}")))?;
        Ok(rows)
    }


}
