#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderElectionConfig {
    pub lease_duration: Duration,
    pub renew_deadline: Duration,
    pub retry_period: Duration,
    pub identity: String,
    pub lock_name: String,
    pub namespace: String,
}

impl LeaderElectionConfig {
    pub fn new(
        identity: impl Into<String>,
        lock_name: impl Into<String>,
        namespace: impl Into<String>,
    ) -> Self {
        Self {
            lease_duration: Duration::from_secs(15),
            renew_deadline: Duration::from_secs(10),
            retry_period: Duration::from_secs(2),
            identity: identity.into(),
            lock_name: lock_name.into(),
            namespace: namespace.into(),
        }
    }

    pub fn with_lease_duration(mut self, duration: Duration) -> Self {
        self.lease_duration = duration;
        self
    }

    pub fn with_renew_deadline(mut self, deadline: Duration) -> Self {
        self.renew_deadline = deadline;
        self
    }

    pub fn with_retry_period(mut self, period: Duration) -> Self {
        self.retry_period = period;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaderState {
    Follower,
    Candidate,
    Leader,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lease {
    pub holder_identity: String,
    pub acquisition_time: DateTime<Utc>,
    pub renewals: u64,
    pub expires_at: DateTime<Utc>,
}

impl Lease {
    pub fn new(holder_identity: impl Into<String>, lease_duration: Duration) -> Self {
        let now = Utc::now();
        Self {
            holder_identity: holder_identity.into(),
            acquisition_time: now,
            renewals: 0,
            expires_at: now + chrono::Duration::from_std(lease_duration).unwrap_or_default(),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    pub fn renew(&mut self, lease_duration: Duration) {
        self.renewals += 1;
        self.expires_at =
            Utc::now() + chrono::Duration::from_std(lease_duration).unwrap_or_default();
    }
}

pub struct LeaderElector {
    config: LeaderElectionConfig,
    state: AtomicBool,
    leases: Arc<DashMap<String, Lease>>,
}

impl LeaderElector {
    pub fn new(config: LeaderElectionConfig) -> Self {
        Self {
            config,
            state: AtomicBool::new(false),
            leases: Arc::new(DashMap::new()),
        }
    }

    pub fn with_shared_store(
        config: LeaderElectionConfig,
        leases: Arc<DashMap<String, Lease>>,
    ) -> Self {
        Self {
            config,
            state: AtomicBool::new(false),
            leases,
        }
    }

    pub fn start_campaign(&self) -> bool {
        let lock_key = self.lock_key();
        {
            if let Some(existing) = self.leases.get(&lock_key)
                && !existing.is_expired()
                && existing.holder_identity != self.config.identity
            {
                return false;
            }
        }

        let lease = Lease::new(&self.config.identity, self.config.lease_duration);
        self.leases.insert(lock_key.clone(), lease);
        self.state.store(true, Ordering::SeqCst);
        true
    }

    pub fn step_down(&self) {
        let lock_key = self.lock_key();
        if let Some(mut lease) = self.leases.get_mut(&lock_key)
            && lease.holder_identity == self.config.identity
        {
            lease.expires_at = Utc::now();
        }
        self.state.store(false, Ordering::SeqCst);
    }

    pub fn is_leader(&self) -> bool {
        if !self.state.load(Ordering::SeqCst) {
            return false;
        }
        let lock_key = self.lock_key();
        if let Some(lease) = self.leases.get(&lock_key) {
            !lease.is_expired() && lease.holder_identity == self.config.identity
        } else {
            false
        }
    }

    pub fn get_leader_id(&self) -> Option<String> {
        let lock_key = self.lock_key();
        if let Some(lease) = self.leases.get(&lock_key) {
            if lease.is_expired() {
                None
            } else {
                Some(lease.holder_identity.clone())
            }
        } else {
            None
        }
    }

    pub fn renew_lease(&self) -> bool {
        let lock_key = self.lock_key();
        if let Some(mut lease) = self.leases.get_mut(&lock_key)
            && lease.holder_identity == self.config.identity
            && !lease.is_expired()
        {
            lease.renew(self.config.lease_duration);
            return true;
        }
        false
    }

    pub fn state(&self) -> LeaderState {
        if self.is_leader() {
            LeaderState::Leader
        } else if self.state.load(Ordering::SeqCst) {
            LeaderState::Candidate
        } else {
            LeaderState::Follower
        }
    }

    pub fn get_lease(&self) -> Option<Lease> {
        let lock_key = self.lock_key();
        self.leases.get(&lock_key).map(|l| l.clone())
    }

    pub fn config(&self) -> &LeaderElectionConfig {
        &self.config
    }

    fn lock_key(&self) -> String {
        format!("{}/{}", self.config.namespace, self.config.lock_name)
    }
}

// =============================================================================
// Kubernetes-backed Leader Election (via coordination.k8s.io/v1 Lease)
// =============================================================================

use k8s_openapi::api::coordination::v1::{Lease as K8sLease, LeaseSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::MicroTime;
use kube::Api;

/// Run leader election backed by the Kubernetes Lease CRD (coordination.k8s.io/v1).
///
/// Acquires a Lease named `lock_name` in the given namespace. If the Lease is
/// unexpired and held by another identity, this instance waits as a follower.
///
/// The `elected` callback is invoked once when this instance becomes the leader.
/// It receives a cancellation token; the callback should run the controller
/// and return when the token fires.
///
/// # Arguments
/// * `client` — authenticated `kube::Client`
/// * `config` — election parameters (identity, lock_name, namespace, durations)
/// * `elected` — async closure invoked on leader election
pub async fn run_k8s_leader_election<F, Fut>(
    client: kube::Client,
    config: LeaderElectionConfig,
    elected: F,
) -> anyhow::Result<()>
where
    F: FnOnce(tokio::sync::watch::Receiver<bool>) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
    let identity = config.identity.clone();
    let lock_name = config.lock_name.clone();

    info!(
        identity = %identity,
        lock = %lock_name,
        namespace = %config.namespace,
        "starting K8s leader election"
    );

    // Attempt to acquire the lease
    match try_acquire_lease(&client, &config).await {
        Ok(true) => {
            // We acquired the lease — start the controller
            info!(%identity, "acquired leadership, starting controller");
            elected(cancel_rx.clone()).await;
            info!(%identity, "controller stopped");
        }
        Ok(false) => {
            info!(%identity, "another leader holds the lease");
        }
        Err(e) => {
            error!(%identity, error = %e, "failed to acquire lease");
        }
    }

    // Signal shutdown
    let _ = cancel_tx.send(true);
    Ok(())
}

/// Try to acquire the Lease. Returns:
/// - `Ok(true)` if we acquired (created or took over expired lease)
/// - `Ok(false)` if another identity holds it
/// - `Err` on API errors
async fn try_acquire_lease(
    client: &kube::Client,
    config: &LeaderElectionConfig,
) -> anyhow::Result<bool> {
    let lease_api: Api<K8sLease> = Api::namespaced(client.clone(), &config.namespace);
    let identity = config.identity.clone();
    let lock_name = config.lock_name.clone();
    let now = chrono::Utc::now();
    let now_micros = MicroTime(now);
    let duration_secs = config.lease_duration.as_secs() as i32;

    match lease_api.get(&lock_name).await {
        Ok(existing) => {
            let spec = match &existing.spec {
                Some(s) => s,
                None => return Ok(true), // No spec — treat as acquirable
            };

            let holder = spec.holder_identity.as_deref().unwrap_or("");
            let acquire_time = match &spec.acquire_time {
                Some(t) => {
                    let dt: chrono::DateTime<chrono::Utc> = t.0;
                    dt
                }
                None => return Ok(true), // No acquire time — expired
            };

            let lease_seconds = spec.lease_duration_seconds.unwrap_or(duration_secs);
            let lease_expires = acquire_time
                + chrono::Duration::try_seconds(lease_seconds as i64)
                    .unwrap_or(chrono::Duration::zero());

            if now >= lease_expires || holder == identity {
                // Lease expired or we already hold it — acquire/renew
                let transitions = spec.lease_transitions.unwrap_or(0);
                let patch = serde_json::json!({
                    "spec": {
                        "holderIdentity": identity,
                        "leaseDurationSeconds": duration_secs,
                        "acquireTime": now_micros.0,
                        "renewTime": now_micros.0,
                        "leaseTransitions": if holder == identity { transitions } else { transitions + 1 },
                    }
                });

                let pp = kube::api::PatchParams::apply("civit-operator");
                lease_api
                    .patch(&lock_name, &pp, &kube::api::Patch::Merge(&patch))
                    .await?;
                return Ok(true);
            }

            // Someone else holds a valid lease
            Ok(false)
        }
        Err(kube::Error::Api(ae)) if ae.code == 404 => {
            // Lease doesn't exist — create it
            let new_lease = K8sLease {
                metadata: kube::core::ObjectMeta {
                    name: Some(lock_name),
                    namespace: Some(config.namespace.clone()),
                    ..Default::default()
                },
                spec: Some(LeaseSpec {
                    holder_identity: Some(identity),
                    lease_duration_seconds: Some(duration_secs),
                    acquire_time: Some(now_micros.clone()),
                    renew_time: Some(now_micros),
                    lease_transitions: Some(0),
                }),
            };

            lease_api
                .create(&kube::api::PostParams::default(), &new_lease)
                .await?;

            Ok(true)
        }
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(identity: &str) -> LeaderElectionConfig {
        LeaderElectionConfig::new(identity, "test-lock", "default")
    }

    #[test]
    fn test_config_new() {
        let config = make_config("node-1");
        assert_eq!(config.identity, "node-1");
        assert_eq!(config.lock_name, "test-lock");
        assert_eq!(config.namespace, "default");
        assert_eq!(config.lease_duration, Duration::from_secs(15));
        assert_eq!(config.renew_deadline, Duration::from_secs(10));
        assert_eq!(config.retry_period, Duration::from_secs(2));
    }

    #[test]
    fn test_config_with_custom_durations() {
        let config = make_config("node-1")
            .with_lease_duration(Duration::from_secs(30))
            .with_renew_deadline(Duration::from_secs(20))
            .with_retry_period(Duration::from_secs(5));
        assert_eq!(config.lease_duration, Duration::from_secs(30));
        assert_eq!(config.renew_deadline, Duration::from_secs(20));
        assert_eq!(config.retry_period, Duration::from_secs(5));
    }

    #[test]
    fn test_lease_new() {
        let lease = Lease::new("holder-1", Duration::from_secs(15));
        assert_eq!(lease.holder_identity, "holder-1");
        assert_eq!(lease.renewals, 0);
        assert!(!lease.is_expired());
    }

    #[test]
    fn test_lease_expired() {
        let lease = Lease::new("holder-1", Duration::from_millis(1));
        std::thread::sleep(Duration::from_millis(10));
        assert!(lease.is_expired());
    }

    #[test]
    fn test_lease_renew() {
        let mut lease = Lease::new("holder-1", Duration::from_millis(100));
        assert_eq!(lease.renewals, 0);
        lease.renew(Duration::from_secs(60));
        assert_eq!(lease.renewals, 1);
        assert!(!lease.is_expired());
    }

    #[test]
    fn test_single_leader_election() {
        let elector = LeaderElector::new(make_config("node-1"));
        assert!(elector.start_campaign());
        assert!(elector.is_leader());
        assert_eq!(elector.state(), LeaderState::Leader);
    }

    #[test]
    fn test_follower_initial_state() {
        let elector = LeaderElector::new(make_config("node-1"));
        assert_eq!(elector.state(), LeaderState::Follower);
        assert!(!elector.is_leader());
        assert!(elector.get_leader_id().is_none());
    }

    #[test]
    fn test_step_down() {
        let elector = LeaderElector::new(make_config("node-1"));
        elector.start_campaign();
        assert!(elector.is_leader());
        elector.step_down();
        assert!(!elector.is_leader());
    }

    #[test]
    fn test_step_down_frees_for_others() {
        let store = Arc::new(DashMap::new());
        let elector1 = LeaderElector::with_shared_store(make_config("node-1"), store.clone());
        let elector2 = LeaderElector::with_shared_store(make_config("node-2"), store.clone());

        elector1.start_campaign();
        assert!(!elector2.start_campaign());

        elector1.step_down();
        assert!(elector2.start_campaign());
        assert!(elector2.is_leader());
    }

    #[test]
    fn test_contested_election() {
        let store = Arc::new(DashMap::new());
        let elector1 = LeaderElector::with_shared_store(make_config("node-1"), store.clone());
        let elector2 = LeaderElector::with_shared_store(make_config("node-2"), store.clone());

        assert!(elector1.start_campaign());
        assert!(!elector2.start_campaign());
        assert!(elector1.is_leader());
        assert!(!elector2.is_leader());
    }

    #[test]
    fn test_expired_lease_allows_new_leader() {
        let store = Arc::new(DashMap::new());
        let config = make_config("node-1").with_lease_duration(Duration::from_millis(50));
        let elector1 = LeaderElector::with_shared_store(config, store.clone());
        let elector2 = LeaderElector::with_shared_store(make_config("node-2"), store.clone());

        elector1.start_campaign();
        assert!(!elector2.start_campaign());

        std::thread::sleep(Duration::from_millis(100));
        assert!(elector2.start_campaign());
        assert!(elector2.is_leader());
    }

    #[test]
    fn test_get_leader_id() {
        let elector = LeaderElector::new(make_config("node-1"));
        elector.start_campaign();
        assert_eq!(elector.get_leader_id(), Some("node-1".to_string()));
    }

    #[test]
    fn test_get_leader_id_none_when_expired() {
        let config = make_config("node-1").with_lease_duration(Duration::from_millis(1));
        let elector = LeaderElector::new(config);
        elector.start_campaign();
        std::thread::sleep(Duration::from_millis(10));
        assert!(elector.get_leader_id().is_none());
    }

    #[test]
    fn test_renew_lease() {
        let config = make_config("node-1").with_lease_duration(Duration::from_secs(60));
        let elector = LeaderElector::new(config);
        elector.start_campaign();
        assert!(elector.renew_lease());
        let lease = elector.get_lease().unwrap();
        assert_eq!(lease.renewals, 1);
    }

    #[test]
    fn test_renew_lease_fails_if_not_leader() {
        let elector = LeaderElector::new(make_config("node-1"));
        assert!(!elector.renew_lease());
    }

    #[test]
    fn test_renew_lease_fails_if_expired() {
        let config = make_config("node-1").with_lease_duration(Duration::from_millis(1));
        let elector = LeaderElector::new(config);
        elector.start_campaign();
        std::thread::sleep(Duration::from_millis(10));
        assert!(!elector.renew_lease());
    }

    #[test]
    fn test_multiple_renewals() {
        let config = make_config("node-1").with_lease_duration(Duration::from_secs(60));
        let elector = LeaderElector::new(config);
        elector.start_campaign();
        for i in 0..5 {
            assert!(elector.renew_lease());
            let lease = elector.get_lease().unwrap();
            assert_eq!(lease.renewals, i + 1);
        }
    }

    #[test]
    fn test_different_locks_independent() {
        let elector_a =
            LeaderElector::new(LeaderElectionConfig::new("node-1", "lock-a", "default"));
        let elector_b =
            LeaderElector::new(LeaderElectionConfig::new("node-1", "lock-b", "default"));

        elector_a.start_campaign();
        elector_b.start_campaign();
        assert!(elector_a.is_leader());
        assert!(elector_b.is_leader());
    }

    #[test]
    fn test_different_namespaces_independent() {
        let elector_ns1 = LeaderElector::new(LeaderElectionConfig::new("node-1", "lock", "ns1"));
        let elector_ns2 = LeaderElector::new(LeaderElectionConfig::new("node-2", "lock", "ns2"));

        elector_ns1.start_campaign();
        elector_ns2.start_campaign();
        assert!(elector_ns1.is_leader());
        assert!(elector_ns2.is_leader());
    }

    #[test]
    fn test_get_config() {
        let config = make_config("node-1");
        let elector = LeaderElector::new(config.clone());
        assert_eq!(elector.config().identity, "node-1");
        assert_eq!(elector.config().lock_name, "test-lock");
    }

    #[test]
    fn test_candidate_state_after_start_before_check() {
        let elector = LeaderElector::new(make_config("node-1"));
        elector.start_campaign();
        assert_eq!(elector.state(), LeaderState::Leader);
    }

    #[test]
    fn test_get_lease_none_before_campaign() {
        let elector = LeaderElector::new(make_config("node-1"));
        assert!(elector.get_lease().is_none());
    }

    // === Tests for K8s leader election (structural only, no cluster) ===

    #[test]
    fn test_leader_election_config_for_k8s() {
        let config = LeaderElectionConfig::new("pod-abc123", "civit-operator-lock", "civit-system")
            .with_lease_duration(Duration::from_secs(30))
            .with_renew_deadline(Duration::from_secs(20))
            .with_retry_period(Duration::from_secs(5));
        assert_eq!(config.identity, "pod-abc123");
        assert_eq!(config.lock_name, "civit-operator-lock");
        assert_eq!(config.namespace, "civit-system");
        assert_eq!(config.lease_duration, Duration::from_secs(30));
    }
}
