#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Last-Writer-Wins Register: resolves conflicts by keeping the value with the highest timestamp.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LWWRegister<T: Clone> {
    pub value: T,
    pub timestamp: u64,
    pub node_id: String,
}

impl<T: Clone> LWWRegister<T> {
    pub fn new(value: T, timestamp: u64, node_id: String) -> Self {
        Self {
            value,
            timestamp,
            node_id,
        }
    }

    /// Merge with another register, keeping the one with the higher timestamp.
    /// On tie, the node_id with the lexicographically greater value wins.
    pub fn merge(self, other: LWWRegister<T>) -> LWWRegister<T> {
        if other.timestamp > self.timestamp {
            other
        } else if other.timestamp < self.timestamp {
            self
        } else if other.node_id > self.node_id {
            other
        } else {
            self
        }
    }

    pub fn get(&self) -> &T {
        &self.value
    }
}

/// Observed-Remove Set: supports concurrent add and remove. Remove is tracked via tombstones.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ORSet<T: Hash + Eq + Clone + Ord> {
    elements: HashMap<T, HashSet<(u64, String)>>,
    tombstones: HashMap<T, HashSet<(u64, String)>>,
}

impl<T: Hash + Eq + Clone + Ord> ORSet<T> {
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
            tombstones: HashMap::new(),
        }
    }

    pub fn add(&mut self, element: T, timestamp: u64, node_id: String) {
        self.elements
            .entry(element)
            .or_default()
            .insert((timestamp, node_id));
    }

    /// Mark an element for removal (tombstone).
    pub fn remove(&mut self, element: &T) {
        if let Some(tags) = self.elements.remove(element) {
            self.tombstones
                .entry(element.clone())
                .or_default()
                .extend(tags);
        }
    }

    /// Merge another ORSet into this one.
    /// Union of add tags, union of tombstone tags.
    pub fn merge(&mut self, other: ORSet<T>) {
        for (elem, other_tags) in other.elements {
            let own_tags = self.elements.entry(elem.clone()).or_default();
            own_tags.extend(other_tags);
            // If all tags for this element are in the tombstones, keep it removed
        }
        for (elem, other_tags) in other.tombstones {
            let own_tombs = self.tombstones.entry(elem.clone()).or_default();
            own_tombs.extend(other_tags);
        }
        // Clean up: if an element's add tags are a subset of its tombstone tags, it stays removed
        self.cleanup();
    }

    fn cleanup(&mut self) {
        let to_remove: Vec<T> = self
            .elements
            .iter()
            .filter(|(elem, tags)| {
                if let Some(tombs) = self.tombstones.get(*elem) {
                    // If all add tags are in tombstones, the element is dead
                    tags.iter().all(|t| tombs.contains(t))
                } else {
                    false
                }
            })
            .map(|(elem, _)| elem.clone())
            .collect();
        for elem in to_remove {
            self.elements.remove(&elem);
        }
    }

    pub fn contains(&self, element: &T) -> bool {
        self.elements.contains_key(element)
    }

    pub fn elements(&self) -> Vec<&T> {
        self.elements.keys().collect()
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

impl<T: Hash + Eq + Clone + Ord> Default for ORSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// An entry in an RGA sequence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RgaEntry<T: Clone> {
    pub value: T,
    pub timestamp: u64,
    pub node_id: String,
    pub parent_timestamp: u64,
    pub deleted: bool,
}

/// Replicated Growable Array: a CRDT for ordered sequences (like lists / text).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RGA<T: Clone> {
    pub entries: Vec<RgaEntry<T>>,
}

