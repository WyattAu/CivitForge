#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    pub max_tokens: u32,
    pub max_chunks: usize,
    pub overlap_ratio: f32,
    pub summary_threshold: usize,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_tokens: 4096,
            max_chunks: 20,
            overlap_ratio: 0.1,
            summary_threshold: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkSource {
    pub file_path: String,
    pub entity_type: String,
    pub entity_name: String,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextChunk {
    pub id: String,
    pub content: String,
    pub tokens: usize,
    pub relevance: f32,
    pub source: ChunkSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindow {
    pub chunks: Vec<ContextChunk>,
    pub total_tokens: usize,
    pub query: String,
    max_tokens: usize,
}

impl ContextWindow {
    pub fn new(query: String, max_tokens: u32) -> Self {
        Self {
            chunks: Vec::new(),
            total_tokens: 0,
            query,
            max_tokens: max_tokens as usize,
        }
    }

    pub fn add_chunk(&mut self, chunk: ContextChunk) -> bool {
        if self.total_tokens + chunk.tokens > self.max_tokens {
            return false;
        }
        self.total_tokens += chunk.tokens;
        self.chunks.push(chunk);
        true
    }

    pub fn remaining_capacity(&self) -> usize {
        self.max_tokens.saturating_sub(self.total_tokens)
    }

    pub fn is_full(&self) -> bool {
        self.total_tokens >= self.max_tokens
    }

    pub fn to_prompt(&self) -> String {
        let mut prompt = String::from("Codebase context:\n\n");
        for chunk in &self.chunks {
            prompt.push_str(&format!(
                "[{}:{}] {} ({}) lines {}-{}\n{}\n\n",
                chunk.source.file_path,
                chunk.id,
                chunk.source.entity_name,
                chunk.source.entity_type,
                chunk.source.start_line,
                chunk.source.end_line,
                chunk.content,
            ));
        }
        prompt
    }

    pub fn summarize(&self) -> String {
        if self.chunks.is_empty() {
            return String::new();
        }
        if self.chunks.len() <= 3 {
            let parts: Vec<String> = self
                .chunks
                .iter()
                .map(|c| format!("{}:{} {}", c.source.file_path, c.source.entity_name, &c.content[..c.content.len().min(80)]))
                .collect();
            return parts.join("\n");
        }
        let files: Vec<String> = self
            .chunks
            .iter()
            .map(|c| format!("{}:{}", c.source.file_path, c.source.entity_name))
            .collect();
        let unique_files: std::collections::HashSet<&String> = files.iter().collect();
        let mut summary = format!("{} relevant code chunks from {} locations:\n", self.chunks.len(), unique_files.len());
        for f in &files {
            summary.push_str(&format!("  - {}\n", f));
        }
        summary
    }
}

fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    (text.len() as f32 / 4.0).ceil() as usize
}

pub fn build_code_review_prompt(context: &ContextWindow, diff: &str) -> String {
    let mut prompt = String::from("You are a senior code reviewer. Review the following diff with the provided codebase context.\n\n");
    prompt.push_str("## Codebase Context\n\n");
    prompt.push_str(&context.to_prompt());
    prompt.push_str("## Diff to Review\n\n");
    prompt.push_str(diff);
    prompt.push_str("\n\nProvide specific, actionable feedback. Focus on correctness, security, performance, and code style.\n");
    prompt
}

pub fn build_architecture_query_prompt(context: &ContextWindow, query: &str) -> String {
    let mut prompt = String::from("You are a codebase architect assistant. Answer questions about the codebase structure.\n\n");
    prompt.push_str("## Relevant Code\n\n");
    prompt.push_str(&context.to_prompt());
    prompt.push_str(&format!("## Question\n\n{}\n\n", query));
    prompt.push_str("Provide a clear, structured answer referencing specific files and line numbers.\n");
    prompt
}

pub fn create_chunk(
    id: &str,
    content: &str,
    relevance: f32,
    file_path: &str,
    entity_type: &str,
    entity_name: &str,
    start_line: usize,
    end_line: usize,
) -> ContextChunk {
    ContextChunk {
        id: id.into(),
        content: content.into(),
        tokens: estimate_tokens(content),
        relevance,
        source: ChunkSource {
            file_path: file_path.into(),
            entity_type: entity_type.into(),
            entity_name: entity_name.into(),
            start_line,
            end_line,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_config_default() {
        let config = ContextConfig::default();
        assert_eq!(config.max_tokens, 4096);
        assert_eq!(config.max_chunks, 20);
        assert!((config.overlap_ratio - 0.1).abs() < 0.001);
        assert_eq!(config.summary_threshold, 5);
    }

    #[test]
    fn test_add_chunk() {
        let mut window = ContextWindow::new("test query".into(), 1000);
        let chunk = create_chunk("c1", "fn main() {}", 0.9, "src/main.rs", "Function", "main", 1, 3);
        assert!(window.add_chunk(chunk));
        assert_eq!(window.chunks.len(), 1);
        assert!(!window.is_full());
    }

    #[test]
    fn test_add_chunk_exceeds_capacity() {
        let mut window = ContextWindow::new("test".into(), 5);
        let chunk = create_chunk("c1", "hello world this is long text", 0.9, "f.rs", "F", "f", 1, 2);
        assert!(!window.add_chunk(chunk));
        assert_eq!(window.chunks.len(), 0);
    }

    #[test]
    fn test_remaining_capacity() {
        let mut window = ContextWindow::new("test".into(), 1000);
        window.add_chunk(create_chunk("c1", "abc", 0.9, "f.rs", "F", "f", 1, 2));
        let remaining = window.remaining_capacity();
        assert!(remaining < 1000);
        assert!(remaining > 0);
    }

    #[test]
    fn test_to_prompt() {
        let mut window = ContextWindow::new("test".into(), 4096);
        window.add_chunk(create_chunk("c1", "fn main() {}", 0.9, "src/main.rs", "Function", "main", 1, 3));
        let prompt = window.to_prompt();
        assert!(prompt.contains("src/main.rs"));
        assert!(prompt.contains("main"));
    }

    #[test]
    fn test_summarize_empty() {
        let window = ContextWindow::new("test".into(), 4096);
        assert!(window.summarize().is_empty());
    }

    #[test]
    fn test_summarize_few_chunks() {
        let mut window = ContextWindow::new("test".into(), 4096);
        window.add_chunk(create_chunk("c1", "fn main() {}", 0.9, "src/main.rs", "Function", "main", 1, 3));
        window.add_chunk(create_chunk("c2", "fn helper() {}", 0.8, "src/util.rs", "Function", "helper", 5, 7));
        let summary = window.summarize();
        assert!(summary.contains("src/main.rs"));
        assert!(summary.contains("src/util.rs"));
    }

    #[test]
    fn test_build_code_review_prompt() {
        let mut window = ContextWindow::new("review".into(), 4096);
        window.add_chunk(create_chunk("c1", "fn main() {}", 0.9, "src/main.rs", "Function", "main", 1, 3));
        let prompt = build_code_review_prompt(&window, "+let x = 5;");
        assert!(prompt.contains("code reviewer"));
        assert!(prompt.contains("+let x = 5;"));
        assert!(prompt.contains("Codebase Context"));
    }

    #[test]
    fn test_build_architecture_query_prompt() {
        let window = ContextWindow::new("arch".into(), 4096);
        let prompt = build_architecture_query_prompt(&window, "where is auth handled?");
        assert!(prompt.contains("architect"));
        assert!(prompt.contains("where is auth handled?"));
    }

    #[test]
    fn test_chunk_source() {
        let source = ChunkSource {
            file_path: "src/main.rs".into(),
            entity_type: "Function".into(),
            entity_name: "main".into(),
            start_line: 1,
            end_line: 10,
        };
        let json = serde_json::to_string(&source).unwrap();
        let de: ChunkSource = serde_json::from_str(&json).unwrap();
        assert_eq!(de.file_path, "src/main.rs");
    }
}
