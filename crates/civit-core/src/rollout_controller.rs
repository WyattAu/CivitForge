#![forbid(unsafe_code)]

//! Health-gated feature rollout.
//!
//! A flag sitting mid-rollout is an experiment running in production, and the
//! research on progressive delivery is consistent about how that experiment
//! should be run: promote on green, roll back on red, and never decide from
//! insufficient evidence. Google's SRE canary guidance is explicit that
//! canary traffic is a small fraction of total traffic and therefore *noisier*
//! than the main system's, which is why this controller tolerates isolated
//! bad windows instead of acting on the first one.
//!
//! Two properties matter more than the automation itself:
//!
//! 1. **Every decision is auditable.** Each observation writes a
//!    `flag_rollout_events` row with its evidence, so "why did this jump to
//!    60%" is answerable months later.
//! 2. **State is durable.** The stage index and failure streak live in the
//!    database, so a restart cannot hand a failing rollout a clean slate and
//!    promote it on the next tick.
//!
//! The controller never invents evidence: with no traffic in the window the
//! gate holds, because "no data" is not "healthy".

use std::sync::Arc;
use std::time::Duration;

use uuid::Uuid;

use civit_db::{DbError, DbRepository, Result};
use civit_telemetry::HealthWindowSnapshot;
use flag_kit::gate::{Decision, HealthGate, HealthSnapshot, Stage, Verdict};

/// Tuning for the controller, all of it conservative by default.
#[derive(Debug, Clone)]
pub struct RolloutControllerConfig {
    /// How often to observe each in-flight rollout.
    pub tick: Duration,
    /// Stage progression. Percentage values are validated by `Stage::new`.
    pub stages: Vec<u8>,
    /// Consecutive bad windows tolerated before rollback.
    pub failure_limit: u32,
    /// Error-rate ceiling per stage.
    pub max_error_rate: f64,
    /// p99 latency ceiling in milliseconds.
    pub max_latency_p99_ms: f64,
    /// Minimum time a stage must be observed before a verdict.
    pub min_stage_duration: Duration,
    /// Minimum requests before a verdict.
    pub min_samples: u64,
}

impl Default for RolloutControllerConfig {
    /// The canary-agent defaults: 5% → 25% → 50% → 100%, a ten-minute
    /// minimum window, and two bad windows before rollback. Two rather than
    /// one because a single window on low traffic is mostly noise.
    fn default() -> Self {
        Self {
            tick: Duration::from_secs(60),
            stages: vec![5, 25, 50, 100],
            failure_limit: 2,
            max_error_rate: 0.01,
            max_latency_p99_ms: 1000.0,
            min_stage_duration: Duration::from_secs(600),
            min_samples: 100,
        }
    }
}

impl RolloutControllerConfig {
    /// Builds the gate this configuration describes.
    ///
    /// # Errors
    /// Returns `Err` when a stage percentage is out of range or the error
    /// rate is outside `0.0..=1.0`.
    pub fn build_gate(&self) -> std::result::Result<HealthGate, flag_kit::gate::StageError> {
        // A loop rather than a closure: the builders return `Result` at
        // two points, and a closure returning `Stage` cannot propagate them.
        // Also avoids the `Result` alias from civit_db shadowing
        // `std::result::Result` in an inferred type position.
        let mut stages = Vec::with_capacity(self.stages.len());
        for pct in &self.stages {
            let stage = Stage::new(*pct)?
                .with_min_duration(self.min_stage_duration)
                .with_min_samples(self.min_samples)
                .with_max_error_rate(self.max_error_rate)?
                .with_max_latency_p99_ms(self.max_latency_p99_ms);
            stages.push(stage);
        }
        HealthGate::new(stages, self.failure_limit)
    }
}

/// What one observation did.
#[derive(Debug, Clone, PartialEq)]
pub struct Observation {
    pub flag_id: Uuid,
    pub flag_name: String,
    /// Kit decision.
    pub decision: Decision,
    /// Kit reason code.
    pub reason: String,
    /// Exposure before the observation.
    pub percentage_before: i32,
    /// Exposure after the observation.
    pub percentage_after: i32,
    pub error_rate: Option<f64>,
    pub latency_p99_ms: Option<f64>,
    pub total_samples: i64,
}

/// Drives health-gated rollouts from a rolling health window.
#[derive(Debug, Clone)]
pub struct RolloutController {
    db: Arc<DbRepository>,
    config: RolloutControllerConfig,
}

impl RolloutController {
    /// Creates a controller.
    #[must_use]
    pub fn new(db: Arc<DbRepository>, config: RolloutControllerConfig) -> Self {
        Self { db, config }
    }

    /// Observes every in-flight rollout once.
    ///
    /// # Errors
    /// Returns `Err` only when the flag list cannot be read. A per-flag
    /// failure is logged and skipped: one broken flag must not stop the
    /// others from being gated.
    pub async fn tick_once(&self, window: &civit_telemetry::HealthWindow) -> Result<Vec<Observation>> {
        let flags = self.db.list_in_flight_rollout_flags().await?;
        let snapshot = window.snapshot();
        let mut out = Vec::with_capacity(flags.len());
        for flag in flags {
            match self.observe_flag(&flag, &snapshot).await {
                Ok(obs) => out.push(obs),
                Err(e) => {
                    tracing::warn!(
                        flag = %flag.name,
                        error = %e,
                        "rollout observation failed; skipping this flag"
                    );
                }
            }
        }
        Ok(out)
    }

