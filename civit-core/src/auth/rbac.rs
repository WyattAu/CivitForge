#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    Admin,
    Member,
    Guest,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Member => "member",
            Self::Guest => "guest",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(Self::Admin),
            "member" => Some(Self::Member),
            "guest" => Some(Self::Guest),
            _ => None,
        }
    }

    pub fn role_str(&self) -> &'static str {
        self.as_str()
    }

    pub fn permissions(&self) -> HashSet<Permission> {
        match self {
            Self::Admin => {
                let mut perms = Self::Member.permissions();
                perms.insert(Permission::ManageUsers);
                perms.insert(Permission::ManageOrg);
                perms.insert(Permission::AdminSettings);
                perms
            }
            Self::Member => {
                let mut perms = Self::Guest.permissions();
                perms.insert(Permission::CreateRepo);
                perms.insert(Permission::Push);
                perms.insert(Permission::CreateIssue);
                perms.insert(Permission::CreatePR);
                perms.insert(Permission::ManagePipeline);
                perms
            }
            Self::Guest => {
                let mut perms = HashSet::new();
                perms.insert(Permission::ReadRepo);
                perms.insert(Permission::ViewIssue);
                perms.insert(Permission::ViewPR);
                perms
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Read,
    Write,
    Delete,
    Admin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    ReadRepo,
    CreateRepo,
    Push,
    ViewIssue,
    CreateIssue,
    ViewPR,
    CreatePR,
    ManagePipeline,
    ManageUsers,
    ManageOrg,
    AdminSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub resource: String,
    pub actions: Vec<Action>,
    pub roles: Vec<Role>,
    pub conditions: Option<PolicyConditions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConditions {
    pub owner_only: bool,
    pub org_scoped: bool,
    pub public_only: bool,
}

#[derive(Debug, Clone)]
pub struct PolicyEngine {
    policies: Vec<Policy>,
}

impl PolicyEngine {
    pub fn new(policies: Vec<Policy>) -> Self {
        Self { policies }
    }

    pub fn default_policies() -> Self {
        Self {
            policies: vec![
                Policy {
                    resource: "repo".into(),
                    actions: vec![Action::Read],
                    roles: vec![Role::Admin, Role::Member, Role::Guest],
                    conditions: None,
                },
                Policy {
                    resource: "repo".into(),
                    actions: vec![Action::Write, Action::Delete],
                    roles: vec![Role::Admin, Role::Member],
                    conditions: Some(PolicyConditions {
                        owner_only: false,
                        org_scoped: false,
                        public_only: false,
                    }),
                },
                Policy {
                    resource: "user".into(),
                    actions: vec![Action::Admin],
                    roles: vec![Role::Admin],
                    conditions: None,
                },
                Policy {
                    resource: "org".into(),
                    actions: vec![Action::Admin],
                    roles: vec![Role::Admin],
                    conditions: Some(PolicyConditions {
                        owner_only: false,
                        org_scoped: true,
                        public_only: false,
                    }),
                },
                Policy {
                    resource: "pipeline".into(),
                    actions: vec![Action::Read, Action::Write],
                    roles: vec![Role::Admin, Role::Member],
                    conditions: None,
                },
            ],
        }
    }

    pub fn check(&self, role: Role, action: Action, resource: &str) -> bool {
        for policy in &self.policies {
            if policy.resource == resource
                && policy.actions.contains(&action)
                && policy.roles.contains(&role)
            {
                debug!(role = ?role, action = ?action, resource = %resource, "policy matched");
                return true;
            }
        }
        debug!(role = ?role, action = ?action, resource = %resource, "no policy matched");
        false
    }

    pub fn check_with_conditions(
        &self,
        role: Role,
        action: Action,
        resource: &str,
        is_owner: bool,
        is_org_member: bool,
        is_public: bool,
    ) -> bool {
        for policy in &self.policies {
            if policy.resource != resource
                || !policy.actions.contains(&action)
                || !policy.roles.contains(&role)
            {
                continue;
            }
            if let Some(ref conds) = policy.conditions {
                if conds.owner_only && !is_owner {
                    continue;
                }
                if conds.org_scoped && !is_org_member {
                    continue;
                }
                if conds.public_only && !is_public {
                    continue;
                }
            }
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_permissions() {
        let admin_perms = Role::Admin.permissions();
        let guest_perms = Role::Guest.permissions();
        assert!(admin_perms.contains(&Permission::ManageUsers));
        assert!(admin_perms.contains(&Permission::Push));
        assert!(!guest_perms.contains(&Permission::Push));
        assert!(guest_perms.contains(&Permission::ReadRepo));
    }

    #[test]
    fn test_default_policy_guest_read_repo() {
        let engine = PolicyEngine::default_policies();
        assert!(engine.check(Role::Guest, Action::Read, "repo"));
        assert!(!engine.check(Role::Guest, Action::Write, "repo"));
    }

    #[test]
    fn test_default_policy_admin_full_access() {
        let engine = PolicyEngine::default_policies();
        assert!(engine.check(Role::Admin, Action::Read, "repo"));
        assert!(engine.check(Role::Admin, Action::Write, "repo"));
        assert!(engine.check(Role::Admin, Action::Admin, "user"));
        assert!(engine.check(Role::Admin, Action::Admin, "org"));
    }

    #[test]
    fn test_default_policy_member_write_repo() {
        let engine = PolicyEngine::default_policies();
        assert!(engine.check(Role::Member, Action::Write, "repo"));
        assert!(!engine.check(Role::Member, Action::Admin, "user"));
    }

    #[test]
    fn test_conditional_policy_org_scoped() {
        let engine = PolicyEngine::default_policies();
        assert!(engine.check_with_conditions(
            Role::Admin,
            Action::Admin,
            "org",
            false,
            true,
            false
        ));
        assert!(!engine.check_with_conditions(
            Role::Admin,
            Action::Admin,
            "org",
            false,
            false,
            false
        ));
    }

    #[test]
    fn test_conditional_policy_owner_only() {
        let engine = PolicyEngine::new(vec![Policy {
            resource: "repo".into(),
            actions: vec![Action::Delete],
            roles: vec![Role::Member],
            conditions: Some(PolicyConditions {
                owner_only: true,
                org_scoped: false,
                public_only: false,
            }),
        }]);
        assert!(engine.check_with_conditions(
            Role::Member,
            Action::Delete,
            "repo",
            true,
            false,
            false
        ));
        assert!(!engine.check_with_conditions(
            Role::Member,
            Action::Delete,
            "repo",
            false,
            true,
            false
        ));
    }

    #[test]
    fn test_role_from_str() {
        assert_eq!(Role::from_str("admin"), Some(Role::Admin));
        assert_eq!(Role::from_str("member"), Some(Role::Member));
        assert_eq!(Role::from_str("guest"), Some(Role::Guest));
        assert_eq!(Role::from_str("superadmin"), None);
    }

    #[test]
    fn test_guest_limited_to_read() {
        let engine = PolicyEngine::default_policies();
        assert!(engine.check(Role::Guest, Action::Read, "repo"));
        assert!(!engine.check(Role::Guest, Action::Write, "repo"));
        assert!(!engine.check(Role::Guest, Action::Delete, "repo"));
        assert!(!engine.check(Role::Guest, Action::Admin, "repo"));
        assert!(!engine.check(Role::Guest, Action::Admin, "user"));
    }
}
