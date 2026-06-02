//! Repository visibility levels.

use serde::{Deserialize, Serialize};

/// Repository visibility — controls who can see the repo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// Anyone can see the repo, including anonymous users.
    Public,
    /// Only authenticated users who are members of the org can see it.
    Internal,
    /// Only explicitly authorized users can see it.
    Private,
}

impl Visibility {
    /// Returns true if anonymous (unauthenticated) users can access the repo.
    pub const fn is_public(self) -> bool {
        matches!(self, Visibility::Public)
    }
}

impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Visibility::Public => write!(f, "public"),
            Visibility::Internal => write!(f, "internal"),
            Visibility::Private => write!(f, "private"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip() {
        for v in [
            Visibility::Public,
            Visibility::Internal,
            Visibility::Private,
        ] {
            let json = serde_json::to_string(&v).unwrap();
            let back: Visibility = serde_json::from_str(&json).unwrap();
            assert_eq!(v, back);
        }
    }

    #[test]
    fn display() {
        assert_eq!(Visibility::Public.to_string(), "public");
        assert_eq!(Visibility::Internal.to_string(), "internal");
        assert_eq!(Visibility::Private.to_string(), "private");
    }

    #[test]
    fn is_public() {
        assert!(Visibility::Public.is_public());
        assert!(!Visibility::Private.is_public());
    }
}
