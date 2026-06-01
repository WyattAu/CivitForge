#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type PartitionId = u32;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PartitionScheme {
    ByRepository,
    ByOrganization,
    ByEventType,
    RoundRobin,
    ConsistentHash { virtual_nodes: u32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Partition {
    pub id: PartitionId,
    pub topic: String,
    pub node_assignment: String,
    pub lag: u64,
    pub message_count: u64,
}

impl Partition {
    pub fn new(id: PartitionId, topic: String, node_assignment: String) -> Self {
        Self {
            id,
            topic,
            node_assignment,
            lag: 0,
            message_count: 0,
        }
    }
}

pub struct Partitioner {
    scheme: PartitionScheme,
    partitions: Vec<Partition>,
    nodes: Vec<String>,
    round_robin_index: PartitionId,
    hash_ring: HashMap<u64, PartitionId>,
}

impl Partitioner {
    pub fn new(scheme: PartitionScheme, num_partitions: u32, nodes: Vec<String>) -> Self {
        let partitions: Vec<Partition> = (0..num_partitions)
            .map(|id| {
                let node_idx = (id as usize) % nodes.len().max(1);
                Partition::new(
                    id,
                    format!("topic-{id}"),
                    nodes.get(node_idx).cloned().unwrap_or_default(),
                )
            })
            .collect();

        let hash_ring = if matches!(scheme, PartitionScheme::ConsistentHash { .. }) {
            build_hash_ring(&partitions, 150)
        } else {
            HashMap::new()
        };

        Self {
            scheme,
            partitions,
            nodes,
            round_robin_index: 0,
            hash_ring,
        }
    }

    pub fn assign_partition(&mut self, key: &str) -> PartitionId {
        match &self.scheme {
            PartitionScheme::RoundRobin => {
                let id = self.round_robin_index % self.partitions.len() as PartitionId;
                self.round_robin_index =
                    (self.round_robin_index + 1) % self.partitions.len() as PartitionId;
                id
            }
            PartitionScheme::ByRepository | PartitionScheme::ByEventType => {
                let hash = simple_hash(key);
                (hash as PartitionId) % self.partitions.len() as PartitionId
            }
            PartitionScheme::ByOrganization => {
                let hash = simple_hash(key);
                (hash as PartitionId) % self.partitions.len() as PartitionId
            }
            PartitionScheme::ConsistentHash { .. } => {
                let hash = consistent_hash(key);
                self.find_partition_for_hash(hash)
            }
        }
    }

    pub fn rebalance(&mut self) -> Vec<(PartitionId, String, String)> {
        let mut reassignments = Vec::new();
        for partition in &mut self.partitions {
            let new_node_idx = (partition.id as usize) % self.nodes.len().max(1);
            let new_node = self.nodes.get(new_node_idx).cloned().unwrap_or_default();
            if partition.node_assignment != new_node {
                let old = std::mem::replace(&mut partition.node_assignment, new_node.clone());
                reassignments.push((partition.id, old, new_node));
            }
        }
        if matches!(self.scheme, PartitionScheme::ConsistentHash { .. }) {
            self.hash_ring = build_hash_ring(&self.partitions, 150);
        }
        reassignments
    }

    pub fn add_node(&mut self, node: String) {
        if !self.nodes.contains(&node) {
            self.nodes.push(node);
            self.rebalance();
        }
    }

    pub fn remove_node(&mut self, node: &str) -> bool {
        if let Some(pos) = self.nodes.iter().position(|n| n == node) {
            self.nodes.remove(pos);
            self.rebalance();
            true
        } else {
            false
        }
    }

    pub fn get_partition_stats(&self) -> Vec<&Partition> {
        self.partitions.iter().collect()
    }

    pub fn partition_count(&self) -> usize {
        self.partitions.len()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn record_message(&mut self, partition_id: PartitionId) {
        if let Some(p) = self.partitions.iter_mut().find(|p| p.id == partition_id) {
            p.message_count += 1;
        }
    }

    pub fn set_lag(&mut self, partition_id: PartitionId, lag: u64) {
        if let Some(p) = self.partitions.iter_mut().find(|p| p.id == partition_id) {
            p.lag = lag;
        }
    }

    fn find_partition_for_hash(&self, hash: u64) -> PartitionId {
        if self.hash_ring.is_empty() {
            return 0;
        }
        let keys: Vec<u64> = self
            .hash_ring
            .keys()
            .filter(|&&k| k <= hash)
            .copied()
            .collect();
        let key = keys
            .last()
            .copied()
            .unwrap_or_else(|| self.hash_ring.keys().copied().max().unwrap_or(0));
        *self.hash_ring.get(&key).unwrap_or(&0)
    }
}

fn simple_hash(key: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in key.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}

fn consistent_hash(key: &str) -> u64 {
    let bytes = key.as_bytes();
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn build_hash_ring(partitions: &[Partition], virtual_nodes: u32) -> HashMap<u64, PartitionId> {
    let mut ring = HashMap::new();
    for p in partitions {
        for i in 0..virtual_nodes {
            let vnode_key = format!("{}:{}", p.id, i);
            let hash = consistent_hash(&vnode_key);
            ring.entry(hash).or_insert(p.id);
        }
    }
    ring
}

#[derive(Debug, Clone, PartialEq)]
pub enum LbAlgorithm {
    RoundRobin,
    LeastConnections,
    Random,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadBalancerConfig {
    pub algorithm: LbAlgorithm,
    pub health_check_interval_secs: u64,
    pub max_retries: u32,
    pub timeout_ms: u64,
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            algorithm: LbAlgorithm::RoundRobin,
            health_check_interval_secs: 30,
            max_retries: 3,
            timeout_ms: 5000,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AutoscalingPolicy {
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub target_cpu_percent: f32,
    pub scale_up_threshold: f32,
    pub scale_down_threshold: f32,
    pub cooldown_secs: u64,
}

impl Default for AutoscalingPolicy {
    fn default() -> Self {
        Self {
            min_replicas: 1,
            max_replicas: 10,
            target_cpu_percent: 70.0,
            scale_up_threshold: 80.0,
            scale_down_threshold: 30.0,
            cooldown_secs: 300,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScalingDecision {
    ScaleUp(u32),
    ScaleDown(u32),
    NoOp,
}

#[derive(Debug, Clone)]
pub struct ScalingMetrics {
    pub current_replicas: u32,
    pub cpu_percent: f32,
    pub memory_percent: f32,
    pub request_rate: f64,
    pub active_connections: u32,
}

pub struct Autoscaler {
    policy: AutoscalingPolicy,
    last_scale_action: Option<std::time::Instant>,
}

impl Autoscaler {
    pub fn new(policy: AutoscalingPolicy) -> Self {
        Self {
            policy,
            last_scale_action: None,
        }
    }

    pub fn evaluate(&mut self, metrics: &ScalingMetrics) -> ScalingDecision {
        let current = metrics.current_replicas;

        if let Some(last) = self.last_scale_action {
            if last.elapsed().as_secs() < self.policy.cooldown_secs {
                return ScalingDecision::NoOp;
            }
        }

        if metrics.cpu_percent > self.policy.scale_up_threshold
            && current < self.policy.max_replicas
        {
            let desired = std::cmp::min(current + 1, self.policy.max_replicas);
            self.last_scale_action = Some(std::time::Instant::now());
            return ScalingDecision::ScaleUp(desired);
        }

        if metrics.cpu_percent < self.policy.scale_down_threshold
            && current > self.policy.min_replicas
        {
            let desired = std::cmp::max(current - 1, self.policy.min_replicas);
            self.last_scale_action = Some(std::time::Instant::now());
            return ScalingDecision::ScaleDown(desired);
        }

        ScalingDecision::NoOp
    }

    pub fn set_policy(&mut self, policy: AutoscalingPolicy) {
        self.policy = policy;
    }

    pub fn policy(&self) -> &AutoscalingPolicy {
        &self.policy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_hash_deterministic() {
        assert_eq!(simple_hash("repo1"), simple_hash("repo1"));
        assert_eq!(simple_hash("repo1"), simple_hash("repo1"));
        assert_ne!(simple_hash("repo1"), simple_hash("repo2"));
    }

    #[test]
    fn test_consistent_hash_deterministic() {
        assert_eq!(consistent_hash("key-a"), consistent_hash("key-a"));
        assert_ne!(consistent_hash("key-a"), consistent_hash("key-b"));
    }

    #[test]
    fn test_build_hash_ring() {
        let partitions = vec![
            Partition::new(0, "t0".into(), "n1".into()),
            Partition::new(1, "t1".into(), "n2".into()),
        ];
        let ring = build_hash_ring(&partitions, 100);
        assert_eq!(ring.len(), 200);
    }

    #[test]
    fn test_partitioner_new_round_robin() {
        let p = Partitioner::new(PartitionScheme::RoundRobin, 4, vec!["node1".into()]);
        assert_eq!(p.partition_count(), 4);
        assert_eq!(p.node_count(), 1);
    }

    #[test]
    fn test_partitioner_assign_round_robin() {
        let mut p = Partitioner::new(PartitionScheme::RoundRobin, 4, vec!["node1".into()]);
        let mut assigned = Vec::new();
        for _ in 0..8 {
            assigned.push(p.assign_partition("any-key"));
        }
        assert_eq!(assigned, vec![0, 1, 2, 3, 0, 1, 2, 3]);
    }

    #[test]
    fn test_partitioner_assign_by_repo() {
        let mut p = Partitioner::new(PartitionScheme::ByRepository, 4, vec!["node1".into()]);
        let id1 = p.assign_partition("user/repo1");
        let id2 = p.assign_partition("user/repo1");
        let id3 = p.assign_partition("user/repo2");
        assert_eq!(id1, id2);
        assert!(id3 < 4);
    }

    #[test]
    fn test_partitioner_assign_by_event_type() {
        let mut p = Partitioner::new(PartitionScheme::ByEventType, 8, vec!["n1".into()]);
        let id1 = p.assign_partition("push");
        let id2 = p.assign_partition("push");
        let _id3 = p.assign_partition("issue");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_partitioner_assign_by_org() {
        let mut p = Partitioner::new(PartitionScheme::ByOrganization, 4, vec!["n1".into()]);
        let id1 = p.assign_partition("myorg");
        let id2 = p.assign_partition("myorg");
        let _id3 = p.assign_partition("otherorg");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_partitioner_assign_consistent_hash() {
        let mut p = Partitioner::new(
            PartitionScheme::ConsistentHash { virtual_nodes: 100 },
            4,
            vec!["n1".into()],
        );
        let id1 = p.assign_partition("stable-key");
        let id2 = p.assign_partition("stable-key");
        let id3 = p.assign_partition("other-key");
        assert_eq!(id1, id2);
        assert!(id1 < 4);
        assert!(id3 < 4);
    }

    #[test]
    fn test_partitioner_add_node() {
        use std::collections::HashSet;
        let mut p = Partitioner::new(PartitionScheme::RoundRobin, 4, vec!["n1".into()]);
        p.add_node("n2".into());
        assert_eq!(p.node_count(), 2);
        let stats = p.get_partition_stats();
        let assigned_nodes: HashSet<String> =
            stats.iter().map(|s| s.node_assignment.clone()).collect();
        assert!(assigned_nodes.contains("n1") || assigned_nodes.contains("n2"));
    }

    #[test]
    fn test_partitioner_add_node_duplicate() {
        let mut p = Partitioner::new(PartitionScheme::RoundRobin, 4, vec!["n1".into()]);
        p.add_node("n1".into());
        assert_eq!(p.node_count(), 1);
    }

    #[test]
    fn test_partitioner_remove_node() {
        let mut p = Partitioner::new(
            PartitionScheme::RoundRobin,
            4,
            vec!["n1".into(), "n2".into()],
        );
        assert!(p.remove_node("n2"));
        assert_eq!(p.node_count(), 1);
        assert!(!p.remove_node("n2"));
    }

    #[test]
    fn test_partitioner_rebalance() {
        let mut p = Partitioner::new(PartitionScheme::RoundRobin, 3, vec!["n1".into()]);
        let reassignments = p.rebalance();
        assert!(reassignments.is_empty());
    }

    #[test]
    fn test_partitioner_get_stats() {
        let p = Partitioner::new(PartitionScheme::RoundRobin, 3, vec!["n1".into()]);
        let stats = p.get_partition_stats();
        assert_eq!(stats.len(), 3);
        assert_eq!(stats[0].id, 0);
    }

    #[test]
    fn test_partitioner_record_message() {
        let mut p = Partitioner::new(PartitionScheme::RoundRobin, 3, vec!["n1".into()]);
        p.record_message(0);
        p.record_message(0);
        p.record_message(1);
        let stats = p.get_partition_stats();
        assert_eq!(stats[0].message_count, 2);
        assert_eq!(stats[1].message_count, 1);
        assert_eq!(stats[2].message_count, 0);
    }

    #[test]
    fn test_partitioner_set_lag() {
        let mut p = Partitioner::new(PartitionScheme::RoundRobin, 3, vec!["n1".into()]);
        p.set_lag(1, 500);
        let stats = p.get_partition_stats();
        assert_eq!(stats[0].lag, 0);
        assert_eq!(stats[1].lag, 500);
    }

    #[test]
    fn test_load_balancer_config_default() {
        let config = LoadBalancerConfig::default();
        assert_eq!(config.health_check_interval_secs, 30);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.timeout_ms, 5000);
        assert!(matches!(config.algorithm, LbAlgorithm::RoundRobin));
    }

    #[test]
    fn test_autoscaling_policy_default() {
        let policy = AutoscalingPolicy::default();
        assert_eq!(policy.min_replicas, 1);
        assert_eq!(policy.max_replicas, 10);
        assert_eq!(policy.target_cpu_percent, 70.0);
        assert_eq!(policy.scale_up_threshold, 80.0);
        assert_eq!(policy.scale_down_threshold, 30.0);
        assert_eq!(policy.cooldown_secs, 300);
    }

    #[test]
    fn test_autoscaler_scale_up() {
        let policy = AutoscalingPolicy::default();
        let mut autoscaler = Autoscaler::new(policy);
        let metrics = ScalingMetrics {
            current_replicas: 2,
            cpu_percent: 90.0,
            memory_percent: 50.0,
            request_rate: 1000.0,
            active_connections: 100,
        };
        let decision = autoscaler.evaluate(&metrics);
        assert_eq!(decision, ScalingDecision::ScaleUp(3));
    }

    #[test]
    fn test_autoscaler_scale_down() {
        let policy = AutoscalingPolicy::default();
        let mut autoscaler = Autoscaler::new(policy);
        let metrics = ScalingMetrics {
            current_replicas: 5,
            cpu_percent: 10.0,
            memory_percent: 20.0,
            request_rate: 50.0,
            active_connections: 10,
        };
        let decision = autoscaler.evaluate(&metrics);
        assert_eq!(decision, ScalingDecision::ScaleDown(4));
    }

    #[test]
    fn test_autoscaler_noop() {
        let policy = AutoscalingPolicy::default();
        let mut autoscaler = Autoscaler::new(policy);
        let metrics = ScalingMetrics {
            current_replicas: 3,
            cpu_percent: 50.0,
            memory_percent: 50.0,
            request_rate: 500.0,
            active_connections: 50,
        };
        let decision = autoscaler.evaluate(&metrics);
        assert_eq!(decision, ScalingDecision::NoOp);
    }

    #[test]
    fn test_autoscaler_respects_max_replicas() {
        let policy = AutoscalingPolicy {
            min_replicas: 1,
            max_replicas: 3,
            target_cpu_percent: 70.0,
            scale_up_threshold: 80.0,
            scale_down_threshold: 30.0,
            cooldown_secs: 0,
        };
        let mut autoscaler = Autoscaler::new(policy);
        let metrics = ScalingMetrics {
            current_replicas: 3,
            cpu_percent: 99.0,
            memory_percent: 99.0,
            request_rate: 9999.0,
            active_connections: 9999,
        };
        let decision = autoscaler.evaluate(&metrics);
        assert_eq!(decision, ScalingDecision::NoOp);
    }

    #[test]
    fn test_autoscaler_respects_min_replicas() {
        let policy = AutoscalingPolicy {
            min_replicas: 2,
            max_replicas: 10,
            target_cpu_percent: 70.0,
            scale_up_threshold: 80.0,
            scale_down_threshold: 30.0,
            cooldown_secs: 0,
        };
        let mut autoscaler = Autoscaler::new(policy);
        let metrics = ScalingMetrics {
            current_replicas: 2,
            cpu_percent: 5.0,
            memory_percent: 5.0,
            request_rate: 0.0,
            active_connections: 0,
        };
        let decision = autoscaler.evaluate(&metrics);
        assert_eq!(decision, ScalingDecision::NoOp);
    }

    #[test]
    fn test_autoscaler_cooldown() {
        let policy = AutoscalingPolicy {
            min_replicas: 1,
            max_replicas: 10,
            target_cpu_percent: 70.0,
            scale_up_threshold: 80.0,
            scale_down_threshold: 30.0,
            cooldown_secs: 3600,
        };
        let mut autoscaler = Autoscaler::new(policy);
        let metrics = ScalingMetrics {
            current_replicas: 2,
            cpu_percent: 90.0,
            memory_percent: 50.0,
            request_rate: 1000.0,
            active_connections: 100,
        };
        let d1 = autoscaler.evaluate(&metrics);
        assert_eq!(d1, ScalingDecision::ScaleUp(3));
        let d2 = autoscaler.evaluate(&metrics);
        assert_eq!(d2, ScalingDecision::NoOp);
    }

    #[test]
    fn test_autoscaler_set_policy() {
        let mut autoscaler = Autoscaler::new(AutoscalingPolicy::default());
        let new_policy = AutoscalingPolicy {
            min_replicas: 3,
            max_replicas: 20,
            target_cpu_percent: 60.0,
            scale_up_threshold: 70.0,
            scale_down_threshold: 20.0,
            cooldown_secs: 600,
        };
        autoscaler.set_policy(new_policy.clone());
        assert_eq!(autoscaler.policy().min_replicas, 3);
        assert_eq!(autoscaler.policy().max_replicas, 20);
    }

    #[test]
    fn test_partition_new() {
        let p = Partition::new(42, "my-topic".into(), "node-a".into());
        assert_eq!(p.id, 42);
        assert_eq!(p.topic, "my-topic");
        assert_eq!(p.node_assignment, "node-a");
        assert_eq!(p.lag, 0);
        assert_eq!(p.message_count, 0);
    }

    #[test]
    fn test_lb_algorithm_variants() {
        let algos = [
            LbAlgorithm::RoundRobin,
            LbAlgorithm::LeastConnections,
            LbAlgorithm::Random,
        ];
        assert_eq!(algos.len(), 3);
    }
}
