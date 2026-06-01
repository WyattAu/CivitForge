#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelArchitecture {
    Transformer,
    Mamba,
    Rwkv,
    Mixtral,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub architecture: ModelArchitecture,
    pub parameter_count: u64,
    pub quantization: String,
    pub context_length: u32,
    pub created_at: DateTime<Utc>,
    pub size_bytes: u64,
    pub checksum: String,
}

impl ModelInfo {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            architecture: ModelArchitecture::Transformer,
            parameter_count: 0,
            quantization: String::new(),
            context_length: 4096,
            created_at: Utc::now(),
            size_bytes: 0,
            checksum: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelRegistry {
    models: DashMap<String, ModelInfo>,
    aliases: DashMap<String, String>,
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self {
            models: DashMap::new(),
            aliases: DashMap::new(),
        }
    }

    pub fn register_model(&self, model: ModelInfo) -> anyhow::Result<()> {
        let id = model.id.clone();
        if self.models.contains_key(&id) {
            anyhow::bail!("model '{id}' already registered");
        }
        self.models.insert(id, model);
        Ok(())
    }

    pub fn unregister_model(&self, id: &str) -> bool {
        let removed = self.models.remove(id).is_some();
        if removed {
            self.aliases.retain(|_, v| v != id);
        }
        removed
    }

    pub fn get_model(&self, id: &str) -> Option<ModelInfo> {
        self.resolve_alias(id)
            .and_then(|resolved| self.models.get(&resolved).map(|r| r.value().clone()))
    }

    pub fn list_models(&self) -> Vec<ModelInfo> {
        self.models.iter().map(|r| r.value().clone()).collect()
    }

    pub fn resolve_alias(&self, name: &str) -> Option<String> {
        if self.models.contains_key(name) {
            return Some(name.into());
        }
        self.aliases.get(name).map(|r| r.value().clone())
    }

    pub fn set_alias(&self, alias: &str, model_id: &str) -> anyhow::Result<()> {
        if !self.models.contains_key(model_id) {
            anyhow::bail!("model '{model_id}' not found for alias");
        }
        self.aliases.insert(alias.into(), model_id.into());
        Ok(())
    }

    pub fn model_count(&self) -> usize {
        self.models.len()
    }
}

#[derive(Debug, Clone)]
pub struct StreamingResponse {
    pub chunk_id: u64,
    pub content: String,
    pub finish_reason: Option<String>,
    pub token_count: u32,
    pub created_at: DateTime<Utc>,
}

pub struct StreamHandle {
    chunks: Vec<StreamingResponse>,
    position: usize,
}

