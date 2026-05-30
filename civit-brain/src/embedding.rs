#![forbid(unsafe_code)]

use crate::models::CodeEntity;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingVector {
    pub id: String,
    pub data: Vec<f32>,
    pub metadata: EmbeddingMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingMetadata {
    pub source: String,
    pub entity_id: String,
    pub model: String,
    pub dimensions: usize,
}

#[derive(Debug, Clone)]
pub struct EmbeddingWorker {
    dimensions: usize,
    model_name: String,
}

impl EmbeddingWorker {
    pub fn new(dimensions: usize) -> Self {
        Self {
            dimensions,
            model_name: "text-embedding-ada-002".into(),
        }
    }

    pub fn with_model(dimensions: usize, model_name: String) -> Self {
        Self {
            dimensions,
            model_name,
        }
    }

    pub async fn embed_text(&self, text: &str) -> anyhow::Result<EmbeddingVector> {
        let hash = simple_hash(text);
        let mut data = Vec::with_capacity(self.dimensions);
        let text_bytes = text.as_bytes();

        for i in 0..self.dimensions {
            let byte_idx = i % text_bytes.len();
            let value = (text_bytes[byte_idx] as f32 / 255.0) * 2.0 - 1.0;
            let positional = (i as f32 / self.dimensions as f32) * 0.1;
            data.push(value + positional);
        }

        let id = format!("emb-{hash}");
        debug!(id = %id, dims = self.dimensions, "generated embedding");

        Ok(EmbeddingVector {
            id,
            data,
            metadata: EmbeddingMetadata {
                source: "text".into(),
                entity_id: hash,
                model: self.model_name.clone(),
                dimensions: self.dimensions,
            },
        })
    }

    pub async fn embed_batch(&self, texts: &[&str]) -> anyhow::Result<Vec<EmbeddingVector>> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            let vector = self.embed_text(text).await?;
            results.push(vector);
        }
        debug!(count = results.len(), "generated batch embeddings");
        Ok(results)
    }

    pub async fn embed_entity(&self, entity: &CodeEntity) -> anyhow::Result<EmbeddingVector> {
        let text = format!(
            "{} {} {} {}:{} {}",
            entity.entity_type,
            entity.name,
            entity.file_path,
            entity.start_line,
            entity.end_line,
            entity.entity_type,
        );
        let mut vector = self.embed_text(&text).await?;
        vector.metadata.source = "code_entity".into();
        vector.metadata.entity_id = entity.id.clone();
        Ok(vector)
    }

    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
    }

    pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}

fn simple_hash(text: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embed_text() {
        let worker = EmbeddingWorker::new(128);
        let vector = worker.embed_text("hello world").await.unwrap();
        assert_eq!(vector.data.len(), 128);
        assert_eq!(vector.metadata.dimensions, 128);
        assert!(!vector.id.is_empty());
    }

    #[tokio::test]
    async fn test_embed_batch() {
        let worker = EmbeddingWorker::new(64);
        let texts = vec!["hello", "world", "foo"];
        let results = worker.embed_batch(&texts).await.unwrap();
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|v| v.data.len() == 64));
    }

    #[tokio::test]
    async fn test_embed_entity() {
        let worker = EmbeddingWorker::new(32);
        let entity = CodeEntity {
            id: "e1".into(),
            entity_type: "Function".into(),
            name: "main".into(),
            file_path: "src/main.rs".into(),
            start_line: 1,
            end_line: 10,
        };
        let vector = worker.embed_entity(&entity).await.unwrap();
        assert_eq!(vector.metadata.source, "code_entity");
        assert_eq!(vector.metadata.entity_id, "e1");
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let vec = vec![1.0, 0.0, 0.0, 0.0];
        let sim = EmbeddingWorker::cosine_similarity(&vec, &vec);
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = EmbeddingWorker::cosine_similarity(&a, &b);
        assert!(sim.abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_different_sizes() {
        let a = vec![1.0, 2.0];
        let b = vec![1.0];
        let sim = EmbeddingWorker::cosine_similarity(&a, &b);
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        let dist = EmbeddingWorker::euclidean_distance(&a, &b);
        assert!((dist - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_with_model() {
        let worker = EmbeddingWorker::with_model(256, "custom-model".into());
        assert_eq!(worker.model_name, "custom-model");
        assert_eq!(worker.dimensions, 256);
    }
}
