#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::security_scanner_v22::SecurityScanRuleV22;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanRuleSetV23 {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rules: Vec<SecurityScanRuleV22>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

impl SecurityScanRuleSetV23 {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            rules: Vec::new(),
            enabled: true,
            created_at: Utc::now(),
        }
    }

    pub fn with_rules(mut self, rules: Vec<SecurityScanRuleV22>) -> Self {
        self.rules = rules;
        self
    }

    pub fn add_rule(&mut self, rule: SecurityScanRuleV22) {
        self.rules.push(rule);
    }

    pub fn enabled_rules(&self) -> Vec<&SecurityScanRuleV22> {
        self.rules.iter().filter(|r| r.enabled).collect()
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanDedupEntryV23 {
    pub id: String,
    pub vulnerability_id: String,
    pub repo_id: String,
    pub file_path: String,
    pub line_number: Option<u32>,
    pub dedup_hash: String,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

impl SecurityScanDedupEntryV23 {
    pub fn new(
        vulnerability_id: String,
        repo_id: String,
        file_path: String,
        line_number: Option<u32>,
    ) -> Self {
        let now = Utc::now();
        let dedup_hash = format!(
            "{}:{}:{}:{}",
            vulnerability_id,
            repo_id,
            file_path,
            line_number.map_or("0".to_string(), |l| l.to_string())
        );
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            vulnerability_id,
            repo_id,
            file_path,
            line_number,
            dedup_hash,
            first_seen_at: now,
            last_seen_at: now,
        }
    }

    pub fn touch(&mut self) {
        self.last_seen_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsePositiveRecordV23 {
    pub id: String,
    pub vulnerability_id: String,
    pub rule_id: String,
    pub reason: String,
    pub marked_by: Option<String>,
    pub marked_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub active: bool,
}

impl FalsePositiveRecordV23 {
    pub fn new(
        vulnerability_id: String,
        rule_id: String,
        reason: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            vulnerability_id,
            rule_id,
            reason,
            marked_by: None,
            marked_at: Utc::now(),
            expires_at: None,
            active: true,
        }
    }

    pub fn with_marked_by(mut self, user_id: &str) -> Self {
        self.marked_by = Some(user_id.into());
        self
    }

    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |exp| Utc::now() > exp)
    }

    pub fn revoke(&mut self) {
        self.active = false;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanScheduleV23 {
    pub id: String,
    pub repo_id: String,
    pub rule_set_id: String,
    pub interval_minutes: u32,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl ScanScheduleV23 {
    pub fn new(repo_id: String, rule_set_id: String, interval_minutes: u32) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            repo_id,
            rule_set_id,
            interval_minutes,
            enabled: true,
            last_run: None,
            next_run: Some(now + chrono::Duration::minutes(interval_minutes as i64)),
            created_at: now,
        }
    }

    pub fn record_run(&mut self) {
        self.last_run = Some(Utc::now());
        self.next_run =
            Some(Utc::now() + chrono::Duration::minutes(self.interval_minutes as i64));
    }

    pub fn is_due(&self) -> bool {
        self.next_run.map_or(false, |next| Utc::now() >= next)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeduplicationEngineV23 {
    entries: Vec<SecurityScanDedupEntryV23>,
    hash_index: HashMap<String, Vec<usize>>,
}

impl DeduplicationEngineV23 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn check_and_record(
        &mut self,
        vulnerability_id: String,
        repo_id: String,
        file_path: String,
        line_number: Option<u32>,
    ) -> DeduplicationResultV23 {
        let entry =
            SecurityScanDedupEntryV23::new(vulnerability_id.clone(), repo_id.clone(), file_path.clone(), line_number);
        let hash = entry.dedup_hash.clone();

        if let Some(indices) = self.hash_index.get(&hash) {
            if let Some(&idx) = indices.first() {
                let existing = &mut self.entries[idx];
                existing.touch();
                return DeduplicationResultV23 {
                    is_duplicate: true,
                    entry_id: existing.id.clone(),
                    first_seen: existing.first_seen_at,
                    last_seen: existing.last_seen_at,
                    occurrence_count: indices.len() as u32 + 1,
                };
            }
        }

        let idx = self.entries.len();
        let entry_id = entry.id.clone();
        let first_seen = entry.first_seen_at;
        let last_seen = entry.last_seen_at;
        self.hash_index
            .entry(hash)
            .or_default()
            .push(idx);
        self.entries.push(entry);

        DeduplicationResultV23 {
            is_duplicate: false,
            entry_id,
            first_seen,
            last_seen,
            occurrence_count: 1,
        }
    }

    pub fn get_entries_for_repo(&self, repo_id: &str) -> Vec<&SecurityScanDedupEntryV23> {
        self.entries
            .iter()
            .filter(|e| e.repo_id == repo_id)
            .collect()
    }

    pub fn total_unique(&self) -> usize {
        self.entries.len()
    }

    pub fn total_occurrences(&self) -> usize {
        self.hash_index.values().map(|v| v.len()).sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeduplicationResultV23 {
    pub is_duplicate: bool,
    pub entry_id: String,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub occurrence_count: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FalsePositiveTrackerV23 {
    records: Vec<FalsePositiveRecordV23>,
    by_vulnerability: HashMap<String, Vec<usize>>,
}

impl FalsePositiveTrackerV23 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mark_false_positive(
        &mut self,
        vulnerability_id: String,
        rule_id: String,
        reason: String,
        marked_by: Option<String>,
    ) -> FalsePositiveRecordV23 {
        let mut record = FalsePositiveRecordV23::new(vulnerability_id.clone(), rule_id, reason);
        record.marked_by = marked_by;
        let idx = self.records.len();
        self.by_vulnerability
            .entry(vulnerability_id)
            .or_default()
            .push(idx);
        self.records.push(record.clone());
        record
    }

    pub fn is_false_positive(&self, vulnerability_id: &str) -> bool {
        self.by_vulnerability
            .get(vulnerability_id)
            .map(|indices| {
                indices.iter().any(|&idx| {
                    let r = &self.records[idx];
                    r.active && !r.is_expired()
                })
            })
            .unwrap_or(false)
    }

    pub fn revoke(&mut self, record_id: &str) -> Result<(), String> {
        let record = self
            .records
            .iter_mut()
            .find(|r| r.id == record_id)
            .ok_or("Record not found")?;
        record.revoke();
        Ok(())
    }

    pub fn active_count(&self) -> usize {
        self.records
            .iter()
            .filter(|r| r.active && !r.is_expired())
            .count()
    }

    pub fn get_records_for_vulnerability(
        &self,
        vulnerability_id: &str,
    ) -> Vec<&FalsePositiveRecordV23> {
        self.by_vulnerability
            .get(vulnerability_id)
            .map(|indices| indices.iter().map(|&idx| &self.records[idx]).collect())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanSchedulingEngineV23 {
    schedules: Vec<ScanScheduleV23>,
}

impl ScanSchedulingEngineV23 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_schedule(&mut self, schedule: ScanScheduleV23) {
        self.schedules.push(schedule);
    }

    pub fn get_due_schedules(&self) -> Vec<&ScanScheduleV23> {
        self.schedules
            .iter()
            .filter(|s| s.enabled && s.is_due())
            .collect()
    }

    pub fn record_run(&mut self, schedule_id: &str) -> Result<(), String> {
        let schedule = self
            .schedules
            .iter_mut()
            .find(|s| s.id == schedule_id)
            .ok_or("Schedule not found")?;
        schedule.record_run();
        Ok(())
    }

    pub fn disable_schedule(&mut self, schedule_id: &str) -> Result<(), String> {
        let schedule = self
            .schedules
            .iter_mut()
            .find(|s| s.id == schedule_id)
            .ok_or("Schedule not found")?;
        schedule.enabled = false;
        Ok(())
    }

    pub fn schedules_for_repo(&self, repo_id: &str) -> Vec<&ScanScheduleV23> {
        self.schedules
            .iter()
            .filter(|s| s.repo_id == repo_id)
            .collect()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuleSetManagerV23 {
    rule_sets: Vec<SecurityScanRuleSetV23>,
}

impl RuleSetManagerV23 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_rule_set(&mut self, rule_set: SecurityScanRuleSetV23) {
        self.rule_sets.push(rule_set);
    }

    pub fn get_rule_set(&self, id: &str) -> Option<&SecurityScanRuleSetV23> {
        self.rule_sets.iter().find(|rs| rs.id == id)
    }

    pub fn get_enabled_rule_sets(&self) -> Vec<&SecurityScanRuleSetV23> {
        self.rule_sets.iter().filter(|rs| rs.enabled).collect()
    }

    pub fn list_rule_sets(&self) -> &[SecurityScanRuleSetV23] {
        &self.rule_sets
    }

    pub fn disable_rule_set(&mut self, id: &str) -> Result<(), String> {
        let rs = self
            .rule_sets
            .iter_mut()
            .find(|rs| rs.id == id)
            .ok_or("Rule set not found")?;
        rs.enabled = false;
        Ok(())
    }

    pub fn total_rules(&self) -> usize {
        self.rule_sets.iter().map(|rs| rs.rules.len()).sum()
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::from("=== Security Scan Rule Sets V23 Report ===\n\n");
        report.push_str(&format!("Total Rule Sets: {}\n", self.rule_sets.len()));
        report.push_str(&format!(
            "Enabled Rule Sets: {}\n",
            self.get_enabled_rule_sets().len()
        ));
        report.push_str(&format!("Total Rules: {}\n\n", self.total_rules()));

        for rs in &self.rule_sets {
            report.push_str(&format!(
                "Rule Set: {} ({} rules, {})\n",
                rs.name,
                rs.rules.len(),
                if rs.enabled {
                    "enabled"
                } else {
                    "disabled"
                }
            ));
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_set_v23_new() {
        let rs = SecurityScanRuleSetV23::new("Test".into(), "Desc".into());
        assert_eq!(rs.name, "Test");
        assert!(rs.enabled);
        assert!(rs.rules.is_empty());
    }

    #[test]
    fn test_rule_set_v23_add_rule() {
        let mut rs = SecurityScanRuleSetV23::new("Test".into(), "Desc".into());
        let rule = SecurityScanRuleV22::new("R1".into(), "D".into(), RuleTypeV22::Regex, "p".into());
        rs.add_rule(rule);
        assert_eq!(rs.rule_count(), 1);
    }

    #[test]
    fn test_rule_set_v23_enabled_rules() {
        let mut rs = SecurityScanRuleSetV23::new("Test".into(), "Desc".into());
        let mut r1 = SecurityScanRuleV22::new("R1".into(), "D".into(), RuleTypeV22::Regex, "p".into());
        r1.enabled = true;
        let mut r2 = SecurityScanRuleV22::new("R2".into(), "D".into(), RuleTypeV22::Regex, "p".into());
        r2.enabled = false;
        rs.add_rule(r1);
        rs.add_rule(r2);
        assert_eq!(rs.enabled_rules().len(), 1);
    }

    #[test]
    fn test_dedup_entry_v23_new() {
        let entry =
            SecurityScanDedupEntryV23::new("v1".into(), "r1".into(), "src/main.rs".into(), Some(42));
        assert_eq!(entry.vulnerability_id, "v1");
        assert_eq!(entry.repo_id, "r1");
        assert_eq!(entry.file_path, "src/main.rs");
        assert_eq!(entry.line_number, Some(42));
        assert!(!entry.dedup_hash.is_empty());
    }

    #[test]
    fn test_deduplication_engine_v23_no_duplicate() {
        let mut engine = DeduplicationEngineV23::new();
        let result = engine.check_and_record(
            "v1".into(),
            "r1".into(),
            "src/main.rs".into(),
            Some(42),
        );
        assert!(!result.is_duplicate);
        assert_eq!(result.occurrence_count, 1);
        assert_eq!(engine.total_unique(), 1);
    }

    #[test]
    fn test_deduplication_engine_v23_duplicate() {
        let mut engine = DeduplicationEngineV23::new();
        engine.check_and_record("v1".into(), "r1".into(), "src/main.rs".into(), Some(42));
        let result = engine.check_and_record(
            "v1".into(),
            "r1".into(),
            "src/main.rs".into(),
            Some(42),
        );
        assert!(result.is_duplicate);
        assert_eq!(result.occurrence_count, 2);
        assert_eq!(engine.total_unique(), 1);
    }

    #[test]
    fn test_deduplication_engine_v23_different_repos() {
        let mut engine = DeduplicationEngineV23::new();
        engine.check_and_record("v1".into(), "r1".into(), "src/main.rs".into(), Some(42));
        let result = engine.check_and_record(
            "v1".into(),
            "r2".into(),
            "src/main.rs".into(),
            Some(42),
        );
        assert!(!result.is_duplicate);
        assert_eq!(engine.total_unique(), 2);
    }

    #[test]
    fn test_false_positive_tracker_v23_mark() {
        let mut tracker = FalsePositiveTrackerV23::new();
        let record = tracker.mark_false_positive(
            "v1".into(),
            "r1".into(),
            "Not a real vulnerability".into(),
            Some("user-1".into()),
        );
        assert!(record.active);
        assert!(tracker.is_false_positive("v1"));
        assert_eq!(tracker.active_count(), 1);
    }

    #[test]
    fn test_false_positive_tracker_v23_revoke() {
        let mut tracker = FalsePositiveTrackerV23::new();
        let record = tracker.mark_false_positive(
            "v1".into(),
            "r1".into(),
            "Reason".into(),
            None,
        );
        tracker.revoke(&record.id).unwrap();
        assert!(!tracker.is_false_positive("v1"));
        assert_eq!(tracker.active_count(), 0);
    }

    #[test]
    fn test_false_positive_tracker_v23_nonexistent_revoke() {
        let mut tracker = FalsePositiveTrackerV23::new();
        assert!(tracker.revoke("nonexistent").is_err());
    }

    #[test]
    fn test_scan_schedule_v23_new() {
        let schedule = ScanScheduleV23::new("r1".into(), "rs1".into(), 60);
        assert_eq!(schedule.repo_id, "r1");
        assert!(schedule.enabled);
        assert!(schedule.is_due());
    }

    #[test]
    fn test_scan_schedule_v23_record_run() {
        let mut schedule = ScanScheduleV23::new("r1".into(), "rs1".into(), 60);
        schedule.record_run();
        assert!(schedule.last_run.is_some());
        assert!(!schedule.is_due());
    }

    #[test]
    fn test_scan_scheduling_engine_v23() {
        let mut engine = ScanSchedulingEngineV23::new();
        let schedule = ScanScheduleV23::new("r1".into(), "rs1".into(), 60);
        engine.add_schedule(schedule);
        assert_eq!(engine.get_due_schedules().len(), 1);
    }

    #[test]
    fn test_scan_scheduling_engine_v23_disable() {
        let mut engine = ScanSchedulingEngineV23::new();
        let schedule = ScanScheduleV23::new("r1".into(), "rs1".into(), 60);
        engine.add_schedule(schedule);
        let id = engine.schedules[0].id.clone();
        engine.disable_schedule(&id).unwrap();
        assert_eq!(engine.get_due_schedules().len(), 0);
    }

    #[test]
    fn test_rule_set_manager_v23() {
        let mut manager = RuleSetManagerV23::new();
        let rs = SecurityScanRuleSetV23::new("Test".into(), "Desc".into());
        manager.add_rule_set(rs);
        assert_eq!(manager.list_rule_sets().len(), 1);
        assert_eq!(manager.get_enabled_rule_sets().len(), 1);
    }

    #[test]
    fn test_rule_set_manager_v23_report() {
        let mut manager = RuleSetManagerV23::new();
        manager.add_rule_set(SecurityScanRuleSetV23::new("Test".into(), "Desc".into()));
        let report = manager.generate_report();
        assert!(report.contains("Rule Sets V23 Report"));
    }
}
