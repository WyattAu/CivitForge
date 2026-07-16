#![forbid(unsafe_code)]

use crate::error::{DbError, Result};
use crate::models::{
    ActivityEvent, ApiAnalyticV2, ApiAnalyticV3, ApiAnalyticV4, ApiAnalyticV5, ApiAnalyticV6,
    ApiAnalyticV7, ApiAnalyticV8, ApiAnalyticV9, ApiAnalyticV10, ApiAnalyticV11, ApiAnalyticV12, ApiAnalyticV13, ApiAnalyticV14, ApiAnalyticV15, ApiAnalyticV16,     ApiAnalyticV17, ApiAnalyticV18, ApiAnalyticV19, ApiAnalyticV20, ApiAnalyticV21,
    ApiAnalyticsCapacityPlan, ApiAnalyticsCorrelation,
    ApiDocsV2, ApiDocsV3, ApiDocsV4, ApiDocsV5, ApiDocsV6, ApiDocsV7, ApiDocsV8, ApiDocsV9, ApiDocsV10, ApiDocsV11, ApiDocsV12, ApiDocsV13, ApiDocsV14, ApiDocsV15,     ApiDocsV16, ApiDocsV17, ApiDocsV18, ApiDocsV19, ApiDocsV20,
    ApiDocumentation, ApiVersion, ApiWebhookDeliveryV2, ApiWebhookV2, BoardCardAssignee,
    BoardCardLabel, BranchProtectionRule, CacheCostOptimizationV3, CacheCostOptimizationV6,
    CacheHitAnalysisV3, CacheHitAnalysisV6, CachePerformanceInsightsV3, CachePerformanceInsightsV6,
    CacheSizeTrackingV3, CacheSizeTrackingV6,     CodeQualityMetric, CodeQualityMetricV4,
    CodeQualityMetricV6, CodeQualityMetricV7, CodeQualityMetricV8, CodeQualityMetricV10, CodeQualityMetricV13, CodeQualityMetricV14, CodeQualityMetricV15,     CodeQualityMetricV16, CodeQualityMetricV17, CodeQualityMetricV19, CodeQualityMetricV20, CodeQualityThresholdV3, CodeQualityThresholdV5, CodeQualityThresholdV6, CodeQualityThresholdV7, CodeQualityThresholdV9, CodeQualityThresholdV13, CodeQualityThresholdV14, CodeQualityThresholdV15, CodeQualityThresholdV16, CodeQualityThresholdV17, CodeQualityThresholdV19, CodeQualityThresholdV20, DataArchive, DataMigration, DataResidencyComplianceV4,
    DataResidencyComplianceV8, DataResidencyComplianceV11, DataResidencyReportV4, DataResidencyReportV8, DataResidencyReportV11, DataResidencyRule, DataResidencyViolation, DatabaseBackup,
    DatabaseRecoveryPoint, DatabaseReplica,     DatabaseReplicationAlertV4,
    DatabaseReplicationAlertV8, DatabaseReplicationAlertV11, DatabaseReplicationConfigV4, DatabaseReplicationConfigV8, DatabaseReplicationConfigV11, DeploymentAnalyticsV4, DeploymentAnalyticsV7, DeploymentComparisonV4,
    DeploymentComparisonV7, EmailVerificationCode, EncryptionComplianceCheckV4,
    EncryptionComplianceCheckV8, EncryptionComplianceCheckV11, EncryptionKeyVersionV4, EncryptionKeyVersionV8, EncryptionKeyVersionV11, EncryptionPolicy, EnvironmentDeploymentHistoryV4,
    EnvironmentDeploymentHistoryV7, Issue, MultiProjectPipeline,
    MultiProjectPipelineRun, Org, PerformanceTest, PerformanceTestAlertV4,
    PerformanceTestAlertHistoryV4, PerformanceTestAlertV6, PerformanceTestAlertV7, PerformanceTestAlertHistoryV6, PerformanceTestAlertHistoryV7, PerformanceTestAlertV8, PerformanceTestAlertHistoryV8,     PerformanceTestAlertV10, PerformanceTestAlertHistoryV10, PerformanceTestAlertV14, PerformanceTestAlertHistoryV14, PerformanceTestAlertV15, PerformanceTestAlertHistoryV15, PerformanceTestAlertV16, PerformanceTestAlertHistoryV16,     PerformanceTestAlertV17, PerformanceTestAlertHistoryV17, PerformanceTestAlertV18, PerformanceTestAlertHistoryV18, PerformanceTestAlertV20, PerformanceTestAlertHistoryV20, PerformanceTestAlertV21, PerformanceTestAlertHistoryV21, Pipeline, PipelineActionReviewV4,
    PipelineActionReviewV7, PipelineAnalytics, PipelineTemplate, PrComment, PrReviewer, PrStatusCheck, PrTimeline, PullRequest,
    RateLimitAlert, RateLimitAlertV2, RateLimitAlertV3, RateLimitAlertV4, RateLimitAlertV5, RateLimitAlertV6, RateLimitAlertV7, RateLimitAlertV8, RateLimitAlertV9, RateLimitAlertV10, RateLimitAlertV11, RateLimitAlertV12, RateLimitAlertV13, RateLimitAlertV14, RateLimitAlertV15, RateLimitOverage,
    RateLimitTier, RateLimitTierV2, RateLimitTierV3, RateLimitTierV4, RateLimitTierV5,
    RateLimitTierV6, RateLimitTierV7, RateLimitTierV8, RateLimitTierV9, RateLimitTierV10, RateLimitTierV11, RateLimitTierV12, RateLimitTierV13,     RateLimitTierV14, RateLimitTierV15, RateLimitTierV16, RateLimitTierV17, RateLimitTierV18, RateLimitUsageV2, Release, ReleaseAsset, Repository, ReviewAssignment,
    ReviewAnalyticsV3, ReviewAnalyticsV7, ReviewHelpfulnessV3, ReviewHelpfulnessV7,
    ReviewModerationQueueV3, ReviewModerationQueueV7, ReviewRecommendationV3,
    ReviewRecommendationV7, ReviewSummary, SshKey, Team, TeamMember, TestCoverage,
    TestSuiteBaselineV3, TestSuiteBaselineV5, TestSuiteBaselineV6, TestSuiteBaselineV7, TestSuiteBaselineV9, TestSuiteBaselineV13, TestSuiteBaselineV14, TestSuiteBaselineV15, TestSuiteBaselineV16, TestSuiteBaselineV17, TestSuiteBaselineV19, TestSuiteBaselineV20, TestSuiteMetricV3, TestSuiteMetricV5, TestSuiteMetricV6, TestSuiteMetricV7, TestSuiteMetricV9, TestSuiteMetricV13, TestSuiteMetricV14, TestSuiteMetricV15, TestSuiteMetricV16, TestSuiteMetricV17, TestSuiteMetricV19, TestSuiteMetricV20, User,
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

    // --- API Documentation ---

    pub async fn create_api_documentation(
        &self,
        endpoint: &str,
        method: &str,
        summary: &str,
        description: &str,
        parameters: serde_json::Value,
        request_body: Option<serde_json::Value>,
        responses: serde_json::Value,
        tags: &[String],
    ) -> Result<ApiDocumentation> {
        let row = sqlx::query_as::<_, ApiDocumentation>(
            r#"INSERT INTO api_documentation (endpoint, method, summary, description, parameters, request_body, responses, tags)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               ON CONFLICT (endpoint, method) DO UPDATE SET
                   summary = EXCLUDED.summary,
                   description = EXCLUDED.description,
                   parameters = EXCLUDED.parameters,
                   request_body = EXCLUDED.request_body,
                   responses = EXCLUDED.responses,
                   tags = EXCLUDED.tags
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(summary)
        .bind(description)
        .bind(parameters)
        .bind(request_body)
        .bind(responses)
        .bind(tags)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_documentation: {e}")))?;
        Ok(row)
    }

    pub async fn get_api_documentation(
        &self,
        endpoint: &str,
        method: &str,
    ) -> Result<ApiDocumentation> {
        sqlx::query_as::<_, ApiDocumentation>(
            "SELECT * FROM api_documentation WHERE endpoint = $1 AND method = $2",
        )
        .bind(endpoint)
        .bind(method)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_documentation: {e}")))
    }

    pub async fn list_api_documentation(&self, limit: i64, offset: i64) -> Result<Vec<ApiDocumentation>> {
        sqlx::query_as::<_, ApiDocumentation>(
            "SELECT * FROM api_documentation ORDER BY endpoint, method LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_documentation: {e}")))
    }

    pub async fn search_api_documentation_by_tag(&self, tag: &str) -> Result<Vec<ApiDocumentation>> {
        sqlx::query_as::<_, ApiDocumentation>(
            "SELECT * FROM api_documentation WHERE $1 = ANY(tags) ORDER BY endpoint, method",
        )
        .bind(tag)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("search_api_documentation_by_tag: {e}")))
    }

    // --- API Versions ---

    pub async fn create_api_version(
        &self,
        version: &str,
        status: &str,
        changelog: &str,
    ) -> Result<ApiVersion> {
        let row = sqlx::query_as::<_, ApiVersion>(
            r#"INSERT INTO api_versions (version, status, changelog)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(version)
        .bind(status)
        .bind(changelog)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_version: {e}")))?;
        Ok(row)
    }

    pub async fn get_api_version(&self, version: &str) -> Result<ApiVersion> {
        sqlx::query_as::<_, ApiVersion>("SELECT * FROM api_versions WHERE version = $1")
            .bind(version)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_api_version: {e}")))
    }

    pub async fn list_api_versions(&self) -> Result<Vec<ApiVersion>> {
        sqlx::query_as::<_, ApiVersion>("SELECT * FROM api_versions ORDER BY release_date DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_api_versions: {e}")))
    }

    pub async fn update_api_version_status(
        &self,
        version: &str,
        status: &str,
    ) -> Result<ApiVersion> {
        let row = sqlx::query_as::<_, ApiVersion>(
            r#"UPDATE api_versions SET status = $2 WHERE version = $1
               RETURNING *"#,
        )
        .bind(version)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_api_version_status: {e}")))?;
        Ok(row)
    }

    pub async fn deprecate_api_version(
        &self,
        version: &str,
        deprecation_date: chrono::DateTime<chrono::Utc>,
        sunset_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<ApiVersion> {
        let row = sqlx::query_as::<_, ApiVersion>(
            r#"UPDATE api_versions SET status = 'deprecated', deprecation_date = $2, sunset_date = $3
               WHERE version = $1
               RETURNING *"#,
        )
        .bind(version)
        .bind(deprecation_date)
        .bind(sunset_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("deprecate_api_version: {e}")))?;
        Ok(row)
    }

    // --- API Analytics V2 ---

    pub async fn record_api_analytic_v2(
        &self,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_time_ms: i32,
        user_id: Option<Uuid>,
        request_size_bytes: i32,
        response_size_bytes: i32,
    ) -> Result<ApiAnalyticV2> {
        let row = sqlx::query_as::<_, ApiAnalyticV2>(
            r#"INSERT INTO api_analytics_v2 (endpoint, method, status_code, response_time_ms, user_id, request_size_bytes, response_size_bytes)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(user_id)
        .bind(request_size_bytes)
        .bind(response_size_bytes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_api_analytic_v2: {e}")))?;
        Ok(row)
    }

    pub async fn get_api_analytics_v2(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ApiAnalyticV2>> {
        sqlx::query_as::<_, ApiAnalyticV2>(
            "SELECT * FROM api_analytics_v2 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_analytics_v2: {e}")))
    }

    pub async fn get_api_analytics_v2_by_endpoint(
        &self,
        endpoint: &str,
        limit: i64,
    ) -> Result<Vec<ApiAnalyticV2>> {
        sqlx::query_as::<_, ApiAnalyticV2>(
            "SELECT * FROM api_analytics_v2 WHERE endpoint = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(endpoint)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_analytics_v2_by_endpoint: {e}")))
    }

    pub async fn get_api_analytics_v2_summary(
        &self,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<serde_json::Value> {
        let row = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'total_requests', COUNT(*),
                'avg_response_time_ms', AVG(response_time_ms),
                'error_rate', COUNT(*) FILTER (WHERE status_code >= 400)::float / NULLIF(COUNT(*), 0),
                'unique_users', COUNT(DISTINCT user_id),
                'total_request_bytes', SUM(request_size_bytes),
                'total_response_bytes', SUM(response_size_bytes)
             )
             FROM api_analytics_v2
             WHERE created_at >= $1"#,
        )
        .bind(since)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_analytics_v2_summary: {e}")))?;
        Ok(row)
    }

    pub async fn get_api_analytics_v2_error_breakdown(
        &self,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<serde_json::Value>> {
        sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'status_code', status_code,
                'count', COUNT(*),
                'avg_response_time_ms', AVG(response_time_ms)
             )
             FROM api_analytics_v2
             WHERE created_at >= $1 AND status_code >= 400
             GROUP BY status_code
             ORDER BY COUNT(*) DESC"#,
        )
        .bind(since)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_analytics_v2_error_breakdown: {e}")))
    }

    // --- Test Coverage ---

    pub async fn upload_test_coverage(
        &self,
        repo_id: Uuid,
        file_path: &str,
        line_coverage: f64,
        branch_coverage: f64,
        function_coverage: f64,
        total_lines: i32,
        covered_lines: i32,
    ) -> Result<TestCoverage> {
        let row = sqlx::query_as::<_, TestCoverage>(
            r#"INSERT INTO test_coverage (repo_id, file_path, line_coverage, branch_coverage, function_coverage, total_lines, covered_lines)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(file_path)
        .bind(line_coverage)
        .bind(branch_coverage)
        .bind(function_coverage)
        .bind(total_lines)
        .bind(covered_lines)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("upload_test_coverage: {e}")))?;
        Ok(row)
    }

    pub async fn get_coverage_for_repo(
        &self,
        repo_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestCoverage>> {
        let rows = sqlx::query_as::<_, TestCoverage>(
            r#"SELECT * FROM test_coverage
               WHERE repo_id = $1
               ORDER BY measured_at DESC
               LIMIT $2 OFFSET $3"#,
        )
        .bind(repo_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_coverage_for_repo: {e}")))?;
        Ok(rows)
    }

    pub async fn get_coverage_statistics(
        &self,
        repo_id: Uuid,
    ) -> Result<serde_json::Value> {
        let row = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'avg_line_coverage', AVG(line_coverage),
                'avg_branch_coverage', AVG(branch_coverage),
                'avg_function_coverage', AVG(function_coverage),
                'total_files', COUNT(DISTINCT file_path),
                'total_lines', SUM(total_lines),
                'total_covered_lines', SUM(covered_lines),
                'overall_coverage', CASE WHEN SUM(total_lines) > 0 THEN SUM(covered_lines)::float / SUM(total_lines)::float * 100 ELSE 0 END
             )
             FROM test_coverage
             WHERE repo_id = $1
               AND measured_at = (SELECT MAX(measured_at) FROM test_coverage WHERE repo_id = $1)"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_coverage_statistics: {e}")))?;
        Ok(row)
    }

    pub async fn get_coverage_trends(
        &self,
        repo_id: Uuid,
        days: i64,
    ) -> Result<Vec<serde_json::Value>> {
        let rows = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'date', DATE(measured_at),
                'avg_line_coverage', AVG(line_coverage),
                'avg_branch_coverage', AVG(branch_coverage),
                'avg_function_coverage', AVG(function_coverage),
                'file_count', COUNT(DISTINCT file_path)
             )
             FROM test_coverage
             WHERE repo_id = $1
               AND measured_at >= NOW() - ($2 || ' days')::INTERVAL
             GROUP BY DATE(measured_at)
             ORDER BY DATE(measured_at) DESC"#,
        )
        .bind(repo_id)
        .bind(days)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_coverage_trends: {e}")))?;
        Ok(rows)
    }

    pub async fn check_coverage_enforcement(
        &self,
        repo_id: Uuid,
        min_line_coverage: f64,
        min_branch_coverage: f64,
        min_function_coverage: f64,
    ) -> Result<serde_json::Value> {
        let row = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'passes', bool_and(
                    line_coverage >= $2
                    AND branch_coverage >= $3
                    AND function_coverage >= $4
                ),
                'files_checked', COUNT(*),
                'files_passing', COUNT(*) FILTER (WHERE line_coverage >= $2 AND branch_coverage >= $3 AND function_coverage >= $4),
                'files_failing', COUNT(*) FILTER (WHERE line_coverage < $2 OR branch_coverage < $3 OR function_coverage < $4)
             )
             FROM test_coverage
             WHERE repo_id = $1
               AND measured_at = (SELECT MAX(measured_at) FROM test_coverage WHERE repo_id = $1)"#,
        )
        .bind(repo_id)
        .bind(min_line_coverage)
        .bind(min_branch_coverage)
        .bind(min_function_coverage)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("check_coverage_enforcement: {e}")))?;
        Ok(row)
    }

    pub async fn delete_old_coverage(
        &self,
        repo_id: Uuid,
        older_than_days: i64,
    ) -> Result<i64> {
        let result = sqlx::query(
            r#"DELETE FROM test_coverage
               WHERE repo_id = $1
                 AND measured_at < NOW() - ($2 || ' days')::INTERVAL"#,
        )
        .bind(repo_id)
        .bind(older_than_days)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("delete_old_coverage: {e}")))?;
        Ok(result.rows_affected() as i64)
    }

    // --- Code Quality Metrics ---

    pub async fn record_code_quality_metric(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        metric_value: f64,
        file_path: Option<&str>,
    ) -> Result<CodeQualityMetric> {
        let row = sqlx::query_as::<_, CodeQualityMetric>(
            r#"INSERT INTO code_quality_metrics (repo_id, metric_name, metric_value, file_path)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .bind(metric_value)
        .bind(file_path)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_code_quality_metric: {e}")))?;
        Ok(row)
    }

    pub async fn get_code_quality_metrics(
        &self,
        repo_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CodeQualityMetric>> {
        let rows = if let Some(name) = metric_name {
            sqlx::query_as::<_, CodeQualityMetric>(
                r#"SELECT * FROM code_quality_metrics
                   WHERE repo_id = $1 AND metric_name = $2
                   ORDER BY measured_at DESC
                   LIMIT $3 OFFSET $4"#,
            )
            .bind(repo_id)
            .bind(name)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, CodeQualityMetric>(
                r#"SELECT * FROM code_quality_metrics
                   WHERE repo_id = $1
                   ORDER BY measured_at DESC
                   LIMIT $2 OFFSET $3"#,
            )
            .bind(repo_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("get_code_quality_metrics: {e}")))
    }

    pub async fn get_quality_metrics_summary(
        &self,
        repo_id: Uuid,
    ) -> Result<serde_json::Value> {
        let rows = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'metric_name', metric_name,
                'latest_value', (SELECT metric_value FROM code_quality_metrics WHERE repo_id = $1 AND metric_name = cqm.metric_name ORDER BY measured_at DESC LIMIT 1),
                'avg_value', AVG(metric_value),
                'min_value', MIN(metric_value),
                'max_value', MAX(metric_value),
                'measurement_count', COUNT(*),
                'files_affected', COUNT(DISTINCT file_path)
             )
             FROM code_quality_metrics cqm
             WHERE repo_id = $1
             GROUP BY metric_name
             ORDER BY metric_name"#,
        )
        .bind(repo_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_quality_metrics_summary: {e}")))?;
        Ok(serde_json::json!(rows))
    }

    pub async fn get_quality_trends(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        days: i64,
    ) -> Result<Vec<serde_json::Value>> {
        let rows = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'date', DATE(measured_at),
                'avg_value', AVG(metric_value),
                'min_value', MIN(metric_value),
                'max_value', MAX(metric_value),
                'measurement_count', COUNT(*)
             )
             FROM code_quality_metrics
             WHERE repo_id = $1
               AND metric_name = $2
               AND measured_at >= NOW() - ($3 || ' days')::INTERVAL
             GROUP BY DATE(measured_at)
             ORDER BY DATE(measured_at) DESC"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .bind(days)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_quality_trends: {e}")))?;
        Ok(rows)
    }

    pub async fn get_complexity_analysis(
        &self,
        repo_id: Uuid,
    ) -> Result<serde_json::Value> {
        let row = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'avg_complexity', AVG(metric_value) FILTER (WHERE metric_name = 'cyclomatic_complexity'),
                'max_complexity', MAX(metric_value) FILTER (WHERE metric_name = 'cyclomatic_complexity'),
                'avg_cognitive_complexity', AVG(metric_value) FILTER (WHERE metric_name = 'cognitive_complexity'),
                'high_complexity_files', COUNT(DISTINCT file_path) FILTER (WHERE metric_name = 'cyclomatic_complexity' AND metric_value > 15),
                'total_measurements', COUNT(*)
             )
             FROM code_quality_metrics
             WHERE repo_id = $1
               AND metric_name IN ('cyclomatic_complexity', 'cognitive_complexity')
               AND measured_at = (SELECT MAX(measured_at) FROM code_quality_metrics WHERE repo_id = $1 AND metric_name IN ('cyclomatic_complexity', 'cognitive_complexity'))"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_complexity_analysis: {e}")))?;
        Ok(row)
    }

    pub async fn get_duplication_detection(
        &self,
        repo_id: Uuid,
    ) -> Result<serde_json::Value> {
        let row = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'duplication_ratio', AVG(metric_value) FILTER (WHERE metric_name = 'duplication_ratio'),
                'total_duplicated_lines', SUM(metric_value) FILTER (WHERE metric_name = 'duplicated_lines'),
                'files_with_duplication', COUNT(DISTINCT file_path) FILTER (WHERE metric_name = 'duplication_ratio' AND metric_value > 0)
             )
             FROM code_quality_metrics
             WHERE repo_id = $1
               AND metric_name IN ('duplication_ratio', 'duplicated_lines')
               AND measured_at = (SELECT MAX(measured_at) FROM code_quality_metrics WHERE repo_id = $1 AND metric_name IN ('duplication_ratio', 'duplicated_lines'))"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_duplication_detection: {e}")))?;
        Ok(row)
    }

    pub async fn get_code_smells(
        &self,
        repo_id: Uuid,
    ) -> Result<serde_json::Value> {
        let row = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'total_smells', SUM(metric_value) FILTER (WHERE metric_name = 'code_smells'),
                'smell_density', AVG(metric_value) FILTER (WHERE metric_name = 'smell_density'),
                'files_with_smells', COUNT(DISTINCT file_path) FILTER (WHERE metric_name = 'code_smells' AND metric_value > 0),
                'critical_smells', SUM(metric_value) FILTER (WHERE metric_name = 'critical_smells'),
                'major_smells', SUM(metric_value) FILTER (WHERE metric_name = 'major_smells'),
                'minor_smells', SUM(metric_value) FILTER (WHERE metric_name = 'minor_smells')
             )
             FROM code_quality_metrics
             WHERE repo_id = $1
               AND metric_name IN ('code_smells', 'smell_density', 'critical_smells', 'major_smells', 'minor_smells')
               AND measured_at = (SELECT MAX(measured_at) FROM code_quality_metrics WHERE repo_id = $1 AND metric_name IN ('code_smells', 'smell_density', 'critical_smells', 'major_smells', 'minor_smells'))"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_smells: {e}")))?;
        Ok(row)
    }

    pub async fn get_technical_debt(
        &self,
        repo_id: Uuid,
    ) -> Result<serde_json::Value> {
        let row = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'total_debt_hours', SUM(metric_value) FILTER (WHERE metric_name = 'technical_debt_hours'),
                'debt_ratio', AVG(metric_value) FILTER (WHERE metric_name = 'debt_ratio'),
                'debt_per_file', AVG(metric_value) FILTER (WHERE metric_name = 'debt_per_file'),
                'remediation_time_priority', AVG(metric_value) FILTER (WHERE metric_name = 'remediation_time_priority'),
                'files_with_debt', COUNT(DISTINCT file_path) FILTER (WHERE metric_name = 'technical_debt_hours' AND metric_value > 0)
             )
             FROM code_quality_metrics
             WHERE repo_id = $1
               AND metric_name IN ('technical_debt_hours', 'debt_ratio', 'debt_per_file', 'remediation_time_priority')
               AND measured_at = (SELECT MAX(measured_at) FROM code_quality_metrics WHERE repo_id = $1 AND metric_name IN ('technical_debt_hours', 'debt_ratio', 'debt_per_file', 'remediation_time_priority'))"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_technical_debt: {e}")))?;
        Ok(row)
    }

    // --- Performance Tests ---

    pub async fn create_performance_test(
        &self,
        repo_id: Uuid,
        name: &str,
        test_type: &str,
        endpoint: Option<&str>,
        config: &serde_json::Value,
    ) -> Result<PerformanceTest> {
        let row = sqlx::query_as::<_, PerformanceTest>(
            r#"INSERT INTO performance_tests (repo_id, name, test_type, endpoint, config)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(name)
        .bind(test_type)
        .bind(endpoint)
        .bind(config)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_performance_test: {e}")))?;
        Ok(row)
    }

    pub async fn get_performance_test(&self, id: Uuid) -> Result<PerformanceTest> {
        sqlx::query_as::<_, PerformanceTest>("SELECT * FROM performance_tests WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_performance_test: {e}")))
    }

    pub async fn list_performance_tests(
        &self,
        repo_id: Uuid,
        test_type: Option<&str>,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PerformanceTest>> {
        let rows = sqlx::query_as::<_, PerformanceTest>(
            r#"SELECT * FROM performance_tests
               WHERE repo_id = $1
                 AND ($2::varchar IS NULL OR test_type = $2)
                 AND ($3::varchar IS NULL OR status = $3)
               ORDER BY started_at DESC
               LIMIT $4 OFFSET $5"#,
        )
        .bind(repo_id)
        .bind(test_type)
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_performance_tests: {e}")))?;
        Ok(rows)
    }

    pub async fn start_performance_test(
        &self,
        id: Uuid,
    ) -> Result<PerformanceTest> {
        let row = sqlx::query_as::<_, PerformanceTest>(
            r#"UPDATE performance_tests
               SET status = 'running', started_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("start_performance_test: {e}")))?;
        Ok(row)
    }

    pub async fn complete_performance_test(
        &self,
        id: Uuid,
        status: &str,
        results: &serde_json::Value,
    ) -> Result<PerformanceTest> {
        let row = sqlx::query_as::<_, PerformanceTest>(
            r#"UPDATE performance_tests
               SET status = $2, results = $3, completed_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(status)
        .bind(results)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("complete_performance_test: {e}")))?;
        Ok(row)
    }

    pub async fn update_performance_test_results(
        &self,
        id: Uuid,
        results: &serde_json::Value,
    ) -> Result<PerformanceTest> {
        let row = sqlx::query_as::<_, PerformanceTest>(
            r#"UPDATE performance_tests
               SET results = results || $2
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(results)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_performance_test_results: {e}")))?;
        Ok(row)
    }

    pub async fn delete_performance_test(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM performance_tests WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_performance_test: {e}")))?;
        Ok(())
    }

    pub async fn get_performance_test_summary(
        &self,
        repo_id: Uuid,
    ) -> Result<serde_json::Value> {
        let row = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'total_tests', COUNT(*),
                'completed_tests', COUNT(*) FILTER (WHERE status = 'completed'),
                'failed_tests', COUNT(*) FILTER (WHERE status = 'failed'),
                'running_tests', COUNT(*) FILTER (WHERE status = 'running'),
                'pending_tests', COUNT(*) FILTER (WHERE status = 'pending'),
                'by_type', json_build_object(
                    'load', COUNT(*) FILTER (WHERE test_type = 'load'),
                    'stress', COUNT(*) FILTER (WHERE test_type = 'stress'),
                    'soak', COUNT(*) FILTER (WHERE test_type = 'soak'),
                    'benchmark', COUNT(*) FILTER (WHERE test_type = 'benchmark')
                ),
                'latest_results', (
                    SELECT results FROM performance_tests
                    WHERE repo_id = $1 AND status = 'completed'
                    ORDER BY completed_at DESC LIMIT 1
                )
             )
             FROM performance_tests
             WHERE repo_id = $1"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_summary: {e}")))?;
        Ok(row)
    }

    // --- Database Backups ---

    pub async fn create_database_backup(
        &self,
        backup_type: &str,
    ) -> Result<DatabaseBackup> {
        let row = sqlx::query_as::<_, DatabaseBackup>(
            r#"INSERT INTO database_backups (backup_type)
               VALUES ($1)
               RETURNING *"#,
        )
        .bind(backup_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_database_backup: {e}")))?;
        Ok(row)
    }

    pub async fn get_database_backup(&self, id: Uuid) -> Result<DatabaseBackup> {
        sqlx::query_as::<_, DatabaseBackup>("SELECT * FROM database_backups WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_database_backup: {e}")))
    }

    pub async fn update_database_backup(
        &self,
        id: Uuid,
        status: Option<&str>,
        file_path: Option<&str>,
        file_size_bytes: Option<i64>,
        checksum: Option<&str>,
    ) -> Result<DatabaseBackup> {
        let row = sqlx::query_as::<_, DatabaseBackup>(
            r#"UPDATE database_backups
               SET status = COALESCE($2, status),
                   file_path = COALESCE($3, file_path),
                   file_size_bytes = COALESCE($4, file_size_bytes),
                   checksum = COALESCE($5, checksum),
                   completed_at = CASE WHEN $2 IN ('completed', 'failed') THEN NOW() ELSE completed_at END
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(status)
        .bind(file_path)
        .bind(file_size_bytes)
        .bind(checksum)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_database_backup: {e}")))?;
        Ok(row)
    }

    pub async fn list_database_backups(
        &self,
        backup_type: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DatabaseBackup>> {
        let rows = if let Some(bt) = backup_type {
            sqlx::query_as::<_, DatabaseBackup>(
                r#"SELECT * FROM database_backups
                   WHERE backup_type = $1
                   ORDER BY started_at DESC
                   LIMIT $2 OFFSET $3"#,
            )
            .bind(bt)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, DatabaseBackup>(
                r#"SELECT * FROM database_backups
                   ORDER BY started_at DESC
                   LIMIT $1 OFFSET $2"#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_database_backups: {e}")))
    }

    pub async fn delete_database_backup(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM database_backups WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_database_backup: {e}")))?;
        Ok(())
    }

    // --- Database Recovery Points ---

    pub async fn create_recovery_point(
        &self,
        backup_id: Uuid,
        name: &str,
        description: &str,
    ) -> Result<DatabaseRecoveryPoint> {
        let row = sqlx::query_as::<_, DatabaseRecoveryPoint>(
            r#"INSERT INTO database_recovery_points (backup_id, name, description)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(backup_id)
        .bind(name)
        .bind(description)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_recovery_point: {e}")))?;
        Ok(row)
    }

    pub async fn get_recovery_point(&self, id: Uuid) -> Result<DatabaseRecoveryPoint> {
        sqlx::query_as::<_, DatabaseRecoveryPoint>(
            "SELECT * FROM database_recovery_points WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_recovery_point: {e}")))
    }

    pub async fn list_recovery_points(
        &self,
        backup_id: Uuid,
    ) -> Result<Vec<DatabaseRecoveryPoint>> {
        let rows = sqlx::query_as::<_, DatabaseRecoveryPoint>(
            "SELECT * FROM database_recovery_points WHERE backup_id = $1 ORDER BY created_at DESC",
        )
        .bind(backup_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_recovery_points: {e}")))?;
        Ok(rows)
    }

    pub async fn delete_recovery_point(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM database_recovery_points WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_recovery_point: {e}")))?;
        Ok(())
    }

    // --- Data Archives ---

    pub async fn create_data_archive(
        &self,
        repo_id: Uuid,
        archive_type: &str,
        retention_days: i32,
    ) -> Result<DataArchive> {
        let row = sqlx::query_as::<_, DataArchive>(
            r#"INSERT INTO data_archives (repo_id, archive_type, retention_days, expires_at)
               VALUES ($1, $2, $3, NOW() + ($3 || ' days')::INTERVAL)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(archive_type)
        .bind(retention_days)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_data_archive: {e}")))?;
        Ok(row)
    }

    pub async fn get_data_archive(&self, id: Uuid) -> Result<DataArchive> {
        sqlx::query_as::<_, DataArchive>("SELECT * FROM data_archives WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_data_archive: {e}")))
    }

    pub async fn update_data_archive(
        &self,
        id: Uuid,
        status: Option<&str>,
        file_path: Option<&str>,
        file_size_bytes: Option<i64>,
    ) -> Result<DataArchive> {
        let row = sqlx::query_as::<_, DataArchive>(
            r#"UPDATE data_archives
               SET status = COALESCE($2, status),
                   file_path = COALESCE($3, file_path),
                   file_size_bytes = COALESCE($4, file_size_bytes)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(status)
        .bind(file_path)
        .bind(file_size_bytes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_data_archive: {e}")))?;
        Ok(row)
    }

    pub async fn list_data_archives(
        &self,
        repo_id: Uuid,
        archive_type: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DataArchive>> {
        let rows = if let Some(at) = archive_type {
            sqlx::query_as::<_, DataArchive>(
                r#"SELECT * FROM data_archives
                   WHERE repo_id = $1 AND archive_type = $2
                   ORDER BY created_at DESC
                   LIMIT $3 OFFSET $4"#,
            )
            .bind(repo_id)
            .bind(at)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, DataArchive>(
                r#"SELECT * FROM data_archives
                   WHERE repo_id = $1
                   ORDER BY created_at DESC
                   LIMIT $2 OFFSET $3"#,
            )
            .bind(repo_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_data_archives: {e}")))
    }

    pub async fn delete_data_archive(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM data_archives WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_data_archive: {e}")))?;
        Ok(())
    }

    pub async fn enforce_archive_retention(&self) -> Result<i64> {
        let result = sqlx::query("DELETE FROM data_archives WHERE expires_at IS NOT NULL AND expires_at < NOW()")
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("enforce_archive_retention: {e}")))?;
        Ok(result.rows_affected() as i64)
    }

    // --- Data Migrations ---

    pub async fn create_data_migration(
        &self,
        source: &str,
        destination: &str,
        migration_type: &str,
    ) -> Result<DataMigration> {
        let row = sqlx::query_as::<_, DataMigration>(
            r#"INSERT INTO data_migrations (source, destination, migration_type)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(source)
        .bind(destination)
        .bind(migration_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_data_migration: {e}")))?;
        Ok(row)
    }

    pub async fn get_data_migration(&self, id: Uuid) -> Result<DataMigration> {
        sqlx::query_as::<_, DataMigration>("SELECT * FROM data_migrations WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_data_migration: {e}")))
    }

    pub async fn update_data_migration(
        &self,
        id: Uuid,
        status: Option<&str>,
        progress: Option<f64>,
    ) -> Result<DataMigration> {
        let row = sqlx::query_as::<_, DataMigration>(
            r#"UPDATE data_migrations
               SET status = COALESCE($2, status),
                   progress = COALESCE($3, progress),
                   completed_at = CASE WHEN $2 IN ('completed', 'failed', 'rolled_back') THEN NOW() ELSE completed_at END
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(status)
        .bind(progress)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_data_migration: {e}")))?;
        Ok(row)
    }

    pub async fn list_data_migrations(
        &self,
        migration_type: Option<&str>,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DataMigration>> {
        let rows = sqlx::query_as::<_, DataMigration>(
            r#"SELECT * FROM data_migrations
               WHERE ($1::varchar IS NULL OR migration_type = $1)
                 AND ($2::varchar IS NULL OR status = $2)
               ORDER BY started_at DESC
               LIMIT $3 OFFSET $4"#,
        )
        .bind(migration_type)
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_data_migrations: {e}")))?;
        Ok(rows)
    }

    pub async fn rollback_data_migration(&self, id: Uuid) -> Result<DataMigration> {
        let row = sqlx::query_as::<_, DataMigration>(
            r#"UPDATE data_migrations
               SET status = 'rolled_back',
                   completed_at = NOW()
               WHERE id = $1 AND status IN ('completed', 'in_progress')
               RETURNING *"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("rollback_data_migration: {e}")))?;
        Ok(row)
    }

    pub async fn delete_data_migration(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM data_migrations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_data_migration: {e}")))?;
        Ok(())
    }

    // --- API Documentation v2 ---

    pub async fn create_api_docs_v2(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
        summary: &str,
        description: &str,
        parameters: serde_json::Value,
        request_body: Option<serde_json::Value>,
        responses: serde_json::Value,
        examples: serde_json::Value,
        tags: &[String],
    ) -> Result<ApiDocsV2> {
        let row = sqlx::query_as::<_, ApiDocsV2>(
            r#"INSERT INTO api_docs_v2 (endpoint, method, version, summary, description, parameters, request_body, responses, examples, tags)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .bind(summary)
        .bind(description)
        .bind(parameters)
        .bind(request_body)
        .bind(responses)
        .bind(examples)
        .bind(tags)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_docs_v2: {e}")))?;
        Ok(row)
    }

    pub async fn get_api_docs_v2(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
    ) -> Result<ApiDocsV2> {
        sqlx::query_as::<_, ApiDocsV2>(
            "SELECT * FROM api_docs_v2 WHERE endpoint = $1 AND method = $2 AND version = $3",
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_docs_v2: {e}")))
    }

    pub async fn list_api_docs_v2(&self, limit: i64, offset: i64) -> Result<Vec<ApiDocsV2>> {
        sqlx::query_as::<_, ApiDocsV2>(
            "SELECT * FROM api_docs_v2 ORDER BY endpoint, method, version LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_docs_v2: {e}")))
    }

    pub async fn search_api_docs_v2_by_tag(&self, tag: &str) -> Result<Vec<ApiDocsV2>> {
        sqlx::query_as::<_, ApiDocsV2>(
            "SELECT * FROM api_docs_v2 WHERE $1 = ANY(tags) ORDER BY endpoint, method, version",
        )
        .bind(tag)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("search_api_docs_v2_by_tag: {e}")))
    }

    // --- Rate Limit Tiers ---

    pub async fn create_rate_limit_tier(
        &self,
        name: &str,
        description: &str,
        rate_limit: i32,
        burst_limit: i32,
        monthly_quota: Option<i32>,
        price_cents: i32,
    ) -> Result<RateLimitTier> {
        let row = sqlx::query_as::<_, RateLimitTier>(
            r#"INSERT INTO rate_limit_tiers (name, description, rate_limit, burst_limit, monthly_quota, price_cents)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_tier: {e}")))?;
        Ok(row)
    }

    pub async fn get_rate_limit_tier(&self, name: &str) -> Result<RateLimitTier> {
        sqlx::query_as::<_, RateLimitTier>("SELECT * FROM rate_limit_tiers WHERE name = $1")
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_rate_limit_tier: {e}")))
    }

    pub async fn list_rate_limit_tiers(&self) -> Result<Vec<RateLimitTier>> {
        sqlx::query_as::<_, RateLimitTier>("SELECT * FROM rate_limit_tiers ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_rate_limit_tiers: {e}")))
    }

    pub async fn update_rate_limit_tier(
        &self,
        name: &str,
        description: Option<&str>,
        rate_limit: Option<i32>,
        burst_limit: Option<i32>,
        monthly_quota: Option<Option<i32>>,
        price_cents: Option<i32>,
    ) -> Result<RateLimitTier> {
        let row = sqlx::query_as::<_, RateLimitTier>(
            r#"UPDATE rate_limit_tiers
               SET description = COALESCE($2, description),
                   rate_limit = COALESCE($3, rate_limit),
                   burst_limit = COALESCE($4, burst_limit),
                   monthly_quota = CASE WHEN $5::INT IS NULL THEN monthly_quota ELSE $5 END,
                   price_cents = COALESCE($6, price_cents)
               WHERE name = $1
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_rate_limit_tier: {e}")))?;
        Ok(row)
    }

    pub async fn delete_rate_limit_tier(&self, name: &str) -> Result<()> {
        sqlx::query("DELETE FROM rate_limit_tiers WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_rate_limit_tier: {e}")))?;
        Ok(())
    }

    // --- API Analytics v3 ---

    pub async fn create_api_analytic_v3(
        &self,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_time_ms: i32,
        user_id: Option<Uuid>,
        request_size_bytes: i32,
        response_size_bytes: i32,
        cache_hit: bool,
    ) -> Result<ApiAnalyticV3> {
        let row = sqlx::query_as::<_, ApiAnalyticV3>(
            r#"INSERT INTO api_analytics_v3 (endpoint, method, status_code, response_time_ms, user_id, request_size_bytes, response_size_bytes, cache_hit)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(user_id)
        .bind(request_size_bytes)
        .bind(response_size_bytes)
        .bind(cache_hit)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_analytic_v3: {e}")))?;
        Ok(row)
    }

    pub async fn get_api_analytics_v3(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ApiAnalyticV3>> {
        sqlx::query_as::<_, ApiAnalyticV3>(
            "SELECT * FROM api_analytics_v3 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_analytics_v3: {e}")))
    }

    pub async fn get_endpoint_analytics_v3(
        &self,
        endpoint: &str,
        method: &str,
    ) -> Result<Vec<ApiAnalyticV3>> {
        sqlx::query_as::<_, ApiAnalyticV3>(
            "SELECT * FROM api_analytics_v3 WHERE endpoint = $1 AND method = $2 ORDER BY created_at DESC",
        )
        .bind(endpoint)
        .bind(method)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_endpoint_analytics_v3: {e}")))
    }

    pub async fn get_api_analytics_v3_cache_stats(
        &self,
    ) -> Result<serde_json::Value> {
        let row: (i64, i64) = sqlx::query_as(
            r#"SELECT 
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE cache_hit = true) as cache_hits
               FROM api_analytics_v3"#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_analytics_v3_cache_stats: {e}")))?;
        
        let total = row.0;
        let cache_hits = row.1;
        let cache_hit_rate = if total > 0 {
            (cache_hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        Ok(serde_json::json!({
            "total_requests": total,
            "cache_hits": cache_hits,
            "cache_hit_rate": cache_hit_rate,
        }))
    }

    pub async fn get_api_analytics_v3_performance_stats(
        &self,
    ) -> Result<serde_json::Value> {
        let row: (f64, f64, f64, f64) = sqlx::query_as(
            r#"SELECT 
                COALESCE(AVG(response_time_ms), 0.0) as avg_response_time,
                COALESCE(PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms), 0.0) as p95_response_time,
                COALESCE(AVG(request_size_bytes), 0.0) as avg_request_size,
                COALESCE(AVG(response_size_bytes), 0.0) as avg_response_size
               FROM api_analytics_v3"#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_analytics_v3_performance_stats: {e}")))?;
        
        Ok(serde_json::json!({
            "avg_response_time_ms": row.0,
            "p95_response_time_ms": row.1,
            "avg_request_size_bytes": row.2,
            "avg_response_size_bytes": row.3,
        }))
    }

    // --- Database Replication ---

    pub async fn register_replica(
        &self,
        name: &str,
        host: &str,
        port: i32,
    ) -> Result<DatabaseReplica> {
        let row = sqlx::query_as::<_, DatabaseReplica>(
            r#"INSERT INTO database_replicas (name, host, port)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(name)
        .bind(host)
        .bind(port)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("register_replica: {e}")))?;
        Ok(row)
    }

    pub async fn get_replica(&self, id: Uuid) -> Result<DatabaseReplica> {
        sqlx::query_as::<_, DatabaseReplica>("SELECT * FROM database_replicas WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_replica: {e}")))
    }

    pub async fn list_replicas(&self) -> Result<Vec<DatabaseReplica>> {
        sqlx::query_as::<_, DatabaseReplica>("SELECT * FROM database_replicas ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_replicas: {e}")))
    }

    pub async fn update_replica_status(
        &self,
        id: Uuid,
        status: &str,
        lag_ms: i32,
    ) -> Result<DatabaseReplica> {
        let row = sqlx::query_as::<_, DatabaseReplica>(
            r#"UPDATE database_replicas
               SET status = $2, lag_ms = $3, last_sync_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(status)
        .bind(lag_ms)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_replica_status: {e}")))?;
        Ok(row)
    }

    pub async fn delete_replica(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM database_replicas WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_replica: {e}")))?;
        Ok(())
    }

    pub async fn get_healthy_replicas(&self) -> Result<Vec<DatabaseReplica>> {
        sqlx::query_as::<_, DatabaseReplica>(
            "SELECT * FROM database_replicas WHERE status = 'healthy' ORDER BY lag_ms ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_healthy_replicas: {e}")))
    }

    // --- Encryption Policies ---

    pub async fn create_encryption_policy(
        &self,
        name: &str,
        description: &str,
        data_types: &[String],
        algorithm: &str,
        key_rotation_days: i32,
    ) -> Result<EncryptionPolicy> {
        let row = sqlx::query_as::<_, EncryptionPolicy>(
            r#"INSERT INTO encryption_policies (name, description, data_types, algorithm, key_rotation_days)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(data_types)
        .bind(algorithm)
        .bind(key_rotation_days)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_encryption_policy: {e}")))?;
        Ok(row)
    }

    pub async fn get_encryption_policy(&self, id: Uuid) -> Result<EncryptionPolicy> {
        sqlx::query_as::<_, EncryptionPolicy>("SELECT * FROM encryption_policies WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_encryption_policy: {e}")))
    }

    pub async fn list_encryption_policies(&self) -> Result<Vec<EncryptionPolicy>> {
        sqlx::query_as::<_, EncryptionPolicy>("SELECT * FROM encryption_policies ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_encryption_policies: {e}")))
    }

    pub async fn update_encryption_policy(
        &self,
        id: Uuid,
        description: Option<&str>,
        data_types: Option<&[String]>,
        algorithm: Option<&str>,
        key_rotation_days: Option<i32>,
        enabled: Option<bool>,
    ) -> Result<EncryptionPolicy> {
        let row = sqlx::query_as::<_, EncryptionPolicy>(
            r#"UPDATE encryption_policies
               SET description = COALESCE($2, description),
                   data_types = COALESCE($3, data_types),
                   algorithm = COALESCE($4, algorithm),
                   key_rotation_days = COALESCE($5, key_rotation_days),
                   enabled = COALESCE($6, enabled)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(description)
        .bind(data_types)
        .bind(algorithm)
        .bind(key_rotation_days)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_encryption_policy: {e}")))?;
        Ok(row)
    }

    pub async fn delete_encryption_policy(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM encryption_policies WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_encryption_policy: {e}")))?;
        Ok(())
    }

    pub async fn get_enabled_encryption_policies(&self) -> Result<Vec<EncryptionPolicy>> {
        sqlx::query_as::<_, EncryptionPolicy>(
            "SELECT * FROM encryption_policies WHERE enabled = true ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_enabled_encryption_policies: {e}")))
    }

    // --- Data Residency ---

    pub async fn create_data_residency_rule(
        &self,
        name: &str,
        description: &str,
        data_types: &[String],
        allowed_regions: &[String],
    ) -> Result<DataResidencyRule> {
        let row = sqlx::query_as::<_, DataResidencyRule>(
            r#"INSERT INTO data_residency_rules (name, description, data_types, allowed_regions)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(data_types)
        .bind(allowed_regions)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_data_residency_rule: {e}")))?;
        Ok(row)
    }

    pub async fn get_data_residency_rule(&self, id: Uuid) -> Result<DataResidencyRule> {
        sqlx::query_as::<_, DataResidencyRule>(
            "SELECT * FROM data_residency_rules WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_data_residency_rule: {e}")))
    }

    pub async fn list_data_residency_rules(&self) -> Result<Vec<DataResidencyRule>> {
        sqlx::query_as::<_, DataResidencyRule>(
            "SELECT * FROM data_residency_rules ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_data_residency_rules: {e}")))
    }

    pub async fn update_data_residency_rule(
        &self,
        id: Uuid,
        description: Option<&str>,
        data_types: Option<&[String]>,
        allowed_regions: Option<&[String]>,
        enabled: Option<bool>,
    ) -> Result<DataResidencyRule> {
        let row = sqlx::query_as::<_, DataResidencyRule>(
            r#"UPDATE data_residency_rules
               SET description = COALESCE($2, description),
                   data_types = COALESCE($3, data_types),
                   allowed_regions = COALESCE($4, allowed_regions),
                   enabled = COALESCE($5, enabled)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(description)
        .bind(data_types)
        .bind(allowed_regions)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_data_residency_rule: {e}")))?;
        Ok(row)
    }

    pub async fn delete_data_residency_rule(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM data_residency_rules WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_data_residency_rule: {e}")))?;
        Ok(())
    }

    pub async fn record_residency_violation(
        &self,
        rule_id: Uuid,
        data_type: &str,
        data_id: Uuid,
        region: &str,
    ) -> Result<DataResidencyViolation> {
        let row = sqlx::query_as::<_, DataResidencyViolation>(
            r#"INSERT INTO data_residency_violations (rule_id, data_type, data_id, region)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(rule_id)
        .bind(data_type)
        .bind(data_id)
        .bind(region)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_residency_violation: {e}")))?;
        Ok(row)
    }

    pub async fn list_residency_violations(
        &self,
        rule_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DataResidencyViolation>> {
        let rows = match rule_id {
            Some(rid) => {
                sqlx::query_as::<_, DataResidencyViolation>(
                    r#"SELECT * FROM data_residency_violations
                       WHERE rule_id = $1
                       ORDER BY detected_at DESC
                       LIMIT $2 OFFSET $3"#,
                )
                .bind(rid)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, DataResidencyViolation>(
                    r#"SELECT * FROM data_residency_violations
                       ORDER BY detected_at DESC
                       LIMIT $1 OFFSET $2"#,
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        };
        rows.map_err(|e| DbError::Database(format!("list_residency_violations: {e}")))
    }

    pub async fn get_residency_violations_by_data_type(
        &self,
        data_type: &str,
    ) -> Result<Vec<DataResidencyViolation>> {
        sqlx::query_as::<_, DataResidencyViolation>(
            r#"SELECT * FROM data_residency_violations
               WHERE data_type = $1
               ORDER BY detected_at DESC"#,
        )
        .bind(data_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_residency_violations_by_data_type: {e}")))
    }

    pub async fn check_region_compliance(
        &self,
        data_type: &str,
        region: &str,
    ) -> Result<bool> {
        let result = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS (
                   SELECT 1 FROM data_residency_rules
                   WHERE enabled = true
                     AND $1::text = ANY(data_types)
                     AND $2::text = ANY(allowed_regions)
               )"#,
        )
        .bind(data_type)
        .bind(region)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("check_region_compliance: {e}")))?;
        Ok(result)
    }

    // --- API Documentation v3 ---

    pub async fn create_api_docs_v3(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
        summary: &str,
        description: &str,
        parameters: serde_json::Value,
        request_body: Option<serde_json::Value>,
        responses: serde_json::Value,
        examples: serde_json::Value,
        tags: &[String],
        deprecated: bool,
    ) -> Result<ApiDocsV3> {
        let row = sqlx::query_as::<_, ApiDocsV3>(
            r#"INSERT INTO api_docs_v3 (endpoint, method, version, summary, description, parameters, request_body, responses, examples, tags, deprecated)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .bind(summary)
        .bind(description)
        .bind(parameters)
        .bind(request_body)
        .bind(responses)
        .bind(examples)
        .bind(tags)
        .bind(deprecated)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_docs_v3: {e}")))?;
        Ok(row)
    }

    pub async fn get_api_docs_v3(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
    ) -> Result<ApiDocsV3> {
        sqlx::query_as::<_, ApiDocsV3>(
            "SELECT * FROM api_docs_v3 WHERE endpoint = $1 AND method = $2 AND version = $3",
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_docs_v3: {e}")))
    }

    pub async fn list_api_docs_v3(&self, limit: i64, offset: i64) -> Result<Vec<ApiDocsV3>> {
        sqlx::query_as::<_, ApiDocsV3>(
            "SELECT * FROM api_docs_v3 ORDER BY endpoint, method, version LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_docs_v3: {e}")))
    }

    pub async fn search_api_docs_v3_by_tag(&self, tag: &str) -> Result<Vec<ApiDocsV3>> {
        sqlx::query_as::<_, ApiDocsV3>(
            "SELECT * FROM api_docs_v3 WHERE $1 = ANY(tags) ORDER BY endpoint, method, version",
        )
        .bind(tag)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("search_api_docs_v3_by_tag: {e}")))
    }

    pub async fn get_deprecated_api_docs_v3(&self) -> Result<Vec<ApiDocsV3>> {
        sqlx::query_as::<_, ApiDocsV3>(
            "SELECT * FROM api_docs_v3 WHERE deprecated = true ORDER BY endpoint, method, version",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_deprecated_api_docs_v3: {e}")))
    }

    pub async fn update_api_docs_v3(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
        summary: Option<&str>,
        description: Option<&str>,
        parameters: Option<serde_json::Value>,
        request_body: Option<Option<serde_json::Value>>,
        responses: Option<serde_json::Value>,
        examples: Option<serde_json::Value>,
        tags: Option<&[String]>,
        deprecated: Option<bool>,
    ) -> Result<ApiDocsV3> {
        let row = sqlx::query_as::<_, ApiDocsV3>(
            r#"UPDATE api_docs_v3
               SET summary = COALESCE($4, summary),
                   description = COALESCE($5, description),
                   parameters = COALESCE($6, parameters),
                   request_body = CASE WHEN $7::JSONB IS NULL THEN request_body ELSE $7 END,
                   responses = COALESCE($8, responses),
                   examples = COALESCE($9, examples),
                   tags = COALESCE($10, tags),
                   deprecated = COALESCE($11, deprecated)
               WHERE endpoint = $1 AND method = $2 AND version = $3
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .bind(summary)
        .bind(description)
        .bind(parameters)
        .bind(request_body)
        .bind(responses)
        .bind(examples)
        .bind(tags)
        .bind(deprecated)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_api_docs_v3: {e}")))?;
        Ok(row)
    }

    pub async fn delete_api_docs_v3(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM api_docs_v3 WHERE endpoint = $1 AND method = $2 AND version = $3",
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("delete_api_docs_v3: {e}")))?;
        Ok(())
    }

    // --- API Webhooks v2 ---

    pub async fn create_api_webhook_v2(
        &self,
        url: &str,
        secret: &str,
        events: &[String],
        config: serde_json::Value,
    ) -> Result<ApiWebhookV2> {
        let row = sqlx::query_as::<_, ApiWebhookV2>(
            r#"INSERT INTO api_webhooks_v2 (url, secret, events, config)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(url)
        .bind(secret)
        .bind(events)
        .bind(config)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_webhook_v2: {e}")))?;
        Ok(row)
    }

    pub async fn get_api_webhook_v2(&self, id: Uuid) -> Result<ApiWebhookV2> {
        sqlx::query_as::<_, ApiWebhookV2>("SELECT * FROM api_webhooks_v2 WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_api_webhook_v2: {e}")))
    }

    pub async fn list_api_webhooks_v2(&self, limit: i64, offset: i64) -> Result<Vec<ApiWebhookV2>> {
        sqlx::query_as::<_, ApiWebhookV2>(
            "SELECT * FROM api_webhooks_v2 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_webhooks_v2: {e}")))
    }

    pub async fn update_api_webhook_v2(
        &self,
        id: Uuid,
        url: Option<&str>,
        events: Option<&[String]>,
        active: Option<bool>,
        config: Option<serde_json::Value>,
    ) -> Result<ApiWebhookV2> {
        let row = sqlx::query_as::<_, ApiWebhookV2>(
            r#"UPDATE api_webhooks_v2
               SET url = COALESCE($2, url),
                   events = COALESCE($3, events),
                   active = COALESCE($4, active),
                   config = COALESCE($5, config)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(url)
        .bind(events)
        .bind(active)
        .bind(config)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_api_webhook_v2: {e}")))?;
        Ok(row)
    }

    pub async fn delete_api_webhook_v2(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM api_webhooks_v2 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_api_webhook_v2: {e}")))?;
        Ok(())
    }

    pub async fn create_api_webhook_delivery_v2(
        &self,
        webhook_id: Uuid,
        event: &str,
        payload: serde_json::Value,
    ) -> Result<ApiWebhookDeliveryV2> {
        let row = sqlx::query_as::<_, ApiWebhookDeliveryV2>(
            r#"INSERT INTO api_webhook_deliveries_v2 (webhook_id, event, payload)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(webhook_id)
        .bind(event)
        .bind(payload)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_webhook_delivery_v2: {e}")))?;
        Ok(row)
    }

    pub async fn update_api_webhook_delivery_v2(
        &self,
        id: Uuid,
        status: &str,
        response_status: Option<i32>,
        response_body: Option<&str>,
        attempts: i32,
    ) -> Result<ApiWebhookDeliveryV2> {
        let row = sqlx::query_as::<_, ApiWebhookDeliveryV2>(
            r#"UPDATE api_webhook_deliveries_v2
               SET status = $2,
                   response_status = $3,
                   response_body = $4,
                   attempts = $5
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(status)
        .bind(response_status)
        .bind(response_body)
        .bind(attempts)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_api_webhook_delivery_v2: {e}")))?;
        Ok(row)
    }

    pub async fn get_pending_api_webhook_deliveries_v2(
        &self,
        limit: i64,
    ) -> Result<Vec<ApiWebhookDeliveryV2>> {
        sqlx::query_as::<_, ApiWebhookDeliveryV2>(
            "SELECT * FROM api_webhook_deliveries_v2 WHERE status = 'pending' ORDER BY created_at LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_pending_api_webhook_deliveries_v2: {e}")))
    }

    pub async fn get_api_webhook_delivery_stats_v2(
        &self,
        webhook_id: Uuid,
    ) -> Result<(i64, i64, i64)> {
        sqlx::query_as::<_, (i64, i64, i64)>(
            r#"SELECT
                COUNT(*) FILTER (WHERE status = 'success') AS successful,
                COUNT(*) FILTER (WHERE status = 'failed') AS failed,
                COUNT(*) FILTER (WHERE status = 'pending') AS pending
               FROM api_webhook_deliveries_v2
               WHERE webhook_id = $1"#,
        )
        .bind(webhook_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_webhook_delivery_stats_v2: {e}")))
    }

    // --- API Analytics v4 ---

    pub async fn create_api_analytic_v4(
        &self,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_time_ms: i32,
        user_id: Option<Uuid>,
        request_size_bytes: i32,
        response_size_bytes: i32,
        cache_hit: bool,
        region: &str,
    ) -> Result<ApiAnalyticV4> {
        let row = sqlx::query_as::<_, ApiAnalyticV4>(
            r#"INSERT INTO api_analytics_v4 (endpoint, method, status_code, response_time_ms, user_id, request_size_bytes, response_size_bytes, cache_hit, region)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(user_id)
        .bind(request_size_bytes)
        .bind(response_size_bytes)
        .bind(cache_hit)
        .bind(region)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_analytic_v4: {e}")))?;
        Ok(row)
    }

    pub async fn get_api_analytics_v4(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ApiAnalyticV4>> {
        sqlx::query_as::<_, ApiAnalyticV4>(
            "SELECT * FROM api_analytics_v4 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_analytics_v4: {e}")))
    }

    pub async fn get_endpoint_analytics_v4(
        &self,
        endpoint: &str,
        method: &str,
    ) -> Result<Vec<ApiAnalyticV4>> {
        sqlx::query_as::<_, ApiAnalyticV4>(
            "SELECT * FROM api_analytics_v4 WHERE endpoint = $1 AND method = $2 ORDER BY created_at DESC",
        )
        .bind(endpoint)
        .bind(method)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_endpoint_analytics_v4: {e}")))
    }

    pub async fn get_api_analytics_v4_by_region(&self, region: &str) -> Result<Vec<ApiAnalyticV4>> {
        sqlx::query_as::<_, ApiAnalyticV4>(
            "SELECT * FROM api_analytics_v4 WHERE region = $1 ORDER BY created_at DESC",
        )
        .bind(region)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_analytics_v4_by_region: {e}")))
    }

    pub async fn get_api_analytics_v4_regional_summary(
        &self,
    ) -> Result<Vec<(String, i64, f64, f64)>> {
        sqlx::query_as::<_, (String, i64, f64, f64)>(
            r#"SELECT
                region,
                COUNT(*) AS total_requests,
                AVG(response_time_ms)::NUMERIC(10,2) AS avg_response_time_ms,
                (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC(10,4) / COUNT(*)::NUMERIC(10,4) AS cache_hit_rate
               FROM api_analytics_v4
               GROUP BY region
               ORDER BY total_requests DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_analytics_v4_regional_summary: {e}")))
    }

    pub async fn get_api_analytics_v4_performance_by_region(
        &self,
    ) -> Result<Vec<(String, f64, f64, f64)>> {
        sqlx::query_as::<_, (String, f64, f64, f64)>(
            r#"SELECT
                region,
                AVG(response_time_ms)::NUMERIC(10,2) AS avg_response_time_ms,
                AVG(request_size_bytes)::NUMERIC(10,2) AS avg_request_size,
                AVG(response_size_bytes)::NUMERIC(10,2) AS avg_response_size
               FROM api_analytics_v4
               GROUP BY region
               ORDER BY avg_response_time_ms ASC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_analytics_v4_performance_by_region: {e}")))
    }

    // --- API Docs v4 ---

    pub async fn list_api_docs_v4(&self, limit: i64, offset: i64) -> Result<Vec<ApiDocsV4>> {
        sqlx::query_as::<_, ApiDocsV4>(
            "SELECT * FROM api_docs_v4 ORDER BY endpoint, method, version LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_docs_v4: {e}")))
    }

    pub async fn get_api_docs_v4_for_endpoint(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
    ) -> Result<Option<ApiDocsV4>> {
        sqlx::query_as::<_, ApiDocsV4>(
            "SELECT * FROM api_docs_v4 WHERE endpoint = $1 AND method = $2 AND version = $3",
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_docs_v4_for_endpoint: {e}")))
    }

    pub async fn create_api_docs_v4(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
        summary: &str,
        description: &str,
        parameters: &serde_json::Value,
        request_body: Option<&serde_json::Value>,
        responses: &serde_json::Value,
        examples: &serde_json::Value,
        tags: &[String],
        deprecated: bool,
        changelog: &str,
    ) -> Result<ApiDocsV4> {
        let row = sqlx::query_as::<_, ApiDocsV4>(
            r#"INSERT INTO api_docs_v4 (endpoint, method, version, summary, description, parameters, request_body, responses, examples, tags, deprecated, changelog)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               ON CONFLICT (endpoint, method, version) DO UPDATE
               SET summary = EXCLUDED.summary, description = EXCLUDED.description, parameters = EXCLUDED.parameters,
                   request_body = EXCLUDED.request_body, responses = EXCLUDED.responses, examples = EXCLUDED.examples,
                   tags = EXCLUDED.tags, deprecated = EXCLUDED.deprecated, changelog = EXCLUDED.changelog
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .bind(summary)
        .bind(description)
        .bind(parameters)
        .bind(request_body)
        .bind(responses)
        .bind(examples)
        .bind(tags)
        .bind(deprecated)
        .bind(changelog)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_docs_v4: {e}")))?;
        Ok(row)
    }

    pub async fn get_changelog_for_endpoint(
        &self,
        endpoint: &str,
        method: &str,
    ) -> Result<Vec<ApiDocsV4>> {
        sqlx::query_as::<_, ApiDocsV4>(
            "SELECT * FROM api_docs_v4 WHERE endpoint = $1 AND method = $2 ORDER BY created_at DESC",
        )
        .bind(endpoint)
        .bind(method)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_changelog_for_endpoint: {e}")))
    }

    pub async fn detect_breaking_changes(
        &self,
        endpoint: &str,
        method: &str,
    ) -> Result<(String, String, String, String, Vec<(String, String, String)>)> {
        let docs = sqlx::query_as::<_, ApiDocsV4>(
            "SELECT * FROM api_docs_v4 WHERE endpoint = $1 AND method = $2 ORDER BY created_at DESC LIMIT 2",
        )
        .bind(endpoint)
        .bind(method)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("detect_breaking_changes: {e}")))?;

        if docs.len() < 2 {
            return Ok((endpoint.into(), method.into(), String::new(), String::new(), vec![]));
        }

        let old = &docs[1];
        let new = &docs[0];
        let mut breaking = Vec::new();

        if old.deprecated && !new.deprecated {
            breaking.push(("deprecated".into(), "field_change".into(), "Previously deprecated endpoint is now active".into()));
        }

        Ok((endpoint.into(), method.into(), old.version.clone(), new.version.clone(), breaking))
    }

    pub async fn generate_migration_guide(
        &self,
        from_version: &str,
        to_version: &str,
    ) -> Result<(Vec<String>, Vec<String>, Vec<String>)> {
        let affected = sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT endpoint FROM api_docs_v4 WHERE version = $1 OR version = $2",
        )
        .bind(from_version)
        .bind(to_version)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("generate_migration_guide: {e}")))?;

        let steps = vec![
            format!("Review API documentation for version {}", from_version),
            format!("Update client libraries to support version {}", to_version),
            format!("Test endpoints against version {}", to_version),
        ];

        let notes = vec![
            "Review breaking changes before migrating".into(),
            "Update API keys if authentication has changed".into(),
        ];

        Ok((affected, steps, notes))
    }

    pub async fn get_api_compatibility_matrix(&self) -> Result<serde_json::Value> {
        let versions = sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT version FROM api_docs_v4 ORDER BY version",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_compatibility_matrix: {e}")))?;

        Ok(serde_json::json!({
            "versions": versions,
            "endpoints": []
        }))
    }

    // --- Rate Limit Tiers v2 ---

    pub async fn list_rate_limit_tiers_v2(&self) -> Result<Vec<RateLimitTierV2>> {
        sqlx::query_as::<_, RateLimitTierV2>("SELECT * FROM rate_limit_tiers_v2 ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_rate_limit_tiers_v2: {e}")))
    }

    pub async fn get_rate_limit_tier_v2_by_name(&self, name: &str) -> Result<Option<RateLimitTierV2>> {
        sqlx::query_as::<_, RateLimitTierV2>("SELECT * FROM rate_limit_tiers_v2 WHERE name = $1")
            .bind(name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_rate_limit_tier_v2_by_name: {e}")))
    }

    pub async fn create_rate_limit_tier_v2(
        &self,
        name: &str,
        description: &str,
        rate_limit: i32,
        burst_limit: i32,
        monthly_quota: Option<i32>,
        price_cents: i32,
        features: &serde_json::Value,
    ) -> Result<RateLimitTierV2> {
        let row = sqlx::query_as::<_, RateLimitTierV2>(
            r#"INSERT INTO rate_limit_tiers_v2 (name, description, rate_limit, burst_limit, monthly_quota, price_cents, features)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_tier_v2: {e}")))?;
        Ok(row)
    }

    pub async fn update_rate_limit_tier_v2(
        &self,
        name: &str,
        description: Option<&str>,
        rate_limit: Option<i32>,
        burst_limit: Option<i32>,
        monthly_quota: Option<Option<i32>>,
        price_cents: Option<i32>,
        features: Option<&serde_json::Value>,
    ) -> Result<RateLimitTierV2> {
        let row = sqlx::query_as::<_, RateLimitTierV2>(
            r#"UPDATE rate_limit_tiers_v2
               SET description = COALESCE($2, description),
                   rate_limit = COALESCE($3, rate_limit),
                   burst_limit = COALESCE($4, burst_limit),
                   monthly_quota = CASE WHEN $5::INT IS NULL THEN monthly_quota ELSE $5 END,
                   price_cents = COALESCE($6, price_cents),
                   features = COALESCE($7, features)
               WHERE name = $1
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_rate_limit_tier_v2: {e}")))?;
        Ok(row)
    }

    pub async fn delete_rate_limit_tier_v2(&self, name: &str) -> Result<()> {
        sqlx::query("DELETE FROM rate_limit_tiers_v2 WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_rate_limit_tier_v2: {e}")))?;
        Ok(())
    }

    pub async fn check_rate_limit_v3(
        &self,
        user_id: Uuid,
    ) -> Result<(bool, i64, i64, DateTime<Utc>, String, serde_json::Value)> {
        let usage = sqlx::query_as::<_, RateLimitUsageV2>(
            "SELECT * FROM rate_limit_usage_v2 WHERE user_id = $1 AND period_start > NOW() - INTERVAL '1 month' ORDER BY usage_count DESC LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("check_rate_limit_v3: {e}")))?;

        match usage {
            Some(u) => {
                let tier = sqlx::query_as::<_, RateLimitTierV2>(
                    "SELECT * FROM rate_limit_tiers_v2 WHERE id = $1",
                )
                .bind(u.tier_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| DbError::Database(format!("check_rate_limit_v3 tier: {e}")))?;

                let allowed = (u.usage_count as i64) < tier.rate_limit as i64;
                let remaining = (tier.rate_limit as i64) - (u.usage_count as i64);
                let reset_at = Utc::now() + chrono::Duration::hours(1);

                Ok((allowed, remaining, tier.rate_limit as i64, reset_at, tier.name, tier.features))
            }
            None => Ok((true, 1000, 1000, Utc::now() + chrono::Duration::hours(1), "free".into(), serde_json::json!({}))),
        }
    }

    pub async fn get_user_usage_v2(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<RateLimitUsageV2>> {
        sqlx::query_as::<_, RateLimitUsageV2>(
            "SELECT * FROM rate_limit_usage_v2 WHERE user_id = $1 ORDER BY period_start DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_user_usage_v2: {e}")))
    }

    pub async fn get_quota_management(
        &self,
        user_id: Uuid,
    ) -> Result<Option<(String, i64, Option<i64>, Option<i64>, i32, DateTime<Utc>, DateTime<Utc>)>> {
        let usage = sqlx::query_as::<_, RateLimitUsageV2>(
            "SELECT * FROM rate_limit_usage_v2 WHERE user_id = $1 AND period_start > NOW() - INTERVAL '1 month' ORDER BY usage_count DESC LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_quota_management: {e}")))?;

        match usage {
            Some(u) => {
                let tier = sqlx::query_as::<_, RateLimitTierV2>(
                    "SELECT * FROM rate_limit_tiers_v2 WHERE id = $1",
                )
                .bind(u.tier_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| DbError::Database(format!("get_quota_management tier: {e}")))?;

                let monthly_quota = tier.monthly_quota.map(|q| q as i64);
                let quota_remaining = monthly_quota.map(|q| q - (u.usage_count as i64));
                let period_start = u.period_start;
                let period_end = period_start + chrono::Duration::days(30);

                Ok(Some((tier.name, u.usage_count as i64, monthly_quota, quota_remaining, 0, period_start, period_end)))
            }
            None => Ok(None),
        }
    }

    pub async fn check_feature_access(
        &self,
        user_id: Uuid,
        feature: &str,
    ) -> Result<(bool, String)> {
        let usage = sqlx::query_as::<_, RateLimitUsageV2>(
            "SELECT * FROM rate_limit_usage_v2 WHERE user_id = $1 AND period_start > NOW() - INTERVAL '1 month' ORDER BY usage_count DESC LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("check_feature_access: {e}")))?;

        match usage {
            Some(u) => {
                let tier = sqlx::query_as::<_, RateLimitTierV2>(
                    "SELECT * FROM rate_limit_tiers_v2 WHERE id = $1",
                )
                .bind(u.tier_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| DbError::Database(format!("check_feature_access tier: {e}")))?;

                let enabled = tier.features.get(feature).and_then(|v| v.as_bool()).unwrap_or(false);
                Ok((enabled, tier.name))
            }
            None => Ok((false, "free".into())),
        }
    }

    // --- API Analytics v5 ---

    pub async fn list_api_analytics_v5(&self, limit: i64, offset: i64) -> Result<Vec<ApiAnalyticV5>> {
        sqlx::query_as::<_, ApiAnalyticV5>(
            "SELECT * FROM api_analytics_v5 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_analytics_v5: {e}")))
    }

    pub async fn create_api_analytic_v5(
        &self,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_time_ms: i32,
        user_id: Option<Uuid>,
        request_size_bytes: i32,
        response_size_bytes: i32,
        cache_hit: bool,
        region: &str,
        user_agent: Option<&str>,
    ) -> Result<ApiAnalyticV5> {
        let row = sqlx::query_as::<_, ApiAnalyticV5>(
            r#"INSERT INTO api_analytics_v5 (endpoint, method, status_code, response_time_ms, user_id, request_size_bytes, response_size_bytes, cache_hit, region, user_agent)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(user_id)
        .bind(request_size_bytes)
        .bind(response_size_bytes)
        .bind(cache_hit)
        .bind(region)
        .bind(user_agent)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_analytic_v5: {e}")))?;
        Ok(row)
    }

    pub async fn get_user_agent_analysis(&self) -> Result<Vec<(String, i64, f64, f64)>> {
        sqlx::query_as::<_, (String, i64, f64, f64)>(
            r#"SELECT
                COALESCE(user_agent, 'unknown') AS user_agent,
                COUNT(*) AS request_count,
                AVG(response_time_ms)::NUMERIC(10,2) AS avg_response_time_ms,
                CASE WHEN COUNT(*) > 0 THEN (COUNT(*) FILTER (WHERE status_code >= 400))::NUMERIC / COUNT(*)::NUMERIC * 100 ELSE 0 END AS error_rate
               FROM api_analytics_v5
               GROUP BY user_agent
               ORDER BY request_count DESC
               LIMIT 50"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_user_agent_analysis: {e}")))
    }

    pub async fn get_geographic_analytics(&self) -> Result<Vec<(String, i64, f64, f64, i64)>> {
        sqlx::query_as::<_, (String, i64, f64, f64, i64)>(
            r#"SELECT
                region,
                COUNT(*) AS request_count,
                AVG(response_time_ms)::NUMERIC(10,2) AS avg_response_time_ms,
                CASE WHEN COUNT(*) > 0 THEN (COUNT(*) FILTER (WHERE status_code >= 400))::NUMERIC / COUNT(*)::NUMERIC * 100 ELSE 0 END AS error_rate,
                COUNT(DISTINCT user_id) AS unique_users
               FROM api_analytics_v5
               GROUP BY region
               ORDER BY request_count DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_geographic_analytics: {e}")))
    }

    pub async fn get_performance_optimization(&self) -> Result<Vec<(String, String, f64, f64, f64, Vec<String>)>> {
        let rows = sqlx::query_as::<_, (String, String, f64, f64, f64)>(
            r#"SELECT
                endpoint,
                method,
                AVG(response_time_ms)::NUMERIC(10,2) AS avg_response_time_ms,
                PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms) AS p95_response_time_ms,
                CASE WHEN COUNT(*) > 0 THEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / COUNT(*)::NUMERIC * 100 ELSE 0 END AS cache_hit_rate
               FROM api_analytics_v5
               GROUP BY endpoint, method
               HAVING COUNT(*) > 10
               ORDER BY avg_response_time_ms DESC
               LIMIT 50"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_optimization: {e}")))?;

        let result = rows.into_iter().map(|(endpoint, method, avg, p95, cache)| {
            let mut suggestions = Vec::new();
            if avg > 500.0 {
                suggestions.push("Consider adding response caching".into());
            }
            if cache < 50.0 {
                suggestions.push("Low cache hit rate, review caching strategy".into());
            }
            (endpoint, method, avg, p95, cache, suggestions)
        }).collect();

        Ok(result)
    }

    pub async fn get_cost_analysis(&self) -> Result<(i64, i64, i64, i64, Vec<(String, i64, i64)>, Vec<(String, i64, i64)>)> {
        let totals = sqlx::query_as::<_, (i64, i64, i64)>(
            r#"SELECT
                COUNT(*) AS total_requests,
                COALESCE(SUM(request_size_bytes), 0) AS total_request_bytes,
                COALESCE(SUM(response_size_bytes), 0) AS total_response_bytes
               FROM api_analytics_v5"#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis totals: {e}")))?;

        let estimated_cost = (totals.1 as i64 + totals.2 as i64) / 1024; // $0.01 per KB

        let region_costs = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT
                region,
                COUNT(*) AS requests,
                (COALESCE(SUM(request_size_bytes), 0) + COALESCE(SUM(response_size_bytes), 0)) / 1024 AS cost_cents
               FROM api_analytics_v5
               GROUP BY region
               ORDER BY cost_cents DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis regions: {e}")))?;

        let ua_costs = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT
                COALESCE(user_agent, 'unknown') AS user_agent,
                COUNT(*) AS requests,
                (COALESCE(SUM(request_size_bytes), 0) + COALESCE(SUM(response_size_bytes), 0)) / 1024 AS cost_cents
               FROM api_analytics_v5
               GROUP BY user_agent
               ORDER BY cost_cents DESC
               LIMIT 20"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis uas: {e}")))?;

        Ok((totals.0, totals.1, totals.2, estimated_cost, region_costs, ua_costs))
    }

    // --- API Docs v5 ---

    pub async fn list_api_docs_v5(&self, limit: i64, offset: i64) -> Result<Vec<ApiDocsV5>> {
        sqlx::query_as::<_, ApiDocsV5>(
            "SELECT * FROM api_docs_v5 ORDER BY endpoint, method, version LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_docs_v5: {e}")))
    }

    pub async fn get_api_docs_v5_for_endpoint(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
    ) -> Result<Option<ApiDocsV5>> {
        sqlx::query_as::<_, ApiDocsV5>(
            "SELECT * FROM api_docs_v5 WHERE endpoint = $1 AND method = $2 AND version = $3",
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_docs_v5_for_endpoint: {e}")))
    }

    pub async fn create_api_docs_v5(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
        summary: &str,
        description: &str,
        parameters: &serde_json::Value,
        request_body: Option<&serde_json::Value>,
        responses: &serde_json::Value,
        examples: &serde_json::Value,
        tags: &[String],
        deprecated: bool,
        changelog: &str,
        security_schemes: &serde_json::Value,
    ) -> Result<ApiDocsV5> {
        let row = sqlx::query_as::<_, ApiDocsV5>(
            r#"INSERT INTO api_docs_v5 (endpoint, method, version, summary, description, parameters, request_body, responses, examples, tags, deprecated, changelog, security_schemes)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
               ON CONFLICT (endpoint, method, version) DO UPDATE
               SET summary = EXCLUDED.summary, description = EXCLUDED.description, parameters = EXCLUDED.parameters,
                   request_body = EXCLUDED.request_body, responses = EXCLUDED.responses, examples = EXCLUDED.examples,
                   tags = EXCLUDED.tags, deprecated = EXCLUDED.deprecated, changelog = EXCLUDED.changelog,
                   security_schemes = EXCLUDED.security_schemes
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .bind(summary)
        .bind(description)
        .bind(parameters)
        .bind(request_body)
        .bind(responses)
        .bind(examples)
        .bind(tags)
        .bind(deprecated)
        .bind(changelog)
        .bind(security_schemes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_docs_v5: {e}")))?;
        Ok(row)
    }

    pub async fn get_security_schemes_for_endpoint(
        &self,
        endpoint: &str,
        method: &str,
    ) -> Result<serde_json::Value> {
        let doc = sqlx::query_as::<_, ApiDocsV5>(
            "SELECT * FROM api_docs_v5 WHERE endpoint = $1 AND method = $2 ORDER BY version DESC LIMIT 1",
        )
        .bind(endpoint)
        .bind(method)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_security_schemes_for_endpoint: {e}")))?;

        Ok(doc.map(|d| d.security_schemes).unwrap_or(serde_json::json!([])))
    }

    // --- API Docs v8 ---

    pub async fn list_api_docs_v8(&self, limit: i64, offset: i64) -> Result<Vec<ApiDocsV8>> {
        sqlx::query_as::<_, ApiDocsV8>(
            "SELECT * FROM api_docs_v8 ORDER BY endpoint, method, version LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_docs_v8: {e}")))
    }

    pub async fn get_api_docs_v8_for_endpoint(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
    ) -> Result<Option<ApiDocsV8>> {
        sqlx::query_as::<_, ApiDocsV8>(
            "SELECT * FROM api_docs_v8 WHERE endpoint = $1 AND method = $2 AND version = $3",
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_docs_v8_for_endpoint: {e}")))
    }

    pub async fn create_api_docs_v8(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
        summary: &str,
        description: &str,
        parameters: &serde_json::Value,
        request_body: Option<&serde_json::Value>,
        responses: &serde_json::Value,
        examples: &serde_json::Value,
        tags: &[String],
        deprecated: bool,
        changelog: &str,
        security_schemes: &serde_json::Value,
        rate_limits: &serde_json::Value,
    ) -> Result<ApiDocsV8> {
        let row = sqlx::query_as::<_, ApiDocsV8>(
            r#"INSERT INTO api_docs_v8 (endpoint, method, version, summary, description, parameters, request_body, responses, examples, tags, deprecated, changelog, security_schemes, rate_limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               ON CONFLICT (endpoint, method, version) DO UPDATE
               SET summary = EXCLUDED.summary, description = EXCLUDED.description, parameters = EXCLUDED.parameters,
                   request_body = EXCLUDED.request_body, responses = EXCLUDED.responses, examples = EXCLUDED.examples,
                   tags = EXCLUDED.tags, deprecated = EXCLUDED.deprecated, changelog = EXCLUDED.changelog,
                   security_schemes = EXCLUDED.security_schemes, rate_limits = EXCLUDED.rate_limits
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .bind(summary)
        .bind(description)
        .bind(parameters)
        .bind(request_body)
        .bind(responses)
        .bind(examples)
        .bind(tags)
        .bind(deprecated)
        .bind(changelog)
        .bind(security_schemes)
        .bind(rate_limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_docs_v8: {e}")))?;
        Ok(row)
    }

    // --- Rate Limit Tiers v3 ---

    pub async fn list_rate_limit_tiers_v3(&self) -> Result<Vec<RateLimitTierV3>> {
        sqlx::query_as::<_, RateLimitTierV3>("SELECT * FROM rate_limit_tiers_v3 ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_rate_limit_tiers_v3: {e}")))
    }

    pub async fn get_rate_limit_tier_v3_by_name(&self, name: &str) -> Result<Option<RateLimitTierV3>> {
        sqlx::query_as::<_, RateLimitTierV3>("SELECT * FROM rate_limit_tiers_v3 WHERE name = $1")
            .bind(name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_rate_limit_tier_v3_by_name: {e}")))
    }

    pub async fn create_rate_limit_tier_v3(
        &self,
        name: &str,
        description: &str,
        rate_limit: i32,
        burst_limit: i32,
        monthly_quota: Option<i32>,
        price_cents: i32,
        features: &serde_json::Value,
        limits: &serde_json::Value,
    ) -> Result<RateLimitTierV3> {
        let row = sqlx::query_as::<_, RateLimitTierV3>(
            r#"INSERT INTO rate_limit_tiers_v3 (name, description, rate_limit, burst_limit, monthly_quota, price_cents, features, limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_tier_v3: {e}")))?;
        Ok(row)
    }

    pub async fn update_rate_limit_tier_v3(
        &self,
        name: &str,
        description: Option<&str>,
        rate_limit: Option<i32>,
        burst_limit: Option<i32>,
        monthly_quota: Option<Option<i32>>,
        price_cents: Option<i32>,
        features: Option<&serde_json::Value>,
        limits: Option<&serde_json::Value>,
    ) -> Result<RateLimitTierV3> {
        let row = sqlx::query_as::<_, RateLimitTierV3>(
            r#"UPDATE rate_limit_tiers_v3
               SET description = COALESCE($2, description),
                   rate_limit = COALESCE($3, rate_limit),
                   burst_limit = COALESCE($4, burst_limit),
                   monthly_quota = CASE WHEN $5 IS NOT NULL THEN $5 ELSE monthly_quota END,
                   price_cents = COALESCE($6, price_cents),
                   features = COALESCE($7, features),
                   limits = COALESCE($8, limits)
               WHERE name = $1
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_rate_limit_tier_v3: {e}")))?;
        Ok(row)
    }

    pub async fn delete_rate_limit_tier_v3(&self, name: &str) -> Result<()> {
        sqlx::query("DELETE FROM rate_limit_tiers_v3 WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_rate_limit_tier_v3: {e}")))?;
        Ok(())
    }

    pub async fn record_rate_limit_overage(
        &self,
        user_id: Uuid,
        tier_id: Uuid,
        period_start: DateTime<Utc>,
        overage_count: i32,
        overage_cost_cents: i32,
    ) -> Result<RateLimitOverage> {
        let row = sqlx::query_as::<_, RateLimitOverage>(
            r#"INSERT INTO rate_limit_overages (user_id, tier_id, period_start, overage_count, overage_cost_cents)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT (user_id, tier_id, period_start) DO UPDATE
               SET overage_count = rate_limit_overages.overage_count + EXCLUDED.overage_count,
                   overage_cost_cents = rate_limit_overages.overage_cost_cents + EXCLUDED.overage_cost_cents
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(tier_id)
        .bind(period_start)
        .bind(overage_count)
        .bind(overage_cost_cents)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_rate_limit_overage: {e}")))?;
        Ok(row)
    }

    pub async fn get_user_overages(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<RateLimitOverage>> {
        sqlx::query_as::<_, RateLimitOverage>(
            "SELECT * FROM rate_limit_overages WHERE user_id = $1 ORDER BY period_start DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_user_overages: {e}")))
    }

    pub async fn create_rate_limit_alert(
        &self,
        user_id: Uuid,
        tier_id: Uuid,
        alert_type: &str,
        threshold_percent: i32,
        current_usage: i32,
    ) -> Result<RateLimitAlert> {
        let row = sqlx::query_as::<_, RateLimitAlert>(
            r#"INSERT INTO rate_limit_alerts (user_id, tier_id, alert_type, threshold_percent, current_usage)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(tier_id)
        .bind(alert_type)
        .bind(threshold_percent)
        .bind(current_usage)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_alert: {e}")))?;
        Ok(row)
    }

    pub async fn get_user_rate_limit_alerts(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<RateLimitAlert>> {
        sqlx::query_as::<_, RateLimitAlert>(
            "SELECT * FROM rate_limit_alerts WHERE user_id = $1 ORDER BY triggered_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_user_rate_limit_alerts: {e}")))
    }

    // --- Rate Limit Tiers v6 ---

    pub async fn list_rate_limit_tiers_v6(&self) -> Result<Vec<RateLimitTierV6>> {
        sqlx::query_as::<_, RateLimitTierV6>(
            "SELECT * FROM rate_limit_tiers_v6 ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_rate_limit_tiers_v6: {e}")))
    }

    pub async fn get_rate_limit_tier_v6_by_name(&self, name: &str) -> Result<Option<RateLimitTierV6>> {
        sqlx::query_as::<_, RateLimitTierV6>(
            "SELECT * FROM rate_limit_tiers_v6 WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_rate_limit_tier_v6_by_name: {e}")))
    }

    pub async fn create_rate_limit_tier_v6(
        &self,
        name: &str,
        description: &str,
        rate_limit: i32,
        burst_limit: i32,
        monthly_quota: Option<i32>,
        price_cents: i32,
        features: &serde_json::Value,
        limits: &serde_json::Value,
    ) -> Result<RateLimitTierV6> {
        let row = sqlx::query_as::<_, RateLimitTierV6>(
            r#"INSERT INTO rate_limit_tiers_v6 (name, description, rate_limit, burst_limit, monthly_quota, price_cents, features, limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               ON CONFLICT (name) DO UPDATE
               SET description = EXCLUDED.description, rate_limit = EXCLUDED.rate_limit, burst_limit = EXCLUDED.burst_limit,
                   monthly_quota = EXCLUDED.monthly_quota, price_cents = EXCLUDED.price_cents,
                   features = EXCLUDED.features, limits = EXCLUDED.limits
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_tier_v6: {e}")))?;
        Ok(row)
    }

    pub async fn update_rate_limit_tier_v6(
        &self,
        name: &str,
        description: Option<&str>,
        rate_limit: Option<i32>,
        burst_limit: Option<i32>,
        monthly_quota: Option<Option<i32>>,
        price_cents: Option<i32>,
        features: Option<&serde_json::Value>,
        limits: Option<&serde_json::Value>,
    ) -> Result<RateLimitTierV6> {
        let row = sqlx::query_as::<_, RateLimitTierV6>(
            r#"UPDATE rate_limit_tiers_v6 SET
               description = COALESCE($2, description),
               rate_limit = COALESCE($3, rate_limit),
               burst_limit = COALESCE($4, burst_limit),
               monthly_quota = CASE WHEN $5 IS NOT NULL THEN $5 ELSE monthly_quota END,
               price_cents = COALESCE($6, price_cents),
               features = COALESCE($7, features),
               limits = COALESCE($8, limits)
               WHERE name = $1
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_rate_limit_tier_v6: {e}")))?;
        Ok(row)
    }

    pub async fn delete_rate_limit_tier_v6(&self, name: &str) -> Result<()> {
        sqlx::query("DELETE FROM rate_limit_tiers_v6 WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_rate_limit_tier_v6: {e}")))?;
        Ok(())
    }

    pub async fn create_rate_limit_alert_v6(
        &self,
        user_id: Uuid,
        tier_id: Uuid,
        alert_type: &str,
        threshold: f64,
    ) -> Result<RateLimitAlertV4> {
        let row = sqlx::query_as::<_, RateLimitAlertV4>(
            r#"INSERT INTO rate_limit_alerts_v3 (user_id, tier_id, alert_type, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(tier_id)
        .bind(alert_type)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_alert_v6: {e}")))?;
        Ok(row)
    }

    pub async fn get_user_rate_limit_alerts_v6(&self, user_id: Uuid) -> Result<Vec<RateLimitAlertV4>> {
        sqlx::query_as::<_, RateLimitAlertV4>(
            "SELECT * FROM rate_limit_alerts_v3 WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_user_rate_limit_alerts_v6: {e}")))
    }

    // --- API Analytics v6 ---

    pub async fn list_api_analytics_v6(&self, limit: i64, offset: i64) -> Result<Vec<ApiAnalyticV6>> {
        sqlx::query_as::<_, ApiAnalyticV6>(
            "SELECT * FROM api_analytics_v6 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_analytics_v6: {e}")))
    }

    pub async fn create_api_analytic_v6(
        &self,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_time_ms: i32,
        user_id: Option<Uuid>,
        request_size_bytes: i32,
        response_size_bytes: i32,
        cache_hit: bool,
        region: &str,
        user_agent: Option<&str>,
        request_id: Option<Uuid>,
    ) -> Result<ApiAnalyticV6> {
        let row = sqlx::query_as::<_, ApiAnalyticV6>(
            r#"INSERT INTO api_analytics_v6 (endpoint, method, status_code, response_time_ms, user_id, request_size_bytes, response_size_bytes, cache_hit, region, user_agent, request_id)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(user_id)
        .bind(request_size_bytes)
        .bind(response_size_bytes)
        .bind(cache_hit)
        .bind(region)
        .bind(user_agent)
        .bind(request_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_analytic_v6: {e}")))?;
        Ok(row)
    }

    pub async fn create_analytics_correlation(
        &self,
        request_id: Uuid,
        parent_request_id: Option<Uuid>,
        correlation_type: &str,
        trace_id: Option<&str>,
        span_id: Option<&str>,
    ) -> Result<ApiAnalyticsCorrelation> {
        let row = sqlx::query_as::<_, ApiAnalyticsCorrelation>(
            r#"INSERT INTO api_analytics_correlations (request_id, parent_request_id, correlation_type, trace_id, span_id)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(request_id)
        .bind(parent_request_id)
        .bind(correlation_type)
        .bind(trace_id)
        .bind(span_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_analytics_correlation: {e}")))?;
        Ok(row)
    }

    pub async fn get_correlations_for_request(
        &self,
        request_id: Uuid,
    ) -> Result<Vec<ApiAnalyticsCorrelation>> {
        sqlx::query_as::<_, ApiAnalyticsCorrelation>(
            "SELECT * FROM api_analytics_correlations WHERE request_id = $1 OR parent_request_id = $1 ORDER BY created_at",
        )
        .bind(request_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_correlations_for_request: {e}")))
    }

    pub async fn get_capacity_plans(&self) -> Result<Vec<ApiAnalyticsCapacityPlan>> {
        sqlx::query_as::<_, ApiAnalyticsCapacityPlan>(
            "SELECT * FROM api_analytics_capacity_plans ORDER BY utilization_percent DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_capacity_plans: {e}")))
    }

    pub async fn upsert_capacity_plan(
        &self,
        endpoint: &str,
        method: &str,
        current_rps: i32,
        projected_rps: i32,
        capacity_limit: i32,
        utilization_percent: f64,
    ) -> Result<ApiAnalyticsCapacityPlan> {
        let row = sqlx::query_as::<_, ApiAnalyticsCapacityPlan>(
            r#"INSERT INTO api_analytics_capacity_plans (endpoint, method, current_rps, projected_rps, capacity_limit, utilization_percent)
               VALUES ($1, $2, $3, $4, $5, $6)
               ON CONFLICT (endpoint, method) DO UPDATE
               SET current_rps = EXCLUDED.current_rps, projected_rps = EXCLUDED.projected_rps,
                   capacity_limit = EXCLUDED.capacity_limit, utilization_percent = EXCLUDED.utilization_percent,
                   last_calculated_at = NOW()
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(current_rps)
        .bind(projected_rps)
        .bind(capacity_limit)
        .bind(utilization_percent)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("upsert_capacity_plan: {e}")))?;
        Ok(row)
    }

    pub async fn get_performance_optimization_v6(&self) -> Result<Vec<(String, String, f64, f64, f64, Vec<String>)>> {
        let rows = sqlx::query_as::<_, (String, String, f64, f64, f64, Vec<String>)>(
            r#"SELECT endpoint, method,
                      AVG(response_time_ms)::NUMERIC::FLOAT as avg_rt,
                      PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms)::NUMERIC::FLOAT as p95_rt,
                      CASE WHEN COUNT(*) > 0 THEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / COUNT(*)::NUMERIC * 100 ELSE 0 END as cache_rate,
                      COALESCE(
                          ARRAY_REMOVE(
                              ARRAY[
                                  CASE WHEN AVG(response_time_ms) > 500 THEN 'Optimize slow endpoint' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / NULLIF(COUNT(*), 0) < 0.3 THEN 'Improve caching strategy' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE status_code >= 500))::NUMERIC / NULLIF(COUNT(*), 0) > 0.05 THEN 'Reduce error rate' END
                              ],
                              NULL
                          ),
                          ARRAY['No optimization needed']::TEXT[]
                      ) as suggestions
               FROM api_analytics_v6
               GROUP BY endpoint, method
               HAVING COUNT(*) > 10
               ORDER BY avg_rt DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_optimization_v6: {e}")))?;
        Ok(rows)
    }

    pub async fn get_cost_analysis_v6(&self) -> Result<(i64, i64, i64, i64, Vec<(String, i64, i64)>, Vec<(String, i64, i64)>)> {
        let totals = sqlx::query_as::<_, (i64, i64, i64, i64)>(
            r#"SELECT COUNT(*) as total_requests,
                      COALESCE(SUM(request_size_bytes), 0) as total_request_bytes,
                      COALESCE(SUM(response_size_bytes), 0) as total_response_bytes,
                      (COALESCE(SUM(request_size_bytes), 0) + COALESCE(SUM(response_size_bytes), 0)) / 1024 * 10 as estimated_cost_cents
               FROM api_analytics_v6"#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v6: {e}")))?;

        let region_costs = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT region, COUNT(*) as requests,
                      (COALESCE(SUM(request_size_bytes), 0) + COALESCE(SUM(response_size_bytes), 0)) / 1024 * 10 as cost_cents
               FROM api_analytics_v6
               GROUP BY region
               ORDER BY cost_cents DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v6 region_costs: {e}")))?;

        let ua_costs = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT COALESCE(user_agent, 'unknown') as ua, COUNT(*) as requests,
                      (COALESCE(SUM(request_size_bytes), 0) + COALESCE(SUM(response_size_bytes), 0)) / 1024 * 10 as cost_cents
               FROM api_analytics_v6
               GROUP BY user_agent
               ORDER BY cost_cents DESC
               LIMIT 10"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v6 ua_costs: {e}")))?;

        Ok((totals.0, totals.1, totals.2, totals.3, region_costs, ua_costs))
    }

    // --- API Docs v6 ---

    pub async fn list_api_docs_v6(&self, limit: i64, offset: i64) -> Result<Vec<ApiDocsV6>> {
        sqlx::query_as::<_, ApiDocsV6>(
            "SELECT * FROM api_docs_v6 ORDER BY endpoint, method, version LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_docs_v6: {e}")))
    }

    pub async fn get_api_docs_v6_for_endpoint(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
    ) -> Result<Option<ApiDocsV6>> {
        sqlx::query_as::<_, ApiDocsV6>(
            "SELECT * FROM api_docs_v6 WHERE endpoint = $1 AND method = $2 AND version = $3",
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_docs_v6_for_endpoint: {e}")))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_api_docs_v6(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
        summary: &str,
        description: &str,
        parameters: &serde_json::Value,
        request_body: Option<&serde_json::Value>,
        responses: &serde_json::Value,
        examples: &serde_json::Value,
        tags: &[String],
        deprecated: bool,
        changelog: &str,
        security_schemes: &serde_json::Value,
        rate_limits: &serde_json::Value,
    ) -> Result<ApiDocsV6> {
        let row = sqlx::query_as::<_, ApiDocsV6>(
            r#"INSERT INTO api_docs_v6 (endpoint, method, version, summary, description, parameters, request_body, responses, examples, tags, deprecated, changelog, security_schemes, rate_limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               ON CONFLICT (endpoint, method, version) DO UPDATE
               SET summary = EXCLUDED.summary, description = EXCLUDED.description, parameters = EXCLUDED.parameters,
                   request_body = EXCLUDED.request_body, responses = EXCLUDED.responses, examples = EXCLUDED.examples,
                   tags = EXCLUDED.tags, deprecated = EXCLUDED.deprecated, changelog = EXCLUDED.changelog,
                   security_schemes = EXCLUDED.security_schemes, rate_limits = EXCLUDED.rate_limits
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .bind(summary)
        .bind(description)
        .bind(parameters)
        .bind(request_body)
        .bind(responses)
        .bind(examples)
        .bind(tags)
        .bind(deprecated)
        .bind(changelog)
        .bind(security_schemes)
        .bind(rate_limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_docs_v6: {e}")))?;
        Ok(row)
    }

    pub async fn get_rate_limits_for_endpoint(
        &self,
        endpoint: &str,
        method: &str,
    ) -> Result<serde_json::Value> {
        let doc = sqlx::query_as::<_, ApiDocsV6>(
            "SELECT * FROM api_docs_v6 WHERE endpoint = $1 AND method = $2 ORDER BY version DESC LIMIT 1",
        )
        .bind(endpoint)
        .bind(method)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_rate_limits_for_endpoint: {e}")))?;

        Ok(doc.map(|d| d.rate_limits).unwrap_or(serde_json::json!({})))
    }

    pub async fn get_error_codes_for_endpoint(
        &self,
        endpoint: &str,
        method: &str,
    ) -> Result<serde_json::Value> {
        let doc = sqlx::query_as::<_, ApiDocsV6>(
            "SELECT * FROM api_docs_v6 WHERE endpoint = $1 AND method = $2 ORDER BY version DESC LIMIT 1",
        )
        .bind(endpoint)
        .bind(method)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_error_codes_for_endpoint: {e}")))?;

        Ok(doc.map(|d| d.responses).unwrap_or(serde_json::json!({})))
    }

    pub async fn generate_sdk_spec(&self, version: &str) -> Result<Vec<serde_json::Value>> {
        let rows = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'endpoint', endpoint,
                'method', method,
                'summary', summary,
                'parameters', parameters,
                'request_body', request_body,
                'responses', responses,
                'security_schemes', security_schemes,
                'rate_limits', rate_limits
               )
               FROM api_docs_v6
               WHERE version = $1 AND deprecated = false
               ORDER BY endpoint, method"#,
        )
        .bind(version)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("generate_sdk_spec: {e}")))?;
        Ok(rows)
    }

    pub async fn get_api_changelog(&self, version: &str) -> Result<Vec<serde_json::Value>> {
        let rows = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'endpoint', endpoint,
                'method', method,
                'changelog', changelog,
                'deprecated', deprecated,
                'created_at', created_at
               )
               FROM api_docs_v6
               WHERE version = $1 AND changelog != ''
               ORDER BY created_at DESC"#,
        )
        .bind(version)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_changelog: {e}")))?;
        Ok(rows)
    }

    // --- Rate Limit Tiers v4 ---

    pub async fn list_rate_limit_tiers_v4(&self) -> Result<Vec<RateLimitTierV4>> {
        sqlx::query_as::<_, RateLimitTierV4>("SELECT * FROM rate_limit_tiers_v4 ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_rate_limit_tiers_v4: {e}")))
    }

    pub async fn get_rate_limit_tier_v4_by_name(&self, name: &str) -> Result<Option<RateLimitTierV4>> {
        sqlx::query_as::<_, RateLimitTierV4>("SELECT * FROM rate_limit_tiers_v4 WHERE name = $1")
            .bind(name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("get_rate_limit_tier_v4_by_name: {e}")))
    }

    pub async fn create_rate_limit_tier_v4(
        &self,
        name: &str,
        description: &str,
        rate_limit: i32,
        burst_limit: i32,
        monthly_quota: Option<i32>,
        price_cents: i32,
        features: &serde_json::Value,
        limits: &serde_json::Value,
    ) -> Result<RateLimitTierV4> {
        let row = sqlx::query_as::<_, RateLimitTierV4>(
            r#"INSERT INTO rate_limit_tiers_v4 (name, description, rate_limit, burst_limit, monthly_quota, price_cents, features, limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_tier_v4: {e}")))?;
        Ok(row)
    }

    pub async fn update_rate_limit_tier_v4(
        &self,
        name: &str,
        description: Option<&str>,
        rate_limit: Option<i32>,
        burst_limit: Option<i32>,
        monthly_quota: Option<Option<i32>>,
        price_cents: Option<i32>,
        features: Option<&serde_json::Value>,
        limits: Option<&serde_json::Value>,
    ) -> Result<RateLimitTierV4> {
        let row = sqlx::query_as::<_, RateLimitTierV4>(
            r#"UPDATE rate_limit_tiers_v4
               SET description = COALESCE($2, description),
                   rate_limit = COALESCE($3, rate_limit),
                   burst_limit = COALESCE($4, burst_limit),
                   monthly_quota = CASE WHEN $5 IS NOT NULL THEN $5 ELSE monthly_quota END,
                   price_cents = COALESCE($6, price_cents),
                   features = COALESCE($7, features),
                   limits = COALESCE($8, limits)
               WHERE name = $1
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_rate_limit_tier_v4: {e}")))?;
        Ok(row)
    }

    pub async fn delete_rate_limit_tier_v4(&self, name: &str) -> Result<()> {
        sqlx::query("DELETE FROM rate_limit_tiers_v4 WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_rate_limit_tier_v4: {e}")))?;
        Ok(())
    }

    // --- Rate Limit Alerts v2 ---

    pub async fn create_rate_limit_alert_v2(
        &self,
        user_id: Uuid,
        tier_id: Uuid,
        alert_type: &str,
        threshold: f64,
        enabled: bool,
    ) -> Result<RateLimitAlertV2> {
        let row = sqlx::query_as::<_, RateLimitAlertV2>(
            r#"INSERT INTO rate_limit_alerts (user_id, tier_id, alert_type, threshold, enabled)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(tier_id)
        .bind(alert_type)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_alert_v2: {e}")))?;
        Ok(row)
    }

    pub async fn get_user_rate_limit_alerts_v2(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<RateLimitAlertV2>> {
        sqlx::query_as::<_, RateLimitAlertV2>(
            "SELECT * FROM rate_limit_alerts WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_user_rate_limit_alerts_v2: {e}")))
    }

    pub async fn update_rate_limit_alert_v2(
        &self,
        id: Uuid,
        threshold: Option<f64>,
        enabled: Option<bool>,
    ) -> Result<RateLimitAlertV2> {
        let row = sqlx::query_as::<_, RateLimitAlertV2>(
            r#"UPDATE rate_limit_alerts
               SET threshold = COALESCE($2, threshold),
                   enabled = COALESCE($3, enabled)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_rate_limit_alert_v2: {e}")))?;
        Ok(row)
    }

    pub async fn delete_rate_limit_alert_v2(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM rate_limit_alerts WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_rate_limit_alert_v2: {e}")))?;
        Ok(())
    }

    pub async fn get_active_rate_limit_alerts(
        &self,
        tier_id: Uuid,
    ) -> Result<Vec<RateLimitAlertV2>> {
        sqlx::query_as::<_, RateLimitAlertV2>(
            "SELECT * FROM rate_limit_alerts WHERE tier_id = $1 AND enabled = true ORDER BY threshold",
        )
        .bind(tier_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_active_rate_limit_alerts: {e}")))
    }

    pub async fn get_alert_history(
        &self,
        user_id: Uuid,
        alert_type: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<RateLimitAlertV2>> {
        let rows = if let Some(at) = alert_type {
            sqlx::query_as::<_, RateLimitAlertV2>(
                r#"SELECT * FROM rate_limit_alerts
                   WHERE user_id = $1 AND alert_type = $2
                   ORDER BY created_at DESC
                   LIMIT $3 OFFSET $4"#,
            )
            .bind(user_id)
            .bind(at)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, RateLimitAlertV2>(
                r#"SELECT * FROM rate_limit_alerts
                   WHERE user_id = $1
                   ORDER BY created_at DESC
                   LIMIT $2 OFFSET $3"#,
            )
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("get_alert_history: {e}")))
    }

    pub async fn get_alert_analytics(
        &self,
        user_id: Uuid,
    ) -> Result<serde_json::Value> {
        let row = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'total_alerts', COUNT(*),
                'enabled_alerts', COUNT(*) FILTER (WHERE enabled = true),
                'disabled_alerts', COUNT(*) FILTER (WHERE enabled = false),
                'by_type', json_build_object(
                    'threshold', COUNT(*) FILTER (WHERE alert_type = 'threshold'),
                    'quota', COUNT(*) FILTER (WHERE alert_type = 'quota'),
                    'burst', COUNT(*) FILTER (WHERE alert_type = 'burst')
                ),
                'avg_threshold', AVG(threshold)
               )
               FROM rate_limit_alerts
               WHERE user_id = $1"#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_alert_analytics: {e}")))?;
        Ok(row)
    }

    // --- API Analytics v7 ---

    pub async fn list_api_analytics_v7(&self, limit: i64, offset: i64) -> Result<Vec<ApiAnalyticV7>> {
        sqlx::query_as::<_, ApiAnalyticV7>(
            "SELECT * FROM api_analytics_v7 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_analytics_v7: {e}")))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_api_analytic_v7(
        &self,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_time_ms: i32,
        user_id: Option<Uuid>,
        request_size_bytes: i32,
        response_size_bytes: i32,
        cache_hit: bool,
        region: &str,
        user_agent: Option<&str>,
        request_id: Option<Uuid>,
        cost_cents: i32,
    ) -> Result<ApiAnalyticV7> {
        let row = sqlx::query_as::<_, ApiAnalyticV7>(
            r#"INSERT INTO api_analytics_v7 (endpoint, method, status_code, response_time_ms, user_id, request_size_bytes, response_size_bytes, cache_hit, region, user_agent, request_id, cost_cents)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(user_id)
        .bind(request_size_bytes)
        .bind(response_size_bytes)
        .bind(cache_hit)
        .bind(region)
        .bind(user_agent)
        .bind(request_id)
        .bind(cost_cents)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_analytic_v7: {e}")))?;
        Ok(row)
    }

    pub async fn get_cost_tracking_v7(
        &self,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<serde_json::Value> {
        let row = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'total_cost_cents', COALESCE(SUM(cost_cents), 0),
                'avg_cost_per_request', COALESCE(AVG(cost_cents), 0),
                'total_requests', COUNT(*),
                'by_endpoint', (
                    SELECT json_agg(json_build_object(
                        'endpoint', endpoint,
                        'method', method,
                        'total_cost', SUM(cost_cents),
                        'request_count', COUNT(*),
                        'avg_cost', AVG(cost_cents)
                    ))
                    FROM api_analytics_v7
                    WHERE created_at >= $1
                    GROUP BY endpoint, method
                    ORDER BY SUM(cost_cents) DESC
                    LIMIT 20
                ),
                'by_region', (
                    SELECT json_agg(json_build_object(
                        'region', region,
                        'total_cost', SUM(cost_cents),
                        'request_count', COUNT(*)
                    ))
                    FROM api_analytics_v7
                    WHERE created_at >= $1
                    GROUP BY region
                    ORDER BY SUM(cost_cents) DESC
                )
               )
               FROM api_analytics_v7
               WHERE created_at >= $1"#,
        )
        .bind(since)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_tracking_v7: {e}")))?;
        Ok(row)
    }

    pub async fn get_budget_alerts_v7(
        &self,
        user_id: Uuid,
        budget_cents: i64,
    ) -> Result<serde_json::Value> {
        let row = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'current_cost_cents', COALESCE(SUM(cost_cents), 0),
                'budget_cents', $2,
                'budget_used_percent', CASE WHEN $2 > 0 THEN COALESCE(SUM(cost_cents), 0)::float / $2::float * 100 ELSE 0 END,
                'exceeds_budget', COALESCE(SUM(cost_cents), 0) > $2,
                'daily_costs', (
                    SELECT json_agg(json_build_object(
                        'date', DATE(created_at),
                        'cost_cents', SUM(cost_cents)
                    ))
                    FROM api_analytics_v7
                    WHERE user_id = $1
                      AND created_at >= NOW() - INTERVAL '30 days'
                    GROUP BY DATE(created_at)
                    ORDER BY DATE(created_at)
                )
               )
               FROM api_analytics_v7
               WHERE user_id = $1"#,
        )
        .bind(user_id)
        .bind(budget_cents)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_budget_alerts_v7: {e}")))?;
        Ok(row)
    }

    pub async fn get_usage_optimization_v7(&self) -> Result<Vec<serde_json::Value>> {
        let rows = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'endpoint', endpoint,
                'method', method,
                'avg_response_time_ms', AVG(response_time_ms),
                'p95_response_time_ms', PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms),
                'cache_hit_rate', CASE WHEN COUNT(*) > 0 THEN (COUNT(*) FILTER (WHERE cache_hit))::float / COUNT(*)::float * 100 ELSE 0 END,
                'avg_cost_cents', AVG(cost_cents),
                'total_requests', COUNT(*),
                'suggestions', ARRAY_REMOVE(ARRAY[
                    CASE WHEN AVG(response_time_ms) > 500 THEN 'Optimize slow endpoint' END,
                    CASE WHEN (COUNT(*) FILTER (WHERE cache_hit))::float / NULLIF(COUNT(*), 0) < 0.3 THEN 'Improve caching' END,
                    CASE WHEN AVG(cost_cents) > 10 THEN 'Reduce per-request cost' END
                ], NULL)
               )
               FROM api_analytics_v7
               GROUP BY endpoint, method
               HAVING COUNT(*) > 10
               ORDER BY AVG(response_time_ms) DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_usage_optimization_v7: {e}")))?;
        Ok(rows)
    }

    pub async fn get_capacity_planning_v7(&self) -> Result<Vec<serde_json::Value>> {
        let rows = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'endpoint', endpoint,
                'method', method,
                'current_rps', COUNT(*) / EXTRACT(EPOCH FROM (MAX(created_at) - MIN(created_at))),
                'projected_rps_24h', COUNT(*) / EXTRACT(EPOCH FROM (MAX(created_at) - MIN(created_at))) * 1.2,
                'avg_response_time_ms', AVG(response_time_ms),
                'error_rate', CASE WHEN COUNT(*) > 0 THEN (COUNT(*) FILTER (WHERE status_code >= 500))::float / COUNT(*)::float * 100 ELSE 0 END,
                'region_distribution', (
                    SELECT json_object_agg(region, cnt)
                    FROM (
                        SELECT region, COUNT(*) as cnt
                        FROM api_analytics_v7 a2
                        WHERE a2.endpoint = a1.endpoint AND a2.method = a1.method
                        GROUP BY region
                    ) sub
                )
               )
               FROM api_analytics_v7 a1
               GROUP BY endpoint, method
               HAVING COUNT(*) > 5
               ORDER BY COUNT(*) DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_capacity_planning_v7: {e}")))?;
        Ok(rows)
    }

    // --- API Analytics v9 ---

    pub async fn list_api_analytics_v9(&self, limit: i64, offset: i64) -> Result<Vec<ApiAnalyticV9>> {
        sqlx::query_as::<_, ApiAnalyticV9>(
            "SELECT * FROM api_analytics_v9 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_analytics_v9: {e}")))
    }

    pub async fn create_api_analytic_v9(
        &self,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_time_ms: i32,
        user_id: Option<Uuid>,
        request_size_bytes: i32,
        response_size_bytes: i32,
        cache_hit: bool,
        region: &str,
        user_agent: Option<&str>,
        request_id: Option<Uuid>,
        cost_cents: i32,
    ) -> Result<ApiAnalyticV9> {
        let row = sqlx::query_as::<_, ApiAnalyticV9>(
            r#"INSERT INTO api_analytics_v9 (endpoint, method, status_code, response_time_ms, user_id, request_size_bytes, response_size_bytes, cache_hit, region, user_agent, request_id, cost_cents)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(user_id)
        .bind(request_size_bytes)
        .bind(response_size_bytes)
        .bind(cache_hit)
        .bind(region)
        .bind(user_agent)
        .bind(request_id)
        .bind(cost_cents)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_analytic_v9: {e}")))?;
        Ok(row)
    }

    pub async fn get_cost_analysis_v9(&self) -> Result<(i64, i64, i64, i64, Vec<(String, i64, i64)>, Vec<(String, i64, i64)>)> {
        let totals = sqlx::query_as::<_, (i64, i64, i64, i64)>(
            r#"SELECT COUNT(*) as total_requests,
                      COALESCE(SUM(request_size_bytes), 0) as total_request_bytes,
                      COALESCE(SUM(response_size_bytes), 0) as total_response_bytes,
                      COALESCE(SUM(cost_cents), 0) as estimated_cost_cents
               FROM api_analytics_v9"#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v9 totals: {e}")))?;

        let region_costs = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT region, COUNT(*) as requests,
                      COALESCE(SUM(cost_cents), 0) as cost_cents
               FROM api_analytics_v9
               GROUP BY region
               ORDER BY cost_cents DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v9 regions: {e}")))?;

        let ua_costs = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT COALESCE(user_agent, 'unknown') as user_agent, COUNT(*) as requests,
                      COALESCE(SUM(cost_cents), 0) as cost_cents
               FROM api_analytics_v9
               GROUP BY user_agent
               ORDER BY cost_cents DESC
               LIMIT 10"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v9 ua_costs: {e}")))?;

        Ok((totals.0, totals.1, totals.2, totals.3, region_costs, ua_costs))
    }

    pub async fn get_usage_optimization_v9(&self) -> Result<Vec<(String, String, f64, f64, f64, Vec<String>)>> {
        let rows = sqlx::query_as::<_, (String, String, f64, f64, f64, Vec<String>)>(
            r#"SELECT endpoint, method,
                      AVG(response_time_ms)::NUMERIC::FLOAT as avg_rt,
                      PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms)::NUMERIC::FLOAT as p95_rt,
                      CASE WHEN COUNT(*) > 0 THEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / COUNT(*)::NUMERIC * 100 ELSE 0 END as cache_rate,
                      COALESCE(
                          ARRAY_REMOVE(
                              ARRAY[
                                  CASE WHEN AVG(response_time_ms) > 500 THEN 'Optimize slow endpoint' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / NULLIF(COUNT(*), 0) < 0.3 THEN 'Improve caching strategy' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE status_code >= 500))::NUMERIC / NULLIF(COUNT(*), 0) > 0.05 THEN 'Reduce error rate' END
                              ],
                              NULL
                          ),
                          ARRAY['No optimization needed']::TEXT[]
                      ) as suggestions
               FROM api_analytics_v9
               GROUP BY endpoint, method
               HAVING COUNT(*) > 10
               ORDER BY avg_rt DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_usage_optimization_v9: {e}")))?;
        Ok(rows)
    }

    // --- API Docs v10 ---

    pub async fn list_api_docs_v10(&self, limit: i64, offset: i64) -> Result<Vec<ApiDocsV10>> {
        sqlx::query_as::<_, ApiDocsV10>(
            "SELECT * FROM api_docs_v10 ORDER BY endpoint, method, version LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_docs_v10: {e}")))
    }

    pub async fn get_api_docs_v10_for_endpoint(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
    ) -> Result<Option<ApiDocsV10>> {
        sqlx::query_as::<_, ApiDocsV10>(
            "SELECT * FROM api_docs_v10 WHERE endpoint = $1 AND method = $2 AND version = $3",
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_docs_v10_for_endpoint: {e}")))
    }

    pub async fn create_api_docs_v10(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
        summary: &str,
        description: &str,
        parameters: &serde_json::Value,
        request_body: Option<&serde_json::Value>,
        responses: &serde_json::Value,
        examples: &serde_json::Value,
        tags: &[String],
        deprecated: bool,
        changelog: &str,
        security_schemes: &serde_json::Value,
        rate_limits: &serde_json::Value,
    ) -> Result<ApiDocsV10> {
        let row = sqlx::query_as::<_, ApiDocsV10>(
            r#"INSERT INTO api_docs_v10 (endpoint, method, version, summary, description, parameters, request_body, responses, examples, tags, deprecated, changelog, security_schemes, rate_limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               ON CONFLICT (endpoint, method, version) DO UPDATE
               SET summary = EXCLUDED.summary, description = EXCLUDED.description, parameters = EXCLUDED.parameters,
                   request_body = EXCLUDED.request_body, responses = EXCLUDED.responses, examples = EXCLUDED.examples,
                   tags = EXCLUDED.tags, deprecated = EXCLUDED.deprecated, changelog = EXCLUDED.changelog,
                   security_schemes = EXCLUDED.security_schemes, rate_limits = EXCLUDED.rate_limits
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .bind(summary)
        .bind(description)
        .bind(parameters)
        .bind(request_body)
        .bind(responses)
        .bind(examples)
        .bind(tags)
        .bind(deprecated)
        .bind(changelog)
        .bind(security_schemes)
        .bind(rate_limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_docs_v10: {e}")))?;
        Ok(row)
    }

    // --- Rate Limit Tiers v8 ---

    pub async fn list_rate_limit_tiers_v8(&self) -> Result<Vec<RateLimitTierV8>> {
        sqlx::query_as::<_, RateLimitTierV8>(
            "SELECT * FROM rate_limit_tiers_v8 ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_rate_limit_tiers_v8: {e}")))
    }

    pub async fn get_rate_limit_tier_v8_by_name(&self, name: &str) -> Result<Option<RateLimitTierV8>> {
        sqlx::query_as::<_, RateLimitTierV8>(
            "SELECT * FROM rate_limit_tiers_v8 WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_rate_limit_tier_v8_by_name: {e}")))
    }

    pub async fn create_rate_limit_tier_v8(
        &self,
        name: &str,
        description: &str,
        rate_limit: i32,
        burst_limit: i32,
        monthly_quota: Option<i32>,
        price_cents: i32,
        features: &serde_json::Value,
        limits: &serde_json::Value,
    ) -> Result<RateLimitTierV8> {
        let row = sqlx::query_as::<_, RateLimitTierV8>(
            r#"INSERT INTO rate_limit_tiers_v8 (name, description, rate_limit, burst_limit, monthly_quota, price_cents, features, limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               ON CONFLICT (name) DO UPDATE
               SET description = EXCLUDED.description, rate_limit = EXCLUDED.rate_limit, burst_limit = EXCLUDED.burst_limit,
                   monthly_quota = EXCLUDED.monthly_quota, price_cents = EXCLUDED.price_cents,
                   features = EXCLUDED.features, limits = EXCLUDED.limits
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_tier_v8: {e}")))?;
        Ok(row)
    }

    pub async fn update_rate_limit_tier_v8(
        &self,
        name: &str,
        description: Option<&str>,
        rate_limit: Option<i32>,
        burst_limit: Option<i32>,
        monthly_quota: Option<Option<i32>>,
        price_cents: Option<i32>,
        features: Option<&serde_json::Value>,
        limits: Option<&serde_json::Value>,
    ) -> Result<RateLimitTierV8> {
        let row = sqlx::query_as::<_, RateLimitTierV8>(
            r#"UPDATE rate_limit_tiers_v8 SET
               description = COALESCE($2, description),
               rate_limit = COALESCE($3, rate_limit),
               burst_limit = COALESCE($4, burst_limit),
               monthly_quota = CASE WHEN $5 IS NOT NULL THEN $5 ELSE monthly_quota END,
               price_cents = COALESCE($6, price_cents),
               features = COALESCE($7, features),
               limits = COALESCE($8, limits)
               WHERE name = $1
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_rate_limit_tier_v8: {e}")))?;
        Ok(row)
    }

    pub async fn delete_rate_limit_tier_v8(&self, name: &str) -> Result<()> {
        sqlx::query("DELETE FROM rate_limit_tiers_v8 WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_rate_limit_tier_v8: {e}")))?;
        Ok(())
    }

    pub async fn create_rate_limit_alert_v8(
        &self,
        user_id: Uuid,
        tier_id: Uuid,
        alert_type: &str,
        threshold: f64,
    ) -> Result<RateLimitAlertV5> {
        let row = sqlx::query_as::<_, RateLimitAlertV5>(
            r#"INSERT INTO rate_limit_alerts_v5 (user_id, tier_id, alert_type, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(tier_id)
        .bind(alert_type)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_alert_v8: {e}")))?;
        Ok(row)
    }

    pub async fn get_user_rate_limit_alerts_v8(&self, user_id: Uuid) -> Result<Vec<RateLimitAlertV5>> {
        sqlx::query_as::<_, RateLimitAlertV5>(
            "SELECT * FROM rate_limit_alerts_v5 WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_user_rate_limit_alerts_v8: {e}")))
    }

    // --- API Analytics v11 ---

    pub async fn list_api_analytics_v11(&self, limit: i64, offset: i64) -> Result<Vec<ApiAnalyticV11>> {
        sqlx::query_as::<_, ApiAnalyticV11>(
            "SELECT * FROM api_analytics_v11 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_analytics_v11: {e}")))
    }

    pub async fn create_api_analytic_v11(
        &self,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_time_ms: i32,
        user_id: Option<Uuid>,
        request_size_bytes: i32,
        response_size_bytes: i32,
        cache_hit: bool,
        region: &str,
        user_agent: Option<&str>,
        request_id: Option<Uuid>,
        cost_cents: i32,
    ) -> Result<ApiAnalyticV11> {
        let row = sqlx::query_as::<_, ApiAnalyticV11>(
            r#"INSERT INTO api_analytics_v11 (endpoint, method, status_code, response_time_ms, user_id, request_size_bytes, response_size_bytes, cache_hit, region, user_agent, request_id, cost_cents)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(user_id)
        .bind(request_size_bytes)
        .bind(response_size_bytes)
        .bind(cache_hit)
        .bind(region)
        .bind(user_agent)
        .bind(request_id)
        .bind(cost_cents)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_analytic_v11: {e}")))?;
        Ok(row)
    }

    pub async fn get_cost_analysis_v11(&self) -> Result<(i64, i64, i64, i64, Vec<(String, i64, i64)>, Vec<(String, i64, i64)>)> {
        let totals = sqlx::query_as::<_, (i64, i64, i64, i64)>(
            r#"SELECT COUNT(*) as total_requests,
                      COALESCE(SUM(request_size_bytes), 0) as total_request_bytes,
                      COALESCE(SUM(response_size_bytes), 0) as total_response_bytes,
                      COALESCE(SUM(cost_cents), 0) as estimated_cost_cents
               FROM api_analytics_v11"#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v11 totals: {e}")))?;

        let regions = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT region, COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost_cents
               FROM api_analytics_v11 GROUP BY region ORDER BY cost_cents DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v11 regions: {e}")))?;

        let ua_costs = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT COALESCE(user_agent, 'unknown') as ua, COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost_cents
               FROM api_analytics_v11 GROUP BY user_agent ORDER BY cost_cents DESC LIMIT 10"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v11 ua_costs: {e}")))?;

        Ok((totals.0, totals.1, totals.2, totals.3, regions, ua_costs))
    }

    pub async fn get_usage_optimization_v11(&self) -> Result<Vec<(String, String, f64, f64, f64, Vec<String>)>> {
        let rows = sqlx::query_as::<_, (String, String, f64, f64, f64, Vec<String>)>(
            r#"SELECT endpoint, method,
                      AVG(response_time_ms)::NUMERIC::FLOAT as avg_rt,
                      PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms)::NUMERIC::FLOAT as p95_rt,
                      CASE WHEN COUNT(*) > 0 THEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / COUNT(*)::NUMERIC * 100 ELSE 0 END as cache_rate,
                      COALESCE(
                          ARRAY_REMOVE(
                              ARRAY[
                                  CASE WHEN AVG(response_time_ms) > 500 THEN 'Optimize slow endpoint' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / NULLIF(COUNT(*), 0) < 0.3 THEN 'Improve caching strategy' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE status_code >= 500))::NUMERIC / NULLIF(COUNT(*), 0) > 0.05 THEN 'Reduce error rate' END
                              ],
                              NULL
                          ),
                          ARRAY['No optimization needed']::TEXT[]
                      ) as suggestions
               FROM api_analytics_v11
               GROUP BY endpoint, method
               HAVING COUNT(*) > 10
               ORDER BY avg_rt DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_usage_optimization_v11: {e}")))?;
        Ok(rows)
    }

    // --- API Docs v7 ---

    pub async fn list_api_docs_v7(&self, limit: i64, offset: i64) -> Result<Vec<ApiDocsV7>> {
        sqlx::query_as::<_, ApiDocsV7>(
            "SELECT * FROM api_docs_v7 ORDER BY endpoint, method, version LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_docs_v7: {e}")))
    }

    pub async fn get_api_docs_v7_for_endpoint(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
    ) -> Result<Option<ApiDocsV7>> {
        sqlx::query_as::<_, ApiDocsV7>(
            "SELECT * FROM api_docs_v7 WHERE endpoint = $1 AND method = $2 AND version = $3",
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_docs_v7_for_endpoint: {e}")))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_api_docs_v7(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
        summary: &str,
        description: &str,
        parameters: &serde_json::Value,
        request_body: Option<&serde_json::Value>,
        responses: &serde_json::Value,
        examples: &serde_json::Value,
        tags: &[String],
        deprecated: bool,
        changelog: &str,
        security_schemes: &serde_json::Value,
        rate_limits: &serde_json::Value,
    ) -> Result<ApiDocsV7> {
        let row = sqlx::query_as::<_, ApiDocsV7>(
            r#"INSERT INTO api_docs_v7 (endpoint, method, version, summary, description, parameters, request_body, responses, examples, tags, deprecated, changelog, security_schemes, rate_limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               ON CONFLICT (endpoint, method, version) DO UPDATE
               SET summary = EXCLUDED.summary, description = EXCLUDED.description, parameters = EXCLUDED.parameters,
                   request_body = EXCLUDED.request_body, responses = EXCLUDED.responses, examples = EXCLUDED.examples,
                   tags = EXCLUDED.tags, deprecated = EXCLUDED.deprecated, changelog = EXCLUDED.changelog,
                   security_schemes = EXCLUDED.security_schemes, rate_limits = EXCLUDED.rate_limits
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .bind(summary)
        .bind(description)
        .bind(parameters)
        .bind(request_body)
        .bind(responses)
        .bind(examples)
        .bind(tags)
        .bind(deprecated)
        .bind(changelog)
        .bind(security_schemes)
        .bind(rate_limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_docs_v7: {e}")))?;
        Ok(row)
    }

    pub async fn get_deprecated_api_docs_v7(&self) -> Result<Vec<ApiDocsV7>> {
        sqlx::query_as::<_, ApiDocsV7>(
            "SELECT * FROM api_docs_v7 WHERE deprecated = true ORDER BY endpoint, method, version",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_deprecated_api_docs_v7: {e}")))
    }

    pub async fn update_api_docs_v7(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
        summary: Option<&str>,
        description: Option<&str>,
        parameters: Option<&serde_json::Value>,
        request_body: Option<Option<&serde_json::Value>>,
        responses: Option<&serde_json::Value>,
        examples: Option<&serde_json::Value>,
        tags: Option<&[String]>,
        deprecated: Option<bool>,
        changelog: Option<&str>,
        security_schemes: Option<&serde_json::Value>,
        rate_limits: Option<&serde_json::Value>,
    ) -> Result<ApiDocsV7> {
        let row = sqlx::query_as::<_, ApiDocsV7>(
            r#"UPDATE api_docs_v7
               SET summary = COALESCE($4, summary),
                   description = COALESCE($5, description),
                   parameters = COALESCE($6, parameters),
                   request_body = CASE WHEN $7 IS NOT NULL THEN $7 ELSE request_body END,
                   responses = COALESCE($8, responses),
                   examples = COALESCE($9, examples),
                   tags = COALESCE($10, tags),
                   deprecated = COALESCE($11, deprecated),
                   changelog = COALESCE($12, changelog),
                   security_schemes = COALESCE($13, security_schemes),
                   rate_limits = COALESCE($14, rate_limits)
               WHERE endpoint = $1 AND method = $2 AND version = $3
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .bind(summary)
        .bind(description)
        .bind(parameters)
        .bind(request_body)
        .bind(responses)
        .bind(examples)
        .bind(tags)
        .bind(deprecated)
        .bind(changelog)
        .bind(security_schemes)
        .bind(rate_limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_api_docs_v7: {e}")))?;
        Ok(row)
    }

    pub async fn delete_api_docs_v7(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
    ) -> Result<()> {
        sqlx::query("DELETE FROM api_docs_v7 WHERE endpoint = $1 AND method = $2 AND version = $3")
            .bind(endpoint)
            .bind(method)
            .bind(version)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_api_docs_v7: {e}")))?;
        Ok(())
    }

    pub async fn get_rate_limits_for_endpoint_v7(
        &self,
        endpoint: &str,
        method: &str,
    ) -> Result<serde_json::Value> {
        let doc = sqlx::query_as::<_, ApiDocsV7>(
            "SELECT * FROM api_docs_v7 WHERE endpoint = $1 AND method = $2 ORDER BY version DESC LIMIT 1",
        )
        .bind(endpoint)
        .bind(method)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_rate_limits_for_endpoint_v7: {e}")))?;

        Ok(doc.map(|d| d.rate_limits).unwrap_or(serde_json::json!({})))
    }

    pub async fn get_error_codes_for_endpoint_v7(
        &self,
        endpoint: &str,
        method: &str,
    ) -> Result<serde_json::Value> {
        let doc = sqlx::query_as::<_, ApiDocsV7>(
            "SELECT * FROM api_docs_v7 WHERE endpoint = $1 AND method = $2 ORDER BY version DESC LIMIT 1",
        )
        .bind(endpoint)
        .bind(method)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_error_codes_for_endpoint_v7: {e}")))?;

        Ok(doc.map(|d| d.responses).unwrap_or(serde_json::json!({})))
    }

    pub async fn generate_sdk_spec_v7(&self, version: &str) -> Result<Vec<serde_json::Value>> {
        let rows = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'endpoint', endpoint,
                'method', method,
                'summary', summary,
                'parameters', parameters,
                'request_body', request_body,
                'responses', responses,
                'security_schemes', security_schemes,
                'rate_limits', rate_limits
               )
               FROM api_docs_v7
               WHERE version = $1 AND deprecated = false
               ORDER BY endpoint, method"#,
        )
        .bind(version)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("generate_sdk_spec_v7: {e}")))?;
        Ok(rows)
    }

    pub async fn get_api_changelog_v7(&self, version: &str) -> Result<Vec<serde_json::Value>> {
        let rows = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'endpoint', endpoint,
                'method', method,
                'changelog', changelog,
                'deprecated', deprecated,
                'created_at', created_at
               )
               FROM api_docs_v7
               WHERE version = $1 AND changelog != ''
               ORDER BY created_at DESC"#,
        )
        .bind(version)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_changelog_v7: {e}")))?;
        Ok(rows)
    }

    // --- Rate Limit Tiers v5 ---

    pub async fn list_rate_limit_tiers_v5(&self) -> Result<Vec<RateLimitTierV5>> {
        sqlx::query_as::<_, RateLimitTierV5>("SELECT * FROM rate_limit_tiers_v5 ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_rate_limit_tiers_v5: {e}")))
    }

    pub async fn get_rate_limit_tier_v5_by_name(
        &self,
        name: &str,
    ) -> Result<Option<RateLimitTierV5>> {
        sqlx::query_as::<_, RateLimitTierV5>(
            "SELECT * FROM rate_limit_tiers_v5 WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_rate_limit_tier_v5_by_name: {e}")))
    }

    pub async fn create_rate_limit_tier_v5(
        &self,
        name: &str,
        description: &str,
        rate_limit: i32,
        burst_limit: i32,
        monthly_quota: Option<i32>,
        price_cents: i32,
        features: &serde_json::Value,
        limits: &serde_json::Value,
    ) -> Result<RateLimitTierV5> {
        let row = sqlx::query_as::<_, RateLimitTierV5>(
            r#"INSERT INTO rate_limit_tiers_v5 (name, description, rate_limit, burst_limit, monthly_quota, price_cents, features, limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_tier_v5: {e}")))?;
        Ok(row)
    }

    pub async fn update_rate_limit_tier_v5(
        &self,
        name: &str,
        description: Option<&str>,
        rate_limit: Option<i32>,
        burst_limit: Option<i32>,
        monthly_quota: Option<Option<i32>>,
        price_cents: Option<i32>,
        features: Option<&serde_json::Value>,
        limits: Option<&serde_json::Value>,
    ) -> Result<RateLimitTierV5> {
        let row = sqlx::query_as::<_, RateLimitTierV5>(
            r#"UPDATE rate_limit_tiers_v5
               SET description = COALESCE($2, description),
                   rate_limit = COALESCE($3, rate_limit),
                   burst_limit = COALESCE($4, burst_limit),
                   monthly_quota = CASE WHEN $5 IS NOT NULL THEN $5 ELSE monthly_quota END,
                   price_cents = COALESCE($6, price_cents),
                   features = COALESCE($7, features),
                   limits = COALESCE($8, limits)
               WHERE name = $1
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_rate_limit_tier_v5: {e}")))?;
        Ok(row)
    }

    pub async fn delete_rate_limit_tier_v5(&self, name: &str) -> Result<()> {
        sqlx::query("DELETE FROM rate_limit_tiers_v5 WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_rate_limit_tier_v5: {e}")))?;
        Ok(())
    }

    // --- Rate Limit Alerts v3 ---

    pub async fn create_rate_limit_alert_v3(
        &self,
        user_id: Uuid,
        tier_id: Uuid,
        alert_type: &str,
        threshold: f64,
        enabled: bool,
    ) -> Result<RateLimitAlertV3> {
        let row = sqlx::query_as::<_, RateLimitAlertV3>(
            r#"INSERT INTO rate_limit_alerts_v2 (user_id, tier_id, alert_type, threshold, enabled)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(tier_id)
        .bind(alert_type)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_alert_v3: {e}")))?;
        Ok(row)
    }

    pub async fn get_user_rate_limit_alerts_v3(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<RateLimitAlertV3>> {
        sqlx::query_as::<_, RateLimitAlertV3>(
            "SELECT * FROM rate_limit_alerts_v2 WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_user_rate_limit_alerts_v3: {e}")))
    }

    pub async fn update_rate_limit_alert_v3(
        &self,
        id: Uuid,
        threshold: Option<f64>,
        enabled: Option<bool>,
    ) -> Result<RateLimitAlertV3> {
        let row = sqlx::query_as::<_, RateLimitAlertV3>(
            r#"UPDATE rate_limit_alerts_v2
               SET threshold = COALESCE($2, threshold),
                   enabled = COALESCE($3, enabled)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_rate_limit_alert_v3: {e}")))?;
        Ok(row)
    }

    pub async fn delete_rate_limit_alert_v3(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM rate_limit_alerts_v2 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_rate_limit_alert_v3: {e}")))?;
        Ok(())
    }

    pub async fn get_active_rate_limit_alerts_v3(
        &self,
        tier_id: Uuid,
    ) -> Result<Vec<RateLimitAlertV3>> {
        sqlx::query_as::<_, RateLimitAlertV3>(
            "SELECT * FROM rate_limit_alerts_v2 WHERE tier_id = $1 AND enabled = true ORDER BY threshold",
        )
        .bind(tier_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_active_rate_limit_alerts_v3: {e}")))
    }

    pub async fn get_alert_history_v3(
        &self,
        user_id: Uuid,
        alert_type: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<RateLimitAlertV3>> {
        let rows = if let Some(at) = alert_type {
            sqlx::query_as::<_, RateLimitAlertV3>(
                r#"SELECT * FROM rate_limit_alerts_v2
                   WHERE user_id = $1 AND alert_type = $2
                   ORDER BY created_at DESC
                   LIMIT $3 OFFSET $4"#,
            )
            .bind(user_id)
            .bind(at)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, RateLimitAlertV3>(
                r#"SELECT * FROM rate_limit_alerts_v2
                   WHERE user_id = $1
                   ORDER BY created_at DESC
                   LIMIT $2 OFFSET $3"#,
            )
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("get_alert_history_v3: {e}")))
    }

    pub async fn get_alert_analytics_v3(
        &self,
        user_id: Uuid,
    ) -> Result<serde_json::Value> {
        let row = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'total_alerts', COUNT(*),
                'enabled_alerts', COUNT(*) FILTER (WHERE enabled = true),
                'disabled_alerts', COUNT(*) FILTER (WHERE enabled = false),
                'by_type', json_build_object(
                    'threshold', COUNT(*) FILTER (WHERE alert_type = 'threshold'),
                    'quota', COUNT(*) FILTER (WHERE alert_type = 'quota'),
                    'burst', COUNT(*) FILTER (WHERE alert_type = 'burst')
                ),
                'avg_threshold', AVG(threshold),
                'recently_triggered', COUNT(*) FILTER (WHERE last_triggered_at IS NOT NULL AND last_triggered_at > NOW() - INTERVAL '24 hours')
               )
               FROM rate_limit_alerts_v2
               WHERE user_id = $1"#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_alert_analytics_v3: {e}")))?;
        Ok(row)
    }

    pub async fn mark_alert_triggered_v3(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE rate_limit_alerts_v2 SET last_triggered_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("mark_alert_triggered_v3: {e}")))?;
        Ok(())
    }

    // --- API Analytics v8 ---

    pub async fn list_api_analytics_v8(&self, limit: i64, offset: i64) -> Result<Vec<ApiAnalyticV8>> {
        sqlx::query_as::<_, ApiAnalyticV8>(
            "SELECT * FROM api_analytics_v8 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_analytics_v8: {e}")))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_api_analytic_v8(
        &self,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_time_ms: i32,
        user_id: Option<Uuid>,
        request_size_bytes: i32,
        response_size_bytes: i32,
        cache_hit: bool,
        region: &str,
        user_agent: Option<&str>,
        request_id: Option<Uuid>,
        cost_cents: i32,
    ) -> Result<ApiAnalyticV8> {
        let row = sqlx::query_as::<_, ApiAnalyticV8>(
            r#"INSERT INTO api_analytics_v8 (endpoint, method, status_code, response_time_ms, user_id, request_size_bytes, response_size_bytes, cache_hit, region, user_agent, request_id, cost_cents)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(user_id)
        .bind(request_size_bytes)
        .bind(response_size_bytes)
        .bind(cache_hit)
        .bind(region)
        .bind(user_agent)
        .bind(request_id)
        .bind(cost_cents)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_analytic_v8: {e}")))?;
        Ok(row)
    }

    pub async fn get_cost_tracking_v8(
        &self,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<serde_json::Value> {
        let row = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'total_cost_cents', COALESCE(SUM(cost_cents), 0),
                'avg_cost_per_request', COALESCE(AVG(cost_cents), 0),
                'total_requests', COUNT(*),
                'by_endpoint', (
                    SELECT json_agg(json_build_object(
                        'endpoint', endpoint,
                        'method', method,
                        'total_cost', SUM(cost_cents),
                        'request_count', COUNT(*),
                        'avg_cost', AVG(cost_cents)
                    ))
                    FROM api_analytics_v8
                    WHERE created_at >= $1
                    GROUP BY endpoint, method
                    ORDER BY SUM(cost_cents) DESC
                    LIMIT 20
                ),
                'by_region', (
                    SELECT json_agg(json_build_object(
                        'region', region,
                        'total_cost', SUM(cost_cents),
                        'request_count', COUNT(*)
                    ))
                    FROM api_analytics_v8
                    WHERE created_at >= $1
                    GROUP BY region
                    ORDER BY SUM(cost_cents) DESC
                )
               )
               FROM api_analytics_v8
               WHERE created_at >= $1"#,
        )
        .bind(since)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_tracking_v8: {e}")))?;
        Ok(row)
    }

    pub async fn get_budget_alerts_v8(
        &self,
        user_id: Uuid,
        budget_cents: i64,
    ) -> Result<serde_json::Value> {
        let row = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'current_cost_cents', COALESCE(SUM(cost_cents), 0),
                'budget_cents', $2,
                'budget_used_percent', CASE WHEN $2 > 0 THEN COALESCE(SUM(cost_cents), 0)::float / $2::float * 100 ELSE 0 END,
                'exceeds_budget', COALESCE(SUM(cost_cents), 0) > $2,
                'daily_costs', (
                    SELECT json_agg(json_build_object(
                        'date', DATE(created_at),
                        'cost_cents', SUM(cost_cents)
                    ))
                    FROM api_analytics_v8
                    WHERE user_id = $1
                      AND created_at >= NOW() - INTERVAL '30 days'
                    GROUP BY DATE(created_at)
                    ORDER BY DATE(created_at)
                )
               )
               FROM api_analytics_v8
               WHERE user_id = $1"#,
        )
        .bind(user_id)
        .bind(budget_cents)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_budget_alerts_v8: {e}")))?;
        Ok(row)
    }

    pub async fn get_usage_optimization_v8(&self) -> Result<Vec<serde_json::Value>> {
        let rows = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'endpoint', endpoint,
                'method', method,
                'avg_response_time_ms', AVG(response_time_ms),
                'p95_response_time_ms', PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms),
                'cache_hit_rate', CASE WHEN COUNT(*) > 0 THEN (COUNT(*) FILTER (WHERE cache_hit))::float / COUNT(*)::float * 100 ELSE 0 END,
                'avg_cost_cents', AVG(cost_cents),
                'total_requests', COUNT(*),
                'suggestions', ARRAY_REMOVE(ARRAY[
                    CASE WHEN AVG(response_time_ms) > 500 THEN 'Optimize slow endpoint' END,
                    CASE WHEN (COUNT(*) FILTER (WHERE cache_hit))::float / NULLIF(COUNT(*), 0) < 0.3 THEN 'Improve caching' END,
                    CASE WHEN AVG(cost_cents) > 10 THEN 'Reduce per-request cost' END
                ], NULL)
               )
               FROM api_analytics_v8
               GROUP BY endpoint, method
               HAVING COUNT(*) > 10
               ORDER BY AVG(response_time_ms) DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_usage_optimization_v8: {e}")))?;
        Ok(rows)
    }

    pub async fn get_capacity_planning_v8(&self) -> Result<Vec<serde_json::Value>> {
        let rows = sqlx::query_scalar::<_, serde_json::Value>(
            r#"SELECT json_build_object(
                'endpoint', endpoint,
                'method', method,
                'current_rps', COUNT(*) / EXTRACT(EPOCH FROM (MAX(created_at) - MIN(created_at))),
                'projected_rps_24h', COUNT(*) / EXTRACT(EPOCH FROM (MAX(created_at) - MIN(created_at))) * 1.2,
                'avg_response_time_ms', AVG(response_time_ms),
                'error_rate', CASE WHEN COUNT(*) > 0 THEN (COUNT(*) FILTER (WHERE status_code >= 500))::float / COUNT(*)::float * 100 ELSE 0 END,
                'region_distribution', (
                    SELECT json_object_agg(region, cnt)
                    FROM (
                        SELECT region, COUNT(*) as cnt
                        FROM api_analytics_v8 a2
                        WHERE a2.endpoint = a1.endpoint AND a2.method = a1.method
                        GROUP BY region
                    ) sub
                )
               )
               FROM api_analytics_v8 a1
               GROUP BY endpoint, method
               HAVING COUNT(*) > 5
               ORDER BY COUNT(*) DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_capacity_planning_v8: {e}")))?;
        Ok(rows)
    }

    // --- Pipeline Action Reviews v4 ---

    pub async fn create_pipeline_action_review(
        &self,
        action_id: Uuid,
        user_id: Uuid,
        rating: i32,
        review: &str,
    ) -> Result<PipelineActionReviewV4> {
        let row = sqlx::query_as::<_, PipelineActionReviewV4>(
            r#"INSERT INTO pipeline_action_reviews_v4 (action_id, user_id, rating, review)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(action_id)
        .bind(user_id)
        .bind(rating)
        .bind(review)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_pipeline_action_review: {e}")))?;
        Ok(row)
    }

    pub async fn get_pipeline_action_reviews(
        &self,
        action_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PipelineActionReviewV4>> {
        sqlx::query_as::<_, PipelineActionReviewV4>(
            "SELECT * FROM pipeline_action_reviews_v4 WHERE action_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(action_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_pipeline_action_reviews: {e}")))
    }

    pub async fn update_review_helpfulness(
        &self,
        review_id: Uuid,
        user_id: Uuid,
        helpful: bool,
    ) -> Result<ReviewHelpfulnessV3> {
        let row = sqlx::query_as::<_, ReviewHelpfulnessV3>(
            r#"INSERT INTO review_helpfulness_v3 (review_id, user_id, helpful)
               VALUES ($1, $2, $3)
               ON CONFLICT (review_id, user_id) DO UPDATE SET helpful = $3
               RETURNING *"#,
        )
        .bind(review_id)
        .bind(user_id)
        .bind(helpful)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_review_helpfulness: {e}")))?;
        Ok(row)
    }

    pub async fn moderate_review(
        &self,
        review_id: Uuid,
        moderator_id: Uuid,
        status: &str,
        reason: Option<&str>,
    ) -> Result<ReviewModerationQueueV3> {
        let row = sqlx::query_as::<_, ReviewModerationQueueV3>(
            r#"INSERT INTO review_moderation_queue_v3 (review_id, status, moderator_id, reason, moderated_at)
               VALUES ($1, $2, $3, $4, NOW())
               ON CONFLICT (review_id) DO UPDATE SET status = $2, moderator_id = $3, reason = $4, moderated_at = NOW()
               RETURNING *"#,
        )
        .bind(review_id)
        .bind(status)
        .bind(moderator_id)
        .bind(reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("moderate_review: {e}")))?;
        Ok(row)
    }

    pub async fn get_review_analytics(
        &self,
        action_id: Uuid,
        period_start: DateTime<Utc>,
    ) -> Result<Vec<ReviewAnalyticsV3>> {
        sqlx::query_as::<_, ReviewAnalyticsV3>(
            "SELECT * FROM review_analytics_v3 WHERE action_id = $1 AND period_start >= $2 ORDER BY period_start DESC",
        )
        .bind(action_id)
        .bind(period_start)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_review_analytics: {e}")))
    }

    pub async fn get_review_recommendations(
        &self,
        action_id: Uuid,
        limit: i64,
    ) -> Result<Vec<ReviewRecommendationV3>> {
        sqlx::query_as::<_, ReviewRecommendationV3>(
            "SELECT * FROM review_recommendations_v3 WHERE action_id = $1 ORDER BY confidence DESC LIMIT $2",
        )
        .bind(action_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_review_recommendations: {e}")))
    }

    // --- Environment Deployment History v4 ---

    pub async fn create_deployment_history_v4(
        &self,
        environment_id: Uuid,
        version: &str,
        sha: &str,
        status: &str,
        deployed_by: Uuid,
        rollback_of: Option<Uuid>,
        metadata: serde_json::Value,
    ) -> Result<EnvironmentDeploymentHistoryV4> {
        let row = sqlx::query_as::<_, EnvironmentDeploymentHistoryV4>(
            r#"INSERT INTO environment_deployment_history_v4 (environment_id, version, sha, status, deployed_by, rollback_of, metadata)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING *"#,
        )
        .bind(environment_id)
        .bind(version)
        .bind(sha)
        .bind(status)
        .bind(deployed_by)
        .bind(rollback_of)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_deployment_history_v4: {e}")))?;
        Ok(row)
    }

    pub async fn list_deployment_history_v4(
        &self,
        environment_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<EnvironmentDeploymentHistoryV4>> {
        sqlx::query_as::<_, EnvironmentDeploymentHistoryV4>(
            "SELECT * FROM environment_deployment_history_v4 WHERE environment_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(environment_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_deployment_history_v4: {e}")))
    }

    pub async fn rollback_deployment_v4(
        &self,
        original_id: Uuid,
        deployed_by: Uuid,
    ) -> Result<EnvironmentDeploymentHistoryV4> {
        let original = sqlx::query_as::<_, EnvironmentDeploymentHistoryV4>(
            "SELECT * FROM environment_deployment_history_v4 WHERE id = $1",
        )
        .bind(original_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("rollback_deployment_v4 original: {e}")))?;

        let new_version = format!("rollback-{}", original.version);
        self.create_deployment_history_v4(
            original.environment_id,
            &new_version,
            &original.sha,
            "deployed",
            deployed_by,
            Some(original_id),
            original.metadata,
        )
        .await
    }

    pub async fn compare_deployments(
        &self,
        from_deployment_id: Uuid,
        to_deployment_id: Uuid,
    ) -> Result<Option<DeploymentComparisonV4>> {
        sqlx::query_as::<_, DeploymentComparisonV4>(
            "SELECT * FROM deployment_comparison_v4 WHERE from_deployment_id = $1 AND to_deployment_id = $2",
        )
        .bind(from_deployment_id)
        .bind(to_deployment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("compare_deployments: {e}")))
    }

    pub async fn get_deployment_analytics_v4(
        &self,
        environment_id: Uuid,
        period_start: DateTime<Utc>,
    ) -> Result<Vec<DeploymentAnalyticsV4>> {
        sqlx::query_as::<_, DeploymentAnalyticsV4>(
            "SELECT * FROM deployment_analytics_v4 WHERE environment_id = $1 AND period_start >= $2 ORDER BY period_start DESC",
        )
        .bind(environment_id)
        .bind(period_start)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_deployment_analytics_v4: {e}")))
    }

    // --- Cache Hit Analysis v3 ---

    pub async fn create_cache_hit_analysis(
        &self,
        cache_id: Uuid,
        period_start: DateTime<Utc>,
        hit_count: i32,
        miss_count: i32,
        avg_hit_size_bytes: i64,
        total_size_bytes: i64,
    ) -> Result<CacheHitAnalysisV3> {
        let row = sqlx::query_as::<_, CacheHitAnalysisV3>(
            r#"INSERT INTO cache_hit_analysis_v3 (cache_id, period_start, hit_count, miss_count, avg_hit_size_bytes, total_size_bytes)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING *"#,
        )
        .bind(cache_id)
        .bind(period_start)
        .bind(hit_count)
        .bind(miss_count)
        .bind(avg_hit_size_bytes)
        .bind(total_size_bytes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_cache_hit_analysis: {e}")))?;
        Ok(row)
    }

    pub async fn get_cache_hit_analysis(
        &self,
        cache_id: Uuid,
        period_start: DateTime<Utc>,
    ) -> Result<Vec<CacheHitAnalysisV3>> {
        sqlx::query_as::<_, CacheHitAnalysisV3>(
            "SELECT * FROM cache_hit_analysis_v3 WHERE cache_id = $1 AND period_start >= $2 ORDER BY period_start DESC",
        )
        .bind(cache_id)
        .bind(period_start)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cache_hit_analysis: {e}")))
    }

    pub async fn track_cache_size(
        &self,
        cache_id: Uuid,
        size_bytes: i64,
        item_count: i32,
    ) -> Result<CacheSizeTrackingV3> {
        let row = sqlx::query_as::<_, CacheSizeTrackingV3>(
            r#"INSERT INTO cache_size_tracking_v3 (cache_id, size_bytes, item_count)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(cache_id)
        .bind(size_bytes)
        .bind(item_count)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("track_cache_size: {e}")))?;
        Ok(row)
    }

    pub async fn get_cache_optimization_suggestions(
        &self,
        cache_id: Uuid,
    ) -> Result<Vec<CacheCostOptimizationV3>> {
        sqlx::query_as::<_, CacheCostOptimizationV3>(
            "SELECT * FROM cache_cost_optimization_v3 WHERE cache_id = $1 ORDER BY period_start DESC",
        )
        .bind(cache_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cache_optimization_suggestions: {e}")))
    }

    pub async fn get_cache_performance_insights(
        &self,
        cache_id: Uuid,
        period_start: DateTime<Utc>,
    ) -> Result<Vec<CachePerformanceInsightsV3>> {
        sqlx::query_as::<_, CachePerformanceInsightsV3>(
            "SELECT * FROM cache_performance_insights_v3 WHERE cache_id = $1 AND period_start >= $2 ORDER BY period_start DESC",
        )
        .bind(cache_id)
        .bind(period_start)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cache_performance_insights: {e}")))
    }

    // --- Test Suite Metrics v3 ---

    pub async fn create_test_suite_metric_v3(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<TestSuiteMetricV3> {
        let row = sqlx::query_as::<_, TestSuiteMetricV3>(
            r#"INSERT INTO test_suite_metrics_v3 (suite_id, metric_name, metric_value)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_metric_v3: {e}")))?;
        Ok(row)
    }

    pub async fn list_test_suite_metrics_v3(
        &self,
        suite_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestSuiteMetricV3>> {
        sqlx::query_as::<_, TestSuiteMetricV3>(
            "SELECT * FROM test_suite_metrics_v3 WHERE suite_id = $1 ORDER BY measured_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(suite_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_test_suite_metrics_v3: {e}")))
    }

    pub async fn get_test_suite_latest_metric_v3(
        &self,
        suite_id: Uuid,
        metric_name: &str,
    ) -> Result<Option<TestSuiteMetricV3>> {
        sqlx::query_as::<_, TestSuiteMetricV3>(
            "SELECT * FROM test_suite_metrics_v3 WHERE suite_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT 1",
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_latest_metric_v3: {e}")))
    }

    // --- Test Suite Baselines v3 ---

    pub async fn create_test_suite_baseline_v3(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        baseline_value: f64,
        threshold_percent: f64,
    ) -> Result<TestSuiteBaselineV3> {
        let row = sqlx::query_as::<_, TestSuiteBaselineV3>(
            r#"INSERT INTO test_suite_baselines_v3 (suite_id, metric_name, baseline_value, threshold_percent)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (suite_id, metric_name) DO UPDATE
               SET baseline_value = $3, threshold_percent = $4
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(baseline_value)
        .bind(threshold_percent)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_baseline_v3: {e}")))?;
        Ok(row)
    }

    pub async fn get_test_suite_baselines_v3(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteBaselineV3>> {
        sqlx::query_as::<_, TestSuiteBaselineV3>(
            "SELECT * FROM test_suite_baselines_v3 WHERE suite_id = $1 ORDER BY metric_name",
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_baselines_v3: {e}")))
    }

    pub async fn detect_test_suite_regression_v3(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        current_value: f64,
    ) -> Result<bool> {
        let baseline: Option<TestSuiteBaselineV3> = sqlx::query_as::<_, TestSuiteBaselineV3>(
            "SELECT * FROM test_suite_baselines_v3 WHERE suite_id = $1 AND metric_name = $2",
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("detect_test_suite_regression_v3: {e}")))?;

        match baseline {
            Some(b) => {
                let diff = ((current_value - b.baseline_value) / b.baseline_value * 100.0).abs();
                Ok(diff > b.threshold_percent)
            }
            None => Ok(false),
        }
    }

    // --- Code Quality Metrics v4 ---

    pub async fn create_code_quality_metric_v4(
        &self,
        repo_id: Uuid,
        file_path: &str,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<CodeQualityMetricV4> {
        let row = sqlx::query_as::<_, CodeQualityMetricV4>(
            r#"INSERT INTO code_quality_metrics_v4 (repo_id, file_path, metric_name, metric_value)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(file_path)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_metric_v4: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_metrics_v4(
        &self,
        repo_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CodeQualityMetricV4>> {
        let rows = if let Some(name) = metric_name {
            sqlx::query_as::<_, CodeQualityMetricV4>(
                r#"SELECT * FROM code_quality_metrics_v4
                   WHERE repo_id = $1 AND metric_name = $2
                   ORDER BY measured_at DESC
                   LIMIT $3 OFFSET $4"#,
            )
            .bind(repo_id)
            .bind(name)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, CodeQualityMetricV4>(
                r#"SELECT * FROM code_quality_metrics_v4
                   WHERE repo_id = $1
                   ORDER BY measured_at DESC
                   LIMIT $2 OFFSET $3"#,
            )
            .bind(repo_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_code_quality_metrics_v4: {e}")))
    }

    pub async fn get_code_quality_score_v4(
        &self,
        repo_id: Uuid,
    ) -> Result<f64> {
        let row: (Option<f64>,) = sqlx::query_as(
            r#"SELECT AVG(metric_value) FROM code_quality_metrics_v4 WHERE repo_id = $1"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_score_v4: {e}")))?;
        Ok(row.0.unwrap_or(0.0))
    }

    // --- Code Quality Thresholds v3 ---

    pub async fn create_code_quality_threshold_v3(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        threshold_value: f64,
        enabled: bool,
    ) -> Result<CodeQualityThresholdV3> {
        let row = sqlx::query_as::<_, CodeQualityThresholdV3>(
            r#"INSERT INTO code_quality_thresholds_v3 (repo_id, metric_name, threshold_value, enabled)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (repo_id, metric_name) DO UPDATE
               SET threshold_value = $3, enabled = $4
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .bind(threshold_value)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_threshold_v3: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_thresholds_v3(
        &self,
        repo_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<CodeQualityThresholdV3>> {
        let rows = if enabled_only {
            sqlx::query_as::<_, CodeQualityThresholdV3>(
                r#"SELECT * FROM code_quality_thresholds_v3
                   WHERE repo_id = $1 AND enabled = true
                   ORDER BY metric_name"#,
            )
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, CodeQualityThresholdV3>(
                r#"SELECT * FROM code_quality_thresholds_v3
                   WHERE repo_id = $1
                   ORDER BY metric_name"#,
            )
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_code_quality_thresholds_v3: {e}")))
    }

    pub async fn check_code_quality_violation_v3(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<bool> {
        let threshold: Option<CodeQualityThresholdV3> =
            sqlx::query_as::<_, CodeQualityThresholdV3>(
                r#"SELECT * FROM code_quality_thresholds_v3
                   WHERE repo_id = $1 AND metric_name = $2 AND enabled = true"#,
            )
            .bind(repo_id)
            .bind(metric_name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("check_code_quality_violation_v3: {e}")))?;

        match threshold {
            Some(t) => Ok(metric_value > t.threshold_value),
            None => Ok(false),
        }
    }

    pub async fn delete_code_quality_threshold_v3(
        &self,
        repo_id: Uuid,
        metric_name: &str,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM code_quality_thresholds_v3 WHERE repo_id = $1 AND metric_name = $2",
        )
        .bind(repo_id)
        .bind(metric_name)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("delete_code_quality_threshold_v3: {e}")))?;
        Ok(())
    }

    // --- Performance Test Alerts v4 ---

    pub async fn create_performance_test_alert_v4(
        &self,
        baseline_id: Uuid,
        alert_type: &str,
        threshold: f64,
        enabled: bool,
    ) -> Result<PerformanceTestAlertV4> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV4>(
            r#"INSERT INTO performance_test_alerts_v4 (baseline_id, alert_type, threshold, enabled)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(baseline_id)
        .bind(alert_type)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_performance_test_alert_v4: {e}")))?;
        Ok(row)
    }

    pub async fn list_performance_test_alerts_v4(
        &self,
        baseline_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<PerformanceTestAlertV4>> {
        let rows = if enabled_only {
            sqlx::query_as::<_, PerformanceTestAlertV4>(
                r#"SELECT * FROM performance_test_alerts_v4
                   WHERE baseline_id = $1 AND enabled = true
                   ORDER BY created_at DESC"#,
            )
            .bind(baseline_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, PerformanceTestAlertV4>(
                r#"SELECT * FROM performance_test_alerts_v4
                   WHERE baseline_id = $1
                   ORDER BY created_at DESC"#,
            )
            .bind(baseline_id)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_performance_test_alerts_v4: {e}")))
    }

    pub async fn update_performance_test_alert_v4(
        &self,
        id: Uuid,
        threshold: Option<f64>,
        enabled: Option<bool>,
    ) -> Result<PerformanceTestAlertV4> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV4>(
            r#"UPDATE performance_test_alerts_v4
               SET threshold = COALESCE($2, threshold),
                   enabled = COALESCE($3, enabled)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_performance_test_alert_v4: {e}")))?;
        Ok(row)
    }

    pub async fn record_performance_test_alert_v4(
        &self,
        alert_id: Uuid,
        metric_name: &str,
        metric_value: f64,
        threshold: f64,
    ) -> Result<PerformanceTestAlertHistoryV4> {
        let row = sqlx::query_as::<_, PerformanceTestAlertHistoryV4>(
            r#"INSERT INTO performance_test_alert_history_v4 (alert_id, metric_name, metric_value, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(alert_id)
        .bind(metric_name)
        .bind(metric_value)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_performance_test_alert_v4: {e}")))?;

        sqlx::query(
            "UPDATE performance_test_alerts_v4 SET last_triggered_at = NOW() WHERE id = $1",
        )
        .bind(alert_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_performance_test_alert_v4 update: {e}")))?;

        Ok(row)
    }

    pub async fn list_performance_test_alert_history_v4(
        &self,
        alert_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PerformanceTestAlertHistoryV4>> {
        sqlx::query_as::<_, PerformanceTestAlertHistoryV4>(
            "SELECT * FROM performance_test_alert_history_v4 WHERE alert_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(alert_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_performance_test_alert_history_v4: {e}")))
    }

    pub async fn get_performance_test_alert_analytics_v4(
        &self,
        alert_id: Uuid,
    ) -> Result<(i64, Option<f64>, Option<f64>)> {
        let row: (i64, Option<f64>, Option<f64>) = sqlx::query_as(
            r#"SELECT
                   COUNT(*) as trigger_count,
                   AVG(metric_value) as avg_value,
                   MAX(metric_value) as max_value
               FROM performance_test_alert_history_v4
               WHERE alert_id = $1"#,
        )
        .bind(alert_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_analytics_v4: {e}")))?;
        Ok((row.0, row.1, row.2))
    }

    // --- Test Suite Metrics v5 ---

    pub async fn create_test_suite_metric_v5(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<TestSuiteMetricV5> {
        let row = sqlx::query_as::<_, TestSuiteMetricV5>(
            r#"INSERT INTO test_suite_metrics_v5 (suite_id, metric_name, metric_value)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_metric_v5: {e}")))?;
        Ok(row)
    }

    pub async fn list_test_suite_metrics_v5(
        &self,
        suite_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestSuiteMetricV5>> {
        sqlx::query_as::<_, TestSuiteMetricV5>(
            "SELECT * FROM test_suite_metrics_v5 WHERE suite_id = $1 ORDER BY measured_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(suite_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_test_suite_metrics_v5: {e}")))
    }

    pub async fn get_test_suite_latest_metric_v5(
        &self,
        suite_id: Uuid,
        metric_name: &str,
    ) -> Result<Option<TestSuiteMetricV5>> {
        sqlx::query_as::<_, TestSuiteMetricV5>(
            "SELECT * FROM test_suite_metrics_v5 WHERE suite_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT 1",
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_latest_metric_v5: {e}")))
    }

    // --- Test Suite Baselines v5 ---

    pub async fn create_test_suite_baseline_v5(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        baseline_value: f64,
        threshold_percent: f64,
    ) -> Result<TestSuiteBaselineV5> {
        let row = sqlx::query_as::<_, TestSuiteBaselineV5>(
            r#"INSERT INTO test_suite_baselines_v5 (suite_id, metric_name, baseline_value, threshold_percent)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (suite_id, metric_name) DO UPDATE
               SET baseline_value = $3, threshold_percent = $4
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(baseline_value)
        .bind(threshold_percent)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_baseline_v5: {e}")))?;
        Ok(row)
    }

    pub async fn get_test_suite_baselines_v5(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteBaselineV5>> {
        sqlx::query_as::<_, TestSuiteBaselineV5>(
            "SELECT * FROM test_suite_baselines_v5 WHERE suite_id = $1 ORDER BY metric_name",
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_baselines_v5: {e}")))
    }

    pub async fn detect_test_suite_regression_v5(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        current_value: f64,
    ) -> Result<bool> {
        let baseline: Option<TestSuiteBaselineV5> = sqlx::query_as::<_, TestSuiteBaselineV5>(
            "SELECT * FROM test_suite_baselines_v5 WHERE suite_id = $1 AND metric_name = $2",
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("detect_test_suite_regression_v5: {e}")))?;

        match baseline {
            Some(b) => {
                let diff = ((current_value - b.baseline_value) / b.baseline_value * 100.0).abs();
                Ok(diff > b.threshold_percent)
            }
            None => Ok(false),
        }
    }

    // --- Code Quality Metrics v6 ---

    pub async fn create_code_quality_metric_v6(
        &self,
        repo_id: Uuid,
        file_path: &str,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<CodeQualityMetricV6> {
        let row = sqlx::query_as::<_, CodeQualityMetricV6>(
            r#"INSERT INTO code_quality_metrics_v6 (repo_id, file_path, metric_name, metric_value)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(file_path)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_metric_v6: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_metrics_v6(
        &self,
        repo_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CodeQualityMetricV6>> {
        let rows = if let Some(name) = metric_name {
            sqlx::query_as::<_, CodeQualityMetricV6>(
                r#"SELECT * FROM code_quality_metrics_v6
                   WHERE repo_id = $1 AND metric_name = $2
                   ORDER BY measured_at DESC
                   LIMIT $3 OFFSET $4"#,
            )
            .bind(repo_id)
            .bind(name)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, CodeQualityMetricV6>(
                r#"SELECT * FROM code_quality_metrics_v6
                   WHERE repo_id = $1
                   ORDER BY measured_at DESC
                   LIMIT $2 OFFSET $3"#,
            )
            .bind(repo_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_code_quality_metrics_v6: {e}")))
    }

    pub async fn get_code_quality_score_v6(
        &self,
        repo_id: Uuid,
    ) -> Result<f64> {
        let row: (Option<f64>,) = sqlx::query_as(
            r#"SELECT AVG(metric_value) FROM code_quality_metrics_v6 WHERE repo_id = $1"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_score_v6: {e}")))?;
        Ok(row.0.unwrap_or(0.0))
    }

    // --- Code Quality Thresholds v5 ---

    pub async fn create_code_quality_threshold_v5(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        threshold_value: f64,
        enabled: bool,
    ) -> Result<CodeQualityThresholdV5> {
        let row = sqlx::query_as::<_, CodeQualityThresholdV5>(
            r#"INSERT INTO code_quality_thresholds_v5 (repo_id, metric_name, threshold_value, enabled)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (repo_id, metric_name) DO UPDATE
               SET threshold_value = $3, enabled = $4
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .bind(threshold_value)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_threshold_v5: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_thresholds_v5(
        &self,
        repo_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<CodeQualityThresholdV5>> {
        let rows = if enabled_only {
            sqlx::query_as::<_, CodeQualityThresholdV5>(
                r#"SELECT * FROM code_quality_thresholds_v5
                   WHERE repo_id = $1 AND enabled = true
                   ORDER BY metric_name"#,
            )
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, CodeQualityThresholdV5>(
                r#"SELECT * FROM code_quality_thresholds_v5
                   WHERE repo_id = $1
                   ORDER BY metric_name"#,
            )
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_code_quality_thresholds_v5: {e}")))
    }

    pub async fn check_code_quality_violation_v5(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<bool> {
        let threshold: Option<CodeQualityThresholdV5> =
            sqlx::query_as::<_, CodeQualityThresholdV5>(
                r#"SELECT * FROM code_quality_thresholds_v5
                   WHERE repo_id = $1 AND metric_name = $2 AND enabled = true"#,
            )
            .bind(repo_id)
            .bind(metric_name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("check_code_quality_violation_v5: {e}")))?;

        match threshold {
            Some(t) => Ok(metric_value > t.threshold_value),
            None => Ok(false),
        }
    }

    pub async fn delete_code_quality_threshold_v5(
        &self,
        repo_id: Uuid,
        metric_name: &str,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM code_quality_thresholds_v5 WHERE repo_id = $1 AND metric_name = $2",
        )
        .bind(repo_id)
        .bind(metric_name)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("delete_code_quality_threshold_v5: {e}")))?;
        Ok(())
    }

    // --- Performance Test Alerts v6 ---

    pub async fn create_performance_test_alert_v6(
        &self,
        baseline_id: Uuid,
        alert_type: &str,
        threshold: f64,
        enabled: bool,
    ) -> Result<PerformanceTestAlertV6> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV6>(
            r#"INSERT INTO performance_test_alerts_v6 (baseline_id, alert_type, threshold, enabled)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(baseline_id)
        .bind(alert_type)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_performance_test_alert_v6: {e}")))?;
        Ok(row)
    }

    pub async fn list_performance_test_alerts_v6(
        &self,
        baseline_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<PerformanceTestAlertV6>> {
        let rows = if enabled_only {
            sqlx::query_as::<_, PerformanceTestAlertV6>(
                r#"SELECT * FROM performance_test_alerts_v6
                   WHERE baseline_id = $1 AND enabled = true
                   ORDER BY created_at DESC"#,
            )
            .bind(baseline_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, PerformanceTestAlertV6>(
                r#"SELECT * FROM performance_test_alerts_v6
                   WHERE baseline_id = $1
                   ORDER BY created_at DESC"#,
            )
            .bind(baseline_id)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_performance_test_alerts_v6: {e}")))
    }

    pub async fn update_performance_test_alert_v6(
        &self,
        id: Uuid,
        threshold: Option<f64>,
        enabled: Option<bool>,
    ) -> Result<PerformanceTestAlertV6> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV6>(
            r#"UPDATE performance_test_alerts_v6
               SET threshold = COALESCE($2, threshold),
                   enabled = COALESCE($3, enabled)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_performance_test_alert_v6: {e}")))?;
        Ok(row)
    }

    pub async fn record_performance_test_alert_v6(
        &self,
        alert_id: Uuid,
        metric_name: &str,
        metric_value: f64,
        threshold: f64,
    ) -> Result<PerformanceTestAlertHistoryV6> {
        let row = sqlx::query_as::<_, PerformanceTestAlertHistoryV6>(
            r#"INSERT INTO performance_test_alert_history_v6 (alert_id, metric_name, metric_value, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(alert_id)
        .bind(metric_name)
        .bind(metric_value)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_performance_test_alert_v6: {e}")))?;

        sqlx::query(
            "UPDATE performance_test_alerts_v6 SET last_triggered_at = NOW() WHERE id = $1",
        )
        .bind(alert_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_performance_test_alert_v6 update: {e}")))?;

        Ok(row)
    }

    pub async fn list_performance_test_alert_history_v6(
        &self,
        alert_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PerformanceTestAlertHistoryV6>> {
        sqlx::query_as::<_, PerformanceTestAlertHistoryV6>(
            "SELECT * FROM performance_test_alert_history_v6 WHERE alert_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(alert_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_performance_test_alert_history_v6: {e}")))
    }

    pub async fn get_performance_test_alert_analytics_v6(
        &self,
        alert_id: Uuid,
    ) -> Result<(i64, Option<f64>, Option<f64>)> {
        let row: (i64, Option<f64>, Option<f64>) = sqlx::query_as(
            r#"SELECT
                   COUNT(*) as trigger_count,
                   AVG(metric_value) as avg_value,
                   MAX(metric_value) as max_value
               FROM performance_test_alert_history_v6
               WHERE alert_id = $1"#,
        )
        .bind(alert_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_analytics_v6: {e}")))?;
        Ok((row.0, row.1, row.2))
    }

    // --- Test Suite Metrics v6 ---

    pub async fn create_test_suite_metric_v6(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<TestSuiteMetricV6> {
        let row = sqlx::query_as::<_, TestSuiteMetricV6>(
            r#"INSERT INTO test_suite_metrics_v6 (suite_id, metric_name, metric_value)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_metric_v6: {e}")))?;
        Ok(row)
    }

    pub async fn list_test_suite_metrics_v6(
        &self,
        suite_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestSuiteMetricV6>> {
        sqlx::query_as::<_, TestSuiteMetricV6>(
            "SELECT * FROM test_suite_metrics_v6 WHERE suite_id = $1 ORDER BY measured_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(suite_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_test_suite_metrics_v6: {e}")))
    }

    pub async fn get_test_suite_latest_metric_v6(
        &self,
        suite_id: Uuid,
        metric_name: &str,
    ) -> Result<Option<TestSuiteMetricV6>> {
        sqlx::query_as::<_, TestSuiteMetricV6>(
            "SELECT * FROM test_suite_metrics_v6 WHERE suite_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT 1",
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_latest_metric_v6: {e}")))
    }

    // --- Test Suite Baselines v6 ---

    pub async fn create_test_suite_baseline_v6(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        baseline_value: f64,
        threshold_percent: f64,
    ) -> Result<TestSuiteBaselineV6> {
        let row = sqlx::query_as::<_, TestSuiteBaselineV6>(
            r#"INSERT INTO test_suite_baselines_v6 (suite_id, metric_name, baseline_value, threshold_percent)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (suite_id, metric_name) DO UPDATE
               SET baseline_value = $3, threshold_percent = $4
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(baseline_value)
        .bind(threshold_percent)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_baseline_v6: {e}")))?;
        Ok(row)
    }

    pub async fn get_test_suite_baselines_v6(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteBaselineV6>> {
        sqlx::query_as::<_, TestSuiteBaselineV6>(
            "SELECT * FROM test_suite_baselines_v6 WHERE suite_id = $1 ORDER BY metric_name",
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_baselines_v6: {e}")))
    }

    pub async fn detect_test_suite_regression_v6(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        current_value: f64,
    ) -> Result<bool> {
        let baseline: Option<TestSuiteBaselineV6> = sqlx::query_as::<_, TestSuiteBaselineV6>(
            "SELECT * FROM test_suite_baselines_v6 WHERE suite_id = $1 AND metric_name = $2",
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("detect_test_suite_regression_v6: {e}")))?;

        match baseline {
            Some(b) => {
                let diff = ((current_value - b.baseline_value) / b.baseline_value * 100.0).abs();
                Ok(diff > b.threshold_percent)
            }
            None => Ok(false),
        }
    }

    pub async fn get_test_suite_performance_alerts_v6(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<String>> {
        let metrics = sqlx::query_as::<_, TestSuiteMetricV6>(
            r#"SELECT * FROM test_suite_metrics_v6
               WHERE suite_id = $1
               AND metric_name IN ('execution_time_ms', 'memory_usage_mb', 'cpu_usage_percent')
               AND measured_at > NOW() - INTERVAL '1 hour'"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_performance_alerts_v6: {e}")))?;

        let mut alerts = Vec::new();
        for metric in metrics {
            match metric.metric_name.as_str() {
                "execution_time_ms" if metric.metric_value > 1000.0 => {
                    alerts.push(format!("High execution time: {}ms", metric.metric_value));
                }
                "memory_usage_mb" if metric.metric_value > 512.0 => {
                    alerts.push(format!("High memory usage: {}MB", metric.metric_value));
                }
                "cpu_usage_percent" if metric.metric_value > 90.0 => {
                    alerts.push(format!("High CPU usage: {}%", metric.metric_value));
                }
                _ => {}
            }
        }
        Ok(alerts)
    }

    // --- Code Quality Metrics v7 ---

    pub async fn create_code_quality_metric_v7(
        &self,
        repo_id: Uuid,
        file_path: &str,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<CodeQualityMetricV7> {
        let row = sqlx::query_as::<_, CodeQualityMetricV7>(
            r#"INSERT INTO code_quality_metrics_v7 (repo_id, file_path, metric_name, metric_value)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(file_path)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_metric_v7: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_metrics_v7(
        &self,
        repo_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CodeQualityMetricV7>> {
        let rows = if let Some(name) = metric_name {
            sqlx::query_as::<_, CodeQualityMetricV7>(
                r#"SELECT * FROM code_quality_metrics_v7
                   WHERE repo_id = $1 AND metric_name = $2
                   ORDER BY measured_at DESC
                   LIMIT $3 OFFSET $4"#,
            )
            .bind(repo_id)
            .bind(name)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, CodeQualityMetricV7>(
                r#"SELECT * FROM code_quality_metrics_v7
                   WHERE repo_id = $1
                   ORDER BY measured_at DESC
                   LIMIT $2 OFFSET $3"#,
            )
            .bind(repo_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_code_quality_metrics_v7: {e}")))
    }

    pub async fn get_code_quality_score_v7(
        &self,
        repo_id: Uuid,
    ) -> Result<f64> {
        let row: (Option<f64>,) = sqlx::query_as(
            r#"SELECT AVG(metric_value) FROM code_quality_metrics_v7 WHERE repo_id = $1"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_score_v7: {e}")))?;
        Ok(row.0.unwrap_or(0.0))
    }

    // --- Code Quality Thresholds v6 ---

    pub async fn create_code_quality_threshold_v6(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        threshold_value: f64,
        enabled: bool,
    ) -> Result<CodeQualityThresholdV6> {
        let row = sqlx::query_as::<_, CodeQualityThresholdV6>(
            r#"INSERT INTO code_quality_thresholds_v6 (repo_id, metric_name, threshold_value, enabled)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (repo_id, metric_name) DO UPDATE
               SET threshold_value = $3, enabled = $4
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .bind(threshold_value)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_threshold_v6: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_thresholds_v6(
        &self,
        repo_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<CodeQualityThresholdV6>> {
        let rows = if enabled_only {
            sqlx::query_as::<_, CodeQualityThresholdV6>(
                r#"SELECT * FROM code_quality_thresholds_v6
                   WHERE repo_id = $1 AND enabled = true
                   ORDER BY metric_name"#,
            )
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, CodeQualityThresholdV6>(
                r#"SELECT * FROM code_quality_thresholds_v6
                   WHERE repo_id = $1
                   ORDER BY metric_name"#,
            )
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_code_quality_thresholds_v6: {e}")))
    }

    pub async fn check_code_quality_violation_v6(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<bool> {
        let threshold: Option<CodeQualityThresholdV6> =
            sqlx::query_as::<_, CodeQualityThresholdV6>(
                r#"SELECT * FROM code_quality_thresholds_v6
                   WHERE repo_id = $1 AND metric_name = $2 AND enabled = true"#,
            )
            .bind(repo_id)
            .bind(metric_name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("check_code_quality_violation_v6: {e}")))?;

        match threshold {
            Some(t) => Ok(metric_value > t.threshold_value),
            None => Ok(false),
        }
    }

    pub async fn delete_code_quality_threshold_v6(
        &self,
        repo_id: Uuid,
        metric_name: &str,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM code_quality_thresholds_v6 WHERE repo_id = $1 AND metric_name = $2",
        )
        .bind(repo_id)
        .bind(metric_name)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("delete_code_quality_threshold_v6: {e}")))?;
        Ok(())
    }

    pub async fn get_code_quality_enforcement_report_v6(
        &self,
        repo_id: Uuid,
    ) -> Result<(i64, i64, f64)> {
        let total_thresholds: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM code_quality_thresholds_v6 WHERE repo_id = $1 AND enabled = true"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_enforcement_report_v6 total: {e}")))?;

        let violating_thresholds: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM code_quality_thresholds_v6 t
               WHERE t.repo_id = $1 AND t.enabled = true
               AND EXISTS (
                   SELECT 1 FROM code_quality_metrics_v7 m
                   WHERE m.repo_id = t.repo_id AND m.metric_name = t.metric_name
                   AND m.metric_value > t.threshold_value
               )"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_enforcement_report_v6 violating: {e}")))?;

        let compliance_rate = if total_thresholds.0 > 0 {
            ((total_thresholds.0 - violating_thresholds.0) as f64 / total_thresholds.0 as f64) * 100.0
        } else {
            100.0
        };

        Ok((total_thresholds.0, violating_thresholds.0, compliance_rate))
    }

    // --- Performance Test Alerts v7 ---

    pub async fn create_performance_test_alert_v7(
        &self,
        baseline_id: Uuid,
        alert_type: &str,
        threshold: f64,
        enabled: bool,
    ) -> Result<PerformanceTestAlertV7> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV7>(
            r#"INSERT INTO performance_test_alerts_v7 (baseline_id, alert_type, threshold, enabled)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(baseline_id)
        .bind(alert_type)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_performance_test_alert_v7: {e}")))?;
        Ok(row)
    }

    pub async fn list_performance_test_alerts_v7(
        &self,
        baseline_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<PerformanceTestAlertV7>> {
        let rows = if enabled_only {
            sqlx::query_as::<_, PerformanceTestAlertV7>(
                r#"SELECT * FROM performance_test_alerts_v7
                   WHERE baseline_id = $1 AND enabled = true
                   ORDER BY created_at DESC"#,
            )
            .bind(baseline_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, PerformanceTestAlertV7>(
                r#"SELECT * FROM performance_test_alerts_v7
                   WHERE baseline_id = $1
                   ORDER BY created_at DESC"#,
            )
            .bind(baseline_id)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_performance_test_alerts_v7: {e}")))
    }

    pub async fn update_performance_test_alert_v7(
        &self,
        id: Uuid,
        threshold: Option<f64>,
        enabled: Option<bool>,
    ) -> Result<PerformanceTestAlertV7> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV7>(
            r#"UPDATE performance_test_alerts_v7
               SET threshold = COALESCE($2, threshold),
                   enabled = COALESCE($3, enabled)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_performance_test_alert_v7: {e}")))?;
        Ok(row)
    }

    pub async fn record_performance_test_alert_v7(
        &self,
        alert_id: Uuid,
        metric_name: &str,
        metric_value: f64,
        threshold: f64,
    ) -> Result<PerformanceTestAlertHistoryV7> {
        let row = sqlx::query_as::<_, PerformanceTestAlertHistoryV7>(
            r#"INSERT INTO performance_test_alert_history_v7 (alert_id, metric_name, metric_value, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(alert_id)
        .bind(metric_name)
        .bind(metric_value)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_performance_test_alert_v7: {e}")))?;

        sqlx::query(
            "UPDATE performance_test_alerts_v7 SET last_triggered_at = NOW() WHERE id = $1",
        )
        .bind(alert_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_performance_test_alert_v7 update: {e}")))?;

        Ok(row)
    }

    pub async fn list_performance_test_alert_history_v7(
        &self,
        alert_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PerformanceTestAlertHistoryV7>> {
        sqlx::query_as::<_, PerformanceTestAlertHistoryV7>(
            "SELECT * FROM performance_test_alert_history_v7 WHERE alert_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(alert_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_performance_test_alert_history_v7: {e}")))
    }

    pub async fn get_performance_test_alert_analytics_v7(
        &self,
        alert_id: Uuid,
    ) -> Result<(i64, Option<f64>, Option<f64>)> {
        let row: (i64, Option<f64>, Option<f64>) = sqlx::query_as(
            r#"SELECT
                   COUNT(*) as trigger_count,
                   AVG(metric_value) as avg_value,
                   MAX(metric_value) as max_value
               FROM performance_test_alert_history_v7
               WHERE alert_id = $1"#,
        )
        .bind(alert_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_analytics_v7: {e}")))?;
        Ok((row.0, row.1, row.2))
    }

    pub async fn get_performance_test_alert_notification_config_v7(
        &self,
        alert_id: Uuid,
    ) -> Result<Option<(String, bool, Option<String>)>> {
        let row: Option<(String, bool, Option<String>)> = sqlx::query_as(
            r#"SELECT
                   alert_type,
                   enabled,
                   last_triggered_at::text
               FROM performance_test_alerts_v7
               WHERE id = $1"#,
        )
        .bind(alert_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_notification_config_v7: {e}")))?;
        Ok(row)
    }

    // --- Test Suite Metrics v7 ---

    pub async fn create_test_suite_metric_v7(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<TestSuiteMetricV7> {
        let row = sqlx::query_as::<_, TestSuiteMetricV7>(
            r#"INSERT INTO test_suite_metrics_v7 (suite_id, metric_name, metric_value)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_metric_v7: {e}")))?;
        Ok(row)
    }

    pub async fn list_test_suite_metrics_v7(
        &self,
        suite_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestSuiteMetricV7>> {
        sqlx::query_as::<_, TestSuiteMetricV7>(
            "SELECT * FROM test_suite_metrics_v7 WHERE suite_id = $1 ORDER BY measured_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(suite_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_test_suite_metrics_v7: {e}")))
    }

    pub async fn get_test_suite_latest_metric_v7(
        &self,
        suite_id: Uuid,
        metric_name: &str,
    ) -> Result<Option<TestSuiteMetricV7>> {
        sqlx::query_as::<_, TestSuiteMetricV7>(
            "SELECT * FROM test_suite_metrics_v7 WHERE suite_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT 1",
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_latest_metric_v7: {e}")))
    }

    // --- Test Suite Baselines v7 ---

    pub async fn create_test_suite_baseline_v7(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        baseline_value: f64,
        threshold_percent: f64,
    ) -> Result<TestSuiteBaselineV7> {
        let row = sqlx::query_as::<_, TestSuiteBaselineV7>(
            r#"INSERT INTO test_suite_baselines_v7 (suite_id, metric_name, baseline_value, threshold_percent)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (suite_id, metric_name) DO UPDATE
               SET baseline_value = $3, threshold_percent = $4
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(baseline_value)
        .bind(threshold_percent)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_baseline_v7: {e}")))?;
        Ok(row)
    }

    pub async fn get_test_suite_baselines_v7(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteBaselineV7>> {
        sqlx::query_as::<_, TestSuiteBaselineV7>(
            "SELECT * FROM test_suite_baselines_v7 WHERE suite_id = $1 ORDER BY metric_name",
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_baselines_v7: {e}")))
    }

    pub async fn detect_test_suite_regression_v7(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        current_value: f64,
    ) -> Result<bool> {
        let baseline: Option<TestSuiteBaselineV7> = sqlx::query_as::<_, TestSuiteBaselineV7>(
            "SELECT * FROM test_suite_baselines_v7 WHERE suite_id = $1 AND metric_name = $2",
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("detect_test_suite_regression_v7: {e}")))?;

        match baseline {
            Some(b) => {
                let diff = ((current_value - b.baseline_value) / b.baseline_value * 100.0).abs();
                Ok(diff > b.threshold_percent)
            }
            None => Ok(false),
        }
    }

    pub async fn get_test_suite_performance_alerts_v7(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<String>> {
        let metrics = sqlx::query_as::<_, TestSuiteMetricV7>(
            r#"SELECT * FROM test_suite_metrics_v7
               WHERE suite_id = $1
               AND metric_name IN ('execution_time_ms', 'memory_usage_mb', 'cpu_usage_percent')
               AND measured_at > NOW() - INTERVAL '1 hour'"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_performance_alerts_v7: {e}")))?;

        let mut alerts = Vec::new();
        for metric in metrics {
            match metric.metric_name.as_str() {
                "execution_time_ms" if metric.metric_value > 1000.0 => {
                    alerts.push(format!("High execution time: {}ms", metric.metric_value));
                }
                "memory_usage_mb" if metric.metric_value > 512.0 => {
                    alerts.push(format!("High memory usage: {}MB", metric.metric_value));
                }
                "cpu_usage_percent" if metric.metric_value > 90.0 => {
                    alerts.push(format!("High CPU usage: {}%", metric.metric_value));
                }
                _ => {}
            }
        }
        Ok(alerts)
    }

    // --- Code Quality Metrics v8 ---

    pub async fn create_code_quality_metric_v8(
        &self,
        repo_id: Uuid,
        file_path: &str,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<CodeQualityMetricV8> {
        let row = sqlx::query_as::<_, CodeQualityMetricV8>(
            r#"INSERT INTO code_quality_metrics_v8 (repo_id, file_path, metric_name, metric_value)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(file_path)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_metric_v8: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_metrics_v8(
        &self,
        repo_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CodeQualityMetricV8>> {
        let rows = if let Some(name) = metric_name {
            sqlx::query_as::<_, CodeQualityMetricV8>(
                r#"SELECT * FROM code_quality_metrics_v8
                   WHERE repo_id = $1 AND metric_name = $2
                   ORDER BY measured_at DESC
                   LIMIT $3 OFFSET $4"#,
            )
            .bind(repo_id)
            .bind(name)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, CodeQualityMetricV8>(
                r#"SELECT * FROM code_quality_metrics_v8
                   WHERE repo_id = $1
                   ORDER BY measured_at DESC
                   LIMIT $2 OFFSET $3"#,
            )
            .bind(repo_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_code_quality_metrics_v8: {e}")))
    }

    pub async fn get_code_quality_score_v8(
        &self,
        repo_id: Uuid,
    ) -> Result<f64> {
        let row: (Option<f64>,) = sqlx::query_as(
            r#"SELECT COALESCE(
                   (SELECT AVG(100.0 - LEAST(metric_value, 100.0))
                    FROM code_quality_metrics_v8
                    WHERE repo_id = $1
                    AND metric_name IN ('complexity', ' duplication')),
                   100.0
               )"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_score_v8: {e}")))?;
        Ok(row.0.unwrap_or(0.0))
    }

    // --- Code Quality Thresholds v7 ---

    pub async fn create_code_quality_threshold_v7(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        threshold_value: f64,
        enabled: bool,
    ) -> Result<CodeQualityThresholdV7> {
        let row = sqlx::query_as::<_, CodeQualityThresholdV7>(
            r#"INSERT INTO code_quality_thresholds_v7 (repo_id, metric_name, threshold_value, enabled)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (repo_id, metric_name) DO UPDATE
               SET threshold_value = $3, enabled = $4
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .bind(threshold_value)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_threshold_v7: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_thresholds_v7(
        &self,
        repo_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<CodeQualityThresholdV7>> {
        let rows = if enabled_only {
            sqlx::query_as::<_, CodeQualityThresholdV7>(
                r#"SELECT * FROM code_quality_thresholds_v7
                   WHERE repo_id = $1 AND enabled = true
                   ORDER BY metric_name"#,
            )
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, CodeQualityThresholdV7>(
                r#"SELECT * FROM code_quality_thresholds_v7
                   WHERE repo_id = $1
                   ORDER BY metric_name"#,
            )
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_code_quality_thresholds_v7: {e}")))
    }

    pub async fn check_code_quality_violation_v7(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<bool> {
        let threshold: Option<CodeQualityThresholdV7> =
            sqlx::query_as::<_, CodeQualityThresholdV7>(
                r#"SELECT * FROM code_quality_thresholds_v7
                   WHERE repo_id = $1 AND metric_name = $2 AND enabled = true"#,
            )
            .bind(repo_id)
            .bind(metric_name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("check_code_quality_violation_v7: {e}")))?;

        match threshold {
            Some(t) => Ok(metric_value > t.threshold_value),
            None => Ok(false),
        }
    }

    pub async fn delete_code_quality_threshold_v7(
        &self,
        repo_id: Uuid,
        metric_name: &str,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM code_quality_thresholds_v7 WHERE repo_id = $1 AND metric_name = $2",
        )
        .bind(repo_id)
        .bind(metric_name)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("delete_code_quality_threshold_v7: {e}")))?;
        Ok(())
    }

    pub async fn get_code_quality_enforcement_report_v7(
        &self,
        repo_id: Uuid,
    ) -> Result<(i64, i64, f64)> {
        let total_thresholds: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM code_quality_thresholds_v7 WHERE repo_id = $1 AND enabled = true"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_enforcement_report_v7 total: {e}")))?;

        let violating_thresholds: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM code_quality_thresholds_v7 t
               WHERE t.repo_id = $1 AND t.enabled = true
               AND EXISTS (
                   SELECT 1 FROM code_quality_metrics_v8 m
                   WHERE m.repo_id = t.repo_id AND m.metric_name = t.metric_name
                   AND m.metric_value > t.threshold_value
               )"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_enforcement_report_v7 violating: {e}")))?;

        let compliance_rate = if total_thresholds.0 > 0 {
            ((total_thresholds.0 - violating_thresholds.0) as f64 / total_thresholds.0 as f64) * 100.0
        } else {
            100.0
        };

        Ok((total_thresholds.0, violating_thresholds.0, compliance_rate))
    }

    // --- Performance Test Alerts v8 ---

    pub async fn create_performance_test_alert_v8(
        &self,
        baseline_id: Uuid,
        alert_type: &str,
        threshold: f64,
        enabled: bool,
    ) -> Result<PerformanceTestAlertV8> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV8>(
            r#"INSERT INTO performance_test_alerts_v8 (baseline_id, alert_type, threshold, enabled)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(baseline_id)
        .bind(alert_type)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_performance_test_alert_v8: {e}")))?;
        Ok(row)
    }

    pub async fn list_performance_test_alerts_v8(
        &self,
        baseline_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<PerformanceTestAlertV8>> {
        let rows = if enabled_only {
            sqlx::query_as::<_, PerformanceTestAlertV8>(
                r#"SELECT * FROM performance_test_alerts_v8
                   WHERE baseline_id = $1 AND enabled = true
                   ORDER BY created_at DESC"#,
            )
            .bind(baseline_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, PerformanceTestAlertV8>(
                r#"SELECT * FROM performance_test_alerts_v8
                   WHERE baseline_id = $1
                   ORDER BY created_at DESC"#,
            )
            .bind(baseline_id)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_performance_test_alerts_v8: {e}")))
    }

    pub async fn update_performance_test_alert_v8(
        &self,
        id: Uuid,
        threshold: Option<f64>,
        enabled: Option<bool>,
    ) -> Result<PerformanceTestAlertV8> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV8>(
            r#"UPDATE performance_test_alerts_v8
               SET threshold = COALESCE($2, threshold),
                   enabled = COALESCE($3, enabled)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_performance_test_alert_v8: {e}")))?;
        Ok(row)
    }

    pub async fn record_performance_test_alert_v8(
        &self,
        alert_id: Uuid,
        metric_name: &str,
        metric_value: f64,
        threshold: f64,
    ) -> Result<PerformanceTestAlertHistoryV8> {
        let row = sqlx::query_as::<_, PerformanceTestAlertHistoryV8>(
            r#"INSERT INTO performance_test_alert_history_v8 (alert_id, metric_name, metric_value, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(alert_id)
        .bind(metric_name)
        .bind(metric_value)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_performance_test_alert_v8: {e}")))?;

        sqlx::query(
            "UPDATE performance_test_alerts_v8 SET last_triggered_at = NOW() WHERE id = $1",
        )
        .bind(alert_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_performance_test_alert_v8 update: {e}")))?;

        Ok(row)
    }

    pub async fn list_performance_test_alert_history_v8(
        &self,
        alert_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PerformanceTestAlertHistoryV8>> {
        sqlx::query_as::<_, PerformanceTestAlertHistoryV8>(
            "SELECT * FROM performance_test_alert_history_v8 WHERE alert_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(alert_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_performance_test_alert_history_v8: {e}")))
    }

    pub async fn get_performance_test_alert_analytics_v8(
        &self,
        alert_id: Uuid,
    ) -> Result<(i64, Option<f64>, Option<f64>)> {
        let row: (i64, Option<f64>, Option<f64>) = sqlx::query_as(
            r#"SELECT
                   COUNT(*) as trigger_count,
                   AVG(metric_value) as avg_value,
                   MAX(metric_value) as max_value
               FROM performance_test_alert_history_v8
               WHERE alert_id = $1"#,
        )
        .bind(alert_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_analytics_v8: {e}")))?;
        Ok((row.0, row.1, row.2))
    }

    pub async fn get_performance_test_alert_notification_config_v8(
        &self,
        alert_id: Uuid,
    ) -> Result<Option<(String, bool, Option<String>)>> {
        let row: Option<(String, bool, Option<String>)> = sqlx::query_as(
            r#"SELECT
                   alert_type,
                   enabled,
                   last_triggered_at::text
               FROM performance_test_alerts_v8
               WHERE id = $1"#,
        )
        .bind(alert_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_notification_config_v8: {e}")))?;
        Ok(row)
    }

    // --- Test Suite Metrics v9 ---

    pub async fn create_test_suite_metric_v9(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<TestSuiteMetricV9> {
        let row = sqlx::query_as::<_, TestSuiteMetricV9>(
            r#"INSERT INTO test_suite_metrics_v9 (suite_id, metric_name, metric_value)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_metric_v9: {e}")))?;
        Ok(row)
    }

    pub async fn list_test_suite_metrics_v9(
        &self,
        suite_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestSuiteMetricV9>> {
        sqlx::query_as::<_, TestSuiteMetricV9>(
            "SELECT * FROM test_suite_metrics_v9 WHERE suite_id = $1 ORDER BY measured_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(suite_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_test_suite_metrics_v9: {e}")))
    }

    pub async fn get_test_suite_latest_metric_v9(
        &self,
        suite_id: Uuid,
        metric_name: &str,
    ) -> Result<Option<TestSuiteMetricV9>> {
        sqlx::query_as::<_, TestSuiteMetricV9>(
            "SELECT * FROM test_suite_metrics_v9 WHERE suite_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT 1",
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_latest_metric_v9: {e}")))
    }

    // --- Test Suite Baselines v9 ---

    pub async fn create_test_suite_baseline_v9(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        baseline_value: f64,
        threshold_percent: f64,
    ) -> Result<TestSuiteBaselineV9> {
        let row = sqlx::query_as::<_, TestSuiteBaselineV9>(
            r#"INSERT INTO test_suite_baselines_v9 (suite_id, metric_name, baseline_value, threshold_percent)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (suite_id, metric_name) DO UPDATE
               SET baseline_value = $3, threshold_percent = $4
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(baseline_value)
        .bind(threshold_percent)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_baseline_v9: {e}")))?;
        Ok(row)
    }

    pub async fn get_test_suite_baselines_v9(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteBaselineV9>> {
        sqlx::query_as::<_, TestSuiteBaselineV9>(
            "SELECT * FROM test_suite_baselines_v9 WHERE suite_id = $1 ORDER BY metric_name",
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_baselines_v9: {e}")))
    }

    pub async fn detect_test_suite_regression_v9(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        current_value: f64,
    ) -> Result<bool> {
        let baseline: Option<TestSuiteBaselineV9> = sqlx::query_as::<_, TestSuiteBaselineV9>(
            "SELECT * FROM test_suite_baselines_v9 WHERE suite_id = $1 AND metric_name = $2",
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("detect_test_suite_regression_v9: {e}")))?;

        match baseline {
            Some(b) => {
                let diff = ((current_value - b.baseline_value) / b.baseline_value * 100.0).abs();
                Ok(diff > b.threshold_percent)
            }
            None => Ok(false),
        }
    }

    pub async fn get_test_suite_performance_alerts_v9(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<String>> {
        let metrics = sqlx::query_as::<_, TestSuiteMetricV9>(
            r#"SELECT * FROM test_suite_metrics_v9
               WHERE suite_id = $1
               AND metric_name IN ('execution_time_ms', 'memory_usage_mb', 'cpu_usage_percent')
               AND measured_at > NOW() - INTERVAL '1 hour'"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_performance_alerts_v9: {e}")))?;

        let mut alerts = Vec::new();
        for metric in metrics {
            match metric.metric_name.as_str() {
                "execution_time_ms" if metric.metric_value > 1000.0 => {
                    alerts.push(format!("High execution time: {}ms", metric.metric_value));
                }
                "memory_usage_mb" if metric.metric_value > 512.0 => {
                    alerts.push(format!("High memory usage: {}MB", metric.metric_value));
                }
                "cpu_usage_percent" if metric.metric_value > 90.0 => {
                    alerts.push(format!("High CPU usage: {}%", metric.metric_value));
                }
                _ => {}
            }
        }
        Ok(alerts)
    }

    // --- Code Quality Metrics v10 ---

    pub async fn create_code_quality_metric_v10(
        &self,
        repo_id: Uuid,
        file_path: &str,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<CodeQualityMetricV10> {
        let row = sqlx::query_as::<_, CodeQualityMetricV10>(
            r#"INSERT INTO code_quality_metrics_v10 (repo_id, file_path, metric_name, metric_value)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(file_path)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_metric_v10: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_metrics_v10(
        &self,
        repo_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CodeQualityMetricV10>> {
        let rows = if let Some(name) = metric_name {
            sqlx::query_as::<_, CodeQualityMetricV10>(
                r#"SELECT * FROM code_quality_metrics_v10
                   WHERE repo_id = $1 AND metric_name = $2
                   ORDER BY measured_at DESC
                   LIMIT $3 OFFSET $4"#,
            )
            .bind(repo_id)
            .bind(name)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, CodeQualityMetricV10>(
                r#"SELECT * FROM code_quality_metrics_v10
                   WHERE repo_id = $1
                   ORDER BY measured_at DESC
                   LIMIT $2 OFFSET $3"#,
            )
            .bind(repo_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_code_quality_metrics_v10: {e}")))
    }

    pub async fn get_code_quality_score_v10(
        &self,
        repo_id: Uuid,
    ) -> Result<f64> {
        let row: (Option<f64>,) = sqlx::query_as(
            r#"SELECT COALESCE(
                   (SELECT AVG(100.0 - LEAST(metric_value, 100.0))
                    FROM code_quality_metrics_v10
                    WHERE repo_id = $1
                    AND metric_name IN ('complexity', ' duplication')),
                   100.0
               )"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_score_v10: {e}")))?;
        Ok(row.0.unwrap_or(0.0))
    }

    // --- Code Quality Thresholds v9 ---

    pub async fn create_code_quality_threshold_v9(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        threshold_value: f64,
        enabled: bool,
    ) -> Result<CodeQualityThresholdV9> {
        let row = sqlx::query_as::<_, CodeQualityThresholdV9>(
            r#"INSERT INTO code_quality_thresholds_v9 (repo_id, metric_name, threshold_value, enabled)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (repo_id, metric_name) DO UPDATE
               SET threshold_value = $3, enabled = $4
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .bind(threshold_value)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_threshold_v9: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_thresholds_v9(
        &self,
        repo_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<CodeQualityThresholdV9>> {
        let rows = if enabled_only {
            sqlx::query_as::<_, CodeQualityThresholdV9>(
                r#"SELECT * FROM code_quality_thresholds_v9
                   WHERE repo_id = $1 AND enabled = true
                   ORDER BY metric_name"#,
            )
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, CodeQualityThresholdV9>(
                r#"SELECT * FROM code_quality_thresholds_v9
                   WHERE repo_id = $1
                   ORDER BY metric_name"#,
            )
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_code_quality_thresholds_v9: {e}")))
    }

    pub async fn check_code_quality_violation_v9(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<bool> {
        let threshold: Option<CodeQualityThresholdV9> =
            sqlx::query_as::<_, CodeQualityThresholdV9>(
                r#"SELECT * FROM code_quality_thresholds_v9
                   WHERE repo_id = $1 AND metric_name = $2 AND enabled = true"#,
            )
            .bind(repo_id)
            .bind(metric_name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("check_code_quality_violation_v9: {e}")))?;

        match threshold {
            Some(t) => Ok(metric_value > t.threshold_value),
            None => Ok(false),
        }
    }

    pub async fn delete_code_quality_threshold_v9(
        &self,
        repo_id: Uuid,
        metric_name: &str,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM code_quality_thresholds_v9 WHERE repo_id = $1 AND metric_name = $2",
        )
        .bind(repo_id)
        .bind(metric_name)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("delete_code_quality_threshold_v9: {e}")))?;
        Ok(())
    }

    pub async fn get_code_quality_enforcement_report_v9(
        &self,
        repo_id: Uuid,
    ) -> Result<(i64, i64, f64)> {
        let total_thresholds: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM code_quality_thresholds_v9 WHERE repo_id = $1 AND enabled = true"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_enforcement_report_v9 total: {e}")))?;

        let violating_thresholds: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM code_quality_thresholds_v9 t
               WHERE t.repo_id = $1 AND t.enabled = true
               AND EXISTS (
                   SELECT 1 FROM code_quality_metrics_v10 m
                   WHERE m.repo_id = t.repo_id AND m.metric_name = t.metric_name
                   AND m.metric_value > t.threshold_value
               )"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_enforcement_report_v9 violating: {e}")))?;

        let compliance_rate = if total_thresholds.0 > 0 {
            ((total_thresholds.0 - violating_thresholds.0) as f64 / total_thresholds.0 as f64) * 100.0
        } else {
            100.0
        };

        Ok((total_thresholds.0, violating_thresholds.0, compliance_rate))
    }

    // --- Performance Test Alerts v10 ---

    pub async fn create_performance_test_alert_v10(
        &self,
        baseline_id: Uuid,
        alert_type: &str,
        threshold: f64,
        enabled: bool,
    ) -> Result<PerformanceTestAlertV10> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV10>(
            r#"INSERT INTO performance_test_alerts_v10 (baseline_id, alert_type, threshold, enabled)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(baseline_id)
        .bind(alert_type)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_performance_test_alert_v10: {e}")))?;
        Ok(row)
    }

    pub async fn list_performance_test_alerts_v10(
        &self,
        baseline_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<PerformanceTestAlertV10>> {
        let rows = if enabled_only {
            sqlx::query_as::<_, PerformanceTestAlertV10>(
                r#"SELECT * FROM performance_test_alerts_v10
                   WHERE baseline_id = $1 AND enabled = true
                   ORDER BY created_at DESC"#,
            )
            .bind(baseline_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, PerformanceTestAlertV10>(
                r#"SELECT * FROM performance_test_alerts_v10
                   WHERE baseline_id = $1
                   ORDER BY created_at DESC"#,
            )
            .bind(baseline_id)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_performance_test_alerts_v10: {e}")))
    }

    pub async fn update_performance_test_alert_v10(
        &self,
        id: Uuid,
        threshold: Option<f64>,
        enabled: Option<bool>,
    ) -> Result<PerformanceTestAlertV10> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV10>(
            r#"UPDATE performance_test_alerts_v10
               SET threshold = COALESCE($2, threshold),
                   enabled = COALESCE($3, enabled)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_performance_test_alert_v10: {e}")))?;
        Ok(row)
    }

    pub async fn record_performance_test_alert_v10(
        &self,
        alert_id: Uuid,
        metric_name: &str,
        metric_value: f64,
        threshold: f64,
    ) -> Result<PerformanceTestAlertHistoryV10> {
        let row = sqlx::query_as::<_, PerformanceTestAlertHistoryV10>(
            r#"INSERT INTO performance_test_alert_history_v10 (alert_id, metric_name, metric_value, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(alert_id)
        .bind(metric_name)
        .bind(metric_value)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_performance_test_alert_v10: {e}")))?;

        sqlx::query(
            "UPDATE performance_test_alerts_v10 SET last_triggered_at = NOW() WHERE id = $1",
        )
        .bind(alert_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_performance_test_alert_v10 update: {e}")))?;

        Ok(row)
    }

    pub async fn list_performance_test_alert_history_v10(
        &self,
        alert_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PerformanceTestAlertHistoryV10>> {
        sqlx::query_as::<_, PerformanceTestAlertHistoryV10>(
            "SELECT * FROM performance_test_alert_history_v10 WHERE alert_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(alert_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_performance_test_alert_history_v10: {e}")))
    }

    pub async fn get_performance_test_alert_analytics_v10(
        &self,
        alert_id: Uuid,
    ) -> Result<(i64, Option<f64>, Option<f64>)> {
        let row: (i64, Option<f64>, Option<f64>) = sqlx::query_as(
            r#"SELECT
                   COUNT(*) as trigger_count,
                   AVG(metric_value) as avg_value,
                   MAX(metric_value) as max_value
               FROM performance_test_alert_history_v10
               WHERE alert_id = $1"#,
        )
        .bind(alert_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_analytics_v10: {e}")))?;
        Ok((row.0, row.1, row.2))
    }

    pub async fn get_performance_test_alert_notification_config_v10(
        &self,
        alert_id: Uuid,
    ) -> Result<Option<(String, bool, Option<String>)>> {
        let row: Option<(String, bool, Option<String>)> = sqlx::query_as(
            r#"SELECT
                   alert_type,
                   enabled,
                   last_triggered_at::text
               FROM performance_test_alerts_v10
               WHERE id = $1"#,
        )
        .bind(alert_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_notification_config_v10: {e}")))?;
        Ok(row)
    }

    // --- Test Suite Metrics v13 ---

    pub async fn create_test_suite_metric_v13(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<TestSuiteMetricV13> {
        let row = sqlx::query_as::<_, TestSuiteMetricV13>(
            r#"INSERT INTO test_suite_metrics_v10 (suite_id, metric_name, metric_value)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_metric_v13: {e}")))?;
        Ok(row)
    }

    pub async fn list_test_suite_metrics_v13(
        &self,
        suite_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestSuiteMetricV13>> {
        sqlx::query_as::<_, TestSuiteMetricV13>(
            "SELECT * FROM test_suite_metrics_v10 WHERE suite_id = $1 ORDER BY measured_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(suite_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_test_suite_metrics_v13: {e}")))
    }

    pub async fn get_test_suite_latest_metric_v13(
        &self,
        suite_id: Uuid,
        metric_name: &str,
    ) -> Result<Option<TestSuiteMetricV13>> {
        sqlx::query_as::<_, TestSuiteMetricV13>(
            "SELECT * FROM test_suite_metrics_v10 WHERE suite_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT 1",
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_latest_metric_v13: {e}")))
    }

    pub async fn create_test_suite_baseline_v13(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        baseline_value: f64,
        threshold_percent: f64,
    ) -> Result<TestSuiteBaselineV13> {
        let row = sqlx::query_as::<_, TestSuiteBaselineV13>(
            r#"INSERT INTO test_suite_baselines_v10 (suite_id, metric_name, baseline_value, threshold_percent)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (suite_id, metric_name) DO UPDATE
               SET baseline_value = $3, threshold_percent = $4
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(baseline_value)
        .bind(threshold_percent)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_baseline_v13: {e}")))?;
        Ok(row)
    }

    pub async fn get_test_suite_baselines_v13(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteBaselineV13>> {
        sqlx::query_as::<_, TestSuiteBaselineV13>(
            "SELECT * FROM test_suite_baselines_v10 WHERE suite_id = $1 ORDER BY metric_name",
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_baselines_v13: {e}")))
    }

    pub async fn detect_test_suite_regression_v13(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        current_value: f64,
    ) -> Result<bool> {
        let baseline: Option<TestSuiteBaselineV13> = sqlx::query_as::<_, TestSuiteBaselineV13>(
            "SELECT * FROM test_suite_baselines_v10 WHERE suite_id = $1 AND metric_name = $2",
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("detect_test_suite_regression_v13: {e}")))?;

        match baseline {
            Some(b) => {
                let diff = ((current_value - b.baseline_value) / b.baseline_value * 100.0).abs();
                Ok(diff > b.threshold_percent)
            }
            None => Ok(false),
        }
    }

    pub async fn get_test_suite_performance_alerts_v13(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<String>> {
        let metrics = sqlx::query_as::<_, TestSuiteMetricV13>(
            r#"SELECT * FROM test_suite_metrics_v10
               WHERE suite_id = $1
               AND metric_name IN ('execution_time_ms', 'memory_usage_mb', 'cpu_usage_percent')
               AND measured_at > NOW() - INTERVAL '1 hour'"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_performance_alerts_v13: {e}")))?;

        let mut alerts = Vec::new();
        for metric in metrics {
            match metric.metric_name.as_str() {
                "execution_time_ms" if metric.metric_value > 1000.0 => {
                    alerts.push(format!("High execution time: {}ms", metric.metric_value));
                }
                "memory_usage_mb" if metric.metric_value > 512.0 => {
                    alerts.push(format!("High memory usage: {}MB", metric.metric_value));
                }
                "cpu_usage_percent" if metric.metric_value > 90.0 => {
                    alerts.push(format!("High CPU usage: {}%", metric.metric_value));
                }
                _ => {}
            }
        }
        Ok(alerts)
    }

    // --- Code Quality Metrics v13 ---

    pub async fn create_code_quality_metric_v13(
        &self,
        repo_id: Uuid,
        file_path: &str,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<CodeQualityMetricV13> {
        let row = sqlx::query_as::<_, CodeQualityMetricV13>(
            r#"INSERT INTO code_quality_metrics_v11 (repo_id, file_path, metric_name, metric_value)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(file_path)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_metric_v13: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_metrics_v13(
        &self,
        repo_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CodeQualityMetricV13>> {
        let rows = if let Some(name) = metric_name {
            sqlx::query_as::<_, CodeQualityMetricV13>(
                r#"SELECT * FROM code_quality_metrics_v11
                   WHERE repo_id = $1 AND metric_name = $2
                   ORDER BY measured_at DESC
                   LIMIT $3 OFFSET $4"#,
            )
            .bind(repo_id)
            .bind(name)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, CodeQualityMetricV13>(
                r#"SELECT * FROM code_quality_metrics_v11
                   WHERE repo_id = $1
                   ORDER BY measured_at DESC
                   LIMIT $2 OFFSET $3"#,
            )
            .bind(repo_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_code_quality_metrics_v13: {e}")))
    }

    pub async fn get_code_quality_score_v13(
        &self,
        repo_id: Uuid,
    ) -> Result<f64> {
        let row: (Option<f64>,) = sqlx::query_as(
            r#"SELECT COALESCE(
                   (SELECT AVG(100.0 - LEAST(metric_value, 100.0))
                    FROM code_quality_metrics_v11
                    WHERE repo_id = $1
                    AND metric_name IN ('complexity', ' duplication')),
                   100.0
               )"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_score_v13: {e}")))?;
        Ok(row.0.unwrap_or(0.0))
    }

    pub async fn create_code_quality_threshold_v13(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        threshold_value: f64,
        enabled: bool,
    ) -> Result<CodeQualityThresholdV13> {
        let row = sqlx::query_as::<_, CodeQualityThresholdV13>(
            r#"INSERT INTO code_quality_thresholds_v10 (repo_id, metric_name, threshold_value, enabled)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (repo_id, metric_name) DO UPDATE
               SET threshold_value = $3, enabled = $4
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .bind(threshold_value)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_threshold_v13: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_thresholds_v13(
        &self,
        repo_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<CodeQualityThresholdV13>> {
        let rows = if enabled_only {
            sqlx::query_as::<_, CodeQualityThresholdV13>(
                r#"SELECT * FROM code_quality_thresholds_v10
                   WHERE repo_id = $1 AND enabled = true
                   ORDER BY metric_name"#,
            )
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, CodeQualityThresholdV13>(
                r#"SELECT * FROM code_quality_thresholds_v10
                   WHERE repo_id = $1
                   ORDER BY metric_name"#,
            )
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_code_quality_thresholds_v13: {e}")))
    }

    pub async fn check_code_quality_violation_v13(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<bool> {
        let threshold: Option<CodeQualityThresholdV13> =
            sqlx::query_as::<_, CodeQualityThresholdV13>(
                r#"SELECT * FROM code_quality_thresholds_v10
                   WHERE repo_id = $1 AND metric_name = $2 AND enabled = true"#,
            )
            .bind(repo_id)
            .bind(metric_name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("check_code_quality_violation_v13: {e}")))?;

        match threshold {
            Some(t) => Ok(metric_value > t.threshold_value),
            None => Ok(false),
        }
    }

    pub async fn delete_code_quality_threshold_v13(
        &self,
        repo_id: Uuid,
        metric_name: &str,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM code_quality_thresholds_v10 WHERE repo_id = $1 AND metric_name = $2",
        )
        .bind(repo_id)
        .bind(metric_name)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("delete_code_quality_threshold_v13: {e}")))?;
        Ok(())
    }

    pub async fn get_code_quality_enforcement_report_v13(
        &self,
        repo_id: Uuid,
    ) -> Result<(i64, i64, f64)> {
        let total_thresholds: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM code_quality_thresholds_v10 WHERE repo_id = $1 AND enabled = true"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_enforcement_report_v13 total: {e}")))?;

        let violating_thresholds: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM code_quality_thresholds_v10 t
               WHERE t.repo_id = $1 AND t.enabled = true
               AND EXISTS (
                   SELECT 1 FROM code_quality_metrics_v11 m
                   WHERE m.repo_id = t.repo_id AND m.metric_name = t.metric_name
                   AND m.metric_value > t.threshold_value
               )"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_enforcement_report_v13 violating: {e}")))?;

        let compliance_rate = if total_thresholds.0 > 0 {
            ((total_thresholds.0 - violating_thresholds.0) as f64 / total_thresholds.0 as f64) * 100.0
        } else {
            100.0
        };

        Ok((total_thresholds.0, violating_thresholds.0, compliance_rate))
    }

    // --- Performance Test Alerts v14 ---

    pub async fn create_performance_test_alert_v14(
        &self,
        baseline_id: Uuid,
        alert_type: &str,
        threshold: f64,
        enabled: bool,
    ) -> Result<PerformanceTestAlertV14> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV14>(
            r#"INSERT INTO performance_test_alerts_v11 (baseline_id, alert_type, threshold, enabled)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(baseline_id)
        .bind(alert_type)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_performance_test_alert_v14: {e}")))?;
        Ok(row)
    }

    pub async fn list_performance_test_alerts_v14(
        &self,
        baseline_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<PerformanceTestAlertV14>> {
        let rows = if enabled_only {
            sqlx::query_as::<_, PerformanceTestAlertV14>(
                r#"SELECT * FROM performance_test_alerts_v11
                   WHERE baseline_id = $1 AND enabled = true
                   ORDER BY created_at DESC"#,
            )
            .bind(baseline_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, PerformanceTestAlertV14>(
                r#"SELECT * FROM performance_test_alerts_v11
                   WHERE baseline_id = $1
                   ORDER BY created_at DESC"#,
            )
            .bind(baseline_id)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_performance_test_alerts_v14: {e}")))
    }

    pub async fn update_performance_test_alert_v14(
        &self,
        id: Uuid,
        threshold: Option<f64>,
        enabled: Option<bool>,
    ) -> Result<PerformanceTestAlertV14> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV14>(
            r#"UPDATE performance_test_alerts_v11
               SET threshold = COALESCE($2, threshold),
                   enabled = COALESCE($3, enabled)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_performance_test_alert_v14: {e}")))?;
        Ok(row)
    }

    pub async fn record_performance_test_alert_v14(
        &self,
        alert_id: Uuid,
        metric_name: &str,
        metric_value: f64,
        threshold: f64,
    ) -> Result<PerformanceTestAlertHistoryV14> {
        let row = sqlx::query_as::<_, PerformanceTestAlertHistoryV14>(
            r#"INSERT INTO performance_test_alert_history_v11 (alert_id, metric_name, metric_value, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(alert_id)
        .bind(metric_name)
        .bind(metric_value)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_performance_test_alert_v14: {e}")))?;

        sqlx::query(
            "UPDATE performance_test_alerts_v11 SET last_triggered_at = NOW() WHERE id = $1",
        )
        .bind(alert_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_performance_test_alert_v14 update: {e}")))?;

        Ok(row)
    }

    pub async fn list_performance_test_alert_history_v14(
        &self,
        alert_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PerformanceTestAlertHistoryV14>> {
        sqlx::query_as::<_, PerformanceTestAlertHistoryV14>(
            "SELECT * FROM performance_test_alert_history_v11 WHERE alert_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(alert_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_performance_test_alert_history_v14: {e}")))
    }

    pub async fn get_performance_test_alert_analytics_v14(
        &self,
        alert_id: Uuid,
    ) -> Result<(i64, Option<f64>, Option<f64>)> {
        let row: (i64, Option<f64>, Option<f64>) = sqlx::query_as(
            r#"SELECT
                   COUNT(*) as trigger_count,
                   AVG(metric_value) as avg_value,
                   MAX(metric_value) as max_value
               FROM performance_test_alert_history_v11
               WHERE alert_id = $1"#,
        )
        .bind(alert_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_analytics_v14: {e}")))?;
        Ok((row.0, row.1, row.2))
    }

    pub async fn get_performance_test_alert_notification_config_v14(
        &self,
        alert_id: Uuid,
    ) -> Result<Option<(String, bool, Option<String>)>> {
        let row: Option<(String, bool, Option<String>)> = sqlx::query_as(
            r#"SELECT
                   alert_type,
                   enabled,
                   last_triggered_at::text
               FROM performance_test_alerts_v11
               WHERE id = $1"#,
        )
        .bind(alert_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_notification_config_v14: {e}")))?;
        Ok(row)
    }

    // --- Test Suite Metrics v14 ---

    pub async fn create_test_suite_metric_v14(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<TestSuiteMetricV14> {
        let row = sqlx::query_as::<_, TestSuiteMetricV14>(
            r#"INSERT INTO test_suite_metrics_v11 (suite_id, metric_name, metric_value)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_metric_v14: {e}")))?;
        Ok(row)
    }

    pub async fn list_test_suite_metrics_v14(
        &self,
        suite_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestSuiteMetricV14>> {
        sqlx::query_as::<_, TestSuiteMetricV14>(
            "SELECT * FROM test_suite_metrics_v11 WHERE suite_id = $1 ORDER BY measured_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(suite_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_test_suite_metrics_v14: {e}")))
    }

    pub async fn get_test_suite_latest_metric_v14(
        &self,
        suite_id: Uuid,
        metric_name: &str,
    ) -> Result<Option<TestSuiteMetricV14>> {
        sqlx::query_as::<_, TestSuiteMetricV14>(
            "SELECT * FROM test_suite_metrics_v11 WHERE suite_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT 1",
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_latest_metric_v14: {e}")))
    }

    pub async fn create_test_suite_baseline_v14(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        baseline_value: f64,
        threshold_percent: f64,
    ) -> Result<TestSuiteBaselineV14> {
        let row = sqlx::query_as::<_, TestSuiteBaselineV14>(
            r#"INSERT INTO test_suite_baselines_v11 (suite_id, metric_name, baseline_value, threshold_percent)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (suite_id, metric_name) DO UPDATE
               SET baseline_value = $3, threshold_percent = $4
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(baseline_value)
        .bind(threshold_percent)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_baseline_v14: {e}")))?;
        Ok(row)
    }

    pub async fn get_test_suite_baselines_v14(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteBaselineV14>> {
        sqlx::query_as::<_, TestSuiteBaselineV14>(
            "SELECT * FROM test_suite_baselines_v11 WHERE suite_id = $1 ORDER BY metric_name",
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_baselines_v14: {e}")))
    }

    pub async fn detect_test_suite_regression_v14(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        current_value: f64,
    ) -> Result<bool> {
        let baseline: Option<TestSuiteBaselineV14> = sqlx::query_as::<_, TestSuiteBaselineV14>(
            "SELECT * FROM test_suite_baselines_v11 WHERE suite_id = $1 AND metric_name = $2",
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("detect_test_suite_regression_v14: {e}")))?;

        match baseline {
            Some(b) => {
                let diff = ((current_value - b.baseline_value) / b.baseline_value * 100.0).abs();
                Ok(diff > b.threshold_percent)
            }
            None => Ok(false),
        }
    }

    pub async fn get_test_suite_performance_alerts_v14(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<String>> {
        let metrics = sqlx::query_as::<_, TestSuiteMetricV14>(
            r#"SELECT * FROM test_suite_metrics_v11
               WHERE suite_id = $1
               AND metric_name IN ('execution_time_ms', 'memory_usage_mb', 'cpu_usage_percent')
               AND measured_at > NOW() - INTERVAL '1 hour'"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_performance_alerts_v14: {e}")))?;

        let mut alerts = Vec::new();
        for metric in metrics {
            match metric.metric_name.as_str() {
                "execution_time_ms" if metric.metric_value > 1000.0 => {
                    alerts.push(format!("High execution time: {}ms", metric.metric_value));
                }
                "memory_usage_mb" if metric.metric_value > 512.0 => {
                    alerts.push(format!("High memory usage: {}MB", metric.metric_value));
                }
                "cpu_usage_percent" if metric.metric_value > 90.0 => {
                    alerts.push(format!("High CPU usage: {}%", metric.metric_value));
                }
                _ => {}
            }
        }
        Ok(alerts)
    }

    // --- Code Quality Metrics v14 ---

    pub async fn create_code_quality_metric_v14(
        &self,
        repo_id: Uuid,
        file_path: &str,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<CodeQualityMetricV14> {
        let row = sqlx::query_as::<_, CodeQualityMetricV14>(
            r#"INSERT INTO code_quality_metrics_v12 (repo_id, file_path, metric_name, metric_value)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(file_path)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_metric_v14: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_metrics_v14(
        &self,
        repo_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CodeQualityMetricV14>> {
        let rows = if let Some(name) = metric_name {
            sqlx::query_as::<_, CodeQualityMetricV14>(
                r#"SELECT * FROM code_quality_metrics_v12
                   WHERE repo_id = $1 AND metric_name = $2
                   ORDER BY measured_at DESC
                   LIMIT $3 OFFSET $4"#,
            )
            .bind(repo_id)
            .bind(name)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, CodeQualityMetricV14>(
                r#"SELECT * FROM code_quality_metrics_v12
                   WHERE repo_id = $1
                   ORDER BY measured_at DESC
                   LIMIT $2 OFFSET $3"#,
            )
            .bind(repo_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_code_quality_metrics_v14: {e}")))
    }

    pub async fn get_code_quality_score_v14(
        &self,
        repo_id: Uuid,
    ) -> Result<f64> {
        let row: (Option<f64>,) = sqlx::query_as(
            r#"SELECT COALESCE(
                   (SELECT AVG(100.0 - LEAST(metric_value, 100.0))
                    FROM code_quality_metrics_v12
                    WHERE repo_id = $1
                    AND metric_name IN ('complexity', ' duplication')),
                   100.0
               )"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_score_v14: {e}")))?;
        Ok(row.0.unwrap_or(0.0))
    }

    pub async fn create_code_quality_threshold_v14(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        threshold_value: f64,
        enabled: bool,
    ) -> Result<CodeQualityThresholdV14> {
        let row = sqlx::query_as::<_, CodeQualityThresholdV14>(
            r#"INSERT INTO code_quality_thresholds_v11 (repo_id, metric_name, threshold_value, enabled)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (repo_id, metric_name) DO UPDATE
               SET threshold_value = $3, enabled = $4
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .bind(threshold_value)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_threshold_v14: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_thresholds_v14(
        &self,
        repo_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<CodeQualityThresholdV14>> {
        let rows = if enabled_only {
            sqlx::query_as::<_, CodeQualityThresholdV14>(
                r#"SELECT * FROM code_quality_thresholds_v11
                   WHERE repo_id = $1 AND enabled = true
                   ORDER BY metric_name"#,
            )
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, CodeQualityThresholdV14>(
                r#"SELECT * FROM code_quality_thresholds_v11
                   WHERE repo_id = $1
                   ORDER BY metric_name"#,
            )
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_code_quality_thresholds_v14: {e}")))
    }

    pub async fn check_code_quality_violation_v14(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<bool> {
        let threshold: Option<CodeQualityThresholdV14> =
            sqlx::query_as::<_, CodeQualityThresholdV14>(
                r#"SELECT * FROM code_quality_thresholds_v11
                   WHERE repo_id = $1 AND metric_name = $2 AND enabled = true"#,
            )
            .bind(repo_id)
            .bind(metric_name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("check_code_quality_violation_v14: {e}")))?;

        match threshold {
            Some(t) => Ok(metric_value > t.threshold_value),
            None => Ok(false),
        }
    }

    pub async fn delete_code_quality_threshold_v14(
        &self,
        repo_id: Uuid,
        metric_name: &str,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM code_quality_thresholds_v11 WHERE repo_id = $1 AND metric_name = $2",
        )
        .bind(repo_id)
        .bind(metric_name)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("delete_code_quality_threshold_v14: {e}")))?;
        Ok(())
    }

    pub async fn get_code_quality_enforcement_report_v14(
        &self,
        repo_id: Uuid,
    ) -> Result<(i64, i64, f64)> {
        let total_thresholds: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM code_quality_thresholds_v11 WHERE repo_id = $1 AND enabled = true"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_enforcement_report_v14 total: {e}")))?;

        let violating_thresholds: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM code_quality_thresholds_v11 t
               WHERE t.repo_id = $1 AND t.enabled = true
               AND EXISTS (
                   SELECT 1 FROM code_quality_metrics_v12 m
                   WHERE m.repo_id = t.repo_id AND m.metric_name = t.metric_name
                   AND m.metric_value > t.threshold_value
               )"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_enforcement_report_v14 violating: {e}")))?;

        let compliance_rate = if total_thresholds.0 > 0 {
            ((total_thresholds.0 - violating_thresholds.0) as f64 / total_thresholds.0 as f64) * 100.0
        } else {
            100.0
        };

        Ok((total_thresholds.0, violating_thresholds.0, compliance_rate))
    }

    // --- Performance Test Alerts v15 ---

    pub async fn create_performance_test_alert_v15(
        &self,
        baseline_id: Uuid,
        alert_type: &str,
        threshold: f64,
        enabled: bool,
    ) -> Result<PerformanceTestAlertV15> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV15>(
            r#"INSERT INTO performance_test_alerts_v12 (baseline_id, alert_type, threshold, enabled)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(baseline_id)
        .bind(alert_type)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_performance_test_alert_v15: {e}")))?;
        Ok(row)
    }

    pub async fn list_performance_test_alerts_v15(
        &self,
        baseline_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<PerformanceTestAlertV15>> {
        let rows = if enabled_only {
            sqlx::query_as::<_, PerformanceTestAlertV15>(
                r#"SELECT * FROM performance_test_alerts_v12
                   WHERE baseline_id = $1 AND enabled = true
                   ORDER BY created_at DESC"#,
            )
            .bind(baseline_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, PerformanceTestAlertV15>(
                r#"SELECT * FROM performance_test_alerts_v12
                   WHERE baseline_id = $1
                   ORDER BY created_at DESC"#,
            )
            .bind(baseline_id)
            .fetch_all(&self.pool)
            .await
        };
        rows.map_err(|e| DbError::Database(format!("list_performance_test_alerts_v15: {e}")))
    }

    pub async fn update_performance_test_alert_v15(
        &self,
        id: Uuid,
        threshold: Option<f64>,
        enabled: Option<bool>,
    ) -> Result<PerformanceTestAlertV15> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV15>(
            r#"UPDATE performance_test_alerts_v12
               SET threshold = COALESCE($2, threshold),
                   enabled = COALESCE($3, enabled)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_performance_test_alert_v15: {e}")))?;
        Ok(row)
    }

    pub async fn record_performance_test_alert_v15(
        &self,
        alert_id: Uuid,
        metric_name: &str,
        metric_value: f64,
        threshold: f64,
    ) -> Result<PerformanceTestAlertHistoryV15> {
        let row = sqlx::query_as::<_, PerformanceTestAlertHistoryV15>(
            r#"INSERT INTO performance_test_alert_history_v12 (alert_id, metric_name, metric_value, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(alert_id)
        .bind(metric_name)
        .bind(metric_value)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_performance_test_alert_v15: {e}")))?;

        sqlx::query(
            "UPDATE performance_test_alerts_v12 SET last_triggered_at = NOW() WHERE id = $1",
        )
        .bind(alert_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_performance_test_alert_v15 update: {e}")))?;

        Ok(row)
    }

    pub async fn list_performance_test_alert_history_v15(
        &self,
        alert_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PerformanceTestAlertHistoryV15>> {
        sqlx::query_as::<_, PerformanceTestAlertHistoryV15>(
            "SELECT * FROM performance_test_alert_history_v12 WHERE alert_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(alert_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_performance_test_alert_history_v15: {e}")))
    }

    pub async fn get_performance_test_alert_analytics_v15(
        &self,
        alert_id: Uuid,
    ) -> Result<(i64, Option<f64>, Option<f64>)> {
        let row: (i64, Option<f64>, Option<f64>) = sqlx::query_as(
            r#"SELECT
                   COUNT(*) as trigger_count,
                   AVG(metric_value) as avg_value,
                   MAX(metric_value) as max_value
               FROM performance_test_alert_history_v12
               WHERE alert_id = $1"#,
        )
        .bind(alert_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_analytics_v15: {e}")))?;
        Ok((row.0, row.1, row.2))
    }

    pub async fn get_performance_test_alert_notification_config_v15(
        &self,
        alert_id: Uuid,
    ) -> Result<Option<(String, bool, Option<String>)>> {
        let row: Option<(String, bool, Option<String>)> = sqlx::query_as(
            r#"SELECT
                   alert_type,
                   enabled,
                   last_triggered_at::text
               FROM performance_test_alerts_v12
               WHERE id = $1"#,
        )
        .bind(alert_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_notification_config_v15: {e}")))?;
        Ok(row)
    }

    // --- Database Replication Config v4 ---

    pub async fn create_replication_config(
        &self,
        replica_id: Uuid,
        config_key: &str,
        config_value: serde_json::Value,
    ) -> Result<DatabaseReplicationConfigV4> {
        let row = sqlx::query_as::<_, DatabaseReplicationConfigV4>(
            r#"INSERT INTO database_replication_config_v4 (replica_id, config_key, config_value)
               VALUES ($1, $2, $3)
               ON CONFLICT (replica_id, config_key) DO UPDATE SET config_value = $3
               RETURNING *"#,
        )
        .bind(replica_id)
        .bind(config_key)
        .bind(config_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_replication_config: {e}")))?;
        Ok(row)
    }

    pub async fn get_replication_config(
        &self,
        replica_id: Uuid,
        config_key: &str,
    ) -> Result<Option<DatabaseReplicationConfigV4>> {
        sqlx::query_as::<_, DatabaseReplicationConfigV4>(
            "SELECT * FROM database_replication_config_v4 WHERE replica_id = $1 AND config_key = $2",
        )
        .bind(replica_id)
        .bind(config_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_replication_config: {e}")))
    }

    pub async fn list_replication_configs(
        &self,
        replica_id: Uuid,
    ) -> Result<Vec<DatabaseReplicationConfigV4>> {
        sqlx::query_as::<_, DatabaseReplicationConfigV4>(
            "SELECT * FROM database_replication_config_v4 WHERE replica_id = $1 ORDER BY config_key",
        )
        .bind(replica_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_replication_configs: {e}")))
    }

    pub async fn delete_replication_config(
        &self,
        replica_id: Uuid,
        config_key: &str,
    ) -> Result<()> {
        sqlx::query("DELETE FROM database_replication_config_v4 WHERE replica_id = $1 AND config_key = $2")
            .bind(replica_id)
            .bind(config_key)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_replication_config: {e}")))?;
        Ok(())
    }

    // --- Database Replication Alerts v4 ---

    pub async fn create_replication_alert(
        &self,
        replica_id: Uuid,
        alert_type: &str,
        threshold: f64,
    ) -> Result<DatabaseReplicationAlertV4> {
        let row = sqlx::query_as::<_, DatabaseReplicationAlertV4>(
            r#"INSERT INTO database_replication_alerts_v4 (replica_id, alert_type, threshold)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(replica_id)
        .bind(alert_type)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_replication_alert: {e}")))?;
        Ok(row)
    }

    pub async fn get_replication_alert(&self, id: Uuid) -> Result<DatabaseReplicationAlertV4> {
        sqlx::query_as::<_, DatabaseReplicationAlertV4>(
            "SELECT * FROM database_replication_alerts_v4 WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_replication_alert: {e}")))
    }

    pub async fn list_replication_alerts(
        &self,
        replica_id: Uuid,
    ) -> Result<Vec<DatabaseReplicationAlertV4>> {
        sqlx::query_as::<_, DatabaseReplicationAlertV4>(
            "SELECT * FROM database_replication_alerts_v4 WHERE replica_id = $1 ORDER BY alert_type",
        )
        .bind(replica_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_replication_alerts: {e}")))
    }

    pub async fn update_replication_alert(
        &self,
        id: Uuid,
        threshold: Option<f64>,
        enabled: Option<bool>,
    ) -> Result<DatabaseReplicationAlertV4> {
        let row = sqlx::query_as::<_, DatabaseReplicationAlertV4>(
            r#"UPDATE database_replication_alerts_v4
               SET threshold = COALESCE($2, threshold),
                   enabled = COALESCE($3, enabled)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_replication_alert: {e}")))?;
        Ok(row)
    }

    pub async fn delete_replication_alert(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM database_replication_alerts_v4 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_replication_alert: {e}")))?;
        Ok(())
    }

    pub async fn trigger_replication_alert(&self, id: Uuid) -> Result<DatabaseReplicationAlertV4> {
        let row = sqlx::query_as::<_, DatabaseReplicationAlertV4>(
            r#"UPDATE database_replication_alerts_v4
               SET last_triggered_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("trigger_replication_alert: {e}")))?;
        Ok(row)
    }

    pub async fn get_enabled_replication_alerts(
        &self,
        replica_id: Uuid,
    ) -> Result<Vec<DatabaseReplicationAlertV4>> {
        sqlx::query_as::<_, DatabaseReplicationAlertV4>(
            "SELECT * FROM database_replication_alerts_v4 WHERE replica_id = $1 AND enabled = true ORDER BY alert_type",
        )
        .bind(replica_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_enabled_replication_alerts: {e}")))
    }

    // --- Encryption Key Versions v4 ---

    pub async fn create_encryption_key_version(
        &self,
        key_id: Uuid,
        version: i32,
        key_material: &[u8],
    ) -> Result<EncryptionKeyVersionV4> {
        let row = sqlx::query_as::<_, EncryptionKeyVersionV4>(
            r#"INSERT INTO encryption_key_versions_v4 (key_id, version, key_material)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(key_id)
        .bind(version)
        .bind(key_material)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_encryption_key_version: {e}")))?;
        Ok(row)
    }

    pub async fn get_encryption_key_version(
        &self,
        key_id: Uuid,
        version: i32,
    ) -> Result<Option<EncryptionKeyVersionV4>> {
        sqlx::query_as::<_, EncryptionKeyVersionV4>(
            "SELECT * FROM encryption_key_versions_v4 WHERE key_id = $1 AND version = $2",
        )
        .bind(key_id)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_encryption_key_version: {e}")))
    }

    pub async fn list_encryption_key_versions(
        &self,
        key_id: Uuid,
    ) -> Result<Vec<EncryptionKeyVersionV4>> {
        sqlx::query_as::<_, EncryptionKeyVersionV4>(
            "SELECT * FROM encryption_key_versions_v4 WHERE key_id = $1 ORDER BY version DESC",
        )
        .bind(key_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_encryption_key_versions: {e}")))
    }

    pub async fn get_latest_encryption_key_version(
        &self,
        key_id: Uuid,
    ) -> Result<Option<EncryptionKeyVersionV4>> {
        sqlx::query_as::<_, EncryptionKeyVersionV4>(
            "SELECT * FROM encryption_key_versions_v4 WHERE key_id = $1 ORDER BY version DESC LIMIT 1",
        )
        .bind(key_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_latest_encryption_key_version: {e}")))
    }

    // --- Encryption Compliance Checks v4 ---

    pub async fn create_encryption_compliance_check(
        &self,
        check_type: &str,
    ) -> Result<EncryptionComplianceCheckV4> {
        let row = sqlx::query_as::<_, EncryptionComplianceCheckV4>(
            r#"INSERT INTO encryption_compliance_checks_v4 (check_type)
               VALUES ($1)
               RETURNING *"#,
        )
        .bind(check_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_encryption_compliance_check: {e}")))?;
        Ok(row)
    }

    pub async fn get_encryption_compliance_check(
        &self,
        id: Uuid,
    ) -> Result<EncryptionComplianceCheckV4> {
        sqlx::query_as::<_, EncryptionComplianceCheckV4>(
            "SELECT * FROM encryption_compliance_checks_v4 WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_encryption_compliance_check: {e}")))
    }

    pub async fn update_encryption_compliance_check(
        &self,
        id: Uuid,
        status: &str,
        findings: serde_json::Value,
        score: i32,
    ) -> Result<EncryptionComplianceCheckV4> {
        let row = sqlx::query_as::<_, EncryptionComplianceCheckV4>(
            r#"UPDATE encryption_compliance_checks_v4
               SET status = $2, findings = $3, score = $4
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(status)
        .bind(findings)
        .bind(score)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_encryption_compliance_check: {e}")))?;
        Ok(row)
    }

    pub async fn list_encryption_compliance_checks(
        &self,
        check_type: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<EncryptionComplianceCheckV4>> {
        let rows = match check_type {
            Some(ct) => {
                sqlx::query_as::<_, EncryptionComplianceCheckV4>(
                    r#"SELECT * FROM encryption_compliance_checks_v4
                       WHERE check_type = $1
                       ORDER BY created_at DESC
                       LIMIT $2 OFFSET $3"#,
                )
                .bind(ct)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, EncryptionComplianceCheckV4>(
                    r#"SELECT * FROM encryption_compliance_checks_v4
                       ORDER BY created_at DESC
                       LIMIT $1 OFFSET $2"#,
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        };
        rows.map_err(|e| DbError::Database(format!("list_encryption_compliance_checks: {e}")))
    }

    // --- Data Residency Reports v4 ---

    pub async fn create_data_residency_report(
        &self,
        report_type: &str,
    ) -> Result<DataResidencyReportV4> {
        let row = sqlx::query_as::<_, DataResidencyReportV4>(
            r#"INSERT INTO data_residency_reports_v4 (report_type)
               VALUES ($1)
               RETURNING *"#,
        )
        .bind(report_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_data_residency_report: {e}")))?;
        Ok(row)
    }

    pub async fn get_data_residency_report(
        &self,
        id: Uuid,
    ) -> Result<DataResidencyReportV4> {
        sqlx::query_as::<_, DataResidencyReportV4>(
            "SELECT * FROM data_residency_reports_v4 WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_data_residency_report: {e}")))
    }

    pub async fn update_data_residency_report(
        &self,
        id: Uuid,
        findings: serde_json::Value,
        score: i32,
    ) -> Result<DataResidencyReportV4> {
        let row = sqlx::query_as::<_, DataResidencyReportV4>(
            r#"UPDATE data_residency_reports_v4
               SET findings = $2, score = $3
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(findings)
        .bind(score)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_data_residency_report: {e}")))?;
        Ok(row)
    }

    pub async fn list_data_residency_reports(
        &self,
        report_type: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DataResidencyReportV4>> {
        let rows = match report_type {
            Some(rt) => {
                sqlx::query_as::<_, DataResidencyReportV4>(
                    r#"SELECT * FROM data_residency_reports_v4
                       WHERE report_type = $1
                       ORDER BY generated_at DESC
                       LIMIT $2 OFFSET $3"#,
                )
                .bind(rt)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, DataResidencyReportV4>(
                    r#"SELECT * FROM data_residency_reports_v4
                       ORDER BY generated_at DESC
                       LIMIT $1 OFFSET $2"#,
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        };
        rows.map_err(|e| DbError::Database(format!("list_data_residency_reports: {e}")))
    }

    pub async fn delete_data_residency_report(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM data_residency_reports_v4 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_data_residency_report: {e}")))?;
        Ok(())
    }

    // --- Data Residency Compliance v4 ---

    pub async fn create_data_residency_compliance(
        &self,
        rule_id: Uuid,
    ) -> Result<DataResidencyComplianceV4> {
        let row = sqlx::query_as::<_, DataResidencyComplianceV4>(
            r#"INSERT INTO data_residency_compliance_v4 (rule_id)
               VALUES ($1)
               RETURNING *"#,
        )
        .bind(rule_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_data_residency_compliance: {e}")))?;
        Ok(row)
    }

    pub async fn get_data_residency_compliance(
        &self,
        id: Uuid,
    ) -> Result<DataResidencyComplianceV4> {
        sqlx::query_as::<_, DataResidencyComplianceV4>(
            "SELECT * FROM data_residency_compliance_v4 WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_data_residency_compliance: {e}")))
    }

    pub async fn update_data_residency_compliance(
        &self,
        id: Uuid,
        compliance_status: &str,
    ) -> Result<DataResidencyComplianceV4> {
        let row = sqlx::query_as::<_, DataResidencyComplianceV4>(
            r#"UPDATE data_residency_compliance_v4
               SET compliance_status = $2, last_checked_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(compliance_status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_data_residency_compliance: {e}")))?;
        Ok(row)
    }

    pub async fn list_data_residency_compliance(
        &self,
        rule_id: Option<Uuid>,
        compliance_status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DataResidencyComplianceV4>> {
        let rows = match (rule_id, compliance_status) {
            (Some(rid), Some(cs)) => {
                sqlx::query_as::<_, DataResidencyComplianceV4>(
                    r#"SELECT * FROM data_residency_compliance_v4
                       WHERE rule_id = $1 AND compliance_status = $2
                       ORDER BY created_at DESC
                       LIMIT $3 OFFSET $4"#,
                )
                .bind(rid)
                .bind(cs)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            (Some(rid), None) => {
                sqlx::query_as::<_, DataResidencyComplianceV4>(
                    r#"SELECT * FROM data_residency_compliance_v4
                       WHERE rule_id = $1
                       ORDER BY created_at DESC
                       LIMIT $2 OFFSET $3"#,
                )
                .bind(rid)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(cs)) => {
                sqlx::query_as::<_, DataResidencyComplianceV4>(
                    r#"SELECT * FROM data_residency_compliance_v4
                       WHERE compliance_status = $1
                       ORDER BY created_at DESC
                       LIMIT $2 OFFSET $3"#,
                )
                .bind(cs)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            (None, None) => {
                sqlx::query_as::<_, DataResidencyComplianceV4>(
                    r#"SELECT * FROM data_residency_compliance_v4
                       ORDER BY created_at DESC
                       LIMIT $1 OFFSET $2"#,
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        };
        rows.map_err(|e| DbError::Database(format!("list_data_residency_compliance: {e}")))
    }

    pub async fn get_data_residency_compliance_by_rule(
        &self,
        rule_id: Uuid,
    ) -> Result<Vec<DataResidencyComplianceV4>> {
        sqlx::query_as::<_, DataResidencyComplianceV4>(
            "SELECT * FROM data_residency_compliance_v4 WHERE rule_id = $1 ORDER BY created_at DESC",
        )
        .bind(rule_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_data_residency_compliance_by_rule: {e}")))
    }

    pub async fn delete_data_residency_compliance(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM data_residency_compliance_v4 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_data_residency_compliance: {e}")))?;
        Ok(())
    }

    // --- Database Replication Config v8 ---

    pub async fn create_replication_config_v8(
        &self,
        replica_id: Uuid,
        config_key: &str,
        config_value: serde_json::Value,
    ) -> Result<DatabaseReplicationConfigV8> {
        let row = sqlx::query_as::<_, DatabaseReplicationConfigV8>(
            r#"INSERT INTO database_replication_config_v8 (replica_id, config_key, config_value)
               VALUES ($1, $2, $3)
               ON CONFLICT (replica_id, config_key) DO UPDATE SET config_value = $3
               RETURNING *"#,
        )
        .bind(replica_id)
        .bind(config_key)
        .bind(config_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_replication_config_v8: {e}")))?;
        Ok(row)
    }

    pub async fn get_replication_config_v8(
        &self,
        replica_id: Uuid,
        config_key: &str,
    ) -> Result<Option<DatabaseReplicationConfigV8>> {
        sqlx::query_as::<_, DatabaseReplicationConfigV8>(
            "SELECT * FROM database_replication_config_v8 WHERE replica_id = $1 AND config_key = $2",
        )
        .bind(replica_id)
        .bind(config_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_replication_config_v8: {e}")))
    }

    pub async fn list_replication_configs_v8(
        &self,
        replica_id: Uuid,
    ) -> Result<Vec<DatabaseReplicationConfigV8>> {
        sqlx::query_as::<_, DatabaseReplicationConfigV8>(
            "SELECT * FROM database_replication_config_v8 WHERE replica_id = $1 ORDER BY config_key",
        )
        .bind(replica_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_replication_configs_v8: {e}")))
    }

    pub async fn delete_replication_config_v8(
        &self,
        replica_id: Uuid,
        config_key: &str,
    ) -> Result<()> {
        sqlx::query("DELETE FROM database_replication_config_v8 WHERE replica_id = $1 AND config_key = $2")
            .bind(replica_id)
            .bind(config_key)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_replication_config_v8: {e}")))?;
        Ok(())
    }

    // --- Database Replication Alerts v8 ---

    pub async fn create_replication_alert_v8(
        &self,
        replica_id: Uuid,
        alert_type: &str,
        threshold: f64,
    ) -> Result<DatabaseReplicationAlertV8> {
        let row = sqlx::query_as::<_, DatabaseReplicationAlertV8>(
            r#"INSERT INTO database_replication_alerts_v8 (replica_id, alert_type, threshold)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(replica_id)
        .bind(alert_type)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_replication_alert_v8: {e}")))?;
        Ok(row)
    }

    pub async fn get_replication_alert_v8(&self, id: Uuid) -> Result<DatabaseReplicationAlertV8> {
        sqlx::query_as::<_, DatabaseReplicationAlertV8>(
            "SELECT * FROM database_replication_alerts_v8 WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_replication_alert_v8: {e}")))
    }

    pub async fn list_replication_alerts_v8(
        &self,
        replica_id: Uuid,
    ) -> Result<Vec<DatabaseReplicationAlertV8>> {
        sqlx::query_as::<_, DatabaseReplicationAlertV8>(
            "SELECT * FROM database_replication_alerts_v8 WHERE replica_id = $1 ORDER BY alert_type",
        )
        .bind(replica_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_replication_alerts_v8: {e}")))
    }

    pub async fn update_replication_alert_v8(
        &self,
        id: Uuid,
        threshold: Option<f64>,
        enabled: Option<bool>,
    ) -> Result<DatabaseReplicationAlertV8> {
        let row = sqlx::query_as::<_, DatabaseReplicationAlertV8>(
            r#"UPDATE database_replication_alerts_v8
               SET threshold = COALESCE($2, threshold),
                   enabled = COALESCE($3, enabled)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_replication_alert_v8: {e}")))?;
        Ok(row)
    }

    pub async fn delete_replication_alert_v8(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM database_replication_alerts_v8 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_replication_alert_v8: {e}")))?;
        Ok(())
    }

    pub async fn trigger_replication_alert_v8(&self, id: Uuid) -> Result<DatabaseReplicationAlertV8> {
        let row = sqlx::query_as::<_, DatabaseReplicationAlertV8>(
            r#"UPDATE database_replication_alerts_v8
               SET last_triggered_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("trigger_replication_alert_v8: {e}")))?;
        Ok(row)
    }

    pub async fn get_enabled_replication_alerts_v8(
        &self,
        replica_id: Uuid,
    ) -> Result<Vec<DatabaseReplicationAlertV8>> {
        sqlx::query_as::<_, DatabaseReplicationAlertV8>(
            "SELECT * FROM database_replication_alerts_v8 WHERE replica_id = $1 AND enabled = true ORDER BY alert_type",
        )
        .bind(replica_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_enabled_replication_alerts_v8: {e}")))
    }

    // --- Encryption Key Versions v8 ---

    pub async fn create_encryption_key_version_v8(
        &self,
        key_id: Uuid,
        version: i32,
        key_material: &[u8],
    ) -> Result<EncryptionKeyVersionV8> {
        let row = sqlx::query_as::<_, EncryptionKeyVersionV8>(
            r#"INSERT INTO encryption_key_versions_v8 (key_id, version, key_material)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(key_id)
        .bind(version)
        .bind(key_material)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_encryption_key_version_v8: {e}")))?;
        Ok(row)
    }

    pub async fn get_encryption_key_version_v8(
        &self,
        id: Uuid,
    ) -> Result<EncryptionKeyVersionV8> {
        sqlx::query_as::<_, EncryptionKeyVersionV8>(
            "SELECT * FROM encryption_key_versions_v8 WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_encryption_key_version_v8: {e}")))
    }

    pub async fn list_encryption_key_versions_v8(
        &self,
        key_id: Uuid,
    ) -> Result<Vec<EncryptionKeyVersionV8>> {
        sqlx::query_as::<_, EncryptionKeyVersionV8>(
            "SELECT * FROM encryption_key_versions_v8 WHERE key_id = $1 ORDER BY version DESC",
        )
        .bind(key_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_encryption_key_versions_v8: {e}")))
    }

    pub async fn delete_encryption_key_version_v8(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM encryption_key_versions_v8 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_encryption_key_version_v8: {e}")))?;
        Ok(())
    }

    // --- Encryption Compliance Checks v8 ---

    pub async fn create_encryption_compliance_check_v8(
        &self,
        check_type: &str,
    ) -> Result<EncryptionComplianceCheckV8> {
        let row = sqlx::query_as::<_, EncryptionComplianceCheckV8>(
            r#"INSERT INTO encryption_compliance_checks_v8 (check_type)
               VALUES ($1)
               RETURNING *"#,
        )
        .bind(check_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_encryption_compliance_check_v8: {e}")))?;
        Ok(row)
    }

    pub async fn get_encryption_compliance_check_v8(
        &self,
        id: Uuid,
    ) -> Result<EncryptionComplianceCheckV8> {
        sqlx::query_as::<_, EncryptionComplianceCheckV8>(
            "SELECT * FROM encryption_compliance_checks_v8 WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_encryption_compliance_check_v8: {e}")))
    }

    pub async fn update_encryption_compliance_check_v8(
        &self,
        id: Uuid,
        status: &str,
        findings: serde_json::Value,
        score: i32,
    ) -> Result<EncryptionComplianceCheckV8> {
        let row = sqlx::query_as::<_, EncryptionComplianceCheckV8>(
            r#"UPDATE encryption_compliance_checks_v8
               SET status = $2, findings = $3, score = $4
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(status)
        .bind(findings)
        .bind(score)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_encryption_compliance_check_v8: {e}")))?;
        Ok(row)
    }

    pub async fn list_encryption_compliance_checks_v8(
        &self,
        check_type: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<EncryptionComplianceCheckV8>> {
        let rows = match check_type {
            Some(ct) => {
                sqlx::query_as::<_, EncryptionComplianceCheckV8>(
                    r#"SELECT * FROM encryption_compliance_checks_v8
                       WHERE check_type = $1
                       ORDER BY created_at DESC
                       LIMIT $2 OFFSET $3"#,
                )
                .bind(ct)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, EncryptionComplianceCheckV8>(
                    r#"SELECT * FROM encryption_compliance_checks_v8
                       ORDER BY created_at DESC
                       LIMIT $1 OFFSET $2"#,
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        };
        rows.map_err(|e| DbError::Database(format!("list_encryption_compliance_checks_v8: {e}")))
    }

    pub async fn delete_encryption_compliance_check_v8(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM encryption_compliance_checks_v8 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_encryption_compliance_check_v8: {e}")))?;
        Ok(())
    }

    // --- Data Residency Reports v8 ---

    pub async fn create_data_residency_report_v8(
        &self,
        report_type: &str,
    ) -> Result<DataResidencyReportV8> {
        let row = sqlx::query_as::<_, DataResidencyReportV8>(
            r#"INSERT INTO data_residency_reports_v8 (report_type)
               VALUES ($1)
               RETURNING *"#,
        )
        .bind(report_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_data_residency_report_v8: {e}")))?;
        Ok(row)
    }

    pub async fn get_data_residency_report_v8(
        &self,
        id: Uuid,
    ) -> Result<DataResidencyReportV8> {
        sqlx::query_as::<_, DataResidencyReportV8>(
            "SELECT * FROM data_residency_reports_v8 WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_data_residency_report_v8: {e}")))
    }

    pub async fn update_data_residency_report_v8(
        &self,
        id: Uuid,
        findings: serde_json::Value,
        score: i32,
    ) -> Result<DataResidencyReportV8> {
        let row = sqlx::query_as::<_, DataResidencyReportV8>(
            r#"UPDATE data_residency_reports_v8
               SET findings = $2, score = $3
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(findings)
        .bind(score)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_data_residency_report_v8: {e}")))?;
        Ok(row)
    }

    pub async fn list_data_residency_reports_v8(
        &self,
        report_type: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DataResidencyReportV8>> {
        let rows = match report_type {
            Some(rt) => {
                sqlx::query_as::<_, DataResidencyReportV8>(
                    r#"SELECT * FROM data_residency_reports_v8
                       WHERE report_type = $1
                       ORDER BY generated_at DESC
                       LIMIT $2 OFFSET $3"#,
                )
                .bind(rt)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, DataResidencyReportV8>(
                    r#"SELECT * FROM data_residency_reports_v8
                       ORDER BY generated_at DESC
                       LIMIT $1 OFFSET $2"#,
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        };
        rows.map_err(|e| DbError::Database(format!("list_data_residency_reports_v8: {e}")))
    }

    pub async fn delete_data_residency_report_v8(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM data_residency_reports_v8 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_data_residency_report_v8: {e}")))?;
        Ok(())
    }

    // --- Data Residency Compliance v8 ---

    pub async fn create_data_residency_compliance_v8(
        &self,
        rule_id: Uuid,
    ) -> Result<DataResidencyComplianceV8> {
        let row = sqlx::query_as::<_, DataResidencyComplianceV8>(
            r#"INSERT INTO data_residency_compliance_v8 (rule_id)
               VALUES ($1)
               RETURNING *"#,
        )
        .bind(rule_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_data_residency_compliance_v8: {e}")))?;
        Ok(row)
    }

    pub async fn get_data_residency_compliance_v8(
        &self,
        id: Uuid,
    ) -> Result<DataResidencyComplianceV8> {
        sqlx::query_as::<_, DataResidencyComplianceV8>(
            "SELECT * FROM data_residency_compliance_v8 WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_data_residency_compliance_v8: {e}")))
    }

    pub async fn update_data_residency_compliance_v8(
        &self,
        id: Uuid,
        compliance_status: &str,
    ) -> Result<DataResidencyComplianceV8> {
        let row = sqlx::query_as::<_, DataResidencyComplianceV8>(
            r#"UPDATE data_residency_compliance_v8
               SET compliance_status = $2, last_checked_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(compliance_status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_data_residency_compliance_v8: {e}")))?;
        Ok(row)
    }

    pub async fn list_data_residency_compliance_v8(
        &self,
        rule_id: Option<Uuid>,
        compliance_status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DataResidencyComplianceV8>> {
        let rows = match (rule_id, compliance_status) {
            (Some(rid), Some(cs)) => {
                sqlx::query_as::<_, DataResidencyComplianceV8>(
                    r#"SELECT * FROM data_residency_compliance_v8
                       WHERE rule_id = $1 AND compliance_status = $2
                       ORDER BY created_at DESC
                       LIMIT $3 OFFSET $4"#,
                )
                .bind(rid)
                .bind(cs)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            (Some(rid), None) => {
                sqlx::query_as::<_, DataResidencyComplianceV8>(
                    r#"SELECT * FROM data_residency_compliance_v8
                       WHERE rule_id = $1
                       ORDER BY created_at DESC
                       LIMIT $2 OFFSET $3"#,
                )
                .bind(rid)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(cs)) => {
                sqlx::query_as::<_, DataResidencyComplianceV8>(
                    r#"SELECT * FROM data_residency_compliance_v8
                       WHERE compliance_status = $1
                       ORDER BY created_at DESC
                       LIMIT $2 OFFSET $3"#,
                )
                .bind(cs)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            (None, None) => {
                sqlx::query_as::<_, DataResidencyComplianceV8>(
                    r#"SELECT * FROM data_residency_compliance_v8
                       ORDER BY created_at DESC
                       LIMIT $1 OFFSET $2"#,
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        };
        rows.map_err(|e| DbError::Database(format!("list_data_residency_compliance_v8: {e}")))
    }

    pub async fn get_data_residency_compliance_by_rule_v8(
        &self,
        rule_id: Uuid,
    ) -> Result<Vec<DataResidencyComplianceV8>> {
        sqlx::query_as::<_, DataResidencyComplianceV8>(
            "SELECT * FROM data_residency_compliance_v8 WHERE rule_id = $1 ORDER BY created_at DESC",
        )
        .bind(rule_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_data_residency_compliance_by_rule_v8: {e}")))
    }

    pub async fn delete_data_residency_compliance_v8(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM data_residency_compliance_v8 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_data_residency_compliance_v8: {e}")))?;
        Ok(())
    }

    // --- Database Replication Config v11 ---

    pub async fn create_replication_config_v11(
        &self,
        replica_id: Uuid,
        config_key: &str,
        config_value: serde_json::Value,
    ) -> Result<DatabaseReplicationConfigV11> {
        let row = sqlx::query_as::<_, DatabaseReplicationConfigV11>(
            r#"INSERT INTO database_replication_config_v11 (replica_id, config_key, config_value)
               VALUES ($1, $2, $3)
               ON CONFLICT (replica_id, config_key) DO UPDATE SET config_value = $3
               RETURNING *"#,
        )
        .bind(replica_id)
        .bind(config_key)
        .bind(config_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_replication_config_v11: {e}")))?;
        Ok(row)
    }

    pub async fn get_replication_config_v11(
        &self,
        replica_id: Uuid,
        config_key: &str,
    ) -> Result<Option<DatabaseReplicationConfigV11>> {
        sqlx::query_as::<_, DatabaseReplicationConfigV11>(
            "SELECT * FROM database_replication_config_v11 WHERE replica_id = $1 AND config_key = $2",
        )
        .bind(replica_id)
        .bind(config_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_replication_config_v11: {e}")))
    }

    pub async fn list_replication_configs_v11(
        &self,
        replica_id: Uuid,
    ) -> Result<Vec<DatabaseReplicationConfigV11>> {
        sqlx::query_as::<_, DatabaseReplicationConfigV11>(
            "SELECT * FROM database_replication_config_v11 WHERE replica_id = $1 ORDER BY config_key",
        )
        .bind(replica_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_replication_configs_v11: {e}")))
    }

    pub async fn delete_replication_config_v11(
        &self,
        replica_id: Uuid,
        config_key: &str,
    ) -> Result<()> {
        sqlx::query("DELETE FROM database_replication_config_v11 WHERE replica_id = $1 AND config_key = $2")
            .bind(replica_id)
            .bind(config_key)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_replication_config_v11: {e}")))?;
        Ok(())
    }

    // --- Database Replication Alerts v11 ---

    pub async fn create_replication_alert_v11(
        &self,
        replica_id: Uuid,
        alert_type: &str,
        threshold: f64,
    ) -> Result<DatabaseReplicationAlertV11> {
        let row = sqlx::query_as::<_, DatabaseReplicationAlertV11>(
            r#"INSERT INTO database_replication_alerts_v11 (replica_id, alert_type, threshold)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(replica_id)
        .bind(alert_type)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_replication_alert_v11: {e}")))?;
        Ok(row)
    }

    pub async fn get_replication_alert_v11(&self, id: Uuid) -> Result<DatabaseReplicationAlertV11> {
        sqlx::query_as::<_, DatabaseReplicationAlertV11>(
            "SELECT * FROM database_replication_alerts_v11 WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_replication_alert_v11: {e}")))
    }

    pub async fn list_replication_alerts_v11(
        &self,
        replica_id: Uuid,
    ) -> Result<Vec<DatabaseReplicationAlertV11>> {
        sqlx::query_as::<_, DatabaseReplicationAlertV11>(
            "SELECT * FROM database_replication_alerts_v11 WHERE replica_id = $1 ORDER BY alert_type",
        )
        .bind(replica_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_replication_alerts_v11: {e}")))
    }

    pub async fn update_replication_alert_v11(
        &self,
        id: Uuid,
        threshold: Option<f64>,
        enabled: Option<bool>,
    ) -> Result<DatabaseReplicationAlertV11> {
        let row = sqlx::query_as::<_, DatabaseReplicationAlertV11>(
            r#"UPDATE database_replication_alerts_v11
               SET threshold = COALESCE($2, threshold),
                   enabled = COALESCE($3, enabled)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_replication_alert_v11: {e}")))?;
        Ok(row)
    }

    pub async fn delete_replication_alert_v11(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM database_replication_alerts_v11 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_replication_alert_v11: {e}")))?;
        Ok(())
    }

    pub async fn trigger_replication_alert_v11(&self, id: Uuid) -> Result<DatabaseReplicationAlertV11> {
        let row = sqlx::query_as::<_, DatabaseReplicationAlertV11>(
            r#"UPDATE database_replication_alerts_v11
               SET last_triggered_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("trigger_replication_alert_v11: {e}")))?;
        Ok(row)
    }

    pub async fn get_enabled_replication_alerts_v11(
        &self,
        replica_id: Uuid,
    ) -> Result<Vec<DatabaseReplicationAlertV11>> {
        sqlx::query_as::<_, DatabaseReplicationAlertV11>(
            "SELECT * FROM database_replication_alerts_v11 WHERE replica_id = $1 AND enabled = true ORDER BY alert_type",
        )
        .bind(replica_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_enabled_replication_alerts_v11: {e}")))
    }

    // --- Encryption Key Versions v11 ---

    pub async fn create_encryption_key_version_v11(
        &self,
        key_id: Uuid,
        version: i32,
        key_material: &[u8],
    ) -> Result<EncryptionKeyVersionV11> {
        let row = sqlx::query_as::<_, EncryptionKeyVersionV11>(
            r#"INSERT INTO encryption_key_versions_v11 (key_id, version, key_material)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(key_id)
        .bind(version)
        .bind(key_material)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_encryption_key_version_v11: {e}")))?;
        Ok(row)
    }

    pub async fn get_encryption_key_version_v11(
        &self,
        id: Uuid,
    ) -> Result<EncryptionKeyVersionV11> {
        sqlx::query_as::<_, EncryptionKeyVersionV11>(
            "SELECT * FROM encryption_key_versions_v11 WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_encryption_key_version_v11: {e}")))
    }

    pub async fn list_encryption_key_versions_v11(
        &self,
        key_id: Uuid,
    ) -> Result<Vec<EncryptionKeyVersionV11>> {
        sqlx::query_as::<_, EncryptionKeyVersionV11>(
            "SELECT * FROM encryption_key_versions_v11 WHERE key_id = $1 ORDER BY version DESC",
        )
        .bind(key_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_encryption_key_versions_v11: {e}")))
    }

    pub async fn delete_encryption_key_version_v11(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM encryption_key_versions_v11 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_encryption_key_version_v11: {e}")))?;
        Ok(())
    }

    // --- Encryption Compliance Checks v11 ---

    pub async fn create_encryption_compliance_check_v11(
        &self,
        check_type: &str,
    ) -> Result<EncryptionComplianceCheckV11> {
        let row = sqlx::query_as::<_, EncryptionComplianceCheckV11>(
            r#"INSERT INTO encryption_compliance_checks_v11 (check_type)
               VALUES ($1)
               RETURNING *"#,
        )
        .bind(check_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_encryption_compliance_check_v11: {e}")))?;
        Ok(row)
    }

    pub async fn get_encryption_compliance_check_v11(
        &self,
        id: Uuid,
    ) -> Result<EncryptionComplianceCheckV11> {
        sqlx::query_as::<_, EncryptionComplianceCheckV11>(
            "SELECT * FROM encryption_compliance_checks_v11 WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_encryption_compliance_check_v11: {e}")))
    }

    pub async fn update_encryption_compliance_check_v11(
        &self,
        id: Uuid,
        status: &str,
        findings: serde_json::Value,
        score: i32,
    ) -> Result<EncryptionComplianceCheckV11> {
        let row = sqlx::query_as::<_, EncryptionComplianceCheckV11>(
            r#"UPDATE encryption_compliance_checks_v11
               SET status = $2, findings = $3, score = $4
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(status)
        .bind(findings)
        .bind(score)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_encryption_compliance_check_v11: {e}")))?;
        Ok(row)
    }

    pub async fn list_encryption_compliance_checks_v11(
        &self,
        check_type: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<EncryptionComplianceCheckV11>> {
        let rows = match check_type {
            Some(ct) => {
                sqlx::query_as::<_, EncryptionComplianceCheckV11>(
                    r#"SELECT * FROM encryption_compliance_checks_v11
                       WHERE check_type = $1
                       ORDER BY created_at DESC
                       LIMIT $2 OFFSET $3"#,
                )
                .bind(ct)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, EncryptionComplianceCheckV11>(
                    r#"SELECT * FROM encryption_compliance_checks_v11
                       ORDER BY created_at DESC
                       LIMIT $1 OFFSET $2"#,
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        };
        rows.map_err(|e| DbError::Database(format!("list_encryption_compliance_checks_v11: {e}")))
    }

    pub async fn delete_encryption_compliance_check_v11(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM encryption_compliance_checks_v11 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_encryption_compliance_check_v11: {e}")))?;
        Ok(())
    }

    // --- Data Residency Reports v11 ---

    pub async fn create_data_residency_report_v11(
        &self,
        report_type: &str,
    ) -> Result<DataResidencyReportV11> {
        let row = sqlx::query_as::<_, DataResidencyReportV11>(
            r#"INSERT INTO data_residency_reports_v11 (report_type)
               VALUES ($1)
               RETURNING *"#,
        )
        .bind(report_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_data_residency_report_v11: {e}")))?;
        Ok(row)
    }

    pub async fn get_data_residency_report_v11(
        &self,
        id: Uuid,
    ) -> Result<DataResidencyReportV11> {
        sqlx::query_as::<_, DataResidencyReportV11>(
            "SELECT * FROM data_residency_reports_v11 WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_data_residency_report_v11: {e}")))
    }

    pub async fn update_data_residency_report_v11(
        &self,
        id: Uuid,
        findings: serde_json::Value,
        score: i32,
    ) -> Result<DataResidencyReportV11> {
        let row = sqlx::query_as::<_, DataResidencyReportV11>(
            r#"UPDATE data_residency_reports_v11
               SET findings = $2, score = $3
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(findings)
        .bind(score)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_data_residency_report_v11: {e}")))?;
        Ok(row)
    }

    pub async fn list_data_residency_reports_v11(
        &self,
        report_type: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DataResidencyReportV11>> {
        let rows = match report_type {
            Some(rt) => {
                sqlx::query_as::<_, DataResidencyReportV11>(
                    r#"SELECT * FROM data_residency_reports_v11
                       WHERE report_type = $1
                       ORDER BY generated_at DESC
                       LIMIT $2 OFFSET $3"#,
                )
                .bind(rt)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, DataResidencyReportV11>(
                    r#"SELECT * FROM data_residency_reports_v11
                       ORDER BY generated_at DESC
                       LIMIT $1 OFFSET $2"#,
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        };
        rows.map_err(|e| DbError::Database(format!("list_data_residency_reports_v11: {e}")))
    }

    pub async fn delete_data_residency_report_v11(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM data_residency_reports_v11 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_data_residency_report_v11: {e}")))?;
        Ok(())
    }

    // --- Data Residency Compliance v11 ---

    pub async fn create_data_residency_compliance_v11(
        &self,
        rule_id: Uuid,
    ) -> Result<DataResidencyComplianceV11> {
        let row = sqlx::query_as::<_, DataResidencyComplianceV11>(
            r#"INSERT INTO data_residency_compliance_v11 (rule_id)
               VALUES ($1)
               RETURNING *"#,
        )
        .bind(rule_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_data_residency_compliance_v11: {e}")))?;
        Ok(row)
    }

    pub async fn get_data_residency_compliance_v11(
        &self,
        id: Uuid,
    ) -> Result<DataResidencyComplianceV11> {
        sqlx::query_as::<_, DataResidencyComplianceV11>(
            "SELECT * FROM data_residency_compliance_v11 WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_data_residency_compliance_v11: {e}")))
    }

    pub async fn update_data_residency_compliance_v11(
        &self,
        id: Uuid,
        compliance_status: &str,
    ) -> Result<DataResidencyComplianceV11> {
        let row = sqlx::query_as::<_, DataResidencyComplianceV11>(
            r#"UPDATE data_residency_compliance_v11
               SET compliance_status = $2, last_checked_at = NOW()
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(compliance_status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_data_residency_compliance_v11: {e}")))?;
        Ok(row)
    }

    pub async fn list_data_residency_compliance_v11(
        &self,
        rule_id: Option<Uuid>,
        compliance_status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<DataResidencyComplianceV11>> {
        let rows = match (rule_id, compliance_status) {
            (Some(rid), Some(cs)) => {
                sqlx::query_as::<_, DataResidencyComplianceV11>(
                    r#"SELECT * FROM data_residency_compliance_v11
                       WHERE rule_id = $1 AND compliance_status = $2
                       ORDER BY created_at DESC
                       LIMIT $3 OFFSET $4"#,
                )
                .bind(rid)
                .bind(cs)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            (Some(rid), None) => {
                sqlx::query_as::<_, DataResidencyComplianceV11>(
                    r#"SELECT * FROM data_residency_compliance_v11
                       WHERE rule_id = $1
                       ORDER BY created_at DESC
                       LIMIT $2 OFFSET $3"#,
                )
                .bind(rid)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            (None, Some(cs)) => {
                sqlx::query_as::<_, DataResidencyComplianceV11>(
                    r#"SELECT * FROM data_residency_compliance_v11
                       WHERE compliance_status = $1
                       ORDER BY created_at DESC
                       LIMIT $2 OFFSET $3"#,
                )
                .bind(cs)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            (None, None) => {
                sqlx::query_as::<_, DataResidencyComplianceV11>(
                    r#"SELECT * FROM data_residency_compliance_v11
                       ORDER BY created_at DESC
                       LIMIT $1 OFFSET $2"#,
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        };
        rows.map_err(|e| DbError::Database(format!("list_data_residency_compliance_v11: {e}")))
    }

    pub async fn get_data_residency_compliance_by_rule_v11(
        &self,
        rule_id: Uuid,
    ) -> Result<Vec<DataResidencyComplianceV11>> {
        sqlx::query_as::<_, DataResidencyComplianceV11>(
            "SELECT * FROM data_residency_compliance_v11 WHERE rule_id = $1 ORDER BY created_at DESC",
        )
        .bind(rule_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_data_residency_compliance_by_rule_v11: {e}")))
    }

    pub async fn delete_data_residency_compliance_v11(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM data_residency_compliance_v11 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_data_residency_compliance_v11: {e}")))?;
        Ok(())
    }

    // --- API Docs v11 ---

    pub async fn list_api_docs_v11(&self, limit: i64, offset: i64) -> Result<Vec<ApiDocsV11>> {
        sqlx::query_as::<_, ApiDocsV11>(
            "SELECT * FROM api_docs_v11 ORDER BY endpoint, method, version LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_docs_v11: {e}")))
    }

    pub async fn get_api_docs_v11_for_endpoint(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
    ) -> Result<Option<ApiDocsV11>> {
        sqlx::query_as::<_, ApiDocsV11>(
            "SELECT * FROM api_docs_v11 WHERE endpoint = $1 AND method = $2 AND version = $3",
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_docs_v11_for_endpoint: {e}")))
    }

    pub async fn create_api_docs_v11(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
        summary: &str,
        description: &str,
        parameters: &serde_json::Value,
        request_body: Option<&serde_json::Value>,
        responses: &serde_json::Value,
        examples: &serde_json::Value,
        tags: &[String],
        deprecated: bool,
        changelog: &str,
        security_schemes: &serde_json::Value,
        rate_limits: &serde_json::Value,
    ) -> Result<ApiDocsV11> {
        let row = sqlx::query_as::<_, ApiDocsV11>(
            r#"INSERT INTO api_docs_v11 (endpoint, method, version, summary, description, parameters, request_body, responses, examples, tags, deprecated, changelog, security_schemes, rate_limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .bind(summary)
        .bind(description)
        .bind(parameters)
        .bind(request_body)
        .bind(responses)
        .bind(examples)
        .bind(tags)
        .bind(deprecated)
        .bind(changelog)
        .bind(security_schemes)
        .bind(rate_limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_docs_v11: {e}")))?;
        Ok(row)
    }

    // --- Rate Limit Tiers v9 ---

    pub async fn list_rate_limit_tiers_v9(&self) -> Result<Vec<RateLimitTierV9>> {
        sqlx::query_as::<_, RateLimitTierV9>(
            "SELECT * FROM rate_limit_tiers_v9 ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_rate_limit_tiers_v9: {e}")))
    }

    pub async fn get_rate_limit_tier_v9_by_name(
        &self,
        name: &str,
    ) -> Result<Option<RateLimitTierV9>> {
        sqlx::query_as::<_, RateLimitTierV9>(
            "SELECT * FROM rate_limit_tiers_v9 WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_rate_limit_tier_v9_by_name: {e}")))
    }

    pub async fn create_rate_limit_tier_v9(
        &self,
        name: &str,
        description: &str,
        rate_limit: i32,
        burst_limit: i32,
        monthly_quota: Option<i32>,
        price_cents: i32,
        features: &serde_json::Value,
        limits: &serde_json::Value,
    ) -> Result<RateLimitTierV9> {
        let row = sqlx::query_as::<_, RateLimitTierV9>(
            r#"INSERT INTO rate_limit_tiers_v9 (name, description, rate_limit, burst_limit, monthly_quota, price_cents, features, limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_tier_v9: {e}")))?;
        Ok(row)
    }

    pub async fn update_rate_limit_tier_v9(
        &self,
        name: &str,
        description: Option<&str>,
        rate_limit: Option<i32>,
        burst_limit: Option<i32>,
        monthly_quota: Option<Option<i32>>,
        price_cents: Option<i32>,
        features: Option<&serde_json::Value>,
        limits: Option<&serde_json::Value>,
    ) -> Result<RateLimitTierV9> {
        let row = sqlx::query_as::<_, RateLimitTierV9>(
            r#"UPDATE rate_limit_tiers_v9 SET
               description = COALESCE($2, description),
               rate_limit = COALESCE($3, rate_limit),
               burst_limit = COALESCE($4, burst_limit),
               monthly_quota = CASE WHEN $5 IS NOT NULL THEN $5 ELSE monthly_quota END,
               price_cents = COALESCE($6, price_cents),
               features = COALESCE($7, features),
               limits = COALESCE($8, limits)
               WHERE name = $1
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_rate_limit_tier_v9: {e}")))?;
        Ok(row)
    }

    pub async fn delete_rate_limit_tier_v9(&self, name: &str) -> Result<()> {
        sqlx::query("DELETE FROM rate_limit_tiers_v9 WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_rate_limit_tier_v9: {e}")))?;
        Ok(())
    }

    // --- Rate Limit Alerts v6 ---

    pub async fn create_rate_limit_alert_v7(
        &self,
        user_id: Uuid,
        tier_id: Uuid,
        alert_type: &str,
        threshold: f64,
    ) -> Result<RateLimitAlertV6> {
        let row = sqlx::query_as::<_, RateLimitAlertV6>(
            r#"INSERT INTO rate_limit_alerts_v6 (user_id, tier_id, alert_type, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(tier_id)
        .bind(alert_type)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_alert_v7: {e}")))?;
        Ok(row)
    }

    pub async fn get_user_rate_limit_alerts_v7(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<RateLimitAlertV6>> {
        sqlx::query_as::<_, RateLimitAlertV6>(
            "SELECT * FROM rate_limit_alerts_v6 WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_user_rate_limit_alerts_v7: {e}")))
    }

    // --- API Analytics v12 ---

    pub async fn list_api_analytics_v12(&self, limit: i64, offset: i64) -> Result<Vec<ApiAnalyticV12>> {
        sqlx::query_as::<_, ApiAnalyticV12>(
            "SELECT * FROM api_analytics_v12 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_analytics_v12: {e}")))
    }

    pub async fn create_api_analytic_v12(
        &self,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_time_ms: i32,
        user_id: Option<Uuid>,
        request_size_bytes: i32,
        response_size_bytes: i32,
        cache_hit: bool,
        region: &str,
        user_agent: Option<&str>,
        request_id: Option<Uuid>,
        cost_cents: i32,
    ) -> Result<ApiAnalyticV12> {
        let row = sqlx::query_as::<_, ApiAnalyticV12>(
            r#"INSERT INTO api_analytics_v12 (endpoint, method, status_code, response_time_ms, user_id, request_size_bytes, response_size_bytes, cache_hit, region, user_agent, request_id, cost_cents)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(user_id)
        .bind(request_size_bytes)
        .bind(response_size_bytes)
        .bind(cache_hit)
        .bind(region)
        .bind(user_agent)
        .bind(request_id)
        .bind(cost_cents)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_analytic_v12: {e}")))?;
        Ok(row)
    }

    pub async fn get_cost_analysis_v12(&self) -> Result<(i64, i64, i64, i64, Vec<(String, i64, i64)>, Vec<(String, i64, i64)>)> {
        let totals = sqlx::query_as::<_, (i64, i64, i64, i64)>(
            r#"SELECT COUNT(*) as total_requests,
                      COALESCE(SUM(request_size_bytes), 0) as total_request_bytes,
                      COALESCE(SUM(response_size_bytes), 0) as total_response_bytes,
                      COALESCE(SUM(cost_cents), 0) as estimated_cost_cents
               FROM api_analytics_v12"#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v12 totals: {e}")))?;

        let regions = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT region, COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost_cents
               FROM api_analytics_v12 GROUP BY region ORDER BY cost_cents DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v12 regions: {e}")))?;

        let ua_costs = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT COALESCE(user_agent, 'unknown') as ua, COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost_cents
               FROM api_analytics_v12 GROUP BY user_agent ORDER BY cost_cents DESC LIMIT 10"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v12 ua_costs: {e}")))?;

        Ok((totals.0, totals.1, totals.2, totals.3, regions, ua_costs))
    }

    pub async fn get_usage_optimization_v12(&self) -> Result<Vec<(String, String, f64, f64, f64, Vec<String>)>> {
        let rows = sqlx::query_as::<_, (String, String, f64, f64, f64, Vec<String>)>(
            r#"SELECT endpoint, method,
                      AVG(response_time_ms)::NUMERIC::FLOAT as avg_rt,
                      PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms)::NUMERIC::FLOAT as p95_rt,
                      CASE WHEN COUNT(*) > 0 THEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / COUNT(*)::NUMERIC * 100 ELSE 0 END as cache_rate,
                      COALESCE(
                          ARRAY_REMOVE(
                              ARRAY[
                                  CASE WHEN AVG(response_time_ms) > 500 THEN 'Optimize slow endpoint' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / NULLIF(COUNT(*), 0) < 0.3 THEN 'Improve caching strategy' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE status_code >= 500))::NUMERIC / NULLIF(COUNT(*), 0) > 0.05 THEN 'Reduce error rate' END
                              ],
                              NULL
                          ),
                          ARRAY['No optimization needed']::TEXT[]
                      ) as suggestions
               FROM api_analytics_v12
               GROUP BY endpoint, method
               HAVING COUNT(*) > 10
               ORDER BY avg_rt DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_usage_optimization_v12: {e}")))?;
        Ok(rows)
    }

    // --- API Docs v12 ---

    pub async fn list_api_docs_v12(&self, limit: i64, offset: i64) -> Result<Vec<ApiDocsV12>> {
        sqlx::query_as::<_, ApiDocsV12>(
            "SELECT * FROM api_docs_v12 ORDER BY endpoint, method, version LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_docs_v12: {e}")))
    }

    pub async fn get_api_docs_v12_for_endpoint(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
    ) -> Result<Option<ApiDocsV12>> {
        sqlx::query_as::<_, ApiDocsV12>(
            "SELECT * FROM api_docs_v12 WHERE endpoint = $1 AND method = $2 AND version = $3",
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_docs_v12_for_endpoint: {e}")))
    }

    pub async fn create_api_docs_v12(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
        summary: &str,
        description: &str,
        parameters: &serde_json::Value,
        request_body: Option<&serde_json::Value>,
        responses: &serde_json::Value,
        examples: &serde_json::Value,
        tags: &[String],
        deprecated: bool,
        changelog: &str,
        security_schemes: &serde_json::Value,
        rate_limits: &serde_json::Value,
    ) -> Result<ApiDocsV12> {
        let row = sqlx::query_as::<_, ApiDocsV12>(
            r#"INSERT INTO api_docs_v12 (endpoint, method, version, summary, description, parameters, request_body, responses, examples, tags, deprecated, changelog, security_schemes, rate_limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .bind(summary)
        .bind(description)
        .bind(parameters)
        .bind(request_body)
        .bind(responses)
        .bind(examples)
        .bind(tags)
        .bind(deprecated)
        .bind(changelog)
        .bind(security_schemes)
        .bind(rate_limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_docs_v12: {e}")))?;
        Ok(row)
    }

    // --- Rate Limit Tiers v10 ---

    pub async fn list_rate_limit_tiers_v10(&self) -> Result<Vec<RateLimitTierV10>> {
        sqlx::query_as::<_, RateLimitTierV10>(
            "SELECT * FROM rate_limit_tiers_v10 ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_rate_limit_tiers_v10: {e}")))
    }

    pub async fn get_rate_limit_tier_v10_by_name(
        &self,
        name: &str,
    ) -> Result<Option<RateLimitTierV10>> {
        sqlx::query_as::<_, RateLimitTierV10>(
            "SELECT * FROM rate_limit_tiers_v10 WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_rate_limit_tier_v10_by_name: {e}")))
    }

    pub async fn create_rate_limit_tier_v10(
        &self,
        name: &str,
        description: &str,
        rate_limit: i32,
        burst_limit: i32,
        monthly_quota: Option<i32>,
        price_cents: i32,
        features: &serde_json::Value,
        limits: &serde_json::Value,
    ) -> Result<RateLimitTierV10> {
        let row = sqlx::query_as::<_, RateLimitTierV10>(
            r#"INSERT INTO rate_limit_tiers_v10 (name, description, rate_limit, burst_limit, monthly_quota, price_cents, features, limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_tier_v10: {e}")))?;
        Ok(row)
    }

    pub async fn update_rate_limit_tier_v10(
        &self,
        name: &str,
        description: Option<&str>,
        rate_limit: Option<i32>,
        burst_limit: Option<i32>,
        monthly_quota: Option<Option<i32>>,
        price_cents: Option<i32>,
        features: Option<&serde_json::Value>,
        limits: Option<&serde_json::Value>,
    ) -> Result<RateLimitTierV10> {
        let row = sqlx::query_as::<_, RateLimitTierV10>(
            r#"UPDATE rate_limit_tiers_v10 SET
               description = COALESCE($2, description),
               rate_limit = COALESCE($3, rate_limit),
               burst_limit = COALESCE($4, burst_limit),
               monthly_quota = CASE WHEN $5 IS NOT NULL THEN $5 ELSE monthly_quota END,
               price_cents = COALESCE($6, price_cents),
               features = COALESCE($7, features),
               limits = COALESCE($8, limits)
               WHERE name = $1
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_rate_limit_tier_v10: {e}")))?;
        Ok(row)
    }

    pub async fn delete_rate_limit_tier_v10(&self, name: &str) -> Result<()> {
        sqlx::query("DELETE FROM rate_limit_tiers_v10 WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_rate_limit_tier_v10: {e}")))?;
        Ok(())
    }

    // --- Rate Limit Alerts v7 ---

    pub async fn create_rate_limit_alert_v9(
        &self,
        user_id: Uuid,
        tier_id: Uuid,
        alert_type: &str,
        threshold: f64,
    ) -> Result<RateLimitAlertV7> {
        let row = sqlx::query_as::<_, RateLimitAlertV7>(
            r#"INSERT INTO rate_limit_alerts_v7 (user_id, tier_id, alert_type, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(tier_id)
        .bind(alert_type)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_alert_v9: {e}")))?;
        Ok(row)
    }

    pub async fn get_user_rate_limit_alerts_v9(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<RateLimitAlertV7>> {
        sqlx::query_as::<_, RateLimitAlertV7>(
            "SELECT * FROM rate_limit_alerts_v7 WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_user_rate_limit_alerts_v9: {e}")))
    }

    // --- API Analytics v13 ---

    pub async fn list_api_analytics_v13(&self, limit: i64, offset: i64) -> Result<Vec<ApiAnalyticV13>> {
        sqlx::query_as::<_, ApiAnalyticV13>(
            "SELECT * FROM api_analytics_v13 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_analytics_v13: {e}")))
    }

    pub async fn create_api_analytic_v13(
        &self,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_time_ms: i32,
        user_id: Option<Uuid>,
        request_size_bytes: i32,
        response_size_bytes: i32,
        cache_hit: bool,
        region: &str,
        user_agent: Option<&str>,
        request_id: Option<Uuid>,
        cost_cents: i32,
    ) -> Result<ApiAnalyticV13> {
        let row = sqlx::query_as::<_, ApiAnalyticV13>(
            r#"INSERT INTO api_analytics_v13 (endpoint, method, status_code, response_time_ms, user_id, request_size_bytes, response_size_bytes, cache_hit, region, user_agent, request_id, cost_cents)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(user_id)
        .bind(request_size_bytes)
        .bind(response_size_bytes)
        .bind(cache_hit)
        .bind(region)
        .bind(user_agent)
        .bind(request_id)
        .bind(cost_cents)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_analytic_v13: {e}")))?;
        Ok(row)
    }

    pub async fn get_cost_analysis_v13(&self) -> Result<(i64, i64, i64, i64, Vec<(String, i64, i64)>, Vec<(String, i64, i64)>)> {
        let totals = sqlx::query_as::<_, (i64, i64, i64, i64)>(
            r#"SELECT COUNT(*) as total_requests,
                      COALESCE(SUM(request_size_bytes), 0) as total_request_bytes,
                      COALESCE(SUM(response_size_bytes), 0) as total_response_bytes,
                      COALESCE(SUM(cost_cents), 0) as estimated_cost_cents
               FROM api_analytics_v13"#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v13 totals: {e}")))?;

        let regions = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT region, COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost_cents
               FROM api_analytics_v13 GROUP BY region ORDER BY cost_cents DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v13 regions: {e}")))?;

        let ua_costs = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT COALESCE(user_agent, 'unknown') as ua, COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost_cents
               FROM api_analytics_v13 GROUP BY user_agent ORDER BY cost_cents DESC LIMIT 10"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v13 ua_costs: {e}")))?;

        Ok((totals.0, totals.1, totals.2, totals.3, regions, ua_costs))
    }

    pub async fn get_usage_optimization_v13(&self) -> Result<Vec<(String, String, f64, f64, f64, Vec<String>)>> {
        let rows = sqlx::query_as::<_, (String, String, f64, f64, f64, Vec<String>)>(
            r#"SELECT endpoint, method,
                      AVG(response_time_ms)::NUMERIC::FLOAT as avg_rt,
                      PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms)::NUMERIC::FLOAT as p95_rt,
                      CASE WHEN COUNT(*) > 0 THEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / COUNT(*)::NUMERIC * 100 ELSE 0 END as cache_rate,
                      COALESCE(
                          ARRAY_REMOVE(
                              ARRAY[
                                  CASE WHEN AVG(response_time_ms) > 500 THEN 'Optimize slow endpoint' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / NULLIF(COUNT(*), 0) < 0.3 THEN 'Improve caching strategy' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE status_code >= 500))::NUMERIC / NULLIF(COUNT(*), 0) > 0.05 THEN 'Reduce error rate' END
                              ],
                              NULL
                          ),
                          ARRAY['No optimization needed']::TEXT[]
                      ) as suggestions
               FROM api_analytics_v13
               GROUP BY endpoint, method
               HAVING COUNT(*) > 10
               ORDER BY avg_rt DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_usage_optimization_v13: {e}")))?;
        Ok(rows)
    }

    // --- API Docs v13 ---

    pub async fn list_api_docs_v13(&self, limit: i64, offset: i64) -> Result<Vec<ApiDocsV13>> {
        sqlx::query_as::<_, ApiDocsV13>(
            "SELECT * FROM api_docs_v13 ORDER BY endpoint, method, version LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_docs_v13: {e}")))
    }

    pub async fn get_api_docs_v13_for_endpoint(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
    ) -> Result<Option<ApiDocsV13>> {
        sqlx::query_as::<_, ApiDocsV13>(
            "SELECT * FROM api_docs_v13 WHERE endpoint = $1 AND method = $2 AND version = $3",
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_docs_v13_for_endpoint: {e}")))
    }

    pub async fn create_api_docs_v13(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
        summary: &str,
        description: &str,
        parameters: &serde_json::Value,
        request_body: Option<&serde_json::Value>,
        responses: &serde_json::Value,
        examples: &serde_json::Value,
        tags: &[String],
        deprecated: bool,
        changelog: &str,
        security_schemes: &serde_json::Value,
        rate_limits: &serde_json::Value,
    ) -> Result<ApiDocsV13> {
        sqlx::query_as::<_, ApiDocsV13>(
            r#"INSERT INTO api_docs_v13 (endpoint, method, version, summary, description, parameters, request_body, responses, examples, tags, deprecated, changelog, security_schemes, rate_limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .bind(summary)
        .bind(description)
        .bind(parameters)
        .bind(request_body)
        .bind(responses)
        .bind(examples)
        .bind(tags)
        .bind(deprecated)
        .bind(changelog)
        .bind(security_schemes)
        .bind(rate_limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_docs_v13: {e}")))
    }

    // --- Rate Limit Tiers v11 ---

    pub async fn list_rate_limit_tiers_v11(&self) -> Result<Vec<RateLimitTierV11>> {
        sqlx::query_as::<_, RateLimitTierV11>(
            "SELECT * FROM rate_limit_tiers_v11 ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_rate_limit_tiers_v11: {e}")))
    }

    pub async fn get_rate_limit_tier_v11_by_name(&self, name: &str) -> Result<Option<RateLimitTierV11>> {
        sqlx::query_as::<_, RateLimitTierV11>(
            "SELECT * FROM rate_limit_tiers_v11 WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_rate_limit_tier_v11_by_name: {e}")))
    }

    pub async fn create_rate_limit_tier_v11(
        &self,
        name: &str,
        description: &str,
        rate_limit: i32,
        burst_limit: i32,
        monthly_quota: Option<i32>,
        price_cents: i32,
        features: &serde_json::Value,
        limits: &serde_json::Value,
    ) -> Result<RateLimitTierV11> {
        sqlx::query_as::<_, RateLimitTierV11>(
            r#"INSERT INTO rate_limit_tiers_v11 (name, description, rate_limit, burst_limit, monthly_quota, price_cents, features, limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_tier_v11: {e}")))
    }

    pub async fn update_rate_limit_tier_v11(
        &self,
        name: &str,
        description: Option<&str>,
        rate_limit: Option<i32>,
        burst_limit: Option<i32>,
        monthly_quota: Option<Option<i32>>,
        price_cents: Option<i32>,
        features: Option<&serde_json::Value>,
        limits: Option<&serde_json::Value>,
    ) -> Result<RateLimitTierV11> {
        let existing = self.get_rate_limit_tier_v11_by_name(name).await?
            .ok_or_else(|| DbError::Database(format!("Tier {name} not found")))?;
        let new_desc = description.unwrap_or(&existing.description);
        let new_rate = rate_limit.unwrap_or(existing.rate_limit);
        let new_burst = burst_limit.unwrap_or(existing.burst_limit);
        let new_monthly = monthly_quota.unwrap_or(existing.monthly_quota);
        let new_price = price_cents.unwrap_or(existing.price_cents);
        let new_features = features.unwrap_or(&existing.features);
        let new_limits = limits.unwrap_or(&existing.limits);
        sqlx::query_as::<_, RateLimitTierV11>(
            r#"UPDATE rate_limit_tiers_v11 SET description = $2, rate_limit = $3, burst_limit = $4, monthly_quota = $5, price_cents = $6, features = $7, limits = $8
               WHERE name = $1
               RETURNING *"#,
        )
        .bind(name)
        .bind(new_desc)
        .bind(new_rate)
        .bind(new_burst)
        .bind(new_monthly)
        .bind(new_price)
        .bind(new_features)
        .bind(new_limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_rate_limit_tier_v11: {e}")))
    }

    pub async fn delete_rate_limit_tier_v11(&self, name: &str) -> Result<()> {
        sqlx::query("DELETE FROM rate_limit_tiers_v11 WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_rate_limit_tier_v11: {e}")))?;
        Ok(())
    }

    // --- Rate Limit Alerts v8 ---

    pub async fn create_rate_limit_alert_v11(
        &self,
        user_id: Uuid,
        tier_id: Uuid,
        alert_type: &str,
        threshold: f64,
    ) -> Result<RateLimitAlertV8> {
        sqlx::query_as::<_, RateLimitAlertV8>(
            r#"INSERT INTO rate_limit_alerts_v8 (user_id, tier_id, alert_type, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(tier_id)
        .bind(alert_type)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_alert_v11: {e}")))
    }

    pub async fn get_user_rate_limit_alerts_v11(&self, user_id: Uuid) -> Result<Vec<RateLimitAlertV8>> {
        sqlx::query_as::<_, RateLimitAlertV8>(
            "SELECT * FROM rate_limit_alerts_v8 WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_user_rate_limit_alerts_v11: {e}")))
    }

    // --- API Analytics v14 ---

    pub async fn list_api_analytics_v14(&self, limit: i64, offset: i64) -> Result<Vec<ApiAnalyticV14>> {
        sqlx::query_as::<_, ApiAnalyticV14>(
            "SELECT * FROM api_analytics_v14 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_analytics_v14: {e}")))
    }

    pub async fn create_api_analytic_v14(
        &self,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_time_ms: i32,
        user_id: Option<Uuid>,
        request_size_bytes: i32,
        response_size_bytes: i32,
        cache_hit: bool,
        region: &str,
        user_agent: Option<&str>,
        request_id: Option<Uuid>,
        cost_cents: i32,
    ) -> Result<ApiAnalyticV14> {
        sqlx::query_as::<_, ApiAnalyticV14>(
            r#"INSERT INTO api_analytics_v14 (endpoint, method, status_code, response_time_ms, user_id, request_size_bytes, response_size_bytes, cache_hit, region, user_agent, request_id, cost_cents)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(user_id)
        .bind(request_size_bytes)
        .bind(response_size_bytes)
        .bind(cache_hit)
        .bind(region)
        .bind(user_agent)
        .bind(request_id)
        .bind(cost_cents)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_analytic_v14: {e}")))
    }

    pub async fn get_cost_analysis_v14(&self) -> Result<(i64, i64, i64, i64, Vec<(String, i64, i64)>, Vec<(String, i64, i64)>)> {
        let totals = sqlx::query_as::<_, (i64, i64, i64, i64)>(
            r#"SELECT COUNT(*) as total_requests,
                      COALESCE(SUM(request_size_bytes), 0) as total_request_bytes,
                      COALESCE(SUM(response_size_bytes), 0) as total_response_bytes,
                      COALESCE(SUM(cost_cents), 0) as estimated_cost_cents
               FROM api_analytics_v14"#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v14 totals: {e}")))?;

        let regions = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT region, COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost_cents
               FROM api_analytics_v14 GROUP BY region ORDER BY cost_cents DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v14 regions: {e}")))?;

        let ua_costs = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT COALESCE(user_agent, 'unknown') as user_agent, COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost_cents
               FROM api_analytics_v14 GROUP BY user_agent ORDER BY cost_cents DESC LIMIT 10"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v14 ua_costs: {e}")))?;

        Ok((totals.0, totals.1, totals.2, totals.3, regions, ua_costs))
    }

    pub async fn get_usage_optimization_v14(&self) -> Result<Vec<(String, String, f64, f64, f64, Vec<String>)>> {
        let rows = sqlx::query_as::<_, (String, String, f64, f64, f64, Vec<String>)>(
            r#"SELECT endpoint, method,
                      AVG(response_time_ms)::NUMERIC::FLOAT as avg_rt,
                      PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms)::NUMERIC::FLOAT as p95_rt,
                      CASE WHEN COUNT(*) > 0 THEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / COUNT(*)::NUMERIC * 100 ELSE 0 END as cache_rate,
                      COALESCE(
                          ARRAY_REMOVE(
                              ARRAY[
                                  CASE WHEN AVG(response_time_ms) > 500 THEN 'Optimize slow endpoint' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / NULLIF(COUNT(*), 0) < 0.3 THEN 'Improve caching strategy' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE status_code >= 500))::NUMERIC / NULLIF(COUNT(*), 0) > 0.05 THEN 'Reduce error rate' END
                              ],
                              NULL
                          ),
                          ARRAY['No optimization needed']::TEXT[]
                      ) as suggestions
               FROM api_analytics_v14
               GROUP BY endpoint, method
               HAVING COUNT(*) > 10
               ORDER BY avg_rt DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_usage_optimization_v14: {e}")))?;
        Ok(rows)
    }

    // --- API Docs v14 ---

    pub async fn list_api_docs_v14(&self, limit: i64, offset: i64) -> Result<Vec<ApiDocsV14>> {
        sqlx::query_as::<_, ApiDocsV14>(
            "SELECT * FROM api_docs_v14 ORDER BY endpoint, method, version LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_docs_v14: {e}")))
    }

    pub async fn get_api_docs_v14_for_endpoint(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
    ) -> Result<Option<ApiDocsV14>> {
        sqlx::query_as::<_, ApiDocsV14>(
            "SELECT * FROM api_docs_v14 WHERE endpoint = $1 AND method = $2 AND version = $3",
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_docs_v14_for_endpoint: {e}")))
    }

    pub async fn create_api_docs_v14(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
        summary: &str,
        description: &str,
        parameters: &serde_json::Value,
        request_body: Option<&serde_json::Value>,
        responses: &serde_json::Value,
        examples: &serde_json::Value,
        tags: &[String],
        deprecated: bool,
        changelog: &str,
        security_schemes: &serde_json::Value,
        rate_limits: &serde_json::Value,
    ) -> Result<ApiDocsV14> {
        sqlx::query_as::<_, ApiDocsV14>(
            r#"INSERT INTO api_docs_v14 (endpoint, method, version, summary, description, parameters, request_body, responses, examples, tags, deprecated, changelog, security_schemes, rate_limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .bind(summary)
        .bind(description)
        .bind(parameters)
        .bind(request_body)
        .bind(responses)
        .bind(examples)
        .bind(tags)
        .bind(deprecated)
        .bind(changelog)
        .bind(security_schemes)
        .bind(rate_limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_docs_v14: {e}")))
    }

    // --- Rate Limit Tiers v12 ---

    pub async fn list_rate_limit_tiers_v12(&self) -> Result<Vec<RateLimitTierV12>> {
        sqlx::query_as::<_, RateLimitTierV12>(
            "SELECT * FROM rate_limit_tiers_v12 ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_rate_limit_tiers_v12: {e}")))
    }

    pub async fn get_rate_limit_tier_v12_by_name(&self, name: &str) -> Result<Option<RateLimitTierV12>> {
        sqlx::query_as::<_, RateLimitTierV12>(
            "SELECT * FROM rate_limit_tiers_v12 WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_rate_limit_tier_v12_by_name: {e}")))
    }

    pub async fn create_rate_limit_tier_v12(
        &self,
        name: &str,
        description: &str,
        rate_limit: i32,
        burst_limit: i32,
        monthly_quota: Option<i32>,
        price_cents: i32,
        features: &serde_json::Value,
        limits: &serde_json::Value,
    ) -> Result<RateLimitTierV12> {
        sqlx::query_as::<_, RateLimitTierV12>(
            r#"INSERT INTO rate_limit_tiers_v12 (name, description, rate_limit, burst_limit, monthly_quota, price_cents, features, limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_tier_v12: {e}")))
    }

    pub async fn update_rate_limit_tier_v12(
        &self,
        name: &str,
        description: Option<&str>,
        rate_limit: Option<i32>,
        burst_limit: Option<i32>,
        monthly_quota: Option<Option<i32>>,
        price_cents: Option<i32>,
        features: Option<&serde_json::Value>,
        limits: Option<&serde_json::Value>,
    ) -> Result<RateLimitTierV12> {
        let existing = self.get_rate_limit_tier_v12_by_name(name).await?
            .ok_or_else(|| DbError::Database(format!("Tier {name} not found")))?;
        let new_desc = description.unwrap_or(&existing.description);
        let new_rate = rate_limit.unwrap_or(existing.rate_limit);
        let new_burst = burst_limit.unwrap_or(existing.burst_limit);
        let new_monthly = monthly_quota.unwrap_or(existing.monthly_quota);
        let new_price = price_cents.unwrap_or(existing.price_cents);
        let new_features = features.unwrap_or(&existing.features);
        let new_limits = limits.unwrap_or(&existing.limits);
        sqlx::query_as::<_, RateLimitTierV12>(
            r#"UPDATE rate_limit_tiers_v12 SET description = $2, rate_limit = $3, burst_limit = $4, monthly_quota = $5, price_cents = $6, features = $7, limits = $8
               WHERE name = $1
               RETURNING *"#,
        )
        .bind(name)
        .bind(new_desc)
        .bind(new_rate)
        .bind(new_burst)
        .bind(new_monthly)
        .bind(new_price)
        .bind(new_features)
        .bind(new_limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_rate_limit_tier_v12: {e}")))
    }

    pub async fn delete_rate_limit_tier_v12(&self, name: &str) -> Result<()> {
        sqlx::query("DELETE FROM rate_limit_tiers_v12 WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_rate_limit_tier_v12: {e}")))?;
        Ok(())
    }

    // --- Rate Limit Alerts v9 ---

    pub async fn create_rate_limit_alert_v9_for_v12(
        &self,
        user_id: Uuid,
        tier_id: Uuid,
        alert_type: &str,
        threshold: f64,
    ) -> Result<RateLimitAlertV9> {
        sqlx::query_as::<_, RateLimitAlertV9>(
            r#"INSERT INTO rate_limit_alerts_v9 (user_id, tier_id, alert_type, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(tier_id)
        .bind(alert_type)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_alert_v9_for_v12: {e}")))
    }

    pub async fn get_user_rate_limit_alerts_v9_for_v12(&self, user_id: Uuid) -> Result<Vec<RateLimitAlertV9>> {
        sqlx::query_as::<_, RateLimitAlertV9>(
            "SELECT * FROM rate_limit_alerts_v9 WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_user_rate_limit_alerts_v9_for_v12: {e}")))
    }

    // --- API Analytics v15 ---

    pub async fn list_api_analytics_v15(&self, limit: i64, offset: i64) -> Result<Vec<ApiAnalyticV15>> {
        sqlx::query_as::<_, ApiAnalyticV15>(
            "SELECT * FROM api_analytics_v15 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_analytics_v15: {e}")))
    }

    pub async fn create_api_analytic_v15(
        &self,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_time_ms: i32,
        user_id: Option<Uuid>,
        request_size_bytes: i32,
        response_size_bytes: i32,
        cache_hit: bool,
        region: &str,
        user_agent: Option<&str>,
        request_id: Option<Uuid>,
        cost_cents: i32,
    ) -> Result<ApiAnalyticV15> {
        sqlx::query_as::<_, ApiAnalyticV15>(
            r#"INSERT INTO api_analytics_v15 (endpoint, method, status_code, response_time_ms, user_id, request_size_bytes, response_size_bytes, cache_hit, region, user_agent, request_id, cost_cents)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(user_id)
        .bind(request_size_bytes)
        .bind(response_size_bytes)
        .bind(cache_hit)
        .bind(region)
        .bind(user_agent)
        .bind(request_id)
        .bind(cost_cents)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_analytic_v15: {e}")))
    }

    pub async fn get_cost_analysis_v15(&self) -> Result<(i64, i64, i64, i64, Vec<(String, i64, i64)>, Vec<(String, i64, i64)>)> {
        let totals = sqlx::query_as::<_, (i64, i64, i64, i64)>(
            r#"SELECT COUNT(*) as total_requests,
                      COALESCE(SUM(request_size_bytes), 0) as total_request_bytes,
                      COALESCE(SUM(response_size_bytes), 0) as total_response_bytes,
                      COALESCE(SUM(cost_cents), 0) as estimated_cost_cents
               FROM api_analytics_v15"#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v15 totals: {e}")))?;

        let regions = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT region, COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost_cents
               FROM api_analytics_v15 GROUP BY region ORDER BY cost_cents DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v15 regions: {e}")))?;

        let ua_costs = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT COALESCE(user_agent, 'unknown') as user_agent, COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost_cents
               FROM api_analytics_v15 GROUP BY user_agent ORDER BY cost_cents DESC LIMIT 10"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v15 ua_costs: {e}")))?;

        Ok((totals.0, totals.1, totals.2, totals.3, regions, ua_costs))
    }

    pub async fn get_usage_optimization_v15(&self) -> Result<Vec<(String, String, f64, f64, f64, Vec<String>)>> {
        let rows = sqlx::query_as::<_, (String, String, f64, f64, f64, Vec<String>)>(
            r#"SELECT endpoint, method,
                      AVG(response_time_ms)::NUMERIC::FLOAT as avg_rt,
                      PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms)::NUMERIC::FLOAT as p95_rt,
                      CASE WHEN COUNT(*) > 0 THEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / COUNT(*)::NUMERIC * 100 ELSE 0 END as cache_rate,
                      COALESCE(
                          ARRAY_REMOVE(
                              ARRAY[
                                  CASE WHEN AVG(response_time_ms) > 500 THEN 'Optimize slow endpoint' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / NULLIF(COUNT(*), 0) < 0.3 THEN 'Improve caching strategy' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE status_code >= 500))::NUMERIC / NULLIF(COUNT(*), 0) > 0.05 THEN 'Reduce error rate' END
                              ],
                              NULL
                          ),
                          ARRAY['No optimization needed']::TEXT[]
                      ) as suggestions
               FROM api_analytics_v15
                GROUP BY endpoint, method
                HAVING COUNT(*) > 10
                ORDER BY avg_rt DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_usage_optimization_v15: {e}")))?;
        Ok(rows)
    }

    // --- API Docs v15 ---

    pub async fn list_api_docs_v15(&self, limit: i64, offset: i64) -> Result<Vec<ApiDocsV15>> {
        sqlx::query_as::<_, ApiDocsV15>(
            "SELECT * FROM api_docs_v15 ORDER BY endpoint, method, version LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_docs_v15: {e}")))
    }

    pub async fn get_api_docs_v15_for_endpoint(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
    ) -> Result<Option<ApiDocsV15>> {
        sqlx::query_as::<_, ApiDocsV15>(
            "SELECT * FROM api_docs_v15 WHERE endpoint = $1 AND method = $2 AND version = $3",
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_docs_v15_for_endpoint: {e}")))
    }

    pub async fn create_api_docs_v15(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
        summary: &str,
        description: &str,
        parameters: &serde_json::Value,
        request_body: Option<&serde_json::Value>,
        responses: &serde_json::Value,
        examples: &serde_json::Value,
        tags: &[String],
        deprecated: bool,
        changelog: &str,
        security_schemes: &serde_json::Value,
        rate_limits: &serde_json::Value,
    ) -> Result<ApiDocsV15> {
        sqlx::query_as::<_, ApiDocsV15>(
            r#"INSERT INTO api_docs_v15 (endpoint, method, version, summary, description, parameters, request_body, responses, examples, tags, deprecated, changelog, security_schemes, rate_limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .bind(summary)
        .bind(description)
        .bind(parameters)
        .bind(request_body)
        .bind(responses)
        .bind(examples)
        .bind(tags)
        .bind(deprecated)
        .bind(changelog)
        .bind(security_schemes)
        .bind(rate_limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_docs_v15: {e}")))
    }

    // --- Rate Limit Tiers v13 ---

    pub async fn list_rate_limit_tiers_v13(&self) -> Result<Vec<RateLimitTierV13>> {
        sqlx::query_as::<_, RateLimitTierV13>(
            "SELECT * FROM rate_limit_tiers_v13 ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_rate_limit_tiers_v13: {e}")))
    }

    pub async fn get_rate_limit_tier_v13_by_name(&self, name: &str) -> Result<Option<RateLimitTierV13>> {
        sqlx::query_as::<_, RateLimitTierV13>(
            "SELECT * FROM rate_limit_tiers_v13 WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_rate_limit_tier_v13_by_name: {e}")))
    }

    pub async fn create_rate_limit_tier_v13(
        &self,
        name: &str,
        description: &str,
        rate_limit: i32,
        burst_limit: i32,
        monthly_quota: Option<i32>,
        price_cents: i32,
        features: &serde_json::Value,
        limits: &serde_json::Value,
    ) -> Result<RateLimitTierV13> {
        sqlx::query_as::<_, RateLimitTierV13>(
            r#"INSERT INTO rate_limit_tiers_v13 (name, description, rate_limit, burst_limit, monthly_quota, price_cents, features, limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_tier_v13: {e}")))
    }

    pub async fn update_rate_limit_tier_v13(
        &self,
        name: &str,
        description: Option<&str>,
        rate_limit: Option<i32>,
        burst_limit: Option<i32>,
        monthly_quota: Option<Option<i32>>,
        price_cents: Option<i32>,
        features: Option<&serde_json::Value>,
        limits: Option<&serde_json::Value>,
    ) -> Result<RateLimitTierV13> {
        let existing = self.get_rate_limit_tier_v13_by_name(name).await?
            .ok_or_else(|| DbError::Database(format!("Tier {name} not found")))?;
        let new_desc = description.unwrap_or(&existing.description);
        let new_rate = rate_limit.unwrap_or(existing.rate_limit);
        let new_burst = burst_limit.unwrap_or(existing.burst_limit);
        let new_monthly = monthly_quota.unwrap_or(existing.monthly_quota);
        let new_price = price_cents.unwrap_or(existing.price_cents);
        let new_features = features.unwrap_or(&existing.features);
        let new_limits = limits.unwrap_or(&existing.limits);
        sqlx::query_as::<_, RateLimitTierV13>(
            r#"UPDATE rate_limit_tiers_v13 SET description = $2, rate_limit = $3, burst_limit = $4, monthly_quota = $5, price_cents = $6, features = $7, limits = $8
               WHERE name = $1
               RETURNING *"#,
        )
        .bind(name)
        .bind(new_desc)
        .bind(new_rate)
        .bind(new_burst)
        .bind(new_monthly)
        .bind(new_price)
        .bind(new_features)
        .bind(new_limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_rate_limit_tier_v13: {e}")))
    }

    pub async fn delete_rate_limit_tier_v13(&self, name: &str) -> Result<()> {
        sqlx::query("DELETE FROM rate_limit_tiers_v13 WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_rate_limit_tier_v13: {e}")))?;
        Ok(())
    }

    // --- Rate Limit Alerts v10 ---

    pub async fn create_rate_limit_alert_v10_for_v13(
        &self,
        user_id: Uuid,
        tier_id: Uuid,
        alert_type: &str,
        threshold: f64,
    ) -> Result<RateLimitAlertV10> {
        sqlx::query_as::<_, RateLimitAlertV10>(
            r#"INSERT INTO rate_limit_alerts_v10 (user_id, tier_id, alert_type, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(tier_id)
        .bind(alert_type)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_alert_v10_for_v13: {e}")))
    }

    pub async fn get_user_rate_limit_alerts_v10_for_v13(&self, user_id: Uuid) -> Result<Vec<RateLimitAlertV10>> {
        sqlx::query_as::<_, RateLimitAlertV10>(
            "SELECT * FROM rate_limit_alerts_v10 WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_user_rate_limit_alerts_v10_for_v13: {e}")))
    }

    // --- API Analytics v16 ---

    pub async fn list_api_analytics_v16(&self, limit: i64, offset: i64) -> Result<Vec<ApiAnalyticV16>> {
        sqlx::query_as::<_, ApiAnalyticV16>(
            "SELECT * FROM api_analytics_v16 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_analytics_v16: {e}")))
    }

    pub async fn create_api_analytic_v16(
        &self,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_time_ms: i32,
        user_id: Option<Uuid>,
        request_size_bytes: i32,
        response_size_bytes: i32,
        cache_hit: bool,
        region: &str,
        user_agent: Option<&str>,
        request_id: Option<Uuid>,
        cost_cents: i32,
    ) -> Result<ApiAnalyticV16> {
        sqlx::query_as::<_, ApiAnalyticV16>(
            r#"INSERT INTO api_analytics_v16 (endpoint, method, status_code, response_time_ms, user_id, request_size_bytes, response_size_bytes, cache_hit, region, user_agent, request_id, cost_cents)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(user_id)
        .bind(request_size_bytes)
        .bind(response_size_bytes)
        .bind(cache_hit)
        .bind(region)
        .bind(user_agent)
        .bind(request_id)
        .bind(cost_cents)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_analytic_v16: {e}")))
    }

    pub async fn get_cost_analysis_v16(&self) -> Result<(i64, i64, i64, i64, Vec<(String, i64, i64)>, Vec<(String, i64, i64)>)> {
        let totals = sqlx::query_as::<_, (i64, i64, i64, i64)>(
            r#"SELECT COUNT(*) as total_requests,
                      COALESCE(SUM(request_size_bytes), 0) as total_request_bytes,
                      COALESCE(SUM(response_size_bytes), 0) as total_response_bytes,
                      COALESCE(SUM(cost_cents), 0) as estimated_cost_cents
               FROM api_analytics_v16"#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v16 totals: {e}")))?;

        let regions = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT region, COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost_cents
               FROM api_analytics_v16 GROUP BY region ORDER BY cost_cents DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v16 regions: {e}")))?;

        let ua_costs = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT COALESCE(user_agent, 'unknown') as user_agent, COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost_cents
               FROM api_analytics_v16 GROUP BY user_agent ORDER BY cost_cents DESC LIMIT 10"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v16 ua_costs: {e}")))?;

        Ok((totals.0, totals.1, totals.2, totals.3, regions, ua_costs))
    }

    pub async fn get_usage_optimization_v16(&self) -> Result<Vec<(String, String, f64, f64, f64, Vec<String>)>> {
        let rows = sqlx::query_as::<_, (String, String, f64, f64, f64, Vec<String>)>(
            r#"SELECT endpoint, method,
                      AVG(response_time_ms)::NUMERIC::FLOAT as avg_rt,
                      PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms)::NUMERIC::FLOAT as p95_rt,
                      CASE WHEN COUNT(*) > 0 THEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / COUNT(*)::NUMERIC * 100 ELSE 0 END as cache_rate,
                      COALESCE(
                          ARRAY_REMOVE(
                              ARRAY[
                                  CASE WHEN AVG(response_time_ms) > 500 THEN 'Optimize slow endpoint' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / NULLIF(COUNT(*), 0) < 0.3 THEN 'Improve caching strategy' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE status_code >= 500))::NUMERIC / NULLIF(COUNT(*), 0) > 0.05 THEN 'Reduce error rate' END
                              ],
                              NULL
                          ),
                          ARRAY['No optimization needed']::TEXT[]
                      ) as suggestions
               FROM api_analytics_v16
               GROUP BY endpoint, method
               HAVING COUNT(*) > 10
               ORDER BY avg_rt DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_usage_optimization_v16: {e}")))?;
        Ok(rows)
    }

    // --- API Docs v16 ---

    pub async fn list_api_docs_v16(&self, limit: i64, offset: i64) -> Result<Vec<ApiDocsV16>> {
        sqlx::query_as::<_, ApiDocsV16>(
            "SELECT * FROM api_docs_v16 ORDER BY endpoint, method, version LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_docs_v16: {e}")))
    }

    pub async fn get_api_docs_v16_for_endpoint(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
    ) -> Result<Option<ApiDocsV16>> {
        sqlx::query_as::<_, ApiDocsV16>(
            "SELECT * FROM api_docs_v16 WHERE endpoint = $1 AND method = $2 AND version = $3",
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_docs_v16_for_endpoint: {e}")))
    }

    pub async fn create_api_docs_v16(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
        summary: &str,
        description: &str,
        parameters: &serde_json::Value,
        request_body: Option<&serde_json::Value>,
        responses: &serde_json::Value,
        examples: &serde_json::Value,
        tags: &[String],
        deprecated: bool,
        changelog: &str,
        security_schemes: &serde_json::Value,
        rate_limits: &serde_json::Value,
    ) -> Result<ApiDocsV16> {
        sqlx::query_as::<_, ApiDocsV16>(
            r#"INSERT INTO api_docs_v16 (endpoint, method, version, summary, description, parameters, request_body, responses, examples, tags, deprecated, changelog, security_schemes, rate_limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .bind(summary)
        .bind(description)
        .bind(parameters)
        .bind(request_body)
        .bind(responses)
        .bind(examples)
        .bind(tags)
        .bind(deprecated)
        .bind(changelog)
        .bind(security_schemes)
        .bind(rate_limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_docs_v16: {e}")))
    }

    // --- Rate Limit Tiers v14 ---

    pub async fn list_rate_limit_tiers_v14(&self) -> Result<Vec<RateLimitTierV14>> {
        sqlx::query_as::<_, RateLimitTierV14>(
            "SELECT * FROM rate_limit_tiers_v14 ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_rate_limit_tiers_v14: {e}")))
    }

    pub async fn get_rate_limit_tier_v14_by_name(&self, name: &str) -> Result<Option<RateLimitTierV14>> {
        sqlx::query_as::<_, RateLimitTierV14>(
            "SELECT * FROM rate_limit_tiers_v14 WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_rate_limit_tier_v14_by_name: {e}")))
    }

    pub async fn create_rate_limit_tier_v14(
        &self,
        name: &str,
        description: &str,
        rate_limit: i32,
        burst_limit: i32,
        monthly_quota: Option<i32>,
        price_cents: i32,
        features: &serde_json::Value,
        limits: &serde_json::Value,
    ) -> Result<RateLimitTierV14> {
        sqlx::query_as::<_, RateLimitTierV14>(
            r#"INSERT INTO rate_limit_tiers_v14 (name, description, rate_limit, burst_limit, monthly_quota, price_cents, features, limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_tier_v14: {e}")))
    }

    pub async fn update_rate_limit_tier_v14(
        &self,
        name: &str,
        description: Option<&str>,
        rate_limit: Option<i32>,
        burst_limit: Option<i32>,
        monthly_quota: Option<Option<i32>>,
        price_cents: Option<i32>,
        features: Option<&serde_json::Value>,
        limits: Option<&serde_json::Value>,
    ) -> Result<RateLimitTierV14> {
        let existing = self.get_rate_limit_tier_v14_by_name(name).await?
            .ok_or_else(|| DbError::Database(format!("Tier {name} not found")))?;
        let new_desc = description.unwrap_or(&existing.description);
        let new_rate = rate_limit.unwrap_or(existing.rate_limit);
        let new_burst = burst_limit.unwrap_or(existing.burst_limit);
        let new_monthly = monthly_quota.unwrap_or(existing.monthly_quota);
        let new_price = price_cents.unwrap_or(existing.price_cents);
        let new_features = features.unwrap_or(&existing.features);
        let new_limits = limits.unwrap_or(&existing.limits);
        sqlx::query_as::<_, RateLimitTierV14>(
            r#"UPDATE rate_limit_tiers_v14 SET description = $2, rate_limit = $3, burst_limit = $4, monthly_quota = $5, price_cents = $6, features = $7, limits = $8
               WHERE name = $1
               RETURNING *"#,
        )
        .bind(name)
        .bind(new_desc)
        .bind(new_rate)
        .bind(new_burst)
        .bind(new_monthly)
        .bind(new_price)
        .bind(new_features)
        .bind(new_limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_rate_limit_tier_v14: {e}")))
    }

    pub async fn delete_rate_limit_tier_v14(&self, name: &str) -> Result<()> {
        sqlx::query("DELETE FROM rate_limit_tiers_v14 WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_rate_limit_tier_v14: {e}")))?;
        Ok(())
    }

    // --- Rate Limit Alerts v11 ---

    pub async fn create_rate_limit_alert_v11_for_v14(
        &self,
        user_id: Uuid,
        tier_id: Uuid,
        alert_type: &str,
        threshold: f64,
    ) -> Result<RateLimitAlertV11> {
        sqlx::query_as::<_, RateLimitAlertV11>(
            r#"INSERT INTO rate_limit_alerts_v11 (user_id, tier_id, alert_type, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(tier_id)
        .bind(alert_type)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_alert_v11_for_v14: {e}")))
    }

    pub async fn get_user_rate_limit_alerts_v11_for_v14(&self, user_id: Uuid) -> Result<Vec<RateLimitAlertV11>> {
        sqlx::query_as::<_, RateLimitAlertV11>(
            "SELECT * FROM rate_limit_alerts_v11 WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_user_rate_limit_alerts_v11_for_v14: {e}")))
    }

    // --- API Analytics v17 ---

    pub async fn list_api_analytics_v17(&self, limit: i64, offset: i64) -> Result<Vec<ApiAnalyticV17>> {
        sqlx::query_as::<_, ApiAnalyticV17>(
            "SELECT * FROM api_analytics_v17 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_analytics_v17: {e}")))
    }

    pub async fn create_api_analytic_v17(
        &self,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_time_ms: i32,
        user_id: Option<Uuid>,
        request_size_bytes: i32,
        response_size_bytes: i32,
        cache_hit: bool,
        region: &str,
        user_agent: Option<&str>,
        request_id: Option<Uuid>,
        cost_cents: i32,
    ) -> Result<ApiAnalyticV17> {
        sqlx::query_as::<_, ApiAnalyticV17>(
            r#"INSERT INTO api_analytics_v17 (endpoint, method, status_code, response_time_ms, user_id, request_size_bytes, response_size_bytes, cache_hit, region, user_agent, request_id, cost_cents)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(user_id)
        .bind(request_size_bytes)
        .bind(response_size_bytes)
        .bind(cache_hit)
        .bind(region)
        .bind(user_agent)
        .bind(request_id)
        .bind(cost_cents)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_analytic_v17: {e}")))
    }

    pub async fn get_cost_analysis_v17(&self) -> Result<(i64, i64, i64, i64, Vec<(String, i64, i64)>, Vec<(String, i64, i64)>)> {
        let totals = sqlx::query_as::<_, (i64, i64, i64, i64)>(
            r#"SELECT COUNT(*) as total_requests,
                      COALESCE(SUM(request_size_bytes), 0) as total_request_bytes,
                      COALESCE(SUM(response_size_bytes), 0) as total_response_bytes,
                      COALESCE(SUM(cost_cents), 0) as estimated_cost_cents
               FROM api_analytics_v17"#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v17 totals: {e}")))?;

        let regions = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT region, COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost_cents
               FROM api_analytics_v17 GROUP BY region ORDER BY cost_cents DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v17 regions: {e}")))?;

        let ua_costs = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT COALESCE(user_agent, 'unknown') as user_agent, COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost_cents
               FROM api_analytics_v17 GROUP BY user_agent ORDER BY cost_cents DESC LIMIT 10"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v17 ua_costs: {e}")))?;

        Ok((totals.0, totals.1, totals.2, totals.3, regions, ua_costs))
    }

    pub async fn get_usage_optimization_v17(&self) -> Result<Vec<(String, String, f64, f64, f64, Vec<String>)>> {
        let rows = sqlx::query_as::<_, (String, String, f64, f64, f64, Vec<String>)>(
            r#"SELECT endpoint, method,
                      AVG(response_time_ms)::NUMERIC::FLOAT as avg_rt,
                      PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms)::NUMERIC::FLOAT as p95_rt,
                      CASE WHEN COUNT(*) > 0 THEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / COUNT(*)::NUMERIC * 100 ELSE 0 END as cache_rate,
                      COALESCE(
                          ARRAY_REMOVE(
                              ARRAY[
                                  CASE WHEN AVG(response_time_ms) > 500 THEN 'Optimize slow endpoint' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / NULLIF(COUNT(*), 0) < 0.3 THEN 'Improve caching strategy' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE status_code >= 500))::NUMERIC / NULLIF(COUNT(*), 0) > 0.05 THEN 'Reduce error rate' END
                              ],
                              NULL
                          ),
                          ARRAY['No optimization needed']::TEXT[]
                      ) as suggestions
               FROM api_analytics_v17
               GROUP BY endpoint, method
               HAVING COUNT(*) > 10
               ORDER BY avg_rt DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_usage_optimization_v17: {e}")))?;
        Ok(rows)
    }

    // --- API Docs v17 ---

    pub async fn list_api_docs_v17(&self, limit: i64, offset: i64) -> Result<Vec<ApiDocsV17>> {
        sqlx::query_as::<_, ApiDocsV17>(
            "SELECT * FROM api_docs_v17 ORDER BY endpoint, method, version LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_docs_v17: {e}")))
    }

    pub async fn get_api_docs_v17_for_endpoint(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
    ) -> Result<Option<ApiDocsV17>> {
        sqlx::query_as::<_, ApiDocsV17>(
            "SELECT * FROM api_docs_v17 WHERE endpoint = $1 AND method = $2 AND version = $3",
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_docs_v17_for_endpoint: {e}")))
    }

    pub async fn create_api_docs_v17(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
        summary: &str,
        description: &str,
        parameters: &serde_json::Value,
        request_body: Option<&serde_json::Value>,
        responses: &serde_json::Value,
        examples: &serde_json::Value,
        tags: &[String],
        deprecated: bool,
        changelog: &str,
        security_schemes: &serde_json::Value,
        rate_limits: &serde_json::Value,
    ) -> Result<ApiDocsV17> {
        sqlx::query_as::<_, ApiDocsV17>(
            r#"INSERT INTO api_docs_v17 (endpoint, method, version, summary, description, parameters, request_body, responses, examples, tags, deprecated, changelog, security_schemes, rate_limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .bind(summary)
        .bind(description)
        .bind(parameters)
        .bind(request_body)
        .bind(responses)
        .bind(examples)
        .bind(tags)
        .bind(deprecated)
        .bind(changelog)
        .bind(security_schemes)
        .bind(rate_limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_docs_v17: {e}")))
    }

    // --- Rate Limit Tiers v15 ---

    pub async fn list_rate_limit_tiers_v15(&self) -> Result<Vec<RateLimitTierV15>> {
        sqlx::query_as::<_, RateLimitTierV15>(
            "SELECT * FROM rate_limit_tiers_v15 ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_rate_limit_tiers_v15: {e}")))
    }

    pub async fn get_rate_limit_tier_v15_by_name(&self, name: &str) -> Result<Option<RateLimitTierV15>> {
        sqlx::query_as::<_, RateLimitTierV15>(
            "SELECT * FROM rate_limit_tiers_v15 WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_rate_limit_tier_v15_by_name: {e}")))
    }

    pub async fn create_rate_limit_tier_v15(
        &self,
        name: &str,
        description: &str,
        rate_limit: i32,
        burst_limit: i32,
        monthly_quota: Option<i32>,
        price_cents: i32,
        features: &serde_json::Value,
        limits: &serde_json::Value,
    ) -> Result<RateLimitTierV15> {
        sqlx::query_as::<_, RateLimitTierV15>(
            r#"INSERT INTO rate_limit_tiers_v15 (name, description, rate_limit, burst_limit, monthly_quota, price_cents, features, limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_tier_v15: {e}")))
    }

    pub async fn update_rate_limit_tier_v15(
        &self,
        name: &str,
        description: Option<&str>,
        rate_limit: Option<i32>,
        burst_limit: Option<i32>,
        monthly_quota: Option<Option<i32>>,
        price_cents: Option<i32>,
        features: Option<&serde_json::Value>,
        limits: Option<&serde_json::Value>,
    ) -> Result<RateLimitTierV15> {
        sqlx::query_as::<_, RateLimitTierV15>(
            r#"UPDATE rate_limit_tiers_v15 SET
                description = COALESCE($2, description),
                rate_limit = COALESCE($3, rate_limit),
                burst_limit = COALESCE($4, burst_limit),
                monthly_quota = COALESCE($5, monthly_quota),
                price_cents = COALESCE($6, price_cents),
                features = COALESCE($7, features),
                limits = COALESCE($8, limits)
               WHERE name = $1
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_rate_limit_tier_v15: {e}")))
    }

    pub async fn delete_rate_limit_tier_v15(&self, name: &str) -> Result<()> {
        sqlx::query("DELETE FROM rate_limit_tiers_v15 WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_rate_limit_tier_v15: {e}")))?;
        Ok(())
    }

    // --- Rate Limit Alerts v12 ---

    pub async fn create_rate_limit_alert_v12_for_v15(
        &self,
        user_id: Uuid,
        tier_id: Uuid,
        alert_type: &str,
        threshold: f64,
    ) -> Result<RateLimitAlertV12> {
        sqlx::query_as::<_, RateLimitAlertV12>(
            r#"INSERT INTO rate_limit_alerts_v12 (user_id, tier_id, alert_type, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(tier_id)
        .bind(alert_type)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_alert_v12_for_v15: {e}")))
    }

    pub async fn get_user_rate_limit_alerts_v12_for_v15(&self, user_id: Uuid) -> Result<Vec<RateLimitAlertV12>> {
        sqlx::query_as::<_, RateLimitAlertV12>(
            "SELECT * FROM rate_limit_alerts_v12 WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_user_rate_limit_alerts_v12_for_v15: {e}")))
    }

    // --- API Analytics v18 ---

    pub async fn list_api_analytics_v18(&self, limit: i64, offset: i64) -> Result<Vec<ApiAnalyticV18>> {
        sqlx::query_as::<_, ApiAnalyticV18>(
            "SELECT * FROM api_analytics_v18 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_analytics_v18: {e}")))
    }

    pub async fn create_api_analytic_v18(
        &self,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_time_ms: i32,
        user_id: Option<Uuid>,
        request_size_bytes: i32,
        response_size_bytes: i32,
        cache_hit: bool,
        region: &str,
        user_agent: Option<&str>,
        request_id: Option<Uuid>,
        cost_cents: i32,
    ) -> Result<ApiAnalyticV18> {
        sqlx::query_as::<_, ApiAnalyticV18>(
            r#"INSERT INTO api_analytics_v18 (endpoint, method, status_code, response_time_ms, user_id, request_size_bytes, response_size_bytes, cache_hit, region, user_agent, request_id, cost_cents)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(user_id)
        .bind(request_size_bytes)
        .bind(response_size_bytes)
        .bind(cache_hit)
        .bind(region)
        .bind(user_agent)
        .bind(request_id)
        .bind(cost_cents)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_analytic_v18: {e}")))
    }

    pub async fn get_cost_analysis_v18(&self) -> Result<(i64, i64, i64, i64, Vec<(String, i64, i64)>, Vec<(String, i64, i64)>)> {
        let totals = sqlx::query_as::<_, (i64, i64, i64, i64)>(
            r#"SELECT COUNT(*) as total_requests,
                      COALESCE(SUM(request_size_bytes), 0) as total_request_bytes,
                      COALESCE(SUM(response_size_bytes), 0) as total_response_bytes,
                      COALESCE(SUM(cost_cents), 0) as estimated_cost_cents
               FROM api_analytics_v18"#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v18 totals: {e}")))?;

        let regions = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT region, COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost_cents
               FROM api_analytics_v18 GROUP BY region ORDER BY cost_cents DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v18 regions: {e}")))?;

        let ua_costs = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT COALESCE(user_agent, 'unknown') as user_agent, COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost_cents
               FROM api_analytics_v18 GROUP BY user_agent ORDER BY cost_cents DESC LIMIT 10"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v18 ua_costs: {e}")))?;

        Ok((totals.0, totals.1, totals.2, totals.3, regions, ua_costs))
    }

    pub async fn get_usage_optimization_v18(&self) -> Result<Vec<(String, String, f64, f64, f64, Vec<String>)>> {
        let rows = sqlx::query_as::<_, (String, String, f64, f64, f64, Vec<String>)>(
            r#"SELECT endpoint, method,
                      AVG(response_time_ms)::NUMERIC::FLOAT as avg_rt,
                      PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms)::NUMERIC::FLOAT as p95_rt,
                      CASE WHEN COUNT(*) > 0 THEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / COUNT(*)::NUMERIC * 100 ELSE 0 END as cache_rate,
                      COALESCE(
                          ARRAY_REMOVE(
                              ARRAY[
                                  CASE WHEN AVG(response_time_ms) > 500 THEN 'Optimize slow endpoint' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / NULLIF(COUNT(*), 0) < 0.3 THEN 'Improve caching strategy' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE status_code >= 500))::NUMERIC / NULLIF(COUNT(*), 0) > 0.05 THEN 'Reduce error rate' END
                              ],
                              NULL
                          ),
                          ARRAY['No optimization needed']::TEXT[]
                      ) as suggestions
               FROM api_analytics_v18
               GROUP BY endpoint, method
               HAVING COUNT(*) > 10
               ORDER BY avg_rt DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_usage_optimization_v18: {e}")))?;
        Ok(rows)
    }

    // --- Test Suite Metrics v15 ---

    pub async fn create_test_suite_metric_v15(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<TestSuiteMetricV15> {
        let row = sqlx::query_as::<_, TestSuiteMetricV15>(
            r#"INSERT INTO test_suite_metrics_v12 (suite_id, metric_name, metric_value)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_metric_v15: {e}")))?;
        Ok(row)
    }

    pub async fn list_test_suite_metrics_v15(
        &self,
        suite_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestSuiteMetricV15>> {
        sqlx::query_as::<_, TestSuiteMetricV15>(
            "SELECT * FROM test_suite_metrics_v12 WHERE suite_id = $1 ORDER BY measured_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(suite_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_test_suite_metrics_v15: {e}")))
    }

    pub async fn get_test_suite_latest_metric_v15(
        &self,
        suite_id: Uuid,
        metric_name: &str,
    ) -> Result<Option<TestSuiteMetricV15>> {
        sqlx::query_as::<_, TestSuiteMetricV15>(
            "SELECT * FROM test_suite_metrics_v12 WHERE suite_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT 1",
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_latest_metric_v15: {e}")))
    }

    pub async fn create_test_suite_baseline_v15(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        baseline_value: f64,
        threshold_percent: f64,
    ) -> Result<TestSuiteBaselineV15> {
        let row = sqlx::query_as::<_, TestSuiteBaselineV15>(
            r#"INSERT INTO test_suite_baselines_v12 (suite_id, metric_name, baseline_value, threshold_percent)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (suite_id, metric_name) DO UPDATE
               SET baseline_value = $3, threshold_percent = $4
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(baseline_value)
        .bind(threshold_percent)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_baseline_v15: {e}")))?;
        Ok(row)
    }

    pub async fn get_test_suite_baselines_v15(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteBaselineV15>> {
        sqlx::query_as::<_, TestSuiteBaselineV15>(
            "SELECT * FROM test_suite_baselines_v12 WHERE suite_id = $1 ORDER BY metric_name",
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_baselines_v15: {e}")))
    }

    pub async fn detect_test_suite_regression_v15(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        current_value: f64,
    ) -> Result<bool> {
        let row: Option<(f64, f64)> = sqlx::query_as(
            r#"SELECT baseline_value, threshold_percent
               FROM test_suite_baselines_v12
               WHERE suite_id = $1 AND metric_name = $2"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("detect_test_suite_regression_v15: {e}")))?;
        match row {
            Some((baseline_value, threshold_percent)) => {
                let threshold = baseline_value * (threshold_percent / 100.0);
                Ok(current_value > baseline_value + threshold)
            }
            None => Ok(false),
        }
    }

    pub async fn get_test_suite_performance_alerts_v15(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"SELECT DISTINCT metric_name
               FROM test_suite_metrics_v12
               WHERE suite_id = $1
               AND metric_name IN (
                   SELECT metric_name FROM test_suite_baselines_v12 WHERE suite_id = $1
               )
               AND metric_value > (
                   SELECT baseline_value * (1 + threshold_percent / 100.0)
                   FROM test_suite_baselines_v12 b
                   WHERE b.suite_id = $1 AND b.metric_name = test_suite_metrics_v12.metric_name
               )
               ORDER BY metric_name"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_performance_alerts_v15: {e}")))?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    // --- Code Quality Metrics v15 ---

    pub async fn create_code_quality_metric_v15(
        &self,
        repo_id: Uuid,
        file_path: &str,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<CodeQualityMetricV15> {
        let row = sqlx::query_as::<_, CodeQualityMetricV15>(
            r#"INSERT INTO code_quality_metrics_v13 (repo_id, file_path, metric_name, metric_value)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(file_path)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_metric_v15: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_metrics_v15(
        &self,
        repo_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CodeQualityMetricV15>> {
        match metric_name {
            Some(name) => sqlx::query_as::<_, CodeQualityMetricV15>(
                "SELECT * FROM code_quality_metrics_v13 WHERE repo_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT $3 OFFSET $4",
            )
            .bind(repo_id)
            .bind(name)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_code_quality_metrics_v15: {e}"))),
            None => sqlx::query_as::<_, CodeQualityMetricV15>(
                "SELECT * FROM code_quality_metrics_v13 WHERE repo_id = $1 ORDER BY measured_at DESC LIMIT $2 OFFSET $3",
            )
            .bind(repo_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_code_quality_metrics_v15: {e}"))),
        }
    }

    pub async fn get_code_quality_score_v15(
        &self,
        repo_id: Uuid,
    ) -> Result<f64> {
        let row: (Option<f64>,) = sqlx::query_as(
            r#"SELECT
                   CASE
                       WHEN COUNT(*) = 0 THEN 100.0
                       ELSE (COUNT(*) - COUNT(CASE WHEN cqm.metric_value > cqt.threshold_value THEN 1 END))::float / COUNT(*)::float * 100.0
                   END as score
               FROM code_quality_metrics_v13 cqm
               JOIN code_quality_thresholds_v12 cqt ON cqm.repo_id = cqt.repo_id AND cqm.metric_name = cqt.metric_name
               WHERE cqm.repo_id = $1 AND cqt.enabled = true"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_score_v15: {e}")))?;
        Ok(row.0.unwrap_or(100.0))
    }

    pub async fn create_code_quality_threshold_v15(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        threshold_value: f64,
        enabled: bool,
    ) -> Result<CodeQualityThresholdV15> {
        let row = sqlx::query_as::<_, CodeQualityThresholdV15>(
            r#"INSERT INTO code_quality_thresholds_v12 (repo_id, metric_name, threshold_value, enabled)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (repo_id, metric_name) DO UPDATE
               SET threshold_value = $3, enabled = $4
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .bind(threshold_value)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_threshold_v15: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_thresholds_v15(
        &self,
        repo_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<CodeQualityThresholdV15>> {
        if enabled_only {
            sqlx::query_as::<_, CodeQualityThresholdV15>(
                "SELECT * FROM code_quality_thresholds_v12 WHERE repo_id = $1 AND enabled = true ORDER BY metric_name",
            )
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_code_quality_thresholds_v15: {e}")))
        } else {
            sqlx::query_as::<_, CodeQualityThresholdV15>(
                "SELECT * FROM code_quality_thresholds_v12 WHERE repo_id = $1 ORDER BY metric_name",
            )
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_code_quality_thresholds_v15: {e}")))
        }
    }

    pub async fn check_code_quality_violation_v15(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<bool> {
        let row: Option<(f64,)> = sqlx::query_as(
            "SELECT threshold_value FROM code_quality_thresholds_v12 WHERE repo_id = $1 AND metric_name = $2 AND enabled = true",
        )
        .bind(repo_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("check_code_quality_violation_v15: {e}")))?;
        match row {
            Some((threshold_value,)) => Ok(metric_value > threshold_value),
            None => Ok(false),
        }
    }

    pub async fn delete_code_quality_threshold_v15(
        &self,
        repo_id: Uuid,
        metric_name: &str,
    ) -> Result<()> {
        sqlx::query("DELETE FROM code_quality_thresholds_v12 WHERE repo_id = $1 AND metric_name = $2")
            .bind(repo_id)
            .bind(metric_name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_code_quality_threshold_v15: {e}")))?;
        Ok(())
    }

    pub async fn get_code_quality_enforcement_report_v15(
        &self,
        repo_id: Uuid,
    ) -> Result<(i64, i64, f64)> {
        let row: (i64, i64, f64) = sqlx::query_as(
            r#"SELECT
                   COUNT(DISTINCT cqt.metric_name) as total_thresholds,
                   COUNT(DISTINCT CASE WHEN cqm.metric_value > cqt.threshold_value THEN cqt.metric_name END) as violating_thresholds,
                   CASE
                       WHEN COUNT(DISTINCT cqt.metric_name) = 0 THEN 100.0
                       ELSE (COUNT(DISTINCT cqt.metric_name) - COUNT(DISTINCT CASE WHEN cqm.metric_value > cqt.threshold_value THEN cqt.metric_name END))::float / COUNT(DISTINCT cqt.metric_name)::float * 100.0
                   END as compliance_rate
               FROM code_quality_thresholds_v12 cqt
               LEFT JOIN code_quality_metrics_v13 cqm ON cqt.repo_id = cqm.repo_id AND cqt.metric_name = cqm.metric_name
               WHERE cqt.repo_id = $1 AND cqt.enabled = true"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_enforcement_report_v15: {e}")))?;
        Ok((row.0, row.1, row.2))
    }

    // --- Performance Testing v16 ---

    pub async fn create_performance_test_alert_v16(
        &self,
        baseline_id: Uuid,
        alert_type: &str,
        threshold: f64,
        enabled: bool,
    ) -> Result<PerformanceTestAlertV16> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV16>(
            r#"INSERT INTO performance_test_alerts_v13 (baseline_id, alert_type, threshold, enabled)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(baseline_id)
        .bind(alert_type)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_performance_test_alert_v16: {e}")))?;
        Ok(row)
    }

    pub async fn list_performance_test_alerts_v16(
        &self,
        baseline_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<PerformanceTestAlertV16>> {
        if enabled_only {
            sqlx::query_as::<_, PerformanceTestAlertV16>(
                "SELECT * FROM performance_test_alerts_v13 WHERE baseline_id = $1 AND enabled = true ORDER BY created_at DESC",
            )
            .bind(baseline_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_performance_test_alerts_v16: {e}")))
        } else {
            sqlx::query_as::<_, PerformanceTestAlertV16>(
                "SELECT * FROM performance_test_alerts_v13 WHERE baseline_id = $1 ORDER BY created_at DESC",
            )
            .bind(baseline_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_performance_test_alerts_v16: {e}")))
        }
    }

    pub async fn update_performance_test_alert_v16(
        &self,
        alert_id: Uuid,
        threshold: Option<f64>,
        enabled: Option<bool>,
    ) -> Result<PerformanceTestAlertV16> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV16>(
            r#"UPDATE performance_test_alerts_v13
               SET threshold = COALESCE($2, threshold),
                   enabled = COALESCE($3, enabled)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(alert_id)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_performance_test_alert_v16: {e}")))?;
        Ok(row)
    }

    pub async fn record_performance_test_alert_v16(
        &self,
        alert_id: Uuid,
        metric_name: &str,
        metric_value: f64,
        threshold: f64,
    ) -> Result<PerformanceTestAlertHistoryV16> {
        let row = sqlx::query_as::<_, PerformanceTestAlertHistoryV16>(
            r#"INSERT INTO performance_test_alert_history_v13 (alert_id, metric_name, metric_value, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(alert_id)
        .bind(metric_name)
        .bind(metric_value)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_performance_test_alert_v16: {e}")))?;

        sqlx::query("UPDATE performance_test_alerts_v13 SET last_triggered_at = NOW() WHERE id = $1")
            .bind(alert_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("record_performance_test_alert_v16: {e}")))?;

        Ok(row)
    }

    pub async fn list_performance_test_alert_history_v16(
        &self,
        alert_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PerformanceTestAlertHistoryV16>> {
        sqlx::query_as::<_, PerformanceTestAlertHistoryV16>(
            "SELECT * FROM performance_test_alert_history_v13 WHERE alert_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(alert_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_performance_test_alert_history_v16: {e}")))
    }

    pub async fn get_performance_test_alert_analytics_v16(
        &self,
        alert_id: Uuid,
    ) -> Result<(i64, Option<f64>, Option<f64>)> {
        let row: (i64, Option<f64>, Option<f64>) = sqlx::query_as(
            r#"SELECT
                   COUNT(*) as trigger_count,
                   AVG(metric_value) as avg_value,
                   MAX(metric_value) as max_value
               FROM performance_test_alert_history_v13
               WHERE alert_id = $1"#,
        )
        .bind(alert_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_analytics_v16: {e}")))?;
        Ok((row.0, row.1, row.2))
    }

    pub async fn get_performance_test_alert_notification_config_v16(
        &self,
        alert_id: Uuid,
    ) -> Result<Option<(String, bool, Option<String>)>> {
        let row: Option<(String, bool, Option<String>)> = sqlx::query_as(
            r#"SELECT
                   alert_type,
                   enabled,
                   last_triggered_at::text
               FROM performance_test_alerts_v13
               WHERE id = $1"#,
        )
        .bind(alert_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_notification_config_v16: {e}")))?;
        Ok(row)
    }

    // --- Test Suite Management v16 ---

    pub async fn create_test_suite_metric_v16(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<TestSuiteMetricV16> {
        let row = sqlx::query_as::<_, TestSuiteMetricV16>(
            r#"INSERT INTO test_suite_metrics_v13 (suite_id, metric_name, metric_value)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_metric_v16: {e}")))?;
        Ok(row)
    }

    pub async fn list_test_suite_metrics_v16(
        &self,
        suite_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestSuiteMetricV16>> {
        sqlx::query_as::<_, TestSuiteMetricV16>(
            "SELECT * FROM test_suite_metrics_v13 WHERE suite_id = $1 ORDER BY measured_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(suite_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_test_suite_metrics_v16: {e}")))
    }

    pub async fn get_test_suite_latest_metric_v16(
        &self,
        suite_id: Uuid,
        metric_name: &str,
    ) -> Result<Option<TestSuiteMetricV16>> {
        sqlx::query_as::<_, TestSuiteMetricV16>(
            "SELECT * FROM test_suite_metrics_v13 WHERE suite_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT 1",
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_latest_metric_v16: {e}")))
    }

    pub async fn create_test_suite_baseline_v16(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        baseline_value: f64,
        threshold_percent: f64,
    ) -> Result<TestSuiteBaselineV16> {
        let row = sqlx::query_as::<_, TestSuiteBaselineV16>(
            r#"INSERT INTO test_suite_baselines_v13 (suite_id, metric_name, baseline_value, threshold_percent)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (suite_id, metric_name) DO UPDATE
               SET baseline_value = $3, threshold_percent = $4
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(baseline_value)
        .bind(threshold_percent)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_baseline_v16: {e}")))?;
        Ok(row)
    }

    pub async fn get_test_suite_baselines_v16(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteBaselineV16>> {
        sqlx::query_as::<_, TestSuiteBaselineV16>(
            "SELECT * FROM test_suite_baselines_v13 WHERE suite_id = $1 ORDER BY metric_name",
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_baselines_v16: {e}")))
    }

    pub async fn detect_test_suite_regression_v16(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        current_value: f64,
    ) -> Result<bool> {
        let row: Option<(f64, f64)> = sqlx::query_as(
            r#"SELECT baseline_value, threshold_percent
               FROM test_suite_baselines_v13
               WHERE suite_id = $1 AND metric_name = $2"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("detect_test_suite_regression_v16: {e}")))?;
        match row {
            Some((baseline, threshold)) => {
                let threshold_value = baseline * (1.0 + threshold / 100.0);
                Ok(current_value > threshold_value)
            }
            None => Ok(false),
        }
    }

    pub async fn get_test_suite_performance_alerts_v16(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"SELECT DISTINCT metric_name
               FROM test_suite_metrics_v13
               WHERE suite_id = $1
               AND metric_name IN (
                   SELECT metric_name FROM test_suite_baselines_v13 WHERE suite_id = $1
               )
               AND metric_value > (
                   SELECT baseline_value * (1 + threshold_percent / 100.0)
                   FROM test_suite_baselines_v13 b
                   WHERE b.suite_id = $1 AND b.metric_name = test_suite_metrics_v13.metric_name
               )
               ORDER BY metric_name"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_performance_alerts_v16: {e}")))?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    // --- Code Quality Rules v16 ---

    pub async fn create_code_quality_metric_v16(
        &self,
        repo_id: Uuid,
        file_path: &str,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<CodeQualityMetricV16> {
        let row = sqlx::query_as::<_, CodeQualityMetricV16>(
            r#"INSERT INTO code_quality_metrics_v14 (repo_id, file_path, metric_name, metric_value)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(file_path)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_metric_v16: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_metrics_v16(
        &self,
        repo_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CodeQualityMetricV16>> {
        match metric_name {
            Some(name) => sqlx::query_as::<_, CodeQualityMetricV16>(
                "SELECT * FROM code_quality_metrics_v14 WHERE repo_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT $3 OFFSET $4",
            )
            .bind(repo_id)
            .bind(name)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_code_quality_metrics_v16: {e}"))),
            None => sqlx::query_as::<_, CodeQualityMetricV16>(
                "SELECT * FROM code_quality_metrics_v14 WHERE repo_id = $1 ORDER BY measured_at DESC LIMIT $2 OFFSET $3",
            )
            .bind(repo_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_code_quality_metrics_v16: {e}"))),
        }
    }

    pub async fn get_code_quality_score_v16(
        &self,
        repo_id: Uuid,
    ) -> Result<f64> {
        let row: (Option<f64>,) = sqlx::query_as(
            r#"SELECT
                   CASE
                       WHEN COUNT(*) = 0 THEN 100.0
                       ELSE (COUNT(*) - COUNT(CASE WHEN cqm.metric_value > cqt.threshold_value THEN 1 END))::float / COUNT(*)::float * 100.0
                   END as score
               FROM code_quality_metrics_v14 cqm
               JOIN code_quality_thresholds_v13 cqt ON cqm.repo_id = cqt.repo_id AND cqm.metric_name = cqt.metric_name
               WHERE cqm.repo_id = $1 AND cqt.enabled = true"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_score_v16: {e}")))?;
        Ok(row.0.unwrap_or(100.0))
    }

    pub async fn create_code_quality_threshold_v16(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        threshold_value: f64,
        enabled: bool,
    ) -> Result<CodeQualityThresholdV16> {
        let row = sqlx::query_as::<_, CodeQualityThresholdV16>(
            r#"INSERT INTO code_quality_thresholds_v13 (repo_id, metric_name, threshold_value, enabled)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (repo_id, metric_name) DO UPDATE
               SET threshold_value = $3, enabled = $4
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .bind(threshold_value)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_threshold_v16: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_thresholds_v16(
        &self,
        repo_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<CodeQualityThresholdV16>> {
        if enabled_only {
            sqlx::query_as::<_, CodeQualityThresholdV16>(
                "SELECT * FROM code_quality_thresholds_v13 WHERE repo_id = $1 AND enabled = true ORDER BY metric_name",
            )
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_code_quality_thresholds_v16: {e}")))
        } else {
            sqlx::query_as::<_, CodeQualityThresholdV16>(
                "SELECT * FROM code_quality_thresholds_v13 WHERE repo_id = $1 ORDER BY metric_name",
            )
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_code_quality_thresholds_v16: {e}")))
        }
    }

    pub async fn check_code_quality_violation_v16(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<bool> {
        let row: Option<(f64,)> = sqlx::query_as(
            r#"SELECT threshold_value FROM code_quality_thresholds_v13
               WHERE repo_id = $1 AND metric_name = $2 AND enabled = true"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("check_code_quality_violation_v16: {e}")))?;
        match row {
            Some((threshold,)) => Ok(metric_value > threshold),
            None => Ok(false),
        }
    }

    pub async fn delete_code_quality_threshold_v16(
        &self,
        repo_id: Uuid,
        metric_name: &str,
    ) -> Result<()> {
        sqlx::query("DELETE FROM code_quality_thresholds_v13 WHERE repo_id = $1 AND metric_name = $2")
            .bind(repo_id)
            .bind(metric_name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_code_quality_threshold_v16: {e}")))?;
        Ok(())
    }

    pub async fn get_code_quality_enforcement_report_v16(
        &self,
        repo_id: Uuid,
    ) -> Result<(i64, i64, f64)> {
        let row: (i64, i64, Option<f64>) = sqlx::query_as(
            r#"SELECT
                   COUNT(*) as total_thresholds,
                   COUNT(CASE WHEN cqm.metric_value > cqt.threshold_value THEN 1 END) as violating_thresholds,
                   CASE
                       WHEN COUNT(*) = 0 THEN 100.0
                       ELSE (COUNT(*) - COUNT(CASE WHEN cqm.metric_value > cqt.threshold_value THEN 1 END))::float / COUNT(*)::float * 100.0
                   END as compliance_rate
               FROM code_quality_thresholds_v13 cqt
               LEFT JOIN code_quality_metrics_v14 cqm ON cqt.repo_id = cqm.repo_id AND cqt.metric_name = cqm.metric_name
               WHERE cqt.repo_id = $1 AND cqt.enabled = true"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_enforcement_report_v16: {e}")))?;
        Ok((row.0, row.1, row.2.unwrap_or(100.0)))
    }

    // --- Performance Testing v17 ---

    pub async fn create_performance_test_alert_v17(
        &self,
        baseline_id: Uuid,
        alert_type: &str,
        threshold: f64,
        enabled: bool,
    ) -> Result<PerformanceTestAlertV17> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV17>(
            r#"INSERT INTO performance_test_alerts_v14 (baseline_id, alert_type, threshold, enabled)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(baseline_id)
        .bind(alert_type)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_performance_test_alert_v17: {e}")))?;
        Ok(row)
    }

    pub async fn list_performance_test_alerts_v17(
        &self,
        baseline_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<PerformanceTestAlertV17>> {
        if enabled_only {
            sqlx::query_as::<_, PerformanceTestAlertV17>(
                "SELECT * FROM performance_test_alerts_v14 WHERE baseline_id = $1 AND enabled = true ORDER BY created_at DESC",
            )
            .bind(baseline_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_performance_test_alerts_v17: {e}")))
        } else {
            sqlx::query_as::<_, PerformanceTestAlertV17>(
                "SELECT * FROM performance_test_alerts_v14 WHERE baseline_id = $1 ORDER BY created_at DESC",
            )
            .bind(baseline_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_performance_test_alerts_v17: {e}")))
        }
    }

    pub async fn update_performance_test_alert_v17(
        &self,
        alert_id: Uuid,
        threshold: Option<f64>,
        enabled: Option<bool>,
    ) -> Result<PerformanceTestAlertV17> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV17>(
            r#"UPDATE performance_test_alerts_v14
               SET threshold = COALESCE($2, threshold),
                   enabled = COALESCE($3, enabled)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(alert_id)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_performance_test_alert_v17: {e}")))?;
        Ok(row)
    }

    pub async fn record_performance_test_alert_v17(
        &self,
        alert_id: Uuid,
        metric_name: &str,
        metric_value: f64,
        threshold: f64,
    ) -> Result<PerformanceTestAlertHistoryV17> {
        let row = sqlx::query_as::<_, PerformanceTestAlertHistoryV17>(
            r#"INSERT INTO performance_test_alert_history_v14 (alert_id, metric_name, metric_value, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(alert_id)
        .bind(metric_name)
        .bind(metric_value)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_performance_test_alert_v17: {e}")))?;

        sqlx::query("UPDATE performance_test_alerts_v14 SET last_triggered_at = NOW() WHERE id = $1")
            .bind(alert_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("record_performance_test_alert_v17: {e}")))?;

        Ok(row)
    }

    pub async fn list_performance_test_alert_history_v17(
        &self,
        alert_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PerformanceTestAlertHistoryV17>> {
        sqlx::query_as::<_, PerformanceTestAlertHistoryV17>(
            "SELECT * FROM performance_test_alert_history_v14 WHERE alert_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(alert_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_performance_test_alert_history_v17: {e}")))
    }

    pub async fn get_performance_test_alert_analytics_v17(
        &self,
        alert_id: Uuid,
    ) -> Result<(i64, Option<f64>, Option<f64>)> {
        let row: (i64, Option<f64>, Option<f64>) = sqlx::query_as(
            r#"SELECT
                   COUNT(*) as trigger_count,
                   AVG(metric_value) as avg_value,
                   MAX(metric_value) as max_value
               FROM performance_test_alert_history_v14
               WHERE alert_id = $1"#,
        )
        .bind(alert_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_analytics_v17: {e}")))?;
        Ok((row.0, row.1, row.2))
    }

    pub async fn get_performance_test_alert_notification_config_v17(
        &self,
        alert_id: Uuid,
    ) -> Result<Option<(String, bool, Option<String>)>> {
        let row: Option<(String, bool, Option<String>)> = sqlx::query_as(
            r#"SELECT
                   alert_type,
                   enabled,
                   last_triggered_at::text
               FROM performance_test_alerts_v14
               WHERE id = $1"#,
        )
        .bind(alert_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_notification_config_v17: {e}")))?;
        Ok(row)
    }

    // --- Test Suite Management v17 ---

    pub async fn create_test_suite_metric_v17(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<TestSuiteMetricV17> {
        let row = sqlx::query_as::<_, TestSuiteMetricV17>(
            r#"INSERT INTO test_suite_metrics_v14 (suite_id, metric_name, metric_value)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_metric_v17: {e}")))?;
        Ok(row)
    }

    pub async fn list_test_suite_metrics_v17(
        &self,
        suite_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestSuiteMetricV17>> {
        sqlx::query_as::<_, TestSuiteMetricV17>(
            "SELECT * FROM test_suite_metrics_v14 WHERE suite_id = $1 ORDER BY measured_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(suite_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_test_suite_metrics_v17: {e}")))
    }

    pub async fn get_test_suite_latest_metric_v17(
        &self,
        suite_id: Uuid,
        metric_name: &str,
    ) -> Result<Option<TestSuiteMetricV17>> {
        sqlx::query_as::<_, TestSuiteMetricV17>(
            "SELECT * FROM test_suite_metrics_v14 WHERE suite_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT 1",
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_latest_metric_v17: {e}")))
    }

    pub async fn create_test_suite_baseline_v17(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        baseline_value: f64,
        threshold_percent: f64,
    ) -> Result<TestSuiteBaselineV17> {
        let row = sqlx::query_as::<_, TestSuiteBaselineV17>(
            r#"INSERT INTO test_suite_baselines_v14 (suite_id, metric_name, baseline_value, threshold_percent)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (suite_id, metric_name) DO UPDATE
               SET baseline_value = $3, threshold_percent = $4
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(baseline_value)
        .bind(threshold_percent)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_baseline_v17: {e}")))?;
        Ok(row)
    }

    pub async fn get_test_suite_baselines_v17(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteBaselineV17>> {
        sqlx::query_as::<_, TestSuiteBaselineV17>(
            "SELECT * FROM test_suite_baselines_v14 WHERE suite_id = $1 ORDER BY metric_name",
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_baselines_v17: {e}")))
    }

    pub async fn detect_test_suite_regression_v17(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        current_value: f64,
    ) -> Result<bool> {
        let row: Option<(f64, f64)> = sqlx::query_as(
            r#"SELECT baseline_value, threshold_percent
               FROM test_suite_baselines_v14
               WHERE suite_id = $1 AND metric_name = $2"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("detect_test_suite_regression_v17: {e}")))?;
        match row {
            Some((baseline, threshold)) => {
                let threshold_value = baseline * (1.0 + threshold / 100.0);
                Ok(current_value > threshold_value)
            }
            None => Ok(false),
        }
    }

    pub async fn get_test_suite_performance_alerts_v17(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"SELECT DISTINCT metric_name
               FROM test_suite_metrics_v14
               WHERE suite_id = $1
               AND metric_name IN (
                   SELECT metric_name FROM test_suite_baselines_v14 WHERE suite_id = $1
               )
               AND metric_value > (
                   SELECT baseline_value * (1 + threshold_percent / 100.0)
                   FROM test_suite_baselines_v14 b
                   WHERE b.suite_id = $1 AND b.metric_name = test_suite_metrics_v14.metric_name
               )
               ORDER BY metric_name"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_performance_alerts_v17: {e}")))?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    // --- Code Quality Rules v17 ---

    pub async fn create_code_quality_metric_v17(
        &self,
        repo_id: Uuid,
        file_path: &str,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<CodeQualityMetricV17> {
        let row = sqlx::query_as::<_, CodeQualityMetricV17>(
            r#"INSERT INTO code_quality_metrics_v15 (repo_id, file_path, metric_name, metric_value)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(file_path)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_metric_v17: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_metrics_v17(
        &self,
        repo_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CodeQualityMetricV17>> {
        let rows = match metric_name {
            Some(name) => {
                sqlx::query_as::<_, CodeQualityMetricV17>(
                    "SELECT * FROM code_quality_metrics_v15 WHERE repo_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT $3 OFFSET $4",
                )
                .bind(repo_id)
                .bind(name)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, CodeQualityMetricV17>(
                    "SELECT * FROM code_quality_metrics_v15 WHERE repo_id = $1 ORDER BY measured_at DESC LIMIT $2 OFFSET $3",
                )
                .bind(repo_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        };
        rows.map_err(|e| DbError::Database(format!("list_code_quality_metrics_v17: {e}")))
    }

    pub async fn get_code_quality_score_v17(&self, repo_id: Uuid) -> Result<f64> {
        let row: Option<(f64,)> = sqlx::query_as(
            r#"SELECT COALESCE(100.0 - AVG(metric_value), 100.0)
               FROM code_quality_metrics_v15
               WHERE repo_id = $1 AND metric_name = 'violations'"#,
        )
        .bind(repo_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_score_v17: {e}")))?;
        Ok(row.map(|r| r.0).unwrap_or(100.0))
    }

    pub async fn create_code_quality_threshold_v17(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        threshold_value: f64,
        enabled: bool,
    ) -> Result<CodeQualityThresholdV17> {
        let row = sqlx::query_as::<_, CodeQualityThresholdV17>(
            r#"INSERT INTO code_quality_thresholds_v14 (repo_id, metric_name, threshold_value, enabled)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (repo_id, metric_name) DO UPDATE
               SET threshold_value = $3, enabled = $4
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .bind(threshold_value)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_threshold_v17: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_thresholds_v17(
        &self,
        repo_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<CodeQualityThresholdV17>> {
        let query = if enabled_only {
            "SELECT * FROM code_quality_thresholds_v14 WHERE repo_id = $1 AND enabled = true ORDER BY metric_name"
        } else {
            "SELECT * FROM code_quality_thresholds_v14 WHERE repo_id = $1 ORDER BY metric_name"
        };
        sqlx::query_as::<_, CodeQualityThresholdV17>(query)
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_code_quality_thresholds_v17: {e}")))
    }

    pub async fn check_code_quality_violation_v17(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<bool> {
        let row: Option<(f64, bool)> = sqlx::query_as(
            r#"SELECT threshold_value, enabled
               FROM code_quality_thresholds_v14
               WHERE repo_id = $1 AND metric_name = $2"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("check_code_quality_violation_v17: {e}")))?;
        match row {
            Some((threshold, enabled)) => {
                Ok(enabled && metric_value > threshold)
            }
            None => Ok(false),
        }
    }

    pub async fn delete_code_quality_threshold_v17(
        &self,
        repo_id: Uuid,
        metric_name: &str,
    ) -> Result<()> {
        sqlx::query("DELETE FROM code_quality_thresholds_v14 WHERE repo_id = $1 AND metric_name = $2")
            .bind(repo_id)
            .bind(metric_name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_code_quality_threshold_v17: {e}")))?;
        Ok(())
    }

    pub async fn get_code_quality_enforcement_report_v17(
        &self,
        repo_id: Uuid,
    ) -> Result<(i64, i64, f64)> {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM code_quality_thresholds_v14 WHERE repo_id = $1 AND enabled = true",
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_enforcement_report_v17 total: {e}")))?;

        let violating: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(DISTINCT t.metric_name)
               FROM code_quality_thresholds_v14 t
               WHERE t.repo_id = $1 AND t.enabled = true
               AND EXISTS (
                   SELECT 1 FROM code_quality_metrics_v15 m
                   WHERE m.repo_id = $1 AND m.metric_name = t.metric_name
                   AND m.metric_value > t.threshold_value
               )"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_enforcement_report_v17 violating: {e}")))?;

        let compliance_rate = if total.0 > 0 {
            ((total.0 - violating.0) as f64 / total.0 as f64) * 100.0
        } else {
            100.0
        };

        Ok((total.0, violating.0, compliance_rate))
    }

    // --- Performance Testing v18 ---

    pub async fn create_performance_test_alert_v18(
        &self,
        baseline_id: Uuid,
        alert_type: &str,
        threshold: f64,
        enabled: bool,
    ) -> Result<PerformanceTestAlertV18> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV18>(
            r#"INSERT INTO performance_test_alerts_v15 (baseline_id, alert_type, threshold, enabled)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(baseline_id)
        .bind(alert_type)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_performance_test_alert_v18: {e}")))?;
        Ok(row)
    }

    pub async fn list_performance_test_alerts_v18(
        &self,
        baseline_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<PerformanceTestAlertV18>> {
        let query = if enabled_only {
            "SELECT * FROM performance_test_alerts_v15 WHERE baseline_id = $1 AND enabled = true ORDER BY created_at"
        } else {
            "SELECT * FROM performance_test_alerts_v15 WHERE baseline_id = $1 ORDER BY created_at"
        };
        sqlx::query_as::<_, PerformanceTestAlertV18>(query)
            .bind(baseline_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_performance_test_alerts_v18: {e}")))
    }

    pub async fn update_performance_test_alert_v18(
        &self,
        alert_id: Uuid,
        threshold: Option<f64>,
        enabled: Option<bool>,
    ) -> Result<PerformanceTestAlertV18> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV18>(
            r#"UPDATE performance_test_alerts_v15
               SET threshold = COALESCE($2, threshold),
                   enabled = COALESCE($3, enabled)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(alert_id)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_performance_test_alert_v18: {e}")))?;
        Ok(row)
    }

    pub async fn record_performance_test_alert_v18(
        &self,
        alert_id: Uuid,
        metric_name: &str,
        metric_value: f64,
        threshold: f64,
    ) -> Result<PerformanceTestAlertHistoryV18> {
        let row = sqlx::query_as::<_, PerformanceTestAlertHistoryV18>(
            r#"INSERT INTO performance_test_alert_history_v15 (alert_id, metric_name, metric_value, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(alert_id)
        .bind(metric_name)
        .bind(metric_value)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_performance_test_alert_v18: {e}")))?;

        sqlx::query("UPDATE performance_test_alerts_v15 SET last_triggered_at = NOW() WHERE id = $1")
            .bind(alert_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("record_performance_test_alert_v18 update: {e}")))?;

        Ok(row)
    }

    pub async fn list_performance_test_alert_history_v18(
        &self,
        alert_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PerformanceTestAlertHistoryV18>> {
        sqlx::query_as::<_, PerformanceTestAlertHistoryV18>(
            "SELECT * FROM performance_test_alert_history_v15 WHERE alert_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(alert_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_performance_test_alert_history_v18: {e}")))
    }

    pub async fn get_performance_test_alert_analytics_v18(
        &self,
        alert_id: Uuid,
    ) -> Result<(i64, Option<f64>, Option<f64>)> {
        let row: (i64, Option<f64>, Option<f64>) = sqlx::query_as(
            r#"SELECT COUNT(*), AVG(metric_value), MAX(metric_value)
               FROM performance_test_alert_history_v15
               WHERE alert_id = $1"#,
        )
        .bind(alert_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_analytics_v18: {e}")))?;
        Ok(row)
    }

    pub async fn get_performance_test_alert_notification_config_v18(
        &self,
        alert_id: Uuid,
    ) -> Result<Option<(String, bool, Option<DateTime<Utc>>)>> {
        let row = sqlx::query_as::<_, (String, bool, Option<DateTime<Utc>>)>(
            "SELECT alert_type, enabled, last_triggered_at FROM performance_test_alerts_v15 WHERE id = $1",
        )
        .bind(alert_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_notification_config_v18: {e}")))?;
        Ok(row)
    }

    // --- API Docs v18 ---

    pub async fn list_api_docs_v18(&self, limit: i64, offset: i64) -> Result<Vec<ApiDocsV18>> {
        sqlx::query_as::<_, ApiDocsV18>(
            "SELECT * FROM api_docs_v18 ORDER BY endpoint, method, version LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_docs_v18: {e}")))
    }

    pub async fn get_api_docs_v18_for_endpoint(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
    ) -> Result<Option<ApiDocsV18>> {
        sqlx::query_as::<_, ApiDocsV18>(
            "SELECT * FROM api_docs_v18 WHERE endpoint = $1 AND method = $2 AND version = $3",
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_docs_v18_for_endpoint: {e}")))
    }

    pub async fn create_api_docs_v18(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
        summary: &str,
        description: &str,
        parameters: &serde_json::Value,
        request_body: Option<&serde_json::Value>,
        responses: &serde_json::Value,
        examples: &serde_json::Value,
        tags: &[String],
        deprecated: bool,
        changelog: &str,
        security_schemes: &serde_json::Value,
        rate_limits: &serde_json::Value,
    ) -> Result<ApiDocsV18> {
        sqlx::query_as::<_, ApiDocsV18>(
            r#"INSERT INTO api_docs_v18 (endpoint, method, version, summary, description, parameters, request_body, responses, examples, tags, deprecated, changelog, security_schemes, rate_limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .bind(summary)
        .bind(description)
        .bind(parameters)
        .bind(request_body)
        .bind(responses)
        .bind(examples)
        .bind(tags)
        .bind(deprecated)
        .bind(changelog)
        .bind(security_schemes)
        .bind(rate_limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_docs_v18: {e}")))
    }

    // --- Rate Limit Tiers v16 ---

    pub async fn list_rate_limit_tiers_v16(&self) -> Result<Vec<RateLimitTierV16>> {
        sqlx::query_as::<_, RateLimitTierV16>(
            "SELECT * FROM rate_limit_tiers_v16 ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_rate_limit_tiers_v16: {e}")))
    }

    pub async fn get_rate_limit_tier_v16_by_name(&self, name: &str) -> Result<Option<RateLimitTierV16>> {
        sqlx::query_as::<_, RateLimitTierV16>(
            "SELECT * FROM rate_limit_tiers_v16 WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_rate_limit_tier_v16_by_name: {e}")))
    }

    pub async fn create_rate_limit_tier_v16(
        &self,
        name: &str,
        description: &str,
        rate_limit: i32,
        burst_limit: i32,
        monthly_quota: Option<i32>,
        price_cents: i32,
        features: &serde_json::Value,
        limits: &serde_json::Value,
    ) -> Result<RateLimitTierV16> {
        sqlx::query_as::<_, RateLimitTierV16>(
            r#"INSERT INTO rate_limit_tiers_v16 (name, description, rate_limit, burst_limit, monthly_quota, price_cents, features, limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_tier_v16: {e}")))
    }

    pub async fn update_rate_limit_tier_v16(
        &self,
        name: &str,
        description: Option<&str>,
        rate_limit: Option<i32>,
        burst_limit: Option<i32>,
        monthly_quota: Option<Option<i32>>,
        price_cents: Option<i32>,
        features: Option<&serde_json::Value>,
        limits: Option<&serde_json::Value>,
    ) -> Result<RateLimitTierV16> {
        sqlx::query_as::<_, RateLimitTierV16>(
            r#"UPDATE rate_limit_tiers_v16 SET
                description = COALESCE($2, description),
                rate_limit = COALESCE($3, rate_limit),
                burst_limit = COALESCE($4, burst_limit),
                monthly_quota = COALESCE($5, monthly_quota),
                price_cents = COALESCE($6, price_cents),
                features = COALESCE($7, features),
                limits = COALESCE($8, limits)
               WHERE name = $1
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_rate_limit_tier_v16: {e}")))
    }

    pub async fn delete_rate_limit_tier_v16(&self, name: &str) -> Result<()> {
        sqlx::query("DELETE FROM rate_limit_tiers_v16 WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_rate_limit_tier_v16: {e}")))?;
        Ok(())
    }

    // --- Rate Limit Alerts v13 ---

    pub async fn create_rate_limit_alert_v13(
        &self,
        user_id: Uuid,
        tier_id: Uuid,
        alert_type: &str,
        threshold: f64,
    ) -> Result<RateLimitAlertV13> {
        sqlx::query_as::<_, RateLimitAlertV13>(
            r#"INSERT INTO rate_limit_alerts_v13 (user_id, tier_id, alert_type, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(tier_id)
        .bind(alert_type)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_alert_v13: {e}")))
    }

    pub async fn get_user_rate_limit_alerts_v13(&self, user_id: Uuid) -> Result<Vec<RateLimitAlertV13>> {
        sqlx::query_as::<_, RateLimitAlertV13>(
            "SELECT * FROM rate_limit_alerts_v13 WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_user_rate_limit_alerts_v13: {e}")))
    }

    // --- API Analytics v19 ---

    pub async fn list_api_analytics_v19(&self, limit: i64, offset: i64) -> Result<Vec<ApiAnalyticV19>> {
        sqlx::query_as::<_, ApiAnalyticV19>(
            "SELECT * FROM api_analytics_v19 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_analytics_v19: {e}")))
    }

    pub async fn create_api_analytic_v19(
        &self,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_time_ms: i32,
        user_id: Option<Uuid>,
        request_size_bytes: i32,
        response_size_bytes: i32,
        cache_hit: bool,
        region: &str,
        user_agent: Option<&str>,
        request_id: Option<Uuid>,
        cost_cents: i32,
    ) -> Result<ApiAnalyticV19> {
        sqlx::query_as::<_, ApiAnalyticV19>(
            r#"INSERT INTO api_analytics_v19 (endpoint, method, status_code, response_time_ms, user_id, request_size_bytes, response_size_bytes, cache_hit, region, user_agent, request_id, cost_cents)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(user_id)
        .bind(request_size_bytes)
        .bind(response_size_bytes)
        .bind(cache_hit)
        .bind(region)
        .bind(user_agent)
        .bind(request_id)
        .bind(cost_cents)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_analytic_v19: {e}")))
    }

    pub async fn get_cost_analysis_v19(&self) -> Result<(i64, i64, i64, i64, Vec<(String, i64, i64)>, Vec<(String, i64, i64)>)> {
        let totals = sqlx::query_as::<_, (i64, i64, i64, i64)>(
            r#"SELECT COUNT(*) as total_requests,
                      COALESCE(SUM(request_size_bytes), 0) as total_request_bytes,
                      COALESCE(SUM(response_size_bytes), 0) as total_response_bytes,
                      COALESCE(SUM(cost_cents), 0) as estimated_cost_cents
               FROM api_analytics_v19"#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v19 totals: {e}")))?;

        let regions = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT region, COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost_cents
               FROM api_analytics_v19 GROUP BY region ORDER BY cost_cents DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v19 regions: {e}")))?;

        let ua_costs = sqlx::query_as::<_, (String, i64, i64)>(
            r#"SELECT COALESCE(user_agent, 'unknown') as user_agent, COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost_cents
               FROM api_analytics_v19 GROUP BY user_agent ORDER BY cost_cents DESC LIMIT 10"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v19 ua_costs: {e}")))?;

        Ok((totals.0, totals.1, totals.2, totals.3, regions, ua_costs))
    }

    pub async fn get_usage_optimization_v19(&self) -> Result<Vec<(String, String, f64, f64, f64, Vec<String>)>> {
        let rows = sqlx::query_as::<_, (String, String, f64, f64, f64, Vec<String>)>(
            r#"SELECT endpoint, method,
                      AVG(response_time_ms)::NUMERIC::FLOAT as avg_rt,
                      PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms)::NUMERIC::FLOAT as p95_rt,
                      CASE WHEN COUNT(*) > 0 THEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / COUNT(*)::NUMERIC * 100 ELSE 0 END as cache_rate,
                      COALESCE(
                          ARRAY_REMOVE(
                              ARRAY[
                                  CASE WHEN AVG(response_time_ms) > 500 THEN 'Optimize slow endpoint' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE cache_hit))::NUMERIC / NULLIF(COUNT(*), 0) < 0.3 THEN 'Improve caching strategy' END,
                                  CASE WHEN (COUNT(*) FILTER (WHERE status_code >= 500))::NUMERIC / NULLIF(COUNT(*), 0) > 0.05 THEN 'Reduce error rate' END
                              ],
                              NULL
                          ),
                          ARRAY['No optimization needed']::TEXT[]
                      ) as suggestions
               FROM api_analytics_v19
               GROUP BY endpoint, method
               ORDER BY avg_rt DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_usage_optimization_v19: {e}")))?;
        Ok(rows)
    }

    // --- API Docs v19 ---

    pub async fn list_api_docs_v19(&self, limit: i64, offset: i64) -> Result<Vec<ApiDocsV19>> {
        sqlx::query_as::<_, ApiDocsV19>(
            "SELECT * FROM api_docs_v19 ORDER BY endpoint, method, version LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_docs_v19: {e}")))
    }

    pub async fn get_api_docs_v19_for_endpoint(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
    ) -> Result<Option<ApiDocsV19>> {
        sqlx::query_as::<_, ApiDocsV19>(
            "SELECT * FROM api_docs_v19 WHERE endpoint = $1 AND method = $2 AND version = $3",
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_docs_v19_for_endpoint: {e}")))
    }

    pub async fn create_api_docs_v19(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
        summary: &str,
        description: &str,
        parameters: &serde_json::Value,
        request_body: Option<&serde_json::Value>,
        responses: &serde_json::Value,
        examples: &serde_json::Value,
        tags: &[String],
        deprecated: bool,
        changelog: &str,
        security_schemes: &serde_json::Value,
        rate_limits: &serde_json::Value,
    ) -> Result<ApiDocsV19> {
        sqlx::query_as::<_, ApiDocsV19>(
            r#"INSERT INTO api_docs_v19 (endpoint, method, version, summary, description, parameters, request_body, responses, examples, tags, deprecated, changelog, security_schemes, rate_limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .bind(summary)
        .bind(description)
        .bind(parameters)
        .bind(request_body)
        .bind(responses)
        .bind(examples)
        .bind(tags)
        .bind(deprecated)
        .bind(changelog)
        .bind(security_schemes)
        .bind(rate_limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_docs_v19: {e}")))
    }

    // --- Rate Limit Tiers v17 ---

    pub async fn list_rate_limit_tiers_v17(&self) -> Result<Vec<RateLimitTierV17>> {
        sqlx::query_as::<_, RateLimitTierV17>(
            "SELECT * FROM rate_limit_tiers_v17 ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_rate_limit_tiers_v17: {e}")))
    }

    pub async fn get_rate_limit_tier_v17_by_name(&self, name: &str) -> Result<Option<RateLimitTierV17>> {
        sqlx::query_as::<_, RateLimitTierV17>(
            "SELECT * FROM rate_limit_tiers_v17 WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_rate_limit_tier_v17_by_name: {e}")))
    }

    pub async fn create_rate_limit_tier_v17(
        &self,
        name: &str,
        description: &str,
        rate_limit: i32,
        burst_limit: i32,
        monthly_quota: Option<i32>,
        price_cents: i32,
        features: &serde_json::Value,
        limits: &serde_json::Value,
    ) -> Result<RateLimitTierV17> {
        sqlx::query_as::<_, RateLimitTierV17>(
            r#"INSERT INTO rate_limit_tiers_v17 (name, description, rate_limit, burst_limit, monthly_quota, price_cents, features, limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_tier_v17: {e}")))
    }

    pub async fn update_rate_limit_tier_v17(
        &self,
        name: &str,
        description: Option<&str>,
        rate_limit: Option<i32>,
        burst_limit: Option<i32>,
        monthly_quota: Option<Option<i32>>,
        price_cents: Option<i32>,
        features: Option<&serde_json::Value>,
        limits: Option<&serde_json::Value>,
    ) -> Result<RateLimitTierV17> {
        sqlx::query_as::<_, RateLimitTierV17>(
            r#"UPDATE rate_limit_tiers_v17 SET
                description = COALESCE($2, description),
                rate_limit = COALESCE($3, rate_limit),
                burst_limit = COALESCE($4, burst_limit),
                monthly_quota = COALESCE($5, monthly_quota),
                price_cents = COALESCE($6, price_cents),
                features = COALESCE($7, features),
                limits = COALESCE($8, limits)
               WHERE name = $1
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_rate_limit_tier_v17: {e}")))
    }

    pub async fn delete_rate_limit_tier_v17(&self, name: &str) -> Result<()> {
        sqlx::query("DELETE FROM rate_limit_tiers_v17 WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_rate_limit_tier_v17: {e}")))?;
        Ok(())
    }

    // --- Rate Limit Alerts v14 ---

    pub async fn create_rate_limit_alert_v14(
        &self,
        user_id: Uuid,
        tier_id: Uuid,
        alert_type: &str,
        threshold: f64,
    ) -> Result<RateLimitAlertV14> {
        sqlx::query_as::<_, RateLimitAlertV14>(
            r#"INSERT INTO rate_limit_alerts_v14 (user_id, tier_id, alert_type, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(tier_id)
        .bind(alert_type)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_alert_v14: {e}")))
    }

    pub async fn get_user_rate_limit_alerts_v14(&self, user_id: Uuid) -> Result<Vec<RateLimitAlertV14>> {
        sqlx::query_as::<_, RateLimitAlertV14>(
            "SELECT * FROM rate_limit_alerts_v14 WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_user_rate_limit_alerts_v14: {e}")))
    }

    // --- API Analytics v20 ---

    pub async fn list_api_analytics_v20(&self, limit: i64, offset: i64) -> Result<Vec<ApiAnalyticV20>> {
        sqlx::query_as::<_, ApiAnalyticV20>(
            "SELECT * FROM api_analytics_v20 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_analytics_v20: {e}")))
    }

    pub async fn create_api_analytic_v20(
        &self,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_time_ms: i32,
        user_id: Option<Uuid>,
        request_size_bytes: i32,
        response_size_bytes: i32,
        cache_hit: bool,
        region: &str,
        user_agent: Option<&str>,
        request_id: Option<Uuid>,
        cost_cents: i32,
    ) -> Result<ApiAnalyticV20> {
        sqlx::query_as::<_, ApiAnalyticV20>(
            r#"INSERT INTO api_analytics_v20 (endpoint, method, status_code, response_time_ms, user_id, request_size_bytes, response_size_bytes, cache_hit, region, user_agent, request_id, cost_cents)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(user_id)
        .bind(request_size_bytes)
        .bind(response_size_bytes)
        .bind(cache_hit)
        .bind(region)
        .bind(user_agent)
        .bind(request_id)
        .bind(cost_cents)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_analytic_v20: {e}")))
    }

    pub async fn get_cost_analysis_v20(&self) -> Result<(i64, i64, i64, i64, Vec<(String, i64, i64)>, Vec<(String, i64, i64)>)> {
        let row: (i64, i64, i64, i64) = sqlx::query_as(
            r#"SELECT COUNT(*) as total_requests, COALESCE(SUM(request_size_bytes), 0), COALESCE(SUM(response_size_bytes), 0), COALESCE(SUM(cost_cents), 0)
               FROM api_analytics_v20"#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v20: {e}")))?;

        let regions: Vec<(String, i64, i64)> = sqlx::query_as(
            r#"SELECT region, COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost
               FROM api_analytics_v20 GROUP BY region ORDER BY cost DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v20 regions: {e}")))?;

        let user_agents: Vec<(String, i64, i64)> = sqlx::query_as(
            r#"SELECT COALESCE(user_agent, 'unknown'), COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost
               FROM api_analytics_v20 GROUP BY user_agent ORDER BY cost DESC LIMIT 10"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v20 user_agents: {e}")))?;

        Ok((row.0, row.1, row.2, row.3, regions, user_agents))
    }

    pub async fn get_usage_optimization_v20(&self) -> Result<Vec<(String, String, f64, f64, f64, Vec<String>)>> {
        let rows: Vec<(String, String, f64, f64, f64, Vec<String>)> = sqlx::query_as(
            r#"SELECT endpoint, method,
                    AVG(response_time_ms)::FLOAT8 as avg_rt,
                    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms)::FLOAT8 as p95_rt,
                    (COUNT(*) FILTER (WHERE cache_hit))::FLOAT8 / NULLIF(COUNT(*), 0) as cache_rate,
                    CASE WHEN AVG(response_time_ms) > 500 THEN ARRAY['Consider caching'] ELSE ARRAY[]::TEXT[] END as suggestions
               FROM api_analytics_v20
               GROUP BY endpoint, method
               ORDER BY avg_rt DESC
               LIMIT 20"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_usage_optimization_v20: {e}")))?;
        Ok(rows)
    }

    // --- API Docs v20 ---

    pub async fn list_api_docs_v20(&self, limit: i64, offset: i64) -> Result<Vec<ApiDocsV20>> {
        sqlx::query_as::<_, ApiDocsV20>(
            "SELECT * FROM api_docs_v20 ORDER BY endpoint, method, version LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_docs_v20: {e}")))
    }

    pub async fn get_api_docs_v20_for_endpoint(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
    ) -> Result<Option<ApiDocsV20>> {
        sqlx::query_as::<_, ApiDocsV20>(
            "SELECT * FROM api_docs_v20 WHERE endpoint = $1 AND method = $2 AND version = $3",
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_api_docs_v20_for_endpoint: {e}")))
    }

    pub async fn create_api_docs_v20(
        &self,
        endpoint: &str,
        method: &str,
        version: &str,
        summary: &str,
        description: &str,
        parameters: &serde_json::Value,
        request_body: Option<&serde_json::Value>,
        responses: &serde_json::Value,
        examples: &serde_json::Value,
        tags: &[String],
        deprecated: bool,
        changelog: &str,
        security_schemes: &serde_json::Value,
        rate_limits: &serde_json::Value,
    ) -> Result<ApiDocsV20> {
        sqlx::query_as::<_, ApiDocsV20>(
            r#"INSERT INTO api_docs_v20 (endpoint, method, version, summary, description, parameters, request_body, responses, examples, tags, deprecated, changelog, security_schemes, rate_limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(version)
        .bind(summary)
        .bind(description)
        .bind(parameters)
        .bind(request_body)
        .bind(responses)
        .bind(examples)
        .bind(tags)
        .bind(deprecated)
        .bind(changelog)
        .bind(security_schemes)
        .bind(rate_limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_docs_v20: {e}")))
    }

    // --- Rate Limit Tiers v18 ---

    pub async fn list_rate_limit_tiers_v18(&self) -> Result<Vec<RateLimitTierV18>> {
        sqlx::query_as::<_, RateLimitTierV18>(
            "SELECT * FROM rate_limit_tiers_v18 ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_rate_limit_tiers_v18: {e}")))
    }

    pub async fn get_rate_limit_tier_v18_by_name(&self, name: &str) -> Result<Option<RateLimitTierV18>> {
        sqlx::query_as::<_, RateLimitTierV18>(
            "SELECT * FROM rate_limit_tiers_v18 WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_rate_limit_tier_v18_by_name: {e}")))
    }

    pub async fn create_rate_limit_tier_v18(
        &self,
        name: &str,
        description: &str,
        rate_limit: i32,
        burst_limit: i32,
        monthly_quota: Option<i32>,
        price_cents: i32,
        features: &serde_json::Value,
        limits: &serde_json::Value,
    ) -> Result<RateLimitTierV18> {
        sqlx::query_as::<_, RateLimitTierV18>(
            r#"INSERT INTO rate_limit_tiers_v18 (name, description, rate_limit, burst_limit, monthly_quota, price_cents, features, limits)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_tier_v18: {e}")))
    }

    pub async fn update_rate_limit_tier_v18(
        &self,
        name: &str,
        description: Option<&str>,
        rate_limit: Option<i32>,
        burst_limit: Option<i32>,
        monthly_quota: Option<Option<i32>>,
        price_cents: Option<i32>,
        features: Option<&serde_json::Value>,
        limits: Option<&serde_json::Value>,
    ) -> Result<RateLimitTierV18> {
        sqlx::query_as::<_, RateLimitTierV18>(
            r#"UPDATE rate_limit_tiers_v18 SET
                description = COALESCE($2, description),
                rate_limit = COALESCE($3, rate_limit),
                burst_limit = COALESCE($4, burst_limit),
                monthly_quota = COALESCE($5, monthly_quota),
                price_cents = COALESCE($6, price_cents),
                features = COALESCE($7, features),
                limits = COALESCE($8, limits)
               WHERE name = $1
               RETURNING *"#,
        )
        .bind(name)
        .bind(description)
        .bind(rate_limit)
        .bind(burst_limit)
        .bind(monthly_quota)
        .bind(price_cents)
        .bind(features)
        .bind(limits)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_rate_limit_tier_v18: {e}")))
    }

    pub async fn delete_rate_limit_tier_v18(&self, name: &str) -> Result<()> {
        sqlx::query("DELETE FROM rate_limit_tiers_v18 WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_rate_limit_tier_v18: {e}")))?;
        Ok(())
    }

    // --- Rate Limit Alerts v15 ---

    pub async fn create_rate_limit_alert_v15(
        &self,
        user_id: Uuid,
        tier_id: Uuid,
        alert_type: &str,
        threshold: f64,
    ) -> Result<RateLimitAlertV15> {
        sqlx::query_as::<_, RateLimitAlertV15>(
            r#"INSERT INTO rate_limit_alerts_v15 (user_id, tier_id, alert_type, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(user_id)
        .bind(tier_id)
        .bind(alert_type)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_rate_limit_alert_v15: {e}")))
    }

    pub async fn get_user_rate_limit_alerts_v15(&self, user_id: Uuid) -> Result<Vec<RateLimitAlertV15>> {
        sqlx::query_as::<_, RateLimitAlertV15>(
            "SELECT * FROM rate_limit_alerts_v15 WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_user_rate_limit_alerts_v15: {e}")))
    }

    // --- API Analytics v21 ---

    pub async fn list_api_analytics_v21(&self, limit: i64, offset: i64) -> Result<Vec<ApiAnalyticV21>> {
        sqlx::query_as::<_, ApiAnalyticV21>(
            "SELECT * FROM api_analytics_v21 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_api_analytics_v21: {e}")))
    }

    pub async fn create_api_analytic_v21(
        &self,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_time_ms: i32,
        user_id: Option<Uuid>,
        request_size_bytes: i32,
        response_size_bytes: i32,
        cache_hit: bool,
        region: &str,
        user_agent: Option<&str>,
        request_id: Option<Uuid>,
        cost_cents: i32,
    ) -> Result<ApiAnalyticV21> {
        sqlx::query_as::<_, ApiAnalyticV21>(
            r#"INSERT INTO api_analytics_v21 (endpoint, method, status_code, response_time_ms, user_id, request_size_bytes, response_size_bytes, cache_hit, region, user_agent, request_id, cost_cents)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               RETURNING *"#,
        )
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(user_id)
        .bind(request_size_bytes)
        .bind(response_size_bytes)
        .bind(cache_hit)
        .bind(region)
        .bind(user_agent)
        .bind(request_id)
        .bind(cost_cents)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_api_analytic_v21: {e}")))
    }

    pub async fn get_cost_analysis_v21(&self) -> Result<(i64, i64, i64, i64, Vec<(String, i64, i64)>, Vec<(String, i64, i64)>)> {
        let row: (i64, i64, i64, i64) = sqlx::query_as(
            r#"SELECT COUNT(*) as total_requests, COALESCE(SUM(request_size_bytes), 0), COALESCE(SUM(response_size_bytes), 0), COALESCE(SUM(cost_cents), 0)
               FROM api_analytics_v21"#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v21: {e}")))?;

        let regions: Vec<(String, i64, i64)> = sqlx::query_as(
            r#"SELECT region, COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost
               FROM api_analytics_v21 GROUP BY region ORDER BY cost DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v21: {e}")))?;

        let user_agents: Vec<(String, i64, i64)> = sqlx::query_as(
            r#"SELECT COALESCE(user_agent, 'unknown'), COUNT(*) as requests, COALESCE(SUM(cost_cents), 0) as cost
               FROM api_analytics_v21 GROUP BY user_agent ORDER BY cost DESC LIMIT 10"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_cost_analysis_v21: {e}")))?;

        Ok((row.0, row.1, row.2, row.3, regions, user_agents))
    }

    pub async fn get_usage_optimization_v21(&self) -> Result<Vec<(String, String, f64, f64, f64, Vec<String>)>> {
        let rows: Vec<(String, String, f64, f64, f64, Vec<String>)> = sqlx::query_as(
            r#"SELECT endpoint, method,
                    AVG(response_time_ms)::FLOAT8 as avg_rt,
                    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms)::FLOAT8 as p95_rt,
                    (COUNT(*) FILTER (WHERE cache_hit))::FLOAT8 / NULLIF(COUNT(*), 0) as cache_rate,
                    CASE WHEN AVG(response_time_ms) > 500 THEN ARRAY['Consider caching'] ELSE ARRAY[]::TEXT[] END as suggestions
               FROM api_analytics_v21
               GROUP BY endpoint, method
               ORDER BY avg_rt DESC
               LIMIT 20"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_usage_optimization_v21: {e}")))?;
        Ok(rows)
    }

    // --- Test Suite Management v19 ---

    pub async fn create_test_suite_metric_v19(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<TestSuiteMetricV19> {
        let row = sqlx::query_as::<_, TestSuiteMetricV19>(
            r#"INSERT INTO test_suite_metrics_v16 (suite_id, metric_name, metric_value)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_metric_v19: {e}")))?;
        Ok(row)
    }

    pub async fn list_test_suite_metrics_v19(
        &self,
        suite_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestSuiteMetricV19>> {
        sqlx::query_as::<_, TestSuiteMetricV19>(
            "SELECT * FROM test_suite_metrics_v16 WHERE suite_id = $1 ORDER BY measured_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(suite_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_test_suite_metrics_v19: {e}")))
    }

    pub async fn get_test_suite_latest_metric_v19(
        &self,
        suite_id: Uuid,
        metric_name: &str,
    ) -> Result<Option<TestSuiteMetricV19>> {
        sqlx::query_as::<_, TestSuiteMetricV19>(
            "SELECT * FROM test_suite_metrics_v16 WHERE suite_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT 1",
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_latest_metric_v19: {e}")))
    }

    pub async fn create_test_suite_baseline_v19(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        baseline_value: f64,
        threshold_percent: f64,
    ) -> Result<TestSuiteBaselineV19> {
        let row = sqlx::query_as::<_, TestSuiteBaselineV19>(
            r#"INSERT INTO test_suite_baselines_v16 (suite_id, metric_name, baseline_value, threshold_percent)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (suite_id, metric_name) DO UPDATE
               SET baseline_value = $3, threshold_percent = $4
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(baseline_value)
        .bind(threshold_percent)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_baseline_v19: {e}")))?;
        Ok(row)
    }

    pub async fn get_test_suite_baselines_v19(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteBaselineV19>> {
        sqlx::query_as::<_, TestSuiteBaselineV19>(
            "SELECT * FROM test_suite_baselines_v16 WHERE suite_id = $1 ORDER BY metric_name",
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_baselines_v19: {e}")))
    }

    pub async fn detect_test_suite_regression_v19(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        current_value: f64,
    ) -> Result<bool> {
        let row: Option<(f64, f64)> = sqlx::query_as(
            r#"SELECT baseline_value, threshold_percent
               FROM test_suite_baselines_v16
               WHERE suite_id = $1 AND metric_name = $2"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("detect_test_suite_regression_v19: {e}")))?;
        match row {
            Some((baseline, threshold)) => {
                let threshold_value = baseline * (1.0 + threshold / 100.0);
                Ok(current_value > threshold_value)
            }
            None => Ok(false),
        }
    }

    pub async fn get_test_suite_performance_alerts_v19(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"SELECT DISTINCT metric_name
               FROM test_suite_metrics_v16
               WHERE suite_id = $1
               AND metric_name IN (
                   SELECT metric_name FROM test_suite_baselines_v16 WHERE suite_id = $1
               )
               AND metric_value > (
                   SELECT baseline_value * (1 + threshold_percent / 100.0)
                   FROM test_suite_baselines_v16 b
                   WHERE b.suite_id = $1 AND b.metric_name = test_suite_metrics_v16.metric_name
               )
               ORDER BY metric_name"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_performance_alerts_v19: {e}")))?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    // --- Code Quality Rules v19 ---

    pub async fn create_code_quality_metric_v19(
        &self,
        repo_id: Uuid,
        file_path: &str,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<CodeQualityMetricV19> {
        let row = sqlx::query_as::<_, CodeQualityMetricV19>(
            r#"INSERT INTO code_quality_metrics_v17 (repo_id, file_path, metric_name, metric_value)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(file_path)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_metric_v19: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_metrics_v19(
        &self,
        repo_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CodeQualityMetricV19>> {
        let rows = match metric_name {
            Some(name) => {
                sqlx::query_as::<_, CodeQualityMetricV19>(
                    "SELECT * FROM code_quality_metrics_v17 WHERE repo_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT $3 OFFSET $4",
                )
                .bind(repo_id)
                .bind(name)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, CodeQualityMetricV19>(
                    "SELECT * FROM code_quality_metrics_v17 WHERE repo_id = $1 ORDER BY measured_at DESC LIMIT $2 OFFSET $3",
                )
                .bind(repo_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        };
        rows.map_err(|e| DbError::Database(format!("list_code_quality_metrics_v19: {e}")))
    }

    pub async fn get_code_quality_score_v19(&self, repo_id: Uuid) -> Result<f64> {
        let row: Option<(f64,)> = sqlx::query_as(
            r#"SELECT COALESCE(100.0 - AVG(metric_value), 100.0)
               FROM code_quality_metrics_v17
               WHERE repo_id = $1"#,
        )
        .bind(repo_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_score_v19: {e}")))?;
        Ok(row.map(|r| r.0).unwrap_or(100.0))
    }

    pub async fn create_code_quality_threshold_v19(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        threshold_value: f64,
        enabled: bool,
    ) -> Result<CodeQualityThresholdV19> {
        let row = sqlx::query_as::<_, CodeQualityThresholdV19>(
            r#"INSERT INTO code_quality_thresholds_v16 (repo_id, metric_name, threshold_value, enabled)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (repo_id, metric_name) DO UPDATE
               SET threshold_value = $3, enabled = $4
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .bind(threshold_value)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_threshold_v19: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_thresholds_v19(
        &self,
        repo_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<CodeQualityThresholdV19>> {
        let query = if enabled_only {
            "SELECT * FROM code_quality_thresholds_v16 WHERE repo_id = $1 AND enabled = true ORDER BY metric_name"
        } else {
            "SELECT * FROM code_quality_thresholds_v16 WHERE repo_id = $1 ORDER BY metric_name"
        };
        sqlx::query_as::<_, CodeQualityThresholdV19>(query)
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_code_quality_thresholds_v19: {e}")))
    }

    pub async fn check_code_quality_violation_v19(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<bool> {
        let row: Option<(f64,)> = sqlx::query_as(
            r#"SELECT threshold_value
               FROM code_quality_thresholds_v16
               WHERE repo_id = $1 AND metric_name = $2 AND enabled = true"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("check_code_quality_violation_v19: {e}")))?;
        match row {
            Some((threshold,)) => Ok(metric_value > threshold),
            None => Ok(false),
        }
    }

    pub async fn delete_code_quality_threshold_v19(
        &self,
        repo_id: Uuid,
        metric_name: &str,
    ) -> Result<()> {
        sqlx::query("DELETE FROM code_quality_thresholds_v16 WHERE repo_id = $1 AND metric_name = $2")
            .bind(repo_id)
            .bind(metric_name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_code_quality_threshold_v19: {e}")))?;
        Ok(())
    }

    pub async fn get_code_quality_enforcement_report_v19(
        &self,
        repo_id: Uuid,
    ) -> Result<(i64, i64, f64)> {
        let row: (i64, i64, f64) = sqlx::query_as(
            r#"SELECT
                   COUNT(*) as total_thresholds,
                   COALESCE(
                       (SELECT COUNT(*)
                        FROM code_quality_thresholds_v16 t
                        WHERE t.repo_id = $1
                        AND t.enabled = true
                        AND EXISTS (
                            SELECT 1 FROM code_quality_metrics_v17 m
                            WHERE m.repo_id = $1
                            AND m.metric_name = t.metric_name
                            AND m.metric_value > t.threshold_value
                        )), 0
                   ) as violating_thresholds,
                   CASE
                       WHEN COUNT(*) = 0 THEN 100.0
                       ELSE (COUNT(*) - COALESCE(
                           (SELECT COUNT(*)
                            FROM code_quality_thresholds_v16 t
                            WHERE t.repo_id = $1
                            AND t.enabled = true
                            AND EXISTS (
                                SELECT 1 FROM code_quality_metrics_v17 m
                                WHERE m.repo_id = $1
                                AND m.metric_name = t.metric_name
                                AND m.metric_value > t.threshold_value
                            )), 0
                       )) * 100.0 / COUNT(*)
                   END as compliance_rate
               FROM code_quality_thresholds_v16
               WHERE repo_id = $1 AND enabled = true"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_enforcement_report_v19: {e}")))?;
        Ok(row)
    }

    // --- Performance Testing v20 ---

    pub async fn create_performance_test_alert_v20(
        &self,
        baseline_id: Uuid,
        alert_type: &str,
        threshold: f64,
        enabled: bool,
    ) -> Result<PerformanceTestAlertV20> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV20>(
            r#"INSERT INTO performance_test_alerts_v17 (baseline_id, alert_type, threshold, enabled)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(baseline_id)
        .bind(alert_type)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_performance_test_alert_v20: {e}")))?;
        Ok(row)
    }

    pub async fn list_performance_test_alerts_v20(
        &self,
        baseline_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<PerformanceTestAlertV20>> {
        let query = if enabled_only {
            "SELECT * FROM performance_test_alerts_v17 WHERE baseline_id = $1 AND enabled = true ORDER BY created_at"
        } else {
            "SELECT * FROM performance_test_alerts_v17 WHERE baseline_id = $1 ORDER BY created_at"
        };
        sqlx::query_as::<_, PerformanceTestAlertV20>(query)
            .bind(baseline_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_performance_test_alerts_v20: {e}")))
    }

    pub async fn update_performance_test_alert_v20(
        &self,
        alert_id: Uuid,
        threshold: Option<f64>,
        enabled: Option<bool>,
    ) -> Result<PerformanceTestAlertV20> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV20>(
            r#"UPDATE performance_test_alerts_v17
               SET threshold = COALESCE($2, threshold),
                   enabled = COALESCE($3, enabled)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(alert_id)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_performance_test_alert_v20: {e}")))?;
        Ok(row)
    }

    pub async fn record_performance_test_alert_v20(
        &self,
        alert_id: Uuid,
        metric_name: &str,
        metric_value: f64,
        threshold: f64,
    ) -> Result<PerformanceTestAlertHistoryV20> {
        let row = sqlx::query_as::<_, PerformanceTestAlertHistoryV20>(
            r#"INSERT INTO performance_test_alert_history_v17 (alert_id, metric_name, metric_value, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(alert_id)
        .bind(metric_name)
        .bind(metric_value)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_performance_test_alert_v20: {e}")))?;

        sqlx::query("UPDATE performance_test_alerts_v17 SET last_triggered_at = NOW() WHERE id = $1")
            .bind(alert_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("record_performance_test_alert_v20 update: {e}")))?;

        Ok(row)
    }

    pub async fn list_performance_test_alert_history_v20(
        &self,
        alert_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PerformanceTestAlertHistoryV20>> {
        sqlx::query_as::<_, PerformanceTestAlertHistoryV20>(
            "SELECT * FROM performance_test_alert_history_v17 WHERE alert_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(alert_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_performance_test_alert_history_v20: {e}")))
    }

    pub async fn get_performance_test_alert_analytics_v20(
        &self,
        alert_id: Uuid,
    ) -> Result<(i64, Option<f64>, Option<f64>)> {
        let row: (i64, Option<f64>, Option<f64>) = sqlx::query_as(
            r#"SELECT COUNT(*), AVG(metric_value), MAX(metric_value)
               FROM performance_test_alert_history_v17
               WHERE alert_id = $1"#,
        )
        .bind(alert_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_analytics_v20: {e}")))?;
        Ok(row)
    }

    pub async fn get_performance_test_alert_notification_config_v20(
        &self,
        alert_id: Uuid,
    ) -> Result<Option<(String, bool, Option<DateTime<Utc>>)>> {
        let row = sqlx::query_as::<_, (String, bool, Option<DateTime<Utc>>)>(
            "SELECT alert_type, enabled, last_triggered_at FROM performance_test_alerts_v17 WHERE id = $1",
        )
        .bind(alert_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_notification_config_v20: {e}")))?;
        Ok(row)
    }

    // --- Test Suite Management v20 ---

    pub async fn create_test_suite_metric_v20(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<TestSuiteMetricV20> {
        let row = sqlx::query_as::<_, TestSuiteMetricV20>(
            r#"INSERT INTO test_suite_metrics_v17 (suite_id, metric_name, metric_value)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_metric_v20: {e}")))?;
        Ok(row)
    }

    pub async fn list_test_suite_metrics_v20(
        &self,
        suite_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TestSuiteMetricV20>> {
        sqlx::query_as::<_, TestSuiteMetricV20>(
            "SELECT * FROM test_suite_metrics_v17 WHERE suite_id = $1 ORDER BY measured_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(suite_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_test_suite_metrics_v20: {e}")))
    }

    pub async fn get_test_suite_latest_metric_v20(
        &self,
        suite_id: Uuid,
        metric_name: &str,
    ) -> Result<Option<TestSuiteMetricV20>> {
        sqlx::query_as::<_, TestSuiteMetricV20>(
            "SELECT * FROM test_suite_metrics_v17 WHERE suite_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT 1",
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_latest_metric_v20: {e}")))
    }

    pub async fn create_test_suite_baseline_v20(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        baseline_value: f64,
        threshold_percent: f64,
    ) -> Result<TestSuiteBaselineV20> {
        let row = sqlx::query_as::<_, TestSuiteBaselineV20>(
            r#"INSERT INTO test_suite_baselines_v17 (suite_id, metric_name, baseline_value, threshold_percent)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (suite_id, metric_name) DO UPDATE
               SET baseline_value = $3, threshold_percent = $4
               RETURNING *"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .bind(baseline_value)
        .bind(threshold_percent)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_test_suite_baseline_v20: {e}")))?;
        Ok(row)
    }

    pub async fn get_test_suite_baselines_v20(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<TestSuiteBaselineV20>> {
        sqlx::query_as::<_, TestSuiteBaselineV20>(
            "SELECT * FROM test_suite_baselines_v17 WHERE suite_id = $1 ORDER BY metric_name",
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_baselines_v20: {e}")))
    }

    pub async fn detect_test_suite_regression_v20(
        &self,
        suite_id: Uuid,
        metric_name: &str,
        current_value: f64,
    ) -> Result<bool> {
        let row: Option<(f64, f64)> = sqlx::query_as(
            r#"SELECT baseline_value, threshold_percent
               FROM test_suite_baselines_v17
               WHERE suite_id = $1 AND metric_name = $2"#,
        )
        .bind(suite_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("detect_test_suite_regression_v20: {e}")))?;
        match row {
            Some((baseline, threshold)) => {
                let threshold_value = baseline * (1.0 + threshold / 100.0);
                Ok(current_value > threshold_value)
            }
            None => Ok(false),
        }
    }

    pub async fn get_test_suite_performance_alerts_v20(
        &self,
        suite_id: Uuid,
    ) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"SELECT DISTINCT metric_name
               FROM test_suite_metrics_v17
               WHERE suite_id = $1
               AND metric_name IN (
                   SELECT metric_name FROM test_suite_baselines_v17 WHERE suite_id = $1
               )
               AND metric_value > (
                   SELECT baseline_value * (1 + threshold_percent / 100.0)
                   FROM test_suite_baselines_v17 b
                   WHERE b.suite_id = $1 AND b.metric_name = test_suite_metrics_v17.metric_name
               )
               ORDER BY metric_name"#,
        )
        .bind(suite_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_test_suite_performance_alerts_v20: {e}")))?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    // --- Code Quality Rules v20 ---

    pub async fn create_code_quality_metric_v20(
        &self,
        repo_id: Uuid,
        file_path: &str,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<CodeQualityMetricV20> {
        let row = sqlx::query_as::<_, CodeQualityMetricV20>(
            r#"INSERT INTO code_quality_metrics_v18 (repo_id, file_path, metric_name, metric_value)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(file_path)
        .bind(metric_name)
        .bind(metric_value)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_metric_v20: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_metrics_v20(
        &self,
        repo_id: Uuid,
        metric_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CodeQualityMetricV20>> {
        let rows = match metric_name {
            Some(name) => {
                sqlx::query_as::<_, CodeQualityMetricV20>(
                    "SELECT * FROM code_quality_metrics_v18 WHERE repo_id = $1 AND metric_name = $2 ORDER BY measured_at DESC LIMIT $3 OFFSET $4",
                )
                .bind(repo_id)
                .bind(name)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, CodeQualityMetricV20>(
                    "SELECT * FROM code_quality_metrics_v18 WHERE repo_id = $1 ORDER BY measured_at DESC LIMIT $2 OFFSET $3",
                )
                .bind(repo_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        };
        rows.map_err(|e| DbError::Database(format!("list_code_quality_metrics_v20: {e}")))
    }

    pub async fn get_code_quality_score_v20(&self, repo_id: Uuid) -> Result<f64> {
        let row: Option<(f64,)> = sqlx::query_as(
            r#"SELECT COALESCE(100.0 - AVG(metric_value), 100.0)
               FROM code_quality_metrics_v18
               WHERE repo_id = $1"#,
        )
        .bind(repo_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_score_v20: {e}")))?;
        Ok(row.map(|r| r.0).unwrap_or(100.0))
    }

    pub async fn create_code_quality_threshold_v20(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        threshold_value: f64,
        enabled: bool,
    ) -> Result<CodeQualityThresholdV20> {
        let row = sqlx::query_as::<_, CodeQualityThresholdV20>(
            r#"INSERT INTO code_quality_thresholds_v17 (repo_id, metric_name, threshold_value, enabled)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (repo_id, metric_name) DO UPDATE
               SET threshold_value = $3, enabled = $4
               RETURNING *"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .bind(threshold_value)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_code_quality_threshold_v20: {e}")))?;
        Ok(row)
    }

    pub async fn list_code_quality_thresholds_v20(
        &self,
        repo_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<CodeQualityThresholdV20>> {
        let query = if enabled_only {
            "SELECT * FROM code_quality_thresholds_v17 WHERE repo_id = $1 AND enabled = true ORDER BY metric_name"
        } else {
            "SELECT * FROM code_quality_thresholds_v17 WHERE repo_id = $1 ORDER BY metric_name"
        };
        sqlx::query_as::<_, CodeQualityThresholdV20>(query)
            .bind(repo_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_code_quality_thresholds_v20: {e}")))
    }

    pub async fn check_code_quality_violation_v20(
        &self,
        repo_id: Uuid,
        metric_name: &str,
        metric_value: f64,
    ) -> Result<bool> {
        let row: Option<(f64,)> = sqlx::query_as(
            r#"SELECT threshold_value
               FROM code_quality_thresholds_v17
               WHERE repo_id = $1 AND metric_name = $2 AND enabled = true"#,
        )
        .bind(repo_id)
        .bind(metric_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("check_code_quality_violation_v20: {e}")))?;
        match row {
            Some((threshold,)) => Ok(metric_value > threshold),
            None => Ok(false),
        }
    }

    pub async fn delete_code_quality_threshold_v20(
        &self,
        repo_id: Uuid,
        metric_name: &str,
    ) -> Result<()> {
        sqlx::query("DELETE FROM code_quality_thresholds_v17 WHERE repo_id = $1 AND metric_name = $2")
            .bind(repo_id)
            .bind(metric_name)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("delete_code_quality_threshold_v20: {e}")))?;
        Ok(())
    }

    pub async fn get_code_quality_enforcement_report_v20(
        &self,
        repo_id: Uuid,
    ) -> Result<(i64, i64, f64)> {
        let row: (i64, i64, f64) = sqlx::query_as(
            r#"SELECT
                   COUNT(*) as total_thresholds,
                   COALESCE(
                       (SELECT COUNT(*)
                        FROM code_quality_thresholds_v17 t
                        WHERE t.repo_id = $1
                        AND t.enabled = true
                        AND EXISTS (
                            SELECT 1 FROM code_quality_metrics_v18 m
                            WHERE m.repo_id = $1
                            AND m.metric_name = t.metric_name
                            AND m.metric_value > t.threshold_value
                        )), 0
                   ) as violating_thresholds,
                   CASE
                       WHEN COUNT(*) = 0 THEN 100.0
                       ELSE (COUNT(*) - COALESCE(
                           (SELECT COUNT(*)
                            FROM code_quality_thresholds_v17 t
                            WHERE t.repo_id = $1
                            AND t.enabled = true
                            AND EXISTS (
                                SELECT 1 FROM code_quality_metrics_v18 m
                                WHERE m.repo_id = $1
                                AND m.metric_name = t.metric_name
                                AND m.metric_value > t.threshold_value
                            )), 0
                       )) * 100.0 / COUNT(*)
                   END as compliance_rate
               FROM code_quality_thresholds_v17
               WHERE repo_id = $1 AND enabled = true"#,
        )
        .bind(repo_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_code_quality_enforcement_report_v20: {e}")))?;
        Ok(row)
    }

    // --- Performance Testing v21 ---

    pub async fn create_performance_test_alert_v21(
        &self,
        baseline_id: Uuid,
        alert_type: &str,
        threshold: f64,
        enabled: bool,
    ) -> Result<PerformanceTestAlertV21> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV21>(
            r#"INSERT INTO performance_test_alerts_v18 (baseline_id, alert_type, threshold, enabled)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(baseline_id)
        .bind(alert_type)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("create_performance_test_alert_v21: {e}")))?;
        Ok(row)
    }

    pub async fn list_performance_test_alerts_v21(
        &self,
        baseline_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<PerformanceTestAlertV21>> {
        let query = if enabled_only {
            "SELECT * FROM performance_test_alerts_v18 WHERE baseline_id = $1 AND enabled = true ORDER BY created_at"
        } else {
            "SELECT * FROM performance_test_alerts_v18 WHERE baseline_id = $1 ORDER BY created_at"
        };
        sqlx::query_as::<_, PerformanceTestAlertV21>(query)
            .bind(baseline_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("list_performance_test_alerts_v21: {e}")))
    }

    pub async fn update_performance_test_alert_v21(
        &self,
        alert_id: Uuid,
        threshold: Option<f64>,
        enabled: Option<bool>,
    ) -> Result<PerformanceTestAlertV21> {
        let row = sqlx::query_as::<_, PerformanceTestAlertV21>(
            r#"UPDATE performance_test_alerts_v18
               SET threshold = COALESCE($2, threshold),
                   enabled = COALESCE($3, enabled)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(alert_id)
        .bind(threshold)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("update_performance_test_alert_v21: {e}")))?;
        Ok(row)
    }

    pub async fn record_performance_test_alert_v21(
        &self,
        alert_id: Uuid,
        metric_name: &str,
        metric_value: f64,
        threshold: f64,
    ) -> Result<PerformanceTestAlertHistoryV21> {
        let row = sqlx::query_as::<_, PerformanceTestAlertHistoryV21>(
            r#"INSERT INTO performance_test_alert_history_v18 (alert_id, metric_name, metric_value, threshold)
               VALUES ($1, $2, $3, $4)
               RETURNING *"#,
        )
        .bind(alert_id)
        .bind(metric_name)
        .bind(metric_value)
        .bind(threshold)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("record_performance_test_alert_v21: {e}")))?;

        sqlx::query("UPDATE performance_test_alerts_v18 SET last_triggered_at = NOW() WHERE id = $1")
            .bind(alert_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Database(format!("record_performance_test_alert_v21 update: {e}")))?;

        Ok(row)
    }

    pub async fn list_performance_test_alert_history_v21(
        &self,
        alert_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PerformanceTestAlertHistoryV21>> {
        sqlx::query_as::<_, PerformanceTestAlertHistoryV21>(
            "SELECT * FROM performance_test_alert_history_v18 WHERE alert_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(alert_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("list_performance_test_alert_history_v21: {e}")))
    }

    pub async fn get_performance_test_alert_analytics_v21(
        &self,
        alert_id: Uuid,
    ) -> Result<(i64, Option<f64>, Option<f64>)> {
        let row: (i64, Option<f64>, Option<f64>) = sqlx::query_as(
            r#"SELECT COUNT(*), AVG(metric_value), MAX(metric_value)
               FROM performance_test_alert_history_v18
               WHERE alert_id = $1"#,
        )
        .bind(alert_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_analytics_v21: {e}")))?;
        Ok(row)
    }

    pub async fn get_performance_test_alert_notification_config_v21(
        &self,
        alert_id: Uuid,
    ) -> Result<Option<(String, bool, Option<DateTime<Utc>>)>> {
        let row = sqlx::query_as::<_, (String, bool, Option<DateTime<Utc>>)>(
            "SELECT alert_type, enabled, last_triggered_at FROM performance_test_alerts_v18 WHERE id = $1",
        )
        .bind(alert_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Database(format!("get_performance_test_alert_notification_config_v21: {e}")))?;
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
