#![forbid(unsafe_code)]

//! Access control lists for CivitForge.
//!
//! Provides fine-grained access control with permission management,
//! inheritance rules, and audit logging.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlEntry {
    pub id: Uuid,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub principal_type: String,
    pub principal_id: Uuid,
    pub permission: String,
    pub granted_by: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAclEntry {
    pub resource_type: String,
    pub resource_id: Uuid,
    pub principal_type: String,
    pub principal_id: Uuid,
    pub permission: String,
    pub granted_by: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCheck {
    pub resource_type: String,
    pub resource_id: Uuid,
    pub principal_type: String,
    pub principal_id: Uuid,
    pub permission: String,
}

#[derive(Debug, sqlx::FromRow)]
struct AclRow {
    id: Uuid,
    resource_type: String,
    resource_id: Uuid,
    principal_type: String,
    principal_id: Uuid,
    permission: String,
    granted_by: Option<Uuid>,
    expires_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<AclRow> for AccessControlEntry {
    fn from(row: AclRow) -> Self {
        AccessControlEntry {
            id: row.id,
            resource_type: row.resource_type,
            resource_id: row.resource_id,
            principal_type: row.principal_type,
            principal_id: row.principal_id,
            permission: row.permission,
            granted_by: row.granted_by,
            expires_at: row.expires_at,
            created_at: row.created_at,
        }
    }
}

pub struct AclService {
    pool: PgPool,
}

impl AclService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn grant(&self, input: CreateAclEntry) -> Result<AccessControlEntry, sqlx::Error> {
        let row = sqlx::query_as::<_, AclRow>(
            r#"INSERT INTO access_control_lists
             (resource_type, resource_id, principal_type, principal_id, permission, granted_by, expires_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (resource_type, resource_id, principal_type, principal_id, permission)
             DO UPDATE SET granted_by = $6, expires_at = $7
             RETURNING id, resource_type, resource_id, principal_type, principal_id,
                       permission, granted_by, expires_at, created_at"#,
        )
        .bind(&input.resource_type)
        .bind(input.resource_id)
        .bind(&input.principal_type)
        .bind(input.principal_id)
        .bind(&input.permission)
        .bind(input.granted_by)
        .bind(input.expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    pub async fn revoke(
        &self,
        resource_type: &str,
        resource_id: Uuid,
        principal_type: &str,
        principal_id: Uuid,
        permission: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"DELETE FROM access_control_lists
             WHERE resource_type = $1 AND resource_id = $2
             AND principal_type = $3 AND principal_id = $4 AND permission = $5"#,
        )
        .bind(resource_type)
        .bind(resource_id)
        .bind(principal_type)
        .bind(principal_id)
        .bind(permission)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn check_permission(&self, check: &PermissionCheck) -> Result<bool, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct CountRow {
            count: Option<i64>,
        }

        let row = sqlx::query_as::<_, CountRow>(
            r#"SELECT COUNT(*) as count FROM access_control_lists
             WHERE resource_type = $1 AND resource_id = $2
             AND principal_type = $3 AND principal_id = $4
             AND permission = $5
             AND (expires_at IS NULL OR expires_at > NOW())"#,
        )
        .bind(&check.resource_type)
        .bind(check.resource_id)
        .bind(&check.principal_type)
        .bind(check.principal_id)
        .bind(&check.permission)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.count.unwrap_or(0) > 0)
    }

    pub async fn list_for_resource(
        &self,
        resource_type: &str,
        resource_id: Uuid,
    ) -> Result<Vec<AccessControlEntry>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AclRow>(
            r#"SELECT id, resource_type, resource_id, principal_type, principal_id,
               permission, granted_by, expires_at, created_at
             FROM access_control_lists
             WHERE resource_type = $1 AND resource_id = $2
             ORDER BY created_at DESC"#,
        )
        .bind(resource_type)
        .bind(resource_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_for_principal(
        &self,
        principal_type: &str,
        principal_id: Uuid,
    ) -> Result<Vec<AccessControlEntry>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AclRow>(
            r#"SELECT id, resource_type, resource_id, principal_type, principal_id,
               permission, granted_by, expires_at, created_at
             FROM access_control_lists
             WHERE principal_type = $1 AND principal_id = $2
             AND (expires_at IS NULL OR expires_at > NOW())
             ORDER BY created_at DESC"#,
        )
        .bind(principal_type)
        .bind(principal_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn check_inherited(
        &self,
        check: &PermissionCheck,
    ) -> Result<bool, sqlx::Error> {
        if self.check_permission(check).await? {
            return Ok(true);
        }

        let resource_ids = self.get_parent_resources(&check.resource_type, check.resource_id).await?;

        for (parent_type, parent_id) in resource_ids {
            let parent_check = PermissionCheck {
                resource_type: parent_type,
                resource_id: parent_id,
                principal_type: check.principal_type.clone(),
                principal_id: check.principal_id,
                permission: check.permission.clone(),
            };
            if self.check_permission(&parent_check).await? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn get_parent_resources(
        &self,
        resource_type: &str,
        resource_id: Uuid,
    ) -> Result<Vec<(String, Uuid)>, sqlx::Error> {
        let mut parents = Vec::new();

        match resource_type {
            "pull_request" => {
                #[derive(sqlx::FromRow)]
                struct RepoRow {
                    repo_id: Uuid,
                }
                let row = sqlx::query_as::<_, RepoRow>(
                    r#"SELECT repo_id FROM pull_requests WHERE id = $1"#,
                )
                .bind(resource_id)
                .fetch_optional(&self.pool)
                .await?;
                if let Some(r) = row {
                    parents.push(("repository".into(), r.repo_id));
                }
            }
            "issue" => {
                #[derive(sqlx::FromRow)]
                struct RepoRow {
                    repo_id: Uuid,
                }
                let row = sqlx::query_as::<_, RepoRow>(
                    r#"SELECT repo_id FROM issues WHERE id = $1"#,
                )
                .bind(resource_id)
                .fetch_optional(&self.pool)
                .await?;
                if let Some(r) = row {
                    parents.push(("repository".into(), r.repo_id));
                }
            }
            "pipeline" => {
                #[derive(sqlx::FromRow)]
                struct RepoRow {
                    repo_id: Uuid,
                }
                let row = sqlx::query_as::<_, RepoRow>(
                    r#"SELECT repo_id FROM pipeline_definitions WHERE id = $1"#,
                )
                .bind(resource_id)
                .fetch_optional(&self.pool)
                .await?;
                if let Some(r) = row {
                    parents.push(("repository".into(), r.repo_id));
                }
            }
            _ => {}
        }

        Ok(parents)
    }

    pub async fn cleanup_expired(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"DELETE FROM access_control_lists WHERE expires_at IS NOT NULL AND expires_at < NOW()"#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn audit_log(
        &self,
        resource_type: &str,
        resource_id: Uuid,
    ) -> Result<Vec<AccessControlEntry>, sqlx::Error> {
        self.list_for_resource(resource_type, resource_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_check_construction() {
        let check = PermissionCheck {
            resource_type: "repository".into(),
            resource_id: Uuid::new_v4(),
            principal_type: "user".into(),
            principal_id: Uuid::new_v4(),
            permission: "read".into(),
        };
        assert_eq!(check.resource_type, "repository");
        assert_eq!(check.permission, "read");
    }

    #[test]
    fn test_acl_entry_serialization() {
        let entry = AccessControlEntry {
            id: Uuid::new_v4(),
            resource_type: "pull_request".into(),
            resource_id: Uuid::new_v4(),
            principal_type: "user".into(),
            principal_id: Uuid::new_v4(),
            permission: "merge".into(),
            granted_by: None,
            expires_at: None,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("pull_request"));
        assert!(json.contains("merge"));
    }
}
