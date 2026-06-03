//! Permission system types shared between backend and frontend.
//!
//! This module defines the core permission model:
//! - `Resource` — what is being accessed (org, repo, pipeline, etc.)
//! - `Action` — what operation is being performed (create, read, update, etc.)
//! - `PermissionCheck` — result of a permission evaluation
//! - `BranchProtectionRule` — branch-level access restrictions

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// The type of resource being accessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resource {
    Organization,
    Repository,
    Pipeline,
    PipelineVariable,
    Runner,
    Package,
    Branch,
    Tag,
    Issue,
    Wiki,
    User,
}

impl Resource {
    /// Returns the snake_case string representation for database storage.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Organization => "organization",
            Self::Repository => "repository",
            Self::Pipeline => "pipeline",
            Self::PipelineVariable => "pipeline_variable",
            Self::Runner => "runner",
            Self::Package => "package",
            Self::Branch => "branch",
            Self::Tag => "tag",
            Self::Issue => "issue",
            Self::Wiki => "wiki",
            Self::User => "user",
        }
    }
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

/// The operation being performed on a resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Create,
    Read,
    Update,
    Delete,
    Administer,
    Transfer,
    Fork,
    Push,
    ForcePush,
    Merge,
    Rebase,
    TriggerPipeline,
    CancelPipeline,
    ManageVariables,
    ManageWebhooks,
    ManageMembers,
    ManageRunners,
    DownloadArtifact,
    PublishPackage,
}

impl Action {
    /// Returns the snake_case string representation for database storage.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Read => "read",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::Administer => "administer",
            Self::Transfer => "transfer",
            Self::Fork => "fork",
            Self::Push => "push",
            Self::ForcePush => "force_push",
            Self::Merge => "merge",
            Self::Rebase => "rebase",
            Self::TriggerPipeline => "trigger_pipeline",
            Self::CancelPipeline => "cancel_pipeline",
            Self::ManageVariables => "manage_variables",
            Self::ManageWebhooks => "manage_webhooks",
            Self::ManageMembers => "manage_members",
            Self::ManageRunners => "manage_runners",
            Self::DownloadArtifact => "download_artifact",
            Self::PublishPackage => "publish_package",
        }
    }
}

// ---------------------------------------------------------------------------
// Permission Check Result
// ---------------------------------------------------------------------------

/// Result of a permission evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCheck {
    pub allowed: bool,
    pub resource: Resource,
    pub action: Action,
    pub reason: Option<String>,
}

impl PermissionCheck {
    /// Create an allowed result.
    pub fn allowed(resource: Resource, action: Action) -> Self {
        Self {
            allowed: true,
            resource,
            action,
            reason: None,
        }
    }

    /// Create a denied result with reason.
    pub fn denied(resource: Resource, action: Action, reason: impl Into<String>) -> Self {
        Self {
            allowed: false,
            resource,
            action,
            reason: Some(reason.into()),
        }
    }
}

// ---------------------------------------------------------------------------
// Branch Protection
// ---------------------------------------------------------------------------

/// A rule protecting a branch from unauthorized operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchProtectionRule {
    pub id: uuid::Uuid,
    pub repo_id: crate::id::RepoId,
    /// Glob pattern for matching branches (e.g., "main", "release/*").
    pub pattern: String,
    /// Whether pushes are restricted to specific roles.
    pub push_restricted: bool,
    /// Roles allowed to push when restricted (empty = maintainers+).
    pub allowed_roles: Vec<String>,
    /// Number of approvals required to merge (None = no requirement).
    pub required_approvals: Option<u32>,
    /// Whether CI must pass before merge.
    pub require_ci: bool,
    /// Whether force-push is allowed.
    pub force_push_allowed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Evaluation of whether a push operation is allowed against branch rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushEvaluation {
    pub rule_id: Option<uuid::Uuid>,
    pub violations: Vec<String>,
}

// ---------------------------------------------------------------------------
// Pipeline Variables
// ---------------------------------------------------------------------------

/// An encrypted CI/CD variable stored per-repo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineVariableResponse {
    pub id: uuid::Uuid,
    pub repo_id: crate::id::RepoId,
    pub name: String,
    /// Whether the value is masked in logs.
    pub masked: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Request to create/update a pipeline variable.
#[derive(Debug, Deserialize)]
pub struct UpsertPipelineVariableRequest {
    pub name: String,
    pub value: String,
    pub masked: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_check_allowed() {
        let check = PermissionCheck::allowed(Resource::Repository, Action::Read);
        assert!(check.allowed);
        assert!(check.reason.is_none());
    }

    #[test]
    fn permission_check_denied() {
        let check = PermissionCheck::denied(
            Resource::Repository,
            Action::Delete,
            "Guest role cannot delete repos",
        );
        assert!(!check.allowed);
        assert_eq!(
            check.reason.as_deref(),
            Some("Guest role cannot delete repos")
        );
    }
}
