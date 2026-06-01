#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// A single turn in a conversation, with metadata for efficient retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub id: String,
    pub role: ConversationRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub token_count: usize,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversationRole {
    User,
    Assistant,
    System,
}

/// Persistent conversation history with automatic summarization and eviction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationHistory {
    pub id: String,
    pub turns: VecDeque<ConversationTurn>,
    pub max_turns: usize,
    pub max_tokens: usize,
    pub total_tokens: usize,
    pub auto_summarize: bool,
    pub summarize_every_n_turns: usize,
}

impl ConversationHistory {
    pub fn new(id: impl Into<String>, max_turns: usize, max_tokens: usize) -> Self {
        Self {
            id: id.into(),
            turns: VecDeque::new(),
            max_turns,
            max_tokens,
            total_tokens: 0,
            auto_summarize: true,
            summarize_every_n_turns: 6,
        }
    }

    pub fn with_auto_summarize(mut self, enabled: bool, every_n: usize) -> Self {
        self.auto_summarize = enabled;
        self.summarize_every_n_turns = every_n;
        self
    }

    pub fn add_turn(&mut self, turn: ConversationTurn) {
        if self.turns.len() >= self.max_turns {
            if let Some(evicted) = self.turns.pop_front() {
                self.total_tokens -= evicted.token_count;
            }
        }
        self.total_tokens += turn.token_count;
        self.turns.push_back(turn);
    }

    pub fn get_recent(&self, n: usize) -> Vec<&ConversationTurn> {
        let start = self.turns.len().saturating_sub(n);
        self.turns.iter().skip(start).collect()
    }

    pub fn get_all(&self) -> Vec<&ConversationTurn> {
        self.turns.iter().collect()
    }

    pub fn len(&self) -> usize {
        self.turns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.turns.is_empty()
    }

    /// Generates a summary of the conversation. In production, this would call the LLM.
    /// The stub implementation extracts key phrases from user messages.
    pub fn generate_summary(&self) -> String {
        if self.turns.is_empty() {
            return String::new();
        }
        let user_msgs: Vec<&str> = self
            .turns
            .iter()
            .filter(|t| t.role == ConversationRole::User)
            .map(|t| t.content.as_str())
            .collect();
        let assistant_count = self
            .turns
            .iter()
            .filter(|t| t.role == ConversationRole::Assistant)
            .count();
        format!(
            "Conversation with {} user queries and {} assistant responses. Topics: {}",
            user_msgs.len(),
            assistant_count,
            user_msgs.join("; ")
        )
    }

    /// Truncates old turns when total token budget is exceeded.
    pub fn enforce_token_budget(&mut self) {
        while self.total_tokens > self.max_tokens && !self.turns.is_empty() {
            if let Some(evicted) = self.turns.pop_front() {
                self.total_tokens -= evicted.token_count;
            }
        }
    }

    /// Checks if summarization should be triggered.
    pub fn should_summarize(&self) -> bool {
        self.auto_summarize
            && !self.turns.is_empty()
            && self.turns.len() % self.summarize_every_n_turns == 0
    }

    /// Compresses older turns into a single summary turn, freeing token budget.
    pub fn compress_history(&mut self) {
        if self.turns.len() < 4 {
            return;
        }
        let keep_recent = self.turns.len() / 2;
        let to_summarize: Vec<ConversationTurn> =
            self.turns.drain(..self.turns.len() - keep_recent).collect();
        let mut tokens_freed = 0usize;
        for turn in &to_summarize {
            tokens_freed += turn.token_count;
        }
        let summary_content = to_summarize
            .iter()
            .filter(|t| t.role == ConversationRole::User)
            .map(|t| t.content.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        let summary_tokens = estimate_tokens(&summary_content);
        let summary_turn = ConversationTurn {
            id: uuid::Uuid::new_v4().to_string(),
            role: ConversationRole::System,
            content: format!("[Earlier conversation summary] {summary_content}"),
            timestamp: Utc::now(),
            token_count: summary_tokens,
            summary: Some(summary_content),
        };
        self.total_tokens = self.total_tokens.saturating_sub(tokens_freed) + summary_tokens;
        self.turns.push_front(summary_turn);
    }

    pub fn clear(&mut self) {
        self.turns.clear();
        self.total_tokens = 0;
    }
}

fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    (text.len() as f32 / 4.0).ceil() as usize
}

/// Manages multiple concurrent chat sessions with per-session history and token budgets.
#[derive(Debug, Clone)]
pub struct SessionManager {
    sessions: dashmap::DashMap<String, ConversationHistory>,
    max_sessions: usize,
    default_max_turns: usize,
    default_max_tokens: usize,
}

impl SessionManager {
    pub fn new(max_sessions: usize, default_max_turns: usize, default_max_tokens: usize) -> Self {
        Self {
            sessions: dashmap::DashMap::new(),
            max_sessions,
            default_max_turns,
            default_max_tokens,
        }
    }

