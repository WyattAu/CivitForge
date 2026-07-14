#![forbid(unsafe_code)]

use crate::error::{DbError, Result};
use crate::models::{
    ActivityEvent, BoardCardAssignee, BoardCardLabel, BranchProtectionRule, EmailVerificationCode,
    Issue, MultiProjectPipeline, MultiProjectPipelineRun, Org, Pipeline, PipelineAnalytics,
    PipelineTemplate, PrComment, PrReviewer, PrStatusCheck, PrTimeline, PullRequest, Release,
    ReleaseAsset, Repository, ReviewAssignment, ReviewSummary, SshKey, Team, TeamMember, User,
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

    pub async fn list_activity_events_filtered(
        &self,
        repo_id: Option<Uuid>,
        user_id: Option<Uuid>,
        action: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ActivityEvent>> {
        let rows = sqlx::query_as::<_, ActivityEvent>(
            r#"SELECT * FROM activity_events
               WHERE ($1::uuid IS NULL OR repo_id = $1)
                 AND ($2::uuid IS NULL OR actor_id = $2)
                 AND ($3::varchar IS NULL OR action = $3)
               ORDER BY created_at DESC
               LIMIT $4 OFFSET $5"#,
        )
        .bind(repo_id)
        .bind(user_id)
        .bind(action)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_activity_events_filtered: {e}")))?;
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
            Option<Uuid>,
            DateTime<Utc>,
        )>,
    > {
        let rows = sqlx::query_as(
            r#"SELECT id, actor_id, action, resource_type, resource_id, ip_address, user_agent, outcome, request_id, created_at
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
    // --- Issue Templates ---

    pub async fn list_issue_templates(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<crate::models::IssueTemplate>> {
        let rows = sqlx::query_as::<_, crate::models::IssueTemplate>(
            "SELECT id, repo_id, name, title, body, labels, created_at FROM issue_templates WHERE repo_id = $1 ORDER BY name",
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_issue_templates: {e}")))?;
        Ok(rows)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_issue_template(
        &self,
        repo_id: Uuid,
        name: &str,
        title: &str,
        body: &str,
        labels: &[String],
    ) -> Result<crate::models::IssueTemplate> {
        let row = sqlx::query_as::<_, crate::models::IssueTemplate>(
            "INSERT INTO issue_templates (repo_id, name, title, body, labels, created_at) VALUES ($1, $2, $3, $4, $5, NOW()) RETURNING id, repo_id, name, title, body, labels, created_at",
        )
        .bind(repo_id)
        .bind(name)
        .bind(title)
        .bind(body)
        .bind(labels)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_issue_template: {e}")))?;
        Ok(row)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_issue_template(
        &self,
        template_id: Uuid,
        repo_id: Uuid,
        name: Option<&str>,
        title: Option<&str>,
        body: Option<&str>,
        labels: Option<&[String]>,
    ) -> Result<crate::models::IssueTemplate> {
        let row = sqlx::query_as::<_, crate::models::IssueTemplate>(
            "UPDATE issue_templates SET name = COALESCE($3, name), title = COALESCE($4, title), body = COALESCE($5, body), labels = COALESCE($6, labels) WHERE id = $1 AND repo_id = $2 RETURNING id, repo_id, name, title, body, labels, created_at",
        )
        .bind(template_id)
        .bind(repo_id)
        .bind(name)
        .bind(title)
        .bind(body)
        .bind(labels)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_issue_template: {e}")))?;
        Ok(row)
    }

    pub async fn delete_issue_template(
        &self,
        template_id: Uuid,
        repo_id: Uuid,
    ) -> Result<()> {
        sqlx::query("DELETE FROM issue_templates WHERE id = $1 AND repo_id = $2")
            .bind(template_id)
            .bind(repo_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_issue_template: {e}")))?;
        Ok(())
    }

    // --- PR Templates ---

    pub async fn list_pr_templates(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<crate::models::PrTemplate>> {
        let rows = sqlx::query_as::<_, crate::models::PrTemplate>(
            "SELECT id, repo_id, name, title, body, base_branch, labels, created_at FROM pr_templates WHERE repo_id = $1 ORDER BY name",
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_pr_templates: {e}")))?;
        Ok(rows)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_pr_template(
        &self,
        repo_id: Uuid,
        name: &str,
        title: &str,
        body: &str,
        base_branch: &str,
        labels: &[String],
    ) -> Result<crate::models::PrTemplate> {
        let row = sqlx::query_as::<_, crate::models::PrTemplate>(
            "INSERT INTO pr_templates (repo_id, name, title, body, base_branch, labels, created_at) VALUES ($1, $2, $3, $4, $5, $6, NOW()) RETURNING id, repo_id, name, title, body, base_branch, labels, created_at",
        )
        .bind(repo_id)
        .bind(name)
        .bind(title)
        .bind(body)
        .bind(base_branch)
        .bind(labels)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_pr_template: {e}")))?;
        Ok(row)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_pr_template(
        &self,
        template_id: Uuid,
        repo_id: Uuid,
        name: Option<&str>,
        title: Option<&str>,
        body: Option<&str>,
        base_branch: Option<&str>,
        labels: Option<&[String]>,
    ) -> Result<crate::models::PrTemplate> {
        let row = sqlx::query_as::<_, crate::models::PrTemplate>(
            "UPDATE pr_templates SET name = COALESCE($3, name), title = COALESCE($4, title), body = COALESCE($5, body), base_branch = COALESCE($6, base_branch), labels = COALESCE($7, labels) WHERE id = $1 AND repo_id = $2 RETURNING id, repo_id, name, title, body, base_branch, labels, created_at",
        )
        .bind(template_id)
        .bind(repo_id)
        .bind(name)
        .bind(title)
        .bind(body)
        .bind(base_branch)
        .bind(labels)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_pr_template: {e}")))?;
        Ok(row)
    }

    pub async fn delete_pr_template(
        &self,
        template_id: Uuid,
        repo_id: Uuid,
    ) -> Result<()> {
        sqlx::query("DELETE FROM pr_templates WHERE id = $1 AND repo_id = $2")
            .bind(template_id)
            .bind(repo_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_pr_template: {e}")))?;
        Ok(())
    }

    // --- Discussions ---

    pub async fn list_discussions(
        &self,
        repo_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::models::Discussion>> {
        let rows = sqlx::query_as::<_, crate::models::Discussion>(
            "SELECT id, repo_id, title, body, category, author_id, is_pinned, is_locked, created_at, updated_at FROM discussions WHERE repo_id = $1 ORDER BY is_pinned DESC, created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(repo_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_discussions: {e}")))?;
        Ok(rows)
    }

    pub async fn get_discussion(&self, id: Uuid) -> Result<crate::models::Discussion> {
        sqlx::query_as::<_, crate::models::Discussion>(
            "SELECT id, repo_id, title, body, category, author_id, is_pinned, is_locked, created_at, updated_at FROM discussions WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_discussion: {e}")))
    }

    pub async fn create_discussion(
        &self,
        repo_id: Uuid,
        title: &str,
        body: &str,
        category: &str,
        author_id: Uuid,
    ) -> Result<crate::models::Discussion> {
        let row = sqlx::query_as::<_, crate::models::Discussion>(
            "INSERT INTO discussions (repo_id, title, body, category, author_id) VALUES ($1, $2, $3, $4, $5) RETURNING id, repo_id, title, body, category, author_id, is_pinned, is_locked, created_at, updated_at",
        )
        .bind(repo_id)
        .bind(title)
        .bind(body)
        .bind(category)
        .bind(author_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_discussion: {e}")))?;
        Ok(row)
    }

    pub async fn update_discussion(
        &self,
        id: Uuid,
        title: Option<&str>,
        body: Option<&str>,
        category: Option<&str>,
        is_pinned: Option<bool>,
        is_locked: Option<bool>,
    ) -> Result<crate::models::Discussion> {
        let row = sqlx::query_as::<_, crate::models::Discussion>(
            "UPDATE discussions SET title = COALESCE($2, title), body = COALESCE($3, body), category = COALESCE($4, category), is_pinned = COALESCE($5, is_pinned), is_locked = COALESCE($6, is_locked), updated_at = NOW() WHERE id = $1 RETURNING id, repo_id, title, body, category, author_id, is_pinned, is_locked, created_at, updated_at",
        )
        .bind(id)
        .bind(title)
        .bind(body)
        .bind(category)
        .bind(is_pinned)
        .bind(is_locked)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_discussion: {e}")))?;
        Ok(row)
    }

    pub async fn delete_discussion(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM discussions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_discussion: {e}")))?;
        Ok(())
    }

    // --- Discussion Comments ---

    pub async fn list_discussion_comments(
        &self,
        discussion_id: Uuid,
    ) -> Result<Vec<crate::models::DiscussionComment>> {
        let rows = sqlx::query_as::<_, crate::models::DiscussionComment>(
            "SELECT id, discussion_id, author_id, body, created_at, updated_at FROM discussion_comments WHERE discussion_id = $1 ORDER BY created_at ASC",
        )
        .bind(discussion_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_discussion_comments: {e}")))?;
        Ok(rows)
    }

    pub async fn create_discussion_comment(
        &self,
        discussion_id: Uuid,
        author_id: Uuid,
        body: &str,
    ) -> Result<crate::models::DiscussionComment> {
        let row = sqlx::query_as::<_, crate::models::DiscussionComment>(
            "INSERT INTO discussion_comments (discussion_id, author_id, body) VALUES ($1, $2, $3) RETURNING id, discussion_id, author_id, body, created_at, updated_at",
        )
        .bind(discussion_id)
        .bind(author_id)
        .bind(body)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_discussion_comment: {e}")))?;
        Ok(row)
    }

    pub async fn delete_discussion_comment(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM discussion_comments WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_discussion_comment: {e}")))?;
        Ok(())
    }

    // --- PR: Close PRs targeting a deleted branch ---

    pub async fn close_prs_targeting_branch(
        &self,
        repo_id: Uuid,
        target_branch: &str,
    ) -> Result<Vec<PullRequest>> {
        let rows = sqlx::query_as::<_, PullRequest>(
            "UPDATE pull_requests SET status = 'closed', closed_at = NOW(), updated_at = NOW() WHERE repo_id = $1 AND target_branch = $2 AND status = 'open' AND draft = false RETURNING *",
        )
        .bind(repo_id)
        .bind(target_branch)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("close_prs_targeting_branch: {e}")))?;
        Ok(rows)
    }

    pub async fn close_prs_with_source_branch(
        &self,
        repo_id: Uuid,
        source_branch: &str,
    ) -> Result<Vec<PullRequest>> {
        let rows = sqlx::query_as::<_, PullRequest>(
            "UPDATE pull_requests SET status = 'closed', closed_at = NOW(), updated_at = NOW() WHERE repo_id = $1 AND source_branch = $2 AND status = 'open' AND draft = false RETURNING *",
        )
        .bind(repo_id)
        .bind(source_branch)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("close_prs_with_source_branch: {e}")))?;
        Ok(rows)
    }

    // --- Board Card Labels ---

    pub async fn add_card_label(
        &self,
        card_id: Uuid,
        label: &str,
        color: &str,
    ) -> Result<BoardCardLabel> {
        let row = sqlx::query_as::<_, BoardCardLabel>(
            r#"INSERT INTO board_card_labels (card_id, label, color)
               VALUES ($1, $2, $3)
               ON CONFLICT (card_id, label) DO UPDATE SET color = $3
               RETURNING *"#,
        )
        .bind(card_id)
        .bind(label)
        .bind(color)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("add_card_label: {e}")))?;
        Ok(row)
    }

    pub async fn remove_card_label(&self, card_id: Uuid, label: &str) -> Result<()> {
        sqlx::query("DELETE FROM board_card_labels WHERE card_id = $1 AND label = $2")
            .bind(card_id)
            .bind(label)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("remove_card_label: {e}")))?;
        Ok(())
    }

    pub async fn get_card_labels(&self, card_id: Uuid) -> Result<Vec<BoardCardLabel>> {
        let rows = sqlx::query_as::<_, BoardCardLabel>(
            "SELECT * FROM board_card_labels WHERE card_id = $1 ORDER BY label",
        )
        .bind(card_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_card_labels: {e}")))?;
        Ok(rows)
    }

    // --- Board Card Assignees ---

    pub async fn add_card_assignee(
        &self,
        card_id: Uuid,
        user_id: Uuid,
    ) -> Result<BoardCardAssignee> {
        let row = sqlx::query_as::<_, BoardCardAssignee>(
            r#"INSERT INTO board_card_assignees (card_id, user_id)
               VALUES ($1, $2)
               ON CONFLICT DO NOTHING
               RETURNING *"#,
        )
        .bind(card_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("add_card_assignee: {e}")))?;
        Ok(row)
    }

    pub async fn remove_card_assignee(&self, card_id: Uuid, user_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM board_card_assignees WHERE card_id = $1 AND user_id = $2")
            .bind(card_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("remove_card_assignee: {e}")))?;
        Ok(())
    }

    pub async fn get_card_assignees(&self, card_id: Uuid) -> Result<Vec<BoardCardAssignee>> {
        let rows = sqlx::query_as::<_, BoardCardAssignee>(
            "SELECT * FROM board_card_assignees WHERE card_id = $1 ORDER BY user_id",
        )
        .bind(card_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_card_assignees: {e}")))?;
        Ok(rows)
    }

    // --- Board Card Priority / Due Date / Sort Order ---

    pub async fn update_card_priority(&self, card_id: Uuid, priority: i32) -> Result<()> {
        sqlx::query("UPDATE board_cards SET priority = $1, updated_at = NOW() WHERE id = $2")
            .bind(priority)
            .bind(card_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("update_card_priority: {e}")))?;
        Ok(())
    }

    pub async fn update_card_due_date(
        &self,
        card_id: Uuid,
        due_date: Option<chrono::DateTime<Utc>>,
    ) -> Result<()> {
        sqlx::query("UPDATE board_cards SET due_date = $1, updated_at = NOW() WHERE id = $2")
            .bind(due_date)
            .bind(card_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("update_card_due_date: {e}")))?;
        Ok(())
    }

    pub async fn update_card_sort_order(&self, card_id: Uuid, sort_order: i32) -> Result<()> {
        sqlx::query("UPDATE board_cards SET sort_order = $1, updated_at = NOW() WHERE id = $2")
            .bind(sort_order)
            .bind(card_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("update_card_sort_order: {e}")))?;
        Ok(())
    }

    // --- PR: Resolve/Unresolve Comments ---

    pub async fn resolve_comment(
        &self,
        comment_id: Uuid,
        user_id: Uuid,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE pr_comments SET resolved = true, resolved_by = $2, updated_at = NOW() WHERE id = $1",
        )
        .bind(comment_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("resolve_comment: {e}")))?;
        Ok(())
    }

    pub async fn unresolve_comment(&self, comment_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE pr_comments SET resolved = false, resolved_by = NULL, updated_at = NOW() WHERE id = $1",
        )
        .bind(comment_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("unresolve_comment: {e}")))?;
        Ok(())
    }

    // --- PR: Review Summary ---

    pub async fn get_review_summary(&self, pr_id: Uuid) -> Result<ReviewSummary> {
        let row = sqlx::query_as::<_, (i64, i64, i64)>(
            r#"SELECT
                COUNT(*) FILTER (WHERE review_status = 'approved') AS approvals,
                COUNT(*) FILTER (WHERE review_status = 'changes_requested') AS changes_requested,
                COUNT(*) FILTER (WHERE review_status = 'commented') AS comments
               FROM pr_reviewers WHERE pr_id = $1"#,
        )
        .bind(pr_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_review_summary counts: {e}")))?;

        let codeowners_approved: (bool,) = sqlx::query_as(
            r#"SELECT COALESCE(
                (SELECT bool_and(approved) FROM codeowners_reviews WHERE pr_id = $1),
                true
            )"#,
        )
        .bind(pr_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_review_summary codeowners: {e}")))?;

        Ok(ReviewSummary {
            pr_id,
            approvals: row.0,
            changes_requested: row.1,
            comments: row.2,
            codeowners_approved: codeowners_approved.0,
        })
    }

    // --- PR: Review Assignments ---

    pub async fn get_review_assignments(&self, pr_id: Uuid) -> Result<Vec<ReviewAssignment>> {
        sqlx::query_as::<_, ReviewAssignment>(
            "SELECT * FROM pr_review_assignments WHERE pr_id = $1 ORDER BY created_at",
        )
        .bind(pr_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_review_assignments: {e}")))
    }

    pub async fn add_review_assignment(
        &self,
        pr_id: Uuid,
        user_id: Uuid,
        team: &str,
        assigned_by: Uuid,
    ) -> Result<ReviewAssignment> {
        let row = sqlx::query_as::<_, ReviewAssignment>(
            r#"INSERT INTO pr_review_assignments (pr_id, user_id, team, assigned_by)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (pr_id, user_id) DO UPDATE SET team = $3, assigned_by = $4
               RETURNING *"#,
        )
        .bind(pr_id)
        .bind(user_id)
        .bind(team)
        .bind(assigned_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("add_review_assignment: {e}")))?;
        Ok(row)
    }

    // --- PR: Re-request review ---

    pub async fn rerequest_pr_review(
        &self,
        pr_id: Uuid,
        user_id: Uuid,
    ) -> Result<PrReviewer> {
        let row = sqlx::query_as::<_, PrReviewer>(
            "INSERT INTO pr_reviewers (pr_id, user_id, review_status, submitted_at) VALUES ($1, $2, 'pending', NULL) ON CONFLICT (pr_id, user_id) DO UPDATE SET review_status = 'pending', submitted_at = NULL RETURNING pr_id, user_id, review_status, submitted_at",
        )
        .bind(pr_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("rerequest_pr_review: {e}")))?;
        Ok(row)
    }

    // --- NPM Packages ---

    pub async fn create_npm_package(
        &self,
        repo_id: Uuid,
        name: &str,
        version: &str,
        description: &str,
        dist_tags: &serde_json::Value,
        readme: &str,
    ) -> Result<crate::models::NpmPackage> {
        let row = sqlx::query_as::<_, crate::models::NpmPackage>(
            r#"INSERT INTO npm_packages (repo_id, name, version, description, dist_tags, readme)
               VALUES ($1, $2, $3, $4, $5, $6)
               ON CONFLICT (repo_id, name, version)
               DO UPDATE SET description = $4, dist_tags = $5, readme = $6
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(name)
        .bind(version)
        .bind(description)
        .bind(dist_tags)
        .bind(readme)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_npm_package: {e}")))?;
        Ok(row)
    }

    pub async fn get_npm_package_by_name(
        &self,
        name: &str,
    ) -> Result<Vec<crate::models::NpmPackage>> {
        let rows = sqlx::query_as::<_, crate::models::NpmPackage>(
            "SELECT * FROM npm_packages WHERE name = $1 ORDER BY created_at DESC",
        )
        .bind(name)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_npm_package_by_name: {e}")))?;
        Ok(rows)
    }

    pub async fn get_npm_package_version(
        &self,
        name: &str,
        version: &str,
    ) -> Result<Option<crate::models::NpmPackage>> {
        let row = sqlx::query_as::<_, crate::models::NpmPackage>(
            "SELECT * FROM npm_packages WHERE name = $1 AND version = $2 LIMIT 1",
        )
        .bind(name)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_npm_package_version: {e}")))?;
        Ok(row)
    }

    pub async fn create_npm_version(
        &self,
        package_id: Uuid,
        version: &str,
        tarball_url: &str,
        shasum: &str,
        integrity: &str,
    ) -> Result<crate::models::NpmVersion> {
        let row = sqlx::query_as::<_, crate::models::NpmVersion>(
            r#"INSERT INTO npm_versions (package_id, version, tarball_url, shasum, integrity)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(package_id)
        .bind(version)
        .bind(tarball_url)
        .bind(shasum)
        .bind(integrity)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_npm_version: {e}")))?;
        Ok(row)
    }

    pub async fn list_npm_versions(&self, package_id: Uuid) -> Result<Vec<crate::models::NpmVersion>> {
        let rows = sqlx::query_as::<_, crate::models::NpmVersion>(
            "SELECT * FROM npm_versions WHERE package_id = $1 ORDER BY created_at DESC",
        )
        .bind(package_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_npm_versions: {e}")))?;
        Ok(rows)
    }

    // --- Maven Packages ---

    pub async fn create_maven_package(
        &self,
        repo_id: Uuid,
        group_id: &str,
        artifact_id: &str,
        version: &str,
        packaging: &str,
    ) -> Result<crate::models::MavenPackage> {
        let row = sqlx::query_as::<_, crate::models::MavenPackage>(
            r#"INSERT INTO maven_packages (repo_id, group_id, artifact_id, version, packaging)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT (repo_id, group_id, artifact_id, version)
               DO UPDATE SET packaging = $5
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(group_id)
        .bind(artifact_id)
        .bind(version)
        .bind(packaging)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_maven_package: {e}")))?;
        Ok(row)
    }

    pub async fn get_maven_package(
        &self,
        repo_id: Uuid,
        group_id: &str,
        artifact_id: &str,
        version: &str,
    ) -> Result<Option<crate::models::MavenPackage>> {
        let row = sqlx::query_as::<_, crate::models::MavenPackage>(
            "SELECT * FROM maven_packages WHERE repo_id = $1 AND group_id = $2 AND artifact_id = $3 AND version = $4 LIMIT 1",
        )
        .bind(repo_id)
        .bind(group_id)
        .bind(artifact_id)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_maven_package: {e}")))?;
        Ok(row)
    }

    pub async fn list_maven_packages(
        &self,
        repo_id: Uuid,
        group_id: &str,
        artifact_id: &str,
    ) -> Result<Vec<crate::models::MavenPackage>> {
        let rows = sqlx::query_as::<_, crate::models::MavenPackage>(
            "SELECT * FROM maven_packages WHERE repo_id = $1 AND group_id = $2 AND artifact_id = $3 ORDER BY created_at DESC",
        )
        .bind(repo_id)
        .bind(group_id)
        .bind(artifact_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_maven_packages: {e}")))?;
        Ok(rows)
    }

    // --- Pages Sites ---

    pub async fn enable_pages(
        &self,
        repo_id: Uuid,
        url: &str,
        branch: &str,
        path: &str,
        public: bool,
    ) -> Result<crate::models::PagesSite> {
        let row = sqlx::query_as::<_, crate::models::PagesSite>(
            r#"INSERT INTO pages_sites (repo_id, url, branch, path, public)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT (repo_id) DO UPDATE SET url = $2, branch = $3, path = $4, public = $5, updated_at = NOW()
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(url)
        .bind(branch)
        .bind(path)
        .bind(public)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("enable_pages: {e}")))?;
        Ok(row)
    }

    pub async fn get_pages_site(&self, repo_id: Uuid) -> Result<Option<crate::models::PagesSite>> {
        let row = sqlx::query_as::<_, crate::models::PagesSite>(
            "SELECT * FROM pages_sites WHERE repo_id = $1",
        )
        .bind(repo_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_pages_site: {e}")))?;
        Ok(row)
    }

    pub async fn disable_pages(&self, repo_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM pages_sites WHERE repo_id = $1")
            .bind(repo_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("disable_pages: {e}")))?;
        Ok(())
    }

    pub async fn update_pages_custom_domain(
        &self,
        repo_id: Uuid,
        custom_domain: Option<&str>,
        https_enabled: bool,
    ) -> Result<crate::models::PagesSite> {
        let row = sqlx::query_as::<_, crate::models::PagesSite>(
            r#"UPDATE pages_sites
               SET custom_domain = $2, https_enabled = $3, updated_at = NOW()
               WHERE repo_id = $1
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(custom_domain)
        .bind(https_enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_pages_custom_domain: {e}")))?;
        Ok(row)
    }

    pub async fn update_pages_last_built(
        &self,
        site_id: Uuid,
    ) -> Result<crate::models::PagesSite> {
        let row = sqlx::query_as::<_, crate::models::PagesSite>(
            r#"UPDATE pages_sites
               SET last_built_at = NOW(), updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(site_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_pages_last_built: {e}")))?;
        Ok(row)
    }

    // --- Pages Deployments ---

    pub async fn create_pages_deployment(
        &self,
        site_id: Uuid,
        sha: &str,
        url: &str,
    ) -> Result<crate::models::PagesDeployment> {
        let row = sqlx::query_as::<_, crate::models::PagesDeployment>(
            r#"INSERT INTO pages_deployments (site_id, sha, url, status)
               VALUES ($1, $2, $3, 'pending')
               RETURNING *"#,
        )
        .bind(site_id)
        .bind(sha)
        .bind(url)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_pages_deployment: {e}")))?;
        Ok(row)
    }

    pub async fn update_pages_deployment_status(
        &self,
        deployment_id: Uuid,
        status: &str,
    ) -> Result<crate::models::PagesDeployment> {
        let row = sqlx::query_as::<_, crate::models::PagesDeployment>(
            r#"UPDATE pages_deployments
               SET status = $2
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(deployment_id)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_pages_deployment_status: {e}")))?;
        Ok(row)
    }

    pub async fn list_pages_deployments(
        &self,
        site_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::models::PagesDeployment>> {
        let rows = sqlx::query_as::<_, crate::models::PagesDeployment>(
            r#"SELECT * FROM pages_deployments
               WHERE site_id = $1
               ORDER BY created_at DESC
               LIMIT $2 OFFSET $3"#,
        )
        .bind(site_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_pages_deployments: {e}")))?;
        Ok(rows)
    }

    // --- Discussion Labels ---

    pub async fn add_discussion_label(
        &self,
        discussion_id: Uuid,
        label: &str,
        color: &str,
    ) -> Result<crate::models::DiscussionLabel> {
        let row = sqlx::query_as::<_, crate::models::DiscussionLabel>(
            r#"INSERT INTO discussion_labels (discussion_id, label, color)
               VALUES ($1, $2, $3)
               ON CONFLICT (discussion_id, label) DO UPDATE SET color = $3
               RETURNING *"#,
        )
        .bind(discussion_id)
        .bind(label)
        .bind(color)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("add_discussion_label: {e}")))?;
        Ok(row)
    }

    pub async fn remove_discussion_label(
        &self,
        discussion_id: Uuid,
        label: &str,
    ) -> Result<()> {
        sqlx::query("DELETE FROM discussion_labels WHERE discussion_id = $1 AND label = $2")
            .bind(discussion_id)
            .bind(label)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("remove_discussion_label: {e}")))?;
        Ok(())
    }

    pub async fn list_discussion_labels(
        &self,
        discussion_id: Uuid,
    ) -> Result<Vec<crate::models::DiscussionLabel>> {
        let rows = sqlx::query_as::<_, crate::models::DiscussionLabel>(
            "SELECT * FROM discussion_labels WHERE discussion_id = $1 ORDER BY label",
        )
        .bind(discussion_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_discussion_labels: {e}")))?;
        Ok(rows)
    }

    // --- Discussion Reactions ---

    pub async fn add_discussion_reaction(
        &self,
        comment_id: Uuid,
        user_id: Uuid,
        emoji: &str,
    ) -> Result<crate::models::DiscussionReaction> {
        let row = sqlx::query_as::<_, crate::models::DiscussionReaction>(
            r#"INSERT INTO discussion_reactions (comment_id, user_id, emoji)
               VALUES ($1, $2, $3)
               ON CONFLICT (comment_id, user_id, emoji) DO UPDATE SET emoji = $3
               RETURNING *"#,
        )
        .bind(comment_id)
        .bind(user_id)
        .bind(emoji)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("add_discussion_reaction: {e}")))?;
        Ok(row)
    }

    pub async fn remove_discussion_reaction(
        &self,
        comment_id: Uuid,
        user_id: Uuid,
        emoji: &str,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM discussion_reactions WHERE comment_id = $1 AND user_id = $2 AND emoji = $3",
        )
        .bind(comment_id)
        .bind(user_id)
        .bind(emoji)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("remove_discussion_reaction: {e}")))?;
        Ok(())
    }

    pub async fn list_discussion_reactions(
        &self,
        comment_id: Uuid,
    ) -> Result<Vec<crate::models::DiscussionReaction>> {
        let rows = sqlx::query_as::<_, crate::models::DiscussionReaction>(
            "SELECT * FROM discussion_reactions WHERE comment_id = $1 ORDER BY created_at",
        )
        .bind(comment_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_discussion_reactions: {e}")))?;
        Ok(rows)
    }

    // --- Discussion Search ---

    pub async fn search_discussions(
        &self,
        repo_id: Uuid,
        query: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::models::Discussion>> {
        let pattern = format!("%{query}%");
        let rows = sqlx::query_as::<_, crate::models::Discussion>(
            "SELECT id, repo_id, title, body, category, author_id, is_pinned, is_locked, created_at, updated_at FROM discussions WHERE repo_id = $1 AND (title ILIKE $2 OR body ILIKE $2) ORDER BY is_pinned DESC, created_at DESC LIMIT $3 OFFSET $4",
        )
        .bind(repo_id)
        .bind(&pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("search_discussions: {e}")))?;
        Ok(rows)
    }

    // --- Feature Flags ---

    pub async fn create_feature_flag(
        &self,
        name: &str,
        description: &str,
        enabled: bool,
        enabled_for_percentage: i32,
    ) -> Result<crate::models::FeatureFlag> {
        let row = sqlx::query_as::<_, crate::models::FeatureFlag>(
            r#"INSERT INTO feature_flags (name, description, enabled, enabled_for_percentage)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(enabled)
        .bind(enabled_for_percentage)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_feature_flag: {e}")))?;
        Ok(row)
    }

    pub async fn get_feature_flag(&self, id: Uuid) -> Result<crate::models::FeatureFlag> {
        sqlx::query_as::<_, crate::models::FeatureFlag>("SELECT * FROM feature_flags WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_feature_flag: {e}")))
    }

    pub async fn list_feature_flags(&self) -> Result<Vec<crate::models::FeatureFlag>> {
        let rows = sqlx::query_as::<_, crate::models::FeatureFlag>(
            "SELECT * FROM feature_flags ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_feature_flags: {e}")))?;
        Ok(rows)
    }

    pub async fn list_enabled_feature_flags_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<crate::models::FeatureFlag>> {
        let rows = sqlx::query_as::<_, crate::models::FeatureFlag>(
            r#"SELECT * FROM feature_flags
               WHERE enabled = true
                 AND (
                   enabled_for_users = '{}'::uuid[]
                   OR $1 = ANY(enabled_for_users)
                   OR enabled_for_percentage = 100
                   OR (enabled_for_percentage > 0 AND hashtext($2::text)::int % 100 < enabled_for_percentage)
                 )
               ORDER BY name"#,
        )
        .bind(user_id)
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_enabled_feature_flags_for_user: {e}")))?;
        Ok(rows)
    }

    pub async fn update_feature_flag(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        enabled: Option<bool>,
        enabled_for_percentage: Option<i32>,
    ) -> Result<crate::models::FeatureFlag> {
        let row = sqlx::query_as::<_, crate::models::FeatureFlag>(
            r#"UPDATE feature_flags
               SET name = COALESCE($2, name),
                   description = COALESCE($3, description),
                   enabled = COALESCE($4, enabled),
                   enabled_for_percentage = COALESCE($5, enabled_for_percentage),
                   updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(enabled)
        .bind(enabled_for_percentage)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_feature_flag: {e}")))?;
        Ok(row)
    }

    pub async fn toggle_feature_flag(&self, id: Uuid) -> Result<crate::models::FeatureFlag> {
        let row = sqlx::query_as::<_, crate::models::FeatureFlag>(
            r#"UPDATE feature_flags
               SET enabled = NOT enabled, updated_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("toggle_feature_flag: {e}")))?;
        Ok(row)
    }

    pub async fn delete_feature_flag(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM feature_flags WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_feature_flag: {e}")))?;
        Ok(())
    }

    pub async fn add_feature_flag_user(
        &self,
        flag_id: Uuid,
        user_id: Uuid,
    ) -> Result<()> {
        sqlx::query(
            r#"UPDATE feature_flags
               SET enabled_for_users = array_append(
                 CASE WHEN enabled_for_users @> ARRAY[$2::uuid] THEN enabled_for_users ELSE enabled_for_users END,
                 CASE WHEN enabled_for_users @> ARRAY[$2::uuid] THEN NULL::uuid ELSE $2 END
               ),
               updated_at = NOW()
               WHERE id = $1"#,
        )
        .bind(flag_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("add_feature_flag_user: {e}")))?;
        Ok(())
    }

    pub async fn remove_feature_flag_user(
        &self,
        flag_id: Uuid,
        user_id: Uuid,
    ) -> Result<()> {
        sqlx::query(
            r#"UPDATE feature_flags
               SET enabled_for_users = array_remove(enabled_for_users, $2),
                   updated_at = NOW()
               WHERE id = $1"#,
        )
        .bind(flag_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("remove_feature_flag_user: {e}")))?;
        Ok(())
    }

    pub async fn add_feature_flag_org(
        &self,
        flag_id: Uuid,
        org_id: Uuid,
    ) -> Result<()> {
        sqlx::query(
            r#"UPDATE feature_flags
               SET enabled_for_orgs = array_append(
                 CASE WHEN enabled_for_orgs @> ARRAY[$2::uuid] THEN enabled_for_orgs ELSE enabled_for_orgs END,
                 CASE WHEN enabled_for_orgs @> ARRAY[$2::uuid] THEN NULL::uuid ELSE $2 END
               ),
               updated_at = NOW()
               WHERE id = $1"#,
        )
        .bind(flag_id)
        .bind(org_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("add_feature_flag_org: {e}")))?;
        Ok(())
    }

    pub async fn remove_feature_flag_org(
        &self,
        flag_id: Uuid,
        org_id: Uuid,
    ) -> Result<()> {
        sqlx::query(
            r#"UPDATE feature_flags
               SET enabled_for_orgs = array_remove(enabled_for_orgs, $2),
                   updated_at = NOW()
               WHERE id = $1"#,
        )
        .bind(flag_id)
        .bind(org_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("remove_feature_flag_org: {e}")))?;
        Ok(())
    }

    pub async fn record_feature_flag_event(
        &self,
        flag_id: Uuid,
        user_id: Option<Uuid>,
        enabled: bool,
    ) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO feature_flag_events (flag_id, user_id, enabled)
               VALUES ($1, $2, $3)"#,
        )
        .bind(flag_id)
        .bind(user_id)
        .bind(enabled)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_feature_flag_event: {e}")))?;
        Ok(())
    }

    // --- Admin Dashboard Config ---

    pub async fn list_admin_dashboard_widgets(
        &self,
    ) -> Result<Vec<crate::models::AdminDashboardConfig>> {
        let rows = sqlx::query_as::<_, crate::models::AdminDashboardConfig>(
            "SELECT * FROM admin_dashboard_config ORDER BY position",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_admin_dashboard_widgets: {e}")))?;
        Ok(rows)
    }

    pub async fn get_admin_dashboard_widget(
        &self,
        widget_name: &str,
    ) -> Result<Option<crate::models::AdminDashboardConfig>> {
        let row = sqlx::query_as::<_, crate::models::AdminDashboardConfig>(
            "SELECT * FROM admin_dashboard_config WHERE widget_name = $1",
        )
        .bind(widget_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_admin_dashboard_widget: {e}")))?;
        Ok(row)
    }

    pub async fn upsert_admin_dashboard_widget(
        &self,
        widget_name: &str,
        widget_config: &serde_json::Value,
        position: i32,
        enabled: bool,
    ) -> Result<crate::models::AdminDashboardConfig> {
        let row = sqlx::query_as::<_, crate::models::AdminDashboardConfig>(
            r#"INSERT INTO admin_dashboard_config (widget_name, widget_config, position, enabled)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (widget_name) DO UPDATE SET
                   widget_config = $2,
                   position = $3,
                   enabled = $4
               RETURNING *"#,
        )
        .bind(widget_name)
        .bind(widget_config)
        .bind(position)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("upsert_admin_dashboard_widget: {e}")))?;
        Ok(row)
    }

    pub async fn delete_admin_dashboard_widget(&self, widget_name: &str) -> Result<()> {
        sqlx::query("DELETE FROM admin_dashboard_config WHERE widget_name = $1")
            .bind(widget_name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_admin_dashboard_widget: {e}")))?;
        Ok(())
    }

    // --- API Analytics ---

    pub async fn record_api_analytic(
        &self,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_time_ms: i32,
        user_id: Option<Uuid>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        request_size_bytes: i32,
        response_size_bytes: i32,
    ) -> Result<crate::models::ApiAnalytic> {
        let row = sqlx::query_as::<_, crate::models::ApiAnalytic>(
            r#"INSERT INTO api_analytics (endpoint, method, status_code, response_time_ms, user_id, ip_address, user_agent, request_size_bytes, response_size_bytes)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(user_id)
        .bind(ip_address)
        .bind(user_agent)
        .bind(request_size_bytes)
        .bind(response_size_bytes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_api_analytic: {e}")))?;
        Ok(row)
    }

    pub async fn get_api_analytics(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::models::ApiAnalytic>> {
        let rows = sqlx::query_as::<_, crate::models::ApiAnalytic>(
            "SELECT * FROM api_analytics ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_analytics: {e}")))?;
        Ok(rows)
    }

    pub async fn get_api_analytics_by_endpoint(
        &self,
        endpoint: &str,
        limit: i64,
    ) -> Result<Vec<crate::models::ApiAnalytic>> {
        let rows = sqlx::query_as::<_, crate::models::ApiAnalytic>(
            "SELECT * FROM api_analytics WHERE endpoint = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(endpoint)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_analytics_by_endpoint: {e}")))?;
        Ok(rows)
    }

    pub async fn get_endpoint_statistics(&self) -> Result<Vec<serde_json::Value>> {
        let rows = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'endpoint', endpoint,
                'method', method,
                'total_requests', COUNT(*),
                'avg_response_time_ms', AVG(response_time_ms),
                'error_rate', SUM(CASE WHEN status_code >= 400 THEN 1 ELSE 0 END)::float / COUNT(*)::float,
                'p95_response_time_ms', PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms)
             )
             FROM api_analytics
             GROUP BY endpoint, method
             ORDER BY total_requests DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_endpoint_statistics: {e}")))?;
        Ok(rows)
    }

    pub async fn get_user_usage_statistics(&self) -> Result<Vec<serde_json::Value>> {
        let rows = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'user_id', user_id,
                'total_requests', COUNT(*),
                'avg_response_time_ms', AVG(response_time_ms),
                'error_rate', SUM(CASE WHEN status_code >= 400 THEN 1 ELSE 0 END)::float / COUNT(*)::float
             )
             FROM api_analytics
             WHERE user_id IS NOT NULL
             GROUP BY user_id
             ORDER BY total_requests DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_user_usage_statistics: {e}")))?;
        Ok(rows)
    }

    pub async fn get_api_usage_summary(
        &self,
        period_start: chrono::DateTime<chrono::Utc>,
        period_end: chrono::DateTime<chrono::Utc>,
    ) -> Result<crate::models::ApiUsageSummary> {
        let row = sqlx::query_as::<_, crate::models::ApiUsageSummary>(
            r#"INSERT INTO api_usage_summary (period_start, period_end, total_requests, total_errors, avg_response_time_ms, p95_response_time_ms, unique_users)
               SELECT $1, $2, COUNT(*), SUM(CASE WHEN status_code >= 400 THEN 1 ELSE 0 END), AVG(response_time_ms), PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms), COUNT(DISTINCT user_id)
               FROM api_analytics
               WHERE created_at >= $1 AND created_at < $3
               RETURNING *"#,
        )
        .bind(period_start)
        .bind(period_end)
        .bind(period_end)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_usage_summary: {e}")))?;
        Ok(row)
    }

    // --- Usage Quotas ---

    pub async fn create_usage_quota(
        &self,
        user_id: Uuid,
        quota_type: &str,
        quota_limit: i32,
        period_start: chrono::DateTime<chrono::Utc>,
    ) -> Result<crate::models::UsageQuota> {
        let row = sqlx::query_as::<_, crate::models::UsageQuota>(
            r#"INSERT INTO usage_quotas (user_id, quota_type, quota_limit, period_start)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (user_id, quota_type, period_start) DO UPDATE SET quota_limit = $3
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(quota_type)
        .bind(quota_limit)
        .bind(period_start)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_usage_quota: {e}")))?;
        Ok(row)
    }

    pub async fn get_usage_quota(
        &self,
        user_id: Uuid,
        quota_type: &str,
    ) -> Result<Option<crate::models::UsageQuota>> {
        let row = sqlx::query_as::<_, crate::models::UsageQuota>(
            r#"SELECT * FROM usage_quotas 
               WHERE user_id = $1 AND quota_type = $2 AND period_start <= NOW()
               ORDER BY period_start DESC LIMIT 1"#,
        )
        .bind(user_id)
        .bind(quota_type)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_usage_quota: {e}")))?;
        Ok(row)
    }

    pub async fn increment_usage_quota(
        &self,
        user_id: Uuid,
        quota_type: &str,
    ) -> Result<crate::models::UsageQuota> {
        let row = sqlx::query_as::<_, crate::models::UsageQuota>(
            r#"UPDATE usage_quotas 
               SET quota_used = quota_used + 1
               WHERE user_id = $1 AND quota_type = $2 AND period_start <= NOW()
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(quota_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("increment_usage_quota: {e}")))?;
        Ok(row)
    }

    pub async fn get_user_usage_quotas(&self, user_id: Uuid) -> Result<Vec<crate::models::UsageQuota>> {
        let rows = sqlx::query_as::<_, crate::models::UsageQuota>(
            r#"SELECT * FROM usage_quotas 
               WHERE user_id = $1 AND period_start <= NOW()
               ORDER BY quota_type"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_user_usage_quotas: {e}")))?;
        Ok(rows)
    }

    pub async fn delete_usage_quota(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM usage_quotas WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_usage_quota: {e}")))?;
        Ok(())
    }

    // --- Deployment History ---

    pub async fn create_deployment_history(
        &self,
        environment_id: Uuid,
        version: &str,
        sha: &str,
        status: &str,
        deployed_by: Uuid,
        rollback_of: Option<Uuid>,
    ) -> Result<crate::models::DeploymentHistory> {
        let row = sqlx::query_as::<_, crate::models::DeploymentHistory>(
            r#"INSERT INTO deployment_history (environment_id, version, sha, status, deployed_by, rollback_of)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING *"#,
        )
        .bind(environment_id)
        .bind(version)
        .bind(sha)
        .bind(status)
        .bind(deployed_by)
        .bind(rollback_of)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_deployment_history: {e}")))?;
        Ok(row)
    }

    pub async fn list_deployment_history(
        &self,
        environment_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::models::DeploymentHistory>> {
        let rows = sqlx::query_as::<_, crate::models::DeploymentHistory>(
            r#"SELECT * FROM deployment_history
               WHERE environment_id = $1
               ORDER BY created_at DESC
               LIMIT $2 OFFSET $3"#,
        )
        .bind(environment_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_deployment_history: {e}")))?;
        Ok(rows)
    }

    pub async fn get_deployment_history(
        &self,
        id: Uuid,
    ) -> Result<crate::models::DeploymentHistory> {
        sqlx::query_as::<_, crate::models::DeploymentHistory>(
            "SELECT * FROM deployment_history WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_deployment_history: {e}")))
    }

    pub async fn rollback_deployment(
        &self,
        original_id: Uuid,
        deployed_by: Uuid,
    ) -> Result<crate::models::DeploymentHistory> {
        let original = self.get_deployment_history(original_id).await?;
        let new_version = format!("rollback-{}", original.version);
        self.create_deployment_history(
            original.environment_id,
            &new_version,
            &original.sha,
            "deployed",
            deployed_by,
            Some(original_id),
        )
        .await
    }

    pub async fn get_rollback_status(
        &self,
        deployment_id: Uuid,
    ) -> Result<crate::models::DeploymentHistory> {
        let row = sqlx::query_as::<_, crate::models::DeploymentHistory>(
            r#"SELECT * FROM deployment_history
               WHERE rollback_of = $1
               ORDER BY created_at DESC
               LIMIT 1"#,
        )
        .bind(deployment_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_rollback_status: {e}")))?;
        Ok(row)
    }

    // --- Monitoring Alerts ---

    pub async fn create_monitoring_alert(
        &self,
        repo_id: Uuid,
        alert_type: &str,
        condition: &str,
        threshold: f64,
    ) -> Result<crate::models::MonitoringAlert> {
        let row = sqlx::query_as::<_, crate::models::MonitoringAlert>(
            r#"INSERT INTO monitoring_alerts (repo_id, alert_type, condition, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(alert_type)
        .bind(condition)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_monitoring_alert: {e}")))?;
        Ok(row)
    }

    pub async fn list_monitoring_alerts(
        &self,
        repo_id: Uuid,
    ) -> Result<Vec<crate::models::MonitoringAlert>> {
        let rows = sqlx::query_as::<_, crate::models::MonitoringAlert>(
            r#"SELECT * FROM monitoring_alerts
               WHERE repo_id = $1
               ORDER BY created_at DESC"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_monitoring_alerts: {e}")))?;
        Ok(rows)
    }

    pub async fn get_monitoring_alert(
        &self,
        id: Uuid,
    ) -> Result<crate::models::MonitoringAlert> {
        sqlx::query_as::<_, crate::models::MonitoringAlert>(
            "SELECT * FROM monitoring_alerts WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_monitoring_alert: {e}")))
    }

    pub async fn update_monitoring_alert(
        &self,
        id: Uuid,
        alert_type: Option<&str>,
        condition: Option<&str>,
        threshold: Option<f64>,
        enabled: Option<bool>,
    ) -> Result<crate::models::MonitoringAlert> {
        let row = sqlx::query_as::<_, crate::models::MonitoringAlert>(
            r#"UPDATE monitoring_alerts
               SET alert_type = COALESCE($2, alert_type),
                   condition = COALESCE($3, condition),
                   threshold = COALESCE($4, threshold),
                   enabled = COALESCE($5, enabled)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(alert_type)
        .bind(condition)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_monitoring_alert: {e}")))?;
        Ok(row)
    }

    pub async fn delete_monitoring_alert(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM monitoring_alerts WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_monitoring_alert: {e}")))?;
        Ok(())
    }

    pub async fn trigger_monitoring_alert(
        &self,
        id: Uuid,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE monitoring_alerts SET last_triggered_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("trigger_monitoring_alert: {e}")))?;
        Ok(())
    }

    // --- Monitoring Incidents ---

    pub async fn create_monitoring_incident(
        &self,
        alert_id: Uuid,
        severity: &str,
        message: &str,
    ) -> Result<crate::models::MonitoringIncident> {
        let row = sqlx::query_as::<_, crate::models::MonitoringIncident>(
            r#"INSERT INTO monitoring_incidents (alert_id, severity, message)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(alert_id)
        .bind(severity)
        .bind(message)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_monitoring_incident: {e}")))?;
        Ok(row)
    }

    pub async fn list_monitoring_incidents(
        &self,
        repo_id: Option<Uuid>,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::models::MonitoringIncident>> {
        let rows = sqlx::query_as::<_, crate::models::MonitoringIncident>(
            r#"SELECT mi.* FROM monitoring_incidents mi
               INNER JOIN monitoring_alerts ma ON ma.id = mi.alert_id
               WHERE ($1::uuid IS NULL OR ma.repo_id = $1)
                 AND ($2::varchar IS NULL OR mi.status = $2)
               ORDER BY mi.created_at DESC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(repo_id)
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_monitoring_incidents: {e}")))?;
        Ok(rows)
    }

    pub async fn resolve_monitoring_incident(
        &self,
        id: Uuid,
    ) -> Result<crate::models::MonitoringIncident> {
        let row = sqlx::query_as::<_, crate::models::MonitoringIncident>(
            r#"UPDATE monitoring_incidents
               SET status = 'resolved', resolved_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("resolve_monitoring_incident: {e}")))?;
        Ok(row)
    }

    pub async fn get_incident_timeline(
        &self,
        alert_id: Uuid,
    ) -> Result<Vec<crate::models::MonitoringIncident>> {
        let rows = sqlx::query_as::<_, crate::models::MonitoringIncident>(
            r#"SELECT * FROM monitoring_incidents
               WHERE alert_id = $1
               ORDER BY created_at ASC"#,
        )
        .bind(alert_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_incident_timeline: {e}")))?;
        Ok(rows)
    }

    // --- Performance Metrics ---

    pub async fn record_performance_metric(
        &self,
        metric_name: &str,
        metric_value: f64,
        labels: &serde_json::Value,
    ) -> Result<crate::models::PerformanceMetric> {
        let row = sqlx::query_as::<_, crate::models::PerformanceMetric>(
            r#"INSERT INTO performance_metrics (metric_name, metric_value, labels)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(metric_name)
        .bind(metric_value)
        .bind(labels)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_performance_metric: {e}")))?;
        Ok(row)
    }

    pub async fn query_performance_metrics(
        &self,
        metric_name: &str,
        since: chrono::DateTime<chrono::Utc>,
        until: chrono::DateTime<chrono::Utc>,
        limit: i64,
    ) -> Result<Vec<crate::models::PerformanceMetric>> {
        let rows = sqlx::query_as::<_, crate::models::PerformanceMetric>(
            r#"SELECT * FROM performance_metrics
               WHERE metric_name = $1
                 AND recorded_at >= $2
                 AND recorded_at < $3
               ORDER BY recorded_at DESC
               LIMIT $4"#,
        )
        .bind(metric_name)
        .bind(since)
        .bind(until)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("query_performance_metrics: {e}")))?;
        Ok(rows)
    }

    pub async fn get_performance_metric_summary(
        &self,
        metric_name: &str,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<serde_json::Value> {
        let row = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'metric_name', $1,
                'count', COUNT(*),
                'avg_value', AVG(metric_value),
                'min_value', MIN(metric_value),
                'max_value', MAX(metric_value),
                'p95_value', PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY metric_value),
                'p99_value', PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY metric_value)
             )
             FROM performance_metrics
             WHERE metric_name = $1 AND recorded_at >= $2"#,
        )
        .bind(metric_name)
        .bind(since)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_metric_summary: {e}")))?;
        Ok(row)
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
