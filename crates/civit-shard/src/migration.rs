use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// The phases of a shard migration, executed in order.
///
/// Each phase represents a distinct operational state with specific read/write
/// behavior and rollback capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MigrationPhase {
    /// Writing to both the original primary and new shards simultaneously.
    /// Reads still go to the original primary.
    DualWrite,
    /// Reads are routed to shards for migrated repos; writes continue dual-writing.
    ReadFromShards,
    /// All traffic routes through the shard router. No more dual-write.
    Cutover,
    /// Legacy infrastructure removed. Shard-only operation.
    Decommission,
}

impl MigrationPhase {
    /// All phases in order.
    pub const ALL: &'static [MigrationPhase] = &[
        MigrationPhase::DualWrite,
        MigrationPhase::ReadFromShards,
        MigrationPhase::Cutover,
        MigrationPhase::Decommission,
    ];

    /// The next phase in the migration sequence, if any.
    pub fn next(&self) -> Option<MigrationPhase> {
        match self {
            MigrationPhase::DualWrite => Some(MigrationPhase::ReadFromShards),
            MigrationPhase::ReadFromShards => Some(MigrationPhase::Cutover),
            MigrationPhase::Cutover => Some(MigrationPhase::Decommission),
            MigrationPhase::Decommission => None,
        }
    }

    /// The previous phase (for rollback), if any.
    pub fn previous(&self) -> Option<MigrationPhase> {
        match self {
            MigrationPhase::DualWrite => None,
            MigrationPhase::ReadFromShards => Some(MigrationPhase::DualWrite),
            MigrationPhase::Cutover => Some(MigrationPhase::ReadFromShards),
            MigrationPhase::Decommission => Some(MigrationPhase::Cutover),
        }
    }

    /// Returns the zero-indexed position of this phase.
    pub fn index(&self) -> usize {
        match self {
            MigrationPhase::DualWrite => 0,
            MigrationPhase::ReadFromShards => 1,
            MigrationPhase::Cutover => 2,
            MigrationPhase::Decommission => 3,
        }
    }
}

impl fmt::Display for MigrationPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MigrationPhase::DualWrite => write!(f, "dual_write"),
            MigrationPhase::ReadFromShards => write!(f, "read_from_shards"),
            MigrationPhase::Cutover => write!(f, "cutover"),
            MigrationPhase::Decommission => write!(f, "decommission"),
        }
    }
}

impl std::str::FromStr for MigrationPhase {
    type Err = MigrationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dual_write" => Ok(MigrationPhase::DualWrite),
            "read_from_shards" => Ok(MigrationPhase::ReadFromShards),
            "cutover" => Ok(MigrationPhase::Cutover),
            "decommission" => Ok(MigrationPhase::Decommission),
            _ => Err(MigrationError::InvalidPhase(s.to_string())),
        }
    }
}

#[derive(Error, Debug)]
pub enum MigrationError {
    #[error("invalid migration phase: {0}")]
    InvalidPhase(String),

    #[error("cannot advance from phase {current} to {target}: {reason}")]
    InvalidTransition {
        current: MigrationPhase,
        target: MigrationPhase,
        reason: String,
    },

    #[error("migration is already complete")]
    AlreadyComplete,

    #[error("migration has not started")]
    NotStarted,
}

/// Tracks the progress and state of a shard migration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationState {
    /// The current phase of the migration.
    pub current_phase: MigrationPhase,
    /// When the current phase started.
    pub phase_started_at: DateTime<Utc>,
    /// Overall progress (0.0 to 1.0).
    pub progress: f64,
    /// Number of repositories migrated in the current phase.
    pub migrated_count: u64,
    /// Total number of repositories to migrate.
    pub total_count: u64,
    /// Error log for failed migrations.
    pub errors: Vec<MigrationErrorEntry>,
    /// When the migration started.
    pub started_at: DateTime<Utc>,
    /// When the migration completed (if it has).
    pub completed_at: Option<DateTime<Utc>>,
    /// Whether the migration has been rolled back.
    pub rolled_back: bool,
}

/// A single error that occurred during migration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrationErrorEntry {
    /// The repo that failed to migrate.
    pub repo_id: String,
    /// Description of the error.
    pub message: String,
    /// When the error occurred.
    pub occurred_at: DateTime<Utc>,
    /// Whether this error was retried successfully.
    pub retried: bool,
}

