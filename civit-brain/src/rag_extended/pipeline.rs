#![forbid(unsafe_code)]

use crate::rag_extended::context::{ContextChunk, ContextConfig};
use crate::rag_extended::conversation::{ConversationHistory, TokenBudget};
use serde::{Deserialize, Serialize};

/// Query type for routing to appropriate retrieval strategies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryType {
    /// "What does function X do?" -- retrieve specific code entities
    CodeUnderstanding,
    /// "Where are all database writes?" -- retrieve architectural paths
    ArchitectureQuery,
    /// "Fix this bug in function Y" -- retrieve context around a specific location
    BugFix,
    /// "Refactor module Z" -- retrieve module structure and dependencies
    Refactoring,
    /// General conversational query
    General,
}

/// Parsed query with extracted intent and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedQuery {
    pub raw: String,
    pub query_type: QueryType,
    pub mentioned_files: Vec<String>,
    pub mentioned_symbols: Vec<String>,
    pub mentioned_languages: Vec<String>,
}

impl ParsedQuery {
    pub fn parse(raw: impl Into<String>) -> Self {
        let raw_str = raw.into();
        let lower = raw_str.to_lowercase();
        let query_type = Self::classify(&lower);
        let mentioned_files =
            Self::extract_mentions(&lower, &["file", "in ", " at ", ".rs", ".py", ".go", ".ts"]);
        let mentioned_symbols =
            Self::extract_mentions(&lower, &["function", "struct", "class", "method", "fn "]);
        let mentioned_languages = Self::extract_language_mentions(&lower);
        Self {
            raw: raw_str,
            query_type,
            mentioned_files,
            mentioned_symbols,
            mentioned_languages,
        }
    }

    fn classify(text: &str) -> QueryType {
        if text.contains("where")
            && (text.contains("all") || text.contains("write") || text.contains("read"))
        {
            return QueryType::ArchitectureQuery;
        }
        if text.contains("fix") || text.contains("bug") || text.contains("error") {
            return QueryType::BugFix;
        }
        if text.contains("refactor") || text.contains("restructure") || text.contains("reorganize")
        {
            return QueryType::Refactoring;
        }
        if text.contains("what does") || text.contains("how does") || text.contains("explain") {
            return QueryType::CodeUnderstanding;
        }
        QueryType::General
    }

    fn extract_mentions(text: &str, triggers: &[&str]) -> Vec<String> {
        let mut mentions = Vec::new();
        for trigger in triggers {
            if text.contains(trigger) {
                // Extract the word(s) after the trigger
                if let Some(pos) = text.find(trigger) {
                    let after = text[pos + trigger.len()..].trim();
                    let word: String = after
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '.')
                        .collect();
                    if !word.is_empty() && !mentions.contains(&word) {
                        mentions.push(word);
                    }
                }
            }
        }
        mentions
    }

    fn extract_language_mentions(text: &str) -> Vec<String> {
        let languages = [
            "rust",
            "python",
            "go",
            "typescript",
            "javascript",
            "java",
            "c++",
            "kotlin",
            "swift",
        ];
        languages
            .iter()
            .filter(|l| text.contains(**l))
            .map(|l| (*l).to_string())
            .collect()
    }
}

/// Retrieval strategy configuration for different query types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalConfig {
    pub top_k: usize,
    pub min_relevance: f32,
    pub include_callers: bool,
    pub include_callees: bool,
    pub include_tests: bool,
    pub context_lines_before: usize,
    pub context_lines_after: usize,
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            top_k: 10,
            min_relevance: 0.5,
            include_callers: false,
            include_callees: false,
            include_tests: false,
            context_lines_before: 5,
            context_lines_after: 5,
        }
    }
}

impl RetrievalConfig {
    pub fn for_query_type(query_type: &QueryType) -> Self {
        match query_type {
            QueryType::ArchitectureQuery => Self {
                top_k: 20,
                min_relevance: 0.3,
                include_callers: true,
                include_callees: true,
                include_tests: false,
                context_lines_before: 3,
                context_lines_after: 3,
            },
            QueryType::BugFix => Self {
                top_k: 15,
                min_relevance: 0.4,
                include_callers: true,
                include_callees: true,
                include_tests: true,
                context_lines_before: 10,
                context_lines_after: 10,
            },
            QueryType::Refactoring => Self {
                top_k: 15,
                min_relevance: 0.4,
                include_callers: true,
                include_callees: true,
                include_tests: true,
                context_lines_before: 5,
                context_lines_after: 5,
            },
            QueryType::CodeUnderstanding => Self {
                top_k: 10,
                min_relevance: 0.5,
                include_callers: true,
                include_callees: false,
                include_tests: false,
                context_lines_before: 5,
                context_lines_after: 5,
            },
            QueryType::General => Self::default(),
        }
    }
}

