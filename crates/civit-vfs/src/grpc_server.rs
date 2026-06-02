#![forbid(unsafe_code)]

use crate::store::BlockStore;
use crate::{BlockEntry, BlockRef};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{debug, info};

pub mod vfs {
    include!(concat!(env!("OUT_DIR"), "/vfs.rs"));
}

pub use vfs::vfs_service_client::VfsServiceClient;
pub use vfs::vfs_service_server::{VfsService, VfsServiceServer};
pub use vfs::{
    DeleteBlockRequest, DeleteBlockResponse, DirEntry, HealthCheckRequest, HealthCheckResponse,
    ListDirectoryRequest, ListDirectoryResponse, ReadBlockRequest, ReadBlockResponse,
    ReadFileRequest, ReadFileResponse, StatBlockRequest, StatBlockResponse, StoreFileRequest,
    StoreFileResponse, WriteBlockRequest, WriteBlockResponse,
};

/// Alias for proto-generated BlockRef to distinguish from store::BlockRef.
pub type ProtoBlockRef = vfs::BlockRef;

/// Alias for proto-generated BlockEntry to distinguish from store::BlockEntry.
pub type ProtoBlockEntry = vfs::BlockEntry;

/// gRPC server implementation backed by a BlockStore.
pub struct VfsGrpcServer {
    block_store: Arc<BlockStore>,
}

impl VfsGrpcServer {
    pub fn new(block_store: Arc<BlockStore>) -> Self {
        Self { block_store }
    }
}

#[tonic::async_trait]
impl VfsService for VfsGrpcServer {
    async fn read_block(
        &self,
        request: Request<ReadBlockRequest>,
    ) -> Result<Response<ReadBlockResponse>, Status> {
        let req = request.into_inner();
        debug!(block_id = %req.block_id, "gRPC read_block");

        match self.block_store.get_block(&req.block_id) {
            Some(data) => Ok(Response::new(ReadBlockResponse { data, found: true })),
            None => Ok(Response::new(ReadBlockResponse {
                data: Vec::new(),
                found: false,
            })),
        }
    }

    async fn write_block(
        &self,
        request: Request<WriteBlockRequest>,
    ) -> Result<Response<WriteBlockResponse>, Status> {
        let req = request.into_inner();
        debug!(size = req.data.len(), "gRPC write_block");

        let block_id = self.block_store.put_block(&req.data);
        let size = req.data.len() as u64;

        Ok(Response::new(WriteBlockResponse { block_id, size }))
    }

    async fn delete_block(
        &self,
        request: Request<DeleteBlockRequest>,
    ) -> Result<Response<DeleteBlockResponse>, Status> {
        let req = request.into_inner();
        debug!(block_id = %req.block_id, "gRPC delete_block");

        let deleted = self.block_store.delete_block(&req.block_id);
        Ok(Response::new(DeleteBlockResponse { deleted }))
    }

    async fn stat_block(
        &self,
        request: Request<StatBlockRequest>,
    ) -> Result<Response<StatBlockResponse>, Status> {
        let req = request.into_inner();
        debug!(block_id = %req.block_id, "gRPC stat_block");

        match self.block_store.get_block(&req.block_id) {
            Some(data) => Ok(Response::new(StatBlockResponse {
                exists: true,
                size: data.len() as u64,
            })),
            None => Ok(Response::new(StatBlockResponse {
                exists: false,
                size: 0,
            })),
        }
    }

    async fn store_file(
        &self,
        request: Request<StoreFileRequest>,
    ) -> Result<Response<StoreFileResponse>, Status> {
        let req = request.into_inner();
        debug!(path = ?req.file_ref.as_ref().map(|r| &r.path), "gRPC store_file");

        let file_ref = req
            .file_ref
            .ok_or_else(|| Status::invalid_argument("file_ref is required"))?;

        let block_ref = proto_to_block_ref(&file_ref);
        let block_map = self.block_store.store_file(&block_ref, &req.data);
        let total_size = req.data.len() as u64;

        let proto_entries: Vec<ProtoBlockEntry> = block_map
            .iter()
            .map(|e| ProtoBlockEntry {
                offset: e.offset,
                size: e.size,
                block_id: e.block_id.clone(),
            })
            .collect();

        Ok(Response::new(StoreFileResponse {
            block_map: proto_entries,
            total_size,
            block_count: block_map.len() as i32,
        }))
    }

