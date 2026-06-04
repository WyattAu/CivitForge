#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorClock<V: Hash + Eq>(HashMap<V, u64>);

impl<V: Hash + Eq + Clone> VectorClock<V> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn increment(&mut self, node: &V) {
        let entry = self.0.entry(node.clone()).or_insert(0);
        *entry += 1;
    }

    pub fn get(&self, node: &V) -> u64 {
        *self.0.get(node).unwrap_or(&0)
    }

    pub fn happened_before(&self, other: &VectorClock<V>) -> bool {
        if self == other {
            return false;
        }
        for (key, val) in &self.0 {
            if other.0.get(key).copied().unwrap_or(0) < *val {
                return false;
            }
        }
        true
    }

    pub fn is_concurrent(&self, other: &VectorClock<V>) -> bool {
        !self.happened_before(other) && !other.happened_before(self)
    }

    pub fn merge(&mut self, other: &VectorClock<V>) {
        for (key, val) in &other.0 {
            let entry = self.0.entry(key.clone()).or_insert(0);
            *entry = (*entry).max(*val);
        }
    }

    pub fn descends_from(&self, other: &VectorClock<V>) -> bool {
        for (key, val) in &other.0 {
            if self.0.get(key).copied().unwrap_or(0) < *val {
                return false;
            }
        }
        true
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn entries(&self) -> &HashMap<V, u64> {
        &self.0
    }
}

impl<V: Hash + Eq + Clone> Default for VectorClock<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: Hash + Eq> PartialEq for VectorClock<V> {
    fn eq(&self, other: &Self) -> bool {
        let all_keys: std::collections::HashSet<&V> = self.0.keys().chain(other.0.keys()).collect();
        all_keys
            .iter()
            .all(|k| self.0.get(k).copied().unwrap_or(0) == other.0.get(k).copied().unwrap_or(0))
    }
}

impl<V: Hash + Eq> Eq for VectorClock<V> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_clock_is_empty() {
        let vc = VectorClock::<String>::new();
        assert!(vc.is_empty());
        assert_eq!(vc.len(), 0);
    }

    #[test]
    fn test_increment() {
        let mut vc = VectorClock::new();
        vc.increment(&"a".to_string());
        assert_eq!(vc.get(&"a".to_string()), 1);
        vc.increment(&"a".to_string());
        assert_eq!(vc.get(&"a".to_string()), 2);
        assert_eq!(vc.get(&"b".to_string()), 0);
    }

    #[test]
    fn test_multiple_nodes() {
        let mut vc = VectorClock::new();
        vc.increment(&"a".to_string());
        vc.increment(&"b".to_string());
        vc.increment(&"a".to_string());
        assert_eq!(vc.len(), 2);
        assert_eq!(vc.get(&"a".to_string()), 2);
        assert_eq!(vc.get(&"b".to_string()), 1);
    }

    #[test]
    fn test_happened_before() {
        let mut a = VectorClock::new();
        let mut b = VectorClock::new();
        a.increment(&"x".to_string());
        a.increment(&"y".to_string());
        b.increment(&"x".to_string());
        b.increment(&"y".to_string());
        b.increment(&"z".to_string());
        assert!(a.happened_before(&b));
        assert!(!b.happened_before(&a));
    }

    #[test]
    fn test_happened_before_equal_returns_false() {
        let mut a = VectorClock::new();
        let mut b = VectorClock::new();
        a.increment(&"x".to_string());
        b.increment(&"x".to_string());
        assert!(!a.happened_before(&b));
        assert!(!b.happened_before(&a));
    }

    #[test]
    fn test_happened_before_strictly_greater() {
        let mut a = VectorClock::new();
        let mut b = VectorClock::new();
        a.increment(&"x".to_string());
        a.increment(&"y".to_string());
        b.increment(&"x".to_string());
        assert!(!a.happened_before(&b));
    }

    #[test]
    fn test_is_concurrent() {
        let mut a = VectorClock::new();
        let mut b = VectorClock::new();
        a.increment(&"x".to_string());
        a.increment(&"y".to_string());
        b.increment(&"x".to_string());
        b.increment(&"z".to_string());
        assert!(a.is_concurrent(&b));
        assert!(b.is_concurrent(&a));
    }

    #[test]
    fn test_not_concurrent_when_ordered() {
        let mut a = VectorClock::new();
        let mut b = VectorClock::new();
        a.increment(&"x".to_string());
        b.increment(&"x".to_string());
        b.increment(&"y".to_string());
        assert!(!a.is_concurrent(&b));
    }

    #[test]
    fn test_merge() {
        let mut a = VectorClock::new();
        let mut b = VectorClock::new();
        a.increment(&"x".to_string());
        a.increment(&"x".to_string());
        b.increment(&"y".to_string());
        b.increment(&"y".to_string());
        b.increment(&"y".to_string());
        a.merge(&b);
        assert_eq!(a.get(&"x".to_string()), 2);
        assert_eq!(a.get(&"y".to_string()), 3);
    }

    #[test]
    fn test_merge_takes_max() {
        let mut a = VectorClock::new();
        let mut b = VectorClock::new();
        a.increment(&"x".to_string());
        a.increment(&"x".to_string());
        a.increment(&"x".to_string());
        b.increment(&"x".to_string());
        b.increment(&"x".to_string());
        a.merge(&b);
        assert_eq!(a.get(&"x".to_string()), 3);
    }

    #[test]
    fn test_descends_from() {
        let mut a = VectorClock::new();
        let mut b = VectorClock::new();
        a.increment(&"x".to_string());
        a.increment(&"y".to_string());
        a.increment(&"z".to_string());
        b.increment(&"x".to_string());
        b.increment(&"y".to_string());
        assert!(a.descends_from(&b));
        assert!(!b.descends_from(&a));
    }

    #[test]
    fn test_descends_from_equal() {
        let mut a = VectorClock::new();
        let mut b = VectorClock::new();
        a.increment(&"x".to_string());
        b.increment(&"x".to_string());
        assert!(a.descends_from(&b));
        assert!(b.descends_from(&a));
    }

    #[test]
    fn test_descends_from_missing_key() {
        let mut a = VectorClock::new();
        let mut b = VectorClock::new();
        a.increment(&"x".to_string());
        b.increment(&"x".to_string());
        b.increment(&"y".to_string());
        assert!(!a.descends_from(&b));
    }

    #[test]
    fn test_serialization() {
        let mut vc = VectorClock::new();
        vc.increment(&"region-a".to_string());
        vc.increment(&"region-b".to_string());
        vc.increment(&"region-a".to_string());
        let json = serde_json::to_string(&vc).unwrap();
        let deser: VectorClock<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.get(&"region-a".to_string()), 2);
        assert_eq!(deser.get(&"region-b".to_string()), 1);
    }
}
