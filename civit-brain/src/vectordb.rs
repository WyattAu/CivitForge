#![forbid(unsafe_code)]

use crate::embedding::{EmbeddingVector, EmbeddingWorker};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct VectorDbConfig {
    pub collection_name: String,
    pub dimension: usize,
    pub distance_metric: DistanceMetric,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    Dot,
}

#[derive(Debug, Clone)]
pub struct VectorDbClient {
    config: VectorDbConfig,
    store: DashMap<String, (EmbeddingVector, serde_json::Value)>,
}

impl VectorDbClient {
    pub fn new(config: VectorDbConfig) -> Self {
        Self {
            config,
            store: DashMap::new(),
        }
    }

    pub fn upsert(
        &self,
        vector: &EmbeddingVector,
        metadata: serde_json::Value,
    ) -> anyhow::Result<()> {
        self.store
            .insert(vector.id.clone(), (vector.clone(), metadata));
        debug!(id = %vector.id, "upserted vector");
        Ok(())
    }

    pub fn upsert_batch(
        &self,
        vectors: &[(EmbeddingVector, serde_json::Value)],
    ) -> anyhow::Result<usize> {
        let mut count = 0usize;
        for (vector, metadata) in vectors {
            self.store
                .insert(vector.id.clone(), (vector.clone(), metadata.clone()));
            count += 1;
        }
        debug!(count = count, "upserted batch");
        Ok(count)
    }

    pub fn search(&self, query: &[f32], top_k: usize) -> Vec<VectorSearchResult> {
        let _query_norm = match self.config.distance_metric {
            DistanceMetric::Cosine => {
                let norm: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
                if norm > 0.0 {
                    query.iter().map(|x| x / norm).collect()
                } else {
                    query.to_vec()
                }
            }
            _ => query.to_vec(),
        };

        let mut results: Vec<VectorSearchResult> = self
            .store
            .iter()
            .filter_map(|entry| {
                let (vector, metadata) = entry.value();
                if vector.data.len() != query.len() {
                    return None;
                }
                let score = match self.config.distance_metric {
                    DistanceMetric::Cosine => {
                        EmbeddingWorker::cosine_similarity(query, &vector.data)
                    }
                    DistanceMetric::Euclidean => {
                        let dist = EmbeddingWorker::euclidean_distance(query, &vector.data);
                        1.0 / (1.0 + dist)
                    }
                    DistanceMetric::Dot => query
                        .iter()
                        .zip(vector.data.iter())
                        .map(|(a, b)| a * b)
                        .sum(),
                };
                Some(VectorSearchResult {
                    id: vector.id.clone(),
                    score,
                    metadata: metadata.clone(),
                })
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(top_k);

        debug!(top_k = results.len(), "search complete");
        results
    }

    pub fn delete(&self, id: &str) -> bool {
        let removed = self.store.remove(id);
        removed.is_some()
    }

    pub fn count(&self) -> usize {
        self.store.len()
    }

    pub fn clear(&self) {
        self.store.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_client() -> VectorDbClient {
        VectorDbClient::new(VectorDbConfig {
            collection_name: "test".into(),
            dimension: 4,
            distance_metric: DistanceMetric::Cosine,
        })
    }

    fn make_vector(id: &str, data: Vec<f32>) -> EmbeddingVector {
        EmbeddingVector {
            id: id.into(),
            data,
            metadata: crate::embedding::EmbeddingMetadata {
                source: "test".into(),
                entity_id: id.into(),
                model: "test-model".into(),
                dimensions: 4,
            },
        }
    }

    #[test]
    fn test_upsert_and_search() {
        let client = make_client();
        let vec = make_vector("v1", vec![1.0, 0.0, 0.0, 0.0]);
        client
            .upsert(&vec, serde_json::json!({"name": "test"}))
            .unwrap();
        assert_eq!(client.count(), 1);

        let results = client.search(&[1.0, 0.0, 0.0, 0.0], 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "v1");
        assert!((results[0].score - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_search_top_k() {
        let client = make_client();
        client
            .upsert(
                &make_vector("v1", vec![1.0, 0.0, 0.0, 0.0]),
                serde_json::json!({}),
            )
            .unwrap();
        client
            .upsert(
                &make_vector("v2", vec![0.9, 0.1, 0.0, 0.0]),
                serde_json::json!({}),
            )
            .unwrap();
        client
            .upsert(
                &make_vector("v3", vec![0.0, 1.0, 0.0, 0.0]),
                serde_json::json!({}),
            )
            .unwrap();

        let results = client.search(&[1.0, 0.0, 0.0, 0.0], 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "v1");
    }

    #[test]
    fn test_delete_vector() {
        let client = make_client();
        client
            .upsert(
                &make_vector("v1", vec![1.0, 0.0, 0.0, 0.0]),
                serde_json::json!({}),
            )
            .unwrap();
        assert!(client.delete("v1"));
        assert_eq!(client.count(), 0);
        assert!(!client.delete("nonexistent"));
    }

    #[test]
    fn test_upsert_batch() {
        let client = make_client();
        let batch = vec![
            (
                make_vector("b1", vec![1.0, 0.0, 0.0, 0.0]),
                serde_json::json!({}),
            ),
            (
                make_vector("b2", vec![0.0, 1.0, 0.0, 0.0]),
                serde_json::json!({}),
            ),
        ];
        let count = client.upsert_batch(&batch).unwrap();
        assert_eq!(count, 2);
        assert_eq!(client.count(), 2);
    }

    #[test]
    fn test_clear() {
        let client = make_client();
        client
            .upsert(
                &make_vector("v1", vec![1.0, 0.0, 0.0, 0.0]),
                serde_json::json!({}),
            )
            .unwrap();
        client.clear();
        assert_eq!(client.count(), 0);
    }

    #[test]
    fn test_euclidean_metric() {
        let client = VectorDbClient::new(VectorDbConfig {
            collection_name: "test".into(),
            dimension: 2,
            distance_metric: DistanceMetric::Euclidean,
        });
        client
            .upsert(&make_vector("v1", vec![0.0, 0.0]), serde_json::json!({}))
            .unwrap();
        let results = client.search(&[3.0, 4.0], 1);
        assert_eq!(results[0].id, "v1");
        let expected_score = 1.0 / (1.0 + 5.0);
        assert!((results[0].score - expected_score).abs() < 0.01);
    }
}