impl MigrationState {
    /// Create a new migration state starting at the first phase.
    pub fn new(total_count: u64) -> Self {
        Self {
            current_phase: MigrationPhase::DualWrite,
            phase_started_at: Utc::now(),
            progress: 0.0,
            migrated_count: 0,
            total_count,
            errors: Vec::new(),
            started_at: Utc::now(),
            completed_at: None,
            rolled_back: false,
        }
    }

    /// Advance to the next phase in the migration sequence.
    pub fn advance(&mut self) -> Result<(), MigrationError> {
        let next = self
            .current_phase
            .next()
            .ok_or(MigrationError::AlreadyComplete)?;

        self.current_phase = next;
        self.phase_started_at = Utc::now();
        Ok(())
    }

    /// Transition to a specific phase. Only allows moving forward or to the
    /// immediately previous phase (rollback).
    pub fn transition_to(&mut self, target: MigrationPhase) -> Result<(), MigrationError> {
        // Allow advancing to next phase
        if self.current_phase.next() == Some(target) {
            self.current_phase = target;
            self.phase_started_at = Utc::now();
            return Ok(());
        }

        // Allow rollback to previous phase
        if self.current_phase.previous() == Some(target) {
            self.current_phase = target;
            self.phase_started_at = Utc::now();
            self.rolled_back = true;
            return Ok(());
        }

        Err(MigrationError::InvalidTransition {
            current: self.current_phase,
            target,
            reason: format!(
                "can only advance to {:?} or rollback to {:?}",
                self.current_phase.next(),
                self.current_phase.previous()
            ),
        })
    }

    /// Roll back to the previous phase.
    pub fn rollback(&mut self) -> Result<(), MigrationError> {
        let prev = self
            .current_phase
            .previous()
            .ok_or(MigrationError::NotStarted)?;

        self.current_phase = prev;
        self.phase_started_at = Utc::now();
        self.rolled_back = true;
        Ok(())
    }

    /// Record a successful migration of a repository.
    pub fn record_migration(&mut self) {
        self.migrated_count += 1;
        self.update_progress();
    }

    /// Record a migration error.
    pub fn record_error(&mut self, repo_id: impl Into<String>, message: impl Into<String>) {
        self.errors.push(MigrationErrorEntry {
            repo_id: repo_id.into(),
            message: message.into(),
            occurred_at: Utc::now(),
            retried: false,
        });
    }

    /// Mark an error as retried.
    pub fn mark_error_retried(&mut self, index: usize) {
        if let Some(entry) = self.errors.get_mut(index) {
            entry.retried = true;
        }
    }

    /// Mark the migration as complete.
    pub fn complete(&mut self) {
        self.progress = 1.0;
        self.migrated_count = self.total_count;
        self.completed_at = Some(Utc::now());
    }

    /// Returns true if the migration is complete.
    pub fn is_complete(&self) -> bool {
        self.completed_at.is_some()
    }

    /// Returns true if there are unhandled errors.
    pub fn has_errors(&self) -> bool {
        self.errors.iter().any(|e| !e.retried)
    }

    /// Returns the number of unhandled errors.
    pub fn unhandled_error_count(&self) -> usize {
        self.errors.iter().filter(|e| !e.retried).count()
    }

    fn update_progress(&mut self) {
        if self.total_count > 0 {
            self.progress = self.migrated_count as f64 / self.total_count as f64;
        }
    }
}

impl Default for MigrationState {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_phase_ordering() {
        assert!(MigrationPhase::DualWrite < MigrationPhase::ReadFromShards);
        assert!(MigrationPhase::ReadFromShards < MigrationPhase::Cutover);
        assert!(MigrationPhase::Cutover < MigrationPhase::Decommission);
    }

    #[test]
    fn test_migration_phase_next() {
        assert_eq!(
            MigrationPhase::DualWrite.next(),
            Some(MigrationPhase::ReadFromShards)
        );
        assert_eq!(
            MigrationPhase::ReadFromShards.next(),
            Some(MigrationPhase::Cutover)
        );
        assert_eq!(
            MigrationPhase::Cutover.next(),
            Some(MigrationPhase::Decommission)
        );
        assert_eq!(MigrationPhase::Decommission.next(), None);
    }

