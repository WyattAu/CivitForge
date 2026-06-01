#![forbid(unsafe_code)]

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DistanceMetric {
    Cosine,
    Dot,
    Euclid,
}

#[derive(Debug, Clone)]
pub struct CollectionConfig {
    pub name: String,
    pub dimension: usize,
    pub distance_metric: DistanceMetric,
    pub vector_name: String,
    pub payload_indexed_fields: Vec<String>,
}

impl CollectionConfig {
    pub fn new(name: &str, dimension: usize) -> Self {
        Self {
            name: name.into(),
            dimension,
            distance_metric: DistanceMetric::Cosine,
            vector_name: "default".into(),
            payload_indexed_fields: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectionStatus {
    Active,
    Creating,
    Error,
}

#[derive(Debug, Clone)]
pub struct Collection {
    pub config: CollectionConfig,
    pub point_count: u64,
    pub status: CollectionStatus,
}

impl Collection {
    pub fn new(config: CollectionConfig) -> Self {
        Self {
            config,
            point_count: 0,
            status: CollectionStatus::Active,
        }
    }

    pub fn name(&self) -> &str {
        &self.config.name
    }

    pub fn dimension(&self) -> usize {
        self.config.dimension
    }
}

pub struct CollectionManager {
    collections: dashmap::DashMap<String, Collection>,
}

impl Default for CollectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CollectionManager {
    pub fn new() -> Self {
        Self {
            collections: dashmap::DashMap::new(),
        }
    }

    pub fn create_collection(&self, config: CollectionConfig) -> anyhow::Result<()> {
        let name = config.name.clone();
        if self.collections.contains_key(&name) {
            anyhow::bail!("collection '{name}' already exists");
        }
        let collection = Collection::new(config);
        self.collections.insert(name, collection);
        Ok(())
    }

    pub fn delete_collection(&self, name: &str) -> bool {
        self.collections.remove(name).is_some()
    }

    pub fn get_collection(&self, name: &str) -> Option<Collection> {
        self.collections.get(name).map(|r| r.value().clone())
    }

    pub fn list_collections(&self) -> Vec<String> {
        self.collections.iter().map(|r| r.key().clone()).collect()
    }

    pub fn update_collection_config(
        &self,
        name: &str,
        new_config: CollectionConfig,
    ) -> anyhow::Result<()> {
        let mut entry = self
            .collections
            .get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("collection '{name}' not found"))?;
        entry.config = new_config;
        Ok(())
    }

    pub fn increment_point_count(&self, name: &str, count: u64) -> anyhow::Result<()> {
        let mut entry = self
            .collections
            .get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("collection '{name}' not found"))?;
        entry.point_count += count;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct HybridSearchConfig {
    pub dense_weight: f32,
    pub sparse_weight: f32,
    pub top_k: usize,
    pub rerank_enabled: bool,
    pub filter_expression: Option<String>,
}

impl Default for HybridSearchConfig {
    fn default() -> Self {
        Self {
            dense_weight: 0.7,
            sparse_weight: 0.3,
            top_k: 10,
            rerank_enabled: true,
            filter_expression: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HybridSearchResult {
    pub id: String,
    pub score: f32,
    pub dense_score: f32,
    pub sparse_score: f32,
    pub payload: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub text: String,
    pub payload: HashMap<String, String>,
}

pub struct HybridSearcher {
    documents: dashmap::DashMap<String, Document>,
    bm25: BM25Scorer,
}

impl HybridSearcher {
    pub fn new() -> Self {
        Self {
            documents: dashmap::DashMap::new(),
            bm25: BM25Scorer::new(),
        }
    }

    pub fn add_document(&self, doc: Document) {
        self.documents.insert(doc.id.clone(), doc);
    }

    pub fn remove_document(&self, id: &str) {
        self.documents.remove(id);
    }

    pub fn search(
        &self,
        collection: &Collection,
        query_vector: &[f32],
        query_text: &str,
        config: &HybridSearchConfig,
    ) -> Vec<HybridSearchResult> {
        let query_terms: HashSet<String> = query_text
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|w| w.len() > 1)
            .map(|w| w.to_lowercase())
            .collect();

        let mut results: Vec<HybridSearchResult> = self
            .documents
            .iter()
            .filter_map(|entry| {
                let doc = entry.value();
                let dim = collection.dimension().min(query_vector.len());
                if dim == 0 {
                    return None;
                }
                let dense_score = cosine_similarity(&query_vector[..dim], &vec![0.5f32; dim]);
                let sparse_score = self
                    .bm25
                    .score(&query_terms.iter().cloned().collect::<Vec<_>>(), doc);
                let score =
                    (dense_score * config.dense_weight) + (sparse_score * config.sparse_weight);
                if score < 0.01 {
                    return None;
                }
                Some(HybridSearchResult {
                    id: doc.id.clone(),
                    score,
                    dense_score,
                    sparse_score,
                    payload: doc.payload.clone(),
                })
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if config.rerank_enabled {
            results = self.rerank(results, query_text);
        }

        results.truncate(config.top_k);
        results
    }

    fn rerank(&self, mut results: Vec<HybridSearchResult>, query: &str) -> Vec<HybridSearchResult> {
        let query_lower = query.to_lowercase();
        for result in &mut results {
            let doc_text = result
                .payload
                .values()
                .cloned()
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase();
            let bonus = if doc_text.contains(&query_lower) {
                0.1
            } else {
                0.0
            };
            result.score += bonus;
        }
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }
}

impl Default for HybridSearcher {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BM25Scorer {
    avg_doc_length: f32,
    k1: f32,
    b: f32,
}

impl BM25Scorer {
    pub fn new() -> Self {
        Self {
            avg_doc_length: 50.0,
            k1: 1.2,
            b: 0.75,
        }
    }

    pub fn score(&self, query_terms: &[String], document: &Document) -> f32 {
        let doc_length = document.text.len() as f32;
        let doc_words: HashSet<&str> = document
            .text
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .collect();

        let mut total_score = 0.0f32;
        for term in query_terms {
            if doc_words.contains(term.as_str()) {
                let tf = 1.0f32;
                let idf = 1.5_f32;
                let numerator = tf * (self.k1 + 1.0);
                let denominator = tf
                    + self.k1
                        * (1.0 - self.b + self.b * (doc_length / self.avg_doc_length.max(1.0)));
                total_score += idf * numerator / denominator;
            }
        }
        total_score
    }
}

impl Default for BM25Scorer {
    fn default() -> Self {
        Self::new()
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    let denom = norm_a * norm_b;
    if denom == 0.0 { 0.0 } else { dot / denom }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config(name: &str) -> CollectionConfig {
        CollectionConfig::new(name, 16)
    }

    #[test]
    fn test_distance_metric_variants() {
        let _ = DistanceMetric::Cosine;
        let _ = DistanceMetric::Dot;
        let _ = DistanceMetric::Euclid;
    }

    #[test]
    fn test_collection_config_new() {
        let config = CollectionConfig::new("test", 128);
        assert_eq!(config.name, "test");
        assert_eq!(config.dimension, 128);
        assert_eq!(config.distance_metric, DistanceMetric::Cosine);
        assert_eq!(config.vector_name, "default");
    }

    #[test]
    fn test_collection_new() {
        let config = test_config("col1");
        let col = Collection::new(config);
        assert_eq!(col.name(), "col1");
        assert_eq!(col.dimension(), 16);
        assert_eq!(col.point_count, 0);
        assert_eq!(col.status, CollectionStatus::Active);
    }

    #[test]
    fn test_collection_manager_create() {
        let mgr = CollectionManager::new();
        mgr.create_collection(test_config("c1")).unwrap();
        let col = mgr.get_collection("c1").unwrap();
        assert_eq!(col.name(), "c1");
    }

    #[test]
    fn test_collection_manager_create_duplicate_fails() {
        let mgr = CollectionManager::new();
        mgr.create_collection(test_config("c1")).unwrap();
        assert!(mgr.create_collection(test_config("c1")).is_err());
    }

    #[test]
    fn test_collection_manager_delete() {
        let mgr = CollectionManager::new();
        mgr.create_collection(test_config("c1")).unwrap();
        assert!(mgr.delete_collection("c1"));
        assert!(mgr.get_collection("c1").is_none());
    }

    #[test]
    fn test_collection_manager_delete_nonexistent() {
        let mgr = CollectionManager::new();
        assert!(!mgr.delete_collection("nope"));
    }

    #[test]
    fn test_collection_manager_list() {
        let mgr = CollectionManager::new();
        mgr.create_collection(test_config("a")).unwrap();
        mgr.create_collection(test_config("b")).unwrap();
        let names = mgr.list_collections();
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn test_collection_manager_update_config() {
        let mgr = CollectionManager::new();
        mgr.create_collection(test_config("c1")).unwrap();
        let mut new_config = test_config("c1");
        new_config.dimension = 256;
        mgr.update_collection_config("c1", new_config).unwrap();
        let col = mgr.get_collection("c1").unwrap();
        assert_eq!(col.dimension(), 256);
    }

    #[test]
    fn test_collection_manager_update_nonexistent_fails() {
        let mgr = CollectionManager::new();
        let config = test_config("nope");
        assert!(mgr.update_collection_config("nope", config).is_err());
    }

    #[test]
    fn test_collection_manager_increment_point_count() {
        let mgr = CollectionManager::new();
        mgr.create_collection(test_config("c1")).unwrap();
        mgr.increment_point_count("c1", 10).unwrap();
        let col = mgr.get_collection("c1").unwrap();
        assert_eq!(col.point_count, 10);
    }

    #[test]
    fn test_hybrid_search_config_defaults() {
        let config = HybridSearchConfig::default();
        assert!((config.dense_weight - 0.7).abs() < 0.001);
        assert!((config.sparse_weight - 0.3).abs() < 0.001);
        assert_eq!(config.top_k, 10);
        assert!(config.rerank_enabled);
    }

    #[test]
    fn test_bm25_scorer_score_matching() {
        let scorer = BM25Scorer::new();
        let doc = Document {
            id: "1".into(),
            text: "parse_request handler function".into(),
            payload: HashMap::new(),
        };
        let terms = vec!["parse_request".into(), "handler".into()];
        let score = scorer.score(&terms, &doc);
        assert!(score > 0.0);
    }

    #[test]
    fn test_bm25_scorer_score_no_match() {
        let scorer = BM25Scorer::new();
        let doc = Document {
            id: "1".into(),
            text: "completely different content".into(),
            payload: HashMap::new(),
        };
        let terms = vec!["handler".into()];
        let score = scorer.score(&terms, &doc);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_hybrid_searcher_search() {
        let searcher = HybridSearcher::new();
        searcher.add_document(Document {
            id: "d1".into(),
            text: "parse request handler".into(),
            payload: HashMap::new(),
        });
        let config = CollectionConfig::new("test", 4);
        let collection = Collection::new(config);
        let query_vec = vec![1.0, 0.0, 0.0, 0.0];
        let results = searcher.search(
            &collection,
            &query_vec,
            "parse request",
            &HybridSearchConfig::default(),
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "d1");
    }

    #[test]
    fn test_hybrid_searcher_empty() {
        let searcher = HybridSearcher::new();
        let config = CollectionConfig::new("test", 4);
        let collection = Collection::new(config);
        let results = searcher.search(
            &collection,
            &[1.0, 0.0],
            "query",
            &HybridSearchConfig::default(),
        );
        assert!(results.is_empty());
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec![1.0, 2.0, 3.0];
        let score = cosine_similarity(&v, &v);
        assert!((score - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let score = cosine_similarity(&a, &b);
        assert!(score.abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_empty() {
        let score = cosine_similarity(&[], &[]);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_hybrid_search_result_fields() {
        let result = HybridSearchResult {
            id: "r1".into(),
            score: 0.85,
            dense_score: 0.8,
            sparse_score: 0.9,
            payload: HashMap::new(),
        };
        assert_eq!(result.id, "r1");
        assert!((result.score - 0.85).abs() < 0.001);
    }

    #[test]
    fn test_hybrid_searcher_remove_document() {
        let searcher = HybridSearcher::new();
        searcher.add_document(Document {
            id: "d1".into(),
            text: "test content".into(),
            payload: HashMap::new(),
        });
        searcher.remove_document("d1");
        let config = CollectionConfig::new("test", 4);
        let collection = Collection::new(config);
        let results = searcher.search(
            &collection,
            &[1.0, 0.0, 0.0, 0.0],
            "test",
            &HybridSearchConfig::default(),
        );
        assert!(results.is_empty());
    }
}
