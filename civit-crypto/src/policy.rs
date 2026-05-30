#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Effect {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Get,
    List,
    Create,
    Update,
    Delete,
    Admin,
    Execute,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Resource {
    Repository,
    Pipeline,
    Issue,
    PullRequest,
    User,
    Organization,
    Secret,
    Artifact,
    Any,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub id: String,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
    pub attributes: HashMap<String, String>,
}

impl Subject {
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    pub fn in_group(&self, group: &str) -> bool {
        self.groups.iter().any(|g| g == group)
    }

    pub fn has_attribute(&self, key: &str, value: &str) -> bool {
        self.attributes
            .get(key)
            .map(|v| v == value)
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStatement {
    pub id: String,
    pub effect: Effect,
    pub actions: Vec<Action>,
    pub resources: Vec<Resource>,
    pub principals: Option<Vec<String>>,
    pub conditions: Option<Vec<Condition>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    RoleRequired(String),
    GroupRequired(String),
    AttributeMatch { key: String, value: String },
    OwnerOnly,
    ResourcePublic,
    ResourcePrivate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessResult {
    pub allowed: bool,
    pub matched_policy: Option<String>,
    pub reason: String,
}

pub struct PolicyEngine {
    statements: Vec<PolicyStatement>,
}

impl PolicyEngine {
    pub fn new(statements: Vec<PolicyStatement>) -> Self {
        Self { statements }
    }

    pub fn default_policies() -> Self {
        Self::new(vec![
            PolicyStatement {
                id: "allow-read-public".into(),
                effect: Effect::Allow,
                actions: vec![Action::Get, Action::List],
                resources: vec![Resource::Repository, Resource::Issue, Resource::PullRequest],
                principals: None,
                conditions: Some(vec![Condition::ResourcePublic]),
            },
            PolicyStatement {
                id: "allow-member-crud".into(),
                effect: Effect::Allow,
                actions: vec![Action::Get, Action::List, Action::Create, Action::Update],
                resources: vec![
                    Resource::Repository,
                    Resource::Issue,
                    Resource::PullRequest,
                    Resource::Pipeline,
                ],
                principals: None,
                conditions: Some(vec![Condition::RoleRequired("member".into())]),
            },
            PolicyStatement {
                id: "allow-admin-all".into(),
                effect: Effect::Allow,
                actions: vec![
                    Action::Get,
                    Action::List,
                    Action::Create,
                    Action::Update,
                    Action::Delete,
                    Action::Admin,
                ],
                resources: vec![Resource::Any],
                principals: None,
                conditions: Some(vec![Condition::RoleRequired("admin".into())]),
            },
            PolicyStatement {
                id: "deny-guest-admin".into(),
                effect: Effect::Deny,
                actions: vec![Action::Admin, Action::Delete],
                resources: vec![Resource::Any],
                principals: None,
                conditions: Some(vec![Condition::RoleRequired("guest".into())]),
            },
        ])
    }

    pub fn evaluate(
        &self,
        subject: &Subject,
        action: Action,
        resource: Resource,
        resource_attrs: &HashMap<String, String>,
    ) -> AccessResult {
        let mut denied: Option<&PolicyStatement> = None;

        for stmt in &self.statements {
            if !stmt.actions.contains(&action) && !stmt.actions.contains(&Action::Admin) {
                continue;
            }
            if !stmt.resources.contains(&resource) && !stmt.resources.contains(&Resource::Any) {
                continue;
            }

            if let Some(ref conditions) = stmt.conditions {
                if !conditions
                    .iter()
                    .all(|cond| self.evaluate_condition(cond, subject, resource_attrs))
                {
                    continue;
                }
            }

            match stmt.effect {
                Effect::Deny => {
                    denied = Some(stmt);
                }
                Effect::Allow => {
                    return AccessResult {
                        allowed: true,
                        matched_policy: Some(stmt.id.clone()),
                        reason: format!("allowed by policy: {}", stmt.id),
                    };
                }
            }
        }

        if let Some(deny_stmt) = denied {
            return AccessResult {
                allowed: false,
                matched_policy: Some(deny_stmt.id.clone()),
                reason: format!("denied by policy: {}", deny_stmt.id),
            };
        }

        AccessResult {
            allowed: false,
            matched_policy: None,
            reason: "no matching policy".into(),
        }
    }

    fn evaluate_condition(
        &self,
        condition: &Condition,
        subject: &Subject,
        resource_attrs: &HashMap<String, String>,
    ) -> bool {
        match condition {
            Condition::RoleRequired(role) => subject.has_role(role),
            Condition::GroupRequired(group) => subject.in_group(group),
            Condition::AttributeMatch { key, value } => subject.has_attribute(key, value),
            Condition::OwnerOnly => subject.has_attribute(
                "owner_id",
                resource_attrs.get("owner_id").unwrap_or(&String::new()),
            ),
            Condition::ResourcePublic => resource_attrs
                .get("visibility")
                .map(|v| v == "public")
                .unwrap_or(false),
            Condition::ResourcePrivate => resource_attrs
                .get("visibility")
                .map(|v| v == "private")
                .unwrap_or(false),
        }
    }

    pub fn add_statement(&mut self, statement: PolicyStatement) {
        debug!(id = %statement.id, "added policy statement");
        self.statements.push(statement);
    }

    pub fn remove_statement(&mut self, id: &str) -> bool {
        let len_before = self.statements.len();
        self.statements.retain(|s| s.id != id);
        self.statements.len() < len_before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn admin_subject() -> Subject {
        Subject {
            id: "admin-1".into(),
            roles: vec!["admin".into()],
            groups: vec!["org-1".into()],
            attributes: HashMap::new(),
        }
    }

    fn member_subject() -> Subject {
        Subject {
            id: "member-1".into(),
            roles: vec!["member".into()],
            groups: vec!["team-a".into()],
            attributes: HashMap::new(),
        }
    }

    fn guest_subject() -> Subject {
        Subject {
            id: "guest-1".into(),
            roles: vec!["guest".into()],
            groups: vec![],
            attributes: HashMap::new(),
        }
    }

    fn public_repo_attrs() -> HashMap<String, String> {
        vec![("visibility".into(), "public".into())]
            .into_iter()
            .collect()
    }

    #[test]
    fn test_admin_full_access() {
        let engine = PolicyEngine::default_policies();
        let result = engine.evaluate(
            &admin_subject(),
            Action::Delete,
            Resource::Repository,
            &public_repo_attrs(),
        );
        assert!(result.allowed);
    }

    #[test]
    fn test_member_read_write() {
        let engine = PolicyEngine::default_policies();
        let result = engine.evaluate(
            &member_subject(),
            Action::Get,
            Resource::Repository,
            &public_repo_attrs(),
        );
        assert!(result.allowed);

        let result = engine.evaluate(
            &member_subject(),
            Action::Create,
            Resource::Repository,
            &public_repo_attrs(),
        );
        assert!(result.allowed);
    }

    #[test]
    fn test_member_cannot_delete() {
        let engine = PolicyEngine::default_policies();
        let result = engine.evaluate(
            &member_subject(),
            Action::Delete,
            Resource::Repository,
            &public_repo_attrs(),
        );
        assert!(!result.allowed);
    }

    #[test]
    fn test_guest_read_public() {
        let engine = PolicyEngine::default_policies();
        let result = engine.evaluate(
            &guest_subject(),
            Action::Get,
            Resource::Repository,
            &public_repo_attrs(),
        );
        assert!(result.allowed);
    }

    #[test]
    fn test_guest_cannot_admin() {
        let engine = PolicyEngine::default_policies();
        let result = engine.evaluate(
            &guest_subject(),
            Action::Admin,
            Resource::User,
            &HashMap::new(),
        );
        assert!(!result.allowed);
    }

    #[test]
    fn test_deny_guest_admin_explicit() {
        let engine = PolicyEngine::default_policies();
        let result = engine.evaluate(
            &guest_subject(),
            Action::Admin,
            Resource::Any,
            &HashMap::new(),
        );
        assert!(!result.allowed);
        assert!(result.reason.contains("denied"));
    }

    #[test]
    fn test_add_and_remove_statement() {
        let mut engine = PolicyEngine::new(vec![]);
        let stmt = PolicyStatement {
            id: "custom-1".into(),
            effect: Effect::Allow,
            actions: vec![Action::Get],
            resources: vec![Resource::Repository],
            principals: None,
            conditions: None,
        };
        engine.add_statement(stmt);
        assert!(
            engine
                .evaluate(
                    &guest_subject(),
                    Action::Get,
                    Resource::Repository,
                    &HashMap::new()
                )
                .allowed
        );
        engine.remove_statement("custom-1");
        assert!(
            !engine
                .evaluate(
                    &guest_subject(),
                    Action::Get,
                    Resource::Repository,
                    &HashMap::new()
                )
                .allowed
        );
    }

    #[test]
    fn test_subject_role_check() {
        let s = admin_subject();
        assert!(s.has_role("admin"));
        assert!(!s.has_role("guest"));
    }

    #[test]
    fn test_subject_group_check() {
        let s = member_subject();
        assert!(s.in_group("team-a"));
        assert!(!s.in_group("team-b"));
    }

    #[test]
    fn test_no_matching_policy() {
        let engine = PolicyEngine::new(vec![]);
        let result = engine.evaluate(
            &admin_subject(),
            Action::Get,
            Resource::Repository,
            &HashMap::new(),
        );
        assert!(!result.allowed);
        assert_eq!(result.reason, "no matching policy");
    }

    #[test]
    fn test_private_repo_guest_denied() {
        let engine = PolicyEngine::default_policies();
        let attrs = vec![("visibility".into(), "private".into())]
            .into_iter()
            .collect();
        let result = engine.evaluate(&guest_subject(), Action::Get, Resource::Repository, &attrs);
        assert!(!result.allowed);
    }
}
