#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelFormat {
    Gguf,
    Safetensors,
    Onnx,
    Candle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Quantization {
    Q4_0,
    Q4_1,
    Q5_0,
    Q5_1,
    Q8_0,
    Fp16,
    Fp32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub size_bytes: u64,
    pub format: ModelFormat,
    pub quantization: Quantization,
    pub context_length: u32,
    pub created_at: DateTime<Utc>,
    pub path: String,
}

pub struct ModelRegistry {
    models: DashMap<String, ModelInfo>,
    default_model: Option<String>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self {
            models: DashMap::new(),
            default_model: None,
        }
    }

    pub fn register(&self, info: ModelInfo) -> anyhow::Result<()> {
        if self.models.contains_key(&info.id) {
            anyhow::bail!("model {} already registered", info.id);
        }
        self.models.insert(info.id.clone(), info);
        Ok(())
    }

    pub fn unregister(&mut self, id: &str) -> bool {
        let removed = self.models.remove(id);
        if removed.is_some() {
            if self.default_model.as_deref() == Some(id) {
                self.default_model = None;
            }
            true
        } else {
            false
        }
    }

    pub fn get(&self, id: &str) -> Option<ModelInfo> {
        self.models.get(id).map(|r| r.clone())
    }

    pub fn list(&self) -> Vec<ModelInfo> {
        self.models.iter().map(|r| r.value().clone()).collect()
    }

    pub fn set_default(&mut self, id: &str) -> anyhow::Result<()> {
        if !self.models.contains_key(id) {
            anyhow::bail!("model {} not found", id);
        }
        self.default_model = Some(id.to_owned());
        Ok(())
    }

    pub fn get_default(&self) -> Option<ModelInfo> {
        self.default_model
            .as_deref()
            .and_then(|id| self.models.get(id).map(|r| r.clone()))
    }

    pub fn search(&self, query: &str) -> Vec<ModelInfo> {
        let query_lower = query.to_lowercase();
        self.models
            .iter()
            .filter(|entry| {
                let info = entry.value();
                info.name.to_lowercase().contains(&query_lower)
                    || info.id.to_lowercase().contains(&query_lower)
            })
            .map(|r| r.value().clone())
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    pub max_tokens_per_request: u32,
    pub tokens_per_minute: u32,
    pub tokens_per_day: u32,
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self {
            max_tokens_per_request: 4096,
            tokens_per_minute: 60_000,
            tokens_per_day: 500_000,
        }
    }
}

#[derive(Debug)]
pub struct TokenUsage {
    pub total_tokens: AtomicU64,
    pub minute_tokens: AtomicU64,
    pub day_tokens: AtomicU64,
    pub last_minute_reset: AtomicU64,
    pub last_day_reset: AtomicU64,
}

impl TokenUsage {
    pub fn new() -> Self {
        let now = Utc::now().timestamp() as u64;
        Self {
            total_tokens: AtomicU64::new(0),
            minute_tokens: AtomicU64::new(0),
            day_tokens: AtomicU64::new(0),
            last_minute_reset: AtomicU64::new(now),
            last_day_reset: AtomicU64::new(now),
        }
    }
}

impl Default for TokenUsage {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageSnapshot {
    pub total_tokens: u64,
    pub minute_tokens: u64,
    pub day_tokens: u64,
    pub remaining_minute: u64,
    pub remaining_day: u64,
}

pub struct TokenCounter {
    budgets: DashMap<String, TokenBudget>,
    usage: DashMap<String, TokenUsage>,
}

impl TokenCounter {
    pub fn new() -> Self {
        Self {
            budgets: DashMap::new(),
            usage: DashMap::new(),
        }
    }

    pub fn register_budget(&self, id: String, budget: TokenBudget) {
        self.budgets.insert(id.clone(), budget);
        self.usage.entry(id).or_insert_with(TokenUsage::new);
    }

