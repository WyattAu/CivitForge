#![forbid(unsafe_code)]

use crate::llm::inference::{InferenceRequest, InferenceService};
use crate::rag_extended::context::{ContextChunk, ContextConfig, ContextWindow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub sources: Vec<crate::rag_extended::context::ChunkSource>,
}

pub struct ChatSession {
    pub id: Uuid,
    pub messages: Vec<ChatMessage>,
    pub context_window: ContextWindow,
    pub max_history: usize,
}

impl ChatSession {
    pub fn new(query: String, max_tokens: u32, max_history: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            messages: Vec::new(),
            context_window: ContextWindow::new(query, max_tokens),
            max_history,
        }
    }

    pub fn add_user_message(&mut self, content: String) {
        let msg = ChatMessage {
            role: ChatRole::User,
            content,
            timestamp: Utc::now(),
            sources: Vec::new(),
        };
        self.messages.push(msg);
        self.trim_history();
    }

    pub fn add_assistant_response(
        &mut self,
        content: String,
        sources: Vec<crate::rag_extended::context::ChunkSource>,
    ) {
        let msg = ChatMessage {
            role: ChatRole::Assistant,
            content,
            timestamp: Utc::now(),
            sources,
        };
        self.messages.push(msg);
        self.trim_history();
    }

    pub fn get_history(&self) -> Vec<&ChatMessage> {
        self.messages.iter().collect()
    }

    pub fn summarize_history(&self) -> String {
        if self.messages.is_empty() {
            return String::new();
        }
        let user_msgs: Vec<&str> = self
            .messages
            .iter()
            .filter(|m| matches!(m.role, ChatRole::User))
            .map(|m| m.content.as_str())
            .collect();
        format!(
            "Conversation with {} messages ({} user queries). Topics: {}",
            self.messages.len(),
            user_msgs.len(),
            user_msgs.join(", ")
        )
    }

    pub fn clear_history(&mut self) {
        self.messages.clear();
    }

    fn trim_history(&mut self) {
        while self.messages.len() > self.max_history {
            self.messages.remove(0);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatQueryResult {
    pub response: String,
    pub sources: Vec<crate::rag_extended::context::ChunkSource>,
    pub confidence: f32,
}

pub struct CodebaseChat {
    pub inference: InferenceService,
    pub context_manager: ContextConfig,
}

impl CodebaseChat {
    pub fn new(inference: InferenceService, context_manager: ContextConfig) -> Self {
        Self {
            inference,
            context_manager,
        }
    }

    pub fn query(
        &self,
        session: &mut ChatSession,
        query: &str,
        codebase_context: &[ContextChunk],
    ) -> anyhow::Result<ChatQueryResult> {
        session.add_user_message(query.into());

        for chunk in codebase_context {
            session.context_window.add_chunk(chunk.clone());
        }

        let context_prompt = session.context_window.to_prompt();
        let history_summary = session.summarize_history();

        let full_prompt = if history_summary.is_empty() {
            format!("{context_prompt}\n\nUser question: {query}")
        } else {
            format!(
                "Conversation context: {history_summary}\n\n{context_prompt}\n\nUser question: {query}"
            )
        };

        let response = self.inference.generate(&InferenceRequest {
            prompt: full_prompt,
            max_tokens: Some(self.context_manager.max_tokens),
            temperature: Some(0.3),
            stop: None,
            system_prompt: Some(
                "You are a codebase assistant. Answer questions about code structure, \
                 functionality, and architecture. Reference specific files and line numbers."
                    .into(),
            ),
            messages: Vec::new(),
        })?;

        let sources: Vec<crate::rag_extended::context::ChunkSource> = session
            .context_window
            .chunks
            .iter()
            .map(|c| c.source.clone())
            .collect();

        let confidence = if sources.is_empty() {
            0.3
        } else {
            let avg_relevance: f32 = session
                .context_window
                .chunks
                .iter()
                .map(|c| c.relevance)
                .sum::<f32>()
                / session.context_window.chunks.len() as f32;
            (avg_relevance * 0.8 + 0.2).min(1.0)
        };

        let result = ChatQueryResult {
            response: response.text,
            sources,
            confidence,
        };

        session.add_assistant_response(result.response.clone(), result.sources.clone());

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::inference::InferenceConfig;
    use crate::rag_extended::context::create_chunk;

    fn make_chat() -> CodebaseChat {
        let config = InferenceConfig {
            model_id: "chat-model".into(),
            host: "127.0.0.1".into(),
            port: 8081,
            ..Default::default()
        };
        let inference = InferenceService::new(config);
        let context_config = ContextConfig::default();
        CodebaseChat::new(inference, context_config)
    }

    fn make_chunk(id: &str, content: &str) -> ContextChunk {
        create_chunk(id, content, 0.9, "src/main.rs", "Function", "main", 1, 5)
    }

    #[test]
    fn test_chat_session_new() {
        let session = ChatSession::new("test".into(), 4096, 50);
        assert!(session.messages.is_empty());
        assert_eq!(session.max_history, 50);
    }

    #[test]
    fn test_add_user_message() {
        let mut session = ChatSession::new("test".into(), 4096, 50);
        session.add_user_message("hello".into());
        assert_eq!(session.messages.len(), 1);
        assert!(matches!(session.messages[0].role, ChatRole::User));
    }

    #[test]
    fn test_add_assistant_response() {
        let mut session = ChatSession::new("test".into(), 4096, 50);
        session.add_user_message("hello".into());
        session.add_assistant_response("hi there".into(), Vec::new());
        assert_eq!(session.messages.len(), 2);
        assert!(matches!(session.messages[1].role, ChatRole::Assistant));
    }

    #[test]
    fn test_get_history() {
        let mut session = ChatSession::new("test".into(), 4096, 50);
        session.add_user_message("q1".into());
        session.add_assistant_response("a1".into(), Vec::new());
        assert_eq!(session.get_history().len(), 2);
    }

    #[test]
    fn test_summarize_history() {
        let mut session = ChatSession::new("test".into(), 4096, 50);
        session.add_user_message("what is main?".into());
        let summary = session.summarize_history();
        assert!(summary.contains("what is main?"));
    }

    #[test]
    fn test_summarize_history_empty() {
        let session = ChatSession::new("test".into(), 4096, 50);
        assert!(session.summarize_history().is_empty());
    }

    #[test]
    fn test_clear_history() {
        let mut session = ChatSession::new("test".into(), 4096, 50);
        session.add_user_message("q".into());
        session.clear_history();
        assert!(session.messages.is_empty());
    }

    #[test]
    fn test_trim_history() {
        let mut session = ChatSession::new("test".into(), 4096, 2);
        session.add_user_message("q1".into());
        session.add_assistant_response("a1".into(), Vec::new());
        session.add_user_message("q2".into());
        session.add_assistant_response("a2".into(), Vec::new());
        session.add_user_message("q3".into());
        assert_eq!(session.messages.len(), 2);
    }

    #[test]
    fn test_query_basic() {
        let chat = make_chat();
        let mut session = ChatSession::new("test".into(), 4096, 50);
        let chunks = vec![make_chunk("c1", "fn main() { println!(\"hello\"); }")];
        let result = chat
            .query(&mut session, "what does main do?", &chunks)
            .unwrap();
        assert!(!result.response.is_empty());
        assert_eq!(result.sources.len(), 1);
    }

    #[test]
    fn test_query_empty_context() {
        let chat = make_chat();
        let mut session = ChatSession::new("test".into(), 4096, 50);
        let result = chat.query(&mut session, "hello?", &[]).unwrap();
        assert!(result.confidence < 0.5);
    }

    #[test]
    fn test_query_adds_messages_to_session() {
        let chat = make_chat();
        let mut session = ChatSession::new("test".into(), 4096, 50);
        let chunks = vec![make_chunk("c1", "fn test() {}")];
        chat.query(&mut session, "explain test", &chunks).unwrap();
        assert_eq!(session.messages.len(), 2);
        assert!(matches!(session.messages[0].role, ChatRole::User));
        assert!(matches!(session.messages[1].role, ChatRole::Assistant));
    }

    #[test]
    fn test_chat_message_serialization() {
        let msg = ChatMessage {
            role: ChatRole::User,
            content: "hello".into(),
            timestamp: Utc::now(),
            sources: Vec::new(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let de: ChatMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(de.content, "hello");
    }

    #[test]
    fn test_chat_query_result_serialization() {
        let result = ChatQueryResult {
            response: "answer".into(),
            sources: Vec::new(),
            confidence: 0.95,
        };
        let json = serde_json::to_string(&result).unwrap();
        let de: ChatQueryResult = serde_json::from_str(&json).unwrap();
        assert!((de.confidence - 0.95).abs() < 0.001);
    }
}
