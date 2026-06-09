#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::fmt;

pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const DEFAULT_RETENTION_DAYS: u64 = 365;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AuditOutcome {
    Success,
    Failure,
    Denied,
    Error,
}

impl fmt::Display for AuditOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Failure => write!(f, "failure"),
            Self::Denied => write!(f, "denied"),
            Self::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
    pub actor_id: String,
    pub action: String,
    pub resource_type: String,
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
    entries: Vec<AuditEntry>,
    head_hash: String,
    next_id: u64,
    retention_days: u64,
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
            entries: Vec::new(),
            head_hash: GENESIS_HASH.to_string(),
            next_id: 1,
            retention_days: DEFAULT_RETENTION_DAYS,
            prune_boundary: GENESIS_HASH.to_string(),
        }
    }

    pub fn with_retention(retention_days: u64) -> Self {
        Self {
            retention_days,
            prune_boundary: GENESIS_HASH.to_string(),
            ..Self::new()
        }
    }

    pub fn append(&mut self, builder: AuditEntryBuilder) -> anyhow::Result<AuditEntry> {
        let entry = builder.build(self.next_id, self.head_hash.clone())?;
        self.next_id += 1;
        let hash = compute_entry_hash(&entry);
        let mut entry = entry;
        entry.entry_hash = hash.clone();
        self.head_hash = hash;
        let id = entry.id;
        self.entries.push(entry.clone());
        Ok(self.entries.iter().find(|e| e.id == id).unwrap().clone())
    }

    pub fn verify_chain(&self) -> ChainVerification {
        let mut expected_prev = GENESIS_HASH.to_string();
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.previous_hash != expected_prev {
                return ChainVerification {
                    valid: false,
                    entries_verified: i,
                    first_invalid_index: Some(i),
                    head_hash: self.head_hash.clone(),
                };
            }
            if !verify_entry_hash(entry) {
                return ChainVerification {
                    valid: false,
                    entries_verified: i,
                    first_invalid_index: Some(i),
                    head_hash: self.head_hash.clone(),
                };
            }
            expected_prev = entry.entry_hash.clone();
        }
        ChainVerification {
            valid: true,
            entries_verified: self.entries.len(),
            first_invalid_index: None,
            head_hash: self.head_hash.clone(),
        }
    }

    pub fn export_json(&self) -> String {
        serde_json::to_string_pretty(&self.entries).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn export_csv(&self) -> String {
        let mut csv = String::from(
            "id,timestamp,actor_id,action,resource_type,resource_id,ip_address,user_agent,outcome,details,previous_hash,entry_hash\n",
        );
        for e in &self.entries {
            let ip = e.ip_address.as_deref().unwrap_or("");
            let ua = e.user_agent.as_deref().unwrap_or("");
            let details = serde_json::to_string(&e.details).unwrap_or_default();
            let details_truncated = if details.len() > 8 {
                format!("{}...", hex::encode(&details.as_bytes()[..8]))
            } else {
                hex::encode(details.as_bytes())
            };
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{},{}\n",
                e.id,
                e.timestamp.to_rfc3339(),
                e.actor_id,
                e.action,
                e.resource_type,
                e.resource_id,
                ip,
                ua,
                e.outcome,
                details_truncated,
                e.previous_hash,
                e.entry_hash
            ));
        }
        csv
    }

    pub fn query(
        &self,
        actor: Option<&str>,
        action: Option<&str>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| {
                if let Some(a) = actor {
                    if e.actor_id != a {
                        return false;
                    }
                }
                if let Some(a) = action {
                    if e.action != a {
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

    pub fn verify_entry_hash(entry: &AuditEntry) -> bool {
        let computed = compute_entry_hash(entry);
        computed == entry.entry_hash
    }

    pub fn merkle_root(entries: &[AuditEntry]) -> String {
        if entries.is_empty() {
            return String::new();
        }
        let hashes: Vec<String> = entries.iter().map(|e| e.entry_hash.clone()).collect();
        let mut current: Vec<String> = hashes;
        while current.len() > 1 {
            let mut next = Vec::new();
            for chunk in current.chunks(2) {
                if chunk.len() == 2 {
                    let combined = format!("{}{}", chunk[0], chunk[1]);
                    let hash = Sha256::digest(combined.as_bytes());
                    next.push(hex::encode(hash));
                } else {
                    next.push(chunk[0].clone());
                }
            }
            current = next;
        }
        current.into_iter().next().unwrap_or_default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    pub fn retention_days(&self) -> u64 {
        self.retention_days
    }

    pub fn prune_retention(&mut self) -> usize {
        let cutoff = Utc::now() - chrono::Duration::days(self.retention_days as i64);
        let before = self.entries.len();
        while let Some(front) = self.entries.first() {
            if front.timestamp < cutoff {
                self.prune_boundary = front.entry_hash.clone();
                self.entries.remove(0);
            } else {
                break;
            }
        }
        before - self.entries.len()
    }
}

pub struct AuditEntryBuilder {
    actor_id: String,
    action: String,
    resource_type: String,
    resource_id: String,
    ip_address: Option<String>,
    user_agent: Option<String>,
    outcome: AuditOutcome,
    details: serde_json::Value,
    timestamp: Option<DateTime<Utc>>,
}

impl AuditEntryBuilder {
    pub fn new(
        actor_id: impl Into<String>,
        action: impl Into<String>,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            action: action.into(),
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            ip_address: None,
            user_agent: None,
            outcome: AuditOutcome::Success,
            details: serde_json::Value::Object(serde_json::Map::new()),
            timestamp: None,
        }
    }

    pub fn ip_address(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    pub fn outcome(mut self, outcome: AuditOutcome) -> Self {
        self.outcome = outcome;
        self
    }

    pub fn details(mut self, details: serde_json::Value) -> Self {
        self.details = details;
        self
    }

    pub fn timestamp(mut self, ts: DateTime<Utc>) -> Self {
        self.timestamp = Some(ts);
        self
    }

    fn build(self, id: u64, previous_hash: String) -> anyhow::Result<AuditEntry> {
        Ok(AuditEntry {
            id,
            timestamp: self.timestamp.unwrap_or(Utc::now()),
            actor_id: self.actor_id,
            action: self.action,
            resource_type: self.resource_type,
            resource_id: self.resource_id,
            ip_address: self.ip_address,
            user_agent: self.user_agent,
            outcome: self.outcome,
            details: self.details,
            previous_hash,
            entry_hash: String::new(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ChainVerification {
    pub valid: bool,
    pub entries_verified: usize,
    pub first_invalid_index: Option<usize>,
    pub head_hash: String,
}

fn compute_entry_hash(entry: &AuditEntry) -> String {
    let data = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        entry.id,
        entry.timestamp.to_rfc3339(),
        entry.actor_id,
        entry.action,
        entry.resource_type,
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

fn verify_entry_hash(entry: &AuditEntry) -> bool {
    let computed = compute_entry_hash(entry);
    computed == entry.entry_hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_entry() {
        let mut trail = AuditTrail::new();
        let entry = trail
            .append(
                AuditEntryBuilder::new("user-1", "login", "session", "sess-001")
                    .outcome(AuditOutcome::Success)
                    .ip_address("10.0.0.1"),
            )
            .unwrap();
        assert_eq!(entry.id, 1);
        assert_eq!(entry.actor_id, "user-1");
        assert_eq!(entry.action, "login");
        assert_eq!(entry.outcome, AuditOutcome::Success);
        assert!(!entry.entry_hash.is_empty());
        assert_eq!(trail.len(), 1);
    }

    #[test]
    fn test_verify_chain_valid() {
        let mut trail = AuditTrail::new();
        trail
            .append(AuditEntryBuilder::new("user-1", "login", "session", "s1"))
            .unwrap();
        trail
            .append(AuditEntryBuilder::new("user-1", "read", "file", "f1"))
            .unwrap();
        let verification = trail.verify_chain();
        assert!(verification.valid);
        assert_eq!(verification.entries_verified, 2);
        assert!(verification.first_invalid_index.is_none());
    }

    #[test]
    fn test_detect_tampering() {
        let mut trail = AuditTrail::new();
        trail
            .append(AuditEntryBuilder::new("user-1", "login", "session", "s1"))
            .unwrap();
        trail
            .append(AuditEntryBuilder::new("user-1", "read", "file", "f1"))
            .unwrap();
        let verification = trail.verify_chain();
        assert!(verification.valid);

        trail.entries.last_mut().unwrap().action = "tampered".to_string();

        let verification = trail.verify_chain();
        assert!(!verification.valid);
        assert_eq!(verification.first_invalid_index, Some(1));
    }

    #[test]
    fn test_query_by_actor() {
        let mut trail = AuditTrail::new();
        trail
            .append(AuditEntryBuilder::new("user-1", "login", "session", "s1"))
            .unwrap();
        trail
            .append(AuditEntryBuilder::new("user-2", "login", "session", "s2"))
            .unwrap();
        trail
            .append(AuditEntryBuilder::new("user-1", "read", "file", "f1"))
            .unwrap();
        let results = trail.query(Some("user-1"), None, None, None);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_by_action() {
        let mut trail = AuditTrail::new();
        trail
            .append(AuditEntryBuilder::new("user-1", "login", "session", "s1"))
            .unwrap();
        trail
            .append(AuditEntryBuilder::new("user-1", "read", "file", "f1"))
            .unwrap();
        trail
            .append(AuditEntryBuilder::new("user-1", "read", "file", "f2"))
            .unwrap();
        let results = trail.query(None, Some("read"), None, None);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_by_time_range() {
        let mut trail = AuditTrail::new();
        let t1 = Utc::now() - chrono::Duration::days(2);
        let t2 = Utc::now() - chrono::Duration::days(1);
        let t3 = Utc::now();
        trail
            .append(AuditEntryBuilder::new("user-1", "a", "r", "1").timestamp(t1))
            .unwrap();
        trail
            .append(AuditEntryBuilder::new("user-1", "b", "r", "2").timestamp(t2))
            .unwrap();
        trail
            .append(AuditEntryBuilder::new("user-1", "c", "r", "3").timestamp(t3))
            .unwrap();
        let from = t2 - chrono::Duration::seconds(1);
        let to = t3 + chrono::Duration::seconds(1);
        let results = trail.query(None, None, Some(from), Some(to));
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_merkle_root() {
        let mut trail = AuditTrail::new();
        trail
            .append(AuditEntryBuilder::new("user-1", "login", "session", "s1"))
            .unwrap();
        let root = AuditTrail::merkle_root(trail.entries());
        assert!(!root.is_empty());
        let entries = trail.entries();
        assert_eq!(root, entries[0].entry_hash);
    }

    #[test]
    fn test_merkle_root_multiple() {
        let mut trail = AuditTrail::new();
        trail
            .append(AuditEntryBuilder::new("user-1", "a", "r", "1"))
            .unwrap();
        trail
            .append(AuditEntryBuilder::new("user-1", "b", "r", "2"))
            .unwrap();
        let root = AuditTrail::merkle_root(trail.entries());
        assert!(!root.is_empty());
        let entries = trail.entries();
        assert_ne!(root, entries[0].entry_hash);
        assert_ne!(root, entries[1].entry_hash);
    }

    #[test]
    fn test_export_json() {
        let mut trail = AuditTrail::new();
        trail
            .append(AuditEntryBuilder::new("user-1", "login", "session", "s1"))
            .unwrap();
        let json = trail.export_json();
        assert!(json.contains("user-1"));
        assert!(json.contains("login"));
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 1);
    }

    #[test]
    fn test_export_csv() {
        let mut trail = AuditTrail::new();
        trail
            .append(
                AuditEntryBuilder::new("user-1", "login", "session", "s1")
                    .ip_address("10.0.0.1")
                    .user_agent("test-agent"),
            )
            .unwrap();
        let csv = trail.export_csv();
        assert!(csv.starts_with("id,timestamp,actor_id"));
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[1].contains("user-1"));
        assert!(lines[1].contains("10.0.0.1"));
    }

    #[test]
    fn test_empty_trail() {
        let trail = AuditTrail::new();
        assert!(trail.is_empty());
        assert_eq!(trail.len(), 0);
        let verification = trail.verify_chain();
        assert!(verification.valid);
        assert_eq!(verification.entries_verified, 0);
    }

    #[test]
    fn test_chain_hash_linking() {
        let mut trail = AuditTrail::new();
        let e1 = trail
            .append(AuditEntryBuilder::new("u1", "a", "r", "1"))
            .unwrap();
        let e2 = trail
            .append(AuditEntryBuilder::new("u1", "b", "r", "2"))
            .unwrap();
        assert_eq!(e1.previous_hash, GENESIS_HASH);
        assert_eq!(e2.previous_hash, e1.entry_hash);
    }
}
