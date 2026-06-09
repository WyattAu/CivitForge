#![forbid(unsafe_code)]

pub use crate::audit::trail::{
    AuditEntry, AuditEntryBuilder, AuditOutcome, AuditTrail, ChainVerification, GENESIS_HASH,
};

pub type AuditLogEntry = AuditEntry;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_reexported_types_are_accessible() {
        let mut trail = AuditTrail::new();
        let entry = trail
            .append(
                AuditEntryBuilder::new("user-1", "login", "session", "sess-001")
                    .outcome(AuditOutcome::Success),
            )
            .unwrap();
        assert_eq!(entry.id, 1);
        assert_eq!(entry.actor_id, "user-1");
    }

    #[test]
    fn test_retention_via_reexport() {
        let mut trail = AuditTrail::with_retention(1);
        let old = Utc::now() - chrono::Duration::days(2);
        trail
            .append(
                AuditEntryBuilder::new("user-1", "old", "r", "1").timestamp(old),
            )
            .unwrap();
        trail
            .append(AuditEntryBuilder::new("user-1", "new", "r", "2"))
            .unwrap();
        assert_eq!(trail.len(), 2);
        let pruned = trail.prune_retention();
        assert_eq!(pruned, 1);
        assert_eq!(trail.len(), 1);
    }

    #[test]
    fn test_audit_log_entry_alias() {
        let entry = AuditLogEntry {
            id: 1,
            timestamp: Utc::now(),
            actor_id: "user-1".to_string(),
            action: "test".to_string(),
            resource_type: "test".to_string(),
            resource_id: "1".to_string(),
            ip_address: None,
            user_agent: None,
            outcome: AuditOutcome::Success,
            details: serde_json::json!({}),
            previous_hash: String::new(),
            entry_hash: String::new(),
        };
        assert_eq!(entry.id, 1);
    }
}
