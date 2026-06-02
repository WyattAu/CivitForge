#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub parameter_count: u64,
    pub context_window: u32,
    pub max_tokens: u32,
    pub endpoint: Option<String>,
    pub quantization: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub content: String,
    pub tokens_used: u32,
    pub model: String,
    pub duration_ms: u64,
    pub finish_reason: String,
}

pub trait LlmProvider: Send + Sync {
    fn infer(
        &self,
        messages: &[ChatMessage],
        config: &ModelConfig,
        max_tokens: u32,
    ) -> Result<InferenceResult, String>;
    fn is_available(&self) -> bool;
    fn supported_models(&self) -> Vec<ModelConfig>;
}

#[cfg(test)]
pub struct StubLlmProvider;

#[cfg(test)]
impl StubLlmProvider {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
impl Default for StubLlmProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl LlmProvider for StubLlmProvider {
    fn infer(
        &self,
        messages: &[ChatMessage],
        config: &ModelConfig,
        _max_tokens: u32,
    ) -> Result<InferenceResult, String> {
        let content = messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default();
        Ok(InferenceResult {
            content: format!(
                "[STUB] LLM response to: {}...",
                &content.chars().take(100).collect::<String>()
            ),
            tokens_used: content.split_whitespace().count() as u32,
            model: config.name.clone(),
            duration_ms: 0,
            finish_reason: "stop".into(),
        })
    }

    fn is_available(&self) -> bool {
        true
    }

