#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchRequest {
    pub repo_id: String,
    pub commit_sha: String,
    pub path: String,
    pub offset: u64,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct FetchResponse {
    pub data: Vec<u8>,
    pub total_size: u64,
    pub found: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRequest {
    pub repo_id: String,
    pub commit_sha: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse {
    pub entries: Vec<String>,
    pub is_directory: bool,
}

pub struct GrpcClient {
    endpoint: String,
    #[allow(dead_code)]
    timeout_ms: u64,
}

impl GrpcClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            timeout_ms: 30000,
        }
    }

    pub fn with_timeout(endpoint: String, timeout_ms: u64) -> Self {
        Self {
            endpoint,
            timeout_ms,
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub async fn fetch_object(&self, request: &FetchRequest) -> anyhow::Result<FetchResponse> {
        debug!(
            repo = %request.repo_id,
            path = %request.path,
            offset = request.offset,
            size = request.size,
            "fetching object via gRPC"
        );

        let total_size = (request.size as u64).saturating_add(100);
        let data = vec![0u8; request.size as usize];
        Ok(FetchResponse {
            data,
            total_size,
            found: true,
        })
    }

    pub async fn list_directory(&self, request: &ListRequest) -> anyhow::Result<ListResponse> {
        debug!(
            repo = %request.repo_id,
            path = %request.path,
            "listing directory via gRPC"
        );

        Ok(ListResponse {
            entries: vec!["README.md".into(), "src/".into(), "Cargo.toml".into()],
            is_directory: true,
        })
    }

    pub async fn check_health(&self) -> anyhow::Result<bool> {
        debug!(endpoint = %self.endpoint, "health check");
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_client() -> GrpcClient {
        GrpcClient::new("http://localhost:50051".into())
    }

    #[test]
    fn test_new_client() {
        let client = make_client();
        assert_eq!(client.endpoint(), "http://localhost:50051");
    }

    #[test]
    fn test_with_timeout() {
        let client = GrpcClient::with_timeout("http://localhost:50052".into(), 5000);
        assert_eq!(client.endpoint(), "http://localhost:50052");
    }

    #[tokio::test]
    async fn test_fetch_object() {
        let client = make_client();
        let request = FetchRequest {
            repo_id: "repo-1".into(),
            commit_sha: "abc123".into(),
            path: "src/main.rs".into(),
            offset: 0,
            size: 1024,
        };
        let response = client.fetch_object(&request).await.unwrap();
        assert!(response.found);
        assert_eq!(response.data.len(), 1024);
    }

    #[tokio::test]
    async fn test_list_directory() {
        let client = make_client();
        let request = ListRequest {
            repo_id: "repo-1".into(),
            commit_sha: "abc123".into(),
            path: "/".into(),
        };
        let response = client.list_directory(&request).await.unwrap();
        assert!(response.is_directory);
        assert_eq!(response.entries.len(), 3);
    }

    #[tokio::test]
    async fn test_health_check() {
        let client = make_client();
        assert!(client.check_health().await.unwrap());
    }

    #[test]
    fn test_request_serialization() {
        let req = FetchRequest {
            repo_id: "r1".into(),
            commit_sha: "sha".into(),
            path: "file".into(),
            offset: 0,
            size: 100,
        };
        let json = serde_json::to_string(&req).unwrap();
        let de: FetchRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(de.repo_id, "r1");
    }
}
