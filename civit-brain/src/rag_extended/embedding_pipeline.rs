#![forbid(unsafe_code)]

use crate::ast::AstNode;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub enum EmbeddingStrategy {
    MeanPooling,
    MaxPooling,
    ClsToken,
    FirstToken,
}

#[derive(Debug, Clone)]
pub struct EmbeddingPipelineConfig {
    pub model_id: String,
    pub dimension: usize,
    pub batch_size: usize,
    pub max_seq_length: usize,
    pub pooling_strategy: EmbeddingStrategy,
}

impl Default for EmbeddingPipelineConfig {
    fn default() -> Self {
        Self {
            model_id: "default-embedding".into(),
            dimension: 128,
            batch_size: 32,
            max_seq_length: 512,
            pooling_strategy: EmbeddingStrategy::MeanPooling,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmbeddingRequest {
    pub input_ids: Vec<u32>,
    pub attention_mask: Vec<u32>,
    pub token_type_ids: Vec<u32>,
    pub input_text: String,
}

impl EmbeddingRequest {
    pub fn from_text(text: &str) -> Self {
        let input_ids: Vec<u32> = text.bytes().map(|b| b as u32).collect();
        Self {
            attention_mask: vec![1; input_ids.len()],
            token_type_ids: vec![0; input_ids.len()],
            input_ids,
            input_text: text.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmbeddingResult {
    pub embedding: Vec<f32>,
    pub dimension: usize,
    pub tokens_used: usize,
}

#[derive(Debug, Clone)]
pub struct EmbeddingBatch {
    pub requests: Vec<EmbeddingRequest>,
    pub results: Vec<EmbeddingResult>,
}

impl EmbeddingBatch {
    pub fn new(requests: Vec<EmbeddingRequest>) -> Self {
        Self {
            requests,
            results: Vec::new(),
        }
    }

    pub fn with_results(requests: Vec<EmbeddingRequest>, results: Vec<EmbeddingResult>) -> Self {
        Self { requests, results }
    }

    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }

    pub fn len(&self) -> usize {
        self.requests.len()
    }
}

pub trait EmbeddingModel: Send + Sync {
    fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>>;
    fn embed_batch(&self, texts: &[&str]) -> anyhow::Result<Vec<Vec<f32>>>;
}

pub struct StubEmbeddingModel {
    dimension: usize,
}

impl StubEmbeddingModel {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

impl EmbeddingModel for StubEmbeddingModel {
    fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let hash = hash_text(text);
        let mut embedding = Vec::with_capacity(self.dimension);
        for i in 0..self.dimension {
            let mut hasher = DefaultHasher::new();
            hash.hash(&mut hasher);
            (i as u64).hash(&mut hasher);
            let val = (hasher.finish() % 1000) as f32 / 1000.0;
            embedding.push(val);
        }
        Ok(embedding)
    }

    fn embed_batch(&self, texts: &[&str]) -> anyhow::Result<Vec<Vec<f32>>> {
        texts.iter().map(|t| self.embed(t)).collect()
    }
}

pub struct EmbeddingPipeline {
    pub config: EmbeddingPipelineConfig,
    model: Box<dyn EmbeddingModel>,
}

impl EmbeddingPipeline {
    pub fn new(config: EmbeddingPipelineConfig, model: Box<dyn EmbeddingModel>) -> Self {
        Self { config, model }
    }

    pub fn with_stub(config: EmbeddingPipelineConfig) -> Self {
        let model = Box::new(StubEmbeddingModel::new(config.dimension));
        Self { config, model }
    }

    pub fn embed_text(&self, text: &str) -> anyhow::Result<EmbeddingResult> {
        let embedding = self.model.embed(text)?;
        let tokens_used = (text.len() as f32 / 4.0).ceil() as usize;
        Ok(EmbeddingResult {
            dimension: embedding.len(),
            tokens_used,
            embedding,
        })
    }

    pub fn embed_ast_node(&self, node: &AstNode) -> EmbeddingResult {
        let text = format_ast_node(node);
        let embedding = self
            .model
            .embed(&text)
            .unwrap_or_else(|_| vec![0.0; self.config.dimension]);
        let tokens_used = (text.len() as f32 / 4.0).ceil() as usize;
        EmbeddingResult {
            dimension: embedding.len(),
            tokens_used,
            embedding,
        }
    }

    pub fn embed_documentation(&self, doc: &str) -> EmbeddingResult {
        let embedding = self
            .model
            .embed(doc)
            .unwrap_or_else(|_| vec![0.0; self.config.dimension]);
        let tokens_used = (doc.len() as f32 / 4.0).ceil() as usize;
        EmbeddingResult {
            dimension: embedding.len(),
            tokens_used,
            embedding,
        }
    }

    pub fn embed_batch_items(&self, items: Vec<&str>) -> EmbeddingBatch {
        let requests: Vec<EmbeddingRequest> = items
            .iter()
            .map(|t| EmbeddingRequest::from_text(t))
            .collect();
        let texts: Vec<&str> = items.to_vec();
        let results = self
            .model
            .embed_batch(&texts)
            .unwrap_or_default()
            .into_iter()
            .map(|embedding| EmbeddingResult {
                dimension: embedding.len(),
                tokens_used: 0,
                embedding,
            })
            .collect();
        EmbeddingBatch::with_results(requests, results)
    }
}

fn hash_text(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

fn format_ast_node(node: &AstNode) -> String {
    format!(
        "{:?} {} (lines {}-{}, {} children, complexity: {:?})",
        node.kind,
        node.name,
        node.start_line,
        node.end_line,
        node.children.len(),
        node.complexity
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::NodeKind;

    fn test_config() -> EmbeddingPipelineConfig {
        EmbeddingPipelineConfig {
            model_id: "test-model".into(),
            dimension: 16,
            batch_size: 8,
            max_seq_length: 128,
            pooling_strategy: EmbeddingStrategy::MeanPooling,
        }
    }

    fn make_node(name: &str, kind: NodeKind) -> AstNode {
        AstNode {
            id: 1,
            kind,
            name: name.into(),
            start_line: 1,
            end_line: 10,
            children: Vec::new(),
            complexity: Some(2.5),
        }
    }

    #[test]
    fn test_embedding_strategy_variants() {
        let _ = EmbeddingStrategy::MeanPooling;
        let _ = EmbeddingStrategy::MaxPooling;
        let _ = EmbeddingStrategy::ClsToken;
        let _ = EmbeddingStrategy::FirstToken;
    }

    #[test]
    fn test_pipeline_config_default() {
        let config = EmbeddingPipelineConfig::default();
        assert_eq!(config.model_id, "default-embedding");
        assert_eq!(config.dimension, 128);
        assert_eq!(config.batch_size, 32);
        assert_eq!(config.max_seq_length, 512);
    }

    #[test]
    fn test_embedding_request_from_text() {
        let req = EmbeddingRequest::from_text("hello world");
        assert_eq!(req.input_text, "hello world");
        assert_eq!(req.input_ids.len(), "hello world".len());
        assert!(req.attention_mask.iter().all(|&m| m == 1));
        assert!(req.token_type_ids.iter().all(|&t| t == 0));
    }

    #[test]
    fn test_embedding_request_from_empty_text() {
        let req = EmbeddingRequest::from_text("");
        assert!(req.input_ids.is_empty());
        assert!(req.attention_mask.is_empty());
    }

    #[test]
    fn test_embedding_batch_new() {
        let batch = EmbeddingBatch::new(vec![EmbeddingRequest::from_text("a")]);
        assert_eq!(batch.len(), 1);
        assert!(!batch.is_empty());
        assert!(batch.results.is_empty());
    }

    #[test]
    fn test_embedding_batch_with_results() {
        let batch = EmbeddingBatch::with_results(
            vec![EmbeddingRequest::from_text("a")],
            vec![EmbeddingResult {
                embedding: vec![0.1],
                dimension: 1,
                tokens_used: 1,
            }],
        );
        assert_eq!(batch.results.len(), 1);
    }

    #[test]
    fn test_embedding_batch_empty() {
        let batch = EmbeddingBatch::new(vec![]);
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
    }

    #[test]
    fn test_stub_embedding_model_embed() {
        let model = StubEmbeddingModel::new(8);
        let result = model.embed("hello").unwrap();
        assert_eq!(result.len(), 8);
        assert!(result.iter().all(|&v| (0.0..=1.0).contains(&v)));
    }

    #[test]
    fn test_stub_embedding_model_deterministic() {
        let model = StubEmbeddingModel::new(16);
        let a = model.embed("deterministic").unwrap();
        let b = model.embed("deterministic").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn test_stub_embedding_model_different_inputs() {
        let model = StubEmbeddingModel::new(16);
        let a = model.embed("alpha").unwrap();
        let b = model.embed("beta").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn test_stub_embedding_model_batch() {
        let model = StubEmbeddingModel::new(8);
        let results = model.embed_batch(&["a", "b", "c"]).unwrap();
        assert_eq!(results.len(), 3);
        for r in &results {
            assert_eq!(r.len(), 8);
        }
    }

    #[test]
    fn test_pipeline_embed_text() {
        let pipeline = EmbeddingPipeline::with_stub(test_config());
        let result = pipeline.embed_text("test document").unwrap();
        assert_eq!(result.dimension, 16);
        assert!(result.tokens_used > 0);
    }

    #[test]
    fn test_pipeline_embed_ast_node() {
        let pipeline = EmbeddingPipeline::with_stub(test_config());
        let node = make_node("main", NodeKind::Function);
        let result = pipeline.embed_ast_node(&node);
        assert_eq!(result.dimension, 16);
        assert!(result.tokens_used > 0);
    }

    #[test]
    fn test_pipeline_embed_documentation() {
        let pipeline = EmbeddingPipeline::with_stub(test_config());
        let result = pipeline.embed_documentation("This function parses a request.");
        assert_eq!(result.dimension, 16);
        assert!(result.tokens_used > 0);
    }

    #[test]
    fn test_pipeline_embed_documentation_empty() {
        let pipeline = EmbeddingPipeline::with_stub(test_config());
        let result = pipeline.embed_documentation("");
        assert_eq!(result.dimension, 16);
        assert_eq!(result.tokens_used, 0);
    }

    #[test]
    fn test_pipeline_embed_batch_items() {
        let pipeline = EmbeddingPipeline::with_stub(test_config());
        let items = vec!["hello", "world", "test"];
        let batch = pipeline.embed_batch_items(items);
        assert_eq!(batch.requests.len(), 3);
        assert_eq!(batch.results.len(), 3);
    }

    #[test]
    fn test_pipeline_embed_batch_items_empty() {
        let pipeline = EmbeddingPipeline::with_stub(test_config());
        let batch = pipeline.embed_batch_items(vec![]);
        assert!(batch.is_empty());
    }

    #[test]
    fn test_pipeline_with_custom_model() {
        let config = test_config();
        let pipeline = EmbeddingPipeline::new(config, Box::new(StubEmbeddingModel::new(16)));
        let result = pipeline.embed_text("custom model test").unwrap();
        assert_eq!(result.dimension, 16);
    }

    #[test]
    fn test_format_ast_node() {
        let node = make_node("parse_request", NodeKind::Function);
        let text = format_ast_node(&node);
        assert!(text.contains("parse_request"));
        assert!(text.contains("Function"));
        assert!(text.contains("1-10"));
    }

    #[test]
    fn test_hash_text_deterministic() {
        assert_eq!(hash_text("same"), hash_text("same"));
        assert_ne!(hash_text("a"), hash_text("b"));
    }
}
