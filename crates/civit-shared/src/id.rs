//! Typed ID newtypes for compile-time ID confusion prevention.
//!
//! Use these instead of raw `i64` for entity IDs.

use serde::{Deserialize, Serialize};

macro_rules! typed_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub i64);

        impl $name {
            pub const fn new(id: i64) -> Self {
                Self(id)
            }

            pub const fn get(self) -> i64 {
                self.0
            }
        }

        impl From<i64> for $name {
            fn from(v: i64) -> Self {
                Self(v)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

typed_id!(UserId, "Unique user identifier.");
typed_id!(OrgId, "Unique organization identifier.");
typed_id!(RepoId, "Unique repository identifier.");
typed_id!(IssueId, "Unique issue identifier within a repository.");
typed_id!(
    PullRequestId,
    "Unique pull request identifier within a repository."
);
typed_id!(
    PipelineId,
    "Unique pipeline identifier within a repository."
);
typed_id!(PipelineRunId, "Unique pipeline run identifier.");
typed_id!(CommentId, "Unique comment identifier.");
typed_id!(LabelId, "Unique label identifier.");
typed_id!(MilestoneId, "Unique milestone identifier.");
typed_id!(SshKeyId, "Unique SSH key identifier.");
typed_id!(RunnerId, "Unique CI/CD runner identifier.");
typed_id!(WebhookId, "Unique webhook endpoint identifier.");
typed_id!(DeployKeyId, "Unique deploy key identifier.");
typed_id!(ArtifactId, "Unique build artifact identifier.");
typed_id!(OciImageId, "Unique OCI container image identifier.");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_id_serde_roundtrip() {
        let id = UserId::new(42);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "42");
        let back: UserId = serde_json::from_str(&json).unwrap();
        assert_eq!(back, id);
    }

    #[test]
    fn typed_id_equality() {
        let a = RepoId::new(1);
        let b = RepoId::new(1);
        let c = RepoId::new(2);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn typed_id_display() {
        assert_eq!(format!("{}", UserId::new(99)), "99");
    }
}
