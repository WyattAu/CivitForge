#![forbid(unsafe_code)]

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub data_type: DataType,
    pub retention_period: Duration,
    pub action: RetentionAction,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    AuditLogs,
    AccessLogs,
    RepositoryData,
    ArtifactData,
    PipelineLogs,
    SessionData,
    TempFiles,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetentionAction {
    Delete,
    Archive,
    ArchiveThenDelete,
    Anonymize,
}

pub struct RetentionEvaluator {
    policies: std::sync::Mutex<Vec<RetentionPolicy>>,
}

impl RetentionEvaluator {
    pub fn new() -> Self {
        Self {
            policies: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn add_policy(&self, policy: RetentionPolicy) {
        let mut policies = self.policies.lock().unwrap();
        policies.push(policy);
    }

    pub fn remove_policy(&self, id: &str) -> bool {
        let mut policies = self.policies.lock().unwrap();
        let before = policies.len();
        policies.retain(|p| p.id != id);
        policies.len() < before
    }

    pub fn evaluate(&self, data_type: DataType, created_at: DateTime<Utc>) -> Vec<RetentionPolicy> {
        let policies = self.policies.lock().unwrap();
        let now = Utc::now();
        policies
            .iter()
            .filter(|p| p.enabled && p.data_type == data_type)
            .filter(|p| now - created_at > p.retention_period)
            .cloned()
            .collect()
    }

    pub fn policies_for(&self, data_type: DataType) -> Vec<RetentionPolicy> {
        let policies = self.policies.lock().unwrap();
        policies
            .iter()
            .filter(|p| p.data_type == data_type)
            .cloned()
            .collect()
    }

    pub fn all_policies(&self) -> Vec<RetentionPolicy> {
        let policies = self.policies.lock().unwrap();
        policies.clone()
    }
}

impl Default for RetentionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_policy(
        id: &str,
        name: &str,
        data_type: DataType,
        period: Duration,
        action: RetentionAction,
        enabled: bool,
    ) -> RetentionPolicy {
        let now = Utc::now();
        RetentionPolicy {
            id: id.to_string(),
            name: name.to_string(),
            description: format!("Policy: {name}"),
            data_type,
            retention_period: period,
            action,
            enabled,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn test_create_retention_policy() {
        let policy = make_policy(
            "p1",
            "Audit Cleanup",
            DataType::AuditLogs,
            Duration::days(90),
            RetentionAction::Archive,
            true,
        );
        assert_eq!(policy.id, "p1");
        assert_eq!(policy.data_type, DataType::AuditLogs);
        assert!(policy.enabled);
    }

    #[test]
    fn test_evaluator_new_empty() {
        let eval = RetentionEvaluator::new();
        assert!(eval.all_policies().is_empty());
    }

    #[test]
    fn test_evaluator_default_empty() {
        let eval = RetentionEvaluator::default();
        assert!(eval.all_policies().is_empty());
    }

    #[test]
    fn test_add_policy() {
        let eval = RetentionEvaluator::new();
        eval.add_policy(make_policy(
            "p1",
            "Audit",
            DataType::AuditLogs,
            Duration::days(90),
            RetentionAction::Delete,
            true,
        ));
        assert_eq!(eval.all_policies().len(), 1);
    }

    #[test]
    fn test_add_multiple_policies() {
        let eval = RetentionEvaluator::new();
        eval.add_policy(make_policy(
            "p1",
            "Audit",
            DataType::AuditLogs,
            Duration::days(90),
            RetentionAction::Delete,
            true,
        ));
        eval.add_policy(make_policy(
            "p2",
            "Access",
            DataType::AccessLogs,
            Duration::days(30),
            RetentionAction::Archive,
            true,
        ));
        eval.add_policy(make_policy(
            "p3",
            "Temp",
            DataType::TempFiles,
            Duration::days(1),
            RetentionAction::Delete,
            true,
        ));
        assert_eq!(eval.all_policies().len(), 3);
    }

    #[test]
    fn test_remove_policy() {
        let eval = RetentionEvaluator::new();
        eval.add_policy(make_policy(
            "p1",
            "Audit",
            DataType::AuditLogs,
            Duration::days(90),
            RetentionAction::Delete,
            true,
        ));
        assert!(eval.remove_policy("p1"));
        assert!(eval.all_policies().is_empty());
    }

    #[test]
    fn test_remove_nonexistent_policy() {
        let eval = RetentionEvaluator::new();
        assert!(!eval.remove_policy("nonexistent"));
    }

    #[test]
    fn test_remove_one_of_many() {
        let eval = RetentionEvaluator::new();
        eval.add_policy(make_policy(
            "p1",
            "Audit",
            DataType::AuditLogs,
            Duration::days(90),
            RetentionAction::Delete,
            true,
        ));
        eval.add_policy(make_policy(
            "p2",
            "Access",
            DataType::AccessLogs,
            Duration::days(30),
            RetentionAction::Archive,
            true,
        ));
        eval.add_policy(make_policy(
            "p3",
            "Temp",
            DataType::TempFiles,
            Duration::days(1),
            RetentionAction::Delete,
            true,
        ));
        assert!(eval.remove_policy("p2"));
        assert_eq!(eval.all_policies().len(), 2);
        assert_eq!(eval.all_policies()[0].id, "p1");
        assert_eq!(eval.all_policies()[1].id, "p3");
    }

    #[test]
    fn test_evaluate_expired_audit_logs() {
        let eval = RetentionEvaluator::new();
        eval.add_policy(make_policy(
            "p1",
            "Audit",
            DataType::AuditLogs,
            Duration::days(90),
            RetentionAction::Archive,
            true,
        ));
        let created_at = Utc::now() - Duration::days(91);
        let result = eval.evaluate(DataType::AuditLogs, created_at);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "p1");
    }

    #[test]
    fn test_evaluate_not_expired() {
        let eval = RetentionEvaluator::new();
        eval.add_policy(make_policy(
            "p1",
            "Audit",
            DataType::AuditLogs,
            Duration::days(90),
            RetentionAction::Archive,
            true,
        ));
        let created_at = Utc::now() - Duration::days(30);
        let result = eval.evaluate(DataType::AuditLogs, created_at);
        assert!(result.is_empty());
    }

    #[test]
    fn test_evaluate_disabled_policy() {
        let eval = RetentionEvaluator::new();
        eval.add_policy(make_policy(
            "p1",
            "Audit",
            DataType::AuditLogs,
            Duration::days(90),
            RetentionAction::Delete,
            false,
        ));
        let created_at = Utc::now() - Duration::days(100);
        let result = eval.evaluate(DataType::AuditLogs, created_at);
        assert!(result.is_empty());
    }

    #[test]
    fn test_evaluate_wrong_data_type() {
        let eval = RetentionEvaluator::new();
        eval.add_policy(make_policy(
            "p1",
            "Audit",
            DataType::AuditLogs,
            Duration::days(90),
            RetentionAction::Delete,
            true,
        ));
        let created_at = Utc::now() - Duration::days(100);
        let result = eval.evaluate(DataType::TempFiles, created_at);
        assert!(result.is_empty());
    }

    #[test]
    fn test_evaluate_multiple_matching_policies() {
        let eval = RetentionEvaluator::new();
        eval.add_policy(make_policy(
            "p1",
            "Archive Audit",
            DataType::AuditLogs,
            Duration::days(90),
            RetentionAction::Archive,
            true,
        ));
        eval.add_policy(make_policy(
            "p2",
            "Delete Audit",
            DataType::AuditLogs,
            Duration::days(180),
            RetentionAction::Delete,
            true,
        ));
        let created_at = Utc::now() - Duration::days(200);
        let result = eval.evaluate(DataType::AuditLogs, created_at);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_evaluate_partial_match_by_age() {
        let eval = RetentionEvaluator::new();
        eval.add_policy(make_policy(
            "p1",
            "Archive Audit",
            DataType::AuditLogs,
            Duration::days(90),
            RetentionAction::Archive,
            true,
        ));
        eval.add_policy(make_policy(
            "p2",
            "Delete Audit",
            DataType::AuditLogs,
            Duration::days(180),
            RetentionAction::Delete,
            true,
        ));
        let created_at = Utc::now() - Duration::days(100);
        let result = eval.evaluate(DataType::AuditLogs, created_at);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "p1");
    }

    #[test]
    fn test_policies_for_type() {
        let eval = RetentionEvaluator::new();
        eval.add_policy(make_policy(
            "p1",
            "Audit",
            DataType::AuditLogs,
            Duration::days(90),
            RetentionAction::Archive,
            true,
        ));
        eval.add_policy(make_policy(
            "p2",
            "Access",
            DataType::AccessLogs,
            Duration::days(30),
            RetentionAction::Delete,
            true,
        ));
        eval.add_policy(make_policy(
            "p3",
            "Audit2",
            DataType::AuditLogs,
            Duration::days(365),
            RetentionAction::ArchiveThenDelete,
            true,
        ));
        let audit_policies = eval.policies_for(DataType::AuditLogs);
        assert_eq!(audit_policies.len(), 2);
        let access_policies = eval.policies_for(DataType::AccessLogs);
        assert_eq!(access_policies.len(), 1);
    }

    #[test]
    fn test_policies_for_empty() {
        let eval = RetentionEvaluator::new();
        assert!(eval.policies_for(DataType::AuditLogs).is_empty());
    }

    #[test]
    fn test_all_policies() {
        let eval = RetentionEvaluator::new();
        eval.add_policy(make_policy(
            "p1",
            "A",
            DataType::AuditLogs,
            Duration::days(1),
            RetentionAction::Delete,
            true,
        ));
        eval.add_policy(make_policy(
            "p2",
            "B",
            DataType::TempFiles,
            Duration::days(1),
            RetentionAction::Delete,
            true,
        ));
        let all = eval.all_policies();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_data_type_serialization() {
        assert_eq!(
            serde_json::to_string(&DataType::AuditLogs).unwrap(),
            "\"AuditLogs\""
        );
        assert_eq!(
            serde_json::to_string(&DataType::TempFiles).unwrap(),
            "\"TempFiles\""
        );
    }

    #[test]
    fn test_data_type_equality() {
        assert_eq!(DataType::AuditLogs, DataType::AuditLogs);
        assert_ne!(DataType::AuditLogs, DataType::AccessLogs);
    }

    #[test]
    fn test_retention_action_serialization() {
        assert_eq!(
            serde_json::to_string(&RetentionAction::Delete).unwrap(),
            "\"Delete\""
        );
        assert_eq!(
            serde_json::to_string(&RetentionAction::Archive).unwrap(),
            "\"Archive\""
        );
        assert_eq!(
            serde_json::to_string(&RetentionAction::ArchiveThenDelete).unwrap(),
            "\"ArchiveThenDelete\""
        );
        assert_eq!(
            serde_json::to_string(&RetentionAction::Anonymize).unwrap(),
            "\"Anonymize\""
        );
    }

    #[test]
    fn test_retention_action_equality() {
        assert_eq!(RetentionAction::Delete, RetentionAction::Delete);
        assert_ne!(RetentionAction::Archive, RetentionAction::Delete);
        assert_ne!(RetentionAction::Archive, RetentionAction::ArchiveThenDelete);
    }

    #[test]
    fn test_policy_serialization_roundtrip() {
        let policy = make_policy(
            "p1",
            "Test Policy",
            DataType::PipelineLogs,
            Duration::days(60),
            RetentionAction::Anonymize,
            true,
        );
        let json = serde_json::to_string(&policy).unwrap();
        let deserialized: RetentionPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(policy.id, deserialized.id);
        assert_eq!(policy.data_type, deserialized.data_type);
        assert_eq!(policy.action, deserialized.action);
        assert_eq!(policy.enabled, deserialized.enabled);
    }

    #[test]
    fn test_evaluate_at_exact_boundary() {
        let eval = RetentionEvaluator::new();
        eval.add_policy(make_policy(
            "p1",
            "Exact",
            DataType::SessionData,
            Duration::days(7),
            RetentionAction::Delete,
            true,
        ));
        let created_at = Utc::now() - Duration::days(6);
        let result = eval.evaluate(DataType::SessionData, created_at);
        assert!(result.is_empty());
    }
}
