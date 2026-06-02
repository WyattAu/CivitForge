#![forbid(unsafe_code)]

//! Embedding client for generating vector representations of text.
//!
//! Supports two backends:
//!
//! 1. **API** (default): Calls `/v1/embeddings` on any OpenAI-compatible server
//!    (OpenAI, vLLM, Ollama, Voyage AI, text-embeddings-inference, etc.)
//!
//! 2. **Deterministic** (testing/dev): Produces vectors from byte patterns.
//!    No semantic meaning — for development and testing only.
//!
//! Backend is selected at construction time or via environment variable
//! `CIVITFORGE_EMBEDDING_BACKEND=api|deterministic`.

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the embedding client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Base URL of the embeddings API server.
    /// Examples: `http://localhost:11434` (Ollama), `http://localhost:8000` (vLLM),
    /// `https://api.openai.com` (OpenAI).
    pub base_url: String,

    /// Model name to use for embeddings.
    /// Examples: `text-embedding-3-small`, `nomic-embed-text`, `all-MiniLM-L6-v2`.
    pub model: String,

    /// Dimensions expected from the model. Used for validation.
    /// If 0, accepts whatever dimensions the model returns.
    pub dimensions: usize,

    /// API key for authenticated endpoints. None for local servers.
    #[serde(default)]
    pub api_key: Option<String>,

    /// HTTP timeout in seconds.
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_timeout() -> u64 {
    30
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            // Read from environment or use sensible defaults
            base_url: std::env::var("CIVITFORGE_EMBEDDING_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434".into()),
            model: std::env::var("CIVITFORGE_EMBEDDING_MODEL")
                .unwrap_or_else(|_| "nomic-embed-text".into()),
            dimensions: 0,
            api_key: std::env::var("CIVITFORGE_EMBEDDING_API_KEY").ok(),
            timeout_secs: 30,
        }
    }
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// A single embedding response item from the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponseItem {
    /// The embedding object.
    pub object: String,
    /// The embedding vector.
    pub embedding: Vec<f32>,
    /// The index of this embedding in the request input list.
    pub index: usize,
}

/// Full response from the `/v1/embeddings` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsApiResponse {
    /// The list of embedding objects.
    pub data: Vec<EmbeddingResponseItem>,
    /// The model used for the embeddings.
    pub model: String,
    /// Usage statistics from the API.
    pub usage: Option<EmbeddingUsage>,
}

/// Token usage information returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingUsage {
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}

// ---------------------------------------------------------------------------
// Legacy types (backward compat)
// ---------------------------------------------------------------------------

/// A generated embedding vector with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingVector {
    pub id: String,
    pub data: Vec<f32>,
    pub metadata: EmbeddingMetadata,
}

impl EmbeddingVector {
    /// Create an embedding vector with the given data.
    pub fn new(
        id: String,
        data: Vec<f32>,
        model: String,
        source: String,
        entity_id: String,
    ) -> Self {
        Self {
            id,
            metadata: EmbeddingMetadata {
                source,
                entity_id,
                model,
                dimensions: data.len(),
            },
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingMetadata {
    pub source: String,
    pub entity_id: String,
    pub model: String,
    pub dimensions: usize,
}

// ---------------------------------------------------------------------------
// Backend enum
// ---------------------------------------------------------------------------

/// Which embedding backend to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmbeddingBackend {
    /// Call `/v1/embeddings` on an OpenAI-compatible API server.
    Api,
    /// Deterministic byte-pattern vectors (testing/dev only, no semantic meaning).
    Deterministic,
}

impl Default for EmbeddingBackend {
    fn default() -> Self {
        match std::env::var("CIVITFORGE_EMBEDDING_BACKEND").as_deref() {
            Ok("deterministic") => EmbeddingBackend::Deterministic,
            _ => EmbeddingBackend::Api,
        }
    }
}

// ---------------------------------------------------------------------------
// EmbeddingWorker (main entry point)
// ---------------------------------------------------------------------------

/// Embedding client that generates vector representations of text.
///
/// Supports two backends:
/// - `Api`: Calls any OpenAI-compatible `/v1/embeddings` endpoint
/// - `Deterministic`: Produces deterministic vectors from byte patterns (dev/test only)
#[derive(Clone)]
pub struct EmbeddingWorker {
    config: EmbeddingConfig,
    backend: EmbeddingBackend,
    http_client: std::sync::Arc<reqwest::Client>,
}

impl std::fmt::Debug for EmbeddingWorker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbeddingWorker")
            .field("config", &self.config)
            .field("backend", &self.backend)
            .field("http_client", &format_args!("Arc<Client>"))
            .finish()
    }
}

