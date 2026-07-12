//! Repository domain types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::RepoId;
use crate::id::UserId;
use crate::visibility::Visibility;

/// Repository representation for API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoResponse {
    pub id: RepoId,
    pub name: String,
    pub full_name: String, // "owner/repo"
    pub description: Option<String>,
    pub visibility: Visibility,
    pub owner_id: UserId,
    pub org_id: Option<i64>,
    pub default_branch: String,
    pub is_fork: bool,
    pub parent_repo_id: Option<RepoId>,
    pub ssh_clone_url: Option<String>,
    pub http_clone_url: Option<String>,
    #[serde(default)]
    pub starred: Option<bool>,
    #[serde(default)]
    pub watched: Option<bool>,
    #[serde(default)]
    pub stars_count: Option<i64>,
    #[serde(default)]
    pub watchers_count: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response from toggling star status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarToggleResponse {
    pub starred: bool,
    pub stars_count: i64,
}

/// Response from toggling watch status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchToggleResponse {
    pub watched: bool,
    pub watchers_count: i64,
}

/// Request to create a new repository.
#[derive(Debug, Deserialize)]
pub struct CreateRepoRequest {
    pub name: String,
    pub description: Option<String>,
    pub visibility: Option<Visibility>,
    pub org_id: Option<i64>,
    pub default_branch: Option<String>,
    pub initialize: Option<bool>, // create initial commit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_repo_defaults() {
        let req = CreateRepoRequest {
            name: "test-repo".into(),
            description: None,
            visibility: None,
            org_id: None,
            default_branch: None,
            initialize: None,
        };
        assert_eq!(req.name, "test-repo");
        assert!(req.description.is_none());
    }
}