impl StreamHandle {
    pub fn new(chunks: Vec<StreamingResponse>) -> Self {
        Self {
            chunks,
            position: 0,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.position >= self.chunks.len()
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    pub fn total_tokens(&self) -> u32 {
        self.chunks.iter().map(|c| c.token_count).sum()
    }
}

pub trait InferenceStream: Send + Sync {
    fn next_chunk(&mut self) -> Option<StreamingResponse>;
}

pub struct StubInferenceStream {
    chunks: Vec<StreamingResponse>,
    position: usize,
}

impl StubInferenceStream {
    pub fn new(responses: Vec<&str>) -> Self {
        let chunks = responses
            .iter()
            .enumerate()
            .map(|(i, text)| StreamingResponse {
                chunk_id: i as u64,
                content: text.to_string(),
                finish_reason: if i == responses.len() - 1 {
                    Some("stop".into())
                } else {
                    None
                },
                token_count: text.split_whitespace().count() as u32,
                created_at: Utc::now(),
            })
            .collect();
        Self {
            chunks,
            position: 0,
        }
    }

    pub fn empty() -> Self {
        Self {
            chunks: Vec::new(),
            position: 0,
        }
    }
}

impl Default for StubInferenceStream {
    fn default() -> Self {
        Self::empty()
    }
}

impl InferenceStream for StubInferenceStream {
    fn next_chunk(&mut self) -> Option<StreamingResponse> {
        if self.position < self.chunks.len() {
            let chunk = self.chunks[self.position].clone();
            self.position += 1;
            Some(chunk)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct UsageEntry {
    pub user_tokens: u64,
    pub repo_tokens: u64,
    pub daily_tokens: u64,
}

#[derive(Debug)]
pub struct TokenBudgetManager {
    per_user_budget: u64,
    per_repo_budget: u64,
    daily_budget: u64,
    current_usage: DashMap<String, UsageEntry>,
}

impl TokenBudgetManager {
    pub fn new(per_user_budget: u64, per_repo_budget: u64, daily_budget: u64) -> Self {
        Self {
            per_user_budget,
            per_repo_budget,
            daily_budget,
            current_usage: DashMap::new(),
        }
    }

    pub fn check_budget(&self, user_id: &str, repo_id: &str, estimated_tokens: u32) -> bool {
        let user_key = format!("user:{user_id}");
        let repo_key = format!("repo:{repo_id}");
        let daily_key = "daily".to_string();

        let estimated = estimated_tokens as u64;

        if let Some(entry) = self.current_usage.get(&user_key) {
            if entry.user_tokens + estimated > self.per_user_budget {
                return false;
            }
        }

        if let Some(entry) = self.current_usage.get(&repo_key) {
            if entry.repo_tokens + estimated > self.per_repo_budget {
                return false;
            }
        }

        if let Some(entry) = self.current_usage.get(&daily_key) {
            if entry.daily_tokens + estimated > self.daily_budget {
                return false;
            }
        }

        true
    }

    pub fn consume(&self, user_id: &str, repo_id: &str, tokens: u32) {
        let tokens = tokens as u64;
        let user_key = format!("user:{user_id}");
        let repo_key = format!("repo:{repo_id}");
        let daily_key = "daily".to_string();

        self.current_usage
            .entry(user_key)
            .or_insert_with(|| UsageEntry {
                user_tokens: 0,
                repo_tokens: 0,
                daily_tokens: 0,
            })
            .user_tokens += tokens;

        self.current_usage
            .entry(repo_key)
            .or_insert_with(|| UsageEntry {
                user_tokens: 0,
                repo_tokens: 0,
                daily_tokens: 0,
            })
            .repo_tokens += tokens;

        self.current_usage
            .entry(daily_key)
            .or_insert_with(|| UsageEntry {
                user_tokens: 0,
                repo_tokens: 0,
                daily_tokens: 0,
            })
            .daily_tokens += tokens;
    }

    pub fn get_usage(&self, key: &str) -> Option<UsageEntry> {
        self.current_usage.get(key).map(|r| r.value().clone())
    }

    pub fn reset_daily(&self) {
        self.current_usage.remove("daily");
    }

    pub fn per_user_budget(&self) -> u64 {
        self.per_user_budget
    }

    pub fn per_repo_budget(&self) -> u64 {
        self.per_repo_budget
    }

    pub fn daily_budget(&self) -> u64 {
        self.daily_budget
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_model(id: &str, name: &str) -> ModelInfo {
        let mut info = ModelInfo::new(id, name);
        info.parameter_count = 7_000_000_000;
        info.size_bytes = 3_800_000_000;
        info.checksum = "abc123".into();
        info.quantization = "Q4_0".into();
        info
    }

    #[test]
    fn test_model_info_new() {
        let info = ModelInfo::new("m1", "Test Model");
        assert_eq!(info.id, "m1");
        assert_eq!(info.name, "Test Model");
        assert_eq!(info.architecture, ModelArchitecture::Transformer);
    }

    #[test]
    fn test_model_architecture_variants() {
        let _ = ModelArchitecture::Transformer;
        let _ = ModelArchitecture::Mamba;
        let _ = ModelArchitecture::Rwkv;
        let _ = ModelArchitecture::Mixtral;
        let _ = ModelArchitecture::Custom("custom-arch".into());
    }

    #[test]
    fn test_model_architecture_serialization() {
        let arch = ModelArchitecture::Transformer;
        let json = serde_json::to_string(&arch).unwrap();
        let de: ModelArchitecture = serde_json::from_str(&json).unwrap();
        assert_eq!(arch, de);
    }

    #[test]
    fn test_model_registry_register_and_get() {
        let registry = ModelRegistry::new();
        registry
            .register_model(make_model("m1", "Model One"))
            .unwrap();
        let model = registry.get_model("m1").unwrap();
        assert_eq!(model.name, "Model One");
    }

    #[test]
    fn test_model_registry_register_duplicate_fails() {
        let registry = ModelRegistry::new();
        registry.register_model(make_model("m1", "A")).unwrap();
        assert!(registry.register_model(make_model("m1", "B")).is_err());
    }

    #[test]
    fn test_model_registry_unregister() {
        let registry = ModelRegistry::new();
        registry.register_model(make_model("m1", "A")).unwrap();
        assert!(registry.unregister_model("m1"));
        assert!(registry.get_model("m1").is_none());
    }

    #[test]
    fn test_model_registry_unregister_nonexistent() {
        let registry = ModelRegistry::new();
        assert!(!registry.unregister_model("nope"));
    }

    #[test]
    fn test_model_registry_list() {
        let registry = ModelRegistry::new();
        registry.register_model(make_model("m1", "A")).unwrap();
        registry.register_model(make_model("m2", "B")).unwrap();
        assert_eq!(registry.list_models().len(), 2);
    }

    #[test]
    fn test_model_registry_alias() {
        let registry = ModelRegistry::new();
        registry.register_model(make_model("m1", "Model")).unwrap();
        registry.set_alias("latest", "m1").unwrap();
        assert_eq!(registry.resolve_alias("latest"), Some("m1".into()));
        let model = registry.get_model("latest").unwrap();
        assert_eq!(model.id, "m1");
    }

    #[test]
    fn test_model_registry_alias_nonexistent_model() {
        let registry = ModelRegistry::new();
        assert!(registry.set_alias("latest", "nope").is_err());
    }

    #[test]
    fn test_model_registry_alias_cleared_on_unregister() {
        let registry = ModelRegistry::new();
        registry.register_model(make_model("m1", "Model")).unwrap();
        registry.set_alias("latest", "m1").unwrap();
        registry.unregister_model("m1");
        assert!(registry.resolve_alias("latest").is_none());
    }

    #[test]
    fn test_model_registry_model_count() {
        let registry = ModelRegistry::new();
        assert_eq!(registry.model_count(), 0);
        registry.register_model(make_model("m1", "A")).unwrap();
        assert_eq!(registry.model_count(), 1);
    }

    #[test]
    fn test_streaming_response_fields() {
        let response = StreamingResponse {
            chunk_id: 0,
            content: "hello".into(),
            finish_reason: Some("stop".into()),
            token_count: 1,
            created_at: Utc::now(),
        };
        assert_eq!(response.chunk_id, 0);
        assert_eq!(response.content, "hello");
    }

    #[test]
    fn test_stream_handle_new() {
        let handle = StreamHandle::new(vec![StreamingResponse {
            chunk_id: 0,
            content: "a".into(),
            finish_reason: Some("stop".into()),
            token_count: 1,
            created_at: Utc::now(),
        }]);
        assert!(!handle.is_complete());
        assert_eq!(handle.len(), 1);
        assert_eq!(handle.total_tokens(), 1);
    }

    #[test]
    fn test_stream_handle_empty() {
        let handle = StreamHandle::new(vec![]);
        assert!(handle.is_complete());
        assert!(handle.is_empty());
    }

    #[test]
    fn test_stub_inference_stream_next_chunk() {
        let mut stream = StubInferenceStream::new(vec!["hello", "world"]);
        let c1 = stream.next_chunk().unwrap();
        assert_eq!(c1.content, "hello");
        assert!(c1.finish_reason.is_none());
        let c2 = stream.next_chunk().unwrap();
        assert_eq!(c2.content, "world");
        assert_eq!(c2.finish_reason.as_deref(), Some("stop"));
        assert!(stream.next_chunk().is_none());
    }

    #[test]
    fn test_stub_inference_stream_empty() {
        let mut stream = StubInferenceStream::empty();
        assert!(stream.next_chunk().is_none());
    }

    #[test]
    fn test_stub_inference_stream_default() {
        let mut stream = StubInferenceStream::default();
        assert!(stream.next_chunk().is_none());
    }

    #[test]
    fn test_stub_inference_stream_single() {
        let mut stream = StubInferenceStream::new(vec!["only chunk"]);
        let c = stream.next_chunk().unwrap();
        assert_eq!(c.content, "only chunk");
        assert_eq!(c.finish_reason.as_deref(), Some("stop"));
        assert!(stream.next_chunk().is_none());
    }

    #[test]
    fn test_token_budget_manager_check_budget_ok() {
        let mgr = TokenBudgetManager::new(1000, 5000, 100000);
        assert!(mgr.check_budget("user1", "repo1", 100));
    }

    #[test]
    fn test_token_budget_manager_check_budget_exceeded_user() {
        let mgr = TokenBudgetManager::new(100, 5000, 100000);
        mgr.consume("user1", "repo1", 80);
        assert!(!mgr.check_budget("user1", "repo1", 50));
    }

    #[test]
    fn test_token_budget_manager_check_budget_exceeded_repo() {
        let mgr = TokenBudgetManager::new(10000, 100, 100000);
        mgr.consume("user1", "repo1", 80);
        assert!(!mgr.check_budget("user1", "repo1", 50));
    }

    #[test]
    fn test_token_budget_manager_check_budget_exceeded_daily() {
        let mgr = TokenBudgetManager::new(10000, 10000, 100);
        mgr.consume("user1", "repo1", 80);
        assert!(!mgr.check_budget("user1", "repo1", 50));
    }

    #[test]
    fn test_token_budget_manager_consume() {
        let mgr = TokenBudgetManager::new(1000, 5000, 100000);
        mgr.consume("user1", "repo1", 50);
        let usage = mgr.get_usage("user:user1").unwrap();
        assert_eq!(usage.user_tokens, 50);
        let repo_usage = mgr.get_usage("repo:repo1").unwrap();
        assert_eq!(repo_usage.repo_tokens, 50);
        let daily = mgr.get_usage("daily").unwrap();
        assert_eq!(daily.daily_tokens, 50);
    }

    #[test]
    fn test_token_budget_manager_get_usage_missing() {
        let mgr = TokenBudgetManager::new(1000, 5000, 100000);
        assert!(mgr.get_usage("user:unknown").is_none());
    }

    #[test]
    fn test_token_budget_manager_reset_daily() {
        let mgr = TokenBudgetManager::new(1000, 5000, 100);
        mgr.consume("user1", "repo1", 50);
        mgr.reset_daily();
        assert!(mgr.get_usage("daily").is_none());
        assert!(mgr.get_usage("user:user1").is_some());
    }

    #[test]
    fn test_token_budget_manager_budget_accessors() {
        let mgr = TokenBudgetManager::new(100, 200, 300);
        assert_eq!(mgr.per_user_budget(), 100);
        assert_eq!(mgr.per_repo_budget(), 200);
        assert_eq!(mgr.daily_budget(), 300);
    }
}
