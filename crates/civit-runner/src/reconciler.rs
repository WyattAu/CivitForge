#![forbid(unsafe_code)]

use crate::crd::{PipelineRunSpec, PipelineRunStatus, RunPhase};
use chrono::Utc;
use dashmap::DashMap;
use std::collections::VecDeque;
use parking_lot::Mutex;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct CompletedRun {
    pub name: String,
    pub status: PipelineRunStatus,
    pub duration: chrono::Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReconcileAction {
    Schedule,
    Wait,
    Complete,
    Fail,
    Cancel,
}

#[derive(Debug, Clone)]
pub struct ReconcileResult {
    pub name: String,
    pub action: ReconcileAction,
    pub reason: String,
}

pub struct Reconciler {
    pub namespace: String,
    pub resync_interval: Duration,
    pub max_parallel: u32,
    pub running: DashMap<String, PipelineRunStatus>,
    pub queue: DashMap<String, PipelineRunSpec>,
    completed: Mutex<VecDeque<CompletedRun>>,
}

impl Reconciler {
    pub fn new(namespace: impl Into<String>, resync_interval: Duration, max_parallel: u32) -> Self {
        Self {
            namespace: namespace.into(),
            resync_interval,
            max_parallel,
            running: DashMap::new(),
            queue: DashMap::new(),
            completed: Mutex::new(VecDeque::new()),
        }
    }

    pub fn submit(&self, spec: PipelineRunSpec) -> anyhow::Result<()> {
        let name = spec.name.clone();
        if self.running.contains_key(&name) || self.queue.contains_key(&name) {
            anyhow::bail!("pipeline run '{name}' already exists");
        }
        let status = PipelineRunStatus::new_pending("submitted");
        self.queue.insert(name.clone(), spec);
        self.running.insert(name, status);
        Ok(())
    }

    pub fn reconcile_all(&self) -> Vec<ReconcileResult> {
        let mut results = Vec::new();
        let mut to_complete = Vec::new();
        let mut current_running = self.running_count() as u32;

        for item in self.queue.iter() {
            let (name, spec) = (item.key().clone(), item.value().clone());
            if let Some(mut status_entry) = self.running.get_mut(&name) {
                let status = status_entry.value_mut();
                match status.phase {
                    RunPhase::Pending => {
                        if current_running < self.max_parallel {
                            status.phase = RunPhase::Running;
                            status.message = "scheduled".into();
                            status.started_at = Some(Utc::now());
                            current_running += 1;
                            results.push(ReconcileResult {
                                name: name.clone(),
                                action: ReconcileAction::Schedule,
                                reason: "slot available".into(),
                            });
                        } else {
                            results.push(ReconcileResult {
                                name,
                                action: ReconcileAction::Wait,
                                reason: "no slots available".into(),
                            });
                        }
                    }
                    RunPhase::Running => {
                        let all_succeeded = spec.steps.iter().all(|s| {
                            status
                                .step_statuses
                                .iter()
                                .any(|ss| ss.name == s.name && ss.phase == RunPhase::Succeeded)
                        });
                        let any_failed = status
                            .step_statuses
                            .iter()
                            .any(|ss| ss.phase == RunPhase::Failed);

                        if any_failed {
                            status.phase = RunPhase::Failed;
                            status.message = "step failed".into();
                            status.finished_at = Some(Utc::now());
                            results.push(ReconcileResult {
                                name: name.clone(),
                                action: ReconcileAction::Fail,
                                reason: "step failure detected".into(),
                            });
                            to_complete.push(name);
                        } else if all_succeeded && !spec.steps.is_empty() {
                            status.phase = RunPhase::Succeeded;
                            status.message = "all steps succeeded".into();
                            status.finished_at = Some(Utc::now());
                            results.push(ReconcileResult {
                                name: name.clone(),
                                action: ReconcileAction::Complete,
                                reason: "all steps succeeded".into(),
                            });
                            to_complete.push(name);
                        } else if spec.steps.is_empty() {
                            status.phase = RunPhase::Succeeded;
                            status.finished_at = Some(Utc::now());
                            status.message = "no steps".into();
                            results.push(ReconcileResult {
                                name: name.clone(),
                                action: ReconcileAction::Complete,
                                reason: "no steps".into(),
                            });
                            to_complete.push(name);
                        } else {
                            results.push(ReconcileResult {
                                name,
                                action: ReconcileAction::Wait,
                                reason: "steps in progress".into(),
                            });
                        }
                    }
                    RunPhase::Cancelled => {
                        status.finished_at = Some(Utc::now());
                        status.message = "cancelled".into();
                        results.push(ReconcileResult {
                            name: name.clone(),
                            action: ReconcileAction::Cancel,
                            reason: "user cancelled".into(),
                        });
                        to_complete.push(name);
                    }
                    _ => {
                        results.push(ReconcileResult {
                            name,
                            action: ReconcileAction::Complete,
                            reason: "already terminal".into(),
                        });
                    }
                }
            }
        }

        for name in to_complete {
            self.finalize(&name);
        }

        results
    }

    fn finalize(&self, name: &str) {
        if let Some((_, status)) = self.running.remove(name) {
            let started = status.started_at;
            let finished = status.finished_at.unwrap_or(Utc::now());
            let duration = finished - started.unwrap_or(finished);
            self.completed.lock().push_back(CompletedRun {
                name: name.into(),
                status,
                duration,
            });
            self.queue.remove(name);
        }
    }

    pub fn get_status(&self, name: &str) -> Option<PipelineRunStatus> {
        self.running.get(name).map(|r| r.clone())
    }

    pub fn cancel(&self, name: &str) -> bool {
        if self.queue.contains_key(name)
            && let Some(mut status) = self.running.get_mut(name)
            && !status.value().is_terminal()
        {
            status.value_mut().phase = RunPhase::Cancelled;
            return true;
        }
        false
    }

    pub fn running_count(&self) -> usize {
        self.running
            .iter()
            .filter(|e| e.value().phase == RunPhase::Running)
            .count()
    }

    pub fn completed_count(&self) -> usize {
        self.completed.lock().len()
    }

    pub fn cleanup_old(&self, max_age: Duration) -> usize {
        let max_age_delta = chrono::Duration::from_std(max_age).unwrap_or(chrono::Duration::zero());
        let mut completed = self.completed.lock();
        let mut removed = 0;
        while let Some(front) = completed.front() {
            if front.duration < max_age_delta {
                break;
            }
            completed.pop_front();
            removed += 1;
        }
        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crd::{CrdStep, ResourceRequirements};
    use std::collections::HashMap;

    fn test_spec(name: &str) -> PipelineRunSpec {
        PipelineRunSpec {
            name: name.into(),
            repo_url: "https://github.com/test/repo".into(),
            commit_sha: "sha".into(),
            branch: "main".into(),
            steps: vec![CrdStep {
                name: "build".into(),
                image: "alpine:latest".into(),
                commands: vec!["make".into()],
                env: HashMap::new(),
                condition: None,
                workdir: None,
            }],
            triggers: vec![],
            timeout_seconds: 300,
            resources: ResourceRequirements {
                cpu: "1".into(),
                memory: "1Gi".into(),
                gpu: None,
            },
            node_selector: HashMap::new(),
            tolerations: vec![],
        }
    }

    #[test]
    fn test_submit_and_get_status() {
        let r = Reconciler::new("default", Duration::from_secs(30), 4);
        let spec = test_spec("run-1");
        r.submit(spec).unwrap();
        let status = r.get_status("run-1").unwrap();
        assert_eq!(status.phase, RunPhase::Pending);
    }

    #[test]
    fn test_submit_duplicate_fails() {
        let r = Reconciler::new("default", Duration::from_secs(30), 4);
        r.submit(test_spec("dup")).unwrap();
        assert!(r.submit(test_spec("dup")).is_err());
    }

    #[test]
    fn test_reconcile_pending_to_running() {
        let r = Reconciler::new("default", Duration::from_secs(30), 4);
        r.submit(test_spec("run-a")).unwrap();
        let results = r.reconcile_all();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].action, ReconcileAction::Schedule);
        assert_eq!(r.get_status("run-a").unwrap().phase, RunPhase::Running);
    }

    #[test]
    fn test_reconcile_wait_when_full() {
        let r = Reconciler::new("default", Duration::from_secs(30), 1);
        r.submit(test_spec("run-x")).unwrap();
        r.submit(test_spec("run-y")).unwrap();
        let results = r.reconcile_all();
        let scheduled = results
            .iter()
            .filter(|r| r.action == ReconcileAction::Schedule)
            .count();
        let waiting = results
            .iter()
            .filter(|r| r.action == ReconcileAction::Wait)
            .count();
        assert_eq!(scheduled, 1);
        assert_eq!(waiting, 1);
    }

    #[test]
    fn test_cancel() {
        let r = Reconciler::new("default", Duration::from_secs(30), 4);
        r.submit(test_spec("run-c")).unwrap();
        r.reconcile_all();
        let cancelled = r.cancel("run-c");
        assert!(cancelled);
        let results = r.reconcile_all();
        let cancel_result = results.iter().find(|r| r.action == ReconcileAction::Cancel);
        assert!(cancel_result.is_some());
    }

    #[test]
    fn test_completed_count() {
        let r = Reconciler::new("default", Duration::from_secs(30), 4);
        r.submit(test_spec("run-d")).unwrap();
        r.reconcile_all();
        assert_eq!(r.completed_count(), 0);
        r.reconcile_all();
        assert_eq!(r.completed_count(), 0);
    }

    #[test]
    fn test_running_count() {
        let r = Reconciler::new("default", Duration::from_secs(30), 4);
        r.submit(test_spec("rc1")).unwrap();
        r.submit(test_spec("rc2")).unwrap();
        r.reconcile_all();
        assert_eq!(r.running_count(), 2);
    }

    #[test]
    fn test_reconcile_empty_steps_completes() {
        let r = Reconciler::new("default", Duration::from_secs(30), 4);
        let mut spec = test_spec("empty-steps");
        spec.steps.clear();
        r.submit(spec).unwrap();
        let results = r.reconcile_all();
        assert!(
            results
                .iter()
                .any(|r| r.action == ReconcileAction::Schedule)
        );
        let results2 = r.reconcile_all();
        assert!(
            results2
                .iter()
                .any(|r| r.action == ReconcileAction::Complete)
        );
    }
}
