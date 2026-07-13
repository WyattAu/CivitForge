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
}