    fn supported_models(&self) -> Vec<ModelConfig> {
        vec![ModelConfig {
            name: "stub-code-review".into(),
            parameter_count: 7_000_000_000,
            context_window: 16384,
            max_tokens: 4096,
            endpoint: None,
            quantization: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stub_config() -> ModelConfig {
        ModelConfig {
            name: "test-model".into(),
            parameter_count: 7_000_000_000,
            context_window: 8192,
            max_tokens: 2048,
            endpoint: Some("http://localhost:8080".into()),
            quantization: Some("q4_0".into()),
        }
    }

    #[test]
    fn test_stub_provider_new() {
        let _provider = StubLlmProvider::new();
    }

    #[test]
    fn test_stub_provider_default_trait() {
        let _provider: Box<dyn LlmProvider> = Box::new(StubLlmProvider);
        assert!(_provider.is_available());
    }

    #[test]
    fn test_stub_provider_infer() {
        let provider = StubLlmProvider::new();
        let messages = vec![ChatMessage {
            role: "user".into(),
            content: "Review this code for bugs".into(),
        }];
        let result = provider.infer(&messages, &stub_config(), 1024).unwrap();
        assert!(result.content.contains("[STUB]"));
        assert!(result.content.contains("Review this code"));
    }

    #[test]
    fn test_stub_provider_infer_empty_messages() {
        let provider = StubLlmProvider::new();
        let result = provider.infer(&[], &stub_config(), 512).unwrap();
        assert!(result.content.contains("[STUB]"));
        assert_eq!(result.tokens_used, 0);
    }

    #[test]
    fn test_stub_provider_infer_single_message() {
        let provider = StubLlmProvider::new();
        let messages = vec![ChatMessage {
            role: "system".into(),
            content: "You are a helpful assistant.".into(),
        }];
        let result = provider.infer(&messages, &stub_config(), 256).unwrap();
        assert!(result.content.contains("[STUB]"));
        assert!(result.content.contains("helpful assistant"));
    }

    #[test]
    fn test_stub_provider_infer_multiple_messages() {
        let provider = StubLlmProvider::new();
        let messages = vec![
            ChatMessage {
                role: "system".into(),
                content: "Be concise.".into(),
            },
            ChatMessage {
                role: "user".into(),
                content: "What is 2+2?".into(),
            },
        ];
        let result = provider.infer(&messages, &stub_config(), 128).unwrap();
        assert!(result.content.contains("What is 2+2?"));
    }

    #[test]
    fn test_stub_provider_is_available() {
        let provider = StubLlmProvider::new();
        assert!(provider.is_available());
    }

    #[test]
    fn test_stub_provider_supported_models() {
        let provider = StubLlmProvider::new();
        let models = provider.supported_models();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "stub-code-review");
        assert_eq!(models[0].parameter_count, 7_000_000_000);
        assert_eq!(models[0].context_window, 16384);
        assert_eq!(models[0].max_tokens, 4096);
    }

    #[test]
    fn test_stub_provider_supported_model_details() {
        let provider = StubLlmProvider::new();
        let model = &provider.supported_models()[0];
        assert!(model.endpoint.is_none());
        assert!(model.quantization.is_none());
    }

    #[test]
    fn test_model_config_creation() {
        let config = stub_config();
        assert_eq!(config.name, "test-model");
        assert_eq!(config.parameter_count, 7_000_000_000);
        assert_eq!(config.context_window, 8192);
        assert_eq!(config.max_tokens, 2048);
    }

    #[test]
    fn test_model_config_optional_fields() {
        let config = ModelConfig {
            name: "minimal".into(),
            parameter_count: 0,
            context_window: 0,
            max_tokens: 0,
            endpoint: None,
            quantization: None,
        };
        assert!(config.endpoint.is_none());
        assert!(config.quantization.is_none());
    }

    #[test]
    fn test_chat_message_creation() {
        let msg = ChatMessage {
            role: "assistant".into(),
            content: "Hello, world!".into(),
        };
        assert_eq!(msg.role, "assistant");
        assert_eq!(msg.content, "Hello, world!");
    }

    #[test]
    fn test_inference_result_creation() {
        let result = InferenceResult {
            content: "response text".into(),
            tokens_used: 42,
            model: "gpt-4".into(),
            duration_ms: 150,
            finish_reason: "stop".into(),
        };
        assert_eq!(result.content, "response text");
        assert_eq!(result.tokens_used, 42);
        assert_eq!(result.model, "gpt-4");
        assert_eq!(result.duration_ms, 150);
        assert_eq!(result.finish_reason, "stop");
    }

    #[test]
    fn test_stub_provider_returns_stub_prefix() {
        let provider = StubLlmProvider::new();
        let messages = vec![ChatMessage {
            role: "user".into(),
            content: "test prompt".into(),
        }];
        let result = provider.infer(&messages, &stub_config(), 100).unwrap();
        assert!(result.content.starts_with("[STUB]"));
    }

    #[test]
    fn test_stub_provider_tokens_count() {
        let provider = StubLlmProvider::new();
        let messages = vec![ChatMessage {
            role: "user".into(),
            content: "one two three four five".into(),
        }];
        let result = provider.infer(&messages, &stub_config(), 100).unwrap();
        assert_eq!(result.tokens_used, 5);
    }

    #[test]
    fn test_stub_provider_model_name_in_result() {
        let provider = StubLlmProvider::new();
        let config = ModelConfig {
            name: "my-custom-model".into(),
            parameter_count: 13_000_000_000,
            context_window: 32768,
            max_tokens: 8192,
            endpoint: None,
            quantization: None,
        };
        let messages = vec![ChatMessage {
            role: "user".into(),
            content: "hi".into(),
        }];
        let result = provider.infer(&messages, &config, 100).unwrap();
        assert_eq!(result.model, "my-custom-model");
    }

    #[test]
    fn test_stub_provider_finish_reason() {
        let provider = StubLlmProvider::new();
        let messages = vec![ChatMessage {
            role: "user".into(),
            content: "test".into(),
        }];
        let result = provider.infer(&messages, &stub_config(), 100).unwrap();
        assert_eq!(result.finish_reason, "stop");
    }

    #[test]
    fn test_stub_provider_duration_ms() {
        let provider = StubLlmProvider::new();
        let messages = vec![ChatMessage {
            role: "user".into(),
            content: "test".into(),
        }];
        let result = provider.infer(&messages, &stub_config(), 100).unwrap();
        assert_eq!(result.duration_ms, 0);
    }

    #[test]
    fn test_model_config_serialization() {
        let config = stub_config();
        let json = serde_json::to_string(&config).unwrap();
        let de: ModelConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(de.name, "test-model");
        assert_eq!(de.parameter_count, 7_000_000_000);
        assert_eq!(de.endpoint.as_deref(), Some("http://localhost:8080"));
    }

    #[test]
    fn test_chat_message_serialization() {
        let msg = ChatMessage {
            role: "user".into(),
            content: "hello".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let de: ChatMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(de.role, "user");
        assert_eq!(de.content, "hello");
    }

    #[test]
    fn test_inference_result_serialization() {
        let result = InferenceResult {
            content: "test output".into(),
            tokens_used: 10,
            model: "test-model".into(),
            duration_ms: 50,
            finish_reason: "stop".into(),
        };
        let json = serde_json::to_string(&result).unwrap();
        let de: InferenceResult = serde_json::from_str(&json).unwrap();
        assert_eq!(de.content, "test output");
        assert_eq!(de.tokens_used, 10);
    }

    #[test]
    fn test_stub_provider_long_content_truncated() {
        let provider = StubLlmProvider::new();
        let long_content = "x".repeat(200);
        let messages = vec![ChatMessage {
            role: "user".into(),
            content: long_content.clone(),
        }];
        let result = provider.infer(&messages, &stub_config(), 100).unwrap();
        assert!(result.content.len() < 300);
    }
}