    #[test]
    fn test_migration_phase_previous() {
        assert_eq!(MigrationPhase::DualWrite.previous(), None);
        assert_eq!(
            MigrationPhase::ReadFromShards.previous(),
            Some(MigrationPhase::DualWrite)
        );
        assert_eq!(
            MigrationPhase::Cutover.previous(),
            Some(MigrationPhase::ReadFromShards)
        );
        assert_eq!(
            MigrationPhase::Decommission.previous(),
            Some(MigrationPhase::Cutover)
        );
    }

    #[test]
    fn test_migration_phase_index() {
        assert_eq!(MigrationPhase::DualWrite.index(), 0);
        assert_eq!(MigrationPhase::ReadFromShards.index(), 1);
        assert_eq!(MigrationPhase::Cutover.index(), 2);
        assert_eq!(MigrationPhase::Decommission.index(), 3);
    }

    #[test]
    fn test_migration_phase_display() {
        assert_eq!(MigrationPhase::DualWrite.to_string(), "dual_write");
        assert_eq!(
            MigrationPhase::ReadFromShards.to_string(),
            "read_from_shards"
        );
        assert_eq!(MigrationPhase::Cutover.to_string(), "cutover");
        assert_eq!(MigrationPhase::Decommission.to_string(), "decommission");
    }

    #[test]
    fn test_migration_phase_from_str() {
        assert_eq!(
            "dual_write".parse::<MigrationPhase>().unwrap(),
            MigrationPhase::DualWrite
        );
        assert_eq!(
            "read_from_shards".parse::<MigrationPhase>().unwrap(),
            MigrationPhase::ReadFromShards
        );
        assert_eq!(
            "cutover".parse::<MigrationPhase>().unwrap(),
            MigrationPhase::Cutover
        );
        assert_eq!(
            "decommission".parse::<MigrationPhase>().unwrap(),
            MigrationPhase::Decommission
        );
        assert!("invalid".parse::<MigrationPhase>().is_err());
    }

    #[test]
    fn test_migration_state_new() {
        let state = MigrationState::new(100);
        assert_eq!(state.current_phase, MigrationPhase::DualWrite);
        assert_eq!(state.total_count, 100);
        assert_eq!(state.migrated_count, 0);
        assert_eq!(state.progress, 0.0);
        assert!(!state.is_complete());
        assert!(!state.has_errors());
    }

    #[test]
    fn test_migration_advance() {
        let mut state = MigrationState::new(10);
        state.advance().unwrap();
        assert_eq!(state.current_phase, MigrationPhase::ReadFromShards);

        state.advance().unwrap();
        assert_eq!(state.current_phase, MigrationPhase::Cutover);

        state.advance().unwrap();
        assert_eq!(state.current_phase, MigrationPhase::Decommission);

        assert!(state.advance().is_err());
    }

    #[test]
    fn test_migration_rollback() {
        let mut state = MigrationState::new(10);
        state.advance().unwrap();
        state.advance().unwrap();
        assert_eq!(state.current_phase, MigrationPhase::Cutover);

        state.rollback().unwrap();
        assert_eq!(state.current_phase, MigrationPhase::ReadFromShards);
        assert!(state.rolled_back);

        state.rollback().unwrap();
        assert_eq!(state.current_phase, MigrationPhase::DualWrite);

        assert!(state.rollback().is_err());
    }

    #[test]
    fn test_migration_transition_to_next() {
        let mut state = MigrationState::new(10);
        state.transition_to(MigrationPhase::ReadFromShards).unwrap();
        assert_eq!(state.current_phase, MigrationPhase::ReadFromShards);
    }

    #[test]
    fn test_migration_transition_to_previous() {
        let mut state = MigrationState::new(10);
        state.advance().unwrap();
        assert_eq!(state.current_phase, MigrationPhase::ReadFromShards);

        state.transition_to(MigrationPhase::DualWrite).unwrap();
        assert_eq!(state.current_phase, MigrationPhase::DualWrite);
    }

    #[test]
    fn test_migration_transition_invalid_skip() {
        let mut state = MigrationState::new(10);
        // Skipping from DualWrite to Cutover is not allowed
        assert!(state.transition_to(MigrationPhase::Cutover).is_err());
    }

    #[test]
    fn test_migration_transition_invalid_reverse_skip() {
        let mut state = MigrationState::new(10);
        state.advance().unwrap();
        state.advance().unwrap();
        // Skipping from Cutover back to DualWrite is not allowed
        assert!(state.transition_to(MigrationPhase::DualWrite).is_err());
    }

