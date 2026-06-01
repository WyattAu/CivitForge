#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AffinityOperator {
    In,
    NotIn,
    Exists,
    DoesNotExist,
    Gt,
    Lt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffinityTerm {
    pub key: String,
    pub operator: AffinityOperator,
    pub values: Vec<String>,
}

impl AffinityTerm {
    pub fn new(key: impl Into<String>, operator: AffinityOperator) -> Self {
        Self {
            key: key.into(),
            operator,
            values: vec![],
        }
    }

    pub fn with_values(mut self, values: Vec<String>) -> Self {
        self.values = values;
        self
    }

    pub fn matches_label(&self, labels: &HashMap<String, String>) -> bool {
        match &self.operator {
            AffinityOperator::In => labels
                .get(&self.key)
                .is_some_and(|v| self.values.contains(v)),
            AffinityOperator::NotIn => labels
                .get(&self.key)
                .is_none_or(|v| !self.values.contains(v)),
            AffinityOperator::Exists => labels.contains_key(&self.key),
            AffinityOperator::DoesNotExist => !labels.contains_key(&self.key),
            AffinityOperator::Gt => labels.get(&self.key).is_some_and(|v| {
                self.values.first().is_some_and(|threshold| {
                    v.parse::<i64>()
                        .ok()
                        .is_some_and(|val| val > threshold.parse::<i64>().unwrap_or(0))
                })
            }),
            AffinityOperator::Lt => labels.get(&self.key).is_some_and(|v| {
                self.values.first().is_some_and(|threshold| {
                    v.parse::<i64>()
                        .ok()
                        .is_some_and(|val| val < threshold.parse::<i64>().unwrap_or(0))
                })
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedAffinityTerm {
    pub weight: i32,
    pub term: AffinityTerm,
}

impl WeightedAffinityTerm {
    pub fn new(weight: i32, term: AffinityTerm) -> Self {
        Self { weight, term }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAffinity {
    pub required: Vec<AffinityTerm>,
    pub preferred: Vec<WeightedAffinityTerm>,
    pub anti_affinity: Vec<AffinityTerm>,
}

impl NodeAffinity {
    pub fn new() -> Self {
        Self {
            required: vec![],
            preferred: vec![],
            anti_affinity: vec![],
        }
    }

    pub fn require(mut self, term: AffinityTerm) -> Self {
        self.required.push(term);
        self
    }

    pub fn prefer(mut self, weight: i32, term: AffinityTerm) -> Self {
        self.preferred.push(WeightedAffinityTerm::new(weight, term));
        self
    }

    pub fn anti_affinity(mut self, term: AffinityTerm) -> Self {
        self.anti_affinity.push(term);
        self
    }
}

impl Default for NodeAffinity {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TolerationOperator {
    Equal,
    Exists,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaintEffect {
    NoSchedule,
    NoExecute,
    PreferNoSchedule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Toleration {
    pub key: Option<String>,
    pub operator: TolerationOperator,
    pub value: Option<String>,
    pub effect: Option<TaintEffect>,
    pub toleration_seconds: Option<u64>,
}

impl Toleration {
    pub fn new(key: impl Into<String>, operator: TolerationOperator) -> Self {
        Self {
            key: Some(key.into()),
            operator,
            value: None,
            effect: None,
            toleration_seconds: None,
        }
    }

    pub fn exists(effect: Option<TaintEffect>) -> Self {
        Self {
            key: None,
            operator: TolerationOperator::Exists,
            value: None,
            effect,
            toleration_seconds: None,
        }
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn with_effect(mut self, effect: TaintEffect) -> Self {
        self.effect = Some(effect);
        self
    }

    pub fn with_toleration_seconds(mut self, seconds: u64) -> Self {
        self.toleration_seconds = Some(seconds);
        self
    }

    pub fn tolerates(
        &self,
        taint_key: &str,
        taint_value: &Option<String>,
        taint_effect: &Option<TaintEffect>,
    ) -> bool {
        if self.effect.is_some() && self.effect != *taint_effect {
            return false;
        }

        match &self.operator {
            TolerationOperator::Exists => {
                self.key.is_none() || self.key.as_deref() == Some(taint_key)
            }
            TolerationOperator::Equal => {
                self.key.as_deref() == Some(taint_key)
                    && self
                        .value
                        .as_ref()
                        .is_none_or(|v| Some(v.as_str()) == taint_value.as_deref())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Taint {
    pub key: String,
    pub value: Option<String>,
    pub effect: TaintEffect,
}

impl Taint {
    pub fn new(key: impl Into<String>, effect: TaintEffect) -> Self {
        Self {
            key: key.into(),
            value: None,
            effect,
        }
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct SchedulingResult {
    pub node_name: String,
    pub score: i32,
    pub matched_affinities: Vec<String>,
    pub matched_tolerations: Vec<String>,
    pub taint_violations: Vec<String>,
}

impl SchedulingResult {
    pub fn new(node_name: impl Into<String>) -> Self {
        Self {
            node_name: node_name.into(),
            score: 0,
            matched_affinities: vec![],
            matched_tolerations: vec![],
            taint_violations: vec![],
        }
    }

    pub fn is_schedulable(&self) -> bool {
        self.taint_violations.is_empty()
    }
}

pub struct NodeAffinityMatcher;

impl NodeAffinityMatcher {
    pub fn match_node(
        affinity: &NodeAffinity,
        node_name: &str,
        node_labels: &HashMap<String, String>,
        _node_taints: &[Taint],
    ) -> SchedulingResult {
        let mut result = SchedulingResult::new(node_name);

        for term in &affinity.required {
            if term.matches_label(node_labels) {
                result.matched_affinities.push(term.key.clone());
            } else {
                result.score = i32::MIN;
                return result;
            }
        }

        for weighted in &affinity.preferred {
            if weighted.term.matches_label(node_labels) {
                result.score += weighted.weight;
                result.matched_affinities.push(weighted.term.key.clone());
            }
        }

        for term in &affinity.anti_affinity {
            if term.matches_label(node_labels) {
                result.score -= 100;
                result
                    .taint_violations
                    .push(format!("anti-affinity: {}", term.key));
            }
        }

        result
    }

    pub fn match_node_with_tolerations(
        affinity: &NodeAffinity,
        tolerations: &[Toleration],
        node_name: &str,
        node_labels: &HashMap<String, String>,
        node_taints: &[Taint],
    ) -> SchedulingResult {
        let mut result = Self::match_node(affinity, node_name, node_labels, node_taints);

        for taint in node_taints {
            let tolerated = tolerations
                .iter()
                .any(|t| t.tolerates(&taint.key, &taint.value, &Some(taint.effect.clone())));
            if !tolerated {
                result
                    .taint_violations
                    .push(format!("untolerated taint: {}", taint.key));
                result.score = i32::MIN;
            } else {
                result.matched_tolerations.push(taint.key.clone());
            }
        }

        result
    }

    pub fn gpu_affinity() -> NodeAffinity {
        NodeAffinity::new()
            .require(AffinityTerm::new(
                "node-role.kubernetes.io/gpu",
                AffinityOperator::Exists,
            ))
            .prefer(
                50,
                AffinityTerm::new("gpu-type", AffinityOperator::In)
                    .with_values(vec!["nvidia-tesla-a100".into()]),
            )
    }

    pub fn large_memory_affinity() -> NodeAffinity {
        NodeAffinity::new()
            .require(
                AffinityTerm::new("memory", AffinityOperator::Gt).with_values(vec!["64000".into()]),
            )
            .prefer(
                30,
                AffinityTerm::new("memory", AffinityOperator::Gt)
                    .with_values(vec!["128000".into()]),
            )
    }

    pub fn gpu_tolerations() -> Vec<Toleration> {
        vec![
            Toleration::new("nvidia.com/gpu", TolerationOperator::Exists)
                .with_effect(TaintEffect::NoSchedule),
        ]
    }

    pub fn large_memory_tolerations() -> Vec<Toleration> {
        vec![
            Toleration::new("dedicated", TolerationOperator::Equal)
                .with_value("memory-intensive")
                .with_effect(TaintEffect::NoSchedule),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gpu_node_labels() -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("node-role.kubernetes.io/gpu".into(), "true".into());
        m.insert("gpu-type".into(), "nvidia-tesla-a100".into());
        m
    }

    fn cpu_node_labels() -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("node-role.kubernetes.io/cpu".into(), "true".into());
        m
    }

    fn large_memory_labels() -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("memory".into(), "131072".into());
        m
    }

    fn small_memory_labels() -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("memory".into(), "16000".into());
        m
    }

    #[test]
    fn test_affinity_operator_in_match() {
        let term =
            AffinityTerm::new("zone", AffinityOperator::In).with_values(vec!["us-east-1a".into()]);
        let mut labels = HashMap::new();
        labels.insert("zone".into(), "us-east-1a".into());
        assert!(term.matches_label(&labels));
    }

    #[test]
    fn test_affinity_operator_in_no_match() {
        let term =
            AffinityTerm::new("zone", AffinityOperator::In).with_values(vec!["us-east-1a".into()]);
        let mut labels = HashMap::new();
        labels.insert("zone".into(), "us-west-2b".into());
        assert!(!term.matches_label(&labels));
    }

    #[test]
    fn test_affinity_operator_not_in_match() {
        let term = AffinityTerm::new("zone", AffinityOperator::NotIn)
            .with_values(vec!["us-east-1a".into()]);
        let mut labels = HashMap::new();
        labels.insert("zone".into(), "us-west-2b".into());
        assert!(term.matches_label(&labels));
    }

    #[test]
    fn test_affinity_operator_exists() {
        let term = AffinityTerm::new("gpu", AffinityOperator::Exists);
        let mut labels = HashMap::new();
        labels.insert("gpu".into(), "any".into());
        assert!(term.matches_label(&labels));
    }

    #[test]
    fn test_affinity_operator_does_not_exist() {
        let term = AffinityTerm::new("gpu", AffinityOperator::DoesNotExist);
        let labels = HashMap::new();
        assert!(term.matches_label(&labels));
    }

    #[test]
    fn test_affinity_operator_gt() {
        let term =
            AffinityTerm::new("memory", AffinityOperator::Gt).with_values(vec!["64000".into()]);
        assert!(term.matches_label(&large_memory_labels()));
        assert!(!term.matches_label(&small_memory_labels()));
    }

    #[test]
    fn test_affinity_operator_lt() {
        let term =
            AffinityTerm::new("memory", AffinityOperator::Lt).with_values(vec!["32000".into()]);
        assert!(term.matches_label(&small_memory_labels()));
        assert!(!term.matches_label(&large_memory_labels()));
    }

    #[test]
    fn test_required_affinity_pass() {
        let affinity = NodeAffinity::new().require(AffinityTerm::new(
            "node-role.kubernetes.io/gpu",
            AffinityOperator::Exists,
        ));
        let result =
            NodeAffinityMatcher::match_node(&affinity, "gpu-node", &gpu_node_labels(), &[]);
        assert!(result.score != i32::MIN);
        assert!(
            result
                .matched_affinities
                .contains(&"node-role.kubernetes.io/gpu".to_string())
        );
    }

    #[test]
    fn test_required_affinity_fail() {
        let affinity = NodeAffinity::new().require(AffinityTerm::new(
            "node-role.kubernetes.io/gpu",
            AffinityOperator::Exists,
        ));
        let result =
            NodeAffinityMatcher::match_node(&affinity, "cpu-node", &cpu_node_labels(), &[]);
        assert_eq!(result.score, i32::MIN);
    }

    #[test]
    fn test_preferred_affinity_scoring() {
        let affinity = NodeAffinity::new().prefer(
            50,
            AffinityTerm::new("gpu-type", AffinityOperator::In)
                .with_values(vec!["nvidia-tesla-a100".into()]),
        );
        let result =
            NodeAffinityMatcher::match_node(&affinity, "a100-node", &gpu_node_labels(), &[]);
        assert_eq!(result.score, 50);
        assert_eq!(result.matched_affinities.len(), 1);
    }

    #[test]
    fn test_preferred_affinity_no_match() {
        let affinity = NodeAffinity::new().prefer(
            50,
            AffinityTerm::new("gpu-type", AffinityOperator::In)
                .with_values(vec!["nvidia-tesla-a100".into()]),
        );
        let result =
            NodeAffinityMatcher::match_node(&affinity, "other-node", &cpu_node_labels(), &[]);
        assert_eq!(result.score, 0);
        assert_eq!(result.matched_affinities.len(), 0);
    }

    #[test]
    fn test_anti_affinity_penalty() {
        let mut labels = HashMap::new();
        labels.insert("env".into(), "production".into());
        let affinity = NodeAffinity::new().anti_affinity(
            AffinityTerm::new("env", AffinityOperator::In).with_values(vec!["production".into()]),
        );
        let result = NodeAffinityMatcher::match_node(&affinity, "prod-node", &labels, &[]);
        assert!(result.score < 0);
        assert!(!result.is_schedulable());
    }

    #[test]
    fn test_toleration_equal_match() {
        let tol = Toleration::new("dedicated", TolerationOperator::Equal)
            .with_value("gpu")
            .with_effect(TaintEffect::NoSchedule);
        assert!(tol.tolerates(
            "dedicated",
            &Some("gpu".into()),
            &Some(TaintEffect::NoSchedule)
        ));
    }

    #[test]
    fn test_toleration_equal_no_match_key() {
        let tol = Toleration::new("dedicated", TolerationOperator::Equal).with_value("gpu");
        assert!(!tol.tolerates("other", &Some("gpu".into()), &None));
    }

    #[test]
    fn test_toleration_exists_match() {
        let tol = Toleration::exists(Some(TaintEffect::NoSchedule));
        assert!(tol.tolerates(
            "any-key",
            &Some("any-val".into()),
            &Some(TaintEffect::NoSchedule)
        ));
    }

    #[test]
    fn test_toleration_exists_no_match_effect() {
        let tol = Toleration::exists(Some(TaintEffect::NoSchedule));
        assert!(!tol.tolerates("key", &None, &Some(TaintEffect::NoExecute)));
    }

    #[test]
    fn test_tolerations_with_node() {
        let affinity = NodeAffinity::new();
        let tolerations = vec![
            Toleration::new("dedicated", TolerationOperator::Equal)
                .with_value("gpu")
                .with_effect(TaintEffect::NoSchedule),
        ];
        let taints = vec![Taint::new("dedicated", TaintEffect::NoSchedule).with_value("gpu")];
        let result = NodeAffinityMatcher::match_node_with_tolerations(
            &affinity,
            &tolerations,
            "gpu-node",
            &gpu_node_labels(),
            &taints,
        );
        assert!(result.is_schedulable());
        assert!(
            result
                .matched_tolerations
                .contains(&"dedicated".to_string())
        );
    }

    #[test]
    fn test_untolerated_taint_blocks() {
        let affinity = NodeAffinity::new();
        let tolerations: Vec<Toleration> = vec![];
        let taints = vec![Taint::new("special", TaintEffect::NoSchedule)];
        let result = NodeAffinityMatcher::match_node_with_tolerations(
            &affinity,
            &tolerations,
            "special-node",
            &HashMap::new(),
            &taints,
        );
        assert!(!result.is_schedulable());
        assert_eq!(result.score, i32::MIN);
    }

    #[test]
    fn test_gpu_preset_builder() {
        let affinity = NodeAffinityMatcher::gpu_affinity();
        assert!(!affinity.required.is_empty());
        assert!(!affinity.preferred.is_empty());
    }

    #[test]
    fn test_large_memory_preset_builder() {
        let affinity = NodeAffinityMatcher::large_memory_affinity();
        let result =
            NodeAffinityMatcher::match_node(&affinity, "big-node", &large_memory_labels(), &[]);
        assert!(result.score > 0);
    }

    #[test]
    fn test_large_memory_preset_rejects_small() {
        let affinity = NodeAffinityMatcher::large_memory_affinity();
        let result =
            NodeAffinityMatcher::match_node(&affinity, "small-node", &small_memory_labels(), &[]);
        assert_eq!(result.score, i32::MIN);
    }

    #[test]
    fn test_gpu_tolerations_preset() {
        let tolerations = NodeAffinityMatcher::gpu_tolerations();
        assert_eq!(tolerations.len(), 1);
    }

    #[test]
    fn test_scheduling_result_is_schedulable() {
        let mut result = SchedulingResult::new("node-1");
        assert!(result.is_schedulable());
        result.taint_violations.push("bad taint".into());
        assert!(!result.is_schedulable());
    }

    #[test]
    fn test_toleration_with_seconds() {
        let tol = Toleration::new("key", TolerationOperator::Equal)
            .with_value("val")
            .with_effect(TaintEffect::NoExecute)
            .with_toleration_seconds(300);
        assert_eq!(tol.toleration_seconds, Some(300));
        assert_eq!(tol.effect, Some(TaintEffect::NoExecute));
    }
}