/// Result of a retrieval operation containing ranked chunks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    pub chunks: Vec<ContextChunk>,
    pub query_type: QueryType,
    pub total_candidates: usize,
    pub retrieval_latency_ms: u64,
}

impl RetrievalResult {
    pub fn top_chunks(&self, n: usize) -> Vec<&ContextChunk> {
        self.chunks.iter().take(n).collect()
    }

    pub fn unique_files(&self) -> Vec<String> {
        let mut files: Vec<String> = self
            .chunks
            .iter()
            .map(|c| c.source.file_path.clone())
            .collect();
        files.sort();
        files.dedup();
        files
    }

    pub fn total_tokens(&self) -> usize {
        self.chunks.iter().map(|c| c.tokens).sum()
    }
}

/// Mock retriever for testing and development without a real vector DB.
#[derive(Debug, Clone)]
pub struct MockRetriever {
    chunks: Vec<ContextChunk>,
}

impl MockRetriever {
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    pub fn with_chunks(chunks: Vec<ContextChunk>) -> Self {
        Self { chunks }
    }

    pub fn add_chunk(&mut self, chunk: ContextChunk) {
        self.chunks.push(chunk);
    }

    /// Simple keyword-based retrieval that simulates semantic search.
    pub fn retrieve(&self, query: &ParsedQuery, config: &RetrievalConfig) -> RetrievalResult {
        let start = std::time::Instant::now();
        let query_lower = query.raw.to_lowercase();
        let mut scored: Vec<(usize, f32)> = Vec::new();

        for (idx, chunk) in self.chunks.iter().enumerate() {
            let mut score = 0.0f32;
            let chunk_lower = chunk.content.to_lowercase();
            // Keyword overlap scoring
            let query_words: Vec<&str> = query_lower.split_whitespace().collect();
            let chunk_words: std::collections::HashSet<&str> =
                chunk_lower.split_whitespace().collect();
            let matches: usize = query_words
                .iter()
                .filter(|w| chunk_words.contains(*w))
                .count();
            if !query_words.is_empty() {
                score += (matches as f32) / (query_words.len() as f32);
            }
            // File path bonus
            for file in &query.mentioned_files {
                if chunk.source.file_path.to_lowercase().contains(file) {
                    score += 0.3;
                }
            }
            // Symbol bonus
            for symbol in &query.mentioned_symbols {
                if chunk.source.entity_name.to_lowercase().contains(symbol) {
                    score += 0.4;
                }
            }
            if score >= config.min_relevance {
                scored.push((idx, score));
            }
        }
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let top_k = scored.into_iter().take(config.top_k);
        let chunks: Vec<ContextChunk> = top_k
            .map(|(idx, score)| {
                let mut c = self.chunks[idx].clone();
                c.relevance = c.relevance.max(score);
                c
            })
            .collect();
        let total_candidates = self.chunks.len();
        let retrieval_latency_ms = start.elapsed().as_millis() as u64;
        RetrievalResult {
            chunks,
            query_type: query.query_type.clone(),
            total_candidates,
            retrieval_latency_ms,
        }
    }
}

impl Default for MockRetriever {
    fn default() -> Self {
        Self::new()
    }
}

/// Orchestrates retrieval, context window management, and history for multi-turn RAG.
#[derive(Debug)]
pub struct RagOrchestrator {
    context_config: ContextConfig,
    retriever: MockRetriever,
}

impl RagOrchestrator {
    pub fn new(context_config: ContextConfig) -> Self {
        Self {
            context_config,
            retriever: MockRetriever::new(),
        }
    }

    pub fn with_retriever(mut self, retriever: MockRetriever) -> Self {
        self.retriever = retriever;
        self
    }

    pub fn add_retrievable_chunk(&mut self, chunk: ContextChunk) {
        self.retriever.add_chunk(chunk);
    }