    #[test]
    fn test_migration_record_progress() {
        let mut state = MigrationState::new(100);
        for _ in 0..25 {
            state.record_migration();
        }
        assert_eq!(state.migrated_count, 25);
        assert!((state.progress - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_migration_complete() {
        let mut state = MigrationState::new(100);
        state.complete();
        assert!(state.is_complete());
        assert_eq!(state.progress, 1.0);
        assert_eq!(state.migrated_count, 100);
        assert!(state.completed_at.is_some());
    }

    #[test]
    fn test_migration_errors() {
        let mut state = MigrationState::new(100);
        assert!(!state.has_errors());
        assert_eq!(state.unhandled_error_count(), 0);

        state.record_error("repo-1", "connection timeout");
        assert!(state.has_errors());
        assert_eq!(state.unhandled_error_count(), 1);

        state.mark_error_retried(0);
        assert!(!state.has_errors());
        assert_eq!(state.unhandled_error_count(), 0);
    }

    #[test]
    fn test_migration_error_entry_fields() {
        let mut state = MigrationState::new(10);
        state.record_error("repo-42", "disk full");

        let entry = &state.errors[0];
        assert_eq!(entry.repo_id, "repo-42");
        assert_eq!(entry.message, "disk full");
        assert!(!entry.retried);
        assert!(entry.occurred_at <= Utc::now());
    }

    #[test]
    fn test_migration_phase_all() {
        assert_eq!(MigrationPhase::ALL.len(), 4);
        assert_eq!(MigrationPhase::ALL[0], MigrationPhase::DualWrite);
        assert_eq!(MigrationPhase::ALL[3], MigrationPhase::Decommission);
    }

    #[test]
    fn test_migration_state_default() {
        let state = MigrationState::default();
        assert_eq!(state.current_phase, MigrationPhase::DualWrite);
        assert_eq!(state.total_count, 0);
    }

    #[test]
    fn test_migration_state_serialization() {
        let mut state = MigrationState::new(50);
        state.record_migration();
        state.record_error("r1", "err");
        state.advance().unwrap();

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: MigrationState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.current_phase, MigrationPhase::ReadFromShards);
        assert_eq!(deserialized.migrated_count, 1);
        assert_eq!(deserialized.errors.len(), 1);
    }

    #[test]
    fn test_migration_error_entry_serialization() {
        let entry = MigrationErrorEntry {
            repo_id: "r1".into(),
            message: "timeout".into(),
            occurred_at: Utc::now(),
            retried: false,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: MigrationErrorEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, deserialized);
    }

    #[test]
    fn test_full_migration_lifecycle() {
        let mut state = MigrationState::new(4);

        // Phase 1: DualWrite
        assert_eq!(state.current_phase, MigrationPhase::DualWrite);
        state.record_migration();
        state.record_migration();
        state.advance().unwrap();

        // Phase 2: ReadFromShards
        assert_eq!(state.current_phase, MigrationPhase::ReadFromShards);
        state.record_migration();
        state.record_error("repo-3", "consistency check failed");
        state.mark_error_retried(0);
        state.advance().unwrap();

        // Phase 3: Cutover
        assert_eq!(state.current_phase, MigrationPhase::Cutover);
        state.record_migration();
        state.advance().unwrap();

        // Phase 4: Decommission
        assert_eq!(state.current_phase, MigrationPhase::Decommission);
        state.complete();

        assert!(state.is_complete());
        assert!(!state.has_errors());
        assert_eq!(state.progress, 1.0);
    }

    #[test]
    fn test_rollback_during_migration() {
        let mut state = MigrationState::new(10);
        state.record_migration();
        state.record_migration();
        state.advance().unwrap();

        // In ReadFromShards, discover an issue
        state.record_error("repo-5", "data inconsistency");
        state.rollback().unwrap();

        assert_eq!(state.current_phase, MigrationPhase::DualWrite);
        assert!(state.rolled_back);
        // Progress and errors are preserved
        assert_eq!(state.migrated_count, 2);
        assert_eq!(state.errors.len(), 1);
    }

    #[test]
    fn test_mark_error_retried_out_of_bounds() {
        let mut state = MigrationState::new(10);
        state.record_error("r1", "err");
        state.mark_error_retried(99); // Should not panic
        assert!(!state.errors[0].retried);
    }
}
