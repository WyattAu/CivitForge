#![forbid(unsafe_code)]

use crate::crd::{PipelineRunSpec, Toleration};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug)]
pub struct NodePool {
    pub name: String,
    pub labels: HashMap<String, String>,
    pub capacity: u32,
    pub available: AtomicU32,
    pub taints: Vec<Toleration>,
}

impl NodePool {
    pub fn new(name: impl Into<String>, capacity: u32, labels: HashMap<String, String>) -> Self {
        let avail = AtomicU32::new(capacity);
        Self {
            name: name.into(),
            labels,
            capacity,
            available: avail,
            taints: vec![],
        }
    }

    pub fn acquire(&self) -> bool {
        loop {
            let cur = self.available.load(Ordering::SeqCst);
            if cur == 0 {
                return false;
            }
            if self
                .available
                .compare_exchange(cur, cur - 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return true;
            }
        }
    }

    pub fn release_slot(&self) {
        let cur = self.available.load(Ordering::SeqCst);
        let new = if cur < self.capacity { cur + 1 } else { cur };
        self.available.store(new, Ordering::SeqCst);
    }
}

#[derive(Debug, Clone)]
pub struct ScheduleDecision {
    pub pool: String,
    pub node: Option<String>,
    pub reason: String,
}

pub struct Scheduler {
    pub node_pools: HashMap<String, NodePool>,
    pub default_pool: String,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            node_pools: HashMap::new(),
            default_pool: String::new(),
        }
    }

    pub fn add_pool(&mut self, pool: NodePool) {
        let name = pool.name.clone();
        if self.default_pool.is_empty() {
            self.default_pool = name.clone();
        }
        self.node_pools.insert(name, pool);
    }

    pub fn schedule(&self, spec: &PipelineRunSpec) -> ScheduleDecision {
        if let Some(ref gpu) = spec.resources.gpu {
            let gpu_pool_name = self.find_gpu_pool();
            if let Some(pool_name) = gpu_pool_name {
                if let Some(pool) = self.node_pools.get(&pool_name) {
                    if pool.acquire() {
                        return ScheduleDecision {
                            pool: pool_name.clone(),
                            node: None,
                            reason: format!("gpu requirement matched pool '{}'", pool_name),
                        };
                    }
                    return ScheduleDecision {
                        pool: pool_name.clone(),
                        node: None,
                        reason: format!("gpu pool '{}' full", pool_name),
                    };
                }
            }
            return ScheduleDecision {
                pool: String::new(),
                node: None,
                reason: format!("no pool for gpu request: {}", gpu),
            };
        }

        let preferred = self.find_matching_pool(spec);
        if let Some(pool_name) = preferred {
            if let Some(pool) = self.node_pools.get(&pool_name) {
                if pool.acquire() {
                    return ScheduleDecision {
                        pool: pool_name.clone(),
                        node: None,
                        reason: format!("matched pool '{}'", pool_name),
                    };
                }
            }
        }

        if let Some(pool) = self.node_pools.get(&self.default_pool) {
            if pool.acquire() {
                return ScheduleDecision {
                    pool: self.default_pool.clone(),
                    node: None,
                    reason: "fallback to default pool".into(),
                };
            }
        }

        ScheduleDecision {
            pool: String::new(),
            node: None,
            reason: "no capacity available".into(),
        }
    }

    pub fn release(&self, pool: &str) {
        if let Some(p) = self.node_pools.get(pool) {
            p.release_slot();
        }
    }

    pub fn available_capacity(&self, pool: &str) -> u32 {
        self.node_pools
            .get(pool)
            .map(|p| p.available.load(Ordering::SeqCst))
            .unwrap_or(0)
    }

    fn find_gpu_pool(&self) -> Option<String> {
        for (name, pool) in &self.node_pools {
            let has_gpu_label = pool.labels.iter().any(|(k, _)| k.contains("gpu"));
            if has_gpu_label {
                return Some(name.clone());
            }
        }
        None
    }

    fn find_matching_pool(&self, spec: &PipelineRunSpec) -> Option<String> {
        for (name, pool) in &self.node_pools {
            let mut matches = true;
            for (k, v) in &spec.node_selector {
                if pool.labels.get(k) != Some(v) {
                    matches = false;
                    break;
                }
            }
            if matches && pool.available.load(Ordering::SeqCst) > 0 {
                return Some(name.clone());
            }
        }
        None
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crd::ResourceRequirements;

    fn make_spec(gpu: bool) -> PipelineRunSpec {
        PipelineRunSpec {
            name: "test".into(),
            repo_url: "https://github.com/test/repo".into(),
            commit_sha: "sha".into(),
            branch: "main".into(),
            steps: vec![],
            triggers: vec![],
            timeout_seconds: 300,
            resources: ResourceRequirements {
                cpu: "1".into(),
                memory: "1Gi".into(),
                gpu: if gpu {
                    Some("nvidia.com/gpu=1".into())
                } else {
                    None
                },
            },
            node_selector: HashMap::new(),
            tolerations: vec![],
        }
    }

    #[test]
    fn test_node_pool_acquire_release() {
        let pool = NodePool::new("test-pool", 2, HashMap::new());
        assert!(pool.acquire());
        assert!(pool.acquire());
        assert!(!pool.acquire());
        pool.release_slot();
        assert!(pool.acquire());
    }

    #[test]
    fn test_schedule_to_default_pool() {
        let mut sched = Scheduler::new();
        sched.add_pool(NodePool::new("general", 4, HashMap::new()));
        let spec = make_spec(false);
        let decision = sched.schedule(&spec);
        assert_eq!(decision.pool, "general");
        assert_eq!(sched.available_capacity("general"), 3);
    }

    #[test]
    fn test_gpu_scheduling() {
        let mut sched = Scheduler::new();
        sched.add_pool(NodePool::new("general", 4, HashMap::new()));
        let mut gpu_labels = HashMap::new();
        gpu_labels.insert("gpu".into(), "true".into());
        sched.add_pool(NodePool::new("gpu-pool", 2, gpu_labels));

        let spec = make_spec(true);
        let decision = sched.schedule(&spec);
        assert_eq!(decision.pool, "gpu-pool");
        assert_eq!(sched.available_capacity("gpu-pool"), 1);
    }

    #[test]
    fn test_no_capacity() {
        let mut sched = Scheduler::new();
        let pool = NodePool::new("tiny", 0, HashMap::new());
        sched.add_pool(pool);
        let spec = make_spec(false);
        let decision = sched.schedule(&spec);
        assert_eq!(decision.reason, "no capacity available");
    }

    #[test]
    fn test_release_increases_capacity() {
        let mut sched = Scheduler::new();
        sched.add_pool(NodePool::new("rel", 1, HashMap::new()));
        let spec = make_spec(false);
        let decision = sched.schedule(&spec);
        assert_eq!(decision.pool, "rel");
        assert_eq!(sched.available_capacity("rel"), 0);
        sched.release("rel");
        assert_eq!(sched.available_capacity("rel"), 1);
    }

    #[test]
    fn test_node_selector_matching() {
        let mut sched = Scheduler::new();
        let mut labels = HashMap::new();
        labels.insert("disktype".into(), "ssd".into());
        sched.add_pool(NodePool::new("ssd-pool", 2, labels));
        sched.add_pool(NodePool::new("general", 10, HashMap::new()));

        let mut spec = make_spec(false);
        spec.node_selector.insert("disktype".into(), "ssd".into());
        let decision = sched.schedule(&spec);
        assert_eq!(decision.pool, "ssd-pool");
    }

    #[test]
    fn test_gpu_pool_full_falls_back() {
        let mut sched = Scheduler::new();
        let mut gpu_labels = HashMap::new();
        gpu_labels.insert("gpu".into(), "true".into());
        sched.add_pool(NodePool::new("gpu-pool", 0, gpu_labels));
        sched.add_pool(NodePool::new("general", 4, HashMap::new()));

        let spec = make_spec(true);
        let decision = sched.schedule(&spec);
        assert_eq!(decision.pool, "gpu-pool");
        assert!(decision.reason.contains("full"));
    }
}