    pub fn check_budget(&self, id: &str) -> bool {
        let budget = match self.budgets.get(id) {
            Some(b) => b,
            None => return true,
        };
        let usage = match self.usage.get(id) {
            Some(u) => u,
            None => return true,
        };

        let now = Utc::now().timestamp() as u64;

        let last_minute = usage.last_minute_reset.load(Ordering::Relaxed);
        if now.saturating_sub(last_minute) >= 60 {
            usage.minute_tokens.store(0, Ordering::Relaxed);
            usage.last_minute_reset.store(now, Ordering::Relaxed);
        }

        let last_day = usage.last_day_reset.load(Ordering::Relaxed);
        if now.saturating_sub(last_day) >= 86_400 {
            usage.day_tokens.store(0, Ordering::Relaxed);
            usage.last_day_reset.store(now, Ordering::Relaxed);
        }

        usage.minute_tokens.load(Ordering::Relaxed) < budget.tokens_per_minute as u64
            && usage.day_tokens.load(Ordering::Relaxed) < budget.tokens_per_day as u64
    }

    pub fn record_usage(&self, id: &str, tokens: u32) {
        if let Some(usage) = self.usage.get(id) {
            usage.total_tokens.fetch_add(tokens as u64, Ordering::Relaxed);
            usage.minute_tokens.fetch_add(tokens as u64, Ordering::Relaxed);
            usage.day_tokens.fetch_add(tokens as u64, Ordering::Relaxed);
        }
    }

    pub fn get_usage(&self, id: &str) -> TokenUsageSnapshot {
        let now = Utc::now().timestamp() as u64;

        let usage = self
            .usage
            .get(id)
            .map(|u| {
                let last_minute = u.last_minute_reset.load(Ordering::Relaxed);
                if now.saturating_sub(last_minute) >= 60 {
                    u.minute_tokens.store(0, Ordering::Relaxed);
                    u.last_minute_reset.store(now, Ordering::Relaxed);
                }
                let last_day = u.last_day_reset.load(Ordering::Relaxed);
                if now.saturating_sub(last_day) >= 86_400 {
                    u.day_tokens.store(0, Ordering::Relaxed);
                    u.last_day_reset.store(now, Ordering::Relaxed);
                }
                TokenUsageSnapshot {
                    total_tokens: u.total_tokens.load(Ordering::Relaxed),
                    minute_tokens: u.minute_tokens.load(Ordering::Relaxed),
                    day_tokens: u.day_tokens.load(Ordering::Relaxed),
                    remaining_minute: 0,
                    remaining_day: 0,
                }
            })
            .unwrap_or(TokenUsageSnapshot {
                total_tokens: 0,
                minute_tokens: 0,
                day_tokens: 0,
                remaining_minute: 0,
                remaining_day: 0,
            });

        let budget = self
            .budgets
            .get(id)
            .map(|b| TokenUsageSnapshot {
                total_tokens: usage.total_tokens,
                minute_tokens: usage.minute_tokens,
                day_tokens: usage.day_tokens,
                remaining_minute: (b.tokens_per_minute as u64).saturating_sub(usage.minute_tokens),
                remaining_day: (b.tokens_per_day as u64).saturating_sub(usage.day_tokens),
            })
            .unwrap_or(usage);

        budget
    }
}

impl Default for TokenCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_model_info(id: &str, name: &str) -> ModelInfo {
        ModelInfo {
            id: id.into(),
            name: name.into(),
            version: "1.0.0".into(),
            size_bytes: 1024,
            format: ModelFormat::Gguf,
            quantization: Quantization::Q4_0,
            context_length: 4096,
            created_at: Utc::now(),
            path: format!("/models/{id}"),
        }
    }

    #[test]
    fn test_register_and_get() {
        let registry = ModelRegistry::new();
        let info = make_model_info("model-1", "Test Model");
        registry.register(info.clone()).unwrap();
        let retrieved = registry.get("model-1").unwrap();
        assert_eq!(retrieved.id, "model-1");
        assert_eq!(retrieved.name, "Test Model");
    }