impl<T: Clone> RGA<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Insert a new value after the position identified by `after_timestamp`.
    /// `after_timestamp == 0` means insert at the beginning.
    pub fn insert(&mut self, value: T, after_timestamp: u64, timestamp: u64, node_id: String) {
        let pos = self
            .entries
            .iter()
            .position(|e| e.timestamp == after_timestamp)
            .map(|p| p + 1)
            .unwrap_or(0);

        self.entries.insert(
            pos,
            RgaEntry {
                value,
                timestamp,
                node_id,
                parent_timestamp: after_timestamp,
                deleted: false,
            },
        );
    }

    /// Append a value at the end.
    pub fn append(&mut self, value: T, timestamp: u64, node_id: String) {
        let parent = self.entries.last().map(|e| e.timestamp).unwrap_or(0);
        self.entries.push(RgaEntry {
            value,
            timestamp,
            node_id,
            parent_timestamp: parent,
            deleted: false,
        });
    }

    /// Mark an entry as deleted by its timestamp.
    pub fn delete(&mut self, timestamp: u64) {
        for entry in &mut self.entries {
            if entry.timestamp == timestamp {
                entry.deleted = true;
                break;
            }
        }
    }

    /// Merge two RGAs. Uses a causal-merge: entries present in both are kept,
    /// entries from either side are interleaved by timestamp ordering.
    pub fn merge(&mut self, other: RGA<T>) {
        let mut merged: Vec<RgaEntry<T>> = Vec::new();

        // Collect all known timestamps from self
        let self_timestamps: HashSet<u64> = self.entries.iter().map(|e| e.timestamp).collect();
        let _other_timestamps: HashSet<u64> = other.entries.iter().map(|e| e.timestamp).collect();

        // Add all entries from self
        for entry in self.entries.drain(..) {
            merged.push(entry);
        }

        // Add entries from other that are not in self
        for entry in other.entries {
            if !self_timestamps.contains(&entry.timestamp) {
                merged.push(entry);
            }
        }

        // Sort by timestamp to maintain ordering consistency
        merged.sort_by_key(|e| e.timestamp);

        self.entries = merged;
    }

    /// Get the sequence as a vector of references, excluding deleted entries.
    pub fn to_vec(&self) -> Vec<&T> {
        self.entries
            .iter()
            .filter(|e| !e.deleted)
            .map(|e| &e.value)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.entries.iter().filter(|e| !e.deleted).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: Clone> Default for RGA<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Add-Wins Set: similar to ORSet, but a concurrent add wins over a remove.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddWinsSet<T: Hash + Eq + Clone + Ord> {
    adds: HashMap<T, HashSet<(u64, String)>>,
    removes: HashMap<T, HashSet<(u64, String)>>,
}

impl<T: Hash + Eq + Clone + Ord> AddWinsSet<T> {
    pub fn new() -> Self {
        Self {
            adds: HashMap::new(),
            removes: HashMap::new(),
        }
    }

    pub fn add(&mut self, element: T, timestamp: u64, node_id: String) {
        self.adds
            .entry(element)
            .or_default()
            .insert((timestamp, node_id));
    }

    pub fn remove(&mut self, element: T, timestamp: u64, node_id: String) {
        // A node always knows about its own prior adds for this element.
        // Remove only the add tags from the same node_id (concurrent adds from other nodes remain).
        if let Some(add_tags) = self.adds.get(&element) {
            let own_tags: Vec<(u64, String)> = add_tags
                .iter()
                .filter(|(_, nid)| *nid == node_id)
                .cloned()
                .collect();
            self.removes
                .entry(element.clone())
                .or_default()
                .extend(own_tags);
        }
        // Also record the explicit remove tag
        self.removes
            .entry(element)
            .or_default()
            .insert((timestamp, node_id));
    }

    /// Merge another AddWinsSet. Add wins over concurrent remove.
    /// An element is present if its add set minus its remove set is non-empty.
    pub fn merge(&mut self, other: AddWinsSet<T>) {
        for (elem, other_tags) in other.adds {
            let own = self.adds.entry(elem.clone()).or_default();
            own.extend(other_tags);
        }
        for (elem, other_tags) in other.removes {
            let own = self.removes.entry(elem.clone()).or_default();
            own.extend(other_tags);
        }
    }

    /// Check if an element is present (add wins: present if add_tags > remove_tags for any tag).
    pub fn contains(&self, element: &T) -> bool {
        match (self.adds.get(element), self.removes.get(element)) {
            (Some(adds), Some(removes)) => {
                // Add wins: if there exists an add tag not covered by removes
                adds.iter().any(|t| !removes.contains(t))
            }
            (Some(_), None) => true,
            _ => false,
        }
    }

    pub fn elements(&self) -> Vec<&T> {
        self.adds.keys().filter(|k| self.contains(k)).collect()
    }

    pub fn len(&self) -> usize {
        self.adds.keys().filter(|k| self.contains(k)).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: Hash + Eq + Clone + Ord> Default for AddWinsSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// CausalContext wraps a VectorClock to provide causal ordering utilities.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CausalContext {
    clock: crate::federation::vector_clock::VectorClock<String>,
}

impl CausalContext {
    pub fn new() -> Self {
        Self {
            clock: crate::federation::vector_clock::VectorClock::new(),
        }
    }

    pub fn with_clock(clock: crate::federation::vector_clock::VectorClock<String>) -> Self {
        Self { clock }
    }

    pub fn increment(&mut self, node: &str) {
        self.clock.increment(&node.to_string());
    }

    /// Returns true if `a` happened before `b`.
    pub fn happens_before(
        &self,
        a: &crate::federation::vector_clock::VectorClock<String>,
        b: &crate::federation::vector_clock::VectorClock<String>,
    ) -> bool {
        a.happened_before(b)
    }

    /// Returns true if `a` and `b` are concurrent (neither happened before the other).
    pub fn concurrent(
        &self,
        a: &crate::federation::vector_clock::VectorClock<String>,
        b: &crate::federation::vector_clock::VectorClock<String>,
    ) -> bool {
        a.is_concurrent(b)
    }

    /// Merge two vector clocks, returning the result (element-wise max).
    pub fn merge_clocks(
        &self,
        a: &crate::federation::vector_clock::VectorClock<String>,
        b: &crate::federation::vector_clock::VectorClock<String>,
    ) -> crate::federation::vector_clock::VectorClock<String> {
        let mut result = a.clone();
        result.merge(b);
        result
    }

    pub fn clock(&self) -> &crate::federation::vector_clock::VectorClock<String> {
        &self.clock
    }
}

impl Default for CausalContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::federation::vector_clock::VectorClock;

    // ===== LWWRegister Tests =====

    #[test]
    fn lww_new_and_get() {
        let reg = LWWRegister::new("hello".to_string(), 100, "node-a".into());
        assert_eq!(reg.get(), "hello");
        assert_eq!(reg.timestamp, 100);
        assert_eq!(reg.node_id, "node-a");
    }

    #[test]
    fn lww_merge_with_newer() {
        let a = LWWRegister::new("old".to_string(), 10, "node-a".into());
        let b = LWWRegister::new("new".to_string(), 20, "node-b".into());
        let merged = a.merge(b);
        assert_eq!(merged.get(), "new");
        assert_eq!(merged.timestamp, 20);
    }

    #[test]
    fn lww_merge_with_older() {
        let a = LWWRegister::new("keep".to_string(), 50, "node-a".into());
        let b = LWWRegister::new("discard".to_string(), 10, "node-b".into());
        let merged = a.merge(b);
        assert_eq!(merged.get(), "keep");
        assert_eq!(merged.timestamp, 50);
    }

    #[test]
    fn lww_merge_same_timestamp_tiebreak_by_node_id() {
        let a = LWWRegister::new("value-a".to_string(), 100, "node-a".into());
        let b = LWWRegister::new("value-b".to_string(), 100, "node-b".into());
        let merged = a.merge(b);
        // "node-b" > "node-a" lexicographically
        assert_eq!(merged.get(), "value-b");
        assert_eq!(merged.node_id, "node-b");
    }

    #[test]
    fn lww_merge_same_timestamp_same_node() {
        let a = LWWRegister::new("value".to_string(), 100, "node-a".into());
        let b = LWWRegister::new("value2".to_string(), 100, "node-a".into());
        let merged = a.clone().merge(b);
        // Same timestamp and node_id => self wins
        assert_eq!(merged.get(), "value");
    }

    #[test]
    fn lww_merge_chain() {
        let a = LWWRegister::new(1, 10, "n1".into());
        let b = LWWRegister::new(2, 20, "n2".into());
        let c = LWWRegister::new(3, 15, "n3".into());
        let result = a.merge(b).merge(c);
        assert_eq!(*result.get(), 2); // timestamp 20 is highest
    }

    // ===== ORSet Tests =====

    #[test]
    fn orset_new_is_empty() {
        let s = ORSet::<String>::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn orset_add_and_contains() {
        let mut s = ORSet::new();
        s.add("hello".to_string(), 1, "n1".into());
        assert!(s.contains(&"hello".to_string()));
        assert!(!s.contains(&"world".to_string()));
    }

    #[test]
    fn orset_add_multiple() {
        let mut s = ORSet::new();
        s.add("a".to_string(), 1, "n1".into());
        s.add("b".to_string(), 2, "n1".into());
        s.add("c".to_string(), 3, "n1".into());
        assert_eq!(s.len(), 3);
        let mut elems = s.elements();
        elems.sort();
        assert_eq!(
            elems,
            vec![&"a".to_string(), &"b".to_string(), &"c".to_string()]
        );
    }

    #[test]
    fn orset_remove() {
        let mut s = ORSet::new();
        s.add("x".to_string(), 1, "n1".into());
        assert!(s.contains(&"x".to_string()));
        s.remove(&"x".to_string());
        assert!(!s.contains(&"x".to_string()));
        assert!(s.is_empty());
    }

    #[test]
    fn orset_remove_nonexistent() {
        let mut s = ORSet::<String>::new();
        s.remove(&"ghost".to_string());
        assert!(s.is_empty());
    }

    #[test]
    fn orset_merge_disjoint() {
        let mut a = ORSet::new();
        a.add("x".to_string(), 1, "n1".into());
        let mut b = ORSet::new();
        b.add("y".to_string(), 2, "n2".into());
        a.merge(b);
        assert!(a.contains(&"x".to_string()));
        assert!(a.contains(&"y".to_string()));
        assert_eq!(a.len(), 2);
    }

    #[test]
    fn orset_merge_overlapping() {
        let mut a = ORSet::new();
        a.add("shared".to_string(), 1, "n1".into());
        let mut b = ORSet::new();
        b.add("shared".to_string(), 2, "n2".into());
        a.merge(b);
        // "shared" should have two tags (from n1 and n2)
        assert!(a.contains(&"shared".to_string()));
        assert_eq!(a.len(), 1);
    }

    #[test]
    fn orset_merge_concurrent_add_remove() {
        // n1 adds "x", n2 removes "x" concurrently
        let mut a = ORSet::new();
        a.add("x".to_string(), 1, "n1".into());
        let mut b = ORSet::new();
        b.add("x".to_string(), 1, "n1".into()); // sees the add
        b.remove(&"x".to_string()); // then removes it

        a.merge(b);
        // In basic ORSet, if all tags are tombstoned, it's removed
        // The add tag from n1 was tombstoned by b's remove
        assert!(!a.contains(&"x".to_string()));
    }

    #[test]
    fn orset_merge_idempotency() {
        let mut a = ORSet::new();
        a.add("a".to_string(), 1, "n1".into());
        let original = a.clone();
        let b = a.clone();
        a.merge(b);
        assert_eq!(a, original);
    }

    #[test]
    fn orset_merge_commutativity() {
        let mut a = ORSet::new();
        a.add("x".to_string(), 1, "n1".into());
        let mut b = ORSet::new();
        b.add("y".to_string(), 2, "n2".into());

        let a1 = a.clone();
        let b1 = b.clone();
        let mut result1 = a1;
        result1.merge(b1);

        let mut result2 = b;
        result2.merge(a);

        assert_eq!(result1.len(), result2.len());
        let mut e1 = result1.elements();
        let mut e2 = result2.elements();
        e1.sort();
        e2.sort();
        assert_eq!(e1, e2);
    }

    #[test]
    fn orset_remove_then_add_back() {
        let mut s = ORSet::new();
        s.add("x".to_string(), 1, "n1".into());
        s.remove(&"x".to_string());
        assert!(!s.contains(&"x".to_string()));
        // Re-add with new timestamp
        s.add("x".to_string(), 5, "n1".into());
        assert!(s.contains(&"x".to_string()));
    }

    // ===== RGA Tests =====

    #[test]
    fn rga_new_is_empty() {
        let rga = RGA::<String>::new();
        assert!(rga.is_empty());
        assert_eq!(rga.to_vec().len(), 0);
    }

    #[test]
    fn rga_append_and_to_vec() {
        let mut rga = RGA::new();
        rga.append("a".to_string(), 1, "n1".into());
        rga.append("b".to_string(), 2, "n1".into());
        rga.append("c".to_string(), 3, "n1".into());
        assert_eq!(
            rga.to_vec(),
            vec![&"a".to_string(), &"b".to_string(), &"c".to_string()]
        );
        assert_eq!(rga.len(), 3);
    }

    #[test]
    fn rga_insert_after() {
        let mut rga = RGA::new();
        rga.append("a".to_string(), 1, "n1".into());
        rga.append("c".to_string(), 3, "n1".into());
        rga.insert("b".to_string(), 1, 2, "n1".into()); // insert after "a" (timestamp 1)
        assert_eq!(
            rga.to_vec(),
            vec![&"a".to_string(), &"b".to_string(), &"c".to_string()]
        );
    }

    #[test]
    fn rga_insert_at_beginning() {
        let mut rga = RGA::new();
        rga.append("b".to_string(), 2, "n1".into());
        rga.insert("a".to_string(), 0, 1, "n1".into()); // after_timestamp=0 => insert at start
        assert_eq!(rga.to_vec(), vec![&"a".to_string(), &"b".to_string()]);
    }

    #[test]
    fn rga_delete() {
        let mut rga = RGA::new();
        rga.append("a".to_string(), 1, "n1".into());
        rga.append("b".to_string(), 2, "n1".into());
        rga.append("c".to_string(), 3, "n1".into());
        rga.delete(2);
        assert_eq!(rga.to_vec(), vec![&"a".to_string(), &"c".to_string()]);
        assert_eq!(rga.len(), 2);
    }

    #[test]
    fn rga_delete_nonexistent() {
        let mut rga = RGA::new();
        rga.append("a".to_string(), 1, "n1".into());
        rga.delete(999);
        assert_eq!(rga.to_vec(), vec![&"a".to_string()]);
    }

    #[test]
    fn rga_merge_disjoint() {
        let mut a = RGA::new();
        a.append("a".to_string(), 1, "n1".into());
        let mut b = RGA::new();
        b.append("b".to_string(), 2, "n2".into());
        a.merge(b);
        let result = a.to_vec();
        assert!(result.contains(&&"a".to_string()));
        assert!(result.contains(&&"b".to_string()));
    }

    #[test]
    fn rga_merge_overlapping() {
        let mut a = RGA::new();
        a.append("shared".to_string(), 1, "n1".into());
        let mut b = RGA::new();
        b.append("shared".to_string(), 1, "n1".into());
        a.merge(b);
        // Same timestamp => deduplicated
        assert_eq!(a.to_vec(), vec![&"shared".to_string()]);
    }

    #[test]
    fn rga_merge_preserves_order_by_timestamp() {
        let mut a = RGA::new();
        a.append("c".to_string(), 3, "n1".into());
        let mut b = RGA::new();
        b.append("a".to_string(), 1, "n2".into());
        b.append("b".to_string(), 2, "n2".into());
        a.merge(b);
        assert_eq!(
            a.to_vec(),
            vec![&"a".to_string(), &"b".to_string(), &"c".to_string()]
        );
    }

    #[test]
    fn rga_merge_delete_propagation() {
        let mut a = RGA::new();
        a.append("x".to_string(), 1, "n1".into());
        a.append("y".to_string(), 2, "n1".into());
        a.delete(1);

        let mut b = RGA::new();
        b.append("x".to_string(), 1, "n1".into());
        b.append("y".to_string(), 2, "n1".into());

        a.merge(b);
        // "x" was deleted in a, "y" was not => only "y" visible
        assert_eq!(a.to_vec(), vec![&"y".to_string()]);
    }

    #[test]
    fn rga_concurrent_inserts() {
        let mut a = RGA::new();
        a.append("a".to_string(), 1, "n1".into());
        a.append("c".to_string(), 3, "n1".into());

        let mut b = RGA::new();
        b.append("a".to_string(), 1, "n1".into());
        b.append("b".to_string(), 2, "n2".into());
        b.append("c".to_string(), 3, "n1".into());

        a.merge(b);
        let result = a.to_vec();
        assert_eq!(result.len(), 3);
        assert!(result.contains(&&"a".to_string()));
        assert!(result.contains(&&"b".to_string()));
        assert!(result.contains(&&"c".to_string()));
    }

    // ===== AddWinsSet Tests =====

    #[test]
    fn add_wins_set_new_is_empty() {
        let s = AddWinsSet::<String>::new();
        assert!(s.is_empty());
    }

    #[test]
    fn add_wins_set_add_and_contains() {
        let mut s = AddWinsSet::new();
        s.add("item".to_string(), 1, "n1".into());
        assert!(s.contains(&"item".to_string()));
    }

    #[test]
    fn add_wins_set_remove() {
        let mut s = AddWinsSet::new();
        s.add("item".to_string(), 1, "n1".into());
        s.remove("item".to_string(), 2, "n1".into());
        assert!(!s.contains(&"item".to_string()));
    }

    #[test]
    fn add_wins_set_concurrent_add_remove() {
        // n1 adds at time 1, n2 removes at time 1 (concurrent)
        // Both happen concurrently => add wins
        let mut s = AddWinsSet::new();
        s.add("item".to_string(), 1, "n1".into());
        s.remove("item".to_string(), 1, "n2".into());
        // Add tag (1, n1) is NOT in remove set (which has (1, n2)) => add wins
        assert!(s.contains(&"item".to_string()));
    }

    #[test]
    fn add_wins_set_sequential_remove_wins() {
        let mut s = AddWinsSet::new();
        s.add("item".to_string(), 1, "n1".into());
        // Remove with same tag as add => not concurrent
        s.remove("item".to_string(), 1, "n1".into());
        assert!(!s.contains(&"item".to_string()));
    }

    #[test]
    fn add_wins_set_merge() {
        let mut a = AddWinsSet::new();
        a.add("x".to_string(), 1, "n1".into());
        let mut b = AddWinsSet::new();
        b.add("y".to_string(), 2, "n2".into());
        a.merge(b);
        assert!(a.contains(&"x".to_string()));
        assert!(a.contains(&"y".to_string()));
    }

    #[test]
    fn add_wins_set_merge_concurrent_add_remove() {
        // a has add(x, 1, n1)
        // b has add(x, 1, n1) and remove(x, 1, n2) concurrently
        let mut a = AddWinsSet::new();
        a.add("x".to_string(), 1, "n1".into());

        let mut b = AddWinsSet::new();
        b.add("x".to_string(), 1, "n1".into());
        b.remove("x".to_string(), 1, "n2".into());

        a.merge(b);
        // After merge: add tags = {(1,n1)}, remove tags = {(1,n2)}
        // Add wins because (1,n1) not in remove set
        assert!(a.contains(&"x".to_string()));
    }

    #[test]
    fn add_wins_set_merge_same_tag_remove() {
        let mut a = AddWinsSet::new();
        a.add("x".to_string(), 1, "n1".into());

        let mut b = AddWinsSet::new();
        b.add("x".to_string(), 1, "n1".into());
        b.remove("x".to_string(), 1, "n1".into()); // same tag removed

        a.merge(b);
        // add tags = {(1,n1)}, remove tags = {(1,n1)} => all adds covered => removed
        assert!(!a.contains(&"x".to_string()));
    }

    #[test]
    fn add_wins_set_elements() {
        let mut s = AddWinsSet::new();
        s.add("a".to_string(), 1, "n1".into());
        s.add("b".to_string(), 2, "n1".into());
        s.add("c".to_string(), 3, "n1".into());
        s.remove("b".to_string(), 4, "n1".into());
        let mut elems = s.elements();
        elems.sort();
        assert_eq!(elems, vec![&"a".to_string(), &"c".to_string()]);
    }

    #[test]
    fn add_wins_set_merge_idempotency() {
        let mut a = AddWinsSet::new();
        a.add("x".to_string(), 1, "n1".into());
        let original = a.clone();
        a.merge(AddWinsSet::new());
        assert_eq!(a, original);
    }

    // ===== CausalContext Tests =====

    #[test]
    fn causal_context_new() {
        let ctx = CausalContext::new();
        assert!(ctx.clock().is_empty());
    }

    #[test]
    fn causal_context_happens_before() {
        let ctx = CausalContext::new();
        let mut a = VectorClock::new();
        a.increment(&"n1".to_string());
        let mut b = VectorClock::new();
        b.increment(&"n1".to_string());
        b.increment(&"n2".to_string());
        assert!(ctx.happens_before(&a, &b));
        assert!(!ctx.happens_before(&b, &a));
    }

    #[test]
    fn causal_context_concurrent() {
        let ctx = CausalContext::new();
        let mut a = VectorClock::new();
        a.increment(&"n1".to_string());
        a.increment(&"n2".to_string());
        let mut b = VectorClock::new();
        b.increment(&"n1".to_string());
        b.increment(&"n3".to_string());
        assert!(ctx.concurrent(&a, &b));
    }

    #[test]
    fn causal_context_not_concurrent() {
        let ctx = CausalContext::new();
        let mut a = VectorClock::new();
        a.increment(&"n1".to_string());
        let mut b = VectorClock::new();
        b.increment(&"n1".to_string());
        b.increment(&"n2".to_string());
        assert!(!ctx.concurrent(&a, &b));
    }

    #[test]
    fn causal_context_merge_clocks() {
        let ctx = CausalContext::new();
        let mut a = VectorClock::new();
        a.increment(&"n1".to_string());
        a.increment(&"n1".to_string());
        let mut b = VectorClock::new();
        b.increment(&"n2".to_string());
        let merged = ctx.merge_clocks(&a, &b);
        assert_eq!(merged.get(&"n1".to_string()), 2);
        assert_eq!(merged.get(&"n2".to_string()), 1);
    }

    #[test]
    fn causal_context_increment() {
        let mut ctx = CausalContext::new();
        ctx.increment("node-a");
        ctx.increment("node-a");
        ctx.increment("node-b");
        assert_eq!(ctx.clock().get(&"node-a".to_string()), 2);
        assert_eq!(ctx.clock().get(&"node-b".to_string()), 1);
    }

    #[test]
    fn causal_context_merge_clocks_idempotent() {
        let ctx = CausalContext::new();
        let mut a = VectorClock::new();
        a.increment(&"n1".to_string());
        let merged = ctx.merge_clocks(&a, &a);
        assert_eq!(merged.get(&"n1".to_string()), 1);
    }

    // ===== Serialization Tests =====

    #[test]
    fn lww_serialization_roundtrip() {
        let reg = LWWRegister::new(vec![1, 2, 3], 42, "node-x".into());
        let json = serde_json::to_string(&reg).unwrap();
        let deser: LWWRegister<Vec<i32>> = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.value, vec![1, 2, 3]);
        assert_eq!(deser.timestamp, 42);
    }

    #[test]
    fn orset_serialization_roundtrip() {
        let mut s = ORSet::new();
        s.add("hello".to_string(), 1, "n1".into());
        s.add("world".to_string(), 2, "n1".into());
        let json = serde_json::to_string(&s).unwrap();
        let deser: ORSet<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.len(), 2);
        assert!(deser.contains(&"hello".to_string()));
    }

    #[test]
    fn rga_serialization_roundtrip() {
        let mut rga = RGA::new();
        rga.append("x".to_string(), 1, "n1".into());
        rga.append("y".to_string(), 2, "n1".into());
        let json = serde_json::to_string(&rga).unwrap();
        let deser: RGA<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.to_vec(), vec![&"x".to_string(), &"y".to_string()]);
    }

    #[test]
    fn add_wins_set_serialization_roundtrip() {
        let mut s = AddWinsSet::new();
        s.add("a".to_string(), 1, "n1".into());
        let json = serde_json::to_string(&s).unwrap();
        let deser: AddWinsSet<String> = serde_json::from_str(&json).unwrap();
        assert!(deser.contains(&"a".to_string()));
    }

    #[test]
    fn causal_context_serialization_roundtrip() {
        let mut ctx = CausalContext::new();
        ctx.increment("n1");
        ctx.increment("n2");
        let json = serde_json::to_string(&ctx).unwrap();
        let deser: CausalContext = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.clock().get(&"n1".to_string()), 1);
    }
}
