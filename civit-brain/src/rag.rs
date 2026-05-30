#![forbid(unsafe_code)]

use crate::embedding::EmbeddingWorker;
use crate::models::CodeEntity;
use crate::vectordb::VectorDbClient;
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

#[derive(Debug, Clone)]
pub struct RAGPipeline {
    embedding_worker: EmbeddingWorker,
    vector_db: VectorDbClient,
    max_context_chunks: usize,
    min_relevance_score: f32,
}

impl RAGPipeline {
    pub fn new(
        embedding_worker: EmbeddingWorker,
        vector_db: VectorDbClient,
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

    pub async fn retrieve(&self, query: &str) -> anyhow::Result<RAGContext> {
        let query_embedding = self.embedding_worker.embed_text(query).await?;
        let search_results = self
            .vector_db
            .search(&query_embedding.data, self.max_context_chunks);

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
        self.vector_db.upsert(&embedding, metadata)?;
        Ok(())
    }
}

fn estimate_tokens(text: &str) -> usize {
    (text.len() as f32 / 4.0).ceil() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vectordb::{DistanceMetric, VectorDbConfig};

    fn make_rag() -> RAGPipeline {
        let worker = EmbeddingWorker::new(16);
        let db = VectorDbClient::new(VectorDbConfig {
            collection_name: "test".into(),
            dimension: 16,
            distance_metric: DistanceMetric::Cosine,
        });
        RAGPipeline::new(worker, db, 5, 0.0)
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
}
