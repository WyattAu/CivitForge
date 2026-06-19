use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents the assignment of a repository (or entity) to a specific shard.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShardAssignment {
    /// The user/owner ID associated with the assignment.
    pub user_id: String,
    /// The repository ID being assigned.
    pub repo_id: String,
    /// The shard ID this repository is assigned to.
    pub shard_id: String,
    /// When the assignment was created.
    pub assigned_at: DateTime<Utc>,
    /// When the data was fully migrated to the target shard (if applicable).
    pub migrated_at: Option<DateTime<Utc>>,
    /// Current migration status of this assignment.
    pub status: AssignmentStatus,
}

/// Migration status of a shard assignment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssignmentStatus {
    /// Not yet migrated to the target shard.
    Pending,
    /// Data copy is in progress.
    Migrating,
    /// Reads and writes are routed to the assigned shard.
    Active,
    /// Old data has been cleaned up.
    Archived,
}

impl std::fmt::Display for AssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssignmentStatus::Pending => write!(f, "pending"),
            AssignmentStatus::Migrating => write!(f, "migrating"),
            AssignmentStatus::Active => write!(f, "active"),
            AssignmentStatus::Archived => write!(f, "archived"),
        }
    }
}

impl ShardAssignment {
    pub fn new(
        user_id: impl Into<String>,
        repo_id: impl Into<String>,
        shard_id: impl Into<String>,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            repo_id: repo_id.into(),
            shard_id: shard_id.into(),
            assigned_at: Utc::now(),
            migrated_at: None,
            status: AssignmentStatus::Pending,
        }
    }

    /// Transition this assignment to the `Migrating` status.
    pub fn start_migration(&mut self) {
        self.status = AssignmentStatus::Migrating;
    }

    /// Transition this assignment to the `Active` status.
    pub fn activate(&mut self) {
        self.status = AssignmentStatus::Active;
        self.migrated_at = Some(Utc::now());
    }

    /// Transition this assignment to the `Archived` status.
    pub fn archive(&mut self) {
        self.status = AssignmentStatus::Archived;
    }
}

/// Metadata about a shard's current state and capacity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShardMetadata {
    /// Unique identifier for the shard.
    pub shard_id: String,
    /// Geographic or logical region.
    pub region: String,
    /// Current operational status.
    pub status: ShardStatus,
    /// Maximum capacity (e.g., max repositories, max storage in bytes).
    pub capacity: u64,
    /// Current utilization (e.g., current repositories, current storage in bytes).
    pub current_load: u64,
    /// When this shard was created.
    pub created_at: DateTime<Utc>,
    /// Last time this shard's health was checked.
    pub last_health_check: Option<DateTime<Utc>>,
}

/// Operational status of a shard.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShardStatus {
    /// Shard is active and accepting traffic.
    Active,
    /// Shard is being drained (no new assignments, existing data migrating out).
    Draining,
    /// Shard is offline and not serving traffic.
    Offline,
}

impl std::fmt::Display for ShardStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShardStatus::Active => write!(f, "active"),
            ShardStatus::Draining => write!(f, "draining"),
            ShardStatus::Offline => write!(f, "offline"),
        }
    }
}

impl ShardMetadata {
    pub fn new(shard_id: impl Into<String>, region: impl Into<String>, capacity: u64) -> Self {
        Self {
            shard_id: shard_id.into(),
            region: region.into(),
            status: ShardStatus::Active,
            capacity,
            current_load: 0,
            created_at: Utc::now(),
            last_health_check: None,
        }
    }

    /// Returns the load as a fraction of capacity (0.0 to 1.0).
    pub fn load_ratio(&self) -> f64 {
        if self.capacity == 0 {
            return 0.0;
        }
        self.current_load as f64 / self.capacity as f64
    }

    /// Returns true if the shard is at or above 80% capacity.
    pub fn is_near_capacity(&self) -> bool {
        self.load_ratio() >= 0.8
    }

    /// Record a health check.
    pub fn record_health_check(&mut self) {
        self.last_health_check = Some(Utc::now());
    }

    /// Mark the shard as draining.
    pub fn start_draining(&mut self) {
        self.status = ShardStatus::Draining;
    }

    /// Mark the shard as offline.
    pub fn go_offline(&mut self) {
        self.status = ShardStatus::Offline;
    }

    /// Mark the shard as active.
    pub fn activate(&mut self) {
        self.status = ShardStatus::Active;
    }
}

