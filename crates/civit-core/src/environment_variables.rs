//! Environment Variables v3: Advanced environment variable management with
//! secret support, inheritance, and validation.

#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentVariable {
    pub id: Uuid,
    pub environment_id: Uuid,
    pub name: String,
    pub value: String,
    pub encrypted: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentVariableInheritance {
    pub id: Uuid,
    pub child_env_id: Uuid,
    pub parent_env_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVariableRequest {
    pub name: String,
    pub value: String,
    pub encrypted: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVariableRequest {
    pub value: Option<String>,
    pub encrypted: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddInheritanceRequest {
    pub parent_env_id: Uuid,
}

#[derive(Debug, sqlx::FromRow)]
struct VariableRow {
    id: Uuid,
    environment_id: Uuid,
    name: String,
    value: String,
    encrypted: bool,
    created_at: DateTime<Utc>,
}

impl From<VariableRow> for EnvironmentVariable {
    fn from(row: VariableRow) -> Self {
        EnvironmentVariable {
            id: row.id,
            environment_id: row.environment_id,
            name: row.name,
            value: row.value,
            encrypted: row.encrypted,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct InheritanceRow {
    id: Uuid,
    child_env_id: Uuid,
    parent_env_id: Uuid,
    created_at: DateTime<Utc>,
}

impl From<InheritanceRow> for EnvironmentVariableInheritance {
    fn from(row: InheritanceRow) -> Self {
        EnvironmentVariableInheritance {
            id: row.id,
            child_env_id: row.child_env_id,
            parent_env_id: row.parent_env_id,
            created_at: row.created_at,
        }
    }
}

pub struct EnvironmentVariablesService {
    pool: PgPool,
}

impl EnvironmentVariablesService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_variables(
        &self,
        environment_id: Uuid,
        include_inherited: bool,
    ) -> Result<Vec<EnvironmentVariable>, sqlx::Error> {
        let mut variables: Vec<EnvironmentVariable> = Vec::new();

        if include_inherited {
            // Get parent environments recursively
            let parent_ids = self.get_parent_environment_ids(environment_id).await?;

            // Get variables from parent environments (in order from root to child)
            for parent_id in parent_ids.iter().rev() {
                let parent_vars = self.get_direct_variables(*parent_id).await?;
                for var in parent_vars {
                    // Remove existing variable with same name (child overrides parent)
                    variables.retain(|v| v.name != var.name);
                    variables.push(var);
                }
            }
        }

        // Get direct variables for this environment (highest priority)
        let direct_vars = self.get_direct_variables(environment_id).await?;
        for var in direct_vars {
            variables.retain(|v| v.name != var.name);
            variables.push(var);
        }

        variables.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(variables)
    }

    async fn get_direct_variables(
        &self,
        environment_id: Uuid,
    ) -> Result<Vec<EnvironmentVariable>, sqlx::Error> {
        let rows = sqlx::query_as::<_, VariableRow>(
            "SELECT id, environment_id, name, value, encrypted, created_at
             FROM environment_variables
             WHERE environment_id = $1
             ORDER BY name",
        )
        .bind(environment_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn get_parent_environment_ids(
        &self,
        environment_id: Uuid,
    ) -> Result<Vec<Uuid>, sqlx::Error> {
        let mut parent_ids = Vec::new();
        let mut current_id = environment_id;
        let mut visited = std::collections::HashSet::new();

        loop {
            if visited.contains(&current_id) {
                break; // Prevent infinite loops
            }
            visited.insert(current_id);

            let row: Option<(Uuid,)> = sqlx::query_as(
                "SELECT parent_env_id FROM environment_variable_inheritance
                 WHERE child_env_id = $1
                 LIMIT 1",
            )
            .bind(current_id)
            .fetch_optional(&self.pool)
            .await?;

            match row {
                Some((parent_id,)) => {
                    parent_ids.push(parent_id);
                    current_id = parent_id;
                }
                None => break,
            }
        }

        Ok(parent_ids)
    }

    pub async fn get_variable(
        &self,
        environment_id: Uuid,
        name: &str,
    ) -> Result<Option<EnvironmentVariable>, sqlx::Error> {
        let row = sqlx::query_as::<_, VariableRow>(
            "SELECT id, environment_id, name, value, encrypted, created_at
             FROM environment_variables
             WHERE environment_id = $1 AND name = $2",
        )
        .bind(environment_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn create_variable(
        &self,
        environment_id: Uuid,
        request: CreateVariableRequest,
    ) -> Result<EnvironmentVariable, sqlx::Error> {
        let encrypted = request.encrypted.unwrap_or(false);

        let row = sqlx::query_as::<_, VariableRow>(
            "INSERT INTO environment_variables (environment_id, name, value, encrypted, created_at)
             VALUES ($1, $2, $3, $4, NOW())
             RETURNING id, environment_id, name, value, encrypted, created_at",
        )
        .bind(environment_id)
        .bind(&request.name)
        .bind(&request.value)
        .bind(encrypted)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn update_variable(
        &self,
        environment_id: Uuid,
        name: &str,
        request: UpdateVariableRequest,
    ) -> Result<EnvironmentVariable, sqlx::Error> {
        let row = sqlx::query_as::<_, VariableRow>(
            "UPDATE environment_variables
             SET value = COALESCE($3, value),
                 encrypted = COALESCE($4, encrypted)
             WHERE environment_id = $1 AND name = $2
             RETURNING id, environment_id, name, value, encrypted, created_at",
        )
        .bind(environment_id)
        .bind(name)
        .bind(request.value)
        .bind(request.encrypted)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn delete_variable(
        &self,
        environment_id: Uuid,
        name: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM environment_variables WHERE environment_id = $1 AND name = $2",
        )
        .bind(environment_id)
        .bind(name)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn add_inheritance(
        &self,
        child_env_id: Uuid,
        parent_env_id: Uuid,
    ) -> Result<EnvironmentVariableInheritance, sqlx::Error> {
        let row = sqlx::query_as::<_, InheritanceRow>(
            "INSERT INTO environment_variable_inheritance (child_env_id, parent_env_id, created_at)
             VALUES ($1, $2, NOW())
             RETURNING id, child_env_id, parent_env_id, created_at",
        )
        .bind(child_env_id)
        .bind(parent_env_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn remove_inheritance(
        &self,
        child_env_id: Uuid,
        parent_env_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM environment_variable_inheritance
             WHERE child_env_id = $1 AND parent_env_id = $2",
        )
        .bind(child_env_id)
        .bind(parent_env_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn list_inheritances(
        &self,
        environment_id: Uuid,
    ) -> Result<Vec<EnvironmentVariableInheritance>, sqlx::Error> {
        let rows = sqlx::query_as::<_, InheritanceRow>(
            "SELECT id, child_env_id, parent_env_id, created_at
             FROM environment_variable_inheritance
             WHERE child_env_id = $1
             ORDER BY created_at",
        )
        .bind(environment_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn validate_variable_name(name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("Variable name cannot be empty".to_string());
        }

        if name.len() > 256 {
            return Err("Variable name too long (max 256 characters)".to_string());
        }

        // Check for valid identifier: letters, digits, underscores
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
        {
            return Err(
                "Variable name can only contain letters, digits, and underscores".to_string(),
            );
        }

        // Must start with letter or underscore
        if let Some(first) = name.chars().next()
            && !first.is_alphabetic() && first != '_' {
                return Err("Variable name must start with a letter or underscore".to_string());
            }

        Ok(())
    }

    pub async fn validate_variable_value(value: &str) -> Result<(), String> {
        if value.len() > 1024 * 1024 {
            // 1MB limit
            return Err("Variable value too long (max 1MB)".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_variable_name_valid() {
        assert!(EnvironmentVariablesService::validate_variable_name("MY_VAR").await.is_ok());
        assert!(EnvironmentVariablesService::validate_variable_name("_private").await.is_ok());
        assert!(EnvironmentVariablesService::validate_variable_name("var123").await.is_ok());
    }

    #[tokio::test]
    async fn test_validate_variable_name_invalid() {
        assert!(EnvironmentVariablesService::validate_variable_name("").await.is_err());
        assert!(EnvironmentVariablesService::validate_variable_name("123VAR").await.is_err());
        assert!(EnvironmentVariablesService::validate_variable_name("my-var").await.is_err());
        assert!(EnvironmentVariablesService::validate_variable_name("my var").await.is_err());
    }

    #[tokio::test]
    async fn test_validate_variable_value() {
        assert!(EnvironmentVariablesService::validate_variable_value("hello").await.is_ok());
        assert!(EnvironmentVariablesService::validate_variable_value("").await.is_ok());

        let long_value = "x".repeat(1024 * 1024 + 1);
        assert!(EnvironmentVariablesService::validate_variable_value(&long_value).await.is_err());
    }

    #[test]
    fn test_environment_variable_serialize() {
        let var = EnvironmentVariable {
            id: Uuid::new_v4(),
            environment_id: Uuid::new_v4(),
            name: "DATABASE_URL".to_string(),
            value: "postgres://localhost/mydb".to_string(),
            encrypted: false,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&var).unwrap();
        assert!(json.contains("DATABASE_URL"));
        assert!(json.contains("postgres://localhost/mydb"));
    }

    #[test]
    fn test_create_variable_request_deserialize() {
        let json = r#"{"name": "API_KEY", "value": "secret123", "encrypted": true}"#;
        let req: CreateVariableRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "API_KEY");
        assert_eq!(req.value, "secret123");
        assert_eq!(req.encrypted, Some(true));
    }

    #[test]
    fn test_inheritance_serialize() {
        let inh = EnvironmentVariableInheritance {
            id: Uuid::new_v4(),
            child_env_id: Uuid::new_v4(),
            parent_env_id: Uuid::new_v4(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&inh).unwrap();
        assert!(json.contains("child_env_id"));
        assert!(json.contains("parent_env_id"));
    }
}
