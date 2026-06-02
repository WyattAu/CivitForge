#![forbid(unsafe_code)]

use crate::embedding::EmbeddingWorker;
use crate::models::CodeEntity;
use crate::vectordb::{VectorDb, VectorDbClient};
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGContext {
    pub query: String,
    pub chunks: Vec<CodeChunk>,
    pub total_tokens_estimate: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChunk {
    pub entity_id: String,
    pub entity_type: String,
    pub name: String,
    pub file_path: String,
    pub content: String,
    pub relevance_score: f32,
}

/// RAG pipeline generic over any vector database backend.
/// Use `InMemoryRagPipeline` for tests or `RAGPipeline<QdrantVectorDbAdapter>` for production.
pub struct RAGPipeline<T: VectorDb> {
    embedding_worker: EmbeddingWorker,
    vector_db: T,
    max_context_chunks: usize,
    min_relevance_score: f32,
}

/// Convenience alias: RAG pipeline backed by the in-memory DashMap vector store.
pub type InMemoryRagPipeline = RAGPipeline<VectorDbClient>;

impl<T: VectorDb> RAGPipeline<T> {
    pub fn new(
        embedding_worker: EmbeddingWorker,
        vector_db: T,
        max_context_chunks: usize,
        min_relevance_score: f32,
    ) -> Self {
        Self {
            embedding_worker,
            vector_db,
            max_context_chunks,
            min_relevance_score,
        }
    }

    /// Build a RAG pipeline from environment variables.
    ///
    /// Vector backend selection:
    /// - `CIVITFORGE_VECTOR_BACKEND=inmemory` (default) — DashMap-based, no external deps
    /// - `CIVITFORGE_VECTOR_BACKEND=qdrant` — Qdrant HTTP client (reads QDRANT_URL, etc.)
    ///
    /// Embedding config: reads `CIVITFORGE_EMBEDDING_*` vars (see `EmbeddingWorker::new()`).
    pub fn from_env() -> Self
    where
        T: VectorDb + Default,
    {
        let db = T::default();
        let worker = EmbeddingWorker::new();
        Self::new(worker, db, 10, 0.5)
    }

    pub async fn retrieve(&self, query: &str) -> anyhow::Result<RAGContext> {
        let query_embedding = self.embedding_worker.embed_text(query).await?;
        let search_results = self
            .vector_db
            .search(&query_embedding.data, self.max_context_chunks)
            .await;

        let chunks: Vec<CodeChunk> = search_results
            .into_iter()
            .filter(|r| r.score >= self.min_relevance_score)
            .map(|r| CodeChunk {
                entity_id: r.id.clone(),
                entity_type: r
                    .metadata
                    .get("entity_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .into(),
                name: r
                    .metadata
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .into(),
                file_path: r
                    .metadata
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .into(),
                content: r
                    .metadata
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .into(),
                relevance_score: r.score,
            })
            .collect();

        let total_tokens: usize = chunks.iter().map(|c| estimate_tokens(&c.content)).sum();

        debug!(query = %query, chunks = chunks.len(), tokens = total_tokens, "RAG retrieval complete");

        Ok(RAGContext {
            query: query.into(),
            chunks,
            total_tokens_estimate: total_tokens,
        })
    }

    pub fn build_prompt(&self, context: &RAGContext, user_query: &str) -> String {
        let mut prompt =
            String::from("You are a code review assistant. Use the following code context:\n\n");

        for chunk in &context.chunks {
            prompt.push_str(&format!(
                "[{}:{}] {} ({})\n{}\n\n",
                chunk.file_path, chunk.entity_id, chunk.name, chunk.entity_type, chunk.content,
            ));
        }

        prompt.push_str(&format!(
            "User question: {user_query}\n\nProvide a detailed code review:"
        ));
        prompt
    }

    pub async fn index_entity(&self, entity: &CodeEntity, content: &str) -> anyhow::Result<()> {
        let embedding = self.embedding_worker.embed_entity(entity).await?;
        let metadata = serde_json::json!({
            "entity_type": entity.entity_type,
            "name": entity.name,
            "file_path": entity.file_path,
            "start_line": entity.start_line,
            "end_line": entity.end_line,
            "content": content,
        });
        self.vector_db.upsert(&embedding, metadata).await?;
        Ok(())
    }
}

fn estimate_tokens(text: &str) -> usize {
    (text.len() as f32 / 4.0).ceil() as usize
}

// ---------------------------------------------------------------------------
// LlmCodeReviewer — end-to-end RAG → LLM code review
// ---------------------------------------------------------------------------

use crate::llm::provider::{ChatMessage, LlmProvider, ModelConfig};

/// End-to-end code reviewer: retrieves context via RAG, sends to LLM for review.
///
/// ```text
/// Diff/Code → RAG retrieve() → context prompt → LlmProvider.infer() → ReviewResult
/// ```
pub struct LlmCodeReviewer<T: VectorDb, P: LlmProvider> {
    rag: RAGPipeline<T>,
    llm: P,
    model_config: ModelConfig,
    max_response_tokens: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LlmReviewResult {
    pub review_text: String,
    pub context_chunks: Vec<CodeChunk>,
    pub tokens_used: u32,
    pub model: String,
    pub duration_ms: u64,
}

impl<T: VectorDb, P: LlmProvider> LlmCodeReviewer<T, P> {
    pub fn new(
        rag: RAGPipeline<T>,
        llm: P,
        model_config: ModelConfig,
        max_response_tokens: u32,
    ) -> Self {
        Self {
            rag,
            llm,
            model_config,
            max_response_tokens,
        }
    }

    /// Review a diff or code snippet end-to-end: RAG retrieve → LLM infer.
    pub async fn review(
        &self,
        diff_content: &str,
        file_path: &str,
    ) -> anyhow::Result<LlmReviewResult> {
        let start = std::time::Instant::now();

        // 1. Retrieve relevant context from vector DB
        let context = self.rag.retrieve(diff_content).await?;

        // 2. Build prompt with RAG context + diff
        let prompt = self.rag.build_prompt(
            &context,
            &format!("Review this diff for {file_path}:\n{diff_content}"),
        );

        // 3. Send to LLM
        let messages = vec![ChatMessage {
            role: "user".into(),
            content: prompt,
        }];

        let result = self
            .llm
            .infer(&messages, &self.model_config, self.max_response_tokens)
            .map_err(|e| anyhow::anyhow!("LLM inference failed: {e}"))?;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(LlmReviewResult {
            review_text: result.content,
            context_chunks: context.chunks,
            tokens_used: result.tokens_used,
            model: result.model,
            duration_ms,
        })
    }

    /// Index a code entity into the vector DB for future retrieval.
    pub async fn index_entity(&self, entity: &CodeEntity, content: &str) -> anyhow::Result<()> {
        self.rag.index_entity(entity, content).await
    }

    /// Check if both the vector DB and LLM are available.
    pub async fn health(&self) -> (bool, bool) {
        let db_health = self.rag.vector_db.health().await;
        let llm_health = self.llm.is_available();
        (db_health, llm_health)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::provider::{ModelConfig, StubLlmProvider};
    use crate::vectordb::{DistanceMetric, VectorDbConfig};

    fn make_rag() -> InMemoryRagPipeline {
        let worker = EmbeddingWorker::with_dimensions(16);
        let db = VectorDbClient::new(VectorDbConfig {
            collection_name: "test".into(),
            dimension: 16,
            distance_metric: DistanceMetric::Cosine,
        });
        RAGPipeline::new(worker, db, 5, 0.0)
    }

    fn make_review_config() -> ModelConfig {
        ModelConfig {
            name: "test-code-reviewer".into(),
            parameter_count: 7_000_000_000,
            context_window: 8192,
            max_tokens: 2048,
            endpoint: None,
            quantization: None,
        }
    }

    fn make_llm_reviewer() -> LlmCodeReviewer<VectorDbClient, StubLlmProvider> {
        LlmCodeReviewer::new(
            make_rag(),
            StubLlmProvider::new(),
            make_review_config(),
            1024,
        )
    }

    #[tokio::test]
    async fn test_retrieve_empty() {
        let rag = make_rag();
        let context = rag.retrieve("what is main?").await.unwrap();
        assert_eq!(context.chunks.len(), 0);
        assert_eq!(context.query, "what is main?");
    }

    #[tokio::test]
    async fn test_index_and_retrieve() {
        let rag = make_rag();
        let entity = CodeEntity {
            id: "e1".into(),
            entity_type: "Function".into(),
            name: "main".into(),
            file_path: "src/main.rs".into(),
            start_line: 1,
            end_line: 10,
        };
        rag.index_entity(&entity, "fn main() { println!(\"hello\"); }")
            .await
            .unwrap();
        let context = rag.retrieve("main function").await.unwrap();
        assert!(!context.chunks.is_empty());
    }

    #[test]
    fn test_build_prompt() {
        let rag = make_rag();
        let context = RAGContext {
            query: "review this".into(),
            chunks: vec![CodeChunk {
                entity_id: "e1".into(),
                entity_type: "Function".into(),
                name: "main".into(),
                file_path: "src/main.rs".into(),
                content: "fn main() {}".into(),
                relevance_score: 0.95,
            }],
            total_tokens_estimate: 10,
        };
        let prompt = rag.build_prompt(&context, "is this good code?");
        assert!(prompt.contains("main"));
        assert!(prompt.contains("src/main.rs"));
        assert!(prompt.contains("is this good code?"));
    }

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(estimate_tokens("hello"), 2);
        assert_eq!(estimate_tokens(""), 0);
        assert!(estimate_tokens(&("a".repeat(100))) >= 25);
    }

    // -----------------------------------------------------------------------
    // LlmCodeReviewer tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_llm_reviewer_empty_db() {
        let reviewer = make_llm_reviewer();
        let result = reviewer.review("+let x = 5;", "src/main.rs").await.unwrap();
        // Stub LLM should return something
        assert!(!result.review_text.is_empty());
        assert!(result.review_text.contains("[STUB]"));
        assert_eq!(result.model, "test-code-reviewer");
        assert!(result.context_chunks.is_empty());
    }

    #[tokio::test]
    async fn test_llm_reviewer_with_indexed_entity() {
        let reviewer = make_llm_reviewer();
        let entity = CodeEntity {
            id: "e1".into(),
            entity_type: "Function".into(),
            name: "authenticate".into(),
            file_path: "src/auth.rs".into(),
            start_line: 1,
            end_line: 10,
        };
        reviewer
            .index_entity(
                &entity,
                "fn authenticate(token: &str) -> bool { token == \"secret\" }",
            )
            .await
            .unwrap();

        let result = reviewer
            .review("+let auth = authenticate(&token);", "src/main.rs")
            .await
            .unwrap();
        assert!(!result.review_text.is_empty());
        // The indexed entity should be retrieved as context
        assert!(!result.context_chunks.is_empty());
    }

    #[tokio::test]
    async fn test_llm_reviewer_health() {
        let reviewer = make_llm_reviewer();
        let (db_health, llm_health) = reviewer.health().await;
        assert!(db_health, "in-memory DB should be healthy");
        assert!(llm_health, "stub LLM should be available");
    }

    #[test]
    fn test_llm_review_result_serialization() {
        let result = LlmReviewResult {
            review_text: "LGTM".into(),
            context_chunks: vec![],
            tokens_used: 42,
            model: "test".into(),
            duration_ms: 10,
        };
        let json = serde_json::to_string(&result).unwrap();
        let de: LlmReviewResult = serde_json::from_str(&json).unwrap();
        assert_eq!(de.review_text, "LGTM");
        assert_eq!(de.tokens_used, 42);
    }

    #[test]
    fn test_in_memory_rag_pipeline_type_alias() {
        // Verify the type alias compiles correctly
        let worker = EmbeddingWorker::with_dimensions(8);
        let db = VectorDbClient::new(VectorDbConfig {
            collection_name: "alias-test".into(),
            dimension: 8,
            distance_metric: DistanceMetric::Cosine,
        });
        let _: InMemoryRagPipeline = RAGPipeline::new(worker, db, 5, 0.0);
    }
}
