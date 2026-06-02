#![forbid(unsafe_code)]

use crate::embedding::{EmbeddingVector, EmbeddingWorker};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::future::Future;
use tracing::debug;

// ---------------------------------------------------------------------------
// VectorDb trait — async abstraction over any vector store backend
// ---------------------------------------------------------------------------

/// Async interface that both in-memory and Qdrant backends implement.
/// All methods are async so callers never need to know the backend.
///
/// Uses `impl Future` return types (RPITIT) for explicit `Send` bounds.
/// Not object-safe — use via generics `T: VectorDb`, not `dyn VectorDb`.
pub trait VectorDb: Send + Sync {
    /// Insert or replace a single vector with associated metadata.
    fn upsert(
        &self,
        vector: &EmbeddingVector,
        metadata: serde_json::Value,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;

    /// Bulk insert vectors. Returns the count actually upserted.
    fn upsert_batch(
        &self,
        vectors: &[(EmbeddingVector, serde_json::Value)],
    ) -> impl Future<Output = anyhow::Result<usize>> + Send;

    /// Similarity search. Returns up to `top_k` results sorted by score descending.
    fn search(
        &self,
        query: &[f32],
        top_k: usize,
    ) -> impl Future<Output = Vec<VectorSearchResult>> + Send;

    /// Remove a vector by id. Returns true if it existed.
    fn delete(&self, id: &str) -> impl Future<Output = anyhow::Result<bool>> + Send;

    /// Total number of stored vectors.
    fn count(&self) -> impl Future<Output = anyhow::Result<usize>> + Send;

    /// Health check (connectivity / liveness). Returns true if backend is reachable.
    fn health(&self) -> impl Future<Output = bool> + Send {
        async move { self.count().await.is_ok() }
    }
}

// ---------------------------------------------------------------------------
// Shared types
// ---------------------------------------------------------------------------

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

impl Default for VectorDbClient {
    fn default() -> Self {
        Self::new(VectorDbConfig {
            collection_name: "default".into(),
            dimension: 384,
            distance_metric: DistanceMetric::Cosine,
        })
    }
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

impl VectorDb for VectorDbClient {
    async fn upsert(
        &self,
        vector: &EmbeddingVector,
        metadata: serde_json::Value,
    ) -> anyhow::Result<()> {
        // Delegate to existing sync implementation.
        Self::upsert(self, vector, metadata)
    }

    async fn upsert_batch(
        &self,
        vectors: &[(EmbeddingVector, serde_json::Value)],
    ) -> anyhow::Result<usize> {
        Self::upsert_batch(self, vectors)
    }

    async fn search(&self, query: &[f32], top_k: usize) -> Vec<VectorSearchResult> {
        Self::search(self, query, top_k)
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        Ok(Self::delete(self, id))
    }

    async fn count(&self) -> anyhow::Result<usize> {
        Ok(Self::count(self))
    }
}

/// Adapter that delegates to a QdrantClient, implementing the same interface
/// as the in-memory VectorDbClient but backed by a remote Qdrant instance.
#[derive(Clone)]
pub struct QdrantVectorDbAdapter {
    client: crate::qdrant::client::QdrantClient,
}

impl Default for QdrantVectorDbAdapter {
    fn default() -> Self {
        Self::from_env()
    }
}

impl QdrantVectorDbAdapter {
    pub fn new(config: crate::qdrant::client::QdrantConfig) -> Self {
        Self {
            client: crate::qdrant::client::QdrantClient::new(config),
        }
    }

    pub fn qdrant_client(&self) -> &crate::qdrant::client::QdrantClient {
        &self.client
    }

    pub async fn upsert(
        &self,
        vector: &EmbeddingVector,
        metadata: serde_json::Value,
    ) -> anyhow::Result<()> {
        let point = crate::qdrant::client::QdrantPoint {
            id: vector.id.clone(),
            vector: vector.data.clone(),
            payload: metadata,
        };
        self.client.upsert_points(vec![point]).await?;
        Ok(())
    }

    pub async fn upsert_batch(
        &self,
        vectors: &[(EmbeddingVector, serde_json::Value)],
    ) -> anyhow::Result<usize> {
        let points: Vec<crate::qdrant::client::QdrantPoint> = vectors
            .iter()
            .map(|(v, m)| crate::qdrant::client::QdrantPoint {
                id: v.id.clone(),
                vector: v.data.clone(),
                payload: m.clone(),
            })
            .collect();
        let count = points.len();
        self.client.upsert_points(points).await?;
        Ok(count)
    }