    /// Processes a query: parse intent, retrieve context, manage token budget.
    pub fn process_query(
        &self,
        query_text: &str,
        history: &ConversationHistory,
    ) -> RagPipelineResult {
        let query = ParsedQuery::parse(query_text);
        let retrieval_config = RetrievalConfig::for_query_type(&query.query_type);
        let retrieval = self.retriever.retrieve(&query, &retrieval_config);
        // Estimate system prompt tokens
        let system_tokens = 200;
        let history_tokens = history.total_tokens;
        let budget = TokenBudget::new(self.context_config.max_tokens as usize, 0.2);
        // Build context window respecting budget
        let context_tokens = budget.remaining_for_context();
        let mut context_window = crate::rag_extended::context::ContextWindow::new(
            query_text.to_string(),
            context_tokens as u32,
        );
        let mut tokens_used = 0usize;
        for chunk in &retrieval.chunks {
            if tokens_used + chunk.tokens > context_tokens {
                break;
            }
            context_window.add_chunk(chunk.clone());
            tokens_used += chunk.tokens;
        }
        RagPipelineResult {
            query,
            retrieval,
            context_prompt: context_window.to_prompt(),
            context_summary: context_window.summarize(),
            chunks_used: context_window.chunks.len(),
            tokens_used: tokens_used + system_tokens + history_tokens,
            within_budget: !budget.is_over_budget(),
        }
    }
}

