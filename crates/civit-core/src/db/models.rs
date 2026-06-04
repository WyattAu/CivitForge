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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub merged_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
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
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&user).unwrap();
        let de: User = serde_json::from_str(&json).unwrap();
        assert_eq!(de.username, "alice");
        assert_eq!(de.role, "admin");
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
}