    /// Observes one flag, rebuilding its durable gate state.
    ///
    /// # Errors
    /// Propagates database failures so the caller can log and continue.
    pub async fn observe_flag(
        &self,
        flag: &civit_db::models::FeatureFlag,
        snapshot: &HealthWindowSnapshot,
    ) -> Result<Observation> {
        let gate = self
            .config
            .build_gate()
            .map_err(|e| DbError::Database(e.to_string()))?;

        let state = self.db.get_flag_rollout(flag.id).await?;
        let (stage_index, consecutive_failures, stage_started_at) = match &state {
            Some(s) => (
                s.stage_index.max(0) as usize,
                s.consecutive_failures.max(0) as u32,
                s.stage_started_at,
            ),
            None => {
                let started = self.db.start_flag_rollout(flag.id).await?;
                (started.stage_index.max(0) as usize, 0, started.stage_started_at)
            }
        };

        // Restore the gate's position and streak. Without this a restart
        // silently resets both, and a rollout two bad windows from a rollback
        // would promote on its first healthy window.
        let mut restored = gate;
        restored.restore_stage(stage_index);
        restored.restore_streak(consecutive_failures);

        let percentage_before = flag.enabled_for_percentage;
        let observed_for = chrono::Utc::now()
            .signed_duration_since(stage_started_at)
            .to_std()
            .unwrap_or(Duration::ZERO);

        let verdict: Verdict = restored.observe(
            HealthSnapshot {
                total: snapshot.total,
                errors: snapshot.errors,
                latencies_ms: snapshot.latencies.clone(),
            },
            observed_for,
        );

        let (percentage_after, final_stage_index, final_streak) = match verdict.decision {
            Decision::Promote => {
                // Advance the gate first, then mirror its new stage. Reading
                // the stage before advancing applies the PREVIOUS stage's
                // exposure and leaves every rollout one tick behind — the
                // same defect the kit's RolloutController tests caught, back
                // again in this reimplementation.
                if restored.advance().is_err() {
                    // Gate complete: the rollout is finished, not promoted.
                    self.db
                        .set_feature_flag_percentage(flag.id, 100)
                        .await?;
                    self.persist(
                        flag,
                        "completed",
                        &verdict,
                        percentage_before,
                        100,
                        restored.stage_index(),
                        restored.consecutive_failures(),
                        snapshot,
                    )
                    .await?;
                    return Ok(Observation {
                        flag_id: flag.id,
                        flag_name: flag.name.clone(),
                        decision: Decision::Promote,
                        reason: verdict.reason.as_str().to_string(),
                        percentage_before,
                        percentage_after: 100,
                        error_rate: verdict.error_rate,
                        latency_p99_ms: verdict.latency_p99_ms,
                        total_samples: snapshot.total as i64,
                    });
                }
                let target = restored.stage().percentage;
                self.db
                    .set_feature_flag_percentage(flag.id, i32::from(target))
                    .await?;
                (
                    i32::from(target),
                    restored.stage_index(),
                    restored.consecutive_failures(),
                )
            }
            Decision::Rollback => {
                // Zero exposure and restart the stage clock.
                self.db.set_feature_flag_percentage(flag.id, 0).await?;
                restored.restart();
                (0, restored.stage_index(), restored.consecutive_failures())
            }
            Decision::Hold => (percentage_before, restored.stage_index(), restored.consecutive_failures()),
        };

        self.persist(
            flag,
            verdict.decision.as_str(),
            &verdict,
            percentage_before,
            percentage_after,
            final_stage_index,
            final_streak,
            snapshot,
        )
        .await?;

        Ok(Observation {
            flag_id: flag.id,
            flag_name: flag.name.clone(),
            decision: verdict.decision,
            reason: verdict.reason.as_str().to_string(),
            percentage_before,
            percentage_after,
            error_rate: verdict.error_rate,
            latency_p99_ms: verdict.latency_p99_ms,
            total_samples: snapshot.total as i64,
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn persist(
        &self,
        flag: &civit_db::models::FeatureFlag,
        decision: &str,
        verdict: &Verdict,
        percentage_before: i32,
        percentage_after: i32,
        stage_index: usize,
        streak: u32,
        snapshot: &HealthWindowSnapshot,
    ) -> Result<()> {
        self.db
            .record_flag_rollout_observation(
                flag.id,
                decision,
                verdict.reason.as_str(),
                stage_index as i32,
                i32::try_from(streak).unwrap_or(i32::MAX),
                verdict.error_rate,
                verdict.latency_p99_ms,
                i64::try_from(snapshot.total).unwrap_or(i64::MAX),
                percentage_before,
                percentage_after,
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn default_config_builds_the_conventional_gate() {
        let gate = RolloutControllerConfig::default().build_gate().unwrap();
        assert_eq!(gate.stage().percentage, 5);
        assert_eq!(gate.failure_limit, 2, "one bad window is mostly noise");
    }

    #[test]
    fn stages_are_validated() {
        let cfg = RolloutControllerConfig {
            stages: vec![5, 200],
            ..RolloutControllerConfig::default()
        };
        assert!(cfg.build_gate().is_err());

        let cfg = RolloutControllerConfig {
            max_error_rate: 2.0,
            ..RolloutControllerConfig::default()
        };
        assert!(cfg.build_gate().is_err());
    }

    #[test]
    fn min_duration_and_samples_reach_the_gate() {
        let cfg = RolloutControllerConfig {
            min_stage_duration: Duration::from_secs(42),
            min_samples: 7,
            ..RolloutControllerConfig::default()
        };
        let gate = cfg.build_gate().unwrap();
        assert_eq!(gate.stage().min_duration, Duration::from_secs(42));
        assert_eq!(gate.stage().min_samples, 7);
    }

    /// A zero window is the case that must never look healthy.
    #[test]
    fn zero_error_rate_is_none_without_traffic() {
        assert!(HealthWindowSnapshot::default().error_rate().is_none());
    }
}