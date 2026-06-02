#![forbid(unsafe_code)]

use crate::cel::CelExpression;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyVersion {
    pub version_id: String,
    pub policy_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub author: String,
    pub description: String,
    pub expression: CelExpression,
    pub checksum: String,
    pub parent_version: Option<String>,
}

impl PolicyVersion {
    pub fn compute_checksum(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.policy_id.hash(&mut hasher);
        self.expression.raw.hash(&mut hasher);
        self.checksum = format!("{:016x}", hasher.finish());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyAction {
    Created,
    Updated,
    RolledBack,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDiff {
    pub added_conditions: Vec<String>,
    pub removed_conditions: Vec<String>,
    pub modified_conditions: Vec<String>,
    pub metadata_changes: Vec<String>,
}

pub struct PolicyStore {
    policies: DashMap<String, PolicyVersion>,
    versions: HashMap<String, Vec<PolicyVersion>>,
    current: HashMap<String, String>,
}

impl Default for PolicyStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyStore {
    pub fn new() -> Self {
        Self {
            policies: DashMap::new(),
            versions: HashMap::new(),
            current: HashMap::new(),
        }
    }

    pub fn create_policy(
        &mut self,
        policy_id: impl Into<String>,
        author: impl Into<String>,
        description: impl Into<String>,
        expression: CelExpression,
    ) -> PolicyVersion {
        let policy_id = policy_id.into();
        let version_id = uuid::Uuid::new_v4().to_string();
        let mut version = PolicyVersion {
            version_id: version_id.clone(),
            policy_id: policy_id.clone(),
            created_at: chrono::Utc::now(),
            author: author.into(),
            description: description.into(),
            expression,
            checksum: String::new(),
            parent_version: None,
        };
        version.compute_checksum();
        let version_clone = version.clone();
        self.policies.insert(version_id.clone(), version);
        self.versions
            .entry(policy_id.clone())
            .or_default()
            .push(version_clone.clone());
        self.current.insert(policy_id, version_id);
        version_clone
    }

    pub fn update_policy(
        &mut self,
        policy_id: &str,
        author: impl Into<String>,
        description: impl Into<String>,
        expression: CelExpression,
    ) -> Option<PolicyVersion> {
        let parent = self.current.get(policy_id)?.clone();
        let version_id = uuid::Uuid::new_v4().to_string();
        let mut version = PolicyVersion {
            version_id: version_id.clone(),
            policy_id: policy_id.to_string(),
            created_at: chrono::Utc::now(),
            author: author.into(),
            description: description.into(),
            expression,
            checksum: String::new(),
            parent_version: Some(parent),
        };
        version.compute_checksum();
        let version_clone = version.clone();
        self.policies.insert(version_id.clone(), version);
        self.versions
            .entry(policy_id.to_string())
            .or_default()
            .push(version_clone.clone());
        self.current.insert(policy_id.to_string(), version_id);
        Some(version_clone)
    }

    pub fn get_current_policy(&self, policy_id: &str) -> Option<PolicyVersion> {
        let version_id = self.current.get(policy_id)?;
        self.policies.get(version_id).map(|r| r.value().clone())
    }

    pub fn list_versions(&self, policy_id: &str) -> Vec<PolicyVersion> {
        self.versions.get(policy_id).cloned().unwrap_or_default()
    }

    pub fn diff_versions(&self, policy_id: &str, v1: &str, v2: &str) -> Option<PolicyDiff> {
        let all_versions = self.versions.get(policy_id)?;
        let first = all_versions.iter().find(|v| v.version_id == v1)?;
        let second = all_versions.iter().find(|v| v.version_id == v2)?;
        let added = Vec::new();
        let removed = Vec::new();
        let mut modified = Vec::new();
        let mut meta = Vec::new();
        if first.expression.raw != second.expression.raw {
            modified.push(first.expression.raw.clone());
        }
        if first.author != second.author {
            meta.push(format!("author: {} -> {}", first.author, second.author));
        }
        if first.description != second.description {
            meta.push("description changed".to_string());
        }
        if first.checksum != second.checksum && modified.is_empty() {
            modified.push("(checksum changed without expression change)".into());
        }
        Some(PolicyDiff {
            added_conditions: added,
            removed_conditions: removed,
            modified_conditions: modified,
            metadata_changes: meta,
        })
    }

    pub fn rollback(&mut self, policy_id: &str, target_version_id: &str) -> Option<PolicyVersion> {
        let all_versions = self.versions.get(policy_id)?;
        let target = all_versions
            .iter()
            .find(|v| v.version_id == target_version_id)?;
        self.current
            .insert(policy_id.to_string(), target_version_id.to_string());
        Some(target.clone())
    }

    pub fn policy_count(&self) -> usize {
        self.current.len()
    }

    pub fn version_count(&self, policy_id: &str) -> usize {
        self.versions.get(policy_id).map(|v| v.len()).unwrap_or(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyAuditEntry {
    pub action: PolicyAction,
    pub policy_id: String,
    pub version: String,
    pub author: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub changes: String,
}

pub struct PolicyAuditTrail {
    entries: Vec<PolicyAuditEntry>,
}

impl Default for PolicyAuditTrail {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyAuditTrail {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn record(
        &mut self,
        action: PolicyAction,
        policy_id: impl Into<String>,
        version: impl Into<String>,
        author: impl Into<String>,
        changes: impl Into<String>,
    ) {
        self.entries.push(PolicyAuditEntry {
            action,
            policy_id: policy_id.into(),
            version: version.into(),
            author: author.into(),
            timestamp: chrono::Utc::now(),
            changes: changes.into(),
        });
    }

    pub fn entries(&self) -> &[PolicyAuditEntry] {
        &self.entries
    }

    pub fn entries_for_policy(&self, policy_id: &str) -> Vec<&PolicyAuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.policy_id == policy_id)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GeoFenceEffect {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoFenceRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cidr_ranges: Vec<String>,
    pub allowed_actions: Vec<String>,
    pub denied_actions: Vec<String>,
    pub priority: u32,
    pub effect: GeoFenceEffect,
}

impl GeoFenceRule {
    pub fn new(id: impl Into<String>, name: impl Into<String>, effect: GeoFenceEffect) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            cidr_ranges: Vec::new(),
            allowed_actions: Vec::new(),
            denied_actions: Vec::new(),
            priority: 0,
            effect,
        }
    }

    pub fn with_cidr(mut self, range: impl Into<String>) -> Self {
        self.cidr_ranges.push(range.into());
        self
    }

    pub fn with_allowed_action(mut self, action: impl Into<String>) -> Self {
        self.allowed_actions.push(action.into());
        self
    }

    pub fn with_denied_action(mut self, action: impl Into<String>) -> Self {
        self.denied_actions.push(action.into());
        self
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    fn ip_in_ranges(&self, ip: &IpAddr) -> bool {
        if self.cidr_ranges.is_empty() {
            return true;
        }
        for range in &self.cidr_ranges {
            if ip_matches_cidr(ip, range) {
                return true;
            }
        }
        false
    }
}

fn ip_matches_cidr(ip: &IpAddr, cidr: &str) -> bool {
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return false;
    }
    let network_ip = match IpAddr::from_str(parts[0]) {
        Ok(addr) => addr,
        Err(_) => return false,
    };
    let prefix_len: u32 = match parts[1].parse() {
        Ok(len) => len,
        Err(_) => return false,
    };

    match ip {
        IpAddr::V4(v4) => {
            let net_bits = match network_ip {
                IpAddr::V4(v4_net) => u32::from_be_bytes(v4_net.octets()),
                _ => return false,
            };
            let ip_bits = u32::from_be_bytes(v4.octets());
            if prefix_len >= 32 {
                ip_bits == net_bits
            } else if prefix_len == 0 {
                true
            } else {
                let mask = !0u32 << (32 - prefix_len);
                (ip_bits & mask) == (net_bits & mask)
            }
        }
        IpAddr::V6(v6) => {
            let net_bits = match network_ip {
                IpAddr::V6(v6_net) => u128::from_be_bytes(v6_net.octets()),
                _ => return false,
            };
            let ip_bits = u128::from_be_bytes(v6.octets());
            if prefix_len >= 128 {
                ip_bits == net_bits
            } else if prefix_len == 0 {
                true
            } else {
                let mask = !0u128 << (128 - prefix_len);
                (ip_bits & mask) == (net_bits & mask)
            }
        }
    }
}

pub struct GeoFenceEvaluator;

impl Default for GeoFenceEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl GeoFenceEvaluator {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate(ip: &str, action: &str, rules: &[GeoFenceRule]) -> bool {
        let parsed_ip = match IpAddr::from_str(ip) {
            Ok(addr) => addr,
            Err(_) => return false,
        };
        let mut sorted_rules: Vec<&GeoFenceRule> = rules.iter().collect();
        sorted_rules.sort_by_key(|r| std::cmp::Reverse(r.priority));
        for rule in &sorted_rules {
            if !rule.ip_in_ranges(&parsed_ip) {
                continue;
            }
            if !rule.denied_actions.is_empty() && rule.denied_actions.iter().any(|a| a == action) {
                return false;
            }
            if !rule.allowed_actions.is_empty() && rule.allowed_actions.iter().any(|a| a == action)
            {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_expression() -> CelExpression {
        CelExpression::parse("user.role == \"admin\"")
    }

    #[test]
    fn test_create_policy() {
        let mut store = PolicyStore::new();
        let version = store.create_policy("pol-1", "alice", "initial policy", test_expression());
        assert_eq!(version.policy_id, "pol-1");
        assert_eq!(version.author, "alice");
        assert!(!version.version_id.is_empty());
        assert!(!version.checksum.is_empty());
    }

    #[test]
    fn test_get_current_policy() {
        let mut store = PolicyStore::new();
        store.create_policy("pol-1", "alice", "initial", test_expression());
        let current = store.get_current_policy("pol-1").unwrap();
        assert_eq!(current.policy_id, "pol-1");
    }

    #[test]
    fn test_get_current_policy_missing() {
        let store = PolicyStore::new();
        assert!(store.get_current_policy("nonexistent").is_none());
    }

    #[test]
    fn test_update_policy() {
        let mut store = PolicyStore::new();
        store.create_policy("pol-1", "alice", "v1", test_expression());
        let new_expr = CelExpression::parse("user.role == \"member\"");
        let v2 = store.update_policy("pol-1", "bob", "v2", new_expr).unwrap();
        assert_eq!(v2.author, "bob");
        assert!(v2.parent_version.is_some());
        let current = store.get_current_policy("pol-1").unwrap();
        assert_eq!(current.author, "bob");
    }

    #[test]
    fn test_update_nonexistent_policy() {
        let mut store = PolicyStore::new();
        let result = store.update_policy("nope", "alice", "desc", test_expression());
        assert!(result.is_none());
    }

    #[test]
    fn test_list_versions() {
        let mut store = PolicyStore::new();
        store.create_policy("pol-1", "alice", "v1", test_expression());
        let new_expr = CelExpression::parse("user.age > 18");
        store.update_policy("pol-1", "bob", "v2", new_expr);
        let versions = store.list_versions("pol-1");
        assert_eq!(versions.len(), 2);
    }

    #[test]
    fn test_list_versions_empty() {
        let store = PolicyStore::new();
        let versions = store.list_versions("nope");
        assert!(versions.is_empty());
    }

    #[test]
    fn test_diff_versions() {
        let mut store = PolicyStore::new();
        let v1 = store.create_policy("pol-1", "alice", "v1", test_expression());
        let new_expr = CelExpression::parse("user.age > 18");
        let v2 = store.update_policy("pol-1", "bob", "v2", new_expr).unwrap();
        let diff = store
            .diff_versions("pol-1", &v1.version_id, &v2.version_id)
            .unwrap();
        assert!(!diff.modified_conditions.is_empty());
        assert!(!diff.metadata_changes.is_empty());
    }

    #[test]
    fn test_diff_same_version() {
        let mut store = PolicyStore::new();
        let v1 = store.create_policy("pol-1", "alice", "v1", test_expression());
        let diff = store
            .diff_versions("pol-1", &v1.version_id, &v1.version_id)
            .unwrap();
        assert!(diff.modified_conditions.is_empty());
        assert!(diff.metadata_changes.is_empty());
    }

    #[test]
    fn test_rollback() {
        let mut store = PolicyStore::new();
        let v1 = store.create_policy("pol-1", "alice", "v1", test_expression());
        store.update_policy("pol-1", "bob", "v2", CelExpression::parse("user.age > 18"));
        let rolled_back = store.rollback("pol-1", &v1.version_id).unwrap();
        assert_eq!(rolled_back.author, "alice");
        let current = store.get_current_policy("pol-1").unwrap();
        assert_eq!(current.version_id, v1.version_id);
    }

    #[test]
    fn test_policy_count() {
        let mut store = PolicyStore::new();
        assert_eq!(store.policy_count(), 0);
        store.create_policy("pol-1", "a", "d", test_expression());
        store.create_policy("pol-2", "b", "d", test_expression());
        assert_eq!(store.policy_count(), 2);
    }

    #[test]
    fn test_version_count() {
        let mut store = PolicyStore::new();
        store.create_policy("pol-1", "a", "d", test_expression());
        store.update_policy("pol-1", "b", "d", CelExpression::parse("user.age > 18"));
        assert_eq!(store.version_count("pol-1"), 2);
        assert_eq!(store.version_count("nope"), 0);
    }

    #[test]
    fn test_audit_trail_record() {
        let mut trail = PolicyAuditTrail::new();
        trail.record(
            PolicyAction::Created,
            "pol-1",
            "v1",
            "alice",
            "created policy",
        );
        assert_eq!(trail.len(), 1);
        let entry = &trail.entries()[0];
        assert_eq!(entry.policy_id, "pol-1");
        assert_eq!(entry.author, "alice");
        assert!(matches!(entry.action, PolicyAction::Created));
    }

    #[test]
    fn test_audit_trail_filter_by_policy() {
        let mut trail = PolicyAuditTrail::new();
        trail.record(PolicyAction::Created, "pol-1", "v1", "alice", "");
        trail.record(PolicyAction::Updated, "pol-2", "v1", "bob", "");
        trail.record(PolicyAction::Updated, "pol-1", "v2", "alice", "");
        let entries = trail.entries_for_policy("pol-1");
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_audit_trail_empty() {
        let trail = PolicyAuditTrail::new();
        assert!(trail.is_empty());
        assert_eq!(trail.len(), 0);
    }

    #[test]
    fn test_geofence_rule_builder() {
        let rule = GeoFenceRule::new("r1", "block-corp", GeoFenceEffect::Deny)
            .with_cidr("10.0.0.0/8")
            .with_denied_action("delete");
        assert_eq!(rule.id, "r1");
        assert_eq!(rule.effect, GeoFenceEffect::Deny);
        assert_eq!(rule.cidr_ranges.len(), 1);
        assert_eq!(rule.denied_actions.len(), 1);
    }

    #[test]
    fn test_geofence_rule_priority() {
        let rule = GeoFenceRule::new("r1", "test", GeoFenceEffect::Allow).with_priority(100);
        assert_eq!(rule.priority, 100);
    }

    #[test]
    fn test_geofence_evaluate_allow() {
        let rules = vec![
            GeoFenceRule::new("r1", "allow-internal", GeoFenceEffect::Allow)
                .with_cidr("10.0.0.0/8")
                .with_allowed_action("read"),
        ];
        assert!(GeoFenceEvaluator::evaluate("10.0.0.1", "read", &rules));
    }

    #[test]
    fn test_geofence_evaluate_deny() {
        let rules = vec![
            GeoFenceRule::new("r1", "deny-external", GeoFenceEffect::Deny)
                .with_cidr("0.0.0.0/0")
                .with_denied_action("delete"),
        ];
        assert!(!GeoFenceEvaluator::evaluate("8.8.8.8", "delete", &rules));
    }

    #[test]
    fn test_geofence_evaluate_no_match() {
        let rules = vec![
            GeoFenceRule::new("r1", "allow-internal", GeoFenceEffect::Allow)
                .with_cidr("10.0.0.0/8")
                .with_allowed_action("read"),
        ];
        assert!(!GeoFenceEvaluator::evaluate("192.168.1.1", "read", &rules));
    }

    #[test]
    fn test_geofence_evaluate_empty_ranges() {
        let rules = vec![
            GeoFenceRule::new("r1", "allow-all", GeoFenceEffect::Allow).with_allowed_action("read"),
        ];
        assert!(GeoFenceEvaluator::evaluate("1.2.3.4", "read", &rules));
    }

    #[test]
    fn test_geofence_evaluate_invalid_ip() {
        let rules = vec![];
        assert!(!GeoFenceEvaluator::evaluate("not-an-ip", "read", &rules));
    }

    #[test]
    fn test_geofence_priority_ordering() {
        let rules = vec![
            GeoFenceRule::new("allow-all", "allow", GeoFenceEffect::Allow)
                .with_priority(1)
                .with_allowed_action("delete"),
            GeoFenceRule::new("deny-admin", "deny", GeoFenceEffect::Deny)
                .with_priority(100)
                .with_denied_action("delete"),
        ];
        assert!(!GeoFenceEvaluator::evaluate("10.0.0.1", "delete", &rules));
    }

    #[test]
    fn test_geofence_ipv6_cidr() {
        let rules = vec![
            GeoFenceRule::new("r1", "v6-allow", GeoFenceEffect::Allow)
                .with_cidr("::1/128")
                .with_allowed_action("read"),
        ];
        assert!(GeoFenceEvaluator::evaluate("::1", "read", &rules));
        assert!(!GeoFenceEvaluator::evaluate("::2", "read", &rules));
    }

    #[test]
    fn test_geofence_multiple_cidr() {
        let rules = vec![
            GeoFenceRule::new("r1", "multi-cidr", GeoFenceEffect::Allow)
                .with_cidr("10.0.0.0/8")
                .with_cidr("192.168.0.0/16")
                .with_allowed_action("read"),
        ];
        assert!(GeoFenceEvaluator::evaluate("10.1.2.3", "read", &rules));
        assert!(GeoFenceEvaluator::evaluate("192.168.1.1", "read", &rules));
        assert!(!GeoFenceEvaluator::evaluate("8.8.8.8", "read", &rules));
    }

    #[test]
    fn test_policy_version_checksum_deterministic() {
        let expr = CelExpression::parse("user.role == \"admin\"");
        let mut v1 = PolicyVersion {
            version_id: "v1".into(),
            policy_id: "p1".into(),
            created_at: chrono::Utc::now(),
            author: "alice".into(),
            description: "test".into(),
            expression: expr.clone(),
            checksum: String::new(),
            parent_version: None,
        };
        v1.compute_checksum();
        let mut v2 = PolicyVersion {
            version_id: "v2".into(),
            policy_id: "p1".into(),
            created_at: chrono::Utc::now(),
            author: "alice".into(),
            description: "test".into(),
            expression: expr,
            checksum: String::new(),
            parent_version: None,
        };
        v2.compute_checksum();
        assert_eq!(v1.checksum, v2.checksum);
    }

    #[test]
    fn test_policy_version_checksum_differs() {
        let mut v1 = PolicyVersion {
            version_id: "v1".into(),
            policy_id: "p1".into(),
            created_at: chrono::Utc::now(),
            author: "alice".into(),
            description: "test".into(),
            expression: CelExpression::parse("user.role == \"admin\""),
            checksum: String::new(),
            parent_version: None,
        };
        v1.compute_checksum();
        let mut v2 = PolicyVersion {
            version_id: "v2".into(),
            policy_id: "p1".into(),
            created_at: chrono::Utc::now(),
            author: "alice".into(),
            description: "test".into(),
            expression: CelExpression::parse("user.role == \"member\""),
            checksum: String::new(),
            parent_version: None,
        };
        v2.compute_checksum();
        assert_ne!(v1.checksum, v2.checksum);
    }
}