    pub fn create_session(&self, id: impl Into<String>) -> String {
        let sid = id.into();
        if self.sessions.len() >= self.max_sessions && !self.sessions.contains_key(&sid) {
            self.evict_oldest();
        }
        let history =
            ConversationHistory::new(&sid, self.default_max_turns, self.default_max_tokens);
        self.sessions.insert(sid.clone(), history);
        sid
    }

    pub fn get_session(&self, id: &str) -> Option<ConversationHistory> {
        self.sessions.get(id).map(|r| r.value().clone())
    }

    pub fn session_exists(&self, id: &str) -> bool {
        self.sessions.contains_key(id)
    }

    pub fn remove_session(&self, id: &str) -> bool {
        self.sessions.remove(id).is_some()
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn list_session_ids(&self) -> Vec<String> {
        self.sessions.iter().map(|r| r.key().clone()).collect()
    }

    fn evict_oldest(&self) {
        let oldest = self
            .sessions
            .iter()
            .min_by_key(|r| {
                r.value()
                    .turns
                    .back()
                    .map(|t| t.timestamp)
                    .unwrap_or_else(|| DateTime::<Utc>::MIN_UTC)
            })
            .map(|r| r.key().clone());
        if let Some(key) = oldest {
            self.sessions.remove(&key);
        }
    }
}

/// Token budget tracker that enforces limits across multiple contexts.
#[derive(Debug, Clone)]
pub struct TokenBudget {
    pub total_budget: usize,
    pub system_prompt_tokens: usize,
    pub context_tokens: usize,
    pub history_tokens: usize,
    pub reserved_for_response: usize,
}

impl TokenBudget {
    pub fn new(total_budget: usize, reserve_for_response: f32) -> Self {
        let reserved = (total_budget as f32 * reserve_for_response) as usize;
        Self {
            total_budget,
            system_prompt_tokens: 0,
            context_tokens: 0,
            history_tokens: 0,
            reserved_for_response: reserved,
        }
    }

    pub fn remaining_for_context(&self) -> usize {
        self.total_budget
            .saturating_sub(self.system_prompt_tokens)
            .saturating_sub(self.history_tokens)
            .saturating_sub(self.reserved_for_response)
    }

    pub fn remaining_for_history(&self) -> usize {
        self.total_budget
            .saturating_sub(self.system_prompt_tokens)
            .saturating_sub(self.context_tokens)
            .saturating_sub(self.reserved_for_response)
    }

    pub fn is_over_budget(&self) -> bool {
        let used = self.system_prompt_tokens + self.context_tokens + self.history_tokens;
        used > self.total_budget.saturating_sub(self.reserved_for_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_turn(role: ConversationRole, content: &str) -> ConversationTurn {
        ConversationTurn {
            id: uuid::Uuid::new_v4().to_string(),
            role,
            content: content.into(),
            timestamp: Utc::now(),
            token_count: estimate_tokens(content),
            summary: None,
        }
    }

    #[test]
    fn test_conversation_history_new() {
        let h = ConversationHistory::new("test", 100, 4096);
        assert!(h.is_empty());
        assert_eq!(h.len(), 0);
    }

    #[test]
    fn test_add_turn() {
        let mut h = ConversationHistory::new("test", 100, 4096);
        h.add_turn(make_turn(ConversationRole::User, "hello"));
        assert_eq!(h.len(), 1);
    }

    #[test]
    fn test_max_turns_eviction() {
        let mut h = ConversationHistory::new("test", 3, 4096);
        h.add_turn(make_turn(ConversationRole::User, "q1"));
        h.add_turn(make_turn(ConversationRole::Assistant, "a1"));
        h.add_turn(make_turn(ConversationRole::User, "q2"));
        h.add_turn(make_turn(ConversationRole::Assistant, "a2"));
        // With max 3, oldest should be evicted when adding 4th
        assert_eq!(h.len(), 3);
        assert_eq!(h.turns[0].content, "a1");
    }

    #[test]
    fn test_get_recent() {
        let mut h = ConversationHistory::new("test", 100, 4096);
        h.add_turn(make_turn(ConversationRole::User, "q1"));
        h.add_turn(make_turn(ConversationRole::Assistant, "a1"));
        h.add_turn(make_turn(ConversationRole::User, "q2"));
        let recent = h.get_recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].content, "a1");
    }

    #[test]
    fn test_generate_summary() {
        let mut h = ConversationHistory::new("test", 100, 4096);
        h.add_turn(make_turn(ConversationRole::User, "what is main?"));
        h.add_turn(make_turn(
            ConversationRole::Assistant,
            "main is entry point",
        ));
        let summary = h.generate_summary();
        assert!(summary.contains("what is main?"));
        assert!(summary.contains("1 user queries"));
    }

    #[test]
    fn test_generate_summary_empty() {
        let h = ConversationHistory::new("test", 100, 4096);
        assert!(h.generate_summary().is_empty());
    }

    #[test]
    fn test_enforce_token_budget() {
        let mut h = ConversationHistory::new("test", 100, 50);
        h.add_turn(make_turn(
            ConversationRole::User,
            "This is a very long message that should exceed the token budget when combined with others",
        ));
        h.add_turn(make_turn(
            ConversationRole::Assistant,
            "This is another very long message that should also contribute to exceeding the token budget",
        ));
        h.enforce_token_budget();
        assert!(h.total_tokens <= 50);
    }

