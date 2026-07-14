use super::types::*;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct InfrastructureStore {
    pool: PgPool,
}

impl InfrastructureStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_template(&self, req: CreateTemplateRequest) -> Result<InfrastructureTemplate, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let description = req.description.unwrap_or_default();
        let variables = req.variables.unwrap_or(serde_json::json!({}));

        sqlx::query(
            r#"INSERT INTO infrastructure_templates (id, name, description, provider, template_content, variables, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        )
        .bind(id)
        .bind(&req.name)
        .bind(&description)
        .bind(&req.provider)
        .bind(&req.template_content)
        .bind(&variables)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(InfrastructureTemplate {
            id,
            name: req.name,
            description,
            provider: req.provider,
            template_content: req.template_content,
            variables,
            created_at: now,
        })
    }

    pub async fn get_template(&self, id: Uuid) -> Result<Option<InfrastructureTemplate>, sqlx::Error> {
        let row = sqlx::query_as::<_, TemplateRow>(
            r#"SELECT id, name, description, provider, template_content, variables, created_at
               FROM infrastructure_templates WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(InfrastructureTemplate::from))
    }

    pub async fn list_templates(&self, limit: i64, offset: i64) -> Result<Vec<InfrastructureTemplate>, sqlx::Error> {
        let rows = sqlx::query_as::<_, TemplateRow>(
            r#"SELECT id, name, description, provider, template_content, variables, created_at
               FROM infrastructure_templates ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(InfrastructureTemplate::from).collect())
    }

    pub async fn update_template(&self, id: Uuid, req: UpdateTemplateRequest) -> Result<InfrastructureTemplate, sqlx::Error> {
        let mut template = self.get_template(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;

        if let Some(name) = req.name {
            sqlx::query(r#"UPDATE infrastructure_templates SET name = $1 WHERE id = $2"#)
                .bind(&name).bind(id).execute(&self.pool).await?;
            template.name = name;
        }
        if let Some(description) = req.description {
            sqlx::query(r#"UPDATE infrastructure_templates SET description = $1 WHERE id = $2"#)
                .bind(&description).bind(id).execute(&self.pool).await?;
            template.description = description;
        }
        if let Some(provider) = req.provider {
            sqlx::query(r#"UPDATE infrastructure_templates SET provider = $1 WHERE id = $2"#)
                .bind(&provider).bind(id).execute(&self.pool).await?;
            template.provider = provider;
        }
        if let Some(template_content) = req.template_content {
            sqlx::query(r#"UPDATE infrastructure_templates SET template_content = $1 WHERE id = $2"#)
                .bind(&template_content).bind(id).execute(&self.pool).await?;
            template.template_content = template_content;
        }
        if let Some(variables) = req.variables {
            sqlx::query(r#"UPDATE infrastructure_templates SET variables = $1 WHERE id = $2"#)
                .bind(&variables).bind(id).execute(&self.pool).await?;
            template.variables = variables;
        }

        Ok(template)
    }

    pub async fn delete_template(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM infrastructure_templates WHERE id = $1"#)
            .bind(id).execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn deploy(&self, template_id: Uuid, req: DeployRequest) -> Result<InfrastructureDeployment, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let variables = req.variables.unwrap_or(serde_json::json!({}));

        sqlx::query(
            r#"INSERT INTO infrastructure_deployments (id, template_id, environment, status, variables, started_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(id)
        .bind(template_id)
        .bind(&req.environment)
        .bind(InfraDeploymentStatus::Pending.to_string())
        .bind(&variables)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(InfrastructureDeployment {
            id,
            template_id,
            environment: req.environment,
            status: InfraDeploymentStatus::Pending,
            variables,
            started_at: now,
            completed_at: None,
        })
    }

    pub async fn get_deployment(&self, id: Uuid) -> Result<Option<InfrastructureDeployment>, sqlx::Error> {
        let row = sqlx::query_as::<_, DeploymentRow>(
            r#"SELECT id, template_id, environment, status, variables, started_at, completed_at
               FROM infrastructure_deployments WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(InfrastructureDeployment::from))
    }

    pub async fn list_deployments(&self, template_id: Uuid, limit: i64, offset: i64) -> Result<Vec<InfrastructureDeployment>, sqlx::Error> {
        let rows = sqlx::query_as::<_, DeploymentRow>(
            r#"SELECT id, template_id, environment, status, variables, started_at, completed_at
               FROM infrastructure_deployments WHERE template_id = $1 ORDER BY started_at DESC LIMIT $2 OFFSET $3"#,
        )
        .bind(template_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(InfrastructureDeployment::from).collect())
    }

    pub async fn complete_deployment(&self, id: Uuid, success: bool) -> Result<InfrastructureDeployment, sqlx::Error> {
        let now = Utc::now();
        let status = if success { InfraDeploymentStatus::Completed } else { InfraDeploymentStatus::Failed };

        sqlx::query(
            r#"UPDATE infrastructure_deployments SET status = $1, completed_at = $2 WHERE id = $3"#,
        )
        .bind(status.to_string())
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_deployment(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)
    }
}

#[derive(sqlx::FromRow)]
struct TemplateRow {
    id: Uuid,
    name: String,
    description: String,
    provider: String,
    template_content: String,
    variables: serde_json::Value,
    created_at: chrono::DateTime<Utc>,
}

impl From<TemplateRow> for InfrastructureTemplate {
    fn from(row: TemplateRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            provider: row.provider,
            template_content: row.template_content,
            variables: row.variables,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct DeploymentRow {
    id: Uuid,
    template_id: Uuid,
    environment: String,
    status: String,
    variables: serde_json::Value,
    started_at: chrono::DateTime<Utc>,
    completed_at: Option<chrono::DateTime<Utc>>,
}

impl From<DeploymentRow> for InfrastructureDeployment {
    fn from(row: DeploymentRow) -> Self {
        Self {
            id: row.id,
            template_id: row.template_id,
            environment: row.environment,
            status: row.status.parse().unwrap_or(InfraDeploymentStatus::Pending),
            variables: row.variables,
            started_at: row.started_at,
            completed_at: row.completed_at,
        }
    }
}
