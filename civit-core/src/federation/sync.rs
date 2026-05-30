#![forbid(unsafe_code)]

use crate::error::{CoreError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncNode {
    pub instance_id: String,
    pub domain: String,
    pub last_sync: Option<String>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEdge {
    pub from: String,
    pub to: String,
    pub weight: u32,
    pub protocol: SyncProtocol,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncProtocol {
    ActivityPub,
    GitPushMirror,
    DagSync,
}

#[derive(Debug, Clone)]
pub struct DagSyncEngine {
    nodes: HashMap<String, SyncNode>,
    edges: Vec<SyncEdge>,
    instance_id: String,
}

impl DagSyncEngine {
    pub fn new(instance_id: String) -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            instance_id,
        }
    }

    pub fn register_node(&mut self, node: SyncNode) {
        info!(instance = %node.instance_id, "registered sync node");
        self.nodes.insert(node.instance_id.clone(), node);
    }

    pub fn add_edge(&mut self, edge: SyncEdge) {
        debug!(from = %edge.from, to = %edge.to, "added sync edge");
        self.edges.push(edge);
    }

    pub fn remove_node(&mut self, instance_id: &str) {
        self.nodes.remove(instance_id);
        self.edges
            .retain(|e| e.from != instance_id && e.to != instance_id);
    }

    pub fn get_neighbors(&self, instance_id: &str) -> Vec<&SyncNode> {
        let neighbor_ids: HashSet<&str> = self
            .edges
            .iter()
            .filter(|e| e.from == instance_id)
            .map(|e| e.to.as_str())
            .collect();
        neighbor_ids
            .iter()
            .filter_map(|id| self.nodes.get(*id))
            .collect()
    }

    pub fn find_sync_path(&self, target: &str) -> Result<Vec<String>> {
        if !self.nodes.contains_key(target) {
            return Err(CoreError::Federation(format!(
                "target node not found: {target}"
            )));
        }
        if !self.nodes.contains_key(&self.instance_id) {
            return Err(CoreError::Federation("self node not registered".into()));
        }

        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<(String, Vec<String>)> = VecDeque::new();
        queue.push_back((self.instance_id.clone(), vec![self.instance_id.clone()]));
        visited.insert(self.instance_id.clone());

        while let Some((current, path)) = queue.pop_front() {
            if current == target {
                return Ok(path);
            }
            let neighbors: Vec<&str> = self
                .edges
                .iter()
                .filter(|e| e.from == current)
                .map(|e| e.to.as_str())
                .collect();
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    visited.insert(neighbor.to_string());
                    let mut new_path = path.clone();
                    new_path.push(neighbor.to_string());
                    queue.push_back((neighbor.to_string(), new_path));
                }
            }
        }

        Err(CoreError::Federation(format!("no sync path to {target}")))
    }

    pub fn has_cycle(&self) -> bool {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        let adj: HashMap<&str, Vec<&str>> = {
            let mut map: HashMap<&str, Vec<&str>> = HashMap::new();
            for edge in &self.edges {
                map.entry(edge.from.as_str())
                    .or_default()
                    .push(edge.to.as_str());
            }
            map
        };

        for node in self.nodes.keys() {
            if self.dfs_cycle(node, &adj, &mut visited, &mut stack) {
                return true;
            }
        }
        false
    }

    fn dfs_cycle(
        &self,
        node: &str,
        adj: &HashMap<&str, Vec<&str>>,
        visited: &mut HashSet<String>,
        stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        stack.insert(node.to_string());

        if let Some(neighbors) = adj.get(node) {
            for neighbor in neighbors {
                if !visited.contains(*neighbor) {
                    if self.dfs_cycle(neighbor, adj, visited, stack) {
                        return true;
                    }
                } else if stack.contains(*neighbor) {
                    return true;
                }
            }
        }

        stack.remove(node);
        false
    }

    pub fn topology_summary(&self) -> (usize, usize) {
        (self.nodes.len(), self.edges.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine() -> DagSyncEngine {
        DagSyncEngine::new("inst-a".into())
    }

    fn make_nodes() -> (SyncNode, SyncNode, SyncNode) {
        (
            SyncNode {
                instance_id: "inst-a".into(),
                domain: "a.forge".into(),
                last_sync: None,
                capabilities: vec!["git".into()],
            },
            SyncNode {
                instance_id: "inst-b".into(),
                domain: "b.forge".into(),
                last_sync: None,
                capabilities: vec!["git".into()],
            },
            SyncNode {
                instance_id: "inst-c".into(),
                domain: "c.forge".into(),
                last_sync: None,
                capabilities: vec!["git".into()],
            },
        )
    }

    #[test]
    fn test_register_and_get_neighbors() {
        let mut engine = make_engine();
        let (a, b, c) = make_nodes();
        engine.register_node(a);
        engine.register_node(b);
        engine.register_node(c);
        engine.add_edge(SyncEdge {
            from: "inst-a".into(),
            to: "inst-b".into(),
            weight: 1,
            protocol: SyncProtocol::DagSync,
        });
        engine.add_edge(SyncEdge {
            from: "inst-a".into(),
            to: "inst-c".into(),
            weight: 2,
            protocol: SyncProtocol::ActivityPub,
        });

        let neighbors = engine.get_neighbors("inst-a");
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn test_find_sync_path() {
        let mut engine = make_engine();
        let (a, b, c) = make_nodes();
        engine.register_node(a);
        engine.register_node(b);
        engine.register_node(c);
        engine.add_edge(SyncEdge {
            from: "inst-a".into(),
            to: "inst-b".into(),
            weight: 1,
            protocol: SyncProtocol::DagSync,
        });
        engine.add_edge(SyncEdge {
            from: "inst-b".into(),
            to: "inst-c".into(),
            weight: 1,
            protocol: SyncProtocol::DagSync,
        });

        let path = engine.find_sync_path("inst-c").unwrap();
        assert_eq!(path, vec!["inst-a", "inst-b", "inst-c"]);
    }

    #[test]
    fn test_find_sync_path_no_route() {
        let mut engine = make_engine();
        let (a, b, _) = make_nodes();
        engine.register_node(a);
        engine.register_node(b);
        assert!(engine.find_sync_path("inst-b").is_err());
    }

    #[test]
    fn test_no_cycle_in_dag() {
        let mut engine = make_engine();
        let (a, b, c) = make_nodes();
        engine.register_node(a);
        engine.register_node(b);
        engine.register_node(c);
        engine.add_edge(SyncEdge {
            from: "inst-a".into(),
            to: "inst-b".into(),
            weight: 1,
            protocol: SyncProtocol::DagSync,
        });
        engine.add_edge(SyncEdge {
            from: "inst-b".into(),
            to: "inst-c".into(),
            weight: 1,
            protocol: SyncProtocol::DagSync,
        });
        assert!(!engine.has_cycle());
    }

    #[test]
    fn test_detect_cycle() {
        let mut engine = make_engine();
        let (a, b, c) = make_nodes();
        engine.register_node(a);
        engine.register_node(b);
        engine.register_node(c);
        engine.add_edge(SyncEdge {
            from: "inst-a".into(),
            to: "inst-b".into(),
            weight: 1,
            protocol: SyncProtocol::DagSync,
        });
        engine.add_edge(SyncEdge {
            from: "inst-b".into(),
            to: "inst-c".into(),
            weight: 1,
            protocol: SyncProtocol::DagSync,
        });
        engine.add_edge(SyncEdge {
            from: "inst-c".into(),
            to: "inst-a".into(),
            weight: 1,
            protocol: SyncProtocol::DagSync,
        });
        assert!(engine.has_cycle());
    }

    #[test]
    fn test_remove_node() {
        let mut engine = make_engine();
        let (a, b, _c) = make_nodes();
        engine.register_node(a);
        engine.register_node(b);
        engine.add_edge(SyncEdge {
            from: "inst-a".into(),
            to: "inst-b".into(),
            weight: 1,
            protocol: SyncProtocol::DagSync,
        });
        engine.remove_node("inst-b");
        assert_eq!(engine.topology_summary(), (1, 0));
    }
}