    #[test]
    fn test_register_duplicate_fails() {
        let registry = ModelRegistry::new();
        registry.register(make_model_info("m1", "A")).unwrap();
        assert!(registry.register(make_model_info("m1", "B")).is_err());
    }

    #[test]
    fn test_unregister() {
        let mut registry = ModelRegistry::new();
        registry.register(make_model_info("m1", "A")).unwrap();
        assert!(registry.unregister("m1"));
        assert!(registry.get("m1").is_none());
    }

    #[test]
    fn test_unregister_nonexistent() {
        let mut registry = ModelRegistry::new();
        assert!(!registry.unregister("nope"));
    }

    #[test]
    fn test_list() {
        let registry = ModelRegistry::new();
        registry.register(make_model_info("m1", "A")).unwrap();
        registry.register(make_model_info("m2", "B")).unwrap();
        assert_eq!(registry.list().len(), 2);
    }

    #[test]
    fn test_set_default() {
        let mut registry = ModelRegistry::new();
        registry.register(make_model_info("m1", "A")).unwrap();
        registry.set_default("m1").unwrap();
        let default = registry.get_default().unwrap();
        assert_eq!(default.id, "m1");
    }

    #[test]
    fn test_set_default_nonexistent_fails() {
        let mut registry = ModelRegistry::new();
        assert!(registry.set_default("nope").is_err());
    }

    #[test]
    fn test_unregister_clears_default() {
        let mut registry = ModelRegistry::new();
        registry.register(make_model_info("m1", "A")).unwrap();
        registry.set_default("m1").unwrap();
        registry.unregister("m1");
        assert!(registry.get_default().is_none());
    }

    #[test]
    fn test_search() {
        let registry = ModelRegistry::new();
        registry.register(make_model_info("llama-7b", "Llama 7B")).unwrap();
        registry.register(make_model_info("mistral-7b", "Mistral 7B")).unwrap();
        let results = registry.search("llama");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "llama-7b");
    }

    #[test]
    fn test_search_case_insensitive() {
        let registry = ModelRegistry::new();
        registry.register(make_model_info("LlamaModel", "Llama")).unwrap();
        let results = registry.search("llama");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_token_budget_default() {
        let budget = TokenBudget::default();
        assert_eq!(budget.max_tokens_per_request, 4096);
        assert_eq!(budget.tokens_per_minute, 60_000);
        assert_eq!(budget.tokens_per_day, 500_000);
    }

    #[test]
    fn test_token_counter_check_budget() {
        let counter = TokenCounter::new();
        counter.register_budget(
            "model-a".into(),
            TokenBudget {
                max_tokens_per_request: 100,
                tokens_per_minute: 100,
                tokens_per_day: 1000,
            },
        );
        assert!(counter.check_budget("model-a"));
        counter.record_usage("model-a", 50);
        assert!(counter.check_budget("model-a"));
        counter.record_usage("model-a", 50);
        assert!(!counter.check_budget("model-a"));
    }

    #[test]
    fn test_token_counter_get_usage() {
        let counter = TokenCounter::new();
        counter.register_budget(
            "model-a".into(),
            TokenBudget {
                max_tokens_per_request: 100,
                tokens_per_minute: 100,
                tokens_per_day: 1000,
            },
        );
        counter.record_usage("model-a", 30);
        let snap = counter.get_usage("model-a");
        assert_eq!(snap.total_tokens, 30);
        assert_eq!(snap.remaining_minute, 70);
        assert_eq!(snap.remaining_day, 970);
    }

    #[test]
    fn test_token_counter_unknown_id() {
        let counter = TokenCounter::new();
        assert!(counter.check_budget("unknown"));
        let snap = counter.get_usage("unknown");
        assert_eq!(snap.total_tokens, 0);
    }

    #[test]
    fn test_model_info_serialization() {
        let info = make_model_info("m1", "Test");
        let json = serde_json::to_string(&info).unwrap();
        let de: ModelInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(de.id, "m1");
        assert_eq!(de.format, ModelFormat::Gguf);
    }
}