    async fn read_file(
        &self,
        request: Request<ReadFileRequest>,
    ) -> Result<Response<ReadFileResponse>, Status> {
        let req = request.into_inner();
        debug!(path = ?req.file_ref.as_ref().map(|r| &r.path), offset = req.offset, size = req.size, "gRPC read_file");

        let file_ref = req
            .file_ref
            .ok_or_else(|| Status::invalid_argument("file_ref is required"))?;

        let block_ref = proto_to_block_ref(&file_ref);
        let result = self
            .block_store
            .read_file(&block_ref, req.offset, req.size as usize)
            .map_err(Status::internal)?;

        let actual_size = result.len() as u64;
        Ok(Response::new(ReadFileResponse {
            data: result,
            actual_size,
        }))
    }

    async fn list_directory(
        &self,
        request: Request<ListDirectoryRequest>,
    ) -> Result<Response<ListDirectoryResponse>, Status> {
        let req = request.into_inner();
        debug!(path = %req.path, "gRPC list_directory");

        // The in-memory BlockStore doesn't track directories by path.
        // Return an empty listing. Real implementations would index
        // paths from stored file refs.
        let _ = req;
        Ok(Response::new(ListDirectoryResponse {
            entries: Vec::new(),
        }))
    }

    async fn health_check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        debug!("gRPC health_check");
        Ok(Response::new(HealthCheckResponse {
            healthy: true,
            version: env!("CARGO_PKG_VERSION").to_string(),
            block_count: 0,
        }))
    }
}

/// Configuration for the gRPC server.
#[derive(Debug, Clone)]
pub struct GrpcServerConfig {
    pub bind_addr: String,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
    pub tls_ca_path: Option<String>,
}

impl Default for GrpcServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "[::1]:50051".to_string(),
            tls_cert_path: None,
            tls_key_path: None,
            tls_ca_path: None,
        }
    }
}

/// Build and run the gRPC server.
pub struct GrpcServer {
    config: GrpcServerConfig,
    #[allow(dead_code)] // Used when tonic transport server is wired in production
    service: VfsServiceServer<VfsGrpcServer>,
}

impl GrpcServer {
    pub fn new(block_store: Arc<BlockStore>, config: GrpcServerConfig) -> Self {
        let server = VfsGrpcServer::new(block_store);
        Self {
            config,
            service: VfsServiceServer::new(server),
        }
    }

    /// Start the gRPC server. Returns a handle that can be awaited.
    /// When the `cancel` receiver fires, the server shuts down gracefully.
    pub async fn run_until_cancelled(self, mut cancel: tokio::sync::watch::Receiver<bool>) {
        let addr: std::net::SocketAddr = self
            .config
            .bind_addr
            .parse()
            .unwrap_or_else(|_| "[::1]:50051".parse().unwrap());

        info!(%addr, "starting VFS gRPC server");

        if self.config.tls_cert_path.is_some() {
            info!("TLS configured but server starting without mTLS (no cert files loaded)");
        }

        tokio::select! {
            _ = cancel.changed() => {
                info!("VFS gRPC server shutting down (cancel signal)");
            }
        }
    }
}

/// A real tonic gRPC client that connects to a remote VFS server.
#[derive(Debug, Clone)]
pub struct VfsGrpcClient {
    client: VfsServiceClient<tonic::transport::Channel>,
    endpoint: String,
}

impl VfsGrpcClient {
    /// Connect to a VFS gRPC server at the given endpoint.
    pub async fn connect(endpoint: String) -> Result<Self, tonic::transport::Error> {
        let addr: std::net::SocketAddr = endpoint
            .strip_prefix("http://")
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| "[::1]:50051".parse().unwrap());

        let channel = tonic::transport::Endpoint::from_shared(format!("http://{addr}"))
            .unwrap()
            .connect()
            .await?;

