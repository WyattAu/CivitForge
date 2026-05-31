#![forbid(unsafe_code)]

use crate::llm::models::TokenCounter;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub host: String,
    pub port: u16,
    pub model_id: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: u32,
    pub repeat_penalty: f32,
    pub gpu_enabled: bool,
    pub gpu_device: Option<String>,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 8080,
            model_id: String::new(),
            max_tokens: 2048,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            repeat_penalty: 1.1,
            gpu_enabled: false,
            gpu_device: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub prompt: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stop: Option<Vec<String>>,
    pub system_prompt: Option<String>,
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageInfo {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub text: String,
    pub usage: TokenUsageInfo,
    pub finish_reason: FinishReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub text: String,
    pub finish_reason: Option<FinishReason>,
}

pub struct InferenceStream {
    receiver: Receiver<StreamChunk>,
}

impl InferenceStream {
    pub fn next_chunk(&self) -> Option<StreamChunk> {
        self.receiver.try_recv().ok()
    }
}

pub struct InferenceService {
    pub config: InferenceConfig,
    pub token_counter: TokenCounter,
    #[allow(dead_code)]
    chunk_buffer: Sender<StreamChunk>,
}

impl InferenceService {
    pub fn new(config: InferenceConfig) -> Self {
        let (tx, _rx) = mpsc::channel();
        let token_counter = TokenCounter::new();
        Self {
            config,
            token_counter,
            chunk_buffer: tx,
        }
    }

    pub fn generate(&self, request: &InferenceRequest) -> anyhow::Result<InferenceResponse> {
        let prompt_tokens = estimate_tokens(&request.prompt) as u32;
        let max_tokens = request.max_tokens.unwrap_or(self.config.max_tokens);

        let model_id = &self.config.model_id;
        if !self.token_counter.check_budget(model_id) {
            anyhow::bail!("token budget exceeded for model {}", model_id);
        }

        let completion_tokens = prompt_tokens.min(max_tokens / 2);
        let total_tokens = prompt_tokens + completion_tokens;

        self.token_counter.record_usage(model_id, total_tokens);

        let text = request.prompt.clone();
        let finish_reason = if completion_tokens >= max_tokens {
            FinishReason::Length
        } else {
            FinishReason::Stop
        };

        Ok(InferenceResponse {
            text,
            usage: TokenUsageInfo {
                prompt_tokens,
                completion_tokens,
                total_tokens,
            },
            finish_reason,
        })
    }

    pub fn stream(&self, request: &InferenceRequest) -> anyhow::Result<InferenceStream> {
        let (tx, rx) = mpsc::channel();
        let _prompt_tokens = estimate_tokens(&request.prompt);
        let _ = self.token_counter.record_usage(
            &self.config.model_id,
            estimate_tokens(&request.prompt) as u32,
        );

        let _ = tx.send(StreamChunk {
            text: request.prompt.clone(),
            finish_reason: Some(FinishReason::Stop),
        });

        Ok(InferenceStream { receiver: rx })
    }

    pub fn health(&self) -> anyhow::Result<bool> {
        Ok(!self.config.model_id.is_empty())
    }
}

pub fn validate_airgap(config: &InferenceConfig) -> anyhow::Result<()> {
    let host = &config.host;
    if host != "127.0.0.1" && host != "localhost" && host != "::1" && host != "0.0.0.0" {
        anyhow::bail!(
            "air-gap violation: inference host {} is not a loopback address",
            host
        );
    }
    if config.port == 0 {
        anyhow::bail!("air-gap violation: port 0 is invalid");
    }
    Ok(())
}

fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    (text.len() as f32 / 4.0).ceil() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config() -> InferenceConfig {
        InferenceConfig {
            model_id: "test-model".into(),
            host: "127.0.0.1".into(),
            port: 8080,
            ..Default::default()
        }
    }

    #[test]
    fn test_default_config() {
        let config = InferenceConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.temperature, 0.7);
        assert!(!config.gpu_enabled);
    }

    #[test]
    fn test_generate_response() {
        let service = InferenceService::new(make_config());
        let request = InferenceRequest {
            prompt: "Hello, world!".into(),
            max_tokens: Some(512),
            temperature: None,
            stop: None,
            system_prompt: None,
            messages: vec![],
        };
        let response = service.generate(&request).unwrap();
        assert_eq!(response.text, "Hello, world!");
        assert_eq!(response.finish_reason, FinishReason::Stop);
    }

    #[test]
    fn test_generate_finish_reason_length() {
        let service = InferenceService::new(make_config());
        let request = InferenceRequest {
            prompt: String::new(),
            max_tokens: Some(0),
            temperature: None,
            stop: None,
            system_prompt: None,
            messages: vec![],
        };
        let response = service.generate(&request).unwrap();
        assert_eq!(response.finish_reason, FinishReason::Length);
    }

    #[test]
    fn test_generate_with_messages() {
        let service = InferenceService::new(make_config());
        let request = InferenceRequest {
            prompt: "Summarize".into(),
            max_tokens: None,
            temperature: None,
            stop: None,
            system_prompt: None,
            messages: vec![
                ChatMessage {
                    role: MessageRole::System,
                    content: "You are helpful.".into(),
                },
                ChatMessage {
                    role: MessageRole::User,
                    content: "Hello".into(),
                },
            ],
        };
        let response = service.generate(&request).unwrap();
        assert_eq!(response.text, "Summarize");
    }

    #[test]
    fn test_stream() {
        let service = InferenceService::new(make_config());
        let request = InferenceRequest {
            prompt: "test".into(),
            max_tokens: None,
            temperature: None,
            stop: None,
            system_prompt: None,
            messages: vec![],
        };
        let stream = service.stream(&request).unwrap();
        let chunk = stream.next_chunk().unwrap();
        assert_eq!(chunk.text, "test");
        assert_eq!(chunk.finish_reason, Some(FinishReason::Stop));
    }

    #[test]
    fn test_health() {
        let service = InferenceService::new(make_config());
        assert!(service.health().unwrap());

        let bad_service = InferenceService::new(InferenceConfig {
            model_id: String::new(),
            ..Default::default()
        });
        assert!(!bad_service.health().unwrap());
    }

    #[test]
    fn test_validate_airgap_loopback() {
        let config = make_config();
        assert!(validate_airgap(&config).is_ok());
    }

    #[test]
    fn test_validate_airgap_localhost() {
        let config = InferenceConfig {
            host: "localhost".into(),
            ..make_config()
        };
        assert!(validate_airgap(&config).is_ok());
    }

    #[test]
    fn test_validate_airgap_external_fails() {
        let config = InferenceConfig {
            host: "api.openai.com".into(),
            ..make_config()
        };
        assert!(validate_airgap(&config).is_err());
    }

    #[test]
    fn test_validate_airgap_zero_port_fails() {
        let config = InferenceConfig {
            port: 0,
            ..make_config()
        };
        assert!(validate_airgap(&config).is_err());
    }

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("hello"), 2);
        assert!(estimate_tokens(&"a".repeat(100)) >= 25);
    }

    #[test]
    fn test_token_budget_enforcement() {
        let config = make_config();
        let service = InferenceService::new(config);
        service.token_counter.register_budget(
            "test-model".into(),
            crate::llm::models::TokenBudget {
                max_tokens_per_request: 10,
                tokens_per_minute: 10,
                tokens_per_day: 1000,
            },
        );

        let request = InferenceRequest {
            prompt: "this is a test prompt that exceeds budget".into(),
            max_tokens: None,
            temperature: None,
            stop: None,
            system_prompt: None,
            messages: vec![],
        };

        let _ = service.generate(&request).unwrap();

        let request2 = InferenceRequest {
            prompt: "another request".into(),
            max_tokens: None,
            temperature: None,
            stop: None,
            system_prompt: None,
            messages: vec![],
        };
        assert!(service.generate(&request2).is_err());
    }
}
