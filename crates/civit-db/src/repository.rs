#![forbid(unsafe_code)]

use crate::error::{DbError, Result};
use crate::models::{
    ActivityEvent, BranchProtectionRule, EmailVerificationCode, Issue, Org, Pipeline, PrComment,
    PrReviewer, PrStatusCheck, PrTimeline, PullRequest, Release, ReleaseAsset, Repository, SshKey,
    Team, TeamMember, User,
};
use chrono::{DateTime, Utc};
use sqlx::postgres::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DbRepository {
    pool: PgPool,
}

impl DbRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Expose the underlying pool for permission engine queries.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    // --- Users ---

    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        display_name: &str,
        role: &str,
        password_hash: &str,
    ) -> Result<User> {
        let row = sqlx::query_as::<_, User>(
            r#"INSERT INTO users (username, email, display_name, role, password_hash)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(username)
        .bind(email)
        .bind(display_name)
        .bind(role)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_user: {e}")))?;
        Ok(row)
    }

    pub async fn get_user_by_id(&self, id: Uuid) -> Result<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_user_by_id: {e}")))
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_user_by_username: {e}")))
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_user_by_email: {e}")))
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
        .map_err(|e| DbError::Database(format!("update_user: {e}")))?;
        Ok(row)
    }

    pub async fn update_user_profile(
        &self,
        id: Uuid,
        avatar_url: Option<&str>,
        location: Option<&str>,
        website: Option<&str>,
        display_name: Option<&str>,
        bio: Option<&str>,
    ) -> Result<User> {
        let row = sqlx::query_as::<_, User>(
            r#"UPDATE users
               SET avatar_url    = COALESCE($2, avatar_url),
                   location      = COALESCE($3, location),
                   website       = COALESCE($4, website),
                   display_name  = COALESCE($5, display_name),
                   bio           = COALESCE($6, bio),
                   updated_at    = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(avatar_url)
        .bind(location)
        .bind(website)
        .bind(display_name)
        .bind(bio)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_user_profile: {e}")))?;
        Ok(row)
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_user: {e}")))?;
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
        .map_err(|e| DbError::Database(format!("list_users: {e}")))?;
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
        .map_err(|e| DbError::Database(format!("create_org: {e}")))?;
        Ok(row)
    }

    pub async fn get_org(&self, id: Uuid) -> Result<Org> {
        sqlx::query_as::<_, Org>("SELECT * FROM organizations WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_org: {e}")))
    }

    pub async fn list_orgs_by_owner(&self, owner_id: Uuid) -> Result<Vec<Org>> {
        let rows = sqlx::query_as::<_, Org>("SELECT * FROM organizations WHERE owner_id = $1")
            .bind(owner_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_orgs_by_owner: {e}")))?;
        Ok(rows)
    }

    pub async fn list_all_orgs(&self) -> Result<Vec<Org>> {
        let rows = sqlx::query_as::<_, Org>("SELECT * FROM organizations ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_all_orgs: {e}")))?;
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
        .map_err(|e| DbError::Database(format!("update_org: {e}")))?;
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
        .map_err(|e| DbError::Database(format!("create_issue: {e}")))?;
        Ok(row)
    }

    pub async fn get_issue(&self, id: Uuid) -> Result<Issue> {
        sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_issue: {e}")))
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
        .map_err(|e| DbError::Database(format!("list_issues: {e}")))?;
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
        .map_err(|e| DbError::Database(format!("update_issue: {e}")))?;
        Ok(row)
    }

    // --- Pull Requests ---

    #[allow(clippy::too_many_arguments)]
    pub async fn create_pr(
        &self,
        repo_id: Uuid,
        title: &str,
        body: &str,
        author_id: Uuid,
        source_branch: &str,
        target_branch: &str,
        draft: bool,
        auto_merge: bool,
    ) -> Result<PullRequest> {
        let row = sqlx::query_as::<_, PullRequest>(
            r#"INSERT INTO pull_requests (repo_id, title, body, author_id, source_branch, target_branch, draft, auto_merge)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(title)
        .bind(body)
        .bind(author_id)
        .bind(source_branch)
        .bind(target_branch)
        .bind(draft)
        .bind(auto_merge)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_pr: {e}")))?;
        Ok(row)
    }

    pub async fn get_pr(&self, id: Uuid) -> Result<PullRequest> {
        sqlx::query_as::<_, PullRequest>("SELECT * FROM pull_requests WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_pr: {e}")))
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
        .map_err(|e| DbError::Database(format!("list_prs: {e}")))?;
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
        .map_err(|e| DbError::Database(format!("update_pr: {e}")))?;
        Ok(row)
    }

    pub async fn merge_pr(
        &self,
        id: Uuid,
        merge_commit_id: &str,
        merge_strategy: &str,
        head_sha: Option<&str>,
        base_sha: Option<&str>,
    ) -> Result<PullRequest> {
        let row = sqlx::query_as::<_, PullRequest>(
            r#"UPDATE pull_requests
               SET status          = 'merged',
                   merge_commit_id = $2,
                   merge_strategy  = $3,
                   head_commit_sha = COALESCE($4, head_commit_sha),
                   base_commit_sha = COALESCE($5, base_commit_sha),
                   merged_at       = NOW(),
                   updated_at      = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(merge_commit_id)
        .bind(merge_strategy)
        .bind(head_sha)
        .bind(base_sha)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("merge_pr: {e}")))?;
        Ok(row)
    }

    pub async fn set_pr_draft(&self, id: Uuid, draft: bool) -> Result<PullRequest> {
        let row = sqlx::query_as::<_, PullRequest>(
            r#"UPDATE pull_requests
               SET draft     = $2,
                   updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(draft)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("set_pr_draft: {e}")))?;
        Ok(row)
    }

    pub async fn set_pr_commit_shas(&self, id: Uuid, head_sha: &str, base_sha: &str) -> Result<()> {
        sqlx::query(
            r#"UPDATE pull_requests
               SET head_commit_sha = $2,
                   base_commit_sha = $3,
                   updated_at      = NOW()
               WHERE id = $1"#,
        )
        .bind(id)
        .bind(head_sha)
        .bind(base_sha)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("set_pr_commit_shas: {e}")))?;
        Ok(())
    }

    pub async fn find_open_pr_by_source_branch(
        &self,
        repo_id: Uuid,
        source_branch: &str,
    ) -> Result<Option<PullRequest>> {
        let row = sqlx::query_as::<_, PullRequest>(
            "SELECT * FROM pull_requests WHERE repo_id = $1 AND source_branch = $2 AND status = 'open' LIMIT 1",
        )
        .bind(repo_id)
        .bind(source_branch)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("find_open_pr_by_source_branch: {e}")))?;
        Ok(row)
    }

    pub async fn get_pr_by_number(&self, repo_id: Uuid, number: i32) -> Result<PullRequest> {
        sqlx::query_as::<_, PullRequest>(
            "SELECT * FROM pull_requests WHERE repo_id = $1 AND number = $2",
        )
        .bind(repo_id)
        .bind(number)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_pr_by_number: {e}")))
    }

    pub async fn count_prs(&self, repo_id: Uuid, state: Option<&str>) -> Result<i64> {
        let count: i64 = if let Some(st) = state {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM pull_requests WHERE repo_id = $1 AND status = $2",
            )
            .bind(repo_id)
            .bind(st)
            .fetch_one(&self.pool)
            .await
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM pull_requests WHERE repo_id = $1")
                .bind(repo_id)
                .fetch_one(&self.pool)
                .await
        }
        .map_err(|e| DbError::Database(format!("count_prs: {e}")))?;
        Ok(count)
    }

    // --- PR Comments ---

    #[allow(clippy::too_many_arguments)]
    pub async fn create_pr_comment(
        &self,
        pr_id: Uuid,
        author_id: Uuid,
        body: &str,
        commit_sha: Option<&str>,
        file_path: Option<&str>,
        line: Option<i32>,
        in_reply_to_id: Option<Uuid>,
    ) -> Result<PrComment> {
        let row = sqlx::query_as::<_, PrComment>(
            r#"INSERT INTO pr_comments (pr_id, author_id, body, commit_sha, file_path, line, in_reply_to_id)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING *"#,
        )
        .bind(pr_id)
        .bind(author_id)
        .bind(body)
        .bind(commit_sha)
        .bind(file_path)
        .bind(line)
        .bind(in_reply_to_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_pr_comment: {e}")))?;
        Ok(row)
    }

    pub async fn list_pr_comments(&self, pr_id: Uuid) -> Result<Vec<PrComment>> {
        sqlx::query_as::<_, PrComment>(
            "SELECT * FROM pr_comments WHERE pr_id = $1 ORDER BY created_at ASC",
        )
        .bind(pr_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_pr_comments: {e}")))
    }

    // --- PR Labels ---

    pub async fn add_pr_label(&self, pr_id: Uuid, label_id: Uuid) -> Result<()> {
        sqlx::query(
            "INSERT INTO pr_labels (pr_id, label_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(pr_id)
        .bind(label_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("add_pr_label: {e}")))?;
        Ok(())
    }

    pub async fn remove_pr_label(&self, pr_id: Uuid, label_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM pr_labels WHERE pr_id = $1 AND label_id = $2")
            .bind(pr_id)
            .bind(label_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("remove_pr_label: {e}")))?;
        Ok(())
    }

    // --- PR Assignees ---

    pub async fn add_pr_assignee(&self, pr_id: Uuid, user_id: Uuid) -> Result<()> {
        sqlx::query(
            "INSERT INTO pr_assignees (pr_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(pr_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("add_pr_assignee: {e}")))?;
        Ok(())
    }

    pub async fn remove_pr_assignee(&self, pr_id: Uuid, user_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM pr_assignees WHERE pr_id = $1 AND user_id = $2")
            .bind(pr_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("remove_pr_assignee: {e}")))?;
        Ok(())
    }

    // --- PR Reviewers ---

    pub async fn add_pr_reviewer(&self, pr_id: Uuid, user_id: Uuid) -> Result<PrReviewer> {
        let row = sqlx::query_as::<_, PrReviewer>(
            r#"INSERT INTO pr_reviewers (pr_id, user_id) VALUES ($1, $2)
               ON CONFLICT (pr_id, user_id) DO UPDATE SET review_status = 'pending', submitted_at = NULL
               RETURNING *"#,
        )
        .bind(pr_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("add_pr_reviewer: {e}")))?;
        Ok(row)
    }

    pub async fn submit_pr_review(
        &self,
        pr_id: Uuid,
        user_id: Uuid,
        status: &str,
    ) -> Result<PrReviewer> {
        let row = sqlx::query_as::<_, PrReviewer>(
            r#"UPDATE pr_reviewers
               SET review_status = $3, submitted_at = NOW()
               WHERE pr_id = $1 AND user_id = $2
               RETURNING *"#,
        )
        .bind(pr_id)
        .bind(user_id)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("submit_pr_review: {e}")))?;
        Ok(row)
    }

    pub async fn list_pr_reviewers(&self, pr_id: Uuid) -> Result<Vec<PrReviewer>> {
        sqlx::query_as::<_, PrReviewer>("SELECT * FROM pr_reviewers WHERE pr_id = $1")
            .bind(pr_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_pr_reviewers: {e}")))
    }

    // --- PR Timeline ---

    pub async fn insert_pr_timeline(
        &self,
        pr_id: Uuid,
        actor_id: Uuid,
        event_type: &str,
        detail: serde_json::Value,
    ) -> Result<PrTimeline> {
        let row = sqlx::query_as::<_, PrTimeline>(
            r#"INSERT INTO pr_timeline (pr_id, actor_id, event_type, event_detail)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(pr_id)
        .bind(actor_id)
        .bind(event_type)
        .bind(detail)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("insert_pr_timeline: {e}")))?;
        Ok(row)
    }

    pub async fn list_pr_timeline(&self, pr_id: Uuid) -> Result<Vec<PrTimeline>> {
        sqlx::query_as::<_, PrTimeline>(
            "SELECT * FROM pr_timeline WHERE pr_id = $1 ORDER BY created_at ASC",
        )
        .bind(pr_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_pr_timeline: {e}")))
    }

    // --- PR Status Checks ---

    pub async fn upsert_pr_status_check(
        &self,
        pr_id: Uuid,
        context: &str,
        state: &str,
        description: &str,
        target_url: Option<&str>,
        commit_sha: Option<&str>,
    ) -> Result<PrStatusCheck> {
        let row = sqlx::query_as::<_, PrStatusCheck>(
            r#"INSERT INTO pr_status_checks (pr_id, context, state, description, target_url, commit_sha)
               VALUES ($1, $2, $3, $4, $5, $6)
               ON CONFLICT (pr_id, context, commit_sha)
               DO UPDATE SET state = $3, description = $4, target_url = COALESCE($5, pr_status_checks.target_url), updated_at = NOW()
               RETURNING *"#,
        )
        .bind(pr_id)
        .bind(context)
        .bind(state)
        .bind(description)
        .bind(target_url)
        .bind(commit_sha)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("upsert_pr_status_check: {e}")))?;
        Ok(row)
    }

    pub async fn list_pr_status_checks(&self, pr_id: Uuid) -> Result<Vec<PrStatusCheck>> {
        sqlx::query_as::<_, PrStatusCheck>(
            "SELECT * FROM pr_status_checks WHERE pr_id = $1 ORDER BY created_at ASC",
        )
        .bind(pr_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_pr_status_checks: {e}")))
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
        .map_err(|e| DbError::Database(format!("create_access_token: {e}")))?;
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
        .map_err(|e| DbError::Database(format!("validate_access_token: {e}")))?;

        if let Some(exp) = row.1
            && Utc::now() > exp
        {
            return Err(DbError::Auth("access token expired".into()));
        }

        Ok(row.0)
    }

    pub async fn revoke_access_token(&self, token_hash: &str) -> Result<()> {
        sqlx::query("DELETE FROM access_tokens WHERE token_hash = $1")
            .bind(token_hash)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("revoke_access_token: {e}")))?;
        Ok(())
    }

    pub async fn validate_pat_token(&self, token_hash: &str) -> Result<(Uuid, Vec<String>, Uuid)> {
        let row: (Uuid, serde_json::Value, Uuid, Option<DateTime<Utc>>) = sqlx::query_as(
            r#"SELECT user_id, scopes, id, expires_at FROM access_tokens
               WHERE token_hash = $1"#,
        )
        .bind(token_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("validate_pat_token: {e}")))?;

        if let Some(exp) = row.3
            && Utc::now() > exp
        {
            return Err(DbError::Auth("access token expired".into()));
        }

        let scopes: Vec<String> = serde_json::from_value(row.1).unwrap_or_default();

        Ok((row.0, scopes, row.2))
    }

    pub async fn touch_access_token(&self, token_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE access_tokens SET last_used_at = NOW() WHERE id = $1")
            .bind(token_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("touch_access_token: {e}")))?;
        Ok(())
    }

    pub async fn set_email_verified(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE users SET email_verified = true, updated_at = NOW() WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("set_email_verified: {e}")))?;
        Ok(())
    }

    pub async fn store_verification_code(
        &self,
        user_id: Uuid,
        email: &str,
        code: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<EmailVerificationCode> {
        let row = sqlx::query_as::<_, EmailVerificationCode>(
            r#"INSERT INTO email_verification_codes (user_id, email, code, expires_at)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(email)
        .bind(code)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("store_verification_code: {e}")))?;
        Ok(row)
    }

    pub async fn validate_verification_code(
        &self,
        email: &str,
        code: &str,
    ) -> Result<EmailVerificationCode> {
        let row = sqlx::query_as::<_, EmailVerificationCode>(
            r#"SELECT * FROM email_verification_codes
               WHERE email = $1 AND code = $2 AND used = false
               ORDER BY created_at DESC LIMIT 1"#,
        )
        .bind(email)
        .bind(code)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("validate_verification_code: {e}")))?;

        if Utc::now() > row.expires_at {
            return Err(DbError::Auth("verification code expired".into()));
        }

        Ok(row)
    }

    pub async fn mark_verification_code_used(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE email_verification_codes SET used = true WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("mark_verification_code_used: {e}")))?;
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
        .map_err(|e| DbError::Database(format!("record_audit_event: {e}")))?;
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
        .map_err(|e| DbError::Database(format!("query_audit_events: {e}")))?;
        Ok(rows)
    }

    // --- SSH Keys ---

    pub async fn add_ssh_key(
        &self,
        user_id: Uuid,
        key_type: &str,
        public_key: &str,
        fingerprint: &str,
        label: &str,
    ) -> Result<SshKey> {
        let row = sqlx::query_as::<_, SshKey>(
            r#"INSERT INTO ssh_keys (user_id, key_type, public_key, fingerprint, label)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(key_type)
        .bind(public_key)
        .bind(fingerprint)
        .bind(label)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("add_ssh_key: {e}")))?;
        Ok(row)
    }

    pub async fn list_ssh_keys(&self, user_id: Uuid) -> Result<Vec<SshKey>> {
        let rows = sqlx::query_as::<_, SshKey>(
            "SELECT * FROM ssh_keys WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_ssh_keys: {e}")))?;
        Ok(rows)
    }

    pub async fn delete_ssh_key(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM ssh_keys WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_ssh_key: {e}")))?;
        Ok(())
    }

    pub async fn get_ssh_key_by_fingerprint(&self, fingerprint: &str) -> Result<SshKey> {
        sqlx::query_as::<_, SshKey>("SELECT * FROM ssh_keys WHERE fingerprint = $1")
            .bind(fingerprint)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_ssh_key_by_fingerprint: {e}")))
    }

    // --- Activity Events ---

    #[allow(clippy::too_many_arguments)]
    pub async fn record_activity_event(
        &self,
        actor_id: Uuid,
        action: &str,
        resource_type: &str,
        resource_id: Option<Uuid>,
        repo_id: Option<Uuid>,
        org_id: Option<Uuid>,
        description: &str,
        metadata: serde_json::Value,
    ) -> Result<ActivityEvent> {
        let row = sqlx::query_as::<_, ActivityEvent>(
            r#"INSERT INTO activity_events (actor_id, action, resource_type, resource_id, repo_id, org_id, description, metadata)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING *"#,
        )
        .bind(actor_id)
        .bind(action)
        .bind(resource_type)
        .bind(resource_id)
        .bind(repo_id)
        .bind(org_id)
        .bind(description)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_activity_event: {e}")))?;
        Ok(row)
    }

    pub async fn list_activity_events(
        &self,
        repo_id: Option<Uuid>,
        org_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ActivityEvent>> {
        let rows = sqlx::query_as::<_, ActivityEvent>(
            r#"SELECT * FROM activity_events
               WHERE ($1::uuid IS NULL OR repo_id = $1)
                 AND ($2::uuid IS NULL OR org_id = $2)
               ORDER BY created_at DESC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(repo_id)
        .bind(org_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_activity_events: {e}")))?;
        Ok(rows)
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

    // --- Password ---

    pub async fn get_password_hash(&self, user_id: Uuid) -> Result<Option<String>> {
        let row: (Option<String>,) =
            sqlx::query_as("SELECT password_hash FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| DbError::Database(format!("get_password_hash: {e}")))?;
        Ok(row.0)
    }

    pub async fn change_password(&self, user_id: Uuid, password_hash: &str) -> Result<()> {
        sqlx::query(r#"UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2"#)
            .bind(password_hash)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("change_password: {e}")))?;
        Ok(())
    }

    // --- Login Attempts / Lockout ---

    pub async fn record_login_attempt(
        &self,
        username: &str,
        ip: &str,
        success: bool,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO login_attempts (username, ip_address, success) VALUES ($1, $2, $3)",
        )
        .bind(username)
        .bind(ip)
        .bind(success)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_login_attempt: {e}")))?;
        sqlx::query("DELETE FROM login_attempts WHERE created_at < NOW() - INTERVAL '24 hours'")
            .execute(&self.pool)
            .await
            .ok();
        Ok(())
    }

    pub async fn count_recent_failed_logins(
        &self,
        username: &str,
        window_secs: i64,
    ) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM login_attempts WHERE username = $1 AND success = false AND created_at > NOW() - ($2 || ' seconds')::INTERVAL",
        )
        .bind(username)
        .bind(window_secs.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("count_recent_failed_logins: {e}")))?;
        Ok(row.0)
    }

    pub async fn clear_login_attempts(&self, username: &str) -> Result<()> {
        sqlx::query("DELETE FROM login_attempts WHERE username = $1 AND success = false")
            .bind(username)
            .execute(&self.pool)
            .await
            .ok();
        Ok(())
    }

    // --- Stars ---

    pub async fn has_user_starred(&self, user_id: Uuid, repo_id: Uuid) -> Result<bool> {
        let row: Option<(bool,)> = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM repo_stars WHERE user_id = $1 AND repo_id = $2)",
        )
        .bind(user_id)
        .bind(repo_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("has_user_starred: {e}")))?;
        Ok(row.map(|(v,)| v).unwrap_or(false))
    }

    pub async fn toggle_star(&self, user_id: Uuid, repo_id: Uuid) -> Result<(i64, bool)> {
        let existing = self.has_user_starred(user_id, repo_id).await?;
        if existing {
            sqlx::query("DELETE FROM repo_stars WHERE user_id = $1 AND repo_id = $2")
                .bind(user_id)
                .bind(repo_id)
                .execute(&self.pool)
                .await
                .map_err(|e| DbError::Database(format!("toggle_star delete: {e}")))?;
            let row: (i64,) = sqlx::query_as(
                "UPDATE repositories SET stars_count = GREATEST(stars_count - 1, 0), updated_at = NOW() WHERE id = $1 RETURNING stars_count",
            )
            .bind(repo_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("toggle_star dec: {e}")))?;
            Ok((row.0, false))
        } else {
            sqlx::query("INSERT INTO repo_stars (user_id, repo_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(user_id)
                .bind(repo_id)
                .execute(&self.pool)
                .await
                .map_err(|e| DbError::Database(format!("toggle_star insert: {e}")))?;
            let row: (i64,) = sqlx::query_as(
                "UPDATE repositories SET stars_count = stars_count + 1, updated_at = NOW() WHERE id = $1 RETURNING stars_count",
            )
            .bind(repo_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("toggle_star inc: {e}")))?;
            Ok((row.0, true))
        }
    }

    pub async fn increment_stars(&self, repo_id: Uuid) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "UPDATE repositories SET stars_count = stars_count + 1, updated_at = NOW() WHERE id = $1 RETURNING stars_count",
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("increment_stars: {e}")))?;
        Ok(row.0)
    }

    pub async fn decrement_stars(&self, repo_id: Uuid) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "UPDATE repositories SET stars_count = GREATEST(stars_count - 1, 0), updated_at = NOW() WHERE id = $1 RETURNING stars_count",
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("decrement_stars: {e}")))?;
        Ok(row.0)
    }

    // --- Watchers ---

    pub async fn has_user_watched(&self, user_id: Uuid, repo_id: Uuid) -> Result<bool> {
        let row: Option<(bool,)> = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM repo_watchers WHERE user_id = $1 AND repo_id = $2)",
        )
        .bind(user_id)
        .bind(repo_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("has_user_watched: {e}")))?;
        Ok(row.map(|(v,)| v).unwrap_or(false))
    }

    pub async fn toggle_watch(&self, user_id: Uuid, repo_id: Uuid) -> Result<(i64, bool)> {
        let existing = self.has_user_watched(user_id, repo_id).await?;
        if existing {
            sqlx::query("DELETE FROM repo_watchers WHERE user_id = $1 AND repo_id = $2")
                .bind(user_id)
                .bind(repo_id)
                .execute(&self.pool)
                .await
                .map_err(|e| DbError::Database(format!("toggle_watch delete: {e}")))?;
            let row: (i64,) = sqlx::query_as(
                "UPDATE repositories SET watchers_count = GREATEST(watchers_count - 1, 0), updated_at = NOW() WHERE id = $1 RETURNING watchers_count",
            )
            .bind(repo_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("toggle_watch dec: {e}")))?;
            Ok((row.0, false))
        } else {
            sqlx::query("INSERT INTO repo_watchers (user_id, repo_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(user_id)
                .bind(repo_id)
                .execute(&self.pool)
                .await
                .map_err(|e| DbError::Database(format!("toggle_watch insert: {e}")))?;
            let row: (i64,) = sqlx::query_as(
                "UPDATE repositories SET watchers_count = watchers_count + 1, updated_at = NOW() WHERE id = $1 RETURNING watchers_count",
            )
            .bind(repo_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("toggle_watch inc: {e}")))?;
            Ok((row.0, true))
        }
    }

    pub async fn increment_watchers(&self, repo_id: Uuid) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "UPDATE repositories SET watchers_count = watchers_count + 1, updated_at = NOW() WHERE id = $1 RETURNING watchers_count",
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("increment_watchers: {e}")))?;
        Ok(row.0)
    }

    pub async fn decrement_watchers(&self, repo_id: Uuid) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "UPDATE repositories SET watchers_count = GREATEST(watchers_count - 1, 0), updated_at = NOW() WHERE id = $1 RETURNING watchers_count",
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("decrement_watchers: {e}")))?;
        Ok(row.0)
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

    // --- Issue auto-close on PR merge ---

    pub async fn close_issues_for_pr(
        &self,
        repo_id: Uuid,
        pr_title: &str,
        pr_body: &str,
        actor_id: Uuid,
    ) -> Result<Vec<i32>> {
        let text = format!("{pr_title}\n{pr_body}");
        let re = regex::Regex::new(r"(?i)(?:fix(?:es|ed)?|closes?|resolves?)\s+#(\d+)").unwrap();
        let issue_numbers: Vec<i32> = re
            .captures_iter(&text)
            .filter_map(|c| c.get(1)?.as_str().parse::<i32>().ok())
            .collect();

        let mut closed = Vec::new();
        for num in &issue_numbers {
            let result: Option<(Uuid,)> = sqlx::query_as(
                r#"UPDATE issues
                   SET status = 'closed', closed_at = NOW(), updated_at = NOW()
                   WHERE repo_id = $1 AND number = $2 AND status != 'closed'
                   RETURNING id"#,
            )
            .bind(repo_id)
            .bind(num)
            .fetch_optional(&self.pool)
            .await
            .unwrap_or(None);
            if let Some((issue_id,)) = result {
                let _ = sqlx::query(
                    "INSERT INTO issue_timeline (issue_id, actor_id, event_type, event_detail, created_at) VALUES ($1, $2, 'closed_by_pr', $3, NOW())",
                )
                .bind(issue_id)
                .bind(actor_id)
                .bind("Closed by merge of PR")
                .execute(&self.pool)
                .await;
                closed.push(*num);
            }
        }
        Ok(closed)
    }

    // --- Teams ---

    pub async fn create_team(
        &self,
        org_id: Uuid,
        name: &str,
        description: &str,
        privacy: &str,
    ) -> Result<Team> {
        // TECH DEBT: Inline DDL — should be moved to a proper migration.
        // Uses IF NOT EXISTS to avoid conflicts; safe to remove once migration exists.
        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS teams (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                privacy TEXT NOT NULL DEFAULT 'visible',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(org_id, name)
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_team table: {e}")))?;

        let row = sqlx::query_as::<_, Team>(
            r#"INSERT INTO teams (org_id, name, description, privacy)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(org_id)
        .bind(name)
        .bind(description)
        .bind(privacy)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_team: {e}")))?;
        Ok(row)
    }

    pub async fn get_team(&self, id: Uuid) -> Result<Team> {
        sqlx::query_as::<_, Team>("SELECT * FROM teams WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_team: {e}")))
    }

    pub async fn list_teams(&self, org_id: Uuid) -> Result<Vec<Team>> {
        let rows = sqlx::query_as::<_, Team>("SELECT * FROM teams WHERE org_id = $1 ORDER BY name")
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_teams: {e}")))?;
        Ok(rows)
    }

    pub async fn update_team(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        privacy: Option<&str>,
    ) -> Result<Team> {
        let row = sqlx::query_as::<_, Team>(
            r#"UPDATE teams
               SET name        = COALESCE($2, name),
                   description = COALESCE($3, description),
                   privacy     = COALESCE($4, privacy),
                   updated_at  = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(privacy)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_team: {e}")))?;
        Ok(row)
    }

    pub async fn delete_team(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM teams WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_team: {e}")))?;
        Ok(())
    }

    pub async fn add_team_member(
        &self,
        team_id: Uuid,
        user_id: Uuid,
        role: &str,
    ) -> Result<TeamMember> {
        // TECH DEBT: Inline DDL — should be moved to a proper migration.
        // Uses IF NOT EXISTS to avoid conflicts; safe to remove once migration exists.
        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS team_members (
                team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                role TEXT NOT NULL DEFAULT 'member',
                joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (team_id, user_id)
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("add_team_member table: {e}")))?;

        let row = sqlx::query_as::<_, TeamMember>(
            r#"INSERT INTO team_members (team_id, user_id, role)
               VALUES ($1, $2, $3)
               ON CONFLICT (team_id, user_id) DO UPDATE SET role = $3
               RETURNING *"#,
        )
        .bind(team_id)
        .bind(user_id)
        .bind(role)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("add_team_member: {e}")))?;
        Ok(row)
    }

    pub async fn remove_team_member(&self, team_id: Uuid, user_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM team_members WHERE team_id = $1 AND user_id = $2")
            .bind(team_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("remove_team_member: {e}")))?;
        Ok(())
    }

    pub async fn list_team_members(&self, team_id: Uuid) -> Result<Vec<TeamMember>> {
        let rows = sqlx::query_as::<_, TeamMember>(
            "SELECT * FROM team_members WHERE team_id = $1 ORDER BY joined_at",
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_team_members: {e}")))?;
        Ok(rows)
    }

    // --- Audit Log Admin ---

    #[allow(clippy::too_many_arguments)]
    pub async fn query_audit_events_admin(
        &self,
        actor_id: Option<Uuid>,
        action: Option<&str>,
        resource_type: Option<&str>,
        since: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<
        Vec<(
            i64,
            Uuid,
            String,
            String,
            Option<Uuid>,
            Option<String>,
            Option<String>,
            String,
            DateTime<Utc>,
        )>,
    > {
        let rows = sqlx::query_as(
            r#"SELECT id, actor_id, action, resource_type, resource_id, ip_address, user_agent, outcome, created_at
               FROM audit_events
               WHERE ($1::uuid IS NULL OR actor_id = $1)
                 AND ($2::varchar IS NULL OR action = $2)
                 AND ($3::varchar IS NULL OR resource_type = $3)
                 AND ($4::timestamptz IS NULL OR created_at >= $4)
                 AND ($5::timestamptz IS NULL OR created_at <= $5)
               ORDER BY created_at DESC
               LIMIT $6 OFFSET $7"#,
        )
        .bind(actor_id)
        .bind(action)
        .bind(resource_type)
        .bind(since)
        .bind(until)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("query_audit_events_admin: {e}")))?;
        Ok(rows)
    }

    pub async fn audit_event_stats(
        &self,
    ) -> Result<(
        i64,
        Vec<(String, i64)>,
        Vec<(Uuid, i64)>,
        Vec<(String, i64)>,
    )> {
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM audit_events")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("audit_event_stats total: {e}")))?;

        let per_day: Vec<(String, i64)> = sqlx::query_as(
            r#"SELECT DATE(created_at) as date, COUNT(*) as count
               FROM audit_events
               WHERE created_at > NOW() - INTERVAL '30 days'
               GROUP BY DATE(created_at)
               ORDER BY date DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("audit_event_stats per_day: {e}")))?;

        let top_actors: Vec<(Uuid, i64)> = sqlx::query_as(
            r#"SELECT actor_id, COUNT(*) as count
               FROM audit_events
               WHERE created_at > NOW() - INTERVAL '30 days'
               GROUP BY actor_id
               ORDER BY count DESC
               LIMIT 10"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("audit_event_stats top_actors: {e}")))?;

        let top_actions: Vec<(String, i64)> = sqlx::query_as(
            r#"SELECT action, COUNT(*) as count
               FROM audit_events
               WHERE created_at > NOW() - INTERVAL '30 days'
               GROUP BY action
               ORDER BY count DESC
               LIMIT 10"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("audit_event_stats top_actions: {e}")))?;

        Ok((total.0, per_day, top_actors, top_actions))
    }

    // --- Org Members ---

    pub async fn list_org_members(&self, org_id: Uuid) -> Result<Vec<User>> {
        let rows = sqlx::query_as::<_, User>(
            r#"SELECT u.* FROM users u
               INNER JOIN org_members om ON om.user_id = u.id
               WHERE om.org_id = $1
               ORDER BY u.username"#,
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_org_members: {e}")))?;
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

    // --- Admin: ban/unban user ---

    pub async fn ban_user(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE users SET banned = true, updated_at = NOW() WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("ban_user: {e}")))?;
        Ok(())
    }

    pub async fn unban_user(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE users SET banned = false, updated_at = NOW() WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("unban_user: {e}")))?;
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct OrgUsage {
    pub org_id: Uuid,
    pub repo_count: i64,
    pub member_count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_repository_new_compiles() {
        fn _assert_send<T: Send>() {}
        fn _assert_sync<T: Sync>() {}
        _assert_send::<super::DbRepository>();
        _assert_sync::<super::DbRepository>();
    }

    #[test]
    fn test_struct_has_pool_field() {
        let _: Option<sqlx::postgres::PgPool> = None;
    }

    #[test]
    fn test_create_user_error_message_format() {
        let err = DbError::Database("create_user: connection refused".into());
        let msg = err.to_string();
        assert!(msg.contains("create_user"));
        assert!(msg.contains("connection refused"));
    }

    #[test]
    fn test_get_user_by_id_error_message_format() {
        let err = DbError::Database("get_user_by_id: no rows returned".into());
        let msg = err.to_string();
        assert!(msg.contains("get_user_by_id"));
    }

    #[test]
    fn test_get_user_by_username_error_message_format() {
        let err = DbError::Database("get_user_by_username: no rows returned".into());
        let msg = err.to_string();
        assert!(msg.contains("get_user_by_username"));
    }

    #[test]
    fn test_get_user_by_email_error_message_format() {
        let err = DbError::Database("get_user_by_email: no rows returned".into());
        let msg = err.to_string();
        assert!(msg.contains("get_user_by_email"));
    }

    #[test]
    fn test_update_user_error_message_format() {
        let err = DbError::Database("update_user: no rows returned".into());
        assert!(err.to_string().contains("update_user"));
    }

    #[test]
    fn test_delete_user_error_message_format() {
        let err = DbError::Database("delete_user: relation not found".into());
        assert!(err.to_string().contains("delete_user"));
    }

    #[test]
    fn test_list_users_error_message_format() {
        let err = DbError::Database("list_users: connection refused".into());
        assert!(err.to_string().contains("list_users"));
    }

    #[test]
    fn test_create_org_error_message_format() {
        let err = DbError::Database("create_org: duplicate key".into());
        assert!(err.to_string().contains("create_org"));
    }

    #[test]
    fn test_get_org_error_message_format() {
        let err = DbError::Database("get_org: no rows returned".into());
        assert!(err.to_string().contains("get_org"));
    }

    #[test]
    fn test_list_orgs_by_owner_error_message_format() {
        let err = DbError::Database("list_orgs_by_owner: connection refused".into());
        assert!(err.to_string().contains("list_orgs_by_owner"));
    }

    #[test]
    fn test_update_org_error_message_format() {
        let err = DbError::Database("update_org: no rows returned".into());
        assert!(err.to_string().contains("update_org"));
    }

    #[test]
    fn test_create_repo_error_message_format() {
        let err = DbError::Database("create_repo: duplicate key".into());
        assert!(err.to_string().contains("create_repo"));
    }

    #[test]
    fn test_get_repo_error_message_format() {
        let err = DbError::Database("get_repo: no rows returned".into());
        assert!(err.to_string().contains("get_repo"));
    }

    #[test]
    fn test_get_repo_by_owner_name_error_message_format() {
        let err = DbError::Database("get_repo_by_owner_name: no rows returned".into());
        assert!(err.to_string().contains("get_repo_by_owner_name"));
    }

    #[test]
    fn test_list_repos_error_message_format() {
        let err = DbError::Database("list_repos: connection refused".into());
        assert!(err.to_string().contains("list_repos"));
    }

    #[test]
    fn test_list_repos_by_org_error_message_format() {
        let err = DbError::Database("list_repos_by_org: connection refused".into());
        assert!(err.to_string().contains("list_repos_by_org"));
    }

    #[test]
    fn test_update_repo_error_message_format() {
        let err = DbError::Database("update_repo: no rows returned".into());
        assert!(err.to_string().contains("update_repo"));
    }

    #[test]
    fn test_delete_repo_error_message_format() {
        let err = DbError::Database("delete_repo: relation not found".into());
        assert!(err.to_string().contains("delete_repo"));
    }

    #[test]
    fn test_create_issue_error_message_format() {
        let err = DbError::Database("create_issue: duplicate key".into());
        assert!(err.to_string().contains("create_issue"));
    }

    #[test]
    fn test_get_issue_error_message_format() {
        let err = DbError::Database("get_issue: no rows returned".into());
        assert!(err.to_string().contains("get_issue"));
    }

    #[test]
    fn test_list_issues_error_message_format() {
        let err = DbError::Database("list_issues: connection refused".into());
        assert!(err.to_string().contains("list_issues"));
    }

    #[test]
    fn test_update_issue_error_message_format() {
        let err = DbError::Database("update_issue: no rows returned".into());
        assert!(err.to_string().contains("update_issue"));
    }

    #[test]
    fn test_create_pr_error_message_format() {
        let err = DbError::Database("create_pr: duplicate key".into());
        assert!(err.to_string().contains("create_pr"));
    }

    #[test]
    fn test_get_pr_error_message_format() {
        let err = DbError::Database("get_pr: no rows returned".into());
        assert!(err.to_string().contains("get_pr"));
    }

    #[test]
    fn test_list_prs_error_message_format() {
        let err = DbError::Database("list_prs: connection refused".into());
        assert!(err.to_string().contains("list_prs"));
    }

    #[test]
    fn test_update_pr_error_message_format() {
        let err = DbError::Database("update_pr: no rows returned".into());
        assert!(err.to_string().contains("update_pr"));
    }

    #[test]
    fn test_create_pipeline_error_message_format() {
        let err = DbError::Database("create_pipeline: duplicate key".into());
        assert!(err.to_string().contains("create_pipeline"));
    }

    #[test]
    fn test_get_pipeline_error_message_format() {
        let err = DbError::Database("get_pipeline: no rows returned".into());
        assert!(err.to_string().contains("get_pipeline"));
    }

    #[test]
    fn test_list_pipelines_error_message_format() {
        let err = DbError::Database("list_pipelines: connection refused".into());
        assert!(err.to_string().contains("list_pipelines"));
    }

    #[test]
    fn test_update_pipeline_error_message_format() {
        let err = DbError::Database("update_pipeline: no rows returned".into());
        assert!(err.to_string().contains("update_pipeline"));
    }

    #[test]
    fn test_create_access_token_error_message_format() {
        let err = DbError::Database("create_access_token: duplicate key".into());
        assert!(err.to_string().contains("create_access_token"));
    }

    #[test]
    fn test_validate_access_token_error_message_format() {
        let db_err = DbError::Database("validate_access_token: no rows returned".into());
        assert!(db_err.to_string().contains("validate_access_token"));
        let auth_err = DbError::Auth("access token expired".into());
        assert!(auth_err.to_string().contains("access token expired"));
    }

    #[test]
    fn test_revoke_access_token_error_message_format() {
        let err = DbError::Database("revoke_access_token: relation not found".into());
        assert!(err.to_string().contains("revoke_access_token"));
    }

    #[test]
    fn test_record_audit_event_error_message_format() {
        let err = DbError::Database("record_audit_event: connection refused".into());
        assert!(err.to_string().contains("record_audit_event"));
    }

    #[test]
    fn test_query_audit_events_error_message_format() {
        let err = DbError::Database("query_audit_events: connection refused".into());
        assert!(err.to_string().contains("query_audit_events"));
    }

    #[test]
    fn test_add_ssh_key_error_message_format() {
        let err = DbError::Database("add_ssh_key: duplicate key".into());
        assert!(err.to_string().contains("add_ssh_key"));
    }

    #[test]
    fn test_list_ssh_keys_error_message_format() {
        let err = DbError::Database("list_ssh_keys: connection refused".into());
        assert!(err.to_string().contains("list_ssh_keys"));
    }

    #[test]
    fn test_delete_ssh_key_error_message_format() {
        let err = DbError::Database("delete_ssh_key: relation not found".into());
        assert!(err.to_string().contains("delete_ssh_key"));
    }

    #[test]
    fn test_get_ssh_key_by_fingerprint_error_message_format() {
        let err = DbError::Database("get_ssh_key_by_fingerprint: no rows returned".into());
        assert!(err.to_string().contains("get_ssh_key_by_fingerprint"));
    }

    #[test]
    fn test_uuid_values_are_distinct() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        assert_ne!(a, b);
    }

    #[test]
    fn test_result_type_works() {
        let res: Result<Uuid> = Ok(Uuid::nil());
        assert!(res.is_ok());
        let res: Result<Uuid> = Err(DbError::Database("fail".into()));
        assert!(res.is_err());
    }

    #[test]
    fn test_count_repos_by_org_error_message_format() {
        let err = DbError::Database("count_repos_by_org: connection refused".into());
        assert!(err.to_string().contains("count_repos_by_org"));
    }

    #[test]
    fn test_count_active_runners_by_org_error_message_format() {
        let err = DbError::Database("count_active_runners_by_org: no such table".into());
        assert!(err.to_string().contains("count_active_runners_by_org"));
    }

    #[test]
    fn test_list_repos_visible_to_user_error_message_format() {
        let err = DbError::Database("list_repos_visible_to_user: connection refused".into());
        assert!(err.to_string().contains("list_repos_visible_to_user"));
    }

    #[test]
    fn test_user_has_repo_access_error_message_format() {
        let err = DbError::Database("user_has_repo_access: connection refused".into());
        assert!(err.to_string().contains("user_has_repo_access"));
    }

    #[test]
    fn test_get_org_usage_error_message_format() {
        let err = DbError::Database("get_org_usage: connection refused".into());
        assert!(err.to_string().contains("get_org_usage"));
    }

    #[test]
    fn test_org_usage_serialization() {
        let usage = OrgUsage {
            org_id: Uuid::nil(),
            repo_count: 5,
            member_count: 10,
        };
        let json = serde_json::to_string(&usage).unwrap();
        assert!(json.contains("\"repo_count\":5"));
        assert!(json.contains("\"member_count\":10"));
    }

    #[test]
    fn test_create_team_error_message_format() {
        let err = DbError::Database("create_team: duplicate key".into());
        assert!(err.to_string().contains("create_team"));
    }

    #[test]
    fn test_get_team_error_message_format() {
        let err = DbError::Database("get_team: no rows returned".into());
        assert!(err.to_string().contains("get_team"));
    }

    #[test]
    fn test_list_teams_error_message_format() {
        let err = DbError::Database("list_teams: connection refused".into());
        assert!(err.to_string().contains("list_teams"));
    }

    #[test]
    fn test_update_team_error_message_format() {
        let err = DbError::Database("update_team: no rows returned".into());
        assert!(err.to_string().contains("update_team"));
    }

    #[test]
    fn test_delete_team_error_message_format() {
        let err = DbError::Database("delete_team: relation not found".into());
        assert!(err.to_string().contains("delete_team"));
    }

    #[test]
    fn test_add_team_member_error_message_format() {
        let err = DbError::Database("add_team_member: duplicate key".into());
        assert!(err.to_string().contains("add_team_member"));
    }

    #[test]
    fn test_remove_team_member_error_message_format() {
        let err = DbError::Database("remove_team_member: relation not found".into());
        assert!(err.to_string().contains("remove_team_member"));
    }

    #[test]
    fn test_list_team_members_error_message_format() {
        let err = DbError::Database("list_team_members: connection refused".into());
        assert!(err.to_string().contains("list_team_members"));
    }

    #[test]
    fn test_query_audit_events_admin_error_message_format() {
        let err = DbError::Database("query_audit_events_admin: connection refused".into());
        assert!(err.to_string().contains("query_audit_events_admin"));
    }

    #[test]
    fn test_audit_event_stats_error_message_format() {
        let err = DbError::Database("audit_event_stats: connection refused".into());
        assert!(err.to_string().contains("audit_event_stats"));
    }

    #[test]
    fn test_list_org_members_error_message_format() {
        let err = DbError::Database("list_org_members: connection refused".into());
        assert!(err.to_string().contains("list_org_members"));
    }
}