/// Complete result from the RAG pipeline ready for LLM inference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagPipelineResult {
    pub query: ParsedQuery,
    pub retrieval: RetrievalResult,
    pub context_prompt: String,
    pub context_summary: String,
    pub chunks_used: usize,
    pub tokens_used: usize,
    pub within_budget: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag_extended::context::create_chunk;

    fn make_chunk(id: &str, content: &str, file: &str, name: &str) -> ContextChunk {
        create_chunk(id, content, 0.8, file, "Function", name, 1, 10)
    }

    #[test]
    fn test_parsed_query_code_understanding() {
        let q = ParsedQuery::parse("What does the main function do?");
        assert_eq!(q.query_type, QueryType::CodeUnderstanding);
    }

    #[test]
    fn test_parsed_query_architecture() {
        let q = ParsedQuery::parse("Where are all database write paths?");
        assert_eq!(q.query_type, QueryType::ArchitectureQuery);
    }

    #[test]
    fn test_parsed_query_bug_fix() {
        let q = ParsedQuery::parse("Fix the bug in authenticate");
        assert_eq!(q.query_type, QueryType::BugFix);
    }

    #[test]
    fn test_parsed_query_refactoring() {
        let q = ParsedQuery::parse("Refactor the auth module");
        assert_eq!(q.query_type, QueryType::Refactoring);
    }

    #[test]
    fn test_parsed_query_general() {
        let q = ParsedQuery::parse("Hello, how are you?");
        assert_eq!(q.query_type, QueryType::General);
    }

    #[test]
    fn test_parsed_query_mentions_files() {
        let q = ParsedQuery::parse("What is in file main.rs");
        assert!(!q.mentioned_files.is_empty());
    }

    #[test]
    fn test_parsed_query_mentions_symbols() {
        let q = ParsedQuery::parse("Explain the function authenticate");
        assert!(!q.mentioned_symbols.is_empty());
    }

    #[test]
    fn test_parsed_query_mentions_languages() {
        let q = ParsedQuery::parse("Show me Rust code examples");
        assert!(!q.mentioned_languages.is_empty());
        assert!(q.mentioned_languages.contains(&"rust".to_string()));
    }

    #[test]
    fn test_retrieval_config_for_query_type() {
        let config = RetrievalConfig::for_query_type(&QueryType::ArchitectureQuery);
        assert_eq!(config.top_k, 20);
        assert!(config.include_callers);
        assert!(config.include_callees);
    }

    #[test]
    fn test_retrieval_config_default() {
        let config = RetrievalConfig::default();
        assert_eq!(config.top_k, 10);
        assert!(!config.include_callers);
    }

    #[test]
    fn test_retrieval_result_top_chunks() {
        let result = RetrievalResult {
            chunks: vec![
                make_chunk("c1", "content1", "f1.rs", "fn1"),
                make_chunk("c2", "content2", "f2.rs", "fn2"),
                make_chunk("c3", "content3", "f3.rs", "fn3"),
            ],
            query_type: QueryType::General,
            total_candidates: 10,
            retrieval_latency_ms: 5,
        };
        assert_eq!(result.top_chunks(2).len(), 2);
    }

    #[test]
    fn test_retrieval_result_unique_files() {
        let result = RetrievalResult {
            chunks: vec![
                make_chunk("c1", "a", "src/main.rs", "main"),
                make_chunk("c2", "b", "src/main.rs", "helper"),
                make_chunk("c3", "c", "src/lib.rs", "init"),
            ],
            query_type: QueryType::General,
            total_candidates: 3,
            retrieval_latency_ms: 1,
        };
        let files = result.unique_files();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_retrieval_result_total_tokens() {
        let result = RetrievalResult {
            chunks: vec![make_chunk("c1", "hello world", "f.rs", "fn1")],
            query_type: QueryType::General,
            total_candidates: 1,
            retrieval_latency_ms: 1,
        };
        assert!(result.total_tokens() > 0);
    }

    #[test]
    fn test_mock_retriever_basic() {
        let mut retriever = MockRetriever::new();
        retriever.add_chunk(make_chunk(
            "c1",
            "how does authenticate user verification work function",
            "auth.rs",
            "authenticate",
        ));
        retriever.add_chunk(make_chunk(
            "c2",
            "connect database function",
            "db.rs",
            "connect_database",
        ));
        let query = ParsedQuery::parse("How does authenticate work?");
        let config = RetrievalConfig {
            min_relevance: 0.2,
            top_k: 10,
            ..Default::default()
        };
        let result = retriever.retrieve(&query, &config);
        // The word "authenticate" and "work" and "does" should match
        assert!(!result.chunks.is_empty());
    }

    #[test]
    fn test_mock_retriever_respects_top_k() {
        let mut retriever = MockRetriever::new();
        for i in 0..20 {
            retriever.add_chunk(make_chunk(
                &format!("c{i}"),
                "test function code here",
                "test.rs",
                &format!("fn{i}"),
            ));
        }
        let query = ParsedQuery::parse("test function code");
        let config = RetrievalConfig {
            top_k: 5,
            ..Default::default()
        };
        let result = retriever.retrieve(&query, &config);
        assert!(result.chunks.len() <= 5);
    }

    #[test]
    fn test_mock_retriever_respects_min_relevance() {
        let mut retriever = MockRetriever::new();
        retriever.add_chunk(make_chunk(
            "c1",
            "completely unrelated content about gardening",
            "garden.rs",
            "plant",
        ));
        let query = ParsedQuery::parse("database authentication system");
        let config = RetrievalConfig {
            min_relevance: 0.8,
            ..Default::default()
        };
        let result = retriever.retrieve(&query, &config);
        assert!(result.chunks.is_empty());
    }

    #[test]
    fn test_rag_orchestrator_process_query() {
        let mut orchestrator = RagOrchestrator::new(ContextConfig::default());
        orchestrator.add_retrievable_chunk(make_chunk(
            "c1",
            "fn main() { println!(\"hello\"); }",
            "main.rs",
            "main",
        ));
        orchestrator.add_retrievable_chunk(make_chunk(
            "c2",
            "fn auth() { verify(user); }",
            "auth.rs",
            "auth",
        ));
        let history = ConversationHistory::new("test", 10, 1000);
        let result = orchestrator.process_query("What does main do?", &history);
        assert_eq!(result.query.query_type, QueryType::CodeUnderstanding);
        assert!(result.within_budget);
    }

    #[test]
    fn test_rag_orchestrator_context_limiting() {
        let mut orchestrator = RagOrchestrator::new(ContextConfig {
            max_tokens: 100,
            max_chunks: 2,
            ..Default::default()
        });
        // Add large chunk that will exceed budget
        let large_content = "x".repeat(500);
        orchestrator.add_retrievable_chunk(make_chunk("c1", &large_content, "big.rs", "bigfn"));
        let history = ConversationHistory::new("test", 10, 1000);
        let result = orchestrator.process_query("explain bigfn", &history);
        // Budget should limit chunks used
        assert!(result.chunks_used <= 1);
    }

    #[test]
    fn test_parsed_query_serialization() {
        let q = ParsedQuery::parse("What does main do?");
        let json = serde_json::to_string(&q).unwrap();
        let de: ParsedQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(de.raw, "What does main do?");
    }

    #[test]
    fn test_rag_pipeline_result_serialization() {
        let result = RagPipelineResult {
            query: ParsedQuery::parse("test"),
            retrieval: RetrievalResult {
                chunks: Vec::new(),
                query_type: QueryType::General,
                total_candidates: 0,
                retrieval_latency_ms: 0,
            },
            context_prompt: "prompt".into(),
            context_summary: "summary".into(),
            chunks_used: 0,
            tokens_used: 100,
            within_budget: true,
        };
        let json = serde_json::to_string(&result).unwrap();
        let de: RagPipelineResult = serde_json::from_str(&json).unwrap();
        assert!(de.within_budget);
    }
}
