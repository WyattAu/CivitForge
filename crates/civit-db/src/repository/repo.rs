#![forbid(unsafe_code)]
#![allow(clippy::too_many_arguments)]

use crate::error::{DbError, Result};
use crate::models::{BranchProtectionRule, Release, ReleaseAsset, Repository};
use super::OrgUsage;
use uuid::Uuid;

impl super::DbRepository {
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
        .map_err(|e| DbError::Database(format!("create_repo: {e}")))?;
        Ok(row)
    }

    pub async fn get_repo(&self, id: Uuid) -> Result<Repository> {
        sqlx::query_as::<_, Repository>("SELECT * FROM repositories WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_repo: {e}")))
    }

    pub async fn get_repo_by_owner_name(&self, owner_id: Uuid, name: &str) -> Result<Repository> {
        sqlx::query_as::<_, Repository>(
            "SELECT * FROM repositories WHERE owner_id = $1 AND name = $2",
        )
        .bind(owner_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_repo_by_owner_name: {e}")))
    }

    pub async fn count_repos(&self) -> Result<i64> {
        let row = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM repositories")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("count_repos: {e}")))?;
        Ok(row)
    }

    pub async fn list_repos(&self, limit: i64, offset: i64) -> Result<Vec<Repository>> {
        let rows = sqlx::query_as::<_, Repository>(
            "SELECT * FROM repositories ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_repos: {e}")))?;
        Ok(rows)
    }

    pub async fn list_repos_by_org(&self, org_id: Uuid) -> Result<Vec<Repository>> {
        let rows = sqlx::query_as::<_, Repository>("SELECT * FROM repositories WHERE org_id = $1")
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_repos_by_org: {e}")))?;
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
        .map_err(|e| DbError::Database(format!("update_repo: {e}")))?;
        Ok(row)
    }

    pub async fn delete_repo(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM repositories WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_repo: {e}")))?;
        Ok(())
    }

    // --- Multi-tenancy: Org-scoped resources ---

    pub async fn count_repos_by_org(&self, org_id: Uuid) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM repositories WHERE org_id = $1")
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("count_repos_by_org: {e}")))?;
        Ok(row.0)
    }

    pub async fn count_active_runners_by_org(&self, org_id: Uuid) -> Result<i64> {
        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM runners WHERE org_id = $1 AND status = 'active'")
                .bind(org_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| DbError::Database(format!("count_active_runners_by_org: {e}")))?;
        Ok(row.0)
    }

    pub async fn list_repos_visible_to_user(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Repository>> {
        let rows = sqlx::query_as::<_, Repository>(
            r#"SELECT * FROM repositories
               WHERE owner_id = $1
                  OR org_id IN (SELECT org_id FROM org_members WHERE user_id = $1)
                  OR visibility = 'public'
               ORDER BY updated_at DESC
               LIMIT $2 OFFSET $3"#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_repos_visible_to_user: {e}")))?;
        Ok(rows)
    }

    pub async fn user_has_repo_access(&self, user_id: Uuid, repo_id: Uuid) -> Result<bool> {
        let row: (bool,) = sqlx::query_as(
            r#"SELECT EXISTS(
                   SELECT 1 FROM repositories
                   WHERE id = $1
                     AND (owner_id = $2
                          OR org_id IN (SELECT org_id FROM org_members WHERE user_id = $2)
                          OR visibility = 'public')
               )"#,
        )
        .bind(repo_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("user_has_repo_access: {e}")))?;
        Ok(row.0)
    }

    pub async fn get_org_usage(&self, org_id: Uuid) -> Result<OrgUsage> {
        let repo_count = self.count_repos_by_org(org_id).await.unwrap_or(0);
        let member_count: i64 = {
            let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM org_members WHERE org_id = $1")
                .bind(org_id)
                .fetch_one(&self.pool)
                .await
                .unwrap_or((0i64,));
            row.0
        };
        Ok(OrgUsage {
            org_id,
            repo_count,
            member_count,
        })
    }

    // --- Forks ---

    pub async fn create_fork(
        &self,
        name: &str,
        description: &str,
        owner_id: Uuid,
        parent_repo_id: Uuid,
        visibility: &str,
        default_branch: &str,
    ) -> Result<Repository> {
        let row = sqlx::query_as::<_, Repository>(
            r#"INSERT INTO repositories (name, description, owner_id, visibility, default_branch, is_fork, parent_repo_id)
               VALUES ($1, $2, $3, $4, $5, true, $6)
               RETURNING id, name, description, owner_id, org_id, visibility, default_branch, is_fork, parent_repo_id, stars_count, watchers_count, created_at, updated_at"#,
        )
        .bind(name)
        .bind(description)
        .bind(owner_id)
        .bind(visibility)
        .bind(default_branch)
        .bind(parent_repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_fork: {e}")))?;
        Ok(row)
    }

    pub async fn list_forks(&self, parent_repo_id: Uuid) -> Result<Vec<Repository>> {
        let rows = sqlx::query_as::<_, Repository>(
            "SELECT * FROM repositories WHERE parent_repo_id = $1 ORDER BY created_at DESC",
        )
        .bind(parent_repo_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_forks: {e}")))?;
        Ok(rows)
    }

    // --- Releases ---

    #[allow(clippy::too_many_arguments)]
    pub async fn create_release(
        &self,
        repo_id: Uuid,
        tag_name: &str,
        name: &str,
        body: Option<&str>,
        draft: bool,
        prerelease: bool,
        author_id: Uuid,
    ) -> Result<Release> {
        // TECH DEBT: Inline DDL — should be moved to a proper migration.
        // Uses IF NOT EXISTS to avoid conflicts; safe to remove once migration exists.
        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS releases (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                tag_name TEXT NOT NULL,
                name TEXT NOT NULL,
                body TEXT,
                draft BOOLEAN NOT NULL DEFAULT false,
                prerelease BOOLEAN NOT NULL DEFAULT false,
                author_id UUID NOT NULL REFERENCES users(id),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                published_at TIMESTAMPTZ,
                UNIQUE(repo_id, tag_name)
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_release table: {e}")))?;

        let row = sqlx::query_as::<_, Release>(
            r#"INSERT INTO releases (repo_id, tag_name, name, body, draft, prerelease, author_id, published_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, CASE WHEN $5 = false THEN NOW() ELSE NULL END)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(tag_name)
        .bind(name)
        .bind(body)
        .bind(draft)
        .bind(prerelease)
        .bind(author_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_release: {e}")))?;
        Ok(row)
    }

    pub async fn list_releases(&self, repo_id: Uuid) -> Result<Vec<Release>> {
        let rows = sqlx::query_as::<_, Release>(
            "SELECT * FROM releases WHERE repo_id = $1 ORDER BY created_at DESC",
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_releases: {e}")))?;
        Ok(rows)
    }

    pub async fn get_release(&self, id: Uuid) -> Result<Release> {
        sqlx::query_as::<_, Release>("SELECT * FROM releases WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_release: {e}")))
    }

    pub async fn delete_release(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM releases WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_release: {e}")))?;
        Ok(())
    }

    // --- Release Assets ---

    pub async fn create_release_asset(
        &self,
        release_id: Uuid,
        name: &str,
        content_type: &str,
        size: i64,
        author_id: Uuid,
    ) -> Result<ReleaseAsset> {
        // TECH DEBT: Inline DDL — should be moved to a proper migration.
        // Uses IF NOT EXISTS to avoid conflicts; safe to remove once migration exists.
        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS release_assets (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                release_id UUID NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                content_type TEXT NOT NULL,
                size BIGINT NOT NULL DEFAULT 0,
                download_count BIGINT NOT NULL DEFAULT 0,
                author_id UUID NOT NULL REFERENCES users(id),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_release_asset table: {e}")))?;

        let row = sqlx::query_as::<_, ReleaseAsset>(
            r#"INSERT INTO release_assets (release_id, name, content_type, size, author_id)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(release_id)
        .bind(name)
        .bind(content_type)
        .bind(size)
        .bind(author_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_release_asset: {e}")))?;
        Ok(row)
    }

    pub async fn list_release_assets(&self, release_id: Uuid) -> Result<Vec<ReleaseAsset>> {
        let rows = sqlx::query_as::<_, ReleaseAsset>(
            "SELECT * FROM release_assets WHERE release_id = $1 ORDER BY created_at DESC",
        )
        .bind(release_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_release_assets: {e}")))?;
        Ok(rows)
    }

    // --- Branch Protection Rules ---

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_branch_protection(
        &self,
        repo_id: Uuid,
        branch_pattern: &str,
        require_pull_request: bool,
        required_approving_reviews: i32,
        required_status_checks: &[String],
        enforce_admins: bool,
        allow_force_pushes: bool,
        allow_deletions: bool,
    ) -> Result<BranchProtectionRule> {
        // TECH DEBT: Inline DDL — should be moved to a proper migration.
        // Uses IF NOT EXISTS to avoid conflicts; safe to remove once migration exists.
        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS branch_protection_rules (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                repo_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
                branch_pattern TEXT NOT NULL,
                require_pull_request BOOLEAN NOT NULL DEFAULT false,
                required_approving_reviews INTEGER NOT NULL DEFAULT 0,
                required_status_checks TEXT[] NOT NULL DEFAULT '{}',
                enforce_admins BOOLEAN NOT NULL DEFAULT false,
                allow_force_pushes BOOLEAN NOT NULL DEFAULT false,
                allow_deletions BOOLEAN NOT NULL DEFAULT false,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(repo_id, branch_pattern)
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("upsert_branch_protection table: {e}")))?;

        let row = sqlx::query_as::<_, BranchProtectionRule>(
            r#"INSERT INTO branch_protection_rules
                   (repo_id, branch_pattern, require_pull_request, required_approving_reviews,
                    required_status_checks, enforce_admins, allow_force_pushes, allow_deletions)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               ON CONFLICT (repo_id, branch_pattern)
               DO UPDATE SET
                   require_pull_request = $3,
                   required_approving_reviews = $4,
                   required_status_checks = $5,
                   enforce_admins = $6,
                   allow_force_pushes = $7,
                   allow_deletions = $8,
                   updated_at = NOW()
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(branch_pattern)
        .bind(require_pull_request)
        .bind(required_approving_reviews)
        .bind(required_status_checks)
        .bind(enforce_admins)
        .bind(allow_force_pushes)
        .bind(allow_deletions)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("upsert_branch_protection: {e}")))?;
        Ok(row)
    }

    pub async fn get_branch_protection(&self, repo_id: Uuid) -> Result<Vec<BranchProtectionRule>> {
        let rows = sqlx::query_as::<_, BranchProtectionRule>(
            "SELECT * FROM branch_protection_rules WHERE repo_id = $1 ORDER BY branch_pattern",
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_branch_protection: {e}")))?;
        Ok(rows)
    }

    // --- Topics ---

    pub async fn get_repo_topics(&self, repo_id: Uuid) -> Result<Vec<String>> {
        let row: (Option<Vec<String>>,) =
            sqlx::query_as("SELECT topics FROM repositories WHERE id = $1")
                .bind(repo_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| DbError::Database(format!("get_repo_topics: {e}")))?;
        Ok(row.0.unwrap_or_default())
    }

    pub async fn set_repo_topics(&self, repo_id: Uuid, topics: &[String]) -> Result<()> {
        sqlx::query("UPDATE repositories SET topics = $2, updated_at = NOW() WHERE id = $1")
            .bind(repo_id)
            .bind(topics)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("set_repo_topics: {e}")))?;
        Ok(())
    }

    // --- Archive ---

    pub async fn set_repo_archived(&self, repo_id: Uuid, archived: bool) -> Result<Repository> {
        let row = sqlx::query_as::<_, Repository>(
            r#"UPDATE repositories
               SET archived = $2, updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(archived)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("set_repo_archived: {e}")))?;
        Ok(row)
    }

    // --- Transfer ---

    pub async fn transfer_repo(&self, repo_id: Uuid, new_owner_id: Uuid) -> Result<Repository> {
        let row = sqlx::query_as::<_, Repository>(
            r#"UPDATE repositories
               SET owner_id = $2, updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(new_owner_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("transfer_repo: {e}")))?;
        Ok(row)
    }

    // --- Default branch ---

    pub async fn set_default_branch(&self, repo_id: Uuid, branch: &str) -> Result<Repository> {
        let row = sqlx::query_as::<_, Repository>(
            r#"UPDATE repositories
               SET default_branch = $2, updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(branch)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("set_default_branch: {e}")))?;
        Ok(row)
    }

    // --- Admin: list repos with search ---

    pub async fn admin_list_repos(
        &self,
        search: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Repository>> {
        let rows = if let Some(q) = search {
            let pattern = format!("%{q}%");
            sqlx::query_as::<_, Repository>(
                r#"SELECT * FROM repositories
                   WHERE name ILIKE $1 OR description ILIKE $1
                   ORDER BY created_at DESC
                   LIMIT $2 OFFSET $3"#,
            )
            .bind(&pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, Repository>(
                "SELECT * FROM repositories ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("admin_list_repos: {e}")))
    }


}
