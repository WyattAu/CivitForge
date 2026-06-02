#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: Option<serde_json::Value>,
}

impl Embedding {
    pub fn dimensions(&self) -> usize {
        self.vector.len()
    }

    pub fn cosine_similarity(&self, other: &Embedding) -> f64 {
        if self.vector.len() != other.vector.len() {
            return 0.0;
        }
        let dot: f64 = self
            .vector
            .iter()
            .zip(other.vector.iter())
            .map(|(a, b)| (*a as f64) * (*b as f64))
            .sum();
        let mag_a: f64 = (self
            .vector
            .iter()
            .map(|v| (*v as f64) * (*v as f64))
            .sum::<f64>())
        .sqrt();
        let mag_b: f64 = (other
            .vector
            .iter()
            .map(|v| (*v as f64) * (*v as f64))
            .sum::<f64>())
        .sqrt();
        if mag_a == 0.0 || mag_b == 0.0 {
            return 0.0;
        }
        dot / (mag_a * mag_b)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f64,
    pub metadata: Option<serde_json::Value>,
}

pub trait VectorDatabase: Send + Sync {
    fn insert(&self, embedding: &Embedding) -> Result<(), String>;
    fn search(&self, query: &Embedding, top_k: usize) -> Result<Vec<SearchResult>, String>;
    fn delete(&self, id: &str) -> Result<(), String>;
    fn count(&self) -> usize;
}

pub struct InMemoryVectorDb {
    entries: std::sync::Mutex<Vec<Embedding>>,
}

impl InMemoryVectorDb {
    pub fn new() -> Self {
        Self {
            entries: std::sync::Mutex::new(Vec::new()),
        }
    }
}

impl Default for InMemoryVectorDb {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorDatabase for InMemoryVectorDb {
    fn insert(&self, embedding: &Embedding) -> Result<(), String> {
        let mut entries = self.entries.lock().map_err(|e| e.to_string())?;
        entries.push(embedding.clone());
        Ok(())
    }

    fn search(&self, query: &Embedding, top_k: usize) -> Result<Vec<SearchResult>, String> {
        let entries = self.entries.lock().map_err(|e| e.to_string())?;
        let mut results: Vec<(f64, &str, Option<serde_json::Value>)> = Vec::new();
        for entry in entries.iter() {
            let score = query.cosine_similarity(entry);
            results.push((score, &entry.id, entry.metadata.clone()));
        }
        results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        Ok(results
            .into_iter()
            .rev()
            .take(top_k)
            .map(|(s, id, meta)| SearchResult {
                id: id.to_string(),
                score: s,
                metadata: meta,
            })
            .collect())
    }

    fn delete(&self, id: &str) -> Result<(), String> {
        let mut entries = self.entries.lock().map_err(|e| e.to_string())?;
        let before = entries.len();
        entries.retain(|e| e.id != id);
        if entries.len() < before {
            Ok(())
        } else {
            Err(format!("embedding not found: {id}"))
        }
    }

