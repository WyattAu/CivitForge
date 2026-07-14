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

    pub async fn create_module(&self, req: CreateModuleRequest) -> Result<InfrastructureModule, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let description = req.description.unwrap_or_default();
        let config = req.config.unwrap_or(serde_json::json!({}));
        let version = req.version.unwrap_or_else(|| "1.0.0".to_string());

        sqlx::query(
            r#"INSERT INTO infrastructure_modules (id, name, description, provider, module_type, config, version, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        )
        .bind(id)
        .bind(&req.name)
        .bind(&description)
        .bind(&req.provider)
        .bind(&req.module_type)
        .bind(&config)
        .bind(&version)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(InfrastructureModule {
            id,
            name: req.name,
            description,
            provider: req.provider,
            module_type: req.module_type,
            config,
            version,
            created_at: now,
        })
    }

    pub async fn get_module(&self, id: Uuid) -> Result<Option<InfrastructureModule>, sqlx::Error> {
        let row = sqlx::query_as::<_, ModuleRow>(
            r#"SELECT id, name, description, provider, module_type, config, version, created_at
               FROM infrastructure_modules WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(InfrastructureModule::from))
    }

    pub async fn list_modules(&self, provider: Option<&str>, module_type: Option<&str>, limit: i64, offset: i64) -> Result<Vec<InfrastructureModule>, sqlx::Error> {
        match (provider, module_type) {
            (Some(p), Some(t)) => {
                let rows = sqlx::query_as::<_, ModuleRow>(
                    r#"SELECT id, name, description, provider, module_type, config, version, created_at
                       FROM infrastructure_modules WHERE provider = $1 AND module_type = $2
                       ORDER BY created_at DESC LIMIT $3 OFFSET $4"#,
                )
                .bind(p)
                .bind(t)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?;
                Ok(rows.into_iter().map(InfrastructureModule::from).collect())
            }
            (Some(p), None) => {
                let rows = sqlx::query_as::<_, ModuleRow>(
                    r#"SELECT id, name, description, provider, module_type, config, version, created_at
                       FROM infrastructure_modules WHERE provider = $1
                       ORDER BY created_at DESC LIMIT $2 OFFSET $3"#,
                )
                .bind(p)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?;
                Ok(rows.into_iter().map(InfrastructureModule::from).collect())
            }
            (None, Some(t)) => {
                let rows = sqlx::query_as::<_, ModuleRow>(
                    r#"SELECT id, name, description, provider, module_type, config, version, created_at
                       FROM infrastructure_modules WHERE module_type = $1
                       ORDER BY created_at DESC LIMIT $2 OFFSET $3"#,
                )
                .bind(t)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?;
                Ok(rows.into_iter().map(InfrastructureModule::from).collect())
            }
            (None, None) => {
                let rows = sqlx::query_as::<_, ModuleRow>(
                    r#"SELECT id, name, description, provider, module_type, config, version, created_at
                       FROM infrastructure_modules
                       ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?;
                Ok(rows.into_iter().map(InfrastructureModule::from).collect())
            }
        }
    }

    pub async fn update_module(&self, id: Uuid, req: UpdateModuleRequest) -> Result<InfrastructureModule, sqlx::Error> {
        let mut module = self.get_module(id).await?.ok_or_else(|| sqlx::Error::RowNotFound)?;

        if let Some(name) = req.name {
            sqlx::query(r#"UPDATE infrastructure_modules SET name = $1 WHERE id = $2"#)
                .bind(&name).bind(id).execute(&self.pool).await?;
            module.name = name;
        }
        if let Some(description) = req.description {
            sqlx::query(r#"UPDATE infrastructure_modules SET description = $1 WHERE id = $2"#)
                .bind(&description).bind(id).execute(&self.pool).await?;
            module.description = description;
        }
        if let Some(provider) = req.provider {
            sqlx::query(r#"UPDATE infrastructure_modules SET provider = $1 WHERE id = $2"#)
                .bind(&provider).bind(id).execute(&self.pool).await?;
            module.provider = provider;
        }
        if let Some(module_type) = req.module_type {
            sqlx::query(r#"UPDATE infrastructure_modules SET module_type = $1 WHERE id = $2"#)
                .bind(&module_type).bind(id).execute(&self.pool).await?;
            module.module_type = module_type;
        }
        if let Some(config) = req.config {
            sqlx::query(r#"UPDATE infrastructure_modules SET config = $1 WHERE id = $2"#)
                .bind(&config).bind(id).execute(&self.pool).await?;
            module.config = config;
        }
        if let Some(version) = req.version {
            sqlx::query(r#"UPDATE infrastructure_modules SET version = $1 WHERE id = $2"#)
                .bind(&version).bind(id).execute(&self.pool).await?;
            module.version = version;
        }

        Ok(module)
    }

    pub async fn delete_module(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM infrastructure_modules WHERE id = $1"#)
            .bind(id).execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn create_module_dependency(&self, module_id: Uuid, req: CreateModuleDependencyRequest) -> Result<ModuleDependency, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let version_constraint = req.version_constraint.unwrap_or_else(|| "*".to_string());

        sqlx::query(
            r#"INSERT INTO infrastructure_module_deps (id, module_id, dependency_id, version_constraint, created_at)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(id)
        .bind(module_id)
        .bind(req.dependency_id)
        .bind(&version_constraint)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(ModuleDependency {
            id,
            module_id,
            dependency_id: req.dependency_id,
            version_constraint,
            created_at: now,
        })
    }

    pub async fn list_module_dependencies(&self, module_id: Uuid) -> Result<Vec<ModuleDependency>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ModuleDepRow>(
            r#"SELECT id, module_id, dependency_id, version_constraint, created_at
               FROM infrastructure_module_deps WHERE module_id = $1 ORDER BY created_at"#,
        )
        .bind(module_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(ModuleDependency::from).collect())
    }

    pub async fn delete_module_dependency(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(r#"DELETE FROM infrastructure_module_deps WHERE id = $1"#)
            .bind(id).execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn resolve_dependencies(&self, module_id: Uuid) -> Result<Vec<InfrastructureModule>, sqlx::Error> {
        let deps = self.list_module_dependencies(module_id).await?;
        let mut resolved = Vec::new();
        
        for dep in deps {
            if let Some(module) = self.get_module(dep.dependency_id).await? {
                if self.version_matches(&module.version, &dep.version_constraint) {
                    resolved.push(module);
                }
            }
        }
        
        Ok(resolved)
    }

    pub async fn get_module_versions(&self, name: &str) -> Result<Vec<ModuleVersion>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ModuleRow>(
            r#"SELECT id, name, description, provider, module_type, config, version, created_at
               FROM infrastructure_modules WHERE name = $1 ORDER BY created_at DESC"#,
        )
        .bind(name)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| ModuleVersion {
            version: r.version,
            created_at: r.created_at,
        }).collect())
    }

    pub async fn search_marketplace(&self, query: &str, provider: Option<&str>, limit: i64, offset: i64) -> Result<Vec<ModuleMarketplaceItem>, sqlx::Error> {
        let search_pattern = format!("%{}%", query);
        
        let rows = match provider {
            Some(p) => {
                sqlx::query_as::<_, ModuleRow>(
                    r#"SELECT id, name, description, provider, module_type, config, version, created_at
                       FROM infrastructure_modules WHERE (name ILIKE $1 OR description ILIKE $1) AND provider = $2
                       ORDER BY created_at DESC LIMIT $3 OFFSET $4"#,
                )
                .bind(&search_pattern)
                .bind(p)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, ModuleRow>(
                    r#"SELECT id, name, description, provider, module_type, config, version, created_at
                       FROM infrastructure_modules WHERE (name ILIKE $1 OR description ILIKE $1)
                       ORDER BY created_at DESC LIMIT $2 OFFSET $3"#,
                )
                .bind(&search_pattern)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
        };
        
        let modules: Vec<InfrastructureModule> = rows.into_iter().map(InfrastructureModule::from).collect();
        
        let mut items = Vec::new();
        for module in modules {
            let deps = self.list_module_dependencies(module.id).await.unwrap_or_default();
            items.push(ModuleMarketplaceItem {
                module,
                download_count: 0,
                rating: 0.0,
                dependencies: deps,
            });
        }
        
        Ok(items)
    }

    fn version_matches(&self, version: &str, constraint: &str) -> bool {
        if constraint == "*" {
            return true;
        }
        
        if constraint.starts_with(">=") {
            if let Some(min_version) = constraint.strip_prefix(">=") {
                return version >= min_version;
            }
        }
        
        if constraint.starts_with("^") {
            if let Some(min_version) = constraint.strip_prefix("^") {
                return version.starts_with(min_version);
            }
        }
        
        version == constraint
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

#[derive(sqlx::FromRow)]
struct ModuleRow {
    id: Uuid,
    name: String,
    description: String,
    provider: String,
    module_type: String,
    config: serde_json::Value,
    version: String,
    created_at: chrono::DateTime<Utc>,
}

impl From<ModuleRow> for InfrastructureModule {
    fn from(row: ModuleRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            provider: row.provider,
            module_type: row.module_type,
            config: row.config,
            version: row.version,
            created_at: row.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ModuleDepRow {
    id: Uuid,
    module_id: Uuid,
    dependency_id: Uuid,
    version_constraint: String,
    created_at: chrono::DateTime<Utc>,
}

impl From<ModuleDepRow> for ModuleDependency {
    fn from(row: ModuleDepRow) -> Self {
        Self {
            id: row.id,
            module_id: row.module_id,
            dependency_id: row.dependency_id,
            version_constraint: row.version_constraint,
            created_at: row.created_at,
        }
    }
}