    pub async fn search(&self, query: &[f32], top_k: usize) -> Vec<VectorSearchResult> {
        let request = crate::qdrant::client::QdrantSearchRequest {
            vector: query.to_vec(),
            top_k,
            filter: None,
            with_payload: true,
            score_threshold: None,
        };
        match self.client.search(request).await {
            Ok(points) => points
                .into_iter()
                .map(|p| VectorSearchResult {
                    id: p.id,
                    score: p.score,
                    metadata: p.payload,
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    pub async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        let filter = crate::qdrant::client::FilterCondition::match_eq("id", serde_json::json!(id));
        let mut f = crate::qdrant::client::QdrantFilter::new();
        f.must.push(filter);
        let deleted = self.client.delete_points(f).await?;
        Ok(deleted > 0)
    }

    pub async fn count(&self) -> anyhow::Result<usize> {
        match self.client.collection_info().await {
            Ok(info) => Ok(info.vectors_count as usize),
            Err(_) => Ok(0),
        }
    }

    pub async fn health(&self) -> anyhow::Result<bool> {
        self.client.health().await
    }

    /// Convenience constructor that reads Qdrant config from environment variables:
    /// - `CIVITFORGE_QDRANT_URL` (default: `http://localhost:6333`)
    /// - `CIVITFORGE_QDRANT_API_KEY` (optional)
    /// - `CIVITFORGE_QDRANT_COLLECTION` (default: `civitforge`)
    /// - `CIVITFORGE_QDRANT_VECTOR_SIZE` (default: `384`)
    pub fn from_env() -> Self {
        let url = std::env::var("CIVITFORGE_QDRANT_URL")
            .unwrap_or_else(|_| "http://localhost:6333".into());
        let api_key = std::env::var("CIVITFORGE_QDRANT_API_KEY").ok();
        let collection_name =
            std::env::var("CIVITFORGE_QDRANT_COLLECTION").unwrap_or_else(|_| "civitforge".into());
        let vector_size: usize = std::env::var("CIVITFORGE_QDRANT_VECTOR_SIZE")
            .unwrap_or_else(|_| "384".into())
            .parse()
            .unwrap_or(384);

        let config = crate::qdrant::client::QdrantConfig {
            url,
            api_key,
            collection_name,
            vector_size,
            timeout: std::time::Duration::from_secs(30),
        };
        Self::new(config)
    }
}

impl VectorDb for QdrantVectorDbAdapter {
    async fn upsert(
        &self,
        vector: &EmbeddingVector,
        metadata: serde_json::Value,
    ) -> anyhow::Result<()> {
        Self::upsert(self, vector, metadata).await
    }

    async fn upsert_batch(
        &self,
        vectors: &[(EmbeddingVector, serde_json::Value)],
    ) -> anyhow::Result<usize> {
        Self::upsert_batch(self, vectors).await
    }

    async fn search(&self, query: &[f32], top_k: usize) -> Vec<VectorSearchResult> {
        Self::search(self, query, top_k).await
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        Self::delete(self, id).await
    }

    async fn count(&self) -> anyhow::Result<usize> {
        Self::count(self).await
    }

    async fn health(&self) -> bool {
        Self::health(self).await.unwrap_or(false)
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

    // -----------------------------------------------------------------------
    // VectorDb trait tests (in-memory backend)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_trait_upsert_and_search() {
        let client = make_client();
        let vec = make_vector("tv1", vec![1.0, 0.0, 0.0, 0.0]);
        VectorDb::upsert(&client, &vec, serde_json::json!({"name": "trait-test"}))
            .await
            .unwrap();
        assert_eq!(VectorDb::count(&client).await.unwrap(), 1);

        let results = VectorDb::search(&client, &[1.0, 0.0, 0.0, 0.0], 5).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "tv1");
    }

    #[tokio::test]
    async fn test_trait_upsert_batch() {
        let client = make_client();
        let batch = vec![
            (
                make_vector("tb1", vec![1.0, 0.0, 0.0, 0.0]),
                serde_json::json!({}),
            ),
            (
                make_vector("tb2", vec![0.0, 1.0, 0.0, 0.0]),
                serde_json::json!({}),
            ),
        ];
        let count = VectorDb::upsert_batch(&client, &batch).await.unwrap();
        assert_eq!(count, 2);
        assert_eq!(VectorDb::count(&client).await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_trait_delete() {
        let client = make_client();
        VectorDb::upsert(
            &client,
            &make_vector("td1", vec![1.0, 0.0, 0.0, 0.0]),
            serde_json::json!({}),
        )
        .await
        .unwrap();
        assert!(VectorDb::delete(&client, "td1").await.unwrap());
        assert_eq!(VectorDb::count(&client).await.unwrap(), 0);
        assert!(!VectorDb::delete(&client, "nonexistent").await.unwrap());
    }

    #[tokio::test]
    async fn test_trait_health() {
        let client = make_client();
        assert!(VectorDb::health(&client).await);
    }

    #[tokio::test]
    async fn test_trait_search_empty() {
        let client = make_client();
        let results = VectorDb::search(&client, &[1.0, 0.0, 0.0, 0.0], 5).await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_trait_search_filters_by_dimension() {
        let client = make_client();
        VectorDb::upsert(
            &client,
            &make_vector("dim4", vec![1.0, 0.0, 0.0, 0.0]),
            serde_json::json!({}),
        )
        .await
        .unwrap();
        let results = VectorDb::search(&client, &[1.0, 0.0], 5).await;
        assert!(
            results.is_empty(),
            "mismatched dimensions should yield no results"
        );
    }

    #[test]
    fn test_qdrant_adapter_from_env() {
        // from_env should not panic with default env vars
        let _adapter = QdrantVectorDbAdapter::from_env();
    }

    #[test]
    fn test_qdrant_adapter_default() {
        let _adapter = QdrantVectorDbAdapter::default();
    }

    #[test]
    fn test_vector_db_client_default() {
        let client = VectorDbClient::default();
        assert_eq!(client.count(), 0);
    }
}