    fn count(&self) -> usize {
        self.entries.lock().map_or(0, |e| e.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_embedding(id: &str, vector: Vec<f32>) -> Embedding {
        Embedding {
            id: id.into(),
            vector,
            metadata: None,
        }
    }

    fn make_embedding_with_meta(id: &str, vector: Vec<f32>, meta: serde_json::Value) -> Embedding {
        Embedding {
            id: id.into(),
            vector,
            metadata: Some(meta),
        }
    }

    #[test]
    fn test_embedding_creation() {
        let emb = make_embedding("e1", vec![1.0, 0.0, 0.0]);
        assert_eq!(emb.id, "e1");
        assert_eq!(emb.vector.len(), 3);
        assert!(emb.metadata.is_none());
    }

    #[test]
    fn test_embedding_dimensions() {
        let emb = make_embedding("e1", vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(emb.dimensions(), 4);
        let empty = make_embedding("e0", vec![]);
        assert_eq!(empty.dimensions(), 0);
    }

    #[test]
    fn test_cosine_similarity_identical_vectors() {
        let a = make_embedding("a", vec![1.0, 2.0, 3.0]);
        let b = make_embedding("b", vec![1.0, 2.0, 3.0]);
        let sim = a.cosine_similarity(&b);
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal_vectors() {
        let a = make_embedding("a", vec![1.0, 0.0]);
        let b = make_embedding("b", vec![0.0, 1.0]);
        let sim = a.cosine_similarity(&b);
        assert!(sim.abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_zero_vector() {
        let a = make_embedding("a", vec![0.0, 0.0]);
        let b = make_embedding("b", vec![1.0, 2.0]);
        assert_eq!(a.cosine_similarity(&b), 0.0);
        assert_eq!(b.cosine_similarity(&a), 0.0);
    }

    #[test]
    fn test_cosine_similarity_different_sizes() {
        let a = make_embedding("a", vec![1.0, 2.0]);
        let b = make_embedding("b", vec![1.0]);
        assert_eq!(a.cosine_similarity(&b), 0.0);
    }

    #[test]
    fn test_cosine_similarity_negative_correlation() {
        let a = make_embedding("a", vec![1.0, 0.0]);
        let b = make_embedding("b", vec![-1.0, 0.0]);
        let sim = a.cosine_similarity(&b);
        assert!((sim - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_search_result_creation() {
        let sr = SearchResult {
            id: "r1".into(),
            score: 0.95,
            metadata: Some(serde_json::json!({"key": "val"})),
        };
        assert_eq!(sr.id, "r1");
        assert!((sr.score - 0.95).abs() < 0.001);
    }

    #[test]
    fn test_in_memory_db_new() {
        let db = InMemoryVectorDb::new();
        assert_eq!(db.count(), 0);
    }

    #[test]
    fn test_in_memory_db_default() {
        let db = InMemoryVectorDb::default();
        assert_eq!(db.count(), 0);
    }

    #[test]
    fn test_in_memory_db_insert_and_count() {
        let db = InMemoryVectorDb::new();
        let emb = make_embedding("e1", vec![1.0, 0.0, 0.0]);
        db.insert(&emb).unwrap();
        assert_eq!(db.count(), 1);
    }

    #[test]
    fn test_in_memory_db_insert_multiple() {
        let db = InMemoryVectorDb::new();
        db.insert(&make_embedding("a", vec![1.0, 0.0])).unwrap();
        db.insert(&make_embedding("b", vec![0.0, 1.0])).unwrap();
        db.insert(&make_embedding("c", vec![1.0, 1.0])).unwrap();
        assert_eq!(db.count(), 3);
    }

    #[test]
    fn test_in_memory_db_search_single() {
        let db = InMemoryVectorDb::new();
        db.insert(&make_embedding("e1", vec![1.0, 0.0, 0.0]))
            .unwrap();
        let query = make_embedding("q", vec![1.0, 0.0, 0.0]);
        let results = db.search(&query, 5).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "e1");
        assert!((results[0].score - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_in_memory_db_search_top_k() {
        let db = InMemoryVectorDb::new();
        db.insert(&make_embedding("a", vec![1.0, 0.0])).unwrap();
        db.insert(&make_embedding("b", vec![0.9, 0.1])).unwrap();
        db.insert(&make_embedding("c", vec![0.0, 1.0])).unwrap();
        let query = make_embedding("q", vec![1.0, 0.0]);
        let results = db.search(&query, 2).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "a");
        assert!(results[0].score > results[1].score);
    }

    #[test]
    fn test_in_memory_db_search_empty() {
        let db = InMemoryVectorDb::new();
        let query = make_embedding("q", vec![1.0, 0.0]);
        let results = db.search(&query, 5).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_in_memory_db_delete_existing() {
        let db = InMemoryVectorDb::new();
        db.insert(&make_embedding("a", vec![1.0])).unwrap();
        db.insert(&make_embedding("b", vec![2.0])).unwrap();
        db.delete("a").unwrap();
        assert_eq!(db.count(), 1);
    }

    #[test]
    fn test_in_memory_db_delete_nonexistent() {
        let db = InMemoryVectorDb::new();
        let result = db.delete("nope");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("nope"));
    }

    #[test]
    fn test_in_memory_db_count_after_operations() {
        let db = InMemoryVectorDb::new();
        assert_eq!(db.count(), 0);
        db.insert(&make_embedding("a", vec![1.0])).unwrap();
        assert_eq!(db.count(), 1);
        db.delete("a").unwrap();
        assert_eq!(db.count(), 0);
    }

    #[test]
    fn test_in_memory_db_search_with_metadata() {
        let db = InMemoryVectorDb::new();
        db.insert(&make_embedding_with_meta(
            "e1",
            vec![1.0, 0.0],
            serde_json::json!({"source": "test"}),
        ))
        .unwrap();
        let query = make_embedding("q", vec![1.0, 0.0]);
        let results = db.search(&query, 1).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].metadata.as_ref().unwrap()["source"], "test");
    }

    #[test]
    fn test_embedding_serialization() {
        let emb =
            make_embedding_with_meta("e1", vec![0.5, 0.25], serde_json::json!({"key": "value"}));
        let json = serde_json::to_string(&emb).unwrap();
        let de: Embedding = serde_json::from_str(&json).unwrap();
        assert_eq!(de.id, "e1");
        assert_eq!(de.vector.len(), 2);
        assert_eq!(de.metadata.as_ref().unwrap()["key"], "value");
    }

    #[test]
    fn test_search_result_serialization() {
        let sr = SearchResult {
            id: "r1".into(),
            score: 0.87,
            metadata: None,
        };
        let json = serde_json::to_string(&sr).unwrap();
        let de: SearchResult = serde_json::from_str(&json).unwrap();
        assert_eq!(de.id, "r1");
        assert!((de.score - 0.87).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_opposite_vectors() {
        let a = make_embedding("a", vec![3.0, 4.0]);
        let b = make_embedding("b", vec![-3.0, -4.0]);
        let sim = a.cosine_similarity(&b);
        assert!((sim - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_insert_overwrites_on_same_id() {
        let db = InMemoryVectorDb::new();
        db.insert(&make_embedding("a", vec![1.0])).unwrap();
        db.insert(&make_embedding("a", vec![2.0])).unwrap();
        assert_eq!(db.count(), 2);
    }
}
