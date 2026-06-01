#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;

const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const DEFAULT_RETENTION_DAYS: u64 = 365;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ActorType {
    User,
    System,
    Service,
}

impl std::fmt::Display for ActorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User => write!(f, "user"),
            Self::System => write!(f, "system"),
            Self::Service => write!(f, "service"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AuditOutcome {
    Success,
    Failure,
    Error,
}

impl std::fmt::Display for AuditOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Failure => write!(f, "failure"),
            Self::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditLogEntry {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
    pub actor: ActorType,
    pub action: String,
    pub resource: String,
    pub resource_id: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub outcome: AuditOutcome,
    pub details: serde_json::Value,
    pub previous_hash: String,
    pub entry_hash: String,
}

#[derive(Debug, Clone)]
pub struct AuditTrail {
    entries: VecDeque<AuditLogEntry>,
    chain_head: Option<String>,
    retention_days: u64,
    next_id: u64,
    prune_boundary: String,
}

impl Default for AuditTrail {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditTrail {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            chain_head: None,
            retention_days: DEFAULT_RETENTION_DAYS,
            next_id: 1,
            prune_boundary: GENESIS_HASH.to_string(),
        }
    }

    pub fn with_retention(retention_days: u64) -> Self {
        Self {
            retention_days,
            ..Self::new()
        }
    }

    pub fn append(&mut self, mut entry: AuditLogEntry) -> &AuditLogEntry {
        entry.id = self.next_id;
        self.next_id += 1;
        if entry.timestamp == DateTime::<Utc>::default() {
            entry.timestamp = Utc::now();
        }
        entry.previous_hash = self
            .chain_head
            .clone()
            .unwrap_or_else(|| GENESIS_HASH.to_string());
        let hash = compute_entry_hash(&entry);
        entry.entry_hash = hash.clone();
        self.chain_head = Some(hash);
        self.entries.push_back(entry);
        self.entries.back().unwrap()
    }

    pub fn verify_chain(&self) -> bool {
        let mut expected_prev = self.prune_boundary.clone();
        for entry in &self.entries {
            if entry.previous_hash != expected_prev {
                return false;
            }
            let computed = compute_entry_hash(entry);
            if computed != entry.entry_hash {
                return false;
            }
            expected_prev = entry.entry_hash.clone();
        }
        true
    }

    pub fn export_json(&self) -> String {
        serde_json::to_string_pretty(&self.entries).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn export_csv(&self) -> String {
        let mut csv = String::from(
            "id,timestamp,actor,action,resource,resource_id,ip_address,user_agent,outcome,details,previous_hash,entry_hash\n",
        );
        for e in &self.entries {
            let ip = e.ip_address.as_deref().unwrap_or("");
            let ua = e.user_agent.as_deref().unwrap_or("");
            let details = serde_json::to_string(&e.details).unwrap_or_default();
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{},{}\n",
                e.id,
                e.timestamp.to_rfc3339(),
                e.actor,
                e.action,
                e.resource,
                e.resource_id,
                ip,
                ua,
                e.outcome,
                details,
                e.previous_hash,
                e.entry_hash
            ));
        }
        csv
    }

    pub fn query(
        &self,
        actor: Option<&ActorType>,
        action: Option<&str>,
        resource: Option<&str>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Vec<&AuditLogEntry> {
        self.entries
            .iter()
            .filter(|e| {
                if let Some(a) = actor {
                    if &e.actor != a {
                        return false;
                    }
                }
                if let Some(a) = action {
                    if e.action != a {
                        return false;
                    }
                }
                if let Some(r) = resource {
                    if e.resource != r {
                        return false;
                    }
                }
                if let Some(f) = from {
                    if e.timestamp < f {
                        return false;
                    }
                }
                if let Some(t) = to {
                    if e.timestamp > t {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    pub fn prune_retention(&mut self) -> usize {
        let cutoff = Utc::now() - chrono::Duration::days(self.retention_days as i64);
        let before = self.entries.len();
        while let Some(front) = self.entries.front() {
            if front.timestamp < cutoff {
                self.prune_boundary = front.entry_hash.clone();
                self.entries.pop_front();
            } else {
                break;
            }
        }
        before - self.entries.len()
    }

    pub fn get_head_hash(&self) -> Option<&str> {
        self.chain_head.as_deref()
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn entries(&self) -> &VecDeque<AuditLogEntry> {
        &self.entries
    }
}

fn compute_entry_hash(entry: &AuditLogEntry) -> String {
    let data = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        entry.id,
        entry.timestamp.to_rfc3339(),
        entry.actor,
        entry.action,
        entry.resource,
        entry.resource_id,
        entry.ip_address.as_deref().unwrap_or(""),
        entry.user_agent.as_deref().unwrap_or(""),
        entry.outcome,
        serde_json::to_string(&entry.details).unwrap_or_default(),
        entry.previous_hash
    );
    let hash = Sha256::digest(data.as_bytes());
    hex::encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(actor: ActorType, action: &str, resource: &str) -> AuditLogEntry {
        AuditLogEntry {
            id: 0,
            timestamp: Utc::now(),
            actor,
            action: action.to_string(),
            resource: resource.to_string(),
            resource_id: "res-001".to_string(),
            ip_address: Some("10.0.0.1".to_string()),
            user_agent: Some("test-agent/1.0".to_string()),
            outcome: AuditOutcome::Success,
            details: serde_json::Value::Object(serde_json::Map::new()),
            previous_hash: String::new(),
            entry_hash: String::new(),
        }
    }

    #[test]
    fn test_append_single_entry() {
        let mut trail = AuditTrail::new();
        let entry = make_entry(ActorType::User, "login", "session");
        let result = trail.append(entry);
        assert_eq!(result.id, 1);
        assert_eq!(result.actor, ActorType::User);
        assert_eq!(result.action, "login");
        assert_eq!(trail.entry_count(), 1);
    }

    #[test]
    fn test_chain_hash_linking() {
        let mut trail = AuditTrail::new();
        trail.append(make_entry(ActorType::User, "a", "r"));
        trail.append(make_entry(ActorType::System, "b", "r"));
        let entries = trail.entries();
        let r1 = &entries[0];
        let r2 = &entries[1];
        assert_eq!(r1.previous_hash, GENESIS_HASH);
        assert_eq!(r2.previous_hash, r1.entry_hash);
        assert_ne!(r1.entry_hash, r2.entry_hash);
    }

    #[test]
    fn test_verify_chain_valid() {
        let mut trail = AuditTrail::new();
        trail.append(make_entry(ActorType::User, "login", "session"));
        trail.append(make_entry(ActorType::Service, "deploy", "app"));
        assert!(trail.verify_chain());
    }

    #[test]
    fn test_verify_chain_empty() {
        let trail = AuditTrail::new();
        assert!(trail.verify_chain());
    }

    #[test]
    fn test_detect_tampering_action() {
        let mut trail = AuditTrail::new();
        trail.append(make_entry(ActorType::User, "login", "session"));
        trail.append(make_entry(ActorType::User, "read", "file"));
        assert!(trail.verify_chain());
        trail.entries.back_mut().unwrap().action = "tampered".to_string();
        assert!(!trail.verify_chain());
    }

    #[test]
    fn test_detect_tampering_actor() {
        let mut trail = AuditTrail::new();
        trail.append(make_entry(ActorType::User, "login", "session"));
        trail.append(make_entry(ActorType::System, "restart", "service"));
        assert!(trail.verify_chain());
        trail.entries.back_mut().unwrap().actor = ActorType::User;
        assert!(!trail.verify_chain());
    }

    #[test]
    fn test_detect_tampering_hash() {
        let mut trail = AuditTrail::new();
        trail.append(make_entry(ActorType::User, "login", "session"));
        assert!(trail.verify_chain());
        trail.entries.back_mut().unwrap().entry_hash = "deadbeef".to_string();
        assert!(!trail.verify_chain());
    }

    #[test]
    fn test_detect_tampering_previous_hash() {
        let mut trail = AuditTrail::new();
        trail.append(make_entry(ActorType::User, "a", "r"));
        trail.append(make_entry(ActorType::User, "b", "r"));
        assert!(trail.verify_chain());
        trail.entries.back_mut().unwrap().previous_hash = GENESIS_HASH.to_string();
        assert!(!trail.verify_chain());
    }

    #[test]
    fn test_detect_tampering_details() {
        let mut trail = AuditTrail::new();
        trail.append(make_entry(ActorType::User, "login", "session"));
        assert!(trail.verify_chain());
        trail.entries.back_mut().unwrap().details = serde_json::json!({"tampered": true});
        assert!(!trail.verify_chain());
    }

    #[test]
    fn test_large_chain_integrity() {
        let mut trail = AuditTrail::new();
        for i in 0..100 {
            trail.append(make_entry(
                ActorType::Service,
                &format!("action-{i}"),
                "resource",
            ));
        }
        assert_eq!(trail.entry_count(), 100);
        assert!(trail.verify_chain());
    }

    #[test]
    fn test_query_by_actor() {
        let mut trail = AuditTrail::new();
        trail.append(make_entry(ActorType::User, "login", "session"));
        trail.append(make_entry(ActorType::System, "cron", "job"));
        trail.append(make_entry(ActorType::User, "logout", "session"));
        let results = trail.query(Some(&ActorType::User), None, None, None, None);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_by_action() {
        let mut trail = AuditTrail::new();
        trail.append(make_entry(ActorType::User, "login", "session"));
        trail.append(make_entry(ActorType::User, "read", "file"));
        trail.append(make_entry(ActorType::User, "read", "file"));
        let results = trail.query(None, Some("read"), None, None, None);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_by_resource() {
        let mut trail = AuditTrail::new();
        trail.append(make_entry(ActorType::User, "read", "file"));
        trail.append(make_entry(ActorType::User, "read", "database"));
        trail.append(make_entry(ActorType::User, "write", "file"));
        let results = trail.query(None, None, Some("file"), None, None);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_by_time_range() {
        let mut trail = AuditTrail::new();
        let t1 = Utc::now() - chrono::Duration::days(3);
        let t2 = Utc::now() - chrono::Duration::days(1);
        let t3 = Utc::now();
        let mut e1 = make_entry(ActorType::User, "a", "r");
        e1.timestamp = t1;
        let mut e2 = make_entry(ActorType::User, "b", "r");
        e2.timestamp = t2;
        let mut e3 = make_entry(ActorType::User, "c", "r");
        e3.timestamp = t3;
        trail.append(e1);
        trail.append(e2);
        trail.append(e3);
        let from = t2 - chrono::Duration::seconds(1);
        let to = t3 + chrono::Duration::seconds(1);
        let results = trail.query(None, None, None, Some(from), Some(to));
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_empty_result() {
        let trail = AuditTrail::new();
        let results = trail.query(Some(&ActorType::User), None, None, None, None);
        assert!(results.is_empty());
    }

    #[test]
    fn test_query_combined_filters() {
        let mut trail = AuditTrail::new();
        trail.append(make_entry(ActorType::User, "login", "session"));
        trail.append(make_entry(ActorType::System, "login", "session"));
        trail.append(make_entry(ActorType::User, "logout", "session"));
        let results = trail.query(Some(&ActorType::User), Some("login"), None, None, None);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_export_json() {
        let mut trail = AuditTrail::new();
        trail.append(make_entry(ActorType::User, "login", "session"));
        let json = trail.export_json();
        assert!(json.contains("login"));
        assert!(json.contains("session"));
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 1);
    }

    #[test]
    fn test_export_json_empty() {
        let trail = AuditTrail::new();
        let json = trail.export_json();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn test_export_csv() {
        let mut trail = AuditTrail::new();
        trail.append(make_entry(ActorType::User, "login", "session"));
        let csv = trail.export_csv();
        assert!(csv.starts_with("id,timestamp,actor,action,resource"));
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[1].contains("login"));
        assert!(lines[1].contains("10.0.0.1"));
    }

    #[test]
    fn test_export_csv_empty() {
        let trail = AuditTrail::new();
        let csv = trail.export_csv();
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_export_csv_multiple() {
        let mut trail = AuditTrail::new();
        trail.append(make_entry(ActorType::User, "a", "r"));
        trail.append(make_entry(ActorType::System, "b", "r"));
        let csv = trail.export_csv();
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_prune_retention_removes_old() {
        let mut trail = AuditTrail::with_retention(1);
        let old = Utc::now() - chrono::Duration::days(2);
        let mut e1 = make_entry(ActorType::User, "old", "r");
        e1.timestamp = old;
        let mut e2 = make_entry(ActorType::User, "new", "r");
        e2.timestamp = Utc::now();
        trail.append(e1);
        trail.append(e2);
        assert_eq!(trail.entry_count(), 2);
        let pruned = trail.prune_retention();
        assert_eq!(pruned, 1);
        assert_eq!(trail.entry_count(), 1);
    }

    #[test]
    fn test_prune_retention_none_removed() {
        let mut trail = AuditTrail::with_retention(365);
        trail.append(make_entry(ActorType::User, "new", "r"));
        let pruned = trail.prune_retention();
        assert_eq!(pruned, 0);
        assert_eq!(trail.entry_count(), 1);
    }

    #[test]
    fn test_prune_retention_empty() {
        let mut trail = AuditTrail::with_retention(1);
        let pruned = trail.prune_retention();
        assert_eq!(pruned, 0);
    }

    #[test]
    fn test_prune_retention_after_tampering_check() {
        let mut trail = AuditTrail::with_retention(1);
        let old = Utc::now() - chrono::Duration::days(5);
        let mut e = make_entry(ActorType::User, "old", "r");
        e.timestamp = old;
        trail.append(e);
        trail.append(make_entry(ActorType::User, "new", "r"));
        assert!(trail.verify_chain());
        trail.prune_retention();
        assert!(trail.verify_chain());
    }

    #[test]
    fn test_get_head_hash() {
        let mut trail = AuditTrail::new();
        assert!(trail.get_head_hash().is_none());
        trail.append(make_entry(ActorType::User, "login", "session"));
        assert!(trail.get_head_hash().is_some());
        assert_ne!(trail.get_head_hash().unwrap(), GENESIS_HASH);
    }

    #[test]
    fn test_entry_count() {
        let mut trail = AuditTrail::new();
        assert_eq!(trail.entry_count(), 0);
        trail.append(make_entry(ActorType::User, "a", "r"));
        assert_eq!(trail.entry_count(), 1);
        trail.append(make_entry(ActorType::User, "b", "r"));
        assert_eq!(trail.entry_count(), 2);
    }

    #[test]
    fn test_actor_type_display() {
        assert_eq!(ActorType::User.to_string(), "user");
        assert_eq!(ActorType::System.to_string(), "system");
        assert_eq!(ActorType::Service.to_string(), "service");
    }

    #[test]
    fn test_audit_outcome_display() {
        assert_eq!(AuditOutcome::Success.to_string(), "success");
        assert_eq!(AuditOutcome::Failure.to_string(), "failure");
        assert_eq!(AuditOutcome::Error.to_string(), "error");
    }

    #[test]
    fn test_all_outcomes() {
        let mut trail = AuditTrail::new();
        let mut e1 = make_entry(ActorType::User, "a", "r");
        e1.outcome = AuditOutcome::Success;
        let mut e2 = make_entry(ActorType::User, "b", "r");
        e2.outcome = AuditOutcome::Failure;
        let mut e3 = make_entry(ActorType::System, "c", "r");
        e3.outcome = AuditOutcome::Error;
        trail.append(e1);
        trail.append(e2);
        trail.append(e3);
        assert_eq!(trail.entry_count(), 3);
        assert!(trail.verify_chain());
    }
}
