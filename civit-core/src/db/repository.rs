#![forbid(unsafe_code)]

use crate::db::models::{Issue, Org, Pipeline, PullRequest, Repository, User};
use crate::error::{CoreError, Result};
use chrono::{DateTime, Utc};
use sqlx::postgres::PgPool;
use uuid::Uuid;

pub struct DbRepository {
    pool: PgPool,
}

impl DbRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // --- Users ---

    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        display_name: &str,
        role: &str,
    ) -> Result<User> {
        let row = sqlx::query_as::<_, User>(
            r#"INSERT INTO users (username, email, display_name, role)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(username)
        .bind(email)
        .bind(display_name)
        .bind(role)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("create_user: {e}")))?;
        Ok(row)
    }

    pub async fn get_user_by_id(&self, id: Uuid) -> Result<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("get_user_by_id: {e}")))
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("get_user_by_username: {e}")))
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("get_user_by_email: {e}")))
    }

    pub async fn update_user(
        &self,
        id: Uuid,
        display_name: Option<&str>,
        bio: Option<&str>,
        role: Option<&str>,
    ) -> Result<User> {
        let row = sqlx::query_as::<_, User>(
            r#"UPDATE users
               SET display_name  = COALESCE($2, display_name),
                   bio           = COALESCE($3, bio),
                   role          = COALESCE($4, role),
                   updated_at    = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(display_name)
        .bind(bio)
        .bind(role)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("update_user: {e}")))?;
        Ok(row)
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("delete_user: {e}")))?;
        Ok(())
    }

    pub async fn list_users(&self, limit: i64, offset: i64) -> Result<Vec<User>> {
        let rows = sqlx::query_as::<_, User>(
            "SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("list_users: {e}")))?;
        Ok(rows)
    }

    // --- Organizations ---

    pub async fn create_org(
        &self,
        name: &str,
        display_name: &str,
        description: &str,
        visibility: &str,
        owner_id: Uuid,
    ) -> Result<Org> {
        let row = sqlx::query_as::<_, Org>(
            r#"INSERT INTO organizations (name, display_name, description, visibility, owner_id)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(name)
        .bind(display_name)
        .bind(description)
        .bind(visibility)
        .bind(owner_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("create_org: {e}")))?;
        Ok(row)
    }

    pub async fn get_org(&self, id: Uuid) -> Result<Org> {
        sqlx::query_as::<_, Org>("SELECT * FROM organizations WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("get_org: {e}")))
    }

    pub async fn list_orgs_by_owner(&self, owner_id: Uuid) -> Result<Vec<Org>> {
        let rows = sqlx::query_as::<_, Org>("SELECT * FROM organizations WHERE owner_id = $1")
            .bind(owner_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("list_orgs_by_owner: {e}")))?;
        Ok(rows)
    }

    pub async fn update_org(
        &self,
        id: Uuid,
        display_name: Option<&str>,
        description: Option<&str>,
        visibility: Option<&str>,
    ) -> Result<Org> {
        let row = sqlx::query_as::<_, Org>(
            r#"UPDATE organizations
               SET display_name = COALESCE($2, display_name),
                   description  = COALESCE($3, description),
                   visibility   = COALESCE($4, visibility),
                   updated_at   = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(display_name)
        .bind(description)
        .bind(visibility)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("update_org: {e}")))?;
        Ok(row)
    }

    // --- Repositories ---

    pub async fn create_repo(
        &self,
        name: &str,
        description: &str,
        owner_id: Uuid,
        org_id: Option<Uuid>,
        visibility: &str,
        default_branch: &str,
    ) -> Result<Repository> {
        let row = sqlx::query_as::<_, Repository>(
            r#"INSERT INTO repositories (name, description, owner_id, org_id, visibility, default_branch)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(owner_id)
        .bind(org_id)
        .bind(visibility)
        .bind(default_branch)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("create_repo: {e}")))?;
        Ok(row)
    }

    pub async fn get_repo(&self, id: Uuid) -> Result<Repository> {
        sqlx::query_as::<_, Repository>("SELECT * FROM repositories WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("get_repo: {e}")))
    }

    pub async fn get_repo_by_owner_name(&self, owner_id: Uuid, name: &str) -> Result<Repository> {
        sqlx::query_as::<_, Repository>(
            "SELECT * FROM repositories WHERE owner_id = $1 AND name = $2",
        )
        .bind(owner_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("get_repo_by_owner_name: {e}")))
    }

    pub async fn list_repos(&self, limit: i64, offset: i64) -> Result<Vec<Repository>> {
        let rows = sqlx::query_as::<_, Repository>(
            "SELECT * FROM repositories ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("list_repos: {e}")))?;
        Ok(rows)
    }

    pub async fn list_repos_by_org(&self, org_id: Uuid) -> Result<Vec<Repository>> {
        let rows = sqlx::query_as::<_, Repository>("SELECT * FROM repositories WHERE org_id = $1")
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("list_repos_by_org: {e}")))?;
        Ok(rows)
    }

    pub async fn update_repo(
        &self,
        id: Uuid,
        description: Option<&str>,
        visibility: Option<&str>,
        default_branch: Option<&str>,
    ) -> Result<Repository> {
        let row = sqlx::query_as::<_, Repository>(
            r#"UPDATE repositories
               SET description     = COALESCE($2, description),
                   visibility      = COALESCE($3, visibility),
                   default_branch  = COALESCE($4, default_branch),
                   updated_at      = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(description)
        .bind(visibility)
        .bind(default_branch)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("update_repo: {e}")))?;
        Ok(row)
    }

    pub async fn delete_repo(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM repositories WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("delete_repo: {e}")))?;
        Ok(())
    }

    // --- Issues ---

    pub async fn create_issue(
        &self,
        repo_id: Uuid,
        title: &str,
        body: &str,
        author_id: Uuid,
    ) -> Result<Issue> {
        let row = sqlx::query_as::<_, Issue>(
            r#"INSERT INTO issues (repo_id, title, body, author_id)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(title)
        .bind(body)
        .bind(author_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("create_issue: {e}")))?;
        Ok(row)
    }

    pub async fn get_issue(&self, id: Uuid) -> Result<Issue> {
        sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("get_issue: {e}")))
    }

    pub async fn list_issues(&self, repo_id: Uuid, limit: i64, offset: i64) -> Result<Vec<Issue>> {
        let rows = sqlx::query_as::<_, Issue>(
            "SELECT * FROM issues WHERE repo_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(repo_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("list_issues: {e}")))?;
        Ok(rows)
    }

    pub async fn update_issue(
        &self,
        id: Uuid,
        title: Option<&str>,
        body: Option<&str>,
        status: Option<&str>,
        assignee_id: Option<Option<Uuid>>,
    ) -> Result<Issue> {
        let row = sqlx::query_as::<_, Issue>(
            r#"UPDATE issues
               SET title      = COALESCE($2, title),
                   body       = COALESCE($3, body),
                   status     = COALESCE($4, status),
                   assignee_id = COALESCE($5, assignee_id),
                   updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(title)
        .bind(body)
        .bind(status)
        .bind(assignee_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("update_issue: {e}")))?;
        Ok(row)
    }

    // --- Pull Requests ---

    pub async fn create_pr(
        &self,
        repo_id: Uuid,
        title: &str,
        body: &str,
        author_id: Uuid,
        source_branch: &str,
        target_branch: &str,
    ) -> Result<PullRequest> {
        let row = sqlx::query_as::<_, PullRequest>(
            r#"INSERT INTO pull_requests (repo_id, title, body, author_id, source_branch, target_branch)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(title)
        .bind(body)
        .bind(author_id)
        .bind(source_branch)
        .bind(target_branch)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("create_pr: {e}")))?;
        Ok(row)
    }

    pub async fn get_pr(&self, id: Uuid) -> Result<PullRequest> {
        sqlx::query_as::<_, PullRequest>("SELECT * FROM pull_requests WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("get_pr: {e}")))
    }

    pub async fn list_prs(
        &self,
        repo_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PullRequest>> {
        let rows = sqlx::query_as::<_, PullRequest>(
            "SELECT * FROM pull_requests WHERE repo_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(repo_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("list_prs: {e}")))?;
        Ok(rows)
    }

    pub async fn update_pr(
        &self,
        id: Uuid,
        title: Option<&str>,
        body: Option<&str>,
        status: Option<&str>,
    ) -> Result<PullRequest> {
        let row = sqlx::query_as::<_, PullRequest>(
            r#"UPDATE pull_requests
               SET title      = COALESCE($2, title),
                   body       = COALESCE($3, body),
                   status     = COALESCE($4, status),
                   updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(title)
        .bind(body)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("update_pr: {e}")))?;
        Ok(row)
    }

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
        .map_err(|e| CoreError::Database(format!("create_pipeline: {e}")))?;
        Ok(row)
    }

    pub async fn get_pipeline(&self, id: Uuid) -> Result<Pipeline> {
        sqlx::query_as::<_, Pipeline>("SELECT * FROM pipelines WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("get_pipeline: {e}")))
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
        .map_err(|e| CoreError::Database(format!("list_pipelines: {e}")))?;
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
        .map_err(|e| CoreError::Database(format!("update_pipeline: {e}")))?;
        Ok(row)
    }

    // --- Access Tokens ---

    pub async fn create_access_token(
        &self,
        user_id: Uuid,
        name: &str,
        token_hash: &str,
        scopes: &[String],
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Uuid> {
        let row: (Uuid,) = sqlx::query_as(
            r#"INSERT INTO access_tokens (user_id, name, token_hash, scopes, expires_at)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING id"#,
        )
        .bind(user_id)
        .bind(name)
        .bind(token_hash)
        .bind(scopes)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("create_access_token: {e}")))?;
        Ok(row.0)
    }

    pub async fn validate_access_token(&self, token_hash: &str) -> Result<Uuid> {
        let row: (Uuid, Option<DateTime<Utc>>) = sqlx::query_as(
            r#"SELECT user_id, expires_at FROM access_tokens
               WHERE token_hash = $1"#,
        )
        .bind(token_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("validate_access_token: {e}")))?;

        if let Some(exp) = row.1 {
            if Utc::now() > exp {
                return Err(CoreError::Auth("access token expired".into()));
            }
        }

        Ok(row.0)
    }

    pub async fn revoke_access_token(&self, token_hash: &str) -> Result<()> {
        sqlx::query("DELETE FROM access_tokens WHERE token_hash = $1")
            .bind(token_hash)
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("revoke_access_token: {e}")))?;
        Ok(())
    }

    // --- Audit Events ---

    #[allow(clippy::too_many_arguments)]
    pub async fn record_audit_event(
        &self,
        actor_id: Uuid,
        action: &str,
        resource_type: &str,
        resource_id: Option<Uuid>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        outcome: &str,
    ) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            r#"INSERT INTO audit_events (actor_id, action, resource_type, resource_id, ip_address, user_agent, outcome)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING id"#,
        )
        .bind(actor_id)
        .bind(action)
        .bind(resource_type)
        .bind(resource_id)
        .bind(ip_address)
        .bind(user_agent)
        .bind(outcome)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("record_audit_event: {e}")))?;
        Ok(row.0)
    }

    pub async fn query_audit_events(
        &self,
        actor_id: Option<Uuid>,
        resource_type: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<(i64, Uuid, String, String, Option<Uuid>, DateTime<Utc>)>> {
        let rows = sqlx::query_as(
            r#"SELECT id, actor_id, action, resource_type, resource_id, created_at
               FROM audit_events
               WHERE ($1::uuid IS NULL OR actor_id = $1)
                 AND ($2::varchar IS NULL OR resource_type = $2)
               ORDER BY created_at DESC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(actor_id)
        .bind(resource_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("query_audit_events: {e}")))?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_db_repository_new_compiles() {
        fn _assert_send<T: Send>() {}
        fn _assert_sync<T: Sync>() {}
        _assert_send::<super::DbRepository>();
        _assert_sync::<super::DbRepository>();
    }
}