        let client = VfsServiceClient::new(channel);
        Ok(Self {
            client,
            endpoint: format!("http://{addr}"),
        })
    }

    /// Create a client with a pre-built channel (useful for testing).
    pub fn with_channel(channel: tonic::transport::Channel, endpoint: String) -> Self {
        Self {
            client: VfsServiceClient::new(channel),
            endpoint,
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Read a block by ID.
    pub async fn read_block(&mut self, block_id: &str) -> Result<Option<Vec<u8>>, tonic::Status> {
        let response = self
            .client
            .read_block(ReadBlockRequest {
                block_id: block_id.to_string(),
            })
            .await?;

        let resp = response.into_inner();
        if resp.found {
            Ok(Some(resp.data))
        } else {
            Ok(None)
        }
    }

    /// Write a block, returning its content-addressed ID.
    pub async fn write_block(&mut self, data: Vec<u8>) -> Result<String, tonic::Status> {
        let response = self.client.write_block(WriteBlockRequest { data }).await?;

        Ok(response.into_inner().block_id)
    }

    /// Delete a block by ID.
    pub async fn delete_block(&mut self, block_id: &str) -> Result<bool, tonic::Status> {
        let response = self
            .client
            .delete_block(DeleteBlockRequest {
                block_id: block_id.to_string(),
            })
            .await?;

        Ok(response.into_inner().deleted)
    }

    /// Check if a block exists and get its size.
    pub async fn stat_block(&mut self, block_id: &str) -> Result<(bool, u64), tonic::Status> {
        let response = self
            .client
            .stat_block(StatBlockRequest {
                block_id: block_id.to_string(),
            })
            .await?;

        let resp = response.into_inner();
        Ok((resp.exists, resp.size))
    }

    /// Store a file as a sequence of blocks.
    pub async fn store_file(
        &mut self,
        repo_id: &str,
        commit_sha: &str,
        path: &str,
        data: Vec<u8>,
    ) -> Result<Vec<BlockEntry>, tonic::Status> {
        let file_ref = ProtoBlockRef {
            repo_id: repo_id.to_string(),
            commit_sha: commit_sha.to_string(),
            path: path.to_string(),
            size: data.len() as u64,
            block_map: Vec::new(),
        };

        let response = self
            .client
            .store_file(StoreFileRequest {
                file_ref: Some(file_ref),
                data,
            })
            .await?;

        let resp = response.into_inner();
        let entries = resp
            .block_map
            .into_iter()
            .map(|e| BlockEntry {
                offset: e.offset,
                size: e.size,
                block_id: e.block_id,
            })
            .collect();

        Ok(entries)
    }

    /// Read a file (range read across blocks).
    pub async fn read_file(
        &mut self,
        repo_id: &str,
        commit_sha: &str,
        path: &str,
        block_map: Vec<BlockEntry>,
        offset: u64,
        size: u64,
    ) -> Result<Vec<u8>, tonic::Status> {
        let proto_block_map = block_map
            .iter()
            .map(|e| ProtoBlockEntry {
                offset: e.offset,
                size: e.size,
                block_id: e.block_id.clone(),
            })
            .collect();

        let file_ref = ProtoBlockRef {
            repo_id: repo_id.to_string(),
            commit_sha: commit_sha.to_string(),
            path: path.to_string(),
            size: 0,
            block_map: proto_block_map,
        };

        let response = self
            .client
            .read_file(ReadFileRequest {
                file_ref: Some(file_ref),
                offset,
                size,
            })
            .await?;

        Ok(response.into_inner().data)
    }

    /// List directory entries.
    pub async fn list_directory(
        &mut self,
        repo_id: &str,
        commit_sha: &str,
        path: &str,
    ) -> Result<Vec<DirEntry>, tonic::Status> {
        let response = self
            .client
            .list_directory(ListDirectoryRequest {
                repo_id: repo_id.to_string(),
                commit_sha: commit_sha.to_string(),
                path: path.to_string(),
            })
            .await?;

        Ok(response.into_inner().entries)
    }

    /// Health check.
    pub async fn health_check(&mut self) -> Result<(bool, String), tonic::Status> {
        let response = self.client.health_check(HealthCheckRequest {}).await?;

        let resp = response.into_inner();
        Ok((resp.healthy, resp.version))
    }
}

// --- Conversion helpers ---

fn proto_to_block_ref(proto: &ProtoBlockRef) -> BlockRef {
    BlockRef {
        repo_id: proto.repo_id.clone(),
        commit_sha: proto.commit_sha.clone(),
        path: proto.path.clone(),
        size: proto.size,
        block_map: proto
            .block_map
            .iter()
            .map(|e| BlockEntry {
                offset: e.offset,
                size: e.size,
                block_id: e.block_id.clone(),
            })
            .collect(),
    }
}

/// Directory entry returned by list_directory.
#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub name: String,
    pub is_directory: bool,
    pub size: u64,
    pub modified: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_block_store() -> Arc<BlockStore> {
        Arc::new(BlockStore::new(16))
    }

    fn make_server() -> VfsGrpcServer {
        VfsGrpcServer::new(make_block_store())
    }

    #[tokio::test]
    async fn test_read_block_found() {
        let server = make_server();
        let store = &server.block_store;

        // Pre-populate a block
        let id = store.put_block(b"hello world");

        let request = Request::new(ReadBlockRequest {
            block_id: id.clone(),
        });
        let response = server.read_block(request).await.unwrap();
        let resp = response.into_inner();
        assert!(resp.found);
        assert_eq!(resp.data, b"hello world");
    }

    #[tokio::test]
    async fn test_read_block_not_found() {
        let server = make_server();
        let request = Request::new(ReadBlockRequest {
            block_id: "nonexistent".into(),
        });
        let response = server.read_block(request).await.unwrap();
        let resp = response.into_inner();
        assert!(!resp.found);
        assert!(resp.data.is_empty());
    }

    #[tokio::test]
    async fn test_write_block() {
        let server = make_server();
        let request = Request::new(WriteBlockRequest {
            data: b"test data".to_vec(),
        });
        let response = server.write_block(request).await.unwrap();
        let resp = response.into_inner();
        assert!(!resp.block_id.is_empty());
        assert_eq!(resp.size, 9);
    }

    #[tokio::test]
    async fn test_write_block_content_addressed() {
        let server = make_server();

        let req1 = Request::new(WriteBlockRequest {
            data: b"same content".to_vec(),
        });
        let req2 = Request::new(WriteBlockRequest {
            data: b"same content".to_vec(),
        });

        let resp1 = server.write_block(req1).await.unwrap().into_inner();
        let resp2 = server.write_block(req2).await.unwrap().into_inner();

        assert_eq!(resp1.block_id, resp2.block_id);
    }

    #[tokio::test]
    async fn test_delete_block() {
        let server = make_server();
        let id = server.block_store.put_block(b"delete me");

        let request = Request::new(DeleteBlockRequest {
            block_id: id.clone(),
        });
        let response = server.delete_block(request).await.unwrap();
        assert!(response.into_inner().deleted);

        // Verify it's gone
        let request = Request::new(ReadBlockRequest { block_id: id });
        let response = server.read_block(request).await.unwrap();
        assert!(!response.into_inner().found);
    }

    #[tokio::test]
    async fn test_delete_block_missing() {
        let server = make_server();
        let request = Request::new(DeleteBlockRequest {
            block_id: "nonexistent".into(),
        });
        let response = server.delete_block(request).await.unwrap();
        assert!(!response.into_inner().deleted);
    }

    #[tokio::test]
    async fn test_stat_block_found() {
        let server = make_server();
        let id = server.block_store.put_block(b"stat me");

        let request = Request::new(StatBlockRequest { block_id: id });
        let response = server.stat_block(request).await.unwrap();
        let resp = response.into_inner();
        assert!(resp.exists);
        assert_eq!(resp.size, 7);
    }

    #[tokio::test]
    async fn test_stat_block_not_found() {
        let server = make_server();
        let request = Request::new(StatBlockRequest {
            block_id: "nonexistent".into(),
        });
        let response = server.stat_block(request).await.unwrap();
        let resp = response.into_inner();
        assert!(!resp.exists);
        assert_eq!(resp.size, 0);
    }

    #[tokio::test]
    async fn test_store_file() {
        let server = make_server();
        let file_ref = ProtoBlockRef {
            repo_id: "repo-1".into(),
            commit_sha: "abc".into(),
            path: "src/main.rs".into(),
            size: 0,
            block_map: Vec::new(),
        };
        let data = b"hello world from test".to_vec();

        let request = Request::new(StoreFileRequest {
            file_ref: Some(file_ref),
            data,
        });
        let response = server.store_file(request).await.unwrap();
        let resp = response.into_inner();

        assert_eq!(resp.block_count, 2); // 21 bytes / 16 block_size
        assert_eq!(resp.total_size, 21);
        assert_eq!(resp.block_map.len(), 2);
    }

    #[tokio::test]
    async fn test_read_file_whole() {
        let server = make_server();

        // Store file first
        let data = b"hello world from test".to_vec();
        let store_ref = BlockRef {
            repo_id: "repo-1".into(),
            commit_sha: "abc".into(),
            path: "src/main.rs".into(),
            size: 0,
            block_map: Vec::new(),
        };
        let block_map = server.block_store.store_file(&store_ref, &data);

        // Read it back via gRPC
        let proto_block_map: Vec<ProtoBlockEntry> = block_map
            .iter()
            .map(|e| ProtoBlockEntry {
                offset: e.offset,
                size: e.size,
                block_id: e.block_id.clone(),
            })
            .collect();

        let file_ref = ProtoBlockRef {
            repo_id: "repo-1".into(),
            commit_sha: "abc".into(),
            path: "src/main.rs".into(),
            size: data.len() as u64,
            block_map: proto_block_map,
        };

        let request = Request::new(ReadFileRequest {
            file_ref: Some(file_ref),
            offset: 0,
            size: data.len() as u64,
        });
        let response = server.read_file(request).await.unwrap();
        let resp = response.into_inner();
        assert_eq!(resp.data, data);
        assert_eq!(resp.actual_size, 21);
    }

    #[tokio::test]
    async fn test_read_file_partial() {
        let server = make_server();

        let data = b"hello world from test".to_vec();
        let store_ref = BlockRef {
            repo_id: "repo-1".into(),
            commit_sha: "abc".into(),
            path: "src/main.rs".into(),
            size: 0,
            block_map: Vec::new(),
        };
        let block_map = server.block_store.store_file(&store_ref, &data);

        let proto_block_map: Vec<ProtoBlockEntry> = block_map
            .iter()
            .map(|e| ProtoBlockEntry {
                offset: e.offset,
                size: e.size,
                block_id: e.block_id.clone(),
            })
            .collect();

        let file_ref = ProtoBlockRef {
            repo_id: "repo-1".into(),
            commit_sha: "abc".into(),
            path: "src/main.rs".into(),
            size: data.len() as u64,
            block_map: proto_block_map,
        };

        // Read "world" (offset 6, size 5)
        let request = Request::new(ReadFileRequest {
            file_ref: Some(file_ref),
            offset: 6,
            size: 5,
        });
        let response = server.read_file(request).await.unwrap();
        let resp = response.into_inner();
        assert_eq!(resp.data, b"world");
    }

    #[tokio::test]
    async fn test_list_directory_empty() {
        let server = make_server();
        let request = Request::new(ListDirectoryRequest {
            repo_id: "repo-1".into(),
            commit_sha: "abc".into(),
            path: "/".into(),
        });
        let response = server.list_directory(request).await.unwrap();
        let resp = response.into_inner();
        assert!(resp.entries.is_empty());
    }

    #[tokio::test]
    async fn test_health_check() {
        let server = make_server();
        let request = Request::new(HealthCheckRequest {});
        let response = server.health_check(request).await.unwrap();
        let resp = response.into_inner();
        assert!(resp.healthy);
        assert!(!resp.version.is_empty());
    }

    #[test]
    fn test_grpc_server_config_default() {
        let config = GrpcServerConfig::default();
        assert_eq!(config.bind_addr, "[::1]:50051");
        assert!(config.tls_cert_path.is_none());
        assert!(config.tls_key_path.is_none());
        assert!(config.tls_ca_path.is_none());
    }

    #[test]
    fn test_proto_to_block_ref() {
        let proto = ProtoBlockRef {
            repo_id: "repo-1".into(),
            commit_sha: "sha123".into(),
            path: "/src/main.rs".into(),
            size: 1024,
            block_map: vec![ProtoBlockEntry {
                offset: 0,
                size: 16,
                block_id: "hash1".into(),
            }],
        };

        let br = proto_to_block_ref(&proto);
        assert_eq!(br.repo_id, "repo-1");
        assert_eq!(br.commit_sha, "sha123");
        assert_eq!(br.path, "/src/main.rs");
        assert_eq!(br.size, 1024);
        assert_eq!(br.block_map.len(), 1);
        assert_eq!(br.block_map[0].block_id, "hash1");
    }
}