impl Default for EmbeddingWorker {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbeddingWorker {
    /// Create a worker with default configuration.
    ///
    /// Reads from environment variables:
    /// - `CIVITFORGE_EMBEDDING_BASE_URL` (default: `http://localhost:11434`)
    /// - `CIVITFORGE_EMBEDDING_MODEL` (default: `nomic-text-embed`)
    /// - `CIVITFORGE_EMBEDDING_API_KEY` (optional)
    /// - `CIVITFORGE_EMBEDDING_BACKEND` (`api` or `deterministic`, default: `api`)
    pub fn new() -> Self {
        Self::with_config(EmbeddingConfig::default())
    }

    /// Create a deterministic embedding worker with a specific dimension count.
    /// Backward-compatible with the old API `EmbeddingWorker::new(dimensions)`.
    pub fn with_dimensions(dimensions: usize) -> Self {
        Self::with_config_and_backend(
            EmbeddingConfig {
                dimensions,
                ..EmbeddingConfig::default()
            },
            EmbeddingBackend::Deterministic,
        )
    }

    /// Create an embedding worker with explicit configuration.
    pub fn with_config(config: EmbeddingConfig) -> Self {
        let backend = EmbeddingBackend::default();
        Self::with_config_and_backend(config, backend)
    }

    /// Create an embedding worker with explicit configuration and backend selection.
    pub fn with_config_and_backend(config: EmbeddingConfig, backend: EmbeddingBackend) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            config,
            backend,
            http_client: std::sync::Arc::new(http_client),
        }
    }

    /// Generate an embedding vector for a single text.
    pub async fn embed_text(&self, text: &str) -> anyhow::Result<EmbeddingVector> {
        let data = match self.backend {
            EmbeddingBackend::Api => self.embed_api(text).await?,
            EmbeddingBackend::Deterministic => deterministic_embed(text, 768),
        };

        let model = match self.backend {
            EmbeddingBackend::Api => self.config.model.clone(),
            EmbeddingBackend::Deterministic => "deterministic".into(),
        };

        let hash = simple_hash(text);
        Ok(EmbeddingVector::new(
            format!("emb-{hash}"),
            data,
            model,
            "text".into(),
            hash,
        ))
    }

    /// Generate embeddings for a batch of texts in a single API call.
    pub async fn embed_batch(&self, texts: &[&str]) -> anyhow::Result<Vec<EmbeddingVector>> {
        let vectors = match self.backend {
            EmbeddingBackend::Api => self.embed_batch_api(texts).await?,
            EmbeddingBackend::Deterministic => texts
                .iter()
                .map(|t| {
                    let hash = simple_hash(t);
                    EmbeddingVector::new(
                        format!("emb-{hash}"),
                        deterministic_embed(t, 768),
                        "deterministic".into(),
                        "text".into(),
                        hash,
                    )
                })
                .collect(),
        };
        debug!(count = vectors.len(), "generated batch embeddings");
        Ok(vectors)
    }

    /// Generate an embedding for a code entity.
    pub async fn embed_entity(
        &self,
        entity: &crate::models::CodeEntity,
    ) -> anyhow::Result<EmbeddingVector> {
        let text = format!(
            "{} {} {} {}:{}",
            entity.entity_type, entity.name, entity.file_path, entity.start_line, entity.end_line,
        );
        let mut vector = self.embed_text(&text).await?;
        vector.metadata.source = "code_entity".into();
        vector.metadata.entity_id = entity.id.clone();
        Ok(vector)
    }

    /// Check if the embedding backend is available.
    pub async fn health_check(&self) -> bool {
        match self.backend {
            EmbeddingBackend::Api => {
                let url = format!("{}/v1/models", self.config.base_url.trim_end_matches('/'));
                let mut req = self.http_client.get(&url);
                if let Some(key) = &self.config.api_key {
                    req = req.header("Authorization", format!("Bearer {key}"));
                }
                match req.send().await {
                    Ok(resp) => resp.status().is_success(),
                    Err(_) => false,
                }
            }
            EmbeddingBackend::Deterministic => true,
        }
    }

    /// Which backend is being used.
    pub fn backend(&self) -> EmbeddingBackend {
        self.backend
    }

    /// The configured embedding model name.
    pub fn model_name(&self) -> &str {
        &self.config.model
    }

    /// The configured dimensions (0 = accept whatever model returns).
    pub fn dimensions(&self) -> usize {
        self.config.dimensions
    }

    // -----------------------------------------------------------------------
    // API backend
    // -----------------------------------------------------------------------

    async fn embed_api(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let inputs = [text];
        self.embed_batch_api(&inputs)
            .await
            .map(|mut vecs| vecs.pop().unwrap().data)
    }

    async fn embed_batch_api(&self, texts: &[&str]) -> anyhow::Result<Vec<EmbeddingVector>> {
        let body = serde_json::json!({
            "model": self.config.model,
            "input": texts,
            "encoding_format": "float",
        });

        let url = format!(
            "{}/v1/embeddings",
            self.config.base_url.trim_end_matches('/')
        );

        let mut req = self.http_client.post(&url).json(&body);
        if let Some(key) = &self.config.api_key {
            req = req.header("Authorization", format!("Bearer {key}"));
        }

        let resp = req.send().await?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            warn!(%status, %text, "embeddings API error");
            anyhow::bail!("embeddings API returned {status}: {text}");
        }

        let api_resp: EmbeddingsApiResponse = resp.json().await?;
        debug!(
            model = %api_resp.model,
            count = api_resp.data.len(),
            dims = api_resp.data.first().map(|d| d.embedding.len()).unwrap_or(0),
            "API embeddings response"
        );

        // Validate dimensions if configured
        if self.config.dimensions > 0 {
            for item in &api_resp.data {
                if item.embedding.len() != self.config.dimensions {
                    warn!(
                        expected = self.config.dimensions,
                        actual = item.embedding.len(),
                        "dimension mismatch"
                    );
                    anyhow::bail!(
                        "dimension mismatch: expected {}, got {}",
                        self.config.dimensions,
                        item.embedding.len()
                    );
                }
            }
        }

        let hash_fn = |i: usize| simple_hash(texts.get(i).copied().unwrap_or(""));

        Ok(api_resp
            .data
            .into_iter()
            .map(|item| {
                let hash = hash_fn(item.index);
                EmbeddingVector::new(
                    format!("emb-{hash}"),
                    item.embedding,
                    api_resp.model.clone(),
                    "text".into(),
                    hash,
                )
            })
            .collect())
    }

    // -----------------------------------------------------------------------
    // Distance metrics (shared)
    // -----------------------------------------------------------------------

    /// Cosine similarity between two vectors.
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

    /// Euclidean distance between two vectors.
    pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}