    #[test]
    fn test_should_summarize() {
        let mut h = ConversationHistory::new("test", 100, 4096).with_auto_summarize(true, 4);
        // Add 4 turns: should trigger (4 % 4 == 0)
        for i in 0..2 {
            h.add_turn(make_turn(ConversationRole::User, &format!("q{i}")));
            h.add_turn(make_turn(ConversationRole::Assistant, &format!("a{i}")));
        }
        assert!(h.should_summarize()); // 4 % 4 == 0
    }

    #[test]
    fn test_compress_history() {
        let mut h = ConversationHistory::new("test", 100, 4096);
        h.add_turn(make_turn(ConversationRole::User, "q1"));
        h.add_turn(make_turn(ConversationRole::Assistant, "a1"));
        h.add_turn(make_turn(ConversationRole::User, "q2"));
        h.add_turn(make_turn(ConversationRole::Assistant, "a2"));
        h.add_turn(make_turn(ConversationRole::User, "q3"));
        h.add_turn(make_turn(ConversationRole::Assistant, "a3"));
        h.compress_history();
        // First turn should be a system summary
        assert_eq!(h.turns[0].role, ConversationRole::System);
        assert!(
            h.turns[0]
                .content
                .contains("[Earlier conversation summary]")
        );
    }

    #[test]
    fn test_compress_history_too_short() {
        let mut h = ConversationHistory::new("test", 100, 4096);
        h.add_turn(make_turn(ConversationRole::User, "q1"));
        h.add_turn(make_turn(ConversationRole::Assistant, "a1"));
        h.compress_history();
        // Should not compress with only 2 turns
        assert_eq!(h.turns.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut h = ConversationHistory::new("test", 100, 4096);
        h.add_turn(make_turn(ConversationRole::User, "q1"));
        h.clear();
        assert!(h.is_empty());
        assert_eq!(h.total_tokens, 0);
    }

    #[test]
    fn test_session_manager_create() {
        let mgr = SessionManager::new(10, 50, 4096);
        let sid = mgr.create_session("s1");
        assert_eq!(sid, "s1");
        assert_eq!(mgr.session_count(), 1);
    }

    #[test]
    fn test_session_manager_get() {
        let mgr = SessionManager::new(10, 50, 4096);
        mgr.create_session("s1");
        assert!(mgr.get_session("s1").is_some());
        assert!(mgr.get_session("nonexistent").is_none());
    }

    #[test]
    fn test_session_manager_remove() {
        let mgr = SessionManager::new(10, 50, 4096);
        mgr.create_session("s1");
        assert!(mgr.remove_session("s1"));
        assert_eq!(mgr.session_count(), 0);
        assert!(!mgr.remove_session("nonexistent"));
    }

    #[test]
    fn test_session_manager_eviction() {
        let mgr = SessionManager::new(2, 50, 4096);
        mgr.create_session("s1");
        mgr.create_session("s2");
        mgr.create_session("s3"); // Should evict oldest
        assert_eq!(mgr.session_count(), 2);
    }

    #[test]
    fn test_session_manager_list_ids() {
        let mgr = SessionManager::new(10, 50, 4096);
        mgr.create_session("s1");
        mgr.create_session("s2");
        let ids = mgr.list_session_ids();
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn test_token_budget_new() {
        let budget = TokenBudget::new(4096, 0.2);
        assert_eq!(budget.reserved_for_response, 819);
    }

    #[test]
    fn test_token_budget_remaining() {
        let mut budget = TokenBudget::new(4096, 0.2);
        budget.system_prompt_tokens = 100;
        budget.history_tokens = 500;
        let remaining_ctx = budget.remaining_for_context();
        assert!(remaining_ctx > 0);
    }

    #[test]
    fn test_token_budget_over_budget() {
        let mut budget = TokenBudget::new(100, 0.2);
        budget.system_prompt_tokens = 50;
        budget.context_tokens = 50;
        budget.history_tokens = 50;
        assert!(budget.is_over_budget());
    }

    #[test]
    fn test_token_budget_not_over_budget() {
        let mut budget = TokenBudget::new(1000, 0.2);
        budget.system_prompt_tokens = 100;
        budget.context_tokens = 200;
        budget.history_tokens = 100;
        assert!(!budget.is_over_budget());
    }

    #[test]
    fn test_conversation_turn_serialization() {
        let turn = ConversationTurn {
            id: "t1".into(),
            role: ConversationRole::User,
            content: "hello".into(),
            timestamp: Utc::now(),
            token_count: 5,
            summary: None,
        };
        let json = serde_json::to_string(&turn).unwrap();
        let de: ConversationTurn = serde_json::from_str(&json).unwrap();
        assert_eq!(de.content, "hello");
    }

    #[test]
    fn test_conversation_history_serialization() {
        let h = ConversationHistory::new("test", 100, 4096);
        let json = serde_json::to_string(&h).unwrap();
        let de: ConversationHistory = serde_json::from_str(&json).unwrap();
        assert_eq!(de.id, "test");
    }
}
