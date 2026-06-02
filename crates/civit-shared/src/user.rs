//! User domain types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::UserId;

/// User role within an organization or repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    /// Full control, can delete the resource.
    Owner,
    /// Full management, cannot delete owner resources.
    Admin,
    /// Manage settings, merge PRs, manage CI variables.
    Maintainer,
    /// Push to non-protected branches, create PRs.
    Developer,
    /// Read-only + comment on issues/PRs.
    Reporter,
    /// Limited read (public repos only).
    Guest,
}

impl UserRole {
    /// Returns the numeric rank (higher = more permissions).
    pub const fn rank(&self) -> u8 {
        match self {
            UserRole::Owner => 60,
            UserRole::Admin => 50,
            UserRole::Maintainer => 40,
            UserRole::Developer => 30,
            UserRole::Reporter => 20,
            UserRole::Guest => 10,
        }
    }

    /// Whether this role has write access.
    pub const fn can_write(&self) -> bool {
        self.rank() >= UserRole::Developer.rank()
    }

    /// Whether this role has admin access.
    pub const fn can_admin(&self) -> bool {
        self.rank() >= UserRole::Admin.rank()
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Owner => write!(f, "owner"),
            UserRole::Admin => write!(f, "admin"),
            UserRole::Maintainer => write!(f, "maintainer"),
            UserRole::Developer => write!(f, "developer"),
            UserRole::Reporter => write!(f, "reporter"),
            UserRole::Guest => write!(f, "guest"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "owner" => Ok(Self::Owner),
            "admin" => Ok(Self::Admin),
            "maintainer" => Ok(Self::Maintainer),
            "developer" => Ok(Self::Developer),
            "reporter" => Ok(Self::Reporter),
            "guest" => Ok(Self::Guest),
            _ => Err(format!("unknown role: '{s}'")),
        }
    }
}

/// User representation for API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new user.
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub password: String,
}

/// Request to update a user profile.
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub email: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_ranking() {
        assert!(UserRole::Owner.rank() > UserRole::Admin.rank());
        assert!(UserRole::Admin.rank() > UserRole::Maintainer.rank());
        assert!(UserRole::Maintainer.rank() > UserRole::Developer.rank());
        assert!(UserRole::Developer.rank() > UserRole::Reporter.rank());
        assert!(UserRole::Reporter.rank() > UserRole::Guest.rank());
    }

    #[test]
    fn role_permissions() {
        assert!(UserRole::Owner.can_write());
        assert!(UserRole::Developer.can_write());
        assert!(!UserRole::Reporter.can_write());
        assert!(UserRole::Admin.can_admin());
        assert!(!UserRole::Maintainer.can_admin());
    }

    #[test]
    fn role_serde() {
        for role in [
            UserRole::Owner,
            UserRole::Admin,
            UserRole::Maintainer,
            UserRole::Developer,
            UserRole::Reporter,
            UserRole::Guest,
        ] {
            let json = serde_json::to_string(&role).unwrap();
            let back: UserRole = serde_json::from_str(&json).unwrap();
            assert_eq!(role, back);
        }
    }
}