// ---------------------------------------------------------------------------
// Deterministic embedding (testing/dev fallback)
// ---------------------------------------------------------------------------

/// Generate a deterministic but semantically meaningless vector from text.
/// Only for testing and development — has zero semantic quality.
fn deterministic_embed(text: &str, dimensions: usize) -> Vec<f32> {
    let hash = simple_hash(text);
    let hash_bytes = hash.as_bytes();
    let mut data = Vec::with_capacity(dimensions);

    for i in 0..dimensions {
        // Use the hash bytes cyclically as a deterministic seed
        let seed_byte = hash_bytes[i % hash_bytes.len()];
        // Mix in the position for positional encoding
        let positional = (i as f32 / dimensions as f32) * 0.001;
        let value = (seed_byte as f32 / 255.0) * 2.0 - 1.0;
        data.push(value + positional);
    }

    data
}

// ---------------------------------------------------------------------------
// Hash utility
// ---------------------------------------------------------------------------

fn simple_hash(text: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CodeEntity;

    fn api_config() -> EmbeddingConfig {
        EmbeddingConfig {
            base_url: "http://localhost:12345".into(), // unlikely to be running
            model: "test-model".into(),
            dimensions: 0,
            api_key: None,
            timeout_secs: 1,
        }
    }

    #[test]
    fn test_default_config() {
        let config = EmbeddingConfig::default();
        // Should have sensible defaults
        assert!(!config.base_url.is_empty());
        assert!(!config.model.is_empty());
        assert_eq!(config.timeout_secs, 30);
    }

    #[tokio::test]
    async fn test_deterministic_embed_text() {
        let worker =
            EmbeddingWorker::with_config_and_backend(api_config(), EmbeddingBackend::Deterministic);
        let vector = worker.embed_text("hello world").await.unwrap();
        assert_eq!(vector.data.len(), 768);
        assert_eq!(vector.metadata.model, "deterministic");
    }

    #[tokio::test]
    async fn test_deterministic_embed_batch() {
        let worker =
            EmbeddingWorker::with_config_and_backend(api_config(), EmbeddingBackend::Deterministic);
        let texts = vec!["hello", "world", "foo"];
        let results = worker.embed_batch(&texts).await.unwrap();
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|v| v.data.len() == 768));
    }

    #[tokio::test]
    async fn test_deterministic_embed_entity() {
        let worker =
            EmbeddingWorker::with_config_and_backend(api_config(), EmbeddingBackend::Deterministic);
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
        assert_eq!(vector.data.len(), 768);
    }

    #[tokio::test]
    async fn test_deterministic_deterministic_vectors() {
        let worker =
            EmbeddingWorker::with_config_and_backend(api_config(), EmbeddingBackend::Deterministic);
        // Same text → same vector
        let v1 = worker.embed_text("hello").await.unwrap();
        let v2 = worker.embed_text("hello").await.unwrap();
        assert_eq!(v1.data, v2.data);
        // Different text → different vector (most likely)
        let v3 = worker.embed_text("goodbye").await.unwrap();
        assert_ne!(v1.data, v3.data);
    }

    #[tokio::test]
    async fn test_deterministic_embed_empty() {
        let worker =
            EmbeddingWorker::with_config_and_backend(api_config(), EmbeddingBackend::Deterministic);
        let vector = worker.embed_text("").await.unwrap();
        assert_eq!(vector.data.len(), 768);
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
    fn test_cosine_similarity_zero_vector() {
        let a = vec![0.0, 0.0];
        let b = vec![1.0, 0.0];
        assert_eq!(EmbeddingWorker::cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        let dist = EmbeddingWorker::euclidean_distance(&a, &b);
        assert!((dist - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_euclidean_distance_same() {
        let a = vec![1.0, 2.0];
        let dist = EmbeddingWorker::euclidean_distance(&a, &a);
        assert!(dist.abs() < 0.001);
    }

    #[test]
    fn test_embedding_vector_new() {
        let vec = EmbeddingVector::new(
            "emb-abc".into(),
            vec![0.1, 0.2, 0.3],
            "test-model".into(),
            "text".into(),
            "abc".into(),
        );
        assert_eq!(vec.id, "emb-abc");
        assert_eq!(vec.data.len(), 3);
        assert_eq!(vec.metadata.model, "test-model");
    }

    #[test]
    fn test_config_serialization() {
        let config = EmbeddingConfig {
            base_url: "http://localhost:11434".into(),
            model: "nomic-embed-text".into(),
            dimensions: 768,
            api_key: Some("sk-test".into()),
            timeout_secs: 60,
        };
        let json = serde_json::to_string(&config).unwrap();
        let de: EmbeddingConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(de.model, "nomic-embed-text");
        assert_eq!(de.dimensions, 768);
        assert_eq!(de.api_key.as_deref(), Some("sk-test"));
    }

    #[test]
    fn test_embedding_vector_serialization() {
        let vec = EmbeddingVector::new(
            "emb-123".into(),
            vec![1.0, 2.0],
            "model".into(),
            "source".into(),
            "id123".into(),
        );
        let json = serde_json::to_string(&vec).unwrap();
        let de: EmbeddingVector = serde_json::from_str(&json).unwrap();
        assert_eq!(de.id, "emb-123");
        assert_eq!(de.data.len(), 2);
    }

    #[test]
    fn test_embedding_backend_default() {
        let backend = EmbeddingBackend::default();
        // Default should be Api unless env var is set
        assert_eq!(backend, EmbeddingBackend::Api);
    }

    #[tokio::test]
    async fn test_backend_property() {
        let worker =
            EmbeddingWorker::with_config_and_backend(api_config(), EmbeddingBackend::Deterministic);
        assert_eq!(worker.backend(), EmbeddingBackend::Deterministic);
        assert_eq!(worker.model_name(), "test-model");
    }

    #[test]
    fn test_simple_hash_deterministic() {
        let h1 = simple_hash("hello");
        let h2 = simple_hash("hello");
        let h3 = simple_hash("world");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
        assert!(h1.len() == 16); // 16 hex chars
    }

    #[tokio::test]
    async fn test_api_embed_unreachable() {
        let worker = EmbeddingWorker::with_config_and_backend(api_config(), EmbeddingBackend::Api);
        // Port 12345 unlikely to have a server — should error
        let result = worker.embed_text("test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_health_check_deterministic() {
        let worker =
            EmbeddingWorker::with_config_and_backend(api_config(), EmbeddingBackend::Deterministic);
        assert!(worker.health_check().await);
    }

    #[tokio::test]
    async fn test_health_check_api_unreachable() {
        let worker = EmbeddingWorker::with_config_and_backend(api_config(), EmbeddingBackend::Api);
        // Server not running
        assert!(!worker.health_check().await);
    }

    #[test]
    fn test_embeddings_api_response_parse() {
        let json = serde_json::json!({
            "data": [
                {
                    "object": "embedding",
                    "embedding": [0.1, 0.2, 0.3],
                    "index": 0
                },
                {
                    "object": "embedding",
                    "embedding": [0.4, 0.5, 0.6],
                    "index": 1
                }
            ],
            "model": "text-embedding-3-small",
            "usage": {
                "prompt_tokens": 8,
                "total_tokens": 8
            }
        });
        let resp: EmbeddingsApiResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.data.len(), 2);
        assert_eq!(resp.data[0].embedding, vec![0.1, 0.2, 0.3]);
        assert_eq!(resp.model, "text-embedding-3-small");
        assert_eq!(resp.usage.unwrap().prompt_tokens, 8);
    }

    #[test]
    fn test_embeddings_api_response_minimal() {
        let json = serde_json::json!({"data": [], "model": "test"});
        let resp: EmbeddingsApiResponse = serde_json::from_value(json).unwrap();
        assert!(resp.data.is_empty());
    }
}