/// A collection of shard assignments for tracking purposes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssignmentTracker {
    assignments: Vec<ShardAssignment>,
}

impl AssignmentTracker {
    pub fn new() -> Self {
        Self {
            assignments: Vec::new(),
        }
    }

    /// Add a new assignment.
    pub fn add(&mut self, assignment: ShardAssignment) {
        self.assignments.push(assignment);
    }

    /// Find the assignment for a given repo.
    pub fn find_by_repo(&self, repo_id: &str) -> Option<&ShardAssignment> {
        self.assignments.iter().find(|a| a.repo_id == repo_id)
    }

    /// Find all assignments for a given shard.
    pub fn find_by_shard(&self, shard_id: &str) -> Vec<&ShardAssignment> {
        self.assignments
            .iter()
            .filter(|a| a.shard_id == shard_id)
            .collect()
    }

    /// Find all assignments for a given user.
    pub fn find_by_user(&self, user_id: &str) -> Vec<&ShardAssignment> {
        self.assignments
            .iter()
            .filter(|a| a.user_id == user_id)
            .collect()
    }

    /// Count assignments per shard.
    pub fn count_per_shard(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        for a in &self.assignments {
            *counts.entry(a.shard_id.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Get total number of assignments.
    pub fn total(&self) -> usize {
        self.assignments.len()
    }

    /// Get all assignments.
    pub fn all(&self) -> &[ShardAssignment] {
        &self.assignments
    }

    /// Remove an assignment by repo_id.
    pub fn remove_by_repo(&mut self, repo_id: &str) -> Option<ShardAssignment> {
        if let Some(pos) = self.assignments.iter().position(|a| a.repo_id == repo_id) {
            Some(self.assignments.remove(pos))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shard_assignment_new() {
        let a = ShardAssignment::new("user-1", "repo-1", "shard-0");
        assert_eq!(a.user_id, "user-1");
        assert_eq!(a.repo_id, "repo-1");
        assert_eq!(a.shard_id, "shard-0");
        assert_eq!(a.status, AssignmentStatus::Pending);
        assert!(a.migrated_at.is_none());
    }

    #[test]
    fn test_assignment_status_transitions() {
        let mut a = ShardAssignment::new("u", "r", "s");
        assert_eq!(a.status, AssignmentStatus::Pending);

        a.start_migration();
        assert_eq!(a.status, AssignmentStatus::Migrating);

        a.activate();
        assert_eq!(a.status, AssignmentStatus::Active);
        assert!(a.migrated_at.is_some());

        a.archive();
        assert_eq!(a.status, AssignmentStatus::Archived);
    }

    #[test]
    fn test_assignment_status_display() {
        assert_eq!(AssignmentStatus::Pending.to_string(), "pending");
        assert_eq!(AssignmentStatus::Migrating.to_string(), "migrating");
        assert_eq!(AssignmentStatus::Active.to_string(), "active");
        assert_eq!(AssignmentStatus::Archived.to_string(), "archived");
    }

    #[test]
    fn test_shard_metadata_new() {
        let m = ShardMetadata::new("shard-0", "us-east-1", 1_000_000);
        assert_eq!(m.shard_id, "shard-0");
        assert_eq!(m.region, "us-east-1");
        assert_eq!(m.status, ShardStatus::Active);
        assert_eq!(m.capacity, 1_000_000);
        assert_eq!(m.current_load, 0);
        assert!(!m.is_near_capacity());
    }

    #[test]
    fn test_shard_metadata_load_ratio() {
        let mut m = ShardMetadata::new("s", "r", 100);
        assert_eq!(m.load_ratio(), 0.0);

        m.current_load = 50;
        assert_eq!(m.load_ratio(), 0.5);

        m.current_load = 100;
        assert_eq!(m.load_ratio(), 1.0);
    }

    #[test]
    fn test_shard_metadata_near_capacity() {
        let mut m = ShardMetadata::new("s", "r", 100);
        assert!(!m.is_near_capacity());

        m.current_load = 79;
        assert!(!m.is_near_capacity());

        m.current_load = 80;
        assert!(m.is_near_capacity());

        m.current_load = 100;
        assert!(m.is_near_capacity());
    }

    #[test]
    fn test_shard_metadata_status_transitions() {
        let mut m = ShardMetadata::new("s", "r", 100);
        assert_eq!(m.status, ShardStatus::Active);

        m.start_draining();
        assert_eq!(m.status, ShardStatus::Draining);

        m.go_offline();
        assert_eq!(m.status, ShardStatus::Offline);

        m.activate();
        assert_eq!(m.status, ShardStatus::Active);
    }

    #[test]
    fn test_shard_metadata_health_check() {
        let mut m = ShardMetadata::new("s", "r", 100);
        assert!(m.last_health_check.is_none());

        m.record_health_check();
        assert!(m.last_health_check.is_some());
    }

    #[test]
    fn test_shard_metadata_status_display() {
        assert_eq!(ShardStatus::Active.to_string(), "active");
        assert_eq!(ShardStatus::Draining.to_string(), "draining");
        assert_eq!(ShardStatus::Offline.to_string(), "offline");
    }

    #[test]
    fn test_assignment_tracker_add_and_find() {
        let mut tracker = AssignmentTracker::new();
        tracker.add(ShardAssignment::new("u1", "repo-1", "shard-0"));
        tracker.add(ShardAssignment::new("u1", "repo-2", "shard-1"));
        tracker.add(ShardAssignment::new("u2", "repo-3", "shard-0"));

        assert_eq!(tracker.total(), 3);

        let found = tracker.find_by_repo("repo-2").unwrap();
        assert_eq!(found.shard_id, "shard-1");

        assert!(tracker.find_by_repo("repo-99").is_none());
    }

    #[test]
    fn test_tracker_find_by_shard() {
        let mut tracker = AssignmentTracker::new();
        tracker.add(ShardAssignment::new("u", "r1", "shard-0"));
        tracker.add(ShardAssignment::new("u", "r2", "shard-0"));
        tracker.add(ShardAssignment::new("u", "r3", "shard-1"));

        let on_shard_0 = tracker.find_by_shard("shard-0");
        assert_eq!(on_shard_0.len(), 2);

        let on_shard_1 = tracker.find_by_shard("shard-1");
        assert_eq!(on_shard_1.len(), 1);
    }

    #[test]
    fn test_tracker_find_by_user() {
        let mut tracker = AssignmentTracker::new();
        tracker.add(ShardAssignment::new("u1", "r1", "s0"));
        tracker.add(ShardAssignment::new("u1", "r2", "s1"));
        tracker.add(ShardAssignment::new("u2", "r3", "s0"));

        let u1_repos = tracker.find_by_user("u1");
        assert_eq!(u1_repos.len(), 2);
    }

    #[test]
    fn test_tracker_count_per_shard() {
        let mut tracker = AssignmentTracker::new();
        tracker.add(ShardAssignment::new("u", "r1", "shard-0"));
        tracker.add(ShardAssignment::new("u", "r2", "shard-0"));
        tracker.add(ShardAssignment::new("u", "r3", "shard-1"));

        let counts = tracker.count_per_shard();
        assert_eq!(counts.get("shard-0"), Some(&2));
        assert_eq!(counts.get("shard-1"), Some(&1));
    }

    #[test]
    fn test_tracker_remove_by_repo() {
        let mut tracker = AssignmentTracker::new();
        tracker.add(ShardAssignment::new("u", "r1", "s0"));
        tracker.add(ShardAssignment::new("u", "r2", "s1"));

        let removed = tracker.remove_by_repo("r1");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().repo_id, "r1");
        assert_eq!(tracker.total(), 1);
        assert!(tracker.find_by_repo("r1").is_none());
    }

    #[test]
    fn test_tracker_remove_nonexistent() {
        let mut tracker = AssignmentTracker::new();
        assert!(tracker.remove_by_repo("nope").is_none());
    }

    #[test]
    fn test_assignment_serialization() {
        let a = ShardAssignment::new("u1", "r1", "s0");
        let json = serde_json::to_string(&a).unwrap();
        let deserialized: ShardAssignment = serde_json::from_str(&json).unwrap();
        assert_eq!(a, deserialized);
    }

    #[test]
    fn test_metadata_serialization() {
        let m = ShardMetadata::new("shard-0", "us-east-1", 1000);
        let json = serde_json::to_string(&m).unwrap();
        let deserialized: ShardMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(m, deserialized);
    }

    #[test]
    fn test_assignment_status_serialization() {
        for status in [
            AssignmentStatus::Pending,
            AssignmentStatus::Migrating,
            AssignmentStatus::Active,
            AssignmentStatus::Archived,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: AssignmentStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, deserialized);
        }
    }

    #[test]
    fn test_shard_status_serialization() {
        for status in [
            ShardStatus::Active,
            ShardStatus::Draining,
            ShardStatus::Offline,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: ShardStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, deserialized);
        }
    }
}
