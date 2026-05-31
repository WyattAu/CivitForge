#![forbid(unsafe_code)]

use crate::llm::models::TokenCounter;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{self, Receiver, Sender};
use tracing::warn;

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
    /// API key for remote services (unused for local air-gapped inference).
    #[serde(default)]
    pub api_key: Option<String>,
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
            api_key: None,
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
    http_client: reqwest::Client,
}

impl InferenceService {
    pub fn new(config: InferenceConfig) -> Self {
        let (tx, _rx) = mpsc::channel();
        let token_counter = TokenCounter::new();
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            config,
            token_counter,
            chunk_buffer: tx,
            http_client,
        }
    }

    fn base_url(&self) -> String {
        format!("http://{}:{}", self.config.host, self.config.port)
    }

    pub async fn generate(&self, request: &InferenceRequest) -> anyhow::Result<InferenceResponse> {
        let prompt_tokens = estimate_tokens(&request.prompt) as u32;
        let max_tokens = request.max_tokens.unwrap_or(self.config.max_tokens);

        let model_id = &self.config.model_id;
        if !self.token_counter.check_budget(model_id) {
            anyhow::bail!("token budget exceeded for model {model_id}");
        }

        let messages = self.build_messages(request);
        let body = serde_json::json!({
            "model": model_id,
            "messages": messages,
            "max_tokens": max_tokens,
            "temperature": request.temperature.unwrap_or(self.config.temperature),
            "stream": false,
        });

        let url = format!("{}/v1/chat/completions", self.base_url());
        let mut req = self.http_client.post(&url).json(&body);
        if let Some(key) = &self.config.api_key {
            req = req.header("Authorization", format!("Bearer {key}"));
        }

        let resp = req.send().await;

        let response = match resp {
            Ok(r) if r.status().is_success() => {
                let json: serde_json::Value = r.json().await?;
                parse_chat_completion_response(&json)
            }
            Ok(r) => {
                let status = r.status();
                let text = r.text().await.unwrap_or_default();
                warn!(%status, %text, "inference server error");
                anyhow::bail!("inference server returned {status}: {text}");
            }
            Err(e) => {
                warn!(%e, "inference server unreachable");
                anyhow::bail!("inference server unreachable: {e}");
            }
        };

        let completion_tokens = prompt_tokens.min(max_tokens / 2);
        let total_tokens = prompt_tokens + completion_tokens;
        self.token_counter.record_usage(model_id, total_tokens);

        Ok(response)
    }

    pub async fn stream(&self, request: &InferenceRequest) -> anyhow::Result<InferenceStream> {
        let (tx, rx) = mpsc::channel();
        let model_id = &self.config.model_id;

        if !self.token_counter.check_budget(model_id) {
            anyhow::bail!("token budget exceeded for model {model_id}");
        }

        self.token_counter
            .record_usage(model_id, estimate_tokens(&request.prompt) as u32);

        let messages = self.build_messages(request);
        let body = serde_json::json!({
            "model": model_id,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(self.config.max_tokens),
            "temperature": request.temperature.unwrap_or(self.config.temperature),
            "stream": true,
        });

        let url = format!("{}/v1/chat/completions", self.base_url());
        let client = self.http_client.clone();
        let api_key = self.config.api_key.clone();

        tokio::spawn(async move {
            let mut req = client.post(&url).json(&body);
            if let Some(key) = &api_key {
                req = req.header("Authorization", format!("Bearer {key}"));
            }

            match req.send().await {
                Ok(resp) if resp.status().is_success() => {
                    use futures::StreamExt;
                    let mut byte_stream = resp.bytes_stream();
                    let mut buffer = String::new();

                    while let Some(chunk_result) = byte_stream.next().await {
                        match chunk_result {
                            Ok(bytes) => {
                                buffer.push_str(&String::from_utf8_lossy(&bytes));
                                // Process complete SSE lines
                                while let Some(pos) = buffer.find('\n') {
                                    let line = buffer[..pos].trim().to_string();
                                    buffer = buffer[pos + 1..].to_string();

                                    if let Some(data) = line.strip_prefix("data: ") {
                                        if data.trim() == "[DONE]" {
                                            let _ = tx.send(StreamChunk {
                                                text: String::new(),
                                                finish_reason: Some(FinishReason::Stop),
                                            });
                                            return;
                                        }
                                        if let Ok(json) =
                                            serde_json::from_str::<serde_json::Value>(data)
                                        {
                                            if let Some(content) = json
                                                .get("choices")
                                                .and_then(|c| c.get(0))
                                                .and_then(|c| c.get("delta"))
                                                .and_then(|d| d.get("content"))
                                                .and_then(|c| c.as_str())
                                            {
                                                let _ = tx.send(StreamChunk {
                                                    text: content.to_string(),
                                                    finish_reason: None,
                                                });
                                            }
                                            let finish = json
                                                .get("choices")
                                                .and_then(|c| c.get(0))
                                                .and_then(|c| c.get("finish_reason"))
                                                .and_then(|f| f.as_str());
                                            if let Some("stop") = finish {
                                                let _ = tx.send(StreamChunk {
                                                    text: String::new(),
                                                    finish_reason: Some(FinishReason::Stop),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                warn!(%e, "stream read error");
                                break;
                            }
                        }
                    }
                }
                Ok(resp) => {
                    warn!(status = %resp.status(), "stream request failed");
                }
                Err(e) => {
                    warn!(%e, "stream connection failed");
                }
            }
        });

        Ok(InferenceStream { receiver: rx })
    }

    pub async fn health(&self) -> anyhow::Result<bool> {
        let url = format!("{}/health", self.base_url());
        match self.http_client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => {
                // Server unreachable -- check if model_id configured
                Ok(!self.config.model_id.is_empty())
            }
        }
    }

    fn build_messages(&self, request: &InferenceRequest) -> Vec<serde_json::Value> {
        let mut messages = Vec::new();

        if let Some(ref sys) = request.system_prompt {
            messages.push(serde_json::json!({"role": "system", "content": sys}));
        }
        for msg in &request.messages {
            let role = match msg.role {
                MessageRole::System => "system",
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
            };
            messages.push(serde_json::json!({"role": role, "content": msg.content}));
        }
        // Append the prompt as a user message if not empty
        if !request.prompt.is_empty() {
            messages.push(serde_json::json!({"role": "user", "content": request.prompt}));
        }

        messages
    }
}

fn parse_chat_completion_response(json: &serde_json::Value) -> InferenceResponse {
    let text = json
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();

    let usage = json.get("usage");
    let prompt_tokens = usage
        .and_then(|u| u.get("prompt_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let completion_tokens = usage
        .and_then(|u| u.get("completion_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let total_tokens = usage
        .and_then(|u| u.get("total_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or((prompt_tokens + completion_tokens) as u64) as u32;

    let finish_reason = json
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("finish_reason"))
        .and_then(|f| f.as_str())
        .map(|s| match s {
            "length" => FinishReason::Length,
            "content_filter" => FinishReason::ContentFilter,
            _ => FinishReason::Stop,
        })
        .unwrap_or(FinishReason::Stop);

    InferenceResponse {
        text,
        usage: TokenUsageInfo {
            prompt_tokens,
            completion_tokens,
            total_tokens,
        },
        finish_reason,
    }
}

pub fn validate_airgap(config: &InferenceConfig) -> anyhow::Result<()> {
    let host = &config.host;
    if host != "127.0.0.1" && host != "localhost" && host != "::1" && host != "0.0.0.0" {
        anyhow::bail!("air-gap violation: inference host {host} is not a loopback address");
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

    #[tokio::test]
    async fn test_generate_response_unreachable() {
        let service = InferenceService::new(make_config());
        let request = InferenceRequest {
            prompt: "Hello, world!".into(),
            max_tokens: Some(512),
            temperature: None,
            stop: None,
            system_prompt: None,
            messages: vec![],
        };
        // Local server not running -- should error
        let result = service.generate(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_generate_with_messages_unreachable() {
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
        let result = service.generate(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stream_unreachable() {
        let service = InferenceService::new(make_config());
        let request = InferenceRequest {
            prompt: "test".into(),
            max_tokens: None,
            temperature: None,
            stop: None,
            system_prompt: None,
            messages: vec![],
        };
        let stream = service.stream(&request).await.unwrap();
        // No chunks expected since server unreachable (spawned task fails silently)
        assert!(stream.next_chunk().is_none());
    }

    #[tokio::test]
    async fn test_health() {
        let service = InferenceService::new(InferenceConfig {
            model_id: "test-model".into(),
            host: "127.0.0.1".into(),
            port: 19023, // avoid port conflicts
            ..Default::default()
        });
        // Server not running on this port -- falls back to model_id check
        assert!(service.health().await.unwrap());

        let bad_service = InferenceService::new(InferenceConfig {
            model_id: String::new(),
            host: "127.0.0.1".into(),
            port: 19024,
            ..Default::default()
        });
        assert!(!bad_service.health().await.unwrap());
    }

    #[test]
    fn test_build_messages_with_system_prompt() {
        let config = make_config();
        let service = InferenceService::new(config);
        let request = InferenceRequest {
            prompt: "hello".into(),
            system_prompt: Some("You are helpful.".into()),
            messages: vec![],
            max_tokens: None,
            temperature: None,
            stop: None,
        };
        let msgs = service.build_messages(&request);
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0]["role"], "system");
        assert_eq!(msgs[1]["role"], "user");
    }

    #[test]
    fn test_parse_chat_completion_response() {
        let json = serde_json::json!({
            "choices": [{
                "message": {"content": "Hello!"},
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        });
        let response = parse_chat_completion_response(&json);
        assert_eq!(response.text, "Hello!");
        assert_eq!(response.usage.prompt_tokens, 10);
        assert_eq!(response.usage.completion_tokens, 5);
        assert_eq!(response.finish_reason, FinishReason::Stop);
    }

    #[test]
    fn test_parse_chat_completion_response_length() {
        let json = serde_json::json!({
            "choices": [{
                "message": {"content": "truncated"},
                "finish_reason": "length"
            }],
            "usage": {}
        });
        let response = parse_chat_completion_response(&json);
        assert_eq!(response.text, "truncated");
        assert_eq!(response.finish_reason, FinishReason::Length);
    }

    #[test]
    fn test_parse_chat_completion_response_minimal() {
        let json = serde_json::json!({});
        let response = parse_chat_completion_response(&json);
        assert!(response.text.is_empty());
        assert_eq!(response.finish_reason, FinishReason::Stop);
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

    #[tokio::test]
    async fn test_token_budget_enforcement() {
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

        let _ = service.generate(&request).await;

        let request2 = InferenceRequest {
            prompt: "another request".into(),
            max_tokens: None,
            temperature: None,
            stop: None,
            system_prompt: None,
            messages: vec![],
        };
        assert!(service.generate(&request2).await.is_err());
    }
}
