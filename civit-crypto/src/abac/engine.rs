#![forbid(unsafe_code)]

use crate::abac::conditions::{AbacContext, PolicyCondition, evaluate_condition};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbacPolicy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub effect: Effect,
    pub target_actions: Vec<String>,
    pub target_resources: Vec<String>,
    pub conditions: Vec<PolicyCondition>,
    pub priority: u32,
}

impl AbacPolicy {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        effect: Effect,
        priority: u32,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            effect,
            target_actions: Vec::new(),
            target_resources: Vec::new(),
            conditions: Vec::new(),
            priority,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.target_actions.push(action.into());
        self
    }

    pub fn with_resource_type(mut self, resource: impl Into<String>) -> Self {
        self.target_resources.push(resource.into());
        self
    }

    pub fn with_condition(mut self, condition: PolicyCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    fn matches_target(&self, context: &AbacContext) -> bool {
        if !self.target_actions.is_empty()
            && !self
                .target_actions
                .iter()
                .any(|a| a == "*" || a == &context.action)
        {
            return false;
        }
        if !self.target_resources.is_empty()
            && !self
                .target_resources
                .iter()
                .any(|r| r == "*" || r == &context.resource.type_)
        {
            return false;
        }
        true
    }

    fn evaluate_conditions(&self, context: &AbacContext) -> Vec<(String, bool)> {
        self.conditions
            .iter()
            .map(|c| {
                let result = evaluate_condition(c, context);
                (c.condition_type.to_string(), result)
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct PolicyDecision {
    pub policy_id: String,
    pub allowed: bool,
    pub reason: String,
    pub evaluated_conditions: Vec<(String, bool)>,
}

pub struct AbacEngine {
    policies: Vec<AbacPolicy>,
    audit_log: Vec<PolicyDecision>,
}

impl Default for AbacEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AbacEngine {
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
            audit_log: Vec::new(),
        }
    }

    pub fn add_policy(&mut self, policy: AbacPolicy) {
        self.policies.push(policy);
        self.policies.sort_by_key(|p| p.priority);
    }

    pub fn remove_policy(&mut self, id: &str) -> bool {
        let len_before = self.policies.len();
        self.policies.retain(|p| p.id != id);
        self.policies.len() < len_before
    }

    pub fn evaluate(&mut self, context: &AbacContext) -> PolicyDecision {
        for policy in &self.policies {
            if !policy.matches_target(context) {
                continue;
            }

            let evaluated = policy.evaluate_conditions(context);
            let all_passed = evaluated.iter().all(|(_, passed)| *passed);

            if !all_passed {
                continue;
            }

            let decision = match policy.effect {
                Effect::Allow => PolicyDecision {
                    policy_id: policy.id.clone(),
                    allowed: true,
                    reason: format!(
                        "allowed by policy '{}' (priority {})",
                        policy.name, policy.priority
                    ),
                    evaluated_conditions: evaluated,
                },
                Effect::Deny => PolicyDecision {
                    policy_id: policy.id.clone(),
                    allowed: false,
                    reason: format!(
                        "denied by policy '{}' (priority {})",
                        policy.name, policy.priority
                    ),
                    evaluated_conditions: evaluated,
                },
            };

            self.audit_log.push(decision.clone());
            return decision;
        }

        let default_deny = PolicyDecision {
            policy_id: "default-deny".to_string(),
            allowed: false,
            reason: "no matching policy; default deny".to_string(),
            evaluated_conditions: Vec::new(),
        };
        self.audit_log.push(default_deny.clone());
        default_deny
    }

    pub fn list_policies(&self) -> &[AbacPolicy] {
        &self.policies
    }

    pub fn audit_trail(&self) -> &[PolicyDecision] {
        &self.audit_log
    }

    pub fn policy_count(&self) -> usize {
        self.policies.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abac::conditions::{
        AbacEnvironment, AbacResource, AbacSubject, ConditionType, DevicePosture, PolicyCondition,
    };

    fn admin_context() -> AbacContext {
        let subject = AbacSubject::new("user-1").with_role("admin");
        let resource = AbacResource::new("res-1", "server");
        let env = AbacEnvironment::new();
        AbacContext::new(subject, resource, "deploy", env)
    }

    fn dev_context() -> AbacContext {
        let subject = AbacSubject::new("user-2").with_role("developer");
        let resource = AbacResource::new("res-1", "server");
        let env = AbacEnvironment::new();
        AbacContext::new(subject, resource, "deploy", env)
    }

    fn guest_context() -> AbacContext {
        let subject = AbacSubject::new("user-3").with_role("guest");
        let resource = AbacResource::new("res-1", "server");
        let env = AbacEnvironment::new();
        AbacContext::new(subject, resource, "deploy", env)
    }

    #[test]
    fn test_admin_allowed() {
        let mut engine = AbacEngine::new();
        engine.add_policy(
            AbacPolicy::new("p1", "Admin Deploy", Effect::Allow, 1)
                .with_action("deploy")
                .with_resource_type("server")
                .with_condition(PolicyCondition::new(
                    ConditionType::RoleMatch,
                    "admin",
                    serde_json::Value::Null,
                )),
        );

        let decision = engine.evaluate(&admin_context());
        assert!(decision.allowed);
        assert_eq!(decision.policy_id, "p1");
    }

    #[test]
    fn test_dev_denied_no_match() {
        let mut engine = AbacEngine::new();
        engine.add_policy(
            AbacPolicy::new("p1", "Admin Deploy", Effect::Allow, 1)
                .with_action("deploy")
                .with_resource_type("server")
                .with_condition(PolicyCondition::new(
                    ConditionType::RoleMatch,
                    "admin",
                    serde_json::Value::Null,
                )),
        );

        let decision = engine.evaluate(&dev_context());
        assert!(!decision.allowed);
        assert_eq!(decision.policy_id, "default-deny");
    }

    #[test]
    fn test_deny_policy() {
        let mut engine = AbacEngine::new();
        engine.add_policy(
            AbacPolicy::new("p1", "Deny Guest", Effect::Deny, 1)
                .with_action("*")
                .with_resource_type("*")
                .with_condition(PolicyCondition::new(
                    ConditionType::RoleMatch,
                    "guest",
                    serde_json::Value::Null,
                )),
        );

        let decision = engine.evaluate(&guest_context());
        assert!(!decision.allowed);
        assert_eq!(decision.policy_id, "p1");
        assert!(decision.reason.contains("denied"));
    }

    #[test]
    fn test_priority_ordering() {
        let mut engine = AbacEngine::new();
        engine.add_policy(
            AbacPolicy::new("p2", "Deny All Deploy", Effect::Deny, 10)
                .with_action("deploy")
                .with_resource_type("*"),
        );
        engine.add_policy(
            AbacPolicy::new("p1", "Admin Allow", Effect::Allow, 1)
                .with_action("deploy")
                .with_resource_type("*")
                .with_condition(PolicyCondition::new(
                    ConditionType::RoleMatch,
                    "admin",
                    serde_json::Value::Null,
                )),
        );

        let decision = engine.evaluate(&admin_context());
        assert!(decision.allowed);
        assert_eq!(decision.policy_id, "p1");
    }

    #[test]
    fn test_multiple_conditions_all_must_match() {
        let mut engine = AbacEngine::new();
        engine.add_policy(
            AbacPolicy::new("p1", "Managed Admin", Effect::Allow, 1)
                .with_action("*")
                .with_resource_type("*")
                .with_condition(PolicyCondition::new(
                    ConditionType::RoleMatch,
                    "admin",
                    serde_json::Value::Null,
                ))
                .with_condition(PolicyCondition::new(
                    ConditionType::DevicePostureMatch,
                    "managed",
                    serde_json::Value::Null,
                )),
        );

        let managed_env = AbacEnvironment::new().with_device_posture(DevicePosture::Managed);
        let subject = AbacSubject::new("user-1").with_role("admin");
        let resource = AbacResource::new("r1", "server");
        let ctx = AbacContext::new(subject, resource, "deploy", managed_env);
        let decision = engine.evaluate(&ctx);
        assert!(decision.allowed);

        let unmanaged_env = AbacEnvironment::new().with_device_posture(DevicePosture::Unmanaged);
        let subject2 = AbacSubject::new("user-1").with_role("admin");
        let resource2 = AbacResource::new("r1", "server");
        let ctx2 = AbacContext::new(subject2, resource2, "deploy", unmanaged_env);
        let decision2 = engine.evaluate(&ctx2);
        assert!(!decision2.allowed);
    }

    #[test]
    fn test_empty_engine_default_deny() {
        let mut engine = AbacEngine::new();
        let decision = engine.evaluate(&admin_context());
        assert!(!decision.allowed);
        assert_eq!(decision.policy_id, "default-deny");
    }

    #[test]
    fn test_remove_policy() {
        let mut engine = AbacEngine::new();
        engine.add_policy(
            AbacPolicy::new("p1", "Allow", Effect::Allow, 1)
                .with_action("*")
                .with_resource_type("*"),
        );
        assert!(engine.remove_policy("p1"));
        assert_eq!(engine.policy_count(), 0);
        assert!(!engine.remove_policy("nonexistent"));
    }

    #[test]
    fn test_audit_trail() {
        let mut engine = AbacEngine::new();
        engine.add_policy(
            AbacPolicy::new("p1", "Allow", Effect::Allow, 1)
                .with_action("read")
                .with_resource_type("document"),
        );
        let subject = AbacSubject::new("u1");
        let resource = AbacResource::new("r1", "document");
        let ctx = AbacContext::new(subject, resource, "read", AbacEnvironment::new());
        engine.evaluate(&ctx);
        engine.evaluate(&ctx);
        assert_eq!(engine.audit_trail().len(), 2);
    }

    #[test]
    fn test_target_action_wildcard() {
        let mut engine = AbacEngine::new();
        engine.add_policy(
            AbacPolicy::new("p1", "Wildcard", Effect::Allow, 1)
                .with_action("*")
                .with_resource_type("*"),
        );
        let ctx = admin_context();
        let decision = engine.evaluate(&ctx);
        assert!(decision.allowed);
    }

    #[test]
    fn test_target_resource_filter() {
        let mut engine = AbacEngine::new();
        engine.add_policy(
            AbacPolicy::new("p1", "Doc Only", Effect::Allow, 1)
                .with_action("*")
                .with_resource_type("document"),
        );
        let subject = AbacSubject::new("u1");
        let doc = AbacResource::new("d1", "document");
        let ctx_doc = AbacContext::new(subject.clone(), doc, "read", AbacEnvironment::new());
        assert!(engine.evaluate(&ctx_doc).allowed);

        let server = AbacResource::new("s1", "server");
        let ctx_srv = AbacContext::new(subject, server, "read", AbacEnvironment::new());
        assert!(!engine.evaluate(&ctx_srv).allowed);
    }

    #[test]
    fn test_evaluated_conditions_recorded() {
        let mut engine = AbacEngine::new();
        engine.add_policy(
            AbacPolicy::new("p1", "Admin", Effect::Allow, 1)
                .with_action("*")
                .with_resource_type("*")
                .with_condition(PolicyCondition::new(
                    ConditionType::RoleMatch,
                    "admin",
                    serde_json::Value::Null,
                )),
        );
        let decision = engine.evaluate(&admin_context());
        assert_eq!(decision.evaluated_conditions.len(), 1);
        assert_eq!(decision.evaluated_conditions[0].0, "role_match");
        assert!(decision.evaluated_conditions[0].1);
    }
}
