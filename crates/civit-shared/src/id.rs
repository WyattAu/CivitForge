//! Typed ID newtypes for compile-time ID confusion prevention.
//!
//! Use these instead of raw `uuid::Uuid` for entity IDs.

use serde::{Deserialize, Serialize};

macro_rules! typed_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub uuid::Uuid);

        impl $name {
            pub const fn new(id: uuid::Uuid) -> Self {
                Self(id)
            }

            pub fn get(&self) -> uuid::Uuid {
                self.0
            }

            pub fn nil() -> Self {
                Self(uuid::Uuid::nil())
            }
        }

        impl From<uuid::Uuid> for $name {
            fn from(v: uuid::Uuid) -> Self {
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
        let id = UserId::new(uuid::Uuid::nil());
        let json = serde_json::to_string(&id).unwrap();
        let back: UserId = serde_json::from_str(&json).unwrap();
        assert_eq!(back, id);
    }

    #[test]
    fn typed_id_equality() {
        let a = RepoId::new(uuid::Uuid::nil());
        let b = RepoId::new(uuid::Uuid::nil());
        let c = RepoId::new(uuid::Uuid::new_v4());
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn typed_id_display() {
        let id = UserId::new(uuid::Uuid::nil());
        assert_eq!(format!("{id}"), "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn typed_id_nil() {
        assert_eq!(UserId::nil().get(), uuid::Uuid::nil());
    }
}
