#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub bio: String,
    pub role: String,
    #[serde(default)]
    pub email_verified: bool,
    #[serde(default)]
    pub banned: bool,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Org {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub visibility: String,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Repository {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub owner_id: Uuid,
    pub org_id: Option<Uuid>,
    pub visibility: String,
    pub default_branch: String,
    pub is_fork: bool,
    pub parent_repo_id: Option<Uuid>,
    pub stars_count: i64,
    pub watchers_count: i64,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub topics: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Issue {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub number: i32,
    pub title: String,
    pub body: String,
    pub status: String,
    pub author_id: Uuid,
    pub assignee_id: Option<Uuid>,
    pub labels: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PullRequest {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub number: i32,
    pub title: String,
    pub body: String,
    pub status: String,
    pub author_id: Uuid,
    pub source_branch: String,
    pub target_branch: String,
    pub merge_commit_id: Option<String>,
    pub draft: bool,
    pub head_commit_sha: Option<String>,
    pub base_commit_sha: Option<String>,
    pub merge_strategy: String,
    #[serde(default)]
    pub auto_merge: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub merged_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PrComment {
    pub id: Uuid,
    pub pr_id: Uuid,
    pub author_id: Uuid,
    pub body: String,
    pub commit_sha: Option<String>,
    pub file_path: Option<String>,
    pub start_line: Option<i32>,
    pub end_line: Option<i32>,
    pub line: Option<i32>,
    pub in_reply_to_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PrReviewer {
    pub pr_id: Uuid,
    pub user_id: Uuid,
    pub review_status: String,
    pub submitted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PrTimeline {
    pub id: Uuid,
    pub pr_id: Uuid,
    pub actor_id: Uuid,
    pub event_type: String,
    pub event_detail: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PrStatusCheck {
    pub id: Uuid,
    pub pr_id: Uuid,
    pub context: String,
    pub state: String,
    pub description: String,
    pub target_url: Option<String>,
    pub commit_sha: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SshKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key_type: String,
    pub public_key: String,
    pub fingerprint: String,
    pub label: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Pipeline {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub commit_sha: String,
    pub status: String,
    pub trigger: String,
    pub yaml_path: String,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PipelineTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub yaml_content: String,
    pub category: String,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PipelineAnalytics {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_runs: i32,
    pub successful_runs: i32,
    pub failed_runs: i32,
    pub avg_duration_ms: i32,
    pub total_duration_ms: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MultiProjectPipeline {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub project_ids: Vec<Uuid>,
    pub trigger_rules: serde_json::Value,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MultiProjectPipelineRun {
    pub id: Uuid,
    pub pipeline_id: Uuid,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ActivityEvent {
    pub id: i64,
    pub actor_id: Uuid,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub repo_id: Option<Uuid>,
    pub org_id: Option<Uuid>,
    pub description: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EmailVerificationCode {
    pub id: Uuid,
    pub user_id: Uuid,
    pub code: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Release {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub tag_name: String,
    pub name: String,
    pub body: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub author_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReleaseAsset {
    pub id: Uuid,
    pub release_id: Uuid,
    pub name: String,
    pub content_type: String,
    pub size: i64,
    pub download_count: i64,
    pub author_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IssueTemplate {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PrTemplate {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub title: String,
    pub body: String,
    pub base_branch: String,
    pub labels: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Discussion {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub title: String,
    pub body: String,
    pub category: String,
    pub author_id: Uuid,
    pub is_pinned: bool,
    pub is_locked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DiscussionComment {
    pub id: Uuid,
    pub discussion_id: Uuid,
    pub author_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BranchProtectionRule {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub branch_pattern: String,
    pub require_pull_request: bool,
    pub required_approving_reviews: i32,
    pub required_status_checks: Vec<String>,
    pub enforce_admins: bool,
    pub allow_force_pushes: bool,
    pub allow_deletions: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Team {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub description: String,
    pub privacy: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TeamMember {
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReviewSummary {
    pub pr_id: Uuid,
    pub approvals: i64,
    pub changes_requested: i64,
    pub comments: i64,
    pub codeowners_approved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReviewAssignment {
    pub id: Uuid,
    pub pr_id: Uuid,
    pub user_id: Uuid,
    pub team: String,
    pub assigned_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WebAuthnCredential {
    pub id: Uuid,
    pub user_id: Uuid,
    pub credential_id: Vec<u8>,
    pub public_key: Vec<u8>,
    pub counter: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BoardCardLabel {
    pub id: uuid::Uuid,
    pub card_id: uuid::Uuid,
    pub label: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BoardCardAssignee {
    pub id: uuid::Uuid,
    pub card_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NpmPackage {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub version: String,
    pub description: String,
    pub dist_tags: serde_json::Value,
    pub readme: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NpmVersion {
    pub id: Uuid,
    pub package_id: Uuid,
    pub version: String,
    pub tarball_url: String,
    pub shasum: String,
    pub integrity: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MavenPackage {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub packaging: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PagesSite {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub url: String,
    pub branch: String,
    pub path: String,
    pub public: bool,
    pub custom_domain: Option<String>,
    pub https_enabled: bool,
    pub last_built_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PagesDeployment {
    pub id: Uuid,
    pub site_id: Uuid,
    pub sha: String,
    pub url: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DiscussionLabel {
    pub id: Uuid,
    pub discussion_id: Uuid,
    pub label: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DiscussionReaction {
    pub id: Uuid,
    pub comment_id: Uuid,
    pub user_id: Uuid,
    pub emoji: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FeatureFlag {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub enabled_for_users: Vec<Uuid>,
    pub enabled_for_percentage: i32,
    pub enabled_for_orgs: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FeatureFlagEvent {
    pub id: Uuid,
    pub flag_id: Uuid,
    pub user_id: Option<Uuid>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AdminDashboardConfig {
    pub id: Uuid,
    pub widget_name: String,
    pub widget_config: serde_json::Value,
    pub position: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_serialization() {
        let user = User {
            id: Uuid::nil(),
            username: "alice".into(),
            email: "alice@example.com".into(),
            display_name: "Alice Smith".into(),
            bio: "Developer".into(),
            role: "admin".into(),
            email_verified: true,
            banned: false,
            avatar_url: None,
            location: None,
            website: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&user).unwrap();
        let de: User = serde_json::from_str(&json).unwrap();
        assert_eq!(de.username, "alice");
        assert_eq!(de.role, "admin");
        assert!(de.email_verified);
    }

    #[test]
    fn test_repository_serialization() {
        let repo = Repository {
            id: Uuid::nil(),
            name: "test-repo".into(),
            description: "A test repo".into(),
            owner_id: Uuid::nil(),
            org_id: None,
            visibility: "public".into(),
            default_branch: "main".into(),
            is_fork: false,
            parent_repo_id: None,
            stars_count: 0,
            watchers_count: 0,
            archived: false,
            topics: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&repo).unwrap();
        let de: Repository = serde_json::from_str(&json).unwrap();
        assert!(!de.is_fork);
        assert_eq!(de.visibility, "public");
    }

    #[test]
    fn test_issue_serialization() {
        let issue = Issue {
            id: Uuid::nil(),
            repo_id: Uuid::nil(),
            number: 42,
            title: "Bug fix".into(),
            body: "Something broke".into(),
            status: "open".into(),
            author_id: Uuid::nil(),
            assignee_id: None,
            labels: vec!["bug".into(), "critical".into()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            closed_at: None,
        };
        let json = serde_json::to_string(&issue).unwrap();
        let de: Issue = serde_json::from_str(&json).unwrap();
        assert_eq!(de.number, 42);
        assert_eq!(de.labels.len(), 2);
    }

    #[test]
    fn test_pr_serialization() {
        let pr = PullRequest {
            id: Uuid::nil(),
            repo_id: Uuid::nil(),
            number: 1,
            title: "Add feature".into(),
            body: "Description".into(),
            status: "open".into(),
            author_id: Uuid::nil(),
            source_branch: "feature".into(),
            target_branch: "main".into(),
            merge_commit_id: None,
            draft: false,
            auto_merge: false,
            head_commit_sha: None,
            base_commit_sha: None,
            merge_strategy: "merge".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            merged_at: None,
            closed_at: None,
        };
        let json = serde_json::to_string(&pr).unwrap();
        let de: PullRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(de.source_branch, "feature");
        assert_eq!(de.target_branch, "main");
    }

    #[test]
    fn test_ssh_key_serialization() {
        let key = SshKey {
            id: Uuid::nil(),
            user_id: Uuid::nil(),
            key_type: "ssh-ed25519".into(),
            public_key: "AAAAC3NzaC1lZDI1NTE5AAAAI...".into(),
            fingerprint: "SHA256:abc123".into(),
            label: "my-laptop".into(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&key).unwrap();
        let de: SshKey = serde_json::from_str(&json).unwrap();
        assert_eq!(de.key_type, "ssh-ed25519");
        assert_eq!(de.fingerprint, "SHA256:abc123");
        assert_eq!(de.label, "my-laptop");
    }

    #[test]
    fn test_release_serialization() {
        let release = Release {
            id: Uuid::nil(),
            repo_id: Uuid::nil(),
            tag_name: "v1.0.0".into(),
            name: "Release 1.0".into(),
            body: Some("First release".into()),
            draft: false,
            prerelease: false,
            author_id: Uuid::nil(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            published_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&release).unwrap();
        let de: Release = serde_json::from_str(&json).unwrap();
        assert_eq!(de.tag_name, "v1.0.0");
        assert!(!de.draft);
    }

    #[test]
    fn test_release_asset_serialization() {
        let asset = ReleaseAsset {
            id: Uuid::nil(),
            release_id: Uuid::nil(),
            name: "binary.tar.gz".into(),
            content_type: "application/gzip".into(),
            size: 1024,
            download_count: 0,
            author_id: Uuid::nil(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&asset).unwrap();
        let de: ReleaseAsset = serde_json::from_str(&json).unwrap();
        assert_eq!(de.name, "binary.tar.gz");
        assert_eq!(de.size, 1024);
    }

    #[test]
    fn test_team_serialization() {
        let team = Team {
            id: Uuid::nil(),
            org_id: Uuid::nil(),
            name: "backend".into(),
            description: "Backend team".into(),
            privacy: "visible".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&team).unwrap();
        let de: Team = serde_json::from_str(&json).unwrap();
        assert_eq!(de.name, "backend");
        assert_eq!(de.privacy, "visible");
    }

    #[test]
    fn test_team_member_serialization() {
        let member = TeamMember {
            team_id: Uuid::nil(),
            user_id: Uuid::nil(),
            role: "maintainer".into(),
            joined_at: Utc::now(),
        };
        let json = serde_json::to_string(&member).unwrap();
        let de: TeamMember = serde_json::from_str(&json).unwrap();
        assert_eq!(de.role, "maintainer");
    }

    #[test]
    fn test_pr_template_serialization() {
        let tmpl = PrTemplate {
            id: Uuid::nil(),
            repo_id: Uuid::nil(),
            name: "Feature".into(),
            title: "feat: ".into(),
            body: "## Description".into(),
            base_branch: "main".into(),
            labels: vec!["feature".into()],
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&tmpl).unwrap();
        let de: PrTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(de.name, "Feature");
        assert_eq!(de.base_branch, "main");
    }

    #[test]
    fn test_discussion_serialization() {
        let disc = Discussion {
            id: Uuid::nil(),
            repo_id: Uuid::nil(),
            title: "RFC: New API".into(),
            body: "Proposal".into(),
            category: "rfc".into(),
            author_id: Uuid::nil(),
            is_pinned: true,
            is_locked: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&disc).unwrap();
        let de: Discussion = serde_json::from_str(&json).unwrap();
        assert_eq!(de.title, "RFC: New API");
        assert!(de.is_pinned);
    }

    #[test]
    fn test_discussion_comment_serialization() {
        let cmt = DiscussionComment {
            id: Uuid::nil(),
            discussion_id: Uuid::nil(),
            author_id: Uuid::nil(),
            body: "Great idea!".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&cmt).unwrap();
        let de: DiscussionComment = serde_json::from_str(&json).unwrap();
        assert_eq!(de.body, "Great idea!");
    }

    #[test]
    fn test_branch_protection_rule_serialization() {
        let rule = BranchProtectionRule {
            id: Uuid::nil(),
            repo_id: Uuid::nil(),
            branch_pattern: "main".into(),
            require_pull_request: true,
            required_approving_reviews: 2,
            required_status_checks: vec!["ci/test".into()],
            enforce_admins: true,
            allow_force_pushes: false,
            allow_deletions: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&rule).unwrap();
        let de: BranchProtectionRule = serde_json::from_str(&json).unwrap();
        assert_eq!(de.branch_pattern, "main");
        assert!(de.require_pull_request);
        assert_eq!(de.required_approving_reviews, 2);
        assert!(de.enforce_admins);
    }

    #[test]
    fn test_test_coverage_serialization() {
        let coverage = TestCoverage {
            id: Uuid::nil(),
            repo_id: Uuid::nil(),
            file_path: "src/main.rs".into(),
            line_coverage: 85.5,
            branch_coverage: 72.3,
            function_coverage: 90.0,
            total_lines: 200,
            covered_lines: 171,
            measured_at: Utc::now(),
        };
        let json = serde_json::to_string(&coverage).unwrap();
        let de: TestCoverage = serde_json::from_str(&json).unwrap();
        assert_eq!(de.file_path, "src/main.rs");
        assert!((de.line_coverage - 85.5).abs() < f64::EPSILON);
        assert_eq!(de.total_lines, 200);
    }

    #[test]
    fn test_code_quality_metric_serialization() {
        let metric = CodeQualityMetric {
            id: Uuid::nil(),
            repo_id: Uuid::nil(),
            metric_name: "cyclomatic_complexity".into(),
            metric_value: 12.5,
            file_path: Some("src/lib.rs".into()),
            measured_at: Utc::now(),
        };
        let json = serde_json::to_string(&metric).unwrap();
        let de: CodeQualityMetric = serde_json::from_str(&json).unwrap();
        assert_eq!(de.metric_name, "cyclomatic_complexity");
        assert!((de.metric_value - 12.5).abs() < f64::EPSILON);
        assert_eq!(de.file_path.as_deref(), Some("src/lib.rs"));
    }

    #[test]
    fn test_performance_test_serialization() {
        let test = PerformanceTest {
            id: Uuid::nil(),
            repo_id: Uuid::nil(),
            name: "load test api".into(),
            test_type: "load".into(),
            endpoint: Some("/api/v1/users".into()),
            config: serde_json::json!({"concurrent_users": 100, "duration_seconds": 60}),
            status: "completed".into(),
            results: serde_json::json!({"avg_response_ms": 150, "p95_response_ms": 300, "error_rate": 0.01}),
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&test).unwrap();
        let de: PerformanceTest = serde_json::from_str(&json).unwrap();
        assert_eq!(de.test_type, "load");
        assert_eq!(de.status, "completed");
        assert!(de.completed_at.is_some());
    }

    #[test]
    fn test_database_backup_serialization() {
        let backup = DatabaseBackup {
            id: Uuid::nil(),
            backup_type: "full".into(),
            status: "completed".into(),
            file_path: Some("/backups/db_001.dump".into()),
            file_size_bytes: 1024000,
            checksum: Some("sha256:abc123".into()),
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&backup).unwrap();
        let de: DatabaseBackup = serde_json::from_str(&json).unwrap();
        assert_eq!(de.backup_type, "full");
        assert_eq!(de.status, "completed");
        assert_eq!(de.file_size_bytes, 1024000);
    }

    #[test]
    fn test_database_recovery_point_serialization() {
        let point = DatabaseRecoveryPoint {
            id: Uuid::nil(),
            backup_id: Uuid::nil(),
            name: "pre-deploy-1.0".into(),
            description: "Recovery point before v1.0 deploy".into(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&point).unwrap();
        let de: DatabaseRecoveryPoint = serde_json::from_str(&json).unwrap();
        assert_eq!(de.name, "pre-deploy-1.0");
        assert!(de.description.contains("v1.0"));
    }

    #[test]
    fn test_data_archive_serialization() {
        let archive = DataArchive {
            id: Uuid::nil(),
            repo_id: Uuid::nil(),
            archive_type: "code".into(),
            status: "archived".into(),
            file_path: Some("/archives/repo.tar.gz".into()),
            file_size_bytes: 52428800,
            retention_days: 365,
            created_at: Utc::now(),
            expires_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&archive).unwrap();
        let de: DataArchive = serde_json::from_str(&json).unwrap();
        assert_eq!(de.archive_type, "code");
        assert_eq!(de.retention_days, 365);
    }

    #[test]
    fn test_data_migration_serialization() {
        let migration = DataMigration {
            id: Uuid::nil(),
            source: "postgres://old-host/repo".into(),
            destination: "postgres://new-host/repo".into(),
            migration_type: "full".into(),
            status: "in_progress".into(),
            progress: 45.5,
            started_at: Utc::now(),
            completed_at: None,
        };
        let json = serde_json::to_string(&migration).unwrap();
        let de: DataMigration = serde_json::from_str(&json).unwrap();
        assert_eq!(de.source, "postgres://old-host/repo");
        assert!((de.progress - 45.5).abs() < f64::EPSILON);
        assert!(de.completed_at.is_none());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiAnalytic {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: i32,
    pub user_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_size_bytes: i32,
    pub response_size_bytes: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiUsageSummary {
    pub id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_requests: i32,
    pub total_errors: i32,
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub unique_users: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UsageQuota {
    pub id: Uuid,
    pub user_id: Uuid,
    pub quota_type: String,
    pub quota_limit: i32,
    pub quota_used: i32,
    pub period_start: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DeploymentHistory {
    pub id: Uuid,
    pub environment_id: Uuid,
    pub version: String,
    pub sha: String,
    pub status: String,
    pub deployed_by: Uuid,
    pub rollback_of: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MonitoringAlert {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub alert_type: String,
    pub condition: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MonitoringIncident {
    pub id: Uuid,
    pub alert_id: Uuid,
    pub severity: String,
    pub message: String,
    pub status: String,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PerformanceMetric {
    pub id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub labels: serde_json::Value,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiDocumentation {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub summary: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub request_body: Option<serde_json::Value>,
    pub responses: serde_json::Value,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiVersion {
    pub id: Uuid,
    pub version: String,
    pub status: String,
    pub release_date: DateTime<Utc>,
    pub deprecation_date: Option<DateTime<Utc>>,
    pub sunset_date: Option<DateTime<Utc>>,
    pub changelog: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiAnalyticV2 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: i32,
    pub user_id: Option<Uuid>,
    pub request_size_bytes: i32,
    pub response_size_bytes: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TestCoverage {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub file_path: String,
    pub line_coverage: f64,
    pub branch_coverage: f64,
    pub function_coverage: f64,
    pub total_lines: i32,
    pub covered_lines: i32,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CodeQualityMetric {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub file_path: Option<String>,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PerformanceTest {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub test_type: String,
    pub endpoint: Option<String>,
    pub config: serde_json::Value,
    pub status: String,
    pub results: serde_json::Value,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DatabaseBackup {
    pub id: Uuid,
    pub backup_type: String,
    pub status: String,
    pub file_path: Option<String>,
    pub file_size_bytes: i64,
    pub checksum: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DatabaseRecoveryPoint {
    pub id: Uuid,
    pub backup_id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DataArchive {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub archive_type: String,
    pub status: String,
    pub file_path: Option<String>,
    pub file_size_bytes: i64,
    pub retention_days: i32,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DataMigration {
    pub id: Uuid,
    pub source: String,
    pub destination: String,
    pub migration_type: String,
    pub status: String,
    pub progress: f64,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Workflow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub trigger_type: String,
    pub trigger_config: serde_json::Value,
    pub steps: serde_json::Value,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowRun {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub status: String,
    pub current_step: i32,
    pub total_steps: i32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AutomationRule {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: serde_json::Value,
    pub actions: serde_json::Value,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScheduledTask {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub cron_expression: String,
    pub task_type: String,
    pub task_config: serde_json::Value,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiDocsV2 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub request_body: Option<serde_json::Value>,
    pub responses: serde_json::Value,
    pub examples: serde_json::Value,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitTier {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    pub price_cents: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiAnalyticV3 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: i32,
    pub user_id: Option<Uuid>,
    pub request_size_bytes: i32,
    pub response_size_bytes: i32,
    pub cache_hit: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DatabaseReplica {
    pub id: Uuid,
    pub name: String,
    pub host: String,
    pub port: i32,
    pub status: String,
    pub lag_ms: i32,
    pub last_sync_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EncryptionPolicy {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub data_types: Vec<String>,
    pub algorithm: String,
    pub key_rotation_days: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DataResidencyRule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub data_types: Vec<String>,
    pub allowed_regions: Vec<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DataResidencyViolation {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub data_type: String,
    pub data_id: Uuid,
    pub region: String,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiDocsV3 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub request_body: Option<serde_json::Value>,
    pub responses: serde_json::Value,
    pub examples: serde_json::Value,
    pub tags: Vec<String>,
    pub deprecated: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiWebhookV2 {
    pub id: Uuid,
    pub url: String,
    pub secret: String,
    pub events: Vec<String>,
    pub active: bool,
    pub config: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiWebhookDeliveryV2 {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub attempts: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiAnalyticV4 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: i32,
    pub user_id: Option<Uuid>,
    pub request_size_bytes: i32,
    pub response_size_bytes: i32,
    pub cache_hit: bool,
    pub region: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiDocsV4 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub request_body: Option<serde_json::Value>,
    pub responses: serde_json::Value,
    pub examples: serde_json::Value,
    pub tags: Vec<String>,
    pub deprecated: bool,
    pub changelog: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitTierV2 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    pub price_cents: i32,
    pub features: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitUsageV2 {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub usage_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiAnalyticV5 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: i32,
    pub user_id: Option<Uuid>,
    pub request_size_bytes: i32,
    pub response_size_bytes: i32,
    pub cache_hit: bool,
    pub region: String,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiDocsV5 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub request_body: Option<serde_json::Value>,
    pub responses: serde_json::Value,
    pub examples: serde_json::Value,
    pub tags: Vec<String>,
    pub deprecated: bool,
    pub changelog: String,
    pub security_schemes: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitTierV3 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    pub price_cents: i32,
    pub features: serde_json::Value,
    pub limits: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitOverage {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub overage_count: i32,
    pub overage_cost_cents: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitAlert {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier_id: Uuid,
    pub alert_type: String,
    pub threshold_percent: i32,
    pub current_usage: i32,
    pub triggered_at: DateTime<Utc>,
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiAnalyticV6 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: i32,
    pub user_id: Option<Uuid>,
    pub request_size_bytes: i32,
    pub response_size_bytes: i32,
    pub cache_hit: bool,
    pub region: String,
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiAnalyticsCorrelation {
    pub id: Uuid,
    pub request_id: Uuid,
    pub parent_request_id: Option<Uuid>,
    pub correlation_type: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiAnalyticsCapacityPlan {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub current_rps: i32,
    pub projected_rps: i32,
    pub capacity_limit: i32,
    pub utilization_percent: f64,
    pub last_calculated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiDocsV6 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub request_body: Option<serde_json::Value>,
    pub responses: serde_json::Value,
    pub examples: serde_json::Value,
    pub tags: Vec<String>,
    pub deprecated: bool,
    pub changelog: String,
    pub security_schemes: serde_json::Value,
    pub rate_limits: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitTierV4 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    pub price_cents: i32,
    pub features: serde_json::Value,
    pub limits: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitAlertV2 {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiAnalyticV7 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: i32,
    pub user_id: Option<Uuid>,
    pub request_size_bytes: i32,
    pub response_size_bytes: i32,
    pub cache_hit: bool,
    pub region: String,
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub cost_cents: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiDocsV7 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub request_body: Option<serde_json::Value>,
    pub responses: serde_json::Value,
    pub examples: serde_json::Value,
    pub tags: Vec<String>,
    pub deprecated: bool,
    pub changelog: String,
    pub security_schemes: serde_json::Value,
    pub rate_limits: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitTierV5 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    pub price_cents: i32,
    pub features: serde_json::Value,
    pub limits: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitAlertV3 {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiAnalyticV8 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: i32,
    pub user_id: Option<Uuid>,
    pub request_size_bytes: i32,
    pub response_size_bytes: i32,
    pub cache_hit: bool,
    pub region: String,
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub cost_cents: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiDocsV8 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub request_body: Option<serde_json::Value>,
    pub responses: serde_json::Value,
    pub examples: serde_json::Value,
    pub tags: Vec<String>,
    pub deprecated: bool,
    pub changelog: String,
    pub security_schemes: serde_json::Value,
    pub rate_limits: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitTierV6 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    pub price_cents: i32,
    pub features: serde_json::Value,
    pub limits: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitAlertV4 {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiAnalyticV9 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: i32,
    pub user_id: Option<Uuid>,
    pub request_size_bytes: i32,
    pub response_size_bytes: i32,
    pub cache_hit: bool,
    pub region: String,
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub cost_cents: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiDocsV10 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub request_body: Option<serde_json::Value>,
    pub responses: serde_json::Value,
    pub examples: serde_json::Value,
    pub tags: Vec<String>,
    pub deprecated: bool,
    pub changelog: String,
    pub security_schemes: serde_json::Value,
    pub rate_limits: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitTierV8 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    pub price_cents: i32,
    pub features: serde_json::Value,
    pub limits: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiAnalyticV11 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: i32,
    pub user_id: Option<Uuid>,
    pub request_size_bytes: i32,
    pub response_size_bytes: i32,
    pub cache_hit: bool,
    pub region: String,
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub cost_cents: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PipelineActionReviewV4 {
    pub id: Uuid,
    pub action_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub review: String,
    pub helpful_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReviewHelpfulnessV3 {
    pub id: Uuid,
    pub review_id: Uuid,
    pub user_id: Uuid,
    pub helpful: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReviewModerationQueueV3 {
    pub id: Uuid,
    pub review_id: Uuid,
    pub status: String,
    pub moderator_id: Option<Uuid>,
    pub reason: Option<String>,
    pub moderated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReviewAnalyticsV3 {
    pub id: Uuid,
    pub action_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub total_reviews: i32,
    pub avg_rating: f64,
    pub rating_distribution: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReviewRecommendationV3 {
    pub id: Uuid,
    pub action_id: Uuid,
    pub user_id: Uuid,
    pub reason: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EnvironmentDeploymentHistoryV4 {
    pub id: Uuid,
    pub environment_id: Uuid,
    pub version: String,
    pub sha: String,
    pub status: String,
    pub deployed_by: Uuid,
    pub rollback_of: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DeploymentComparisonV4 {
    pub id: Uuid,
    pub from_deployment_id: Uuid,
    pub to_deployment_id: Uuid,
    pub diff_summary: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DeploymentAnalyticsV4 {
    pub id: Uuid,
    pub environment_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub total_deployments: i32,
    pub successful_deployments: i32,
    pub failed_deployments: i32,
    pub avg_deploy_time_ms: i64,
    pub rollback_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CacheHitAnalysisV3 {
    pub id: Uuid,
    pub cache_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub hit_count: i32,
    pub miss_count: i32,
    pub avg_hit_size_bytes: i64,
    pub total_size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CacheSizeTrackingV3 {
    pub id: Uuid,
    pub cache_id: Uuid,
    pub measured_at: DateTime<Utc>,
    pub size_bytes: i64,
    pub item_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CacheCostOptimizationV3 {
    pub id: Uuid,
    pub cache_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub estimated_savings_bytes: i64,
    pub recommended_actions: serde_json::Value,
    pub applied_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CachePerformanceInsightsV3 {
    pub id: Uuid,
    pub cache_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub hit_rate: f64,
    pub avg_hit_latency_ms: i64,
    pub avg_miss_latency_ms: i64,
    pub eviction_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TestSuiteMetricV3 {
    pub id: Uuid,
    pub suite_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TestSuiteBaselineV3 {
    pub id: Uuid,
    pub suite_id: Uuid,
    pub metric_name: String,
    pub baseline_value: f64,
    pub threshold_percent: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CodeQualityMetricV4 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub file_path: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CodeQualityThresholdV3 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub metric_name: String,
    pub threshold_value: f64,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PerformanceTestAlertV4 {
    pub id: Uuid,
    pub baseline_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PerformanceTestAlertHistoryV4 {
    pub id: Uuid,
    pub alert_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TestSuiteMetricV5 {
    pub id: Uuid,
    pub suite_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TestSuiteBaselineV5 {
    pub id: Uuid,
    pub suite_id: Uuid,
    pub metric_name: String,
    pub baseline_value: f64,
    pub threshold_percent: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CodeQualityMetricV6 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub file_path: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CodeQualityThresholdV5 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub metric_name: String,
    pub threshold_value: f64,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PerformanceTestAlertV6 {
    pub id: Uuid,
    pub baseline_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PerformanceTestAlertHistoryV6 {
    pub id: Uuid,
    pub alert_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TestSuiteMetricV6 {
    pub id: Uuid,
    pub suite_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TestSuiteBaselineV6 {
    pub id: Uuid,
    pub suite_id: Uuid,
    pub metric_name: String,
    pub baseline_value: f64,
    pub threshold_percent: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CodeQualityMetricV7 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub file_path: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CodeQualityThresholdV6 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub metric_name: String,
    pub threshold_value: f64,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PerformanceTestAlertV7 {
    pub id: Uuid,
    pub baseline_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PerformanceTestAlertHistoryV7 {
    pub id: Uuid,
    pub alert_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TestSuiteMetricV7 {
    pub id: Uuid,
    pub suite_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TestSuiteBaselineV7 {
    pub id: Uuid,
    pub suite_id: Uuid,
    pub metric_name: String,
    pub baseline_value: f64,
    pub threshold_percent: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TestSuiteMetricV9 {
    pub id: Uuid,
    pub suite_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TestSuiteBaselineV9 {
    pub id: Uuid,
    pub suite_id: Uuid,
    pub metric_name: String,
    pub baseline_value: f64,
    pub threshold_percent: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CodeQualityMetricV8 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub file_path: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CodeQualityThresholdV7 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub metric_name: String,
    pub threshold_value: f64,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CodeQualityMetricV10 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub file_path: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub measured_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CodeQualityThresholdV9 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub metric_name: String,
    pub threshold_value: f64,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PerformanceTestAlertV8 {
    pub id: Uuid,
    pub baseline_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PerformanceTestAlertHistoryV8 {
    pub id: Uuid,
    pub alert_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PerformanceTestAlertV10 {
    pub id: Uuid,
    pub baseline_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PerformanceTestAlertHistoryV10 {
    pub id: Uuid,
    pub alert_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DatabaseReplicationConfigV4 {
    pub id: Uuid,
    pub replica_id: Uuid,
    pub config_key: String,
    pub config_value: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DatabaseReplicationAlertV4 {
    pub id: Uuid,
    pub replica_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EncryptionKeyVersionV4 {
    pub id: Uuid,
    pub key_id: Uuid,
    pub version: i32,
    pub key_material: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EncryptionComplianceCheckV4 {
    pub id: Uuid,
    pub check_type: String,
    pub status: String,
    pub findings: serde_json::Value,
    pub score: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DataResidencyReportV4 {
    pub id: Uuid,
    pub report_type: String,
    pub findings: serde_json::Value,
    pub score: i32,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DataResidencyComplianceV4 {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub compliance_status: String,
    pub last_checked_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DatabaseReplicationConfigV8 {
    pub id: Uuid,
    pub replica_id: Uuid,
    pub config_key: String,
    pub config_value: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DatabaseReplicationAlertV8 {
    pub id: Uuid,
    pub replica_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EncryptionKeyVersionV8 {
    pub id: Uuid,
    pub key_id: Uuid,
    pub version: i32,
    pub key_material: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EncryptionComplianceCheckV8 {
    pub id: Uuid,
    pub check_type: String,
    pub status: String,
    pub findings: serde_json::Value,
    pub score: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DataResidencyReportV8 {
    pub id: Uuid,
    pub report_type: String,
    pub findings: serde_json::Value,
    pub score: i32,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DataResidencyComplianceV8 {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub compliance_status: String,
    pub last_checked_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiDocsV9 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub request_body: Option<serde_json::Value>,
    pub responses: serde_json::Value,
    pub examples: serde_json::Value,
    pub tags: Vec<String>,
    pub deprecated: bool,
    pub changelog: String,
    pub security_schemes: serde_json::Value,
    pub rate_limits: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitTierV7 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    pub price_cents: i32,
    pub features: serde_json::Value,
    pub limits: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitAlertV5 {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiAnalyticV10 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: i32,
    pub user_id: Option<Uuid>,
    pub request_size_bytes: i32,
    pub response_size_bytes: i32,
    pub cache_hit: bool,
    pub region: String,
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub cost_cents: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PipelineActionReviewV7 {
    pub id: Uuid,
    pub action_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub review: String,
    pub helpful_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReviewHelpfulnessV7 {
    pub id: Uuid,
    pub review_id: Uuid,
    pub user_id: Uuid,
    pub helpful: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReviewModerationQueueV7 {
    pub id: Uuid,
    pub review_id: Uuid,
    pub status: String,
    pub moderator_id: Option<Uuid>,
    pub reason: Option<String>,
    pub moderated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReviewAnalyticsV7 {
    pub id: Uuid,
    pub action_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub total_reviews: i32,
    pub avg_rating: f64,
    pub rating_distribution: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReviewRecommendationV7 {
    pub id: Uuid,
    pub action_id: Uuid,
    pub user_id: Uuid,
    pub reason: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EnvironmentDeploymentHistoryV7 {
    pub id: Uuid,
    pub environment_id: Uuid,
    pub version: String,
    pub sha: String,
    pub status: String,
    pub deployed_by: Uuid,
    pub rollback_of: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DeploymentComparisonV7 {
    pub id: Uuid,
    pub from_deployment_id: Uuid,
    pub to_deployment_id: Uuid,
    pub diff_summary: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DeploymentAnalyticsV7 {
    pub id: Uuid,
    pub environment_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub total_deployments: i32,
    pub successful_deployments: i32,
    pub failed_deployments: i32,
    pub avg_deploy_time_ms: i64,
    pub rollback_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CacheHitAnalysisV6 {
    pub id: Uuid,
    pub cache_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub hit_count: i32,
    pub miss_count: i32,
    pub avg_hit_size_bytes: i64,
    pub total_size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CacheSizeTrackingV6 {
    pub id: Uuid,
    pub cache_id: Uuid,
    pub measured_at: DateTime<Utc>,
    pub size_bytes: i64,
    pub item_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CacheCostOptimizationV6 {
    pub id: Uuid,
    pub cache_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub estimated_savings_bytes: i64,
    pub recommended_actions: serde_json::Value,
    pub applied_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CachePerformanceInsightsV6 {
    pub id: Uuid,
    pub cache_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub hit_rate: f64,
    pub avg_hit_latency_ms: i64,
    pub avg_miss_latency_ms: i64,
    pub eviction_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiDocsV11 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub request_body: Option<serde_json::Value>,
    pub responses: serde_json::Value,
    pub examples: serde_json::Value,
    pub tags: Vec<String>,
    pub deprecated: bool,
    pub changelog: String,
    pub security_schemes: serde_json::Value,
    pub rate_limits: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitTierV9 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    pub price_cents: i32,
    pub features: serde_json::Value,
    pub limits: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitAlertV6 {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiAnalyticV12 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: i32,
    pub user_id: Option<Uuid>,
    pub request_size_bytes: i32,
    pub response_size_bytes: i32,
    pub cache_hit: bool,
    pub region: String,
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub cost_cents: i32,
    pub created_at: DateTime<Utc>,
}

// --- API Docs v12 ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiDocsV12 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub request_body: Option<serde_json::Value>,
    pub responses: serde_json::Value,
    pub examples: serde_json::Value,
    pub tags: Vec<String>,
    pub deprecated: bool,
    pub changelog: String,
    pub security_schemes: serde_json::Value,
    pub rate_limits: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// --- Rate Limit Tiers v10 ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitTierV10 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    pub price_cents: i32,
    pub features: serde_json::Value,
    pub limits: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// --- Rate Limit Alerts v7 ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitAlertV7 {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// --- API Analytics v13 ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiAnalyticV13 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: i32,
    pub user_id: Option<Uuid>,
    pub request_size_bytes: i32,
    pub response_size_bytes: i32,
    pub cache_hit: bool,
    pub region: String,
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub cost_cents: i32,
    pub created_at: DateTime<Utc>,
}

// --- API Docs v13 ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiDocsV13 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub request_body: Option<serde_json::Value>,
    pub responses: serde_json::Value,
    pub examples: serde_json::Value,
    pub tags: Vec<String>,
    pub deprecated: bool,
    pub changelog: String,
    pub security_schemes: serde_json::Value,
    pub rate_limits: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// --- Rate Limit Tiers v11 ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitTierV11 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    pub price_cents: i32,
    pub features: serde_json::Value,
    pub limits: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// --- Rate Limit Alerts v8 ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitAlertV8 {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// --- API Analytics v14 ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiAnalyticV14 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: i32,
    pub user_id: Option<Uuid>,
    pub request_size_bytes: i32,
    pub response_size_bytes: i32,
    pub cache_hit: bool,
    pub region: String,
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub cost_cents: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowTemplateV8 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowTemplateReviewV7 {
    pub id: Uuid,
    pub template_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub review: String,
    pub helpful_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AutomationRuleV11 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: serde_json::Value,
    pub actions: serde_json::Value,
    pub priority: i32,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub run_count: i32,
    pub success_rate: f64,
    pub avg_execution_time_ms: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScheduledTaskTemplateV8 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub task_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowTemplateV9 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowTemplateReviewV8 {
    pub id: Uuid,
    pub template_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub review: String,
    pub helpful_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AutomationRuleV12 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: serde_json::Value,
    pub actions: serde_json::Value,
    pub priority: i32,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub run_count: i32,
    pub success_rate: f64,
    pub avg_execution_time_ms: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScheduledTaskTemplateV9 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub task_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowTemplateV10 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowTemplateReviewV9 {
    pub id: Uuid,
    pub template_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub review: String,
    pub helpful_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AutomationRuleV13 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: serde_json::Value,
    pub actions: serde_json::Value,
    pub priority: i32,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub run_count: i32,
    pub success_rate: f64,
    pub avg_execution_time_ms: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScheduledTaskTemplateV10 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub task_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowTemplateV11 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub template_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowTemplateReviewV10 {
    pub id: Uuid,
    pub template_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub review: String,
    pub helpful_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AutomationRuleV14 {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub trigger_type: String,
    pub conditions: serde_json::Value,
    pub actions: serde_json::Value,
    pub priority: i32,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub run_count: i32,
    pub success_rate: f64,
    pub avg_execution_time_ms: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScheduledTaskTemplateV11 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub task_type: String,
    pub config: serde_json::Value,
    pub is_public: bool,
    pub author_id: Option<Uuid>,
    pub usage_count: i32,
    pub rating: f64,
    pub created_at: DateTime<Utc>,
}

// --- API Docs v14 ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiDocsV14 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub request_body: Option<serde_json::Value>,
    pub responses: serde_json::Value,
    pub examples: serde_json::Value,
    pub tags: Vec<String>,
    pub deprecated: bool,
    pub changelog: String,
    pub security_schemes: serde_json::Value,
    pub rate_limits: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// --- Rate Limit Tiers v12 ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitTierV12 {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rate_limit: i32,
    pub burst_limit: i32,
    pub monthly_quota: Option<i32>,
    pub price_cents: i32,
    pub features: serde_json::Value,
    pub limits: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// --- Rate Limit Alerts v9 ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitAlertV9 {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier_id: Uuid,
    pub alert_type: String,
    pub threshold: f64,
    pub enabled: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// --- API Analytics v15 ---

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiAnalyticV15 {
    pub id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_time_ms: i32,
    pub user_id: Option<Uuid>,
    pub request_size_bytes: i32,
    pub response_size_bytes: i32,
    pub cache_hit: bool,
    pub region: String,
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub cost_cents: i32,
    pub created_at: DateTime<Utc>,
}
